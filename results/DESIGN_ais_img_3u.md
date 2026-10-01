# Design loop: ais_img_3u

Owner: Agastya. `tools/pipeline.py ais_img_3u` — sizing (Rust, adcs-design) -> SILS mode matrix (Rust engine, POP in the loop,
C flight software) -> assess -> converge, repeated until nothing is left to change; then select, dispatch, Monte Carlo and
soft OILS of the selected configuration. Flow: `docs/figures/flow_design_to_hils.svg`, method: `docs/DESIGN_LOOP.md`.

**Selected: `mtq_fmr` — Magnetorquers + fluid momentum loop — closest (not feasible)**, converged after 6 iteration(s). Knowledge class: fine. Sensors: star_tracker, magnetometer, sun_sensors, gyro, gnss, earth_sensor.

Open requirement gaps of the selected family (what the case must relax, or the next design lever):

- fault: rotor_fail: ape_los_p9973, ake_los_p9973

## Iterations

| iteration | knobs | options feasible | selected | changes | blocked |
|---:|---|---:|---|---|---|
| 1 | laws as written | 9/25 | mtq_fmr (closest (not feasible)) | nadir_pointing/fmr+mtq: fly every pointing algorithm<br>nadir_pointing/fmr+rcs: fly every pointing algorithm<br>nadir_pointing/mtq: fly every mtq_pointing algorithm<br>sun_acquisition/mtq: fly every sun_acquisition algorithm<br>sun_referencing/mtq: fly every mtq_pointing algorithm | 15 |
| 2 | laws as written | 9/25 | mtq_fmr (closest (not feasible)) | rate stability: gyro noise x1 -> x0.3 (fibre-optic class)<br>nadir_pointing/mtq: tune every mtq_pointing law's gains, min over the gains of the worst seed (Bruni & Celani 2017)<br>sun_acquisition/mtq: tune every sun_acquisition law's gains, min over the gains of the worst seed (Bruni & Celani 2017)<br>sun_referencing/mtq: tune every mtq_pointing law's gains, min over the gains of the worst seed (Bruni & Celani 2017) | 14 |
| 3 | gyro noise x0.3 | 9/25 | mtq_fmr (closest (not feasible)) | gyro grade back to x1: rate-stability violation 13.6 -> 13.7<br>nadir_pointing/fmr+mtq: fluid-loop flow sensor 2 -> 0.5 mm/s (1 sigma)<br>mtqp: authority x1 -> x1.5 (performance) | 13 |
| 4 | mtqp x1.5, flow sensor 0.5 mm/s | 11/25 | mtq_fmr (closest (not feasible)) | mtqp: authority back to x1 (no improvement)<br>nadir_pointing/fmr+rcs: fluid-loop flow sensor 0.5 -> 0.125 mm/s (1 sigma) | 12 |
| 5 | mtqp x1, flow sensor 0.125 mm/s | 12/25 | mtq_fmr (closest (not feasible)) | mass (mtq_fmr): one star-tracker head instead of two | 11 |
| 6 | mtqp x1, 1 ST head, flow sensor 0.125 mm/s | 11/25 | mtq_fmr (feasible) | — | 13 |

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
- sun_referencing/fmr+rcs: power is the thrusters' valve power (RCS dumping), not the pump
- sun_referencing/vscmg+mtq: power fails at the sized authority (vscmg); the part's standby power is the floor
- sun_referencing/vscmg+rcs: power fails at the sized authority (vscmg); the part's standby power is the floor

## Every configuration compared (last iteration)

Selection rule (node `select`, docs/NODES.md): least mass_kg, then power_W, then volume_L, then simplicity among feasible solution families; a single fault the family does not survive is a gap (select.fault_policy gap). The benchmarks are ranked by the same rule; best benchmark: **`mtq_rw`** (feasible).

| family | role | rank | feasible | mass [kg] | power [W] | volume [L] | momentum actuator | gaps |
|---|---|---:|---|---:|---:|---:|---|---|
| mtq | solution | — | no | 0.542 | 3.02 | 0.017 | coils only | sun_acquisition: sun_acquisition_time (performance), sun_angle_p95 (performance); sun_referencing: sun_ape_p9973 (performance); nadir_pointing: ape_los_p9973 (performance), ake_los_p9973 (knowledge), rate_stability_p9973 (performance) |
| mtq_fmr | solution | — | no | 1.432 | 5.92 | 0.536 | fluid loop (3 rings) | fault: rotor_fail: ape_los_p9973, ake_los_p9973 |
| mtq_fmr_rcs | solution | — | no | 1.898 | 5.97 | 1.378 | fluid loop (3 rings) | budget: mass_kg 1.9 > 1.6; budget: volume_L 1.38 > 1; fault: rotor_fail: ape_los_p9973, ake_los_p9973 |
| mtq_rw | benchmark | 1 | yes | 0.701 | 3.71 | 0.069 | CAT-CUBESPACE-CUBEWHEEL-CW0017 x3 | — |
| mtq_rw_rcs | benchmark | 2 | yes | 1.167 | 3.76 | 0.912 | CAT-CUBESPACE-CUBEWHEEL-CW0017 x3 | — |
| mtq_cmg | benchmark | — | no | 1.721 | 6.81 | 0.808 | CAT-TENSOR-TECH-ADCS400 x4 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power); budget: mass_kg 1.72 > 1.6 |
| mtq_vscmg | benchmark | — | no | 1.721 | 6.81 | 0.808 | CAT-TENSOR-TECH-ADCS400-VSCMG x4 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power); budget: mass_kg 1.72 > 1.6 |
| mtq_cmg_rcs | benchmark | — | no | 2.187 | 6.86 | 1.650 | CAT-TENSOR-TECH-ADCS400 x4 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power); budget: mass_kg 2.19 > 1.6; budget: volume_L 1.65 > 1 |
| mtq_vscmg_rcs | benchmark | — | no | 2.187 | 6.86 | 1.650 | CAT-TENSOR-TECH-ADCS400-VSCMG x4 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power); budget: mass_kg 2.19 > 1.6; budget: volume_L 1.65 > 1 |

## Selected methods (mtq_fmr)

| mode | option | feasible | objective | algorithms | failing |
|---|---|---|---:|---|---|
| detumble | mtq | yes | 58.59 detumble_time | attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, sun_acquisition=sunspin_l1l2 | — |
| sun_acquisition | fmr | yes | 2.292 sun_acquisition_time | allocation=idmas_split, attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, pointing=pid, sun_acquisition=sunspin_l1l2 | — |
| sun_referencing | fmr+mtq | yes | 0.1571 sun_ape_p9973 | allocation=idmas_split, attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, pointing=pid, sun_acquisition=sunspin_l1l2 | — |
| nadir_pointing | fmr+mtq | yes | 0.00405 ape_los_p9973 | allocation=idmas_split, attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, pointing=pid, sun_acquisition=sunspin_l1l2 | — |

## Every solution family flown as the mission (node `family_missions`)

Detumble -> Sun acquisition -> nadir with each family's best methods from the loop, C flight software (C = Rust bit for bit), and the Monte Carlo pass rate of each requirement metric.

| family | selected | feasible | mass [kg] | methods | detumble_time | ape_los_p9973 | ake_los_p9973 | power_mean | C = Rust | MC pass rates |
|---|---|---|---:|---|---:|---:|---:|---:|---|---|
| mtq | no | no | 0.542 | detumble=mtq, sun_acquisition=mtq, sun_referencing=mtq, nadir_pointing=mtq | 59.59 ✓ | 15.15 ✗ | 0.002918 ✓ | 0.07229 ✓ | True | detumble_time 100 %; ape_los_p9973 0 %; ake_los_p9973 33 %; power_mean 100 % |
| mtq_fmr | yes | no | 1.432 | detumble=mtq, sun_acquisition=fmr, sun_referencing=fmr+mtq, nadir_pointing=fmr+mtq | 73.74 ✓ | 0.004029 ✓ | 0.003262 ✓ | 0.8019 ✓ | True | detumble_time 100 %; ape_los_p9973 100 %; ake_los_p9973 100 %; power_mean 100 % |
| mtq_fmr_rcs | no | no | 1.898 | detumble=rcs, sun_acquisition=fmr, sun_referencing=fmr+mtq, nadir_pointing=fmr+mtq | 1.058 ✓ | 0.004015 ✓ | 0.003261 ✓ | 1.495 ✓ | True | detumble_time 100 %; ape_los_p9973 100 %; ake_los_p9973 100 %; power_mean 83 % |

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
| nadir_pointing | fmr+mtq | pid@bw4 | yes | 0.00405 | — |
| nadir_pointing | fmr+rcs | smc | yes | 0.005836 | — |
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
| sun_referencing | fmr+mtq | default | yes | 0.1571 | — |
| sun_referencing | fmr+rcs | default | no | 0.1571 | power_mean (power) |
| sun_referencing | mtq | mtq_celani2026@mtq_gain_p=4,mtq_gain_d=0.25,handover_out_dps=0.25 | no | 18.26 | sun_ape_p9973 (performance) |
| sun_referencing | rw+mtq | default | yes | 0.1568 | — |
| sun_referencing | rw+rcs | default | yes | 0.1569 | — |
| sun_referencing | vscmg+mtq | default | no | 0.1571 | power_mean (power) |
| sun_referencing | vscmg+rcs | default | no | 0.1572 | power_mean (power) |

## Monte Carlo of the dispatched mission (12 runs; dispersions: inertia, residual_dipole, ltan_h, alt_km, cm_offset, solar_flux, kp, accommodation, reflectivity, arg_lat_deg, inertia_products, epoch_days)

| metric | req | mean ± std | [min, max] | pass rate |
|---|---:|---|---|---:|
| detumble_time (min) | 284 | 61.86 ± 14.4 | [22.81, 79.89] | 100 % |
| ape_los_p9973 (deg) | 0.01 | 0.004067 ± 0.000491 | [0.003526, 0.005512] | 100 % |
| ake_los_p9973 (deg) | 0.005 | 0.00327 ± 0.000527 | [0.002672, 0.004832] | 100 % |
| power_mean (W) | 2 | 1.099 ± 0.542 | [0.2427, 1.905] | 100 % |
| power_peak (W) | — | 3.262 ± 0.00981 | [3.248, 3.277] | — |
| propellant (g) | — | 0 ± 0 | [0, 0] | — |

## SILS and soft OILS of the dispatched mission

The same scenario, product and blob: in-process C (SILS) and the flight software as Cortex-M4F firmware in QEMU,
C and Rust builds, with exact per-step instruction counts and the command latency applied (docs/SOFT_OILS.md).

| metric | req | SILS | soft OILS (C on M4F) | soft OILS (Rust on M4F) |
|---|---:|---:|---:|---:|
| detumble_time | 284 | 73.74 ✓ | 74.62 ✓ | 74.62 ✓ |
| ape_los_p9973 | 0.01 | 0.004029 ✓ | 0.004103 ✓ | 0.00411 ✓ |
| ake_los_p9973 | 0.005 | 0.003262 ✓ | 0.00335 ✓ | 0.003369 ✓ |
| power_mean | 2 | 0.8019 ✓ | 0.6745 ✓ | 0.6834 ✓ |
| power_peak | — | 3.26  | 3.259  | 3.26  |
| propellant | — | 0  | 0  | 0  |

| OBC build | instructions/step mean / max | exec max [ms] | latency mean / max [ms] | CPU load max | overruns |
|---|---:|---:|---:|---:|---:|
| C (arm-none-eabi-gcc -O2) | 2.308e+05 / 7.86e+05 | 5.848 | 2.886 / 7.017 | 5.8 % | 0 |
| Rust (thumbv7em-none-eabihf) | 2.887e+05 / 8.916e+05 | 6.634 | 3.317 / 7.803 | 6.6 % | 0 |

Dispatch: `dist/dispatch/ais_img_3u/mtq_fmr/converged` (blob, sized products, BUILD.md). C = Rust flight software bitwise on the engine: True.

