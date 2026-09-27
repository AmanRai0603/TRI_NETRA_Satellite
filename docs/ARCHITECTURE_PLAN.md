# TRI-NETRA ADCS — Codebase Architecture Plan (SILS)

**Owner: Agastya.** Copyright (c) 2026 Agastya. All rights reserved.

This plan is built from exactly three inputs, and nothing else:

| # | Input | What it supplied |
|---|---|---|
| 1 | `trinetra_adcs-platform-architecture.zip` | The build specification (`spec/SPEC.md`), the ADCS node tree, the MATLAB SILS layout `+asils` (SPEC §10.8), the twin map of SILS elements, the case format `adcs-case/1`, the part/product/algorithm catalogue, scenarios, campaigns and metrics |
| 2 | `PrecisionOrbitPropagator_OD_v51.zip` | The Precision Orbit Propagator (POP): Cowell propagator with spherical-harmonic gravity, DE440 Sun/Moon, drag (DTM2020 / JB2008 / NRLMSISE / exponential), SRP, ERP, tides, relativity, frames — vendored unchanged in `matlab_sils/pop/` (one Octave-compatibility line, see below) |
| 3 | `TRI-NETRa Standard Code.rar` | The existing MATLAB ADCS SILS (MTQ / RW+MTQ stacks, R42 port): sensor models, B-dot, torque→dipole allocation, nadir guidance, quaternion PID, IGRF, quaternion library, conventions |

## 1. What changed, in one paragraph

The Standard Code's truth orbit was **direct data**: `sim.truthEphemeris` loaded a
frozen 359 MB Orekit CSV (`numericalpropagation_true_*.csv`) or a pre-computed
`pop_cache/*.mat` ephemeris before the loop started, and the disturbance torques
were evaluated at the **onboard J2-propagated** position. Both are removed. The
truth orbit is now **propagated inside the loop** by the POP's own force model,
one RK4 step at a time (`asils.orbit`), and every environment quantity the
attitude sees — magnetic field, Sun vector, eclipse, atmospheric density,
atmosphere-relative velocity, SRP pressure — is read from that precision orbit
and its POP context. The OD information *flows* into the disturbance torques,
which were upgraded from one area at a fixed CP–CM offset to a per-facet
model. The ADCS nodes that were stubs in the Standard Code (MEKF, nadir
pointing with magnetorquers, fine pointing with wheels, momentum dumping,
target slew) are implemented, and the whole chain runs from a case CSV.

## 2. The loop (one tick = 0.1 s)

```
            ┌────────────────────── case CSV (adcs-case/1) ──────────────────────┐
            │  orbit 550 km SSO / LTAN · mass & inertia · surfaces · dipole · req │
            └──────────────┬──────────────────────────────────────────────────────┘
                           ▼
 asils.config  ◄── scenario JSON (duration, initial state, modes, metrics)
      │        ◄── product JSON  (fitted parts: data/parts/*.json)
      ▼
┌──────────────┐ r,v   ┌────────────────────┐  B,ŝ,ν,ρ,v_rel ┌──────────────────┐
│ asils.orbit  │──────►│ asils.env          │───────────────►│ asils.env.torques│
│ POP in-loop  │ ctx   │ IGRF · shadow ·    │                │ GG · aero facets │
│ RK4 on op.rhs│──────►│ DE440 Sun · DTM2020│                │ SRP facets · mag │
└──────▲───────┘       └─────────┬──────────┘                └────────┬─────────┘
       │ attitude (R_bi, box-wing)│ truth B, ŝ                        │ τ_dist
       │                          ▼                                   ▼
       │               ┌────────────────────┐   z   ┌───────────┐ cmds ┌──────────────┐
       │               │ asils.devices      │──────►│ asils.fsw │─────►│ devices.mtq  │
       │               │ gyro·mag·CSS·ST·GPS│       │ (below)   │      │ devices.rw   │
       │               └─────────▲──────────┘       └───────────┘      └──────┬───────┘
       │                         │ q, ω, h_w                                   │ τ_mtq, τ_w
       │                ┌────────┴────────────────────────────────────────────▼──┐
       └────────────────│ asils.plant  rigid body + wheels, RK4, |q| = 1          │
                        └──────────────────────────┬─────────────────────────────┘
                                                   ▼
                              asils.metrics (APE/AKE/RKS/…) → asils.viz / asils.result
```

Rates: plant 10 Hz (RK4), POP orbit node 0.1 Hz with Hermite dense output at every
tick, environment 1 Hz (held), gyro 10 Hz, magnetometer 10 Hz (clean sample in the
coil-off window), sun sensors 10 Hz, star tracker 5 Hz with 0.1 s latency, GNSS 1 Hz,
MTQ duty 1 s (0.2 s measure, 0.8 s drive), wheel control 10 Hz.

## 3. Node by node — where each node lives and what fills it

Node ids follow the twin map (`spec/plan/twin_map.toml`); the file is its MATLAB twin.

| Twin-map node | File (`matlab_sils/+asils/`) | Filled with | Source |
|---|---|---|---|
| `case.read` | `+case/read.m` | CSV parser with the `adcs-case/1` checks | spec §8.3 |
| `product.supply` | `+product/load.m` | parts → descriptor-level device block | spec §7, catalogue |
| `scenario.load` | `+scenario/load.m` | exported scenario JSON | spec §10.1 |
| `env.orbit` | `+orbit/init.m`, `advance.m`, `node.m`, `state.m`, `context.m` | **In-loop POP**: RK4 on `op.accel` (J2–J6 field, DE440 third body, DTM2020 drag, SRP conical eclipse), cubic-Hermite dense output, per-node context | **POP v51** (replaces Orekit CSV / pop_cache) |
| `env.field` | `+env/igrf_coefs.m`, `igrf_gh.m`, `igrf_ned.m`, `field.m` | IGRF-13 degree 13 at the POP position, NED→ECEF→ECI with the POP frame | Standard Code `env.igrf*` (ported) |
| `env.density` | `+orbit/node.m` → `context.m` | density from the POP drag model at each node, log-linear between | POP `atmos.dtm2020` |
| `env.sun_eclipse` | `+env/shadow.m`, `+orbit/node.m` | DE440 Sun, conical shadow (Montenbruck & Gill) | POP `ephemInputs` |
| `env.disturbances` | `+env/geometry.m`, `torques.m` | gravity gradient; free-molecular aero per facet (Schaaf–Chambre, σn/σt, vb/v); SRP per facet (specular + diffuse) × ν; residual dipole × B | Standard Code `env.disturbances` (upgraded) |
| `plant.dynamics` | `+plant/deriv.m`, `step.m`, `geometry.m`, `axes.m` | rigid body + every momentum-exchange device: fixed rotors (wheels, fluid rings) and gimballed rotors (CMG, VSCMG), Euler with a variable-axis momentum, RK4, renormalise | Standard Code `plant.dynamics` (generalised) |
| `device.gyro` | `+devices/gyro.m` | scale/misalign, bias, RRW, ARW, range | Standard Code `sens.gyro` + SYN-GYRO-1 |
| `device.magnetometer` | `+devices/magnetometer.m` | scale/misalign, bias, noise, coil coupling, range | Standard Code `sens.magnetometer` + SYN-MAG-1 |
| `device.sun_sensor` | `+devices/sun_sensor.m` | six heads, FOV, eclipse, bias + noise | Standard Code `sens.sunSensor` + SYN-SUN-1 |
| `device.star_tracker` | `+devices/star_tracker.m`, `star_catalogue.m`, `st_history.m`, `+fsw/quest.m` | star-field model (4000-star catalogue, 12 brightest in the FOV, centroid noise), QUEST / q-method attitude, two heads, latency (true history), rate / Sun / Earth exclusion | Standard Code `sens.starTracker` + SYN-ST-1 |
| `device.gnss` | `+devices/gps.m` | position/velocity noise | Standard Code `sens.gps` |
| `device.coil_tile` | `+devices/mtq.m` | per-coil saturation, scale, misalignment, power | Standard Code `act.mtqModel` + SYN-CT-1 |
| `device.reaction_wheel`, `device.magneto_fluidic_panel`, `device.cmg`, `device.vscmg` | `+devices/mex.m` | wheels: torque/speed limits, friction with driver compensation, noise; fluid rings: ρA2S momentum, 0.75 s laminar loss, closed-loop flow driver, pump power; CMG/VSCMG: rotor speed loop, gimbal rate limits | Standard Code `act.rwModel`, SYN-RW-10, SYN-MFP-1 (IDMAS), TRN-CMG-1, TRN-VSCMG-1 |
| `device.rcs` | `+devices/rcs.m` | six cold-gas couples, PWM with minimum impulse bit and valve resolution, thrust scale/misalignment, propellant | TRN-RCS-3U |
| `device.coarse_sun_sensor` | `+devices/css.m` | six cosine photodiodes with Earth albedo | TRN-CSS-1 |
| `fsw.bdot` | `+fsw/bdot.m` | Avanzini–Giulietti gain, unit-vector derivative, direction-preserving saturation, measure-then-drive duty | Standard Code `ctrl.bdot`, `act.saturateDipole` |
| `fsw.mekf` | `+fsw/mekf_init.m`, `mekf_predict.m`, `mekf_vector.m`, `mekf_quat.m`, `triad.m` | Markley–Crassidis MEKF (attitude + gyro bias), Sun / field / star-tracker updates, TRIAD initialisation | Standard Code `ad.mekf` was a **stub** — implemented |
| `fsw.guidance` | `+fsw/guidance.m` | nadir (ported), off-nadir target, cycloidal target slew, inertial | Standard Code `guid.nadir` |
| `fsw.mtq_pd` | `+fsw/mtq_pd.m`, `torque2dipole.m` | magnetic three-axis PD + min-norm dipole | Standard Code `act.torque2dipole` |
| `fsw.control_law` | `+fsw/control_law.m`, `lqr_gain.m` | PID (ported), LQR (Hamiltonian ARE, Bryson weights), sliding mode (Crassidis–Markley); gyroscopic compensation and slew-acceleration feedforward | Standard Code `ctrl.nadirPointing` PID |
| `fsw.pd_alloc`, `fsw.idmas_split`, `fsw.cmg_sr`, `fsw.rcs_pwm` | `+fsw/allocate.m`, `steer_sr.m`, `rcs_duty.m`, `dump.m` | minimum-norm rotor allocation; IDMAS split (coils take the torque across B, rings the rest); singularity-robust CMG/VSCMG steering; RCS slew assist and dumping with MIB-aware feedforward; cross-product magnetic dump; rotor FDIR with coil backup | Standard Code `ctrl.rwDump`; spec algorithms |
| `hal` | `+hal/*.m` | the `adcs_hal.h` boundary in MATLAB: register-level frames, `sils` / `loopback` / `udp` backends, real-time pacing, HILS stimulus | spec `fsw/include/adcs_hal.h`; `docs/OILS_HILS.md` |
| faults | `+faults/apply.m` | scheduled rotor / gimbal / star-tracker head / coil / gyro / GNSS / valve faults | spec §10.2 fault campaigns |
| mode manager | `+fsw/step.m`, `init.m` | tick order, onboard orbit, modes, auto transition | Standard Code `modes.step` |
| `run.single` | `run.m` | the loop above | spec §9.3 |
| `metric.*` | `+metrics/derive.m`, `evaluate.m`, `window.m`, `time_to.m` | APE/AKE (3-axis and payload LOS), RKS, detumble, settle, momentum, power | spec §10.3 (ECSS-E-ST-60-10C) |
| `campaign.run` | `+campaign/run.m`, `draw.m`, `collect.m`, `summarise.m`, `write.m` | Monte Carlo and edge-case (each dispersion at its bounds, then all adverse) campaigns with per-run streams, parfor / worker processes, ensemble percentile at the case level | spec §10.2 |
| `recorder`, `result.document` | `+rec/write.m`, `+result/save.m` | CSV + JSON channels, HTML result | spec §9.8, §13.5 |
| `viz.*` | `+viz/run.m`, `campaign.m` | per-test figures | spec §10.8.4 |

## 4. The two cases

| | AIS (`cases/ais_3u.csv`) | Imaging (`cases/ais_img_3u.csv`) |
|---|---|---|
| Orbit | 550 km SSO, i = 97.593°, **dawn–dusk LTAN 06:00** | 550 km SSO, i = 97.593°, **LTAN 10:00** |
| Pointing requirement | APE 10° (AKE 5°) | APE 0.01° 3σ (AKE 0.005° 3σ) |
| Product | `TRN-P-3U-AIS`: 3 coils, magnetometer, 6 sun sensors, MEMS gyro, GNSS; antenna axis +X (long axis) on nadir | `TRN-P-3U-IMG`: 3 reaction wheels, **two** star-tracker heads (±25° from zenith), precision MEMS gyro, 3 coils, magnetometer, 6 sun sensors, GNSS; camera axis +Y on nadir |
| Modes | detumble (B-dot) → nadir_mtq | detumble (B-dot) → nadir_rw / target_rw / slew_rw with magnetic dumping |
| Scenarios | `detumble_ais`, `nadir_hold_ais`, `mission_ais` | `detumble_img`, `fine_hold_img`, `slew_img`, `mission_img` |
| Monte Carlo | `mc_detumble_ais`, `mc_nadir_ais` | `mc_fine_img`, `mc_slew_img` |

## 4b. Actuator families (all on the imaging case, plus the AIS coils-only case)

| family (spec) | product | fine pointing | agile slew | momentum management |
|---|---|---|---|---|
| `mtq` | `TRN-P-3U-AIS`, `TRN-P-3U-AIS-CSS` | coils only (magnetic PD) | — | residual-dipole compensation |
| `mtq_rw` | `TRN-P-3U-IMG` | 3 wheels, PID / LQR / SMC | torque-limited | coils, cross-product law |
| `mtq_fmr` | `TRN-P-3U-FMR` | 3 fluid rings, IDMAS split with the coils | momentum-limited (1 mN m s) | coils |
| `mtq_fmr_rcs` | `TRN-P-3U-FMR-RCS` | rings + coils | RCS takes the axes the rings cannot hold | coils |
| `mtq_rw_rcs` | `TRN-P-3U-RW-RCS` | wheels | RCS assists beyond wheel torque / momentum | RCS (MIB-aware) |
| `mtq_cmg` | `TRN-P-3U-CMG` | 4-SGCMG pyramid, SR steering | gimbal torque | coils |
| `mtq_vscmg` | `TRN-P-3U-VSCMG` | 4-VSCMG pyramid, SR steering + wheel mode | gimbal + wheel torque | coils |

## 5. Disturbance torques — the model and why it is better

The Standard Code (`env.disturbances`) computed drag and SRP as one force on one
reference area applied at a fixed CP–CM offset, with a scalar Cd and reflectivity,
at the **J2-propagated** onboard position, and an exponential or NRLMSISE density
that needed the Aerospace Toolbox. The SILS now uses:

- **Orbit state and density from the precision OD** (POP RK4 in the loop; DTM2020
  with solar/geomagnetic indices; co-rotating atmosphere for v_rel).
- **Per-facet free-molecular aerodynamics** (Schaaf & Chambre flat plate, Hughes
  1986 eq. 8.34): normal and tangential accommodation, re-emission speed ratio,
  each facet's own lever arm about the CM, so the aero torque varies with the
  attitude and the flow angle instead of being a fixed-direction vector.
- **Per-facet SRP** with specular/diffuse split and the conical-shadow factor
  (penumbra included), Sun from DE440, pressure scaled with the Sun distance.
- **Gravity gradient** with the true inertia; **residual dipole** against IGRF
  degree 13 at the true position.
- All four switchable (`P.env.on`) and dispersed in the Monte Carlo (CM offset,
  dipole, accommodation, reflectivity, solar flux, Kp, inertia).

## 6. What is intentionally *not* in the SILS

Per the operating model of the spec, the platform pieces — Rust engine, portal,
rig (OILS/HILS), intake, evidence/certification — are the platform's, not the
SILS twin's (SPEC §10.8.1). This repository delivers the **SILS** (the MATLAB
twin) complete and runnable; the spec package is kept in `spec/` unchanged as the
plan for the rest.

## 7. Compatibility notes

- Runs in MATLAB (base, no toolboxes; Parallel Computing Toolbox used when present)
  and GNU Octave ≥ 8 (all results in this repository were produced in Octave 8.4).
- POP change: `de440.state` calls `de440.chebval` (package-qualified) because
  Octave does not resolve `private/` folders inside `+packages`; MATLAB behaviour
  is identical.
- IGRF: the Standard Code table ends at 2025 with no secular variation; epochs up
  to 2030 are extrapolated with the 2020→2025 rate (error ≈ tens of nT in 2027).

## 8. Design decisions the SILS drove (each is evidence in `docs/RESULTS.md`)

| # | Finding in the closed loop | Decision |
|---|---|---|
| D1 | With +Y on nadir the long (minimum-inertia) axis lies along-track: gravity gradient is destabilising in pitch and slow magnetic control cannot hold 10°. | AIS antenna axis = +X (long axis) on nadir: gravity gradient stabilises pitch/roll, coils add damping and yaw. |
| D2 | The case's 0.01 A m² residual dipole (≈2.4e-7 N m) exceeds the gravity-gradient restoring torque and the magnetic loop's authority at useful gains. | FSW cancels the ground-calibrated dipole with the coils (`fsw.m_res_est`); the Monte Carlo disperses the true dipole 0.5–2× so calibration error is flown. |
| D3 | B-dot from a 0.2 s field difference is noise-dominated below ~1°/s and re-tumbles the spacecraft. | Gyro-fed B-dot (ḃ = −ω×b), the Standard Code's rate-feedback branch; magnetometer differencing over consecutive 1 s windows as fallback. |
| D4 | SYN-GYRO-1 (0.17°/√h) leaves ≈0.02° 3σ jitter through the rate loop. | Imaging gyro = precision MEMS class (`TRN-GYRO-P1`, 0.034°/√h). |
| D5 | Single zenith star tracker: the Sun enters its 30° exclusion cone for ~580 s per orbit at LTAN 10:00. | Two heads canted ±25° from zenith, fused in the MEKF; gyro coasting across outages instead of coarse Sun/field updates. |
| D6 | Magnetic momentum-dump pulses (≈4e-7 N m) reach the body unopposed. | Wheel loop feeds forward the known dump torque m×B. |
| D7 | Rate error must use the reference rate carried into body axes, A(q_e)ω_ref (bug found at large AIS errors). | Fixed in both pointing laws. |
| D8 | A single-sample star-tracker history made the latency 0.2 s instead of 0.1 s (orbit-rate × 0.1 s = 0.006° bias). | True attitude history kept every tick and interpolated at t − latency. |
| D9 | Fluid-ring loss compensated open-loop with a ±20 % loss dispersion leaves a torque error proportional to the stored momentum; the rings stop delivering after ~10 min. | Closed-loop flow driver (integrated command, filtered flow sensor, servo), like a wheel speed loop. |
| D10 | Rings store only ~1 mN m s; thrusters cannot unload that (one MIB pulse ≈ 150 µN m over a tick). | Rings are unloaded by the coils (IDMAS); RCS only assists slews. RCS dump thresholds scale with each product's momentum capacity. |
| D11 | A thruster request below the MIB is not fired, but was fed forward to the wheels. | The FSW quantises its own valve commands (MIB, valve resolution) before feeding them forward. |
| D12 | ζ = 2 holds tighter, ζ = 0.9 settles a 30° slew in ~13 s instead of ~45 s. | Damping scheduled by mode: hold scenarios fly 2.0, slew/target scenarios 0.9. |
