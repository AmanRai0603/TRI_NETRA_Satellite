# Design loop: ais_3u

Owner: Agastya. `tools/pipeline.py ais_3u` — sizing (Rust, adcs-design) -> SILS mode matrix (Rust engine, POP in the loop,
C flight software) -> assess -> converge, repeated until nothing is left to change; then select, dispatch, Monte Carlo and
soft OILS of the selected configuration. Flow: `docs/figures/flow_design_to_hils.svg`, method: `docs/DESIGN_LOOP.md`.

**Selected: `mtq_fmr` — Magnetorquers + fluid momentum loop — feasible**, converged after 6 iteration(s). Knowledge class: coarse. Sensors: magnetometer, sun_sensors, gyro, gnss, earth_sensor.

## Iterations

| iteration | knobs | options feasible | selected | changes | blocked |
|---:|---|---:|---|---|---|
| 1 | laws as written | 1/25 | mtq (closest (not feasible)) | nadir_pointing/fmr+mtq: power -> electromagnetic pump with more copper (lambda 0.1 -> 0.3 kg/W)<br>nadir_pointing/mtq: fly every mtq_pointing algorithm<br>sun_acquisition/mtq: fly every sun_acquisition algorithm<br>sun_referencing/mtq: fly every mtq_pointing algorithm | 16 |
| 2 | pump lambda 0.3 kg/W | 2/25 | mtq_fmr (closest (not feasible)) | nadir_pointing/fmr+mtq: power -> electromagnetic pump with more copper (lambda 0.3 -> 0.9 kg/W)<br>nadir_pointing/mtq: tune every mtq_pointing law's gains, min over the gains of the worst seed (Bruni & Celani 2017)<br>sun_acquisition/mtq: tune every sun_acquisition law's gains, min over the gains of the worst seed (Bruni & Celani 2017)<br>sun_referencing/mtq: tune every mtq_pointing law's gains, min over the gains of the worst seed (Bruni & Celani 2017) | 16 |
| 3 | pump lambda 0.9 kg/W | 4/25 | mtq_fmr (closest (not feasible)) | mass (mtq_fmr): fluid-loop momentum x1 -> x0.75<br>mtqp: authority x1 -> x1.5 (performance) | 18 |
| 4 | fmr x0.75, mtqp x1.5, pump lambda 0.9 kg/W | 5/25 | mtq_fmr (closest (not feasible)) | mtqp: authority back to x1 (no improvement) | 19 |
| 5 | fmr x0.75, mtqp x1, pump lambda 0.9 kg/W | 4/25 | mtq_fmr (closest (not feasible)) | mass (mtq_fmr): fluid-loop momentum x0.75 -> x0.5625 | 19 |
| 6 | fmr x0.562, mtqp x1, pump lambda 0.9 kg/W | 6/25 | mtq_fmr (feasible) | — | 17 |

Why the loop stopped (nothing left that a knob can change):

- detumble/rcs: power fails at the sized authority (rcs); the part's standby power is the floor
- mtqp: more authority did not reduce the performance violation (17.8 -> 17.7); kept at x1
- nadir_pointing/cmg+mtq: power fails at the sized authority (cmg); the part's standby power is the floor
- nadir_pointing/cmg+rcs: power fails at the sized authority (cmg); the part's standby power is the floor
- nadir_pointing/rw+mtq: power fails at the sized authority (rw); the part's standby power is the floor
- nadir_pointing/rw+rcs: power fails at the sized authority (rw); the part's standby power is the floor
- nadir_pointing/vscmg+mtq: power fails at the sized authority (vscmg); the part's standby power is the floor
- nadir_pointing/vscmg+rcs: power fails at the sized authority (vscmg); the part's standby power is the floor
- sun_acquisition/cmg: power fails at the sized authority (cmg); the part's standby power is the floor
- sun_acquisition/rw: power fails at the sized authority (rw); the part's standby power is the floor
- sun_acquisition/vscmg: power fails at the sized authority (vscmg); the part's standby power is the floor
- sun_referencing/cmg+mtq: power fails at the sized authority (cmg); the part's standby power is the floor
- sun_referencing/cmg+rcs: power fails at the sized authority (cmg); the part's standby power is the floor
- sun_referencing/rw+mtq: power fails at the sized authority (rw); the part's standby power is the floor
- sun_referencing/rw+rcs: power fails at the sized authority (rw); the part's standby power is the floor
- sun_referencing/vscmg+mtq: power fails at the sized authority (vscmg); the part's standby power is the floor
- sun_referencing/vscmg+rcs: power fails at the sized authority (vscmg); the part's standby power is the floor

## Every configuration compared (last iteration)

Selection rule (node `select`, docs/NODES.md): least mass_kg, then power_W, then volume_L, then simplicity among feasible solution families; a single fault the family does not survive is a gap (select.fault_policy gap). The benchmarks are ranked by the same rule; best benchmark: **`mtq_rw`** (closest (not feasible)).

| family | role | rank | feasible | mass [kg] | power [W] | volume [L] | momentum actuator | gaps |
|---|---|---:|---|---:|---:|---:|---|---|
| mtq_fmr | solution | 1 | yes | 1.012 | 2.06 | 0.538 | fluid loop (3 rings) | — |
| mtq | solution | — | no | 0.240 | 2.10 | 0.020 | coils only | sun_acquisition: sun_angle_p95 (performance); sun_referencing: sun_ape_p9973 (performance); nadir_pointing: ape_los_p9973 (performance); fault: coil_fail: ape_los_p9973; fault: gyro_bias_step: ape_los_p9973 |
| mtq_fmr_rcs | solution | — | no | 1.544 | 2.11 | 1.557 | fluid loop (3 rings) | budget: volume_L 1.56 > 1 |
| mtq_rw | benchmark | — | no | 0.395 | 2.75 | 0.071 | CAT-CUBESPACE-CUBEWHEEL-CW0017 x3 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power) |
| mtq_rw_rcs | benchmark | — | no | 0.927 | 2.80 | 1.090 | CAT-CUBESPACE-CUBEWHEEL-CW0017 x3 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power); budget: volume_L 1.09 > 1 |
| mtq_cmg | benchmark | — | no | 1.415 | 5.85 | 0.810 | CAT-TENSOR-TECH-ADCS400 x4 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power) |
| mtq_vscmg | benchmark | — | no | 1.415 | 5.85 | 0.810 | CAT-TENSOR-TECH-ADCS400-VSCMG x4 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power) |
| mtq_cmg_rcs | benchmark | — | no | 1.947 | 5.90 | 1.829 | CAT-TENSOR-TECH-ADCS400 x4 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power); budget: mass_kg 1.95 > 1.6; budget: volume_L 1.83 > 1 |
| mtq_vscmg_rcs | benchmark | — | no | 1.947 | 5.90 | 1.829 | CAT-TENSOR-TECH-ADCS400-VSCMG x4 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power); budget: mass_kg 1.95 > 1.6; budget: volume_L 1.83 > 1 |

## Selected methods (mtq_fmr)

| mode | option | feasible | objective | algorithms | failing |
|---|---|---|---:|---|---|
| detumble | mtq | yes | 41.31 detumble_time | attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, sun_acquisition=sunspin_l1l2 | — |
| sun_acquisition | fmr | yes | 2.292 sun_acquisition_time | allocation=idmas_split, attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, pointing=pid, sun_acquisition=sunspin_l1l2 | — |
| sun_referencing | fmr+mtq | yes | 0.3733 sun_ape_p9973 | allocation=idmas_split, attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, pointing=pid, sun_acquisition=sunspin_l1l2 | — |
| nadir_pointing | fmr+mtq | yes | 0.1491 ape_los_p9973 | allocation=idmas_split, attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, pointing=pid, sun_acquisition=sunspin_l1l2 | — |

## Every solution family flown as the mission (node `family_missions`)

Detumble -> Sun acquisition -> nadir with each family's best methods from the loop, C flight software (C = Rust bit for bit), and the Monte Carlo pass rate of each requirement metric.

| family | selected | feasible | mass [kg] | methods | detumble_time | ape_los_p9973 | ake_los_p9973 | power_mean | C = Rust | MC pass rates |
|---|---|---|---:|---|---:|---:|---:|---:|---|---|
| mtq | no | no | 0.240 | detumble=mtq, sun_acquisition=mtq, sun_referencing=mtq, nadir_pointing=mtq | 42.57 ✓ | 5.703 ✓ | 0.1246 ✓ | 0.09076 ✓ | True | detumble_time 100 %; ape_los_p9973 33 %; ake_los_p9973 100 %; power_mean 100 % |
| mtq_fmr | yes | yes | 1.012 | detumble=mtq, sun_acquisition=fmr, sun_referencing=fmr+mtq, nadir_pointing=fmr+mtq | 52.91 ✓ | 0.1584 ✓ | 0.1568 ✓ | 0.1949 ✓ | True | detumble_time 100 %; ape_los_p9973 100 %; ake_los_p9973 100 %; power_mean 100 % |
| mtq_fmr_rcs | no | no | 1.544 | detumble=mtq, sun_acquisition=fmr, sun_referencing=fmr+mtq, nadir_pointing=fmr+rcs | 52.91 ✓ | 0.1547 ✓ | 0.153 ✓ | 0.1755 ✓ | True | detumble_time 100 %; ape_los_p9973 100 %; ake_los_p9973 100 %; power_mean 100 % |

## Coils only: every law of the literature, each at its best gains (nodes `matrix` + `tune`)

Each law's result is its worst seed at the gains that make that worst seed best (Bruni & Celani's min-max); the laws are listed by mode, best first. Sources: docs/MTQ_LITERATURE.md.

| mode | law | paper | best gains | feasible | objective (worst seed) | failing |
|---|---|---|---|---|---:|---|
| sun_acquisition | sunspin_damped | P11 -> P5, R_z floor | spin_rate_dps 2, ss_gain 3 | no | 71.84 sun_acquisition_time | sun_angle_p95 |
| sun_acquisition | sunspin_l1l2 | P11 UPMSat-2 -> P5 He et al. 2023 | spin_rate_dps 2, ss_gain 3 | no | 82.39 sun_acquisition_time | sun_angle_p95 |
| sun_acquisition | sunspin_l1l2_e2 | P11 -> P5, eclipse E2 | spin_rate_dps 2, ss_gain 3 | no | 82.39 sun_acquisition_time | sun_angle_p95 |
| sun_acquisition | sun_boresight_celani2026 | P8 Celani 2026 | nominal | no | 79.37 sun_acquisition_time | sun_angle_p95, sun_acquisition_time |
| sun_acquisition | sunspin_deruiter2011 | P11 -> P2 de Ruiter 2011 | spin_rate_dps 6, ss_gain 0.3 | no | 92.81 sun_acquisition_time | sun_acquisition_time, sun_angle_p95 |
| sun_referencing | mtq_celani2015 | P4 Celani 2015 | mtq_gain_p 0.25, mtq_gain_d 1, handover_out_dps 0.25 | no | 85.68 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_avanzini2021 | P16 Avanzini et al. 2021 | mtq_gain_p 0.25, mtq_gain_d 1, handover_out_dps 0.25 | no | 85.68 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_celani2026 | P8 Celani 2026 | mtq_gain_p 0.25, mtq_gain_d 1, handover_out_dps 0.25 | no | 86.16 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_tango2013 | P3 TANGO 2013 (flown) | mtq_gain_p 0.25, mtq_gain_d 1, handover_out_dps 0.25 | no | 133.1 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_lovera2004 | P1 Lovera & Astolfi 2004 | mtq_gain_p 1, mtq_gain_d 0.25, handover_out_dps 1 | no | 138.6 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_pd | baseline PD | nominal | no | 150.4 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_rate_damp | baseline rate damping | nominal | no | 164.7 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_lqr | baseline LQR | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 1 | no | 174 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_smc | baseline SMC | nominal | no | 177.5 sun_ape_p9973 | sun_ape_p9973 |
| nadir_pointing | mtq_celani2026 | P8 Celani 2026 | mtq_gain_p 4, mtq_gain_d 0.25, handover_out_dps 0.5 | no | 10.31 ape_los_p9973 | ape_los_p9973 |
| nadir_pointing | mtq_pd | baseline PD | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 1 | no | 11.24 ape_los_p9973 | ape_los_p9973 |
| nadir_pointing | mtq_smc | baseline SMC | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 1 | no | 12.79 ape_los_p9973 | ape_los_p9973 |
| nadir_pointing | mtq_tango2013 | P3 TANGO 2013 (flown) | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 1 | no | 24.08 ape_los_p9973 | ape_los_p9973 |
| nadir_pointing | mtq_celani2015 | P4 Celani 2015 | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 0.25 | no | 26.22 ape_los_p9973 | ape_los_p9973 |
| nadir_pointing | mtq_lovera2004 | P1 Lovera & Astolfi 2004 | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 0.25 | no | 32.68 ape_los_p9973 | ape_los_p9973 |
| nadir_pointing | mtq_lqr | baseline LQR | nominal | no | 36.65 ape_los_p9973 | ape_los_p9973 |
| nadir_pointing | mtq_avanzini2021 | P16 Avanzini et al. 2021 | mtq_gain_p 4, mtq_gain_d 1, handover_out_dps 0.5 | no | 88.14 ape_los_p9973 | ape_los_p9973 |
| nadir_pointing | mtq_rate_damp | baseline rate damping | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 0.25 | no | 177.1 ape_los_p9973 | ape_los_p9973 |

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
| detumble | mtq | default | yes | 41.31 | — |
| detumble | rcs | default | no | 1.342 | power_peak (power) |
| nadir_pointing | cmg+mtq | default | no | 0.1465 | power_mean (power) |
| nadir_pointing | cmg+rcs | default | no | 0.1465 | power_mean (power) |
| nadir_pointing | fmr+mtq | default | yes | 0.1491 | — |
| nadir_pointing | fmr+rcs | default | yes | 0.1491 | — |
| nadir_pointing | mtq | mtq_celani2026@mtq_gain_p=4,mtq_gain_d=0.25,handover_out_dps=0.5 | no | 10.31 | ape_los_p9973 (performance) |
| nadir_pointing | rw+mtq | default | no | 0.1465 | power_mean (power) |
| nadir_pointing | rw+rcs | default | no | 0.1466 | power_mean (power) |
| nadir_pointing | vscmg+mtq | default | no | 0.1465 | power_mean (power) |
| nadir_pointing | vscmg+rcs | default | no | 0.1465 | power_mean (power) |
| sun_acquisition | cmg | default | no | 2.275 | power_mean (power) |
| sun_acquisition | fmr | default | yes | 2.292 | — |
| sun_acquisition | mtq | sunspin_damped@spin_rate_dps=2,ss_gain=3 | no | 71.84 | sun_angle_p95 (performance) |
| sun_acquisition | rw | default | no | 2.275 | power_mean (power) |
| sun_acquisition | vscmg | default | no | 2.392 | power_mean (power) |
| sun_referencing | cmg+mtq | default | no | 0.3655 | power_mean (power) |
| sun_referencing | cmg+rcs | default | no | 0.3656 | power_mean (power) |
| sun_referencing | fmr+mtq | default | yes | 0.3733 | — |
| sun_referencing | fmr+rcs | default | yes | 0.3734 | — |
| sun_referencing | mtq | mtq_celani2015@mtq_gain_p=0.25,mtq_gain_d=1,handover_out_dps=0.25 | no | 85.68 | sun_ape_p9973 (performance) |
| sun_referencing | rw+mtq | default | no | 0.3659 | power_mean (power) |
| sun_referencing | rw+rcs | default | no | 0.3659 | power_mean (power) |
| sun_referencing | vscmg+mtq | default | no | 0.3656 | power_mean (power) |
| sun_referencing | vscmg+rcs | default | no | 0.3657 | power_mean (power) |

## Monte Carlo of the dispatched mission (12 runs; dispersions: inertia, residual_dipole, ltan_h, alt_km, cm_offset, solar_flux, kp, accommodation, reflectivity, arg_lat_deg, inertia_products, epoch_days)

| metric | req | mean ± std | [min, max] | pass rate |
|---|---:|---|---|---:|
| detumble_time (min) | 284 | 59.09 ± 14.9 | [26.21, 76.66] | 100 % |
| ape_los_p9973 (deg) | 10 | 0.3205 ± 0.114 | [0.1493, 0.5514] | 100 % |
| ake_los_p9973 (deg) | 5 | 0.3192 ± 0.113 | [0.1527, 0.5484] | 100 % |
| power_mean (W) | 0.5 | 0.1715 ± 0.0205 | [0.1327, 0.2044] | 100 % |
| power_peak (W) | — | 0.3826 ± 0.00515 | [0.3764, 0.3912] | — |
| propellant (g) | — | 0 ± 0 | [0, 0] | — |

## SILS and soft OILS of the dispatched mission

The same scenario, product and blob: in-process C (SILS) and the flight software as Cortex-M4F firmware in QEMU,
C and Rust builds, with exact per-step instruction counts and the command latency applied (docs/SOFT_OILS.md).

| metric | req | SILS | soft OILS (C on M4F) | soft OILS (Rust on M4F) |
|---|---:|---:|---:|---:|
| detumble_time | 284 | 52.91 ✓ | —  | —  |
| ape_los_p9973 | 10 | 0.1584 ✓ | —  | —  |
| ake_los_p9973 | 5 | 0.1568 ✓ | —  | —  |
| power_mean | 0.5 | 0.1949 ✓ | —  | —  |
| power_peak | — | 0.4009  | —  | —  |
| propellant | — | 0  | —  | —  |

| OBC build | instructions/step mean / max | exec max [ms] | latency mean / max [ms] | CPU load max | overruns |
|---|---:|---:|---:|---:|---:|

Dispatch: `dist/dispatch/ais_3u/mtq_fmr/converged` (blob, sized products, BUILD.md). C = Rust flight software bitwise on the engine: True.

