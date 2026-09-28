# Design loop: ais_img_3u

Owner: Agastya. `tools/pipeline.py ais_img_3u` — sizing (Rust, adcs-design) -> SILS mode matrix (Rust engine, POP in the loop,
C flight software) -> assess -> converge, repeated until nothing is left to change; then select, dispatch, Monte Carlo and
soft OILS of the selected configuration. Flow: `docs/figures/flow_design_to_hils.svg`, method: `docs/DESIGN_LOOP.md`.

**Selected: `mtq_fmr` — Magnetorquers + fluid momentum loop — closest (not feasible)**, converged after 5 iteration(s). Knowledge class: fine. Sensors: star_tracker, magnetometer, sun_sensors, gyro, gnss, earth_sensor.

Open requirement gaps of the selected family (what the case must relax, or the next design lever):

- nadir_pointing: ape_los_p9973 (performance), rate_stability_p9973 (performance)
- budget: mass_kg 1.56 > 1

## Iterations

| iteration | knobs | options feasible | selected | changes | blocked |
|---:|---|---:|---|---|---|
| 1 | laws as written | 13/25 | mtq_fmr (closest (not feasible)) | nadir_pointing/cmg+mtq: fly every pointing algorithm<br>nadir_pointing/cmg+rcs: fly every pointing algorithm<br>nadir_pointing/fmr+mtq: fly every pointing algorithm<br>nadir_pointing/fmr+rcs: fly every pointing algorithm<br>nadir_pointing/fmr+rcs: power -> permanent-magnet pump yoke on the fluid loop<br>nadir_pointing/mtq: fly every mtq_pointing algorithm<br>nadir_pointing/rw+mtq: fly every pointing algorithm<br>nadir_pointing/rw+rcs: fly every pointing algorithm<br>nadir_pointing/vscmg+mtq: fly every pointing algorithm<br>nadir_pointing/vscmg+rcs: fly every pointing algorithm<br>sun_acquisition/mtq: fly every sun_acquisition algorithm<br>sun_referencing/mtq: fly every mtq_pointing algorithm | 2 |
| 2 | fmr_pm | 14/25 | mtq_fmr (closest (not feasible)) | cmg: authority x1 -> x1.5 (performance)<br>fmr: authority x1 -> x1.5 (performance)<br>mtqp: authority x1 -> x1.5 (performance)<br>rw: authority x1 -> x1.5 (performance)<br>vscmg: authority x1 -> x1.5 (performance) | 1 |
| 3 | cmg x1.5, fmr x1.5, mtqp x1.5, rw x1.5, vscmg x1.5, fmr_pm | 14/25 | mtq_fmr (closest (not feasible)) | cmg: authority back to x1 (no improvement)<br>mtqp: authority back to x1 (no improvement)<br>rw: authority back to x1 (no improvement)<br>fmr: authority x1.5 -> x2.25 (performance)<br>vscmg: authority x1.5 -> x2.25 (performance) | 4 |
| 4 | cmg x1, fmr x2.25, mtqp x1, rw x1, vscmg x2.25, fmr_pm | 14/25 | mtq_fmr (closest (not feasible)) | fmr: authority back to x1.5 (no improvement)<br>vscmg: authority back to x1.5 (no improvement) | 6 |
| 5 | cmg x1, fmr x1.5, mtqp x1, rw x1, vscmg x1.5, fmr_pm | 14/25 | mtq_fmr (closest (not feasible)) | — | 6 |

Why the loop stopped (nothing left that a knob can change):

- cmg: more authority did not reduce the performance violation (10.2 -> 10.2); kept at x1
- fmr: more authority did not reduce the performance violation (32.1 -> 67.5); kept at x1.5
- mtqp: more authority did not reduce the performance violation (1.35e+04 -> 1.35e+04); kept at x1
- nadir_pointing/mtq: knowledge fails with the star tracker fitted
- rw: more authority did not reduce the performance violation (6.66 -> 6.72); kept at x1
- vscmg: more authority did not reduce the performance violation (6.9 -> 6.66); kept at x1.5

## Families (last iteration)

| family | role | feasible | mass [kg] | power [W] | volume [L] | gaps |
|---|---|---|---:|---:|---:|---|
| mtq | solution | no | 0.830 | 3.90 | 0.012 | sun_acquisition: sun_acquisition_time (performance), sun_angle_p95 (performance); sun_referencing: sun_ape_p9973 (performance); nadir_pointing: ape_los_p9973 (performance), ake_los_p9973 (knowledge), rate_stability_p9973 (performance) |
| mtq_fmr | solution | no | 1.559 | 3.85 | 0.474 | nadir_pointing: ape_los_p9973 (performance), rate_stability_p9973 (performance); budget: mass_kg 1.56 > 1 |
| mtq_fmr_rcs | solution | no | 2.025 | 3.90 | 1.316 | nadir_pointing: ape_los_p9973 (performance), rate_stability_p9973 (performance); budget: mass_kg 2.02 > 1; budget: volume_L 1.32 > 0.6 |
| mtq_rw | benchmark | no | 1.083 | 4.50 | 0.142 | nadir_pointing: rate_stability_p9973 (performance); budget: mass_kg 1.08 > 1 |
| mtq_rw_rcs | benchmark | no | 1.548 | 4.55 | 0.983 | nadir_pointing: rate_stability_p9973 (performance); budget: mass_kg 1.55 > 1; budget: volume_L 0.983 > 0.6 |
| mtq_cmg | benchmark | no | 1.164 | 4.68 | 0.272 | nadir_pointing: rate_stability_p9973 (performance); budget: mass_kg 1.16 > 1 |
| mtq_cmg_rcs | benchmark | no | 1.629 | 4.73 | 1.114 | nadir_pointing: rate_stability_p9973 (performance); budget: mass_kg 1.63 > 1; budget: volume_L 1.11 > 0.6 |
| mtq_vscmg | benchmark | no | 1.231 | 5.14 | 0.316 | nadir_pointing: rate_stability_p9973 (performance); budget: mass_kg 1.23 > 1 |
| mtq_vscmg_rcs | benchmark | no | 1.697 | 5.19 | 1.158 | nadir_pointing: rate_stability_p9973 (performance); budget: mass_kg 1.7 > 1; budget: volume_L 1.16 > 0.6 |

## Selected methods (mtq_fmr)

| mode | option | feasible | objective | algorithms | failing |
|---|---|---|---:|---|---|
| detumble | mtq | yes | 63.54 detumble_time | attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, sun_acquisition=sunspin_l1l2 | — |
| sun_acquisition | fmr | yes | 2.275 sun_acquisition_time | allocation=idmas_split, attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, pointing=pid, sun_acquisition=sunspin_l1l2 | — |
| sun_referencing | fmr+mtq | yes | 0.01472 sun_ape_p9973 | allocation=idmas_split, attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, pointing=pid, sun_acquisition=sunspin_l1l2 | — |
| nadir_pointing | fmr+mtq | no | 0.01211 ape_los_p9973 | allocation=idmas_split, attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, pointing=pid, sun_acquisition=sunspin_l1l2 | ape_los_p9973, rate_stability_p9973 |

## Mode matrix (last iteration, best algorithm per option)

| mode | option | algorithm | feasible | objective | failing (cause) |
|---|---|---|---|---:|---|
| detumble | mtq | default | yes | 63.54 | — |
| detumble | rcs | default | yes | 1.092 | — |
| nadir_pointing | cmg+mtq | lqr | no | 0.003221 | rate_stability_p9973 (performance) |
| nadir_pointing | cmg+rcs | lqr | no | 0.003227 | rate_stability_p9973 (performance) |
| nadir_pointing | fmr+mtq | pid | no | 0.01211 | ape_los_p9973 (performance), rate_stability_p9973 (performance) |
| nadir_pointing | fmr+rcs | pid | no | 0.01212 | ape_los_p9973 (performance), rate_stability_p9973 (performance) |
| nadir_pointing | mtq | mtq_rate_damp | no | 132.4 | ape_los_p9973 (performance), ake_los_p9973 (knowledge), rate_stability_p9973 (performance) |
| nadir_pointing | rw+mtq | pid | no | 0.004933 | rate_stability_p9973 (performance) |
| nadir_pointing | rw+rcs | lqr | no | 0.003205 | rate_stability_p9973 (performance) |
| nadir_pointing | vscmg+mtq | lqr | no | 0.003998 | rate_stability_p9973 (performance) |
| nadir_pointing | vscmg+rcs | pid | no | 0.005145 | rate_stability_p9973 (performance) |
| sun_acquisition | cmg | default | yes | 2.275 | — |
| sun_acquisition | fmr | default | yes | 2.275 | — |
| sun_acquisition | mtq | sunspin_damped | no | 98.47 | sun_acquisition_time (performance), sun_angle_p95 (performance) |
| sun_acquisition | rw | default | yes | 2.275 | — |
| sun_acquisition | vscmg | default | yes | 2.358 | — |
| sun_referencing | cmg+mtq | default | yes | 0.00867 | — |
| sun_referencing | cmg+rcs | default | yes | 0.008654 | — |
| sun_referencing | fmr+mtq | default | yes | 0.01472 | — |
| sun_referencing | fmr+rcs | default | yes | 0.01466 | — |
| sun_referencing | mtq | mtq_rate_damp | no | 141.8 | sun_ape_p9973 (performance) |
| sun_referencing | rw+mtq | default | yes | 0.00865 | — |
| sun_referencing | rw+rcs | default | yes | 0.008624 | — |
| sun_referencing | vscmg+mtq | default | yes | 0.008685 | — |
| sun_referencing | vscmg+rcs | default | yes | 0.008508 | — |

## Monte Carlo of the dispatched mission (12 runs; dispersions: inertia, residual_dipole, cm_offset, solar_flux, accommodation, reflectivity, arg_lat_deg)

| metric | req | mean ± std | [min, max] | pass rate |
|---|---:|---|---|---:|
| detumble_time (min) | 284 | 73.6 ± 15.5 | [44.66, 92.64] | 100 % |
| ape_los_p9973 (deg) | 0.01 | 0.01188 ± 0.000505 | [0.01126, 0.0128] | 0 % |
| ake_los_p9973 (deg) | 0.005 | 0.00281 ± 0.000332 | [0.002367, 0.003611] | 100 % |
| power_mean (W) | 2 | 0.04135 ± 0.00194 | [0.03819, 0.04514] | 100 % |
| power_peak (W) | — | 0.1211 ± 0.00332 | [0.1171, 0.1293] | — |
| propellant (g) | — | 0 ± 0 | [0, 0] | — |

## SILS and soft OILS of the dispatched mission

The same scenario, product and blob: in-process C (SILS) and the flight software as Cortex-M4F firmware in QEMU,
C and Rust builds, with exact per-step instruction counts and the command latency applied (docs/SOFT_OILS.md).

| metric | req | SILS | soft OILS (C on M4F) | soft OILS (Rust on M4F) |
|---|---:|---:|---:|---:|
| detumble_time | 284 | 82.17 ✓ | 82.76 ✓ | 82.74 ✓ |
| ape_los_p9973 | 0.01 | 0.01247 ✗ | 0.01249 ✗ | 0.01252 ✗ |
| ake_los_p9973 | 0.005 | 0.003124 ✓ | 0.003113 ✓ | 0.00312 ✓ |
| power_mean | 2 | 0.04263 ✓ | 0.04342 ✓ | 0.04343 ✓ |
| power_peak | — | 0.125  | 0.125  | 0.1255  |
| propellant | — | 0  | 0  | 0  |

| OBC build | instructions/step mean / max | exec max [ms] | latency mean / max [ms] | CPU load max | overruns |
|---|---:|---:|---:|---:|---:|
| C (arm-none-eabi-gcc -O2) | 2.152e+05 / 8.459e+05 | 6.294 | 2.770 / 7.463 | 6.3 % | 0 |
| Rust (thumbv7em-none-eabihf) | 2.89e+05 / 9.872e+05 | 7.345 | 3.319 / 8.514 | 7.3 % | 0 |

Dispatch: `dist/dispatch/ais_img_3u/mtq_fmr/converged` (blob, sized products, BUILD.md). C = Rust flight software bitwise on the engine: True.

