# TRI-NETRA ADCS: the upgrade plan before the first release

> **Answer first.** Part A is the owner's eleven phases, numbered as given (the review's
> phases 1–4, then phases 5–11), each with what is already done in TRI-NETRA and what is
> not. Part B is the extra work TRI-NETRA needs that those phases do not name: flight
> software, design, SILS, soft OILS, OILS, HILS, visualisation, evidence, release. The
> v1.0.0 release waits until all of it is done. The findings come from eight independent
> audits of this repository (each citing file and line); the most serious were re-checked
> in the code (marked ✔).
>
> **Kind:** plan · **For:** the owner and whoever does the work · **Status:** open (30 Sep 2026)

Legend: ✅ done · ❌ not done · 🟡 partly. Sizes: S ≤ 1 day, M 2–4 days, L a week or more.

---

# Part A — the eleven phases

## Phase 1 — Safety and correctness

✅ Done: the app checks Host and Origin, requires its own header on POST, caps sizes and connections, decodes `%` safely; text written into generated C/Rust is refused unless safe; no external `date` program; case CSV values, ids, `--set` types, engine settings and run limits refused by name.

❌ Not done:
1. ✔ **Strict loading.** Scenario, product, part, algorithm, catalogue and knobs JSON are read with defaults: a missing *or wrong-typed* key silently takes a default (`adcs-sim/src/json.rs:10-14`); unknown keys are never refused. Typed, strict loaders with named refusals. (M)
2. ✔ **`--set` on a key the engine never reads is accepted** (`config.rs:117`). A registry of read keys; anything else refused. (S)
3. ✔ **Hidden limits.** Schedule cut to 8 (`config.rs:233`); more than 8 rotors, 4 gimbals, 8 coils or Sun heads panic (`product.rs:56,104,71,140`); star-tracker heads `.min(2)`, RCS always 6 thrusters (`product.rs:121,128`); `q_inertial` length and fault index unchecked (`config.rs:254`, `run.rs:259-267`). All refused by name. (S)
4. ✔ **Metrics.** A misspelt `requirement` key leaves the requirement unjudged (`metrics.rs:170`); unknown kind, window or statistic gives NaN or "all". Refused. (S)
5. **Algorithms.** Unknown slot ignored, no fitting candidate gives `""`, an unmapped id silently gets law 0 or 1 (`config.rs:55-75,236-247`). Refused. (S)
6. **Values replaced by 0 or literals.** Unstated part values become 0 (`config.rs:382-393`); box size, CM direction, accommodation, `vb_ratio`, `spec_frac`, magnetometer coil coupling hard-coded (`config.rs:217,406-407`, `product.rs:136`). From the case or part, or refused. (M)
7. **Default case "ais_3u"** in the CLI and the app (`fly.rs:20`, `routes.rs:127`). Removed. (S)
8. **The MATLAB twin** accepts unknown `set` paths (`+asils/util/setpaths.m:7`) and has 27 silent `getf` defaults (`config.m`). Same rules as the engine. (M)
9. **App safety still open:** a panic during a flight poisons the flight lock and every later flight answers 409 ✔ (`routes.rs:18,119`); export temp file name collides (`routes.rs:105`); connection-cap race (`main.rs:126`); symlinks followed in the store (`store.rs:148`). (S)

**Done when:** a test per rule breaks one shipped input at a time and gets a named refusal, in engine and twin; every shipped input still flies.

## Phase 2 — Test the enforcers

✅ Done: tests for the node verifier, the generators, the command registry, the step announcements (ast), the registry against clap (TOML parser); temp-copy integration tests; Python lint; each check proven by breaking its target.

❌ Not done:
1. Unit tests for `adcs-sim-core` (none), `metrics`, `Dev::load`, `rec::write`, export/import round trip, the app's `routes.rs`, the CLI's `fly.rs`. (M)
2. Tests for the untested tools: `pipeline*` (converge, select), `report*`, `vv_report`, `rescore`, `run_matrix`, `floquet`, `kit`, `pack_*`, `check_all`, `fswcfg`, the engine's `draw`/`summarise`. (M)
3. Mutation testing (cargo-mutants) on `adcs-sim-core` and the flight software's control and estimation, with a recorded kill rate. (M)
4. Fuzzing (cargo-fuzz): parameter blob, adcs-link frames, UART frames, telecommands, `.trinetra` import, the app's request parser. (M)
5. `make check` in CI and a real recursion/stack check in the C build (`fsw/Makefile:4,29`). (S)
6. Twin: seed the random tests (`run_all_tests.m:25-108`); tests for `campaign.draw`, `metrics.evaluate`, `solution.*` (27 tests for 147 functions). (M)

**Done when:** every enforced rule has a passing and a failing test; fuzzers run clean for a set time; the mutation kill rate is in `check_all`.

## Phase 3 — Structure

✅ Done: clap for `adcs`; typed errors in `adcs-sim` and `adcs-design`; `engine.py`, `report.py`, `pipeline.py` split by concern; shared `tools/common.py`; page HTML out of code strings; one build helper (one `build.rs`).

❌ Not done:
1. Split the longest functions: `run` 267 lines, `Config::build` 262, `Dev::load` 118 (`run.rs:144`, `config.rs:190`, `product.rs:43`). (M)
2. One run pipeline shared by the CLI and the app (case resolution, run, derive, evaluate, write) instead of two copies (`fly.rs:8-70`, `routes.rs:118-145`). (S)
3. Typed errors in `adcs-pop` and `adcs-fsw-abi`; the app's "already running" passed as a parsed string (`main.rs:110`). (S)
4. One case-CSV parser in Python (six today), one `ROOT` (seventeen tools redefine it). (S)
5. Dead code (`config.rs:461`, `run.rs:243,319,408`). (S)

**Done when:** no engine function above 120 lines; the CLI and the app call one pipeline; no `Result<_, String>` in the engine's crates.

## Phase 4 — Right-size the process

✅ Done: CODEOWNERS names the real owner; `vX.Y.Z` tags checked against `VERSION`; CI jobs with time limits; no approval gate that cannot pass; a glossary; `START_HERE`, `CHANGING`, `ENVIRONMENT`.

❌ Not done:
1. The release proves less than CI: `prove` runs neither the twin nor the QEMU parity, and nothing requires the tagged commit to be green (`release.yml:54`); NOT RUN still passes (`check_all.py:116`) — add `--strict`. (S)
2. One version: `VERSION` 1.0.0, Cargo 1.2.0, a hand-written `ENGINE` string, zip names mixing both (`adcs-sim/src/lib.rs:24`, `pack_flight.py:21`). One source, checked in `prove`. (S)
3. Reproducible builds: `--locked`, `rust-toolchain.toml`, actions pinned by SHA, `setup-python` in every job, pinned `pyflakes`. (S)
4. Branch protection and one human review before merge (nothing enforces it today). (S)
5. Docs merged into one path: 23 documents overlap (ARCHITECTURE_PLAN, DESIGN_LOOP, SOLUTION_PIPELINE, NODES tell one flow; SOFT_OILS and VIRTUAL_OBC exist in `docs/` and `results/`); stale text (RESULTS.md "Produced by matlab_sils", SOFT_OILS.md 104 vs 105/107, `realsat/README.md:37` `_retired/`, `spec/` naming `ADCS_PLATFORM`, `_package/`, `xtask`, two different `pack_matlab.py`). (M)
6. Licence and policy: ✔ NOTICE.md says all rights reserved and no licence, while the release publishes the wheel, kits and source zips; add the matching `LICENSE` (or publish only to collaborators), `SECURITY.md`, `CHANGELOG`, release notes generated from `VERSION`. (S, owner decides)

## Phase 5 — Failure handling

✅ Done: panics leave a crash report; every engine write is whole-file; app requests catch panics; empty environment variables mean unset; typed engine errors with exit 2 (refused) or 1 (failed).

❌ Not done:
1. ✔ **Stale results read as new.** Campaign, Monte Carlo and matrix folders are not cleared; a failed run's old manifest is read (`engine_campaigns.py:144-162`, `pipeline_verify.py:151-175`); failed runs drop out of pass rates instead of counting as failures. (S)
2. ✔ **`engine.py` exits 0 whatever fails** (`engine.py:96`); so do `run_matrix`, `vobc`, `fsw-parity` (`engine_runs.py:50-125`, `run_matrix.py:65-94`). (S)
3. `rescore.py` keeps a removed requirement's old verdict and skips unreadable files (`:30,43,88`). (S)
4. `vv_report.py` can ship an old PDF: Chromium's exit status ignored, old file accepted (`:632-637`). (S)
5. Silent skips and swallowed errors: `engine_twin.py:57,64`, `vv_report.py:32`, `kit.py:77-88`, `except: pass` in three tools; writes that bypass the whole-file writer (`vv_report.py:637`, `kit.py:58-89`, `macapp.py:54-60`, `pipeline_verify.py:101`, `report_base.py:83`). (S)
6. The OBC link fails forever instead of loudly: no read timeouts, a bad-CRC frame dropped with no reply so the engine blocks, no reconnection (`link.rs:104-122`, `adcs_link.c:176`). (S; the protocol work is in Part B5)

**Done when:** a test per item forces the failure and sees a non-zero exit, a named message, and no stale file used.

## Phase 6 — Scripts you can follow

✅ Done: the pipeline table (`docs/commands.toml`), `explain`, generated `COMMANDS.md` with a test; numbered steps from the registry in `engine.py` and the design loop, held to the registry by an ast test; `--dry-run` in `engine.py`, `pipeline.py`, `rescore.py`; a trace log; `why <file>`.

❌ Not done:
1. `--dry-run` on every tool that writes (about 15 tools: `report`, `vv_report`, `pack_*`, `gen_fsw_params`, `export_catalogue`, `catalogue`, `floquet`, `kit`, `macapp`, `make_icon`, `nodes_doc`, `components_doc`, `run_matrix`, `verify_nodes`, `build_wheel`); `components_doc.py:35` and `pack_flight.py:51` work at import, with no argparse. (M)
2. On failure: which step stopped, whether files are unchanged or put back, the exact command to retry. (M)
3. Registry fields for what each command checks, how to undo it, and where its code is (file, function), held true by a test. (S)
4. `trinetra.py trace` to show the last run; `why` with history (which run wrote a file, when, from which inputs). (S)
5. The journey diagram: edit → generate → check → fly → verify → PR → CI → release. (S)
6. Numbered steps in every tool that runs for more than a moment. (S)

## Phase 7 — Storage and outputs

✅ Done: the result package (folder + `.trinetra` share file); inputs kept once by fingerprint; summary always kept, time series thinned, pinned runs whole; an index behind one interface (plain file, never stale); a shared folder via `TRINETRA_STORE`; export/import.

❌ Not done:
1. Retention defaults (time series 30 days, summaries forever) applied without asking. (S)
2. Opening a thinned run re-flies it and shows what changed if the engine moved on. (S)
3. Output types: figures also as SVG/PDF; a report per run with SVG figures. (M)
4. ✔ **Results older than the engine.** Flying `nadir_hold_ais` now gives APE 10.8°, the committed run 12.6°. A freshness check (every stored result's engine, flight-software and input fingerprints against today's) and a full re-fly. (M)
5. Repository weight: the V&V PDF tracked twice; duplicate data files (DTCFILE, SOLFSMY, two 1.4 MB HTML); `de440s.bsp` 32.7 MB; 6,600 committed run files. (S, owner decides)
6. Twin runs (1.2 GB `rec.mat`) go through the same store rules. (S)

## Phase 8 — Component library and lesson pages

✅ Done: nothing of this phase yet (three pages are built separately: results page, V&V report, app page).

❌ Not done:
1. One component set (HTML, one script, one stylesheet, bundled fonts): station, say-simply, real-thing, where-it-breaks, try-it, figure, table, equation, story, check-yourself, references, claim tag; every page assembled from it, and a check that refuses hand-built components. (L)
2. `lesson.toml` per mode or algorithm (B-dot first), outside the result fingerprints. (M)
3. "Try it" widgets declared (sliders for inputs, outputs, a figure), computed by the engine compiled to WebAssembly (`adcs-sim-core` is already `no_std`). (L)
4. A lesson form with preview through the same review flow; pages work offline, no outside hosts (the report page loads Google Fonts today, `report_pages.py:98`). (M)

## Phase 9 — Figure numbers from the kernel, checked against references

✅ Done: figures are drawn from recorded runs, not typed-in numbers; the truth environment is bit-identical between engine and twin on all 40 scenarios.

❌ Not done:
1. **Engine runs are never plotted** — the report reads only the twin store (`report_base.py:123`). One plotting path over both stores. (M)
2. Numbers typed into figures and report text instead of read from the case or the runs: requirement lines 0.01 and 20 (`report_runs.py:213-215`, `report_pages.py:28-30`), 0.5°/s and 20° (`report_runs.py:34,92`), literature numbers and "identical" cards (`vv_report.py:265,580,590`), versions and epoch (`report_pages.py:101-102`). (S)
3. Logic written more than once becomes one source: Kp→ap (three places), campaign draws (Python and MATLAB), statistics, the pass rule (three), scenario building, orbit period (three). (M)
4. Selected figures checked against reference values or pictures (orbit, field, density, detumble curves). (M)
5. Constants and units from one place: Earth radius and μ repeated as literals (`config.rs:212-213`, `run.rs:277`), height in km in one function and m in another (`field.rs:44,111`). (S)

## Phase 10 — Readability guide, file splits, typed errors

✅ Done: the big files split (largest hand-written source 820 lines); typed errors in the engine library.

❌ Not done:
1. A one-page readability guide (how code and docs are written here). (S)
2. The function splits and remaining typed errors of phase 3 (tracked there). 
3. Unit types for quantities that cross module boundaries (length, angle, rate, field). (M)
4. Speed: per-tick allocations (`run.rs:140`), unbounded soft-OILS statistics (`run.rs:338`), a shifting 64-entry star-tracker history (`sensors.rs:156`); memory of a 30-day run measured and bounded. (S)

## Phase 11 — When first needed: Parquet, SQLite, animation, 3D

✅ Correctly not started: nothing needs them yet (the store is small; the plain-file index works).

❌ Build when the trigger is met: SQLite index (above ~50,000 results or when queries are wanted); Parquet for large series; attitude animation and the 3D geometry viewer (these become needed with Part B7 items 6–7).

---

# Part B — extra work TRI-NETRA needs

## B1. Flight software safety (L)
1. ✔ Decoded parameters trusted after the CRC: counts and indices up to 255 overrun C arrays (`adcs_fsw.c:128,276,348,404`, `adcs_drv.c:105,124`) and NaN/0 reach the laws. A generated `validate()` in C and Rust from `params.toml` with min/max; `adcs_fsw_init` refuses an invalid blob.
2. ✔ No NaN guard before actuators; NaN→int in `q15` is undefined in C and 0 in Rust, so the builds diverge (`adcs_drv.c:110-139`, `fsw-rs/src/drv.rs:129`). A last guard: non-finite command → zero and a fault.
3. Divisions without guards (field magnitude, `dt`, periods, maxima: `adcs_fsw.c:336-544`, `adcs_ctl.c:36-189`, `adcs_alloc.c:56-84`).
4. ✔ Sensor dropout ignored: `mag_ok` never read, a failed gyro's stale rate used (`adcs_drv.c:65`, `adcs_fsw.c:314`). Staleness timers; laws stop using a lost sensor.
5. Safe mode and FDIR: none today. Invalid attitude, lost orbit, stale sensor for N s → magnetorquer-only detumble; watchdog kick; Rust panic resets instead of `loop {}` (`fsw-rs/src/lib.rs:147`); a mode 11–254 no longer holds the last dipole (`adcs_fsw.c:653`).
6. Estimator guards: singular innovation rejects the update (`adcs_est.c:56`), star-tracker updates gated (`:102`), quaternion norm checked, TRIAD refuses parallel vectors.
7. Telecommands: only "set mode" exists and it does not check the fitted hardware (`adcs_fsw.c:667`). Add feasibility checks (TM/TC design is in B5).
8. Resources: stack region, guard and `-fstack-usage` budget; one libm for both firmware builds; `CFLAGS ?=` can drop the determinism flags (`fsw/Makefile:6`).

**Done when:** fuzzing (Phase 2.4) runs clean; a test per FDIR rule flies the fault and sees safe mode; C = Rust stays bit-identical.

## B2. Design (L)
The design loop runs end to end and picks an actuator family; it does not yet do an analysis-level ADCS design.
1. **Requirement flow-down and traceability**: tag each case key and metric with its spec row id; ✔ 26 of 58 case keys are never read (e.g. `req.rpe`, `req.pde`, `req.dump`, `req.prop`, `req.faults`, `req.recover`, `mission.duty`, `mass.iunc`, `magnetic.dunc`, `flex.*`, `resources.*`, `pointing.et`) — each gets code or is refused as unsupported.
2. **Pointing error budget** (spec rows gp_0–gp_5, no code): knowledge + control + alignment + thermal + jitter, allocated against APE/RPE; ✔ jitter returns NaN (`metrics.rs:165`) — port `jitter.m`.
3. **Lifetime environment and momentum**: sweep beta angle and season, solar cycle over `mission.life`; replace the arbitrary `0.25` secular factor (`adcs-design/src/lib.rs:124`); momentum dumping sized from `req.dump`/`req.hsat`.
4. **Power, thermal and data budgets**: orbit-average and peak power with duty and eclipse, checked against `resources.*` in select (only mass and volume are checked today, `pipeline_verify.py:32-35`).
5. **Sensor trade**: choose sensors from a catalogue against the knowledge allocation (every product gets the same fixed suite today, `lib.rs:405-409`).
6. **Redundancy and FDIR in selection**: fault campaigns count toward feasibility.
7. **Beyond 3U**: box size, class and survey product from the case (`lib.rs:80,339,412`); Monte Carlo dispersions generated from the case, not looked up for two known cases (`pipeline_verify.py:147`); products of inertia in the case.
8. **Sizing laws** documented with sources and margins (coil 0.5·B_min/0.3·B_mean, RCS Isp 60 s, slew propellant left out, `lib.rs:175-297`); fix the selection-order mismatch (`pipeline.py:18` "simplest" vs "lightest").
9. **The spec package**: about 90 of 243 system rows have code and none is linked by id; a spec → code → test matrix, and `spec/` brought up to date.

## B3. SILS (L)
1. **ECSS-E-ST-60-10 pointing metrics**: APE, RPE, AKE, MKE, PDE, PRE with windows and the temporal/ensemble/mixed interpretation and confidence level, in engine and twin.
2. **Power model**: solar arrays from attitude and shadow, battery state of charge, eclipse depth of discharge (only consumption is modelled today, `run.rs:365`).
3. **Monte Carlo statistics**: ✔ 15–40 runs per campaign cannot support 99.73 % claims (about 1,100 failure-free runs needed at 95 % confidence); run counts sized for the claim; disperse epoch, season, beta, products of inertia, latency, orbit; the same inertia knowledge in the flight software on both sides.
4. **Missing dynamics**: wheel imbalance and jitter in the loop; one or two flexible modes; fuel slosh when RCS carries propellant.
5. **Device fidelity**: magnetorquer RL dynamics, hysteresis, eddy currents; wheel stiction/Dahl friction, back-EMF, speed limit; star tracker Moon exclusion, blinding recovery, rate-dependent noise; GNSS latency and outages; outgassing, albedo and IR pressure torques; self-shadowing of appendages.
6. **Scenarios**: target pointing, Sun modes (`sun_mtq`, `sun_fine`), RCS detumble and rotor Sun acquisition as start modes, safe-mode entry and recovery, eclipse transitions, magnetometer/GNSS/RCS-valve faults.
7. **The 13 engine-vs-twin disagreements** fixed at their traced causes: FMR field-power switch hysteresis, RCS dump thrust-scale compensation, the Sun-spin sign-flip lock, the coils-only MEKF spread (`results/ENGINE_PARITY.md:228-232`).
8. **Port to the engine** what only the twin has: star-tracker image chain, CSS chain, jitter.

## B4. Soft OILS (M)
1. Overruns and deadline margin become pass/fail (they are only counted, `run.rs:340`).
2. CPI range sweep and an interrupt/jitter allowance; SPI rate as an option; link and HAL-copy overhead counted.
3. QEMU run with `-icount` as the link header promises (`adcs-fsw-abi/src/lib.rs:149`).
4. Calibration against a real board (B5.2): measured execution times replace the assumed CPI.

## B5. OILS with a real OBC (L)
1. **Harden the link**: timeouts, a NAK on a bad CRC, TICK sequence numbers and retry, heartbeat, reconnect-or-abort with a named error, wall-clock slip recorded per tick; an RS-422/UART transport beside TCP.
2. **A reference board target** (e.g. `fsw/targets/stm32f4/`: startup, linker script, UART link, cycle-counter trailer) — only QEMU, POSIX and the link server exist today.
3. **A real-bus HAL for that board** (I2C, SPI, CAN, PWM, UART), replacing the link-served HAL calls.
4. **HAL v2**: watchdog, non-volatile storage (calibration, parameters), power switches and latch-up reset, bus reset, sensor timestamps, faults on the link (`adcs_hal.h:51-104` has none).
5. **TM/TC**: ✔ `fsw/tm` and `fsw/tc` do not exist and nothing calls `adcs_hal_tm_emit`; housekeeping packets, parameter-table and calibration upload with CRC, event reports, command verification.
6. **Time**: PPS or clock-drift handling; the OBC clock is the engine's `now_ns` today.

## B6. HILS (L)
HILS is documentation only today.
1. **Interface emulation unit**: an engine-driven device box (second MCU or USB-I2C/CAN adapter) that answers the OBC's real buses from TICK data — the step from OILS to real buses.
2. **Stimulus drivers**: Helmholtz-cage currents, Sun-simulator pointing, air-bearing telemetry in; the missing `write_stimulus` (`+asils/+hal/stimulus.m:7`); the MATLAB UDP backend replaced by adcs-link (no CRC, sequence or timeout today).
3. **One device at a time**: magnetometer, then coils, then wheels, each with an acceptance test against SILS.

## B7. Visualisation and reports (L)
1. **The app shows time series**: a channels endpoint; attitude error, rates, modes, momentum and power with requirement lines and eclipse bands, zoom and hover (the app draws no plots today).
2. **One plotting module** for engine and twin runs, one palette and figure numbering, the full 11-mode axis (✔ MATLAB clips at 5, Python labels 7: `+asils/+viz/run.m:32`, `report_base.py:151`).
3. **Requirement verification matrix in the V&V report**: requirement → method → run/ledger → verdict, test procedures, deviations; open items from a data file, not code (`vv_report.py:533-567`); stale section cross-references fixed (`:401-402,467`).
4. **Pointing budget figure**, mission-long momentum/saturation and power/duty timelines, eclipse bands on every time plot.
5. **Monte Carlo envelopes** with p50/p95/p99.73 bands and confidence intervals; run-to-run overlays (twin vs engine, C vs Rust, SILS vs OILS); NaN no longer drawn as zero (`report_solutions.py:95`).
6. **3D attitude and geometry viewer**: body axes, sensor fields of view, Sun/Earth/Moon exclusion cones (Phase 11 trigger).
7. Figures exported as SVG/PDF; dark-mode aware; no outside fonts.

## B8. Evidence and traceability (L)
1. **A requirements traceability matrix**, generated: every stated requirement → metric → scenario → latest result; ✔ `req.slew` has no metric; 66 of 173 metrics are unjudged — each decided.
2. **Outside validation of the orbit propagator**: the CHAMP/ITSG orbit-determination comparison stored as a ledger; published cases (Vallado, SOFA, published DTM2020/JB2008) as tests. Today it is checked only against its own MATLAB original (`pop/08_test/run_all_tests.m:237`).
3. **Published cases for the algorithms**: at least B-dot, MEKF, LQR and QUEST against numbers from their papers.
4. **Catalogue**: each datasheet number traced to its page and tested; the 14 synthetic parts replaced by real units.
5. **`status` headline** extended: rows without an outside reference, requirements without a metric, stale results.
6. Owner work: confirm the 31 algorithms (all UNCONFIRMED), choose real parts.

## B9. Release (M) — the last step
1. The wheel tagged per platform (it is tagged `any` but carries native programs) and installed and run on Windows and macOS too; macOS binaries in the wheel signed like the kit's.
2. C/Rust parity run on Windows (MinGW) and macOS in CI, not only at release.
3. Release notes generated from `VERSION` (file names are hard-coded today).
4. Then: merge PR #12, tag `v1.0.0`, watch every job, publish.

---

## Order

| wave | work | about |
|---|---|---|
| 1 | Phase 1, Phase 5, B1 (flight software safety) | 2 weeks |
| 2 | Phase 2, Phase 3, B8 (evidence, traceability, freshness) | 3 weeks |
| 3 | B2 design, B3 SILS, B4 soft OILS | 4 weeks |
| 4 | Phase 6, Phase 7, Phase 9, Phase 10, B7 visualisation | 3 weeks |
| 5 | Phase 8 lesson pages | 2 weeks |
| 6 | B5 OILS, B6 HILS (need a board and hardware) | from 3 weeks, hardware-paced |
| 7 | Phase 4, B9, release | 1 week |

Each wave ends with `check_all` and CI green and this file's ticks updated.

## Decisions the owner makes

| decision | suggested |
|---|---|
| Licence of published downloads (NOTICE.md says all rights reserved) | decide before wave 7 |
| Safe-mode policy (B1.5) | magnetorquer-only detumble after 60 s of invalid attitude or stale field |
| Lesson pages (Phase 8) | yes, B-dot first |
| Retention of time series (Phase 7) | 30 days |
| `de440s.bsp` and committed run files (Phase 7) | fetch by checksum; keep ledgers, thin runs |
| Reference OBC board for OILS (B5.2) | an STM32F4 class board, or the owner's flight OBC |
| HILS equipment available (B6) | owner lists what exists (cage, air bearing, Sun simulator) |
| Does v1.0.0 wait for B5/B6 (hardware-paced)? | release after waves 1–5 and 7; OILS/HILS in v1.1 — owner's call |
