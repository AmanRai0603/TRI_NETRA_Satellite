# TRI-NETRA ADCS — SILS results

**Owner: Agastya.** Copyright (c) 2026 Agastya. All rights reserved.

Produced by `matlab_sils` (GNU Octave 8.4) with the Precision Orbit Propagator stepped inside the attitude loop.
Figures: `results/index.html` (open in a browser) and `results/figures/`. Single runs are the nominal case, seed 1.

## AIS 3U, magnetorquers only (10°, SSO dawn–dusk)

### `detumble_ais` — 3U AIS — B-dot detumble from a 10 deg/s tumble (coils only)

| metric | value | required | verdict |
|---|---|---|---|
| detumble_time | 58.26 min | 284 | ✔ PASS |
| power_mean | 0.02455 W | 0.5 | ✔ PASS |
| power_peak | 0.3639 W | 1.5 | ✔ PASS |

### `nadir_hold_ais` — 3U AIS — nadir hold with magnetorquers only (10 deg requirement)

| metric | value | required | verdict |
|---|---|---|---|
| ape_3axis_last_orbit | 49.93 deg | — | — |
| ape_los_last_orbit | 7.248 deg | 10 | ✔ PASS |
| ake_los_last_orbit | 3.864 deg | 5 | ✔ PASS |
| power_mean | 0.0108 W | 0.5 | ✔ PASS |

### `nadir_hold_ais_css` — 3U AIS — nadir hold with coarse cosine sun sensors (albedo)

| metric | value | required | verdict |
|---|---|---|---|
| ape_3axis_last_orbit | 179.9 deg | — | — |
| ape_los_last_orbit | 36.56 deg | 10 | ✖ FAIL |
| ake_los_last_orbit | 41.56 deg | 5 | ✖ FAIL |
| power_mean | 0.01071 W | 0.5 | ✔ PASS |

### `sun_spin_ais` — 3U AIS — coils-only Sun acquisition: detumble, spin-up, Sun-pointing spin

| metric | value | required | verdict |
|---|---|---|---|
| sun_spin_entry | 40.44 min | 95 | ✔ PASS |
| sun_angle_last_orbit | 6.503 deg | 20 | ✔ PASS |
| spin_rate_error_last_orbit | 0.04568 deg/s | 0.5 | ✔ PASS |
| sun_spin_share_last_orbit | 78.81 % | 90 | ✖ FAIL |
| power_mean | 0.2062 W | 0.5 | ✔ PASS |

Events: 0 s detumble; 855 s spinup; 2426 s sun_spin; 2542 s spinup; 3695 s sun_spin; 3744 s spinup; 4073 s sun_spin; 6903 s spinup; 7920 s sun_spin; 12391 s spinup; 13607 s sun_spin

### `mission_ais` — 3U AIS — integrated chain: detumble then nadir hold (auto mode switch)

| metric | value | required | verdict |
|---|---|---|---|
| detumble_time | 58.26 min | 284 | ✔ PASS |
| ape_los_last_orbit | 9.114 deg | 10 | ✔ PASS |
| ape_3axis_last_orbit | 77.13 deg | — | — |
| ake_los_last_orbit | 4.046 deg | 5 | ✔ PASS |

Events: 0 s detumble; 5011 s nadir_mtq

### `fault_coil_ais` — 3U AIS — X coil fails during detumble

| metric | value | required | verdict |
|---|---|---|---|
| detumble_time | 128.3 min | 284 | ✔ PASS |
| power_mean | 0.02199 W | 0.5 | ✔ PASS |
| power_peak | 0.3163 W | 1.5 | ✔ PASS |

Events: 0 s detumble; 60 s FAULT injected: coil_fail 1

### `fault_gyro_ais` — 3U AIS — gyro bias step of 0.1 deg/s during nadir hold

| metric | value | required | verdict |
|---|---|---|---|
| ape_3axis_last_orbit | 180 deg | — | — |
| ape_los_last_orbit | 56.79 deg | 10 | ✖ FAIL |
| ake_los_last_orbit | 42.59 deg | 5 | ✖ FAIL |
| power_mean | 0.01077 W | 0.5 | ✔ PASS |

Events: 0 s nadir_mtq; 6000 s FAULT injected: gyro_bias_step 0

## Imaging 3U, magnetorquers + reaction wheels (0.01°, SSO 10:00)

### `detumble_img` — 3U imaging — B-dot detumble from a 10 deg/s tumble

| metric | value | required | verdict |
|---|---|---|---|
| detumble_time | 21.98 min | 284 | ✔ PASS |

### `fine_hold_img` — 3U imaging — fine nadir hold, reaction wheels + star tracker (0.01 deg)

| metric | value | required | verdict |
|---|---|---|---|
| ape_los_p9973 | 0.009166 deg | 0.01 | ✔ PASS |
| ape_los_max | 0.01092 deg | — | — |
| ape_3ax_p9973 | 0.009595 deg | — | — |
| ake_los_p9973 | 0.003333 deg | 0.005 | ✔ PASS |
| rate_stability_p9973 | 0.00367 deg/s | 0.005 | ✔ PASS |
| time_to_0p01_deg | 83.5 s | — | — |
| wheel_momentum_peak | 0.003257 N m s | — | — |
| power_mean | 1.557 W | 2 | ✔ PASS |

### `slew_img` — 3U imaging — 30 deg target slew in 60 s and settle

| metric | value | required | verdict |
|---|---|---|---|
| settle_time_after_slew | 12.6 s | 20 | ✔ PASS |
| ape_los_on_target_p9973 | 0.009894 deg | 0.01 | ✔ PASS |
| wheel_momentum_peak | 0.003338 N m s | — | — |

### `agile_slew_img` — 3U imaging — (img): 90 deg pitch in 15 s

| metric | value | required | verdict |
|---|---|---|---|
| settle_time_after_slew | 51.8 s | — | — |
| ape_los_on_target_p9973 | 0.00988 deg | 0.01 | ✔ PASS |
| wheel_momentum_peak | 0.00692 N m s | — | — |

### `mission_img` — 3U imaging — integrated chain: detumble then fine nadir hold

| metric | value | required | verdict |
|---|---|---|---|
| detumble_time | 23.68 min | 284 | ✔ PASS |
| ape_los_last_half_orbit_p9973 | 0.007627 deg | 0.01 | ✔ PASS |

Events: 0 s detumble; 1378 s nadir_fine

### `fault_wheel_img` — 3U imaging — reaction wheel 1 fails during fine hold (FDIR + coils)

| metric | value | required | verdict |
|---|---|---|---|
| ape_los_p9973 | 15.83 deg | 0.01 | ✖ FAIL |
| ape_los_max | 15.87 deg | — | — |
| ape_3ax_p9973 | 15.83 deg | — | — |
| ake_los_p9973 | 0.003649 deg | 0.005 | ✔ PASS |
| rate_stability_p9973 | 0.3827 deg/s | 0.005 | ✖ FAIL |
| time_to_0p01_deg | 83.5 s | — | — |
| wheel_momentum_peak | 0.003675 N m s | — | — |
| power_mean | 1.191 W | 2 | ✔ PASS |

Events: 0 s nadir_fine; 1200 s FAULT injected: rotor_fail 1; 1212 s FDIR: rotor 1 isolated

### `fault_st_img` — 3U imaging — star-tracker head 2 fails during fine hold

| metric | value | required | verdict |
|---|---|---|---|
| ape_los_p9973 | 0.1843 deg | 0.01 | ✖ FAIL |
| ape_los_max | 0.1892 deg | — | — |
| ape_3ax_p9973 | 0.1858 deg | — | — |
| ake_los_p9973 | 0.1847 deg | 0.005 | ✖ FAIL |
| rate_stability_p9973 | 0.004082 deg/s | 0.005 | ✔ PASS |
| time_to_0p01_deg | 83.5 s | — | — |
| wheel_momentum_peak | 0.003257 N m s | — | — |
| power_mean | 1.557 W | 2 | ✔ PASS |

Events: 0 s nadir_fine; 1200 s FAULT injected: st_head_fail 2

## Imaging 3U, magnetorquers + fluid momentum rings (IDMAS)

### `fine_hold_fmr` — 3U imaging — fine nadir hold, coils + fluid momentum rings (IDMAS split)

| metric | value | required | verdict |
|---|---|---|---|
| ape_los_p9973 | 0.006033 deg | 0.01 | ✔ PASS |
| ape_los_max | 0.007836 deg | — | — |
| ape_3ax_p9973 | 0.007028 deg | — | — |
| ake_los_p9973 | 0.003321 deg | 0.005 | ✔ PASS |
| rate_stability_p9973 | 0.004654 deg/s | 0.005 | ✔ PASS |
| time_to_0p01_deg | 84.3 s | — | — |
| wheel_momentum_peak | 0.0001624 N m s | — | — |
| power_mean | 4.348 W | 2 | ✖ FAIL |

### `slew_fmr` — 3U imaging — 30 deg target slew in 60 s, coils + fluid momentum rings (IDMAS split)

| metric | value | required | verdict |
|---|---|---|---|
| settle_time_after_slew | 14 s | 20 | ✔ PASS |
| ape_los_on_target_p9973 | 0.008053 deg | 0.01 | ✔ PASS |
| wheel_momentum_peak | 0.000112 N m s | — | — |

### `mission_fmr` — 3U imaging — tumble, detumble, fine hold: coils + fluid momentum rings (IDMAS split)

| metric | value | required | verdict |
|---|---|---|---|
| detumble_time | 24.48 min | 284 | ✔ PASS |
| ape_los_last_half_orbit_p9973 | 0.006388 deg | 0.01 | ✔ PASS |

Events: 0 s detumble; 1411 s nadir_fine

## Imaging 3U, magnetorquers + fluid rings + cold-gas RCS

### `fine_hold_fmr_rcs` — 3U imaging — fine nadir hold, coils + fluid rings + cold-gas RCS

| metric | value | required | verdict |
|---|---|---|---|
| ape_los_p9973 | 0.00616 deg | 0.01 | ✔ PASS |
| ape_los_max | 0.006973 deg | — | — |
| ape_3ax_p9973 | 0.006878 deg | — | — |
| ake_los_p9973 | 0.00348 deg | 0.005 | ✔ PASS |
| rate_stability_p9973 | 0.004643 deg/s | 0.005 | ✔ PASS |
| time_to_0p01_deg | 88.2 s | — | — |
| wheel_momentum_peak | 0.000163 N m s | — | — |
| power_mean | 4.367 W | 2 | ✖ FAIL |

### `slew_fmr_rcs` — 3U imaging — 30 deg target slew in 60 s, coils + fluid rings + cold-gas RCS

| metric | value | required | verdict |
|---|---|---|---|
| settle_time_after_slew | 12.9 s | 20 | ✔ PASS |
| ape_los_on_target_p9973 | 0.007078 deg | 0.01 | ✔ PASS |
| wheel_momentum_peak | 0.0001118 N m s | — | — |

### `agile_slew_fmr_rcs` — 3U imaging — (fmr_rcs): 90 deg pitch in 15 s

| metric | value | required | verdict |
|---|---|---|---|
| settle_time_after_slew | 39.9 s | — | — |
| ape_los_on_target_p9973 | 0.007225 deg | 0.01 | ✔ PASS |
| wheel_momentum_peak | 0.0009452 N m s | — | — |

### `mission_fmr_rcs` — 3U imaging — tumble, detumble, fine hold: coils + fluid rings + cold-gas RCS

| metric | value | required | verdict |
|---|---|---|---|
| detumble_time | 64.12 min | 284 | ✔ PASS |
| ape_los_last_half_orbit_p9973 | 0.006253 deg | 0.01 | ✔ PASS |

Events: 0 s detumble; 3793 s nadir_fine

## Imaging 3U, magnetorquers + reaction wheels + cold-gas RCS

### `fine_hold_rw_rcs` — 3U imaging — fine nadir hold, coils + reaction wheels + cold-gas RCS

| metric | value | required | verdict |
|---|---|---|---|
| ape_los_p9973 | 0.008375 deg | 0.01 | ✔ PASS |
| ape_los_max | 0.009926 deg | — | — |
| ape_3ax_p9973 | 0.008488 deg | — | — |
| ake_los_p9973 | 0.003439 deg | 0.005 | ✔ PASS |
| rate_stability_p9973 | 0.003604 deg/s | 0.005 | ✔ PASS |
| time_to_0p01_deg | 87.2 s | — | — |
| wheel_momentum_peak | 0.002803 N m s | — | — |
| power_mean | 1.51 W | 2 | ✔ PASS |

### `slew_rw_rcs` — 3U imaging — 30 deg target slew in 60 s, coils + reaction wheels + cold-gas RCS

| metric | value | required | verdict |
|---|---|---|---|
| settle_time_after_slew | 11.2 s | 20 | ✔ PASS |
| ape_los_on_target_p9973 | 0.007829 deg | 0.01 | ✔ PASS |
| wheel_momentum_peak | 0.003419 N m s | — | — |

### `agile_slew_rw_rcs` — 3U imaging — (rw_rcs): 90 deg pitch in 15 s

| metric | value | required | verdict |
|---|---|---|---|
| settle_time_after_slew | 39.5 s | — | — |
| ape_los_on_target_p9973 | 0.008114 deg | 0.01 | ✔ PASS |
| wheel_momentum_peak | 0.003363 N m s | — | — |

### `mission_rw_rcs` — 3U imaging — tumble, detumble, fine hold: coils + reaction wheels + cold-gas RCS

| metric | value | required | verdict |
|---|---|---|---|
| detumble_time | 24.26 min | 284 | ✔ PASS |
| ape_los_last_half_orbit_p9973 | 0.007673 deg | 0.01 | ✔ PASS |

Events: 0 s detumble; 1408 s nadir_fine

## Imaging 3U, magnetorquers + 4 SGCMG

### `fine_hold_cmg` — 3U imaging — fine nadir hold, coils + 4 SGCMG pyramid

| metric | value | required | verdict |
|---|---|---|---|
| ape_los_p9973 | 0.005523 deg | 0.01 | ✔ PASS |
| ape_los_max | 0.006414 deg | — | — |
| ape_3ax_p9973 | 0.006146 deg | — | — |
| ake_los_p9973 | 0.00341 deg | 0.005 | ✔ PASS |
| rate_stability_p9973 | 0.002425 deg/s | 0.005 | ✔ PASS |
| time_to_0p01_deg | 83.5 s | — | — |
| wheel_momentum_peak | 0.004 N m s | — | — |
| power_mean | 1.61 W | 2 | ✔ PASS |

### `slew_cmg` — 3U imaging — 30 deg target slew in 60 s, coils + 4 SGCMG pyramid

| metric | value | required | verdict |
|---|---|---|---|
| settle_time_after_slew | 11.4 s | 20 | ✔ PASS |
| ape_los_on_target_p9973 | 0.004542 deg | 0.01 | ✔ PASS |
| wheel_momentum_peak | 0.004 N m s | — | — |

### `agile_slew_cmg` — 3U imaging — (cmg): 90 deg pitch in 15 s

| metric | value | required | verdict |
|---|---|---|---|
| settle_time_after_slew | 38.7 s | — | — |
| ape_los_on_target_p9973 | 0.005492 deg | 0.01 | ✔ PASS |
| wheel_momentum_peak | 0.004 N m s | — | — |

### `mission_cmg` — 3U imaging — tumble, detumble, fine hold: coils + 4 SGCMG pyramid

| metric | value | required | verdict |
|---|---|---|---|
| detumble_time | 59.46 min | 284 | ✔ PASS |
| ape_los_last_half_orbit_p9973 | 0.005659 deg | 0.01 | ✔ PASS |

Events: 0 s detumble; 3516 s nadir_fine

### `fault_gimbal_cmg` — 3U imaging — one CMG gimbal sticks during fine hold

| metric | value | required | verdict |
|---|---|---|---|
| ape_los_p9973 | 0.006362 deg | 0.01 | ✔ PASS |
| ape_los_max | 0.007352 deg | — | — |
| ape_3ax_p9973 | 0.006689 deg | — | — |
| ake_los_p9973 | 0.00343 deg | 0.005 | ✔ PASS |
| rate_stability_p9973 | 0.002951 deg/s | 0.005 | ✔ PASS |
| time_to_0p01_deg | 83.5 s | — | — |
| wheel_momentum_peak | 0.004 N m s | — | — |
| power_mean | 1.61 W | 2 | ✔ PASS |

Events: 0 s nadir_fine; 1200 s FAULT injected: gimbal_stuck 1

## Imaging 3U, magnetorquers + 4 VSCMG

### `fine_hold_vscmg` — 3U imaging — fine nadir hold, coils + 4 VSCMG pyramid

| metric | value | required | verdict |
|---|---|---|---|
| ape_los_p9973 | 0.009129 deg | 0.01 | ✔ PASS |
| ape_los_max | 0.01036 deg | — | — |
| ape_3ax_p9973 | 0.009208 deg | — | — |
| ake_los_p9973 | 0.003558 deg | 0.005 | ✔ PASS |
| rate_stability_p9973 | 0.004557 deg/s | 0.005 | ✔ PASS |
| time_to_0p01_deg | 89.3 s | — | — |
| wheel_momentum_peak | 0.004201 N m s | — | — |
| power_mean | 2.015 W | 2 | ✖ FAIL |

### `slew_vscmg` — 3U imaging — 30 deg target slew in 60 s, coils + 4 VSCMG pyramid

| metric | value | required | verdict |
|---|---|---|---|
| settle_time_after_slew | 14.4 s | 20 | ✔ PASS |
| ape_los_on_target_p9973 | 0.01027 deg | 0.01 | ✖ FAIL |
| wheel_momentum_peak | 0.004051 N m s | — | — |

### `agile_slew_vscmg` — 3U imaging — (vscmg): 90 deg pitch in 15 s

| metric | value | required | verdict |
|---|---|---|---|
| settle_time_after_slew | 51.6 s | — | — |
| ape_los_on_target_p9973 | 0.01007 deg | 0.01 | ✖ FAIL |
| wheel_momentum_peak | 0.008087 N m s | — | — |

### `mission_vscmg` — 3U imaging — tumble, detumble, fine hold: coils + 4 VSCMG pyramid

| metric | value | required | verdict |
|---|---|---|---|
| detumble_time | 59.54 min | 284 | ✔ PASS |
| ape_los_last_half_orbit_p9973 | 0.01111 deg | 0.01 | ✖ FAIL |

Events: 0 s detumble; 3526 s nadir_fine

## Monte Carlo and edge-case campaigns

### `mc_detumble_ais` — 24 runs of `detumble_ais`

| metric | mean | std | ensemble percentile | required | runs passing | verdict |
|---|---|---|---|---|---|---|
| detumble_time | 48.5 | 12 | p95: 70.66 min | 284 | 100% | ✔ PASS |
| power_mean | 0.02267 | 0.00701 | p95: 0.03338 W | 0.5 | 100% | ✔ PASS |
| power_peak | 0.3902 | 0.169 | p95: 0.63 W | 1.5 | 100% | ✔ PASS |

### `mc_nadir_ais` — 24 runs of `nadir_hold_ais`

| metric | mean | std | ensemble percentile | required | runs passing | verdict |
|---|---|---|---|---|---|---|
| ape_3axis_last_orbit | 177.8 | 7.4 | p95: 180 deg | — | — | — |
| ape_los_last_orbit | 89.69 | 38.5 | p95: 163.6 deg | 10 | 0% | ✖ FAIL |
| ake_los_last_orbit | 3.3 | 1.73 | p95: 6.077 deg | 5 | 75% | ✖ FAIL |
| power_mean | 0.01145 | 0.0025 | p95: 0.01449 W | 0.5 | 100% | ✔ PASS |

### `edge_nadir_ais` — 17 runs of `nadir_hold_ais`

| metric | mean | std | ensemble percentile | required | runs passing | verdict |
|---|---|---|---|---|---|---|
| ape_3axis_last_orbit | 112.2 | 50 | p95: 180 deg | — | — | — |
| ape_los_last_orbit | 24.91 | 40.4 | p95: 155.1 deg | 10 | 59% | ✖ FAIL |
| ake_los_last_orbit | 3.712 | 2.58 | p95: 11.79 deg | 5 | 82% | ✖ FAIL |
| power_mean | 0.01057 | 0.00121 | p95: 0.01289 W | 0.5 | 100% | ✔ PASS |

### `mc_fine_img` — 24 runs of `fine_hold_img`

| metric | mean | std | ensemble percentile | required | runs passing | verdict |
|---|---|---|---|---|---|---|
| ape_los_p9973 | 0.01315 | 0.00434 | p99.73: 0.02007 deg | 0.01 | 29% | ✖ FAIL |
| ape_los_max | 0.01599 | 0.0044 | p95: 0.02173 deg | — | — | — |
| ape_3ax_p9973 | 0.01338 | 0.00425 | p95: 0.01957 deg | — | — | — |
| ake_los_p9973 | 0.002894 | 0.000605 | p99.73: 0.00408 deg | 0.005 | 100% | ✔ PASS |
| rate_stability_p9973 | 0.003738 | 9.57e-05 | p99.73: 0.003962 deg/s | 0.005 | 100% | ✔ PASS |
| time_to_0p01_deg | 69.32 | 11.1 | p95: 84.8 s | — | — | — |
| wheel_momentum_peak | 0.003075 | 0.000218 | p95: 0.0034 N m s | — | — | — |
| power_mean | 1.558 | 0.0021 | p95: 1.561 W | 2 | 100% | ✔ PASS |

### `edge_fine_img` — 15 runs of `fine_hold_img`

| metric | mean | std | ensemble percentile | required | runs passing | verdict |
|---|---|---|---|---|---|---|
| ape_los_p9973 | 0.01236 | 0.00475 | p99.73: 0.02038 deg | 0.01 | 40% | ✖ FAIL |
| ape_los_max | 0.01488 | 0.00506 | p95: 0.02368 deg | — | — | — |
| ape_3ax_p9973 | 0.01258 | 0.00465 | p95: 0.02055 deg | — | — | — |
| ake_los_p9973 | 0.003307 | 0.000418 | p99.73: 0.004166 deg | 0.005 | 100% | ✔ PASS |
| rate_stability_p9973 | 0.003689 | 0.000116 | p99.73: 0.003893 deg/s | 0.005 | 100% | ✔ PASS |
| time_to_0p01_deg | 71.66 | 9.51 | p95: 88.7 s | — | — | — |
| wheel_momentum_peak | 0.003344 | 0.00013 | p95: 0.003651 N m s | — | — | — |
| power_mean | 1.558 | 0.00216 | p95: 1.564 W | 2 | 100% | ✔ PASS |

### `mc_slew_img` — 40 runs of `slew_img`

| metric | mean | std | ensemble percentile | required | runs passing | verdict |
|---|---|---|---|---|---|---|
| settle_time_after_slew | 13.03 | 0.959 | p95: 14.4 s | 20 | 100% | ✔ PASS |
| ape_los_on_target_p9973 | 0.00815 | 0.00115 | p99.73: 0.01114 deg | 0.01 | 92% | ✖ FAIL |
| wheel_momentum_peak | 0.003475 | 0.000273 | p95: 0.00377 N m s | — | — | — |

### `mc_slew_cmg` — 20 runs of `slew_cmg`

| metric | mean | std | ensemble percentile | required | runs passing | verdict |
|---|---|---|---|---|---|---|
| settle_time_after_slew | 13.14 | 0.46 | p95: 13.8 s | 20 | 100% | ✔ PASS |
| ape_los_on_target_p9973 | 0.003931 | 0.000635 | p99.73: 0.005957 deg | 0.01 | 100% | ✔ PASS |
| wheel_momentum_peak | 0.004 | 0 | p95: 0.004 N m s | — | — | — |

### `mc_agile_rw_rcs` — 20 runs of `agile_slew_rw_rcs`

| metric | mean | std | ensemble percentile | required | runs passing | verdict |
|---|---|---|---|---|---|---|
| settle_time_after_slew | 38.66 | 2.7 | p95: 42.4 s | — | — | — |
| ape_los_on_target_p9973 | 0.008337 | 0.000712 | p99.73: 0.00977 deg | 0.01 | 100% | ✔ PASS |
| wheel_momentum_peak | 0.003391 | 4.68e-05 | p95: 0.003461 N m s | — | — | — |

