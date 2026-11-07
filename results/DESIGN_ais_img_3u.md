# Design loop: ais_img_3u

Owner: Agastya. `tools/pipeline.py ais_img_3u` — sizing (Rust, adcs-design) -> SILS mode matrix (Rust engine, POP in the loop,
C flight software) -> assess -> converge, repeated until nothing is left to change; then select, dispatch, Monte Carlo and
soft OILS of the selected configuration. Flow: `docs/figures/flow_design_to_hils.svg`, method: `docs/DESIGN_LOOP.md`.

**Selected: `mtq_fmr` — Magnetorquers + fluid momentum loop — closest (not feasible)**, converged after 8 iteration(s). Knowledge class: fine. Sensors: star_tracker, magnetometer, sun_sensors, gyro, gnss, earth_sensor.

Open requirement gaps of the selected family (what the case must relax, or the next design lever):

- sun_referencing: power_mean (power)
- nadir_pointing: ape_los_p9973 (performance), rate_stability_p9973 (performance), power_mean (power)
- budget: mass_kg 8.02 > 1.85
- budget: power_peak_W 5.03 > 4

## Iterations

| iteration | knobs | options feasible | selected | changes | blocked |
|---:|---|---:|---|---|---|
| 1 | laws as written | 1/25 | mtq_fmr_rcs (closest (not feasible)) | nadir_pointing/fmr+mtq: fly every pointing algorithm<br>nadir_pointing/fmr+mtq: power -> electromagnetic pump with more copper (lambda 0.1 -> 0.3 kg/W)<br>nadir_pointing/fmr+rcs: fly every pointing algorithm<br>nadir_pointing/mtq: fly every mtq_pointing algorithm<br>nadir_pointing/rw+mtq: fly every pointing algorithm<br>nadir_pointing/rw+rcs: fly every pointing algorithm<br>sun_acquisition/mtq: fly every sun_acquisition algorithm<br>sun_referencing/fmr+rcs: fly every pointing algorithm<br>sun_referencing/mtq: fly every mtq_pointing algorithm | 17 |
| 2 | pump lambda 0.3 kg/W | 2/25 | mtq_fmr_rcs (closest (not feasible)) | nadir_pointing/fmr+mtq: power -> electromagnetic pump with more copper (lambda 0.3 -> 0.9 kg/W)<br>nadir_pointing/mtq: tune every mtq_pointing law's gains, min over the gains of the worst seed (Bruni & Celani 2017)<br>sun_acquisition/mtq: tune every sun_acquisition law's gains, min over the gains of the worst seed (Bruni & Celani 2017)<br>sun_referencing/fmr+mtq: fly every pointing algorithm<br>sun_referencing/mtq: tune every mtq_pointing law's gains, min over the gains of the worst seed (Bruni & Celani 2017) | 21 |
| 3 | pump lambda 0.9 kg/W | 2/25 | mtq_fmr_rcs (closest (not feasible)) | nadir_pointing/fmr+mtq: power -> electromagnetic pump with more copper (lambda 0.9 -> 2.7 kg/W)<br>mtqp: authority x1 -> x1.5 (performance) | 22 |
| 4 | mtqp x1.5, pump lambda 2.7 kg/W | 3/25 | mtq (closest (not feasible)) | mtqp: authority back to x1 (no improvement)<br>nadir_pointing/fmr+mtq: power -> electromagnetic pump with more copper (lambda 2.7 -> 3 kg/W) | 23 |
| 5 | mtqp x1, pump lambda 3 kg/W | 2/25 | mtq_fmr_rcs (closest (not feasible)) | — | 27 |
| 6 | mtqp x1, fmr x1.5, pump lambda 3 kg/W | 1/25 | mtq_fmr_rcs (closest (not feasible)) | sun_acquisition/fmr: fluid-loop flow sensor 2 -> 0.5 mm/s (1 sigma) | 27 |
| 7 | mtqp x1, fmr x1.5, pump lambda 3 kg/W, flow sensor 0.5 mm/s | 2/25 | mtq_fmr (closest (not feasible)) | — | 26 |
| 8 | mtqp x1, fmr x2.25, pump lambda 3 kg/W, flow sensor 0.5 mm/s | 2/25 | mtq_fmr (closest (not feasible)) | — | 26 |

**Robustness (node `mc`) after iteration 5:** the Monte Carlo of `mtq_fmr_rcs` failed ape_los_p9973 (0 % of runs pass), ake_los_p9973 (0 % of runs pass), power_mean (0 % of runs pass). robustness (mtq_fmr_rcs): performance fails in dispersed runs -> fmr authority x1 -> x1.5 robustness (mtq_fmr_rcs): power fails in dispersed runs and no power lever is left robustness (mtq_fmr_rcs): knowledge fails in dispersed runs with the star tracker fitted

**Robustness (node `mc`) after iteration 7:** the Monte Carlo of `mtq_fmr` failed ape_los_p9973 (0 % of runs pass), ake_los_p9973 (0 % of runs pass), power_mean (0 % of runs pass). robustness (mtq_fmr): performance fails in dispersed runs -> fmr authority x1.5 -> x2.25 robustness (mtq_fmr): power fails in dispersed runs and no power lever is left robustness (mtq_fmr): knowledge fails in dispersed runs with the star tracker fitted

**Robustness (node `mc`) after iteration 8:** the Monte Carlo of `mtq_fmr` failed ape_los_p9973 (0 % of runs pass), ake_los_p9973 (0 % of runs pass), power_mean (0 % of runs pass). robustness (mtq_fmr): performance fails in dispersed runs -> fmr authority x2.25 -> x3.375 robustness (mtq_fmr): power fails in dispersed runs and no power lever is left robustness (mtq_fmr): knowledge fails in dispersed runs with the star tracker fitted

**Robustness (node `mc`) after iteration 8:** the Monte Carlo of `mtq_fmr` failed ape_los_p9973 (0 % of runs pass), ake_los_p9973 (0 % of runs pass), power_mean (0 % of runs pass). robustness (mtq_fmr): performance fails in dispersed runs -> fmr authority x3.375 -> x4 robustness (mtq_fmr): power fails in dispersed runs and no power lever is left robustness (mtq_fmr): knowledge fails in dispersed runs with the star tracker fitted

Why the loop stopped (nothing left that a knob can change):

- detumble/rcs: power fails at the sized authority (rcs); the part's standby power is the floor
- mtqp: more authority did not reduce the performance violation (3.14e+03 -> 3.29e+03); kept at x1
- nadir_pointing/cmg+mtq: power fails at the sized authority (cmg); the part's standby power is the floor
- nadir_pointing/cmg+rcs: power fails at the sized authority (cmg); the part's standby power is the floor
- nadir_pointing/fmr+mtq: knowledge fails with the star tracker fitted
- nadir_pointing/fmr+mtq: performance and power both fail — no authority change helps
- nadir_pointing/fmr+mtq: power fails with the pump at lambda 3 kg/W (bound)
- nadir_pointing/fmr+rcs: knowledge fails with the star tracker fitted
- nadir_pointing/fmr+rcs: performance and power both fail — no authority change helps
- nadir_pointing/fmr+rcs: power is the thrusters' valve power (RCS dumping), not the pump
- nadir_pointing/mtq: performance and power both fail — no authority change helps
- nadir_pointing/rw+mtq: power fails at the sized authority (rw); the part's standby power is the floor
- nadir_pointing/rw+rcs: power fails at the sized authority (rw); the part's standby power is the floor
- nadir_pointing/vscmg+mtq: power fails at the sized authority (vscmg); the part's standby power is the floor
- nadir_pointing/vscmg+rcs: power fails at the sized authority (vscmg); the part's standby power is the floor
- sun_acquisition/cmg: power fails at the sized authority (cmg); the part's standby power is the floor
- sun_acquisition/rw: power fails at the sized authority (rw); the part's standby power is the floor
- sun_acquisition/vscmg: power fails at the sized authority (vscmg); the part's standby power is the floor
- sun_referencing/cmg+mtq: power fails at the sized authority (cmg); the part's standby power is the floor
- sun_referencing/cmg+rcs: power fails at the sized authority (cmg); the part's standby power is the floor
- sun_referencing/fmr+mtq: power fails with the pump at lambda 3 kg/W (bound)
- sun_referencing/fmr+rcs: power is the thrusters' valve power (RCS dumping), not the pump
- sun_referencing/rw+mtq: power fails at the sized authority (rw); the part's standby power is the floor
- sun_referencing/rw+rcs: power fails at the sized authority (rw); the part's standby power is the floor
- sun_referencing/vscmg+mtq: power fails at the sized authority (vscmg); the part's standby power is the floor
- sun_referencing/vscmg+rcs: power fails at the sized authority (vscmg); the part's standby power is the floor

## Every configuration compared (last iteration)

Selection rule (node `select`, docs/NODES.md): least mass_kg, then power_W, then volume_L, then simplicity among feasible solution families; a single fault the family does not survive is a gap (select.fault_policy gap). The benchmarks are ranked by the same rule; best benchmark: **`mtq_rw`** (closest (not feasible)).

| family | role | rank | feasible | mass [kg] | power [W] | volume [L] | momentum actuator | gaps |
|---|---|---:|---|---:|---:|---:|---|---|
| mtq | solution | — | no | 0.842 | 4.02 | 0.017 | coils only | sun_acquisition: sun_acquisition_time (performance), sun_angle_p95 (performance); sun_referencing: sun_ape_p9973 (performance); nadir_pointing: ape_los_p9973 (performance), rate_stability_p9973 (performance), power_mean (power); budget: power_peak_W 4.02 > 4; fault: gyro_bias_step: ake_los_p9973 |
| mtq_fmr | solution | — | no | 8.015 | 4.98 | 0.536 | fluid loop (3 rings) | sun_referencing: power_mean (power); nadir_pointing: ape_los_p9973 (performance), rate_stability_p9973 (performance), power_mean (power); budget: mass_kg 8.02 > 1.85; budget: power_peak_W 5.03 > 4 |
| mtq_fmr_rcs | solution | — | no | 8.481 | 5.03 | 1.378 | fluid loop (3 rings) | sun_referencing: power_mean (power); nadir_pointing: ape_los_p9973 (performance), rate_stability_p9973 (performance), power_mean (power); budget: mass_kg 8.48 > 1.85; budget: volume_L 1.38 > 1; budget: power_peak_W 5.08 > 4 |
| mtq_rw | benchmark | — | no | 1.166 | 6.12 | 0.097 | CAT-CUBESPACE-CUBEWHEEL-CW0057 x3 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power); budget: power_peak_W 11.9 > 4 |
| mtq_rw_rcs | benchmark | — | no | 1.632 | 6.17 | 0.939 | CAT-CUBESPACE-CUBEWHEEL-CW0057 x3 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power); budget: power_peak_W 12 > 4 |
| mtq_cmg | benchmark | — | no | 2.021 | 7.81 | 0.808 | CAT-TENSOR-TECH-ADCS400 x4 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power); budget: mass_kg 2.02 > 1.85; budget: power_peak_W 7.81 > 4 |
| mtq_vscmg | benchmark | — | no | 2.021 | 7.81 | 0.808 | CAT-TENSOR-TECH-ADCS400-VSCMG x4 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power); budget: mass_kg 2.02 > 1.85; budget: power_peak_W 7.81 > 4 |
| mtq_cmg_rcs | benchmark | — | no | 2.487 | 7.86 | 1.650 | CAT-TENSOR-TECH-ADCS400 x4 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power); budget: mass_kg 2.49 > 1.85; budget: volume_L 1.65 > 1; budget: power_peak_W 7.86 > 4 |
| mtq_vscmg_rcs | benchmark | — | no | 2.487 | 7.86 | 1.650 | CAT-TENSOR-TECH-ADCS400-VSCMG x4 | sun_acquisition: power_mean (power); sun_referencing: power_mean (power); nadir_pointing: power_mean (power); budget: mass_kg 2.49 > 1.85; budget: volume_L 1.65 > 1; budget: power_peak_W 7.86 > 4 |

## Selected methods (mtq_fmr)

| mode | option | feasible | objective | algorithms | failing |
|---|---|---|---:|---|---|
| detumble | mtq | yes | 56.37 detumble_time | attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, sun_acquisition=sunspin_l1l2 | — |
| sun_acquisition | fmr | yes | 2.275 sun_acquisition_time | allocation=idmas_split, attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, pointing=pid, sun_acquisition=sunspin_l1l2 | — |
| sun_referencing | fmr+mtq | no | 0.02018 sun_ape_p9973 | allocation=idmas_split, attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_pd, pointing=pid, sun_acquisition=sunspin_l1l2 | power_mean |
| nadir_pointing | mtq | no | 30.96 ape_los_p9973 | attitude=mekf, detumble=bdot_gyro, mtq_pointing=mtq_celani2026, sun_acquisition=sunspin_l1l2 | ape_los_p9973, rate_stability_p9973, power_mean |

## Every solution family flown as the mission (node `family_missions`)

Detumble -> Sun acquisition -> nadir with each family's best methods from the loop, C flight software (C = Rust bit for bit), and the Monte Carlo pass rate of each requirement metric.

| family | selected | feasible | mass [kg] | methods | detumble_time | ape_los_p9973 | ake_los_p9973 | power_mean | C = Rust | MC pass rates |
|---|---|---|---:|---|---:|---:|---:|---:|---|---|
| mtq | no | no | 0.842 | detumble=mtq, sun_acquisition=mtq, sun_referencing=mtq, nadir_pointing=mtq | 57.34 ✓ | 15.28 ✗ | 0.002629 ✓ | 2.871 ✗ | True | detumble_time 100 %; ape_los_p9973 0 %; ake_los_p9973 58 %; power_mean 0 % |
| mtq_fmr | yes | no | 8.015 | detumble=mtq, sun_acquisition=fmr, sun_referencing=fmr+mtq, nadir_pointing=mtq | 75.28 ✓ | 19.56 ✗ | 0.01119 ✗ | 2.85 ✗ | True | detumble_time 100 %; ape_los_p9973 0 %; ake_los_p9973 0 %; power_mean 0 % |
| mtq_fmr_rcs | no | no | 8.481 | detumble=mtq, sun_acquisition=fmr, sun_referencing=fmr+mtq, nadir_pointing=mtq | 75.28 ✓ | 19.56 ✗ | 0.01119 ✗ | 2.85 ✗ | True | detumble_time 100 %; ape_los_p9973 0 %; ake_los_p9973 0 %; power_mean 0 % |

## Coils only: every law of the literature, each at its best gains (nodes `matrix` + `tune`)

Each law's result is its worst seed at the gains that make that worst seed best (Bruni & Celani's min-max); the laws are listed by mode, best first. Sources: docs/MTQ_LITERATURE.md.

| mode | law | paper | best gains | feasible | objective (worst seed) | failing |
|---|---|---|---|---|---:|---|
| sun_acquisition | sunspin_deruiter2011 | P11 -> P2 de Ruiter 2011 | spin_rate_dps 2, ss_gain 1 | no | 27.82 sun_acquisition_time | sun_acquisition_time, sun_angle_p95 |
| sun_acquisition | sunspin_damped | P11 -> P5, R_z floor | nominal | no | 68.92 sun_acquisition_time | sun_angle_p95, sun_acquisition_time |
| sun_acquisition | sunspin_l1l2 | P11 UPMSat-2 -> P5 He et al. 2023 | spin_rate_dps 2, ss_gain 0.3 | no | 99.02 sun_acquisition_time | sun_acquisition_time, sun_angle_p95 |
| sun_acquisition | sunspin_l1l2_e2 | P11 -> P5, eclipse E2 | spin_rate_dps 2, ss_gain 0.3 | no | 99.02 sun_acquisition_time | sun_acquisition_time, sun_angle_p95 |
| sun_acquisition | sun_boresight_celani2026 | P8 Celani 2026 | nominal | no | 104.7 sun_acquisition_time | sun_angle_p95, sun_acquisition_time |
| sun_referencing | mtq_celani2026 | P8 Celani 2026 | mtq_gain_p 4, mtq_gain_d 0.25, handover_out_dps 0.25 | no | 19.2 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_tango2013 | P3 TANGO 2013 (flown) | mtq_gain_p 0.25, mtq_gain_d 4, handover_out_dps 0.25 | no | 52.7 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_celani2015 | P4 Celani 2015 | mtq_gain_p 0.25, mtq_gain_d 4, handover_out_dps 0.25 | no | 69.26 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_avanzini2021 | P16 Avanzini et al. 2021 | mtq_gain_p 0.25, mtq_gain_d 4, handover_out_dps 0.25 | no | 69.26 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_lovera2004 | P1 Lovera & Astolfi 2004 | mtq_gain_p 0.25, mtq_gain_d 1, handover_out_dps 0.25 | no | 131.5 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_rate_damp | baseline rate damping | nominal | no | 154 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_pd | baseline PD | nominal | no | 178.9 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_lqr | baseline LQR | nominal | no | 179.3 sun_ape_p9973 | sun_ape_p9973 |
| sun_referencing | mtq_smc | baseline SMC | nominal | no | 179.7 sun_ape_p9973 | sun_ape_p9973 |
| nadir_pointing | mtq_celani2026 | P8 Celani 2026 | mtq_gain_p 1, mtq_gain_d 0.25, handover_out_dps 1 | no | 30.96 ape_los_p9973 | ape_los_p9973, rate_stability_p9973, power_mean |
| nadir_pointing | mtq_celani2015 | P4 Celani 2015 | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 0.5 | no | 43.07 ape_los_p9973 | ape_los_p9973, ake_los_p9973, rate_stability_p9973, power_mean |
| nadir_pointing | mtq_lovera2004 | P1 Lovera & Astolfi 2004 | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 0.5 | no | 68.09 ape_los_p9973 | ape_los_p9973, ake_los_p9973, rate_stability_p9973, power_mean |
| nadir_pointing | mtq_tango2013 | P3 TANGO 2013 (flown) | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 0.5 | no | 107.8 ape_los_p9973 | ape_los_p9973, rate_stability_p9973, power_mean, ake_los_p9973 |
| nadir_pointing | mtq_rate_damp | baseline rate damping | nominal | no | 132.5 ape_los_p9973 | ape_los_p9973, ake_los_p9973, rate_stability_p9973, power_mean |
| nadir_pointing | mtq_avanzini2021 | P16 Avanzini et al. 2021 | mtq_gain_p 4, mtq_gain_d 1, handover_out_dps 0.25 | no | 144 ape_los_p9973 | ape_los_p9973, ake_los_p9973, rate_stability_p9973, power_mean |
| nadir_pointing | mtq_pd | baseline PD | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 1 | no | 158.8 ape_los_p9973 | ape_los_p9973, ake_los_p9973, rate_stability_p9973, power_mean |
| nadir_pointing | mtq_smc | baseline SMC | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 1 | no | 171.1 ape_los_p9973 | ape_los_p9973, ake_los_p9973, rate_stability_p9973, power_mean |
| nadir_pointing | mtq_lqr | baseline LQR | mtq_gain_p 0.25, mtq_gain_d 0.25, handover_out_dps 1 | no | 174.5 ape_los_p9973 | ape_los_p9973, ake_los_p9973, rate_stability_p9973, power_mean |

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
| detumble | mtq | default | yes | 56.37 | — |
| detumble | rcs | default | no | 1.875 | power_peak (power) |
| nadir_pointing | cmg+mtq | default | no | 0.004446 | power_mean (power) |
| nadir_pointing | cmg+rcs | default | no | 0.004444 | power_mean (power) |
| nadir_pointing | fmr+mtq | pid@bw2.5 | no | 0.02876 | ape_los_p9973 (performance), ake_los_p9973 (knowledge), rate_stability_p9973 (performance), power_mean (power) |
| nadir_pointing | fmr+rcs | pid@bw4 | no | 0.01867 | ape_los_p9973 (performance), ake_los_p9973 (knowledge), rate_stability_p9973 (performance), power_mean (power) |
| nadir_pointing | mtq | mtq_celani2026@mtq_gain_p=1,mtq_gain_d=0.25,handover_out_dps=1 | no | 30.96 | ape_los_p9973 (performance), rate_stability_p9973 (performance), power_mean (power) |
| nadir_pointing | rw+mtq | pid@bw2.5 | no | 0.004851 | power_mean (power) |
| nadir_pointing | rw+rcs | pid@bw2.5 | no | 0.003697 | power_mean (power) |
| nadir_pointing | vscmg+mtq | default | no | 0.004822 | power_mean (power) |
| nadir_pointing | vscmg+rcs | default | no | 0.004827 | power_mean (power) |
| sun_acquisition | cmg | default | no | 2.275 | power_mean (power) |
| sun_acquisition | fmr | default | yes | 2.275 | — |
| sun_acquisition | mtq | sunspin_deruiter2011@spin_rate_dps=2,ss_gain=1 | no | 27.82 | sun_acquisition_time (performance), sun_angle_p95 (performance) |
| sun_acquisition | rw | default | no | 2.275 | power_mean (power) |
| sun_acquisition | vscmg | default | no | 2.375 | power_mean (power) |
| sun_referencing | cmg+mtq | default | no | 0.008198 | power_mean (power) |
| sun_referencing | cmg+rcs | default | no | 0.008211 | power_mean (power) |
| sun_referencing | fmr+mtq | pid@bw4 | no | 0.02018 | power_mean (power) |
| sun_referencing | fmr+rcs | pid@bw4 | no | 0.02019 | power_mean (power) |
| sun_referencing | mtq | mtq_celani2026@mtq_gain_p=4,mtq_gain_d=0.25,handover_out_dps=0.25 | no | 19.2 | sun_ape_p9973 (performance) |
| sun_referencing | rw+mtq | default | no | 0.01089 | power_mean (power) |
| sun_referencing | rw+rcs | default | no | 0.0109 | power_mean (power) |
| sun_referencing | vscmg+mtq | default | no | 0.008192 | power_mean (power) |
| sun_referencing | vscmg+rcs | default | no | 0.008185 | power_mean (power) |

## Monte Carlo of the dispatched mission (12 runs; dispersions: inertia, residual_dipole, ltan_h, alt_km, cm_offset, solar_flux, kp, accommodation, reflectivity, arg_lat_deg, inertia_products, epoch_days)

| metric | req | mean ± std | [min, max] | pass rate |
|---|---:|---|---|---:|
| detumble_time (min) | 284 | 64.58 ± 10.7 | [40.74, 79.61] | 100 % |
| ape_los_p9973 (deg) | 0.01 | 84.61 ± 43.4 | [24.79, 154.4] | 0 % |
| ake_los_p9973 (deg) | 0.005 | 0.08326 ± 0.226 | [0.008668, 0.7987] | 0 % |
| power_mean (W) | 2 | 2.844 ± 0.0188 | [2.823, 2.882] | 0 % |
| power_peak (W) | — | 3.633 ± 0.00477 | [3.623, 3.639] | — |
| propellant (g) | — | 0 ± 0 | [0, 0] | — |

## SILS and soft OILS of the dispatched mission

The same scenario, product and blob: in-process C (SILS) and the flight software as Cortex-M4F firmware in QEMU,
C and Rust builds, with exact per-step instruction counts and the command latency applied (docs/SOFT_OILS.md).

| metric | req | SILS | soft OILS (C on M4F) | soft OILS (Rust on M4F) |
|---|---:|---:|---:|---:|
| detumble_time | 284 | 75.28 ✓ | 76.47 ✓ | 76.47 ✓ |
| ape_los_p9973 | 0.01 | 19.56 ✗ | 31.15 ✗ | 46.58 ✗ |
| ake_los_p9973 | 0.005 | 0.01119 ✗ | 0.01952 ✗ | 0.02719 ✗ |
| power_mean | 2 | 2.85 ✗ | 2.853 ✗ | 2.852 ✗ |
| power_peak | — | 3.639  | 3.643  | 3.645  |
| propellant | — | 0  | 0  | 0  |

| OBC build | instructions/step mean / max | exec max [ms] | latency mean / max [ms] | CPU load max | overruns |
|---|---:|---:|---:|---:|---:|
| C (arm-none-eabi-gcc -O2) | 3.152e+05 / 1.053e+06 | 7.833 | 3.515 / 9.002 | 7.8 % | 0 |
| Rust (thumbv7em-none-eabihf) | 4.088e+05 / 1.242e+06 | 9.238 | 4.211 / 10.407 | 9.2 % | 0 |

Dispatch: `dist/dispatch/ais_img_3u/mtq_fmr/converged` (blob, sized products, BUILD.md). C = Rust flight software bitwise on the engine: True.

