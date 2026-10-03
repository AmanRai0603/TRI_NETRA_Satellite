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
   engine/ (Rust)  adcs-design    demand survey + sizing of every option (the +asils/+sizing laws, identical)
                   adcs-pop       the POP v51 port: time scales, frames, EOP, DE440,     │
                                  gravity, tides, radiation, DTM2020/JB2008 drag, integ. │
                   adcs-sim-core  plant, field, torques, sensor and actuator models,     │
                   (no_std)       device byte codecs (+ analytic fallback orbit)        │
                   adcs-fsw-abi   the emulated buses; extern "C" adcs_hal_*; links fsw/ ◄┘
                   adcs-sim       case + scenario + product → config → loop → metrics → adcs-rec/1
                   adcs-cli       `adcs run | params | parity`, --fsw c|rust|obc-posix|qemu|tcp:..
   fsw/targets/    virtual OBC    adcs-link/1: the flight software as a process or as Cortex-M4F
                                  firmware in QEMU, in lockstep with the engine (docs/VIRTUAL_OBC.md)
                        │
   tools/ (Python)  pipeline.py   the design loop: size -> SILS matrix -> assess -> converge -> faults -> select -> dispatch -> MC -> soft OILS
                    vv_report.py  the V&V report (template -> HTML -> PDF)
                    engine.py     build, run, Monte Carlo, C-vs-Rust parity, engine-vs-MATLAB ledger
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
| plant, environment, orbit, devices | **Rust** (`engine/`) | the full POP propagator ported model by model (bit-identical to the MATLAB twin's orbit and environment), ≈4 500× real time with a star tracker and ≈44 000× coils-only on one core; deterministic (counter-based randomness); `no_std` plant core |
| orchestration, reports, generators | **Python** (`tools/`) | process pools, JSON/CSV, matplotlib, HTML |
| design twin | **MATLAB / Octave** (`matlab_sils/`) | where the algorithms were designed and traded; POP v51 in the loop |

## How agreement is measured (`fsw/twin_map.toml`)

- **C = Rust flight software: bit-identical.** `adcs parity <scenario>` runs the same closed loop
  with each build behind the same bytes and compares every recorded sample; on the host both
  builds use the platform libm and produce identical trajectories (all scenarios tried; engine test
  `run_twice_identical_and_c_equals_rust`). The unit suites pin the same PWM words.
- **Flight software vs the MATLAB twin: test vectors.** IGRF, Sun model, QUEST/TRIAD/MEKF, the
  laws and the RCS duty are checked against numbers the twin printed (`fsw/tests`, `fsw-rs/tests`).
- **Engine vs MATLAB twin.** The engine runs the Rust port of POP (`adcs-pop`) in the loop exactly
  as the twin runs the MATLAB POP: orbit, Sun, Moon, shadow, density and field are bit-identical on
  every recorded sample of all 40 scenarios. The random streams differ by design, so single runs
  are compared by metric ratio and verdict agreement in `results/ENGINE_PARITY.md`, and Monte
  Carlo on both sides compares distributions.
- **Virtual OBC: bit-identical.** The flight software as a separate process or as Cortex-M4F
  firmware in QEMU, over adcs-link/1, reproduces the in-process trajectories (35/35 in
  `results/VIRTUAL_OBC.md`).

## Commands

```bash
python3 tools/engine.py build                 # C tests + strict check, Rust FSW tests, engine tests, release builds
python3 tools/engine.py run                   # every scenario on the engine (C flight software)
python3 tools/engine.py run slew_img --fsw rust
python3 tools/engine.py mc fine_hold_img --seeds 50
python3 tools/engine.py fsw-parity            # C vs Rust flight software, all scenarios
python3 tools/engine.py twin-parity           # engine vs MATLAB twin -> results/ENGINE_PARITY.md
python3 tools/engine.py vobc                  # virtual OBC (process + QEMU Cortex-M4) -> results/VIRTUAL_OBC.md
engine/target/release/adcs run mission_img --fsw qemu        # the OBC firmware in QEMU in the loop

engine/target/release/adcs run detumble_ais --fsw c --seed 3
engine/target/release/adcs params mission_img --out mission_img.fswcfg   # the blob an OBC boots from
```

The flight-software build on its own:

```bash
make -C fsw test check                                  # C: unit tests, -Werror, forbidden-call scan
cd fsw-rs && cargo test --release                       # Rust: the same vectors
cargo build --release --no-default-features --features cabi --target thumbv7em-none-eabihf
```
