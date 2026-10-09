# Database first: the plan that comes before everything else

> **Replaced by [`PLAN_2_0.md`](PLAN_2_0.md)** (5 Oct 2026). Kept as the record of the audit; its §8 says where each package went.

**In one line:** TRI-NETRA is meant to be an application built over one design database that people edit on Drive,
and today it is not. This plan makes it so, puts that database on Drive once and correctly, proves the whole
application runs from it end to end, and releases that as **v1.1.0**. Only then do the roadmap's phases continue.

The owner's principle (5 Oct 2026): *the code is just code reading over the database; the application is built
over it.* Everything below follows from that sentence.

## 1 · Where it stands (audit of 5 Oct 2026)

| Question | Answer today |
|---|---|
| Where is the design people edit? | On Drive: `Design/structure/` (20 group files) and `Design/nodes/` (765 node files), written by the three browser apps. Frozen into `Design/releases/` when a lead seals |
| What does the engine fly? | Repository files: `catalogue/*.toml`, `scenarios/`, `campaigns/`, `trades/` → `tools/export_catalogue.py` → `matlab_sils/data/*.json`, and `matlab_sils/cases/*.csv` |
| What is `design.tndb`? | A **copy** of those repository files (163 inputs, 213 case lines) plus a seed snapshot of 734 nodes. It is already out of step with the 765 node files beside it, and all 734 shared nodes differ in content |
| Does an edit on Drive change any number? | **No.** No tool turns node values into case values or engine inputs |
| Do the tools read the database? | The engine and the desktop app do (cases and inputs). The design loop, campaigns, traceability, pointing budget, V&V report and MATLAB twin read the repository directly |
| Can the database be rebuilt from Drive? | Only by `tools/group.py merge`, and only from sealed releases: with none sealed it **empties** the nodes. It also refills the inputs from the repository and leaves a stray `design.tndb.prev` in the Drive folder |
| Is anything on Drive? | Guides and two sheets only; `Apps/` and `Design/` are empty |

Ids DB1–DB16 below refer to the findings of that audit.

## 2 · Target: what lives where

```
 Drive "Trinetra Database"  (the source of truth for everything that is design)
   Apps/        the three browser apps
   Design/
     structure/ 20 group files: groups, stages, people (leads, authors, checkers), edges, contracts
     nodes/     every node: statement, values, pseudocode, tests, outputs
     library/   design data that is not a node: the catalogue (parts, products, algorithms, components,
                modes, families, classes), cases, scenarios, campaigns, trades, dispersions, KPI
                definitions, delivery waves. One file per owning group, edited in the Files app
     releases/  sealed releases (made by the Group app)
     deliveries/ delivery records (made by tools/delivery.py)
     design.tndb BUILT from the folders above by one tool; never edited by hand
     results/   read-only summary of the latest verified results (evaluation, traceability per case)
   Guides/      guides and the engineering documents
   MANIFEST.json, README.txt

 Repository  (code, physics data, and the seed)
   code and tests; physics/reference data shipped with the program (DE440, DTM2020, JB2008, XYS06, IGRF);
   the seed that made the first Drive content (spec, catalogue TOML) — read once, then retired as a source;
   design/live/: the last pulled snapshot of Drive's Design/ folder, which CI and the tools run on

 Local / CI only  (never on Drive)
   the results store, dist/, build outputs, engine/target
```

**One flow, always the same:**
1. **Pull.** Drive's `Design/` comes into a checked snapshot.
2. **Build.** `design.tndb` is built from structure, nodes, library and releases.
3. **Fly.** The engine, the design loop, the campaigns, evaluation and traceability run on that database alone.
4. **Publish.** The rebuilt `design.tndb` and the results summary go back to Drive.

Only new or changed files are pushed, and people's edits are never overwritten.

## 3 · Work packages (version 1.1.0, "database first")

The comparison with the owner's 3 Oct plan, layer by layer, is `docs/VISION_VS_CURRENT.md`; D9–D12 come from it.


| WP | What | Fixes | Acceptance (all automated) | Effort |
|---|---|---|---|---|
| **D1 · Library** | A design-file kind `library` with typed tables mirroring today's catalogue, case, scenario, campaign, trade, dispersion, KPI and wave sources. A one-time import from the repository seeds them. `export_catalogue` is turned round: `matlab_sils/data` JSON is generated **from the library**, not from TOML | DB1, DB10 | Import then export reproduces every current `matlab_sils/data` file **byte for byte**, and every case CSV line for line; the Files app opens and edits a library file | M–L |
| **D2 · Build** | `tools/design_build.py DIR`: one deterministic build of `design.tndb` from `structure/`, `nodes/`, `library/` and `releases/`. A sealed group contributes its release; an unsealed group contributes its current node files, marked unreleased. Fills `catalogue_output` from node outputs (contract versions from releases, 0 before). Writes nothing else into the folder. `seed`, `merge` and `drive_pack` all call it | DB2, DB5, DB11, DB14 | 765 nodes in the database equal the node files; a group sealing changes only that group's rows; build twice gives the same bytes; no `.prev` file in the folder | M |
| **D3 · Nodes drive numbers** | The case values the engine flies come from the node values they are mapped to (`design_case.node`: 162 of 213 lines today). Lines without a node stay library lines. A mapped node with no value is refused by name, never filled with a guess | DB3 | Change a value in a node file → rebuild → the engine flies the new value (test); every mapped line traced to its node in the manifest of each run | M |
| **D4 · One reader** | One design-source module for every Python tool (design loop, campaigns, evaluate, trace, pointing budget, V&V report, delivery), honouring `--design DIR` and `TRINETRA_DESIGN`. The engine gets `--design`, finds a kit's database as the app does, and `adcs size` reads through the overlay. A folder or unreadable database is refused clearly. DE440 is found beside the program with a clear error otherwise. `evaluate` accepts the pack root. The MATLAB twin reads an export generated from the database | DB6, DB7, DB8, DB12, DB13, DB16 | A CI job with the repository's design data **removed** (`catalogue/`, `scenarios/`, `campaigns/`, `trades/`, `matlab_sils/data`, `matlab_sils/cases` moved away) runs the full chain from the snapshot alone | M |
| **D5 · Sync** | `tools/design_sync.py pull DIR` (into `design/live/`, every file checked, changes listed) and `push DIR` (only new or changed built files and the results summary; refuses to overwrite a file changed on Drive since the last pull). Builds on `drive_pack --verify` | DB4 (D0.2 done, D0.3, D0.4) | Pull → build → push → verify round trip with no problem; an edit made on Drive between pull and push is preserved and reported | S–M |
| **D6 · Pack and upload** | The pack is the D2 build of the seed, with `library/`, `MANIFEST.json`, guides and a correct README. The specification below says what must and must not be there. The owner deletes Drive's `Apps/` and `Design/` and uploads the zip once. `--verify --first-upload` must pass on the Drive folder | DB14 | Verify passes on the uploaded folder (via Drive for desktop or a downloaded copy) | S |
| **D7 · End to end from Drive** | Pull the uploaded Drive folder, build, and run everything from it: both design loops, every campaign, soft OILS, evaluation, traceability, V&V report, desktop app, delivery status | — | Every number equals the one flown from the repository files today (as P13 showed for the engine). The run is refused if any tool reads repository design data (D4's job). Results summary published to Drive | M |
| **D9 · Generated code in the engine** | The engine runs each group's code generated from its nodes' pseudocode (`adcs-relations` since S7.16, folded with the design's relations; `adcs-groups` before), with the hand-written model kept as the reference it must equal, one group at a time | `docs/VISION_VS_CURRENT.md` layer 8 | Each switched group reproduces the hand-written model's results on every scenario (bit for bit or within stated tolerance); a node's pseudocode changed on Drive changes the engine | L |
| **D10 · Main app over the database** | The desktop app draws every page from `design.tndb`: groups, nodes, values with their source, outputs, closures, evaluation, traceability; each run linked to the node values it flew | layers 11, 13 | Browser tests: from a node to the runs that used it and back | M |
| **D11 · Outputs into the database** | Latest evaluation and closures per case written into the database (and `Design/results/`), so apps show outputs beside inputs | layers 11, 13 | The node and group apps show a node's latest result and closure | M |
| **D12 · Twin from the database** | The MATLAB twin flies an export generated from the database | layer 12 | Twin suites pass on the export; engine and twin read the same snapshot | S–M |
| **D8 · Release v1.1.0** | Release notes say what "database first" changes, and correct 1.0.0's statements (the KPI passes not demonstrated, power as actuator power only). The release assets add the Drive pack zip and its manifest. Tag by the owner as before | — | Release workflow green; the pack zip in the release equals the uploaded one | S |

**Owner actions in this phase:**
- **D6:** delete Drive's `Apps/` and `Design/`, upload the zip, then tell me (or run `--verify`).
- **D6 onward:** add the group leads in the Group app. No group can seal until it has one (DB9). Lead names are yours to give; the seed will not invent people.
- **D8:** push the tag.

**Do not upload the zip sent on 5 Oct.** Its database disagrees with its node files, and its file layout changes in D1–D2. The upload happens once, in D6.

## 4 · The pack: what must and must not be on Drive

| Must be there | Must **not** be there |
|---|---|
| `Apps/TRI-NETRA Files.html`, `Group.html`, `Node.html` | The desktop app's page, test-app templates, any code (`.py .rs .c .h .m`) |
| `Design/structure/*.group.tndb` (20) | `design.tndb.prev` or any backup copy; Drive conflict copies `… (1).tndb` |
| `Design/nodes/*.node.tndb` (765) | Google-converted files in `Design/` or `Apps/` (`.gdoc`, `.gsheet`) |
| `Design/library/*.tndb` (D1) | The results store, `dist/`, build outputs, wheels, executables |
| `Design/design.tndb` (built, D2) | DE440 and other physics data (they ship with the program) |
| `Design/releases/`, `Design/deliveries/` (made by the apps and tools, empty at upload) | Raw run folders or time series |
| `Design/results/` (read-only summary, D7) | Repository TOML/CSV copies of design data (the library replaces them) |
| `Guides/` (the 7 guides already there, plus `Guides/Engineering/`) | |
| `README.txt`, `MANIFEST.json` | |

`tools/drive_pack.py --verify` enforces this table. Its manifest lists every expected file with size and SHA-256; the
rules name what may appear later (releases, deliveries, `.editing` markers) and what never may.

## 5 · Order after v1.1.0

The roadmap's phases follow, renumbered after this release (details in `docs/TECHNICAL_ROADMAP.md`):

| Release | Phase | Content |
|---|---|---|
| v1.2 | U0 | Correct 1.0.0's wrong results and the critical flight-software and GNC defects, each with a test that failed before it |
| v1.3 | U1 | Environment, frames and requirements you can cite; orbit-propagator fixes |
| v1.4 | U2 | Stability margins, statistics, budgets, independent referents |
| v1.5 | U3 | Devices, GNC, system completeness, FMECA |
| v1.6 | U4 | Flight-software assurance, TM/TC, Renode soft OILS |
| v2.0 | U5 | Board OILS and HILS (hardware-paced) |

From v1.1.0 on, every phase works on the database: changes to design data are made in the design files and flow
through pull, build, fly and publish. Changes to the code are made in the repository.
