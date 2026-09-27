
---

## 19. The build, phase by phase

Each phase ends on a gate, not on a date. A phase is done when every command in its **Green** line passes. Its **Acceptance** is observed and recorded in `docs/BUILD_EVIDENCE.md` with the real output pasted, not paraphrased. At its **Stop points** a person decides before the next phase starts. Every phase ends with a report in `docs/phases/P<n>.md`: what was built, what is mocked, what was not done, and what a person must do next (rule 9).

The order is by dependency, not by number:

| Phase | Needs | Why |
|---|---|---|
| P1 | P0 | |
| P2 | P1 | screening writes candidate supplies on tree rows, over imported cases |
| P3 | P2 | a scenario's product is screened and supplied through `adcs-config` |
| P3M | P4 | the twin's app, figures and examples show campaigns and tuning, and parity compares them, which P4 builds |
| P4 | P3 | campaigns derive runs from scenarios |
| P5 | P4 | the dashboard reads campaigns |
| P6 | P4 | the rig reuses the loop engine, its emulators and the campaign runner, and writes ledger lines |
| P7 | P6 | HILS extends the rig |
| P8 | P7 | the package reads the ledger, and the ledger's HILS and lab-twin lines come from P7 |
| P9 | P5, P8 | quotes, the client views, the mock rig's witness stream, and the evidence page |

The MATLAB twin is not a phase of its own. Each phase from P1 writes its SILS elements in both engines, in the same change (§10.8.7): physics and the case reader in P1, product supply in P2, the environment, plant, devices, flight software, metrics, recorder, runs and results in P3, the campaign runner and the tuner in P4. So the zip `matlab-pack` builds on every push from P1 is whole for the phase reached. P3M adds only what the twin alone has (the app, the figures, the examples) and the parity job, and runs beside P5, P6 and P7. So once P3M and another of them are green, the `twin` and `matlab-pack` jobs are given both (`--phase P6,P3M`), and reaching P5 alone never asks for P3M's app. Each phase's **Green** repeats the Green of what it needs, not of the phase numbered before it.

### P0 — The platform, with an empty tree, and nothing that writes

- Create `ADCS_PLATFORM`. Copy per §3.4 and rename per §3.3. Remove every editing path §3.4 lists, completely (F17). Fix F1–F6, F8–F11 and F13–F15 (§3.5). F7 waits for D11 (P1); F12 is built in P4.
- The empty tree must pass the gate: V13 with an empty `DEMONSTRATION` list is a pass (F15), and the web face loads with no cases (F13, F14).
- Leave `layers/` holding only `root.toml` and `cycles.toml`. There are no node crates and no bundles.
- Write `AGENTS.md`, `intake/AGENT.md`, `CONTRIBUTING.md`, `README.md` and `areas/*.md` per §17. Write the scope hook (§17.4). Rewrite `docs/manual.toml` for what exists now.
- Copy `_package/manual/` to `manual/`, `_package/derisk/` to `derisk/`, and `_package/tools/{explain_kit,explain_check,derisk}.py` to `tools/`; run `tools/derisk.py table --file docs/RISKS.md` once, which writes the register's table into the repository (SPEC.md stays in `_package/`, read-only). Keep both manuals true from now on, written to the explanation standard (§5.12): a phase that adds a command or a view updates its page. Its command blocks are tagged with the phase that builds them (§16.4).
- **Green:** `cargo run -p xtask -- gate && cargo test --workspace`; `cargo clippy --workspace --all-targets -- -D warnings`; each selftest named in §18's `tooling` job (never a glob of `tools/*.py`); `selftest_panels`; `tools/instruction_lint.py` at 0 findings; `cargo build --profile user`; the `no-writes` job; the `derisk` job (`tools/derisk.py check`, `selftest`, and `table --check --file docs/RISKS.md`) and the `explain` job over the manuals (`tools/explain_check.py --md`; the rendered templates join it in P1).
- **Acceptance:** `cargo xtask status` reports 0 rows and names nothing as broken. The daemon starts on 7787, shows an empty tree with its manual, and has no route that writes: every `POST` it answers is listed, and none touches the repository.
- **Stop points:** D10 (shared platform or two repositories), D12 (teams, and the developer reviewer group behind each owner).

### P1 — Units, physics, the ADCS tree, and intake

- **Units and physics.** Add the units and quantities of §6.1 and the frame types. Write `adcs-core::physics` per §6.2, with property tests only: zero at count zero, monotonic where the relation is, and dimension checks. No expected values are invented.
- **The twin, from the start (§10.8.7).** Copy `_package/matlab_sils/`, `_package/plan/twin_map.toml` (with `plan/`, below) and `_package/tools/{pack_matlab,twin_check}.py`. Write `cargo xtask twin check|list`, porting `twin_check.py` rule for rule. Every physics function is written in the same change as its twin, `matlab_sils/+asils/+physics/+<module>/<name>.m`, and both run its fixtures; `asils.case.read` and `asils.case.template` are written with `adcs-case`. Add the CI jobs `twin`, `matlab-pack` (with `--phase` set to the phase reached) and `matlab-twin` (§18).
- **The tree's shape.** Copy `_package/plan/` to `plan/`, `_package/catalogue/` to `catalogue/` and `_package/designs/` to `designs/` (the case checker reads `meta.class` and `meta.families` against them), and `_package/tools/{build_tree,validate_plan,check_seed_with_vleo,plan_model}.py` to `tools/`. The validator checks only the directories copied so far. Write `tools/plan_rows.py` and adapt `tools/seed_tree.py` per §5.7, then seed. Every row is `seeded` and answers `NotRun`.
- **Check the seeding.** Assert that every node id equals `plan/expected_node_ids.json`. Generate `sources/`, `layers/` (every group's `cases` list empty, §5.6) and `CODEOWNERS`.
- **Cases.** Write `adcs-catalogue` (reading the working copy), then `adcs-case` (§8.3): the checker, the CSV reader and writer against `plan/case_inputs.toml`, import, export, the distribution report, the case store, and `adcs case template|check|import|export|report`. Add `Case.unstated`, `Case.assumed`, `Case.range` and `Case.level`, the `NotStated` and `NotFitted` answers, and `zero_when_absent` with gate check 7h (H7). Load cases at run time from the store (F16). Copy `_package/tools/check_case.py`, which CI keeps until `adcs case check` replaces it. The reference cases are imported after the seed forms (below), because the importer reads the reference values of `default` inputs (`orbit.ecc`, `surface.refl`, `surface.cd`) from their sheets.
- **Platform extensions.** Implement evidence rows and gate check 7g (§5.5); the `tier` field; the `[request]` table and `requests/` in a node folder (H7); the compiled bundle data in `adcs-core/build.rs` (§6); the KPI closures, written by the seeder from `plan/kpis.toml` (§5.5); and the UNCONFIRMED handling in the gap pass, `ready` and credibility (§5.8).
- **Intake.** Write `adcs-intake` and `cargo xtask intake check|write|verify|mark|reply|issue|selftest` (§5.11), porting the rules of `_package/tools/intake.py` one for one, with the same codes, D01–D07 and X01–X03 included (its docstring lists where the stand-in is narrower than the command); `intake write`'s version history (`versions.toml`, `versions/<n>/`) and its ledger writes (§5.13); `cargo xtask decision record` (§17.2); `cargo xtask manual`, replacing `_package/tools/manual_pages.py`; `intake write` over `form.rs`'s pure `set`, `normalise` and `value_allowed`. Write `cargo xtask form export` for node forms, the library and the case editor, filling `_package/forms/node_form.html`, `case_editor.html` and `library.html`, which are copied unchanged. Add the daemon's `GET /v1/form/…` routes and "Ask for a change" on the node page. Add Playwright and Chromium to the CI image. Copy `_package/tools/{forms,intake,form_browser_check,results,manual_pages,plan_model,make_examples}.py`, `_package/forms/` and `_package/results/`.
- **De-risking and explaining.** Write `adcs-derisk` and `cargo xtask derisk check|record|rollup|narrative|table|selftest` (§5.13), porting `_package/tools/derisk.py` rule for rule; `cargo xtask explain check`, replacing `_package/tools/explain_check.py`; the `risk::` physics functions (§6.2) and the `derisk` supplier in the supply map, which reads `derisk/rollup.toml`. The Risk management rows arrive as seed forms with the rest.
- **F7.** After a person settles D11, implement gate check 10b to that rule. Until then the phase report names F7 as open, and every HOLE the implementation agent writes is reviewed by eye against it.
- **The seed forms, through intake.** `tools/forms.py seeds` writes the 82 seed forms. Take each through the full pipeline of §5.11, in dependency order (a row after the rows it reads): `intake check --seed`, then the implementation agent on its brief (`intake write`, the HOLEs, any new physics function), then `intake verify` and the gate. One intake branch per seed form, reviewed like any other. Then import `plan/cases/*.csv` into `cases/`, and have every release install them into a new user's store, so the default case `ais_3u` is there on first run. Write `tools/atmos_table.py`, and the manifest skeletons for `igrf14` and `atmos-density`.
- **Green:** P0 green, plus `tools/validate_plan.py --selftest` and the full check, and `tools/build_tree.py --check`; the seeded ids equal `plan/expected_node_ids.json`; `intake selftest`; the seven test vectors pass; the `plan`, `intake`, `cases`, `forms`, `manual`, `derisk`, `explain`, `twin`, `matlab-pack` and `matlab-twin` jobs (§18), with `twin check --repo . --phase P1` and `pack_matlab --phase P1` at 0 refusals.
- **Acceptance:**
  - `adcs run sys_fluid_momentum_rings_ring_spin_down_time --case ais_img_3u` (tree row `gf_7`) answers within the test vector's tolerance of IDMAS v2 §03B's printed 0.75 s. It prints its chain, and its credibility is governed by Mathematics, because nobody's name is on the relation yet.
  - Its node folder holds `requests/<request id>.request.html`, its sheet's `[request] last` names that id, and `confirmed_by` reads `UNCONFIRMED · via <request id> · awaiting a person`.
  - `adcs run sys_pointing_error_budget_ape_budget_total` (`gp_5`) prints "n ran, m blocked". It names `gp_0`, `gp_1`, `gp_2` and `gp_4`, which no form has specified yet, and `gp_3`, which is `NotStated` because the case leaves `pointing.et` blank.
  - Rows reading the field or density are blocked with `DataMissing` naming the bundle, until P1's stop point.
  - `xtask ready` holds every seeded-then-specified row, because none is attested.
  - `adcs case check plan/cases/*.csv` and `adcs case import plan/cases/ais_3u.csv plan/cases/ais_img_3u.csv` print the reports of §8.9. `ref_c3_12u` imports with every `stated` input unstated and its three `default` inputs assumed; a row that reads an unstated key answers `NotStated`, naming the CSV key, never the sheet's reference value.
  - **The team member's loop, end to end:** `cargo xtask form export sys_fluid_momentum_rings_ring_spin_down_time`; in a browser, one field changed, a name, a reason and a "Checked by" given, and the file saved; `intake check` passes it and writes a brief; the implementation agent implements it; `intake verify` and the gate pass; the branch holds exactly that change, and `confirmed_by` now carries the attesting person's name from the form. A second check of the same file reports F04 (the node has moved on). `intake reply --status released` writes a file whose status card says so.
  - That round trip's form carries a belief record: `intake write` appends version 2 to the node's `versions.toml`, keeps version 1's sheet in `versions/1/`, and writes `derisk/beliefs/<request id>.toml`; the node's exported form then shows both versions, and `derisk check` passes.
  - The zip the P1 push built, unzipped on a machine with base MATLAB only: `startup_asils`, then `asils.case.read('cases/ais_3u.csv')` reads the case with the platform's report, and `asils.physics.fmr.spin_down_time` answers `gf_7`'s test vector within its tolerance. `TWIN.md` lists every P1 element as in the zip.
  - `cargo xtask derisk rollup` gives the 18 Risk management rows their values, and `adcs run mgt_conclusion_highest_open_risk_level` answers the highest open level in the register; `cargo xtask derisk narrative --quarter <current>` writes the page, the `.xlsx` and the `.csv`, and the page says no prose is written yet.
- **Stop points:**
  - D4 (redistribution terms) before any bundle is published, and D9 (the bundle licence-expiry policy).
  - D11 (the rule of check 10b), so that F7 can be built.
  - D25 (the implementation agent's model, who may start it, and what it may reach), before the first seed form is implemented.
  - D5 (the MATLAB release and licences), for the `matlab-twin` runner. Until it is settled, the twin's tests run on a developer's licensed machine before merge, and the phase report says so.
  - D26 (the risk scale and every starting level) and D27 (who writes the quarterly prose), before the first release's rollup; until then every level says "proposed".
  - A person downloads `igrf14coeffs.txt` from NOAA NCEI.
  - A person reviews and publishes the two bundles (two reviewers).
  - Engineers start confirming the seeded rows, each with a node form of type "confirm". Reaching the next phase does not wait for all of them.

### P2 — Catalogue and screening

- The catalogue was copied in P1. The engine runs on the working copy, and every manifest says `catalogue = "unpublished"` (§7.2). Publishing the bundle waits for D7 and is not a prerequisite for P3.
- Write `adcs-config` (§8.4) with `supply_map.rs`, candidates over products and over part combinations, the zero count for every empty slot, `adcs configure`, `adcs product list`, and `POST /v1/configure`. Write its twin `asils.product.load` in the same change.
- **Green:** P1 green, plus `twin check --repo . --phase P2` and `pack_matlab --phase P2` at 0 refusals, plus `adcs-config`'s own tests. They prove:
  - determinism: the same screening twice gives the same bytes;
  - the classification rules;
  - that a combination's counts are its filled slots' and zero elsewhere;
  - that an `offered` product holding a synthetic part is refused.
- **Acceptance:** §8.9's screening bullets, recorded as observed.
- **Stop points:**
  - D1 (restricted rows) before any result leaves engineering;
  - D7 (bundle boundaries), before the catalogue bundle is published, which is needed by P8, not by P3;
  - D13 (currency);
  - D18 (which bought parts replace the synthetic ones);
  - D21 (the satellite classes' ranges and standard cases);
  - a person confirms or replaces the two default 3U cases' plan-reference requirements (§8.9), and the class standards that copy them until D21.

  Until D18, no product can be offered, so client mode answers "no catalogue product" by design.

### P3 — The loop engine and SILS

- Write `adcs-sim-core`, `adcs-sim` and `adcs-fsw-abi` (§9).
- Copy `_package/scenarios/`, `_package/campaigns/`, `_package/devices/`, `_package/rig/` and `_package/fsw/include/`. Write the remaining `devices/SYN-*.toml` protocols in full, and the reference flight software in `fsw/` (§9.10), starting with B-dot.
- Write each element's twin in the same change as the element (§10.8.7): `asils.rng`, `asils.env`, `asils.plant`, `asils.devices.<kind>`, `asils.fsw.<id>` for every algorithm, `asils.metrics.<kind>`, `asils.scenario`, `asils.rec`, `asils.run` and `asils.result` (save, open, import, index) on the shipped result template and the store layout (§13.5.4). Add `cargo xtask rng-reference`, which writes the platform's 1,000 reference draws for the pack.
- If the company's existing detumble C code is handed over, wrap it behind `adcs_fsw.h` first and run it before the reference. It is the pilot the facility software was always meant to start from.
- **Green:** P2 green, plus `sim-determinism`, `sim-smoke` and `fsw` (§18); `twin check --repo . --phase P3` and `pack_matlab --phase P3` at 0 refusals; `matlab-twin` with the 1,000 reference draws reproduced exactly.
- **Acceptance:**
  - `adcs sim run nadir_hold_3u` completes on `SYN-P-3U-MTQ` against `ais_3u` and records APE and AKE over the last orbit. Whether coils alone meet 10° is recorded, not expected.
  - `adcs sim run detumble_3u` completes on `SYN-P-3U-MTQ` and records `detumble_time`, reading its orbit, epoch, tumble rate and mass properties from `ais_3u` through `case:` keys. If the plant reads a key the case leaves blank, the run is refused naming it, the refusal is recorded, and the case's owner states it before Green.
  - `adcs sim run detumble_3u --case ref_c3_12u` is refused, naming every unstated key it reads: the orbit, the epoch, the tumble rate, the mass properties, the surfaces and the residual dipole.
  - `adcs sim run inertial_hold_3u` completes on the synthetic twins against `ais_img_3u`, and produces every bound metric. Whether it meets 0.01° is recorded, not expected. This is the machinery pilot.
  - `adcs sim run inertial_hold_3u_idmas` is refused, naming the IDMAS parts' `nan` fields.
  - `adcs sim run slew_150kg_v3` is refused, naming the `nan` fields, the missing mounts, and the case keys `ref_c2_150kg` leaves unstated.
  - `adcs sim run detumble_3u --save` writes a result document into the local store, beside `ais_3u`'s case (§13.5.4). It opens offline in a browser, shows the run's channels and the requirement verdict, and `adcs result list` finds it. `adcs result import` of that file into an empty store files it beside its case again, and runs nothing.
  - Measure and record the wall-time ratio and runs per hour (§9.9).
  - The zip the P3 push built runs `asils.run('detumble_3u', asils.case.read('cases/ais_3u.csv'))` in MATLAB and saves its result into the zip's `store/`, beside `ais_3u`'s case. A blank `stated` input in a copied case CSV refuses the twin's run with the same message the platform prints.
- **Stop points:**
  - D5, if still open (the MATLAB release and licences, now for the parity work);
  - the company decides whether its own flight build or the reference is the default build for client SILS.

### P3M — The MATLAB SILS twin

- Every SILS element already has its twin, written with it from P1 to P4 (§10.8.7). This phase writes only what the twin alone has, and the comparison:
  1. `asils.viz` (run, campaign, compare, case) and `asils.app`, the map's one twin-only element;
  2. the six examples;
  3. the CI job `matlab-parity` (§10.8.6).
- **Green:** P4 green, plus:
  - `matlab-pack --phase P3M` (two builds, identical bytes), with `TWIN.md` listing every element with a MATLAB side as in the zip;
  - `matlab-twin` on a MATLAB runner;
  - `matlab-parity` runs and writes its ledger lines.
- **Acceptance:**
  - A person who has never seen the platform downloads the latest release's zip and unzips it on a machine with base MATLAB only. Following only `manual/`, they open the case editor, write a case, check it, run `startup_asils` and the six examples. Each example draws its figures and prints its metrics against the case's requirements, and each finished run appears in `store/` beside its case. The builder records a screen capture.
  - `asils.run('detumble_3u', asils.case.read('cases/ais_3u.csv'))` and the platform's run of the same scenario appear overlaid in `asils.viz.compare`. Their parity lines are in the ledger with the cause left for a person.
  - `asils.result.save` writes a result document for `detumble_3u`. Opened in a browser, it compares against the platform's result for the same case with no MATLAB running; `asils.result.import` of the platform's result files it beside the same case folder.
- **Stop points:**
  - D5 (MATLAB licences, now including the CI runner and students' use);
  - D23 (which twin–platform differences fail CI);
  - D24 (who may receive the zip outside the company, and on what terms);
  - a person writes the first causes on the twin's parity lines (H15).

### P4 — Campaigns, evidence, tuning and the solver

- Write the campaign runner: all six types, the process pool, manifests and resumption. Case ranges become dispersions, and a sweep may vary a case input. Wire `EvidenceSupply` into `Case.evidence` and the evidence closures. Score Uncertainty for evidence rows (F12). Write `tools/sil_parity.py` and the ledger.
- Write `asils.campaign.run` with the campaign runner and `asils.tune` with `adcs-tune`, each in the same change (§10.8.7).
- Write `adcs-tune` (§8.5) and `adcs-solve` (§8.6): `adcs solve`, `adcs design`, `POST /v1/solve` and `POST /v1/design`. The designer saves candidates to the store, never to `catalogue/`.
- **Green:** P3 green, plus:
  - a 20-run Monte Carlo that reproduces bit for bit from its manifest;
  - `twin check --repo . --phase P4` and `pack_matlab --phase P4` at 0 refusals;
  - the same tuning job twice giving the same tuned-set hash;
  - `solve-smoke` (§18).
- **Acceptance:**
  - `adcs sim campaign inertial_hold_mc500` runs 500 runs and supplies `p1a_0` with rung SILS.
  - Its credibility shows InputPedigree 0, because synthetic parts flew. That is the honest result, and it is the acceptance.
  - The edge, sweep and fault campaigns run.
  - Every campaign run with `--save` writes one campaign result document (§13.5.2): every run's metrics and dispersions, full channels for the nominal, best, worst and failing runs, and `adcs sim rerun` re-creates any other run bit for bit.
  - `inertial_hold_3u`'s parity line against IDMAS v2 §13 is written, with the cause left empty for a person.
  - `inertial_hold_labtwin_syn` runs on the synthetic lab; `inertial_hold_labtwin` is refused, naming `hils_bay1`'s `nan` fields.
  - `adcs solve ais_3u` and `adcs solve ais_img_3u` each answer "no catalogue product meets this case". Its client view says no product is offered for the class. Its internal view names every candidate tried and why none can be offered.
  - `adcs design designs/cubesat_3u_ais.toml` and `designs/cubesat_3u_img.toml` run, tuning each surviving combination. They save every passing one to the store as a `D-cubesat_3u-*` candidate marked synthetic, with a result document per job, or report that none passed and which requirement stopped each. Either outcome is recorded as observed, never asserted. `git status` is clean afterwards.
- **Stop points:**
  - a person writes the first parity causes (H15);
  - a person confirms or changes each algorithm's bounds (H14);
  - D22 (what evidence a candidate needs before it may be offered, and any weights in the tuning objective).

### P5 — Visualisation

- Write the web modules, SSE streaming and replay (§13), and a panel spec for each new view, the Risk management view included. Every view carries the explanation kit (`tools/explain_kit.py`) and its marks (§5.12).
- **Green:** P4 green, plus `panels`: render, move, read and match on synthetic runs, light and dark; the `explain` job over every view.
- **Acceptance:** a person can open a finished Monte Carlo campaign, find its worst run on the scatter, and scrub that run's 3D view against its plots. The builder records a screen capture of doing exactly that. "Open a result" files an uploaded result beside its case and shows it in the workbench with no engine call (§13.5.4). No view offers to edit anything but a case.
- **Stop points:** a person looks at each new panel's reference and records their sign-off in the panel's `confirmed_by`, or asks for changes. D2 (the demonstration subset the browser carries).

### P6 — The loop contract and the rig, on mocks

- Write `adcs-bus::loop` (§11) and `adcs-rig`: the real-time loop, transports and the emulators reused from `adcs-sim`, plus a mock IEU and a latency-measurement mode.
- Write `adcs rig fit` for the OILS lines (§12.10), reading the case's `v3` needs and the lab file, and make `adcs rig arm` refuse an OILS campaign whose fit is short, NotMeasured or NotStated.
- Run PIL with the loop contract over UDP to a process standing in for the target, then on a Raspberry Pi if a person provides one.
- **Green:** P4 green, plus `rig-mock`.
- **Acceptance:** the same campaign gives SILS and mock-OILS ledger lines, and the difference has a cause the builder can state: the mock IEU's latency. `adcs rig fit ais_3u --lab rig/labs/hils_bay1.toml --rung oils` answers NotMeasured on every measured line, naming each field, and `adcs rig arm` refuses; against `syn_lab.toml` it prints a verdict on every line, recorded as observed.
- **Stop points:** D14 (the IEU hardware).

### P7 — HILS, the lab twin and safety

- Write the environment-simulator drivers against their interface documents with mock backends, the lab twin's use of each measured lab value as bring-up records it (the campaign type exists from P4), `safety/`, `docs/RIG_BRINGUP.md` with one line per lab-file field, and the H-rig checklist.
- Extend `adcs rig fit` to the HILS lines (§12.10).
- **Green:** P6 green, plus a HILS campaign against the mock environment simulator.
- **Acceptance:** for `inertial_hold_labtwin_syn`, SILS, lab-twin and mock-HILS lines appear side by side in the parity ledger (and in the parity view once P5 is done). `adcs rig fit ais_img_3u --lab rig/labs/syn_lab.toml --rung hils` prints every line with its verdict, and a HILS campaign on `hils_bay1` is refused by `adcs rig arm`, naming each unmeasured field.
- **Stop points:** a person runs bring-up device by device, and signs H-rig per configuration.

### P8 — Evidence and certification

- Write `adcs-evidence` per §14: requirement package, verification matrix, ledger views, reports, as-built twin, certificate generator (unsigned) and package export with the exclusion test.
- **Green:** P7 green, plus `evidence`.
- **Acceptance:** for a mock order on `ais_img_3u`, the package builds and the certificate **refuses to issue**. It lists every reason: synthetic parts flew, parity causes are open, the requirements are not all written, there is no signatory. That refusal, complete and by name, is the acceptance.
- **Stop points:** D15 (software criticality category), and the ECSS tailoring for the first order.

### P9 — The portal

- Write `adcs-portal`, `adcs-worker`, migrations, roles, routes, the order state machine, case upload and the case editor on screen, result upload and listing, the requests inbox (§5.10.6) with replies, the solver and design-request paths, the internal catalogue view with recorded H14 decisions, unit descriptors at calibration, quotas, quote hashing, PO upload, witness streaming from the mock rig, the FMU sandbox (after D16), and `deploy/compose.yaml`.
- **Green:** P5 and P8 green, plus `portal`.
- **Acceptance:** an end-to-end test on the compose stack runs the whole path:
  1. a client user downloads the case template, uploads `ais_img_3u.csv` into a project, and sees the import report;
  2. runs the solver on the production catalogue, gets "no catalogue product meets this case" with its gap, and requests a design;
  3. a designer user sees the request, runs the design job, and the candidate appears in the internal catalogue view and never in the client's;
  4. on the **test catalogue**, the client solves, runs a SILS nominal and a small Monte Carlo, and sees the dashboard. The test catalogue is `crates/adcs-portal/tests/fixtures/catalogue/`, loaded only by the `test-catalogue` feature. It holds one product, `TEST-P-1`, with `test_fixture = true`, `status = "offered"` and `[promotion] by = "TEST FIXTURE — not a person"`. The production build refuses any product with `test_fixture`, and `tools/validate_plan.py` never reads that directory;
  5. requests a quote; a sales user issues it through H-quote; the client accepts and uploads a PO;
  6. sales accepts the PO;
  7. a mock OILS campaign streams to the client's witness view;
  8. the evidence page shows the refusing certificate from P8;
  9. an internal `engineer` downloads a node form from the portal, sends it filled, sees it "received" in the inbox and its issue opened; a `developer` posts "check failed" with a finding, and the engineer sees it on the request.

  The tenant-isolation suite passes, and no route changes the software.
- **Stop points:** D6 (the running budget), D16, D17, D19, and an external security review before the first real client (D20).

### P10 — Hardening

- Finish `docs/manual.toml` for everything. Complete both manuals (§16.4) against the software as built, with real outputs pasted where a page shows one. Write `docs/USING_IT.md` as a walkthrough, as VLEO's is.
- Finish the devcontainer; do a release dry run through the `release` environment; review `docs/SPEC_DEVIATIONS.md` line by line with a person.
- **Green:** everything, in both profiles, plus a full nightly run.
- **Acceptance:**
  - A team member who has never used the software follows only `manual/user/` in the released web app: writes a case, checks it, runs it, reopens the saved result, and sends a node form for a change they want, with its belief record. Nobody helps. The builder records the transcript.
  - **The teach-back test of every template** (risk R-17): for the node form, the case editor, the library, a result and the narrative, one person who has not seen it fills it, or reads it and explains it back; each miss is fixed, and the test is recorded as a belief record that lowers R-17 only if it held.
  - A new developer follows only `manual/developer/` and `README.md` from a fresh clone, in a Codespace: takes that request through intake, has the agent implement it, verifies, and replies. The builder records the transcript.
- **Stop points:** D3 (code signing) and D8 (signing-key custody) before the first release.

---

## 20. Decisions the builder must not take

The builder prepares each one: options, costs, a recommendation if asked. It then stops. D1–D9 are VLEO's DELIVERY_PLAN list, carried over because each still applies.

| | Decision | Needed by |
|---|---|---|
| D1 | which ADCS rows are restricted and never ship (candidates: allocation weights, controller gains, pump efficiency maps, friction curves) | P2 |
| D2 | the demonstration subset the browser carries | P5 |
| D3 | code-signing certificates and notarisation | P10 |
| D4 | redistribution terms for IGRF-14, the NRLMSIS 2.0 table and the star catalogue | P1 |
| D5 | the MATLAB release and licences: the SIL #1 parity, code generation, the MATLAB twin's CI runner (from P1, since the twin is built in lockstep), and students' use of the twin | P1 |
| D6 | the monthly running budget for the portal and workers | P9 |
| D7 | bundle boundaries (one catalogue bundle, or one per family) | P2 |
| D8 | signing-key custody | P10 |
| D9 | the licence-expiry policy for bundles | P1 |
| D10 | extract a shared platform used by VLEO and ADCS, or keep two repositories in step by hand | P0 |
| D11 | the exact rule of gate check 10b (F7): what a hole body may contain | P1 |
| D12 | the GitHub team behind each owner | P0 |
| D13 | the currency and who owns the price table | P2 |
| D14 | the interface emulation unit's hardware, and the electrical level of each flight interface | P6 |
| D15 | the flight software's criticality category under ECSS-Q-ST-80C | P8 |
| D16 | the FMI host: adopt a Rust crate or bind the reference C headers; which FMI versions | P9 |
| D17 | where the portal is hosted, and the data residency clients need | P9 |
| D18 | which bought parts, from which suppliers, replace the synthetic ones | P2 |
| D19 | the client SILS quota and whether client runs are charged | P9 |
| D20 | the external security review before the first real client | P9 |
| D21 | the satellite classes: the small-satellite mass ranges, and each class's standard case with its `lo`/`hi` ranges and standard requirements | P2 |
| D22 | what a candidate product needs before it may be offered (Monte Carlo size, minimum worst margin, which scenarios), and whether the tuning objective carries weights | P4 |
| D23 | which differences between the MATLAB twin and the platform's SILS fail CI, per metric and per channel | P3M |
| D24 | who may receive the MATLAB twin outside the company (students, partner institutes), under what agreement, and whether the `trainee` role grants it | P3M |
| D25 | which model the implementation agent runs on, which accounts may start it, and what it may reach from the checkout (network, secrets) | P1 |
| D26 | the risk scale: how ECSS-M-ST-80C's severity and likelihood map to L1–L5, every starting level in `derisk/risks.toml`, and who may open, raise and lower a risk | P1 |
| D27 | the de-risking narrative: when a quarter closes, who writes its prose in `derisk/narratives/<Q>.md`, and who reads it before the next review | P1 |

---

## 21. Risks carried

**In one line:** the risks are kept in the risk register, `derisk/risks.toml`, not here; this table is generated from it (`python3 tools/derisk.py table`; in the repository the same table is `docs/RISKS.md`, and CI fails when it differs), and every level is a proposal until decision D26 confirms it.

A risk's level moves only through a belief record, and goes down only when a test is recorded (§5.13). The layer-1 Risk management rows count this register; the quarterly narrative explains it.

<!-- risks:begin (generated by tools/derisk.py table) -->
| Risk | Area | Level | Why it is real | What is done about it | The test that closes it |
|---|---|---|---|---|---|
| R-01 The tree is mostly unwritten | node | L4 | Most sheets need a person's form; the seed content covers the pilot thread and the risk branch only. | Fill by thread, not by subsystem. The node library marks what is written; xtask ready counts what remains. | xtask ready reports every node a release's product reads as specified, and a product built on them passes its campaign |
| R-02 Requests arrive faster than the developer team can take them | node | L3 | Every change goes through one team. | The checker does the first reading in seconds; the brief bounds implementation; the inbox shows the queue. | two consecutive quarters in which the median request goes from 'received' to 'released' within one release cycle |
| R-03 A request is complete but wrong | math | L4 | The checker proves consistency, not truth. | 'Checked by' makes the judgement a named person's; test vectors come from cited pages; an unconfirmed relation runs as UNCONFIRMED. | every node a released product reads is confirmed by a person and carries at least one test vector from a cited page |
| R-04 The IDMAS parts are mostly unmeasured | model | L4 | Pump efficiency, friction curves, mass and power are 'nan' in IDMAS v2 §16. | Every result that reads them carries low credibility until the bench measures them. | bench measurements of each IDMAS part's unmeasured fields, recorded in the part files with their reports |
| R-05 A PC is not a hard real-time target | model | L3 | 1 kHz over UDP on commodity hardware can jitter. | Measure at commissioning (SPEC.md §12.3); move the plant to the PolarFire SoC if it cannot hold. | the commissioning run records deadline misses per hour below the facility's allowance for the whole of a campaign |
| R-06 An air bearing is not orbit | model | L3 | Its residual torque exceeds a CubeSat's orbital disturbances. | HILS is compared with the lab twin, never with the orbit run (SPEC.md §12.6); adcs rig fit refuses a HILS campaign whose bearing, cage or stimulators fall short of what the case needs (SPEC.md §12.10). | a HILS campaign and its lab twin agree within D23's tolerances on every metric the certificate states |
| R-07 Customer flight code keeps hidden state | algorithm | L3 | C statics break repeatability across runs. | The init-twice test (SPEC.md §9.6) refuses such a build by name. | the init-twice test passes on every customer build a campaign runs |
| R-08 Intellectual property leaves through the browser or an FMU | output | L4 | VLEO's WASM ships the whole kernel (F3); an FMU is native code. | F3 is fixed; FMUs run only in a sandbox (SPEC.md §15.6); restricted content is excluded by test (SPEC.md §14.6). | the restricted-content test passes on every artefact a release publishes, and D20's external review finds no route out |
| R-09 Synthetic or demonstration numbers reach a client as evidence | output | L5 | Round test values look plausible. | SYN-* naming, the status rule, InputPedigree 0, a CI refusal (SPEC.md §7.3); demo results are marked at the top and never evidence. | the CI refusal and the result viewer's demo banner are shown, by test, to stop every synthetic path the release has |
| R-10 The MATLAB twin drifts from the platform | model | L3 | Two implementations of one engine diverge unless something checks them. | One set of definitions, exported; every SILS element written in both engines in the same change from P1, as plan/twin_map.toml lists them, and the twin job refusing a one-sided change (SPEC.md §10.8.7); the same fixtures and draws; the matlab-parity job on every push. | matlab-parity green on every push for a whole quarter, with every ledger line explained by a person |
| R-11 Tuning fits the case, not the physics | algorithm | L4 | A search over gains finds whatever the plant model rewards. | Tune on nominal plus edge corners; confirm with a Monte Carlo never tuned on (SPEC.md §8.5). | a tuned product's HILS result agrees with its confirming Monte Carlo within D23's tolerances |
| R-12 The design sweep explodes | algorithm | L2 | Parts x counts x families grows as a product. | Screening prunes before SILS; keep caps what is saved; the sweep's size is printed first. | the largest design job of a quarter finishes inside its stated budget |
| R-13 A case CSV is read two ways | input | L3 | A portal form and a file upload drift apart. | One importer, adcs-case; the case editor writes a CSV; the format is generated from the tree and checked in CI. | every face (web, workbench, MATLAB, portal) imports the reference cases to the same case hash |
| R-14 Tenant data leaks from the multi-tenant portal | output | L5 | A tenant data leak is the costliest defect a portal can have. | The isolation suite on every route and role; D20 before the first client. | D20's external review and the isolation suite both pass before the first client's project is opened |
| R-15 Pointing-budget contributors are not all independent and random | math | L3 | The APE row adds its contributors in quadrature; a bias among them would add linearly and the budget would be optimistic. | The assumption is written on the row (gp_5) and every result that reads it says so. | each contributor classified as random or bias by a person, with the classification's source, and the budget rule matched to it |
| R-16 A reader takes a demonstration or unconfirmed result as evidence | visualisation | L4 | A clean chart looks authoritative whatever produced it. | The result viewer states the engine and credibility in its first sentence; demo results carry a banner; UNCONFIRMED rows are named. | a reader test: people who have not seen the viewer are shown a demo and a real result and asked which is evidence; every one answers right |
| R-17 A template explains itself only to the person who wrote it | visualisation | L3 | Every template in this package was written by its authors, who cannot see what a new reader misses. | Every template follows the explanation standard adcs-explain/1 and tools/explain_check.py checks the parts a program can see. | a teach-back test per template: a new team member fills it, or reads it and explains it back, without help, and the misses are fixed |
| R-18 The lab cannot meet a case's OILS or HILS needs | model | L3 | A case may need a field range, a bearing torque, a truth accuracy or a stimulator the bay does not have, and a campaign booked on it would measure the lab, not the unit. | The tree computes each case's rig needs; the lab file holds the measured capabilities; adcs rig fit compares them and adcs rig arm refuses a short fit (SPEC.md §12.10). | adcs rig fit passes for the reference cases on hils_bay1 with every line measured |
<!-- risks:end -->

---

## 22. Sources

- VLEO_SIMULATOR at `abf79ee`: [README](https://github.com/AmanRai0603/VLEO_SIMULATOR/blob/main/README.md), [AGENTS.md](https://github.com/AmanRai0603/VLEO_SIMULATOR/blob/main/AGENTS.md), [docs/ARCHITECTURE.md](https://github.com/AmanRai0603/VLEO_SIMULATOR/blob/main/docs/ARCHITECTURE.md), [docs/DELIVERY_PLAN.md](https://github.com/AmanRai0603/VLEO_SIMULATOR/blob/main/docs/DELIVERY_PLAN.md), [docs/NODE_AUTHORING.md](https://github.com/AmanRai0603/VLEO_SIMULATOR/blob/main/docs/NODE_AUTHORING.md), [crates/vleo-bus/src/lib.rs](https://github.com/AmanRai0603/VLEO_SIMULATOR/blob/main/crates/vleo-bus/src/lib.rs), and the files named in §3; read in full for this document.
- CD-06 · VLEO Integrated Design Tool: plan and working model (the source document of VLEO's tree).
- IDMAS v2 Exploration: from magnetorquer pointing to an actuator system for any satellite (Orbitt Space, 2026).
- *Eight Loops, One Beam* (v2), an RF ion thruster taught with the learner's loop and thirteen explainer's techniques, supplied with this plan. Used for: the explanation standard `adcs-explain/1` (§5.12).
- ECSS-M-ST-80C, Space project management — Risk management (ECSS, 31 July 2008). Used for: scoring a risk by severity and likelihood, which the five risk levels map onto by decision D26 (§5.13).
- The company's Quarterly De-risking Narrative template (`derisk/narrative_template.xlsx` is regenerated from its title and columns). Used for: the belief record's fields and the narrative's columns (§5.13).
- [ECSS-E-ST-60-30C, Satellite AOCS requirements (30 August 2013)](https://ecss.nl/standard/ecss-e-st-60-30c-satellite-attitude-and-orbit-control-system-aocs-requirements/)
- [ECSS-E-ST-60-10C, Control performance (15 November 2008)](https://ecss.nl/standard/ecss-e-st-60-10c-control-performance/)
- [ECSS-E-ST-10-02C, Verification](https://ecss.nl/standard/ecss-e-st-10-02c-verification/) and [ECSS-E-ST-10-03C Rev.1, Testing (31 May 2022)](https://ecss.nl/standard/ecss-e-st-10-03c-rev-1-testing-31-may-2022/)
- [NOAA NCEI, International Geomagnetic Reference Field](https://www.ncei.noaa.gov/products/international-geomagnetic-reference-field): IGRF-14, finalized November 2024
- [MathWorks, Export Simulink models to functional mock-up units](https://www.mathworks.com/help/fmuexport/ug/export-simulink-models-to-functional-mock-up-units.html): FMI 2.0 and 3.0 via Simulink FMU Builder
- [Phoronix, Real-time PREEMPT_RT support merged for Linux 6.12](https://www.phoronix.com/news/Linux-6.12-Does-Real-Time)
- [FMI 3.0 specification](https://fmi-standard.org/docs/3.0/)

The OILS and HILS facilities, and the twin's lockstep (§10.8.7, §12.2, §12.10; each is also a `[[source]]` in `plan/seed_content.toml`, so a node form can cite it):

- [Schwartz, Peck and Hall, Historical review of air-bearing spacecraft simulators, *JGCD* 26(4), 2003](https://doi.org/10.2514/2.5085). Used for: a bearing's residual torque is the weight times the centre-of-mass offset; tilt limits by bearing type.
- Ibrahim et al., Air-bearing testbeds for CubeSat and nanosatellite (review), Research Square preprint rs-10893767, 2026, not yet peer reviewed. Used for: typical CubeSat bearing figures (balancing, tilt of about ±45° to ±50°), cited as a preprint.
- da Silva et al., Helmholtz cage design and validation for nanosatellites HWIL testing, *IEEE Transactions on Aerospace and Electronic Systems*, 2019. Used for: what a cage is specified by (field per axis, magnitude and direction error, homogeneous volume) and how an orbit's field is commanded and checked.
- ASTM E927-19, Standard classification for solar simulators (ASTM International, 2019). Used for: stating a Sun simulator's non-uniformity and temporal instability by class.
- Zhao, Zhuang and Lembeck, A low-cost hardware-in-the-loop simulator facilitating CubeSat star tracker development, 38th Small Satellite Conference, 2024. Used for: a star stimulator's angular step, faintest magnitude and frame rate.
- Bremer et al., The Euclid AOCS simulation facilities, ESA GNC 2017. Used for: one model and one parameter database carried from the MATLAB engineering simulator into the real-time simulator and every later facility; the pattern of the twin's lockstep.
- Colagrossi, Silvestrini and Lavagna, Flat-sat facility for processor-in-the-loop verification and testing of nanosatellite ADCS, ESA GNC-ICATT 2023. Used for: the OILS rig's emulated interfaces, missed-tick count and isolation; one functional simulator from model-in-the-loop to PIL.
- Nadeem, Empirical methods for reaction wheel micro-vibration verification in a production environment, 36th Small Satellite Conference, 2022. Used for: wheel imbalance and disturbance measured on a force dynamometer.
- Springmann and Cutler, Magnetic sensor calibration and residual dipole characterization for application to nanosatellites, AIAA 2010-7518. Used for: measuring a coil's dipole and a satellite's residual dipole.
- ECSS-E-TM-10-21A, System modelling and simulation (ECSS, 16 April 2010). Used for: the names and roles of simulation facilities, and model reuse across them.

---

## Appendix A — Glossary

| Term | Meaning here |
|---|---|
| SILS | software-in-the-loop simulation: flight code linked into the loop engine, faster than real time |
| PIL | processor-in-the-loop: flight code on the target processor, plant on a PC |
| OILS | OBC-in-the-loop simulation: the flight OBC against the real-time plant, every port answered by the IEU |
| HILS | hardware-in-the-loop simulation: the real ADCS unit in the cage on the air bearing, with stimulators |
| IEU | interface emulation unit: the board that answers the OBC's flight connectors with bytes from the plant |
| lab twin | the scenario run with the plant replaced by a model of the lab; what HILS is compared with |
| twin map, lockstep | `plan/twin_map.toml`: every SILS element's platform side and MATLAB side, the phase that writes both and the test both pass; lockstep is the rule that an element changes in both engines in the same pull request, or carries `twin:none` with a reason (§10.8.7) |
| rig needs | layer 2's "OILS rig needs" and "HILS rig needs" rows: what a rig must do for this case (field range, bearing torque allowed, truth accuracy, ...), computed from the case (§5.3) |
| `lab` supplier, NotMeasured | the facility rows of layer 1 read their values from the lab file (`rig/labs/<lab>.toml`); a `nan` there answers `NotMeasured`, naming the field (§8.2) |
| rig fit | `adcs rig fit`: the case's rig needs against the lab's measured capabilities, one verdict per line; a campaign whose fit is short is not armed (§12.10) |
| evidence row | an achieved KPI row only a campaign can supply (§5.5) |
| parity ledger | per scenario, metric and rung: the value, its differences, and a person's cause (§14.2) |
| case | one customer's statement: one CSV in the fixed format `adcs-case/1`, imported and hashed (§8.3) |
| unstated | a case input left blank under the `stated` policy: every row that reads it is blocked, naming the key; nothing is guessed |
| family | a catalogue configuration: `mtq`, `mtq_rw`, `mtq_fmr`, `mtq_fmr_rcs` |
| product | a configuration of catalogue parts (slots, counts, mounts) plus the flight algorithms it carries (§7.7); `candidate`, `offered` or `retired` |
| algorithm | a flight algorithm with its tunable parameters and their bounds (§7.8) |
| tuned set | the parameter values the tuner found for one case and one product; restricted, recorded by hash |
| satellite class | what the design team designs for; its standard is a case CSV (§7.9) |
| result document | one self-contained HTML file holding a finished run or campaign: the case CSV as run, metrics, verdicts and the kept channels; opened offline, compared, never read back as evidence (§13.5) |
| node form | one self-contained HTML file that is a node's document, the request form and, once saved, the request; the one way anyone asks for a change (§5.10) |
| request | a filled node form: change, new, confirm, feedback or something else; named `REQ-<date>-<node>-<hash>` once checked |
| intake | the developer team's pipeline for a request: check, implement, verify, review, release, reply (§5.11) |
| seed form | a node form carrying the package's seed content for one of the 82 seed rows (the pilot thread and the Risk management branch), taken through intake at the first build (§5.8) |
| belief | a bet a node, an input, an output, a model, a relation, an algorithm or a page rests on, recorded with the test that would settle it; broke, held or untested (§5.13) |
| risk, R-nn, L0–L5 | something that could go wrong because a belief may be wrong, in `derisk/risks.toml`, with a level from 0 (closed) to 5 (critical) and its closing test |
| version (of a node) | one state of a node, with the request and belief that made it, what was wrong before and what it gives; `versions.toml` (§5.13) |
| Risk management | the layer-1 branch that counts the ledger at every release and concludes (§5.2, §5.13) |
| de-risking narrative | the quarterly page of the ledger, in the company template's columns, with a person's prose (§5.13) |
| `adcs-explain/1` | the explanation standard every page a person reads follows (§5.12) |
| station | a concept told in four steps: say it simply, now the real thing, where the simple version breaks, try it (§5.12) |
| Learn · Read · Expert | the three depths of every page (§5.12) |
| implementation agent | Claude Code, started by a developer on a checked request's brief; writes code, never content (§17.4) |
| Checked by | the node form's field for the engineer who stands behind a relation or value; the only source of a name in `confirmed_by` (§5.8) |
| case editor | the offline HTML page that writes a case CSV and says what it can run (§8.3.3) |
| store | a folder of cases and results, each result beside the case it ran, with an index rebuilt from the files (§13.5.4) |
| MATLAB SILS twin | the SILS engine again in plain MATLAB, released as one zip with the case editor, the node library, the result viewer and the user manual; for team members and developers, never clients (§10.8) |
| client mode, designer mode | `adcs solve` over offered products for a case; `adcs design` over part combinations for a class, saving the ones that pass (§8.6) |
| descriptor | one part's record, read by the quote, the tree, the plant, the flight config and the EEPROM (§7) |
| APE, RPE, PDE, AKE, RKE | absolute pointing, relative pointing, pointing drift, absolute knowledge and relative knowledge errors (ECSS-E-ST-60-10C) |
| UNCONFIRMED | what `confirmed_by` says until an engineer attests the node in a form; honest, never a failure (§5.8) |
