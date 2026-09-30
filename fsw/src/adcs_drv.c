/* adcs_drv.c -- drivers of the synthetic parts over the byte HAL (fsw/pseudocode/09).
 * Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved. */
#include "adcs_drv.h"
#include "adcs_devices.h"
#include "adcs_hal.h"

static int16_t rd16(const uint8_t *b) { return (int16_t)((uint16_t)b[0] | ((uint16_t)b[1] << 8)); }
static int32_t rd32(const uint8_t *b) { return (int32_t)((uint32_t)b[0] | ((uint32_t)b[1] << 8) | ((uint32_t)b[2] << 16) | ((uint32_t)b[3] << 24)); }
static void wr16(uint8_t *b, int16_t v) { b[0] = (uint8_t)((uint16_t)v & 0xFFu); b[1] = (uint8_t)(((uint16_t)v >> 8) & 0xFFu); }

uint16_t adcs_crc16(const uint8_t *p, int n)
{
    uint16_t c = 0xFFFFu; int i, k;
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

/* pull available bytes; return the payload length of the newest complete frame with sync2, or -1 */
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
        if (adcs_crc16(b + s + 3, n) == (uint16_t)(b[s + 3 + n] | (b[s + 4 + n] << 8)) && n <= cap) {
            for (i = 0; i < n; i++) payload[i] = b[s + 3 + i];
            found = n;
        }
        for (i = s + 5 + n; i < rx_len[port]; i++) b[i - (s + 5 + n)] = b[i];
        rx_len[port] -= s + 5 + n;
    }
    return found;
}

void adcs_drv_read(const adcs_params_t *p, adcs_meas_t *z)
{
    uint8_t tx[16] = {0}, rx[64], pl[64];
    int i, n;
    adcs_can_frame_t f;
    /* magnetometer */
    tx[0] = 0x00;
    z->mag_ok = adcs_hal_i2c_xfer(ADCS_MAG_BUS, ADCS_MAG_ADDR, tx, 1, rx, 7) == ADCS_OK && (rx[0] & 1u);
    if (z->mag_ok) for (i = 0; i < 3; i++) z->B[i] = rd16(rx + 1 + 2*i)*ADCS_MAG_LSB_T;
    /* gyro */
    z->gyro_ok = 0;
    if (p->has_gyro) {
        tx[0] = ADCS_GYRO_CMD;
        if (adcs_hal_spi_xfer(ADCS_GYRO_BUS, ADCS_GYRO_CS, tx, rx, 13) == ADCS_OK && (rx[0] & 1u)) {
            z->gyro_ok = 1;
            for (i = 0; i < 3; i++) z->w[i] = rd32(rx + 1 + 4*i)*ADCS_GYRO_LSB;
        }
    }
    /* Sun-sensor set, Earth sensor */
    tx[0] = 0x00;
    z->sun_ok = p->has_sun && adcs_hal_i2c_xfer(ADCS_SUN_BUS, ADCS_SUN_ADDR, tx, 1, rx, 7) == ADCS_OK && (rx[0] & 1u);
    if (z->sun_ok) for (i = 0; i < 3; i++) z->sun[i] = rd16(rx + 1 + 2*i)/ADCS_Q15;
    z->es_ok = p->has_es && adcs_hal_i2c_xfer(ADCS_ES_BUS, ADCS_ES_ADDR, tx, 1, rx, 7) == ADCS_OK && (rx[0] & 1u);
    if (z->es_ok) for (i = 0; i < 3; i++) z->nadir[i] = rd16(rx + 1 + 2*i)/ADCS_Q15;
    /* star tracker */
    z->st_ok = 0;
    for (i = 0; i < ADCS_MAX_HEADS; i++) z->st_valid[i] = 0;
    if (p->has_st && (n = uart_frame(ADCS_ST_UART, ADCS_ST_SYNC2, pl, sizeof pl)) > 0) {
        int nh = pl[0], h, k;
        for (h = 0; h < nh && h < ADCS_MAX_HEADS && 1 + 17*(h + 1) <= n; h++) {
            const uint8_t *e = pl + 1 + 17*h;
            z->st_valid[h] = e[0] & 1u;
            for (k = 0; k < 4; k++) z->q_st[h][k] = rd32(e + 1 + 4*k)/ADCS_Q30;
            if (z->st_valid[h]) z->st_ok = 1;
        }
    }
    /* GNSS */
    z->gps_ok = 0;
    if (p->has_gps && (n = uart_frame(ADCS_GPS_UART, ADCS_GPS_SYNC2, pl, sizeof pl)) >= 25 && (pl[0] & 1u)) {
        z->gps_ok = 1;
        for (i = 0; i < 3; i++) { z->r[i] = rd32(pl + 1 + 4*i)*0.01; z->v[i] = rd32(pl + 13 + 4*i)*0.001; }
    }
    /* momentum-device telemetry */
    while (adcs_hal_can_recv(ADCS_CAN_PORT, &f) == ADCS_OK) {
        if (f.id >= ADCS_CAN_ROTOR_TM && f.id < ADCS_CAN_ROTOR_TM + ADCS_MAX_ROTORS && f.dlc >= 8) {
            int r = (int)(f.id - ADCS_CAN_ROTOR_TM);
            z->h[r] = rd32(f.data)*ADCS_H_LSB;
            if (r < p->nr && p->rot_gi[r] > 0) z->delta[p->rot_gi[r] - 1] = rd32(f.data + 4)*ADCS_DELTA_LSB;
        }
    }
}

/* A command that is not a finite number drives nothing: x - x is 0 only for a finite x, so a
 * NaN or an infinity from upstream becomes a zero output, never a cast of NaN (undefined in C). */
#define FINITE(x) ((x) - (x) == 0)

static int16_t q15(adcs_real x)
{
    adcs_real v = x*ADCS_Q15;
    if (!FINITE(v)) return 0;
    if (v > ADCS_Q15) v = ADCS_Q15;
    if (v < -ADCS_Q15) v = -ADCS_Q15;
    return (int16_t)(v < 0 ? v - 0.5 : v + 0.5);
}

void adcs_drv_write(const adcs_params_t *p, const adcs_real m_body[3], const adcs_real cmd_r[ADCS_MAX_ROTORS],
                    const adcs_real cmd_g[ADCS_MAX_GIMBALS], const adcs_real duty[ADCS_MAX_COUPLES])
{
    adcs_can_frame_t f;
    int i;
    for (i = 0; i < 3; i++) adcs_hal_pwm_set((uint8_t)i, q15(m_body[i]/p->m_max));
    for (i = 0; i < p->nr; i++) {
        f.id = ADCS_CAN_ROTOR_CMD + (uint32_t)i; f.extended = 0; f.dlc = 2;
        wr16(f.data, q15(cmd_r[i]/p->rot_tmax[i]));
        adcs_hal_can_send(ADCS_CAN_PORT, &f);
    }
    for (i = 0; i < p->ng; i++) {
        f.id = ADCS_CAN_GIMBAL_CMD + (uint32_t)i; f.extended = 0; f.dlc = 2;
        wr16(f.data, q15(cmd_g[i]/p->gim_rate_max));
        adcs_hal_can_send(ADCS_CAN_PORT, &f);
    }
    if (p->nc > 0) {
        f.id = ADCS_CAN_VALVES; f.extended = 0; f.dlc = 8;
        for (i = 0; i < 8; i++) f.data[i] = 0;
        for (i = 0; i < p->nc && i < ADCS_MAX_COUPLES; i++) {
            adcs_real ms = duty[i]*p->dt/ADCS_VALVE_LSB_S + 0.5;
            f.data[i] = (uint8_t)(!FINITE(ms) ? 0 : (ms > 255 ? 255 : (ms < 0 ? 0 : ms)));  /* closed on a bad duty */
        }
        adcs_hal_can_send(ADCS_CAN_PORT, &f);
    }
}
