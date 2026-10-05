# The owner's plan against how TRI-NETRA works today

**In one line:** the 3 Oct 2026 plan (`docs/RELEASE_PLAN.md` §1) says the design lives in database files that people
write with the node and group apps, and every program reads only those files. Today the apps and the files exist
and work, but the engine, the design loop and most tools still read repository files. The database is a copy of
them, and nothing edited in the apps reaches a number. This page compares the two, layer by layer, and lists what is
missing.

## The plan, in its own words

> Everything lives in the databases, and every app reads or writes only them: the node app writes one node file;
> the group app writes a group's structure and releases; the main application reads `design.tndb` to draw every
> page; the engine (`adcs-sim`, plus the generated row code) runs a case and writes the result package, which the
> app, the reports, the Python package and the MATLAB twin read.

```
 DRIVE (files are the database): structure ×20 · nodes · releases · delivery
   ▲ lead writes            ▲ author writes          │ sealed releases
 GROUP APP                 NODE APP                  ▼
                                      DEVELOPER: verify → merge → design.tndb → engine code from pseudocode
                                      (Rust engine + MATLAB twin) → tests → test app → group accepts → main
                                                             ▼
                     MAIN APPLICATION: design.tndb ─▶ every page; engine runs a case ─▶ results ─▶ plotted
```

## Layer by layer

Status: ✅ as planned · 🟡 partly · ❌ not as planned.

| # | Layer of the plan | How it is today | Status |
|---|---|---|---|
| 1 | **Drive holds the database**: each owner team's shared drive, files are the database | One folder, "Trinetra Database"; its `Apps/` and `Design/` are **empty** (guides only). Team shared drives never created | ❌ |
| 2 | **Node files**, one per node, written by authors | 765 node files built by the seed and carry-over (734 from the spec + 31 added from the code), each carrying what the repository says and listing what is missing; all valid. Not on Drive yet | 🟡 |
| 3 | **Node app**: fill, view, sign one node | Built and tested (P5): every node kind, live checks, preview, mark ready, second-person check. Works on node files from disk or Drive (Chrome/Edge) | ✅ (unused until the upload) |
| 4 | **Group app**: structure, assemble, release | Built and tested (P4, P6, P12): map, stages, people, contracts, change requests, impact checks, seal, re-issue, deliveries, accept. **No group has a lead**, so none can seal | ✅ app · ❌ in use |
| 5 | **Releases and delivery** on Drive | `.tnrel` sealed releases and `deliveries/` records are made by the app and `tools/delivery.py`; five waves rehearsed in tests. None exists for real | ✅ machinery · ❌ in use |
| 6 | **verify → merge → design.tndb** | `tools/group.py verify/merge` exist. Merge takes **sealed releases only**: with none sealed it empties the nodes; it refills the engine's inputs from the **repository**, not from Drive; it leaves a backup file in the Drive folder. The shipped `design.tndb` is a seed snapshot (734 nodes) that already disagrees with the 765 node files beside it | ❌ |
| 7 | **design.tndb holds the design** | It holds copies: 213 case lines from `matlab_sils/cases/*.csv`; 163 engine inputs copied from `matlab_sils/data`, which is itself generated from `catalogue/`, `scenarios/`, `campaigns/`, `trades/` TOML in the repository; a seed snapshot of nodes; an empty `catalogue_output`. Dispersions, KPI definitions and delivery waves are not in it at all | ❌ |
| 8 | **Engine code from pseudocode**; the engine runs "adcs-sim plus the generated row code" | `tools/groupcode.py` generates each group's code from its nodes (Rust crate `adcs-groups`, WebAssembly, MATLAB `+groups`) and proves it reproduces the interpreter. **The engine does not use it**: `adcs-sim` depends only on its hand-written models; the generated code sits beside it | ❌ |
| 9 | **Tests → test app → group accepts** | Per-group test apps generated and passing in the browser; acceptance is a signature in the group app. 0 of 20 accepted | ✅ machinery · ❌ in use |
| 10 | **Engine runs a case from the database** | Yes for the case and the scenario/part/algorithm files, through an overlay (`TRINETRA_DESIGN`). But they are copies of repository files; **no node value reaches a case value**; the design loop, campaigns, traceability, pointing budget and V&V report read the repository directly | 🟡 |
| 11 | **Main application draws every page from design.tndb** | The desktop app has three tabs: *Fly* (pick case and scenario from the database, run), *Runs* (results, figures, report), *Design* (group list with counts, a case's lines). No page shows nodes, groups' content, outputs, closures, evaluation or traceability | 🟡 |
| 12 | **Results package read by the app, reports, Python package, MATLAB twin** | `.trinetra` export, results store and index, app figures and report per run, Python package reads the database: yes. The **MATLAB twin reads repository files**, not the database or the result package | 🟡 |
| 13 | **Inputs and outputs visible end to end** (a node → the number the engine flies → the result it produces) | Not linked: the node app shows a node, the main app shows a run, but nothing traces a node's value into a run or a run's result back to the node or closure it serves | ❌ |
| 14 | **The database updated over time** as people work | No route: nothing pulls Drive work back, and the repository rebuilds the database from its own files, so Drive and the repository would drift apart unseen | ❌ |

## What is missing

Grouped as the work that closes it. D1–D8 are in `docs/DATABASE_FIRST_PLAN.md`; D9–D12 are added here because the
plan's picture needs them and that document did not yet include them.

| Missing | Layers | Work |
|---|---|---|
| The design data that is not a node (catalogue, cases, scenarios, campaigns, trades, dispersions, KPIs, waves) as database files people edit, with the engine's inputs generated **from** them | 7, 10 | **D1 Library** |
| One build of `design.tndb` from Drive's structure, nodes, library and releases, keeping unsealed groups' current nodes | 6, 7 | **D2 Build** |
| Node values becoming the numbers the engine flies | 10, 13 | **D3 Nodes drive numbers** |
| Every tool reading only the database | 10, 12 | **D4 One reader** |
| A route from Drive back and forth, never overwriting people's work | 14 | **D5 Sync** |
| The database on Drive, once and correctly | 1, 2 | **D6 Pack and upload** |
| Proof that everything runs from Drive with the same numbers | all | **D7 End to end** |
| The engine running the code generated from the groups' pseudocode, row by row, with the hand-written model kept as the reference it must equal | 8 | **D9 Generated code in the engine** (new) |
| The main app drawing every page from the database: groups, nodes, values with their source, outputs, closures, evaluation, traceability, and each run linked to the node values it flew | 11, 13 | **D10 Main app over the database** (new) |
| Results flowing back into the database (latest evaluation and closures per case), so the apps can show outputs beside inputs | 11, 13 | **D11 Outputs into the database** (new) |
| The MATLAB twin flying the same database snapshot (an export generated from it) | 12 | **D12 Twin from the database** (new) |
| Team shared drives and named leads | 1, 4, 5, 9 | **Owner**: the Drive admin creates the drives; the owner names a lead per group |
| A release that contains all of it | — | **D8 Release** |

**Order:**
1. D1 → D2 → D3 → D4 make the database the source of truth.
2. D5 → D6 put it on Drive once.
3. D9–D12 complete the picture: generated code in the engine, the app over the database, outputs back, twin.
4. D7 proves it end to end.
5. D8 releases it.

D9 is the largest: the engine's hand-written models stay as the reference, and the generated code must reproduce
them before it replaces them, one group at a time.

## What is already right and stays

- The design file format (SQLite `.tndb`, `.tnrel`) and its checker.
- The three browser apps with their manuals and tests.
- The pseudocode language with its interpreter and translators.
- The generated group code and test apps.
- Releases, deliveries, acceptance.
- The engine's overlay that reads cases and inputs from a database.
- The result package and store.
- The Python package's database reader.
- The verify check for a Drive folder.

The gap is not in these pieces but in the joins between them.
