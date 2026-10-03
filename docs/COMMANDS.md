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
| [`adcs results`](#adcs-results) | The results store: every run with its provenance, one line each; one run in full; runs kept (pinned) or thinned to their manifest when old; a run as one share file, and back; any read-only question to its SQLite index; the runs another engine or other inputs flew; a stored run flown again with what changed. |
| [`engine.py build`](#enginepy-build) | Build and test everything that flies: the C flight software, the Rust flight software (host and Cortex-M), the virtual OBC firmware and the Rust engine. |
| [`engine.py run`](#enginepy-run) | Fly scenarios on the engine in parallel (every scenario when none is named). |
| [`engine.py mc`](#enginepy-mc) | A seed sweep of one scenario: the same scenario flown with N sensor-noise seeds. |
| [`engine.py fsw-parity`](#enginepy-fsw-parity) | C against Rust flight software on the same loop, same bytes, scenario by scenario. |
| [`engine.py twin-parity`](#enginepy-twin-parity) | The engine's metrics against the MATLAB twin's, scenario by scenario and metric by metric. |
| [`engine.py vobc`](#enginepy-vobc) | The flight software as a separate process and as Cortex-M4 firmware in QEMU, over adcs-link/1, compared with the in-process builds. |
| [`engine.py dispatch`](#enginepy-dispatch) | The recommended solution's flight configuration for each case, and a mission check of it on the engine. |
| [`engine.py campaign`](#enginepy-campaign) | Every Monte Carlo and edge campaign (or those named) on the engine, with the twin's draws: the same dispersions, bounds, run count and per-run seeds. |
| [`engine.py campaign-ledger`](#enginepy-campaign-ledger) | Rewrite the engine-vs-twin campaign ledger from the stored summaries, without flying anything. |
| [`engine.py oils`](#enginepy-oils) | SILS and soft OILS side by side: the flight software as Cortex-M4F firmware, with exact instruction timing and command latency; every soft-OILS run judged on overruns and on its worst-case latency against the deadline; --cpi flies a CPI sweep. |
| [`engine.py oils-ledger`](#enginepy-oils-ledger) | Rewrite the soft-OILS ledger from the stored runs, without flying anything. |
| [`engine.py solutions`](#enginepy-solutions) | The customer-case solution matrix on the engine: every mission mode x option x seed, with the sized products. |
| [`pipeline.py pipeline`](#pipelinepy-pipeline) | The design loop, node by node: from a customer case to a selected, dispatched, verified ADCS (docs/DESIGN_LOOP.md). |
| [`report.py report`](#reportpy-report) | The results report from the filed runs: figures, verdict tables, the summary and the results documents. It reads; it never flies. |
| [`vv_report.py vv-report`](#vv_reportpy-vv-report) | The downloadable V&V report: the template filled from the filed results, as self-contained HTML and as a PDF. |
| [`pack_matlab.py pack-matlab`](#pack_matlabpy-pack-matlab) | The downloadable MATLAB SILS zip, deterministic (sorted files, fixed timestamps), with its SHA-256 manifest. |
| [`pack_flight.py pack-flight`](#pack_flightpy-pack-flight) | The flight software and Rust engine zip, in the repository's layout so it builds as unpacked. |
| [`gen_fsw_params.py gen-fsw-params`](#gen_fsw_paramspy-gen-fsw-params) | The flight software's parameter and table sources, C and Rust, from their one definition. --check says which generated file is stale, and changes nothing. |
| [`fsw_stack.py fsw-stack`](#fsw_stackpy-fsw-stack) | The flight software's deepest stack on the Cortex-M4 firmware, from GCC's call graph, against the stack the linker script reserves. Recursion and unbounded frames are refused; a library routine is charged a fixed frame and named. |
| [`design_rows.py design-rows`](#design_rowspy-design-rows) | Every row of the ADCS tree, from the spec package: 734 rows (133 in layer 1, 194 in layer 2, 368 in the subsystem layers, 39 closures), each with its short id, the id its node file carries, its layer, kind, branch and label. Fails when the counts SPEC.md states do not hold or an id repeats. |
| [`groups.py groups`](#groupspy-groups) | The group map (design/groups.toml) against every row of the tree: each row in exactly one of the 20 discipline groups, each group holding the rows it states, each stage inside its group, every override and boundary naming real rows and groups. --row says where one row goes and why. |
| [`tndb.py tndb`](#tndbpy-tndb) | The design files (node, group, release, design database), from design/schema.toml: check a file's format, version and every table; dump it as canonical JSON; print the SQL that makes a kind; write or check the files made from the schema (design/ddl.sql, design/js/tndb_schema.js). An older file is upgraded with a copy kept; a newer one is refused. |
| [`seed_design.py seed-design`](#seed_designpy-seed-design) | Seeds the design files from the spec: a group file for each of the 20 groups, a node file for each of the 734 rows (the 82 the spec seeds with their content, the rest as shells), and the starting design database. Never overwrites; --check seeds into a temporary folder and checks every file. |
| [`version.py version`](#versionpy-version) | One version for the repository: VERSION is the source, and the engine's Cargo workspace, the Rust flight software's Cargo package and the C flight software's build id follow it; the Rust build ids are built from their Cargo version. --check fails on any drift; --set writes a new version everywhere. |
| [`mutation.py mutation`](#mutationpy-mutation) | Mutation testing of the flight software's guidance, control and estimation (fsw-rs/src/guid.rs, ctl.rs, est.rs) with cargo-mutants: each small deliberate fault in turn, and whether the Rust flight software's tests catch it. Writes the kill rate per function and the missed mutants; --check fails under the floor the tool states. |
| [`trace.py trace`](#tracepy-trace) | The requirements traceability matrix: every requirement a case states, what checks it (a flown scenario's metric, the design loop's budget or mode flights, the reference slew's profile) and what the latest stored result says. --check refuses a metric that neither judges nor says why it only reports, a requirement key the case lacks, and a stated requirement nothing checks; it writes nothing. |
| [`pointing_budget.py pointing-budget`](#pointing_budgetpy-pointing-budget) | The absolute pointing error budget (SPEC rows gp_0 to gp_5) of each fine-pointing scenario: knowledge and control from one flight on today's engine, payload alignment from the product, thermal distortion from the case, rotor jitter from the engine, their root-sum-square against req.ape, and the room req.ape leaves for alignment and thermal. A term nobody states keeps the budget incomplete. |
| [`export_catalogue.py export-catalogue`](#export_cataloguepy-export-catalogue) | The catalogue, scenarios, campaigns and trades from TOML to the JSON the MATLAB twin and the engine read. --check says which JSON has drifted from its TOML, and changes nothing. |
| [`nodes_doc.py nodes-doc`](#nodes_docpy-nodes-doc) | docs/NODES.md and docs/CATALOGUE.md from the node registry and the datasheet catalogue. |
| [`components_doc.py components-doc`](#components_docpy-components-doc) | docs/COMPONENTS.md: every sensor and actuator, the model the SILS flies, and its processing chain. |
| [`catalogue.py catalogue`](#cataloguepy-catalogue) | Re-derive each bought wheel's and CMG's modelling block from its datasheet numbers (never guessed), then the catalogue document. |
| [`verify_nodes.py verify-nodes`](#verify_nodespy-verify-nodes) | Node-by-node verification of the design loop: each decision recomputed from the node's stored inputs, not trusted from its verdict. |
| [`rescore.py rescore`](#rescorepy-rescore) | Re-judge stored runs against the case files as they are now: a changed requirement changes a verdict, not a trajectory. |
| [`floquet.py floquet`](#floquetpy-floquet) | Floquet multipliers of the coils-only nadir loop: the certificate that the periodic magnetic control is stable. |
| [`run_matrix.py run-matrix`](#run_matrixpy-run-matrix) | The whole MATLAB-twin test matrix in GNU Octave on N workers, longest jobs first; campaigns and trades collected at the end. |
| [`fswcfg.py fswcfg`](#fswcfgpy-fswcfg) | Decode and check a flight-software parameter blob (adcs-fswcfg/1) and print every field as JSON. |
| [`trinetra.py explain`](#trinetrapy-explain) | This registry: every command, what it does before it does it; `why` names the command that writes a file; `status` puts the evidence debt first (unconfirmed algorithms, synthetic parts, unselectable catalogue models, engine-twin disagreements, failing design-loop checks); `docs` writes docs/COMMANDS.md. |
| [`check_all.py check-all`](#check_allpy-check-all) | Every check the repository has, one line each with a verdict: the Python tests, the generated files, the C and Rust flight software, the engine, the specification package and the stored design loop; --octave adds the MATLAB twin's suites. |
| [`kit.py kit`](#kitpy-kit) | The tool as a team member gets it: the programs beside exactly the files they read (data, cases, the ephemeris), with VERSION and the first-run documents; --files-only leaves the programs out, for the Python package. |
| [`make_icon.py make-icon`](#make_iconpy-make-icon) | Draw the desktop app's icon (three eyes on an orbit) as PNG, ICO and ICNS; run only to change it. |
| [`macapp.py macapp`](#macapppy-macapp) | The macOS desktop app, TRI-NETRA ADCS.app, from a macOS kit: the app program in Contents/MacOS, the kit's data in Contents/Resources, the icon, Info.plist and PkgInfo. The release workflow signs it and zips it. |
| [`build_wheel.py build-wheel`](#build_wheelpy-build-wheel) | One Python package for every computer: the front end (python/trinetra_adcs), the engine and the desktop app for each system given, and the data; it installs trinetra-adcs and trinetra-adcs-app and compiles nothing. |
| [`adcs-sim scenario-schema`](#adcs-sim-scenario-schema) | The scenario schema for the MATLAB twin: every scenario key the engine reads, with its type, written from the engine's own table so the twin refuses exactly what the engine refuses. |
| [`pcode.py pcode`](#pcodepy-pcode) | Pseudocode v2 (docs/PSEUDOCODE_V2.md): check a file (units, types, every output set, no recursion), run a function in the interpreter, and write or check everything it makes: the physics (spec/physics/*.pc) as a Rust crate and a MATLAB package with test vectors from the interpreter, the language's self-test the same way, the flight algorithms (fsw/pseudocode/*.pc) as test vectors for the C and Rust flight software, the MATLAB runtime, and the browser checker page. gen --check also holds the physics to spec/plan/physics.toml; fixtures runs the seeded, sourced test vectors of the physics rows. |

## adcs run

Fly one scenario on one case: the plant, the environment and the POP orbit in Rust, the flight software (C or Rust, or a virtual OBC) behind the byte HAL, then the metrics and their verdicts.

    adcs run <scenario> [--case F] [--fsw c|rust|obc-posix|qemu|...] [--seed N] [--out DIR] [--set k=v]... [--oils] [--quiet]

**Steps**

1. check the scenario id, the case and every --set (refused by name, never guessed)
2. build the configuration: case + scenario + product, the flight-software parameters
3. fly the closed loop for the scenario's duration
4. derive the metrics and judge them against the case's requirements
5. keep the case and scenario files it flew, once each by fingerprint, in the store's inputs/
6. write channels.csv, then manifest.json with the run's provenance (engine source, case, scenario and product fingerprints)
7. apply the store's retention when the run went into the store (a kit: time series older than 30 days, pinned runs aside; TRINETRA_RETENTION_DAYS)

- **Reads:** `matlab_sils/data/scenarios/<scenario>.json`; `matlab_sils/cases/<case>.csv`; `matlab_sils/data/products, parts, algorithms`; `matlab_sils/pop/.../de440s.bsp`
- **Writes:** `matlab_sils/store/results_engine/<scenario>/ (or --out): channels.csv, manifest.json`; `matlab_sils/store/inputs/ (or <out>/inputs/)`
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

The results store: every run with its provenance, one line each; one run in full; runs kept (pinned) or thinned to their manifest when old; a run as one share file, and back; any read-only question to its SQLite index; the runs another engine or other inputs flew; a stored run flown again with what changed.

    adcs results list [DIR] | show <run> | pin|unpin <run> | thin --older-than DAYS [DIR] [--dry-run] | export <run> --out F.trinetra | import F.trinetra --out DIR | query --sql SELECT [DIR] | stale [DIR] | refly <run> [--out DIR] [--fsw T]

**Steps**

1. list: find every manifest.json (adcs-rec/1) under the folder, reading only those new or changed since its index (.adcs-index.sqlite), then bring the index to exactly what is there
2. show: print the run's provenance and requirement metrics and the command that flies it again, from the inputs it kept
3. pin: mark a run to keep; thin: remove the bulk (channels.csv, the twin's rec.mat and run_*.mat) of unpinned runs older than DAYS, keeping every manifest
4. export: write README, manifest, channels and the kept inputs into one zip
5. import: check every entry's name and checksum, then write them into the folder
6. query: bring the index up to date, then run one read-only SQL statement on its runs and metrics tables
7. stale: compare every run's engine source, case, scenario and product fingerprints with today's and name what differs; exit 1 when any run is stale
8. refly: fly the run again from its kept inputs into <store>/refly/<run> and print every metric stored against now

- **Reads:** `matlab_sils/store/results_engine/ (or DIR)`; the case, scenario and product files the runs name (stale); the run's kept inputs (refly)
- **Writes:** `list, query, stale: <folder>/.adcs-index.sqlite; pin: <run>/PINNED; thin: removes the bulk files; export: the .trinetra file; import: the folder; refly: <store>/refly/<run>/ (or --out)`
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

1. fly each scenario with the C and the Rust flight software (`adcs parity`)
2. print the largest differences

- **Reads:** the scenarios
- **Writes:** nothing
- **Starts:** adcs parity

## engine.py twin-parity

The engine's metrics against the MATLAB twin's, scenario by scenario and metric by metric.

    python3 tools/engine.py twin-parity

**Steps**

1. read each scenario's engine and twin manifests and pair their metrics
2. tabulate verdict agreement and ratios, and write the ledger

- **Reads:** `matlab_sils/store/results_engine/`; `matlab_sils/store/results/`
- **Writes:** `results/ENGINE_PARITY.md`; `results/engine_parity.json`
- **Starts:** nothing

## engine.py vobc

The flight software as a separate process and as Cortex-M4 firmware in QEMU, over adcs-link/1, compared with the in-process builds.

    python3 tools/engine.py vobc [scenario ...] [--duration S]

**Steps**

1. build the virtual OBC firmware (make obc)
2. fly each scenario in-process and on each virtual OBC with `adcs parity`, which compares the trajectories
3. write results/VIRTUAL_OBC.md

- **Reads:** the scenarios; `fsw/build/ (the OBC firmware)`
- **Writes:** `results/VIRTUAL_OBC.md`
- **Starts:** adcs parity; qemu-system-arm

## engine.py dispatch

The recommended solution's flight configuration for each case, and a mission check of it on the engine.

    python3 tools/engine.py dispatch [case ...]

**Steps**

1. read the case's recommended solution
2. write the mission scenario, the parameter blob and its decoded JSON
3. fly the mission with C and with Rust, and write the check and the build notes

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

SILS and soft OILS side by side: the flight software as Cortex-M4F firmware, with exact instruction timing and command latency; every soft-OILS run judged on overruns and on its worst-case latency against the deadline; --cpi flies a CPI sweep.

    python3 tools/engine.py oils [scenario ...] [--fsw qemu|qemu-rs] [--duration S] [--cpi C ...] [--jobs N]

**Steps**

1. build the virtual OBC firmware (make obc)
2. fly each scenario in SILS and with --oils on QEMU (and at each --cpi)
3. tabulate latency, CPU load, the worst-case deadline margin, the sweep and the metric differences

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

1. write each case's mode x option scenarios
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
4. select: the lightest solution family (then power, then volume) that passes every mode within the mass and volume budget
5. converge: resize or upgrade what failed, and repeat from size until nothing is left to change
6. faults: each solution family's mission flown once per single fault its hardware can carry; select counts the faults it does not survive
7. dispatch: the selected family's flight configuration, checked with C and Rust
8. mc: a Monte Carlo of the dispatched mission
9. robust: a requirement the Monte Carlo breaks sends the loop back to size with more margin
10. family_missions: every solution family's mission flown (SILS C and Rust, and a Monte Carlo)
11. certify: the Floquet certificate of each magnetic pointing law
12. soft_oils: the dispatched mission on the Cortex-M4F firmware (unless --no-oils)
13. ledger: the design ledger and its JSON

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
- **Writes:** `results/vv/TRINETRA_ADCS_VV_report.html`; `results/vv/vv_artifact.html`; `dist/TRINETRA_ADCS_VV_report.pdf`
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

- **Reads:** `fsw/params/params.toml`; `matlab_sils/data/igrf13coeffs.txt`
- **Writes:** `fsw/include/adcs_params.h`; `fsw/src/adcs_params.c`; `fsw-rs/src/params.rs`; `matlab_sils/data/igrf13.json`; `fsw/include/adcs_igrf13.h`; `fsw-rs/src/igrf13.rs`; `fsw/pseudocode/02_igrf13.pc`
- **Starts:** nothing

## fsw_stack.py fsw-stack

The flight software's deepest stack on the Cortex-M4 firmware, from GCC's call graph, against the stack the linker script reserves. Recursion and unbounded frames are refused; a library routine is charged a fixed frame and named.

    python3 tools/fsw_stack.py

**Steps**

1. compile the firmware sources with -fstack-usage -fcallgraph-info=su
2. walk the call graph from Reset_Handler and add the frames along the deepest path
3. fail if it is over _stack_size in link.ld

- **Reads:** `fsw/src/`; `fsw/targets/link/`; `fsw/targets/qemu-mps2/`
- **Writes:** nothing
- **Starts:** arm-none-eabi-gcc

## design_rows.py design-rows

Every row of the ADCS tree, from the spec package: 734 rows (133 in layer 1, 194 in layer 2, 368 in the subsystem layers, 39 closures), each with its short id, the id its node file carries, its layer, kind, branch and label. Fails when the counts SPEC.md states do not hold or an id repeats.

    python3 tools/design_rows.py [--list]

**Steps**

1. read the spec's tree, KPI list and long-id map
2. list every row, naming the subsystem-layer rows
3. check the counts and that every id is unique

- **Reads:** `spec/plan/tree.json`; `spec/plan/kpis.toml`; `spec/plan/expected_node_ids.json`
- **Writes:** nothing
- **Starts:** nothing

## groups.py groups

The group map (design/groups.toml) against every row of the tree: each row in exactly one of the 20 discipline groups, each group holding the rows it states, each stage inside its group, every override and boundary naming real rows and groups. --row says where one row goes and why.

    python3 tools/groups.py [--check | --row ID]

**Steps**

1. read the group map and every row
2. place each row (override, then its target, then its branch)
3. check the placement against the map

- **Reads:** `design/groups.toml`; `spec/plan/`
- **Writes:** nothing
- **Starts:** nothing

## tndb.py tndb

The design files (node, group, release, design database), from design/schema.toml: check a file's format, version and every table; dump it as canonical JSON; print the SQL that makes a kind; write or check the files made from the schema (design/ddl.sql, design/js/tndb_schema.js). An older file is upgraded with a copy kept; a newer one is refused.

    python3 tools/tndb.py check FILE... | dump FILE | ddl KIND | gen [--check]

**Steps**

1. read the schema
2. open each file, checking or upgrading its format
3. check, dump, print or generate

- **Reads:** `design/schema.toml`
- **Writes:** `design/ddl.sql`; `design/js/tndb_schema.js`
- **Starts:** nothing

## seed_design.py seed-design

Seeds the design files from the spec: a group file for each of the 20 groups, a node file for each of the 734 rows (the 82 the spec seeds with their content, the rest as shells), and the starting design database. Never overwrites; --check seeds into a temporary folder and checks every file.

    python3 tools/seed_design.py [--out DIR | --check]

**Steps**

1. write the group files, the node files and the design database from the spec
2. check every file against the schema

- **Reads:** `design/groups.toml`; `design/schema.toml`; `spec/plan/`
- **Writes:** `build/design/`
- **Starts:** nothing

## version.py version

One version for the repository: VERSION is the source, and the engine's Cargo workspace, the Rust flight software's Cargo package and the C flight software's build id follow it; the Rust build ids are built from their Cargo version. --check fails on any drift; --set writes a new version everywhere.

    python3 tools/version.py [--check | --set X.Y.Z]

**Steps**

1. read VERSION and the version each part states
2. check the Rust build ids take their Cargo version
3. with --set, write the new version into VERSION and every part

- **Reads:** VERSION; `engine/Cargo.toml`; `fsw-rs/Cargo.toml`; `fsw/src/adcs_fsw.c`; `engine/crates/adcs-sim/src/lib.rs`; `fsw-rs/src/fsw.rs`
- **Writes:** VERSION; `engine/Cargo.toml`; `fsw-rs/Cargo.toml`; `fsw/src/adcs_fsw.c`
- **Starts:** nothing

## mutation.py mutation

Mutation testing of the flight software's guidance, control and estimation (fsw-rs/src/guid.rs, ctl.rs, est.rs) with cargo-mutants: each small deliberate fault in turn, and whether the Rust flight software's tests catch it. Writes the kill rate per function and the missed mutants; --check fails under the floor the tool states.

    python3 tools/mutation.py [--from DIR] [--check] [--jobs N]

**Steps**

1. make and test every mutant (cargo mutants, about 30 min)
2. count caught and missed per function
3. write the record; with --check, fail under the floor

- **Reads:** `fsw-rs/src/guid.rs`; `fsw-rs/src/ctl.rs`; `fsw-rs/src/est.rs`; `fsw-rs/tests/`
- **Writes:** `results/MUTATION.md`; `results/mutation.json`
- **Starts:** cargo-mutants

## trace.py trace

The requirements traceability matrix: every requirement a case states, what checks it (a flown scenario's metric, the design loop's budget or mode flights, the reference slew's profile) and what the latest stored result says. --check refuses a metric that neither judges nor says why it only reports, a requirement key the case lacks, and a stated requirement nothing checks; it writes nothing.

    python3 tools/trace.py [--check]

**Steps**

1. read the cases, the scenarios and the mode catalogue
2. map each stated requirement to its checks and their latest stored verdicts
3. write the matrix (or, with --check, refuse what is undecided or unchecked)

- **Reads:** `matlab_sils/cases/`; `matlab_sils/data/scenarios/`; `matlab_sils/data/modes/`; `matlab_sils/store/results_engine/`; `matlab_sils/store/pipeline/`
- **Writes:** `results/TRACEABILITY.md`; `results/traceability.json`
- **Starts:** nothing

## pointing_budget.py pointing-budget

The absolute pointing error budget (SPEC rows gp_0 to gp_5) of each fine-pointing scenario: knowledge and control from one flight on today's engine, payload alignment from the product, thermal distortion from the case, rotor jitter from the engine, their root-sum-square against req.ape, and the room req.ape leaves for alignment and thermal. A term nobody states keeps the budget incomplete.

    python3 tools/pointing_budget.py

**Steps**

1. fly each fine-pointing scenario once with the jitter term added
2. read the product's payload alignment and the case's thermal distortion
3. add the terms in quadrature and judge against req.ape
4. write the budget page

- **Reads:** `matlab_sils/data/scenarios/`; `matlab_sils/cases/`; `catalogue/products/`
- **Writes:** `results/POINTING_BUDGET.md`; `results/pointing_budget.json`
- **Starts:** `engine/target/release/adcs`

## export_catalogue.py export-catalogue

The catalogue, scenarios, campaigns and trades from TOML to the JSON the MATLAB twin and the engine read. --check says which JSON has drifted from its TOML, and changes nothing.

    python3 tools/export_catalogue.py [--check]

**Steps**

1. read every TOML source
2. write (or, with --check, compare) the JSON

- **Reads:** `catalogue/`; `scenarios/`; `campaigns/`; `trades/`
- **Writes:** `matlab_sils/data/*/<id>.json`; `matlab_sils/data/families.json`; `matlab_sils/data/classes.json`
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

This registry: every command, what it does before it does it; `why` names the command that writes a file; `status` puts the evidence debt first (unconfirmed algorithms, synthetic parts, unselectable catalogue models, engine-twin disagreements, failing design-loop checks); `docs` writes docs/COMMANDS.md.

    python3 tools/trinetra.py list | explain <command> | why <file> | status | docs [--check]

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

## kit.py kit

The tool as a team member gets it: the programs beside exactly the files they read (data, cases, the ephemeris), with VERSION and the first-run documents; --files-only leaves the programs out, for the Python package.

    python3 tools/kit.py [--bin DIR] [--out DIR] [--files-only]

**Steps**

1. empty the output folder
2. copy the release-built programs (the desktop app named for Windows when it is a .exe)
3. copy data/, cases/ and the DE440 ephemeris
4. write VERSION with each part's version, and the documents

- **Reads:** `engine/target/<target>/release/ (or --bin)`; `matlab_sils/data, matlab_sils/cases, the ephemeris`; `docs/START_HERE.md, FIRST_RUN.md, COMMANDS.md`
- **Writes:** `dist/kit/trinetra-adcs-<version>/ (or --out)`
- **Starts:** nothing

## make_icon.py make-icon

Draw the desktop app's icon (three eyes on an orbit) as PNG, ICO and ICNS; run only to change it.

    python3 tools/make_icon.py

**Steps**

1. draw the icon at 1024 px
2. write the PNG, the multi-size ICO and the ICNS

- **Reads:** nothing
- **Writes:** `engine/crates/trinetra-app/icon/trinetra.png, .ico, .icns`
- **Starts:** nothing

## macapp.py macapp

The macOS desktop app, TRI-NETRA ADCS.app, from a macOS kit: the app program in Contents/MacOS, the kit's data in Contents/Resources, the icon, Info.plist and PkgInfo. The release workflow signs it and zips it.

    python3 tools/macapp.py --kit <macOS kit> [--out DIR]

**Steps**

1. refuse a kit with no trinetra-app program
2. empty <out>/TRI-NETRA ADCS.app
3. copy the program, the data and the documents, the icon
4. write Info.plist (name, version, bundle id) and PkgInfo

- **Reads:** the macOS kit; `engine/crates/trinetra-app/icon/trinetra.icns`
- **Writes:** `dist/app/TRI-NETRA ADCS.app (or --out)`
- **Starts:** nothing

## build_wheel.py build-wheel

One Python package for every computer: the front end (python/trinetra_adcs), the engine and the desktop app for each system given, and the data; it installs trinetra-adcs and trinetra-adcs-app and compiles nothing.

    python3 tools/build_wheel.py --kit <files-only kit> --bin SYSTEM=DIR... [--out DIR] | --selftest

**Steps**

1. check the kit and the systems
2. write every file with its executable bit and its SHA-256 in RECORD
3. write METADATA, WHEEL and the entry points
4. verify the RECORD against the wheel

- **Reads:** `python/trinetra_adcs/`; a files-only kit; each system's adcs and trinetra-app
- **Writes:** `dist/trinetra_adcs-<version>-py3-none-any.whl (or --out)`
- **Starts:** nothing

## adcs-sim scenario-schema

The scenario schema for the MATLAB twin: every scenario key the engine reads, with its type, written from the engine's own table so the twin refuses exactly what the engine refuses.

    ADCS_WRITE_SCHEMA=1 cargo test -p adcs-sim schema_json   (in engine/)

**Steps**

1. serialise the engine's scenario schema (engine/crates/adcs-sim/src/schema.rs)
2. write it when asked; otherwise fail the test if the file differs

- **Reads:** `engine/crates/adcs-sim/src/schema.rs`
- **Writes:** `matlab_sils/data/scenario_schema.json`
- **Starts:** cargo

## pcode.py pcode

Pseudocode v2 (docs/PSEUDOCODE_V2.md): check a file (units, types, every output set, no recursion), run a function in the interpreter, and write or check everything it makes: the physics (spec/physics/*.pc) as a Rust crate and a MATLAB package with test vectors from the interpreter, the language's self-test the same way, the flight algorithms (fsw/pseudocode/*.pc) as test vectors for the C and Rust flight software, the MATLAB runtime, and the browser checker page. gen --check also holds the physics to spec/plan/physics.toml; fixtures runs the seeded, sourced test vectors of the physics rows.

    python3 tools/pcode.py check FILE... | run FILE... --fn NAME --args JSON | gen [--check] | fixtures

**Steps**

1. check the pseudocode
2. translate it to Rust and MATLAB
3. draw test vectors from the interpreter
4. write or compare every generated file

- **Reads:** `spec/physics/`; `design/pcode_selftest/`; `fsw/pseudocode/`; `design/js/pcode.js`; `design/js/pcode_gen.js`; `design/js/pcode_check.template.html`; `spec/plan/physics.toml`; `spec/plan/seed_content.toml`
- **Writes:** `engine/crates/adcs-physics/`; `engine/crates/pcode-selftest/`; `matlab_sils/+asils/+physics/`; `matlab_sils/+asils/+pcselftest/`; `matlab_sils/+asils/+pc/`; `matlab_sils/data/physics_vectors.json`; `matlab_sils/data/pcselftest_vectors.json`; `fsw/tests/pcode_vectors.txt`; `design/pcode_checker.html`
- **Starts:** node
