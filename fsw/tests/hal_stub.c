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

uint64_t adcs_hal_time_ns(void) { return HS.now_ns; }

adcs_status_t adcs_hal_uart_write(uint8_t port, const uint8_t *buf, size_t len) { (void)port; (void)buf; (void)len; return ADCS_OK; }

adcs_status_t adcs_hal_uart_read(uint8_t port, uint8_t *buf, size_t cap, size_t *got)
{
    size_t n;
    if (port > 2) return ADCS_E_ARG;
    n = HS.uart_len[port] < cap ? HS.uart_len[port] : cap;
    if (n == 0) { *got = 0; return ADCS_E_TIMEOUT; }
    memcpy(buf, HS.uart[port], n);
    memmove(HS.uart[port], HS.uart[port] + n, HS.uart_len[port] - n);
    HS.uart_len[port] -= n; *got = n;
    return ADCS_OK;
}

adcs_status_t adcs_hal_i2c_xfer(uint8_t bus, uint8_t addr7, const uint8_t *tx, size_t tx_len, uint8_t *rx, size_t rx_len)
{
    const uint8_t *src = 0;
    (void)tx; (void)tx_len;
    if (bus == ADCS_MAG_BUS && addr7 == ADCS_MAG_ADDR) src = HS.mag;
    else if (bus == ADCS_SUN_BUS && addr7 == ADCS_SUN_ADDR) src = HS.sun;
    else if (bus == ADCS_ES_BUS && addr7 == ADCS_ES_ADDR) src = HS.es;
    if (!src) return ADCS_E_NODEV;
    memcpy(rx, src, rx_len > 7 ? 7 : rx_len);
    return ADCS_OK;
}

adcs_status_t adcs_hal_spi_xfer(uint8_t bus, uint8_t cs, const uint8_t *tx, uint8_t *rx, size_t len)
{
    (void)tx;
    if (bus != ADCS_GYRO_BUS || cs != ADCS_GYRO_CS) return ADCS_E_NODEV;
    memcpy(rx, HS.gyro, len > 13 ? 13 : len);
    return ADCS_OK;
}

adcs_status_t adcs_hal_can_send(uint8_t port, const adcs_can_frame_t *f)
{
    (void)port;
    if (HS.n_can_tx < 32) HS.can_tx[HS.n_can_tx++] = *f;
    return ADCS_OK;
}

adcs_status_t adcs_hal_can_recv(uint8_t port, adcs_can_frame_t *f)
{
    (void)port;
    if (HS.n_can_rx == 0) return ADCS_E_TIMEOUT;
    *f = HS.can_rx[0];
    memmove(HS.can_rx, HS.can_rx + 1, (size_t)(HS.n_can_rx - 1)*sizeof *f);
    HS.n_can_rx--;
    return ADCS_OK;
}

adcs_status_t adcs_hal_pwm_set(uint8_t ch, int16_t duty) { if (ch < 8) HS.pwm[ch] = duty; return ADCS_OK; }
adcs_status_t adcs_hal_gpio_write(uint16_t pin, uint8_t level) { (void)pin; (void)level; return ADCS_OK; }
adcs_status_t adcs_hal_gpio_read(uint16_t pin, uint8_t *level) { (void)pin; *level = 0; return ADCS_OK; }
adcs_status_t adcs_hal_adc_read(uint8_t ch, uint16_t *raw) { (void)ch; *raw = 0; return ADCS_OK; }
adcs_status_t adcs_hal_tm_emit(uint16_t apid, const uint8_t *p, size_t n) { (void)apid; (void)p; (void)n; return ADCS_OK; }
