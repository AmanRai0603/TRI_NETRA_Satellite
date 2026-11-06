# S11 inventory: every correction, where it lives, what fails before it

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.

S11 step 1 (`docs/PLAN_2_0.md`, S11 Corrections (U0)). It lists every item S11 corrects, read from the code at `e92df168` and from the regression copy (`tests/regression/design.tndb`). Nothing was changed but this page and `results/s11_inventory.json` (the same content, for tools). The item ids are those of `docs/TECHNICAL_ROADMAP.md`, `docs/TEST_BENCH_PLAN.md`, `docs/TEST_STRATEGY.md`, `docs/ORBIT_PROPAGATOR_AUDIT.md` and `docs/DESIGN_DECISIONS.md`. A claim says the file and line that shows it; a line number is of this commit.

## Answer first

- **52 things to do: 37 fixes and 15 decision stubs.** 51 are open today, 1 is partly fixed (E1: the twin was moved by S7, the parity tool is still blind to direction), 0 are fully fixed. S6-S7 fixed none of them outright; they moved four items nearer: E1 (twin frames), E6 (DTM2020 now a method, so only the rows must call it), G6 and the F parameters (the gains are design methods now, so G6 is a design change with no code), and the verdict code of A1 (kpi states the windows).
- **Where it lives:** both 11, code 17, design 24. Most flight fixes are both: the algorithm is a design node (`fsw_modes`, `fsw_estimation`, `fsw_allocation`, `fsw_control`, `fsw_steplaws`) but the tick that orders the calls (`fsw/src/adcs_fsw.c`, `fsw-rs/src/fsw.rs`, `matlab_sils/+asils/+fsw/step.m`) is runtime code and holds the checks F1-F5 and G1, G2, G4 need. Three copies of the tick must change together.
- **By kind and tier** (`.claude/model_policy.toml`): code/standard 36; decide/deep 15; transcribe/standard 1. The 15 decision stubs are deep; every fix is standard; the regeneration, parity and re-fly steps are light.
- **By size:** M 13, S 39 (decisions counted as S; the fixes are S and M, with TS4/F-runtime the largest).
- **Owner decisions needed: 9, all with a default** so work can start without an answer; nobody signs anything in S11 (section 5). The ones that change what is built: DD8 confidence level (A1), DD50 time source (F1), F10 remove or support the single-precision build, A5 how to disperse inertia, F5 detumble exit without a gyro.
- **Not in the items but met on the way:** `gyro_fail`, reset, clock-step and jitter faults do not exist in the engine's fault table (`run.rs:222-241`), so the plan's done-when ("the gyro, reset and timing faults have scenario cases") needs the rig of step S11.4 first. `results/engine_parity.json` now has 1 differing verdict of 192, not the roadmap's five untraced ones; A3's defect (the unconditional "all traced" text) is still there.
- **Proposed order:** 18 steps (section 6): decisions, the golden table, the runtime and its fault rig, the gyro chain, the estimator and gains, the device models, power and sizing, the verdicts, the density, the orbit step, soft OILS, the weak tests, then regenerate and re-fly.

## 1 · Summary table

| id | status | lives | kind | tier | size | step |
|---|---|---|---|---|---|---|
| A1 | open | both | code | standard | M | S11.10 |
| A2 | open | code | code | standard | M | S11.10 |
| A3 | open | code | code | standard | S | S11.10 |
| A5 | open | both | code | standard | S | S11.9 |
| A7 | open | both | code | standard | S | S11.10 |
| Y1 | open | both | code | standard | M | S11.8 |
| Y2 | open | design | code | standard | S | S11.8 |
| Y4 | open | design | code | standard | S | S11.8 |
| Y5 | open | design | code | standard | M | S11.8 |
| M1 | open | design | code | standard | S | S11.7 |
| M2 | open | design | code | standard | S | S11.7 |
| M5 | open | design | code | standard | S | S11.7 |
| M9 | open | design | code | standard | S | S11.7 |
| M10 | open | code | code | standard | S | S11.7 |
| F2/G3 | open | both | code | standard | M | S11.5 |
| G1 | open | both | code | standard | M | S11.6 |
| G2 | open | both | code | standard | S | S11.6 |
| G4 | open | both | code | standard | S | S11.5 |
| G6 | open | design | code | standard | S | S11.6 |
| F4 | open | both | code | standard | S | S11.6 |
| F5 | open | both | code | standard | S | S11.5 |
| F1 | open | both | code | standard | M | S11.4 |
| F3 | open | code | code | standard | M | S11.4 |
| F10/G8 | open | code | code | standard | S | S11.4 |
| E1 | partly fixed | code | code | standard | S | S11.11 |
| E6 | open | design | transcribe | standard | M | S11.11 |
| P5 | open | code | code | standard | S | S11.12 |
| O2 | open | code | code | standard | S | S11.13 |
| O12 | open | code | code | standard | S | S11.13 |
| O19 | open | code | code | standard | S | S11.13 |
| O20 | open | code | code | standard | S | S11.13 |
| O21 | open | code | code | standard | S | S11.13 |
| O11 (B0) | open | code | code | standard | M | S11.14 |
| TS11 | open | code | code | standard | M | S11.3 |
| TS4 | open | code | code | standard | M | S11.4 |
| TS3 | open | code | code | standard | M | S11.6 |
| TS17 | open | code | code | standard | S | S11.15 |
| the 15 decisions (DD8, DD2, DD39+15, DD40, DD50, DD12, DD59+60, DD9+61, DD14+19, DD46+47, DD22+23, DD33+35, DD57, DD52, DD1+38) | open | design | decide | deep | S each | S11.2 |

Kinds are those of `.claude/model_policy.toml`. A fix made in both halves (design revision and code in the same commit) is `code`, as the plan's S11 text says; E6 is `transcribe` because it is a design revision from a cited source with a published-vector case.

## 2 · The fixes, item by item

Each entry: what the item says (its id in its source), whether it is open and how that is known, where it lives, the test that fails before the fix, then kind, size, dependencies and any owner decision.

### verdicts

**A1** (TECHNICAL_ROADMAP A1 (Critical)). Closures 'pass' without the stated probability or confidence; NaN runs dropped from the evidence.
- Status: **open**. Known from: tools/evaluate.py:231-270 takes the sorted value at ceil(claim*n) of the finite values (line 256 drops non-finite ones) and compares it with the limit; no lower confidence bound, no 'not demonstrated'. tools/trace.py:142 is a second verdict source ('not met (k of n)'). results/EVALUATION.md still shows ais_3u APE and AKE 'pass'.
- Lives in: both. Design: kpi group: kpi_metric_statistics (design/revisions/S7.14b/kpistats.pc; a percentile of fewer than 371 samples is the largest, S7.14b finding); the closure rows of ais_3u and ais_img_3u (kpi_*_verified, ape/ake). Evidence rows p1a_0, p1a_1 are computed by evaluate, not by a node.. Code: tools/evaluate.py:231-270 (evidence), tools/trace.py:142, tools/vv_report.py; no pass on a design the selection marks infeasible (selection.json)..
- Fails before the fix: tests/test_evaluate.py: 12 runs all within the limit at 99.73 % must give 'not demonstrated (R >= 0.78 at 95 %)'; a NaN run counts as a failure; one verdict function used by evaluate and trace; an infeasible selection never passes.
- Work: code (standard tier), size M, step S11.10. Depends on: the verdict method (step 1, deep) and DD8; A7 (windows) changes which samples are judged. Owner decision: YES (DD8): the confidence level. Options: 95 % one-sided Clopper-Pearson as the roadmap says (default used if the owner is silent); 90 %; per requirement. The method itself (Clopper-Pearson, NaN as failure) is the deep session's.

**A2** (TECHNICAL_ROADMAP A2 (Critical)). ENGINE_CAMPAIGNS says 'the same run count ... the engine is therefore the more conservative'; false (twin 24/40/20/20 runs, engine 1,109; the engine is the less conservative on ape_los_p9973).
- Status: **open**. Known from: tools/engine_campaigns.py:358-366 still writes the sentence as fixed text; results/ENGINE_CAMPAIGNS.md:11 carries it.
- Lives in: code. Design: none (a tool and a result). Code: tools/engine_campaigns.py:358-366 (ledger text); the twin draws asils.campaign.draw (matlab_sils/+asils/+campaign/draw.m); engine draws in adcs-sim campaign code..
- Fails before the fix: A test that the ledger text is built from the run counts of the data (a ledger with unequal counts cannot say 'same'); a campaign of >= 100 identical draws on both sides and a two-sample test, written to the ledger.
- Work: code (standard tier), size M, step S11.10. Depends on: A5 (same draws must be physical); the twin re-fly (step 15). Owner decision: none (the 100-run twin campaign is slow in Octave: a run for the light tier, hours)

**A3** (TECHNICAL_ROADMAP A3 (Critical)). V&V report says every differing verdict is traced; five are not.
- Status: **open**. Known from: tools/vv_report.py:459 writes 'Verdicts that differ (all traced ...)' unconditionally. results/engine_parity.json now has 1 differing verdict of 192 (mission_rw_rcs ape_los_last_half_orbit_p9973) and tools/engine_twin.py NOTES holds no entry keyed by it; the count five of the roadmap is stale, the defect is not.
- Lives in: code. Design: none. Code: tools/vv_report.py:453-459; tools/engine_twin.py:15-60 (NOTES, free text keyed by a string)..
- Fails before the fix: tests/test_report_helpers.py: a differing verdict with no cause-table entry prints 'untraced'; the cause table keyed by (scenario, metric) with a test that every differing verdict in results/engine_parity.json has one or is printed untraced.
- Work: code (standard tier), size S, step S11.10. Depends on: none. Owner decision: none

**A5** (TECHNICAL_ROADMAP A5 (High)). About 4 % of Monte Carlo inertia draws and every edge-campaign inertia corner break the triangle inequality: the body is impossible.
- Status: **open**. Known from: matlab_sils/+asils/+campaign/draw.m:30-33 scales each principal moment independently by +/- frac; engine applies it with truthplant scale_inertia (engine/crates/adcs-sim/src/config.rs:476, gen/truthplant.rs:57); nothing checks Ix+Iy>=Iz. config.rs:588 only checks positivity.
- Lives in: both. Design: dyn_truth_plant (design/revisions/S7.11/truthplant.pc: scale_inertia, inertia_products) - the check or the bounded scaling belongs here. Code: draw.m:30 (twin) and the engine's campaign draw (adcs-sim, same draws by seed); config.rs:476-488..
- Fails before the fix: engine/crates/adcs-sim/tests/inputs.rs: a draw with Ix=1,Iy=1,Iz=2.4 is refused by name; campaign test: 1e4 draws, all satisfy the triangle inequality; edge corners are realisable.
- Work: code (standard tier), size S, step S11.9. Depends on: none. Owner decision: YES (small): disperse R diag R^T with bounded moments (roadmap), or reject and redraw. Default used: reject and redraw (keeps the stored dispersion and its seeds readable); the owner may prefer rotation-based.

**A7** (TECHNICAL_ROADMAP A7 (High)). Judging windows: fine-hold judges half of one orbit, so tracker blinding in the other half is never scored; slews leave 280-320 s unjudged.
- Status: **open**. Known from: matlab_sils/data/scenarios/fine_hold_img.json: ape_los_p9973 window 'last_half_orbit'; slew_img.json: settle from_s 260, ape window 'after_s:320'. MetricWindow has last_orbit (kpistats.pc:12) but no eclipse-transition window.
- Lives in: both. Design: kpi_metric_statistics (design/revisions/S7.14b/kpistats.pc: MetricWindow choice and window rule; add an eclipse-transition window). Code: scenario files under matlab_sils/data/scenarios (exported from the catalogue; the case side), schema.rs window words..
- Fails before the fix: kpi vector case: a run of two orbits judges the window 'last_orbit' over the full orbit and the eclipse window over the +/- transition; a run where the tracker is blind in the first half fails the verdict (it passes today).
- Work: code (standard tier), size S, step S11.10. Depends on: A1 (the same verdict code); step order: windows before the verdict re-run. Owner decision: none (the S7.14b finding: 'pointing' window counts Sun spin modes - kpi states it)

### power and sizing

**Y1** (TECHNICAL_ROADMAP Y1 (Critical)). power_mean passes req.pavg on the coils, wheels and thrusters only; sensors (1.6 W AIS, 2.6 W imaging) appear nowhere.
- Status: **open**. Known from: engine/crates/adcs-sim/src/run.rs:396-402 sums a.p_mtq, a.p_mex, a.p_rcs only; metrics power channels (design/revisions/S7.14b/kpimetrics.pc:65) read those; gb_2 'ADCS peak power' not computed by the sizing (S7 inventory §6).
- Lives in: both. Design: kpi_metric_evaluate (power_mean, power_peak, power_margin kinds); sens group nodes for each part's steady power (catalogue power_steady_W); sizing budget in the design (l3_budget_row_*, gb_*, sizebudget.pc); design_power_system (S7.14: body cells, no seasonal change - noted).. Code: run.rs:396-402 and the twin's asils.run (a.p_*); the recorder channel for the total; engine adcs-design lib.rs select (static budget)..
- Fails before the fix: adcs-sim/tests/power.rs: ais_3u nominal with the sensors on must read >= 1.6 W more than today and break req.pavg 0.5 W; sizing test: the selection's static budget includes sensors.
- Work: code (standard tier), size M, step S11.8. Depends on: A1 (verdict), Y2 (the sizing re-runs), DD61 scope (decision stub). Owner decision: none (scope 'whole ADCS' is the plan's done-when); per-mode sensor duty is the developer's revision, unsigned

**Y2** (TECHNICAL_ROADMAP Y2 (Critical)). CMG and VSCMG benchmarks: 1.4 kg, 5.9 W, fail power; the ADCS400 cluster of 4 CMGs counted as one of four units.
- Status: **open**. Known from: design/revisions/S7.15/sizerotor.pc:3,38 (units = 4 for a pyramid, each half the need) and :27 (cmg_cluster accepted as a CMG); engine/crates/adcs-design/src/lib.rs:287,313 print 'one of {units}' and use the datasheet's cluster figures per unit.
- Lives in: design. Design: design_sizing_cmg, design_sizing_vscmg, design_sizing_rw (sizerotor.pc: rotor_need, rotor_fits); catalogue_datasheet_derive (design/revisions/S7.15/catderive.pc) for the cluster's derived numbers.. Code: lib.rs:287-320 only prints and reads (glue); no relation change in code..
- Fails before the fix: adcs-design test: a catalogue 'cmg_cluster' model is chosen as one unit carrying the whole need (units = 1) and its mass/power are not multiplied by 4; before: 4x.
- Work: code (standard tier), size S, step S11.8. Depends on: S7 finding (catalogue divides by stated zero) belongs with it; Y4 (the need that is sized). Owner decision: none

**Y4** (TECHNICAL_ROADMAP Y4 (High)). h_req = max(h_dist, h_slew), not the sum; the mass lever leaves 1.14x instead of the stated x2.
- Status: **open**. Known from: design/revisions/S7.15/sizedemand.pc:260 'h_req = k_h*size_max(h_dist, h_slew)'.
- Lives in: design. Design: l3_budget_row_01 / gw_2 and the sizing demand method (sizedemand.pc: size_demand), k_h = 2 default at :188 (DD9 margins).. Code: none.
- Fails before the fix: adcs-design test: h_req equals k_h*(h_dist + h_slew); the achieved margin is reported and a lever result below the floor is refused.
- Work: code (standard tier), size S, step S11.8. Depends on: Y2, Y5 (same file); S7 finding 'the sizing stops with a panic when the pump has no feasible point' gets worse when h_req grows: fix it with Y4 (refuse by name). Owner decision: YES (small, DD9): the margin floor. Default: the stated k_h = 2 as the floor, achieved margin reported.

**Y5** (TECHNICAL_ROADMAP Y5 (High)). Demand survey integrates torque in the body frame (no omega x h); orbit-rate transport exceeds the surveyed peak; cyclic/secular split wrong.
- Status: **open**. Known from: design/revisions/S7.15/sizedemand.pc survey_orbit (the sum of torque x dt, 'the integral the running sum', lines 88-135) with no transport term; the torques come from the engine (adcs-design lib.rs survey).
- Lives in: design. Design: survey_orbit in sizedemand.pc (nodes l3_budget_row_01, gw_2, design demand rows). Code: lib.rs survey (calls the method and hands over the torques; may need to hand over the body rate).
- Fails before the fix: sizedemand vector: constant inertial torque on a body rotating at orbit rate must integrate to the inertial momentum (cyclic/secular split analytic); body-frame sum gives a different answer today.
- Work: code (standard tier), size M, step S11.8. Depends on: Y4. Owner decision: none

### models

**M1** (TECHNICAL_ROADMAP M1 (High)). mtq_fmr survives every single fault; a failed fluid ring keeps its momentum forever (zero friction terms).
- Status: **open**. Known from: design/revisions/S7.7/rotorset.pc:115-117 failed rotor coasts with rotor_coast(h, jrot, viscous, coulomb) (wheelfriction.pc:30); ring friction passed as 0.0, 0.0 (engine/crates/adcs-sim/src/product.rs:166); ring desc already has t_sd (rotorset.pc:40).
- Lives in: design. Design: act_rotor_set (rotorset.pc), wheelfriction.pc rotor_coast; ring_constants/ringlimits (spin-down time T_sd). Code: product.rs:160-170 (descriptor fill from the product; reads, calls), actuators.rs:106 comment only.
- Fails before the fix: adcs-sim-core/tests/chains.rs: a failed ring's momentum decays as exp(-t/T_sd), 3 s scale; the fault campaign mtq_fmr transfers the momentum to the body (body rate up to 10 deg/s class). Before: constant.
- Work: code (standard tier), size S, step S11.7. Depends on: act_rotor_set shared with M2, M5 (one agent, one file). Owner decision: none

**M2** (TECHNICAL_ROADMAP M2 (High)). Wheel power leaves out copper loss I^2R: up to +0.75 W per wheel; peak power understated in slews; same for the VSCMG rotor.
- Status: **open**. Known from: design/revisions/S7.7/wheelpower.pc:9 'p = p_steady + abs(tm*om)/eta'.
- Lives in: design. Design: l3_rw_row_07 (wheelpower.pc wheel_power); act_rotor_set uses it at rotorset.pc:133,142. Code: none (twin flies the same generated model).
- Fails before the fix: act vector: wheel_power(0, tau, om) adds tau^2*w_nl/tau_stall (as roadmap); case: stall-torque slew on a wheel shows +0.75 W class.
- Work: code (standard tier), size S, step S11.7. Depends on: Y1 (power budget moves with it); M1/M5. Owner decision: none

**M5** (TECHNICAL_ROADMAP M5 (Medium)). rotor_fail on a CMG does nothing (zero friction): the rotor keeps h0 and the gimbal still turns.
- Status: **open**. Known from: engine/crates/adcs-sim/src/product.rs:187 add_rotor(... Kind::Cmg, ..., 0.0, 0.0 ...) friction zero; rotorset.pc:115 coasts with those; the gimbal of a failed CMG not stopped.
- Lives in: design. Design: act_rotor_set; act_cmg_model (design/revisions/S7.7/cmgmodel.pc). Code: product.rs:187-189.
- Fails before the fix: fault test: rotor_fail on a CMG spins the rotor down and the gimbal command is ignored; before: h0 stays.
- Work: code (standard tier), size S, step S11.7. Depends on: M1 (same file); the CMG bearing friction value is not in the catalogue. Owner decision: YES (small): CMG bearing friction/spin-down. Options: a catalogue datasheet field with a source; or the wheel's friction ratio as a stated developer value. Default: the second, unsigned, source named.

**M9** (TECHNICAL_ROADMAP M9 (Low)). Star-tracker latency > 0.5 s silently becomes 0.
- Status: **open**. Known from: engine/crates/adcs-sim-core/src/sensors.rs:160 history span 0.5 s (ST_HIST); beyond it the delayed attitude falls back to the newest.
- Lives in: design. Design: sens_star_tracker (design/revisions/S7.9/sttracker.pc: st_history, st_delayed). Code: config/product check: a refusal belongs in config.rs `check`.
- Fails before the fix: inputs.rs: a product stating st latency 0.6 s is refused by name (CHANGING.md: a refusal is never a substitution).
- Work: code (standard tier), size S, step S11.7. Depends on: none. Owner decision: none (rule decides: refuse)

**M10** (TECHNICAL_ROADMAP M10 (Low)). GNSS rate_Hz ignored.
- Status: **open**. Known from: engine/crates/adcs-sim/src/run.rs:263 'gps: ((1.0/dt).round()).max(1)' - one fix per loop tick regardless of rate_hz; GpsDesc.rate_hz read in product.rs:264 but unused.
- Lives in: code. Design: none. Code: run.rs:253-264 (Rates) and the twin's asils.run schedule.
- Fails before the fix: engine test: rate_hz 0.1 gives one fix per 10 s; twin parity on the same.
- Work: code (standard tier), size S, step S11.7. Depends on: none. Owner decision: none

### GNC logic

**F2/G3** (TECHNICAL_ROADMAP F2/G3 (Critical)). A silent gyro keeps feeding its last rate to the MEKF, controllers and detumble exit for 60 s; the 900 s tracker coast then lets attitude diverge with no fault (demo: 143 deg in 50 s).
- Status: **open**. Known from: fsw/src/adcs_drv.c:135 'if (o.gyro_ok) gl_o3(o.w, z->w)' keeps the last w; fsw/pseudocode/08_mode_manager.pc:213-242 holds w0 and counts gyro_age, fault at 60 s (:198); fsw/src/adcs_fsw.c:231-246 predicts on z->w and coasts st_coast_s; :305 only the detumble law switches on gyro_age. The engine has no gyro_fail fault kind (run.rs:222-241).
- Lives in: both. Design: fsw_modes (gdn: 08_mode_manager.pc, fdir_hold at :206-242), fsw_estimation (nav: mekf_predict with inflated Q), fsw_drivers (fsw: 09_drivers.pc:197). Code: tick: fsw/src/adcs_fsw.c:215-246 (C), fsw-rs/src/fsw.rs, matlab_sils/+asils/+fsw/step.m; driver glue adcs_drv.c:135; a gyro_fail fault kind in run.rs:222 (+schema.rs, twin faults)..
- Fails before the fix: fsw-rs/tests/fsw.rs and fsw/tests/test_fsw.c: gyro frozen -> stale flag within 3 ticks, rate zeroed, Q inflated, coast ended; new scenario fault_gyro_silent: attitude error stays bounded (143 deg in 50 s today).
- Work: code (standard tier), size M, step S11.5. Depends on: TS4 rig (step 4: gyro_fail fault kind, tick tests); F5, G4 (same data path). Owner decision: none (stale limit < detumble hold, per roadmap)

**G1** (TECHNICAL_ROADMAP G1 (High)). TRIAD start with isotropic P=(0.05 rad)^2 and sin theta >= 1e-3; divergence counter reset by any accepted update.
- Status: **open**. Known from: fsw/src/adcs_fsw.c:223-226 adcs_mekf_init(..., 0.05, ...) after adcs_triad; fsw/pseudocode/03_estimation.pc:205 sin threshold 1e-3; n_rej reset at :264 region.
- Lives in: both. Design: fsw_estimation (nav: triad at 03_estimation.pc:205, mekf_init :7; the rejection counter is in the tick). Code: tick: adcs_fsw.c:214-226,264 (C), fsw.rs, step.m.
- Fails before the fix: fsw tests: TRIAD with Sun and field 5 deg apart is refused (sin < 0.15..0.25); a magnetometer rejected forever triggers per-sensor re-init; before: accepted with P=0.05^2.
- Work: code (standard tier), size M, step S11.6. Depends on: F4 (same init block); TS2 estimator checks come later (S13). Owner decision: none (threshold within 0.15-0.25 chosen with the Shuster-Oh source; unsigned)

**G2** (TECHNICAL_ROADMAP G2 (Medium-high)). Rotor torque commands clipped per actuator: direction not kept; no momentum-saturation check.
- Status: **open**. Known from: fsw/pseudocode/07_allocation.pc:95 scales gimbal rates as a vector (done for gimbals) but the rotor command is clipped per rotor (plant act_clamp, rotorset.pc:126; driver Q15); adcs_alloc_rotors at fsw/src/adcs_fsw.c:162 has no scaling or saturation exclusion.
- Lives in: both. Design: fsw_allocation (ctl: 07_allocation.pc rotor allocation). Code: adcs_alloc_rotors in fsw/src/adcs_fsw.c:162 and fsw-rs; tick :395-440.
- Fails before the fix: fsw test: a command with one axis beyond torque_max is scaled as a whole vector, direction kept; a saturated rotor is dropped from the allocation.
- Work: code (standard tier), size S, step S11.6. Depends on: G4, G6 (control chain); F2. Owner decision: none

**G4** (TECHNICAL_ROADMAP G4 (Medium)). No rate damping when attitude is invalid: pointing modes command hdot=0, coils-only modes do nothing.
- Status: **open**. Known from: fsw/src/adcs_fsw.c:402 control law runs only 'S.ad_ok'; no damping branch for !ad_ok (the only !ad_ok output is the tick's rate copy at :270).
- Lives in: both. Design: fsw_steplaws (ctl: 06_step_laws.pc), fsw_modes. Code: tick :395-445, fsw.rs, step.m.
- Fails before the fix: scenario fault_st_blind: attitude invalid for an eclipse; body rate stays bounded with damping; today it drifts.
- Work: code (standard tier), size S, step S11.5. Depends on: F2/G3, F5 (rate source when the gyro is silent: B-dot). Owner decision: none

**G6** (TECHNICAL_ROADMAP G6 (Medium)). PID loops tuned to rw_bandwidth, damping 2: Kp = J wn^2 acts on the quaternion vector part (theta/2), so they fly at about 0.7x the bandwidth.
- Status: **open**. Known from: design/revisions/S7.13/fswrw.pc:14 'kp = ii*wn*wn', :16 ki = 0.15 ii wn^3; fswmtq.pc:36 same for the magnetic PD; config.rs:344-345 only calls them.
- Lives in: design. Design: fsw_param_rw_Kp / _Kd / _Ki (fswrw.pc fsw_rw_pid), fsw_param_mtq_Kp (fswmtq.pc fsw_mtq_pd), sb_kp (fswmtq.pc:79). Code: none (the blob is regenerated from the design).
- Fails before the fix: TS3 step test: 0.5 deg step, disturbances off, rise/settle vs the closed-loop poles within 10 %; bandwidth = rw_bandwidth within 10 %. Today ~0.7x. S7 findings to settle in the same revision: the magnetic SMC slope wn/z twice the fine pointing's; ctl gc_0/gc_1 analysis row not the flown loop (ais_img_3u settle 56.6 s vs 2.2 s).
- Work: code (standard tier), size S, step S11.6. Depends on: TS3 (the test); DD33 (gain design method). Owner decision: none (fsw/ctl confirm in S20)

**F4** (TECHNICAL_ROADMAP F4 (High)). Star-tracker initialisation skips the norm check (|q|=0.5 frame becomes identity with confident P; demo 180 deg); unconfigured head can be selected.
- Status: **open**. Known from: fsw/src/adcs_fsw.c:214-221 init takes z->q_st[h] with no norm test; the update at :235 does (ST_NORM_TOL 1e-3, :25); n_heads >= 1 not required.
- Lives in: both. Design: fsw_estimation (nav): a plausibility function used by init and update. Code: tick :214-221 (C), fsw.rs, step.m.
- Fails before the fix: fsw tests: frame with |q| = 0.5 refused at init and counted; n_heads = 0 never selects a head.
- Work: code (standard tier), size S, step S11.6. Depends on: G1 (same block). Owner decision: YES (boundary, small): put the plausibility test in a design method both the init and update call (recommended) or leave it in the tick shell.

**F5** (TECHNICAL_ROADMAP F5 (High)). Without a gyro the detumble exit is judged on a zeroed rate, so it always exits after the hold.
- Status: **open**. Known from: 08_mode_manager.pc:240 'if not gyro_ok and not has_gyro: w = 0'; exit test at adcs_fsw.c:369 (ho_t) on z->w.
- Lives in: both. Design: fsw_modes (gdn); the B-dot rate (adcs_fsw.c:310) already computes a field derivative. Code: tick :360-372.
- Fails before the fix: fsw test: has_gyro=0, tumbling body, hold elapsed: no exit while the B-dot rate is above ho_out; before: exits.
- Also fails before the fix: a healthy gyro, where the exit must use the gyro's rate; and a gyro failing during
  detumble, where the exit must move to the B-dot rate and not exit early. Scenario cases: gyro, no gyro, and a
  gyro failing in detumble.
- Work: code (standard tier), size S, step S11.5. Depends on: F2/G3. **Owner's answer (6 Nov 2026, section 5):
  both rate sources.** The gyro's rate when the gyro is healthy; the B-dot rate estimate from the magnetometer
  when there is no gyro or it has failed. The gyro's health does the switching.

### flight software

**F1** (TECHNICAL_ROADMAP F1 (Critical)). Onboard date = blob jd0 + time since init: every reset rewinds the calendar; a clock step backwards wraps to 585 years; the schedule replays after reboot.
- Status: **open**. Known from: fsw/src/adcs_fsw.c:193-194 'S.t = (now_ns - S.start_ns)*1e-9' (uint64 subtraction wraps on a backwards step) and 'S.jd = p->jd0 + S.t/86400'; same fsw-rs/src/fsw.rs:216 and the twin step.m.
- Lives in: both. Design: env_time_frames (design/revisions S7.3, 02_time_frames_models.pc: epoch handling), fsw_param_jd0 (stated). Code: runtime: tick and init in adcs_fsw.c / fsw.rs / step.m; HAL has no non-volatile time (adcs_hal.h).
- Fails before the fix: TS4 reset test: warm reset at day 30 keeps JD within 1 s and the schedule is not replayed; clock step -1 s / +1 h: monotonic, bounded.
- Work: code (standard tier), size M, step S11.4. Depends on: DD50 (onboard time source); TS4 rig; F3 (the same tick arithmetic). Owner decision: YES (DD50): time source. Options: (a) GNSS time when a fix is held, else the last good time carried by a monotonic bounded clock (no HAL change, recommended for S11); (b) time telecommand; (c) non-volatile time through a new HAL call (HAL v2, S15). Default: (a) and the clock fix; (b),(c) later.

**F3** (TECHNICAL_ROADMAP F3 (High)). Rate groups from floating time: with +/-1 us jitter the fine law runs on 63 % of ticks and 25 % of coil cycles skip updates.
- Status: **open**. Known from: fsw/src/adcs_fsw.c:205 fmod(S.t + 1e-9, mtq_period) and :401-402 'S.t - S.last_ctrl >= rw_dt - 1e-9'; fsw-rs/src/fsw.rs:230 fmod; fuzz uses exact timestamps (fsw-rs/tests/fuzz.rs).
- Lives in: code. Design: none (runtime: tick and scheduler). Code: adcs_fsw.c, fsw-rs/src/fsw.rs, matlab_sils/+asils/+fsw/step.m (one integer tick counter per rate group).
- Fails before the fix: TS4 jitter test: +/-1 us and +/-5 ms on the timestamps, every rate group runs on >= 99.9 % of its ticks; before 63 %.
- Work: code (standard tier), size M, step S11.4. Depends on: TS4 rig; F1. Owner decision: none (DD35 integer-tick groups is the roadmap's fix)

**F10/G8** (TECHNICAL_ROADMAP F10/G8 (Medium)). Single-precision build is broken (incompatible pointers, 278 conversion warnings) while docs say it is tested.
- Status: **open**. Known from: gcc -DADCS_REAL_FLOAT -fsyntax-only on fsw/src/adcs_fsw.c: incompatible-pointer warnings at fsw/src/adcs_alg_glue.h:61,67 (20 warnings for this file alone); fsw/pseudocode/00_conventions.md:18 claims a float build 'tested separately'; fsw-rs/Cargo.toml has no f32 feature.
- Lives in: code. Design: none (a claim in a doc and a build flag). Code: fsw/pseudocode/00_conventions.md:18, fsw/include/adcs_math.h:14, adcs_alg_glue.h.
- Fails before the fix: a test that the claim and the build agree: either `make f32` builds without warnings or the doc says there is none (docs test).
- Work: code (standard tier), size S, step S11.4. Depends on: DD46/DD47. Owner decision: YES (small): remove the claim now (recommended, S) or support the build (M, needs DD46's FPU decision).

### twin and density

**E1** (TECHNICAL_ROADMAP E1 (High)). ENGINE_PARITY says field 'bit-identical' engine vs twin; twin not moved to GMST x IAU-76 precession; they differ up to 0.23 deg; the parity tool compares only |B|.
- Status: **partly fixed**. Known from: The twin now flies the design's generated frames: matlab_sils/+asils/+orbit/env.m:9 applies precession_iau76 before Earth rotation; the onboard side is the generated asils.alg/relations (S7.17-S7.19b). Still open: tools/engine_twin.py:118 reports only 'max |B| ratio - 1' and the Sun direction, no field direction.
- Lives in: code. Design: none. Code: tools/engine_twin.py (parity table), results/ENGINE_PARITY.md (older than the runs it pairs: S7 finding).
- Fails before the fix: tests/test_engine_helpers.py: the parity tool computes the angle between engine and twin B, Sun and nadir directions; a case where the twin omits precession shows 0.2 deg.
- Work: code (standard tier), size S, step S11.11. Depends on: twin re-fly (step 15). Owner decision: none

**E6** (TECHNICAL_ROADMAP E6 (Medium)). Spec 'worst aero torque' row uses a static table 7.4x below the DTM2020 high-activity peak, and the spec still names NRLMSIS.
- Status: **open**. Known from: node m3_3 (tests/regression/design.tndb) is the static exponential table (Vallado table 8-4) whose why-text names NRLMSIS 2.0; gd_1 (aerodynamic torque) reads it. DTM2020 is already a method (env_dtm2020_operational, design/revisions/S7.3b/dtm2020.pc), so no transcription is needed; the sizing survey's F10.7=250 keeps Kp=2 (P6, S12).
- Lives in: design. Design: m3_3, gd_1 (and the density rows of the survey, env_density_model). Code: none.
- Fails before the fix: env vector: m3_3 at 550 km, F10.7 = 250 (high), 70 (low) equals the DTM2020 value within the published tolerance; before: 7.4x low / 15x high. gd_1 'worst' uses the high bundle.
- Work: transcribe (standard tier), size M, step S11.11. Depends on: none (DTM2020 exists); the sizing survey (Y4, Y5) re-runs after it. Owner decision: none

### orbit step

**P5** (ORBIT_PROPAGATOR_AUDIT P5 (Low-med)). RK4 error grows steeply with step (0.43 m at 10 s -> 1.87 km at 60 s); 600 s accepted.
- Status: **open**. Known from: engine/crates/adcs-sim/src/config.rs:599 'within("engine.orbit_step_s", ..., 1e-3, 600.0)'.
- Lives in: code. Design: none. Code: config.rs:599.
- Fails before the fix: inputs.rs: engine.orbit_step_s = 60 is refused by name; 30 accepted; before: accepted.
- Work: code (standard tier), size S, step S11.12. Depends on: none. Owner decision: none (cap at 30 s; RK78 nodes belong to S12)

### soft OILS

**O2** (TEST_BENCH_PLAN O2 (H)). A missing plugin silently changes the time base (exec max 0.69 ms instead of 5.13 ms), still passing.
- Status: **open**. Known from: engine/crates/adcs-fsw-abi/src/lib.rs:173 'if plugin.is_file()' adds -plugin, else runs without.
- Lives in: code. Design: none. Code: adcs-fsw-abi/src/lib.rs:164-173; manifest in tools/engine_oils.py.
- Fails before the fix: adcs-fsw-abi test: --oils without fsw/build/insn_count.so is refused; the manifest records the time-base source.
- Work: code (standard tier), size S, step S11.13. Depends on: none. Owner decision: none

**O12** (TEST_BENCH_PLAN O12 (M)). Device map contradicts the HAL header on almost every port.
- Status: **open**. Known from: archive/design-1.0/spec/rig/device_maps/example_3u.toml: magnetometer bus=1 addr 0x1E; fsw/include/adcs_devices.h:13 ADCS_MAG_BUS 0u, coils/rotors on CAN (:35). The map moved to archive/ with 1.0 and nothing reads it.
- Lives in: code. Design: none. Code: fsw/include/adcs_devices.h (what flies), the device map (a rig input).
- Fails before the fix: a CI check (tools or test) that each port in the map equals adcs_devices.h; fails today on the magnetometer bus.
- Work: code (standard tier), size S, step S11.13. Depends on: S7 finding: no fsw/devices.toml exists (the numbers are devices.rs and adcs_devices.h). Owner decision: YES (small): which is the truth. Default: the header (what the code and the emulators run); the map is corrected from it and checked in CI; generating the header from the map is the alternative (M).

**O19** (TEST_BENCH_PLAN O19 (H)). SILS-vs-OILS acceptance is verdict agreement only; knife-edge metrics flip on noise.
- Status: **open**. Known from: tools/engine_oils.py:89 'agree = sum(pass_sils == pass_oils)'; no numeric tolerance.
- Lives in: code. Design: none. Code: tools/engine_oils.py:89-141.
- Fails before the fix: tests: a pair 0.010 vs 0.0100001 deg with the verdict flipped is accepted within the numeric threshold, a pair 0.010 vs 0.013 fails; thresholds of TEST_BENCH_PLAN section 5.
- Work: code (standard tier), size S, step S11.13. Depends on: O20. Owner decision: none

**O20** (TEST_BENCH_PLAN O20 (M)). CI runs no timing: no --oils run, no deadline regression, no CPI sweep.
- Status: **open**. Known from: .github/workflows/ci.yml:72-75 flies only a C/Rust-vs-QEMU parity of nadir_hold_ais (300 s); no --oils run, no deadline check (grep).
- Lives in: code. Design: none. Code: .github/workflows/ci.yml; tools/engine_oils.py.
- Fails before the fix: a CI job that runs >= 5 scenarios with --oils and fails when the deadline margin falls below the stored minimum; B0's done-when.
- Work: code (standard tier), size S, step S11.13. Depends on: O2, O19. Owner decision: none

**O21** (TEST_BENCH_PLAN O21 (M)). No wire-level recording: a HIL anomaly cannot be replayed in soft OILS.
- Status: **open**. Known from: no recorder in fsw/targets/link/adcs_link.c or tools/engine_oils.py (grep).
- Lives in: code. Design: none. Code: fsw/targets/link/adcs_link.c, adcs-fsw-abi, tools/engine_oils.py.
- Fails before the fix: a run with --record writes the TICK/OUT byte stream with wall time; replaying it reproduces the verdict.
- Work: code (standard tier), size S, step S11.13. Depends on: O11. Owner decision: none

**O11 (B0)** (TEST_BENCH_PLAN O11 (H); B0). No bus-level fault injection (NACK, corrupt CRC, CAN error frames, stuck registers, delays, brownout).
- Status: **open**. Known from: engine/crates/adcs-sim/src/run.rs:222-241 knows only rotor_fail, gimbal_stuck, st_head_fail, coil_fail, gyro_bias_step, gps_outage, rcs_valve_fail, mag_fail; none at bus level.
- Lives in: code. Design: none. Code: run.rs fault table, schema.rs, adcs-sim-core emu.rs, the twin's +faults.
- Fails before the fix: per fault point in the device map, a case where the fault is injected and the driver raises its fault flag (TS4's 'PWM write error: fault raised').
- Work: code (standard tier), size M, step S11.14. Depends on: O12 (the fault points come from the map); TS4 rig (shares the fault kinds). Owner decision: none

### tests

**TS11** (TEST_STRATEGY TS11 (H)). CI never re-flies the scenarios; a gain change can merge green with every stored verdict stale.
- Status: **open**. Known from: no results/golden.json (ls results); 48 scenarios in matlab_sils/data/scenarios; ci.yml runs check_all and no scenario re-fly.
- Lives in: code. Design: design cases (the 48 scenarios' metrics are the table's rows). Code: tools (a new golden tool or engine.py command), .github/workflows/ci.yml.
- Fails before the fix: a test that a gain change moves a golden metric and the check names the scenario and metric; an unchanged tree re-flies all 48 inside the tolerance.
- Work: code (standard tier), size M, step S11.3. Depends on: none; first, so every later fix shows what it moved. Owner decision: YES (small): the tolerance. Default: bit-exact for the same toolchain fingerprint, a relative tolerance per metric otherwise (stated in the table).

**TS4** (TEST_STRATEGY TS4 (C)). Robustness and negative tests miss the known failure classes; fuzz uses exact timestamps and <= 200 ticks; no reset tests; mutation excludes fsw, modes, fdir, drivers.
- Status: **open**. Known from: fsw-rs/tests/fuzz.rs (3000 structured inputs, no jitter, no reset); no 'reset' in fsw-rs/tests; no gyro_fail / reset / clock fault kind in run.rs:222-241; fsw/tests/test_fsw.c has no warm-reset case.
- Lives in: code. Design: design cases (new scenarios for the gyro, reset and timing faults - the plan's done-when). Code: fsw-rs/tests, fsw/tests/test_fsw.c, run.rs fault kinds, matlab_sils/+asils/+faults.
- Fails before the fix: tests named in TEST_STRATEGY TS4: warm reset day 30, clock step -1 s/+1 h, jitter, gyro frozen, |q| = 0.5, gyro 1000 rad/s, babbling CAN. They are the failing tests of F1-F5 and land with them; the mutation and coverage gates are U4 (S15), not S11.
- Work: code (standard tier), size M, step S11.4. Depends on: F1-F5 land with the matching test; the rig step 4 first. Owner decision: none

**TS3** (TEST_STRATEGY TS3 (H)). Controllers never checked against their linear design (G6 was found by audit).
- Status: **open**. Known from: no step-response or sweep test in fsw-rs/tests (ctl.rs, laws.rs test laws, not loops).
- Lives in: code. Design: none. Code: fsw-rs/tests, a harness over the engine with disturbances off.
- Fails before the fix: per law and axis: 0.5 deg step, rise/overshoot/2 % settling vs the poles of the blob within 10 %; sweep GM >= 6 dB, PM >= 30 deg, bandwidth = rw_bandwidth within 10 %. Fails today for the PID (0.7x).
- Work: code (standard tier), size M, step S11.6. Depends on: G6 (the fix it guards). Owner decision: none

**TS17** (TEST_STRATEGY TS17 (L-M)). 'Passes if not broken' oracles: replace t_igrf range by point vectors, t_shadow 3 points by a boundary sweep, detumble_reduces_rate by an analytic rate, t_mekf by NEES.
- Status: **open**. Known from: matlab_sils/tests/run_all_tests.m:84 t_igrf, :96 t_shadow, :148 t_mekf; engine/crates/adcs-sim/tests/engine.rs:43 detumble_reduces_rate (w1 < 0.2 w0).
- Lives in: code. Design: none (point vectors are published values: from the S7.3 IGRF/shadow vectors). Code: the four tests.
- Fails before the fix: the new oracle fails a mutated model (a 5 % IGRF coefficient change, a shifted shadow edge) that the old one passes.
- Work: code (standard tier), size S, step S11.15. Depends on: none (the audit of every other test is the same step). Owner decision: none

### The 15 design decisions (`docs/DESIGN_DECISIONS.md`)

What the item says: each of the 15 "most need an owner-approved record" decisions has a design node with options, evidence and a signature slot (the DR format of that page); the first step of roadmap U0 is "write DR stubs ... tag the literals ... the owner signs". Open today for all 15: no node of kind decision exists (the regression copy's node kinds are group, internal, leaf, achieved, required, closure_analysis, closure_verified, closure_interface, interface), `design/decisions/` does not exist, and the register marks them M, O, P or I (none R). Where they live: design, in the owning group, written as stated or open nodes by a developer revision (`[[revision.node]]` in `design/revisions_2_0.toml`, behaviour `stated` with the present choice, or `open` where none is made); the literal each governs (`k_h = 2` in `sizedemand.pc:188`, `st_coast_s`, the 60 s gyro limit at `08_mode_manager.pc:198`, the 15-24 run campaigns) is tagged with the id. The plan gives them to the deep tier. The test: a design check that each stub has options, evidence and an unsigned status, and that each tagged literal names its id (the check proposed by `DESIGN_DECISIONS.md`, to extend `tools/groups.py --check`: a size M tool change that belongs with step S11.2). Signing is the owner's, in S20.

| decision | title | register | owning group | what the stub must carry |
|---|---|---|---|---|
| DD8 | Statistical interpretation and confidence per requirement | M (missing) | gdn/kpi | Urgent. Drives A1: 'level 99.73 only'. Options: 95 % one-sided CP; 90 %; per requirement. |
| DD2 | Nadir definition and yaw steering | O (open) | gdn | geocentric -r vs geodetic nadir vs yaw steering; E7 up to 0.17 deg; payload ICD needed. |
| DD39+DD15 | Safe-mode concept, independence, magnetometer redundancy | O / M | fdir | coils-only detumble (G5 deadlock) vs sun-spin vs second magnetometer; needs the FMECA. |
| DD40 | FDIR hierarchy (gyro staleness, 900 s coast, watchdog) | M | fdir | Fixes F2/G3 limits (stale limit < detumble hold): the stub records the numbers chosen. |
| DD50 | Onboard time, synchronisation, TM/TC standard | M | nav/fdir | Urgent. Gates F1: GNSS time vs time TC vs non-volatile time. |
| DD12 | Solution vs benchmark family policy | P | design | Urgent, business and engineering; lighter feasible benchmarks excluded today. |
| DD59+DD60 | Ranking rule; may robustness break the budget | O | design | least mass (code) vs worst margin (SPEC 8.6); escalation past the budget (D15, Y6). |
| DD9+DD61 | Margins policy and budget scope | I / M | design/act | k_h = 2, k_tau 1.5 unsourced; power scope = whole ADCS (Y1) is the plan's; mass growth allowance Y3 is S12. |
| DD14+DD19 | Star-tracker heads, placement, keep-out | P / M | sens | the loop overrides the recorded rationale (one head removed as a mass lever). |
| DD46+DD47 | OBC, FPU, numeric precision | O / I | fsw/lab | Gates F10: remove the f32 claim or support it; board open (H1). |
| DD22+DD23 | Momentum envelope, saturation margin, dump strategy and interval | I / O | act/ctl | req.hsat and req.dump blank; interval defaults to a quarter orbit (lib.rs:204). |
| DD33+DD35 | Gain design method with margins; integer-tick rate groups | I | ctl/fsw | Records G6's gain method and F3's integer tick. |
| DD57 | Monte Carlo sizing and dispersion pedigree | O | vv | 15-24 runs per campaign, 1109 for claims; ties to DD8 and A5 (dispersion method). |
| DD52 | N2O pressure vessel factors; galinstan limits; compatibility | M | act/risk | Urgent: MEOP 70 bar with no proof/burst factor; Ga/Al compatibility unrecorded. |
| DD1+DD38 | Complete mode set with transition matrix; LEOP and tip-off range | P / M | gdn | tip-off 10 deg/s UNCONFIRMED; no separation/LEOP mode. |

## 3 · Found on the way: S7_INVENTORY section 6

Every finding of `docs/S7_INVENTORY.md` section 6, grouped as that page groups them (the text there is the record; the mapping below is from its words, and the entries that touch S11 items were checked in the code). "Owner" means the owning group decides in S20; S12-S16 are the later phases of the upgrade.

| S7 finding | goes to |
|---|---|
| env: torques.rs cites albedo 0.31 and 235 W/m^2, flies 0.30 and 237 | env owner; S12 (E8-E15 environment details) |
| env: two forms of solar pressure (4.56e-6 N/m^2 over distance vs F_sun/c) | env owner; S12 |
| env: two low-precision Sun series (engine's and the flight software's) | env owner; S12 |
| dyn S7.4/S7.5/S7.11: cm direction, surface constants, flexible mode delta, cm offset | closed by S7.11 (the engine reads the design); dyn owner |
| act S7.7/S7.11: momentum-device defaults, rotor telemetry noises; a non-ring rotor carries ring fields at neutral values | M1 and M5 touch the same descriptor (step S11.7); otherwise act owner |
| sens S7.6/S7.8/S7.9/S7.10: Earth half-angle, no Earth sensor in any product, no shipped scenario flies the tracker noise/image model, image-chain constants, st_attitude ill-conditioned (Jacobi) | sens owner; S12; F4's test is a flight-software unit test, so needs no scenario |
| gdn S7.8: a product with no boresight flies +Y and Sun axis -Z | gdn owner (DD2, DD6); not S11 |
| env S7.11: orbit period by two forms; model switches stated in code | env owner; S12 |
| oils, fsw S7.12: drivers write constants inline; no fsw/devices.toml | O12 (device map vs header) in S11.13 uses the same numbers; the naming of driver constants is the fsw owner's change |
| fsw S7.13: a slot with no algorithm still flies a law; magnetic SMC slope wn/z twice the fine pointing's; GG feed-forward judged at one reference point | SMC slope: G6 (S11.6, settled in the same revision); the rest fsw owner |
| fsw, dyn S7.13: GUID table of the mode manager's held by no node | fsw/gdn owner; S14 |
| tools S7.13/S7.19: evaluate reads no stated value.number | fixed in S7.19 |
| twin S7.13/S7.17: config.m and init.m compute their own | closed by S7.17 |
| kpi S7.14b: two libm in the metrics; window = every sample in a mode other than the first; percentile of fewer than 371 samples is the largest; empty-window defaults; metric defaults | A7 (windows) and A1 (percentile, confidence) in S11.10; libm and defaults kpi owner |
| pnt S7.14: page total vs gp_5 two formulas; jitter lever and bandwidth; incomplete budgets | S13 (budgets, T1/T2); pnt owner |
| pnt S7.19: gp_0/gp_1 wiring would close a loop | pnt owner; S13 |
| design S7.14: power system node (cells on six faces, no seasonal change, battery without limits) | Y1 uses it for the whole-ADCS budget (S11.8); seasonal and thermal detail S12 |
| tools S7.14/S7.15: catalogue derive rule and Floquet left; evaluate takes a module's last function | closed by S7.15/S7.15b |
| act, design S7.15: sizing panics (exit 101) when the pump grid has no feasible point (4 of 300 knob sets) | Y4 (S11.8): h_req grows with the sum, so the refusal by name is made with it |
| catalogue S7.15: catderive divides by a stated 0 | Y2 (S11.8) touches catderive; refuse a zero or make NEED ask for positive numbers |
| translators S7.10/S7.15/S7.15b/S7.8: -0 draw, MATLAB complex sqrt/acos (fixed), module renumbering, Default derive on arrays > 32 | translator owner; a later stage (S12+); the two fixed are closed |
| ctl S7.15b: ais_3u damping certified at 1 - 2.5e-13 (marginal); certificate restates the laws and orbit | S13 (T1, T2) and the ctl owner |
| ctl S7.15b: tool refused LQR and sliding-mode laws | ctl owner; no answer changed |
| language S7.15b: buffer inout only | later (S12+ translators) |
| docs S7.15/S7.14/S7.10: HEALTH.md says 36 built-in; page splices; template placeholder | chore at the end of S11 (regenerate HEALTH.md, S11.17) |
| env S7.6 (not caused by it): results/ENGINE_PARITY.md older than the runs it pairs | A2/A3/E1 (S11.10-11): the ledger is regenerated from the data |
| design: twin sizes wheels by a law, engine selects from the catalogue; twin's fluid ring has no pump design | S12/S13 (design owner); not S11 |
| design: gb_2 ADCS peak power is not computed by the sizing | Y1 (S11.8): peak power is judged on the total |
| all: adcs-physics generated and unused; hand-written copies of the same laws | S12 (V1) and the boundary check; not S11 |
| fsw: the 147 fsw_param_* hold layout, not values | closed by S7.13 |
| the conversion: behaviour() makes any node with code.* built-in | closed by S7.1b / S7.15b (0 built-in) |
| design S7.18: +asils/+relations not a function of the design file alone; twin campaign collect fixed; twin fingerprint moves | the fingerprint note: the re-fly in S11.16 refreshes every stored twin run; the rest closed |
| boundary S7.19/S7.19b: 20 declared exceptions removed; Earth rate, Kepler period, EOP, nav orbit, precision orbit, argument of latitude | closed by S7.19b (tools/boundary.toml declares none); the nav onboard orbit is generated into the tick |
| ctl S7.19 (a closure moves): ais_img_3u settling by analysis 56.6 s vs flown 2.2 s (gc_0, gc_1 vs fsw_tune_rw_bandwidth) | G6/TS3 (S11.6): the flown loop is the referent; ctl states gc_0/gc_1 as flown or wires them to the tuning |
| env, act S7.19: analysis rows and the sizing give the same quantities two ways (gd_4, gd_5, gm_1, gm_2) | env and act owners; S13 |
| gdn, fsw S7.19: a case names no product, so product-dependent parameters use gdn's default boresight | gdn/fsw owner; S14 |
| fsw S7.19: LQR gains need the Riccati solve; interpreter has no such builtin | later (S12+ translator); a care builtin |
| tools S7.19: the health map for rows computed during a run | closed by S7.19 |

## 4 · Dependencies that fix the order

- TS11 (golden table) comes first, so each fix shows which scenarios and metrics it moved.
- The fault rig (TS4: `gyro_fail`, reset, clock step, jitter) comes before F1-F5, G4: their failing tests need it.
- F3 and F1 change the tick's arithmetic in all three copies; the other tick fixes (F2, F4, F5, G1, G2, G4) follow so they are made on the final arithmetic.
- M2 and Y1 both move the power numbers; Y2, Y4, Y5 move the sizing; E6 moves the demand survey. Do M, then Y (S11.7, S11.8), then E6 (S11.11), then re-run the selection once.
- A7 (windows) and A1 (confidence) come after every fix that moves a metric, and A2's 100-draw campaign after A5.
- Each design change is a developer revision with its source, then `python3 tools/pcode.py gen` and `python3 tools/engine_build.py gen`; each tick or model change is made in Rust, C and MATLAB in the same commit (`docs/CHANGING.md`) and proven by `engine.py twin-parity` and `engine.py fsw-parity`.
- If a method's allowed calls change (for example a design method the tick calls), the toolbox version in `adcs-sim/src/source.rs`, `tools/design_inputs.py` and `python/trinetra_adcs/design.py` is raised together. None of the steps below is expected to; F4's plausibility method and F2's rate hold call only the maths toolbox.

## 5 · Owner decisions

All have a default; the developer takes the default and marks it unsigned if the owner does not answer. Nothing here is signed in S11.

| # | question | item | options | default |
|---|---|---|---|---|
| 1 | Confidence level of a verdict | A1 (DD8) | 95 % one-sided Clopper-Pearson (roadmap); 90 %; set per requirement | 95 %, NaN a failure, else "not demonstrated (R >= x at 95 %)" |
| 2 | Source of onboard time after a reset | F1 (DD50) | (a) GNSS time held by a monotonic bounded clock; (b) a time telecommand; (c) non-volatile time through a new HAL call (HAL v2, S15) | (a) now; (b), (c) later |
| 3 | Single-precision build | F10/G8 (DD46/47) | remove the claim (S); support it (M, after the FPU choice) | remove the claim |
| 4 | How to disperse the inertia | A5 | redraw until the triangle inequality holds; disperse R diag R^T with bounded moments | redraw |
| 5 | Detumble exit without a gyro | F5 | B-dot rate estimate for the exit (exists in the tick); refuse auto-next without a gyro | B-dot estimate |
| 6 | Which is the truth, the device map or the HAL header | O12 | header wins and the map is checked against it in CI; generate the header from the map (M) | header wins |
| 7 | Golden table tolerance | TS11 | bit-exact for the same toolchain fingerprint, relative tolerance otherwise; one tolerance everywhere | bit-exact per fingerprint, per-metric relative otherwise |
| 8 | Margin floor for h_req | Y4 (DD9) | the stated k_h = 2; a derived margin | k_h = 2 floor, achieved margin reported |
| 9 | CMG bearing friction and spin-down | M5 | a datasheet field with a source (catalogue change); the wheel's friction ratio as a stated developer value | the wheel's ratio, source named |

**The owner's answers (6 Nov 2026):** the defaults for 1-4 and 6-9. For 5, both sources of rate: a product with a
healthy gyro leaves detumble on the gyro's rate. A product with no gyro, or whose gyro has failed or been declared
unhealthy, leaves on the B-dot rate estimate from the magnetometer. The switch between them is the gyro's health,
not a setting, and the scenario cases hold all three: gyro, no gyro, and a gyro failing during detumble. It is
still the developer's revision, marked unsigned like the rest.

Smaller choices the developer makes and marks unsigned, with the source on the revision: G1's sin threshold (0.15-0.25, Shuster and Oh), F4's plausibility placement (a design method both the init and the update call), P5 (cap at 30 s; RK78 is S12), E6 (high bundle for "worst").

## 6 · Proposed order of work

Each step is one agent's job of one kind of work. Every code step ends with `python3 tools/verify_work.py --since <start>` and the from-design, parity and boundary checks it names; each fix has its failing test written first.

| step | kind | tier | size | job | items |
|---|---|---|---|---|---|
| S11.1 | decide | deep | S | The owner's answers to the 7 questions in section 5 (or the stated defaults), and the verdict-with-confidence method for A1 written as a short method note in the PR description. | A1, A5, F1, F5, F10, O12, TS11, Y4, M5 |
| S11.2 | decide | deep | M | The 15 decision stubs as nodes of the design (a developer revision with its sources, unsigned): current choice, options with evidence from the audits, owner team. | the 15 decisions |
| S11.3 | code | standard | M | TS11 first: the golden table (results/golden.json), the tool that re-flies all 48 scenarios and compares, the CI job. Every later step then shows what it moved. | TS11 |
| S11.4 | code | standard | L | The flight runtime's tick and the failing-test rig: gyro_fail, reset, clock-step and jitter faults (engine, twin, C and Rust tick tests), F3 integer ticks, F1 monotonic bounded clock and carried epoch, F10 claim. | TS4, F3, F1, F10/G8 |
| S11.5 | code | standard | M | The gyro chain, design and tick together (fsw_modes, fsw_estimation, steplaws): F2/G3, F5, G4. | F2/G3, F5, G4 |
| S11.6 | code | standard | M | The estimator start, allocation and gains: F4, G1 (fsw_estimation), G2 (fsw_allocation), G6 with its step-response test (fswrw, fswmtq). | F4, G1, G2, G6, TS3 |
| S11.7 | code | standard | M | The device models: M1, M5, M2 (act_rotor_set), M9, M10 (sens and run schedule), engine and twin. | M1, M2, M5, M9, M10 |
| S11.8 | code | standard | M | Power and sizing: Y2, Y4, Y5 (sizerotor, sizedemand), then Y1 (sensor power per mode in the run and in the static budget). | Y1, Y2, Y4, Y5 |
| S11.9 | code | standard | S | A5 inertia dispersion in both draws (engine and twin). | A5 |
| S11.10 | code | standard | M | The verdicts: A7 windows (kpi), A1 evaluate and trace on one verdict function, A3 cause table, A2 ledger text and the 100-draw two-sample run (as a command; the long run is S11.16). | A7, A1, A3, A2 |
| S11.11 | transcribe | standard | M | E6: m3_3 and gd_1 take the DTM2020 bundle (low, mean, high) from env, spec text off NRLMSIS, with a vector case; E1: the parity tool compares field, Sun and nadir directions. | E6, E1 |
| S11.12 | code | standard | S | P5: cap the orbit step at 30 s, refusal test. | P5 |
| S11.13 | code | standard | M | Soft OILS hardening, first half: O2, O12 (check in CI), O19, O20, O21. | O2, O12, O19, O20, O21 |
| S11.14 | code | standard | M | O11: bus-level fault injection for soft OILS (NACK, CRC, stuck register, delay, brownout). | O11 |
| S11.15 | code | standard | S | TS17: replace the weak oracles; audit every test for the same pattern and list what was changed. | TS17 |
| S11.16 | run | light | M | Regenerate (pcode.py gen, engine_build.py gen), the parity gates (twin-parity, fsw-parity), reseal the flight images, re-fly the cases that moved, and every twin campaign (all 8, 184 runs: not flown on the S7 twin, docs/S7_REPORT.md section 4; a long twin re-fly goes to a watch step), the 100-draw A2 campaign, refresh the golden table. | all |
| S11.17 | chore | light | S | Counts and tables: results/HEALTH.md, the ledger, ENGINE_PARITY and ENGINE_CAMPAIGNS regenerated, the zip 1.x corrected conversion for the drive. | all |
| S11.18 | decide | deep | S | The phase review: every fix has its failing test, source and comparison on its revision; the owner's list for S20. | all |

Step S11.4 is the largest (size L) and could be split into the rig, then F3, then F1, if one agent cannot hold the three tick copies at once.

## 7 · What this page did not do

- It changed no code, design or test. It ran no scenario, and did not run the full test suite; the open/fixed calls rest on the files and lines named, and on one compile of `fsw/src/adcs_fsw.c` with `-DADCS_REAL_FLOAT` (F10).
- E1 is called partly fixed from reading `+orbit/env.m:9` and the generated frames; a B-direction column in the parity tool will confirm the 0.23 deg gap is closed.
- Sizes are the roadmap's, raised where the code showed three tick copies (F1, F3, F2/G3, G1) or a rewritten survey (Y1, Y5).
