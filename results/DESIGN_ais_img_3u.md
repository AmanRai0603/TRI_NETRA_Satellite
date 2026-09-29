# Design loop: ais_img_3u

Owner: Agastya. `tools/pipeline.py ais_img_3u` — sizing (Rust, adcs-design) -> SILS mode matrix (Rust engine, POP in the loop,
C flight software) -> assess -> converge, repeated until nothing is left to change; then select, dispatch, Monte Carlo and
soft OILS of the selected configuration. Flow: `docs/figures/flow_design_to_hils.svg`, method: `docs/DESIGN_LOOP.md`.

**Selected: `mtq_fmr` — Magnetorquers + fluid momentum loop — feasible**, converged after 9 iteration(s). Knowledge class: fine. Sensors: star_tracker, magnetometer, sun_sensors, gyro, gnss, earth_sensor.

## Iterations

| iteration | knobs | options feasible | selected | changes | blocked |
|---:|---|---:|---|---|---|
| 1 | laws as written | 9/25 | mtq_fmr (closest (not feasible)) | nadir_pointing/fmr+mtq: fly every pointing algorithm<br>nadir_pointing/fmr+rcs: fly every pointing algorithm<br>nadir_pointing/mtq: fly every mtq_pointing algorithm<br>sun_acquisition/mtq: fly every sun_acquisition algorithm<br>sun_referencing/mtq: fly every mtq_pointing algorithm | 13 |
| 2 | laws as written | 9/25 | mtq_fmr (closest (not feasible)) | rate stability: gyro noise x1 -> x0.3 (fibre-optic class)<br>mtqp: authority x1 -> x1.5 (performance) | 14 |
| 3 | mtqp x1.5, gyro noise x0.3 | 9/25 | mtq_fmr (closest (not feasible)) | gyro grade back to x1: rate-stability violation 13.8 -> 13.8<br>mtqp: authority back to x1 (no improvement)<br>nadir_pointing/fmr+mtq: fluid-loop flow sensor 2 -> 0.5 mm/s (1 sigma) | 15 |
| 4 | mtqp x1, flow sensor 0.5 mm/s | 10/25 | mtq_fmr (closest (not feasible)) | mass (mtq_fmr): one star-tracker head instead of two | 14 |
| 5 | mtqp x1, 1 ST head, flow sensor 0.5 mm/s | 7/25 | mtq_fmr (closest (not feasible)) | nadir_pointing/cmg+mtq: fly every pointing algorithm<br>nadir_pointing/cmg+rcs: fly every pointing algorithm<br>nadir_pointing/fmr+mtq: fluid-loop flow sensor 0.5 -> 0.125 mm/s (1 sigma)<br>nadir_pointing/rw+mtq: fly every pointing algorithm<br>nadir_pointing/rw+rcs: fly every pointing algorithm<br>nadir_pointing/vscmg+mtq: fly every pointing algorithm<br>nadir_pointing/vscmg+rcs: fly every pointing algorithm<br>mass lever st_heads undone: it broke nadir_pointing/fmr+mtq, requirement violation 0 -> 41.1 | 17 |
| 6 | mtqp x1, flow sensor 0.125 mm/s | 12/25 | mtq_fmr (closest (not feasible)) | mass (mtq_fmr): lighter pump, less copper (lambda 0.1 -> 0.0333333 kg/W) | 12 |
| 7 | mtqp x1, pump lambda 0.0333333 kg/W, flow sensor 0.125 mm/s | 11/25 | mtq_fmr (feasible) | — | 13 |
| 8 | mtqp x1, pump lambda 0.1 kg/W, flow sensor 0.125 mm/s | 12/25 | mtq_fmr (closest (not feasible)) | mass (mtq_fmr): fluid-loop momentum x1 -> x0.75 | 12 |
| 9 | mtqp x1, fmr x0.75, pump lambda 0.1 kg/W, flow sensor 0.125 mm/s | 11/25 | mtq_fmr (feasible) | — | 13 |

**Robustness (node `mc`) after iteration 7:** the Monte Carlo of `mtq_fmr` failed power_mean (75 % of runs pass). robustness (mtq_fmr): mean power fails in dispersed runs -> pump with more copper (lambda 0.0333333 -> 0.1 kg/W); lighter-pump lever closed

Why the loop stopped (nothing left that a knob can change):

- mtqp: more authority did not reduce the performance violation (1.33e+04 -> 1.34e+04); kept at x1
- nadir_pointing/cmg+mtq: power fails at the sized authority (cmg); the part's standby power is the floor
- nadir_pointing/cmg+rcs: power fails at the sized authority (cmg); the part's standby power is the floor
- nadir_pointing/mtq: knowledge fails with the star tracker fitted
- nadir_pointing/vscmg+mtq: power fails at the sized authority (vscmg); the part's standby power is the floor
- nadir_pointing/vscmg+rcs: power fails at the sized authority (vscmg); the part's standby power is the floor
- sun_acquisition/cmg: power fails at the sized authority (cmg); the part's standby power is the floor
- sun_acquisition/vscmg: power fails at the sized authority (vscmg); the part's standby power is the floor
- sun_referencing/cmg+mtq: power fails at the sized authority (cmg); the part's standby power is the floor
- sun_referencing/cmg+rcs: power fails at the sized authority (cmg); the part's standby power is the floor
- sun_referencing/fmr+rcs: power is the thrusters' valve power (RCS dumping), not the pump
- sun_referencing/vscmg+mtq: power fails at the sized authority (vscmg); the part's standby power is the floor
- sun_referencing/vscmg+rcs: power fails at the sized authority (vscmg); the part's standby power is the floor

## Every configuration compared (last iteration)

Selection rule (node `select`, docs/NODES.md): least mass_kg, then power_W, then volume_L, then simplicity among feasible solution families. The benchmarks are ranked by the same rule; best benchmark: **`mtq_rw`** (feasible).

| family | role | rank | feasible | mass [kg] | power [W] | volume [L] | momentum actuator | gaps |
|---|---|---:|---|---:|---:|---:|---|---|
| mtq_fmr | solution | 1 | yes | 1.534 | 6.46 | 0.534 | fluid loop (3 rings) | — |
| mtq | solution | — | no | 0.830 | 3.90 | 0.012 | coils only | sun_acquisition: sun_acquisition_time (performance), sun_angle_p95 (performance); sun_referencing: sun_ape_p9973 (performance); nadir_pointing: ape_los_p9973 (performance), ake_los_p9973 (knowledge), rate_stability_p9973 (performance) |
| mtq_fmr_rcs | solution | — | no | 2.000 | 6.51 | 1.376 | fluid loop (3 rings) | budget: mass_kg 2 > 1.6; budget: volume_L 1.38 > 1 |
| mtq_rw | benchmark | 1 | yes | 0.996 | 4.66 | 0.068 | CAT-CUBESPACE-CUBEWHEEL-CW0017 x3 | — |
| mtq_rw_rcs | benchmark | 2 | yes | 1.462 | 4.71 | 0.909 | CAT-CUBESPACE-CUBEWHEEL-CW0017 x3 | — |
| mtq_cmg | benchmark | — | no | 2.016 | 7.76 | 0.806 | CAT-TENSOR-TECH-ADCS400 x4 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power); budget: mass_kg 2.02 > 1.6 |
| mtq_vscmg | benchmark | — | no | 2.016 | 7.76 | 0.806 | CAT-TENSOR-TECH-ADCS400-VSCMG x4 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power); budget: mass_kg 2.02 > 1.6 |
| mtq_cmg_rcs | benchmark | — | no | 2.482 | 7.81 | 1.648 | CAT-TENSOR-TECH-ADCS400 x4 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power); budget: mass_kg 2.48 > 1.6; budget: volume_L 1.65 > 1 |
| mtq_vscmg_rcs | benchmark | — | no | 2.482 | 7.81 | 1.648 | CAT-TENSOR-TECH-ADCS400-VSCMG x4 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power); budget: mass_kg 2.48 > 1.6; budget: volume_L 1.65 > 1 |

## Selected methods (mtq_fmr)

| mode | option | feasible | objective | algorithms | failing |
|---|---|---|---:|---|---|
| detumble | mtq | yes | 63.54 detumble_time | attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, sun_acquisition=sunspin_l1l2 | — |
| sun_acquisition | fmr | yes | 2.275 sun_acquisition_time | allocation=idmas_split, attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, pointing=pid, sun_acquisition=sunspin_l1l2 | — |
| sun_referencing | fmr+mtq | yes | 0.008868 sun_ape_p9973 | allocation=idmas_split, attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, pointing=pid, sun_acquisition=sunspin_l1l2 | — |
| nadir_pointing | fmr+mtq | yes | 0.003412 ape_los_p9973 | allocation=idmas_split, attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, pointing=pid, sun_acquisition=sunspin_l1l2 | — |

## Every solution family flown as the mission (node `family_missions`)

Detumble -> Sun acquisition -> nadir with each family's best methods from the loop, C flight software (C = Rust bit for bit), and the Monte Carlo pass rate of each requirement metric.

| family | selected | feasible | mass [kg] | methods | detumble_time | ape_los_p9973 | ake_los_p9973 | power_mean | C = Rust | MC pass rates |
|---|---|---|---:|---|---:|---:|---:|---:|---|---|
| mtq | no | no | 0.830 | detumble=mtq, sun_acquisition=mtq, sun_referencing=mtq, nadir_pointing=mtq | 65.04 ✓ | 176 ✗ | 1.015 ✗ | 0.09473 ✓ | True | detumble_time 100 %; ape_los_p9973 0 %; ake_los_p9973 0 %; power_mean 100 % |
| mtq_fmr | yes | yes | 1.534 | detumble=mtq, sun_acquisition=fmr, sun_referencing=fmr+mtq, nadir_pointing=fmr+mtq | 81.21 ✓ | 0.003543 ✓ | 0.002926 ✓ | 0.9228 ✓ | True | detumble_time 100 %; ape_los_p9973 100 %; ake_los_p9973 100 %; power_mean 100 % |
| mtq_fmr_rcs | no | no | 2.000 | detumble=rcs, sun_acquisition=fmr, sun_referencing=fmr+mtq, nadir_pointing=fmr+mtq | 2.042 ✓ | 0.003546 ✓ | 0.002925 ✓ | 1.014 ✓ | True | detumble_time 100 %; ape_los_p9973 100 %; ake_los_p9973 100 %; power_mean 100 % |

## Mode matrix (last iteration, best algorithm per option)

| mode | option | algorithm | feasible | objective | failing (cause) |
|---|---|---|---|---:|---|
| detumble | mtq | default | yes | 63.54 | — |
| detumble | rcs | default | yes | 1.108 | — |
| nadir_pointing | cmg+mtq | pid@bw4 | no | 0.003386 | power_mean (power) |
| nadir_pointing | cmg+rcs | pid@bw4 | no | 0.003385 | power_mean (power) |
| nadir_pointing | fmr+mtq | pid@bw4 | yes | 0.003412 | — |
| nadir_pointing | fmr+rcs | pid | yes | 0.005198 | — |
| nadir_pointing | mtq | mtq_rate_damp | no | 132.4 | ape_los_p9973 (performance), ake_los_p9973 (knowledge), rate_stability_p9973 (performance) |
| nadir_pointing | rw+mtq | pid@bw4 | yes | 0.003379 | — |
| nadir_pointing | rw+rcs | pid@bw4 | yes | 0.00339 | — |
| nadir_pointing | vscmg+mtq | pid@bw4 | no | 0.003379 | power_mean (power) |
| nadir_pointing | vscmg+rcs | pid@bw4 | no | 0.003377 | power_mean (power) |
| sun_acquisition | cmg | default | no | 2.275 | power_mean (power) |
| sun_acquisition | fmr | default | yes | 2.275 | — |
| sun_acquisition | mtq | sunspin_damped | no | 98.47 | sun_acquisition_time (performance), sun_angle_p95 (performance) |
| sun_acquisition | rw | default | yes | 2.275 | — |
| sun_acquisition | vscmg | default | no | 2.375 | power_mean (power) |
| sun_referencing | cmg+mtq | default | no | 0.008673 | power_mean (power) |
| sun_referencing | cmg+rcs | default | no | 0.008625 | power_mean (power) |
| sun_referencing | fmr+mtq | default | yes | 0.008868 | — |
| sun_referencing | fmr+rcs | default | no | 0.008901 | power_mean (power) |
| sun_referencing | mtq | mtq_rate_damp | no | 141.8 | sun_ape_p9973 (performance) |
| sun_referencing | rw+mtq | default | yes | 0.008679 | — |
| sun_referencing | rw+rcs | default | yes | 0.008676 | — |
| sun_referencing | vscmg+mtq | default | no | 0.008665 | power_mean (power) |
| sun_referencing | vscmg+rcs | default | no | 0.008652 | power_mean (power) |

## Monte Carlo of the dispatched mission (12 runs; dispersions: inertia, residual_dipole, cm_offset, solar_flux, accommodation, reflectivity, arg_lat_deg)

| metric | req | mean ± std | [min, max] | pass rate |
|---|---:|---|---|---:|
| detumble_time (min) | 284 | 71.62 ± 15.5 | [42.92, 91.16] | 100 % |
| ape_los_p9973 (deg) | 0.01 | 0.00338 ± 0.000338 | [0.002841, 0.00408] | 100 % |
| ake_los_p9973 (deg) | 0.005 | 0.002656 ± 0.000346 | [0.001955, 0.003335] | 100 % |
| power_mean (W) | 2 | 0.9999 ± 0.362 | [0.3904, 1.615] | 100 % |
| power_peak (W) | — | 2.811 ± 0.00757 | [2.799, 2.823] | — |
| propellant (g) | — | 0 ± 0 | [0, 0] | — |

## SILS and soft OILS of the dispatched mission

The same scenario, product and blob: in-process C (SILS) and the flight software as Cortex-M4F firmware in QEMU,
C and Rust builds, with exact per-step instruction counts and the command latency applied (docs/SOFT_OILS.md).

| metric | req | SILS | soft OILS (C on M4F) | soft OILS (Rust on M4F) |
|---|---:|---:|---:|---:|
| detumble_time | 284 | 81.21 ✓ | 80.22 ✓ | 80.22 ✓ |
| ape_los_p9973 | 0.01 | 0.003543 ✓ | 0.003612 ✓ | 0.003656 ✓ |
| ake_los_p9973 | 0.005 | 0.002926 ✓ | 0.002964 ✓ | 0.002962 ✓ |
| power_mean | 2 | 0.9228 ✓ | 0.9295 ✓ | 1.008 ✓ |
| power_peak | — | 2.81  | 2.805  | 2.805  |
| propellant | — | 0  | 0  | 0  |

| OBC build | instructions/step mean / max | exec max [ms] | latency mean / max [ms] | CPU load max | overruns |
|---|---:|---:|---:|---:|---:|
| C (arm-none-eabi-gcc -O2) | 2.158e+05 / 8.46e+05 | 6.295 | 2.774 / 7.464 | 6.3 % | 0 |
| Rust (thumbv7em-none-eabihf) | 2.899e+05 / 9.875e+05 | 7.348 | 3.326 / 8.517 | 7.3 % | 0 |

Dispatch: `dist/dispatch/ais_img_3u/mtq_fmr/converged` (blob, sized products, BUILD.md). C = Rust flight software bitwise on the engine: True.

