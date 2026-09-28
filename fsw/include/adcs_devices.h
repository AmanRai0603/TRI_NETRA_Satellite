/*
 * adcs_devices.h -- byte protocols of the synthetic parts (fsw/pseudocode/09_drivers_hal.md).
 * The same numbers drive the flight drivers (C and Rust) and the engine's
 * device emulators; a real part replaces its block with its ICD values.
 * All multi-byte fields little-endian.
 * Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
 */
#ifndef ADCS_DEVICES_H
#define ADCS_DEVICES_H

/* magnetometer SYN-MAG-1: I2C, read reg 0x00 -> status u8, Bx By Bz int16 */
#define ADCS_MAG_BUS 0u
#define ADCS_MAG_ADDR 0x1Eu
#define ADCS_MAG_LSB_T (1.0e-4/32768.0)
/* gyro: SPI, cmd 0x80 -> status u8, wx wy wz int32 */
#define ADCS_GYRO_BUS 0u
#define ADCS_GYRO_CS 0u
#define ADCS_GYRO_CMD 0x80u
#define ADCS_GYRO_LSB (5.0/8388608.0)
/* Sun-sensor set: I2C -> status u8 (bit0 valid), sx sy sz int16 Q15 (body) */
#define ADCS_SUN_BUS 1u
#define ADCS_SUN_ADDR 0x60u
/* Earth sensor: I2C -> status u8 (bit0 valid), nx ny nz int16 Q15 (body) */
#define ADCS_ES_BUS 1u
#define ADCS_ES_ADDR 0x30u
/* star tracker: UART frame EB 90 len | nh u8 | per head: valid u8, q int32 x4 Q30 (x y z w, ECI->body) | CRC-16/CCITT */
#define ADCS_ST_UART 1u
#define ADCS_ST_SYNC1 0xEBu
#define ADCS_ST_SYNC2 0x90u
#define ADCS_Q30 1073741824.0
/* GNSS: UART frame EB 91 len | fix u8, r int32 x3 [cm], v int32 x3 [mm/s] | CRC-16/CCITT */
#define ADCS_GPS_UART 2u
#define ADCS_GPS_SYNC2 0x91u
/* momentum devices on CAN 0 */
#define ADCS_CAN_PORT 0u
#define ADCS_CAN_ROTOR_CMD 0x100u       /* + i : int16 Q15 of torque_max            */
#define ADCS_CAN_GIMBAL_CMD 0x140u      /* + j : int16 Q15 of gimbal_rate_max       */
#define ADCS_CAN_ROTOR_TM 0x200u        /* + i : h int32 [1e-9 N m s], delta int32 [1e-7 rad] */
#define ADCS_CAN_VALVES 0x300u          /* on-time u8 per couple [1 ms]             */
#define ADCS_H_LSB 1.0e-9
#define ADCS_DELTA_LSB 1.0e-7
#define ADCS_VALVE_LSB_S 0.001
/* coils: PWM channel = coil (body axis), duty Q15 of m_max */
#define ADCS_Q15 32767.0

#endif
