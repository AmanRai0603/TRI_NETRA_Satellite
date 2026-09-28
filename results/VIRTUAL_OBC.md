# Virtual OBC loop

Owner: Agastya. `tools/engine.py vobc` -- the Rust engine drives the flight software over
adcs-link/1 (fsw/targets/link) running as a separate host process and as bare-metal firmware on an
emulated Cortex-M4F (QEMU mps2-an386, arm-none-eabi-gcc + newlib for C, rustc thumbv7em-none-eabihf for
Rust), and compares every recorded sample with the in-process build (600 s per scenario).

| scenario | reference | virtual OBC | result | wall [s] |
|---|---|---|---|---:|
| detumble_ais | c (in-process) | obc-posix | bit-identical | 0.2 |
| detumble_ais | rust (in-process) | obc-posix-rs | bit-identical | 0.2 |
| detumble_ais | c (in-process) | qemu | bit-identical | 19.8 |
| detumble_ais | rust (in-process) | qemu-rs | bit-identical | 21.4 |
| detumble_ais | c (in-process) | rust | bit-identical | 0.0 |
| mission_ais | c (in-process) | obc-posix | bit-identical | 0.2 |
| mission_ais | rust (in-process) | obc-posix-rs | bit-identical | 0.2 |
| mission_ais | c (in-process) | qemu | bit-identical | 25.6 |
| mission_ais | rust (in-process) | qemu-rs | bit-identical | 21.2 |
| mission_ais | c (in-process) | rust | bit-identical | 0.0 |
| fine_hold_img | c (in-process) | obc-posix | bit-identical | 0.6 |
| fine_hold_img | rust (in-process) | obc-posix-rs | bit-identical | 0.6 |
| fine_hold_img | c (in-process) | qemu | bit-identical | 73.4 |
| fine_hold_img | rust (in-process) | qemu-rs | bit-identical | 80.8 |
| fine_hold_img | c (in-process) | rust | bit-identical | 0.2 |
| slew_img | c (in-process) | obc-posix | bit-identical | 0.5 |
| slew_img | rust (in-process) | obc-posix-rs | bit-identical | 0.6 |
| slew_img | c (in-process) | qemu | bit-identical | 64.3 |
| slew_img | rust (in-process) | qemu-rs | bit-identical | 74.4 |
| slew_img | c (in-process) | rust | bit-identical | 0.2 |
| fine_hold_cmg | c (in-process) | obc-posix | bit-identical | 0.7 |
| fine_hold_cmg | rust (in-process) | obc-posix-rs | bit-identical | 0.7 |
| fine_hold_cmg | c (in-process) | qemu | bit-identical | 80.7 |
| fine_hold_cmg | rust (in-process) | qemu-rs | bit-identical | 82.9 |
| fine_hold_cmg | c (in-process) | rust | bit-identical | 0.2 |
| fine_hold_fmr_rcs | c (in-process) | obc-posix | bit-identical | 0.5 |
| fine_hold_fmr_rcs | rust (in-process) | obc-posix-rs | bit-identical | 0.5 |
| fine_hold_fmr_rcs | c (in-process) | qemu | bit-identical | 64.0 |
| fine_hold_fmr_rcs | rust (in-process) | qemu-rs | bit-identical | 65.4 |
| fine_hold_fmr_rcs | c (in-process) | rust | bit-identical | 0.2 |
| sun_spin_ais | c (in-process) | obc-posix | bit-identical | 0.1 |
| sun_spin_ais | rust (in-process) | obc-posix-rs | bit-identical | 0.1 |
| sun_spin_ais | c (in-process) | qemu | bit-identical | 20.3 |
| sun_spin_ais | rust (in-process) | qemu-rs | bit-identical | 20.4 |
| sun_spin_ais | c (in-process) | rust | bit-identical | 0.0 |
