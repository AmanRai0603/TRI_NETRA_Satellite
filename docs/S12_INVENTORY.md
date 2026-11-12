# S12 inventory: every environment, frame, requirement and orbit item, where it lives, what fails before it

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.

This is S12 step 1 (`docs/PLAN_2_0.md`, S12 Environment, frames, requirements, orbit (U1)). It lists every item S12 makes, read from the code at `914210de` and from the regression copy (`tests/regression/design.tndb`). Nothing was changed except this page and `results/s12_inventory.json`, which holds the same content for tools.

The item ids come from:
- `docs/TECHNICAL_ROADMAP.md` (E, T, Y, V);
- `docs/ORBIT_PROPAGATOR_AUDIT.md` (P);
- `docs/TEST_STRATEGY.md` (TS1 and the environment scenarios);
- `docs/ADCS_GAPS.md` (D14, V5).

Each claim names the file and line that shows it. Line numbers are those of this commit.

## Answer first

- **44 items to make.** 35 are open and 8 partly done (P6, Y18, V1, and five of TS1's seven oracles). One, RK78 nodes in the loop, is outside S12's list; the default is to drop it.
- **The done-when does not hold today.**
  - Every environment model names its source, but only 4 of 17 have a published-vector case, 3 of them only in part (section 2a).
  - Of the 39 physics relations, 4 have a sourced vector (7 vectors in all). The docs' "7 of 39" counted the vectors as relations.
  - Requirements are not all stated or refused. The sizing fills six blanks with values of its own (Y14), and one campaign draws a tip-off range no one stated (Y15).
  - Most of the two cases' requirement values are the developer's reference, marked UNCONFIRMED (section 3).
- **Where the items live:** 25 in both halves, 12 in code, 7 in the design.
  - Most environment fixes are both. The model is a design method of env, but the choice of which model the loop flies (the Earth-frame build, the index source, the facets) is still stated in code: `adcs-sim/src/run.rs:29-38`, `adcs-pop/src/accel.rs:175`. S12 moves each choice into a stated env node.
- **By kind and tier** (`.claude/model_policy.toml`):
  - 38 code (standard tier);
  - 5 transcribe (standard tier): IGRF-14, T4, P7, V1, PV;
  - 1 decide (deep tier).
  - Each step also has light-tier regeneration and parity runs.
- **24 owner decisions, each with a default** (section 5). S12.1 states each one as the developer's proposal in its record, unsigned, as one node or one switch. Work can start without an answer.
  - The four that change the most:
    - #1, IGRF-14;
    - #4, the Earth frame on build A, with onboard nutation and EOP;
    - #18 and #24: blank requirements refused by name, and the developer's reference values refused as requirements. These block most closures until you state them, which is what the done-when asks.
- **Data, from what this environment can reach:**
  - Only package registries are open. ICGEM, CelesTrak, IERS and NOAA are not.
  - Reachable through packages:
    - IGRF-14 in two independent copies: `ppigrf` 2.1.0 `IGRF14.shc` and `pyIGRF14` 1.0.4 `igrf14coeffs.txt`;
    - the IERS `finals2000A.all` and `Leap_Second.dat`, from `astropy-iers-data` 0.2026.11.9;
    - CelesTrak SW-All from 1957 to 2041, with the predictions and the May 2024 storm, from `spaceweather` 0.4.2;
    - the NRLMSIS test references, from `pymsis` 0.13.0.
  - **EGM2008 cannot be reached.** T4 and P7 (S12.8) wait for the host `icgem.gfz-potsdam.de`. The environment change is proposed when that step is reached.
- **Proposed order: 18 steps** (section 6), in this order:
  1. the decisions;
  2. the data;
  3. the analytic oracles, first, in parallel with the data;
  4. the truth's Earth frame, then onboard frames and aberration;
  5. IGRF-14;
  6. the field error and interpolation;
  7. gravity;
  8. space weather;
  9. drag and geometry;
  10. the propagator's code fixes;
  11. requirements, then mass;
  12. the references and the done-when check;
  13. the environment scenarios;
  14. the re-fly and the review.


## 1 · Summary table

| id | group | status | lives | kind | tier | size | step |
| --- | --- | --- | --- | --- | --- | --- | --- |
| E2 | frames | open | both | code | standard | M | S12.5 |
| T5/E3 | frames | open | both | code | standard | M | S12.4 (truth), S12.5 (onboard) |
| E4 | frames | open | both | code | standard | S | S12.4 |
| P2 | propagator | open | both | code | standard | M | S12.4 |
| P8c | propagator | open | code | code | standard | S | S12.4 |
| P9 | propagator | open | both | code | standard | S | S12.4 |
| E14 | environment details | open | both | code | standard | S | S12.4 |
| IGRF-14 | fields | open | both | transcribe | standard | S | S12.6 |
| E10 | fields | open | design | code | standard | S | S12.6 |
| E5 | fields | open | both | code | standard | M | S12.7 |
| E8 | environment details | open | both | code | standard | S | S12.7 |
| T4 | fields | open | both | transcribe | standard | S | S12.8 |
| P7 | propagator | open | both | transcribe | standard | S | S12.8 |
| P1 | propagator | open | both | code | standard | M | S12.9 |
| P3 | propagator | open | both | code | standard | S | S12.9 |
| P6 | propagator | partly fixed | both | code | standard | S | S12.9 |
| T6/E9 | environment details | open | both | code | standard | S | S12.9 |
| P10 | propagator | open | both | code | standard | S | S12.10 |
| E15 | environment details | open | both | code | standard | M | S12.10 |
| E11 | environment details | open | design | code | standard | S | S12.10 |
| E12 | environment details | open | design | code | standard | S | S12.10 |
| P12 | propagator | open | design | code | standard | M | S12.10 |
| E13 | environment details | open | design | code | standard | S | S12.10 |
| P4 | propagator | open | code | code | standard | S | S12.3 |
| P8a | propagator | open | code | code | standard | S | S12.11 |
| P8b | propagator | open | code | code | standard | S | S12.11 |
| RK78 | propagator | not in scope | code | decide | deep | S | S12.1 |
| V1 | references | partly fixed | both | transcribe | standard | M | S12.14 |
| PV | references | open | design | transcribe | standard | L | S12.14 |
| Y14 | requirements | open | both | code | standard | M | S12.12 |
| Y15 | requirements | open | both | code | standard | S | S12.12 |
| Y17 | requirements | open | both | code | standard | S | S12.12 |
| Y18 | requirements | partly fixed | design | code | standard | S | S12.12 |
| Y3 | sizing | open | both | code | standard | M | S12.13 |
| TS1(a) | tests | partly fixed | code | code | standard | S | S12.3 |
| TS1(b) | tests | open | code | code | standard | S | S12.3 |
| TS1(c) | tests | partly fixed | code | code | standard | S | S12.3 |
| TS1(d) | tests | open | code | code | standard | S | S12.3 |
| TS1(e) | tests | partly fixed | code | code | standard | S | S12.3 |
| TS1(f) | tests | partly fixed | code | code | standard | S | S12.6 |
| TS1(g) | tests | partly fixed | code | code | standard | S | S12.3 |
| ENV-beta | tests | open | both | code | standard | M | S12.15 |
| ENV-storm | tests | open | both | code | standard | M | S12.15 |
| ENV-season | tests | open | both | code | standard | M | S12.15 |

Kinds are those of `.claude/model_policy.toml`. A fix made in both halves (a design revision and the code in the same commit) is `code`; an item whose substance is a cited source and its published-vector case is `transcribe`.

## 2 · The items, one by one

Each entry gives:
- what the item says, and its source;
- whether it is open, and how that is known;
- where it lives;
- the test that fails before the fix;
- its kind, size and step, its dependencies, and the owner decision it needs (section 5).

### frames

**E2** (TECHNICAL_ROADMAP E2 (High)). No aberration of starlight, truth or onboard: annual 20.5" + orbital 5.2" = about 0.0071 deg, above the 0.005 deg imaging AKE; invisible because the truth omits it too.
- Status: **open**. Known from: no 'aberr' in the code or design outside the roadmap; gen/sttracker.rs:398 st_sample takes no velocity; star directions are the J2000 catalogue through the true mount (S11.7/sttracker.pc:9-11).
- Lives in: both. Design: sens_star_tracker (S11.7/sttracker.pc st_sample, st_quest_head; S7.10/strender.pc); onboard fsw_estimation (fsw/pseudocode/03_estimation.pc). Code: adcs-sim-core/src/gen/sttracker.rs, gen/strender.rs; fsw/alg estimation.c, fsw-rs/src/alg/estimation.rs (generated).
- Fails before the fix: truth tracker at 7.6 km/s orbital + 29.8 km/s Earth velocity: the reported star directions differ from the geometric ones by v/c (up to 1.24e-4 rad); today 0. Onboard, the corrected residual below 0.0005 deg.
- Work: code (standard tier), size M, step S12.5. Depends on: T5/E3 (the velocities' frame); the Earth velocity from the Sun model. Owner decision: #5: onboard correction from GNSS velocity plus the Sun model's derivative (default), or an uplinked Earth velocity.

**T5/E3** (TECHNICAL_ROADMAP T5/E3 (High)). Nutation, the equation of the equinoxes, UT1-UTC and polar motion omitted on both sides: up to about 25" (0.007 deg) between the onboard Earth frame and the tracker's J2000.
- Status: **open**. Known from: S7.3/caltime.pc:4-5,75 'UT1 = UTC', 'nutation and polar motion left out'; the IAU 2006 kernel (env_iau2006) exists but the loop builds Gmst (adcs-sim/src/run.rs:33).
- Lives in: both. Design: env_calendar_time (S7.3/caltime.pc), env_time_frames (fsw/pseudocode/02_time_frames_models.pc); present and unused: env_iau2006, env_eop (S7.19b/eop.pc), env_tidal_eop. Code: adcs-sim/src/run.rs:33,58-68; fsw/src/adcs_env.c:10-16; fsw-rs/src/env.rs; matlab_sils/+asils/+models/+frames/eci2ecef.m.
- Fails before the fix: at 2027-01-01 the engine's truth Earth frame against ERFA c2t06a with finals2000A: about 25" today, below 1 mas after; onboard (truncated nutation, dUT1 and pole by parameter) below 1".
- Work: code (standard tier), size M, step S12.4 (truth), S12.5 (onboard). Depends on: EOP data in the design (S12.2); P2, P8c, P9. Owner decision: #4: onboard IAU 2000B nutation with dUT1, xp, yp as flight parameters set by telecommand (default); truth on build A.

**E4** (TECHNICAL_ROADMAP E4 (Medium); ORBIT_PROPAGATOR_AUDIT P2). POP's gravity and drag use an Earth frame without precession while the field uses precession: J2 axis 0.15 deg off the 2027 pole, about 2 % SSO node-rate error; a flight comment gives 0.37 deg.
- Status: **open**. Known from: run.rs:33 Build::Gmst; run.rs:58-60 precession for the field only; S7.3/earthframes.pc:25-33 C = R3(gmst); fsw/pseudocode/02_time_frames_models.md:21-22 the comment.
- Lives in: both. Design: env_earth_frames (S7.3/earthframes.pc); env_orbit_fast (S7.6/orbitfast.pc: zonals about J2000 z); the build choice is code today (revisions_2_0.toml:348). Code: adcs-sim/src/run.rs:33,58-60; adcs-pop/src/frames/mod.rs:143-221.
- Fails before the fix: angle between POP's ITRF z axis in ECI and the precessed pole at 2027.0: 0.15 deg today, below 0.001 deg after; one day's position moves 2.29 km (audit section 2).
- Work: code (standard tier), size S, step S12.4. Depends on: P2 (the same build); P8c. Owner decision: none.

### propagator

**P2** (ORBIT_PROPAGATOR_AUDIT P2 (Medium)). The loop builds POP on Gmst: the Earth frame 1,259-1,360" off, the J2 pole 0.15 deg, 2.3 km a day; use build A with the shipped XYS06 and EOP.
- Status: **open**. Known from: adcs-sim/src/run.rs:33 Build::Gmst; twin +asils/+orbit/world.m:36-38 and accel.m:16 eci2ecef_gmst.
- Lives in: both. Design: env_earth_frames, env_iau2006, env_eop hold the models; the build is a model switch stated in code (S7 inventory: model switches, S12): a stated env node chooses it. Code: run.rs:33 and :59 (the field's precession would double-count once C is GCRF to ITRF); frames/mod.rs:143-215; twin context.m:4.
- Fails before the fix: the loop's C within 1e-9 rad of try_eci2ecef(Build::A) (1,259" off at 2025-01-10 today); B in ECI equals IGRF at the ITRF position within 1 nT.
- Work: code (standard tier), size M, step S12.4. Depends on: P9, P8c, the EOP snapshot (S12.2). Owner decision: #4 (build A, linear EOP) and #3 (a frozen EOP snapshot with an expiry).

**P8c** (ORBIT_PROPAGATOR_AUDIT P8 (Low), third fix). earth_rate_eci returns [0, 0, omega] on any frame error, silently.
- Status: **open**. Known from: adcs-pop/src/frames/mod.rs:215 r.unwrap_or([0,0,omega_earth()]); used by World::new (accel.rs:130).
- Lives in: code. Design: none (earth_rate_from is design, earthframes.pc:37; the fallback is the enabler's). Code: frames/mod.rs:201-216; accel.rs:129-140.
- Fails before the fix: World::new with Build::A and no EOP gives Err(NoEop), not [0,0,omega].
- Work: code (standard tier), size S, step S12.4. Depends on: before P2. Owner decision: none.

**P9** (ORBIT_PROPAGATOR_AUDIT P9 (Low)). Leap seconds through 2017 with no expiry guard; the EOP extrapolation flag never shown.
- Status: **open**. Known from: gen/leapsec.rs (28 rows to 2017); S7.3/timescales.pc:108-115 tai_minus_utc has no validity end; caltime.pc:8 assumes 37 s to 2035; FrameInfo.eop_flag (frames/mod.rs:130,159) never read (adcs-sim rec.rs:82).
- Lives in: both. Design: env leap-second table with its validity end (from Leap_Second.dat's expiry), tai_minus_utc returning a status past it. Code: adcs-pop/src/eop.rs (the file's 'expires' header); adcs-sim rec.rs (the header records eop_flag).
- Fails before the fix: tai_minus_utc at 2036-01-01 gives 37 silently today; after, 'beyond validity'; a build-A run past finals records eop_flag = 1.
- Work: code (standard tier), size S, step S12.4. Depends on: the leap-second file (S12.2). Owner decision: #3: past the expiry, warn and record in a run, refuse in a published-vector case.

**P7** (ORBIT_PROPAGATOR_AUDIT P7 (Low-medium)). Tesserals to degree 4 move a day's position by 12 km: irrelevant to attitude, relevant to the ground track (quantifies T4).
- Status: **open**. Known from: S7.3d/gravity.pc:52-63 default_field J2-J6; S7.6/forcemodel.pc:55 grav_degree 6.
- Lives in: both. Design: as T4. Code: as T4.
- Fails before the fix: as T4, and a day's position with degree 20 differs from the zonal field by about 12 km.
- Work: transcribe (standard tier), size S, step S12.8. Depends on: T4 (one fix). Owner decision: #7.

**P1** (ORBIT_PROPAGATOR_AUDIT P1 (Medium)). World feeds drag only the manual indices: JB2008, research DTM2020 and dated DTM2020 are unreachable from the loop (blocks T6/E9).
- Status: **open**. Known from: adcs-pop/src/accel.rs:175 SwSources { manual, ..Default } (the audit's :140 is stale); World holds only sw_manual (:111); twin world.m:16 manual.
- Lives in: both. Design: env_space_weather (S7.3b/swindex.pc); a stated env node choosing the index source (manual, dated table, JB2008), like env_f107_default. Code: accel.rs:102-113,129,175; drag.rs:218-229,320-331; adcs-sim/src/run.rs:33-37; twin world.m, accel.m.
- Fails before the fix: a World with JB2008 drag returns Err(MissingDriver("jb_idx")) today (drag.rs:324); after, a finite density equal to atmos_drag_golden; dated DTM2020 on 2025-01-10 about 0.50 of the 130/Kp 2 density.
- Work: code (standard tier), size M, step S12.9. Depends on: the dated index table in the design (S12.2). Owner decision: #8: manual indices stay the default, dated by a stated node.

**P3** (ORBIT_PROPAGATOR_AUDIT P3 (Medium)). On the dated path DTM2020 gets the daily-mean Kp in both the 3-h-delayed and 24-h-mean slots, and same-day F10.7, so storm onsets are smoothed by up to a day.
- Status: **open**. Known from: adcs-pop/src/spaceweather.rs:232 KpIn::Scalar(daily mean); atmos/dtm2020.rs:52 Scalar -> [k,0,k,0]; spaceweather.rs:196 lag_f107 false.
- Lives in: both. Design: env_space_weather (swindex.pc:104 from_manual builds akp; the table branch's selection rule moves to the design). Code: spaceweather.rs:193-243 (a reader holding a selection rule).
- Fails before the fix: synthetic day, daily Kp 3, 3-hourly ap [7 x4, 207 x4]: at 15:30 UTC akp = [3,0,3,0] today; after, akp[0] = kp of the 12-15 h ap (about 8), akp[2] the 24-h mean, F10.7 the previous day's.
- Work: code (standard tier), size S, step S12.9. Depends on: P1. Owner decision: #9: F10.7 lagged 24 h per the DTM2020 specification (default).

**P6** (ORBIT_PROPAGATOR_AUDIT P6 (Medium); S11_REPORT section 5 (the sizing survey's high activity)). Fixed F10.7 halves density on an active day; the survey's F10.7 = 250 keeps Kp = 2. S11 found the survey's high activity inert on the fast orbit (its density ignores F10.7).
- Status: **partly fixed**. Known from: S11.11 made m3_3 the DTM2020 bundle (atmosbundle.pc:13-26, still Kp 2); S11.8/sizedemand.pc:53-58 sweeps F10.7 only; adcs-design/src/lib.rs:199-203 sets f107/f107a only; on POP the survey moves 4 % (tau_peak 4.105e-7 at 65, 4.273e-7 at 250), on the fast orbit 0 %.
- Lives in: both. Design: survey_sweep (sizedemand.pc: Kp/Ap per activity), bundle_activity (atmosbundle.pc), env_f107_default/env_kp_default/env_ap_default: one stated activity table used by all three. Code: adcs-design/src/lib.rs:196-210; adcs-sim/src/config.rs:744.
- Fails before the fix: the survey's high entry carries the standard's high Ap, not Kp 2; its aero tau_peak rises by the DTM2020 density ratio; on the fast orbit the high and low entries differ (equal today).
- Work: code (standard tier), size S, step S12.9. Depends on: #2 (the activity table); P1. Owner decision: #2: one activity table (low, moderate, high: F10.7 and Ap) from ECSS-E-ST-10-04C, and the survey always on DTM2020.

**P10** (ORBIT_PROPAGATOR_AUDIT P10 (Low)). Orbit drag uses a fixed cannonball while the attitude torques use facets; set_attitude has no effect.
- Status: **open**. Known from: adcs-sim/src/run.rs:35 srp_facets empty, drag_facets None; S7.6/forcemodel.pc:62,69 cannonball; set_attitude sets r_bi only (accel.rs:313).
- Lives in: both. Design: sils_forces drag_model (forcemodel.pc:52-73) on the facet set of env (S7.4/facets.pc). Code: run.rs:33-37; accel.rs:96-100; twin accel.m:43.
- Fails before the fix: 3U 0.34 x 0.1 x 0.1 m at 550 km: the orbit's drag is identical for X-ram and Z-ram today; after, their ratio is 3.4.
- Work: code (standard tier), size S, step S12.10. Depends on: none. Owner decision: #10: the torques' facets with the case's Cd (default), so orbit and torque agree.

**P12** (ORBIT_PROPAGATOR_AUDIT P12 (Low)). The SSO start uses osculating elements: about 1 deg of LTAN a year.
- Status: **open**. Known from: S7.6/forcemodel.pc:145-154 sso_initial; S7.6/orbitstart.pc the same; twin init.m:27.
- Lives in: design. Design: sso_initial (forcemodel.pc), env_orbit_start (orbitstart.pc): mean to osculating. Code: generated only.
- Fails before the fix: POP J2-J6, 550 km, 30 days: node drift against the mean Sun 0.0027 deg/day today, below 0.0003 deg/day after.
- Work: code (standard tier), size M, step S12.10. Depends on: P4, TS1(d). Owner decision: #13: Eckstein-Ustinov (J2, near-circular) (default).

**P4** (ORBIT_PROPAGATOR_AUDIT P4 (Medium)). None of the six integrators (or the Hermite output) has a test.
- Status: **open**. Known from: no integ::, rk78, gauss_jackson8 or hermite in adcs-pop/tests; integ.rs has no test module.
- Lives in: code. Design: none: integ.rs is the toolbox (boundary.toml:207). Code: adcs-pop/src/integ.rs:27-372; a new adcs-pop/tests/integ.rs.
- Fails before the fix: two-body e 0.1, a 7000 km, one period: RK78 within 1 mm of the closed form; RK4 error ratio about 16 per halving; GJ8 against RK78 within 1 cm on a J2 day; energy drift below 1e-10 over 10 orbits. A B8 coefficient changed by 1e-6 fails them.
- Work: code (standard tier), size S, step S12.3. Depends on: none. Owner decision: none.

**P8a** (ORBIT_PROPAGATOR_AUDIT P8 (Low), first fix). World::accel clones the whole Forces on every evaluation.
- Status: **open**. Known from: adcs-pop/src/accel.rs:165 let f = self.forces.clone(); srp_sc clones the facets (:143).
- Lives in: code. Design: none. Code: accel.rs:142-144,161-213.
- Fails before the fix: a counting allocator: World::accel with relativity and ERP on allocates at least 2 times per call today, 0 after; results bit-identical.
- Work: code (standard tier), size S, step S12.11. Depends on: none. Owner decision: none.

**P8b** (ORBIT_PROPAGATOR_AUDIT P8 (Low), second fix). integ::hermite scans every node on each query: dense output O(n^2).
- Status: **open**. Known from: adcs-pop/src/integ.rs:363 for k in 0..n.
- Lives in: code. Design: none (toolbox). Code: integ.rs:359-372.
- Fails before the fix: 1e5 nodes and 1e5 ascending queries: seconds today, milliseconds with a search or cursor; bit-identical results.
- Work: code (standard tier), size S, step S12.11. Depends on: P4. Owner decision: none.

**RK78** (ORBIT_PROPAGATOR_AUDIT P5 'or RK78 nodes'; S11_INVENTORY ('RK78 is S12')). P5 was fixed in S11 by the 30 s cap; the RK78-node alternative was passed to S12 but no S12 row carries it.
- Status: **not in scope**. Known from: adcs-sim config.rs:667 step within 1e-3..30 s (S11.12); PLAN_2_0 S12 lists P1-P4, P6-P10, P12.
- Lives in: code. Design: none. Code: accel.rs:250-267 InLoop::advance (RK4).
- Fails before the fix: none unless adopted
- Work: decide (deep tier), size S, step S12.1. Depends on: P4. Owner decision: #15: drop it (default): RK4 at 10 s is 0.43 m a day, far below attitude relevance.

### environment details

**E14** (TECHNICAL_ROADMAP E10-E15 (Low), the fifth). No epoch range guard in the engine or the flight software, though the leap-second, IGRF and EOP data have limited spans.
- Status: **open**. Known from: fsw/src/adcs_params.c:361 checks only jd0 finite and positive; S7.11/caseorbit.pc mission_epoch takes any year; igrf_gh clamps and extrapolates (02_time_frames_models.pc:102-107).
- Lives in: both. Design: env_case_orbit (S7.11/caseorbit.pc): a stated valid span; the flight parameter rule (fsw/params/params.toml). Code: fsw/src/adcs_params.c:361, fsw-rs params; adcs-sim/src/config.rs:707.
- Fails before the fix: mission.epoch = 2036.5 refused by name by the engine; a flight blob with jd0 = 2466000 (2039) fails validate. Today both are accepted.
- Work: code (standard tier), size S, step S12.4. Depends on: IGRF-14 (its span to 2030); P9. Owner decision: #3: the span 2025.0-2030.0 intersected with the leap-second file's expiry.

**E8** (TECHNICAL_ROADMAP E8 (Low)). Environment held 1 s: B turns up to 0.21 deg/s, hidden only because the magnetometer samples on the refresh tick.
- Status: **open**. Known from: adcs-sim/src/run.rs:538-541 refresh when k % rt.env == 0, else hold; env_refresh_step 1 s (revisions_2_0.toml:2369).
- Lives in: both. Design: env_refresh_step; a stated interpolation rule (dyn's truth step, S11.9/truthplant.pc). Code: adcs-sim/src/run.rs:286,537-541; the twin's asils.run.
- Fails before the fix: dt 0.1 s, refresh 1 s, 550 km: held B against the field at t = k + 0.9 s up to about 0.19 deg today, below 0.005 deg interpolated; a magnetometer sampled 0.05 s off the tick shows the step today.
- Work: code (standard tier), size S, step S12.7. Depends on: none. Owner decision: none (linear interpolation of B in ECI; the Sun and density are slow).

**T6/E9** (TECHNICAL_ROADMAP T6/E9 (Low); DD44). Space weather constant in the loop; 'high activity' keeps Kp = 2; no storm case.
- Status: **open**. Known from: adcs-sim/src/run.rs:37 ManualIndices (130, Kp 2); atmosbundle.pc:13-17 'the high activity's Ap is S12's'; the parsers exist, no index file ships.
- Lives in: both. Design: env_f107_default, env_f107a_default, env_kp_default, env_ap_default (revisions_2_0.toml:1985-2015); env_space_weather; m3_3. Code: run.rs:37; accel.rs:175; config.rs:670-672.
- Fails before the fix: a dated run on 2024-05-11 (Kp 8-9, Ap 236-400): DTM2020 density at 550 km at least 3 times the Kp 2 value (equal today); the high bundle carries the standard's Ap.
- Work: code (standard tier), size S, step S12.9. Depends on: P1, P3, P6; the dated table (S12.2). Owner decision: #8: the dated table from CelesTrak SW-All; one historic storm (10-11 May 2024) as a scenario.

**E15** (TECHNICAL_ROADMAP E10-E15 (Low), the sixth). No thermospheric wind: the relative velocity is co-rotation only.
- Status: **open**. Known from: adcs-sim/src/run.rs:64,70 corotating velocity; S7.6/orbitfast.pc:34; POP's drag wind hook (drag.rs:32-35) zero in the loop.
- Lives in: both. Design: env_drag_force rel_velocity (S7.6/drag.pc); env_orbit_fast; a wind model node. Code: run.rs:64,70; adcs-pop/src/drag.rs:32-71.
- Fails before the fix: v_rel at 550 km, 70 deg latitude, Kp 6 differs from co-rotation by at least 50 m/s with the wind on (0 today).
- Work: code (standard tier), size M, step S12.10. Depends on: T6/E9. Owner decision: #11: a wind of stated magnitude by latitude and Kp, truth only, dispersed (default; HWM14's coefficient file is not in the repository).

**E11** (TECHNICAL_ROADMAP E10-E15 (Low), the second; ORBIT_PROPAGATOR_AUDIT). Eclipse on a spherical Earth; POP's 'fine' model (oblate Earth plus 12 km atmosphere) exists.
- Status: **open**. Known from: S7.4/shadow.pc:1-5 conical on a 6378137 m sphere; run.rs:63,69; Fine in S11.8c/srp.pc:111; onboard cylindrical (DD43).
- Lives in: design. Design: l3_dist_row_10 (shadow.pc), env_srp_force (srp.pc), m2_7. Code: generated only (gen/shadow.rs); run.rs calls it.
- Fails before the fix: 550 km at beta 0: entry and exit from shadow_fraction against POP Fine within 0.5 s (seconds apart today).
- Work: code (standard tier), size S, step S12.10. Depends on: none. Owner decision: none.

**E12** (TECHNICAL_ROADMAP E10-E15 (Low), the third). LTAN taken from the apparent Sun (the equation of time: up to 4 deg in RAAN).
- Status: **open**. Known from: S7.6/orbitstart.pc:10-17 raan = atan2(sun) + (ltan-12) 15 deg; forcemodel.pc:142-146 the DE440 Sun.
- Lives in: design. Design: env_orbit_start (orbitstart.pc), env_force_model sso_initial (forcemodel.pc). Code: generated only; accel.rs:318-323 and run.rs:37,43 call it.
- Fails before the fix: at 2027-02-11 and 2027-11-03 the RAAN minus the mean Sun's RA - (12 - LTAN) 15 deg below 0.01 deg (3.5 and 4.1 deg today).
- Work: code (standard tier), size S, step S12.10. Depends on: P12. Owner decision: #12: LTAN is mean solar time (default, the launch-provider convention).

**E13** (TECHNICAL_ROADMAP E10-E15 (Low), the fourth; S11_INVENTORY section 3). Albedo is a point at nadir, with constants differing from their comments (0.30 flown against 0.31 cited, 237 against 235 W/m2).
- Status: **open**. Known from: S7.4/albedo.pc:4-12, S7.4/earthir.pc:4-10; POP's Knocke rings (S7.6/erp.pc) unused for torques.
- Lives in: design. Design: l3_dist_row_04, l3_dist_row_05; env_erp_force (rings). Code: generated only (gen/albedo.rs, gen/earthir.rs, torques.rs calls).
- Fails before the fix: each constant equals its cited source (fails today); the point-source albedo pressure at 550 km, Sun zenith 60 deg, within 10 % of the Knocke rings.
- Work: code (standard tier), size S, step S12.10. Depends on: none. Owner decision: #14: the torques on the Knocke 1988 rings with Knocke's constants (default).

### fields

**IGRF-14** (TECHNICAL_ROADMAP D14/V5 (Medium); ADCS_GAPS D14, V5; DESIGN_DECISIONS DD42). The field is IGRF-13 (2025.0 is 2020.0 plus five years of secular variation), extrapolated to the 2027 epoch; one IGRF-14 table for the engine, both flight softwares and the twin.
- Status: **open**. Known from: env_igrf13_coefficients (revisions_2_0.toml:145-153, reader igrf13 in tools/readers.py:54); generated gen/igrf13.rs, fsw/alg/src/igrf13.c, fsw-rs/src/alg/igrf13.rs; no IGRF-14 file in the repository; a second copy of g10, g11, h11 in design/groups/env.pc:90-96.
- Lives in: both. Design: env_igrf13_coefficients (renamed to IGRF-14, its source the IAGA file); igrf_gh (fsw/pseudocode/02_time_frames_models.pc:95-108); env.pc dipole constants. Code: generated tables in 4 crates and the twin (matlab_sils/data/igrf13.json, +asils/+alg/+igrf13); tools/readers.py.
- Fails before the fix: field at 2027.0, 45N 105E 500 km against an independent IGRF-14 implementation within 0.01 nT (tens of nT apart today); g10 at 2025.0 equals igrf14coeffs.txt (-29350.0; today -29376.3); every generated table hashes equal.
- Work: transcribe (standard tier), size S, step S12.6. Depends on: S12.2 (the file in); E10; E14. Owner decision: #1: adopt IGRF-14, onboard degree 10, truth 13 (default); DD42 waits on the owner, the default taken unsigned.

**E10** (TECHNICAL_ROADMAP E10-E15 (Low), the first). The dipole rows use R_E = 6378.137 km instead of IGRF's reference radius a = 6371.2 km (0.33 % high).
- Status: **open**. Known from: design/groups/env.pc:104-118 b0*(R_E/r)^3 with R_E 6378137 (:68); truth and onboard synthesis use 6371.2 (02_time_frames_models.pc:115).
- Lives in: design. Design: env.pc dipole, dipole_field_equator, dipole_field_max; rows m3_0, m3_1. Code: generated only (adcs-relations/src/gen/env.rs).
- Fails before the fix: dipole_field_equator(6928.137 km) = B0 (6371.2/6928.137)^3 to 1e-12 relative; today 1.00327 times that.
- Work: code (standard tier), size S, step S12.6. Depends on: IGRF-14 (B0 from it). Owner decision: none.

**E5** (TECHNICAL_ROADMAP E5 (Medium)). Field model error not represented (no crustal or external field: 20-50 nT quiet, 100-500 nT storm, 0.1-0.9 deg), so the magnetometer measures the onboard model exactly.
- Status: **open**. Known from: S7.3/truthfield.pc:1-3 IGRF only; the only truth/onboard difference is degree 13 against 10 (env_field_degree, fsw_param_igrf_nmax).
- Lives in: both. Design: l3_dist_row_08 (truthfield.pc); a new truth-only error field in env; its dispersion in the campaign draw. Code: adcs-sim/src/run.rs:63,69; the campaign draw (adcs-sim and the twin's +asils/+campaign/draw.m).
- Fails before the fix: truth B minus onboard IGRF at zero sensor noise: a few nT today; after, RMS 20-50 nT quiet and 100-500 nT at Dst -200 nT; two seeds draw different error fields.
- Work: code (standard tier), size M, step S12.7. Depends on: IGRF-14; T6/E9 (Dst/Kp drives the external term). Owner decision: #6: the error field's source: a Dst-scaled external dipole (the ring current) plus a crustal term of stated RMS, dispersed per draw (default); a higher-degree crustal table (WMM-HR, EMM) needs its file.

**T4** (TECHNICAL_ROADMAP T4 (Medium); ORBIT_PROPAGATOR_AUDIT P7). Gravity is zonal J2-J6 only, no field file ships, comments claim tesserals; ship EGM2008 20x20.
- Status: **open**. Known from: adcs-pop/src/gravity/field.rs:52-58 nmax 6, sbar zeros; only a synthetic .gfc (tests/data/gravity_tides_synth.gfc); adcs-sim-core/src/orbit.rs:7 claims 'degree-6 field with tesserals'.
- Lives in: both. Design: env_gravity_field (S7.3d/gravity.pc default_field), a gravfield data node read from the ICGEM file; sils_forces degree (S7.6/forcemodel.pc:52-58). Code: gravity/field.rs:52-66 (the .gfc reader exists, :210); orbit.rs:7 comment.
- Fails before the fix: the shipped field to degree 20 with normalised C22 = 2.43938357e-6, S22 = -1.40027370e-6 (EGM2008); today S22 = 0; the acceleration at a point matches an ICGEM-computed value to 1e-12 relative.
- Work: transcribe (standard tier), size S, step S12.8. Depends on: E4/P2 (tesserals need the right Earth-fixed frame); the file: icgem.gfz-potsdam.de is not reachable from this environment. Owner decision: #7: 20x20 in POP, the fast orbit stays J2-J6 (default); the network host for the file.

### references

**V1** (TECHNICAL_ROADMAP section 3 U1 'V1 orbit reference ledger'; ADCS_GAPS V1; the campaign itself is U2 (S13, with P13)). The propagator is not validated outside itself: one ledger row per environment model with its outside referent, data, test and status.
- Status: **partly fixed**. Known from: docs/references.toml holds 15 rows (time_frames and igrf13 checked; orbit_propagation and atmosphere owed); adcs-pop/tests/published.rs (ERFA); fsw-rs/tests/igrf.rs (8 pyIGRF points, IGRF-13); DE440, DTM2020, JB2008, gravity, SRP, albedo, eclipse have no outside referent.
- Lives in: both. Design: each environment node's source field (present) and its published-vector case (owed). Code: docs/references.toml rows; new published tests (Vallado Example 3-15 frames, DTM2020's own test outputs, the IGRF-14 points).
- Fails before the fix: tests/test_env_sources.py: every environment model (the table in section 2a) names its source and has a published-vector case or a row 'owed' with the reason; fails today on the models listed there.
- Work: transcribe (standard tier), size M, step S12.14. Depends on: every other S12 step (it is the done-when check). Owner decision: #16: the ledger stays docs/references.toml, one row per model, each with a test (default); the precise-orbit arcs are S13's.

**PV** (TECHNICAL_ROADMAP section 3 U1 'sourced vectors for the 32 physics rows'; ADCS_GAPS D14 ('7 of 39')). Most physics relations are checked only for units, translation and range; few have a published test vector.
- Status: **open**. Known from: spec/plan/physics.toml (in the design) has 39 relations; the 7 sourced vectors (IDMAS v2) cover only 4 relations (fmr ring_momentum, spin_down_time; rcs propellant_per_slew, propellant_per_year); the other 35 have interpreter vectors only. The docs counted vectors as relations.
- Lives in: design. Design: lib_spec_physics_* and the 31 rows that name spec.physics (env, act, ctl, pnt, risk); 8 relations used by no row. Code: tools/pcode.py fixtures (provenance); adcs-relations tests.
- Fails before the fix: every relation of spec/plan/physics.toml has a fixture whose provenance is not 'interpreter' and that names its page; fails on 35 of 39 today.
- Work: transcribe (standard tier), size L, step S12.14. Depends on: IGRF-14, E10 (the dipole rows change first). Owner decision: #17: source the 31 used relations; the 8 no row uses are marked unused, not sourced (default).

### requirements

**Y14** (TECHNICAL_ROADMAP Y14 (Medium); SPEC section 8.3.2 'Blanks are never guessed' (archive/design-1.0/spec/SPEC.md:1470-1477); DD5, DD10). Blank requirements are filled with values against SPEC 8.3.2 (the invented 60 s slew sets tau_req at 100 times tau_dist); req.wmax blank while the tracker limit equals the slew peak; no temperature, cleanliness, LEOP-energy, availability or data keys.
- Status: **open**. Known from: S11.8/sizedemand.pc:228-238 demand_defaults (req.slew 60 s, req.dump 0.25 orbit, req.hsat k_h 2, mission.life 3 yr, mission.spd, sangle, w0); adcs-design/src/lib.rs:224-228 notes 'blank: ... taken' and goes on; sizing.json tau_slew 3.8e-5 against tau_dist 4.4-5.6e-7.
- Lives in: both. Design: the case requirement nodes (p1k_*, p2k_*, p3k_*, p4k_*, m1_*); l3_budget_row_01 (sizedemand.pc demand_defaults); the case template (new keys). Code: adcs-design/src/lib.rs:218-229; adcs-sim/src/config.rs:212-226 (CASE_NEEDS, the existing refusal pattern); tools/trace.py:108-110.
- Fails before the fix: demand() on ais_3u (req.slew, req.dump, mission.life blank) returns an error naming each blank (today Ok with 'req.slew blank: 60 s taken'); the case template carries req.temp_op, req.cleanliness, req.leop_energy, req.avail, req.data (absent from its 71 rows today).
- Work: code (standard tier), size M, step S12.12. Depends on: #18 (deep: which blanks refuse and which have a stated rule); Y15. Owner decision: #18: refuse every blank the sizing needs, the closure reading 'not stated' (default); #24: the developer reference values (UNCONFIRMED) refused as requirements, kept as proposals.

**Y15** (TECHNICAL_ROADMAP Y15 (Medium), A14; DD38; DR-gdn-002). Tip-off is a single 10 deg/s (an unconfirmed plan value) while deployers reach 20-30 deg/s; it should be a range, detumble sized at its top.
- Status: **open**. Known from: m1_5 = 10 'UNCONFIRMED adcs_ref_c1', lo/hi blank in both cases; sizedemand.pc:230 default w0 10; campaigns mc_detumble_ais draw 5-15 (a range no one stated); DR-gdn-002 'keep 10 deg/s'.
- Lives in: both. Design: m1_5; l3_budget_row_01 (h_detumble); DR-gdn-002. Code: adcs-design/src/lib.rs:218-225; detumble scenarios' case:mission.w0; mc_detumble_ais.json.
- Fails before the fix: ais_3u with mission.w0 lo 0, hi 30: the sizing uses 30 and the campaign draws within [lo, hi] from the case (today 10 and 5-15); a blank mission.w0 is refused, not set to 10.
- Work: code (standard tier), size S, step S12.12. Depends on: Y14 (the refusal). Owner decision: #19: the tip-off range: refused until the launch provider states it; the developer's proposal 0-30 deg/s per axis, sized at 30, to be confirmed.

**Y17** (TECHNICAL_ROADMAP Y17 (Low), with ADCS_GAPS D10; docs/SIZING_LAWS.md:26-28). The 'bang-bang' slew is cycloidal (2 pi theta/T^2, 57 % more torque than bang-bang's 4 theta/T^2): mislabelled; the doc claims a k_tau lever the code never raises.
- Status: **open**. Known from: S11.8/sizedemand.pc:8 'bang-bang', :293 a_slew = 2 pi ...; rw.pc gw_4 bang_bang_peak_torque 4 I theta/T^2; S11.8b/looprules.pc has no k_tau event.
- Lives in: both. Design: l3_budget_row_01 (sizedemand.pc), gw_3, gw_4. Code: adcs-design/src/lib.rs:88,218-221; docs/SIZING_LAWS.md:26-29 (also stale on h_req, Y4).
- Fails before the fix: theta 30 deg, T 60 s, J 0.042: the sizing's profile is named 'cycloidal' and its peak torque is pi/2 times gw_4's bang-bang (named bang-bang today); SIZING_LAWS claims no lever looprules lacks.
- Work: code (standard tier), size S, step S12.12. Depends on: Y14 (a slew is sized only when stated). Owner decision: #20: cycloidal, named so (it is the profile flown by slew_img) (default).

**Y18** (TECHNICAL_ROADMAP Y18 (Low), with ADCS_GAPS D10; SIZING_LAWS.md:66-67). The RCS propellant holds two detumbles' impulse (x2) with no reason or source, beside the unsourced x1.2 and 600 s.
- Status: **partly fixed**. Known from: S7.15/sizercs.pc:4 'holds the detumble twice', :48 it_det = 2.0 h_detumble/arm_eff; SIZING_LAWS.md:66 states it without reason.
- Lives in: design. Design: design_sizing_rcs, l3_budget_row_05 (sizercs.pc). Code: generated only (gen/sizercs.rs).
- Fails before the fix: every literal factor in rcs_size (2 detumbles, 1.2, 600 s) is a named stated value with a source or a decision record (inline today).
- Work: code (standard tier), size S, step S12.12. Depends on: none. Owner decision: #21: 2 detumbles (LEOP and one recovery), stated with that reason; 1.2 and 600 s stated as the developer's, unsourced, unsigned.

### sizing

**Y3** (TECHNICAL_ROADMAP Y3 (Critical); DD9, DD24, DD61; DR-design-003 ('mass growth is S12's')). Imaging mass 'feasible' at 1.794 of 1.85 kg is not robust: coil mass from an invented anchor (physics needs about 85 g, not 21 g), no electronics, harness or brackets, no growth allowance.
- Status: **open**. Known from: S7.15/sizemtq.pc:40 mass = 0.03 m/0.45 (the SYN-CT-1 anchor, revisions_2_0.toml:3218): 6.93 g a coil; S11.8/sizebudget.pc sums the parts only.
- Lives in: both. Design: design_sizing_mtq, gm_5, l3_budget_row_02 (sizemtq.pc); gb_0, l3_budget_row_06 (sizebudget.pc); the catalogue's maturity field. Code: adcs-design/src/lib.rs:263-285, 450-465.
- Fails before the fix: an air-core coil of side 0.09 m for 0.08 A m2 at 0.054 W weighs at least the copper bound rho_cu rho_e (4a)^2 (m/A)^2/P (about 36 g; 5.3 g today); gb_0 carries an electronics/harness line and a growth allowance above 0 by maturity (neither exists).
- Work: code (standard tier), size M, step S12.13. Depends on: #22. Owner decision: #22: AIAA S-120A growth allowance by maturity (in-house sized parts as new design), and the electronics/harness line from a cited datasheet or refused as not stated (default).

### tests

**TS1(a)** (TEST_STRATEGY TS1 (H)). torque-free axisymmetric body: nutation rate (I3-I1)/I1 w3 to 1e-6, nutation angle constant to 1e-9 rad
- Status: **partly fixed**. Known from: adcs-sim-core/tests/core.rs:67 checks momentum, energy and the quaternion norm only.
- Lives in: code. Design: none (the oracle is the published formula in the test). Code: adcs-sim-core/tests/core.rs.
- Fails before the fix: I = [0.02, 0.02, 0.05], w = [0.01, 0, 0.1]: the body rate of w_perp is 0.15 rad/s to 1e-6, the nutation angle constant to 1e-9 over 600 s
- Work: code (standard tier), size S, step S12.3. Depends on: none. Owner decision: none.

**TS1(b)** (TEST_STRATEGY TS1 (H)). gravity-gradient libration: pitch frequency n sqrt(3(Ix-Iz)/Iy) within 0.5 % (Hughes ch. 9)
- Status: **open**. Known from: core.rs:94 checks the torque at one attitude only.
- Lives in: code. Design: none (the oracle is the published formula in the test). Code: adcs-sim-core/tests/core.rs.
- Fails before the fix: free nadir body at 550 km, 0.5 deg pitch offset, control off: the fitted period within 0.5 %
- Work: code (standard tier), size S, step S12.3. Depends on: none. Owner decision: none.

**TS1(c)** (TEST_STRATEGY TS1 (H)). B-dot initial decay against Avanzini and Giulietti within 15 %
- Status: **partly fixed**. Known from: adcs-sim/tests/engine.rs:43 holds a home-derived rate in 0.55-1.05.
- Lives in: code. Design: none (the oracle is the published formula in the test). Code: adcs-sim/tests/engine.rs.
- Fails before the fix: detumble_ais: the initial decay within 15 % of the 2012 paper's prediction
- Work: code (standard tier), size S, step S12.3. Depends on: none. Owner decision: none.

**TS1(d)** (TEST_STRATEGY TS1 (H)). J2 nodal rate within 0.1 %, SSO 0.9856 deg/day within 0.5 %
- Status: **open**. Known from: no nodal-rate test; t_sso_start (run_all_tests.m:62) checks geometry.
- Lives in: code. Design: none (the oracle is the published formula in the test). Code: adcs-pop/tests (new).
- Fails before the fix: J2 only, 550 km, i 97.6 deg, a day: mean RAAN rate = -1.5 n J2 (Re/p)^2 cos i within 0.1 %
- Work: code (standard tier), size S, step S12.3. Depends on: none. Owner decision: none.

**TS1(e)** (TEST_STRATEGY TS1 (H)). eclipse durations against Vallado section 5.3 within 2 s
- Status: **partly fixed**. Known from: the twin's t_shadow is a 3-point edge sweep (TS17).
- Lives in: code. Design: none (the oracle is the published formula in the test). Code: adcs-sim-core tests (new).
- Fails before the fix: 550 km circular at beta 0: the umbra duration within 2 s of Vallado's formula
- Work: code (standard tier), size S, step S12.3. Depends on: none. Owner decision: none.

**TS1(f)** (TEST_STRATEGY TS1 (H)). 50 published IGRF points at engine and twin level
- Status: **partly fixed**. Known from: fsw-rs/tests/igrf.rs 8 pyIGRF (IGRF-13) points, flight code only; the twin's t_igrf compares with the design's own vectors.
- Lives in: code. Design: none (the oracle is the published formula in the test). Code: fsw-rs/tests/igrf.rs; engine and twin tests (new).
- Fails before the fix: 50 IGRF-14 points from two independent implementations, each within 1 nT, at engine, both flight softwares and twin
- Work: code (standard tier), size S, step S12.6. Depends on: IGRF-14. Owner decision: none.

**TS1(g)** (TEST_STRATEGY TS1 (H)). flat-plate SRP and drag torque oracles
- Status: **partly fixed**. Known from: core.rs:429 Earth IR flat plate only; the twin's t_torques checks sign and symmetry.
- Lives in: code. Design: none (the oracle is the published formula in the test). Code: adcs-sim-core/tests/core.rs.
- Fails before the fix: -X facet 0.1 x 0.34 m, centre of mass 1 cm along +Y: SRP F = pA(1 + rho_s + 2 rho_d/3) and drag F = 1/2 rho v^2 Cd A give 0.01 F about Z to 1e-12
- Work: code (standard tier), size S, step S12.3. Depends on: none. Owner decision: none.

**ENV-beta** (TEST_STRATEGY section 4 and U1 row ('environment scenarios (beta, storm, season)')). environment scenarios: beta minimum and maximum per case (TEST_STRATEGY section 4)
- Status: **open**. Known from: the 80 scenarios have no environment block (adcs-sim/src/config.rs:699-701); engine.epoch_days, ltan_h, f107, kp exist only as campaign knobs.
- Lives in: both. Design: case orbit (S7.11/caseorbit.pc), the activity table (#2), the scenario cases (revision cases). Code: adcs-sim/src/config.rs (an environment block), the twin's scenario reader, results/golden.json.
- Fails before the fix: each case (ais_3u LTAN 6, ais_img_3u LTAN 10) has scenarios at its year's beta minimum and maximum: 0 today
- Work: code (standard tier), size M, step S12.15. Depends on: a scenario 'environment' block (config.rs:699-701 accepts none). Owner decision: #23: by epoch at the case's own LTAN (default).

**ENV-storm** (TEST_STRATEGY section 4 and U1 row ('environment scenarios (beta, storm, season)')). environment scenarios: solar activity and storm (F10.7 70/150/250, Kp 0/4/9 with field error)
- Status: **open**. Known from: the 80 scenarios have no environment block (adcs-sim/src/config.rs:699-701); engine.epoch_days, ltan_h, f107, kp exist only as campaign knobs.
- Lives in: both. Design: case orbit (S7.11/caseorbit.pc), the activity table (#2), the scenario cases (revision cases). Code: adcs-sim/src/config.rs (an environment block), the twin's scenario reader, results/golden.json.
- Fails before the fix: scenarios at the activity table's three levels and a storm, with stated verdicts: 0 today; with P3 a dated storm reaches akp[0] = 9 within 3 h of onset
- Work: code (standard tier), size M, step S12.15. Depends on: P1, P3, P6, E5. Owner decision: #8 (synthetic levels first, the dated May 2024 storm after P1).

**ENV-season** (TEST_STRATEGY section 4 and U1 row ('environment scenarios (beta, storm, season)')). environment scenarios: season (solstices, equinoxes; eclipse season against none)
- Status: **open**. Known from: the 80 scenarios have no environment block (adcs-sim/src/config.rs:699-701); engine.epoch_days, ltan_h, f107, kp exist only as campaign knobs.
- Lives in: both. Design: case orbit (S7.11/caseorbit.pc), the activity table (#2), the scenario cases (revision cases). Code: adcs-sim/src/config.rs (an environment block), the twin's scenario reader, results/golden.json.
- Fails before the fix: per-case scenarios at the solstices and equinoxes; ais_3u eclipse fraction 0.223 at its epoch and 0 at +91.3 d asserted: 0 today
- Work: code (standard tier), size M, step S12.15. Depends on: the environment block. Owner decision: none.

### 2a · The environment models today: source and published vector

The done-when asks that every environment model name its source and have a published-vector case. The lines marked `## vectors: N` in the `.pc` files are vectors the interpreter draws to check the translations (`docs/PSEUDOCODE_V2.md`), not published values, and are not counted here.

| model | node or method | source named | published-vector case today | made in |
| --- | --- | --- | --- | --- |
| Field (IGRF synthesis, truth and onboard) | l3_dist_row_08 truthfield.pc; env_time_frames igrf_gh | Alken 2021 (IGRF-13), WGS-84 | partly: 8 pyIGRF points, IGRF-13, flight code only (fsw-rs/tests/igrf.rs) | S12.6 |
| Dipole relations | env.pc dipole* | Wertz 1978; cites IGRF-14, uses 13 | no (interpreter vectors) | S12.6, S12.14 |
| Static density | l3_dist_row_07 truthdensity.pc | Vallado table 8-4 | partly (the table's 500 km value, tests/test_density_bundle.py:26) | S12.14 |
| DTM2020, JB2008 | env_dtm2020_operational, env_jb2008 | Bruinsma 2021; Bowman 2008 | no (MATLAB parity only) | S12.14 |
| Sun and Moon, fast | l3_dist_row_09 sunfast.pc; env_moon_fast | Montenbruck and Gill 3.3.2 | no | S12.14 |
| Onboard Sun | env_time_frames sun_model | Vallado algorithm 29 | no | S12.14 |
| DE440 | env_de440 | Park 2021 | no (Horizons owed) | S12.14 |
| Gravity | env_gravity_field; env_orbit_fast zonals | Montenbruck and Gill 3.2; EGM2008, EGM96 | no (a two-body closed form only) | S12.8, S12.14 |
| SRP | env_srp_force; l3_dist_row_03 | Montenbruck and Gill 3.73/3.75; Wertz 17-6 | no | S12.3 (TS1 g) |
| Albedo, Earth IR | l3_dist_row_04, _05 | Knocke 1988 (point form) | no (its own formula) | S12.10 |
| Eclipse | l3_dist_row_10 shadow.pc; m2_7 | Montenbruck and Gill 3.4.2 | no (interpreter vectors; the twin's 3 points) | S12.3 (TS1 e) |
| Frames, engine and onboard | env_time_frames; env_calendar_time | Aoki 1982; Lieske 1977 | no | S12.4, S12.5 |
| Frames, POP Gmst build | env_earth_frames | yes | no | S12.4 |
| Frames, IAU 2006, EOP | env_iau2006; env_eop; env_tidal_eop | IERS TN36, SOFA | yes for iau2006 (ERFA, adcs-pop/tests/published.rs); EOP no | S12.4 |
| Time scales | env_time_scales | IERS ch. 10, Bulletin C | partly (ERFA cal2jd, dat) | S12.4 |
| Space weather | env_space_weather | Bartels 1957 | no | S12.9 |
| Drag (gas-surface) | env_gas_surface; env_drag_force | Sentman 1961; Moe 2005 | no | S12.3 (TS1 g), S12.14 |

Every model names a source; four have a published-vector case (one whole, three in part); thirteen have none. The dipole rows cite IGRF-14 and use IGRF-13's constants.

## 3 · Found on the way

- **The requirements of the two cases, by who stated them.** "owner" means you stated it. "ref" means the developer's reference, marked `UNCONFIRMED · adcs_ref_c1`. "filled" means blank, with a value the code supplies (`S11.8/sizedemand.pc:228-238`, `adcs-design/src/lib.rs:218-229`). A dash means not stated and not filled.

  | key | ais_3u | ais_img_3u |
  |---|---|---|
  | req.ape | 10 deg, owner | 0.01 deg at 99.73 %, owner |
  | req.ake | 5, ref | 0.005 at 99.73 %, ref |
  | req.rks | – | 0.005 at 99.73 %, owner |
  | req.rpe, req.pde, req.rke | – | – |
  | req.slew | filled, 60 s | 60 s, ref |
  | req.settle | – | 20 s, ref |
  | req.spo, req.track, req.wmax | – | – (the tracker's 0.017 rad/s limit equals the slew peak) |
  | req.detumble | 284 s, ref | 284 s, ref |
  | req.sunacq | 95 s, ref | 95 s, ref |
  | req.faults, req.recover | – | – |
  | req.hsat | filled, k_h = 2 | filled, k_h = 2 |
  | req.dump | filled, 0.25 orbit | filled, 0.25 orbit |
  | req.mass | 1.6 kg, owner | 1.85 kg, owner |
  | req.pavg, req.ppk | 0.5, 1.5 W, ref | 2.0, 4.0 W, ref |
  | req.vol | 1.0 U, owner | 1.0 U, owner |
  | req.prop | – | – |
  | the level of every requirement but the three stated | filled, 99.73 % | filled, 99.73 % |
  | mission.life | filled, 3 yr | filled, 3 yr |
  | mission.epoch | 2027, ref | 2027, ref |
  | mission.spd, mission.sangle | 10, 30, ref | 10, 30, ref |
  | mission.w0 | 10 deg/s, ref, no range (the detumble campaign draws 5-15) | the same |

  You stated orbit.alt and orbit.ltan; orbit.inc is derived from them. No key exists for temperature, cleanliness, LEOP energy, availability or data.
- **`docs/references.toml` is already the reference ledger V1 asks for.** It has 15 rows. Time and frames, and IGRF-13, are checked. `orbit_propagation` and `atmosphere` are owed. S12.14 adds one row per environment model of section 2a. It does not start a new ledger.
- **S11's "the survey's high activity is inert" holds only on the fast orbit.** The default sizing flies POP. There, F10.7 250 against 65 moves the aero torque peak by 4 % (4.105e-7 against 4.273e-7 N m), with Kp held at 2. On `engine.orbit = fast` it moves 0 %. Both are fixed by #2 (P6, S12.9).
- **Two flux tables disagree.** The survey sweeps F10.7 65 and 250 (`sizedemand.pc:53`). The bundle uses 70/130/250 (`S11.11/atmosbundle.pc`). TEST_STRATEGY uses 70/150/250. #2 makes them one.
- **A double count is waiting in P2.** `run.rs:59` applies precession to the field frame on top of the Earth frame `C`. Once `C` is GCRF to ITRF (build A), precession would be counted twice. S12.4 removes it in the same commit.
- **Stale pointers in the sources.**
  - P1's line is `accel.rs:175`, not `:140`.
  - The roadmap's E10-E15 is one row. Its order maps to E10 dipole radius, E11 eclipse, E12 LTAN, E13 albedo, E14 epoch guard and E15 wind. Only E11 is confirmed elsewhere (the audit); the rest is inferred from the order.
  - `docs/SIZING_LAWS.md:26-29` still gives h_req as a maximum, which S11's Y4 made a sum.
- **A flight comment disagrees with the truth.** `fsw/pseudocode/02_time_frames_models.md:21-22` gives the omitted precession as 0.37 deg. The audit measures 0.15 deg on the J2 pole at 2027. E4 corrects the comment.
- **The dipole's constants are kept twice.** `design/groups/env.pc:90-96` holds g10, g11 and h11 beside the IGRF table. IGRF-14 (S12.6) makes the dipole read the table.

## 4 · Dependencies that fix the order

- **The decisions come first** (S12.1). Steps 2-15 take their defaults from them, and each default is one node or one switch.
- **The data comes before the models that read it** (S12.2): IGRF-14, EOP and leap seconds, and space weather. Two package copies must agree before a table enters the design. EGM2008 is the exception and waits for its host, so S12.8 can slip without holding up anything but P7.
- **The analytic oracles come before the fixes they measure** (S12.3: TS1 (a)-(e), (g), and P4). They are code only, so they run in a worktree beside S12.2.
- **The truth's Earth frame comes before everything that depends on the frame.**
  - Order within S12.4: P8c, then P9, then P2 and E4, then T5/E3's truth.
  - Next, S12.5: the onboard frame, then E2, whose velocities are in that frame.
  - Gravity's tesserals (S12.8) need the right Earth-fixed frame, so they come after S12.4.
- **Space weather comes before what Kp drives** (S12.9): the field's external term (E5), the wind (E15) and the storm scenarios (S12.15).
- **Requirements come before mass** (S12.12, then S12.13): Y3's budget is judged against what Y14 states or refuses.
- **The references and the done-when check come last among the fixes** (S12.14), because every model above changes first. The scenarios follow (S12.15), then one re-fly (S12.16).
- **Each design change is a developer revision with its source**, then `python3 tools/pcode.py gen`, `python3 tools/engine_build.py gen` and `python3 tools/flight_build.py gen`.
  - A tick or model change is made in Rust, C and MATLAB in the same commit (`docs/CHANGING.md`).
  - It is proven by `engine.py twin-parity`, `engine.py fsw-parity` and `tools/golden.py check`.
  - Golden rows that move are listed in the commit.
- **If a method's allowed calls change, the toolbox version rises.** For example, if the onboard nutation series needs a new maths function, the version in `adcs-sim/src/source.rs`, `tools/design_inputs.py` and `python/trinetra_adcs/design.py` rises together.


## 5 · Owner decisions

Each has a default. The developer takes the default as the developer's proposal and states it in its record in `design/decisions/` (S12.1), marked unsigned. Each is one node or one switch, so it is easy to change. Nothing is signed in S12.

| # | question | items | options | default |
| --- | --- | --- | --- | --- |
| 1 | Field model | IGRF-14 (DD42, V5) | adopt IGRF-14 for the engine, both flight softwares and the twin; stay on IGRF-13 | IGRF-14; onboard degree 10, truth 13 |
| 2 | Solar and geomagnetic activity levels | P6, T6/E9, E6's bundle, ENV-storm | the ECSS-E-ST-10-04C table (low, moderate, high long-term: F10.7 with its Ap); the survey's present 65/250 and the bundle's 70/130/250 with Kp 2 | one stated activity table from ECSS-E-ST-10-04C, used by the bundle, the survey and the scenarios; the values transcribed from the standard's table with its number, to be checked against the page by the owner |
| 3 | Data validity and its guard | P9, E14, P2 | warn and record past an expiry; refuse | a frozen EOP and leap-second snapshot (IERS finals2000A, Leap_Second.dat) with its expiry; a run past it warns and records, a published-vector case refuses; the epoch span 2025.0-2030.0 intersected with the files' expiry |
| 4 | Earth frame, truth and onboard | T5/E3, P2, E4 | truth on IAU 2006/2000A build A, B or C; onboard nutation full, IAU 2000B or none | truth on build A with EOP; onboard IAU 2000B with dUT1 and the pole as flight parameters set by telecommand |
| 5 | Aberration onboard | E2 | GNSS velocity plus the Sun model's derivative; an uplinked Earth velocity | GNSS velocity plus the Sun model's derivative (annual and orbital terms) |
| 6 | Field error in the truth | E5 | a Dst-scaled external dipole plus a crustal term of stated RMS; a higher-degree crustal table (WMM-HR, EMM: a file not in the repository); a stochastic field | the Dst-scaled external dipole plus a crustal term of stated RMS, dispersed per Monte Carlo draw, the magnitudes from the roadmap's cited ranges |
| 7 | Gravity field | T4, P7 | EGM2008 20x20 in POP; a lower degree | 20x20 in POP, the fast orbit stays J2-J6; the file needs icgem.gfz-potsdam.de, not reachable today |
| 8 | Space-weather indices in the loop | P1, T6/E9, ENV-storm | manual (today), dated table, JB2008 indices | manual stays the default; the dated CelesTrak SW-All table by a stated node; one historic storm (10-11 May 2024) as a scenario |
| 9 | F10.7 lag on the dated path | P3 | same day (today); the previous day (the DTM2020 specification) | the previous day |
| 10 | Orbit drag shape | P10 | the attitude torques' facets with the case's Cd; Sentman panels | the torques' facets with the case's Cd |
| 11 | Thermospheric wind | E15 | a stated wind by latitude and Kp; HWM14 (a file not in the repository); none | a stated wind by latitude and Kp, truth only, dispersed, its magnitudes the developer's and unsigned |
| 12 | What LTAN means | E12 | mean solar time; apparent | mean solar time |
| 13 | Mean-element start | P12 | Eckstein-Ustinov; Brouwer-Lyddane | Eckstein-Ustinov (J2, near-circular) |
| 14 | Albedo and Earth IR for the torques | E13 | Knocke 1988 rings with their constants; the point at nadir with its constants corrected | the rings with Knocke's constants |
| 15 | RK78 nodes in the loop | RK78 | adopt; drop | drop: RK4 at 10 s is 0.43 m a day |
| 16 | The reference ledger's form | V1 | docs/references.toml rows each with a test; design nodes | docs/references.toml, one row per model, each with a test or 'owed' with its reason |
| 17 | Physics relations no row uses (8) | PV | source them; mark them unused | mark unused, source the 31 used |
| 18 | Blank requirements | Y14 | refuse every blank the sizing or a closure needs; case-wide stated defaults in a decision record | refuse by name; the closure reads 'not stated'; no slew is sized unless req.slew and req.wmax are stated |
| 19 | Tip-off range | Y15 | the launch provider's figure; a developer proposal | refused until stated; the developer's proposal 0-30 deg/s per axis, sized at 30, written in the record for the owner to confirm, never flown as a requirement |
| 20 | The slew profile the sizing assumes | Y17 | cycloidal; bang-bang; rest-to-rest with a stated wmax | cycloidal, named so |
| 21 | Detumbles the propellant holds | Y18 | 2 (LEOP and one recovery); N per life; the owner's count | 2, stated with that reason |
| 22 | Mass growth and the electronics line | Y3 | AIAA S-120A by maturity; a flat 20 % | S-120A by maturity (sized in-house parts as new design); the electronics and harness line refused as not stated until a datasheet or the owner gives it |
| 23 | Beta extremes | ENV-beta | by epoch at the case's LTAN; by LTAN | by epoch at the case's own LTAN |
| 24 | Requirement values from the developer's reference (UNCONFIRMED, adcs_ref_c1) | Y14 | refused as requirements (the closure reads 'not stated'; the run reports the metric against the proposal as information); kept as requirements | refused as requirements, each kept in a record as the developer's proposal for the owner to state; the run still reports the metric against it, marked unconfirmed |

**Where a default is a value no one has stated:**
- **#2's table** is transcribed from ECSS-E-ST-10-04C, with its table number.
- **#6's and #11's magnitudes** are taken from the roadmap's cited ranges. They are the developer's and unsigned, and their records say so.
- **#19 and #24** are refusals by default. A value the developer proposes is written in the record for you to state. It is never flown as a requirement.

**The developer also makes smaller choices**, marked unsigned, each with its source on its revision:
- the interpolation of B (linear, in ECI);
- the onboard nutation's truncation within IAU 2000B;
- the EOP snapshot's date;
- the 50 IGRF-14 points' grid;
- the counting allocator in P8a's test.


## 6 · Proposed order of work

Each step is one agent's job of one kind of work. Every code step ends with `python3 tools/verify_work.py --since <start>` and the checks it names. Each fix lands in the design (a developer revision with its source, unsigned) and in the code in the same commit, with its failing test written first. Design editors run one at a time; S12.3 and S12.11 are code only and run in worktrees beside them.

| step | kind | tier | size | job | items | after |
| --- | --- | --- | --- | --- | --- | --- |
| S12.1 | decide | deep | M | The 24 owner decisions of section 5 as decision records (design/decisions/, carried in as nodes, unsigned), each on its default; the requirement rules of Y14-Y18 written as the developer's proposal. A design editor. | section 5 | - |
| S12.2 | transcribe | standard | M | The data in: IGRF-14 (two independent copies, the ppigrf IGRF14.shc and pyIGRF14 igrf14coeffs.txt, checked equal), the IERS finals2000A and Leap_Second.dat (astropy-iers-data), CelesTrak SW-All (the spaceweather package), each read into an env node by tools/readers.py with its source and date. EGM2008 waits for its host (S12.8). A design editor. | IGRF-14, P2, P9, P1, T6/E9 | S12.1 |
| S12.3 | code | standard | M | The analytic oracles first, so later fixes are measured: TS1 (a)-(e), (g) and P4's integrator tests. Code only, in a worktree, in parallel with S12.2. | TS1(a)-(e),(g), P4 | - |
| S12.4 | code | standard | M | The truth's Earth frame: P8c, P9, then P2 and E4 (build A with EOP in the loop and the twin, the field's precession not counted twice), T5/E3's truth side, E14's epoch guard. | P8c, P9, P2, E4, T5/E3, E14 | S12.2 |
| S12.5 | code | standard | M | Onboard frames and the tracker: T5/E3's onboard side (IAU 2000B, dUT1 and pole as flight parameters) and E2 (aberration in the truth tracker, its correction onboard); C, Rust, twin; flight images resealed. | T5/E3, E2 | S12.4 |
| S12.6 | transcribe | standard | S | IGRF-14 as one table for the engine, both flight softwares and the twin; E10's radius; TS1(f) with 50 IGRF-14 points from two independent implementations. | IGRF-14, E10, TS1(f) | S12.2 |
| S12.7 | code | standard | M | The truth field's error (E5, dispersed per draw) and B interpolated between refreshes (E8). | E5, E8 | S12.6, S12.9 (Kp/Dst) |
| S12.8 | transcribe | standard | S | EGM2008 20x20 (T4, P7) once its file can be fetched; the comments that claim tesserals corrected now. | T4, P7 | S12.4; the network host |
| S12.9 | code | standard | M | Space weather: P1 (the World's drivers), P3 (3-hourly Kp, F10.7 lag), P6 and S11's survey finding (one activity table, the survey on DTM2020), T6/E9 (dated indices). | P1, P3, P6, T6/E9 | S12.2 |
| S12.10 | code | standard | M | Drag and geometry: P10 (facets), E15 (wind), E11 (fine eclipse), E12 (mean Sun), P12 (mean elements), E13 (rings). | P10, E15, E11, E12, P12, E13 | S12.9, S12.3 (TS1 d, e) |
| S12.11 | code | standard | S | The propagator's two code fixes, P8a (no clone) and P8b (dense output), bit-identical. Code only, in a worktree, in parallel. | P8a, P8b | S12.3 |
| S12.12 | code | standard | M | Requirements: Y14 (blanks refused by name, the new keys), Y15 (tip-off a range), Y17 (the profile named), Y18 (the factors stated). | Y14, Y15, Y17, Y18 | S12.1 |
| S12.13 | code | standard | M | Mass: Y3 (the coil from its copper bound, the growth allowance by maturity, the electronics line stated or refused). | Y3 | S12.12 |
| S12.14 | transcribe | standard | L | The references: published vectors for the 31 used physics relations (PV), the reference ledger (V1) and its check that every environment model names its source and has a published-vector case: the done-when. | PV, V1 | every step above |
| S12.15 | code | standard | M | The environment scenarios: a scenario environment block (engine and twin), beta extremes, the activity levels and a storm, the seasons; golden rows added. | ENV-beta, ENV-storm, ENV-season | S12.9, S12.7 |
| S12.16 | run | light | M | Regenerate, reseal, re-fly every case that moved (engine, campaigns, the design loop, soft OILS, the twin), the parity gates, golden. | all | S12.15 |
| S12.17 | chore | light | S | Counts and tables (HEALTH, the ledgers, ENGINE_PARITY), the zip 1.x conversion. | all | S12.16 |
| S12.18 | decide | deep | S | The phase review and docs/S12_REPORT.md with the token ledger. | all | S12.17 |

S12.4, S12.9 and S12.10 change the truth that every scenario flies, so the golden table moves in most rows. Each of those commits lists the rows it moved and the verdicts that changed. The re-fly is done once, in S12.16, after S12.15.

## 7 · What this page did not do

- **It changed no code, design or test, and ran no scenario.** The open and fixed calls rest on the files and lines named.
- **The data sources were checked only as far as downloads:**
  - IGRF-14's `g10` at 2020.0 is `-29403.41` (`ppigrf`);
  - SW-All covers 10-11 May 2024 (Ap 236-400 on the 11th).
  - Nothing from the packages was copied into the repository.
- **Some figures in section 2 are the sources' and were not re-measured.** These are E2's 0.0071 deg, T5/E3's 25", E4's 0.15 deg and E8's 0.21 deg/s. Each step measures its own figure in its failing test.
- **Sizes are the roadmap's or the audit's.** They were raised where the fix is in three copies of the tick (E2, T5/E3 onboard) or where a new truth model is dispersed (E5, E15).
