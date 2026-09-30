# TRI_NETRA_Satellite
Complete Satellite ADCS Design and Simulation (MBSE/SILS/OILS/HILS) Codebase

**Owner: Agastya.** Copyright (c) 2026 Agastya. All rights reserved. See `NOTICE.md`.

## Use it without building anything

Download the app for your system (Windows, macOS, Linux) or the one Python package from the
[latest release](https://github.com/AmanRai0603/TRI_NETRA_Satellite/releases/latest), open
**TRI-NETRA ADCS**, pick a case and a scenario, and read the verdicts. `docs/START_HERE.md`
walks through it; `docs/FIRST_RUN.md` says what an unsigned program's first start asks.

    python -m pip install trinetra_adcs-<version>-py3-none-any.whl
    trinetra-adcs-app          # the desktop app
    trinetra-adcs help         # the engine's command line (adcs)

## What it does

A customer gives one **case**: orbit, mass properties, surfaces and pointing requirements
(`adcs-case/1`, `matlab_sils/cases/*.csv`). The codebase answers two questions: which of our three
ADCS solutions flies that satellite, and how does it compare with standard actuators on every
parameter?

![From customer case to flight](docs/figures/flow_design_to_hils.svg)

```
case -> demand -> sizing -> SILS mode matrix -> assess -> converge (resize / upgrade, repeat) -> select
     -> dispatch -> Monte Carlo + edge campaigns -> soft OILS (flight software on a virtual Cortex-M4F) -> V&V report
     -> OILS (real OBC) -> HILS
```

The whole loop runs with one command, `python3 tools/pipeline.py` (`docs/DESIGN_LOOP.md`), and the
downloadable V&V report (`dist/TRINETRA_ADCS_VV_report.pdf`) is written by `python3 tools/vv_report.py`.
Who runs what (C / Rust / Python / MATLAB): `docs/figures/architecture_languages.svg`, `docs/LANGUAGES.md`.

- **Our solutions:** magnetorquers only · + fluid momentum loop · + fluid loop + N2O cold-gas RCS.
- **Benchmarks** (sized to the same case, compared only): reaction wheels, CMG, VSCMG, each also with RCS.
- **Mission modes:** detumble · Sun acquisition · Sun referencing · nadir pointing. Modes are data,
  so more can be added.

## Where things are

| path | what |
|---|---|
| `fsw/` | the flight software in **embedded C** (C99) behind `adcs_fsw.h` / `adcs_hal.h`; `fsw/pseudocode/` is the contract, `fsw/params/params.toml` the parameter set |
| `fsw-rs/` | the same flight software in **Rust** (`no_std`; feature `cabi` exports the C ABI) |
| `engine/` | the **Rust SILS engine**: `adcs-design` (demand survey + sizing, identical to the MATLAB laws), `adcs-pop` (the full POP propagator ported to Rust, bit-identical to MATLAB), plant, device emulators speaking bytes, config, metrics, recorder; `adcs` CLI |
| `fsw/targets/` | the **virtual OBC**: the flight software as a process or as Cortex-M4F firmware in QEMU, in lockstep with the engine over adcs-link/1 (`docs/VIRTUAL_OBC.md`); the same link reaches a real OBC |
| `matlab_sils/` | the SILS twin (MATLAB / GNU Octave), with the Precision Orbit Propagator in the loop; start at `matlab_sils/README.md` |
| `catalogue/` | parts, products, algorithms (slot + hardware needs), `families.toml` (solution / benchmark), `modes/`, `components/` |
| `scenarios/`, `campaigns/`, `trades/` | test scenarios, Monte Carlo / edge campaigns, algorithm and hardware trades |
| `docs/` | `DESIGN_LOOP.md` (start here), `NODES.md` (every node: inputs, outputs, parameters, rules), `MTQ_LITERATURE.md` (the sixteen magnetorquer-only papers in the flight software), `NAV_GUIDANCE_AUDIT.md` (navigation and guidance assumptions audited and corrected), `CATALOGUE.md` (bought wheels and CMGs from datasheets), `SOFT_OILS.md`, `figures/`, `SOLUTION_PIPELINE.md`, `LANGUAGES.md` (C / Rust / Python / MATLAB roles), `VIRTUAL_OBC.md`, `ARCHITECTURE_PLAN.md`, `COMPONENTS.md`, `OILS_HILS.md`, `RESULTS.md`, `SELECTION.md`, `SOLUTIONS.md` |
| `results/index.html` | the report with every figure |
| `dist/` | the downloadable MATLAB SILS zip and the flight-software + Rust-engine zip; `dist/dispatch/` holds the dispatch packages |
| `tools/` | `pipeline.py` (the design loop, node by node), `engine.py` (build, run, Monte Carlo, campaigns, soft OILS and parity on the Rust engine), `vv_report.py` + `templates/vv_report.html` (the V&V report, HTML + PDF), `figures/` (the two diagrams), `gen_fsw_params.py` (params → C + Rust), `export_catalogue.py` (TOML → JSON), `run_matrix.py` (parallel runner), `report.py`, `pack_matlab.py`, `pack_flight.py` (flight software + engine zip), `components_doc.py` |
| `engine/crates/trinetra-app/` | the desktop app: a local page to pick a case and a scenario, fly it, keep and export runs |
| `python/trinetra_adcs/` | the Python package's front end: it runs the engine and the app it carries for the computer it is on |
| `tests/`, `tools/check_all.py` | the tools' tests, and every check in one command (`python3 tools/check_all.py`; CI runs it) |
| `docs/commands.toml`, `docs/COMMANDS.md` | every command: what it does, its steps, what it reads and writes (`python3 tools/trinetra.py explain <command>`) |
| `docs/CHANGING.md` | for each kind of change: what to edit, what to regenerate, which check proves it |
| `tools/kit.py`, `tools/macapp.py`, `tools/build_wheel.py` | the release downloads: the kits, the macOS app, the Python package (`docs/RELEASE_SETUP.md`) |
| `spec/` | the ADCS platform's build specification: a standalone package, with its own tools and checks |

## Run it

Before pushing, `python3 tools/check_all.py` runs every check. `--dry-run` on `engine.py` and
`pipeline.py` says what a command will do before it does it.

```bash
python3 tools/engine.py build && python3 tools/engine.py run   # flight software (C + Rust) and the Rust engine
python3 tools/engine.py fsw-parity && python3 tools/engine.py twin-parity
python3 tools/pipeline.py                                    # design loop: size -> SILS -> converge -> select -> dispatch -> MC -> soft OILS
python3 tools/engine.py campaign && python3 tools/engine.py oils   # every MC / edge campaign vs MATLAB; SILS + soft OILS, every scenario
python3 tools/vv_report.py                                   # results/vv/*.html + dist/TRINETRA_ADCS_VV_report.pdf
python3 tools/export_catalogue.py                           # after editing any TOML
python3 tools/run_matrix.py --workers 4                     # scenarios, campaigns, trades
python3 tools/run_matrix.py --workers 4 --only solutions    # the customer-case solution matrix
python3 tools/report.py && python3 tools/pack_matlab.py
```

In MATLAB or Octave: `cd matlab_sils; startup_asils; addpath tests; run_all_tests`, then the examples in
`matlab_sils/examples/`.
