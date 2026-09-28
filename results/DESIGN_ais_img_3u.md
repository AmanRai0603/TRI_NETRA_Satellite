# Design loop: ais_img_3u

Owner: Agastya. `tools/pipeline.py ais_img_3u` — sizing (Rust, adcs-design) -> SILS mode matrix (Rust engine, POP in the loop,
C flight software) -> assess -> converge, repeated until nothing is left to change; then select, dispatch, Monte Carlo and
soft OILS of the selected configuration. Flow: `docs/figures/flow_design_to_hils.svg`, method: `docs/DESIGN_LOOP.md`.

**Selected: `mtq_fmr` — Magnetorquers + fluid momentum loop — closest (not feasible)**, converged after 16 iteration(s). Knowledge class: fine. Sensors: star_tracker, magnetometer, sun_sensors, gyro, gnss, earth_sensor.

Open requirement gaps of the selected family (what the case must relax, or the next design lever):

- budget: mass_kg 1.75 > 1

## Iterations

| iteration | knobs | options feasible | selected | changes | blocked |
|---:|---|---:|---|---|---|
| 1 | laws as written | 13/25 | mtq_fmr (closest (not feasible)) | nadir_pointing/cmg+mtq: fly every pointing algorithm<br>nadir_pointing/cmg+rcs: fly every pointing algorithm<br>nadir_pointing/fmr+mtq: fly every pointing algorithm<br>nadir_pointing/fmr+rcs: fly every pointing algorithm<br>nadir_pointing/mtq: fly every mtq_pointing algorithm<br>nadir_pointing/rw+mtq: fly every pointing algorithm<br>nadir_pointing/rw+rcs: fly every pointing algorithm<br>nadir_pointing/vscmg+mtq: fly every pointing algorithm<br>nadir_pointing/vscmg+rcs: fly every pointing algorithm<br>sun_acquisition/mtq: fly every sun_acquisition algorithm<br>sun_referencing/mtq: fly every mtq_pointing algorithm | 3 |
| 2 | laws as written | 13/25 | mtq_fmr (closest (not feasible)) | rate stability: gyro noise x1 -> x0.3 (fibre-optic class)<br>mtqp: authority x1 -> x1.5 (performance) | 4 |
| 3 | mtqp x1.5, gyro noise x0.3 | 18/25 | mtq_fmr (closest (not feasible)) | mtqp: authority back to x1 (no improvement)<br>rate stability: gyro noise x0.3 -> x0.1 (fibre-optic class)<br>vscmg: authority x1 -> x1.5 (performance) | 5 |
| 4 | mtqp x1, vscmg x1.5, gyro noise x0.1 | 19/25 | mtq_fmr (closest (not feasible)) | nadir_pointing/fmr+mtq: fluid-loop flow sensor 2 -> 0.5 mm/s (1 sigma) | 5 |
| 5 | mtqp x1, vscmg x1.5, flow sensor 0.5 mm/s, gyro noise x0.1 | 20/25 | mtq_fmr (closest (not feasible)) | nadir_pointing/fmr+mtq: fluid-loop flow sensor 0.5 -> 0.125 mm/s (1 sigma) | 2 |
| 6 | mtqp x1, vscmg x1.5, flow sensor 0.125 mm/s, gyro noise x0.1 | 20/25 | mtq_fmr (closest (not feasible)) | nadir_pointing/fmr+mtq: fluid-loop flow sensor 0.125 -> 0.05 mm/s (1 sigma)<br>fmr: authority x1 -> x1.5 (performance) | 2 |
| 7 | mtqp x1, vscmg x1.5, fmr x1.5, flow sensor 0.05 mm/s, gyro noise x0.1 | 20/25 | mtq_fmr (closest (not feasible)) | mass (mtq_fmr): one star-tracker head instead of two<br>fmr: authority x1.5 -> x2.25 (performance) | 2 |
| 8 | mtqp x1, vscmg x1.5, fmr x2.25, 1 ST head, flow sensor 0.05 mm/s, gyro noise x0.1 | 13/25 | mtq_fmr (closest (not feasible)) | fmr: authority back to x1.5 (no improvement)<br>mass lever st_heads undone: it broke requirement violation 0.538 -> 2.83<br>cmg: authority x1 -> x1.5 (performance)<br>rw: authority x1 -> x1.5 (performance)<br>vscmg: authority x1.5 -> x2.25 (performance) | 12 |
| 9 | mtqp x1, vscmg x2.25, fmr x1.5, cmg x1.5, rw x1.5, flow sensor 0.05 mm/s, gyro noise x0.1 | 20/25 | mtq_fmr (closest (not feasible)) | mass (mtq_fmr): lighter pump, less copper (lambda 0.1 -> 0.0333333 kg/W) | 3 |
| 10 | mtqp x1, vscmg x2.25, fmr x1.5, cmg x1.5, rw x1.5, pump lambda 0.0333333 kg/W, flow sensor 0.05 mm/s, gyro noise x0.1 | 19/25 | mtq_fmr (closest (not feasible)) | mass (mtq_fmr): lighter pump, less copper (lambda 0.0333333 -> 0.0111111 kg/W) | 4 |
| 11 | mtqp x1, vscmg x2.25, fmr x1.5, cmg x1.5, rw x1.5, pump lambda 0.0111111 kg/W, flow sensor 0.05 mm/s, gyro noise x0.1 | 20/25 | mtq_fmr (closest (not feasible)) | mass lever fmr_lambda undone: it broke sun_acquisition/fmr | 4 |
| 12 | mtqp x1, vscmg x2.25, fmr x1.5, cmg x1.5, rw x1.5, pump lambda 0.0333333 kg/W, flow sensor 0.05 mm/s, gyro noise x0.1 | 19/25 | mtq_fmr (closest (not feasible)) | mass (mtq_fmr): fluid-loop momentum x1.5 -> x1.125 | 4 |
| 13 | mtqp x1, vscmg x2.25, fmr x1.12, cmg x1.5, rw x1.5, pump lambda 0.0333333 kg/W, flow sensor 0.05 mm/s, gyro noise x0.1 | 19/25 | mtq_fmr (closest (not feasible)) | mass (mtq_fmr): fluid-loop momentum x1.125 -> x0.84375 | 4 |
| 14 | mtqp x1, vscmg x2.25, fmr x0.844, cmg x1.5, rw x1.5, pump lambda 0.0333333 kg/W, flow sensor 0.05 mm/s, gyro noise x0.1 | 19/25 | mtq_fmr (closest (not feasible)) | mass (mtq_fmr): fluid-loop momentum x0.84375 -> x0.632812 | 4 |
| 15 | mtqp x1, vscmg x2.25, fmr x0.633, cmg x1.5, rw x1.5, pump lambda 0.0333333 kg/W, flow sensor 0.05 mm/s, gyro noise x0.1 | 21/25 | mtq_fmr (closest (not feasible)) | mass (mtq_fmr): fluid-loop momentum x0.632812 -> x0.5 | 3 |
| 16 | mtqp x1, vscmg x2.25, fmr x0.5, cmg x1.5, rw x1.5, pump lambda 0.0333333 kg/W, flow sensor 0.05 mm/s, gyro noise x0.1 | 21/25 | mtq_fmr (closest (not feasible)) | — | 4 |

Why the loop stopped (nothing left that a knob can change):

- mass (mtq_fmr): no lever left (budget: mass_kg 1.75 > 1)
- mtqp: more authority did not reduce the performance violation (1.35e+04 -> 1.36e+04); kept at x1
- nadir_pointing/mtq: knowledge fails with the star tracker fitted
- sun_referencing/fmr+rcs: power is the thrusters' valve power (RCS dumping), not the pump

## Families (last iteration)

| family | role | feasible | mass [kg] | power [W] | volume [L] | gaps |
|---|---|---|---:|---:|---:|---|
| mtq | solution | no | 1.370 | 6.60 | 0.012 | sun_acquisition: sun_acquisition_time (performance), sun_angle_p95 (performance); sun_referencing: sun_ape_p9973 (performance); nadir_pointing: ape_los_p9973 (performance), ake_los_p9973 (knowledge), rate_stability_p9973 (performance); budget: mass_kg 1.37 > 1 |
| mtq_fmr | solution | no | 1.755 | 9.26 | 0.534 | budget: mass_kg 1.75 > 1 |
| mtq_fmr_rcs | solution | no | 2.221 | 9.31 | 1.376 | budget: mass_kg 2.22 > 1; budget: volume_L 1.38 > 0.6 |
| mtq_rw | benchmark | no | 1.646 | 7.27 | 0.164 | budget: mass_kg 1.65 > 1 |
| mtq_rw_rcs | benchmark | no | 2.112 | 7.32 | 1.005 | budget: mass_kg 2.11 > 1; budget: volume_L 1.01 > 0.6 |
| mtq_cmg | benchmark | no | 1.733 | 7.44 | 0.316 | budget: mass_kg 1.73 > 1 |
| mtq_cmg_rcs | benchmark | no | 2.199 | 7.49 | 1.158 | budget: mass_kg 2.2 > 1; budget: volume_L 1.16 > 0.6 |
| mtq_vscmg | benchmark | no | 1.812 | 7.93 | 0.369 | budget: mass_kg 1.81 > 1 |
| mtq_vscmg_rcs | benchmark | no | 2.278 | 7.98 | 1.211 | budget: mass_kg 2.28 > 1; budget: volume_L 1.21 > 0.6 |

## Selected methods (mtq_fmr)

| mode | option | feasible | objective | algorithms | failing |
|---|---|---|---:|---|---|
| detumble | mtq | yes | 63.54 detumble_time | attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, sun_acquisition=sunspin_l1l2 | — |
| sun_acquisition | fmr | yes | 2.275 sun_acquisition_time | allocation=idmas_split, attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, pointing=pid, sun_acquisition=sunspin_l1l2 | — |
| sun_referencing | fmr+mtq | yes | 0.00474 sun_ape_p9973 | allocation=idmas_split, attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, pointing=pid, sun_acquisition=sunspin_l1l2 | — |
| nadir_pointing | fmr+mtq | yes | 0.0013 ape_los_p9973 | allocation=idmas_split, attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, pointing=smc, sun_acquisition=sunspin_l1l2 | — |

## Mode matrix (last iteration, best algorithm per option)

| mode | option | algorithm | feasible | objective | failing (cause) |
|---|---|---|---|---:|---|
| detumble | mtq | default | yes | 63.54 | — |
| detumble | rcs | default | yes | 1.108 | — |
| nadir_pointing | cmg+mtq | smc | yes | 0.001261 | — |
| nadir_pointing | cmg+rcs | smc | yes | 0.001254 | — |
| nadir_pointing | fmr+mtq | smc | yes | 0.0013 | — |
| nadir_pointing | fmr+rcs | smc | yes | 0.001286 | — |
| nadir_pointing | mtq | mtq_rate_damp | no | 133.4 | ape_los_p9973 (performance), ake_los_p9973 (knowledge), rate_stability_p9973 (performance) |
| nadir_pointing | rw+mtq | pid | yes | 0.004843 | — |
| nadir_pointing | rw+rcs | pid | yes | 0.001557 | — |
| nadir_pointing | vscmg+mtq | pid | yes | 0.001577 | — |
| nadir_pointing | vscmg+rcs | pid | yes | 0.001582 | — |
| sun_acquisition | cmg | default | yes | 2.275 | — |
| sun_acquisition | fmr | default | yes | 2.275 | — |
| sun_acquisition | mtq | sunspin_damped | no | 98.47 | sun_acquisition_time (performance), sun_angle_p95 (performance) |
| sun_acquisition | rw | default | yes | 2.275 | — |
| sun_acquisition | vscmg | default | yes | 2.408 | — |
| sun_referencing | cmg+mtq | default | yes | 0.004651 | — |
| sun_referencing | cmg+rcs | default | yes | 0.00465 | — |
| sun_referencing | fmr+mtq | default | yes | 0.00474 | — |
| sun_referencing | fmr+rcs | default | no | 0.004758 | power_mean (power) |
| sun_referencing | mtq | mtq_rate_damp | no | 141.8 | sun_ape_p9973 (performance) |
| sun_referencing | rw+mtq | default | yes | 0.004891 | — |
| sun_referencing | rw+rcs | default | yes | 0.00487 | — |
| sun_referencing | vscmg+mtq | default | yes | 0.004922 | — |
| sun_referencing | vscmg+rcs | default | yes | 0.004897 | — |

## Monte Carlo of the dispatched mission (12 runs; dispersions: inertia, residual_dipole, cm_offset, solar_flux, accommodation, reflectivity, arg_lat_deg)

| metric | req | mean ± std | [min, max] | pass rate |
|---|---:|---|---|---:|
| detumble_time (min) | 284 | 73.22 ± 15.6 | [44.46, 92.33] | 100 % |
| ape_los_p9973 (deg) | 0.01 | 0.007674 ± 0.00543 | [0.001366, 0.01746] | 83 % |
| ake_los_p9973 (deg) | 0.005 | 0.001205 ± 0.000383 | [0.0007565, 0.001958] | 100 % |
| power_mean (W) | 2 | 1.173 ± 0.41 | [0.2061, 1.669] | 100 % |
| power_peak (W) | — | 2.904 ± 0.0111 | [2.886, 2.918] | — |
| propellant (g) | — | 0 ± 0 | [0, 0] | — |

## SILS and soft OILS of the dispatched mission

The same scenario, product and blob: in-process C (SILS) and the flight software as Cortex-M4F firmware in QEMU,
C and Rust builds, with exact per-step instruction counts and the command latency applied (docs/SOFT_OILS.md).

| metric | req | SILS | soft OILS (C on M4F) | soft OILS (Rust on M4F) |
|---|---:|---:|---:|---:|
| detumble_time | 284 | 81.84 ✓ | 82.78 ✓ | 82.72 ✓ |
| ape_los_p9973 | 0.01 | 0.001184 ✓ | 0.001136 ✓ | 0.001113 ✓ |
| ake_los_p9973 | 0.005 | 0.001159 ✓ | 0.001152 ✓ | 0.001158 ✓ |
| power_mean | 2 | 1.349 ✓ | 1.388 ✓ | 1.456 ✓ |
| power_peak | — | 2.897  | 2.906  | 2.905  |
| propellant | — | 0  | 0  | 0  |

| OBC build | instructions/step mean / max | exec max [ms] | latency mean / max [ms] | CPU load max | overruns |
|---|---:|---:|---:|---:|---:|
| C (arm-none-eabi-gcc -O2) | 2.167e+05 / 8.475e+05 | 6.306 | 2.781 / 7.475 | 6.3 % | 0 |
| Rust (thumbv7em-none-eabihf) | 2.904e+05 / 9.879e+05 | 7.351 | 3.330 / 8.520 | 7.4 % | 0 |

Dispatch: `dist/dispatch/ais_img_3u/mtq_fmr/converged` (blob, sized products, BUILD.md). C = Rust flight software bitwise on the engine: True.

