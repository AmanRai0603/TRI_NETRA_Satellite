# Soft OILS: SILS and the flight software on the virtual OBC, side by side

Owner: Agastya. `tools/engine.py oils` flies every scenario twice on the Rust engine (POP in the loop):
**SILS** with the C flight software in-process, and **soft OILS** with the flight software built for the
OBC (arm-none-eabi-gcc, Cortex-M4F hard-float) running as firmware in QEMU mps2-an386 (`--fsw qemu`)
behind adcs-link/1. In soft OILS every command reaches the actuators only after the OBC has read its
sensors on the buses, executed the step and sent its CAN frames; the previous command holds until then
(docs/SOFT_OILS.md). The step's execution is its **exact** instruction count (QEMU plugin
fsw/targets/qemu-mps2/insn_count.c) x CPI 1.25 / 168 MHz;
buses: I2C 400 kHz, SPI 1 MHz, CAN 1 Mbit/s.
Runs are deterministic: the same scenario gives the same instruction counts and trajectory every time.

**Verdict agreement SILS vs soft OILS: 105 of 107 judged metrics** over 40 scenarios.

## OBC timing budget

| scenario | period [ms] | instructions mean / max | exec max [ms] | latency mean / max [ms] | CPU load mean / max | overruns | deadline margin min [ms] |
|---|---:|---:|---:|---:|---:|---:|---:|
| agile_slew_cmg | 100 | 3.5e+05 / 8.935e+05 | 6.648 | 4.198 / 8.242 | 2.6 % / 6.6 % | 0 | 91.76 |
| agile_slew_fmr_rcs | 100 | 3.009e+05 / 8.453e+05 | 6.289 | 3.313 / 7.363 | 2.2 % / 6.3 % | 0 | 92.64 |
| agile_slew_img | 100 | 2.945e+05 / 8.438e+05 | 6.278 | 3.135 / 7.222 | 2.2 % / 6.3 % | 0 | 92.78 |
| agile_slew_rw_rcs | 100 | 3.044e+05 / 8.506e+05 | 6.329 | 3.339 / 7.403 | 2.3 % / 6.3 % | 0 | 92.6 |
| agile_slew_vscmg | 100 | 3.682e+05 / 9.166e+05 | 6.82 | 4.333 / 8.414 | 2.7 % / 6.8 % | 0 | 91.59 |
| detumble_ais | 200 | 1.191e+05 / 4.637e+05 | 3.45 | 1.44 / 4.004 | 0.4 % / 1.7 % | 0 | 196 |
| detumble_ais_bangbang | 200 | 1.176e+05 / 4.637e+05 | 3.45 | 1.429 / 4.004 | 0.4 % / 1.7 % | 0 | 196 |
| detumble_ais_mag | 200 | 1.187e+05 / 4.637e+05 | 3.45 | 1.437 / 4.004 | 0.4 % / 1.7 % | 0 | 196 |
| detumble_img | 100 | 8.121e+04 / 4.891e+05 | 3.639 | 1.548 / 4.583 | 0.6 % / 3.6 % | 0 | 95.42 |
| fault_coil_ais | 200 | 1.191e+05 / 4.637e+05 | 3.45 | 1.44 / 4.004 | 0.4 % / 1.7 % | 0 | 196 |
| fault_gimbal_cmg | 100 | 3.621e+05 / 8.976e+05 | 6.679 | 4.288 / 8.273 | 2.7 % / 6.7 % | 0 | 91.73 |
| fault_gyro_ais | 200 | 1.935e+05 / 6.373e+05 | 4.742 | 1.994 / 5.296 | 0.7 % / 2.4 % | 0 | 194.7 |
| fault_st_img | 100 | 2.542e+05 / 8.417e+05 | 6.263 | 2.836 / 7.207 | 1.9 % / 6.3 % | 0 | 92.79 |
| fault_wheel_img | 100 | 2.896e+05 / 8.441e+05 | 6.281 | 3.099 / 7.225 | 2.2 % / 6.3 % | 0 | 92.78 |
| fine_hold_cmg | 100 | 3.621e+05 / 8.975e+05 | 6.678 | 4.288 / 8.272 | 2.7 % / 6.7 % | 0 | 91.73 |
| fine_hold_fmr | 100 | 3.102e+05 / 8.455e+05 | 6.291 | 3.252 / 7.235 | 2.3 % / 6.3 % | 0 | 92.77 |
| fine_hold_fmr_rcs | 100 | 3.129e+05 / 8.484e+05 | 6.313 | 3.402 / 7.387 | 2.3 % / 6.3 % | 0 | 92.61 |
| fine_hold_img | 100 | 3.1e+05 / 8.452e+05 | 6.288 | 3.251 / 7.232 | 2.3 % / 6.3 % | 0 | 92.77 |
| fine_hold_img_lqr | 100 | 3.102e+05 / 8.458e+05 | 6.293 | 3.252 / 7.237 | 2.3 % / 6.3 % | 0 | 92.76 |
| fine_hold_img_smc | 100 | 3.13e+05 / 8.483e+05 | 6.312 | 3.273 / 7.256 | 2.3 % / 6.3 % | 0 | 92.74 |
| fine_hold_rw_rcs | 100 | 3.151e+05 / 8.523e+05 | 6.342 | 3.419 / 7.416 | 2.3 % / 6.3 % | 0 | 92.58 |
| fine_hold_vscmg | 100 | 3.831e+05 / 9.187e+05 | 6.836 | 4.445 / 8.43 | 2.9 % / 6.8 % | 0 | 91.57 |
| mission_ais | 200 | 1.857e+05 / 7.584e+05 | 5.643 | 1.936 / 6.197 | 0.7 % / 2.8 % | 0 | 193.8 |
| mission_cmg | 100 | 3.053e+05 / 8.969e+05 | 6.673 | 3.865 / 8.267 | 2.3 % / 6.7 % | 0 | 91.73 |
| mission_fmr | 100 | 2.613e+05 / 8.462e+05 | 6.296 | 2.889 / 7.24 | 1.9 % / 6.3 % | 0 | 92.76 |
| mission_fmr_rcs | 100 | 2.577e+05 / 8.494e+05 | 10.41 | 2.843 / 11.48 | 1.8 % / 10.4 % | 0 | 88.52 |
| mission_img | 100 | 2.594e+05 / 8.45e+05 | 48.46 | 2.726 / 49.4 | 1.8 % / 48.5 % | 0 | 50.6 |
| mission_rw_rcs | 100 | 2.645e+05 / 8.546e+05 | 40.84 | 2.895 / 41.91 | 1.8 % / 40.8 % | 0 | 58.09 |
| mission_vscmg | 100 | 3.226e+05 / 9.18e+05 | 6.83 | 3.994 / 8.424 | 2.4 % / 6.8 % | 0 | 91.58 |
| nadir_hold_ais | 200 | 1.935e+05 / 6.373e+05 | 4.742 | 1.994 / 5.296 | 0.7 % / 2.4 % | 0 | 194.7 |
| nadir_hold_ais_css | 200 | 1.935e+05 / 6.377e+05 | 4.745 | 1.994 / 5.299 | 0.7 % / 2.4 % | 0 | 194.7 |
| slew_cmg | 100 | 3.686e+05 / 8.937e+05 | 6.65 | 4.337 / 8.244 | 2.7 % / 6.6 % | 0 | 91.76 |
| slew_fmr | 100 | 3.168e+05 / 8.414e+05 | 6.26 | 3.301 / 7.204 | 2.4 % / 6.3 % | 0 | 92.8 |
| slew_fmr_rcs | 100 | 3.194e+05 / 8.444e+05 | 6.282 | 3.451 / 7.357 | 2.4 % / 6.3 % | 0 | 92.64 |
| slew_img | 100 | 3.165e+05 / 8.415e+05 | 6.261 | 3.299 / 7.205 | 2.4 % / 6.3 % | 0 | 92.79 |
| slew_img_lqr | 100 | 3.166e+05 / 8.423e+05 | 6.267 | 3.3 / 7.211 | 2.4 % / 6.3 % | 0 | 92.79 |
| slew_img_smc | 100 | 3.194e+05 / 8.449e+05 | 6.287 | 3.32 / 7.231 | 2.4 % / 6.3 % | 0 | 92.77 |
| slew_rw_rcs | 100 | 3.211e+05 / 8.474e+05 | 6.305 | 3.463 / 7.379 | 2.4 % / 6.3 % | 0 | 92.62 |
| slew_vscmg | 100 | 3.895e+05 / 9.153e+05 | 6.81 | 4.492 / 8.404 | 2.9 % / 6.8 % | 0 | 91.6 |
| sun_spin_ais | 200 | 1.205e+05 / 4.651e+05 | 3.46 | 1.451 / 4.014 | 0.4 % / 1.7 % | 0 | 196 |

## Metrics: SILS vs soft OILS

| scenario | metric | req | SILS | soft OILS | SILS | soft OILS |
|---|---|---:|---:|---:|---|---|
| agile_slew_cmg | ape_los_on_target_p9973 (deg) | 0.01 | 0.004544 | 0.00449 | pass | pass |
| agile_slew_fmr_rcs | ape_los_on_target_p9973 (deg) | 0.01 | 0.006614 | 0.00659 | pass | pass |
| agile_slew_img | ape_los_on_target_p9973 (deg) | 0.01 | 0.008686 | 0.008674 | pass | pass |
| agile_slew_rw_rcs | ape_los_on_target_p9973 (deg) | 0.01 | 0.008702 | 0.008728 | pass | pass |
| agile_slew_vscmg | ape_los_on_target_p9973 (deg) | 0.01 | 0.01008 | 0.009977 | FAIL | pass ⚠ |
| detumble_ais | detumble_time (min) | 284 | 37.79 | 38.07 | pass | pass |
| detumble_ais | power_mean (W) | 0.5 | 0.0217 | 0.02213 | pass | pass |
| detumble_ais | power_peak (W) | 1.5 | 0.5502 | 0.5502 | pass | pass |
| detumble_ais_bangbang | detumble_time (min) | 284 | 54.02 | 54.81 | pass | pass |
| detumble_ais_bangbang | power_mean (W) | 0.5 | 0.4366 | 0.4634 | pass | pass |
| detumble_ais_bangbang | power_peak (W) | 1.5 | 0.717 | 0.717 | pass | pass |
| detumble_ais_mag | detumble_time (min) | 284 | 41.16 | 41.59 | pass | pass |
| detumble_ais_mag | power_mean (W) | 0.5 | 0.07839 | 0.08269 | pass | pass |
| detumble_ais_mag | power_peak (W) | 1.5 | 0.6377 | 0.6556 | pass | pass |
| detumble_img | detumble_time (min) | 284 | 47.54 | 47.49 | pass | pass |
| fault_coil_ais | detumble_time (min) | 284 | 82.17 | 82.42 | pass | pass |
| fault_coil_ais | power_mean (W) | 0.5 | 0.01821 | 0.01808 | pass | pass |
| fault_coil_ais | power_peak (W) | 1.5 | 0.5502 | 0.5502 | pass | pass |
| fault_gimbal_cmg | ape_los_p9973 (deg) | 0.01 | 0.005509 | 0.005493 | pass | pass |
| fault_gimbal_cmg | ake_los_p9973 (deg) | 0.005 | 0.002624 | 0.00263 | pass | pass |
| fault_gimbal_cmg | rate_stability_p9973 (deg/s) | 0.001 | 0.002963 | 0.002979 | FAIL | FAIL |
| fault_gimbal_cmg | power_mean (W) | 2 | 1.61 | 1.61 | pass | pass |
| fault_gyro_ais | ape_los_last_orbit (deg) | 10 | 52.78 | 52.93 | FAIL | FAIL |
| fault_gyro_ais | ake_los_last_orbit (deg) | 5 | 41.92 | 42.1 | FAIL | FAIL |
| fault_gyro_ais | power_mean (W) | 0.5 | 0.01056 | 0.01054 | pass | pass |
| fault_st_img | ape_los_p9973 (deg) | 0.01 | 0.1022 | 0.1009 | FAIL | FAIL |
| fault_st_img | ake_los_p9973 (deg) | 0.005 | 0.1093 | 0.1092 | FAIL | FAIL |
| fault_st_img | rate_stability_p9973 (deg/s) | 0.001 | 0.003771 | 0.003764 | FAIL | FAIL |
| fault_st_img | power_mean (W) | 2 | 1.557 | 1.557 | pass | pass |
| fault_wheel_img | ape_los_p9973 (deg) | 0.01 | 28.62 | 28.48 | FAIL | FAIL |
| fault_wheel_img | ake_los_p9973 (deg) | 0.005 | 0.002858 | 0.002754 | pass | pass |
| fault_wheel_img | rate_stability_p9973 (deg/s) | 0.001 | 0.6244 | 0.6298 | FAIL | FAIL |
| fault_wheel_img | power_mean (W) | 2 | 1.213 | 1.213 | pass | pass |
| fine_hold_cmg | ape_los_p9973 (deg) | 0.01 | 0.005021 | 0.005038 | pass | pass |
| fine_hold_cmg | ake_los_p9973 (deg) | 0.005 | 0.002626 | 0.002626 | pass | pass |
| fine_hold_cmg | rate_stability_p9973 (deg/s) | 0.001 | 0.002437 | 0.002462 | FAIL | FAIL |
| fine_hold_cmg | power_mean (W) | 2 | 1.61 | 1.61 | pass | pass |
| fine_hold_fmr | ape_los_p9973 (deg) | 0.01 | 0.005689 | 0.005671 | pass | pass |
| fine_hold_fmr | ake_los_p9973 (deg) | 0.005 | 0.002631 | 0.002621 | pass | pass |
| fine_hold_fmr | rate_stability_p9973 (deg/s) | 0.001 | 0.004729 | 0.004792 | FAIL | FAIL |
| fine_hold_fmr | power_mean (W) | 2 | 0.08406 | 0.08489 | pass | pass |
| fine_hold_fmr_rcs | ape_los_p9973 (deg) | 0.01 | 0.005689 | 0.005676 | pass | pass |
| fine_hold_fmr_rcs | ake_los_p9973 (deg) | 0.005 | 0.002631 | 0.002628 | pass | pass |
| fine_hold_fmr_rcs | rate_stability_p9973 (deg/s) | 0.001 | 0.004729 | 0.004796 | FAIL | FAIL |
| fine_hold_fmr_rcs | power_mean (W) | 2 | 0.08406 | 0.08487 | pass | pass |
| fine_hold_img | ape_los_p9973 (deg) | 0.01 | 0.01049 | 0.01146 | FAIL | FAIL |
| fine_hold_img | ake_los_p9973 (deg) | 0.005 | 0.002608 | 0.002601 | pass | pass |
| fine_hold_img | rate_stability_p9973 (deg/s) | 0.001 | 0.003552 | 0.003548 | FAIL | FAIL |
| fine_hold_img | power_mean (W) | 2 | 1.557 | 1.557 | pass | pass |
| fine_hold_img_lqr | ape_los_p9973 (deg) | 0.01 | 0.004156 | 0.004144 | pass | pass |
| fine_hold_img_lqr | ake_los_p9973 (deg) | 0.005 | 0.002652 | 0.002647 | pass | pass |
| fine_hold_img_lqr | rate_stability_p9973 (deg/s) | 0.001 | 0.006704 | 0.00681 | FAIL | FAIL |
| fine_hold_img_lqr | power_mean (W) | 2 | 1.558 | 1.558 | pass | pass |
| fine_hold_img_smc | ape_los_p9973 (deg) | 0.01 | 0.01945 | 0.01951 | FAIL | FAIL |
| fine_hold_img_smc | ake_los_p9973 (deg) | 0.005 | 0.002649 | 0.002652 | pass | pass |
| fine_hold_img_smc | rate_stability_p9973 (deg/s) | 0.001 | 0.003482 | 0.003499 | FAIL | FAIL |
| fine_hold_img_smc | power_mean (W) | 2 | 1.557 | 1.557 | pass | pass |
| fine_hold_rw_rcs | ape_los_p9973 (deg) | 0.01 | 0.03388 | 0.03351 | FAIL | FAIL |
| fine_hold_rw_rcs | ake_los_p9973 (deg) | 0.005 | 0.002602 | 0.002594 | pass | pass |
| fine_hold_rw_rcs | rate_stability_p9973 (deg/s) | 0.001 | 0.005641 | 0.005641 | FAIL | FAIL |
| fine_hold_rw_rcs | power_mean (W) | 2 | 1.53 | 1.53 | pass | pass |
| fine_hold_vscmg | ape_los_p9973 (deg) | 0.01 | 0.00828 | 0.00828 | pass | pass |
| fine_hold_vscmg | ake_los_p9973 (deg) | 0.005 | 0.002636 | 0.002638 | pass | pass |
| fine_hold_vscmg | rate_stability_p9973 (deg/s) | 0.001 | 0.004762 | 0.004786 | FAIL | FAIL |
| fine_hold_vscmg | power_mean (W) | 2 | 2.014 | 2.014 | FAIL | FAIL |
| mission_ais | detumble_time (min) | 284 | 37.79 | 38.07 | pass | pass |
| mission_ais | ape_los_last_orbit (deg) | 10 | 8.935 | 7.807 | pass | pass |
| mission_ais | ake_los_last_orbit (deg) | 5 | 1.188 | 3.818 | pass | pass |
| mission_cmg | detumble_time (min) | 284 | 57.01 | 56.76 | pass | pass |
| mission_cmg | ape_los_last_half_orbit_p9973 (deg) | 0.01 | 0.005241 | 0.00528 | pass | pass |
| mission_fmr | detumble_time (min) | 284 | 61.33 | 61.51 | pass | pass |
| mission_fmr | ape_los_last_half_orbit_p9973 (deg) | 0.01 | 0.005954 | 0.005997 | pass | pass |
| mission_fmr_rcs | detumble_time (min) | 284 | 61.33 | 61.51 | pass | pass |
| mission_fmr_rcs | ape_los_last_half_orbit_p9973 (deg) | 0.01 | 0.005954 | 0.005964 | pass | pass |
| mission_img | detumble_time (min) | 284 | 51.01 | 51.39 | pass | pass |
| mission_img | ape_los_last_half_orbit_p9973 (deg) | 0.01 | 0.01066 | 0.01047 | FAIL | FAIL |
| mission_rw_rcs | detumble_time (min) | 284 | 51.01 | 51.39 | pass | pass |
| mission_rw_rcs | ape_los_last_half_orbit_p9973 (deg) | 0.01 | 0.007564 | 0.00757 | pass | pass |
| mission_vscmg | detumble_time (min) | 284 | 57.02 | 56.86 | pass | pass |
| mission_vscmg | ape_los_last_half_orbit_p9973 (deg) | 0.01 | 0.009595 | 0.009586 | pass | pass |
| nadir_hold_ais | ape_los_last_orbit (deg) | 10 | 12.64 | 12.53 | FAIL | FAIL |
| nadir_hold_ais | ake_los_last_orbit (deg) | 5 | 5.491 | 5.514 | FAIL | FAIL |
| nadir_hold_ais | power_mean (W) | 0.5 | 0.01054 | 0.01051 | pass | pass |
| nadir_hold_ais_css | ape_los_last_orbit (deg) | 10 | 39.44 | 39.12 | FAIL | FAIL |
| nadir_hold_ais_css | ake_los_last_orbit (deg) | 5 | 39.22 | 39.12 | FAIL | FAIL |
| nadir_hold_ais_css | power_mean (W) | 0.5 | 0.01064 | 0.0106 | pass | pass |
| slew_cmg | settle_time_after_slew (s) | 20 | 12.4 | 12.5 | pass | pass |
| slew_cmg | ape_los_on_target_p9973 (deg) | 0.01 | 0.004233 | 0.004257 | pass | pass |
| slew_fmr | settle_time_after_slew (s) | 20 | 12.9 | 13 | pass | pass |
| slew_fmr | ape_los_on_target_p9973 (deg) | 0.01 | 0.006529 | 0.00665 | pass | pass |
| slew_fmr_rcs | settle_time_after_slew (s) | 20 | 12.9 | 12.9 | pass | pass |
| slew_fmr_rcs | ape_los_on_target_p9973 (deg) | 0.01 | 0.006529 | 0.006685 | pass | pass |
| slew_img | settle_time_after_slew (s) | 20 | 12.1 | 12.1 | pass | pass |
| slew_img | ape_los_on_target_p9973 (deg) | 0.01 | 0.007925 | 0.007944 | pass | pass |
| slew_img_lqr | settle_time_after_slew (s) | 20 | 0 | 0 | pass | pass |
| slew_img_lqr | ape_los_on_target_p9973 (deg) | 0.01 | 0.004439 | 0.004462 | pass | pass |
| slew_img_smc | settle_time_after_slew (s) | 20 | — | — | FAIL | FAIL |
| slew_img_smc | ape_los_on_target_p9973 (deg) | 0.01 | 0.0178 | 0.01791 | FAIL | FAIL |
| slew_rw_rcs | settle_time_after_slew (s) | 20 | 12.1 | 12.1 | pass | pass |
| slew_rw_rcs | ape_los_on_target_p9973 (deg) | 0.01 | 0.007922 | 0.007941 | pass | pass |
| slew_vscmg | settle_time_after_slew (s) | 20 | 13.2 | 13.2 | pass | pass |
| slew_vscmg | ape_los_on_target_p9973 (deg) | 0.01 | 0.01083 | 0.01072 | FAIL | FAIL |
| sun_spin_ais | sun_spin_entry (min) | 95 | 142.4 | 59.66 | FAIL | pass ⚠ |
| sun_spin_ais | sun_angle_last_orbit (deg) | 20 | 93.84 | 125.1 | FAIL | FAIL |
| sun_spin_ais | spin_rate_error_last_orbit (deg/s) | 0.5 | 1.116 | 1.589 | FAIL | FAIL |
| sun_spin_ais | sun_spin_share_last_orbit (%) | 90 | 13.75 | 0 | FAIL | FAIL |
| sun_spin_ais | power_mean (W) | 0.5 | 0.45 | 0.4744 | pass | pass |
