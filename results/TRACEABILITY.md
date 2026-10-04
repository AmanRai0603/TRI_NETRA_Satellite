# Requirements traceability

> **Answer first.** 19 stated requirements across 2 cases; 11 met, 8 not met, 0 not yet flown, 0 with nothing checking them. Of 207 shipped metrics, 129 judge and 78 report with a stated reason.

> **Kind:** generated (`python3 tools/trace.py`) · **Source:** the cases, the scenarios, the latest stored runs, the design loop's selection

A requirement marked † is UNCONFIRMED in its case (a plan value a person has not confirmed).

## ais_3u

| requirement | value | checked by | latest | status |
|---|---|---|---|---|
| Absolute pointing error (APE) (`req.ape`) | 10 Degree | flight `fault_gyro_ais` / ape_los_last_orbit<br>flight `mission_ais` / ape_los_last_orbit<br>flight `nadir_hold_ais` / ape_los_last_orbit<br>flight `nadir_hold_ais_css` / ape_los_last_orbit<br>flight `safe_mode_ais` / ape_los_last_orbit<br>design loop: nadir_pointing / ape_los_p9973 | 10.92 ❌<br>11.5 ❌<br>10.67 ❌<br>9.706 ✅<br>12.3 ❌<br>0.1487 ✅ | not met (4 of 6) |
| Absolute knowledge error (AKE) (`req.ake`) † | 5 Degree | flight `fault_gyro_ais` / ake_los_last_orbit<br>flight `mission_ais` / ake_los_last_orbit<br>flight `nadir_hold_ais` / ake_los_last_orbit<br>flight `nadir_hold_ais_css` / ake_los_last_orbit<br>design loop: nadir_pointing / ake_los_p9973 | 1.038 ✅<br>1.316 ✅<br>1.638 ✅<br>1.569 ✅<br>0.146 ✅ | met |
| Detumble time (`req.detumble`) † | 284 Minute | flight `detumble_ais` / detumble_time<br>flight `detumble_ais_bangbang` / detumble_time<br>flight `detumble_ais_mag` / detumble_time<br>flight `fault_coil_ais` / detumble_time<br>flight `mission_ais` / detumble_time<br>design loop: detumble / detumble_time | 37.74 ✅<br>55.74 ✅<br>41.09 ✅<br>82.19 ✅<br>37.74 ✅<br>41.31 ✅ | met |
| Sun-acquisition time in safe mode (`req.sunacq`) † | 95 Minute | flight `sun_spin_ais` / sun_spin_entry<br>design loop: sun_acquisition / sun_acquisition_time | 63.22 ✅<br>2.292 ✅ | met |
| ADCS mass (`req.mass`) | 1.6 Kilogram | design: selected `mtq_fmr` budget | 2.319 ❌ | not met (1 of 1) |
| ADCS orbit-average power (`req.pavg`) † | 0.5 Watt | flight `detumble_ais` / power_mean<br>flight `detumble_ais_bangbang` / power_mean<br>flight `detumble_ais_mag` / power_mean<br>flight `fault_coil_ais` / power_mean<br>flight `fault_gyro_ais` / power_mean<br>flight `nadir_hold_ais` / power_mean<br>flight `nadir_hold_ais_css` / power_mean<br>flight `sun_mtq_ais` / power_mean<br>flight `sun_spin_ais` / power_mean<br>design loop: nadir_pointing / power_mean<br>design loop: sun_acquisition / power_mean<br>design loop: sun_referencing / power_mean | 0.02192 ✅<br>0.4556 ✅<br>0.07734 ✅<br>0.01819 ✅<br>0.01084 ✅<br>0.0106 ✅<br>0.01098 ✅<br>0.01113 ✅<br>0.4641 ✅<br>0.2781 ✅<br>0.1838 ✅<br>0.236 ✅ | met |
| ADCS peak power (`req.ppk`) † | 1.5 Watt | flight `detumble_ais` / power_peak<br>flight `detumble_ais_bangbang` / power_peak<br>flight `detumble_ais_mag` / power_peak<br>flight `fault_coil_ais` / power_peak<br>design loop: detumble / power_peak | 0.5476 ✅<br>0.717 ✅<br>0.6483 ✅<br>0.5476 ✅<br>0.3616 ✅ | met |
| ADCS volume (`req.vol`) | 1 Litre | design: selected `mtq_fmr` budget | 0.538 ✅ | met |

Not stated by the case (nothing to check): `req.rpe`, `req.pde`, `req.rks`, `req.rke`, `req.slew`, `req.settle`, `req.spo`, `req.track`, `req.wmax`, `req.faults`, `req.recover`, `req.hsat`, `req.dump`, `req.prop`.

## ais_img_3u

| requirement | value | checked by | latest | status |
|---|---|---|---|---|
| Absolute pointing error (APE) (`req.ape`) | 0.01 Degree | flight `agile_slew_cmg` / ape_los_on_target_p9973<br>flight `agile_slew_fmr_rcs` / ape_los_on_target_p9973<br>flight `agile_slew_img` / ape_los_on_target_p9973<br>flight `agile_slew_rw_rcs` / ape_los_on_target_p9973<br>flight `agile_slew_vscmg` / ape_los_on_target_p9973<br>flight `fault_gimbal_cmg` / ape_los_p9973<br>flight `fault_gps_img` / ape_los_p9973<br>flight `fault_st_img` / ape_los_p9973<br>flight `fault_valve_rcs` / ape_los_p9973<br>flight `fault_wheel_img` / ape_los_p9973<br>flight `fine_hold_cmg` / ape_los_p9973<br>flight `fine_hold_fmr` / ape_los_p9973<br>flight `fine_hold_fmr_rcs` / ape_los_p9973<br>flight `fine_hold_img` / ape_los_p9973<br>flight `fine_hold_img_lqr` / ape_los_p9973<br>flight `fine_hold_img_smc` / ape_los_p9973<br>flight `fine_hold_rw_rcs` / ape_los_p9973<br>flight `fine_hold_vscmg` / ape_los_p9973<br>flight `mission_cmg` / ape_los_last_half_orbit_p9973<br>flight `mission_fmr` / ape_los_last_half_orbit_p9973<br>flight `mission_fmr_rcs` / ape_los_last_half_orbit_p9973<br>flight `mission_img` / ape_los_last_half_orbit_p9973<br>flight `mission_rw_rcs` / ape_los_last_half_orbit_p9973<br>flight `mission_vscmg` / ape_los_last_half_orbit_p9973<br>flight `slew_cmg` / ape_los_on_target_p9973<br>flight `slew_fmr` / ape_los_on_target_p9973<br>flight `slew_fmr_rcs` / ape_los_on_target_p9973<br>flight `slew_img` / ape_los_on_target_p9973<br>flight `slew_img_lqr` / ape_los_on_target_p9973<br>flight `slew_img_smc` / ape_los_on_target_p9973<br>flight `slew_rw_rcs` / ape_los_on_target_p9973<br>flight `slew_vscmg` / ape_los_on_target_p9973<br>flight `target_img` / ape_los_p9973<br>design loop: nadir_pointing / ape_los_p9973 | 0.005506 ✅<br>0.006692 ✅<br>0.007963 ✅<br>0.008167 ✅<br>0.01165 ❌<br>0.005705 ✅<br>0.0112 ❌<br>0.00698 ✅<br>2.749 ❌<br>0.68 ❌<br>0.005218 ✅<br>0.005962 ✅<br>0.005962 ✅<br>0.006873 ✅<br>0.004237 ✅<br>0.01914 ❌<br>0.00681 ✅<br>0.008369 ✅<br>0.005214 ✅<br>0.005884 ✅<br>0.005884 ✅<br>0.00734 ✅<br>0.01489 ❌<br>0.009745 ✅<br>0.003681 ✅<br>0.006193 ✅<br>0.006193 ✅<br>0.007934 ✅<br>0.00471 ✅<br>0.01784 ❌<br>0.007921 ✅<br>0.01082 ❌<br>0.01207 ❌<br>0.004032 ✅ | not met (9 of 34) |
| Absolute knowledge error (AKE) (`req.ake`) † | 0.005 Degree | flight `fault_gimbal_cmg` / ake_los_p9973<br>flight `fault_gps_img` / ake_los_p9973<br>flight `fault_st_img` / ake_los_p9973<br>flight `fault_valve_rcs` / ake_los_p9973<br>flight `fault_wheel_img` / ake_los_p9973<br>flight `fine_hold_cmg` / ake_los_p9973<br>flight `fine_hold_fmr` / ake_los_p9973<br>flight `fine_hold_fmr_rcs` / ake_los_p9973<br>flight `fine_hold_img` / ake_los_p9973<br>flight `fine_hold_img_lqr` / ake_los_p9973<br>flight `fine_hold_img_smc` / ake_los_p9973<br>flight `fine_hold_rw_rcs` / ake_los_p9973<br>flight `fine_hold_vscmg` / ake_los_p9973<br>flight `target_img` / ake_los_p9973<br>design loop: nadir_pointing / ake_los_p9973 | 0.002868 ✅<br>0.002839 ✅<br>0.003056 ✅<br>0.002728 ✅<br>0.002983 ✅<br>0.002848 ✅<br>0.002868 ✅<br>0.002868 ✅<br>0.002842 ✅<br>0.002857 ✅<br>0.00275 ✅<br>0.002828 ✅<br>0.002852 ✅<br>0.01149 ❌<br>0.003411 ✅ | not met (1 of 15) |
| Rate stability (`req.rks`) | 0.005 DegreePerSecond | flight `fault_gimbal_cmg` / rate_stability_p9973<br>flight `fault_st_img` / rate_stability_p9973<br>flight `fault_wheel_img` / rate_stability_p9973<br>flight `fine_hold_cmg` / rate_stability_p9973<br>flight `fine_hold_fmr` / rate_stability_p9973<br>flight `fine_hold_fmr_rcs` / rate_stability_p9973<br>flight `fine_hold_img` / rate_stability_p9973<br>flight `fine_hold_img_lqr` / rate_stability_p9973<br>flight `fine_hold_img_smc` / rate_stability_p9973<br>flight `fine_hold_rw_rcs` / rate_stability_p9973<br>flight `fine_hold_vscmg` / rate_stability_p9973<br>design loop: nadir_pointing / rate_stability_p9973 | 0.002987 ✅<br>0.003484 ✅<br>0.03143 ❌<br>0.002469 ✅<br>0.004811 ✅<br>0.004811 ✅<br>0.003493 ✅<br>0.006898 ❌<br>0.00339 ✅<br>0.003533 ✅<br>0.004737 ✅<br>0.004324 ✅ | not met (2 of 12) |
| Reference slew time (`req.slew`) † | 60 Second | profile `slew_cmg`: 30° in 60 s<br>profile `slew_fmr`: 30° in 60 s<br>profile `slew_fmr_rcs`: 30° in 60 s<br>profile `slew_img`: 30° in 60 s<br>profile `slew_img_lqr`: 30° in 60 s<br>profile `slew_img_smc`: 30° in 60 s<br>profile `slew_rw_rcs`: 30° in 60 s<br>profile `slew_vscmg`: 30° in 60 s | ✅<br>✅<br>✅<br>✅<br>✅<br>✅<br>✅<br>✅ | met |
| Settling time after a slew (`req.settle`) † | 20 Second | flight `slew_cmg` / settle_time_after_slew<br>flight `slew_fmr` / settle_time_after_slew<br>flight `slew_fmr_rcs` / settle_time_after_slew<br>flight `slew_img` / settle_time_after_slew<br>flight `slew_img_lqr` / settle_time_after_slew<br>flight `slew_img_smc` / settle_time_after_slew<br>flight `slew_rw_rcs` / settle_time_after_slew<br>flight `slew_vscmg` / settle_time_after_slew | 12.4 ✅<br>12.9 ✅<br>12.9 ✅<br>12.1 ✅<br>0 ✅<br>— ❌<br>12.1 ✅<br>13.2 ✅ | not met (1 of 8) |
| Detumble time (`req.detumble`) † | 284 Minute | flight `detumble_img` / detumble_time<br>flight `detumble_rcs` / detumble_time<br>flight `mission_cmg` / detumble_time<br>flight `mission_fmr` / detumble_time<br>flight `mission_fmr_rcs` / detumble_time<br>flight `mission_img` / detumble_time<br>flight `mission_rw_rcs` / detumble_time<br>flight `mission_vscmg` / detumble_time<br>design loop: detumble / detumble_time | 47.58 ✅<br>— ❌<br>56.77 ✅<br>61.12 ✅<br>61.12 ✅<br>49.84 ✅<br>49.83 ✅<br>56.89 ✅<br>58.59 ✅ | not met (1 of 9) |
| Sun-acquisition time in safe mode (`req.sunacq`) † | 95 Minute | flight `sun_acq_rotor_img` / sun_acquisition_time<br>design loop: sun_acquisition / sun_acquisition_time | 2.208 ✅<br>2.292 ✅ | met |
| ADCS mass (`req.mass`) | 1.85 Kilogram | design: selected `mtq_fmr` budget | 1.794 ✅ | met |
| ADCS orbit-average power (`req.pavg`) † | 2 Watt | flight `fault_gimbal_cmg` / power_mean<br>flight `fault_gps_img` / power_mean<br>flight `fault_st_img` / power_mean<br>flight `fault_valve_rcs` / power_mean<br>flight `fault_wheel_img` / power_mean<br>flight `fine_hold_cmg` / power_mean<br>flight `fine_hold_fmr` / power_mean<br>flight `fine_hold_fmr_rcs` / power_mean<br>flight `fine_hold_img` / power_mean<br>flight `fine_hold_img_lqr` / power_mean<br>flight `fine_hold_img_smc` / power_mean<br>flight `fine_hold_rw_rcs` / power_mean<br>flight `fine_hold_vscmg` / power_mean<br>flight `sun_fine_img` / power_mean<br>flight `target_img` / power_mean<br>design loop: nadir_pointing / power_mean<br>design loop: sun_acquisition / power_mean<br>design loop: sun_referencing / power_mean | 1.61 ✅<br>1.559 ✅<br>1.559 ✅<br>1.513 ✅<br>1.138 ✅<br>1.61 ✅<br>0.1012 ✅<br>0.1012 ✅<br>1.559 ✅<br>1.56 ✅<br>1.559 ✅<br>1.532 ✅<br>2.014 ❌<br>1.526 ✅<br>1.564 ✅<br>0.3004 ✅<br>0.1948 ✅<br>0.6782 ✅ | not met (1 of 18) |
| ADCS peak power (`req.ppk`) † | 4 Watt | design loop: detumble / power_peak | 0.3025 ✅ | met |
| ADCS volume (`req.vol`) | 1 Litre | design: selected `mtq_fmr` budget | 0.6602 ✅ | met |

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
| `detumble_rcs` | propellant | the propellant a thruster detumble costs: the case states no propellant budget (req.prop) |
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
| `fine_hold_img` | rpe_los_10s_p9973 | ECSS relative pointing error over 10 s windows: the case does not state req.rpe |
| `fine_hold_img` | pde_los_10s_60s_max | ECSS pointing drift between 10 s windows 60 s apart: the case does not state req.pde |
| `fine_hold_img` | rke_los_10s_p9973 | ECSS relative knowledge error over 10 s windows: the case does not state req.rke |
| `fine_hold_img` | battery_dod_last_orbit | the deepest battery discharge over the orbit: the case does not state a depth-of-discharge limit |
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
| `mission_ais` | battery_dod_last_orbit | the deepest battery discharge over the orbit: the case does not state a depth-of-discharge limit |
| `mission_img` | battery_dod_last_orbit | the deepest battery discharge over the orbit: the case does not state a depth-of-discharge limit |
| `nadir_hold_ais` | ape_3axis_last_orbit | yaw about the antenna axis is not required: the line of sight is judged (ape_los_last_orbit) |
| `nadir_hold_ais` | battery_dod_last_orbit | the deepest battery discharge over the orbit: the case does not state a depth-of-discharge limit |
| `nadir_hold_ais_css` | ape_3axis_last_orbit | yaw about the antenna axis is not required: the line of sight is judged (ape_los_last_orbit) |
| `safe_mode_ais` | safe_mode_entry | the minute safe mode began: the fault at 100 min plus the 60 s the flight software waits |
| `safe_mode_ais` | recovery_to_ape | from the magnetometer's return to 10 min held inside the APE: the case does not state req.recover |
| `slew_cmg` | wheel_momentum_peak | momentum used, for sizing: the case does not state the margin req.hsat |
| `slew_fmr` | wheel_momentum_peak | momentum used, for sizing: the case does not state the margin req.hsat |
| `slew_fmr_rcs` | wheel_momentum_peak | momentum used, for sizing: the case does not state the margin req.hsat |
| `slew_img` | wheel_momentum_peak | momentum used, for sizing: the case does not state the margin req.hsat |
| `slew_img_lqr` | wheel_momentum_peak | momentum used, for sizing: the case does not state the margin req.hsat |
| `slew_img_smc` | wheel_momentum_peak | momentum used, for sizing: the case does not state the margin req.hsat |
| `slew_rw_rcs` | wheel_momentum_peak | momentum used, for sizing: the case does not state the margin req.hsat |
| `slew_vscmg` | wheel_momentum_peak | momentum used, for sizing: the case does not state the margin req.hsat |
| `sun_acq_rotor_img` | power_margin_orbit | the orbit that includes the acquisition: a deficit here is drawn from the battery |
| `sun_acq_rotor_img` | battery_dod_orbit | the deepest discharge of the acquisition orbit: the case does not state a depth-of-discharge limit |
