# ADCS design decisions: what is decided, where, and what still needs an owner's record

**In one line:** the register of every decision an ADCS design must make, from pointing concept to OBC and operations.
For each: whether TRI-NETRA has made it, recorded it with alternatives, left it as a constant in code, or not faced
it yet. Ends with a decision-record format and the 15 decisions that most need one.

Companion to `docs/TECHNICAL_ROADMAP.md` (the audit), whose finding ids (A, E, F, G, M, Y, T) and the `docs/ADCS_GAPS.md`
ids (D, S, V, H) are cited here.

## Why this matters

TRI-NETRA has good machinery for decisions:
- owner tables: SPEC §20, `docs/RELEASE_PLAN.md` §9;
- signed human decisions: SPEC §17.2, `xtask decision record`;
- belief records: `spec/derisk/beliefs/`;
- 12 trade studies: `trades/`, `docs/SELECTION.md`.

But almost none of these hold an *ADCS architecture* decision:
- Of the 16 beliefs, three are ADCS-technical.
- SPEC §20's D1–D27 are almost all programme and business decisions.
- The 12 trades compare control laws inside a fixed architecture; every outcome is a "proposal", and all 31 algorithms are UNCONFIRMED (V4).

Meanwhile:
- About 60 design values sit as literals in `engine/crates/adcs-sim/src/config.rs`, with more in `fsw/src/adcs_fsw.c` and `adcs_fdir.c`. Examples: bandwidths, damping, rate groups, MEKF gate, star-tracker coast, dump thresholds, FDIR windows, onboard IGRF degree.
- Seven sizing constants have no source (`docs/SIZING_LAWS.md`).

A reviewer cannot tell which of these are engineering choices, which are placeholders, and which were tuned to make a run pass.

**The largest unrecorded decision** is the solution/benchmark family policy (`catalogue/families.toml`):
- For `ais_img_3u` the loop selects the in-house fluid ring at 1.794 kg and 3.52 W.
- The feasible wheel benchmark needs 0.701 kg and 3.71 W.
- That may be the right business choice. It is not recorded as an engineering decision with drivers, alternatives, TRL and risk.

## Status codes

| Code | Meaning |
|---|---|
| R | Made and recorded with rationale and alternatives |
| P | Made; rationale partly recorded; no alternatives shown |
| I | Made implicitly: a hard-coded constant (file:line) |
| O | Open, waiting on the owner |
| M | Missing: not faced at all |

## The register

### Mission and pointing concept

| Id | Decision | Status | Current choice (where) | Alternatives a reviewer expects | Reference | Action |
|---|---|---|---|---|---|---|
| DD1 | Mode set | P | detumble, Sun acquisition, Sun referencing, nadir, target, slew; 11 FSW states (`config.rs:17-20`, `catalogue/modes/`) | separation/LEOP, dump, inertial hold, payload calibration, explicit "no orbit control" | ECSS-E-ST-60-30C §4 | DR + transition matrix (Y8) |
| DD2 | Nadir definition, yaw steering | O | geocentric −r̂, no yaw steering (`docs/NAV_GUIDANCE_AUDIT.md`) | geodetic nadir; yaw steering; target-fixed frame | ECSS-E-ST-60-10C | Payload + owner DR before any APE claim (E7: up to 0.17°) |
| DD3 | Frames and onboard frame chain | R | J2000, IAU-76 precession, no nutation; scalar-last passive DCM (`fsw/pseudocode/00_conventions.md`, `02_*`) | IAU 2006/2000A; TEME fallback | Markley & Crassidis ch. 2; IERS TN36 | Add the floor (E2, T5) to the pointing budget |
| DD4 | Power-face yaw flip | P | on, hysteresis 0.1 (`config.rs:421`) | continuous Sun-optimal yaw; fixed yaw | SMAD ch. 19/21 | Record hysteresis and imaging effect |
| DD5 | Agility: slew profile and constraints | I | single-axis versine; 30° in 60 s UNCONFIRMED | eigenaxis, time-optimal, keep-out cones | Wie ch. 7; Kjellberg & Lightsey 2016 | Owner states `req.slew`, `req.wmax` (Y14, Y15) |
| DD6 | Payload and long axis in the body | I | boresight +Y_B; long axis along-track, GG-unstable (product TOML) | long axis to nadir (GG-stable) | Wertz §17; Sidi ch. 4 | DR linking accommodation to GG choice |

### Requirement formulation

| Id | Decision | Status | Current choice | Alternatives | Reference | Action |
|---|---|---|---|---|---|---|
| DD7 | Pointing indices | O | APE 0.01°, AKE 0.005° (UNCONFIRMED), rate stability; RPE and PDE blank | full ECSS set, RPE window | ECSS-E-ST-60-10C §4 | Owner sets RPE/PDE and windows |
| DD8 | Statistical interpretation, confidence | M | `level` 99.73 only; interpretation and confidence unstated (A1, A6) | ensemble, temporal or mixed per index, stated confidence | ECSS-E-ST-60-10C §4.2.3, §5.2 | **Urgent** DR per requirement |
| DD9 | Margins policy | I | k_h = 2 (`adcs-design/src/lib.rs:222`), k_tau 1.5, ×2 coil, ×1.2 propellant unsourced; no mass growth allowance (Y3) | maturity-based margins | ECSS-E-ST-10C; AIAA S-120A | Margins DR; derive each constant |
| DD10 | Who confirms requirement values | O | most case values UNCONFIRMED (D1) | — | NASA SP-2016-6105 §6.2 | Owner sign-off per key |

### Stabilisation concept and actuator families

| Id | Decision | Status | Current choice | Alternatives | Reference | Action |
|---|---|---|---|---|---|---|
| DD11 | Stabilisation concept | I | 3-axis active; Sun-spin for acquisition and safe | for the 10° AIS class: passive magnet + hysteresis rods, GG boom, momentum bias | SMAD Table 19-4 | Trade DR for the AIS class (D13) |
| DD12 | Solution vs benchmark families | P | `families.toml` roles; lighter feasible benchmarks excluded | in-house vs bought, TRL-weighted | NASA SP-2016-6105 §6.8 | **Urgent** business + engineering DR |
| DD20 | Momentum-exchange technology | P | trades + design loop | RW / CMG / VSCMG / fluid ring | Wie ch. 7 | DR after Y1/Y2/Y12 corrections |
| DD21 | Rotor geometry and redundancy | P | 3 orthogonal rings + skewed spare; 3 orthogonal wheels; 4-CMG pyramid (skew angle unrecorded) | 4-wheel pyramid/tetrahedron | Markley & Crassidis ch. 7 | DR with momentum envelope |
| DD22 | Momentum envelope, bias, saturation margin | I | wheel bias 2e-3 N·m·s (`config.rs:369`); target ≤ 0.25 h_max; `req.hsat` blank | zero bias vs bias | SMAD §19.1 | Owner sets `req.hsat` |
| DD23 | Dumping strategy and interval | O | coil gain 2e-3; RCS hysteresis 4e-3/1e-3; interval blank → quarter orbit taken (`lib.rs:204`) | continuous vs scheduled; coils vs thrusters | Sidi ch. 7 | **Urgent**: owner `req.dump`, `mission.life` |
| DD24 | Magnetorquer design | I | air core scaled from an invented anchor (Y3); pump stray dipole unbudgeted (Y7) | torque rods; cleanliness budget | ECSS-E-ST-20-07C; NASA SP-8018 | DR + magnetic budget |
| DD25 | Thruster propellant | P | N2O cold gas, 12 valves, 70 bar MEOP | butane, R-134a, warm gas, electrospray, none | SMAD ch. 18 | Trade DR (safety DD52) |
| DD26 | Actuator saturation handling | I | per-actuator clip (G2) | vector scaling, allocation with limits | Wie §7.3 | DR + fix |

### Sensors and calibration

| Id | Decision | Status | Current choice | Alternatives | Reference | Action |
|---|---|---|---|---|---|---|
| DD13 | Sensor suite per product class | I | same suite for every family | per-class suites; gyroless; no tracker on AIS | ECSS-E-ST-60-30C §5 | DR per class |
| DD14 | Star-tracker heads: number, placement | P/conflict | 2 heads ±25° with rationale in a product comment; the loop removes one as a mass lever; 51/55 MC failures are both heads blind (D3) | 3 heads; orbit-normal head; gyro coast budget | Markley & Crassidis ch. 5 | **Urgent** DR; lock the lever |
| DD15 | Magnetometer redundancy | M | single magnetometer, single-point failure; `mag_fail` excluded (Y11) | 2 magnetometers; field-free safe mode | ECSS-Q-ST-30-02C | DR from FMECA |
| DD16 | Gyro grade | R | precision MEMS (gyro trade) | FOG; gyroless | Wertz ch. 7 | Record trade outcome |
| DD17 | Sun-sensor type (AIS) | R | `trades/trade_sun_sensor_ais.toml` | — | — | Confirm |
| DD18 | Calibration strategy | I/M | tracker-to-payload residual 1e-5 rad assumed; magnetometer bias only; gyro scale not estimated | onboard bias/scale/misalignment states; calibration ops | Markley & Crassidis ch. 7 | Calibration DR + ops plan (T9) |
| DD19 | Keep-out and FOV policy | M | 30° Sun exclusion in the part; none in guidance | constrained guidance; head selection | Kjellberg & Lightsey | DR with D3 study (T11) |

### Estimation, control, guidance

| Id | Decision | Status | Current choice | Alternatives | Reference | Action |
|---|---|---|---|---|---|---|
| DD27 | Estimator states | P | 6-state MEKF (`fsw/pseudocode/03_estimation.md`) | 9/15 states (scale, misalignment, mag bias), UKF | Markley & Crassidis ch. 6–7 | DR |
| DD28 | Initialisation and re-init | I | TRIAD with isotropic P; TRIAD_MIN_SIN 1e-3; bias lost on re-init (G1, G15) | TRIAD covariance; keep bias | Shuster & Oh 1981 | DR + fix |
| DD29 | Gating and consistency | I | χ² 16.27 at 3 dof; tracker ungated (G10, S6) | 2-dof gate; NIS/NEES | Bar-Shalom | DR |
| DD30 | Latency and time-tagging | P | tracker and GNSS latency only | time-stamped measurement buffer | ECSS-E-ST-60-30C | DR |
| DD31 | Onboard orbit | R | GNSS + J2 Verlet ("32 km/orbit" rationale) | SGP4 from ground TLE fallback | Vallado | Add the fallback decision |
| DD32 | Control law per mode | P | trade proposals; defaults in `config.rs:69-70` | — | — | Owner confirms per slot |
| DD33 | Gain design method, margins | I | bandwidth/damping placement (`config.rs:338-340`); Bryson LQR weights hard-coded; B-dot ×3 on orbital inclination (G11); no margins (T1); PID gain scaling (G6) | loop shaping with margins; μ-analysis | ECSS-E-ST-60-10C; Wie ch. 7 | **Urgent** DR + margin analysis |
| DD34 | Feed-forwards | P | GG feed-forward rule; residual dipole compensation; tracking FF incomplete (G9) | — | Schaub & Junkins ch. 8 | DR |
| DD35 | Rate groups and timing | I | dt 0.1 s; fine law 10 Hz; coil cycle 1 s with 0.2 s measure window; float-time scheduling (F3) | integer tick groups | ECSS-E-ST-40C §5.4 | **Urgent** DR |
| DD36 | Allocation and singularities | P | pinv / IDMAS / SR inverse λ0 1e-9, μ 10, k_null 0.002 (`config.rs:380`) | null-space equalisation (G7) | Schaub, Vadali & Junkins 1998 | DR |

### Modes, safe mode, FDIR

| Id | Decision | Status | Current choice | Alternatives | Reference | Action |
|---|---|---|---|---|---|---|
| DD37 | Transition logic, autonomy | I | auto_next + ≤ 8 schedule entries; detumble exit 0.5°/s for 60 s, while `catalogue/modes/detumble.toml` says 10 min and the metric uses 600 s | ECSS autonomy levels E1–E4 | ECSS-E-ST-70-11C | DR on autonomy level |
| DD38 | LEOP and separation | M | none; tip-off 10°/s UNCONFIRMED | inhibit timers, deployment, first-acquisition energy | ECSS-E-ST-60-30C; CubeSat Design Spec | DR |
| DD39 | Safe-mode concept and independence | O | coils-only detumble after 60 s sensor silence; can deadlock (G5); same OBC and magnetometer as nominal (H5) | separate safe controller; Sun-spin safe; rotor+gyro safe | ECSS-E-ST-60-30C; SAVOIR | **Urgent** owner DR |
| DD40 | FDIR hierarchy | M | rotor window 120 s/0.5 % (fluid rings only); magnetometer 0.25–4×\|B\|; star-tracker coast 900 s with a stale gyro (F2) | levels 0–4; innovation-based isolation | ECSS-E-ST-60-30C; SAVOIR FDIR | **Urgent** DR |
| DD41 | Fault-tolerance policy | R | single fault survived ("gap" policy); `req.faults` blank; magnetometer excluded | — | ECSS-Q-ST-30C | Reconcile with DD15 |

### Onboard models and environment assumptions

| Id | Decision | Status | Current choice | Alternatives | Reference | Action |
|---|---|---|---|---|---|---|
| DD42 | Onboard field model | O | IGRF-13 degree 10 (truth 13), extrapolated past 2025 | IGRF-14; degree vs CPU | IGRF-14 | Owner adopts IGRF-14 |
| DD43 | Onboard Sun and eclipse | P | Vallado low-precision; cylindrical shadow | conical penumbra | Vallado | Record (S3) |
| DD44 | Truth environment assumptions | I | F10.7 130, Kp 2; accommodation 0.8; CM-offset direction fixed [0.30, 0.70, −0.65] (`config.rs:654`); dipole split equally | worst-case directions; DTM2020 low/mean/high everywhere | ECSS-E-ST-10-04C | DR on worst-case philosophy |
| DD45 | Epoch and lifetime | O | 2027 UNCONFIRMED; life blank → 3 yr taken | — | — | Owner |

### Processor, numerics, interfaces, parameters

| Id | Decision | Status | Current choice | Alternatives | Reference | Action |
|---|---|---|---|---|---|---|
| DD46 | OBC and processor | O | Cortex-M4F (QEMU); board open (H1) | M7 with double FPU; rad-tolerant MCU | ECSS-E-ST-40C | **Urgent** owner |
| DD47 | Numeric precision | I/conflict | software double on a single-precision FPU (F9); f32 build broken but claimed tested (F10) | mixed precision; M7 | ECSS-E-ST-40C §5.8.3 | DR with DD46 |
| DD48 | Execution model | M | bare-metal polled loop; no concurrency contract (F14) | RTOS; time partitioning | ECSS-E-ST-40C | DR |
| DD49 | Buses and protocols | P | I2C/SPI/UART/CAN map (`09_drivers_hal.md`); EB90 framing; electrical level open (SPEC D14); device map contradicts the HAL header (O12) | RS-422/485; CSP | ECSS-E-ST-50C | Record; one source of truth |
| DD50 | Time sync and TM/TC standard | M | onboard date from the blob epoch (F1); no TM/TC (F8) | GNSS PPS discipline; CCSDS SPP + PUS-C | ECSS-E-ST-70-41C; CCSDS 133.0-B | **Urgent** DR |
| DD51 | Parameter management | P | CRC'd blob, validated; full re-init only; no schema hash (F13) | PUS service 20 patch with echo; golden blob | ECSS-E-ST-70-41C | DR |

### Safety, standards, operations, verification, design loop

| Id | Decision | Status | Current choice | Alternatives | Reference | Action |
|---|---|---|---|---|---|---|
| DD52 | Propellant, pressure vessel, liquid metal | M | tank at MEOP 70 bar, no MDP/proof/burst factor; galinstan freezes at −19 °C; Ga/Al compatibility unrecorded (Y9) | ECSS factors; launch rules | ECSS-E-ST-32-02C; CubeSat Design Spec | **Urgent** DR |
| DD53 | Standards baseline and tailoring | P | ECSS list in SPEC §14; criticality open (D15) | compliance matrix | ECSS-S-ST-00C | DR |
| DD54 | Ground operations concept | M | no operations group (Y16) | — | ECSS-E-ST-70C | DR |
| DD55 | Verification ladder | R | SILS → soft OILS → OILS → HILS (SPEC §12, `docs/OILS_HILS.md`) | — | ECSS-E-ST-10-02C | — (see `docs/TEST_BENCH_PLAN.md`) |
| DD56 | Method per requirement | P | matrix specified (SPEC §14.1); about 90 of 243 rows linked (D11) | — | ECSS-E-ST-10-02C §5.2 | Populate (`docs/TEST_STRATEGY.md` TS6) |
| DD57 | Monte Carlo sizing and dispersion pedigree | O | 15–24 runs per campaign, 1109 for claims; uniform dispersions (A15) | CP sizing; worst-case search | ECSS-E-ST-60-10C §5; Hanson & Beard | Tie to DD8 |
| DD58 | Engine vs twin parity thresholds | O | SPEC D23 | — | NASA-STD-7009B | Owner |
| DD59 | Ranking rule | O | code: least mass; SPEC §8.6: worst margin (D10) | weighted utility; margin-first | NASA SP-2016-6105 §6.8 | **Urgent** owner |
| DD60 | Robustness step vs budget | O | escalates past the budget (D15, Y6) | stop at budget, name the failure | — | Owner |
| DD61 | Budget scope | M | power = actuators only (Y1); mass without electronics or harness (Y3); no thermal/data (D8) | full ADCS budgets | ECSS-E-ST-10C | DR |

## Decision record (DR) format

Reuse what exists rather than adding a tool:
- design nodes;
- belief records;
- `xtask decision record` signatures;
- `fsw/params/params.toml`.

**Where:**
- Design nodes of `kind = "decision"` in the group that owns them: `gdn` DD1–DD6/DD37, `nav` DD27–DD31, `fdir` DD39–DD41, `design` DD9/DD59–DD61, and so on.
- Until carried into the group database, source text lives in `design/decisions/DR-<group>-NNN.toml`.

**Fields:**

```toml
schema = "trinetra-decision/1"
id = "DR-gdn-002"                 title = "Nadir definition"
group = "gdn"  owner_team = "gnc" status = "proposed"   # proposed | accepted | superseded
context  = "why a choice is needed; requirement rows it serves (req.ape, req.ake)"
drivers  = ["req.ape", "payload ICD"]      standard = ["ECSS-E-ST-60-10C §4"]
[[option]]  name = "geocentric"            evidence = "run / campaign / trade id"   rejected_because = "..."
[[option]]  name = "geodetic + yaw steering" evidence = "..."
choice = "..."   rationale = "..."   consequences = "..."
sets = ["fsw/params/params.toml:gd_kind", "engine/crates/adcs-sim/src/config.rs:<function>"]
belief = "B-0xx"   revisit_when = "payload ICD v1 / IGRF-15"   supersedes = ""
signature = { role = "...", name = "...", at = "..." }   # by the person, via xtask decision record
```

**Checks:**
- Every field in `params.toml` and every sizing constant gets `decided_by = "DR-…"`, or `derived = "<node>"` when a formula sets it.
- A check (extending `tools/groups.py --check`) fails when a design literal in `config.rs`, the FSW `#define`s or `adcs-design` carries neither a DR tag nor a `derived` tag.
- A confirmed trade becomes the evidence of its DR's options, replacing "proposal" in `docs/SELECTION.md`.
- A generated `docs/DECISIONS.md` lists every DR and its status.
- No agent signs a decision on a person's behalf. Agents write stubs; the owner signs.

## The 15 decisions that most need an owner-approved record

1. **DD8**: statistical interpretation and confidence per requirement. Without it no verdict means anything.
2. **DD2**: nadir definition and yaw steering.
3. **DD39 + DD15**: safe-mode concept, independence, magnetometer redundancy.
4. **DD40**: FDIR hierarchy, including gyro staleness, the 900 s coast and a watchdog.
5. **DD50**: onboard time, synchronisation, TM/TC standard.
6. **DD12**: solution vs benchmark family policy.
7. **DD59 + DD60**: ranking rule; may robustness break the budget.
8. **DD9 + DD61**: margins policy and budget scope.
9. **DD14 + DD19**: star-tracker heads, placement, keep-out (the loop overrides the recorded rationale today).
10. **DD46 + DD47**: OBC, FPU and numeric precision.
11. **DD22 + DD23**: momentum envelope, saturation margin, dump strategy and interval.
12. **DD33 + DD35**: gain design method with margins; integer-tick rate groups.
13. **DD57**: Monte Carlo sizing, dispersion pedigree, method per requirement.
14. **DD52**: N2O pressure vessel factors; galinstan thermal limits; material compatibility.
15. **DD1 + DD38**: complete mode set with transition matrix; LEOP sequence and tip-off range.

**First step (roadmap U0):**
1. Write DR stubs for these 15, with options and evidence pulled from the audit, the gap register and `trades/`.
2. Tag the existing literals with the DR ids they belong to.
3. The owner signs each through the existing `xtask decision record` route.

## References

- ECSS-E-ST-60-30C, ECSS-E-ST-60-10C, ECSS-E-ST-10C, ECSS-E-ST-10-02C, ECSS-E-ST-10-04C, ECSS-E-ST-32-02C, ECSS-E-ST-40C Rev.1, ECSS-E-ST-70-41C, ECSS-E-ST-20-07C.
- Wertz, Everett & Puschell, *Space Mission Engineering: The New SMAD* (2011) ch. 19; Wertz, *Spacecraft Attitude Determination and Control*; Sidi; Markley & Crassidis (2014); Wie (2008).
- NASA/SP-2016-6105 Rev2 (§6.8 decision analysis).
- Architecture decision records (context, options, decision, consequences).

Links are in `docs/TECHNICAL_ROADMAP.md`.
