# ADCS Platform — Build Specification

| | |
|---|---|
| Document | ADCS-SPEC-01 · the build specification for the ADCS platform repository |
| Version | 2.0 · 27 September 2026 |
| Owner | Aman Kumar Rai |
| Product content from | IDMAS v2 Exploration (IDMAS V1 coils, V2 fluid rings, V3 RCS) |
| Read by | Claude Code, which builds the repository from it; the developer team, who maintain it |

This document, and the files beside it, are everything needed to build one repository: the ADCS company's platform.

- **Inside the company**, the platform takes a customer's case, one CSV in a fixed format, to a catalogue product tuned and proven in SILS for that case. When the catalogue has none, the design team's runs of the same engine design one.
- **As a test facility**, it tests that ADCS in SILS, PIL, OILS and HILS on one scenario template, and issues the certificate.
- **Towards the client**, it offers SILS with visualisation before the order. After the purchase order, it gives OILS and HILS results with visualisation, plus the certification.

The repository is a sheet-driven engine with an ADCS tree in it (§3). Beside the engine it has seven parts:

1. case import from a fixed CSV;
2. a solver and designer over a catalogue of products;
3. a time-stepping loop engine;
4. a real-time rig;
5. a portal;
6. a plain-MATLAB twin of the SILS engine;
7. an intake that turns a team member's node form into checked, implemented and released software.

This document specifies every part: the engine's method in §3, the rest in the sections that follow.

**How the software is run and changed (§1.6).** The developer team builds, maintains and upgrades the software. Nobody else changes it, and the released software never changes itself. Everyone else uses it and gets two things:

- **the released software.** Their only input is a case CSV. Everything else is a feature they use: the views, the node documents, runs, and saved results that reopen without re-running.
- **the node form.** Any request to change the software starts with it: a changed node, a new node, a confirmation, or feedback on a release. The developer team checks the form, implements it, verifies it against the form, reviews and releases it. They answer in the same file.

Two things run through all of it. **Every page a person reads explains itself** to one standard, `adcs-explain/1`, taken from *Eight Loops, One Beam* (§5.12). And **every decision rests on a recorded belief** (§5.13): a node gets a new version only because a belief broke or a risk needs lowering, each version says what was wrong with the last and what it gives, and the Risk management branch of the tree concludes, release by release, how much of what the platform rests on has met evidence.

---

## 0. How to use this document

### 0.1 The package

| Path | What it is | Read by |
|---|---|---|
| `SPEC.md` | this document, assembled from `spec/` | everyone |
| `CLAUDE_CODE_PROMPT.md` | the prompt that starts each build phase | the person driving Claude Code |
| `RELEASE.md` | the package's release notes: what it adds, its counts, the checks run with their results | the developer team |
| `manual/user/` | the user manual: what a team member reads to use the released software and to send a request (§16.4) | team members; shipped in every release and in the MATLAB zip |
| `manual/developer/` | the developer manual: how the developer team receives a request, checks it, implements it, verifies, reviews, releases and replies (§16.4) | the developer team |
| `plan/tree.json` | the ADCS tree in its seven-key shape: 418 rows, 307 edges, one door | the seeder (§5.7) |
| `plan/case_template.csv`, `plan/case_inputs.toml` | the fixed case format `adcs-case/1`: the blank template (54 inputs, 5 meta rows, each explained in its note) and its registry, which also records the supplier of every declared layer-2 row; both generated | the case importer (§8.3), the case editor, the case checker |
| `plan/cases/*.csv` | four reference cases in that format: the two default 3U cases, `ais_3u` (10° pointing) and `ais_img_3u` (0.01°), plus a 150 kg bus and an unstated 12U | `adcs case import`, the pilot |
| `plan/expected_node_ids.json` | the node id the seeder gives each tree row | cross-checks in P1 |
| `plan/seed_content.toml` | the pilot thread's 61 rows and the risk branch's 21, written from cited sources (26 in all, the OILS and HILS facility references among them), with 7 test vectors transcribed from them and the spin-down node's own explanation (the standard's worked example). It is the content of the 82 seed forms (§5.8). | `tools/forms.py seeds`, then intake |
| `plan/kpis.toml` | the 22 KPIs: requirement, evidence row, analysis row, metric, usual sense, closure slug | the seeder's closures, the campaign runner, `validate_plan.py` |
| `derisk/risks.toml`, `derisk/beliefs/*.toml` | the risk register (18 risks, from §21, levels proposed until D26) and 16 belief records for the decisions this package took, most honestly untested (§5.13) | `tools/derisk.py`, the Risk management rows, intake |
| `derisk/narrative_template.xlsx` | the company's quarterly de-risking narrative template, its seven columns | the narrative (§5.13) |
| `plan/units.toml`, `plan/physics.toml` | the units and quantities a node may declare (the base registry plus §6.1), and the functions of `adcs-core::physics` (§6.2) | the node form's pick-lists, the intake checker |
| `catalogue/schema.toml` | the module descriptor standard | solver, loop engine, FSW config, EEPROM |
| `catalogue/families.toml` | the four configuration families, their slots and the algorithms each may carry | solver |
| `catalogue/products/*.toml` | 5 seeded products, all `candidate`: configuration, counts, mounts, algorithms | solver, loop engine |
| `catalogue/algorithms/*.toml` | 4 flight algorithms with their tunable parameters and bounds | tuner (§8.5) |
| `catalogue/classes.toml`, `catalogue/classes/*.csv` | 7 satellite classes; the 3U class's two standard cases | designer (§8.6) |
| `catalogue/parts/*.toml` | 13 part descriptors: 6 IDMAS parts (reference or placeholder) and 7 synthetic parts (`SYN-*`) | everything that reads a part |
| `designs/*.toml` | two design jobs, the 3U class against each standard | `adcs design` |
| `scenarios/*.toml` | 5 scenarios in the one template every rung reads | loop engine, rig, MATLAB twin |
| `campaigns/*.toml` | 7 campaigns, covering every campaign type | campaign runner |
| `fsw/include/adcs_hal.h`, `adcs_fsw.h` | the C boundary between flight software and every rung; compile clean as C99 and C++17 | flight software, loop engine, rig |
| `rig/device_maps/example_3u.toml`, `rig/labs/{hils_bay1,syn_lab}.toml` | the OILS/HILS wiring, and the lab as a plant and as a facility: rig host, IEU, field cage, air bearing, Sun and star stimulators, test stands, safety and power (the real bay, every value `nan` until measured, and a synthetic lab for tests); the tree's facility rows read their capabilities from these files (the `lab` supplier) | rig, lab twin, `adcs rig fit`, the tree |
| `devices/SYN-MAG-1.toml` | an example device protocol file (§9.5) | loop engine, IEU |
| `forms/node_form.html` | the node form, `adcs-node-form/1` (§5.10): one offline HTML file that is a node's document, the request form and, once saved, the request | team members; the exporter |
| `forms/case_editor.html` | the case editor, `adcs-case-editor/1` (§8.3.3): writes the case CSV, with the readiness report | team members, clients |
| `forms/library.html` | the node library's index page (§5.10.5) | the exporter |
| `forms/examples/` | node forms for `gf_7` (the explanation standard's worked example), `p1k_0` and `rk4_0` (a Risk management conclusion), the seed form for `gf_7`, a new-node request, a returned request with its belief record and the developer's replies, and the case editor filled from `ais_img_3u` and blank; all written by `tools/make_examples.py` | people, to see what each is |
| `results/template.html`, `results/examples/` | the result document (§13.5): one self-contained page per finished run or campaign, reopened without re-running; two demonstration results from a deliberately simple model | every user, the engine, the MATLAB tool |
| `matlab_sils/` | the MATLAB SILS twin's README and package contents; its code is written in lockstep with the platform, element by element, from P1 (§10.8.7) | developers, team members |
| `plan/twin_map.toml` | the twin map, `adcs-twin-map/1`: every SILS element (99 today, from four registries and twenty single elements), its platform side, its MATLAB side, the phase that builds both, the rungs that reuse it and the test both pass (§10.8.7) | `twin_check.py`, `pack_matlab.py`, CI |
| `tools/build_tree.py` | builds `plan/tree.json`, `plan/case_inputs.toml` and `plan/case_template.csv` from readable Python (`--check` compares without writing); edit this, not the generated files | the developer team |
| `tools/plan_model.py` | the plan as one read-only object, shared by the tools below | the tools |
| `tools/forms.py` | writes node forms, seed forms, the new-node request, the case editor and the node library from the plan | the developer team, CI, `pack_matlab.py` |
| `tools/intake.py` | the intake checker, the sheet writer and the verifier (§5.11), with 54 checks (the belief record's D01–D07 and the explanation's X01–X03 among them) and a selftest of 66 deliberate mistakes; the stand-in for `cargo xtask intake` | the developer team, CI |
| `tools/derisk.py` | the ledger (§5.13): check (L01–L09), rollup into the risk rows, the quarterly narrative as HTML, `.xlsx` and `.csv`, §21's table; the stand-in for `cargo xtask derisk` | the developer team, the quality team, CI |
| `tools/explain_check.py`, `tools/explain_kit.py` | the explanation standard (§5.12): the check over every rendered template example and manual page, and the one copy of the kit every template carries | the developer team, CI |
| `tools/make_examples.py` | writes every example in `forms/examples/` and `results/examples/` from the current templates | the developer team |
| `tools/manual_pages.py` | writes the manuals' generated parts (the case keys page, the intake codes table, the ledger rules table, the twin rules tables) and checks them; the stand-in for `cargo xtask manual` | the developer team, CI |
| `tools/check_case.py` | the case checker (§8.3.4): the format, what the case says, and which scenarios it can run | team members, CI |
| `tools/results.py` | writes the demonstration results, reads any result, imports results into a local store beside their cases, rebuilds a store's index, exports a run's channels | people, CI |
| `tools/validate_plan.py` | checks the whole package for internal consistency; `--selftest` applies 28 deliberate breakages to a copy and requires each to be caught; it checks only the directories copied so far | CI, the developer team |
| `tools/form_browser_check.py` | opens the node form, the case editor, the library and the results in headless Chromium and proves each loop (§5.10.8) | CI |
| `tools/pack_matlab.py` | builds the twin's download, `adcs_sils_matlab_<version>.zip`, deterministically, from the repository, with the twin map and `TWIN.md`; refuses a zip missing a twin the phase builds | CI, the developer team |
| `tools/twin_check.py` | the lockstep check (§10.8.7): the map is whole (TW01–TW04), a checkout has both sides of every element its phase builds (TW05), a pull request changes both sides or says why not (TW06); the stand-in for `cargo xtask twin` | CI, the developer team |
| `spec/`, `tools/assemble_spec.sh` | the sections this document is assembled from, and the script that joins them (`--check` compares) | whoever edits the package |

Run `python3 tools/validate_plan.py` after any edit to the package. It must print `0 finding(s)`.

The package lives in the new repository at `_package/`, read-only. Each phase copies from it into the repository's own paths (`plan/`, `derisk/`, `catalogue/`, `designs/`, `devices/`, `scenarios/`, `campaigns/`, `rig/`, `fsw/include/`, `forms/`, `results/`, `manual/`, `matlab_sils/`, `tools/`), so the package's own README never collides with the repository's.

### 0.2 Rules for the builder

The builder is Claude Code, working for the developer team. These rules hold in every phase, and they are the same rules the implementation agent works under after release (§5.11).

1. **Build what §3.4 lists, named as §3.3 says.** A file this document does not call for is not written. What §3.4 says is never present is absent completely, not left dormant.
2. **Never write a person's name.** A node's `confirmed_by`, a panel's `confirmed_by`, a promotion's `by` and a certificate's signatory are a person's attestation. A node's `confirmed_by` is written only by `cargo xtask intake write`, from the `attested_by` a person typed into a node form; every other person's decision only by `cargo xtask decision record`, from that person's own record (§17.2). Otherwise it is `UNCONFIRMED · <what it is> · awaiting a person` (§5.8).
3. **Never produce an expected value.** A test vector's `expect` is transcribed from a page of a cited source by a person, in a node form, or it is not written. `plan/seed_content.toml` holds seven such vectors. This includes parity references: IDMAS v2 §13's numbers are a second opinion, never something to tune toward.
4. **Never widen a tolerance, skip a test, or edit a generated file outside a HOLE.**
5. **A refusal is never a substitution.** A part with a `nan` field, a scenario that needs it, a row with no theory, a verified closure with no campaign behind it: each is refused by name. That is a correct result, and the phases in §19 expect several of them.
6. **Node content arrives only through intake.** No sheet is written by hand, by a script, or by an agent except through `cargo xtask intake write` from a request that passed `cargo xtask intake check`, followed by `intake verify`. That includes the first 82 rows, which arrive as seed forms (§5.8). Two tools write a sheet's *shape*, never its content: the seeder (every row's identity, place and edges, and the closures and requirement rows that `plan/kpis.toml` fixes, §5.5, §5.7) and `cargo xtask new`, run only as the first step of a new-node request.
7. **Stop at the decisions in §20.** They belong to a person. Do the preparatory work, write what is needed to decide, and stop.
8. **When this document and the built code disagree:** the code wins on mechanism (how a sheet is loaded, how the resolver orders nodes), and this document wins on ADCS content (which rows exist, what a scenario means) and on the operating model (§1.6). Record every such disagreement in `docs/SPEC_DEVIATIONS.md` with the file, the line and the choice made.
9. **Say what you did not do.** Every phase report lists what is not built yet, what is mocked, and what needs hardware or a person.

### 0.3 Reading order

Read §1 to §4 before touching anything; §1.6 is the operating model everything else serves. §5 to §18 are the reference for each part, with §5.10 to §5.13 (the node form, intake, the explanation standard and de-risking) and §16.4 (the manuals) read by everyone. §19 is the build, one phase at a time, each with its commands and its acceptance test. §20 is what the builder must not decide.
