# Every row, every closure, from the design database

**In one line:** for each case, every row of the design with its value (stated by the case, computed by its pseudocode, or supplied by a campaign) or why it has none, and every KPI closure answered or blocked by name (`tools/evaluate.py`, `docs/END_TO_END.md`).

## ais_3u

From the node files (no group has released yet). Rows: 12 computed, 4 evidence, 669 not computed, 27 stated. Closures: 34 blocked, 4 pass.

| Closure | KPI | Answer | Why |
|---|---|---|---|
| `kpi_absolute_pointing_error_ape_verified` | Absolute pointing error (APE) | **pass** | p1a_0 = 0.5455 deg <= 10.0 Degree (p1k_0) |
| `kpi_absolute_pointing_error_ape_analysis` | Absolute pointing error (APE) | blocked | gp_5 has no value (needs terms (no input names it)) |
| `kpi_absolute_knowledge_error_ake_verified` | Absolute knowledge error (AKE) | **pass** | p1a_1 = 0.5438 deg <= 5.0 Degree (p1k_1) |
| `kpi_absolute_knowledge_error_ake_analysis` | Absolute knowledge error (AKE) | blocked | ge_5 has no value (no value, relation or pseudocode yet) |
| `kpi_relative_pointing_error_rpe_verified` | Relative pointing error (RPE) | blocked | the requirement p1k_2 has no value (the case does not state req.rpe) |
| `kpi_relative_pointing_error_rpe_analysis` | Relative pointing error (RPE) | blocked | the requirement p1k_2 has no value (the case does not state req.rpe) |
| `kpi_pointing_drift_error_pde_verified` | Pointing drift error (PDE) | blocked | the requirement p1k_3 has no value (the case does not state req.pde) |
| `kpi_pointing_drift_error_pde_analysis` | Pointing drift error (PDE) | blocked | the requirement p1k_3 has no value (the case does not state req.pde) |
| `kpi_rate_stability_verified` | Rate stability | blocked | the requirement p1k_4 has no value (the case does not state req.rks) |
| `kpi_relative_knowledge_error_rke_verified` | Relative knowledge error (RKE) | blocked | the requirement p1k_5 has no value (the case does not state req.rke) |
| `kpi_reference_slew_time_verified` | Reference slew time | blocked | the requirement p2k_0 has no value (the case does not state req.slew) |
| `kpi_reference_slew_time_analysis` | Reference slew time | blocked | the requirement p2k_0 has no value (the case does not state req.slew) |
| `kpi_settling_time_after_a_slew_verified` | Settling time after a slew | blocked | the requirement p2k_1 has no value (the case does not state req.settle) |
| `kpi_settling_time_after_a_slew_analysis` | Settling time after a slew | blocked | the requirement p2k_1 has no value (the case does not state req.settle) |
| `kpi_slews_per_orbit_verified` | Slews per orbit | blocked | the requirement p2k_2 has no value (the case does not state req.spo) |
| `kpi_target_tracking_rate_verified` | Target-tracking rate | blocked | the requirement p2k_3 has no value (the case does not state req.track) |
| `kpi_maximum_body_rate_verified` | Maximum body rate | blocked | the requirement p2k_4 has no value (the case does not state req.wmax) |
| `kpi_detumble_time_verified` | Detumble time | **pass** | p3a_0 = 76.38 min <= 284.0 Minute (p3k_0) |
| `kpi_detumble_time_analysis` | Detumble time | blocked | gq_0 has no value (no value, relation or pseudocode yet) |
| `kpi_sun_acquisition_time_in_safe_mode_verified` | Sun-acquisition time in safe mode | blocked | p3a_1 has no value (evidence: no engine campaign of this case judges a metric against req.sunacq) |
| `kpi_sun_acquisition_time_in_safe_mode_analysis` | Sun-acquisition time in safe mode | blocked | gq_1 has no value (no value, relation or pseudocode yet) |
| `kpi_faults_tolerated_verified` | Faults tolerated | blocked | the requirement p3k_2 has no value (the case does not state req.faults) |
| `kpi_faults_tolerated_analysis` | Faults tolerated | blocked | the requirement p3k_2 has no value (the case does not state req.faults) |
| `kpi_recovery_time_after_a_single_fault_verified` | Recovery time after a single fault | blocked | the requirement p3k_3 has no value (the case does not state req.recover) |
| `kpi_recovery_time_after_a_single_fault_analysis` | Recovery time after a single fault | blocked | the requirement p3k_3 has no value (the case does not state req.recover) |
| `kpi_momentum_saturation_margin_verified` | Momentum saturation margin | blocked | the requirement p3k_4 has no value (the case does not state req.hsat) |
| `kpi_momentum_dump_interval_verified` | Momentum dump interval | blocked | the requirement p3k_5 has no value (the case does not state req.dump) |
| `kpi_momentum_dump_interval_analysis` | Momentum dump interval | blocked | the requirement p3k_5 has no value (the case does not state req.dump) |
| `kpi_adcs_mass_verified` | ADCS mass | blocked | p4a_0 has no value (evidence: no engine campaign of this case judges a metric against req.mass) |
| `kpi_adcs_mass_analysis` | ADCS mass | blocked | gb_0 has no value (no value, relation or pseudocode yet) |
| `kpi_adcs_orbit_average_power_verified` | ADCS orbit-average power | **pass** | p4a_1 = 0.3554 W <= 0.5 Watt (p4k_1) |
| `kpi_adcs_orbit_average_power_analysis` | ADCS orbit-average power | blocked | gb_1 has no value (no value, relation or pseudocode yet) |
| `kpi_adcs_peak_power_verified` | ADCS peak power | blocked | p4a_2 has no value (evidence: no engine campaign of this case judges a metric against req.ppk) |
| `kpi_adcs_peak_power_analysis` | ADCS peak power | blocked | gb_2 has no value (no value, relation or pseudocode yet) |
| `kpi_adcs_volume_verified` | ADCS volume | blocked | p4a_3 has no value (evidence: no engine campaign of this case judges a metric against req.vol) |
| `kpi_adcs_volume_analysis` | ADCS volume | blocked | gb_3 has no value (no value, relation or pseudocode yet) |
| `kpi_rcs_propellant_per_year_verified` | RCS propellant per year | blocked | the requirement p4k_4 has no value (the case does not state req.prop) |
| `kpi_rcs_propellant_per_year_analysis` | RCS propellant per year | blocked | the requirement p4k_4 has no value (the case does not state req.prop) |

| Row | Group | State | Value | Where from / why not |
|---|---|---|---|---|
| `gd_0` | env | computed | 6.34679e-08 (SI) | gd_0(r, i_max, i_min) |
| `gd_1` | env | computed | 1.36971e-08 (SI) | gd_1(rho, v, c_d, a_fr, c_pa) |
| `gd_2` | env | computed | 5.10763e-09 (SI) | gd_2(p_srp, a_sun, q, c_ps) |
| `m1_1` | case | stated | 27.0 Year | the case, mission.epoch |
| `m1_3` | case | stated | 10.0 Count | the case, mission.spd |
| `m1_4` | case | stated | 30.0 Degree | the case, mission.sangle |
| `m1_5` | case | stated | 10.0 DegreePerSecond | the case, mission.w0 |
| `m2_0` | env | stated | 550.0 Kilometre | the case, orbit.alt |
| `m2_1` | env | stated | 97.593 Degree | the case, orbit.inc |
| `m2_2` | env | stated | 0.0 One | the case, orbit.ecc |
| `m2_3` | env | stated | 6.0 Hour | the case, orbit.ltan |
| `m2_4` | env | computed | 6.92814e+06 (SI) | radius(h) |
| `m2_5` | env | computed | 5738.99 (SI) | m2_5(r) |
| `m2_6` | env | computed | 0.00109482 (SI) | mean_motion(t_orb) |
| `m3_0` | env | computed | 2.3256e-05 (SI) | m3_0(r) |
| `m3_1` | env | computed | 4.6512e-05 (SI) | m3_1(r, lambda_max) |
| `m3_2` | env | computed | 1.5708 (SI) | max_magnetic_latitude(i) |
| `m3_3` | env | computed | 3.18278e-13 (SI) | density_at(h, t_epoch) |
| `m3_4` | env | computed | 7585.09 (SI) | circular_speed(r) |
| `m3_5` | env | computed | 4.69452e-06 (SI) | m3_5(t_epoch) |
| `p1a_0` | kpi | evidence | 0.5455 deg | the design loop's Monte Carlo of the selected family mtq_fmr (mc_dispatch_ais_3u): ape_los_p9973, the worst of 12 runs |
| `p1a_1` | kpi | evidence | 0.5438 deg | the design loop's Monte Carlo of the selected family mtq_fmr (mc_dispatch_ais_3u): ake_los_p9973, the worst of 12 runs |
| `p1k_0` | kpi | stated | 10.0 Degree | the case, req.ape |
| `p1k_1` | kpi | stated | 5.0 Degree | the case, req.ake |
| `p3a_0` | kpi | evidence | 76.38 min | the design loop's Monte Carlo of the selected family mtq_fmr (mc_dispatch_ais_3u): detumble_time, the worst of 12 runs |
| `p3k_0` | kpi | stated | 284.0 Minute | the case, req.detumble |
| `p3k_1` | kpi | stated | 95.0 Minute | the case, req.sunacq |
| `p4a_1` | kpi | evidence | 0.3554 W | the design loop's Monte Carlo of the selected family mtq_fmr (mc_dispatch_ais_3u): power_mean, the worst of 12 runs |
| `p4k_0` | kpi | stated | 1.6 Kilogram | the case, req.mass |
| `p4k_1` | kpi | stated | 0.5 Watt | the case, req.pavg |
| `p4k_2` | kpi | stated | 1.5 Watt | the case, req.ppk |
| `p4k_3` | kpi | stated | 1.0 Litre | the case, req.vol |
| `s1_0` | dyn | stated | 4.0 Kilogram | the case, mass.m |
| `s1_1` | dyn | stated | 0.042 KilogramSquareMetre | the case, mass.imax |
| `s1_2` | dyn | stated | 0.042 KilogramSquareMetre | the case, mass.iint |
| `s1_3` | dyn | stated | 0.0067 KilogramSquareMetre | the case, mass.imin |
| `s2_0` | dyn | stated | 0.034 SquareMetre | the case, surface.afr |
| `s2_1` | dyn | stated | 0.02 Metre | the case, surface.cpa |
| `s2_2` | dyn | stated | 0.034 SquareMetre | the case, surface.asun |
| `s2_3` | dyn | stated | 0.02 Metre | the case, surface.cps |
| `s2_4` | dyn | stated | 0.6 One | the case, surface.refl |
| `s2_5` | dyn | stated | 2.2 One | the case, surface.cd |
| `s3_0` | dyn | stated | 0.01 AmpereSquareMetre | the case, magnetic.dres |

**Not computed (669), by why:**

- no value, relation or pseudocode yet: 556 (act_cmg_model, act_vscmg_gimbal_limits, act_vscmg_model, cf_1, cf_4, cf_5, ci1_0, ci1_1, ci1_2, ci1_3, ci1_4, cm1_0, …)
- needs an input that has no value: 36 (act_cmg_axes, act_cmg_steering, dyn_kinematics, fdir_rotor_health, fdir_safe_mode, fdir_sensor_health, gc_2, gd_3, gd_4, gd_5, gdn_boresight_offset, gdn_guidance, …)
- a relation, but no pseudocode yet: its author writes it: 32 (cf_0, cf_2, cf_3, gc_0, gc_1, gf_0, gf_1, gf_2, gf_3, gf_4, gf_5, gm_0, …)
- the case does not state pointing.et: 1 (gp_3)
- the case does not state mission.life: 1 (m1_0)
- the case does not state mission.duty: 1 (m1_2)
- evidence: no engine campaign of this case judges a metric against req.rpe: 1 (p1a_2)
- evidence: no engine campaign of this case judges a metric against req.pde: 1 (p1a_3)
- evidence: no engine campaign of this case judges a metric against req.rks: 1 (p1a_4)
- evidence: no engine campaign of this case judges a metric against req.rke: 1 (p1a_5)
- the case does not state req.rpe: 1 (p1k_2)
- the case does not state req.pde: 1 (p1k_3)
- the case does not state req.rks: 1 (p1k_4)
- the case does not state req.rke: 1 (p1k_5)
- evidence: no engine campaign of this case judges a metric against req.slew: 1 (p2a_0)
- evidence: no engine campaign of this case judges a metric against req.settle: 1 (p2a_1)
- evidence: no engine campaign of this case judges a metric against req.spo: 1 (p2a_2)
- evidence: no engine campaign of this case judges a metric against req.track: 1 (p2a_3)
- evidence: no engine campaign of this case judges a metric against req.wmax: 1 (p2a_4)
- the case does not state req.slew: 1 (p2k_0)
- the case does not state req.settle: 1 (p2k_1)
- the case does not state req.spo: 1 (p2k_2)
- the case does not state req.track: 1 (p2k_3)
- the case does not state req.wmax: 1 (p2k_4)
- evidence: no engine campaign of this case judges a metric against req.sunacq: 1 (p3a_1)
- evidence: no engine campaign of this case judges a metric against req.faults: 1 (p3a_2)
- evidence: no engine campaign of this case judges a metric against req.recover: 1 (p3a_3)
- evidence: no engine campaign of this case judges a metric against req.hsat: 1 (p3a_4)
- evidence: no engine campaign of this case judges a metric against req.dump: 1 (p3a_5)
- the case does not state req.faults: 1 (p3k_2)
- the case does not state req.recover: 1 (p3k_3)
- the case does not state req.hsat: 1 (p3k_4)
- the case does not state req.dump: 1 (p3k_5)
- evidence: no engine campaign of this case judges a metric against req.mass: 1 (p4a_0)
- evidence: no engine campaign of this case judges a metric against req.ppk: 1 (p4a_2)
- evidence: no engine campaign of this case judges a metric against req.vol: 1 (p4a_3)
- evidence: no engine campaign of this case judges a metric against req.prop: 1 (p4a_4)
- the case does not state req.prop: 1 (p4k_4)
- the case does not state mass.cm: 1 (s1_4)
- the case does not state mass.iunc: 1 (s1_5)
- the case does not state magnetic.dunc: 1 (s3_1)
- the case does not state flex.fmode: 1 (s4_0)
- the case does not state flex.mpart: 1 (s4_1)
- the case does not state resources.palloc: 1 (s5_0)
- the case does not state resources.malloc: 1 (s5_1)
- the case does not state resources.valloc: 1 (s5_2)
- the case does not state resources.vbus: 1 (s5_3)
- the case does not state resources.nif: 1 (s5_4)

## ais_img_3u

From the node files (no group has released yet). Rows: 14 computed, 4 evidence, 664 not computed, 30 stated. Closures: 34 blocked, 4 pass.

| Closure | KPI | Answer | Why |
|---|---|---|---|
| `kpi_absolute_pointing_error_ape_verified` | Absolute pointing error (APE) | **pass** | p1a_0 = 0.00542 deg <= 0.01 Degree (p1k_0) |
| `kpi_absolute_pointing_error_ape_analysis` | Absolute pointing error (APE) | blocked | gp_5 has no value (needs terms (no input names it)) |
| `kpi_absolute_knowledge_error_ake_verified` | Absolute knowledge error (AKE) | **pass** | p1a_1 = 0.004819 deg <= 0.005 Degree (p1k_1) |
| `kpi_absolute_knowledge_error_ake_analysis` | Absolute knowledge error (AKE) | blocked | ge_5 has no value (no value, relation or pseudocode yet) |
| `kpi_relative_pointing_error_rpe_verified` | Relative pointing error (RPE) | blocked | the requirement p1k_2 has no value (the case does not state req.rpe) |
| `kpi_relative_pointing_error_rpe_analysis` | Relative pointing error (RPE) | blocked | the requirement p1k_2 has no value (the case does not state req.rpe) |
| `kpi_pointing_drift_error_pde_verified` | Pointing drift error (PDE) | blocked | the requirement p1k_3 has no value (the case does not state req.pde) |
| `kpi_pointing_drift_error_pde_analysis` | Pointing drift error (PDE) | blocked | the requirement p1k_3 has no value (the case does not state req.pde) |
| `kpi_rate_stability_verified` | Rate stability | blocked | p1a_4 has no value (evidence: no engine campaign of this case judges a metric against req.rks) |
| `kpi_relative_knowledge_error_rke_verified` | Relative knowledge error (RKE) | blocked | the requirement p1k_5 has no value (the case does not state req.rke) |
| `kpi_reference_slew_time_verified` | Reference slew time | blocked | p2a_0 has no value (evidence: no engine campaign of this case judges a metric against req.slew) |
| `kpi_reference_slew_time_analysis` | Reference slew time | blocked | gc_5 has no value (no value, relation or pseudocode yet) |
| `kpi_settling_time_after_a_slew_verified` | Settling time after a slew | blocked | p2a_1 has no value (evidence: no engine campaign of this case judges a metric against req.settle) |
| `kpi_settling_time_after_a_slew_analysis` | Settling time after a slew | blocked | gc_2 has no value (needs w_n from gc_0, zeta from gc_1 (a loop, or rows not computed)) |
| `kpi_slews_per_orbit_verified` | Slews per orbit | blocked | the requirement p2k_2 has no value (the case does not state req.spo) |
| `kpi_target_tracking_rate_verified` | Target-tracking rate | blocked | the requirement p2k_3 has no value (the case does not state req.track) |
| `kpi_maximum_body_rate_verified` | Maximum body rate | blocked | the requirement p2k_4 has no value (the case does not state req.wmax) |
| `kpi_detumble_time_verified` | Detumble time | **pass** | p3a_0 = 79.89 min <= 284.0 Minute (p3k_0) |
| `kpi_detumble_time_analysis` | Detumble time | blocked | gq_0 has no value (no value, relation or pseudocode yet) |
| `kpi_sun_acquisition_time_in_safe_mode_verified` | Sun-acquisition time in safe mode | blocked | p3a_1 has no value (evidence: no engine campaign of this case judges a metric against req.sunacq) |
| `kpi_sun_acquisition_time_in_safe_mode_analysis` | Sun-acquisition time in safe mode | blocked | gq_1 has no value (no value, relation or pseudocode yet) |
| `kpi_faults_tolerated_verified` | Faults tolerated | blocked | the requirement p3k_2 has no value (the case does not state req.faults) |
| `kpi_faults_tolerated_analysis` | Faults tolerated | blocked | the requirement p3k_2 has no value (the case does not state req.faults) |
| `kpi_recovery_time_after_a_single_fault_verified` | Recovery time after a single fault | blocked | the requirement p3k_3 has no value (the case does not state req.recover) |
| `kpi_recovery_time_after_a_single_fault_analysis` | Recovery time after a single fault | blocked | the requirement p3k_3 has no value (the case does not state req.recover) |
| `kpi_momentum_saturation_margin_verified` | Momentum saturation margin | blocked | the requirement p3k_4 has no value (the case does not state req.hsat) |
| `kpi_momentum_dump_interval_verified` | Momentum dump interval | blocked | the requirement p3k_5 has no value (the case does not state req.dump) |
| `kpi_momentum_dump_interval_analysis` | Momentum dump interval | blocked | the requirement p3k_5 has no value (the case does not state req.dump) |
| `kpi_adcs_mass_verified` | ADCS mass | blocked | p4a_0 has no value (evidence: no engine campaign of this case judges a metric against req.mass) |
| `kpi_adcs_mass_analysis` | ADCS mass | blocked | gb_0 has no value (no value, relation or pseudocode yet) |
| `kpi_adcs_orbit_average_power_verified` | ADCS orbit-average power | **pass** | p4a_1 = 0.4945 W <= 2.0 Watt (p4k_1) |
| `kpi_adcs_orbit_average_power_analysis` | ADCS orbit-average power | blocked | gb_1 has no value (no value, relation or pseudocode yet) |
| `kpi_adcs_peak_power_verified` | ADCS peak power | blocked | p4a_2 has no value (evidence: no engine campaign of this case judges a metric against req.ppk) |
| `kpi_adcs_peak_power_analysis` | ADCS peak power | blocked | gb_2 has no value (no value, relation or pseudocode yet) |
| `kpi_adcs_volume_verified` | ADCS volume | blocked | p4a_3 has no value (evidence: no engine campaign of this case judges a metric against req.vol) |
| `kpi_adcs_volume_analysis` | ADCS volume | blocked | gb_3 has no value (no value, relation or pseudocode yet) |
| `kpi_rcs_propellant_per_year_verified` | RCS propellant per year | blocked | the requirement p4k_4 has no value (the case does not state req.prop) |
| `kpi_rcs_propellant_per_year_analysis` | RCS propellant per year | blocked | the requirement p4k_4 has no value (the case does not state req.prop) |

| Row | Group | State | Value | Where from / why not |
|---|---|---|---|---|
| `gd_0` | env | computed | 6.34679e-08 (SI) | gd_0(r, i_max, i_min) |
| `gd_1` | env | computed | 1.36971e-08 (SI) | gd_1(rho, v, c_d, a_fr, c_pa) |
| `gd_2` | env | computed | 5.10763e-09 (SI) | gd_2(p_srp, a_sun, q, c_ps) |
| `gw_3` | act | computed | 0.000733038 (SI) | gw_3(i_max, theta, t_slew) |
| `gw_4` | act | computed | 2.44346e-05 (SI) | gw_4(i_max, theta, t_slew) |
| `m1_1` | case | stated | 27.0 Year | the case, mission.epoch |
| `m1_3` | case | stated | 10.0 Count | the case, mission.spd |
| `m1_4` | case | stated | 30.0 Degree | the case, mission.sangle |
| `m1_5` | case | stated | 10.0 DegreePerSecond | the case, mission.w0 |
| `m2_0` | env | stated | 550.0 Kilometre | the case, orbit.alt |
| `m2_1` | env | stated | 97.593 Degree | the case, orbit.inc |
| `m2_2` | env | stated | 0.0 One | the case, orbit.ecc |
| `m2_3` | env | stated | 10.0 Hour | the case, orbit.ltan |
| `m2_4` | env | computed | 6.92814e+06 (SI) | radius(h) |
| `m2_5` | env | computed | 5738.99 (SI) | m2_5(r) |
| `m2_6` | env | computed | 0.00109482 (SI) | mean_motion(t_orb) |
| `m3_0` | env | computed | 2.3256e-05 (SI) | m3_0(r) |
| `m3_1` | env | computed | 4.6512e-05 (SI) | m3_1(r, lambda_max) |
| `m3_2` | env | computed | 1.5708 (SI) | max_magnetic_latitude(i) |
| `m3_3` | env | computed | 3.18278e-13 (SI) | density_at(h, t_epoch) |
| `m3_4` | env | computed | 7585.09 (SI) | circular_speed(r) |
| `m3_5` | env | computed | 4.69452e-06 (SI) | m3_5(t_epoch) |
| `p1a_0` | kpi | evidence | 0.00542 deg | the design loop's Monte Carlo of the selected family mtq_fmr (mc_dispatch_ais_img_3u): ape_los_p9973, the worst of 12 runs |
| `p1a_1` | kpi | evidence | 0.004819 deg | the design loop's Monte Carlo of the selected family mtq_fmr (mc_dispatch_ais_img_3u): ake_los_p9973, the worst of 12 runs |
| `p1k_0` | kpi | stated | 0.01 Degree | the case, req.ape |
| `p1k_1` | kpi | stated | 0.005 Degree | the case, req.ake |
| `p1k_4` | kpi | stated | 0.005 DegreePerSecond | the case, req.rks |
| `p2k_0` | kpi | stated | 60.0 Second | the case, req.slew |
| `p2k_1` | kpi | stated | 20.0 Second | the case, req.settle |
| `p3a_0` | kpi | evidence | 79.89 min | the design loop's Monte Carlo of the selected family mtq_fmr (mc_dispatch_ais_img_3u): detumble_time, the worst of 12 runs |
| `p3k_0` | kpi | stated | 284.0 Minute | the case, req.detumble |
| `p3k_1` | kpi | stated | 95.0 Minute | the case, req.sunacq |
| `p4a_1` | kpi | evidence | 0.4945 W | the design loop's Monte Carlo of the selected family mtq_fmr (mc_dispatch_ais_img_3u): power_mean, the worst of 12 runs |
| `p4k_0` | kpi | stated | 1.85 Kilogram | the case, req.mass |
| `p4k_1` | kpi | stated | 2.0 Watt | the case, req.pavg |
| `p4k_2` | kpi | stated | 4.0 Watt | the case, req.ppk |
| `p4k_3` | kpi | stated | 1.0 Litre | the case, req.vol |
| `s1_0` | dyn | stated | 4.0 Kilogram | the case, mass.m |
| `s1_1` | dyn | stated | 0.042 KilogramSquareMetre | the case, mass.imax |
| `s1_2` | dyn | stated | 0.042 KilogramSquareMetre | the case, mass.iint |
| `s1_3` | dyn | stated | 0.0067 KilogramSquareMetre | the case, mass.imin |
| `s2_0` | dyn | stated | 0.034 SquareMetre | the case, surface.afr |
| `s2_1` | dyn | stated | 0.02 Metre | the case, surface.cpa |
| `s2_2` | dyn | stated | 0.034 SquareMetre | the case, surface.asun |
| `s2_3` | dyn | stated | 0.02 Metre | the case, surface.cps |
| `s2_4` | dyn | stated | 0.6 One | the case, surface.refl |
| `s2_5` | dyn | stated | 2.2 One | the case, surface.cd |
| `s3_0` | dyn | stated | 0.01 AmpereSquareMetre | the case, magnetic.dres |

**Not computed (664), by why:**

- no value, relation or pseudocode yet: 556 (act_cmg_model, act_vscmg_gimbal_limits, act_vscmg_model, cf_1, cf_4, cf_5, ci1_0, ci1_1, ci1_2, ci1_3, ci1_4, cm1_0, …)
- needs an input that has no value: 34 (act_cmg_axes, act_cmg_steering, dyn_kinematics, fdir_rotor_health, fdir_safe_mode, fdir_sensor_health, gc_2, gd_3, gd_4, gd_5, gdn_boresight_offset, gdn_guidance, …)
- a relation, but no pseudocode yet: its author writes it: 32 (cf_0, cf_2, cf_3, gc_0, gc_1, gf_0, gf_1, gf_2, gf_3, gf_4, gf_5, gm_0, …)
- the case does not state pointing.et: 1 (gp_3)
- the case does not state mission.life: 1 (m1_0)
- the case does not state mission.duty: 1 (m1_2)
- evidence: no engine campaign of this case judges a metric against req.rpe: 1 (p1a_2)
- evidence: no engine campaign of this case judges a metric against req.pde: 1 (p1a_3)
- evidence: no engine campaign of this case judges a metric against req.rks: 1 (p1a_4)
- evidence: no engine campaign of this case judges a metric against req.rke: 1 (p1a_5)
- the case does not state req.rpe: 1 (p1k_2)
- the case does not state req.pde: 1 (p1k_3)
- the case does not state req.rke: 1 (p1k_5)
- evidence: no engine campaign of this case judges a metric against req.slew: 1 (p2a_0)
- evidence: no engine campaign of this case judges a metric against req.settle: 1 (p2a_1)
- evidence: no engine campaign of this case judges a metric against req.spo: 1 (p2a_2)
- evidence: no engine campaign of this case judges a metric against req.track: 1 (p2a_3)
- evidence: no engine campaign of this case judges a metric against req.wmax: 1 (p2a_4)
- the case does not state req.spo: 1 (p2k_2)
- the case does not state req.track: 1 (p2k_3)
- the case does not state req.wmax: 1 (p2k_4)
- evidence: no engine campaign of this case judges a metric against req.sunacq: 1 (p3a_1)
- evidence: no engine campaign of this case judges a metric against req.faults: 1 (p3a_2)
- evidence: no engine campaign of this case judges a metric against req.recover: 1 (p3a_3)
- evidence: no engine campaign of this case judges a metric against req.hsat: 1 (p3a_4)
- evidence: no engine campaign of this case judges a metric against req.dump: 1 (p3a_5)
- the case does not state req.faults: 1 (p3k_2)
- the case does not state req.recover: 1 (p3k_3)
- the case does not state req.hsat: 1 (p3k_4)
- the case does not state req.dump: 1 (p3k_5)
- evidence: no engine campaign of this case judges a metric against req.mass: 1 (p4a_0)
- evidence: no engine campaign of this case judges a metric against req.ppk: 1 (p4a_2)
- evidence: no engine campaign of this case judges a metric against req.vol: 1 (p4a_3)
- evidence: no engine campaign of this case judges a metric against req.prop: 1 (p4a_4)
- the case does not state req.prop: 1 (p4k_4)
- the case does not state mass.cm: 1 (s1_4)
- the case does not state mass.iunc: 1 (s1_5)
- the case does not state magnetic.dunc: 1 (s3_1)
- the case does not state flex.fmode: 1 (s4_0)
- the case does not state flex.mpart: 1 (s4_1)
- the case does not state resources.palloc: 1 (s5_0)
- the case does not state resources.malloc: 1 (s5_1)
- the case does not state resources.valloc: 1 (s5_2)
- the case does not state resources.vbus: 1 (s5_3)
- the case does not state resources.nif: 1 (s5_4)

