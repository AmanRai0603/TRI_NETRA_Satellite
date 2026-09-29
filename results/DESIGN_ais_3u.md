# Design loop: ais_3u

Owner: Agastya. `tools/pipeline.py ais_3u` — sizing (Rust, adcs-design) -> SILS mode matrix (Rust engine, POP in the loop,
C flight software) -> assess -> converge, repeated until nothing is left to change; then select, dispatch, Monte Carlo and
soft OILS of the selected configuration. Flow: `docs/figures/flow_design_to_hils.svg`, method: `docs/DESIGN_LOOP.md`.

**Selected: `mtq_fmr` — Magnetorquers + fluid momentum loop — feasible**, converged after 4 iteration(s). Knowledge class: coarse. Sensors: magnetometer, sun_sensors, gyro, gnss, earth_sensor.

## Iterations

| iteration | knobs | options feasible | selected | changes | blocked |
|---:|---|---:|---|---|---|
| 1 | laws as written | 1/25 | mtq (closest (not feasible)) | nadir_pointing/fmr+mtq: power -> electromagnetic pump with more copper (lambda 0.1 -> 0.3 kg/W)<br>nadir_pointing/mtq: fly every mtq_pointing algorithm<br>sun_acquisition/mtq: fly every sun_acquisition algorithm<br>sun_referencing/mtq: fly every mtq_pointing algorithm | 16 |
| 2 | pump lambda 0.3 kg/W | 4/25 | mtq_fmr (feasible) | mtqp: authority x1 -> x1.5 (performance) | 18 |
| 3 | mtqp x1.5, pump lambda 0.3 kg/W | 4/25 | mtq_fmr (feasible) | mtqp: authority back to x1 (no improvement) | 19 |
| 4 | mtqp x1, pump lambda 0.3 kg/W | 4/25 | mtq_fmr (feasible) | — | 19 |

Why the loop stopped (nothing left that a knob can change):

- detumble/rcs: power fails at the sized authority (rcs); the part's standby power is the floor
- mtqp: more authority did not reduce the performance violation (39.9 -> 41); kept at x1
- nadir_pointing/cmg+mtq: power fails at the sized authority (cmg); the part's standby power is the floor
- nadir_pointing/cmg+rcs: power fails at the sized authority (cmg); the part's standby power is the floor
- nadir_pointing/fmr+rcs: power is the thrusters' valve power (RCS dumping), not the pump
- nadir_pointing/rw+mtq: power fails at the sized authority (rw); the part's standby power is the floor
- nadir_pointing/rw+rcs: power fails at the sized authority (rw); the part's standby power is the floor
- nadir_pointing/vscmg+mtq: power fails at the sized authority (vscmg); the part's standby power is the floor
- nadir_pointing/vscmg+rcs: power fails at the sized authority (vscmg); the part's standby power is the floor
- sun_acquisition/cmg: power fails at the sized authority (cmg); the part's standby power is the floor
- sun_acquisition/rw: power fails at the sized authority (rw); the part's standby power is the floor
- sun_acquisition/vscmg: power fails at the sized authority (vscmg); the part's standby power is the floor
- sun_referencing/cmg+mtq: power fails at the sized authority (cmg); the part's standby power is the floor
- sun_referencing/cmg+rcs: power fails at the sized authority (cmg); the part's standby power is the floor
- sun_referencing/fmr+rcs: power is the thrusters' valve power (RCS dumping), not the pump
- sun_referencing/rw+mtq: power fails at the sized authority (rw); the part's standby power is the floor
- sun_referencing/rw+rcs: power fails at the sized authority (rw); the part's standby power is the floor
- sun_referencing/vscmg+mtq: power fails at the sized authority (vscmg); the part's standby power is the floor
- sun_referencing/vscmg+rcs: power fails at the sized authority (vscmg); the part's standby power is the floor

## Every configuration compared (last iteration)

Selection rule (node `select`, docs/NODES.md): least mass_kg, then power_W, then volume_L, then simplicity among feasible solution families. The benchmarks are ranked by the same rule; best benchmark: **`mtq_rw`** (closest (not feasible)).

| family | role | rank | feasible | mass [kg] | power [W] | volume [L] | momentum actuator | gaps |
|---|---|---:|---|---:|---:|---:|---|---|
| mtq_fmr | solution | 1 | yes | 1.446 | 2.54 | 0.537 | fluid loop (3 rings) | — |
| mtq | solution | — | no | 0.236 | 2.06 | 0.018 | coils only | sun_acquisition: sun_acquisition_time (performance), sun_angle_p95 (performance); sun_referencing: sun_ape_p9973 (performance); nadir_pointing: ape_los_p9973 (performance) |
| mtq_fmr_rcs | solution | — | no | 1.965 | 2.59 | 1.519 | fluid loop (3 rings) | budget: mass_kg 1.96 > 1.6; budget: volume_L 1.52 > 1 |
| mtq_rw | benchmark | — | no | 0.393 | 2.73 | 0.070 | CAT-CUBESPACE-CUBEWHEEL-CW0017 x3 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power) |
| mtq_rw_rcs | benchmark | — | no | 0.911 | 2.78 | 1.052 | CAT-CUBESPACE-CUBEWHEEL-CW0017 x3 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power); budget: volume_L 1.05 > 1 |
| mtq_cmg | benchmark | — | no | 1.413 | 5.83 | 0.809 | CAT-TENSOR-TECH-ADCS400 x4 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power) |
| mtq_vscmg | benchmark | — | no | 1.413 | 5.83 | 0.809 | CAT-TENSOR-TECH-ADCS400-VSCMG x4 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power) |
| mtq_cmg_rcs | benchmark | — | no | 1.931 | 5.88 | 1.791 | CAT-TENSOR-TECH-ADCS400 x4 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power); budget: mass_kg 1.93 > 1.6; budget: volume_L 1.79 > 1 |
| mtq_vscmg_rcs | benchmark | — | no | 1.931 | 5.88 | 1.791 | CAT-TENSOR-TECH-ADCS400-VSCMG x4 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power); budget: mass_kg 1.93 > 1.6; budget: volume_L 1.79 > 1 |

## Selected methods (mtq_fmr)

| mode | option | feasible | objective | algorithms | failing |
|---|---|---|---:|---|---|
| detumble | mtq | yes | 48.74 detumble_time | attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, sun_acquisition=sunspin_l1l2 | — |
| sun_acquisition | fmr | yes | 2.292 sun_acquisition_time | allocation=idmas_split, attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, pointing=pid, sun_acquisition=sunspin_l1l2 | — |
| sun_referencing | fmr+mtq | yes | 0.2513 sun_ape_p9973 | allocation=idmas_split, attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, pointing=pid, sun_acquisition=sunspin_l1l2 | — |
| nadir_pointing | fmr+mtq | yes | 0.1514 ape_los_p9973 | allocation=idmas_split, attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, pointing=pid, sun_acquisition=sunspin_l1l2 | — |

## Every solution family flown as the mission (node `family_missions`)

Detumble -> Sun acquisition -> nadir with each family's best methods from the loop, C flight software (C = Rust bit for bit), and the Monte Carlo pass rate of each requirement metric.

| family | selected | feasible | mass [kg] | methods | detumble_time | ape_los_p9973 | ake_los_p9973 | power_mean | C = Rust | MC pass rates |
|---|---|---|---:|---|---:|---:|---:|---:|---|---|
| mtq | no | no | 0.236 | detumble=mtq, sun_acquisition=mtq, sun_referencing=mtq, nadir_pointing=mtq | 49.96 ✓ | 174.2 ✗ | 1.479 ✓ | 0.1495 ✓ | True | detumble_time 100 %; ape_los_p9973 0 %; ake_los_p9973 83 %; power_mean 100 % |
| mtq_fmr | yes | yes | 1.446 | detumble=mtq, sun_acquisition=fmr, sun_referencing=fmr+mtq, nadir_pointing=fmr+mtq | 53.46 ✓ | 0.1516 ✓ | 0.15 ✓ | 0.233 ✓ | True | detumble_time 100 %; ape_los_p9973 100 %; ake_los_p9973 100 %; power_mean 100 % |
| mtq_fmr_rcs | no | no | 1.965 | detumble=mtq, sun_acquisition=fmr, sun_referencing=fmr+mtq, nadir_pointing=fmr+mtq | 53.46 ✓ | 0.1516 ✓ | 0.1499 ✓ | 0.2791 ✓ | True | detumble_time 100 %; ape_los_p9973 100 %; ake_los_p9973 100 %; power_mean 100 % |

## Mode matrix (last iteration, best algorithm per option)

| mode | option | algorithm | feasible | objective | failing (cause) |
|---|---|---|---|---:|---|
| detumble | mtq | default | yes | 48.74 | — |
| detumble | rcs | default | no | 1.875 | power_peak (power) |
| nadir_pointing | cmg+mtq | default | no | 0.1481 | power_mean (power) |
| nadir_pointing | cmg+rcs | default | no | 0.1481 | power_mean (power) |
| nadir_pointing | fmr+mtq | default | yes | 0.1514 | — |
| nadir_pointing | fmr+rcs | default | no | 0.1513 | power_mean (power) |
| nadir_pointing | mtq | mtq_smc | no | 11.85 | ape_los_p9973 (performance) |
| nadir_pointing | rw+mtq | default | no | 0.1482 | power_mean (power) |
| nadir_pointing | rw+rcs | default | no | 0.1481 | power_mean (power) |
| nadir_pointing | vscmg+mtq | default | no | 0.1481 | power_mean (power) |
| nadir_pointing | vscmg+rcs | default | no | 0.1481 | power_mean (power) |
| sun_acquisition | cmg | default | no | 2.275 | power_mean (power) |
| sun_acquisition | fmr | default | yes | 2.292 | — |
| sun_acquisition | mtq | sunspin_damped | no | — | sun_acquisition_time (performance), sun_angle_p95 (performance) |
| sun_acquisition | rw | default | no | 2.292 | power_mean (power) |
| sun_acquisition | vscmg | default | no | 2.392 | power_mean (power) |
| sun_referencing | cmg+mtq | default | no | 0.2464 | power_mean (power) |
| sun_referencing | cmg+rcs | default | no | 0.2464 | power_mean (power) |
| sun_referencing | fmr+mtq | default | yes | 0.2513 | — |
| sun_referencing | fmr+rcs | default | no | 0.2512 | power_mean (power) |
| sun_referencing | mtq | mtq_rate_damp | no | 127.2 | sun_ape_p9973 (performance) |
| sun_referencing | rw+mtq | default | no | 0.2468 | power_mean (power) |
| sun_referencing | rw+rcs | default | no | 0.2463 | power_mean (power) |
| sun_referencing | vscmg+mtq | default | no | 4.875 | power_mean (power) |
| sun_referencing | vscmg+rcs | default | no | 2.536 | power_mean (power) |

## Monte Carlo of the dispatched mission (12 runs; dispersions: inertia, residual_dipole, cm_offset, solar_flux, kp, accommodation, reflectivity, arg_lat_deg)

| metric | req | mean ± std | [min, max] | pass rate |
|---|---:|---|---|---:|
| detumble_time (min) | 284 | 56.77 ± 22.3 | [7.792, 92.28] | 100 % |
| ape_los_p9973 (deg) | 10 | 0.2426 ± 0.0731 | [0.1603, 0.3825] | 100 % |
| ake_los_p9973 (deg) | 5 | 0.2393 ± 0.0732 | [0.1574, 0.3818] | 100 % |
| power_mean (W) | 0.5 | 0.3186 ± 0.0923 | [0.2113, 0.493] | 100 % |
| power_peak (W) | — | 0.859 ± 0.0129 | [0.8384, 0.8847] | — |
| propellant (g) | — | 0 ± 0 | [0, 0] | — |

## SILS and soft OILS of the dispatched mission

The same scenario, product and blob: in-process C (SILS) and the flight software as Cortex-M4F firmware in QEMU,
C and Rust builds, with exact per-step instruction counts and the command latency applied (docs/SOFT_OILS.md).

| metric | req | SILS | soft OILS (C on M4F) | soft OILS (Rust on M4F) |
|---|---:|---:|---:|---:|
| detumble_time | 284 | 53.46 ✓ | 53.46 ✓ | 53.46 ✓ |
| ape_los_p9973 | 10 | 0.1516 ✓ | 0.1217 ✓ | 0.1274 ✓ |
| ake_los_p9973 | 5 | 0.15 ✓ | 0.1184 ✓ | 0.1234 ✓ |
| power_mean | 0.5 | 0.233 ✓ | 0.2175 ✓ | 0.2189 ✓ |
| power_peak | — | 0.8385  | 0.85  | 0.8508  |
| propellant | — | 0  | 0  | 0  |

| OBC build | instructions/step mean / max | exec max [ms] | latency mean / max [ms] | CPU load max | overruns |
|---|---:|---:|---:|---:|---:|
| C (arm-none-eabi-gcc -O2) | 1.624e+05 / 8.24e+05 | 6.131 | 2.377 / 7.300 | 6.1 % | 0 |
| Rust (thumbv7em-none-eabihf) | 2.174e+05 / 9.609e+05 | 7.149 | 2.787 / 8.318 | 7.1 % | 0 |

Dispatch: `dist/dispatch/ais_3u/mtq_fmr/converged` (blob, sized products, BUILD.md). C = Rust flight software bitwise on the engine: True.

