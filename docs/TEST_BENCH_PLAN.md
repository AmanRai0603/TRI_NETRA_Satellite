# From soft OILS to HILS: what exists, what is missing, and how to build it

**In one line:** an audit of everything between pure SILS and flight hardware, set against ESA, NASA and industry test-bench
practice, with a target architecture, the equipment it needs, phases with entry and exit criteria, and the acceptance
thresholds between each rung of the ladder.

Companion to `docs/TECHNICAL_ROADMAP.md` (phase U5), `docs/OILS_HILS.md` (the original plan), `docs/SOFT_OILS.md`
and `docs/ADCS_GAPS.md` H1–H5.

## 1 · Where it stands

**Soft OILS is real, deterministic and well engineered for what it is.** It is a *processor-in-the-loop with exact
instruction counts*:
- The C and Rust firmware run on QEMU `mps2-an386` (Cortex-M4F) with `-icount`.
- A TCG plugin counts instructions per step exactly. Count × CPI (1.25, assumed) / 168 MHz, plus a bus-time formula, becomes command latency applied inside the plant step.
- Overruns and worst-case margin are judged as metrics.
- Two runs give byte-identical channels.
- `results/SOFT_OILS.md`: 124 of 126 verdicts agree with SILS over 47 scenarios; maximum load 7.5 %.
- CI builds both firmwares, checks stack depth, and runs a 300 s bit-parity flight for C and for Rust.

**Everything beyond it is paper:**
- no board target;
- no interface emulation unit (IEU);
- no real-time plant;
- no time synchronisation;
- no bus-level fault injection;
- no stimulator drivers;
- no ground-segment link.

`spec/spec/11_contract.md` defines a loop contract (UDP per cycle, sequence, CRC-32, PPS + TimeSync, FaultInject,
Heartbeat, 1 kHz rig cycle), but the `adcs-bus::loop` and `adcs-rig` crates it describes do not exist. `spec/rig/labs/hils_bay1.toml`
has every measured field `nan`, and the `oils`, `hils` and `lab` design groups are declared but have zero sealed rows.

There are **two incompatible OILS protocols**:
- **adcs-link/1** (Rust/C): lockstep, CRC-16, no sequence numbers or heartbeat.
- **MATLAB `udp` backend**: raw datagrams, no header, CRC, sequence or timeout. If no reply arrives it silently uses the MATLAB flight software's own command, so a dead OBC looks like a passing run.

## 2 · Findings

Severity: C critical, H high, M medium, L low. Effort: S days, M 1–2 weeks, L longer.

### Soft OILS (processor in the loop)

| Id | Sev. | Finding | Evidence | Fix | Effort |
|---|---|---|---|---|---|
| O1 | H | Timing = instruction count × one assumed CPI; neither CPI nor board calibrated; code runs from zero-wait SSRAM while the CPI is quoted for STM32F4 flash | `run.rs:366`; H2; QEMU documents icount as not cycle-accurate | Board cycle counter (DWT->CYCCNT trailer already designed) | S once a board exists |
| O2 | H | A missing plugin silently changes the time base: reproduced, exec max 0.69 ms instead of 5.13 ms (7.4× optimistic), still passing, no warning | `adcs-fsw-abi/src/lib.rs:173` (`if plugin.is_file()`) | Refuse `--oils` on QEMU without the plugin; record time-base source in the manifest | S |
| O3 | M | No peripheral, interrupt or DMA models: HAL served from RAM copies; drivers never touch a register; ISR load a constant 50 µs | `fsw/targets/qemu-mps2/main.c` | Add Renode (STM32F4 platform: UART, I2C, SPI, timers, CAN, IRQs) as a second backend beside QEMU | M–L |
| O4 | M | Memory map not the target's (4 MB / 2 MB vs STM32F4 1 MB flash, 192 kB SRAM incl. 64 kB CCM without DMA) | `link.ld` | Second linker script with real limits as a CI check | S |
| O5 | M | OBC slaved to TICK, time frozen per step, so jitter, drift and rate-group aliasing (F3) are invisible by construction | `adcs_link.c`; spec §11.3 says the OBC runs on its own clock | Free-running soft-OILS mode on virtual time with a timer interrupt | M |
| O6 | L | No sampling skew between sensors; no backlog beyond one tick | `run.rs:426-440` | Per-device sample times; command queue | S–M |

### OILS with the real OBC

| Id | Sev. | Finding | Fix | Effort |
|---|---|---|---|---|
| O7 | C | No board target, real-bus HAL or IEU (`fsw/targets/` = link, posix, qemu-mps2; `[ieu]` all `nan`) | Board BSP + IEU v0 (H1) | L |
| O8 | H | adcs-link/1 not robust for a physical link: no sequence or tick echo; no heartbeat, retry or reconnect; a corrupt length stalls 60 s; CRC-16 not the spec's CRC-32; lockstep at 115 200 baud costs about 50 ms of a 100 ms period | adcs-link/2 = the spec's loop contract | M |
| O9 | H | No real-time execution: `--realtime` is `thread::sleep`, slip not recorded | Rig host on PREEMPT_RT (mainline since Linux 6.12), cyclictest/rtla evidence; plant at 1 kHz | M |
| O10 | H | No time synchronisation (OBC clock = engine `now_ns`) | PPS from the IEU + TimeSync epoch; OBC disciplines its timer; offset recorded every cycle | M |
| O11 | H | No bus-level fault injection (NACK, corrupt CRC, CAN error frames, stuck registers, delays, brownout), although the device map lists fault points | Faults in TICK for soft OILS; at the IEU later | S / M |
| O12 | M | Device map contradicts the HAL header on almost every port (magnetometer bus, gyro chip select, tracker and GNSS UARTs, coils on CAN vs PWM) | Generate `adcs_devices.h` from the device map, or check one against the other in CI | S |
| O13 | M | The MATLAB `udp` backend diverges and hides a dead OBC | Deprecate it, or make it speak adcs-link/2 | S–M |
| O14 | M | HAL v1 lacks watchdog, NVM, power switching, bus reset, sample time stamps, asynchronous transfers; `tm_emit` never called | HAL v2 (B5.4, H3, F7) | M |

### HILS per device, FlatSat, acceptance

| Id | Sev. | Finding | Fix | Effort |
|---|---|---|---|---|
| O15 | H | No stimulator drivers and no closed-loop stimulus: `stimulus.m` is open-loop replay, `write_stimulus` does not exist. On an air bearing the cage must follow the *measured* platform attitude | Common `Stimulus` interface, mock backends first; cage closed-loop on platform attitude | M per stimulator |
| O16 | M | Lab facts unmeasured (`hils_bay1.toml` all `nan`); `adcs rig fit` absent | Lab commissioning campaign | hardware-paced |
| O17 | M | Star-tracker and GNSS strategy undecided | Emulate both electrically at OILS; buy optical stimulation only for a dedicated tracker functional test | M / L |
| O18 | H | No TM/TC path to a mission control system (F8) | PUS-C subset + Yamcs (XTCE MDB) | L |
| O19 | H | SILS-vs-OILS acceptance is verdict agreement only; knife-edge metrics (e.g. 0.010 vs 0.010°) flip on noise | Numeric thresholds (§5) | S |
| O20 | M | CI runs no timing: no `--oils` run, no deadline regression, no CPI sweep | Add them | S |
| O21 | M | No wire-level recording, so a HIL anomaly cannot be replayed in soft OILS | Log the TICK/OUT byte stream with wall time | S |
| O22 | L | Docs drift from results (`docs/SOFT_OILS.md` 104/107 and 6.8 % vs `results/SOFT_OILS.md` 124/126 and 7.5 %) | Generate the numbers | S |
| O23 | M | The spec's architecture is not built: `adcs-bus::loop`, `adcs-rig`, mock IEU, `rig fit` / `rig arm` | This plan's backbone | L |

## 3 · Target architecture

```
 Rig host (PREEMPT_RT Linux, isolated cores)               Interface Emulation Unit (IEU)
 adcs-rig: adcs-sim plant at 1 kHz, device emulators ─UDP─► MCU/FPGA: I2C/SPI targets, UART/RS-422,
   (the same emu:: code as SILS), loop contract §11  ◄────  CAN nodes, PWM capture + coil RL load,
   wire-level recorder, fault injector                      GPIO/PPS out, ADC sources, relays
   stimulator drivers (cage SCPI, Sun shutter,              │ flight connectors (isolated)
   star display, GNSS UDP/PPS)                              ▼
   Yamcs (XTCE MDB) ◄── TM/TC (CCSDS space packets) ──►  OBC (board target, HAL v2 BSP, PPS-disciplined clock)
```

- **The engine is the plant server.** `adcs-rig` reuses `adcs-sim` behind `adcs-bus::loop`, exactly as SILS does, so a scenario runs unchanged on every rung.
- **One protocol: adcs-link/2 = the spec's loop contract.**
  - Header: version, sequence, tick, `t_ns`. CRC-32, heartbeat, FaultInject, TimeSync.
  - Two transports: lockstep in-process/pipe for deterministic regression, and UDP free-running for the rig, with lost and late datagrams counted.
- **HAL v2:**
  - time-stamped samples;
  - asynchronous/DMA completion;
  - watchdog;
  - NVM with CRC;
  - power switches and bus reset;
  - a working `tm_emit`;
  - PPS capture.
- **Time.** The rig owns time. The IEU drives PPS, TimeSync carries the epoch, the OBC disciplines its own timer, and the offset is logged every cycle.
- **Device emulation.** Transaction-level, in the IEU. An I2C/SPI target answers from a double-buffered register image that the host updates at 1 kHz; UART frames, CAN nodes and PWM capture into a coil-equivalent RL load.

| Adopt | Why |
|---|---|
| Renode (STM32F4 platform) as a second soft-OILS backend | Real peripherals and IRQs, deterministic virtual time; QEMU + plugin stays for fast exact counts |
| Yamcs + yamcs-pymdb (XTCE) | Open-source mission control with CCSDS space packets, COP-1, PUS preprocessing |
| NOS3 patterns | Per-device simulators on a software bus; ground system in the loop |
| PREEMPT_RT + cyclictest/rtla | Rig-host determinism evidence |
| ECSS-E-ST-40-07C (SMP) | Only if models must go into an ESA-class simulator |

| Build | Why |
|---|---|
| `adcs-bus::loop`, `adcs-rig`, IEU firmware, board BSP, adcs-link/2, stimulator drivers, `rig fit`/`arm` | Specific to TRI-NETRA's device set and its refusal-by-name model |

## 4 · Phases

| Phase | Entry | Work | Exit |
|---|---|---|---|
| **B0 · Soft-OILS hardening** (2–3 weeks, no hardware) | — | O2, O12, O19, O20, O21; bus faults in TICK (O11); adcs-link/2 with sequence + CRC-32; MATLAB `udp` deprecated; real STM32F4 limits in a linker check | CI runs ≥ 5 scenarios with `--oils` and a deadline-margin regression; every device-map fault point injectable; SILS vs soft OILS within §5 thresholds |
| **B1 · Renode + free-running OBC** (3–4 weeks) | B0 | STM32F4 Renode platform with the real BSP drivers; peripherals backed by the engine over a socket; timer-IRQ-driven step; HAL v2 draft; integer-tick rate groups (F3) | Same verdicts as QEMU lockstep on the full matrix; ±50 µs injected jitter changes no verdict |
| **B2 · Board OILS** (4–6 weeks after the board arrives; H1) | Board chosen; IEU v0 (MCU dev board with I2C/SPI target, CAN, UART, PPS GPIO); rig host on PREEMPT_RT | `fsw/targets/stm32f4/` with a cycle-counter trailer; CPI and WCET calibration (O1, H2); `adcs-rig` at 1 kHz; PPS + TimeSync; Yamcs on CCSDS packets | Rig cyclictest p99.99 < 50 µs, max < 100 µs over 24 h; lost/late datagrams < 1e-5, all logged; OBC–rig offset < 100 µs; WCET < 50 % of the period and soft-OILS prediction within ±20 %; detumble → nadir verdicts match SILS |
| **B3 · Per-device HILS** (hardware-paced) | B2; the relevant `hils_bay1.toml` fields measured | Polarity and phasing for every device → magnetometer in the cage → coils (dipole by reference fluxgate) → Sun sensors under the Sun simulator → wheels/rings on a dynamometer | Per device: calibration residuals within the datasheet; device-map line flipped from emulated to real; campaign still matches SILS |
| **B4 · Closed loop on the air bearing** | Balanced platform with measured residual torque; optical metrology; cage closed loop on platform attitude | Lab-twin campaign (bearing inertia, residual gravity, drag, cage error); detumble and pointing on the bearing | HILS vs lab twin within §5; e-stop and interlocks tested independently of software |
| **B5 · Day-in-the-life FlatSat** | B2–B4; Yamcs MDB complete | ≥ 24 h (≥ 15 orbits) with eclipses, mode changes, TC uploads, a safe-mode entry and recovery, injected faults, a reset or brownout | Zero unexplained anomalies; every difference from SILS has a ledger cause |

These map onto the roadmap's U-phases:
- B0 lands with U0/U4.
- B1 lands with U4.
- B2–B5 are U5.

### Equipment (capability needs, indicative)

| Item | Need |
|---|---|
| Rig host | x86, ≥ 8 cores, PREEMPT_RT, isolated cores, NIC with hardware time stamps |
| IEU | STM32H7-class MCU or small FPGA: ≥ 2 I2C targets, 1 SPI target, 2 UART/RS-422, CAN-FD, 8 PWM capture (≤ 1 µs), PPS out, ADC sources; about 1 kV galvanic isolation |
| Reference OBC | STM32F4 class or the flight OBC, with an SWD probe |
| Programmable supply | For brownout injection |
| Helmholtz cage | ±200 µT; ≤ 1 % uniformity over a 0.3–0.4 m cube; ≥ 10 Hz bandwidth; bipolar SCPI supplies; reference fluxgate < 1 nT rms |
| Air bearing | Spherical, ≥ ±30° tilt, automatic balancing to about 1 µm, 100 Hz optical metrology at ≤ 1e-4 rad |
| Sun simulator | AM0-class lamp, collimation < 1°, shutter |
| Star stimulator (optional) | Collimated micro-display, ≥ 30 Hz, < 30 ms latency |
| GNSS simulator (optional) | Single-constellation L1 with UDP trajectory input and PPS |
| Wheel dynamometer | — |

## 5 · Acceptance thresholds (to be recorded as decisions)

| Comparison | Threshold |
|---|---|
| SILS vs soft OILS (lockstep), zero latency | Bit-identical (as today) |
| SILS vs soft OILS, with the latency model | All verdicts equal, except metrics within 5 % of their requirement, which are flagged knife-edge and judged on a 12-seed distribution; per-metric difference ≤ max(5 %, 3σ of the seed spread); mode timeline identical within one control period |
| Soft OILS vs board OILS | Same instruction count for the same binary; measured cycles / (count × CPI_cal) within ±10 %; deadline margin ≥ 40 % of the period at measured worst case; zero overruns over 24 h; attitude and rate RMS difference ≤ 2× sensor noise; mode transitions within ±2 periods |
| OILS vs HILS | Magnetometer in the cage: bias/scale residual < 2 % / 1°; closed-loop detumble time within ±20 % of the lab twin over 5 repeats; pointing on the bearing within the lab twin's 95th percentile; every exceedance a ledger line with a cause |
| Link and real time | 10⁶ random frames: no hang, no unanswered frame; 1 % injected datagram loss gives counted hold-last behaviour and no verdict flip in detumble; PPS offset < 100 µs |

## Sources

- ECSS-E-TM-10-21A, System modelling and simulation: https://ecss.nl/wp-content/uploads/standards/ecss-e/ECSS-E-TM-10-21A16April2010.pdf
- ESA SVF / ATB practice: https://indico.esa.int/event/94/contributions/3592/attachments/2867/3332/1630_-_The_SVF-Lite_Configuration.pdf ; https://indico.esa.int/event/109/sessions/79/attachments/249/286/Functional_Engineering_Simulators.pdf
- ECSS-E-ST-10-03C Rev.1, Testing: https://ecss.nl/standard/ecss-e-st-10-03c-rev-1-testing/
- ECSS-E-ST-40-07C (SMP) and Rev.1 (2025): https://ecss.nl/standard/ecss-e-st-40-07c-simulation-modelling-platform-2-march-2020/
- NASA NOS3: https://nos3.readthedocs.io/en/v1_07_03/ ; https://ntrs.nasa.gov/api/citations/20240004699/downloads/20240416_FSW_Workshop_NOS3.pdf
- NASA 42: https://software.nasa.gov/software/GSC-16720-1
- Basilisk: https://hanspeterschaub.info/basilisk/
- Renode: https://renode.readthedocs.io/en/latest/introduction/supported-boards.html
- QEMU icount: https://qemu.googlesource.com/qemu/+/refs/heads/staging-8.1/docs/devel/tcg-icount.rst
- PREEMPT_RT, cyclictest, rtla: https://wiki.archlinux.org/title/Realtime_kernel
- Speedgoat / dSPACE satellite HIL: https://www.speedgoat.com/user-stories/aalto-university
- Helmholtz cage and air-bearing CubeSat HIL: https://doaj.org/article/90d66800f437462bac377592d83b7aa9 ; https://tomgra.folk.ntnu.no/papers/Olsen_IAC_2021.pdf ; https://strathprints.strath.ac.uk/48665/7/Post_etal_JIAS_2014_Design_and_construction_of_a_magnetic_field_simulator.pdf
- Star-tracker optical stimulators: https://arxiv.org/pdf/2407.02172 ; https://www.terma.com/media/twadrwcq/2-pager-_terma-dogse.pdf
- GNSS HIL simulators: https://www.rohde-schwarz.com/ca/_56279-646976.html
- Yamcs: https://www.spaceapplications.com/wp-content/uploads/2023/11/Product-Sheet_YAMCS.pdf
