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

## The deadline, judged

The trajectory flies the nominal latency; the verdict is taken on the worst case:

```
worst = bus time + execution x cpi_max / cpi + interrupts      must land within deadline_frac x dt
```

with `cpi_max` 2.0 (the flash accelerator missing every fetch), 50 us of interrupts that may preempt
a step, and half the control period, by default (`--cpi-max`, `--isr-us`, `--deadline-frac`). Every
soft-OILS run adds two judged metrics to its verdicts: `oils_overruns` (none allowed) and
`oils_worst_case_margin` (the deadline less the worst latency, in ms; at least 0).
`tools/engine.py oils --cpi 1.0 1.5 2.0` flies the whole matrix again at each CPI (the latency
changes, so the trajectory may) and tabulates overruns, margin and verdicts per CPI.

What the instruction count holds: everything inside `adcs_fsw_step`, including the HAL's copies of
the sensor bytes (on a real OBC those are the bus transfers, already timed above: counted twice, so
the budget is conservative). The adcs-link framing outside the step is the simulation's own and
is not counted. QEMU runs with `-icount shift=0,sleep=off`, so the firmware's own counter (the
link's timing trailer) advances one nanosecond per instruction and repeats run to run.

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
| SPI | 1 MHz | `--spi-khz` |
| CAN | 1 Mbit/s | `--can-kbps` |
| worst-case cycles per instruction | 2.0 | `--cpi-max` |
| interrupts that may preempt a step | 50 us | `--isr-us` |
| deadline, share of the control period | 0.5 | `--deadline-frac` |

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

## What the matrix found, and what was fixed

The first matrix showed 104 of 107 verdicts agreeing between SILS and soft OILS. Each of the
three disagreements was then traced to its root cause:

- **Bang-bang B-dot: a defect in the law, now fixed.** A latency sweep with `--latency-ms`
  (0.5, 1, 2, 4, 8, 20, 50 ms, in-process) gave non-monotonic results: 165 min, never, 192 min,
  26 min, and so on. A 12-seed sweep with **zero** latency then showed 4 of 12 seeds never
  settling.
  - Cause: pure sign switching leaves a limit cycle around the 0.5 deg/s exit rate. Soft OILS had
    only picked a different realisation.
  - Fix: a boundary layer. Full dipole outside it; inside it, a proportional law at 4 × the B-dot
    gain.
  - Result: 12 of 12 seeds detumble (17–106 min) at 0, 4 and 8 ms latency. The fix is identical
    in C, Rust, MATLAB and the pseudocode (`06_detumble_sunspin.md`), and C and Rust remain
    bit-identical.
- **Sun spin: same statistics with and without the OBC.** 24 seeds give 17/24 passing in both.
  - Seed 1 fails in SILS too: Sun spin is entered with the Sun only 28° below the body XY plane.
    The L2 precession transient reaches 1.05 deg/s transverse rate and trips the 1 deg/s exit
    guard, so the controller falls back to spin-up.
  - This is Sun-acquisition tuning, not an OILS effect. The exit guard's rate and dwell are now
    scenario keys (`fsw.sun_spin_perp_out_dps`, `fsw.sun_spin_dwell_out_s`) for that trade.
- **VSCMG agile slew: a knife edge.** Across 12 seeds, the APE with 0, 3 and 8 ms latency differs
  by under 1 %, and all of them sit at 0.010 deg against 0.010 deg. It is a design-margin
  question, not the OBC.

The OBC is never the limit: the worst load is 6.8 % of the control period (VSCMG steering,
~0.92 M instructions per step), with no overrun anywhere. The current matrix is in
`results/SOFT_OILS.md`.
