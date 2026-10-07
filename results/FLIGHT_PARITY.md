# Flight parity: the flight software from the design against the hand-written software

Owner: Agastya. `tools/flight_parity.py` compares the stored results of the flight software built from the design
(docs/PLAN_2_0.md S6: the algorithms written by `tools/flight_build.py`) with the results of the hand-written
software it replaces, flown with the same commands (`engine.py campaign`, `engine.py oils`). It flies nothing.
Identical means no value further apart than 1e-12 relative; on the soft-OILS firmware the deadlines and verdicts
are judged, and the instruction counts and metric differences are shown.

| set | baseline | current |
|---|---|---|
| c | `.../scratchpad/s6baseline` | `the tree` |
| rust | `.../scratchpad/s6baseline` | `.../s6run/rust` |
| qemu-rs | `.../s6run/qemu_rs_pre` | `.../s6run/qemu_rs` |

**2 regression(s):**

- c soft OILS sun_spin_ais oils: verdict(s) changed: ['spin_rate_error_last_orbit (FAIL -> pass)', 'sun_spin_entry (FAIL -> pass)']
- qemu-rs soft OILS sun_spin_ais oils: verdict(s) changed: ['sun_spin_entry (pass -> FAIL)']

Not judged against a baseline:

- c soft OILS target_img oils: not in the baseline; judged on its own deadlines only

## What the run found

- Sets: c = the C flight software (campaigns --fsw c, soft OILS --fsw qemu) against the committed results of 1.0.0's hand-written C; rust = the Rust flight software's campaigns (--fsw rust) against the same committed results; qemu-rs = soft OILS --fsw qemu-rs against the hand-written Rust firmware of 85d8e4f (fsw-rs before S6 step 4), flown the same day with the same engine (ADCS_REPO pointing at that build).
- SILS: every campaign value, draw, statistic, ECSS interpretation and verdict is bit-identical for C and for Rust (266,448 values and 11,401 verdicts per set, 4,516 runs, none failed); results/ENGINE_CAMPAIGNS.md and engine_campaigns.json come out byte for byte. The SILS half of every soft-OILS scenario is bit-identical too (1,449 metrics).
- Timing: the generated firmware runs more instructions per step, C +4.4 % to +20.2 % mean (+9.0 % on average) and +4.3 % to +9.3 % max; Rust +5.2 % to +16.5 % mean and +3.6 % to +26.3 % max. The cause is the translation of the pseudocode's int to int64_t / i64: on the 32-bit Cortex-M4 the byte handling of 09_drivers (UART framing, CRC-16, CAN unpacking) costs several times as much, and fixed-size arrays are passed by value (pc_a64i 512 bytes, pc_a96i 768 bytes per call). On the host (callgrind, fine_hold_img, 120 s) the drivers' framing and CRC-16 rise from 2.3 to 8.9 million instructions, guidance from 0.46 to 1.03 million, the estimator's update and prediction by 4 % and 11 %. The same by-value arrays raise the deepest firmware stack from 4,728 to 12,208 bytes (adcs_drv_read 5,880 bytes, then drivers_drv_read and drivers_uart_frame); it fits the 32,768 reserved (37 %).
- mission_ais on qemu-rs, max +26.3 %: one step, 3292.0 s, where nadir_mtq's first steps coincide with the every-fifth-step heavy step (1,032,496 instructions); the switch from detumble came 2.2 s earlier than before (3291.2 s against 3293.4 s) because the command latency moved, so the phase differs. The steady heavy steps are +4 % (849 k against 817 k).
- Deadlines: no overrun in any of the 96 soft-OILS runs; the smallest worst-case margin goes from 36.35 to 35.76 ms (C) and from 34.01 to 33.21 ms (Rust) against the 50 ms deadline of the 100 ms steps; CPU load at most 7.9 % (C) and 9.5 % (Rust).
- sun_spin_ais: the soft-OILS verdicts on sun_spin_entry (req 95 min) and spin_rate_error_last_orbit (req 0.5 deg/s) move with the command latency, not with the algorithms: its SILS run is bit-identical, and before S6 the same algorithms already gave 131.9 min (C firmware, FAIL) and 67.2 min (Rust firmware, pass). Now C gives 64.0 min and 0.126 deg/s (both pass), Rust 104.4 min (FAIL) and 0.208 deg/s (pass). The spin-up and Sun-spin modes alternate, and when Sun-spin is first held depends on the step at which each command lands. They stay listed above as verdict changes: a change of timing, not of the algorithms.
- target_img: the committed results hold its SILS run only (no soft-OILS run was stored at P11); it is flown now on both firmwares, with no overrun and a worst-case margin of 37.05 ms (C) and 35.15 ms (Rust; 35.90 ms on the hand-written Rust firmware).

## SILS campaigns

| set | campaign | scenario | fsw before / now | runs | values compared | differ | max relative | verdicts | changed |
|---|---|---|---|---:|---:|---:|---:|---:|---:|
| c | edge_fine_img | fine_hold_img | c / c | 15 | 1809 | 0 | 0 | 75 | 0 |
| c | edge_nadir_ais | nadir_hold_ais | c / c | 17 | 991 | 0 | 0 | 68 | 0 |
| c | mc_agile_rw_rcs | agile_slew_rw_rcs | c / c | 1109 | 42195 | 0 | 0 | 1109 | 0 |
| c | mc_detumble_ais | detumble_ais | c / c | 24 | 1003 | 0 | 0 | 72 | 0 |
| c | mc_fine_img | fine_hold_img | c / c | 1109 | 134378 | 0 | 0 | 5545 | 0 |
| c | mc_nadir_ais | nadir_hold_ais | c / c | 24 | 1678 | 0 | 0 | 96 | 0 |
| c | mc_slew_cmg | slew_cmg | c / c | 1109 | 42197 | 0 | 0 | 2218 | 0 |
| c | mc_slew_img | slew_img | c / c | 1109 | 42197 | 0 | 0 | 2218 | 0 |
| rust | edge_fine_img | fine_hold_img | c / rust | 15 | 1809 | 0 | 0 | 75 | 0 |
| rust | edge_nadir_ais | nadir_hold_ais | c / rust | 17 | 991 | 0 | 0 | 68 | 0 |
| rust | mc_agile_rw_rcs | agile_slew_rw_rcs | c / rust | 1109 | 42195 | 0 | 0 | 1109 | 0 |
| rust | mc_detumble_ais | detumble_ais | c / rust | 24 | 1003 | 0 | 0 | 72 | 0 |
| rust | mc_fine_img | fine_hold_img | c / rust | 1109 | 134378 | 0 | 0 | 5545 | 0 |
| rust | mc_nadir_ais | nadir_hold_ais | c / rust | 24 | 1678 | 0 | 0 | 96 | 0 |
| rust | mc_slew_cmg | slew_cmg | c / rust | 1109 | 42197 | 0 | 0 | 2218 | 0 |
| rust | mc_slew_img | slew_img | c / rust | 1109 | 42197 | 0 | 0 | 2218 | 0 |

## Soft OILS: the deadline and the timing

| set | scenario | ticks | overruns | instructions mean before / now | change | max before / now | change | deadline margin min [ms] before / now | worst-case margin [ms] before / now |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| c | agile_slew_cmg | 6001 | 0 | 4.209e+05 / 4.402e+05 | +4.59 % | 9.877e+05 / 1.032e+06 | +4.49 % | 91.06 / 90.73 | 36.6 / 36.07 |
| c | agile_slew_fmr_rcs | 6001 | 0 | 3.717e+05 / 4.062e+05 | +9.28 % | 9.378e+05 / 9.987e+05 | +6.50 % | 91.95 / 91.5 | 37.71 / 36.99 |
| c | agile_slew_img | 6001 | 0 | 3.649e+05 / 3.979e+05 | +9.05 % | 9.345e+05 / 9.935e+05 | +6.31 % | 92.1 / 91.66 | 37.88 / 37.18 |
| c | agile_slew_rw_rcs | 6001 | 0 | 3.753e+05 / 4.099e+05 | +9.21 % | 9.44e+05 / 1.005e+06 | +6.46 % | 91.9 / 91.45 | 37.64 / 36.91 |
| c | agile_slew_vscmg | 6001 | 0 | 4.392e+05 / 4.586e+05 | +4.40 % | 1.007e+06 / 1.052e+06 | +4.47 % | 90.92 / 90.58 | 36.37 / 35.83 |
| c | detumble_ais | 86101 | 0 | 1.661e+05 / 1.811e+05 | +8.99 % | 5.334e+05 / 5.632e+05 | +5.59 % | 195.5 / 195.3 | 93.05 / 92.69 |
| c | detumble_ais_bangbang | 86101 | 0 | 1.646e+05 / 1.794e+05 | +9.03 % | 5.334e+05 / 5.633e+05 | +5.62 % | 195.5 / 195.3 | 93.05 / 92.69 |
| c | detumble_ais_mag | 86101 | 0 | 1.656e+05 / 1.806e+05 | +9.02 % | 5.335e+05 / 5.633e+05 | +5.58 % | 195.5 / 195.3 | 93.05 / 92.69 |
| c | detumble_img | 172201 | 0 | 1.245e+05 / 1.485e+05 | +19.33 % | 5.494e+05 / 5.999e+05 | +9.18 % | 94.97 / 94.59 | 42.47 / 41.86 |
| c | detumble_rcs | 18001 | 0 | 1.24e+05 / 1.49e+05 | +20.20 % | 5.6e+05 / 6.12e+05 | +9.29 % | 94.76 / 94.37 | 42.21 / 41.59 |
| c | fault_coil_ais | 86101 | 0 | 1.661e+05 / 1.811e+05 | +9.00 % | 5.336e+05 / 5.632e+05 | +5.55 % | 195.5 / 195.3 | 93.04 / 92.69 |
| c | fault_gimbal_cmg | 57401 | 0 | 4.129e+05 / 4.351e+05 | +5.36 % | 9.87e+05 / 1.036e+06 | +4.98 % | 91.06 / 90.7 | 36.61 / 36.02 |
| c | fault_gps_img | 57401 | 0 | 3.598e+05 / 3.952e+05 | +9.86 % | 9.342e+05 / 9.969e+05 | +6.72 % | 92.11 / 91.64 | 37.88 / 37.14 |
| c | fault_gyro_ais | 86101 | 0 | 2.699e+05 / 2.875e+05 | +6.55 % | 8.39e+05 / 8.752e+05 | +4.30 % | 193.2 / 192.9 | 89.41 / 88.98 |
| c | fault_st_img | 57401 | 0 | 3.159e+05 / 3.509e+05 | +11.10 % | 9.293e+05 / 9.922e+05 | +6.77 % | 92.14 / 91.67 | 37.94 / 37.19 |
| c | fault_valve_rcs | 57401 | 0 | 3.689e+05 / 4.062e+05 | +10.13 % | 9.454e+05 / 1.009e+06 | +6.71 % | 91.89 / 91.42 | 37.62 / 36.87 |
| c | fault_wheel_img | 57401 | 0 | 3.544e+05 / 3.902e+05 | +10.10 % | 9.328e+05 / 9.959e+05 | +6.77 % | 92.12 / 91.65 | 37.9 / 37.15 |
| c | fine_hold_cmg | 57401 | 0 | 4.13e+05 / 4.351e+05 | +5.36 % | 9.871e+05 / 1.037e+06 | +5.01 % | 91.06 / 90.69 | 36.61 / 36.02 |
| c | fine_hold_fmr | 57401 | 0 | 3.607e+05 / 3.966e+05 | +9.95 % | 9.348e+05 / 9.974e+05 | +6.70 % | 92.1 / 91.63 | 37.88 / 37.13 |
| c | fine_hold_fmr_rcs | 57401 | 0 | 3.638e+05 / 4.011e+05 | +10.25 % | 9.367e+05 / 1.002e+06 | +6.97 % | 91.96 / 91.47 | 37.73 / 36.95 |
| c | fine_hold_img | 57401 | 0 | 3.605e+05 / 3.964e+05 | +9.95 % | 9.343e+05 / 9.966e+05 | +6.67 % | 92.1 / 91.64 | 37.88 / 37.14 |
| c | fine_hold_img_lqr | 57401 | 0 | 3.607e+05 / 3.964e+05 | +9.91 % | 9.36e+05 / 9.967e+05 | +6.49 % | 92.09 / 91.64 | 37.86 / 37.14 |
| c | fine_hold_img_smc | 57401 | 0 | 3.635e+05 / 3.993e+05 | +9.86 % | 9.387e+05 / 9.996e+05 | +6.50 % | 92.07 / 91.62 | 37.83 / 37.11 |
| c | fine_hold_rw_rcs | 57401 | 0 | 3.677e+05 / 4.05e+05 | +10.16 % | 9.432e+05 / 1.009e+06 | +7.00 % | 91.91 / 91.42 | 37.65 / 36.86 |
| c | fine_hold_vscmg | 57401 | 0 | 4.342e+05 / 4.564e+05 | +5.10 % | 1.008e+06 / 1.057e+06 | +4.81 % | 90.9 / 90.54 | 36.35 / 35.77 |
| c | mission_ais | 143501 | 0 | 2.589e+05 / 2.763e+05 | +6.71 % | 7.172e+05 / 7.517e+05 | +4.81 % | 194.1 / 193.9 | 90.86 / 90.45 |
| c | mission_cmg | 172201 | 0 | 3.545e+05 / 3.772e+05 | +6.39 % | 9.866e+05 / 1.036e+06 | +4.96 % | 91.07 / 90.7 | 36.61 / 36.03 |
| c | mission_fmr | 172201 | 0 | 3.106e+05 / 3.44e+05 | +10.78 % | 9.372e+05 / 9.969e+05 | +6.37 % | 92.08 / 91.64 | 37.85 / 37.14 |
| c | mission_fmr_rcs | 172201 | 0 | 3.134e+05 / 3.481e+05 | +11.10 % | 9.406e+05 / 1.001e+06 | +6.47 % | 91.93 / 91.47 | 37.68 / 36.95 |
| c | mission_img | 172201 | 0 | 3.186e+05 / 3.525e+05 | +10.62 % | 9.352e+05 / 9.977e+05 | +6.69 % | 92.1 / 91.63 | 37.87 / 37.13 |
| c | mission_rw_rcs | 172201 | 0 | 3.252e+05 / 3.605e+05 | +10.85 % | 9.434e+05 / 1.007e+06 | +6.76 % | 91.91 / 91.43 | 37.65 / 36.89 |
| c | mission_vscmg | 172201 | 0 | 3.721e+05 / 3.948e+05 | +6.09 % | 1.008e+06 / 1.058e+06 | +4.93 % | 90.91 / 90.54 | 36.36 / 35.76 |
| c | nadir_hold_ais | 86101 | 0 | 2.7e+05 / 2.877e+05 | +6.54 % | 7.18e+05 / 7.509e+05 | +4.59 % | 194.1 / 193.9 | 90.85 / 90.46 |
| c | nadir_hold_ais_css | 86101 | 0 | 2.699e+05 / 2.876e+05 | +6.55 % | 7.177e+05 / 7.497e+05 | +4.45 % | 194.1 / 193.9 | 90.85 / 90.47 |
| c | safe_mode_ais | 86101 | 0 | 2.6e+05 / 2.775e+05 | +6.69 % | 7.18e+05 / 7.509e+05 | +4.59 % | 194.1 / 193.9 | 90.85 / 90.46 |
| c | slew_cmg | 6001 | 0 | 4.194e+05 / 4.405e+05 | +5.03 % | 9.835e+05 / 1.03e+06 | +4.76 % | 91.09 / 90.74 | 36.65 / 36.09 |
| c | slew_fmr | 6001 | 0 | 3.671e+05 / 4.02e+05 | +9.50 % | 9.32e+05 / 9.916e+05 | +6.40 % | 92.12 / 91.68 | 37.91 / 37.2 |
| c | slew_fmr_rcs | 6001 | 0 | 3.702e+05 / 4.065e+05 | +9.80 % | 9.34e+05 / 9.964e+05 | +6.68 % | 91.98 / 91.51 | 37.76 / 37.01 |
| c | slew_img | 6001 | 0 | 3.669e+05 / 4.017e+05 | +9.48 % | 9.3e+05 / 9.913e+05 | +6.59 % | 92.14 / 91.68 | 37.93 / 37.2 |
| c | slew_img_lqr | 6001 | 0 | 3.67e+05 / 4.017e+05 | +9.46 % | 9.314e+05 / 9.93e+05 | +6.61 % | 92.13 / 91.67 | 37.92 / 37.19 |
| c | slew_img_smc | 6001 | 0 | 3.697e+05 / 4.046e+05 | +9.42 % | 9.335e+05 / 9.957e+05 | +6.67 % | 92.11 / 91.65 | 37.89 / 37.15 |
| c | slew_rw_rcs | 6001 | 0 | 3.718e+05 / 4.08e+05 | +9.73 % | 9.384e+05 / 9.992e+05 | +6.48 % | 91.94 / 91.49 | 37.7 / 36.98 |
| c | slew_vscmg | 6001 | 0 | 4.404e+05 / 4.616e+05 | +4.79 % | 1.004e+06 / 1.053e+06 | +4.82 % | 90.93 / 90.57 | 36.4 / 35.82 |
| c | sun_acq_rotor_img | 57401 | 0 | 2.811e+05 / 3.148e+05 | +11.96 % | 9.028e+05 / 9.639e+05 | +6.76 % | 92.34 / 91.88 | 38.26 / 37.53 |
| c | sun_fine_img | 57401 | 0 | 3.529e+05 / 3.878e+05 | +9.89 % | 9.51e+05 / 1.013e+06 | +6.52 % | 91.98 / 91.52 | 37.68 / 36.95 |
| c | sun_mtq_ais | 86101 | 0 | 2.743e+05 / 2.92e+05 | +6.43 % | 7.173e+05 / 7.511e+05 | +4.71 % | 194.1 / 193.9 | 90.86 / 90.45 |
| c | sun_spin_ais | 86101 | 0 | 1.675e+05 / 1.831e+05 | +9.29 % | 5.402e+05 / 5.714e+05 | +5.77 % | 195.4 / 195.2 | 92.96 / 92.59 |
| c | target_img | 57401 | 0 | — / 4.024e+05 | — | — / 1.005e+06 | — | — / 91.58 | — / 37.05 |
| qemu-rs | agile_slew_cmg | 6001 | 0 | 5.533e+05 / 5.931e+05 | +7.20 % | 1.174e+06 / 1.241e+06 | +5.69 % | 89.67 / 89.17 | 34.38 / 33.59 |
| qemu-rs | agile_slew_fmr_rcs | 6001 | 0 | 4.76e+05 / 5.137e+05 | +7.93 % | 1.098e+06 / 1.162e+06 | +5.88 % | 90.76 / 90.28 | 35.81 / 35.04 |
| qemu-rs | agile_slew_img | 6001 | 0 | 4.668e+05 / 5.03e+05 | +7.75 % | 1.09e+06 / 1.153e+06 | +5.79 % | 90.94 / 90.47 | 36.03 / 35.28 |
| qemu-rs | agile_slew_rw_rcs | 6001 | 0 | 4.772e+05 / 5.15e+05 | +7.91 % | 1.1e+06 / 1.165e+06 | +5.87 % | 90.74 / 90.26 | 35.78 / 35.01 |
| qemu-rs | agile_slew_vscmg | 6001 | 0 | 5.763e+05 / 6.167e+05 | +7.02 % | 1.199e+06 / 1.267e+06 | +5.60 % | 89.48 / 88.98 | 34.08 / 33.28 |
| qemu-rs | detumble_ais | 86101 | 0 | 1.846e+05 / 1.968e+05 | +6.57 % | 5.675e+05 / 5.943e+05 | +4.72 % | 195.2 / 195 | 92.64 / 92.32 |
| qemu-rs | detumble_ais_bangbang | 86101 | 0 | 1.838e+05 / 1.96e+05 | +6.59 % | 5.675e+05 / 5.943e+05 | +4.72 % | 195.2 / 195 | 92.64 / 92.32 |
| qemu-rs | detumble_ais_mag | 86101 | 0 | 1.844e+05 / 1.965e+05 | +6.58 % | 5.675e+05 / 5.943e+05 | +4.72 % | 195.2 / 195 | 92.64 / 92.32 |
| qemu-rs | detumble_img | 172201 | 0 | 1.43e+05 / 1.656e+05 | +15.81 % | 5.946e+05 / 6.381e+05 | +7.32 % | 94.63 / 94.31 | 41.93 / 41.41 |
| qemu-rs | detumble_rcs | 18001 | 0 | 1.45e+05 / 1.689e+05 | +16.51 % | 6.04e+05 / 6.491e+05 | +7.46 % | 94.43 / 94.1 | 41.69 / 41.15 |
| qemu-rs | fault_coil_ais | 86101 | 0 | 1.846e+05 / 1.967e+05 | +6.57 % | 5.675e+05 / 5.943e+05 | +4.72 % | 195.2 / 195 | 92.64 / 92.32 |
| qemu-rs | fault_gimbal_cmg | 57401 | 0 | 5.425e+05 / 5.807e+05 | +7.04 % | 1.174e+06 / 1.238e+06 | +5.42 % | 89.67 / 89.2 | 34.38 / 33.62 |
| qemu-rs | fault_gps_img | 57401 | 0 | 4.594e+05 / 4.938e+05 | +7.48 % | 1.093e+06 / 1.153e+06 | +5.51 % | 90.93 / 90.48 | 36 / 35.28 |
| qemu-rs | fault_gyro_ais | 86101 | 0 | 3.353e+05 / 3.53e+05 | +5.28 % | 9.833e+05 / 1.018e+06 | +3.55 % | 192.1 / 191.9 | 87.69 / 87.27 |
| qemu-rs | fault_st_img | 57401 | 0 | 3.998e+05 / 4.328e+05 | +8.25 % | 1.083e+06 / 1.144e+06 | +5.56 % | 91 / 90.55 | 36.11 / 35.39 |
| qemu-rs | fault_valve_rcs | 57401 | 0 | 4.674e+05 / 5.035e+05 | +7.72 % | 1.101e+06 / 1.163e+06 | +5.61 % | 90.74 / 90.28 | 35.77 / 35.04 |
| qemu-rs | fault_wheel_img | 57401 | 0 | 4.509e+05 / 4.858e+05 | +7.74 % | 1.089e+06 / 1.15e+06 | +5.63 % | 90.96 / 90.5 | 36.05 / 35.32 |
| qemu-rs | fine_hold_cmg | 57401 | 0 | 5.425e+05 / 5.806e+05 | +7.03 % | 1.174e+06 / 1.238e+06 | +5.45 % | 89.67 / 89.19 | 34.38 / 33.62 |
| qemu-rs | fine_hold_fmr | 57401 | 0 | 4.607e+05 / 4.954e+05 | +7.52 % | 1.092e+06 / 1.152e+06 | +5.51 % | 90.93 / 90.48 | 36 / 35.29 |
| qemu-rs | fine_hold_fmr_rcs | 57401 | 0 | 4.652e+05 / 5.013e+05 | +7.75 % | 1.097e+06 / 1.158e+06 | +5.62 % | 90.77 / 90.31 | 35.82 / 35.09 |
| qemu-rs | fine_hold_img | 57401 | 0 | 4.606e+05 / 4.953e+05 | +7.52 % | 1.092e+06 / 1.153e+06 | +5.51 % | 90.93 / 90.48 | 36 / 35.28 |
| qemu-rs | fine_hold_img_lqr | 57401 | 0 | 4.608e+05 / 4.955e+05 | +7.52 % | 1.093e+06 / 1.153e+06 | +5.53 % | 90.93 / 90.48 | 36 / 35.28 |
| qemu-rs | fine_hold_img_smc | 57401 | 0 | 4.631e+05 / 4.977e+05 | +7.48 % | 1.095e+06 / 1.155e+06 | +5.49 % | 90.91 / 90.46 | 35.97 / 35.25 |
| qemu-rs | fine_hold_rw_rcs | 57401 | 0 | 4.666e+05 / 5.027e+05 | +7.73 % | 1.1e+06 / 1.162e+06 | +5.62 % | 90.74 / 90.28 | 35.78 / 35.04 |
| qemu-rs | fine_hold_vscmg | 57401 | 0 | 5.698e+05 / 6.084e+05 | +6.78 % | 1.202e+06 / 1.266e+06 | +5.35 % | 89.46 / 88.98 | 34.05 / 33.28 |
| qemu-rs | mission_ais | 143501 | 0 | 3.192e+05 / 3.363e+05 | +5.35 % | 8.176e+05 / 1.032e+06 | +26.29 % | 193.4 / 191.8 | 89.66 / 87.1 |
| qemu-rs | mission_cmg | 172201 | 0 | 4.619e+05 / 4.973e+05 | +7.66 % | 1.175e+06 / 1.239e+06 | +5.43 % | 89.66 / 89.19 | 34.37 / 33.61 |
| qemu-rs | mission_fmr | 172201 | 0 | 3.93e+05 / 4.251e+05 | +8.17 % | 1.093e+06 / 1.153e+06 | +5.51 % | 90.92 / 90.47 | 35.99 / 35.28 |
| qemu-rs | mission_fmr_rcs | 172201 | 0 | 3.97e+05 / 4.305e+05 | +8.45 % | 1.098e+06 / 1.159e+06 | +5.62 % | 90.76 / 90.3 | 35.81 / 35.08 |
| qemu-rs | mission_img | 172201 | 0 | 4.041e+05 / 4.366e+05 | +8.05 % | 1.093e+06 / 1.153e+06 | +5.51 % | 90.93 / 90.48 | 36 / 35.28 |
| qemu-rs | mission_rw_rcs | 172201 | 0 | 4.096e+05 / 4.436e+05 | +8.29 % | 1.1e+06 / 1.161e+06 | +5.58 % | 90.74 / 90.28 | 35.78 / 35.05 |
| qemu-rs | mission_vscmg | 172201 | 0 | 4.84e+05 / 5.198e+05 | +7.40 % | 1.204e+06 / 1.268e+06 | +5.31 % | 89.45 / 88.98 | 34.03 / 33.27 |
| qemu-rs | nadir_hold_ais | 86101 | 0 | 3.355e+05 / 3.532e+05 | +5.28 % | 8.181e+05 / 8.506e+05 | +3.98 % | 193.4 / 193.1 | 89.66 / 89.27 |
| qemu-rs | nadir_hold_ais_css | 86101 | 0 | 3.355e+05 / 3.533e+05 | +5.28 % | 8.179e+05 / 8.503e+05 | +3.97 % | 193.4 / 193.1 | 89.66 / 89.27 |
| qemu-rs | safe_mode_ais | 86101 | 0 | 3.216e+05 / 3.388e+05 | +5.33 % | 8.181e+05 / 8.506e+05 | +3.97 % | 193.4 / 193.1 | 89.66 / 89.27 |
| qemu-rs | slew_cmg | 6001 | 0 | 5.546e+05 / 5.952e+05 | +7.32 % | 1.178e+06 / 1.245e+06 | +5.69 % | 89.64 / 89.14 | 34.33 / 33.54 |
| qemu-rs | slew_fmr | 6001 | 0 | 4.73e+05 / 5.101e+05 | +7.84 % | 1.096e+06 / 1.159e+06 | +5.78 % | 90.9 / 90.43 | 35.96 / 35.21 |
| qemu-rs | slew_fmr_rcs | 6001 | 0 | 4.775e+05 / 5.16e+05 | +8.06 % | 1.1e+06 / 1.165e+06 | +5.91 % | 90.74 / 90.26 | 35.78 / 35.01 |
| qemu-rs | slew_img | 6001 | 0 | 4.728e+05 / 5.099e+05 | +7.84 % | 1.096e+06 / 1.16e+06 | +5.80 % | 90.9 / 90.43 | 35.96 / 35.2 |
| qemu-rs | slew_img_lqr | 6001 | 0 | 4.73e+05 / 5.101e+05 | +7.84 % | 1.096e+06 / 1.16e+06 | +5.79 % | 90.9 / 90.43 | 35.96 / 35.2 |
| qemu-rs | slew_img_smc | 6001 | 0 | 4.754e+05 / 5.125e+05 | +7.80 % | 1.099e+06 / 1.162e+06 | +5.79 % | 90.88 / 90.41 | 35.93 / 35.17 |
| qemu-rs | slew_rw_rcs | 6001 | 0 | 4.773e+05 / 5.158e+05 | +8.07 % | 1.102e+06 / 1.166e+06 | +5.88 % | 90.73 / 90.25 | 35.76 / 34.99 |
| qemu-rs | slew_vscmg | 6001 | 0 | 5.817e+05 / 6.228e+05 | +7.06 % | 1.205e+06 / 1.273e+06 | +5.58 % | 89.44 / 88.94 | 34.01 / 33.21 |
| qemu-rs | sun_acq_rotor_img | 57401 | 0 | 3.648e+05 / 3.981e+05 | +9.14 % | 1.066e+06 / 1.129e+06 | +5.93 % | 91.13 / 90.66 | 36.32 / 35.57 |
| qemu-rs | sun_fine_img | 57401 | 0 | 4.374e+05 / 4.712e+05 | +7.72 % | 1.102e+06 / 1.162e+06 | +5.50 % | 90.86 / 90.41 | 35.89 / 35.17 |
| qemu-rs | sun_mtq_ais | 86101 | 0 | 3.388e+05 / 3.566e+05 | +5.24 % | 8.178e+05 / 8.502e+05 | +3.97 % | 193.4 / 193.1 | 89.66 / 89.27 |
| qemu-rs | sun_spin_ais | 86101 | 0 | 1.874e+05 / 2.007e+05 | +7.11 % | 5.684e+05 / 5.951e+05 | +4.71 % | 195.2 / 195 | 92.63 / 92.31 |
| qemu-rs | target_img | 57401 | 0 | 4.686e+05 / 5.057e+05 | +7.91 % | 1.101e+06 / 1.164e+06 | +5.70 % | 90.86 / 90.4 | 35.9 / 35.15 |

## Soft OILS: the metrics

| set | scenario | SILS differ / compared | SILS max relative | OILS differ / compared | OILS max relative | OILS worst metric | verdicts changed |
|---|---|---:|---:|---:|---:|---|---|
| c | agile_slew_cmg | 0 / 21 | 0 | 2 / 35 | 0.01444 | [oils_worst_case_margin].value | none |
| c | agile_slew_fmr_rcs | 0 / 21 | 0 | 3 / 35 | 0.01924 | [oils_worst_case_margin].value | none |
| c | agile_slew_img | 0 / 21 | 0 | 3 / 35 | 0.01852 | [oils_worst_case_margin].value | none |
| c | agile_slew_rw_rcs | 0 / 21 | 0 | 3 / 35 | 0.0193 | [oils_worst_case_margin].value | none |
| c | agile_slew_vscmg | 0 / 21 | 0 | 3 / 35 | 0.01473 | [oils_worst_case_margin].value | none |
| c | detumble_ais | 0 / 21 | 0 | 3 / 35 | 0.003817 | [oils_worst_case_margin].value | none |
| c | detumble_ais_bangbang | 0 / 21 | 0 | 3 / 35 | 0.004329 | [power_mean].value | none |
| c | detumble_ais_mag | 0 / 21 | 0 | 3 / 35 | 0.004329 | [power_peak].value | none |
| c | detumble_img | 0 / 7 | 0 | 1 / 21 | 0.01414 | [oils_worst_case_margin].value | none |
| c | detumble_rcs | 0 / 14 | 0 | 2 / 28 | 0.01466 | [oils_worst_case_margin].value | none |
| c | fault_coil_ais | 0 / 21 | 0 | 3 / 35 | 0.008818 | [power_mean].value | none |
| c | fault_gimbal_cmg | 0 / 56 | 0 | 7 / 70 | 0.01598 | [oils_worst_case_margin].value | none |
| c | fault_gps_img | 0 / 21 | 0 | 4 / 35 | 0.01971 | [oils_worst_case_margin].value | none |
| c | fault_gyro_ais | 0 / 28 | 0 | 5 / 42 | 0.004808 | [oils_worst_case_margin].value | none |
| c | fault_st_img | 0 / 56 | 0 | 8 / 70 | 0.01974 | [oils_worst_case_margin].value | none |
| c | fault_valve_rcs | 0 / 21 | 0 | 4 / 35 | 0.02008 | [oils_worst_case_margin].value | none |
| c | fault_wheel_img | 0 / 56 | 0 | 8 / 70 | 0.01983 | [oils_worst_case_margin].value | none |
| c | fine_hold_cmg | 0 / 56 | 0 | 7 / 70 | 0.01608 | [oils_worst_case_margin].value | none |
| c | fine_hold_fmr | 0 / 56 | 0 | 8 / 70 | 0.0197 | [oils_worst_case_margin].value | none |
| c | fine_hold_fmr_rcs | 0 / 56 | 0 | 8 / 70 | 0.0206 | [oils_worst_case_margin].value | none |
| c | fine_hold_img | 0 / 91 | 0 | 13 / 105 | 0.01958 | [oils_worst_case_margin].value | none |
| c | fine_hold_img_lqr | 0 / 56 | 0 | 8 / 70 | 0.0191 | [oils_worst_case_margin].value | none |
| c | fine_hold_img_smc | 0 / 56 | 0 | 8 / 70 | 0.01919 | [oils_worst_case_margin].value | none |
| c | fine_hold_rw_rcs | 0 / 56 | 0 | 8 / 70 | 0.02088 | [oils_worst_case_margin].value | none |
| c | fine_hold_vscmg | 0 / 56 | 0 | 8 / 70 | 0.01587 | [oils_worst_case_margin].value | none |
| c | mission_ais | 0 / 42 | 0 | 7 / 56 | 0.07281 | [ape_los_last_orbit].value | none |
| c | mission_cmg | 0 / 14 | 0 | 2 / 28 | 0.01591 | [oils_worst_case_margin].value | none |
| c | mission_fmr | 0 / 14 | 0 | 2 / 28 | 0.01877 | [oils_worst_case_margin].value | none |
| c | mission_fmr_rcs | 0 / 14 | 0 | 2 / 28 | 0.01922 | [oils_worst_case_margin].value | none |
| c | mission_img | 0 / 28 | 0 | 5 / 42 | 0.01966 | [oils_worst_case_margin].value | none |
| c | mission_rw_rcs | 0 / 14 | 0 | 2 / 28 | 0.3908 | [ape_los_last_half_orbit_p9973].value | none |
| c | mission_vscmg | 0 / 14 | 0 | 2 / 28 | 0.01627 | [oils_worst_case_margin].value | none |
| c | nadir_hold_ais | 0 / 42 | 0 | 7 / 56 | 0.004314 | [oils_worst_case_margin].value | none |
| c | nadir_hold_ais_css | 0 / 28 | 0 | 5 / 42 | 0.005799 | [ake_los_last_orbit].value | none |
| c | safe_mode_ais | 0 / 21 | 0 | 3 / 35 | 0.03468 | [ape_los_last_orbit].value | none |
| c | slew_cmg | 0 / 21 | 0 | 2 / 35 | 0.0191 | [ape_los_on_target_p9973].value | none |
| c | slew_fmr | 0 / 21 | 0 | 3 / 35 | 0.01872 | [oils_worst_case_margin].value | none |
| c | slew_fmr_rcs | 0 / 21 | 0 | 3 / 35 | 0.01968 | [oils_worst_case_margin].value | none |
| c | slew_img | 0 / 21 | 0 | 3 / 35 | 0.01924 | [oils_worst_case_margin].value | none |
| c | slew_img_lqr | 0 / 21 | 0 | 3 / 35 | 0.01932 | [oils_worst_case_margin].value | none |
| c | slew_img_smc | 0 / 21 | 0 | 3 / 35 | 0.01955 | [oils_worst_case_margin].value | none |
| c | slew_rw_rcs | 0 / 21 | 0 | 3 / 35 | 0.01919 | [oils_worst_case_margin].value | none |
| c | slew_vscmg | 0 / 21 | 0 | 4 / 35 | 0.01582 | [oils_worst_case_margin].value | none |
| c | sun_acq_rotor_img | 0 / 21 | 0 | 3 / 35 | 0.01899 | [oils_worst_case_margin].value | none |
| c | sun_fine_img | 0 / 21 | 0 | 4 / 35 | 0.01959 | [oils_worst_case_margin].value | none |
| c | sun_mtq_ais | 0 / 21 | 0 | 4 / 35 | 0.004425 | [oils_worst_case_margin].value | none |
| c | sun_spin_ais | 0 / 35 | 0 | 8 / 49 | 1 | [spin_rate_error_last_orbit].pass | oils spin_rate_error_last_orbit (FAIL -> pass), oils sun_spin_entry (FAIL -> pass) |
| c | target_img | 0 / 21 | 0 | — / 5 | — | — | none |
| qemu-rs | agile_slew_cmg | 0 / 21 | 0 | 2 / 35 | 0.02311 | [oils_worst_case_margin].value | none |
| qemu-rs | agile_slew_fmr_rcs | 0 / 21 | 0 | 4 / 35 | 0.06214 | [settle_time_after_slew].value | none |
| qemu-rs | agile_slew_img | 0 / 21 | 0 | 3 / 35 | 0.02086 | [oils_worst_case_margin].value | none |
| qemu-rs | agile_slew_rw_rcs | 0 / 21 | 0 | 3 / 35 | 0.02148 | [oils_worst_case_margin].value | none |
| qemu-rs | agile_slew_vscmg | 0 / 21 | 0 | 4 / 35 | 0.02346 | [oils_worst_case_margin].value | none |
| qemu-rs | detumble_ais | 0 / 21 | 0 | 4 / 35 | 0.003442 | [oils_worst_case_margin].value | none |
| qemu-rs | detumble_ais_bangbang | 0 / 21 | 0 | 3 / 35 | 0.009231 | [power_mean].value | none |
| qemu-rs | detumble_ais_mag | 0 / 21 | 0 | 4 / 35 | 0.003442 | [oils_worst_case_margin].value | none |
| qemu-rs | detumble_img | 0 / 7 | 0 | 1 / 21 | 0.01237 | [oils_worst_case_margin].value | none |
| qemu-rs | detumble_rcs | 0 / 14 | 0 | 2 / 28 | 0.01286 | [oils_worst_case_margin].value | none |
| qemu-rs | fault_coil_ais | 0 / 21 | 0 | 4 / 35 | 0.009851 | [power_mean].value | none |
| qemu-rs | fault_gimbal_cmg | 0 / 56 | 0 | 7 / 70 | 0.02204 | [oils_worst_case_margin].value | none |
| qemu-rs | fault_gps_img | 0 / 21 | 0 | 4 / 35 | 0.01993 | [oils_worst_case_margin].value | none |
| qemu-rs | fault_gyro_ais | 0 / 28 | 0 | 5 / 42 | 0.00474 | [oils_worst_case_margin].value | none |
| qemu-rs | fault_st_img | 0 / 56 | 0 | 8 / 70 | 0.01987 | [oils_worst_case_margin].value | none |
| qemu-rs | fault_valve_rcs | 0 / 21 | 0 | 4 / 35 | 0.02054 | [oils_worst_case_margin].value | none |
| qemu-rs | fault_wheel_img | 0 / 56 | 0 | 8 / 70 | 0.02431 | [ake_los_p9973].value | none |
| qemu-rs | fine_hold_cmg | 0 / 56 | 0 | 7 / 70 | 0.02217 | [oils_worst_case_margin].value | none |
| qemu-rs | fine_hold_fmr | 0 / 56 | 0 | 8 / 70 | 0.0199 | [oils_worst_case_margin].value | none |
| qemu-rs | fine_hold_fmr_rcs | 0 / 56 | 0 | 8 / 70 | 0.02047 | [oils_worst_case_margin].value | none |
| qemu-rs | fine_hold_img | 0 / 91 | 0 | 13 / 105 | 0.01992 | [oils_worst_case_margin].value | none |
| qemu-rs | fine_hold_img_lqr | 0 / 56 | 0 | 8 / 70 | 0.01997 | [oils_worst_case_margin].value | none |
| qemu-rs | fine_hold_img_smc | 0 / 56 | 0 | 8 / 70 | 0.01991 | [oils_worst_case_margin].value | none |
| qemu-rs | fine_hold_rw_rcs | 0 / 56 | 0 | 8 / 70 | 0.02057 | [oils_worst_case_margin].value | none |
| qemu-rs | fine_hold_vscmg | 0 / 56 | 0 | 8 / 70 | 0.02249 | [oils_worst_case_margin].value | none |
| qemu-rs | mission_ais | 0 / 42 | 0 | 7 / 56 | 0.08361 | [ake_los_last_orbit].value | none |
| qemu-rs | mission_cmg | 0 / 14 | 0 | 3 / 28 | 0.02212 | [oils_worst_case_margin].value | none |
| qemu-rs | mission_fmr | 0 / 14 | 0 | 2 / 28 | 0.01991 | [oils_worst_case_margin].value | none |
| qemu-rs | mission_fmr_rcs | 0 / 14 | 0 | 2 / 28 | 0.0205 | [oils_worst_case_margin].value | none |
| qemu-rs | mission_img | 0 / 28 | 0 | 4 / 42 | 0.01991 | [oils_worst_case_margin].value | none |
| qemu-rs | mission_rw_rcs | 0 / 14 | 0 | 2 / 28 | 0.1642 | [ape_los_last_half_orbit_p9973].value | none |
| qemu-rs | mission_vscmg | 0 / 14 | 0 | 2 / 28 | 0.02237 | [oils_worst_case_margin].value | none |
| qemu-rs | nadir_hold_ais | 0 / 42 | 0 | 7 / 56 | 0.004318 | [oils_worst_case_margin].value | none |
| qemu-rs | nadir_hold_ais_css | 0 / 28 | 0 | 5 / 42 | 0.005781 | [ake_los_last_orbit].value | none |
| qemu-rs | safe_mode_ais | 0 / 21 | 0 | 3 / 35 | 0.03464 | [ape_los_last_orbit].value | none |
| qemu-rs | slew_cmg | 0 / 21 | 0 | 2 / 35 | 0.02323 | [oils_worst_case_margin].value | none |
| qemu-rs | slew_fmr | 0 / 21 | 0 | 3 / 35 | 0.02098 | [oils_worst_case_margin].value | none |
| qemu-rs | slew_fmr_rcs | 0 / 21 | 0 | 3 / 35 | 0.02162 | [oils_worst_case_margin].value | none |
| qemu-rs | slew_img | 0 / 21 | 0 | 3 / 35 | 0.02103 | [oils_worst_case_margin].value | none |
| qemu-rs | slew_img_lqr | 0 / 21 | 0 | 3 / 35 | 0.02102 | [oils_worst_case_margin].value | none |
| qemu-rs | slew_img_smc | 0 / 21 | 0 | 3 / 35 | 0.02106 | [oils_worst_case_margin].value | none |
| qemu-rs | slew_rw_rcs | 0 / 21 | 0 | 3 / 35 | 0.02158 | [oils_worst_case_margin].value | none |
| qemu-rs | slew_vscmg | 0 / 21 | 0 | 3 / 35 | 0.02355 | [oils_worst_case_margin].value | none |
| qemu-rs | sun_acq_rotor_img | 0 / 21 | 0 | 3 / 35 | 0.0207 | [oils_worst_case_margin].value | none |
| qemu-rs | sun_fine_img | 0 / 21 | 0 | 4 / 35 | 0.02011 | [oils_worst_case_margin].value | none |
| qemu-rs | sun_mtq_ais | 0 / 21 | 0 | 4 / 35 | 0.004309 | [oils_worst_case_margin].value | none |
| qemu-rs | sun_spin_ais | 0 / 35 | 0 | 7 / 49 | 1 | [sun_spin_entry].pass | oils sun_spin_entry (pass -> FAIL) |
| qemu-rs | target_img | 0 / 21 | 0 | 4 / 35 | 0.0208 | [oils_worst_case_margin].value | none |

Set c, over its 48 scenarios: instructions mean change +4.40 % to +20.20 %, max change +4.30 % to +9.29 %; overruns 0; minimum worst-case margin 35.76 ms.

Set qemu-rs, over its 48 scenarios: instructions mean change +5.24 % to +16.51 %, max change +3.55 % to +26.29 %; overruns 0; minimum worst-case margin 33.21 ms.
