# Design loop: ais_img_3u

Owner: Agastya. `tools/pipeline.py ais_img_3u` — sizing (Rust, adcs-design) -> SILS mode matrix (Rust engine, POP in the loop,
C flight software) -> assess -> converge, repeated until nothing is left to change; then select, dispatch, Monte Carlo and
soft OILS of the selected configuration. Flow: `docs/figures/flow_design_to_hils.svg`, method: `docs/DESIGN_LOOP.md`.

**Selected: `mtq_fmr` — Magnetorquers + fluid momentum loop — feasible**, converged after 7 iteration(s). Knowledge class: fine. Sensors: star_tracker, magnetometer, sun_sensors, gyro, gnss, earth_sensor.

## Iterations

| iteration | knobs | options feasible | selected | changes | blocked |
|---:|---|---:|---|---|---|
| 1 | laws as written | 9/25 | mtq_fmr (closest (not feasible)) | nadir_pointing/fmr+mtq: fly every pointing algorithm<br>nadir_pointing/fmr+rcs: fly every pointing algorithm<br>nadir_pointing/mtq: fly every mtq_pointing algorithm<br>sun_acquisition/mtq: fly every sun_acquisition algorithm<br>sun_referencing/mtq: fly every mtq_pointing algorithm | 15 |
| 2 | laws as written | 9/25 | mtq_fmr (closest (not feasible)) | rate stability: gyro noise x1 -> x0.3 (fibre-optic class)<br>nadir_pointing/mtq: tune every mtq_pointing law's gains, min over the gains of the worst seed (Bruni & Celani 2017)<br>sun_acquisition/mtq: tune every sun_acquisition law's gains, min over the gains of the worst seed (Bruni & Celani 2017)<br>sun_referencing/mtq: tune every mtq_pointing law's gains, min over the gains of the worst seed (Bruni & Celani 2017) | 14 |
| 3 | gyro noise x0.3 | 9/25 | mtq_fmr (closest (not feasible)) | rate stability: gyro noise x0.3 -> x0.1 (fibre-optic class)<br>mtqp: authority x1 -> x1.5 (performance) | 13 |
| 4 | mtqp x1.5, gyro noise x0.1 | 9/25 | mtq_fmr (closest (not feasible)) | gyro grade back to x0.3: rate-stability violation 10.6 -> 10.1<br>nadir_pointing/fmr+mtq: fluid-loop flow sensor 2 -> 0.5 mm/s (1 sigma)<br>mtqp: authority x1.5 -> x2.25 (performance) | 13 |
| 5 | mtqp x2.25, flow sensor 0.5 mm/s, gyro noise x0.3 | 13/25 | mtq_fmr (closest (not feasible)) | mtqp: authority back to x1.5 (no improvement) | 11 |
| 6 | mtqp x1.5, flow sensor 0.5 mm/s, gyro noise x0.3 | 12/25 | mtq_fmr (closest (not feasible)) | mass (mtq_fmr): one star-tracker head instead of two | 11 |
| 7 | mtqp x1.5, 1 ST head, flow sensor 0.5 mm/s, gyro noise x0.3 | 10/25 | mtq_fmr (feasible) | — | 13 |

Why the loop stopped (nothing left that a knob can change):

- mtqp: more authority did not reduce the performance violation (278 -> 302); kept at x1.5
- nadir_pointing/cmg+mtq: power fails at the sized authority (cmg); the part's standby power is the floor
- nadir_pointing/cmg+rcs: power fails at the sized authority (cmg); the part's standby power is the floor
- nadir_pointing/fmr+rcs: power is the thrusters' valve power (RCS dumping), not the pump
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
| mtq_fmr | solution | 1 | yes | 1.567 | 6.58 | 0.534 | fluid loop (3 rings) | — |
| mtq | solution | — | no | 0.684 | 3.74 | 0.018 | coils only | sun_acquisition: sun_acquisition_time (performance), sun_angle_p95 (performance); sun_referencing: sun_ape_p9973 (performance); nadir_pointing: ape_los_p9973 (performance), rate_stability_p9973 (performance) |
| mtq_fmr_rcs | solution | — | no | 2.033 | 6.63 | 1.375 | fluid loop (3 rings) | budget: mass_kg 2.03 > 1.6; budget: volume_L 1.37 > 1 |
| mtq_rw | benchmark | 1 | yes | 0.836 | 4.36 | 0.068 | CAT-CUBESPACE-CUBEWHEEL-CW0017 x3 | — |
| mtq_rw_rcs | benchmark | 2 | yes | 1.302 | 4.41 | 0.908 | CAT-CUBESPACE-CUBEWHEEL-CW0017 x3 | — |
| mtq_cmg | benchmark | — | no | 1.856 | 7.46 | 0.806 | CAT-TENSOR-TECH-ADCS400 x4 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power); budget: mass_kg 1.86 > 1.6 |
| mtq_vscmg | benchmark | — | no | 1.856 | 7.46 | 0.806 | CAT-TENSOR-TECH-ADCS400-VSCMG x4 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power); budget: mass_kg 1.86 > 1.6 |
| mtq_cmg_rcs | benchmark | — | no | 2.322 | 7.51 | 1.647 | CAT-TENSOR-TECH-ADCS400 x4 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power); budget: mass_kg 2.32 > 1.6; budget: volume_L 1.65 > 1 |
| mtq_vscmg_rcs | benchmark | — | no | 2.322 | 7.51 | 1.647 | CAT-TENSOR-TECH-ADCS400-VSCMG x4 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power); budget: mass_kg 2.32 > 1.6; budget: volume_L 1.65 > 1 |

## Selected methods (mtq_fmr)

| mode | option | feasible | objective | algorithms | failing |
|---|---|---|---:|---|---|
| detumble | mtq | yes | 59.69 detumble_time | attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, sun_acquisition=sunspin_l1l2 | — |
| sun_acquisition | fmr | yes | 2.292 sun_acquisition_time | allocation=idmas_split, attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, pointing=pid, sun_acquisition=sunspin_l1l2 | — |
| sun_referencing | fmr+mtq | yes | 0.1363 sun_ape_p9973 | allocation=idmas_split, attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, pointing=pid, sun_acquisition=sunspin_l1l2 | — |
| nadir_pointing | fmr+mtq | yes | 0.003161 ape_los_p9973 | allocation=idmas_split, attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, pointing=pid, sun_acquisition=sunspin_l1l2 | — |

## Every solution family flown as the mission (node `family_missions`)

Detumble -> Sun acquisition -> nadir with each family's best methods from the loop, C flight software (C = Rust bit for bit), and the Monte Carlo pass rate of each requirement metric.

| family | selected | feasible | mass [kg] | methods | detumble_time | ape_los_p9973 | ake_los_p9973 | power_mean | C = Rust | MC pass rates |
|---|---|---|---:|---|---:|---:|---:|---:|---|---|
| mtq | no | no | 0.684 | detumble=mtq, sun_acquisition=mtq, sun_referencing=mtq, nadir_pointing=mtq | 60.62 ✓ | 86.18 ✗ | 0.0091 ✗ | 0.07323 ✓ | True | detumble_time 100 %; ape_los_p9973 0 %; ake_los_p9973 42 %; power_mean 100 % |
| mtq_fmr | yes | yes | 1.567 | detumble=mtq, sun_acquisition=fmr, sun_referencing=fmr+mtq, nadir_pointing=fmr+mtq | 81.12 ✓ | 0.003013 ✓ | 0.001923 ✓ | 0.8799 ✓ | True | detumble_time 100 %; ape_los_p9973 100 %; ake_los_p9973 100 %; power_mean 100 % |
| mtq_fmr_rcs | no | no | 2.033 | detumble=rcs, sun_acquisition=fmr, sun_referencing=fmr+mtq, nadir_pointing=fmr+mtq | 1.075 ✓ | 0.003023 ✓ | 0.001925 ✓ | 1.178 ✓ | True | detumble_time 100 %; ape_los_p9973 100 %; ake_los_p9973 100 %; power_mean 100 % |

## Coils only: every law of the literature, each at its best gains (nodes `matrix` + `tune`)

Each law's result is its worst seed at the gains that make that worst seed best (Bruni & Celani's min-max); the laws are listed by mode, best first. Sources: docs/MTQ_LITERATURE.md.

| mode | law | paper | best gains | feasible | objective (worst seed) | failing |
|---|---|---|---|---|---:|---|
| sun_acquisition | sunspin_damped | P11 -> P5, R_z floor | spin_rate_dps 2, ss_gain 0.3 | no | 26.51 sun_acquisition_time | sun_acquisition_time, sun_angle_p95 |
| sun_acquisition | sunspin_l1l2 | P11 UPMSat-2 -> P5 He et al. 2023 | spin_rate_dps 2, ss_gain 0.3 | no | 26.51 sun_acquisition_time | sun_acquisition_time, sun_angle_p95 |
| sun_acquisition | sunspin_l1l2_e2 | P11 -> P5, eclipse E2 | spin_rate_dps 2, ss_gain 0.3 | no | 26.51 sun_acquisition_time | sun_acquisition_time, sun_angle_p95 |
| sun_acquisition | sunspin_deruiter2011 | P11 -> P2 de Ruiter 2011 | spin_rate_dps 2, ss_gain 0.3 | no | 26.51 sun_acquisition_time | sun_acquisition_time, sun_angle_p95 |
| sun_acquisition | sun_boresight_celani2026 | P8 Celani 2026 | nominal | no | 135.1 sun_acquisition_time | sun_angle_p95, sun_acquisition_time |
| sun_referencing | mtq_celani2026 | P8 Celani 2026 | mtq_gain_p 4, mtq_gain_d 0.25, handover_out_dps 0.25 | no | 18.84 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_tango2013 | P3 TANGO 2013 (flown) | mtq_gain_p 0.25, mtq_gain_d 4, handover_out_dps 0.25 | no | 57.42 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_celani2015 | P4 Celani 2015 | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 0.25 | no | 66.7 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_avanzini2021 | P16 Avanzini et al. 2021 | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 0.25 | no | 66.7 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_lovera2004 | P1 Lovera & Astolfi 2004 | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 0.25 | no | 137.7 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_rate_damp | baseline rate damping | nominal | no | 153.2 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_pd | baseline PD | nominal | no | 170.5 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_smc | baseline SMC | nominal | no | 172.9 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_lqr | baseline LQR | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 0.25 | no | 178 sun_ape_p9973 | sun_ape_p9973 |
| nadir_pointing | mtq_celani2026 | P8 Celani 2026 | mtq_gain_p 4, mtq_gain_d 4, handover_out_dps 0.25 | no | 83.47 ape_los_p9973 | ape_los_p9973, rate_stability_p9973 |
| nadir_pointing | mtq_celani2015 | P4 Celani 2015 | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 1 | no | 45.94 ape_los_p9973 | ape_los_p9973, ake_los_p9973, rate_stability_p9973 |
| nadir_pointing | mtq_lovera2004 | P1 Lovera & Astolfi 2004 | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 1 | no | 68.32 ape_los_p9973 | ape_los_p9973, ake_los_p9973, rate_stability_p9973 |
| nadir_pointing | mtq_tango2013 | P3 TANGO 2013 (flown) | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 0.5 | no | 81.86 ape_los_p9973 | ape_los_p9973, ake_los_p9973, rate_stability_p9973 |
| nadir_pointing | mtq_rate_damp | baseline rate damping | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 0.25 | no | 113.7 ape_los_p9973 | ape_los_p9973, ake_los_p9973, rate_stability_p9973 |
| nadir_pointing | mtq_pd | baseline PD | nominal | no | 149.1 ape_los_p9973 | ape_los_p9973, ake_los_p9973, rate_stability_p9973 |
| nadir_pointing | mtq_avanzini2021 | P16 Avanzini et al. 2021 | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 0.5 | no | 152.7 ape_los_p9973 | ape_los_p9973, ake_los_p9973, rate_stability_p9973 |
| nadir_pointing | mtq_smc | baseline SMC | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 0.25 | no | 172.5 ape_los_p9973 | ape_los_p9973, ake_los_p9973, rate_stability_p9973 |
| nadir_pointing | mtq_lqr | baseline LQR | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 0.25 | no | 177.1 ape_los_p9973 | ape_los_p9973, ake_los_p9973, rate_stability_p9973 |

## Coils-only nadir loop certificate (node `certify`, Floquet multipliers, Celani 2026's method)

The loop linearised about nadir with the gyroscopic and gravity-gradient terms, the coil duty and the field along one orbit; all |mu| < 1 certifies the periodic loop (a boresight law keeps one multiplier at 1 by design).

| law | gains | max abs(mu) | certified |
|---|---|---:|---|
| mtq_pd | nominal | 4.9033 | no |
| mtq_rate_damp | nominal | 1.9820 | no |
| mtq_lovera2004 | nominal | 4.8218 | no |
| mtq_celani2015 | nominal | 2.4125 | no |
| mtq_avanzini2021 | nominal | 7.0389 | no |
| mtq_celani2026 (dispatched) | {'mtq_gain_p': 4.0, 'mtq_gain_d': 4.0} | 1.3509 | no |
| mtq_tango2013 | nominal | 7.1476 | no |

## Mode matrix (last iteration, best algorithm per option)

| mode | option | algorithm | feasible | objective | failing (cause) |
|---|---|---|---|---:|---|
| detumble | mtq | default | yes | 59.69 | — |
| detumble | rcs | default | yes | 1.125 | — |
| nadir_pointing | cmg+mtq | default | no | 0.002571 | power_mean (power) |
| nadir_pointing | cmg+rcs | default | no | 0.002568 | power_mean (power) |
| nadir_pointing | fmr+mtq | pid@bw4 | yes | 0.003161 | — |
| nadir_pointing | fmr+rcs | pid@bw4 | no | 0.003151 | power_mean (power) |
| nadir_pointing | mtq | mtq_celani2026@mtq_gain_p=4,mtq_gain_d=4,handover_out_dps=0.25 | no | 83.47 | ape_los_p9973 (performance), rate_stability_p9973 (performance) |
| nadir_pointing | rw+mtq | default | yes | 0.00263 | — |
| nadir_pointing | rw+rcs | default | yes | 0.002633 | — |
| nadir_pointing | vscmg+mtq | default | no | 0.002753 | power_mean (power) |
| nadir_pointing | vscmg+rcs | default | no | 0.002767 | power_mean (power) |
| sun_acquisition | cmg | default | no | 2.275 | power_mean (power) |
| sun_acquisition | fmr | default | yes | 2.292 | — |
| sun_acquisition | mtq | sunspin_damped@spin_rate_dps=2,ss_gain=0.3 | no | 26.51 | sun_acquisition_time (performance), sun_angle_p95 (performance) |
| sun_acquisition | rw | default | yes | 2.258 | — |
| sun_acquisition | vscmg | default | no | 2.342 | power_mean (power) |
| sun_referencing | cmg+mtq | default | no | 0.1365 | power_mean (power) |
| sun_referencing | cmg+rcs | default | no | 0.1365 | power_mean (power) |
| sun_referencing | fmr+mtq | default | yes | 0.1363 | — |
| sun_referencing | fmr+rcs | default | no | 0.1363 | power_mean (power) |
| sun_referencing | mtq | mtq_celani2026@mtq_gain_p=4,mtq_gain_d=0.25,handover_out_dps=0.25 | no | 18.84 | sun_ape_p9973 (performance) |
| sun_referencing | rw+mtq | default | yes | 0.1367 | — |
| sun_referencing | rw+rcs | default | yes | 0.1367 | — |
| sun_referencing | vscmg+mtq | default | no | 0.1363 | power_mean (power) |
| sun_referencing | vscmg+rcs | default | no | 0.1363 | power_mean (power) |

## Monte Carlo of the dispatched mission (12 runs; dispersions: inertia, residual_dipole, cm_offset, solar_flux, accommodation, reflectivity, arg_lat_deg)

| metric | req | mean ± std | [min, max] | pass rate |
|---|---:|---|---|---:|
| detumble_time (min) | 284 | 72.59 ± 14.6 | [42.92, 90.17] | 100 % |
| ape_los_p9973 (deg) | 0.01 | 0.00317 ± 0.000307 | [0.002789, 0.003726] | 100 % |
| ake_los_p9973 (deg) | 0.005 | 0.002204 ± 0.000437 | [0.001718, 0.003363] | 100 % |
| power_mean (W) | 2 | 0.8316 ± 0.346 | [0.4045, 1.536] | 100 % |
| power_peak (W) | — | 3.224 ± 0.00921 | [3.215, 3.237] | — |
| propellant (g) | — | 0 ± 0 | [0, 0] | — |

## SILS and soft OILS of the dispatched mission

The same scenario, product and blob: in-process C (SILS) and the flight software as Cortex-M4F firmware in QEMU,
C and Rust builds, with exact per-step instruction counts and the command latency applied (docs/SOFT_OILS.md).

| metric | req | SILS | soft OILS (C on M4F) | soft OILS (Rust on M4F) |
|---|---:|---:|---:|---:|
| detumble_time | 284 | 81.12 ✓ | 81.09 ✓ | 81.09 ✓ |
| ape_los_p9973 | 0.01 | 0.003013 ✓ | 0.003318 ✓ | 0.003385 ✓ |
| ake_los_p9973 | 0.005 | 0.001923 ✓ | 0.001998 ✓ | 0.00202 ✓ |
| power_mean | 2 | 0.8799 ✓ | 0.7039 ✓ | 0.6328 ✓ |
| power_peak | — | 3.222  | 3.223  | 3.22  |
| propellant | — | 0  | 0  | 0  |

| OBC build | instructions/step mean / max | exec max [ms] | latency mean / max [ms] | CPU load max | overruns |
|---|---:|---:|---:|---:|---:|
| C (arm-none-eabi-gcc -O2) | 2.213e+05 / 7.693e+05 | 5.724 | 2.815 / 6.893 | 5.7 % | 0 |
| Rust (thumbv7em-none-eabihf) | 2.799e+05 / 8.777e+05 | 6.531 | 3.251 / 7.700 | 6.5 % | 0 |

Dispatch: `dist/dispatch/ais_img_3u/mtq_fmr/converged` (blob, sized products, BUILD.md). C = Rust flight software bitwise on the engine: True.

