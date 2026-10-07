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

**Verdict agreement SILS vs soft OILS: 127 of 129 judged metrics** over 48 scenarios.

## OBC timing budget

| scenario | period [ms] | instructions mean / max | exec max [ms] | latency mean / max [ms] | CPU load mean / max | overruns | deadline margin min [ms] |
|---|---:|---:|---:|---:|---:|---:|---:|
| agile_slew_cmg | 100 | 4.402e+05 / 1.032e+06 | 7.679 | 4.869 / 9.273 | 3.3 % / 7.7 % | 0 | 90.73 |
| agile_slew_fmr_rcs | 100 | 4.062e+05 / 9.987e+05 | 7.431 | 4.096 / 8.505 | 3.0 % / 7.4 % | 0 | 91.5 |
| agile_slew_img | 100 | 3.979e+05 / 9.935e+05 | 7.392 | 3.905 / 8.336 | 3.0 % / 7.4 % | 0 | 91.66 |
| agile_slew_rw_rcs | 100 | 4.099e+05 / 1.005e+06 | 7.478 | 4.124 / 8.552 | 3.0 % / 7.5 % | 0 | 91.45 |
| agile_slew_vscmg | 100 | 4.586e+05 / 1.052e+06 | 7.826 | 5.006 / 9.42 | 3.4 % / 7.8 % | 0 | 90.58 |
| detumble_ais | 200 | 1.811e+05 / 5.632e+05 | 4.19 | 1.901 / 4.744 | 0.7 % / 2.1 % | 0 | 195.3 |
| detumble_ais_bangbang | 200 | 1.794e+05 / 5.633e+05 | 4.191 | 1.889 / 4.745 | 0.7 % / 2.1 % | 0 | 195.3 |
| detumble_ais_mag | 200 | 1.806e+05 / 5.633e+05 | 4.191 | 1.898 / 4.745 | 0.7 % / 2.1 % | 0 | 195.3 |
| detumble_img | 100 | 1.485e+05 / 5.999e+05 | 4.463 | 2.049 / 5.407 | 1.1 % / 4.5 % | 0 | 94.59 |
| detumble_rcs | 100 | 1.49e+05 / 6.12e+05 | 4.553 | 2.183 / 5.627 | 1.1 % / 4.6 % | 0 | 94.37 |
| fault_coil_ais | 200 | 1.811e+05 / 5.632e+05 | 4.191 | 1.901 / 4.745 | 0.7 % / 2.1 % | 0 | 195.3 |
| fault_gimbal_cmg | 100 | 4.351e+05 / 1.036e+06 | 7.709 | 4.831 / 9.303 | 3.2 % / 7.7 % | 0 | 90.7 |
| fault_gps_img | 100 | 3.952e+05 / 9.969e+05 | 7.417 | 3.885 / 8.361 | 2.9 % / 7.4 % | 0 | 91.64 |
| fault_gyro_ais | 200 | 2.875e+05 / 8.752e+05 | 6.512 | 2.693 / 7.066 | 1.1 % / 3.3 % | 0 | 192.9 |
| fault_st_img | 100 | 3.509e+05 / 9.922e+05 | 7.382 | 3.555 / 8.326 | 2.6 % / 7.4 % | 0 | 91.67 |
| fault_valve_rcs | 100 | 4.062e+05 / 1.009e+06 | 7.507 | 4.097 / 8.581 | 3.0 % / 7.5 % | 0 | 91.42 |
| fault_wheel_img | 100 | 3.902e+05 / 9.959e+05 | 7.41 | 3.847 / 8.354 | 2.9 % / 7.4 % | 0 | 91.65 |
| fine_hold_cmg | 100 | 4.351e+05 / 1.037e+06 | 7.712 | 4.832 / 9.306 | 3.2 % / 7.7 % | 0 | 90.69 |
| fine_hold_fmr | 100 | 3.966e+05 / 9.974e+05 | 7.421 | 3.895 / 8.365 | 3.0 % / 7.4 % | 0 | 91.63 |
| fine_hold_fmr_rcs | 100 | 4.011e+05 / 1.002e+06 | 7.455 | 4.058 / 8.529 | 3.0 % / 7.5 % | 0 | 91.47 |
| fine_hold_img | 100 | 3.964e+05 / 9.966e+05 | 7.416 | 3.893 / 8.36 | 2.9 % / 7.4 % | 0 | 91.64 |
| fine_hold_img_lqr | 100 | 3.964e+05 / 9.967e+05 | 7.416 | 3.893 / 8.36 | 2.9 % / 7.4 % | 0 | 91.64 |
| fine_hold_img_smc | 100 | 3.993e+05 / 9.996e+05 | 7.438 | 3.915 / 8.382 | 3.0 % / 7.4 % | 0 | 91.62 |
| fine_hold_rw_rcs | 100 | 4.05e+05 / 1.009e+06 | 7.509 | 4.087 / 8.583 | 3.0 % / 7.5 % | 0 | 91.42 |
| fine_hold_vscmg | 100 | 4.564e+05 / 1.057e+06 | 7.864 | 4.99 / 9.458 | 3.4 % / 7.9 % | 0 | 90.54 |
| mission_ais | 200 | 2.763e+05 / 7.517e+05 | 5.593 | 2.609 / 6.147 | 1.0 % / 2.8 % | 0 | 193.9 |
| mission_cmg | 100 | 3.772e+05 / 1.036e+06 | 7.705 | 4.4 / 9.299 | 2.8 % / 7.7 % | 0 | 90.7 |
| mission_fmr | 100 | 3.44e+05 / 9.969e+05 | 7.417 | 3.504 / 8.361 | 2.6 % / 7.4 % | 0 | 91.64 |
| mission_fmr_rcs | 100 | 3.481e+05 / 1.001e+06 | 7.451 | 3.664 / 8.525 | 2.6 % / 7.5 % | 0 | 91.47 |
| mission_img | 100 | 3.525e+05 / 9.977e+05 | 7.424 | 3.567 / 8.368 | 2.6 % / 7.4 % | 0 | 91.63 |
| mission_rw_rcs | 100 | 3.605e+05 / 1.007e+06 | 7.494 | 3.756 / 8.568 | 2.7 % / 7.5 % | 0 | 91.43 |
| mission_vscmg | 100 | 3.948e+05 / 1.058e+06 | 7.87 | 4.531 / 9.464 | 2.9 % / 7.9 % | 0 | 90.54 |
| nadir_hold_ais | 200 | 2.877e+05 / 7.509e+05 | 5.587 | 2.694 / 6.141 | 1.1 % / 2.8 % | 0 | 193.9 |
| nadir_hold_ais_css | 200 | 2.876e+05 / 7.497e+05 | 5.578 | 2.694 / 6.132 | 1.1 % / 2.8 % | 0 | 193.9 |
| safe_mode_ais | 200 | 2.775e+05 / 7.509e+05 | 5.587 | 2.618 / 6.141 | 1.0 % / 2.8 % | 0 | 193.9 |
| slew_cmg | 100 | 4.405e+05 / 1.03e+06 | 7.666 | 4.871 / 9.26 | 3.3 % / 7.7 % | 0 | 90.74 |
| slew_fmr | 100 | 4.02e+05 / 9.916e+05 | 7.378 | 3.935 / 8.322 | 3.0 % / 7.4 % | 0 | 91.68 |
| slew_fmr_rcs | 100 | 4.065e+05 / 9.964e+05 | 7.414 | 4.099 / 8.488 | 3.0 % / 7.4 % | 0 | 91.51 |
| slew_img | 100 | 4.017e+05 / 9.913e+05 | 7.376 | 3.933 / 8.32 | 3.0 % / 7.4 % | 0 | 91.68 |
| slew_img_lqr | 100 | 4.017e+05 / 9.93e+05 | 7.388 | 3.933 / 8.332 | 3.0 % / 7.4 % | 0 | 91.67 |
| slew_img_smc | 100 | 4.046e+05 / 9.957e+05 | 7.409 | 3.954 / 8.353 | 3.0 % / 7.4 % | 0 | 91.65 |
| slew_rw_rcs | 100 | 4.08e+05 / 9.992e+05 | 7.434 | 4.11 / 8.508 | 3.0 % / 7.4 % | 0 | 91.49 |
| slew_vscmg | 100 | 4.616e+05 / 1.053e+06 | 7.832 | 5.028 / 9.426 | 3.4 % / 7.8 % | 0 | 90.57 |
| sun_acq_rotor_img | 100 | 3.148e+05 / 9.639e+05 | 7.172 | 3.286 / 8.116 | 2.3 % / 7.2 % | 0 | 91.88 |
| sun_fine_img | 100 | 3.878e+05 / 1.013e+06 | 7.537 | 3.829 / 8.481 | 2.9 % / 7.5 % | 0 | 91.52 |
| sun_mtq_ais | 200 | 2.92e+05 / 7.511e+05 | 5.588 | 2.726 / 6.142 | 1.1 % / 2.8 % | 0 | 193.9 |
| sun_spin_ais | 200 | 1.831e+05 / 5.714e+05 | 4.252 | 1.917 / 4.806 | 0.7 % / 2.1 % | 0 | 195.2 |
| target_img | 100 | 4.024e+05 / 1.005e+06 | 7.474 | 3.938 / 8.418 | 3.0 % / 7.5 % | 0 | 91.58 |

## The deadline, judged on the worst case

Each step's worst case: execution at CPI 2.0 (not 1.25), plus 50 us of interrupts that may
preempt it, plus the bus time; the command must land within 50 % of the control period. Both are
judged metrics of every soft-OILS run (`oils_overruns`, `oils_worst_case_margin`). The instruction count includes the HAL's
copies inside the step (on a real OBC those are the bus transfers, timed separately: counted twice, so conservative);
the adcs-link framing outside the step is the simulation's, not the OBC's, and is not counted.

| scenario | deadline [ms] | worst-case margin [ms] | verdict |
|---|---:|---:|---|
| agile_slew_cmg | 50 | 36.07 | pass |
| agile_slew_fmr_rcs | 50 | 36.99 | pass |
| agile_slew_img | 50 | 37.18 | pass |
| agile_slew_rw_rcs | 50 | 36.91 | pass |
| agile_slew_vscmg | 50 | 35.83 | pass |
| detumble_ais | 100 | 92.69 | pass |
| detumble_ais_bangbang | 100 | 92.69 | pass |
| detumble_ais_mag | 100 | 92.69 | pass |
| detumble_img | 50 | 41.86 | pass |
| detumble_rcs | 50 | 41.59 | pass |
| fault_coil_ais | 100 | 92.69 | pass |
| fault_gimbal_cmg | 50 | 36.02 | pass |
| fault_gps_img | 50 | 37.14 | pass |
| fault_gyro_ais | 100 | 88.98 | pass |
| fault_st_img | 50 | 37.19 | pass |
| fault_valve_rcs | 50 | 36.87 | pass |
| fault_wheel_img | 50 | 37.15 | pass |
| fine_hold_cmg | 50 | 36.02 | pass |
| fine_hold_fmr | 50 | 37.13 | pass |
| fine_hold_fmr_rcs | 50 | 36.95 | pass |
| fine_hold_img | 50 | 37.14 | pass |
| fine_hold_img_lqr | 50 | 37.14 | pass |
| fine_hold_img_smc | 50 | 37.11 | pass |
| fine_hold_rw_rcs | 50 | 36.86 | pass |
| fine_hold_vscmg | 50 | 35.77 | pass |
| mission_ais | 100 | 90.45 | pass |
| mission_cmg | 50 | 36.03 | pass |
| mission_fmr | 50 | 37.14 | pass |
| mission_fmr_rcs | 50 | 36.95 | pass |
| mission_img | 50 | 37.13 | pass |
| mission_rw_rcs | 50 | 36.89 | pass |
| mission_vscmg | 50 | 35.76 | pass |
| nadir_hold_ais | 100 | 90.46 | pass |
| nadir_hold_ais_css | 100 | 90.47 | pass |
| safe_mode_ais | 100 | 90.46 | pass |
| slew_cmg | 50 | 36.09 | pass |
| slew_fmr | 50 | 37.2 | pass |
| slew_fmr_rcs | 50 | 37.01 | pass |
| slew_img | 50 | 37.2 | pass |
| slew_img_lqr | 50 | 37.19 | pass |
| slew_img_smc | 50 | 37.15 | pass |
| slew_rw_rcs | 50 | 36.98 | pass |
| slew_vscmg | 50 | 35.82 | pass |
| sun_acq_rotor_img | 50 | 37.53 | pass |
| sun_fine_img | 50 | 36.95 | pass |
| sun_mtq_ais | 100 | 90.45 | pass |
| sun_spin_ais | 100 | 92.59 | pass |
| target_img | 50 | 37.05 | pass |

## Metrics: SILS vs soft OILS

| scenario | metric | req | SILS | soft OILS | SILS | soft OILS |
|---|---|---:|---:|---:|---|---|
| agile_slew_cmg | ape_los_on_target_p9973 (deg) | 0.01 | 0.005506 | 0.005556 | pass | pass |
| agile_slew_fmr_rcs | ape_los_on_target_p9973 (deg) | 0.01 | 0.006692 | 0.006745 | pass | pass |
| agile_slew_img | ape_los_on_target_p9973 (deg) | 0.01 | 0.007963 | 0.008026 | pass | pass |
| agile_slew_rw_rcs | ape_los_on_target_p9973 (deg) | 0.01 | 0.008167 | 0.00828 | pass | pass |
| agile_slew_vscmg | ape_los_on_target_p9973 (deg) | 0.01 | 0.01165 | 0.0118 | FAIL | FAIL |
| detumble_ais | detumble_time (min) | 284 | 37.74 | 38.04 | pass | pass |
| detumble_ais | power_mean (W) | 0.5 | 0.02192 | 0.0221 | pass | pass |
| detumble_ais | power_peak (W) | 1.5 | 0.5476 | 0.5475 | pass | pass |
| detumble_ais_bangbang | detumble_time (min) | 284 | 55.74 | 55.01 | pass | pass |
| detumble_ais_bangbang | power_mean (W) | 0.5 | 0.4556 | 0.4598 | pass | pass |
| detumble_ais_bangbang | power_peak (W) | 1.5 | 0.717 | 0.717 | pass | pass |
| detumble_ais_mag | detumble_time (min) | 284 | 41.09 | 41.59 | pass | pass |
| detumble_ais_mag | power_mean (W) | 0.5 | 0.07734 | 0.08094 | pass | pass |
| detumble_ais_mag | power_peak (W) | 1.5 | 0.6483 | 0.644 | pass | pass |
| detumble_img | detumble_time (min) | 284 | 47.58 | 47.49 | pass | pass |
| detumble_rcs | detumble_time (min) | 284 | — | — | FAIL | FAIL |
| fault_coil_ais | detumble_time (min) | 284 | 82.19 | 82.32 | pass | pass |
| fault_coil_ais | power_mean (W) | 0.5 | 0.01819 | 0.01858 | pass | pass |
| fault_coil_ais | power_peak (W) | 1.5 | 0.5476 | 0.5475 | pass | pass |
| fault_gimbal_cmg | ape_los_p9973 (deg) | 0.01 | 0.005705 | 0.005722 | pass | pass |
| fault_gimbal_cmg | ake_los_p9973 (deg) | 0.005 | 0.002868 | 0.002866 | pass | pass |
| fault_gimbal_cmg | rate_stability_p9973 (deg/s) | 0.005 | 0.002987 | 0.003034 | pass | pass |
| fault_gimbal_cmg | power_mean (W) | 2 | 1.61 | 1.61 | pass | pass |
| fault_gps_img | ape_los_p9973 (deg) | 0.01 | 0.0112 | 0.0112 | FAIL | FAIL |
| fault_gps_img | ake_los_p9973 (deg) | 0.005 | 0.002839 | 0.002841 | pass | pass |
| fault_gps_img | power_mean (W) | 2 | 1.559 | 1.559 | pass | pass |
| fault_gyro_ais | ape_los_last_orbit (deg) | 10 | 10.92 | 11.58 | FAIL | FAIL |
| fault_gyro_ais | ake_los_last_orbit (deg) | 5 | 1.038 | 1.038 | pass | pass |
| fault_gyro_ais | power_mean (W) | 0.5 | 0.01084 | 0.0108 | pass | pass |
| fault_st_img | ape_los_p9973 (deg) | 0.01 | 0.00698 | 0.006968 | pass | pass |
| fault_st_img | ake_los_p9973 (deg) | 0.005 | 0.003056 | 0.003055 | pass | pass |
| fault_st_img | rate_stability_p9973 (deg/s) | 0.005 | 0.003484 | 0.003516 | pass | pass |
| fault_st_img | power_mean (W) | 2 | 1.559 | 1.559 | pass | pass |
| fault_valve_rcs | ape_los_p9973 (deg) | 0.01 | 2.749 | 2.749 | FAIL | FAIL |
| fault_valve_rcs | ake_los_p9973 (deg) | 0.005 | 0.002728 | 0.002727 | pass | pass |
| fault_valve_rcs | power_mean (W) | 2 | 1.513 | 1.513 | pass | pass |
| fault_wheel_img | ape_los_p9973 (deg) | 0.01 | 0.68 | 0.6737 | FAIL | FAIL |
| fault_wheel_img | ake_los_p9973 (deg) | 0.005 | 0.002983 | 0.003009 | pass | pass |
| fault_wheel_img | rate_stability_p9973 (deg/s) | 0.005 | 0.03143 | 0.03139 | FAIL | FAIL |
| fault_wheel_img | power_mean (W) | 2 | 1.138 | 1.154 | pass | pass |
| fine_hold_cmg | ape_los_p9973 (deg) | 0.01 | 0.005218 | 0.005214 | pass | pass |
| fine_hold_cmg | ake_los_p9973 (deg) | 0.005 | 0.002848 | 0.002848 | pass | pass |
| fine_hold_cmg | rate_stability_p9973 (deg/s) | 0.005 | 0.002469 | 0.002493 | pass | pass |
| fine_hold_cmg | power_mean (W) | 2 | 1.61 | 1.61 | pass | pass |
| fine_hold_fmr | ape_los_p9973 (deg) | 0.01 | 0.005962 | 0.005983 | pass | pass |
| fine_hold_fmr | ake_los_p9973 (deg) | 0.005 | 0.002868 | 0.002873 | pass | pass |
| fine_hold_fmr | rate_stability_p9973 (deg/s) | 0.005 | 0.004811 | 0.004868 | pass | pass |
| fine_hold_fmr | power_mean (W) | 2 | 0.1012 | 0.1023 | pass | pass |
| fine_hold_fmr_rcs | ape_los_p9973 (deg) | 0.01 | 0.005962 | 0.005996 | pass | pass |
| fine_hold_fmr_rcs | ake_los_p9973 (deg) | 0.005 | 0.002868 | 0.002868 | pass | pass |
| fine_hold_fmr_rcs | rate_stability_p9973 (deg/s) | 0.005 | 0.004811 | 0.004877 | pass | pass |
| fine_hold_fmr_rcs | power_mean (W) | 2 | 0.1012 | 0.1023 | pass | pass |
| fine_hold_img | ape_los_p9973 (deg) | 0.01 | 0.006873 | 0.006889 | pass | pass |
| fine_hold_img | ake_los_p9973 (deg) | 0.005 | 0.002842 | 0.002839 | pass | pass |
| fine_hold_img | rate_stability_p9973 (deg/s) | 0.005 | 0.003493 | 0.003527 | pass | pass |
| fine_hold_img | power_mean (W) | 2 | 1.559 | 1.559 | pass | pass |
| fine_hold_img | power_margin_last_orbit (W) | 0 | 1.645 | 1.645 | pass | pass |
| fine_hold_img_lqr | ape_los_p9973 (deg) | 0.01 | 0.004237 | 0.004252 | pass | pass |
| fine_hold_img_lqr | ake_los_p9973 (deg) | 0.005 | 0.002857 | 0.002857 | pass | pass |
| fine_hold_img_lqr | rate_stability_p9973 (deg/s) | 0.005 | 0.006898 | 0.007008 | FAIL | FAIL |
| fine_hold_img_lqr | power_mean (W) | 2 | 1.56 | 1.56 | pass | pass |
| fine_hold_img_smc | ape_los_p9973 (deg) | 0.01 | 0.01914 | 0.01909 | FAIL | FAIL |
| fine_hold_img_smc | ake_los_p9973 (deg) | 0.005 | 0.00275 | 0.002751 | pass | pass |
| fine_hold_img_smc | rate_stability_p9973 (deg/s) | 0.005 | 0.00339 | 0.003412 | pass | pass |
| fine_hold_img_smc | power_mean (W) | 2 | 1.559 | 1.559 | pass | pass |
| fine_hold_rw_rcs | ape_los_p9973 (deg) | 0.01 | 0.00681 | 0.006829 | pass | pass |
| fine_hold_rw_rcs | ake_los_p9973 (deg) | 0.005 | 0.002828 | 0.002834 | pass | pass |
| fine_hold_rw_rcs | rate_stability_p9973 (deg/s) | 0.005 | 0.003533 | 0.003523 | pass | pass |
| fine_hold_rw_rcs | power_mean (W) | 2 | 1.532 | 1.532 | pass | pass |
| fine_hold_vscmg | ape_los_p9973 (deg) | 0.01 | 0.008369 | 0.008412 | pass | pass |
| fine_hold_vscmg | ake_los_p9973 (deg) | 0.005 | 0.002852 | 0.002854 | pass | pass |
| fine_hold_vscmg | rate_stability_p9973 (deg/s) | 0.005 | 0.004737 | 0.004746 | pass | pass |
| fine_hold_vscmg | power_mean (W) | 2 | 2.014 | 2.014 | FAIL | FAIL |
| mission_ais | detumble_time (min) | 284 | 37.74 | 38.04 | pass | pass |
| mission_ais | ape_los_last_orbit (deg) | 10 | 11.5 | 12.35 | FAIL | FAIL |
| mission_ais | ake_los_last_orbit (deg) | 5 | 1.316 | 0.7793 | pass | pass |
| mission_ais | power_margin_last_orbit (W) | 0 | 6.736 | 6.812 | pass | pass |
| mission_cmg | detumble_time (min) | 284 | 56.77 | 56.96 | pass | pass |
| mission_cmg | ape_los_last_half_orbit_p9973 (deg) | 0.01 | 0.005214 | 0.005139 | pass | pass |
| mission_fmr | detumble_time (min) | 284 | 61.12 | 61.04 | pass | pass |
| mission_fmr | ape_los_last_half_orbit_p9973 (deg) | 0.01 | 0.005884 | 0.005881 | pass | pass |
| mission_fmr_rcs | detumble_time (min) | 284 | 61.12 | 61.04 | pass | pass |
| mission_fmr_rcs | ape_los_last_half_orbit_p9973 (deg) | 0.01 | 0.005884 | 0.005872 | pass | pass |
| mission_img | detumble_time (min) | 284 | 49.84 | 49.61 | pass | pass |
| mission_img | ape_los_last_half_orbit_p9973 (deg) | 0.01 | 0.00734 | 0.007387 | pass | pass |
| mission_img | power_margin_last_orbit (W) | 0 | 1.634 | 1.634 | pass | pass |
| mission_rw_rcs | detumble_time (min) | 284 | 49.83 | 49.61 | pass | pass |
| mission_rw_rcs | ape_los_last_half_orbit_p9973 (deg) | 0.01 | 0.01489 | 0.01783 | FAIL | FAIL |
| mission_vscmg | detumble_time (min) | 284 | 56.89 | 57.04 | pass | pass |
| mission_vscmg | ape_los_last_half_orbit_p9973 (deg) | 0.01 | 0.009745 | 0.009773 | pass | pass |
| nadir_hold_ais | ape_los_last_orbit (deg) | 10 | 10.67 | 9.721 | FAIL | pass ⚠ |
| nadir_hold_ais | ake_los_last_orbit (deg) | 5 | 1.638 | 1.275 | pass | pass |
| nadir_hold_ais | power_mean (W) | 0.5 | 0.0106 | 0.01063 | pass | pass |
| nadir_hold_ais | power_margin_last_orbit (W) | 0 | 6.506 | 6.615 | pass | pass |
| nadir_hold_ais_css | ape_los_last_orbit (deg) | 10 | 9.706 | 9.734 | pass | pass |
| nadir_hold_ais_css | ake_los_last_orbit (deg) | 5 | 1.569 | 1.374 | pass | pass |
| nadir_hold_ais_css | power_mean (W) | 0.5 | 0.01098 | 0.01105 | pass | pass |
| safe_mode_ais | ape_los_last_orbit (deg) | 10 | 12.3 | 17.5 | FAIL | FAIL |
| slew_cmg | settle_time_after_slew (s) | 20 | 12.4 | 12.5 | pass | pass |
| slew_cmg | ape_los_on_target_p9973 (deg) | 0.01 | 0.003681 | 0.003703 | pass | pass |
| slew_fmr | settle_time_after_slew (s) | 20 | 12.9 | 12.9 | pass | pass |
| slew_fmr | ape_los_on_target_p9973 (deg) | 0.01 | 0.006193 | 0.006279 | pass | pass |
| slew_fmr_rcs | settle_time_after_slew (s) | 20 | 12.9 | 12.9 | pass | pass |
| slew_fmr_rcs | ape_los_on_target_p9973 (deg) | 0.01 | 0.006193 | 0.006202 | pass | pass |
| slew_img | settle_time_after_slew (s) | 20 | 12.1 | 12.1 | pass | pass |
| slew_img | ape_los_on_target_p9973 (deg) | 0.01 | 0.007934 | 0.007927 | pass | pass |
| slew_img_lqr | settle_time_after_slew (s) | 20 | 0 | 0 | pass | pass |
| slew_img_lqr | ape_los_on_target_p9973 (deg) | 0.01 | 0.00471 | 0.004691 | pass | pass |
| slew_img_smc | settle_time_after_slew (s) | 20 | — | — | FAIL | FAIL |
| slew_img_smc | ape_los_on_target_p9973 (deg) | 0.01 | 0.01784 | 0.01793 | FAIL | FAIL |
| slew_rw_rcs | settle_time_after_slew (s) | 20 | 12.1 | 12.1 | pass | pass |
| slew_rw_rcs | ape_los_on_target_p9973 (deg) | 0.01 | 0.007921 | 0.007903 | pass | pass |
| slew_vscmg | settle_time_after_slew (s) | 20 | 13.2 | 13.1 | pass | pass |
| slew_vscmg | ape_los_on_target_p9973 (deg) | 0.01 | 0.01082 | 0.01074 | FAIL | FAIL |
| sun_acq_rotor_img | sun_acquisition_time (min) | 95 | 2.208 | 2.208 | pass | pass |
| sun_fine_img | sun_angle_p95 (deg) | 5 | 0.006891 | 0.006888 | pass | pass |
| sun_fine_img | power_margin_orbit (W) | 0 | 2.298 | 2.298 | pass | pass |
| sun_fine_img | power_mean (W) | 2 | 1.526 | 1.526 | pass | pass |
| sun_mtq_ais | sun_angle_last_orbit_p95 (deg) | 20 | 87.26 | 87.38 | FAIL | FAIL |
| sun_mtq_ais | power_margin_last_orbit (W) | 0 | 6.326 | 6.326 | pass | pass |
| sun_mtq_ais | power_mean (W) | 0.5 | 0.01113 | 0.0111 | pass | pass |
| sun_spin_ais | sun_spin_entry (min) | 95 | 63.22 | 64.02 | pass | pass |
| sun_spin_ais | sun_angle_last_orbit (deg) | 20 | 142.3 | 44.27 | FAIL | FAIL |
| sun_spin_ais | spin_rate_error_last_orbit (deg/s) | 0.5 | 1.954 | 0.1258 | FAIL | pass ⚠ |
| sun_spin_ais | sun_spin_share_last_orbit (%) | 90 | 0 | 45.3 | FAIL | FAIL |
| sun_spin_ais | power_mean (W) | 0.5 | 0.4641 | 0.3965 | pass | pass |
| target_img | ape_los_p9973 (deg) | 0.01 | 0.01207 | 0.01204 | FAIL | FAIL |
| target_img | ake_los_p9973 (deg) | 0.005 | 0.01149 | 0.01146 | FAIL | FAIL |
| target_img | power_mean (W) | 2 | 1.564 | 1.564 | pass | pass |
