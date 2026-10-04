# Technical audit of 1.0.0, and the roadmap to 2.0

**In one line:** an end-to-end technical audit of TRI-NETRA ADCS 1.0.0 in six domains, each finding checked in the
code against a named standard or textbook, sorted into what is *wrong today*, what is *missing*, and the phases
(v1.1 → v2.0) that close them.

This document adds to `docs/ADCS_GAPS.md` (31 gaps: ids D, S, V, H) and `docs/UPGRADE_PLAN.md` and does not repeat them.

Its companions go deeper on four subjects:

| Document | Subject | Ids |
|---|---|---|
| `docs/TOOL_COMPARISON.md` | Capability matrix against the established tools, and what to adopt, interoperate with or build | — |
| `docs/DESIGN_DECISIONS.md` | Register of 61 ADCS design decisions and the decision-record format | DD |
| `docs/TEST_BENCH_PLAN.md` | Soft OILS → OILS → HILS: findings, target architecture, equipment, phases, acceptance thresholds | O |
| `docs/TEST_STRATEGY.md` | Testing at every level, the scenario matrix and the test pyramid | TS |
New findings carry a domain prefix:

| Prefix | Domain |
|---|---|
| E | orbit, time, frames, environment |
| M | dynamics, sensor and actuator models |
| G | onboard guidance, navigation and control |
| F | flight software as embedded software |
| A | verification, statistics and performance assessment |
| Y | system design and the design loop |
| T | the first-pass items, kept |

**Density:** DTM2020, already in `adcs-pop`, is the one density model everywhere: truth, sizing, spec physics rows
and worst-case rows. NRLMSIS is not used (owner's decision).

## 0 · What the audit changes about 1.0.0's own results

Seven findings mean numbers shipped in 1.0.0 are wrong, or claims about them are. They are corrected first (phase U0),
before any new capability. Each was confirmed in the code, not only reported.

| Id | What 1.0.0 says | What is true | Where |
|---|---|---|---|
| A1 | `results/EVALUATION.md`: APE and AKE closures **pass** for both cases | The design loop's 12-run Monte Carlo has no claim, so `evaluate.py` takes the worst of 12 runs and calls it a pass against a 99.73 % requirement. 12/12 shows only R ≥ 0.78 at 95 %. The 1,109-run `mc_fine_img` shows 93.8 % (not met), and `results/TRACEABILITY.md` already says "not met". The `ais_3u` pass is on a design marked not feasible. | `tools/evaluate.py:139-175` |
| Y1 | `power_mean` passes `req.pavg` ("ADCS orbit-average power"): 0.35 W ≤ 2.0 W imaging, 0.12 W ≤ 0.5 W AIS | The metric sums coils + wheels + thrusters only. Sensors (1.6 W AIS, 2.6 W imaging: GNSS, star tracker, gyro, CSS, magnetometer) appear nowhere. With them, every `ais_3u` family breaks 0.5 W and the imaging selection breaks 2.0 W. | `engine/crates/adcs-sim/src/metrics.rs:312-313` |
| Y2 | CMG and VSCMG benchmarks: 1.4 kg, 5.9 W, fail power | The catalogue's ADCS400 is a *cluster* of 4 CMGs, counted as one of four units: mass, power, volume and momentum are 4× too high. | `engine/crates/adcs-design/src/lib.rs:282` |
| M1 | `mtq_fmr` survives every single fault (spare ring) | A failed fluid ring keeps its momentum forever: rings get zero friction terms, and the failed branch applies only those. In reality the fluid stops in T_sd ≈ 3 s and its momentum, up to 10°/s of body rate, goes into the body. The fault campaign never sees the transient. | `engine/crates/adcs-sim-core/src/actuators.rs:106`, `engine/crates/adcs-sim/src/product.rs:165` |
| G6 | PID loops tuned to `rw_bandwidth`, damping 2 | Kp = J·ωn² acts on the quaternion vector part (θ/2), while the LQR branch correctly uses 2e. The PID and magnetic-PD loops fly at about 0.7× the stated bandwidth, and the dominant pole is about 2× slower (ζ_eff ≈ 2.8). | `fsw-rs/src/ctl.rs:42`, `engine/crates/adcs-sim/src/config.rs:340` |
| A2 | `results/ENGINE_CAMPAIGNS.md`: "the same run count … the engine is therefore the more conservative" | The twin flew 24/40/20/20 runs; the engine flew 1,109. On `ape_los_p9973` the twin passes 29 % and the engine 95 %: the engine is the *less* conservative, by a margin with p ≈ 1e-17 of being chance. | `tools/engine_campaigns.py:358-366` |
| E1 | `results/ENGINE_PARITY.md`: field "bit-identical" engine vs twin | Commit aef56c8 moved the engine and both flight softwares to GMST × IAU-76 precession; the twin was not moved (truth field, onboard field, GNSS frame). They differ by up to 0.23° in field direction. The parity tool compares only \|B\|, so it cannot see this. | `matlab_sils/+asils/run.m:83`, `+fsw/step.m:56` |

Also wrong but not changing a verdict today:
- A3: the V&V report says every differing verdict is traced; five are not.
- A5: about 4 % of Monte Carlo inertia draws, and every edge-campaign inertia corner, break the triangle inequality, so the body is physically impossible.
- E6: the spec's "worst aero torque" row uses a static table 7.4× below the DTM2020 high-activity peak, and the spec still names NRLMSIS.

**What this means for the release:**
- **The 1.0.0 binaries are not affected.** They fly the code faithfully.
- **Three of 1.0.0's verdicts must be corrected:** its *verdicts about the two cases* (A1, Y1), and the CMG benchmark comparison (Y2).
- **The release notes must say so.** "4 closures pass" should read "not demonstrated", and the power verdicts as "actuator power only".

## 1 · Method

Six independent audits, one per domain:
1. orbit, time and environment;
2. dynamics and devices;
3. onboard GNC;
4. flight software;
5. verification;
6. system design.

Each read the code, and where cheap built throwaway programs against the repository's own crates to check numbers. Examples:
- momentum conservation;
- LQR against scipy's CARE;
- q-method against numpy;
- DTM2020 densities;
- frame differences;
- stale-gyro and reset demos linked against the C flight software.

The serious findings were then re-checked in the code. Standards and references:

| Area | Standard or reference |
|---|---|
| AOCS requirements, modes, FDIR, verification | ECSS-E-ST-60-30C (2013) [1] |
| Control performance, indices, budgets, margins | ECSS-E-ST-60-10C (2008) [2]; ECSS-E-HB-60-10A; ESA ESSB-HB-E-003 Pointing error engineering handbook [3] |
| Star sensors | ECSS-E-ST-60-20C Rev.2 (2019) [4] |
| Space environment | ECSS-E-ST-10-04C Rev.1 (2020) [5]; IGRF-14 [6]; DTM2020 (Bruinsma & Boniface 2021); IERS Conventions 2010 (TN 36) |
| Verification and testing | ECSS-E-ST-10-02C; ECSS-E-ST-10-03C Rev.1 (2022) [7]; NASA GSFC-STD-7000 (GEVS) |
| Flight software | ECSS-E-ST-40C Rev.1 (2025) [8]; ECSS-Q-ST-80C Rev.2; MISRA C:2012; Holzmann "Power of 10" (2006) |
| TM/TC and time | CCSDS 133.0-B-2; CCSDS 301.0-B; ECSS-E-ST-70-41C (PUS) |
| EMC, magnetic cleanliness | ECSS-E-ST-20-07C; NASA SP-8018 |
| Model credibility | NASA-STD-7009B (2024) [9] |
| Monte Carlo for verification | Hanson & Beard, NASA TP-2010-216447 |
| Textbooks | Markley & Crassidis 2014; Wertz; Wertz, Everett & Puschell (New SMAD) 2011; Wie 2008; Hughes; Schaub & Junkins; Sidi; Montenbruck & Gill; Vallado |

**What is solid** (so the rest is read in proportion). The audits confirmed all of the following, numerically where noted:

- **Equations and conventions:**
  - The gyrostat equations, term by term.
  - The flex hybrid coordinates.
  - Quaternion conventions throughout.
  - RK4 at 0.1 s: |ΔH|/|H| = 2.6e-7 per orbit, 5th-order convergence.
  - Orbit RK4 at 10 s with Hermite output: 0.4 m per day against a 1 s step.
- **Estimation and control:**
  - The MEKF (Farrenkopf Q, Joseph update, correct χ²).
  - TRIAD, the q-method and the LQR solver (scipy CARE to 1e-9).
  - Every actuator-law sign.
- **Environment:** the IGRF synthesis, precession and GMST, and the disturbance-torque formulas.
- **Statistics:** the success-run count (1,109), Clopper-Pearson, and NaN-as-failure counting.
- **Physics of the fluid loop:** the empump physics.
- **Flight-software engineering:**
  - Static memory, and stack at 14 % of reserve.
  - CRC'd and fully validated parameters.
  - NaN-safe outputs.
  - C = Rust bit for bit.
  - 97.9 % mutation score where measured.

The defects found sit in the logic *around* these cores, and in what is judged and how.

## 2 · Findings by domain

Effort: S = days, M = 1–2 weeks, L = several weeks. "U*n*" is the phase in §3.

### 2.1 Orbit, time, frames, environment

| Id | Sev. | Finding | Reference | Fix | Effort | Phase |
|---|---|---|---|---|---|---|
| E1 | High | Twin not moved to the precession frame; parity blind to field direction (§0) | ECSS-E-ST-10-02C; NASA-STD-7009B | Port `prec_rot` to the twin; parity compares B, Sun and nadir *directions*; re-fly | S | U0 |
| E2 | High | No aberration of starlight, truth or onboard: annual 20.5″ + orbital 5.2″ ≈ 0.0071°, above the 0.005° imaging AKE; invisible because truth omits it too | ECSS-E-ST-60-20C Rev.2; IERS TN36 ch. 5 | Truth applies aberration to star vectors; onboard corrects with GNSS and Earth velocity | S–M | U1 |
| T5/E3 | High | Nutation, equation of the equinoxes, UT1−UTC and polar motion omitted both sides: up to about 25″ (0.007°) between the onboard Earth frame and the star tracker's J2000 | IERS TN36 §5; Vallado §3.7 | Truth on the IAU 2006/2000A build with EOP (already in `adcs-pop`); onboard truncated nutation plus UT1−UTC and pole by TC | S–M | U1 |
| E4 | Medium | Truth uses two Earth-fixed frames: POP gravity and drag without precession (J2 axis 0.15° off the 2027 pole, about 2 % SSO node-rate error, about 0.6° LTAN per month) while the field uses precession; an onboard comment says 0.4° | Montenbruck & Gill §5; IERS TN36 | Apply precession (or the IAU 2006 build) inside POP's GMST build | S | U1 |
| E5 | Medium | Field model error not represented: no crustal field, no external field (20–50 nT quiet, 100–500 nT storm, i.e. 0.1–0.9° direction), so the magnetometer measures exactly the onboard model | ECSS-E-ST-10-04C Rev.1 §5 | Truth-only error field (higher degree plus a Dst-driven external term), dispersed in Monte Carlo | M | U1 |
| E6 | Medium | Spec physics density is a static table: 15× high at solar minimum, 7.4× low on the DTM2020 peak at F10.7 = 250; the "worst aero" row understates 7.4×. The spec still names NRLMSIS | DTM2020; ECSS-E-ST-10-04C Rev.1 §8 | DTM2020 low/mean/high bundle from `adcs-pop` for every density row; "worst" uses high; spec text to DTM2020 | S | U0 |
| T4 | Medium | Gravity is zonal J2–J6 only; no field file ships; comments claim tesserals | EGM2008 (ICGEM) | Ship EGM2008 20×20; fix comments | S | U1 |
| D14/V5 | Medium | IGRF-13 at 2020.0, extrapolated past 2025 | IGRF-14 [6] | One IGRF-14 table for engine, both flight softwares and twin | S | U1 |
| E8 | Low | Environment held 1 s: B turns up to 0.21°/s, hidden only because the magnetometer samples on the refresh tick | — | Interpolate B between refreshes | S | U1 |
| T6/E9 | Low | Space weather constant in the loop; "high activity" keeps Kp = 2; no storm case | ECSS-E-ST-10-04C Rev.1 | Dated indices (CelesTrak/GFZ), storm scenarios, high-activity Ap per the standard | S | U1 |
| E7 | Low | Nadir is geocentric (up to 0.17° from geodetic); no Earth-fixed target tracking | Vallado §3.2 | Geodetic nadir; ground-target guidance | M | U3 |
| E10–E15 | Low | Dipole rows use R_E instead of IGRF's a (0.33 %); spherical-Earth eclipse; LTAN from the apparent Sun (±4° RAAN); albedo as a point at nadir with constants differing from comments; no epoch range guard in engine/FSW; no thermospheric wind | Knocke 1988; IERS | One fix each | S | U1 |

### 2.2 Dynamics, sensor and actuator models

| Id | Sev. | Finding | Reference | Fix | Effort | Phase |
|---|---|---|---|---|---|---|
| M1 | High | Failed fluid ring keeps its momentum (§0) | Hughes ch. 3 (free rotor), Poiseuille decay | Failed ring decays with −h/T_sd; re-fly the `mtq_fmr` fault campaign | S | U0 |
| M5 | Medium | `rotor_fail` on a CMG does nothing (zero friction): the rotor keeps h0 and the gimbal still turns | — | Bearing friction and spin-down | S | U0 |
| M2 | High | Wheel power leaves out copper loss I²R: up to +0.75 W per wheel; peak power understated up to about 150 % in slews | Motor model; SMAD power budgets | Add τ²·ω_nl/τ_stall; same for the VSCMG rotor | S | U0 |
| M3 | Med-high | Fluid ring loss is straight-pipe laminar only. 16 bends (K ≈ 0.5 each), Dean, entry and Hartmann losses are missing, so the real loss is about 1.5–2×; the shipped pump has about 7 % margin over the modelled loss; SYN-MFP-1 runs turbulent (Re 4025) | White, *Fluid Mechanics* ch. 6 | f(Re)·L/d + ΣK + Hartmann term; size the pump with stated margin; check Re in the sizer | M | U3 |
| M4 | Medium | Wheel friction compensation cancels the *dispersed* friction (a perfect compensator); the residual at fscale = 2 is 10× the modelled one | Olsson et al. 1998; Bialke | Compensate only nominal friction; move `friction_comp`, `eta`, `torque_noise` into the part | S | U3 |
| M6 | Medium | Gyro noise assumes a 10 Hz averaging output; the part says 100 Hz sampled (3.2× more noise if not filtered); no bandwidth or group delay | IEEE Std 952; M&C §4.7 | State output type and bandwidth in the part; model it | S | U3 |
| T8 | Medium | Gyro bias instability in parts, unused by the model | IEEE Std 952 | Gauss-Markov or flicker term fitted to the Allan curve | S | U3 |
| M7 | Medium | Star tracker error is bias + white noise only: no FOV-dependent low/high spatial frequency error, no thermo-elastic orbit-periodic bias; noise not marked 1σ or 3σ | ECSS-E-ST-60-20C Rev.2 error classes | LSFE/HSFE fields, periodic bias, 1σ in the schema | M | U3 |
| T7 | High (imaging) | Synthetic 4,000-star even catalogue; pair-angle identification | ECSS-E-ST-60-20C Rev.2 | Hipparcos/Tycho-2 catalogue, lost-in-space (pyramid), stray light | M | U3 |
| M8 | Med-low | Coil-to-magnetometer coupling is a scalar; physically a 3×3 matrix from geometry | Griffiths ch. 5; Wertz | Coupling matrix in the part | S | U3 |
| T10 | Medium | Wheel micro-vibration only as a post-run number | Masterson, Miller & Grogan 2002; ESSB-HB-E-003 | Harmonic disturbances in the dynamics | M | U3 |
| T12/M11 | Low-med | RCS: plain duty, no PWPF; valve power charged a whole tick (20× at the minimum impulse bit); only failed-closed valves, while stuck-open is the worse case; misalignment gives no net force | Wie (PWPF); ECSS-E-ST-60-30C FDIR | PWPF; stuck-open fault; force and CM torque; per-pulse power | S–M | U3 |
| T13 | Low | Propellant mass and inertia fixed; N2O slosh unbounded | Abramson NASA SP-106 | Mass update; pendulum slosh if the fill makes it matter | S | U3 |
| M9, M10 | Low | Star-tracker latency > 0.5 s silently becomes 0; GNSS `rate_Hz` ignored | — | Refuse or size history; honour the rate | S | U0 |
| M12–M17 | Low | CMG gimbal inertia terms dropped (about 0.25 %); no gimbal rate loop; failed wheel spin-down chatters; RK4 numerical damping 4–20 % of structural ζ; dispersion `dist` ignored and one shared stream (adding a device reshuffles all draws); S2 wrongly says FSW has no latency compensation (it has, star tracker) | Schaub, Vadali & Junkins 1998; Hughes ch. 12 | One fix each; correct S2 | S | U3 |

### 2.3 Onboard guidance, navigation and control

| Id | Sev. | Finding | Reference | Fix | Effort | Phase |
|---|---|---|---|---|---|---|
| F2/G3 | Critical | A silent gyro keeps feeding its last rate to the MEKF, the controllers and the detumble exit for 60 s; the star-tracker coast (900 s) then lets the attitude diverge with no fault. Demo: 143° error in 50 s, faults = 0 | ECSS-E-ST-60-30C (sensor validity before use) | On `!gyro_ok`: zero rate with inflated Q, end the coast, flag stale within ticks; stale limit < detumble hold | S | U0 |
| G1 | High | TRIAD start with isotropic P = (0.05 rad)² and sin θ ≥ 1e-3, plus a divergence counter reset by any accepted update: the magnetometer can be rejected forever while a wrong roll about the Sun line persists with small P | Shuster & Oh 1981; M&C §5.2, §6 | TRIAD covariance for P, sin θ ≥ 0.15–0.25, per-sensor rejection counters with re-init | S | U0 |
| G2 | Med-high | Rotor torque commands clipped per actuator: torque direction not kept (coils and gimbals already scale as a vector); no momentum-saturation check | Wie §7.3; M&C §7.7 | Scale the whole command vector; drop saturated rotors from allocation | S | U0 |
| G6 | Medium | PID gains on q_v: about 0.7× bandwidth (§0) | Wie §7.3 | Kp = 2Jω², Ki = 2·0.15·Jω³ (or act on 2q_v); re-tune; re-fly | S | U0 |
| G4 | Medium | No rate damping when attitude is invalid: pointing modes command ḣ = 0, coils-only modes do nothing; up to an eclipse of drift | ECSS-E-ST-60-30C; Wie §7; Sidi ch. 7 | Rate damping (gyro or B-dot) while `!ad_ok` | S | U0 |
| G5 | Medium | Safe mode can deadlock with zero actuation (magnetometer dead → coils off, rotors idle, exit blocked); no autonomous recovery | ECSS-E-ST-60-30C (safe mode, recovery) | Magnetometer-less safe mode on rotors and gyro; clear and re-arm on recovery | M | U3 |
| F5 | High | Without a gyro the detumble exit is judged on a zeroed rate, so it always exits after the hold | — | B-dot rate estimate for the exit, or refuse auto-next | S | U0 |
| F4 | High | Star-tracker *initialisation* skips the norm check (a CRC-valid \|q\| = 0.5 frame becomes identity with a confident P; demo: 180° error); unconfigured head can be selected | ECSS-E-ST-60-30C | Same plausibility as the update; require n_heads ≥ 1 | S | U0 |
| G7 | Med-low | VSCMG speed-equalisation not projected on the null space: about 8e-6 N·m per rotor on the body (10× environment); diagonal-only SR inverse can stall at elliptic singularities | Schaub, Vadali & Junkins 1998; Wie | Null-space projection; off-diagonal-ε SR inverse | S | U3 |
| G9 | Low | Tracking feed-forward misses A(q_e) and the ω×ω_r term | Schaub & Junkins ch. 8 | Full feed-forward | S | U3 |
| G10, S6 | Low | χ² gate 3 dof for a rank-2 vector measurement; star-tracker update ungated; no NEES test | Bar-Shalom; M&C eq. 7.39 | 2-dof gate or tangent-plane R; NEES/NIS test, then the gate | S | U2 |
| G11 | Low | B-dot gain uses orbital inclination and ×3; the reference uses magnetic inclination | Avanzini & Giulietti 2012 | Use ξ_m, justify the factor | S | U3 |
| G12 | Low-med | Sensor FDIR covers silence and \|B\| only (a 2× magnetometer scale fault passes); no stuck gyro, bias jump, gyro-vs-tracker check | ECSS-E-ST-60-30C | Innovation-based isolation per sensor | M | U3 |
| G13–G16, T11 | Low | No capture/PID hysteresis; rate low-pass lag (17° at 1 rad/s) uncounted; gyro bias lost on every re-init; Sun update only on the first tick of a cycle; q-method degeneracy unchecked; single-axis versine slews, no eigenaxis or keep-out cones | Wie ch. 7; Kjellberg & Lightsey 2016 | One fix each; eigenaxis and constrained slews | S–M | U3 |

### 2.4 Flight software as embedded software

| Id | Sev. | Finding | Reference | Fix | Effort | Phase |
|---|---|---|---|---|---|---|
| F1 | Critical | Onboard date = the blob's `jd0` + time since init: every reset rewinds the calendar (30 days later: Earth angle and Sun about 30° wrong, a healthy magnetometer rejected); a clock step backwards wraps to 585 years (undefined behaviour); the schedule replays after a reboot | ECSS-E-ST-70-41C service 9; CCSDS 301.0-B; ECSS-E-ST-40C Rev.1 §5.4 | Persistent onboard time from GNSS or a time TC; monotonic, bounded clock; absolute-time schedule | M | U0 |
| F3 | High | Rate groups from floating time: with ±1 µs jitter the fine law runs on 63 % of ticks and 25 % of coil cycles skip their measurement updates; invisible because SILS and QEMU pass exact ns | ECSS-E-ST-40C Rev.1 §5.4–5.5 | Integer tick counter for rate groups | S | U0 |
| F6 | High | Almost no sensor plausibility checks: gyro up to ±1280 rad/s into the MEKF, no Sun or Earth-sensor norm check, no GNSS upper bound or time tag, rotor h unchecked | MISRA Dir 4.14; ECSS-E-ST-60-30C | Range, norm and rate gates with fault counters | M | U4 |
| F7 | High | C HardFault hangs (`for(;;)`), Rust resets: the builds differ on the key failure path; a reset loses isolated faults, schedule and estimator; no watchdog, no persistent context, no fallback blob | ECSS-E-ST-60-30C FDIR hierarchy; SAVOIR | Watchdog; same fault handler both builds; CRC'd persistent context; golden blob | M | U4 |
| F8, T16 | High | TM/TC absent: one TC, no integrity, no acceptance or execution reports, no housekeeping or events (`tm_emit` never called), no parameter load, no FDIR reset | CCSDS 133.0-B-2; ECSS-E-ST-70-41C services 1, 3, 5, 8, 9, 20 | Space packets and PUS-C services | L | U4 |
| F9 | Medium | All arithmetic double in software on a single-precision FPU (685 `__aeabi_dmul` calls, 0 `vmul.f32`); timing evidence is observed, not WCET; CPI assumed | ECSS-E-ST-40C Rev.1 §5.8.3 | Board WCET/high-water mark; FPU choice (M7 double) or mixed precision | M | U5 |
| F10/G8 | Medium | Single-precision build broken (incompatible pointers, 278 conversion warnings) while docs say it is tested | — | Remove the claim or support it | S | U0 |
| F11, F12 | Medium | Loops bounded only by external input (a babbling CAN node holds the step); HAL errors on actuator writes ignored | Power of 10 rules 2, 7; MISRA 17.7 | Cap frames per tick; count and raise actuator write faults | S | U4 |
| F13 | Medium | Parameter schema identity is magic + length only; no cross-field rules (e.g. `bdot_k > 0`, which is divided by); update only by full re-init | ECSS-E-ST-70-41C service 20 | Schema hash, cross-field validation, in-flight patch with echo | M | U4 |
| F14 | Medium | No concurrency contract: a TC from another task changes the mode mid-step | ECSS-E-ST-40C computational model | Queue TCs, apply at tick start | S | U4 |
| F15, T14 | Medium | Measured: `adcs_fsw.c` (step, modes, timing) 48 % lines, 34 % branches; mutation excludes fsw, modes, fdir and drivers where F1–F5 live; fuzz runs ≤ 200 ticks with exact timestamps; no assertions; C-vs-Rust cannot find common-mode bugs (all of F1–F5) | ECSS-E-ST-40C Rev.1 §5.8.3.5; ECSS-Q-ST-80C Rev.2 §6.3.5 | Scenario tests (dropouts, reset, jitter, gyroless, long missions); mutation on all FSW; MC/DC (Clang ≥ 18) and MISRA (cppcheck) in CI; Frama-C/Kani on the GNC core | M | U4 |
| F16 | Medium | Build id is a constant string; ARM GCC unpinned; no `-Werror` on ARM; no reproducible-build check | ECSS-Q-ST-80C §5.6, §6.2 | Content-hash build id; pinned toolchain; reproducibility check | S | U4 |
| T15 | Medium | No single-event-upset injection; state and buffers unprotected | ECSS-Q-ST-60-15C | Bit-flip fault kind; scrubbing/checks | S–M | U4 |
| F17, F18 | Low | NULL blob in C is UB; `nmax` precondition; MISRA 13.5; 151 float `==`; covariance never symmetrised; determinism holds per platform libm only | MISRA C:2012 | One fix each; single libm (H3) | S | U4 |

### 2.5 Verification, statistics and performance assessment

| Id | Sev. | Finding | Reference | Fix | Effort | Phase |
|---|---|---|---|---|---|---|
| A1 | Critical | Closures "pass" without the stated probability or confidence (§0); NaN runs dropped from evidence | ECSS-E-ST-60-10C §4.2, §5; ECSS-E-ST-10-02C §5.2; Hanson & Beard | Pass only when CP_lower ≥ level with NaN as failure; else "not demonstrated (R ≥ x at 95 %)"; one verdict source for evaluate and trace; no pass on an infeasible design | S | U0 |
| A2 | Critical | Campaign ledger's validation claims false (§0); engine and twin populations differ (twin max 0.020°, engine heavy tail to 0.276°) | NASA-STD-7009B §4.4, §4.7 | Generate the text from the data; fly both on identical draws (≥ 100), two-sample test, trace the shift | M | U0 |
| A3 | Critical | "All differing verdicts traced": five are not | ECSS-E-ST-10-02C §5.2.5 | Cause table keyed by scenario and metric; print "untraced" | S | U0 |
| A4 | High | Pointing budget RSS-sums biases (alignment, thermal), so the room left is 0.0073° where linear summation leaves 0.0030° (2.4×); built from one run; gp_1 = √(APE² − AKE²) invalid for correlated terms; star-tracker bias, time-tag, ephemeris, aberration and nutation missing | ECSS-E-ST-60-10C Annex A/B; ECSS-E-HB-60-10A §5; ESSB-HB-E-003 §6 | Classify each term (bias, random, periodic) and sum per class; take the flown term from the campaign | M | U2 |
| A5 | High | Impossible inertias (§0) | Hanson & Beard §2.3 | Disperse R·diag·Rᵀ with bounded moments, or reject; realisable edge corners | S | U0 |
| A6 | High | The claim tests 99.73 % of runs of a temporal p99.73: neither ECSS ensemble, temporal nor mixed; cases state no interpretation or confidence; claim columns not shown | ECSS-E-ST-60-10C §4.2.3, §5.2 | `interpretation` and `confidence` per requirement; judge with the matching statistic; show CP_lower and claim_met | M | U2 |
| A7 | High | Judging windows: fine-hold judges half of one orbit, so tracker blinding in the other half is never scored (part of D3); slews leave 280–320 s unjudged | ECSS-E-ST-60-30C §5.2, §6 | One full orbit after acquisition plus an eclipse-transition window; slew APE from t0 + T + settle | S | U0 |
| T1 | High | No gain, phase, modulus or delay margins anywhere | ECSS-E-ST-60-10C; ECSS-E-ST-60-30C | Linearisation per mode; margins, Nichols; μ over the dispersions; include filter lag (G14) | M | U2 |
| T2 | High | No sensitivity analysis or worst-case search; failures not attributed (D15, S1) | Saltelli 2008; Hanson & Beard | Morris and Sobol; seeded worst-case search; keep failing runs (see Y6) | M | U2 |
| A8 | Medium | RPE, MPE and PDE on fixed blocks, not sliding windows; partial last block; separation rounded | ECSS-E-ST-60-10C §3.2, §4 | Sliding window or worst placement | S | U2 |
| A9 | Medium | Edge campaigns are not worst cases: random directions, unproved "all adverse", arg_lat low = high, 5 of 12 dispersions left out | ECSS-E-ST-60-30C §6.3 | Deterministic adverse directions; search (T2) | M | U2 |
| A10–A12 | Medium | `trace.py` verdicts from single nominal runs and mixed designs; units and levels never checked by the engine; `≥` KPIs would take the best tail | ECSS-E-ST-10-02C §5.2.4 | Verdict per selected configuration from the campaign; SI conversion with refusal; tail by sense | S | U2 |
| A13, T3 | Medium | Parity is verification, not validation; no credibility assessment | NASA-STD-7009B §4.4 | Credibility table; distribution comparison; a flown referent (V1, V2) | M | U2 |
| A14 | Medium | Detumble campaign runs 11,480 s for a 17,040 s requirement; 24-run pass shows 88 % at 95 %; tip-off range unsourced | ECSS-E-ST-60-30C §6 | Run past the requirement; size runs; tip-off from the deployer | S | U2 |
| A15–A18 | Low | Uniform dispersions without pedigree; three campaigns share a seed; no convergence plots; ensemble alignment by index; strictest level applied to unstated metrics; `power_peak` folds from f64::MIN | Hanson & Beard | One fix each | S | U2 |

### 2.6 System design and the design loop

| Id | Sev. | Finding | Reference | Fix | Effort | Phase |
|---|---|---|---|---|---|---|
| Y1 | Critical | Power judged on actuators only (§0) | ECSS-E-ST-60-30C; SMAD ch. 21 | Per-mode sensor and electronics power in the run; judge `req.pavg/ppk` on the total; check the static budget in `select` | M | U0 |
| Y2 | Critical | CMG cluster counted 4× (§0) | SMAD ch. 19 | One unit carrying the whole need | S | U0 |
| Y3 | Critical | Imaging mass "feasible" at 1.794/1.85 kg (3 % margin) is not robust: coil mass from an invented anchor (physics needs about 85 g, not 21 g); no electronics, drivers, harness or brackets; no mass growth allowance | ECSS-E-ST-10C margins; AIAA S-120A | Physical coil sizing; electronics/harness line; maturity-based growth allowance | M | U1 |
| Y4 | High | h_req = max(h_dist, h_slew), not the sum; the mass lever leaves 1.14× instead of the stated ×2 | SMAD §19; ECSS-E-ST-60-30C | Sum; floor the margin; report achieved margin | S | U0 |
| Y5 | High | Demand survey integrates torque in the body frame (no ω×h); orbit-rate transport (0.9e-6 N·m) exceeds the surveyed peak; cyclic/secular split wrong | Wertz §17/§19; Sidi ch. 7 | Integrate inertially | S | U0 |
| Y6 | High | Robustness node escalates authority without a did-it-help check (92 % → 83 % → still escalated to 2.32 kg), ignores the budget, and deletes the failing runs' evidence; a 10° APE miss is a gross failure, not an authority shortfall | — (sharpens D15) | Classify failures by cause; revert if no gain; respect the budget; keep failing runs; size by success-run | M | U2 |
| Y7 | High | Magnetic cleanliness unbudgeted; pumps declared dipole-free while each is an electromagnet (about 0.04 A·m² stray vs 0.01 A·m² total residual) | ECSS-E-ST-20-07C; NASA SP-8018 | Pump dipole model per state; residual-dipole budget; magnetometer placement | M | U3 |
| Y8 | High | Modes short of ECSS: no LEOP/separation, no slew-and-settle in the matrix (`req.settle/slew` bound in no mode), no dump mode, no explicit "no orbit control"; transitions unjudged | ECSS-E-ST-60-30C | Separation mode; slew mode in the matrix; transition matrix | M | U3 |
| Y9 | High | Thermal limits: galinstan freezes at −19 °C and expands; N2O tank goes liquid-full at about 33 °C and over MEOP above 36.4 °C; tank sized at MEOP with no MDP, proof or burst factor | ECSS-E-ST-32-02C; CubeSat Design Spec | Component temperature keys, heater power, tank at MDP with factors, launch-compliance row | M | U3 |
| Y10 | Medium | Rings encircle the 3U face while the case allots 1U; the RCS sphere is 100.4 mm in a 100 mm envelope | — | Geometric packaging check per part | M | U3 |
| Y11 | High | No FMECA; the single magnetometer is a single-point failure for detumble, dumping and safe mode (`mag_fail` excluded); one gyro | ECSS-Q-ST-30-02C (FMECA) | Functional FMECA tied to the fault set; redundancy trade per sensor | M | U3 |
| Y12, Y13 | Medium | Asymmetric power accounting (wheel standby always on, ring field duty-cycled); pump optimum sits on the grid bounds; driver efficiency 0.85 at 10 A unrealistic | — | Like-for-like trade; widen grid; realistic driver and harness losses | S–M | U3 |
| Y14, Y15 | Medium | Blank requirements filled with values against SPEC §8.3.2 (the invented 60 s slew sets τ_req 100× τ_dist); tip-off single 10°/s (deployers reach 20–30°/s); `req.wmax` blank while the tracker limit equals the slew peak; no temperature, cleanliness, LEOP-energy, availability or data keys | SPEC §8.3.2; ECSS-E-ST-60-30C | Refuse blanks; tip-off range; the missing keys | S–M | U1 |
| Y16 | Medium | No design group for thermal, EMC/cleanliness, structures/alignment, operations or AIT; empty CMG/VSCMG and sizing branches | — | Groups or rows for each | M | U3 |
| Y17, Y18 | Low | "Bang-bang" slew is actually cycloidal (57 % more torque, conservative, mislabelled); doc claims `k_tau` the code never raises; RCS detumble impulse ×2 undocumented | — | Correct labels and constants (with D10) | S | U1 |

## 3 · Phases

Each phase ends as 1.0.0 did:
1. Engine source changed.
2. Every stale run re-flown.
3. C = Rust parity.
4. `check_all` green.
5. Gap register, release notes and V&V report updated.
6. A tagged release.

Physics changes are batched per phase so the store is re-flown once.

### U0 · Correct what 1.0.0 states → **v1.1** (about 3–4 weeks)
Everything in §0, plus the critical and high logic defects that are days each:

| Group | Items |
|---|---|
| Verdicts | A1, A2, A3, A5, A7 |
| Power and sizing | Y1, Y2, Y4, Y5 |
| Models | M1, M2, M5, M9, M10 |
| GNC logic | F2/G3, G1, G2, G4, G6, F4, F5 |
| Flight software | F1, F3, F10/G8 |
| Twin and density | E1, E6 (DTM2020 everywhere) |

**Exit:**
- every verdict is demonstrated at its stated probability and confidence, or says "not demonstrated";
- power is whole-ADCS;
- the twin flies the flight frame;
- the gyro, reset and timing faults are covered by new scenario tests.

### U1 · Environment and requirements you can cite → **v1.2**

| Group | Items |
|---|---|
| Frames | E2 aberration, T5/E3 IAU 2006 frames with EOP, E4 |
| Fields | E5 field error, T4 EGM2008 20×20, IGRF-14 (D14, V5) |
| Environment details | E8, T6/E9, E10–E15 |
| References and sizing | sourced vectors for the 32 physics rows, V1 orbit reference ledger, Y3 mass with growth allowance |
| Requirements | Y14, Y15, Y17, Y18 |

**Exit:**
- every environment model names its source and has a published-vector test;
- every requirement is stated or refused, never invented.

### U2 · Margins, statistics and budgets → **v1.3**

| Group | Items |
|---|---|
| Stability and drivers | T1 margins and μ, T2 sensitivity and worst-case search |
| Statistics and budgets | A4 budget by error class, A6 ECSS interpretations, A8–A18, Y6 robustness node |
| Estimator checks | S6 + G10 NEES and gates |
| Credibility | T3/A13 credibility table |
| Remaining scope | D2, S1, S3 |

**Exit:** every mode reports margins, and every claim either passes with confidence or names its drivers.

### U3 · Devices, GNC and system completeness → **v1.4**

| Group | Items |
|---|---|
| Sensors | T7 real catalogue, M7 tracker error classes, T8 + M6 gyro |
| Actuators and dynamics | M3 ring losses, M4, M8, T10 micro-vibration, M11–M17, T12, T13 |
| GNC | G5 safe mode, G7, G9, G11–G16, T11 constrained slews, E7 |
| Calibration and FDIR | T9 magnetometer calibration, D6 + G12 FDIR |
| System | Y7 magnetic cleanliness, Y8 modes, Y9 thermal, Y10 packaging, Y11 FMECA, Y12, Y13, Y16 |

**Exit:** each device is checked against its datasheet, the mode set matches ECSS-E-ST-60-30C, and an FMECA drives the fault set.

### U4 · Flight-software assurance → **v1.5**

| Group | Items |
|---|---|
| Robustness | F6 plausibility gates, F7 watchdog, persistent context and golden blob, F11, F12 |
| TM/TC | F8/T16 CCSDS + PUS-C, F13 parameter service, F14 |
| Verification evidence | F15/T14 coverage, mutation on all FSW, MISRA, proofs |
| Build | F16 reproducible builds |
| Faults and hygiene | T15 SEU injection, F17, F18, H3 |

**Exit:** a software verification report in ECSS-E-ST-40C Rev.1 shape, generated by CI.

### U5 · Hardware → **v2.0** (hardware-paced; needs the owner's board and lab choices, H1 and H4)

| Group | Items |
|---|---|
| Board | Board OILS (H1), F9 WCET and FPU, H2 timing calibration |
| HILS, in the order ECSS-E-ST-10-03C Rev.1 and GEVS expect | polarity and phasing → Helmholtz-cage magnetometer/coil calibration → Sun-simulator CSS/FSS calibration → air-bearing closed loop → day-in-the-life against the engine |

**Exit:** each run is accepted against the same scenario in SILS, and the models' NASA-STD-7009B validation levels are raised.

## 4 · Open tools and data

| Tool or data | Use here | Licence |
|---|---|---|
| IGRF-14 coefficients (IAGA) | Field, truth and onboard | Public |
| DTM2020 (already in `adcs-pop`) | The one density model everywhere | As shipped with POP |
| EGM2008 (ICGEM) | Gravity truth, 20×20 | Public |
| IERS EOP and leap seconds | Frames and time | Public |
| ERFA | Reference vectors (already in tests) | BSD-3 |
| CelesTrak / GFZ space weather | Dated F10.7, Kp, ap | Public |
| Hipparcos, Tycho-2 (ESA) | Star catalogue | Public |
| Orekit | Orbit cross-validation (V1) | Apache-2.0 |
| Basilisk; NASA 42 | Attitude-dynamics cross-checks on shared scenarios | ISC; NASA Open Source Agreement |
| Clang/LLVM ≥ 18 MC/DC, cargo-llvm-cov | Structural coverage | Apache-2.0 with LLVM exception |
| cppcheck MISRA addon | MISRA C:2012 | GPL-3.0 (tool only) |
| Frama-C EVA; Kani | Run-time-error proofs (C; Rust) | LGPL-2.1; Apache-2.0/MIT |
| Yamcs or OpenC3 COSMOS | Ground side of CCSDS/PUS | AGPL-3.0 |

These supply data, cross-checks and evidence; none replaces the engine. TRI-NETRA's main strengths stay:
- two hand-written flight softwares (C99 and Rust) held to one pseudocode and bit-identical to each other;
- the physics and design relations translated from pseudocode and checked by its interpreter;
- the closed design loop;
- soft OILS with judged timing.

The full side-by-side comparison with Basilisk, NASA 42, NOS3, GMAT, Orekit, Tudat, MATLAB, PSS, STK/SOLIS, FreeFlyer, Renode, QEMU, cFS/F Prime, SMP/SIMULUS and Yamcs/COSMOS is in `docs/TOOL_COMPARISON.md`.

## 5 · Deliberately not proposed

- A UKF in place of the MEKF (the MEKF is correct and is the flight standard; NEES is the check).
- NRLMSIS (owner's decision: DTM2020).
- Gravity beyond 20×20, ocean tides and relativity in the loop (below ADCS sensitivity).
- Multi-body dynamics.
- Learning-based methods (research, not established practice).

## Sources

1. [ECSS-E-ST-60-30C, Satellite AOCS requirements (2013)](https://ecss.nl/standard/ecss-e-st-60-30c-satellite-attitude-and-orbit-control-system-aocs-requirements/)
2. [ECSS-E-ST-60-10C, Control performance (2008)](https://ecss.nl/standard/ecss-e-st-60-10c-control-performance/)
3. [ESA ESSB-HB-E-003, Pointing error engineering handbook (2011)](https://everyspec.com/ESA/ESSB-HB-E-003_19JUL2011_48282/)
4. [ECSS-E-ST-60-20C Rev.2, Star sensor terminology and performance specification (2019)](https://ecss.nl/standard/ecss-e-st-60-20c-rev-2-star-sensor-terminology-and-performance-specification-15-may-2019/)
5. [ECSS-E-ST-10-04C Rev.1, Space environment (2020)](https://ecss.nl/standard/ecss-e-st-10-04c-rev-1-space-environment-15-june-2020/)
6. [IGRF-14 (Nov 2024, valid to 2030)](https://en.wikipedia.org/wiki/International_Geomagnetic_Reference_Field)
7. [ECSS-E-ST-10-03C Rev.1, Testing (2022)](https://ecss.nl/wp-content/uploads/2022/05/ECSS-E-ST-10-03-Rev.1(31May2022).pdf)
8. [ECSS-E-ST-40C Rev.1, Software (2025)](https://ecss.nl/standard/ecss-e-st-40c-rev-1-software-30-april-2025)
9. [NASA-STD-7009B, Models and simulations (2024)](https://standards.nasa.gov/sites/default/files/standards/NASA/B/1/NASA-STD-7009B-Final-3-5-2024.pdf)
10. [Basilisk](https://hanspeterschaub.info/basilisk/); [NASA 42](https://software.nasa.gov/software/GSC-16720-1)

Textbooks and papers named in the tables are standard references; each enters `docs/references.toml` with its full
citation when its phase starts.
