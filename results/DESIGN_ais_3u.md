# Design loop: ais_3u

Owner: Agastya. `tools/pipeline.py ais_3u` — sizing (Rust, adcs-design) -> SILS mode matrix (Rust engine, POP in the loop,
C flight software) -> assess -> converge, repeated until nothing is left to change; then select, dispatch, Monte Carlo and
soft OILS of the selected configuration. Flow: `docs/figures/flow_design_to_hils.svg`, method: `docs/DESIGN_LOOP.md`.

**Selected: `mtq_fmr` — Magnetorquers + fluid momentum loop — closest (not feasible)**, converged after 8 iteration(s). Knowledge class: coarse. Sensors: magnetometer, sun_sensors, gyro, gnss, earth_sensor.

Open requirement gaps of the selected family (what the case must relax, or the next design lever):

- detumble: power_peak (power)
- sun_acquisition: power_mean (power)
- sun_referencing: sun_ape_p9973 (performance), power_mean (power)
- nadir_pointing: power_mean (power)
- budget: mass_kg 11.3 > 1.6
- budget: power_peak_W 3.86 > 1.5

## Iterations

| iteration | knobs | options feasible | selected | changes | blocked |
|---:|---|---:|---|---|---|
| 1 | laws as written | 0/25 | mtq_fmr (closest (not feasible)) | nadir_pointing/fmr+mtq: power -> electromagnetic pump with more copper (lambda 0.1 -> 0.3 kg/W)<br>nadir_pointing/mtq: fly every mtq_pointing algorithm<br>sun_acquisition/mtq: fly every sun_acquisition algorithm<br>sun_referencing/fmr+mtq: fly every pointing algorithm<br>sun_referencing/fmr+rcs: fly every pointing algorithm<br>sun_referencing/mtq: fly every mtq_pointing algorithm<br>mtqp: authority x1 -> x0.75 (power) | 16 |
| 2 | mtqp x0.75, pump lambda 0.3 kg/W | 0/25 | mtq_fmr_rcs (closest (not feasible)) | nadir_pointing/fmr+mtq: fly every pointing algorithm<br>nadir_pointing/fmr+mtq: power -> electromagnetic pump with more copper (lambda 0.3 -> 0.9 kg/W)<br>nadir_pointing/fmr+rcs: fly every pointing algorithm<br>sun_acquisition/mtq: tune every sun_acquisition law's gains, min over the gains of the worst seed (Bruni & Celani 2017)<br>sun_referencing/mtq: tune every mtq_pointing law's gains, min over the gains of the worst seed (Bruni & Celani 2017)<br>mtqp: authority x0.75 -> x0.5625 (power) | 17 |
| 3 | mtqp x0.562, pump lambda 0.9 kg/W | 0/25 | mtq_fmr_rcs (closest (not feasible)) | nadir_pointing/fmr+mtq: power -> electromagnetic pump with more copper (lambda 0.9 -> 2.7 kg/W)<br>mtqp: authority x0.5625 -> x0.5 (power) | 20 |
| 4 | mtqp x0.5, pump lambda 2.7 kg/W | 0/25 | mtq_fmr_rcs (closest (not feasible)) | nadir_pointing/fmr+mtq: power -> electromagnetic pump with more copper (lambda 2.7 -> 3 kg/W)<br>nadir_pointing/mtq: tune every mtq_pointing law's gains, min over the gains of the worst seed (Bruni & Celani 2017) | 21 |
| 5 | mtqp x0.5, pump lambda 3 kg/W | 0/25 | mtq_fmr_rcs (closest (not feasible)) | — | 26 |
| 6 | mtqp x0.5, fmr x1.5, pump lambda 3 kg/W | 0/25 | mtq_fmr_rcs (closest (not feasible)) | — | 27 |
| 7 | mtqp x0.5, fmr x2.25, pump lambda 3 kg/W | 0/25 | mtq_fmr_rcs (closest (not feasible)) | — | 27 |
| 8 | mtqp x0.5, fmr x3.38, pump lambda 3 kg/W | 0/25 | mtq_fmr_rcs (closest (not feasible)) | — | 26 |

**Robustness (node `mc`) after iteration 5:** the Monte Carlo of `mtq_fmr` failed ape_los_p9973 (0 % of runs pass), power_mean (0 % of runs pass). robustness (mtq_fmr): performance fails in dispersed runs -> fmr authority x1 -> x1.5 robustness (mtq_fmr): power fails in dispersed runs and no power lever is left

**Robustness (node `mc`) after iteration 6:** the Monte Carlo of `mtq_fmr_rcs` failed ape_los_p9973 (83 % of runs pass), power_mean (0 % of runs pass). robustness (mtq_fmr_rcs): performance fails in dispersed runs -> fmr authority x1.5 -> x2.25 robustness (mtq_fmr_rcs): power fails in dispersed runs and no power lever is left

**Robustness (node `mc`) after iteration 7:** the Monte Carlo of `mtq_fmr` failed detumble_time (92 % of runs pass), ape_los_p9973 (8 % of runs pass), power_mean (0 % of runs pass). robustness (mtq_fmr): performance fails in dispersed runs -> fmr authority x2.25 -> x3.375 robustness (mtq_fmr): power fails in dispersed runs and no power lever is left

**Robustness (node `mc`) after iteration 8:** the Monte Carlo of `mtq_fmr` failed detumble_time (83 % of runs pass), ape_los_p9973 (0 % of runs pass), power_mean (0 % of runs pass). robustness (mtq_fmr): performance fails in dispersed runs -> fmr authority x3.375 -> x4 robustness (mtq_fmr): power fails in dispersed runs and no power lever is left

Why the loop stopped (nothing left that a knob can change):

- detumble/rcs: performance and power both fail — no authority change helps
- mtqp: authority at its lower bound (x0.5)
- nadir_pointing/cmg+mtq: power fails at the sized authority (cmg); the part's standby power is the floor
- nadir_pointing/cmg+rcs: power fails at the sized authority (cmg); the part's standby power is the floor
- nadir_pointing/fmr+mtq: performance and power both fail — no authority change helps
- nadir_pointing/fmr+mtq: power fails with the pump at lambda 3 kg/W (bound)
- nadir_pointing/fmr+rcs: power is the thrusters' valve power (RCS dumping), not the pump
- nadir_pointing/rw+mtq: power fails at the sized authority (rw); the part's standby power is the floor
- nadir_pointing/rw+rcs: power fails at the sized authority (rw); the part's standby power is the floor
- nadir_pointing/vscmg+mtq: power fails at the sized authority (vscmg); the part's standby power is the floor
- nadir_pointing/vscmg+rcs: power fails at the sized authority (vscmg); the part's standby power is the floor
- sun_acquisition/cmg: power fails at the sized authority (cmg); the part's standby power is the floor
- sun_acquisition/fmr: power fails with the pump at lambda 3 kg/W (bound)
- sun_acquisition/mtq: performance and power both fail — no authority change helps
- sun_acquisition/rw: power fails at the sized authority (rw); the part's standby power is the floor
- sun_acquisition/vscmg: power fails at the sized authority (vscmg); the part's standby power is the floor
- sun_referencing/cmg+mtq: power fails at the sized authority (cmg); the part's standby power is the floor
- sun_referencing/cmg+rcs: power fails at the sized authority (cmg); the part's standby power is the floor
- sun_referencing/fmr+mtq: performance and power both fail — no authority change helps
- sun_referencing/fmr+mtq: power fails with the pump at lambda 3 kg/W (bound)
- sun_referencing/fmr+rcs: power is the thrusters' valve power (RCS dumping), not the pump
- sun_referencing/mtq: performance and power both fail — no authority change helps
- sun_referencing/rw+mtq: power fails at the sized authority (rw); the part's standby power is the floor
- sun_referencing/rw+rcs: power fails at the sized authority (rw); the part's standby power is the floor
- sun_referencing/vscmg+mtq: power fails at the sized authority (vscmg); the part's standby power is the floor
- sun_referencing/vscmg+rcs: power fails at the sized authority (vscmg); the part's standby power is the floor

## Every configuration compared (last iteration)

Selection rule (node `select`, docs/NODES.md): least mass_kg, then power_W, then volume_L, then simplicity among feasible solution families; a single fault the family does not survive is a gap (select.fault_policy gap). The benchmarks are ranked by the same rule; best benchmark: **`mtq_rw`** (closest (not feasible)).

| family | role | rank | feasible | mass [kg] | power [W] | volume [L] | momentum actuator | gaps |
|---|---|---:|---|---:|---:|---:|---|---|
| mtq | solution | — | no | 0.215 | 1.85 | 0.010 | coils only | detumble: power_peak (power); sun_acquisition: sun_acquisition_time (performance), sun_angle_p95 (performance), power_mean (power); sun_referencing: sun_ape_p9973 (performance), power_mean (power); nadir_pointing: power_mean (power); budget: power_peak_W 1.85 > 1.5; fault: coil_fail: ape_los_p9973; fault: gyro_bias_step: ape_los_p9973 |
| mtq_fmr | solution | — | no | 11.319 | 3.80 | 0.538 | fluid loop (3 rings) | detumble: power_peak (power); sun_acquisition: power_mean (power); sun_referencing: sun_ape_p9973 (performance), power_mean (power); nadir_pointing: power_mean (power); budget: mass_kg 11.3 > 1.6; budget: power_peak_W 3.86 > 1.5 |
| mtq_fmr_rcs | solution | — | no | 11.851 | 3.85 | 1.556 | fluid loop (3 rings) | detumble: power_peak (power); sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power); budget: mass_kg 11.9 > 1.6; budget: volume_L 1.56 > 1; budget: power_peak_W 3.91 > 1.5; fault: gyro_bias_step: ape_los_p9973; fault: rcs_valve_fail: ape_los_p9973 |
| mtq_rw | benchmark | — | no | 0.560 | 4.16 | 0.098 | CAT-CUBESPACE-CUBEWHEEL-CW0057 x3 | detumble: power_peak (power); sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power); budget: power_peak_W 9.95 > 1.5 |
| mtq_rw_rcs | benchmark | — | no | 1.092 | 4.21 | 1.116 | CAT-CUBESPACE-CUBEWHEEL-CW0057 x3 | detumble: power_peak (power); sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power); budget: volume_L 1.12 > 1; budget: power_peak_W 10 > 1.5 |
| mtq_cmg | benchmark | — | no | 1.415 | 5.85 | 0.810 | CAT-TENSOR-TECH-ADCS400 x4 | detumble: power_peak (power); sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power); budget: power_peak_W 5.85 > 1.5 |
| mtq_vscmg | benchmark | — | no | 1.415 | 5.85 | 0.810 | CAT-TENSOR-TECH-ADCS400-VSCMG x4 | detumble: power_peak (power); sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power); budget: power_peak_W 5.85 > 1.5 |
| mtq_cmg_rcs | benchmark | — | no | 1.947 | 5.90 | 1.828 | CAT-TENSOR-TECH-ADCS400 x4 | detumble: power_peak (power); sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power); budget: mass_kg 1.95 > 1.6; budget: volume_L 1.83 > 1; budget: power_peak_W 5.9 > 1.5 |
| mtq_vscmg_rcs | benchmark | — | no | 1.947 | 5.90 | 1.828 | CAT-TENSOR-TECH-ADCS400-VSCMG x4 | detumble: power_peak (power); sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power); budget: mass_kg 1.95 > 1.6; budget: volume_L 1.83 > 1; budget: power_peak_W 5.9 > 1.5 |

## Selected methods (mtq_fmr)

| mode | option | feasible | objective | algorithms | failing |
|---|---|---|---:|---|---|
| detumble | mtq | no | 50.74 detumble_time | attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, sun_acquisition=sunspin_l1l2 | power_peak |
| sun_acquisition | fmr | no | 2.292 sun_acquisition_time | allocation=idmas_split, attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, pointing=pid, sun_acquisition=sunspin_l1l2 | power_mean |
| sun_referencing | mtq | no | 85.83 sun_ape_p9973 | attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_celani2015, sun_acquisition=sunspin_l1l2 | sun_ape_p9973, power_mean |
| nadir_pointing | mtq | no | 6.491 ape_los_p9973 | attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_celani2026, sun_acquisition=sunspin_l1l2 | power_mean |

## Every solution family flown as the mission (node `family_missions`)

Detumble -> Sun acquisition -> nadir with each family's best methods from the loop, C flight software (C = Rust bit for bit), and the Monte Carlo pass rate of each requirement metric.

| family | selected | feasible | mass [kg] | methods | detumble_time | ape_los_p9973 | ake_los_p9973 | power_mean | C = Rust | MC pass rates |
|---|---|---|---:|---|---:|---:|---:|---:|---|---|
| mtq | no | no | 0.215 | detumble=mtq, sun_acquisition=mtq, sun_referencing=mtq, nadir_pointing=mtq | 52.04 ✓ | 5.286 ✓ | 0.1284 ✓ | 1.652 ✗ | True | detumble_time 100 %; ape_los_p9973 17 %; ake_los_p9973 100 %; power_mean 0 % |
| mtq_fmr | yes | no | 11.319 | detumble=mtq, sun_acquisition=fmr, sun_referencing=mtq, nadir_pointing=mtq | 191.3 ✓ | 32.65 ✗ | 0.455 ✓ | 1.619 ✗ | True | detumble_time 83 %; ape_los_p9973 0 %; ake_los_p9973 100 %; power_mean 0 % |
| mtq_fmr_rcs | no | no | 11.851 | detumble=mtq, sun_acquisition=fmr, sun_referencing=fmr+rcs, nadir_pointing=fmr+rcs | — | 0.3869 ✓ | 0.1604 ✓ | 2.144 ✗ | True | detumble_time 83 %; ape_los_p9973 100 %; ake_los_p9973 100 %; power_mean 0 % |

## Coils only: every law of the literature, each at its best gains (nodes `matrix` + `tune`)

Each law's result is its worst seed at the gains that make that worst seed best (Bruni & Celani's min-max); the laws are listed by mode, best first. Sources: docs/MTQ_LITERATURE.md.

| mode | law | paper | best gains | feasible | objective (worst seed) | failing |
|---|---|---|---|---|---:|---|
| sun_acquisition | sunspin_l1l2 | P11 UPMSat-2 -> P5 He et al. 2023 | spin_rate_dps 4, ss_gain 1 | no | 76.37 sun_acquisition_time | sun_acquisition_time, sun_angle_p95, power_mean |
| sun_acquisition | sunspin_l1l2_e2 | P11 -> P5, eclipse E2 | spin_rate_dps 2, ss_gain 1 | no | 77.37 sun_acquisition_time | sun_angle_p95, power_mean, sun_acquisition_time |
| sun_acquisition | sun_boresight_celani2026 | P8 Celani 2026 | nominal | no | 79.76 sun_acquisition_time | sun_angle_p95, power_mean, sun_acquisition_time |
| sun_acquisition | sunspin_damped | P11 -> P5, R_z floor | spin_rate_dps 2, ss_gain 1 | no | 88.87 sun_acquisition_time | sun_angle_p95, power_mean, sun_acquisition_time |
| sun_acquisition | sunspin_deruiter2011 | P11 -> P2 de Ruiter 2011 | spin_rate_dps 2, ss_gain 0.3 | no | 101.9 sun_acquisition_time | sun_acquisition_time, sun_angle_p95, power_mean |
| sun_referencing | mtq_celani2015 | P4 Celani 2015 | mtq_gain_p 0.25, mtq_gain_d 1, handover_out_dps 0.25 | no | 85.83 sun_ape_p9973 | sun_ape_p9973, power_mean |
| sun_referencing | mtq_avanzini2021 | P16 Avanzini et al. 2021 | mtq_gain_p 0.25, mtq_gain_d 1, handover_out_dps 0.25 | no | 85.83 sun_ape_p9973 | sun_ape_p9973, power_mean |
| sun_referencing | mtq_celani2026 | P8 Celani 2026 | mtq_gain_p 0.25, mtq_gain_d 1, handover_out_dps 0.25 | no | 86.24 sun_ape_p9973 | sun_ape_p9973, power_mean |
| sun_referencing | mtq_tango2013 | P3 TANGO 2013 (flown) | mtq_gain_p 0.25, mtq_gain_d 1, handover_out_dps 0.25 | no | 133.3 sun_ape_p9973 | sun_ape_p9973, power_mean |
| sun_referencing | mtq_lovera2004 | P1 Lovera & Astolfi 2004 | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 0.25 | no | 138.3 sun_ape_p9973 | sun_ape_p9973, power_mean |
| sun_referencing | mtq_pd | baseline PD | nominal | no | 150.7 sun_ape_p9973 | sun_ape_p9973, power_mean |
| sun_referencing | mtq_rate_damp | baseline rate damping | nominal | no | 164.7 sun_ape_p9973 | sun_ape_p9973, power_mean |
| sun_referencing | mtq_lqr | baseline LQR | nominal | no | 173.5 sun_ape_p9973 | sun_ape_p9973, power_mean |
| sun_referencing | mtq_smc | baseline SMC | nominal | no | 177.8 sun_ape_p9973 | sun_ape_p9973, power_mean |
| nadir_pointing | mtq_celani2026 | P8 Celani 2026 | mtq_gain_p 4, mtq_gain_d 0.25, handover_out_dps 0.5 | no | 6.491 ape_los_p9973 | power_mean |
| nadir_pointing | mtq_pd | baseline PD | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 0.25 | no | 8.078 ape_los_p9973 | power_mean |
| nadir_pointing | mtq_smc | baseline SMC | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 0.25 | no | 11.71 ape_los_p9973 | power_mean, ape_los_p9973 |
| nadir_pointing | mtq_tango2013 | P3 TANGO 2013 (flown) | mtq_gain_p 0.25, mtq_gain_d 1, handover_out_dps 0.25 | no | 24.15 ape_los_p9973 | ape_los_p9973, power_mean |
| nadir_pointing | mtq_celani2015 | P4 Celani 2015 | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 0.25 | no | 26.26 ape_los_p9973 | ape_los_p9973, power_mean |
| nadir_pointing | mtq_lovera2004 | P1 Lovera & Astolfi 2004 | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 0.25 | no | 34.51 ape_los_p9973 | ape_los_p9973, power_mean |
| nadir_pointing | mtq_lqr | baseline LQR | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 1 | no | 34.71 ape_los_p9973 | ape_los_p9973, power_mean |
| nadir_pointing | mtq_avanzini2021 | P16 Avanzini et al. 2021 | mtq_gain_p 4, mtq_gain_d 1, handover_out_dps 0.5 | no | 70.19 ape_los_p9973 | ape_los_p9973, power_mean |
| nadir_pointing | mtq_rate_damp | baseline rate damping | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 1 | no | 173.7 ape_los_p9973 | ape_los_p9973, power_mean |

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
| detumble | mtq | default | no | 50.74 | power_peak (power) |
| detumble | rcs | default | no | — | detumble_time (performance), power_peak (power) |
| nadir_pointing | cmg+mtq | default | no | 0.1483 | power_mean (power) |
| nadir_pointing | cmg+rcs | default | no | 0.1482 | power_mean (power) |
| nadir_pointing | fmr+mtq | pid | no | 66.41 | ape_los_p9973 (performance), power_mean (power) |
| nadir_pointing | fmr+rcs | pid@bw4 | no | 0.3748 | power_mean (power) |
| nadir_pointing | mtq | mtq_celani2026@mtq_gain_p=4,mtq_gain_d=0.25,handover_out_dps=0.5 | no | 6.491 | power_mean (power) |
| nadir_pointing | rw+mtq | default | no | 0.1489 | power_mean (power) |
| nadir_pointing | rw+rcs | default | no | 0.1489 | power_mean (power) |
| nadir_pointing | vscmg+mtq | default | no | 0.1483 | power_mean (power) |
| nadir_pointing | vscmg+rcs | default | no | 0.1482 | power_mean (power) |
| sun_acquisition | cmg | default | no | 2.275 | power_mean (power) |
| sun_acquisition | fmr | default | no | 2.292 | power_mean (power) |
| sun_acquisition | mtq | sunspin_l1l2@spin_rate_dps=4,ss_gain=1 | no | 76.37 | sun_acquisition_time (performance), sun_angle_p95 (performance), power_mean (power) |
| sun_acquisition | rw | default | no | 2.275 | power_mean (power) |
| sun_acquisition | vscmg | default | no | 2.392 | power_mean (power) |
| sun_referencing | cmg+mtq | default | no | 0.3677 | power_mean (power) |
| sun_referencing | cmg+rcs | default | no | 0.3677 | power_mean (power) |
| sun_referencing | fmr+mtq | pid | no | 146.1 | sun_ape_p9973 (performance), power_mean (power) |
| sun_referencing | fmr+rcs | pid@bw4 | no | 1.731 | power_mean (power) |
| sun_referencing | mtq | mtq_celani2015@mtq_gain_p=0.25,mtq_gain_d=1,handover_out_dps=0.25 | no | 85.83 | sun_ape_p9973 (performance), power_mean (power) |
| sun_referencing | rw+mtq | default | no | 0.3701 | power_mean (power) |
| sun_referencing | rw+rcs | default | no | 0.3701 | power_mean (power) |
| sun_referencing | vscmg+mtq | default | no | 0.3675 | power_mean (power) |
| sun_referencing | vscmg+rcs | default | no | 0.3675 | power_mean (power) |

## Monte Carlo of the dispatched mission (12 runs; dispersions: inertia, residual_dipole, ltan_h, alt_km, cm_offset, solar_flux, kp, accommodation, reflectivity, arg_lat_deg, inertia_products, epoch_days)

| metric | req | mean ± std | [min, max] | pass rate |
|---|---:|---|---|---:|
| detumble_time (min) | 284 | 191.3 ± 0 | [191.3, 191.3] | 83 % |
| ape_los_p9973 (deg) | 10 | 155.5 ± 22.3 | [106.9, 179.7] | 0 % |
| ake_los_p9973 (deg) | 5 | 1.248 ± 0.641 | [0.3849, 2.539] | 100 % |
| power_mean (W) | 0.5 | 1.619 ± 0.0022 | [1.614, 1.622] | 0 % |
| power_peak (W) | — | 1.779 ± 0.0123 | [1.741, 1.787] | — |
| propellant (g) | — | 0 ± 0 | [0, 0] | — |

## SILS and soft OILS of the dispatched mission

The same scenario, product and blob: in-process C (SILS) and the flight software as Cortex-M4F firmware in QEMU,
C and Rust builds, with exact per-step instruction counts and the command latency applied (docs/SOFT_OILS.md).

| metric | req | SILS | soft OILS (C on M4F) | soft OILS (Rust on M4F) |
|---|---:|---:|---:|---:|
| detumble_time | 284 | 191.3 ✓ | 191.3 ✓ | 191.3 ✓ |
| ape_los_p9973 | 10 | 32.65 ✗ | 28.77 ✗ | 30.25 ✗ |
| ake_los_p9973 | 5 | 0.455 ✓ | 0.2693 ✓ | 0.2668 ✓ |
| power_mean | 0.5 | 1.619 ✗ | 1.619 ✗ | 1.619 ✗ |
| power_peak | — | 1.778  | 1.775  | 1.776  |
| propellant | — | 0  | 0  | 0  |

| OBC build | instructions/step mean / max | exec max [ms] | latency mean / max [ms] | CPU load max | overruns |
|---|---:|---:|---:|---:|---:|
| C (arm-none-eabi-gcc -O2) | 1.881e+05 / 8.961e+05 | 6.667 | 2.568 / 7.836 | 6.7 % | 0 |
| Rust (thumbv7em-none-eabihf) | 2.331e+05 / 1.047e+06 | 7.788 | 2.903 / 8.957 | 7.8 % | 0 |

Dispatch: `dist/dispatch/ais_3u/mtq_fmr/converged` (blob, sized products, BUILD.md). C = Rust flight software bitwise on the engine: True.

