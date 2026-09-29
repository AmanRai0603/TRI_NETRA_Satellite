# Design loop: ais_img_3u

Owner: Agastya. `tools/pipeline.py ais_img_3u` — sizing (Rust, adcs-design) -> SILS mode matrix (Rust engine, POP in the loop,
C flight software) -> assess -> converge, repeated until nothing is left to change; then select, dispatch, Monte Carlo and
soft OILS of the selected configuration. Flow: `docs/figures/flow_design_to_hils.svg`, method: `docs/DESIGN_LOOP.md`.

**Selected: `mtq_fmr` — Magnetorquers + fluid momentum loop — feasible**, converged after 7 iteration(s). Knowledge class: fine. Sensors: star_tracker, magnetometer, sun_sensors, gyro, gnss, earth_sensor.

## Iterations

| iteration | knobs | options feasible | selected | changes | blocked |
|---:|---|---:|---|---|---|
| 1 | laws as written | 19/25 | mtq_fmr (closest (not feasible)) | nadir_pointing/fmr+mtq: fly every pointing algorithm<br>nadir_pointing/fmr+rcs: fly every pointing algorithm<br>nadir_pointing/mtq: fly every mtq_pointing algorithm<br>sun_acquisition/mtq: fly every sun_acquisition algorithm<br>sun_referencing/mtq: fly every mtq_pointing algorithm | 3 |
| 2 | laws as written | 19/25 | mtq_fmr (closest (not feasible)) | rate stability: gyro noise x1 -> x0.3 (fibre-optic class)<br>mtqp: authority x1 -> x1.5 (performance) | 4 |
| 3 | mtqp x1.5, gyro noise x0.3 | 19/25 | mtq_fmr (closest (not feasible)) | gyro grade back to x1: rate-stability violation 13.8 -> 13.8<br>mtqp: authority back to x1 (no improvement)<br>nadir_pointing/fmr+mtq: fluid-loop flow sensor 2 -> 0.5 mm/s (1 sigma) | 5 |
| 4 | mtqp x1, flow sensor 0.5 mm/s | 20/25 | mtq_fmr (closest (not feasible)) | mass (mtq_fmr): one star-tracker head instead of two | 4 |
| 5 | mtqp x1, 1 ST head, flow sensor 0.5 mm/s | 13/25 | mtq_fmr (closest (not feasible)) | nadir_pointing/cmg+mtq: fly every pointing algorithm<br>nadir_pointing/cmg+rcs: fly every pointing algorithm<br>nadir_pointing/fmr+mtq: fluid-loop flow sensor 0.5 -> 0.125 mm/s (1 sigma)<br>nadir_pointing/rw+mtq: fly every pointing algorithm<br>nadir_pointing/rw+rcs: fly every pointing algorithm<br>nadir_pointing/vscmg+mtq: fly every pointing algorithm<br>nadir_pointing/vscmg+rcs: fly every pointing algorithm<br>mass lever st_heads undone: it broke nadir_pointing/fmr+mtq, requirement violation 0 -> 41.1 | 11 |
| 6 | mtqp x1, flow sensor 0.125 mm/s | 22/25 | mtq_fmr (closest (not feasible)) | mass (mtq_fmr): lighter pump, less copper (lambda 0.1 -> 0.0333333 kg/W) | 2 |
| 7 | mtqp x1, pump lambda 0.0333333 kg/W, flow sensor 0.125 mm/s | 21/25 | mtq_fmr (feasible) | — | 3 |

Why the loop stopped (nothing left that a knob can change):

- mtqp: more authority did not reduce the performance violation (1.33e+04 -> 1.34e+04); kept at x1
- nadir_pointing/mtq: knowledge fails with the star tracker fitted
- sun_referencing/fmr+rcs: power is the thrusters' valve power (RCS dumping), not the pump

## Families (last iteration)

| family | role | feasible | mass [kg] | power [W] | volume [L] | gaps |
|---|---|---|---:|---:|---:|---|
| mtq | solution | no | 0.830 | 3.90 | 0.012 | sun_acquisition: sun_acquisition_time (performance), sun_angle_p95 (performance); sun_referencing: sun_ape_p9973 (performance); nadir_pointing: ape_los_p9973 (performance), ake_los_p9973 (knowledge), rate_stability_p9973 (performance) |
| mtq_fmr | solution | yes | 1.596 | 9.16 | 0.534 | — |
| mtq_fmr_rcs | solution | no | 2.062 | 9.21 | 1.376 | budget: mass_kg 2.06 > 1.6; budget: volume_L 1.38 > 1 |
| mtq_rw | benchmark | yes | 1.083 | 4.50 | 0.142 | — |
| mtq_rw_rcs | benchmark | yes | 1.548 | 4.55 | 0.983 | — |
| mtq_cmg | benchmark | yes | 1.164 | 4.68 | 0.272 | — |
| mtq_cmg_rcs | benchmark | no | 1.629 | 4.73 | 1.114 | budget: mass_kg 1.63 > 1.6; budget: volume_L 1.11 > 1 |
| mtq_vscmg | benchmark | yes | 1.198 | 5.08 | 0.272 | — |
| mtq_vscmg_rcs | benchmark | no | 1.664 | 5.13 | 1.114 | budget: mass_kg 1.66 > 1.6; budget: volume_L 1.11 > 1 |

## Selected methods (mtq_fmr)

| mode | option | feasible | objective | algorithms | failing |
|---|---|---|---:|---|---|
| detumble | mtq | yes | 63.54 detumble_time | attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, sun_acquisition=sunspin_l1l2 | — |
| sun_acquisition | fmr | yes | 2.275 sun_acquisition_time | allocation=idmas_split, attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, pointing=pid, sun_acquisition=sunspin_l1l2 | — |
| sun_referencing | fmr+mtq | yes | 0.008977 sun_ape_p9973 | allocation=idmas_split, attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, pointing=pid, sun_acquisition=sunspin_l1l2 | — |
| nadir_pointing | fmr+mtq | yes | 0.003535 ape_los_p9973 | allocation=idmas_split, attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, pointing=pid, sun_acquisition=sunspin_l1l2 | — |

## Mode matrix (last iteration, best algorithm per option)

| mode | option | algorithm | feasible | objective | failing (cause) |
|---|---|---|---|---:|---|
| detumble | mtq | default | yes | 63.54 | — |
| detumble | rcs | default | yes | 1.108 | — |
| nadir_pointing | cmg+mtq | pid@bw4 | yes | 0.003385 | — |
| nadir_pointing | cmg+rcs | pid@bw4 | yes | 0.003386 | — |
| nadir_pointing | fmr+mtq | pid@bw4 | yes | 0.003535 | — |
| nadir_pointing | fmr+rcs | smc | yes | 0.005546 | — |
| nadir_pointing | mtq | mtq_rate_damp | no | 132.4 | ape_los_p9973 (performance), ake_los_p9973 (knowledge), rate_stability_p9973 (performance) |
| nadir_pointing | rw+mtq | pid@bw2.5 | yes | 0.003915 | — |
| nadir_pointing | rw+rcs | pid@bw4 | yes | 0.003377 | — |
| nadir_pointing | vscmg+mtq | pid | yes | 0.007184 | — |
| nadir_pointing | vscmg+rcs | pid@bw2.5 | yes | 0.004091 | — |
| sun_acquisition | cmg | default | yes | 2.275 | — |
| sun_acquisition | fmr | default | yes | 2.275 | — |
| sun_acquisition | mtq | sunspin_damped | no | 98.47 | sun_acquisition_time (performance), sun_angle_p95 (performance) |
| sun_acquisition | rw | default | yes | 2.275 | — |
| sun_acquisition | vscmg | default | yes | 2.975 | — |
| sun_referencing | cmg+mtq | default | yes | 0.00867 | — |
| sun_referencing | cmg+rcs | default | yes | 0.008654 | — |
| sun_referencing | fmr+mtq | default | yes | 0.008977 | — |
| sun_referencing | fmr+rcs | default | no | 0.008949 | power_mean (power) |
| sun_referencing | mtq | mtq_rate_damp | no | 141.8 | sun_ape_p9973 (performance) |
| sun_referencing | rw+mtq | default | yes | 0.00865 | — |
| sun_referencing | rw+rcs | default | yes | 0.008624 | — |
| sun_referencing | vscmg+mtq | default | yes | 0.0085 | — |
| sun_referencing | vscmg+rcs | default | yes | 1.888 | — |

## Monte Carlo of the dispatched mission (12 runs; dispersions: inertia, residual_dipole, cm_offset, solar_flux, accommodation, reflectivity, arg_lat_deg)

| metric | req | mean ± std | [min, max] | pass rate |
|---|---:|---|---|---:|
| detumble_time (min) | 284 | 73.64 ± 15.5 | [44.49, 92.39] | 100 % |
| ape_los_p9973 (deg) | 0.01 | 0.003414 ± 0.000333 | [0.002923, 0.004088] | 100 % |
| ake_los_p9973 (deg) | 0.005 | 0.002662 ± 0.000348 | [0.001964, 0.003348] | 100 % |
| power_mean (W) | 2 | 1.497 ± 0.802 | [0.239, 2.864] | 75 % |
| power_peak (W) | — | 5.499 ± 0.00991 | [5.483, 5.514] | — |
| propellant (g) | — | 0 ± 0 | [0, 0] | — |

## SILS and soft OILS of the dispatched mission

The same scenario, product and blob: in-process C (SILS) and the flight software as Cortex-M4F firmware in QEMU,
C and Rust builds, with exact per-step instruction counts and the command latency applied (docs/SOFT_OILS.md).

| metric | req | SILS | soft OILS (C on M4F) | soft OILS (Rust on M4F) |
|---|---:|---:|---:|---:|
| detumble_time | 284 | 81.91 ✓ | 82.81 ✓ | 82.76 ✓ |
| ape_los_p9973 | 0.01 | 0.003645 ✓ | 0.003707 ✓ | 0.003734 ✓ |
| ake_los_p9973 | 0.005 | 0.002948 ✓ | 0.002961 ✓ | 0.002959 ✓ |
| power_mean | 2 | 1.049 ✓ | 0.9794 ✓ | 1.222 ✓ |
| power_peak | — | 5.489  | 5.499  | 5.49  |
| propellant | — | 0  | 0  | 0  |

| OBC build | instructions/step mean / max | exec max [ms] | latency mean / max [ms] | CPU load max | overruns |
|---|---:|---:|---:|---:|---:|
| C (arm-none-eabi-gcc -O2) | 2.154e+05 / 8.458e+05 | 6.293 | 2.771 / 7.462 | 6.3 % | 0 |
| Rust (thumbv7em-none-eabihf) | 2.89e+05 / 9.876e+05 | 7.349 | 3.319 / 8.518 | 7.3 % | 0 |

Dispatch: `dist/dispatch/ais_img_3u/mtq_fmr/converged` (blob, sized products, BUILD.md). C = Rust flight software bitwise on the engine: True.

