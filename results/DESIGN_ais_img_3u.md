# Design loop: ais_img_3u

Owner: Agastya. `tools/pipeline.py ais_img_3u` — sizing (Rust, adcs-design) -> SILS mode matrix (Rust engine, POP in the loop,
C flight software) -> assess -> converge, repeated until nothing is left to change; then select, dispatch, Monte Carlo and
soft OILS of the selected configuration. Flow: `docs/figures/flow_design_to_hils.svg`, method: `docs/DESIGN_LOOP.md`.

**Selected: `mtq_fmr` — Magnetorquers + fluid momentum loop — closest (not feasible)**, converged after 16 iteration(s). Knowledge class: fine. Sensors: star_tracker, magnetometer, sun_sensors, gyro, gnss, earth_sensor.

Open requirement gaps of the selected family (what the case must relax, or the next design lever):

- nadir_pointing: rate_stability_p9973 (performance)
- budget: mass_kg 1.21 > 1

## Iterations

| iteration | knobs | options feasible | selected | changes | blocked |
|---:|---|---:|---|---|---|
| 1 | laws as written | 13/25 | mtq_fmr (closest (not feasible)) | nadir_pointing/cmg+mtq: fly every pointing algorithm<br>nadir_pointing/cmg+rcs: fly every pointing algorithm<br>nadir_pointing/fmr+mtq: fly every pointing algorithm<br>nadir_pointing/fmr+rcs: fly every pointing algorithm<br>nadir_pointing/mtq: fly every mtq_pointing algorithm<br>nadir_pointing/rw+mtq: fly every pointing algorithm<br>nadir_pointing/rw+rcs: fly every pointing algorithm<br>nadir_pointing/vscmg+mtq: fly every pointing algorithm<br>nadir_pointing/vscmg+rcs: fly every pointing algorithm<br>sun_acquisition/mtq: fly every sun_acquisition algorithm<br>sun_referencing/mtq: fly every mtq_pointing algorithm | 3 |
| 2 | laws as written | 13/25 | mtq_fmr (closest (not feasible)) | rate stability: gyro noise x1 -> x0.3 (fibre-optic class)<br>gyro grade back to x1: rate-stability violation 290 -> 290<br>mtqp: authority x1 -> x1.5 (performance) | 4 |
| 3 | mtqp x1.5 | 13/25 | mtq_fmr (closest (not feasible)) | mtqp: authority back to x1 (no improvement)<br>nadir_pointing/fmr+mtq: fluid-loop flow sensor 2 -> 0.5 mm/s (1 sigma)<br>cmg: authority x1 -> x1.5 (performance)<br>rw: authority x1 -> x1.5 (performance)<br>vscmg: authority x1 -> x1.5 (performance) | 5 |
| 4 | mtqp x1, cmg x1.5, rw x1.5, vscmg x1.5, flow sensor 0.5 mm/s | 13/25 | mtq_fmr (closest (not feasible)) | cmg: authority back to x1 (no improvement)<br>rw: authority back to x1 (no improvement)<br>nadir_pointing/fmr+mtq: fluid-loop flow sensor 0.5 -> 0.2 mm/s (1 sigma)<br>fmr: authority x1 -> x1.5 (performance)<br>vscmg: authority x1.5 -> x2.25 (performance) | 5 |
| 5 | mtqp x1, cmg x1, rw x1, vscmg x2.25, fmr x1.5, flow sensor 0.2 mm/s | 14/25 | mtq_fmr (closest (not feasible)) | vscmg: authority back to x1.5 (no improvement)<br>fmr: authority x1.5 -> x2.25 (performance) | 5 |
| 6 | mtqp x1, cmg x1, rw x1, vscmg x1.5, fmr x2.25, flow sensor 0.2 mm/s | 13/25 | mtq_fmr (closest (not feasible)) | fmr: authority back to x1.5 (no improvement) | 7 |
| 7 | mtqp x1, cmg x1, rw x1, vscmg x1.5, fmr x1.5, flow sensor 0.2 mm/s | 14/25 | mtq_fmr (closest (not feasible)) | mass (mtq_fmr): one star-tracker head instead of two | 6 |
| 8 | mtqp x1, cmg x1, rw x1, vscmg x1.5, fmr x1.5, 1 ST head, flow sensor 0.2 mm/s | 14/25 | mtq_fmr (closest (not feasible)) | mass lever st_heads undone: it broke requirement violation 3.21 -> 46.6 | 14 |
| 9 | mtqp x1, cmg x1, rw x1, vscmg x1.5, fmr x1.5, flow sensor 0.2 mm/s | 14/25 | mtq_fmr (closest (not feasible)) | mass (mtq_fmr): lighter pump, less copper (lambda 0.1 -> 0.0333333 kg/W) | 6 |
| 10 | mtqp x1, cmg x1, rw x1, vscmg x1.5, fmr x1.5, pump lambda 0.0333333 kg/W, flow sensor 0.2 mm/s | 13/25 | mtq_fmr (closest (not feasible)) | mass (mtq_fmr): lighter pump, less copper (lambda 0.0333333 -> 0.0111111 kg/W) | 9 |
| 11 | mtqp x1, cmg x1, rw x1, vscmg x1.5, fmr x1.5, pump lambda 0.0111111 kg/W, flow sensor 0.2 mm/s | 12/25 | mtq (closest (not feasible)) | mass lever fmr_lambda undone: it broke sun_acquisition/fmr, requirement violation 3.18 -> 3.45 | 10 |
| 12 | mtqp x1, cmg x1, rw x1, vscmg x1.5, fmr x1.5, pump lambda 0.0333333 kg/W, flow sensor 0.2 mm/s | 13/25 | mtq_fmr (closest (not feasible)) | mass (mtq_fmr): fluid-loop momentum x1.5 -> x1.125 | 9 |
| 13 | mtqp x1, cmg x1, rw x1, vscmg x1.5, fmr x1.12, pump lambda 0.0333333 kg/W, flow sensor 0.2 mm/s | 13/25 | mtq_fmr (closest (not feasible)) | mass (mtq_fmr): fluid-loop momentum x1.125 -> x0.84375 | 7 |
| 14 | mtqp x1, cmg x1, rw x1, vscmg x1.5, fmr x0.844, pump lambda 0.0333333 kg/W, flow sensor 0.2 mm/s | 13/25 | mtq_fmr (closest (not feasible)) | mass (mtq_fmr): fluid-loop momentum x0.84375 -> x0.632812 | 9 |
| 15 | mtqp x1, cmg x1, rw x1, vscmg x1.5, fmr x0.633, pump lambda 0.0333333 kg/W, flow sensor 0.2 mm/s | 14/25 | mtq_fmr (closest (not feasible)) | mass (mtq_fmr): fluid-loop momentum x0.632812 -> x0.5 | 6 |
| 16 | mtqp x1, cmg x1, rw x1, vscmg x1.5, fmr x0.5, pump lambda 0.0333333 kg/W, flow sensor 0.2 mm/s | 13/25 | mtq_fmr (closest (not feasible)) | — | 8 |

Why the loop stopped (nothing left that a knob can change):

- cmg: more authority did not reduce the performance violation (10.2 -> 10.2); kept at x1
- fmr: more authority did not reduce the performance violation (9.08 -> 12.2); kept at x1.5
- mass (mtq_fmr): no lever left (nadir_pointing: rate_stability_p9973 (performance) budget: mass_kg 1.21 > 1)
- mtqp: more authority did not reduce the performance violation (1.35e+04 -> 1.35e+04); kept at x1
- nadir_pointing/mtq: knowledge fails with the star tracker fitted
- rw: more authority did not reduce the performance violation (7.15 -> 7.2); kept at x1
- sun_referencing/fmr+rcs: power is the thrusters' valve power (RCS dumping), not the pump
- vscmg: more authority did not reduce the performance violation (6.9 -> 7.11); kept at x1.5

## Families (last iteration)

| family | role | feasible | mass [kg] | power [W] | volume [L] | gaps |
|---|---|---|---:|---:|---:|---|
| mtq | solution | no | 0.830 | 3.90 | 0.012 | sun_acquisition: sun_acquisition_time (performance), sun_angle_p95 (performance); sun_referencing: sun_ape_p9973 (performance); nadir_pointing: ape_los_p9973 (performance), ake_los_p9973 (knowledge), rate_stability_p9973 (performance) |
| mtq_fmr | solution | no | 1.215 | 6.56 | 0.534 | nadir_pointing: rate_stability_p9973 (performance); budget: mass_kg 1.21 > 1 |
| mtq_fmr_rcs | solution | no | 1.681 | 6.61 | 1.376 | nadir_pointing: rate_stability_p9973 (performance); budget: mass_kg 1.68 > 1; budget: volume_L 1.38 > 0.6 |
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
| sun_referencing | fmr+mtq | yes | 0.00872 sun_ape_p9973 | allocation=idmas_split, attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, pointing=pid, sun_acquisition=sunspin_l1l2 | — |
| nadir_pointing | fmr+mtq | no | 0.00337 ape_los_p9973 | allocation=idmas_split, attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, pointing=pid, sun_acquisition=sunspin_l1l2 | rate_stability_p9973 |

## Mode matrix (last iteration, best algorithm per option)

| mode | option | algorithm | feasible | objective | failing (cause) |
|---|---|---|---|---:|---|
| detumble | mtq | default | yes | 63.54 | — |
| detumble | rcs | default | yes | 1.108 | — |
| nadir_pointing | cmg+mtq | lqr | no | 0.003221 | rate_stability_p9973 (performance) |
| nadir_pointing | cmg+rcs | lqr | no | 0.003227 | rate_stability_p9973 (performance) |
| nadir_pointing | fmr+mtq | pid@bw4 | no | 0.00337 | rate_stability_p9973 (performance) |
| nadir_pointing | fmr+rcs | pid | no | 0.005165 | rate_stability_p9973 (performance) |
| nadir_pointing | mtq | mtq_rate_damp | no | 132.4 | ape_los_p9973 (performance), ake_los_p9973 (knowledge), rate_stability_p9973 (performance) |
| nadir_pointing | rw+mtq | pid@bw2.5 | no | 0.003915 | rate_stability_p9973 (performance) |
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
| sun_referencing | fmr+mtq | default | yes | 0.00872 | — |
| sun_referencing | fmr+rcs | default | no | 0.008739 | power_mean (power) |
| sun_referencing | mtq | mtq_rate_damp | no | 141.8 | sun_ape_p9973 (performance) |
| sun_referencing | rw+mtq | default | yes | 0.00865 | — |
| sun_referencing | rw+rcs | default | yes | 0.008624 | — |
| sun_referencing | vscmg+mtq | default | yes | 0.008685 | — |
| sun_referencing | vscmg+rcs | default | yes | 0.008508 | — |

Dispatch: `dist/dispatch/ais_img_3u/mtq_fmr/converged` (blob, sized products, BUILD.md). C = Rust flight software bitwise on the engine: True.

