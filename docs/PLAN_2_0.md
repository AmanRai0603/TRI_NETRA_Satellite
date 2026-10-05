# The plan to 2.0.0

> **Answer first.** One release, 2.0.0, built in eight phases (S0–S7) and switched over on one planned day.
>
> After it:
> - everyone has one application and one shared drive, **Trinetra Database**;
> - each person looks after their own part of the design database every day;
> - everyone sees today's design built from everyone's latest work;
> - the system engineer releases the design that decisions are made on;
> - the developer maintains only the code.
>
> The database comes first. At the end of S4 you get **zip 1**: the whole design converted and proven to give
> today's numbers. You upload it by hand. Each phase proves what it builds in CI and merges into `main` on your
> word. Nothing changes for anyone until the switch-over.
>
> **Kind:** explanation + plan · **For:** everyone · **Status:** proposed 5 Oct 2026, waiting for your decisions
> (§ "What needs your word")

What 2.0.0 is, is in two documents:
- **the design model**, `docs/SYSTEM_MODEL.md`;
- **how it is operated**, `docs/OPERATING_2_0.md`: the valves and the daily rhythm, the roles, the application
  and its workspaces, today's design, the health map, how data comes in, every file, version and folder, each
  step from W1 to W16, and who hears what.

This page is only how we get there. It replaces `docs/DATABASE_FIRST_PLAN.md` (D1–D12 are placed in the last
section) and renumbers the technical roadmap. The phase ids are S0–S7 so they do not collide with the roadmap's
(E, M, G, F, A, Y, T, P, O, TS, DD, DB, D, U, B).

## Why 2.0.0, and why one release

TRI-NETRA has shipped 1.0.0. What changes now is larger than a minor version:
- the file formats (format 2);
- the rules;
- the application (three pages and a desktop app become one);
- the roles;
- where the design lives (out of the repository, onto the drive).

So the next release is **2.0.0**, and it ships once. After it:

| what | made by | when |
|---|---|---|
| an application release, the code | the developer | rarely |
| today's design | the application, from the drive, for everyone | whenever anyone opens it |
| a released design | the system engineer, and no one else | when today's design is right |
| a programme decision | the programme manager, as a release of the programme's branch | after a released design |

## The one idea every phase serves

**The design never passes through the code again.** The developer moves it out of the repository once, at S3. From
then on it is written, checked, combined, run, signed and released in the application, by the people who own it.

Everything the developer does today on a group's release becomes checks in one library, which the application
runs at every valve. Today that work is:
- verify and merge (`tools/group.py`);
- generate (`tools/groupcode.py`);
- deliver a test app, and accept it (`tools/delivery.py`);
- build `design.tndb` (`tools/seed_design.py`, `tools/drive_pack.py`).

## The code, as 2.0.0 arranges it

| part | is | crate or folder | runs |
|---|---|---|---|
| **kernel** | the physics generated from the pseudocode; the time engine; the orbit propagator; the sizing; the plotting | `adcs-physics`, `adcs-sim-core`, `adcs-sim`, `adcs-pop`, `adcs-design`, `adcs-plot` | installed; `adcs-sim-core` (no_std) also as WebAssembly in the page |
| **flight software** | C and Rust, one interface | `fsw/`, `fsw-rs/`, `adcs-fsw-abi` | in the engine, the emulated OBC (soft OILS), the boards |
| **the library** | reads, writes and checks every file kind; the pseudocode interpreter; the design-graph engine; today's design; the health map; signing | `trinetra-design`, grown from today's read-only reader | installed (native) and in the page (WebAssembly) |
| **the application** | five workspaces over the library and the engine | `trinetra-app` (installed) and `apps/TRI-NETRA.html` (the page) | everyone |
| **CLI, Python package, tools** | scripted use, CI, the release | `adcs-cli`, `python/trinetra_adcs`, `tools/` | the developer and CI; they read the design through the library |
| **twin** | the MATLAB reference | `matlab_sils/` | flies an export generated from the design |

**One library, two hosts.**
- The library's core is plain Rust over rows: the checks, the interpreter, the graph, the health map and the
  signature chain.
- Each host does its own file I/O. The installed application and the CLI use SQLite (`rusqlite`, as
  `trinetra-design` does today). The page uses the vendored `sql.js` (`design/vendor/sqljs`), as the three pages
  do today.

So the same checks run wherever a file is opened. The JavaScript and Python implementations that exist today
(`design/js/pcode.js`, `node_model.js`, `structure.js`, `release.js`; `tools/pcode.py`, `tools/tndb.py`,
`tools/group.py`) become its test oracles until the switch-over, then retire.

## While it is built

- **Nothing changes for anyone until the switch-over.** 1.0.0 stays the released tool: the kits, the wheel, the
  desktop app, the three pages and their manuals (`docs/GROUP_APP.md`, `docs/NODE_APP.md`). Nobody is writing
  design files on the drive yet: it is empty apart from the guides and two sheets. So nobody's work has to be
  frozen or upgraded.
- **The repository's design data is frozen at S3.** After the conversion, any change to it must be a corrected
  conversion (zip 1.1, below), never an edit beside it. The 1.0.0 content is kept read-only under
  `archive/design-1.0/` until 2.0.0 ships, then deleted.
- **The new application is built beside the old, not patched into it.** The old pages, the delivery tool and the
  test apps retire on the switch-over day, so nobody has two of anything to choose between.
- **You try it first.** At the end of S5 you get the application with the converted design. You live with it for
  some days in every role (programme manager, system engineer, the subsystem engineer of `env`, a node engineer),
  round the whole cycle, W1 to W10. S6 does not start until that round is answered.

## The phases

| phase | needs | delivers | who |
|---|---|---|---|
| **S0 · Safe ground** | nothing | the application and the engine refuse a design they cannot run; results name their design; the 1.0.0 notes corrected | developer; you approve the notes |
| **S1 · Rules, roles and the model** | S0 can run beside it | the system model, the operating model and the code's architecture for TRI-NETRA; the code's rules, the design's rules, the valves and the role names; the mount table | developer, **you** |
| **S2 · Files, versions and keys** | S1 | format 2 for every file kind; versions; signatures; the one library with every check | developer |
| **S3 · The design leaves the repository** | S2 | the 765 nodes, the catalogue and the cases converted into the drive's layout; programme, systems and the groups as files; built-in relations kept as code by node id | developer |
| **S4 · The engine runs the design → zip 1** | S3 | the design graph and the time engine read the converted files; today's design; the health map; the declared loop; **the parity gate**; **zip 1**, uploaded by you and checked on the drive | developer, **you upload** |
| **S5 · The application** | S2, then S4 | one application, installed and as a page, with its five workspaces, doing W1 to W16 | developer, **you try it** |
| **S6 · The drive and the people → zip 2** | S4, S5 | the application and the guides on the drive (zip 2); keys; sharing; one guide per role; START HERE | developer, **programme manager** |
| **S7 · Switch-over and proof** | S6 | the first released design; `env` round the whole cycle with no developer; then **2.0.0** | **you** |

### S0 · Safe ground

- A design names the application version and toolbox it needs. The engine (`adcs-cli`), the desktop app and the
  Python package check it, and refuse a mismatch by name.
- Every result carries the fingerprint of the design and the version of the engine that made it, and is marked
  stale when either changes. This extends the run manifest's provenance (`docs/RESULTS.md`).
- The faults the data-flow audit found in reading a design are fixed, each with a test that failed before:
  - `adcs size` fails when the design comes from a database;
  - DE440 is not found from an empty working directory;
  - `evaluate` crashes on the pack's root folder;
  - a folder or an unreadable database is not refused clearly.
- The pseudocode checker page (`design/pcode_checker.html`) is rebuilt whenever the interpreter it embeds changes;
  a check in `tools/check_all.py` holds it.
- The published 1.0.0 release notes are corrected (`docs/TECHNICAL_ROADMAP.md` §0):
  - the KPI passes were not demonstrated (A1);
  - power is actuator power only (Y1);
  - the CMG cluster is counted four times (Y2).

  You approve the text and edit the release on GitHub.

**Done when** a design built for another engine is refused in a test, and every result names its design.

### S1 · Rules, roles and the model

- **The model:** `docs/SYSTEM_MODEL.md` and `docs/OPERATING_2_0.md` (proposed with this plan), reviewed and
  approved.
- **The code's architecture:** `docs/CODE_ARCHITECTURE.md`, the table above written out: what each crate and
  folder does, and which part may call which.
- **The code's rules:** `CONTRIBUTING.md`, `docs/CHANGING.md` and the agents' files in `.claude/agents/`,
  rewritten for a repository that holds only code. These stay:
  - refuse, never guess (a missing input is refused by name, never filled);
  - never edit a generated file;
  - the twin moves with the engine, and the C flight software with the Rust, in the same change;
  - every formula in the kernel is traced to its source (`docs/references.toml`).

  This is new: an application release must give the current released design's answers unchanged (W15).
- **The design's rules,** written as checks for the library (S2) and described for people in the operating model:
  - one writer per file, and the signature chain;
  - an expected value never comes from the code under test;
  - no relation supplied by an assistant, and a transcription signed by the person who checked it;
  - every requirement says which way it binds;
  - a node reads its children only through their ports;
  - a release is never edited;
  - a parameter is changed only by the level that owns it;
  - every valve's owner controls what passes it;
  - the design is released by the system engineer alone, and today's design is never taken for a released one;
  - evidence counts only from results made by the design it is judged in.
- **The roles and their names:** programme manager, system engineer, subsystem engineer, node engineer,
  developer, each with a deputy, plus the checker's signature where a group asks for it. Every owner of a branch
  is the system engineer of that branch. Every page, screen and guide uses these names and no other
  (`docs/GLOSSARY.md`, `design/manual/05_glossary.md`).
- **The tree:** the mount of every group, the dissolving of `case`, and `programme` and `systems` as groups
  (`docs/SYSTEM_MODEL.md` §8).
- **Replaced, not kept beside:**
  - `docs/RELEASE_PLAN.md` §1 is replaced by the operating model;
  - `docs/RULES_PROPOSAL.md` is replaced by the design's rules.

  Both are marked as describing 1.0.0.

**Done when** you approve, with a second reviewer.

### S2 · Files, versions and keys

- **One schema, format 2** (`design/schema.toml`), for every file kind in the operating model §10:
  - a **block** whose parent goes to any depth, with its perspective, behaviour, contract version and owner;
  - a **port**, with type, unit, range (each end with its reason), state, maturity and direction; while open,
    its owner and the gate it is due by; and its bundle;
  - **wire, mount, closure** (by analysis or by evidence, with the metric and campaign), and **loop** (what
    settles, tolerance, iteration limit);
  - **case** and variation, **text, table, media**;
  - **signature, change, request, issue, preview, answer**, and the integration record.
- **Port types:** number with unit, whole number, choice, yes/no, list and parameter first. Table, time series and
  uncertain value come later.
- **Identity and versions** of every file, by the operating model §13.
- **Keys:**
  - make a key, lock it by passphrase, sign, and check a signature through the chain (operating model §14);
  - Ed25519 with a PBKDF2 and AES-GCM lock, the same algorithms in Web Crypto and in Rust.
- **One library**, `trinetra-design`, grown from today's read-only reader. It reads, writes and checks every kind,
  compiled for the installed application and, as WebAssembly, for the page.
  - Every check that today's tools and pages run moves into it:
    - `tools/tndb.py check`;
    - `tools/group.py verify`;
    - `tools/delivery.py`;
    - the node app's live checks (the spec's intake rules);
    - the group app's impact checks;
    - the seal.
  - What is checked: the signatures, the method and its units, the cases, both ends of every range, the
    de-risking record, and the assistant rules.
  - **The pseudocode interpreter in Rust**, held equal to `design/js/pcode.js` and `tools/pcode.py` on every test
    vector, and to the generated `adcs-physics` (which already equals the interpreter).
- **Upgrade:** a format-1 file (1.0.0's) upgrades when opened, and keeps a copy. **Nothing a group wrote is
  dropped.**
- **Comparison:** any two revisions, releases or designs, node by node.

**Done when:**
- the 765 node files and 20 group files of 1.0.0 upgrade with nothing dropped, field by field;
- signatures check through the chain;
- the library refuses everything today's tools refuse, in a test for each.

### S3 · The design leaves the repository

This is the last time the design passes through the code.

1. **The tree becomes files.** `spec/` (the tree, the seed content, the physics pseudocode, the KPIs) and
   `design/groups.toml` and `design/carry.toml` become:
   - the programme's branch (layer 1), the systems branch (layer 2), and one branch per group (21 groups:
     today's 20, less `case`, plus `programme` and `systems`);
   - each row's layer becomes its perspective tag;
   - each group mounts on its block, its mount drawn from today's interface rows (`docs/SYSTEM_MODEL.md` §8);
   - today's 238 derivation and 23 contribution edges become wires, and the 38 closures sit on the blocks they
     close;
   - the proposed next level is added as open blocks;
   - today's parameters are placed at the level that owns them.
2. **Every value gets its state, maturity and range**, by rules that invent nothing:
   - stated with a source → decided;
   - a required row's bound → allocated;
   - computed → achieved;
   - anything else → open;
   - maturity is estimated unless the source says calculated or measured;
   - a range comes from the 82 nodes that carry `lower` and `upper` with their reasons; any other open value is
     listed without one.
3. **The library data becomes design:**
   - `catalogue/` (parts, products, algorithms, components, modes, families, classes): lookup and stated blocks of
     `catalogue`, `act` and `sens`;
   - dispersions, KPI definitions and delivery waves: blocks of `vv`, `kpi` and `programme`;
   - the reference cases, the 48 scenarios, the 8 campaigns and the 12 trades: case files in `cases/`;
   - the case lines that have a node (162 of 213) are wired to it, so the time engine flies the node's value;
     the rest stay case values, listed.
4. **The relations still in code** are gathered under one registry, keyed by node id: `adcs-physics` (generated
   from the pseudocode), `adcs-sim-core` and `adcs-design`. Each such node's behaviour is **built-in**, marked.
5. **Baseline releases.** Each group's converted content is sealed as release **0.1** by the conversion. It is
   marked *converted, not yet signed by a person*, and every node in it shows **unproven** until its engineer
   signs it.
6. **Readable copies:**
   - `readable/*.csv`;
   - `readable/Conversion.csv`, which places every field of every 1.0.0 node, every catalogue entry and every case
     line, or lists it as dropped. It must list none.
7. **The repository** keeps the example group and the converted design's copy as the regression for W15
   (`tests/fixtures/`, `tests/regression/`). The node pages, seed and carry-over tools, `export_catalogue.py` and
   the generators that read `spec/` leave the build, and their checks now run in the library.

**Done when:**
- the conversion report lists nothing as dropped;
- every file passes the library's checks;
- the repository builds and tests with no design data in it apart from the example group and the regression copy.

### S4 · The engine runs the design → zip 1

- **The design graph is built from the files** when the application opens, not compiled into it, installed and
  in the page. It runs every behaviour:
  - method, run by the interpreter;
  - children;
  - stated;
  - lookup;
  - open, refused by name;
  - built-in, found by node id.
- **The time engine reads the design.**
  - Every case input comes from its node (a mapped node with no value is refused by name), and every catalogue
    input from its lookup block.
  - Each run's manifest lists every input's node and revision.
  - Its metrics return as evidence to the `closure_verified` blocks.
  - **One reader for every tool:** the design loop, campaigns, evaluate, trace, the pointing budget, the V&V
    report, the Python package and the CLI.
  - The MATLAB twin flies an export generated from the design.
- **Today's design** is built from the drive's folder on opening:
  - from every group's latest sealed release that passes its checks;
  - a refused release is replaced by its group's last good one, and marked;
  - the application says which releases it used.

  The same holds at every valve: a group's today is built from its nodes' latest signed revisions.
- **The declared loop:** the design loop's sizing → mass and inertia → demand cycle is declared on the `design`
  block and iterated there, with its tolerance. Any other cycle is refused by name.
- **Every value** carries its state, maturity and range. **Every closure** gives its range verdict and tornado.
- **Every node** gets its health-map state, with the roll-up valve by valve and the trace to cause (operating
  model §6).
- A design names the application version and toolbox it needs.

**The parity gate.** For every row in every case, the new path gives today's answer within the row's own
tolerance, and refuses where today refuses. There is no switch without it. The gate covers:
- every row and closure of `results/EVALUATION.md` and `results/END_TO_END.md`;
- the node verifier (`tools/verify_nodes.py`): every node passes;
- every committed run re-flown from the design: both cases, every scenario, every campaign, with the same
  metrics;
- engine-to-twin parity (`results/ENGINE_PARITY.md`), unchanged, with the twin on the export.

**Zip 1** is packed once the gate holds (`tools/drive.py`, grown from `drive_pack.py`). It holds the drive's
layout (operating model §15) without `apps/`:
- `groups/` with every group file, node file and baseline release;
- `cases/`;
- `readable/`;
- the empty folders `design/`, `daily/`, `integration/`, `issues/` and `results/`;
- `guides/`: the model, the operating model, this plan and the engineering documents;
- START HERE, saying what works yet and what does not;
- `MANIFEST.json`, with each file's size, SHA-256 and MD5.

**Your upload:**
1. In Trinetra Database, delete `Apps/` and `Design/`.
2. Move `Guides/` and the two sheets into `old-1.0/`. The check ignores that folder, and you delete it after a
   month.
3. Upload zip 1's content, with Google's "convert uploads" off.
4. Then either you run `python3 tools/drive.py --verify "<folder>" --first-upload` on Drive for desktop, or I check
   it through the Drive connector, read-only. I compare every file's name, size and MD5 with the manifest; Drive
   reports an MD5 for every uploaded file.
5. Then I run the parity gate once more on a copy of what is on the drive.

**Until the first released design (S7) I may send a corrected zip 1.1,** replacing `groups/` and `cases/` only,
because nobody has written on the drive yet. After S7, never.

**Done when:**
- parity holds for the whole design, installed and in the page;
- today's design is built identically on two computers from the same files;
- a deliberately broken node is traced to by name from the KPI it breaks;
- the uploaded drive passes the check.

### S5 · The application

One application, installed and as a page from the drive, with the same screens. It knows each person by their key,
and opens on My work.

- **My work:** everything that needs the person now, at their valve.
- **Node workspace:** W3, and the node engineer's side of W4 and W13. *Fly it* runs a case with the node's value
  inside today's design.
- **System workspace:**
  - one workspace at every valve: W1 and W8 for the system engineer; W2, W4, W5, W7, W12 and W13 for a subsystem
    engineer;
  - today's view of the branch, impact, the sheet, the N2 at any depth, people, comparisons;
  - for the system engineer: the ADCS budgets across releases, the declared loop, previews and requests.
- **Programme workspace:** W10; mission health per case, gates (the order lifecycle and the review gates), risks
  and beliefs, the organisation, measures across releases.
- **Explore workspace:**
  - today's design and any released one, side by side;
  - runs, variations, sweeps, campaigns and, installed, soft OILS;
  - the health map with trace to cause;
  - each run's figures and report;
  - W6's daily discussion on one screen.
- **Bringing data in,** every way in the operating model §7:
  - a formula typed as written;
  - a table pasted from any spreadsheet or datasheet;
  - a worked example as a case;
  - CSV results from MATLAB, Python or a spreadsheet;
  - units written naturally, and a range or a spread as typed;
  - a PDF beside the node;
  - *start from a similar node*;
  - a customer's CSV as a new case.
- **Made easy to use** by the eight points of the operating model §8.
- **What it replaces:** today's Files, Node and Group pages, the desktop app's three tabs, the test-app template,
  node-form intake, `tools/delivery.py`, and `tools/group.py merge`. They are removed at the switch-over.

**Done when** CI drives every workflow on the example group, installed and as a page, with no developer step in W1
to W13. That includes:
- a second writer stopped in W3;
- a sealed release appearing in today's design for a second person;
- a refused release replaced by its last good one, and marked;
- an objection answered in W7;
- a failing closure traced to its node, raised as an issue, and closed by the release that fixes it;
- a node broken down whose children reproduce its cases, and one whose children do not;
- a new group mounted on a node, its owner becoming the valve above it;
- a design released, reviewed at each level, and a programme decision taken back into today's design;
- a node's value flown, and an evidence closure answered from that flight;
- each way of bringing data in.

**Your trial** follows: some days in every role, round W1 to W10, on the converted design.

### S6 · The drive and the people → zip 2

- `tools/drive.py` packs only what the developer owns:
  - `apps/` with the application installed for each OS and as a page, `CHECKSUMS` and `NOTES.md`;
  - `guides/`.

  **Zip 2** adds these to the drive and never touches `groups/`, `cases/` or `design/`. Everything else is
  written by the application.
- **START HERE**, with the daily rhythm and the programme manager's key fingerprint.
- **The sharing table** (operating model §15), as a checklist for the programme manager.
- **One guide per role:** node engineer, subsystem engineer, system engineer, programme manager, developer. Each is
  written from the operating model, with the role's day and its workflows step by step, and replaces
  `design/manual/`'s author, lead and user guides.
- **Replaced, not kept beside:**
  - `docs/GROUP_APP.md`, `docs/NODE_APP.md`, `docs/MAIN_APP.md`, `docs/FILES_IN_THE_BROWSER.md`,
    `docs/DELIVERY.md`, `docs/CARRY_OVER.md`, `docs/DATABASE_FIRST_PLAN.md` and `docs/VISION_VS_CURRENT.md`;
  - the developer-loop skill (`.claude/skills/trinetra-coordinator`), rewritten for W14 and W15.

**Done when** someone who has never seen the application can follow each role's guide, from an empty drive to a
released design.

### S7 · Switch-over and proof

One planned day, then some days of real use. Nothing is overwritten.

1. **Freeze.** Nothing to freeze on the drive, since nobody has written on it yet. The repository's design data
   has been frozen since S3. The date is told to everyone who will hold a role.
2. **Parity once more,** on the drive's copy, on the day.
3. **The drive.** Trinetra Database holds zip 1 and zip 2. `old-1.0/` is kept read-only for a month.
4. **Share and register.**
   - The programme manager shares each folder by the table, writes their key fingerprint into START HERE, and
     registers the system engineer's, each subsystem engineer's and each deputy's key.
   - Each subsystem engineer registers their node engineers' and checkers' keys.
5. **First release.**
   - Each subsystem engineer seals their group's baseline as **1.0** with their own key. Its unsigned converted
     nodes are carried and marked unproven, and the seal records how many.
   - The system engineer releases the first design, **2026.mm.1**.
   - The developer puts the 2.0.0 release candidate into `apps/`.
6. **Live with it, with no developer in the loop.** Every day, today's design is opened and looked at. **`env` 1.1**
   goes round the whole cycle, W1 to W10:
   - **One design activity parameter.** Today the atmospheric density (`m3_3`, `l3_dist_row_07`) is a static
     table times an activity factor set separately in each scenario (`density_scale`). It becomes one stated
     parameter that every density node reads: `env_design_activity`, the F10.7 and Ap design levels, with their
     source.
   - **Density as an open range.** It runs from solar minimum to solar maximum, each end stated with its source
     (DTM2020 at those levels). The tornado on the aerodynamic torque (`gd_1`) and the secular momentum (`gd_5`)
     shows it as the widest bar, and the closures that read them give their range verdicts.

   It is signed, sealed, seen in today's design, answered by the groups it moves (`act`, `pnt`, `design`),
   released, reviewed at each level, and decided on.
7. **Ship.** You say **"Ship 2.0.0"**. You push the tag `v2.0.0`; this session cannot push tags. The release
   workflow builds the kits, the wheel and the page. START HERE goes to every subsystem engineer.

**Done when** `env` 1.1 is in a released design, decided on by the programme, without the developer touching it.

## After 2.0.0

- **Built-in to method.** Each group writes methods for its built-in relations, in the Node workspace.
  - The application runs both the method and the built-in on the node's cases and across its range, and shows any
    difference.
  - Once a method is released, the developer deletes the built-in in a later application release.
  - About 160 nodes start as built-in. 166 cite Rust code today; 48 already have pseudocode.
- **Generated code in the time engine.** The flight algorithms run from their nodes' pseudocode
  (`adcs-groups`), one group at a time, each reproducing the hand-written model on every scenario first.
- **Groups break their branches down further,** as data, with no developer.
- **New kinds of maths and features** arrive as requests (W14) and application releases. DTM2020 as a toolbox
  function the methods can call is the first.
- **The technical roadmap resumes,** renumbered:

  | release | phase | content |
  |---|---|---|
  | v2.1 | U0 | correct 1.0.0's wrong results and the critical flight-software and GNC defects |
  | v2.2 | U1 | environment, frames and requirements you can cite; orbit-propagator fixes |
  | v2.3 | U2 | stability margins, statistics, budgets, independent referents |
  | v2.4 | U3 | devices, GNC, system completeness, FMECA |
  | v2.5 | U4 | flight-software assurance, TM/TC, Renode soft OILS |
  | v3.0 | U5 | board OILS and HILS (hardware-paced) |

  Every design-data change in them is made in the application by its owner; every code change is made in the
  repository.
- **Later:**
  - variance-based sensitivity and the probability each closure holds;
  - an optimiser over the open ranges;
  - each closure's verification method;
  - nodes that keep state from one time step to the next;
  - table, time-series and uncertain-value ports;
  - SysML v2 and FMI 3.0 exchange.

## What needs your word

- **The decisions in S1:**
  1. version 2.0.0;
  2. the model: one recursive block, layers as perspectives;
  3. the role names, deputies, and the checker's signature kept where a group asks for it;
  4. the mount table, `programme` and `systems` as groups, `case` dissolved;
  5. `act` as one group with four mounts, or four groups;
  6. one library in Rust, native and WebAssembly, with today's JavaScript and Python as its test oracles;
  7. the time engine's algorithms built-in in 2.0.0;
  8. baseline releases 0.1, and a first seal that carries unsigned converted nodes, marked;
  9. maturity seeding, and the margin policy (proposed: estimated 20 %, calculated 10 %, measured 3 %);
  10. the file names (operating model §10);
  11. `env` as the pilot, with its two changes.
- **Each phase's merge.**
- **Zip 1's upload, and zip 2's.**
- **The trial** at the end of S5.
- **The people:** who holds each role, and each deputy.
- **The switch-over date.**
- **"Ship 2.0.0",** and the tag.

The developer does not merge, release, change a rule or touch the design on its own initiative, and no assistant
supplies a relation.

## Where this plan breaks

- **The parity gate is the hard part.** If the new path cannot reproduce today's numbers, S4 stops until it does,
  and zip 1 and everything after it wait.
- **The interpreter in Rust is new code.** It is held to the two interpreters that exist on every vector, and to
  the generated physics. A disagreement stops S2.
- **The application carries everything now.** With no developer in the loop, every check the developer made by
  hand must be in the library, and the application must be easy enough that nobody needs one. S2 and S5 are the
  largest phases, and your trial is the test.
- **Today's design depends on the drive being in step.** S4 proves two computers build the same one, and the
  application always says which releases it used.
- **Two engines.** The design graph and the time engine meet through mapped inputs and evidence. 51 of 213 case
  lines have no node, and stay case values until a group claims them.
- **People learning a new application, roles, keys and a daily rhythm.** S7 tries it in real use, on one group,
  before anyone else depends on it.
- **Built-in relations are a debt.** On 2.0.0 most computing nodes still compute in code. They are marked, listed
  by owner, and replaced by methods after, not hidden.
- **The page cannot run everything.** Monte Carlo campaigns and soft OILS need the installed application, and the
  page says so.

## Where the database-first plan went

| `docs/DATABASE_FIRST_PLAN.md` | now |
|---|---|
| D1 Library | S3, step 3: the library data becomes blocks and case files |
| D2 Build | S4: today's design and the released design replace the single `design.tndb` |
| D3 Nodes drive numbers | S3 step 3 (case lines wired to nodes) and S4 |
| D4 One reader | S4 |
| D5 Sync | not needed: the application works on the drive's folder itself; the repository keeps the regression copy |
| D6 Pack and upload | zip 1 (S4) and zip 2 (S6) |
| D7 End to end | S4's parity gate and S7's proof |
| D8 Release | S7, as 2.0.0 |
| D9 Generated code in the engine | after 2.0.0 |
| D10 Main app over the database | S5 |
| D11 Outputs into the database | S4 (evidence into closures) and S5 (results kept with their design) |
| D12 Twin from the database | S4 |
