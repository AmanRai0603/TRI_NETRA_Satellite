# TRI-NETRA: the software plan to v1.0.0

> **Answer first.** v1.0.0 sets the architecture: the structure, templates, data and process the
> codebase is built on, with its known ADCS gaps stated rather than hidden. This file is the
> **release gate**: the release waits for every phase below. The ADCS technical work (design,
> fidelity, validation, hardware) is **not** a release gate; it lives in
> [`ADCS_GAPS.md`](ADCS_GAPS.md), the register of what to study and resolve after v1.0.0.
>
> **Kind:** plan · **For:** the owner (Agastya) and whoever does the work · **Status:** open, waiting
> for the owner's additions (2 Oct 2026)

Legend: ✅ done · 🟡 partly · ❌ not done · ➕ owner's addition. Sizes: S ≤ 1 day, M 2–4 days, L a week
or more. Item numbers in brackets point to the evidence in [`UPGRADE_PLAN.md`](UPGRADE_PLAN.md),
which stays as the record of what was found and done.

## Already done (the base v1.0.0 builds on)

Phase 1 safety and correctness · Phase 2 tests that find faults (unit, end to end, fuzzing, mutation,
twin) · Phase 3 structure (splits, one flight pipeline, typed errors, one ROOT) · Phase 5 failure
handling · B1 flight-software safety (all but the watchdog kick and one libm, both waiting on a
board: see `ADCS_GAPS.md` H) · B8 traceability matrix and `status` headline · B4 soft-OILS deadlines,
CPI sweep, `-icount` · the results database (SQLite index, queries, retention, `refly`) · the
release workflow, wheel and desktop app (signing-ready).

---

## S1. Foundation: one version, reproducible builds, process (M)

The release must prove what CI proves, from sources anyone can rebuild.

1. ❌ **One version source**: `VERSION` 1.0.0, Cargo 1.2.0, a hand-written `ENGINE` string and zip
   names mixing both today; one file, every build reads it, `prove` checks it. [4.2] (S)
2. ❌ **Reproducible builds**: `--locked`, `rust-toolchain.toml`, GitHub actions pinned by SHA,
   `setup-python` in every job, pinned `pyflakes`. [4.3] (S)
3. ❌ **The release proves what CI proves**: `prove` runs the twin and the QEMU parity; the tagged
   commit must be green; a check reported NOT RUN fails the release. [4.1] (S)
4. ❌ **Branch protection** and one human review before merge. [4.4] (S, owner's setting)
5. ❌ **Licence**: NOTICE.md says all rights reserved while the release publishes downloads; a
   matching `LICENSE`, or publish to collaborators only. [4.6] (S, owner decides)
6. ❌ **One docs path**: 23 overlapping documents (ARCHITECTURE_PLAN, DESIGN_LOOP,
   SOLUTION_PIPELINE, NODES tell one flow; SOFT_OILS and VIRTUAL_OBC in two places); one index,
   one document per subject, stale text removed. [4.5] (M)

➕ Owner's additions:

## S2. Single sources: constants, units, shared logic (M)

Every number and every rule written once.

1. ❌ **Constants and units from one place**: Earth radius and μ repeated as literals
   (`config.rs:212-213`, `run.rs:277`), height in km in one function and m in another
   (`field.rs:44,111`). [9.5] (S)
2. ❌ **Unit types** for quantities that cross module boundaries (length, angle, rate, field). [10.3] (M)
3. ❌ **Logic written once**: Kp→ap (three places), campaign draws (Python and MATLAB), statistics,
   the pass rule (three), scenario building, orbit period (three). [9.3] (M)
4. ❌ **No typed-in numbers** in figures and report text: requirement lines, thresholds, literature
   numbers, versions and epoch read from the case, the runs or `VERSION`. [9.2] (S)

➕ Owner's additions:

## S3. Scripts you can follow (M)

Every tool says what it will do, what it did, and how to undo or retry it.

1. ❌ **`--dry-run` on every tool that writes** (about 15: `report`, `vv_report`, `pack_*`,
   `gen_fsw_params`, `export_catalogue`, `catalogue`, `floquet`, `kit`, `macapp`, `make_icon`,
   `nodes_doc`, `components_doc`, `run_matrix`, `verify_nodes`, `build_wheel`); argparse for the two
   that work at import. [6.1] (M)
2. ❌ **On failure**: which step stopped, whether files are unchanged or put back, the exact
   command to retry. [6.2] (M)
3. ❌ **Registry fields** for what each command checks, how to undo it and where its code is, held
   true by a test. [6.3] (S)
4. ❌ **`trace` and `why` with history**: the last run shown; which run wrote a file, when, from
   which inputs. [6.4] (S)
5. ❌ **Numbered steps** in every tool that runs for more than a moment. [6.6] (S)
6. ❌ **The journey diagram**: edit → generate → check → fly → verify → PR → CI → release. [6.5] (S)

➕ Owner's additions:

## S4. Data: storage, database, results (M)

What is stored, how it is found, and that it matches the engine that produced it.

1. 🟡 **Stored results match the engine**: every run records its engine fingerprint and `adcs
   results stale` names the rest. **Not done:** the re-fly of everything stale after the rotor-FDIR
   change (engine scenarios and campaigns, the `ais_3u` design loop, soft OILS in batches, the twin).
   The `ais_img_3u` design loop is re-flown (`c88370c`). [7.4] (M, machine time)
2. ❌ **Twin runs carry a source fingerprint**, so `stale` judges them like engine runs. [7.6] (S)
3. ❌ **Repository weight**: the V&V PDF tracked twice, duplicate data files, `de440s.bsp` (32.7 MB)
   fetched by checksum, committed run files thinned to ledgers. [7.5] (S, owner decides)
4. ❌ **Output types**: figures also as SVG/PDF; a report per run. [7.3] (M, with S5)

➕ Owner's additions (database and storage):

## S5. Templates: one look, one plotting path, data-driven pages (L)

Every page, figure and report built from one set of parts, so the code underneath can change without
touching how results are shown.

1. ❌ **One plotting module** for engine and twin runs: one palette, one figure numbering, the full
   11-mode axis; the report reads the engine store too (it reads only the twin's today,
   `report_base.py:123`). [9.1, B7.2] (M)
2. ❌ **One component set** (HTML, one script, one stylesheet, bundled fonts): figure, table,
   equation, claim tag, references and the lesson parts; every page assembled from it; a check that
   refuses hand-built components. [8.1] (L)
3. ❌ **Pages work offline**: no outside hosts (the report loads Google Fonts today); figures dark-mode
   aware. [8.4, B7.7] (S)
4. ❌ **Report template per run** with SVG figures (S4.4). [7.3] (M)
5. ❌ **The V&V report's verification matrix from data**: requirement → method → run → verdict, open
   items from a data file, not code (`vv_report.py:533-567`); stale cross-references fixed. [B7.3] (M)
6. ❌ **`lesson.toml`**: the template for documenting a mode or algorithm (B-dot first), outside the
   result fingerprints. [8.2] (M)

➕ Owner's additions (architecture and templates):

## S6. Readability and speed (S–M)

1. ❌ **A one-page readability guide**: how code and docs are written here. [10.1] (S)
2. ❌ **Remaining function splits and typed errors** (tracked from Phase 3). [10.2] (M)
3. ❌ **Speed and memory**: per-tick allocations (`run.rs:140`), unbounded soft-OILS statistics
   (`run.rs:338`), a shifting 64-entry star-tracker history (`sensors.rs:156`); a 30-day run's
   memory measured and bounded. [10.4] (S)

➕ Owner's additions:

## S7. Release v1.0.0 (M) — the last step

1. ❌ **The wheel tagged per platform** (tagged `any` today but carries native programs), installed and
   run on Windows and macOS; macOS binaries signed like the kit's. [B9.1] (S)
2. ❌ **C/Rust parity on Windows (MinGW) and macOS in CI**, not only at release. [B9.2] (S)
3. ❌ **Release notes generated** from `VERSION` and from `ADCS_GAPS.md` (the known gaps shipped with
   the release). [B9.3] (S)
4. ❌ Merge PR #12, tag `v1.0.0`, watch every job, publish. [B9.4]

---

## Proposed for after v1.0.0 (software, not gating)

These are software work too, but large and not needed to fix the architecture. The owner may move
any of them into S1–S7.

| item | why later |
|---|---|
| The app shows time series (attitude error, rates, modes, momentum, power, zoom) [B7.1] | builds on S5's plotting module |
| Pointing-budget figure, mission timelines, eclipse bands on every time plot [B7.4] | builds on S5 |
| Monte Carlo envelopes and run-to-run overlays [B7.5] | builds on S5 |
| "Try it" widgets on the engine compiled to WebAssembly [8.3] | L; needs the component set first |
| A lesson form with preview through the review flow [8.4] | needs `lesson.toml` first |
| 3D attitude and geometry viewer, Parquet for long series [B7.6, Phase 11] | built when first needed |

## Order

S1 → S2 → S3 → S4 → S5 → S6 → S7. S1 first because every later change is then built, versioned and
proved the same way; S2 before S5 because templates should read single sources; S4's re-fly runs in
the background throughout. Each phase ends with `check_all` and CI green and this file's ticks
updated.

| phase | about |
|---|---|
| S1 foundation | 3–4 days |
| S2 single sources | 3–4 days |
| S3 scripts | 3–4 days |
| S4 data | 2–3 days (+ machine time) |
| S5 templates | 1.5–2 weeks |
| S6 readability | 2–3 days |
| S7 release | 2 days |

About five to six weeks, before the owner's additions.

## Decisions the owner makes

| decision | suggested |
|---|---|
| Licence of published downloads (S1.5) | decide before S7 |
| `de440s.bsp` and committed run files (S4.3) | fetch by checksum; keep ledgers, thin runs |
| Retention of time series | 30 days (as built) |
| Lesson pages in v1.0.0 (S5.6) | the template yes; the lesson pages after |
| Items moved from "after v1.0.0" into the gate | owner's call |
