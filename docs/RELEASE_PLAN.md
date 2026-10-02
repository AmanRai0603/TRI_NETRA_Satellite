# TRI-NETRA: the plan to release 0.5.0

> **Answer first.** 0.5.0 is the release that sets TRI-NETRA's architecture: the ADCS design lives in
> database files that the people who own each part write themselves (a **node app** for each author,
> a **group app** for each lead), the developer side turns a sealed group release into checked,
> generated software, and the **main application** draws every page from one design database and
> plots the engine's results. The owner's eleven phases are folded into it (§7), so nothing from them
> is lost. The ADCS technical gaps stay in [`ADCS_GAPS.md`](ADCS_GAPS.md) and do not gate this
> release.
>
> **Kind:** plan · **For:** the owner (Agastya) and the developer side · **Status:** proposed, waiting
> for the owner's confirmation (2 Oct 2026)

The units of the plan are TRI-NETRA's own, from the spec package (`spec/SPEC.md` §5):

| term | in TRI-NETRA | count |
|---|---|---|
| **node** | one row of the ADCS tree: a requirement, a relation, a KPI, an evidence row (`spec/plan/tree.json`) | 734 once seeded (327 in layers 1–2, 368 in the 14 subsystem layers, 39 closures) |
| **group** | the unit a lead releases: a layer-1 branch, a layer-2 branch, a subsystem layer, the closure layer | 30 (9 + 6 + 14 + 1) |
| **owner team** | `systems`, `environment`, `sensing`, `actuators`, `gnc`, `avionics`, `verification`, `sales`, `programme`, `facility`, `quality` (§5.9) | 11 |
| **worked example** | the fluid momentum rings layer, `fmr` (36 rows), with `gf_7` (ring spin-down time), already the explanation standard's worked example | 1 group |

---

## 1 · The whole system in one picture

```
                    GOOGLE DRIVE (each owner team's shared drive; files are the database)
 ┌──────────────────────────────────────────────────────────────────────────────────────────┐
 │ structure/fmr.group.tndb   nodes/<id>.node.tndb ×N   releases/fmr-1.0.tnrel   delivery/  │
 └───────▲───────────────────────────▲──────────────────────────┬───────────────────▲───────┘
         │ lead writes               │ each author writes       │ sealed release     │ test app
   ┌─────┴──────┐              ┌─────┴──────┐                   ▼                    │
   │ GROUP APP  │              │  NODE APP  │         DEVELOPER: skill + agents ─────┘
   │ structure, │              │ fill, view │         verify → merge → design.tndb
   │ assemble,  │              │ sign one   │         → engine code from pseudocode
   │ release    │              │ node       │         (Rust engine + MATLAB twin) → tests
   └────────────┘              └────────────┘         → test app → group accepts → main
                                                                │
                                                                ▼
                                MAIN APPLICATION (release 0.5.0, for everyone)
                                design.tndb (all groups) ─▶ the app draws every page
                                engine (adcs-sim + generated rows) runs a case
                                ─▶ results (.trinetra result package) ─▶ plotted
```

**Everything lives in the databases, and every app reads or writes only them:**
- the **node app** writes one node file; the **group app** writes the structure and the releases;
- the **main application** (today's `trinetra-app`, grown) reads `design.tndb` to draw every page;
- the **engine** (`adcs-sim`, plus the generated row code) runs a case and writes the result
  package, which the app, the reports, the Python package and the MATLAB twin read.

The files are SQLite. Proposed names: `.tndb` for a structure, node or design database, `.tnrel` for a
sealed group release, and the existing `.trinetra` result package for results (owner to confirm, §8).

**What already exists and is built on, not replaced:** the engine (`adcs-sim`, `adcs-sim-core`,
`adcs-pop`), both flight softwares (C and Rust, bit-identical), the MATLAB twin, the design loop, the
results store (folders, manifests, the SQLite index, retention, `refly`), the desktop app and the
Python wheel, the node form prototype (`spec/forms/node_form.html`), the explanation standard
(`adcs-explain/1`) and its checker, the de-risking ledger (`spec/derisk/`), the tree seeder
(`spec/tools/build_tree.py`).

## 2 · Audit of the plan

| # | Area | Risk or gap found | How the plan handles it | Phase |
|---|---|---|---|---|
| 1 | **Order of work** | Authors would write pseudocode before the language can express their logic: several outputs, loops that settle, tables, and the state a filter carries from tick to tick | **Pseudocode v2 early**, before the node app ships | P2 |
| 2 | **Flight software from pseudocode** | Generated flight code must stay safe and C = Rust bit for bit; the twin must stay in lockstep (SPEC §10.8.7) | In 0.5.0 the translator writes **engine and twin** code (tree relations, `adcs-core::physics`, the MATLAB twin). The flight software stays hand-written; each algorithm's pseudocode v2 runs as the reference its C and Rust builds must reproduce on test vectors (owner may move the line, §8) | P2, P10 |
| 3 | **Most rows have no code yet** | The tree is a spec; about 90 of 243 system rows have code, none linked by id; the engine is case- and scenario-driven, not row-driven | The starting `design.tndb` is seeded from the spec package and links every row that has code today to it, by id; a row without code shows as such, never as a number | P1, P11 |
| 4 | Browsers | Saving straight to a Drive file works only in Chrome and Edge | Chrome/Edge supported for editing; other browsers can view, and save by downloading | P3 |
| 5 | Database in the browser | Some SQLite storage modes need server settings that a file opened from disk cannot have | The database is kept in memory and written to the file whole; no server; node files capped (e.g. 50 MB), each node's pictures capped (e.g. 500 KB) | P3 |
| 6 | Drive | The same person on two computers; Drive's "(1)" conflict copies | "Open elsewhere" marker; conflict copies detected and reported, never ignored | P3 |
| 7 | **Formats change over time** | Next year's app must still open this year's files | Every file carries its format version; newer apps upgrade old files (with a backup); older apps refuse newer files clearly | P1 |
| 8 | Uploaded files | An uploaded SVG or PDF could carry a script | Images shown as images (scripts never run); PDFs downloaded, not shown inside the app; no network; all text escaped | P4–P6 |
| 9 | Signatures | There are no accounts, so a name is typed, not proven | Accepted for internal use and stated plainly; Drive's record of who saved each file backs it up; "Checked by" keeps its SPEC §5.8 meaning | P6 |
| 10 | **Dependencies between groups** | `pnt` relies on `est` and `ctl`; the closures rely on everything | The released catalogue of outputs is versioned; at merge the developer tools list every group a changed output reaches, and those owners must accept | P9, P12 |
| 11 | **Moving over without breaking today's app** | Today's app, engine, design loop and stored results work from cases, scenarios, parts and code | `design.tndb` is built from the current spec package and catalogue, so the main app reads databases from day one; every engine model keeps its code until its group releases through the loop | P10–P11 |
| 12 | Python and MATLAB users | Faces other than the browser | The Python package reads `design.tndb` (SQLite, standard library); the MATLAB/Octave twin reads an export written from it (Octave has no built-in SQLite) | P11 |
| 13 | Rules | SPEC §3.2 rule 1 makes the sheet, written only by intake, the only source; rule 2 refuses `agent-generated` expected values | Rewritten before the developer side changes the repository: a group release becomes the source; transcription by an assistant is allowed when marked, and an expected value still never comes from the code under test | P9 |
| 14 | Usability | Many authors are engineers, not software people | Every app has an in-app manual, a tour per role, help on every field, and a usability test with real members before release; every page follows `adcs-explain/1` | P7 |
| 15 | Testing | Browser apps break unnoticed | Every app walked in a real browser in CI (Playwright, as `form_browser_check.py` does), with deliberate breakage to prove each check catches it (the `--selftest` pattern) | every phase |
| 16 | Recovery | A lost or damaged file | Drive file history, frozen releases, re-issuing a node file from any release | P6 |
| 17 | **Stored results older than the engine** | Every stored run before the rotor-FDIR change is stale; generated store files swamp review (one commit added 3,907) | Re-fly before release; committed generated pages and store files retire to CI builds and ledgers | P0, P11, P14 |
| 18 | People-dependent steps | Content, sign-offs, Drive setup, merges | Named per phase, so nothing waits silently (§6) | — |

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

A requirement, a closure, a door or an interface row keeps its fixed shape (SPEC §5.5): its node file
offers only the explanation, feedback and "something else".

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
| Tree, node and group pages | Drawn from `design.tndb` with the same drawing code as the two apps; hand-built and committed generated pages retire |
| Running | The case editor, the scenarios and the engine as today; results as the `.trinetra` package, plotted, compared and reopened; "try it" on a node runs its generated code in the page |
| Design loop and V&V | The design ledger, the V&V report and the traceability matrix read the design database and the results store |
| Releases | Shows which release of each group the app contains |

**Manuals:** a guide for each role (author, lead, developer, user), generated from the one manual
source (`spec/manual/`), each opened in place by its app.

## 4 · Phases

| Phase | What it delivers | Done when | Who |
|---|---|---|---|
| **P0 · Foundation** | PR #12 merged to `main` (the architecture so far); one version source (`VERSION` 0.5.0, Cargo, app, wheel, zips read it); reproducible builds (`--locked`, toolchain file, actions pinned); the release proves what CI proves (twin and QEMU parity in `prove`, NOT RUN fails); branch protection with one human review; licence decided; committed generated pages and run files moved to CI builds and ledgers | `main` carries the work, CI green; a tagged dry-run release builds from a clean checkout | You confirm the merge and the settings; I do the rest |
| **P1 · Data model** | Schemas for the structure, node, release, design and results files: contracts, revisions, history, comments, change requests, belief records, archive, format version and upgrades. Generated from the spec (`spec/plan/*.toml`, `tree.json`), with readers in Rust, JavaScript and Python | The `fmr` example round-trips through every file type unchanged; an old-format file upgrades; every row with code today is linked by id | Me |
| **P2 · Pseudocode v2** | Several outputs, loops that settle, tables, arrays, state carried between ticks, units; the checker in the browser (syntax and units); the translator to Rust (`adcs-core::physics`, the row code) and to MATLAB (the twin), and an interpreter | Every relation in `spec/plan/physics.toml` and every algorithm in `fsw/pseudocode/` can be written in it; translator = interpreter; the hand-written C and Rust flight software reproduce the interpreter on test vectors | Me |
| **P3 · Files in the browser** | SQLite inside the offline page; open and save files on Drive; "open elsewhere" marker; conflict-copy detection; history and undo; crash-safe saving; size caps. Starts with a proof in one offline page | A node file saves to a Drive folder, reopens, survives a crash and refuses a second editor | Me |
| **P4 · Group app: structure** | Map, contracts, stages, people, issue node files, every structure action with its impact check, change requests | A 3-node group and `hils` (48 rows) set up and restructured with no node file broken | Me |
| **P5 · Node app** | All steps, uploads, equation helper, picture wizard, results import (paste from Excel), live checks (units, the explanation standard's marks, evidence debt), preview, ready, sign, contract updates | A test author fills `fmr`'s nodes from scratch | Me |
| **P6 · Group app: assemble → release** | Assemble, checks across nodes, comments, compare, seal, node files stamped and kept, re-issue, import and export of today's node-form files; **a computing node with no outside fixture cannot be sealed** | `fmr`: node files → `fmr-1.0.tnrel` → re-issue → 1.1 | Me |
| **P7 · Manuals and usability** | Tours, field help, role guides, print; a usability session with 2–3 real members; fixes from it | Members complete a node and a release without help | Me + you choose testers |
| **P8 · `fmr` worked example + Drive** | The fluid-ring structure and node files from the spec: pseudocode transcribed from the existing Rust (`empump`, the ring sizing, the plant's ring model) and IDMAS v2 §03–§07, results from outside fixtures and the MATLAB twin (marked as the twin, not outside), drafts and gaps marked. Drive shared drives set up per owner team | `fmr` 1.0 sealed, every item showing its origin; members open it from Drive | Me + your Drive admin + the `actuators` owner confirms |
| **P9 · Rules + developer intake** | SPEC §3.2 and §5.10–5.11, `docs/CHANGING.md` and a new `CONTRIBUTING.md` rewritten; `group verify` / `group merge` → `design.tndb`; the catalogue of outputs; the impact listing across groups; every new command in the pipeline table (`explain`, numbered steps, `--dry-run`, what it checks, how to undo, where its code is) | `fmr` 1.0 merges into `design.tndb`; the catalogue lists its outputs | Me; you approve the rule change |
| **P10 · Developer skill + agents** | Coordinator skill, backend agent (translator first), frontend agent; `group wire / test / build / deliver`; tests against the group's results, edge cases and every other group; C = Rust and engine = twin checks on what changed; the generated code compiled to WebAssembly for "try it"; each generator explained by example | `fmr`'s sealed file becomes a delivered test app with no hand-written physics | Me |
| **P11 · Main app on databases** | The starting `design.tndb` from today's spec and catalogue; the app's pages read it through one component set and one plotting module (engine and twin); results as SVG/PDF and a report per run; a run records only its input hash and what differs from the release defaults; the Python package reads the database; committed pages retired; every stale stored run re-flown | Today's app runs entirely from `design.tndb` with the same numbers; `adcs results stale` finds nothing | Me |
| **P12 · Test → accept → release loop** | Delivery notes, acceptance in the group app, `group accept`, `ship`; the release ships the apps and `design.tndb`; catalogue refresh | `fmr` goes from test app to accepted to merged | `actuators` lead accepts; you confirm `main` |
| **P13 · All groups** | All 30 groups' structure files and node files issued from the spec, ready for their teams | Every lead can open their group | Me |
| **P14 · Release 0.5.0** | Full gate and tests, browser checks of the three apps, the readability and change guides, API docs built in CI, release notes from the groups' release records and `ADCS_GAPS.md`, wheel per platform, C/Rust parity on Windows and macOS, the release | 0.5.0 published: main app, node app, group app, `design.tndb`, wheel, kits | You confirm the release |

**Order and overlap:**
- P0 comes first.
- P1 and P2 run side by side.
- P3 is needed before P4–P6.
- P7 follows P6.
- P8 needs P6 and the Drive admin.
- P9–P10 need P2 and P8.
- P11 can start after P1, alongside P4–P8.
- P12 needs P10 and P11.
- P13 needs P6.
- P14 comes last.

## 5 · What 0.5.0 contains

- **Node app and group app:** offline, one file each, with manuals.
- **Main application:** reads `design.tndb`, runs the engine, plots the results; the design loop,
  V&V report and traceability on the same data.
- **`fmr`:** released through the full loop, its row code generated from pseudocode and tested against
  its own results, the twin in lockstep.
- **All 30 groups:** set up and ready for their teams. Their existing behaviour is unchanged until
  each releases through the loop.
- **The developer side:** skill, agents and commands, documented.
- **Rules:** rewritten for this way of working.
- **Known gaps:** `ADCS_GAPS.md`, listed in the release notes.

## 6 · What I need from you along the way

| When | What |
|---|---|
| P0 | OK to merge PR #12 to `main`; branch protection on; the licence decision |
| P7 | 2–3 members for a usability session |
| P8 | Your Google Workspace admin for the shared drives; the `actuators` owner's review of the transcribed `fmr` content |
| P9 | Approval of the rewritten rules (SPEC §3.2, the intake) |
| P12, P14 | Your confirmation for merging to `main` and for the release |

## 7 · The owner's eleven phases, folded in

Phases 1, 2, 3 and 5 (failure handling) are done (record: [`UPGRADE_PLAN.md`](UPGRADE_PLAN.md)).
Every open item of the eleven is placed here:

| Phase | Open item | Placed in |
|---|---|---|
| 4 Right-size the process | One human review with branch protection; one version; reproducible builds; the release proves CI; licence | P0 |
| 4 | Stop committing generated pages; build them in CI | P0, P11 |
| 4 | Docs merged into one path (README → architecture → one contributor guide) | P9, P14 |
| 5 Content (evidence debt) | Evidence debt as the headline (done in `status`; extended to node files) | P5, P6 |
| 5 | Refuse to publish a computing row without an outside fixture | P6 |
| 5 | Owners send their first real content (theory, relation, fixtures) | P8, P13 |
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
| Deployment | A+ (each laptop plus a shared folder), decided; Drive shared drives are the shared folder; a team server only if chosen later | P3, P8 |

## 8 · Decisions the owner makes

| decision | suggested |
|---|---|
| The version: this plan's 0.5.0 (the repository says 1.0.0 today, never released) | 0.5.0 |
| File names: `.tndb`, `.tnrel`, results as `.trinetra` | as proposed |
| What a group is: a branch or layer (30) or a tree group (106) | 30 |
| Generated flight software in 0.5.0, or the flight software stays hand-written against the pseudocode reference (audit #2) | stays hand-written in 0.5.0 |
| Licence of published downloads | decide in P0 |
| Retention of time series | 30 days (as built) |
