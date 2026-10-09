# TRI-NETRA ADCS SILS — MATLAB (asils)

**Owner: Agastya.** Copyright (c) 2026 Agastya. All rights reserved.

A complete, closed-loop software-in-the-loop simulation of the TRI-NETRA 3U
ADCS. The truth orbit is the **Precision Orbit Propagator (POP v51)** stepped
*inside* the attitude loop (no pre-computed or recorded orbit data), and every
environment quantity — magnetic field, Sun, eclipse, density, atmosphere-relative
velocity, SRP pressure — flows from it into the disturbance torques, the sensors
and the flight software.

**The twin flies the design's code.** Every model (the plant, the environment, the units and their chains, the
emulators' codecs, the metrics, the sizing laws) is the design's, generated into `+asils/+models` (and the relations
into `+asils/+relations`) by `../tools/engine_build.py`; the flight software's algorithms are `../fsw/pseudocode`,
generated into `+asils/+alg` by `../tools/flight_build.py`, behind a tick (`asils.fsw.step`, the engine's flight
software's `fsw.rs` step in MATLAB) loaded with the very adcs-fswcfg/1 blob the engine builds (`adcs params`, decoded
by the generated `asils.fsw.params_decode`). What is written by hand is the runner: the time stepping, the bus and
the hardware-abstraction plumbing, the recording, the set-up's reading of the case, scenario and product, the
campaigns and trades, the plots and the files. A run's noise and dispersions draw from the language's SplitMix64
streams (`asils.pc`), the engine's, so the twin and the engine fly the same run value for value.

Runs in **MATLAB** (base MATLAB, no toolboxes; `parfor` used if the Parallel
Computing Toolbox is present) and **GNU Octave ≥ 8**.

## Quick start

```matlab
startup_asils                                              % once per session
rec = asils.run('nadir_hold_ais', 'cases/ais_3u.csv');     % AIS 3U, 10 deg, dawn-dusk SSO
asils.viz.run(rec, '', true)                               % figures
asils.result.save(rec)                                     % store/results/nadir_hold_ais/
% its figures and report, drawn as the engine's runs are: adcs figures store/results/nadir_hold_ais --out figs

rec = asils.run('fine_hold_img', 'cases/ais_img_3u.csv');  % imaging 3U, 0.01 deg, SSO 10:00
addpath tools; run_campaign('mc_slew_img')                 % a Monte Carlo
addpath tests; run_all_tests                               % the test suite
```

## Open a design and fly it (`+trinetra`)

The twin opens a design database (`.tndb`) itself, through the library's own command (`tndb`, bundled with the
application: JSON over `system()`, no toolbox, MATLAB and GNU Octave ≥ 8), generates every function it flies from it,
and files each run beside your copy of the design. No Python at run time.

```matlab
startup_asils                                         % once per session
tn = trinetra.open('work/design.tndb')                % shows the design's health
trinetra.build(tn)                                    % work/generated/: every generated function, read-only, on the path
rec = trinetra.run(tn, 'detumble_ais')                % work/results/detumble_ais/: channels, manifest, rec.mat, figures, report
res = trinetra.campaign(tn, 'mc_nadir_ais')           % work/results/mc_nadir_ais/
trinetra.which('aero_torque')                         % the node (and its release and revision) a function came from
```

- **`trinetra.open(file)`** reads the design (`tndb health`, `tndb read`): its version and toolbox, its nodes by
  behaviour and the built-in count, its groups' releases and signatures, its inputs (each held to its fingerprint),
  and whether its generated code is built and is this design's. `tn.inputs` holds the twin's inputs by path
  (`data/...`, `cases/<id>.csv`): the bytes `tools/from_design.py` exports to `data/` and `cases/` here.
- **`trinetra.build(tn)`** runs `tndb build-matlab`: the design's models (`+asils/+models`), relations
  (`+asils/+relations`, with the groups' wiring `design/groups` when the twin sits in the repository), flight
  algorithms (`+asils/+alg`) and the language's runtime (`+asils/+pc`), byte for byte what `tools/engine_build.py`
  and `tools/flight_build.py` write here, into `generated/` beside the design, read-only, with `generated/index.json`
  naming each file's node, release and revision (`trinetra.which`). It goes on the path ahead of this folder's copies.
- **`trinetra.run(tn, scenario, ...)`** and **`trinetra.campaign(tn, id, ...)`** fly the runner here (`asils.run`,
  `asils.campaign.run`) on the design's inputs and generated code (`trinetra.use`): the case, scenario, product, parts,
  algorithms, catalogue and stated values are the design's (`asils.util.design`), never `data/`'s, and the engine
  builds the flight software's blob from the same design (`TRINETRA_DESIGN=… adcs params`). The run is filed in
  `results/<scenario>/` (its manifest names the design), with its figures and report drawn by the engine's plotting
  (`adcs figures`, `adcs report`). Options as `asils.run` (`'seed'`, `'set'`, `'quiet'`, `'checkpoint'`) and
  `'case'`, `'save'`, `'figures'`, `'report'`, `'out'`.
- **`c = trinetra.use(tn)`** makes any of the twin's own functions run on the design until `clear c`
  (`asils.config`, `asils.sizing.size_all`, ...).
- The programs are found as `$TNDB_BIN` / `$ADCS_BIN`, else `bin/` here (an install's), else the repository's
  `engine/target/release/` (`cargo build --release -p trinetra-design -p adcs-cli` in `engine/`).

Until the switch, the committed path stays as it was: `asils.run('nadir_hold_ais', 'cases/ais_3u.csv')` flies the
copies committed here (`data/`, `cases/`, `+asils/+models`, `+relations`, `+alg`, `+pc`). The two give the same run bit
for bit on the regression copy (`tests/test_trinetra_open.m`, in the suite).

## Your input is a case CSV

`cases/ais_3u.csv` and `cases/ais_img_3u.csv` (format `adcs-case/1`, every key
explained in `cases/case_template.csv`). Change the orbit, mass properties,
surfaces, residual dipole or requirements there. Anything else can be changed
per run without editing code:

```matlab
rec = asils.run('nadir_hold_ais', 'cases/ais_3u.csv', 'seed', 7, ...
      'set', struct('engine__f107', 220, 'engine__mass_kg', 4.3, 'engine__duration_s', 6000));
```

## Actuator families (products in `data/products`)

| family | product | what it flies |
|---|---|---|
| coils only | `TRN-P-3U-AIS`, `TRN-P-3U-AIS-CSS` | B-dot detumble (4 laws), magnetic nadir pointing (4 laws), Standard Code Sun-pointing spin (L1 spin-up + L2 He et al.) |
| coils + reaction wheels | `TRN-P-3U-IMG` | fine pointing with PID, LQR or sliding-mode control; magnetic dumping |
| coils + fluid momentum rings | `TRN-P-3U-FMR` | IDMAS split: coils take the torque across the field, galinstan rings the rest |
| coils + fluid rings + RCS | `TRN-P-3U-FMR-RCS` | rings for fine pointing, cold-gas thrusters for agile slews |
| coils + wheels + RCS | `TRN-P-3U-RW-RCS` | wheels for fine pointing, thrusters assist slews and unload the wheels |
| coils + 4 SGCMG | `TRN-P-3U-CMG` | singularity-robust steering, agile slews |
| coils + 4 VSCMG | `TRN-P-3U-VSCMG` | gimbal + wheel-mode torque through singularities |

Every imaging product carries two star-tracker heads (star-field model solved by
QUEST), a precision MEMS gyro, magnetometer, sun sensors and GNSS.

## From a customer's case to a solution

```matlab
Z   = asils.sizing.size_all('ais_3u');          % demand survey + every actuator sized to the case
asils.solution.run('ais_3u');                   % every mission mode x option x seed (~50 runs a case)
Sol = asils.solution.collect('ais_3u');         % per-family verdicts, recommended solution
asils.solution.dispatch('ais_3u');              % ../dist/dispatch/ais_3u/<family>/ for OILS / the OBC
```

- **Mission modes** (`data/modes`, from `../catalogue/modes`): `detumble`, `sun_acquisition`,
  `sun_referencing`, `nadir_pointing`. Each mode lists its options: which actuator does the job and
  which dumps momentum. Every option is flown on its case-sized product from the same start and seeds.
- **Families** (`data/families.json`): our solutions `mtq`, `mtq_fmr`, `mtq_fmr_rcs`; the benchmarks
  `mtq_rw`, `mtq_cmg` and `mtq_vscmg`, each also with RCS. The simplest solution that passes every
  mode is recommended; the benchmarks are only compared.
- **Sizing** (`+asils/+sizing`, the engine's `adcs size` in MATLAB): the disturbance survey on the POP
  orbit over the design's sweep of seasons and solar activity, then the design's laws (sizedemand,
  sizemtq, sizefmr with its pump, sizercs, sizebudget, sizesensors) and the catalogue's lightest
  wheel, CMG or VSCMG that meets the need (sizerotor). The convergence loop's knobs:
  `asils.sizing.knobs`.
- **Unit chains**: each unit's own processing chain (the star tracker's render → centroid → identify →
  QUEST, the Sun sensor's quadrant currents → angles, the fluid loop's flow servo) is the design's,
  flown by `asils.devices.sense` / `actuate` as the part's level asks. See `../docs/COMPONENTS.md`.

## Algorithms: one job, several algorithms, several hardware sets

Every algorithm is registered once in `data/algorithms/<id>.json` (source `../catalogue/algorithms`)
with its **slot** (the job it does) and what hardware it **needs**:

| slot | algorithms |
|---|---|
| `detumble` | `bdot_gyro`, `bdot_mag`, `bdot_bangbang`, `genbdot_l1` (Standard Code L1) |
| `mtq_pointing` (coils) | `mtq_pd`, `mtq_lqr`, `mtq_smc`, `mtq_rate_damp` |
| `sun_acquisition` (coils; alias `sun_spin`) | `sunspin_l1l2` (Standard Code spin-up L1 + He et al. L2) |
| `pointing` (momentum devices) | `pid`, `lqr`, `smc` |
| `allocation` | `rotor_pinv`, `idmas_split`, `cmg_sr`, `vscmg_sr` |
| `thrusters`, `attitude` | `rcs_pwm`, `mekf` |

`asils.fsw.select` resolves each slot once per run: the scenario's `[fsw] algorithms = {slot = id}`,
else the product's `[selected]` table, else the first compatible default. A choice the hardware
cannot fly is refused by name. Override one for a single run:
`asils.run('nadir_hold_ais', 'cases/ais_3u.csv', 'set', struct('fsw__algorithms__mtq_pointing', 'mtq_smc'))`.

**Trades** (`data/trades`, 12) fly every candidate — an algorithm for a slot, another product, or a
tuning — on the same seeds, rank those meeting every requirement by their worst objective value, and
propose a winner: `run_trade({'trade_mtq_pointing_ais'})` → `store/trades/<id>/trade.json`, `trade.png`.
Proposals are summarised in `../docs/SELECTION.md`; a person confirms them into the product's `[selected]`.

## Scenarios (`data/scenarios`, 34), campaigns (`data/campaigns`, 8), trades (`data/trades`, 12)

- **AIS (coils only):** `detumble_ais`, `nadir_hold_ais`, `nadir_hold_ais_css`, `sun_spin_ais`
  (detumble → spin-up → Sun-pointing spin, no attitude solution), `mission_ais` (tumble → detumble → nadir),
  faults `fault_coil_ais`, `fault_gyro_ais`.
- **Imaging, per family** (`img` = wheels, `fmr`, `fmr_rcs`, `rw_rcs`, `cmg`, `vscmg`):
  `fine_hold_<f>`, `slew_<f>` (30° in 60 s), `agile_slew_<f>` (90° pitch in 15 s), `mission_<f>`
  (tumble → detumble → fine hold); faults `fault_wheel_img`, `fault_st_img`, `fault_gimbal_cmg`.
- **Campaigns:** Monte Carlo `mc_detumble_ais`, `mc_nadir_ais`, `mc_fine_img`, `mc_slew_img`,
  `mc_slew_cmg`, `mc_agile_rw_rcs`; edge cases (each dispersion at its bounds, then all adverse)
  `edge_nadir_ais`, `edge_fine_img`.

- **Trades:** detumble law, magnetic pointing law, Sun-spin rate and Sun sensing (AIS); fine-pointing law
  per actuator family (RW, CMG, VSCMG, fluid rings), slew law, fluid-ring allocation; actuator family for
  the fine hold and for the agile slew.

Run everything: `python3 ../tools/run_matrix.py --workers 4` (Octave), or `run_scenarios`,
`run_campaign` and `run_trade` in MATLAB.

## OILS / HILS

Sensors and actuators cross a hardware-abstraction boundary (`+asils/+hal`, the MATLAB side of
`adcs_hal.h`) as register values in every run. Switch the backend to `loopback` (byte frames) or
`udp` (flight OBC / rig in the loop) with real-time pacing; `asils.hal.stimulus(rec)` gives the
Helmholtz-cage field, Sun-simulator direction and air-bearing rate for a HILS replay. See
`../docs/OILS_HILS.md`.

## Layout

| folder | contents |
|---|---|
| `+asils/` | the SILS: generated `+models` (the design's models), `+relations`, `+alg` (the flight software's algorithms), `+pc` (the language's runtime); by hand `+orbit` (in-loop POP), `+env`, `+plant`, `+devices` (the units' states, stepped through `+models`), `+fsw` (the tick, the blob), `+hal`, `+faults`, `+metrics`, `+sizing`, `+solution`, `+campaign`, `+trade`, `+rec`, `+viz`, `+result`, `run.m`, `config.m` |
| `pop/` | Precision Orbit Propagator v51 (vendored) |
| `cases/`, `data/` | the case CSVs; exported parts, products, algorithms, modes, families, components, scenarios, campaigns, trades (JSON) |
| `+trinetra/` | open a design database, build its code, fly it (`open`, `build`, `run`, `campaign`, `which`, `use`, `health`) |
| `examples/`, `tests/`, `tools/` | worked examples, the test suite (44 tests), batch drivers |
| `store/` | your results, filed per scenario |

Architecture, node by node: `../docs/ARCHITECTURE_PLAN.md`. Results: `../docs/RESULTS.md`. Selection: `../docs/SELECTION.md`. Solutions: `../docs/SOLUTION_PIPELINE.md`, `../docs/SOLUTIONS.md`.
