# Release: ADCS platform build package 1.0

**In one line:** the final planning release of the build package. It adds the MATLAB twin's lockstep with the platform and every OILS and HILS node the architecture needs, and it passes every check the package carries.

Date: 2026-09-27. It goes into the new repository at `_package/`, read-only (SPEC.md §0.1).

## What this release adds

**The MATLAB SILS twin, in lockstep from P1** (SPEC.md §10.8.7)
- `plan/twin_map.toml` lists every SILS element: 99 today, from four registries (39 physics functions, 4 algorithms, 25 metric kinds, 11 part kinds) and 20 single elements.
- For each element it names the platform side, the MATLAB side, the phase that writes both, the rungs that reuse it and the test both must pass.
- Three elements are one-sided by design, and each says why.
- `tools/twin_check.py` stands in for `cargo xtask twin check`, rules TW01–TW06. It checks that the map is whole, that a checkout has both sides of everything its phases build (P3M runs beside P5–P7), and that a pull request changes both sides or carries `twin:none` with a reason.
- `tools/pack_matlab.py` puts the map and a `TWIN.md` in the zip. With `--phase`, it refuses a zip that is missing a twin those phases build.
- §19 is rebuilt so each phase writes its SILS elements in both engines:
  - P1: physics and the case reader;
  - P2: product supply;
  - P3: environment, plant, devices, flight software, metrics, recorder, runs and results;
  - P4: the campaign runner and the tuner.
- P3M keeps only the app, the figures, the examples and parity.
- The `twin`, `matlab-pack` and `matlab-twin` CI jobs run from P1.
- An intake brief for a new physics function now asks for its MATLAB twin in the same commit.
- New developer manual page: `manual/developer/09_twin.md`.

**OILS and HILS, as nodes** (SPEC.md §5.3, §8.2, §12.2, §12.10)
- Layer 1's test facility gains six rig groups, with 42 measured capabilities read from the lab file by the new `lab` supplier, plus three declared policy rows:
  - OILS rig;
  - magnetic field simulator;
  - air-bearing platform;
  - optical and RF stimulators;
  - actuator test stands;
  - rig safety and power.
- A `nan` in the lab file answers `NotMeasured`, by name.
- Layer 2 gains "OILS rig needs" (5 rows) and "HILS rig needs" (16 rows, with the actuator rows tagged per family). The subsystem layers gain the OILS and HILS rigs.
- `adcs rig fit` (§12.10) compares the case's needs with the lab's measured capabilities line by line. `adcs rig arm` refuses a short fit. It is built in P6 (OILS) and P7 (HILS).
- `rig/labs/hils_bay1.toml` describes the real bay, and every value is `nan` until it is measured. `rig/labs/syn_lab.toml` holds stated synthetic values for tests.
- Ten facility references are added to §22 and to `plan/seed_content.toml`: Schwartz 2003, Ibrahim 2026 (a preprint), da Silva 2019, ASTM E927-19, Zhao 2024, Bremer 2017, Colagrossi 2023, Nadeem 2022, Springmann 2010 and ECSS-E-TM-10-21A.

**The ledger**
- B-015 is the belief that two hand-kept engines stay in step under the lockstep. It is untested and bears on R-10.
- B-016 is the belief that rig needs compared with lab capabilities are enough to book a rig. It is untested and opens R-18, "The lab cannot meet a case's OILS or HILS needs" (L3, proposed).
- `derisk record --risk` names a risk without moving it.

## The package now holds

| | |
|---|---|
| Tree | 418 rows and 307 edges, with one door and 14 subsystem layers. Once seeded it gives 734 sheets. |
| Case format | `adcs-case/1`: 54 inputs and 5 meta rows |
| Seed content | 82 seed forms, 26 cited sources and 7 transcribed test vectors |
| Ledger | 18 risks (levels proposed until D26) and 16 belief records |
| Twin map | 99 elements |
| Intake | 66 deliberate mistakes refused by code, 88 clean requests and 82 seed sheets |
| Validator | 28 deliberate breakages caught |

## Checks run for this release

Every command in README's "Check the package" was run on the final tree. Each result below is the command's own output:

- `validate_plan.py`: 0 findings, 418 rows. `--selftest`: 28 mutations caught.
- `build_tree.py --check`: current.
- `intake.py selftest`: 66 mistakes refused, 88 clean requests passed, 82 sheets verified.
- `derisk.py check`: 0 findings, 18 risks, 16 beliefs. Its `selftest` passed, and `table --check` reported the table current.
- `twin_check.py`: 99 elements, 96 on both sides and 3 one-sided by design. Its `--selftest` passed.
- `manual_pages.py --check` and `explain_kit.py --check`: current. `explain_check.py`: 19 manual pages and 12 rendered pages. Its `--selftest` passed.
- `check_case.py` on the four reference cases: passed. `form_browser_check.py` in headless Chromium: passed.
- `assemble_spec.sh --check`: current.
- `pack_matlab.py` built twice gave identical bytes. `--phase P1` refuses, naming the missing twins, which is correct because no MATLAB code is written yet.

An independent review read the lockstep, the rig nodes and the ledger without having seen them written. It found twelve issues, all fixed in this release. The main ones:
- the phase ordering around P3M;
- the §12.10 comparison senses;
- two unmeasured lists in the real bay's file;
- how one-sided changes were matched;
- a source title;
- a broken table.

## What this release does not do

- No MATLAB or Rust code exists yet. The zip says "0 of 97" in `TWIN.md` (the 97 elements with a MATLAB side), and the twin is written phase by phase, starting in P1.
- The rig-need rows and the facility rows have no content. They arrive through node forms and the lab's bring-up records.
- Every risk level is a proposal until D26. D5 (the MATLAB licences, now needed from P1) and D14 (the IEU) are decisions for a person.
