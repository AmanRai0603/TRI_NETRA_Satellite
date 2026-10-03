# TRI-NETRA: the plan to release 1.0.0

> **Answer first.** 1.0.0 sets TRI-NETRA's architecture end to end, for **every** part of the design
> at once: the whole ADCS tree (734 nodes today in 20 groups, each one discipline) lives in database files that the people who
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
| **group** | the unit a lead releases: **one discipline**, holding every row of that type wherever it sits in the tree (its subsystem rows, its system-level targets, the facility rows that describe it). The tree's four layers stay as they are for the engine; a group is the ownership and release overlay on them | 20 (§4) |
| **owner team** | `systems`, `environment`, `sensing`, `actuators`, `gnc`, `avionics`, `verification`, `sales`, `programme`, `facility`, `quality` (SPEC §5.9); ten of them lead a group, `sales` writes inside `business` | 11 |

---

## 1 · The whole system in one picture

```
                    GOOGLE DRIVE (each owner team's shared drive; files are the database)
 ┌──────────────────────────────────────────────────────────────────────────────────────────┐
 │ structure/<group>.group.tndb ×20   nodes/<id>.node.tndb ×734   releases/<group>-1.0.tnrel │
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
                                design.tndb (all 20 groups) ─▶ the app draws every page
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
| 3 | **Content is uneven across groups** | 14 groups have rich content in the repository, 5 partial, 1 thin; guidance, CMG, VSCMG, dynamics and onboard navigation have code but no rows yet (§4) | Every group is transcribed from what exists; what is missing is a visible draft or gap with the owner team named, never invented; a thin group still goes through the whole loop with its structure, explanation and gaps | P8 |
| 4 | **Most rows have no code yet** | About 90 of 243 system rows have code, none linked by id; 166 internal subsystem rows are "to be named"; the engine is case- and scenario-driven, not row-driven | Transcription names the internal rows from the code that already computes them (the pump laws, the MEKF steps, the allocation) and links every computing row by id; a row without code shows as such, never as a number | P1, P8, P11 |
| 5 | **Dependencies between groups** | `pnt` relies on `est` and `ctl`; the KPI closures on everything; the business rows on the catalogue | Groups go through the loop in five waves in dependency order (§5); the versioned catalogue of outputs lists every group a changed output reaches, and those owners accept | P9, P12 |
| 6 | **Twenty releases, ten teams** | Every group needs a lead's seal and acceptance; people may be slow | The loop is built and proven on every group by the developer side; a group a person has not yet signed ships visibly UNCONFIRMED (SPEC §5.8), with the release notes naming it (owner decides, §9) | P12, P14 |
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
| Tree, node and group pages | Every node and all 20 groups drawn from `design.tndb` with the same drawing code as the two apps; hand-built and committed generated pages retire |
| Running | The case editor, scenarios and engine as today; results as the `.trinetra` package, plotted, compared and reopened; "try it" on a node runs its generated code in the page |
| Design loop and V&V | The design ledger, V&V report, traceability matrix and closures read the design database and the results store |
| Releases | Shows which release of each group the app contains |

**Manuals:** a guide for each role (author, lead, developer, user), generated from the one manual
source (`spec/manual/`), each opened in place by its app.

## 4 · The groups, by type, and what each already has

**The rule** (owner, 3 Oct 2026). One group is one discipline, owned by one team and released by one
lead: satellite dynamics in one group; every environment and disturbance torque in one; every sensor
in one; every navigation algorithm under attitude estimation; every actuator, CMG and VSCMG included,
in one; guidance in one (the spec's tree has none: it is added); the controller in one; FDIR on its
own; the design loop on its own, with a stage for each subsystem; flight software on its own. A
group holds every row of its type wherever the spec's four layers put it (a subsystem's rows together
with the system-level targets they answer; a rig with its facility rows). The owner will refine
groups and nodes part by part when editing starts; the group app's structure actions (split, merge,
move, with their impact check, P4) are how.

**The tree grows where a discipline has no rows yet.** Guidance, CMG, VSCMG, the satellite's
dynamics and onboard navigation are computed by the engine and the flight software today but have no
rows in the spec's tree. Their rows are added in P8 through the new-node path, transcribed from the
code that computes them, so the node count rises above 734. The spec's 166 internal subsystem rows
"to be named" are named in P8 from the code too, and a row that sizes something (rather than models
it) moves to the design loop's stage for its subsystem.

Rich: most nodes can be transcribed from code, twin, pseudocode or data already in the repository.
Partial: some content exists, the rest is a marked draft. Thin: the spec's text only; the owner team
writes the content, the loop carries its structure and explanation.

| # | Group | Lead team | Spec rows it holds | Spec nodes | Rows added in P8 from existing code | What already exists (carried over in P8) | Content |
|---|---|---|---|---:|---|---|---|
| **Inputs** | | | | | | | |
| 1 | `case` Case and mission | systems | `ci1` case · `m1` mission requirements · `cf` hardware fitted | 17 | — | case CSV and template, `case.rs`, `case_inputs.toml`, the case editor, product fill counts | rich |
| 2 | `dyn` Satellite dynamics | systems | `s1` mass properties · `s2` surfaces and offsets · `s3` magnetic cleanliness · `s4` flexible modes | 16 | rigid-body dynamics and kinematics, rotor and ring momentum coupling, the flexible mode (hybrid coordinates) | `plant.rs`, `+asils/+plant`, the flexible-mode tests, the case's mass, surface, magnetic and flex rows | rich |
| 3 | `env` Environment and disturbance torques | environment | `m2` orbit · `m3` environment along the orbit · `sb0` disturbance targets · layer `dist` | 45 | — | `adcs-pop` (gravity, drag, SRP, third bodies, tides, space weather), `orbit.rs`, `field.rs`, `atmos.rs`, `ephem.rs`, `torques.rs`, the demand survey, `+asils/+env` | rich |
| **Sense and know** | | | | | | | |
| 4 | `sens` Sensors | sensing | layer `sens` · its `sb1` targets (`gs`) | 36 | — | `sensors.rs`, `comp.rs` (star-tracker image chain, Sun-sensor chain), magnetometer, gyro, Sun sensors, earth sensor, GNSS parts, `+devices`, `+comp` | rich |
| 5 | `nav` Navigation and attitude estimation | gnc | layer `est` · its `sb1` targets (`ge`) | 28 | onboard orbit and time navigation (GNSS fix, onboard propagator, latency), TRIAD, QUEST, the gyro filter | `fsw/pseudocode/02`, `03`, MEKF/TRIAD/QUEST in C, Rust and twin | rich |
| **Decide and act** | | | | | | | |
| 6 | `gdn` Guidance and mode management | gnc | the mode-sequencing targets `gq_0`, `gq_1`, `gq_4` and their layer-3 rows (layer `modes` and `sb3`) | 21 | nadir, target, Sun, slew and inertial guidance, yaw flip, boresight offset; the mode manager | `fsw/pseudocode/04`, `08`, guidance in C, Rust and twin, the modes catalogue | rich |
| 7 | `ctl` Controller and allocation | gnc | layer `ctl` · its `sb3` targets (`gc`) | 34 | — | `fsw/pseudocode/05`, `07`, LQR, PD, PID, SMC, the ten magnetorquer laws, B-dot, allocation, IDMAS split | rich |
| 8 | `fdir` FDIR | gnc | the fault targets `gq_2`, `gq_3` and their layer-3 rows | 6 | sensor health, rotor FDIR (instantaneous and windowed), safe mode, fault injection | FDIR in C, Rust and twin, `+faults`, the fault campaign; open items in `ADCS_GAPS.md` D6 | rich |
| 9 | `act` Actuators | actuators | layers `mtq`, `rw`, `fmr`, `rcs` · their `sb2` targets (`gm`, `gw`, `gf`, `gr`) | 138 | CMG and VSCMG (models, gimbal steering, singularity handling); one stage per actuator in the group | coil, wheel, ring, thruster, CMG and VSCMG models in engine and twin, `empump.rs`, `cmg_sr`, the datasheet catalogue (`TRN-CMG-1`, `TRN-VSCMG-1`, …), IDMAS v2 §03–§07 | rich |
| **Host** | | | | | | | |
| 10 | `fsw` Flight software and OBC | avionics | layer `fsw` · its `sb4` targets (`gx`) | 27 | — | `fsw/`, `fsw-rs/`, HAL, OBC link, stack check, firmware, parameter blob | rich |
| **Design** | | | | | | | |
| 11 | `design` Design loop | systems | layer `budget` · its `sb4` targets (`gb`) · `s5` resources offered to the ADCS · `ct2` design runs · `ct3` case matching | 32 | the sizing rows of every subsystem, one stage per subsystem (sensors, magnetorquers, wheels, rings, thrusters, CMG, VSCMG, budgets, selection) | `adcs-design`, `docs/SIZING_LAWS.md`, the design-loop nodes (`nodes.json`), `tools/pipeline*.py`, the fault campaign and redundancy step | rich |
| 12 | `pnt` Pointing error budget | systems | layer `pnt` · its `sb3` targets (`gp`) | 31 | — | `pointing_budget.py`, jitter (alignment and thermal unstated) | partial |
| **Service and closure** | | | | | | | |
| 13 | `kpi` Pointing service and KPI closures | systems | `p1`–`p4` required and achieved KPIs · the closure layer | 83 | — | the case's `req.*` keys, `metrics.rs`, `kpis.toml`, campaigns, `trace.py`; the closures are written by the seeder | rich |
| **Verify** | | | | | | | |
| 14 | `vv` Verification and standards | quality | `v1` coverage · `v2` campaign evidence · `st1`–`st7` standards | 20 | — | campaigns, the results store, the V&V report, ECSS-E-ST-60-10C metrics, traceability, mutation and test coverage | partial |
| 15 | `oils` OILS rig | verification | layer `oils` · `v3` rig needs · `fa1` SILS capacity · `fa2` OILS rig | 38 | — | soft OILS, `VIRTUAL_OBC.md`, the OBC link, soft-OILS timing (no board yet) | partial |
| 16 | `hils` HILS rig | verification | layer `hils` · `v4` rig needs | 64 | — | `OILS_HILS.md`, `spec/rig/` device maps | partial |
| 17 | `lab` Test equipment | facility | `fa3` field simulator · `fa5` air bearing · `fa6` stimulators · `fa7` test stands · `fa8` rig safety · `fa4` facility use | 37 | — | `spec/rig/labs` | partial |
| **Business** | | | | | | | |
| 18 | `catalogue` Catalogue and heritage | systems | `ct1` products · `hr1` flight record | 8 | — | `catalogue/` (parts, products, families, classes, 25 algorithms), `results/DESIGN_*`, `SELECTION.md` | rich |
| 19 | `business` Commercial, orders and supply | programme (sales writes) | `cm1`–`cm3` · `od1`–`od2` · `su1`–`su3` | 32 | — | spec text only | thin |
| 20 | `risk` Risk management | quality | `rk1`–`rk4` | 21 | — | `spec/derisk/` ledger, `derisk.py`, the narrative template | rich |
| | **Total** | | | **734** | plus the added rows | | 14 rich · 5 partial · 1 thin |

Lead teams (confirmed by the owner): systems 7 groups, gnc 4, verification 2, quality 2, environment,
sensing, actuators, avionics, facility and programme 1 each. `ADCS_GAPS.md` holds the technical gaps
inside them.

### How the groups work together (owner, 3 Oct 2026)

1. **One group, one code module, in C, Rust and the twin.** Each group's generated and hand-written
   code lives in its own module on every side, so a group's change touches only its module and
   C = Rust parity is checked group by group. Today guidance, mode management and FDIR sit inside
   the flight software's step (`adcs_fsw.c`, `fsw.rs`, the twin's `step.m`); they are split out
   into `adcs_guid` / `guid.rs` / `guidance.m`, `adcs_modes` / `modes.rs` / `modes.m` and
   `adcs_fdir` / `fdir.rs` / `fdir.m`, and `fsw` keeps the shell: the scheduler, the HAL, the
   parameter tables, the OBC link. Bit-identical before and after, on every shipped scenario.

   | group | C | Rust | twin |
   |---|---|---|---|
   | `env` | `adcs_env.c` (onboard models) | `adcs-pop`, `adcs-sim-core` `orbit`/`field`/`atmos`/`ephem`/`torques`, `fsw-rs` `env.rs` | `+env`, `+orbit` |
   | `sens` | `adcs_drv.c` (sensor drivers) | `adcs-sim-core` `sensors`/`comp`, `fsw-rs` `drv.rs` | `+devices`, `+comp` |
   | `nav` | `adcs_est.c` | `est.rs` | `+fsw` `mekf_*`, `triad`, `quest` |
   | `gdn` | `adcs_guid.c`, `adcs_modes.c` (new) | `guid.rs`, `modes.rs` (new) | `guidance.m`, `modes.m`, `yaw_flip.m`, `boresight_offset.m` |
   | `ctl` | `adcs_ctl.c`, `adcs_alloc.c` | `ctl.rs`, `alloc.rs` | `control_law.m`, `allocate.m`, `bdot.m`, … |
   | `fdir` | `adcs_fdir.c` (new) | `fdir.rs` (new) | `fdir.m` (new), `+faults` |
   | `act` | `adcs_drv.c` (actuator drivers) | `adcs-sim-core` `actuators`, `adcs-design` `empump` | `+devices`, `+plant` actuator parts |
   | `dyn` | — | `adcs-sim-core` `plant` | `+plant` |
   | `fsw` | `adcs_fsw.c` (shell), `adcs_params.c`, `adcs_math.c` | `fsw.rs` (shell), `params.rs`, `math.rs`, `hal.rs`, `cabi.rs` | `step.m` (shell), `init.m` |
   | `design` | — | `adcs-design` | `+sizing`, `+solution` |
   | `kpi`, `vv` | — | `adcs-sim` `metrics` | `+metrics` |

2. **Stages with stage owners inside the big groups.** `act` (one stage each: magnetorquers,
   wheels, fluid rings, thrusters, CMG, VSCMG), `design` (one stage per subsystem's sizing, then
   budgets and selection), `env` (orbit, field, atmosphere, disturbance torques), `kpi`
   (requirements and achievements, closures) and `hils` (rig integration, rig needs). A stage owner
   signs the stage; the group lead seals the group.

3. **Boundary rules**, written in `groups.toml` and checked, so every row has exactly one owner:

   | where groups meet | the rule |
   |---|---|
   | `dyn` ↔ `act` | the actuator's own physics (torque, momentum, power, faults) is `act`'s; how its momentum enters the body's equations is `dyn`'s |
   | a subsystem ↔ `design` | a row that models how the device behaves stays in the subsystem; a row that sizes it (how big, how heavy, which part) is `design`'s |
   | `nav`, `gdn`, `ctl`, `fdir` ↔ `fsw` | the algorithm is its group's; when and how it is called (scheduling, timing, the HAL, the parameter tables) is `fsw`'s |
   | `kpi` ↔ `vv` | what is required and what was achieved is `kpi`'s; how it was proven (campaigns, coverage, standards) is `vv`'s |

4. **A contract at every boundary.** Each group publishes the outputs other groups read, with their
   units and ranges (`env`: the disturbance torques; `nav`: the attitude and rate estimate; `gdn`:
   the reference attitude and rate; `ctl`: the torque command; `act`: the delivered torque and
   momentum; `design`: the sized parts and budgets). The catalogue of outputs (P9) is these
   contracts; a change to one lists every group that reads it, and those owners accept it (P12).

## 5 · Phases

| Phase | What it delivers | Done when | Who |
|---|---|---|---|
| **P0 · Foundation** | PR #12 merged to `main`; one version source (`VERSION` 1.0.0, read by Cargo, the app, the wheel and the zips); reproducible builds (`--locked`, toolchain file, actions pinned); the release proves what CI proves (twin and QEMU parity in `prove`, NOT RUN fails); branch protection with one human review; licence decided; committed generated pages and run files moved to CI builds and ledgers | `main` carries the work, CI green; a tagged dry-run release builds from a clean checkout | You confirm the merge and the settings; I do the rest |
| **P1 · Data model** | Schemas for the structure, node, release, design and results files: contracts, revisions, history, comments, change requests, belief records, archive, format version and upgrades. Generated from the spec (`spec/plan/*.toml`, `tree.json`), with readers in Rust, JavaScript and Python; the group map of §4 as data (`groups.toml`: groups, stages and stage owners, the boundary rules, the code module of each group), checked to cover every row exactly once and every row to have one owner. A seeder writes all 20 structure files and the 734 node shells from the tree | Every group's structure and node shells round-trip through every file type unchanged; an old-format file upgrades | Me |
| **P2 · Pseudocode v2** | Guidance, mode management and FDIR split out of the flight software's step into their own modules in C, Rust and the twin, bit-identical before and after (§4, rule 1); several outputs, loops that settle, tables, arrays, state carried between ticks, units; the checker in the browser; the translator to Rust (`adcs-core::physics`, the row code) and to MATLAB (the twin); an interpreter | Every relation in `physics.toml` and every algorithm in `fsw/pseudocode/` written in it; translator = interpreter; the hand-written C and Rust flight software reproduce the interpreter on test vectors | Me |
| **P3 · Files in the browser** | SQLite inside the offline page; open and save files on Drive; "open elsewhere" marker; conflict-copy detection; history and undo; crash-safe saving; size caps; one component set and bundled fonts, no outside hosts | A node file saves to a Drive folder, reopens, survives a crash and refuses a second editor | Me |
| **P4 · Group app: structure** | Map, contracts, stages and stage owners, people, issue node files, every structure action with its impact check, change requests | All 20 groups open; the largest (`act`, 138 and growing) and the smallest (`catalogue`, 8) restructured, a node moved between two groups, with no node file broken | Me |
| **P5 · Node app** | All steps, uploads, equation helper, picture wizard, results import (paste from Excel), live checks (units, the explanation standard's marks, evidence debt), preview, ready, sign, contract updates | Every node kind (declared, computed, KPI, evidence, closure, interface) filled and previewed | Me |
| **P6 · Group app: assemble → release** | Assemble, checks across nodes, comments, compare, seal, node files stamped and kept, re-issue, import of today's node-form files; **a computing node with no outside fixture cannot be sealed as confirmed** | Every group can seal 1.0, re-issue a node and seal 1.1 | Me |
| **P7 · Manuals and usability** | Tours, field help, role guides, print, the journey diagram; a usability session with 2–3 real members; fixes from it | Members complete a node and a release without help | Me + you choose testers |
| **P8 · All content carried over + Drive** | For **all 20 groups**, every node filled from what exists (§4); the rows a discipline has no node for yet (guidance, CMG, VSCMG, dynamics, onboard navigation, each subsystem's sizing) added from the code that computes them; the `modes` layer split between `gdn` and `fdir`: pseudocode transcribed from the Rust, C and twin, theory from the pseudocode documents and references, results from outside fixtures and the twin (marked as the twin), parts and algorithms from the catalogue, the 166 internal subsystem rows named from the code that computes them; what does not exist marked as draft or gap with its owner team. Drive shared drives set up for the 10 lead teams | All 20 groups' node files filled, each item showing its origin; every team opens its groups from Drive | Me + your Drive admin |
| **P9 · Rules + developer intake** | SPEC §3.2 and §5.10–5.11, `docs/CHANGING.md` and a new `CONTRIBUTING.md` rewritten; `group verify` / `group merge` → `design.tndb`; the catalogue of outputs as the contract at every group boundary (§4, rule 4); the impact listing across groups; every command in the pipeline table (`explain`, numbered steps, `--dry-run`, what it checks, how to undo, where its code is) | All 20 groups verify and merge into `design.tndb`; the catalogue lists every output | Me; you approve the rule change |
| **P10 · Developer skill + agents** | Coordinator skill, backend agent (translator first), frontend agent; `group wire / test / build / deliver`; for **every computing row of every group**, the code generated from pseudocode and tested against its results, edge cases and every other group; C = Rust and engine = twin per group module (§4, rule 1); WebAssembly for "try it"; generators explained by example | Every group delivers a test app with no hand-written physics in its rows, and today's engine numbers unchanged | Me |
| **P11 · Main app on databases** | `design.tndb` from all 20 groups; the app's pages read it through one component set and one plotting module (engine and twin); results as SVG/PDF and a report per run; a run records only its input hash and what differs from the release defaults; sweeps keep only what their figures need; the Python package reads the database; committed pages retired; every stale stored run re-flown | Today's app runs entirely from `design.tndb` with the same numbers; `adcs results stale` finds nothing | Me |
| **P12 · Test → accept → release, every group** | Delivery notes, acceptance in the group app, `group accept`, `ship`; the 20 groups go through in five waves (below); catalogue refresh after each | All 20 groups delivered; each accepted by its lead, or shipped visibly UNCONFIRMED as the owner decides (§9) | Leads accept; you confirm `main` |
| **P13 · The whole system end to end** | One case through everything: `ais_3u` and `ais_img_3u` as case CSVs → every row evaluated or shown as not computed → the design loop → campaigns → evidence rows → all 39 closures → the V&V report and traceability, all from `design.tndb` | Both cases run end to end from the databases, with the same numbers as today and every closure answering or blocked by name | Me |
| **P14 · Release 1.0.0** | Full gate and tests, browser checks of the three apps, the readability and change guides, API docs built in CI, release notes from the groups' release records and `ADCS_GAPS.md`, wheel per platform, C/Rust parity on Windows and macOS, the release | 1.0.0 published: main app, node app, group app, `design.tndb`, wheel, kits | You confirm the release |

**The five waves of P12** (dependency order; each wave's outputs are in the catalogue before the next):

| Wave | Groups | Why in this order |
|---|---|---|
| A · inputs | `case`, `dyn`, `env` | everything reads the case, the satellite's dynamics and the environment |
| B · hardware | `sens`, `act` | read A; feed navigation, control and FDIR |
| C · GNC | `nav`, `gdn`, `ctl`, `fdir` | read B |
| D · system | `fsw`, `design`, `pnt`, `catalogue`, `kpi` | read C; the design loop sizes every subsystem; the KPI closures read everything |
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

**Progress of P0** (merged to `main` with PR #12):

| item | state |
|---|---|
| One version source | ✅ `tools/version.py`: `VERSION` 1.0.0; the engine's Cargo workspace (was 1.2.0), the Rust and C flight software follow it; the Rust build ids take their Cargo version; `check_all` checks it |
| Reproducible builds | ✅ `rust-toolchain.toml` (1.94.1, firmware target); every `cargo build` and test `--locked`; every action pinned to a commit SHA; `setup-python` in every job that runs Python; `tools/requirements-ci.txt` pins pyflakes, numpy, matplotlib; cargo-mutants pinned |
| The release proves what CI proves | ✅ `check_all --strict` (NOT RUN fails) in the release's `prove`, with the ARM compiler; the release re-runs CI's firmware (QEMU parity) and twin jobs before it publishes |
| Merge PR #12 to `main` | ✅ merged |
| Branch protection with one human review | the owner's repository setting |
| Licence | the owner's decision |
| Generated pages and run files out of git | 🟡 the design loop's per-iteration scenario files (8,292) and the duplicate V&V PDF left git: tracked files 17,276 → 8,983, every check passing in a clean checkout. The campaign runs' manifests and the rendered pages stay until P11 replaces them with the result package and pages drawn from the design database |

**Progress of P1** (data model):

| item | state |
|---|---|
| Every row of the tree, with a stable id | ✅ `tools/design_rows.py`: 734 rows (133 layer 1, 194 layer 2, 368 layer 3, 39 closures); layer-3 ids `l3_<layer>_interface`, `l3_<layer>_<target>_required` / `_achieved`, `l3_<layer>_row_<nn>` for the 166 rows to be named |
| The group map as data | ✅ `design/groups.toml`: 20 groups with lead team, branches, stages, the code module of each group in C, Rust and the twin, the `modes` split (`gq_0`, `gq_1`, `gq_4` to `gdn`; `gq_2`, `gq_3` to `fdir`) and the boundary rules; `tools/groups.py --check`: every row in exactly one group |
| The file formats | ✅ `design/schema.toml`: node, group, release and design files (SQLite), each with its format version; `tools/tndb.py` makes, opens, checks, dumps and loads them, upgrades an older file keeping the original, refuses a newer one |
| Readers in three languages | ✅ Python (`tools/tndb.py`); Rust (`engine/crates/trinetra-design`, held to `design/ddl.sql`); the browser's schema (`design/js/tndb_schema.js`, generated; the reader on SQLite in WebAssembly comes with the apps in P3) |
| The seeder | ✅ `tools/seed_design.py`: 20 group files, 734 node shells and the design database from the spec, each field marked with where it came from; nothing invented |
| Proof | ✅ `tests/test_design_files.py` (every kind round-trips unchanged, an old file upgrades, a newer one is refused, a picture round-trips byte for byte); `check_all` runs `design-files` |
| Results file in the schema | the result package (`.trinetra`) and index stay as they are; the design files point at them by result id. Belief records and the archive arrive with the apps that write them (P4, P6) |

## 6 · What 1.0.0 contains

- **Node app and group app:** offline, one file each, with manuals.
- **All 20 groups through the full loop:** structure, 734 nodes carried over from what exists, sealed,
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
| P8 | Your Google Workspace admin for the shared drives of the 10 lead teams; a lead named for each of the 20 groups |
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
| What a group is | **one discipline, 20 groups** (§4), the owner's list; refined part by part when editing starts |
| File names: `.tndb`, `.tnrel`, results as `.trinetra` | as proposed (owner) |
| Generated flight software in 1.0.0 | **stays hand-written**, checked against the pseudocode reference (owner) |
| A group whose lead has not accepted by release | **ships visibly UNCONFIRMED**, named in the release notes (owner) |
| Retention of time series | 30 days, as built (owner) |
| The lead teams | as in §4 (owner); the person leading each group is named in P8 |
| Licence of published downloads | open: decided in P0 |
