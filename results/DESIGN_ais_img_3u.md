# Design loop: ais_img_3u

Owner: Agastya. `tools/pipeline.py ais_img_3u` — sizing (Rust, adcs-design) -> SILS mode matrix (Rust engine, POP in the loop,
C flight software) -> assess -> converge, repeated until nothing is left to change; then select, dispatch, Monte Carlo and
soft OILS of the selected configuration. Flow: `docs/figures/flow_design_to_hils.svg`, method: `docs/DESIGN_LOOP.md`.

**Selected: `mtq_fmr` — Magnetorquers + fluid momentum loop — feasible**, converged after 12 iteration(s). Knowledge class: fine. Sensors: star_tracker, magnetometer, sun_sensors, gyro, gnss, earth_sensor.

## Iterations

| iteration | knobs | options feasible | selected | changes | blocked |
|---:|---|---:|---|---|---|
| 1 | laws as written | 9/25 | mtq_fmr (closest (not feasible)) | nadir_pointing/fmr+mtq: fly every pointing algorithm<br>nadir_pointing/fmr+rcs: fly every pointing algorithm<br>nadir_pointing/mtq: fly every mtq_pointing algorithm<br>sun_acquisition/mtq: fly every sun_acquisition algorithm<br>sun_referencing/mtq: fly every mtq_pointing algorithm | 15 |
| 2 | laws as written | 9/25 | mtq_fmr (closest (not feasible)) | rate stability: gyro noise x1 -> x0.3 (fibre-optic class)<br>nadir_pointing/mtq: tune every mtq_pointing law's gains, min over the gains of the worst seed (Bruni & Celani 2017)<br>sun_acquisition/mtq: tune every sun_acquisition law's gains, min over the gains of the worst seed (Bruni & Celani 2017)<br>sun_referencing/mtq: tune every mtq_pointing law's gains, min over the gains of the worst seed (Bruni & Celani 2017) | 16 |
| 3 | gyro noise x0.3 | 9/25 | mtq_fmr (closest (not feasible)) | gyro grade back to x1: rate-stability violation 30 -> 30<br>nadir_pointing/fmr+mtq: fluid-loop flow sensor 2 -> 0.5 mm/s (1 sigma)<br>mtqp: authority x1 -> x1.5 (performance) | 14 |
| 4 | mtqp x1.5, flow sensor 0.5 mm/s | 11/25 | mtq_fmr (feasible) | mtqp: authority back to x1 (no improvement)<br>nadir_pointing/fmr+rcs: fluid-loop flow sensor 0.5 -> 0.125 mm/s (1 sigma) | 12 |
| 5 | mtqp x1, flow sensor 0.125 mm/s | 12/25 | mtq_fmr (feasible) | — | 11 |
| 6 | mtqp x1, flow sensor 0.125 mm/s | 10/25 | mtq_fmr (closest (not feasible)) | mass (mtq_fmr): one star-tracker head instead of two | 13 |
| 7 | mtqp x1, 1 ST head, flow sensor 0.125 mm/s | 10/25 | mtq_fmr (closest (not feasible)) | mass (mtq_fmr): lighter pump, less copper (lambda 0.1 -> 0.0333333 kg/W) | 14 |
| 8 | mtqp x1, pump lambda 0.0333333 kg/W, 1 ST head, flow sensor 0.125 mm/s | 10/25 | mtq_fmr (feasible) | — | 14 |
| 9 | mtqp x1, pump lambda 0.1 kg/W, 1 ST head, flow sensor 0.125 mm/s | 10/25 | mtq_fmr (closest (not feasible)) | mass (mtq_fmr): fluid-loop momentum x1 -> x0.75 | 14 |
| 10 | mtqp x1, fmr x0.75, pump lambda 0.1 kg/W, 1 ST head, flow sensor 0.125 mm/s | 10/25 | mtq_fmr (feasible) | — | 14 |
| 11 | mtqp x1, fmr x0.75, pump lambda 0.3 kg/W, 1 ST head, flow sensor 0.125 mm/s | 12/25 | mtq_fmr (closest (not feasible)) | mass (mtq_fmr): fluid-loop momentum x0.75 -> x0.5625 | 12 |
| 12 | mtqp x1, fmr x0.562, pump lambda 0.3 kg/W, 1 ST head, flow sensor 0.125 mm/s | 12/25 | mtq_fmr (feasible) | — | 12 |

**Redundancy (node `faults`) after iteration 5:** `mtq_fmr_rcs` did not survive fault: rotor_fail: ape_los_p9973, ake_los_p9973. redundancy (mtq_fmr_rcs): fault: rotor_fail: ape_los_p9973, ake_los_p9973 -> a spare fluid ring, skewed, that stands in for any one ring

**Robustness (node `mc`) after iteration 8:** the Monte Carlo of `mtq_fmr` failed power_mean (58 % of runs pass). robustness (mtq_fmr): mean power fails in dispersed runs -> pump with more copper (lambda 0.0333333 -> 0.1 kg/W); lighter-pump lever closed

**Robustness (node `mc`) after iteration 10:** the Monte Carlo of `mtq_fmr` failed power_mean (83 % of runs pass). robustness (mtq_fmr): mean power fails in dispersed runs -> pump with more copper (lambda 0.1 -> 0.3 kg/W); lighter-pump lever closed

Why the loop stopped (nothing left that a knob can change):

- mtqp: more authority did not reduce the performance violation (1.97e+03 -> 5.36e+03); kept at x1
- nadir_pointing/cmg+mtq: power fails at the sized authority (cmg); the part's standby power is the floor
- nadir_pointing/cmg+rcs: power fails at the sized authority (cmg); the part's standby power is the floor
- nadir_pointing/mtq: knowledge fails with the star tracker fitted
- nadir_pointing/vscmg+mtq: power fails at the sized authority (vscmg); the part's standby power is the floor
- nadir_pointing/vscmg+rcs: power fails at the sized authority (vscmg); the part's standby power is the floor
- sun_acquisition/cmg: power fails at the sized authority (cmg); the part's standby power is the floor
- sun_acquisition/vscmg: power fails at the sized authority (vscmg); the part's standby power is the floor
- sun_referencing/cmg+mtq: power fails at the sized authority (cmg); the part's standby power is the floor
- sun_referencing/cmg+rcs: power fails at the sized authority (cmg); the part's standby power is the floor
- sun_referencing/vscmg+mtq: power fails at the sized authority (vscmg); the part's standby power is the floor
- sun_referencing/vscmg+rcs: power fails at the sized authority (vscmg); the part's standby power is the floor

## Every configuration compared (last iteration)

Selection rule (node `select`, docs/NODES.md): least mass_kg, then power_W, then volume_L, then simplicity among feasible solution families; a single fault the family does not survive is a gap (select.fault_policy gap). The benchmarks are ranked by the same rule; best benchmark: **`mtq_rw`** (feasible).

| family | role | rank | feasible | mass [kg] | power [W] | volume [L] | momentum actuator | gaps |
|---|---|---:|---|---:|---:|---:|---|---|
| mtq_fmr | solution | 1 | yes | 1.794 | 3.52 | 0.660 | fluid loop (3 rings) | — |
| mtq | solution | — | no | 0.542 | 3.02 | 0.017 | coils only | sun_acquisition: sun_acquisition_time (performance), sun_angle_p95 (performance); sun_referencing: sun_ape_p9973 (performance); nadir_pointing: ape_los_p9973 (performance), ake_los_p9973 (knowledge), rate_stability_p9973 (performance) |
| mtq_fmr_rcs | solution | — | no | 2.261 | 3.57 | 1.502 | fluid loop (3 rings) | budget: mass_kg 2.26 > 1.85; budget: volume_L 1.5 > 1 |
| mtq_rw | benchmark | 1 | yes | 0.701 | 3.71 | 0.069 | CAT-CUBESPACE-CUBEWHEEL-CW0017 x3 | — |
| mtq_rw_rcs | benchmark | 2 | yes | 1.167 | 3.76 | 0.912 | CAT-CUBESPACE-CUBEWHEEL-CW0017 x3 | — |
| mtq_cmg | benchmark | — | no | 1.721 | 6.81 | 0.808 | CAT-TENSOR-TECH-ADCS400 x4 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power) |
| mtq_vscmg | benchmark | — | no | 1.721 | 6.81 | 0.808 | CAT-TENSOR-TECH-ADCS400-VSCMG x4 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power) |
| mtq_cmg_rcs | benchmark | — | no | 2.187 | 6.86 | 1.650 | CAT-TENSOR-TECH-ADCS400 x4 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power); budget: mass_kg 2.19 > 1.85; budget: volume_L 1.65 > 1 |
| mtq_vscmg_rcs | benchmark | — | no | 2.187 | 6.86 | 1.650 | CAT-TENSOR-TECH-ADCS400-VSCMG x4 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power); budget: mass_kg 2.19 > 1.85; budget: volume_L 1.65 > 1 |

## Selected methods (mtq_fmr)

| mode | option | feasible | objective | algorithms | failing |
|---|---|---|---:|---|---|
| detumble | mtq | yes | 58.59 detumble_time | attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, sun_acquisition=sunspin_l1l2 | — |
| sun_acquisition | fmr | yes | 2.292 sun_acquisition_time | allocation=idmas_split, attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, pointing=pid, sun_acquisition=sunspin_l1l2 | — |
| sun_referencing | fmr+mtq | yes | 0.1565 sun_ape_p9973 | allocation=idmas_split, attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, pointing=pid, sun_acquisition=sunspin_l1l2 | — |
| nadir_pointing | fmr+mtq | yes | 0.004032 ape_los_p9973 | allocation=idmas_split, attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, pointing=pid, sun_acquisition=sunspin_l1l2 | — |

## Every solution family flown as the mission (node `family_missions`)

Detumble -> Sun acquisition -> nadir with each family's best methods from the loop, C flight software (C = Rust bit for bit), and the Monte Carlo pass rate of each requirement metric.

| family | selected | feasible | mass [kg] | methods | detumble_time | ape_los_p9973 | ake_los_p9973 | power_mean | C = Rust | MC pass rates |
|---|---|---|---:|---|---:|---:|---:|---:|---|---|
| mtq | no | no | 0.542 | detumble=mtq, sun_acquisition=mtq, sun_referencing=mtq, nadir_pointing=mtq | 59.59 ✓ | 15.15 ✗ | 0.002918 ✓ | 0.07229 ✓ | True | detumble_time 100 %; ape_los_p9973 0 %; ake_los_p9973 33 %; power_mean 100 % |
| mtq_fmr | yes | yes | 1.794 | detumble=mtq, sun_acquisition=fmr, sun_referencing=fmr+mtq, nadir_pointing=fmr+mtq | 73.83 ✓ | 0.004066 ✓ | 0.003305 ✓ | 0.3934 ✓ | True | detumble_time 100 %; ape_los_p9973 100 %; ake_los_p9973 100 %; power_mean 100 % |
| mtq_fmr_rcs | no | no | 2.261 | detumble=rcs, sun_acquisition=fmr, sun_referencing=fmr+rcs, nadir_pointing=fmr+mtq | 2.025 ✓ | 0.00406 ✓ | 0.003309 ✓ | 0.3551 ✓ | True | detumble_time 100 %; ape_los_p9973 100 %; ake_los_p9973 100 %; power_mean 100 % |

## Coils only: every law of the literature, each at its best gains (nodes `matrix` + `tune`)

Each law's result is its worst seed at the gains that make that worst seed best (Bruni & Celani's min-max); the laws are listed by mode, best first. Sources: docs/MTQ_LITERATURE.md.

| mode | law | paper | best gains | feasible | objective (worst seed) | failing |
|---|---|---|---|---|---:|---|
| sun_acquisition | sunspin_deruiter2011 | P11 -> P2 de Ruiter 2011 | spin_rate_dps 2, ss_gain 3 | no | 76.71 sun_acquisition_time | sun_acquisition_time, sun_angle_p95 |
| sun_acquisition | sunspin_l1l2_e2 | P11 -> P5, eclipse E2 | spin_rate_dps 4, ss_gain 1 | no | 77.84 sun_acquisition_time | sun_acquisition_time, sun_angle_p95 |
| sun_acquisition | sunspin_damped | P11 -> P5, R_z floor | spin_rate_dps 4, ss_gain 1 | no | 79.34 sun_acquisition_time | sun_acquisition_time, sun_angle_p95 |
| sun_acquisition | sunspin_l1l2 | P11 UPMSat-2 -> P5 He et al. 2023 | spin_rate_dps 4, ss_gain 0.3 | no | 85.51 sun_acquisition_time | sun_acquisition_time, sun_angle_p95 |
| sun_acquisition | sun_boresight_celani2026 | P8 Celani 2026 | nominal | no | 136 sun_acquisition_time | sun_angle_p95, sun_acquisition_time |
| sun_referencing | mtq_celani2026 | P8 Celani 2026 | mtq_gain_p 4, mtq_gain_d 0.25, handover_out_dps 0.25 | no | 18.26 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_tango2013 | P3 TANGO 2013 (flown) | mtq_gain_p 0.25, mtq_gain_d 4, handover_out_dps 0.25 | no | 56.17 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_celani2015 | P4 Celani 2015 | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 0.25 | no | 66.58 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_avanzini2021 | P16 Avanzini et al. 2021 | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 0.25 | no | 66.58 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_lovera2004 | P1 Lovera & Astolfi 2004 | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 0.25 | no | 138.5 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_rate_damp | baseline rate damping | nominal | no | 153.5 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_smc | baseline SMC | nominal | no | 175.1 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_lqr | baseline LQR | nominal | no | 179 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_pd | baseline PD | nominal | no | 179.2 sun_ape_p9973 | sun_ape_p9973 |
| nadir_pointing | mtq_celani2026 | P8 Celani 2026 | mtq_gain_p 1, mtq_gain_d 0.25, handover_out_dps 1 | no | 22.77 ape_los_p9973 | ape_los_p9973, ake_los_p9973, rate_stability_p9973 |
| nadir_pointing | mtq_celani2015 | P4 Celani 2015 | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 0.5 | no | 42.08 ape_los_p9973 | ape_los_p9973, ake_los_p9973, rate_stability_p9973 |
| nadir_pointing | mtq_lovera2004 | P1 Lovera & Astolfi 2004 | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 0.25 | no | 62.15 ape_los_p9973 | ape_los_p9973, ake_los_p9973, rate_stability_p9973 |
| nadir_pointing | mtq_tango2013 | P3 TANGO 2013 (flown) | mtq_gain_p 1, mtq_gain_d 4, handover_out_dps 0.5 | no | 100.6 ape_los_p9973 | ape_los_p9973, ake_los_p9973, rate_stability_p9973 |
| nadir_pointing | mtq_rate_damp | baseline rate damping | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 1 | no | 103.4 ape_los_p9973 | ape_los_p9973, rate_stability_p9973, ake_los_p9973 |
| nadir_pointing | mtq_avanzini2021 | P16 Avanzini et al. 2021 | mtq_gain_p 4, mtq_gain_d 4, handover_out_dps 0.5 | no | 138.8 ape_los_p9973 | ape_los_p9973, ake_los_p9973, rate_stability_p9973 |
| nadir_pointing | mtq_pd | baseline PD | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 1 | no | 157.7 ape_los_p9973 | ape_los_p9973, ake_los_p9973, rate_stability_p9973 |
| nadir_pointing | mtq_lqr | baseline LQR | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 0.25 | no | 169.9 ape_los_p9973 | ape_los_p9973, ake_los_p9973, rate_stability_p9973 |
| nadir_pointing | mtq_smc | baseline SMC | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 0.25 | no | 176.6 ape_los_p9973 | ape_los_p9973, ake_los_p9973, rate_stability_p9973 |

## Coils-only nadir loop certificate (node `certify`, Floquet multipliers, Celani 2026's method)

The loop linearised about nadir with the gyroscopic and gravity-gradient terms, the coil duty and the field along one orbit; all |mu| < 1 certifies the periodic loop (a boresight law keeps one multiplier at 1 by design).

| law | gains | max abs(mu) | certified |
|---|---|---:|---|
| mtq_pd | nominal | 4.9033 | no |
| mtq_rate_damp | nominal | 1.9820 | no |
| mtq_lovera2004 | nominal | 4.8218 | no |
| mtq_celani2015 | nominal | 2.4125 | no |
| mtq_avanzini2021 | nominal | 7.0389 | no |
| mtq_celani2026 (dispatched) | {'mtq_gain_p': 1.0, 'mtq_gain_d': 0.25} | 4.1581 | no |
| mtq_tango2013 | nominal | 7.1476 | no |

## Mode matrix (last iteration, best algorithm per option)

| mode | option | algorithm | feasible | objective | failing (cause) |
|---|---|---|---|---:|---|
| detumble | mtq | default | yes | 58.59 | — |
| detumble | rcs | default | yes | 1.125 | — |
| nadir_pointing | cmg+mtq | default | no | 0.005534 | power_mean (power) |
| nadir_pointing | cmg+rcs | default | no | 0.005547 | power_mean (power) |
| nadir_pointing | fmr+mtq | pid@bw4 | yes | 0.004032 | — |
| nadir_pointing | fmr+rcs | pid@bw4 | yes | 0.004042 | — |
| nadir_pointing | mtq | mtq_celani2026@mtq_gain_p=1,mtq_gain_d=0.25,handover_out_dps=1 | no | 22.77 | ape_los_p9973 (performance), ake_los_p9973 (knowledge), rate_stability_p9973 (performance) |
| nadir_pointing | rw+mtq | default | yes | 0.00572 | — |
| nadir_pointing | rw+rcs | default | yes | 0.005743 | — |
| nadir_pointing | vscmg+mtq | default | no | 0.005972 | power_mean (power) |
| nadir_pointing | vscmg+rcs | default | no | 0.006053 | power_mean (power) |
| sun_acquisition | cmg | default | no | 2.275 | power_mean (power) |
| sun_acquisition | fmr | default | yes | 2.292 | — |
| sun_acquisition | mtq | sunspin_deruiter2011@spin_rate_dps=2,ss_gain=3 | no | 76.71 | sun_acquisition_time (performance), sun_angle_p95 (performance) |
| sun_acquisition | rw | default | yes | 2.258 | — |
| sun_acquisition | vscmg | default | no | 2.342 | power_mean (power) |
| sun_referencing | cmg+mtq | default | no | 0.157 | power_mean (power) |
| sun_referencing | cmg+rcs | default | no | 0.157 | power_mean (power) |
| sun_referencing | fmr+mtq | default | yes | 0.1565 | — |
| sun_referencing | fmr+rcs | default | yes | 0.1564 | — |
| sun_referencing | mtq | mtq_celani2026@mtq_gain_p=4,mtq_gain_d=0.25,handover_out_dps=0.25 | no | 18.26 | sun_ape_p9973 (performance) |
| sun_referencing | rw+mtq | default | yes | 0.1568 | — |
| sun_referencing | rw+rcs | default | yes | 0.1569 | — |
| sun_referencing | vscmg+mtq | default | no | 0.1571 | power_mean (power) |
| sun_referencing | vscmg+rcs | default | no | 0.1572 | power_mean (power) |

## Monte Carlo of the dispatched mission (12 runs; dispersions: inertia, residual_dipole, ltan_h, alt_km, cm_offset, solar_flux, kp, accommodation, reflectivity, arg_lat_deg, inertia_products, epoch_days)

| metric | req | mean ± std | [min, max] | pass rate |
|---|---:|---|---|---:|
| detumble_time (min) | 284 | 61.91 ± 14.4 | [22.82, 79.89] | 100 % |
| ape_los_p9973 (deg) | 0.01 | 0.004101 ± 0.000467 | [0.003495, 0.00542] | 100 % |
| ake_los_p9973 (deg) | 0.005 | 0.003256 ± 0.000524 | [0.002726, 0.004819] | 100 % |
| power_mean (W) | 2 | 0.3501 ± 0.081 | [0.1582, 0.4945] | 100 % |
| power_peak (W) | — | 0.8539 ± 0.00929 | [0.8361, 0.8695] | — |
| propellant (g) | — | 0 ± 0 | [0, 0] | — |

## SILS and soft OILS of the dispatched mission

The same scenario, product and blob: in-process C (SILS) and the flight software as Cortex-M4F firmware in QEMU,
C and Rust builds, with exact per-step instruction counts and the command latency applied (docs/SOFT_OILS.md).

| metric | req | SILS | soft OILS (C on M4F) | soft OILS (Rust on M4F) |
|---|---:|---:|---:|---:|
| detumble_time | 284 | 73.83 ✓ | 74.71 ✓ | 74.79 ✓ |
| ape_los_p9973 | 0.01 | 0.004066 ✓ | 0.004204 ✓ | 0.004229 ✓ |
| ake_los_p9973 | 0.005 | 0.003305 ✓ | 0.003374 ✓ | 0.003401 ✓ |
| power_mean | 2 | 0.3934 ✓ | 0.3578 ✓ | 0.3575 ✓ |
| power_peak | — | 0.856  | 0.859  | 0.8594  |
| propellant | — | 0  | 0  | 0  |

| OBC build | instructions/step mean / max | exec max [ms] | latency mean / max [ms] | CPU load max | overruns |
|---|---:|---:|---:|---:|---:|
| C (arm-none-eabi-gcc -O2) | 2.396e+05 / 7.996e+05 | 5.950 | 3.082 / 7.249 | 5.9 % | 0 |
| Rust (thumbv7em-none-eabihf) | 2.973e+05 / 9.028e+05 | 6.717 | 3.511 / 8.016 | 6.7 % | 0 |

Dispatch: `dist/dispatch/ais_img_3u/mtq_fmr/converged` (blob, sized products, BUILD.md). C = Rust flight software bitwise on the engine: True.

