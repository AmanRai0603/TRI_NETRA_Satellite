# Requirements traceability

> **Answer first.** 19 stated requirements across 2 cases; 9 met, 10 not met, 0 not yet flown, 0 with nothing checking them. Of 304 shipped metrics, 171 judge and 133 report with a stated reason.

> **Kind:** generated (`python3 tools/trace.py`) · **Source:** the cases, the scenarios, the latest stored runs, the design loop's selection

A requirement marked † is UNCONFIRMED in its case (a plan value a person has not confirmed).

## ais_3u

| requirement | value | checked by | latest | status |
|---|---|---|---|---|
| Absolute pointing error (APE) (`req.ape`) | 10 Degree | flight `fault_gyro_ais` / ape_los_last_orbit<br>flight `mission_ais` / ape_los_last_orbit<br>flight `nadir_hold_ais` / ape_los_last_orbit<br>flight `nadir_hold_ais_css` / ape_los_last_orbit<br>flight `safe_mode_ais` / ape_los_last_orbit<br>design loop: nadir_pointing / ape_los_p9973 | 11.57 ❌<br>10.78 ❌<br>10.61 ❌<br>9.709 ✅<br>12.04 ❌<br>6.491 ✅ | not met (4 of 6) |
| Absolute knowledge error (AKE) (`req.ake`) † | 5 Degree | flight `fault_clock_step_back` / ake_los_last_orbit<br>flight `fault_clock_step_fwd` / ake_los_last_orbit<br>flight `fault_fsw_reset_day30` / ake_los_last_orbit<br>flight `fault_gyro_ais` / ake_los_last_orbit<br>flight `mission_ais` / ake_los_last_orbit<br>flight `nadir_hold_ais` / ake_los_last_orbit<br>flight `nadir_hold_ais_css` / ake_los_last_orbit<br>design loop: nadir_pointing / ake_los_p9973 | 1.158 ✅<br>1.158 ✅<br>1.047 ✅<br>1.044 ✅<br>1.286 ✅<br>1.605 ✅<br>1.563 ✅<br>0.3657 ✅ | met |
| Detumble time (`req.detumble`) † | 284 Minute | flight `detumble_ais` / detumble_time<br>flight `detumble_ais_bangbang` / detumble_time<br>flight `detumble_ais_mag` / detumble_time<br>flight `fault_coil_ais` / detumble_time<br>flight `mission_ais` / detumble_time<br>design loop: detumble / detumble_time | 37.74 ✅<br>55.74 ✅<br>41.09 ✅<br>82.19 ✅<br>37.74 ✅<br>50.74 ✅ | met |
| Sun-acquisition time in safe mode (`req.sunacq`) † | 95 Minute | flight `sun_spin_ais` / sun_spin_entry<br>design loop: sun_acquisition / sun_acquisition_time | 63.22 ✅<br>2.292 ✅ | met |
| ADCS mass (`req.mass`) | 1.6 Kilogram | design: selected `mtq_fmr` budget | 11.32 ❌ | not met (1 of 1) |
| ADCS orbit-average power (`req.pavg`) † | 0.5 Watt | flight `detumble_ais` / power_mean<br>flight `detumble_ais_bangbang` / power_mean<br>flight `detumble_ais_mag` / power_mean<br>flight `fault_coil_ais` / power_mean<br>flight `fault_gyro_ais` / power_mean<br>flight `nadir_hold_ais` / power_mean<br>flight `nadir_hold_ais_css` / power_mean<br>flight `sun_mtq_ais` / power_mean<br>flight `sun_spin_ais` / power_mean<br>design loop: nadir_pointing / power_mean<br>design loop: sun_acquisition / power_mean<br>design loop: sun_referencing / power_mean | 1.522 ❌<br>1.956 ❌<br>1.577 ❌<br>1.518 ❌<br>1.511 ❌<br>1.511 ❌<br>1.231 ❌<br>1.511 ❌<br>1.964 ❌<br>1.624 ❌<br>1.702 ❌<br>1.612 ❌ | not met (12 of 12) |
| ADCS peak power (`req.ppk`) † | 1.5 Watt | flight `detumble_ais` / power_peak<br>flight `detumble_ais_bangbang` / power_peak<br>flight `detumble_ais_mag` / power_peak<br>flight `fault_coil_ais` / power_peak<br>design loop: detumble / power_peak | 2.048 ❌<br>2.217 ❌<br>2.148 ❌<br>2.048 ❌<br>1.785 ❌ | not met (5 of 5) |
| ADCS volume (`req.vol`) | 1 Litre | design: selected `mtq_fmr` budget | 0.538 ✅ | met |

Not stated by the case (nothing to check): `req.rpe`, `req.pde`, `req.rks`, `req.rke`, `req.slew`, `req.settle`, `req.spo`, `req.track`, `req.wmax`, `req.faults`, `req.recover`, `req.hsat`, `req.dump`, `req.prop`.

## ais_img_3u

| requirement | value | checked by | latest | status |
|---|---|---|---|---|
| Absolute pointing error (APE) (`req.ape`) | 0.01 Degree | flight `agile_slew_cmg` / ape_los_on_target_p9973<br>flight `agile_slew_fmr_rcs` / ape_los_on_target_p9973<br>flight `agile_slew_img` / ape_los_on_target_p9973<br>flight `agile_slew_rw_rcs` / ape_los_on_target_p9973<br>flight `agile_slew_vscmg` / ape_los_on_target_p9973<br>flight `fault_clock_jitter_1us` / ape_los_p9973<br>flight `fault_clock_jitter_5ms` / ape_los_p9973<br>flight `fault_gimbal_cmg` / ape_los_p9973<br>flight `fault_gps_img` / ape_los_p9973<br>flight `fault_st_img` / ape_los_p9973<br>flight `fault_valve_rcs` / ape_los_p9973<br>flight `fault_wheel_img` / ape_los_p9973<br>flight `fine_hold_cmg` / ape_los_p9973<br>flight `fine_hold_fmr` / ape_los_p9973<br>flight `fine_hold_fmr_rcs` / ape_los_p9973<br>flight `fine_hold_img` / ape_los_p9973<br>flight `fine_hold_img_lqr` / ape_los_p9973<br>flight `fine_hold_img_smc` / ape_los_p9973<br>flight `fine_hold_rw_rcs` / ape_los_p9973<br>flight `fine_hold_vscmg` / ape_los_p9973<br>flight `mission_cmg` / ape_los_last_half_orbit_p9973<br>flight `mission_fmr` / ape_los_last_half_orbit_p9973<br>flight `mission_fmr_rcs` / ape_los_last_half_orbit_p9973<br>flight `mission_img` / ape_los_last_half_orbit_p9973<br>flight `mission_rw_rcs` / ape_los_last_half_orbit_p9973<br>flight `mission_vscmg` / ape_los_last_half_orbit_p9973<br>flight `slew_cmg` / ape_los_on_target_p9973<br>flight `slew_fmr` / ape_los_on_target_p9973<br>flight `slew_fmr_rcs` / ape_los_on_target_p9973<br>flight `slew_img` / ape_los_on_target_p9973<br>flight `slew_img_lqr` / ape_los_on_target_p9973<br>flight `slew_img_smc` / ape_los_on_target_p9973<br>flight `slew_rw_rcs` / ape_los_on_target_p9973<br>flight `slew_vscmg` / ape_los_on_target_p9973<br>flight `target_img` / ape_los_p9973<br>design loop: nadir_pointing / ape_los_p9973 | 0.004502 ✅<br>0.006668 ✅<br>0.006681 ✅<br>0.006559 ✅<br>0.009575 ✅<br>0.004616 ✅<br>0.004633 ✅<br>0.00502 ✅<br>0.01047 ❌<br>0.005508 ✅<br>1.221 ❌<br>0.4723 ❌<br>0.004055 ✅<br>0.004915 ✅<br>0.004915 ✅<br>0.005202 ✅<br>0.004128 ✅<br>0.01077 ❌<br>0.005185 ✅<br>0.006398 ✅<br>0.00431 ✅<br>0.004874 ✅<br>0.004874 ✅<br>0.005449 ✅<br>0.005363 ✅<br>0.006916 ✅<br>0.003666 ✅<br>0.006051 ✅<br>0.006051 ✅<br>0.006261 ✅<br>0.00469 ✅<br>0.01097 ❌<br>0.006262 ✅<br>0.007254 ✅<br>0.01163 ❌<br>30.96 ❌ | not met (7 of 36) |
| Absolute knowledge error (AKE) (`req.ake`) † | 0.005 Degree | flight `fault_clock_jitter_1us` / ake_los_p9973<br>flight `fault_clock_jitter_5ms` / ake_los_p9973<br>flight `fault_gimbal_cmg` / ake_los_p9973<br>flight `fault_gps_img` / ake_los_p9973<br>flight `fault_st_img` / ake_los_p9973<br>flight `fault_valve_rcs` / ake_los_p9973<br>flight `fault_wheel_img` / ake_los_p9973<br>flight `fine_hold_cmg` / ake_los_p9973<br>flight `fine_hold_fmr` / ake_los_p9973<br>flight `fine_hold_fmr_rcs` / ake_los_p9973<br>flight `fine_hold_img` / ake_los_p9973<br>flight `fine_hold_img_lqr` / ake_los_p9973<br>flight `fine_hold_img_smc` / ake_los_p9973<br>flight `fine_hold_rw_rcs` / ake_los_p9973<br>flight `fine_hold_vscmg` / ake_los_p9973<br>flight `target_img` / ake_los_p9973<br>design loop: nadir_pointing / ake_los_p9973 | 0.002289 ✅<br>0.002288 ✅<br>0.002853 ✅<br>0.00283 ✅<br>0.003059 ✅<br>0.002747 ✅<br>0.002984 ✅<br>0.002597 ✅<br>0.002591 ✅<br>0.002591 ✅<br>0.002605 ✅<br>0.002639 ✅<br>0.002592 ✅<br>0.002606 ✅<br>0.002635 ✅<br>0.01144 ❌<br>0.00278 ✅ | not met (1 of 17) |
| Rate stability (`req.rks`) | 0.005 DegreePerSecond | flight `fault_gimbal_cmg` / rate_stability_p9973<br>flight `fault_st_img` / rate_stability_p9973<br>flight `fault_wheel_img` / rate_stability_p9973<br>flight `fine_hold_cmg` / rate_stability_p9973<br>flight `fine_hold_fmr` / rate_stability_p9973<br>flight `fine_hold_fmr_rcs` / rate_stability_p9973<br>flight `fine_hold_img` / rate_stability_p9973<br>flight `fine_hold_img_lqr` / rate_stability_p9973<br>flight `fine_hold_img_smc` / rate_stability_p9973<br>flight `fine_hold_rw_rcs` / rate_stability_p9973<br>flight `fine_hold_vscmg` / rate_stability_p9973<br>design loop: nadir_pointing / rate_stability_p9973 | 0.003289 ✅<br>0.003552 ✅<br>0.04016 ❌<br>0.002673 ✅<br>0.00498 ✅<br>0.00498 ✅<br>0.003654 ✅<br>0.006662 ❌<br>0.003477 ✅<br>0.003648 ✅<br>0.004649 ✅<br>0.1408 ❌ | not met (3 of 12) |
| Reference slew time (`req.slew`) † | 60 Second | profile `slew_cmg`: 30° in 60 s<br>profile `slew_fmr`: 30° in 60 s<br>profile `slew_fmr_rcs`: 30° in 60 s<br>profile `slew_img`: 30° in 60 s<br>profile `slew_img_lqr`: 30° in 60 s<br>profile `slew_img_smc`: 30° in 60 s<br>profile `slew_rw_rcs`: 30° in 60 s<br>profile `slew_vscmg`: 30° in 60 s | ✅<br>✅<br>✅<br>✅<br>✅<br>✅<br>✅<br>✅ | met |
| Settling time after a slew (`req.settle`) † | 20 Second | flight `slew_cmg` / settle_time_after_slew<br>flight `slew_fmr` / settle_time_after_slew<br>flight `slew_fmr_rcs` / settle_time_after_slew<br>flight `slew_img` / settle_time_after_slew<br>flight `slew_img_lqr` / settle_time_after_slew<br>flight `slew_img_smc` / settle_time_after_slew<br>flight `slew_rw_rcs` / settle_time_after_slew<br>flight `slew_vscmg` / settle_time_after_slew | 5.7 ✅<br>10.1 ✅<br>10.1 ✅<br>8.9 ✅<br>0 ✅<br>8.9 ✅<br>8.9 ✅<br>9 ✅ | met |
| Detumble time (`req.detumble`) † | 284 Minute | flight `detumble_img` / detumble_time<br>flight `detumble_rcs` / detumble_time<br>flight `mission_cmg` / detumble_time<br>flight `mission_fmr` / detumble_time<br>flight `mission_fmr_rcs` / detumble_time<br>flight `mission_img` / detumble_time<br>flight `mission_rw_rcs` / detumble_time<br>flight `mission_vscmg` / detumble_time<br>design loop: detumble / detumble_time | 47.58 ✅<br>— ❌<br>56.79 ✅<br>61.44 ✅<br>61.44 ✅<br>49.86 ✅<br>49.86 ✅<br>56.89 ✅<br>56.37 ✅ | not met (1 of 9) |
| Sun-acquisition time in safe mode (`req.sunacq`) † | 95 Minute | flight `sun_acq_rotor_img` / sun_acquisition_time<br>design loop: sun_acquisition / sun_acquisition_time | 2.208 ✅<br>2.275 ✅ | met |
| ADCS mass (`req.mass`) | 1.85 Kilogram | design: selected `mtq_fmr` budget | 12 ❌ | not met (1 of 1) |
| ADCS orbit-average power (`req.pavg`) † | 2 Watt | flight `fault_gimbal_cmg` / power_mean<br>flight `fault_gps_img` / power_mean<br>flight `fault_st_img` / power_mean<br>flight `fault_valve_rcs` / power_mean<br>flight `fault_wheel_img` / power_mean<br>flight `fine_hold_cmg` / power_mean<br>flight `fine_hold_fmr` / power_mean<br>flight `fine_hold_fmr_rcs` / power_mean<br>flight `fine_hold_img` / power_mean<br>flight `fine_hold_img_lqr` / power_mean<br>flight `fine_hold_img_smc` / power_mean<br>flight `fine_hold_rw_rcs` / power_mean<br>flight `fine_hold_vscmg` / power_mean<br>flight `sun_fine_img` / power_mean<br>flight `target_img` / power_mean<br>design loop: nadir_pointing / power_mean<br>design loop: sun_acquisition / power_mean<br>design loop: sun_referencing / power_mean | 5.11 ❌<br>5.059 ❌<br>5.059 ❌<br>5.013 ❌<br>4.641 ❌<br>5.11 ❌<br>3.601 ❌<br>3.601 ❌<br>5.065 ❌<br>5.066 ❌<br>5.065 ❌<br>5.037 ❌<br>5.515 ❌<br>5.026 ❌<br>5.064 ❌<br>3.619 ❌<br>1.641 ✅<br>4.01 ❌ | not met (17 of 18) |
| ADCS peak power (`req.ppk`) † | 4 Watt | design loop: detumble / power_peak | 1.903 ✅ | met |
| ADCS volume (`req.vol`) | 1 Litre | design: selected `mtq_fmr` budget | 0.5363 ✅ | met |

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
| `fault_bus_coils_brownout` | ake_los_max | the knowledge error across the line of sight over the run, beside the flag |
| `fault_bus_coils_brownout` | ape_los_max | the performance error across the line of sight after 60 s (the 2 deg start settled): what the fault does to the pointing (a bus delay shows here) |
| `fault_bus_coils_nack` | ake_los_max | the knowledge error across the line of sight over the run, beside the flag |
| `fault_bus_coils_nack` | ape_los_max | the performance error across the line of sight after 60 s (the 2 deg start settled): what the fault does to the pointing (a bus delay shows here) |
| `fault_bus_coils_stuck` | ake_los_max | the knowledge error across the line of sight over the run, beside the flag |
| `fault_bus_coils_stuck` | ape_los_max | the performance error across the line of sight after 60 s (the 2 deg start settled): what the fault does to the pointing (a bus delay shows here) |
| `fault_bus_gnss_brownout` | ake_los_max | the knowledge error across the line of sight over the run, beside the flag |
| `fault_bus_gnss_brownout` | ape_los_max | the performance error across the line of sight after 60 s (the 2 deg start settled): what the fault does to the pointing (a bus delay shows here) |
| `fault_bus_gnss_crc` | ake_los_max | the knowledge error across the line of sight over the run, beside the flag |
| `fault_bus_gnss_crc` | ape_los_max | the performance error across the line of sight after 60 s (the 2 deg start settled): what the fault does to the pointing (a bus delay shows here) |
| `fault_bus_gnss_nack` | ake_los_max | the knowledge error across the line of sight over the run, beside the flag |
| `fault_bus_gnss_nack` | ape_los_max | the performance error across the line of sight after 60 s (the 2 deg start settled): what the fault does to the pointing (a bus delay shows here) |
| `fault_bus_gnss_stuck` | ake_los_max | the knowledge error across the line of sight over the run, beside the flag |
| `fault_bus_gnss_stuck` | ape_los_max | the performance error across the line of sight after 60 s (the 2 deg start settled): what the fault does to the pointing (a bus delay shows here) |
| `fault_bus_gyro_brownout` | ake_los_max | the knowledge error across the line of sight over the run, beside the flag |
| `fault_bus_gyro_brownout` | ape_los_max | the performance error across the line of sight after 60 s (the 2 deg start settled): what the fault does to the pointing (a bus delay shows here) |
| `fault_bus_gyro_delay` | ake_los_max | the knowledge error across the line of sight over the run, beside the flag |
| `fault_bus_gyro_delay` | ape_los_max | the performance error across the line of sight after 60 s (the 2 deg start settled): what the fault does to the pointing (a bus delay shows here) |
| `fault_bus_gyro_nack` | ake_los_max | the knowledge error across the line of sight over the run, beside the flag |
| `fault_bus_gyro_nack` | ape_los_max | the performance error across the line of sight after 60 s (the 2 deg start settled): what the fault does to the pointing (a bus delay shows here) |
| `fault_bus_gyro_stuck` | ake_los_max | the knowledge error across the line of sight over the run, beside the flag |
| `fault_bus_gyro_stuck` | ape_los_max | the performance error across the line of sight after 60 s (the 2 deg start settled): what the fault does to the pointing (a bus delay shows here) |
| `fault_bus_mag_brownout` | ake_los_max | the knowledge error across the line of sight over the run, beside the flag |
| `fault_bus_mag_brownout` | ape_los_max | the performance error across the line of sight after 60 s (the 2 deg start settled): what the fault does to the pointing (a bus delay shows here) |
| `fault_bus_mag_nack` | ake_los_max | the knowledge error across the line of sight over the run, beside the flag |
| `fault_bus_mag_nack` | ape_los_max | the performance error across the line of sight after 60 s (the 2 deg start settled): what the fault does to the pointing (a bus delay shows here) |
| `fault_bus_mag_stuck` | ake_los_max | the knowledge error across the line of sight over the run, beside the flag |
| `fault_bus_mag_stuck` | ape_los_max | the performance error across the line of sight after 60 s (the 2 deg start settled): what the fault does to the pointing (a bus delay shows here) |
| `fault_bus_mex_brownout` | ake_los_max | the knowledge error across the line of sight over the run, beside the flag |
| `fault_bus_mex_brownout` | ape_los_max | the performance error across the line of sight after 60 s (the 2 deg start settled): what the fault does to the pointing (a bus delay shows here) |
| `fault_bus_mex_can_error` | ake_los_max | the knowledge error across the line of sight over the run, beside the flag |
| `fault_bus_mex_can_error` | ape_los_max | the performance error across the line of sight after 60 s (the 2 deg start settled): what the fault does to the pointing (a bus delay shows here) |
| `fault_bus_mex_nack` | ake_los_max | the knowledge error across the line of sight over the run, beside the flag |
| `fault_bus_mex_nack` | ape_los_max | the performance error across the line of sight after 60 s (the 2 deg start settled): what the fault does to the pointing (a bus delay shows here) |
| `fault_bus_mex_stuck` | ake_los_max | the knowledge error across the line of sight over the run, beside the flag |
| `fault_bus_mex_stuck` | ape_los_max | the performance error across the line of sight after 60 s (the 2 deg start settled): what the fault does to the pointing (a bus delay shows here) |
| `fault_bus_st_brownout` | ake_los_max | the knowledge error across the line of sight over the run, beside the flag |
| `fault_bus_st_brownout` | ape_los_max | the performance error across the line of sight after 60 s (the 2 deg start settled): what the fault does to the pointing (a bus delay shows here) |
| `fault_bus_st_crc` | ake_los_max | the knowledge error across the line of sight over the run, beside the flag |
| `fault_bus_st_crc` | ape_los_max | the performance error across the line of sight after 60 s (the 2 deg start settled): what the fault does to the pointing (a bus delay shows here) |
| `fault_bus_st_nack` | ake_los_max | the knowledge error across the line of sight over the run, beside the flag |
| `fault_bus_st_nack` | ape_los_max | the performance error across the line of sight after 60 s (the 2 deg start settled): what the fault does to the pointing (a bus delay shows here) |
| `fault_bus_st_stuck` | ake_los_max | the knowledge error across the line of sight over the run, beside the flag |
| `fault_bus_st_stuck` | ape_los_max | the performance error across the line of sight after 60 s (the 2 deg start settled): what the fault does to the pointing (a bus delay shows here) |
| `fault_bus_sun_brownout` | ake_los_max | the knowledge error across the line of sight over the run, beside the flag |
| `fault_bus_sun_brownout` | ape_los_max | the performance error across the line of sight after 60 s (the 2 deg start settled): what the fault does to the pointing (a bus delay shows here) |
| `fault_bus_sun_nack` | ake_los_max | the knowledge error across the line of sight over the run, beside the flag |
| `fault_bus_sun_nack` | ape_los_max | the performance error across the line of sight after 60 s (the 2 deg start settled): what the fault does to the pointing (a bus delay shows here) |
| `fault_bus_sun_stuck` | ake_los_max | the knowledge error across the line of sight over the run, beside the flag |
| `fault_bus_sun_stuck` | ape_los_max | the performance error across the line of sight after 60 s (the 2 deg start settled): what the fault does to the pointing (a bus delay shows here) |
| `fault_clock_step_back` | ape_los_last_orbit | the performance error beside the judged knowledge error: this product's line of sight does not meet req.ape on its case in a clean run either |
| `fault_clock_step_fwd` | ape_los_last_orbit | the performance error beside the judged knowledge error: this product's line of sight does not meet req.ape on its case in a clean run either |
| `fault_fsw_reset_day30` | ape_los_last_orbit | the performance error beside the judged knowledge error: this product's line of sight does not meet req.ape on its case in a clean run either |
| `fault_gimbal_cmg` | ape_los_max | the worst sample, beside the judged p99.73: req.ape is a 3-sigma requirement |
| `fault_gimbal_cmg` | ape_3ax_p9973 | the three-axis error, beside the judged line of sight: req.ape is on the imager's line of sight |
| `fault_gimbal_cmg` | time_to_0p01_deg | acquisition from the initial error: no requirement states it (req.settle is after a slew) |
| `fault_gimbal_cmg` | wheel_momentum_peak | momentum used, for sizing: the case does not state the margin req.hsat |
| `fault_gyro_ais` | ape_3axis_last_orbit | yaw about the antenna axis is not required: the line of sight is judged (ape_los_last_orbit) |
| `fault_gyro_silent` | ake_los_max | the knowledge error across the line of sight, beside the judged three-axis bound (a clean run holds 0.004 deg) |
| `fault_gyro_silent` | ape_los_max | the performance error across the line of sight (a clean run holds 0.008 deg) |
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
