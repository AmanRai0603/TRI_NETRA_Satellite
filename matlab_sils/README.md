# TRI-NETRA ADCS SILS — MATLAB (asils)

**Owner: Agastya.** Copyright (c) 2026 Agastya. All rights reserved.

A complete, closed-loop software-in-the-loop simulation of the TRI-NETRA 3U
ADCS. The truth orbit is the **Precision Orbit Propagator (POP v51)** stepped
*inside* the attitude loop (no pre-computed or recorded orbit data), and every
environment quantity — magnetic field, Sun, eclipse, density, atmosphere-relative
velocity, SRP pressure — flows from it into the disturbance torques, the sensors
and the flight software.

Runs in **MATLAB** (base MATLAB, no toolboxes; `parfor` used if the Parallel
Computing Toolbox is present) and **GNU Octave ≥ 8**.

## Quick start

```matlab
startup_asils                                              % once per session
rec = asils.run('nadir_hold_ais', 'cases/ais_3u.csv');     % AIS 3U, 10 deg, dawn-dusk SSO
asils.viz.run(rec, '', true)                               % figures
asils.result.save(rec)                                     % store/results/nadir_hold_ais/

rec = asils.run('fine_hold_img', 'cases/ais_img_3u.csv');  % imaging 3U, 0.01 deg, SSO 10:00
addpath tools; run_campaign('mc_slew_img')                 % a Monte Carlo
addpath tests; run_all_tests                               % the test suite
```

## Your input is a case CSV

`cases/ais_3u.csv` and `cases/ais_img_3u.csv` (format `adcs-case/1`, every key
explained in `cases/case_template.csv`). Change the orbit, mass properties,
surfaces, residual dipole or requirements there. Anything else can be changed
per run without editing code:

```matlab
rec = asils.run('nadir_hold_ais', 'cases/ais_3u.csv', 'seed', 7, ...
      'set', struct('env__F107', 220, 'sc__mass_kg', 4.3, 'sim__duration_s', 6000));
```

## Actuator families (products in `data/products`)

| family | product | what it flies |
|---|---|---|
| coils only | `TRN-P-3U-AIS`, `TRN-P-3U-AIS-CSS` | B-dot detumble, magnetic nadir pointing (fine or coarse cosine sun sensors) |
| coils + reaction wheels | `TRN-P-3U-IMG` | fine pointing with PID, LQR or sliding-mode control; magnetic dumping |
| coils + fluid momentum rings | `TRN-P-3U-FMR` | IDMAS split: coils take the torque across the field, galinstan rings the rest |
| coils + fluid rings + RCS | `TRN-P-3U-FMR-RCS` | rings for fine pointing, cold-gas thrusters for agile slews |
| coils + wheels + RCS | `TRN-P-3U-RW-RCS` | wheels for fine pointing, thrusters assist slews and unload the wheels |
| coils + 4 SGCMG | `TRN-P-3U-CMG` | singularity-robust steering, agile slews |
| coils + 4 VSCMG | `TRN-P-3U-VSCMG` | gimbal + wheel-mode torque through singularities |

Every imaging product carries two star-tracker heads (star-field model solved by
QUEST), a precision MEMS gyro, magnetometer, sun sensors and GNSS.

## Scenarios (`data/scenarios`, 39) and campaigns (`data/campaigns`, 8)

- **AIS (coils only):** `detumble_ais` (+ `_mag`, `_bangbang` B-dot laws), `nadir_hold_ais`,
  `nadir_hold_ais_css`, `mission_ais` (tumble → detumble → nadir), faults `fault_coil_ais`, `fault_gyro_ais`.
- **Imaging, per family** (`img` = wheels, `fmr`, `fmr_rcs`, `rw_rcs`, `cmg`, `vscmg`):
  `fine_hold_<f>`, `slew_<f>` (30° in 60 s), `agile_slew_<f>` (90° pitch in 15 s), `mission_<f>`
  (tumble → detumble → fine hold); controller comparison `fine_hold_img_lqr/_smc`, `slew_img_lqr/_smc`;
  faults `fault_wheel_img`, `fault_st_img`, `fault_gimbal_cmg`.
- **Campaigns:** Monte Carlo `mc_detumble_ais`, `mc_nadir_ais`, `mc_fine_img`, `mc_slew_img`,
  `mc_slew_cmg`, `mc_agile_rw_rcs`; edge cases (each dispersion at its bounds, then all adverse)
  `edge_nadir_ais`, `edge_fine_img`.

Run everything: `python3 ../tools/run_matrix.py --workers 4` (Octave), or `run_scenarios` +
`run_campaign` in MATLAB.

## OILS / HILS

Sensors and actuators cross a hardware-abstraction boundary (`+asils/+hal`, the MATLAB side of
`adcs_hal.h`) as register values in every run. Switch the backend to `loopback` (byte frames) or
`udp` (flight OBC / rig in the loop) with real-time pacing; `asils.hal.stimulus(rec)` gives the
Helmholtz-cage field, Sun-simulator direction and air-bearing rate for a HILS replay. See
`../docs/OILS_HILS.md`.

## Layout

| folder | contents |
|---|---|
| `+asils/` | the SILS: `+orbit` (in-loop POP), `+env`, `+plant`, `+devices`, `+fsw`, `+hal`, `+faults`, `+metrics`, `+campaign`, `+rec`, `+viz`, `+result`, `run.m`, `config.m` |
| `pop/` | Precision Orbit Propagator v51 (vendored) |
| `cases/`, `data/` | the case CSVs; exported parts, products, scenarios, campaigns (JSON) |
| `examples/`, `tests/`, `tools/` | worked examples, the test suite (17 tests), batch drivers |
| `store/` | your results, filed per scenario |

Architecture, node by node: `../docs/ARCHITECTURE_PLAN.md`. Results: `../docs/RESULTS.md`.
