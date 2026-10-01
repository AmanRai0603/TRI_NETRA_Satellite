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

✅ Done (wave 1): the app checks Host and Origin, requires its own header on POST, caps sizes and connections, decodes `%` safely; text written into generated C/Rust is refused unless safe; no external `date` program; case CSV values, ids, `--set` types, engine settings and run limits refused by name.

1. ✅ ✔ **Strict loading.** Scenarios are checked against one schema (`schema.rs`, exported to the twin): unknown keys and wrong types refused, a key left out takes its documented default. Products and parts: every value a fitted device reads must be stated. Design knobs: unknown keys, wrong types and out-of-range values refused (`Knobs::from_json`). Catalogue JSON is its TOML (`export_catalogue --check`).
2. ✅ ✔ **`--set` on a key the engine never reads** is refused (the schema's settable keys; `ENGINE_KEYS` for `engine.*`).
3. ✅ ✔ **Hidden limits** refused by name: schedule length, rotors, gimbals, coils, Sun heads, star-tracker heads, RCS thruster count, `q_inertial`, fault index and device.
4. ✅ ✔ **Metrics**: a requirement key the case does not state, an unknown kind, window or statistic, a judged metric of a kind not computed — refused.
5. ✅ **Algorithms**: an unknown slot, or an id the engine does not fly, is refused naming where it runs.
6. ✅ **Values replaced by literals.** The body comes from the class the case names (`catalogue/classes.toml`, CubeSat Design Specification; a blank or unknown `meta.class` is refused); the magnetometer's coil coupling from its part (`coil_coupling_T_per_Am2`); unstated part values are refused. The surface model's accommodation, `vb_ratio` and specular share are named engine settings (`engine.accommodation`, `engine.vb_ratio`, `engine.spec_frac`, Moe & Moe 2005), recorded under every run's `assumptions` with the CM direction. Every shipped scenario flies byte-identical channels before and after.
7. ✅ **Default case "ais_3u"** removed from the CLI and the app.
8. ✅ **The MATLAB twin**: the same scenario schema, `set` paths that do not exist refused, the class body and the part's coupling read as the engine reads them (27/27 tests).
9. ✅ **App safety**: a panic during a flight no longer poisons the flight lock; unique export temp names; links never followed in the store; at the connection cap nothing more is accepted (it waits in the kernel queue, as the comment always said) and a failed thread spawn gives its slot back.

**Done when:** a test per rule breaks one shipped input at a time and gets a named refusal, in engine and twin; every shipped input still flies. **Met** (`engine/crates/adcs-sim/tests/inputs.rs`, the twin's `t_scenario_check`).

## Phase 2 — Test the enforcers

✅ Done before: tests for the node verifier, the generators, the command registry, the step announcements (ast), the registry against clap (TOML parser); temp-copy integration tests; Python lint; each check proven by breaking its target.

1. ✅ **Unit and end-to-end tests** where there were none: `adcs-sim-core` (conservation of momentum and energy, wheel exchange, the gravity-gradient closed form, a circular orbit's radius and period, quaternion/DCM round trips, the bus codecs); every shipped product loads; a flight judged, recorded, shared and read back unchanged; the `adcs` command's exit codes and every `results` subcommand; every app route (refusals, a flight, the store, an export).
2. ✅ **Tests for the tools**: 135 new (pipeline converge/select/robust, the ledger, `fswcfg`, `check_all`, `kit`, the report helpers, campaign draws and summaries, `run_matrix`, `rescore`, `floquet`). `fswcfg` raised a raw `struct.error` on a short blob and read a payload of the wrong length; it now refuses both. `pack_*` are scripts with no functions (left as they are).
3. ✅ **Mutation testing** (`tools/mutation.py`, `check_all --mutation`, weekly in CI) of the flight software's control and estimation, with the kill rate per function in `results/MUTATION.md`. 685 of 700 caught (97.9 %; the first run caught 58 %): closed-form tests for estimation, guidance and every control law; the 15 survivors are equivalent mutants (x/s for s = ±1, `>` against `>=` at a value no flight reaches); the floor is 95 %.
4. ✅ **Fuzzing** (seeded, on stable Rust; `ADCS_FUZZ_N` sets the count): parameter blobs, telecommands and bus bytes into the flight software; C against Rust bit for bit on random configurations and bus data; the app's request parser; the `.trinetra` reader; the OBC link server. Two faults found and fixed: finite but extreme parameters gave NaN commands (now refused by generic magnitude rules and unit-vector rules in both builds' generated validators), and the C driver sent uninitialised stack bytes past a CAN frame's length (now zeroed).
5. ✅ `make check` runs in CI (through `check_all`); the stack-depth and recursion check (`tools/fsw_stack.py`) runs in CI's firmware job.
6. ✅ **Twin**: the random tests seeded; tests for `campaign.draw`, `metrics.evaluate`, `solution.*` (30 tests).

**Done when:** every enforced rule has a passing and a failing test; fuzzers run clean for a set time; the mutation kill rate is in `check_all`. **Met.**

## Phase 3 — Structure

✅ Done: clap for `adcs`; typed errors in `adcs-sim` and `adcs-design`; `engine.py`, `report.py`, `pipeline.py` split by concern; shared `tools/common.py`; page HTML out of code strings; one build helper (one `build.rs`).

1. ✅ The longest functions split into named stages, every shipped scenario byte-identical before and after: `run` (270 → about 110 lines: units, initial state, faults, sensors to bytes, soft-OILS latency, actuators, the recorded row, the plant step), `Config::build` (286 → about 90: the flight-software parameter table in eight stages, the faults, the engine settings), `Dev::load` (148 → about 30: capacities, actuator slots, sensor slots, a `Part` that refuses unstated values). Left whole on purpose: the ported atmosphere models (`jb2008_core`, `gldtm`, `gldtm_hp`), line-for-line transliterations kept checkable against their published sources.
2. ✅ One flight pipeline (`adcs_sim::flight`: the case for the scenario, the run, the metrics, the record, the store's retention) called by `adcs run`, the app and `adcs results refly`.
3. ✅ Typed errors in `adcs-pop` (`PopError`: data, unsupported, run) and `adcs-fsw-abi` (`FswError`: refused, link), mapped into the engine's kinds (a refused configuration exits 2, a dead link 1); the app's start-up outcome is an enum, not a parsed string; the `.trinetra` reader's errors are typed.
4. ✅ One case reader in the tools (`common.case_rows`, with `case_values` on it) and one `ROOT` (`common.ROOT`; sixteen copies removed).
5. ✅ Dead code removed (`can_rx_count`, `_unused`, the `From<String>` fallback).

**Done when:** no engine function above 120 lines (ported reference models aside); the CLI and the app call one pipeline; no `Result<_, String>` in the engine's crates. **Met.**

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

1. ✅ ✔ **Stale results read as new.** Campaign, Monte Carlo, matrix and run folders are cleared before a run; a failed run counts as a failure in every pass rate (`n_failed_runs`), never drops out.
2. ✅ ✔ **`engine.py`, `run_matrix`, `vobc`, `fsw-parity`** exit non-zero when anything failed.
3. ✅ `rescore.py` clears a removed requirement's verdict, refuses a missing case and fails on unreadable files.
4. ✅ `vv_report.py` refuses to build on a missing ledger, removes the old PDF first and checks Chromium's exit status.
5. ✅ Silent skips and swallowed errors removed (`engine_twin`, `vv_report`, `kit`); writes go through the whole-file writer.
6. ✅ The OBC link fails loudly: replies checked, a refusal for every bad frame, read timeouts (B1.9). Reconnection belongs to the OILS protocol work (B5).

**Done when:** a test per item forces the failure and sees a non-zero exit, a named message, and no stale file used. **Met** (`tests/test_failures.py`, `tests/test_link.py`, `link.rs` tests).

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

✅ Done: the result package (folder + `.trinetra` share file); inputs kept once by fingerprint; summary always kept, time series thinned, pinned runs whole; a shared folder via `TRINETRA_STORE`; export/import. **The results database:** a SQLite index per listed folder (`.adcs-index.sqlite`: a `runs` table with provenance and verdict counts, a `metrics` table with every value, requirement and verdict), still only a cache over the manifests (never stale, rebuilt when damaged) and asked anything read-only with `adcs results query --sql`.

1. ✅ Retention applied without asking: every run written into the store thins the unpinned runs older than the retention; an installed kit keeps time series 30 days and summaries for ever, a repository checkout keeps everything (its pages are drawn from local series); `TRINETRA_RETENTION_DAYS` sets it. (Owner to confirm 30 days.)
2. ✅ `adcs results refly <run>` flies a stored run again from the inputs it kept (optionally on another `--fsw`) and prints every metric stored against now, flagging changed values and verdicts.
3. ❌ Output types: figures also as SVG/PDF; a report per run with SVG figures. (M)
4. 🟡 ✔ **Results older than the engine.** Flying `nadir_hold_ais` now gives APE 10.8°, the committed run 12.6°. **Done:** runs record the fingerprint of the engine's and the flight software's sources (`engine_source`, built by `build.rs`) and of their product and part files, both in the result id; `adcs results stale` names every run another engine, case, scenario or product flew (all 448 committed runs today: they predate provenance). **Not done:** the full re-fly of the committed results (wave 2, with Phase 2). (M)
5. ❌ Repository weight: the V&V PDF tracked twice; duplicate data files (DTCFILE, SOLFSMY, two 1.4 MB HTML); `de440s.bsp` 32.7 MB; 6,600 committed run files. (S, owner decides)
6. 🟡 Twin runs go through the same store rules: listed, queried and thinned like engine runs (`thin` removes `rec.mat` and `run_*.mat` too). **Not done:** the twin records no source fingerprint yet, so `stale` can only say so. (S)

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

✅ The SQLite results index is built (Phase 7). The rest is correctly not started: nothing needs it yet.

✅ SQLite index: built (queries were wanted; see Phase 7). ❌ Build when the trigger is met: Parquet for large series; attitude animation and the 3D geometry viewer (these become needed with Part B7 items 6–7).

---

# Part B — extra work TRI-NETRA needs

## B1. Flight software safety (L)
1. ✅ ✔ Decoded parameters trusted after the CRC: counts and indices up to 255 overrun C arrays and NaN/0 reach the laws. **Done:** a generated `validate()` in C and Rust from `params.toml`; `adcs_fsw_init` refuses an invalid blob (-12) and names the field.
2. ✅ ✔ No NaN guard before actuators (NaN→int undefined in C, 0 in Rust). **Done:** a non-finite coil, rotor or gimbal command drives 0, a non-finite valve duty keeps the valve closed, in both builds, with a test each.
3. ✅ Divisions without guards. **Done:** orbit acceleration at |r| ≈ 0, a GNSS fix inside the Earth (dropped, the orbit propagated), the rate hand-over at zero field, a thruster couple with no torque about its axis; the rest were already guarded or divide by a parameter `validate()` now keeps positive.
4. ✅ ✔ Sensor dropout ignored. **Done:** a field reading outside 0.25–4 × the model field is a stuck or dead magnetometer; the last good field is held, used by the coils for at most two coil cycles, never fed to the MEKF, TRIAD or the coil-cycle sums; B-dot falls back to the field-derivative law while the gyro is not fresh.
5. 🟡 Safe mode and FDIR. **Done:** a magnetometer or gyro silent 60 s raises a fault bit (8, 9) and holds magnetorquer detumble until it answers; a Rust panic resets the Cortex-M (SYSRESETREQ) or aborts on a host; a state outside the table commands no dipole. **Not done:** the watchdog kick (needs a HAL call, with the OILS board in B5). Invalid attitude is deliberately not a trigger: a pointing state starts without attitude at every eclipse start, and the magnetic hand-over already covers it.
6. ✅ Estimator guards. **Done:** a singular innovation covariance or a NaN χ² rejects the update; TRIAD refuses pairs within 0.06° of parallel; a star-tracker quaternion off unit length is not a reading. The star-tracker innovation gate was tried and left out: after a slew or a coast the covariance is too small and the gate rejected the updates that correct it (knowledge 0.004° → 0.77° in `fault_wheel_img`); gating waits on a consistent filter (B3).
7. ✅ Telecommands. **Done:** a state the fitted hardware cannot fly is refused by telecommand (-3) and at init in the start, next or schedule (-13): fine states need rotors or thrusters, `SUN_ACQ_ROTOR` rotors, `DETUMBLE_RCS` thrusters.
8. 🟡 Resources. **Done:** `link.ld` reserves 32 KiB of stack and the linker refuses a `.data + .bss` that leaves less; `tools/fsw_stack.py` walks GCC's call graph (deepest path about 4.9 KiB, 15 %) and is a `check_all` check; the determinism flags are always on whatever `CFLAGS` a caller passes. **Not done:** one libm for both firmware builds (C = Rust is bit-identical on all 40 scenarios today, so nothing differs yet).
9. ✅ The OBC link (`adcs_link.c`, `link.rs`). **Done:** the firmware bounds every read by the frame length and answers every frame it cannot take (bad CRC, over length, short, over capacity, unknown type) with a refusal code instead of silence; the engine checks each reply's type and length before anything reaches the bus, waits at most `ADCS_LINK_TIMEOUT_S` (60 s), refuses to drop CAN frames beyond 32, stops a silent OBC rather than waiting on it, and names the link fault in the run's error.

**Proof:** all 40 shipped scenarios fly byte-identical channels before and after, in the C and the Rust build; C in-process = both POSIX OBCs = both QEMU Cortex-M4 firmwares, bit for bit; a test per rule in C (45 checks), Rust and Python (`tests/test_link.py`).

**Done when:** fuzzing (Phase 2.4) runs clean; a test per FDIR rule flies the fault and sees safe mode; C = Rust stays bit-identical.

## B2. Design (L)
The design loop runs end to end and picks an actuator family; it does not yet do an analysis-level ADCS design.
1. 🟡 **Requirement flow-down and traceability**: tag each case key and metric with its spec row id; ✔ 26 of 58 case keys are never read (e.g. `req.rpe`, `req.pde`, `req.dump`, `req.prop`, `req.faults`, `req.recover`, `mission.duty`, `mass.iunc`, `magnetic.dunc`, `flex.*`, `resources.*`, `pointing.et`) — each gets code or is refused as unsupported.
   **Done (today's count: 22 of 71 template keys have no reader):** the requirements among them (`req.rpe`, `req.pde`, `req.rke`, `req.dump`, `req.prop`, `req.faults`, `req.recover`, …) are covered by `trace --check`: a stated requirement must be judged by a shipped metric. The rest are refused by name when a case states them, engine and twin (`CASE_UNMODELLED`: `mission.duty`, `mass.cm`, `resources.vbus`, `resources.nif`; `mass.iunc` and `magnetic.dunc` now set the design loop's Monte Carlo, B2.7), and so are the two the facet model cannot honour: a `surface.cps` that differs from `surface.cpa` (one centre-of-mass offset serves both torques) and a `surface.asun` other than the class body's largest face (no deployables). Both shipped cases state cps = cpa and asun = the 10 × 34 cm face, so nothing they state was dropped. `resources.malloc/palloc/valloc` are now read by `select` (B2.4). **Owed:** code for each refused key (the duty cycle, a CM offset of its own, the bus voltage, the data interfaces).
2. 🟡 **Pointing error budget** (spec rows gp_0–gp_5, no code): knowledge + control + alignment + thermal + jitter, allocated against APE/RPE; ✔ jitter returns NaN (`metrics.rs:165`) — port `jitter.m`.
   **Done:** `tools/pointing_budget.py` → `results/POINTING_BUDGET.md`: each fine-pointing scenario flown once; knowledge (gp_0) and control (gp_1, inferred) from the flown loop, payload alignment (gp_2, the product's `payload_alignment_rad`), thermal (gp_3, the case's `pointing.et`, no longer refused), jitter (gp_4, the engine's), the RSS total (gp_5, the R-15 independence caveat stated) and the room req.ape leaves for alignment and thermal. Today every imaging budget is **incomplete**: no product states its payload alignment and no case its thermal distortion; the room is 0.0073–0.0080° on the wheel and fluid-ring fine holds, none on `target_img`, and not computable on the CMG and VSCMG holds (their rotor parts state no imbalance, so no jitter). **Not done:** the RPE side (gp_4 against `req.rpe`, blank in both cases).
3. 🟡 **Lifetime environment and momentum**: sweep beta angle and season, solar cycle over `mission.life`; replace the arbitrary `0.25` secular factor (`adcs-design/src/lib.rs:124`); momentum dumping sized from `req.dump`/`req.hsat`.
   **Done:** the demand survey flies four seasons × the long-term low and high solar activity (F10.7 65 and 250 sfu, ECSS-E-ST-10-04C) and sizes for the worst, each recorded (`survey_sweep`); the secular momentum held between dumps is `req.dump` hours of it (`req.hsat` already sets the margin). On `ais_3u` the design momentum grows 21 % and the weakest field drops 8 % (season, not solar activity, drives it at 550 km). **Not done:** both cases leave `req.dump` blank, so the quarter-orbit default still applies (recorded in the survey's notes); the solar cycle is bounded, not followed date by date over `mission.life` (blank in both cases).
4. 🟡 **Power, thermal and data budgets**: orbit-average and peak power with duty and eclipse, checked against `resources.*` in select (only mass and volume are checked today, `pipeline_verify.py:32-35`).
   **Done:** `select` checks the family budget against the platform's allocations `resources.malloc`, `resources.valloc`, `resources.palloc` whenever the case states them (both cases leave them blank today); the node verifier recomputes the same gaps; orbit-average and peak power are judged per mode by the flown `power_mean`/`power_peak`, and the generation side by B3.2's `power_margin`. **Not done:** thermal and data budgets (`resources.nif`, `resources.vbus` are still unread).
5. **Sensor trade**: choose sensors from a catalogue against the knowledge allocation (every product gets the same fixed suite today, `lib.rs:405-409`).
6. 🟡 **Redundancy and FDIR in selection**: fault campaigns count toward feasibility.
   **Done:** node `faults` (`tools/pipeline_verify.py`; set, rule and policy in `nodes.json` `faults` and `select.fault_policy`): once the loop has converged, each solution family's mission (as dispatch builds it) flies fault-free and once per single fault its product carries: one coil, one rotor (a fluid ring is a rotor), one gimbal, the second star-tracker head when there are two, a 0.1°/s gyro bias step, a 1800 s GNSS outage, one RCS valve; each injected a quarter orbit before the requirement window (the last half orbit), seeds 1 and 2. A fault fails when a requirement the fault-free run meets is missed under it, and is a gap `fault: <kind>: <metrics>`. Select then runs again, before dispatch: `gap` (the default) makes such a family infeasible; `rank` keeps it feasible and puts the fewest failed faults first. `verify_nodes.py` recomputes the set from each product, every verdict from the stored metrics, the gaps and select's count (`tests/test_faults.py`, `tests/test_verify_nodes.py`). Flown on the stored converged selections (this commit's engine, outside the loop): on `ais_3u` `mtq_fmr` survives all four of its faults and stays selected; on `ais_img_3u` `mtq_fmr` and `mtq_fmr_rcs` both lose fine pointing when one fluid ring fails (APE 82° and 1.1° against 0.01°), so under `gap` that case has no feasible solution family and `mtq_fmr` is named closest. The default is `gap` because `ais_3u` keeps a feasible family under it; the `ais_img_3u` miss is a finding, not a reason to relax the rule: three rings, one per axis, leave no spare axis.
   **Not done:** the stored selections predate the node (the verifier notes this; the next design-loop run flies it); no knob answers a fault gap (a fourth, skewed ring or wheel is not a sizing option); the benchmarks are not flown under faults (`faults.roles`); `mag_fail` is left out of the set because a silent magnetometer puts the flight software in safe mode by design (B1.5).
7. 🟡 **Beyond 3U**: box size, class and survey product from the case (`lib.rs:80,339,412`); Monte Carlo dispersions generated from the case, not looked up for two known cases (`pipeline_verify.py:147`); products of inertia in the case.
   **Done:** the parts are sized on the case class's body (`catalogue/classes.toml`; identical for the shipped 3U cases); the design loop's Monte Carlo is generated from the case (`case_dispersions`: the orbit draws centred on `orbit.ltan`/`orbit.alt`, the inertia and residual-dipole spread from `mass.iunc`/`magnetic.dunc` when stated, else `catalogue/dispersions.toml`, the values the shipped campaigns fly; the Kp draw now applies to both cases). **Not done:** the survey still flies with the TRN-P-3U-AIS product (only its disturbance torques are used, which do not depend on the product); products of inertia are drawn, not stated in the case.
8. 🟡 **Sizing laws** documented with sources and margins (coil 0.5·B_min/0.3·B_mean, RCS Isp 60 s, slew propellant left out, `lib.rs:175-297`); fix the selection-order mismatch (`pipeline.py:18` "simplest" vs "lightest").
   **Done:** `docs/SIZING_LAWS.md` states every sizing law with its constants, margins and blank-case defaults, and marks each constant that has no stated source (seven of them) for the owner to confirm. Every description of `select` now states the rule the code applies (least mass, then power, then volume; `nodes.json` `select.rank_feasible`). **Owed:** SPEC §8.6 ranks by worst margin first.
9. **The spec package**: about 90 of 243 system rows have code and none is linked by id; a spec → code → test matrix, and `spec/` brought up to date.

## B3. SILS (L)
1. ✅ **ECSS-E-ST-60-10 pointing metrics**: APE, RPE, AKE, MKE, PDE, PRE with windows and the temporal/ensemble/mixed interpretation and confidence level, in engine and twin.
   **Done:** `metrics::ecss` (rpe/mpe/pde, rke/mke/kde and their line-of-sight forms, `delta_s` and `separation_s` checked by the schema), the twin's `+metrics/ecss.m`; the campaign ledger states the temporal, ensemble and mixed interpretation of each claim (`tests/ecss.rs`).
2. ✅ **Power model**: solar arrays from attitude and shadow, battery state of charge, eclipse depth of discharge.
   **Done:** `metrics::PowerSystem` from the case's `power` rows (all or none; the two cases carry UNCONFIRMED 3U values), the `P_gen_W` and `soc` channels, metrics `power_margin`, `battery_dod`, `soc_min`, engine and twin (`tests/power.rs`).
3. ✅ **Monte Carlo statistics**: ✔ 15–40 runs per campaign cannot support 99.73 % claims (about 1,100 failure-free runs needed at 95 % confidence); run counts sized for the claim; disperse epoch, season, beta, products of inertia, latency, orbit; the same inertia knowledge in the flight software on both sides.
4. ✅ **Missing dynamics**: wheel imbalance and jitter in the loop; one or two flexible modes; fuel slosh when RCS carries propellant.
   **Done:** one flexible mode in hybrid coordinates (case `flex` rows; momentum conserved, free-free period and damping tested, engine = twin step to 1e-14); rotor imbalance jitter computed in the frequency domain from each rotor part's stated imbalance (a time-domain model at the 10 Hz loop would alias it; unstated imbalance: not computed). **Decided not modelled:** slosh: the RCS carries 0.05 kg of propellant against a 4.0 kg body (case `mass.m`, UNCONFIRMED).
5. 🟡 **Device fidelity**: magnetorquer RL dynamics, hysteresis, eddy currents; wheel stiction/Dahl friction, back-EMF, speed limit; star tracker Moon exclusion, blinding recovery, rate-dependent noise; GNSS latency and outages; outgassing, albedo and IR pressure torques; self-shadowing of appendages. **Done** (engine and twin, every value from the part, refused by name when not stated): the coil's dipole lags its saturated command by its L/R time constant (`time_constant_s`; exact over a step); the wheel's motor torque-speed line from k_t, R and the supply (back-EMF), the drive's speed limit, and stiction (Stribeck excess over Coulomb, Karnopp stick at rest: `friction_static_Nm`, `stribeck_speed_rad_s`); the star tracker's Moon exclusion, blind time after the Sun or Moon leaves its cone, noise growing with body rate (`noise_doubling_rate_rad_s`); the GNSS fix of `latency_s` ago; Earth albedo and infrared pressure on the facets (annual-mean albedo 0.30 and 237 W/m², nadir point source). Closed-form tests in both. The flight software treats a fix as current: at 0.1 s the along-track lag (760 m) puts the pointing reference 0.006° off and sixteen fine-pointing verdicts at 0.01° now fail (fifteen pass again with `latency_s = 0`; in `mission_rw_rcs` the wheel stiction adds as much again); `sun_spin_ais` (seed 1) loses its spin, as it does at seed 3: its capture was already marginal (B3.7). **Then:** the flight software carries a fix forward by the receiver's stated latency (new parameter `gps_latency`: the ECEF fix rotated at its own epoch and propagated by one two-body + J2 Verlet step; C, Rust and twin; 0 keeps the old behaviour bit for bit). Fifteen of the sixteen pass again, C = Rust bit-identical; `mission_rw_rcs` (0.0149°) is the wheel stiction: its Z wheel sits within 2e-5 N m s of rest (inside the Stribeck band, J·ω_s = 1.6e-5 N m s) for the whole last half orbit while X and Y hold ±1.1e-3, because the RCS dumping only acts outside its 1–4 mN m s band and never holds a bias on that wheel; owed: a bias the dumping holds on every wheel, or stiction in the driver's friction compensation. **Not done:** hysteresis and eddy currents (the coils are air-core), Dahl pre-sliding, outgassing, self-shadowing; the VSCMG rotor keeps the plain wheel model; latency compensation in the flight software.
6. 🟡 **Scenarios**: target pointing, Sun modes (`sun_mtq`, `sun_fine`), RCS detumble and rotor Sun acquisition as start modes, safe-mode entry and recovery, eclipse transitions, magnetometer/GNSS/RCS-valve faults.
   **Done:** eight new scenarios (`safe_mode_ais`, `target_img`, `sun_fine_img`, `sun_mtq_ais`, `detumble_rcs`, `sun_acq_rotor_img`, `fault_gps_img`, `fault_valve_rcs`); a fault may now end (`faults[].end_s`, magnetometer and GNSS only, engine and twin), so safe mode is flown in and out. Every new metric is judged against a stated requirement or says why it only reports. What they found (engine, seed 1; each is a finding, not a scenario error):
   - `sun_fine_img` passes; `sun_acq_rotor_img` acquires the Sun in 2.2 min.
   - `safe_mode_ais`: safe mode at 101.0 min (fault + 60 s), recovery to nadir hold at 111.0 min; the last-orbit APE (17.5°) misses the 10° `req.ape`, which coils-only nadir hold on this product also misses (`nadir_hold_ais`).
   - `sun_mtq_ais`: coils-only Sun referencing does not hold the Sun (p95 87° against the 20° guard), flown with the design loop's own law and start (`mtq_avanzini2021`, rate from guidance). The design loop agrees: it marks the coils-only family infeasible for Sun referencing on this case (`sun_ape_p9973`), and the selected AIS family carries fluid rings.
   - `detumble_rcs`: the thrusters take the 10°/s tumble to 0.7°/s in 60 s and then stall between 0.45 and 0.7°/s, never holding 0.5°/s. Cause: the 5 ms minimum impulse bit against the law's 20 s damping time and 1 s period gives a floor of mib·τ·T/(J·Tc) ≈ 0.4°/s about the long axes (5 ms · 3 mN m · 20 s / 0.042 kg m² · 1 s). Remedy for B2: hand over to coil B-dot below about 1°/s, or size the damping time from the impulse bit.
   - `target_img`: APE 0.0121° against 0.01°, AKE 0.0114° against 0.005°: target pointing misses the imaging budget, and knowledge is the larger part (B2 sensor trade).
   - `fault_gps_img`: marginal through a 30-minute GNSS outage: seed 1 misses (APE 0.0112° against 0.01°, the outage leg 0.0113°, AKE 0.0028° inside), seeds 2 and 3 pass (0.0080°, 0.0068°), the twin 0.0076°; the fault is injected and cleared at the same times on both (2000 s, 3800 s).
   - `fault_valve_rcs`: a stuck valve drives APE to 2.75°; the flight software has no thruster FDIR (B2 FDIR).
   **Not done:** eclipse transitions as their own scenario (every orbit-long scenario crosses eclipse; none judges the transition).
7. 🟡 **The 13 engine-vs-twin disagreements** fixed at their traced causes: FMR field-power switch hysteresis, RCS dump thrust-scale compensation, the Sun-spin sign-flip lock, the coils-only MEKF spread (`results/ENGINE_PARITY.md:228-232`).
   **Done:** the fluid-ring field-power switch has a hysteresis band (on above 2 %, off below 1 % of h_max; engine and twin). **Tried and not shipped:** an on-orbit thrust-scale calibration in the flight software (estimates 0.52–0.81 against a true ~1.0: the dump pulses do not excite it enough); the finding stands: inhibit RCS dumping in the fine modes or calibrate on a dedicated manoeuvre (B2). **Owed:** the Sun-spin hemisphere manoeuvre (3 of 12 seeds miss 95 min, 2 of 12 end above 20°); the SMC tuning (seeds 0.0126–0.0292° against 0.01°).
8. 🟡 **Port to the engine** what only the twin has: star-tracker image chain, CSS chain, jitter.
   **Done:** jitter (B3.4). The star tracker's `model = "image"` (render → centroid → identify → QUEST with the residual check; `adcs-sim-core::comp::star_tracker`, no allocation: the frame, scratch, pair table and votes are the run's buffers) and the Sun sensors' `level = "chain"` (quadrant currents → angles; `comp::sun_sensor`), selected by the same product fields as the twin. Every camera and head value comes from the part (`detector_px`, `psf_sigma_px`, `flux_mag6_e`, `background_e`, `read_noise_e`, `centroid_k_sigma`, `max_spots` ≤ 32, `id_tol_rad`, `id_mag_tol`, `fit_tol_rad`; `aperture_side_m`, `aperture_height_m`, `current_noise_frac`, `current_min_frac`), each refused by name when not stated, in the engine and the twin (the twin's `camera`/`head` baselines stay for its unit tests only); an unknown model or level is refused (before, the engine flew `"image"` as `"quest"`). The twin's device-level `image` model failed on a wrong field (`D.st.K`); fixed. Tests: centroids within 0.15 px noise-free and 0.25 px with noise, attitude within 20″ noise-free and 60″ with noise, identified ids are the rendered stars, a hot pixel is dropped, currents → angles exact (1e-7 rad) over ±39°; engine = twin on the same noise-free frame (pair table exact, 20 spots within 1e-9 px, attitude within 1e-6″, currents within 1e-15), in both suites; a 40 s `fine_hold_img` on both chains answers on every frame the star-field model does, knowledge 0.0031° (star-field 0.0027°). No shipped part states the camera or head values and no shipped product selects either level, so no shipped scenario changes. **Not done:** a shipped part with the chains' values (the in-house heads are not characterised); the image model costs about 30 ms a frame per head; the earth-sensor chain (`asils.comp.earth_sensor`) is not wired into the twin's device either.

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
1. ✅ **Traceability matrix**, generated (`tools/trace.py` → `results/TRACEABILITY.md`): every stated requirement of each case → what checks it (a flown scenario's metric, the design loop's budget or its mode flights, the reference slew's profile) → the latest stored verdict. All 19 stated requirements are checked. The 66 metrics that judged nothing now each say why they only report (`diagnostic`, engine and twin); `req.slew` is checked by the slew scenarios flying `mission.sangle` in `req.slew` (their settle and APE are then judged); `req.sunacq` and `req.ppk` on the imaging case by the design loop's mode flights. `trace --check` is in `check_all`.
2. 🟡 **Propagator, outside validation.** ✅ The time and frame kernel against ERFA's (SOFA's) published test vectors (`adcs-pop/tests/published.rs`). ✅ IGRF-13 against an independent implementation (pyIGRF), which found the table ported from the Standard Code wrong by up to 75 nT; the IAGA coefficient file is now the one source of the table. ❌ Owed: a real satellite's precise orbit as a ledger, and the atmosphere models' published reference outputs (neither was reachable from the build environment).
3. 🟡 **Algorithms against outside references** (`docs/references.toml`): ✅ LQR (SciPy's Riccati solver, the closed form), B-dot gain (Avanzini & Giulietti 2012), QUEST (SciPy's SVD solution), TRIAD, the MEKF's equations (Markley & Crassidis 2014) in closed form, guidance frames. ❌ Owed: an MEKF consistency (NEES) test (with B3), and one published case from the magnetorquer papers.
4. ❌ **Catalogue**: each datasheet number traced to its page; the 14 synthetic parts replaced by real units (owner's choice of parts).
5. ✅ **`status` headline** extended: requirements nothing checks, requirements not met, undecided metrics, models without an outside reference, stale stored runs.
6. Owner work: confirm the 31 algorithms (all UNCONFIRMED), choose real parts; decide IGRF-14 (the mission epoch, 2027, is past IGRF-13's 2025: the field is extrapolated from the 2020–2025 secular variation).

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
| Safe-mode policy (B1.5) | as built: magnetorquer-only detumble after a magnetometer or gyro is silent 60 s; invalid attitude is not a trigger (see B1.5) — owner to confirm |
| Lesson pages (Phase 8) | yes, B-dot first |
| Retention of time series (Phase 7) | 30 days |
| `de440s.bsp` and committed run files (Phase 7) | fetch by checksum; keep ledgers, thin runs |
| Reference OBC board for OILS (B5.2) | an STM32F4 class board, or the owner's flight OBC |
| HILS equipment available (B6) | owner lists what exists (cage, air bearing, Sun simulator) |
| Does v1.0.0 wait for B5/B6 (hardware-paced)? | release after waves 1–5 and 7; OILS/HILS in v1.1 — owner's call |
