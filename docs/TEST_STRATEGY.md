# Test strategy: from model unit tests to operations validation

**In one line:** an audit of TRI-NETRA's testing at every level against ECSS and NASA practice. It finds that the suite is
large and exact but checks *consistency* far more than *correctness*. It sets out the tests, the scenario matrix and the
phases that close that gap.

Companion to `docs/TECHNICAL_ROADMAP.md`, `docs/TEST_BENCH_PLAN.md` and `docs/ADCS_GAPS.md`.

## 1 · What exists (measured)

| Suite | Result |
|---|---|
| Python unittest (`tests/`, 32 files) | 319 tests pass |
| Engine crates (13) | 178 tests pass. adcs-sim 58 · adcs-pop 50 (ERFA vectors, MATLAB POP) · adcs-sim-core 25 · adcs-plot 12 · adcs-cli 10 · others 23 |
| Rust flight software | 58 pass, 1 ignored; **0 unit tests inside `fsw-rs/src`** |
| C flight software | 45 checks; pseudocode vectors: 78 functions, 38,696 values, 38,636 bit for bit |
| MATLAB/Octave twin | 44 / 44 (with the design opened, built and flown by +trinetra, S7.18) |
| `tools/check_all.py` | 22 checks always, 4 optional (pages, twin, propagator, mutation) |
| Fuzzing | Parameter blobs, telecommands, bus bytes: 3,000 × 20 ticks, exact timestamps |
| C vs Rust differential | 60 random configurations × 200 ticks, bit for bit |
| Scenarios | 48 (detumble 6, fine hold 12 incl. faults, slew 8, agile 5, mission chains 7, magnetic AIS 4, Sun 5); each of the 8 fault kinds flown once |
| Campaigns | 6 Monte Carlo + 2 edge |
| Parity | Engine vs twin 94/107 verdicts; SILS vs soft OILS 124/126; virtual OBC bit-identical over 600 s |
| Mutation | 97.9 % on `ctl.rs` and `est.rs` only |
| CI | `ci.yml` on every PR (check_all, QEMU parity 300 s, Octave, drive pack, API docs); weekly mutation; release on tag. **No CI job re-flies the scenarios or campaigns.** |

Worth keeping:
- bit-exact equivalence from pseudocode to C to Rust to QEMU;
- seeded determinism;
- inputs refused by name;
- external oracles from ERFA, pyIGRF and scipy;
- the ECSS index oracles;
- stack analysis;
- the mutation culture.

This is well above typical CubeSat practice.

## 2 · The central finding

Almost every oracle in the suite is one of three kinds:
1. **Self-consistency:** conservation, determinism.
2. **Equivalence:** C = Rust = QEMU, engine ≈ twin, Rust ≈ interpreter.
3. **"It works" thresholds:** detumble ends below 0.2·ω₀; the MEKF converges below 0.05°.

Very few say **"this matches the design or the physics, to a stated tolerance"**.

The consequence is visible in the audit:
- **G6:** the PID loop flies at about 0.7× its bandwidth.
- **F1:** the calendar rewinds after a reset.
- **F2/G3:** a stale gyro gives 143° of error in 50 s.
- **F4/F5:** star-tracker initialisation accepts a non-unit quaternion, and a gyroless detumble exit fires on a zeroed rate.

All of them passed every test. Equivalence cannot find common-mode errors; a threshold cannot find a loop that is 30 %
slow but still converges. ECSS-E-ST-10-02C §5.2, ECSS-E-ST-60-30C §5 and ECSS-E-ST-40C Rev.1 §5.5–5.6 all point to
the same fix: **specification-based oracles**.

## 3 · Gaps and the tests that close them

Severity: C critical, H high, M medium, L low. Effort: S days, M 1–2 weeks, L longer.

| Id | Level | Sev. | Gap | Tests to add (pass criterion) | Effort |
|---|---|---|---|---|---|
| TS1 | Model unit | H | Few analytic oracles; weak ones in the twin (`t_igrf` is a range check, `t_shadow` 3 points) | (a) torque-free axisymmetric body: nutation rate λ = (I₃−I₁)/I₁·ω₃ to 1e-6, nutation angle constant to 1e-9 rad; (b) GG libration: pitch frequency n·√(3(I_x−I_z)/I_y) within 0.5 % (Hughes ch. 9); (c) B-dot initial decay vs Avanzini & Giulietti within 15 %; (d) J2 nodal rate within 0.1 %, SSO 0.9856°/day within 0.5 %; (e) eclipse durations vs Vallado §5.3 within 2 s; (f) 50 published IGRF points at engine and twin level; (g) flat-plate SRP and drag torque | M |
| TS2 | Algorithm | H | No statistical consistency tests of the estimator (S6, G10) | NEES/NIS: 50 seeds × 3 orbits, time-averaged NEES in the χ² 95 % band (about [2.32, 3.68] at 3 dof; [4.9, 7.1] at 6 dof), NIS per sensor in band, innovation lag-1 autocorrelation < 0.1; observability test (magnetometer-only along a near-constant field: covariance grows as Q predicts); steady-state 3σ regression (fail on > 20 % drift) | M |
| TS3 | Algorithm | H | Controllers never checked against their linear design (G6 was found by audit) | Per law and axis: 0.5° step, disturbances off; rise, overshoot, 2 % settling vs the closed-loop poles from the parameter blob within 10 %; frequency sweep on the linearised loop: GM ≥ 6 dB, PM ≥ 30°, bandwidth = `rw_bandwidth` within 10 % | S–M |
| TS4 | FSW | C | Robustness and negative tests miss the known failure classes; fuzz and differential use exact timestamps and ≤ 200 ticks; no reset tests; mutation excludes `fsw`, `modes`, `fdir`, drivers | Warm reset at day 30: onboard JD continuous within 1 s, schedule not replayed. Clock step −1 s / +1 h and ±1 µs / ±5 ms jitter: every rate group runs on ≥ 99.9 % of its ticks. Gyro frozen: stale within 3 ticks. Tracker \|q\| = 0.5: refused. Gyro 1,000 rad/s, Sun norm 0: refused + counted. Babbling CAN 10⁴ frames: step time bounded. PWM write error: fault raised. Mutation on all FSW files (and C via mull/dextool); coverage gates: lines ≥ 90 %, branches ≥ 80 % on `adcs_fsw.c` (48 % / 34 % today) | M |
| TS5 | FSW / system | H | No soak test; longest scenario about 8 h | 14-day engine soak of the imaging case: no NaN; wheel momentum < 80 % h_max with dumping; daily APE p99.73 shows no trend; covariance symmetric to 1e-12; onboard time error < 1 ms. Plus a 24 h QEMU soak, bit-identical to in-process every 600 s. Weekly | S |
| TS6 | FSW | H | Not requirements-based: no software requirements specification; tests carry no requirement ids (D11, A10) | SRS of about 100–150 requirements; every test tagged; `trace.py --check` fails on untested requirements or untraced tests; verification control document per ECSS-E-ST-10-02C Annex B | M |
| TS7 | FSW | M | Timing rests on an assumed CPI; overrun behaviour never exercised | QEMU busy-loop of 1.2 × the period: next tick skipped or marked, command held, event raised. WCET (static or measured high-water mark) as a CI artefact, CPU ≤ 50 % at worst case | S (M on board) |
| TS8 | FSW | H (after F8) | No TM/TC or parameter-update tests | PUS conformance per service (1, 3, 5, 8, 9, 20): acceptance and execution reports; mid-hold parameter patch takes effect next tick, echoed, refused out of range with no state change; housekeeping against the TM database | M |
| TS9 | System | H | Thin scenario matrix: only detumble→nadir, safe-mode entry/exit and slew→fine transitions judged; each fault once; no named environments | The matrix in §4: eclipse transitions, β extremes, solar min/max and storm (with field error E5), tip-off envelope to 30°/s, LEOP, day-in-the-life, combined faults and faults during transitions | M–L |
| TS10 | Statistics | M | No statistical regression or convergence evidence; three campaigns share a seed | Independent seed streams; persisted quantiles and CP bounds; nightly reduced MC (100 runs) with KS/Anderson-Darling against the baseline (fail at p < 0.01 unless declared); running-p99.73 convergence plot with bootstrap interval | S–M |
| TS11 | Regression | H | CI never re-flies the scenarios; a gain change can merge green with every stored verdict stale | Golden metric table per scenario (`results/golden.json`: value, tolerance, fingerprint); per PR, re-fly all 48 on the engine and compare; nightly reduced campaigns + store staleness gate; weekly full campaigns + mutation | S–M |
| TS12 | Cross-validation | M | Cross-checks are self-referential: engine vs twin and adcs-pop vs MATLAB POP share an author and assumptions (E1 shows the shared blind spots) | Three shared scenarios in Basilisk or NASA 42 (torque-free + GG; B-dot with a dipole field; wheel PD hold): histories within about 0.1° over one orbit, distributions for MC. Orekit orbit over 7 days with matched force models: ≤ 100 m along-track | M |
| TS13 | AIT | H | No polarity, phasing or calibration procedures, even as simulations | Write them now as engine scenarios (`polarity_*`) and replay unchanged in HILS: +X field in the cage → +X magnetometer within 5 %; +X coil → +X dipole at a reference magnetometer; +X rate table → +X gyro, scale within 1 %; positive wheel torque → opposite body reaction on the air bearing; open-loop B-dot sign on the bearing. Then magnetometer ellipsoid fit, wheel friction curve, TVAC sensor bias/scale vs temperature, residual dipole vs the Y7 budget | S to write, L to run |
| TS14 | Facilities | M | No simulator plan per lifecycle phase | One-page facility map per ECSS-E-TM-10-21A: TRI-NETRA has the functional engineering and mission performance simulators (engine, twin) and an SVF prototype (soft OILS); none of: FVT/HILS harness, AIV simulator, operations/training simulator. State validation status and reuse path | S |
| TS15 | Operations | M (H before flight) | No end-to-end ground or procedure validation | Yamcs/COSMOS on the link server; LEOP and contingency procedures run by operators over CCSDS against the soak and LEOP scenarios; each contingency reaches safe mode and recovers | L (after F8) |
| TS16 | Independence | M | One author writes code, tests and oracles | Independent reviewer writes TS3/TS4/TS9 tests from the SRS without reading the code; test review board per phase (ECSS-Q-ST-80C Rev.2 ISVV by criticality; ADCS safe mode typically category B) | M |
| TS17 | Oracle quality | L–M | "Passes if not broken" oracles | Replace `t_igrf` range → point vectors; `t_shadow` 3 points → boundary sweep; `detumble_reduces_rate` → analytic rate; `t_mekf` → NEES; audit every test for such oracles | S |

## 4 · Test architecture

### Pyramid (counts are targets)

| Layer | Content | When |
|---|---|---|
| 1. Model and algorithm unit | Analytic oracles (TS1), published vectors, linear-design checks (TS3), estimator consistency (TS2); about 400 tests (about 280 today) | Every PR |
| 2. FSW unit and integration | Requirement-tagged, robustness and negative cases (TS4); fuzz with jittered time and long flights; C/Rust differential; mutation on all files; coverage gates incl. MC/DC | Every PR |
| 3. Scenario | The matrix below, golden table with tolerances (TS11) | Every PR (engine); nightly (twin) |
| 4. Statistical | MC and edge campaigns, CP verdicts, KS regression, convergence | Nightly reduced; weekly full |
| 5. Soak | 14 days engine; 24 h QEMU | Weekly |
| 6. Independent cross-validation | Basilisk/42, Orekit | Per release |
| 7. OILS / HILS / AIT | Polarity, calibration, functional, performance, TVAC, magnetic, each against its SILS twin | Hardware-paced |
| 8. Operations validation | Procedures over the ground segment | Pre-flight |

### Scenario coverage matrix

| Axis | Values |
|---|---|
| Mode / transition | detumble; coils-only Sun and nadir; rotor Sun acquisition; fine nadir; target; slew; dump; safe; transitions detumble→Sun→nadir, nadir→slew→target→nadir, any→safe→recovery; eclipse in/out within each |
| Environment | β min and max per case; eclipse season vs none; F10.7 70/150/250 (DTM2020); Kp 0/4/9 with field error; solstices and equinoxes |
| Initial conditions | Tip-off 2/10/20/30°/s about major, minor and intermediate axes; attitude 0/90/180° |
| Faults (from the FMECA, Y11) | Each sensor silent, stuck, biased, scale ×2, noise ×10; each actuator dead, stuck, half torque, reversed polarity; OBC reset, clock step, overrun, SEU; combined (fault in transition, fault in eclipse) |
| Duration | Single mode (1–3 orbits); LEOP (≤ 1 day); day in the life (1 day); soak (14 days) |

Pairwise (orthogonal-array) selection over these axes gives about 90 deterministic scenarios instead of the full product.
Every cell carries an expected outcome (pass, or degraded within a stated bound) and a numeric criterion.

## 5 · Phases (aligned with the roadmap)

| Phase | Test work | Exit |
|---|---|---|
| U0 · v1.1 | TS11 golden regression in CI; TS4 reset, clock, jitter and stale-gyro tests (guarding F1–F5 as they are fixed); TS3 step responses (guarding G6); TS17 | Every U0 fix lands with a test that failed before it |
| U1 · v1.2 | TS1 analytic suite; environment scenarios (β, storm, season) | Every model has an analytic or published oracle |
| U2 · v1.3 | TS2 NEES/NIS; TS10 statistical regression; TS12 Basilisk/42 and Orekit; TS9 eclipse transitions and LEOP | Credibility table backed by independent comparisons |
| U3 · v1.4 | TS9 full fault matrix from the FMECA; TS5 soak; TS13 procedures written as SILS scenarios | Each mode and transition verified per ECSS-E-ST-60-30C |
| U4 · v1.5 | TS6 SRS and verification control document; TS4 coverage and mutation gates; TS7 overrun tests; TS8 PUS and parameter tests; TS16 independent tests | Software verification report in ECSS-E-ST-40C Rev.1 shape, generated by CI |
| U5 · v2.0 | TS13 executed on board and HILS; TS14 facility map; TS15 operations validation | Each hardware run accepted against its SILS twin |

Roughly 3–4 person-months of test engineering before hardware.

## References

- ECSS-E-ST-10-02C (verification)
- ECSS-E-ST-10-03C Rev.1 (testing)
- ECSS-E-ST-60-30C (AOCS verification)
- ECSS-E-ST-60-10C (performance)
- ECSS-E-ST-40C Rev.1 and ECSS-Q-ST-80C Rev.2 (software)
- ECSS-E-TM-10-21A (simulation facilities): https://ecss.nl/hbstms/ecss-e-tm-10-21a-system-modelling-and-simulation/
- NASA-STD-7009B (model credibility)
- NASA GSFC-STD-7000 (GEVS)
- Hanson & Beard, NASA TP-2010-216447 (Monte Carlo for verification)
- Hughes, *Spacecraft Attitude Dynamics*
- Avanzini & Giulietti 2012 (B-dot)
- Vallado, *Fundamentals of Astrodynamics and Applications*
- Bar-Shalom et al. (NEES/NIS)

Links for the standards are in `docs/TECHNICAL_ROADMAP.md`.
