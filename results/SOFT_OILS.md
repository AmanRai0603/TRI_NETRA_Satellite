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

**Verdict agreement SILS vs soft OILS: 104 of 107 judged metrics** over 40 scenarios.

## OBC timing budget

| scenario | period [ms] | instructions mean / max | exec max [ms] | latency mean / max [ms] | CPU load mean / max | overruns | deadline margin min [ms] |
|---|---:|---:|---:|---:|---:|---:|---:|
| agile_slew_cmg | 100 | 3.5e+05 / 8.951e+05 | 6.66 | 4.198 / 8.254 | 2.6 % / 6.7 % | 0 | 91.75 |
| agile_slew_fmr_rcs | 100 | 3.009e+05 / 8.447e+05 | 6.285 | 3.313 / 7.359 | 2.2 % / 6.3 % | 0 | 92.64 |
| agile_slew_img | 100 | 2.945e+05 / 8.428e+05 | 6.271 | 3.135 / 7.215 | 2.2 % / 6.3 % | 0 | 92.78 |
| agile_slew_rw_rcs | 100 | 3.044e+05 / 8.516e+05 | 6.337 | 3.339 / 7.411 | 2.3 % / 6.3 % | 0 | 92.59 |
| agile_slew_vscmg | 100 | 3.682e+05 / 9.158e+05 | 6.814 | 4.334 / 8.408 | 2.7 % / 6.8 % | 0 | 91.59 |
| detumble_ais | 200 | 1.191e+05 / 4.637e+05 | 3.45 | 1.44 / 4.004 | 0.4 % / 1.7 % | 0 | 196 |
| detumble_ais_bangbang | 200 | 1.166e+05 / 4.637e+05 | 3.45 | 1.422 / 4.004 | 0.4 % / 1.7 % | 0 | 196 |
| detumble_ais_mag | 200 | 1.186e+05 / 4.637e+05 | 3.45 | 1.437 / 4.004 | 0.4 % / 1.7 % | 0 | 196 |
| detumble_img | 100 | 8.121e+04 / 4.891e+05 | 3.639 | 1.548 / 4.583 | 0.6 % / 3.6 % | 0 | 95.42 |
| fault_coil_ais | 200 | 1.191e+05 / 4.637e+05 | 3.45 | 1.44 / 4.004 | 0.4 % / 1.7 % | 0 | 196 |
| fault_gimbal_cmg | 100 | 3.62e+05 / 8.974e+05 | 6.677 | 4.287 / 8.271 | 2.7 % / 6.7 % | 0 | 91.73 |
| fault_gyro_ais | 200 | 1.935e+05 / 6.373e+05 | 4.742 | 1.994 / 5.296 | 0.7 % / 2.4 % | 0 | 194.7 |
| fault_st_img | 100 | 2.542e+05 / 8.422e+05 | 6.266 | 2.836 / 7.21 | 1.9 % / 6.3 % | 0 | 92.79 |
| fault_wheel_img | 100 | 2.897e+05 / 8.442e+05 | 6.281 | 3.099 / 7.225 | 2.2 % / 6.3 % | 0 | 92.77 |
| fine_hold_cmg | 100 | 3.621e+05 / 8.975e+05 | 6.678 | 4.288 / 8.272 | 2.7 % / 6.7 % | 0 | 91.73 |
| fine_hold_fmr | 100 | 3.103e+05 / 8.454e+05 | 6.29 | 3.252 / 7.234 | 2.3 % / 6.3 % | 0 | 92.77 |
| fine_hold_fmr_rcs | 100 | 3.13e+05 / 8.482e+05 | 6.311 | 3.403 / 7.385 | 2.3 % / 6.3 % | 0 | 92.62 |
| fine_hold_img | 100 | 3.1e+05 / 8.455e+05 | 6.291 | 3.251 / 7.235 | 2.3 % / 6.3 % | 0 | 92.77 |
| fine_hold_img_lqr | 100 | 3.102e+05 / 8.454e+05 | 6.29 | 3.252 / 7.234 | 2.3 % / 6.3 % | 0 | 92.77 |
| fine_hold_img_smc | 100 | 3.131e+05 / 8.484e+05 | 6.313 | 3.273 / 7.257 | 2.3 % / 6.3 % | 0 | 92.74 |
| fine_hold_rw_rcs | 100 | 3.151e+05 / 8.527e+05 | 6.345 | 3.419 / 7.419 | 2.3 % / 6.3 % | 0 | 92.58 |
| fine_hold_vscmg | 100 | 3.831e+05 / 9.188e+05 | 6.836 | 4.445 / 8.43 | 2.9 % / 6.8 % | 0 | 91.57 |
| mission_ais | 200 | 1.857e+05 / 7.584e+05 | 5.643 | 1.936 / 6.197 | 0.7 % / 2.8 % | 0 | 193.8 |
| mission_cmg | 100 | 3.053e+05 / 8.971e+05 | 6.675 | 3.865 / 8.269 | 2.3 % / 6.7 % | 0 | 91.73 |
| mission_fmr | 100 | 2.614e+05 / 8.461e+05 | 6.296 | 2.889 / 7.24 | 1.9 % / 6.3 % | 0 | 92.76 |
| mission_fmr_rcs | 100 | 2.638e+05 / 8.496e+05 | 6.322 | 3.037 / 7.396 | 2.0 % / 6.3 % | 0 | 92.6 |
| mission_img | 100 | 2.681e+05 / 8.452e+05 | 6.289 | 2.939 / 7.233 | 2.0 % / 6.3 % | 0 | 92.77 |
| mission_rw_rcs | 100 | 2.744e+05 / 8.548e+05 | 6.36 | 3.116 / 7.434 | 2.0 % / 6.4 % | 0 | 92.57 |
| mission_vscmg | 100 | 3.226e+05 / 9.181e+05 | 6.831 | 3.994 / 8.425 | 2.4 % / 6.8 % | 0 | 91.57 |
| nadir_hold_ais | 200 | 1.935e+05 / 6.373e+05 | 4.742 | 1.994 / 5.296 | 0.7 % / 2.4 % | 0 | 194.7 |
| nadir_hold_ais_css | 200 | 1.935e+05 / 6.377e+05 | 4.745 | 1.994 / 5.299 | 0.7 % / 2.4 % | 0 | 194.7 |
| slew_cmg | 100 | 3.686e+05 / 8.938e+05 | 6.65 | 4.336 / 8.244 | 2.7 % / 6.7 % | 0 | 91.76 |
| slew_fmr | 100 | 3.167e+05 / 8.42e+05 | 6.265 | 3.301 / 7.209 | 2.4 % / 6.3 % | 0 | 92.79 |
| slew_fmr_rcs | 100 | 3.195e+05 / 8.449e+05 | 6.286 | 3.451 / 7.36 | 2.4 % / 6.3 % | 0 | 92.64 |
| slew_img | 100 | 3.165e+05 / 8.419e+05 | 6.264 | 3.299 / 7.208 | 2.4 % / 6.3 % | 0 | 92.79 |
| slew_img_lqr | 100 | 3.167e+05 / 8.42e+05 | 6.265 | 3.3 / 7.209 | 2.4 % / 6.3 % | 0 | 92.79 |
| slew_img_smc | 100 | 3.194e+05 / 8.447e+05 | 6.285 | 3.32 / 7.229 | 2.4 % / 6.3 % | 0 | 92.77 |
| slew_rw_rcs | 100 | 3.211e+05 / 8.472e+05 | 6.304 | 3.463 / 7.378 | 2.4 % / 6.3 % | 0 | 92.62 |
| slew_vscmg | 100 | 3.895e+05 / 9.149e+05 | 6.807 | 4.492 / 8.401 | 2.9 % / 6.8 % | 0 | 91.6 |
| sun_spin_ais | 200 | 1.204e+05 / 4.651e+05 | 3.46 | 1.45 / 4.014 | 0.4 % / 1.7 % | 0 | 196 |

## Metrics: SILS vs soft OILS

| scenario | metric | req | SILS | soft OILS | SILS | soft OILS |
|---|---|---:|---:|---:|---|---|
| agile_slew_cmg | ape_los_on_target_p9973 (deg) | 0.01 | 0.004544 | 0.004497 | pass | pass |
| agile_slew_fmr_rcs | ape_los_on_target_p9973 (deg) | 0.01 | 0.006614 | 0.00662 | pass | pass |
| agile_slew_img | ape_los_on_target_p9973 (deg) | 0.01 | 0.008686 | 0.008668 | pass | pass |
| agile_slew_rw_rcs | ape_los_on_target_p9973 (deg) | 0.01 | 0.008702 | 0.008715 | pass | pass |
| agile_slew_vscmg | ape_los_on_target_p9973 (deg) | 0.01 | 0.01008 | 0.009971 | FAIL | pass ⚠ |
| detumble_ais | detumble_time (min) | 284 | 37.79 | 38.07 | pass | pass |
| detumble_ais | power_mean (W) | 0.5 | 0.0217 | 0.02213 | pass | pass |
| detumble_ais | power_peak (W) | 1.5 | 0.5502 | 0.5502 | pass | pass |
| detumble_ais_bangbang | detumble_time (min) | 284 | 32.21 | — | pass | FAIL ⚠ |
| detumble_ais_bangbang | power_mean (W) | 0.5 | 0.7114 | 0.7119 | FAIL | FAIL |
| detumble_ais_bangbang | power_peak (W) | 1.5 | 0.72 | 0.72 | pass | pass |
| detumble_ais_mag | detumble_time (min) | 284 | 41.16 | 41.59 | pass | pass |
| detumble_ais_mag | power_mean (W) | 0.5 | 0.07839 | 0.08271 | pass | pass |
| detumble_ais_mag | power_peak (W) | 1.5 | 0.6377 | 0.655 | pass | pass |
| detumble_img | detumble_time (min) | 284 | 47.54 | 47.49 | pass | pass |
| fault_coil_ais | detumble_time (min) | 284 | 82.17 | 82.42 | pass | pass |
| fault_coil_ais | power_mean (W) | 0.5 | 0.01821 | 0.01808 | pass | pass |
| fault_coil_ais | power_peak (W) | 1.5 | 0.5502 | 0.5502 | pass | pass |
| fault_gimbal_cmg | ape_los_p9973 (deg) | 0.01 | 0.005509 | 0.005452 | pass | pass |
| fault_gimbal_cmg | ake_los_p9973 (deg) | 0.005 | 0.002624 | 0.002626 | pass | pass |
| fault_gimbal_cmg | rate_stability_p9973 (deg/s) | 0.001 | 0.002963 | 0.002996 | FAIL | FAIL |
| fault_gimbal_cmg | power_mean (W) | 2 | 1.61 | 1.61 | pass | pass |
| fault_gyro_ais | ape_los_last_orbit (deg) | 10 | 52.78 | 52.93 | FAIL | FAIL |
| fault_gyro_ais | ake_los_last_orbit (deg) | 5 | 41.92 | 42.1 | FAIL | FAIL |
| fault_gyro_ais | power_mean (W) | 0.5 | 0.01056 | 0.01054 | pass | pass |
| fault_st_img | ape_los_p9973 (deg) | 0.01 | 0.1022 | 0.1009 | FAIL | FAIL |
| fault_st_img | ake_los_p9973 (deg) | 0.005 | 0.1093 | 0.1093 | FAIL | FAIL |
| fault_st_img | rate_stability_p9973 (deg/s) | 0.001 | 0.003771 | 0.003763 | FAIL | FAIL |
| fault_st_img | power_mean (W) | 2 | 1.557 | 1.557 | pass | pass |
| fault_wheel_img | ape_los_p9973 (deg) | 0.01 | 28.62 | 28.48 | FAIL | FAIL |
| fault_wheel_img | ake_los_p9973 (deg) | 0.005 | 0.002858 | 0.002774 | pass | pass |
| fault_wheel_img | rate_stability_p9973 (deg/s) | 0.001 | 0.6244 | 0.6298 | FAIL | FAIL |
| fault_wheel_img | power_mean (W) | 2 | 1.213 | 1.213 | pass | pass |
| fine_hold_cmg | ape_los_p9973 (deg) | 0.01 | 0.005021 | 0.005009 | pass | pass |
| fine_hold_cmg | ake_los_p9973 (deg) | 0.005 | 0.002626 | 0.002627 | pass | pass |
| fine_hold_cmg | rate_stability_p9973 (deg/s) | 0.001 | 0.002437 | 0.002473 | FAIL | FAIL |
| fine_hold_cmg | power_mean (W) | 2 | 1.61 | 1.61 | pass | pass |
| fine_hold_fmr | ape_los_p9973 (deg) | 0.01 | 0.005689 | 0.005659 | pass | pass |
| fine_hold_fmr | ake_los_p9973 (deg) | 0.005 | 0.002631 | 0.00262 | pass | pass |
| fine_hold_fmr | rate_stability_p9973 (deg/s) | 0.001 | 0.004729 | 0.004776 | FAIL | FAIL |
| fine_hold_fmr | power_mean (W) | 2 | 0.08406 | 0.08486 | pass | pass |
| fine_hold_fmr_rcs | ape_los_p9973 (deg) | 0.01 | 0.005689 | 0.005674 | pass | pass |
| fine_hold_fmr_rcs | ake_los_p9973 (deg) | 0.005 | 0.002631 | 0.00262 | pass | pass |
| fine_hold_fmr_rcs | rate_stability_p9973 (deg/s) | 0.001 | 0.004729 | 0.004784 | FAIL | FAIL |
| fine_hold_fmr_rcs | power_mean (W) | 2 | 0.08406 | 0.08487 | pass | pass |
| fine_hold_img | ape_los_p9973 (deg) | 0.01 | 0.01049 | 0.01144 | FAIL | FAIL |
| fine_hold_img | ake_los_p9973 (deg) | 0.005 | 0.002608 | 0.002601 | pass | pass |
| fine_hold_img | rate_stability_p9973 (deg/s) | 0.001 | 0.003552 | 0.003542 | FAIL | FAIL |
| fine_hold_img | power_mean (W) | 2 | 1.557 | 1.557 | pass | pass |
| fine_hold_img_lqr | ape_los_p9973 (deg) | 0.01 | 0.004156 | 0.004145 | pass | pass |
| fine_hold_img_lqr | ake_los_p9973 (deg) | 0.005 | 0.002652 | 0.002651 | pass | pass |
| fine_hold_img_lqr | rate_stability_p9973 (deg/s) | 0.001 | 0.006704 | 0.006797 | FAIL | FAIL |
| fine_hold_img_lqr | power_mean (W) | 2 | 1.558 | 1.558 | pass | pass |
| fine_hold_img_smc | ape_los_p9973 (deg) | 0.01 | 0.01945 | 0.0195 | FAIL | FAIL |
| fine_hold_img_smc | ake_los_p9973 (deg) | 0.005 | 0.002649 | 0.002647 | pass | pass |
| fine_hold_img_smc | rate_stability_p9973 (deg/s) | 0.001 | 0.003482 | 0.003506 | FAIL | FAIL |
| fine_hold_img_smc | power_mean (W) | 2 | 1.557 | 1.557 | pass | pass |
| fine_hold_rw_rcs | ape_los_p9973 (deg) | 0.01 | 0.03388 | 0.03349 | FAIL | FAIL |
| fine_hold_rw_rcs | ake_los_p9973 (deg) | 0.005 | 0.002602 | 0.0026 | pass | pass |
| fine_hold_rw_rcs | rate_stability_p9973 (deg/s) | 0.001 | 0.005641 | 0.005647 | FAIL | FAIL |
| fine_hold_rw_rcs | power_mean (W) | 2 | 1.53 | 1.53 | pass | pass |
| fine_hold_vscmg | ape_los_p9973 (deg) | 0.01 | 0.00828 | 0.008298 | pass | pass |
| fine_hold_vscmg | ake_los_p9973 (deg) | 0.005 | 0.002636 | 0.002641 | pass | pass |
| fine_hold_vscmg | rate_stability_p9973 (deg/s) | 0.001 | 0.004762 | 0.004791 | FAIL | FAIL |
| fine_hold_vscmg | power_mean (W) | 2 | 2.014 | 2.014 | FAIL | FAIL |
| mission_ais | detumble_time (min) | 284 | 37.79 | 38.07 | pass | pass |
| mission_ais | ape_los_last_orbit (deg) | 10 | 8.935 | 7.821 | pass | pass |
| mission_ais | ake_los_last_orbit (deg) | 5 | 1.188 | 3.79 | pass | pass |
| mission_cmg | detumble_time (min) | 284 | 57.01 | 56.76 | pass | pass |
| mission_cmg | ape_los_last_half_orbit_p9973 (deg) | 0.01 | 0.005241 | 0.005191 | pass | pass |
| mission_fmr | detumble_time (min) | 284 | 61.33 | 61.51 | pass | pass |
| mission_fmr | ape_los_last_half_orbit_p9973 (deg) | 0.01 | 0.005954 | 0.005914 | pass | pass |
| mission_fmr_rcs | detumble_time (min) | 284 | 61.33 | 61.51 | pass | pass |
| mission_fmr_rcs | ape_los_last_half_orbit_p9973 (deg) | 0.01 | 0.005954 | 0.005981 | pass | pass |
| mission_img | detumble_time (min) | 284 | 51.01 | 51.39 | pass | pass |
| mission_img | ape_los_last_half_orbit_p9973 (deg) | 0.01 | 0.01066 | 0.01023 | FAIL | FAIL |
| mission_rw_rcs | detumble_time (min) | 284 | 51.01 | 51.39 | pass | pass |
| mission_rw_rcs | ape_los_last_half_orbit_p9973 (deg) | 0.01 | 0.007564 | 0.007636 | pass | pass |
| mission_vscmg | detumble_time (min) | 284 | 57.02 | 56.86 | pass | pass |
| mission_vscmg | ape_los_last_half_orbit_p9973 (deg) | 0.01 | 0.009595 | 0.009606 | pass | pass |
| nadir_hold_ais | ape_los_last_orbit (deg) | 10 | 12.64 | 12.53 | FAIL | FAIL |
| nadir_hold_ais | ake_los_last_orbit (deg) | 5 | 5.491 | 5.514 | FAIL | FAIL |
| nadir_hold_ais | power_mean (W) | 0.5 | 0.01054 | 0.01051 | pass | pass |
| nadir_hold_ais_css | ape_los_last_orbit (deg) | 10 | 39.44 | 39.12 | FAIL | FAIL |
| nadir_hold_ais_css | ake_los_last_orbit (deg) | 5 | 39.22 | 39.12 | FAIL | FAIL |
| nadir_hold_ais_css | power_mean (W) | 0.5 | 0.01064 | 0.0106 | pass | pass |
| slew_cmg | settle_time_after_slew (s) | 20 | 12.4 | 12.5 | pass | pass |
| slew_cmg | ape_los_on_target_p9973 (deg) | 0.01 | 0.004233 | 0.004254 | pass | pass |
| slew_fmr | settle_time_after_slew (s) | 20 | 12.9 | 12.9 | pass | pass |
| slew_fmr | ape_los_on_target_p9973 (deg) | 0.01 | 0.006529 | 0.006635 | pass | pass |
| slew_fmr_rcs | settle_time_after_slew (s) | 20 | 12.9 | 12.9 | pass | pass |
| slew_fmr_rcs | ape_los_on_target_p9973 (deg) | 0.01 | 0.006529 | 0.006696 | pass | pass |
| slew_img | settle_time_after_slew (s) | 20 | 12.1 | 12.1 | pass | pass |
| slew_img | ape_los_on_target_p9973 (deg) | 0.01 | 0.007925 | 0.007935 | pass | pass |
| slew_img_lqr | settle_time_after_slew (s) | 20 | 0 | 0 | pass | pass |
| slew_img_lqr | ape_los_on_target_p9973 (deg) | 0.01 | 0.004439 | 0.004463 | pass | pass |
| slew_img_smc | settle_time_after_slew (s) | 20 | — | — | FAIL | FAIL |
| slew_img_smc | ape_los_on_target_p9973 (deg) | 0.01 | 0.0178 | 0.01787 | FAIL | FAIL |
| slew_rw_rcs | settle_time_after_slew (s) | 20 | 12.1 | 12.1 | pass | pass |
| slew_rw_rcs | ape_los_on_target_p9973 (deg) | 0.01 | 0.007922 | 0.007932 | pass | pass |
| slew_vscmg | settle_time_after_slew (s) | 20 | 13.2 | 13.2 | pass | pass |
| slew_vscmg | ape_los_on_target_p9973 (deg) | 0.01 | 0.01083 | 0.01075 | FAIL | FAIL |
| sun_spin_ais | sun_spin_entry (min) | 95 | 142.4 | 61.91 | FAIL | pass ⚠ |
| sun_spin_ais | sun_angle_last_orbit (deg) | 20 | 93.84 | 122 | FAIL | FAIL |
| sun_spin_ais | spin_rate_error_last_orbit (deg/s) | 0.5 | 1.116 | 1.455 | FAIL | FAIL |
| sun_spin_ais | sun_spin_share_last_orbit (%) | 90 | 13.75 | 0 | FAIL | FAIL |
| sun_spin_ais | power_mean (W) | 0.5 | 0.45 | 0.4715 | pass | pass |
