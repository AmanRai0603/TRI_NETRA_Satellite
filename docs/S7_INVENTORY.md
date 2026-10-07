# S7 inventory: every relation still in code

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.

S7 step 1 (`docs/PLAN_2_0.md`, S7). It lists every relation that still lives only in code, which S7 writes as a
method in the design and then generates. Read from the code at `cac5aac` and from the regression copy
(`tests/regression/design.tndb`). Nothing was changed but this page and `results/s7_inventory.json` (the same
content, for tools). Line counts are of the code as it stands.

## Counts

**Now, after S7.3e (7 Oct 2026): 89 built-in nodes**, the relations computed in code (act 38, design 19, dyn 4,
env 9, oils 1, pnt 4, sens 14). Behaviours of the 1,183 nodes: open 304, stated 298, children 134, lookup 126,
evidence 119, built-in 89, method 75, closure 38. (After S7.1b: 91 built-in of 1,155; S7.2b added 3 data nodes, S7.3
8 method nodes, turned l3_dist_row_07 and l3_dist_row_08 into methods and nav_time_frames into a stated row whose
method moved to env; S7.3b-e added 6 data nodes and 11 method nodes of env, the published models of the precision
orbit, which the inventory had called toolbox, so the built-in count did not move.) `python3 tools/health.py tests/regression --built-in` counts them;
`tests/test_built_in.py` holds the list, and each S7 step takes its nodes out of it. The counts below are the
inventory's, at `cac5aac`.

**Amended to the owner's ruling of 7 Oct 2026** (`docs/PLAN_2_0.md`, the boundary and S7): every published model, with
its data, is design, and so are the metrics; code keeps only the maths toolbox, the readers of files' formats, the step
order and the recorder. What the inventory called toolbox because a standard fixes it is now a **relation (published
model)**, to move into `env`, `orbit` or `kpi` with its data as the node's table:

- **23 published-model items, 5,299 lines** (`results/s7_inventory.json`, `counts.after_ruling_7_oct`): 19 adcs-pop
  modules (4,870 lines: time scales, frames and the IAU 2006/2000A kernel, tidal EOP, EOP, geodetic, DE440 evaluation,
  DTM2020 operational and research, JB2008, the density switch, the gravity field, spherical harmonics, potential,
  zonal, gravity force, solid and ocean tides, relativity); 3 in adcs-sim-core (`field.rs` IGRF, `time.rs`, `coe2rv`,
  181 lines); `fsw/pseudocode/02_time_frames_models.pc` and `02_igrf13.pc` (248 lines, already pseudocode, moving into
  `env`). Space weather (820 lines) splits into data (the indices as tables), code (the file parsers) and a relation
  (which index each model takes). In the twin: `matlab_sils/pop`'s published models, `+env/igrf_*`, `+util` jd, jd2utc,
  decyear, addsec.
- **The metrics** (`adcs-sim/src/metrics.rs` derive, ecss, time_to, window, evaluate, 230 lines; the twin's `+metrics`)
  are relations: methods of `kpi` (step S7.14b).
- **Toolbox, unchanged: 5 items, 816 lines**: `la.rs` and `pm.rs` (vectors, matrices, quaternions, libm), `rng.rs`
  (random streams), the LQR's Kleinman/Lyapunov solve (a Riccati solver; its weights are relations), adcs-pop's
  integrators and its Octave numerics (`datenum` moves with the time scales). **Code (file readers): 1 module** (`spk.rs`,
  the DAF/SPK reader, which loads the DE440 slice into the design), and the reader halves of `eop.rs`, `spaceweather.rs`,
  `gravity/field.rs` (ICGEM .gfc) and `oceantides.rs` (FES .bin).
- **Translator gaps: 15** (G14 data tables from the design, G15 long series and run-time data, section 4). **Steps: 26**
  (S7.2b published data in the design; S7.3 time, frames and IGRF; S7.3b atmosphere; S7.3c ephemeris; S7.3d gravity and
  tides; S7.3e relativity; S7.14b metrics; section 7).

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
- **adcs-pop** (before the ruling of 7 Oct, above): 23 modules toolbox (6,506 lines: time, frames, EOP, DE440 reading, DTM2020, JB2008,
  space weather, integrators, gravity field, tides, relativity); 9 modules relation (2,121 lines: the spacecraft force
  models and the force sum).
- **adcs-physics:** all generated (39 relations, 41 functions, 1,003 lines); nothing in the engine calls it.
- **Tools (Python):** 4 relations (613 lines): pointing terms, catalogue derive rule, Floquet
  certificate, design-loop rules.
- **MATLAB twin:** 160 generated files (2,896 lines: +physics, +groups, +pc, +pcselftest);
  90 hand-written files in 9 packages that hold relations (3,175 lines), the flight algorithms
  of +fsw among them; the POP (256 files, 19,828 lines) is toolbox.
- **Translator gaps:** 13 (section 4), 15 with the ruling's G14 and G15. G1, G3, G4, G5 and G6 block the device models; G14 blocks every published model.

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
| gm_4 | act | Magnetorquer power | adcs-design/src/lib.rs:mtq (power_at_max_W = 0.3 m/0.45) | +sizing | S7.14b | The metrics (the ruling of 7 Oct): how each KPI is measured from a run (APE, AKE, RKS, the ECSS indices, windows, time-to, verdicts; adcs-sim/src/metrics.rs derive, ecss, time_to, window, evaluate; the twin's +metrics) as methods of kpi, generated into the engine and the twin. The recorder and the step order stay core. | every scenario's metrics and every campaign's results identical (results/EVALUATION.md, ENGINE_SOLUTIONS); twin-parity |
| S7.15 |
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
| IGRF-13 field: coefficients at a date, geodetic, NED synthesis, to ECI *fsw/pseudocode/02 has igrf_gh, geodetic, igrf_ned, field_eci; the core can call the generated fsw-rs alg frames instead of its own copy.* | adcs-sim-core/src/field.rs (1-115) | 115 | l3_dist_row_08, env_igrf13_coefficients, nav_time_frames | relation (published model: IGRF-13), already pseudocode *(was toolbox; ruling of 7 Oct)* | G14 | S7.3 |
| Julian date, calendar, decimal year, GMST, precession, ECI to ECEF *gmst_rot, prec_rot, eci2ecef, decyear are also in fsw/pseudocode/02: use one.* | adcs-sim-core/src/time.rs (1-56) | 56 | nav_time_frames | relation (published model: time scales and frames) *(was toolbox)* | — | S7.3 |
| Vectors, matrices, quaternions, q-method; libm wrappers | adcs-sim-core/src/la.rs, pm.rs | 157 | none | toolbox (maths: stays) | — | - |
| Counter-based random streams | adcs-sim-core/src/rng.rs | 39 | none | toolbox (random streams: stays) | G1 | S7.2 |
| Box facets from the class box and the centre-of-mass offset | adcs-sim-core/src/torques.rs:Facets::boxed (11-19) | 9 | l3_dist_row_02, s2_0, s2_1, s2_2 | relation | — | S7.4 |
| Gravity-gradient, aerodynamic per facet, radiation per facet, residual dipole torques *gd_0..gd_3 are worst-case analysis methods; the per-facet in-loop torques are new methods.* | adcs-sim-core/src/torques.rs:torques, radiation (45-89) | 43 | l3_dist_row_01, l3_dist_row_02, l3_dist_row_03, l3_dist_row_06, gd_0, gd_1, gd_2, gd_3 | relation | — | S7.4 |
| Earth albedo and infrared pressure *Finding for env: the comment cites albedo 0.31 and 235 W/m^2; the code flies 0.30 and 237 W/m^2.* | adcs-sim-core/src/torques.rs:earth_pressure + EARTH_ALBEDO, EARTH_IR_W_M2 (22-41) | 20 | l3_dist_row_04, l3_dist_row_05 | relation + 2 parameters | — | S7.4 |
| Fast orbit forces: J2-J6 zonal, Sun and Moon point masses, co-rotating drag, SRP with shadow; node context *No node; env adds an orbit-forces node. The fast model is the engine.orbit=fast choice.* | adcs-sim-core/src/orbit.rs:accel, ctx + MU, RE, OMEGA_E, MU_SUN, MU_MOON, J (9-89) | 81 | none | relation + parameters | G13 | S7.6 |
| Orbit RK4 with Hermite interpolation; context interpolation | adcs-sim-core/src/orbit.rs:Orbit::new, advance, state, context (91-148) | 58 | none | core (integrator) | G8 | S7.6 |
| Classical elements to state | adcs-sim-core/src/orbit.rs:coe2rv (152-161) | 10 | none | relation (published model: two-body elements to state) *(was toolbox)* | — | S7.6 |
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
| Single-axis LQR by Kleinman iteration (Lyapunov solve) | adcs-sim/src/lqr.rs:solve, lyap, chain3 (5-57) | 53 | fsw_param_mtq_Klqr, fsw_param_rw_Klqr | toolbox (a CARE solver: stays); the weights are relation | — | S7.13 |
| Part to device descriptor: motor stall torque and no-load speed, ring k_hv, spin-down time, h_max, pump torque 2h/tsd, thruster couples 2 F arm, device defaults (torque_noise 0.001, friction_comp 0.95, eta 0.8, k_speed 1, k_flow 2, flow_tau 0.3) *Ring momentum and spin-down time exist as methods (act ring_momentum, spin_down_time): call them. File reading and capacity checks stay code.* | adcs-sim/src/product.rs:add_rotor, fit_actuator_, fit_sensor, Dev::load (110-294) | 185 | l3_rw_row_01, l3_fmr_row_12, l3_rcs_row_01, gf_6, gf_7 | relation + parameters | G6, G10 | S7.11 |
| Truth set-up and environment refresh: RAAN from LTAN (fast), co-rotating air velocity, Earth half-angle, nadir | adcs-sim/src/run.rs:Truth::new, Truth::env (28-75); run (Sky, 506) | 50 | m2_3, l3_dist_row_08, l3_dist_row_10 | relation (glue) | G6 | S7.6 |
| Initial attitude, rate and rotor momenta from the scenario *No node; the scenario semantics belong to vv or case.* | adcs-sim/src/run.rs:initial_state (266-304) | 39 | none | relation | G1, G6 | S7.11 |
| Sensor sampling order; GNSS fix to ECEF; rotor telemetry noise 1e-7 N m s and 1e-5 rad *The tachometer and gimbal-angle noises have no node: act states them.* | adcs-sim/src/run.rs:sense (315-361) | 47 | l3_sens_row_14 | core (order) + relation (2 hidden noise values) | G1 | S7.8 |
| Actuation, propellant use and empty tank; coil torque m x B; latency hold (soft OILS) | adcs-sim/src/run.rs:actuate, step_plant (389-441) | 66 | l3_rcs_row_08, gd_3 | core (order) + relation (m x B, empty tank) | — | S7.7 |
| Soft-OILS latency model (bus times, CPI) | adcs-sim/src/run.rs:oils_latency (365-381) | 17 | gx_3, v3_2 | rig (code) | — | - |
| Tick order, recorder | adcs-sim/src/run.rs:run (443-559), row | 130 | none | core | G8 | - |
| Array power and battery state of charge; solar constant 1361 W/m^2 *No node: a power node (programme or design) is needed.* | adcs-sim/src/metrics.rs:PowerSystem, SOLAR_CONSTANT, derive (power part) (20-45, 152-165) | 40 | none | relation + parameter | — | S7.14 |
| Rotor-imbalance jitter (frequency domain) | adcs-sim/src/metrics.rs:jitter, eig_min3 (60-98) | 39 | gp_4, l3_pnt_row_06 | relation | — | S7.14 |
| APE, AKE, RKS, ECSS indices, windows, verdicts | adcs-sim/src/metrics.rs:derive, ecss, time_to, window, evaluate | 230 | none | relation (the metrics: methods of kpi) *(was core; ruling of 7 Oct)* | — | S7.14b |

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

**Amended to the ruling of 7 Oct 2026.** The inventory first called every published model whose evaluation a standard
fixes (DTM2020, IGRF, DE440, IERS 2010, JB2008, EGM, FES2004) toolbox, its coefficients a lookup. The owner ruled
otherwise: every published model, with its data, is design. Each becomes a method of `env` or `orbit` over its
coefficients as the node's table (G14), citing its publication; code keeps the integrators, the Octave numerics and the
file readers. A model whose form or values are the spacecraft design's (drag panels and accommodation, SRP and ERP
box-wing, which forces are summed, the SSO set-up) was already a relation.

| module | what | lines | kind | step |
|---|---|---|---|---|
| adcs-pop/src/time.rs | Time scales (UTC, TAI, TT, TDB, UT1, GPS; leap seconds) | 289 | relation (published model); the leap seconds a table *(was toolbox)* | S7.3 |
| adcs-pop/src/frames/mod.rs | ECI and ECEF builds | 224 | relation (published model) *(was toolbox)* | S7.3 |
| adcs-pop/src/frames/iau2006.rs | IAU 2006/2000A kernel | 262 | relation (published model); the xys06 series a table *(was toolbox)* | S7.3 |
| adcs-pop/src/frames/tidal.rs | IERS tidal EOP | 319 | relation (published model); its 6 tables data *(was toolbox)* | S7.3 |
| adcs-pop/src/eop.rs | Earth orientation parameters | 482 | relation (splice, interpolation) + data (the EOP values) + code (the IERS file readers) *(was toolbox)* | S7.3 |
| adcs-pop/src/geodetic.rs | ECEF to geodetic | 32 | relation (published model: WGS-84) *(was toolbox)* | S7.3 |
| adcs-pop/src/spk.rs | DAF/SPK reader | 249 | code (file reader: loads the DE440 slice into the design) | S7.3c *(done: the reader stays; the evaluation is env_de440's)* |
| adcs-pop/src/ephem.rs | DE440 API and bundle | 277 | relation (published model: Chebyshev evaluation over the coefficient slice) *(was toolbox)* | S7.3c *(done: env_de440)* |
| adcs-pop/src/atmos/dtm2020.rs | DTM2020 operational | 430 | relation (published model); coefficients a table *(was toolbox)* | S7.3b *(done: env_dtm2020_operational)* |
| adcs-pop/src/atmos/dtm2020_research.rs | DTM2020 research | 333 | relation (published model); coefficients a table *(was toolbox)* | S7.3b *(done: env_dtm2020_research)* |
| adcs-pop/src/atmos/jb2008.rs | JB2008 | 645 | relation (published model); coefficients a table *(was toolbox)* | S7.3b *(done: env_jb2008)* |
| adcs-pop/src/atmos/octave.rs | Octave numerics | 182 | toolbox (pow, mod, rem, norm, interp1, erf: maths); `datenum` moves with the time scales | - |
| adcs-pop/src/atmos/mod.rs | Density model switch | 237 | relation (a choice, G6, and its adapters) *(was toolbox)* | S7.3b *(done: env_density_model)* |
| adcs-pop/src/atmos/exponential.rs | Vallado exponential density | 85 | relation, already a method (env::atmosphere) | S7.6 *(done in S7.3b: env_exponential_atmosphere)* |
| adcs-pop/src/spaceweather.rs | Space-weather files and indices | 820 | data (the indices as tables) + code (the file parsers) + relation (which index each model takes) *(was toolbox)* | S7.3b *(done: env_space_weather, env_jb2008_indices, env_kp_ap_table; the run-time files' parsers and samplers stay code)* |
| adcs-pop/src/integ.rs | Integrators | 385 | toolbox (integrators: stays) | - |
| adcs-pop/src/gravity/field.rs | Gravity field loader | 257 | relation (normalisation, zonals) + code (the ICGEM .gfc reader) *(was toolbox)* | S7.3d *(done: env_gravity_field, env_gravity_default_field; the .gfc reader stays)* |
| adcs-pop/src/gravity/sphharm.rs | Spherical-harmonic evaluation | 191 | relation (published model); the coefficients a table *(was toolbox)* | S7.3d *(done: env_gravity_field)* |
| adcs-pop/src/gravity/potential.rs | Geopotential | 66 | relation (published model) *(was toolbox)* | S7.3d *(done: env_gravity_field)* |
| adcs-pop/src/gravity/zonal.rs | Two-body and J2..J6 | 78 | relation (published model) *(was toolbox)* | S7.3d *(done: env_gravity_field)* |
| adcs-pop/src/gravity/mod.rs | Gravity force | 152 | relation (published model) *(was toolbox)* | S7.3d *(done: env_gravity_field's frames and norm; the model dispatch and workspaces stay code)* |
| adcs-pop/src/solidtides.rs | Solid-Earth tides (IERS 2010) | 213 | relation (published model) *(was toolbox)* | S7.3d *(done: env_solid_tides)* |
| adcs-pop/src/oceantides.rs | Ocean tides (FES2004) | 225 | relation (published model); the FES table data, its .bin reader code *(was toolbox)* | S7.3d *(done: env_ocean_tides, env_ocean_tide_tables)* |
| adcs-pop/src/relativity.rs | Post-Newtonian terms (IERS 2010) | 158 | relation (published model) *(was toolbox)* | S7.3e *(done: env_relativity)* |
| adcs-pop/src/thirdbody.rs | Sun and Moon point masses | 169 | relation (decide: a standard law, but which bodies and how is the design's) | S7.6 |
| adcs-pop/src/thirdbody/secular.rs | Lidov-Kozai secular dynamics | 135 | relation (not in the loop) | S7.6 |
| adcs-pop/src/drag.rs | Drag: cannonball and panel models | 520 | relation (spacecraft force model) | S7.6 |
| adcs-pop/src/drag/gsi.rs | Gas-surface interaction coefficients | 115 | relation (decide: published GSI forms, design accommodation) | S7.6 |
| adcs-pop/src/drag/geom.rs | Drag geometry | 162 | relation | S7.6 |
| adcs-pop/src/srp.rs | SRP, eclipse, box-wing | 350 | relation (spacecraft force model) | S7.6 |
| adcs-pop/src/erp.rs | Earth radiation pressure | 277 | relation (spacecraft force model) | S7.6 |
| adcs-pop/src/accel.rs | Force selection and sum; sso_initial; the in-loop stepper | 308 | relation (forces chosen, cr = 1 + refl, SSO set-up) + core (stepper) | S7.6 |

Also moving: **`fsw/pseudocode/02_time_frames_models.pc` and `02_igrf13.pc`** (248 lines: GMST, precession, ECI to
ECEF, decimal year, the Sun model, geodetic, the IGRF synthesis and its 26 x 195 table), already pseudocode, into `env`
(S7.3); the flight build then generates the flight's copy from `env`. Their `IGRF_GH` const is written whole at every use
by the MATLAB translator today (G14). **Moved (S7.2b, S7.3):** the IGRF table is env_igrf13_coefficients' data (`data`,
not `const`: one copy, indexed in place), the onboard models env_time_frames' method; `tools/from_design.py` writes both
files, and the engine's field takes `igrf_gh` and `igrf_ned` from the same module.

### The published models and their data

What each published model carries into the design (S7.2b), how big, what reads it in (code), and what the translators
need for it. Sizes are values (doubles, or ints where marked).

| model | data in the design | size | reader (code) | translator needs | step |
|---|---|---|---|---|---|
| Time scales | leap seconds (MJD, TAI-UTC, drift) | 28 x 3 | `Leap_Second.dat` / IANA list parser | G14; `table ... step` | S7.3 *(done, S7.2b: env_leap_seconds, 28 x 3 year, month, TAI-UTC, read by `readers.py matlab_matrix` from the Octave POP's leapTable.m)* |
| IGRF-13 | Gauss coefficients per 5-year epoch, secular variation | 26 x 195 (5,070) + 195 | `igrf13coeffs.txt` (gen_fsw_params.py) | G14 (today a `const`, inlined by MATLAB) | S7.3 *(done, S7.2b: env_igrf13_coefficients' data, 26 epochs + 26 x 195, read by `readers.py igrf13`)* |
| IAU 2006/2000A | X, Y polynomials; 3,082 X/Y terms over 1,309 argument rows of 14 integer multipliers; s series | 56,162 (449 kB `xys06.bin`) | the xys06 export | G14 with int data, G15 long series, G12 sizes | S7.3 *(done, S7.2b: env_xys06_series, 12 + 6 + 3,082 x 18 + 66 x 11 values, read by `readers.py xys06`)* |
| Tidal EOP (IERS) | ocean, libration and zonal tables | 6 tables, 1,378 | none (in tidal.rs today) | G14 | S7.3 *(done, S7.2b: env_tidal_eop_terms, 1,378 values, read by `readers.py matlab_matrix` from the Octave POP's four tidal_eop_models files)* |
| EOP | x, y, UT1-UTC, LOD, dX, dY per day (finals2000A, C04) | the case's span: about 730 x 6 for two years (files: 6,074 and 2,070 rows here; full C04 ~23,000) | finals/C04 parsers (eop.rs) | G14; 4-point Lagrange interpolation as a loop | S7.3 *(not loaded, S7.2b: the repository holds no published EOP file; the files under adcs-pop/tests/data/time_frames_eop are refgen's synthetic set, a test fixture; every run flies the GMST build with dUT1 = 0, so the runs' span (2027) needs none. To load one, a finals2000A or C04 file of the span must be added and a reader written (eop.rs's parsers are the model for it); the EOP splice and interpolation stay in eop.rs until then)* |
| Exponential atmosphere | Vallado's table | 28 x 3 | none | `table ... step` (already `env::atmosphere`) | S7.3b *(done: env_exponential_atmosphere reads env.atmosphere, m3_3's one table; adcs-pop's typed copy deleted)* |
| DTM2020 operational / research | 96 terms x 9 species each; ap/Kp table | 864 + 864 + 76 | coefficient files (refgen) | G14, G6 (model), G5 (fill values) | S7.3b *(done: env_dtm2020_coefficients, 2 x 96 x 9, read by `readers.py text_table` from refgen's exports; the ap-Kp tables env_kp_ap_table, 2 x 28 + 2 x 38, by `matlab_matrix` from the Octave POP's spaceweather.m and bint_oe.m)* |
| JB2008 | polynomial and harmonic coefficients | about 200 | none (in jb2008.rs) | G14 | S7.3b *(done: the model's own coefficients, about 70, are constants of its transcription, as in JB2008.m; its data are the SET indices, below)* |
| Space weather | F10.7, F10.7a, S10, M10, Y10, Kp, ap, Dst-derived dTc | the case's span: about 730 days x 11 (SOLFSMY) and x 26 (DTCFILE); files 10,733 and 10,729 rows | SOLFSMY, DTCFILE, OMNI2, GFZ Hpo, SWPC parsers | G14, G5 (missing values as nan) | S7.3b *(done: env_jb2008_indices holds the whole SET release 8_1_0, 10,729 days x 10 and x 26 (386,244 values, 2.5 MB of the regression copy), read by `readers.py solfsmy` and `dtcfile`, the parsers of get_jb2008_indices; the whole file, not a span, because the adcs-pop tests fly JB2008 from 1997 to 2026 and the runs take manual indices. The OMNI2, GFZ and SWPC files a run may read are not in the repository: their parsers stay code)* |
| DE440 | Chebyshev records of the Sun (11 coef, 16 d), EMB (13, 16 d), Moon (13, 4 d), Earth (13, 4 d) | about 9,220 a year of span (74 kB); the whole `de440s.bsp` is 32.7 MB for 300 years | `spk.rs` (DAF/SPK) | G14, G15 (Clenshaw recursion) | S7.3c *(done: env_de440_slice, the runs' span 1 Dec 2026 to 1 Feb 2028 TDB, 28 + 28 + 108 + 108 records, 11,020 values (240 kB), read by `readers.py de440_slice`, the parser of de440.open; the evaluation is the recurrence of de440.chebval, as the code, not Clenshaw's)* |
| Gravity (EGM) | GM, Re, normalised C, S | J2..J6 today (5); a full field 2(n+1)^2 (about 10,000 at degree 70) | ICGEM .gfc parser | G14 (2-D), G12, G4 (Legendre scratch) | S7.3d *(done: env_gravity_default_field, GM, Re, J2..J6, read by `matlab_matrix` from grav.defaultField.m; a .gfc field stays the reader's, handed to the generated evaluation, which holds degree 120 at most (G12) with its workspaces by reference (G4))* |
| Ocean tides (FES2004) | degree-10 coefficients; 8 main lines | about 5,300 (42 kB `.bin`) + 8 x 10 | the .bin reader | G14 | S7.3d *(done: env_ocean_tide_tables, FES2004 1,052 x 12 and the main lines 8 x 10 (12,704 values), read by `readers.py fes_bin` and `matlab_cell`)* |
| Solid tides, relativity | Love numbers, constants (IERS 2010) | tens | none | parameters as stated nodes | S7.3d, S7.3e *(done as constants of the methods' transcriptions (IERS 2010 table 6.3, eq. 10.12); stated nodes for them, which a method would read, are later work)* |

**Embedded, not loaded at run time.** Each table is generated into the code from the design (as the IGRF table is
today), the time series as the slice the case's runs span. The readers stay code and write into the design, never into
a model. Where a campaign must move the epoch past the slice, the case's slice grows; a model reading a file at run
time would bring the published data back into code.

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
| +env | 8 | 236 | relation (torques, shadow, Earth pressure, facets; igrf_* the IGRF synthesis, a published model *(was toolbox)*) | S7.3-S7.4 | — |
| +plant | 4 | 101 | relation (deriv, geometry, axes); core (step) | S7.5 | — |
| +orbit | 6 | 130 | relation (init, node); core (advance, state, context) | S7.6 | The twin flies the POP (matlab_sils/pop), as the engine does with orbit=pop. |
| +sizing | 9 | 452 | relation (demand, mtq, rw, cmg, fmr, rcs, jitter, size_all); code (print) | S7.15 | Differs from adcs-design: wheels and CMGs by law, no pump design. |
| +product | 1 | 264 | relation (descriptors, budget_); code (reading) | S7.11 | — |
| +fsw | 28 | 1088 | flight algorithms written by hand (could be generated); step.m is the tick shell | S7.17 | A subset: no literature magnetic laws (Lovera, Celani, Avanzini, Tango, de Ruiter). |
| +metrics | 6 | 280 | relation (the metrics: methods of kpi) *(was core; ruling of 7 Oct)*; print.m code | S7.14b | — |
| +hal | 10 | 158 | runtime and rig (lsb.m: the drivers' scaling, design) | S7.12 | — |
| +faults | 1 | 42 | core (fault injection) | - | — |
| +campaign | 6 | 213 | core (campaigns; the dispersions are design data) | - | — |
| +quat | 9 | 71 | toolbox | - | — |
| +util | 12 | 167 | toolbox and code; jd, jd2utc, decyear, addsec are time scales: relation (published model) | S7.3 | — |
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
| matlab_sils/pop | 256 | 19828 | relation (the published models: 03_frames_time, 04_atmosphere, 02_forces gravity, tides, relativity; and the force models), toolbox (01_core +integ, Octave numerics), code (05_data readers) *(was toolbox)* | S7.3-S7.6 | The reference the Rust port is held to. By folder (files/lines): 01_core 29/1,721; 02_forces 64/1,499; 03_frames_time 46/2,087; 04_atmosphere 23/3,403; 05_data 29/2,865; 06_validation 50/5,952; 07_examples, 08_test, 11_compare 14/2,243. |

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

**After S7.2 (7 Oct 2026):** G1 (random streams), G4 (inputs by reference, `inout`), G5 (inf and nan), G6 (choices),
G7 (the sort), G12 (named capacities) and G14 (data tables) are in the language, its interpreters and its three
translators (`docs/PSEUDOCODE_V2.md`); G2, G3, G8, G9, G10, G11, G13 and G15 remain, as written below.

**After S7.3 (7 Oct 2026):** `trunc`, `log2` and the platform's `hypot` (Rust `f64::hypot`, C `hypot`, MATLAB `hypot`;
the interpreters port V8's, held to Node's on 40,000 arguments) are in the language, toolbox `trinetra-toolbox/3`, because
the published models' code calls them (POP's geodetic is `x.hypot(y)`, Octave's datevec `log2`, SOFA's `trunc`). G13 is
met for the generated engine models: the Rust translator's maths module (`crate::pm`, the libm crate, for adcs-sim-core;
std for adcs-pop, as the code it replaced). G15 is met for the IAU 2006 series (a loop over a 3,082 x 18 data table).
G8 is met by thin calls: the core keeps its signatures and hands values to the generated functions. G11 (speed): the
generated time scales cost 1.5 us a step against 0.9 us, the CIO build 290 us against 76 us (the series' sine and cosine
taken per term, not per distinct argument); no run's wall time moved measurably (48 scenarios 25.6 s against 24.6 s).

**After S7.3b-e (7 Oct 2026):** no new construct; four translator fixes the published models showed. (1) `pow` in the Rust
translation for std (adcs-pop) hands `f64::powf` its exponent through `core::hint::black_box`: the optimiser rewrote
`powf(x, 2.0)` as `x*x`, which is not the library's pow (88 of 200,000 IAU 2006 polynomials differed in the last bit);
where the code it transcribes squared through `powf(x, 2.0)`, the transcription writes `x^2` (geodetic, GMST, the IAU
2006 polynomial, J2, the tides' degree-2 potential). (2) MATLAB cannot index a call's result (`f().x`): a field of one
is `getfield(f(), 'x')`. (3) The C and Rust vector dispatchers read and write an array of more than 1,024 numbers with a
loop (the gravity workspaces are 15,129 each); no committed dispatcher has one. (4) A function whose result amplifies the
maths library's last bit (the tides' central difference, about a million times) states `## tolerance: 1e-8` in its
documentation: tools/translators.py holds its translations to that, relative to its largest output. G4 (inout arrays)
carries the gravity workspaces by reference; G6 the density switch and the gravity model; G12 the gravity capacity.
G11 (speed), measured old against generated, per call: DTM2020 operational 3.99 against 4.81 us (+21 %), research 4.79
against 6.22 us (+30 %), JB2008 23.3 against 29.3 us (+26 %), the density switch on the in-loop path 3.23 against 3.76
us (+16 %), ephemInputs 0.67 against 0.77 us (+15 %), the 6 x 6 spherical harmonics within a timer's noise; the 48
scenarios 24.4 s before and 22.7 to 25.4 s after (the machine's noise): the orbit is a small part of a step. The cost is
the translation's by-value arrays and records (DTM2020's 97-term column and its Legendre record copied into each of nine
expansions); passing an input the callee does not change by reference (`&[f64; N]`) is the fix proposed (G11). The
parity gate on the regression copy stands as after S7.3 (inputs 166, layout 151, blobs 48, runs 48, campaigns 4,516,
soft OILS 2, the twin 1, all equal; evaluate 1,496 of 1,500, the four l3_dist_row_07/08 reason-text rows S7.3 named).
The regression copy is 9.6 MB (6.5 MB after S7.3): JB2008's SET indices 2.5 MB, DE440's slice 0.24 MB, FES2004 0.25 MB.
Found on the way: the standard Kp-ap table is still typed twice in code besides env's (tools/engine_campaigns.py
KP_NODES and AP_NODES, the twin's +campaign/draw.m), held equal by tests/test_kp2ap.py.

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

- **G14.** Data tables from the design (the ruling of 7 Oct). Every published model carries data: IGRF 5,070 values, the
  IAU 2006 series 56,162, DTM2020 864 a model, the DE440 slice about 9,200 a year of span. A `const` array works in Rust
  and C, but the MATLAB translator writes the whole literal at every use (fsw/pseudocode/02's `IGRF_GH`, 26 x 195, inside
  a loop), and a MATLAB literal cannot be indexed; nothing marks the values as a node's table. Needs a `data` item: a
  named, typed (real with a unit, or int), 1-D or 2-D array whose values the design writes row by row, emitted once
  (Rust static, C const, MATLAB a function holding a persistent copy) and indexed in place.
- **G15.** Long series and run-time data. The IAU 2006 series (3,082 terms over 1,309 integer argument rows), Chebyshev
  (Clenshaw) and Legendre recursions are loops over data: G14 with int data, G12 for their sizes, G4 for scratch by
  reference; no new construct. The time series (EOP, space weather, DE440) are embedded as the slice the case's runs
  span, generated from the design; reading a file at run time stays the readers' work, into the design. Octave's speed on
  the 3,082-term sum is to be measured (G11).

**What the published models need first** (Part A of the ruling): G14 data tables, before any of S7.3-S7.3e; then table
lookup with interpolation (leap seconds, the exponential table: `table ... step` exists; EOP's 4-point Lagrange and the
daily space-weather lookup are loops over G14 data); long polynomial and Fourier series (loops over int and real G14
data, sized by G12); choices (G6) for the density model and the EOP source; nan (G5) for a missing index. Data loaded
from files at run time is not needed: the design holds the slice, embedded at generation.

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
| S7.2 | **Done (7 Oct 2026):** data tables, inf and nan, choices, named capacities, sort, inputs by reference, random streams, in both interpreters and the three translators, each held by the language's self-test (23 functions, every translation equal to the interpreter) and the streams to rng.rs bit for bit; toolbox trinetra-toolbox/2 (`docs/PSEUDOCODE_V2.md`). Translator and toolbox groundwork, one construct at a time: first what the published models need, G14 data tables from the design (a `data` item, emitted once, indexed in place); then G1 random streams (toolbox), G5 inf and nan, G6 choices, G4 arrays by reference, G12 named capacities, G7 a toolbox sort. In the JS and Rust interpreters and the Rust, C and MATLAB translators. Raise the toolbox to trinetra-toolbox/2 in adcs-sim/src/source.rs, tools/design_inputs.py and python/trinetra_adcs/design.py together. | pcode.py gen --check; cargo test -p trinetra-pcode -p pcode-selftest; the twin's selftest vectors in Octave; a stream drawn in Rust, C and MATLAB equals rng.rs bit for bit; existing generated files byte for byte unless a construct is used |
| S7.2b | **Done (7 Oct 2026) for what S7.3 needs**: the leap seconds (env_leap_seconds), the IGRF-13 table (env_igrf13_coefficients: the flight software's and the engine's one copy, `fsw/pseudocode/02_igrf13.pc` written from it), the IAU 2006/2000A series (env_xys06_series) and the tidal EOP terms (env_tidal_eop_terms), each a `data` module of its node, read by `tools/readers.py` (igrf13, matlab_matrix, xys06) through the developer's revision S7.2b (`[[revision.data]]`, design/revisions_2_0.toml), citing its publication and its file with the file's sha256; `tests/test_published_data.py` holds every table equal, bit for bit, to its reader's reading and to what the code flew (62,778 values). Not loaded: EOP (no published file in the repository; the runs need none). The atmosphere, space weather, DE440, gravity and FES tables are S7.3b-d's. Published data in the design (the ruling of 7 Oct): each published model's coefficients and data as a table of an env or orbit node, citing its publication, written by the file readers (code) from the files the engine reads today. Sizes in section 2, *The published models and their data*: leap seconds 28 x 3; IGRF-13 26 x 195 + 195; IAU 2006/2000A xys06 56,162; tidal EOP 1,378; EOP the case's span (about 730 x 6); DTM2020 2 x 864 + 76; JB2008 about 200; exponential 28 x 3; space weather the case's span (about 730 x 11 and x 26); DE440 about 9,220 a year of span; gravity J2..J6 (a full field 2(n+1)^2); FES2004 about 5,300. | from_design --check: each table equals the file its reader loads, value for value (bits); the readers' tests; no run changes |
| S7.3 | **Done (7 Oct 2026) but EOP**: env's methods, the developer's unsigned transcriptions (design/revisions/S7.3/*.pc, `[[revision.method]]`), generated by `tools/engine_build.py` into adcs-sim-core/src/gen (truth time `caltime`, field `truthfield` over env_time_frames' `igrf_gh`/`igrf_ned`, fast density `truthdensity` over m3_3's `atmosphere` table, `elements` coe2rv), adcs-pop/src/gen (`timescales`, `geodesy`, `earthframes`, `iau2006`, `tidaleop`) and the twin's +asils/+models; time.rs, field.rs, atmos.rs, coe2rv, adcs-pop time.rs, geodetic.rs, frames (gmst build, Earth rate), iau2006.rs, tidal.rs are thin calls; fsw-rs/src/igrf13.rs deleted; fsw/pseudocode/02 moved into env (env_time_frames), nav_time_frames stated. Held: 200,000 random inputs a function, the old code against the generated, 0 bits different (sim-core and pop); adcs-pop's Octave tests 0e0 as before; 48 of 48 scenarios bit for bit; fsw-parity 48/48; built-in 91 to 89 (env 11 to 9). The twin: its jd, jd2utc and decyear call the generated ones (bit for bit in Octave); its IGRF synthesis stays its own until S7.17 (the generated one differs from it in the last bits: igrf_gh to 1.5e-12 relative, the field to 2.5e-15), so its stored runs stand. Not done: the EOP splice and interpolation (eop.rs: no EOP data in the design, above); the solar pressure: m3_5 is another model (F_sun/c at the Earth-Sun distance of a series: 0.46 % below the engine's 4.56e-6 (AU/d)^2 at the Sun's distance from the spacecraft), so p_srp stays for S7.4 with the Sun. Time and frames, and IGRF (published models, the ruling of 7 Oct): time scales with the leap-second table; Julian date, GMST, precession, ECI to ECEF (core time.rs and fsw/pseudocode/02, one method each in env); the IAU 2006/2000A CIO kernel over the xys06 tables; tidal EOP; EOP splice and interpolation over the EOP table; geodetic; the IGRF-13 synthesis over its table (fsw/pseudocode/02 and 02_igrf13 move into env, one copy; the flight build generates the flight's copy from env). Also: the core's density table calls env::atmosphere and density_at; p_srp calls m3_5. Delete the hand copies. | adcs-pop time and frames tests against the Octave reference vectors (bit for bit where they are today); every engine scenario: metrics identical (bit for bit where no transcendental changed, else within the twin ledger's bounds); engine.py fsw-parity 48/48; flight_build gen --check shows only the moved source |
| S7.3b | **Done (7 Oct 2026).** env's methods (design/revisions/S7.3b): env_dtm2020_operational, env_dtm2020_research, env_jb2008, env_exponential_atmosphere, env_density_model (the switch, a choice), env_space_weather (kp2ap, ap2kp, the manual branch, data.drivers), over env_dtm2020_coefficients, env_jb2008_indices and env_kp_ap_table; generated into adcs-pop/src/gen and the twin's +asils/+models; atmos/*.rs and spaceweather.rs's models are thin calls, their Rust parsers of the coefficient and SET files deleted (the readers are tools/readers.py). Held: 200,000 random inputs a model and 100,000 of the switch, the adapters and the manual space weather, old against generated, 0 bits different; the adcs-pop atmosphere tests 0e0 as before; 48 of 48 scenarios bit for bit. Atmosphere (published models): the exponential table (already env::atmosphere), DTM2020 operational and research, JB2008 over their coefficient tables; the density switch as a choice (G6); the space-weather indices as data tables, the index each model takes as a relation; the SOLFSMY, DTCFILE, OMNI2, GFZ Hpo and SWPC parsers stay code. | adcs-pop atmosphere tests against the Octave reference vectors; engine scenarios with orbit=pop identical |
| S7.3c | **Done (7 Oct 2026).** env_de440 (design/revisions/S7.3c) over env_de440_slice: the record choice, the Chebyshev recurrence, de440.sun, moon, earth and ephemInputs with de440.constants(); spk.rs keeps the DAF/SPK reader and hands its records to the generated evaluation (an epoch outside the slice, another body). Held: 200,000 epochs in the span (from the slice, and through an open kernel), 200,000 elsewhere and 200,000 de440.state calls of every segment, old against generated, 0 bits; the adcs-pop ephemeris tests as before; 48 of 48 scenarios bit for bit. Ephemeris (published model): DE440 Chebyshev evaluation (Clenshaw) of the Sun, EMB, Moon and Earth over the coefficient slice the case's runs span; the per-step bundle as a method. spk.rs (DAF/SPK) stays code and loads the slice into the design. | adcs-pop ephemeris tests (positions bit for bit against today's ephem.rs on the slice); engine scenarios identical |
| S7.3d | **Done (7 Oct 2026).** env_gravity_field, env_solid_tides, env_ocean_tides (design/revisions/S7.3d) over env_gravity_default_field and env_ocean_tide_tables; gravity/*.rs, solidtides.rs and oceantides.rs are thin calls and workspaces; the ICGEM .gfc reader stays code. Held: 100,000 random points over the default, a 70 x 70 and a 20 x 20 field (every degree and order), the potential, J2..J6, the force in each model, every tide function, old against generated, 0 bits; the adcs-pop gravity and tides tests 0e0 as before; 48 of 48 scenarios bit for bit. Gravity and tides (published models): normalisation and zonals of the field, spherical-harmonic evaluation, potential, the gravity force over the coefficient table; solid-Earth tides (IERS 2010); ocean tides (FES2004 table, the 8 main lines). The ICGEM .gfc and FES .bin readers stay code. | adcs-pop gravity and tides tests against the Octave reference vectors; ENGINE_PARITY truth-environment table unchanged |
| S7.3e | **Done (7 Oct 2026).** env_relativity (design/revisions/S7.3e): the three terms and relativity.total; relativity.rs is thin calls. Held: 200,000 random states, every term list, old against generated, 0 bits; the adcs-pop relativity tests as before; 48 of 48 scenarios bit for bit. Relativity (published model): the IERS 2010 post-Newtonian terms (Schwarzschild, Lense-Thirring, de Sitter) as methods of orbit. | adcs-pop relativity tests against the Octave reference vectors; scenarios identical |
| S7.4 | Environment and disturbances (env): facets, gravity gradient, aero and radiation per facet, Earth albedo and IR, residual dipole, Sun (fast), Moon, shadow; their parameters as stated nodes. Transcribe from torques.rs and ephem.rs. | vectors: interpreter = Rust = C = MATLAB; every scenario's metrics identical; twin-parity no worse |
| S7.5 | The plant (dyn): deriv with rotors, gimbals and the flexible mode, momentum. The core RK4 calls the generated deriv (G8). | vectors; momentum conservation test (adcs-sim-core tests); scenarios identical; twin-parity |
| S7.6 | The orbit: fast forces and context (orbit.rs accel, ctx); RAAN from LTAN; then the POP spacecraft force models (drag, SRP, ERP, third body, force sum, sso_initial) as relations over the published-model methods of S7.3-S7.3e (no POP toolbox remains); coe2rv as a method of orbit. | adcs-pop tests against the Octave reference vectors; ENGINE_PARITY truth-environment table unchanged; scenarios identical |
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

