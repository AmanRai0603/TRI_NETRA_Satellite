# Technical roadmap after 1.0.0: what the engine and flight software still lack, by standard

**In one line:** an end-to-end look at TRI-NETRA ADCS 1.0.0 against the ADCS standards, the textbooks and the open tools,
naming the gaps that no plan holds yet, what each one buys when closed, and the phases that take the engine and flight
software from 1.0.0 to 2.0.

This document adds to two others and does not repeat them: `docs/ADCS_GAPS.md` (the 31 gaps already recorded, with
ids D, S, V, H) and `docs/UPGRADE_PLAN.md` (Parts A and B). A gap here gets a new id **T*n*** only when neither of
those holds it; a gap they already hold is listed in §4 with the standard that settles it and the phase it lands in.
Research topics (new laws, new estimators, learning methods) are left out on purpose: everything here is established
practice with a public source.

## 1 · How this was checked

Every engine, twin and flight-software model was read and compared with:

| Area | Standard or reference |
|---|---|
| AOCS requirements and verification | ECSS-E-ST-60-30C, *Satellite AOCS requirements* (2013) [1] |
| Pointing and control performance, stability and robustness | ECSS-E-ST-60-10C, *Control performance* (2008) [2]; ECSS-E-HB-60-10A, *Control performance guidelines*; ESA ESSB-HB-E-003, *Pointing error engineering handbook* (2011) [3] |
| Star sensors | ECSS-E-ST-60-20C Rev.2, *Star sensor terminology and performance specification* (2019) [4] |
| Space environment | ECSS-E-ST-10-04C Rev.1, *Space environment* (2020) [5]; IGRF-14 (Nov 2024) [6]; NRLMSIS 2.1 (2022) [7]; IERS Conventions 2010 (TN 36) |
| Testing | ECSS-E-ST-10-03C Rev.1, *Testing* (2022) [8]; NASA GSFC-STD-7000 (GEVS) |
| Flight software | ECSS-E-ST-40C Rev.1, *Software* (2025) [9]; ECSS-Q-ST-80C Rev.2, *Software product assurance*; MISRA C:2012 |
| Model credibility | NASA-STD-7009B, *Models and simulations* (2024) [10] |
| Telemetry and telecommand | CCSDS 133.0-B-2 *Space Packet Protocol*; ECSS-E-ST-70-41C *Packet utilization standard* |
| Textbooks | Markley & Crassidis, *Fundamentals of Spacecraft Attitude Determination and Control* (2014); Wertz (ed.), *Spacecraft Attitude Determination and Control*; Montenbruck & Gill, *Satellite Orbits*; Vallado, *Fundamentals of Astrodynamics and Applications*; Wie, *Space Vehicle Dynamics and Control*; Hughes, *Spacecraft Attitude Dynamics* |
| Open reference tools | Basilisk (CU Boulder AVS Lab / LASP) [11]; NASA 42 (GSFC) [12]; Orekit; ERFA |

## 2 · Where 1.0.0 stands

Strong already, and better than most open ADCS tools:

- **Physics.** The orbit truth is a precision propagator with a DE440 Sun and Moon, DTM2020 or JB2008 density and a conical shadow. The rigid body carries up to 8 rotors and 4 gimbals with full gyroscopic coupling and one flexible mode.
- **Flight software.** Two implementations, C99 and no_std Rust, written from one pseudocode and bit-identical under differential fuzzing. They include an MEKF, eleven modes, singularity-robust CMG steering and a family of magnetorquer laws from the literature.
- **Verification.** Monte Carlo sized by the success-run theorem with Clopper-Pearson bounds, and ECSS-E-ST-60-10C pointing indices. Also edge campaigns, fault injection, mutation testing (97.9 %), and soft OILS on a QEMU Cortex-M4F.

What follows is what stands between that and a tool whose numbers an outside reviewer can accept for flight.

## 3 · Gaps no plan holds yet (new ids T1–T16)

Ranked by what closing them buys. "Effort" is S (days), M (one to two weeks), L (several weeks).

### Analysis the standards ask for

**T1 · No frequency-domain stability or robustness margins (high benefit, M).**
- *Today:* the only linear analysis is Floquet multipliers for the coils-only nadir loop (`tools/floquet.py`). Gain, phase, modulus and delay margins are computed for no loop.
- *Standard:* ECSS-E-ST-60-10C makes stability and robustness margins part of control performance [2]. ECSS-E-ST-60-30C asks for them in AOCS verification [1].
- *Closes it:* a linearisation tool for each controller × actuator × mode (plant, sensor delay, actuator lag and the flex mode included). It reports open-loop gain, phase, modulus and delay margins, and Nichols and Bode figures through `adcs-plot`. The structured-singular-value (μ) check then runs over the same dispersion ranges the Monte Carlo draws.
- *Benefit:* tells you before a 1,109-run campaign whether a design can meet its claim. Today S1 (no campaign meets 99.73 %) is found after the fact. It also gives the margins every design review asks for.

**T2 · No sensitivity analysis behind the Monte Carlo (high benefit, M).**
- *Today:* campaigns report pass rates but not which dispersion drives a failure. D15 and S1 are "study the failing draw" by hand.
- *Source:* global sensitivity with Morris screening and Sobol indices (Saltelli et al., *Global Sensitivity Analysis: The Primer*, 2008), and worst-case search seeded from the failing draws. Hanson & Beard (NASA TP-2010-216447) cover Monte Carlo used for requirement verification.
- *Benefit:* each failing claim names its top two or three drivers. That turns D15, S1 and D3 into targeted fixes instead of authority scaling, which is what pushed `ais_3u` to 2.32 kg.

**T3 · Model credibility not assessed (medium benefit, S).**
- *Today:* fidelity is described model by model, but no model carries a credibility level.
- *Standard:* NASA-STD-7009B grades each model on its factors, from verification to input pedigree to uncertainty characterisation [10].
- *Benefit:* the V&V report states, per result, how far it can be trusted. It is also the format reviewers outside the team expect.

### Truth environment

**T4 · The gravity field is zonal J2–J6 only (medium benefit, S).**
- *Today:* `Field::default_field()` (`engine/crates/adcs-pop/src/gravity/field.rs`) carries no tesseral or sectoral terms, and no real field file ships. The `.gfc` loader exists, but the gravity data folder is empty. Comments in the twin and in `results/ENGINE_PARITY.md` imply tesserals.
- *Closes it:* ship EGM2008 truncated to 20×20 from ICGEM; it is public. Fix the comments either way.
- *Benefit:* orbit truth at the tens-of-metres level over a day. That matters for GNSS-free navigation, for the onboard J2 propagator's error budget and for V1. Today's error is hidden because both sides share the same field.

**T5 · Nutation and polar motion are left out by both the truth and the flight software (high benefit for imaging, S–M).**
- *Today:* the in-loop frame is IAU-76 precession plus GMST on both sides (`engine/crates/adcs-sim/src/run.rs`, `fsw-rs/src/env.rs`). The IAU 2006/2000A CIO builds with EOP exist in `adcs-pop` but are not used in the loop.
- *Why it matters:* nutation reaches about 17″ in longitude and 9″ in obliquity, roughly 0.005°. That is the whole AKE requirement of `ais_img_3u`. It is invisible today because both sides make the same omission, and the star catalogue is synthetic (T7).
- *Closes it:* the truth runs the IAU 2006/2000A build with IERS EOP (IERS Conventions 2010). The flight software gets a truncated nutation series, about 0.1″ is enough, or the error goes into the pointing budget explicitly. Leap seconds come from the IERS file instead of the table that ends in 2017.
- *Benefit:* the 0.005° knowledge claim becomes true in an absolute frame, not only relative to the simulator's own frame.

**T6 · Space weather is a constant in the loop (low–medium benefit, S).**
- *Today:* F10.7 = 130 and Kp = 2 throughout a run. Monte Carlo disperses F10.7, but no run follows a dated history, although the OMNI2, GFZ and SWPC parsers exist.
- *Closes it:* dated indices from CelesTrak or GFZ, and storm scenarios such as May 2024. This also serves D7's date-by-date solar-cycle sweep.
- *Benefit:* drag torque and momentum dumping are judged on the storms a satellite actually meets.

### Sensors

**T7 · The star catalogue is synthetic (high benefit, M).**
- *Today:* 4,000 stars on a Fibonacci sphere, perfectly even. Real skies have sparse regions where a narrow FOV sees too few stars, and identification uses pair angles.
- *Standard:* ECSS-E-ST-60-20C Rev.2 defines the availability, accuracy and robustness terms (lost in space, tracking, stray light) [4].
- *Closes it:* a catalogue built from Hipparcos or Tycho-2 (ESA, public) to the head's magnitude limit. Add a lost-in-space identifier (pyramid, Mortari et al. 2004) and a stray-light and Earth-albedo background in the rendered frame.
- *Benefit:* real availability numbers. These feed D3 (the second head dropping out) and the gyro-coast budget, and the pointing budget gains a star-tracker term defined per ECSS.

**T8 · Gyro bias instability is not modelled (medium benefit, S).**
- *Today:* `bias_instability_rad_s` is in the parts but unused by the sensor model. The only drift terms are constant bias plus rate random walk.
- *Standard:* IEEE Std 952 (Allan-variance specification and test of fibre-optic gyros) and the Allan-variance noise terms in Markley & Crassidis ch. 4.
- *Closes it:* a flicker or Gauss-Markov bias term fitted to the datasheet Allan curve, plus an Allan-variance check of the simulated gyro against the datasheet.
- *Benefit:* the 900 s gyro coast during star-tracker outages is judged with the drift that dominates at those times. Today that drift is optimistic.

**T9 · No magnetometer calibration (medium benefit for AIS, M).**
- *Today:* `+comp/+magnetometer/calibrate.m` is a stub, and the flight software subtracts only a constant residual dipole. A 500 nT bias on a 20–50 µT field is roughly 1–1.5° of direction error, a large share of AIS's 10° APE.
- *Source:* TWOSTEP (Alonso & Shuster, *J. Astronaut. Sci.* 50(4), 2002) for on-ground and in-flight attitude-independent calibration; Markley & Crassidis ch. 5.
- *Benefit:* AIS pointing margin without new hardware. The same routine serves the Helmholtz-cage test in H4.

### Actuators and dynamics

**T10 · Wheel micro-vibration does not enter the dynamics (medium benefit for imaging, M).**
- *Today:* imbalance becomes an analytic jitter number after the run (`adcs-sim/src/metrics.rs` `jitter`). The harmonic forces and torques never reach the body, the estimator or the control loop.
- *Source:* the empirical wheel disturbance model of Masterson, Miller & Grogan (*J. Sound Vib.* 249(3), 2002), with harmonics of wheel speed, and the ESA pointing error engineering handbook's treatment of jitter [3].
- *Benefit:* RPE and jitter come from the flown run, so they include resonance with the flex mode. That needs D2's RPE budget.

**T11 · Slews are single-axis profiles with no attitude constraints (medium benefit, M).**
- *Today:* rotation about a fixed axis with a versine rate profile (`fsw-rs/src/guid.rs`). There is no eigenaxis computation between two arbitrary attitudes and no keep-out cone for the star tracker or a payload.
- *Source:* eigenaxis rest-to-rest slews (Wie, ch. 7); constrained reorientation (Kjellberg & Lightsey, *JGCD* 39(1), 2016).
- *Benefit:* agile imaging scenarios that do not blind the star tracker. A blinded tracker is a likely contributor to D3.

**T12 · Thruster modulation is plain duty with no PWPF (low–medium benefit, S).**
- *Today:* on-time = |τ|/τ_couple · T, quantised. Pulse-width pulse-frequency modulation, the standard for cold gas (Wie; Song & Agrawal), is not offered.
- *Benefit:* fewer pulses and lower propellant use near the minimum impulse bit. It addresses D12, the RCS detumble stalling at 0.45–0.7°/s, at its cause.

**T13 · Propellant mass and inertia are not updated (low benefit, S).** Propellant use is integrated, but mass, CM and inertia stay fixed. For an N2O system this also means liquid slosh is not bounded. A pendulum-equivalent model (Abramson, NASA SP-106) should be added if the tank fraction makes it more than a few percent of the inertia.

### Flight-software assurance

**T14 · No structural coverage, coding-standard or run-time-error evidence (high benefit before flight, M).**
- *Today:* mutation testing and fuzzing are strong. But no MC/DC or decision coverage is measured, MISRA is "a subset where a free checker covers it" (`spec/SPEC.md`), and absence of run-time errors is not proven.
- *Standard:* ECSS-E-ST-40C Rev.1 (Annex U, code verification) and ECSS-Q-ST-80C Rev.2 set coverage and code-verification evidence by criticality category [9]; MISRA C:2012.
- *Closes it:* MC/DC with Clang 18+ (`-fcoverage-mcdc`) on the C flight software and `cargo llvm-cov` on Rust; the cppcheck MISRA addon; Frama-C EVA (C) or Kani (Rust) proofs of no overflow or out-of-bounds in the estimator, control and allocation code.
- *Benefit:* the evidence an independent software V&V review asks for, produced by CI rather than by hand.

**T15 · Radiation effects on the flight software are not exercised (medium benefit, S–M).**
- *Today:* no single-event upset is injected. The parameter blob has a CRC, but MEKF state, mode and command buffers do not.
- *Standard:* ECSS-Q-ST-60-15C, *Radiation hardness assurance*; ECSS-E-ST-10-04C Rev.1 for the environment [5].
- *Closes it:* a bit-flip fault kind in the engine (state, parameters, stack) and checks or scrubbing in the flight software where a flip is not detected.
- *Benefit:* FDIR is tested against the fault a LEO satellite sees most often.

**T16 · TM/TC has no standard framing (medium benefit, part of B5.5).** The planned `fsw/tm` and `fsw/tc` should use CCSDS Space Packets and ECSS PUS-C services (housekeeping, events, function management), so ground tools such as Yamcs or OpenC3 COSMOS read them without custom decoders.

## 4 · Recorded gaps, with the standard that settles each

| Gap (ADCS_GAPS) | Standard or source that settles it | Phase here |
|---|---|---|
| D14, V5 IGRF-13 at 2020.0, extrapolated past 2025 | IGRF-14, valid to 2030 [6] | U1 |
| D14 static exponential density in the physics rows | NRLMSIS 2.1 [7] (port of the public Fortran) or the DTM2020 already in `adcs-pop` | U1 |
| D2 no RPE budget, no alignment or thermal terms | ECSS-E-ST-60-10C indices and summation rules [2]; ESSB-HB-E-003 [3] | U2 |
| S1 Monte Carlo claims not met | T1 margins + T2 drivers, then fix at the cause | U2 |
| S6 no NEES test, star-tracker gate off | NEES/NIS χ² consistency (Bar-Shalom et al., *Estimation with Applications to Tracking and Navigation*) | U2 |
| D6 no wheel or thruster FDIR | ECSS-E-ST-60-30C FDIR requirements [1]; T12 for the thrusters | U3 |
| S2 coil hysteresis, Dahl friction | Dahl (1968) friction; Jiles–Atherton hysteresis for cored rods | U3 |
| S3 no eclipse-transition scenario | ECSS-E-ST-60-30C performance in all modes and transitions [1] | U2 |
| V1 no precise-orbit reference | A real satellite's precise orbit (e.g. Swarm or GRACE-FO), cross-checked with Orekit | U1 |
| H1, H4 no board, no HILS | ECSS-E-ST-10-03C Rev.1 functional and performance tests [8]; §5 U5 | U5 |
| B5.5 TM/TC | CCSDS 133.0-B-2, ECSS-E-ST-70-41C (T16) | U4 |

## 5 · Phases

Each phase ends the way 1.0.0 did:
1. Engine source changed, every stale run re-flown, C = Rust parity.
2. `check_all` green, and the gap register and V&V report updated.
3. A tagged release.

The engine-source fingerprint means each phase's physics changes re-fly the store once, so physics changes are batched per phase.

### U1 · Truth you can cite → **v1.1**
- IGRF-14 in truth and onboard; one shared coefficient file for the engine, both flight softwares and the twin (D14, V5).
- EGM2008 20×20 shipped and used by the truth (T4).
- IAU 2006/2000A frames with IERS EOP and leap seconds in the truth, and nutation onboard (T5).
- Dated space weather; NRLMSIS 2.1 alongside DTM2020 and JB2008; Knocke-style albedo for the CSS and torques (T6, D14).
- Sourced test vectors for the 32 physics relations still without one (D14).
- A reference ledger against Orekit and a real precise orbit (V1).
- **Exit:** every environment model names its source and has a published-vector test; V1 and the D14 rows close.

### U2 · Margins, drivers and budgets → **v1.2**
- Linearisation and margins for every loop, Nichols figures and μ over the dispersions (T1).
- Morris and Sobol sensitivity and worst-case search (T2); then fix S1 and D15 at the driver they name.
- NEES/NIS test, then the star-tracker innovation gate (S6).
- RPE budget and the eclipse-transition scenario (D2, S3).
- NASA-STD-7009B credibility table in the V&V report (T3).
- **Exit:** every mode reports margins against stated thresholds, and every Monte Carlo claim either passes or names its drivers.

### U3 · Datasheet-grade devices and guidance → **v1.3**
- Real star catalogue, lost-in-space identification and stray light (T7).
- Allan-variance gyro (T8); TWOSTEP magnetometer calibration onboard and in the twin (T9).
- Wheel harmonic disturbances in the dynamics (T10); coil hysteresis and Dahl friction (S2).
- Eigenaxis and constrained slews (T11); PWPF and the RCS-to-B-dot hand-over (T12, D12).
- Wheel and thruster FDIR (D6); propellant mass and inertia update (T13).
- **Exit:** each device model is checked against its datasheet, and the imaging and agile scenarios run without tracker blinding by design.

### U4 · Flight-software assurance → **v1.4**
- MC/DC and decision coverage in CI, MISRA C:2012 checking, and run-time-error proofs on the GNC core (T14).
- Single-event-upset injection and scrubbing (T15).
- CCSDS/PUS TM/TC (T16, B5.5); watchdog and a single libm (H3).
- **Exit:** a software verification report in ECSS-E-ST-40C Rev.1 shape, generated by CI.

### U5 · Hardware → **v2.0** (hardware-paced; needs the owner's board and lab choices, H1 and H4)
- **OILS:** the chosen board (H1); calibrate the soft-OILS timing on it (H2).
- **HILS rig and its tests:** run in the order ECSS-E-ST-10-03C Rev.1 and GEVS expect [8]:
  - polarity and phasing of every sensor and actuator;
  - Helmholtz-cage magnetometer and coil calibration, with TWOSTEP from U3;
  - Sun-simulator CSS and FSS calibration;
  - closed loop on an air bearing;
  - a day-in-the-life run against the engine.
- Each run is accepted against the same scenario in SILS.
- **Exit:** the engine's models are validated against hardware, raising their NASA-STD-7009B validation levels.

## 6 · Open tools and data to use

| Tool or data | Use here | Licence |
|---|---|---|
| IGRF-14 coefficients (IAGA) [6] | Field in truth and onboard | Public |
| EGM2008 (ICGEM) | Gravity truth, 20×20 | Public |
| IERS EOP (finals2000A, C04) and leap seconds | Frames and time | Public |
| ERFA | Reference vectors for frames (already used in tests) | BSD-3 |
| NRLMSIS 2.1 (NRL) [7] | Density reference and model | Public source |
| CelesTrak / GFZ space-weather files | Dated F10.7, Kp, ap | Public |
| Hipparcos, Tycho-2 (ESA) | Star catalogue | Public |
| Orekit | Orbit cross-validation (V1) | Apache-2.0 |
| Basilisk [11] | Attitude-dynamics cross-check on shared scenarios | ISC |
| NASA 42 [12] | Second attitude-dynamics cross-check | NASA Open Source Agreement |
| Clang/LLVM 18+ MC/DC, cargo-llvm-cov | Structural coverage | Apache-2.0 with LLVM exception |
| cppcheck MISRA addon | MISRA C:2012 checking | GPL-3.0 (tool only, not linked) |
| Frama-C EVA; Kani | Run-time-error proofs (C; Rust) | LGPL-2.1; Apache-2.0/MIT |
| Yamcs or OpenC3 COSMOS | Ground side of CCSDS/PUS TM/TC | AGPL-3.0; commercial/AGPL |

None of these replaces the engine. They supply data, cross-checks and evidence. Porting the engine to Basilisk or 42 would throw away the C = Rust = interpreter chain that is TRI-NETRA's main strength.

## 7 · Deliberately not proposed

- **A UKF or other estimator in place of the MEKF:** the MEKF is the flight standard (Markley & Crassidis ch. 6), and no gap points at it. NEES (S6) is the check to do instead.
- **Gravity beyond 20×20, ocean tides, relativity in the loop:** below ADCS sensitivity; they stay available offline in `adcs-pop`.
- **Multi-body or deployable dynamics:** no product in the catalogue needs them yet.
- **Learning-based control or estimation:** research, not established practice.

## Sources

1. [ECSS-E-ST-60-30C, Satellite AOCS requirements (30 Aug 2013)](https://ecss.nl/standard/ecss-e-st-60-30c-satellite-attitude-and-orbit-control-system-aocs-requirements/)
2. [ECSS-E-ST-60-10C, Control performance (15 Nov 2008)](https://ecss.nl/standard/ecss-e-st-60-10c-control-performance/); [ECSS E-60 control performance training, with PEET (2023)](https://ecss.nl/wp-content/uploads/2023/10/03-ECSS-E-60-Control-Engineering-Control-Performance_with_PEET_2023-ESTEC.pdf)
3. [ESA ESSB-HB-E-003, Pointing error engineering handbook (2011)](https://everyspec.com/ESA/ESSB-HB-E-003_19JUL2011_48282/)
4. [ECSS-E-ST-60-20C Rev.2, Star sensor terminology and performance specification (15 May 2019)](https://ecss.nl/standard/ecss-e-st-60-20c-rev-2-star-sensor-terminology-and-performance-specification-15-may-2019/)
5. [ECSS-E-ST-10-04C Rev.1, Space environment (15 Jun 2020)](https://ecss.nl/standard/ecss-e-st-10-04c-rev-1-space-environment-15-june-2020/)
6. [International Geomagnetic Reference Field, IGRF-14 (Nov 2024, valid 1900–2030)](https://en.wikipedia.org/wiki/International_Geomagnetic_Reference_Field)
7. [NRLMSIS 2.1, Emmert et al., JGR Space Physics 127(10), 2022](https://research.chalmers.se/en/publication/532920)
8. [ECSS-E-ST-10-03C Rev.1, Testing (31 May 2022)](https://ecss.nl/wp-content/uploads/2022/05/ECSS-E-ST-10-03-Rev.1(31May2022).pdf)
9. [ECSS-E-ST-40C Rev.1, Software (30 Apr 2025)](https://ecss.nl/standard/ecss-e-st-40c-rev-1-software-30-april-2025)
10. [NASA-STD-7009B, Standard for Models and Simulations (5 Mar 2024)](https://standards.nasa.gov/sites/default/files/standards/NASA/B/1/NASA-STD-7009B-Final-3-5-2024.pdf)
11. [Basilisk astrodynamics simulation framework](https://hanspeterschaub.info/basilisk/)
12. [NASA 42, GSC-16720-1](https://software.nasa.gov/software/GSC-16720-1)

Textbooks and papers cited by name above (Markley & Crassidis 2014; Wertz; Montenbruck & Gill; Vallado; Wie; Hughes; Alonso & Shuster 2002; Masterson, Miller & Grogan 2002; Mortari et al. 2004; Kjellberg & Lightsey 2016; Saltelli et al. 2008; Hanson & Beard 2010; Abramson SP-106; Bar-Shalom et al.; Dahl 1968) are standard references; each enters `docs/references.toml` with its full citation when its phase starts.
