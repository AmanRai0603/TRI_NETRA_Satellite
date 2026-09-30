# Design loop: ais_img_3u

Owner: Agastya. `tools/pipeline.py ais_img_3u` — sizing (Rust, adcs-design) -> SILS mode matrix (Rust engine, POP in the loop,
C flight software) -> assess -> converge, repeated until nothing is left to change; then select, dispatch, Monte Carlo and
soft OILS of the selected configuration. Flow: `docs/figures/flow_design_to_hils.svg`, method: `docs/DESIGN_LOOP.md`.

**Selected: `mtq_fmr` — Magnetorquers + fluid momentum loop — feasible**, converged after 6 iteration(s). Knowledge class: fine. Sensors: star_tracker, magnetometer, sun_sensors, gyro, gnss, earth_sensor.

## Iterations

| iteration | knobs | options feasible | selected | changes | blocked |
|---:|---|---:|---|---|---|
| 1 | laws as written | 9/25 | mtq_fmr (closest (not feasible)) | nadir_pointing/fmr+mtq: fly every pointing algorithm<br>nadir_pointing/fmr+rcs: fly every pointing algorithm<br>nadir_pointing/mtq: fly every mtq_pointing algorithm<br>sun_acquisition/mtq: fly every sun_acquisition algorithm<br>sun_referencing/mtq: fly every mtq_pointing algorithm | 15 |
| 2 | laws as written | 9/25 | mtq_fmr (closest (not feasible)) | rate stability: gyro noise x1 -> x0.3 (fibre-optic class)<br>nadir_pointing/mtq: tune every mtq_pointing law's gains, min over the gains of the worst seed (Bruni & Celani 2017)<br>sun_acquisition/mtq: tune every sun_acquisition law's gains, min over the gains of the worst seed (Bruni & Celani 2017)<br>sun_referencing/mtq: tune every mtq_pointing law's gains, min over the gains of the worst seed (Bruni & Celani 2017) | 14 |
| 3 | gyro noise x0.3 | 9/25 | mtq_fmr (closest (not feasible)) | gyro grade back to x1: rate-stability violation 13.6 -> 13.7<br>nadir_pointing/fmr+mtq: fluid-loop flow sensor 2 -> 0.5 mm/s (1 sigma)<br>mtqp: authority x1 -> x1.5 (performance) | 13 |
| 4 | mtqp x1.5, flow sensor 0.5 mm/s | 10/25 | mtq_fmr (closest (not feasible)) | mtqp: authority back to x1 (no improvement) | 13 |
| 5 | mtqp x1, flow sensor 0.5 mm/s | 10/25 | mtq_fmr (closest (not feasible)) | mass (mtq_fmr): one star-tracker head instead of two | 13 |
| 6 | mtqp x1, 1 ST head, flow sensor 0.5 mm/s | 10/25 | mtq_fmr (feasible) | — | 13 |

Why the loop stopped (nothing left that a knob can change):

- mtqp: more authority did not reduce the performance violation (2.6e+03 -> 2.85e+03); kept at x1
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
| mtq_fmr | solution | 1 | yes | 1.427 | 5.88 | 0.534 | fluid loop (3 rings) | — |
| mtq | solution | — | no | 0.530 | 2.90 | 0.012 | coils only | sun_acquisition: sun_acquisition_time (performance), sun_angle_p95 (performance); sun_referencing: sun_ape_p9973 (performance); nadir_pointing: ape_los_p9973 (performance), rate_stability_p9973 (performance) |
| mtq_fmr_rcs | solution | — | no | 1.893 | 5.93 | 1.375 | fluid loop (3 rings) | budget: mass_kg 1.89 > 1.6; budget: volume_L 1.37 > 1 |
| mtq_rw | benchmark | 1 | yes | 0.696 | 3.66 | 0.068 | CAT-CUBESPACE-CUBEWHEEL-CW0017 x3 | — |
| mtq_rw_rcs | benchmark | 2 | yes | 1.162 | 3.71 | 0.908 | CAT-CUBESPACE-CUBEWHEEL-CW0017 x3 | — |
| mtq_cmg | benchmark | — | no | 1.716 | 6.76 | 0.806 | CAT-TENSOR-TECH-ADCS400 x4 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power); budget: mass_kg 1.72 > 1.6 |
| mtq_vscmg | benchmark | — | no | 1.716 | 6.76 | 0.806 | CAT-TENSOR-TECH-ADCS400-VSCMG x4 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power); budget: mass_kg 1.72 > 1.6 |
| mtq_cmg_rcs | benchmark | — | no | 2.182 | 6.81 | 1.647 | CAT-TENSOR-TECH-ADCS400 x4 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power); budget: mass_kg 2.18 > 1.6; budget: volume_L 1.65 > 1 |
| mtq_vscmg_rcs | benchmark | — | no | 2.182 | 6.81 | 1.647 | CAT-TENSOR-TECH-ADCS400-VSCMG x4 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power); budget: mass_kg 2.18 > 1.6; budget: volume_L 1.65 > 1 |

## Selected methods (mtq_fmr)

| mode | option | feasible | objective | algorithms | failing |
|---|---|---|---:|---|---|
| detumble | mtq | yes | 58.91 detumble_time | attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, sun_acquisition=sunspin_l1l2 | — |
| sun_acquisition | fmr | yes | 2.292 sun_acquisition_time | allocation=idmas_split, attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, pointing=pid, sun_acquisition=sunspin_l1l2 | — |
| sun_referencing | fmr+mtq | yes | 0.1566 sun_ape_p9973 | allocation=idmas_split, attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, pointing=pid, sun_acquisition=sunspin_l1l2 | — |
| nadir_pointing | fmr+mtq | yes | 0.004483 ape_los_p9973 | allocation=idmas_split, attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, pointing=pid, sun_acquisition=sunspin_l1l2 | — |

## Every solution family flown as the mission (node `family_missions`)

Detumble -> Sun acquisition -> nadir with each family's best methods from the loop, C flight software (C = Rust bit for bit), and the Monte Carlo pass rate of each requirement metric.

| family | selected | feasible | mass [kg] | methods | detumble_time | ape_los_p9973 | ake_los_p9973 | power_mean | C = Rust | MC pass rates |
|---|---|---|---:|---|---:|---:|---:|---:|---|---|
| mtq | no | no | 0.530 | detumble=mtq, sun_acquisition=mtq, sun_referencing=mtq, nadir_pointing=mtq | 59.99 ✓ | 23.34 ✗ | 0.01675 ✗ | 0.0592 ✓ | True | detumble_time 100 %; ape_los_p9973 0 %; ake_los_p9973 42 %; power_mean 100 % |
| mtq_fmr | yes | yes | 1.427 | detumble=mtq, sun_acquisition=fmr, sun_referencing=fmr+mtq, nadir_pointing=fmr+mtq | 81.21 ✓ | 0.004249 ✓ | 0.003157 ✓ | 0.9336 ✓ | True | detumble_time 100 %; ape_los_p9973 100 %; ake_los_p9973 100 %; power_mean 100 % |
| mtq_fmr_rcs | no | no | 1.893 | detumble=rcs, sun_acquisition=fmr, sun_referencing=fmr+mtq, nadir_pointing=fmr+mtq | 1.075 ✓ | 0.004241 ✓ | 0.003154 ✓ | 1.219 ✓ | True | detumble_time 100 %; ape_los_p9973 100 %; ake_los_p9973 100 %; power_mean 100 % |

## Coils only: every law of the literature, each at its best gains (nodes `matrix` + `tune`)

Each law's result is its worst seed at the gains that make that worst seed best (Bruni & Celani's min-max); the laws are listed by mode, best first. Sources: docs/MTQ_LITERATURE.md.

| mode | law | paper | best gains | feasible | objective (worst seed) | failing |
|---|---|---|---|---|---:|---|
| sun_acquisition | sunspin_deruiter2011 | P11 -> P2 de Ruiter 2011 | spin_rate_dps 2, ss_gain 3 | no | 34.49 sun_acquisition_time | sun_acquisition_time, sun_angle_p95 |
| sun_acquisition | sunspin_l1l2 | P11 UPMSat-2 -> P5 He et al. 2023 | spin_rate_dps 4, ss_gain 3 | no | 66.71 sun_acquisition_time | sun_acquisition_time, sun_angle_p95 |
| sun_acquisition | sunspin_l1l2_e2 | P11 -> P5, eclipse E2 | spin_rate_dps 4, ss_gain 3 | no | 66.71 sun_acquisition_time | sun_acquisition_time, sun_angle_p95 |
| sun_acquisition | sunspin_damped | P11 -> P5, R_z floor | spin_rate_dps 2, ss_gain 1 | no | 78.51 sun_acquisition_time | sun_acquisition_time, sun_angle_p95 |
| sun_acquisition | sun_boresight_celani2026 | P8 Celani 2026 | nominal | no | 135.2 sun_acquisition_time | sun_angle_p95, sun_acquisition_time |
| sun_referencing | mtq_celani2026 | P8 Celani 2026 | mtq_gain_p 4, mtq_gain_d 0.25, handover_out_dps 0.25 | no | 19.26 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_tango2013 | P3 TANGO 2013 (flown) | mtq_gain_p 0.25, mtq_gain_d 4, handover_out_dps 0.25 | no | 58.1 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_celani2015 | P4 Celani 2015 | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 0.25 | no | 66.54 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_avanzini2021 | P16 Avanzini et al. 2021 | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 0.25 | no | 66.54 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_lovera2004 | P1 Lovera & Astolfi 2004 | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 0.25 | no | 136.8 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_rate_damp | baseline rate damping | nominal | no | 153 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_pd | baseline PD | nominal | no | 171 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_smc | baseline SMC | nominal | no | 173.2 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_lqr | baseline LQR | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 0.25 | no | 178 sun_ape_p9973 | sun_ape_p9973 |
| nadir_pointing | mtq_celani2026 | P8 Celani 2026 | mtq_gain_p 4, mtq_gain_d 0.25, handover_out_dps 0.5 | no | 26.78 ape_los_p9973 | ape_los_p9973, rate_stability_p9973 |
| nadir_pointing | mtq_celani2015 | P4 Celani 2015 | mtq_gain_p 1, mtq_gain_d 1, handover_out_dps 0.25 | no | 52.39 ape_los_p9973 | ape_los_p9973, rate_stability_p9973 |
| nadir_pointing | mtq_lovera2004 | P1 Lovera & Astolfi 2004 | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 0.5 | no | 76.26 ape_los_p9973 | ape_los_p9973, ake_los_p9973, rate_stability_p9973 |
| nadir_pointing | mtq_tango2013 | P3 TANGO 2013 (flown) | mtq_gain_p 1, mtq_gain_d 1, handover_out_dps 0.25 | no | 77.66 ape_los_p9973 | ape_los_p9973, ake_los_p9973, rate_stability_p9973 |
| nadir_pointing | mtq_rate_damp | baseline rate damping | nominal | no | 123.5 ape_los_p9973 | ape_los_p9973, ake_los_p9973, rate_stability_p9973 |
| nadir_pointing | mtq_avanzini2021 | P16 Avanzini et al. 2021 | nominal | no | 134.3 ape_los_p9973 | ape_los_p9973, ake_los_p9973, rate_stability_p9973 |
| nadir_pointing | mtq_pd | baseline PD | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 0.25 | no | 150.8 ape_los_p9973 | ape_los_p9973, ake_los_p9973, rate_stability_p9973 |
| nadir_pointing | mtq_lqr | baseline LQR | nominal | no | 166.8 ape_los_p9973 | ape_los_p9973, ake_los_p9973, rate_stability_p9973 |
| nadir_pointing | mtq_smc | baseline SMC | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 1 | no | 176.3 ape_los_p9973 | ape_los_p9973, ake_los_p9973, rate_stability_p9973 |

## Coils-only nadir loop certificate (node `certify`, Floquet multipliers, Celani 2026's method)

The loop linearised about nadir with the gyroscopic and gravity-gradient terms, the coil duty and the field along one orbit; all |mu| < 1 certifies the periodic loop (a boresight law keeps one multiplier at 1 by design).

| law | gains | max abs(mu) | certified |
|---|---|---:|---|
| mtq_pd | nominal | 4.9033 | no |
| mtq_rate_damp | nominal | 1.9820 | no |
| mtq_lovera2004 | nominal | 4.8218 | no |
| mtq_celani2015 | nominal | 2.4125 | no |
| mtq_avanzini2021 | nominal | 7.0389 | no |
| mtq_celani2026 (dispatched) | {'mtq_gain_p': 4.0, 'mtq_gain_d': 0.25} | 2.7519 | no |
| mtq_tango2013 | nominal | 7.1476 | no |

## Mode matrix (last iteration, best algorithm per option)

| mode | option | algorithm | feasible | objective | failing (cause) |
|---|---|---|---|---:|---|
| detumble | mtq | default | yes | 58.91 | — |
| detumble | rcs | default | yes | 1.125 | — |
| nadir_pointing | cmg+mtq | default | no | 0.005565 | power_mean (power) |
| nadir_pointing | cmg+rcs | default | no | 0.005563 | power_mean (power) |
| nadir_pointing | fmr+mtq | pid@bw4 | yes | 0.004483 | — |
| nadir_pointing | fmr+rcs | pid@bw4 | no | 0.004476 | power_mean (power) |
| nadir_pointing | mtq | mtq_celani2026@mtq_gain_p=4,mtq_gain_d=0.25,handover_out_dps=0.5 | no | 26.78 | ape_los_p9973 (performance), rate_stability_p9973 (performance) |
| nadir_pointing | rw+mtq | default | yes | 0.005696 | — |
| nadir_pointing | rw+rcs | default | yes | 0.005686 | — |
| nadir_pointing | vscmg+mtq | default | no | 0.005977 | power_mean (power) |
| nadir_pointing | vscmg+rcs | default | no | 0.006026 | power_mean (power) |
| sun_acquisition | cmg | default | no | 2.275 | power_mean (power) |
| sun_acquisition | fmr | default | yes | 2.292 | — |
| sun_acquisition | mtq | sunspin_deruiter2011@spin_rate_dps=2,ss_gain=3 | no | 34.49 | sun_acquisition_time (performance), sun_angle_p95 (performance) |
| sun_acquisition | rw | default | yes | 2.258 | — |
| sun_acquisition | vscmg | default | no | 2.342 | power_mean (power) |
| sun_referencing | cmg+mtq | default | no | 0.1569 | power_mean (power) |
| sun_referencing | cmg+rcs | default | no | 0.1569 | power_mean (power) |
| sun_referencing | fmr+mtq | default | yes | 0.1566 | — |
| sun_referencing | fmr+rcs | default | no | 0.1566 | power_mean (power) |
| sun_referencing | mtq | mtq_celani2026@mtq_gain_p=4,mtq_gain_d=0.25,handover_out_dps=0.25 | no | 19.26 | sun_ape_p9973 (performance) |
| sun_referencing | rw+mtq | default | yes | 0.1567 | — |
| sun_referencing | rw+rcs | default | yes | 0.1567 | — |
| sun_referencing | vscmg+mtq | default | no | 0.1571 | power_mean (power) |
| sun_referencing | vscmg+rcs | default | no | 0.157 | power_mean (power) |

## Monte Carlo of the dispatched mission (12 runs; dispersions: inertia, residual_dipole, cm_offset, solar_flux, accommodation, reflectivity, arg_lat_deg)

| metric | req | mean ± std | [min, max] | pass rate |
|---|---:|---|---|---:|
| detumble_time (min) | 284 | 72.62 ± 14.6 | [42.96, 90.17] | 100 % |
| ape_los_p9973 (deg) | 0.01 | 0.004304 ± 0.000387 | [0.003872, 0.005259] | 100 % |
| ake_los_p9973 (deg) | 0.005 | 0.003296 ± 0.000449 | [0.002766, 0.0045] | 100 % |
| power_mean (W) | 2 | 0.8959 ± 0.351 | [0.4424, 1.624] | 100 % |
| power_peak (W) | — | 3.224 ± 0.00874 | [3.214, 3.238] | — |
| propellant (g) | — | 0 ± 0 | [0, 0] | — |

## SILS and soft OILS of the dispatched mission

The same scenario, product and blob: in-process C (SILS) and the flight software as Cortex-M4F firmware in QEMU,
C and Rust builds, with exact per-step instruction counts and the command latency applied (docs/SOFT_OILS.md).

| metric | req | SILS | soft OILS (C on M4F) | soft OILS (Rust on M4F) |
|---|---:|---:|---:|---:|
| detumble_time | 284 | 81.21 ✓ | 81.12 ✓ | 81.12 ✓ |
| ape_los_p9973 | 0.01 | 0.004249 ✓ | 0.00452 ✓ | 0.004579 ✓ |
| ake_los_p9973 | 0.005 | 0.003157 ✓ | 0.003185 ✓ | 0.003208 ✓ |
| power_mean | 2 | 0.9336 ✓ | 0.8047 ✓ | 0.7481 ✓ |
| power_peak | — | 3.22  | 3.223  | 3.222  |
| propellant | — | 0  | 0  | 0  |

| OBC build | instructions/step mean / max | exec max [ms] | latency mean / max [ms] | CPU load max | overruns |
|---|---:|---:|---:|---:|---:|
| C (arm-none-eabi-gcc -O2) | 2.212e+05 / 7.7e+05 | 5.729 | 2.815 / 6.898 | 5.7 % | 0 |
| Rust (thumbv7em-none-eabihf) | 2.797e+05 / 8.782e+05 | 6.534 | 3.250 / 7.703 | 6.5 % | 0 |

Dispatch: `dist/dispatch/ais_img_3u/mtq_fmr/converged` (blob, sized products, BUILD.md). C = Rust flight software bitwise on the engine: True.

