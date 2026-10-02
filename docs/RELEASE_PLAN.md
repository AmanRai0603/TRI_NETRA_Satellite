# TRI-NETRA: the plan to release 0.5.0

> **Answer first.** 0.5.0 sets TRI-NETRA's architecture end to end, for **every** part of the design
> at once: the whole ADCS tree (734 nodes in 30 groups) lives in database files that the people who
> own each part write themselves (a **node app** for each author, a **group app** for each lead); the
> developer side turns every sealed group release into checked, generated software; and the **main
> application** draws every page from one design database and plots the engine's results. There is
> no single worked example: most groups already have their content in the repository (code, the
> MATLAB twin, pseudocode, the catalogue, references, results), and all of it is carried into the new
> files and through the whole loop. The owner's eleven phases are folded in (§8). The ADCS technical
> gaps stay in [`ADCS_GAPS.md`](ADCS_GAPS.md) and do not gate this release.
>
> **Kind:** plan · **For:** the owner (Agastya) and the developer side · **Status:** proposed, waiting
> for the owner's confirmation (2 Oct 2026)

The units of the plan are TRI-NETRA's own, from the spec package (`spec/SPEC.md` §5,
`spec/plan/tree.json`):

| term | in TRI-NETRA | count |
|---|---|---|
| **node** | one row of the ADCS tree: a requirement, a relation, a KPI, an evidence row, a closure | 734 once seeded: 327 in layers 1–2, 368 in the subsystem layers, 39 closures |
| **group** | the unit a lead releases: a layer-1 branch, a layer-2 branch, a subsystem layer, the closure layer | 30 = 9 + 6 + 14 + 1 (§4) |
| **owner team** | `systems`, `environment`, `sensing`, `actuators`, `gnc`, `avionics`, `verification`, `sales`, `programme`, `facility`, `quality` (SPEC §5.9) | 11 |

---

## 1 · The whole system in one picture

```
                    GOOGLE DRIVE (each owner team's shared drive; files are the database)
 ┌──────────────────────────────────────────────────────────────────────────────────────────┐
 │ structure/<group>.group.tndb ×30   nodes/<id>.node.tndb ×734   releases/<group>-1.0.tnrel │
 │ delivery/                                                                                │
 └───────▲───────────────────────────▲──────────────────────────┬───────────────────▲───────┘
         │ each lead writes          │ each author writes       │ sealed releases    │ test apps
   ┌─────┴──────┐              ┌─────┴──────┐                   ▼                    │
   │ GROUP APP  │              │  NODE APP  │         DEVELOPER: skill + agents ─────┘
   │ structure, │              │ fill, view │         verify → merge → design.tndb
   │ assemble,  │              │ sign one   │         → engine code from pseudocode
   │ release    │              │ node       │         (Rust engine + MATLAB twin) → tests
   └────────────┘              └────────────┘         → test app → group accepts → main
                                                                │
                                                                ▼
                                MAIN APPLICATION (release 0.5.0, for everyone)
                                design.tndb (all 30 groups) ─▶ the app draws every page
                                engine (adcs-sim + generated rows) runs a case
                                ─▶ results (.trinetra result package) ─▶ plotted
```

**Everything lives in the databases, and every app reads or writes only them:**
- the **node app** writes one node file; the **group app** writes a group's structure and releases;
- the **main application** (today's `trinetra-app`, grown) reads `design.tndb` to draw every page;
- the **engine** (`adcs-sim`, plus the generated row code) runs a case and writes the result package,
  which the app, the reports, the Python package and the MATLAB twin read.

The files are SQLite. Proposed names: `.tndb` for a structure, node or design database, `.tnrel` for a
sealed group release, the existing `.trinetra` result package for results (owner to confirm, §9).

**Built on, not replaced:** the engine (`adcs-sim`, `adcs-sim-core`, `adcs-pop`, `adcs-design`), both
flight softwares (C and Rust, bit-identical), the MATLAB twin, the design loop, the results store
(folders, manifests, SQLite index, retention, `refly`), the desktop app and Python wheel, the node
form prototype (`spec/forms/node_form.html`), the case editor and library forms, the explanation
standard (`adcs-explain/1`) and its checker, the de-risking ledger (`spec/derisk/`), the tree seeder
(`spec/tools/build_tree.py`), the twin map (`spec/plan/twin_map.toml`).

## 2 · Audit of the plan

| # | Area | Risk or gap found | How the plan handles it | Phase |
|---|---|---|---|---|
| 1 | **Order of work** | Authors would write pseudocode before the language can express their logic: several outputs, loops that settle, tables, state a filter carries from tick to tick | **Pseudocode v2 early**, before the node app ships | P2 |
| 2 | **Flight software from pseudocode** | Generated flight code must stay safe, C = Rust bit for bit, the twin in lockstep (SPEC §10.8.7) | The translator writes **engine and twin** code (tree relations, `adcs-core::physics`, the MATLAB twin). The flight software stays hand-written in 0.5.0; each algorithm's pseudocode v2 is the reference its C and Rust builds must reproduce on test vectors (owner may move the line, §9) | P2, P10 |
| 3 | **Content is uneven across groups** | 18 groups have rich content in the repository, 7 partial, 4 thin, and the closures are generated (§4) | Every group is transcribed from what exists; what is missing is a visible draft or gap with the owner team named, never invented; a thin group still goes through the whole loop with its structure, explanation and gaps | P8 |
| 4 | **Most rows have no code yet** | About 90 of 243 system rows have code, none linked by id; 166 internal subsystem rows are "to be named"; the engine is case- and scenario-driven, not row-driven | Transcription names the internal rows from the code that already computes them (the pump laws, the MEKF steps, the allocation) and links every computing row by id; a row without code shows as such, never as a number | P1, P8, P11 |
| 5 | **Dependencies between groups** | `pnt` relies on `est` and `ctl`; the closures on everything; the company layer on the catalogue and design runs | Groups go through the loop in five waves in dependency order (§5); the versioned catalogue of outputs lists every group a changed output reaches, and those owners accept | P9, P12 |
| 6 | **Thirty releases, eleven teams** | Every group needs a lead's seal and acceptance; people may be slow | The loop is built and proven on every group by the developer side; a group a person has not yet signed ships visibly UNCONFIRMED (SPEC §5.8), with the release notes naming it (owner decides, §9) | P12, P14 |
| 7 | Browsers | Saving straight to a Drive file works only in Chrome and Edge | Chrome/Edge supported for editing; other browsers view, and save by downloading | P3 |
| 8 | Database in the browser | Some SQLite storage modes need server settings a file opened from disk cannot have | Kept in memory and written to the file whole; no server; node files capped (e.g. 50 MB), each node's pictures capped (e.g. 500 KB) | P3 |
| 9 | Drive | The same person on two computers; Drive's "(1)" conflict copies | "Open elsewhere" marker; conflict copies detected and reported, never ignored | P3 |
| 10 | **Formats change over time** | Next year's app must open this year's files | Every file carries its format version; newer apps upgrade old files (with a backup); older apps refuse newer files clearly | P1 |
| 11 | Uploaded files | An uploaded SVG or PDF could carry a script | Images shown as images (scripts never run); PDFs downloaded, not shown in the app; no network; all text escaped | P4–P6 |
| 12 | Signatures | No accounts, so a name is typed, not proven | Accepted for internal use and stated plainly; Drive's record of who saved each file backs it up; "Checked by" keeps its SPEC §5.8 meaning | P6 |
| 13 | **Moving over without breaking today's app** | Today's app, engine, design loop and stored results work from cases, scenarios, parts and code | `design.tndb` is built from the current spec, catalogue and code links on day one; every engine model keeps its code until its group releases through the loop, and then the generated code must give the same numbers | P10–P11 |
| 14 | Python and MATLAB users | Faces other than the browser | The Python package reads `design.tndb` (standard-library SQLite); the MATLAB/Octave twin reads an export written from it (Octave has no built-in SQLite) | P11 |
| 15 | Rules | SPEC §3.2 rule 1 makes the sheet, written only by intake, the only source; rule 2 refuses `agent-generated` expected values | Rewritten before the developer side changes the repository: a group release becomes the source; transcription by an assistant is allowed when marked; an expected value still never comes from the code under test | P9 |
| 16 | Usability | Many authors are engineers, not software people | In-app manual, a tour per role, help on every field, a usability test with real members; every page follows `adcs-explain/1` | P7 |
| 17 | Testing | Browser apps break unnoticed | Every app walked in a real browser in CI (Playwright, as `form_browser_check.py` does), with deliberate breakage proving each check (the `--selftest` pattern) | every phase |
| 18 | Recovery | A lost or damaged file | Drive file history, frozen releases, re-issuing a node file from any release | P6 |
| 19 | **Stored results older than the engine** | Every stored run before the rotor-FDIR change is stale; generated store files swamp review (one commit added 3,907) | Re-fly before release; committed generated pages and store files retire to CI builds and ledgers | P0, P11, P14 |
| 20 | People-dependent steps | Content sign-off, Drive setup, merges, acceptance | Named per phase (§7), so nothing waits silently | — |

## 3 · The apps and their manuals

**Node app (every author).** The node form (`adcs-node-form/1`) grown into an app that writes a node
file.

| Screen | Purpose |
|---|---|
| **Home** | Their node: progress per step, comments from the lead, contract changes waiting |
| **Steps** (by node kind) | Identity · Explanation (say it simply → the real thing → where it breaks → try it) · Theory and equations · Inputs and output · Pseudocode · Results · Evidence (outside fixtures, sources) · Pictures · Code · Belief record (what was believed, what tested it) · Review |
| **Preview** | The node exactly as the main application will show it |
| **Ready and sign** | Mark ready; sign ("Checked by"); send a change request about the contract |
| **Help** | A tour on first open, help beside every field, the manual |

A requirement, closure, door or interface row keeps its fixed shape (SPEC §5.5): its node file offers
only the explanation, feedback and "something else".

**Group app (group lead)**

| Screen | Purpose |
|---|---|
| **Map** | Nodes, arrows, stages, inputs from other groups; add, rename, split, merge, archive with an impact check |
| **People** | Members, authors per node, issue node files, re-issue |
| **Progress** | Every node's status, readiness, signatures, evidence debt, changes since the last release |
| **Assemble and review** | The whole group exactly as the main application will show it; checks across nodes; comments; change requests |
| **Release** | Sign, seal 1.0 / 1.1…, compare releases, delivery status |
| **Help** | Tour, manual, role guide |

**Main application (everyone, 0.5.0)**

| Area | What changes |
|---|---|
| Tree, node and group pages | All 734 nodes and 30 groups drawn from `design.tndb` with the same drawing code as the two apps; hand-built and committed generated pages retire |
| Running | The case editor, scenarios and engine as today; results as the `.trinetra` package, plotted, compared and reopened; "try it" on a node runs its generated code in the page |
| Design loop and V&V | The design ledger, V&V report, traceability matrix and closures read the design database and the results store |
| Releases | Shows which release of each group the app contains |

**Manuals:** a guide for each role (author, lead, developer, user), generated from the one manual
source (`spec/manual/`), each opened in place by its app.

## 4 · Every group, and what it already has

Rich: most nodes can be transcribed from code, twin, pseudocode or data already in the repository.
Partial: some content exists, the rest is a marked draft. Thin: the spec's text only; the owner team
writes the content, the loop carries its structure and explanation.

| # | Group | Owner | Nodes | What already exists (transcribed in P8) | Content |
|---|---|---|---:|---|---|
| **Layer 1 — the company** | | | **133** | | |
| 1 | `cas` Case intake | systems | 5 | case CSV format and template, `case.rs`, `case_inputs.toml`, the case editor | rich |
| 2 | `cat` Catalogue | systems | 11 | `catalogue/` (parts, products, families, classes, 25 algorithms), the design loop, `adcs-design`, `results/DESIGN_*`, `SELECTION.md` | rich |
| 3 | `cmr` Commercial | sales | 14 | spec text only | thin |
| 4 | `ord` Order lifecycle | programme | 13 | spec text only | thin |
| 5 | `fac` Test facility | facility | 50 | `spec/rig/` (device maps, labs), `OILS_HILS.md`, soft-OILS timing | partial |
| 6 | `std` Standards & compliance | quality | 11 | ECSS-E-ST-60-10C metrics, `trace.py`, mutation and test coverage | partial |
| 7 | `sup` Supply chain | programme | 5 | spec text only | thin |
| 8 | `hrt` Heritage | programme | 3 | spec text only | thin |
| 9 | `rsk` Risk management | systems | 21 | `spec/derisk/` ledger, `derisk.py`, the narrative template | rich |
| **Layer 2 — the satellite's ADCS** | | | **194** | | |
| 10 | `svc` Pointing service | systems | 44 | the case's `req.*` keys, `metrics.rs`, `kpis.toml`, campaigns, `trace.py` | rich |
| 11 | `cpt` ADCS configuration | systems | 6 | product fill and counts (`product.rs`) | rich |
| 12 | `msn` Mission and orbit | environment | 20 | `adcs-pop`, `orbit.rs`, `field.rs`, `atmos.rs`, `ephem.rs`, seed rows `m1_*`, `m2_*` | rich |
| 13 | `sat` Satellite as the ADCS sees it | systems | 21 | the case's mass, surface, magnetic, flex, resources rows; `plant.rs`, flexible mode | rich |
| 14 | `sub` ADCS subsystems (system level) | systems | 73 | the subsystem targets: metrics, sizing, design-loop budgets | partial |
| 15 | `ver` Verification | verification | 30 | campaigns, results store, soft OILS, V&V report, rig needs in docs | partial |
| **Layer 3 — the subsystems** | | | **368** | | |
| 16 | `dist` Disturbance environment | environment | 25 | `torques.rs`, `adcs-pop` drag/SRP, the demand survey, `+asils/+env` | rich |
| 17 | `sens` Attitude sensors | sensing | 29 | `sensors.rs`, `comp.rs` (star-tracker image chain, Sun-sensor chain), parts, `+devices`, `+comp` | rich |
| 18 | `est` Attitude estimation | gnc | 22 | `fsw/pseudocode/03`, MEKF/TRIAD/QUEST in C, Rust and twin | rich |
| 19 | `mtq` Magnetic actuation | actuators | 26 | coil model, sizing laws §2, ten magnetorquer laws, Floquet certificate | rich |
| 20 | `rw` Reaction wheels | actuators | 26 | wheel model, datasheet catalogue, sizing §3 | rich |
| 21 | `fmr` Fluid momentum rings | actuators | 36 | `empump.rs`, ring sizing, plant ring model, spare ring, windowed FDIR, IDMAS v2 §03–§07 | rich |
| 22 | `rcs` Reaction control thrusters | actuators | 22 | thruster model, sizing §5, `rcs_pwm`, `rcs_rate` | rich |
| 23 | `ctl` Control and allocation | gnc | 28 | `fsw/pseudocode/05`, `07`, LQR, PD, SMC, allocation, IDMAS split | rich |
| 24 | `pnt` Pointing error budget | gnc | 25 | `pointing_budget.py`, jitter (alignment and thermal unstated) | partial |
| 25 | `modes` Modes and FDIR | gnc | 22 | `fsw/pseudocode/08`, modes catalogue, faults, FDIR | rich |
| 26 | `fsw` Flight software and OBC interfaces | avionics | 22 | `fsw/`, `fsw-rs/`, HAL, OBC link, stack check, firmware | rich |
| 27 | `budget` ADCS unit budgets | systems | 17 | sizing budgets (mass, power, volume), `select` | rich |
| 28 | `oils` OILS rig | verification | 20 | soft OILS, `VIRTUAL_OBC.md`, the link (no board yet) | partial |
| 29 | `hils` HILS rig | verification | 48 | `OILS_HILS.md`, `spec/rig/labs` | partial |
| **Closures** | | | **39** | | |
| 30 | `x_closure` KPI closures | systems | 39 | written by the seeder from `kpis.toml`; no authoring | generated |

18 rich, 7 partial, 4 thin, 1 generated. (`ADCS_GAPS.md` holds the technical gaps inside them.) The owner of each layer-3 group is SPEC §5.4's; the owners of the layer-1 and layer-2 branches are proposed here from §5.9's list, for the owner to confirm with the leads (P8).

## 5 · Phases

| Phase | What it delivers | Done when | Who |
|---|---|---|---|
| **P0 · Foundation** | PR #12 merged to `main`; one version source (`VERSION` 0.5.0, read by Cargo, the app, the wheel and the zips); reproducible builds (`--locked`, toolchain file, actions pinned); the release proves what CI proves (twin and QEMU parity in `prove`, NOT RUN fails); branch protection with one human review; licence decided; committed generated pages and run files moved to CI builds and ledgers | `main` carries the work, CI green; a tagged dry-run release builds from a clean checkout | You confirm the merge and the settings; I do the rest |
| **P1 · Data model** | Schemas for the structure, node, release, design and results files: contracts, revisions, history, comments, change requests, belief records, archive, format version and upgrades. Generated from the spec (`spec/plan/*.toml`, `tree.json`), with readers in Rust, JavaScript and Python. A seeder writes all 30 structure files and 734 node shells from the tree | Every group's structure and node shells round-trip through every file type unchanged; an old-format file upgrades | Me |
| **P2 · Pseudocode v2** | Several outputs, loops that settle, tables, arrays, state carried between ticks, units; the checker in the browser; the translator to Rust (`adcs-core::physics`, the row code) and to MATLAB (the twin); an interpreter | Every relation in `physics.toml` and every algorithm in `fsw/pseudocode/` written in it; translator = interpreter; the hand-written C and Rust flight software reproduce the interpreter on test vectors | Me |
| **P3 · Files in the browser** | SQLite inside the offline page; open and save files on Drive; "open elsewhere" marker; conflict-copy detection; history and undo; crash-safe saving; size caps; one component set and bundled fonts, no outside hosts | A node file saves to a Drive folder, reopens, survives a crash and refuses a second editor | Me |
| **P4 · Group app: structure** | Map, contracts, stages, people, issue node files, every structure action with its impact check, change requests | All 30 groups open; the largest (`fac`, 50; `hils`, 48) and the smallest (`hrt`, 3) restructured with no node file broken | Me |
| **P5 · Node app** | All steps, uploads, equation helper, picture wizard, results import (paste from Excel), live checks (units, the explanation standard's marks, evidence debt), preview, ready, sign, contract updates | Every node kind (declared, computed, KPI, evidence, closure, interface) filled and previewed | Me |
| **P6 · Group app: assemble → release** | Assemble, checks across nodes, comments, compare, seal, node files stamped and kept, re-issue, import of today's node-form files; **a computing node with no outside fixture cannot be sealed as confirmed** | Every group can seal 1.0, re-issue a node and seal 1.1 | Me |
| **P7 · Manuals and usability** | Tours, field help, role guides, print, the journey diagram; a usability session with 2–3 real members; fixes from it | Members complete a node and a release without help | Me + you choose testers |
| **P8 · All content carried over + Drive** | For **all 30 groups**, every node filled from what exists (§4): pseudocode transcribed from the Rust, C and twin, theory from the pseudocode documents and references, results from outside fixtures and the twin (marked as the twin), parts and algorithms from the catalogue, the 166 internal subsystem rows named from the code that computes them; what does not exist marked as draft or gap with its owner team. Drive shared drives set up for the 11 teams | All 30 groups' node files filled, each item showing its origin; every team opens its groups from Drive | Me + your Drive admin |
| **P9 · Rules + developer intake** | SPEC §3.2 and §5.10–5.11, `docs/CHANGING.md` and a new `CONTRIBUTING.md` rewritten; `group verify` / `group merge` → `design.tndb`; the catalogue of outputs; the impact listing across groups; every command in the pipeline table (`explain`, numbered steps, `--dry-run`, what it checks, how to undo, where its code is) | All 30 groups verify and merge into `design.tndb`; the catalogue lists every output | Me; you approve the rule change |
| **P10 · Developer skill + agents** | Coordinator skill, backend agent (translator first), frontend agent; `group wire / test / build / deliver`; for **every computing row of every group**, the code generated from pseudocode and tested against its results, edge cases and every other group; C = Rust and engine = twin on what changed; WebAssembly for "try it"; generators explained by example | Every group delivers a test app with no hand-written physics in its rows, and today's engine numbers unchanged | Me |
| **P11 · Main app on databases** | `design.tndb` from all 30 groups; the app's pages read it through one component set and one plotting module (engine and twin); results as SVG/PDF and a report per run; a run records only its input hash and what differs from the release defaults; sweeps keep only what their figures need; the Python package reads the database; committed pages retired; every stale stored run re-flown | Today's app runs entirely from `design.tndb` with the same numbers; `adcs results stale` finds nothing | Me |
| **P12 · Test → accept → release, every group** | Delivery notes, acceptance in the group app, `group accept`, `ship`; the 30 groups go through in five waves (below); catalogue refresh after each | All 30 groups delivered; each accepted by its lead, or shipped visibly UNCONFIRMED as the owner decides (§9) | Leads accept; you confirm `main` |
| **P13 · The whole system end to end** | One case through everything: `ais_3u` and `ais_img_3u` as case CSVs → every row evaluated or shown as not computed → the design loop → campaigns → evidence rows → all 39 closures → the V&V report and traceability, all from `design.tndb` | Both cases run end to end from the databases, with the same numbers as today and every closure answering or blocked by name | Me |
| **P14 · Release 0.5.0** | Full gate and tests, browser checks of the three apps, the readability and change guides, API docs built in CI, release notes from the groups' release records and `ADCS_GAPS.md`, wheel per platform, C/Rust parity on Windows and macOS, the release | 0.5.0 published: main app, node app, group app, `design.tndb`, wheel, kits | You confirm the release |

**The five waves of P12** (dependency order; each wave's outputs are in the catalogue before the next):

| Wave | Groups | Why first |
|---|---|---|
| A · inputs | `cas`, `msn`, `sat`, `cpt`, `svc` | everything reads the case, the orbit and the satellite |
| B · environment and hardware | `dist`, `sens`, `mtq`, `rw`, `fmr`, `rcs` | read A; feed estimation and control |
| C · GNC | `est`, `ctl`, `modes`, `pnt` | read B |
| D · system | `fsw`, `budget`, `sub`, `cat`, `ver`, `x_closure` | read C; the closures read everything |
| E · rigs and company | `oils`, `hils`, `fac`, `std`, `rsk`, `cmr`, `ord`, `sup`, `hrt` | read D; the thin groups last, with their teams' content |

**Order and overlap:**
- P0 comes first.
- P1 and P2 run side by side.
- P3 is needed before P4–P6.
- P7 follows P6.
- P8 needs P1 and P2, and runs alongside P3–P6 (it fills the files those apps open).
- P9–P10 need P2 and P8.
- P11 can start after P1, alongside P4–P8.
- P12 needs P10 and P11.
- P13 needs P12's wave D.
- P14 comes last.

## 6 · What 0.5.0 contains

- **Node app and group app:** offline, one file each, with manuals.
- **All 30 groups through the full loop:** structure, 734 nodes carried over from what exists, sealed,
  verified, merged, delivered, accepted (or visibly UNCONFIRMED); every computing row's code generated
  from pseudocode and tested against its results, the twin in lockstep.
- **Main application:** reads `design.tndb`, runs the engine, plots the results; the design loop, V&V
  report, traceability and closures on the same data; both reference cases end to end.
- **The developer side:** skill, agents and commands, documented.
- **Rules:** rewritten for this way of working.
- **Known gaps:** `ADCS_GAPS.md` and every node marked draft or gap, listed in the release notes.

## 7 · What I need from you along the way

| When | What |
|---|---|
| P0 | OK to merge PR #12 to `main`; branch protection on; the licence decision |
| P7 | 2–3 members for a usability session |
| P8 | Your Google Workspace admin for the shared drives of the 11 teams; a lead named for each of the 30 groups |
| P9 | Approval of the rewritten rules (SPEC §3.2, the intake) |
| P12 | Each lead's acceptance of their group (or your decision to ship it UNCONFIRMED) |
| P12, P14 | Your confirmation for merging to `main` and for the release |

## 8 · The owner's eleven phases, folded in

Phases 1, 2, 3 and 5 (failure handling) are done (record: [`UPGRADE_PLAN.md`](UPGRADE_PLAN.md)).
Every open item of the eleven is placed here:

| Phase | Open item | Placed in |
|---|---|---|
| 4 Right-size the process | One human review with branch protection; one version; reproducible builds; the release proves CI; licence | P0 |
| 4 | Stop committing generated pages; build them in CI | P0, P11 |
| 4 | Docs merged into one path (README → architecture → one contributor guide) | P9, P14 |
| 5 Content (evidence debt) | Evidence debt as the headline (done in `status`; extended to node files and groups) | P5, P6 |
| 5 | Refuse to publish a computing row without an outside fixture | P6 |
| 5 | Owners send their real content (theory, relation, fixtures) | P8 (carried over), P12 (signed) |
| 6 Scripts you can follow | Pipeline table, `explain`, numbered steps, `--dry-run` on every tool that writes, failure reports with the retry command, registry fields, `trace` and `why` with history | P9 (new commands), P14 (every tool) |
| 6 | The journey diagram: author → lead → developer → test app → accept → `main` → release | P7 |
| 7 Storage and outputs | Output types (SVG/PDF), a report per run | P11 |
| 7 | A run records its input hash and what differs from the defaults; sweeps keep only what their figures need | P11 |
| 7 | Size caps on each node's pictures; photos and video outside git | P3 |
| 7 | Stored results match the engine (re-fly) | P11 |
| 8 Component library and lesson pages | One component set for every page **and** form (node app, group app, main app, case editor, reports, role guides, manual); a check that refuses hand-built components | P3–P6, P11 |
| 8 | `lesson.toml` → the node file's Explanation step (`adcs-explain/1`), outside the result fingerprints | P1, P5 |
| 8 | "Try it" widgets declared, computed by the generated code in WebAssembly | P10, P11 |
| 8 | Lesson form with preview through the review flow → the node app and the group app's review | P5, P6 |
| 8 | Pages offline, bundled fonts, no outside hosts | P3 |
| 9 Figure numbers from the kernel | The numbers in figures come from the engine and the generated code; selected figures checked against their reference values and pictures | P10, P11 |
| 10 Readability | Readability guide; "how to change the code" guide for the common changes; generators explained by example; docs on public items and API docs in CI; remaining splits and typed errors | P10, P14 |
| 11 When first needed | SQLite is now the file format (P1); Parquet, animation and 3D when a trigger is met | P1; later |
| Deployment | A+ (each laptop plus a shared folder), decided; the Drive shared drives are the shared folder; a team server only if chosen later | P3, P8 |

## 9 · Decisions the owner makes

| decision | suggested |
|---|---|
| The version: 0.5.0 (the repository says 1.0.0 today, never released) | 0.5.0 |
| File names: `.tndb`, `.tnrel`, results as `.trinetra` | as proposed |
| What a group is: a branch or layer (30) or a tree group (106) | 30 |
| Generated flight software in 0.5.0, or the flight software stays hand-written against the pseudocode reference (audit #2) | stays hand-written in 0.5.0 |
| A group whose lead has not accepted by release: hold the release, or ship it visibly UNCONFIRMED | ship UNCONFIRMED, named in the release notes |
| The leads of the 30 groups | owner names them (P8) |
| Licence of published downloads | decide in P0 |
| Retention of time series | 30 days (as built) |
