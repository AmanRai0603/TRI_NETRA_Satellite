# Requirements traceability

> **Answer first.** 19 stated requirements across 2 cases; 11 met, 8 not met, 0 not yet flown, 0 with nothing checking them. Of 173 shipped metrics, 107 judge and 66 report with a stated reason.

> **Kind:** generated (`python3 tools/trace.py`) · **Source:** the cases, the scenarios, the latest stored runs, the design selection

A requirement marked † is UNCONFIRMED in its case (a plan value a person has not confirmed).

## ais_3u

| requirement | value | checked by | latest | status |
|---|---|---|---|---|
| Absolute pointing error (APE) (`req.ape`) | 10 Degree | flight `fault_gyro_ais` / ape_los_last_orbit<br>flight `mission_ais` / ape_los_last_orbit<br>flight `nadir_hold_ais` / ape_los_last_orbit<br>flight `nadir_hold_ais_css` / ape_los_last_orbit<br>design loop: nadir_pointing / ape_los_p9973 | 52.78 ❌<br>8.935 ✅<br>12.64 ❌<br>39.44 ❌<br>0.1471 ✅ | not met (3 of 5) |
| Absolute knowledge error (AKE) (`req.ake`) † | 5 Degree | flight `fault_gyro_ais` / ake_los_last_orbit<br>flight `mission_ais` / ake_los_last_orbit<br>flight `nadir_hold_ais` / ake_los_last_orbit<br>flight `nadir_hold_ais_css` / ake_los_last_orbit<br>design loop: nadir_pointing / ake_los_p9973 | 41.92 ❌<br>1.188 ✅<br>5.491 ❌<br>39.22 ❌<br>0.1461 ✅ | not met (3 of 5) |
| Detumble time (`req.detumble`) † | 284 Minute | flight `detumble_ais` / detumble_time<br>flight `detumble_ais_bangbang` / detumble_time<br>flight `detumble_ais_mag` / detumble_time<br>flight `fault_coil_ais` / detumble_time<br>flight `mission_ais` / detumble_time<br>design loop: detumble / detumble_time | 37.79 ✅<br>54.02 ✅<br>41.16 ✅<br>82.17 ✅<br>37.79 ✅<br>41.01 ✅ | met |
| Sun-acquisition time in safe mode (`req.sunacq`) † | 95 Minute | flight `sun_spin_ais` / sun_spin_entry<br>design loop: sun_acquisition / sun_acquisition_time | 142.4 ❌<br>2.292 ✅ | not met (1 of 2) |
| ADCS mass (`req.mass`) | 1.6 Kilogram | design: selected `mtq_fmr` budget | 1.446 ✅ | met |
| ADCS orbit-average power (`req.pavg`) † | 0.5 Watt | flight `detumble_ais` / power_mean<br>flight `detumble_ais_bangbang` / power_mean<br>flight `detumble_ais_mag` / power_mean<br>flight `fault_coil_ais` / power_mean<br>flight `fault_gyro_ais` / power_mean<br>flight `nadir_hold_ais` / power_mean<br>flight `nadir_hold_ais_css` / power_mean<br>flight `sun_spin_ais` / power_mean<br>design loop: nadir_pointing / power_mean<br>design loop: sun_acquisition / power_mean<br>design loop: sun_referencing / power_mean | 0.0217 ✅<br>0.4366 ✅<br>0.07839 ✅<br>0.01821 ✅<br>0.01056 ✅<br>0.01054 ✅<br>0.01064 ✅<br>0.45 ✅<br>0.4695 ✅<br>0.2357 ✅<br>0.4296 ✅ | met |
| ADCS peak power (`req.ppk`) † | 1.5 Watt | flight `detumble_ais` / power_peak<br>flight `detumble_ais_bangbang` / power_peak<br>flight `detumble_ais_mag` / power_peak<br>flight `fault_coil_ais` / power_peak<br>design loop: detumble / power_peak | 0.5502 ✅<br>0.717 ✅<br>0.6377 ✅<br>0.5502 ✅<br>0.4776 ✅ | met |
| ADCS volume (`req.vol`) | 1 Litre | design: selected `mtq_fmr` budget | 0.5371 ✅ | met |

Not stated by the case (nothing to check): `req.rpe`, `req.pde`, `req.rks`, `req.rke`, `req.slew`, `req.settle`, `req.spo`, `req.track`, `req.wmax`, `req.faults`, `req.recover`, `req.hsat`, `req.dump`, `req.prop`.

## ais_img_3u

| requirement | value | checked by | latest | status |
|---|---|---|---|---|
| Absolute pointing error (APE) (`req.ape`) | 0.01 Degree | flight `agile_slew_cmg` / ape_los_on_target_p9973<br>flight `agile_slew_fmr_rcs` / ape_los_on_target_p9973<br>flight `agile_slew_img` / ape_los_on_target_p9973<br>flight `agile_slew_rw_rcs` / ape_los_on_target_p9973<br>flight `agile_slew_vscmg` / ape_los_on_target_p9973<br>flight `fault_gimbal_cmg` / ape_los_p9973<br>flight `fault_st_img` / ape_los_p9973<br>flight `fault_wheel_img` / ape_los_p9973<br>flight `fine_hold_cmg` / ape_los_p9973<br>flight `fine_hold_fmr` / ape_los_p9973<br>flight `fine_hold_fmr_rcs` / ape_los_p9973<br>flight `fine_hold_img` / ape_los_p9973<br>flight `fine_hold_img_lqr` / ape_los_p9973<br>flight `fine_hold_img_smc` / ape_los_p9973<br>flight `fine_hold_rw_rcs` / ape_los_p9973<br>flight `fine_hold_vscmg` / ape_los_p9973<br>flight `mission_cmg` / ape_los_last_half_orbit_p9973<br>flight `mission_fmr` / ape_los_last_half_orbit_p9973<br>flight `mission_fmr_rcs` / ape_los_last_half_orbit_p9973<br>flight `mission_img` / ape_los_last_half_orbit_p9973<br>flight `mission_rw_rcs` / ape_los_last_half_orbit_p9973<br>flight `mission_vscmg` / ape_los_last_half_orbit_p9973<br>flight `slew_cmg` / ape_los_on_target_p9973<br>flight `slew_fmr` / ape_los_on_target_p9973<br>flight `slew_fmr_rcs` / ape_los_on_target_p9973<br>flight `slew_img` / ape_los_on_target_p9973<br>flight `slew_img_lqr` / ape_los_on_target_p9973<br>flight `slew_img_smc` / ape_los_on_target_p9973<br>flight `slew_rw_rcs` / ape_los_on_target_p9973<br>flight `slew_vscmg` / ape_los_on_target_p9973<br>design loop: nadir_pointing / ape_los_p9973 | 0.004544 ✅<br>0.006614 ✅<br>0.008686 ✅<br>0.008702 ✅<br>0.01008 ❌<br>0.005509 ✅<br>0.1022 ❌<br>28.62 ❌<br>0.005021 ✅<br>0.005689 ✅<br>0.005689 ✅<br>0.01049 ❌<br>0.004156 ✅<br>0.01945 ❌<br>0.03388 ❌<br>0.00828 ✅<br>0.005241 ✅<br>0.005954 ✅<br>0.005954 ✅<br>0.01066 ❌<br>0.007564 ✅<br>0.009595 ✅<br>0.004233 ✅<br>0.006529 ✅<br>0.006529 ✅<br>0.007925 ✅<br>0.004439 ✅<br>0.0178 ❌<br>0.007922 ✅<br>0.01083 ❌<br>0.004483 ✅ | not met (9 of 31) |
| Absolute knowledge error (AKE) (`req.ake`) † | 0.005 Degree | flight `fault_gimbal_cmg` / ake_los_p9973<br>flight `fault_st_img` / ake_los_p9973<br>flight `fault_wheel_img` / ake_los_p9973<br>flight `fine_hold_cmg` / ake_los_p9973<br>flight `fine_hold_fmr` / ake_los_p9973<br>flight `fine_hold_fmr_rcs` / ake_los_p9973<br>flight `fine_hold_img` / ake_los_p9973<br>flight `fine_hold_img_lqr` / ake_los_p9973<br>flight `fine_hold_img_smc` / ake_los_p9973<br>flight `fine_hold_rw_rcs` / ake_los_p9973<br>flight `fine_hold_vscmg` / ake_los_p9973<br>design loop: nadir_pointing / ake_los_p9973 | 0.002624 ✅<br>0.1093 ❌<br>0.002858 ✅<br>0.002626 ✅<br>0.002631 ✅<br>0.002631 ✅<br>0.002608 ✅<br>0.002652 ✅<br>0.002649 ✅<br>0.002602 ✅<br>0.002636 ✅<br>0.003247 ✅ | not met (1 of 12) |
| Rate stability (`req.rks`) | 0.005 DegreePerSecond | flight `fault_gimbal_cmg` / rate_stability_p9973<br>flight `fault_st_img` / rate_stability_p9973<br>flight `fault_wheel_img` / rate_stability_p9973<br>flight `fine_hold_cmg` / rate_stability_p9973<br>flight `fine_hold_fmr` / rate_stability_p9973<br>flight `fine_hold_fmr_rcs` / rate_stability_p9973<br>flight `fine_hold_img` / rate_stability_p9973<br>flight `fine_hold_img_lqr` / rate_stability_p9973<br>flight `fine_hold_img_smc` / rate_stability_p9973<br>flight `fine_hold_rw_rcs` / rate_stability_p9973<br>flight `fine_hold_vscmg` / rate_stability_p9973<br>design loop: nadir_pointing / rate_stability_p9973 | 0.002963 ✅<br>0.003771 ✅<br>0.6244 ❌<br>0.002437 ✅<br>0.004729 ✅<br>0.004729 ✅<br>0.003552 ✅<br>0.006704 ❌<br>0.003482 ✅<br>0.005641 ❌<br>0.004762 ✅<br>0.004965 ✅ | not met (3 of 12) |
| Reference slew time (`req.slew`) † | 60 Second | profile `slew_cmg`: 30° in 60 s<br>profile `slew_fmr`: 30° in 60 s<br>profile `slew_fmr_rcs`: 30° in 60 s<br>profile `slew_img`: 30° in 60 s<br>profile `slew_img_lqr`: 30° in 60 s<br>profile `slew_img_smc`: 30° in 60 s<br>profile `slew_rw_rcs`: 30° in 60 s<br>profile `slew_vscmg`: 30° in 60 s | ✅<br>✅<br>✅<br>✅<br>✅<br>✅<br>✅<br>✅ | met |
| Settling time after a slew (`req.settle`) † | 20 Second | flight `slew_cmg` / settle_time_after_slew<br>flight `slew_fmr` / settle_time_after_slew<br>flight `slew_fmr_rcs` / settle_time_after_slew<br>flight `slew_img` / settle_time_after_slew<br>flight `slew_img_lqr` / settle_time_after_slew<br>flight `slew_img_smc` / settle_time_after_slew<br>flight `slew_rw_rcs` / settle_time_after_slew<br>flight `slew_vscmg` / settle_time_after_slew | 12.4 ✅<br>12.9 ✅<br>12.9 ✅<br>12.1 ✅<br>0 ✅<br>— ❌<br>12.1 ✅<br>13.2 ✅ | not met (1 of 8) |
| Detumble time (`req.detumble`) † | 284 Minute | flight `detumble_img` / detumble_time<br>flight `mission_cmg` / detumble_time<br>flight `mission_fmr` / detumble_time<br>flight `mission_fmr_rcs` / detumble_time<br>flight `mission_img` / detumble_time<br>flight `mission_rw_rcs` / detumble_time<br>flight `mission_vscmg` / detumble_time<br>design loop: detumble / detumble_time | 47.54 ✅<br>57.01 ✅<br>61.33 ✅<br>61.33 ✅<br>51.01 ✅<br>51.01 ✅<br>57.02 ✅<br>58.91 ✅ | met |
| Sun-acquisition time in safe mode (`req.sunacq`) † | 95 Minute | design loop: sun_acquisition / sun_acquisition_time | 2.292 ✅ | met |
| ADCS mass (`req.mass`) | 1.6 Kilogram | design: selected `mtq_fmr` budget | 1.427 ✅ | met |
| ADCS orbit-average power (`req.pavg`) † | 2 Watt | flight `fault_gimbal_cmg` / power_mean<br>flight `fault_st_img` / power_mean<br>flight `fault_wheel_img` / power_mean<br>flight `fine_hold_cmg` / power_mean<br>flight `fine_hold_fmr` / power_mean<br>flight `fine_hold_fmr_rcs` / power_mean<br>flight `fine_hold_img` / power_mean<br>flight `fine_hold_img_lqr` / power_mean<br>flight `fine_hold_img_smc` / power_mean<br>flight `fine_hold_rw_rcs` / power_mean<br>flight `fine_hold_vscmg` / power_mean<br>design loop: nadir_pointing / power_mean<br>design loop: sun_acquisition / power_mean<br>design loop: sun_referencing / power_mean | 1.61 ✅<br>1.557 ✅<br>1.213 ✅<br>1.61 ✅<br>0.08406 ✅<br>0.08406 ✅<br>1.557 ✅<br>1.558 ✅<br>1.557 ✅<br>1.53 ✅<br>2.014 ❌<br>0.2873 ✅<br>0.6754 ✅<br>0.08899 ✅ | not met (1 of 14) |
| ADCS peak power (`req.ppk`) † | 4 Watt | design loop: detumble / power_peak | 0.2181 ✅ | met |
| ADCS volume (`req.vol`) | 1 Litre | design: selected `mtq_fmr` budget | 0.5344 ✅ | met |

Not stated by the case (nothing to check): `req.rpe`, `req.pde`, `req.rke`, `req.spo`, `req.track`, `req.wmax`, `req.faults`, `req.recover`, `req.hsat`, `req.dump`, `req.prop`.

## Metrics that report without judging

| scenario | metric | why |
|---|---|---|
| `agile_slew_cmg` | settle_time_after_slew | a 90 deg agile slew: the case states settling (req.settle) for the 30 deg reference slew, judged in slew_* |
| `agile_slew_cmg` | wheel_momentum_peak | momentum used, for sizing: the case does not state the margin req.hsat |
| `agile_slew_fmr_rcs` | settle_time_after_slew | a 90 deg agile slew: the case states settling (req.settle) for the 30 deg reference slew, judged in slew_* |
| `agile_slew_fmr_rcs` | wheel_momentum_peak | momentum used, for sizing: the case does not state the margin req.hsat |
| `agile_slew_img` | settle_time_after_slew | a 90 deg agile slew: the case states settling (req.settle) for the 30 deg reference slew, judged in slew_* |
| `agile_slew_img` | wheel_momentum_peak | momentum used, for sizing: the case does not state the margin req.hsat |
| `agile_slew_rw_rcs` | settle_time_after_slew | a 90 deg agile slew: the case states settling (req.settle) for the 30 deg reference slew, judged in slew_* |
| `agile_slew_rw_rcs` | wheel_momentum_peak | momentum used, for sizing: the case does not state the margin req.hsat |
| `agile_slew_vscmg` | settle_time_after_slew | a 90 deg agile slew: the case states settling (req.settle) for the 30 deg reference slew, judged in slew_* |
| `agile_slew_vscmg` | wheel_momentum_peak | momentum used, for sizing: the case does not state the margin req.hsat |
| `fault_gimbal_cmg` | ape_los_max | the worst sample, beside the judged p99.73: req.ape is a 3-sigma requirement |
| `fault_gimbal_cmg` | ape_3ax_p9973 | the three-axis error, beside the judged line of sight: req.ape is on the imager's line of sight |
| `fault_gimbal_cmg` | time_to_0p01_deg | acquisition from the initial error: no requirement states it (req.settle is after a slew) |
| `fault_gimbal_cmg` | wheel_momentum_peak | momentum used, for sizing: the case does not state the margin req.hsat |
| `fault_gyro_ais` | ape_3axis_last_orbit | yaw about the antenna axis is not required: the line of sight is judged (ape_los_last_orbit) |
| `fault_st_img` | ape_los_max | the worst sample, beside the judged p99.73: req.ape is a 3-sigma requirement |
| `fault_st_img` | ape_3ax_p9973 | the three-axis error, beside the judged line of sight: req.ape is on the imager's line of sight |
| `fault_st_img` | time_to_0p01_deg | acquisition from the initial error: no requirement states it (req.settle is after a slew) |
| `fault_st_img` | wheel_momentum_peak | momentum used, for sizing: the case does not state the margin req.hsat |
| `fault_wheel_img` | ape_los_max | the worst sample, beside the judged p99.73: req.ape is a 3-sigma requirement |
| `fault_wheel_img` | ape_3ax_p9973 | the three-axis error, beside the judged line of sight: req.ape is on the imager's line of sight |
| `fault_wheel_img` | time_to_0p01_deg | acquisition from the initial error: no requirement states it (req.settle is after a slew) |
| `fault_wheel_img` | wheel_momentum_peak | momentum used, for sizing: the case does not state the margin req.hsat |
| `fine_hold_cmg` | ape_los_max | the worst sample, beside the judged p99.73: req.ape is a 3-sigma requirement |
| `fine_hold_cmg` | ape_3ax_p9973 | the three-axis error, beside the judged line of sight: req.ape is on the imager's line of sight |
| `fine_hold_cmg` | time_to_0p01_deg | acquisition from the initial error: no requirement states it (req.settle is after a slew) |
| `fine_hold_cmg` | wheel_momentum_peak | momentum used, for sizing: the case does not state the margin req.hsat |
| `fine_hold_fmr` | ape_los_max | the worst sample, beside the judged p99.73: req.ape is a 3-sigma requirement |
| `fine_hold_fmr` | ape_3ax_p9973 | the three-axis error, beside the judged line of sight: req.ape is on the imager's line of sight |
| `fine_hold_fmr` | time_to_0p01_deg | acquisition from the initial error: no requirement states it (req.settle is after a slew) |
| `fine_hold_fmr` | wheel_momentum_peak | momentum used, for sizing: the case does not state the margin req.hsat |
| `fine_hold_fmr_rcs` | ape_los_max | the worst sample, beside the judged p99.73: req.ape is a 3-sigma requirement |
| `fine_hold_fmr_rcs` | ape_3ax_p9973 | the three-axis error, beside the judged line of sight: req.ape is on the imager's line of sight |
| `fine_hold_fmr_rcs` | time_to_0p01_deg | acquisition from the initial error: no requirement states it (req.settle is after a slew) |
| `fine_hold_fmr_rcs` | wheel_momentum_peak | momentum used, for sizing: the case does not state the margin req.hsat |
| `fine_hold_img` | ape_los_max | the worst sample, beside the judged p99.73: req.ape is a 3-sigma requirement |
| `fine_hold_img` | ape_3ax_p9973 | the three-axis error, beside the judged line of sight: req.ape is on the imager's line of sight |
| `fine_hold_img` | time_to_0p01_deg | acquisition from the initial error: no requirement states it (req.settle is after a slew) |
| `fine_hold_img` | wheel_momentum_peak | momentum used, for sizing: the case does not state the margin req.hsat |
| `fine_hold_img_lqr` | ape_los_max | the worst sample, beside the judged p99.73: req.ape is a 3-sigma requirement |
| `fine_hold_img_lqr` | ape_3ax_p9973 | the three-axis error, beside the judged line of sight: req.ape is on the imager's line of sight |
| `fine_hold_img_lqr` | time_to_0p01_deg | acquisition from the initial error: no requirement states it (req.settle is after a slew) |
| `fine_hold_img_lqr` | wheel_momentum_peak | momentum used, for sizing: the case does not state the margin req.hsat |
| `fine_hold_img_smc` | ape_los_max | the worst sample, beside the judged p99.73: req.ape is a 3-sigma requirement |
| `fine_hold_img_smc` | ape_3ax_p9973 | the three-axis error, beside the judged line of sight: req.ape is on the imager's line of sight |
| `fine_hold_img_smc` | time_to_0p01_deg | acquisition from the initial error: no requirement states it (req.settle is after a slew) |
| `fine_hold_img_smc` | wheel_momentum_peak | momentum used, for sizing: the case does not state the margin req.hsat |
| `fine_hold_rw_rcs` | ape_los_max | the worst sample, beside the judged p99.73: req.ape is a 3-sigma requirement |
| `fine_hold_rw_rcs` | ape_3ax_p9973 | the three-axis error, beside the judged line of sight: req.ape is on the imager's line of sight |
| `fine_hold_rw_rcs` | time_to_0p01_deg | acquisition from the initial error: no requirement states it (req.settle is after a slew) |
| `fine_hold_rw_rcs` | wheel_momentum_peak | momentum used, for sizing: the case does not state the margin req.hsat |
| `fine_hold_vscmg` | ape_los_max | the worst sample, beside the judged p99.73: req.ape is a 3-sigma requirement |
| `fine_hold_vscmg` | ape_3ax_p9973 | the three-axis error, beside the judged line of sight: req.ape is on the imager's line of sight |
| `fine_hold_vscmg` | time_to_0p01_deg | acquisition from the initial error: no requirement states it (req.settle is after a slew) |
| `fine_hold_vscmg` | wheel_momentum_peak | momentum used, for sizing: the case does not state the margin req.hsat |
| `mission_ais` | ape_3axis_last_orbit | yaw about the antenna axis is not required: the line of sight is judged (ape_los_last_orbit) |
| `nadir_hold_ais` | ape_3axis_last_orbit | yaw about the antenna axis is not required: the line of sight is judged (ape_los_last_orbit) |
| `nadir_hold_ais_css` | ape_3axis_last_orbit | yaw about the antenna axis is not required: the line of sight is judged (ape_los_last_orbit) |
| `slew_cmg` | wheel_momentum_peak | momentum used, for sizing: the case does not state the margin req.hsat |
| `slew_fmr` | wheel_momentum_peak | momentum used, for sizing: the case does not state the margin req.hsat |
| `slew_fmr_rcs` | wheel_momentum_peak | momentum used, for sizing: the case does not state the margin req.hsat |
| `slew_img` | wheel_momentum_peak | momentum used, for sizing: the case does not state the margin req.hsat |
| `slew_img_lqr` | wheel_momentum_peak | momentum used, for sizing: the case does not state the margin req.hsat |
| `slew_img_smc` | wheel_momentum_peak | momentum used, for sizing: the case does not state the margin req.hsat |
| `slew_rw_rcs` | wheel_momentum_peak | momentum used, for sizing: the case does not state the margin req.hsat |
| `slew_vscmg` | wheel_momentum_peak | momentum used, for sizing: the case does not state the margin req.hsat |
