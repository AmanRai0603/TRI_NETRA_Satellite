# Languages: who does what

Owner: Agastya. One algorithm is written once as **pseudocode**, then flown in
**embedded C** and **Rust**. The dynamics run in **Rust**. **Python** drives
everything and writes the reports. **MATLAB** stays the design twin of the SILS.

```
                      fsw/pseudocode/*.md   (the contract: 00 conventions … 09 drivers/HAL)
                        │                      │
            fsw/  (C99, OBC build)        fsw-rs/  (Rust, no_std)
            adcs_fsw.h + adcs_hal.h       Hal trait; feature `cabi` = the same adcs_fsw.h symbols
                        │                      │
                        └──── same params blob (adcs-fswcfg/1, fsw/params/params.toml
                              → tools/gen_fsw_params.py → C + Rust) and same bytes ────┐
                                                                                       │
   engine/ (Rust)  adcs-sim-core  plant, orbit, field, Sun/Moon, torques, sensor and    │
                   (no_std)       actuator models, device byte codecs                  │
                   adcs-fsw-abi   the emulated buses; extern "C" adcs_hal_*; links fsw/ ◄┘
                   adcs-sim       case + scenario + product → config → loop → metrics → adcs-rec/1
                   adcs-cli       `adcs run | params | parity`
                        │
   tools/ (Python)  engine.py     build, run, Monte Carlo, C-vs-Rust parity, engine-vs-MATLAB ledger
                    report.py     figures + results/index.html from any adcs-rec/1 run
                    gen_fsw_params.py, export_catalogue.py, run_matrix.py, pack_matlab.py
                        │
   matlab_sils/ (MATLAB/Octave)   the SILS twin with POP v51 in the loop; design workflow, trades,
                                  sizing, the solution pipeline; the reference the engine is measured against
```

## Why each language sits where it does

| layer | language | why |
|---|---|---|
| flight software on the OBC | **C99** (`fsw/`) | every OBC toolchain has a C compiler; static state, no `malloc`, `time`, `rand` or recursion (`make check`) |
| the same flight software | **Rust** (`fsw-rs/`) | memory safety with no runtime; `no_std`, no heap; builds for `thumbv7em-none-eabihf`; exports the C ABI so it drops in where `libadcs_fsw.a` goes |
| plant, environment, orbit, devices | **Rust** (`engine/`) | fast (≈80–90 000× real time for coils-only runs, ≈85× with a star tracker catalogue search), deterministic (counter-based randomness, pure-Rust libm), `no_std` core that also runs on a rig or in a browser |
| orchestration, reports, generators | **Python** (`tools/`) | process pools, JSON/CSV, matplotlib, HTML |
| design twin | **MATLAB / Octave** (`matlab_sils/`) | where the algorithms were designed and traded; POP v51 in the loop |

## How agreement is measured (`fsw/twin_map.toml`)

- **C = Rust flight software: bit-identical.** `adcs parity <scenario>` runs the same closed loop
  with each build behind the same bytes and compares every recorded sample; on the host both
  builds use the platform libm and produce identical trajectories (all scenarios tried; engine test
  `run_twice_identical_and_c_equals_rust`). The unit suites pin the same PWM words.
- **Flight software vs the MATLAB twin: test vectors.** IGRF, Sun model, QUEST/TRIAD/MEKF, the
  laws and the RCS duty are checked against numbers the twin printed (`fsw/tests`, `fsw-rs/tests`).
- **Engine vs MATLAB twin: a ledger.** The engine's orbit, ephemeris and density are analytic
  (J2–J6, Montenbruck–Gill, exponential) where the twin runs POP/DE440/DTM2020, and the random
  streams differ, so single runs are compared by metric ratio and verdict agreement in
  `results/ENGINE_PARITY.md`; Monte Carlo on both sides compares distributions.

## Commands

```bash
python3 tools/engine.py build                 # C tests + strict check, Rust FSW tests, engine tests, release builds
python3 tools/engine.py run                   # every scenario on the engine (C flight software)
python3 tools/engine.py run slew_img --fsw rust
python3 tools/engine.py mc fine_hold_img --seeds 50
python3 tools/engine.py fsw-parity            # C vs Rust flight software, all scenarios
python3 tools/engine.py twin-parity           # engine vs MATLAB twin -> results/ENGINE_PARITY.md

engine/target/release/adcs run detumble_ais --fsw c --seed 3
engine/target/release/adcs params mission_img --out mission_img.fswcfg   # the blob an OBC boots from
```

The flight-software build on its own:

```bash
make -C fsw test check                                  # C: unit tests, -Werror, forbidden-call scan
cd fsw-rs && cargo test --release                       # Rust: the same vectors
cargo build --release --no-default-features --features cabi --target thumbv7em-none-eabihf
```
