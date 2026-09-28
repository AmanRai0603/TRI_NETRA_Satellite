# Soft OILS: SILS and the flight software on a virtual OBC, together

**Owner: Agastya.** Copyright (c) 2026 Agastya. All rights reserved.

Soft OILS is OILS without a board. The flight software is compiled for the OBC
(`arm-none-eabi-gcc -mcpu=cortex-m4 -mthumb -mfloat-abi=hard -mfpu=fpv4-sp-d16`, or `rustc
--target thumbv7em-none-eabihf`), linked into bare-metal firmware (`fsw/targets/qemu-mps2`),
and runs on an emulated Cortex-M4F (QEMU `mps2-an386`). The Rust engine flies the spacecraft
(POP in the loop) and talks to it over adcs-link/1, one TICK per control period, exactly as it
would talk to a real OBC.

```
adcs run <scenario> --fsw qemu --oils        # C flight software on the virtual M4F
adcs run <scenario> --fsw qemu-rs --oils     # Rust flight software on the virtual M4F
python3 tools/engine.py oils                 # every scenario: SILS and soft OILS side by side -> results/SOFT_OILS.md
python3 tools/pipeline.py                    # the dispatched mission of each case in SILS and soft OILS (node soft_oils)
```

## What soft OILS adds over the zero-latency virtual OBC

The zero-latency lockstep (`--fsw qemu` without `--oils`, `docs/VIRTUAL_OBC.md`) proves the bytes:
the firmware reproduces the in-process trajectory bit for bit. Soft OILS adds **time**. In each
tick the command reaches the actuators only after:

```
latency = sensor reads on the buses          I2C 10 bytes x 9 bits per device (magnetometer, Sun, Earth sensor) / 400 kHz,
                                             SPI 13 bytes x 8 bits (gyro) / 1 MHz
        + OBC execution of the step          exact instruction count x CPI / core clock
        + the CAN command frames             130 bits per frame / 1 Mbit/s
```

Until then the previous command holds: the plant is integrated over `latency` with the old
actuation, then over `dt - latency` with the new one. A latency of a whole period or more is an
**overrun**: the command lands a tick late. Every run's manifest carries an `oils` block with the
latency, execution, bus time, instruction counts, CPU load and overruns.

## Exact, repeatable timing

The execution time is not a wall-clock measurement. A QEMU TCG plugin
(`fsw/targets/qemu-mps2/insn_count.c`, built by `make -C fsw obc`) counts the guest instructions
between two marker instructions that bracket `adcs_fsw_step` in the link server (`mov r9, r9`,
`mov r10, r10`; no compiler emits them). The count is exact, so the same scenario gives the same
counts and the same trajectory on every run and every machine. The conversion to time uses the
OBC model:

| parameter | default | flag |
|---|---|---|
| core clock | 168 MHz (STM32F4 class) | `--obc-mhz` |
| cycles per instruction | 1.25 (flash accelerator on) | `--cpi` |
| I2C | 400 kHz | `--i2c-khz` |
| CAN | 1 Mbit/s | `--can-kbps` |

The FPU of the Cortex-M4F is single precision, so the flight software's double-precision
arithmetic runs in software routines. The instruction counts include that cost, which is the real
cost the OBC will pay.

The link also carries a timing trailer from the target's own counter (the SysTick on the QEMU
target, `clock_gettime` on the POSIX target, the board's timer on a real OBC), so the same
engine-side bookkeeping serves real OILS.

## From soft OILS to OILS

On a real OBC, the same flight software and the same link server run behind a board target.
`fsw/targets/<board>/main.c` supplies the UART or Ethernet transport, the clock and a linker
script, modelled on `fsw/targets/qemu-mps2/main.c`. The engine then connects with
`--fsw tcp:<host:port>` (or a serial-to-TCP bridge) and `--realtime`. The comparison against SILS
and against soft OILS is the same ledger.

## What the matrix found

- Soft OILS matches SILS on 104 of 107 requirement verdicts over the 40 scenarios. The worst OBC load
  is 6.8 % of the control period (VSCMG steering, ~0.92 M instructions per step), with no overrun
  anywhere (`results/SOFT_OILS.md`).
- **Bang-bang B-dot does not survive the latency.** It detumbles in 32 min in SILS. With the OBC's
  2–8 ms command delay it settles into a 0.7–1.0 deg/s limit cycle, so it never holds below
  0.5 deg/s. The proportional laws are unaffected. This is exactly the kind of result SILS cannot show.
- The other two flips are knife edges: `agile_slew_vscmg` APE (0.01008 vs 0.00997 deg against
  0.01 deg), and the known Sun-spin sign-flip lock-up, which the latency happens to avoid in this seed.
