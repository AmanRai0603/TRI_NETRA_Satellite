# POP, the orbit propagator: what it can do, what the loop uses, and what is missing

**In one line:** an audit of TRI-NETRA's precision orbit propagator (Rust `engine/crates/adcs-pop`, MATLAB
`matlab_sils/pop`). It covers every capability it has, which ones the ADCS loop actually uses, what each simplification
costs (measured), how far it is validated, how it compares with Orekit, GMAT and STK HPOP, and the findings P1–P13.

Companion to `docs/TECHNICAL_ROADMAP.md` (E1–E15, T4, T5), `docs/TOOL_COMPARISON.md` §1 and `docs/ADCS_GAPS.md` (V1, D14).
Density is DTM2020 throughout (owner's decision).

## 1 · What POP implements

POP is much richer than what the loop uses:

| Area | Implemented | Data shipped? |
|---|---|---|
| Gravity | Normalised spherical harmonics of any degree, zonal fast path, ICGEM `.gfc` loader | Only the J2–J6 zonal default (EGM2008 GM, Re); no `.gfc` file |
| Third body | Sun and Moon: Battin (default), direct, tidal, Legendre to degree n; Lidov–Kozai secular model | DE440 |
| Solar radiation | Cannonball or box-wing with Sun-tracking arrays; eclipse cylindrical, conical (penumbra) or "fine" (oblate Earth + 12 km atmosphere) | DE440 |
| Earth radiation | Knocke 1988 albedo and IR; simple, CERES-grid, box-wing | Embedded |
| Relativity | Schwarzschild, Lense–Thirring, de Sitter (IERS 2010 ch. 10) | — |
| Tides | Solid (IERS 2010 step 1 + frequency-dependent); ocean, 8 lines (shipped FES2004 table not wired in) | Partial |
| Drag | Cannonball, or free-molecular panels Sentman/DRIA/SESAM/CLL with multi-species composition; box-wing; co-rotating atmosphere; wind hook | — |
| Atmosphere | **DTM2020** operational (F10.7 + Kp) and research (F30 + ap60), JB2008, exponential | DTM2020 tables; JB2008 index files to 2026-05-17 |
| Space weather | Manual indices; dated table lookup with lag options; parsers for OMNI2, GFZ Kp/Hpo, SWPC cycle and 45-day forecasts | Parsers only; no files cached |
| Time | UTC, TAI, TT, TDB, UT1, GPS; GMST82; leap seconds (table ends 2017) | Embedded |
| Frames | `Gmst` (R3(GMST), no precession or nutation) and full IAU 2006/2000A CIO builds A/B/C with finals2000A, EOP 20 C04, sub-daily tidal EOP | XYS06 series ship; EOP files only as test data |
| Ephemerides | Native SPK type-2 reader; DE440s | `de440s.bsp` ships |
| Integrators | RK4, RK45 (DP), RK78 (adaptive), RK6, Nyström 4, Gauss–Jackson 8, Hermite dense output | — |
| Loop interface | `World`, `InLoop` (RK4 nodes + Hermite), `sso_initial` | — |

**MATLAB only (not ported to Rust):**
- TLE parsing and an SGP4 reference (needs an external SGP4);
- SP3 reader;
- ITSG/TU Graz orbit-determination validation;
- TU Delft density comparison;
- live data fetch.

**Neither has:** CCSDS OEM/OPM I/O, manoeuvres, orbit determination, lifetime/decay prediction.

## 2 · What the loop uses, and what each simplification costs

The loop (`engine/crates/adcs-sim/src/run.rs:29-38`) builds POP with:
- the `Gmst` frame;
- zonal J2–J6;
- Battin Sun and Moon;
- cannonball DTM2020 drag at F10.7 = 130, Kp = 2;
- cannonball SRP with conical eclipse.

It steps with RK4 at 10 s nodes.

**Measured, one day:** 550 km SSO, epoch 2025-01-10, 3U at 4 kg, against converged RK78.

| Change from the loop setting | Max \|Δr\| over a day |
|---|---|
| Loop RK4 at 10 s vs RK78 | 0.43 m |
| RK4 at 30 s / 60 s | 67 m / 1.87 km |
| `Gmst` → IAU 2006/2000A + EOP | 2.29 km (1.9 km cross-track) |
| + tesseral terms to degree 4 | 12.0 km |
| F10.7 130/130 → the day's actual 157/186 | 0.97 km |
| DTM2020 → JB2008 | 0.49 km |
| Relativity + solid tides | 25 m |
| Conical → "fine" eclipse | 0.09 m |

**Frame:** POP's `Gmst` build is 1,259″ (2025) to 1,360″ (2027) from IAU 2006 + EOP, and its J2 pole is 0.15° off. This confirms E4.

**Density along the orbit:**
- JB2008/DTM2020 ratio 0.88–1.67 (mean 1.30).
- Fixed 130/130 vs the day's indices: ×0.50.
- The exponential model is right on average but has no diurnal bulge, so its torque is out of phase (0.43–1.58×).

**SSO start:** from osculating elements, the node drifts 0.0027°/day, about 1° of LTAN per year.

**What matters for ADCS.** The loop is self-consistent: GNSS measures the truth orbit and the field is evaluated there. So kilometres of orbit error change no pointing verdict; they make the environment less realistic. What *does* reach ADCS results:
- **Density level:** aero torque is linear in it; the fixed indices halve it on an active day.
- **Frame consistency:** E3/E4.
- **The step limit:** `config.rs` accepts orbit steps up to 600 s with no warning.

## 3 · Validation status

| Checked against | What |
|---|---|
| **External** (ERFA/SOFA vectors) | ERA, fundamental arguments, X/Y/s06, c2t06a, pom00, calendar, leap seconds (`tests/published.rs`) |
| **External** (pyIGRF) | IGRF-13 (outside POP) |
| Parity with the MATLAB original only | Time and frame builds, tidal EOP, ephemerides, third body, SRP and eclipse, ERP, relativity, gravity and tides, DTM2020, JB2008, GSI drag, the in-loop trajectory |
| **Nothing** | **Integrators: no test calls `integ::`** |

Parity with the MATLAB original cannot find errors both share. The MATLAB orbit-determination validation against real
ITSG products is documented but records no result.

**External validation campaign (closes V1):**
1. **Precise orbits.** One-day arcs of GRACE-FO and Swarm A/B/C (kinematic orbits are force-model independent). Batch fit of the initial state; EGM2008 70×70, IAU 2006 build B, DTM2020 with dated indices, estimated Cd. Target: tens of metres per day in RTN.
2. **Densities.** TU Delft accelerometer densities against DTM2020: mean ratio and spread per activity bin, compared with the published DTM2020 assessment.
3. **Orekit cross-run.** Forces switched on one at a time on a shared density. Target: accelerations to about 1e-12 relative, trajectories to cm/day, GCRF→ITRF to under 1 mas.
4. **CCSDS OEM exchange.** POP → Orekit/GMAT (CCSDS 502.0-B-3), differenced.
5. **Vallado cases.** Frame reduction (ch. 3); the SGP4 verification set (AIAA 2006-6753) once SGP4 exists.
6. **Model reference outputs.** DTM2020 Fortran test outputs; JB2008 driver output; DE440 against Horizons/SPICE.
7. **Integrators.** Kepler and J2 closed forms; energy drift; Gauss–Jackson against RK78.

## 4 · POP compared with Orekit, GMAT and STK HPOP

| Capability | POP | Orekit | GMAT | STK HPOP |
|---|---|---|---|---|
| Gravity | Any degree; only J2–J6 ships | SH + time-variable; ICGEM/SHM/EGM loaders | SH (EGM96, JGM, user files) (u) | SH up to EGM2008 (u) |
| Atmosphere | **DTM2020** oper/research, JB2008, exponential | DTM2000, JB2008, NRLMSISE-00, Harris–Priester | Jacchia–Roberts, MSIS (u) | Jacchia–Roberts, DTM2012, MSIS, others |
| Space-weather files | Parsers only; not wired into `World` (P1) | CSSI/CelesTrak, MSAFE loaders | CSSI + Schatten (u) | CSSI (u) |
| Surface forces | Cannonball, box-wing, **GSI panels** | Box-and-wing drag/SRP, lift | Spherical, SPAD (u) | Spherical, N-plate, SPAD (u) |
| Frames/EOP | IAU 2006/2000A A/B/C incl. sub-daily, **unused in the loop** | IERS 2003/2010 | IAU 2000A/1976 (u) | IAU 2006 + EOP (u) |
| Integrators | RK4/45/78, RK6, Nyström, GJ8 (untested) | DP853, GBS, Adams… | RK89, PD78, ABM, BS (u) | RKF78, RKV89, BS, GJ (u) |
| SGP4/analytic | None in Rust | SGP4/SDP4, Brouwer–Lyddane, DSST | Ephemeris propagators (u) | SGP4 propagator |
| Orbit determination | None | Batch LS, Kalman, UKF | Batch LS, EKF | ODTK (separate) |
| Manoeuvres | None | Impulsive, constant thrust | Impulsive, finite burns | Astrogator |
| OEM/OPM | None | CCSDS ODM read/write | OEM output (u) | OEM import/export (u) |
| Lifetime | None | By propagation (u) | (u) | Lifetime tool (u) |
| Coupled inside an attitude loop | **Yes** | No | Kinematic attitude only | No |

(u): from documentation memory, not verified this session.

POP is ahead on GSI panel drag and DTM2020 (STK stops at DTM2012). It is behind on shipped data, on space-weather
plumbing, and on every operations interface: SGP4, OEM, orbit determination, manoeuvres, lifetime.

## 5 · Findings

| Id | Sev. | Finding | Evidence | Fix | Effort | Phase |
|---|---|---|---|---|---|---|
| P1 | Medium | `World` feeds drag only the manual indices: JB2008, DTM2020 research and dated DTM2020 are unreachable from the loop, which blocks T6/E9 although the parsers exist | `adcs-pop/src/accel.rs:140` (`SwSources { manual: …, ..Default::default() }`) | Add table, JB and research drivers to `World` | S | U1 |
| P2 | Medium | `Gmst` build: Earth frame 1,259–1,360″ off, J2 pole 0.15°; 2.3 km/day (quantifies E4) | `run.rs:32`; `frames/mod.rs:188-197` | Use build A with the shipped XYS06 + EOP (cost 5.4 s per RK78 day; once per node in the loop) | S–M | U1 |
| P3 | Medium | On the dated-table path DTM2020 gets the daily-mean Kp in both the 3-h-delayed and 24-h-mean slots, and same-day F10.7 by default, so storm onsets are smoothed by up to a day | `spaceweather.rs:274`, `:238` | Build the 3-hourly Kp input; lag F10.7 by default per the DTM2020 specification | S | U1 |
| P4 | Medium | None of the six integrators has a test | no `integ::` in `adcs-pop/tests` | Closed-form Kepler/J2 and energy-drift tests; GJ8 vs RK78 | S | U1 |
| P5 | Low–med | RK4 error grows steeply with step (0.43 m at 10 s → 1.87 km at 60 s); 600 s accepted | `config.rs:609` | Cap at 30 s, or RK78 nodes | S | U0 |
| P6 | Medium | Fixed F10.7 halves density on an active day; the survey's F10.7 = 250 keeps Kp = 2 (quantifies E6, T6/E9) | `config.rs:678`; `adcs-design/src/lib.rs:116` | Dated indices; ECSS high-activity Ap in the survey | S | U1 |
| P7 | Low–med | Tesserals to degree 4 change the day's position by 12 km (mostly a 60 m semi-major-axis offset); irrelevant to attitude, relevant to ground track and operations (quantifies T4) | — | Ship EGM2008 20×20 | S | U1 |
| P8 | Low | Hot-path clone of `Forces` per evaluation; O(n²) dense output; `earth_rate_eci` silently falls back to [0, 0, ω] on a frame error | `accel.rs:118`; `integ.rs:362`; `frames/mod.rs:221` | One fix each | S | U1 |
| P9 | Low | Leap seconds hard-coded through 2017 with no expiry guard; EOP extrapolation flag never shown | `time.rs:90` | Leap-second file with expiry check; show the flag | S | U1 |
| P10 | Low | Orbit drag uses a fixed cannonball while attitude torques use facets; `set_attitude` has no effect | `run.rs:75` | Pass the facet set | S | U1 |
| P11 | Low | Ocean tides 8 lines, shipped FES2004 unused; solid tides by finite difference (all well below ADCS relevance) | `oceantides.rs` | Wire FES2004 or remove | S | later |
| P12 | Low | SSO starts from osculating elements: about 1° LTAN per year | `accel.rs` `sso_initial` | Start from mean elements (Brouwer / Eckstein–Ustinov) | S–M | U1 |
| P13 | Gap | MATLAB validation and data tooling not ported (SP3, ITSG metrics, TU Delft densities, TLE seed); the Rust crate cannot run V1 | `matlab_sils/pop/09_docs/OD_VALIDATION.md` | Port the readers needed by the campaign (§3) | M | U2 |

### Missing capabilities for ADCS and operations

| Capability | Benefit | Effort | Phase |
|---|---|---|---|
| Dated space weather (P1 + P3), CelesTrak/GFZ files, storm scenario | Correct aero-torque envelopes | S | U1 |
| EOP files and IAU 2006 build in the loop | Arcsec-consistent frames (E3/E4) | S–M | U1 |
| TLE/SGP4 import (Vallado SGP4, TEME → GCRF) | Start from launch-provider TLEs; compare with NORAD | M | U3 |
| CCSDS OEM/OPM import and export | Exchange with ground segment, Orekit/GMAT/STK, validation | S–M | U2 |
| Orbit determination: batch least squares on GNSS fixes or SP3 | Calibrate Cd·A/m; reference for the onboard navigator | L | later |
| Manoeuvres: impulsive and finite burns with mass flow | Thruster families, collision avoidance | M | U3 |
| "Fine" eclipse (oblate + atmosphere), already implemented | Eclipse entry within seconds (E11) | S | U1 |
| Lifetime/decay prediction with DTM2020 and the SWPC cycle forecast (already parsed) | Grounds `mission.life` (now defaulted to 3 years) and debris-rule compliance | M | U3 |
| External validation campaign (§3) | Turns V1 into cited accuracy | M–L | U2 |

## Sources

- Orekit features and atmosphere package: https://www.orekit.org/site-orekit-latest/apidocs/org/orekit/models/earth/atmosphere/package-summary.html
- STK HPOP drag models: https://help.agi.com/stk/Content/hpop/hpopDrag.htm
- GMAT V&V: https://gmat.atlassian.net/wiki/download/attachments/380273226/V%26V%20Presentation%20-%20Final.pdf?api=v2
- ERFA test vectors (github.com/liberfa/erfa, `t_erfa_c.c`); IERS Conventions 2010 (TN 36)
- Montenbruck & Gill, *Satellite Orbits* (2000); Vallado, *Fundamentals of Astrodynamics and Applications*, 4th ed.; Vallado et al., AIAA 2006-6753 (SGP4)
- Bruinsma & Boniface, DTM2020, *Space Weather and Space Climate* (2021); Bowman et al., JB2008, AIAA 2008-6438
- CCSDS 502.0-B-3 (Orbit Data Messages); ICGEM (EGM2008); ITSG-Grace / TU Graz; ESA Swarm SP3; TU Delft thermosphere densities
