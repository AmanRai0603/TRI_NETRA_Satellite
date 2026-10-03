# Mutation testing: the flight software's control and estimation

> **Answer first.** 685 of 700 mutants caught: a kill rate of **97.9 %** (floor 95 %). 1 did not compile.

> **Kind:** generated (`python3 tools/mutation.py`, cargo-mutants 27.1.0) · **Files:** fsw-rs/src/ctl.rs, fsw-rs/src/est.rs

A mutant is the code with one small fault put in on purpose; it is *caught* when a test fails.
A missed mutant is a fault the unit tests would let through: either a test is owed, or the
change makes no observable difference (an equivalent mutant).

| file | function | caught | missed | kill rate |
|---|---|---:|---:|---:|
| src/ctl.rs | `guidance` | 136 | 3 | 98 % |
| src/ctl.rs | `sat_dipole` | 8 | 3 | 73 % |
| src/ctl.rs | `sun_spin` | 43 | 2 | 96 % |
| src/est.rs | `triad` | 26 | 2 | 93 % |
| src/ctl.rs | `boresight_offset` | 17 | 1 | 94 % |
| src/ctl.rs | `mtq_avanzini` | 32 | 1 | 97 % |
| src/ctl.rs | `mtq_tango` | 13 | 1 | 93 % |
| src/est.rs | `Mekf::update3` | 61 | 1 | 98 % |
| src/est.rs | `quest` | 30 | 1 | 97 % |
| src/ctl.rs | `bdot` | 10 | 0 | 100 % |
| src/ctl.rs | `control_law` | 64 | 0 | 100 % |
| src/ctl.rs | `gen_bdot` | 16 | 0 | 100 % |
| src/ctl.rs | `mtq_boresight` | 7 | 0 | 100 % |
| src/ctl.rs | `mtq_celani` | 16 | 0 | 100 % |
| src/ctl.rs | `mtq_err` | 6 | 0 | 100 % |
| src/ctl.rs | `mtq_lovera` | 16 | 0 | 100 % |
| src/ctl.rs | `mtq_pd` | 13 | 0 | 100 % |
| src/ctl.rs | `sun_spin_deruiter` | 48 | 0 | 100 % |
| src/ctl.rs | `torque2dipole` | 6 | 0 | 100 % |
| src/ctl.rs | `yaw_flip` | 12 | 0 | 100 % |
| src/est.rs | `Mekf::predict` | 77 | 0 | 100 % |
| src/est.rs | `Mekf::quat` | 23 | 0 | 100 % |
| src/est.rs | `Mekf::vector` | 4 | 0 | 100 % |
| src/est.rs | `latency` | 1 | 0 | 100 % |

## Missed mutants

- `src/ctl.rs:104:10: replace < with <= in boresight_offset`
- `src/ctl.rs:166:14: replace < with <= in sat_dipole`
- `src/ctl.rs:166:14: replace < with == in sat_dipole`
- `src/ctl.rs:167:20: replace < with <= in sat_dipole`
- `src/ctl.rs:202:14: replace < with <= in sun_spin`
- `src/ctl.rs:203:14: replace < with <= in sun_spin`
- `src/ctl.rs:253:17: replace * with / in mtq_avanzini`
- `src/ctl.rs:273:32: replace * with / in mtq_tango`
- `src/ctl.rs:49:18: replace > with >= in guidance`
- `src/ctl.rs:68:26: replace < with <= in guidance`
- `src/ctl.rs:73:27: replace < with <= in guidance`
- `src/est.rs:109:32: replace > with >= in triad`
- `src/est.rs:110:32: replace > with >= in triad`
- `src/est.rs:142:31: replace > with >= in quest`
- `src/est.rs:55:47: replace > with >= in Mekf::update3`
