# S7 inventory: every relation still in code

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.

S7 step 1 (`docs/PLAN_2_0.md`, S7). It lists every relation that still lives only in code, which S7 writes as a
method in the design and then generates. Read from the code at `cac5aac` and from the regression copy
(`tests/regression/design.tndb`). Nothing was changed but this page and `results/s7_inventory.json` (the same
content, for tools). Line counts are of the code as it stands.

## Counts

**Now, after S7.1b (7 Oct 2026): 91 built-in nodes**, the relations computed in code (act 38, design 19, dyn 4,
env 11, oils 1, pnt 4, sens 14). Behaviours of the 1,155 nodes: open 304, stated 297, children 134, evidence 119,
lookup 117, built-in 91, method 55, closure 38. `python3 tools/health.py tests/regression --built-in` counts them;
`tests/test_built_in.py` holds the list, and each S7 step takes its nodes out of it. The counts below are the
inventory's, at `cac5aac`.

- **Built-in nodes: 271** of 1,155 (behaviours: stated 292, open 286, built-in 271, children 134, lookup 117, method 55).
  - **91 are relations computed in code** (90 to transcribe, 1 to decide): act 38, design 19, dyn 4, env 11, oils 1, pnt 4, sens 14.
  - **180 are not relations in code**: 94 achieved holders, 38 KPI closures,
    43 system leafs whose only link to code is the 1.0.0 owner pointer, 5 flight-software runtime rows. The
    conversion made them built-in because `behaviour()` in `tools/convert_2_0.py` marks any node with a `code.*`
    field, and every achieved or closure kind, as built-in. They need a reclassification, not a method.
- **147 `fsw_param_*` nodes are stated with no value.** Every one is computed in `adcs-sim/src/config.rs`
  (about 47 by a law, 9 choices, 38 constants, 29 copies of a part or case value, 24 scenario values). They are
  relations in code that the built-in count does not see.
- **Engine relations in code:** 32 items in `adcs-sim-core`, `adcs-sim` and `adcs-design`, 2,492 lines.
  Three already have their method or pseudocode in the design (density table, solar pressure, IGRF and frames): only the call is missing.
- **adcs-pop:** 23 modules toolbox (6,506 lines: time, frames, EOP, DE440 reading, DTM2020, JB2008,
  space weather, integrators, gravity field, tides, relativity); 9 modules relation (2,121 lines: the spacecraft force
  models and the force sum).
- **adcs-physics:** all generated (39 relations, 41 functions, 1,003 lines); nothing in the engine calls it.
- **Tools (Python):** 4 relations (613 lines): pointing terms, catalogue derive rule, Floquet
  certificate, design-loop rules.
- **MATLAB twin:** 160 generated files (2,896 lines: +physics, +groups, +pc, +pcselftest);
  90 hand-written files in 9 packages that hold relations (3,175 lines), the flight algorithms
  of +fsw among them; the POP (256 files, 19,828 lines) is toolbox.
- **Translator gaps:** 13 (section 4). G1, G3, G4, G5 and G6 block the device models.

## 1 · The built-in nodes

Per group and class. *relation*: computed in code, to be written as a method. *achieved*: holds a run's
answer. *closure*: a KPI comparison. *no code*: nothing computes it. *runtime*: describes runtime code.

| group | relation | decide | achieved | closure | no_code | runtime | total |
|---|---|---|---|---|---|---|---|
| act | 38 |  | 28 |  |  |  | 66 |
| catalogue |  |  |  |  | 2 |  | 2 |
| ctl |  |  | 6 |  | 3 |  | 9 |
| design | 19 |  | 4 |  | 2 |  | 25 |
| dyn | 4 |  |  |  |  |  | 4 |
| env | 11 |  | 6 |  |  |  | 17 |
| fdir |  |  | 2 |  | 2 |  | 4 |
| fsw |  |  | 5 |  | 4 | 5 | 14 |
| gdn |  |  | 3 |  | 3 |  | 6 |
| hils |  |  | 16 |  | 16 |  | 32 |
| kpi |  |  |  | 38 |  |  | 38 |
| nav |  |  | 6 |  | 5 |  | 11 |
| oils |  | 1 | 5 |  | 6 |  | 12 |
| pnt | 4 |  | 6 |  |  |  | 10 |
| sens | 14 |  | 7 |  |  |  | 21 |
| **all** | 90 | 1 | 94 | 38 | 43 | 5 | 271 |

### 1.1 The relations to write (91)

| node | group | label | computed in (file:function) | twin | step |
|---|---|---|---|---|---|
| act_cmg_model | act | Control moment gyro model | adcs-sim-core/src/actuators.rs:Mex::apply (Cmg) | +devices/mtq.m, mex.m, rcs.m | S7.7 |
| act_vscmg_gimbal_limits | act | Gimbal rate limit and power | adcs-sim-core/src/actuators.rs:Mex::apply (gimbals) | +devices/mtq.m, mex.m, rcs.m | S7.7 |
| act_vscmg_model | act | Variable-speed CMG model | adcs-sim-core/src/actuators.rs:Mex::apply (Vscmg) | +devices/mtq.m, mex.m, rcs.m | S7.7 |
| gm_4 | act | Magnetorquer power | adcs-design/src/lib.rs:mtq (power_at_max_W = 0.3 m/0.45) | +sizing | S7.15 |
| gm_5 | act | Magnetorquer mass | adcs-design/src/lib.rs:mtq (mass_kg = 0.03 m/0.45) | +sizing | S7.15 |
| gw_2 | act | Cyclic momentum to store | adcs-design/src/lib.rs:survey, demand (h_cyclic) | +sizing | S7.15 |
| gw_5 | act | Wheel power | adcs-design/src/lib.rs:rotor (catalogue power_steady_W) | +sizing | S7.15 |
| gw_6 | act | Wheel mass | adcs-design/src/lib.rs:rotor (catalogue mass_kg) | +sizing | S7.15 |
| l3_fmr_row_07 | act | Electromagnetic pump design | adcs-design/src/empump.rs:design, pareto, friction | none (twin +sizing/fmr.m has no pump); +sizing | S7.15 |
| l3_fmr_row_08 | act | Pump efficiency over its range | adcs-sim-core/src/actuators.rs:Mex::new (eta lo..hi), Mex::apply (Fmr) | +devices/mtq.m, mex.m, rcs.m | S7.7 |
| l3_fmr_row_09 | act | Flow response time | adcs-sim-core/src/actuators.rs:Mex::apply (Fmr: flow_tau filter) | +devices/mtq.m, mex.m, rcs.m | S7.7 |
| l3_fmr_row_10 | act | Flow noise | adcs-sim-core/src/actuators.rs:Mex::apply (Fmr: flow_noise_h) | +devices/mtq.m, mex.m, rcs.m | S7.7 |
| l3_fmr_row_11 | act | Ring field power | adcs-sim-core/src/actuators.rs:Mex::apply (Fmr: field_on hysteresis) | +devices/mtq.m, mex.m, rcs.m | S7.7 |
| l3_fmr_row_12 | act | Ring flow limits | adcs-sim-core/src/actuators.rs:Mex::apply (Fmr: h_max, pump clamp); adcs-sim/src/product.rs:fit_actuator_ (k_hv, tsd, hmax) | +devices/mtq.m, mex.m, rcs.m; +product/load.m | S7.7 |
| l3_fmr_row_13 | act | Ring misalignment | adcs-sim-core/src/actuators.rs:Mex::new (misalignment) | +devices/mtq.m, mex.m, rcs.m | S7.7 |
| l3_fmr_row_14 | act | Spare ring axis | adcs-design/src/lib.rs:fmr, spare_axis | +sizing | S7.15 |
| l3_fmr_row_15 | act | Ring cross-section in the box | adcs-design/src/lib.rs:section, ring | +sizing | S7.15 |
| l3_mtq_row_02 | act | Coil time constant (L/R lag) | adcs-sim-core/src/actuators.rs:lag | +devices/mtq.m, mex.m, rcs.m | S7.7 |
| l3_mtq_row_03 | act | Dipole saturation per coil | adcs-sim-core/src/actuators.rs:Mtq::apply | +devices/mtq.m, mex.m, rcs.m | S7.7 |
| l3_mtq_row_04 | act | Coil power at a dipole | adcs-sim-core/src/actuators.rs:Mtq::apply | +devices/mtq.m, mex.m, rcs.m | S7.7 |
| l3_mtq_row_05 | act | Coil axes and their pseudo-inverse | adcs-sim-core/src/actuators.rs:Mtq::new | +devices/mtq.m, mex.m, rcs.m | S7.7 |
| l3_mtq_row_12 | act | Coil scale-factor dispersion | adcs-sim-core/src/actuators.rs:Mtq::new | +devices/mtq.m, mex.m, rcs.m | S7.7 |
| l3_mtq_row_13 | act | A failed coil and its reallocation | adcs-sim-core/src/actuators.rs:Mtq::apply | +devices/mtq.m, mex.m, rcs.m | S7.7 |
| l3_rcs_row_01 | act | Thruster torque directions | adcs-sim/src/product.rs:fit_actuator_ (rcs couples 2 F arm); adcs-sim-core/src/actuators.rs:Rcs::apply | +devices/mtq.m, mex.m, rcs.m; +product/load.m | S7.7 |
| l3_rcs_row_02 | act | Thrust level and dispersion | adcs-sim-core/src/actuators.rs:Rcs::new, Rcs::apply | +devices/mtq.m, mex.m, rcs.m | S7.7 |
| l3_rcs_row_03 | act | Specific impulse and its range | adcs-sim-core/src/actuators.rs:Rcs::new (isp lo..hi) | +devices/mtq.m, mex.m, rcs.m | S7.7 |
| l3_rcs_row_04 | act | Minimum impulse bit | adcs-sim-core/src/actuators.rs:Rcs::apply (mib, res) | +devices/mtq.m, mex.m, rcs.m | S7.7 |
| l3_rcs_row_08 | act | Propellant remaining | adcs-sim-core/src/actuators.rs:Rcs::apply; adcs-sim/src/run.rs:actuate (tank empty) | +devices/mtq.m, mex.m, rcs.m; run.m | S7.7 |
| l3_rcs_row_09 | act | Valve power | adcs-sim-core/src/actuators.rs:Rcs::apply | +devices/mtq.m, mex.m, rcs.m | S7.7 |
| l3_rcs_row_10 | act | Thruster misalignment | adcs-sim-core/src/actuators.rs:Rcs::new | +devices/mtq.m, mex.m, rcs.m | S7.7 |
| l3_rcs_row_11 | act | A failed thruster | adcs-sim-core/src/actuators.rs:Rcs::apply | +devices/mtq.m, mex.m, rcs.m | S7.7 |
| l3_rw_row_01 | act | Wheel motor torque-speed line | adcs-sim-core/src/actuators.rs:wheel_motor; adcs-sim/src/product.rs:fit_actuator_ (t_stall, w_nl) | +devices/mtq.m, mex.m, rcs.m; +product/load.m | S7.7 |
| l3_rw_row_02 | act | Wheel speed limit | adcs-sim-core/src/actuators.rs:wheel_motor, Mex::apply | +devices/mtq.m, mex.m, rcs.m | S7.7 |
| l3_rw_row_03 | act | Wheel bearing friction (Coulomb, viscous, Stribeck) | adcs-sim-core/src/actuators.rs:Mex::apply (Rw: Coulomb, viscous, Stribeck, Karnopp) | +devices/mtq.m, mex.m, rcs.m | S7.7 |
| l3_rw_row_04 | act | Wheel friction compensation | adcs-sim-core/src/actuators.rs:Mex::apply (friction_comp) | +devices/mtq.m, mex.m, rcs.m | S7.7 |
| l3_rw_row_05 | act | Wheel torque noise | adcs-sim-core/src/actuators.rs:Mex::apply (torque_noise) | +devices/mtq.m, mex.m, rcs.m | S7.7 |
| l3_rw_row_06 | act | Wheel misalignment | adcs-sim-core/src/actuators.rs:Mex::new (misalignment) | +devices/mtq.m, mex.m, rcs.m | S7.7 |
| l3_rw_row_07 | act | Wheel steady power | adcs-sim-core/src/actuators.rs:Mex::apply (p_steady + \|tau om\|/eta) | +devices/mtq.m, mex.m, rcs.m | S7.7 |
| design_sizing_cmg | design | CMG sizing | adcs-design/src/lib.rs:rotor ("cmg") | +sizing | S7.15 |
| design_sizing_fmr | design | Fluid ring sizing | adcs-design/src/lib.rs:fmr, ring; adcs-design/src/empump.rs:design | none (twin +sizing/fmr.m has no pump); +sizing | S7.15 |
| design_sizing_mtq | design | Magnetorquer sizing | adcs-design/src/lib.rs:mtq | +sizing | S7.15 |
| design_sizing_rcs | design | Thruster sizing | adcs-design/src/lib.rs:rcs | +sizing | S7.15 |
| design_sizing_rw | design | Reaction wheel sizing | adcs-design/src/lib.rs:rotor ("rw") | +sizing | S7.15 |
| design_sizing_sensors | design | Sensor suite sizing | adcs-design/src/lib.rs:size_all (gyro grade, star-tracker heads) | +sizing | S7.15 |
| design_sizing_vscmg | design | VSCMG sizing | adcs-design/src/lib.rs:rotor ("vscmg") | +sizing | S7.15 |
| gb_0 | design | ADCS mass total | adcs-design/src/lib.rs:budget (mass) | +sizing | S7.15 |
| gb_1 | design | ADCS orbit-average power total | adcs-design/src/lib.rs:budget (power) | +sizing | S7.15 |
| gb_2 | design | ADCS peak power total | adcs-design/src/lib.rs:budget (peak: not computed, the steady sum is used) | +sizing | S7.15 |
| gb_3 | design | ADCS volume total | adcs-design/src/lib.rs:budget (volume) | +sizing | S7.15 |
| l3_budget_row_01 | design | Demand survey | adcs-design/src/lib.rs:survey, demand | +sizing | S7.15 |
| l3_budget_row_02 | design | Magnetorquer sizing | adcs-design/src/lib.rs:mtq | +sizing | S7.15 |
| l3_budget_row_03 | design | Rotor sizing (wheels, CMG, VSCMG) | adcs-design/src/lib.rs:rotor | +sizing | S7.15 |
| l3_budget_row_04 | design | Fluid ring sizing | adcs-design/src/lib.rs:fmr, ring | +sizing | S7.15 |
| l3_budget_row_05 | design | Thruster sizing | adcs-design/src/lib.rs:rcs | +sizing | S7.15 |
| l3_budget_row_06 | design | Mass budget | adcs-design/src/lib.rs:budget | +sizing | S7.15 |
| l3_budget_row_07 | design | Power budget | adcs-design/src/lib.rs:budget | +sizing | S7.15 |
| l3_budget_row_08 | design | Volume budget | adcs-design/src/lib.rs:budget | +sizing | S7.15 |
| dyn_flexible_mode | dyn | Flexible mode (hybrid coordinates) | adcs-sim-core/src/plant.rs:Body::flexible, deriv; adcs-sim/src/config.rs:build (delta) | +plant/deriv.m; config.m | S7.5 |
| dyn_rigid_body | dyn | Rigid-body attitude dynamics | adcs-sim-core/src/plant.rs:deriv | +plant/deriv.m | S7.5 |
| dyn_rotor_coupling | dyn | Rotor and ring momentum coupling | adcs-sim-core/src/plant.rs:Geometry::new, axes, deriv | +plant/deriv.m | S7.5 |
| dyn_total_momentum | dyn | Total angular momentum | adcs-sim-core/src/plant.rs:momentum | +plant/deriv.m | S7.5 |
| l3_dist_row_01 | env | Gravity-gradient torque | adcs-sim-core/src/torques.rs:torques (gg) | +env/torques.m, earth_pressure.m, geometry.m | S7.4 |
| l3_dist_row_02 | env | Aerodynamic torque per facet | adcs-sim-core/src/torques.rs:torques (aero), Facets::boxed | +env/torques.m, earth_pressure.m, geometry.m | S7.4 |
| l3_dist_row_03 | env | Solar radiation pressure torque per facet | adcs-sim-core/src/torques.rs:radiation, torques (srp) | +env/torques.m, earth_pressure.m, geometry.m | S7.4 |
| l3_dist_row_04 | env | Earth albedo pressure torque | adcs-sim-core/src/torques.rs:earth_pressure (albedo) | +env/torques.m, earth_pressure.m, geometry.m | S7.4 |
| l3_dist_row_05 | env | Earth infrared pressure torque | adcs-sim-core/src/torques.rs:earth_pressure (IR) | +env/torques.m, earth_pressure.m, geometry.m | S7.4 |
| l3_dist_row_06 | env | Residual magnetic dipole torque | adcs-sim-core/src/torques.rs:torques (mag) | +env/torques.m, earth_pressure.m, geometry.m | S7.4 |
| l3_dist_row_07 | env | Atmospheric density at altitude | adcs-sim-core/src/atmos.rs:density (fast model); adcs-pop/src/atmos/dtm2020.rs (pop model, toolbox) | pop DTM2020 | S7.3 |
| l3_dist_row_08 | env | Main field along the orbit (IGRF) | adcs-sim-core/src/field.rs:gh, geodetic, ned, eci_at | +env/igrf_*.m, field.m | S7.3 |
| l3_dist_row_09 | env | Sun direction and distance | adcs-sim-core/src/ephem.rs:sun (fast); adcs-pop/src/ephem.rs (DE440, toolbox) | +env/shadow.m; pop DE440 | S7.4 |
| l3_dist_row_10 | env | Earth shadow | adcs-sim-core/src/ephem.rs:shadow | +env/shadow.m; pop DE440 | S7.4 |
| m2_7 | env | Eclipse fraction | adcs-design/src/lib.rs:survey (eclipse_frac over one orbit) | +sizing | S7.4 |
| l3_oils_row_07 | oils | Byte-level sensor emulation | adcs-sim-core/src/emu.rs (the device codecs) | +hal/pack_sensors.m, lsb.m | S7.12 |
| gp_0 | pnt | Knowledge contribution | tools/pointing_budget.py (flown AKE p99.73) | none | S7.14 |
| gp_1 | pnt | Control contribution | tools/pointing_budget.py (sqrt(APE^2 - AKE^2)) | none | S7.14 |
| gp_2 | pnt | Alignment contribution | tools/pointing_budget.py (product payload_alignment_rad) | none | S7.14 |
| gp_4 | pnt | Jitter contribution | adcs-sim/src/metrics.rs:jitter (= asils.sizing.jitter) | +sizing/jitter.m | S7.14 |
| l3_sens_row_01 | sens | Gyro angle random walk | adcs-sim-core/src/sensors.rs:Gyro::sample | +devices | S7.8 |
| l3_sens_row_02 | sens | Gyro bias and its random walk | adcs-sim-core/src/sensors.rs:Gyro::new, sample | +devices | S7.8 |
| l3_sens_row_03 | sens | Gyro scale factor and misalignment | adcs-sim-core/src/sensors.rs:Gyro::new | +devices | S7.8 |
| l3_sens_row_04 | sens | Magnetometer noise and bias | adcs-sim-core/src/sensors.rs:Mag::new, sample | +devices | S7.8 |
| l3_sens_row_05 | sens | Magnetometer scale factor, misalignment and range | adcs-sim-core/src/sensors.rs:Mag::new, sample | +devices | S7.8 |
| l3_sens_row_06 | sens | Coarse Sun sensor cosine response with albedo | adcs-sim-core/src/sensors.rs:Css::new, sample | +devices | S7.8 |
| l3_sens_row_07 | sens | Fine Sun sensor quadrant currents | adcs-sim-core/src/comp.rs:sun_sensor::currents | +comp | S7.8 |
| l3_sens_row_08 | sens | Fine Sun sensor angles from currents | adcs-sim-core/src/comp.rs:sun_sensor::angles | +comp | S7.8 |
| l3_sens_row_09 | sens | Star tracker frame rendering | adcs-sim-core/src/comp.rs:star_tracker::render; sensors.rs:star | +devices; +comp | S7.9 |
| l3_sens_row_10 | sens | Star tracker centroiding | adcs-sim-core/src/comp.rs:star_tracker::centroid | +comp | S7.10 |
| l3_sens_row_11 | sens | Star tracker identification by pair angles | adcs-sim-core/src/comp.rs:star_tracker::pairs, identify | +comp | S7.10 |
| l3_sens_row_12 | sens | Star tracker attitude and residual check | adcs-sim-core/src/comp.rs:star_tracker::attitude, chain; sensors.rs:St::sample_with | +devices; +comp | S7.9 |
| l3_sens_row_13 | sens | Earth sensor horizon measurement | adcs-sim-core/src/sensors.rs:Es::sample | +devices | S7.8 |
| l3_sens_row_14 | sens | GNSS fix noise and latency | adcs-sim-core/src/sensors.rs:Gps::history, delayed, sample; adcs-sim/src/run.rs:sense (ECEF fix) | +devices; run.m | S7.8 |

### 1.2 The built-in nodes that are not relations in code (180)

**Done in S7.1b** (the developer's revision S7.1b, `design/revisions_2_0.toml`, applied by the conversion; each node
sealed unconfirmed with its reason): the 94 achieved holders are **evidence**, and so are the 22 KPI evidence rows
(`p*a_*`, open until then, which name their campaign metric); the 38 KPI closures are **closure** (the library compares
each one's requirement and achieved rows; `tools/evaluate.py` reads the design's KPI table, which every closure row
equals, and `tests/test_built_in.py` holds them equal); gx_1, gx_3 and fa1_1 are **evidence** (soft OILS and the
campaigns measure them); the other 40 owner pointers are **open** (their `code.*` kept as provenance); the 5 runtime
rows are **stated**, marked `code.boundary` (runtime, or test for l3_fsw_row_10). The fates below are as proposed;
where a row says "evidence" for a rig need (v3_*, v4_*), the need is not measured, so it is open until oils or hils
states it.

**Achieved holders (94)**, `l3_*_<row>_achieved`, one per system row: act 28, ctl 6, design 4, env 6, fdir 2, fsw 5, gdn 3, hils 16, nav 6, oils 5, pnt 6, sens 7.
No code computes them; a run or a campaign answers them (`tools/evaluate.py`, evidence). Fate: an evidence
wire from the metric that answers them.

**KPI closures (38)**, `kpi_<slug>_analysis` and `kpi_<slug>_verified`. The comparison is `tools/evaluate.py:evaluate`
(and the generated `mission::closure`). Fate: a closure the library checks, with its own behaviour.

**Runtime rows (5):** l3_fsw_row_01, l3_fsw_row_02, l3_fsw_row_03, l3_fsw_row_04, l3_fsw_row_10 (tick, parameter tables, HAL, drivers, C and Rust parity).
Fate: stated descriptions; the drivers' conversions are already the `fsw_drivers` method.

**System leafs with no computing code (43).** Their `code.*` field names the 1.0.0 owner's code, which does not
compute them.

| nodes | fate |
|---|---|
| ct1_4, hr1_2, ct2_2, ct3_2 | programme statistic: no code computes it; open |
| gc_3, gc_4, gc_5 | authority estimate: no code computes it; open until ctl writes the method |
| gq_2, gq_3 | analysis estimate: no code computes it; open until fdir writes the method |
| gx_1, gx_3 | measured by soft OILS (tools/engine_oils.py): evidence, not a relation |
| gx_2, gx_4 | no code computes it; open (fsw states it) |
| gq_0, gq_1, gq_4 | analysis estimate: no code computes it; open until gdn writes the method, or evidence from campaigns |
| v4_0, v4_1, v4_10, v4_11, v4_12, v4_13, v4_14, v4_15, v4_2, v4_3, v4_4, v4_5, v4_6, v4_7, v4_8, v4_9 | HILS rig need: evidence from the runs' stimulus (+hal/stimulus.m), not a relation in code |
| ge_1, ge_2, ge_3, ge_4, ge_5 | analysis estimate: no code computes it; open until nav writes the method, or evidence from campaigns |
| fa1_1 | measured (runs per hour): evidence |
| v3_0, v3_1, v3_2, v3_3, v3_4 | OILS rig need: evidence from soft OILS runs, or stated by oils |

### 1.3 Relations in code with no node

- Moon position (`adcs-sim-core/ephem.rs:moon`).
- Fast orbit forces and their constants (`orbit.rs:accel`, `ctx`); RAAN from LTAN (`adcs-sim/run.rs:Truth::new`); `sso_initial` (`adcs-pop/accel.rs`).
- Initial attitude, rate and rotor momenta (`run.rs:initial_state`).
- Rotor telemetry noise 1e-7 N m s and gimbal-angle noise 1e-5 rad (`run.rs:sense`).
- Synthetic star catalogue (`sensors.rs:star`, `+devices/star_catalogue.m`).
- Power system: array power, state of charge, solar constant 1361 W/m^2 (`metrics.rs:PowerSystem`).
- Plant values in `config.rs:build`: centre-of-mass direction [0.30, 0.70, -0.65], residual dipole split over three axes, accommodation 0.8, vb_ratio 0.05, specular share 0.5, F10.7 130, Kp 2, Ap 7, orbit step 10 s, zonal degree 6.
- Device defaults in `product.rs:Dev::load`: torque_noise 0.001, friction_comp 0.95, eta 0.8, k_speed 1, k_flow 2, flow_tau 0.3, boresight +Y, Sun axis -Z.
- Sizing defaults in `adcs-design/lib.rs:demand`: 10 deg/s, 30 deg, 60 s, 3 years, k_h 2, quarter orbit; survey seasons and F10.7 65 and 250; the 11 pump constants of `empump.rs`.
- Catalogue derive rule (`tools/catalogue.py`): 6000 rpm, rotor share 0.4, no-load 1.25, static 1.5.
- Floquet certificate (`tools/floquet.py`); design-loop rules (`tools/pipeline_design.py:node_converge`).
- The twin only: the Earth-sensor limb chain (`+comp/+earth_sensor`), the coil and ring drivers and the valve schedule (`+comp/+magnetorquer/drive.m`, `+fluid_loop/drive.m`, `+rcs/schedule.m`).

## 2 · The time engine

Kinds: *relation* (write as a method), *already a method* (call the generated one), *toolbox* (a kind of maths),
*core* (integrator, step order, recorder, metrics, campaigns), *rig* (code by the boundary), *decide* (the owner's word).

### adcs-sim-core

| item | code location | lines | node(s) | kind | gaps | step |
|---|---|---|---|---|---|---|
| Exponential density table (fast orbit) *design/groups/env.pc holds the same Vallado table (env::atmosphere, density_at); call the generated one times density_scale.* | adcs-sim-core/src/atmos.rs:density (1-24) | 24 | l3_dist_row_07, m3_3 | relation, already a method | — | S7.3 |
| Low-precision Sun (Montenbruck-Gill) *The flight software's sun_model (fsw/pseudocode/02) is a different series; keep both or choose one (env decides).* | adcs-sim-core/src/ephem.rs:sun, ecl2eq (10-22) | 13 | l3_dist_row_09 | relation | G13 | S7.4 |
| Low-precision Moon *No node: env adds one.* | adcs-sim-core/src/ephem.rs:moon (25-42) | 18 | none | relation | G13 | S7.4 |
| Solar pressure at a distance *m3_5 solar_pressure exists; check it gives 4.56e-6 (AU/d)^2.* | adcs-sim-core/src/ephem.rs:p_srp (45) | 1 | m3_5 | relation, already a method | — | S7.3 |
| Conical Earth shadow | adcs-sim-core/src/ephem.rs:shadow (48-64) | 17 | l3_dist_row_10, m2_7 | relation | G13 | S7.4 |
| IGRF-13 field: coefficients at a date, geodetic, NED synthesis, to ECI *fsw/pseudocode/02 has igrf_gh, geodetic, igrf_ned, field_eci; the core can call the generated fsw-rs alg frames instead of its own copy.* | adcs-sim-core/src/field.rs (1-115) | 115 | l3_dist_row_08, env_igrf13_coefficients, nav_time_frames | toolbox (IGRF reading), already pseudocode | — | S7.3 |
| Julian date, calendar, decimal year, GMST, precession, ECI to ECEF *gmst_rot, prec_rot, eci2ecef, decyear are also in fsw/pseudocode/02: use one.* | adcs-sim-core/src/time.rs (1-56) | 56 | nav_time_frames | toolbox (time and frames) | — | S7.3 |
| Vectors, matrices, quaternions, q-method; libm wrappers | adcs-sim-core/src/la.rs, pm.rs | 157 | none | toolbox | — | - |
| Counter-based random streams | adcs-sim-core/src/rng.rs | 39 | none | toolbox | G1 | S7.2 |
| Box facets from the class box and the centre-of-mass offset | adcs-sim-core/src/torques.rs:Facets::boxed (11-19) | 9 | l3_dist_row_02, s2_0, s2_1, s2_2 | relation | — | S7.4 |
| Gravity-gradient, aerodynamic per facet, radiation per facet, residual dipole torques *gd_0..gd_3 are worst-case analysis methods; the per-facet in-loop torques are new methods.* | adcs-sim-core/src/torques.rs:torques, radiation (45-89) | 43 | l3_dist_row_01, l3_dist_row_02, l3_dist_row_03, l3_dist_row_06, gd_0, gd_1, gd_2, gd_3 | relation | — | S7.4 |
| Earth albedo and infrared pressure *Finding for env: the comment cites albedo 0.31 and 235 W/m^2; the code flies 0.30 and 237 W/m^2.* | adcs-sim-core/src/torques.rs:earth_pressure + EARTH_ALBEDO, EARTH_IR_W_M2 (22-41) | 20 | l3_dist_row_04, l3_dist_row_05 | relation + 2 parameters | — | S7.4 |
| Fast orbit forces: J2-J6 zonal, Sun and Moon point masses, co-rotating drag, SRP with shadow; node context *No node; env adds an orbit-forces node. The fast model is the engine.orbit=fast choice.* | adcs-sim-core/src/orbit.rs:accel, ctx + MU, RE, OMEGA_E, MU_SUN, MU_MOON, J (9-89) | 81 | none | relation + parameters | G13 | S7.6 |
| Orbit RK4 with Hermite interpolation; context interpolation | adcs-sim-core/src/orbit.rs:Orbit::new, advance, state, context (91-148) | 58 | none | core (integrator) | G8 | S7.6 |
| Classical elements to state | adcs-sim-core/src/orbit.rs:coe2rv (152-161) | 10 | none | toolbox (frames) | — | - |
| Rigid body with rotors, gimbals and one flexible mode; total momentum *dyn_kinematics is already a method; the core keeps RK4 and calls the generated deriv.* | adcs-sim-core/src/plant.rs:Geometry, Body::flexible, deriv, momentum (7-97, 139-146) | 98 | dyn_rigid_body, dyn_rotor_coupling, dyn_flexible_mode, dyn_total_momentum, dyn_kinematics | relation | G8, G12 | S7.5 |
| Plant RK4, sub-steps for a flexible mode *The rule "Omega h at most 0.5" is a numerical choice of the integrator: keep in core.* | adcs-sim-core/src/plant.rs:axpy, step, step_rk4 (99-137) | 39 | none | core (integrator) | G8 | S7.5 |
| Magnetorquers: dispersion, allocation pinv, saturation, L/R lag, power, failed coil | adcs-sim-core/src/actuators.rs:lag, Mtq (7-55) | 49 | l3_mtq_row_02, l3_mtq_row_03, l3_mtq_row_04, l3_mtq_row_05, l3_mtq_row_12, l3_mtq_row_13 | relation | G1, G2, G9 | S7.7 |
| Momentum devices: wheel motor line, friction (Coulomb, viscous, Stribeck, stiction), compensation, noise, power; VSCMG; fluid ring pump, flow filter, field hysteresis; CMG speed loop; gimbal limits | adcs-sim-core/src/actuators.rs:wheel_motor, Mex (57-165) | 109 | l3_rw_row_01, l3_rw_row_02, l3_rw_row_03, l3_rw_row_04, l3_rw_row_05, l3_rw_row_06, l3_rw_row_07, l3_fmr_row_08, l3_fmr_row_09, l3_fmr_row_10, l3_fmr_row_11, l3_fmr_row_12, l3_fmr_row_13, act_cmg_model, act_vscmg_model, act_vscmg_gimbal_limits | relation | G1, G2, G3, G5, G6, G12 | S7.7 |
| Thrusters: dispersion, minimum impulse, valve resolution, mass flow, valve power, failure | adcs-sim-core/src/actuators.rs:Rcs (167-199) | 33 | l3_rcs_row_01, l3_rcs_row_02, l3_rcs_row_03, l3_rcs_row_04, l3_rcs_row_08, l3_rcs_row_09, l3_rcs_row_10, l3_rcs_row_11 | relation | G1 | S7.7 |
| Gyro, magnetometer, fine Sun (noise model), coarse Sun, Earth sensor, GNSS (history and latency) | adcs-sim-core/src/sensors.rs:mis, sf, Gyro, Mag, Sun, Css, Es, Gps (10-122, 267-324) | 171 | l3_sens_row_01, l3_sens_row_02, l3_sens_row_03, l3_sens_row_04, l3_sens_row_05, l3_sens_row_06, l3_sens_row_13, l3_sens_row_14 | relation | G1, G3, G12 | S7.8 |
| Star tracker: synthetic catalogue, latency history, exclusion and blinding, noise model, onboard QUEST model *The catalogue generator has no node: sens adds one (a table or a method).* | adcs-sim-core/src/sensors.rs:star, St (124-265) | 142 | l3_sens_row_09, l3_sens_row_12 | relation | G1, G3, G4, G6, G7, G12 | S7.9 |
| Star-tracker chain: camera, pair table, render, centroid (median), identify (votes), attitude (q-method, residual) | adcs-sim-core/src/comp.rs:head_frame, star_tracker (20-300) | 281 | l3_sens_row_09, l3_sens_row_10, l3_sens_row_11, l3_sens_row_12 | relation | G1, G3, G4, G7, G12 | S7.10 |
| Quadrant Sun sensor: currents and angles | adcs-sim-core/src/comp.rs:sun_sensor (301-339) | 39 | l3_sens_row_07, l3_sens_row_08 | relation | G1 | S7.8 |
| Device byte codecs: registers, UART frames, CAN telemetry; PWM and CAN decode *Proposed: the scaling (LSBs, ranges) is design and already in the drivers' nodes; the framing stays rig code. The node becomes stated.* | adcs-sim-core/src/emu.rs (1-92) | 92 | l3_oils_row_07 | decide (rig code, or the inverse of drv nodes) | — | S7.12 |

### adcs-sim

| item | code location | lines | node(s) | kind | gaps | step |
|---|---|---|---|---|---|---|
| Plant and environment parameters from the case: epoch, mean motion, inertia, centre-of-mass direction [0.30, 0.70, -0.65], residual dipole split, flexible-mode delta, surface constants, engine defaults *The centre-of-mass direction, accommodation 0.8, vb_ratio 0.05, specular share 0.5, f107 130, kp 2, ap 7 are values with no node.* | adcs-sim/src/config.rs:build, apply_engine + ACCOMMODATION, VB_RATIO, SPEC_FRAC (186-189, 465-539, 626-696) | 150 | s1_0, s1_1, s1_2, s1_3, s1_4, s2_1, s2_3, s3_0, s4_0, s4_1, m2_5, m2_6, dyn_flexible_mode | relation + parameters | G5, G6 | S7.11 |
| Flight parameters from case, product and scenario (147 fsw_param_* values): gain laws, LQR weights, literature-law gains, gravity-gradient stability test, MEKF sigmas, rotor targets, choices *The 147 fsw_param_* nodes are stated with no value (S6 finding). Split by pattern match, to confirm per node.* | adcs-sim/src/config.rs:modes_and_laws, guidance_params, mtq_gains, rw_gains, spin_params, rotor_params, rcs_params, sensor_params (216-425) | 210 | fsw_param_* (147) | relation (47 computed), choices (9), parameters (38), wires (29), scenario values (24) | G6, G5 | S7.13 |
| Single-axis LQR by Kleinman iteration (Lyapunov solve) | adcs-sim/src/lqr.rs:solve, lyap, chain3 (5-57) | 53 | fsw_param_mtq_Klqr, fsw_param_rw_Klqr | toolbox (a CARE solver); the weights are relation | — | S7.13 |
| Part to device descriptor: motor stall torque and no-load speed, ring k_hv, spin-down time, h_max, pump torque 2h/tsd, thruster couples 2 F arm, device defaults (torque_noise 0.001, friction_comp 0.95, eta 0.8, k_speed 1, k_flow 2, flow_tau 0.3) *Ring momentum and spin-down time exist as methods (act ring_momentum, spin_down_time): call them. File reading and capacity checks stay code.* | adcs-sim/src/product.rs:add_rotor, fit_actuator_, fit_sensor, Dev::load (110-294) | 185 | l3_rw_row_01, l3_fmr_row_12, l3_rcs_row_01, gf_6, gf_7 | relation + parameters | G6, G10 | S7.11 |
| Truth set-up and environment refresh: RAAN from LTAN (fast), co-rotating air velocity, Earth half-angle, nadir | adcs-sim/src/run.rs:Truth::new, Truth::env (28-75); run (Sky, 506) | 50 | m2_3, l3_dist_row_08, l3_dist_row_10 | relation (glue) | G6 | S7.6 |
| Initial attitude, rate and rotor momenta from the scenario *No node; the scenario semantics belong to vv or case.* | adcs-sim/src/run.rs:initial_state (266-304) | 39 | none | relation | G1, G6 | S7.11 |
| Sensor sampling order; GNSS fix to ECEF; rotor telemetry noise 1e-7 N m s and 1e-5 rad *The tachometer and gimbal-angle noises have no node: act states them.* | adcs-sim/src/run.rs:sense (315-361) | 47 | l3_sens_row_14 | core (order) + relation (2 hidden noise values) | G1 | S7.8 |
| Actuation, propellant use and empty tank; coil torque m x B; latency hold (soft OILS) | adcs-sim/src/run.rs:actuate, step_plant (389-441) | 66 | l3_rcs_row_08, gd_3 | core (order) + relation (m x B, empty tank) | — | S7.7 |
| Soft-OILS latency model (bus times, CPI) | adcs-sim/src/run.rs:oils_latency (365-381) | 17 | gx_3, v3_2 | rig (code) | — | - |
| Tick order, recorder | adcs-sim/src/run.rs:run (443-559), row | 130 | none | core | G8 | - |
| Array power and battery state of charge; solar constant 1361 W/m^2 *No node: a power node (programme or design) is needed.* | adcs-sim/src/metrics.rs:PowerSystem, SOLAR_CONSTANT, derive (power part) (20-45, 152-165) | 40 | none | relation + parameter | — | S7.14 |
| Rotor-imbalance jitter (frequency domain) | adcs-sim/src/metrics.rs:jitter, eig_min3 (60-98) | 39 | gp_4, l3_pnt_row_06 | relation | — | S7.14 |
| APE, AKE, RKS, ECSS indices, windows, verdicts | adcs-sim/src/metrics.rs:derive, ecss, time_to, window, evaluate | 230 | none | core (metrics) | — | - |

### adcs-design

| item | code location | lines | node(s) | kind | gaps | step |
|---|---|---|---|---|---|---|
| Demand survey: one orbit at four attitudes, four seasons, two solar activities; peak torque, cyclic and secular momentum, field, eclipse *Calls the engine (Config, Truth, torques) as a subroutine. The slew laws equal gw_3 and gw_4 (methods). Defaults 10 deg/s, 30 deg, 60 s, 3 years, k_h 2, quarter orbit are values with no node.* | adcs-design/src/lib.rs:survey, demand + SURVEY_EPOCH_DAYS, SURVEY_F107 (111-232) | 122 | l3_budget_row_01, gw_2, m2_7, gw_3, gw_4 | relation + parameters | G6, G8 | S7.15 |
| Magnetorquer sizing (dump, momentum, detumble dipoles; mass and power linear) *gm_1..gm_3 and magnetorquer_dipole_required already exist as methods.* | adcs-design/src/lib.rs:mtq (254-272) | 19 | design_sizing_mtq, l3_budget_row_02, gm_4, gm_5, gm_1, gm_2, gm_3 | relation | — | S7.15 |
| select_rotor: lightest catalogue model meeting the per-unit need *Finding for design: the twin sizes wheels and CMGs by law (+sizing/rw.m, cmg.m); the engine selects from the catalogue.* | adcs-design/src/lib.rs:rotor (280-325) | 46 | design_sizing_rw, design_sizing_cmg, design_sizing_vscmg, l3_budget_row_03, gw_5, gw_6 | relation over a lookup | G3, G7, G10 | S7.15 |
| Fluid rings and the spare ring: faces, section of a skewed plane, ring assembly | adcs-design/src/lib.rs:fmr, spare_axis, section, ring (329-406) | 78 | design_sizing_fmr, l3_budget_row_04, l3_fmr_row_14, l3_fmr_row_15 | relation | G3, G7 | S7.15 |
| Electromagnetic pump and loop co-design (grid search, Darcy, magnet, copper, iron) *Finding for design: the twin's +sizing/fmr.m is the older law without the pump design.* | adcs-design/src/empump.rs (1-130) | 130 | l3_fmr_row_07 | relation + parameters (11 constants) | G3, G7 | S7.15 |
| N2O cold-gas RCS sizing (thrust class, impulses, tank) | adcs-design/src/lib.rs:rcs (409-440) | 32 | design_sizing_rcs, l3_budget_row_05 | relation + parameters | — | S7.15 |
| Mass, power, volume budget per product; families, sensors, gyro grade, star-tracker heads *gb_2 (peak power) is not computed by the sizing: budget sums steady power only.* | adcs-design/src/lib.rs:budget, size_all (445-558) | 112 | l3_budget_row_06, l3_budget_row_07, l3_budget_row_08, gb_0, gb_1, gb_2, gb_3, design_sizing_sensors | relation | G3, G6, G10 | S7.15 |
| Knobs of the design loop | adcs-design/src/lib.rs:Knobs (25-108) | 84 | none | code (the loop's interface) | — | - |

### adcs-pop

The plan names DTM2020, IGRF and DE440 reading as toolbox. The same reading is applied to every published model whose
evaluation a standard fixes (IERS 2010, JB2008, EGM, FES2004): toolbox, its coefficients a lookup. A model whose
form or values are the spacecraft design's (drag panels and accommodation, SRP and ERP box-wing, which forces are
summed, the SSO set-up) is a relation.

| module | what | lines | kind | step |
|---|---|---|---|---|
| adcs-pop/src/time.rs | Time scales | 289 | toolbox | - |
| adcs-pop/src/frames/mod.rs | ECI and ECEF builds | 224 | toolbox | - |
| adcs-pop/src/frames/iau2006.rs | IAU 2006/2000A kernel | 262 | toolbox | - |
| adcs-pop/src/frames/tidal.rs | IERS tidal EOP | 319 | toolbox | - |
| adcs-pop/src/eop.rs | Earth orientation parameters | 482 | toolbox | - |
| adcs-pop/src/geodetic.rs | ECEF to geodetic | 32 | toolbox | - |
| adcs-pop/src/spk.rs | DAF/SPK reader | 249 | toolbox (DE440 reading) | - |
| adcs-pop/src/ephem.rs | DE440 API and bundle | 277 | toolbox (DE440 reading) | - |
| adcs-pop/src/atmos/dtm2020.rs | DTM2020 operational | 430 | toolbox (DTM2020) | - |
| adcs-pop/src/atmos/dtm2020_research.rs | DTM2020 research | 333 | toolbox (DTM2020) | - |
| adcs-pop/src/atmos/jb2008.rs | JB2008 | 645 | toolbox (a published density model, as DTM2020) | - |
| adcs-pop/src/atmos/octave.rs | Octave numerics | 182 | toolbox | - |
| adcs-pop/src/atmos/mod.rs | Density model switch | 237 | toolbox | - |
| adcs-pop/src/atmos/exponential.rs | Vallado exponential density | 85 | relation, already a method (env::atmosphere) | S7.6 |
| adcs-pop/src/spaceweather.rs | Space-weather files and indices | 820 | toolbox (data reading) | - |
| adcs-pop/src/integ.rs | Integrators | 385 | toolbox (integrators) | - |
| adcs-pop/src/gravity/field.rs | Gravity field loader | 257 | toolbox | - |
| adcs-pop/src/gravity/sphharm.rs | Spherical-harmonic evaluation | 191 | toolbox (a published field evaluator; the coefficients a lookup) | - |
| adcs-pop/src/gravity/potential.rs | Geopotential | 66 | toolbox | - |
| adcs-pop/src/gravity/zonal.rs | Two-body and J2..J6 | 78 | toolbox | - |
| adcs-pop/src/gravity/mod.rs | Gravity force | 152 | toolbox | - |
| adcs-pop/src/solidtides.rs | Solid-Earth tides (IERS 2010) | 213 | toolbox (a published standard) | - |
| adcs-pop/src/oceantides.rs | Ocean tides (FES2004) | 225 | toolbox (a published standard; the table a lookup) | - |
| adcs-pop/src/relativity.rs | Post-Newtonian terms (IERS 2010) | 158 | toolbox (a published standard) | - |
| adcs-pop/src/thirdbody.rs | Sun and Moon point masses | 169 | relation (decide: a standard law, but which bodies and how is the design's) | S7.6 |
| adcs-pop/src/thirdbody/secular.rs | Lidov-Kozai secular dynamics | 135 | relation (not in the loop) | S7.6 |
| adcs-pop/src/drag.rs | Drag: cannonball and panel models | 520 | relation (spacecraft force model) | S7.6 |
| adcs-pop/src/drag/gsi.rs | Gas-surface interaction coefficients | 115 | relation (decide: published GSI forms, design accommodation) | S7.6 |
| adcs-pop/src/drag/geom.rs | Drag geometry | 162 | relation | S7.6 |
| adcs-pop/src/srp.rs | SRP, eclipse, box-wing | 350 | relation (spacecraft force model) | S7.6 |
| adcs-pop/src/erp.rs | Earth radiation pressure | 277 | relation (spacecraft force model) | S7.6 |
| adcs-pop/src/accel.rs | Force selection and sum; sso_initial; the in-loop stepper | 308 | relation (forces chosen, cr = 1 + refl, SSO set-up) + core (stepper) | S7.6 |

### adcs-physics and the group crates

| item | location | lines | kind | step |
|---|---|---|---|---|
| The 39 physics relations (41 functions, tables) *No engine crate depends on it. adcs-design and the core re-write some of these laws by hand (slew momentum and torque, magnetorquer dipole, ring momentum, spin-down time, gravity-gradient worst case).* | engine/crates/adcs-physics/src (generated) | 1003 | generated | S7.16 |
| Every computing row's pseudocode, per group (83 functions) *Fold with adcs-physics into one generated crate; retire the per-group crates.* | engine/crates/adcs-groups, adcs-groups-wasm (generated) | 2747 | generated | S7.16 |

### Tools (Python)

| item | location | lines | node(s) | kind | step |
|---|---|---|---|---|---|
| Pointing budget terms gp_0, gp_1, gp_2 from flown runs and the product | tools/pointing_budget.py | 127 | gp_0, gp_1, gp_2, gp_5 | relation | S7.14 |
| Datasheet to model parameters (assumed 6000 rpm, rotor share 0.4, no-load 1.25, static 1.5) *The derived block is held in the design as data today; the rule that made it is code.* | tools/catalogue.py:derive, wheel_motor | 123 | catalogue lookup blocks | relation + parameters | S7.14 |
| Floquet certificate of the coils-only nadir loop *No node: ctl adds a certify node, or it stays an analysis tool.* | tools/floquet.py | 201 | none | relation (analysis) | S7.14 |
| Design-loop convergence rules (resize, upgrade, knob moves) *Rules of the design loop: design or code is the owner's word; not needed for built-in zero.* | tools/pipeline_design.py:node_converge (131-292) | 162 | none | decide (design-loop rules) | later |
| Closure answers (requirement against achieved) | tools/evaluate.py:evaluate | 353 | 38 kpi_* closures | library check | S7.1b |

## 3 · The MATLAB twin (`matlab_sils/+asils`)

| package | files | lines | kind | step | note |
|---|---|---|---|---|---|
| +physics | 44 | 634 | generated (spec/physics) | - | — |
| +groups | 89 | 1868 | generated (group methods) | - | — |
| +pc | 15 | 95 | generated runtime of the MATLAB translator (toolbox) | - | — |
| +pcselftest | 12 | 299 | generated (translator self-test) | - | — |
| +devices | 16 | 436 | relation (device models) | S7.7-S7.9 | mtq, mex, rcs, wheel_motor; gyro, magnetometer, css, sun_sensor, star_tracker, earth_sensor, gps*, st_history, star_catalogue; init (dispersions) |
| +comp | 17 | 274 | relation (component chains) | S7.8-S7.10 | star_tracker, sun_sensor; earth_sensor (twin only); fluid_loop/drive, magnetorquer/drive, rcs/schedule; magnetometer/calibrate (a stub) |
| +env | 8 | 236 | relation (torques, shadow, Earth pressure, facets); toolbox (igrf_*) | S7.3-S7.4 | — |
| +plant | 4 | 101 | relation (deriv, geometry, axes); core (step) | S7.5 | — |
| +orbit | 6 | 130 | relation (init, node); core (advance, state, context) | S7.6 | The twin flies the POP (matlab_sils/pop), as the engine does with orbit=pop. |
| +sizing | 9 | 452 | relation (demand, mtq, rw, cmg, fmr, rcs, jitter, size_all); code (print) | S7.15 | Differs from adcs-design: wheels and CMGs by law, no pump design. |
| +product | 1 | 264 | relation (descriptors, budget_); code (reading) | S7.11 | — |
| +fsw | 28 | 1088 | flight algorithms written by hand (could be generated); step.m is the tick shell | S7.17 | A subset: no literature magnetic laws (Lovera, Celani, Avanzini, Tango, de Ruiter). |
| +metrics | 6 | 280 | core (metrics) | - | — |
| +hal | 10 | 158 | runtime and rig (lsb.m: the drivers' scaling, design) | S7.12 | — |
| +faults | 1 | 42 | core (fault injection) | - | — |
| +campaign | 6 | 213 | core (campaigns; the dispersions are design data) | - | — |
| +quat | 9 | 71 | toolbox | - | — |
| +util | 12 | 167 | toolbox and code | - | — |
| +case | 1 | 84 | code | - | — |
| +scenario | 2 | 121 | code | - | — |
| +rec | 1 | 70 | core (recorder) | - | — |
| +result | 1 | 16 | code | - | — |
| +solution | 7 | 401 | code | - | — |
| +trade | 5 | 217 | code | - | — |
| +viz | 3 | 130 | code | - | — |
| config.m | 1 | 194 | relation (gains, plant parameters: the twin of config.rs) | S7.11, S7.13 | — |
| run.m | 1 | 221 | core (tick order) | - | — |
| version.m | 1 | 5 | code | - | — |
| matlab_sils/pop | 256 | 19828 | toolbox (the POP the engine ports), with the force models as in adcs-pop | S7.6 | The reference the Rust port is held to. |

**Could the twin fly the generated flight software?** Yes. The flight algorithms are already methods (`fsw_estimation`,
`fsw_guidance`, `fsw_control`, `fsw_steplaws`, `fsw_allocation`, `fsw_modes`, `fsw_drivers`, `nav_time_frames`,
`fdir_*`). The MATLAB translator (`tndb translate matlab`, `+pc` runtime) already writes them; today only the
pieces the group rows call reach `+asils/+groups`. What it takes:

- Generate all of `fsw/pseudocode/01`-`09` (78 functions with 01 and 02, as S6 counted) into one MATLAB package, held to the interpreter's vectors as C and Rust are (`fsw/tests/pcode_vectors.txt`).
- Keep a MATLAB tick shell as runtime, the twin of `fsw/src/adcs_fsw.c` (order of a tick, state records, the parameter set). It replaces `+fsw/step.m` (385 lines), whose laws are written by hand today.
- Feed it the same parameters as the blob (the 147 `fsw_param_*`, S7.13), not `config.m`'s own copy.
- Drop the hand-written laws in `+fsw` (28 files, 1,088 lines). They are a subset: `+fsw` has no code for the magnetic laws of Lovera, Celani, Avanzini, Tango or de Ruiter.
- Speed in Octave must be measured (gap G11): the generated code is scalar loops.

## 4 · What the translators cannot express yet

The translators handle `fn`, `proc` with state, tables (step, linear), records, fixed arrays of arrays, `settle` loops,
and the Rust translator can route maths to a module. Not yet:

- **G1.** Random streams. The models draw from counter-based SplitMix64 streams (Box-Muller, a spare kept). Pseudocode has no 64-bit wrapping integers (band/shl stop at 2^53), so a draw cannot be written. Needs a toolbox stream type and draws (normal, uniform, normal3) in the interpreter and the three translators; MATLAB must do the uint64 arithmetic in two 32-bit halves.
- **G2.** A proc calling a proc. Device models keep state and call helpers that keep state. Workaround today: state as a record passed in and returned (as the flight software does).
- **G3.** Arrays of records. Per-rotor state, the star catalogue (direction, magnitude), the pair table, history buffers of (t, q). Workaround: parallel fixed arrays with a count.
- **G4.** Large arrays by reference. The star-tracker frame (detector_px^2 doubles), its scratch copy, the pair table, the 4000-star catalogue. The translators pass arrays by value (part of the +9 % average, up to +26 %, S6 accepted on small arrays); here it would be prohibitive. Needs inout parameters or a workspace record handed by reference.
- **G5.** Non-finite values. Models use INFINITY (no spin-down), NEG_INFINITY (never blind) and NaN (not stated). The translators refuse a non-finite literal. Needs inf and nan constants and isfinite / isnan.
- **G6.** Choices. Kind (rw, fmr, cmg, vscmg), the star-tracker model (noise, quest, image), the Sun-sensor level (model, chain), law names, guidance kinds, initial-attitude kinds, slot names. Needs a choice type declared in the design and written as int constants.
- **G7.** Sorting and early exit. The centroid median, the brightest-12 selection, the candidate order in select_rotor, the polygon order in section(). No sort, break, continue or return; settle and flags work but are long. A toolbox sort (indices of a fixed array) is the clean way.
- **G8.** The core and generated code meet. The integrator (core) calls a generated deriv; the step order calls generated device steps. Needs one stable generated signature per model and records the core can use without defining them again.
- **G9.** Generic units. Helpers such as lag(x, u, tau, dt) serve dipoles, flows and momenta; a function is written for one dimension today.
- **G10.** Reading files. Product and part descriptors, the catalogue scan in select_rotor. Reading stays toolbox; the relation takes the arrays.
- **G11.** Speed. The engine runs long scenarios and 4,516-run campaigns; Octave runs the twin. Generated scalar loops in Octave and by-value arrays in Rust and C may cost much. Measure each step; the target is no worse than S6 accepted.
- **G12.** Named capacities. NR = 8, NG = 4, NC = 6, NH = 2, NS = 8, HIST = 64, GPS_HIST = 256, N_STARS = 4000. An array length is a literal in pseudocode; a named constant length is needed so capacities are stated once.
- **G13.** Deterministic maths. The engine calls the pure-Rust libm (adcs-sim-core/pm.rs) so a trajectory is the same on every target. The Rust translator already takes a maths module; generated engine code must use it, or trajectories move in the last bits.

`docs/PSEUDOCODE_V2.md` ("What it does not do yet") already names G2, G3 and G9. Each new construct changes what a
relation may call: raise the toolbox (`trinetra-toolbox/1` to `/2`) in `adcs-sim/src/source.rs`, `tools/design_inputs.py`
and `python/trinetra_adcs/design.py` together.

## 5 · How the twin reads its inputs, and opening the .tndb

**Today.** `tools/from_design.py` writes `matlab_sils/data/**` from the design's `engine_input` table (163 files, 280 kB)
and `matlab_sils/cases/<id>.csv` from `design_case`; `--check` fails when a file differs. The twin reads them with
`asils.util.readjson` (`jsondecode`), `asils.case.read`, `asils.scenario.load` and `asils.product.load`. The engine reads
the same files, or the .tndb directly (`adcs-sim/src/source.rs`).

**Opening the .tndb from MATLAB or Octave, no toolboxes:**

| way | what it takes | verdict |
|---|---|---|
| the tndb binary over `system()` + `jsondecode` | Add `tndb read FILE [--engine-inputs\|--cases\|--node ID]` printing JSON (an array of {path, body}: jsondecode turns keys with / into mangled field names). The binary already carries SQLite (rusqlite bundled) and ships with the application. Works the same in MATLAB and Octave 7+; one process per run, cached in a containers.Map. Measured here: Octave 8.4 decodes all 163 engine inputs (280 kB) in 0.02 s. | **recommended** |
| Java JDBC | Needs a third-party SQLite JDBC driver jar with native libraries on javaclasspath; Octave may run without a JVM. Fragile, not recommended. | no |
| MEX | Needs a C compiler on every user machine and a build per platform and per MATLAB/Octave ABI. Not recommended. | no |

**Recommended:** add `tndb read DESIGN --engine-inputs | --cases | --node ID` to `engine/crates/trinetra-design/src/bin/tndb.rs`
(it has no read command today), and `asils.util.design_read(path)` in the twin that runs it once per run and caches the
answer. `readjson` and `case.read` take the text from it instead of a file. Same bytes as the files, so the twin's runs
do not change (S7.18).

## 6 · Found on the way (for the owning groups)

- **env:** `torques.rs` cites albedo 0.31 and 235 W/m^2 (Kiehl and Trenberth) and flies 0.30 and 237 W/m^2.
- **design:** the twin sizes wheels and CMGs by a law (`+sizing/rw.m`, `cmg.m`); the engine selects them from the catalogue (`adcs-design/lib.rs:rotor`). The twin's fluid ring has no pump design (`empump.rs`). One method each will force a choice.
- **design:** `gb_2` (ADCS peak power) is not computed by the sizing: `budget` sums steady power only.
- **all:** `adcs-physics` is generated and unused by the engine, while `adcs-design` and the core write some of the same laws by hand (slew momentum and torque = `gw_3`, `gw_4`; ring momentum and spin-down = `gf_6`, `gf_7`; magnetorquer dipole = `gm_1`, `gm_2`; density table = `m3_3`).
- **fsw:** the 147 `fsw_param_*` nodes hold the layout, not the values (S6's finding); the values are config.rs relations.
- **the conversion:** `behaviour()` makes any node with a `code.*` field built-in. 180 of the 271 are not relations in code; the count S7 drives to zero is really 91 rows plus the 147 parameters. (S7.1b moved the 180 by a revision on top of the conversion; `behaviour()` is 1.0.0's rule, kept.)

## 7 · Proposed order of work

Small steps, each closed by its parity check. Each step that writes methods marks them as transcriptions (to be
signed by the node engineer after the switch-over) and holds them equal to the code they replace on their cases and
across their range. The translator comes first; generated files are never edited.

| step | work | parity check |
|---|---|---|
| S7.1 | This inventory. | none (read-only) |
| S7.1b | **Done (7 Oct 2026).** Reclassify the 180 built-in nodes that are not relations in code: 94 achieved holders become evidence wires; 38 KPI closures get a closure behaviour; 5 fsw runtime rows become stated descriptions; 43 system leafs with only an owner pointer become open (or evidence, where a run or rig measures them). The rule is tools/convert_2_0.py behaviour() (a code.* field makes built-in today). Re-convert, rebuild the regression copy. | from_design.py --check: no generated file changes; parity_2_0 unchanged; the built-in count falls from 271 to 91 |
| S7.2 | Translator and toolbox groundwork, one construct at a time: G1 random streams (toolbox), G5 inf and nan, G6 choices, G4 arrays by reference, G12 named capacities, G7 a toolbox sort. In the JS and Rust interpreters and the Rust, C and MATLAB translators. Raise the toolbox to trinetra-toolbox/2 in adcs-sim/src/source.rs, tools/design_inputs.py and python/trinetra_adcs/design.py together. | pcode.py gen --check; cargo test -p trinetra-pcode -p pcode-selftest; the twin's selftest vectors in Octave; a stream drawn in Rust, C and MATLAB equals rng.rs bit for bit |
| S7.3 | Use what is already generated: the core's density table calls env::atmosphere and density_at; field.rs and time.rs call the generated frames and IGRF functions of fsw/pseudocode/02 (one copy); p_srp calls m3_5. Delete the hand copies. | every engine scenario: metrics identical (bit for bit where no transcendental changed, else within the twin ledger's bounds); engine.py fsw-parity |
| S7.4 | Environment and disturbances (env): facets, gravity gradient, aero and radiation per facet, Earth albedo and IR, residual dipole, Sun (fast), Moon, shadow; their parameters as stated nodes. Transcribe from torques.rs and ephem.rs. | vectors: interpreter = Rust = C = MATLAB; every scenario's metrics identical; twin-parity no worse |
| S7.5 | The plant (dyn): deriv with rotors, gimbals and the flexible mode, momentum. The core RK4 calls the generated deriv (G8). | vectors; momentum conservation test (adcs-sim-core tests); scenarios identical; twin-parity |
| S7.6 | The orbit: fast forces and context (orbit.rs accel, ctx); RAAN from LTAN; then the POP spacecraft force models (drag, SRP, ERP, third body, force sum, sso_initial) as relations over the POP toolbox. | adcs-pop tests against the Octave reference vectors; ENGINE_PARITY truth-environment table unchanged; scenarios identical |
| S7.7 | Actuators (act): magnetorquers, wheels, VSCMG, CMG, fluid rings, thrusters, gimbals; m x B and the empty tank. Needs G1, G2/G3, G5, G6. | vectors; scenarios and every campaign identical (same streams); twin-parity |
| S7.8 | Simple sensors (sens): gyro, magnetometer, fine Sun (noise and chain), coarse Sun, Earth sensor, GNSS; the telemetry noises as stated nodes. | vectors; scenarios and campaigns identical |
| S7.9 | The star tracker unit model: catalogue (a table or a method), latency history, exclusion, noise and QUEST models. | vectors; scenarios with each model identical |
| S7.10 | The star-tracker chain: render, centroid, identify, attitude (G4, G7). | vectors; image-model scenarios identical; speed within the S6 bound |
| S7.11 | Descriptors and plant parameters: product.rs derivations and device defaults, config.rs plant values (centre-of-mass direction, residual dipole split, flexible delta, surface constants, initial state). | every Config of every scenario identical (dump and compare); scenarios identical |
| S7.12 | The device codecs: decide (proposed: scaling from the drv nodes, framing stays rig code). The node l3_oils_row_07 becomes stated. | soft OILS: same bytes on every tick (record and compare); engine_oils results unchanged |
| S7.13 | Flight parameters: the 147 fsw_param_* values from the design (47 computed by methods, 9 choices, 38 stated, 29 wires, 24 scenario values); lqr chain3 as a toolbox CARE. Closes S6's finding. | every scenario's configuration blob byte for byte (flight_build.py gen --check, fswcfg); fsw-parity; scenarios identical |
| S7.14 | Run-side relations: power system, jitter, pointing terms gp_0..gp_2, the catalogue derive rule; floquet as an analysis method or kept as a tool (decide). | results/POINTING_BUDGET.md, EVALUATION.md, ENGINE_SOLUTIONS identical |
| S7.15 | Sizing (design): demand survey, magnetorquer, select_rotor, fluid rings with the pump design, RCS, budgets, sensors. One method each; the twin's +sizing generated from the same methods (the engine's laws win where they differ, after the owner's word). | sizing.json and the parts and products of both cases byte for byte; the pipeline's selection unchanged |
| S7.16 | Fold adcs-physics, adcs-groups and adcs-groups-wasm into one generated crate and one MATLAB package; the hand copies of their laws (adcs-design slew, dipole, ring) call it. | cargo test -p adcs-physics -p adcs-groups (as one); groupcode.py gen --check; app routes |
| S7.17 | The twin flies the generated MATLAB of everything above and of fsw/pseudocode 01-09 (the flight algorithms), behind a hand-written MATLAB tick shell (runtime); delete the hand-written relations in +devices, +comp, +env, +plant, +orbit, +sizing, +product, config.m and the algorithms in +fsw. | twin-parity: with the same streams (G1) the twin can match the engine value for value; t_* Octave tests |
| S7.18 | The twin reads the .tndb itself: tndb read (JSON on stdout), asils.util.design_read over system() + jsondecode; matlab_sils/data stops being a copy. | twin runs identical with inputs from the .tndb and from the files |
| S7.19 | The boundary check in tools/check_all.py (a relation outside generated files and the toolbox fails), built-in count zero, the S4 parity gate on the whole chain. | check_all; parity_2_0; built-in count 0 |

Every step also runs `python3 tools/pcode.py gen --check`, `python3 tools/groupcode.py gen --check`,
`cargo test -p adcs-physics -p pcode-selftest -p adcs-groups`, `engine.py twin-parity`, `engine.py fsw-parity` and
`python3 tools/check_all.py`.

