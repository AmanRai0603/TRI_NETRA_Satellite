/* adcs_link.c -- the OBC side of adcs-link/1: adcs_hal.h served from the engine's TICK
 * frames, and the lockstep loop around adcs_fsw.h. Freestanding C99 (no malloc, no stdio).
 * Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved. */
#include "adcs_link.h"
#include "adcs_hal.h"
#include "adcs_fsw.h"

int32_t adcs_fsw_debug(double *out, int n);      /* extension, both builds export it */

#define RXCAP 1024
static struct {
    uint64_t now;
    uint8_t present, mag[7], gyro[13], sun[7], es[7];
    uint8_t rx[3][RXCAP]; int rxn[3];
    adcs_can_frame_t canq[32]; int ncan, can_i;
    adcs_can_frame_t tx[32]; int ntx;
    int16_t pwm[8];
} L;

static uint8_t buf[ADCS_LINK_MAX + 8], out[ADCS_LINK_MAX + 8];

uint16_t adcs_link_crc(const uint8_t *p, size_t n, uint16_t c)
{
    size_t i; int k;
    for (i = 0; i < n; i++) {
        c ^= (uint16_t)((uint16_t)p[i] << 8);
        for (k = 0; k < 8; k++) c = (c & 0x8000u) ? (uint16_t)(((unsigned)c << 1) ^ 0x1021u) : (uint16_t)((unsigned)c << 1);
    }
    return c;
}

static void cpy(uint8_t *d, const uint8_t *s, size_t n) { size_t i; for (i = 0; i < n; i++) d[i] = s[i]; }
static uint16_t r16(const uint8_t *b) { return (uint16_t)(b[0] | (b[1] << 8)); }
static uint32_t r32(const uint8_t *b) { return (uint32_t)b[0] | ((uint32_t)b[1] << 8) | ((uint32_t)b[2] << 16) | ((uint32_t)b[3] << 24); }
static uint64_t r64(const uint8_t *b) { return (uint64_t)r32(b) | ((uint64_t)r32(b + 4) << 32); }
static void w16(uint8_t *b, uint16_t v) { b[0] = (uint8_t)v; b[1] = (uint8_t)(v >> 8); }
static void w32(uint8_t *b, uint32_t v) { int k; for (k = 0; k < 4; k++) b[k] = (uint8_t)(v >> (8*k)); }

/* ---------------- adcs_hal.h over the last TICK ---------------- */
uint64_t adcs_hal_time_ns(void) { return L.now; }
adcs_status_t adcs_hal_uart_write(uint8_t port, const uint8_t *b, size_t n) { (void)port; (void)b; (void)n; return ADCS_OK; }
adcs_status_t adcs_hal_uart_read(uint8_t port, uint8_t *b, size_t cap, size_t *got)
{
    int n, i;
    if (port > 2) return ADCS_E_ARG;
    n = L.rxn[port] < (int)cap ? L.rxn[port] : (int)cap;
    if (n == 0) { *got = 0; return ADCS_E_TIMEOUT; }
    cpy(b, L.rx[port], (size_t)n);
    for (i = n; i < L.rxn[port]; i++) L.rx[port][i - n] = L.rx[port][i];
    L.rxn[port] -= n; *got = (size_t)n;
    return ADCS_OK;
}
adcs_status_t adcs_hal_i2c_xfer(uint8_t bus, uint8_t addr, const uint8_t *tx, size_t txn, uint8_t *rx, size_t rxn)
{
    const uint8_t *src = 0;
    (void)tx; (void)txn;
    if (bus == 0 && addr == 0x1E) { if (!(L.present & 1)) return ADCS_E_NODEV; src = L.mag; }
    else if (bus == 1 && addr == 0x60) { if (!(L.present & 4)) return ADCS_E_NODEV; src = L.sun; }
    else if (bus == 1 && addr == 0x30) { if (!(L.present & 8)) return ADCS_E_NODEV; src = L.es; }
    else return ADCS_E_ARG;
    cpy(rx, src, rxn > 7 ? 7 : rxn);
    return ADCS_OK;
}
adcs_status_t adcs_hal_spi_xfer(uint8_t bus, uint8_t cs, const uint8_t *tx, uint8_t *rx, size_t n)
{
    (void)tx;
    if (bus != 0 || cs != 0) return ADCS_E_ARG;
    if (!(L.present & 2)) return ADCS_E_NODEV;
    cpy(rx, L.gyro, n > 13 ? 13 : n);
    return ADCS_OK;
}
adcs_status_t adcs_hal_can_send(uint8_t port, const adcs_can_frame_t *f) { (void)port; if (L.ntx < 32) L.tx[L.ntx++] = *f; return ADCS_OK; }
adcs_status_t adcs_hal_can_recv(uint8_t port, adcs_can_frame_t *f)
{
    (void)port;
    if (L.can_i >= L.ncan) return ADCS_E_TIMEOUT;
    *f = L.canq[L.can_i++];
    return ADCS_OK;
}
adcs_status_t adcs_hal_pwm_set(uint8_t ch, int16_t d) { if (ch >= 8) return ADCS_E_ARG; L.pwm[ch] = d; return ADCS_OK; }
adcs_status_t adcs_hal_gpio_write(uint16_t pin, uint8_t l) { (void)pin; (void)l; return ADCS_OK; }
adcs_status_t adcs_hal_gpio_read(uint16_t pin, uint8_t *l) { (void)pin; *l = 0; return ADCS_OK; }
adcs_status_t adcs_hal_adc_read(uint8_t ch, uint16_t *raw) { (void)ch; *raw = 0; return ADCS_OK; }
adcs_status_t adcs_hal_tm_emit(uint16_t apid, const uint8_t *p, size_t n) { (void)apid; (void)p; (void)n; return ADCS_OK; }

/* ---------------- framing ---------------- */
static int read_frame(const adcs_link_io_t *io, uint8_t *type, uint16_t *len)
{
    int c, prev = -1, i;
    for (;;) {                                                   /* hunt for A5 5A */
        c = io->getc(io->ctx);
        if (c < 0) return -1;
        if (prev == ADCS_LINK_SYNC1 && c == ADCS_LINK_SYNC2) break;
        prev = c;
    }
    for (i = 0; i < 3; i++) { if ((c = io->getc(io->ctx)) < 0) return -1; buf[i] = (uint8_t)c; }
    *type = buf[0]; *len = r16(buf + 1);
    if (*len > ADCS_LINK_MAX) return -2;
    for (i = 0; i < *len + 2; i++) { if ((c = io->getc(io->ctx)) < 0) return -1; buf[3 + i] = (uint8_t)c; }
    if (adcs_link_crc(buf, (size_t)*len + 3, 0xFFFFu) != r16(buf + 3 + *len)) return -3;
    return 0;
}

static void send(const adcs_link_io_t *io, uint8_t type, size_t len)
{
    uint16_t c;
    out[0] = ADCS_LINK_SYNC1; out[1] = ADCS_LINK_SYNC2; out[2] = type; w16(out + 3, (uint16_t)len);
    c = adcs_link_crc(out + 2, len + 3, 0xFFFFu);
    w16(out + 5 + len, c);
    io->write(io->ctx, out, len + 7);
}

static void ack(const adcs_link_io_t *io, int32_t rc, const char *id)
{
    uint8_t *p = out + 5; int n = 0;
    w32(p, (uint32_t)rc);
    while (id && id[n] && n < 200) { p[5 + n] = (uint8_t)id[n]; n++; }
    p[4] = (uint8_t)n;
    send(io, LINK_ACK, (size_t)(5 + n));
}

static void tick(const adcs_link_io_t *io, const uint8_t *p, uint16_t len)
{
    int k = 0, i, port, n, rc;
    double dbg[48];
    uint8_t *o = out + 5;
    union { double d; uint64_t u; } cv;
    (void)len;
    L.now = r64(p); k = 8;
    L.present = p[k++];
    cpy(L.mag, p + k, 7); k += 7; cpy(L.gyro, p + k, 13); k += 13; cpy(L.sun, p + k, 7); k += 7; cpy(L.es, p + k, 7); k += 7;
    for (port = 1; port <= 2; port++) {
        n = r16(p + k); k += 2;
        for (i = 0; i < n; i++) { if (L.rxn[port] < RXCAP) L.rx[port][L.rxn[port]++] = p[k + i]; }
        k += n;
    }
    L.ncan = p[k++]; L.can_i = 0; if (L.ncan > 32) L.ncan = 32;
    for (i = 0; i < L.ncan; i++) { L.canq[i].id = r32(p + k); L.canq[i].extended = 0; L.canq[i].dlc = p[k + 4]; cpy(L.canq[i].data, p + k + 5, 8); k += 13; }
    L.ntx = 0;
    rc = adcs_fsw_step(L.now);
    k = 0;
    w32(o, (uint32_t)rc); k = 4;
    for (i = 0; i < 8; i++) { w16(o + k, (uint16_t)L.pwm[i]); k += 2; }
    o[k++] = (uint8_t)L.ntx;
    for (i = 0; i < L.ntx; i++) { w32(o + k, L.tx[i].id); o[k + 4] = L.tx[i].dlc; cpy(o + k + 5, L.tx[i].data, 8); k += 13; }
    n = adcs_fsw_debug(dbg, 48); if (n > 48) n = 48; if (n < 0) n = 0;
    o[k++] = (uint8_t)n;
    for (i = 0; i < n; i++) { cv.d = dbg[i]; w32(o + k, (uint32_t)cv.u); w32(o + k + 4, (uint32_t)(cv.u >> 32)); k += 8; }
    send(io, LINK_OUT, (size_t)k);
}

int adcs_link_serve(const adcs_link_io_t *io)
{
    uint8_t type; uint16_t len; int rc;
    adcs_fsw_init_t in;
    for (;;) {
        rc = read_frame(io, &type, &len);
        if (rc == -1) return -1;
        if (rc < 0) continue;                                   /* bad frame: resynchronise */
        switch (type) {
        case LINK_CONFIG:
            L.rxn[0] = L.rxn[1] = L.rxn[2] = 0; L.ncan = L.ntx = 0;
            in.abi_version = ADCS_FSW_ABI_VERSION; in.start_ns = r64(buf + 3);
            in.config_blob = buf + 11; in.config_len = (size_t)len - 8;
            L.now = in.start_ns;
            ack(io, adcs_fsw_init(&in), adcs_fsw_build_id());
            break;
        case LINK_TICK: tick(io, buf + 3, len); break;
        case LINK_CMD: ack(io, adcs_fsw_command(buf + 3, len), 0); break;
        case LINK_BYE: ack(io, 0, 0); return 0;
        default: ack(io, -99, 0); break;
        }
    }
}
