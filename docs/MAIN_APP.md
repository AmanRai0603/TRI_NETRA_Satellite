# The main app on the design database

**In one line:** the engine, the desktop app and the Python package read the cases and every input file from `design.tndb`, the same bytes as the files, so a run flown from the database is the run flown from the files; each run records one hash of its inputs and only what differs from the shipped scenario; every figure and every run's report, for the engine's runs and the twin's alike, is drawn by one plotting module as SVG or PDF; and a sweep keeps only what its figures need (`docs/RELEASE_PLAN.md` P11).

## Say it simply

The design database is the one cupboard. The engine used to take its ingredients from loose jars on the shelf; now it takes them from the cupboard, and they are the very same ingredients, so the cake comes out the same, crumb for crumb. Every cake gets a label saying which cupboard it came from and what was changed from the recipe. One camera photographs every cake, whoever baked it.

**Where the story lies:** the jars are still on the shelf (the data folder stays, for the twin and for anyone flying without a database), and the cupboard is filled from them by a tool, not by hand.

## What the design database holds for the engine

`design.tndb` (format version 2, `design/schema.toml`) holds, beside the groups and nodes:

| Table | What | Filled by |
|---|---|---|
| `design_case` | every case row by row: its line exactly as written, its fields, and the node that declares the value (`spec/plan/case_inputs.toml`) | `tools/design_inputs.py`, from `matlab_sils/cases/*.csv` |
| `engine_input` | every other file the engine reads under `data/` (scenarios, products, parts, algorithms, the catalogue, the classes), its bytes and their fingerprint (FNV-1a 64, as a run records it) | the same, from `matlab_sils/data/` |

`meta.inputs_fingerprint` is one hash over all of them. `tools/seed_design.py` and `tools/group.py merge` both fill them, so a merged design flies; a version-1 database is upgraded by `tools/tndb.py` (a copy kept beside it) and refilled at the next seed or merge.

## Flying from it

| Who | How it finds the database |
|---|---|
| `adcs` (the command line) | `TRINETRA_DESIGN=path/design.tndb adcs run nadir_hold_ais` |
| the desktop app | `TRINETRA_DESIGN`, else `design.tndb` beside its data (every kit and the Python package carry one) |
| Python | `trinetra_adcs.design.open()` (`TRINETRA_DESIGN`, else the package's own) |

With a database in use, every path under `cases/` and `data/` is read from it alone (`engine/crates/adcs-sim/src/source.rs`): a file the database does not hold is refused by name, never read from the folder. The orbit's ephemeris and gravity data, and the store, stay files.

**Why the numbers are the same:** the bytes are the files' own (a case is rebuilt from its lines under the one header, and `tools/design_inputs.py` refuses a case whose lines would not give its bytes back), so a run's case, scenario and product fingerprints, and its result id, are the same. `tests/test_design_source.py` flies `nadir_hold_ais` from the files and from the database alone, in a data folder holding no case and no input, and finds the same result id, the same metrics and the same channels, byte for byte.

## What a run records

Each run's manifest (`inputs`) now also says:

- `input_hash`: one hash over the case, the scenario, the product, the overrides, the seed and the flight software: two runs with the same hash flew the same thing;
- `differs`: only what differs from the shipped scenario (its overrides, and a seed other than 1);
- `design`: the database it flew from (its inputs fingerprint), or null for the files.

## One plotting module

`engine/crates/adcs-plot` (standard library and serde_json only) draws every figure of the project, as SVG or PDF, from one drawing model:

| Face | What |
|---|---|
| `adcs figures RUN_DIR --out DIR [--format svg\|pdf] [--full]` | a run's figures: attitude, disturbances, actuators and power, the Sun spin, and with `--full` the environment, ground track and modes; an engine run or a MATLAB twin run alike (both keep `manifest.json` and `channels.csv` with the same columns) |
| `adcs report RUN_DIR [--out DIR]` | the run's report: `report.html` (provenance, the verdict table, every figure inline, a print stylesheet) and `report.pdf` |
| `adcs plot SPEC.json --out FILE.svg\|.pdf` | any figure described as JSON (the schema is in the crate's documentation): the results page, the campaigns, the trades, the V&V report |
| the app | `GET /v1/figures`, `/v1/figure`, `/v1/report` |

`tools/report*.py` and `tools/vv_report.py` describe their figures as JSON and draw them with `adcs`; nothing in `tools/` uses matplotlib. The twin no longer draws figures when it files a run (`asils.result.save`); `asils.viz.run(rec, '', true)` still shows them on the screen at the desk.

**What is no longer committed:** `results/index.html` and `results/figures/` (210 figures), and the twin's figures and result pages (280 files). They are drawn from time series git does not keep, so they are built where the runs were flown (`python3 tools/report.py`), and one run's figures and report come on demand. The ledgers (`results/*.md`, `results/*.json`) and the V&V report stay.

## Stale runs, engine and twin

`adcs results stale` judges every stored run. An engine run names the engine's source fingerprint and its inputs; a twin run now names the twin's (`asils.util.fingerprint`): its source (`+asils` and POP's code), its case and the data it read, each as Adler-32 over the bytes with carriage returns removed, which the engine recomputes (`twin_tree_fp` in `store.rs`). `tests/test_twin_fingerprint.py` holds the twin, Python's `zlib.adler32` and the engine to one answer.

## Sweeps keep what their figures need

A campaign's `summary.json` holds every run's metrics and draws, all its figures use. After the summary, `tools/engine.py campaign` keeps run 1 (its manifest names the engine and the inputs, so `adcs results stale` judges the campaign) and removes the other runs' folders; `--keep-runs` keeps them all. The committed campaign folders went from about 4,500 files to 16.

## The Python package

```python
import trinetra_adcs.design as d
with d.open() as db:
    db.groups(); db.node("gd_0"); db.readers("gd_0")
    db.case("ais_3u")["req.ape"]         # 10.0
    db.case_rows("ais_3u")               # every row and the node that declares it
    db.scenario("nadir_hold_ais")
```

`python -m trinetra_adcs.design [groups | nodes GROUP | node ID | cases | case ID | scenarios]`. Standard-library `sqlite3`, read-only; nothing to install.

## The app's page

Built by `tools/pages.py app` from `design/pages/app.template.html` and `design/js/app_app.js` with the one component set (`tn_ui.js`, `tn.css`) into `engine/crates/trinetra-app/src/page.html`, which the program carries inside it; `tools/pages.py check` holds it to a fresh build. Tabs: **Fly** (case and scenario from the database, verdicts, figures, the report as PDF or HTML, export), **Runs** (every run kept, its input hash and what differs, its figures and report), **Design** (the groups, and a case as the database holds it, value by value with its node).

Routes: `GET /v1/design`, `GET /v1/design/case?case=C`, besides the flying and run routes listed at the head of `engine/crates/trinetra-app/src/routes.rs`.
