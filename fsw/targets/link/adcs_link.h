/*
 * adcs_link.h -- adcs-link/1: the lockstep byte link between the SILS engine and a
 * virtual or real OBC running the flight software (docs/VIRTUAL_OBC.md).
 *
 * Every message is a frame:  A5 5A | type u8 | len u16 | payload[len] | crc16 u16
 * (little-endian; CRC-16/CCITT-FALSE over type, len and payload).
 *
 *   engine -> OBC                               OBC -> engine
 *   0x01 CONFIG  u64 start_ns, blob[]           0x81 ACK   i32 rc, u8 n, build_id[n]
 *   0x02 TICK    u64 now_ns, u8 present,        0x82 OUT   i32 rc, i16 pwm[8],
 *                mag[7] gyro[13] sun[7] es[7],             u8 ncan, ncan x (u32 id, u8 dlc, data[8]),
 *                u16 n1, uart1[n1], u16 n2, uart2[n2],     u8 ndbg, ndbg x f64 (adcs_fsw_debug)
 *                u8 ncan, ncan x (u32 id, u8 dlc, data[8])
 *   0x03 CMD     tc[]                           0x81 ACK   i32 rc, u8 0
 *   0x04 BYE                                    0x81 ACK   i32 0, u8 0   (then the server returns)
 *
 * present: bit0 magnetometer, bit1 gyro, bit2 Sun sensors, bit3 Earth sensor (an absent
 * device answers ADCS_E_NODEV, as on the bus). The OBC side implements adcs_hal.h from the
 * last TICK, so the flight software's drivers run unchanged.
 * Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
 */
#ifndef ADCS_LINK_H
#define ADCS_LINK_H
#include <stdint.h>
#include <stddef.h>

#define ADCS_LINK_SYNC1 0xA5u
#define ADCS_LINK_SYNC2 0x5Au
#define ADCS_LINK_MAX 4096
enum { LINK_CONFIG = 0x01, LINK_TICK = 0x02, LINK_CMD = 0x03, LINK_BYE = 0x04, LINK_ACK = 0x81, LINK_OUT = 0x82 };

/* the transport: blocking byte I/O supplied by the platform (POSIX pipe/socket, a UART) */
typedef struct {
    int (*getc)(void *ctx);                       /* next byte, or -1 on end of stream */
    void (*write)(void *ctx, const uint8_t *b, size_t n);
    void *ctx;
} adcs_link_io_t;

uint16_t adcs_link_crc(const uint8_t *p, size_t n, uint16_t c);
/* serve the link until BYE or end of stream; returns 0 on BYE */
int adcs_link_serve(const adcs_link_io_t *io);

#endif
