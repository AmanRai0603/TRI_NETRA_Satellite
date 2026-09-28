# Design loop: ais_3u

Owner: Agastya. `tools/pipeline.py ais_3u` — sizing (Rust, adcs-design) -> SILS mode matrix (Rust engine, POP in the loop,
C flight software) -> assess -> converge, repeated until nothing is left to change; then select, dispatch, Monte Carlo and
soft OILS of the selected configuration. Flow: `docs/figures/flow_design_to_hils.svg`, method: `docs/DESIGN_LOOP.md`.

**Selected: `mtq_fmr` — Magnetorquers + fluid momentum loop — closest (not feasible)**, converged after 4 iteration(s). Knowledge class: coarse. Sensors: magnetometer, sun_sensors, gyro, gnss, earth_sensor.

Open requirement gaps of the selected family (what the case must relax, or the next design lever):

- budget: mass_kg 1.6 > 0.35
- budget: volume_L 0.537 > 0.3

## Iterations

| iteration | knobs | options feasible | selected | changes | blocked |
|---:|---|---:|---|---|---|
| 1 | laws as written | 1/25 | mtq (closest (not feasible)) | nadir_pointing/fmr+mtq: power -> electromagnetic pump with more copper (lambda 0.1 -> 0.3 kg/W)<br>nadir_pointing/mtq: fly every mtq_pointing algorithm<br>sun_acquisition/mtq: fly every sun_acquisition algorithm<br>sun_referencing/mtq: fly every mtq_pointing algorithm<br>sun_referencing/vscmg+mtq: fly every pointing algorithm<br>sun_referencing/vscmg+rcs: fly every pointing algorithm | 14 |
| 2 | pump lambda 0.3 kg/W | 4/25 | mtq_fmr (closest (not feasible)) | mass (mtq_fmr): fluid-loop momentum x1 -> x0.75<br>mtqp: authority x1 -> x1.5 (performance) | 18 |
| 3 | fmr x0.75, mtqp x1.5, pump lambda 0.3 kg/W | 3/25 | mtq (closest (not feasible)) | mtqp: authority back to x1 (no improvement)<br>sun_referencing/fmr+mtq: power -> electromagnetic pump with more copper (lambda 0.3 -> 0.9 kg/W)<br>mass lever scale.fmr undone: it broke sun_referencing/fmr+mtq, requirement violation 0 -> 0.0247 | 18 |
| 4 | fmr x1, mtqp x1, pump lambda 0.9 kg/W | 6/25 | mtq_fmr (closest (not feasible)) | — | 18 |

Why the loop stopped (nothing left that a knob can change):

- detumble/rcs: power fails at the sized authority (rcs); the part's standby power is the floor
- mass (mtq_fmr): no lever left (budget: mass_kg 1.6 > 0.35 budget: volume_L 0.537 > 0.3)
- mtqp: more authority did not reduce the performance violation (39.9 -> 41); kept at x1
- nadir_pointing/cmg+mtq: power fails at the sized authority (cmg); the part's standby power is the floor
- nadir_pointing/cmg+rcs: power fails at the sized authority (cmg); the part's standby power is the floor
- nadir_pointing/rw+mtq: power fails at the sized authority (rw); the part's standby power is the floor
- nadir_pointing/rw+rcs: power fails at the sized authority (rw); the part's standby power is the floor
- nadir_pointing/vscmg+mtq: power fails at the sized authority (vscmg); the part's standby power is the floor
- nadir_pointing/vscmg+rcs: power fails at the sized authority (vscmg); the part's standby power is the floor
- sun_acquisition/cmg: power fails at the sized authority (cmg); the part's standby power is the floor
- sun_acquisition/rw: power fails at the sized authority (rw); the part's standby power is the floor
- sun_acquisition/vscmg: power fails at the sized authority (vscmg); the part's standby power is the floor
- sun_referencing/cmg+mtq: power fails at the sized authority (cmg); the part's standby power is the floor
- sun_referencing/cmg+rcs: power fails at the sized authority (cmg); the part's standby power is the floor
- sun_referencing/rw+mtq: power fails at the sized authority (rw); the part's standby power is the floor
- sun_referencing/rw+rcs: power fails at the sized authority (rw); the part's standby power is the floor
- sun_referencing/vscmg+mtq: performance and power both fail — no authority change helps
- sun_referencing/vscmg+rcs: power fails at the sized authority (vscmg); the part's standby power is the floor

## Families (last iteration)

| family | role | feasible | mass [kg] | power [W] | volume [L] | gaps |
|---|---|---|---:|---:|---:|---|
| mtq | solution | no | 0.236 | 2.06 | 0.018 | sun_acquisition: sun_acquisition_time (performance), sun_angle_p95 (performance); sun_referencing: sun_ape_p9973 (performance); nadir_pointing: ape_los_p9973 (performance) |
| mtq_fmr | solution | no | 1.600 | 2.24 | 0.537 | budget: mass_kg 1.6 > 0.35; budget: volume_L 0.537 > 0.3 |
| mtq_fmr_rcs | solution | no | 2.119 | 2.29 | 1.519 | budget: mass_kg 2.12 > 0.35; budget: volume_L 1.52 > 0.3 |
| mtq_rw | benchmark | no | 0.476 | 2.56 | 0.141 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power); budget: mass_kg 0.476 > 0.35 |
| mtq_rw_rcs | benchmark | no | 0.995 | 2.61 | 1.124 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power); budget: mass_kg 0.995 > 0.35; budget: volume_L 1.12 > 0.3 |
| mtq_cmg | benchmark | no | 0.560 | 2.75 | 0.275 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power); budget: mass_kg 0.56 > 0.35 |
| mtq_cmg_rcs | benchmark | no | 1.079 | 2.80 | 1.257 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power); budget: mass_kg 1.08 > 0.35; budget: volume_L 1.26 > 0.3 |
| mtq_vscmg | benchmark | no | 0.595 | 3.15 | 0.275 | sun_acquisition: power_mean (power); sun_referencing: sun_ape_p9973 (performance); nadir_pointing: power_mean (power); budget: mass_kg 0.595 > 0.35 |
| mtq_vscmg_rcs | benchmark | no | 1.114 | 3.20 | 1.257 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power); budget: mass_kg 1.11 > 0.35; budget: volume_L 1.26 > 0.3 |

## Selected methods (mtq_fmr)

| mode | option | feasible | objective | algorithms | failing |
|---|---|---|---:|---|---|
| detumble | mtq | yes | 48.74 detumble_time | attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, sun_acquisition=sunspin_l1l2 | — |
| sun_acquisition | fmr | yes | 2.292 sun_acquisition_time | allocation=idmas_split, attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, pointing=pid, sun_acquisition=sunspin_l1l2 | — |
| sun_referencing | fmr+mtq | yes | 0.2513 sun_ape_p9973 | allocation=idmas_split, attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, pointing=pid, sun_acquisition=sunspin_l1l2 | — |
| nadir_pointing | fmr+mtq | yes | 0.1514 ape_los_p9973 | allocation=idmas_split, attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, pointing=pid, sun_acquisition=sunspin_l1l2 | — |

## Mode matrix (last iteration, best algorithm per option)

| mode | option | algorithm | feasible | objective | failing (cause) |
|---|---|---|---|---:|---|
| detumble | mtq | default | yes | 48.74 | — |
| detumble | rcs | default | no | 1.875 | power_peak (power) |
| nadir_pointing | cmg+mtq | default | no | 0.148 | power_mean (power) |
| nadir_pointing | cmg+rcs | default | no | 0.1481 | power_mean (power) |
| nadir_pointing | fmr+mtq | default | yes | 0.1514 | — |
| nadir_pointing | fmr+rcs | default | yes | 0.1513 | — |
| nadir_pointing | mtq | mtq_smc | no | 11.85 | ape_los_p9973 (performance) |
| nadir_pointing | rw+mtq | default | no | 0.1481 | power_mean (power) |
| nadir_pointing | rw+rcs | default | no | 0.1481 | power_mean (power) |
| nadir_pointing | vscmg+mtq | default | no | 0.1556 | power_mean (power) |
| nadir_pointing | vscmg+rcs | default | no | 0.1518 | power_mean (power) |
| sun_acquisition | cmg | default | no | 2.275 | power_mean (power) |
| sun_acquisition | fmr | default | yes | 2.292 | — |
| sun_acquisition | mtq | sunspin_damped | no | — | sun_acquisition_time (performance), sun_angle_p95 (performance) |
| sun_acquisition | rw | default | no | 2.292 | power_mean (power) |
| sun_acquisition | vscmg | default | no | 2.958 | power_mean (power) |
| sun_referencing | cmg+mtq | default | no | 0.2467 | power_mean (power) |
| sun_referencing | cmg+rcs | default | no | 0.2467 | power_mean (power) |
| sun_referencing | fmr+mtq | default | yes | 0.2513 | — |
| sun_referencing | fmr+rcs | default | yes | 0.2512 | — |
| sun_referencing | mtq | mtq_rate_damp | no | 127.2 | sun_ape_p9973 (performance) |
| sun_referencing | rw+mtq | default | no | 0.2468 | power_mean (power) |
| sun_referencing | rw+rcs | default | no | 0.4947 | power_mean (power) |
| sun_referencing | vscmg+mtq | smc | no | 5.757 | sun_ape_p9973 (performance), power_mean (power) |
| sun_referencing | vscmg+rcs | pid@bw4 | no | 1.238 | power_mean (power) |

## Monte Carlo of the dispatched mission (12 runs; dispersions: inertia, residual_dipole, cm_offset, solar_flux, kp, accommodation, reflectivity, arg_lat_deg)

| metric | req | mean ± std | [min, max] | pass rate |
|---|---:|---|---|---:|
| detumble_time (min) | 284 | 58.22 ± 22.1 | [9.858, 94.17] | 100 % |
| ape_los_p9973 (deg) | 10 | 0.2426 ± 0.0731 | [0.1603, 0.3825] | 100 % |
| ake_los_p9973 (deg) | 5 | 0.2393 ± 0.0732 | [0.1574, 0.3818] | 100 % |
| power_mean (W) | 0.5 | 0.2085 ± 0.0528 | [0.1472, 0.3085] | 100 % |
| power_peak (W) | — | 0.5628 ± 0.0117 | [0.5459, 0.5875] | — |
| propellant (g) | — | 0 ± 0 | [0, 0] | — |

## SILS and soft OILS of the dispatched mission

The same scenario, product and blob: in-process C (SILS) and the flight software as Cortex-M4F firmware in QEMU,
C and Rust builds, with exact per-step instruction counts and the command latency applied (docs/SOFT_OILS.md).

| metric | req | SILS | soft OILS (C on M4F) | soft OILS (Rust on M4F) |
|---|---:|---:|---:|---:|
| detumble_time | 284 | 55.64 ✓ | 55.71 ✓ | 55.71 ✓ |
| ape_los_p9973 | 10 | 0.1516 ✓ | 0.1217 ✓ | 0.1274 ✓ |
| ake_los_p9973 | 5 | 0.15 ✓ | 0.1184 ✓ | 0.1234 ✓ |
| power_mean | 0.5 | 0.1593 ✓ | 0.1501 ✓ | 0.1509 ✓ |
| power_peak | — | 0.5414  | 0.5528  | 0.5536  |
| propellant | — | 0  | 0  | 0  |

| OBC build | instructions/step mean / max | exec max [ms] | latency mean / max [ms] | CPU load max | overruns |
|---|---:|---:|---:|---:|---:|
| C (arm-none-eabi-gcc -O2) | 1.624e+05 / 8.24e+05 | 6.131 | 2.377 / 7.300 | 6.1 % | 0 |
| Rust (thumbv7em-none-eabihf) | 2.174e+05 / 9.609e+05 | 7.149 | 2.787 / 8.318 | 7.1 % | 0 |

Dispatch: `dist/dispatch/ais_3u/mtq_fmr/converged` (blob, sized products, BUILD.md). C = Rust flight software bitwise on the engine: True.

