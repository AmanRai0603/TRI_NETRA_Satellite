/* hal_stub.c -- see hal_stub.h. Owner: Agastya. Copyright (c) 2026 Agastya. */
#include <string.h>
#include "hal_stub.h"
#include "adcs_devices.h"

hal_stub_t HS;

void hal_stub_reset(void) { memset(&HS, 0, sizeof HS); }

void hal_stub_uart_push(uint8_t port, const uint8_t *b, size_t n)
{
    if (port > 2 || HS.uart_len[port] + n > sizeof HS.uart[port]) return;
    memcpy(HS.uart[port] + HS.uart_len[port], b, n);
    HS.uart_len[port] += n;
}

static int bf(int dev, int k1, int k2) { return HS.bf_on && HS.bf_dev == dev && (HS.bf_kind == k1 || HS.bf_kind == k2); }

/* what a device delivers: nothing (a NACK, no supply), its last delivery again (stuck), or its fresh bytes (kept as the last) */
static int bf_deliver(int dev, int slot, const uint8_t *fresh, size_t n, uint8_t *out)
{
    if (bf(dev, 1, 4)) return 0;
    if (bf(dev, 3, 3) && HS.bf_len[slot]) { memcpy(out, HS.bf_last[slot], HS.bf_len[slot]); return 1; }
    memcpy(HS.bf_last[slot], fresh, n); HS.bf_len[slot] = n; memcpy(out, fresh, n);
    return 1;
}

uint64_t adcs_hal_time_ns(void) { return HS.now_ns; }

adcs_status_t adcs_hal_uart_write(uint8_t port, const uint8_t *buf, size_t len) { (void)port; (void)buf; (void)len; return ADCS_OK; }

adcs_status_t adcs_hal_uart_read(uint8_t port, uint8_t *buf, size_t cap, size_t *got)
{
    size_t n;
    if (port > 2) return ADCS_E_ARG;
    if (port == 1 || port == 2) {
        int dev = port == 1 ? 4 : 6;
        if (bf(dev, 1, 4)) HS.uart_len[port] = 0;
        if (bf(dev, 2, 2) && HS.uart_len[port]) HS.uart[port][HS.uart_len[port] - 1] ^= 1;
        if (HS.uart_len[port] && HS.uart_len[port] <= 64) {
            if (bf(dev, 3, 3) && HS.bf_len[10 + port]) { memcpy(HS.uart[port], HS.bf_last[10 + port], HS.bf_len[10 + port]); HS.uart_len[port] = HS.bf_len[10 + port]; }
            else { memcpy(HS.bf_last[10 + port], HS.uart[port], HS.uart_len[port]); HS.bf_len[10 + port] = HS.uart_len[port]; }
        }
    }
    n = HS.uart_len[port] < cap ? HS.uart_len[port] : cap;
    if (n == 0) { *got = 0; return ADCS_E_TIMEOUT; }
    memcpy(buf, HS.uart[port], n);
    memmove(HS.uart[port], HS.uart[port] + n, HS.uart_len[port] - n);
    HS.uart_len[port] -= n; *got = n;
    return ADCS_OK;
}

adcs_status_t adcs_hal_i2c_xfer(uint8_t bus, uint8_t addr7, const uint8_t *tx, size_t tx_len, uint8_t *rx, size_t rx_len)
{
    const uint8_t *src = 0; int dev = 0; uint8_t got[7];
    (void)tx; (void)tx_len;
    if (bus == ADCS_MAG_BUS && addr7 == ADCS_MAG_ADDR) { src = HS.mag; dev = 1; }
    else if (bus == ADCS_SUN_BUS && addr7 == ADCS_SUN_ADDR) { src = HS.sun; dev = 2; }
    else if (bus == ADCS_ES_BUS && addr7 == ADCS_ES_ADDR) { src = HS.es; dev = 3; }
    if (!src) return ADCS_E_NODEV;
    if (!bf_deliver(dev, dev, src, 7, got)) return ADCS_E_NODEV;
    memcpy(rx, got, rx_len > 7 ? 7 : rx_len);
    return ADCS_OK;
}

adcs_status_t adcs_hal_spi_xfer(uint8_t bus, uint8_t cs, const uint8_t *tx, uint8_t *rx, size_t len)
{
    (void)tx;
    if (bus != ADCS_GYRO_BUS || cs != ADCS_GYRO_CS) return ADCS_E_NODEV;
    if (HS.gyro_silent) return ADCS_E_TIMEOUT;
    { uint8_t got[13]; if (!bf_deliver(5, 5, HS.gyro, 13, got)) return ADCS_E_NODEV; memcpy(rx, got, len > 13 ? 13 : len); }
    return ADCS_OK;
}

adcs_status_t adcs_hal_can_send(uint8_t port, const adcs_can_frame_t *f)
{
    (void)port;
    if (bf(8, 1, 5)) { HS.writes_failed++; return ADCS_E_BUS; }
    if (HS.n_can_tx < 32) HS.can_tx[HS.n_can_tx++] = *f;
    return ADCS_OK;
}

adcs_status_t adcs_hal_can_recv(uint8_t port, adcs_can_frame_t *f)
{
    (void)port;
    if (bf(8, 5, 5)) return ADCS_E_BUS;
    if (bf(8, 1, 4)) HS.n_can_rx = 0;
    if (HS.n_can_rx == 0) return ADCS_E_TIMEOUT;
    *f = HS.can_rx[0];
    memmove(HS.can_rx, HS.can_rx + 1, (size_t)(HS.n_can_rx - 1)*sizeof *f);
    HS.n_can_rx--;
    return ADCS_OK;
}

adcs_status_t adcs_hal_pwm_set(uint8_t ch, int16_t duty)
{
    if (bf(7, 1, 1)) { HS.writes_failed++; return ADCS_E_BUS; }
    if (ch < 8) HS.pwm[ch] = duty;
    return ADCS_OK;
}
adcs_status_t adcs_hal_gpio_write(uint16_t pin, uint8_t level) { (void)pin; (void)level; return ADCS_OK; }
adcs_status_t adcs_hal_gpio_read(uint16_t pin, uint8_t *level) { (void)pin; *level = 0; return ADCS_OK; }
adcs_status_t adcs_hal_adc_read(uint8_t ch, uint16_t *raw) { (void)ch; *raw = 0; return ADCS_OK; }
adcs_status_t adcs_hal_tm_emit(uint16_t apid, const uint8_t *p, size_t n) { (void)apid; (void)p; (void)n; return ADCS_OK; }
