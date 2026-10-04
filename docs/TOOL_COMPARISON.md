# TRI-NETRA ADCS compared with established GNC simulation tools

**Scope and method.** TRI-NETRA's state comes from the repository: `docs/TECHNICAL_ROADMAP.md` (audit ids E/M/G/F/A/Y/T), `docs/ADCS_GAPS.md` (D/S/V/H), `docs/UPGRADE_PLAN.md`, `docs/ARCHITECTURE_PLAN.md`, `docs/SOFT_OILS.md`, `docs/OILS_HILS.md`, `docs/PSEUDOCODE_V2.md`, `docs/GENERATORS.md` and `README.md`. External tools were checked against official repos, docs, vendor pages or papers, and each claim has a numbered source [n]. Several official sites (hanspeterschaub.info, orekit.org, mathworks.com, help.agi.com, ntrs.nasa.gov, antmicro.com, readthedocs) could not be fetched from this sandbox. For those, claims rest on search-indexed text from the same official pages; they are cited, not paraphrased from memory. **(u)** marks anything I could not confirm from a source; treat it as unverified.

**Legend:** **Y** = yes, **P** = partial, **N** = no, **—** = out of scope for the tool. Column keys: **TN** TRI-NETRA 1.0.0; **BSK** Basilisk; **42** NASA 42; **NOS3** NOS3 (bundles 42, cFS/F´ and COSMOS/Yamcs, so it also covers OpenSatKit's cFS+42+COSMOS pattern [17]); **GMAT**; **ORE** Orekit; **TUD** Tudat; **MW** MATLAB/Simulink Aerospace Blockset, CubeSat Simulation Library and companion toolboxes; **PSS** Princeton Satellite Systems Spacecraft Control Toolbox; **SOL** STK + SOLIS; **FF** FreeFlyer.

---

## 1. Orbit propagation and environment

| Capability | TN | BSK | 42 | NOS3 | GMAT | ORE | TUD | MW | PSS | SOL | FF |
|---|---|---|---|---|---|---|---|---|---|---|---|
| High-fidelity orbit propagator | Y: POP Cowell RK4 in loop, 0.4 m/day vs 1 s step | Y: gravityEffector, drag, SRP [2] | Y: 2-/3-body, solar system [5] | Y via 42 | Y: harmonics, drag, tides, relativity [10] | Y [11] | Y [14] | Y: Spacecraft Dynamics + EGM2008 [19] | Y [23] | Y (STK) | Y [27] |
| Gravity field | P: zonal J2–J6 only, no tesserals (T4) | Y [2] | Y (u: degree) | Y via 42 | Y [10] | Y | Y [14] | Y: EGM2008 SH [19] | Y (u) | Y | Y |
| Atmosphere density | P: DTM2020 only (owner's choice); the twin's POP also has JB2008 and NRLMSISE | Y: MSIS [3] | Y: NRLMSISE-00 [6] | Y via 42 | Y (u: model list) | Y: DTM2000, JB2008, NRLMSISE-00, Harris-Priester [11] | Y (u: model list) | Y: NRLMSISE-00 [20] | Y [23] | Y | Y (u) |
| Geomagnetic field | P: IGRF-13 deg 13, extrapolated past 2025 (D14/V5); no crustal or external field (E5) | Y: WMM [3] | Y: IGRF + dipole [6] | Y via 42 | — | Y: IGRF + WMM [12] | (u) | Y: IGRF-14 block [21] | Y [23] | Y (u) | (u) |
| Ephemerides | Y: DE440 Sun/Moon | Y: SPICE (u) | Y: planets and moons [5] | Y via 42 | Y: SPICE (attitude CK) [10] | Y | Y | Y (u) | Y | Y | Y |
| Frames / EOP | P: GMST × IAU-76; no nutation, UT1−UTC or polar motion in the loop (T5/E3); EOP exists only in `adcs-pop` | (u) | (u) | (u) | Y | Y: IERS 2010, EOP interpolation [12] | Y | Y (u) | (u) | Y | Y |
| Space-weather data handling | P: constant indices in the loop; no dated files, no storms (T6/E9); the survey sweeps F10.7 65/250 | (u) | (u) | (u) | (u) | Y: CSSI/CelesTrak 3-hourly F10.7/Ap/Kp loader [12] | (u) | Y: solar-flux and geomagnetic-index block [20] | (u) | (u) | (u) |
| Disturbance torques | Y: per-facet aero and SRP, GG, residual dipole, albedo | Y: facetDrag, SRP, GG, MTB [2] | Y | Y via 42 | N: attitude is kinematic [15] | N: no attitude dynamics | Y: aero, SH gravity, 2nd-degree GG [14] | Y (u) | Y [23] | Y [24] | N (u) |

**Reading.** For orbit and environment fidelity, Orekit, GMAT, Tudat and the MATLAB environment blocks all go beyond TRI-NETRA in at least one way: tesseral gravity, IERS-2010 frames with EOP, IGRF-14, and dated space-weather ingestion. Nearly all of these are already TRI-NETRA's own U1 items. What TRI-NETRA has that the orbit tools lack is the precision orbit coupled *inside* the attitude loop, together with per-facet torques.

## 2. Attitude dynamics

| Capability | TN | BSK | 42 | NOS3 | GMAT | ORE | TUD | MW | PSS | SOL | FF |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Rigid body + RK4/variable step | Y: momentum conserved 2.6e-7/orbit | Y [2] | Y [5] | Y via 42 | N: kinematic [15] | N | Y [14] | Y [19] | Y [23] | Y [24] | N: kinematic [27] |
| Multi-body | N (deliberately) | P: hinged bodies, N-hinged [2] | Y: tree topology, rot/trans joints [5] | Y via 42 | N | N | N (u) | P (u) | Y [23] | (u) | N |
| Flexible | P: one mode, hybrid coordinates | Y: hinged panels, spring-mass-damper [2] | Y [5] | Y via 42 | N | N | N | (u) | Y (u) | (u) | N |
| Fuel slosh | N: decided not needed (0.05 kg propellant) | Y: FuelTank, spherical pendulum, LSMD [2] | (u) | (u) | N | N | N | (u) | (u) | (u) | N |
| Wheels / momentum devices | Y: wheels, fluid rings, rotor coupling | Y [2] | Y [7] | Y via 42 | N | N | N | P (u) | Y | Y [24] | N |
| CMG / VSCMG | Y: SGCMG, VSCMG pyramids (gimbal inertia dropped, M12) | Y: VSCMG effector [2][4] | Y: gimbal drives [7] | Y via 42 | N | N | N | (u) | Y (u) | (u) | N |
| Thrusters | Y: 6 cold-gas couples, MIB, no PWPF (T12) | Y [2] | Y: pulsed + proportional, plume [7] | Y via 42 | P: finite burns (translational) [10] | P: manoeuvres | Y: thrust [14] | (u) | Y | Y | P: manoeuvres |
| Mass/propellant variation | N: fixed (T13) | Y (u) | (u) | (u) | Y | Y | Y: mass propagation [14] | Y (u) | Y | Y | Y |

## 3. Sensor and actuator models

| Capability | TN | BSK | 42 | NOS3 | MW | PSS | SOL |
|---|---|---|---|---|---|---|---|
| Star tracker, quaternion level | Y: 2 heads, latency, Sun/Earth/Moon exclusion; bias + white noise only (M7) | Y [3] | Y: NEA, Sun/Earth/Moon exclusion [6] | Y via 42 | (u) | Y (u) | Y [24] |
| Star tracker, image level | P: twin-only render → centroid → pair-angle ID → QUEST; synthetic 4,000-star catalogue (T7, S5) | P: camera module + Vizard image stream [3][9] | P: SatCam/optics (u) | N | (u) | N (u) | N (u) |
| Sun sensors (CSS/FSS) | Y: 6 CSS with albedo, FSS | Y: CSS [3] | Y: CSS with albedo, FSS with NEA [6] | Y via 42 | (u) | Y | Y |
| Magnetometer | Y: scale, misalignment, coil coupling (scalar, M8) | Y [3] | Y: noise, quantisation, saturation [6] | Y | (u) | Y | Y |
| Gyro and Allan terms | P: ARW, RRW, bias; bias instability unused (T8); bandwidth not modelled (M6) | Y: IMU (u: Allan terms) | P: ARW, correlated bias, quantisation [6] | Y via 42 | (u) | Y (u) | Y |
| GNSS | P: noise; rate ignored (M10) | P: simpleNav (u) | Y: pos/vel/time noise [6] | Y | (u) | Y | Y |
| Wheels: friction, jitter | Y: Stribeck/Karnopp, back-EMF; jitter only as a post-run frequency-domain number (T10) | Y: balanced, simple and fully coupled jitter [4] | Y: LuGre friction, jitter on flex [7] | Y via 42 | (u) | Y | Y |
| Magnetorquers | Y: L/R lag, saturation; no hysteresis (S2) | Y: MtbEffector [2] | Y [7] | Y | (u) | Y [26] | Y |
| Thrusters | P: MIB, valve faults; stuck-open missing (T12) | Y [2] | Y [7] | Y | (u) | Y | Y |
| Register-level / byte-level device I/O | Y: HAL frames are quantised in every SILS run | N (message passing) | P: IPC sockets [5] | Y: NOS Engine software buses [16] | N | N | N |

The orbit-only tools (GMAT, Orekit, Tudat, FreeFlyer) have no attitude sensor or actuator models and are left out of this table.

## 4. GNC algorithm libraries

| Capability | TN | BSK | 42 | NOS3 | MW | PSS | SOL |
|---|---|---|---|---|---|---|---|
| Attitude estimators | Y: MEKF (Farrenkopf Q, Joseph), TRIAD, QUEST/q-method; no NEES test (S6) | Y: e.g. inertial UKF, sunline EKF (u: exact names) | P: sample FSW [5] | P: FSW apps (u) | P (u) | Y: Kalman, nonlinear estimation [23] | Y: ODySSy/MAX [24] |
| Controllers | Y: PID (gain error G6), LQR, SMC, magnetic PD, B-dot family, Sun-spin | Y: mrpFeedback, mrpPD, mrpSteering, rate servo [8] | Y (sample) | P | Y: Simulink control | Y [23] | Y |
| Guidance | P: nadir, target, Sun, cycloidal slews; no eigenaxis or keep-out (T11) | Y: hill, inertial, Sun-safe pointing (u) | P | P | P | Y | Y |
| Momentum management | Y: magnetic cross-product dump, RCS dump, IDMAS split | Y: thruster and MTB momentum management [8] | (u) | (u) | (u) | Y | Y (u) |
| CMG steering | Y: SR steering (G7 null-space defect) | (u) | (u) | — | (u) | Y (u) | (u) |
| FDIR | P: silence and \|B\| checks, rotor FDIR, safe mode; no stuck/bias-jump isolation (G12, F2/G3) | P (u) | N (u) | P: cFS Health & Safety, Limit Checker apps [18] | N | P (u) | Y: autonomy in MAX (u) |

## 5. Flight-software integration

The columns change here, because the tools that matter for this category are different.

| Capability | TN | BSK | 42 / NOS3 | cFS / F´ | Renode | QEMU | MW + Speedgoat/dSPACE | SMP / SIMULUS / EuroSim | Yamcs / COSMOS |
|---|---|---|---|---|---|---|---|---|---|
| SILS (FSW in loop) | Y: C and Rust FSW in process, bit-identical | Y: FSW modules in C; EMM algorithms built in BSK, flown on cFS [31] | Y: IPC; NOS3 runs cFS/F´ against 42 [5][16] | Y: host builds; F´ swaps driver components for sim [28] | — | — | Y | Y | — |
| Processor emulation (soft-OILS) | Y: Cortex-M4F on QEMU mps2-an386, exact instruction counts → latency into the plant, judged deadline | N | N: FSW runs as Linux processes (u); JSTAR emulators are separate [33] | — | Y: Cortex-M, STM32F4/H7, LEON3, RISC-V, deterministic, peripherals [34][35] | Y: icount gives deterministic instruction counting; record/replay; not cycle-accurate [36] | P: PIL on real board with execution profiling [22] | Y: SIMULUS emulators ERC32/LEON2-4/GR740/ARM [38] | — |
| OILS (real OBC) | P: adcs-link over TCP/UDP ready; no board target (H1) | P | P | Y (target builds) | — | — | Y: PIL [22] | Y | — |
| HILS (real-time, stimulators) | N: documentation only (H4) | P: real-time clock sync, Black Lion [31] | P: "HWIL IPC" doc [5] | — | — | — | Y: Simulink Real-Time on Speedgoat [40]; SCALEXIO; AOCS SCOE [39] | Y: EuroSim real-time [37] | — |
| Time synchronisation | Y: lockstep per tick; integer tick missing (F3) | Y: clock-tracking modules [31] | Y: NOS3 time driver [16] | P | Y: virtual time | Y: icount virtual clock [36] | Y | Y | — |
| Fault injection | P: scheduled device faults; no SEU or bus faults (T15) | P (u) | Y: NOS3/JSTAR fault injection [33] | P | Y: scriptable via Robot Framework/monitor [34] (u: SEU) | P (u) | Y: electrical fault injection (dSPACE FSX) [39] | Y (u) | — |
| TM/TC + ground | N: one TC, no PUS, no HK (F8) | N | Y: COSMOS 4/5, Yamcs, F´ GDS, AIT [16][41] | Y: CI/TO apps, cFS GS [18]; F´ GDS [28] | — | — | P | Y: SIMULUS + ESOC ground | Y: XTCE, CCSDS 133.0-B-2, TM/TC/AOS/USLP, COP-1, CFDP, PUS preprocessor [41][42] |
| Model portability standard | N | N | N | N | — | — | P (FMI) (u) | Y: ECSS-E-ST-40-07C (2020; Rev.1 2025) [37] | — |

## 6. Analysis and V&V

| Capability | TN | BSK | 42 | GMAT/ORE/FF | MW | PSS | SOL |
|---|---|---|---|---|---|---|---|
| Monte Carlo | Y: per-run streams, 1,109-run success sizing, Clopper-Pearson, NaN as failure; inertia draws can be non-physical (A5) | Y: MonteCarloController, retention, multiprocessing [8] | P: scripted (u) | Y: FF noise functions [27]; GMAT (u) | Y (Simulink Test/parsim, u) | Y (u) | Y (STK Analyzer, u) |
| Linear analysis, margins | N (T1) | N (u) | N | — | Y: linearise, allmargin/diskmargin [22b] | Y: control design tools [23] | N (u) |
| Sensitivity / worst-case search | N (T2) | N (u) | N | N | Y: Simulink Design Optimization (u) | (u) | (u) |
| ECSS pointing metrics and budgets | P: APE/RPE/AKE/MKE/PDE engine and twin; budget RSS-sums biases (A4); confidence verdicts being corrected (A1) | N (u) | N | — | N (u) | Y: pointing budgets [23] | (u) |
| Requirement traceability | P: `trace.py`; 26 of 58 case keys unread | N | N | N | Y: Requirements Toolbox, traceability matrix [22c] | N (u) | N (u) |
| Reporting | Y: generated V&V report HTML/PDF, verdict ledgers | P: plots | P: report files | Y | Y | Y | Y |
| Visualisation | P: 2D SVG/PDF figures, desktop app; no 3D (B7) | Y: Vizard (Unity), keep-in/out cones, cameras [9] | Y: OpenGL [5] | Y (GMAT/FF 3D) | Y (u) | Y: CAD viewer [23] | Y: STK 3D |

## 7. Design and trade support

| Capability | TN | BSK | 42 | MW | PSS | SOL | CubeSpace |
|---|---|---|---|---|---|---|---|
| Disturbance/demand survey | Y: 4 attitudes × seasons × solar activity; integrated in body frame (Y5 error) | P: by simulation | P | P | Y [23][26] | P | Y: D2S2 pointing/power budgets [43] |
| Actuator sizing | Y: wheels, coils, rings, RCS, CMG laws; 7 unsourced constants (D10) | N | N | N | Y: CubeSat Toolbox [26] | (u) | P (own products) |
| Parts catalogue | Y: datasheet catalogue; 14 synthetic parts (V3) | N | N | N | P (u) | P: component GUI [24] | P (own products) |
| Closed design loop (size → fly → converge → select) | Y: `pipeline.py` | N | N | P: optimisation toolboxes (u) | N (u) | N (u) | N |
| Optimisation | P: grid and rank by worst objective | N | N | Y (u) | (u) | (u) | N |

## 8. Engineering qualities

| Quality | TN | BSK | 42 | NOS3 | GMAT | ORE | TUD | MW | PSS | SOL | FF |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Language / speed | Rust engine, C99 + Rust `no_std` FSW, MATLAB/Octave twin | C/C++ with Python [1] | C [6] | C/C++, Linux VMs | C++ | Java | C++ / Python [14] | MATLAB/Simulink | MATLAB [23] | C++ MAX [25] | own script |
| Determinism | Y: exact QEMU instruction counts; C = Rust bit-identical; per-platform libm (F18) | Y: repeatable MC [31] | Y (u) | P | Y | Y | Y | Y | Y | (u) | Y |
| Licence | Proprietary, all rights reserved | ISC [1] | NOSA [30] | NOSA 1.3 [16] | NOSA [10] | Apache-2.0 [13] | BSD-3 (u) | Commercial | Commercial | Commercial | Commercial |
| Documentation | Y: extensive, per-command, audit registers | Y | P | Y | Y | Y | Y | Y | Y | Y | Y |
| Validation evidence / heritage | P: equations checked against ERFA, pyIGRF, scipy; no flight referent (V1, A13); no flight use | Y: EMM flight algorithms [31]; HIL testbeds | Y: GSFC missions (u: specific list) | Y: STF-1 CubeSat [32] | Y: formal V&V, flight ops [10] | Y: ops use (u) | Y: research | Y | Y: long commercial use (u) | Y: MAX is flight code [25] | Y: ops use (u) |

---

## (a) Genuine differentiators of TRI-NETRA

The question framing needs one correction first. The flight software is **not** generated from the pseudocode. The C and Rust flight software are hand-written, held to `fsw/pseudocode/*.pc`, and kept bit-identical to each other. The pseudocode is *translated* to Rust, MATLAB and WebAssembly for the physics and design relations, and an interpreter cross-checks every translation (`docs/PSEUDOCODE_V2.md`, `docs/GENERATORS.md`).

With that understood, these are the differentiators I could not find in the public documentation of any tool reviewed:

1. **Two independent flight-software implementations (C99 and Rust `no_std`) that are bit-identical, plus an executable pseudocode contract with an interpreter that checks every translation (Rust, MATLAB, WASM).**
   - Basilisk FSW modules are C and can be flown, as on EMM [31], and Embedded Coder generates C from Simulink [22]. Neither offers dual-language N-version parity.
   - Caveat (F15): parity cannot find common-mode bugs. F1–F5 were common-mode.
2. **An end-to-end design loop.** Case → demand survey → sizing → SILS mode matrix → converge/resize → selection → Monte Carlo → soft-OILS → V&V report, in one command.
   - The PSS CubeSat Toolbox [26] and CubeSpace D2S2 [43] cover sizing and budgets, but not the automated closed loop with family selection.
   - The loop's verdict logic currently has the defects A1, Y1, Y2 and Y6, so the differentiator is architectural until U0 lands.
3. **Soft-OILS with the timing judged, not just the bytes checked.**
   - A QEMU TCG plugin counts guest instructions exactly between markers. With the CPI and bus models, that count becomes command latency injected into the plant, and overruns and worst-case margin are judged per run.
   - QEMU itself explicitly does not model timing [36]. Embedded Coder PIL measures time on real hardware but does not feed latency back into the plant model [22]. Renode is deterministic but has no plant [34].
   - The weakness: the CPI is assumed (H2).
4. **Register-level HAL frames in every SILS run.** Values are quantised exactly as the drivers will see them, and a loopback byte check confirms the frames, so SILS → OILS → HILS changes only the backend. NOS3 does this at the bus level [16]; Basilisk and 42 work at message or IPC level.
5. **ECSS-E-ST-60-10C pointing indices with statistical claim checking** (success-run sizing, Clopper-Pearson, NaN-as-failure, temporal/ensemble/mixed interpretation). No reviewed open tool ships this, though PSS has pointing budgets [23].
6. **Models of novel actuators**: magneto-fluidic momentum rings and IDMAS allocation, and an N2O cold-gas RCS for CubeSats. These appear in no tool reviewed. Fidelity is still partial (M1, M3).
7. **Self-audit culture**: gap registers, verdicts generated from data, refusal of unstated values by name. This is a process quality rather than a feature, but rare.

## (b) Capabilities TRI-NETRA lacks that matter for a CubeSat ADCS programme

**Priority 1: credibility and flight-readiness**

| # | Capability | Shown by | Benefit | Action |
|---|---|---|---|---|
| 1 | Independent attitude-dynamics referent on shared scenarios (wheels, VSCMG, flex, jitter) | Basilisk [2][4], 42 [5][7] | Validation against a different code base; finds common-mode physics errors that C = Rust parity cannot (A13, NASA-STD-7009B) | **ADOPT** as cross-check (both permissive licences) |
| 2 | Orbit, frame and EOP reference; dated space weather | Orekit [11][12], GMAT [10] | Closes V1 and E3/E4/T6 with a published referent; Orekit's CSSI loader shows the data contract | **ADOPT** Orekit for verification vectors; **BUILD** dated F10.7/Ap/Kp ingestion and IERS EOP natively |
| 3 | CCSDS space packets + PUS-C services, ground segment | Yamcs [41][42], COSMOS [41], NOS3 [16], cFS CI/TO [18] | Turns F8 (no TM/TC) into a testable interface; ops rehearsal | **BUILD** PUS-C in the FSW; **INTEROPERATE**: XTCE database → Yamcs or COSMOS as ground (AGPL tools stay outside the product) |
| 4 | Board-faithful emulation (STM32 peripherals, timers, watchdog, I2C/SPI/CAN) | Renode [34][35] | QEMU mps2-an386 has no STM32 peripherals. Renode would expose F3 (timing jitter), F7 (watchdog) and F11 (babbling bus) before a board exists | **ADOPT** Renode as a second soft-OILS target on the same adcs-link |
| 5 | Fault injection at bus and SEU level | NOS3/JSTAR [33], Renode [34], dSPACE FSX [39] | ECSS-Q-ST-60-15C SEU coverage (T15); bus errors (F12) | **BUILD** a bit-flip and bus-error fault kind in the engine; **ADOPT** Renode hooks for memory faults (u) |
| 6 | Real-time HILS target with stimulator I/O | Speedgoat [40], dSPACE SCALEXIO / AOCS SCOE [39], EuroSim [37], CubeSpace D2S2 HIL mode [43] | H4 is documentation only today | **INTEROPERATE**: run the engine on PREEMPT_RT Linux behind adcs-link first (cheapest for CubeSat budgets); buy a Speedgoat/dSPACE target only if cage and air-bearing I/O need hard real time |

**Priority 2: analysis depth**

| # | Capability | Shown by | Benefit | Action |
|---|---|---|---|---|
| 7 | Linearisation, gain/phase/disk margins per mode | Simulink Control Design / Robust Control [22b], PSS [23] | ECSS-E-ST-60-10C margins (T1); catches G6-type tuning errors | **BUILD** natively (Rust/Python), cross-checked against MATLAB `allmargin` in the twin when a licence exists |
| 8 | Sensitivity and worst-case search | Simulink Design Optimization (u) | Failure attribution (T2, D15) | **BUILD** (Morris/Sobol, seeded search) |
| 9 | Coupled wheel-imbalance jitter in the time domain | Basilisk fully coupled jitter [4]; 42 jitter [7] | Imaging RPE/PDE (T10) | **BUILD**, with Basilisk as reference |
| 10 | Gyro bias instability fitted to the Allan curve | 42 correlated bias [6]; IEEE 952 practice | Imaging AKE realism (T8, M6) | **BUILD** |
| 11 | Real star catalogue and lost-in-space in the image chain | Basilisk camera + Vizard images [3][9] (scene generation only) | Imaging knowledge budget (T7, D3: dual-head outages) | **BUILD** (Hipparcos/Tycho-2); optionally **INTEROPERATE** with Vizard images for stray-light scenes (u) |
| 12 | 3D visualisation (geometry, FOV and keep-out cones) | Vizard [9], STK, 42 OpenGL [5] | Reviewers see exclusion-cone conflicts (D3) at once | **INTEROPERATE**: export CCSDS OEM/AEM so STK, GMAT or other viewers can replay; do not build a 3D engine |
| 13 | Requirements traceability tooling | Requirements Toolbox [22c] | D11: 90 of 243 spec rows linked to nothing | **BUILD** (extend `trace.py`); ReqIF export for **INTEROPERATE** (u) |

**Priority 3: lower for a CubeSat, keep in view**

| # | Capability | Shown by | Benefit | Action |
|---|---|---|---|---|
| 14 | Higher-order gravity, IGRF-14, crustal/external field | MW IGRF-14 block [21], Orekit [12], MW EGM2008 [19] | E5, T4, D14 | **BUILD** (data only; already U1) |
| 15 | Slosh, multi-body | Basilisk [2], 42 [5] | Only if propellant fraction grows | Not now; **ADOPT** Basilisk as the cross-check if needed |
| 16 | FSW framework hosting (scheduler, HK, tables, event services) | cFS [18], F´ [28] | Reuse flight-proven services instead of writing them in U4 | **INTEROPERATE**: wrap `adcs_fsw_step` as a cFS app or F´ component; keep the ADCS core |
| 17 | SMP model portability | ECSS-E-ST-40-07C [37], SIMULUS [38] | Only when supplying ESA-class operational simulators | **INTEROPERATE** later: SMP wrapper around the engine |
| 18 | Flight heritage | Basilisk (EMM) [31], NOS3 (STF-1) [32], SOLIS MAX [25] | Credibility no tool can lend | Fly; compare against telemetry (V1 referent) |

## Bottom line

TRI-NETRA is unusual in joining design sizing, closed-loop GNC verification and an exact-timing soft-OILS path into one deterministic chain, with dual-language flight software. That combination appears in none of the reviewed tools.

It is behind the established tools in four areas:

- **Environment and frame completeness**: Orekit, MATLAB and GMAT.
- **Dynamics breadth and coupled jitter**: Basilisk and 42.
- **Ground and TM/TC integration, and fault-injection infrastructure**: NOS3, cFS, F´, Yamcs and COSMOS.
- **Analysis tooling** (margins, sensitivity, traceability): MATLAB toolboxes and PSS.

The cheapest high-value moves for a CubeSat programme:

- Adopt Basilisk/42 and Orekit as referents.
- Add Renode beside QEMU.
- Build PUS-C and talk to Yamcs or COSMOS.
- Build margins and sensitivity natively.

## Sources

1. Basilisk repository (ISC, C/C++/Python, Vizard): https://github.com/AVSLab/basilisk
2. Kenneally, Piggott, Schaub, "Basilisk: a flexible, scalable and modular astrodynamics simulation", ICATT 2018 (module list incl. FuelTank, HingedRigidBodies, VSCMGs, Thrusters, MtbEffector, facetDrag): https://indico.esa.int/event/224/papers/3869/files/219-Kenneally_2018_ICATT.pdf
3. Basilisk sensor and environment docs (camera, CSS, IMU, magnetometer, starTracker; MsisAtmosphere; magneticFieldWMM): https://hanspeterschaub.info/basilisk/Documentation/simulation/sensors/index.html ; https://hanspeterschaub.info/basilisk/Documentation/simulation/environment/MsisAtmosphere/index.html ; https://hanspeterschaub.info/basilisk/Documentation/simulation/environment/magneticFieldWMM/index.html
4. Basilisk reaction-wheel effector (balanced, simple, fully coupled jitter) and VSCMG effector: https://hanspeterschaub.info/basilisk/_downloads/17eeb82a3f1a8e0b8617c8b8284303ed/Basilisk-REACTIONWHEELSTATEEFFECTOR-20170816.pdf ; https://www.hanspeterschaub.info/basilisk/_downloads/e5f0be4c7eded98b948d83efb814d827/Basilisk-VSCMGSTATEEFFECTOR-20180718.pdf
5. NASA 42 repository README and Docs (multi-body tree, rigid/flex, IPC, OpenGL, "HWIL IPC", "FSW Models"): https://github.com/ericstoneking/42 ; https://github.com/ericstoneking/42/tree/master/Docs
6. 42 sensors and environment source: https://github.com/ericstoneking/42/blob/master/Source/42sensors.c ; https://github.com/ericstoneking/42/blob/master/Source/42environs.c
7. 42 actuators source (LuGre wheel friction, MTB, pulsed/proportional thrusters, jitter): https://github.com/ericstoneking/42/blob/master/Source/42actuators.c
8. Basilisk MonteCarlo controller; fswAlgorithms/attControl list: https://hanspeterschaub.info/basilisk/Documentation/utilities/MonteCarlo/Controller.html ; https://hanspeterschaub.info/bskOlderDocs/bsk_2_1_7/Documentation/fswAlgorithms/attControl/index.html
9. Vizard scripting (keep-in/out cones, cameras, opNav image stream): https://hanspeterschaub.info/bskOlderDocs/bsk_1_8_10/Vizard/vizardSettings.html ; https://hanspeterschaub.info/bskOlderDocs/bsk_1_5_1/Vizard/VizardReleaseNotes.html
10. GMAT wiki and flyer (harmonics, drag, tides, relativity, NOSA, estimation): https://gmat.atlassian.net/wiki/spaces/GW ; https://gmat.atlassian.net/wiki/download/attachments/380273216/GMAT_Flyer.pdf?api=v2
11. Orekit atmosphere package (DTM2000, JB2008, NRLMSISE00, Harris-Priester): https://orekit.org/static/apidocs/org/orekit/models/earth/atmosphere/package-summary.html
12. Orekit GeoMagneticFieldFactory (IGRF, WMM), CssiSpaceWeatherData, frames/EOP: https://orekit.org/site-orekit-development/apidocs/org/orekit/models/earth/GeoMagneticFieldFactory.html ; https://www.orekit.org/static/apidocs/org/orekit/models/earth/atmosphere/data/CssiSpaceWeatherData.html ; https://www.orekit.org/static/apidocs/org/orekit/frames/Predefined.html
13. Orekit licence (Apache-2.0), as listed in TRI-NETRA `docs/TECHNICAL_ROADMAP.md` §4 (u: not re-fetched)
14. Tudat docs (translational/rotational/mass propagation, torque models, tudatpy): https://tudat-space.readthedocs.io ; https://tudat-space.readthedocs.io/en/latest/_src_user_guide/state_propagation/propagation_setup/rotational/available_torque_models.html
15. GMAT attitude models (kinematic): https://documentation.help/GMAT/SpacecraftAttitude.html
16. NOS3 repository (42, cFS, F´, COSMOS, Yamcs, OpenC3, time sync, fault injection, NOSA 1.3): https://github.com/nasa/nos3 ; https://nos3.readthedocs.io/en/v1_07_02/Ground_Software.html
17. OpenSatKit (cFS + 42 + COSMOS): https://ntrs.nasa.gov/citations/20170007710 ; https://github.com/OpenSatKit/OpenSatKit/wiki/Home/11f5a09a5e80d91f2c0c06fdbdbf8ff43b956509
18. cFS bundle (Apache-2.0; CI/TO, HK, Health & Safety, Limit Checker, etc.): https://github.com/nasa/cfs
19. MATLAB Spacecraft Dynamics block; Spherical Harmonic Gravity (EGM2008): https://www.mathworks.com/help/aeroblks/spacecraftdynamics.html ; https://www.mathworks.com/help/aeroblks/sphericalharmonicgravitymodel.html
20. MATLAB NRLMSISE-00 block; solar flux and geomagnetic index block: https://www.mathworks.com/help/aeroblks/nrlmsise00atmospheremodel.html ; https://www.mathworks.com/help/aeroblks/solarfluxandgeomagneticindex.html
21. MATLAB IGRF block (IGRF-14 to 2030): https://www.mathworks.com/help/aeroblks/internationalgeomagneticreferencefield.html ; CubeSat Simulation Library: https://www.mathworks.com/matlabcentral/fileexchange/70030-aerospace-blockset-cubesat-simulation-library
22. Embedded Coder PIL and execution profiling: https://www.mathworks.com/help/ecoder/ug/execution-time-profiling-for-generated-code.html ; https://www.mathworks.com/help/mcb/gs/code-verification-profiling-using-pil.html
    22b. Stability margins of a Simulink model: https://www.mathworks.com/help/robust/ug/stability-margin-of-a-simulink-model.html
    22c. Requirements Toolbox: https://www.mathworks.com/products/simulink-requirements
23. PSS Spacecraft Control Toolbox (MathWorks Connections listing): https://au.mathworks.com/products/connections/product_detail/spacecraft-control-toolbox.html
24. STK SOLIS product and help: https://www.agi.com/products/STK-SOLIS ; https://help.agi.com/stk/Content/solis/solisMarketing.htm ; STK Attitude Simulator plugin: https://help.agi.com/stk/11.3/Subsystems/pluginScripts/Content/attitudePoints.htm
25. Rocket Lab SOLIS / MAX flight software: https://rocketlabusa.com/space-systems/space-software/solis
26. PSS CubeSat Toolbox: https://psatellite.com/getting-started-with-the-cubesat-toolbox/
27. FreeFlyer features (attitude representations, OD estimators, Monte Carlo): https://ai-solutions.com/freeflyer-features/ ; https://ai-solutions.com/_help_Files/freeflyer_scripting_smp.htm
28. F Prime overview and README: https://fprime.jpl.nasa.gov/overview/ ; https://cdn.jsdelivr.net/gh/nasa/fprime@devel/README.md
29. hapsira (poliastro fork, MIT): https://hapsira.readthedocs.io/
30. NASA 42 software catalogue entry: https://software.nasa.gov/software/GSC-16720-1
31. Basilisk real-time/HIL and Black Lion; EMM flight algorithms on cFS: https://hanspeterschaub.info/bskMain.html ; https://hanspeterschaub.info/Papers/ColsMargenet2018.pdf ; https://hanspeterschaub.info/Papers/ColsMargenet2017.pdf
32. NOS3 and STF-1: https://arxiv.org/pdf/1901.07583
33. JSTAR software digital twins, fault injection: https://ntrs.nasa.gov/api/citations/20250005069/downloads/20250617_JSTAR_Software_Digital_Twins_CLDP_Strives.pdf ; https://ntrs.nasa.gov/api/citations/20190029167/downloads/20190029167.pdf
34. Renode (deterministic, STM32 peripherals, Robot Framework, CI): https://antmicro.com/platforms/renode ; https://antmicro.com/blog/2026/07/stm32h7-renode-reference-platform ; https://antmicro.com/blog/2022/07/fully-deterministic-linux-zephyr-micro-ros-testing-in-renode
35. Renode for space (LEON3, HDL co-simulation): https://antmicro.com/blog/2024/02/developing-and-testing-space-systems-with-renode ; https://zephyrproject.org/leon3-support-in-renode/
36. QEMU TCG icount and record/replay: https://qemu.googlesource.com/qemu/+/refs/heads/staging-8.1/docs/devel/tcg-icount.rst ; https://qemu.readthedocs.io/en/master/system/replay.html
37. ECSS-E-ST-40-07C (2020) and Rev.1 (2025); SMP; EuroSim: https://ecss.nl/wp-content/uploads/2020/04/ECSS-E-ST-40-07C(2March2020).docx ; https://ecss.nl/wp-content/uploads/2025/08/ECSS-E-ST-40-07C-Rev.1(5August2025).docx ; https://en.wikipedia.org/wiki/Simulation_Model_Portability
38. SIMULUS / SIMSAT / emulators: https://esoc.esa.int/content/simulus-ng-new-era-satellite-simulation ; https://www.terma.com/space/ground-segment/satellite-simulators/
39. dSPACE SCALEXIO and AOCS SCOE: https://www.dspace.com/scalexio ; https://www.astos.de/products/mil_pil_hil_testbench ; https://www.aerospacetestinginternational.com/?p=21982
40. Speedgoat aerospace / satellite ADCS HIL: https://speedgoat.com/solutions/industries/aerospace ; https://www.speedgoat.com/user-stories/aalto-university
41. Yamcs (AGPL, XTCE, CCSDS links, COP-1, CFDP, SLE): https://www.yamcs.org ; https://yamcs.org/about ; OpenC3 COSMOS: https://builtin.com/company/openc3
42. Yamcs PUS preprocessor (ECSS-E-ST-70-41C): https://docs.yamcs.org/yamcs-server-manual/links/packet-preprocessor/pus/
43. CubeSpace Gen 2 ADCS brochure (D2S2 simulator, HIL mode): https://www.cubesatshop.com/wp-content/uploads/2023/05/CubeSpace-GEN-2-ADCS-2023.pdf
