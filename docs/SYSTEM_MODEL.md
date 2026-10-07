# The system model

> **Answer first.** TRI-NETRA defines any system the same way: one kind of block, broken down from the top to
> any depth. Requirements flow down the blocks, and answers flow back up to decide every margin. This page gives
> that model, the ADCS as it is broken down today, and the published practice each part rests on.
>
> **Kind:** explanation + reference · **For:** everyone · **Status:** proposed for 2.0.0 (5 Oct 2026). Until
> 2.0.0 ships, the tree is held as it is today (`spec/SPEC.md` §5, `design/groups.toml`)

This page is the design's architecture: how a question becomes a tree of blocks, how the blocks connect, and
how the tree decides whether the design closes.

The other pages:
- the code's architecture: `docs/CODE_ARCHITECTURE.md` (written in phase S1);
- how the model is operated, who holds what and where every file sits: `docs/OPERATING_2_0.md`;
- how the work reaches 2.0.0: `docs/PLAN_2_0.md`.

It is a living page. The model in sections 1–7 changes rarely, and by review, like a rule. The breakdown in
section 8 changes as groups break their branches down; when it does, this page follows.

## 1 · Flows down, closes up

A design starts as one question at the top: *can this ADCS give this customer the pointing service they ask
for?* It is broken into the things that decide it, and each of those into the things that decide it. This stops
when every piece is something a person can state, look up or calculate.

- **Requirements flow down.** A parent hands a child a bound and the direction it binds in:
  - `<=` must stay under it (absolute pointing error at most 0.01°);
  - `>=` must reach it (slews per orbit at least 4).
- **Answers flow up.** Only the bottom calculates. An achieved value is never typed at the top; it arrives from
  below, from a relation or from a flight of the time engine.
- **A closure decides.** It holds a requirement, the value achieved, and the margin between them. A change
  anywhere re-closes every margin above it, and a red margin names the block that caused it.

TRI-NETRA closes in two ways, both kept:
- **by analysis:** the design graph computes the achieved value (`closure_analysis`, 16 KPIs);
- **by evidence:** a campaign of the time engine supplies the metric (`closure_verified`, 22 KPIs).

*Said simply:* the top decides what is needed, the bottom decides what is possible, and the closure says
whether the two meet.

## 2 · One block, any depth

There is one kind of element, the **block**. The team calls it a node. There are three relations between blocks:

| relation | means | held today by |
|---|---|---|
| **contains** | a parent holds its children, to any depth | group → stage → layer only; **new in 2.0.0:** a parent on every block |
| **connects** | an output of one block is an input of another; every wire | the group files' `edge` table: 238 derivation edges, 23 contribution edges, 46 relations |
| **closes** | a requirement against an achieved value, with a direction and a margin | the 38 KPI closures and the closure layer's interface row |

A block's **behaviour** is exactly one of seven (`design/schema.toml`, `[behaviours]`; every program that checks a
file holds each block to them):

| behaviour | the block's answer is | example |
|---|---|---|
| **method** | its pseudocode, run on its inputs (`docs/PSEUDOCODE_V2.md`) | `gd_1` aerodynamic torque |
| **children** | whatever its children give at its outputs | `l3_dist_row_11` total disturbance torque |
| **stated** | a value or a description a person states, with its source | `m2_0` altitude, from the case |
| **lookup** | a table and how to read it | a reaction wheel from the datasheet catalogue (`catalogue/`) |
| **evidence** | what a run, a campaign or a rig measures: the metric or source it names, by result id; open until one gives it | `p1a_0` APE achieved, from a campaign's `ape` |
| **closure** | its requirement against its achieved value (its closure row), compared by the library in the requirement's sense | `kpi_absolute_pointing_error_ape_verified` |
| **open** | not decided yet: a draft, refused by name if a run reaches it | a node whose gaps are listed |

A block that describes code by the boundary (section 7: the flight software's runtime, a rig, a test) is stated, and
says so in its content (`code.boundary`: runtime or test); it is not a relation and never built-in.

**Built-in** is a temporary sixth behaviour. The relation is still compiled code (`adcs-sim-core`,
`adcs-design`), found by its node's id until its group writes a method for it. It is marked as such on every
screen.

Built-in is allowed only while the release is being built. The plan removes every built-in before 2.0.0 ships
(`docs/PLAN_2_0.md`, S7). What remains code after that is the toolbox and the runtime (section 7), never a
relation.

A block stops being broken down when its behaviour is a method, a stated value, a lookup, or a wire from
another group.

**A block sees inside its children only through their inputs and outputs.** This is TRI-NETRA's rule that layers
meet only at the door and the interface rows (the node app's check C04, `tools/validate_plan.py`), applied at
every depth instead of at four fixed floors.

### Breaking a node down later

The node a group calls its last break is only the last break *so far*. Any node can be broken down later, and
nothing above it has to change.

1. **Its outside stays:** the same identity, ports and contract. Everything that reads it reads the same ports,
   so no other node and no other group is edited. A breakdown that also changes a port is a contract change,
   and is treated as one.
2. **Its behaviour becomes its children.** The method, value or table it had is kept as its **estimate**. The
   answer from its children is shown beside it, and the difference is a margin like any budget's.
3. **Its cases stay, and test the breakdown.** The children together must reproduce the node's own cases within
   their tolerance. If they do not, a case or the breakdown is wrong, and the version that broke it down says
   which.
4. **Its children belong to the same group,** unless another team will own them. Then the node becomes a
   **mount**: a new group hangs there, and the node's group is to it what the systems group is to a subsystem
   group.
5. **It can be folded back.** A broken-down node can return to a method, for speed or for an earlier stage of a
   study. Its children stay in the release history.

Example: `l3_dist_row_07`, atmospheric density at altitude, is a piecewise-exponential table today. It can be
broken into the solar and geomagnetic activity it depends on and a DTM2020 evaluation. Every reader of its
density port is unchanged.

*Said simply:* breaking a node down replaces what is inside the box, never the box.

**Depth and perspective are different things.** Programme, system and subsystem are *perspectives*: what kind of
question a branch answers. They are tags on a branch, not floors. A block six levels inside the actuators is
still a subsystem block. The run, where a case is set, is the bottom of every branch: its inputs, not its rows.

| TRI-NETRA layer today | perspective in 2.0.0 |
|---|---|
| 1 · the company | programme |
| 2 · the satellite's ADCS | system |
| 3 · subsystem, and the closure addition | subsystem; a closure sits on the block it closes |
| 4 · the run | the case: the inputs at the bottom of every branch |

## 3 · What a block holds

Six parts, each fact written once.

1. **Identity:** the question it answers, its owner, its group, its parent, its perspective, its version and its
   contract version.
2. **Ports:** its inputs and outputs.
   - Each has a type, a unit, a range and the reason for each end of the range.
   - Types: a number with a unit, a whole number, a choice (such as the actuator family: magnetorquer, reaction
     wheel, CMG, VSCMG, fluid ring, RCS), yes or no, a list, a table, a time series, an uncertain value, text,
     a file.
   - Ports can be bundled, so a group passes one bundle (an orbit, a set of mass properties, an actuator
     product) instead of ten wires.
3. **Behaviour:** one of the five above. The equation on the page is drawn from the method, and the range refusals
   are generated from the ports; neither is written twice. Today `in 150 .. 1000` ranges are written inside the
   pseudocode signature and again in the node's `lower`/`upper`; in 2.0.0 the port holds them once.
4. **Evidence:** cases, each with an input, an expected answer, a tolerance and where the answer came from (a
   textbook worked example, a datasheet, a flight record, a MATLAB twin run). **An expected value never comes from
   the code under test.**
5. **Explanation:** said simply, the theory, each assumption and when it fails, and the sources (`docs/references.toml`).
6. **History:** every version: what was believed, what was tested, what is now known, what changed. This is the
   de-risking record that TRI-NETRA keeps today as beliefs and narratives (`spec/derisk/`). Sign-offs are kept,
   each against the content it was given for.

## 4 · Every value carries its state, its maturity and, while open, its range

**State.** Every port, at every level, is in one of four states:

| state | means | example |
|---|---|---|
| **decided** | a value stated at this level, with who, when and why | design altitude 550 km |
| **allocated** | a bound handed to a child | ADCS unit mass at most 1.6 kg; APE contribution of knowledge error at most 0.004° |
| **open** | to be decided below, with an owner and the gate it is due by | star-tracker noise, open until the quote stage |
| **achieved** | computed from the children | 1.42 kg |

So the top can decide what it must and move on, without inventing what it cannot know yet. A block may start
with an estimate, such as the ADCS taking 12 % of the satellite's mass, and be broken down later. The estimate
stays as the budget, the children compute the real value, and the difference is the margin. Mass, power and
pointing budgets are run this way on space programmes, and TRI-NETRA's pointing error budget
(`results/POINTING_BUDGET.md`) is already one.

**Maturity.** Every value is estimated, calculated or measured.
- A closure demands the margin its least mature input needs, so an early estimate cannot look as safe as a
  measurement.
- Mass control on space programmes grades every item this way and carries a growth allowance by grade
  (ANSI/AIAA S-120A).
- TRI-NETRA's sizing applies one margin to every part today (the design loop's `margins` knob); the model grades
  it by maturity, for every value.

**Range.** An open value carries the range it may still take, not a blank. Every closure then says one of three
things:
- it closes for the whole range: the decision can wait;
- it closes for part of it: decide soon, and here is the crossing;
- it fails for all of it: change the design, not the timing.

This is set-based design: carry the set of acceptable values and narrow it as knowledge arrives (Sobek, Ward and
Liker, 1999). TRI-NETRA's sweeps and Monte Carlo campaigns already explore ranges. The model attaches the range
to the value itself, so every closure reads it.

**Which value to decide first.** Each closure ranks the open values behind it by how far each one moves its
margin:
- first a tornado, one input at a time;
- later, variance-based indices that include interactions (Saltelli et al., 2008).

The widest bar is the decision that matters most. For the aerodynamic torque at low altitude, it is the
atmospheric density across the solar cycle.

*Where this breaks:* a range drawn too narrow, or a maturity claimed too high, makes a closure look settled.
Ranges and maturities are stated, sourced and signed like any other value.

## 5 · Connections, and the N2

The **N2** of a block is the matrix of its children:
- each child sits on the diagonal;
- its outputs run along its row and its inputs down its column, with a mark where one child feeds another;
- a mark above the diagonal feeds forward; a mark below it feeds back, and is a loop.

It is a design structure matrix (Eppinger and Browning, 2012), and it is drawn from the wires, never separately.

- **Every block has its own N2,** so the matrix opens to any depth. At the top, it shows the hierarchy as nested
  boxes on the diagonal.
- **A mark that leaves a box** is a connection that crosses a boundary, and that block needs a port for it.
- **A loop belongs to the smallest block that contains it.** It is declared there, with what must settle and how
  tightly, and iterated there. An undeclared loop is refused by name, never run forever.
  - TRI-NETRA's tree declares no loop yet (`spec/SPEC.md` §5).
  - Its one architectural loop runs outside the tree today, in the design loop (`docs/DESIGN_LOOP.md`,
    `tools/pipeline_design.py`): the actuators sized to the demand set the ADCS mass and its contribution to the
    inertia, which set the disturbance and slew demand, which set the sizing.
  - In 2.0.0 it is declared on the `design` block and iterated there. OpenMDAO puts each solver on the group that
    holds the cycle in the same way (Gray et al., 2019).
- **The matrix advises the breakdown.** Reordered, it exposes clusters of blocks that mostly talk to each other:
  candidates for one block and one owner.

## 6 · Groups, valves, and the daily design

A **group** owns a branch: its blocks, its people, its releases. The branch mounts on a block of the branch above
it, and that **mount** is the contract between the two groups. It holds:
- the ports the child group must answer;
- the requirements handed to it, with their direction;
- the inputs it is given.

TRI-NETRA's 14 interface rows (`l3_<sid>_interface`) are these mounts today.

The top of the tree is owned the same way:
- the programme perspective is the **programme** branch, today's layer 1;
- the system perspective is the **systems** branch, today's layer 2.

They are groups like any other, and the subsystem groups mount on their blocks. So there is one mechanism from
the top to the bottom, and no special layer. Today's single door (`Satellite ADCS` under case intake, crossing to
`sys_satellite_adcs`) becomes the mount of `systems` on `programme`.

### Every mount is a valve

Requirements flow down the tree and answers come back up, and every mount is a **valve** between the two. The
owner of the branch above sets what flows down through it: the allocation, the bound, the direction. They also
decide what comes back up: the release they accept and pass on.

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
```

- **The system engineer is the main valve,** between the programme above and the subsystems below. Everything
  the ADCS is passes through it, both ways.
- **Every subsystem engineer is the system engineer of their own system.** They set what flows down to their
  nodes, or to a group mounted below them, and decide what goes back up. Their role is the system engineer's,
  one level down, and the same application serves both.
- **It goes to any depth,** to the last break: a node.

The facility groups (OILS rig, HILS rig, test equipment) and the business groups (catalogue, commercial, risk)
mount on the programme's blocks, through the same valve mechanism. Their owner above is the programme manager.

*Said simply:* the design is one system broken down into systems, each with an owner who controls what flows in
and out of it, down to the smallest part.

### Today's design, and the released design

Every valve has two views of what is below it:

| | made from | for | changes |
|---|---|---|---|
| **today's** | the latest signed work of everything below | seeing, every day, how everyone's latest work fits together | whenever anyone below signs or seals |
| **released** | what the owner chose to pass on, signed | what every decision, review and gate refers to | when the owner releases |

At the top, **today's design** is every group's latest sealed release, combined and run. Anyone can open it, every
day, and see how the whole ADCS stands with everyone's latest work. **The released design** is the one the system
engineer releases: the one everyone works from, and the programme decides on.

The same holds at every valve: a subsystem's today is its node engineers' latest signed nodes, and its release is
what its subsystem engineer seals.

### The cycle

1. **The system sets the frame:** where each group mounts, what is allocated to it, the shared cases.
2. **Each subsystem breaks its branch down** and hands nodes to its node engineers, setting the frame of its own
   system.
3. **Node engineers write their nodes,** and sign each day's work.
4. **Each subsystem sees today's group,** tries it inside today's design, and seals a release when it is right.
   It can see the result of its change, but cannot release it past the main valve.
5. **Everyone sees today's design every day,** and discusses what it shows.
6. **The system engineer integrates,** sees exactly where anything breaks, and **releases the design database:**
   the one everyone works from.
7. **The programme decides last,** against its cases, customers, gates and risks. Its decisions are a release of
   the programme's branch, taken into the next cycle through the main valve.

### A parameter belongs to the level that decides it

A parameter is a stated node, owned like any other. Nobody below can change a parameter above; they can only
ask.

| level | owns |
|---|---|
| programme | the margin policy by maturity; the confidence the design is held to (the Monte Carlo percentile a KPI is judged at); the gate criteria; each case's KPIs |
| system | the allocations (the pointing error budget's split, the ADCS unit's mass, power and volume, the momentum envelope), the system margins, the shared cases, the declared loop |
| subsystem | its own design choices (an actuator family, a filter's tuning, a sensor's grade) |

## 7 · What sits in the database, and what in the code

The design is data. The code makes it run, shows it and checks it.

| in the database (changes the design) | in the code (makes it run) |
|---|---|
| blocks: question, ports, behaviour, cases, explanation, history | the pseudocode interpreter, and the translators to Rust, C and MATLAB |
| every relation: the physics, the environment and disturbance models, the device models, the sizing laws | the toolbox the methods may call, with units: vector and quaternion maths, frames and time, integrators, special functions, and published models (DTM2020, IGRF, DE440 reading) |
| **the flight software's algorithms**: estimation, guidance, control, detumble and Sun-acquisition laws, allocation, mode management, FDIR, the drivers' conversions; its parameters, its mode list, its tables (such as the IGRF coefficients) | **the flight software's runtime** (section 7.1): the hardware abstraction, the C interface, the tick and scheduler, the configuration blob, the boot and the targets |
| wires, mounts and closures | the design-graph engine: order, run, iterate loops, refuse and block by name |
| stated values, ranges, maturities, states | the time engine's core: the integrator, the step order, the recorder, the metrics |
| the catalogue: parts, products, algorithms, components, modes, families, classes | **the rigs' software**: the soft OILS emulator and byte link, the OILS and HILS rig host, the device emulation channels, timing and deadline measurement |
| loops declared on their blocks | range verdicts, tornadoes, sweeps, campaigns |
| cases, their variations, and the results kept with their design | the pages: block page, tree, N2, map, charts (`adcs-plot`) |
| sign-offs, seals, versions; the flight images built from a released design | the checks, signing and verifying; one library that reads and writes every file, in the installed application and in the page |

**Generated, never edited.** Everything the time engine, the flight software and the MATLAB twin compute is
generated from the database by the translators. That covers `adcs-physics`, the device and environment models,
the flight algorithms in C and in Rust, and the twin's functions. Each generated file says so in its header, and
the parity tests hold it equal to the interpreter.

### 7.1 · The flight software: algorithms in the database, the runtime in code

The flight software is part of the design. What it decides, and how, is written by the groups that own it:
- `nav`: estimation, the attitude filter, sensor processing;
- `gdn`: guidance and mode management;
- `ctl`: control, detumble and Sun-acquisition laws, allocation;
- `fdir`: fault detection, isolation and recovery;
- `act` and `sens`: the drivers' conversions (counts to SI, commands to duty);
- `fsw`: the OBC's budgets, the timing, the parameters and their table.

Today these are the ten pseudocode files in `fsw/pseudocode/` and the parameters in `fsw/params/params.toml`.
They become nodes, and the C (`fsw/src`) and Rust (`fsw-rs/src`) algorithm code is generated from them.

Only what the computer needs in order to run the algorithms stays code:

| stays code | why it is not design |
|---|---|
| the hardware abstraction (`adcs_hal.h`, `hal.rs`) and the register-level bus access | it is the boundary to a particular board, the same for every design |
| the C interface and its Rust export (`adcs_fsw.h`, `cabi.rs`, `adcs-fsw-abi`) | the contract between the flight software and the engine, the emulator and the boards |
| the tick, the scheduler, the memory layout (no `malloc`), the configuration blob's format and CRC | how the software runs, not what it decides |
| the targets: POSIX for SILS, QEMU Cortex-M (`fsw/targets/qemu-mps2`) for soft OILS, the boards for OILS and HILS | build and boot |
| the byte link (`fsw/targets/link`), the soft OILS emulator, the rig host (`tools/engine_oils.py`) | the test bench, which flies any design |
| the maths library the algorithms call (`01_math`) | the toolbox: a kind of maths, reviewed as code (the onboard time, frames and field, `02_time_frames_models`, are env's published models since S7.3: design, generated) |

**A flight image is built from a design.** When the system engineer releases a design, the flight build
generates the algorithm code from it, compiles it with the runtime for each target, and checks it on the
nodes' test vectors and against the interpreter. It then seals the images into the released design
(`docs/OPERATING_2_0.md`, W8). Today's design can be built the same way, to try it. A result from soft OILS,
OILS or HILS names the image it flew, and the image names its design.

**The design never passes through the code.** It is written, checked, combined, signed and published in the
application, by the people who own it. The checks the application runs are code, the same code the CLI runs, so a
design passes the same checks wherever it is opened. The decisions are data.

Only three kinds of change need the code, and each goes to the developer as a request:

- **a new kind of maths:** a toolbox function a method may call, such as DTM2020 density or an IGRF field
  evaluation;
- **a new kind of thing:** a port type, a behaviour, a picture, a check; a new flight-software target (a board),
  a new rig channel or interface kind;
- **a fault** in the engine's core, the flight software's runtime, a translator or the application.

A new use of maths, a new node, a new group, a new wire, a deeper breakdown, a new requirement, a new catalogue
part, a new case, a change to a flight algorithm or a flight parameter, or a new mode is data, and never waits for
a developer.

## 8 · The ADCS, broken down

This is the tree as it stands, and where it is proposed to go one level deeper. The tree itself is held by
`spec/` and `design/groups.toml` in the repository until 2.0.0, and by the programme and systems groups' files on
the shared drive after it. This section is a picture of it. When the two disagree, the files are right and this
page is stale.

### Today: 765 nodes in 20 groups

    TRI-NETRA ADCS programme
    ├─ Layer 1 · the company (133 rows)
    │   case intake · catalogue (products, design runs, case matching) · commercial (quote, cost
    │   to deliver, margin) · order lifecycle (enquiry → quote → PO → built → calibrated → OILS →
    │   HILS → certificate → delivery) · test facility (SILS capacity, OILS rig, magnetic field
    │   simulator, air-bearing, stimulators, actuator stands, rig safety) · standards & compliance
    │   (ECSS-E-ST-60-30C, -60-10C, -10-02C, -10-03C, -40C, Q-ST-80C) · supply chain · heritage ·
    │   risk management (register, beliefs and versions, open risk by area, conclusion)
    ├─ Layer 2 · the satellite's ADCS (194 rows)
    │   pointing service the customer needs (22 KPIs: APE, AKE, RPE, PDE, slew and settling time,
    │   rates …, each a required and an achieved group) · ADCS configuration (hardware fitted) ·
    │   mission and orbit · the satellite as the ADCS sees it (mass properties, surfaces, magnetic
    │   cleanliness, flexible modes, resources offered) · ADCS subsystems (disturbance torques,
    │   sensors, estimation, magnetic actuation, reaction wheels, fluid momentum rings, RCS,
    │   control and allocation, pointing error budget, modes and FDIR, flight software, unit
    │   budgets) · verification (coverage, campaign evidence, OILS and HILS rig needs)
    ├─ Layer 3 · 14 subsystem layers (368 rows) and the closure addition (39 rows)
    └─ 31 rows added from the code by the carry-over

Each customer is a **case**, not a branch: it enters through the one door. The reference cases are `ais_3u` (AIS,
10° pointing), `ais_img_3u` (AIS and imaging, 0.01° pointing), `ref_c2_150kg` and `ref_c3_12u`.

### Where each group mounts

Today each group holds every row of one discipline, wherever it sits in the layers (`docs/RELEASE_PLAN.md` §4).
On the block model each group mounts on the block it answers, and the tree becomes one tree. `case` is dissolved:
- its case-intake rows go to `programme`;
- its mission requirements and hardware fitted go to `systems`;
- each customer is a case file.

| group | mounts on | through today's |
|---|---|---|
| **programme** (new) | the root | layer 1's branches |
| **systems** (new) | programme · *Satellite ADCS* | the one door, `sys_satellite_adcs`; layer 2's branches |
| dyn · Satellite dynamics | systems · *Satellite as the ADCS sees it* | rows s1–s4 |
| env · Environment and disturbance torques | systems · *Orbit*, *Environment along the orbit* and *Disturbance torques* | rows m2, m3; `l3_dist_interface` |
| sens · Sensors | systems · *Attitude sensors* | `l3_sens_interface` |
| nav · Navigation and attitude estimation | systems · *Attitude estimation* | `l3_est_interface` |
| gdn · Guidance and mode management | systems · *Modes and FDIR* | `l3_modes_interface` |
| fdir · FDIR | systems · *Modes and FDIR*, beside gdn | its own rows (detection, isolation, recovery) |
| ctl · Controller and allocation | systems · *Control and allocation* | `l3_ctl_interface` |
| act · Actuators | systems · *Magnetic actuation*, *Reaction wheels*, *Fluid momentum rings*, *Reaction control thrusters* | `l3_mtq/rw/fmr/rcs_interface` |
| fsw · Flight software and OBC | systems · *Flight software and OBC interfaces* | `l3_fsw_interface` |
| design · Design loop | systems · *ADCS unit budgets*, *Resources offered to the ADCS*; programme · *Design runs*, *Case matching* | `l3_budget_interface`; rows s5, ct2, ct3 |
| pnt · Pointing error budget | systems · *Pointing error budget* | `l3_pnt_interface` |
| kpi · KPI closures | systems · *Pointing service the customer needs* | the closure layer's interface row |
| vv · Verification and standards | systems · *Verification*; programme · *Standards & compliance* | rows v1–v2, st1–st7 |
| oils · OILS rig | systems · *OILS rig needs*; programme · *Test facility* | `l3_oils_interface` |
| hils · HILS rig | systems · *HILS rig needs* | `l3_hils_interface` |
| lab · Test equipment | programme · *Test facility* | rows fa3–fa8 |
| catalogue · Catalogue and heritage | programme · *Catalogue*, *Heritage* | rows ct1, hr1 |
| business · Commercial, orders and supply | programme · *Commercial*, *Order lifecycle*, *Supply chain* | rows cm1–cm3, od1–od2, su1–su3 |
| risk · Risk management | programme · *Risk management* | rows rk1–rk4 |

The table is held as data in `design/tree_2_0.toml`, with the block ids; the conversion (S3) reads it. A group
that mounts in two places owns two branches. Each mount has its own contract, and both are seen by the
owner above each.

### Proposed: one level deeper

The next level each group may open, in the order an ADCS product tree usually goes from subsystem to equipment.
Each is an **open** block until its group's subsystem engineer breaks it down, and none computes until it does.
A group may break down differently; its subsystem engineer decides.

| group | proposed children |
|---|---|
| env | atmospheric density (DTM2020) · magnetic field (IGRF) · Sun and eclipse · gravity gradient · aerodynamic torque · solar and Earth radiation pressure · residual magnetic torque |
| dyn | mass properties · flexible modes · magnetic cleanliness · surfaces and offsets |
| sens | magnetometer · Sun sensors · star tracker · gyro · GNSS receiver · Earth sensor |
| nav | sensor processing · attitude filter (MEKF) · on-board orbit propagation · knowledge budget |
| gdn | mode logic · guidance profiles (nadir, Sun, target, slew) · referencing |
| fdir | detection · isolation · recovery · safe mode |
| ctl | detumble laws · pointing controllers · momentum management · allocation |
| act | magnetorquers · reaction wheels · CMG and VSCMG · fluid momentum rings and their pump · RCS thrusters and feed |
| fsw | OBC and timing · hardware interfaces (HAL) · data handling · scheduling |
| design | demand survey · actuator sizing · product assembly · budgets |
| pnt | knowledge error · control error · stability and jitter · thermal distortion |
| kpi | the 22 KPIs, each a required, an analysis and an evidence closure |
| vv | verification coverage · campaign evidence · standards close-out |
| oils, hils, lab | rig host · emulation channels · stimulators · actuator stands · safety and power |
| catalogue | products · parts by family · algorithms · components · classes and coverage |
| business | quote · cost to deliver · margin and terms · orders · supply |
| risk | register · beliefs and versions · open risk by area · conclusion |

## 9 · Where this model breaks

- **Breaking down well is the engineer's skill.** The tool can say when to stop; it cannot say what decides a
  question. That is why the subsystem engineer and the system engineer both see every breakdown.
- **Moving a branch moves everyone's work.** A change to a mount is the system engineer's, and every group it
  reaches sees it in a preview before it is released (`docs/OPERATING_2_0.md`, W7).
- **A breakdown can move a value its cases never tested.** Children that reproduce every case of the node they
  replace may still answer differently between those cases. The impact list shows every value of another group
  that moved, so the change is seen, not assumed away.
- **Unlimited depth invites over-breaking.** A block that is one formula should stay one block. The stop rule in
  section 2 is the guard.
- **A block without a method cannot show its curve.** Before 2.0.0 ships, every relation still in compiled code
  is written as a method and proven equal, so none is left (`docs/PLAN_2_0.md`).
- **Generated flight code is only as good as its translator.** The C and Rust translations are held to the
  interpreter on every node's vectors, and the flight images to the hand-written 1.0.0 software on every scenario,
  before anything flies. A change to a translator is a code change, and needs a new application release (W15).
- **A flight image needs a compiler.** The flight build carries its toolchains; a computer without it can try a
  design in SILS through the interpreter, but cannot build an image for a board.
- **Two engines.** The design graph answers rows; the time engine answers flights. They meet only through mapped
  inputs (a node's value becomes a case input) and evidence metrics (a flight's metric becomes a closure's
  achieved value). A case input with no node stays a case value until a group claims it.
- **One number per row runs deep in the engine.** Choices, lists, tables and time series as real ports are engine
  work, not only a new page.

## 10 · References

| part | rests on |
|---|---|
| recursion: a system's elements are systems, and the same processes apply at every level | ISO/IEC/IEEE 15288:2023; [SEBoK, Recursion](https://sebokwiki.org/wiki/Recursion_(glossary)) |
| perspectives, not floors | the Arcadia method: operational, system, logical and physical perspectives ([Roques, *Systems Architecture Modeling with the Arcadia Method*](https://www.iste.co.uk/book.php?id=1261)) |
| blocks, ports, connections, requirements, verification cases, and an API to exchange them | OMG SysML v2 and the Systems Modeling API & Services, adopted 2025 ([OMG](https://www.omg.org/news/releases/pr2025/07-21-25.htm)) |
| N2 as a design structure matrix; partitioning, clustering | Eppinger and Browning, *Design Structure Matrix Methods and Applications*, MIT Press, 2012 ([MIT](https://stuff.mit.edu/people/eppinger/SDE-MIT/DSM_Book.html)) |
| feedback below the diagonal; a solver on the group that holds the cycle | Gray et al., *OpenMDAO*, Structural and Multidisciplinary Optimization 59, 2019 ([U. Michigan](https://mdolab.engin.umich.edu/bibliography/Gray2019a)); [OpenMDAO N2 details](https://openmdao.org/docs/latest/features/model_visualization/n2_details/n2_details.html) |
| ranges, not blanks; decide when the information exists | Sobek, Ward and Liker, *Toyota's principles of set-based concurrent engineering*, MIT Sloan Management Review, 1999 ([MIT SMR](https://sloanreview.mit.edu/article/toyotas-principles-of-setbased-concurrent-engineering)) |
| margins by maturity: estimated, calculated, actual | ANSI/AIAA S-120A-2015, mass properties control for space systems ([NASA NTRS](https://ntrs.nasa.gov/citations/20130014265)) |
| which input moves a result: tornado, then variance-based indices | Saltelli et al., *Global Sensitivity Analysis: The Primer*, Wiley, 2008 ([overview](https://en.wikipedia.org/wiki/Variance-based_sensitivity_analysis)) |
| pointing requirements and their indices (APE, AKE, RPE, PDE) | ECSS-E-ST-60-10C, *Control performance*; ESA pointing error engineering handbook ESSB-HB-E-003 |
| a model's ports described as data any tool can read | FMI 3.0 ([press release](https://fmi-standard.org/assets/FMI_3.0_Press_Release.pdf)) |
| the document is the program: change an input and what depends on it re-runs | reactive notebooks such as marimo ([README](https://fossies.org/linux/marimo/README.md)) |
| one file as the application's document | SQLite as an application file format ([sqlite.org](https://sqlite.org/appfileformat.html)) |
