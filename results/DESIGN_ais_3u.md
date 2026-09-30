# Design loop: ais_3u

Owner: Agastya. `tools/pipeline.py ais_3u` — sizing (Rust, adcs-design) -> SILS mode matrix (Rust engine, POP in the loop,
C flight software) -> assess -> converge, repeated until nothing is left to change; then select, dispatch, Monte Carlo and
soft OILS of the selected configuration. Flow: `docs/figures/flow_design_to_hils.svg`, method: `docs/DESIGN_LOOP.md`.

**Selected: `mtq_fmr` — Magnetorquers + fluid momentum loop — feasible**, converged after 6 iteration(s). Knowledge class: coarse. Sensors: magnetometer, sun_sensors, gyro, gnss, earth_sensor.

## Iterations

| iteration | knobs | options feasible | selected | changes | blocked |
|---:|---|---:|---|---|---|
| 1 | laws as written | 2/25 | mtq (closest (not feasible)) | nadir_pointing/fmr+mtq: power -> electromagnetic pump with more copper (lambda 0.1 -> 0.3 kg/W)<br>nadir_pointing/mtq: thin margin (9.04 of 10): fly every mtq_pointing algorithm<br>sun_acquisition/mtq: fly every sun_acquisition algorithm<br>sun_referencing/mtq: fly every mtq_pointing algorithm | 16 |
| 2 | pump lambda 0.3 kg/W | 5/25 | mtq_fmr (feasible) | nadir_pointing/mtq: thin margin (9.04 of 10): tune every mtq_pointing law's gains (Bruni & Celani 2017)<br>sun_acquisition/mtq: tune every sun_acquisition law's gains, min over the gains of the worst seed (Bruni & Celani 2017)<br>sun_referencing/mtq: tune every mtq_pointing law's gains, min over the gains of the worst seed (Bruni & Celani 2017) | 18 |
| 3 | pump lambda 0.3 kg/W | 5/25 | mtq_fmr (feasible) | mtqp: authority x1 -> x1.5 (performance) | 18 |
| 4 | mtqp x1.5, pump lambda 0.3 kg/W | 6/25 | mtq_fmr (feasible) | mtqp: authority x1.5 -> x2.25 (performance) | 18 |
| 5 | mtqp x2.25, pump lambda 0.3 kg/W | 5/25 | mtq_fmr (feasible) | mtqp: authority back to x1.5 (no improvement) | 19 |
| 6 | mtqp x1.5, pump lambda 0.3 kg/W | 6/25 | mtq_fmr (feasible) | — | 19 |

Why the loop stopped (nothing left that a knob can change):

- detumble/rcs: power fails at the sized authority (rcs); the part's standby power is the floor
- mtqp: more authority did not reduce the performance violation (16.2 -> 16.2); kept at x1.5
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
| mtq | solution | — | no | 0.258 | 2.28 | 0.027 | coils only | sun_referencing: sun_ape_p9973 (performance) |
| mtq_fmr_rcs | solution | — | no | 1.965 | 2.59 | 1.519 | fluid loop (3 rings) | budget: mass_kg 1.96 > 1.6; budget: volume_L 1.52 > 1 |
| mtq_rw | benchmark | — | no | 0.393 | 2.73 | 0.070 | CAT-CUBESPACE-CUBEWHEEL-CW0017 x3 | sun_referencing: power_mean (power) |
| mtq_rw_rcs | benchmark | — | no | 0.911 | 2.78 | 1.052 | CAT-CUBESPACE-CUBEWHEEL-CW0017 x3 | sun_referencing: power_mean (power); budget: volume_L 1.05 > 1 |
| mtq_cmg | benchmark | — | no | 1.413 | 5.83 | 0.809 | CAT-TENSOR-TECH-ADCS400 x4 | sun_referencing: power_mean (power) |
| mtq_vscmg | benchmark | — | no | 1.413 | 5.83 | 0.809 | CAT-TENSOR-TECH-ADCS400-VSCMG x4 | sun_referencing: power_mean (power) |
| mtq_cmg_rcs | benchmark | — | no | 1.931 | 5.88 | 1.791 | CAT-TENSOR-TECH-ADCS400 x4 | sun_referencing: power_mean (power); budget: mass_kg 1.93 > 1.6; budget: volume_L 1.79 > 1 |
| mtq_vscmg_rcs | benchmark | — | no | 1.931 | 5.88 | 1.791 | CAT-TENSOR-TECH-ADCS400-VSCMG x4 | sun_referencing: power_mean (power); budget: mass_kg 1.93 > 1.6; budget: volume_L 1.79 > 1 |

## Selected methods (mtq_fmr)

| mode | option | feasible | objective | algorithms | failing |
|---|---|---|---:|---|---|
| detumble | mtq | yes | 41.01 detumble_time | attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, sun_acquisition=sunspin_l1l2 | — |
| sun_acquisition | fmr | yes | 2.292 sun_acquisition_time | allocation=idmas_split, attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, pointing=pid, sun_acquisition=sunspin_l1l2 | — |
| sun_referencing | fmr+mtq | yes | 0.3797 sun_ape_p9973 | allocation=idmas_split, attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, pointing=pid, sun_acquisition=sunspin_l1l2 | — |
| nadir_pointing | fmr+mtq | yes | 0.1471 ape_los_p9973 | allocation=idmas_split, attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, pointing=pid, sun_acquisition=sunspin_l1l2 | — |

## Every solution family flown as the mission (node `family_missions`)

Detumble -> Sun acquisition -> nadir with each family's best methods from the loop, C flight software (C = Rust bit for bit), and the Monte Carlo pass rate of each requirement metric.

| family | selected | feasible | mass [kg] | methods | detumble_time | ape_los_p9973 | ake_los_p9973 | power_mean | C = Rust | MC pass rates |
|---|---|---|---:|---|---:|---:|---:|---:|---|---|
| mtq | no | no | 0.258 | detumble=mtq, sun_acquisition=mtq, sun_referencing=mtq, nadir_pointing=mtq | 40.46 ✓ | 6.626 ✓ | 0.1437 ✓ | 0.09055 ✓ | True | detumble_time 100 %; ape_los_p9973 0 %; ake_los_p9973 100 %; power_mean 100 % |
| mtq_fmr | yes | yes | 1.446 | detumble=mtq, sun_acquisition=fmr, sun_referencing=fmr+mtq, nadir_pointing=fmr+mtq | 53.46 ✓ | 0.1624 ✓ | 0.1605 ✓ | 0.2223 ✓ | True | detumble_time 100 %; ape_los_p9973 100 %; ake_los_p9973 100 %; power_mean 100 % |
| mtq_fmr_rcs | no | no | 1.965 | detumble=mtq, sun_acquisition=fmr, sun_referencing=fmr+mtq, nadir_pointing=fmr+mtq | 53.46 ✓ | 0.1624 ✓ | 0.1605 ✓ | 0.2514 ✓ | True | detumble_time 100 %; ape_los_p9973 100 %; ake_los_p9973 100 %; power_mean 100 % |

## Coils only: every law of the literature, each at its best gains (nodes `matrix` + `tune`)

Each law's result is its worst seed at the gains that make that worst seed best (Bruni & Celani's min-max); the laws are listed by mode, best first. Sources: docs/MTQ_LITERATURE.md.

| mode | law | paper | best gains | feasible | objective (worst seed) | failing |
|---|---|---|---|---|---:|---|
| sun_acquisition | sunspin_damped | P11 -> P5, R_z floor | spin_rate_dps 4, ss_gain 0.3 | yes | 82.32 sun_acquisition_time | — |
| sun_acquisition | sunspin_l1l2_e2 | P11 -> P5, eclipse E2 | spin_rate_dps 4, ss_gain 0.3 | no | 95.77 sun_acquisition_time | sun_acquisition_time |
| sun_acquisition | sunspin_deruiter2011 | P11 -> P2 de Ruiter 2011 | nominal | no | 30.19 sun_acquisition_time | sun_acquisition_time, sun_angle_p95 |
| sun_acquisition | sunspin_l1l2 | P11 UPMSat-2 -> P5 He et al. 2023 | spin_rate_dps 2, ss_gain 0.3 | no | 67.11 sun_acquisition_time | sun_acquisition_time, sun_angle_p95 |
| sun_acquisition | sun_boresight_celani2026 | P8 Celani 2026 | nominal | no | 79.72 sun_acquisition_time | sun_angle_p95, sun_acquisition_time |
| sun_referencing | mtq_celani2015 | P4 Celani 2015 | mtq_gain_p 0.25, mtq_gain_d 1, handover_out_dps 0.25 | no | 85.94 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_avanzini2021 | P16 Avanzini et al. 2021 | mtq_gain_p 0.25, mtq_gain_d 1, handover_out_dps 0.25 | no | 85.94 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_celani2026 | P8 Celani 2026 | mtq_gain_p 0.25, mtq_gain_d 1, handover_out_dps 0.25 | no | 86.38 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_tango2013 | P3 TANGO 2013 (flown) | mtq_gain_p 0.25, mtq_gain_d 1, handover_out_dps 0.25 | no | 135.4 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_lovera2004 | P1 Lovera & Astolfi 2004 | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 0.25 | no | 136.1 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_pd | baseline PD | nominal | no | 151.2 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_rate_damp | baseline rate damping | nominal | no | 162.8 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_lqr | baseline LQR | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 0.25 | no | 173.6 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_smc | baseline SMC | nominal | no | 179 sun_ape_p9973 | sun_ape_p9973 |
| nadir_pointing | mtq_celani2026 | P8 Celani 2026 | mtq_gain_p 4, mtq_gain_d 0.25, handover_out_dps 0.5 | yes | 6.278 ape_los_p9973 | — |
| nadir_pointing | mtq_pd | baseline PD | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 1 | yes | 7.989 ape_los_p9973 | — |
| nadir_pointing | mtq_smc | baseline SMC | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 0.25 | no | 12 ape_los_p9973 | ape_los_p9973 |
| nadir_pointing | mtq_tango2013 | P3 TANGO 2013 (flown) | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 1 | no | 23.86 ape_los_p9973 | ape_los_p9973 |
| nadir_pointing | mtq_celani2015 | P4 Celani 2015 | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 1 | no | 27.16 ape_los_p9973 | ape_los_p9973 |
| nadir_pointing | mtq_lovera2004 | P1 Lovera & Astolfi 2004 | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 0.25 | no | 34.16 ape_los_p9973 | ape_los_p9973 |
| nadir_pointing | mtq_lqr | baseline LQR | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 0.25 | no | 49.52 ape_los_p9973 | ape_los_p9973 |
| nadir_pointing | mtq_avanzini2021 | P16 Avanzini et al. 2021 | mtq_gain_p 4, mtq_gain_d 1, handover_out_dps 0.5 | no | 68.8 ape_los_p9973 | ape_los_p9973 |
| nadir_pointing | mtq_rate_damp | baseline rate damping | nominal | no | 175 ape_los_p9973 | ape_los_p9973 |

## Coils-only nadir loop certificate (node `certify`, Floquet multipliers, Celani 2026's method)

The loop linearised about nadir with the gyroscopic and gravity-gradient terms, the coil duty and the field along one orbit; all |mu| < 1 certifies the periodic loop (a boresight law keeps one multiplier at 1 by design).

| law | gains | max abs(mu) | certified |
|---|---|---:|---|
| mtq_pd | nominal | 0.5435 | yes |
| mtq_rate_damp | nominal | 1.0000 | yes |
| mtq_lovera2004 | nominal | 1.1470 | no |
| mtq_celani2015 | nominal | 0.7831 | yes |
| mtq_avanzini2021 | nominal | 2.5192 | no |
| mtq_celani2026 (dispatched) | {'mtq_gain_p': 4.0, 'mtq_gain_d': 0.25} | 0.9128 | yes |
| mtq_tango2013 | nominal | 0.0702 | yes |

## Mode matrix (last iteration, best algorithm per option)

| mode | option | algorithm | feasible | objective | failing (cause) |
|---|---|---|---|---:|---|
| detumble | mtq | default | yes | 41.01 | — |
| detumble | rcs | default | no | 1.342 | power_peak (power) |
| nadir_pointing | cmg+mtq | default | no | 0.1465 | power_mean (power) |
| nadir_pointing | cmg+rcs | default | no | 0.1464 | power_mean (power) |
| nadir_pointing | fmr+mtq | default | yes | 0.1471 | — |
| nadir_pointing | fmr+rcs | default | no | 0.1471 | power_mean (power) |
| nadir_pointing | mtq | mtq_celani2026@mtq_gain_p=4,mtq_gain_d=0.25,handover_out_dps=0.5 | yes | 6.278 | — |
| nadir_pointing | rw+mtq | default | no | 0.1465 | power_mean (power) |
| nadir_pointing | rw+rcs | default | no | 0.1465 | power_mean (power) |
| nadir_pointing | vscmg+mtq | default | no | 0.1464 | power_mean (power) |
| nadir_pointing | vscmg+rcs | default | no | 0.1464 | power_mean (power) |
| sun_acquisition | cmg | default | no | 2.275 | power_mean (power) |
| sun_acquisition | fmr | default | yes | 2.292 | — |
| sun_acquisition | mtq | sunspin_damped@spin_rate_dps=4,ss_gain=0.3 | yes | 82.32 | — |
| sun_acquisition | rw | default | no | 2.275 | power_mean (power) |
| sun_acquisition | vscmg | default | no | 2.392 | power_mean (power) |
| sun_referencing | cmg+mtq | default | no | 0.3653 | power_mean (power) |
| sun_referencing | cmg+rcs | default | no | 0.3654 | power_mean (power) |
| sun_referencing | fmr+mtq | default | yes | 0.3797 | — |
| sun_referencing | fmr+rcs | default | no | 0.3798 | power_mean (power) |
| sun_referencing | mtq | mtq_celani2015@mtq_gain_p=0.25,mtq_gain_d=1,handover_out_dps=0.25 | no | 85.94 | sun_ape_p9973 (performance) |
| sun_referencing | rw+mtq | default | no | 0.3658 | power_mean (power) |
| sun_referencing | rw+rcs | default | no | 0.3658 | power_mean (power) |
| sun_referencing | vscmg+mtq | default | no | 0.3655 | power_mean (power) |
| sun_referencing | vscmg+rcs | default | no | 0.3656 | power_mean (power) |

## Monte Carlo of the dispatched mission (12 runs; dispersions: inertia, residual_dipole, cm_offset, solar_flux, kp, accommodation, reflectivity, arg_lat_deg)

| metric | req | mean ± std | [min, max] | pass rate |
|---|---:|---|---|---:|
| detumble_time (min) | 284 | 56.87 ± 22.8 | [7.642, 92.08] | 100 % |
| ape_los_p9973 (deg) | 10 | 0.257 ± 0.0696 | [0.1434, 0.3766] | 100 % |
| ake_los_p9973 (deg) | 5 | 0.2535 ± 0.0691 | [0.14, 0.3726] | 100 % |
| power_mean (W) | 0.5 | 0.3364 ± 0.0943 | [0.2279, 0.4991] | 100 % |
| power_peak (W) | — | 0.8564 ± 0.0112 | [0.8365, 0.8717] | — |
| propellant (g) | — | 0 ± 0 | [0, 0] | — |

## SILS and soft OILS of the dispatched mission

The same scenario, product and blob: in-process C (SILS) and the flight software as Cortex-M4F firmware in QEMU,
C and Rust builds, with exact per-step instruction counts and the command latency applied (docs/SOFT_OILS.md).

| metric | req | SILS | soft OILS (C on M4F) | soft OILS (Rust on M4F) |
|---|---:|---:|---:|---:|
| detumble_time | 284 | 53.46 ✓ | 53.46 ✓ | 53.46 ✓ |
| ape_los_p9973 | 10 | 0.1624 ✓ | 0.1467 ✓ | 0.1402 ✓ |
| ake_los_p9973 | 5 | 0.1605 ✓ | 0.1468 ✓ | 0.1413 ✓ |
| power_mean | 0.5 | 0.2223 ✓ | 0.2048 ✓ | 0.2027 ✓ |
| power_peak | — | 0.858  | 0.8615  | 0.8622  |
| propellant | — | 0  | 0  | 0  |

| OBC build | instructions/step mean / max | exec max [ms] | latency mean / max [ms] | CPU load max | overruns |
|---|---:|---:|---:|---:|---:|
| C (arm-none-eabi-gcc -O2) | 2.135e+05 / 8.936e+05 | 6.649 | 2.757 / 7.818 | 6.6 % | 0 |
| Rust (thumbv7em-none-eabihf) | 2.686e+05 / 1.05e+06 | 7.812 | 3.168 / 8.981 | 7.8 % | 0 |

Dispatch: `dist/dispatch/ais_3u/mtq_fmr/converged` (blob, sized products, BUILD.md). C = Rust flight software bitwise on the engine: True.

