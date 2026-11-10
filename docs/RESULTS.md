# TRI-NETRA ADCS — SILS results

**Owner: Agastya.** Copyright (c) 2026 Agastya. All rights reserved.

Produced by `matlab_sils` (GNU Octave 8.4) with the Precision Orbit Propagator stepped inside the attitude loop.
Figures: `python3 tools/report.py` draws them into `results/figures/` and `results/index.html` where the runs were flown (not committed); one run's: `adcs figures <run>` or `adcs report <run>`. Single runs are the nominal case, seed 1.

## AIS 3U, magnetorquers only (10°, SSO dawn–dusk)

### `detumble_ais` — 3U AIS — B-dot detumble from a 10 deg/s tumble (coils only)

| metric | value | required | verdict |
|---|---|---|---|
| detumble_time | 37.74 min | 284 | ✔ PASS |
| power_mean | 1.522 W | 0.5 | ✖ FAIL |
| power_peak | 2.048 W | 1.5 | ✖ FAIL |

### `nadir_hold_ais` — 3U AIS — nadir hold with magnetorquers only (10 deg requirement)

| metric | value | required | verdict |
|---|---|---|---|
| ape_3axis_last_orbit | 141.2 deg | — | — |
| ape_los_last_orbit | 10.61 deg | 10 | ✖ FAIL |
| ake_los_last_orbit | 1.605 deg | 5 | ✔ PASS |
| power_mean | 1.511 W | 0.5 | ✖ FAIL |
| power_margin_last_orbit | 4.949 W | 0 | ✔ PASS |
| battery_dod_last_orbit | 4.716 % | — | — |

### `nadir_hold_ais_css` — 3U AIS — nadir hold with coarse cosine sun sensors (albedo)

| metric | value | required | verdict |
|---|---|---|---|
| ape_3axis_last_orbit | 180 deg | — | — |
| ape_los_last_orbit | 9.709 deg | 10 | ✔ PASS |
| ake_los_last_orbit | 1.563 deg | 5 | ✔ PASS |
| power_mean | 1.231 W | 0.5 | ✖ FAIL |

### `sun_spin_ais` — 3U AIS — coils-only Sun acquisition: detumble, spin-up, Sun-pointing spin

| metric | value | required | verdict |
|---|---|---|---|
| sun_spin_entry | 63.44 min | 95 | ✔ PASS |
| sun_angle_last_orbit | 110.9 deg | 20 | ✖ FAIL |
| spin_rate_error_last_orbit | 1.394 deg/s | 0.5 | ✖ FAIL |
| sun_spin_share_last_orbit | 0.7318 % | 90 | ✖ FAIL |
| power_mean | 1.972 W | 0.5 | ✖ FAIL |

Events: 0 s detumble; 663 s spinup; 3806 s sun_spin; 3845 s spinup; 4067 s sun_spin; 4111 s spinup; 8414 s sun_spin; 8455 s spinup; 9346 s sun_spin; 9384 s spinup; 14137 s sun_spin; 14179 s spinup

### `mission_ais` — 3U AIS — integrated chain: detumble then nadir hold (auto mode switch)

| metric | value | required | verdict |
|---|---|---|---|
| detumble_time | 37.74 min | 284 | ✔ PASS |
| ape_los_last_orbit | 10.78 deg | 10 | ✖ FAIL |
| ape_3axis_last_orbit | 140.2 deg | — | — |
| ake_los_last_orbit | 1.286 deg | 5 | ✔ PASS |
| power_margin_last_orbit | 4.951 W | 0 | ✔ PASS |
| battery_dod_last_orbit | 4.714 % | — | — |

Events: 0 s detumble; 3299 s nadir_mtq

### `fault_coil_ais` — 3U AIS — X coil fails during detumble

| metric | value | required | verdict |
|---|---|---|---|
| detumble_time | 82.19 min | 284 | ✔ PASS |
| power_mean | 1.518 W | 0.5 | ✖ FAIL |
| power_peak | 2.048 W | 1.5 | ✖ FAIL |

Events: 0 s detumble; 60 s FAULT injected: coil_fail 1; 60 s detumble

### `fault_gyro_ais` — 3U AIS — gyro bias step of 0.1 deg/s during nadir hold

| metric | value | required | verdict |
|---|---|---|---|
| ape_3axis_last_orbit | 142.2 deg | — | — |
| ape_los_last_orbit | 11.57 deg | 10 | ✖ FAIL |
| ake_los_last_orbit | 1.044 deg | 5 | ✔ PASS |
| power_mean | 1.511 W | 0.5 | ✖ FAIL |

Events: 0 s nadir_mtq; 6000 s FAULT injected: gyro_bias_step 0; 6000 s nadir_mtq

## Imaging 3U, magnetorquers + reaction wheels (0.01°, SSO 10:00)

### `detumble_img` — 3U imaging — B-dot detumble from a 10 deg/s tumble

| metric | value | required | verdict |
|---|---|---|---|
| detumble_time | 47.58 min | 284 | ✔ PASS |

### `fine_hold_img` — 3U imaging — fine nadir hold, reaction wheels + star tracker (0.01 deg)

| metric | value | required | verdict |
|---|---|---|---|
| ape_los_p9973 | 0.005202 deg | 0.01 | ✔ PASS |
| ape_los_max | 0.005977 deg | — | — |
| ape_3ax_p9973 | 0.006157 deg | — | — |
| ake_los_p9973 | 0.002603 deg | 0.005 | ✔ PASS |
| rate_stability_p9973 | 0.003651 deg/s | 0.005 | ✔ PASS |
| time_to_0p01_deg | 42.7 s | — | — |
| wheel_momentum_peak | 0.002615 N m s | — | — |
| power_mean | 5.065 W | 2 | ✖ FAIL |
| rpe_los_10s_p9973 | 0.003877 deg | — | — |
| pde_los_10s_60s_max | 0.005467 deg | — | — |
| rke_los_10s_p9973 | 0.001834 deg | — | — |
| power_margin_last_orbit | -1.866 W | 0 | ✖ FAIL |
| battery_dod_last_orbit | 26.77 % | — | — |

### `slew_img` — 3U imaging — 30 deg target slew in 60 s and settle

| metric | value | required | verdict |
|---|---|---|---|
| settle_time_after_slew | 8.9 s | 20 | ✔ PASS |
| ape_los_on_target_p9973 | 0.006261 deg | 0.01 | ✔ PASS |
| wheel_momentum_peak | 0.002759 N m s | — | — |

### `agile_slew_img` — 3U imaging — (img): 90 deg pitch in 15 s

| metric | value | required | verdict |
|---|---|---|---|
| settle_time_after_slew | 37.2 s | — | — |
| ape_los_on_target_p9973 | 0.006681 deg | 0.01 | ✔ PASS |
| wheel_momentum_peak | 0.007236 N m s | — | — |

### `mission_img` — 3U imaging — integrated chain: detumble then fine nadir hold

| metric | value | required | verdict |
|---|---|---|---|
| detumble_time | 49.86 min | 284 | ✔ PASS |
| ape_los_last_half_orbit_p9973 | 0.005448 deg | 0.01 | ✔ PASS |
| power_margin_last_orbit | -1.866 W | 0 | ✖ FAIL |
| battery_dod_last_orbit | 32.6 % | — | — |

Events: 0 s detumble; 2914 s nadir_fine

### `fault_wheel_img` — 3U imaging — reaction wheel 1 fails during fine hold (FDIR + coils)

| metric | value | required | verdict |
|---|---|---|---|
| ape_los_p9973 | 0.4724 deg | 0.01 | ✖ FAIL |
| ape_los_max | 0.4909 deg | — | — |
| ape_3ax_p9973 | 0.4724 deg | — | — |
| ake_los_p9973 | 0.00298 deg | 0.005 | ✔ PASS |
| rate_stability_p9973 | 0.04021 deg/s | 0.005 | ✖ FAIL |
| time_to_0p01_deg | 42.7 s | — | — |
| wheel_momentum_peak | 0.002596 N m s | — | — |
| power_mean | 4.641 W | 2 | ✖ FAIL |

Events: 0 s nadir_fine; 1200 s FAULT injected: rotor_fail 1; 1200 s nadir_fine

### `fault_st_img` — 3U imaging — star-tracker head 2 fails during fine hold

| metric | value | required | verdict |
|---|---|---|---|
| ape_los_p9973 | 0.00551 deg | 0.01 | ✔ PASS |
| ape_los_max | 0.006915 deg | — | — |
| ape_3ax_p9973 | 0.007403 deg | — | — |
| ake_los_p9973 | 0.003062 deg | 0.005 | ✔ PASS |
| rate_stability_p9973 | 0.003555 deg/s | 0.005 | ✔ PASS |
| time_to_0p01_deg | 42.7 s | — | — |
| wheel_momentum_peak | 0.002615 N m s | — | — |
| power_mean | 5.059 W | 2 | ✖ FAIL |

Events: 0 s nadir_fine; 1200 s FAULT injected: st_head_fail 2; 1200 s nadir_fine

## Imaging 3U, magnetorquers + fluid momentum rings (IDMAS)

### `fine_hold_fmr` — 3U imaging — fine nadir hold, coils + fluid momentum rings (IDMAS split)

| metric | value | required | verdict |
|---|---|---|---|
| ape_los_p9973 | 0.004904 deg | 0.01 | ✔ PASS |
| ape_los_max | 0.006288 deg | — | — |
| ape_3ax_p9973 | 0.006064 deg | — | — |
| ake_los_p9973 | 0.002585 deg | 0.005 | ✔ PASS |
| rate_stability_p9973 | 0.004962 deg/s | 0.005 | ✔ PASS |
| time_to_0p01_deg | 43.6 s | — | — |
| wheel_momentum_peak | 0.0002841 N m s | — | — |
| power_mean | 3.601 W | 2 | ✖ FAIL |

### `slew_fmr` — 3U imaging — 30 deg target slew in 60 s, coils + fluid momentum rings (IDMAS split)

| metric | value | required | verdict |
|---|---|---|---|
| settle_time_after_slew | 10.1 s | 20 | ✔ PASS |
| ape_los_on_target_p9973 | 0.006051 deg | 0.01 | ✔ PASS |
| wheel_momentum_peak | 0.0001057 N m s | — | — |

### `mission_fmr` — 3U imaging — tumble, detumble, fine hold: coils + fluid momentum rings (IDMAS split)

| metric | value | required | verdict |
|---|---|---|---|
| detumble_time | 61.44 min | 284 | ✔ PASS |
| ape_los_last_half_orbit_p9973 | 0.004848 deg | 0.01 | ✔ PASS |

Events: 0 s detumble; 3314 s nadir_fine

## Imaging 3U, magnetorquers + fluid rings + cold-gas RCS

### `fine_hold_fmr_rcs` — 3U imaging — fine nadir hold, coils + fluid rings + cold-gas RCS

| metric | value | required | verdict |
|---|---|---|---|
| ape_los_p9973 | 0.004904 deg | 0.01 | ✔ PASS |
| ape_los_max | 0.006288 deg | — | — |
| ape_3ax_p9973 | 0.006064 deg | — | — |
| ake_los_p9973 | 0.002585 deg | 0.005 | ✔ PASS |
| rate_stability_p9973 | 0.004962 deg/s | 0.005 | ✔ PASS |
| time_to_0p01_deg | 43.6 s | — | — |
| wheel_momentum_peak | 0.0002841 N m s | — | — |
| power_mean | 3.601 W | 2 | ✖ FAIL |

### `slew_fmr_rcs` — 3U imaging — 30 deg target slew in 60 s, coils + fluid rings + cold-gas RCS

| metric | value | required | verdict |
|---|---|---|---|
| settle_time_after_slew | 10.1 s | 20 | ✔ PASS |
| ape_los_on_target_p9973 | 0.006051 deg | 0.01 | ✔ PASS |
| wheel_momentum_peak | 0.0001057 N m s | — | — |

### `agile_slew_fmr_rcs` — 3U imaging — (fmr_rcs): 90 deg pitch in 15 s

| metric | value | required | verdict |
|---|---|---|---|
| settle_time_after_slew | 26.4 s | — | — |
| ape_los_on_target_p9973 | 0.006668 deg | 0.01 | ✔ PASS |
| wheel_momentum_peak | 0.001017 N m s | — | — |

### `mission_fmr_rcs` — 3U imaging — tumble, detumble, fine hold: coils + fluid rings + cold-gas RCS

| metric | value | required | verdict |
|---|---|---|---|
| detumble_time | 61.44 min | 284 | ✔ PASS |
| ape_los_last_half_orbit_p9973 | 0.004848 deg | 0.01 | ✔ PASS |

Events: 0 s detumble; 3314 s nadir_fine

## Imaging 3U, magnetorquers + reaction wheels + cold-gas RCS

### `fine_hold_rw_rcs` — 3U imaging — fine nadir hold, coils + reaction wheels + cold-gas RCS

| metric | value | required | verdict |
|---|---|---|---|
| ape_los_p9973 | 0.005186 deg | 0.01 | ✔ PASS |
| ape_los_max | 0.005999 deg | — | — |
| ape_3ax_p9973 | 0.00617 deg | — | — |
| ake_los_p9973 | 0.002607 deg | 0.005 | ✔ PASS |
| rate_stability_p9973 | 0.003648 deg/s | 0.005 | ✔ PASS |
| time_to_0p01_deg | 42.7 s | — | — |
| wheel_momentum_peak | 0.002821 N m s | — | — |
| power_mean | 5.037 W | 2 | ✖ FAIL |

### `slew_rw_rcs` — 3U imaging — 30 deg target slew in 60 s, coils + reaction wheels + cold-gas RCS

| metric | value | required | verdict |
|---|---|---|---|
| settle_time_after_slew | 8.9 s | 20 | ✔ PASS |
| ape_los_on_target_p9973 | 0.006262 deg | 0.01 | ✔ PASS |
| wheel_momentum_peak | 0.002795 N m s | — | — |

### `agile_slew_rw_rcs` — 3U imaging — (rw_rcs): 90 deg pitch in 15 s

| metric | value | required | verdict |
|---|---|---|---|
| settle_time_after_slew | 24.5 s | — | — |
| ape_los_on_target_p9973 | 0.006559 deg | 0.01 | ✔ PASS |
| wheel_momentum_peak | 0.003565 N m s | — | — |

### `mission_rw_rcs` — 3U imaging — tumble, detumble, fine hold: coils + reaction wheels + cold-gas RCS

| metric | value | required | verdict |
|---|---|---|---|
| detumble_time | 49.86 min | 284 | ✔ PASS |
| ape_los_last_half_orbit_p9973 | 0.00537 deg | 0.01 | ✔ PASS |

Events: 0 s detumble; 2914 s nadir_fine

## Imaging 3U, magnetorquers + 4 SGCMG

### `fine_hold_cmg` — 3U imaging — fine nadir hold, coils + 4 SGCMG pyramid

| metric | value | required | verdict |
|---|---|---|---|
| ape_los_p9973 | 0.004074 deg | 0.01 | ✔ PASS |
| ape_los_max | 0.0048 deg | — | — |
| ape_3ax_p9973 | 0.005819 deg | — | — |
| ake_los_p9973 | 0.002597 deg | 0.005 | ✔ PASS |
| rate_stability_p9973 | 0.002673 deg/s | 0.005 | ✔ PASS |
| time_to_0p01_deg | 42.5 s | — | — |
| wheel_momentum_peak | 0.004 N m s | — | — |
| power_mean | 5.11 W | 2 | ✖ FAIL |

### `slew_cmg` — 3U imaging — 30 deg target slew in 60 s, coils + 4 SGCMG pyramid

| metric | value | required | verdict |
|---|---|---|---|
| settle_time_after_slew | 5.7 s | 20 | ✔ PASS |
| ape_los_on_target_p9973 | 0.003666 deg | 0.01 | ✔ PASS |
| wheel_momentum_peak | 0.004 N m s | — | — |

### `agile_slew_cmg` — 3U imaging — (cmg): 90 deg pitch in 15 s

| metric | value | required | verdict |
|---|---|---|---|
| settle_time_after_slew | 24.5 s | — | — |
| ape_los_on_target_p9973 | 0.004502 deg | 0.01 | ✔ PASS |
| wheel_momentum_peak | 0.004 N m s | — | — |

### `mission_cmg` — 3U imaging — tumble, detumble, fine hold: coils + 4 SGCMG pyramid

| metric | value | required | verdict |
|---|---|---|---|
| detumble_time | 56.79 min | 284 | ✔ PASS |
| ape_los_last_half_orbit_p9973 | 0.004289 deg | 0.01 | ✔ PASS |

Events: 0 s detumble; 3315 s nadir_fine

### `fault_gimbal_cmg` — 3U imaging — one CMG gimbal sticks during fine hold

| metric | value | required | verdict |
|---|---|---|---|
| ape_los_p9973 | 0.00502 deg | 0.01 | ✔ PASS |
| ape_los_max | 0.005653 deg | — | — |
| ape_3ax_p9973 | 0.006683 deg | — | — |
| ake_los_p9973 | 0.002853 deg | 0.005 | ✔ PASS |
| rate_stability_p9973 | 0.003289 deg/s | 0.005 | ✔ PASS |
| time_to_0p01_deg | 42.5 s | — | — |
| wheel_momentum_peak | 0.004 N m s | — | — |
| power_mean | 5.11 W | 2 | ✖ FAIL |

Events: 0 s nadir_fine; 1200 s FAULT injected: gimbal_stuck 1; 1200 s nadir_fine

## Imaging 3U, magnetorquers + 4 VSCMG

### `fine_hold_vscmg` — 3U imaging — fine nadir hold, coils + 4 VSCMG pyramid

| metric | value | required | verdict |
|---|---|---|---|
| ape_los_p9973 | 0.006405 deg | 0.01 | ✔ PASS |
| ape_los_max | 0.008855 deg | — | — |
| ape_3ax_p9973 | 0.006849 deg | — | — |
| ake_los_p9973 | 0.00263 deg | 0.005 | ✔ PASS |
| rate_stability_p9973 | 0.004646 deg/s | 0.005 | ✔ PASS |
| time_to_0p01_deg | 40.9 s | — | — |
| wheel_momentum_peak | 0.004489 N m s | — | — |
| power_mean | 5.515 W | 2 | ✖ FAIL |

### `slew_vscmg` — 3U imaging — 30 deg target slew in 60 s, coils + 4 VSCMG pyramid

| metric | value | required | verdict |
|---|---|---|---|
| settle_time_after_slew | 9 s | 20 | ✔ PASS |
| ape_los_on_target_p9973 | 0.007254 deg | 0.01 | ✔ PASS |
| wheel_momentum_peak | 0.004124 N m s | — | — |

### `agile_slew_vscmg` — 3U imaging — (vscmg): 90 deg pitch in 15 s

| metric | value | required | verdict |
|---|---|---|---|
| settle_time_after_slew | 41.3 s | — | — |
| ape_los_on_target_p9973 | 0.009575 deg | 0.01 | ✔ PASS |
| wheel_momentum_peak | 0.008099 N m s | — | — |

### `mission_vscmg` — 3U imaging — tumble, detumble, fine hold: coils + 4 VSCMG pyramid

| metric | value | required | verdict |
|---|---|---|---|
| detumble_time | 56.89 min | 284 | ✔ PASS |
| ape_los_last_half_orbit_p9973 | 0.006873 deg | 0.01 | ✔ PASS |

Events: 0 s detumble; 3315 s nadir_fine

## Monte Carlo and edge-case campaigns

### `mc_detumble_ais` — 24 runs of `detumble_ais`

| metric | mean | std | ensemble percentile | required | runs passing | verdict |
|---|---|---|---|---|---|---|
| detumble_time | 56.64 | 12.5 | p95: 74.19 min | 284 | 100% | ✔ PASS |
| power_mean | 1.527 | 0.00791 | p95: 1.54 W | 0.5 | 0% | ✖ FAIL |
| power_peak | 1.861 | 0.139 | p95: 2.096 W | 1.5 | 0% | ✖ FAIL |

### `mc_nadir_ais` — 24 runs of `nadir_hold_ais`

| metric | mean | std | ensemble percentile | required | runs passing | verdict |
|---|---|---|---|---|---|---|
| ape_3axis_last_orbit | 179.1 | 4.33 | p95: 180 deg | — | — | — |
| ape_los_last_orbit | 101.7 | 47 | p95: 158.8 deg | 10 | 0% | ✖ FAIL |
| ake_los_last_orbit | 1.449 | 0.675 | p95: 2.521 deg | 5 | 100% | ✔ PASS |
| power_mean | 1.512 | 0.00285 | p95: 1.516 W | 0.5 | 0% | ✖ FAIL |
| power_margin_last_orbit | 6.825 | 1.64 | p95: 8.694 W | 0 | 0% | ✖ FAIL |
| battery_dod_last_orbit | 0.7279 | 1.45 | p95: 4.548 % | — | — | — |

### `edge_nadir_ais` — 17 runs of `nadir_hold_ais`

| metric | mean | std | ensemble percentile | required | runs passing | verdict |
|---|---|---|---|---|---|---|
| ape_3axis_last_orbit | 166.9 | 22.8 | p95: 180 deg | — | — | — |
| ape_los_last_orbit | 21.47 | 23.7 | p95: 80.15 deg | 10 | 35% | ✖ FAIL |
| ake_los_last_orbit | 1.929 | 0.975 | p95: 3.836 deg | 5 | 100% | ✔ PASS |
| power_mean | 1.511 | 0.00136 | p95: 1.513 W | 0.5 | 0% | ✖ FAIL |
| power_margin_last_orbit | 5.056 | 0.427 | p95: 5.651 W | 0 | 0% | ✖ FAIL |
| battery_dod_last_orbit | 4.719 | 0.018 | p95: 4.789 % | — | — | — |

### `mc_fine_img` — 24 runs of `fine_hold_img`

| metric | mean | std | ensemble percentile | required | runs passing | verdict |
|---|---|---|---|---|---|---|
| ape_los_p9973 | 0.006315 | 0.00593 | p99.73: 0.03413 deg | 0.01 | 96% | ✖ FAIL |
| ape_los_max | 0.007944 | 0.00601 | p95: 0.007655 deg | — | — | — |
| ape_3ax_p9973 | 0.007692 | 0.00801 | p95: 0.006869 deg | — | — | — |
| ake_los_p9973 | 0.004011 | 0.00642 | p99.73: 0.03409 deg | 0.005 | 96% | ✖ FAIL |
| rate_stability_p9973 | 0.003774 | 0.000103 | p99.73: 0.004059 deg/s | 0.005 | 100% | ✔ PASS |
| time_to_0p01_deg | 39.46 | 7.96 | p95: 46.6 s | — | — | — |
| wheel_momentum_peak | 0.002856 | 0.000266 | p95: 0.003284 N m s | — | — | — |
| power_mean | 5.066 | 0.00529 | p95: 5.073 W | 2 | 0% | ✖ FAIL |
| rpe_los_10s_p9973 | 0.003945 | 0.000184 | p95: 0.004132 deg | — | — | — |
| pde_los_10s_60s_max | 0.006281 | 0.00669 | p95: 0.005834 deg | — | — | — |
| rke_los_10s_p9973 | 0.001811 | 0.000216 | p95: 0.001842 deg | — | — | — |
| power_margin_last_orbit | -1.586 | 0.405 | p95: -1.053 W | 0 | 100% | ✔ PASS |
| battery_dod_last_orbit | 22.75 | 4.02 | p95: 28.87 % | — | — | — |

### `edge_fine_img` — 15 runs of `fine_hold_img`

| metric | mean | std | ensemble percentile | required | runs passing | verdict |
|---|---|---|---|---|---|---|
| ape_los_p9973 | 0.00502 | 0.000217 | p99.73: 0.005484 deg | 0.01 | 100% | ✔ PASS |
| ape_los_max | 0.006861 | 0.000432 | p95: 0.007542 deg | — | — | — |
| ape_3ax_p9973 | 0.006061 | 0.00039 | p95: 0.006909 deg | — | — | — |
| ake_los_p9973 | 0.002652 | 0.000441 | p99.73: 0.004083 deg | 0.005 | 100% | ✔ PASS |
| rate_stability_p9973 | 0.00372 | 5.97e-05 | p99.73: 0.003804 deg/s | 0.005 | 100% | ✔ PASS |
| time_to_0p01_deg | 43.5 | 4.66 | p95: 46.5 s | — | — | — |
| wheel_momentum_peak | 0.002747 | 0.000175 | p95: 0.003189 N m s | — | — | — |
| power_mean | 5.065 | 0.0011 | p95: 5.069 W | 2 | 0% | ✖ FAIL |
| rpe_los_10s_p9973 | 0.003911 | 0.000122 | p95: 0.004115 deg | — | — | — |
| pde_los_10s_60s_max | 0.00485 | 0.000571 | p95: 0.005936 deg | — | — | — |
| rke_los_10s_p9973 | 0.001753 | 3.1e-05 | p95: 0.0018 deg | — | — | — |
| power_margin_last_orbit | -1.866 | 0.000753 | p95: -1.865 W | 0 | 100% | ✔ PASS |
| battery_dod_last_orbit | 26.78 | 0.00869 | p95: 26.8 % | — | — | — |

### `mc_slew_img` — 40 runs of `slew_img`

| metric | mean | std | ensemble percentile | required | runs passing | verdict |
|---|---|---|---|---|---|---|
| settle_time_after_slew | 6.88 | 1.54 | p95: 9.2 s | 20 | 100% | ✔ PASS |
| ape_los_on_target_p9973 | 0.006286 | 0.00116 | p99.73: 0.01145 deg | 0.01 | 98% | ✖ FAIL |
| wheel_momentum_peak | 0.002567 | 0.000142 | p95: 0.002755 N m s | — | — | — |

### `mc_slew_cmg` — 20 runs of `slew_cmg`

| metric | mean | std | ensemble percentile | required | runs passing | verdict |
|---|---|---|---|---|---|---|
| settle_time_after_slew | 6.42 | 0.984 | p95: 8 s | 20 | 100% | ✔ PASS |
| ape_los_on_target_p9973 | 0.003431 | 0.000386 | p99.73: 0.004268 deg | 0.01 | 100% | ✔ PASS |
| wheel_momentum_peak | 0.004 | 0 | p95: 0.004 N m s | — | — | — |

### `mc_agile_rw_rcs` — 20 runs of `agile_slew_rw_rcs`

| metric | mean | std | ensemble percentile | required | runs passing | verdict |
|---|---|---|---|---|---|---|
| settle_time_after_slew | 29.57 | 24.8 | p95: 25.3 s | — | — | — |
| ape_los_on_target_p9973 | 0.01159 | 0.0235 | p99.73: 0.1114 deg | 0.01 | 95% | ✖ FAIL |
| wheel_momentum_peak | 0.003548 | 5.21e-05 | p95: 0.00362 N m s | — | — | — |

