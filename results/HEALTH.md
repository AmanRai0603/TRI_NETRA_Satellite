# The health map

**In one line:** every node's health for each case, worst first, rolled up group by group to the ADCS; each closure's answer, its range verdict and tornado; and, for every closure that does not close, the nodes that cause it (`tools/health.py`, `docs/OPERATING_2_0.md` §6).

Health, worst first: **fails**, **refused**, **blocked**, **open**, **unproven**, **tight**, **closes**. A node shows the worst that applies and lists every one.

## Built-in: relations still in code

**0 built-in nodes**, relations still in compiled code (docs/PLAN_2_0.md S7 lowers this to zero): none.

Not counted: 5 nodes describe code by the boundary (docs/SYSTEM_MODEL.md §7): `l3_fsw_row_01` (runtime), `l3_fsw_row_02` (runtime), `l3_fsw_row_03` (runtime), `l3_fsw_row_04` (runtime), `l3_fsw_row_10` (test).

## ais_3u: the ADCS is **blocked**

From `tests/regression/design.tndb`.

| Group | Health | Nodes by health |
|---|---|---|
| act | **blocked** | 5 blocked, 50 open, 108 unproven |
| business | **open** | 32 open, 8 unproven |
| catalogue | **open** | 8 open, 95 unproven |
| ctl | **open** | 24 open, 16 unproven |
| design | **open** | 15 open, 41 unproven |
| dyn | **open** | 5 open, 36 unproven |
| env | **open** | 9 open, 97 unproven |
| fdir | **open** | 4 open, 6 unproven |
| fsw | **blocked** | 6 blocked, 21 open, 167 unproven |
| gdn | **open** | 17 open, 20 unproven |
| hils | **open** | 47 open, 19 unproven |
| kpi | **blocked** | 34 blocked, 39 open, 28 unproven |
| lab | **open** | 37 open, 6 unproven |
| nav | **open** | 22 open, 16 unproven |
| oils | **open** | 31 open, 11 unproven |
| pnt | **blocked** | 2 blocked, 18 open, 13 unproven |
| programme | **open** | 5 open, 16 unproven |
| risk | **blocked** | 3 blocked, 18 open, 4 unproven |
| sens | **open** | 14 open, 29 unproven |
| systems | **open** | 5 open, 36 unproven |
| vv | **open** | 20 open, 12 unproven |

| Closure | Answer | Range verdict | Tornado: widest bars (margin at low → at high) | Cause |
|---|---|---|---|---|
| `kpi_absolute_pointing_error_ape_verified` | pass | closes for part of it: 17 of 29 runs hold; it fails at edge_nadir_ais run 1, edge_nadir_ais run 3, edge_nadir_ais run 4, edge_nadir_ais run 6, edge_nadir_ais run 7, edge_nadir_ais run 8, … | residual_dipole (-0.296 → -1.94); solar_flux (-0.0115 → -0.186); cm_offset (0.0594 → -0.074) | residual_dipole in edge_nadir_ais: the widest bar: margin -0.2964 at its low end, -1.94 at its high end |
| `kpi_absolute_pointing_error_ape_analysis` | blocked | gp_5 has no value (needs terms[1] from gp_0 (not computed), terms[2] from gp_1 (not computed), terms[3] from gp_2 (not computed), terms[4] from gp_3 (not computed), terms[5] from gp_4 (not computed)) | — | `gp_0` (pnt): computed during a run (generated into adcs-sim): its inputs ake are the run's state and the product it flies, not rows of the design; `gp_1` (pnt): computed during a run (generated into adcs-sim): its inputs ape, ake are the run's state and the product it flies, not rows of the design; `gp_2` (pnt): computed during a run (generated into adcs-sim): its inputs a are the run's state and the product it flies, not rows of the design |
| `kpi_absolute_knowledge_error_ake_verified` | pass | closes for the whole range: all 29 runs hold (least margin 0.01994) | solar_flux (0.0643 → 0.0199); accommodation (0.0744 → 0.0345); cm_offset (0.0723 → 0.0472) | — |
| `kpi_absolute_knowledge_error_ake_analysis` | blocked | ge_5 has no value (no value, relation or pseudocode yet) | — | `ge_5` (nav): an open block: not decided yet |
| `kpi_relative_pointing_error_rpe_verified` | blocked | the requirement p1k_2 has no value (the case does not state req.rpe) | — | `p1k_2` (kpi): an open block: not decided yet |
| `kpi_relative_pointing_error_rpe_analysis` | blocked | the requirement p1k_2 has no value (the case does not state req.rpe) | — | `p1k_2` (kpi): an open block: not decided yet |
| `kpi_pointing_drift_error_pde_verified` | blocked | the requirement p1k_3 has no value (the case does not state req.pde) | — | `p1k_3` (kpi): an open block: not decided yet |
| `kpi_pointing_drift_error_pde_analysis` | blocked | the requirement p1k_3 has no value (the case does not state req.pde) | — | `p1k_3` (kpi): an open block: not decided yet |
| `kpi_rate_stability_verified` | blocked | the requirement p1k_4 has no value (the case does not state req.rks) | — | `p1k_4` (kpi): an open block: not decided yet |
| `kpi_relative_knowledge_error_rke_verified` | blocked | the requirement p1k_5 has no value (the case does not state req.rke) | — | `p1k_5` (kpi): an open block: not decided yet |
| `kpi_reference_slew_time_verified` | blocked | the requirement p2k_0 has no value (the case does not state req.slew) | — | `p2k_0` (kpi): the case does not state req.slew |
| `kpi_reference_slew_time_analysis` | blocked | the requirement p2k_0 has no value (the case does not state req.slew) | — | `p2k_0` (kpi): the case does not state req.slew |
| `kpi_settling_time_after_a_slew_verified` | blocked | the requirement p2k_1 has no value (the case does not state req.settle) | — | `p2k_1` (kpi): an open block: not decided yet |
| `kpi_settling_time_after_a_slew_analysis` | blocked | the requirement p2k_1 has no value (the case does not state req.settle) | — | `p2k_1` (kpi): an open block: not decided yet |
| `kpi_slews_per_orbit_verified` | blocked | the requirement p2k_2 has no value (the case does not state req.spo) | — | `p2k_2` (kpi): an open block: not decided yet |
| `kpi_target_tracking_rate_verified` | blocked | the requirement p2k_3 has no value (the case does not state req.track) | — | `p2k_3` (kpi): an open block: not decided yet |
| `kpi_maximum_body_rate_verified` | blocked | the requirement p2k_4 has no value (the case does not state req.wmax) | — | `p2k_4` (kpi): an open block: not decided yet |
| `kpi_detumble_time_verified` | pass | closes for the whole range: all 12 runs hold (least margin 1.246e+04) | — | — |
| `kpi_detumble_time_analysis` | blocked | gq_0 has no value (no value, relation or pseudocode yet) | — | `gq_0` (gdn): an open block: not decided yet |
| `kpi_sun_acquisition_time_in_safe_mode_verified` | blocked | p3a_1 has no value (evidence: no engine campaign of this case judges a metric against req.sunacq) | — | `p3a_1` (kpi): evidence that no run gives yet (evidence: no engine campaign of this case judges a metric against req.sunacq) |
| `kpi_sun_acquisition_time_in_safe_mode_analysis` | blocked | gq_1 has no value (no value, relation or pseudocode yet) | — | `gq_1` (gdn): an open block: not decided yet |
| `kpi_faults_tolerated_verified` | blocked | the requirement p3k_2 has no value (the case does not state req.faults) | — | `p3k_2` (kpi): an open block: not decided yet |
| `kpi_faults_tolerated_analysis` | blocked | the requirement p3k_2 has no value (the case does not state req.faults) | — | `p3k_2` (kpi): an open block: not decided yet |
| `kpi_recovery_time_after_a_single_fault_verified` | blocked | the requirement p3k_3 has no value (the case does not state req.recover) | — | `p3k_3` (kpi): an open block: not decided yet |
| `kpi_recovery_time_after_a_single_fault_analysis` | blocked | the requirement p3k_3 has no value (the case does not state req.recover) | — | `p3k_3` (kpi): an open block: not decided yet |
| `kpi_momentum_saturation_margin_verified` | blocked | the requirement p3k_4 has no value (the case does not state req.hsat) | — | `p3k_4` (kpi): an open block: not decided yet |
| `kpi_momentum_dump_interval_verified` | blocked | the requirement p3k_5 has no value (the case does not state req.dump) | — | `p3k_5` (kpi): an open block: not decided yet |
| `kpi_momentum_dump_interval_analysis` | blocked | the requirement p3k_5 has no value (the case does not state req.dump) | — | `p3k_5` (kpi): an open block: not decided yet |
| `kpi_adcs_mass_verified` | blocked | p4a_0 has no value (evidence: no engine campaign of this case judges a metric against req.mass) | — | `p4a_0` (kpi): evidence that no run gives yet (evidence: no engine campaign of this case judges a metric against req.mass) |
| `kpi_adcs_mass_analysis` | blocked | gb_0 has no value (its module has 2 functions and names none for this node (by its id or its output's symbol)) | — | `gb_0` (design): its module has 2 functions and names none for this node (by its id or its output's symbol) |
| `kpi_adcs_orbit_average_power_verified` | pass | closes for the whole range: all 29 runs hold (least margin 0.1446) | residual_dipole (0.493 → 0.483); cm_offset (0.489 → 0.489); inertia (0.489 → 0.489) | — |
| `kpi_adcs_orbit_average_power_analysis` | blocked | gb_1 has no value (its module has 2 functions and names none for this node (by its id or its output's symbol)) | — | `gb_1` (design): its module has 2 functions and names none for this node (by its id or its output's symbol) |
| `kpi_adcs_peak_power_verified` | blocked | p4a_2 has no value (evidence: no engine campaign of this case judges a metric against req.ppk) | — | `p4a_2` (kpi): evidence that no run gives yet (evidence: no engine campaign of this case judges a metric against req.ppk) |
| `kpi_adcs_peak_power_analysis` | blocked | gb_2 has no value (its module has 2 functions and names none for this node (by its id or its output's symbol)) | — | `gb_2` (design): its module has 2 functions and names none for this node (by its id or its output's symbol) |
| `kpi_adcs_volume_verified` | blocked | p4a_3 has no value (evidence: no engine campaign of this case judges a metric against req.vol) | — | `p4a_3` (kpi): evidence that no run gives yet (evidence: no engine campaign of this case judges a metric against req.vol) |
| `kpi_adcs_volume_analysis` | blocked | gb_3 has no value (its module has 2 functions and names none for this node (by its id or its output's symbol)) | — | `gb_3` (design): its module has 2 functions and names none for this node (by its id or its output's symbol) |
| `kpi_rcs_propellant_per_year_verified` | blocked | the requirement p4k_4 has no value (the case does not state req.prop) | — | `p4k_4` (kpi): an open block: not decided yet |
| `kpi_rcs_propellant_per_year_analysis` | blocked | the requirement p4k_4 has no value (the case does not state req.prop) | — | `p4k_4` (kpi): an open block: not decided yet |

## ais_img_3u: the ADCS is **fails**

From `tests/regression/design.tndb`.

| Group | Health | Nodes by health |
|---|---|---|
| act | **open** | 50 open, 113 unproven |
| business | **open** | 32 open, 8 unproven |
| catalogue | **open** | 8 open, 95 unproven |
| ctl | **fails** | 1 fails, 24 open, 15 unproven |
| design | **open** | 15 open, 41 unproven |
| dyn | **open** | 5 open, 36 unproven |
| env | **open** | 9 open, 97 unproven |
| fdir | **open** | 4 open, 6 unproven |
| fsw | **blocked** | 6 blocked, 21 open, 167 unproven |
| gdn | **open** | 17 open, 20 unproven |
| hils | **open** | 47 open, 19 unproven |
| kpi | **fails** | 1 fails, 33 blocked, 38 open, 29 unproven |
| lab | **open** | 37 open, 6 unproven |
| nav | **open** | 22 open, 16 unproven |
| oils | **open** | 31 open, 11 unproven |
| pnt | **blocked** | 2 blocked, 18 open, 13 unproven |
| programme | **open** | 5 open, 16 unproven |
| risk | **blocked** | 3 blocked, 18 open, 4 unproven |
| sens | **open** | 14 open, 29 unproven |
| systems | **open** | 5 open, 36 unproven |
| vv | **open** | 20 open, 12 unproven |

| Closure | Answer | Range verdict | Tornado: widest bars (margin at low → at high) | Cause |
|---|---|---|---|---|
| `kpi_absolute_pointing_error_ape_verified` | pass | closes for the whole range: all 27 runs hold (least margin 4.602e-05) | initial_error_deg (7.24e-05 → 5.04e-05); solar_flux (5.72e-05 → 4.6e-05); cm_offset (5.98e-05 → 6.88e-05) | — |
| `kpi_absolute_pointing_error_ape_analysis` | blocked | gp_5 has no value (needs terms[1] from gp_0 (not computed), terms[2] from gp_1 (not computed), terms[3] from gp_2 (not computed), terms[4] from gp_3 (not computed), terms[5] from gp_4 (not computed)) | — | `gp_0` (pnt): computed during a run (generated into adcs-sim): its inputs ake are the run's state and the product it flies, not rows of the design; `gp_1` (pnt): computed during a run (generated into adcs-sim): its inputs ape, ake are the run's state and the product it flies, not rows of the design; `gp_2` (pnt): computed during a run (generated into adcs-sim): its inputs a are the run's state and the product it flies, not rows of the design |
| `kpi_absolute_knowledge_error_ake_verified` | pass | closes for the whole range: all 27 runs hold (least margin 3.164e-06) | solar_flux (4.02e-05 → 1.26e-05); reflectivity (3.38e-05 → 4.16e-05); initial_error_deg (4.04e-05 → 4.69e-05) | — |
| `kpi_absolute_knowledge_error_ake_analysis` | blocked | ge_5 has no value (no value, relation or pseudocode yet) | — | `ge_5` (nav): an open block: not decided yet |
| `kpi_relative_pointing_error_rpe_verified` | blocked | the requirement p1k_2 has no value (the case does not state req.rpe) | — | `p1k_2` (kpi): an open block: not decided yet |
| `kpi_relative_pointing_error_rpe_analysis` | blocked | the requirement p1k_2 has no value (the case does not state req.rpe) | — | `p1k_2` (kpi): an open block: not decided yet |
| `kpi_pointing_drift_error_pde_verified` | blocked | the requirement p1k_3 has no value (the case does not state req.pde) | — | `p1k_3` (kpi): an open block: not decided yet |
| `kpi_pointing_drift_error_pde_analysis` | blocked | the requirement p1k_3 has no value (the case does not state req.pde) | — | `p1k_3` (kpi): an open block: not decided yet |
| `kpi_rate_stability_verified` | blocked | p1a_4 has no value (evidence: no engine campaign of this case judges a metric against req.rks) | — | `p1a_4` (kpi): evidence that no run gives yet (evidence: no engine campaign of this case judges a metric against req.rks) |
| `kpi_relative_knowledge_error_rke_verified` | blocked | the requirement p1k_5 has no value (the case does not state req.rke) | — | `p1k_5` (kpi): an open block: not decided yet |
| `kpi_reference_slew_time_verified` | blocked | p2a_0 has no value (evidence: no engine campaign of this case judges a metric against req.slew) | — | `p2a_0` (kpi): evidence that no run gives yet (evidence: no engine campaign of this case judges a metric against req.slew) |
| `kpi_reference_slew_time_analysis` | blocked | gc_5 has no value (no value, relation or pseudocode yet) | — | `gc_5` (ctl): an open block: not decided yet |
| `kpi_settling_time_after_a_slew_verified` | blocked | p2a_1 has no value (evidence: no engine campaign of this case judges a metric against req.settle) | — | `p2a_1` (kpi): evidence that no run gives yet (evidence: no engine campaign of this case judges a metric against req.settle) |
| `kpi_settling_time_after_a_slew_analysis` | fail | by analysis: the design graph gives one value; its inputs carry no range yet | — | — |
| `kpi_slews_per_orbit_verified` | blocked | the requirement p2k_2 has no value (the case does not state req.spo) | — | `p2k_2` (kpi): an open block: not decided yet |
| `kpi_target_tracking_rate_verified` | blocked | the requirement p2k_3 has no value (the case does not state req.track) | — | `p2k_3` (kpi): an open block: not decided yet |
| `kpi_maximum_body_rate_verified` | blocked | the requirement p2k_4 has no value (the case does not state req.wmax) | — | `p2k_4` (kpi): an open block: not decided yet |
| `kpi_detumble_time_verified` | pass | closes for the whole range: all 12 runs hold (least margin 1.225e+04) | — | — |
| `kpi_detumble_time_analysis` | blocked | gq_0 has no value (no value, relation or pseudocode yet) | — | `gq_0` (gdn): an open block: not decided yet |
| `kpi_sun_acquisition_time_in_safe_mode_verified` | blocked | p3a_1 has no value (evidence: no engine campaign of this case judges a metric against req.sunacq) | — | `p3a_1` (kpi): evidence that no run gives yet (evidence: no engine campaign of this case judges a metric against req.sunacq) |
| `kpi_sun_acquisition_time_in_safe_mode_analysis` | blocked | gq_1 has no value (no value, relation or pseudocode yet) | — | `gq_1` (gdn): an open block: not decided yet |
| `kpi_faults_tolerated_verified` | blocked | the requirement p3k_2 has no value (the case does not state req.faults) | — | `p3k_2` (kpi): an open block: not decided yet |
| `kpi_faults_tolerated_analysis` | blocked | the requirement p3k_2 has no value (the case does not state req.faults) | — | `p3k_2` (kpi): an open block: not decided yet |
| `kpi_recovery_time_after_a_single_fault_verified` | blocked | the requirement p3k_3 has no value (the case does not state req.recover) | — | `p3k_3` (kpi): an open block: not decided yet |
| `kpi_recovery_time_after_a_single_fault_analysis` | blocked | the requirement p3k_3 has no value (the case does not state req.recover) | — | `p3k_3` (kpi): an open block: not decided yet |
| `kpi_momentum_saturation_margin_verified` | blocked | the requirement p3k_4 has no value (the case does not state req.hsat) | — | `p3k_4` (kpi): an open block: not decided yet |
| `kpi_momentum_dump_interval_verified` | blocked | the requirement p3k_5 has no value (the case does not state req.dump) | — | `p3k_5` (kpi): an open block: not decided yet |
| `kpi_momentum_dump_interval_analysis` | blocked | the requirement p3k_5 has no value (the case does not state req.dump) | — | `p3k_5` (kpi): an open block: not decided yet |
| `kpi_adcs_mass_verified` | blocked | p4a_0 has no value (evidence: no engine campaign of this case judges a metric against req.mass) | — | `p4a_0` (kpi): evidence that no run gives yet (evidence: no engine campaign of this case judges a metric against req.mass) |
| `kpi_adcs_mass_analysis` | blocked | gb_0 has no value (its module has 2 functions and names none for this node (by its id or its output's symbol)) | — | `gb_0` (design): its module has 2 functions and names none for this node (by its id or its output's symbol) |
| `kpi_adcs_orbit_average_power_verified` | pass | closes for the whole range: all 27 runs hold (least margin 0.4377) | residual_dipole (0.439 → 0.438); solar_flux (0.441 → 0.44); cm_offset (0.44 → 0.441) | — |
| `kpi_adcs_orbit_average_power_analysis` | blocked | gb_1 has no value (its module has 2 functions and names none for this node (by its id or its output's symbol)) | — | `gb_1` (design): its module has 2 functions and names none for this node (by its id or its output's symbol) |
| `kpi_adcs_peak_power_verified` | blocked | p4a_2 has no value (evidence: no engine campaign of this case judges a metric against req.ppk) | — | `p4a_2` (kpi): evidence that no run gives yet (evidence: no engine campaign of this case judges a metric against req.ppk) |
| `kpi_adcs_peak_power_analysis` | blocked | gb_2 has no value (its module has 2 functions and names none for this node (by its id or its output's symbol)) | — | `gb_2` (design): its module has 2 functions and names none for this node (by its id or its output's symbol) |
| `kpi_adcs_volume_verified` | blocked | p4a_3 has no value (evidence: no engine campaign of this case judges a metric against req.vol) | — | `p4a_3` (kpi): evidence that no run gives yet (evidence: no engine campaign of this case judges a metric against req.vol) |
| `kpi_adcs_volume_analysis` | blocked | gb_3 has no value (its module has 2 functions and names none for this node (by its id or its output's symbol)) | — | `gb_3` (design): its module has 2 functions and names none for this node (by its id or its output's symbol) |
| `kpi_rcs_propellant_per_year_verified` | blocked | the requirement p4k_4 has no value (the case does not state req.prop) | — | `p4k_4` (kpi): an open block: not decided yet |
| `kpi_rcs_propellant_per_year_analysis` | blocked | the requirement p4k_4 has no value (the case does not state req.prop) | — | `p4k_4` (kpi): an open block: not decided yet |

**Nodes that fail, refuse or are tight:**

- `gc_2` (ctl): fails: unproven: converted from 1.0.0, not yet signed by a person; fails: decides kpi_settling_time_after_a_slew_analysis, which fails
- `kpi_settling_time_after_a_slew_analysis` (kpi): fails: unproven: converted from 1.0.0, not yet signed by a person; the developer's revision S7.1b (2026-10-07), not yet signed by a person: behaviour built-in -> closure: a KPI closure: the library compares its requirement with its achieved value in the requirement's sense (its closure row); no code computes it; fails: gc_2 = 56.5771 (SI) <= 20.0 Second (p2k_1)

