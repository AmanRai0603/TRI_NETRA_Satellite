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

## Scenarios (`data/scenarios`) and campaigns (`data/campaigns`)

| id | case | what |
|---|---|---|
| `detumble_ais` | AIS | B-dot detumble from 10 deg/s, 3 orbits |
| `nadir_hold_ais` | AIS | magnetic-only nadir hold from 30 deg, 3 orbits, APE/AKE last orbit |
| `mission_ais` | AIS | tumble → B-dot → automatic switch to nadir hold, 5 orbits |
| `detumble_img` | imaging | B-dot detumble from 10 deg/s, 3 orbits |
| `fine_hold_img` | imaging | wheel + star-tracker fine nadir hold, 1 orbit, 3-sigma APE/AKE/RKS |
| `slew_img` | imaging | 30 deg target slew in 60 s and settle |
| `mission_img` | imaging | tumble → B-dot → wheel nadir hold, 3 orbits |
| `mc_detumble_ais`, `mc_nadir_ais`, `mc_fine_img`, `mc_slew_img` | | Monte Carlo campaigns |

## Layout

| folder | contents |
|---|---|
| `+asils/` | the SILS: `+orbit` (in-loop POP), `+env`, `+plant`, `+devices`, `+fsw`, `+metrics`, `+campaign`, `+rec`, `+viz`, `+result`, `run.m`, `config.m` |
| `pop/` | Precision Orbit Propagator v51 (vendored) |
| `cases/`, `data/` | the case CSVs; exported parts, products, scenarios, campaigns (JSON) |
| `examples/`, `tests/`, `tools/` | six worked examples, the test suite, batch drivers |
| `store/` | your results, filed per scenario |

Architecture, node by node: `../docs/ARCHITECTURE_PLAN.md`. Results: `../docs/RESULTS.md`.
