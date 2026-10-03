# Changing TRI-NETRA ADCS

> **Answer first.** Every kind of change has one place it is made, one command that
> regenerates what follows from it, and one check that proves it. Edit the source, never a
> generated file; run the generator; run `python3 tools/check_all.py`. The table below says
> which is which.
>
> **Kind:** how-to · **For:** whoever changes the code or the data

`python3 tools/trinetra.py explain <command>` says what any command below does before you
run it; `docs/COMMANDS.md` lists them all.

## What to edit, what to run, what proves it

| to change | edit (the source) | then run | the check that proves it |
|---|---|---|---|
| a part, product, algorithm, mode, component or family | `catalogue/**/*.toml` | `python3 tools/export_catalogue.py` | `export_catalogue.py --check`: every JSON is its TOML |
| a scenario, campaign or trade | `scenarios/`, `campaigns/`, `trades/` (`*.toml`) | `python3 tools/export_catalogue.py` | the same, and `tests/inputs.rs`: every shipped scenario builds |
| a new scenario key (the engine reads a new setting) | read it in `engine/crates/adcs-sim/src/config.rs`, add it to `schema.rs` | `ADCS_WRITE_SCHEMA=1 cargo test -p adcs-sim schema_json` (the twin's copy) | an unknown key is refused by engine and twin; `schema_json_is_current` fails until the copy is written |
| a customer case | `matlab_sils/cases/<case>.csv` (adcs-case/1) | `adcs run <scenario> --case ...` | the case readers refuse a bad value by name (engine and twin alike) |
| a flight-software parameter | `fsw/params/params.toml` | `python3 tools/gen_fsw_params.py` | `gen_fsw_params.py --check`; `make -C fsw test`; C = Rust parity |
| a flight-software algorithm | `fsw/pseudocode/` first, then `fsw/src/` (C) and `fsw-rs/src/` (Rust) in the same change | `python3 tools/engine.py build` | `engine.py fsw-parity` (bit-identical C and Rust) |
| the plant, environment or orbit | `engine/crates/adcs-sim-core`, `adcs-pop`, and the MATLAB twin (`matlab_sils/+asils`) in the same change | `python3 tools/engine.py build` | `engine.py twin-parity`; the propagator's Octave suite |
| an engine limit or a refusal | `engine/crates/adcs-sim/src/config.rs` (`check`, `ENGINE_KEYS`, `CASE_NEEDS`) | — | `engine/crates/adcs-sim/tests/inputs.rs` |
| a command, or what it does | the tool, and its entry in `docs/commands.toml` | `python3 tools/trinetra.py docs` | `tests/test_commands.py`: the registry and the tools agree |
| a requirement threshold | `matlab_sils/cases/<case>.csv` (`req.*`) | `python3 tools/rescore.py` | stored runs re-judged; no trajectory changes |
| the design loop | `tools/pipeline.py` and `matlab_sils/data/pipeline/nodes.json` | `python3 tools/pipeline.py` | `python3 tools/verify_nodes.py` |
| the version | `python3 tools/version.py --set X.Y.Z` (never by hand) | — | `version.py --check`: `VERSION`, both Cargo files and the C build id agree |
| the Rust toolchain | `rust-toolchain.toml` (one change of its own; every stored run is then re-flown) | `cargo build --locked` | the engine's and the flight software's tests on the new toolchain |
| a Python package CI installs | `tools/requirements-ci.txt` (pinned) | `python3 -m pip install -r tools/requirements-ci.txt` | `check_all.py` |

## Rules that keep it true

- **The twin moves with the engine.** A change to the physics or to the Monte Carlo draws is
  made in Rust and in MATLAB in the same commit (the Kp → ap table is one table in both,
  and `tests/test_kp2ap.py` holds it so).
- **A refusal is never a substitution.** A value the engine cannot fly is refused with its
  name and range; it is not clamped, defaulted or read as blank.
- **Files are written whole.** Python tools write through `tools/common.py`
  (`write_text`, `write_json`, `atomic_path`), the engine through `adcs_sim::fsio::write`.
- **Generated files are never edited by hand.** Each says so in its header; its `--check`
  finds a hand edit.
- **A result says where it came from.** A run's manifest carries its inputs' fingerprints;
  `adcs results show <run>` prints them.

## Before you push

    python3 tools/check_all.py            # every check, one line each (--octave adds the twin)

CI runs the same command on every pull request.
