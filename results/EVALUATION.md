# Every row, every closure, from the design database

**In one line:** for each case, every row of the design with its value (stated by the case or the design, computed by its pseudocode, or supplied by a campaign) or why it has none, and every KPI closure answered or blocked by name (`tools/evaluate.py`, `docs/END_TO_END.md`).

## ais_3u

From the merged releases (design.tndb). Rows: 55 computed, 4 evidence, 992 not computed, 171 stated. Closures: 34 blocked, 4 pass.

| Closure | KPI | Answer | Why |
|---|---|---|---|
| `kpi_absolute_pointing_error_ape_verified` | Absolute pointing error (APE) | **pass** | p1a_0 = 0.5455 deg <= 10.0 Degree (p1k_0) |
| `kpi_absolute_pointing_error_ape_analysis` | Absolute pointing error (APE) | blocked | gp_5 has no value (needs terms[1] from gp_0 (not computed), terms[2] from gp_1 (not computed), terms[3] from gp_2 (not computed), terms[4] from gp_3 (not computed), terms[5] from gp_4 (not computed)) |
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
| `kpi_adcs_mass_analysis` | ADCS mass | blocked | gb_0 has no value (its module has 2 functions and names none for this node (by its id or its output's symbol)) |
| `kpi_adcs_orbit_average_power_verified` | ADCS orbit-average power | **pass** | p4a_1 = 0.3554 W <= 0.5 Watt (p4k_1) |
| `kpi_adcs_orbit_average_power_analysis` | ADCS orbit-average power | blocked | gb_1 has no value (its module has 2 functions and names none for this node (by its id or its output's symbol)) |
| `kpi_adcs_peak_power_verified` | ADCS peak power | blocked | p4a_2 has no value (evidence: no engine campaign of this case judges a metric against req.ppk) |
| `kpi_adcs_peak_power_analysis` | ADCS peak power | blocked | gb_2 has no value (its module has 2 functions and names none for this node (by its id or its output's symbol)) |
| `kpi_adcs_volume_verified` | ADCS volume | blocked | p4a_3 has no value (evidence: no engine campaign of this case judges a metric against req.vol) |
| `kpi_adcs_volume_analysis` | ADCS volume | blocked | gb_3 has no value (its module has 2 functions and names none for this node (by its id or its output's symbol)) |
| `kpi_rcs_propellant_per_year_verified` | RCS propellant per year | blocked | the requirement p4k_4 has no value (the case does not state req.prop) |
| `kpi_rcs_propellant_per_year_analysis` | RCS propellant per year | blocked | the requirement p4k_4 has no value (the case does not state req.prop) |

| Row | Group | State | Value | Where from / why not |
|---|---|---|---|---|
| `act_cmg_speed_gain` | act | stated | 1.0 | the design, 1.0.0 code: engine/crates/adcs-sim/src/product.rs Dev::load (MexDesc k_speed = 1.0, the device default every product flies) |
| `act_fmr_flow_gain` | act | stated | 2.0 | the design, 1.0.0 code: engine/crates/adcs-sim/src/product.rs Dev::load (MexDesc k_flow = 2.0, the device default every product flies) |
| `act_fmr_flow_tau` | act | stated | 0.3 | the design, 1.0.0 code: engine/crates/adcs-sim/src/product.rs Dev::load (MexDesc flow_tau = 0.3, the device default every product flies) |
| `act_gimbal_tlm_noise` | act | stated | 1e-5 | the design, 1.0.0 code: engine/crates/adcs-sim/src/run.rs sense (x.d[g] + 1e-5 normal, the rotor telemetry the CAN frames carry) |
| `act_rotor_tlm_noise` | act | stated | 1e-7 | the design, 1.0.0 code: engine/crates/adcs-sim/src/run.rs sense (x.h[i] + 1e-7 normal, the rotor telemetry the CAN frames carry) |
| `act_rw_drive_efficiency` | act | stated | 0.8 | the design, 1.0.0 code: engine/crates/adcs-sim/src/product.rs Dev::load (MexDesc eta = 0.8, the device default every product flies) |
| `act_rw_friction_comp` | act | stated | 0.95 | the design, 1.0.0 code: engine/crates/adcs-sim/src/product.rs Dev::load (MexDesc friction_comp = 0.95, the device default every product flies) |
| `act_rw_torque_noise` | act | stated | 0.001 | the design, 1.0.0 code: engine/crates/adcs-sim/src/product.rs Dev::load (MexDesc torque_noise = 0.001, the device default every product flies) |
| `cf_0` | systems | stated | 3 Count | the design, adcs_ref_c1 |
| `cf_2` | systems | stated | 3 Count | the design, adcs_ref_c1 |
| `cf_3` | systems | stated | 0 Count | the design, adcs_ref_c1 |
| `dyn_cm_direction_x` | dyn | stated | 0.30 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (cmd = [0.30, 0.70, -0.65], the direction the case's surface.cpa offset takes) |
| `dyn_cm_direction_y` | dyn | stated | 0.70 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (cmd = [0.30, 0.70, -0.65], the direction the case's surface.cpa offset takes) |
| `dyn_cm_direction_z` | dyn | stated | -0.65 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (cmd = [0.30, 0.70, -0.65], the direction the case's surface.cpa offset takes) |
| `dyn_initial_attitude_kind_default` | dyn | stated | 0 | the design, 1.0.0 code: engine/crates/adcs-sim/src/run.rs initial_state (the kind's match: _ => ATTSTART_NADIR) |
| `dyn_initial_error_angle` | dyn | stated | 0 | the design, 1.0.0 code: engine/crates/adcs-sim/src/run.rs initial_state (json::f(attitude, angle_deg, 0.0)) |
| `dyn_initial_error_axis` | dyn | stated | [1, 0, 0] | the design, 1.0.0 code: engine/crates/adcs-sim/src/run.rs initial_state (unwrap_or([1.0, 0.0, 0.0])) |
| `dyn_initial_rate_extra` | dyn | stated | 0 | the design, 1.0.0 code: engine/crates/adcs-sim/src/run.rs initial_state (json::f(rate, extra_deg_s, 0.0)) |
| `dyn_initial_rate_kind_default` | dyn | stated | 0 | the design, 1.0.0 code: engine/crates/adcs-sim/src/run.rs initial_state (the kind's match: _ => RATESTART_VALUE) |
| `dyn_initial_rate_magnitude` | dyn | stated | 0 | the design, 1.0.0 code: engine/crates/adcs-sim/src/run.rs initial_state (magnitude_deg_s: None => 0.0) |
| `dyn_initial_rate_value` | dyn | stated | [0, 0, 0] | the design, 1.0.0 code: engine/crates/adcs-sim/src/run.rs initial_state (unwrap_or([0.0; 3])) |
| `dyn_surface_accommodation` | dyn | stated | 0.8 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs ACCOMMODATION (its comment: Moe and Moe 2005, LEO) |
| `dyn_surface_specular_share` | dyn | stated | 0.5 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs SPEC_FRAC |
| `dyn_surface_vb_ratio` | dyn | stated | 0.05 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs VB_RATIO (its comment: Moe and Moe 2005, LEO) |
| `dyn_truth_plant` | dyn | computed | [[0.0067, 0, 0], [0, 0.042, 0], [0, 0, 0.042]] (SI) | principal_inertia(imin, iint, imax) |
| `env_ap_default` | env | stated | 7 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (ap: 7.0) |
| `env_case_orbit` | env | computed | 5738.99 (SI) | case_mean_motion(h) -> t |
| `env_density_scale_default` | env | stated | 1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (density_scale: 1.0) |
| `env_f107_default` | env | stated | 130 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (f107: 130.0) |
| `env_f107a_default` | env | stated | 130 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (f107a: 130.0) |
| `env_fast_zonal_degree` | env | stated | 6 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (zonal_max: 6) |
| `env_field_degree` | env | stated | 13 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (igrf_nmax: 13) |
| `env_force_drag` | env | stated | 1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (drag: true) |
| `env_force_srp` | env | stated | 1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (srp: true) |
| `env_force_third_body` | env | stated | 1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (third_body: true) |
| `env_kp_default` | env | stated | 2 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (kp: 2.0) |
| `env_orbit_precision_default` | env | stated | 1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (orbit_model: "pop") |
| `env_orbit_step` | env | stated | 10 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (orbit_step_s: 10.0) |
| `env_refresh_step` | env | stated | 1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (env_dt_s: 1.0) |
| `env_start_arg_lat_default` | env | stated | 0 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (json::f(initial, arg_lat_deg, 0.0)) |
| `env_torque_aero` | env | stated | 1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (env_on: [true; 4], the second) |
| `env_torque_gravity_gradient` | env | stated | 1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (env_on: [true; 4], the first) |
| `env_torque_magnetic` | env | stated | 1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (env_on: [true; 4], the fourth) |
| `env_torque_radiation` | env | stated | 1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (env_on: [true; 4], the third) |
| `fsw_param_J` | fsw | computed | [[0.0067, 0, 0], [0, 0.042, 0], [0, 0, 0.042]] (SI) | fsw_body_model(j, m_res, sun_axis, boresight) -> jf |
| `fsw_param_auto_next` | fsw | stated | 255 | the design, the scenario's fsw.auto_next; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs modes_and_laws (none when it names none: MODE_NONE, 255) |
| `fsw_param_bdot_k` | fsw | computed | 8.76379e-05 (SI) | fsw_bdot_gain(scale, n, inc_deg, ii) |
| `fsw_param_capture_deg` | fsw | stated | 3 | the design, the scenario's fsw.capture_deg; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs rw_gains (json::f(fsw, capture_deg, 3.0)) |
| `fsw_param_capture_rate_deg_s` | fsw | stated | 1 | the design, the scenario's fsw.capture_rate_deg_s; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs rw_gains (json::f(fsw, capture_rate_deg_s, 1.0)) |
| `fsw_param_cmg_k_null` | fsw | stated | 0.002 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs rotor_params (p.cmg_k_null = 0.002) |
| `fsw_param_cmg_lam0` | fsw | stated | 1e-09 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs rotor_params (p.cmg_lam0 = 1e-9) |
| `fsw_param_cmg_mu` | fsw | stated | 10 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs rotor_params (p.cmg_mu = 10.0) |
| `fsw_param_detumble_exit` | fsw | computed | 0.00872665 (SI) | fsw_rate_rad(deg_s) |
| `fsw_param_detumble_hold_s` | fsw | stated | 60 | the design, the scenario's fsw.detumble_hold_s; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs mtq_gains (json::f(fsw, detumble_hold_s, 60.0)) |
| `fsw_param_dt` | fsw | stated | 0.1 | the design, the scenario's time.dt_s; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs build (json::f(time, dt_s, 0.1)) |
| `fsw_param_dump_k` | fsw | stated | 0.002 | the design, the scenario's fsw.dump_gain; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs rotor_params (json::f(fsw, dump_gain, 2e-3)) |
| `fsw_param_fdir_h_frac` | fsw | stated | 0.005 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs rotor_params (p.fdir_h_frac = 0.005: a commanded change under 0.5 % of h_max is not judged) |
| `fsw_param_fdir_s` | fsw | stated | 3 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs rotor_params (p.fdir_s = 3.0) |
| `fsw_param_fdir_win_s` | fsw | stated | 120 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs rotor_params (p.fdir_win_s = 120.0: 120 s windows) |
| `fsw_param_gd_T` | fsw | stated | 1 | the design, the scenario's fsw.guidance.T_s; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs guidance_params (json::f(guidance, T_s, 1.0)) |
| `fsw_param_gd_axis` | fsw | stated | [0, 0, 0] | the design, the scenario's fsw.guidance.axis; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs guidance_params (unwrap_or([0.0; 3])) |
| `fsw_param_gd_flip_hyst` | fsw | stated | 0.1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs sensor_params (p.gd_flip_hyst = 0.1) |
| `fsw_param_gd_kind` | fsw | stated | 0 | the design, the scenario's fsw.guidance.kind; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs guidance_params (json::s(guidance, kind, nadir): nadir, fswchoice's GuidKind 0) |
| `fsw_param_gd_q_inertial` | fsw | stated | [0, 0, 0, 1] | the design, the scenario's fsw.guidance.q_inertial; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs guidance_params (unwrap_or([0.0, 0.0, 0.0, 1.0])) |
| `fsw_param_gd_q_off` | fsw | computed | [0, 0, 0, 1] (SI) | fsw_payload_offset(boresight) |
| `fsw_param_gd_roll_deg` | fsw | stated | 0 | the design, the scenario's fsw.guidance.roll_deg; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs guidance_params (json::f(guidance, roll_deg, 0.0)) |
| `fsw_param_gd_t0` | fsw | stated | 0 | the design, the scenario's fsw.guidance.t0; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs guidance_params (json::f(guidance, t0, 0.0)) |
| `fsw_param_gd_yaw_flip` | fsw | stated | 1 | the design, the scenario's fsw.yaw_flip; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs sensor_params (json::b(fsw, yaw_flip, true)) |
| `fsw_param_gnss_ecef` | fsw | stated | 1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs sensor_params (p.gnss_ecef = 1: the receiver's standard frame, converted onboard) |
| `fsw_param_h_bias` | fsw | stated | 0.002 | the design, the scenario's fsw.wheel_bias_Nms; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs rotor_params (json::f(fsw, wheel_bias_Nms, 2e-3)) |
| `fsw_param_ho_hold_s` | fsw | stated | 60 | the design, the scenario's fsw.handover_hold_s; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs mtq_gains (json::f(fsw, handover_hold_s, 60.0)) |
| `fsw_param_ho_in_dps` | fsw | stated | 1 | the design, the scenario's fsw.handover_in_dps; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs mtq_gains (json::f(fsw, handover_in_dps, 1.0)) |
| `fsw_param_ho_out_dps` | fsw | stated | 0.5 | the design, the scenario's fsw.handover_out_dps; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs mtq_gains (json::f(fsw, handover_out_dps, 0.5)) |
| `fsw_param_igrf_nmax` | fsw | stated | 10 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs sensor_params (p.igrf_nmax = 10) |
| `fsw_param_jd0` | fsw | computed | 2.46141e+06 (SI) | fsw_epoch_earth(jd) -> jd0 |
| `fsw_param_m_res_est` | fsw | computed | [0.0057735, 0.0057735, 0.0057735] (SI) | fsw_body_model(j, m_res, sun_axis, boresight) -> mf |
| `fsw_param_mekf_gate` | fsw | stated | 16.27 | the design, the scenario's fsw.mekf_gate; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs sensor_params (json::f(fsw, mekf_gate, 16.27)) |
| `fsw_param_mekf_meas_scale` | fsw | stated | 1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs sensor_params (p.mekf_meas_scale = 1.0) |
| `fsw_param_mekf_rej_max` | fsw | stated | 30 | the design, the scenario's fsw.mekf_rej_max; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs sensor_params (json::f(fsw, mekf_rej_max, 30.0)) |
| `fsw_param_mtq_Gs` | fsw | computed | [1e-05, 1e-05, 1e-05] (SI) | fsw_mtq_smc(wn, z, phi) -> gs |
| `fsw_param_mtq_Kd` | fsw | computed | [0.000134, 0.00084, 0.00084] (SI) | fsw_mtq_pd(ii, wn, z) -> kd |
| `fsw_param_mtq_Ki` | fsw | computed | [0, 0, 0] (SI) | fsw_mtq_pd(ii, wn, z) -> ki |
| `fsw_param_mtq_Kp` | fsw | computed | [1.675e-07, 1.05e-06, 1.05e-06] (SI) | fsw_mtq_pd(ii, wn, z) -> kp |
| `fsw_param_mtq_Pth` | fsw | computed | [[4.98567e-07, 0, 0], [0, 1.27604e-06, 0], [0, 0, 1.27604e-06]] (SI) | fsw_mtq_tango(ii, wn, z, gp, gd) -> pth |
| `fsw_param_mtq_Pw` | fsw | computed | [[0.000398799, 0, 0], [0, 0.00102111, 0], [0, 0, 0.00102111]] (SI) | fsw_mtq_tango(ii, wn, z, gp, gd) -> pw |
| `fsw_param_mtq_eps` | fsw | stated | 0.001 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs mtq_gains (let eps = 1e-3, the papers' time-scale parameter) |
| `fsw_param_mtq_err_max` | fsw | stated | 0.5 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs mtq_gains (p.mtq_err_max = 0.5) |
| `fsw_param_mtq_gg_ff` | fsw | computed | 3 (SI) | fsw_gg_feedforward(q_off, j) |
| `fsw_param_mtq_int_max` | fsw | stated | 0.05 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs mtq_gains (p.mtq_int_max = 0.05) |
| `fsw_param_mtq_lambda` | fsw | computed | 0.0025 (SI) | fsw_mtq_smc(wn, z, phi) -> lambda |
| `fsw_param_mtq_meas` | fsw | stated | 0.2 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs mtq_gains (p.mtq_meas = 0.2) |
| `fsw_param_mtq_period` | fsw | stated | 1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs mtq_gains (p.mtq_period = 1.0) |
| `fsw_param_mtq_phi` | fsw | stated | 0.0005 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs mtq_gains (p.mtq_phi = 5e-4) |
| `fsw_param_mu` | fsw | computed | 3.986e+14 (SI) | fsw_epoch_earth(jd) -> mu |
| `fsw_param_rate_lpf_s` | fsw | stated | 0.3 | the design, the scenario's fsw.rate_lpf_s; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs sensor_params (json::f(fsw, rate_lpf_s, 0.3)) |
| `fsw_param_rcs_assist` | fsw | stated | 1 | the design, the scenario's fsw.rcs_assist; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs rcs_params (json::b(fsw, rcs_assist, true)) |
| `fsw_param_rcs_assist_frac` | fsw | stated | 0.8 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs rcs_params (p.rcs_assist_frac = 0.8) |
| `fsw_param_rcs_dump` | fsw | stated | 1 | the design, the scenario's fsw.rcs_dump; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs rcs_params (json::b(fsw, rcs_dump, true)) |
| `fsw_param_rcs_dump_hi` | fsw | stated | 0.004 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs rcs_params (p.rcs_dump_hi = 4e-3) |
| `fsw_param_rcs_dump_k` | fsw | stated | 0.05 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs rcs_params (p.rcs_dump_k = 0.05) |
| `fsw_param_rcs_dump_lo` | fsw | stated | 0.001 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs rcs_params (p.rcs_dump_lo = 1e-3) |
| `fsw_param_rcsd_T_damp_s` | fsw | stated | 20 | the design, the scenario's fsw.rcs_damp_s; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs rcs_params (json::f(fsw, rcs_damp_s, 20.0)) |
| `fsw_param_rcsd_deadband_deg_s` | fsw | stated | 0.2 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs rcs_params (p.rcsd_deadband_deg_s = 0.2) |
| `fsw_param_rcsd_period_s` | fsw | stated | 1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs rcs_params (p.rcsd_period_s = 1.0) |
| `fsw_param_roll_axis` | fsw | computed | [0, 1, 0] (SI) | fsw_body_model(j, m_res, sun_axis, boresight) -> roll |
| `fsw_param_rw_Gs` | fsw | computed | [0.00072, 0.00072, 0.00072] (SI) | fsw_rw_smc(wn, z, phi) -> gs |
| `fsw_param_rw_Kd` | fsw | computed | [0.02412, 0.1512, 0.1512] (SI) | fsw_rw_pid(ii, wn, z) -> kd |
| `fsw_param_rw_Ki` | fsw | computed | [0.000732645, 0.0045927, 0.0045927] (SI) | fsw_rw_pid(ii, wn, z) -> ki |
| `fsw_param_rw_Kp` | fsw | computed | [0.005427, 0.03402, 0.03402] (SI) | fsw_rw_pid(ii, wn, z) -> kp |
| `fsw_param_rw_dt` | fsw | computed | 0.1 (SI) | fsw_rw_period(rate_hz) |
| `fsw_param_rw_err_max` | fsw | stated | 0.2 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs rw_gains (p.rw_err_max = 0.2) |
| `fsw_param_rw_int_max` | fsw | stated | 0.02 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs rw_gains (p.rw_int_max = 0.02) |
| `fsw_param_rw_lambda` | fsw | computed | 0.225 (SI) | fsw_rw_smc(wn, z, phi) -> lambda |
| `fsw_param_rw_phi` | fsw | stated | 0.0002 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs rw_gains (p.rw_phi = 2e-4) |
| `fsw_param_sa_done_deg` | fsw | stated | 10 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs spin_params (p.sa_done_deg = 10.0) |
| `fsw_param_sa_done_hold_s` | fsw | stated | 60 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs spin_params (p.sa_done_hold_s = 60.0) |
| `fsw_param_sa_kd` | fsw | stated | 0.1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs spin_params (p.sa_kd = 0.1) |
| `fsw_param_sa_w_max_deg_s` | fsw | stated | 1 | the design, the scenario's fsw.sun_acq_rate_deg_s; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs spin_params (json::f(fsw, sun_acq_rate_deg_s, 1.0)) |
| `fsw_param_sb_kdroll` | fsw | computed | 0.000275896 (SI) | fsw_mtq_roll(e, j, n, roll_wn_orbits, roll_zeta, roll_gain) -> kdroll |
| `fsw_param_sb_kroll` | fsw | computed | 4.53086e-07 (SI) | fsw_mtq_roll(e, j, n, roll_wn_orbits, roll_zeta, roll_gain) -> kroll |
| `fsw_param_sb_roll_gate` | fsw | computed | 0.965926 (SI) | fsw_roll_gate(roll_gate_deg) |
| `fsw_param_ss_dr_k` | fsw | computed | 0.01 (SI) | fsw_sun_spin_gains(g, jzz) -> dr_k |
| `fsw_param_ss_dr_k1` | fsw | stated | 1.5 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs spin_params (p.ss_dr_k1 = 1.5; de Ruiter 2011: k1 > 1, the paper's 1.5) |
| `fsw_param_ss_dr_k2` | fsw | computed | 0.021 (SI) | fsw_sun_spin_gains(g, jzz) -> dr_k2 |
| `fsw_param_ss_dwell_in_s` | fsw | stated | 60 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs spin_params (p.ss_dwell_in_s = 60.0) |
| `fsw_param_ss_dwell_out_s` | fsw | stated | 30 | the design, the scenario's fsw.sun_spin_dwell_out_s; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs spin_params (json::f(fsw, sun_spin_dwell_out_s, 30.0)) |
| `fsw_param_ss_k1` | fsw | computed | 0.01 (SI) | fsw_sun_spin_gains(g, jzz) -> k1 |
| `fsw_param_ss_k2` | fsw | computed | 0.05 (SI) | fsw_sun_spin_gains(g, jzz) -> k2 |
| `fsw_param_ss_k_l1` | fsw | stated | 1000000 | the design, the scenario's fsw.l1_gain; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs spin_params (json::f(fsw, l1_gain, 1e6)) |
| `fsw_param_ss_omega_exit_dps` | fsw | stated | 2 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs spin_params (p.ss_omega_exit_dps = 2.0) |
| `fsw_param_ss_omega_max_dps` | fsw | stated | 100 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs spin_params (p.ss_omega_max_dps = 100.0) |
| `fsw_param_ss_perp_in_dps` | fsw | stated | 0.5 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs spin_params (p.ss_perp_in_dps = 0.5) |
| `fsw_param_ss_perp_out_dps` | fsw | stated | 1 | the design, the scenario's fsw.sun_spin_perp_out_dps; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs spin_params (json::f(fsw, sun_spin_perp_out_dps, 1.0)) |
| `fsw_param_ss_sigma0` | fsw | stated | 1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs spin_params (p.ss_sigma0 = 1.0) |
| `fsw_param_ss_spin_dps` | fsw | stated | 6 | the design, the scenario's fsw.spin_rate_dps; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs spin_params (json::f(fsw, spin_rate_dps, 6.0)), and build (spin_dps) |
| `fsw_param_ss_sun_min` | fsw | stated | 0.05 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs spin_params (p.ss_sun_min = 0.05) |
| `fsw_param_ss_t_check_s` | fsw | stated | 60 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs spin_params (p.ss_t_check_s = 60.0) |
| `fsw_param_ss_z_in_dps` | fsw | stated | 0.5 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs spin_params (p.ss_z_in_dps = 0.5) |
| `fsw_param_st_coast_s` | fsw | stated | 900 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs sensor_params (p.st_coast_s = 900.0) |
| `fsw_param_start_mode` | fsw | stated | 0 | the design, the scenario's fsw.start_mode; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs modes_and_laws (json::s(fsw, start_mode, detumble): detumble, state 0 of adcs_mode_t) |
| `fsw_param_sun_axis` | fsw | computed | [0, 0, -1] (SI) | fsw_body_model(j, m_res, sun_axis, boresight) -> sun |
| `fsw_tune_avanzini_k_over_n` | fsw | stated | 0.84 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs mtq_gains (json::f(fsw, avanzini_k_over_n, 0.84)) |
| `fsw_tune_avanzini_lambda` | fsw | stated | 0.08 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs mtq_gains (json::f(fsw, avanzini_lambda, 0.08)) |
| `fsw_tune_bdot_gain_scale` | fsw | stated | 3 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs mtq_gains (json::f(fsw, bdot_gain_scale, 3.0)) |
| `fsw_tune_detumble_exit_deg_s` | fsw | stated | 0.5 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs mtq_gains (json::f(fsw, detumble_exit_deg_s, 0.5)) |
| `fsw_tune_mtq_gain_d` | fsw | stated | 1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs mtq_gains (json::f(fsw, mtq_gain_d, 1.0)) |
| `fsw_tune_mtq_gain_p` | fsw | stated | 1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs mtq_gains (json::f(fsw, mtq_gain_p, 1.0)) |
| `fsw_tune_mtq_wn` | fsw | stated | 0.005 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs mtq_gains (json::f(fsw, mtq_wn, 0.005)) |
| `fsw_tune_mtq_zeta` | fsw | stated | 2 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs mtq_gains (json::f(fsw, mtq_zeta, 2.0)) |
| `fsw_tune_roll_gain` | fsw | stated | 1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs mtq_gains (json::f(fsw, roll_gain, 1.0)) |
| `fsw_tune_roll_gate_deg` | fsw | stated | 15 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs mtq_gains (json::f(fsw, roll_gate_deg, 15.0)) |
| `fsw_tune_roll_wn_orbits` | fsw | stated | 3 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs mtq_gains (json::f(fsw, roll_wn_orbits, 3.0)) |
| `fsw_tune_roll_zeta` | fsw | stated | 1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs mtq_gains (json::f(fsw, roll_zeta, 1.0)) |
| `fsw_tune_rw_bandwidth` | fsw | stated | 0.9 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs rw_gains (json::f(fsw, rw_bandwidth, 0.9)) |
| `fsw_tune_rw_damping` | fsw | stated | 2 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs rw_gains (json::f(fsw, rw_damping, 2.0)) |
| `fsw_tune_rw_rate_hz` | fsw | stated | 10 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs rw_gains (json::f(fsw, rw_rate_hz, 10.0)) |
| `fsw_tune_ss_gain` | fsw | stated | 1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs spin_params (json::f(fsw, ss_gain, 1.0)) |
| `gc_0` | ctl | stated | 0.1 RadianPerSecond | the design, adcs_ref_c1 |
| `gc_1` | ctl | stated | 0.707 One | the design, adcs_ref_c1 |
| `gc_2` | ctl | computed | 56.5771 (SI) | settling_time_2pct(w_n, zeta) |
| `gd_0` | env | computed | 6.34679e-08 (SI) | gd_0(r, i_max, i_min) |
| `gd_1` | env | computed | 1.36971e-08 (SI) | gd_1(rho, v, c_d, a_fr, c_pa) |
| `gd_2` | env | computed | 5.10763e-09 (SI) | gd_2(p_srp, a_sun, q, c_ps) |
| `gd_3` | env | computed | 4.6512e-07 (SI) | gd_3(residual_dipole, field) |
| `gd_4` | env | computed | 5.47392e-07 (SI) | gd_4(aerodynamic, gravity_gradient, solar, magnetic) |
| `gd_5` | env | computed | 0.00314148 (SI) | gd_5(tau_d, t_orb) |
| `gdn_payload_boresight_x` | gdn | stated | 0.0 | the design, 1.0.0 code: engine/crates/adcs-sim/src/product.rs Dev::load (boresight: [0.0, 1.0, 0.0], the payload boresight the guidance, the metrics and the Earth sensor take when the product states none) |
| `gdn_payload_boresight_y` | gdn | stated | 1.0 | the design, 1.0.0 code: engine/crates/adcs-sim/src/product.rs Dev::load (boresight: [0.0, 1.0, 0.0], the payload boresight the guidance, the metrics and the Earth sensor take when the product states none) |
| `gdn_payload_boresight_z` | gdn | stated | 0.0 | the design, 1.0.0 code: engine/crates/adcs-sim/src/product.rs Dev::load (boresight: [0.0, 1.0, 0.0], the payload boresight the guidance, the metrics and the Earth sensor take when the product states none) |
| `gdn_sun_axis_x` | gdn | stated | 0.0 | the design, 1.0.0 code: engine/crates/adcs-sim/src/product.rs Dev::load (sun_axis: [0.0, 0.0, -1.0], the power face the guidance turns to the Sun when the product states none) |
| `gdn_sun_axis_y` | gdn | stated | 0.0 | the design, 1.0.0 code: engine/crates/adcs-sim/src/product.rs Dev::load (sun_axis: [0.0, 0.0, -1.0], the power face the guidance turns to the Sun when the product states none) |
| `gdn_sun_axis_z` | gdn | stated | -1.0 | the design, 1.0.0 code: engine/crates/adcs-sim/src/product.rs Dev::load (sun_axis: [0.0, 0.0, -1.0], the power face the guidance turns to the Sun when the product states none) |
| `gf_0` | act | stated | 3 Millimetre | the design, idmas_v2 |
| `gf_1` | act | stated | 0.022 SquareMetre | the design, idmas_v2 |
| `gf_2` | act | stated | 6440 KgPerCubicMetre | the design, idmas_v2 |
| `gf_3` | act | stated | 0.0024 PascalSecond | the design, idmas_v2 |
| `gf_4` | act | stated | 0.2 MetrePerSecond | the design, idmas_v2 |
| `gf_5` | act | stated | 0.72 Metre | the design, idmas_v2 |
| `gf_6` | act | computed | 0.000400591 (SI) | gf_6(d, s, rho, v) |
| `gf_7` | act | computed | 0.754688 (SI) | gf_7(d, rho, mu) |
| `gf_8` | act | computed | 0.00173718 (SI) | gf_8(h, mu, l, rho, d, s) |
| `gm_0` | act | stated | 0.45 AmpereSquareMetre | the design, idmas_v2 |
| `gm_1` | act | computed | 0.0235377 (SI) | gm_1(tau_d, b_min) |
| `gm_2` | act | computed | 0.0235377 (SI) | gm_2(momentum, field, dump_time) |
| `gm_3` | act | computed | 1.04652e-05 (SI) | gm_3(m_av, b_min, n_mtq) |
| `gr_1` | act | stated | 0.8 Metre | the design, idmas_v2 |
| `gr_2` | act | stated | 220 Second | the design, idmas_v2 |
| `m1_1` | systems | stated | 27.0 Year | the case, mission.epoch |
| `m1_3` | systems | stated | 10.0 Count | the case, mission.spd |
| `m1_4` | systems | stated | 30.0 Degree | the case, mission.sangle |
| `m1_5` | systems | stated | 10.0 DegreePerSecond | the case, mission.w0 |
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
| `s1_4` | dyn | computed | [0.00599251, 0.0139825, -0.0129838] (SI) | cm_offset(cpa, dir) |
| `s2_0` | dyn | stated | 0.034 SquareMetre | the case, surface.afr |
| `s2_1` | dyn | stated | 0.02 Metre | the case, surface.cpa |
| `s2_2` | dyn | stated | 0.034 SquareMetre | the case, surface.asun |
| `s2_3` | dyn | stated | 0.02 Metre | the case, surface.cps |
| `s2_4` | dyn | stated | 0.6 One | the case, surface.refl |
| `s2_5` | dyn | stated | 2.2 One | the case, surface.cd |
| `s3_0` | dyn | stated | 0.01 AmpereSquareMetre | the case, magnetic.dres |
| `vv_record_step_default` | vv | stated | 1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (json::f(time, record_dt_s, 1.0)) |
| `vv_run_duration_default` | vv | stated | 600 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (json::f(time, duration_s, 600.0)) |

**Not computed (992), by why:**

- no value, relation or pseudocode yet: 711 (act__cmg, act__vscmg, cas, cat, cf, cf_1, cf_4, cf_5, ci1, ci1_0, ci1_1, ci1_2, …)
- computed during a run (generated into adcs-sim-core): 72 (act_cmg_model, act_rotor_set, act_rotor_telemetry, act_vscmg_gimbal_limits, act_vscmg_model, dyn_flexible_mode, dyn_initial_state, dyn_rigid_body, dyn_rotor_coupling, dyn_total_momentum, env_calendar_time, env_moon_fast, …)
- computed during a run (generated into adcs-sim): 47 (env_orbit_start, fsw_param_alloc, fsw_param_bdot_law, fsw_param_es_noise, fsw_param_gim_axis, fsw_param_gim_rate_max, fsw_param_gps_latency, fsw_param_gyro_arw, fsw_param_gyro_rrw, fsw_param_has_es, fsw_param_has_gps, fsw_param_has_gyro, …)
- computed during a run (by the flight software or the engine, at each step): 23 (act_cmg_axes, act_cmg_steering, dyn_kinematics, fdir_rotor_health, fdir_safe_mode, fdir_sensor_health, fsw_allocation, fsw_control, fsw_drivers, fsw_estimation, fsw_guidance, fsw_modes, …)
- computed during a run (generated into adcs-pop): 22 (env_de440, env_density_model, env_drag_force, env_dtm2020_operational, env_dtm2020_research, env_earth_frames, env_erp_force, env_exponential_atmosphere, env_force_model, env_gas_surface, env_geodetic, env_gravity_field, …)
- a relation, but no pseudocode yet: its author writes it: 18 (rk1_0, rk1_1, rk1_2, rk1_3, rk1_4, rk2_0, rk2_1, rk2_2, rk2_3, rk2_4, rk2_5, rk3_0, …)
- needs an input that has no value: 16 (fsw_param_mtq_k1, fsw_param_mtq_k16, fsw_param_mtq_k2, fsw_param_mtq_lam16, fsw_param_sb_kd, fsw_param_sb_kp, gf_9, gp_5, gr_3, gr_4, gw_3, gw_4, …)
- its module has 2 functions and names none for this node (by its id or its output's symbol): 9 (gb_0, gb_1, gb_2, gb_3, l3_budget_row_06, l3_budget_row_07, l3_budget_row_08, l3_fmr_row_14, l3_fmr_row_15)
- its module has 7 functions and names none for this node (by its id or its output's symbol): 7 (design_sizing_cmg, design_sizing_rw, design_sizing_sensors, design_sizing_vscmg, gw_5, gw_6, l3_budget_row_03)
- its module has 5 functions and names none for this node (by its id or its output's symbol): 5 (design_sizing_mtq, gm_4, gm_5, gp_4, l3_budget_row_02)
- its module has 8 functions and names none for this node (by its id or its output's symbol): 3 (ctl_floquet_certificate, design_sizing_fmr, l3_budget_row_04)
- its module has 9 functions and names none for this node (by its id or its output's symbol): 3 (design_loop_converge, design_loop_redundancy, design_loop_robustness)
- its module has 6 functions and names none for this node (by its id or its output's symbol): 2 (catalogue_datasheet_derive, design_power_system)
- its module has 3 functions and names none for this node (by its id or its output's symbol): 2 (design_sizing_rcs, l3_budget_row_05)
- its module has 4 functions and names none for this node (by its id or its output's symbol): 2 (fsw_param_rw_Klqr, l3_fmr_row_07)
- its module has 13 functions and names none for this node (by its id or its output's symbol): 2 (gw_2, l3_budget_row_01)
- computed during a run (generated into flight): 1 (env_time_frames)
- its module has 10 functions and names none for this node (by its id or its output's symbol): 1 (fsw_param_mtq_Klqr)
- the case does not state pointing.et: 1 (gp_3)
- computed during a run (generated into adcs-sim, adcs-sim-core): 1 (kpi_metric_channels)
- the case does not state mission.life: 1 (m1_0)
- the case does not state mission.duty: 1 (m1_2)
- stated, with no value: its method is env_time_frames's since S7.3 (fsw/pseudocode/02_time_frames_models.pc, one copy); the pseudocode it carried from 1.0.0 stays as provenance: 1 (nav_time_frames)
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

From the merged releases (design.tndb). Rows: 60 computed, 4 evidence, 984 not computed, 174 stated. Closures: 33 blocked, 1 fail, 4 pass.

| Closure | KPI | Answer | Why |
|---|---|---|---|
| `kpi_absolute_pointing_error_ape_verified` | Absolute pointing error (APE) | **pass** | p1a_0 = 0.00542 deg <= 0.01 Degree (p1k_0) |
| `kpi_absolute_pointing_error_ape_analysis` | Absolute pointing error (APE) | blocked | gp_5 has no value (needs terms[1] from gp_0 (not computed), terms[2] from gp_1 (not computed), terms[3] from gp_2 (not computed), terms[4] from gp_3 (not computed), terms[5] from gp_4 (not computed)) |
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
| `kpi_settling_time_after_a_slew_analysis` | Settling time after a slew | **fail** | gc_2 = 56.5771 (SI) <= 20.0 Second (p2k_1) |
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
| `kpi_adcs_mass_analysis` | ADCS mass | blocked | gb_0 has no value (its module has 2 functions and names none for this node (by its id or its output's symbol)) |
| `kpi_adcs_orbit_average_power_verified` | ADCS orbit-average power | **pass** | p4a_1 = 0.4945 W <= 2.0 Watt (p4k_1) |
| `kpi_adcs_orbit_average_power_analysis` | ADCS orbit-average power | blocked | gb_1 has no value (its module has 2 functions and names none for this node (by its id or its output's symbol)) |
| `kpi_adcs_peak_power_verified` | ADCS peak power | blocked | p4a_2 has no value (evidence: no engine campaign of this case judges a metric against req.ppk) |
| `kpi_adcs_peak_power_analysis` | ADCS peak power | blocked | gb_2 has no value (its module has 2 functions and names none for this node (by its id or its output's symbol)) |
| `kpi_adcs_volume_verified` | ADCS volume | blocked | p4a_3 has no value (evidence: no engine campaign of this case judges a metric against req.vol) |
| `kpi_adcs_volume_analysis` | ADCS volume | blocked | gb_3 has no value (its module has 2 functions and names none for this node (by its id or its output's symbol)) |
| `kpi_rcs_propellant_per_year_verified` | RCS propellant per year | blocked | the requirement p4k_4 has no value (the case does not state req.prop) |
| `kpi_rcs_propellant_per_year_analysis` | RCS propellant per year | blocked | the requirement p4k_4 has no value (the case does not state req.prop) |

| Row | Group | State | Value | Where from / why not |
|---|---|---|---|---|
| `act_cmg_speed_gain` | act | stated | 1.0 | the design, 1.0.0 code: engine/crates/adcs-sim/src/product.rs Dev::load (MexDesc k_speed = 1.0, the device default every product flies) |
| `act_fmr_flow_gain` | act | stated | 2.0 | the design, 1.0.0 code: engine/crates/adcs-sim/src/product.rs Dev::load (MexDesc k_flow = 2.0, the device default every product flies) |
| `act_fmr_flow_tau` | act | stated | 0.3 | the design, 1.0.0 code: engine/crates/adcs-sim/src/product.rs Dev::load (MexDesc flow_tau = 0.3, the device default every product flies) |
| `act_gimbal_tlm_noise` | act | stated | 1e-5 | the design, 1.0.0 code: engine/crates/adcs-sim/src/run.rs sense (x.d[g] + 1e-5 normal, the rotor telemetry the CAN frames carry) |
| `act_rotor_tlm_noise` | act | stated | 1e-7 | the design, 1.0.0 code: engine/crates/adcs-sim/src/run.rs sense (x.h[i] + 1e-7 normal, the rotor telemetry the CAN frames carry) |
| `act_rw_drive_efficiency` | act | stated | 0.8 | the design, 1.0.0 code: engine/crates/adcs-sim/src/product.rs Dev::load (MexDesc eta = 0.8, the device default every product flies) |
| `act_rw_friction_comp` | act | stated | 0.95 | the design, 1.0.0 code: engine/crates/adcs-sim/src/product.rs Dev::load (MexDesc friction_comp = 0.95, the device default every product flies) |
| `act_rw_torque_noise` | act | stated | 0.001 | the design, 1.0.0 code: engine/crates/adcs-sim/src/product.rs Dev::load (MexDesc torque_noise = 0.001, the device default every product flies) |
| `cf_0` | systems | stated | 3 Count | the design, adcs_ref_c1 |
| `cf_2` | systems | stated | 3 Count | the design, adcs_ref_c1 |
| `cf_3` | systems | stated | 0 Count | the design, adcs_ref_c1 |
| `dyn_cm_direction_x` | dyn | stated | 0.30 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (cmd = [0.30, 0.70, -0.65], the direction the case's surface.cpa offset takes) |
| `dyn_cm_direction_y` | dyn | stated | 0.70 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (cmd = [0.30, 0.70, -0.65], the direction the case's surface.cpa offset takes) |
| `dyn_cm_direction_z` | dyn | stated | -0.65 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (cmd = [0.30, 0.70, -0.65], the direction the case's surface.cpa offset takes) |
| `dyn_initial_attitude_kind_default` | dyn | stated | 0 | the design, 1.0.0 code: engine/crates/adcs-sim/src/run.rs initial_state (the kind's match: _ => ATTSTART_NADIR) |
| `dyn_initial_error_angle` | dyn | stated | 0 | the design, 1.0.0 code: engine/crates/adcs-sim/src/run.rs initial_state (json::f(attitude, angle_deg, 0.0)) |
| `dyn_initial_error_axis` | dyn | stated | [1, 0, 0] | the design, 1.0.0 code: engine/crates/adcs-sim/src/run.rs initial_state (unwrap_or([1.0, 0.0, 0.0])) |
| `dyn_initial_rate_extra` | dyn | stated | 0 | the design, 1.0.0 code: engine/crates/adcs-sim/src/run.rs initial_state (json::f(rate, extra_deg_s, 0.0)) |
| `dyn_initial_rate_kind_default` | dyn | stated | 0 | the design, 1.0.0 code: engine/crates/adcs-sim/src/run.rs initial_state (the kind's match: _ => RATESTART_VALUE) |
| `dyn_initial_rate_magnitude` | dyn | stated | 0 | the design, 1.0.0 code: engine/crates/adcs-sim/src/run.rs initial_state (magnitude_deg_s: None => 0.0) |
| `dyn_initial_rate_value` | dyn | stated | [0, 0, 0] | the design, 1.0.0 code: engine/crates/adcs-sim/src/run.rs initial_state (unwrap_or([0.0; 3])) |
| `dyn_surface_accommodation` | dyn | stated | 0.8 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs ACCOMMODATION (its comment: Moe and Moe 2005, LEO) |
| `dyn_surface_specular_share` | dyn | stated | 0.5 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs SPEC_FRAC |
| `dyn_surface_vb_ratio` | dyn | stated | 0.05 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs VB_RATIO (its comment: Moe and Moe 2005, LEO) |
| `dyn_truth_plant` | dyn | computed | [[0.0067, 0, 0], [0, 0.042, 0], [0, 0, 0.042]] (SI) | principal_inertia(imin, iint, imax) |
| `env_ap_default` | env | stated | 7 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (ap: 7.0) |
| `env_case_orbit` | env | computed | 5738.99 (SI) | case_mean_motion(h) -> t |
| `env_density_scale_default` | env | stated | 1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (density_scale: 1.0) |
| `env_f107_default` | env | stated | 130 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (f107: 130.0) |
| `env_f107a_default` | env | stated | 130 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (f107a: 130.0) |
| `env_fast_zonal_degree` | env | stated | 6 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (zonal_max: 6) |
| `env_field_degree` | env | stated | 13 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (igrf_nmax: 13) |
| `env_force_drag` | env | stated | 1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (drag: true) |
| `env_force_srp` | env | stated | 1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (srp: true) |
| `env_force_third_body` | env | stated | 1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (third_body: true) |
| `env_kp_default` | env | stated | 2 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (kp: 2.0) |
| `env_orbit_precision_default` | env | stated | 1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (orbit_model: "pop") |
| `env_orbit_step` | env | stated | 10 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (orbit_step_s: 10.0) |
| `env_refresh_step` | env | stated | 1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (env_dt_s: 1.0) |
| `env_start_arg_lat_default` | env | stated | 0 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (json::f(initial, arg_lat_deg, 0.0)) |
| `env_torque_aero` | env | stated | 1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (env_on: [true; 4], the second) |
| `env_torque_gravity_gradient` | env | stated | 1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (env_on: [true; 4], the first) |
| `env_torque_magnetic` | env | stated | 1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (env_on: [true; 4], the fourth) |
| `env_torque_radiation` | env | stated | 1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (env_on: [true; 4], the third) |
| `fsw_param_J` | fsw | computed | [[0.0067, 0, 0], [0, 0.042, 0], [0, 0, 0.042]] (SI) | fsw_body_model(j, m_res, sun_axis, boresight) -> jf |
| `fsw_param_auto_next` | fsw | stated | 255 | the design, the scenario's fsw.auto_next; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs modes_and_laws (none when it names none: MODE_NONE, 255) |
| `fsw_param_bdot_k` | fsw | computed | 8.76379e-05 (SI) | fsw_bdot_gain(scale, n, inc_deg, ii) |
| `fsw_param_capture_deg` | fsw | stated | 3 | the design, the scenario's fsw.capture_deg; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs rw_gains (json::f(fsw, capture_deg, 3.0)) |
| `fsw_param_capture_rate_deg_s` | fsw | stated | 1 | the design, the scenario's fsw.capture_rate_deg_s; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs rw_gains (json::f(fsw, capture_rate_deg_s, 1.0)) |
| `fsw_param_cmg_k_null` | fsw | stated | 0.002 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs rotor_params (p.cmg_k_null = 0.002) |
| `fsw_param_cmg_lam0` | fsw | stated | 1e-09 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs rotor_params (p.cmg_lam0 = 1e-9) |
| `fsw_param_cmg_mu` | fsw | stated | 10 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs rotor_params (p.cmg_mu = 10.0) |
| `fsw_param_detumble_exit` | fsw | computed | 0.00872665 (SI) | fsw_rate_rad(deg_s) |
| `fsw_param_detumble_hold_s` | fsw | stated | 60 | the design, the scenario's fsw.detumble_hold_s; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs mtq_gains (json::f(fsw, detumble_hold_s, 60.0)) |
| `fsw_param_dt` | fsw | stated | 0.1 | the design, the scenario's time.dt_s; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs build (json::f(time, dt_s, 0.1)) |
| `fsw_param_dump_k` | fsw | stated | 0.002 | the design, the scenario's fsw.dump_gain; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs rotor_params (json::f(fsw, dump_gain, 2e-3)) |
| `fsw_param_fdir_h_frac` | fsw | stated | 0.005 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs rotor_params (p.fdir_h_frac = 0.005: a commanded change under 0.5 % of h_max is not judged) |
| `fsw_param_fdir_s` | fsw | stated | 3 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs rotor_params (p.fdir_s = 3.0) |
| `fsw_param_fdir_win_s` | fsw | stated | 120 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs rotor_params (p.fdir_win_s = 120.0: 120 s windows) |
| `fsw_param_gd_T` | fsw | stated | 1 | the design, the scenario's fsw.guidance.T_s; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs guidance_params (json::f(guidance, T_s, 1.0)) |
| `fsw_param_gd_axis` | fsw | stated | [0, 0, 0] | the design, the scenario's fsw.guidance.axis; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs guidance_params (unwrap_or([0.0; 3])) |
| `fsw_param_gd_flip_hyst` | fsw | stated | 0.1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs sensor_params (p.gd_flip_hyst = 0.1) |
| `fsw_param_gd_kind` | fsw | stated | 0 | the design, the scenario's fsw.guidance.kind; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs guidance_params (json::s(guidance, kind, nadir): nadir, fswchoice's GuidKind 0) |
| `fsw_param_gd_q_inertial` | fsw | stated | [0, 0, 0, 1] | the design, the scenario's fsw.guidance.q_inertial; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs guidance_params (unwrap_or([0.0, 0.0, 0.0, 1.0])) |
| `fsw_param_gd_q_off` | fsw | computed | [0, 0, 0, 1] (SI) | fsw_payload_offset(boresight) |
| `fsw_param_gd_roll_deg` | fsw | stated | 0 | the design, the scenario's fsw.guidance.roll_deg; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs guidance_params (json::f(guidance, roll_deg, 0.0)) |
| `fsw_param_gd_t0` | fsw | stated | 0 | the design, the scenario's fsw.guidance.t0; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs guidance_params (json::f(guidance, t0, 0.0)) |
| `fsw_param_gd_yaw_flip` | fsw | stated | 1 | the design, the scenario's fsw.yaw_flip; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs sensor_params (json::b(fsw, yaw_flip, true)) |
| `fsw_param_gnss_ecef` | fsw | stated | 1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs sensor_params (p.gnss_ecef = 1: the receiver's standard frame, converted onboard) |
| `fsw_param_h_bias` | fsw | stated | 0.002 | the design, the scenario's fsw.wheel_bias_Nms; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs rotor_params (json::f(fsw, wheel_bias_Nms, 2e-3)) |
| `fsw_param_ho_hold_s` | fsw | stated | 60 | the design, the scenario's fsw.handover_hold_s; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs mtq_gains (json::f(fsw, handover_hold_s, 60.0)) |
| `fsw_param_ho_in_dps` | fsw | stated | 1 | the design, the scenario's fsw.handover_in_dps; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs mtq_gains (json::f(fsw, handover_in_dps, 1.0)) |
| `fsw_param_ho_out_dps` | fsw | stated | 0.5 | the design, the scenario's fsw.handover_out_dps; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs mtq_gains (json::f(fsw, handover_out_dps, 0.5)) |
| `fsw_param_igrf_nmax` | fsw | stated | 10 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs sensor_params (p.igrf_nmax = 10) |
| `fsw_param_jd0` | fsw | computed | 2.46141e+06 (SI) | fsw_epoch_earth(jd) -> jd0 |
| `fsw_param_m_res_est` | fsw | computed | [0.0057735, 0.0057735, 0.0057735] (SI) | fsw_body_model(j, m_res, sun_axis, boresight) -> mf |
| `fsw_param_mekf_gate` | fsw | stated | 16.27 | the design, the scenario's fsw.mekf_gate; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs sensor_params (json::f(fsw, mekf_gate, 16.27)) |
| `fsw_param_mekf_meas_scale` | fsw | stated | 1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs sensor_params (p.mekf_meas_scale = 1.0) |
| `fsw_param_mekf_rej_max` | fsw | stated | 30 | the design, the scenario's fsw.mekf_rej_max; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs sensor_params (json::f(fsw, mekf_rej_max, 30.0)) |
| `fsw_param_mtq_Gs` | fsw | computed | [1e-05, 1e-05, 1e-05] (SI) | fsw_mtq_smc(wn, z, phi) -> gs |
| `fsw_param_mtq_Kd` | fsw | computed | [0.000134, 0.00084, 0.00084] (SI) | fsw_mtq_pd(ii, wn, z) -> kd |
| `fsw_param_mtq_Ki` | fsw | computed | [0, 0, 0] (SI) | fsw_mtq_pd(ii, wn, z) -> ki |
| `fsw_param_mtq_Kp` | fsw | computed | [1.675e-07, 1.05e-06, 1.05e-06] (SI) | fsw_mtq_pd(ii, wn, z) -> kp |
| `fsw_param_mtq_Pth` | fsw | computed | [[4.98567e-07, 0, 0], [0, 1.27604e-06, 0], [0, 0, 1.27604e-06]] (SI) | fsw_mtq_tango(ii, wn, z, gp, gd) -> pth |
| `fsw_param_mtq_Pw` | fsw | computed | [[0.000398799, 0, 0], [0, 0.00102111, 0], [0, 0, 0.00102111]] (SI) | fsw_mtq_tango(ii, wn, z, gp, gd) -> pw |
| `fsw_param_mtq_eps` | fsw | stated | 0.001 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs mtq_gains (let eps = 1e-3, the papers' time-scale parameter) |
| `fsw_param_mtq_err_max` | fsw | stated | 0.5 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs mtq_gains (p.mtq_err_max = 0.5) |
| `fsw_param_mtq_gg_ff` | fsw | computed | 3 (SI) | fsw_gg_feedforward(q_off, j) |
| `fsw_param_mtq_int_max` | fsw | stated | 0.05 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs mtq_gains (p.mtq_int_max = 0.05) |
| `fsw_param_mtq_lambda` | fsw | computed | 0.0025 (SI) | fsw_mtq_smc(wn, z, phi) -> lambda |
| `fsw_param_mtq_meas` | fsw | stated | 0.2 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs mtq_gains (p.mtq_meas = 0.2) |
| `fsw_param_mtq_period` | fsw | stated | 1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs mtq_gains (p.mtq_period = 1.0) |
| `fsw_param_mtq_phi` | fsw | stated | 0.0005 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs mtq_gains (p.mtq_phi = 5e-4) |
| `fsw_param_mu` | fsw | computed | 3.986e+14 (SI) | fsw_epoch_earth(jd) -> mu |
| `fsw_param_rate_lpf_s` | fsw | stated | 0.3 | the design, the scenario's fsw.rate_lpf_s; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs sensor_params (json::f(fsw, rate_lpf_s, 0.3)) |
| `fsw_param_rcs_assist` | fsw | stated | 1 | the design, the scenario's fsw.rcs_assist; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs rcs_params (json::b(fsw, rcs_assist, true)) |
| `fsw_param_rcs_assist_frac` | fsw | stated | 0.8 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs rcs_params (p.rcs_assist_frac = 0.8) |
| `fsw_param_rcs_dump` | fsw | stated | 1 | the design, the scenario's fsw.rcs_dump; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs rcs_params (json::b(fsw, rcs_dump, true)) |
| `fsw_param_rcs_dump_hi` | fsw | stated | 0.004 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs rcs_params (p.rcs_dump_hi = 4e-3) |
| `fsw_param_rcs_dump_k` | fsw | stated | 0.05 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs rcs_params (p.rcs_dump_k = 0.05) |
| `fsw_param_rcs_dump_lo` | fsw | stated | 0.001 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs rcs_params (p.rcs_dump_lo = 1e-3) |
| `fsw_param_rcsd_T_damp_s` | fsw | stated | 20 | the design, the scenario's fsw.rcs_damp_s; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs rcs_params (json::f(fsw, rcs_damp_s, 20.0)) |
| `fsw_param_rcsd_deadband_deg_s` | fsw | stated | 0.2 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs rcs_params (p.rcsd_deadband_deg_s = 0.2) |
| `fsw_param_rcsd_period_s` | fsw | stated | 1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs rcs_params (p.rcsd_period_s = 1.0) |
| `fsw_param_roll_axis` | fsw | computed | [0, 1, 0] (SI) | fsw_body_model(j, m_res, sun_axis, boresight) -> roll |
| `fsw_param_rw_Gs` | fsw | computed | [0.00072, 0.00072, 0.00072] (SI) | fsw_rw_smc(wn, z, phi) -> gs |
| `fsw_param_rw_Kd` | fsw | computed | [0.02412, 0.1512, 0.1512] (SI) | fsw_rw_pid(ii, wn, z) -> kd |
| `fsw_param_rw_Ki` | fsw | computed | [0.000732645, 0.0045927, 0.0045927] (SI) | fsw_rw_pid(ii, wn, z) -> ki |
| `fsw_param_rw_Kp` | fsw | computed | [0.005427, 0.03402, 0.03402] (SI) | fsw_rw_pid(ii, wn, z) -> kp |
| `fsw_param_rw_dt` | fsw | computed | 0.1 (SI) | fsw_rw_period(rate_hz) |
| `fsw_param_rw_err_max` | fsw | stated | 0.2 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs rw_gains (p.rw_err_max = 0.2) |
| `fsw_param_rw_int_max` | fsw | stated | 0.02 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs rw_gains (p.rw_int_max = 0.02) |
| `fsw_param_rw_lambda` | fsw | computed | 0.225 (SI) | fsw_rw_smc(wn, z, phi) -> lambda |
| `fsw_param_rw_phi` | fsw | stated | 0.0002 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs rw_gains (p.rw_phi = 2e-4) |
| `fsw_param_sa_done_deg` | fsw | stated | 10 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs spin_params (p.sa_done_deg = 10.0) |
| `fsw_param_sa_done_hold_s` | fsw | stated | 60 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs spin_params (p.sa_done_hold_s = 60.0) |
| `fsw_param_sa_kd` | fsw | stated | 0.1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs spin_params (p.sa_kd = 0.1) |
| `fsw_param_sa_w_max_deg_s` | fsw | stated | 1 | the design, the scenario's fsw.sun_acq_rate_deg_s; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs spin_params (json::f(fsw, sun_acq_rate_deg_s, 1.0)) |
| `fsw_param_sb_kdroll` | fsw | computed | 0.000275896 (SI) | fsw_mtq_roll(e, j, n, roll_wn_orbits, roll_zeta, roll_gain) -> kdroll |
| `fsw_param_sb_kroll` | fsw | computed | 4.53086e-07 (SI) | fsw_mtq_roll(e, j, n, roll_wn_orbits, roll_zeta, roll_gain) -> kroll |
| `fsw_param_sb_roll_gate` | fsw | computed | 0.965926 (SI) | fsw_roll_gate(roll_gate_deg) |
| `fsw_param_ss_dr_k` | fsw | computed | 0.01 (SI) | fsw_sun_spin_gains(g, jzz) -> dr_k |
| `fsw_param_ss_dr_k1` | fsw | stated | 1.5 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs spin_params (p.ss_dr_k1 = 1.5; de Ruiter 2011: k1 > 1, the paper's 1.5) |
| `fsw_param_ss_dr_k2` | fsw | computed | 0.021 (SI) | fsw_sun_spin_gains(g, jzz) -> dr_k2 |
| `fsw_param_ss_dwell_in_s` | fsw | stated | 60 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs spin_params (p.ss_dwell_in_s = 60.0) |
| `fsw_param_ss_dwell_out_s` | fsw | stated | 30 | the design, the scenario's fsw.sun_spin_dwell_out_s; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs spin_params (json::f(fsw, sun_spin_dwell_out_s, 30.0)) |
| `fsw_param_ss_k1` | fsw | computed | 0.01 (SI) | fsw_sun_spin_gains(g, jzz) -> k1 |
| `fsw_param_ss_k2` | fsw | computed | 0.05 (SI) | fsw_sun_spin_gains(g, jzz) -> k2 |
| `fsw_param_ss_k_l1` | fsw | stated | 1000000 | the design, the scenario's fsw.l1_gain; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs spin_params (json::f(fsw, l1_gain, 1e6)) |
| `fsw_param_ss_omega_exit_dps` | fsw | stated | 2 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs spin_params (p.ss_omega_exit_dps = 2.0) |
| `fsw_param_ss_omega_max_dps` | fsw | stated | 100 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs spin_params (p.ss_omega_max_dps = 100.0) |
| `fsw_param_ss_perp_in_dps` | fsw | stated | 0.5 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs spin_params (p.ss_perp_in_dps = 0.5) |
| `fsw_param_ss_perp_out_dps` | fsw | stated | 1 | the design, the scenario's fsw.sun_spin_perp_out_dps; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs spin_params (json::f(fsw, sun_spin_perp_out_dps, 1.0)) |
| `fsw_param_ss_sigma0` | fsw | stated | 1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs spin_params (p.ss_sigma0 = 1.0) |
| `fsw_param_ss_spin_dps` | fsw | stated | 6 | the design, the scenario's fsw.spin_rate_dps; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs spin_params (json::f(fsw, spin_rate_dps, 6.0)), and build (spin_dps) |
| `fsw_param_ss_sun_min` | fsw | stated | 0.05 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs spin_params (p.ss_sun_min = 0.05) |
| `fsw_param_ss_t_check_s` | fsw | stated | 60 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs spin_params (p.ss_t_check_s = 60.0) |
| `fsw_param_ss_z_in_dps` | fsw | stated | 0.5 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs spin_params (p.ss_z_in_dps = 0.5) |
| `fsw_param_st_coast_s` | fsw | stated | 900 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs sensor_params (p.st_coast_s = 900.0) |
| `fsw_param_start_mode` | fsw | stated | 0 | the design, the scenario's fsw.start_mode; when it states none, as the 1.0.0 code gave it: engine/crates/adcs-sim/src/config.rs modes_and_laws (json::s(fsw, start_mode, detumble): detumble, state 0 of adcs_mode_t) |
| `fsw_param_sun_axis` | fsw | computed | [0, 0, -1] (SI) | fsw_body_model(j, m_res, sun_axis, boresight) -> sun |
| `fsw_tune_avanzini_k_over_n` | fsw | stated | 0.84 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs mtq_gains (json::f(fsw, avanzini_k_over_n, 0.84)) |
| `fsw_tune_avanzini_lambda` | fsw | stated | 0.08 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs mtq_gains (json::f(fsw, avanzini_lambda, 0.08)) |
| `fsw_tune_bdot_gain_scale` | fsw | stated | 3 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs mtq_gains (json::f(fsw, bdot_gain_scale, 3.0)) |
| `fsw_tune_detumble_exit_deg_s` | fsw | stated | 0.5 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs mtq_gains (json::f(fsw, detumble_exit_deg_s, 0.5)) |
| `fsw_tune_mtq_gain_d` | fsw | stated | 1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs mtq_gains (json::f(fsw, mtq_gain_d, 1.0)) |
| `fsw_tune_mtq_gain_p` | fsw | stated | 1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs mtq_gains (json::f(fsw, mtq_gain_p, 1.0)) |
| `fsw_tune_mtq_wn` | fsw | stated | 0.005 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs mtq_gains (json::f(fsw, mtq_wn, 0.005)) |
| `fsw_tune_mtq_zeta` | fsw | stated | 2 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs mtq_gains (json::f(fsw, mtq_zeta, 2.0)) |
| `fsw_tune_roll_gain` | fsw | stated | 1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs mtq_gains (json::f(fsw, roll_gain, 1.0)) |
| `fsw_tune_roll_gate_deg` | fsw | stated | 15 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs mtq_gains (json::f(fsw, roll_gate_deg, 15.0)) |
| `fsw_tune_roll_wn_orbits` | fsw | stated | 3 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs mtq_gains (json::f(fsw, roll_wn_orbits, 3.0)) |
| `fsw_tune_roll_zeta` | fsw | stated | 1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs mtq_gains (json::f(fsw, roll_zeta, 1.0)) |
| `fsw_tune_rw_bandwidth` | fsw | stated | 0.9 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs rw_gains (json::f(fsw, rw_bandwidth, 0.9)) |
| `fsw_tune_rw_damping` | fsw | stated | 2 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs rw_gains (json::f(fsw, rw_damping, 2.0)) |
| `fsw_tune_rw_rate_hz` | fsw | stated | 10 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs rw_gains (json::f(fsw, rw_rate_hz, 10.0)) |
| `fsw_tune_ss_gain` | fsw | stated | 1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs spin_params (json::f(fsw, ss_gain, 1.0)) |
| `gc_0` | ctl | stated | 0.1 RadianPerSecond | the design, adcs_ref_c1 |
| `gc_1` | ctl | stated | 0.707 One | the design, adcs_ref_c1 |
| `gc_2` | ctl | computed | 56.5771 (SI) | settling_time_2pct(w_n, zeta) |
| `gd_0` | env | computed | 6.34679e-08 (SI) | gd_0(r, i_max, i_min) |
| `gd_1` | env | computed | 1.36971e-08 (SI) | gd_1(rho, v, c_d, a_fr, c_pa) |
| `gd_2` | env | computed | 5.10763e-09 (SI) | gd_2(p_srp, a_sun, q, c_ps) |
| `gd_3` | env | computed | 4.6512e-07 (SI) | gd_3(residual_dipole, field) |
| `gd_4` | env | computed | 5.47392e-07 (SI) | gd_4(aerodynamic, gravity_gradient, solar, magnetic) |
| `gd_5` | env | computed | 0.00314148 (SI) | gd_5(tau_d, t_orb) |
| `gdn_payload_boresight_x` | gdn | stated | 0.0 | the design, 1.0.0 code: engine/crates/adcs-sim/src/product.rs Dev::load (boresight: [0.0, 1.0, 0.0], the payload boresight the guidance, the metrics and the Earth sensor take when the product states none) |
| `gdn_payload_boresight_y` | gdn | stated | 1.0 | the design, 1.0.0 code: engine/crates/adcs-sim/src/product.rs Dev::load (boresight: [0.0, 1.0, 0.0], the payload boresight the guidance, the metrics and the Earth sensor take when the product states none) |
| `gdn_payload_boresight_z` | gdn | stated | 0.0 | the design, 1.0.0 code: engine/crates/adcs-sim/src/product.rs Dev::load (boresight: [0.0, 1.0, 0.0], the payload boresight the guidance, the metrics and the Earth sensor take when the product states none) |
| `gdn_sun_axis_x` | gdn | stated | 0.0 | the design, 1.0.0 code: engine/crates/adcs-sim/src/product.rs Dev::load (sun_axis: [0.0, 0.0, -1.0], the power face the guidance turns to the Sun when the product states none) |
| `gdn_sun_axis_y` | gdn | stated | 0.0 | the design, 1.0.0 code: engine/crates/adcs-sim/src/product.rs Dev::load (sun_axis: [0.0, 0.0, -1.0], the power face the guidance turns to the Sun when the product states none) |
| `gdn_sun_axis_z` | gdn | stated | -1.0 | the design, 1.0.0 code: engine/crates/adcs-sim/src/product.rs Dev::load (sun_axis: [0.0, 0.0, -1.0], the power face the guidance turns to the Sun when the product states none) |
| `gf_0` | act | stated | 3 Millimetre | the design, idmas_v2 |
| `gf_1` | act | stated | 0.022 SquareMetre | the design, idmas_v2 |
| `gf_2` | act | stated | 6440 KgPerCubicMetre | the design, idmas_v2 |
| `gf_3` | act | stated | 0.0024 PascalSecond | the design, idmas_v2 |
| `gf_4` | act | stated | 0.2 MetrePerSecond | the design, idmas_v2 |
| `gf_5` | act | stated | 0.72 Metre | the design, idmas_v2 |
| `gf_6` | act | computed | 0.000400591 (SI) | gf_6(d, s, rho, v) |
| `gf_7` | act | computed | 0.754688 (SI) | gf_7(d, rho, mu) |
| `gf_8` | act | computed | 0.00173718 (SI) | gf_8(h, mu, l, rho, d, s) |
| `gf_9` | act | computed | 56.5657 (SI) | pump_pressure_for_torque(tau, l, s, d) |
| `gm_0` | act | stated | 0.45 AmpereSquareMetre | the design, idmas_v2 |
| `gm_1` | act | computed | 0.0235377 (SI) | gm_1(tau_d, b_min) |
| `gm_2` | act | computed | 0.0235377 (SI) | gm_2(momentum, field, dump_time) |
| `gm_3` | act | computed | 1.04652e-05 (SI) | gm_3(m_av, b_min, n_mtq) |
| `gr_1` | act | stated | 0.8 Metre | the design, idmas_v2 |
| `gr_2` | act | stated | 220 Second | the design, idmas_v2 |
| `gr_3` | act | computed | 8.49422e-07 (SI) | propellant_per_slew(h, r, isp) |
| `gr_4` | act | computed | 0 (SI) | propellant_per_year(m_p, n_day, n_rcs) |
| `gw_3` | act | computed | 0.000733038 (SI) | gw_3(i_max, theta, t_slew) |
| `gw_4` | act | computed | 2.44346e-05 (SI) | gw_4(i_max, theta, t_slew) |
| `m1_1` | systems | stated | 27.0 Year | the case, mission.epoch |
| `m1_3` | systems | stated | 10.0 Count | the case, mission.spd |
| `m1_4` | systems | stated | 30.0 Degree | the case, mission.sangle |
| `m1_5` | systems | stated | 10.0 DegreePerSecond | the case, mission.w0 |
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
| `s1_4` | dyn | computed | [0.00599251, 0.0139825, -0.0129838] (SI) | cm_offset(cpa, dir) |
| `s2_0` | dyn | stated | 0.034 SquareMetre | the case, surface.afr |
| `s2_1` | dyn | stated | 0.02 Metre | the case, surface.cpa |
| `s2_2` | dyn | stated | 0.034 SquareMetre | the case, surface.asun |
| `s2_3` | dyn | stated | 0.02 Metre | the case, surface.cps |
| `s2_4` | dyn | stated | 0.6 One | the case, surface.refl |
| `s2_5` | dyn | stated | 2.2 One | the case, surface.cd |
| `s3_0` | dyn | stated | 0.01 AmpereSquareMetre | the case, magnetic.dres |
| `vv_record_step_default` | vv | stated | 1 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (json::f(time, record_dt_s, 1.0)) |
| `vv_run_duration_default` | vv | stated | 600 | the design, 1.0.0 code: engine/crates/adcs-sim/src/config.rs build (json::f(time, duration_s, 600.0)) |

**Not computed (984), by why:**

- no value, relation or pseudocode yet: 711 (act__cmg, act__vscmg, cas, cat, cf, cf_1, cf_4, cf_5, ci1, ci1_0, ci1_1, ci1_2, …)
- computed during a run (generated into adcs-sim-core): 72 (act_cmg_model, act_rotor_set, act_rotor_telemetry, act_vscmg_gimbal_limits, act_vscmg_model, dyn_flexible_mode, dyn_initial_state, dyn_rigid_body, dyn_rotor_coupling, dyn_total_momentum, env_calendar_time, env_moon_fast, …)
- computed during a run (generated into adcs-sim): 47 (env_orbit_start, fsw_param_alloc, fsw_param_bdot_law, fsw_param_es_noise, fsw_param_gim_axis, fsw_param_gim_rate_max, fsw_param_gps_latency, fsw_param_gyro_arw, fsw_param_gyro_rrw, fsw_param_has_es, fsw_param_has_gps, fsw_param_has_gyro, …)
- computed during a run (by the flight software or the engine, at each step): 23 (act_cmg_axes, act_cmg_steering, dyn_kinematics, fdir_rotor_health, fdir_safe_mode, fdir_sensor_health, fsw_allocation, fsw_control, fsw_drivers, fsw_estimation, fsw_guidance, fsw_modes, …)
- computed during a run (generated into adcs-pop): 22 (env_de440, env_density_model, env_drag_force, env_dtm2020_operational, env_dtm2020_research, env_earth_frames, env_erp_force, env_exponential_atmosphere, env_force_model, env_gas_surface, env_geodetic, env_gravity_field, …)
- a relation, but no pseudocode yet: its author writes it: 18 (rk1_0, rk1_1, rk1_2, rk1_3, rk1_4, rk2_0, rk2_1, rk2_2, rk2_3, rk2_4, rk2_5, rk3_0, …)
- needs an input that has no value: 11 (fsw_param_mtq_k1, fsw_param_mtq_k16, fsw_param_mtq_k2, fsw_param_mtq_lam16, fsw_param_sb_kd, fsw_param_sb_kp, gp_5, l3_pnt_row_09, rk4_0, rk4_1, rk4_2)
- its module has 2 functions and names none for this node (by its id or its output's symbol): 9 (gb_0, gb_1, gb_2, gb_3, l3_budget_row_06, l3_budget_row_07, l3_budget_row_08, l3_fmr_row_14, l3_fmr_row_15)
- its module has 7 functions and names none for this node (by its id or its output's symbol): 7 (design_sizing_cmg, design_sizing_rw, design_sizing_sensors, design_sizing_vscmg, gw_5, gw_6, l3_budget_row_03)
- its module has 5 functions and names none for this node (by its id or its output's symbol): 5 (design_sizing_mtq, gm_4, gm_5, gp_4, l3_budget_row_02)
- its module has 8 functions and names none for this node (by its id or its output's symbol): 3 (ctl_floquet_certificate, design_sizing_fmr, l3_budget_row_04)
- its module has 9 functions and names none for this node (by its id or its output's symbol): 3 (design_loop_converge, design_loop_redundancy, design_loop_robustness)
- its module has 6 functions and names none for this node (by its id or its output's symbol): 2 (catalogue_datasheet_derive, design_power_system)
- its module has 3 functions and names none for this node (by its id or its output's symbol): 2 (design_sizing_rcs, l3_budget_row_05)
- its module has 4 functions and names none for this node (by its id or its output's symbol): 2 (fsw_param_rw_Klqr, l3_fmr_row_07)
- its module has 13 functions and names none for this node (by its id or its output's symbol): 2 (gw_2, l3_budget_row_01)
- computed during a run (generated into flight): 1 (env_time_frames)
- its module has 10 functions and names none for this node (by its id or its output's symbol): 1 (fsw_param_mtq_Klqr)
- the case does not state pointing.et: 1 (gp_3)
- computed during a run (generated into adcs-sim, adcs-sim-core): 1 (kpi_metric_channels)
- the case does not state mission.life: 1 (m1_0)
- the case does not state mission.duty: 1 (m1_2)
- stated, with no value: its method is env_time_frames's since S7.3 (fsw/pseudocode/02_time_frames_models.pc, one copy); the pseudocode it carried from 1.0.0 stays as provenance: 1 (nav_time_frames)
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
- the case does not state mass.iunc: 1 (s1_5)
- the case does not state magnetic.dunc: 1 (s3_1)
- the case does not state flex.fmode: 1 (s4_0)
- the case does not state flex.mpart: 1 (s4_1)
- the case does not state resources.palloc: 1 (s5_0)
- the case does not state resources.malloc: 1 (s5_1)
- the case does not state resources.valloc: 1 (s5_2)
- the case does not state resources.vbus: 1 (s5_3)
- the case does not state resources.nif: 1 (s5_4)

