
---

## 3. What is inherited from VLEO_SIMULATOR

### 3.1 The pin

Clone `https://github.com/AmanRai0603/VLEO_SIMULATOR` at commit `abf79ee` into a scratch directory. Treat it as read-only. The new repository is created empty, and files are copied into it phase by phase (§19). VLEO_SIMULATOR is not a git submodule or a dependency of the new repository. The two products share a method, not a build. Extracting a shared platform later is decision D10 (§20).

### 3.2 The five rules, for ADCS

Four carry over word for word; rule 1 gains how the sheet is written, and rule 3 changes only its crate name.

1. **The sheet is the only source.** Its content is written by `cargo xtask intake write` from a checked request, and by nothing else; its shape by the seeder and, for a new node, `cargo xtask new` (§0.2 rule 6). Every other file in a node folder is generated, except the typed lines inside its numbered HOLE blocks.
2. **An expected value may never come from the code under test.** The gate refuses `self-snapshot` and `agent-generated`.
3. **Every formula lives in `adcs-core::physics` and nowhere else.** The loop engine's models live in `adcs-sim-core` (§9) and call the same `physics` functions wherever a relation is shared. The ring momentum in a sheet and the ring momentum in the plant are one function.
4. **Portable maths only.** `adcs_core::units::pmath` in the kernel, in `adcs-sim-core`, and in every hole body.
5. **A refusal is never a substitution.**

### 3.3 The rename map

| VLEO_SIMULATOR | ADCS platform |
|---|---|
| crate prefix `vleo-` / `vleo_` | `adcs-` / `adcs_` (every crate, every generated `use` line) |
| `vleo-mod-*` node crates | `adcs-mod-*` (§5.4 lists them) |
| `struct Vleo` in `vleo-modules` | `struct Adcs` in `adcs-modules` |
| binary `vleo`, cargo alias `vleo` | binary `adcs`, cargo alias `adcs` |
| binary `vleo-daemon` | `adcs-daemon`: the workbench, at http://127.0.0.1:7787 |
| `VLEO_DATA`, `VLEO_PORT` | `ADCS_DATA`, `ADCS_PORT` |
| `VLEO_ALLOW_WRITE` | removed, with every route and control it guarded (§3.4) |
| default port 7777 | 7787, so both tools can run on one machine |
| default case `"nominal"` (hard-coded in `vleo-cli`, `vleo-daemon`, `vleo-ffi`, `vleo-py`) | `"ais_3u"`, the 3U AIS default case; the ADCS plan has no `nominal` case |
| store `~/.vleo/data`, `vleo.lock`, `/.vleo/` | `~/.adcs/data` (bundles), `~/.adcs/store` (cases and results, §13.5.4), `adcs.lock`, `/.adcs/` |
| `cd06/tree.json` | `plan/tree.json` (authored, from `tools/build_tree.py`) |
| `tools/cd06_rows.py` | `tools/plan_rows.py` (maps in §5.7) |
| `tools/cd06_extract.py` | dropped: the tree is authored, not extracted from HTML |
| `layers/root.toml` label "VLEO multipayload programme" | "ADCS products and test facility" |
| `matlab/+vleo`, `vleo_install.m` | `matlab/+adcs`, `adcs_install.m` (add the missing `version.m`) |
| fault text ``run `vleo data sync` `` | ``run `adcs data sync` `` |
| `.devcontainer` port label "VLEO design tool" | "ADCS platform" |

Commit scopes are derived from crate names by `tools/commit_message.py` (`removeprefix`), so change the prefix there and the scopes follow. Reset `ADOPTED_AFTER` to the new repository's first commit.

### 3.4 Copy, adapt, drop

**Copy verbatim, then rename (§3.3) only:**

- `crates/vleo-units`, `vleo-core` (minus the physics modules below), `vleo-bus`, `vleo-data`, `vleo-modules`, `vleo-ffi`, `vleo-wasm`, `vleo-py`;
- `.cargo/`, `rustfmt.toml`, `rust-toolchain.toml`, `.gitignore`;
- `.claude/hooks/*.sh` and `.claude/settings.json`;
- from `tools/`: `branch_audit.py`, `commit_message.py`, `crate_skeleton.py` (with its `ALIAS` map set to `{"x_closure": "closure"}`), `input_check.py`, `instruction_lint.py`, `manual_check.py`, `node_crate_build.rs`, `panel_check.py`, `panel_review.py`, `release_notes.py`, `review_report.py`, `seed_helpers.py`, `template_check.py`, `theory_check.py`, `githooks/commit-msg`, and `selftest_panels/` (which `panel_check.py --selftest` reads);
- `.github/pull_request_template.md`, `.github/dependabot.yml`;
- `panels/tree.toml`, `matrix.toml`, `paths.toml`, `panels/README.md`.

**Copy and rename, then remove every editing path:** `crates/vleo-sheet`, `vleo-cli`, `vleo-daemon`, `xtask/` and `web/`. The released software reads and runs; it never writes its own content (§1.6). Each removal is complete: the code, its tests, its manual entries and its help text go, and `the_manual_is_true` proves nothing still names them.

| Where | What is removed | What stays |
|---|---|---|
| `vleo-sheet/src/form.rs` | `save`, `save_block`, `preview`, `propose`, `set_view` and `publish`, and every caller | `FIELDS`, `ARRAYS`, `structural`, `normalise`, `value_allowed` and the pure `set`: `cargo xtask intake write` renders a sheet with them (§5.11), and the node form's asks are FIELDS' asks |
| `vleo-daemon` | `POST /v1/propose`, `/v1/preview/…`, `/v1/sheet/…`, `/v1/block/…`, `/v1/view/…`, `/v1/publish/…`, and `VLEO_ALLOW_WRITE` | every `GET` route, `/v1/run`, `/v1/sweep`, `/v1/probe`, `/v1/levers`, the read-only `GET /v1/declare/…` |
| `vleo-sheet/src/manual.rs` | `Route.writes` and the "writes" effect | the manual, and `the_manual_is_true` |
| `web/js/sheet.js` | the edit form, the paste-and-preview box, "put these edits on a branch" | the sheet's read-only view |
| `web/js/manual.js` | the text telling a user how to edit or propose from the page | everything else |
| `xtask` | `declare` (as an editing loop), `fill`, `confirm` | `docs`, `assemble`, `gate`, `status`, `active`, `reach`, `gap`, `graph`, `new` (used only by intake), `ready`, `codeowners`, `bundle`, `variables`, `setup`, `mutate`, `differential`; added: `intake`, `form`, `manual`, `decision` and `rng-reference` (§16.2) |
| `vleo-cli` | any subcommand that writes a sheet | every read and run subcommand |

`web/` is copied except `js/solar.js` (F13 and F14 are the two edits that removal needs) and the removals above.

**Copy the structure, rewrite the content:**
`AGENTS.md` (now the standing instructions of the builder and the implementation agent, §17.4), `CONTRIBUTING.md`, `README.md`, `ADOPTION.lock`, `areas/*.md`, `docs/ARCHITECTURE.md`, `docs/NODE_AUTHORING.md` (now how a node is specified in a node form, and what intake writes from it), `docs/USING_IT.md`, `docs/RUNBOOK.md`, `docs/RELEASE_SETUP.md`, `docs/DELIVERY_PLAN.md`, `docs/manual.toml`, `.github/workflows/*.yml`, `.devcontainer/*`, `tools/seed_tree.py`, `tools/cd06_rows.py` (as `plan_rows.py`), `matlab/`, `vleo-core/src/physics/gnc.rs` (kept and extended), `vleo-core/src/physics/mission.rs` (keep `Sense`, `Closure`, `closure()` exactly: gate check 7e reads the literal strings `Sense::AtMost` and `Sense::AtLeast`).

**Drop:**

- **The agent fleet**: `agents/lanes.toml`, `agents/provenance.toml`, `.claude/agents/*.md` (the declaration-drafter, hole-filler, fixture-recorder, test-author, diagnostician, systems-backend and frontend agents), `tools/agent_lanes.py`, `tools/fleet_report.py`, and `docs/WORK_MODEL.md`. Node content comes from people through forms. Code comes from one general implementation agent under a brief (§17.4).
- **VLEO's own content**: every `crates/vleo-mod-*/nodes/*` folder; `vleo-core/src/physics/{aero,comms,cost,env,mass,orbit,payload,power,prop,thermal}.rs` (the ADCS equivalents are written fresh in §6.2, reading these for style); `bundles/solar-*`; `cases/*.toml`; `layers/*.toml`; `sources/sources.toml`; `cd06/`; the 11 solar panel specs and their reference images.
- **VLEO's own documents and scripts**: `docs/MATLAB_PORT_PLAN.md`, `docs/SOLAR_INVENTORY.md`, `docs/SOLAR_ROWS.md`, `docs/GITLAB_TRANSFER.md`, `docs/VARIABLES.md` (regenerated), `docs/AGENT_EVIDENCE.md` (restarted empty as `docs/BUILD_EVIDENCE.md`), `tools/nodes/*.py`, `tools/cd06_extract.py`, `tools/band_confidence_cost.py`, `tools/kp_slot_cost.py`, `tools/rotation_residuals.py`, `tools/solar_inventory.py`, `tools/solar_rows.py`, `tools/matlab_parity.py`. `tools/mat_parity.py` is rewritten as `tools/sil_parity.py` (§10.7).

### 3.5 Defects in VLEO_SIMULATOR to fix while porting

These were found by reading the code at `abf79ee`. Each is fixed in the port and recorded in `docs/SPEC_DEVIATIONS.md`. The ones that touch the generator or the gate are H7 changes, so two reviewers.

| # | In VLEO_SIMULATOR | Fix in the ADCS port |
|---|---|---|
| F1 | `emit.rs::kind_variant` turns an unknown `kind` into `Kind::Computed`; `state_variant` turns an unknown `state` into `State::Published`. A typo becomes a runnable row silently, which breaks rule 5. | Unknown `kind` or `state` is a load error naming the sheet and the value. |
| F2 | `prov_variant` maps an unknown provenance to `AgentGenerated`. It fails safe, but says the wrong thing. | Unknown provenance is a load error naming the string. |
| F3 | `vleo-wasm` links the full `vleo-modules` and filters by the `DEMONSTRATION` list at run time, so the whole kernel ships in the browser. | `adcs-wasm` compiles only the demonstration subset, behind a cargo feature and a generated subset table. A client-facing page must not carry the restricted rows (D1, D2). V13 reads the subset from the generated table. |
| F4 | `DELIVERY_PLAN.md` describes a `user` build profile; the root `Cargo.toml` has none. | Add `[profile.user]` (inherits release; strip, lto fat, panic abort, one codegen unit) and build it on every merge, as the plan already says. |
| F5 | Root `Cargo.toml` declares `vleo-graph` and `vleo-testkit` workspace dependencies that do not exist. | Omit them. |
| F6 | `governing_node(store, subtree_root)` scans every node; `subtree_root` is not a filter. | Filter to the dependency closure of `subtree_root`. The portal reports the governing factor per KPI, so this becomes load-bearing. |
| F7 | Docs claim a gate check refusing a formula inside a node; only `portable-maths` scans hole bodies. | Implement check 10b: a hole body may call `physics::` functions, arithmetic on its bindings and named constants, and nothing that computes a relation. The exact rule is H7; stop at §20 D11. With the implementation agent writing every hole, this check is what keeps rule 3 true. |
| F8 | `Fault::DataUnverified` exists and is never raised. | Raise it when a bundle's hash does not match `adcs.lock`. |
| F9 | Help text and the CODEOWNERS gate step say "from the layer files"; the generator reads each sheet's `owner`. | Correct the wording; the behaviour is right. |
| F10 | `WORK_MODEL.md` says four agents; seven are defined. `provenance.toml` cites H10, which WORK_MODEL never defines. Prose row counts disagree (1396, 1329). | The fleet and `WORK_MODEL.md` are dropped (§3.4). Every H-number used is defined in §17.2; counts are written from `xtask status`; `instruction_lint.py` checks both. |
| F11 | `matlab/+vleo/Contents.m` documents `version.m`, which does not exist. | Write `matlab/+adcs/version.m`. |
| F12 | The Uncertainty credibility factor is a proxy keyed on the node id containing "uncertainty". | Keep the proxy for closure rows; for evidence rows (§5.5), score Uncertainty from the campaign's run count and stated confidence (§5.5 table). Built in P4. |
| F13 | `web/js/node.js` imports `solar.js`, and `app.js` imports `node.js`. Dropping `solar.js` breaks the whole module graph, so the page does not load. | Remove the import and its calls from `node.js`. |
| F14 | `web/js/state.js` reads `S.index.cases[0].id`; with no cases (P0) it throws. The case filter also shows a group only when its `cases` list names the selected case, which cannot work for cases the portal creates at run time. | Guard the empty list. Filter groups by hardware tags (§5.6): a group is shown when its tags meet the fitted-hardware tags of the loaded candidate (the case with its product). |
| F15 | Gate check V13 fails when the `DEMONSTRATION` list is empty, and fails any listed row that is not `published`. An empty tree (P0) and a tree of seeded rows (P1–P4) can never pass it. | V13 passes on an empty list until D2 decides the subset; a listed row must be runnable (`verified` or later), not `published`. H7. |
| F16 | VLEO compiles its cases into the binary: `vleo-sheet/src/load.rs` `load_cases` feeds a static table that `vleo-modules` iterates. That cannot serve cases a portal creates at run time. | Cases become data loaded at run time from the case store (§8.3), never compiled in. `adcs-modules` loses its `CASES` table. `--case <id>` resolves through the store. Every face keeps its default case `ais_3u`, which is imported, not compiled. H7. |
| F17 | The daemon and the web face can write sheets (`VLEO_ALLOW_WRITE`), and `xtask fill`, `confirm` and `declare` let anyone at a terminal change a sheet outside any check. Two write paths that bypass review are two ways for a released tool to drift from its repository. | Every such path is removed (§3.4). A sheet changes only through `cargo xtask intake write` from a checked request, and `intake verify` proves the result. H7. |
