
---

## 9. The loop engine — SILS

### 9.1 Three crates

| Crate | `no_std` | Holds |
|---|---|---|
| `adcs-sim-core` | yes, no alloc | the plant: state, dynamics, environment, sensor and actuator models, the RK4 step. Fixed-capacity arrays (at most 12 coils, 8 rings, 6 wheels, 16 thrusters, 16 sensors). It runs unchanged on the rig host, in the browser, and on an embedded target. |
| `adcs-sim` | no | the scenario loader, multirate scheduler, device emulators, recorder, metrics and campaign runner |
| `adcs-fsw-abi` | no | implements `adcs_hal.h` in Rust (`extern "C"`) over the device emulators; compiles and links the flight C code with the `cc` crate |

VLEO's kernel rule, "no files, no clock, no drawing", holds for `adcs-sim-core` without exception. Time is an input: `step(&mut State, &Inputs, dt) -> Outputs`, where `dt` is fixed per run. The clock belongs to the scheduler in `adcs-sim`, or to the real-time loop in `adcs-rig`, never to the plant. That is what lets one plant serve SILS faster than real time and OILS at exactly real time.

### 9.2 State and dynamics

State (all SI):

- the tick counter (`u64`); `t = tick · dt`, exact;
- orbit `r`, `v` in ECI, propagated with J2 and optional drag;
- attitude `q: Quat<Body, Eci>` and body rate `ω`;
- wheel speeds, ring flow speeds, ring fluid temperature and frozen flag, coil currents (first-order RL), thruster valve states, propellant mass;
- gyro bias random-walk states;
- star tracker availability.

Dynamics follow IDMAS v2 §12.1, extended with wheels. They are written so that total angular momentum is conserved: every torque between the body and a ring's fluid or a wheel's rotor is internal.

```
J ω̇ = −ω × (J ω + A_r h_r + A_w h_w) − A_r ḣ_r − A_w ḣ_w + (D_m m) × B + Σ_k r_k × f_k + τ_d
ḣ_r,i = τ_pump,i − τ_fric,i(h_r,i)        ḣ_w,j = τ_motor,j − τ_fric,j(Ω_j)        q̇ = ½ W(q) ω
```

`h_r` and `h_w` are the rings' fluid momenta and the wheels' rotor momenta about their axes, and the columns of `A_r` and `A_w` are those axes. Wall friction slows the fluid and turns the body by the same amount, so a ring's 0.75 s spin-down moves its momentum into the body; it does not lose it. IDMAS v2 §12.1's `−A u` is this `−A ḣ`, with `u` the commanded change of momentum.

RK4 at the scenario's `dynamics_step_s`. Discrete events (thruster pulses, valve edges, mode changes) happen only on tick boundaries. The quaternion is renormalised after every step, and its norm drift before renormalising is recorded as a diagnostic channel.

### 9.3 The scheduler

Every component (each sensor, the flight software, each actuator driver, the recorder) has a period that must be an integer multiple of the dynamics step. If it is not, the scenario is refused at load, naming the component. It is never rounded. At OILS and HILS the dynamics step is the rig's `plant_step_s` (§12.3), which replaces the scenario's `dynamics_step_s`. Every period must then be an integer multiple of the rig's step, and the run manifest records both steps. Within a tick the order is fixed:

1. environment at `t`;
2. sensors sample truth; device emulators queue their bytes with the sensor's latency;
3. the flight software steps, if this is its tick;
4. actuator emulators read what the HAL received and turn it into commands, with their latency;
5. the plant integrates to `t + dt`;
6. the recorder samples.

### 9.4 Models

The models are named here and specified in `adcs-sim-core` doc comments, each with its source. Each shared relation calls `adcs-core::physics` (rule 3).

| Area | Model |
|---|---|
| Magnetic field | IGRF-14 to degree 13 from the `igrf14` bundle; a tilted dipole as an option |
| Atmosphere | `atmos-density` table; activity from the scenario |
| Sun and eclipse | low-precision solar ephemeris (Vallado); conical eclipse |
| Disturbances | gravity gradient (full tensor); aerodynamic and solar pressure per face from `[geometry]` faces, defaulting to one plate from the case's scalars; residual dipole; stray field from coils and pump yokes at their mounts, felt by the magnetometer |
| Coil, rod | PWM duty to current through a first-order RL model; current limit; dipole per amp; quantisation; rod residual |
| Fluid ring | state `v`; `H = 2ρAvS`; conduction pump `Δp = nIB/h` or induction slip; laminar friction when Re < 2000 and turbulent above; spin-down; frozen below the melt point (no flow, no torque); electrical power = hydraulic ÷ efficiency; flow-EMF sensor noise; yoke stray field while pumping (IDMAS v2 §03, §07, §09) |
| Reaction wheel | torque and speed limits, Coulomb and viscous friction, motor lag, static and dynamic imbalance as jitter at the wheel's speed |
| RCS | pulses with a minimum impulse bit, on-time quantised to the flight software's rate, misalignment, propellant use, and the force on the orbit |
| Magnetometer | truth plus stray field, bias, scale, misalignment, noise, LSB quantisation, range saturation |
| Sun sensor | field of view, eclipse, noise, bias |
| Star tracker | availability from sun and Earth exclusion and the rate limit; noise across and about the boresight; latency |
| Gyro | angle random walk, rate random walk, scale factor, misalignment, quantisation |
| GNSS receiver | position and velocity noise, time to first fix, outages |

### 9.5 Device emulators: every port speaks bytes

A sensor model produces an engineering value. The device emulator turns it into exactly what the part puts on its port: an I2C register map, a UART frame, a CAN frame, an SPI response, a PWM measurement. The flight software's own drivers read those bytes through `adcs_hal.h`, so its drivers are exercised in SILS exactly as they will be on the OBC. That is why a SILS result is evidence about the flight code, not about a model of it.

Each device's protocol is data: `devices/<part_number>.toml` holds registers, scaling, byte order, framing, CRC, timing and error responses. The same file drives:

- the SILS emulator in `adcs-sim`;
- the OILS emulator in the interface emulation unit (§12.2);
- a golden-byte test that a person confirms against the part's interface control document.

The synthetic parts get invented but complete protocols, so the whole path is testable. A real part's protocol is entered from its ICD by a person, and the file names the ICD revision.

### 9.6 The flight software boundary

- `fsw/` is compiled by `adcs-fsw-abi/build.rs` with `cc`, flags `-std=c99 -O2 -ffp-contract=off -fno-fast-math`. With FMA contraction off, the C arithmetic is repeatable on one platform.
- Flight C code keeps state in statics, so one process holds one flight software instance. A campaign runs its runs in a pool of processes (`adcs-worker` or `adcs sim --jobs N`), never in threads.
- `adcs_fsw_init` must reset every static. A test runs the same scenario twice in one process and requires identical output bytes.
- A customer or company build is loaded as a shared library exporting the `adcs_fsw.h` symbols. The run manifest records `adcs_fsw_build_id()`.
- MATLAB-generated C (Embedded Coder) sits behind a thin wrapper that implements `adcs_fsw.h`. The design workflow stays: MATLAB first, then this SIL.
- The flight code's CI (§18) forbids `malloc`, `time()`, `rand()` and recursion. `cppcheck` runs, and a MISRA subset is checked where a free checker covers it.

### 9.7 Determinism

- `adcs-sim-core` uses `pmath` only. Iteration order is fixed (arrays and `BTreeMap`, never `HashMap`). There is no wall clock.
- Randomness is counter-based. The value for run *k*, parameter path *p*, draw *n* is `SplitMix64(hash(campaign_seed, k, p, n))`. It is independent of evaluation order and of how runs are split across processes.
- **Required:** `adcs-sim-core` trajectories are bit-identical across x86_64, aarch64 and wasm32. CI compares the hash of a golden trajectory.
- The flight C code is required to be repeatable on one platform. Across platforms it may differ, and PIL exists to measure that: the difference is a parity-ledger line, never a failure to hide.

### 9.8 The recorder

A run directory in `adcs-rec/1` format contains:

- `manifest.toml`: scenario hash, case hash, product id, tuned-set hash, catalogue version, flight software build id, engine hash, seed, rung, start and end, and every channel with its unit and rate;
- one CSV per channel group, chunked;
- `events.csv` for mode changes, faults and refusals;
- a content hash over all of it.

Four families of channels are kept apart and never mixed in one column: **truth**, **measured**, **estimated** and **commanded**. The views (§13) read a downsampled stream (at most 20 Hz) while the run is live, and the recorder's files afterwards. Parquet is deferred with the trigger VLEO already wrote: "the first bundle that does not fit comfortably as text".

### 9.9 Performance, measured and never assumed

The builder measures and records, in `docs/BUILD_EVIDENCE.md`, the wall-time ratio of `inertial_hold_3u` nominal: 11,400 s simulated at 0.01 s steps, one core. The builder also measures Monte Carlo runs per hour per core. The first measured figure is recorded beside the layer-1 row "Monte Carlo runs per hour", which is computed from the server cores and is checked against it. The requirement that this SIL beat MATLAB's Monte Carlo throughput on the same scenario is checked by measuring both, never asserted.

### 9.10 The reference flight software

`fsw/` ships a complete, deliberately plain flight software so SILS runs from day one. It covers:

- **Modes:** detumble (B-dot, with a magnetometer read window while the coils are off), sun acquisition, inertial hold, nadir, target track and slew.
- **Estimation:** a MEKF on gyro and star tracker, with a magnetometer-and-sun q-method fallback.
- **Control:**
  - the IDMAS split projection `m = B × τ / |B|²`, `u = −A⁺(τ − m × B)` (IDMAS v2 §12.5, L1);
  - B-dot (IDMAS v2 §12.5, L2);
  - quaternion PD;
  - momentum dump `m += k (h × B) / |B|²`, with the sign as corrected in IDMAS v2 §12.3;
  - the allocation QP of IDMAS v2 §12.2 as a fixed-iteration active-set solver.
- **FDIR:** a failed coil or ring becomes a removed column and the same QP is re-solved (IDMAS v2 §12.6).
- **Drivers:** for the synthetic parts' protocols.

It is C99 with no dynamic memory and unit tests. It is a reference, not the product: the company's flight software replaces it behind the two headers, and the scenarios keep working.
