# Virtual OBC loop

Owner: Agastya. `tools/engine.py vobc` -- the Rust engine drives the flight software over
adcs-link/1 (fsw/targets/link) running as a separate host process and as bare-metal firmware on an
emulated Cortex-M4F (QEMU mps2-an386, arm-none-eabi-gcc + newlib for C, rustc thumbv7em-none-eabihf for
Rust), and compares every recorded sample with the in-process build (600 s per scenario).

| scenario | reference | virtual OBC | result | wall [s] |
|---|---|---|---|---:|
| detumble_ais | c (in-process) | obc-posix | bit-identical | 0.3 |
| detumble_ais | rust (in-process) | obc-posix-rs | bit-identical | 0.2 |
| detumble_ais | c (in-process) | qemu | bit-identical | 18.2 |
| detumble_ais | rust (in-process) | qemu-rs | bit-identical | 17.4 |
| detumble_ais | c (in-process) | rust | bit-identical | 0.2 |
| mission_ais | c (in-process) | obc-posix | bit-identical | 0.2 |
| mission_ais | rust (in-process) | obc-posix-rs | bit-identical | 0.2 |
| mission_ais | c (in-process) | qemu | bit-identical | 14.2 |
| mission_ais | rust (in-process) | qemu-rs | bit-identical | 15.4 |
| mission_ais | c (in-process) | rust | bit-identical | 0.1 |
| fine_hold_img | c (in-process) | obc-posix | bit-identical | 0.6 |
| fine_hold_img | rust (in-process) | obc-posix-rs | bit-identical | 0.5 |
| fine_hold_img | c (in-process) | qemu | bit-identical | 55.9 |
| fine_hold_img | rust (in-process) | qemu-rs | bit-identical | 61.1 |
| fine_hold_img | c (in-process) | rust | bit-identical | 0.3 |
| slew_img | c (in-process) | obc-posix | bit-identical | 0.6 |
| slew_img | rust (in-process) | obc-posix-rs | bit-identical | 0.5 |
| slew_img | c (in-process) | qemu | bit-identical | 62.8 |
| slew_img | rust (in-process) | qemu-rs | bit-identical | 63.3 |
| slew_img | c (in-process) | rust | bit-identical | 0.3 |
| fine_hold_cmg | c (in-process) | obc-posix | bit-identical | 0.6 |
| fine_hold_cmg | rust (in-process) | obc-posix-rs | bit-identical | 0.6 |
| fine_hold_cmg | c (in-process) | qemu | bit-identical | 64.2 |
| fine_hold_cmg | rust (in-process) | qemu-rs | bit-identical | 68.9 |
| fine_hold_cmg | c (in-process) | rust | bit-identical | 0.3 |
| fine_hold_fmr_rcs | c (in-process) | obc-posix | bit-identical | 0.4 |
| fine_hold_fmr_rcs | rust (in-process) | obc-posix-rs | bit-identical | 0.4 |
| fine_hold_fmr_rcs | c (in-process) | qemu | bit-identical | 57.5 |
| fine_hold_fmr_rcs | rust (in-process) | qemu-rs | bit-identical | 57.1 |
| fine_hold_fmr_rcs | c (in-process) | rust | bit-identical | 0.5 |
| sun_spin_ais | c (in-process) | obc-posix | bit-identical | 0.2 |
| sun_spin_ais | rust (in-process) | obc-posix-rs | bit-identical | 0.2 |
| sun_spin_ais | c (in-process) | qemu | bit-identical | 15.3 |
| sun_spin_ais | rust (in-process) | qemu-rs | bit-identical | 13.6 |
| sun_spin_ais | c (in-process) | rust | bit-identical | 0.1 |
