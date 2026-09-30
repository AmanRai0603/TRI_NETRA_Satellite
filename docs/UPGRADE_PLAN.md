# TRI-NETRA ADCS: the upgrade plan, phases 1 to 11

> **Answer first.** The release (v1.0.0) waits until all eleven phases are done. The plan
> comes from four independent audits of this repository (flight software and OBC link;
> engine and app; tools, twin and evidence; CI, release and process), each citing file and
> line; the most serious claims were then re-checked against the code. It follows the VLEO
> review's phases where they apply to TRI-NETRA, and adds what that review could not see
> here: the flight software, the OBC link, the evidence behind the numbers, and requirement
> traceability.
>
> **Kind:** plan · **For:** the owner and whoever does the work · **Status:** open (30 Sep 2026)

## Where TRI-NETRA stands

Already done (commits on `claude/brave-dijkstra-rshrb7`, PR #12): inputs refused by name
for case CSV, ids, `--set` types and engine limits; whole-file writes and crash reports;
provenance, kept inputs, index, pin/thin, `.trinetra` share files; the commands registry
with `explain`, `why`, `--dry-run` (engine.py, pipeline.py), steps announced as they run;
clap and typed engine errors; structural checks of the registry (ast, TOML); the desktop
app with Host/Origin checks; CI with time limits; the release workflow, kits, one wheel.

Found open by the audits (re-checked items marked ✔):

| area | the worst of it |
|---|---|
| flight software | ✔ decoded parameters are trusted after the CRC: counts and indices up to 255 overrun fixed arrays in C (`adcs_fsw.c:128`, `adcs_drv.c:105`), NaN and zero reach the laws; ✔ the OBC link ignores the frame length (`adcs_link.c:139`) and a short CONFIG underflows (`:181`); ✔ no `isfinite` anywhere before the actuators, and NaN→int in `q15` is undefined in C (`adcs_drv.c:110`); ✔ `mag_ok` is never read (`adcs_drv.c:65`); no safe mode, no watchdog, Rust panic = `loop {}` (`fsw-rs/src/lib.rs:147`) |
| engine | ✔ a misspelt `--set fsw.…` key is accepted and never read (`config.rs:117`); `json::f/s/b` return the default for missing *and* wrong-typed keys (`json.rs:10-14`); ✔ a misspelt `requirement` in a scenario leaves the requirement unjudged (`metrics.rs:170`); ✔ schedule cut to 8 (`config.rs:233`); more than 8 rotors / 4 gimbals / 8 coils panic (`product.rs:56,104,71`); missing part values become 0 (`config.rs:382-393`); box size, CM direction and accommodation hard-coded (`config.rs:217,406-407`) |
| app | ✔ a panic during a flight poisons `FLYING`, and every later flight answers 409 (`routes.rs:18,119`) |
| tools | ✔ campaign and MC folders are not cleared, so a failed run's old manifest is read as new (`engine_campaigns.py:144`); ✔ `engine.py` exits 0 whatever fails (`engine.py:96`); `rescore.py` keeps stale verdicts (`:43`); `vv_report.py` can ship an old PDF (`:632-637`) |
| evidence | 0 of 31 algorithms confirmed or checked against a published case; 14 of 14 parts synthetic; catalogue numbers untested; the orbit propagator validated only against its own twin (`pop/08_test/run_all_tests.m:237`); ✔ the committed engine results predate the engine (flying `nadir_hold_ais` now gives APE 10.8°, committed 12.6°) |
| traceability | no requirement → metric → scenario → result matrix; ✔ `req.slew` has no metric; 66 of 173 metrics are unjudged |
| release | `prove` runs neither the twin nor the QEMU parity (`release.yml:54`); NOT RUN still passes (`check_all.py:116`); versions disagree (VERSION 1.0.0, engine 1.2.0, hand-written `ENGINE` string); no `--locked`; the wheel is tagged `any` but carries native programs; ✔ NOTICE.md says "all rights reserved, no licence" while the release publishes downloads |

## The phases

Each phase lists its work, the check that says it is done, and a size (S ≤ 1 day, M 2–4
days, L a week or more). Order matters: 1–3 make the tool trustworthy, 4–5 prove it,
6–10 make it followable and maintainable, 11 ships it.

### Phase 1 — Refuse, never guess, everywhere (M)
1. Typed, strict loading of scenario, product, part, algorithm, catalogue and knobs JSON: unknown keys and wrong types refused by name; a default only where the schema says one exists. (`json.rs:10-14`, `config.rs`, `product.rs`)
2. A registry of the keys the engine reads; `--set` on any other key is refused. (`config.rs:117`)
3. Capacities refused by name, never truncated or overrun: schedule entries, rotors, gimbals, coils, Sun heads, star-tracker heads, RCS thrusters, `q_inertial` length, fault index. (`config.rs:233-254`, `product.rs:56-146`, `run.rs:259-267`)
4. Metrics: an unknown kind, window, statistic, or requirement key is refused. (`metrics.rs:95-175`)
5. Algorithms: an unknown slot, an id with no law mapped, or no fitting candidate is refused. (`config.rs:55-75,236-247`)
6. Physics from the case or the part, not from literals: box size, CM offset direction, accommodation, `vb_ratio`, `spec_frac`, magnetometer coil coupling; a part value that is not stated is refused, not set to 0. (`config.rs:217,382-407`, `product.rs:136`)
7. No default case: the scenario names its case or the run is refused. (`fly.rs:20`, `routes.rs:127`)
8. The twin does the same: `setpaths` refuses unknown paths, `getf` defaults only where the schema allows. (`+asils/util/setpaths.m:7`, `getf.m:3`, `config.m`)
9. Soft-OILS inputs `--cpi`, `--obc-mhz` refused at 0 or below. (`cli.rs`)

**Done when:** a test per rule breaks a shipped scenario, product and case one key at a time and gets a named refusal, in the engine and in the twin; every shipped input still flies.

### Phase 2 — Flight software that cannot be driven into undefined behaviour (L) · *new for TRI-NETRA*
1. A generated `adcs_params_validate()` / `Params::validate()` after decode (C and Rust, from `params.toml` with min/max added): counts and indices within capacity, modes valid, every float finite, every divisor positive. `adcs_fsw_init` refuses an invalid blob.
2. The OBC link: every read in TICK bounded by the frame length; CONFIG shorter than 8 refused; oversize data refused loudly, not dropped. (`adcs_link.c:108-181`)
3. The engine side of the link: replies length- and type-checked, read timeouts, BYE with a timeout. (`adcs-fsw-abi/src/link.rs:83-199`)
4. A last guard before the actuators: a non-finite command is zeroed and raises a fault; `q15` and the valve conversion are NaN-safe in C and Rust alike. (`adcs_ctl.c:166`, `adcs_drv.c:110-139`, `fsw-rs/src/drv.rs:129`)
5. Guards on every division the audit lists (field magnitude, `dt`, periods, maxima). (`adcs_fsw.c:336-544`, `adcs_ctl.c:36-189`, `adcs_alloc.c:56-84`)
6. Sensor dropout: `mag_ok`, `gyro_ok` and star-tracker validity used, with staleness timers; the laws that need a lost sensor stop using it.
7. A safe mode and FDIR policy: invalid attitude, lost orbit or a stale sensor for N seconds → magnetorquer-only detumble; a watchdog kick in the step; the Rust panic handler resets instead of spinning. Telecommands check that the requested mode can fly with the fitted hardware. (`adcs_fsw.c:653,667`)
8. Estimator guards: a singular innovation rejects the update; star-tracker updates gated; quaternion norm checked; TRIAD refuses parallel vectors. (`adcs_est.c:56-110`)
9. Resources: a stack region with a guard, `-fstack-usage` checked against a budget; soft-OILS overruns fail the run; one libm for both firmware builds.

**Done when:** fuzzing (phase 4) of decode, link, UART and telecommands runs clean; a test per FDIR rule flies the fault and sees the safe-mode entry; C = Rust stays bit-identical; every shipped scenario's verdicts are unchanged or explained.

### Phase 3 — Failure handling in the tools and the app (M)
1. Campaign, Monte Carlo and matrix runs clear their run folders first; a failed run counts as a failed run, never dropped and never read from an old manifest. (`engine_campaigns.py:144-162`, `pipeline_verify.py:151-175`, `run_matrix.py:65-94`)
2. Every tool exits non-zero when anything it ran failed: `engine.py` commands, `run_matrix`, `vobc`, `fsw-parity`. (`engine.py:96`, `engine_runs.py:50-125`)
3. `rescore.py` clears a removed requirement's verdict and reports unreadable files. (`:30,43,88`)
4. `vv_report.py` deletes the old PDF first and checks Chromium's exit status. (`:632-637`)
5. Silent skips become named failures (`engine_twin.py:57,64`, `vv_report.py:32`, `kit.py:77-88`, the `except: pass` blocks); every write goes through `common.write_*`.
6. The app recovers from a panic in a flight (poisoned mutex handled), exports to a unique temp file, holds the connection cap without a race, and stops following symlinks out of the store. (`routes.rs:18,105`, `main.rs:126`, `store.rs:148`)
7. Typed errors in `adcs-pop` and `adcs-fsw-abi`, so a bad input to the propagator is a refusal, not a run failure; the app no longer passes "already running" as a string. (`error.rs:48`, `main.rs:110`)

**Done when:** a test per item makes the failure happen and sees the exit status, the named message and no stale file used.

### Phase 4 — Test the enforcers, fuzz the boundaries (L)
1. Unit tests for `adcs-sim-core` (none today), `metrics` (derive, evaluate, windows), `Dev::load`, `rec::write`, export/import round trip, `routes.rs`, `fly.rs`.
2. Fuzz targets (cargo-fuzz): parameter blob decode, adcs-link frames, UART frames, telecommands, `.trinetra` import, the app's request parser.
3. Mutation testing (cargo-mutants) on `adcs-sim-core` and the flight software's control and estimation code, with a floor on the kill rate.
4. `make check` in CI, and a real recursion and stack check in the C build. (`Makefile:4,29`)
5. Tests for the untested tools: `pipeline*`, `report*`, `rescore`, `run_matrix`, `floquet`, `kit`, `pack_*`, `check_all`, `fswcfg`, and the engine's `draw`/`summarise`.
6. Twin tests seeded; tests for `campaign.draw`, `metrics.evaluate`, `solution.*`. (`run_all_tests.m:25-108`)

**Done when:** every check the repository enforces has a passing and a failing test; the fuzzers ran a set time with no crash; mutation kill rate recorded in `check_all`.

### Phase 5 — Evidence and traceability (L) · *expanded for TRI-NETRA*
1. A requirements traceability matrix, generated: every stated `req.*` → the metric(s) that judge it → the scenarios → the latest result. A stated requirement with no metric fails the check; add the missing `req.slew` metric; decide for each of the 66 unjudged metrics whether it is judged or informative.
2. Results freshness: a check that every committed result was flown by the current engine, flight software and inputs (fingerprints in the manifest); stale results listed, then every stored result re-flown.
3. External validation of the orbit propagator: the CHAMP / ITSG orbit determination comparison stored as a ledger with its numbers, and published test cases (Vallado, SOFA time and frames, published DTM2020/JB2008 densities) as automated tests.
4. Published cases for the algorithms: at least B-dot, MEKF, LQR and QUEST checked against values from their papers, not only property tests.
5. The catalogue: each datasheet number traced to its source page and checked; the synthetic parts replaced by real units as they are chosen.
6. `status` headline expanded: rows without an outside reference, requirements without a metric, stale results.
7. Owner work (not code): confirm the 31 algorithms, choose real parts.

**Done when:** the matrix shows every stated requirement judged; freshness passes; the propagator and four algorithms have outside-reference tests; `status` shows the remaining debt by name.

### Phase 6 — Scripts you can follow, completed (M)
1. `--dry-run` on every tool that writes; the module-level writers (`components_doc.py:35`, `pack_flight.py:51`) and argument-less tools get argparse.
2. On failure, every tool says which step stopped, whether files are unchanged or were put back, and the command to retry.
3. Registry fields for what each command checks, how to undo it, and where its code is (file and function); a test holds them true.
4. `trinetra.py trace` shows the last run of any tool; `why <file>` adds the file's history (which run wrote it, when, from which inputs).
5. One diagram of a change's journey: edit → generate → check → fly → verify → PR → CI → release.
6. Steps announced by every tool that runs for more than a moment, not only engine.py and the design loop.

**Done when:** `tests/test_steps.py` and the registry tests cover every writing tool; a forced failure in each prints step, file state and retry.

### Phase 7 — Storage and outputs (M)
1. Retention defaults: summaries forever; time series 30 days (to confirm); pinned and exported runs forever; `thin` runs by default age.
2. Opening a thinned run re-flies it and shows what changed if the engine has moved on.
3. Figures as SVG (smaller, diffable, sharp in the report); the results page and V&V report use them.
4. Repository weight: the V&V PDF tracked once, duplicate data files (DTCFILE, SOLFSMY, the two 1.4 MB HTML files) kept once, `de440s.bsp` (32.7 MB) moved to LFS or fetched with a checksum; a decision on the 6,600 committed run files.
5. A shared results folder documented for several machines (`TRINETRA_STORE`), and an index that can move to SQLite later behind the same interface.
6. Deferred until needed, as planned: Parquet for large series, SQLite index, animation, 3D.

**Done when:** the store's size for a year of use is estimated from measured sizes, thinning and re-flying are tested, figures are SVG.

### Phase 8 — One page system, and lesson pages (L) · *decision needed*
1. One component set (HTML, one script, one stylesheet, bundled fonts) used by the results page, the V&V report and the app page; no page builds a component by hand.
2. Lesson pages for the ADCS, one per mode or algorithm (`lesson.toml` beside its data, outside the result fingerprints): what it does, the equations, the paper, where it breaks, and "try it" widgets declared as inputs and outputs, computed by the engine itself (compiled to WebAssembly from `adcs-sim-core`, which is already `no_std`), so a page never shows a number the engine would not compute.
3. Pages work offline; no page contacts an outside host.

**Done when:** the first lesson (B-dot detumble suggested) is generated from its `lesson.toml` with a working try-it widget, and the three existing pages use the component set.

### Phase 9 — One source for every number (M)
1. Logic written more than once becomes one: Kp→ap (three places), campaign draws (Python and MATLAB), statistics, the pass rule (three places), scenario building, orbit period (three places), the case CSV parser (six in Python). The engine or one shared module computes; the others call it or are tested against it.
2. Figures drawn from engine outputs only, never re-derived in the report; selected figures checked against reference values or pictures.
3. Constants (Earth radius, μ, unit conversions) from one place; km/m and deg/rad consistent across `field.rs`, `config.rs`, `run.rs`.

**Done when:** a test fails if any duplicated formula reappears (ast/TOML checks), and each shared number has one definition.

### Phase 10 — Structure, units, speed and readability (M)
1. Split the longest functions: `run` (267 lines), `Config::build` (262), `Dev::load` (118).
2. One run pipeline in `adcs-sim` used by the CLI and the app (case resolution, run, derive, evaluate, write), instead of two copies. (`fly.rs:8-70`, `routes.rs:118-145`)
3. Unit types for the quantities that cross module boundaries (length, angle, rate, field), starting where km/m and deg/rad already mix.
4. Speed: no per-tick allocation (`run.rs:140`), bounded soft-OILS statistics (`run.rs:338`), a ring buffer for the star-tracker history (`sensors.rs:156`); a run of 30 days holds bounded memory.
5. Dead code removed (`config.rs:461`, `run.rs:243,319,408`).
6. A readability guide (one page), and the docs merged into one path: README → START_HERE → ARCHITECTURE (merging ARCHITECTURE_PLAN, DESIGN_LOOP, SOLUTION_PIPELINE) → CHANGING → GLOSSARY; stale text fixed (RESULTS.md "Produced by matlab_sils", the `_retired/` link, `spec/` references to `ADCS_PLATFORM`, `_package/`, `xtask`; the two `pack_matlab.py`).

**Done when:** no function above 120 lines in the engine's own crates; a 30-day run's memory is measured and bounded; docs have one entry path with every link checked.

### Phase 11 — Process, CI and the release (M)
1. One version: `VERSION` drives the Cargo workspace, the `ENGINE` string, the flight software build ids, the zips, the wheel and the app; `prove` checks they agree.
2. The release proves what CI proves: `prove` requires CI green on the tagged commit (twin, QEMU parity) and `check_all --strict` fails on NOT RUN.
3. Reproducible builds: `--locked`, a `rust-toolchain.toml`, actions pinned by SHA, `setup-python` in every job, pinned Python tools, `SOURCE_DATE_EPOCH` for Cargo.
4. The wheel tagged per platform (or the unsupported ones refused at install), and installed and run on Windows and macOS too; macOS binaries in the wheel signed like the kit's.
5. C and Rust parity run on Windows (MinGW) and macOS in CI, not only at release.
6. Licence and policy: a `LICENSE` that matches what is published (NOTICE.md says all rights reserved), `SECURITY.md`, a `CHANGELOG`, release notes generated from `VERSION`.
7. CODEOWNERS and branch protection matching the real team; one human review before merge.
8. Then the v1.0.0 release: merge PR #12, tag, watch every job, publish.

**Done when:** a tag on a green commit builds, tests on three systems, and publishes every download with matching versions and checksums.

## Decisions the owner makes

| decision | options | suggested |
|---|---|---|
| Licence of the published downloads | keep all rights reserved and publish only to collaborators (private repository releases); or choose a licence | decide before phase 11 |
| Safe-mode policy (phase 2.7) | which conditions, how long before safe mode, what safe mode flies | magnetorquer-only detumble after 60 s of invalid attitude or stale field |
| Lesson pages (phase 8) | yes, one per mode/algorithm; or skip phase 8 | yes, starting with B-dot |
| Retention of time series (phase 7) | 30 days, other | 30 days |
| `de440s.bsp` and committed runs (phase 7) | LFS; fetch by checksum; keep as is | fetch by checksum |
| Real parts and algorithm confirmation (phase 5) | owner's engineering work | as parts are chosen |

## Order and size

Phases 1–3 first (they make results trustworthy; about two weeks), then 4–5 (proof and
evidence; about three weeks), then 6–10 in any order (about three weeks), then 11 and the
release (a few days). Each phase ends with `check_all` green, CI green and a note in the
PR; the plan's tables are updated as items close.
