# TRI-NETRA 2.0.0: one design database, one application

> **Answer first.** The next release is **2.0.0**. After it, the design lives only in the shared drive
> **Trinetra Database**, as one kind of block broken down to any depth. Each person looks after their own part
> in **one application**, and everyone sees **today's design**, rebuilt from everyone's latest work whenever
> they open it. Requirements flow down and answers flow back up, through a valve at every mount. The system
> engineer is the only one who releases the design that decisions are made on, and the developer keeps only
> the code.
>
> It is built in nine phases, **S0 to S8**. The database itself comes first: phase S3 gives you **zip 1**,
> which you upload by hand. Every later phase is checked in CI and merged on your word.
>
> **Kind:** plan · **For:** the owner and the developer side · **Status:** proposed 5 Oct 2026, waiting for the
> owner's decisions (§9)

This plan applies the owner's model of 5 Oct 2026 (a system model, an operating model and a release plan) to
TRI-NETRA. It **replaces** `docs/DATABASE_FIRST_PLAN.md` (§8 says where each D package went) and the gap list in
`docs/VISION_VS_CURRENT.md`. It renumbers the technical roadmap's releases (§10).

Phase ids are **S0–S8**, so they do not collide with the roadmap's (E, M, G, F, A, Y, T, P, O, TS, DD, DB, D,
U, B).

---

## 1 · The model, applied to TRI-NETRA

### 1.1 One block, any depth

There is one kind of element, the **block**; the team keeps calling it a node. Blocks have three relations:

| relation | means | in TRI-NETRA today |
|---|---|---|
| **contains** | a parent holds its children, to any depth | only group → stage → layer. **New:** a `parent` on every block |
| **connects** | an output of one block is an input of another | the group files' `edge` table (derivation and contribution edges) |
| **closes** | a requirement against an achieved value, with a direction and a margin | 38 KPI closures (`closure_analysis`, `closure_verified`) and one `closure_interface`, answered by `tools/evaluate.py` |

A block's **behaviour** is exactly one of five:

| behaviour | its answer | TRI-NETRA today (765 nodes) |
|---|---|---|
| **method** | its pseudocode, run by the interpreter | 48 nodes have pseudocode |
| **children** | what its children give at its outputs | 166 `internal` nodes |
| **stated** | a value a person states, with its source | 51 `declared` and 152 with a cited value source |
| **lookup** | a table and how to read it | the catalogue (parts, products, algorithms), today in `catalogue/*.toml` |
| **open** | not decided yet; refused by name if a run reaches it | most leaves; each node file lists its gaps |

A sixth behaviour, **built-in**, is temporary. It means the relation is still compiled code, found by the node's
id. 166 nodes cite Rust code today. Each one is marked as built-in, listed by its owner, and replaced by a method
after 2.0.0 (§7).

**Rule:** a block sees inside its children only through their inputs and outputs. This is today's C04 layer
check ("layers meet only at the door and the interface rows"), applied at every depth.

**Layers become perspectives.** TRI-NETRA's four layers become tags on a branch, not floors:

| layer today | perspective | owned by |
|---|---|---|
| 1 company | programme | the programme manager |
| 2 the satellite's ADCS | system | the system engineer |
| 3 subsystem | subsystem | each subsystem engineer |
| closure addition | the closures sit on the blocks they close | the system engineer (KPIs) |
| 4 the run | the bottom of every branch: a case's inputs | everyone (cases) |

### 1.2 Every value carries its state, its maturity and, while open, its range

| field | values | seeded from today's content |
|---|---|---|
| **state** | decided · allocated · open · achieved | stated with a source → decided; a `required` row's bound → allocated; computed → achieved; anything else → open |
| **maturity** | estimated · calculated · measured | **estimated**, unless the cited source says the value was calculated or measured. Nothing is promoted by guess |
| **range** (while open) | lower and upper, each with its reason | 82 nodes already carry `lower`/`upper` and their reasons; the rest are open with no range, and are listed |

A closure then gives one of three range verdicts: it closes for the whole range (the decision can wait), for part
of it (decide soon; here is the crossing), or for none of it (change the design).

The **margin demanded** depends on the least mature input. The policy is a programme parameter. The proposal is
20 % for estimated, 10 % for calculated and 3 % for measured (ANSI/AIAA S-120A-2015 graded allowances), for you
to set.

Each closure ranks its open inputs with a **tornado chart**. Variance-based indices come later (Saltelli 2008).

### 1.3 The N2 and loops

- Every block has its own N2, drawn from its wires: feed-forward above the diagonal, feedback below it.
- A loop is declared on the smallest block that holds it, with what must settle and how tightly. An undeclared
  loop is refused by name, never run forever.
- TRI-NETRA declares **no loop today** (`spec` §5: "the ADCS tree declares no cycle yet"). It has one real loop,
  run outside the tree by `tools/pipeline_design.py`: actuator sizing → mass and inertia → disturbance and
  slew demand → sizing. S4 declares it on the `design` block and iterates it there.

### 1.4 Groups, mounts and valves

A group owns a branch. The branch mounts on a block of the branch above it, and **that mount is the valve**:
the owner above sets what flows down (allocation, bound, direction) and decides what passes back up.

The top of the tree is owned the same way. Two new groups hold it:
- **programme**, layer 1's frame;
- **systems**, layer 2's frame.

Every other group mounts on one of their blocks. The proposed mounts are below; you confirm them in S1. Today's
14 interface rows (`l3_<sid>_interface`) become the mounts of the subsystem groups.

| group | mounts on | through today's |
|---|---|---|
| **programme** (new) | the root | layer-1 branches: case intake, catalogue, commercial, order lifecycle, test facility, standards, supply, heritage, risk |
| **systems** (new) | programme · *Satellite ADCS* (today's one door, `sys_satellite_adcs`) | layer-2 branches: pointing service, ADCS configuration, mission and orbit, satellite as the ADCS sees it, ADCS subsystems, verification |
| dyn · Satellite dynamics | systems · *Satellite as the ADCS sees it* | layer-2 rows s1–s4 |
| env · Environment and disturbance torques | systems · *Disturbance torques* and *Environment along the orbit* | `l3_dist_interface` |
| sens · Sensors | systems · *Attitude sensors* | `l3_sens_interface` |
| nav · Navigation and attitude estimation | systems · *Attitude estimation* | `l3_est_interface` |
| gdn · Guidance and mode management | systems · *Modes and FDIR* | `l3_modes_interface` |
| fdir · FDIR | gdn · *fault handling* (one level down) | — |
| ctl · Controller and allocation | systems · *Control and allocation* | `l3_ctl_interface` |
| act · Actuators | systems · *Magnetic actuation*, *Reaction wheels*, *Fluid momentum rings*, *Reaction control thrusters* | `l3_mtq/rw/fmr/rcs_interface` (one group, four mounts) |
| fsw · Flight software and OBC | systems · *Flight software and OBC interfaces* | `l3_fsw_interface` |
| design · Design loop | systems · *ADCS unit budgets* | `l3_budget_interface` |
| pnt · Pointing error budget | systems · *Pointing error budget* | `l3_pnt_interface` |
| kpi · Pointing service and KPI closures | systems · *Pointing service the customer needs* | the 39 closure rows |
| vv · Verification and standards | systems · *Verification*, and programme · *Standards & compliance* | layer-1/2 rows |
| oils · OILS rig | systems · *OILS rig needs*, and programme · *Test facility* | `l3_oils_interface` |
| hils · HILS rig | systems · *HILS rig needs* | `l3_hils_interface` |
| lab · Test equipment | programme · *Test facility* | layer-1 rows |
| catalogue · Catalogue and heritage | programme · *Catalogue*, *Heritage* | layer-1 rows |
| business · Commercial, orders and supply | programme · *Commercial*, *Order lifecycle*, *Supply chain* | layer-1 rows |
| risk · Risk management | programme · *Risk management* | layer-1 rows |
| case · Case and mission | **dissolved** (decision S1-6): case-intake rows → programme; mission requirements and hardware fitted → systems; each customer is a **case file** | — |

**Proposed next level.** These are open blocks, until the subsystem engineer breaks them down:

| group | proposed children |
|---|---|
| sens | magnetometer · Sun sensors · star tracker · gyro · GNSS receiver · Earth sensor |
| act | magnetorquers · reaction wheels · CMG and VSCMG · fluid momentum rings and their pump · RCS thrusters and feed |
| nav | attitude filter (MEKF) · orbit propagation on board · sensor processing |
| ctl | detumble laws · pointing controllers · momentum management · allocation |
| gdn | mode logic · guidance profiles · Sun and nadir referencing |
| env | atmospheric density (DTM2020) · magnetic field (IGRF) · solar pressure · gravity gradient · aerodynamic torque |
| dyn | mass properties · flexible modes · magnetic cleanliness |
| fsw | OBC and timing · interfaces (HAL) · data handling · fault detection |
| pnt | knowledge error · control error · stability |
| oils, hils, lab | rig host · emulation channels · stimulators · stands · safety |

### 1.5 Today's design and the released design

| | made from | for | changes |
|---|---|---|---|
| **today's design** | every group's latest sealed release that passes its checks, plus the programme's and the systems' frame | seeing how everyone's latest work fits together, every day | whenever anyone seals |
| **released design** | what the system engineer chose to pass on, signed | every decision, review and gate | when the system engineer releases |

The same two views exist at every valve: a group's "today" is its node engineers' latest signed revisions.

**Every screen says which of the two it shows.**

### 1.6 The ADCS time engine inside this model

TRI-NETRA has two engines, and the model has to hold both:

1. **The design graph.** Requirements, relations, budgets, closures: rows that compute from rows. Today this is
   `tools/evaluate.py` and the pseudocode interpreter (`design/js/pcode.js`, `tools/pcode.py`). In 2.0.0 it runs
   the blocks' behaviours directly from the design files.
2. **The time simulation** (`adcs-sim`, with the C and Rust flight software and the MATLAB twin). It flies a
   **case** over time and produces evidence metrics (APE, AKE, settling time, and so on).
   - In 2.0.0 it reads **every input from the design**: the node values it is mapped to (today 162 of 213 case
     lines have a node) and the catalogue's lookup blocks.
   - Its metrics come back as the evidence for the `closure_verified` blocks.
   - Its algorithms stay compiled, as **built-in** by node id, in 2.0.0. Running the code generated from the
     nodes' pseudocode instead (`adcs-groups`) is the first migration after 2.0.0, one group at a time, each with
     a parity test (§7).

So a node's value reaches the numbers the spacecraft flies, and a flight's result reaches the closure it serves.
Both directions are traced.

### 1.7 Cases, variations and results

- A **case** (`<name>.tncase`) holds the inputs for one run: the customer's CSV lines, the scenario and the
  campaign settings.
- A **variation** is a case that names its base and lists only what it changes. Today's `--set` overrides become
  saved variations.
- **Results** (`<name>.trinetra`, the package that exists today) name the design version, its fingerprint and
  the engine version that made them. They are marked stale when either changes.
- The reference cases, the 48 scenarios, the 8 campaigns and the 12 trades become case files in
  `cases/`. The library data that is not a case (catalogue, dispersions, KPI definitions, delivery waves)
  becomes blocks of the group that owns it.

---

## 2 · Roles

The roles follow the tree. Every owner of a branch is the system engineer of that branch.

| role | is the valve for | owns | releases |
|---|---|---|---|
| **programme manager** | the programme side | customers and cases, KPIs, cost, gates, risks, the margin policy; the people and their keys; the drive | programme decisions, as releases of the `programme` branch |
| **system engineer** | the main valve: the satellite's ADCS | where each group mounts, allocations, system budgets and margins, shared cases, declared loops | **the design**: the only release everyone works from |
| **subsystem engineer** | their own branch, at any depth | its breakdown, its parameters, its node engineers' assignments | their group's releases, into today's design |
| **node engineer** | the last break | their nodes | signed revisions of their nodes |
| **developer** | none; not in the design | the code only | application releases |

- Each of the first three roles has a **deputy**, who signs with their own key; the record says so.
- One person may hold several roles.
- The names replace "lead", "author", "checker", "owner team" and "user" on every page, screen and guide.
- The seed invents no people. You name them in S6.

---

## 3 · What changes, what stays, what goes

| today in TRI-NETRA | in 2.0.0 | |
|---|---|---|
| Files, Node and Group apps (`design/pages/*.template.html`); the desktop app's Fly/Runs/Design page | **one application** with five workspaces (My work, Node, System, Programme, Explore), installed and as a page from the drive | changed |
| design content in the repository (`spec/`, `catalogue/`, `scenarios/`, `campaigns/`, `trades/`, `matlab_sils/cases`, `design/carry.toml`) | the design files on the drive; the repository holds no design | changed (moved once, in S3) |
| `tools/seed_design.py`, `tools/carry_over.py`, `tools/export_catalogue.py` | used once by the conversion, then retired | retired |
| `design.tndb`, a copy built from repository files | today's design, rebuilt on opening; the released design, released by the system engineer | changed |
| `tools/group.py verify/merge` | the library's integration checks, run in the application | changed |
| `tools/delivery.py`, per-group test apps, acceptance by the lead | today's design, which every group sees as soon as it seals | retired |
| sign-off as a typed name | a signature with the person's own key | changed |
| `.tnrel` sealed releases, one writer per file, revisions, change requests, contracts and their versions, impact checks | the same | kept |
| the pseudocode language, interpreter and translators (`docs/PSEUDOCODE_V2.md`) | the same; the method behaviour | kept |
| the engine, flight software, MATLAB twin, results store and `.trinetra` package | the same code, reading the design and naming it in every result | kept, re-wired |
| `adcs-groups` generated code | kept; switched into the engine one group at a time after 2.0.0 | kept |
| the Python package and the CLI | kept; they read the design through the library | kept |

---

## 4 · The files and the drive

### 4.1 File kinds

Every file is one SQLite database in one schema (`design/schema.toml`, format 2). Its `meta` says its kind,
format version, what it was based on, and who wrote it.

| file | name | holds | written by | changes |
|---|---|---|---|---|
| node file | `<node>.node.tndb` | one block: identity, ports, behaviour, cases, explanation, history | its node engineer | each save a revision |
| group file | `<group>.group.tndb` | the branch, its members and their public keys, assignments at contract versions, its mount (read-only), its base | its subsystem engineer; programme and systems by their owners | yes |
| group release | `<group>-<major.minor>.tnrel` | the group file and every node, every signature, the notes, the seal | its owner, sealing | never |
| preview | `preview-<group>-<version>.tnrel` | the released design with one release in it, when that release moves other groups' values | the system engineer | never |
| preview answer | `<group>-<version>.<answering group>.tnanswer` | *fine*, or *object, because…*, signed | the answering subsystem engineer | never |
| issue | `<number>-<group>.tnissue` | what is wrong, where, the evidence, to whom | whoever raises it | never; closed by the release that resolves it |
| daily snapshot | `design-<date>.daily.tnrel` | today's design as built that day, and the releases it used | the system engineer's application | never |
| released design | `design-<year.month.seq>.tnrel` (and `design.tnrel`, the current one) | every integrated release in one tree; the integration record; the application version and toolbox it needs; the system engineer's signature | the system engineer | never |
| case | `<name>.tncase` | a run's inputs, or a variation of another case | anyone | yes |
| results | `<name>.trinetra` | a run's answers, and the design and engine that made them | the application | never |
| key | `<person>.tnkey` | a private key locked by a passphrase | the person | never on the drive |

**One writer per file.** The application refuses to sign for anyone but the file's assigned writer. Integration
refuses any signature the group file did not assign.

### 4.2 Versions

| what | version | rule |
|---|---|---|
| node file | revision 1, 2, 3 … | every save; a signature names the revision it signed |
| node contract | 1, 2 … | raised when the question, a port, a unit or a range changes; readers show *behind* until re-signed |
| group release | major.minor | **major** when anything another group reads changes, **minor** otherwise |
| today's design | the date and the releases in it | rebuilt on opening; kept daily |
| released design | year.month.sequence, such as 2026.11.1 | one per release by the system engineer |
| application | 2.0.0, 2.1.0 … | a design names the oldest application that can run it |

Every file names what it was based on, so "what changed since" always has an answer, and any two of anything
can be compared node by node.

### 4.3 The drive: `Trinetra Database`

```text
Trinetra Database/
  START HERE.txt         the layout, the daily rhythm, the programme manager's key fingerprint
  MANIFEST.json          every file the developer shipped, with size and SHA-256
  apps/                  the application (installed kits and the page) · CHECKSUMS · NOTES.md   ← zip 2
  guides/                one guide per role, the model, the engineering documents
  design/                design.tnrel · design-<version>.tnrel · NOTES.md · archive/
  daily/                 one snapshot of today's design per day
  integration/           previews and the answers to them
  issues/                every issue raised
  readable/              Groups · Nodes · Ports · Wires · Closures · Health · Status, as CSV
  groups/
    programme/           programme.group.tndb · nodes/ · releases/
    systems/             systems.group.tndb · nodes/ · releases/
    env/                 env.group.tndb · nodes/ · releases/
    …                    one folder per group (20 in all)
  cases/                 shared cases and their variations
  results/               results people chose to keep, each naming its design
```

**Who may change what.** The programme manager sets this once, by folder:

| folder | can edit | everyone can read |
|---|---|---|
| `apps/`, `guides/` | the developer | yes |
| `design/`, `daily/`, `readable/`, `integration/` | the system engineer (and, in `integration/<release>/`, the engineers whose values it moves) | yes |
| `groups/programme/` | the programme manager | yes |
| `groups/systems/` | the system engineer | yes |
| `groups/<group>/` | that group's subsystem engineer and node engineers | yes |
| `issues/`, `cases/`, `results/` | everyone, each their own file | yes |

**Rules the library enforces:**
- `releases/` holds sealed releases only; anything else there is ignored, and the application says so.
- Drive conflict copies (`… (1).tndb`) are named, and their owner keeps one.
- Google-converted files, code and backups are refused.
- An application release never touches the design.

### 4.4 Where the history lives, and what GitHub keeps

| history of | kept in |
|---|---|
| each edit to a node or group file | the file itself (revisions) |
| what a group sealed | `releases/`, never deleted |
| how the whole stood each day | `daily/` |
| what was integrated, checked and answered | each released design's integration record |
| every released design | `design/archive/` |
| a second copy of the current released design | the repository, `tests/regression/`, refreshed at each application release (W15) and used as the regression for "an application release must give the same answers" |

Drive's own version history is a backup, not the record. GitHub holds the code, the example group and that
regression copy, so every released design also lives in GitHub.

---

## 5 · The work, step by step (W1–W16, for TRI-NETRA)

This section adapts the operating model's steps to TRI-NETRA. Each step is a test case in S5.

| step | who · where | what |
|---|---|---|
| W1 the system sets the frame | system engineer · System workspace on the satellite's ADCS | mounts, allocations with direction, system budgets (mass, power, pointing, momentum), shared cases; *Start group* writes `groups/<g>/` |
| W2 a subsystem sets its frame | subsystem engineer · System workspace on their branch | breaks the branch down (open blocks with question and ports), assigns node engineers |
| W3 a node is written | node engineer · Node workspace | brings data in (§5.1), writes the block in its live page, runs it inside today's design, signs the day's work |
| W4 a contract changes | node engineer asks · subsystem engineer changes it | version rises, every reader is named and reads *behind* until re-signed |
| W5 a subsystem seals | subsystem engineer | checks every node is signed at its current contract, nothing behind, no duplicates; sees the change run inside today's design; writes the de-risking notes; seals; it joins today's design for everyone |
| W6 everyone looks at today's design | everyone · Explore | the health map, red marks first, each traced to its cause and owner |
| W7 values that move are answered | each subsystem engineer named | *fine* or *object, because…*; the system engineer decides on an objection |
| W8 the system engineer releases | system engineer | the integration checks (seal and chain, base fits, methods, cases, no assistant-supplied relation, both ends of every range); every red mark understood; *Release design* |
| W9 each level reviews | everyone | their part of the released design |
| W10 the programme decides | programme manager · Programme workspace | gates, risks, customers, KPIs, margin policy; a signed release of `programme` |
| W11 an issue | anyone, from a red mark | addressed to a group and node; closed by the release that resolves it |
| W12 a group's base moves | subsystem engineer | *Update base*; nodes whose inputs changed in meaning are named |
| W13 a node is broken down further | subsystem engineer decides | children reproduce the node's cases; the old relation is kept as its estimate; or a new group mounts there |
| W14 something needs the code | any engineer · My work → Requests | a new toolbox function, port type, picture, check, or a fault |
| W15 the application is released | developer | must give the current released design's answers unchanged on every shared case; tagged `v<version>`; into `apps/` |
| W16 something goes wrong | as the operating model says | release the last good design again; a group seals its next version; a refused release falls back to the last good one, marked red |

### 5.1 Bringing data in

Whatever the engineer has, the application takes it as it is:
- a formula as written;
- a table pasted from a spreadsheet;
- a worked example → a case marked with its source;
- CSV results from MATLAB or Python;
- a number with its unit (`0.5 mN·m`, `250 km`);
- a range or a spread (`2e-7 to 8e-7`, `55 ± 10 %`), which becomes an open value;
- a PDF or datasheet beside the node;
- many nodes set up at once from the sheet;
- *start from a similar node*.

Units are converted to SI behind the scenes. Every field shows an example and says in plain words what is wrong.

### 5.2 The health map

Every node, group and closure is coloured by one of seven states:
- closes, tight, fails;
- refused (and why);
- blocked (by which node);
- open, unproven.

The colours roll up valve by valve. **Trace to cause** walks down from a failing closure, ranks the contributors
by how far each moves the margin, and stops at the nodes that cause it, naming their group and engineer.

---

## 6 · The phases

| phase | needs | delivers | your part | size |
|---|---|---|---|---|
| **S0 · Safe ground** | — | the engine and app refuse a design they cannot run; results name their design; the 1.0.0 notes corrected | approve the notes text | S |
| **S1 · The model, rules and roles** | — | TRI-NETRA's system model and operating model; the rules as checks; role names; the mount table; your decisions (§9) | **approve, with a second reviewer** | S–M |
| **S2 · One schema, versions, keys, one library** | S1 | format 2 for every file kind; versions; Ed25519 keys; one library with every check; upgrade of today's files | — | L |
| **S3 · The design leaves the repository → zip 1** | S2 | the whole design converted into the drive's layout; checked; **zip 1** | **delete the old folders, upload zip 1**, run or ask for the check | M–L |
| **S4 · The engine runs the design** | S3 | today's design built from the drive; node values fly; closures, ranges, tornadoes, health map; the declared loop; **the parity gate** | — | L |
| **S5 · The application** | S2, S4 | one application with five workspaces doing W1–W16 | **try it in every role** | L |
| **S6 · The drive and the people** | S5 | **zip 2** (`apps/`, `guides/`, START HERE); keys; sharing; one guide per role | **name the people, share the folders, register keys; upload zip 2** | S–M |
| **S7 · Proof with no developer** | S6 | first released design; the `env` group round the whole cycle | **do the round; say "Ship 2.0.0"; push the tag** | M |
| **S8 · Release 2.0.0** | S7 | release notes, kits, the regression copy | — | S |

S0 can run beside S1 and S2. Zip 1 cannot come before S3: the database must be written in the schema the
application will use, or it would have to be uploaded twice.

### S0 · Safe ground

- A design names the application version and toolbox it needs. The engine, the desktop app and the Python
  package refuse a mismatch by name.
- Every result carries the design's fingerprint and the engine's version, and is marked stale when either
  changes (extends the provenance in `docs/RESULTS.md`).
- Fix the database-reading faults the data-flow audit found:
  - `adcs size` fails under the database;
  - DE440 is not found from an empty working directory;
  - `evaluate` crashes on the pack root;
  - a folder or an unreadable database is not refused clearly.
- Correct the published 1.0.0 notes: the KPI passes were not demonstrated (A1), and power counts actuators only
  (Y1). You approve the text; you edit the release on GitHub.

**Done when:**
- a design built for another engine is refused in a test;
- every result names its design;
- each of those four faults has a test that failed before the fix.

### S1 · The model, rules and roles

- `docs/SYSTEM_MODEL.md`: §1 of this plan written out in full, with the references (§11).
- `docs/OPERATING_MODEL.md`: §2, §4 and §5 written out: the daily rhythm, the workspaces, the steps, status and
  who hears what, versions, keys, the drive.
- The **design's rules**, written as checks for S2's library:
  - one writer per file, and the signature chain;
  - an expected value never comes from the code under test;
  - no relation supplied by an assistant, and a transcription signed by the person who checked it;
  - every requirement says which way it binds;
  - a block reads its children only through their ports;
  - a release is never edited;
  - a parameter is changed only by the level that owns it;
  - the design is released by the system engineer alone;
  - today's design is never shown as a released one.
- The **code's rules** (`CONTRIBUTING.md` and the area rule files), rewritten for a repository that holds only
  code. An application release must give the current released design's answers unchanged (W15).
- Role names in `docs/GLOSSARY.md`. `docs/RELEASE_PLAN.md` §1, `docs/GROUP_APP.md`, `docs/NODE_APP.md`,
  `docs/DELIVERY.md` and `docs/RULES_PROPOSAL.md` are marked *replaced at 2.0.0* (they describe what is in use
  until then).

**Done when** you approve the model, the rules, the roles and the mount table, with a second reviewer.

### S2 · One schema, versions, keys, one library

- **Format 2** of `design/schema.toml`, for every file kind in §4.1:
  - **block** with `parent` to any depth, `perspective`, `behaviour`, `contract_version`;
  - **port**: type, unit, range with its reasons, state, maturity, direction (`>=`, `<=`), owner and due gate
    while open, bundle;
  - **wire, mount, closure, loop** (what settles, tolerance, iteration limit);
  - **case** with variations;
  - **text, table, media**;
  - **signature, change request, issue, preview, answer, integration record**.
- **Port types:** number with unit, whole number, choice, yes/no, list, and parameter first. Table, time series
  and uncertain value come later.
- **Identity and versions** by §4.2.
- **Keys:**
  - make a key, lock it by passphrase, sign, and check through the chain (programme file → system and subsystem
    engineers and deputies → each group file's node engineers);
  - Ed25519, with the lock done by Web Crypto in the browser: no library, no network.
- **One library**, `design/js/tnlib/`. It reads, writes and checks every kind of file, and holds every check now
  spread over:
  - `tools/tndb.py`, `tools/group.py` and `tools/delivery.py`;
  - `design/js/node_model.js`, `structure.js` and `release.js`.

  The checks cover: seal, signatures, method and units, cases, both ends of every range, the de-risking record,
  and the assistant rules. The pages use the library directly. The Python tools and CI call it through Node
  (`pcode_cli.mjs` already works this way). `tools/tndb.py` becomes a thin reader, kept equal to it by test.
- **Upgrade:** a format-1 file opens, upgrades when saved, and keeps a copy. Nothing a group wrote is dropped.
- **Compare** any two revisions, releases or designs, node by node.

**Done when:**
- all 765 node files and 20 group files upgrade with nothing dropped (a field-by-field test);
- signatures check through the chain;
- the library refuses everything `group.py verify` and `delivery.py` refuse today, with a test for each.

### S3 · The design leaves the repository → **zip 1**

This is the last time the design passes through the code.

1. **Convert**, with one tool, `tools/convert_2_0.py`, run once:
   - all 765 nodes, with their parents, perspective tags, state, maturity and ranges (seeded by the rules in
     §1.2);
   - the edges as wires;
   - the 14 interface rows as mounts;
   - the 39 closure rows as closures on the blocks they close;
   - the new `programme` and `systems` groups;
   - the proposed next level (§1.4) as open blocks;
   - parameters placed at the level that owns them.
2. **The library data becomes design:**
   - `catalogue/*.toml` (parts, products, algorithms, components, modes, families, classes) → lookup and stated
     blocks of `catalogue` and `act`/`sens`;
   - dispersions, KPI definitions and delivery waves → blocks of `vv`, `kpi` and `programme`;
   - the reference cases, the 48 scenarios, the 8 campaigns and the 12 trades → `cases/*.tncase`, with today's
     `--set` uses as variations.
3. **Baseline releases:**
   - each group gets release **0.1**, sealed by the conversion and marked *converted, not yet signed by a person*;
   - today's design builds from these baselines, and every node shows **unproven** until its engineer signs it.
4. **Readable copies:**
   - `readable/*.csv`;
   - `readable/Conversion.csv`, which says where each of today's fields went: every field of every node, every
     catalogue entry and every case line, each placed or listed as dropped. Nothing may be listed as dropped.
5. **Pack zip 1**:
   - the layout of §4.3 without `apps/`;
   - `guides/` with the model, the operating model, this plan and the engineering documents;
   - START HERE with what works yet and what does not;
   - `MANIFEST.json`.
6. **Your upload:**
   - delete the old `Apps/` and `Design/` in Trinetra Database;
   - move the two sheets and the old `Guides/` aside (or keep them; the check lists them as extras, not
     problems);
   - upload zip 1's content;
   - run `python3 tools/drive.py --verify "<Trinetra Database>" --first-upload` on a downloaded copy or Drive
     for desktop, or tell me and I check it from a copy you send.
7. **The repository** keeps the example group in `tests/fixtures/`. `spec/`, `catalogue/`, `scenarios/`,
   `campaigns/`, `trades/` and `matlab_sils/cases` leave the build. They are kept read-only under
   `archive/design-1.0/` until 2.0.0 ships, then deleted.

**Until the first released design (S7) the developer may issue a corrected conversion** (zip 1.1, replacing
`groups/` and `cases/` only), because nobody has written on the drive yet. After S7, never.

**Done when:**
- the conversion report lists nothing as dropped;
- verify passes on the packed folder and on your uploaded copy;
- the repository builds and tests with no design data in it, except the example group.

### S4 · The engine runs the design

- **Today's design** is built from the drive on opening:
  - from each group's latest sealed release that passes its checks;
  - a refused release is replaced by its group's last good one, marked red with the reason;
  - it says which releases it used and when.

  The same holds one valve down, from signed node revisions.
- **The design graph** runs every behaviour from the files: method (interpreter), children, stated, lookup, open
  (refused by name), built-in (by node id, marked).
- **The time engine** takes every case input from the node it is mapped to (D3), and every catalogue input from
  the lookup blocks.
  - A mapped node with no value is refused by name.
  - Every run's manifest lists each input's node and revision.
  - Its metrics return as evidence to the `closure_verified` blocks.
- **One reader** for every tool: the design loop, campaigns, evaluate, trace, pointing budget, V&V report, the
  Python package, the CLI and the desktop app.
  - The MATLAB twin flies an export generated from the design (D12).
  - A CI job with the repository's design data removed runs the full chain.
- **The declared loop** (`design`, §1.3) is iterated on its block, with its tolerance. Any other cycle is refused
  by name.
- **Every value** carries state, maturity and range. **Every closure** gives its range verdict and tornado.
- **Every node** gets its health state, rolled up valve by valve, with trace to cause.

**The parity gate.** For every row of today's design (`docs/END_TO_END.md`), and for every run in the committed
results (both cases, every scenario, every campaign), the new path gives today's answer within the row's own
tolerance, and refuses where today refuses. There is no switch without it.

**Done when:**
- parity holds for the whole design;
- today's design is built identically on two computers from the same files;
- a node broken on purpose is traced to by name from the KPI it breaks.

### S5 · The application

There is one application. Installed, it is today's desktop app grown into it, with the full engine. From the
drive, it is the same screens as one page in Chrome or Edge, working offline and saving in place. It knows each
person by their key and opens on **My work**.

| workspace | holds |
|---|---|
| **My work** | what needs you now: nodes behind, failing or unsigned; requests and issues to you; previews to answer; releases waiting at your valve; decisions due |
| **Node** | left: the six parts, each done, missing or failing. Centre: the live page (answer, curve, cases on the curve, the method running line by line). Right: the source beside it. Sign. Ask for a contract change or a breakdown |
| **System** | at every valve: the branch as a tree, the map, the N2 at any depth, the sheet of every node; frame, today's view, impact, seal or release, people, compare. For the system engineer: every budget (mass, power, momentum, pointing, link, thermal) across releases, previews and their answers, requests to the developer |
| **Programme** | mission health per case and customer; gates; risks with their de-risking narrative; the organisation; decisions |
| **Explore** | today's design or any released one, side by side; run cases and variations, sweeps, campaigns; the health map with trace to cause; results kept, each naming its design |

- **Bringing data in** works every way in §5.1.
- **Made easy to use** by eight points:
  1. My work first.
  2. One named next step.
  3. Checks in plain words, with the fix.
  4. Live.
  5. Nothing lost.
  6. One vocabulary.
  7. Pictures and examples.
  8. Works offline, and says when a browser cannot save.

  These follow Nielsen's heuristics and local-first software.
- Today's Files, Node and Group apps, the test-app template, `delivery.py` and acceptance are removed at S7.

**Done when** CI drives every workflow, W1–W13, on the example group, installed and as a page, with no developer
step. That includes:
- a second writer stopped;
- a seal seen by a second person in today's design;
- a refused release replaced by its last good one and marked;
- an objection answered;
- a failing closure traced, raised as an issue and closed by the release that fixes it;
- a breakdown whose children reproduce the node's cases, and one whose children do not;
- a new group mounted;
- a design released and a programme decision taken back in;
- each way of bringing data in.

**Your trial:** at the end of S5 you get the application on a copy of the converted design. You live with it for
some days in every role, round W1 to W10. S6 does not start until your answers are in.

### S6 · The drive and the people → **zip 2**

- `tools/drive.py` (today's `drive_pack.py`) packs only what the developer owns:
  - `apps/` with the installed kits, the page, `CHECKSUMS` and `NOTES.md`;
  - `guides/`;
  - START HERE.

  Zip 2 adds these. It never touches `groups/`, `cases/` or `design/`.
- One guide per role (node engineer, subsystem engineer, system engineer, programme manager, developer), each
  with the role's day and its steps.
- The sharing table (§4.3), as a checklist for the programme manager.
- **Your part:**
  - name the system engineer, each subsystem engineer and each deputy;
  - each makes their key;
  - you write your fingerprint into START HERE;
  - you register the keys and share the folders.

**Done when** someone who has never seen the application can follow each role's guide, from an empty drive to
a released design.

### S7 · Proof with no developer

1. The system engineer releases the first design from the baselines, which become signed releases group by
   group.
2. **The `env` group goes round the whole cycle, W1 to W10.** `env` is the pilot because it has the most nodes
   with pseudocode (15). The round covers:
   - one node rewritten as a method from its source;
   - a range on air density (DTM2020 at the design F10.7), whose tornado ranks it;
   - sealed, seen in today's design, answered by the groups it moves (`act`, `pnt`), released, reviewed and
     decided on.
3. You say **"Ship 2.0.0"**.

**Done when** `env`'s release is in a released design, decided on by the programme, without the developer
touching it.

### S8 · Release 2.0.0

- The release notes say what 2.0.0 changes for each role, and what is still built-in, by owner.
- The kits and the Python wheel as in 1.0.0, plus the page and zip 2.
- `tests/regression/` holds the current released design.
- **You push the tag** `v2.0.0`; this session cannot push tags.

---

## 7 · After 2.0.0

- **Built-in to method.** Each group writes methods for its built-in relations in the Node workspace.
  - The application runs both on the node's cases and across its range, and shows any difference.
  - Once a method is released, a later application release deletes the built-in.
  - 166 nodes start built-in.
- **Generated code in the time engine.** The flight algorithms run from their nodes' pseudocode (`adcs-groups`)
  one group at a time, each reproducing the hand-written model on every scenario first.
- **Groups break their branches down further**, as data, with no developer.
- **The technical roadmap resumes** (§10), every change to design data made in the application.
- **Later:**
  - variance-based sensitivity and the probability each closure holds;
  - an optimiser over the open ranges with every closure as a constraint;
  - each closure's verification method;
  - table, time-series and uncertain-value ports;
  - SysML v2 and FMI 3.0 exchange.

---

## 8 · Where the database-first plan went

| D package (`docs/DATABASE_FIRST_PLAN.md`) | now |
|---|---|
| D1 Library | S3 step 2: the library data becomes blocks and case files, not a separate library kind |
| D2 Build | S4: today's design and the released design replace the single `design.tndb` build |
| D3 Nodes drive numbers | S4 (time engine inputs from nodes) |
| D4 One reader | S4 |
| D5 Sync | replaced: the application works on the drive's folder itself; the repository keeps only the regression copy (§4.4) |
| D6 Pack and upload | S3 (zip 1) and S6 (zip 2) |
| D7 End to end | S4's parity gate and S7's proof |
| D8 Release | S8, as **2.0.0** |
| D9 Generated code in the engine | after 2.0.0 (§7), with built-in by node id until then |
| D10 Main app over the database | S5 |
| D11 Outputs into the database | S4 (evidence into closures) and S5 (results kept with their design) |
| D12 Twin from the database | S4 |

The three items the comparison found missing are all covered:
- a versioned common database: released designs, daily snapshots and the archive (§4);
- cases, variations and results tied to a design: §1.7;
- flying a node from the node app: the Node workspace runs inside today's design (S5).

---

## 9 · Decisions for you (S1)

1. **Version 2.0.0** for this release, since the file formats, the application, the roles and where the design
   lives all change.
2. **The model:** one recursive block; layers become perspectives; "a block reads its children only through
   their ports".
3. **Role names** (§2), replacing lead, author, checker and owner team everywhere.
4. **The mount table** and the proposed next level (§1.4), including `programme` and `systems` as groups.
5. **`act` stays one group with four mounts**, or splits into four groups (one per actuator family).
6. **`case` dissolves**: its rows go to programme and systems, and each customer becomes a case file.
7. **One library in JavaScript**, used by the pages and the installed app, with the Python tools calling it.
8. **The time engine's algorithms stay built-in in 2.0.0**, and move to generated code after.
9. **Baseline releases 0.1** sealed by the conversion and marked unsigned, so today's design exists from day one.
10. **Maturity seeding:** estimated unless the source says otherwise; the margin policy (20 / 10 / 3 %) is yours
    to set.
11. **The file extensions** in §4.1.
12. **The pilot group** for S7: `env`.

---

## 10 · The roadmap after 2.0.0

The technical roadmap (`docs/TECHNICAL_ROADMAP.md`) is renumbered:

| release | phase | content |
|---|---|---|
| v2.1 | U0 | correct 1.0.0's wrong results and the critical flight-software and GNC defects |
| v2.2 | U1 | environment, frames and requirements you can cite; orbit-propagator fixes |
| v2.3 | U2 | stability margins, statistics, budgets, independent referents |
| v2.4 | U3 | devices, GNC, system completeness, FMECA |
| v2.5 | U4 | flight-software assurance, TM/TC, Renode soft OILS |
| v3.0 | U5 | board OILS and HILS (hardware-paced) |

From 2.0.0 on, every change to design data is made in the application by its owner; every change to the code
is made in the repository.

---

## 11 · Where this plan breaks

- **The parity gate is the hard part.** If the new path cannot reproduce today's numbers, S4 stops until it
  does, and everything after it waits.
- **The application now carries everything.** With no developer in the loop, every check the developer made by
  hand must be in the library, and the application must be easy enough that nobody needs one. S2 and S5 are the
  largest phases, and your trial is the test.
- **Two engines.** The design graph and the time simulation are joined through mapped inputs and evidence
  metrics. A case line with no node (51 of 213 today) stays a case value, owned by the case's writer, until a
  group claims it.
- **Built-in relations are a debt.** On 2.0.0 most computing nodes still compute in code. They are marked,
  listed by owner, and replaced after, not hidden.
- **The drive enforces folders, not files.** One writer per file is held by the application and by the
  signature check at integration.
- **Ranges and maturities are only as honest as whoever states them.** Each is signed, with its source shown.
- **The main valve is one person.** A deputy is named, and every step is in the record.
- **A passphrase forgotten is a key lost.** It is replaced, never recovered.

## 12 · References

| part | rests on |
|---|---|
| recursion: a system's elements are systems | ISO/IEC/IEEE 15288:2023; SEBoK, *Recursion* |
| perspectives, not floors | Roques, *Systems Architecture Modeling with the Arcadia Method*, ISTE |
| blocks, ports, connections, requirements, verification cases | OMG SysML v2 and the Systems Modeling API, 2025 |
| N2 as a design structure matrix | Eppinger and Browning, *Design Structure Matrix Methods and Applications*, MIT Press, 2012 |
| a solver on the group that holds the cycle | Gray et al., *OpenMDAO*, Struct. Multidisc. Optim. 59, 2019 |
| ranges, not blanks | Sobek, Ward and Liker, *Toyota's principles of set-based concurrent engineering*, MIT Sloan Management Review, 1999 |
| margins by maturity | ANSI/AIAA S-120A-2015, mass properties control for space systems |
| which input moves a result | Saltelli et al., *Global Sensitivity Analysis: The Primer*, Wiley, 2008 |
| a model's ports as data any tool can read | FMI 3.0 |
| usability | Nielsen, *Ten usability heuristics* |
| the data is the organisation's, offline | Kleppmann et al., *Local-first software*, Ink & Switch, 2019 |
| opening a folder from a page | the File System Access API |
| one file as the application's document | SQLite as an application file format |
| ECSS pointing terms (APE, AKE, RPE, PDE) | ECSS-E-ST-60-10C |
