# Virtual OBC loop (processor-in-the-loop without hardware)

Owner: Agastya. The same flight software that runs linked into the engine can run on a
**virtual OBC**: a separate host process, or bare-metal firmware on an **emulated
Cortex-M4F** (QEMU `mps2-an386`). The engine is the plant and the device emulators; the OBC
is the flight software with its drivers. They meet on **adcs-link/1**, a lockstep byte link.
The same link carries a **real OBC** (a TCP socket, or a serial-to-TCP bridge) for OILS.

```
 Rust engine (adcs-sim)                                  OBC (virtual or real)
 orbit + environment + plant                              adcs_link.c: adcs_hal.h served from TICK
 sensor models -> device bytes (I2C regs,   TICK ───────►  adcs_fsw_step()  (C: fsw/, or Rust: fsw-rs cabi)
   SPI, UART frames with CRC, CAN)                          drivers read the same bytes as on the bus
 actuator models <- PWM words, CAN frames   ◄─────── OUT   PWM, CAN commands, debug vector
```

## The link: adcs-link/1 (`fsw/targets/link/adcs_link.h`)

Frames `A5 5A | type | len u16 | payload | CRC-16/CCITT`. The engine sends `CONFIG` (the
adcs-fswcfg/1 blob) once, then one `TICK` per flight-software tick (time, the I2C/SPI register
images of magnetometer, gyro, Sun sensors, Earth sensor with presence bits, the UART bytes of the
star tracker and GNSS since the last tick, the CAN telemetry of every rotor). The OBC answers each
TICK with `OUT` (the eight PWM words, the CAN frames it sent: rotor torques, gimbal rates, valve
on-times, and the 41-value debug vector the recorder files). `CMD` carries telecommands, `BYE`
ends the session. A TICK is ~120–200 bytes, an OUT ~400 bytes: at 10 Hz that is < 6 kB/s each
way, well inside a 115 200 baud UART.

## Targets (`make -C fsw obc`)

| `--fsw` | what runs the flight software | built with |
|---|---|---|
| `c`, `rust` | linked into the engine (reference) | cc crate / cargo |
| `obc-posix` | `fsw/build/obc_posix`, a host process on stdin/stdout (or `--listen PORT`) | gcc, C flight software |
| `obc-posix-rs` | `fsw/build/obc_posix_rs` | gcc + the Rust flight software (`cabi`) |
| `qemu` | `fsw/build/obc_qemu.elf` on QEMU mps2-an386 (Cortex-M4F, CMSDK UART0, semihosting exit) | arm-none-eabi-gcc + newlib, `-mfloat-abi=hard` |
| `qemu-rs` | `fsw/build/obc_qemu_rs.elf` | rustc `thumbv7em-none-eabihf` + the same link/startup C |
| `spawn:<cmd>` | any program speaking adcs-link/1 on stdin/stdout | — |
| `tcp:<host:port>` | an OBC reachable over TCP: `obc_posix --listen`, or a real OBC behind a serial bridge | — |

The bare-metal firmware (`fsw/targets/qemu-mps2/`) is the OBC port in miniature: vector table,
`.data`/`.bss` initialisation, FPU enable, a polled UART, no heap, no OS. Porting to a flight OBC
means replacing `main.c`'s UART with the OBC's link (or implementing `adcs_hal.h` on the real
buses directly and dropping the link).

## Commands

```bash
make -C fsw obc                                                   # the four virtual OBCs
engine/target/release/adcs run mission_img --fsw qemu             # a full run with the OBC in QEMU
engine/target/release/adcs parity fine_hold_img --fsw c --against qemu      # sample-by-sample comparison
fsw/build/obc_posix --listen 5555 &  engine/target/release/adcs run slew_img --fsw tcp:127.0.0.1:5555
engine/target/release/adcs run nadir_hold_ais --fsw tcp:<obc-bridge>:<port> --realtime       # OILS pace
python3 tools/engine.py vobc                                      # the matrix -> results/VIRTUAL_OBC.md
```

## What the matrix shows (results/VIRTUAL_OBC.md)

The flight software gives bit-identical closed-loop trajectories in-process, as a separate
process over the link, and as Cortex-M4 firmware in QEMU, for both the C and the Rust build: the
link, the HAL-over-link and the cross-compiled maths change nothing. A difference, if a
future toolchain or target introduces one, shows up here as a measured number (spec §9.7:
"the difference is a parity-ledger line, never a failure to hide").
