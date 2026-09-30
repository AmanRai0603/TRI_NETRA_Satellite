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
| **pinned**, **thinned** | A pinned run is kept whole; a thinned run has lost its time series but keeps its verdicts and provenance. | `adcs results pin`, `thin` |
| **`.trinetra` file** | One run as one zip file, to send to someone. | `adcs results export` |
| **kit** | A release download: the programs beside exactly the data they read, with `VERSION`. | `tools/kit.py` |
| **evidence debt** | What is not yet confirmed by a person, flown in the flight code, or agreed by the twin: the headline of `python3 tools/trinetra.py status`. | `tools/trinetra.py` |
| **UNCONFIRMED** | Written on anything a person has not yet checked and signed; nothing is marked confirmed by a tool. | the data files |
| **generated file** | Written by a command from a definition, never edited by hand; `python3 tools/trinetra.py why <file>` names the command. | `.gitattributes` |
