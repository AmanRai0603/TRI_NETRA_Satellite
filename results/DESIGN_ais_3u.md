# Design loop: ais_3u

Owner: Agastya. `tools/pipeline.py ais_3u` — sizing (Rust, adcs-design) -> SILS mode matrix (Rust engine, POP in the loop,
C flight software) -> assess -> converge, repeated until nothing is left to change; then select, dispatch, Monte Carlo and
soft OILS of the selected configuration. Flow: `docs/figures/flow_design_to_hils.svg`, method: `docs/DESIGN_LOOP.md`.

**Selected: `mtq_fmr` — Magnetorquers + fluid momentum loop — feasible**, converged after 7 iteration(s). Knowledge class: coarse. Sensors: magnetometer, sun_sensors, gyro, gnss, earth_sensor.

## Iterations

| iteration | knobs | options feasible | selected | changes | blocked |
|---:|---|---:|---|---|---|
| 1 | laws as written | 1/25 | mtq (closest (not feasible)) | nadir_pointing/fmr+mtq: power -> electromagnetic pump with more copper (lambda 0.1 -> 0.3 kg/W)<br>nadir_pointing/mtq: fly every mtq_pointing algorithm<br>sun_acquisition/mtq: fly every sun_acquisition algorithm<br>sun_referencing/mtq: fly every mtq_pointing algorithm | 16 |
| 2 | pump lambda 0.3 kg/W | 4/25 | mtq_fmr (feasible) | nadir_pointing/mtq: tune every mtq_pointing law's gains, min over the gains of the worst seed (Bruni & Celani 2017)<br>sun_acquisition/mtq: tune every sun_acquisition law's gains, min over the gains of the worst seed (Bruni & Celani 2017)<br>sun_referencing/mtq: tune every mtq_pointing law's gains, min over the gains of the worst seed (Bruni & Celani 2017) | 18 |
| 3 | pump lambda 0.3 kg/W | 5/25 | mtq_fmr (feasible) | mtqp: authority x1 -> x1.5 (performance) | 18 |
| 4 | mtqp x1.5, pump lambda 0.3 kg/W | 5/25 | mtq_fmr (feasible) | mtqp: authority x1.5 -> x2.25 (performance) | 18 |
| 5 | mtqp x2.25, pump lambda 0.3 kg/W | 5/25 | mtq_fmr (feasible) | mtqp: authority x2.25 -> x3.375 (performance) | 18 |
| 6 | mtqp x3.38, pump lambda 0.3 kg/W | 5/25 | mtq_fmr (feasible) | mtqp: authority back to x2.25 (no improvement) | 19 |
| 7 | mtqp x2.25, pump lambda 0.3 kg/W | 5/25 | mtq_fmr (feasible) | — | 19 |

Why the loop stopped (nothing left that a knob can change):

- detumble/rcs: power fails at the sized authority (rcs); the part's standby power is the floor
- mtqp: more authority did not reduce the performance violation (16.2 -> 29.7); kept at x2.25
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
| mtq | solution | — | no | 0.293 | 2.63 | 0.041 | coils only | sun_acquisition: sun_angle_p95 (performance); sun_referencing: sun_ape_p9973 (performance) |
| mtq_fmr_rcs | solution | — | no | 1.965 | 2.59 | 1.519 | fluid loop (3 rings) | budget: mass_kg 1.96 > 1.6; budget: volume_L 1.52 > 1 |
| mtq_rw | benchmark | — | no | 0.393 | 2.73 | 0.070 | CAT-CUBESPACE-CUBEWHEEL-CW0017 x3 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power) |
| mtq_rw_rcs | benchmark | — | no | 0.911 | 2.78 | 1.052 | CAT-CUBESPACE-CUBEWHEEL-CW0017 x3 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); budget: volume_L 1.05 > 1 |
| mtq_cmg | benchmark | — | no | 1.413 | 5.83 | 0.809 | CAT-TENSOR-TECH-ADCS400 x4 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power) |
| mtq_vscmg | benchmark | — | no | 1.413 | 5.83 | 0.809 | CAT-TENSOR-TECH-ADCS400-VSCMG x4 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power) |
| mtq_cmg_rcs | benchmark | — | no | 1.931 | 5.88 | 1.791 | CAT-TENSOR-TECH-ADCS400 x4 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); budget: mass_kg 1.93 > 1.6; budget: volume_L 1.79 > 1 |
| mtq_vscmg_rcs | benchmark | — | no | 1.931 | 5.88 | 1.791 | CAT-TENSOR-TECH-ADCS400-VSCMG x4 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); budget: mass_kg 1.93 > 1.6; budget: volume_L 1.79 > 1 |

## Selected methods (mtq_fmr)

| mode | option | feasible | objective | algorithms | failing |
|---|---|---|---:|---|---|
| detumble | mtq | yes | 39.24 detumble_time | attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, sun_acquisition=sunspin_l1l2 | — |
| sun_acquisition | fmr | yes | 2.292 sun_acquisition_time | allocation=idmas_split, attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, pointing=pid, sun_acquisition=sunspin_l1l2 | — |
| sun_referencing | fmr+mtq | yes | 0.2513 sun_ape_p9973 | allocation=idmas_split, attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, pointing=pid, sun_acquisition=sunspin_l1l2 | — |
| nadir_pointing | fmr+mtq | yes | 0.1514 ape_los_p9973 | allocation=idmas_split, attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, pointing=pid, sun_acquisition=sunspin_l1l2 | — |

## Every solution family flown as the mission (node `family_missions`)

Detumble -> Sun acquisition -> nadir with each family's best methods from the loop, C flight software (C = Rust bit for bit), and the Monte Carlo pass rate of each requirement metric.

| family | selected | feasible | mass [kg] | methods | detumble_time | ape_los_p9973 | ake_los_p9973 | power_mean | C = Rust | MC pass rates |
|---|---|---|---:|---|---:|---:|---:|---:|---|---|
| mtq | no | no | 0.293 | detumble=mtq, sun_acquisition=mtq, sun_referencing=mtq, nadir_pointing=mtq | 40.49 ✓ | 33.72 ✗ | 0.2464 ✓ | 0.1178 ✓ | True | detumble_time 100 %; ape_los_p9973 0 %; ake_los_p9973 100 %; power_mean 100 % |
| mtq_fmr | yes | yes | 1.446 | detumble=mtq, sun_acquisition=fmr, sun_referencing=fmr+mtq, nadir_pointing=fmr+mtq | 53.46 ✓ | 0.1516 ✓ | 0.15 ✓ | 0.233 ✓ | True | detumble_time 100 %; ape_los_p9973 100 %; ake_los_p9973 100 %; power_mean 100 % |
| mtq_fmr_rcs | no | no | 1.965 | detumble=mtq, sun_acquisition=fmr, sun_referencing=fmr+mtq, nadir_pointing=fmr+mtq | 53.46 ✓ | 0.1516 ✓ | 0.1499 ✓ | 0.2791 ✓ | True | detumble_time 100 %; ape_los_p9973 100 %; ake_los_p9973 100 %; power_mean 100 % |

## Coils only: every law of the literature, each at its best gains (nodes `matrix` + `tune`)

Each law's result is its worst seed at the gains that make that worst seed best (Bruni & Celani's min-max); the laws are listed by mode, best first. Sources: docs/MTQ_LITERATURE.md.

| mode | law | paper | best gains | feasible | objective (worst seed) | failing |
|---|---|---|---|---|---:|---|
| sun_acquisition | sunspin_damped | P11 -> P5, R_z floor | spin_rate_dps 4, ss_gain 0.3 | no | 93.09 sun_acquisition_time | sun_angle_p95 |
| sun_acquisition | sunspin_l1l2_e2 | P11 -> P5, eclipse E2 | spin_rate_dps 4, ss_gain 0.3 | no | 96.57 sun_acquisition_time | sun_acquisition_time |
| sun_acquisition | sunspin_l1l2 | P11 UPMSat-2 -> P5 He et al. 2023 | spin_rate_dps 2, ss_gain 0.3 | no | 56.16 sun_acquisition_time | sun_acquisition_time, sun_angle_p95 |
| sun_acquisition | sun_boresight_celani2026 | P8 Celani 2026 | nominal | no | 79.94 sun_acquisition_time | sun_angle_p95, sun_acquisition_time |
| sun_acquisition | sunspin_deruiter2011 | P11 -> P2 de Ruiter 2011 | spin_rate_dps 4, ss_gain 0.3 | no | 95.22 sun_acquisition_time | sun_acquisition_time, sun_angle_p95 |
| sun_referencing | mtq_celani2026 | P8 Celani 2026 | mtq_gain_p 0.25, mtq_gain_d 1 | no | 84.68 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_celani2015 | P4 Celani 2015 | mtq_gain_p 0.25, mtq_gain_d 1 | no | 95.78 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_avanzini2021 | P16 Avanzini et al. 2021 | mtq_gain_p 0.25, mtq_gain_d 1 | no | 95.78 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_lovera2004 | P1 Lovera & Astolfi 2004 | mtq_gain_p 0.25, mtq_gain_d 0.25 | no | 131 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_rate_damp | baseline rate damping | nominal | no | 156.7 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_pd | baseline PD | nominal | no | 159.2 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_tango2013 | P3 TANGO 2013 (flown) | mtq_gain_p 4, mtq_gain_d 4 | no | 162.7 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_lqr | baseline LQR | nominal | no | 172.8 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_smc | baseline SMC | nominal | no | 174.3 sun_ape_p9973 | sun_ape_p9973 |
| nadir_pointing | mtq_celani2026 | P8 Celani 2026 | mtq_gain_p 4, mtq_gain_d 0.25 | yes | 3.728 ape_los_p9973 | — |
| nadir_pointing | mtq_smc | baseline SMC | nominal | no | 11.85 ape_los_p9973 | ape_los_p9973 |
| nadir_pointing | mtq_tango2013 | P3 TANGO 2013 (flown) | mtq_gain_p 0.25, mtq_gain_d 1 | no | 12.91 ape_los_p9973 | ape_los_p9973 |
| nadir_pointing | mtq_lovera2004 | P1 Lovera & Astolfi 2004 | mtq_gain_p 0.25, mtq_gain_d 4 | no | 12.93 ape_los_p9973 | ape_los_p9973 |
| nadir_pointing | mtq_rate_damp | baseline rate damping | nominal | no | 13.17 ape_los_p9973 | ape_los_p9973 |
| nadir_pointing | mtq_pd | baseline PD | nominal | no | 15.2 ape_los_p9973 | ape_los_p9973 |
| nadir_pointing | mtq_celani2015 | P4 Celani 2015 | mtq_gain_p 0.25, mtq_gain_d 1 | no | 17.78 ape_los_p9973 | ape_los_p9973 |
| nadir_pointing | mtq_lqr | baseline LQR | nominal | no | 45.28 ape_los_p9973 | ape_los_p9973 |
| nadir_pointing | mtq_avanzini2021 | P16 Avanzini et al. 2021 | mtq_gain_p 4, mtq_gain_d 1 | no | 54.41 ape_los_p9973 | ape_los_p9973 |

## Coils-only nadir loop certificate (node `certify`, Floquet multipliers, Celani 2026's method)

The loop linearised about nadir with the gyroscopic and gravity-gradient terms, the coil duty and the field along one orbit; all |mu| < 1 certifies the periodic loop (a boresight law keeps one multiplier at 1 by design).

| law | gains | max abs(mu) | certified |
|---|---|---:|---|
| mtq_pd | nominal | 0.5992 | yes |
| mtq_rate_damp | nominal | 1.0000 | no |
| mtq_lovera2004 | nominal | 1.1508 | no |
| mtq_celani2015 | nominal | 0.7797 | yes |
| mtq_avanzini2021 | nominal | 2.5534 | no |
| mtq_celani2026 (dispatched) | {'mtq_gain_p': 4.0, 'mtq_gain_d': 0.25} | 0.0166 | yes |
| mtq_tango2013 | nominal | 0.0727 | yes |

## Mode matrix (last iteration, best algorithm per option)

| mode | option | algorithm | feasible | objective | failing (cause) |
|---|---|---|---|---:|---|
| detumble | mtq | default | yes | 39.24 | — |
| detumble | rcs | default | no | 1.875 | power_peak (power) |
| nadir_pointing | cmg+mtq | default | no | 0.1481 | power_mean (power) |
| nadir_pointing | cmg+rcs | default | no | 0.1481 | power_mean (power) |
| nadir_pointing | fmr+mtq | default | yes | 0.1514 | — |
| nadir_pointing | fmr+rcs | default | no | 0.1513 | power_mean (power) |
| nadir_pointing | mtq | mtq_celani2026@mtq_gain_p=4,mtq_gain_d=0.25 | yes | 3.728 | — |
| nadir_pointing | rw+mtq | default | no | 0.1482 | power_mean (power) |
| nadir_pointing | rw+rcs | default | no | 0.1481 | power_mean (power) |
| nadir_pointing | vscmg+mtq | default | no | 0.1481 | power_mean (power) |
| nadir_pointing | vscmg+rcs | default | no | 0.1481 | power_mean (power) |
| sun_acquisition | cmg | default | no | 2.275 | power_mean (power) |
| sun_acquisition | fmr | default | yes | 2.292 | — |
| sun_acquisition | mtq | sunspin_damped@spin_rate_dps=4,ss_gain=0.3 | no | 93.09 | sun_angle_p95 (performance) |
| sun_acquisition | rw | default | no | 2.292 | power_mean (power) |
| sun_acquisition | vscmg | default | no | 2.392 | power_mean (power) |
| sun_referencing | cmg+mtq | default | no | 0.2464 | power_mean (power) |
| sun_referencing | cmg+rcs | default | no | 0.2464 | power_mean (power) |
| sun_referencing | fmr+mtq | default | yes | 0.2513 | — |
| sun_referencing | fmr+rcs | default | no | 0.2512 | power_mean (power) |
| sun_referencing | mtq | mtq_celani2026@mtq_gain_p=0.25,mtq_gain_d=1 | no | 84.68 | sun_ape_p9973 (performance) |
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
| ape_los_p9973 | 10 | 0.1516 ✓ | 0.1217 ✓ | 0.1275 ✓ |
| ake_los_p9973 | 5 | 0.15 ✓ | 0.1184 ✓ | 0.1234 ✓ |
| power_mean | 0.5 | 0.233 ✓ | 0.2175 ✓ | 0.2189 ✓ |
| power_peak | — | 0.8385  | 0.85  | 0.8508  |
| propellant | — | 0  | 0  | 0  |

| OBC build | instructions/step mean / max | exec max [ms] | latency mean / max [ms] | CPU load max | overruns |
|---|---:|---:|---:|---:|---:|
| C (arm-none-eabi-gcc -O2) | 1.624e+05 / 8.249e+05 | 6.137 | 2.377 / 7.306 | 6.1 % | 0 |
| Rust (thumbv7em-none-eabihf) | 2.176e+05 / 9.61e+05 | 7.150 | 2.788 / 8.319 | 7.2 % | 0 |

Dispatch: `dist/dispatch/ais_3u/mtq_fmr/converged` (blob, sized products, BUILD.md). C = Rust flight software bitwise on the engine: True.

