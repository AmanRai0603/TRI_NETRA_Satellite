# Commands

> Generated from `docs/commands.toml` by `python3 tools/trinetra.py docs`; never edited by hand.
> `python3 tools/trinetra.py explain <command>` prints one of these, and `dry-run <command>` says it
> as what would happen and runs nothing (`--dry-run` on `engine.py` and `pipeline.py` too).

| command | what it does |
|---|---|
| [`adcs run`](#adcs-run) | Fly one scenario on one case: the plant, the environment and the POP orbit in Rust, the flight software (C or Rust, or a virtual OBC) behind the byte HAL, then the metrics and their verdicts. |
| [`adcs params`](#adcs-params) | The flight software's parameter blob (adcs-fswcfg/1) for a scenario, as an OBC boots from it. |
| [`adcs size`](#adcs-size) | The demand survey on the case's orbit, then every actuator option sized to it (magnetorquers, fluid loop, RCS, wheels, CMG, VSCMG). |
| [`adcs parity`](#adcs-parity) | Fly the same scenario with two flight-software targets and report the largest difference in attitude and rate; bit-identical is the expected answer for C and Rust, and any difference exits with status 1. |
| [`adcs results`](#adcs-results) | The results store: every run with its provenance, one line each; one run in full; runs kept (pinned) or thinned to their manifest when old; a run as one share file, and back; any read-only question to its SQLite index; the runs another engine or other inputs flew; a stored run flown again with what changed. |
| [`adcs figures`](#adcs-figures) | A run's figures, the same for an engine run and a MATLAB twin run (both keep manifest.json and channels.csv with the same columns): attitude, disturbance torques, actuators and power, the Sun spin when the run spun up, and with --full the environment, ground track and mode timeline. Drawn by adcs-plot, the one plotting module, as SVG or PDF. |
| [`adcs report`](#adcs-report) | A run's report: what it flew (scenario, case, product, flight software, seed, duration, engine, result id, input fingerprints), every metric against its requirement with the verdict, and every figure of `adcs figures --full`; as HTML with the figures inline (a print stylesheet, so a browser prints it to PDF) and as PDF. |
| [`adcs plot`](#adcs-plot) | Figures described as JSON (panels stacked or in a grid; line, step, scatter, histogram and horizontal-bar series; reference lines, notes, legends; linear or log axes), drawn by adcs-plot. The report tools describe every campaign, comparison, trade and solution figure this way; the schema is in engine/crates/adcs-plot/src/lib.rs. |
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
| [`carry_over.py carry_over`](#carry_overpy-carry_over) | Carries everything the repository already says into a seeded design folder's node files (docs/RELEASE_PLAN.md P8): the spec's seed content into the nodes' own fields, each physics relation as pseudocode with what it calls and where it runs, the case keys and suppliers of declared values, the KPIs' senses and metrics, the algorithms' parameters, the tree's notes; names the internal layer-3 rows and adds the rows a discipline had no node for, from design/carry.toml. Each item is marked with its origin; a field already written is never replaced; what is still missing is listed in each node (status.gaps) with its owner team. --check seeds and carries into a temporary folder and checks every file and rule. |
| [`check_all.py check-all`](#check_allpy-check-all) | Every check the repository has, one line each with a verdict: the Python tests, the generated files, the C and Rust flight software, the engine, the specification package and the stored design loop; --octave adds the MATLAB twin's suites. |
| [`kit.py kit`](#kitpy-kit) | The tool as a team member gets it: the programs beside exactly the files they read (data, cases, the ephemeris), with VERSION and the first-run documents; --files-only leaves the programs out, for the Python package. |
| [`make_icon.py make-icon`](#make_iconpy-make-icon) | Draw the desktop app's icon (three eyes on an orbit) as PNG, ICO and ICNS; run only to change it. |
| [`macapp.py macapp`](#macapppy-macapp) | The macOS desktop app, TRI-NETRA ADCS.app, from a macOS kit: the app program in Contents/MacOS, the kit's data in Contents/Resources, the icon, Info.plist and PkgInfo. The release workflow signs it and zips it. |
| [`build_wheel.py build-wheel`](#build_wheelpy-build-wheel) | One Python package for every computer: the front end (python/trinetra_adcs), the engine and the desktop app for each system given, and the data; it installs trinetra-adcs and trinetra-adcs-app and compiles nothing. |
| [`adcs-sim scenario-schema`](#adcs-sim-scenario-schema) | The scenario schema for the MATLAB twin: every scenario key the engine reads, with its type, written from the engine's own table so the twin refuses exactly what the engine refuses. |
| [`pcode.py pcode`](#pcodepy-pcode) | Pseudocode v2 (docs/PSEUDOCODE_V2.md): check a file (units, types, every output set, no recursion), run a function in the interpreter, and write or check everything it makes: the physics (spec/physics/*.pc) as a Rust crate and a MATLAB package with test vectors from the interpreter, the language's self-test the same way, the flight algorithms (fsw/pseudocode/*.pc) as test vectors for the C and Rust flight software, the MATLAB runtime, and the browser checker page. gen --check also holds the physics to spec/plan/physics.toml; fixtures runs the seeded, sourced test vectors of the physics rows. |
| [`pages.py pages`](#pagespy-pages) | The offline pages (TRI-NETRA Files today; the node and group apps next): each one HTML file that runs from disk, with SQLite in WebAssembly, the fonts and the component set inside it and nothing loaded from anywhere. check builds them into a scratch folder and refuses a vendored file that is not the pinned one, a page that makes its own controls or styles, and anything that would load from an outside host. |
| [`groupcode.py groupcode`](#groupcodepy-groupcode) | Each group's code, generated from its nodes (docs/RELEASE_PLAN.md P10, docs/GENERATORS.md): wire puts every computing row's pseudocode into a module per group (design/groups/, with shared.pc for what several groups call alike, and a wire file naming each row's function and its own test vectors), from the merged releases or from the design as seeded and carried; gen translates them to Rust (engine/crates/adcs-groups), WebAssembly (adcs-groups-wasm) and MATLAB (+asils/+groups) with the interpreter's vectors; test runs the Rust against the vectors and the nodes' own test vectors; deliver builds a test app per group. |
| [`group.py group`](#grouppy-group) | The structure of a design folder (structure/ and nodes/, as tools/seed_design.py writes it and the group app changes it), checked from Python: every node in exactly one group with its node file saying the same group, stage, label and state; every edge kept by the group of the node that reads, from a node that exists and is not archived; every author, contract and stage owner about the group's own nodes and people; no structure action left unfinished. The same rules as the group app's own check (design/js/structure.js), written a second time. verify checks every group's latest sealed release (tools/release.py) and that it is still the group's; merge takes every latest release into design.tndb with the catalogue of every output (the contract at every group boundary); impact lists who reads a node, across groups. |
| [`delivery.py delivery`](#deliverypy-delivery) | Test, deliver, accept, ship (docs/RELEASE_PLAN.md P12, docs/DELIVERY.md): a group's sealed release delivered in its wave's order, with its generated code checked to be the release's, its tests run, its test app and a note; its lead accepts it in the group app; the shipping record says, group by group, who accepted which version, and which group ships visibly UNCONFIRMED and why. |
| [`evaluate.py evaluate`](#evaluatepy-evaluate) | Every row of the design evaluated for a case, or shown as not computed and why; every KPI closure answered or blocked by name (docs/RELEASE_PLAN.md P13). A row's value is stated by the case (from design.tndb, in SI), computed by its pseudocode in the interpreter from the rows its inputs name, or supplied as evidence by the selected design's Monte Carlo; a closure compares its requirement with its evidence or its analysis row in the requirement's sense. |
| [`end_to_end.py end-to-end`](#end_to_endpy-end-to-end) | Both cases through everything from the design database (docs/RELEASE_PLAN.md P13, docs/END_TO_END.md): the design loop and the case's campaigns with every engine run reading its inputs from design.tndb alone (TRINETRA_DESIGN), every number held to the one stored before, every row and closure evaluated, the traceability. |
| [`release_notes.py release-notes`](#release_notespy-release-notes) | The release notes' generated part (docs/RELEASE_PLAN.md P14): what ships, group by group, from the shipping record (tools/delivery.py ship), and every known gap from docs/ADCS_GAPS.md, written into docs/RELEASE_NOTES.md between its markers. |
| [`manual.py manual`](#manualpy-manual) | The apps' manual (design/manual/: a guide per role, the journey of a node, the glossary, the guide to TRI-NETRA Files, and the tours in tours.toml) written as design/js/manual.js, which every app opens in place with Help, and the journey diagram as design/manual/journey.svg. --check says whether both are current and holds the rules: every page starts with its one line (the explanation standard), every tour target is in its app, every field, choice and set of checks in the apps has help beside it. |
| [`node_catalog.py node_catalog`](#node_catalogpy-node_catalog) | The node app's catalogue (design/js/node_catalog.js), written from the spec: the units and the quantities each measures, the physics relations, the sources, the evidence metrics and rungs, the provenance and belief words, the tags and every row of the tree with its label, kind, quantity, unit and layer. The node app offers these as its choices, so they are never typed by hand. --check says whether the committed file is current. |
| [`release.py release`](#releasepy-release) | Sealed group releases (releases/<group>-<version>.tnrel, as the group app seals them), checked from Python: each file passes tools/tndb.py check and is named for its group and version; every node's fingerprint, body fingerprint and the release's fingerprint are the SHA-256 of what they cover; its nodes are the group's as sealed, each confirmed or unconfirmed with why; a confirmed node was checked by someone other than its author and, when it computes, has a test vector from outside the code; the lead's seal names the version and the fingerprint. The same rules as design/js/release.js, written a second time. |
| [`drive_pack.py drive-pack`](#drive_packpy-drive-pack) | The Drive pack: the two offline apps (TRI-NETRA Files, TRI-NETRA Group) and the design seeded from the spec (20 group files, 734 node files), in the layout the shared Drive folder of the design takes, with a README. CI builds it on every push as the trinetra-drive-pack artifact. |

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
- **Checks:** the scenario id, the case keys and every --set exist and are in range (refused by name, never guessed); the metrics against the case's requirements
- **Undo:** Delete the run folder it wrote (the store's index forgets it at the next `adcs results list`); the inputs it kept by fingerprint are shared and harmless to leave.
- **Code:** `engine/crates/adcs-cli/`

## adcs params

The flight software's parameter blob (adcs-fswcfg/1) for a scenario, as an OBC boots from it.

    adcs params <scenario> [--case F] [--set k=v]... --out blob.bin

**Steps**

1. build the configuration as `adcs run` does
2. encode the parameters with their CRC-32

- **Reads:** the scenario, the case, the product
- **Writes:** the --out file
- **Starts:** nothing
- **Checks:** the scenario, case and product exist; the blob's CRC-32
- **Undo:** Delete the --out file.
- **Code:** `engine/crates/adcs-cli/`

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
- **Checks:** the case id; every bought part against the datasheet catalogue
- **Undo:** Delete the sized/ folder it wrote.
- **Code:** `engine/crates/adcs-cli/`

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
- **Checks:** the two flight-software targets give the same attitude and rate within the stated tolerance
- **Undo:** Nothing to undo: it writes nothing.
- **Code:** `engine/crates/adcs-cli/`

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
7. stale: compare every run's engine source, case, scenario and product fingerprints, and the design it flew (when one was in use), with today's and name what differs; exit 1 when any run is stale
8. refly: fly the run again from its kept inputs into <store>/refly/<run> and print every metric stored against now

- **Reads:** `matlab_sils/store/results_engine/ (or DIR)`; the case, scenario and product files the runs name (stale); the run's kept inputs (refly)
- **Writes:** `list, query, stale: <folder>/.adcs-index.sqlite; pin: <run>/PINNED; thin: removes the bulk files; export: the .trinetra file; import: the folder; refly: <store>/refly/<run>/ (or --out)`
- **Starts:** nothing
- **Checks:** each stored run's provenance against the engine and inputs that made it (stale when they changed)
- **Undo:** list/query/stale: nothing (the index is rebuilt from the runs). pin: delete <run>/PINNED. thin: a thinned time series is gone; fly the run again.
- **Code:** `engine/crates/adcs-cli/`

## adcs figures

A run's figures, the same for an engine run and a MATLAB twin run (both keep manifest.json and channels.csv with the same columns): attitude, disturbance torques, actuators and power, the Sun spin when the run spun up, and with --full the environment, ground track and mode timeline. Drawn by adcs-plot, the one plotting module, as SVG or PDF.

    adcs figures RUN_DIR --out DIR [--format svg|pdf] [--full] [--prefix P]

**Steps**

1. read the run's manifest.json and channels.csv (a thinned run is refused with how to fly it again)
2. build each figure, with the case's requirement lines from the manifest's metrics
3. thin long series to the smallest and largest sample per pixel column
4. write <prefix>_<n>_<name>.svg (or .pdf) into DIR and print each file

- **Reads:** `RUN_DIR/manifest.json`; `RUN_DIR/channels.csv`
- **Writes:** `DIR/<prefix>_<n>_<name>.svg|pdf (prefix: the run folder's name)`
- **Starts:** nothing
- **Checks:** the run folder holds manifest.json and channels.csv (a thinned run is refused with `adcs results refly`); every figure is well-formed SVG or PDF
- **Undo:** Delete the files it wrote into DIR; the run is only read.
- **Code:** `engine/crates/adcs-cli/`

## adcs report

A run's report: what it flew (scenario, case, product, flight software, seed, duration, engine, result id, input fingerprints), every metric against its requirement with the verdict, and every figure of `adcs figures --full`; as HTML with the figures inline (a print stylesheet, so a browser prints it to PDF) and as PDF.

    adcs report RUN_DIR [--out DIR]

**Steps**

1. read the run as `adcs figures` does
2. draw the full figure set
3. write report.html (figures inline as SVG) and report.pdf (a page of provenance and verdicts, then a page per figure)

- **Reads:** `RUN_DIR/manifest.json`; `RUN_DIR/channels.csv`
- **Writes:** `DIR (default RUN_DIR)/report.html`; `DIR (default RUN_DIR)/report.pdf`
- **Starts:** nothing
- **Checks:** as for figures; the verdict table is the manifest's own metrics, nothing recomputed
- **Undo:** Delete report.html and report.pdf from the run folder (or --out); the run is only read.
- **Code:** `engine/crates/adcs-cli/`

## adcs plot

Figures described as JSON (panels stacked or in a grid; line, step, scatter, histogram and horizontal-bar series; reference lines, notes, legends; linear or log axes), drawn by adcs-plot. The report tools describe every campaign, comparison, trade and solution figure this way; the schema is in engine/crates/adcs-plot/src/lib.rs.

    adcs plot SPEC.json --out FILE.svg|FILE.pdf

**Steps**

1. read the description; an unknown kind, scale or layout is refused by name
2. lay out the panels, ticks and series
3. write one SVG (several figures: FILE_1.svg, FILE_2.svg, ...) or one PDF with a page per figure

- **Reads:** `SPEC.json`
- **Writes:** the --out file
- **Starts:** nothing
- **Checks:** the JSON description: every kind, scale, layout, limit and grid named and valid (anything else refused with exit 2, by name)
- **Undo:** Delete the --out file(s).
- **Code:** `engine/crates/adcs-cli/`

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
- **Checks:** every build with warnings as errors; the flight software's tests
- **Undo:** Delete the build folders (fsw/build/, fsw-rs/target/, engine/target/); nothing else changes.
- **Code:** `tools/engine.py`

## engine.py run

Fly scenarios on the engine in parallel (every scenario when none is named).

    python3 tools/engine.py run [scenario ...] [--fsw c|rust] [--jobs N] [--seed S]

**Steps**

1. one `adcs run` per scenario, on --jobs processes
2. print each run's verdicts

- **Reads:** `matlab_sils/data/scenarios/`
- **Writes:** `matlab_sils/store/results_engine/<scenario>/`
- **Starts:** adcs run
- **Checks:** as `adcs run`, for each scenario
- **Undo:** Delete the run folder it wrote (the store's index forgets it at the next `adcs results list`); the inputs it kept by fingerprint are shared and harmless to leave.
- **Code:** `tools/engine.py`

## engine.py mc

A seed sweep of one scenario: the same scenario flown with N sensor-noise seeds.

    python3 tools/engine.py mc <scenario> --seeds N [--fsw c|rust] [--jobs N]

**Steps**

1. one `adcs run` per seed
2. summarise every metric over the seeds

- **Reads:** the scenario and its case
- **Writes:** `matlab_sils/store/results_engine/mc_<scenario>/summary.json`
- **Starts:** adcs run
- **Checks:** the seed sweep's spread against the scenario's requirements
- **Undo:** Delete matlab_sils/store/results_engine/mc_<scenario>/.
- **Code:** `tools/engine.py`

## engine.py fsw-parity

C against Rust flight software on the same loop, same bytes, scenario by scenario.

    python3 tools/engine.py fsw-parity [scenario ...] [--duration S]

**Steps**

1. fly each scenario with the C and the Rust flight software (`adcs parity`)
2. print the largest differences

- **Reads:** the scenarios
- **Writes:** nothing
- **Starts:** adcs parity
- **Checks:** the C and Rust flight software give the same numbers, bit for bit where stated
- **Undo:** Nothing to undo: it writes nothing.
- **Code:** `tools/engine.py`

## engine.py twin-parity

The engine's metrics against the MATLAB twin's, scenario by scenario and metric by metric.

    python3 tools/engine.py twin-parity

**Steps**

1. read each scenario's engine and twin manifests and pair their metrics
2. tabulate verdict agreement and ratios, and write the ledger

- **Reads:** `matlab_sils/store/results_engine/`; `matlab_sils/store/results/`
- **Writes:** `results/ENGINE_PARITY.md`; `results/engine_parity.json`
- **Starts:** nothing
- **Checks:** the engine's metrics against the MATLAB twin's, within each metric's tolerance
- **Undo:** It writes generated files only: `git checkout -- <file>` puts back the committed one, or run it again once its source is as you want it.
- **Code:** `tools/engine.py`

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
- **Checks:** the flight software as a process and as QEMU firmware against the in-process run
- **Undo:** It writes generated files only: `git checkout -- <file>` puts back the committed one, or run it again once its source is as you want it.
- **Code:** `tools/engine.py`

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
- **Checks:** the dispatched configuration flies its mission on the engine
- **Undo:** Delete dist/dispatch/<case>/.
- **Code:** `tools/engine.py`

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
- **Checks:** each campaign's statistics against the twin's
- **Undo:** Delete the campaign folders it wrote; It writes generated files only: `git checkout -- <file>` puts back the committed one, or run it again once its source is as you want it.
- **Code:** `tools/engine.py`

## engine.py campaign-ledger

Rewrite the engine-vs-twin campaign ledger from the stored summaries, without flying anything.

    python3 tools/engine.py campaign-ledger

**Steps**

1. read every campaign summary, engine and twin
2. write the ledger

- **Reads:** `matlab_sils/store/results_engine/campaigns/`; `matlab_sils/store/results/`
- **Writes:** `results/ENGINE_CAMPAIGNS.md`; `results/engine_campaigns.json`
- **Starts:** nothing
- **Checks:** the stored summaries are complete
- **Undo:** It writes generated files only: `git checkout -- <file>` puts back the committed one, or run it again once its source is as you want it.
- **Code:** `tools/engine.py`

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
- **Checks:** every tick within its deadline; soft OILS against SILS
- **Undo:** Delete matlab_sils/store/results_engine/soft_oils/; It writes generated files only: `git checkout -- <file>` puts back the committed one, or run it again once its source is as you want it.
- **Code:** `tools/engine.py`

## engine.py oils-ledger

Rewrite the soft-OILS ledger from the stored runs, without flying anything.

    python3 tools/engine.py oils-ledger

**Steps**

1. read the stored SILS and OILS runs
2. write the ledger

- **Reads:** `matlab_sils/store/results_engine/soft_oils/`
- **Writes:** `results/SOFT_OILS.md`; `results/soft_oils.json`
- **Starts:** nothing
- **Checks:** the stored runs are complete
- **Undo:** It writes generated files only: `git checkout -- <file>` puts back the committed one, or run it again once its source is as you want it.
- **Code:** `tools/engine.py`

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
- **Checks:** every mode and option flown, judged against the case
- **Undo:** Delete matlab_sils/store/solutions_engine/; It writes generated files only: `git checkout -- <file>` puts back the committed one, or run it again once its source is as you want it.
- **Code:** `tools/engine.py`

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
- **Checks:** each node's inputs, outputs and rules (matlab_sils/data/pipeline/nodes.json); each decision recomputable
- **Undo:** Delete matlab_sils/store/pipeline/<case>/ and dist/dispatch/<case>/.
- **Code:** `tools/pipeline.py`

## report.py report

The results report from the filed runs: figures, verdict tables, the summary and the results documents. It reads; it never flies.

    python3 tools/report.py

**Steps**

1. read every filed run, campaign and trade
2. draw each run's figures with `adcs figures`, and each campaign, comparison, trade and solution figure from its JSON description with `adcs plot` (SVG)
3. write the page, the summary and the documents

- **Reads:** `matlab_sils/store/results/`; `matlab_sils/store/trades/`
- **Writes:** `results/figures/*.svg`; `results/index.html`; `results/summary.json`; `docs/RESULTS.md`; `docs/SELECTION.md`
- **Starts:** adcs figures; adcs plot
- **Checks:** every filed run it reads exists and is current
- **Undo:** It writes generated files only: `git checkout -- <file>` puts back the committed one, or run it again once its source is as you want it.
- **Code:** `tools/report.py`

## vv_report.py vv-report

The downloadable V&V report: the template filled from the filed results, as self-contained HTML and as a PDF.

    python3 tools/vv_report.py

**Steps**

1. collect every source that exists (a missing one is named, never invented)
2. draw its figures with `adcs plot`, inlined as SVG
3. fill tools/templates/vv_report.html
4. print it to PDF with headless Chromium

- **Reads:** `matlab_sils/store/pipeline/`; `matlab_sils/store/results_engine/`; `matlab_sils/store/results/`; `results/*.json`
- **Writes:** `results/vv/TRINETRA_ADCS_VV_report.html`; `results/vv/vv_artifact.html`; `dist/TRINETRA_ADCS_VV_report.pdf`
- **Starts:** adcs plot; chromium (headless)
- **Checks:** every section of the template is filled from a filed result
- **Undo:** It writes generated files only: `git checkout -- <file>` puts back the committed one, or run it again once its source is as you want it.
- **Code:** `tools/vv_report.py`

## pack_matlab.py pack-matlab

The downloadable MATLAB SILS zip, deterministic (sorted files, fixed timestamps), with its SHA-256 manifest.

    python3 tools/pack_matlab.py

**Steps**

1. collect matlab_sils/ and the documents
2. write the zip beside its name, then rename it into place

- **Reads:** `matlab_sils/`; `docs/`
- **Writes:** `dist/TRINETRA_ADCS_SILS_matlab_<version>.zip`
- **Starts:** nothing
- **Checks:** the zip is deterministic: its SHA-256 manifest
- **Undo:** Delete the zip from dist/.
- **Code:** `tools/pack_matlab.py`

## pack_flight.py pack-flight

The flight software and Rust engine zip, in the repository's layout so it builds as unpacked.

    python3 tools/pack_flight.py

**Steps**

1. collect fsw/, fsw-rs/, engine/, the data and the tools
2. write the zip beside its name, then rename it into place

- **Reads:** `fsw/`; `fsw-rs/`; `engine/`; `matlab_sils/data`; `tools/`
- **Writes:** `dist/TRINETRA_ADCS_flight_engine_<version>.zip`
- **Starts:** nothing
- **Checks:** the zip builds as unpacked
- **Undo:** Delete the zip from dist/.
- **Code:** `tools/pack_flight.py`

## gen_fsw_params.py gen-fsw-params

The flight software's parameter and table sources, C and Rust, from their one definition. --check says which generated file is stale, and changes nothing.

    python3 tools/gen_fsw_params.py [--check]

**Steps**

1. read fsw/params/params.toml and the IGRF table
2. write the C header and source and the Rust module

- **Reads:** `fsw/params/params.toml`; `matlab_sils/data/igrf13coeffs.txt`
- **Writes:** `fsw/include/adcs_params.h`; `fsw/src/adcs_params.c`; `fsw-rs/src/params.rs`; `matlab_sils/data/igrf13.json`; `fsw/include/adcs_igrf13.h`; `fsw-rs/src/igrf13.rs`; `fsw/pseudocode/02_igrf13.pc`
- **Starts:** nothing
- **Checks:** --check: every generated parameter file is its definition
- **Undo:** It writes generated files only: `git checkout -- <file>` puts back the committed one, or run it again once its source is as you want it.
- **Code:** `tools/gen_fsw_params.py`

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
- **Checks:** the deepest call path fits the stack the linker script reserves
- **Undo:** Nothing to undo: it writes nothing.
- **Code:** `tools/fsw_stack.py`

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
- **Checks:** every row exactly once, against the spec's expected ids
- **Undo:** Nothing to undo: it writes nothing.
- **Code:** `tools/design_rows.py`

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
- **Checks:** every row of the tree in exactly one group; the boundary rules
- **Undo:** Nothing to undo: it writes nothing.
- **Code:** `tools/groups.py`

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
- **Checks:** a file's format, version, tables and columns against design/schema.toml; caps on size and pictures
- **Undo:** check: nothing, unless it upgraded an older file, which keeps the original beside it (put it back by renaming). gen: It writes generated files only: `git checkout -- <file>` puts back the committed one, or run it again once its source is as you want it.
- **Code:** `tools/tndb.py`

## seed_design.py seed-design

Seeds the design files from the spec: a group file for each of the 20 groups, a node file for each of the 734 rows (the 82 the spec seeds with their content, the rest as shells), and the starting design database. Never overwrites; --check seeds into a temporary folder and checks every file.

    python3 tools/seed_design.py [--out DIR | --check]

**Steps**

1. write the group files, the node files and the design database from the spec
2. check every file against the schema

- **Reads:** `design/groups.toml`; `design/schema.toml`; `spec/plan/`
- **Writes:** `build/design/`
- **Starts:** nothing
- **Checks:** the group map holds; every file it writes checks
- **Undo:** Delete the folder it wrote (build/design/ or --out). Never run it over a design folder people edit.
- **Code:** `tools/seed_design.py`

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
- **Checks:** every part carries the one version
- **Undo:** It writes generated files only: `git checkout -- <file>` puts back the committed one, or run it again once its source is as you want it.
- **Code:** `tools/version.py`

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
- **Checks:** each mutant is caught by a test
- **Undo:** It writes generated files only: `git checkout -- <file>` puts back the committed one, or run it again once its source is as you want it.
- **Code:** `tools/mutation.py`

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
- **Checks:** every stated requirement is checked by a metric or says why not
- **Undo:** It writes generated files only: `git checkout -- <file>` puts back the committed one, or run it again once its source is as you want it.
- **Code:** `tools/trace.py`

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
- **Checks:** the budget's total against the required pointing error
- **Undo:** It writes generated files only: `git checkout -- <file>` puts back the committed one, or run it again once its source is as you want it.
- **Code:** `tools/pointing_budget.py`

## export_catalogue.py export-catalogue

The catalogue, scenarios, campaigns and trades from TOML to the JSON the MATLAB twin and the engine read. --check says which JSON has drifted from its TOML, and changes nothing.

    python3 tools/export_catalogue.py [--check]

**Steps**

1. read every TOML source
2. write (or, with --check, compare) the JSON

- **Reads:** `catalogue/`; `scenarios/`; `campaigns/`; `trades/`
- **Writes:** `matlab_sils/data/*/<id>.json`; `matlab_sils/data/families.json`; `matlab_sils/data/classes.json`
- **Starts:** nothing
- **Checks:** --check: every JSON is its TOML
- **Undo:** It writes generated files only: `git checkout -- <file>` puts back the committed one, or run it again once its source is as you want it.
- **Code:** `tools/export_catalogue.py`

## nodes_doc.py nodes-doc

docs/NODES.md and docs/CATALOGUE.md from the node registry and the datasheet catalogue.

    python3 tools/nodes_doc.py

**Steps**

1. read the node registry and the catalogue
2. write both documents

- **Reads:** `matlab_sils/data/pipeline/nodes.json`; `matlab_sils/data/catalogue/`
- **Writes:** `docs/NODES.md`; `docs/CATALOGUE.md`
- **Starts:** nothing
- **Checks:** the docs are their registries
- **Undo:** It writes generated files only: `git checkout -- <file>` puts back the committed one, or run it again once its source is as you want it.
- **Code:** `tools/nodes_doc.py`

## components_doc.py components-doc

docs/COMPONENTS.md: every sensor and actuator, the model the SILS flies, and its processing chain.

    python3 tools/components_doc.py

**Steps**

1. read the component files
2. write the document

- **Reads:** `catalogue/components/`
- **Writes:** `docs/COMPONENTS.md`
- **Starts:** nothing
- **Checks:** the doc is its registry
- **Undo:** It writes generated files only: `git checkout -- <file>` puts back the committed one, or run it again once its source is as you want it.
- **Code:** `tools/components_doc.py`

## catalogue.py catalogue

Re-derive each bought wheel's and CMG's modelling block from its datasheet numbers (never guessed), then the catalogue document.

    python3 tools/catalogue.py

**Steps**

1. derive every model's engine parameters by rule
2. mark a model not selectable when its datasheet lacks a number the selection needs

- **Reads:** `matlab_sils/data/catalogue/`
- **Writes:** `matlab_sils/data/catalogue/*.json`; `docs/CATALOGUE.md`
- **Starts:** nothing
- **Checks:** each modelling block re-derived from its datasheet numbers, never guessed
- **Undo:** It writes generated files only: `git checkout -- <file>` puts back the committed one, or run it again once its source is as you want it.
- **Code:** `tools/catalogue.py`

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
- **Checks:** each design-loop decision recomputed from its stored inputs
- **Undo:** It writes generated files only: `git checkout -- <file>` puts back the committed one, or run it again once its source is as you want it.
- **Code:** `tools/verify_nodes.py`

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
- **Checks:** each stored run against the case as it is now
- **Undo:** --dry-run changes nothing. Otherwise run it again against the earlier case files to restore the verdicts.
- **Code:** `tools/rescore.py`

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
- **Checks:** the multipliers lie inside the unit circle (the periodic loop is stable)
- **Undo:** Delete the floquet.json it wrote.
- **Code:** `tools/floquet.py`

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
- **Checks:** every job of the twin's matrix finishes and passes
- **Undo:** Delete the store folders it wrote.
- **Code:** `tools/run_matrix.py`

## fswcfg.py fswcfg

Decode and check a flight-software parameter blob (adcs-fswcfg/1) and print every field as JSON.

    python3 tools/fswcfg.py <blob>

**Steps**

1. check the magic, length and CRC-32
2. decode each field in table order

- **Reads:** the blob; `fsw/params/params.toml`
- **Writes:** nothing
- **Starts:** nothing
- **Checks:** the blob's format and CRC-32
- **Undo:** Nothing to undo: it writes nothing.
- **Code:** `tools/fswcfg.py`

## trinetra.py explain

This registry: every command, what it does before it does it; `why` names the command that writes a file; `status` puts the evidence debt first (unconfirmed algorithms, synthetic parts, unselectable catalogue models, engine-twin disagreements, failing design-loop checks); `docs` writes docs/COMMANDS.md.

    python3 tools/trinetra.py list | explain <command> | why <file> | status | docs [--check]

**Steps**

1. read docs/commands.toml
2. print, explain, or write the document

- **Reads:** `docs/commands.toml`
- **Writes:** `docs/COMMANDS.md (docs only)`
- **Starts:** nothing
- **Checks:** docs --check: docs/COMMANDS.md is this registry
- **Undo:** docs: It writes generated files only: `git checkout -- <file>` puts back the committed one, or run it again once its source is as you want it. The rest writes nothing.
- **Code:** `tools/trinetra.py`

## carry_over.py carry_over

Carries everything the repository already says into a seeded design folder's node files (docs/RELEASE_PLAN.md P8): the spec's seed content into the nodes' own fields, each physics relation as pseudocode with what it calls and where it runs, the case keys and suppliers of declared values, the KPIs' senses and metrics, the algorithms' parameters, the tree's notes; names the internal layer-3 rows and adds the rows a discipline had no node for, from design/carry.toml. Each item is marked with its origin; a field already written is never replaced; what is still missing is listed in each node (status.gaps) with its owner team. --check seeds and carries into a temporary folder and checks every file and rule.

    python3 tools/carry_over.py DIR | --check | --report DIR

**Steps**

1. read the registries and design/carry.toml
2. name the internal rows and add the new rows in the group files
3. fill each node file, never replacing a written field
4. list each node's gaps and owner team

- **Reads:** `spec/plan/`; `spec/physics/`; `fsw/pseudocode/`; `catalogue/algorithms/`; `design/carry.toml`; `design/groups.toml`; DIR
- **Writes:** `DIR/structure/*.group.tndb`; `DIR/nodes/*.node.tndb`
- **Starts:** nothing
- **Checks:** --check: every file and structure rule after the carry, design/carry.toml against the code
- **Undo:** Seed a fresh folder (it is for a folder nobody has edited yet); a field already written is never replaced, so carrying again changes nothing.
- **Code:** `tools/carry_over.py`

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
- **Checks:** every check the repository has
- **Undo:** It writes only build folders and generated reports; delete the build folders, `git checkout` the reports.
- **Code:** `tools/check_all.py`

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
- **Checks:** the kit holds exactly the files its programs read
- **Undo:** Delete the kit folder from dist/.
- **Code:** `tools/kit.py`

## make_icon.py make-icon

Draw the desktop app's icon (three eyes on an orbit) as PNG, ICO and ICNS; run only to change it.

    python3 tools/make_icon.py

**Steps**

1. draw the icon at 1024 px
2. write the PNG, the multi-size ICO and the ICNS

- **Reads:** nothing
- **Writes:** `engine/crates/trinetra-app/icon/trinetra.png, .ico, .icns`
- **Starts:** nothing
- **Checks:** the icon files are written at every size
- **Undo:** It writes generated files only: `git checkout -- <file>` puts back the committed one, or run it again once its source is as you want it.
- **Code:** `tools/make_icon.py`

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
- **Checks:** the app bundle's layout
- **Undo:** Delete the .app from dist/.
- **Code:** `tools/macapp.py`

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
- **Checks:** the wheel's RECORD, entry points and executable bits
- **Undo:** Delete the wheel from dist/.
- **Code:** `tools/build_wheel.py`

## adcs-sim scenario-schema

The scenario schema for the MATLAB twin: every scenario key the engine reads, with its type, written from the engine's own table so the twin refuses exactly what the engine refuses.

    ADCS_WRITE_SCHEMA=1 cargo test -p adcs-sim schema_json   (in engine/)

**Steps**

1. serialise the engine's scenario schema (engine/crates/adcs-sim/src/schema.rs)
2. write it when asked; otherwise fail the test if the file differs

- **Reads:** `engine/crates/adcs-sim/src/schema.rs`
- **Writes:** `matlab_sils/data/scenario_schema.json`
- **Starts:** cargo
- **Checks:** the schema is every key the engine reads
- **Undo:** It writes generated files only: `git checkout -- <file>` puts back the committed one, or run it again once its source is as you want it.
- **Code:** `engine/crates/adcs-sim/`

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
- **Checks:** units, types, every output set, no recursion; gen --check: everything generated is current; fixtures: every seeded test vector within its tolerance
- **Undo:** check and run: nothing. gen: It writes generated files only: `git checkout -- <file>` puts back the committed one, or run it again once its source is as you want it.
- **Code:** `tools/pcode.py`

## pages.py pages

The offline pages (TRI-NETRA Files today; the node and group apps next): each one HTML file that runs from disk, with SQLite in WebAssembly, the fonts and the component set inside it and nothing loaded from anywhere. check builds them into a scratch folder and refuses a vendored file that is not the pinned one, a page that makes its own controls or styles, and anything that would load from an outside host.

    python3 tools/pages.py build [--out DIR] | check

**Steps**

1. check the vendored files against design/vendor/VENDOR.toml
2. inline the stylesheet with its fonts, SQLite with its WebAssembly, and the page's modules
3. hold the page to the rules
4. write it (build) or throw it away (check)

- **Reads:** `design/pages/`; `design/js/`; `design/css/`; `design/vendor/`
- **Writes:** `build/pages/`
- **Starts:** nothing
- **Checks:** vendored files pinned, one component set, no outside hosts
- **Undo:** Delete build/pages/.
- **Code:** `tools/pages.py`

## groupcode.py groupcode

Each group's code, generated from its nodes (docs/RELEASE_PLAN.md P10, docs/GENERATORS.md): wire puts every computing row's pseudocode into a module per group (design/groups/, with shared.pc for what several groups call alike, and a wire file naming each row's function and its own test vectors), from the merged releases or from the design as seeded and carried; gen translates them to Rust (engine/crates/adcs-groups), WebAssembly (adcs-groups-wasm) and MATLAB (+asils/+groups) with the interpreter's vectors; test runs the Rust against the vectors and the nodes' own test vectors; deliver builds a test app per group.

    python3 tools/groupcode.py wire [--design DIR] [--check] | gen [--check] | test | deliver [--out DIR]

**Steps**

1. wire: read the design's nodes, group their pseudocode, refuse a function written two ways
2. gen: translate with the pseudocode's translators, draw vectors in the interpreter
3. test: cargo test -p adcs-groups
4. deliver: build the WebAssembly module and one offline test app per group

- **Reads:** `DIR/design.tndb or the seeded and carried design`; `design/groups/`; `design/js/pcode.js, pcode_gen.js`
- **Writes:** `design/groups/`; `engine/crates/adcs-groups/`; `engine/crates/adcs-groups-wasm/`; `matlab_sils/+asils/+groups/`; `matlab_sils/data/groups_vectors.json`; `dist/test-apps/ (deliver)`
- **Starts:** node; cargo
- **Checks:** wire --check and gen --check: everything generated is current; test: translator = interpreter on every drawn vector, and every node's own test vectors within their tolerance
- **Undo:** wire and gen write generated files only: `git checkout -- <file>` puts back the committed one, or run them again. deliver: delete dist/test-apps/.
- **Code:** `tools/groupcode.py`

## group.py group

The structure of a design folder (structure/ and nodes/, as tools/seed_design.py writes it and the group app changes it), checked from Python: every node in exactly one group with its node file saying the same group, stage, label and state; every edge kept by the group of the node that reads, from a node that exists and is not archived; every author, contract and stage owner about the group's own nodes and people; no structure action left unfinished. The same rules as the group app's own check (design/js/structure.js), written a second time. verify checks every group's latest sealed release (tools/release.py) and that it is still the group's; merge takes every latest release into design.tndb with the catalogue of every output (the contract at every group boundary); impact lists who reads a node, across groups.

    python3 tools/group.py check DIR | list DIR | verify DIR | merge DIR [--require-all] | impact DIR NODE

**Steps**

1. read every group file
2. read every node file and compare it with its group
3. list every problem
4. verify: check every group's latest release with tools/release.py and against the folder
5. merge: write every latest release into design.tndb with the catalogue of outputs, keeping the previous one as design.tndb.prev
6. impact: follow the edges from a node across groups

- **Reads:** `DIR/structure/`; `DIR/nodes/`; `DIR/releases/`
- **Writes:** `DIR/design.tndb (merge only)`; `DIR/design.tndb.prev (merge only)`
- **Starts:** nothing
- **Checks:** the structure rules over a design folder; verify: a release against its group; merge: every group's latest release
- **Undo:** check, list, verify, impact: nothing. merge: delete or restore the design.tndb it wrote (it keeps the previous one as design.tndb.prev).
- **Code:** `tools/group.py`

## delivery.py delivery

Test, deliver, accept, ship (docs/RELEASE_PLAN.md P12, docs/DELIVERY.md): a group's sealed release delivered in its wave's order, with its generated code checked to be the release's, its tests run, its test app and a note; its lead accepts it in the group app; the shipping record says, group by group, who accepted which version, and which group ships visibly UNCONFIRMED and why.

    python3 tools/delivery.py deliver DIR [GROUP ...] [--wave A..E] [--no-test] [--out-of-order REASON] | status DIR [--json] | ship DIR [--out FILE] [--require-accepted]

**Steps**

1. deliver: verify the releases (tools/group.py verify) and the wave order
2. merge every release into design.tndb
3. check the generated code is the release's wiring (tools/groupcode.py)
4. run the group's tests (cargo test -p adcs-groups)
5. build its test app
6. write deliveries/<group>-<version>.delivery.json and .md
7. status: released, delivered, accepted, wave by wave
8. ship: the shipping record (JSON and Markdown)

- **Reads:** `DIR/structure/`; `DIR/releases/`; `DIR/deliveries/`; `design/groups/`; `design/groups.toml`
- **Writes:** `DIR/design.tndb (deliver)`; `DIR/deliveries/<group>-<version>.delivery.json, .md, .test-app.html (deliver)`; `FILE and FILE.md (ship --out)`
- **Starts:** cargo test -p adcs-groups; cargo build --target wasm32-unknown-unknown -p adcs-groups-wasm
- **Checks:** the release passes verify; earlier waves delivered first; the generated code is the release's; the tests pass (recorded); an acceptance is by the lead and names the release's and the delivery's fingerprints
- **Undo:** deliver: delete the delivery's three files from deliveries/ (and restore design.tndb.prev). status: nothing. ship: delete the files it wrote. An acceptance is a signature in the group file; a later release needs its own.
- **Code:** `tools/delivery.py`

## evaluate.py evaluate

Every row of the design evaluated for a case, or shown as not computed and why; every KPI closure answered or blocked by name (docs/RELEASE_PLAN.md P13). A row's value is stated by the case (from design.tndb, in SI), computed by its pseudocode in the interpreter from the rows its inputs name, or supplied as evidence by the selected design's Monte Carlo; a closure compares its requirement with its evidence or its analysis row in the requirement's sense.

    python3 tools/evaluate.py DIR [CASE ...] [--out DIR] [--check]

**Steps**

1. read the design (merged releases, else the node files) and the case from design.tndb
2. state the case's values on the rows that declare them, in SI
3. run every row's pseudocode once its inputs have values
4. take each KPI's evidence from the design loop's Monte Carlo
5. answer or block every closure
6. write results/EVALUATION.md and evaluation.json

- **Reads:** `DIR/design.tndb`; `DIR/nodes/`; `spec/plan/kpis.toml`; `spec/plan/case_inputs.toml`; `matlab_sils/store/pipeline/<case>/mc/summary.json`
- **Writes:** `results/EVALUATION.md`; `results/evaluation.json`
- **Starts:** `node design/js/pcode_cli.mjs run`
- **Checks:** every row has a value or a reason; every closure an answer or the row that blocks it; --check: no closure that answered before is blocked now, no stated value lost
- **Undo:** It writes generated files only: `git checkout -- <file>` puts back the committed one.
- **Code:** `tools/evaluate.py`

## end_to_end.py end-to-end

Both cases through everything from the design database (docs/RELEASE_PLAN.md P13, docs/END_TO_END.md): the design loop and the case's campaigns with every engine run reading its inputs from design.tndb alone (TRINETRA_DESIGN), every number held to the one stored before, every row and closure evaluated, the traceability.

    python3 tools/end_to_end.py [CASE ...] [--design DIR] [--no-loop] [--no-campaigns] [--jobs N]

**Steps**

1. refuse unless the database holds the data folder's own bytes
2. snapshot the case's numbers
3. fly the design loop and the case's campaigns from the database
4. compare every number, and count the runs naming the database
5. evaluate every row and closure; write the traceability
6. write results/END_TO_END.md and end_to_end.json

- **Reads:** `DIR/design.tndb`; `matlab_sils/store/`
- **Writes:** `matlab_sils/store/pipeline/<case>/`; `matlab_sils/store/results_engine/campaigns/`; `results/END_TO_END.md`; `results/end_to_end.json`; `results/EVALUATION.md`; `results/TRACEABILITY.md`
- **Starts:** `tools/pipeline.py`; `tools/engine.py campaign`; `tools/evaluate.py`; `tools/trace.py`
- **Checks:** the database's inputs are the data folder's (tools/design_inputs.py differences); every run names the database; every number that differs is named
- **Undo:** It re-flies stored runs: `git checkout -- matlab_sils/store results` puts back the committed ones.
- **Code:** `tools/end_to_end.py`

## release_notes.py release-notes

The release notes' generated part (docs/RELEASE_PLAN.md P14): what ships, group by group, from the shipping record (tools/delivery.py ship), and every known gap from docs/ADCS_GAPS.md, written into docs/RELEASE_NOTES.md between its markers.

    python3 tools/release_notes.py [--design DIR] [--check]

**Steps**

1. make the shipping record of the design that ships (default: seeded and carried over)
2. read the gap register
3. write the part between the markers

- **Reads:** `docs/ADCS_GAPS.md`; DIR (a design folder)
- **Writes:** `docs/RELEASE_NOTES.md`
- **Starts:** nothing
- **Checks:** --check: the notes are what the records make; every group named accepted or UNCONFIRMED with why
- **Undo:** It writes a generated part only: `git checkout -- docs/RELEASE_NOTES.md`.
- **Code:** `tools/release_notes.py`

## manual.py manual

The apps' manual (design/manual/: a guide per role, the journey of a node, the glossary, the guide to TRI-NETRA Files, and the tours in tours.toml) written as design/js/manual.js, which every app opens in place with Help, and the journey diagram as design/manual/journey.svg. --check says whether both are current and holds the rules: every page starts with its one line (the explanation standard), every tour target is in its app, every field, choice and set of checks in the apps has help beside it.

    python3 tools/manual.py [--check]

**Steps**

1. read the manual pages and the tours
2. draw the journey
3. write design/js/manual.js and design/manual/journey.svg (or, with --check, compare)
4. check the rules

- **Reads:** `design/manual/`; `design/js/*_app.js`; `design/js/node_model.js`
- **Writes:** `design/js/manual.js`; `design/manual/journey.svg`
- **Starts:** nothing
- **Checks:** --check: the manual's pages, tours and field help (see what)
- **Undo:** It writes generated files only: `git checkout -- <file>` puts back the committed one, or run it again once its source is as you want it.
- **Code:** `tools/manual.py`

## node_catalog.py node_catalog

The node app's catalogue (design/js/node_catalog.js), written from the spec: the units and the quantities each measures, the physics relations, the sources, the evidence metrics and rungs, the provenance and belief words, the tags and every row of the tree with its label, kind, quantity, unit and layer. The node app offers these as its choices, so they are never typed by hand. --check says whether the committed file is current.

    python3 tools/node_catalog.py [--check]

**Steps**

1. read the spec's units, quantities, physics, sources, evidence vocabulary and tree
2. write design/js/node_catalog.js (or, with --check, compare)

- **Reads:** `spec/`
- **Writes:** `design/js/node_catalog.js`
- **Starts:** nothing
- **Checks:** --check: the committed catalogue is current
- **Undo:** It writes generated files only: `git checkout -- <file>` puts back the committed one, or run it again once its source is as you want it.
- **Code:** `tools/node_catalog.py`

## release.py release

Sealed group releases (releases/<group>-<version>.tnrel, as the group app seals them), checked from Python: each file passes tools/tndb.py check and is named for its group and version; every node's fingerprint, body fingerprint and the release's fingerprint are the SHA-256 of what they cover; its nodes are the group's as sealed, each confirmed or unconfirmed with why; a confirmed node was checked by someone other than its author and, when it computes, has a test vector from outside the code; the lead's seal names the version and the fingerprint. The same rules as design/js/release.js, written a second time.

    python3 tools/release.py check PATH... | list DIR

**Steps**

1. find the release files (under DIR/releases/ for a folder)
2. check each one
3. list every problem

- **Reads:** `DIR/releases/`; PATH
- **Writes:** nothing
- **Starts:** nothing
- **Checks:** each release file's fingerprints, its nodes, the confirmed rule and the seal
- **Undo:** Nothing to undo: it writes nothing.
- **Code:** `tools/release.py`

## drive_pack.py drive-pack

The Drive pack: the two offline apps (TRI-NETRA Files, TRI-NETRA Group) and the design seeded from the spec (20 group files, 734 node files), in the layout the shared Drive folder of the design takes, with a README. CI builds it on every push as the trinetra-drive-pack artifact.

    python3 tools/drive_pack.py [--out DIR] [--rev REV]

**Steps**

1. build the offline pages
2. seed the design
3. check every design file
4. write the README

- **Reads:** `design/`; `spec/plan/`
- **Writes:** `dist/trinetra-drive-pack/`
- **Starts:** nothing
- **Checks:** the seeded and carried design checks
- **Undo:** Delete dist/trinetra-drive-pack/.
- **Code:** `tools/drive_pack.py`
