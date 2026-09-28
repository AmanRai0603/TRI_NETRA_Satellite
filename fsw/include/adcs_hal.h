/*
 * adcs_hal.h — the hardware abstraction the flight software is written against.
 *
 * ONE HEADER, THREE IMPLEMENTATIONS. The flight software (application and
 * device drivers) calls only what is declared here. What answers the call is
 * the only thing that changes between the rungs:
 *
 *   SILS  crates/adcs-fsw-abi (Rust) implements every function. Bytes written
 *         to a port go to the device emulator for that port inside the loop
 *         engine; bytes read come from it. Time is the loop engine's clock.
 *   PIL   the target processor's board support package implements it; the
 *         ports are wired to the rig, which speaks the adcs-bus loop contract.
 *   OILS  the flight OBC's own BSP implements it; the ports are the flight
 *   HILS  connectors, wired to the rig's interface emulation unit (OILS) or to
 *         real sensors and actuators (HILS).
 *
 * So the device drivers are exercised in SILS exactly as on the OBC: a
 * magnetometer driver reads the same I2C registers from an emulator that it
 * later reads from the part. That is what makes a SILS result evidence about
 * the flight code rather than about a model of it.
 *
 * RULES
 * - C99, no dynamic allocation, no blocking call. Every read is non-blocking
 *   and returns ADCS_E_TIMEOUT when nothing is there.
 * - Units: raw bytes at this layer. Engineering units exist only above the
 *   drivers.
 * - A new function here is an ABI change: bump ADCS_HAL_ABI_VERSION, and it
 *   needs two reviewers (a change here reaches every rung at once).
 */
#ifndef ADCS_HAL_H
#define ADCS_HAL_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define ADCS_HAL_ABI_VERSION 1u

typedef enum {
    ADCS_OK = 0,
    ADCS_E_TIMEOUT = -1, /* nothing to read yet; not an error in a poll loop */
    ADCS_E_BUS = -2,     /* NACK, framing, CRC or arbitration failure       */
    ADCS_E_ARG = -3,     /* a port, bus, address or length that does not exist */
    ADCS_E_NODEV = -4,   /* the device is absent or injected as failed      */
    ADCS_E_BUSY = -5     /* the transfer cannot start this tick             */
} adcs_status_t;

/* ---- time ----------------------------------------------------------- */

/* Monotonic nanoseconds. The loop engine's clock in SILS, the rig-disciplined
 * clock on target. Never wall time. */
uint64_t adcs_hal_time_ns(void);

/* ---- serial: UART, RS-422, RS-485 ----------------------------------- */

adcs_status_t adcs_hal_uart_write(uint8_t port, const uint8_t *buf, size_t len);
adcs_status_t adcs_hal_uart_read(uint8_t port, uint8_t *buf, size_t cap, size_t *got);

/* ---- I2C master ----------------------------------------------------- */

/* Write tx_len bytes, then read rx_len bytes, as one transaction with a
 * repeated start when both are non-zero. */
adcs_status_t adcs_hal_i2c_xfer(uint8_t bus, uint8_t addr7,
                                const uint8_t *tx, size_t tx_len,
                                uint8_t *rx, size_t rx_len);

/* ---- SPI master ----------------------------------------------------- */

adcs_status_t adcs_hal_spi_xfer(uint8_t bus, uint8_t chip_select,
                                const uint8_t *tx, uint8_t *rx, size_t len);

/* ---- CAN ------------------------------------------------------------ */

typedef struct {
    uint32_t id;      /* 11-bit or 29-bit identifier            */
    uint8_t extended; /* 1 when id is 29-bit                     */
    uint8_t dlc;      /* 0..8                                    */
    uint8_t data[8];
} adcs_can_frame_t;

adcs_status_t adcs_hal_can_send(uint8_t port, const adcs_can_frame_t *frame);
adcs_status_t adcs_hal_can_recv(uint8_t port, adcs_can_frame_t *frame);

/* ---- coil drive, GPIO, ADC ------------------------------------------ */

/* Signed duty for an H-bridge channel, Q15: -32767 full reverse, 32767 full
 * forward, 0 off. The rig measures this at the connector in OILS and HILS. */
adcs_status_t adcs_hal_pwm_set(uint8_t channel, int16_t duty_q15);

adcs_status_t adcs_hal_gpio_write(uint16_t pin, uint8_t level);
adcs_status_t adcs_hal_gpio_read(uint16_t pin, uint8_t *level);

/* Raw ADC counts: coil current sense, pump current, coarse sun diodes. */
adcs_status_t adcs_hal_adc_read(uint8_t channel, uint16_t *raw);

/* ---- telemetry ------------------------------------------------------ */

/* Emit one telemetry packet. The payload layout is the flight software's;
 * the rig records it verbatim and decodes it with the packet definitions
 * published beside the build (fsw/tm/). */
adcs_status_t adcs_hal_tm_emit(uint16_t apid, const uint8_t *payload, size_t len);

#ifdef __cplusplus
}
#endif

#endif /* ADCS_HAL_H */
