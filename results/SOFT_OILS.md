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

**Verdict agreement SILS vs soft OILS: 124 of 126 judged metrics** over 47 scenarios.

## OBC timing budget

| scenario | period [ms] | instructions mean / max | exec max [ms] | latency mean / max [ms] | CPU load mean / max | overruns | deadline margin min [ms] |
|---|---:|---:|---:|---:|---:|---:|---:|
| agile_slew_cmg | 100 | 4.209e+05 / 9.877e+05 | 7.349 | 4.726 / 8.943 | 3.1 % / 7.3 % | 0 | 91.06 |
| agile_slew_fmr_rcs | 100 | 3.717e+05 / 9.378e+05 | 6.977 | 3.84 / 8.051 | 2.8 % / 7.0 % | 0 | 91.95 |
| agile_slew_img | 100 | 3.649e+05 / 9.345e+05 | 6.953 | 3.659 / 7.897 | 2.7 % / 7.0 % | 0 | 92.1 |
| agile_slew_rw_rcs | 100 | 3.753e+05 / 9.44e+05 | 7.024 | 3.867 / 8.098 | 2.8 % / 7.0 % | 0 | 91.9 |
| agile_slew_vscmg | 100 | 4.392e+05 / 1.007e+06 | 7.491 | 4.862 / 9.085 | 3.3 % / 7.5 % | 0 | 90.92 |
| detumble_ais | 200 | 1.661e+05 / 5.334e+05 | 3.968 | 1.79 / 4.522 | 0.6 % / 2.0 % | 0 | 195.5 |
| detumble_ais_bangbang | 200 | 1.646e+05 / 5.334e+05 | 3.968 | 1.779 / 4.522 | 0.6 % / 2.0 % | 0 | 195.5 |
| detumble_ais_mag | 200 | 1.656e+05 / 5.335e+05 | 3.969 | 1.786 / 4.523 | 0.6 % / 2.0 % | 0 | 195.5 |
| detumble_img | 100 | 1.245e+05 / 5.494e+05 | 4.088 | 1.87 / 5.032 | 0.9 % / 4.1 % | 0 | 94.97 |
| detumble_rcs | 100 | 1.24e+05 / 5.6e+05 | 4.166 | 1.996 / 5.24 | 0.9 % / 4.2 % | 0 | 94.76 |
| fault_coil_ais | 200 | 1.661e+05 / 5.336e+05 | 3.97 | 1.79 / 4.524 | 0.6 % / 2.0 % | 0 | 195.5 |
| fault_gimbal_cmg | 100 | 4.129e+05 / 9.87e+05 | 7.344 | 4.666 / 8.938 | 3.1 % / 7.3 % | 0 | 91.06 |
| fault_gps_img | 100 | 3.598e+05 / 9.342e+05 | 6.951 | 3.621 / 7.895 | 2.7 % / 7.0 % | 0 | 92.11 |
| fault_gyro_ais | 200 | 2.699e+05 / 8.39e+05 | 6.243 | 2.562 / 6.797 | 1.0 % / 3.1 % | 0 | 193.2 |
| fault_st_img | 100 | 3.159e+05 / 9.293e+05 | 6.914 | 3.294 / 7.858 | 2.4 % / 6.9 % | 0 | 92.14 |
| fault_valve_rcs | 100 | 3.689e+05 / 9.454e+05 | 7.034 | 3.818 / 8.108 | 2.7 % / 7.0 % | 0 | 91.89 |
| fault_wheel_img | 100 | 3.544e+05 / 9.328e+05 | 6.94 | 3.581 / 7.884 | 2.6 % / 6.9 % | 0 | 92.12 |
| fine_hold_cmg | 100 | 4.13e+05 / 9.871e+05 | 7.344 | 4.667 / 8.938 | 3.1 % / 7.3 % | 0 | 91.06 |
| fine_hold_fmr | 100 | 3.607e+05 / 9.348e+05 | 6.955 | 3.628 / 7.899 | 2.7 % / 7.0 % | 0 | 92.1 |
| fine_hold_fmr_rcs | 100 | 3.638e+05 / 9.367e+05 | 6.969 | 3.781 / 8.043 | 2.7 % / 7.0 % | 0 | 91.96 |
| fine_hold_img | 100 | 3.605e+05 / 9.343e+05 | 6.952 | 3.626 / 7.896 | 2.7 % / 7.0 % | 0 | 92.1 |
| fine_hold_img_lqr | 100 | 3.607e+05 / 9.36e+05 | 6.964 | 3.628 / 7.908 | 2.7 % / 7.0 % | 0 | 92.09 |
| fine_hold_img_smc | 100 | 3.635e+05 / 9.387e+05 | 6.984 | 3.649 / 7.928 | 2.7 % / 7.0 % | 0 | 92.07 |
| fine_hold_rw_rcs | 100 | 3.677e+05 / 9.432e+05 | 7.018 | 3.81 / 8.092 | 2.7 % / 7.0 % | 0 | 91.91 |
| fine_hold_vscmg | 100 | 4.342e+05 / 1.008e+06 | 7.503 | 4.825 / 9.097 | 3.2 % / 7.5 % | 0 | 90.9 |
| mission_ais | 200 | 2.589e+05 / 7.172e+05 | 5.336 | 2.48 / 5.89 | 1.0 % / 2.7 % | 0 | 194.1 |
| mission_cmg | 100 | 3.545e+05 / 9.866e+05 | 7.341 | 4.232 / 8.935 | 2.6 % / 7.3 % | 0 | 91.07 |
| mission_fmr | 100 | 3.106e+05 / 9.372e+05 | 6.973 | 3.255 / 7.917 | 2.3 % / 7.0 % | 0 | 92.08 |
| mission_fmr_rcs | 100 | 3.134e+05 / 9.406e+05 | 6.999 | 3.405 / 8.073 | 2.3 % / 7.0 % | 0 | 91.93 |
| mission_img | 100 | 3.186e+05 / 9.352e+05 | 6.958 | 3.315 / 7.902 | 2.4 % / 7.0 % | 0 | 92.1 |
| mission_rw_rcs | 100 | 3.252e+05 / 9.434e+05 | 7.019 | 3.494 / 8.093 | 2.4 % / 7.0 % | 0 | 91.91 |
| mission_vscmg | 100 | 3.721e+05 / 1.008e+06 | 7.5 | 4.363 / 9.094 | 2.8 % / 7.5 % | 0 | 90.91 |
| nadir_hold_ais | 200 | 2.7e+05 / 7.18e+05 | 5.342 | 2.563 / 5.896 | 1.0 % / 2.7 % | 0 | 194.1 |
| nadir_hold_ais_css | 200 | 2.699e+05 / 7.177e+05 | 5.34 | 2.562 / 5.894 | 1.0 % / 2.7 % | 0 | 194.1 |
| safe_mode_ais | 200 | 2.6e+05 / 7.18e+05 | 5.342 | 2.489 / 5.896 | 1.0 % / 2.7 % | 0 | 194.1 |
| slew_cmg | 100 | 4.194e+05 / 9.835e+05 | 7.318 | 4.714 / 8.912 | 3.1 % / 7.3 % | 0 | 91.09 |
| slew_fmr | 100 | 3.671e+05 / 9.32e+05 | 6.934 | 3.676 / 7.878 | 2.7 % / 6.9 % | 0 | 92.12 |
| slew_fmr_rcs | 100 | 3.702e+05 / 9.34e+05 | 6.949 | 3.829 / 8.023 | 2.8 % / 6.9 % | 0 | 91.98 |
| slew_img | 100 | 3.669e+05 / 9.3e+05 | 6.92 | 3.674 / 7.864 | 2.7 % / 6.9 % | 0 | 92.14 |
| slew_img_lqr | 100 | 3.67e+05 / 9.314e+05 | 6.93 | 3.675 / 7.874 | 2.7 % / 6.9 % | 0 | 92.13 |
| slew_img_smc | 100 | 3.697e+05 / 9.335e+05 | 6.946 | 3.695 / 7.89 | 2.8 % / 6.9 % | 0 | 92.11 |
| slew_rw_rcs | 100 | 3.718e+05 / 9.384e+05 | 6.982 | 3.841 / 8.056 | 2.8 % / 7.0 % | 0 | 91.94 |
| slew_vscmg | 100 | 4.404e+05 / 1.004e+06 | 7.472 | 4.871 / 9.066 | 3.3 % / 7.5 % | 0 | 90.93 |
| sun_acq_rotor_img | 100 | 2.811e+05 / 9.028e+05 | 6.717 | 3.036 / 7.661 | 2.1 % / 6.7 % | 0 | 92.34 |
| sun_fine_img | 100 | 3.529e+05 / 9.51e+05 | 7.076 | 3.57 / 8.02 | 2.6 % / 7.1 % | 0 | 91.98 |
| sun_mtq_ais | 200 | 2.743e+05 / 7.173e+05 | 5.337 | 2.595 / 5.891 | 1.0 % / 2.7 % | 0 | 194.1 |
| sun_spin_ais | 200 | 1.675e+05 / 5.402e+05 | 4.02 | 1.801 / 4.574 | 0.6 % / 2.0 % | 0 | 195.4 |

## The deadline, judged on the worst case

Each step's worst case: execution at CPI 2.0 (not 1.25), plus 50 us of interrupts that may
preempt it, plus the bus time; the command must land within 50 % of the control period. Both are
judged metrics of every soft-OILS run (`oils_overruns`, `oils_worst_case_margin`). The instruction count includes the HAL's
copies inside the step (on a real OBC those are the bus transfers, timed separately: counted twice, so conservative);
the adcs-link framing outside the step is the simulation's, not the OBC's, and is not counted.

| scenario | deadline [ms] | worst-case margin [ms] | verdict |
|---|---:|---:|---|
| agile_slew_cmg | 50 | 36.6 | pass |
| agile_slew_fmr_rcs | 50 | 37.71 | pass |
| agile_slew_img | 50 | 37.88 | pass |
| agile_slew_rw_rcs | 50 | 37.64 | pass |
| agile_slew_vscmg | 50 | 36.37 | pass |
| detumble_ais | 100 | 93.05 | pass |
| detumble_ais_bangbang | 100 | 93.05 | pass |
| detumble_ais_mag | 100 | 93.05 | pass |
| detumble_img | 50 | 42.47 | pass |
| detumble_rcs | 50 | 42.21 | pass |
| fault_coil_ais | 100 | 93.04 | pass |
| fault_gimbal_cmg | 50 | 36.61 | pass |
| fault_gps_img | 50 | 37.88 | pass |
| fault_gyro_ais | 100 | 89.41 | pass |
| fault_st_img | 50 | 37.94 | pass |
| fault_valve_rcs | 50 | 37.62 | pass |
| fault_wheel_img | 50 | 37.9 | pass |
| fine_hold_cmg | 50 | 36.61 | pass |
| fine_hold_fmr | 50 | 37.88 | pass |
| fine_hold_fmr_rcs | 50 | 37.73 | pass |
| fine_hold_img | 50 | 37.88 | pass |
| fine_hold_img_lqr | 50 | 37.86 | pass |
| fine_hold_img_smc | 50 | 37.83 | pass |
| fine_hold_rw_rcs | 50 | 37.65 | pass |
| fine_hold_vscmg | 50 | 36.35 | pass |
| mission_ais | 100 | 90.86 | pass |
| mission_cmg | 50 | 36.61 | pass |
| mission_fmr | 50 | 37.85 | pass |
| mission_fmr_rcs | 50 | 37.68 | pass |
| mission_img | 50 | 37.87 | pass |
| mission_rw_rcs | 50 | 37.65 | pass |
| mission_vscmg | 50 | 36.36 | pass |
| nadir_hold_ais | 100 | 90.85 | pass |
| nadir_hold_ais_css | 100 | 90.85 | pass |
| safe_mode_ais | 100 | 90.85 | pass |
| slew_cmg | 50 | 36.65 | pass |
| slew_fmr | 50 | 37.91 | pass |
| slew_fmr_rcs | 50 | 37.76 | pass |
| slew_img | 50 | 37.93 | pass |
| slew_img_lqr | 50 | 37.92 | pass |
| slew_img_smc | 50 | 37.89 | pass |
| slew_rw_rcs | 50 | 37.7 | pass |
| slew_vscmg | 50 | 36.4 | pass |
| sun_acq_rotor_img | 50 | 38.26 | pass |
| sun_fine_img | 50 | 37.68 | pass |
| sun_mtq_ais | 100 | 90.86 | pass |
| sun_spin_ais | 100 | 92.96 | pass |

## Metrics: SILS vs soft OILS

| scenario | metric | req | SILS | soft OILS | SILS | soft OILS |
|---|---|---:|---:|---:|---|---|
| agile_slew_cmg | ape_los_on_target_p9973 (deg) | 0.01 | 0.005506 | 0.005496 | pass | pass |
| agile_slew_fmr_rcs | ape_los_on_target_p9973 (deg) | 0.01 | 0.006692 | 0.006729 | pass | pass |
| agile_slew_img | ape_los_on_target_p9973 (deg) | 0.01 | 0.007963 | 0.008006 | pass | pass |
| agile_slew_rw_rcs | ape_los_on_target_p9973 (deg) | 0.01 | 0.008167 | 0.008257 | pass | pass |
| agile_slew_vscmg | ape_los_on_target_p9973 (deg) | 0.01 | 0.01165 | 0.01183 | FAIL | FAIL |
| detumble_ais | detumble_time (min) | 284 | 37.74 | 38.02 | pass | pass |
| detumble_ais | power_mean (W) | 0.5 | 0.02192 | 0.02208 | pass | pass |
| detumble_ais | power_peak (W) | 1.5 | 0.5476 | 0.5475 | pass | pass |
| detumble_ais_bangbang | detumble_time (min) | 284 | 55.74 | 54.87 | pass | pass |
| detumble_ais_bangbang | power_mean (W) | 0.5 | 0.4556 | 0.4618 | pass | pass |
| detumble_ais_bangbang | power_peak (W) | 1.5 | 0.717 | 0.717 | pass | pass |
| detumble_ais_mag | detumble_time (min) | 284 | 41.09 | 41.59 | pass | pass |
| detumble_ais_mag | power_mean (W) | 0.5 | 0.07734 | 0.0806 | pass | pass |
| detumble_ais_mag | power_peak (W) | 1.5 | 0.6483 | 0.6412 | pass | pass |
| detumble_img | detumble_time (min) | 284 | 47.58 | 47.49 | pass | pass |
| detumble_rcs | detumble_time (min) | 284 | — | — | FAIL | FAIL |
| fault_coil_ais | detumble_time (min) | 284 | 82.19 | 82.37 | pass | pass |
| fault_coil_ais | power_mean (W) | 0.5 | 0.01819 | 0.01842 | pass | pass |
| fault_coil_ais | power_peak (W) | 1.5 | 0.5476 | 0.5475 | pass | pass |
| fault_gimbal_cmg | ape_los_p9973 (deg) | 0.01 | 0.005705 | 0.005745 | pass | pass |
| fault_gimbal_cmg | ake_los_p9973 (deg) | 0.005 | 0.002868 | 0.002868 | pass | pass |
| fault_gimbal_cmg | rate_stability_p9973 (deg/s) | 0.005 | 0.002987 | 0.003019 | pass | pass |
| fault_gimbal_cmg | power_mean (W) | 2 | 1.61 | 1.61 | pass | pass |
| fault_gps_img | ape_los_p9973 (deg) | 0.01 | 0.0112 | 0.0112 | FAIL | FAIL |
| fault_gps_img | ake_los_p9973 (deg) | 0.005 | 0.002839 | 0.002844 | pass | pass |
| fault_gps_img | power_mean (W) | 2 | 1.559 | 1.559 | pass | pass |
| fault_gyro_ais | ape_los_last_orbit (deg) | 10 | 10.92 | 11.57 | FAIL | FAIL |
| fault_gyro_ais | ake_los_last_orbit (deg) | 5 | 1.038 | 1.038 | pass | pass |
| fault_gyro_ais | power_mean (W) | 0.5 | 0.01084 | 0.0108 | pass | pass |
| fault_st_img | ape_los_p9973 (deg) | 0.01 | 0.00698 | 0.006973 | pass | pass |
| fault_st_img | ake_los_p9973 (deg) | 0.005 | 0.003056 | 0.003055 | pass | pass |
| fault_st_img | rate_stability_p9973 (deg/s) | 0.005 | 0.003484 | 0.003514 | pass | pass |
| fault_st_img | power_mean (W) | 2 | 1.559 | 1.559 | pass | pass |
| fault_valve_rcs | ape_los_p9973 (deg) | 0.01 | 2.749 | 2.749 | FAIL | FAIL |
| fault_valve_rcs | ake_los_p9973 (deg) | 0.005 | 0.002728 | 0.002724 | pass | pass |
| fault_valve_rcs | power_mean (W) | 2 | 1.513 | 1.513 | pass | pass |
| fault_wheel_img | ape_los_p9973 (deg) | 0.01 | 0.68 | 0.6739 | FAIL | FAIL |
| fault_wheel_img | ake_los_p9973 (deg) | 0.005 | 0.002983 | 0.002987 | pass | pass |
| fault_wheel_img | rate_stability_p9973 (deg/s) | 0.005 | 0.03143 | 0.03144 | FAIL | FAIL |
| fault_wheel_img | power_mean (W) | 2 | 1.138 | 1.152 | pass | pass |
| fine_hold_cmg | ape_los_p9973 (deg) | 0.01 | 0.005218 | 0.005224 | pass | pass |
| fine_hold_cmg | ake_los_p9973 (deg) | 0.005 | 0.002848 | 0.002847 | pass | pass |
| fine_hold_cmg | rate_stability_p9973 (deg/s) | 0.005 | 0.002469 | 0.002492 | pass | pass |
| fine_hold_cmg | power_mean (W) | 2 | 1.61 | 1.61 | pass | pass |
| fine_hold_fmr | ape_los_p9973 (deg) | 0.01 | 0.005962 | 0.00598 | pass | pass |
| fine_hold_fmr | ake_los_p9973 (deg) | 0.005 | 0.002868 | 0.00287 | pass | pass |
| fine_hold_fmr | rate_stability_p9973 (deg/s) | 0.005 | 0.004811 | 0.004878 | pass | pass |
| fine_hold_fmr | power_mean (W) | 2 | 0.1012 | 0.1023 | pass | pass |
| fine_hold_fmr_rcs | ape_los_p9973 (deg) | 0.01 | 0.005962 | 0.006001 | pass | pass |
| fine_hold_fmr_rcs | ake_los_p9973 (deg) | 0.005 | 0.002868 | 0.00287 | pass | pass |
| fine_hold_fmr_rcs | rate_stability_p9973 (deg/s) | 0.005 | 0.004811 | 0.004859 | pass | pass |
| fine_hold_fmr_rcs | power_mean (W) | 2 | 0.1012 | 0.1023 | pass | pass |
| fine_hold_img | ape_los_p9973 (deg) | 0.01 | 0.006873 | 0.006878 | pass | pass |
| fine_hold_img | ake_los_p9973 (deg) | 0.005 | 0.002842 | 0.00284 | pass | pass |
| fine_hold_img | rate_stability_p9973 (deg/s) | 0.005 | 0.003493 | 0.003527 | pass | pass |
| fine_hold_img | power_mean (W) | 2 | 1.559 | 1.559 | pass | pass |
| fine_hold_img | power_margin_last_orbit (W) | 0 | 1.645 | 1.645 | pass | pass |
| fine_hold_img_lqr | ape_los_p9973 (deg) | 0.01 | 0.004237 | 0.004254 | pass | pass |
| fine_hold_img_lqr | ake_los_p9973 (deg) | 0.005 | 0.002857 | 0.002855 | pass | pass |
| fine_hold_img_lqr | rate_stability_p9973 (deg/s) | 0.005 | 0.006898 | 0.006998 | FAIL | FAIL |
| fine_hold_img_lqr | power_mean (W) | 2 | 1.56 | 1.56 | pass | pass |
| fine_hold_img_smc | ape_los_p9973 (deg) | 0.01 | 0.01914 | 0.01911 | FAIL | FAIL |
| fine_hold_img_smc | ake_los_p9973 (deg) | 0.005 | 0.00275 | 0.002752 | pass | pass |
| fine_hold_img_smc | rate_stability_p9973 (deg/s) | 0.005 | 0.00339 | 0.00342 | pass | pass |
| fine_hold_img_smc | power_mean (W) | 2 | 1.559 | 1.559 | pass | pass |
| fine_hold_rw_rcs | ape_los_p9973 (deg) | 0.01 | 0.00681 | 0.006803 | pass | pass |
| fine_hold_rw_rcs | ake_los_p9973 (deg) | 0.005 | 0.002828 | 0.002846 | pass | pass |
| fine_hold_rw_rcs | rate_stability_p9973 (deg/s) | 0.005 | 0.003533 | 0.003494 | pass | pass |
| fine_hold_rw_rcs | power_mean (W) | 2 | 1.532 | 1.532 | pass | pass |
| fine_hold_vscmg | ape_los_p9973 (deg) | 0.01 | 0.008369 | 0.008449 | pass | pass |
| fine_hold_vscmg | ake_los_p9973 (deg) | 0.005 | 0.002852 | 0.00285 | pass | pass |
| fine_hold_vscmg | rate_stability_p9973 (deg/s) | 0.005 | 0.004737 | 0.004746 | pass | pass |
| fine_hold_vscmg | power_mean (W) | 2 | 2.014 | 2.014 | FAIL | FAIL |
| mission_ais | detumble_time (min) | 284 | 37.74 | 38.02 | pass | pass |
| mission_ais | ape_los_last_orbit (deg) | 10 | 11.5 | 11.45 | FAIL | FAIL |
| mission_ais | ake_los_last_orbit (deg) | 5 | 1.316 | 0.8371 | pass | pass |
| mission_ais | power_margin_last_orbit (W) | 0 | 6.736 | 6.436 | pass | pass |
| mission_cmg | detumble_time (min) | 284 | 56.77 | 56.96 | pass | pass |
| mission_cmg | ape_los_last_half_orbit_p9973 (deg) | 0.01 | 0.005214 | 0.005148 | pass | pass |
| mission_fmr | detumble_time (min) | 284 | 61.12 | 61.04 | pass | pass |
| mission_fmr | ape_los_last_half_orbit_p9973 (deg) | 0.01 | 0.005884 | 0.005894 | pass | pass |
| mission_fmr_rcs | detumble_time (min) | 284 | 61.12 | 61.04 | pass | pass |
| mission_fmr_rcs | ape_los_last_half_orbit_p9973 (deg) | 0.01 | 0.005884 | 0.005906 | pass | pass |
| mission_img | detumble_time (min) | 284 | 49.84 | 49.66 | pass | pass |
| mission_img | ape_los_last_half_orbit_p9973 (deg) | 0.01 | 0.00734 | 0.007413 | pass | pass |
| mission_img | power_margin_last_orbit (W) | 0 | 1.634 | 1.634 | pass | pass |
| mission_rw_rcs | detumble_time (min) | 284 | 49.83 | 49.61 | pass | pass |
| mission_rw_rcs | ape_los_last_half_orbit_p9973 (deg) | 0.01 | 0.01489 | 0.01086 | FAIL | FAIL |
| mission_vscmg | detumble_time (min) | 284 | 56.89 | 57.04 | pass | pass |
| mission_vscmg | ape_los_last_half_orbit_p9973 (deg) | 0.01 | 0.009745 | 0.009738 | pass | pass |
| nadir_hold_ais | ape_los_last_orbit (deg) | 10 | 10.67 | 9.76 | FAIL | pass ⚠ |
| nadir_hold_ais | ake_los_last_orbit (deg) | 5 | 1.638 | 1.279 | pass | pass |
| nadir_hold_ais | power_mean (W) | 0.5 | 0.0106 | 0.01063 | pass | pass |
| nadir_hold_ais | power_margin_last_orbit (W) | 0 | 6.506 | 6.609 | pass | pass |
| nadir_hold_ais_css | ape_los_last_orbit (deg) | 10 | 9.706 | 9.731 | pass | pass |
| nadir_hold_ais_css | ake_los_last_orbit (deg) | 5 | 1.569 | 1.382 | pass | pass |
| nadir_hold_ais_css | power_mean (W) | 0.5 | 0.01098 | 0.01105 | pass | pass |
| safe_mode_ais | ape_los_last_orbit (deg) | 10 | 12.3 | 16.9 | FAIL | FAIL |
| slew_cmg | settle_time_after_slew (s) | 20 | 12.4 | 12.5 | pass | pass |
| slew_cmg | ape_los_on_target_p9973 (deg) | 0.01 | 0.003681 | 0.003775 | pass | pass |
| slew_fmr | settle_time_after_slew (s) | 20 | 12.9 | 12.9 | pass | pass |
| slew_fmr | ape_los_on_target_p9973 (deg) | 0.01 | 0.006193 | 0.006271 | pass | pass |
| slew_fmr_rcs | settle_time_after_slew (s) | 20 | 12.9 | 12.9 | pass | pass |
| slew_fmr_rcs | ape_los_on_target_p9973 (deg) | 0.01 | 0.006193 | 0.006278 | pass | pass |
| slew_img | settle_time_after_slew (s) | 20 | 12.1 | 12.1 | pass | pass |
| slew_img | ape_los_on_target_p9973 (deg) | 0.01 | 0.007934 | 0.007925 | pass | pass |
| slew_img_lqr | settle_time_after_slew (s) | 20 | 0 | 0 | pass | pass |
| slew_img_lqr | ape_los_on_target_p9973 (deg) | 0.01 | 0.00471 | 0.004693 | pass | pass |
| slew_img_smc | settle_time_after_slew (s) | 20 | — | — | FAIL | FAIL |
| slew_img_smc | ape_los_on_target_p9973 (deg) | 0.01 | 0.01784 | 0.01794 | FAIL | FAIL |
| slew_rw_rcs | settle_time_after_slew (s) | 20 | 12.1 | 12.1 | pass | pass |
| slew_rw_rcs | ape_los_on_target_p9973 (deg) | 0.01 | 0.007921 | 0.007885 | pass | pass |
| slew_vscmg | settle_time_after_slew (s) | 20 | 13.2 | 13.2 | pass | pass |
| slew_vscmg | ape_los_on_target_p9973 (deg) | 0.01 | 0.01082 | 0.01075 | FAIL | FAIL |
| sun_acq_rotor_img | sun_acquisition_time (min) | 95 | 2.208 | 2.208 | pass | pass |
| sun_fine_img | sun_angle_p95 (deg) | 5 | 0.006891 | 0.00689 | pass | pass |
| sun_fine_img | power_margin_orbit (W) | 0 | 2.298 | 2.298 | pass | pass |
| sun_fine_img | power_mean (W) | 2 | 1.526 | 1.526 | pass | pass |
| sun_mtq_ais | sun_angle_last_orbit_p95 (deg) | 20 | 87.26 | 87.37 | FAIL | FAIL |
| sun_mtq_ais | power_margin_last_orbit (W) | 0 | 6.326 | 6.326 | pass | pass |
| sun_mtq_ais | power_mean (W) | 0.5 | 0.01113 | 0.0111 | pass | pass |
| sun_spin_ais | sun_spin_entry (min) | 95 | 63.22 | 131.9 | pass | FAIL ⚠ |
| sun_spin_ais | sun_angle_last_orbit (deg) | 20 | 142.3 | 33.49 | FAIL | FAIL |
| sun_spin_ais | spin_rate_error_last_orbit (deg/s) | 0.5 | 1.954 | 0.5721 | FAIL | FAIL |
| sun_spin_ais | sun_spin_share_last_orbit (%) | 90 | 0 | 30.35 | FAIL | FAIL |
| sun_spin_ais | power_mean (W) | 0.5 | 0.4641 | 0.3652 | pass | pass |
