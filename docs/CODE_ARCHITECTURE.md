# The code's architecture

> **Answer first.** The repository holds code only. The code falls into eight parts:
> - the toolbox;
> - the library;
> - the translators;
> - the time engine;
> - the flight software's runtime;
> - the rigs;
> - the application;
> - the tools.
>
> Each may call only the parts below it in the table of §2. Everything that is design (relations, flight
> algorithms, parameters, tables, the catalogue, cases) lives in the database. It reaches the code only as
> **generated** files, which say so in their header and are never edited by hand.
>
> **Kind:** reference · **For:** the developer side · **Status:** proposed for 2.0.0 (S1). Sections marked *today*
> describe 1.0.0 as it stands; the rest is the target the phases of `docs/PLAN_2_0.md` reach.

The design's architecture (blocks, valves, today's design) is `docs/SYSTEM_MODEL.md`. How people work is
`docs/OPERATING_2_0.md`.

## 1 · The boundary: what is code and what is design

**The rule.** A relation, a flight algorithm, a parameter value or a table is design. In the repository it may
appear only:
- in a generated file (header: *Generated … do not edit*);
- in the toolbox, when it is a kind of maths and not a use of it;
- in a test fixture or the regression copy of a released design.

Anywhere else it is a defect, found by the boundary check (S7, `tools/check_all.py`).

| design (database) | code (repository) |
|---|---|
| physics, environment, disturbance and device relations; sizing laws | the toolbox the methods call: vectors, quaternions, frames and time, integrators, special functions; DTM2020, IGRF, DE440 reading |
| flight algorithms (estimation, guidance, control, step laws, allocation, modes, FDIR, drivers' conversions) | the flight software's runtime: HAL, C interface, tick and scheduler, configuration blob format and CRC, targets |
| flight parameters, the mode list, tables (IGRF coefficients) | the flight build: generate, compile, check, seal |
| catalogue, cases and variations, KPIs, dispersions, campaigns | the time engine's core: integrator, step order, recorder, metrics, campaigns |
| wires, mounts, closures, loops; state, maturity, range | the design-graph engine, the health map, range verdicts, tornadoes |
| signatures, seals, versions; flight images of released designs | signing and checking; the library; the application; the CLI, Python package and tools |

**What a design says it needs.** Every design names:
- the toolbox it was built for (`meta.toolbox`, today `trinetra-toolbox/2`);
- the oldest application that can run it (`meta.needs_application`).

The engine, the app and the Python package refuse a design they cannot run, by name
(`engine/crates/adcs-sim/src/source.rs`, `cannot_run`; `python/trinetra_adcs/design.py`). Raise the toolbox
version when the functions a design may call change. Every run records the design's fingerprint, version and
toolbox, and is stale once another design is in use (`adcs results stale`).

## 2 · The parts

Each part may call the parts in the rows below it, and no part above it.

| # | part | today (1.0.0) | in 2.0.0 |
|---|---|---|---|
| 8 | **application** | `engine/crates/trinetra-app` (desktop: Fly, Runs, Design); `design/pages/*.template.html` with `design/js/*_app.js` (Files, Node, Group, test apps) | `trinetra-app` grown into the one application, installed, plus `apps/TRI-NETRA.html` as a page; five workspaces; the flight build |
| 7 | **CLI, Python package, tools** | `engine/crates/adcs-cli` (`adcs`), `python/trinetra_adcs`, `tools/*.py` | the same; they read the design only through the library |
| 6 | **rigs** | the byte link (`fsw/targets/link`), soft OILS on QEMU (`fsw/targets/qemu-mps2`), the rig host (`tools/engine_oils.py`) | the same, plus Renode (S15) and the board rig host (S16) |
| 5 | **flight software's runtime** | `fsw/include/adcs_hal.h`, `adcs_fsw.h`; `fsw-rs/src/hal.rs`, `cabi.rs`; `engine/crates/adcs-fsw-abi`; `fsw/targets/posix`; the configuration blob (`tools/fswcfg.py`) | the same; the algorithm sources beside it (`fsw/src`, `fsw-rs/src`) are generated (S6) |
| 4 | **time engine** | `adcs-sim-core` (plant, no_std), `adcs-sim` (cases, runs, store), `adcs-pop` (orbit), `adcs-design` (sizing), `adcs-plot` (figures) | the core stays code; the models and the sizing laws are generated from the design (S7) |
| 3 | **translators** | `design/js/pcode_gen.js` (pseudocode → Rust, MATLAB), driven by `tools/pcode.py` and `tools/groupcode.py` | + C (S5); used by the flight build and the engine build |
| 2 | **library** | `engine/crates/trinetra-design` (read-only reader); `tools/tndb.py`; `design/js/tnfile.js`, `node_model.js`, `structure.js`, `release.js` | `trinetra-design` grown to read, write and check every file kind, run the interpreter, build today's design, sign (S2); native and WebAssembly; the JavaScript and Python versions become its test oracles, then retire |
| 1 | **toolbox** | `adcs-physics` (generated relations plus their maths), `fsw/pseudocode/01_math.pc`, `02_time_frames_models.pc`; the published models in `adcs-pop` (DTM2020, JB2008, IGRF, DE440) | the maths only; the relations move out to the design (S3, S7) |

**The library today** (`engine/crates/trinetra-design`, with its command `tndb`):

| module | does | held to |
|---|---|---|
| `lib.rs` | the schema compiled in; open, check, read rows | `design/ddl.sql` (tools/tndb.py) |
| `write` | create a file whole; upgrade an older one in place, a copy kept, adding only tables | tests/test_format2.py: all 765 nodes and 20 groups, nothing dropped |
| `content` | the canonical content (`trinetra-content/1`) and the hash a signature covers | `design/js/tncontent.js`, by the cross-language test `content_cross` |
| `keys` | Ed25519 keys, the PBKDF2 + AES-GCM lock, signing, the registry check | `design/js/tnkeys.js` (Web Crypto), by `keys_cross` |
| `chain` | sign a file; check every signature on it against the group file's registry, and that the content is still what was signed | its tests |
| `checks` | `tools/group.py check` and `verify`, `tools/release.py check`, word for word | tests/test_release.py on 40 sealed releases, 8 broken ones and 8 damaged folders |
| `compare` | any two files of one kind, table by table | its tests |
| `node_rules` | the node app's live checks (N02 … D07): a node file read as the app reads it, every rule with its words, the pseudocode through `trinetra-pcode`; `tndb check-node` | `design/js/node_model.js`, by `node_rules`: all 765 carried nodes and 547 broken copies |

**In the page**, signing uses the browser's own Web Crypto (`design/js/tnkeys.js`) and the content encoding
`design/js/tncontent.js`. These are deliberate twins of the library's, not copies to retire: the browser's
crypto keeps a private key out of reach of the page's code. The cross-language tests hold them to the library
byte for byte.

**Generated, today:**
- `adcs-physics`, from `spec/physics/*.pc`;
- `adcs-groups` and `adcs-groups-wasm`, from the nodes;
- `fsw/src/adcs_params.c`, `fsw/include/adcs_params.h`, `fsw-rs/src/params.rs`, `fsw-rs/src/igrf13.rs` (the engine's
  plant field model) and `fsw/pseudocode/02_igrf13.pc`, from `fsw/params/params.toml` and the IGRF coefficients;
- the MATLAB `+groups` package;
- `docs/COMMANDS.md`;
- the pages from their templates;
- `design/pcode_checker.html`.

Each says so in its header, and each `--check` finds a hand edit (`tools/check_all.py`).

## 3 · How a design reaches a number

**Today:**
```text
spec/, catalogue/, scenarios/, campaigns/, trades/ (TOML, CSV)
   └─ tools/export_catalogue.py ─▶ matlab_sils/data/*.json, matlab_sils/cases/*.csv
        ├─ tools/seed_design.py ─▶ design.tndb (a copy: engine_input, design_case)
        └─ adcs-sim reads the files, or the copy through source.rs (TRINETRA_DESIGN)
```

**2.0.0:**
```text
Trinetra Database/groups/*  (the design, written in the application)
   └─ library: today's design or a released design
        ├─ design graph: methods run by the interpreter ─▶ closures, health map
        ├─ engine build: translators ─▶ engine models  ─┐
        ├─ flight build: translators ─▶ flight images ──┼─▶ time engine flies a case ─▶ results (name design and image)
        └─ case inputs from their nodes ────────────────┘                                  └─▶ evidence for closures
```

## 4 · Rules for the code

These are the code's rules; `CONTRIBUTING.md` says how a change is made.

1. **Refuse, never guess.** A missing or out-of-range input is refused by name, never filled with a default.
2. **Never edit a generated file.** Change its source, and regenerate.
3. **The twin moves with the engine, and the C runtime with the Rust,** in the same change.
4. **Every toolbox formula is traced to its source** (`docs/references.toml`).
5. **An expected value never comes from the code under test.**
6. **No design in code** (§1). A relation found in code is moved to the design, or into the toolbox if it is a
   kind of maths.
7. **An application release gives the current released design's answers,** and its flight images' behaviour,
   **unchanged** (W15). A translator change is proven on every node's test vectors before it ships.
8. **A part calls only the parts below it** (§2).
