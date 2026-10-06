/* adcs_drv.c -- drivers of the synthetic parts over the byte HAL (fsw/pseudocode/09).
 * The bus transactions, the UART receive buffers and the frames put on the buses stay here; the
 * decoding of what the buses brought and the command words delegate to their translation of
 * 09_drivers.pc (fsw/alg/src/drivers.c).
 * Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved. */
#include "adcs_drv.h"
#include "adcs_devices.h"
#include "adcs_hal.h"
#include "adcs_alg_glue.h"

static void wr16(uint8_t *b, int16_t v) { b[0] = (uint8_t)((uint16_t)v & 0xFFu); b[1] = (uint8_t)(((uint16_t)v >> 8) & 0xFFu); }

/* written from the design: drivers::crc16 (fsw/alg), which takes up to 64 bytes (every frame of the
 * parts); a longer block, which no part sends, is run through the same polynomial here */
uint16_t adcs_crc16(const uint8_t *p, int n)
{
    uint16_t c = 0xFFFFu; int i, k;
    if (n <= 64) {
        pc_a64i b = {{0}};
        for (i = 0; i < n; i++) b.v[i] = p[i];
        return (uint16_t)drivers_crc16(b, n);
    }
    for (i = 0; i < n; i++) {
        c ^= (uint16_t)((uint16_t)p[i] << 8);
        for (k = 0; k < 8; k++) c = (c & 0x8000u) ? (uint16_t)(((unsigned)c << 1) ^ 0x1021u) : (uint16_t)((unsigned)c << 1);
    }
    return c;
}

/* ---- UART frame assembly (one buffer per port) ---- */
#define RXCAP 256
static uint8_t rx_buf[3][RXCAP];
static int rx_len[3];

void adcs_drv_reset(void) { rx_len[0] = rx_len[1] = rx_len[2] = 0; }

/* pull available bytes; return the payload length of the newest complete frame with sync2, or -1.
 * The bytes of a frame not yet whole are kept for the next read (the design's frame search,
 * drivers::uart_frame, works on one read; this buffer is what carries a frame across reads). */
static int uart_frame(uint8_t port, uint8_t sync2, uint8_t *payload, int cap)
{
    uint8_t tmp[RXCAP];
    size_t got = 0;
    int found = -1, i;
    uint8_t *b = rx_buf[port];
    while (adcs_hal_uart_read(port, tmp, sizeof tmp, &got) == ADCS_OK && got > 0) {
        for (i = 0; i < (int)got; i++) {
            if (rx_len[port] >= RXCAP) rx_len[port] = 0;               /* overflow: resynchronise */
            b[rx_len[port]++] = tmp[i];
        }
        got = 0;
    }
    for (;;) {
        int s = 0, n;
        while (s + 1 < rx_len[port] && !(b[s] == ADCS_ST_SYNC1 && b[s + 1] == sync2)) s++;
        if (s + 3 > rx_len[port]) break;
        n = b[s + 2];
        if (s + 3 + n + 2 > rx_len[port]) { if (s > 0) { for (i = s; i < rx_len[port]; i++) b[i - s] = b[i]; rx_len[port] -= s; } break; }
        if (n <= cap && adcs_crc16(b + s + 3, n) == (uint16_t)(b[s + 3 + n] | (b[s + 4 + n] << 8))) {
            for (i = 0; i < n; i++) payload[i] = b[s + 3 + i];
            found = n;
        }
        for (i = s + 5 + n; i < rx_len[port]; i++) b[i - (s + 5 + n)] = b[i];
        rx_len[port] -= s + 5 + n;
    }
    return found;
}

/* the frame a UART brought (n >= 0 payload bytes, n <= 64), as the one read the design's decoding takes */
static int64_t frame_stream(pc_a96i *b, uint8_t sync2, const uint8_t *pl, int n)
{
    uint16_t c;
    int i;
    if (n < 0) return 0;
    c = adcs_crc16(pl, n);
    b->v[0] = ADCS_ST_SYNC1; b->v[1] = sync2; b->v[2] = n;
    for (i = 0; i < n; i++) b->v[3 + i] = pl[i];
    b->v[3 + n] = c & 0xFFu; b->v[4 + n] = c >> 8;
    return n + 5;
}

/* written from the design: drivers::drv_read (fsw/alg). The transactions are the HAL's: a transfer
 * that fails reads as a status 0 (no reading). A reading the read did not bring is left as it is
 * (the latest), as are the rotor momenta and gimbal angles no telemetry frame brought; the frames are
 * decoded 8 at a time, in the order they came. */
void adcs_drv_read(const adcs_params_t *p, adcs_meas_t *z)
{
    uint8_t tx[16] = {0}, rx[64], pl_st[64], pl_gps[64];
    pc_a7i mag = {{0}}, sun = {{0}}, es = {{0}};
    pc_a13i gyro = {{0}};
    pc_a96i st = {{0}}, gps = {{0}};
    pc_a8i can_id, can_dlc;
    pc_a8a8i can_data;
    int64_t st_len = 0, gps_len = 0;
    drivers_drv_read_out o;
    adcs_can_frame_t f;
    int i, k, n_st = -1, n_gps = -1, nf, more = 1, first = 1;
    /* magnetometer */
    tx[0] = 0x00;
    if (adcs_hal_i2c_xfer(ADCS_MAG_BUS, ADCS_MAG_ADDR, tx, 1, rx, 7) == ADCS_OK) for (i = 0; i < 7; i++) mag.v[i] = rx[i];
    /* gyro */
    if (p->has_gyro) {
        tx[0] = ADCS_GYRO_CMD;
        if (adcs_hal_spi_xfer(ADCS_GYRO_BUS, ADCS_GYRO_CS, tx, rx, 13) == ADCS_OK) for (i = 0; i < 13; i++) gyro.v[i] = rx[i];
    }
    /* Sun-sensor set, Earth sensor */
    tx[0] = 0x00;
    if (p->has_sun && adcs_hal_i2c_xfer(ADCS_SUN_BUS, ADCS_SUN_ADDR, tx, 1, rx, 7) == ADCS_OK) for (i = 0; i < 7; i++) sun.v[i] = rx[i];
    if (p->has_es && adcs_hal_i2c_xfer(ADCS_ES_BUS, ADCS_ES_ADDR, tx, 1, rx, 7) == ADCS_OK) for (i = 0; i < 7; i++) es.v[i] = rx[i];
    /* star tracker, GNSS */
    if (p->has_st) { n_st = uart_frame(ADCS_ST_UART, ADCS_ST_SYNC2, pl_st, sizeof pl_st); st_len = frame_stream(&st, ADCS_ST_SYNC2, pl_st, n_st); }
    if (p->has_gps) { n_gps = uart_frame(ADCS_GPS_UART, ADCS_GPS_SYNC2, pl_gps, sizeof pl_gps); gps_len = frame_stream(&gps, ADCS_GPS_SYNC2, pl_gps, n_gps); }
    /* momentum-device telemetry */
    while (first || more) {
        int h_set[ADCS_MAX_ROTORS] = {0}, d_set[ADCS_MAX_GIMBALS] = {0};
        nf = 0;
        while (more && nf < 8) {
            more = adcs_hal_can_recv(ADCS_CAN_PORT, &f) == ADCS_OK;
            if (!more) break;
            can_id.v[nf] = f.id; can_dlc.v[nf] = f.dlc;
            for (k = 0; k < 8; k++) can_data.v[nf].v[k] = f.data[k];
            if (f.id >= ADCS_CAN_ROTOR_TM && f.id < ADCS_CAN_ROTOR_TM + ADCS_MAX_ROTORS && f.dlc >= 8) {   /* a frame of the layout */
                int r = (int)(f.id - ADCS_CAN_ROTOR_TM);
                h_set[r] = 1;
                if (r < p->nr && p->rot_gi[r] > 0) d_set[p->rot_gi[r] - 1] = 1;
            }
            nf++;
        }
        if (!first && nf == 0) break;
        for (i = nf; i < 8; i++) { can_id.v[i] = 0; can_dlc.v[i] = 0; for (k = 0; k < 8; k++) can_data.v[i].v[k] = 0; }
        o = drivers_drv_read(mag, gyro, sun, es, st, st_len, gps, gps_len, can_id, can_dlc, can_data, nf,
                             p->has_gyro != 0, p->has_sun != 0, p->has_es != 0, p->has_st != 0, p->has_gps != 0, p->nr, gl_u8(p->rot_gi));
        if (first) {
            z->mag_ok = o.mag_ok; if (o.mag_ok) gl_o3(o.b, z->B);
            z->gyro_ok = o.gyro_ok; if (o.gyro_ok) gl_o3(o.w, z->w);
            z->sun_ok = o.sun_ok; if (o.sun_ok) gl_o3(o.sun, z->sun);
            z->es_ok = o.es_ok; if (o.es_ok) gl_o3(o.nadir, z->nadir);
            z->st_ok = o.st_ok;
            for (i = 0; i < ADCS_MAX_HEADS; i++) {
                z->st_valid[i] = o.st_valid.v[i];
                if (n_st > 0 && i < pl_st[0] && 1 + 17*(i + 1) <= n_st) gl_o4(o.q_st.v[i], z->q_st[i]);   /* the heads the frame holds */
            }
            z->gps_ok = o.gps_ok; if (o.gps_ok) { gl_o3(o.r, z->r); gl_o3(o.v, z->v); }
            first = 0;
        }
        for (i = 0; i < ADCS_MAX_ROTORS; i++) if (h_set[i]) z->h[i] = o.h.v[i];
        for (i = 0; i < ADCS_MAX_GIMBALS; i++) if (d_set[i]) z->delta[i] = o.delta.v[i];
        /* the batches after the first bring telemetry only */
        mag.v[0] = 0; gyro.v[0] = 0; sun.v[0] = 0; es.v[0] = 0; st_len = 0; gps_len = 0;
    }
}

/* written from the design: drivers::drv_write (fsw/alg), the command words; the frames are put on
 * the buses here. Every byte of a frame is set: the ones past dlc are 0, never whatever the stack
 * held (the C/Rust differential fuzzer found them leaking onto the bus). */
void adcs_drv_write(const adcs_params_t *p, const adcs_real m_body[3], const adcs_real cmd_r[ADCS_MAX_ROTORS],
                    const adcs_real cmd_g[ADCS_MAX_GIMBALS], const adcs_real duty[ADCS_MAX_COUPLES])
{
    adcs_can_frame_t f;
    pc_a6f du;
    drivers_drv_write_out o;
    int i, j, nc = p->nc < ADCS_MAX_COUPLES ? p->nc : ADCS_MAX_COUPLES;
    for (i = 0; i < ADCS_MAX_COUPLES; i++) du.v[i] = duty[i];
    o = drivers_drv_write(gl_v3(m_body), p->m_max, gl_v8(cmd_r), gl_v8(p->rot_tmax), p->nr, gl_v4(cmd_g), p->gim_rate_max, p->ng, du, nc, p->dt);
    for (j = 0; j < 8; j++) f.data[j] = 0;
    for (i = 0; i < 3; i++) adcs_hal_pwm_set((uint8_t)i, (int16_t)o.pwm.v[i]);
    for (i = 0; i < p->nr; i++) {
        f.id = ADCS_CAN_ROTOR_CMD + (uint32_t)i; f.extended = 0; f.dlc = 2;
        wr16(f.data, (int16_t)o.rot_words.v[i]);
        adcs_hal_can_send(ADCS_CAN_PORT, &f);
    }
    for (i = 0; i < p->ng; i++) {
        f.id = ADCS_CAN_GIMBAL_CMD + (uint32_t)i; f.extended = 0; f.dlc = 2;
        wr16(f.data, (int16_t)o.gim_words.v[i]);
        adcs_hal_can_send(ADCS_CAN_PORT, &f);
    }
    if (p->nc > 0) {
        f.id = ADCS_CAN_VALVES; f.extended = 0; f.dlc = 8;
        for (i = 0; i < 8; i++) f.data[i] = 0;
        for (i = 0; i < nc; i++) f.data[i] = (uint8_t)o.valves.v[i];      /* closed on a bad duty */
        adcs_hal_can_send(ADCS_CAN_PORT, &f);
    }
}
