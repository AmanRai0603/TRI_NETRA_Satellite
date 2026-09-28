# From a customer's case to a dispatched ADCS — the solution pipeline

**Owner: Agastya.** Copyright (c) 2026 Agastya. All rights reserved.

This is the structure the SILS is organised around. A customer gives one case
(`adcs-case/1`: orbit, mass properties, surfaces, requirements). The pipeline
answers one question: **which of our three ADCS solutions flies this satellite,
and how does it compare with the standard actuators, on every parameter?**

```
 customer case (adcs-case/1)
        │
 1  DEMAND      disturbance survey on the precision orbit, detumble momentum,
        │       slew agility, field strength along the orbit      asils.sizing.demand
 2  SIZING      every actuator option sized to that demand:
        │       ours       MTQ, fluid loop (FMR), N2O cold-gas RCS
        │       benchmark  reaction wheel, CMG, VSCMG             asils.sizing.*
        │       -> sized parts + one product per family, mass / power / volume /
        │          propellant / jitter source for each
 3  MODES       every mission mode flown with every method its hardware allows
        │       (mode x option matrix, same seeds)                asils.solution.evaluate
        │         detumble          MTQ | RCS
        │         sun acquisition   MTQ | RW | CMG | VSCMG | FMR
        │         sun referencing   MTQ | RW | CMG | VSCMG | FMR   x  dump by MTQ | RCS
        │         nadir pointing    MTQ | RW | CMG | VSCMG | FMR   x  dump by MTQ | RCS
 4  SOLUTION    our families, simplest first:  mtq  ->  mtq_fmr  ->  mtq_fmr_rcs
        │       the first whose best method passes every mode is RECOMMENDED;
        │       the benchmark families are scored the same way, for comparison only
 5  DISPATCH    the recommended solution: sized product, per-mode method and
                algorithm set, FSW parameters, HAL frame map   asils.solution.dispatch
                -> SILS evidence -> OILS (flight OBC on the HAL link) -> HILS rigs
```

## 1. Mission modes (data: `catalogue/modes/*.toml`)

A mode is **what the satellite must achieve**; a method is **how** (which actuator
and algorithms). Modes are data, so adding one (target tracking, safe hold,
orbit-manoeuvre attitude) is a new file, not new code.

| mode | goal | navigation | guidance | requirement (case key, else mode default) |
|---|---|---|---|---|
| `detumble` | body rate from the tumble to below 0.5 °/s | magnetometer, gyro | none (rate only) | `req.detumble` |
| `sun_acquisition` | the Sun on the power face from any attitude, no attitude solution needed | Sun sensors (fine or coarse), gyro, magnetometer | Sun vector only | `req.sunacq` (time), Sun angle within 20° |
| `sun_referencing` | three-axis Sun-pointing hold: power face on the Sun, a second axis on the orbit normal | Sun sensors, magnetometer, gyro, **star tracker**, MEKF | `sun` | Sun-pointing APE (default 5°), AKE |
| `nadir_pointing` | payload axis on nadir | star tracker, gyro, Sun, magnetometer, Earth sensor, MEKF | `nadir` | `req.ape`, `req.ake`, `req.rks` (jitter) |

Sun referencing and nadir pointing share one control structure. Only the
guidance law changes, and nadir pointing adds its stability (jitter)
constraint. The power face is the product's `sun_axis_body` (default −Z,
as in the Standard Code Sun spin).

### Methods per mode

| mode | option | actuators doing the job | momentum management | algorithms (registry slots) |
|---|---|---|---|---|
| detumble | `mtq` | coils | — | `detumble` (B-dot family, L1) |
| detumble | `rcs` | thrusters | — | `rcs_rate` |
| sun acquisition | `mtq` | coils | — | `sun_acquisition` slot: `sunspin_l1l2` or `sunspin_damped` (L1 spin-up + L2 He et al.) |
| sun acquisition | `rw` `cmg` `vscmg` `fmr` | that rotor set | coils | `sun_acq_rotor` + allocation |
| sun referencing, nadir | `mtq` | coils | — | `mtq_pointing` |
| sun referencing, nadir | `rw` `cmg` `vscmg` `fmr` | that rotor set | coils **or** RCS (`+mtq` / `+rcs`) | `pointing` + allocation + `thrusters` |

Every option is flown on its own **sized** hardware, from the same start
conditions and the same seeds, so a table row compares actuators and nothing else.

## 2. Solutions and benchmarks (data: `catalogue/families.toml`)

| family | role | what it is |
|---|---|---|
| `mtq` | **solution** | magnetorquers only |
| `mtq_fmr` | **solution** | magnetorquers + fluid momentum loop |
| `mtq_fmr_rcs` | **solution** | magnetorquers + fluid loop + N2O cold-gas RCS |
| `mtq_rw`, `mtq_rw_rcs` | benchmark | reaction wheels (with RCS dumping) |
| `mtq_cmg`, `mtq_vscmg` | benchmark | control-moment gyros |

The coils are in every family: they are the safety floor. The benchmark
families are never dispatched. They exist so that every figure of ours
(pointing, jitter, power, mass, volume, propellant, time to acquire) is quoted
next to what a standard actuator of the **same sizing** would give. If one day
the company builds its own wheel or CMG, it becomes a solution family by
changing its `role`.

A family's per-mode method is the best of the options its hardware allows
(e.g. `mtq_fmr_rcs` detumbles with `mtq` or `rcs`, whichever the mode matrix
ranks first). Recommendation rule: **the simplest solution family that passes
every mode**, then worst margin, mass and power, as in SPEC §8.6.

## 3. Sizing (`+asils/+sizing`)

`demand` flies a disturbance survey on the case's orbit (POP in the loop, the
same torque models as the SILS) at the nadir and Sun-referenced attitudes. It
returns the peak disturbance torque, the momentum each orbit builds up (cyclic
and secular), the weakest field along the orbit, the detumble momentum
`I·w0`, and the slew momentum and torque from `mission.sangle` and `req.slew`.

Every actuator is then sized to that demand with stated margins (momentum ×2,
torque ×1.5; the case's `req.hsat` overrides the momentum margin). Each
sizing law is physical and written in its file:

| actuator | sized quantity | model behind mass / power / volume | jitter source |
|---|---|---|---|
| MTQ | dipole for dumping and detumble | air-core coil: dipole = N I A, P ∝ m² at fixed copper | none |
| fluid loop (FMR) | loop momentum `h = ρ A 2S v` | galinstan loop in the 3U face, laminar loss, MHD pump P = ΔP·Q/η | no moving parts: pump ripple only |
| N2O cold-gas RCS | thrust = torque / arm; propellant for detumble, dumping, slews over the life | Isp 60–80 s (nominal 70 s), self-pressurised N2O tank ~50 bar, Al-7075 vessel | pulse impulses during firing |
| reaction wheel | h, τ per axis | rim flywheel at 6000 rpm, ISO 1940 balance grade G2.5 | static and dynamic imbalance at wheel speed |
| CMG, VSCMG | pyramid rotor h0, gimbal rate | constant-speed rotor + gimbal motor | rotor imbalance at constant speed |

Sized parts are written to `store/sized/<case>/parts`, and one product per family
to `store/sized/<case>/products`. They are *sized*, not catalogue parts: they
say how big a standard part would have to be, and so how our product compares.

**Jitter** is evaluated in the frequency domain (`asils.sizing.jitter`): an
imbalance force or torque at a wheel speed Ω far above the control bandwidth
moves the body as a free mass, θ = τ/(I Ω²). It is evaluated over the
recorded wheel-speed history and combined with the simulated rate stability.

## 4. Components (`catalogue/components`, `+asils/+comp`, `docs/COMPONENTS.md`)

Every sensor and actuator has two separate things:

* the **model** the SILS uses (`+asils/+devices`): truth in, device output out,
  with the part's errors. It stands in for the hardware at every rung below HILS.
* the **component chain** (`+asils/+comp/+<component>`): the processing
  that runs *inside* the unit or in its driver on the OBC. This is where
  in-house development goes (image processing, centroiding, star
  identification, horizon detection, flow control). It is packaged per
  component and later ported to the unit's firmware or the OBC.

| component | in-house | chain (node → node → output) | output on the HAL |
|---|---|---|---|
| star tracker | planned | image → centroid → star ID → attitude (QUEST) | quaternion + validity |
| Sun sensor | planned | photodiode currents → α, β → Sun unit vector | Sun vector + validity |
| Earth sensor | planned | IR horizon crossings → horizon fit → nadir vector | nadir vector + validity |
| magnetometer | bought | raw → calibration (bias, scale) | field vector |
| gyro | bought | raw → bias / scale calibration | rate |
| magnetorquer | in-house | dipole command → PWM duty (residual-dipole compensation) | coil duty |
| fluid loop | in-house | momentum command → flow servo → pump drive | ring momentum |
| RCS (N2O) | in-house | torque request → valve schedule (MIB, PWM) | valve on-times |
| RW / CMG / VSCMG | benchmark | torque / gimbal-rate command → motor drive | tach / encoder |

Each chain node carries a status: `implemented`, `baseline` (a standard method
standing in until the in-house one arrives) or `stub`. The SILS runs any
component at one of three **levels**: `model` (fast, statistics only),
`chain` (the component's algorithms in the loop), or `hil` (the real unit
through the HAL). The component's interface to the rest of the ADCS is always
its HAL frame, so replacing one level with another changes nothing else.

## 5. Dispatch and the rungs

`asils.solution.dispatch(case)` writes `dist/dispatch/<case>/<family>/`:

* `product.json`: the sized hardware.
* `modes.json`: per mode, the method and algorithm set, and the transition rules.
* `fsw_params.json` and `adcs_fsw_params.h`: the tuned gains, for the OBC build.
* `hal_map.json`: which frames and fields each component uses.
* `README.md`: the evidence (the SILS runs), then the OILS and HILS steps (`docs/OILS_HILS.md`).

From there the same algorithms go SILS → algorithm hardening → OILS (the flight
OBC on the HAL link, `hal__backend = udp`) → HILS (the real component replacing
its model, one at a time).
