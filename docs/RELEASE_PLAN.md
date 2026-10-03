# TRI-NETRA: the plan to release 1.0.0

> **Answer first.** 1.0.0 sets TRI-NETRA's architecture end to end, for **every** part of the design
> at once: the whole ADCS tree (734 nodes in 22 groups, each one discipline) lives in database files that the people who
> own each part write themselves (a **node app** for each author, a **group app** for each lead); the
> developer side turns every sealed group release into checked, generated software; and the **main
> application** draws every page from one design database and plots the engine's results. There is
> no single worked example: most groups already have their content in the repository (code, the
> MATLAB twin, pseudocode, the catalogue, references, results), and all of it is carried into the new
> files and through the whole loop. The owner's eleven phases are folded in (§8). The ADCS technical
> gaps stay in [`ADCS_GAPS.md`](ADCS_GAPS.md) and do not gate this release.
>
> **Kind:** plan · **For:** the owner (Agastya) and the developer side · **Status:** confirmed by the owner (3 Oct 2026):
> version 1.0.0, the groups by type (§4), the decisions as suggested (§9)

The units of the plan are TRI-NETRA's own, from the spec package (`spec/SPEC.md` §5,
`spec/plan/tree.json`):

| term | in TRI-NETRA | count |
|---|---|---|
| **node** | one row of the ADCS tree: a requirement, a relation, a KPI, an evidence row, a closure | 734 once seeded: 327 in layers 1–2, 368 in the subsystem layers, 39 closures |
| **group** | the unit a lead releases: **one discipline**, holding every row of that type wherever it sits in the tree (its subsystem rows, its system-level targets, the facility rows that describe it). The tree's four layers stay as they are for the engine; a group is the ownership and release overlay on them | 22 (§4) |
| **owner team** | `systems`, `environment`, `sensing`, `actuators`, `gnc`, `avionics`, `verification`, `sales`, `programme`, `facility`, `quality` (SPEC §5.9); ten of them lead a group, `sales` writes inside `business` | 11 |

---

## 1 · The whole system in one picture

```
                    GOOGLE DRIVE (each owner team's shared drive; files are the database)
 ┌──────────────────────────────────────────────────────────────────────────────────────────┐
 │ structure/<group>.group.tndb ×22   nodes/<id>.node.tndb ×734   releases/<group>-1.0.tnrel │
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
                                MAIN APPLICATION (release 1.0.0, for everyone)
                                design.tndb (all 22 groups) ─▶ the app draws every page
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
| 2 | **Flight software from pseudocode** | Generated flight code must stay safe, C = Rust bit for bit, the twin in lockstep (SPEC §10.8.7) | The translator writes **engine and twin** code (tree relations, `adcs-core::physics`, the MATLAB twin). The flight software stays hand-written in 1.0.0; each algorithm's pseudocode v2 is the reference its C and Rust builds must reproduce on test vectors (owner may move the line, §9) | P2, P10 |
| 3 | **Content is uneven across groups** | 16 groups have rich content in the repository, 5 partial, 1 thin (§4) | Every group is transcribed from what exists; what is missing is a visible draft or gap with the owner team named, never invented; a thin group still goes through the whole loop with its structure, explanation and gaps | P8 |
| 4 | **Most rows have no code yet** | About 90 of 243 system rows have code, none linked by id; 166 internal subsystem rows are "to be named"; the engine is case- and scenario-driven, not row-driven | Transcription names the internal rows from the code that already computes them (the pump laws, the MEKF steps, the allocation) and links every computing row by id; a row without code shows as such, never as a number | P1, P8, P11 |
| 5 | **Dependencies between groups** | `pnt` relies on `est` and `ctl`; the KPI closures on everything; the business rows on the catalogue | Groups go through the loop in five waves in dependency order (§5); the versioned catalogue of outputs lists every group a changed output reaches, and those owners accept | P9, P12 |
| 6 | **Twenty-two releases, ten teams** | Every group needs a lead's seal and acceptance; people may be slow | The loop is built and proven on every group by the developer side; a group a person has not yet signed ships visibly UNCONFIRMED (SPEC §5.8), with the release notes naming it (owner decides, §9) | P12, P14 |
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

**Main application (everyone, 1.0.0)**

| Area | What changes |
|---|---|
| Tree, node and group pages | All 734 nodes and 22 groups drawn from `design.tndb` with the same drawing code as the two apps; hand-built and committed generated pages retire |
| Running | The case editor, scenarios and engine as today; results as the `.trinetra` package, plotted, compared and reopened; "try it" on a node runs its generated code in the page |
| Design loop and V&V | The design ledger, V&V report, traceability matrix and closures read the design database and the results store |
| Releases | Shows which release of each group the app contains |

**Manuals:** a guide for each role (author, lead, developer, user), generated from the one manual
source (`spec/manual/`), each opened in place by its app.

## 4 · The groups, by type, and what each already has

**The rule.** One group is one discipline, owned by one team and released by one lead. A group holds
every row of its type wherever the spec's four layers put it: a subsystem's internal rows (layer 3)
together with the system-level targets they answer (layer 2's "Sense it", "Act on it", "Close the
loop", "Host it" rows, and the rig needs), and the test-facility rows (layer 1) together with the rig
they describe. So a lead releases a discipline whole, and no group has to wait on another for rows of
its own kind. The tree itself is unchanged: the engine, the doors and the derivation edges stay as
the spec writes them.

Rich: most nodes can be transcribed from code, twin, pseudocode or data already in the repository.
Partial: some content exists, the rest is a marked draft. Thin: the spec's text only; the owner team
writes the content, the loop carries its structure and explanation.

| # | Group | Type | Lead team | Rows it holds (spec ids) | Nodes | What already exists (carried over in P8) | Content |
|---|---|---|---|---|---:|---|---|
| **Inputs** | | | | | | | |
| 1 | `case` Case and mission | what is asked | systems | `ci1` case · `m1` mission requirements · `cf` hardware fitted | 17 | case CSV and template, `case.rs`, `case_inputs.toml`, the case editor, product fill counts | rich |
| 2 | `body` Satellite body | what the ADCS carries | systems | `s1` mass properties · `s2` surfaces and offsets · `s3` magnetic cleanliness · `s4` flexible modes | 16 | the case's mass, surface, magnetic and flex rows, `plant.rs`, the flexible mode | rich |
| 3 | `env` Orbit and environment | what pushes it | environment | `m2` orbit · `m3` environment along the orbit · `sb0` disturbance targets · layer `dist` | 45 | `adcs-pop`, `orbit.rs`, `field.rs`, `atmos.rs`, `ephem.rs`, `torques.rs`, drag/SRP, the demand survey, `+asils/+env`, seed rows `m1_*`, `m2_*` | rich |
| **Sense** | | | | | | | |
| 4 | `sens` Attitude sensors | sensing hardware | sensing | layer `sens` · its `sb1` targets (`gs`) | 36 | `sensors.rs`, `comp.rs` (star-tracker image chain, Sun-sensor chain), parts, `+devices`, `+comp` | rich |
| 5 | `est` Attitude estimation | knowledge | gnc | layer `est` · its `sb1` targets (`ge`) | 28 | `fsw/pseudocode/03`, MEKF/TRIAD/QUEST in C, Rust and twin | rich |
| **Act** | | | | | | | |
| 6 | `mtq` Magnetic actuation | actuator | actuators | layer `mtq` · its `sb2` targets (`gm`) | 32 | coil model, sizing laws §2, ten magnetorquer laws, the Floquet certificate | rich |
| 7 | `rw` Reaction wheels | actuator | actuators | layer `rw` · its `sb2` targets (`gw`) | 33 | wheel model, datasheet catalogue, sizing §3 | rich |
| 8 | `fmr` Fluid momentum rings | actuator | actuators | layer `fmr` · its `sb2` targets (`gf`) | 46 | `empump.rs`, ring sizing, plant ring model, spare ring, windowed FDIR, IDMAS v2 §03–§07 | rich |
| 9 | `rcs` Reaction control thrusters | actuator | actuators | layer `rcs` · its `sb2` targets (`gr`) | 27 | thruster model, sizing §5, `rcs_pwm`, `rcs_rate` | rich |
| **Close the loop** | | | | | | | |
| 10 | `ctl` Control and allocation | control law | gnc | layer `ctl` · its `sb3` targets (`gc`) | 34 | `fsw/pseudocode/05`, `07`, LQR, PD, SMC, allocation, IDMAS split | rich |
| 11 | `modes` Modes and FDIR | mode logic | gnc | layer `modes` · its `sb3` targets (`gq`) | 27 | `fsw/pseudocode/08`, the modes catalogue, faults, FDIR | rich |
| 12 | `pnt` Pointing error budget | error budget | gnc | layer `pnt` · its `sb3` targets (`gp`) | 31 | `pointing_budget.py`, jitter (alignment and thermal unstated) | partial |
| **Host and budget** | | | | | | | |
| 13 | `fsw` Flight software and OBC | software | avionics | layer `fsw` · its `sb4` targets (`gx`) | 27 | `fsw/`, `fsw-rs/`, HAL, OBC link, stack check, firmware | rich |
| 14 | `budget` ADCS budgets | resources | systems | layer `budget` · its `sb4` targets (`gb`) · `s5` resources offered to the ADCS | 26 | sizing budgets (mass, power, volume), `select`, the platform allocations | rich |
| **Service and closure** | | | | | | | |
| 15 | `kpi` Pointing service and KPI closures | requirement against achievement | systems | `p1`–`p4` required and achieved KPIs · the closure layer (38 closures and its interface) | 83 | the case's `req.*` keys, `metrics.rs`, `kpis.toml`, campaigns, `trace.py`; the closures are written by the seeder, not authored | rich |
| **Verify** | | | | | | | |
| 16 | `vv` Verification and standards | evidence | quality | `v1` verification coverage · `v2` campaign evidence · `st1`–`st7` standards and compliance | 20 | campaigns, the results store, the V&V report, ECSS-E-ST-60-10C metrics, traceability, mutation and test coverage | partial |
| 17 | `oils` OILS rig | rig | verification | layer `oils` · `v3` OILS rig needs · `fa1` SILS capacity · `fa2` OILS rig | 38 | soft OILS, `VIRTUAL_OBC.md`, the OBC link, soft-OILS timing (no board yet) | partial |
| 18 | `hils` HILS rig | rig | verification | layer `hils` · `v4` HILS rig needs | 64 | `OILS_HILS.md`, `spec/rig/` device maps | partial |
| 19 | `lab` Test equipment | facility | facility | `fa3` magnetic field simulator · `fa5` air-bearing platform · `fa6` optical and RF stimulators · `fa7` actuator test stands · `fa8` rig safety and power · `fa4` facility use | 37 | `spec/rig/labs` | partial |
| **Business** | | | | | | | |
| 20 | `catalogue` Catalogue and heritage | products | systems | `ct1` products · `ct2` design runs · `ct3` case matching · `hr1` flight record | 14 | `catalogue/` (parts, products, families, classes, 25 algorithms), the design loop, `adcs-design`, `results/DESIGN_*`, `SELECTION.md` | rich |
| 21 | `business` Commercial, orders and supply | commercial | programme (with sales) | `cm1`–`cm3` commercial · `od1`–`od2` order lifecycle · `su1`–`su3` supply chain | 32 | spec text only | thin |
| 22 | `risk` Risk management | risk | quality | `rk1`–`rk4` | 21 | `spec/derisk/` ledger, `derisk.py`, the narrative template | rich |
| | **Total** | | | | **734** | | 16 rich · 5 partial · 1 thin |

**What changed from the spec's split into 30 branches and layers, and why:**
- the layer-2 subsystem targets (`sb0`–`sb4`, 73 rows) and the rig needs (`v3`, `v4`, 21) moved into
  the discipline that answers them, so a subsystem is released with its own targets;
- the test-facility branch (50 rows) split by what it describes: the OILS rows with the OILS rig, the
  lab equipment as one group;
- the mission branch split by kind: the mission requirements with the case, orbit and environment
  with the disturbances;
- the resources offered to the ADCS (`s5`) joined the budgets they are checked against;
- commercial, orders and supply (all business, all thin) became one group; heritage joined the
  catalogue it records; standards joined verification;
- the KPI requirements, achievements and their closures are one group, because a closure is the
  comparison of the two.

Lead teams: systems 6 groups, gnc 4, actuators 4, verification 2, quality 2, environment, sensing,
avionics, facility and programme 1 each. The owners of layer-3 rows are SPEC §5.4's; the rest are
proposed from §5.9's list, for the owner to confirm with the leads (P8). `ADCS_GAPS.md` holds the
technical gaps inside them.

## 5 · Phases

| Phase | What it delivers | Done when | Who |
|---|---|---|---|
| **P0 · Foundation** | PR #12 merged to `main`; one version source (`VERSION` 1.0.0, read by Cargo, the app, the wheel and the zips); reproducible builds (`--locked`, toolchain file, actions pinned); the release proves what CI proves (twin and QEMU parity in `prove`, NOT RUN fails); branch protection with one human review; licence decided; committed generated pages and run files moved to CI builds and ledgers | `main` carries the work, CI green; a tagged dry-run release builds from a clean checkout | You confirm the merge and the settings; I do the rest |
| **P1 · Data model** | Schemas for the structure, node, release, design and results files: contracts, revisions, history, comments, change requests, belief records, archive, format version and upgrades. Generated from the spec (`spec/plan/*.toml`, `tree.json`), with readers in Rust, JavaScript and Python; the group map of §4 as data (`groups.toml`), checked to cover every row exactly once. A seeder writes all 22 structure files and 734 node shells from the tree | Every group's structure and node shells round-trip through every file type unchanged; an old-format file upgrades | Me |
| **P2 · Pseudocode v2** | Several outputs, loops that settle, tables, arrays, state carried between ticks, units; the checker in the browser; the translator to Rust (`adcs-core::physics`, the row code) and to MATLAB (the twin); an interpreter | Every relation in `physics.toml` and every algorithm in `fsw/pseudocode/` written in it; translator = interpreter; the hand-written C and Rust flight software reproduce the interpreter on test vectors | Me |
| **P3 · Files in the browser** | SQLite inside the offline page; open and save files on Drive; "open elsewhere" marker; conflict-copy detection; history and undo; crash-safe saving; size caps; one component set and bundled fonts, no outside hosts | A node file saves to a Drive folder, reopens, survives a crash and refuses a second editor | Me |
| **P4 · Group app: structure** | Map, contracts, stages, people, issue node files, every structure action with its impact check, change requests | All 22 groups open; the largest (`kpi`, 83; `hils`, 64) and the smallest (`catalogue`, 14) restructured with no node file broken | Me |
| **P5 · Node app** | All steps, uploads, equation helper, picture wizard, results import (paste from Excel), live checks (units, the explanation standard's marks, evidence debt), preview, ready, sign, contract updates | Every node kind (declared, computed, KPI, evidence, closure, interface) filled and previewed | Me |
| **P6 · Group app: assemble → release** | Assemble, checks across nodes, comments, compare, seal, node files stamped and kept, re-issue, import of today's node-form files; **a computing node with no outside fixture cannot be sealed as confirmed** | Every group can seal 1.0, re-issue a node and seal 1.1 | Me |
| **P7 · Manuals and usability** | Tours, field help, role guides, print, the journey diagram; a usability session with 2–3 real members; fixes from it | Members complete a node and a release without help | Me + you choose testers |
| **P8 · All content carried over + Drive** | For **all 22 groups**, every node filled from what exists (§4): pseudocode transcribed from the Rust, C and twin, theory from the pseudocode documents and references, results from outside fixtures and the twin (marked as the twin), parts and algorithms from the catalogue, the 166 internal subsystem rows named from the code that computes them; what does not exist marked as draft or gap with its owner team. Drive shared drives set up for the 10 lead teams | All 22 groups' node files filled, each item showing its origin; every team opens its groups from Drive | Me + your Drive admin |
| **P9 · Rules + developer intake** | SPEC §3.2 and §5.10–5.11, `docs/CHANGING.md` and a new `CONTRIBUTING.md` rewritten; `group verify` / `group merge` → `design.tndb`; the catalogue of outputs; the impact listing across groups; every command in the pipeline table (`explain`, numbered steps, `--dry-run`, what it checks, how to undo, where its code is) | All 22 groups verify and merge into `design.tndb`; the catalogue lists every output | Me; you approve the rule change |
| **P10 · Developer skill + agents** | Coordinator skill, backend agent (translator first), frontend agent; `group wire / test / build / deliver`; for **every computing row of every group**, the code generated from pseudocode and tested against its results, edge cases and every other group; C = Rust and engine = twin on what changed; WebAssembly for "try it"; generators explained by example | Every group delivers a test app with no hand-written physics in its rows, and today's engine numbers unchanged | Me |
| **P11 · Main app on databases** | `design.tndb` from all 22 groups; the app's pages read it through one component set and one plotting module (engine and twin); results as SVG/PDF and a report per run; a run records only its input hash and what differs from the release defaults; sweeps keep only what their figures need; the Python package reads the database; committed pages retired; every stale stored run re-flown | Today's app runs entirely from `design.tndb` with the same numbers; `adcs results stale` finds nothing | Me |
| **P12 · Test → accept → release, every group** | Delivery notes, acceptance in the group app, `group accept`, `ship`; the 22 groups go through in five waves (below); catalogue refresh after each | All 22 groups delivered; each accepted by its lead, or shipped visibly UNCONFIRMED as the owner decides (§9) | Leads accept; you confirm `main` |
| **P13 · The whole system end to end** | One case through everything: `ais_3u` and `ais_img_3u` as case CSVs → every row evaluated or shown as not computed → the design loop → campaigns → evidence rows → all 39 closures → the V&V report and traceability, all from `design.tndb` | Both cases run end to end from the databases, with the same numbers as today and every closure answering or blocked by name | Me |
| **P14 · Release 1.0.0** | Full gate and tests, browser checks of the three apps, the readability and change guides, API docs built in CI, release notes from the groups' release records and `ADCS_GAPS.md`, wheel per platform, C/Rust parity on Windows and macOS, the release | 1.0.0 published: main app, node app, group app, `design.tndb`, wheel, kits | You confirm the release |

**The five waves of P12** (dependency order; each wave's outputs are in the catalogue before the next):

| Wave | Groups | Why in this order |
|---|---|---|
| A · inputs | `case`, `body`, `env` | everything reads the case, the satellite and the environment |
| B · devices | `sens`, `mtq`, `rw`, `fmr`, `rcs` | read A; feed estimation and control |
| C · GNC | `est`, `ctl`, `modes`, `pnt` | read B |
| D · system | `fsw`, `budget`, `catalogue`, `kpi` | read C; the KPI closures read everything |
| E · verification and business | `vv`, `oils`, `hils`, `lab`, `risk`, `business` | read D; the thin group last, with its team's content |

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

## 6 · What 1.0.0 contains

- **Node app and group app:** offline, one file each, with manuals.
- **All 22 groups through the full loop:** structure, 734 nodes carried over from what exists, sealed,
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
| P8 | Your Google Workspace admin for the shared drives of the 10 lead teams; a lead named for each of the 22 groups |
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

| decision | outcome |
|---|---|
| The version | **1.0.0** (owner, 3 Oct 2026) |
| What a group is | **one discipline, 22 groups** (§4) (owner: "one group of the same type") |
| File names: `.tndb`, `.tnrel`, results as `.trinetra` | as proposed (owner) |
| Generated flight software in 1.0.0 | **stays hand-written**, checked against the pseudocode reference (owner) |
| A group whose lead has not accepted by release | **ships visibly UNCONFIRMED**, named in the release notes (owner) |
| Retention of time series | 30 days, as built (owner) |
| The leads of the 22 groups | open: the owner names them (P8) |
| Licence of published downloads | open: decided in P0 |
