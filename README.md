# TRI_NETRA_Satellite
Complete Satellite ADCS Design and Simulation (MBSE/SILS/OILS/HILS) Codebase

**Owner: Agastya.** Copyright (c) 2026 Agastya. All rights reserved. See `NOTICE.md`.

## What it does

A customer gives one **case**: orbit, mass properties, surfaces and pointing requirements
(`adcs-case/1`, `matlab_sils/cases/*.csv`). The codebase answers two questions: which of our three
ADCS solutions flies that satellite, and how does it compare with standard actuators on every
parameter?

```
case  ->  demand  ->  sizing  ->  mission modes x methods (SILS)  ->  solution  ->  dispatch -> OILS -> HILS
```

- **Our solutions:** magnetorquers only · + fluid momentum loop · + fluid loop + N2O cold-gas RCS.
- **Benchmarks** (sized to the same case, compared only): reaction wheels, CMG, VSCMG, each also with RCS.
- **Mission modes:** detumble · Sun acquisition · Sun referencing · nadir pointing. Modes are data,
  so more can be added.

## Where things are

| path | what |
|---|---|
| `fsw/` | the flight software in **embedded C** (C99) behind `adcs_fsw.h` / `adcs_hal.h`; `fsw/pseudocode/` is the contract, `fsw/params/params.toml` the parameter set |
| `fsw-rs/` | the same flight software in **Rust** (`no_std`; feature `cabi` exports the C ABI) |
| `engine/` | the **Rust SILS engine**: `adcs-pop` (the full POP propagator ported to Rust, bit-identical to MATLAB), plant, device emulators speaking bytes, config, metrics, recorder; `adcs` CLI |
| `fsw/targets/` | the **virtual OBC**: the flight software as a process or as Cortex-M4F firmware in QEMU, in lockstep with the engine over adcs-link/1 (`docs/VIRTUAL_OBC.md`); the same link reaches a real OBC |
| `matlab_sils/` | the SILS twin (MATLAB / GNU Octave), with the Precision Orbit Propagator in the loop; start at `matlab_sils/README.md` |
| `catalogue/` | parts, products, algorithms (slot + hardware needs), `families.toml` (solution / benchmark), `modes/`, `components/` |
| `scenarios/`, `campaigns/`, `trades/` | test scenarios, Monte Carlo / edge campaigns, algorithm and hardware trades |
| `docs/` | `SOLUTION_PIPELINE.md` (start here), `LANGUAGES.md` (C / Rust / Python / MATLAB roles), `VIRTUAL_OBC.md`, `ARCHITECTURE_PLAN.md`, `COMPONENTS.md`, `OILS_HILS.md`, `RESULTS.md`, `SELECTION.md`, `SOLUTIONS.md` |
| `results/index.html` | the report with every figure |
| `dist/` | the downloadable MATLAB SILS zip and the flight-software + Rust-engine zip; `dist/dispatch/` holds the dispatch packages |
| `tools/` | `engine.py` (build, run, Monte Carlo and parity on the Rust engine), `gen_fsw_params.py` (params → C + Rust), `export_catalogue.py` (TOML → JSON), `run_matrix.py` (parallel runner), `report.py`, `pack_matlab.py`, `pack_flight.py` (flight software + engine zip), `components_doc.py` |
| `spec/` | the platform architecture package (reference, unchanged) |

## Run it

```bash
python3 tools/engine.py build && python3 tools/engine.py run   # flight software (C + Rust) and the Rust engine
python3 tools/engine.py fsw-parity && python3 tools/engine.py twin-parity
python3 tools/export_catalogue.py                           # after editing any TOML
python3 tools/run_matrix.py --workers 4                     # scenarios, campaigns, trades
python3 tools/run_matrix.py --workers 4 --only solutions    # the customer-case solution matrix
python3 tools/report.py && python3 tools/pack_matlab.py
```

In MATLAB or Octave: `cd matlab_sils; startup_asils; addpath tests; run_all_tests`, then the examples in
`matlab_sils/examples/`.
