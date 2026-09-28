# Virtual OBC loop

Owner: Agastya. `tools/engine.py vobc` -- the Rust engine drives the flight software over
adcs-link/1 (fsw/targets/link) running as a separate host process and as bare-metal firmware on an
emulated Cortex-M4F (QEMU mps2-an386, arm-none-eabi-gcc + newlib for C, rustc thumbv7em-none-eabihf for
Rust), and compares every recorded sample with the in-process build (300 s per scenario).

| scenario | reference | virtual OBC | result | wall [s] |
|---|---|---|---|---:|
| detumble_ais | c (in-process) | obc-posix | bit-identical | 0.2 |
| detumble_ais | rust (in-process) | obc-posix-rs | bit-identical | 0.2 |
| detumble_ais | c (in-process) | qemu | bit-identical | 13.5 |
| detumble_ais | rust (in-process) | qemu-rs | bit-identical | 10.8 |
| detumble_ais | c (in-process) | rust | bit-identical | 0.1 |
| fine_hold_img | c (in-process) | obc-posix | bit-identical | 0.4 |
| fine_hold_img | rust (in-process) | obc-posix-rs | bit-identical | 0.4 |
| fine_hold_img | c (in-process) | qemu | bit-identical | 33.1 |
| fine_hold_img | rust (in-process) | qemu-rs | bit-identical | 35.5 |
| fine_hold_img | c (in-process) | rust | bit-identical | 0.2 |
