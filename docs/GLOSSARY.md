# Glossary

> **Answer first.** The words this repository uses, each in one or two plain sentences, with
> where it lives. Read a term here before reading the document that uses it.
>
> **Kind:** reference · **For:** anyone new to the repository

| term | what it means here | where |
|---|---|---|
| **ADCS** | Attitude determination and control system: the sensors, flight software and actuators that know and set which way a satellite points. | the whole repository |
| **case** | One satellite and its requirements in one CSV (`adcs-case/1`): orbit, mass and inertia, surfaces, pointing requirements `req.*`. A blank value means *not stated*. | `matlab_sils/cases/` |
| **scenario** | What the satellite is asked to do and for how long: a start state, a mode schedule, the metrics to judge. | `scenarios/*.toml` → `matlab_sils/data/scenarios/` |
| **campaign** | Many runs of one scenario: Monte Carlo (random dispersions) or edge (each dispersion at its bounds). | `campaigns/` |
| **product**, **family** | A product is a fitted ADCS (parts and algorithms); a family is a kind of ADCS (coils only, coils with fluid loop, with RCS, wheels, CMG, VSCMG). *Solutions* are ours; *benchmarks* are bought actuators sized to the same case, for comparison. | `catalogue/products/`, `catalogue/families.toml` |
| **engine** | The Rust simulator: plant, environment, orbit (the POP port), device emulators, with the flight software in the loop. `adcs` is its command line. | `engine/` |
| **flight software (FSW)** | The on-board software, written twice from one pseudocode: C99 (`fsw/`) and Rust `no_std` (`fsw-rs/`). The two are required to give bit-identical trajectories. | `fsw/`, `fsw-rs/`, `fsw/pseudocode/` |
| **twin** | The MATLAB / GNU Octave implementation of the same SILS (`asils`), kept in step with the engine; a difference between them is a finding. | `matlab_sils/` |
| **POP** | The precision orbit propagator: gravity field, Sun and Moon, drag, solar pressure. MATLAB original, Rust port (`adcs-pop`), bit-identical. | `matlab_sils/pop/`, `engine/crates/adcs-pop/` |
| **SILS, PIL, OILS, HILS** | Software-, processor-, on-board-computer- and hardware-in-the-loop simulation: the flight software on a PC, on a processor, on the real OBC, and with real hardware. | `docs/OILS_HILS.md` |
| **soft OILS**, **virtual OBC** | The flight software compiled for a Cortex-M4F and run in QEMU in lockstep with the engine, with its exact execution time and bus latency. | `fsw/targets/`, `docs/SOFT_OILS.md` |
| **parity** | Two implementations flown on the same inputs and compared: C against Rust, engine against twin, in-process against QEMU. | `adcs parity`, `results/ENGINE_PARITY.md` |
| **design loop** | Size → fly every mode and option → assess → converge → select → dispatch → Monte Carlo → soft OILS: from a case to a chosen ADCS. | `tools/pipeline.py`, `docs/DESIGN_LOOP.md` |
| **node** | One step of the design loop, with its inputs, outputs, parameters and rules; `verify_nodes.py` recomputes each one's decision from its stored inputs. | `matlab_sils/data/pipeline/nodes.json` |
| **dispatch** | The selected ADCS's flight configuration: the parameter blob an OBC boots from, checked with C and Rust. | `dist/dispatch/` |
| **parameter blob** (`adcs-fswcfg/1`) | Every flight-software parameter in table order with a CRC-32, generated from one definition. | `fsw/params/params.toml` |
| **APE, AKE, RPE, RKS** | Absolute pointing error, absolute knowledge error, relative pointing error, rate stability: the pointing metrics judged against `req.*`. | `docs/NODES.md` |
| **B-dot**, **Sun spin**, **nadir**, **Sun referencing** | Mission modes: detumbling on the magnetic field's rate, spinning about the Sun line, pointing at the Earth, holding a Sun-referenced attitude. | `catalogue/modes/` |
| **Kp**, **ap** | Geomagnetic activity indices (Kp 0–9, ap 0–400) that drive the atmosphere's density; one is turned into the other by the standard table. | `adcs_pop::spaceweather::kp2ap` |
| **Floquet multipliers** | The stability certificate of a periodic control loop: every multiplier inside the unit circle means the magnetic pointing law is stable over the orbit. | `tools/floquet.py` |
| **refusal** | An input the engine cannot fly (text for a number, an orbit outside LEO, a limit exceeded) is refused with its name and range, never clamped or guessed. | `engine/crates/adcs-sim/src/config.rs` |
| **provenance** | What a run records about itself: when, which engine, and fingerprints of the case, the scenario, the overrides and the seed. | `adcs results show` |
| **store** | Where runs are kept: `matlab_sils/store` in a checkout, `~/.trinetra/store` for a release, or `$TRINETRA_STORE`. | `adcs_sim::store_root` |
| **kept inputs** | The case and scenario files a run flew, copied once by fingerprint into the store's `inputs/`, so the run can be flown again exactly after either has changed. | `adcs results show` |
| **pinned**, **thinned** | A pinned run is kept whole; a thinned run has lost its time series but keeps its verdicts and provenance. | `adcs results pin`, `thin` |
| **`.trinetra` file** | One run as one zip file, to send to someone. | `adcs results export` |
| **kit** | A release download: the programs beside exactly the data they read, with `VERSION`. | `tools/kit.py` |
| **evidence debt** | What is not yet confirmed by a person, flown in the flight code, or agreed by the twin: the headline of `python3 tools/trinetra.py status`. | `tools/trinetra.py` |
| **UNCONFIRMED** | Written on anything a person has not yet checked and signed; nothing is marked confirmed by a tool. | the data files |
| **generated file** | Written by a command from a definition, never edited by hand; `python3 tools/trinetra.py why <file>` names the command. | `.gitattributes` |

## The words of 2.0.0

These come with the 2.0.0 model (`docs/SYSTEM_MODEL.md`, `docs/OPERATING_2_0.md`). Until the switch-over they
describe the target; from it, they replace lead, author, checker, stage owner, owner team and user everywhere.

| term | what it means here | where |
|---|---|---|
| **block** (node) | The one kind of element of the design: a question, its ports, one behaviour, its cases, its explanation, its history. It contains other blocks to any depth. | `docs/SYSTEM_MODEL.md` §2–3 |
| **behaviour** | How a block answers: *method* (its pseudocode), *children*, *stated* (a value with its source), *lookup* (a table), or *open* (not decided). *Built-in* (still compiled code, by node id) is temporary and gone before 2.0.0 ships. | `docs/SYSTEM_MODEL.md` §2 |
| **port** | An input or output of a block, with its type, unit, range and the reasons for the range, and its state, maturity and direction. | `docs/SYSTEM_MODEL.md` §3–4 |
| **state** | *decided*, *allocated* (a bound handed to a child), *open* (to be decided below, with an owner and a gate) or *achieved* (computed from below). | `docs/SYSTEM_MODEL.md` §4 |
| **maturity** | Whether a value is *estimated*, *calculated* or *measured*; a closure demands the margin its least mature input needs. | `docs/SYSTEM_MODEL.md` §4 |
| **range verdict** | What a closure says over an open value's range: it closes for all of it, for part of it (with the crossing), or for none. | `docs/SYSTEM_MODEL.md` §4 |
| **tornado** | A closure's open inputs ranked by how far each moves its margin, one at a time. | `docs/SYSTEM_MODEL.md` §4 |
| **closure** | A requirement against its achieved value, with a direction and a margin; by analysis (the design graph) or by evidence (a campaign). | `docs/SYSTEM_MODEL.md` §1 |
| **N2** | The matrix of a block's children: outputs along a row, inputs down a column; a mark below the diagonal is a loop. | `docs/SYSTEM_MODEL.md` §5 |
| **perspective** | Programme, system or subsystem: the kind of question a branch answers; a tag, not a floor (today's layers 1, 2, 3). | `docs/SYSTEM_MODEL.md` §2 |
| **mount**, **valve** | Where a group's branch hangs on a block above it, and the contract between the two; its owner above controls what flows down and what passes back up. | `docs/SYSTEM_MODEL.md` §6, `design/tree_2_0.toml` |
| **programme manager** | The valve for the programme side: cases, KPIs, gates, risks, margin policy; the people, their keys, the drive. | `docs/OPERATING_2_0.md` §2 |
| **system engineer** | The main valve: mounts, allocations, budgets, shared cases; the only one who releases the design. | `docs/OPERATING_2_0.md` §2 |
| **subsystem engineer** | The owner of a group's branch, and the system engineer of their own system; seals the group's releases. Replaces *lead*. | `docs/OPERATING_2_0.md` §2 |
| **node engineer** | The owner of nodes at the last break; signs each day's revisions. Replaces *author*. | `docs/OPERATING_2_0.md` §2 |
| **checker** | Someone other than the node engineer who signs a revision as checked, where the group asks for it. | `docs/OPERATING_2_0.md` §2 |
| **deputy** | Signs for the programme manager, the system engineer or a subsystem engineer when they are away, with their own key. | `docs/OPERATING_2_0.md` §2 |
| **today's design** | Every group's latest sealed release, combined and run, rebuilt whenever anyone opens the application: for seeing and discussing. | `docs/OPERATING_2_0.md` §5 |
| **released design** | The design the system engineer released and signed: for deciding. Versioned year.month.sequence. | `docs/OPERATING_2_0.md` §11, W8 |
| **health map** | Every node, group and closure coloured closes, tight, fails, refused, blocked, open or unproven, rolled up valve by valve, with trace to cause. | `docs/OPERATING_2_0.md` §6 |
| **toolbox** | The maths a method may call (vectors, quaternions, frames and time, published models): code, versioned; a design names the toolbox it was built for. | `docs/CODE_ARCHITECTURE.md` §1 |
| **flight build**, **flight image** | The step that generates the flight algorithms from a design, compiles them with the runtime for a target and checks them; its sealed result, named after its design. | `docs/SYSTEM_MODEL.md` §7.1 |
| **runtime** (flight software's) | The part of the flight software that stays code: HAL, C interface, scheduler, configuration blob, targets. | `docs/SYSTEM_MODEL.md` §7.1 |
| **transcription** | A method or value an assistant wrote from a cited source or from code; it counts only once a person has checked and signed it. | `design/rules_2_0.toml` R04 |
