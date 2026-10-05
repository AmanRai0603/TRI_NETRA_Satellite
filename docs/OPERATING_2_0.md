# How 2.0.0 operates

> **Answer first.** On 2.0.0, everyone has one application and one shared drive, **Trinetra Database**. Each
> person looks after their own part of the design database in it every day. Everyone sees how everyone's latest
> work fits together in **today's design**, which is rebuilt from the drive whenever they open it.
> Requirements flow down through valves, and answers come back up. The system engineer is the main valve, and is
> the only one who releases the design that decisions are made on. The developer maintains only the code.
>
> **Kind:** explanation + reference · **For:** everyone · **Status:** proposed for 2.0.0 (5 Oct 2026)

The model is `docs/SYSTEM_MODEL.md`. The phases that build this are in `docs/PLAN_2_0.md`. Until 2.0.0 ships, the
1.0.0 mechanism is the one in use (`docs/GROUP_APP.md`, `docs/NODE_APP.md`, `docs/DELIVERY.md`), and this page is
the target.

*Said simply:* look after your own part, look at the whole every day, and talk about what it shows. Only the
system engineer releases, so every decision is made on one design.

## 1 · The shape of the work

**Valves.** Every mount in the tree is a valve (`docs/SYSTEM_MODEL.md` §6). Requirements flow down through it,
and answers come back up through it. The owner of the branch above controls both.

```text
                 PROGRAMME  (programme manager)
                      │  cases and customers, KPIs, gates, margin policy
                      ▼
   ═══════════  MAIN VALVE  (system engineer: the satellite's ADCS)  ═══════════
        │             │             │             │             │
        ▼             ▼             ▼             ▼             ▼
     valve         valve         valve         valve         valve      subsystem engineers:
   Sensors       Navigation    Actuators     Control      Environment   each the system engineer
        │             │             │             │             │        of their own system
       ...          nodes         nodes         nodes         nodes      node engineers, at the last break

   The facility and business groups (OILS, HILS, test equipment, catalogue, commercial, risk)
   mount on the programme's blocks, with the programme manager as their valve above.
```

**The daily rhythm.**

| when | who | does |
|---|---|---|
| every day | everyone | opens the application; sees *My work* and today's design; works on their own part; signs what is ready |
| when a group's work is right | its subsystem engineer | seals a release; it joins today's design for everyone at once |
| every day, or as the team agrees | everyone together | looks at today's design's health map and discusses what it shows |
| when today's design is right | the system engineer | releases it: the design everyone decides on |
| after a release | the programme manager | decides on it: gates, risks, cases and customers, KPIs |

**The cycle** each piece of work goes round:

1. The system sets the frame.
2. Subsystems break their branches down.
3. Nodes are written.
4. Subsystems seal.
5. Everyone sees today's design.
6. The system engineer releases.
7. The programme decides, and its decisions go back in through the main valve.

## 2 · Roles

The roles follow the tree, so the organisation and the design have the same shape. Every owner of a branch is
the system engineer of that branch.

| role | is the valve for | owns | releases |
|---|---|---|---|
| **programme manager** | the programme side | cases and customers, KPIs, cost and price, gates (the order lifecycle and the review gates), risks and beliefs, the margin policy; the facility and business groups' frame; the people and their keys; the drive | programme decisions, as releases of the `programme` branch |
| **system engineer** | the main valve: the satellite's ADCS | where each group mounts, allocations, the ADCS budgets and margins (pointing, mass, power, volume, momentum), the shared cases, the declared loop | **the design**: the only release everyone works from |
| **subsystem engineer** | their own system, at any depth | their group's breakdown, its parameters, its node engineers' assignments | their group's releases, into today's design |
| **node engineer** | the last break | their nodes | signed revisions of their nodes |
| **developer** | none: not in the design | the code only: the engine, the flight software, the twin, the library, the application, the tools | application releases |

**Deputies.** The programme file names a deputy for the programme manager, the system engineer and every
subsystem engineer. A deputy signs with their own key, and the record says so.

**Checker.** TRI-NETRA's rule that someone other than the author signs a node as checked is kept, as an optional
second signature. Each group decides in its group file whether its nodes need it before sealing (decision S1-9).

One person may hold several roles. Today the owner holds programme manager, system engineer and every subsystem
engineer's role, until people are named in phase S6.

## 3 · What it replaces

Each piece is kept, changed or retired, once. Nothing is laid over the old.

| today (1.0.0) | on 2.0.0 | |
|---|---|---|
| lead, author, checker, stage owner, owner team, user | programme manager, system engineer, subsystem engineer, node engineer, developer; a checker's signature where a group asks for it | changed |
| TRI-NETRA Files, Node and Group (three pages), and the desktop app (Fly, Runs, Design) | **one application**, with a workspace for each role, installed and as a page | changed |
| the design in the repository (`spec/`, `catalogue/`, `scenarios/`, `campaigns/`, `trades/`, `matlab_sils/cases`, `design/carry.toml`), seeded into node files | the design in the groups' signed files on the shared drive; the repository holds no design | changed |
| the tree's layers and 20 groups by discipline (`design/groups.toml`) | the programme's and the systems' own files; each group mounted on the block it answers | changed |
| stages inside a group, each signed by its owner | child blocks of the group's branch; each node signed by its node engineer | changed |
| the developer takes sealed releases in (`tools/group.py verify` and `merge`), generates code (`tools/groupcode.py`), builds a test app and delivers it (`tools/delivery.py`) | the system engineer integrates in the application, which runs the same checks | changed |
| a test application per release, accepted by the lead | today's design, which every group sees as soon as it seals | retired |
| `design.tndb`, built by the developer from repository files | today's design, rebuilt from the drive on opening; the released design, released by the system engineer | changed |
| a sign-off as a typed name | a signature with the person's own key | changed |
| `deliveries/` records and the five delivery waves | the integration record inside each released design | retired |
| node forms imported by the group app (`spec/manual/user/04_asking_for_changes.md`) | none: every change is a node signed and a group sealed, or a request in the node file | retired |
| `--set` overrides on a run, recorded only in the run's manifest | case variations, saved in `cases/` | changed |
| the results store and the `.trinetra` package | the same, every result naming the design and the engine that made it | kept |
| sealed releases (`.tnrel`), one writer per file, revisions, contracts and change requests, impact checks, the de-risking record | the same | kept |
| the pseudocode language, its interpreter and translators | the same; the *method* behaviour; a C translator joins the Rust and MATLAB ones | kept |
| the flight software's algorithms written by hand in C (`fsw/src`) and Rust (`fsw-rs/src`) from `fsw/pseudocode/`, its parameters in `fsw/params/params.toml` | the algorithms, parameters, modes and tables as nodes of `nav`, `gdn`, `ctl`, `fdir`, `act`, `sens` and `fsw`; the C and Rust generated from them; a flight image built from each released design | changed |
| the flight software's runtime: HAL, C interface, scheduler, targets, the byte link, the soft OILS emulator, the rig host | the same, in the repository | kept |
| relations compiled by hand into the time engine and the sizing (`adcs-sim-core`, `adcs-design`) | methods in the database, the engine's models generated from them; only the engine's core stays code | changed |
| the manuals and their tours (`design/manual/`) | rewritten per role, until a later release folds them into the node page | kept |

## 4 · The application

**One application, for everyone.** It has one node page, one health map, one engine and one way of working. What
a person can change is what they own. Everything else they can see, run and question, but not change.

**Two ways to open it, one application:**

| | what it is | what it can do |
|---|---|---|
| **installed** | today's desktop app (`trinetra-app`) grown into it, for Windows, macOS and Linux | the full engine: every case, Monte Carlo campaigns, soft OILS with the emulated OBC; the **flight build**, which generates the flight software from a design and compiles it for each target with its bundled toolchains. It reads and writes the drive's folder directly (Drive for desktop) |
| **from the drive** | the same screens as one page, `apps/TRI-NETRA.html`, opened in Chrome or Edge | the library and the engine compiled to WebAssembly: every node, every case run, the design graph, sweeps. It works offline and saves in place. Campaigns and soft OILS say "use the installed application" |

**Who you are** is your key. You open it once on each computer, with your passphrase. From the programme file
and the group files, the application knows your roles, and opens on **My work**.

### The workspaces

| workspace | for | holds |
|---|---|---|
| **My work** | everyone | everything that needs you now: your nodes behind, failing or unsigned; requests and issues addressed to you; previews to answer; releases waiting at your valve; decisions due |
| **Node** | node engineer | one node, edited in its live page |
| **System** | the owner of a branch: a subsystem engineer for their system, the system engineer for the satellite's ADCS | the branch, its valve below and above, today's view of it, and its release |
| **Programme** | programme manager | mission health per case and customer, gates, risks, the organisation, decisions |
| **Explore** | everyone | today's design or any released one: run, sweep, campaign, compare, the health map, trace to cause |

### Node workspace

| on screen | |
|---|---|
| left | the node's six parts, each marked done, missing or failing |
| centre | the live node page: answer, curve, cases on the curve, the method running line by line |
| right | the source the node comes from (a paper, a datasheet, a standard), open beside it |

- **Write:** the question; ports with type, unit, range, state and maturity; behaviour; cases; explanation.
  Section 7 says how data comes in.
- **See:** its neighbours and their values in today's design; its own health; any two revisions compared. For a
  node that feeds the time engine, *Fly it*: run the case with this node's current value inside today's design,
  and see the KPIs move.
- **Ask:** for a contract change or a breakdown.
- **Sign** the day's work.

### System workspace

The same workspace at every valve. The subsystem engineer opens it on their system, and the system engineer on
the satellite's ADCS.

| on screen | |
|---|---|
| left | the branch as a tree, each part coloured by its state in today's view |
| centre | the map, the N2, or a sheet of every node, at any depth |
| right | what is selected, in detail |
| top | the valve: what flows in from above (the mount, its allocations) and what is waiting from below (signed nodes, sealed releases) |

- **Frame:** break the branch down; add, wire and assign its parts; set what flows down to each. The sheet sets
  up many nodes at once.
- **Today's view:** the branch built from everyone's latest signed work below, run inside today's design, with
  its health map, closures, range verdicts and tornadoes.
- **Impact:** every value outside the branch that its changes move.
- **Release:**
  - a subsystem engineer *seals* their group, and it joins today's design;
  - the system engineer *releases* the design.
- **People:** who works on what, and what is overdue.
- **Compare** any two of anything.

The system engineer's System workspace has more:
- every ADCS budget, each across releases: the pointing error budget (APE, AKE, RPE, PDE), mass, power, volume,
  momentum and torque authority, and the flight software's CPU and memory;
- the declared loop: whether it settled, in how many iterations;
- previews and their answers;
- requests for the developer.

### Programme workspace

| on screen | |
|---|---|
| centre | mission health: each case's KPIs closed or not, with margins, by analysis and by evidence; cost and price against target |
| left | gates: the order lifecycle (enquiry → quote → PO → built → calibrated → OILS → HILS → certificate → delivery) and the review gates; what each asks, and which branches meet it |
| right | risks: the register, what moved each one, the beliefs tested and held, the de-risking narrative |
| tab | the organisation: the tree coloured by owner; each person's role, branch, nodes, issues, overdue work and deputy |

- **See:** the released design at the programme level, with today's beside it. Every failure is traced down
  through the system to its group and node.
- **Decide:** at a gate, on a risk, on a case, a customer or a KPI. Each decision is signed into the programme's
  branch and goes in through the main valve.
- **People:** members, roles, deputies and keys.

### Explore workspace

- **Open** today's design, any released design, or two side by side.
- **Run** cases and their variations, sweeps and Monte Carlo campaigns. Installed, also soft OILS, flying the
  flight image built from the design shown (or built on the spot from today's design); OILS and HILS fly a
  released design's image on the rig.
- **See** the health map at every level, with trace to cause, and each run's figures and report (`adcs-plot`).
- **Keep** results in `results/`, each naming the design and engine that made them.
- It changes nothing in the design.

## 5 · Today's design

The application builds today's design on opening, from the drive:
- the latest sealed release of every group that passes its checks;
- for a group whose latest is refused, its last good release, marked red with the reason;
- the frame from the systems branch, and the programme's branch.

Everyone who opens it builds it from the same files, so everyone sees the same design. The application says which
releases it used and when it built it. If a computer's copy of the drive is behind, it says so.

Once a day, the system engineer's application keeps a snapshot in `daily/`, so any day can be looked at again and
compared.

Today's design is for seeing and discussing. The released design is for deciding. The application never lets one
be mistaken for the other: every screen says which it shows, and a result kept from today's design says so in
its name.

## 6 · The health map: where exactly it breaks

Every workspace that shows a design colours every node, group and closure:

| state | means |
|---|---|
| **closes** | answered, and every closure it feeds holds its margin |
| **tight** | closes, but within the margin its maturity demands, or only for part of an open range |
| **fails** | a closure it decides does not hold |
| **refused** | the node refused, and says why, such as a value outside its range |
| **blocked** | it cannot run because a node it reads refused or is open; it names that node |
| **open** | not decided yet |
| **unproven** | its cases are not reproduced, it is unsigned, it is behind its contract, or its evidence comes from results made by another design |

- **It rolls up and drills down,** valve by valve. A group is as bad as its worst node, and the ADCS as bad as its
  worst group. One click goes down a level, and the N2 there shows the red wires.
- **Trace to cause.** Click a failing closure. The application walks down through what decides it, ranks the
  contributors by how far each moves the margin, and stops at the nodes that cause it. It names their group,
  their engineer, and what would make it close.
  - For an analysis closure, it walks the design graph.
  - For an evidence closure, it names the campaign and its metric, and the mapped node values that run flew.
- **Raise an issue** from any red mark, addressed to a group and a node. It closes with the release that resolves
  it.

## 7 · Bringing data in, as simply as an engineer can give it

Whatever the engineer already has, the application takes it as it is. Nobody learns a format.

| the engineer has | they | it becomes |
|---|---|---|
| a formula in a paper, a book or their notes | type it as written: `T = 0.5 * rho * v^2 * Cd * A * d` | the method; inputs found, units asked, the equation drawn |
| a table from a datasheet or a spreadsheet | copy it from Excel or Google Sheets and paste | a lookup table (a catalogue part's figures), or cases |
| a worked example in the source | paste its numbers | a case, marked as coming from that source |
| results from their own MATLAB, Python or spreadsheet | save them as CSV and drop the file in | cases and results; the code itself kept as the record |
| a single number with a source | type it with its unit: `550 km`, `30 mN m s`, `0.01 deg` | a stated value, converted to SI, with its source |
| a number they are unsure of | type a range or a spread: `2e-13 to 9e-12`, `0.8 ± 10%` | an open value with its range |
| a PDF, a picture, a datasheet | drop it beside the node | the node's source, linked to its page |
| many nodes to set up | fill the System workspace's sheet, a row each, or paste whole columns | nodes, each opening in its own page |
| something close to what they need | *start from a similar node*, or a template for its kind | a copy, ready to change |
| a customer's requirements | the case CSV in the fixed format (`spec/SPEC.md` §8.3) | a case in `cases/` |

Every field shows an example of what goes in it, and says in plain words what is wrong as it is typed. Units are
read as people write them and converted to SI behind the scenes. An Excel file can be dropped in directly once a
reader for it is vetted into the application, the way SQLite is today (`design/vendor/`).

## 8 · Making it easy to use

1. **My work first.** It always opens on what needs you now.
2. **One next step, named:** *Sign revision 4*, *Seal env 1.2*, *Release design 2026.11.1*.
3. **Checked as you type, in plain words, with the fix.** *Case 3 expects 2.1e-6 N m and the method gives
   1.9e-6 N m: the drag coefficient differs.* Never a code or a hash.
4. **Live.** Answers, curves, cases, closures and the health map move as you edit.
5. **Nothing is lost.** Every save is a revision, every step can be undone, every older file is kept, and a refusal
   never deletes.
6. **One application, one page, one vocabulary.** The same node page in every workspace, one word for each thing
   (`docs/GLOSSARY.md`). People and dates are shown, not fingerprints.
7. **Pictures first, examples always.** A template for each kind of node, a worked example beside every step, and
   the guide for the screen one click away.
8. **It works where people are.** Installed or from the drive, online or not. It says plainly when a browser
   cannot save in place, and what to do.

These follow Nielsen's usability heuristics, and the ideals of local-first software: the data is the
organisation's, on its own machines, usable offline, and readable without the vendor.

## 9 · Parameters and measures, by level

A parameter is a stated node, owned by the level that decides it. A level below can see it and ask for it to
change, never change it.

| level | parameters it owns | measures it watches |
|---|---|---|
| programme | each case's KPIs; the margin policy by maturity; the confidence the design is held to (the campaign percentile, today p99.73 for pointing); gate criteria; cost and price targets | KPIs closed per case, by analysis and by evidence; cost against target; gate readiness; risks open and moved; beliefs tested; issues by owner and age; built-in relations still in code |
| system | allocations to each group; the ADCS budgets and margins (including the flight software's CPU, memory and deadline budget); shared cases and scenarios; the declared loop and its tolerance | each budget's margin, today and across releases; the flight image's measured CPU, memory and deadline misses in soft OILS; groups whose latest release was refused; previews unanswered; closures failing, tight and refused; whether the loop settled |
| subsystem | what flows to its own parts; its design choices, ranges and stated values | its nodes proven of total; open and overdue nodes; issues on the group; contract changes in flight |
| node | its inputs' ranges and its cases | its checks, its cases, both ends of its range |

## 10 · The files

Every file is one SQLite database in one schema (`design/schema.toml`, format 2). Its `meta` says what kind it
is, its format version, what it was based on, and who wrote it.

| file | name | holds | written by | changes after |
|---|---|---|---|---|
| **node file** | `<node>.node.tndb` | one node, all six parts | its node engineer | yes: each save a revision |
| **group file** | `<group>.group.tndb` | the branch, its members and their public keys, who is assigned what at which contract version, its mount (read-only), its base | its subsystem engineer; for the programme and the systems, their owners | yes |
| **group release** | `<group>-<version>.tnrel` | the group file and every node, every signature, release notes, the seal; for the programme, its signed decisions | its owner, sealing | never |
| **preview** | `preview-<group>-<version>.tnrel` | the released design with one release in it, when that release moves other groups' values | the system engineer | never |
| **preview answer** | `<group>-<version>.<answering group>.tnanswer` | *fine*, or *object, because…*, signed | the answering subsystem engineer | never |
| **issue** | `<number>-<group>.tnissue` | what is wrong, where, the evidence from the health map, whom it is addressed to | whoever raises it | never; closed by the release that resolves it |
| **daily snapshot** | `design-<date>.daily.tnrel` | today's design as built that day, and the releases it used | the system engineer's application | never |
| **released design** | `design-<version>.tnrel` | every integrated release, mounted into one tree; the integration record; the application version and toolbox it needs; the system engineer's signature | the system engineer | never |
| **flight image** | `design-<version>.<target>.tnfsw` | the flight software built from a design for one target (POSIX, QEMU Cortex-M, a board): the generated sources, the binary, the runtime and toolchain versions, the configuration blob, and the evidence that it reproduces the interpreter on every node's vectors | the flight build, at the system engineer's release (or on request for today's design, kept in `results/`) | never |
| **case** | `<name>.tncase` | the inputs for a run (the customer's CSV lines, the scenario, the campaign settings), or a variation: its base and what it changes | anyone | yes |
| **results** | `<name>.trinetra` | a run's answers, its figures and report, and the design and engine that made them | the application | never |
| **key** | `<person>.tnkey` | a person's private key, locked by their passphrase | the person | kept by the person, never on the shared drive |

**One writer per file.** Nothing is merged by two people. The application refuses to sign for anyone but the
file's assigned writer, and integration refuses a signature from anyone the group file did not assign.

## 11 · The work, step by step

Steps W1 to W10 are the cycle; W11 to W16 can happen at any time. Each step says who, in which workspace, which
file, what is checked, and the message that goes with it.

### W1 · The system sets the frame

1. **System engineer**, in the System workspace on the satellite's ADCS, sets:
   - the mounts;
   - the allocations handed to each group, with their direction;
   - the ADCS budgets and margins;
   - the shared cases and scenarios;
   - the declared loop.
2. **To start a new group:** add its mount, then *Start group*. That writes `groups/<group>/` with its group
   file, mount and base, and its subsystem engineer from the programme file.
3. **Programme manager:** shares a new group's folder. Its subsystem engineer registers their node engineers'
   keys.
4. A change to the frame shows at once in today's design, and goes out with the next release (W8).

Message, to a new group's subsystem engineer: *"<group> is set up. Open it in the application."*

### W2 · A subsystem sets its own frame

1. **Subsystem engineer**, in the System workspace on their system, breaks the branch down. Each new node starts
   open, with its question and ports; the sheet sets up many at once.
2. They assign each node a node engineer, and issue the node files into `nodes/`.

Message: *"Your node <node> is in your My work. Due by <gate>."*

### W3 · A node is written, every day

1. **Node engineer**, in the Node workspace, brings the data in (section 7) and writes the node in its live page.
   It runs as it is written, inside today's design. A node that feeds the time engine can be flown from here. A
   flight algorithm runs in SILS through the interpreter at once, and can be built into a soft OILS image to try.
2. Each save is a new revision in place. If the file moved on since it was opened, the application stops and
   overwrites nothing.
3. They sign the day's work when its checks pass and its cases agree. The signed revision appears at once in
   today's view of their group. If the group asks for a checker, the node waits for the checker's signature too.

### W4 · A contract changes

1. **Node engineer:** asks, as a request in the node file.
2. **Subsystem engineer**, in the System workspace, changes the contract. Its version rises, and every node that
   reads it is named.
3. They re-issue the changed files. Each keeps its content and reads *behind* until re-signed.
4. A change to the group's own mount is asked of the system engineer (W1).

Message: *"The contract of <node> changed: <what>. Please check and re-sign."*

### W5 · A subsystem seals, and its work joins today's design

1. **Subsystem engineer**, in the System workspace, opens today's view of the group, built from every node
   engineer's latest signed revision. It checks that:
   - every node is signed by its node engineer, at its current contract (and by its checker, where asked);
   - nothing is behind;
   - there are no duplicate files;
   - the group's checks pass.
2. They see it run inside today's design: the health map, every closure the group touches, and every value
   outside the group it moves.
3. They update the base first if a newer design has been released (W12).
4. They write the release notes as a de-risking record: what was believed, what was tested, what is now known,
   and any issues it resolves. The application proposes the version.
5. They seal. The release goes into `releases/`, and **joins today's design for everyone** the next time anyone
   opens it.

If the release moves values in other groups, the application names them, and those subsystem engineers see it in
their My work.

### W6 · Everyone looks at today's design, and discusses it

1. **Everyone**, in Explore or their own workspace, looks at today's design and its health map at their level.
2. **Together,** every day or as the team agrees: the health map on one screen, red marks first, each traced to
   its cause and owner. Each is either understood, or raised as an issue (W11).

### W7 · Values that move are answered

When a release moves another group's values:
1. **Each subsystem engineer named** sees their group run with the change, and answers **fine** or **object,
   because…**, signed.
2. **System engineer**, on an objection, decides one of three:
   - ask the releasing group for its next version;
   - change an allocation in the frame;
   - release anyway, with the objection kept in the record and raised to the programme manager.

### W8 · The system engineer releases the design

1. **System engineer**, in the System workspace on the satellite's ADCS, runs today's design through the
   integration checks. A release is refused whole, with the lines that failed, if any of these fails:
   - the seal and the signature chain;
   - the base still fitting;
   - every node's method, cases and record;
   - no relation supplied by an assistant;
   - both ends of every declared range;
   - the declared loop settling within its tolerance;
   - every evidence closure's results made by this design (or re-flown).
2. Before releasing:
   - every red mark is understood, and either fixed or raised as an issue;
   - every answer is in.
3. *Release design* signs it. The application writes:
   - the design into `design/` under its version, and as `design.tnrel`;
   - the flight images, one per target, into `design/flight/`, each built and checked by the flight build;
   - the one before it into `design/archive/`;
   - `NOTES.md`, `readable/` and the status.

This is the release every decision refers to. No one else can make it.

Message, to everyone: *"Design <version> is released: <what changed; what still fails, and its issues>."*

### W9 · Each level reviews the release

| who | looks at | decides |
|---|---|---|
| node engineer | their node in the released design | whether it still holds; asks for a change (W4) |
| subsystem engineer | their system's health map and closures | their system's next changes (W2, W5); issues to raise |
| system engineer | every budget, closure and red mark, across releases | the next frame (W1); what to ask of which group |
| programme manager | the programme level (W10) | |

### W10 · The programme decides

1. **Programme manager**, in the Programme workspace, sees the released design at the programme level: each
   case's KPIs, cost and price, gate readiness and risks, with every failure traced to its group and node.
2. They decide:
   - pass a gate or not;
   - accept a risk, or ask for it to be retired;
   - change a case's KPI, or the margin policy;
   - take on a new case (a customer's CSV).
3. Each decision is signed and sealed in a release of the programme's branch. It joins today's design at once,
   and goes into the next release through the main valve.

Message: *"Programme decision: <what>, on design <version>."*

### W11 · An issue is raised and resolved

1. **Anyone**, from a red mark in any health map, chooses *Raise issue*. It is written into `issues/`, addressed
   to a group and a node, with the evidence.
2. **Subsystem engineer** sees it in My work and assigns it. It is resolved by a release whose notes name it.
3. It closes when that release is in today's design and the mark is no longer red.

### W12 · A group's base moves

When a design is released, each group's System workspace shows what changed under it: the inputs it reads from
others, and its mount. *Update base* takes the new mount and the neighbours' values. No node's content changes,
and a node whose inputs changed in meaning is named for its node engineer to check.

### W13 · A node is broken down further

1. **Node engineer, or anyone,** proposes it as a request in the node file.
2. **Subsystem engineer** decides one of two:
   - **Break down.** The node's behaviour becomes its children, and its old relation is kept as its estimate.
   - **Mount a new group,** when other engineers will own what is inside. The system engineer starts it (W1),
     and the subsystem engineer becomes its valve above, the system engineer of that new system.
3. The children are issued and written (W2, W3). The node's own cases run against them, beside the estimate.
4. It is sealed as usual (W5): minor if no port changed, major if one did. Integration checks one thing more:
   the children reproduce the node's cases.

### W14 · Something needs the code

This covers a new toolbox function, port type, picture or check, a new flight target or rig channel, and any
fault in the application, the engine's core, a translator, the flight software's runtime or the rigs.

1. **Any engineer** writes the request in My work's *Requests*, with an example.
2. **Developer** builds it in an application release (W15). Until then the node stays open, or keeps its
   estimate, and says what it waits for.

### W15 · The application is released

**Developer only.**
1. The new application must run the **current released design** on every shared case and give the same answers,
   and its flight build must produce flight software that behaves the same on every node's vectors and every
   scenario. This is held by the regression copy in the repository (`tests/regression/`). One that changes the
   design's answers does not ship.
2. A release from `main`, tagged `v<version>`, built by the release workflow (kits for each OS, the Python wheel,
   `SHA256SUMS`).
3. The application, installed and as a page, goes into `apps/` with its checksums, and `apps/NOTES.md` says what
   changed and whether anyone must act.

### W16 · Something goes wrong

| what | what happens |
|---|---|
| a released design is wrong | every earlier one is in `design/archive/`; the system engineer releases the last good one again under a new version, and the notes say why |
| a group release is wrong | a release is never edited; its subsystem engineer seals the next version, and today's design takes it at once |
| a group's latest release is refused | today's design keeps that group's last good release, marked red, until the next one passes |
| a node file was saved by download and left in Downloads | the subsystem engineer's view names the node as unchanged since its last signature |
| someone is away | their deputy signs |
| the drive cannot be reached | the application works on the files already on the computer; nothing is lost, only delayed |
| results were made by an older design | they are marked stale; evidence closures that rest on them read *unproven* until re-flown |

## 12 · Status, and who hears what

There is no server, so status is part of what the application builds. Every workspace shows, for each group:
- its working version, and its last sealed release;
- whether today's design took it or refused it;
- its issues open, and previews unanswered;
- which released design holds it.

The system engineer's application writes it to `readable/Status.csv` once a day, with the snapshot.

Every hand-off is one file in an agreed place, and appears in the receiver's My work. The same words also go by
email or chat, so nobody has to be watching:

| step | from | to | the file | the message |
|---|---|---|---|---|
| W1 | system engineer | subsystem engineer | `groups/<group>/<group>.group.tndb` | the group is set up |
| W2 | subsystem engineer | node engineer | `groups/<group>/nodes/<node>.node.tndb` | your node, due by a gate |
| W4 | subsystem engineer | node engineers named | re-issued node files | the contract changed |
| W5 | subsystem engineer | everyone, in today's design | `groups/<group>/releases/<group>-<version>.tnrel` | sealed; and, to those whose values move, please answer |
| W7 | subsystem engineer | system engineer | the answer | fine, or object |
| W8 | system engineer | everyone | `design/design.tnrel` | design released |
| W10 | programme manager | everyone it concerns | `groups/programme/releases/` | programme decision |
| W11 | anyone | subsystem engineer | `issues/<number>-<group>.tnissue` | an issue on your group |
| W14 | any engineer | developer | the request | what is needed |
| W15 | developer | everyone | `apps/` | application released, and whether to act |

## 13 · Versions

| what | version | rule |
|---|---|---|
| node file | revision 1, 2, 3 … | every save; a signature names the revision it signed |
| node contract | contract version 1, 2 … | raised when the question, a port, a unit or a range changes; the node reads *behind* until re-signed |
| group release | major.minor, such as 1.2 | **major** when anything another group reads changes; **minor** otherwise. The conversion's baselines are 0.1 |
| today's design | the date, and the releases in it | rebuilt on opening; kept daily as a snapshot |
| released design | year.month.sequence, such as 2026.11.1 | one per release by the system engineer |
| application | 2.0.0, 2.1.0 … | the code; a design names the oldest application that can run it |
| results | the run's id | names the design version (or today's date and releases) and the engine version |

**Every file names what it was based on:**
- a node revision, its contract version;
- a group file, the design it was based on;
- a release, its previous release and its base;
- a design, every release in it;
- a result, its case and its design.

So "what changed since" always has an answer, and the application compares any two.

**Where the history lives:**

| history of | kept in |
|---|---|
| each edit to a node or group file | the file itself |
| what a group sealed | `releases/`, never deleted |
| how the whole stood each day | `daily/` |
| what was integrated, checked and answered | each released design's integration record |
| every released design | `design/archive/` |
| a second copy of the current released design | the repository's `tests/regression/`, refreshed at each application release |

Google Drive keeps its own file history too; it is a backup, not the record.

## 14 · People and keys

Each person makes a key once, in the application, and keeps the private half as `<person>.tnkey` on their own
computer, locked by a passphrase.

- **The anchor.** The programme manager's key fingerprint is written in START HERE, which only the programme
  manager can edit. The application shows it on first use and remembers it.
- **The chain.**
  - The programme file lists the system engineer's, every subsystem engineer's and every deputy's public key.
  - Each group file lists its node engineers' and checkers'.
  - A node signature checks against its group file, a group seal against the programme file, and a released
    design against the system engineer's key there.
- **A lost key** is replaced the way it was registered. Old signatures still check against the old public key,
  which stays in the file's history.
- **Someone leaves.** Their nodes are reassigned and signed afresh.

Signing works the same installed or from the page, with no extra library and no network. Ed25519 signatures and
the passphrase lock (PBKDF2 and AES-GCM) are part of Web Crypto in every current browser, and the installed
application uses the same algorithms.

## 15 · The shared drive

```text
Trinetra Database/
  START HERE.txt                 the drive, the daily rhythm, the programme manager's key fingerprint
  MANIFEST.json                  what the developer shipped, with sizes and SHA-256
  apps/                          the application: installed for each computer, and as a page · CHECKSUMS · NOTES.md
  guides/                        one guide per role; the model; the engineering documents
  design/
    design.tnrel                 the released design
    design-2026.11.1.tnrel       the same, under its version
    NOTES.md                     what changed in each, newest first
    flight/                      the flight images of the released design, one per target
    archive/                     every earlier release, with its images
  daily/                         a snapshot of today's design, one per day
  integration/                   previews, and the answers to them
  issues/                        every issue raised
  readable/                      Groups · Nodes · Ports · Wires · Closures · Health · Status, as CSV
  groups/
    programme/                   programme.group.tndb · nodes/ · releases/
    systems/                     systems.group.tndb · nodes/ · releases/
    env/                         env.group.tndb · nodes/ · releases/
    …                            one folder per group
  cases/                         shared cases and their variations
  results/                       results people chose to keep, each naming its design
```

**Who may change what.** The programme manager sets this once, by folder:

| folder | can edit | can read |
|---|---|---|
| `apps/`, `guides/` | the developer | everyone |
| `design/`, `daily/`, `readable/`, `integration/` | the system engineer | everyone |
| `integration/<group>-<version>/` | also the engineers whose values it moves, for their answers | everyone |
| `issues/`, `cases/`, `results/` | everyone, each their own file | everyone |
| `groups/programme/` | the programme manager | everyone |
| `groups/systems/` | the system engineer | everyone |
| `groups/<group>/` | that group's subsystem engineer and node engineers | everyone |

**Rules the application holds to:**
- `releases/` holds sealed releases only; anything else there is ignored, and the application says so.
- Two files where there should be one, such as a name ending `(1)`, are named, and their owner keeps one.
- Google-converted files, code and backup copies in the design folders are named.
- An application release never touches the design.

## 16 · The application and the files across versions

- **Every file states its format version, and the application states the formats it reads.**
- **A newer application opens an older file,** upgrades it when it saves, and keeps the old one beside it. The
  1.0.0 format (format 1) upgrades this way.
- **An older application refuses a newer file by name,** and says which version it needs.
- **The page in `apps/` and the installed application are one release.** Replacing them is the update; each
  shows its version.
- **A design names the oldest application that can run it.**
- **An application release never changes anyone's files.**

## 17 · The developer's side

The repository holds the code and nothing of the design:
- the toolbox the methods call, with units, and the pseudocode interpreter and its translators to Rust, C and
  MATLAB;
- the time engine's core (`adcs-sim-core`, `adcs-sim`, `adcs-pop`): integrator, step order, recorder, metrics;
  its models are generated from the design;
- the flight software's runtime: the HAL, the C interface, the scheduler, the configuration blob, the targets
  (`fsw/targets`), and the flight build;
- the rigs' software: the soft OILS emulator and byte link, the OILS and HILS rig host;
- the plotting (`adcs-plot`) and the MATLAB twin's runner (`matlab_sils/`), whose functions are generated;
- the one library that reads, writes and checks every file, and runs the design graph (`trinetra-design`);
- the application, installed (`trinetra-app`) and as a page;
- the CLI, the Python package and the tools;
- the tests.

The tests run on the example group (`tests/fixtures/`), and on the copy of the current released design taken when
preparing an application release (W15). `CONTRIBUTING.md` governs the code. The design's rules are enforced by
the library's checks, which the application runs, and are described for people in these pages.

| branch or tag | holds |
|---|---|
| a feature branch | one change, by pull request |
| `main` | the code, reviewed and green |
| tag `v<version>` | an application release |

## 18 · Where this breaks

- **The main valve is one person.** Everyone sees today's design without waiting for them, but every decision
  waits for their release. A deputy is named, and every step is in the record.
- **Today's design is only as current as each computer's copy of the drive.** The application says which
  releases it used, and when.
- **Seeing is not deciding.** Today's design is for looking and discussing. A decision made on it instead of a
  release has nothing signed behind it, so every screen says which one it shows.
- **A message not sent is a hand-off not made.** My work shows it either way; sending it is still a person's job.
- **The drive enforces folders, not files.** One writer per file is held by the application and by
  integration's signature check.
- **There is no code review for the design.** In its place: comparisons, health maps, today's design in front of
  everyone every day, checkers where a group asks, and signatures.
- **Trace to cause ranks what moves a margin.** It cannot say which node is wrong; that is the engineers'
  judgement.
- **Evidence costs time.** A campaign behind an evidence closure takes minutes to hours. Until it is re-flown on
  the new design, its closure reads unproven, not passed.
- **A passphrase forgotten is a key lost.** It is replaced, never recovered.

## 19 · References

| for | see |
|---|---|
| usability: status, error prevention, recovery, recognition rather than recall | Nielsen's ten usability heuristics ([NN/g](https://www.nngroup.com/articles/ten-usability-heuristics/)) |
| the data is the organisation's, offline, lasting beyond the software | Kleppmann et al., *Local-first software*, Ink & Switch, 2019 ([inkandswitch.com](https://inkandswitch.com/local-first/)) |
| opening a whole folder from a page, to read and write | the File System Access API ([web.dev](https://web.dev/articles/files/open-a-directory)) |
| signing and locking keys in the browser | the Web Cryptography API: Ed25519, PBKDF2, AES-GCM ([W3C](https://www.w3.org/TR/WebCryptoAPI/)) |
| a change reviewed against the main file before it is merged | branching and review in design tools ([Figma](https://help.figma.com/hc/en-us/articles/360063144053)) |
| margins by maturity, checked at each review | ANSI/AIAA S-120A-2015 ([NASA NTRS](https://ntrs.nasa.gov/citations/20130014265)) |
| ranking what moves a result | Saltelli et al., *Global Sensitivity Analysis: The Primer*, 2008 ([overview](https://en.wikipedia.org/wiki/Variance-based_sensitivity_analysis)) |
| pointing requirements and their verification | ECSS-E-ST-60-10C; ECSS-E-ST-10-02C (verification) |
