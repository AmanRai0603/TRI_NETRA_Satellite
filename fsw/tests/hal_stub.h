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
} hal_stub_t;

extern hal_stub_t HS;
void hal_stub_reset(void);
void hal_stub_uart_push(uint8_t port, const uint8_t *b, size_t n);
#endif
