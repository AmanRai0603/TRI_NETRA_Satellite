/* hal_stub.h -- an in-memory adcs_hal.h for host tests: tests set what the
 * devices would put on their ports and read what the flight software wrote.
 * Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved. */
#ifndef HAL_STUB_H
#define HAL_STUB_H
#include "adcs_hal.h"

typedef struct {
    uint64_t now_ns;
    uint8_t mag[7], gyro[13], sun[7], es[7];
    uint8_t uart[3][512]; size_t uart_len[3];
    adcs_can_frame_t can_rx[32]; int n_can_rx;
    adcs_can_frame_t can_tx[32]; int n_can_tx;
    int16_t pwm[8];
    int gyro_silent;            /* the gyro's transfer fails (a gyro that has stopped answering) */
    /* a bus fault of the rig (O11, S11.14) at one device of the device map, in force while bf_on: bf_dev 1 magnetometer, 2 Sun sensor,
     * 3 Earth sensor, 4 star tracker, 5 gyro, 6 GNSS receiver, 7 coils, 8 CAN; bf_kind 1 NACK, 2 CRC wrong, 3 stuck, 4 supply drop, 5 CAN
     * error frame. bf_last holds what each device last delivered (stuck repeats it); writes_failed counts the output writes refused */
    int bf_dev, bf_kind, bf_on;
    uint8_t bf_last[16][64]; size_t bf_len[16];
    uint32_t writes_failed;
} hal_stub_t;

extern hal_stub_t HS;
void hal_stub_reset(void);
void hal_stub_uart_push(uint8_t port, const uint8_t *b, size_t n);
#endif
