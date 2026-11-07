# Soft OILS: SILS and the flight software on the virtual OBC, side by side

Owner: Agastya. `tools/engine.py oils` flies every scenario twice on the Rust engine (POP in the loop):
**SILS** with the C flight software in-process, and **soft OILS** with the flight software built for the
OBC (arm-none-eabi-gcc, Cortex-M4F hard-float) running as firmware in QEMU mps2-an386 (`--fsw qemu`)
behind adcs-link/1. In soft OILS every command reaches the actuators only after the OBC has read its
sensors on the buses, executed the step and sent its CAN frames; the previous command holds until then
(docs/SOFT_OILS.md). The step's execution is its **exact** instruction count (QEMU plugin
fsw/targets/qemu-mps2/insn_count.c) x CPI 1.25 / 168 MHz;
buses: I2C 400 kHz, SPI 1 MHz, CAN 1 Mbit/s.
Time base of the runs: qemu insn_count plugin (exact guest instructions).
Runs are deterministic: the same scenario gives the same instruction counts and trajectory every time.

**Verdict agreement SILS vs soft OILS: 166 of 171 judged metrics** over 80 scenarios.
**Accepted at the TEST_BENCH_PLAN section 5 tolerance** (difference <= 5%; a flipped verdict only within 5% of the requirement): 161 of 171.

## OBC timing budget

| scenario | period [ms] | instructions mean / max | exec max [ms] | latency mean / max [ms] | CPU load mean / max | overruns | deadline margin min [ms] |
|---|---:|---:|---:|---:|---:|---:|---:|
| agile_slew_cmg | 100 | 4.422e+05 / 1.037e+06 | 7.719 | 4.884 / 9.313 | 3.3 % / 7.7 % | 0 | 90.69 |
| agile_slew_fmr_rcs | 100 | 4.859e+05 / 1.08e+06 | 8.039 | 4.689 / 9.113 | 3.6 % / 8.0 % | 0 | 90.89 |
| agile_slew_img | 100 | 4.787e+05 / 1.075e+06 | 7.999 | 4.506 / 8.943 | 3.6 % / 8.0 % | 0 | 91.06 |
| agile_slew_rw_rcs | 100 | 4.897e+05 / 1.088e+06 | 8.094 | 4.718 / 9.168 | 3.6 % / 8.1 % | 0 | 90.83 |
| agile_slew_vscmg | 100 | 4.571e+05 / 1.057e+06 | 7.861 | 4.995 / 9.455 | 3.4 % / 7.9 % | 0 | 90.54 |
| detumble_ais | 200 | 1.825e+05 / 5.674e+05 | 4.221 | 1.912 / 4.775 | 0.7 % / 2.1 % | 0 | 195.2 |
| detumble_ais_bangbang | 200 | 1.809e+05 / 5.672e+05 | 4.22 | 1.9 / 4.774 | 0.7 % / 2.1 % | 0 | 195.2 |
| detumble_ais_mag | 200 | 1.821e+05 / 5.671e+05 | 4.22 | 1.909 / 4.774 | 0.7 % / 2.1 % | 0 | 195.2 |
| detumble_img | 100 | 1.495e+05 / 6.036e+05 | 4.491 | 2.056 / 5.435 | 1.1 % / 4.5 % | 0 | 94.56 |
| detumble_rcs | 100 | 1.497e+05 / 6.16e+05 | 4.583 | 2.188 / 5.657 | 1.1 % / 4.6 % | 0 | 94.34 |
| fault_bus_coils_brownout | 100 | 4.612e+05 / 1.069e+06 | 7.954 | 4.376 / 8.898 | 3.4 % / 8.0 % | 0 | 91.1 |
| fault_bus_coils_nack | 100 | 4.613e+05 / 1.069e+06 | 7.954 | 4.376 / 8.898 | 3.4 % / 8.0 % | 0 | 91.1 |
| fault_bus_coils_stuck | 100 | 4.612e+05 / 1.069e+06 | 7.955 | 4.376 / 8.899 | 3.4 % / 8.0 % | 0 | 91.1 |
| fault_bus_gnss_brownout | 100 | 4.591e+05 / 1.069e+06 | 7.956 | 4.36 / 8.9 | 3.4 % / 8.0 % | 0 | 91.1 |
| fault_bus_gnss_crc | 100 | 4.579e+05 / 1.035e+06 | 7.698 | 4.351 / 8.642 | 3.4 % / 7.7 % | 0 | 91.36 |
| fault_bus_gnss_nack | 100 | 4.575e+05 / 1.03e+06 | 7.665 | 4.348 / 8.609 | 3.4 % / 7.7 % | 0 | 91.39 |
| fault_bus_gnss_stuck | 100 | 4.17e+05 / 9.215e+05 | 6.856 | 4.047 / 7.8 | 3.1 % / 6.9 % | 0 | 92.2 |
| fault_bus_gyro_brownout | 100 | 2.873e+05 / 1.071e+06 | 7.968 | 3.026 / 8.808 | 2.1 % / 8.0 % | 0 | 91.19 |
| fault_bus_gyro_delay | 100 | 4.613e+05 / 1.071e+06 | 7.97 | 60.38 / 68.91 | 3.4 % / 8.0 % | 0 | 31.09 |
| fault_bus_gyro_nack | 100 | 2.869e+05 / 1.071e+06 | 7.968 | 2.982 / 8.808 | 2.1 % / 8.0 % | 0 | 91.19 |
| fault_bus_gyro_stuck | 100 | 2.881e+05 / 1.074e+06 | 7.992 | 3.088 / 8.936 | 2.1 % / 8.0 % | 0 | 91.06 |
| fault_bus_mag_brownout | 100 | 2.766e+05 / 1.063e+06 | 7.912 | 2.882 / 8.631 | 2.1 % / 7.9 % | 0 | 91.37 |
| fault_bus_mag_nack | 100 | 2.735e+05 / 1.063e+06 | 7.912 | 2.769 / 8.631 | 2.0 % / 7.9 % | 0 | 91.37 |
| fault_bus_mag_stuck | 100 | 4.612e+05 / 1.07e+06 | 7.959 | 4.376 / 8.903 | 3.4 % / 8.0 % | 0 | 91.1 |
| fault_bus_mex_brownout | 100 | 4.32e+05 / 1.058e+06 | 7.87 | 4.158 / 8.814 | 3.2 % / 7.9 % | 0 | 91.19 |
| fault_bus_mex_can_error | 100 | 4.474e+05 / 1.04e+06 | 7.74 | 3.909 / 8.294 | 3.3 % / 7.7 % | 0 | 91.71 |
| fault_bus_mex_nack | 100 | 4.474e+05 / 1.04e+06 | 7.74 | 3.909 / 8.294 | 3.3 % / 7.7 % | 0 | 91.71 |
| fault_bus_mex_stuck | 100 | 4.482e+05 / 1.04e+06 | 7.735 | 4.279 / 8.679 | 3.3 % / 7.7 % | 0 | 91.32 |
| fault_bus_st_brownout | 100 | 4.02e+05 / 1.071e+06 | 7.967 | 3.935 / 8.911 | 3.0 % / 8.0 % | 0 | 91.09 |
| fault_bus_st_crc | 100 | 3.418e+05 / 9.186e+05 | 6.835 | 3.487 / 7.779 | 2.5 % / 6.8 % | 0 | 92.22 |
| fault_bus_st_nack | 100 | 3.395e+05 / 9.186e+05 | 6.835 | 3.47 / 7.779 | 2.5 % / 6.8 % | 0 | 92.22 |
| fault_bus_st_stuck | 100 | 4.172e+05 / 9.234e+05 | 6.871 | 4.048 / 7.815 | 3.1 % / 6.9 % | 0 | 92.19 |
| fault_bus_sun_brownout | 100 | 4.601e+05 / 1.069e+06 | 7.955 | 4.248 / 8.899 | 3.4 % / 8.0 % | 0 | 91.1 |
| fault_bus_sun_nack | 100 | 4.593e+05 / 1.067e+06 | 7.939 | 4.152 / 8.658 | 3.4 % / 7.9 % | 0 | 91.34 |
| fault_bus_sun_stuck | 100 | 4.612e+05 / 1.069e+06 | 7.955 | 4.376 / 8.899 | 3.4 % / 8.0 % | 0 | 91.1 |
| fault_clock_jitter_1us | 100 | 4.877e+05 / 1.076e+06 | 8.005 | 4.572 / 8.949 | 3.6 % / 8.0 % | 0 | 91.05 |
| fault_clock_jitter_5ms | 100 | 4.877e+05 / 1.079e+06 | 8.027 | 4.572 / 8.971 | 3.6 % / 8.0 % | 0 | 91.03 |
| fault_clock_step_back | 200 | 2.898e+05 / 7.551e+05 | 5.618 | 2.71 / 6.172 | 1.1 % / 2.8 % | 0 | 193.8 |
| fault_clock_step_fwd | 200 | 2.898e+05 / 7.547e+05 | 5.615 | 2.71 / 6.169 | 1.1 % / 2.8 % | 0 | 193.8 |
| fault_coil_ais | 200 | 1.826e+05 / 5.672e+05 | 4.22 | 1.912 / 4.774 | 0.7 % / 2.1 % | 0 | 195.2 |
| fault_fsw_reset_day30 | 200 | 2.89e+05 / 7.548e+05 | 5.616 | 2.704 / 6.17 | 1.1 % / 2.8 % | 0 | 193.8 |
| fault_gimbal_cmg | 100 | 4.37e+05 / 1.041e+06 | 7.743 | 4.846 / 9.337 | 3.3 % / 7.7 % | 0 | 90.66 |
| fault_gps_img | 100 | 4.751e+05 / 1.079e+06 | 8.032 | 4.479 / 8.976 | 3.5 % / 8.0 % | 0 | 91.02 |
| fault_gyro_ais | 200 | 2.889e+05 / 7.562e+05 | 5.627 | 2.704 / 6.181 | 1.1 % / 2.8 % | 0 | 193.8 |
| fault_gyro_silent | 100 | 2.242e+05 / 1.076e+06 | 8.008 | 2.613 / 8.952 | 1.7 % / 8.0 % | 0 | 91.05 |
| fault_st_blind | 100 | 3.438e+05 / 1.076e+06 | 8.008 | 3.502 / 8.952 | 2.6 % / 8.0 % | 0 | 91.05 |
| fault_st_img | 100 | 4.309e+05 / 1.076e+06 | 8.008 | 4.15 / 8.952 | 3.2 % / 8.0 % | 0 | 91.05 |
| fault_valve_rcs | 100 | 4.859e+05 / 1.094e+06 | 8.137 | 4.689 / 9.211 | 3.6 % / 8.1 % | 0 | 90.79 |
| fault_wheel_img | 100 | 4.62e+05 / 1.076e+06 | 8.008 | 4.382 / 8.952 | 3.4 % / 8.0 % | 0 | 91.05 |
| fine_hold_cmg | 100 | 4.37e+05 / 1.042e+06 | 7.755 | 4.845 / 9.349 | 3.3 % / 7.8 % | 0 | 90.65 |
| fine_hold_fmr | 100 | 4.762e+05 / 1.08e+06 | 8.032 | 4.487 / 8.976 | 3.5 % / 8.0 % | 0 | 91.02 |
| fine_hold_fmr_rcs | 100 | 4.807e+05 / 1.084e+06 | 8.065 | 4.65 / 9.139 | 3.6 % / 8.1 % | 0 | 90.86 |
| fine_hold_img | 100 | 4.762e+05 / 1.082e+06 | 8.05 | 4.487 / 8.994 | 3.5 % / 8.0 % | 0 | 91.01 |
| fine_hold_img_lqr | 100 | 4.763e+05 / 1.08e+06 | 8.038 | 4.488 / 8.982 | 3.5 % / 8.0 % | 0 | 91.02 |
| fine_hold_img_smc | 100 | 4.792e+05 / 1.083e+06 | 8.06 | 4.509 / 9.004 | 3.6 % / 8.1 % | 0 | 91 |
| fine_hold_rw_rcs | 100 | 4.851e+05 / 1.091e+06 | 8.114 | 4.683 / 9.188 | 3.6 % / 8.1 % | 0 | 90.81 |
| fine_hold_vscmg | 100 | 4.582e+05 / 1.063e+06 | 7.909 | 5.003 / 9.503 | 3.4 % / 7.9 % | 0 | 90.5 |
| mission_ais | 200 | 2.776e+05 / 7.551e+05 | 5.618 | 2.619 / 6.172 | 1.0 % / 2.8 % | 0 | 193.8 |
| mission_cmg | 100 | 3.789e+05 / 1.04e+06 | 7.741 | 4.413 / 9.335 | 2.8 % / 7.7 % | 0 | 90.66 |
| mission_fmr | 100 | 4.084e+05 / 1.081e+06 | 8.046 | 3.983 / 8.99 | 3.0 % / 8.0 % | 0 | 91.01 |
| mission_fmr_rcs | 100 | 4.125e+05 / 1.088e+06 | 8.097 | 4.143 / 9.171 | 3.1 % / 8.1 % | 0 | 90.83 |
| mission_img | 100 | 4.191e+05 / 1.082e+06 | 8.051 | 4.062 / 8.995 | 3.1 % / 8.1 % | 0 | 91.01 |
| mission_rw_rcs | 100 | 4.266e+05 / 1.09e+06 | 8.111 | 4.248 / 9.185 | 3.2 % / 8.1 % | 0 | 90.82 |
| mission_vscmg | 100 | 3.965e+05 / 1.062e+06 | 7.901 | 4.544 / 9.495 | 3.0 % / 7.9 % | 0 | 90.51 |
| nadir_hold_ais | 200 | 2.89e+05 / 7.562e+05 | 5.627 | 2.704 / 6.181 | 1.1 % / 2.8 % | 0 | 193.8 |
| nadir_hold_ais_css | 200 | 2.889e+05 / 7.547e+05 | 5.615 | 2.704 / 6.169 | 1.1 % / 2.8 % | 0 | 193.8 |
| safe_mode_ais | 200 | 2.79e+05 / 7.562e+05 | 5.627 | 2.63 / 6.181 | 1.0 % / 2.8 % | 0 | 193.8 |
| slew_cmg | 100 | 4.424e+05 / 1.036e+06 | 7.711 | 4.886 / 9.305 | 3.3 % / 7.7 % | 0 | 90.69 |
| slew_fmr | 100 | 4.818e+05 / 1.076e+06 | 8.003 | 4.529 / 8.947 | 3.6 % / 8.0 % | 0 | 91.05 |
| slew_fmr_rcs | 100 | 4.862e+05 / 1.079e+06 | 8.031 | 4.691 / 9.105 | 3.6 % / 8.0 % | 0 | 90.89 |
| slew_img | 100 | 4.821e+05 / 1.077e+06 | 8.012 | 4.531 / 8.956 | 3.6 % / 8.0 % | 0 | 91.04 |
| slew_img_lqr | 100 | 4.821e+05 / 1.076e+06 | 8.004 | 4.531 / 8.948 | 3.6 % / 8.0 % | 0 | 91.05 |
| slew_img_smc | 100 | 4.85e+05 / 1.079e+06 | 8.03 | 4.553 / 8.974 | 3.6 % / 8.0 % | 0 | 91.03 |
| slew_rw_rcs | 100 | 4.884e+05 / 1.083e+06 | 8.06 | 4.708 / 9.134 | 3.6 % / 8.1 % | 0 | 90.87 |
| slew_vscmg | 100 | 4.635e+05 / 1.058e+06 | 7.871 | 5.043 / 9.465 | 3.4 % / 7.9 % | 0 | 90.54 |
| sun_acq_rotor_img | 100 | 3.99e+05 / 1.049e+06 | 7.802 | 3.913 / 8.746 | 3.0 % / 7.8 % | 0 | 91.25 |
| sun_fine_img | 100 | 4.682e+05 / 1.098e+06 | 8.172 | 4.428 / 9.116 | 3.5 % / 8.2 % | 0 | 90.88 |
| sun_mtq_ais | 200 | 2.933e+05 / 7.546e+05 | 5.615 | 2.736 / 6.169 | 1.1 % / 2.8 % | 0 | 193.8 |
| sun_spin_ais | 200 | 1.842e+05 / 5.749e+05 | 4.278 | 1.924 / 4.832 | 0.7 % / 2.1 % | 0 | 195.2 |
| target_img | 100 | 4.824e+05 / 1.087e+06 | 8.088 | 4.533 / 9.032 | 3.6 % / 8.1 % | 0 | 90.97 |

## The deadline, judged on the worst case

Each step's worst case: execution at CPI 2.0 (not 1.25), plus 50 us of interrupts that may
preempt it, plus the bus time; the command must land within 50 % of the control period. Both are
judged metrics of every soft-OILS run (`oils_overruns`, `oils_worst_case_margin`). The instruction count includes the HAL's
copies inside the step (on a real OBC those are the bus transfers, timed separately: counted twice, so conservative);
the adcs-link framing outside the step is the simulation's, not the OBC's, and is not counted.

| scenario | deadline [ms] | worst-case margin [ms] | verdict |
|---|---:|---:|---|
| agile_slew_cmg | 50 | 36.01 | pass |
| agile_slew_fmr_rcs | 50 | 36.01 | pass |
| agile_slew_img | 50 | 36.21 | pass |
| agile_slew_rw_rcs | 50 | 35.93 | pass |
| agile_slew_vscmg | 50 | 35.78 | pass |
| detumble_ais | 100 | 92.64 | pass |
| detumble_ais_bangbang | 100 | 92.64 | pass |
| detumble_ais_mag | 100 | 92.64 | pass |
| detumble_img | 50 | 41.82 | pass |
| detumble_rcs | 50 | 41.54 | pass |
| fault_bus_coils_brownout | 50 | 36.28 | pass |
| fault_bus_coils_nack | 50 | 36.28 | pass |
| fault_bus_coils_stuck | 50 | 36.28 | pass |
| fault_bus_gnss_brownout | 50 | 36.28 | pass |
| fault_bus_gnss_crc | 50 | 36.69 | pass |
| fault_bus_gnss_nack | 50 | 36.74 | pass |
| fault_bus_gnss_stuck | 50 | 38.04 | pass |
| fault_bus_gyro_brownout | 50 | 36.36 | pass |
| fault_bus_gyro_delay | 50 | -23.75 | FAIL |
| fault_bus_gyro_nack | 50 | 36.36 | pass |
| fault_bus_gyro_stuck | 50 | 36.22 | pass |
| fault_bus_mag_brownout | 50 | 36.57 | pass |
| fault_bus_mag_nack | 50 | 36.57 | pass |
| fault_bus_mag_stuck | 50 | 36.27 | pass |
| fault_bus_mex_brownout | 50 | 36.41 | pass |
| fault_bus_mex_can_error | 50 | 37.01 | pass |
| fault_bus_mex_nack | 50 | 37.01 | pass |
| fault_bus_mex_stuck | 50 | 36.63 | pass |
| fault_bus_st_brownout | 50 | 36.26 | pass |
| fault_bus_st_crc | 50 | 38.07 | pass |
| fault_bus_st_nack | 50 | 38.07 | pass |
| fault_bus_st_stuck | 50 | 38.01 | pass |
| fault_bus_sun_brownout | 50 | 36.28 | pass |
| fault_bus_sun_nack | 50 | 36.53 | pass |
| fault_bus_sun_stuck | 50 | 36.28 | pass |
| fault_clock_jitter_1us | 50 | 36.2 | pass |
| fault_clock_jitter_5ms | 50 | 36.16 | pass |
| fault_clock_step_back | 100 | 90.41 | pass |
| fault_clock_step_fwd | 100 | 90.41 | pass |
| fault_coil_ais | 100 | 92.64 | pass |
| fault_fsw_reset_day30 | 100 | 90.41 | pass |
| fault_gimbal_cmg | 50 | 35.97 | pass |
| fault_gps_img | 50 | 36.16 | pass |
| fault_gyro_ais | 100 | 90.39 | pass |
| fault_gyro_silent | 50 | 36.19 | pass |
| fault_st_blind | 50 | 36.19 | pass |
| fault_st_img | 50 | 36.19 | pass |
| fault_valve_rcs | 50 | 35.86 | pass |
| fault_wheel_img | 50 | 36.19 | pass |
| fine_hold_cmg | 50 | 35.95 | pass |
| fine_hold_fmr | 50 | 36.15 | pass |
| fine_hold_fmr_rcs | 50 | 35.97 | pass |
| fine_hold_img | 50 | 36.13 | pass |
| fine_hold_img_lqr | 50 | 36.14 | pass |
| fine_hold_img_smc | 50 | 36.11 | pass |
| fine_hold_rw_rcs | 50 | 35.89 | pass |
| fine_hold_vscmg | 50 | 35.7 | pass |
| mission_ais | 100 | 90.41 | pass |
| mission_cmg | 50 | 35.97 | pass |
| mission_fmr | 50 | 36.13 | pass |
| mission_fmr_rcs | 50 | 35.92 | pass |
| mission_img | 50 | 36.12 | pass |
| mission_rw_rcs | 50 | 35.9 | pass |
| mission_vscmg | 50 | 35.71 | pass |
| nadir_hold_ais | 100 | 90.39 | pass |
| nadir_hold_ais_css | 100 | 90.41 | pass |
| safe_mode_ais | 100 | 90.39 | pass |
| slew_cmg | 50 | 36.02 | pass |
| slew_fmr | 50 | 36.2 | pass |
| slew_fmr_rcs | 50 | 36.03 | pass |
| slew_img | 50 | 36.19 | pass |
| slew_img_lqr | 50 | 36.2 | pass |
| slew_img_smc | 50 | 36.16 | pass |
| slew_rw_rcs | 50 | 35.98 | pass |
| slew_vscmg | 50 | 35.76 | pass |
| sun_acq_rotor_img | 50 | 36.52 | pass |
| sun_fine_img | 50 | 35.93 | pass |
| sun_mtq_ais | 100 | 90.41 | pass |
| sun_spin_ais | 100 | 92.55 | pass |
| target_img | 50 | 36.07 | pass |

## Metrics: SILS vs soft OILS

| scenario | metric | req | SILS | soft OILS | SILS | soft OILS |
|---|---|---:|---:|---:|---|---|
| agile_slew_cmg | ape_los_on_target_p9973 (deg) | 0.01 | 0.004502 | 0.004509 | pass | pass |
| agile_slew_fmr_rcs | ape_los_on_target_p9973 (deg) | 0.01 | 0.006668 | 0.006679 | pass | pass |
| agile_slew_img | ape_los_on_target_p9973 (deg) | 0.01 | 0.006681 | 0.006636 | pass | pass |
| agile_slew_rw_rcs | ape_los_on_target_p9973 (deg) | 0.01 | 0.006559 | 0.006609 | pass | pass |
| agile_slew_vscmg | ape_los_on_target_p9973 (deg) | 0.01 | 0.009575 | 0.01041 | pass | FAIL ⚠ REJECTED: differs by 8.0% (limit 5%) |
| detumble_ais | detumble_time (min) | 284 | 37.74 | 38.04 | pass | pass |
| detumble_ais | power_mean (W) | 0.5 | 1.522 | 1.522 | FAIL | FAIL |
| detumble_ais | power_peak (W) | 1.5 | 2.048 | 2.047 | FAIL | FAIL |
| detumble_ais_bangbang | detumble_time (min) | 284 | 55.74 | 55.44 | pass | pass |
| detumble_ais_bangbang | power_mean (W) | 0.5 | 1.956 | 1.959 | FAIL | FAIL |
| detumble_ais_bangbang | power_peak (W) | 1.5 | 2.217 | 2.217 | FAIL | FAIL |
| detumble_ais_mag | detumble_time (min) | 284 | 41.09 | 41.62 | pass | pass |
| detumble_ais_mag | power_mean (W) | 0.5 | 1.577 | 1.581 | FAIL | FAIL |
| detumble_ais_mag | power_peak (W) | 1.5 | 2.148 | 2.144 | FAIL | FAIL |
| detumble_img | detumble_time (min) | 284 | 47.58 | 47.49 | pass | pass |
| detumble_rcs | detumble_time (min) | 284 | — | — | FAIL | FAIL |
| fault_bus_coils_brownout | fault_flag (count) | 1 | 0 | 0 | FAIL | FAIL |
| fault_bus_coils_nack | fault_flag (count) | 1 | 0 | 0 | FAIL | FAIL |
| fault_bus_coils_stuck | fault_flag (count) | 1 | 0 | 0 | FAIL | FAIL |
| fault_bus_gnss_brownout | fault_flag (count) | 1 | 0 | 0 | FAIL | FAIL |
| fault_bus_gnss_crc | fault_flag (count) | 1 | 0 | 0 | FAIL | FAIL |
| fault_bus_gnss_nack | fault_flag (count) | 1 | 0 | 0 | FAIL | FAIL |
| fault_bus_gnss_stuck | fault_flag (count) | 1 | 0 | 0 | FAIL | FAIL |
| fault_bus_gyro_brownout | fault_flag (count) | 1 | 1 | 1 | pass | pass |
| fault_bus_gyro_delay | fault_flag (count) | 1 | 0 | 0 | FAIL | FAIL |
| fault_bus_gyro_nack | fault_flag (count) | 1 | 1 | 1 | pass | pass |
| fault_bus_gyro_stuck | fault_flag (count) | 1 | 1 | 1 | pass | pass |
| fault_bus_mag_brownout | fault_flag (count) | 1 | 1 | 1 | pass | pass |
| fault_bus_mag_nack | fault_flag (count) | 1 | 1 | 1 | pass | pass |
| fault_bus_mag_stuck | fault_flag (count) | 1 | 0 | 0 | FAIL | FAIL |
| fault_bus_mex_brownout | fault_flag (count) | 1 | 1 | 1 | pass | pass |
| fault_bus_mex_can_error | fault_flag (count) | 1 | 1 | 1 | pass | pass |
| fault_bus_mex_nack | fault_flag (count) | 1 | 1 | 1 | pass | pass |
| fault_bus_mex_stuck | fault_flag (count) | 1 | 1 | 1 | pass | pass |
| fault_bus_st_brownout | fault_flag (count) | 1 | 0 | 0 | FAIL | FAIL |
| fault_bus_st_crc | fault_flag (count) | 1 | 0 | 0 | FAIL | FAIL |
| fault_bus_st_nack | fault_flag (count) | 1 | 0 | 0 | FAIL | FAIL |
| fault_bus_st_stuck | fault_flag (count) | 1 | 0 | 0 | FAIL | FAIL |
| fault_bus_sun_brownout | fault_flag (count) | 1 | 0 | 0 | FAIL | FAIL |
| fault_bus_sun_nack | fault_flag (count) | 1 | 0 | 0 | FAIL | FAIL |
| fault_bus_sun_stuck | fault_flag (count) | 1 | 0 | 0 | FAIL | FAIL |
| fault_clock_jitter_1us | fine_law_fraction (%) | 99.9 | 100 | 100 | pass | pass |
| fault_clock_jitter_1us | ape_los_p9973 (deg) | 0.01 | 0.004616 | 0.004579 | pass | pass |
| fault_clock_jitter_1us | ake_los_p9973 (deg) | 0.005 | 0.002289 | 0.002287 | pass | pass |
| fault_clock_jitter_5ms | fine_law_fraction (%) | 99.9 | 100 | 100 | pass | pass |
| fault_clock_jitter_5ms | ape_los_p9973 (deg) | 0.01 | 0.004633 | 0.00458 | pass | pass |
| fault_clock_jitter_5ms | ake_los_p9973 (deg) | 0.005 | 0.002288 | 0.002284 | pass | pass |
| fault_clock_step_back | ake_los_last_orbit (deg) | 5 | 1.158 | 1.14 | pass | pass |
| fault_clock_step_back | date_error_s (s) | 0.001 | 4.47e-06 | 4.47e-06 | pass | pass |
| fault_clock_step_back | schedule_replays (count) | 0 | 0 | 0 | pass | pass |
| fault_clock_step_fwd | ake_los_last_orbit (deg) | 5 | 1.158 | 1.14 | pass | pass |
| fault_clock_step_fwd | date_error_s (s) | 0.001 | 1.819e-12 | 1.819e-12 | pass | pass |
| fault_clock_step_fwd | schedule_replays (count) | 0 | 0 | 0 | pass | pass |
| fault_coil_ais | detumble_time (min) | 284 | 82.19 | 82.32 | pass | pass |
| fault_coil_ais | power_mean (W) | 0.5 | 1.518 | 1.519 | FAIL | FAIL |
| fault_coil_ais | power_peak (W) | 1.5 | 2.048 | 2.047 | FAIL | FAIL |
| fault_fsw_reset_day30 | ake_los_last_orbit (deg) | 5 | 1.047 | 1.117 | pass | pass |
| fault_fsw_reset_day30 | date_error_s (s) | 0.001 | 4.471e-06 | 4.471e-06 | pass | pass |
| fault_fsw_reset_day30 | schedule_replays (count) | 0 | 0 | 0 | pass | pass |
| fault_gimbal_cmg | ape_los_p9973 (deg) | 0.01 | 0.00502 | 0.005001 | pass | pass |
| fault_gimbal_cmg | ake_los_p9973 (deg) | 0.005 | 0.002853 | 0.002854 | pass | pass |
| fault_gimbal_cmg | rate_stability_p9973 (deg/s) | 0.005 | 0.003289 | 0.003328 | pass | pass |
| fault_gimbal_cmg | power_mean (W) | 2 | 5.11 | 5.11 | FAIL | FAIL |
| fault_gps_img | ape_los_p9973 (deg) | 0.01 | 0.01047 | 0.01048 | FAIL | FAIL |
| fault_gps_img | ake_los_p9973 (deg) | 0.005 | 0.00283 | 0.002837 | pass | pass |
| fault_gps_img | power_mean (W) | 2 | 5.059 | 5.059 | FAIL | FAIL |
| fault_gyro_ais | ape_los_last_orbit (deg) | 10 | 11.57 | 11.4 | FAIL | FAIL |
| fault_gyro_ais | ake_los_last_orbit (deg) | 5 | 1.044 | 1.042 | pass | pass |
| fault_gyro_ais | power_mean (W) | 0.5 | 1.511 | 1.511 | FAIL | FAIL |
| fault_gyro_silent | ake_3ax_max (deg) | 1 | 0.02421 | 0.02454 | pass | pass |
| fault_st_blind | rate_max_blind (deg/s) | 0.2 | 0.09574 | 0.09547 | pass | pass |
| fault_st_img | ape_los_p9973 (deg) | 0.01 | 0.005508 | 0.005508 | pass | pass |
| fault_st_img | ake_los_p9973 (deg) | 0.005 | 0.003059 | 0.003065 | pass | pass |
| fault_st_img | rate_stability_p9973 (deg/s) | 0.005 | 0.003552 | 0.00358 | pass | pass |
| fault_st_img | power_mean (W) | 2 | 5.059 | 5.059 | FAIL | FAIL |
| fault_valve_rcs | ape_los_p9973 (deg) | 0.01 | 1.221 | 1.221 | FAIL | FAIL |
| fault_valve_rcs | ake_los_p9973 (deg) | 0.005 | 0.002747 | 0.002746 | pass | pass |
| fault_valve_rcs | power_mean (W) | 2 | 5.013 | 5.013 | FAIL | FAIL |
| fault_wheel_img | ape_los_p9973 (deg) | 0.01 | 0.4723 | 0.4736 | FAIL | FAIL |
| fault_wheel_img | ake_los_p9973 (deg) | 0.005 | 0.002984 | 0.003109 | pass | pass |
| fault_wheel_img | rate_stability_p9973 (deg/s) | 0.005 | 0.04016 | 0.04142 | FAIL | FAIL |
| fault_wheel_img | power_mean (W) | 2 | 4.641 | 4.665 | FAIL | FAIL |
| fine_hold_cmg | ape_los_p9973 (deg) | 0.01 | 0.004055 | 0.004085 | pass | pass |
| fine_hold_cmg | ake_los_p9973 (deg) | 0.005 | 0.002597 | 0.002593 | pass | pass |
| fine_hold_cmg | rate_stability_p9973 (deg/s) | 0.005 | 0.002673 | 0.002696 | pass | pass |
| fine_hold_cmg | power_mean (W) | 2 | 5.11 | 5.11 | FAIL | FAIL |
| fine_hold_fmr | ape_los_p9973 (deg) | 0.01 | 0.004915 | 0.004924 | pass | pass |
| fine_hold_fmr | ake_los_p9973 (deg) | 0.005 | 0.002591 | 0.00259 | pass | pass |
| fine_hold_fmr | rate_stability_p9973 (deg/s) | 0.005 | 0.00498 | 0.005042 | pass | FAIL ⚠ |
| fine_hold_fmr | power_mean (W) | 2 | 3.601 | 3.61 | FAIL | FAIL |
| fine_hold_fmr_rcs | ape_los_p9973 (deg) | 0.01 | 0.004915 | 0.004926 | pass | pass |
| fine_hold_fmr_rcs | ake_los_p9973 (deg) | 0.005 | 0.002591 | 0.002589 | pass | pass |
| fine_hold_fmr_rcs | rate_stability_p9973 (deg/s) | 0.005 | 0.00498 | 0.005054 | pass | FAIL ⚠ |
| fine_hold_fmr_rcs | power_mean (W) | 2 | 3.601 | 3.616 | FAIL | FAIL |
| fine_hold_img | ape_los_p9973 (deg) | 0.01 | 0.005202 | 0.005214 | pass | pass |
| fine_hold_img | ake_los_p9973 (deg) | 0.005 | 0.002605 | 0.002598 | pass | pass |
| fine_hold_img | rate_stability_p9973 (deg/s) | 0.005 | 0.003654 | 0.003676 | pass | pass |
| fine_hold_img | power_mean (W) | 2 | 5.065 | 5.065 | FAIL | FAIL |
| fine_hold_img | power_margin_last_orbit (W) | 0 | -1.866 | -1.866 | FAIL | FAIL |
| fine_hold_img_lqr | ape_los_p9973 (deg) | 0.01 | 0.004128 | 0.004142 | pass | pass |
| fine_hold_img_lqr | ake_los_p9973 (deg) | 0.005 | 0.002639 | 0.002644 | pass | pass |
| fine_hold_img_lqr | rate_stability_p9973 (deg/s) | 0.005 | 0.006662 | 0.006752 | FAIL | FAIL |
| fine_hold_img_lqr | power_mean (W) | 2 | 5.066 | 5.065 | FAIL | FAIL |
| fine_hold_img_smc | ape_los_p9973 (deg) | 0.01 | 0.01077 | 0.01081 | FAIL | FAIL |
| fine_hold_img_smc | ake_los_p9973 (deg) | 0.005 | 0.002592 | 0.002591 | pass | pass |
| fine_hold_img_smc | rate_stability_p9973 (deg/s) | 0.005 | 0.003477 | 0.00352 | pass | pass |
| fine_hold_img_smc | power_mean (W) | 2 | 5.065 | 5.065 | FAIL | FAIL |
| fine_hold_rw_rcs | ape_los_p9973 (deg) | 0.01 | 0.005185 | 0.005231 | pass | pass |
| fine_hold_rw_rcs | ake_los_p9973 (deg) | 0.005 | 0.002606 | 0.0026 | pass | pass |
| fine_hold_rw_rcs | rate_stability_p9973 (deg/s) | 0.005 | 0.003648 | 0.003703 | pass | pass |
| fine_hold_rw_rcs | power_mean (W) | 2 | 5.037 | 5.037 | FAIL | FAIL |
| fine_hold_vscmg | ape_los_p9973 (deg) | 0.01 | 0.006398 | 0.006409 | pass | pass |
| fine_hold_vscmg | ake_los_p9973 (deg) | 0.005 | 0.002635 | 0.002632 | pass | pass |
| fine_hold_vscmg | rate_stability_p9973 (deg/s) | 0.005 | 0.004649 | 0.004681 | pass | pass |
| fine_hold_vscmg | power_mean (W) | 2 | 5.515 | 5.515 | FAIL | FAIL |
| mission_ais | detumble_time (min) | 284 | 37.74 | 38.04 | pass | pass |
| mission_ais | ape_los_last_orbit (deg) | 10 | 10.78 | 11.99 | FAIL | FAIL REJECTED: differs by 10.1% (limit 5%) |
| mission_ais | ake_los_last_orbit (deg) | 5 | 1.286 | 0.8006 | pass | pass REJECTED: differs by 9.7% (limit 5%) |
| mission_ais | power_margin_last_orbit (W) | 0 | 4.952 | 5.152 | pass | pass |
| mission_cmg | detumble_time (min) | 284 | 56.79 | 56.98 | pass | pass |
| mission_cmg | ape_los_last_half_orbit_p9973 (deg) | 0.01 | 0.00431 | 0.004265 | pass | pass |
| mission_fmr | detumble_time (min) | 284 | 61.44 | 61.49 | pass | pass |
| mission_fmr | ape_los_last_half_orbit_p9973 (deg) | 0.01 | 0.004874 | 0.004837 | pass | pass |
| mission_fmr_rcs | detumble_time (min) | 284 | 61.44 | 61.49 | pass | pass |
| mission_fmr_rcs | ape_los_last_half_orbit_p9973 (deg) | 0.01 | 0.004874 | 0.004831 | pass | pass |
| mission_img | detumble_time (min) | 284 | 49.86 | 49.64 | pass | pass |
| mission_img | ape_los_last_half_orbit_p9973 (deg) | 0.01 | 0.005449 | 0.005387 | pass | pass |
| mission_img | power_margin_last_orbit (W) | 0 | -1.866 | -1.866 | FAIL | FAIL |
| mission_rw_rcs | detumble_time (min) | 284 | 49.86 | 49.64 | pass | pass |
| mission_rw_rcs | ape_los_last_half_orbit_p9973 (deg) | 0.01 | 0.005363 | 0.005418 | pass | pass |
| mission_vscmg | detumble_time (min) | 284 | 56.89 | 57.06 | pass | pass |
| mission_vscmg | ape_los_last_half_orbit_p9973 (deg) | 0.01 | 0.006916 | 0.006853 | pass | pass |
| nadir_hold_ais | ape_los_last_orbit (deg) | 10 | 10.61 | 9.639 | FAIL | pass ⚠ REJECTED: differs by 9.2% (limit 5%) |
| nadir_hold_ais | ake_los_last_orbit (deg) | 5 | 1.605 | 1.269 | pass | pass REJECTED: differs by 6.7% (limit 5%) |
| nadir_hold_ais | power_mean (W) | 0.5 | 1.511 | 1.511 | FAIL | FAIL |
| nadir_hold_ais | power_margin_last_orbit (W) | 0 | 4.949 | 5.15 | pass | pass |
| nadir_hold_ais_css | ape_los_last_orbit (deg) | 10 | 9.709 | 9.737 | pass | pass |
| nadir_hold_ais_css | ake_los_last_orbit (deg) | 5 | 1.563 | 1.367 | pass | pass |
| nadir_hold_ais_css | power_mean (W) | 0.5 | 1.231 | 1.231 | FAIL | FAIL |
| safe_mode_ais | ape_los_last_orbit (deg) | 10 | 12.04 | 12.95 | FAIL | FAIL REJECTED: differs by 7.0% (limit 5%) |
| slew_cmg | settle_time_after_slew (s) | 20 | 5.7 | 5.7 | pass | pass |
| slew_cmg | ape_los_on_target_p9973 (deg) | 0.01 | 0.003666 | 0.003692 | pass | pass |
| slew_fmr | settle_time_after_slew (s) | 20 | 10.1 | 10.1 | pass | pass |
| slew_fmr | ape_los_on_target_p9973 (deg) | 0.01 | 0.006051 | 0.006145 | pass | pass |
| slew_fmr_rcs | settle_time_after_slew (s) | 20 | 10.1 | 10.1 | pass | pass |
| slew_fmr_rcs | ape_los_on_target_p9973 (deg) | 0.01 | 0.006051 | 0.006142 | pass | pass |
| slew_img | settle_time_after_slew (s) | 20 | 8.9 | 8.9 | pass | pass |
| slew_img | ape_los_on_target_p9973 (deg) | 0.01 | 0.006261 | 0.006257 | pass | pass |
| slew_img_lqr | settle_time_after_slew (s) | 20 | 0 | 0 | pass | pass |
| slew_img_lqr | ape_los_on_target_p9973 (deg) | 0.01 | 0.00469 | 0.004707 | pass | pass |
| slew_img_smc | settle_time_after_slew (s) | 20 | 8.9 | 8.9 | pass | pass |
| slew_img_smc | ape_los_on_target_p9973 (deg) | 0.01 | 0.01097 | 0.0109 | FAIL | FAIL |
| slew_rw_rcs | settle_time_after_slew (s) | 20 | 8.9 | 8.9 | pass | pass |
| slew_rw_rcs | ape_los_on_target_p9973 (deg) | 0.01 | 0.006262 | 0.006284 | pass | pass |
| slew_vscmg | settle_time_after_slew (s) | 20 | 9 | 8.9 | pass | pass |
| slew_vscmg | ape_los_on_target_p9973 (deg) | 0.01 | 0.007254 | 0.007288 | pass | pass |
| sun_acq_rotor_img | sun_acquisition_time (min) | 95 | 2.208 | 2.208 | pass | pass |
| sun_fine_img | sun_angle_p95 (deg) | 5 | 0.005969 | 0.005977 | pass | pass |
| sun_fine_img | power_margin_orbit (W) | 0 | -1.202 | -1.202 | FAIL | FAIL |
| sun_fine_img | power_mean (W) | 2 | 5.026 | 5.026 | FAIL | FAIL |
| sun_mtq_ais | sun_angle_last_orbit_p95 (deg) | 20 | 87.27 | 87.38 | FAIL | FAIL |
| sun_mtq_ais | power_margin_last_orbit (W) | 0 | 4.826 | 4.826 | pass | pass |
| sun_mtq_ais | power_mean (W) | 0.5 | 1.511 | 1.511 | FAIL | FAIL |
| sun_spin_ais | sun_spin_entry (min) | 95 | 63.22 | 75.61 | pass | pass REJECTED: differs by 13.0% (limit 5%) |
| sun_spin_ais | sun_angle_last_orbit (deg) | 20 | 142.3 | 37.25 | FAIL | FAIL REJECTED: differs by 73.8% (limit 5%) |
| sun_spin_ais | spin_rate_error_last_orbit (deg/s) | 0.5 | 1.954 | 0.1673 | FAIL | pass ⚠ REJECTED: differs by 91.4% (limit 5%) |
| sun_spin_ais | sun_spin_share_last_orbit (%) | 90 | 0 | 52.47 | FAIL | FAIL REJECTED: differs by 58.3% (limit 5%) |
| sun_spin_ais | power_mean (W) | 0.5 | 1.964 | 1.871 | FAIL | FAIL |
| target_img | ape_los_p9973 (deg) | 0.01 | 0.01163 | 0.01164 | FAIL | FAIL |
| target_img | ake_los_p9973 (deg) | 0.005 | 0.01144 | 0.01146 | FAIL | FAIL |
| target_img | power_mean (W) | 2 | 5.064 | 5.064 | FAIL | FAIL |
