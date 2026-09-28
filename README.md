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
| `matlab_sils/` | the SILS (MATLAB / GNU Octave), with the Precision Orbit Propagator in the loop; start at `matlab_sils/README.md` |
| `catalogue/` | parts, products, algorithms (slot + hardware needs), `families.toml` (solution / benchmark), `modes/`, `components/` |
| `scenarios/`, `campaigns/`, `trades/` | test scenarios, Monte Carlo / edge campaigns, algorithm and hardware trades |
| `docs/` | `SOLUTION_PIPELINE.md` (start here), `ARCHITECTURE_PLAN.md`, `COMPONENTS.md`, `OILS_HILS.md`, `RESULTS.md`, `SELECTION.md`, `SOLUTIONS.md` |
| `results/index.html` | the report with every figure |
| `dist/` | the downloadable MATLAB SILS zip; `dist/dispatch/` holds the dispatch packages |
| `tools/` | `export_catalogue.py` (TOML → JSON), `run_matrix.py` (parallel runner), `report.py`, `pack_matlab.py`, `components_doc.py` |
| `spec/` | the platform architecture package (reference, unchanged) |

## Run it

```bash
python3 tools/export_catalogue.py                           # after editing any TOML
python3 tools/run_matrix.py --workers 4                     # scenarios, campaigns, trades
python3 tools/run_matrix.py --workers 4 --only solutions    # the customer-case solution matrix
python3 tools/report.py && python3 tools/pack_matlab.py
```

In MATLAB or Octave: `cd matlab_sils; startup_asils; addpath tests; run_all_tests`, then the examples in
`matlab_sils/examples/`.
