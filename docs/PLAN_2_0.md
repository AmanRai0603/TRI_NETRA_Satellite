# The plan to 2.0.0

> **Answer first.** One release, **2.0.0**, at the end of S17. Before it, the database and the code are built
> **in parallel** and everything is fixed; the people take the design over only **after** the release
> (re-planned on the owner's word, 6 Oct 2026, "The re-plan of 6 October", below). The stages to 2.0.0:
> 1. ground and the model (S0, S1);
> 2. the database (S2–S4);
> 3. everything computed from the database, the flight software included (S5–S7);
> 4. the technical upgrade, made by the developer in the code and in the database together (S11–S16);
> 5. the application: open the database, see it, run the engine, see the impact (S8);
> 6. the one release (S17).
>
> After it, the ownership stage (S18–S21): people, keys and the drive; the switch-over; each group taking its
> branch over, understanding it, breaking it down further and editing it in the application, release after
> release.
>
> At 2.0.0:
> - one application, installed, which you point at the database and which shows the whole design, runs the time
>   engine (SILS, campaigns, soft OILS) and shows the impact of any change on every closure and KPI;
> - the whole design lives in the database: the relations, the catalogue, the cases, and the flight software's
>   algorithms, parameters and modes;
> - the repository carries the code **and** what the code generates from the database, committed, so each also
>   builds, runs and is tested on its own; a check fails whenever the two drift apart;
> - code means the toolbox, the engine's core, the translators, the application, and what the onboard computer
>   and the test rigs need to run (the flight software's runtime, soft OILS, OILS and HILS);
> - until the ownership stage, the developer keeps both in step; nothing is signed by a person yet.
>
> **Zip 1** (S4) put the whole design, converted and proven to give 1.0.0's numbers, on the drive. Corrected
> conversions (zip 1.x) replace it while the developer still owns the design. **Zip 2** (the application) comes
> with the release (S17). Every phase is proven in CI and merged into `main` on your word. Nothing is tagged until
> S17.
>
> **Kind:** explanation + plan · **For:** everyone · **Status:** decisions taken **as proposed** by the owner (5 Oct 2026),
> re-planned on the owner's word (6 Oct 2026: no trial and no people before the release); in progress, phase by
> phase ("Progress", at the end)

What 2.0.0 is, is in two documents:
- **the design model**, `docs/SYSTEM_MODEL.md`, including what is database and what is code (§7, and §7.1 for
  the flight software);
- **how it is operated**, `docs/OPERATING_2_0.md`: the valves and the daily rhythm, the roles, the application
  and its workspaces, today's design, the health map, how data comes in, every file, version and folder, each
  step from W1 to W16, and who hears what.

This page is only how we get there. It replaces:
- `docs/DATABASE_FIRST_PLAN.md` (D1–D12 are placed in the last section);
- the release numbering of `docs/TECHNICAL_ROADMAP.md`: its phases U0–U5 become S11–S16, inside this one release.

The phase ids are S0–S21 so they do not collide with the roadmap's item ids (E, M, G, F, A, Y, T, P, O, TS, DD,
DB, D, B). They keep the numbers they were given on 5 October, so every commit and result that names one still
points at the right phase; the re-plan changed their order, not their names. The order is: S0–S7, S11–S16, S8,
S17, then S18–S21 after the release (S18 and S19 were S9 and S10).

## Why one release, 2.0.0

TRI-NETRA has shipped 1.0.0. What changes now is larger than a minor version:
- the file formats;
- the rules;
- the application (three pages and a desktop app become one);
- the roles;
- where the design lives (the flight software's algorithms included);
- the technical corrections the audit found in 1.0.0.

It ships **once**, when all of it is done and proven together, so nobody works on an in-between state:
- zip 1 (and its corrections, zip 1.x) put the database on the drive; zip 2, the application, comes with the
  release;
- there is no trial of an in-between application and no daily work on the drive before the release;
- the tag `v2.0.0` comes only at S17.

After it, once the ownership stage (S18–S21) has handed each group to its people:

| what | made by | when |
|---|---|---|
| an application release, the code | the developer | rarely |
| today's design | the application, from the drive, for everyone | whenever anyone opens it |
| a released design, with its flight images | the system engineer, and no one else | when today's design is right |
| a programme decision | the programme manager, as a release of the programme's branch | after a released design |

## The re-plan of 6 October

On 6 October 2026, with S0–S5 done and S6 under way, the owner changed the order: **fix everything first, with
the database and the code in parallel, release, and only then hand the design to its people.** Trying an
in-between application, naming people, keys and signing in the middle are of no use before the design is
corrected. What changed:

| was (5 October) | now (6 October) |
|---|---|
| S8 application, then your trial in every role, then S9 people, S10 switch-over, then the upgrade S11–S16 signed by people | the upgrade S11–S16 made by the developer in the code and the database together, then S8 the application, then S17 the release |
| S9 people and zip 2; S10 switch-over | zip 2 at the release (S17); people and switch-over after it, as S18 and S19 |
| groups break their branches down "after 2.0.0", one line | the ownership stage: S20 each group takes its branch over, S21 breakdown and editing |
| design changes by corrected conversion until S10, then people only | by the developer, in the database and the code in parallel, until S19; then people only |
| the application proven by CI and by your trial | proven by CI; you first use it as released |

What did not change: the phases' names and contents up to S7, the database/code boundary, the parity gates, one
release with one tag at S17, and that no assistant supplies a relation.

**Where it stood on 6 October** (checked that day): S2–S5 done; S6 steps 1–4 done (the C and Rust flight
software's algorithms generated from the design, 48 of 48 scenarios identical, C equal to Rust bit for bit), its
steps 5 and 6 under way (S6 done on 7 October); the database and the code generated from it in step (`tools/from_design.py --check`,
`tools/flight_build.py gen --check`: no difference); S0 and S1 waiting for the owner's approval; the work of S0–S6
on the development branch, not yet merged into `main`. The design itself is mostly unfilled: of 1,084 rows, 43
(`ais_3u`) and 48 (`ais_img_3u`) hold a value, and 34 KPI closures per case are blocked (`results/HEALTH.md`,
`results/END_TO_END.md`); the upgrade S11–S16 and S7 fill and correct it before the release.

## The one idea every phase serves

**The design never passes through the code again, and everything that computes is generated from it.** The
developer moves the design out of the repository once, at S3.

**Until the release, the database and the code run in parallel.** The database is the source. The repository
carries what is generated from it (the engine's inputs, the flight software's algorithms and parameters, and from
S7 the engine's models and the twin's functions), committed, so the code builds, runs and is tested on its own,
without the application. Every change, whether a fix or an upgrade, goes into both in the same commit: the
design as a revision of the database, the code by regenerating it. A check fails the build whenever the two drift
apart (`tools/from_design.py --check`, `tools/flight_build.py gen --check`, and S7's). Until the ownership stage,
the developer makes those changes; each design change is marked as the developer's, not yet signed by a person.

From the ownership stage on, the design is written, checked, combined, run, signed and released in the
application, by the people who own it. The time engine's models, the flight software's algorithms and the MATLAB
twin's functions are all generated from it. A design change never waits for a developer, and a developer never
edits a design.

Everything the developer does today on a group's release becomes checks in one library, which the application
runs at every valve. Today that work is:
- verify and merge (`tools/group.py`);
- generate (`tools/groupcode.py`, `tools/gen_fsw_params.py`);
- deliver a test app and accept it (`tools/delivery.py`);
- build `design.tndb`.

## Database and code: the boundary

| in the database | in the code |
|---|---|
| every relation: physics, environment and disturbances, device models, sizing laws, and how each metric is measured | the toolbox the methods call: maths only, with units (arithmetic and elementary functions, vectors, matrices, quaternions, interpolation, root finding, linear and Riccati solvers, random streams, integrators) |
| **every published model, with its data** (owner's word, 7 Oct 2026): the atmosphere (DTM2020, JB2008, the exponential table), the planets' positions (DE440's Chebyshev method and its coefficients), the magnetic field (IGRF), gravity (EGM coefficients, spherical harmonics), time scales and leap seconds, Earth orientation and frames (IAU 2006/2000A, EOP, tidal terms), tides, relativity, space weather indices | the readers of files' formats (SQLite, DAF/SPK, CSV, the space-weather and EOP files) that load published data into the database, never models of their own |
| **the flight software's algorithms**: estimation, guidance, control, detumble and Sun-acquisition laws, allocation, mode management, FDIR, the drivers' conversions | **the flight software's runtime**: HAL, C interface, tick and scheduler, the configuration blob's format, the targets (POSIX, QEMU Cortex-M, boards) |
| the flight parameters, the mode list, the tables (IGRF coefficients) | the flight build: generate, compile, check, seal |
| the catalogue, the cases and their variations, the KPIs and their metrics, dispersions and campaigns | the time engine's core: step order, recorder, the calls to generated models and metrics |
| wires, mounts, closures, loops; states, maturities, ranges | **the rigs**: the soft OILS emulator and byte link, the OILS and HILS rig host, emulation channels, timing measurement |
| signatures, seals, versions; the flight images of each released design | the interpreter and the translators (Rust, C, MATLAB); the library; the application; the CLI and Python package |

## The code, as 2.0.0 arranges it

| part | is | where | runs |
|---|---|---|---|
| **toolbox** | the maths the methods may call, with units, and nothing that is a model of the world or the spacecraft | `fsw/pseudocode/01_math`, the maths part of `adcs-sim-core` and `adcs-pop` (integrators, linear algebra), `+asils/+pc` | everywhere |
| **library** | reads, writes and checks every file kind; the interpreter; the design graph; today's design; the health map; signing | `trinetra-design`, grown from today's read-only reader | installed (native) and in the page (WebAssembly) |
| **translators** | pseudocode to Rust, C and MATLAB | today `design/js/pcode_gen.js` (Rust, MATLAB); C is new | in the flight build and the engine build |
| **time engine** | its core, plus models generated from the design | `adcs-sim-core`, `adcs-sim`, `adcs-pop`, `adcs-design` | installed; `adcs-sim-core` (no_std) also in the page |
| **flight software** | the runtime, plus algorithms generated from the design | `fsw/` (C), `fsw-rs/` (Rust), `adcs-fsw-abi`, `fsw/targets/` | SILS, soft OILS (QEMU), OILS and HILS (boards) |
| **rigs** | soft OILS emulator, byte link, rig host | `fsw/targets/link`, `tools/engine_oils.py` | installed |
| **application** | five workspaces over the library, the engine and the flight build | `trinetra-app` and `apps/TRI-NETRA.html` | everyone |
| **CLI, Python, tools, twin runner** | scripted use, CI, the release; the MATLAB runner with generated functions | `adcs-cli`, `python/trinetra_adcs`, `tools/`, `matlab_sils/` | the developer and CI |

**One library, two hosts.** The library's core is plain Rust over rows: the checks, the interpreter, the graph,
the health map and the signature chain. Each host does its own file I/O:
- the installed application and the CLI use SQLite (`rusqlite`, as `trinetra-design` does today);
- the page uses the vendored `sql.js` (`design/vendor/sqljs`), as the three pages do today.

Today's JavaScript and Python implementations become the library's test oracles until the release, then
retire.

## Who may change the design, and when

| period | a change to the design is made by |
|---|---|
| until S3 | the developer, in the repository, as today |
| S3 to the release (S17), and on until each group's handover in the ownership stage | the **developer**, in the database and the code in parallel: each change a revision of the database with its source, and the code regenerated in the same commit, proven by the parity checks. The drive gets them as corrected conversions (zip 1.1, 1.2 …), replacing `groups/` and `cases/`. Each is marked as the developer's, *not yet signed by a person* |
| from the switch-over (S19) | **people only**, group by group, in the application. The developer may *propose* a revision: a transcription from a cited source, or from a fix proven in code. It arrives in the node engineer's My work, who checks it against the source and signs it, or does not |

No assistant supplies a relation. A transcription names its source; until the ownership stage it is marked
unsigned, and after it is signed by the person who checked it.

## While it is built

- **Nothing changes for anyone until the release (S17).** 1.0.0 stays the released tool. Nobody is writing
  design files on the drive yet, so no work has to be frozen or upgraded.
- **The repository's design data is frozen at S3.** The 1.0.0 content is kept read-only under
  `archive/design-1.0/` until 2.0.0 ships, then deleted.
- **The new application is built beside the old, not patched into it.** The old pages, the delivery tool and the
  test apps retire at the release; their editing work (writing, sealing and delivering a node or a group) comes
  back in the application during the ownership stage.
- **No trial in the middle.** Nobody lives with an in-between application. The application is proven by CI on
  the converted design and the example group (S8), and you first use it as released (S17). Understanding,
  breaking down and editing the design, and handing it to its people, is the ownership stage after the
  release.

## The phases

| stage | phase | delivers | your part | size |
|---|---|---|---|---|
| **1 · Ground and model** | S0 Safe ground | designs refused by an engine that cannot run them; results name their design; 1.0.0 notes corrected | approve the notes | S |
| | S1 Rules, roles, the model | the model, the operating model, the code's architecture; the rules; role names; the mount table; the database/code boundary | **approve, with a second reviewer** | S–M |
| **2 · The database** | S2 Files, versions, keys | format 2 for every kind; versions; signatures; the one library with every check; the interpreter in Rust | — | L |
| | S3 The design leaves the repository | the tree, the catalogue, the cases **and the flight software's algorithms, parameters, modes and tables** converted into group files and cases | — | L |
| | S4 The design runs → **zip 1** | the design graph and the time engine read the design; today's design; health map; the declared loop; **parity gate**; zip 1 | **upload zip 1** | L |
| **3 · Everything from the database** | S5 Translators complete | a C translator; Rust, C and MATLAB covering every construct the flight software and the models use; each held to the interpreter | — | M |
| | S6 Flight software from the database | C and Rust algorithms generated from the nodes; the runtime kept; the flight build; images proven equal to 1.0.0's software in SILS and soft OILS; hand-written algorithm code deleted | — | L |
| | S7 Every relation from the database | every built-in relation written as a method (transcribed, signed later) and proven equal; the engine's and the sizing's models generated; the twin's functions generated; **built-in count zero** | — | L |
| **4 · The technical upgrade, code and database together** | S11 Corrections (U0) | 1.0.0's wrong results and critical defects fixed, in the code and in the database in the same commit, each with a test that failed before | review each phase's merge | M–L |
| | S12 Environment, frames, requirements, orbit (U1) | cited environment and frames; the propagator's fixes | review each phase's merge | L |
| | S13 Margins, statistics, budgets (U2) | stability margins, statistics with confidence, budgets by error class, independent referents | review each phase's merge | L |
| | S14 Devices, GNC, completeness (U3) | devices against datasheets, the GNC items, FDIR, the FMECA and its fault matrix | review each phase's merge | L |
| | S15 Flight-software assurance (U4) | robustness, TM/TC, coverage and MISRA on the generated code, reproducible flight builds, Renode soft OILS | review each phase's merge | L |
| | S16 OILS and HILS ready (U5, code side) | board targets, the rig host, timing, the HILS procedures as cases; proven on emulation | **choose the board and the lab (H1, H4)** | M–L |
| **5 · The application** | S8 The application | one installed application (and its page): open the database, see the whole design and its health, run the time engine (SILS, campaigns, soft OILS), see the impact of a change on every closure and KPI; the flight build | — (proven by CI, no trial) | L |
| **6 · The one release** | S17 Release 2.0.0 | the whole chain from the drive, every closure answered or named, the release notes, the tag | **say "Ship 2.0.0", push the tag** | M |
| **7 · Ownership, after 2.0.0** | S18 The people and the drive (was S9) | a guide per role; START HERE for people; keys; sharing | **name people, share, register keys** | S–M |
| | S19 Switch-over (was S10) | the first released design signed by people, with its flight images; `env` round the whole cycle, no developer | **do the round** | M |
| | S20 Each group takes its branch over | the developer's revisions since S3 signed or answered by their owners, group by group; understanding through the application's views | **sign, or ask** | L |
| | S21 Breakdown and editing | groups break their branches down further and edit them in the application; each change released as usual; new features as application releases | the groups | continuing |

## Stage 1 · Ground and model

### S0 · Safe ground

- A design names the application version and toolbox it needs. The engine, the desktop app and the Python package
  refuse a mismatch by name.
- Every result carries the design's fingerprint and the engine's version, and is marked stale when either changes
  (extends `docs/RESULTS.md`).
- The faults found in reading a design are fixed, each with a test that failed before:
  - `adcs size` under a database;
  - DE440 not found from an empty working directory;
  - `evaluate` on the pack's root;
  - a folder or an unreadable database not refused clearly.
- The pseudocode checker page (`design/pcode_checker.html`) is rebuilt when the interpreter it embeds changes.
- The published 1.0.0 release notes are corrected: the KPI passes were not demonstrated (A1), power is actuator
  power only (Y1), and the CMG cluster is counted four times (Y2). You approve the text and edit the release.

**Done when** a design built for another engine is refused in a test, and every result names its design.

### S1 · Rules, roles and the model

- **The model:** `docs/SYSTEM_MODEL.md` and `docs/OPERATING_2_0.md`, reviewed and approved.
- **The code's architecture:** `docs/CODE_ARCHITECTURE.md`, the two tables above written out: what each crate and
  folder does, which part may call which, and the database/code boundary as a rule. A relation, an algorithm, a
  parameter or a table found in code outside the generated files is a defect.
- **The code's rules:** `CONTRIBUTING.md`, `docs/CHANGING.md` and `.claude/agents/`, rewritten for a repository
  that holds only code.
  - These stay:
    - refuse, never guess;
    - never edit a generated file;
    - the twin moves with the engine, and the C runtime with the Rust;
    - every toolbox formula is traced to its source.
  - These are new:
    - an application release gives the current released design's answers, and its flight images' behaviour,
      unchanged (W15);
    - a translator change is proven on every node's vectors before it ships.
- **The design's rules,** as checks for the library and as words in the operating model:
  - one writer per file, and the signature chain;
  - an expected value never comes from the code under test;
  - no relation supplied by an assistant; a transcription signed by the person who checked it;
  - every requirement says which way it binds;
  - a node reads its children only through their ports;
  - a release is never edited;
  - a parameter is changed only by the level that owns it;
  - every valve's owner controls what passes it;
  - the design is released by the system engineer alone;
  - today's design is never taken for a released one;
  - evidence counts only from results made by the design (and the flight image) it is judged in.
- **The roles:** programme manager, system engineer, subsystem engineer, node engineer, developer, each with a
  deputy, and the checker's signature where a group asks for it. They are used on every page and in
  `docs/GLOSSARY.md`.
- **The tree:**
  - the mount of every group;
  - `programme` and `systems` as groups, and `case` dissolved;
  - the flight software's owners: `nav`, `gdn`, `ctl`, `fdir`, `act`, `sens` and `fsw` (`docs/SYSTEM_MODEL.md`
    §7.1, §8).
- `docs/RELEASE_PLAN.md` §1 and `docs/RULES_PROPOSAL.md` are marked as describing 1.0.0.

**Done when** you approve, with a second reviewer.

## Stage 2 · The database

### S2 · Files, versions and keys

- **Format 2** of `design/schema.toml`, for every kind in the operating model §10:
  - block (parent to any depth, perspective, behaviour, contract version, owner);
  - port (type, unit, range with reasons, state, maturity, direction, owner and gate while open, bundle);
  - wire, mount, closure (by analysis or by evidence), loop;
  - case and variation;
  - text, table, media;
  - signature, change, request, issue, preview, answer, integration record;
  - flight image.
- **Port types:** number with unit, whole number, choice, yes/no, list and parameter first. Fixed-size vectors
  and matrices are added too, for the flight algorithms' states and gains. Table, time series and uncertain value
  come later.
- **Identity and versions** by the operating model §13.
- **Keys:**
  - make, lock by passphrase, sign, and check through the chain;
  - Ed25519, with a PBKDF2 and AES-GCM lock, the same in Web Crypto and in Rust.
- **One library**, `trinetra-design`, grown into it. It reads, writes and checks every kind, natively and as
  WebAssembly.
  - Every check today's tools and pages run moves into it:
    - `tndb.py check`;
    - `group.py verify`;
    - `delivery.py`;
    - the node app's live checks;
    - the group app's impact checks;
    - the seal.
  - What is checked: the signatures, the method and its units, the cases, both ends of every range, the
    de-risking record, and the assistant rules.
- **The pseudocode interpreter in Rust**, held equal to `design/js/pcode.js` and `tools/pcode.py` on every test
  vector, including the flight software's vectors (`fsw/pseudocode/`).
- **Upgrade:** a format-1 file upgrades when opened and keeps a copy. Nothing a group wrote is dropped.
- **Comparison:** any two revisions, releases or designs, node by node.

**Done when:**
- the 765 node files and 20 group files of 1.0.0 upgrade with nothing dropped, field by field;
- signatures check through the chain;
- the library refuses everything today's tools refuse;
- the Rust interpreter equals the two existing ones on every vector.

### S3 · The design leaves the repository

This is the last time the design passes through the code.

1. **The tree** (`spec/`, `design/groups.toml`, `design/carry.toml`) becomes:
   - the programme's branch, the systems branch, and one branch per group (21 groups);
   - perspective tags, mounts from the interface rows;
   - wires from the 238 derivation and 23 contribution edges, and closures on the blocks they close;
   - the proposed next level as open blocks;
   - parameters at the level that owns them.
2. **The relations:** the 39 physics relations (`spec/physics/*.pc`) and every node's pseudocode become methods.
   Relations still only in compiled code become **built-in**, keyed by node id, for S7 to remove.
3. **The flight software's design** becomes nodes of its owning groups:
   - the algorithms of `fsw/pseudocode/03`–`09` (estimation, guidance, control, step laws, allocation, mode
     manager, drivers), each function a node, with the vectors as its cases;
   - the parameters of `fsw/params/params.toml`, each a stated node with its unit, range and source;
   - the mode list as a choice;
   - the IGRF coefficients as a lookup table of `env`.

   `01_math` stays as the toolbox; `02_time_frames_models` (time, frames, IGRF) moves into `env` in S7 (7 Oct
   2026).
4. **Every value gets its state, maturity and range** by rules that invent nothing:
   - stated with a source → decided;
   - a required row's bound → allocated;
   - computed → achieved;
   - otherwise → open;
   - maturity is estimated unless the source says otherwise;
   - ranges come from the nodes that carry them (82 design nodes, every flight parameter with limits).
5. **The library data becomes design:**
   - the catalogue: lookup and stated blocks;
   - dispersions, KPI definitions and waves: blocks of `vv`, `kpi` and `programme`;
   - the reference cases, the 48 scenarios, the 8 campaigns and the 12 trades: case files;
   - case lines with a node (162 of 213) are wired to it.
6. **Baseline releases 0.1** per group, sealed by the conversion and marked *converted, not yet signed by a
   person*. Every node shows unproven until signed.
7. **Readable copies** in `readable/*.csv`, and `readable/Conversion.csv`, which places every field of every
   1.0.0 node, every catalogue entry, every case line, every flight parameter and every flight function, or lists
   it as dropped. It must list none.
8. **The repository keeps its design data until S4's parity gate holds** (moved there on 6 Oct 2026). S4 compares
   the new path against today's, which reads these files. Archiving them here would break the engine and every
   test between the two phases. At the end of S4:
   - the repository keeps the regression copy (`tests/regression/design.tndb`), and later the example group
     (`tests/fixtures/`, S8);
   - `spec/`, `catalogue/`, `scenarios/`, `campaigns/` and `trades/` move to `archive/design-1.0/`;
   - `matlab_sils/data`, `matlab_sils/cases`, `fsw/params/` and `fsw/pseudocode/03`–`09` are still read in place
     by the engine, the twin and the flight software's builds, so they become **generated from the design** and
     checked against it (`tools/from_design.py`) until S6 and S7 generate code from the nodes;
   - the seed, carry-over and export tools leave the build: only the tests of the 1.0.0 node and group files
     still run them, on the archive, until the example group replaces it.

**Done when:**
- nothing is listed as dropped;
- every file passes the library's checks;
- converting twice gives the same files, byte for byte.

### S4 · The design runs → zip 1

- **The design graph** is built from the files on opening, installed and in the page. It runs every behaviour:
  method, children, stated, lookup, open (refused by name), built-in (by node id).
- **The time engine reads the design:**
  - case inputs from their nodes;
  - catalogue inputs from lookup blocks;
  - flight parameters from their nodes, written into the configuration blob that the 1.0.0 flight software
    already reads (`tools/fswcfg.py`).

  Each run's manifest lists every input's node and revision, and its metrics return as evidence to the
  `closure_verified` blocks.
- **One reader for every tool:** the design loop, campaigns, evaluate, trace, the pointing budget, the V&V
  report, the Python package and the CLI. The MATLAB twin flies an export.
- **Today's design** is built from the drive's folder:
  - each group's latest sealed release that passes its checks;
  - a refused release replaced by its last good one, and marked;
  - the application says which releases it used.
- **The declared loop** (sizing → mass and inertia → demand) is iterated on the `design` block. Any other cycle
  is refused by name.
- **Every value** carries state, maturity and range; **every closure** gives its range verdict and tornado;
  **every node** gets its health state, with roll-up and trace to cause.

**The parity gate.** The new path gives today's answer within each row's own tolerance, and refuses where today
refuses. It is checked on:
- every row and closure of `results/EVALUATION.md` and `results/END_TO_END.md`;
- the node verifier (`tools/verify_nodes.py`);
- every committed run re-flown from the design: both cases, every scenario, every campaign;
- engine-to-twin parity (`results/ENGINE_PARITY.md`);
- soft OILS on QEMU with the parameters from the design.

**Then the repository's design data is archived** (S3 step 8, moved here): the repository builds and tests with no
design data in it apart from the example group and the regression copy.

**Zip 1** is packed once the gate holds (`tools/drive.py`). It holds:
- `groups/`, `cases/`, `readable/`;
- the empty `design/`, `daily/`, `integration/`, `issues/` and `results/`;
- `guides/`: the model, the operating model, this plan, the engineering documents;
- START HERE;
- `MANIFEST.json`, with each file's size, SHA-256 and MD5.

**Your upload:**
1. In Trinetra Database, delete `Apps/` and `Design/`.
2. Move `Guides/` and the two sheets into `old-1.0/`. The check ignores that folder.
3. Upload zip 1's content, with "convert uploads" off.
4. Then either you run `python3 tools/drive.py --verify "<folder>" --first-upload` on Drive for desktop, or I
   compare every file's name, size and MD5 with the manifest through the Drive connector, read-only.
5. The parity gate is run once more on a copy of what is on the drive.

**Done when:**
- parity holds, installed and in the page;
- two computers build the same today's design;
- a broken node is traced to by name from the KPI it breaks;
- the drive passes the check.

## Stage 3 · Everything from the database

### S5 · Translators complete

- **A C translator** joins the Rust and MATLAB ones. It writes C99 with no allocation, in the conventions of
  `fsw/pseudocode/00_conventions.md` and the existing `fsw/src`.
- **Every construct the flight software and the models use** is covered in all three languages: fixed-size
  vectors and matrices, quaternions, state that persists between ticks, mode switches, saturation and
  rate-limiting, table lookups.
- **The translators move into the library's build**, so the application's flight build and the developer's engine
  build use the same ones.

**Done when** for every node with a method, its Rust, C and MATLAB translations equal the interpreter on every
vector: bit for bit where the language allows, within the stated tolerance otherwise.

### S6 · Flight software from the database

- **Generated algorithms.** The flight build writes:
  - the algorithm sources of `fsw/src` and `fsw-rs/src` from the nodes of `nav`, `gdn`, `ctl`, `fdir`, `act`,
    `sens` and `fsw`;
  - the parameter tables and the configuration blob's contents from their nodes (replacing
    `tools/gen_fsw_params.py`);
  - the IGRF table from `env`'s lookup.
- **The runtime stays code:**
  - `adcs_hal.h` and `hal.rs`;
  - `adcs_fsw.h` and `cabi.rs`;
  - the tick, scheduler and memory layout;
  - the configuration blob's format and CRC (`tools/fswcfg.py`);
  - `fsw/targets/` (POSIX, QEMU Cortex-M, the link).

  It is separated cleanly from the algorithms, so the boundary is the C interface and nothing else.
- **The flight build** generates the sources, compiles them with the runtime for each target, runs the nodes'
  vectors on the result, and seals it as a flight image. The installed application carries it with its
  toolchains (host, Cortex-M); CI uses the same one.
- **The time engine** loads the flight image built from the design it flies (POSIX for SILS, QEMU for soft OILS),
  and says which image it is.
- **Parity:**
  - the generated C and Rust flight software equals 1.0.0's hand-written software on every vector;
  - every SILS scenario and campaign gives the same metrics;
  - soft OILS on QEMU meets the same deadlines with the same timing (the B4 results);
  - C and Rust still agree with each other.
- Then **the hand-written algorithm code is deleted.** What remains in `fsw/` and `fsw-rs/` is runtime,
  generated, or test.

**Done when:**
- the flight images built from the converted design pass parity on every target;
- a change to a flight parameter or an algorithm node, made in the database, reaches the image and the metrics
  with no step beyond the flight build (a test; from S8 the application runs that build).

### S7 · Every relation from the database

- **Every built-in relation is written as a method.** That covers the time engine's device, environment and
  disturbance models (`adcs-sim-core`), the sizing laws (`adcs-design`), and any remaining row computed in code.
  - Each is a transcription of the code it replaces and of the source that code cites. It is marked as a
    transcription, to be signed by its node engineer after the switch-over.
  - Each is held equal to the code it replaces on its cases and across its range.
- **The engine's models and the sizing are generated** from the design by the Rust translator. Only the engine's
  core stays hand-written: step order, recorder, and the toolbox's maths (integrators among them).
- **The published models move too** (owner's word, 7 Oct 2026: "anything related to the engine, like the
  atmosphere model or the planets' positions, is part of the database; only what enables a run is code"). The
  atmosphere models, the planets' positions, the magnetic field, gravity, time scales, Earth orientation and
  frames, tides, relativity and the space-weather indices become methods of `env` and `orbit` nodes, with their
  coefficients and published data as the nodes' tables, each citing its publication. The engine and the twin
  generate them as they do every other method; code keeps only the file readers that bring the published data in.
  The metrics (how each KPI is measured from a run) become methods of `kpi` too.
- **The MATLAB twin's functions are generated** by the MATLAB translator. Its runner stays code.
  - **The twin flies the generated flight software** (the MATLAB translation of `fsw/pseudocode`), behind a
    hand-written tick, with the flight blob's parameters, in place of today's hand-written `+asils/+fsw`.
  - **The twin opens the database itself** (asked by the owner, 7 Oct 2026): `trinetra.open` reads a `.tndb`
    through the library's own command (`tndb read`, JSON over `system()`, no toolbox, MATLAB and Octave),
    `trinetra.build` generates every MATLAB function from it, and `trinetra.run` flies it. It replaces the export
    `tools/from_design.py` writes for the twin; packaged later as a MATLAB add-on.
- **The work, step by step:** `docs/S7_INVENTORY.md` (S7.1 to S7.19), each step with its parity check.
- **The generated group code** (`adcs-groups`, `adcs-groups-wasm`) is folded into this, and the per-group crates
  retire.
- **Parity** as in S4, on the whole chain, with nothing built-in.

**Done when:**
- the count of built-in nodes is zero;
- the parity gate holds;
- a search for a relation or a published model in code outside the generated files and the toolbox finds none
  (a check in `tools/check_all.py`).

Zip 1.x carries the converted methods to the drive as a corrected conversion, since nobody is writing there yet.

## Stage 4 · The technical upgrade, code and database together

Each phase is made by the developer, in parallel:
- the **code** half (toolbox, engine core, runtime, rigs, translators) by pull request, as always;
- the **design** half as revisions of the database, in the same commit: each a transcription from a cited source,
  or the design side of a fix proven in code, marked as the developer's and *not yet signed by a person*. The
  code generated from the design is regenerated with it, and the parity checks hold the two together.

Nobody signs anything in this stage. Every revision carries its source, the test that failed before it and the
comparison, so that in the ownership stage (S20) its owner can check it by reading, not by re-deriving. Each
phase ends with today's design rebuilt from the database, the generated code regenerated, every case re-flown
where it moved, and the drive given a corrected conversion (zip 1.x). Every fix lands with a test (or a case) that
failed before it. The item ids are those of `docs/TECHNICAL_ROADMAP.md`, `docs/TEST_BENCH_PLAN.md` (O, B),
`docs/TEST_STRATEGY.md` (TS) and `docs/ORBIT_PROPAGATOR_AUDIT.md` (P).

### S11 · Corrections (U0)

| area | items | now |
|---|---|---|
| verdicts | A1, A2, A3, A5, A7 | design (evidence closures, confidence) and code (evaluation) |
| power and sizing | Y1, Y2, Y4, Y5 | design (the sizing methods and budgets) |
| models | M1, M2, M5, M9, M10 | design (the device methods) |
| GNC logic | F2/G3, G1, G2, G4, G6, F4, F5 | **design** (the flight algorithm nodes), then new flight images |
| flight software | F1, F3, F10/G8 | design where it is an algorithm (F1's epoch, for one), code where it is the runtime |
| twin and density | E1, E6 | code (the twin runner) and design (density nodes, DTM2020 as a method of `env`) |
| orbit step | P5 | code (engine core) |
| soft OILS | O2, O12, O19, O20, O21, O11 (B0) | code (rigs) |
| tests | TS11, TS4, TS3, TS17 | code (CI) and design (cases) |
| decisions | the 15 design decisions of `docs/DESIGN_DECISIONS.md` | design (stated nodes with their reasons), signed by their owners in the ownership stage |

**Done when:**
- every verdict is demonstrated at its stated probability and confidence, or says "not demonstrated";
- power is whole-ADCS;
- the twin flies the flight frame;
- the gyro, reset and timing faults have scenario cases.

### S12 · Environment, frames, requirements, orbit (U1)

- Frames: E2, T5/E3, E4.
- Fields: E5, T4, IGRF-14.
- Environment details: E8, T6/E9, E10–E15.
- Sourced vectors for the physics rows, V1, and Y3 (mass with growth allowance, now graded by maturity).
- Requirements: Y14, Y15, Y17, Y18.
- The propagator: P1–P4, P6–P10, P12.
- Tests: TS1 and the environment scenarios.

**Done when** every environment model names its source and has a published-vector case, and every requirement is
stated or refused, never invented.

### S13 · Margins, statistics, budgets (U2)

- Stability and drivers: T1, T2. Each closure's tornado and range verdict is extended with the margins and
  worst-case search.
- Statistics and budgets: A4, A6, A8–A18, Y6.
- Estimator checks: S6 and G10.
- Credibility: T3/A13.
- Scope: D2, S1, S3.
- Independent referents: TS12 (Basilisk and 42 shared scenarios, Orekit), P13, the orbit validation campaign,
  CCSDS OEM export.
- Tests: TS2, TS10, TS9.

**Done when** every mode reports margins, and every claim either passes with confidence or names its drivers.

### S14 · Devices, GNC, completeness (U3)

- Sensors: T7, M7, T8 and M6.
- Actuators and dynamics: M3, M4, M8, T10, M11–M17, T12, T13.
- GNC: G5, G7, G9, G11–G16, T11, E7.
- Calibration and FDIR: T9, D6 and G12.
- System: Y7–Y13, Y16, with the FMECA as nodes of `fdir` and `risk`.
- Operations interfaces: TLE/SGP4, manoeuvres, lifetime.
- Tests: TS9 fault matrix from the FMECA, TS5 14-day soak, TS13 polarity and calibration as cases.

**Done when** each device is checked against its datasheet, the mode set matches ECSS-E-ST-60-30C, and the FMECA
drives the fault set.

### S15 · Flight-software assurance (U4)

- Robustness: F6, F7, the persistent context and golden blob, F11, F12. The runtime's parts are code; the gates
  and their thresholds are design.
- TM/TC: F8/T16 CCSDS and PUS-C, F13 parameter service (reading and writing the parameter nodes' values), F14.
- Evidence, on the generated code and the runtime alike: F15/T14 coverage, mutation, MISRA, proofs.
- F16 reproducible flight builds: the same design and application give the same image, byte for byte.
- T15 SEU injection, F17, F18, H3.
- Test bench B1: the Renode STM32F4 backend, a free-running OBC, HAL v2, adcs-link/2 (O3–O6, O8, O13, O14).
- Tests: TS6 SRS and verification control document generated from the design, TS7, TS8, TS16.

**Done when** a software verification report in ECSS-E-ST-40C Rev.1 shape is generated from the released design
and its flight images by CI.

### S16 · OILS and HILS ready (U5, code side)

- Board OILS (H1): the chosen board as a flight-build target; F9 WCET and FPU; H2 timing calibration.
- Test bench B2–B5: the IEU, the PREEMPT_RT rig host, PPS and TimeSync, Yamcs; the acceptance thresholds of
  `docs/TEST_BENCH_PLAN.md` §5.
- HILS procedures as cases, in the order ECSS-E-ST-10-03C Rev.1 and GEVS expect:
  1. polarity and phasing;
  2. Helmholtz-cage magnetometer and coil calibration;
  3. Sun-simulator sensor calibration;
  4. air-bearing closed loop;
  5. day-in-the-life against the engine.

**Done when** every rig and board target is proven on emulation (Renode, the link, a recorded rig), and each HILS
procedure runs as a case.

**Physical runs on the boards and in the lab are campaigns, not code.** Their results are evidence kept with the
released design, so they need no new release, and they start the day the hardware is on the bench. Whether 2.0.0
waits for the first board run is your decision (below).

## Stage 5 · The application

### S8 · The application

One application, installed and as a page, the same screens. You point it at the database (the drive's folder, or
a copy) and it shows the whole design and runs it. It is built after the technical upgrade, so it shows the
design as 2.0.0 ships it.

- **See:** the tree, group by group and node by node: every value with its state, maturity, range and source;
  every method, wire, mount, closure and loop; every case; who owns what. The health map with trace to cause, and
  each closure's range verdict and tornado.
- **Run:** the time engine on any case: SILS (C or Rust flight software), campaigns, sweeps and variations, and
  soft OILS (installed); the results, figures and report of each run, and which design and flight image made it.
- **See the impact:** change a value or a range in a variation of a case, run it, and see which closures, KPIs
  and budgets move, against the design as it is. A variation is a case file; it never edits the design.
- **System:** every ADCS budget, including the flight software's CPU, memory and deadline budget measured in soft
  OILS; the declared loop.
- **The flight build** in the installed application: build the design to try, and seal the images.
- **Ready for the ownership stage, not used before it:** My work, keys, signing, sealing, releasing, previews and
  requests, bringing data in (operating model §4, §7), each built and proven by CI on the example group, and
  switched on group by group at S19 and S20.
- **Made easy to use** by the eight points of the operating model §8.
- **It replaces** today's three pages, the desktop app's tabs, the test apps, node-form intake, `delivery.py` and
  `group.py merge`. They are removed at the release.

**Done when** CI drives, on the converted design and on the example group, installed and as a page:
- opening the database and building today's design;
- the health map traced from a failing closure to its cause;
- a case, a campaign and a soft-OILS run flown, with their results and report;
- a variation flown and its impact shown on the closures and KPIs;
- a flight parameter changed in a variation and flown in soft OILS;
- a flight image built and sealed;
- every editing workflow of W1 to W13 on the example group (a second writer stopped, a seal seen by a second
  person, a refused release replaced and marked, an objection answered, a breakdown that reproduces its node's
  cases and one that does not, a new group mounted, a design released with its flight images, a programme
  decision taken back, each way of bringing data in), ready for the ownership stage.

There is no trial in the middle: you first use the application as released.

## Stage 6 · The one release

### S17 · Release 2.0.0

1. **The whole chain from the drive:**
   - the design as the developer leaves it (today's design, built from the drive's latest corrected conversion);
   - its flight images on every target;
   - every case and campaign re-flown;
   - soft OILS on QEMU and Renode;
   - every closure answered, or named as open with its owner;
   - the health map;
   - the V&V report and the software verification report generated from it.
2. **The regression copy** of the released design into `tests/regression/`.
3. **Release notes**, for each role:
   - what 2.0.0 changes;
   - that the design is the developer's conversion and corrections, not yet signed by its owners (the ownership
     stage follows);
   - the corrections to 1.0.0's statements;
   - what is still open, by owner.
4. **"Ship 2.0.0".** You say it, and you push the tag `v2.0.0`; this session cannot push tags. The release
   workflow builds the kits (with the flight build), the wheel and the page. The developer packs them as **zip 2**
   (`apps/`, `CHECKSUMS`, `NOTES.md`, `guides/`; it never touches `groups/`, `cases/` or `design/`), you upload
   it, and START HERE goes to everyone.

**Done when** the release workflow is green, the application in `apps/` is the tagged one, and the design it was
proven on is in `tests/regression/`.

## Stage 7 · Ownership, after 2.0.0

The design was converted and corrected by the developer; here it passes to the people who own it, group by group,
release by release. Each phase ships as application releases (2.x) and released designs; none waits for another
2.0.0.

### S18 · The people and the drive (was S9)

- The application is already on the drive (zip 2, S17). Here the people arrive.
- START HERE, the sharing checklist, and one guide per role. These replace `design/manual/`'s author, lead and
  user guides.
- **Replaced, not kept beside:**
  - `docs/GROUP_APP.md`, `docs/NODE_APP.md`, `docs/MAIN_APP.md`, `docs/FILES_IN_THE_BROWSER.md`,
    `docs/DELIVERY.md`, `docs/CARRY_OVER.md`, `docs/DATABASE_FIRST_PLAN.md` and `docs/VISION_VS_CURRENT.md`;
  - the developer-loop skill, rewritten for W14 and W15.
- **Your part:**
  - name the system engineer, each subsystem engineer, each deputy and checker;
  - each makes their key;
  - your fingerprint goes in START HERE;
  - register the keys and share the folders.

**Done when** someone who has never seen the application can follow each role's guide, from an empty drive to a
released design.

### S19 · Switch-over (was S10)

1. **Parity once more,** on the drive's copy.
2. **First seals.** Each subsystem engineer seals their group's baseline as **1.0** with their key. Unsigned
   converted nodes and transcriptions are carried, marked unproven, and the seal records how many.
3. **First released design**, with a flight image for each target, released by the system engineer.
4. **`env` 1.1 round the whole cycle, W1 to W10, with no developer:**
   - **one design activity parameter,** `env_design_activity` (F10.7 and Ap design levels, with source), read by
     every density node. Today each scenario sets its own activity factor (`density_scale`);
   - **density as an open range** from solar minimum to solar maximum, each end stated from DTM2020. The tornado
     on `gd_1` (aerodynamic torque) and `gd_5` (secular momentum) shows it widest, and the closures that read them
     give range verdicts.

   It is signed, sealed, seen in today's design, answered by `act`, `pnt` and `design`, released (with new flight
   images, since the flight software reads the field and density models), reviewed and decided on.
5. From here on, design changes are made by people only.

**Done when** `env` 1.1 is in a released design, decided on by the programme, without the developer touching the
design.

### S20 · Each group takes its branch over

- Group by group, in the order you choose (`env` first, as S19's pilot), the subsystem engineer and node engineers
  go through their branch in the application: what each node says, where it came from, what it feeds.
- Every revision the developer made since S3 (the conversion's transcriptions, S7's methods, S11–S16's
  corrections) arrives in its node engineer's My work with its source, the test that failed before it and the
  comparison. Each is signed, or answered with a change or a question.
- The group seals its first release made by its people. The health map's *unproven* falls as signatures arrive.

**Done when** every group has sealed a release signed by its people, and no developer's revision is left
unanswered.

### S21 · Breakdown and editing

- Groups break their branches down further, as data: a node's children that reproduce its cases.
- Values, ranges, methods and cases are edited in the application, each change seen in today's design, answered
  by the groups it reaches, and released by the system engineer.
- New kinds of maths and features arrive as requests (W14) and application releases (W15).
- Board and lab campaigns, as the hardware arrives.

This phase does not end; it is how the design is worked from then on.

## Later still

- After the ownership stage:
  - variance-based sensitivity and the probability each closure holds;
  - an optimiser over the open ranges;
  - each closure's verification method;
  - table, time-series and uncertain-value ports;
  - SysML v2 and FMI 3.0 exchange.

## What needs your word

- **The decisions in S1:**
  1. one release, 2.0.0, at the end of S17;
  2. the model: one recursive block, layers as perspectives;
  3. the role names, deputies, and the checker kept where a group asks;
  4. the mount table, `programme` and `systems` as groups, `case` dissolved;
  5. `act` as one group with four mounts, or four groups;
  6. **the boundary:** the flight software's algorithms, parameters, modes and tables in the database; its
     runtime, the targets and the rigs in code; the maths library as the toolbox; and (7 Oct 2026) every published
     model with its data in the database, code only what enables a run;
  7. one library in Rust, native and WebAssembly; a C translator;
  8. the flight build inside the installed application, with its toolchains;
  9. every built-in relation transcribed before release, each signed by its node engineer in the ownership stage;
  10. baseline releases 0.1, and a first seal that carries unsigned nodes, marked;
  11. maturity seeding, and the margin policy (proposed: estimated 20 %, calculated 10 %, measured 3 %);
  12. the file names (operating model §10);
  13. `env` as the pilot;
  14. whether 2.0.0 waits for the first physical board run (H1), or ships with OILS and HILS proven on emulation
      (my proposal);
  15. **the re-plan of 6 October** (given): the database and the code in parallel until the release, the
      technical upgrade made by the developer in both, no trial and no people before 2.0.0, and the ownership
      stage (S18–S21) after it.
- **Each phase's merge.**
- **Zip 1's upload** (done), the corrected conversions' (zip 1.x), and zip 2's at the release.
- **The board and lab choices** (H1, H4), in S16.
- **"Ship 2.0.0",** and the tag.
- After the release: **the people**, **the switch-over date**, and **signing the developer's revisions** in S20,
  as the node, subsystem and system engineer (or the people you name).

The developer does not merge, release, change a rule or touch the design on its own initiative, and no assistant
supplies a relation.

## Where this plan breaks

- **One release is a long time without a tag.** 1.0.0 stays the released tool until S17. A defect found in 1.0.0
  meanwhile is fixed on `main`, not shipped as a 1.0.x, unless you ask.
- **2.0.0 ships a design nobody has signed.** Every node is the developer's conversion or correction, and the
  health map shows them *unproven* until S20. The release notes say so. Evidence at 2.0.0 is the developer's
  (tests, parity, re-flown cases); a programme decision taken on it before S20 is taken on unsigned data.
- **Nobody uses the application before it ships.** Without a trial, what CI cannot see (whether a screen is
  understood, whether a workflow fits how a group works) is found after the release and fixed in 2.x releases.
  The editing workflows are proven by CI on the example group only, and switched on group by group from S19.
- **Two copies, one truth.** The database is the source and the code carries what is generated from it; the
  checks that hold them together (`from_design --check`, `flight_build gen --check`, S7's) must run in CI on
  every commit, or the copies drift.
- **The parity gates are the hard part,** three times over: the design read from files (S4), the flight software
  generated (S6), and every relation generated (S7). Each stops the phases after it until it holds.
- **Generated flight code** must meet what the hand-written code met: no allocation, the deadlines in soft OILS,
  MISRA in S15. The translator is held to the conventions of `fsw/pseudocode/00_conventions.md`, and the timing is
  re-measured.
- **The flight build carries compilers.** The installed application grows by its toolchains. A computer without
  them can fly SILS through the interpreter, but cannot build an image.
- **S20 puts a lot of signing on few people.** Hundreds of the developer's revisions will arrive. Until more
  people are named, you sign them. Each arrives with its source, its test that failed before, and the comparison, so checking
  is reading, not re-deriving.
- **The published models are large.** DTM2020, JB2008, IAU 2006/2000A and DE440 are thousands of lines and long
  coefficient tables; written as methods they must stay bit-identical to today's ports, and generated code must
  not slow the engine down past its real-time use. The DE440 data the runs use (its span, its bodies) is kept as a
  table, not the whole file.
- **Transcriptions are only as good as the code they came from.** Proving a method equal to the code proves the
  transcription, not the physics. The physics is proven by the sourced vectors of S12 and the independent
  referents of S13.
- **Today's design depends on the drive being in step.** Two computers must build the same one (S4), and the
  application always says which releases it used.
- **The page cannot run everything.** Campaigns, soft OILS and the flight build need the installed application.

## Where the database-first plan and the roadmap went

| was | now |
|---|---|
| D1 Library | S3, step 5 |
| D2 Build | S4 |
| D3 Nodes drive numbers | S3 and S4 |
| D4 One reader | S4 |
| D5 Sync | not needed: the application works on the drive's folder itself |
| D6 Pack and upload | zip 1 (S4), zip 1.x (S7, S11–S16), zip 2 (S17) |
| D7 End to end | the parity gates (S4, S6, S7) and S17 |
| D8 Release | S17 |
| D9 Generated code in the engine | S7 |
| D10 Main app over the database | S8 |
| D11 Outputs into the database | S4 and S8 |
| D12 Twin from the database | S4 (export) and S7 (generated functions) |
| roadmap U0 (was v1.1) | S11 |
| roadmap U1 (was v1.2) | S12 |
| roadmap U2 (was v1.3) | S13 |
| roadmap U3 (was v1.4) | S14 |
| roadmap U4 (was v1.5) | S15 |
| roadmap U5 (was v2.0) | S16, and board campaigns after |

## Progress

| phase | state | evidence |
|---|---|---|
| S0 Safe ground | built, waiting for your approval of the notes text | `tests/test_safe_ground.py`; `docs/RELEASE_NOTES_1_0_0_CORRECTION.md` |
| S1 Rules, roles, the model | written, waiting for your approval (with a second reviewer) | `docs/SYSTEM_MODEL.md`, `docs/OPERATING_2_0.md`, `docs/CODE_ARCHITECTURE.md`, `design/rules_2_0.toml` (R01–R16), `design/tree_2_0.toml` (21 groups and their mounts), `docs/GLOSSARY.md` (the words of 2.0.0), `CONTRIBUTING.md` (toward 2.0.0) |
| S2 Files, versions and keys | **done**. Format 2 and its lossless upgrade (all 765 nodes and 20 groups, Python and Rust); the library's writer, content hash, keys, signature chain and comparison (Rust and page, cross-checked); `group.py check`/`verify` and `release.py check` in the library, word for word (40 releases, 8 broken, 8 damaged folders); the pseudocode interpreter in Rust (`trinetra-pcode`: 49,085 vector values, 49,074 bit for bit, 11 within 1.5e-15 in `sin`/`cos` by decision; 1,240 broken sources answered alike; builds for WebAssembly); the node app's live checks in the library (all 765 nodes and 547 broken copies, 10,838 problems, identical) | `tests/test_format2.py`, `tests/test_release.py`, `engine/crates/trinetra-design` (`tndb`; `keys_cross`, `content_cross`, `node_rules`), `engine/crates/trinetra-pcode` |
| S3 The design leaves the repository | **done** (archiving moved to the end of S4, see S3 step 8). `tools/convert_2_0.py` writes 21 groups and 1,116 nodes (1,155 after S4 held the rest of 1.0.0's plan): the 765 of 1.0.0 field by field, 119 tree blocks, 7 flight-algorithm modules with their 2,616 vectors as cases, 147 flight parameters, the IGRF table, 76 library files (97, then 115) and the delivery waves; 75 case files (case lines wired to their nodes); a baseline release 0.1 per group; readable copies listing every built-in relation and everything a person must still state. Nothing dropped; every file and release checks in Python and in the library; two runs give the same bytes | `tests/test_convert.py`; `readable/Conversion.csv`, `BuiltIn.csv` |
| S4 The design runs → zip 1 | **done** (6 Oct 2026). Zip 1 is on the drive: every one of its 1,298 files is there with the manifest's name and size, none converted, none extra (read through the Drive connector, `tools/drive.py listing`); `START HERE.txt` checked byte for byte by MD5; the packed folder builds today's design byte for byte equal to the regression copy the gate passed on. Today's design built from the drive (`tools/design_build.py`), the same bytes on any copy of the drive; the parity gate holds on the regression copy (`results/PARITY_2_0.md`): the engine reading the design alone gives 166 of 166 inputs, the parameter table and its C and Rust, 48 of 48 blobs, 48 of 48 stored runs, 4,516 of 4,516 stored campaign runs, 1,500 of 1,500 evaluated rows and closures, soft OILS on QEMU and the MATLAB twin from its export; both cases' design loops and campaigns flown from it change none of 1,023 numbers (`results/END_TO_END.md`); the health map, range verdicts, tornadoes and trace to cause (`results/HEALTH.md`: ais_3u's APE closes for only part of its range); every undeclared cycle refused, the design loop declared; the regression copy and the files generated from it (`tools/from_design.py`); 1.0.0's sources archived (`archive/design-1.0/`); zip 1 packed and checked (`tools/drive.py`). | `tests/test_design_build.py`, `tests/test_from_design.py`, `tests/test_drive.py`, `results/PARITY_2_0.md`, `results/END_TO_END.md`, `results/HEALTH.md` |
| S5 Translators complete | **done** (6 Oct 2026). A C translator (`design/js/pcode_c.js`: C99, no dynamic memory or recursion, the flight software's flags with every warning an error); the Rust and MATLAB translators completed for what the flight software uses (an array of whole numbers where reals are wanted, records through the dispatcher, MATLAB's conditional evaluating only the branch taken); every package (the relations, the language's self-test, the flight software, the group code) translated to Rust, C and MATLAB, built and run on every interpreter vector: 66,421 values each, exact functions bit for bit, the rest within 6.6e-15 (`tools/translators.py`, `results/TRANSLATORS.md`, in `check_all`); the three translators in the library (`trinetra-pcode` `gen`, `tndb translate`), byte for byte the JavaScript's on every package. Known, not yet needed by any node: a proc whose state is a record cannot be translated to Rust or MATLAB, and MATLAB cannot index a constant array written inline; both are fixed when a node first needs them (S6) | `tests/test_translators_lib.py`, `results/TRANSLATORS.md` |
| S6 Flight software from the database | **done** (7 Oct 2026). The flight build (`tools/flight_build.py`) writes the C and Rust algorithms from the design (`fsw/alg`, `fsw-rs/src/alg`) and an algorithms ID that both build ids carry (`... alg d0b5944afc9464e4`), so every run names the algorithms it flew; the runtime's 68 algorithm functions only hand values to the generated ones; all 78 functions held to the interpreter's vectors (45,657 values, worst 6.6e-15). Parity: 48 of 48 scenarios identical, C equal to Rust bit for bit; every campaign (4,516 runs, C and Rust) identical in every value and verdict; soft OILS on QEMU with no overrun in 96 runs. **Accepted by the owner:** the generated code is slower (instructions per step C +9 % on average, up to +20 %, Rust up to +26 % on one step; smallest worst-case margin 35.76 ms C, 33.21 ms Rust, of 50; stack 12,208 of 32,768 bytes), because the translator writes `int` as 64-bit and passes arrays by value; `sun_spin_ais`'s soft-OILS verdicts moved with the latency (its SILS is identical). Sealed flight images per target (posix, posix-rs, qemu, qemu-rs; `seal`, `verify`, `which`; unsigned, the developer's). The done-when test: an algorithm change in a copied design changes only its sources, the ID and its vectors; a flight-software parameter change changes only its blob value and the run's metrics, with no rebuild. **Found, for the owning groups:** the design's `drv_read` zeroes a missing reading while the runtime holds the last one (the runtime's hold kept); the design's NaN edge branches now apply; the Rust capture now divides as C and the design do; the `fsw_param_*` nodes hold the blob's layout but no values: the engine still derives them from the case and scenario until S7. Not yet: a manifest field with the image's hash (runs map to their image by build id). Unused `adcs_igrf13.h` removed. | `results/FLIGHT_PARITY.md`, `results/FLIGHT_IMAGES.md`, `tests/test_flight_build.py`, `tools/flight_build.py gen --check` |
| Re-plan (6 Oct 2026) | given by the owner: the database and the code in parallel until the release; the upgrade S11–S16 before the application S8; no trial and no people before 2.0.0; the ownership stage S18–S21 after it ("The re-plan of 6 October") | this page; `CONTRIBUTING.md` |
| S7 Every relation from the database | in progress. Step 1, the inventory, done: 271 built-in nodes, of which 91 are relations computed in code and 180 are kinds the conversion marked built-in (achieved holders, closures, owner pointers); 147 flight parameter nodes with no value, computed in `adcs-sim/src/config.rs`; 32 engine relation items, 9 orbit force modules, 4 in Python, 90 hand-written MATLAB files; 13 translator gaps; 19 steps proposed. Decisions asked: the device byte codecs, the toolbox line for published orbit models, `floquet.py` and the design-loop rules | `docs/S7_INVENTORY.md`, `results/s7_inventory.json` |
