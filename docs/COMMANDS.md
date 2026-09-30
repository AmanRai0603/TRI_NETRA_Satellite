# Commands

> Generated from `docs/commands.toml` by `python3 tools/trinetra.py docs`; never edited by hand.
> `python3 tools/trinetra.py explain <command>` prints one of these; `--dry-run` on `engine.py`
> and `pipeline.py` prints it and runs nothing.

| command | what it does |
|---|---|
| [`adcs run`](#adcs-run) | Fly one scenario on one case: the plant, the environment and the POP orbit in Rust, the flight software (C or Rust, or a virtual OBC) behind the byte HAL, then the metrics and their verdicts. |
| [`adcs params`](#adcs-params) | The flight software's parameter blob (adcs-fswcfg/1) for a scenario, as an OBC boots from it. |
| [`adcs size`](#adcs-size) | The demand survey on the case's orbit, then every actuator option sized to it (magnetorquers, fluid loop, RCS, wheels, CMG, VSCMG). |
| [`adcs parity`](#adcs-parity) | Fly the same scenario with two flight-software targets and report the largest difference in attitude and rate; bit-identical is the expected answer for C and Rust, and any difference exits with status 1. |
| [`adcs results`](#adcs-results) | The results store: every run with its provenance, one line each; one run in full; a run as one share file, and back. |
| [`engine.py build`](#enginepy-build) | Build and test everything that flies: the C flight software, the Rust flight software (host and Cortex-M), the virtual OBC firmware and the Rust engine. |
| [`engine.py run`](#enginepy-run) | Fly scenarios on the engine in parallel (every scenario when none is named). |
| [`engine.py mc`](#enginepy-mc) | A seed sweep of one scenario: the same scenario flown with N sensor-noise seeds. |
| [`engine.py fsw-parity`](#enginepy-fsw-parity) | C against Rust flight software on the same loop, same bytes, scenario by scenario. |
| [`engine.py twin-parity`](#enginepy-twin-parity) | The engine's metrics against the MATLAB twin's, scenario by scenario and metric by metric. |
| [`engine.py vobc`](#enginepy-vobc) | The flight software as a separate process and as Cortex-M4 firmware in QEMU, over adcs-link/1, compared with the in-process builds. |
| [`engine.py dispatch`](#enginepy-dispatch) | The recommended solution's flight configuration for each case, and a mission check of it on the engine. |
| [`engine.py campaign`](#enginepy-campaign) | Every Monte Carlo and edge campaign (or those named) on the engine, with the twin's draws: the same dispersions, bounds, run count and per-run seeds. |
| [`engine.py campaign-ledger`](#enginepy-campaign-ledger) | Rewrite the engine-vs-twin campaign ledger from the stored summaries, without flying anything. |
| [`engine.py oils`](#enginepy-oils) | SILS and soft OILS side by side: the flight software as Cortex-M4F firmware, with exact instruction timing and command latency. |
| [`engine.py oils-ledger`](#enginepy-oils-ledger) | Rewrite the soft-OILS ledger from the stored runs, without flying anything. |
| [`engine.py solutions`](#enginepy-solutions) | The customer-case solution matrix on the engine: every mission mode x option x seed, with the sized products. |
| [`pipeline.py pipeline`](#pipelinepy-pipeline) | The design loop, node by node: from a customer case to a selected, dispatched, verified ADCS (docs/DESIGN_LOOP.md). |
| [`report.py report`](#reportpy-report) | The results report from the filed runs: figures, verdict tables, the summary and the results documents. It reads; it never flies. |
| [`vv_report.py vv-report`](#vv_reportpy-vv-report) | The downloadable V&V report: the template filled from the filed results, as self-contained HTML and as a PDF. |
| [`pack_matlab.py pack-matlab`](#pack_matlabpy-pack-matlab) | The downloadable MATLAB SILS zip, deterministic (sorted files, fixed timestamps), with its SHA-256 manifest. |
| [`pack_flight.py pack-flight`](#pack_flightpy-pack-flight) | The flight software and Rust engine zip, in the repository's layout so it builds as unpacked. |
| [`gen_fsw_params.py gen-fsw-params`](#gen_fsw_paramspy-gen-fsw-params) | The flight software's parameter and table sources, C and Rust, from their one definition. --check says which generated file is stale, and changes nothing. |
| [`export_catalogue.py export-catalogue`](#export_cataloguepy-export-catalogue) | The catalogue, scenarios, campaigns and trades from TOML to the JSON the MATLAB twin and the engine read. --check says which JSON has drifted from its TOML, and changes nothing. |
| [`nodes_doc.py nodes-doc`](#nodes_docpy-nodes-doc) | docs/NODES.md and docs/CATALOGUE.md from the node registry and the datasheet catalogue. |
| [`components_doc.py components-doc`](#components_docpy-components-doc) | docs/COMPONENTS.md: every sensor and actuator, the model the SILS flies, and its processing chain. |
| [`catalogue.py catalogue`](#cataloguepy-catalogue) | Re-derive each bought wheel's and CMG's modelling block from its datasheet numbers (never guessed), then the catalogue document. |
| [`verify_nodes.py verify-nodes`](#verify_nodespy-verify-nodes) | Node-by-node verification of the design loop: each decision recomputed from the node's stored inputs, not trusted from its verdict. |
| [`rescore.py rescore`](#rescorepy-rescore) | Re-judge stored runs against the case files as they are now: a changed requirement changes a verdict, not a trajectory. |
| [`floquet.py floquet`](#floquetpy-floquet) | Floquet multipliers of the coils-only nadir loop: the certificate that the periodic magnetic control is stable. |
| [`run_matrix.py run-matrix`](#run_matrixpy-run-matrix) | The whole MATLAB-twin test matrix in GNU Octave on N workers, longest jobs first; campaigns and trades collected at the end. |
| [`fswcfg.py fswcfg`](#fswcfgpy-fswcfg) | Decode and check a flight-software parameter blob (adcs-fswcfg/1) and print every field as JSON. |
| [`trinetra.py explain`](#trinetrapy-explain) | This registry: every command, what it does before it does it; `docs` writes docs/COMMANDS.md from it. |
| [`check_all.py check-all`](#check_allpy-check-all) | Every check the repository has, one line each with a verdict: the Python tests, the generated files, the C and Rust flight software, the engine, the specification package and the stored design loop; --octave adds the MATLAB twin's suites. |

## adcs run

Fly one scenario on one case: the plant, the environment and the POP orbit in Rust, the flight software (C or Rust, or a virtual OBC) behind the byte HAL, then the metrics and their verdicts.

    adcs run <scenario> [--case F] [--fsw c|rust|obc-posix|qemu|...] [--seed N] [--out DIR] [--set k=v]... [--oils] [--quiet]

**Steps**

1. check the scenario id, the case and every --set (refused by name, never guessed)
2. build the configuration: case + scenario + product, the flight-software parameters
3. fly the closed loop for the scenario's duration
4. derive the metrics and judge them against the case's requirements
5. write channels.csv, then manifest.json with the run's provenance

- **Reads:** `matlab_sils/data/scenarios/<scenario>.json`; `matlab_sils/cases/<case>.csv`; `matlab_sils/data/products, parts, algorithms`; `matlab_sils/pop/.../de440s.bsp`
- **Writes:** `matlab_sils/store/results_engine/<scenario>/ (or --out): channels.csv, manifest.json`
- **Starts:** nothing

## adcs params

The flight software's parameter blob (adcs-fswcfg/1) for a scenario, as an OBC boots from it.

    adcs params <scenario> [--case F] [--set k=v]... --out blob.bin

**Steps**

1. build the configuration as `adcs run` does
2. encode the parameters with their CRC-32

- **Reads:** the scenario, the case, the product
- **Writes:** the --out file
- **Starts:** nothing

## adcs size

The demand survey on the case's orbit, then every actuator option sized to it (magnetorquers, fluid loop, RCS, wheels, CMG, VSCMG).

    adcs size <case> [--knobs k.json] [--out DIR]

**Steps**

1. check the case id
2. fly the one-orbit survey to find the disturbance torque and momentum demand
3. size every option with the knobs, pick bought wheels and CMGs from the datasheet catalogue
4. write the sized parts, products and the summary

- **Reads:** `matlab_sils/cases/<case>.csv`; `matlab_sils/data/catalogue, parts, families.json`
- **Writes:** `matlab_sils/store/design/<case>/sized/ (or --out): parts/, products/, sizing.json`
- **Starts:** nothing

## adcs parity

Fly the same scenario with two flight-software targets and report the largest difference in attitude and rate; bit-identical is the expected answer for C and Rust, and any difference exits with status 1.

    adcs parity <scenario> [--fsw A --against B] [--set k=v]...

**Steps**

1. fly the loop with target A
2. fly it with target B
3. compare every recorded step

- **Reads:** as `adcs run`
- **Writes:** nothing
- **Starts:** the virtual OBC or QEMU, when a target names one

## adcs results

The results store: every run with its provenance, one line each; one run in full; a run as one share file, and back.

    adcs results list [DIR] | show <run> | export <run> --out F.trinetra | import F.trinetra --out DIR

**Steps**

1. list: find every manifest.json (adcs-rec/1) under the folder
2. show: print the run's provenance and requirement metrics
3. export: write README, manifest and channels into one zip
4. import: check every entry's name and checksum, then write them into the folder

- **Reads:** `matlab_sils/store/results_engine/ (or DIR)`
- **Writes:** `export: the .trinetra file; import: the folder`
- **Starts:** nothing

## engine.py build

Build and test everything that flies: the C flight software, the Rust flight software (host and Cortex-M), the virtual OBC firmware and the Rust engine.

    python3 tools/engine.py build

**Steps**

1. make clean, make test and make check in fsw/ (the C flight software's checks)
2. cargo test the Rust flight software
3. build its C-ABI library for the host and for thumbv7em-none-eabihf
4. make obc: the virtual OBC firmware (process and QEMU) and the instruction plugin
5. cargo test and cargo build --release the engine

- **Reads:** `fsw/, fsw-rs/, engine/`
- **Writes:** `fsw/build/, fsw-rs/target/, engine/target/`
- **Starts:** make; gcc; arm-none-eabi-gcc; cargo

## engine.py run

Fly scenarios on the engine in parallel (every scenario when none is named).

    python3 tools/engine.py run [scenario ...] [--fsw c|rust] [--jobs N] [--seed S]

**Steps**

1. one `adcs run` per scenario, on --jobs processes
2. print each run's verdicts

- **Reads:** `matlab_sils/data/scenarios/`
- **Writes:** `matlab_sils/store/results_engine/<scenario>/`
- **Starts:** adcs run

## engine.py mc

A seed sweep of one scenario: the same scenario flown with N sensor-noise seeds.

    python3 tools/engine.py mc <scenario> --seeds N [--fsw c|rust] [--jobs N]

**Steps**

1. one `adcs run` per seed
2. summarise every metric over the seeds

- **Reads:** the scenario and its case
- **Writes:** `matlab_sils/store/results_engine/mc_<scenario>/summary.json`
- **Starts:** adcs run

## engine.py fsw-parity

C against Rust flight software on the same loop, same bytes, scenario by scenario.

    python3 tools/engine.py fsw-parity [scenario ...] [--duration S]

**Steps**

1. `adcs parity` per scenario
2. print the largest differences

- **Reads:** the scenarios
- **Writes:** nothing
- **Starts:** adcs parity

## engine.py twin-parity

The engine's metrics against the MATLAB twin's, scenario by scenario and metric by metric.

    python3 tools/engine.py twin-parity

**Steps**

1. read each scenario's engine and twin manifests
2. compare every metric and verdict

- **Reads:** `matlab_sils/store/results_engine/`; `matlab_sils/store/results/`
- **Writes:** `results/ENGINE_PARITY.md`; `results/engine_parity.json`
- **Starts:** nothing

## engine.py vobc

The flight software as a separate process and as Cortex-M4 firmware in QEMU, over adcs-link/1, compared with the in-process builds.

    python3 tools/engine.py vobc [scenario ...] [--duration S]

**Steps**

1. fly each scenario in-process and on each virtual OBC
2. compare the trajectories

- **Reads:** the scenarios; `fsw/build/ (the OBC firmware)`
- **Writes:** `results/VIRTUAL_OBC.md`
- **Starts:** adcs parity; qemu-system-arm

## engine.py dispatch

The recommended solution's flight configuration for each case, and a mission check of it on the engine.

    python3 tools/engine.py dispatch [case ...]

**Steps**

1. read the case's selection
2. write the parameter blob, its decoded JSON and the build notes
3. fly the mission with C and with Rust and compare

- **Reads:** `matlab_sils/store/pipeline/<case>/`
- **Writes:** `dist/dispatch/<case>/<family>/fsw/`
- **Starts:** adcs params; adcs run

## engine.py campaign

Every Monte Carlo and edge campaign (or those named) on the engine, with the twin's draws: the same dispersions, bounds, run count and per-run seeds.

    python3 tools/engine.py campaign [campaign ...] [--fsw c|rust] [--jobs N]

**Steps**

1. draw each run's dispersions (the Kp -> ap table included)
2. one `adcs run` per run
3. summarise the campaign
4. rewrite the engine-vs-twin campaign ledger

- **Reads:** `matlab_sils/data/campaigns/`; `matlab_sils/store/results/<campaign>/summary.json (the twin)`
- **Writes:** `matlab_sils/store/results_engine/campaigns/<campaign>/`; `results/ENGINE_CAMPAIGNS.md`; `results/engine_campaigns.json`
- **Starts:** adcs run

## engine.py campaign-ledger

Rewrite the engine-vs-twin campaign ledger from the stored summaries, without flying anything.

    python3 tools/engine.py campaign-ledger

**Steps**

1. read every campaign summary, engine and twin
2. write the ledger

- **Reads:** `matlab_sils/store/results_engine/campaigns/`; `matlab_sils/store/results/`
- **Writes:** `results/ENGINE_CAMPAIGNS.md`; `results/engine_campaigns.json`
- **Starts:** nothing

## engine.py oils

SILS and soft OILS side by side: the flight software as Cortex-M4F firmware, with exact instruction timing and command latency.

    python3 tools/engine.py oils [scenario ...] [--fsw qemu|qemu-rs] [--duration S] [--jobs N]

**Steps**

1. fly each scenario in SILS
2. fly it with --oils on QEMU
3. tabulate latency, CPU load and the metric differences

- **Reads:** the scenarios; `fsw/build/ (the firmware)`
- **Writes:** `matlab_sils/store/results_engine/soft_oils/`; `results/SOFT_OILS.md`; `results/soft_oils.json`
- **Starts:** adcs run --oils; qemu-system-arm

## engine.py oils-ledger

Rewrite the soft-OILS ledger from the stored runs, without flying anything.

    python3 tools/engine.py oils-ledger

**Steps**

1. read the stored SILS and OILS runs
2. write the ledger

- **Reads:** `matlab_sils/store/results_engine/soft_oils/`
- **Writes:** `results/SOFT_OILS.md`; `results/soft_oils.json`
- **Starts:** nothing

## engine.py solutions

The customer-case solution matrix on the engine: every mission mode x option x seed, with the sized products.

    python3 tools/engine.py solutions [case ...] [--seeds 1,2] [--fsw c|rust] [--jobs N]

**Steps**

1. size each case
2. fly every mode, option and seed
3. tabulate feasibility and budgets

- **Reads:** `matlab_sils/cases/`; `matlab_sils/data/modes/`
- **Writes:** `matlab_sils/store/solutions_engine/`; `results/ENGINE_SOLUTIONS.md`
- **Starts:** adcs size; adcs run

## pipeline.py pipeline

The design loop, node by node: from a customer case to a selected, dispatched, verified ADCS (docs/DESIGN_LOOP.md).

    python3 tools/pipeline.py [case ...] [--seeds 1,2] [--max-iter 5] [--jobs N] [--mc-runs 12] [--no-oils]

**Steps**

1. size: the demand survey and every option sized
2. matrix: every mode x option x seed flown with the sized products
3. assess: each failing requirement classed (performance, knowledge, power, propellant)
4. converge: resize or upgrade what failed, and repeat until nothing is left to change
5. select: the simplest solution family that passes every mode within the mass and volume budget
6. dispatch: its flight configuration, checked with C and Rust
7. mc: a Monte Carlo of the dispatched mission
8. soft_oils: the dispatched mission on the Cortex-M4F firmware (unless --no-oils)

- **Reads:** `matlab_sils/cases/<case>.csv`; `matlab_sils/data/`
- **Writes:** `matlab_sils/store/pipeline/<case>/<node>.json`; `dist/dispatch/<case>/`
- **Starts:** adcs size; adcs run; adcs params; qemu-system-arm

## report.py report

The results report from the filed runs: figures, verdict tables, the summary and the results documents. It reads; it never flies.

    python3 tools/report.py

**Steps**

1. read every filed run, campaign and trade
2. draw each test's figures
3. write the page, the summary and the documents

- **Reads:** `matlab_sils/store/results/`; `matlab_sils/store/trades/`
- **Writes:** `results/figures/`; `results/index.html`; `results/summary.json`; `docs/RESULTS.md`; `docs/SELECTION.md`
- **Starts:** nothing

## vv_report.py vv-report

The downloadable V&V report: the template filled from the filed results, as self-contained HTML and as a PDF.

    python3 tools/vv_report.py

**Steps**

1. collect every source that exists (a missing one is named, never invented)
2. fill tools/templates/vv_report.html
3. print it to PDF with headless Chromium

- **Reads:** `matlab_sils/store/pipeline/`; `matlab_sils/store/results_engine/`; `matlab_sils/store/results/`; `results/*.json`
- **Writes:** `results/vv/TRINETRA_ADCS_VV_report.html`; `results/vv/TRINETRA_ADCS_VV_report.pdf`; `dist/TRINETRA_ADCS_VV_report.pdf`
- **Starts:** chromium (headless)

## pack_matlab.py pack-matlab

The downloadable MATLAB SILS zip, deterministic (sorted files, fixed timestamps), with its SHA-256 manifest.

    python3 tools/pack_matlab.py

**Steps**

1. collect matlab_sils/ and the documents
2. write the zip beside its name, then rename it into place

- **Reads:** `matlab_sils/`; `docs/`
- **Writes:** `dist/TRINETRA_ADCS_SILS_matlab_<version>.zip`
- **Starts:** nothing

## pack_flight.py pack-flight

The flight software and Rust engine zip, in the repository's layout so it builds as unpacked.

    python3 tools/pack_flight.py

**Steps**

1. collect fsw/, fsw-rs/, engine/, the data and the tools
2. write the zip beside its name, then rename it into place

- **Reads:** `fsw/`; `fsw-rs/`; `engine/`; `matlab_sils/data`; `tools/`
- **Writes:** `dist/TRINETRA_ADCS_flight_engine_<version>.zip`
- **Starts:** nothing

## gen_fsw_params.py gen-fsw-params

The flight software's parameter and table sources, C and Rust, from their one definition. --check says which generated file is stale, and changes nothing.

    python3 tools/gen_fsw_params.py [--check]

**Steps**

1. read fsw/params/params.toml and the IGRF table
2. write the C header and source and the Rust module

- **Reads:** `fsw/params/params.toml`; `matlab_sils/data/igrf13.json`
- **Writes:** `fsw/include/adcs_params.h`; `fsw/src/adcs_params.c`; `fsw-rs/src/params.rs`; `fsw/include/adcs_igrf13.h`; `fsw-rs/src/igrf13.rs`
- **Starts:** nothing

## export_catalogue.py export-catalogue

The catalogue, scenarios, campaigns and trades from TOML to the JSON the MATLAB twin and the engine read. --check says which JSON has drifted from its TOML, and changes nothing.

    python3 tools/export_catalogue.py [--check]

**Steps**

1. read every TOML source
2. write (or, with --check, compare) the JSON

- **Reads:** `catalogue/`; `scenarios/`; `campaigns/`; `trades/`
- **Writes:** `matlab_sils/data/*/<id>.json`; `matlab_sils/data/families.json`
- **Starts:** nothing

## nodes_doc.py nodes-doc

docs/NODES.md and docs/CATALOGUE.md from the node registry and the datasheet catalogue.

    python3 tools/nodes_doc.py

**Steps**

1. read the node registry and the catalogue
2. write both documents

- **Reads:** `matlab_sils/data/pipeline/nodes.json`; `matlab_sils/data/catalogue/`
- **Writes:** `docs/NODES.md`; `docs/CATALOGUE.md`
- **Starts:** nothing

## components_doc.py components-doc

docs/COMPONENTS.md: every sensor and actuator, the model the SILS flies, and its processing chain.

    python3 tools/components_doc.py

**Steps**

1. read the component files
2. write the document

- **Reads:** `catalogue/components/`
- **Writes:** `docs/COMPONENTS.md`
- **Starts:** nothing

## catalogue.py catalogue

Re-derive each bought wheel's and CMG's modelling block from its datasheet numbers (never guessed), then the catalogue document.

    python3 tools/catalogue.py

**Steps**

1. derive every model's engine parameters by rule
2. mark a model not selectable when its datasheet lacks a number the selection needs

- **Reads:** `matlab_sils/data/catalogue/`
- **Writes:** `matlab_sils/data/catalogue/*.json`; `docs/CATALOGUE.md`
- **Starts:** nothing

## verify_nodes.py verify-nodes

Node-by-node verification of the design loop: each decision recomputed from the node's stored inputs, not trusted from its verdict.

    python3 tools/verify_nodes.py [case ...]

**Steps**

1. read every node's outputs per case
2. recompute each decision and compare
3. exit non-zero when a check fails

- **Reads:** `matlab_sils/data/pipeline/nodes.json`; `matlab_sils/store/pipeline/`
- **Writes:** `results/NODE_VERIFICATION.md`; `results/node_verification.json`
- **Starts:** nothing

## rescore.py rescore

Re-judge stored runs against the case files as they are now: a changed requirement changes a verdict, not a trajectory.

    python3 tools/rescore.py [--dry-run]

**Steps**

1. read every stored metric record
2. apply the case's current requirement
3. recompute campaign statistics

- **Reads:** `matlab_sils/store/`; `matlab_sils/cases/`
- **Writes:** the stored manifests and summaries (not with --dry-run)
- **Starts:** nothing

## floquet.py floquet

Floquet multipliers of the coils-only nadir loop: the certificate that the periodic magnetic control is stable.

    python3 tools/floquet.py <case>

**Steps**

1. linearise the closed loop about the nadir reference along one orbit
2. integrate the monodromy matrix
3. write the multipliers and the verdict

- **Reads:** `matlab_sils/cases/<case>.csv`; the flight-software parameters
- **Writes:** `matlab_sils/store/pipeline/<case>/floquet.json`
- **Starts:** nothing

## run_matrix.py run-matrix

The whole MATLAB-twin test matrix in GNU Octave on N workers, longest jobs first; campaigns and trades collected at the end.

    python3 tools/run_matrix.py --workers N [--only scenarios|campaigns|trades|solutions]

**Steps**

1. list every scenario, campaign run and trade job
2. run them on N Octave workers
3. collect and plot the campaigns and trades

- **Reads:** `matlab_sils/data/`
- **Writes:** `matlab_sils/store/results/`; `matlab_sils/store/trades/`; `matlab_sils/store/logs/`
- **Starts:** octave-cli

## fswcfg.py fswcfg

Decode and check a flight-software parameter blob (adcs-fswcfg/1) and print every field as JSON.

    python3 tools/fswcfg.py <blob>

**Steps**

1. check the magic, length and CRC-32
2. decode each field in table order

- **Reads:** the blob; `fsw/params/params.toml`
- **Writes:** nothing
- **Starts:** nothing

## trinetra.py explain

This registry: every command, what it does before it does it; `docs` writes docs/COMMANDS.md from it.

    python3 tools/trinetra.py list | explain <command> | docs [--check]

**Steps**

1. read docs/commands.toml
2. print, explain, or write the document

- **Reads:** `docs/commands.toml`
- **Writes:** `docs/COMMANDS.md (docs only)`
- **Starts:** nothing

## check_all.py check-all

Every check the repository has, one line each with a verdict: the Python tests, the generated files, the C and Rust flight software, the engine, the specification package and the stored design loop; --octave adds the MATLAB twin's suites.

    python3 tools/check_all.py [--octave] [--only NAME...]

**Steps**

1. run each check in turn, showing what it proves
2. report a check whose program is missing as not run, never as passed
3. print the table and exit 1 if any check failed

- **Reads:** the repository
- **Writes:** `fsw/build/, fsw-rs/target/, engine/target/ (the builds the tests need)`
- **Starts:** python3; make; gcc; cargo; octave-cli (with --octave)
