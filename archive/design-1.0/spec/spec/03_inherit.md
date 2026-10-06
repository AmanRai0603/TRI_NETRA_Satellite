
---

## 3. The method and the workspace

### 3.1 One repository, built from this package

The repository is built from this package alone, phase by phase (§19). It has no upstream: nothing is cloned, pinned or copied from another product, and no other repository is a submodule or a dependency. Whether the engine's method is later extracted into a library other products share is decision D10 (§20).

### 3.2 The five rules, for ADCS

1. **The sheet is the only source.** Its content is written by `cargo xtask intake write` from a checked request, and by nothing else; its shape by the seeder and, for a new node, `cargo xtask new` (§0.2 rule 6). Every other file in a node folder is generated, except the typed lines inside its numbered HOLE blocks.
2. **An expected value may never come from the code under test.** The gate refuses `self-snapshot` and `agent-generated`.
3. **Every formula lives in `adcs-core::physics` and nowhere else.** The loop engine's models live in `adcs-sim-core` (§9) and call the same `physics` functions wherever a relation is shared. The ring momentum in a sheet and the ring momentum in the plant are one function.
4. **Portable maths only.** `adcs_core::units::pmath` in the kernel, in `adcs-sim-core`, and in every hole body.
5. **A refusal is never a substitution.**

### 3.3 Names

| Thing | Name |
|---|---|
| crate prefix | `adcs-` / `adcs_` (every crate, every generated `use` line) |
| node crates | `adcs-mod-*` (§5.4 lists them) |
| the kernel's model type | `struct Adcs` in `adcs-modules` |
| command-line binary, cargo alias | `adcs` |
| the workbench | `adcs-daemon`, at http://127.0.0.1:7787 |
| settings | `ADCS_DATA`, `ADCS_PORT`; there is no setting that allows writing (§3.4) |
| default case | `"ais_3u"`, the 3U AIS reference case |
| local store | `~/.adcs/data` (bundles), `~/.adcs/store` (cases and results, §13.5.4), `adcs.lock`, `/.adcs/` |
| the tree | `plan/tree.json` (authored, from `tools/build_tree.py`) |
| the seeder's maps | `tools/plan_rows.py` (§5.7) |
| the root layer's label | "ADCS products and test facility" |
| the MATLAB package | `matlab/+adcs`, installed by `adcs_install.m`, with `version.m` |
| the fault text for missing data | ``run `adcs data sync` `` |
| the dev container's port label | "ADCS platform" |

Commit scopes are derived from crate names by `tools/commit_message.py` (`removeprefix("adcs-")`). `ADOPTED_AFTER` is the repository's first commit.

### 3.4 The workspace

**The engine.** `crates/adcs-units`, `adcs-core`, `adcs-bus`, `adcs-data`, `adcs-modules`, `adcs-ffi`, `adcs-wasm` and `adcs-py`; `adcs-sheet` (sheet loading and generation), `adcs-cli`, `adcs-daemon`, `xtask/` and `web/`; `.cargo/`, `rustfmt.toml`, `rust-toolchain.toml`, `.gitignore`, `.claude/hooks/*.sh` and `.claude/settings.json`.

**The tools.** In `tools/`: `branch_audit.py`, `commit_message.py`, `crate_skeleton.py` (with its `ALIAS` map set to `{"x_closure": "closure"}`), `input_check.py`, `instruction_lint.py`, `manual_check.py`, `node_crate_build.rs`, `panel_check.py`, `panel_review.py`, `release_notes.py`, `review_report.py`, `seed_helpers.py`, `seed_tree.py`, `plan_rows.py`, `template_check.py`, `theory_check.py`, `sil_parity.py` (§10.7), `githooks/commit-msg`, and `selftest_panels/` (which `panel_check.py --selftest` reads); `.github/pull_request_template.md`, `.github/dependabot.yml`; `panels/tree.toml`, `matrix.toml`, `paths.toml`, `panels/README.md`.

**No editing path.** The released software reads and runs; it never writes its own content (§1.6). The faces are built without any route, control, command or setting that changes a sheet. `the_manual_is_true` proves nothing in the manual or the help text names one.

| Where | Never present | Present |
|---|---|---|
| `adcs-sheet/src/form.rs` | `save`, `save_block`, `preview`, `propose`, `set_view`, `publish` | `FIELDS`, `ARRAYS`, `structural`, `normalise`, `value_allowed` and the pure `set`: `cargo xtask intake write` renders a sheet with them (§5.11), and the node form's asks are FIELDS' asks |
| `adcs-daemon` | any route that writes: propose, preview, sheet, block, view, publish; any setting that allows writing | every `GET` route, `/v1/run`, `/v1/sweep`, `/v1/probe`, `/v1/levers`, the read-only `GET /v1/declare/…` |
| `adcs-sheet/src/manual.rs` | a "writes" effect on a route | the manual, and `the_manual_is_true` |
| `web/js/sheet.js` | an edit form, a paste-and-preview box, "put these edits on a branch" | the sheet's read-only view |
| `web/js/manual.js` | text telling a user how to edit or propose from the page | everything else |
| `xtask` | `declare` as an editing loop, `fill`, `confirm` | `docs`, `assemble`, `gate`, `status`, `active`, `reach`, `gap`, `graph`, `new` (used only by intake), `ready`, `codeowners`, `bundle`, `variables`, `setup`, `mutate`, `differential`, `intake`, `form`, `manual`, `decision` and `rng-reference` (§16.2) |
| `adcs-cli` | any subcommand that writes a sheet | every read and run subcommand |

**Written for ADCS.** `AGENTS.md` (the standing instructions of the builder and the implementation agent, §17.4), `CONTRIBUTING.md`, `README.md`, `ADOPTION.lock`, `areas/*.md`, `docs/ARCHITECTURE.md`, `docs/NODE_AUTHORING.md` (how a node is specified in a node form, and what intake writes from it), `docs/USING_IT.md`, `docs/RUNBOOK.md`, `docs/RELEASE_SETUP.md`, `docs/DELIVERY_PLAN.md`, `docs/manual.toml`, `.github/workflows/*.yml`, `.devcontainer/*`, `matlab/`, and `adcs-core/src/physics/`, whose `gnc.rs` and `mission.rs` hold the base functions (`mission.rs` defines `Sense`, `Closure` and `closure()`; gate check 7e reads the literal strings `Sense::AtMost` and `Sense::AtLeast`). `docs/BUILD_EVIDENCE.md` starts empty.

**Not part of it.** No fleet of specialised agents: node content comes from people through forms, and code from one general implementation agent under a brief (§17.4). No content outside ADCS: every node folder, physics module, bundle, case and layer file is the ADCS platform's own.

### 3.5 Faults the build must not have

Each is a way a platform of this kind goes wrong. Each is designed out from the start and recorded in `docs/SPEC_DEVIATIONS.md` if the build departs from it. The ones that touch the generator or the gate are H7 changes, so two reviewers.

| # | The fault | What the platform does |
|---|---|---|
| F1 | An unknown `kind` becomes `Kind::Computed`, or an unknown `state` becomes `State::Published`: a typo becomes a runnable row silently, which breaks rule 5. | Unknown `kind` or `state` is a load error naming the sheet and the value. |
| F2 | An unknown provenance maps to `AgentGenerated`. It fails safe, but says the wrong thing. | Unknown provenance is a load error naming the string. |
| F3 | The browser build links the whole kernel and filters the demonstration list at run time, so the whole kernel ships in the browser. | `adcs-wasm` compiles only the demonstration subset, behind a cargo feature and a generated subset table. A client-facing page must not carry the restricted rows (D1, D2). V13 reads the subset from the generated table. |
| F4 | A `user` build profile is described but never defined. | `[profile.user]` (inherits release; strip, lto fat, panic abort, one codegen unit), built on every merge. |
| F5 | The root `Cargo.toml` declares workspace dependencies that do not exist. | Declare only crates that exist. |
| F6 | `governing_node(store, subtree_root)` scans every node; `subtree_root` is not a filter. | Filter to the dependency closure of `subtree_root`. The portal reports the governing factor per KPI, so this is load-bearing. |
| F7 | Docs claim a gate check refusing a formula inside a node while only `portable-maths` scans hole bodies. | Implement check 10b: a hole body may call `physics::` functions, arithmetic on its bindings and named constants, and nothing that computes a relation. The exact rule is H7; stop at §20 D11. With the implementation agent writing every hole, this check is what keeps rule 3 true. |
| F8 | `Fault::DataUnverified` exists and is never raised. | Raise it when a bundle's hash does not match `adcs.lock`. |
| F9 | Help text says owners come "from the layer files" while the generator reads each sheet's `owner`. | The wording says the sheet's `owner`, which is what the generator reads. |
| F10 | Prose counts disagree with the software (agents, rows), and an H-number is cited that is never defined. | Every H-number used is defined in §17.2; counts are written from `xtask status`; `instruction_lint.py` checks both. |
| F11 | A MATLAB `Contents.m` documents a `version.m` that does not exist. | Write `matlab/+adcs/version.m`. |
| F12 | The Uncertainty credibility factor is a proxy keyed on the node id containing "uncertainty". | Keep the proxy for closure rows; for evidence rows (§5.5), score Uncertainty from the campaign's run count and stated confidence (§5.5 table). Built in P4. |
| F13 | One web module imports a module the build does not have, and the whole module graph fails to load. | Every import resolves; the panels job loads the page. |
| F14 | The web state reads `S.index.cases[0].id` and throws with no cases (P0); a case filter shows a group only when its `cases` list names the selected case, which cannot work for cases the portal creates at run time. | Guard the empty list. Filter groups by hardware tags (§5.6): a group is shown when its tags meet the fitted-hardware tags of the loaded candidate (the case with its product). |
| F15 | Gate check V13 fails when the `DEMONSTRATION` list is empty, and fails any listed row that is not `published`, so an empty tree (P0) and a tree of seeded rows (P1–P4) can never pass it. | V13 passes on an empty list until D2 decides the subset; a listed row must be runnable (`verified` or later), not `published`. H7. |
| F16 | Cases compiled into the binary as a static table cannot serve cases a portal creates at run time. | Cases are data loaded at run time from the case store (§8.3), never compiled in. `adcs-modules` has no `CASES` table. `--case <id>` resolves through the store. Every face keeps its default case `ais_3u`, which is imported, not compiled. H7. |
| F17 | A face or a terminal command that writes sheets outside any check is a way for a released tool to drift from its repository. | No such path exists (§3.4). A sheet changes only through `cargo xtask intake write` from a checked request, and `intake verify` proves the result. H7. |
