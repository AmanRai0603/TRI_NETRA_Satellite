# 09 — Device drivers over the byte HAL (`adcs_drv`, `drv`)

The flight software sees devices only as bytes through `adcs_hal.h`. The protocols below belong to
the synthetic parts and are data in `fsw/devices.toml`. The same file drives:
- the C and Rust drivers;
- the Rust engine's emulators;
- the MATLAB twin's quantisation (`asils.hal.lsb`).

A real part replaces its entry with its ICD. All multi-byte fields are little-endian.

| device | bus | address / id | read | LSB |
|---|---|---|---|---|
| magnetometer SYN-MAG-1 | I2C bus 0 | 0x1E, reg 0x00 | status u8 (bit0 ready), Bx, By, Bz int16 | 1e-4/2¹⁵ T |
| gyro SYN-GYRO-1 / TRN-GYRO-P1 | SPI bus 0 | CS 0, cmd 0x80 | status u8, ωx, ωy, ωz int32 | 5/2²³ rad/s |
| Sun sensor set SYN-SUN-1 | I2C bus 1 | 0x60, reg 0x00 | status u8 (bit0 valid), sx, sy, sz int16 (body) | Q15 |
| Earth sensor SYN-ES-1 | I2C bus 1 | 0x30, reg 0x00 | status u8 (bit0 valid), nx, ny, nz int16 (body) | Q15 |
| star tracker SYN-ST-1 | UART 1 | frame `EB 90 len` | per head: valid u8, q int32 ×4 (scalar last, ECI→body) + CRC-16/CCITT | Q30 |
| GNSS TRN-GPS-1 | UART 2 | frame `EB 91 len` | fix u8, r int32 ×3, v int32 ×3 + CRC-16 | 1 cm, 1 mm/s |
| momentum devices | CAN 0 | tx 0x100+i: τ cmd int16 Q15 of τmax · 0x140+j: δ̇ cmd int16 Q15 of δ̇max | rx 0x200+i: h int32 (1e-9 N m s), δ int32 (1e-7 rad) | |
| coils | PWM 0..2 | channel = coil | duty int16 Q15 of m_max | |
| N2O valves | CAN 0 | tx 0x300: on-time u8 per couple (1 ms) | — | 1 ms |

A driver returns `ADCS_E_TIMEOUT` when a device has nothing new. The FSW then keeps the last valid
value and marks the sensor invalid for this tick. The magnetometer read also reports whether the
coils were off, because the FSW knows its own duty cycle: that is the field's "clean" flag.
