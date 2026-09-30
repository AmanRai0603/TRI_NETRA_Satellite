
---

## 16. Faces and manuals

### 16.1 The faces read and run

The ported faces keep every read and run command and route they have, renamed (§3.3). Every command and route that wrote a sheet is removed (§3.4, F17). What a face may put into the software is a case CSV, a result document to view and, in the portal, a client's FMU. The software's content changes only through intake, in a developer's checkout.

**CLI (`adcs`).** The `campaign` subcommand means "every stored case against one row", so the simulator's commands live under `sim` and never clobber it.

```
adcs case template                          # the blank template CSV, every key explained in its note
adcs case check <file.csv>...               # the format, what the case says, what it can run (§8.3.4)
adcs case import <file.csv>... [--store <dir>]  # check, place every value, print the distribution report
adcs case export <case> [--out <file.csv>] # the canonical CSV back
adcs case report <case>                    # the distribution report again
adcs configure --case <c> [--families f;g] [--json]  # screening only (§8.4)
adcs solve <case> [--families f;g] [--evaluations N] [--json]   # client mode (§8.6)
adcs design <job id | file.toml> | --case <c> [--families f;g]   # designer mode; the release's jobs by id; saves candidates to the store
adcs product list [--status s] [--class c]
adcs sim run <scenario> [--case <c>] [--product <p>] [--rung sils] [--save]
adcs sim campaign <campaign> [--case <c>] [--jobs N] [--save]
adcs sim rerun <result.html> <k>              # re-create a campaign run whose channels were not kept
adcs sim ledger <campaign>                 # the parity-ledger lines for a campaign
adcs result list [--case c] [--verdict fail]   # the local store (§13.5.4)
adcs result import <file.result.html>...       # file results made elsewhere beside their cases; runs nothing
adcs result open <file> | index [<store>] | channels <file> <run> <out.csv>
adcs fswcfg --case <c> --product <p> --tuned <hash> [--serials <dir>] --out <file>
adcs eeprom --serial <file> --out <file>
adcs evidence package <order-dir> --out <zip>
adcs rig fit <case> --lab <file> [--product <p>] [--rung oils|hils]   # needs against the lab's capabilities (§12.10); reads only
adcs rig arm|run|abort <campaign> --devices <map>    # rig host only; refuses elsewhere, and refuses a campaign whose fit is short
```

**Daemon routes (workbench).** `/v1/parity/<…>` serves parity files, so the ledger gets its own route. There is no write route and no `ADCS_ALLOW_WRITE`.

```
POST /v1/case/check         POST /v1/case/import        GET  /v1/case/<id>/report
GET  /v1/form/<node>        GET  /v1/form/case[/<id>]   GET  /v1/form/library      (§5.10, §8.3.3)
GET  /v1/results            POST /v1/result/import      GET  /v1/result/<id>        (§13.5)
POST /v1/configure          POST /v1/solve              POST /v1/design
POST /v1/sim/run            POST /v1/sim/campaign
GET  /v1/runs/<id>          GET  /v1/stream/<run>     (SSE)
GET  /v1/ledger/<campaign>  GET  /v1/manual/<page>
```

`POST /v1/case/…` and `POST /v1/result/import` write into the user's store (§13.5.4), never into the repository. `POST /v1/design` saves candidates to the store.

**Python (`adcs`).** Adds `case_check(path)`, `case_import(path)`, `case_export(case)`, `configure(case, family=None)`, `solve(case)`, `design(job)`, `sim_run(scenario, case=None, product=None, save=True)`, `campaign(campaign, jobs=1, save=True)`, `result_import(path)`, `ledger(campaign)`.

**MATLAB (`+adcs`).** Adds `adcs.case_import`, `adcs.solve`, `adcs.sim`, `adcs.campaign`, and `adcs.export_parity(simOut, scenarioId)`. The last writes a MATLAB SIL result in the parity-reference format, so the company's MATLAB SIL becomes a ledger line without anyone retyping a number.

### 16.2 The developer's commands

`xtask` is the developer team's, in a checkout. It has the reading and building commands (§3.4) and:

```
cargo xtask intake check <file> [--out intake/requests] [--seed]     # §5.11.2
cargo xtask intake write <request.json>                              # §5.11.4: the only writer of node content
cargo xtask intake verify <request.json>                             # §5.11.4
cargo xtask intake mark verified <request> | published --release <v> | verified --closures
cargo xtask intake reply <file> --status <s> [--note] [--release] [--report] --out <file>
cargo xtask intake issue <file>                                      # feedback and "something else" to issues
cargo xtask intake selftest
cargo xtask form export <node>... [--layer <sid> | --group <id>] [--out <dir>]
cargo xtask form export --library <dir> | --case [<case.csv>]
cargo xtask manual [--check]                                          # the manuals' generated pages (§16.4)
cargo xtask derisk check | selftest                                   # the ledger's rules L01-L09 (§5.13)
cargo xtask derisk record --area <a> --about <x> --believed … --status … [--move R-nn:from:to] [--risk R-nn] [--open …]
cargo xtask derisk rollup [--quarter Qn-YY]                           # the 18 Risk management rows, into derisk/rollup.toml
cargo xtask derisk narrative --quarter Qn-YY --out <dir>              # the quarterly narrative: .html, .xlsx, .csv
cargo xtask derisk table [--check] [--file docs/RISKS.md]             # the register as a table (§21)
cargo xtask explain check [--selftest]                                # the explanation standard over templates and manuals (§5.12)
cargo xtask decision record <record>                                  # copies a person's recorded decision into its file (§17.2)
cargo xtask rng-reference                                             # the platform's reference draws for the MATLAB pack
cargo xtask twin check [--repo . --phase Pn[,Pm]] [--changed <file of paths> [--label twin:none --reason <text>]]   # the lockstep (§10.8.7), TW01-TW06
cargo xtask twin list                                                 # every SILS element, both sides, phase, rungs, test
```

The tree's shape changes with `tools/seed_tree.py --add-group <id> --under <parent>` (§5.7), the team's own work.

`cargo xtask new <id> --like <sibling>` stays, and is run only as the first step of a new-node request's brief.

### 16.3 `docs/manual.toml`

It documents every command, route and setting the software has, for users and developers, and says which is which. The `the_manual_is_true` test enforces it in both directions, and its parsers are extended to the `sim`, `result`, `case`, `rig` and `intake` dispatch. It is the source of the user manual's command reference (§16.4), so the manual can never name a command the software does not have.

### 16.4 The two manuals

The package carries both manuals in full, written for the software as this document specifies it and to the explanation standard (§5.12): every page says its kind (tutorial, how-to, reference, explanation) and depth under its title, opens with **In one line:**, and has the sections its kind needs. The builder copies them in P0, keeps them true in every phase, and completes them in P10. They are checked like code.

**The user manual**, `manual/user/`, is for team members. It is shipped in every release: in the MATLAB zip (`manual/`), on the web face (`GET /v1/manual/<page>`, "Help"), and in the portal. It assumes nothing about the repository, git or code.

| Page | Kind | Says |
|---|---|---|
| `00_start_here.md` | tutorial | the first hour, by doing it: a case edited, a blank that blocks, a run, a result read, a node rebuilt; where to go next |
| `01_your_case.md` | how-to | writing a case in the editor or the template; checking it; changing it |
| `02_running.md` | how-to | running SILS in the web app, the workbench and the MATLAB tool: runs, campaigns, the solver, the designer |
| `03_results.md` | how-to | reading, keeping, comparing, sharing and re-running a result |
| `04_asking_for_changes.md` | how-to | the node form: each request type, each section including "Explain it" and "De-risking", with an assistant or by hand; what happens after sending |
| `05_case_keys.md` | reference | every key of the case format — generated from `plan/case_inputs.toml` |
| `06_reading_a_node.md` | explanation | what a node is, what its page says, kinds, states, credibility |
| `07_why_things_change.md` | explanation | beliefs, risks, versions, and the Risk management conclusion |
| `08_glossary.md` | reference | the words the software uses |

**The developer manual**, `manual/developer/`, is for the developer team.

| Page | Kind | Says |
|---|---|---|
| `00_the_job.md` | explanation | what the developer team owns, and the rules nothing may bend (§0.2, §3.2) |
| `01_intake.md` | how-to | receiving a request; the checker, its report and every code (generated); replying |
| `02_implementing.md` | how-to | running the implementation agent on a brief; intake write (sheet, version, belief, risk moves), HOLEs, new physics functions; verify |
| `03_review_and_release.md` | how-to | reviewing an intake branch; releasing, with the rollup; replying "released" |
| `04_own_changes.md` | how-to | the team's own work, each with its belief record; what needs two reviewers |
| `05_seed_intake.md` | how-to | the first build: taking the 82 seed forms through intake in P1 |
| `06_checks_and_ci.md` | reference | every check in the package and in CI, and what each proves |
| `07_explaining.md` | explanation | the explanation standard, walked through on its worked example, `gf_7` |
| `08_derisking.md` | how-to | keeping the ledger, the rollup at release, the quarterly narrative; the ledger's rules (generated) |
| `09_twin.md` | how-to | the MATLAB twin in lockstep: the twin map, changing an element on both sides, prototypes, `twin:none`; the rules (generated) |

**Checked.** `cargo xtask manual --check` (built in P1; `tools/manual_pages.py --check` is its stand-in in this package) regenerates `05_case_keys.md` from the case registry, the codes table in `developer/01_intake.md` from the checker and the rules table in `developer/08_derisking.md` from the ledger, the twin rules in `developer/09_twin.md` and §10.8.7 from the twin check, and fails on any difference; `cargo xtask explain check` checks every page's kind line, one-line answer and sections; it is green from P1. `tools/manual_check.py` runs every command a manual page shows and compares the output it claims. The manuals are written for the finished software, so a page may show a command a later phase builds: every command block is tagged with the phase that builds it (`<!-- since P3 -->`, as the package's pages already are), and `manual_check.py` runs a block only once its phase is green. The command reference itself is `docs/manual.toml`, which the web face shows under Help → Commands.

---

## 17. Governance

### 17.1 Instruction files

- `AGENTS.md`: the standing instructions of every agent working in the repository, which are now two: the builder during the phases, and the implementation agent after (§17.4). It holds §3.2's five rules, §0.2's rules, and four sections: evidence rows (§5.5), scenarios and campaigns (§10), the rig's safety rule (§12.8), and intake (§5.11): "a node's content comes only from a passing request, through `intake write`".
- `intake/AGENT.md`: the implementation agent's own page: how to read a brief, the scope hook, and the list of things it never does.
- `areas/`: one file per area of the engine, and six more:
  - `simulation.md` for `adcs-sim-core`, `adcs-sim`, `adcs-fsw-abi` and `matlab_sils/`: determinism (§9.7), "the plant never reads a clock", and "the twin follows the platform's equations; a difference is a ledger line, never a silent fix in one of them";
  - `rig.md` for `adcs-rig`, `rig/` and `devices/`: hardware interlocks, mocks and bring-up;
  - `portal.md` for `adcs-portal` and `adcs-worker`: tenant isolation, no engine in the portal, restricted content, no route that changes the software;
  - `evidence.md` for `adcs-evidence` and `catalogue/`: part status rules and the certificate;
  - `solver.md` for `adcs-case`, `adcs-config`, `adcs-tune`, `adcs-solve` and `designs/`: one supplier per value, blanks never guessed, tuned values restricted, candidates never shown to a client and never written to the repository by the software;
  - `intake.md` for `adcs-intake`, `forms/`, `manual/` and `tools/intake.py`: the checker is the authority, the page only helps; a received file's script never runs; interfaces are checked against the repository, never the form.

### 17.2 Human decisions

H1–H9 are the engine's human decisions. H1 (a relation) and H2 (a test vector) are made in a node form by the engineer under "Checked by", and reviewed on the intake branch. H10 is defined, as F10 requires. Seven more follow:

| | Decision | Who | How often |
|---|---|---|---|
| H10 | a change to the implementation agent's standing instructions (`AGENTS.md`, `intake/AGENT.md`), its scope hook, the brief the checker writes, or the model it runs on | the developer team's lead, with the intake selftest passing | per change |
| H11 · H-cert | issuing a certificate of conformance | quality | per order |
| H12 · H-rig | the rig safety check before the first powered test of a configuration | the test facility lead | per configuration |
| H13 · H-quote | price, terms and the export-classification check before a quote reaches a client | sales, with management for non-standard terms | per quote |
| H14 | promoting a part's status (placeholder → reference → qualified → flight-proven), or changing its values; offering a catalogue candidate, or retiring a product; confirming an algorithm's tuning bounds | the part's or product's owner and quality, recorded in the portal with their names (§8.6) | per change |
| H15 | the cause on a parity-ledger line | a domain engineer | per line |
| H16 · H-risk | the risk scale and every starting level (D26); lowering or closing a risk; the quarterly narrative's prose (D27) | the risk's owner team and quality; the prose by quality | per change; per quarter |

**Recording a person's decision.** A decision is recorded by the person who makes it, never typed into a file by someone else (rule 2):

- **H1, H2** (a relation, a test vector): in a node form, under "Checked by" (§5.8).
- **H14, H15, a panel's sign-off**: from P9, on the portal's decision page (`POST /api/v1/internal/decisions`), which writes a decision record with the person's name, role, date and reason. Before P9, the person sends a node form of type "something else" naming the decision, with their own name under "Checked by"; the saved form is the record.

`cargo xtask decision record <record>` reads either kind of record and copies the name, date and record id into the one field it decides (a product's `[promotion]`, an algorithm's `bounds_confirmed_by`, a panel's `confirmed_by`, a ledger line's cause). It refuses a record whose person is a tool or an assistant.

### 17.3 Reviewer counts

These are in the CONTRIBUTING table, which remains the only place the policy is stated:

| Change | Reviewers | Why |
|---|---|---|
| an intake branch (a request implemented) | one developer from the node's owner group (§5.9); two when the node is `significant`, when a new physics function is added, or when the request adds or removes a connection | the review checks the implementation against the form; the form's "Checked by" carries the domain judgement |
| `adcs-intake`, `tools/intake.py`, `forms/node_form.html`, `forms/case_editor.html`, or a new request kind | two | every request passes through them, and the checker is what decides what may reach a sheet |
| `AGENTS.md`, `intake/AGENT.md`, the scope hook | two (H10) | they bound what the implementation agent may do |
| `fsw/include/*.h`, the fswcfg or EEPROM formats | two | an ABI change reaches every rung and every customer build at once |
| `adcs-sim-core` | two | every campaign reads it |
| `devices/*.toml` | two, one of whom checks it against the part's ICD | a wrong register is a wrong measurement at every rung |
| a catalogue part's values or status | two, one from quality (H14) | a quote, a campaign and an EEPROM read it |
| `adcs-config/src/supply_map.rs` | two | a wrong line is a wrong supply in every candidate |
| the case format (`CASE_INPUTS`, `REQ_UNITS`, `META` in `tools/build_tree.py`) | two, and a version bump with a migration (§8.3.6); help text alone is one | every client's file is read through it |
| `catalogue/algorithms/*.toml` bounds | two, one from GNC | the box every tuning searches |
| a product's promotion to offered | written by `xtask decision record` from the recorded H14 decision, never on engineering review alone | from then on a client can buy it |
| a scenario's metric bindings | one, plus the owner of the bound requirement | it decides what passes |
| portal authentication, sessions or tenant code | two | a defect there exposes clients to each other |
| a manual page or a template | one; `manual_check.py`, `xtask manual --check` and `xtask explain check` green | a manual that is wrong is a request that is wrong; a page that does not explain itself is filled wrong |
| a risk lowered or closed in `derisk/risks.toml` | two, one from quality (H16), and the belief that lowers it must say what was tested | the Risk management conclusion reads it |
| a price table | sales and management, not engineering review | it is not engineering content |

### 17.4 The implementation agent

The ADCS platform has no fleet of specialised agents (§3.4). Declarations and test vectors now come from people, through node forms. What remains for an agent is the part a person should not have to type: turning a checked request into code.

**One general agent.** The implementation agent is Claude Code, started by a developer in a checkout, with one brief (§5.11.3). It is not specialised: the same agent implements a declared value, a relation with a new physics function, or a new node, because the brief says exactly what to do and where. It works under:

- `AGENTS.md` and `intake/AGENT.md`, its standing instructions;
- the scope hook in `.claude/hooks/`, which refuses a write to any path the brief does not name. `node.toml`, `fixtures.toml`, `versions.toml`, `versions/` and `derisk/` are refused to the agent's editor altogether: only `cargo xtask intake write` writes them;
- the brief, which names the node, the files, the steps, and the commands that prove the work.

**What proves its work is not the agent.** `intake verify` compares the node with the request field by field. The gate checks the HOLEs (F7's check 10b) and runs every test vector, which people transcribed from cited pages. A developer from the owner group reviews the branch. So an agent's mistake is caught by a check that does not share it.

**What it never does**, whatever a brief or a request says: supply an expected value; write a person's name; set or lower a risk level; widen a tolerance; skip a test; put a formula in a HOLE; edit outside its scope; push, merge or release. When a request cannot be implemented as written, it stops and says why, and the developer replies to the requester.

**Evaluated.** `cargo xtask intake selftest` runs the checker's 66 refusals, and the 82 seed forms through check, write and verify on a scratch tree freshly seeded from `plan/`. H10 requires it green for any change to the agent's instructions, hook, brief or model.

No agent promotes a product, confirms a node or signs anything. `adcs design` saves candidates as a tool run by a person or a scheduled job, with `origin = "designed"` and no name in `[promotion]`. `tools/validate_plan.py` and the gate refuse an `offered` product without a recorded person's decision (§8.6).

### 17.5 Commit scopes

They are derived from crate names, plus `catalogue`, `designs`, `scenarios`, `campaigns`, `rig`, `devices`, `fsw`, `plan`, `forms`, `manual`, `intake`, `matlab_sils` and `deploy`. An intake commit's scope is its node's crate, and its trailer is `Request: <request id>`.

---

## 18. Continuous integration

`gate.yml`'s `tooling` job keeps an explicit list of selftests. Not every script has one: `seed_tree.py` refuses arguments by design, `build_tree.py` takes only `--check`. So the job names each script it runs, and never globs `tools/*.py`. The jobs are: `gate`, `tooling`, `review` (advisory), `mutants`, `panels`, `excluded-faces`, `shipping-profile` (now also building `--profile user`, F4) and `no-std`. The `no-std` job extends to `adcs-sim-core` on `thumbv7em-none-eabihf`. It adds:

| Job | Runs | Green means |
|---|---|---|
| `plan` | `tools/validate_plan.py --selftest` (28 deliberate breakages, each caught) and the full check; `tools/build_tree.py --check` (the tree, the case registry and the case template); the seeded node ids against `plan/expected_node_ids.json` | the plan package is consistent, the generated files are current, and the tree seeded as recorded |
| `intake` | `cargo xtask intake selftest` (and `tools/intake.py selftest` while the package's tools are in use); on an `intake/*` branch, `intake check` and `intake verify` of the request the branch names, then `intake mark verified` when they and the gate pass | the checker refuses what it must and passes what it should; an intake branch implements exactly its request |
| `no-writes` | a scan of the faces, the daemon, the web face and the portal for any route or command that writes a sheet, a layer, the catalogue, a scenario or a campaign; `the_manual_is_true` | the released software cannot change itself (F17) |
| `cases` | `adcs case import` of every `plan/cases/*.csv` and class standard, then `adcs case export`, compared byte for byte; `adcs case check` on each; a malformed-file suite (wrong unit, unknown key, missing key, lo without hi, level on an input) that must each be refused | the format is read one way, and refuses what it should |
| `forms` | `tools/form_browser_check.py` (§5.10.8, §8.3.6, §13.5.6); two exports of one node at one commit, compared | a request a team member fills is a request the developers can check; a case the editor writes is a case the software reads |
| `results` | §13.5.6: platform and MATLAB results for `detumble_3u` each open in the other's viewer; the decoded channels equal the recorder's; the index rebuilds identically; import into an empty store files each result beside its case; the restricted-content test runs over a result | a result is the same file whichever engine wrote it, and opening one runs nothing |
| `manual` | `cargo xtask manual --check` (from P1); `tools/manual_check.py` on every command block whose phase is green | the manuals say what the software does, as far as it is built |
| `derisk` | `cargo xtask derisk check` and `selftest` (`tools/derisk.py` while the package's tools are in use); `derisk table --check --file docs/RISKS.md`; on a pull request touching `crates/adcs-sim-core`, `crates/adcs-sim`, `crates/adcs-tune`, `crates/adcs-solve`, `crates/adcs-result`, `web/`, `forms/`, `results/`, `catalogue/algorithms/` or `scenarios/`, a new belief record or the label `derisk:none` with a reason | the ledger keeps its rules; no change rests on an unrecorded belief; no risk goes down without a test |
| `explain` | `cargo xtask explain check` and `--selftest` over every template's rendered examples and every manual page (`tools/explain_check.py`); `tools/explain_kit.py --check` | every page a person reads carries the marks of `adcs-explain/1`, and every template carries the current kit |
| `twin` | from P1: `cargo xtask twin check` and `--repo . --phase <reached>`; on a pull request, `twin check --changed` over its changed paths, honouring the label `twin:none` with a reason (§10.8.7) | every SILS element the phase builds exists in both engines, and no change moves one engine without the other |
| `matlab-pack` | from P1, on every push: `tools/pack_matlab.py --phase <reached>` twice, compared byte for byte; on a release, the zip attached | the twin's download is current, whole for the phase reached, and reproducible |
| `matlab-twin` | from P1, on a MATLAB runner (`matlab-actions/setup-matlab`, D5): the twin's `matlab.unittest` suite. It covers the physics test vectors, the 1,000 reference draws (from P3), the case-reader refusal suite shared with the `cases` job, each element's shared test from the twin map, and every example script (from P3M) | the twin runs from the zip on base MATLAB |
| `matlab-parity` | §10.8.6: both engines on the golden scenarios; ledger lines written | advisory until D23 says which differences fail |
| `solve-smoke` | `adcs solve ais_3u` and `adcs solve ais_img_3u` (expected: no catalogue product, with the gap report); `adcs design` on a two-combination job with `evaluations = 4`, saving to a temporary store | the solver and the designer start, finish and write what they claim, and nothing in the repository |
| `sim-determinism` | a golden scenario on x86-64 native, on wasm32 under `wasmtime`, and on aarch64 (native runner or QEMU); the trajectory hashes are compared | `adcs-sim-core` is bit-identical across the three |
| `sim-smoke` | `detumble_3u` and `inertial_hold_3u` nominal with the duration cut to one tenth; a 20-run Monte Carlo | runs start, finish, record and produce every bound metric |
| `fsw` | CMake build of `fsw/` with `-std=c99 -Wall -Wextra -Werror -ffp-contract=off`; unit tests; `cppcheck`; a scan refusing `malloc`, `free`, `time`, `rand` and recursion; the init-twice test (§9.6) | the reference flight software builds clean and is repeatable |
| `rig-mock` | an OILS campaign against the mock IEU, and a HILS campaign against the mock environment simulator, at real time for 60 s | the rig loop keeps its deadline on a CI runner, and every mock device answers |
| `portal` | unit tests; integration tests against a PostgreSQL service container; the migration up and down tests; the tenant-isolation suite; the auth suite | no route leaks across tenants or roles |
| `evidence` | an evidence package built twice from one ledger, compared byte for byte; the restricted-content exclusion test (§14.6) | packages are reproducible and carry nothing restricted |

`nightly.yml` adds:

- `inertial_hold_mc500` in full, its metric distributions compared with the last ledger entry; a moved distribution opens an issue for the developer team, never fails silently;
- the parity report against every scenario's parity reference;
- `mutate` and `bundle verify`.

`release.yml` is prove → build → publish with its fail-closed approval, preceded by one release-preparation commit: `cargo xtask intake mark published --release <v>` for every node whose request the release carries, so the sheets that are built say `published` and each new version carries its release; and `cargo xtask derisk rollup`, which writes `derisk/rollup.toml` for the Risk management rows. Its artefacts are the CLI, the daemon, the FFI library, `adcs-rig` (Linux only), a container image with `adcs-portal` and `adcs-worker`, `adcs_sils_matlab_<version>.zip`, the node library, the user manual and the quarter's de-risking narrative. Its release notes list every request the release carries, by request id and node, with the belief each rested on, so each requester can find theirs.
