# Design loop: ais_img_3u

Owner: Agastya. `tools/pipeline.py ais_img_3u` — sizing (Rust, adcs-design) -> SILS mode matrix (Rust engine, POP in the loop,
C flight software) -> assess -> converge, repeated until nothing is left to change; then select, dispatch, Monte Carlo and
soft OILS of the selected configuration. Flow: `docs/figures/flow_design_to_hils.svg`, method: `docs/DESIGN_LOOP.md`.

**Selected: `mtq_fmr` — Magnetorquers + fluid momentum loop — feasible**, converged after 10 iteration(s). Knowledge class: fine. Sensors: star_tracker, magnetometer, sun_sensors, gyro, gnss, earth_sensor.

## Iterations

| iteration | knobs | options feasible | selected | changes | blocked |
|---:|---|---:|---|---|---|
| 1 | laws as written | 9/25 | mtq_fmr (closest (not feasible)) | nadir_pointing/fmr+mtq: fly every pointing algorithm<br>nadir_pointing/fmr+rcs: fly every pointing algorithm<br>nadir_pointing/mtq: fly every mtq_pointing algorithm<br>sun_acquisition/mtq: fly every sun_acquisition algorithm<br>sun_referencing/mtq: fly every mtq_pointing algorithm | 13 |
| 2 | laws as written | 9/25 | mtq_fmr (closest (not feasible)) | rate stability: gyro noise x1 -> x0.3 (fibre-optic class)<br>nadir_pointing/mtq: tune every mtq_pointing law's gains, min over the gains of the worst seed (Bruni & Celani 2017)<br>sun_acquisition/mtq: tune every sun_acquisition law's gains, min over the gains of the worst seed (Bruni & Celani 2017)<br>sun_referencing/mtq: tune every mtq_pointing law's gains, min over the gains of the worst seed (Bruni & Celani 2017) | 13 |
| 3 | gyro noise x0.3 | 9/25 | mtq_fmr (closest (not feasible)) | gyro grade back to x1: rate-stability violation 8.96 -> 13.6<br>nadir_pointing/fmr+mtq: fluid-loop flow sensor 2 -> 0.5 mm/s (1 sigma)<br>mtqp: authority x1 -> x1.5 (performance) | 13 |
| 4 | mtqp x1.5, flow sensor 0.5 mm/s | 10/25 | mtq_fmr (closest (not feasible)) | mtqp: authority back to x1 (no improvement) | 13 |
| 5 | mtqp x1, flow sensor 0.5 mm/s | 10/25 | mtq_fmr (closest (not feasible)) | mass (mtq_fmr): one star-tracker head instead of two | 13 |
| 6 | mtqp x1, 1 ST head, flow sensor 0.5 mm/s | 7/25 | mtq_fmr (closest (not feasible)) | nadir_pointing/cmg+mtq: fly every pointing algorithm<br>nadir_pointing/cmg+rcs: fly every pointing algorithm<br>nadir_pointing/fmr+mtq: fluid-loop flow sensor 0.5 -> 0.125 mm/s (1 sigma)<br>nadir_pointing/rw+mtq: fly every pointing algorithm<br>nadir_pointing/rw+rcs: fly every pointing algorithm<br>nadir_pointing/vscmg+mtq: fly every pointing algorithm<br>nadir_pointing/vscmg+rcs: fly every pointing algorithm<br>mass lever st_heads undone: it broke nadir_pointing/fmr+mtq, requirement violation 0 -> 41.1 | 17 |
| 7 | mtqp x1, flow sensor 0.125 mm/s | 12/25 | mtq_fmr (closest (not feasible)) | mass (mtq_fmr): lighter pump, less copper (lambda 0.1 -> 0.0333333 kg/W) | 11 |
| 8 | mtqp x1, pump lambda 0.0333333 kg/W, flow sensor 0.125 mm/s | 11/25 | mtq_fmr (feasible) | — | 12 |
| 9 | mtqp x1, pump lambda 0.1 kg/W, flow sensor 0.125 mm/s | 12/25 | mtq_fmr (closest (not feasible)) | mass (mtq_fmr): fluid-loop momentum x1 -> x0.75 | 11 |
| 10 | mtqp x1, fmr x0.75, pump lambda 0.1 kg/W, flow sensor 0.125 mm/s | 11/25 | mtq_fmr (feasible) | — | 12 |

**Robustness (node `mc`) after iteration 8:** the Monte Carlo of `mtq_fmr` failed power_mean (75 % of runs pass). robustness (mtq_fmr): mean power fails in dispersed runs -> pump with more copper (lambda 0.0333333 -> 0.1 kg/W); lighter-pump lever closed

Why the loop stopped (nothing left that a knob can change):

- mtqp: more authority did not reduce the performance violation (370 -> 369); kept at x1
- nadir_pointing/cmg+mtq: power fails at the sized authority (cmg); the part's standby power is the floor
- nadir_pointing/cmg+rcs: power fails at the sized authority (cmg); the part's standby power is the floor
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
| mtq | solution | — | no | 0.830 | 3.90 | 0.012 | coils only | sun_acquisition: sun_acquisition_time (performance), sun_angle_p95 (performance); sun_referencing: sun_ape_p9973 (performance); nadir_pointing: ape_los_p9973 (performance), rate_stability_p9973 (performance) |
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
| mtq | no | no | 0.830 | detumble=mtq, sun_acquisition=mtq, sun_referencing=mtq, nadir_pointing=mtq | 65.04 ✓ | 11.64 ✗ | 0.002525 ✓ | 0.07492 ✓ | True | detumble_time 100 %; ape_los_p9973 0 %; ake_los_p9973 100 %; power_mean 100 % |
| mtq_fmr | yes | yes | 1.534 | detumble=mtq, sun_acquisition=fmr, sun_referencing=fmr+mtq, nadir_pointing=fmr+mtq | 81.21 ✓ | 0.003543 ✓ | 0.002926 ✓ | 0.9228 ✓ | True | detumble_time 100 %; ape_los_p9973 100 %; ake_los_p9973 100 %; power_mean 100 % |
| mtq_fmr_rcs | no | no | 2.000 | detumble=rcs, sun_acquisition=fmr, sun_referencing=fmr+mtq, nadir_pointing=fmr+mtq | 2.042 ✓ | 0.003546 ✓ | 0.002925 ✓ | 1.014 ✓ | True | detumble_time 100 %; ape_los_p9973 100 %; ake_los_p9973 100 %; power_mean 100 % |

## Coils only: every law of the literature, each at its best gains (nodes `matrix` + `tune`)

Each law's result is its worst seed at the gains that make that worst seed best (Bruni & Celani's min-max); the laws are listed by mode, best first. Sources: docs/MTQ_LITERATURE.md.

| mode | law | paper | best gains | feasible | objective (worst seed) | failing |
|---|---|---|---|---|---:|---|
| sun_acquisition | sunspin_damped | P11 -> P5, R_z floor | spin_rate_dps 2, ss_gain 3 | no | 72.54 sun_acquisition_time | sun_acquisition_time, sun_angle_p95 |
| sun_acquisition | sunspin_l1l2 | P11 UPMSat-2 -> P5 He et al. 2023 | spin_rate_dps 2, ss_gain 3 | no | 80.06 sun_acquisition_time | sun_angle_p95, sun_acquisition_time |
| sun_acquisition | sunspin_l1l2_e2 | P11 -> P5, eclipse E2 | spin_rate_dps 2, ss_gain 3 | no | 80.06 sun_acquisition_time | sun_angle_p95, sun_acquisition_time |
| sun_acquisition | sunspin_deruiter2011 | P11 -> P2 de Ruiter 2011 | spin_rate_dps 2, ss_gain 1 | no | 104.1 sun_acquisition_time | sun_acquisition_time, sun_angle_p95 |
| sun_acquisition | sun_boresight_celani2026 | P8 Celani 2026 | nominal | no | 104.2 sun_acquisition_time | sun_angle_p95, sun_acquisition_time |
| sun_referencing | mtq_celani2026 | P8 Celani 2026 | mtq_gain_p 4, mtq_gain_d 0.25 | no | 27.63 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_celani2015 | P4 Celani 2015 | mtq_gain_p 0.25, mtq_gain_d 4 | no | 44.91 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_avanzini2021 | P16 Avanzini et al. 2021 | mtq_gain_p 0.25, mtq_gain_d 4 | no | 44.91 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_tango2013 | P3 TANGO 2013 (flown) | mtq_gain_p 0.25, mtq_gain_d 4 | no | 62.19 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_lovera2004 | P1 Lovera & Astolfi 2004 | mtq_gain_p 0.25, mtq_gain_d 0.25 | no | 136.7 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_rate_damp | baseline rate damping | nominal | no | 141.8 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_pd | baseline PD | nominal | no | 162.5 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_smc | baseline SMC | nominal | no | 174.7 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_lqr | baseline LQR | nominal | no | 176.4 sun_ape_p9973 | sun_ape_p9973 |
| nadir_pointing | mtq_celani2026 | P8 Celani 2026 | mtq_gain_p 4, mtq_gain_d 0.25 | no | 3.405 ape_los_p9973 | ape_los_p9973, rate_stability_p9973 |
| nadir_pointing | mtq_tango2013 | P3 TANGO 2013 (flown) | mtq_gain_p 0.25, mtq_gain_d 4 | no | 90.19 ape_los_p9973 | ape_los_p9973, rate_stability_p9973 |
| nadir_pointing | mtq_celani2015 | P4 Celani 2015 | mtq_gain_p 4, mtq_gain_d 1 | no | 90.97 ape_los_p9973 | ape_los_p9973, rate_stability_p9973, ake_los_p9973 |
| nadir_pointing | mtq_lovera2004 | P1 Lovera & Astolfi 2004 | mtq_gain_p 0.25, mtq_gain_d 0.25 | no | 121.4 ape_los_p9973 | ape_los_p9973, ake_los_p9973, rate_stability_p9973 |
| nadir_pointing | mtq_avanzini2021 | P16 Avanzini et al. 2021 | mtq_gain_p 4, mtq_gain_d 1 | no | 125.8 ape_los_p9973 | ape_los_p9973, ake_los_p9973, rate_stability_p9973 |
| nadir_pointing | mtq_rate_damp | baseline rate damping | nominal | no | 132.4 ape_los_p9973 | ape_los_p9973, ake_los_p9973, rate_stability_p9973 |
| nadir_pointing | mtq_lqr | baseline LQR | nominal | no | 170.5 ape_los_p9973 | ape_los_p9973, ake_los_p9973, rate_stability_p9973 |
| nadir_pointing | mtq_pd | baseline PD | nominal | no | 172.8 ape_los_p9973 | ape_los_p9973, ake_los_p9973, rate_stability_p9973 |
| nadir_pointing | mtq_smc | baseline SMC | nominal | no | 178.8 ape_los_p9973 | ape_los_p9973, ake_los_p9973, rate_stability_p9973 |

## Coils-only nadir loop certificate (node `certify`, Floquet multipliers, Celani 2026's method)

The loop linearised about nadir with the gyroscopic and gravity-gradient terms, the coil duty and the field along one orbit; all |mu| < 1 certifies the periodic loop (a boresight law keeps one multiplier at 1 by design).

| law | gains | max abs(mu) | certified |
|---|---|---:|---|
| mtq_pd | nominal | 5.3258 | no |
| mtq_rate_damp | nominal | 3.8961 | no |
| mtq_lovera2004 | nominal | 5.1080 | no |
| mtq_celani2015 | nominal | 2.6356 | no |
| mtq_avanzini2021 | nominal | 647.8124 | no |
| mtq_celani2026 (dispatched) | {'mtq_gain_p': 4.0, 'mtq_gain_d': 0.25} | 0.0395 | yes |
| mtq_tango2013 | nominal | 7.4097 | no |

## Mode matrix (last iteration, best algorithm per option)

| mode | option | algorithm | feasible | objective | failing (cause) |
|---|---|---|---|---:|---|
| detumble | mtq | default | yes | 63.54 | — |
| detumble | rcs | default | yes | 1.108 | — |
| nadir_pointing | cmg+mtq | pid@bw4 | no | 0.003386 | power_mean (power) |
| nadir_pointing | cmg+rcs | pid@bw4 | no | 0.003385 | power_mean (power) |
| nadir_pointing | fmr+mtq | pid@bw4 | yes | 0.003412 | — |
| nadir_pointing | fmr+rcs | pid | yes | 0.005198 | — |
| nadir_pointing | mtq | mtq_celani2026@mtq_gain_p=4,mtq_gain_d=0.25 | no | 3.405 | ape_los_p9973 (performance), rate_stability_p9973 (performance) |
| nadir_pointing | rw+mtq | pid@bw4 | yes | 0.003379 | — |
| nadir_pointing | rw+rcs | pid@bw4 | yes | 0.00339 | — |
| nadir_pointing | vscmg+mtq | pid@bw4 | no | 0.003379 | power_mean (power) |
| nadir_pointing | vscmg+rcs | pid@bw4 | no | 0.003377 | power_mean (power) |
| sun_acquisition | cmg | default | no | 2.275 | power_mean (power) |
| sun_acquisition | fmr | default | yes | 2.275 | — |
| sun_acquisition | mtq | sunspin_damped@spin_rate_dps=2,ss_gain=3 | no | 72.54 | sun_acquisition_time (performance), sun_angle_p95 (performance) |
| sun_acquisition | rw | default | yes | 2.275 | — |
| sun_acquisition | vscmg | default | no | 2.375 | power_mean (power) |
| sun_referencing | cmg+mtq | default | no | 0.008673 | power_mean (power) |
| sun_referencing | cmg+rcs | default | no | 0.008625 | power_mean (power) |
| sun_referencing | fmr+mtq | default | yes | 0.008868 | — |
| sun_referencing | fmr+rcs | default | no | 0.008901 | power_mean (power) |
| sun_referencing | mtq | mtq_celani2026@mtq_gain_p=4,mtq_gain_d=0.25 | no | 27.63 | sun_ape_p9973 (performance) |
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
| ape_los_p9973 | 0.01 | 0.003543 ✓ | 0.003603 ✓ | 0.003655 ✓ |
| ake_los_p9973 | 0.005 | 0.002926 ✓ | 0.002962 ✓ | 0.002967 ✓ |
| power_mean | 2 | 0.9228 ✓ | 0.9291 ✓ | 1.008 ✓ |
| power_peak | — | 2.81  | 2.805  | 2.806  |
| propellant | — | 0  | 0  | 0  |

| OBC build | instructions/step mean / max | exec max [ms] | latency mean / max [ms] | CPU load max | overruns |
|---|---:|---:|---:|---:|---:|
| C (arm-none-eabi-gcc -O2) | 2.158e+05 / 8.464e+05 | 6.298 | 2.774 / 7.467 | 6.3 % | 0 |
| Rust (thumbv7em-none-eabihf) | 2.9e+05 / 9.876e+05 | 7.348 | 3.327 / 8.517 | 7.3 % | 0 |

Dispatch: `dist/dispatch/ais_img_3u/mtq_fmr/converged` (blob, sized products, BUILD.md). C = Rust flight software bitwise on the engine: True.

