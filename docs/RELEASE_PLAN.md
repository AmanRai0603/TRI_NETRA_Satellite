# TRI-NETRA: the plan to release 1.0.0

> **Describes 1.0.0.** Its §1 (the whole system) is replaced for 2.0.0 by `docs/OPERATING_2_0.md` and `docs/SYSTEM_MODEL.md`; it stays the record of how 1.0.0 was built and what is in use until the switch-over.

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
   C = Rust parity is checked group by group. Guidance, mode management and FDIR sat inside the
   flight software's step (`adcs_fsw.c`, `fsw.rs`, the twin's `step.m`); P2 split them out into
   `adcs_guid` / `guid.rs` / `guidance.m`, `adcs_modes` / `modes.rs` / `mode_manager.m` and
   `adcs_fdir` / `fdir.rs` / `fdir.m`, and `fsw` keeps the shell: the tick, the state (private
   header `adcs_fsw_int.h`; in Rust `modes` and `fdir` are children of `fsw`), the HAL, the
   parameter tables, the OBC link. Bit-identical before and after on every shipped scenario.

   | group | C | Rust | twin |
   |---|---|---|---|
   | `env` | `adcs_env.c` (onboard models) | `adcs-pop`, `adcs-sim-core` `orbit`/`field`/`atmos`/`ephem`/`torques`, `fsw-rs` `env.rs` | `+env`, `+orbit` |
   | `sens` | `adcs_drv.c` (sensor drivers) | `adcs-sim-core` `sensors`/`comp`, `fsw-rs` `drv.rs` | `+devices`, `+comp` |
   | `nav` | `adcs_est.c` | `est.rs` | `+fsw` `mekf_*`, `triad`, `quest` |
   | `gdn` | `adcs_guid.c`, `adcs_modes.c` | `guid.rs`, `modes.rs` | `guidance.m`, `mode_manager.m`, `modes.m`, `yaw_flip.m`, `boresight_offset.m` |
   | `ctl` | `adcs_ctl.c`, `adcs_alloc.c` | `ctl.rs`, `alloc.rs` | `control_law.m`, `allocate.m`, `bdot.m`, … |
   | `fdir` | `adcs_fdir.c` | `fdir.rs` | `fdir.m`, `+faults` |
   | `act` | `adcs_drv.c` (actuator drivers) | `adcs-sim-core` `actuators`, `adcs-design` `empump` | `+devices`, `+plant` actuator parts |
   | `dyn` | — | `adcs-sim-core` `plant` | `+plant` |
   | `fsw` | `adcs_fsw.c` (shell), `adcs_fsw_int.h`, `adcs_params.c`, `adcs_math.c` | `fsw.rs` (shell), `params.rs`, `math.rs`, `hal.rs`, `cabi.rs` | `step.m` (shell), `init.m` |
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
| Reproducible builds | ✅ `rust-toolchain.toml` (1.94.1, firmware target); every `cargo build` and test `--locked`; every action pinned to a commit SHA; `setup-python` in every job that runs Python; `tools/requirements-ci.txt` pins pyflakes and numpy (figures are drawn by the engine, `adcs-plot`); cargo-mutants pinned |
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

**Progress of P2** (pseudocode v2):

| item | state |
|---|---|
| Guidance, mode management and FDIR in their own modules | ✅ C: `adcs_guid.c`, `adcs_modes.c`, `adcs_fdir.c`, the state in the private `adcs_fsw_int.h`; Rust: `guid.rs`, and `modes.rs`, `fdir.rs` as children of `fsw` (they work on its private state); twin: `mode_manager.m`, `fdir.m` (guidance was already `guidance.m`). Code moved verbatim. All 48 shipped scenarios flown with C and with Rust before and after: every recorded channel bit for bit the same (96 runs), C = Rust |
| The language: several outputs, settling loops, tables, arrays, state between ticks, units | ✅ `docs/PSEUDOCODE_V2.md`; one implementation in JavaScript (`design/js/pcode.js`): parser, unit and type checker (every output set on every path, no recursion, inputs unchanged), interpreter; records, `proc` with `state`, `settle … until … else`, `table` (step, linear), fixed arrays, vectors, matrices |
| Checker in the browser | ✅ `design/pcode_checker.html`: one offline page with the language inlined (checked in Chromium, phone width included) |
| Translators to Rust and MATLAB; translator = interpreter | ✅ `design/js/pcode_gen.js`, driven by `tools/pcode.py gen`. Vectors drawn from the interpreter, every `proc` as a run of calls with its state carried: Rust and MATLAB reproduce all of them, every function without a transcendental bit for bit (physics 530 of 540 values bit for bit, worst 7e-16; the language's self-test, bit operations and casts included, 744 of 744) |
| Every relation in `physics.toml` written in it | ✅ all 39, in `spec/physics/*.pc`, held to the registry (same functions, same arguments in order); translated to `engine/crates/adcs-physics` and `matlab_sils/+asils/+physics`; the 7 sourced test vectors of the seed content pass in the interpreter. Open technical items in `ADCS_GAPS.md` D14 |
| Every algorithm in `fsw/pseudocode/` written in it; C and Rust reproduce the interpreter on test vectors | ✅ `fsw/pseudocode/01`–`09` `.pc` (maths, time and frames with the IGRF-13 table generated from its one source, estimation, guidance, control laws, the step laws, allocation, mode manager and FDIR, drivers): 78 functions. Vectors from the interpreter in `fsw/tests/pcode_vectors.txt`, run through the hand-written C by `test_pcode` in `make test` (2616 calls, 38 696 values, all bit for bit but 60 values of functions with a transcendental, worst 6.6e-15) and through the Rust by `fsw-rs/tests/pcode.rs` (16 062 values, the same). The drivers go through their public paths over an in-memory HAL. The step laws' private-state functions were made callable with their state passed in (`adcs_fsw.c`; all 96 scenario runs bit for bit the same after); mode manager, FDIR and step laws are held in C, and Rust to C by closed-loop bit identity. Every function not called directly is named with its reason. A changed rounding, clamp or decode in the C was caught by the vectors |

**Progress of P3** (files in the browser; `docs/FILES_IN_THE_BROWSER.md`):

| item | state |
|---|---|
| SQLite inside the offline page | ✅ sql.js 1.14.2 (SQLite in WebAssembly) vendored in `design/vendor/`, pinned by SHA-256 (`VENDOR.toml`), inlined by `tools/pages.py` with its wasm: the file is read whole into memory and written back whole |
| Open and save files on Drive | ✅ the folder picked with the browser's folder access (Chrome, Edge); other browsers open one file and save by downloading. `design/js/tnfile.js` opens a file and checks it as `tools/tndb.py check` does; the saved file passes `tndb check` |
| "Open elsewhere" marker | ✅ `<file>.editing` beside the file (who, since, last seen, browser profile), refreshed every minute and carried by Drive; a Web Lock within one browser; a second editor gets the file read-only with the first one's name; a marker silent for 15 min is taken over and said so; a person's own crashed tab gives the file straight back |
| Conflict-copy detection | ✅ Drive's `name (1).ext` / `conflict` copies matched to their original, flagged in the folder and in the file, never listed as files of their own |
| History and undo | ✅ every change one undoable step (SQL triggers record the inverse; reals restored exactly); every save a `revision` row (who, when, what); Drive's version history named in the page |
| Crash-safe saving | ✅ every change kept in IndexedDB before it returns, offered back after a crash; the save goes through the browser's swap file, is read back and compared by hash, and is refused rather than overwrite a file changed on disk (the work goes to a named copy) |
| Size caps | ✅ 50 MB a node file, 500 KB a picture, from `design/schema.toml`, in the page and in the file layer |
| One component set, bundled fonts, no outside hosts | ✅ `design/js/tn_ui.js` + `design/css/tn.css` (light, dark, phone width); Inter and JetBrains Mono inlined; `tools/pages.py check` refuses a page with its own controls or styles and anything loaded from outside; built pages go to `build/pages/` and the kits, not to git |
| Proof | ✅ `tests/browser/files.test.mjs` in Chromium (11 tests, in `check_all` as `offline-pages`, in CI with Playwright): a node file saves to a folder, reopens, survives a crash before a save and in the middle of one, and refuses a second editor; conflict copies, changes on disk, caps, newer formats; from disk with no request leaving the page; phone width. A real Drive folder needs a person at the folder dialog: the tests use the browser's private folder, which has the same interface. The marker reaches other computers at Drive's sync speed; inside that delay a second save is refused or Drive makes a conflict copy, both reported |

**Progress of P4** (group app: structure; `docs/GROUP_APP.md`):

| item | state |
|---|---|
| Map | ✅ the group's nodes by stage, the edges between them, the inputs from other groups dashed (`graph` in `design/js/tn_ui.js`); click a node for its readers inside and outside the group and its actions |
| Contracts | ✅ the nodes other groups read, listed from the edges; publish or change a contract (output, unit, version, readers) |
| Stages and stage owners | ✅ add a stage, set its owner (a member), move a node between stages |
| People, issue node files | ✅ members and roles; issue a node to an author (the group file and the node file both name them, with the date) |
| Every structure action with its impact check | ✅ add, rename, stage, split, merge, archive, move between groups (`design/js/structure.js`): its impact (notes, changes, stops) before it is done; all or nothing across files (each file prepared and checked, the set recorded in `structure/actions/`, then written; a crash part-way finished from the record); history per group |
| Change requests | ✅ raised to another group, accepted or declined by its lead in its own file; moving a node into a group, and archiving or merging a node another group reads, wait for that group's acceptance |
| Proof | ✅ on the whole seeded design (20 groups, 734 nodes): all 20 groups open; `act` (138) and `catalogue` (8) restructured with every action; a node moved from `act` to `ctl` after `ctl` accepted; what another group reads kept until it agreed; someone else's open file stops an action; afterwards every file passes `tndb check` and the structure rules hold in both checkers (`structure.js` and `tools/group.py`, written separately and given the same broken folders). Under Node (`tests/test_structure.py`) and through the page in Chromium (`tests/browser/group.test.mjs`) |

**Progress of P5** (node app; `docs/NODE_APP.md`):

| item | state |
|---|---|
| All steps | ✅ identity, explanation, theory, inputs and output, value / requirement / evidence, pseudocode, results, evidence, pictures, code, belief record, feedback; each field with its question, why it is asked and an example; the kind (fixed by the tree, given by the spec, or chosen by the author for a row the spec has not named) decides the steps (`design/js/node_model.js`) |
| Starting values | ✅ what the spec already says about the row (seeded content) taken in with one click; never changes what the group decides |
| Equation helper | ✅ a palette of symbols and the node's inputs, inserted at the cursor, with how the equation reads |
| Uploads and picture wizard | ✅ PNG, JPEG, GIF, WebP; over 500 KB made smaller (JPEG, scaled) before it is kept; shown as images only |
| Results import | ✅ pasted from Excel or Sheets (tab-separated), shown as a table, kept in the node file |
| Live checks | ✅ the spec's intake rules for one node, in the page (units against the quantity, sources, inputs and contracts, the pseudocode checker, test vectors run by **Try it**, the explanation standard's marks, evidence metric and rung, belief record); evidence debt; each problem names its rule and its field; the catalogue they check against written from the spec (`tools/node_catalog.py`, current in `check_all`) |
| Preview | ✅ the node as a reader sees it, answer first (`design/js/node_view.js`) |
| Ready, sign | ✅ marked ready with a fingerprint of the content; signed as checked by someone other than its author; any later edit makes both stale |
| Contract updates | ✅ a changed contract on an input is acknowledged on Home; a contract change asked of the owning group as a change request |
| Proof | ✅ on the whole seeded design in Chromium (`tests/browser/node.test.mjs`, in `check_all` as part of `offline-pages`): declared, computed, KPI, evidence, closure, interface and an unnamed row filled and previewed through the page; the folder it leaves passes `tools/tndb.py check` and `tools/group.py check`. Each check rule by rule under Node (`tests/js/node_model.test.mjs`) |

**Progress of P6** (group app: assemble → release; `docs/GROUP_APP.md`):

| item | state |
|---|---|
| Assemble | ✅ every node file of the group read and judged (`design/js/release.js` `assemble`): where its work stands, its signatures and whether they still stand, its problems, its evidence debt, what changed since the last release (Progress); each node viewed as the main application will show it |
| Checks across nodes | ✅ an input from a node that is gone or archived, a quantity that does not match its source, a contract whose unit disagrees with its node, an input with no arrow on the map, a boundary with no contract |
| Comments | ✅ the lead comments on a node; the comment goes into the node file and its author sees it on Home in the node app |
| Stage signatures | ✅ a stage owner (only) signs a stage; the signature covers its nodes as they are and goes stale on any change |
| Seal | ✅ only the lead; refused while the structure is broken, a node file cannot be opened, or nothing changed; `releases/<group>-<version>.tnrel` frozen with SHA-256 fingerprints over every node; versions 1.0, 1.1, … |
| Confirmed or UNCONFIRMED | ✅ confirmed only when checked by someone other than its author and unchanged since, no problem in it or across nodes, its stage signed; **a computing node without a test vector from outside the code is never confirmed**; every other node sealed UNCONFIRMED with its reasons |
| Node files stamped and kept | ✅ every node file stamped with the release (a status line, so no signature goes stale) and sealed: the node app opens it read-only and says so; the release keeps each node whole (content, inputs, test vectors, pictures, signatures) |
| Re-issue | ✅ the lead opens a sealed node again, as it is or as any release sealed it; a missing or damaged node file is made again from a release |
| Compare | ✅ two releases, or a release and the group now, node by node, with the fields that changed |
| Import of today's node forms | ✅ `adcs-node-form/1` files into the node files: the form's answers, inputs, test vectors, attachments and belief record; the author's own fields kept; refused while sealed or with no requester |
| Proof | ✅ on the whole seeded design under Node (`tests/js/release.test.mjs`): every group seals 1.0, re-issues a node and seals 1.1; act shows the confirmed rule, the stage signature, re-issue from a damaged file; env imports a node form. Through the page in Chromium (`tests/browser/release.test.mjs`): act, env and catalogue. Every release checked again by `tools/release.py`, which also finds each release broken on purpose (`tests/test_release.py`, in `check_all` as `release`) |

**Progress of P7** (manuals and usability; `design/manual/`, `docs/USABILITY_SESSION.md`):

| item | state |
|---|---|
| Role guides | ✅ one manual source, `design/manual/`: the journey of a node, a guide for authors (and checkers), for group leads (and stage owners), for developers, for users, the guide to TRI-NETRA Files, the glossary; every page in the explanation standard's shape (its one line first). `tools/manual.py` writes it into the apps (`design/js/manual.js`); `--check` in `check_all` |
| In-app manual | ✅ **Help** in every app's header opens every guide in place, the app's own role first; works offline, at phone width |
| Tours | ✅ one per screen (`design/manual/tours.toml`), each step pointing at the thing it names; shown by itself once to someone opening the app for the first time, again from Help; every target checked to be in its app |
| Field help | ✅ every field, choice and set of checks in the three apps has help beside it (checked by `tools/manual.py`); the node app's step fields carry their question, why it is asked and an example |
| Print | ✅ any manual page, a node's preview (node app), a node's view (group app, Assemble and Progress) print alone, without the controls; the whole page prints without toolbars |
| The journey diagram | ✅ author → checker → stage owner → lead seals → developer team builds → lead accepts → release, with the ways back; drawn by `tools/manual.py` (`design/manual/journey.svg`), in the manual |
| Proof without people | ✅ `tests/browser/help.test.mjs` as a newcomer: the tours, Help, printing, help on every form field, phone width; then a walkthrough found only by what the screens say (labels, tabs, buttons): an author fills a node and marks it ready, a colleague checks it, the stage owner signs, the lead seals act 1.0 with that node confirmed |
| Usability session | ⏳ the kit is ready (`docs/USABILITY_SESSION.md`: tasks, what the observer writes, how findings become fixes; a findings sheet on Drive); it needs 2–3 members chosen by the owner |
| Fixes from it | ⏳ after the session |

**Progress of P8** (all content carried over; `docs/CARRY_OVER.md`):

| item | state |
|---|---|
| Every node filled from what exists | ✅ `tools/carry_over.py`: the spec's seed content into the nodes' own fields; each physics relation as pseudocode (48 compile in the node app; every one with test vectors reproduces them); case keys and suppliers; KPI senses and metrics; algorithm parameters; the tree's notes; each item marked with its origin, nothing replaced that an author wrote |
| Rows a discipline had no node for | ✅ 31 added from the code (`design/carry.toml`): dynamics (rigid body, kinematics, rotor coupling, flexible mode, total momentum), onboard navigation (orbit propagation, GNSS fix, time and frames, TRIAD, QUEST, gyro filter), guidance and mode management, FDIR (sensor and rotor health, safe mode), CMG and VSCMG (model, steering, axes, gimbal limits), sizing per subsystem; each with its flight pseudocode and implementations where there is one |
| The 166 internal rows | ✅ named from the code that computes them (sensor and actuator models, disturbance torques, control laws, estimation, modes, flight software, budgets, rigs), in the group file and the node file; their kind is left to their author |
| What does not exist | ✅ listed in every node (`status.gaps`) with its owner team, on the node app's Home and the group app's Progress; nothing invented. Most is explanation (764 nodes) and belief records (357), which only people write |
| Proof | ✅ `tests/test_carry.py` (`check_all` as `carry`): every file and structure rule holds after the carry; origins, names, added rows, gaps; carrying again changes nothing; the node app's own code reads every node and runs every carried pseudocode. The Drive pack's Design folder is seeded and carried |
| Drive shared drives for the 10 lead teams | ⏳ needs the owner's Drive admin: one shared drive per lead team holding its groups' files |

**Progress of P9** (rules and developer intake):

| item | state |
|---|---|
| `group verify` / `group merge` → `design.tndb` | ✅ `tools/group.py verify DIR`: every group's latest release checked by `tools/release.py` and against the folder (its nodes still the group's, what they read exists, contract readers are groups); `merge DIR`: every latest release into `design.tndb` (the previous kept as `design.tndb.prev`), refused while any release does not verify |
| The catalogue of outputs | ✅ `merge` writes `catalogue_output`: every output of every sealed node, its unit, its contract version and the groups that read it |
| The impact listing across groups | ✅ `tools/group.py impact DIR NODE`: who reads a node, in its group and across groups, transitively, and the contracts on it |
| Every command: explain, steps, dry run, what it checks, how to undo, where its code is | ✅ all 56 commands in `docs/commands.toml` carry `checks` and `undo`; `tools/trinetra.py explain` and `dry-run <command>` print them with the code's location; `docs/COMMANDS.md` generated; `tests/test_commands.py` requires both |
| `CONTRIBUTING.md`, `docs/CHANGING.md` | ✅ who changes what (authors, stage owners, leads, developers), the rules nothing bends; `CHANGING.md` names the source, the command and the check for node content, structure, the design format, the carry map, the manual and the apps |
| SPEC §3.2 and §5.10–5.11 | ⏳ the rewrite is proposed in `docs/RULES_PROPOSAL.md` and waits for the owner's approval; SPEC.md is unchanged until then |
| Proof | ✅ `tests/test_release.py`: all 20 groups, sealed, verify and merge into `design.tndb` (734 nodes, every output in the catalogue); a release its group has moved on from does not verify and is not merged |

**Progress of P10** (developer skill and agents; `docs/GENERATORS.md`):

| item | state |
|---|---|
| Coordinator skill, backend agent, frontend agent | ✅ `.claude/skills/trinetra-coordinator/SKILL.md` (verify → merge → wire → generate → test → deliver → prove), `.claude/agents/backend.md` (translator first; never a node's content), `.claude/agents/frontend.md` (one component set, help on every field, proven in a browser) |
| `group wire / build / test / deliver` | ✅ `tools/groupcode.py wire` (each group's computing rows into `design/groups/`, a function shared by groups written once), `gen` (Rust `adcs-groups`, WebAssembly `adcs-groups-wasm`, MATLAB `+asils/+groups`), `test`, `deliver` (a test app per group) |
| Every computing row's code generated from pseudocode and tested | ✅ 48 rows in 9 groups have pseudocode today (the rest are gaps their authors fill, P8): every one in the generated code; the Rust reproduces the interpreter on every drawn vector (76 functions) and every node's own test vectors; the twin reproduces the same 608 vectors in Octave |
| Engine = twin per group module | ✅ the same vectors through the generated Rust and MATLAB; C = Rust stays the flight software's parity (`engine.py fsw-parity`) |
| WebAssembly for "try it" | ✅ the generated Rust compiled for `wasm32-unknown-unknown` (88 KB), inside each test app; every test vector passes in the interpreter and in WebAssembly, the two agreeing (`tests/browser/testapp.test.mjs`, 20 apps) |
| Generators explained by example | ✅ `docs/GENERATORS.md`: one node (`gd_0`) from its pseudocode to Rust, MATLAB, the tests and its test app |
| Today's engine numbers unchanged | ✅ the generated crates sit beside the engine; `check_all` keeps flying the engine's own tests and parity |
| As groups seal real releases | ⏳ `wire --design DIR` on the merged releases replaces the carried design; the loop is the same |

**Progress of P11** (main app on databases; `docs/MAIN_APP.md`):

| item | state |
|---|---|
| `design.tndb` holds the engine's inputs | ✅ format 2: every case line by line (`design_case`, each value with the node that declares it) and every input file under `data/` (`engine_input`, with its fingerprint); seed and merge fill them (`tools/design_inputs.py`); version 1 upgrades |
| The engine and the app fly from it | ✅ `adcs-sim/src/source.rs`: with `TRINETRA_DESIGN` (or the kit's own `design.tndb`) every case and `data/` path comes from the database alone; a flight from a data folder holding no case and no input has the same result id, metrics and channels (`tests/test_design_source.py`); kits and the wheel carry `design.tndb` |
| One component set, one plotting module | ✅ the app's page rebuilt on `tn_ui.js` (`tools/pages.py app`, compiled in; browser test `tests/browser/app.test.mjs`); `adcs-plot` draws every figure as SVG or PDF, engine and twin runs alike; matplotlib gone from `tools/` |
| SVG/PDF and a report per run | ✅ `adcs figures`, `adcs report` (HTML and PDF), `adcs plot` (any figure as JSON); the app's `/v1/figures`, `/v1/figure`, `/v1/report` |
| A run records its input hash and what differs | ✅ `inputs.input_hash`, `inputs.differs`, `inputs.design` in every manifest |
| Sweeps keep only what their figures need | ✅ a campaign keeps `summary.json` and run 1 (its provenance); about 4,500 committed per-run manifests left git |
| The Python package reads the database | ✅ `trinetra_adcs.design` (standard-library `sqlite3`) and `python -m trinetra_adcs.design` |
| Committed pages retired | ✅ `results/index.html` and its 210 figures, and the twin's 280 figures and result pages, out of git; drawn on demand |
| Every stale run re-flown; `adcs results stale` finds nothing | ✅ engine runs, campaigns, Monte Carlo series, solutions, soft OILS, the dispatch, both design loops and the 40 twin runs re-flown: `adcs results stale` finds 0 of 295 stale (the twin now records its source fingerprint, judged by the engine). `ais_img_3u` selects the same design; `ais_3u` is now closest, not feasible (gap D15) |

**Progress of P12** (test → deliver → accept → ship; `docs/DELIVERY.md`):

| item | state |
|---|---|
| The five waves as data | ✅ `wave` per group in `design/groups.toml`, checked by `tools/groups.py` |
| Delivery | ✅ `tools/delivery.py deliver`: the release verified, earlier waves first, merged, the generated code checked to be the release's, the group's tests run, its test app and a note in `deliveries/` |
| Acceptance in the group app | ✅ **Release → Deliveries → Accept**: the lead's signature naming the release's and the delivery's fingerprints; refused to anyone else, for a version not delivered, a delivery whose tests failed, or twice |
| Status and shipping | ✅ `tools/delivery.py status` and `ship`: every group accepted or visibly UNCONFIRMED, with why |
| Proof | ✅ `tests/test_delivery.py` + `tests/js/waves.test.mjs`: the five waves rehearsed on the carried design with stand-in leads (sealed, delivered, accepted, one left UNCONFIRMED, the refusals) |
| Every group accepted by its lead | ⏳ no lead named yet, so no group has sealed: today all 20 would ship UNCONFIRMED (owner's decision, §9) |

**Progress of P13** (both cases end to end from the design database; `docs/END_TO_END.md`):

| item | state |
|---|---|
| The database holds the files' inputs | ✅ `tools/design_inputs.py differences` empty: every case line and every `data/` file byte for byte, else the run is refused |
| The design loop and the campaigns from `design.tndb` alone | ✅ `tools/end_to_end.py`: every engine run with `TRINETRA_DESIGN`, the soft OILS runs (QEMU, C and Rust firmware) included; `ais_3u` 468 and `ais_img_3u` 555 numbers compared with the ones flown from the files: none differ (`results/END_TO_END.md`) |
| Every row evaluated or shown as not computed | ✅ `tools/evaluate.py`: per case about 45 rows stated, computed by their pseudocode or supplied by the selected design's Monte Carlo; the rest each with why (no pseudocode yet, supplied by a lab or supplier) (`results/EVALUATION.md`) |
| Every closure answering or blocked by name | ✅ 38 per case: 4 pass (APE, AKE, detumble time, orbit-average power), 34 blocked, each naming the row with no value. `ais_3u`'s APE passes at the KPI's claimed probability while the design loop's 99.73 % line-of-sight check is gap D15 |
| The traceability | ✅ `results/TRACEABILITY.md` regenerated from the same run |
| More closures answer | ⏳ as authors write pseudocode and leads release their groups (P12); `tools/evaluate.py --check` refuses a closure that answered before and is blocked now |

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
