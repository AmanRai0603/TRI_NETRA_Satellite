# ADCS Platform — Build Specification

| | |
|---|---|
| Document | ADCS-SPEC-01 · the build specification for the ADCS platform repository |
| Version | 2.0 · 27 September 2026 |
| Owner | Aman Kumar Rai |
| Built on | VLEO_SIMULATOR at commit `abf79ee` (https://github.com/AmanRai0603/VLEO_SIMULATOR) |
| Product content from | IDMAS v2 Exploration (IDMAS V1 coils, V2 fluid rings, V3 RCS) |
| Read by | Claude Code, which builds the repository from it; the developer team, who maintain it |

This document, and the files beside it, are everything needed to build one repository: the ADCS company's platform.

- **Inside the company**, the platform takes a customer's case, one CSV in a fixed format, to a catalogue product tuned and proven in SILS for that case. When the catalogue has none, the design team's runs of the same engine design one.
- **As a test facility**, it tests that ADCS in SILS, PIL, OILS and HILS on one scenario template, and issues the certificate.
- **Towards the client**, it offers SILS with visualisation before the order. After the purchase order, it gives OILS and HILS results with visualisation, plus the certification.

The repository is VLEO_SIMULATOR's machinery with an ADCS tree in it. It adds seven things VLEO_SIMULATOR does not have:

1. case import from a fixed CSV;
2. a solver and designer over a catalogue of products;
3. a time-stepping loop engine;
4. a real-time rig;
5. a portal;
6. a plain-MATLAB twin of the SILS engine;
7. an intake that turns a team member's node form into checked, implemented and released software.

Where VLEO_SIMULATOR already solved a problem, this document says "port it" and names the file. Where it did not, this document specifies the new part.

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
| `plan/tree.json` | the ADCS tree in VLEO's seven-key `cd06/tree.json` shape: 418 rows, 307 edges, one door | the seeder (§5.7) |
| `plan/case_template.csv`, `plan/case_inputs.toml` | the fixed case format `adcs-case/1`: the blank template (54 inputs, 5 meta rows, each explained in its note) and its registry, which also records the supplier of every declared layer-2 row; both generated | the case importer (§8.3), the case editor, the case checker |
| `plan/cases/*.csv` | four reference cases in that format: the two default 3U cases, `ais_3u` (10° pointing) and `ais_img_3u` (0.01°), plus a 150 kg bus and an unstated 12U | `adcs case import`, the pilot |
| `plan/expected_node_ids.json` | the node id VLEO's seeding code gives each tree row, recorded by running it | cross-checks in P1 |
| `plan/seed_content.toml` | the pilot thread's 61 rows and the risk branch's 21, written from cited sources (26 in all, the OILS and HILS facility references among them), with 7 test vectors transcribed from them and the spin-down node's own explanation (the standard's worked example). It is the content of the 82 seed forms (§5.8). | `tools/forms.py seeds`, then intake |
| `plan/kpis.toml` | the 22 KPIs: requirement, evidence row, analysis row, metric, usual sense, closure slug | the seeder's closures, the campaign runner, `validate_plan.py` |
| `derisk/risks.toml`, `derisk/beliefs/*.toml` | the risk register (18 risks, from §21, levels proposed until D26) and 16 belief records for the decisions this package took, most honestly untested (§5.13) | `tools/derisk.py`, the Risk management rows, intake |
| `derisk/narrative_template.xlsx` | the company's quarterly de-risking narrative template, its seven columns | the narrative (§5.13) |
| `plan/units.toml`, `plan/physics.toml` | the units and quantities a node may declare (VLEO's registry plus §6.1), and the functions of `adcs-core::physics` (§6.2) | the node form's pick-lists, the intake checker |
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
| `tools/check_seed_with_vleo.py` | runs VLEO_SIMULATOR's own seeding code over `plan/tree.json` and checks every node id against `plan/expected_node_ids.json` | CI, the developer team |
| `spec/`, `tools/assemble_spec.sh` | the sections this document is assembled from, and the script that joins them (`--check` compares) | whoever edits the package |

Run `python3 tools/validate_plan.py` after any edit to the package. It must print `0 finding(s)`.

The package lives in the new repository at `_package/`, read-only. Each phase copies from it into the repository's own paths (`plan/`, `derisk/`, `catalogue/`, `designs/`, `devices/`, `scenarios/`, `campaigns/`, `rig/`, `fsw/include/`, `forms/`, `results/`, `manual/`, `matlab_sils/`, `tools/`), so the package's own README never collides with the repository's.

### 0.2 Rules for the builder

The builder is Claude Code, working for the developer team. These rules hold in every phase, and they are the same rules the implementation agent works under after release (§5.11).

1. **Port, do not rewrite.** Copy VLEO_SIMULATOR files verbatim and change only what §3.3 lists: the `vleo` → `adcs` names and the domain nouns. Keep each file's template, ordering and comments. A file this document does not mention is copied unchanged or not at all, as §3.4 says. What §3.4 removes is removed completely, not left dormant.
2. **Never write a person's name.** A node's `confirmed_by`, a panel's `confirmed_by`, a promotion's `by` and a certificate's signatory are a person's attestation. A node's `confirmed_by` is written only by `cargo xtask intake write`, from the `attested_by` a person typed into a node form; every other person's decision only by `cargo xtask decision record`, from that person's own record (§17.2). Otherwise it is `UNCONFIRMED · <what it is> · awaiting a person` (§5.8). VLEO's `seed_tree.py` writes "A. Rai / 2026-09-01" into KPI rows; the port must not carry that string.
3. **Never produce an expected value.** A test vector's `expect` is transcribed from a page of a cited source by a person, in a node form, or it is not written. `plan/seed_content.toml` holds seven such vectors. This includes parity references: IDMAS v2 §13's numbers are a second opinion, never something to tune toward.
4. **Never widen a tolerance, skip a test, or edit a generated file outside a HOLE.** Unchanged from VLEO.
5. **A refusal is never a substitution.** A part with a `nan` field, a scenario that needs it, a row with no theory, a verified closure with no campaign behind it: each is refused by name. That is a correct result, and the phases in §19 expect several of them.
6. **Node content arrives only through intake.** No sheet is written by hand, by a script, or by an agent except through `cargo xtask intake write` from a request that passed `cargo xtask intake check`, followed by `intake verify`. That includes the first 82 rows, which arrive as seed forms (§5.8). Two tools write a sheet's *shape*, never its content: the seeder (every row's identity, place and edges, and the closures and requirement rows that `plan/kpis.toml` fixes, §5.5, §5.7) and `cargo xtask new`, run only as the first step of a new-node request.
7. **Stop at the decisions in §20.** They belong to a person. Do the preparatory work, write what is needed to decide, and stop.
8. **When this document and VLEO_SIMULATOR's code disagree:** the code wins on mechanism (how a sheet is loaded, how the resolver orders nodes), and this document wins on ADCS content (which rows exist, what a scenario means) and on the operating model (§1.6). Record every such disagreement in `docs/SPEC_DEVIATIONS.md` with the file, the line and the choice made.
9. **Say what you did not do.** Every phase report lists what is not built yet, what is mocked, and what needs hardware or a person.

### 0.3 Reading order

Read §1 to §4 before touching anything; §1.6 is the operating model everything else serves. §5 to §18 are the reference for each part, with §5.10 to §5.13 (the node form, intake, the explanation standard and de-risking) and §16.4 (the manuals) read by everyone. §19 is the build, one phase at a time, each with its commands and its acceptance test. §20 is what the builder must not decide.

---

## 1. What is being built

### 1.1 The product

The company sells attitude determination and control systems built from a fixed catalogue. It also runs the facility that proves each one: SILS, PIL, OILS and HILS.

A customer states what their satellite must do in **one CSV file in a fixed format**: pointing accuracy, knowledge, stability, slew time, detumble time, faults to survive, the mass and power they can spare, and the satellite and orbit behind them. That file is **the case**. The platform imports it, puts every value where it is needed, and turns it into three things:

1. a shortlist of catalogue **products** that meet every written requirement, each tuned in SILS for this case, with its margins. When no product does, the answer says so, with the gap, and the case becomes a design request for the design team (§8);
2. SILS evidence for each shortlisted product, with visualisation, run on the scenarios that bind the case's requirements;
3. after the purchase order, a built and calibrated unit tested in OILS and HILS on those same scenarios, with visualisation. It ships with a verification matrix, a parity ledger and a certificate of conformance.

A product is a configuration of catalogue parts plus the flight algorithms it carries. The design team finds new products by running the same engine in **designer mode**: every part combination a family allows is swept against a satellite class's standard case, and every combination that passes is saved as a candidate. So the catalogue grows with the company's design work, and no customer is ever a branch of the tree or a line of code.

One platform serves the company's own teams and its clients. The client sees a restricted view of the same engine, never a second implementation.

### 1.2 Who uses it

Two groups meet the platform, and they meet it differently (§1.6).

| Who | Uses | Gives the platform |
|---|---|---|
| **Developer team** | the repository: the tree, the engine, the tools, the intake, CI and releases | code, sheets (only through intake), catalogue files, scenarios, releases, and a reply to every request |
| **Team members**: physics, GNC and ADCS engineers, the design team, flight software, test facility operators, production, sales, quality, management, trainees and students | the released software: the web app and workbench, the MATLAB SILS tool, the portal's internal pages, the node library | case CSVs; node forms (requests and feedback); their own saved results. Some roles also make a person's decision in the software: a promotion (H14), a price (H-quote), a certificate signature (H-cert), a parity cause (H15). |
| **Client engineers** | the portal: their project, the solver, SILS, the witness view | their case CSV, their choice of product and campaign, their own model as an FMU |

What each team member mainly does with the released software:

| Team | Uses it for | Asks the developer team for, by node form |
|---|---|---|
| Physics and GNC engineers | runs, campaigns, the tree's documents and chains | a relation, a bound, a test vector, a new node, a confirmation |
| Design team | the designer (§8.6), class standards, candidate products | a new part, algorithm or product in the catalogue ("something else") |
| Flight software engineers | SILS against their own flight build | an algorithm's parameters, a telemetry change ("something else") |
| Test facility operators | the rig, campaigns, the facility view | a scenario or a campaign ("something else") |
| Production | calibration records: each unit's measured descriptor, uploaded on the order in the portal (§7.4) | a new descriptor field ("something else") |
| Sales | the solver's shortlists, quotes | — |
| Quality | the verification matrix, parity causes, certificates | a confirmation |
| Management | layer 1 of the tree | — |

### 1.3 What the client sees, and when

| Stage | The client sees | The client can do | Stays in-house |
|---|---|---|---|
| Enquiry | the portal, the four families described, the blank case template and the case editor | create a project, upload the case CSV (or fill the case editor on screen), read the case report | everything below layer 2 |
| Sizing | the shortlist of offered products with margins after tuning, refusals named in plain words; or "no catalogue product meets this case" with the gap | pick a product to test, or request a design | L3 internals, restricted rows (D1), tuned values, candidate products |
| SILS | live run view and campaign dashboards (§13) for their case, within a run quota; every finished run as a result document | switch between shortlisted products; run the release's nominal, edge, Monte Carlo, sweep and fault campaigns; upload a model as an FMU; download and re-open results | flight software source, plant internals, controller gains |
| Quote | the requirement package (§14.1): ADCS spec, ICD, verification matrix; the quote, frozen by hash | accept, or change the case and re-quote | cost, margin, price table |
| Purchase order | the order record and its stages | upload the PO | — |
| OILS, HILS | the witness view: the same visualisation, streamed live from the rig, then replayable, and saved as result documents | watch, comment, request a re-run through sales | the rig console, fault-injection controls, facility telemetry |
| Delivery | the evidence package and the certificate of conformance | download | the run ledger across all clients |

OILS and HILS results reach the client only after the purchase order, and always with visualisation. The live witness view is the product feature, not a courtesy.

### 1.4 What the repository contains, in twelve parts

1. **The engine**, ported from VLEO_SIMULATOR: units, kernel, sheet generators, gate, bus, data bundles, faces (§3), with every in-software editing path removed (§3.4).
2. **The ADCS tree**: four layers, 608 node sheets once seeded, 22 KPIs each closed by evidence and 16 of them also by analysis (§5). Every node's content arrives through intake.
3. **The node form and intake** (§5.10, §5.11): the one way the software's content changes. A team member fills a form. The developer team runs the checker, the implementation agent writes the change, the checker verifies it, a person reviews, and the release carries it.
4. **Units and physics** for attitude work (§6).
5. **The catalogue**: the module descriptor standard, parts, families, products, algorithms and satellite classes; per-serial descriptors are recorded against each order as operational data, not in the repository (§7).
6. **Cases, the solver and the designer**: the fixed case CSV, its editor, checker and importer; screening, SILS tuning and selection for a case; design sweeps that save candidate products (§8).
7. **The loop engine**: a deterministic, time-stepping SILS with the flight C code linked in and device emulators on every port (§9, §10).
8. **The loop contract and the rig**: one message set; PIL, OILS and HILS on a real-time host with an interface emulation unit and an environment simulator (§11, §12).
9. **Visualisation and result documents**: one set of views for every rung, live and replayed, and every finished run saved as one file that re-opens without re-running (§13).
10. **Evidence and certification**: verification matrix, parity ledger, reports, certificate (§14).
11. **The portal**: multi-tenant, client and internal roles, run queue, quote to purchase order (§15).
12. **The MATLAB SILS twin**: the SILS engine again in plain MATLAB, released as one zip with the case editor, the node library, the result viewer and the user manual (§10.8). It is built in lockstep with the platform: every SILS element is written in both engines in the same change, from P1, so the zip is whole at every push (§10.8.7).

The flight software itself is the company's own product and lives behind the two headers. This repository ships a **reference flight software** in `fsw/` so that SILS runs out of the box: B-dot, sun acquisition, a MEKF, the IDMAS split-projection law and allocation. The company's flight build replaces it without any change to the platform.

### 1.5 Not in scope

- A flight operating system or the OBC's board support package. The repository defines `adcs_hal.h`; each OBC implements it.
- Replacing MATLAB for design. MATLAB stays the design tool, and the MATLAB SILS twin (§10.8) is where developers try new algorithms first. The loop engine is the platform's SIL, checked against MATLAB (§10.7).
- Editing the software from inside the software. There is no edit mode, no write route and no in-app sheet editor (§3.4). A change is a node form.
- Hardware drivers proven on hardware. Every rig device has a mock backend and a driver written to its interface document. Bring-up on the real device is a checklist a person runs (§12.9).
- Building a GNSS RF signal simulator. HILS buys one. OILS emulates the receiver's output at protocol level.
- Code signing, until decision D3 (§20).

### 1.6 Who does what

This is the operating model. Every other part of this document serves it.

**The developer team owns the software.** They build it, maintain it and upgrade it over time. Only they change the repository, and they change it in one of two ways: their own engineering work (the engine, the tools, the views, the catalogue, scenarios), or a request from a team member, taken through intake. The released software never changes itself. It has no edit mode and no write route.

**Everyone else uses it.** A team member gets exactly two things.

**1. The released software, as a user.** It comes as the web app and workbench, the portal's internal pages, and the MATLAB SILS tool: one zip, rebuilt on every push and attached to every release, always holding exactly the SILS the platform has, because each element is written in both engines at once (§10.8.7). With it, a team member:

- writes their **input** as a **case CSV**, in the case editor or any spreadsheet. The case is the only thing they edit (§8.3);
- checks the case: the software reports what it states, what it leaves blank, and which scenarios it can run;
- runs SILS on it: single runs, Monte Carlo, edge cases, sweeps, faults, the solver and the designer;
- sees the result in the views of §13, reads any node's document in the node library, and compares runs;
- has every finished run saved as a **result document** in their local store, filed beside the case it was run on (§13.5). They reopen it later, share it, or open one a colleague sent, and see everything again without running anything.

Visualisation, documents, interaction and saved results are features of the software, not things anyone edits.

**2. The node form, to ask for anything.** A team member who wants a node changed, a new node, a node confirmed, or who has feedback on a release, fills that node's form (§5.10) and sends the saved file to the developer team. Something that is not a node, such as a part, an algorithm, a scenario or the case format, goes in the same form as "something else". The form is written for them: every field says what it asks, why it matters and gives an example. It carries no code. It carries the relation, the connections, the bounds, the test vectors and the sources, in enough detail that the code can be written from it.

**The loop between them.**

```
TEAM MEMBER                                        DEVELOPER TEAM
───────────                                        ──────────────
case CSV ──▶ released software ──▶ result documents (local store, beside their case; re-open, compare, share)
                                        │
node form: change · new · confirm ·     │ names the case and result
feedback · something else ◀─────────────┘
     │ saved file
     ▼
                                          1. check     cargo xtask intake check <form>
                                             fail ──▶ reply in the form's history ──▶ back to the team member
                                          2. implement the implementation agent, from the checked request's brief:
                                                       intake write (sheet, test vectors, version, belief record),
                                                       code in the HOLEs,
                                                       new physics functions
                                          3. verify    cargo xtask intake verify: the node says what the form says
                                          4. review    a second developer (the node's owner group); gate, tests, downstream nodes
                                          5. release   CLI, workbench, portal, MATLAB zip, node library, manuals
     ▲                                                 │
     └───────── the form comes back: "released in <version>" ◀─┘
     then: run your own case in that version, save the result, and send feedback in the same form
```

**Every page explains itself, and every change says why.** Nobody sits beside a team member to explain a form, a result or a case: each page does it itself, to one standard (§5.12). And every request that changes a node carries the belief it rests on, what tested it, and the risks it opens or closes (§5.13). The developer team's own changes record theirs too. So each node's version history says why every version exists, and the Risk management branch of layer 1 concludes, at every release, how far the platform as a whole has been de-risked.

**What this rules out.**

- There is no in-software editing path: no workbench write route, no draft-and-confirm in the app, no form that writes a sheet by itself (§3.4).
- There is no fleet of specialised agents drafting nodes. Content comes from people, through forms. Code is written by one general implementation agent, run by a developer, under a brief the checker writes and a verify step that compares the result with the form (§5.11, §17.4).
- There is no second copy of an input. A scenario never restates a case value; a case is never typed into a scenario.

Clients outside the company use the portal (§15). It is the same software behind their tenant, with the same case CSV and the same result documents. Clients have no node form: what they want changed reaches the company through sales.

---

## 2. Product configurations

The product is fixed; the configuration is chosen. There are four families, defined as data in `catalogue/families.toml`:

| Family id | What it is | IDMAS | Offered for | Counts it sets |
|---|---|---|---|---|
| `mtq` | Magnetorquers only | V1 | 0.5–30 kg | coils 3–6; wheels, rings, thrusters 0 |
| `mtq_rw` | Magnetorquers + reaction wheels | — | 1–500 kg | coils 3–6, wheels 3–4 |
| `mtq_fmr` | Magnetorquers + fluid momentum rings | V2 | 1–50 kg | coils 3–6, rings 3–4 |
| `mtq_fmr_rcs` | Magnetorquers + fluid rings + RCS | V3 | 50–500 kg | coils 3–6, rings 3–4, thrusters 4–16 |

Four rules follow from treating families as data.

1. **Absence is a count of zero, never a missing row.** Every family runs on the same tree. A satellite with no wheels has `Reaction wheels fitted = 0`, and every wheel row answers zero cleanly: zero momentum, zero power, zero mass. So one closure run answers for every family, and comparing families is comparing candidates.
2. **The family comes from the product, not the case.** A case states the satellite and the requirements. The product (or, in designer mode, the part combination) supplies every count: the filled slots' counts, and zero for the rest (§8.4). A case may narrow the search with `meta.families`, but it never has to name one.
3. **The tree says which rows each family puts in play.** The fifth field of every tree row carries its families (§5.6). The face hides out-of-play groups from a client, and the solver uses the same field to skip whole branches.
4. **Adding a family is a catalogue change, not code.** Magnetorquers + wheels + RCS would be a new `[[family]]`, with its slots and the algorithms it may carry. The solver never names a family in code.

The coils are in every family because they are the safety floor. They detumble, they dump, and they own safe mode. IDMAS v2 §01 is explicit that detumble, safe mode and momentum dumping never depend on the newer hardware. The solver refuses a configuration without coils by name, and every product carries the `bdot` algorithm.

What the case's requirements decide, through the solver:

- the product: in client mode among the offered products, in designer mode the family, parts and counts (by agility, pointing, mass class and fault tolerance: IDMAS v2 §15 "What it can fly");
- the parts within the family (by the closures);
- the counts within the family's range (by fault tolerance: IDMAS v2 §05.2 baseline, fault-tolerant and large-bus layouts);
- the sensor suite (by the knowledge requirements);
- the tuned algorithm parameters (by SILS against the case, §8.5).

One gap is stated rather than hidden: step-and-stare agility on large buses (IDMAS v2 §09.1, "Rings + RCS cover every agility class: No — the gap"). The solver returns it as a named refusal for `mtq_fmr_rcs` and screens `mtq_rw` for comparison.

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

---

## 4. Repository layout

Suggested repository name: `ADCS_PLATFORM`. The name appears in `Cargo.toml [workspace.package] repository`, `.devcontainer` and the README, and changes nowhere else.

```
ADCS_PLATFORM/
  AGENTS.md  CONTRIBUTING.md  README.md  CODEOWNERS(generated)  ADOPTION.lock
  Cargo.toml  Cargo.lock  rust-toolchain.toml  rustfmt.toml  .gitignore
  .cargo/config.toml               aliases: xtask, adcs
  .claude/hooks/  .claude/settings.json     the scope hook the implementation agent runs under (§17.4)
  .github/workflows/{gate,nightly,release}.yml  dependabot.yml  pull_request_template.md
  .github/ISSUE_TEMPLATE/request.yml        "attach a node form" — where a request can arrive
  .devcontainer/{devcontainer.json,setup.sh,start.sh}
  areas/{generators,faces,document,data,numerics,release,simulation,rig,portal,evidence,solver,intake}.md

  plan/                            this package's plan/: tree.json, seed_content.toml, kpis.toml, units.toml,
                                   physics.toml, case_inputs.toml, case_template.csv, cases/*.csv, expected_node_ids.json,
                                   twin_map.toml (every SILS element's platform and MATLAB sides, §10.8.7)
  layers/                          generated once by the seeder; the tree's shape, changed only by a reviewed
                                   developer change (a new group is "something else", §5.10)
  cases/                           the reference cases, imported: `adcs case import plan/cases/*.csv`
  sources/sources.toml             generated once from plan/seed_content.toml [[source]]; grows through intake
  bundles/{igrf14,atmos-density,catalogue}/<version>/{manifest.toml,...}
  catalogue/{schema.toml,families.toml,classes.toml}     the source the catalogue bundle is published from
  catalogue/{parts,products,algorithms}/*.toml  catalogue/classes/*.csv
  designs/*.toml                   design jobs for `adcs design` (§8.6)
  devices/<part>.toml              each part's port protocol: registers, framing, faults (§9.5)
  scenarios/*.toml                 the one template (§10)
  campaigns/*.toml                 campaign definitions
  rig/{device_maps,labs}/*.toml    the rig's wiring, and the lab as a plant and as measured facility capabilities (§12.10)
  fsw/                             reference flight software, C99 (§9.10)
    include/{adcs_hal.h,adcs_fsw.h}
    src/  tm/  tc/  tests/  CMakeLists.txt
  matlab_sils/                     the MATLAB SILS twin (§10.8): +asils/, examples/, tests/; written in lockstep with
                                   crates/ and fsw/, element by element (§10.8.7)
  forms/                           node_form.html, case_editor.html, library.html (§5.10, §8.3.3); examples/
  results/                         template.html, the result document (§13.5); examples/
  manual/user/  manual/developer/  the two manuals (§16.4), shipped and checked
  derisk/                          the de-risking ledger (§5.13): risks.toml, beliefs/<id>.toml, narratives/<quarter>.md
                                   (a person's prose), narrative_template.xlsx, rollup.toml (written at release)
  intake/                          AGENT.md: the implementation agent's standing instructions (§17.4);
                                   requests/<request id>/ — never committed: the working area of `xtask intake`
  crates/adcs-mod-*/nodes/<node>/requests/<request id>.request.html   every request that changed the node
  crates/adcs-mod-*/nodes/<node>/versions.toml, versions/<n>/          every version of the node, and why (§5.13)
  deploy/compose.yaml              the portal stack (§15.9)
  docs/phases/P<n>.md              one report per build phase (§19)
  docs/  panels/  tools/  web/  matlab/  xtask/
  dist/                            built downloads, never committed: adcs_sils_matlab_<version>.zip, the node library

  crates/
    adcs-units       RING 0  quantities, units, frames, pmath          (ported; §6.1 adds)
    adcs-core        RING 1  resolver, graph, credibility, physics     (ported; §6.2 rewrites physics)
    adcs-sheet               the sheet: loader, generators, gate       (ported; F1, F2, F7; write paths removed, F17)
    adcs-bus         RING 2  wire types + the loop contract (§11)      (ported; §11 adds `loop`)
    adcs-data                bundles, store, lockfile                  (ported; F8)
    adcs-mod-*       RING 3  node crates, one per owner layer (§5.4)   (seeded; content by intake)
    adcs-modules             the facade: generated tables              (ported)
    adcs-intake              the node form: read, check, write the sheet, verify, reply (§5.11)  NEW
    adcs-derisk              the ledger: check, record, rollup, narrative (§5.13)               NEW
    adcs-catalogue           reads parts, families, products, algorithms and classes, from the working copy or a bundle (§7)  NEW
    adcs-case                the case format: CSV check, import and export, the report, the case store (§8.3)  NEW
    adcs-config              screening: supply map, closures over candidates (§8.4)                NEW
    adcs-tune                SILS tuning of a candidate's algorithms (§8.5)                        NEW
    adcs-solve               client mode and designer mode; saves candidates to the store (§8.6)   NEW
    adcs-sim-core            the loop engine's pure core, no_std (§9)  NEW
    adcs-sim                 scheduler, device emulators, campaigns, recorder, metrics (§9, §10)  NEW
    adcs-result              result documents and the local store: write, read, import, index (§13.5)  NEW
    adcs-fsw-abi             implements adcs_hal.h for SILS; links the flight C code (§9.6)       NEW
    adcs-rig                 real-time runner, transports, environment-simulator drivers (§12)   NEW
    adcs-evidence            verification matrix, parity ledger, reports, certificate (§14)       NEW
    adcs-cli  adcs-daemon  adcs-ffi  adcs-wasm  adcs-py              FACES (ported; read and run only; §16 adds)
    adcs-portal              multi-tenant HTTP service (§15)           NEW
    adcs-worker              run-queue executor (§15.4)                NEW
```

The ring rule extends to the new crates:

```
adcs-units -> adcs-core -> adcs-bus -> adcs-mod-* -> adcs-modules -> faces
                  |           |
                  +-> adcs-sim-core (no_std: reads adcs-core::physics, adcs-units)
                              |
                  adcs-catalogue -> adcs-units;  adcs-case -> adcs-bus, adcs-modules (node ids and sheet values)
                  adcs-intake -> adcs-sheet, adcs-units, adcs-modules (read), adcs-derisk; used by xtask only, never by a face
                  adcs-derisk -> adcs-units; used by xtask; faces read its rollup.toml and narrative only
                  adcs-config -> adcs-case, adcs-catalogue, adcs-modules
                  adcs-sim (std) -> adcs-fsw-abi, adcs-case, adcs-catalogue;  adcs-tune -> adcs-sim
                  adcs-result -> adcs-case, adcs-sim (types only)
                  adcs-solve -> adcs-config, adcs-tune, adcs-sim;  adcs-evidence -> adcs-sim
                  adcs-rig (std) -> adcs-sim, adcs-bus::loop
                  adcs-portal, adcs-worker -> faces' library APIs only
```

- `adcs-sim-core` may not depend on anything above `adcs-core`, so the plant can never read the tree's generated tables. Parameters reach it as a resolved case and as descriptors.
- `adcs-intake` is a developer tool. No face, no worker and no portal route links it, so the released software has no path by which a request changes it.
- `adcs-portal` never links the engine directly. It enqueues work, and `adcs-worker` runs it, so a portal crash cannot corrupt a run and a run cannot hang the portal.
- `xtask graph`'s ring check is extended to the new crates, and a manifest line that breaks the direction fails the gate.

---

## 5. The ADCS tree

### 5.1 Four layers, one door between each

| Layer | Holds | Rows | Crate |
|---|---|---|---|
| 1 company | case intake, the catalogue, commercial, the order lifecycle, the facility, standards, supply, heritage, risk management | 133 leaves in 42 groups | `adcs-mod-management` |
| 2 the satellite's ADCS | the pointing service asked for, the fitted hardware, mission and orbit, the satellite as the ADCS sees it, the ADCS subsystems at system level, verification | 194 leaves in 49 groups | `adcs-mod-system` |
| 3 subsystem | 14 subsystem layers (12 for the ADCS, 2 for its test rigs), plus the closure addition | 368 rows, plus 38 closures and the closure layer's interface row | `adcs-mod-<sid>`, `adcs-mod-closure` |
| 4 the run | what one evaluation or one campaign produced | — | — |

Every case reaches layer 2 through one row, `Satellite ADCS` under Case intake; the note "the door into this case's engineering layer" makes it cross to `sys_satellite_adcs`. There is exactly one door, and `tools/validate_plan.py` refuses a second. Each subsystem reaches layer 2 through its one `l3_<sid>_interface` row. This is the same shape as VLEO, and VLEO's seeding code accepts it. `tools/validate_plan.py` checks the shape. `tools/check_seed_with_vleo.py` runs VLEO's own `cd06_rows.install`, `LAYER3_SOURCE` and `install_edges` over `plan/tree.json`. That wires 46 relations, 238 derivation edges and 23 contribution edges, skips none, creates one crossing, and passes all 14 layer-3 target assertions. In all, 734 node sheets once seeded: 327 in layers 1 and 2, 368 in the subsystem layers, and 39 in the closure addition.

### 5.2 Layer 1 — the company

| Branch | Groups | What it holds |
|---|---|---|
| Case intake | Case | the case format version, the door, the catalogue product selected, cost & price, acceptance criteria |
| Catalogue | Products, Design runs, Case matching | products offered and awaiting promotion, classes defined and covered, class coverage; combinations evaluated and passed, design yield; cases solved and matched, catalogue hit rate |
| Commercial | Quote, Cost to deliver, Margin and terms | unit, OILS and HILS campaign prices; BOM, labour, facility day cost; margin, data-share discount |
| Order lifecycle | Stage durations, Configuration control | enquiry → shortlist → quote → PO → built → calibrated → OILS → HILS → certificate → delivery; changes, deviations, waivers |
| Test facility | SILS capacity, OILS rig, Magnetic field simulator, Facility use, Air-bearing platform, Optical and RF stimulators, Actuator test stands, Rig safety and power | cores and runs per hour; the OILS rig's hours, real-time step, measured latency and jitter, emulation channels and interface kinds, connector isolation, GNSS output emulation, the rig host to IEU link rate; the cage's range, uniformity, test volume, field and direction error, time constant, reference magnetometer; the bearing's residual torque, tilt, inertia, capacity, balance, drag and truth attitude; the Sun simulator's irradiance, collimation, non-uniformity and instability, the star stimulator's step, faintest magnitude, frame rate and latency, GNSS RF channels; dynamometer bandwidth and torque resolution, imbalance, dipole, ring-torque, thrust and impulse-bit resolution; interlocks, emergency stop, brownout range, supplies; utilisation |
| Standards & compliance | ECSS-E-ST-60-30C, -60-10C, -10-02C, -10-03C, -40C / Q-ST-80C, export control, quality | coverage, tailoring, verification close-out, software coverage, non-conformances |
| Supply chain | Long-lead items, Make or buy, Supplier qualification | wheel and star-tracker lead times, gallium alloy stock |
| Heritage | Flight record | units delivered, orbit-hours, fraction of parts with heritage |
| Risk management | Risk register, Beliefs and versions, Open risk by area, Conclusion | risks open, open at L4 or L5, closed, opened and closed this quarter; beliefs broken, held, untested and recorded, node versions released, the cost of the quarter's tests; the highest open level in each of seven areas; the conclusion: the highest open level, the net risks closed this quarter, the share of beliefs tested (§5.13) |

**Risk management concludes the tree.** Its 18 declared rows are counted from the de-risking ledger (`derisk/`) at every release, by the supplier `derisk`, and never set by a form (V01). Its three computed rows (`risk::highest_level`, `risk::net_closed`, `risk::share_tested`, §6.2) are the platform's conclusion about itself: how much of what it rests on has met evidence, and what is still open. Two relations join it to the rest: the catalogue records the beliefs its products rest on, and the open levels feed the quality verdict under Standards & compliance.

No customer is a branch of this tree. VLEO's tree carried its reference customers as layer-1 groups; that does not scale past a handful, so here every customer is a **case**. It is uploaded as one CSV in the fixed format (§8.3), imported into the case store, and runs through the single door (CD-06 §33, "one architecture + case id"). The reference cases are four such files in `plan/cases/`. The two defaults are one 3U satellite with two missions: `ais_3u` (AIS, 10° pointing) and `ais_img_3u` (AIS and imaging, 0.01° pointing) (§8.9). The other two are `ref_c2_150kg`, a 150 kg bus, and `ref_c3_12u`, a 12U not yet stated.

### 5.3 Layer 2 — the satellite's ADCS

| Branch | Groups (leaves) |
|---|---|
| Pointing service the customer needs | Pointing performance (6), Agility (5), Robustness and safety (6), Resource footprint (5), each as a required group and an achieved group; the required rows are the case CSV's `req.*` keys |
| ADCS configuration | Hardware fitted (6 counts) |
| Mission and orbit | Mission requirements (6), Orbit (8), Environment along the orbit (6) |
| Satellite as the ADCS sees it | Mass properties (6), Surfaces and offsets (6), Magnetic cleanliness (2), Flexible modes (2), Resources offered to the ADCS (5) |
| ADCS subsystems | Disturbance torques (6); Attitude sensors (7), Attitude estimation (6); Magnetic actuation (6), Reaction wheels (7), Fluid momentum rings (10), Reaction control thrusters (5); Control and allocation (6), Pointing error budget (6), Modes and FDIR (5); Flight software and OBC interfaces (5), ADCS unit budgets (4) |
| Verification | Verification coverage (5), Campaign evidence (4), OILS rig needs (5), HILS rig needs (16) |

The 22 KPIs are listed in `plan/kpis.toml`, which is the machine-readable source for this table and for the closures the seeder writes (§5.5). Each has the ECSS-E-ST-60-10C index where one applies, the sense a requirement on it is normally written in, the row that answers it by analysis, and the metric a campaign supplies its evidence with:

| KPI | ECSS index | Usual sense | Analysis row | Evidence metric |
|---|---|---|---|---|
| Absolute pointing error (APE) | APE | `<=` | `gp_5` APE budget total | `ape` |
| Absolute knowledge error (AKE) | AKE | `<=` | `ge_5` Knowledge error, sensors fitted | `ake` |
| Relative pointing error (RPE) | RPE | `<=` | `gp_4` Jitter contribution | `rpe` |
| Pointing drift error (PDE) | PDE | `<=` | `gp_3` Thermal distortion contribution | `pde` |
| Rate stability | — | `<=` | none — evidence only | `rate_stability` |
| Relative knowledge error (RKE) | RKE | `<=` | none — evidence only | `rke` |
| Reference slew time | — | `<=` | `gc_5` Reference slew time achievable | `slew_time` |
| Settling time after a slew | — | `<=` | `gc_2` Settling time | `settling_time` |
| Slews per orbit | — | `>=` | none — evidence only | `slews_per_orbit` |
| Target-tracking rate | — | `>=` | none — evidence only | `tracking_rate` |
| Maximum body rate | — | `>=` | none — evidence only | `max_rate` |
| Detumble time | — | `<=` | `gq_0` Detumble time estimate | `time_to_rate` |
| Sun-acquisition time in safe mode | — | `<=` | `gq_1` Safe-mode sun acquisition estimate | `sun_acquisition` |
| Faults tolerated | — | `>=` | `gq_2` Faults tolerated by the configuration | `faults_survived` |
| Recovery time after a single fault | — | `<=` | `gq_3` Fault recovery time estimate | `recovery_time` |
| Momentum saturation margin | — | `>=` | none — evidence only | `momentum_margin` |
| Momentum dump interval | — | `>=` | `gq_4` Momentum dump interval estimate | `dump_interval` |
| ADCS mass | — | `<=` | `gb_0` ADCS mass total | inspection (the unit weighed) |
| ADCS orbit-average power | — | `<=` | `gb_1` ADCS orbit-average power total | `power` |
| ADCS peak power | — | `<=` | `gb_2` ADCS peak power total | `power_peak` |
| ADCS volume | — | `<=` | `gb_3` ADCS volume total | inspection (the unit measured) |
| RCS propellant per year | — | `<=` | `gr_4` Propellant per year | `consumable_per_year` |

"Evidence only" is a real answer. Those KPIs have no honest closed-form relation at system level, so the tree does not pretend to one. Their analysis closure is never generated, and the evidence closure is how they close.

An analysis row is narrower than a contribution edge. `tree.json`'s KE edges say which variables feed a KPI, which is VLEO's coverage graph, and several evidence-only KPIs keep theirs: settling time feeds rate stability, for instance, without answering it. Only `plan/kpis.toml` says what answers a KPI.

**What each rung's rig must do, for this case.** The two groups `v3` OILS rig needs and `v4` HILS rig needs are computed from the case's own satellite, so a case says what testing its unit will demand before any rig time is booked. The OILS rows turn the flight loop into a real-time plant step, a port count, a latency allowance and a link rate. The HILS rows turn the orbit into a cage field range, accuracy and slew rate; the control authority and the mass into the bearing's allowed residual torque and balance offset (a residual torque is the weight on the bearing times its centre-of-mass offset [Schwartz, Peck and Hall 2003]); the sensors into the Sun simulator's irradiance and collimation and the star stimulator's error and frame rate; and each fitted actuator family into what its test stand must show. The family rows are tagged, so a product without wheels shows no wheel row. The facility's side, what the bay can actually do, is layer 1 (`fa2`, `fa3`, `fa5`–`fa8`). Layers never read across (C04), so the two meet in `adcs rig fit` (§12.10), which refuses a HILS or OILS campaign by name when the lab cannot meet a need. None of these rows is written yet: they arrive through node forms, like every other row.

### 5.4 Layer 3 — fourteen subsystem layers

| sid | Layer | Closes against | Targets | Rows | Owner | Crate |
|---|---|---|---|---|---|---|
| `dist` | Disturbance environment | `gd` Disturbance torques | 6 | 25 | environment | `adcs-mod-dist` |
| `sens` | Attitude sensors | `gs` | 7 | 29 | sensing | `adcs-mod-sens` |
| `est` | Attitude estimation | `ge` | 6 | 22 | gnc | `adcs-mod-est` |
| `mtq` | Magnetic actuation | `gm` | 6 | 26 | actuators | `adcs-mod-mtq` |
| `rw` | Reaction wheels | `gw` | 7 | 26 | actuators | `adcs-mod-rw` |
| `fmr` | Fluid momentum rings | `gf` | 10 | 36 | actuators | `adcs-mod-fmr` |
| `rcs` | Reaction control thrusters | `gr` | 5 | 22 | actuators | `adcs-mod-rcs` |
| `ctl` | Control and allocation | `gc` | 6 | 28 | gnc | `adcs-mod-ctl` |
| `pnt` | Pointing error budget | `gp` | 6 | 25 | gnc | `adcs-mod-pnt` |
| `modes` | Modes and FDIR | `gq` | 5 | 22 | gnc | `adcs-mod-modes` |
| `fsw` | Flight software and OBC interfaces | `gx` | 5 | 22 | avionics | `adcs-mod-fsw` |
| `budget` | ADCS unit budgets | `gb` | 4 | 17 | systems | `adcs-mod-budget` |
| `oils` | OILS rig | `v3` OILS rig needs | 5 | 20 | verification | `adcs-mod-oils` |
| `hils` | HILS rig | `v4` HILS rig needs | 16 | 48 | verification | `adcs-mod-hils` |
| `x_closure` | KPI closures (addition) | — | — | 39 | systems | `adcs-mod-closure` |

Each layer holds its interface row, one required and one achieved row per target (VLEO's seeder derives these), and `rows − 1 − 2·targets` internal rows labelled "to be named". The internal budget is 166 rows across the fourteen layers. The fluid-ring and control layers get the most, 15 each. IDMAS v2 §03–§07 already names the ring's internal relations: pump pressure per stage, conduction and induction pump laws, Reynolds number, turbulent loss, freeze and thaw. §12 names the control stack's: the split projection, allocation weights, dump gain and the mode laws L1–L6. The two rig layers hold the lab's own models, which the lab twin (§12.6) runs: the cage's coil and field-error model, the bearing's residual-torque and drag model, the stimulators' rendering and latency, and each test stand's measurement model.

### 5.5 Closures, and rows that only evidence can answer

Every KPI is closed by evidence, and the 16 with an analysis row are also closed by analysis: 38 closures.

- **The analysis closure**, `kpi_<slug>_analysis` in `l3_x_closure`, compares the requirement row with the KPI's analysis row (`plan/kpis.toml`). It runs whenever the tree does. `<slug>` is the `slug` field of `plan/kpis.toml`, which is VLEO's `slug()` of the KPI's label: `kpi_absolute_pointing_error_ape_analysis`, for example.
- **The evidence closure**, `kpi_<slug>_verified`, compares the requirement row with the achieved row (`p1a_0` and so on). An achieved KPI row is an **evidence row**. Only a campaign can give it a value.

Both closures use `mission::closure(req, ach, Sense::AtMost | Sense::AtLeast)` exactly as VLEO's KPI rows do. The requirement row is written as `kind = "declared"` with a top-level `sense` (VLEO's convention for written requirements; gate check 7d).

**Who writes the closures.** The seeder writes all 38 in full, and every requirement row's shape with them, from `plan/kpis.toml` and `plan/case_inputs.toml`, as VLEO's `KPI()` does (§0.2 rule 6). A closure's content is fixed by the KPI list; nothing in it is a person's statement, so it needs no form. The seeder writes each closure `specified`, and the gate's closure checks (7d, 7e) and a `cargo xtask intake mark verified --closures` at the end of seeding make it `verified`, so closures run from P1. It does not wait for a requirement value, because the value comes from each case, not the sheet.

- inputs `req` and `ach`, one step, and the hole `Ratio::new(mission::closure(req.get(), ach.get(), mission::Sense::AtMost).margin)`, with `AtMost` or `AtLeast` read from the requirement's `sense` (gate 7e);
- a fixed `[theory]` text stating that a closure compares achieved with required in the requirement's sense;
- bounds −100 to 1000 with VLEO's reasons.

A requirement row written this way has kind `declared`, the KPI's `sense`, the unit of its case key, and `lower = 0` ("a requirement is a magnitude"). Its upper bound is 180° for an angle and 1 for a fraction. Any other quantity (a time, a count, a rate, a mass, a power, a volume) gets no upper bound until an engineer sets one, with its reason, in a node form. It carries no reference value, except `p1k_0` and `p2k_0`, whose seed forms carry the reference 3U satellite's values (source `adcs_ref_c1`) for runs without a case. Either way a case decides: a case that leaves a requirement blank lists it in `unstated`, and the row answers `NotStated` for that case whatever its sheet holds. The gate accepts a declared requirement row with no value (an H7 change beside check 7d). The analysis closure exists only when `analysis` is not "none".

A closure, a door and an interface row are not offered for change in a node form: the checker refuses them (F03). Their node forms are read-only documents, and anything to say about them is feedback.

**Evidence rows are the one platform extension to the sheet schema.** They are an H7 change, so two reviewers.

```toml
# p1a_0, as written
id = "sys_kpi_achieved_pointing_performance_absolute_pointing_error_ape"
kind = "achieved"
# ... label, parent, owner, layer, order, state as usual; no [[input]] at all

[evidence]
metric = "ape"                   # a metric kind adcs-sim computes (§10.3)
rungs = ["sils", "oils", "hils"] # which rungs may supply it
```

- **Resolver.** An evidence row's value comes only from `Case.evidence` (below). With none, it faults `NotRun` with the reason "no campaign has supplied this row", and the evidence closure is blocked by name.
- **Bus.** `Case` gains `evidence: Vec<EvidenceSupply>`, beside `supply` and never merged with it:
  `EvidenceSupply { id, value_si, rung: Rung(Sils|Pil|Oils|Hils), campaign_hash, runs, statistic, confidence }`.
  An ordinary `supply` to an evidence row is refused, and so is an `EvidenceSupply` to any other row.
- **Credibility.** For an evidence row the factors are scored from the campaign, not from the sheet:

  | Factor | Score |
  |---|---|
  | Validation | SILS 2, PIL 2, OILS 3, HILS 4 |
  | Verification | 4 when the campaign reproduces from its manifest (§10.6), else 0 |
  | Uncertainty | a Monte Carlo of at least 500 runs, at a stated probability, 4; 100 to 499 runs 3; 20 to 99 runs, or an edge set, 2; fewer than 20 runs, including a single nominal run, 1 |
  | InputPedigree | the lowest status among the parts flown: synthetic 0, placeholder 0, reference 2, qualified 3, flight-proven 4 |
  | Reproducibility | 4 when the campaign manifest is complete and every bundle verified, else 0 |

  So a result flown on a synthetic part can never look credible, however many runs back it.
- **Gate check 7g.** An evidence row has no inputs, names a metric kind `adcs-sim` knows, and lists at least one rung.

### 5.6 Families on the tree

The fifth field of each tree row holds **hardware tags**: `mtq`, `rw`, `fmr` and `rcs`. An empty field means the row is in play whatever is fitted. Each family in `catalogue/families.toml` lists its `tags`. A group is **in play** for a candidate (a case plus a product or part combination) when the group's tags meet the tags whose count row (`cf_*`) is above zero.

- **Seeding** leaves every group's `cases` list empty, which in VLEO means every case. Which groups are in play depends on the product, not the case, so it is decided at run time from the counts.
- **The face** hides a group that is not in play. It filters by tags as F14 describes, so the same rule works for the portal's cases too.
- **The resolver** answers `NotFitted { slot }` for every `product` or `tuned` row the candidate does not supply: the per-unit rows of an empty slot, and a tuned row no algorithm of the product sets. `NotFitted` is not a number. A row that reads it answers `NotFitted` too, naming the slot, so "knowledge error with the star tracker" on a product with no star tracker says so rather than answering with zero noise. The one exception is the zero-answer rule below. A sheet may declare `zero_when_absent = ["cf_1"]`, naming the count rows it reads. When those counts are zero, the `NotFitted` inputs whose hardware tags match those counts contribute exactly zero. This is an H7 kernel extension, with gate check 7h: a `zero_when_absent` row must read the counts it names. `ge_5`, knowledge error with the sensors fitted, uses the same mechanism: it declares `zero_when_absent = ["cf_4"]`, so with no star tracker it reads the magnetometer-and-sun knowledge, and a coils-only product still has an AKE to judge.
- **A run without a product** (the tree alone, as P1's acceptance runs it) reads the `product` and `tuned` rows' sheet values. Those are the reference configuration, IDMAS V2 as the seed content states it for the reference 3U satellite (source `adcs_ref_c1`), and the run says so. A candidate run replaces every one of them (§8.2).

**The zero-answer rule.** A row that describes one unit answers whatever is fitted: one ring's momentum at cruise speed, one slew's propellant if thrusters did all of it. A row that totals, aggregates or feeds a KPI reads the relevant count and answers exactly zero when it is zero. That covers every authority row, every budget total, and propellant per year. Such rows have `lower = 0` with the reason "zero is a real answer: … is not fitted", and their physics function returns zero at count zero. So a family without thrusters reports zero propellant per year, not the three grams a year a 500 kg bus's thruster arm would imply. The gap pass notes a count-fed row whose lower bound is above zero.

### 5.7 Seeding the tree

The seeder is VLEO's `tools/seed_tree.py` with `tools/cd06_rows.py` renamed `tools/plan_rows.py`. Change only the following.

`tools/plan_rows.py`:

```python
OWNER = {"svc": "systems", "cpt": "systems", "msn": "environment", "sat": "systems",
         "sub": "gnc", "ver": "verification",
         "sb0": "environment", "sb1": "sensing", "sb2": "actuators", "sb3": "gnc", "sb4": "avionics",
         "cas": "sales", "cat": "systems", "cmr": "sales", "ord": "programme", "fac": "facility",
         "std": "quality", "sup": "programme", "hrt": "programme", "rsk": "quality"}
TONE = {"svc": "amber", "cpt": "slate", "msn": "violet", "sat": "slate", "sub": "green", "ver": "teal",
        "cas": "teal", "cat": "slate", "cmr": "amber", "ord": "green", "fac": "green",
        "std": "violet", "sup": "violet", "hrt": "violet", "rsk": "amber"}
SYS_ROOT = "sys_satellite_adcs"
LAYER3_GROUP = {s["id"]: s["group"] for s in <tree>["layer3_shape"]}   # read, not hard-coded
LAYER3_OWNER = {"dist": "environment", "sens": "sensing", "est": "gnc", "mtq": "actuators",
                "rw": "actuators", "fmr": "actuators", "rcs": "actuators", "ctl": "gnc",
                "pnt": "gnc", "modes": "gnc", "fsw": "avionics", "budget": "systems"}
```

These are the maps used to run VLEO's code over the tree for this document. With them, the node ids come out exactly as recorded in `plan/expected_node_ids.json`, and P1 checks that they still do.

`tools/seed_tree.py`:

- the root label is "ADCS products and test facility";
- drop the `from nodes import (...)` list, `REPARENT`, every `amend()` call, the `l3_x_envorbit` addition and the VLEO `KPI(...)` calls;
- keep the `l3_x_closure` addition, which VLEO's seeder gives an interface row, `l3_x_closure_interface`, and generate the 38 closure rows of §5.5 in it, seeded;
- `SOURCES` come from `plan/seed_content.toml [[source]]`; later sources arrive through intake, each from a request's `new_sources`;
- the seeder writes no cases. Drop `CASES` and the case-writing half of `emit_supporting()`. After seeding, `adcs case import plan/cases/*.csv --into cases/` writes them through the one importer the portal also uses (§8.3), so there is one implementation of the format;
- the ADCS tree declares no cycle yet. Remove `CYCLE`, and remove the code in `emit_supporting()` that writes an `[[iterate]]` block into every case from it. `layers/cycles.toml` holds only its header comment;
- `owner_crate()` returns `adcs-mod-management`, `adcs-mod-system` and `adcs-mod-<sid>`. `CRATE_ALIAS` comes from `crate_skeleton.ALIAS`, which is set to `{"x_closure": "closure"}` (§3.4);

`seed_tree.py` still refuses to run on a tree that has a published sheet. It runs once, and what it writes is the tree's shape: every row's identity, place, owner, kind and the edges the tree declares, with no content. The one exception is the closures and requirement rows of §5.5, which the KPI list fixes. Every other seeded row answers `NotRun` until its content arrives through intake (§5.8).

**Changing the shape later** is the developer team's own work, never a form's. A new group is added with `tools/seed_tree.py --add-group <id> --under <parent>`, which writes that one group into `layers/` and nothing else, and refuses a group that exists. It is reviewed like any layer change, and the group's nodes then arrive as new-node requests (N01 accepts a group `layers/` declares, even with no nodes yet). A full reseed is possible only before any sheet is published, as VLEO's seeder already enforces.

### 5.8 Node content: seed forms, attestation, and what UNCONFIRMED means

**Every node's content arrives as a node form, including the first.** Seeding gives the tree its shape (§5.7). What a node answers, and how, reaches its sheet only through intake (§5.11). That holds for the pilot thread too. `plan/seed_content.toml` is the content of 82 rows, written from cited sources for this package: the pilot thread's 61 and the Risk management branch's 21. `tools/forms.py seeds` turns each row into a **seed form**: a node form whose request type is `seed`, whose base is the row as seeding leaves it, and whose proposal is the seed content. In P1 the developer team takes all 82 through the same intake a team member's form goes through:

```
cargo xtask intake check <seed form> --seed      # the checker; --seed accepts the package's own requester
<implementation agent, from the brief>            # intake write, then the HOLEs
cargo xtask intake verify <request>                # the sheet says what the form says
cargo xtask gate                                   # and every test vector passes
cargo xtask intake mark verified <request>          # in CI, when both pass
```

`--seed` accepts the request type `seed` and the package's own requester, and only for a node whose state is still `seeded`: a seed form can never overwrite content a person has sent since. There is no second path. The same checker, sheet writer and verifier then serve every request for the rest of the software's life, which is why they are built and proven on the first 82. Each seeded row starts its version history at version 1, "first build" (§5.13).

**States.** A row moves through VLEO's states, and intake decides which:

| State | Means | Reached by |
|---|---|---|
| `seeded` | shape only; answers `NotRun` | seeding |
| `specified` | content written from a request; not yet proven | `intake write` |
| `verified` | the gate passes, the regenerated artefacts match, every test vector passes, and `intake verify` found the sheet equal to its request | `cargo xtask intake mark verified <request>`, run by the intake CI job when verify and the gate pass on the branch |
| `published` | reviewed, merged and released | `cargo xtask intake mark published --release <version>`, run by `release.yml` for every node whose request the release carries |

`state` is the one field besides the content that intake writes, and only through `mark`. `mark` never lowers a state, so a confirmation of a `published` node leaves it `published`. `intake verify` never compares it.

Only `verified`, `published` and `deprecated` rows run (VLEO `State::runnable`).

**Attestation.** A node form has a field "Checked by": the engineer who has checked the relation against its source, and the values, and stands behind them. `intake write` copies that name, with the date and the request id, into `[maths] confirmed_by` and, for a declared row, `[value] confirmed_by`. Nothing else ever writes a name there: not the builder, not the implementation agent, not a script (rule 2). A request with the field empty writes `UNCONFIRMED · via <request id> · awaiting a person`. Seed forms always leave it empty, because the package's author is not a person who can attest.

A relation or value with nobody's name against it is not a failure; it is honest:

- the gap pass notes "the value has nobody's name against it";
- `xtask ready` holds the row;
- the row's own slot scores Mathematics 1 (for a relation) or InputPedigree 1 (for a value), so every number that reads it carries that low score and a client sees it.

It becomes somebody's when an engineer sends a node form of type **confirm**: no change, their name under "Checked by", and their reason. The checker refuses a confirmation that changes anything (F05). A confirmation needs no code and no agent: the checker writes `confirm.md` instead of a brief, and a developer runs `cargo xtask intake write request.json`, which in confirm mode writes only `[maths] confirmed_by`, `[value] confirmed_by` and `[request] last`, leaves the state as it is, and is verified and reviewed like any request.

**What credibility a seeded-then-specified row shows.** Take `gf_7` after its seed form is verified, scored by VLEO's `credibility.rs`:

| Factor | Score | Why |
|---|---|---|
| Mathematics | 1 | the relation has nobody's name against it |
| Assumptions | 3 | one assumption |
| Verification | 4 | the row is verified |
| Validation | 2 | tier B, test vectors pass |
| InputPedigree | 1 | it reads UNCONFIRMED declared values |
| Uncertainty | 1 | inherited: its declared inputs score 1 under VLEO's proxy, and the rollup takes the minimum |
| Understanding | 3 | one step |
| Reproducibility | 4 | data ok |

The lowest is 1, shared by Mathematics, InputPedigree and Uncertainty. Ties go to the earliest factor, so Mathematics governs. A client looking at a margin in the portal therefore sees that it rests on a relation and values nobody has confirmed, until an engineer confirms them by form.

### 5.9 Owners

`CODEOWNERS` is generated from each sheet's `owner` (VLEO F9). The owners are `systems`, `environment`, `sensing`, `actuators`, `gnc`, `avionics`, `verification`, `sales`, `programme`, `facility` and `quality`. Mapping each to a GitHub team is decision D12. The generator's fixed header gains the new crates (§4), `catalogue/`, `scenarios/`, `rig/`, `forms/`, `manual/` and `intake/`.

Each owner maps to a reviewer group inside the developer team (D12). That group reviews the intake branches of the nodes it owns (§5.11, step 4): that the implementation is what the form asked for, that the HOLEs compose physics functions and nothing else, and that the tests and downstream nodes pass. Whether the relation itself is right is not the reviewer's to assume. It is attested in the form by the engineer under "Checked by" (§5.8), and a relation nobody has checked is released as UNCONFIRMED, visibly, until someone does.

### 5.10 The node form: how anyone asks for anything

#### 5.10.1 The rule

Nobody outside the developer team changes the software, and the software never changes itself (§1.6). A team member who wants something different asks for it with a **node form**: one self-contained HTML file, `adcs-node-form/1`. The file is three things at once:

- **the document**: the node as the released software has it, read-only, written to the explanation standard (§5.12): the answer first, where the node sits, then its own station (say it simply, the relation with its why-chain, where it breaks, and a test vector the reader can rebuild), its versions and the beliefs and risks behind them;
- **the request form**: every field a team member may propose, each with the question it asks, why the answer matters and an example;
- **the request itself**, once saved, which the team member sends to the developer team. They answer in the same file.

It opens straight from disk in any browser. It needs no server, no network and no account, so it can be mailed, attached to an issue, dropped in a shared folder or uploaded in the portal. It carries no code, and none is expected from the person filling it. It carries what the code is written from.

`forms/node_form.html` is the template. `tools/forms.py` in this package fills it from the plan; in the built repository, `cargo xtask form export` fills it from `node.toml`. The examples in `forms/examples/`, written by `tools/make_examples.py`, are the package's output:

- `gf_7` (ring spin-down time), the explanation standard's worked example, `p1k_0` (the APE requirement) and `rk4_0` (the Risk management conclusion) as a team member receives them;
- `gf_7`'s seed form (§5.8);
- a blank new-node request;
- a returned request with its belief record, and the developer team's replies in its history.

#### 5.10.2 What a team member can ask for

The form first asks what the person wants. Each choice lights up only the sections it needs.

| Request | For | What it carries |
|---|---|---|
| **Change this node** | correcting or completing what a node says | the whole node as the person wants it; the checker computes what changed |
| **Ask for a new node** | a question the tree does not answer | where it goes (group, a sibling it is shaped like, label, id, kind) and the whole node |
| **Confirm this node** | an engineer has checked the relation, its source and its values, and stands behind them | a name under "Checked by" and a reason; nothing else changes (§5.8) |
| **Feedback on a release** | the software behaved unexpectedly | what was run (release, case CSV, result file), what happened, what was expected. Feedback may be sent in an older copy of the form, such as the one that came back "released": the checker does not hold it to the node's current base. |
| **Something else** | a part, an algorithm, a scenario, a campaign, the case format, a new group in the tree | a subject and a description, with attachments; the developer team turns it into the right change |

A door, an interface row and a closure are fixed by the tree's shape. Their forms offer only feedback and "something else". Any form may carry feedback entries as well as its request.

#### 5.10.3 The sections, and what each field asks

Every field shows three things: the question it asks, why the answer matters, and an example. The asks and whys of the sheet's fields are VLEO's own, from `form.rs` `FIELDS` and `ARRAYS`, so the form and the sheet say the same thing in the same words.

| Section | Fields | Shown for |
|---|---|---|
| Where the new node goes | group, sibling it is shaped like, label, proposed id (filled from group and label), kind | new |
| The question it answers | question; note (what it is NOT for); label; hardware tags | change, new |
| Its answer | symbol; quantity; unit (it must state the quantity); lowest and highest value, each with its reason | change, new |
| What it reads | inputs: binding, the node it reads, the quantity it must be; the producer's label and what it publishes are shown beside each | computed nodes |
| The relation | expression; source; why it is this relation; how to read the answer; the derivation, step by step | computed nodes |
| How it is computed | steps: what each does, what it binds, its quantity, and the physics function it calls, or a description of the new one it needs; the hardware counts that make it zero when absent | computed nodes |
| What has to be true | assumptions: what is assumed and when that stops being true | computed and declared nodes |
| Its value / the requirement / the evidence | a declared number and its source; a requirement's sense; an evidence node's metric and rungs. A value a case supplies is explained and not offered. | declared, required, achieved |
| Test vectors | one row per vector: an input per binding (SI), the expected answer (SI), tolerance, provenance, source and page | computed nodes |
| Sources | a new source's id, author and title, and exactly where | change, new |
| Attachments | any files that help: a page of the source, a plot, a MATLAB prototype, a spreadsheet; 5 MB in all, carried inside the file | all but confirm |
| Your request | subject and description | something else |
| Feedback on a release | release, case CSV, result file, severity, what happened, what was expected | all |
| Explain it | the node in plain words; in one line; a common wrong idea and why it is wrong; two cases, one difference; an analogy and where it breaks; a why-chain down to a law, a standard or a decision (§5.12.3) | change, new |
| De-risking: the belief behind this request | the area (one of seven); what was believed; whether a test broke it, held it, or nobody has tested it; what was tested, or the test that would settle it; what we now know; what was wrong with the version it replaces and what this version gives; what changed in the plan; what the test cost ($k); every risk it opens, raises, lowers or closes (§5.13) | change, new; a confirmation may carry a held belief |
| Explain it back | the request in one or two lines, as the requester would tell a colleague; it travels as `request.summary` | Learn depth |
| Who is asking, and why | name (a person, never a tool), team, contact, needed by, priority, the reason, and "Checked by" | all |
| Review and send | how much of the request is filled; every problem, marked ! (must fix) or i (recorded); every change, now against proposed; Save filled copy; Download answers (JSON) | all |

**The page checks what it can see, with the checker's rules.** A number parses. A unit states its quantity. An input names a node the software has, publishes the quantity declared, and sits in the same layer. The last step binds the answer. A test vector gives one number per binding and cites a page. A requester is not an assistant. Each problem links to its section, and the side navigation counts them per section. The checker (§5.11) is the authority: it applies every rule again, and more, on the developer's side.

**Status.** When the file comes back from the developer team, a "Status of this request" card shows their replies, newest last: received, check failed (with each finding), check passed, in progress, in review, released in a version, needs information, feedback noted, or rejected with the reason. The header shows the latest.

**Before the fields, the page explains the node** (§5.12): a breadcrumb (platform › layer › group › node); one answer-first sentence (what the node is, its kind, version and state, and the highest open risk it carries); "Where this node sits" (what it reads → the node → where its answer goes); the node's document as a station; and "Versions, and what they rest on" (every version with its request, belief, previous issue and benefit, and every belief about the node). Every section shows its kind (tutorial, how-to, reference, explanation), and the depth switch (Learn · Read · Expert) shows the exercises, the explanation, or the relation, limits and sources.

**What else the page does for the person filling it:**

- a "Start here" card that says in six steps what to do and what happens after sending;
- "What happens to this file after you send it", the intake of §5.11 in plain words;
- "Copy an assistant prompt": the exact instruction for editing the file with Claude or another assistant, so that what comes back is still a valid request. It never lets the assistant supply a name or a test-vector number;
- draft recovery: answers kept in the browser as the person types, and offered back if the page is reopened before saving. It is a convenience only; the saved file is the record, and the page works when the browser keeps nothing;
- print, light and dark themes, a layout that works on a phone, and keyboard access throughout.

#### 5.10.4 The file

Everything that matters is the JSON block `<script type="application/json" id="adcs-node-form">`, and the page says so.

| Member | Holds | Who writes it |
|---|---|---|
| `schema`, `title`, `exported`, `mode` | `adcs-node-form/1`; the release it was exported from | the exporter |
| `node` | the node as the release had it: identity, placement, kind, owner, state, and every field of the sheet, including its `explain` table; its `versions`; and, from the ledger, the `beliefs` about it and the `risks` they carry | the exporter |
| `base` | a hash of the node's identity and of every field a form may propose; not its versions, beliefs, risks or state, so a risk another request moves does not make this form stale | the exporter |
| `consumers`, `kpis` | where the node's answer goes | the exporter |
| `catalog` | the pick-lists: every node with the quantity and unit it publishes, groups, units with the quantities each states, quantities, sources with what each is used for, physics functions with their arguments, owners, tags, metrics, rungs, provenances, tiers, the risk register's risks with their levels, the seven areas and the belief statuses | the exporter |
| `request` | `type`; `proposed`, the whole node as wanted (including `explain`); `placement` for a new node; `new_sources`; `reason`, `summary`, `priority`, `needed_by`; `requested_by`, `team`, `contact`; `attested_by`; `derisk`, the belief record (area, believed, status, tested, would_test, now_know, previous_issue, benefit, plan_change, cost_k, moves: risk, from, to, and for a new risk its title, area and closing test); `other`; `attachments` (name, type, size, data URL, note) | the team member |
| `feedback` | entries: release, case, result, severity, observed, expected | the team member |
| `history` | replies: date, who, status, note, release, findings | the developer team, through `cargo xtask intake reply` |

**Save filled copy** writes a copy of the file with the JSON block replaced, named `<node>.request.<date>.html`. The rest is the same template, so a saved copy reopens with every answer in place and can be edited again. Every `<` inside the block is written as `\u003c`, so no value in it can end or open a script.

**Filling it by hand or with an assistant.** A person may type into the page, edit the JSON block in a text editor, or give the file to an assistant with what they want. Only `request` and `feedback` are meant to change. The checker reads those, `base`, and `node.id`, which names the node; it compares the rest of `node` with the repository and reports any difference (I01), and decides everything else from the node as the software has it (§5.11.2). A form whose other members were altered therefore changes nothing it should not.

#### 5.10.5 Where forms come from

- **The node library.** `cargo xtask form export --library <dir>` (`tools/forms.py library` in the package) writes every node's form, the new-node request, the case editor (§8.3.3) and an `index.html`. The index lists every node by layer and group, marks which are written and which are unconfirmed, and has search. Every release carries the library: in the MATLAB zip (`forms/`), and on the workbench's and the portal's internal downloads. It is how a team member reads the tree's documents offline.
- **Single nodes.** `cargo xtask form export <node>... [--layer <sid> | --group <id>] [--out <dir>]`.
- **From the software.** The web face's node page has "Ask for a change", which downloads that node's form from `GET /v1/form/<node>`. The portal serves node forms to internal roles only, because layer 3 is restricted from clients (D1).
- **Seed forms.** `tools/forms.py seeds` writes the 82 seed forms from `plan/seed_content.toml` (§5.8). They exist for the first build only.
- **Determinism.** An exported form of one node at one release is byte-identical every time. Its only date is the release's, when `SOURCE_DATE_EPOCH` is set, as the release sets it. CI compares two exports.

#### 5.10.6 Where a request is sent

A request reaches the developer team by any route that carries a file. Every route ends in the same place: the file, checked by `cargo xtask intake check`.

- **An issue**, with the "Request" issue template (`.github/ISSUE_TEMPLATE/request.yml`), which asks for the file and nothing else.
- **The portal**, for internal roles: `POST /api/v1/internal/requests` stores the file and opens the issue (§15.5).
- **Mail or chat** to the developer team, who attach it to an issue.

#### 5.10.7 Adding a request kind

A node form covers nodes, and "something else" covers the rest by hand. When one kind of "something else" becomes common, it earns its own form: a part's descriptor, an algorithm's bounds, a scenario. A kind is:

- an exporter that fills a template's JSON block;
- a template, or a branch in `node_form.html`, whose page states the kind's rules;
- a checker in `adcs-intake`, with its codes, and a writer and verifier for the files it changes.

None is built until a team needs it. A new kind is two reviewers, because its writer changes the repository.

#### 5.10.8 Acceptance

`tools/form_browser_check.py` runs all of this in headless Chromium, and this package's own run passes:

- A node form (`gf_7`) draws with no script error and reads "unchanged". Two edits, a name and a reason read "2 changes", and the page flags the missing belief record (D01) until the De-risking section is filled; then nothing is left to fix.
- The depth switch hides "Start here" in Expert, and in Learn hides the rebuilt test vector's answer until the reader asks, then judges the reader's answer against the tolerance.
- "Save filled copy" writes a file that reopens with both edits. `tools/intake.py` passes that file, reads back exactly the two changes, and writes a brief.
- An assistant's name as requester is flagged on the page, and refused by the checker (P01).
- A reply written with `intake reply` shows in the returned file's status card and header.
- A new-node request fills its id from the group and label.
- Unsaved answers are offered back after a reload.
- At 390 px, the phone width, the form has no sideways scroll.

- The node library draws, lists every exported node, and its search narrows the list.

The case editor's and the results' checks are in §8.3.6 and §13.5.6.

### 5.11 Intake: how a request becomes software

#### 5.11.1 The pipeline

Intake is the developer team's side of the node form. It is `cargo xtask intake` over the crate `adcs-intake`, run in a checkout. `tools/intake.py` is its stand-in in this package, and runs today on the plan.

```
request file ─▶ 1 CHECK ─▶ 2 IMPLEMENT ─────────────────────────▶ 3 VERIFY ─▶ 4 REVIEW ─▶ 5 RELEASE ─▶ reply
                │           implementation agent, from the brief:               owner,       next
                │           intake write (sheet, test vectors),                 gate,        release
                │           the HOLEs, new physics functions                    downstream
                └─ fail ─▶ reply "check failed" with every finding ─▶ the team member fixes and resends
```

| Step | Command | Who | Writes |
|---|---|---|---|
| 1 Check | `cargo xtask intake check <file> [--out intake/requests]` | a developer | `intake/requests/<request id>/`: `request.json` (the trusted part), `check.md` and `check.json` (the report), `attachments/`, and, on a pass, `brief.md` (change, new) or `confirm.md` (confirm) |
| 2 Implement | the implementation agent, started by a developer with `brief.md` (§17.4); its first step is `cargo xtask intake write request.json` | the agent, watched by the developer | the node's `node.toml`, `fixtures.toml`, `versions.toml` and `versions/<n>/`, `derisk/beliefs/<request id>.toml` and the moves in `derisk/risks.toml` (by `intake write` only); the node's HOLE blocks; new functions in `adcs-core::physics` with their property tests, each with its MATLAB twin in `matlab_sils/+asils/+physics/` (§10.8.7); for a new node, `cargo xtask new <id> --like <sibling>` first |
| 3 Verify | `cargo xtask intake verify request.json`, then `cargo xtask intake mark verified` | the agent, then CI again, which marks the node `verified` when verify and the gate pass | only `state`, through `mark` |
| 4 Review | the pull request of branch `intake/<request id>` | a developer from the node's owner group (CODEOWNERS, §5.9), plus a second reviewer for anything H7 or significant | approval, or a reply "needs information" |
| 5 Release | the ordinary release (§18), which runs `intake mark published --release <v>` | the developer team | the new version, with the request listed in its release notes; the node `published` |
| Reply | `cargo xtask intake reply <file> --status <s> [--note] [--release] [--report check.json] --out <file>` | a developer | the returned file, with the reply in its `history` |

The request id is `REQ-<date>-<node>-<hash>`, from the request's content. It names the working folder, the branch, the commit trailer `Request:`, the copy of the request kept in the node folder (`requests/<request id>.request.html`), and the release note's line. So a node's history is its requests, and every change can be traced to the person who asked, their reason and their test vectors.

**Feedback and "something else"** go through step 1 only. A passing check turns each feedback entry into an issue (`cargo xtask intake issue <file>`, with the node, the release, the case and result named, and the file attached), labelled `request:feedback` and assigned by CODEOWNERS. "Something else" becomes one issue, labelled `request:other`, for the developer team to plan like any of their own work. Both get a reply.

#### 5.11.2 The checker

The checker reads only the JSON block of the file. It never opens the page, so a received file's script never runs on a developer's machine. It decides everything from the node as the repository has it now, never from what the file says the node was. It applies every rule below, prints the report, and exits 0 only when nothing is an error. `python3 tools/intake.py codes` lists them.

| Code | The rule |
|---|---|
| F01 | the file holds an `adcs-node-form/1` block, and is no larger than 25 MB |
| F02 | the request type is change, new, confirm, feedback or other (seed only with `--seed`, at the first build) |
| F03 | the node exists and is offered for change (not a door, an interface or a closure); or a new node's id is free, well formed and prefixed for its layer (`mgt_`, `sys_`, or `l3_<sid>_`) |
| F04 | a change or a confirmation was exported from the node as it is now; otherwise the report names every field that has moved since, and the team member exports a fresh form. Feedback and "something else" may come on an older copy. |
| F05 | a change changes something; a confirmation changes nothing |
| P01 | the requester is a person, named, and not a tool or an assistant |
| P02 | a change, a new node and a confirmation say why |
| P03 | whoever is named as having checked the maths is a person; a seed form names nobody |
| P04 | a confirmation names who checked the node |
| N01 | a new node's group exists (holding nodes, or declared in `layers/` with none yet), and its model sibling is in that group |
| N02 | a new node's kind is computed, declared, required or achieved |
| I01 | an existing node's id, kind, layer, group and owner do not change through a form |
| I02 | a node fed by the case format keeps its label, quantity and unit: that is a case-format change (§8.3.5) |
| I03 | hardware tags are `mtq`, `rw`, `fmr` or `rcs` |
| O01 | the answer has a symbol of letters, digits and _ |
| O02 | the answer's quantity is one the software knows |
| O03 | the answer's unit is one the software knows, and it states that quantity |
| O04 | the lowest and highest values are numbers, in order, and each has a written reason |
| C01 | a computed node reads at least one input; any other kind reads none |
| C02 | each input has a unique binding and reads a node that exists, other than itself |
| C03 | each input's declared quantity is what its producer publishes |
| C04 | inputs stay inside the node's layer: layers meet only at the door and the interface rows |
| C05 | the new connections make no loop |
| C06 | "zero when absent" names hardware count nodes (`cf_*`) the node reads |
| C07 | *warning*: an input's producer has no answer specified yet; implementation waits for it |
| R01 | a computed node has a relation, a source, a reason why, and at least one step |
| R02 | each step says what it does, and names a physics function that exists or describes the new one it needs |
| R03 | the last step binds the answer's symbol |
| R04 | *warning*: the relation uses names that are neither bindings nor step results nor ordinary maths |
| R05 | *warning*: a named physics function takes a different number of arguments than the node has inputs |
| R06 | each assumption says what it assumes and when that stops being true |
| R07 | each derivation step has text |
| V01 | a declared node that no case, product, tuning, evidence, ledger or lab file supplies has a numeric value inside its bounds, and a source; a product or tuned row may leave its reference value blank; a value the evidence tooling, the de-risking ledger or a lab file supplies is never set by a form |
| V02 | a requirement says which way it binds |
| V03 | an evidence node names a metric the simulator computes and at least one rung |
| S01 | every cited source is known, or added in the form with an id, a title and exactly where |
| T01 | a test vector's provenance is independent-derivation, published-source, independent-tool or physical-bound; never self-snapshot or agent-generated |
| T02 | a test vector cites a known source and the page, table or figure |
| T03 | a test vector gives one number per binding, a numeric expected answer and a tolerance above zero |
| T04 | *warning*: a computed node has no test vector, so its validation stays low |
| A01 | attachments are plain file names, 5 MB in all |
| B01 | a feedback entry says what happened |
| B02 | an "other" request has a subject and a description |
| W01 | *warning*: nobody is named under "Checked by", so the node will run as UNCONFIRMED |
| D01 | a change or a new node carries its belief record: its area (one of seven), what was believed, what we now know, and what changed in the plan |
| D02 | a change says what was wrong with the version it replaces and what this version gives |
| D03 | the belief's status is broke, held or untested; broke and held say what tested it; untested says the test that would settle it |
| D04 | each risk it moves is in the register at the level it moves from, or is new (from 0) with a title, an area, a level 1 to 5 and the test that would close it |
| D05 | a risk is lowered or closed only by a tested belief |
| D06 | the cost is blank or a number, zero or more, in thousands of dollars |
| D07 | *warning*: the belief is untested, and is counted under "Beliefs not yet tested" until a test is recorded |
| X01 | an analogy in the node's explanation says where it stops being true |
| X02 | a common wrong idea says why it is wrong |
| X03 | *warning*: a computed node does not say itself simply and in one line, so its document serves experts only |

**The report** (`check.md`) gives the verdict, every finding with its code and section, what changes (field, now, proposed), the connections added and removed, the **impact** (every node downstream of this one, which the gate re-tests, and the KPIs it feeds), any new physics functions needed, and the **de-risking** table: the belief, its status and test, the previous version's issue, this version's benefit, and the risks moved. The developer reads the report before anything else. A pass means the request is complete and consistent with the tree. It does not mean the relation is right: that is what the engineer under "Checked by" attests, and what the test vectors test.

**Interfaces are checked against the tree, not the form.** The producer of every input, the quantity it publishes, its layer, and the loop check all come from the repository. So a form cannot connect a node to something that does not exist, or in a way the tree forbids, however it was edited.

#### 5.11.3 The implementation agent's brief

On a pass, the checker writes `brief.md`. It is the implementation agent's whole task, and nothing outside it is in scope (§17.4):

- **scope**: the node, its crate, and the files it may change: `node.toml`, `fixtures.toml`, `versions.toml`, `versions/` and `derisk/` only through `cargo xtask intake write`, the node's HOLE blocks, and `adcs-core::physics` with its MATLAB twin `matlab_sils/+asils/+physics/` only for the new functions the request describes;
- **in order**: `intake write`; `cargo xtask docs <node>`; each HOLE, one per step, calling the physics function the step names with the bindings in declared order (or writing the new function first, in VLEO's style, with property tests only, and its MATLAB twin in the same commit, run on the request's test vectors); `intake verify`; the gate and the tests, including every downstream node; a commit on `intake/<request id>` with the trailer `Request:`, the request file copied into the node folder; no push;
- **never**: supply an expected value; write a person's name; widen a tolerance or skip a test; put a formula in a HOLE (F7's check 10b enforces it); edit outside the scope. If the request cannot be implemented as written, the agent stops and says why, and the developer replies to the requester.

#### 5.11.4 The sheet writer and the verifier

**`cargo xtask intake write request.json`** is the only path by which a request's declarative content reaches a sheet, a node's version history, or the risk ledger. A requirement is written as VLEO writes one, `kind = "declared"` with a top-level `sense` (§5.5). It renders `node.toml` and `fixtures.toml` in VLEO's sheet shape with `form.rs`'s pure `set`, `normalise` and `value_allowed`: the question and note; the relation and source; the theory, its steps and the assumptions; the output with its bounds and reasons; a declared value; a requirement's sense; an evidence row's metric and rungs; the inputs; the algorithm steps; hardware tags; zero-when-absent; `[request] last = "<request id>"`; and `confirmed_by` from `attested_by`, or `UNCONFIRMED · via <request id> · awaiting a person` (§5.8). It sets the state to `specified`. In confirm mode it writes only the `confirmed_by` fields and `[request] last`. It refuses a request that does not pass the check. `tools/intake.py sheet` is its stand-in; the stand-in reads the form file, where the repository's command reads `request.json`.

Besides the sheet it writes what §5.13 names: the previous sheet copied to `versions/<n>/` and version *n + 1* appended to `versions.toml` (request id, belief, previous issue, benefit, `release` empty until `mark published`); the belief record `derisk/beliefs/<request id>.toml`; and each risk move as a history entry of `derisk/risks.toml`, a new risk taking the next free `R-nn`. The node's `[explain]` table (§5.12.3) is written with the sheet. A confirmation writes no version; a held belief it carries is recorded in `derisk/beliefs/`. The package's `tools/intake.py sheet` writes the three ledger parts as `belief.toml`, `versions.entry.toml` and `risk_moves.toml` beside the sheet.

**`cargo xtask intake verify request.json`** compares the node, as implemented, with the request, field by field:

- every field of the sheet, including `[explain]`;
- the version entry and the belief record, against the request's De-risking section;
- every test vector in `fixtures.toml`;
- each HOLE calls the physics function its step names;
- `confirmed_by` is the attesting person's, or UNCONFIRMED.

Any difference fails it, by field. It never compares `state`. It runs again in CI on the intake branch, and when it and the gate pass, CI runs `intake mark verified`. `tools/intake.py verify` is its stand-in for the sheet and the test vectors.

#### 5.11.5 What intake never does

- It never runs the page of a received file.
- It never writes a node from anything but a passing request, and never changes a node the request does not name.
- It never merges or releases. The only commits it makes itself are the `state` commits of `intake mark`: the intake CI job's on an intake branch, under the CI's own identity, never a person's, and the release's preparation commit. A person reviews every change (H1 for a relation, H2 for a test vector).
- It never supplies a name, a number or a relation. What the software says comes from people, through forms; how the software computes it comes from the implementation agent under a brief, verified against the form.

#### 5.11.6 Acceptance

`python3 tools/intake.py selftest` passes in this package:

- 66 deliberate mistakes, at least one for every rule that is an error (the D and X rules included), are each refused by their code;
- 88 clean requests pass: a real change with its broken belief, a change whose broken belief lowers one risk and opens another, a new node on an untested belief, a confirmation, feedback (including feedback on an older copy), and all 82 seed forms;
- all 82 sheets written from the seed forms verify against their own requests;
- a sheet with one unit changed, a test vector with its expected value changed, and a belief record with its status changed are each caught by verify; the moving change writes version 2 of `gf_7` and names R-04 and the new R-18.

In the built repository, the `intake` CI job runs the same selftest against `adcs-intake`, on a scratch tree freshly seeded from `plan/` (so the seed forms always meet `seeded` nodes, whatever the repository's own nodes now say), and the 82 seed forms go through the full pipeline in P1 (§19).

### 5.12 The explanation standard, `adcs-explain/1`

**In one line:** every page this platform shows a person (the node form, the case editor, the node library, a result, the quarterly narrative, every manual page) is written to one standard, taken from *Eight Loops, One Beam*, so that each one explains itself to someone who has never seen it.

#### 5.12.1 Why there is a standard

Say it simply first. The software never changes itself, and the people who change what it says are not the people who wrote it (§1.6). So every document is read, and often filled in, by someone new, with nobody beside them to explain it. A document that only its author understands turns into requests that fail the checker, results read the wrong way, and cases edited blind.

*Eight Loops, One Beam* (v2) is a worked example of teaching one hard system, an RF ion thruster and its controller, to three kinds of reader at once. It does two things this platform needs:

- the **learner's loop** (ten moves): map what you know, explain it yourself, find the gaps, predict before you test, go back to the source, ask why until you reach a law, rebuild the number, simplify, repeat, teach it back;
- **thirteen techniques for the explainer**: words beside pictures, live models, concrete before abstract, overview then zoom then details, side-by-side comparison, zoom levels, typed sections, answer first, depth for novice or expert, worked then faded examples, contrasting cases, correcting a common wrong idea, and analogies that say where they stop being true.

The standard below is those, turned into rules a page can be checked against. The example stays the example: the platform's own worked example of the standard is one node's document, the ring spin-down time (`gf_7`), walked through rule by rule in the developer manual ([Explaining](../manual/developer/07_explaining.md)).

Where the simple version breaks: a standard cannot make a page clear. It can only make sure the moves that make pages clear are all there. Whether a reader understood is tested by a reader (the teach-back test of risk R-17, §5.13), not by `tools/explain_check.py`.

#### 5.12.2 The rules

Four groups, as in the example: finding your way, seeing it, learning it, and not fooling yourself. The **Mark** column is what `tools/explain_check.py` looks for (in HTML, a `data-` attribute on the rendered page; in Markdown, the text named). A rule with no mark is reviewed by a person.

| Rule | What the page does | Mark |
|---|---|---|
| **Find your way** | | |
| E01 Answer first | The first sentence under the title answers the reader's question: a result's verdict; what a form is for and what the reader has at the end; a manual page's answer. Everything after it is support. | `data-explain="answer"`, the first block in `main`; in Markdown, the first paragraph starts `**In one line:**` |
| E02 Say what kind it is | Every section is one of four kinds, and shows it: **tutorial** (learn by doing it once), **how-to** (do one task), **reference** (look something up), **explanation** (understand why). A section never mixes two. | `data-kind` on each section, with a visible badge; in Markdown, `<!-- kind: … -->` under the title |
| E03 Three depths | One switch, **Learn · Read · Expert**, on every HTML page. *Learn* keeps every exercise, with answers hidden until tried. *Read*, the default, shows the answer, the explanation and the pictures, and skips the exercises. *Expert* shows the answer, the relation, the limits and the sources, and hides the tutorial text. A Markdown page is written at one depth and says which. | a `data-depth-switch` with three buttons; content carries `data-show`; in Markdown, `depth:` in the kind line |
| E04 Overview, then zoom, then details | A map of the whole comes before any part. Details open on demand (a `<details>`, a hover on a source), never in the way. | `data-explain="overview"` before the first detail section |
| E05 Where you are | A breadcrumb in zoom levels, largest first: Platform › Layer › Group › Node, or Store › Case › Result. | `data-explain="zoom"` |
| **See it** | | |
| E06 Words beside pictures | A label sits on the thing it names, not in a legend elsewhere. A picture has one sentence saying what to see in it. | person |
| E07 Same axes | Things compared are drawn side by side, on the same axes and scales. Never two y-axes. | person |
| E08 Rebuild it | Where the page holds a known answer (a test vector, a campaign's metric), the reader can try to reproduce it before it is shown. | `data-explain="rebuild"` where the page has a test vector |
| E09 Never colour alone | Every state (pass, fail, changed, open, closed) is an icon and a word. Colour only repeats it. | `data-state` elements carry an icon and a word |
| **Learn it** | | |
| E10 Station order | A concept is explained in four steps, in this order: **Say it simply** → **Now the real thing** → **Where the simple version breaks** → **Try it**. | `data-step="simply|real|breaks|try"` in order |
| E11 Worked, faded, solo | An example filled in, then one half filled, then the reader's own. In a form: the example beside the field, the reference value shown, then the blank. | person |
| E12 Two cases, one difference | Where two things are easily confused, they are shown side by side, differing in exactly one thing. | `data-explain="contrast"`, not empty; whether it differs in one thing only is a person's check |
| E13 Common wrong idea | The wrong idea is stated plainly, then why it is wrong. Stating only the right idea leaves the wrong one in place. | `data-explain="wrong-idea"` containing `data-explain="because"` |
| E14 Analogy with its limit | An analogy keeps the relations that matter, and says where the story lies. | `data-explain="analogy"` containing `data-explain="analogy-breaks"` |
| E15 Why, down to a floor | "Why?" is asked until it reaches a named law, a standard or a recorded decision, and stops there, naming it. | `data-explain="why"` with a non-empty last item; that the last item names a floor is a person's check |
| E16 Predict before reveal | In Learn, the reader is asked for a guess before the answer is shown. | `data-explain="predict"`, shown in Learn only |
| E17 One line, then teach back | Each station ends with its one-line summary. Each document ends, in Learn, with a prompt to explain the whole back in a few lines. | `data-explain="one-line"`; `data-explain="teach-back"` shown in Learn |
| **Don't fool yourself** | | |
| E18 Every claim carries its tag | Any number or statement that is not the reader's own says what it is: **sourced** [n] (with the page), **derived** (worked here from sourced relations), **reference** (the reference case's value, UNCONFIRMED until a person confirms it), **example** (a demonstration or a made-up number, labelled so). Fiction is never unlabelled. | `data-claim="sourced|derived|reference|example"` |
| E19 Sources say what they are used for | Every source listed says what this page uses it for. | each source entry has `data-explain="used-for"`; in Markdown, a "Used for" column |
| E20 Say where it breaks | Every document says where it stops being reliable: the assumptions and when each fails, the credibility, the open risks. | `data-explain="breaks"` |
| E21 Plain words first | The plain description comes before the technical name, never instead of it ("say it without the word"). | person |

**Kinds of page, and the sections each must have.** In Markdown (the manuals), `tools/explain_check.py` requires, by kind:

| Kind | Must have |
|---|---|
| tutorial | numbered steps; a "Try it" section; an "Explain it back" section |
| how-to | numbered steps; an "If it goes wrong" section |
| reference | at least one table |
| explanation | "Say it simply"; a section whose heading says where it breaks; "Common wrong idea" |

Every Markdown page has the kind line and the one-line answer. A page may hold several sections of one kind; a section of another kind goes on its own page.

#### 5.12.3 Where each template applies it

| Page | Answer first (E01) | Stations (E10) | Where it breaks (E20) |
|---|---|---|---|
| Node form (§5.10) | what this form is for, what the reader sends, and the node's state in one sentence | the node's document: its own *Say it simply*, the relation, its assumptions, and *Rebuild it* from its test vectors | assumptions and when they fail; UNCONFIRMED; the node's open risks |
| Case editor (§8.3.3) | whether the case is ready to run, and what is missing | each section: what it sets, the reference value, the edit | a blank row blocks named scenarios; ranges are drawn from, not checked |
| Node library (§5.10.5) | how many nodes are written, confirmed and at risk | — | unwritten and unconfirmed nodes named |
| Result viewer (§13b) | the verdict sentence: how many requirements pass, on what engine, at what credibility | each metric: what it measures, the number, the margin, what could make it wrong | the engine, demo or real; the credibility; the assumptions of the rows read |
| Quarterly narrative (§5.13) | the platform's conclusion: highest open level, net risks closed, share of beliefs tested | each belief: believed, tested, now know | untested beliefs; open L4 and L5 risks |
| Manual pages | `**In one line:**` | explanation pages | explanation pages: "Where … breaks" |

**The node's own explanation.** A node form carries an optional `explain` object in `request.proposed`: `simply` (the node in plain words), `one_line`, `wrong_idea` and `wrong_because`, `contrast` (two cases, one difference), `analogy` and `analogy_breaks`, and `why_chain` (a list, its last item naming the law, standard or decision it stops at). `intake write` writes them into the sheet's `[explain]` table, and the node's document shows them as its station. Intake checks X01 to X03 (§5.11.2): an analogy must say where it breaks, a wrong idea must say why it is wrong, and a computed node without `simply` and `one_line` is a warning, because its document then serves experts only.

#### 5.12.4 Checking it

`tools/explain_check.py` (in the repository, `cargo xtask explain check`) renders every HTML template's examples in headless Chromium and reads every manual page, and reports each missing mark by rule and file. It runs in the `explain` CI job (§18) with the form browser check. Its own `--selftest` removes or breaks each mark it checks (E01–E05, E08–E10, E12–E20 in the rendered pages; the kind line, the one-line answer, each kind's sections and E19 in Markdown) and requires the check to fail by that rule.

It checks presence and order, not quality. The quality check is a reader: before a template is released, one person who has not seen it fills it, or reads it and explains it back, and each miss is fixed. That test is how risk R-17 is lowered (§5.13).

#### 5.12.5 Common wrong idea

"The standard makes every page longer." It does not have to. *Answer first* and the depth switch make the page shorter for most readers: Read hides the exercises and Expert hides the tutorial. What grows is what is available, not what is in the way.

---

### 5.13 De-risking: beliefs, versions and the risk branch

**In one line:** every decision the platform rests on (a node, an input, an output, a model, a relation, an algorithm, a picture) is recorded as a *belief* with the test that would settle it; a node gets a new version only because a belief broke or a risk needs lowering; each version keeps what was wrong with the last and what it gives; and the risk branch of layer 1 adds all of it up into one conclusion per release.

#### 5.13.1 Say it simply

Every choice in this platform is a bet: that a dipole field is close enough, that five pointing errors add in quadrature, that one CSV holds a whole case. Most bets are right. The expensive ones are the wrong bets nobody wrote down, because nobody knows which results rest on them.

So each bet is written down as a **belief**, with what would test it. When a test breaks a belief, the node that rested on it gets a **new version**, and the version says what was wrong with the last one and what the new one gives. Each belief that is still untested is carried by a **risk** with a level. And a branch of the tree, **Risk management**, counts it all, so a release can say in one line how much of what it rests on has met evidence.

This is the company's quarterly de-risking narrative (its template's title and columns are regenerated as `derisk/narrative_template.xlsx`: what we believed, what we tested, what we now know, what it cost, what changed in the plan, risks opened and closed), kept for every decision instead of once a quarter, and joined to the nodes it is about.

#### 5.13.2 Now the real thing

| Object | Where it lives | What it is |
|---|---|---|
| Risk register | `derisk/risks.toml`, `adcs-risk-register/1` | every risk `R-nn`: title, area, owner (a team), why it is real, what is done, the **closing test**, its level 0–5, and every move of the level (quarter, from, to, the belief that moved it) |
| Belief record | `derisk/beliefs/<id>.toml`, `adcs-belief/1` | quarter; area; what it is about (node ids, paths); what we believed; status `broke`, `held` or `untested`; what we tested; what we now know; what it cost ($k); what changed in the plan; for a node version, the previous version's issue and this version's benefit; for an untested belief, the test that would settle it; the risks it carries; its moves of risk levels |
| Node version | the node folder: `versions.toml` and `versions/<n>/node.toml`, `fixtures.toml` | one entry per version: number, release, request id, belief id and status, previous issue, benefit. The folder keeps the sheet as each version had it |
| Risk branch | layer 1, `mgt_risk_management` (§5.2): Risk register, Beliefs and versions, Open risk by area, Conclusion | 18 declared rows counted from the ledger (supplier `derisk`, never a form) and 3 computed conclusion rows |
| Narrative | `cargo xtask derisk narrative --quarter Q3-26` | one page per quarter, HTML to the explanation standard, and the same table as `.xlsx` and `.csv` in the template's columns |

**Areas.** A belief and a risk each belong to one of seven areas: *node*, *input* (the case format and what a run reads), *output* (results, certificates, what leaves the company), *model* (the plant, the environment, the rig), *math* (the relations in `adcs-core::physics`), *algorithm* (estimation, control, tuning, the solver, the designer), *visualisation* (every page a person reads). The "Open risk by area" rows are the highest open level in each.

**Levels** (proposed; decision D26 confirms the scale and every starting level at P1):

| Level | Name | If it happens |
|---|---|---|
| 0 | closed | — |
| 1 | negligible | one run is repeated |
| 2 | minor | one node needs a new version |
| 3 | significant | a release is withdrawn or a quote is redone |
| 4 | major | a result already delivered, or a certificate, is in doubt |
| 5 | critical | delivery stops, or something leaves that must not |

The idea is ECSS-M-ST-80C's severity and likelihood; the mapping to these five levels is ours, which is why D26 records it.

**The rules of the ledger** (`cargo xtask derisk check` over the crate `adcs-derisk`, and `tools/derisk.py check` in the package):

- A level moves only through a belief, and the move is written in both files with the same quarter, from, to and belief id.
- A level goes **down** only by a tested belief (`held` or `broke`, with `tested` filled). Nothing is lowered by argument, by review or by time passing.
- A level may go **up** at any time, by any belief, tested or not. Bad news is recorded the day it is known, not saved for a meeting.
- An untested belief names the test that would settle it. An open risk names its closing test.
- Owners are teams (§5.9). The person who recorded a belief is the requester of its request.

**How a version is made.** A change or new-node request carries a **De-risking** section (§5.10.3): the belief record. Intake checks it (D01 to D07, §5.11.2). `intake write` then does three things besides the sheet:

1. copies the node's current sheet to `versions/<n>/` before writing the new one, and appends version *n + 1* to `versions.toml`, with the request id, the belief id (the request id), the previous version's issue and this version's benefit;
2. writes `derisk/beliefs/<request id>.toml`;
3. applies the belief's moves to `derisk/risks.toml`, each as a history entry.

`intake verify` compares all three with the request. `intake mark published --release <v>` stamps the release on the version. A seed form writes version 1, "first build", with no belief record of its own; the package's beliefs about the pilot rows arrive with `derisk/` (P0). A confirmation writes no version: it is a belief that **held**, recorded in `derisk/beliefs/` when the form carries one.

**Decisions outside the tree.** A model in `adcs-sim-core`, a change to the result viewer, a new algorithm: the developer team's own change records its belief with `cargo xtask derisk record` (a belief file, and the moves), and the `derisk` CI job refuses a pull request that touches `crates/adcs-sim-core`, `crates/adcs-sim`, `crates/adcs-tune`, `crates/adcs-solve`, `crates/adcs-result`, `web/`, `forms/`, `results/`, `catalogue/algorithms/` or `scenarios/` without a new belief file, unless it carries the label `derisk:none` and a line saying why nothing believed changed (a refactor, a typo).

**The rollup.** `cargo xtask derisk rollup [--quarter Q]` counts the register and the beliefs into the 18 declared risk rows. The release's preparation commit writes the counts to `derisk/rollup.toml`, and `adcs-mod-management` reads them through the supply map, as a product row reads the catalogue bundle; so the values are what those rows answer in that release, and a form never sets them. The three conclusion rows are computed by the engine, like any other: the highest open level (`risk::highest_level`), the net risks closed this quarter (`risk::net_closed`), and the share of beliefs tested (`risk::share_tested`). Together they are the conclusion the platform reports about itself, on the Risk management page of the web face and the portal (§15), and at the top of the narrative.

**The narrative.** `cargo xtask derisk narrative --quarter Q3-26 --out <dir>` writes `narrative_Q3-26.html` (the conclusion first, then the risks by area on the same scale, then one row per belief of the quarter, then every open L4 and L5 risk), `narrative_Q3-26.xlsx` in the template's seven columns, and `.csv` in the same seven plus three that identify each row (belief, status, area). The paragraph of prose at the top is a person's: the quality team writes it in `derisk/narratives/Q3-26.md` (D27). Until it is written, the page says so, and nothing is written in its place.

#### 5.13.3 Where the simple version breaks

- **A belief is only as good as its test.** "Held" means the named test passed, not that the belief is true beyond it. B-004, for example, held for the reference cases' round trip; whether people find the case editor enough is a different belief.
- **The maximum hides progress.** The highest open level does not move until the last risk at that level closes. That is why the conclusion has three rows, not one (B-014 records this as a belief, untested).
- **Counts can be gamed** by splitting one risk into several. The register is reviewed like code (H-rules, §17), and D26 says who may open and lower a risk.
- **Starting levels are proposals.** The package wrote them from §21 without a test. Every one says so in its history until D26 confirms it.

#### 5.13.4 Common wrong idea

"A new version means the old one was a mistake." No. A version replaces its predecessor because a belief was tested; the old version was the best bet on what was known then, and its sheet is kept. A broken belief is the ledger working: it turns a hidden risk into knowledge. What would be a mistake is changing a node without saying which belief moved.

#### 5.13.5 Try it

Open `gp_5`'s node form (the APE budget total). Before reading its assumptions, predict which risk the row carries and at what level. Then open `derisk/beliefs/B-008.toml` and find the risk it opened, and its level, with `python3 tools/derisk.py table --check` passing and R-15 in SPEC.md §21. Last, run `python3 tools/derisk.py rollup`: the maths area shows L4, not R-15's L3. Why? (Another maths risk, R-03, is higher.)

#### 5.13.6 Acceptance

In this package:

- `python3 tools/derisk.py check` passes on `derisk/`, and `python3 tools/derisk.py selftest` refuses each deliberate mistake in the ledger by its rule;
- `python3 tools/derisk.py rollup` gives the 18 risk rows' values, and `narrative` writes the page, the `.xlsx` and the `.csv`;
- `python3 tools/intake.py selftest` includes a change with a broken belief that moves a risk, and refuses each D and X mistake by its code;
- `python3 tools/explain_check.py` passes on every template's examples and every manual page.

---

## 6. Units, physics and reference data

### 6.1 Units and quantities to add

`adcs-units` is VLEO's `vleo-units` plus the following. Each one goes in the `units!` table or the `QUANTITIES` registry. `tests/the_registry_matches_the_types.rs` then keeps the two in step, as it does in VLEO. `plan/units.toml` lists the whole registry, VLEO's and these, with the quantities each unit may state: it is what the node form's pick-lists and the intake checker read in this package (O02, O03), and in the built repository `adcs-intake` reads `adcs-units` itself.

| Add | Kind | SI factor | Why |
|---|---|---|---|
| `MomentOfInertia` | quantity (kg·m²) | — | VLEO declared inertia as `Ratio`/`One`; a unit is a type, and inertia is not a ratio |
| `DynamicViscosity` | quantity (Pa·s) | — | ring fluid |
| `AngularAcceleration` | quantity (rad/s²) | — | slew and wheel dynamics |
| `AngleRandomWalk` | quantity (rad/√s) | — | gyro noise |
| `RateRandomWalk` | quantity (rad/s/√s) | — | gyro bias drift |
| `KilogramSquareMetre` | unit | 1 | inertia |
| `PascalSecond`, `MillipascalSecond` | unit | 1, 1e-3 | viscosity |
| `Millimetre` | unit | 1e-3 | ring bore |
| `Microtesla`, `Nanotesla` | unit | 1e-6, 1e-9 | field, magnetometer noise |
| `MicronewtonMetre`, `MillinewtonMetre` | unit | 1e-6, 1e-3 | disturbance and actuator torque |
| `MillinewtonMetreSecond` | unit | 1e-3 | ring and wheel momentum |
| `Milliwatt` | unit | 1e-3 | holding power |
| `Kilopascal` | unit | 1e3 | pump pressure |
| `Litre` | unit of `Volume` | 1e-3 | volume allocated to the ADCS, and its volume requirement, in the case CSV |
| `DegreePerHour` | unit | 4.848136811095360e-6 | gyro bias |
| `DegreePerSqrtHour` | unit | 2.908882086657216e-4 | gyro angle random walk |
| `DegreePerHourSqrtHour` | unit of `RateRandomWalk` | 8.080228018492267e-8 | gyro rate random walk |
| `RadianPerSecond2` | unit of `AngularAcceleration` | 1 | slew and wheel dynamics |
| `IndianRupee` | unit of `Money` | 1 | layer 1 prices; the currency convention is decision D13 |
| `RiskLevel` | quantity and unit (L) | 1 | the risk rows of layer 1 (§5.13): 0 none open, 1 negligible to 5 would stop delivery. Ordinal: `risk::` compares and takes maxima of it, and nothing adds it |

Attitude types go in `adcs-units/src/frames.rs` beside VLEO's `Body`, `Eci`, `Ecef` and `Lvlh`. `Quat<To, From>` is scalar-first, unit-norm, and typed by its frames, so composing `Quat<Body, Eci>` with `Quat<Eci, Ecef>` compiles and composing it with `Quat<Body, Ecef>` does not. `Dcm<To, From>` works the same way. Every rotation uses `pmath`. These types are shared by `adcs-core` and `adcs-sim-core`, and the Mars Climate Orbiter argument that made units types applies equally to frames.

### 6.2 `adcs-core::physics`

Every relation a sheet or the plant uses lives here (rule 3). Keep `gnc.rs` and `mission.rs` from VLEO, and write the rest fresh in VLEO's style: `no_std`, typed arguments, `pmath` only, a doc comment giving the relation and its source id. Every function that takes a count returns zero at count zero (§5.6).

| Module | Functions (the seed row that calls each) | Source |
|---|---|---|
| `orbit` | `radius(h)` (m2_4) · `circular_period(r)` (m2_5) · `mean_motion(t_orb)` (m2_6) · `circular_speed(r)` (m3_4) · `eclipse_fraction(r, beta)` · `beta_angle(inc, ltan, epoch)` | vallado2013 |
| `env` | `max_magnetic_latitude(i)` (m3_2) · `dipole_field_equator(r)` (m3_0) · `dipole_field_max(r, lambda_max)` (m3_1) · `density_at(h, t_epoch)` (m3_3) · `solar_pressure(t_epoch)` (m3_5) · `sun_distance_au(t_epoch)` | wertz1978, igrf14, nrlmsis2, kopp2011 |
| `gnc` (kept) | VLEO's functions, signatures unchanged: `magnetic_torque(residual_dipole, field)` (gd_3) · `total_disturbance_torque(aerodynamic, gravity_gradient, solar, magnetic)` (gd_4) · `magnetorquer_dipole_required(momentum, field, dump_time)` (gm_2) · `pointing_error_rss(terms: &[Angle])` (gp_5, called with one slice) | wertz1978, smad2011 |
| `gnc` (added) | `gravity_gradient_torque_worst(r, i_max, i_min)` (gd_0; typed inertias, θ = 45°) · `aero_torque(rho, v, c_d, a_fr, c_pa)` (gd_1) · `solar_pressure_torque_at(p_srp, a_sun, q, c_ps)` (gd_2; normal incidence) · `secular_momentum_per_orbit(tau_d, t_orb)` (gd_5). VLEO's `gravity_gradient_torque` and `solar_pressure_torque` stay as they are; their signatures do not fit these rows. | wertz1978, smad2011 |
| `mtq` | `dipole_to_reject(tau_d, b_min)` (gm_1) · `torque_authority(m_av, b_min, n_mtq)` (gm_3) · `coil_dipole(n_turns, current, area)` | smad2011, idmas_v2 |
| `rw` | `bang_bang_peak_momentum(i_max, theta, t_slew)` (gw_3) · `bang_bang_peak_torque(i_max, theta, t_slew)` (gw_4) · `cyclic_momentum_quarter_orbit(tau, t_orb)` | idmas_v2, smad2011 |
| `fmr` | `ring_momentum(d, s, rho, v)` (gf_6) · `spin_down_time(d, rho, mu)` (gf_7) · `holding_power_laminar(h, mu, l, rho, d, s)` (gf_8) · `pump_pressure_for_torque(tau, l, s, d)` (gf_9) · `reynolds(rho, v, d, mu)` · `conduction_pump_pressure(n, i, b, h)` | idmas_v2 |
| `rcs` | `propellant_per_slew(h, r, isp)` (gr_3) · `propellant_per_year(m_p, n_day, n_rcs)` (gr_4; zero when `n_rcs` is zero) | idmas_v2 |
| `ctl` | `settling_time_2pct(w_n, zeta)` (gc_2) | ogata2010 |
| `risk` | `highest_level(levels)` (rk4_0; a slice, the highest open level over the areas) · `net_closed(closed, opened)` (rk4_1) · `share_tested(untested, total)` (rk4_2; zero beliefs recorded is `Undefined`, not 1) | ecss_m_st_80c, adcs_derisk_method |
| `mission` (kept) | `Sense`, `Closure`, `closure(req, ach, sense)`, exactly as in VLEO | — |

`plan/physics.toml` lists every function here with its arguments and the rows that call it; `tools/validate_plan.py` checks that the file and this table name the same functions. That file is the package's registry. From P1 the registry is `adcs-core::physics` itself: `cargo xtask docs` writes a generated table of its public functions, their arguments and their source ids, which `adcs-intake` checks steps against and the form exporter puts in the node form's pick-list. A new function the implementation agent writes therefore appears in the next release's forms with nothing else to edit, and in the twin map's physics family, so the `twin` job asks for its MATLAB twin in the same pull request (§10.8.7). Argument names are the calling rows' binding names, and argument order is their input order. A node form's step names one of these functions, and the implementation agent writes that step's HOLE as exactly `physics::<module>::<fn>(<bindings in order>)`; `intake verify` checks the call.

A kept VLEO function is never changed. Where a node needs something different, its form describes a new function, as the four `gnc` additions above were, and the implementation agent writes it here, in this style, with property tests and its own review.

### 6.3 Reference-data bundles

Each is a VLEO bundle: `bundles/<name>/<version>/` holding a `manifest.toml` (name, version, provenance, `licence_until`, `stale_after_days`, files, `content_hash`), published with `xtask bundle publish` and verified at sync.

| Bundle | Contents | Built by | Read by |
|---|---|---|---|
| `igrf14` | the IGRF-14 Gauss coefficients file from NOAA NCEI (`igrf14coeffs.txt`), unmodified | a person downloads it; `xtask bundle publish` hashes it | `env::*` (the degree-1 terms), `adcs-sim-core` field model (full degree 13) |
| `atmos-density` | a table of log density against altitude, 150–2000 km, at low, mean and high solar activity | `tools/atmos_table.py`, running NRLMSIS 2.0 through the `pymsis` package (ADOPTION.lock) | `env::density_at`, the plant's drag model |
| `catalogue` | every part, product, algorithm and class file and `families.toml`, as published | `xtask bundle publish catalogue/` | solver, loop engine, FSW config generator |

**How bundle data reaches a physics function.** Never as a hole argument. VLEO compiles its measured solar data into `physics/env.rs` as constants, marked "MEASURED DATA, not a published relation". A node's `[data] bundles` makes a run refuse when that bundle is missing from `Case.data`. The ADCS kernel does the same, generated rather than typed:

- `adcs-core/build.rs` reads `bundles/igrf14/<version>/` and `bundles/atmos-density/<version>/` at build time, and emits `const` tables: the IGRF-14 degree-1 terms, and the density table;
- the build records each bundle's content hash as a constant in the kernel;
- at run time, a node that lists the bundle refuses with `DataUnverified` (F8) when the store's verified hash differs from the compiled one, and with `DataMissing` when the store lacks the bundle.

So the kernel stays `no_std` with no files, the data is reviewed as a bundle, and the numbers a run used are the numbers the kernel hash names. The dipole strength is taken at the IGRF-14 model epoch 2025.0, which is an assumption stated on `m3_0`.

The IGRF coefficients file is fetched by a person and recorded with its URL and date in the manifest's `provenance`. The builder does not fetch data at build time; the network rule of VLEO's DELIVERY_PLAN holds, so a campaign makes zero network calls.

The field model in `adcs-sim-core` is a spherical-harmonic synthesis to degree 13 with Schmidt semi-normalised Legendre functions, in `pmath`. Its fixtures come from NOAA's own IGRF calculator, recorded by a person (provenance `independent-tool`), never from this code.

---

## 7. The catalogue and the module descriptor standard

### 7.1 One record, five readers

IDMAS v2 §11 makes every module self-describing: its EEPROM holds geometry, calibration and limits, and the controller builds its allocation matrices from them at boot. §14 leaves the "module descriptor standard" open. `catalogue/schema.toml` is that standard. The same record is read by:

1. the solver, to decide which parts fit and what they cost the bus (§8);
2. the closure engine, as `Case` supplies on declared rows (§8.4);
3. the loop engine, as plant, sensor and actuator parameters;
4. the flight software config generator, as the allocation matrices A and D and the limits;
5. the EEPROM image written into the flight module at calibration.

A part therefore cannot mean one thing to the quote and another to the rig.

### 7.2 Files

The catalogue has two levels: **parts**, the hardware, and **products**, configurations built from parts with the algorithms they fly. Classes and algorithms sit beside them.

- `catalogue/schema.toml`: the module kinds and their required fields (12 kinds).
- `catalogue/parts/<part_number>.toml`: one file per part.
- `catalogue/families.toml`: the four families (§2), each with its hardware `tags` and its **slots**. A slot is one function the configuration must fill (coils, rings, magnetometer, …). It lists the module kinds and parts that may fill it, and the count row that says how many are fitted. Two parts that do the same job, such as a panel ring or a body ring, are alternatives in one slot, never two slots.
- `catalogue/products/<id>.toml`: one file per product (§7.7).
- `catalogue/algorithms/<id>.toml`: one file per flight algorithm, with its tunable parameters (§7.8).
- `catalogue/classes.toml` and `catalogue/classes/<class>.csv`: satellite classes and their standard cases (§7.9).
- `bundles/catalogue/<version>/`: the published, hashed copy, the only one evidence, quotes and client runs may use. Developers may point the engine at the working copy (`--catalogue catalogue/`). Every manifest made that way says `catalogue = "unpublished"`, and CI refuses such a manifest in an evidence package or a quote. So P2 and P3 run before D7 settles the bundle's boundaries.
- per-serial descriptors: one per built unit, recorded at calibration as operational data in the portal, never in the repository (§7.4).

### 7.3 The rules

- **SI, and the key names the unit**: `mass_kg`, `h_max_Nms`, `bore_m`.
- **`nan` means not yet measured or not yet chosen.** A run that needs a `nan` field is refused by name ("IDM-BR-150 has no enclosed_area_m2; it is measured at calibration"). TOML reads `nan`, and `tools/validate_plan.py` accepts it. Today several IDMAS fields are `nan`, and `scenarios/slew_150kg_v3.toml` says its first expected result is exactly that refusal.
- **`status` decides who may use a part:** `synthetic` for software tests and the machinery pilot only, named `SYN-*`, allowed in design runs and internal results (marked as synthetic), and never in an `offered` product, a quote or evidence; `placeholder` for a real part not yet specified; `reference` for values from our own design documents, quotable with "not yet qualified" on the quote line; then `qualified` and `flight-proven`. CI refuses an evidence package or a certificate (§14) whose campaigns flew a synthetic or placeholder part. Such a campaign may still supply an evidence row, where InputPedigree 0 says what it is worth (§5.5).
- **`families`** lists where a part may appear. The solver never places it anywhere else.
- **`[dispersion]`** gives each Monte Carlo distribution, per `schema.toml`'s `dispersions` for the kind. A `nan` sigma is a refusal like any other.

The package has thirteen parts. Six are IDMAS parts: coil tile, 3U magneto-fluidic panel, 150 kg body ring, 150 kg torque rod, RCS module and controller. Their values come from IDMAS v2, with `nan` where the document gives none. Seven are synthetic:

- `SYN-CT-1`, a PWM-driven coil, and `SYN-MFP-1`, a twin of the 3U panel with its geometry and fluid, stated round electrical values, and the 0.10 pump efficiency IDMAS v2 §13 itself assumed. These let the whole machinery run before the bench measures the real parts.
- A wheel, magnetometer, sun sensor, star tracker and gyro, which let every family run in tests.

A model reads only the descriptor fields it needs, and declares which. A `nan` in a field no model of the run reads refuses nothing: the controller's `nan` CPU fields do not stop a SILS run, which does not model the CPU. The in-house star tracker replaces `SYN-ST-1` when its head is characterised. Every bought part replaces its synthetic stand-in when a supplier is chosen, and the change is two reviewers (§17.3).

### 7.4 Per-serial descriptors

At calibration (IDMAS v2 §06.1: "measure dipole per amp, pump torque per amp and the friction curve; store them in the module EEPROM"), production records the unit's descriptor, `<part>/<serial>.toml`. It is the part's record with `[nominal]` replaced by measured values, plus the block below. A serial is operational data, like a case: production uploads it on the order's production page in the portal, where it is checked against `catalogue/schema.toml` on upload and stored with the order (§15.4 `unit`). The repository holds none, and no release is needed to record one.

```toml
[calibration]
by = "<a person>"                 # never an agent
date = "YYYY-MM-DD"
rig = "<rig id>"
campaign_hash = "<the calibration campaign's manifest hash>"
measured = ["dipole_per_amp_Am2_per_A", "friction_curve", "pump_efficiency"]
friction_curve = [[0.0, 0.0], [0.1, 1.2e-6], [0.2, 2.6e-6]]   # [v m/s, N.m] — example shape only
```

The solver reads parts and products. The rig, the EEPROM image and the as-built twin (§14.4) read serials.

### 7.5 What is generated from descriptors

| Artefact | Generator | Format |
|---|---|---|
| Case supplies for the tree | `adcs-config` (§8.4) | `(node id, SI value)` pairs, with the part number in the case note |
| Plant parameters | `adcs-sim` scenario loader | in memory, hashed into the run manifest |
| Flight software configuration | `adcs fswcfg --case <c> --serials <dir>` | `adcs-fswcfg/1` (below) |
| EEPROM image per module | `adcs eeprom --serial <file>` | `adcs-eeprom/1`: part number, serial, descriptor hash, calibrated values, CRC-32 |

`adcs-fswcfg/1` is a little-endian binary with magic `AFCF`, a `u16` version, a `u32` length, a TLV body and a trailing CRC-32 (IEEE 802.3). The sections are:

- `0x01` actuator table: kind, axis unit vector, scale, limits;
- `0x02` allocation matrices D (3 × coils) and A (3 × rings), IDMAS v2 §12.1;
- `0x03` sensor table: kind, mounting quaternion, noise parameters;
- `0x04` mode parameters;
- `0x05` controller gains.

- `0x06` tuned parameters: one record per tuned parameter, `(algorithm id: u16, parameter id: u16, value: f64 SI)`. The ids are each algorithm's `number` and each parameter's `number` in `catalogue/algorithms/*.toml`. They are assigned once and never reused or renumbered, so adding an algorithm never changes an existing flight build's ids. `tools/validate_plan.py` refuses a reused number. `fsw/include/adcs_params.h`, `adcs-catalogue`'s table and the twin's `data/param_ids.json` are all generated from those numbers. The flight software reads a parameter by id and uses its compiled-in default when a record is absent.

`adcs fswcfg --case <c> --product <p> --tuned <hash> [--serials <dir>]` writes it. A product's `algorithms` list selects which of the reference flight software's modes are enabled, by the `modes` each algorithm declares. `adcs_fsw_init` receives it (`fsw/include/adcs_fsw.h`). Sections `0x05` and `0x06` are restricted (D1): never shown to a client and never exported in an evidence package.

### 7.6 Publishing

`xtask bundle publish catalogue/` writes `bundles/catalogue/<YYYY.MM.DD>/` with its manifest and content hash. Publishing is irreversible and needs two reviewers (VLEO CONTRIBUTING). A quote records the catalogue version it was made against, and a new version never changes an old quote (§15.7).

### 7.7 Products

A product is what a client buys and what the solver offers: `adcs-product/1`.

| Field | Meaning |
|---|---|
| `family` | one of the four families (§2) |
| `classes` | the satellite classes it was designed for (§7.9) |
| `status` | `candidate` (saved, internal only), `offered` (a person promoted it, H14), `retired` (kept so old quotes still resolve) |
| `origin` | `seeded` (delivered with this package) or `designed` (found by `adcs design`, then added to the catalogue by the developer team on request, §8.6) |
| `algorithms` | drawn from the family's `algorithms`, always including `bdot` |
| `counts` | every count row of the family. The counts of filled slots are at least 1; the counts of empty slots are 0. |
| `[[fill]]` | one part per filled slot, from the slot's allowed parts |
| `[[mount]]` | where each part sits: `place`, and `normal_body`, `boresight_body` or `position_body_m` |
| `mounts_pending` | a reason, when not every part has a mount yet; a run then refuses, naming the unmounted parts |
| `[design]` | for a designed product: `case_hash` it was designed against, `campaign_hashes` that proved it, the `envelope` it held across, and its `worst_margin` |
| `[promotion]` | `by` and `date`: a person, never an agent |

Tuned parameter values are never stored in a product. They belong to one case, live in the run ledger, and are restricted (D1). A product holding any `SYN-*` part is shown as synthetic everywhere; that is derived from its parts, never a field.

A count whose slot has `count_from` (the RCS module's `thrusters`) comes from the part filling the slot, so a product does not set it.

The package seeds five products, all `candidate`:

- `SYN-P-3U-MTQ` and `SYN-P-3U-FMR`: synthetic, for the detumble and machinery pilots;
- `IDM-P-3U-FMR`: the finalised product on its real parts, refused until the bench measures them;
- `IDM-P-150-V3`: IDMAS V3, refused until its parts are specified and it has mounts;
- `SYN-P-12U-RW`: synthetic, the wheel family, with no mounts yet.

None can be offered until D18 replaces the synthetic sensors, and the in-house controller `IDM-CTRL-1`, which every family requires, is specified and promoted from `placeholder` (H14).

### 7.8 Algorithms

An algorithm is `adcs-algorithm/1`. It declares:

- the flight-software modes it implements;
- the families it serves;
- its source;
- its tunable parameters. Each has a unit, `lo` and `hi`, and a `lin` or `log` scale. It may name the `tuned` tree row it sets. It may be limited to some of the algorithm's families;
- `prototype`: `true` while only the MATLAB twin implements it (§10.8.5). No product may carry a prototype algorithm.

A bound is a number, or `"part:<slot>.<field>"`, read from the nominal descriptor of the part filling that slot, never a serial or a dispersed value: the ring cruise speed's upper bound is the ring's `v_max_m_s`. The tuner clips every box to the domain of the tree row the parameter sets. A log-scaled parameter's midpoint is the geometric mean of its bounds. Between them, a family's algorithms must tune every `tuned` row in play for the family, and `tools/validate_plan.py` checks that.

The four seeded algorithms are:

- `bdot`: every family, because the coils own detumble and safe mode;
- `mekf`;
- `pd_alloc`: wheels;
- `idmas_split`: the IDMAS split projection. With no rings fitted it is the magnetic-only law of IDMAS V1, so the coils-only family carries it too.

Their bounds are design decisions marked `UNCONFIRMED`. A product flying an algorithm with unconfirmed bounds cannot be offered.

### 7.9 Satellite classes

`catalogue/classes.toml` lists the classes the design team designs for:

- the CubeSat classes, with the CubeSat Design Specification's mass limits (Rev. 14.1, Table 1);
- small-satellite classes around 50, 150 and 500 kg, whose ranges are D21.

A class's `standards` are case CSVs in the same fixed format, one per requirement set the class is designed for. Each holds the class's standard inputs, the `lo`/`hi` ranges a design must hold across, and the standard requirements. The 3U class has two: coarse pointing (`cubesat_3u_ais.csv`) and fine pointing (`cubesat_3u_img.csv`), which carry the two default cases' values until D21 sets the class's own. A design job (`designs/*.toml`) names a class, one of its standards, and the families to sweep (§8.6).


---

## 8. Cases, the solver and the designer

### 8.1 The idea in one paragraph

Nothing about a customer is ever written into the tree or the code. A customer's whole statement arrives as **one case**: one CSV file in a fixed format, uploaded once. The importer checks it, and every value goes on its own to wherever it is needed: the tree's declared and requirement rows, the scenarios' orbit, epoch and initial conditions, and the campaigns' dispersions. The **solver** then searches the catalogue. A catalogue **product** is a configuration (which part fills each slot, how many, where they are mounted) plus the flight algorithms it carries. For each product the solver tunes the algorithms in SILS against this case's requirements. If a product meets every written requirement, it is selected. If none does, the client is told so, with the gap. For the design team, the same engine runs as the **designer**. It sweeps part combinations against a satellite class's standard case, tunes each one, and saves every combination that meets the class requirements as a new candidate product. So the catalogue grows from the design team's work, and the next case of that class finds a product without anyone designing from scratch.

```
case CSV ──import──▶ case (every value placed; blanks listed)
                          │
      ┌───────────────────┴────────────────────┐
  client mode: adcs solve                 designer mode: adcs design
  offered products of the case's class    every part combination the families allow
      │                                        │
  screen: analysis closures (tree, fast)   screen: analysis closures
  tune:   algorithms in SILS, per case     tune:   algorithms in SILS, per combination
  confirm: Monte Carlo at the case's level confirm: Monte Carlo at the class's level
      │                                        │
  meets every written requirement?        meets every written requirement?
    yes → selected → quote                  yes → saved as a candidate (data, in the store)
    no  → "no catalogue product meets        → added to the catalogue on request (developer team)
           this case", with the gap            → promoted to offered by people (H14)
           → design request (internal)         → the next case finds it
```

### 8.2 One supplier for every value

Every declared row on layer 2 has exactly one supplier, and `plan/case_inputs.toml` records which:

| Supplier | Rows | How the value arrives |
|---|---|---|
| `case` | 32 inputs (mission, orbit, mass properties, surfaces, magnetic cleanliness, flexible modes, resources offered, thermal distortion) and the 22 requirement rows | the case CSV, row by row |
| `product` | 24 rows: the six counts, sensor noise and alignment, coil dipole, wheel capacity, ring geometry and fluid, thruster values | the product's parts, through `adcs-config`'s supply map (§8.4) |
| `tuned` | 5 rows: estimator rate, ring cruise speed, control bandwidth, damping ratio, control loop rate | the product's algorithms, tuned per case (§8.5) |
| `evidence` | 9 verification rows | the evidence tooling, from finished campaigns (§14) |

On layer 1, the 18 declared rows of Risk management have one supplier too, `derisk`: the de-risking ledger, counted by `cargo xtask derisk rollup` at each release (§5.13). `plan/case_inputs.toml` records it with the others, and intake refuses a form that sets one (V01).

The 42 measured facility rows of layer 1 (the OILS rig, the field cage, the air bearing, the Sun and star stimulators, the test stands, safety and power) have one supplier as well, `lab`: the lab file named with the run, `rig/labs/<lab>.toml`. Each supplier entry names the file's `field` (a dotted path, such as `helmholtz_cage.field_range_T`) and, for a list, a `reduce` of `max` or `len` (the strongest of three axes, the number of interface kinds). A `nan` in the file makes the row answer `NotMeasured`, naming the field; so does a `nan` anywhere in a list reduced with `max` (never the largest of the rest), and a `nan` where a list is expected (the list is written when the thing it lists is installed and shown working). The facility's three policy rows (OILS rig hours per week, the real-time plant step and the deadline misses allowed per hour) are not measured: they are declared, through node forms, like any declared row. So a measured capability is written once, in the lab file, from a person's bring-up record, and the tree and `adcs rig fit` (§12.10) both read it there. `tools/validate_plan.py` checks that every lab file has every field the suppliers name, and intake refuses a form that sets one (V01).

`tools/build_tree.py` holds the supplier lists and refuses to build a tree in which a declared leaf has no supplier or has two.

**What a row answers when its supplier gives nothing:**
- a `case` row: `NotStated`, naming the CSV key (§8.3);
- a `product` or `tuned` row in a candidate run: `NotFitted`, naming the empty slot or the parameter no algorithm sets (§5.6);
- a `lab` row whose field is `nan`, or a run with no lab named: `NotMeasured`, naming the lab file and the field;
- the sheet's reference value, only in a run with no product at all, such as P1's tree runs with `--case ais_img_3u`. They say so.

The layer-1 per-case rows (product selected, cost & price, acceptance criteria) are the portal's to fill from the solution and the quote. They are the company's view, and no engine result reads them.

**InputPedigree of supplied values:**

| Value | InputPedigree |
|---|---|
| a case value the sender stated | 2 |
| a case value whose note begins `UNCONFIRMED` (a stand-in the sender has not confirmed) | 1 |
| an assumed default | 1 |
| a product value | the part's status score (synthetic 0, placeholder 0, reference 2, qualified 3, flight-proven 4) |
| a tuned value | the lowest pedigree of what it was tuned on |

These replace the sheet's UNCONFIRMED score for that row in that run.

The `evidence` rows (`va` to `dsh`) are ordinary supplies that `adcs-evidence` computes from the ledger: counts of requirements verified per rung, run counts and differences. They are not `EvidenceSupply`, which only targets achieved rows. Before P8 they answer `NotStated`. `tools/validate_plan.py` checks the same from the generated file. That is what "automatically distributed wherever needed" means in the code: there is no value without a known source, and no value with two sources.

Scenarios read the case in the same way. Any scenario field may be written `"case:<key>"` (§10.1), and the orbit and epoch must be. A scenario never restates a number the case owns.

### 8.3 The case: `adcs-case/1`, its editor and its checker

The case is the only input a team member or a client edits (§1.6). Everything about it is designed so that a person can write one correctly without help, and so that the software reads it one way everywhere.

#### 8.3.1 The format

`plan/case_template.csv` is the blank template, and `plan/case_inputs.toml` is its registry. `tools/build_tree.py` generates both from the tree, so the format cannot drift from the rows it feeds. The columns are fixed:

| Column | Meaning |
|---|---|
| `section` | `meta`, `req`, `mission`, `orbit`, `mass`, `surface`, `magnetic`, `flex`, `resources`, `pointing` |
| `key` | the stable key, such as `orbit.alt` or `req.ape` |
| `label`, `unit` | as the template prints them. The importer refuses a file whose label or unit differs, so a value in the wrong unit is refused rather than converted. |
| `value` | a number; blank means not stated |
| `lo`, `hi` | optional, both or neither, only on inputs that take a range: what a Monte Carlo draws from and an edge campaign visits |
| `level` | requirements only: the ensemble probability in percent at which the requirement is stated (ECSS-E-ST-60-10C) |
| `note` | free text: where the number came from. A note beginning `UNCONFIRMED` marks a stand-in value the sender has not confirmed. The importer lists every such value, and its InputPedigree is 1 (§8.2). |

Every key appears once, in the template's order. Unknown keys and missing keys are refused. The five `meta` rows are `schema` (always `adcs-case/1`), `case_id`, `title`, `class` (optional; a class in `catalogue/classes.toml`) and `families` (optional; limits the search).

**The template explains itself.** The blank template's `note` column says, for every key, what it means in the words a customer uses ("largest angle allowed between where the payload axis points and where it should point, at the level given"; "body rate just after separation from the launcher; give lo and hi if the launcher states a range"). The text is `CASE_HELP` in `tools/build_tree.py`: it never suggests a value. The same text is shown beside each row in the case editor and listed in the user manual (`manual/user/05_case_keys.md`, generated).

#### 8.3.2 Blanks are never guessed

Each input has a blank policy:

- `stated`: a blank leaves the row **unstated**. The importer writes it into the case's `unstated` list. The row then answers `Fault::NotStated { key }`, never its sheet's reference value, and every row, closure and scenario field that reads it is blocked, naming the CSV key.
- `default`: a blank takes the sheet's reference value (eccentricity 0, drag coefficient 2.2, reflectivity 0.6). The case report lists it as an assumption the case made, and the client sees the list.

A blank requirement is an unwritten requirement: the requirement row answers `NotStated` for this case, even where its sheet carries a reference value, and both its closures say "not stated by this case". A written requirement with a blank level takes 99.73 %, and that is listed as an assumption too.

#### 8.3.3 The case editor

`forms/case_editor.html`, `adcs-case-editor/1`, is how a person who never opens a spreadsheet writes a case. It is one offline HTML file, in the node library and the MATLAB zip, and served in the portal and the web app, where it is the on-screen case form. It follows the explanation standard (§5.12): a breadcrumb; one answer-first sentence, kept current as the person types, saying whether the format is right, how many requirements are written, how many inputs are unstated and which scenarios they block; "What a case is" as the overview; and "What happens to a blank, a range and a level" as a station, with two rows that differ in one thing (`surface.cd` blank takes the reference value; `mass.m` blank blocks), the common wrong idea that a blank is filled sensibly, where the page stops being reliable, and a prediction to try in Learn. It also:

- starts from any reference case (the four in `plan/cases/` are embedded), from a CSV the person already has ("Open a CSV…", which refuses a file whose header, keys or units are not the format's, naming each), or from blank;
- shows the case section by section, each with a line saying what the section is for, and every row with its meaning, its unit, what a blank does ("blank → unstated; blocks what needs it, by name" or "blank → reference value, listed as assumed"), and a requirement's sense in words ("at most — the achieved value must be at or under this");
- offers `lo`, `hi` and `level` only on the rows that take them;
- checks each row as it is typed, with the case checker's rules, and marks the section in the side navigation;
- has "Is it ready?": requirements written, inputs stated and unstated, blanks that take the reference value, requirements taking the default level, values marked `UNCONFIRMED`, and for every scenario of the release whether this case can run it and which of its requirements the scenario judges;
- downloads the CSV in the fixed format, one row per key in the template's order ("Download CSV"), and keeps an editor copy with the values in it ("Save editor copy");
- keeps unsaved values in the browser, offered back on reload, as a convenience only.

`cargo xtask form export --case [<case.csv>]` (`tools/forms.py case` in the package) writes it, blank or filled.

#### 8.3.4 The case checker

`adcs case check <file.csv>...` (`tools/check_case.py` in the package) prints, for each file:

1. **the format**: every rule of §8.3.1. Any failure here and the software refuses the file, by line and key;
2. **what the case says**: requirements written, inputs stated, inputs left unstated (each blocks what needs it, by name), blanks that take the reference value, requirements taking the default level, values marked `UNCONFIRMED`;
3. **what can run**: for every scenario in the release, whether the case states every key it reads — its own `case:` references plus what the plant reads for its environment switches (§10.1) — and which requirements it judges.

A blank is not an error: nothing is guessed, and the report says what it blocks. Every face runs the same check before it runs anything, and the portal shows it as the case report.

#### 8.3.5 Import and the case store

`adcs case import <file.csv>` is `adcs-case::import`. It:

- checks the file (§8.3.4) and refuses it on any format error;
- converts every value to SI, keyed by node id;
- writes the case in VLEO's case format, extended with `unstated`, `assumed`, `range` and `level`, into the **case store**: `~/.adcs/store/cases/<case id>/<sha12>.csv` for the CLI, the workbench and the MATLAB tool (the same layout as the results store, §13.5.4), `cases/` in the repository for the reference cases, and the portal's artefact directory, indexed by tenant and project. A case is stored by the hash of its CSV, and `--case <id>` resolves through the store's index, never through compiled code (F16). Case ids are unique within a tenant;
- hashes the case.

The importer maps tree ids to node ids, and reads sheet reference values for `default` inputs, through `adcs-modules`' generated tables.

It prints the **distribution report**:

- each key and the tree row it went to;
- the scenario fields that read it;
- the closures it unblocks;
- the assumptions taken;
- every blank, with what it blocks.

`adcs case import` takes one or more files. `adcs case export <case>` writes the canonical CSV back; import then export is byte-identical for a canonical file. `adcs case template` prints the blank template.

A case, once hashed, is never rewritten. A changed case is a new case with a new hash, which is how a quote stays frozen (§15.7) and how every result stays beside exactly the case it ran (§13.5.4).

The engine extension is `Case.unstated`, `Case.assumed`, `Case.range` and `Case.level`, beside `supply` and `evidence` (§5.5), plus the `NotStated`, `NotFitted` and `NotMeasured` answers. It is an H7 change, so two reviewers.

#### 8.3.6 Changing the format

The format is the software's, so changing it is a developer change, asked for like any other: a team member sends "something else" in a node form. The change is any change to `plan/case_template.csv`: `CASE_INPUTS`, `CASE_HELP`, `REQ_UNITS`, `META` or `CASE_SCHEMA` in `tools/build_tree.py`, or a tree label an input row carries. `tools/build_tree.py --check` shows it as a changed template. Anything but help text needs two reviewers and a version bump to `adcs-case/2`, with a migration in `adcs-case` that reads every earlier version. The intake checker refuses a node form that would change a case-fed node's label, quantity or unit (I02), so the format cannot change by the side door.

**Acceptance.** `tools/form_browser_check.py`: the case editor loads `ais_img_3u`; one edit downloads a CSV that passes `tools/check_case.py` and differs from the reference in that row only; a CSV with a wrong unit is refused with a message naming the key and the unit; at 390 px there is no sideways scroll. `tools/check_case.py plan/cases/*.csv` reports `ref_c3_12u` able to run no scenario, naming every key each needs.

### 8.4 Screening: the closures, over products or combinations

This is the configurator of the first plan, kept as the solver's first stage. It is the closure engine run over candidate cases and adds no physics of its own. A candidate is the case plus one product, or plus one part combination in designer mode.

1. **Candidates.**
   - Client mode takes every `offered` product of the case's class, then of the other classes whose family serves the case's mass.
   - Designer mode enumerates, per family, one part per slot and a count within the family's range for each counted slot: the cartesian product. A missing required slot is refused by name, and coils are required in every family.
   - `meta.families` narrows both.
2. **Supplies.** The candidate's parts become `Case.supply` entries on the `product` rows:
   - the counts of the filled slots are supplied;
   - a `count_from` count is read from its part (the RCS module's `thrusters`);
   - every other count is supplied as zero, which is how absence stays a count of zero (§2);
   - the per-unit rows of an empty slot answer `NotFitted` (§5.6);
   - `mav` comes from the part in the coils slot. A panel's own coils are the rings slot's.

   Where no descriptor field feeds a row (`ali`, sensor alignment, today), the row answers `NotStated` naming the missing field until `schema.toml` gains it. The mapping lives in `adcs-config/src/supply_map.rs` and is reviewed like a sheet: a wrong line there is a wrong number everywhere. A `nan` field becomes no supply, so every closure that reads it is blocked, naming the part and the field.
3. **Evaluate** every analysis closure (§5.5) in `Branch` mode, with the `tuned` rows at the midpoints of their algorithms' bounds (the geometric mean on a log scale). A closure that fails at the midpoint is evaluated again at every corner of the box of the tuned rows it reads. That is at most 2⁵ corners, since five rows are tuned. It fails only if it fails at every corner, so screening never discards a candidate that tuning could rescue. Record every margin, every refusal and every governing factor.
4. **Classify** the candidate:
   - *feasible* when no analysis closure fails;
   - *incomplete* when none fails but some are blocked, with the blocked ones listed;
   - *infeasible* when any closure fails, with the failing closures listed. It is not tuned, which prunes the designer's sweep cheaply.

   Evidence-only KPIs never count against a candidate; they are what the next stage is for.

### 8.5 Tuning in SILS: `adcs-tune`

Tuning is the "changeable inputs for each product and combination". Each algorithm in `catalogue/algorithms/` declares its parameters, their units, bounds and scale, and the tree row each one sets, if any (§7.8). For one candidate:

- **The search.** The search space is the box made of every parameter of the product's algorithms that is in play for its family and belongs to an algorithm whose modes the scenarios actually run. B-dot's gain is not searched in an inertial-hold scenario. The search is derivative-free and deterministic: a seeded pattern search from the box centre, at most `evaluations` runs (`designs/*.toml`, or `--evaluations` for a case). Log-scaled parameters are searched in log space. A rate's upper bound is clipped to the scenario's `fsw_rate_hz`, and the rate is rounded to an integer divisor of it.
- **Each evaluation.** Every scenario whose `families` include the candidate's family and which binds a requirement the case writes runs as a `nominal` campaign, plus edge corners. The corners take the case's `lo`/`hi` ranges and the scenario's `[model_errors]` at both ends of their ranges, never only their means. Tuning on one run would tune to that run, and a parameter could learn to cancel one known error value, such as `idmas_split`'s `feedforward_scale` against the scenario's feed-forward error.
- **The objective.** Maximise the worst normalised margin across every requirement the case writes: `(required − achieved)/|required|` for `<=`, and the reverse for `>=`.
  - A requirement a scenario binds takes its margin from that metric.
  - A requirement no scenario of the family binds takes its margin from its analysis closure.
  - A requirement with neither, or one verified by inspection (ADCS mass and volume, until a unit is weighed and measured), takes its analysis margin if it has one. Otherwise the candidate is *incomplete*, is never selected, and the requirement is listed. Ties break on lower orbit-average power, then lower peak power. D22 may add weights; until then there are none.
- **The result** is the tuned parameter set and its hash. The `tuned` tree rows are supplied from it. The flight-software parameters go into `adcs-fswcfg/1` section `0x06` (§7.5). The set is restricted (D1): the client sees that a product was tuned and the margins it reached, never the values.
- **Confirm.** The best set is confirmed by a Monte Carlo campaign on the same scenarios, dispersing the case's ranges, the parts' `[dispersion]` tables and the scenario's distributions. Pass means every bound metric meets its requirement at the case's level. That campaign supplies the evidence rows (§5.5) with rung SILS.

A tuning run that hits a bound is reported with that bound named, because the box, not the design, may be what limits it.

### 8.6 Select, or design

**Client mode, `adcs solve <case>`:**

- Products that pass are **selected**, ranked by worst margin, then ADCS mass, then orbit-average power. The solver never reads a price: the portal attaches prices from its operational data when it shows the shortlist (§15.7), so no database sits on the physics path.
- At most three go on the shortlist, each with its margins, its governing credibility, its evidence campaigns and its assumptions.
- If none passes, the answer is **"no catalogue product meets this case"**, with a gap report in two views:
  - the **client view**: per written requirement, the best margin any offered product reached, or "no product is offered for this class";
  - the **internal view**: the same with every product tried, candidates included, and the requirement that stopped each.

  The portal then offers a design request, which reaches the design team as an internal job naming the case (§15.5).

**Designer mode, `adcs design <job id>` (a design job of the release, such as `cubesat_3u_ais`), `adcs design <file.toml>` or `adcs design --case <case>`:**

- Every combination that passes is **saved as a candidate product**, with `status = "candidate"`, `origin = "designed"`, its class, and a `[design]` table: the case hash it was designed against, the campaign hashes that proved it, the envelope it held across (the case's `lo`/`hi`) and its worst margin.
- It is saved as **data, not code**: to the designer's local store (`~/.adcs/store/candidates/<id>.toml`, with a result document per design job, §13.5), or, for a portal design job, to the portal's `product` table. The software never writes to the repository's catalogue.
- At most `keep` are saved per family, best worst-margin first.
- The ids are `D-<class>-<family>-<n>`, numbered in order of saving. `adcs design --case <case>` uses the case's class, stated or inferred (`catalogue/classes.toml`: the narrowest class range that holds the case's mass), and refuses a case with no class.
- Saving is automatic. The candidate is visible internally at once: in the CLI (`adcs product list --status candidate`), the workbench and the portal's internal catalogue view. Client mode never sees it.

**Into the catalogue, then offered.** A candidate reaches the released catalogue in two steps, each a person's:

1. **Add it.** The design team sends a node form of type "something else", attaching the candidate file and its design job's result document. The developer team checks it (`tools/validate_plan.py` over the catalogue with the file added: slots, counts, mounts, algorithms, parts) and adds it to `catalogue/products/` as a `candidate` in the next release.
2. **Offer it.** Promotion to `offered` is H14 (§17.2): the product's owner and quality decide, and each records the decision themselves (§17.2, "Recording a person's decision"). `cargo xtask decision record <record>` copies their names and the date from that record into `[promotion]`; nobody types a name into the file. The developer team then publishes the next catalogue bundle. From then on client mode finds it. The product must hold no synthetic or placeholder part, have a design campaign behind it, and fly only algorithms whose bounds a person has confirmed. `tools/validate_plan.py` refuses an `offered` product that breaks any of these.

So the catalogue is the company's accumulated design work. A new case of a known class usually finds an offered product. A new kind of case produces a design request once, and a product for everyone after it. Layer 1's Catalogue branch tracks it (`mgt_products_class_coverage`, `mgt_design_runs_design_yield`, `mgt_case_matching_catalogue_hit_rate`).

### 8.7 Interface

| | Client mode | Designer mode |
|---|---|---|
| Library | `adcs_solve::solve(&Case, &SolveOptions, &Catalogue) -> Solution` | `adcs_solve::design(&DesignJob, &Catalogue) -> Vec<CandidateProduct>` |
| CLI | `adcs solve <case> [--families f;g] [--evaluations N] [--json]` | `adcs design cubesat_3u_img` (a job of the release) · `adcs design <file.toml>` · `adcs design --case <case> [--families f;g]` |
| Daemon | `POST /v1/solve` | `POST /v1/design` |
| Portal | `POST /api/v1/projects/{id}/solve` (§15.5) | internal only: `POST /api/v1/internal/design` |

`adcs configure --case <c> [--families f;g]` stays as the screening stage alone (§8.4), for engineers.

`Solution` carries:

- per product: the margins after tuning, the tuned-set hash (never the values), the confirming campaign hash, the governing factors, refusals and assumptions;
- the gap report when nothing passes.

`CandidateProduct` is the product file plus its campaign hashes.

### 8.8 What a client sees

The client sees:

- the import report;
- per shortlisted product, its margins, refusals in plain words ("the coils cannot reject the aerodynamic torque at 250 km: 38 % short"), its governing credibility, and its SILS campaigns with visualisation;
- or, when nothing passes, the gap report and the design-request button.

Never shown: restricted rows (D1), tuned values, candidate products, other products' prices, and synthetic or placeholder parts.

### 8.9 Acceptance

The pilot thread is deliberately partial, so the acceptance is what the solver reports, recorded as observed in `docs/BUILD_EVIDENCE.md` and never asserted as a fixture.

**The two default cases.** Both are the same 3U satellite: 4 kg, 500 km sun-synchronous, the reference IDMAS V2 structure (source `adcs_ref_c1`). They differ in what they ask of the ADCS.

| | `ais_3u`: AIS only | `ais_img_3u`: AIS and imaging |
|---|---|---|
| APE | 10° (stated by the user) | 0.01°, 3σ (stated by the user) |
| AKE | 5° | 0.005°, 3σ |
| Rate stability | — | 0.001 °/s (3.6″/s), 3σ |
| Reference slew (30°), settling | — | 60 s, 20 s |
| Detumble from 10 °/s, sun acquisition | 284 min (three orbits), 95 min (one orbit) | the same |
| ADCS mass, average power, peak power, volume | 0.35 kg, 0.5 W, 1.5 W, 0.3 L | 1.0 kg, 2.0 W, 4.0 W, 0.6 L |

The two APE values are the user's. Every other value, including the satellite's, is a stand-in written for this plan. Its reason is in the CSV's `note` column, which begins `UNCONFIRMED`, so it carries InputPedigree 1 until a person confirms or replaces it.

For scale: 0.01° is at the edge of what a 3U has flown. MinXSS-1's wheel-and-star-tracker unit measured 0.0042°–0.0117° at 3σ, depending on the axis (arXiv:1706.06967). So the fine-pointing case is expected to test whether any family reaches it. Whatever the solver says is recorded, never assumed.

`ais_3u` is the faces' default case (§3.3). Its pointing is judged by `nadir_hold_3u`, which runs for every family, and its knowledge by `ge_5`, which answers with or without a star tracker. The two cases are also the 3U class's two standards (§7.9), and each has a design job (`designs/cubesat_3u_ais.toml`, `designs/cubesat_3u_img.toml`).

- `adcs case import plan/cases/ais_3u.csv` succeeds and prints its distribution report:
  - 27 values stated: 19 inputs and 8 requirements. 26 of them are `UNCONFIRMED` stand-ins and listed as such; only `req.ape` is the user's;
  - the 8 requirements' blank levels taken as the 99.73 % default, and listed as assumptions;
  - every other `stated` input listed as unstated, with what each blocks.
- `adcs case import plan/cases/ais_img_3u.csv` states 30 values: 19 inputs and 11 requirements, 3 of them with their own level.
- `plan/cases/ref_c3_12u.csv` imports with every `stated` input unstated and the three `default` inputs (`orbit.ecc`, `surface.refl`, `surface.cd`) assumed. It is a valid case that answers almost nothing and says why.
- `adcs configure --case ais_img_3u` screens four of the five seeded products, all of them `candidate`, since this is the engineer's tool:
  - `SYN-P-3U-FMR`, `IDM-P-3U-FMR` and `SYN-P-3U-MTQ` evaluate the rows the seed forms specified and are classed **incomplete**. The APE closure is blocked by `gp_0`, `gp_1`, `gp_2` and `gp_4`, which nobody has specified yet, and by `gp_3`, which is `NotStated` because the case leaves `pointing.et` blank;
  - `IDM-P-3U-FMR` screens exactly like its synthetic twin. The `nan` fields of its panels (resistance, time constant, pump efficiency, mass) are read by no analysis row, so they refuse nothing here (§7.3). They refuse it at SILS (P3);
  - `SYN-P-12U-RW`, a 12U product whose family still serves 4 kg, is screened after the products of the case's own class;
  - `IDM-P-150-V3` is skipped, because its family does not serve 4 kg, and it is named.
- `adcs solve ais_3u` and `adcs solve ais_img_3u` in client mode both return **"no catalogue product meets this case"**, because no product is `offered`. The client view says "no product is offered for class cubesat_3u". The internal view lists every candidate tried and why none can be offered: synthetic or placeholder parts (D18, `IDM-CTRL-1`). That is the honest state of the catalogue.
- `adcs design designs/cubesat_3u_ais.toml` and `designs/cubesat_3u_img.toml` (from P4) run on the internal parts and save any passing combination as a `D-cubesat_3u-*` candidate, marked synthetic. None of them can be promoted until D18. Which families pass each standard, including none, is the observation to record.

---

## 9. The loop engine — SILS

### 9.1 Three crates

| Crate | `no_std` | Holds |
|---|---|---|
| `adcs-sim-core` | yes, no alloc | the plant: state, dynamics, environment, sensor and actuator models, the RK4 step. Fixed-capacity arrays (at most 12 coils, 8 rings, 6 wheels, 16 thrusters, 16 sensors). It runs unchanged on the rig host, in the browser, and on an embedded target. |
| `adcs-sim` | no | the scenario loader, multirate scheduler, device emulators, recorder, metrics and campaign runner |
| `adcs-fsw-abi` | no | implements `adcs_hal.h` in Rust (`extern "C"`) over the device emulators; compiles and links the flight C code with the `cc` crate |

VLEO's kernel rule, "no files, no clock, no drawing", holds for `adcs-sim-core` without exception. Time is an input: `step(&mut State, &Inputs, dt) -> Outputs`, where `dt` is fixed per run. The clock belongs to the scheduler in `adcs-sim`, or to the real-time loop in `adcs-rig`, never to the plant. That is what lets one plant serve SILS faster than real time and OILS at exactly real time.

### 9.2 State and dynamics

State (all SI):

- the tick counter (`u64`); `t = tick · dt`, exact;
- orbit `r`, `v` in ECI, propagated with J2 and optional drag;
- attitude `q: Quat<Body, Eci>` and body rate `ω`;
- wheel speeds, ring flow speeds, ring fluid temperature and frozen flag, coil currents (first-order RL), thruster valve states, propellant mass;
- gyro bias random-walk states;
- star tracker availability.

Dynamics follow IDMAS v2 §12.1, extended with wheels. They are written so that total angular momentum is conserved: every torque between the body and a ring's fluid or a wheel's rotor is internal.

```
J ω̇ = −ω × (J ω + A_r h_r + A_w h_w) − A_r ḣ_r − A_w ḣ_w + (D_m m) × B + Σ_k r_k × f_k + τ_d
ḣ_r,i = τ_pump,i − τ_fric,i(h_r,i)        ḣ_w,j = τ_motor,j − τ_fric,j(Ω_j)        q̇ = ½ W(q) ω
```

`h_r` and `h_w` are the rings' fluid momenta and the wheels' rotor momenta about their axes, and the columns of `A_r` and `A_w` are those axes. Wall friction slows the fluid and turns the body by the same amount, so a ring's 0.75 s spin-down moves its momentum into the body; it does not lose it. IDMAS v2 §12.1's `−A u` is this `−A ḣ`, with `u` the commanded change of momentum.

RK4 at the scenario's `dynamics_step_s`. Discrete events (thruster pulses, valve edges, mode changes) happen only on tick boundaries. The quaternion is renormalised after every step, and its norm drift before renormalising is recorded as a diagnostic channel.

### 9.3 The scheduler

Every component (each sensor, the flight software, each actuator driver, the recorder) has a period that must be an integer multiple of the dynamics step. If it is not, the scenario is refused at load, naming the component. It is never rounded. At OILS and HILS the dynamics step is the rig's `plant_step_s` (§12.3), which replaces the scenario's `dynamics_step_s`. Every period must then be an integer multiple of the rig's step, and the run manifest records both steps. Within a tick the order is fixed:

1. environment at `t`;
2. sensors sample truth; device emulators queue their bytes with the sensor's latency;
3. the flight software steps, if this is its tick;
4. actuator emulators read what the HAL received and turn it into commands, with their latency;
5. the plant integrates to `t + dt`;
6. the recorder samples.

### 9.4 Models

The models are named here and specified in `adcs-sim-core` doc comments, each with its source. Each shared relation calls `adcs-core::physics` (rule 3).

| Area | Model |
|---|---|
| Magnetic field | IGRF-14 to degree 13 from the `igrf14` bundle; a tilted dipole as an option |
| Atmosphere | `atmos-density` table; activity from the scenario |
| Sun and eclipse | low-precision solar ephemeris (Vallado); conical eclipse |
| Disturbances | gravity gradient (full tensor); aerodynamic and solar pressure per face from `[geometry]` faces, defaulting to one plate from the case's scalars; residual dipole; stray field from coils and pump yokes at their mounts, felt by the magnetometer |
| Coil, rod | PWM duty to current through a first-order RL model; current limit; dipole per amp; quantisation; rod residual |
| Fluid ring | state `v`; `H = 2ρAvS`; conduction pump `Δp = nIB/h` or induction slip; laminar friction when Re < 2000 and turbulent above; spin-down; frozen below the melt point (no flow, no torque); electrical power = hydraulic ÷ efficiency; flow-EMF sensor noise; yoke stray field while pumping (IDMAS v2 §03, §07, §09) |
| Reaction wheel | torque and speed limits, Coulomb and viscous friction, motor lag, static and dynamic imbalance as jitter at the wheel's speed |
| RCS | pulses with a minimum impulse bit, on-time quantised to the flight software's rate, misalignment, propellant use, and the force on the orbit |
| Magnetometer | truth plus stray field, bias, scale, misalignment, noise, LSB quantisation, range saturation |
| Sun sensor | field of view, eclipse, noise, bias |
| Star tracker | availability from sun and Earth exclusion and the rate limit; noise across and about the boresight; latency |
| Gyro | angle random walk, rate random walk, scale factor, misalignment, quantisation |
| GNSS receiver | position and velocity noise, time to first fix, outages |

### 9.5 Device emulators: every port speaks bytes

A sensor model produces an engineering value. The device emulator turns it into exactly what the part puts on its port: an I2C register map, a UART frame, a CAN frame, an SPI response, a PWM measurement. The flight software's own drivers read those bytes through `adcs_hal.h`, so its drivers are exercised in SILS exactly as they will be on the OBC. That is why a SILS result is evidence about the flight code, not about a model of it.

Each device's protocol is data: `devices/<part_number>.toml` holds registers, scaling, byte order, framing, CRC, timing and error responses. The same file drives:

- the SILS emulator in `adcs-sim`;
- the OILS emulator in the interface emulation unit (§12.2);
- a golden-byte test that a person confirms against the part's interface control document.

The synthetic parts get invented but complete protocols, so the whole path is testable. A real part's protocol is entered from its ICD by a person, and the file names the ICD revision.

### 9.6 The flight software boundary

- `fsw/` is compiled by `adcs-fsw-abi/build.rs` with `cc`, flags `-std=c99 -O2 -ffp-contract=off -fno-fast-math`. With FMA contraction off, the C arithmetic is repeatable on one platform.
- Flight C code keeps state in statics, so one process holds one flight software instance. A campaign runs its runs in a pool of processes (`adcs-worker` or `adcs sim --jobs N`), never in threads.
- `adcs_fsw_init` must reset every static. A test runs the same scenario twice in one process and requires identical output bytes.
- A customer or company build is loaded as a shared library exporting the `adcs_fsw.h` symbols. The run manifest records `adcs_fsw_build_id()`.
- MATLAB-generated C (Embedded Coder) sits behind a thin wrapper that implements `adcs_fsw.h`. The design workflow stays: MATLAB first, then this SIL.
- The flight code's CI (§18) forbids `malloc`, `time()`, `rand()` and recursion. `cppcheck` runs, and a MISRA subset is checked where a free checker covers it.

### 9.7 Determinism

- `adcs-sim-core` uses `pmath` only. Iteration order is fixed (arrays and `BTreeMap`, never `HashMap`). There is no wall clock.
- Randomness is counter-based. The value for run *k*, parameter path *p*, draw *n* is `SplitMix64(hash(campaign_seed, k, p, n))`. It is independent of evaluation order and of how runs are split across processes.
- **Required:** `adcs-sim-core` trajectories are bit-identical across x86_64, aarch64 and wasm32. CI compares the hash of a golden trajectory.
- The flight C code is required to be repeatable on one platform. Across platforms it may differ, and PIL exists to measure that: the difference is a parity-ledger line, never a failure to hide.

### 9.8 The recorder

A run directory in `adcs-rec/1` format contains:

- `manifest.toml`: scenario hash, case hash, product id, tuned-set hash, catalogue version, flight software build id, engine hash, seed, rung, start and end, and every channel with its unit and rate;
- one CSV per channel group, chunked;
- `events.csv` for mode changes, faults and refusals;
- a content hash over all of it.

Four families of channels are kept apart and never mixed in one column: **truth**, **measured**, **estimated** and **commanded**. The views (§13) read a downsampled stream (at most 20 Hz) while the run is live, and the recorder's files afterwards. Parquet is deferred with the trigger VLEO already wrote: "the first bundle that does not fit comfortably as text".

### 9.9 Performance, measured and never assumed

The builder measures and records, in `docs/BUILD_EVIDENCE.md`, the wall-time ratio of `inertial_hold_3u` nominal: 11,400 s simulated at 0.01 s steps, one core. The builder also measures Monte Carlo runs per hour per core. The first measured figure is recorded beside the layer-1 row "Monte Carlo runs per hour", which is computed from the server cores and is checked against it. The requirement that this SIL beat MATLAB's Monte Carlo throughput on the same scenario is checked by measuring both, never asserted.

### 9.10 The reference flight software

`fsw/` ships a complete, deliberately plain flight software so SILS runs from day one. It covers:

- **Modes:** detumble (B-dot, with a magnetometer read window while the coils are off), sun acquisition, inertial hold, nadir, target track and slew.
- **Estimation:** a MEKF on gyro and star tracker, with a magnetometer-and-sun q-method fallback.
- **Control:**
  - the IDMAS split projection `m = B × τ / |B|²`, `u = −A⁺(τ − m × B)` (IDMAS v2 §12.5, L1);
  - B-dot (IDMAS v2 §12.5, L2);
  - quaternion PD;
  - momentum dump `m += k (h × B) / |B|²`, with the sign as corrected in IDMAS v2 §12.3;
  - the allocation QP of IDMAS v2 §12.2 as a fixed-iteration active-set solver.
- **FDIR:** a failed coil or ring becomes a removed column and the same QP is re-solved (IDMAS v2 §12.6).
- **Drivers:** for the synthetic parts' protocols.

It is C99 with no dynamic memory and unit tests. It is a reference, not the product: the company's flight software replaces it behind the two headers, and the scenarios keep working.

---

## 10. One scenario template, every campaign type, every rung

### 10.1 The template: `adcs-scenario/1`

A scenario is everything one run needs and nothing a campaign adds, and nothing the case or the product owns. The case supplies the satellite, the orbit, the epoch and the requirements. The product supplies the parts, counts, mounts and algorithms. So one scenario file is a test template that runs for any case and any product of its families: the solver runs it for every candidate it tunes (§8.5), and the rig runs it for the unit on the bench. The five files in `scenarios/` are complete examples:

- `detumble_3u`: B-dot on the PWM coil (`SYN-P-3U-MTQ`), the facility software's pilot, for every family;
- `nadir_hold_3u`: coarse nadir pointing for every family, the scenario the 3U AIS case is judged by;
- `inertial_hold_3u`: the IDMAS v2 §13 case on the synthetic twins (`SYN-P-3U-FMR`), the machinery pilot;
- `inertial_hold_3u_idmas`: the same on the real IDMAS parts (`IDM-P-3U-FMR`), refused until the bench measures them;
- `slew_150kg_v3`: IDMAS V3 (`IDM-P-150-V3`), refused until its parts are specified and its case states an orbit.

The schema:

| Table | Fields | Notes |
|---|---|---|
| top | `schema`, `id`, `label`, `case`, `families` | `case` is the default case: the imported case of that id, from `plan/cases/<id>.csv` for the reference cases. `--case` and the solver replace it; `families` are the families the template can verify |
| `[configuration]` | `product` | the default product, which `--product` and the solver replace; parts, counts and mounts live in the product (§7.7) |
| `[time]` | `epoch = "case:mission.epoch"`, `duration_s`, `dynamics_step_s`, `fsw_rate_hz` | every component's period must be an integer multiple of the dynamics step (§9.3); at OILS and HILS the rig's step replaces it |
| `[orbit]` | `altitude_km`, `inclination_deg`, `eccentricity`, `ltan_h` as `"case:orbit.*"`, and `propagator` | a scenario never restates the orbit; `tools/validate_plan.py` refuses a number there |
| `[initial]` | `attitude`, `rate_body_rad_s`, actuator states | each value is fixed; `{kind = "random", seed_stream = ...}`; or, for a rate, `{kind = "random_direction", magnitude_deg_s = ...}`. An actuator state is one scalar for every unit fitted (`ring_speed_m_s = 0.0`), never a list sized to one product. |
| `[environment]` | field model and degree, atmosphere and activity, each disturbance on or off, eclipse model | |
| `[geometry]` | optional faces (area, normal, centre-of-pressure offset, optical properties) | defaults to one plate from the case's surface rows |
| `[fsw]` | `build`, `mode_sequence` | `build` is `"reference"` or a flight software build id |
| `[model_errors]` | what the flight software's models get wrong (feed-forward scale, friction bias, assumed efficiency) | |
| `[[metric]]` | `id`, `kind`, window and statistic fields, `requirement`, `evidence`, `analysis` | `requirement`, `evidence` and `analysis` are tree ids checked by `tools/validate_plan.py` |
| `[output]` | `rate_hz`, `channels` | |
| `[parity_reference]` | published numbers from another tool | a second opinion only (§10.7) |

**`"case:<key>"`** may stand for any value: the loader reads that key from the case and converts it from the case CSV's unit. `tools/validate_plan.py` checks that the key exists, and that the unit matches a field whose name carries one (`_km`, `_deg`, `_deg_s`, `_h`, `_s`). An unstated key refuses the run, naming the key. This is how a case's values reach a scenario without anyone copying them.

**What the plant reads from the case.** Besides a scenario's own `case:` references, the plant reads the satellite from the case for every environment switch that is on. A case that leaves one of these blank cannot run that scenario, and the refusal names the key. The case checker (§8.3.4) and the case editor's "Is it ready?" read this table (`PLANT_READS` in `tools/plan_model.py`), so a person knows before running what their case can run.

| Always | `aerodynamic = true` | `solar_pressure = true` | `residual_dipole = true` |
|---|---|---|---|
| `mass.m`, `mass.imax`, `mass.iint`, `mass.imin` | `surface.afr`, `surface.cpa`, `surface.cd` | `surface.asun`, `surface.cps`, `surface.refl` | `magnetic.dres` |

**Scenarios are the release's.** A user picks a scenario and a campaign type from the release and runs it on their case; nobody edits a scenario in the software (§1.6). A new scenario, or a change to one, is asked for with a node form's "something else" and written by the developer team.

Anything that varies across runs is written as a distribution where the value would be: `{dist = "normal", mean, sigma}` or `{dist = "uniform", lo, hi}`. Parts' `[dispersion]` tables supply more, and so do the case's `lo`/`hi` ranges, which a campaign draws uniformly. The scenario never says which campaign it is in.

### 10.2 Campaign types: `adcs-campaign/1`

A campaign is a scenario and a rule for deriving runs from it. Every type below is derived from the same file. This is how "edge case, Monte Carlo, all the same template" holds by construction.

| Type | Derives | Fields |
|---|---|---|
| `nominal` | one run, every distribution at its mean | — |
| `montecarlo` | N runs, each distribution drawn | `runs`, `seed`, `dispersions` (`case`, `catalogue`, `scenario`), `statistic` |
| `edge` | each dispersed parameter at each end, one at a time (normal at ±kσ, uniform at lo and hi), then the worst corners found | `method`, `sigma_level`, `corners` |
| `sweep` | one parameter across a range, with seeded repeats per point; the parameter may be a case input (`"case:mission.w0"`) that takes a range | `parameter`, `from`, `to`, `points`, `repeats_per_point` |
| `fault` | the nominal run with each listed fault injected in turn | `faults`, `pass` |
| `labtwin` | the scenario with the plant replaced by the lab (§12.6) | `lab` |

The campaign runner expands a campaign into a run list, hashes it, and writes it before running anything. A campaign that is interrupted resumes from its list. A run that fails to start is recorded as refused, with its reason, and never silently dropped (VLEO: "a sweep records refused points; it never drops them").

### 10.3 Metrics

The metric kinds are the vocabulary §5.5's evidence rows name. The pointing and knowledge metrics follow ECSS-E-ST-60-10C.

| Kind | Computed as |
|---|---|
| `ape` | angle between the target and true attitude, over the window; the temporal statistic taken within the run |
| `rpe` | the error minus its mean over the stability window, then the temporal statistic |
| `pde` | the change of the windowed mean error between windows |
| `ake` | angle between the estimated and true attitude |
| `rke` | `ake` minus its windowed mean |
| `rate_stability` | the body-rate error's temporal statistic over the window |
| `pointing_rms` | root mean square of `ape` over the window |
| `time_to_threshold` | first time the error enters the threshold and stays there for `hold_s` |
| `time_to_rate`, `sun_acquisition`, `slew_time`, `settling_time`, `recovery_time` | the same rule on the named signal |
| `slews_per_orbit`, `tracking_rate`, `max_rate`, `momentum_margin`, `dump_interval` | measured from the recorded channels as their names say |
| `faults_survived` | faults injected and survived without loss of control |
| `power`, `power_peak` | orbit-average and peak electrical power of the ADCS, from the modelled draws (SILS) or measured at the supply (OILS, HILS) |
| `consumable_per_year` | propellant used in the run, scaled to a year of the mission's slews |
| `inspection_mass`, `inspection_volume` | not computed by a campaign: a unit weighed or measured, entered by production with its serial |
| `actuator_peak`, `actuator_power`, `consumable` | the named channel's statistic |

A statistic is always stated at two levels, as ECSS-E-ST-60-10C requires:

- **temporal:** within a run, `max` or a percentile over the window;
- **ensemble:** across runs, a percentile at a stated probability. For a metric bound to a requirement, the probability is the case's `level` for that requirement (`ensemble = "case"`), such as 99.73 % for 3σ.

A metric without both levels cannot supply an evidence row.

### 10.4 Pass, fail and evidence

A metric bound to a `requirement` passes when its ensemble statistic meets the requirement in the requirement's own `sense`, the same `mission::closure` the tree uses. A metric may bind a requirement nobody has written yet. Its value is recorded and its evidence supply kept, and the evidence closure answers "not stated by this case" until the case writes the requirement. The run never invents the bound. Nothing in the campaign restates the requirement's value or its probability: it reads both from the case, so a campaign cannot pass against a number the customer did not write.

A finished campaign emits one `EvidenceSupply` per bound metric (§5.5): the row id, the value, the rung, the campaign hash, the run count, the statistic and its confidence. That becomes `Case.evidence` for the customer's case, and the evidence closures evaluate. Supplying evidence changes no file: it is data in the run ledger, and a case run replays it by campaign hash.

### 10.5 Where the numbers come from

Four sources feed every campaign, and a fifth feeds a campaign on a built unit. They stay separate.

- **The case**: tree values, requirements and their levels, and the ranges to disperse. It is hashed.
- **The product and its tuned set**: parts, counts, mounts, algorithms, and the tuned parameter set's hash (§8.5).
- **The catalogue bundle**: part descriptors, including dispersions. It is versioned and hashed.
- **The scenario**: the run's own conditions. It is hashed.
- **The unit's serial descriptors**, for a campaign on a built unit (calibration, OILS, HILS, the as-built twin): the measured descriptors production recorded for that order (§7.4), exported from the portal as one folder with `GET /api/v1/internal/orders/{id}/serials` and hashed. The rig, `adcs fswcfg --serials <dir>` and `adcs eeprom --serial <file>` read that folder.

A campaign never writes to any of them.

### 10.6 The campaign manifest

`campaign_hash = hash(scenario hash, case hash, product id, tuned-set hash, catalogue bundle version and hash (or `unpublished` and the working copy's content hash, §7.2), serial descriptors hash (or `none` before a unit is built), flight software build id, engine kernel hash, adcs-sim-core hash, seed, run list hash)`. Two campaigns with the same hash are the same campaign (VLEO: "two runs with the same chain hash are the same run"). The evidence row's Reproducibility factor is 4 only when every one of those is present and every bundle verified.

### 10.7 Parity references are not fixtures

`[parity_reference]` holds numbers another tool published: IDMAS v2 §13's MATLAB simulation, or the company's MATLAB SIL. `tools/sil_parity.py` is VLEO's `mat_parity.py`, rewritten for runs. It compares a campaign's metrics with the scenario's parity reference and writes the difference and its cause to the parity ledger (§14.2). The rule is VLEO's, unchanged: a MATLAB number is a second opinion and never a fixture. "Where the two tools disagree, the disagreement is recorded with its size and its reason rather than tuned away."

The first expected parity result is informative either way. IDMAS v2 §13 reports 0.044° RMS and 210 s to 0.1°, with 10–30 % feed-forward errors, a friction model 20 % low and a pump efficiency of 10 %. The loop engine with the reference flight software will land near or far from that, and the ledger says which and why.

### 10.8 The MATLAB SILS twin — `matlab_sils/`

#### 10.8.1 What it is, and who it is for

The twin is a second implementation of **SILS only**, written in plain MATLAB. It reads the same case CSV and the same scenario, campaign, product, part and algorithm definitions as the platform. It runs the same dynamics, models, flight algorithms, campaign types and metrics, and it draws the results as ordinary MATLAB figures.

It has two audiences inside the company, and never clients:

- **Team members use it as released software** (§1.6). They download the zip of the latest release, read `manual/`, and write their input as a case CSV in the case editor it carries. They check the case, run SILS, look at the figures, and have each finished run saved as a result document in the zip's store, beside its case, to reopen, share and compare without running again. They never edit the code: a change they want goes to the developer team as a node form, from the node library in the same zip. An idea for a new algorithm goes the same way, as "something else" with the MATLAB prototype attached.
- **The developer team prototypes in it.** A developer tries a new algorithm, model or tuning idea in the twin and tests it on the real cases and scenarios. They then implement it in the platform (Rust for models, C for flight software), where the parity job (§10.8.6) shows that the two agree, and release both.

**The twin is built in lockstep with the platform, from the first phase (§10.8.7).** Every element of SILS, from a physics function to the campaign runner, is written in both engines in the same change, and the twin map `plan/twin_map.toml` says where each lives on each side. So the zip is never a port made after the fact: at every push it holds exactly the SILS the platform has reached, and whoever downloads it can run it end to end.

The platform stays the implementation of record. The twin is where anyone can run SILS with nothing but MATLAB, and where developers try things quickly. SILS is therefore available in both: in the platform (CLI, workbench, portal) and in MATLAB.

The twin has none of these: the tree and its closures, screening, the designer, the rig, the portal, or evidence and certification. A run in the twin never supplies an evidence row.

It is distinct from `matlab/+adcs` (§16). That package is a face that calls the compiled platform through its FFI library. The twin needs no platform binary at all, only base MATLAB. The Parallel Computing Toolbox is used when present (`parfor` over runs) and never required.

#### 10.8.2 One download, rebuilt on every push

`tools/pack_matlab.py` builds `dist/adcs_sils_matlab_<version>.zip` from the repository. It is in this package and runs today on the package's data:

```
adcs_sils_matlab_<version>/
  README.md  VERSION  MANIFEST.sha256  startup_asils.m
  TWIN.md                       what this zip carries of the twin map, element by element, and what is still to come
  +asils/                       the twin's code (matlab_sils/+asils/ in the repository)
  examples/  tests/             from matlab_sils/
  cases/                        plan/case_template.csv and plan/cases/*.csv, unchanged
  manual/                       the user manual (§16.4)
  forms/                        the case editor (§8.3.3), the node library: index.html, every node's form
                                and the new-node request (§5.10)
  viewer/result_template.html   the result-document page (§13.5) asils.result.save fills
  store/                        the user's local store: cases/, results/ and index.csv (§13.5.4), empty,
                                with a README
  data/                         JSON exports of plan/case_inputs.toml, plan/kpis.toml,
                                catalogue/{families,classes}.toml, catalogue/{parts,products,algorithms}/*,
                                catalogue/classes/*.csv, scenarios/*, campaigns/*, the seed content's test vectors,
                                and the parameter-id table (§7.5); no labtwin campaign;
                                twin_map.json: the twin map expanded, each element marked in_zip (§10.8.7)
  data/bundles/                 igrf14 and atmos-density, only when D4 allows redistributing them;
                                otherwise asils.data.install(<file>) reads the downloaded file
```

- **The case CSV is read directly.** MATLAB has no TOML reader, so everything else is exported to JSON, which `jsondecode` reads. The export is generated, never edited.
- **The version** is the git commit, the catalogue version (or `unpublished` with the working copy's hash), the case format (`adcs-case/1`), the engine hash, which CI passes with `--engine-hash`, and `twin_elements`, how many of the map's MATLAB elements the zip carries. `asils.version()` prints it. `MANIFEST.sha256` lists every file, and `asils.check()` verifies them.
- **The zip is deterministic.** Files are sorted, timestamps fixed and compression set, so the same commit gives the same bytes and CI compares two builds.
- **Kept in step with the repository.**
  - The `matlab-pack` job builds the zip on every push to `main` from P1 on, with `--phase` set to the phase reached, and keeps it as the "latest" artefact. It fails when an element that phase builds has no twin.
  - `release.yml` attaches it to every release.
  - The portal's internal downloads page always serves the latest `main` build (§15.5).

  Nobody maintains a copy by hand, so the twin a student downloads is always the repository as it stands.

#### 10.8.3 The same engine, function for function

| Platform | Twin | Rule |
|---|---|---|
| `adcs-case` import (§8.3) | `asils.case.read(file)` | the same checks, the same refusal messages, `NotStated` for a blank `stated` input; `asils.case.template()` writes the blank form |
| `case:` references (§10.1) | `asils.scenario.load` | resolved identically; an unstated key refuses the run by name |
| product supply (§8.4) | `asils.product.load` | the same counts, zero for empty slots, `NotFitted` for their per-unit values |
| `adcs_core::physics::<module>::<function>` (§6.2) | `asils.physics.<module>.<function>` | same names, same argument order, SI in and out; tested against the same fixtures |
| `adcs-sim-core` state and dynamics (§9.2) | `asils.plant.*` | the same equation, RK4 at `dynamics_step_s`, the same tick order (§9.3), quaternion renormalised each step |
| environment (§9.4) | `asils.env.*` | IGRF-14 synthesis to the same degree, the same density table, J2, conical eclipse |
| device emulators (§9.5) | `asils.devices.<part kind>`, `asils.plant.actuators` | at descriptor level: noise, bias, quantisation, latency, saturation, from the same descriptor fields. No byte-level protocols: a protocol bug is the platform's to find. |
| reference flight software (§9.10) | `asils.fsw.<algorithm id>` | one MATLAB function per algorithm in `catalogue/algorithms/`, reading parameters by the same ids (§7.5). Optionally, `asils.fsw.mex_reference` compiles the reference C flight software as a MEX file behind a MATLAB `adcs_hal.h` shim, so the C code itself runs in the twin. |
| campaign runner (§10.2) | `asils.campaign.run` | nominal, Monte Carlo, edge, sweep and fault; `labtwin` belongs to the rig and is not here |
| metrics (§10.3) | `asils.metrics.*` | the same kinds, windows and two-level statistics, judged against the case's requirement and level |
| `adcs-tune` (§8.5) | `asils.tune` | the same pattern search, box, clipping, objective and corners |
| recorder (§9.8) | `asils.rec.write` | the same `adcs-rec/1` channel names and units, as CSV plus a JSON manifest, so the platform's tools read a twin run |

Every row of this table is one or more entries of the twin map (§10.8.7), which names the file on each side, the phase that builds both and the test both pass.

**Randomness matches draw for draw.** `adcs-sim-core` exposes its counter-based draw (§9.7) as one function, and the pack exports 1,000 reference draws from it. `asils.rng.draw(seed, k, path, n)` must reproduce every one exactly. MATLAB's integer arithmetic saturates rather than wraps, so the 64-bit arithmetic is written out in 32-bit limbs. A Monte Carlo run *k* in the twin therefore flies the same dispersions as run *k* in the platform.

**Trajectories do not match bit for bit.** MATLAB's floating point and libm are not the platform's `pmath`. So parity is a measured difference with a cause (§10.8.6), never a bitwise claim.

#### 10.8.4 Visualisation, in ordinary MATLAB figures

| Function | Draws |
|---|---|
| `asils.viz.run(rec)` | 3D attitude: a 3U body with its faces, body axes, each actuator's axis, the field and Sun vectors, nadir and the target, animated with a time slider. Time plots: APE and AKE against the requirement line from the case, body rate, ring and wheel momenta, commanded dipole, measured field, power. The mode timeline with fault markers. |
| `asils.viz.campaign(res)` | per bound metric: histogram and empirical CDF with the requirement line and the ensemble percentile at the case's level; scatter of the metric against each dispersed parameter; a pass and fail table; the worst run opened in `asils.viz.run` |
| `asils.result.save(rec)`, `asils.result.open(file)`, `asils.result.import(file)`, `asils.result.index()` | a finished run or campaign saved as a result document (§13.5) into `store/results/<case id>/<sha12>/`, beside its case in `store/cases/`, with the same template and the same layout as the platform. `open` reads one back as `rec` without re-running; `import` files a result someone sent, with its case, and runs nothing; `index` rebuilds `store/index.csv` from the files. Every result opens in any browser too. |
| `asils.viz.compare(recA, recB)` | two runs overlaid channel by channel. Either may be a platform run, since the recorder formats are the same. This is how a developer sees their MATLAB change beside the platform. |
| `asils.viz.case(case)` | the case as read: every value, unit, range, level, assumption and unstated key |
| `asils.app` | one window: pick a case CSV, a product and a scenario, run, and open the plots. Built with `uifigure` only. |

#### 10.8.5 Using it

```matlab
startup_asils                                     % adds the twin to the path
c   = asils.case.read('cases/ais_img_3u.csv');    % the fixed CSV, checked
rec = asils.run('inertial_hold_3u', c, 'product', 'SYN-P-3U-FMR');
asils.viz.run(rec)
res = asils.campaign.run('inertial_hold_mc500', c, 'runs', 50);
asils.viz.campaign(res)
t   = asils.tune('inertial_hold_3u', c, 'product', 'SYN-P-3U-FMR', 'evaluations', 30);
```

The zip's `examples/` walks through six cases:

1. detumble on `ais_3u`;
2. fine pointing on `ais_img_3u`;
3. a Monte Carlo;
4. tuning;
5. comparing a twin run with a platform run;
6. saving a result, reopening it without re-running, importing one made by the platform, and comparing the two.

**A developer's loop** (the developer team only):

1. Write the new algorithm as `+asils/+fsw/<id>.m`, and add `catalogue/algorithms/<id>.toml` with `prototype = true`.
2. Run it in the twin on the real cases.
3. Implement it in `fsw/` (C, behind `adcs_fsw.h`), set `prototype = false`, and let the parity job compare the two.

A product may not carry a `prototype` algorithm, and `tools/validate_plan.py` refuses one that does. While `prototype = true` the algorithm is twin-only in the map; the change that sets it to `false` brings the C, and from then on the two move together. `tools/pack_matlab.py` refuses to pack when an element's twin folder exists without its file (an algorithm missing from `+asils/+fsw/`, a function missing from `+asils/+physics/+orbit/`), and, with `--phase`, when any element that phase builds has no twin.

#### 10.8.6 Parity between the twin and the platform

The CI job `matlab-parity` runs `detumble_3u` and `nadir_hold_3u` on `ais_3u` and `inertial_hold_3u` on `ais_img_3u`, nominal and a 20-run Monte Carlo, in both engines. It compares:

- every bound metric;
- the recorded channels over the run;
- the Monte Carlo metric distributions, run for run, since the draws match.

It writes one parity-ledger line per scenario and metric, with the engine `matlab` beside the platform's line (§14.2), and a person states the cause (H15). These lines are in the development ledger, never an order's, so they hold no certificate. Which differences fail the job is D23. Until D23 is decided the job is advisory: it records and never fails. Nothing is tuned to make the two agree. When they disagree, one of them is wrong, and the cause says which.

The job needs MATLAB on a CI runner (`matlab-actions/setup-matlab`), which is part of D5. Who may receive the zip outside the company, such as students at a partner institute, and on what terms, is D24: the zip carries the flight algorithms and the part descriptors. It never carries tuned values, and the pack leaves out whatever D1 restricts, as D1 records in `catalogue/restricted.toml`: whole files under `exclude`, and single keys under `[strip]`.

#### 10.8.7 Lockstep: one change, both engines

**In one line:** every element of SILS is written in the platform and in the MATLAB twin in the same change, from the phase that builds it, so the MATLAB SILS zip is always downloadable and always whole.

A twin ported after the platform is finished is always behind, and nobody can say by how much. ESA's practice on flight projects is the reverse: one model definition and one parameter database, carried from the MATLAB engineering simulator into the real-time simulator and every later facility (Bremer et al. 2017, the Euclid AOCS facilities); one functional simulator reused from model-in-the-loop to processor-in-the-loop (Colagrossi et al. 2023). Both are in §22. The platform does the same with two engines.

**The twin map.** `plan/twin_map.toml` (`adcs-twin-map/1`) lists every element of SILS. For each it gives the platform side (a Rust path and file, or a C file for flight software), the twin side (an `asils.` name and its file under `matlab_sils/+asils/`), the phase that builds both, the rungs that reuse it (SILS, OILS, HILS), and the test both must pass. Four families expand over their registries, so an item is in the map the moment it is in its registry:

| Family | Registry | One element per | Platform | Twin | Phase |
|---|---|---|---|---|---|
| physics | `plan/physics.toml` (in the repository, from P1, the generated table of `adcs-core::physics`, §6.2) | function | `adcs_core::physics::<module>::<name>` | `asils.physics.<module>.<name>` | P1 |
| fsw | `catalogue/algorithms/*.toml` | algorithm | `fsw/src/<id>.c` | `asils.fsw.<id>` | P3 |
| metric | every scenario's `[[metric]]` kind and every KPI's metric in `plan/kpis.toml`, except the two measured by inspection | kind | `adcs_sim::metrics::<kind>` | `asils.metrics.<kind>` | P3 |
| device | `catalogue/parts/*.toml` | part kind | `adcs_sim::emulate::<kind>` | `asils.devices.<kind>` | P3 |

Twenty single elements cover the rest: the case reader and template (P1), product supply (P2), the random draw, field, density, orbit, Sun and eclipse, disturbance torques, dynamics, actuators, scenario loading, a single run, the recorder and the result document (P3), and the campaign runner and the tuner (P4). That is 99 elements today. Three are one-sided by design, each with its reason: the lab twin (P4) and the byte-level protocols are the platform's, and the app and figures (P3M) are the twin's. A prototype algorithm is the twin's until its C lands.

**The rule.** From the phase that builds it, an element changes on both sides in the same pull request, or on neither.

| Check | Where | Refuses |
|---|---|---|
| `cargo xtask twin check` (`tools/twin_check.py` in this package) | every push | a map that is not whole: an element with one side and no reason, a twin outside `+asils`, a family whose registry is empty (TW01–TW04) |
| `twin check --repo . --phase <reached>` | the `twin` CI job | an element that phase, or a phase it needs, builds with either side missing (TW05). P3M runs beside P5–P7 (§19), so its elements are asked for only once it is named: `--phase P6,P3M` |
| `twin check --changed <file of paths>` | the `twin` CI job, on a pull request | one side of an element changed without the other, whatever the phase, unless the pull request carries the label `twin:none` with `--reason` saying why the other side needs nothing (TW06) |
| `pack_matlab.py --phase <reached>` | the `matlab-pack` job | a zip missing a twin that the phases reached build |
| `matlab-twin` | from P1 | the twin's own tests, each element's shared test among them |
| `matlab-parity` (§10.8.6) | from P3M | a difference with no stated cause |

A physics function is the smallest case. A node request that needs a new one (§5.10) gets a brief that asks for the Rust function and its MATLAB twin in the same commit, from the same relation and source, and runs the request's test vectors in both engines. So a node's formula reaches the MATLAB user in the same release as the platform user.

**What it does not claim.** The map proves that both sides exist and changed together, at the level of files: a platform module that holds several physics functions must change with at least one of their twins, but which function changed is not visible from paths, so review and the shared tests see to the rest. The shared tests and the parity job prove that the two agree. None of this proves that they are right: that is the fixtures' and the evidence's job (§14). Whether two definitions kept by hand stay in step is itself a belief (B-015), tested by the parity ledger.

The rule codes, generated from `tools/twin_check.py`:

<!-- twin:begin -->
| Code | The rule |
|---|---|
| TW01 | the map is adcs-twin-map/1; every element has a unique id, a phase that exists and rungs from sils, pil, oils, hils |
| TW02 | an element is on both sides, platform and twin, or is marked only = platform or only = twin, with the reason why |
| TW03 | a twin side is an asils. name in matlab_sils/+asils/; a platform side is a file under crates/ or fsw/ |
| TW04 | every family's registry yields its items, so each physics function, algorithm, metric kind and part kind is in the map |
| TW05 | in a checkout, every element built by the phases reached (and the phases they need) exists on both sides |
| TW06 | a change to one side of an element changes the other in the same pull request, or carries twin:none with a reason; a module file holding several functions changes with at least one of their twins |
<!-- twin:end -->

---

## 11. The loop contract — `adcs-bus::loop`

VLEO's bus crate was already written so that "the identical message types reach the rig and a flight target" (`vleo-bus/src/lib.rs`). The loop contract is a new module in it, `no_std` with `alloc`, `#![forbid(unsafe_code)]`, one struct per message.

### 11.1 Messages

```rust
pub struct TimeSync      { pub tick: u64, pub t_ns: u64, pub pps_edge: bool }
pub struct DeviceBytes   { pub device: DeviceId, pub port: PortRef, pub t_ns: u64, pub bytes: Bytes<256> }  // plant -> emulator -> OBC
pub struct CanFrame      { pub port: u8, pub t_ns: u64, pub id: u32, pub extended: bool, pub dlc: u8, pub data: [u8; 8] }
pub struct PwmCapture    { pub channel: u8, pub t_ns: u64, pub duty_q15: i16, pub period_ns: u32 }              // OBC -> rig
pub struct GpioEdge      { pub pin: u16, pub t_ns: u64, pub level: u8 }
pub struct TelemetryPacket { pub apid: u16, pub t_ns: u64, pub payload: Bytes<1024> }                          // OBC -> rig, recorded verbatim
pub struct Telecommand   { pub t_ns: u64, pub payload: Bytes<512> }                                            // rig -> OBC
pub struct FaultInject   { pub at_tick: u64, pub kind: FaultKind, pub target: DeviceId, pub duration_ticks: u32 }
pub struct TruthSample   { pub tick: u64, pub q_bi: [f64; 4], pub w_b: [f64; 3], pub b_b: [f64; 3], pub h_int: [f64; 3] }
pub struct RunControl    { pub command: RunCommand /* Arm | Start | Pause | Resume | Stop | Abort */, pub run_id: RunId }
pub struct Heartbeat     { pub node: NodeId, pub seq: u32, pub deadline_misses: u32, pub worst_latency_ns: u32 }
```

`DeviceId`, `PortRef`, `FaultKind`, `RunId` and `NodeId` are small enums and newtypes. `Bytes<N>` is a fixed-capacity byte buffer.

### 11.2 Framing and transport

Between the rig host and the interface emulation unit, the contract travels over UDP on a dedicated link:

- one datagram per rig cycle, carrying every message for that cycle;
- a header with version, sequence number and cycle tick;
- a CRC-32 over the payload.

A lost or late datagram is counted, never retried within the cycle. The emulator holds its last value and flags it, and both sides record the event. The version is negotiated at `Arm`; a mismatch refuses the run by name.

In SILS the same messages pass in-process through a channel, so the emulator code is the same code.

### 11.3 Timing

The rig cycle is `plant_step_s` from the device map, 1 kHz in the example. Every message carries the tick it belongs to. The OBC runs on its own clock, disciplined by the rig's PPS line and a `TimeSync` packet. The rig records the offset each cycle, and the recorder puts OBC telemetry on the rig's time base using it.

---

## 12. The rig — PIL, OILS, HILS

### 12.1 Four rungs, one scenario

| Rung | Flight code runs on | Plant runs on | Link | Hardware in the loop | Timing | Who |
|---|---|---|---|---|---|---|
| SILS | linked into the loop engine process | loop engine, on a server | in-process | none | faster than real time | clients and engineers |
| PIL | the target processor (Raspberry Pi, PolarFire SoC) | loop engine on a PC | serial or UDP carrying the loop contract | the processor | soft real time | engineers |
| OILS | the flight OBC (OBC-in-the-loop) | the rig host, real time | the OBC's flight connectors, answered by the interface emulation unit | the OBC, and the GNSS receiver's output emulated for orbit determination | hard real time | engineers, billable campaigns |
| HILS | the flight OBC | the rig host, real time | real sensors and actuators | the full ADCS unit in the Helmholtz cage on the air bearing, with the stimulators | hard real time | qualification of each delivered unit |

The scenario file is the same at every rung. The device map (`rig/device_maps/*.toml`) is the only thing that says, device by device, whether a port is answered by an emulator or by the part. That lets a campaign move from OILS to HILS one device at a time.

### 12.2 Hardware roles

The constraint is to buy standard hardware and build the software, and it holds here.

| Role | What it is | Built or bought |
|---|---|---|
| Rig host | an x86-64 PC running Linux with PREEMPT_RT (in mainline since 6.12), running `adcs-rig` | bought |
| Interface emulation unit (IEU) | an FPGA or MCU board on the flight connectors. It answers I2C and SPI as a target, emulates UART, RS-422 and RS-485, captures PWM and speaks CAN, at electrical level, from `devices/*.toml`. The PolarFire SoC or Red Pitaya boards already in hand are candidates; the choice is D14. | board bought, gateware and firmware built |
| Environment simulator | the Helmholtz cage and its bipolar supplies, the sun simulator and shutter, the star stimulator (microdisplay and collimator), the GNSS RF simulator, the air-bearing platform with its balancer | bought; `adcs-rig` drives each |
| Metrology | the reference fluxgate in the cage, optical truth attitude of the platform, current and voltage probes, an oscilloscope or DAQ for timing | bought |
| Actuator test stands | a force dynamometer for wheel torque, imbalance and micro-vibration (Nadeem 2022), a magnetometer array or rotation rig for coil dipole and residual dipole (Springmann and Cutler 2010), a torque stand for the rings, and a thrust stand with impulse-bit resolution for the thrusters. Each is used only when the product carries that family. | bought |
| Power | programmable supplies for the OBC and the unit, with brownout injection | bought |

What each role must achieve is not written here. It is two sets of rows in the tree (§5.3). **What the lab can do** is layer 1's facility rows (OILS rig, magnetic field simulator, air-bearing platform, optical and RF stimulators, actuator test stands, rig safety and power), each read from the lab file by the `lab` supplier (§8.2), so the file stays the one place a measured capability is written. **What this case needs** is layer 2's "OILS rig needs" and "HILS rig needs" rows, derived from the case: the field range from the orbit's strongest field, the bearing torque allowed from the actuator authority and the disturbance torque, the truth accuracy from the knowledge requirement, and so on. `adcs rig fit` compares the two (§12.10). The figures a facility is specified by follow the literature: Helmholtz cages by field range per axis, magnitude and direction error and the homogeneous volume (da Silva et al. 2019); Sun simulators by the ASTM E927 classes; star stimulators by angular step, faintest magnitude and frame rate (Zhao et al. 2024); air bearings by residual torque, tilt range and balance (Schwartz et al. 2003); flat-sats by the interfaces emulated, missed ticks and isolation (Colagrossi et al. 2023).

### 12.3 Real-time design

- PREEMPT_RT kernel, `isolcpus` for the rig cores, `SCHED_FIFO` for the loop thread, `mlockall`.
- No allocation, no locks and no system calls in the loop except the UDP send and receive.
- The rig's `plant_step_s` is the dynamics step at OILS and HILS, replacing the scenario's (§9.3).
- Every cycle measures its own latency. A cycle that misses its deadline is counted. The run's policy (from the device map) is abort, or continue with the count recorded against the layer-1 row "deadline misses allowed per hour".
- Acceptance: `cyclictest`-style latency histograms are recorded for the rig host at commissioning. The run manifest carries the rig's measured worst-case latency, so a HILS result states the timing it was made under.

### 12.4 `adcs-rig`

- `rt/`: the real-time loop, running `adcs-sim-core` at `plant_step_s`.
- `transport/`: in-process, UDP to the IEU, serial, SocketCAN.
- `emulate/`: the same device emulators as `adcs-sim`, reused, not copied.
- `envsim/`: cage (SCPI over LAN to the supplies; the commanded field is `B_lab = R_lab←ECI · B_ECI(r, t)` from the real-time orbit), sun shutter, star stimulator renderer (stars from the catalogue the tracker carries, at the platform's truth attitude), GNSS simulator control.
- `metrology/`: truth-attitude ingest, reference magnetometer ingest.
- `record/`: the recorder, on the rig's time base.
- `safety/`: software limits that stop a run.

Every driver has a mock backend. The mocks are complete enough that a whole OILS or HILS campaign runs in CI with no hardware attached.

### 12.5 OILS

The flight OBC's own board support package implements `adcs_hal.h`, and the OBC runs the flight build under test. The rig host runs the plant and the device emulators. The IEU answers every sensor port with bytes from the plant and captures every actuator command on the flight connector: PWM duty to the coils, CAN to the ring modules. OILS includes orbit determination: the GNSS receiver's output is emulated from the plant's truth orbit at protocol level.

The pass criteria are the scenario's. The rung's own test is the parity ledger line against SILS for the same campaign.

### 12.6 HILS and the lab twin

In HILS the unit's real sensors and actuators are in the loop, and the environment simulator makes the environment. The cage follows the real-time orbit's field in the lab frame; the sun simulator and star stimulator stand in for the sky; the air bearing gives real rotational dynamics.

**An air bearing is not orbit.** Its residual gravity torque (m·g·|r|, the weight times the centre-of-mass offset, Schwartz et al. 2003; about 1.5 × 10⁻⁴ N·m for a 15 kg platform at a 1 µm offset) is larger than the orbital disturbances a CubeSat must reject, and its tilt is limited. So HILS is never compared with the orbit run. It is compared with the **lab twin**: campaign type `labtwin`, which runs the same scenario with the plant replaced by the lab from `rig/labs/*.toml`, using the platform inertia, the bearing's residual torque, the cage's measured field error and the serials actually fitted. `rig/labs/hils_bay1.toml` is the real bay; every value is `nan` until the facility measures it, so its lab twin is refused until then. `rig/labs/syn_lab.toml` is a synthetic lab with stated round numbers, so the lab-twin machinery runs from P4. The parity ledger carries both lines, SILS against the lab twin against HILS, so the orbit prediction and the lab measurement meet through a model of the lab rather than being compared directly.

### 12.7 Fault injection

The fault kinds in `campaigns/ring_freeze_fault.toml` and the device map's `[[fault_point]]` table are the same names at every rung:

- **SILS:** the emulator or the plant injects them.
- **OILS:** the IEU does: stop answering, corrupt a CRC, NACK, hold a value.
- **HILS:** relays and the programmable supply do: open a coil, brown out the OBC.

The fault-injection controls are internal only. A client sees the faults a campaign injected, never the console.

### 12.8 Safety

Cage current, bearing tilt and supply voltage have **hardware** interlocks and an emergency stop that do not depend on this software. The software limits in `safety/` stop a run earlier and more gently, and never replace the hardware. A person signs the rig's safety check before the first powered test on each configuration (H-rig, §17.2). The builder writes the checklist and does not sign it.

### 12.9 What needs hardware, and what the builder does instead

The builder writes every driver against its device's interface document and a mock backend, and proves the campaign runs end to end against mocks. Bring-up on the real device is `docs/RIG_BRINGUP.md`: a checklist per device of what a person does, measures and records. It is marked not done until a person has done it.

### 12.10 Rig fitness: `adcs rig fit`

**In one line:** before a rig campaign is armed, `adcs rig fit <case> --lab <file> [--product <p>] [--rung oils|hils]` compares what this case needs from the rig with what the lab has measured, row by row, and refuses by name whatever the lab cannot meet or has not measured.

The two sides are in different layers, and a row never reads across layers (C04), so the comparison is not a tree row. It is a read-only report over two sets of rows the tree already computes: layer 2's needs (`v3` OILS rig needs, `v4` HILS rig needs, for this case and product) and layer 1's capabilities (the facility rows: measured ones from the lab file, and the two policy rows marked declared).

| Need (layer 2) | Capability (layer 1) | Fits when |
|---|---|---|
| Real-time plant step needed | Real-time plant step (declared) | the rig's step is at most the need |
| Flight ports to answer | Interface emulation channels | the channels are at least the ports |
| Rig loop latency allowed | Rig loop latency, worst case measured | the measured latency is at most the allowance |
| Emulator link rate needed | Rig host to IEU link rate | the link rate is at least the need |
| Real-time run length | OILS rig hours per week (declared) | one run fits within a week's rig hours; a campaign's total is the booking's, not the fit's |
| Cage field range needed | Helmholtz cage field range | the range is at least the need |
| Cage field accuracy needed | Cage field error against command | the error is at most the need |
| Cage field slew rate needed | Cage field range over coil time constant | the slew the coils can make is at least the need |
| Bearing residual torque allowed | Air-bearing residual torque | the residual is at most the allowance |
| Bearing balance offset allowed | Balance residual centre-of-mass offset | the offset is at most the allowance |
| Bearing tilt range needed | Bearing tilt range | the range is at least the need |
| the unit's mass (`m`) | Platform payload capacity | the mass is at most the capacity |
| Platform inertia to match | Platform inertia | reported: the lab twin flies the sum, so no single comparison decides |
| Truth attitude accuracy needed | Truth attitude accuracy | the lab's accuracy is at most the need |
| Sun simulator irradiance needed | Sun simulator irradiance | the irradiance is at least the need |
| Sun simulator collimation needed | Sun simulator collimation half-angle | the half-angle is at most the need |
| Star stimulator error allowed | Star stimulator angular step | the step is at most the allowance |
| Star stimulator frame rate needed | Star stimulator frame rate | the rate is at least the need |
| Coil dipole to measure (`mtq`) | Coil dipole measurement resolution | the resolution is below the value to measure |
| Wheel torque and disturbance to measure (`rw`) | Dynamometer torque resolution | the resolution is below the value to measure; the dynamometer's bandwidth and the imbalance resolution are reported beside it |
| Ring torque to measure (`fmr`) | Ring torque measurement resolution | the resolution is below the value |
| Thrust to measure (`rcs`) | Thrust stand resolution; impulse bit resolution | the thrust resolution is below the thrust; the impulse bit resolution is reported |

- **One verdict per line:** fits, short (with both values and units), reported (a value shown with no comparison, where none decides), NotMeasured (the lab value is `nan`), NotStated (the case leaves a key the need reads blank), or not applicable (the product has no part of that family). An OILS fit reads the first five lines; a HILS fit reads every line, since HILS also runs on the rig host and the IEU.
- **No margins are invented.** "Fits" is the plain comparison. A margin a person wants on a line is a request on the need row, which then carries it.
- **It gates, and it is not evidence.** `adcs rig arm` refuses a campaign whose fit has a short, NotMeasured or NotStated line, naming each, as `inertial_hold_labtwin` is refused on `hils_bay1`'s `nan` fields. The fit report is filed with the campaign's manifest. It never supplies a row and never changes the lab file: a measured capability reaches the lab file through a person's bring-up record (`docs/RIG_BRINGUP.md`).
- **Built with the rig:** the OILS lines in P6, the HILS lines in P7. `syn_lab.toml` has a value in every field the fit reads, so the machinery runs end to end from the start, and whether it fits a case is recorded, not expected. The real bay answers NotMeasured on every line until it is measured, which is the honest state.


---

## 13. Visualisation

### 13.1 One set of views for every rung

A SILS run, an OILS run and a HILS run are shown by the same views, reading the same `adcs-rec/1` channels. The only difference on screen is a rung badge (SILS, PIL, OILS, HILS, LAB TWIN) and, for a live rig run, a latency and deadline-miss strip. A view that works for SILS therefore works for HILS on the day the rig first runs, and a client who learned the SILS view reads the witness view without being taught.

### 13.2 The views

| View | Shows | Who |
|---|---|---|
| Case | the case editor (§8.3.3) and, once a CSV is opened, the case report: each value, where it went (the tree row, the scenario fields that read it), the assumptions taken, every unstated key with what it blocks, and which scenarios the case can run | client, team member |
| Solution | per product tried: margins after tuning against each requirement, the governing credibility, the confirming campaign; or "no catalogue product meets this case" with the gap per requirement and the design-request button | client (offered products only), engineer |
| Catalogue (internal) | products by class and status, candidates included, with their envelope and worst margin; a design job's sweep, combinations pruned, tuned and passed; the promotion checklist of §8.6 | designer, catalogue owner |
| Run | choose the case, the product (or let the solver choose), a scenario of the release and a campaign type, and a run count within quota; the case's values the scenario reads are shown beside it, read-only. Nothing here edits a scenario (§10.1). | client, team member |
| Run, live and replay | **3D attitude**: the body with its faces, each actuator's axis, the magnetometer's field vector, the Sun vector, nadir, the target, and the star tracker's field of view and exclusion cones. **Time plots**: APE and AKE against the requirement line, body rate, ring momenta and speeds, wheel speeds, commanded dipole, measured field, power. **Mode timeline** with fault markers. **Actuator panel**: each actuator's use against its limit. **Requirement gauges**: each bound metric against its requirement, with the margin. | client (SILS), witness (OILS, HILS), engineer |
| Campaign dashboard | per bound metric: a histogram and CDF across runs with the requirement line and the ensemble percentile marked; scatter of the metric against each dispersed parameter, so the thing that drives a failure is visible; a pass and fail table; one click opens the worst run in the run view | client (SILS), engineer |
| Parity | one metric or channel overlaid across SILS, lab twin, OILS and HILS for the same scenario, with the parity-ledger line and its stated cause | engineer; client after the order |
| Facility (internal) | commanded against measured cage field, bearing tilt and balance, stimulator state, IEU link health, latency histogram | test operator |
| Witness (client, after the order) | the run view, read-only and live, streamed from the rig for the client's own unit, with the campaign's schedule and a comment box routed to sales | client |
| Evidence | the verification matrix, parity ledger, reports and certificate, per order | client (their order), quality |
| Results | the local store (§13.5.4): every saved result, by case, with its verdict and worst margin, searchable; "Open a result", which files a result made anywhere beside its case and shows it without running anything; compare any two | everyone |
| Node | a node's document, read-only, as the node library shows it, and "Ask for a change", which downloads its node form (§5.10) | team member |
| Risk management | the platform's conclusion (highest open level, net risks closed this quarter, share of beliefs tested), the seven areas on one 0–5 scale, the open L4 and L5 risks with their closing tests, and each quarter's narrative (§5.13); read-only | everyone internal; quality |

Every view follows the explanation standard (§5.12): a breadcrumb, an answer-first sentence, an overview before detail, each panel's kind, the Learn · Read · Expert switch, claim tags on every number that is not the user's own, and a "where this breaks" panel.

### 13.3 How it is built

It is VLEO's web face, extended: one `index.html`, one `app.css`, ES modules served individually, no bundler, `app.js` owning every listener, and views emitting `data-` attributes. The new modules are:

- `sim.js`: the run view;
- `campaign.js`: the dashboard;
- `parity.js`;
- `attitude3d.js`: the 3D view;
- `stream.js`: live data;
- `scenario.js`: the scenario picker (read-only);
- `witness.js`;
- `evidence.js`;
- `case.js`: the case view, with the case editor served inside it;
- `results.js`: the results store view;
- `solve.js`: the solution view;
- `catalogue.js`: the internal catalogue view.

The node page (VLEO's, ported, read-only) gains "Ask for a change", which serves the node's form (§5.10). No page of the web face writes anything to the repository or to a sheet (§3.4); the only things a user puts into the software are a case CSV, a result document to view, and, in the portal, a client's FMU.

- **3D:** `three.js`, vendored as one ES module file under `web/vendor/`, MIT licence, recorded in `ADOPTION.lock` with its version and fallback. The fallback is a 2D projection of the same scene.
- **Charts:** VLEO's `chart.js` canvas routine and its measured light and dark `SCHEMES`, extended with a streaming time series that keeps a fixed window.
- **Live data:** server-sent events, `GET /v1/stream/<run>`: one-way, simple, and they pass through proxies. At most 20 Hz of downsampled channels; the recorder keeps full rate.
- **Replay:** the same view reads the recorder's files; the time slider scrubs, and the 3D and the plots move together.
- **Colour and tokens:** VLEO's `app.css` tokens and dark override, unchanged. The rung badge colours are new tokens, validated like the rest.
- **Units:** the face converts for display only. Everything it receives is SI (VLEO `areas/faces.md`).

### 13.4 Panels, checked in a real browser

Every new view is a declared panel in `panels/<id>.toml` and passes VLEO's `panel_check.py` checks:

- it renders;
- it moves when each declared input moves;
- it reads, changing when the engine's answer is intercepted;
- it matches its reference image in light and dark, or declines the pixel check with a reason.

The new panels are `case_report`, `solution_margins`, `catalogue_classes`, `run_3d`, `run_plots`, `run_modes`, `campaign_histogram`, `campaign_scatter`, `parity_overlay`, `facility_cage`, `witness` and `evidence_matrix`. Their `confirmed_by` stays `UNCONFIRMED` until a person has looked at the reference and agreed with it. Their reference images are recorded from the mock rig and a synthetic campaign, so they exist on day one and are replaced when real data exists. A panel reference is a picture of what the code drew, taken to catch the picture changing. It is never evidence about the ADCS, and no evidence package includes one.

### 13.5 Result documents and the local store

#### 13.5.1 Why a result is a file

A run or a campaign is computed once. What it produced is saved once, as a **result document**, and after that it is opened, compared, shared and reported without running anything again. The same file opens in four places:

- any browser, offline;
- the web face and workbench (§13.1);
- the portal;
- the MATLAB SILS tool (§10.8).

So a team member who ran a case last week, or received a colleague's result, sees every plot and every verdict at once. The result is filed beside the case it ran (§13.5.4). The simulation, and its compute, is only needed for a new question.

`results/template.html` is the page, and `tools/results.py` in this package writes and reads it. `results/examples/` holds two demonstration results, a detumble run and a 20-run Monte Carlo on `ais_3u`. They were made by a deliberately simple Python B-dot model written only so the page has real trajectories to show. Every page they produce says so at the top, and their numbers are not evidence (§13.5.7).

#### 13.5.2 The format: `adcs-result/1`

One self-contained HTML file, `<case>_<scenario>_<date>[_<campaign>].result.html`. Like a form document (§5.10), it is a page drawn from one JSON block (`id="adcs-result"`), and every `<` inside the block is written as `\u003c`. The block holds:

| Member | What it records |
|---|---|
| `schema`, `kind`, `title`, `created` | `adcs-result/1`; `run` or `campaign` |
| `engine` | `platform` or `matlab`, with version and commit, and a `note` shown at the top of the page for anything that is not the platform. The demonstration results use `demo`. |
| `case` | id, hash, and **the case CSV exactly as it was run** |
| `product`, `tuned`, `scenario`, `campaign` | the product id, family, parts and whether it is synthetic; the tuned set's hash, never its values; the scenario id and hash; the campaign type, run count, seed and what it dispersed |
| `requirements` | every requirement the metrics are judged against: key, row, label, unit, sense, value, level, and whether the level was an assumed default |
| `metrics` | per metric: kind, unit, bound requirement, every run's value, the ensemble statistic with a note when the run count cannot support the stated level, the margin, and the verdict |
| `runs` | per run: index, dispersed values, metric values, and whether its channels were kept |
| `channels` | rate, names, units, optional chart `groups`, and per kept run the samples as base64 of gzip of little-endian float32, column-major |
| `restricted_excluded`, `notes` | what D1 kept out of this file, and anything the reader must know |
| `credibility` | the level the result can be trusted at, and why (the lowest of the eight scores over the rows it read; `none` for the demonstration engine) |
| `limits`, `unconfirmed` | where this result breaks: what the model leaves out, and the rows it read that nobody has confirmed |

**What is kept.** A single run keeps its channels. A campaign keeps every run's metrics and dispersed values. It keeps full channels for the nominal run, the best run, the worst run and any failing run, plus every run with `--keep all`. Any other run is re-created exactly from the campaign's seed (§9.7): the page says so, and `adcs sim rerun <result> <k>` does it.

**What is left out.** Restricted content never enters a result document, and a test proves it (§14.6). That covers D1 rows and channels, controller gains and tuned values (only the tuned set's hash is kept). A result document made for a client also leaves out layer-3 channels.

#### 13.5.3 What the page shows

The page is the run view and campaign dashboard of §13.2, drawn from the file, written to the explanation standard (§5.12):

- **Answer first:** one sentence under a breadcrumb (store › case › scenario): how many requirements pass, for which case and product, in how many runs, on which engine, whether it is a demonstration, and its credibility.
- **How to read this result** (overview): the four parts, from the verdict to the data. **Using this file** (how-to, hidden in Expert).

- **Summary:** each requirement's verdict, with a status icon and a label, never colour alone. Also the case, product, scenario and run count.
- **Requirements and metrics:** each metric against its requirement at the case's level: achieved value, statistic, margin, verdict. The table is downloadable as CSV.
- **Time histories:** the kept runs' channels, one chart per group, with a crosshair and tooltip. Requirement and threshold lines are drawn dashed. Each run's channels are downloadable as CSV.
- **Attitude:** a 3D view of the body, its axes and the field direction, with a time slider and play.
- **Campaign:** for each metric, a histogram with the requirement and the ensemble statistic marked, and the table of every run. Clicking a kept run shows it above.
- **Compare:** "Open a result to compare…" loads a second result document into the same page. Its metrics sit beside these, and its first channel is overlaid, dashed. The case, product or engine may differ: a MATLAB result against a platform result is how a developer checks the twin (§10.8.6).
- **Where this came from:** engine, case and its hash, product, tuned-set hash, scenario and campaign. It also shows the case CSV itself, downloadable, so anyone can run exactly that case again.
- **Where this result breaks:** the `limits`, the unconfirmed rows read, and every level the case assumed; then the common wrong idea that a pass means the satellite will meet the requirement in orbit, and why not.
- **What a margin is** (a station under the metrics table) and, in Learn, a prediction to make before reading the verdicts and a teach-back at the end.

Every section shows its kind, and the depth switch shows the exercises (Learn), the answer and explanation (Read), or the tables alone (Expert).

The charts follow one set of rules:
- one y-axis per chart;
- at most four series, with the categorical colours in fixed order and a legend;
- dashed ink for requirement lines;
- light and dark palettes selected, not inverted;
- no external script, so the page works offline and prints.

#### 13.5.4 The local store: every result beside its case

Every place that runs SILS keeps a **store**: a folder in which each result document is filed beside the exact case it was run on.

```
<store>/
  cases/<case id>/<sha12>.csv                              the case, as run (the CSV the result carries)
  results/<case id>/<sha12>/<scenario>_<date>[_<campaign>].result.html
  candidates/<id>.toml                                     products the designer saved (§8.6)
  index.csv                                                one row per result; index.sqlite in the workbench
```

A new store starts with the release's reference cases in `cases/`, so the default case `ais_3u` is there on first run. `<sha12>` is the first twelve hex digits of the SHA-256 of the case CSV. Two versions of a case therefore never share a folder, and a result can never be shown beside a case it was not run on.

| Where | The store |
|---|---|
| CLI and workbench | `~/.adcs/store/`; `--store <dir>` for another |
| MATLAB SILS tool | the zip's `store/` |
| Portal | each project's artefact directory, indexed in its PostgreSQL `case` and `run` tables (§15.4) |

**The files are the record, and the index is a cache.** `index.csv` has one row per result: the file, its case file, kind, date, engine, case and case hash, product, scenario, campaign type, runs, overall verdict, worst margin and the file's SHA-256. The workbench keeps the same index in SQLite, `index.sqlite`, for fast search. Both are rebuilt from the files by `adcs result index` or `asils.result.index`, and deleting either loses nothing. There is no database a result lives only in, so a store, or any folder of it, can be copied, zipped or mailed as it is, and opened by any of the three.

**Saving is automatic.** Every run and campaign a user starts in the workbench, the web app or the portal is saved when it finishes; the CLI saves with `--save`, and the MATLAB tool with `asils.result.save`. A run that was refused writes no result; its refusal is in the run's log.

**Opening a result runs nothing.** "Open a result" in the web app and the workbench, `adcs result import <file>...`, `asils.result.import(file)` and the portal's result upload all do the same thing: read the result document, write the case it carries into `cases/` if that exact case is not there yet, file the result under it, and update the index. Then the result is shown — its plots, verdicts and 3D view — with no engine call. That is how a result made last week, or on another computer, or by a colleague, or by the MATLAB tool, or by the rig, is seen again. `tools/results.py import` is the stand-in in this package, and it runs today.

Commands:
- `adcs sim run … --save` and `adcs sim campaign … --save` write the result document into the store;
- `adcs result import <file>...` files results made elsewhere, each beside its case;
- `adcs result list [--case c] [--verdict fail]` searches the index;
- `adcs result open <file>` opens the page;
- `adcs result channels <file> <run> <out.csv>` exports one run's channels;
- `adcs result index [<store>]` rebuilds the index;
- `adcs sim rerun <file> <k>` re-creates a run whose channels were not kept — the one command that runs the engine, and only when asked.

#### 13.5.5 Who writes result documents

| Writer | When |
|---|---|
| `adcs-sim` (the platform) | every run and campaign with `--save`, and every portal SILS run (the portal saves by default) |
| the MATLAB SILS tool | `asils.result.save(rec)`, with the same template, shipped in the zip as `viewer/result_template.html`, into the same store layout |
| `adcs-rig` | every OILS and HILS campaign, with the rung in `engine` and the witness stream's recording |
| `tools/results.py demo` | only the package's two demonstration files |

**A result carries a pointer back to its case, never the other way round.** A case is input and never changes; results accumulate beside it. Deleting a result never touches its case, and a case with no results left stays in the store until someone deletes it.

A result document is output, never input. No engine reads a result document's numbers as evidence, and a campaign's evidence rows come only from the run ledger (§5.5). The page is a view of those numbers.

#### 13.5.6 Checks

The `forms` CI job (§18), in `tools/form_browser_check.py`, also opens the demonstration results in headless Chromium. It checks:
- every chart draws with no script error;
- `tools/results.py import` files both demonstration results under `ais_3u`'s one case folder;
- the channels decode;
- opening a second result adds the comparison;
- `tools/explain_check.py` finds every mark of the standard on both.

In the built repository the `results` job goes further:
- the platform writes a result for `detumble_3u` and the MATLAB twin writes one for the same case;
- each opens in the other's viewer;
- the channels decoded by the page equal the recorder's output;
- `adcs result index` rebuilds the same index twice;
- `adcs result import` of a result into an empty store writes exactly its case and the result, beside each other, and nothing runs;
- the restricted-content test runs over a result document.

#### 13.5.7 The demonstration results are not results

`tools/results.py demo` exists only so the viewer can be seen and tested before the engine exists. Its model is deliberately simple:
- a rigid body with `ais_3u`'s inertia;
- a circular orbit;
- a tilted dipole field fixed in inertial space, with no disturbances;
- ideal sensors, and B-dot on three ideal 0.45 A·m² coils.

Its engine is named `demo`, and every page it writes says so above everything else. It is never a parity reference, a fixture or evidence, and the builder never quotes its numbers as the platform's.

---

## 14. Evidence and certification — `adcs-evidence`

### 14.1 The requirement package: before the order

Generated from the case, the selected product and its tuned-set hash, and shown to the client with the quote. The case CSV itself, as uploaded, is attached with its hash, so the requirement package can be traced back to the file the client sent:

- **ADCS requirement specification**: every written requirement row with its value, sense and analysis margin, laid out in the structure of ECSS-E-ST-60-30C (AOCS requirements). The pointing and knowledge requirements are stated as ECSS-E-ST-60-10C indices with both statistical levels (§10.3).
- **Interface control document**: power, data interfaces, mass, mounting and the EEPROM contents, from the chosen parts' descriptors.
- **Verification matrix**: per requirement, the method (test, analysis, review of design or inspection, per ECSS-E-ST-10-02C), the rung that will supply it (analysis, SILS, OILS, HILS) and the campaign that will run.

After the order, that matrix becomes the test plan.

### 14.2 The parity ledger

One line per scenario × metric × rung × engine. The engine is `platform`, or `matlab` for the MATLAB twin (§10.8.6). A certificate reads only the lines of its own order's campaigns, which are always `platform`. Twin lines belong to the development ledger: they hold no certificate, and their causes are the design team's work, not an order's. Each line holds:

- the value;
- the campaign hash;
- the difference from the rung before it;
- the difference from the analysis row;
- the difference from any parity reference;
- a **cause**, written by a person.

A difference without a cause is an open item that holds the certificate. Nothing is tuned to close a line: a model is changed only when the cause says the model was wrong, and the change is a reviewed commit that names the line.

### 14.3 Reports

Every campaign produces a report: configuration (parts and serials), flight software build id, scenario and campaign hashes, rig and lab, metrics with both statistics, pass or fail per bound requirement, deviations, and links to its runs. The report is HTML generated from the ledger. The PDF is produced by the same pipeline, and regenerating from the same ledger gives the same bytes; CI checks this.

### 14.4 The as-built twin

Before a unit's OILS campaign, the client's SILS scenarios are re-run with that unit's **serial** descriptors in place of the catalogue values. A unit whose as-built run no longer closes is caught before any rig time is spent on it. The as-built results are a parity-ledger line of their own.

### 14.5 The certificate of conformance

The certificate lists:

- the order, the unit serials and the flight software build id;
- every requirement with its closing evidence: rung, campaign, value and margin;
- open deviations and waivers;
- the parity ledger's state;
- the hash of the evidence package.

It is issued only when:

- every requirement in the verification matrix is closed or waived;
- every parity line has a cause;
- no evidence campaign names a synthetic or placeholder part;
- a person in quality signs it (H-cert).

The builder writes the generator and never a signatory. Until D3 there is no signing key, so the certificate carries its package hash and the signatory's name, and says it is unsigned.

### 14.6 The evidence package

A zip of the requirement package, the verification matrix, the parity ledger, every campaign report, the run manifests (without the full-rate channels unless requested), the certificate and a manifest of hashes. Restricted content is excluded by rule and never by hand: the controller gains and tuned parameters (fswcfg sections `0x05` and `0x06`), restricted rows (D1), and other clients' data. A test builds a package from a campaign that contains restricted values and checks that none of them are in the package.

### 14.7 Standards mapping

| Standard | Where it lands |
|---|---|
| ECSS-E-ST-60-30C, AOCS requirements | requirement specification structure (§14.1); layer-1 row "Requirement clauses covered" |
| ECSS-E-ST-60-10C, control performance | metric definitions and statistics (§10.3); the pointing budget (`gp_5`) |
| ECSS-E-ST-10-02C, verification | verification methods and the matrix (§14.1) |
| ECSS-E-ST-10-03C, testing | test levels applied at OILS and HILS; layer-1 row "Test levels applied" |
| ECSS-E-ST-40C and ECSS-Q-ST-80C, software | the flight software's process and CI (§9.6, §18); the software criticality category is D15 |

Which clauses are tailored is a person's decision per order, recorded as layer-1 "Tailoring items".

---

## 15. The portal — `adcs-portal` and `adcs-worker`

### 15.1 Shape

VLEO's daemon stays what it is: one local process on loopback, the engineer's workbench. The portal is a separate service for everyone else, and it never computes a number itself.

```
browser --TLS--> reverse proxy --> adcs-portal (axum + tokio)  --SQL--> PostgreSQL (operational data)
                                        |  enqueue                        ^
                                        v                                 | status, results
                                   run queue (a PostgreSQL table) <--- adcs-worker xN --> engine libraries
                                                                          |           (adcs-case, adcs-solve, adcs-sim, adcs-evidence)
                                                                          v
                                                                    run artefacts (object directory)
facility network --outbound only--> adcs-portal /internal/rig/stream   (live OILS/HILS witness)
```

- **No database on the physics path.** This is VLEO's rule, kept: workers read verified bundles from the local store and write run artefacts. PostgreSQL holds identity, entitlement, projects, the queue, the ledger index, quotes and orders: the "operational data" of VLEO's DELIVERY_PLAN.
- **The queue is a table.** Workers claim jobs with `SELECT … FOR UPDATE SKIP LOCKED`, so there is no second broker. A job runs in its own subprocess (§9.6: one flight software instance per process), with CPU, memory and time limits.
- **The rig pushes; the portal never reaches in.** The facility network makes outbound, authenticated connections to the portal to stream witness data. Nothing on the internet can address a rig host.

### 15.2 Roles

| Role | Can |
|---|---|
| `client_viewer` | see their tenant's projects, runs, quotes, orders, witness streams, evidence |
| `client_engineer` | as viewer, plus upload a case CSV or fill the case editor on screen, run the solver, switch products, run the release's scenarios and campaign types within quota, open result documents, request a design, upload an FMU, accept a quote, upload a PO |
| `sales` | all tenants' projects; price tables; issue quotes (with H-quote); accept POs |
| `engineer` | all projects read-only; run internal campaigns; the node library; send requests (node forms) and see their status |
| `designer` | design jobs; the internal catalogue view, candidates included; the design-request queue; send requests |
| `catalogue_owner` | decide to offer a catalogue candidate, or to retire a product (H14, with quality); the decision is recorded with their name, and the developer team writes it into the next catalogue release (§8.6) |
| `trainee` | download the MATLAB SILS twin (§10.8); nothing else, and no client data. A trainee is a member of the company's own tenant; nobody outside the company holds the role until D24 decides who may, and on what terms. |
| `test_operator` | schedule and run rig campaigns; the facility view; fault injection (internal) |
| `production` | upload each unit's measured descriptor at calibration, checked against the schema on upload (§7.4) |
| `quality` | the verification matrix, parity causes, deviations; sign certificates (H-cert) |
| `developer` | the requests inbox: every request, its file, its check report and its status; post replies (§5.11). Never content: a developer changes the software in the repository, not here. |
| `admin` | tenants, users, quotas; never content |

Every request is authorised against its tenant. A test suite tries every route as every role against another tenant's ids and expects refusal.

### 15.3 The order, as a state machine

```
enquiry -> project_open (case uploaded) -> sizing (solver) -> sils -> quoted --(client accepts, uploads PO)--> po_received
                                              \-> no_product -> design_requested -> (designer saves candidates -> added to the catalogue
                                                 by request in a release -> H14 decision recorded -> catalogue bundle published) -> sizing
  -> po_accepted (sales) -> building (production) -> calibrated (production, serials written)
  -> oils_scheduled -> oils_running -> oils_passed (test_operator)
  -> hils_scheduled -> hils_running -> hils_passed (test_operator)
  -> certifying -> certified (quality, H-cert) -> delivered -> in_flight
```

Any state can go to `on_hold` with a reason. A failed campaign goes back to its `_scheduled` state with a deviation record. Uploading a changed case, or choosing another product, after `quoted` voids the quote and returns to `sizing`, and the old quote stays on record. `no_product` is not a failure: it is the solver's honest answer, and it turns into a design request only when the client asks for one. Every transition is an audit-log row with actor, time and reason.

### 15.4 Data model

The tables: `tenant`, `app_user`, `membership (user, tenant, role)`, `session`, `project (tenant, title)`, `case (project, version, csv_path, csv_sha256, case_hash, schema, imported_at)` (a new upload is a new version; none is overwritten), `request (id, request_id, node, type, file_path, submitted_by, submitted_at, status, issue_url, last_reply_at)` (the inbox of §5.10.6; the file is the record), `decision (kind, subject, decided_by, role, decided_at, reason, record_path)` (every person's decision the software records: H14 promotions, retirements and bounds, H15 causes, panel sign-offs; `xtask decision record` reads the exported record), `solution (case, run, outcome, gap_report_path)`, `candidate (solution, product_id, tuned_set_hash, case_hash, classification, worst_margin, confirm_campaign_hash)`, `product (id, status, origin, class, file_sha256, bundle_version)` (the internal catalogue: the released catalogue's products, plus candidates the moment a portal design job saves them, or a designer uploads one from their store), `design_request (case, requested_by, state, product_ids)`, `design_job (id, class_or_case, toml_hash, state, saved_products)`, `campaign (project, case, product_id, scenario_id, scenario_hash, type, runs, hash, rung, status)` (a scenario is the release's, named by id and hash, never stored per project), `run (campaign, k, status, manifest_hash, artefact_path, result_path)`, `result (project, case, file_path, sha256, engine, kind, verdict, worst_margin, uploaded)` (every result document, made here or uploaded), `evidence_supply (project, row_id, value_si, rung, campaign_hash, runs, statistic, confidence)`, `parity_line`, `price_table (version, part_number, price, currency)`, `quote (project, candidate, hash, total, currency, catalogue_version, price_table_version, valid_until, document_path, state)`, `purchase_order (quote, document_path, received_at, accepted_by)`, `sales_order (quote, state, serials)`, `unit (serial, part_number, sales_order, descriptor_path, checked_at, recorded_by)`, `certificate (sales_order, package_hash, signed_by, signed_at)`, `fmu_upload (project, sha256, fmi_version, status)`, `quota (project, sils_runs_per_month)` (per project, as the layer-1 row states it), `comment`, `audit_log`.

Migrations are plain SQL files under `crates/adcs-portal/migrations/`, applied in order. A test applies them to an empty database, and another applies them then rolls them back.

### 15.5 Routes

Client-facing, JSON under `/api/v1`, server-sent events where marked:

```
POST /auth/login  /auth/logout  /auth/totp
GET  /projects                 POST /projects
GET  /case-template.csv                                             (the blank form)
GET  /projects/{id}            POST /projects/{id}/case            (a CSV upload, or the on-screen form, which writes the same CSV)
GET  /projects/{id}/case       GET  /projects/{id}/case.csv        GET /projects/{id}/case/report   (the import report)
POST /projects/{id}/solve      GET  /projects/{id}/solution        POST /projects/{id}/design-request
GET  /scenarios                (the release's scenarios and campaign types)
POST /projects/{id}/campaigns  GET  /campaigns/{id}                GET /campaigns/{id}/dashboard   (scenario id, type, runs)
GET  /projects/{id}/results    POST /projects/{id}/results         GET /results/{id}               (result documents: list, open an uploaded one, download)
GET  /runs/{id}                GET  /runs/{id}/stream   (SSE)       GET /runs/{id}/replay
POST /projects/{id}/fmu
POST /projects/{id}/quote-request                                 GET /quotes/{id}   POST /quotes/{id}/accept
POST /orders/{id}/po           GET  /orders/{id}                   GET /orders/{id}/witness (SSE)
GET  /orders/{id}/evidence     GET  /certificates/{id}
```

Internal, under `/api/v1/internal`, by role: price tables, quote issue, PO acceptance, rig scheduling, `rig/stream` (the facility's outbound push), unit descriptors (production), deviations, parity causes, certificate signing, design jobs and the design-request queue, the catalogue view with candidates, promotion and retirement decisions (recorded, H14), and:

```
GET  /api/v1/internal/forms/node/<node>      a node's form (internal roles only: layer 3 is restricted from clients)
GET  /api/v1/internal/forms/library          the node library, as a zip
POST /api/v1/internal/requests               send a filled node form: stored, an issue opened, status "received"
GET  /api/v1/internal/requests[/<id>]        the sender's requests and their status; every request for `developer`
POST /api/v1/internal/requests/<id>/reply    `developer` only: a reply, written into the returned file's history
GET  /api/v1/internal/downloads/matlab-sils  the latest release of the twin's zip, for internal roles and `trainee`
GET  /api/v1/internal/downloads/manual       the user manual
GET  /api/v1/internal/risk                   the Risk management conclusion, the register and each quarter's narrative (§5.13), read-only
POST /api/v1/internal/orders/{id}/units      production: one unit's measured descriptor, checked against the schema (§7.4)
GET  /api/v1/internal/orders/{id}/serials    the order's unit descriptors as one hashed folder, for the rig, fswcfg and eeprom (§10.5)
POST /api/v1/internal/decisions              a person records an H14 decision, a parity cause (H15) or a panel sign-off (§17.2)
```

No route changes a node, a scenario, a product file or any other part of the software. A request is stored and forwarded; the change is made in the repository, through intake (§5.11), and arrives with a release.

### 15.6 Client runs and uploaded models

- A client's SILS campaigns count against `quota`. The solver's screening costs nothing, and its tuning and confirmation run in the company's time, not the client's quota (D19 may change that); a client's own 500-run Monte Carlo costs 500.
- The case form on screen is the case editor of §8.3.3, served inside the page. It has the same keys, units, blank policies and checks as the CSV template. Submitting it writes a CSV and imports it through `adcs-case`, exactly as an upload does, so there is one path into the engine. The same page can be downloaded, filled offline, and uploaded later.
- A client picks a scenario of the release and a campaign type; a client never writes a scenario. A new kind of test a client needs reaches the company through sales.
- A client may upload a plant extension or a controller as an FMU (FMI 2.0 or 3.0, co-simulation). An FMU carries native code, so it runs only in a sandboxed worker: a separate user namespace (`bubblewrap` or `nsjail`), no network, a read-only root, and CPU, memory and wall-time limits. Only Linux x86-64 binaries are accepted, and a source FMU is compiled inside the sandbox. Whether to adopt a Rust FMI host crate or bind the FMI reference C headers is D16.
- Every SILS run a client starts is saved as a result document (§13.5) in the project, beside the case version it ran, and listed and downloadable there. The client can upload a result document they were sent, or saved earlier, to see it again in the portal without running anything: the portal files it beside its case, as the local store does (§13.5.4). A result made for a client leaves out layer-3 channels and restricted content.
- A client's FMU is the one input besides the case, and only in the portal. It extends the plant or replaces the controller for that client's runs; it never changes the software.
- A client never sees the reference flight software's source, the controller gains or the plant internals. They see their inputs, their runs' channels and metrics, and the evidence for their order.

### 15.7 Quote to purchase order

- **Price** comes from `price_table` (sales), never from the tree. The tree's layer-1 cost rows are the company's view of cost; the price table is what it charges.
- **A quote's identity** is `SHA-256` over the canonical JSON of: the case hash, the product id and file hash, the tuned-set hash, the kernel and graph hashes, the catalogue bundle version and hash, the price-table version, the currency and the validity date. A quote is a commercial record, so its hash is cryptographic. FNV stays the engine's cache key, as in VLEO.
- **Issuing a quote** is H-quote: a person in sales confirms the price and the export-classification check before the client sees it.
- **The purchase order** is a document the client uploads against a quote id and hash. Sales accepts it, and the order starts. A PO against a voided or expired quote is refused by name.

### 15.8 Security

- TLS at the proxy; `argon2id` password hashes; TOTP second factor for every account; server-side sessions in `HttpOnly`, `Secure`, `SameSite=Strict` cookies; CSRF tokens on every state-changing request.
- Tenant isolation is tested (§15.2). The audit log is append-only; the application role has no `UPDATE` or `DELETE` on it.
- Restricted content (D1 rows, fswcfg sections `0x05` and `0x06`, tuned-set values, other tenants' data) is excluded by the API layer and tested for (§14.6).
- Where the portal and client data are hosted, and what residency clients require, is D17.

### 15.9 Deployment

A first deployment is one server: reverse proxy with TLS, `adcs-portal`, two or more `adcs-worker`, PostgreSQL and the artefact directory, defined in `deploy/compose.yaml`. Every night, a database dump and an artefact sync go off the server. `docs/RUNBOOK.md` gains the restore procedure, and a monthly restore test is a scheduled task a person runs.

---

## 16. Faces and manuals

### 16.1 The faces read and run

The ported faces keep every read and run command and route they have, renamed (§3.3). Every command and route that wrote a sheet is removed (§3.4, F17). What a face may put into the software is a case CSV, a result document to view and, in the portal, a client's FMU. The software's content changes only through intake, in a developer's checkout.

**CLI (`adcs`).** VLEO's `campaign` subcommand already means "every stored case against one row", so the simulator's commands live under `sim` and never clobber it.

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

**Daemon routes (workbench).** `/v1/parity/<…>` already serves parity files in VLEO, so the ledger gets its own route. There is no write route and no `ADCS_ALLOW_WRITE`.

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

`xtask` is the developer team's, in a checkout. It keeps VLEO's reading and building commands (§3.4) and adds:

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

It documents every command, route and setting the software has, for users and developers, and says which is which. VLEO's `the_manual_is_true` test enforces it in both directions, and its parsers are extended to the `sim`, `result`, `case`, `rig` and `intake` dispatch. It is the source of the user manual's command reference (§16.4), so the manual can never name a command the software does not have.

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

**Checked.** `cargo xtask manual --check` (built in P1; `tools/manual_pages.py --check` is its stand-in in this package) regenerates `05_case_keys.md` from the case registry, the codes table in `developer/01_intake.md` from the checker and the rules table in `developer/08_derisking.md` from the ledger, the twin rules in `developer/09_twin.md` and §10.8.7 from the twin check, and fails on any difference; `cargo xtask explain check` checks every page's kind line, one-line answer and sections; it is green from P1. `tools/manual_check.py` (VLEO's) runs every command a manual page shows and compares the output it claims. The manuals are written for the finished software, so a page may show a command a later phase builds: every command block is tagged with the phase that builds it (`<!-- since P3 -->`, as the package's pages already are), and `manual_check.py` runs a block only once its phase is green. The command reference itself is `docs/manual.toml`, which the web face shows under Help → Commands.

---

## 17. Governance

### 17.1 Instruction files

- `AGENTS.md`: the standing instructions of every agent working in the repository, which are now two: the builder during the phases, and the implementation agent after (§17.4). It holds §3.2's five rules, §0.2's rules, and four sections: evidence rows (§5.5), scenarios and campaigns (§10), the rig's safety rule (§12.8), and intake (§5.11): "a node's content comes only from a passing request, through `intake write`".
- `intake/AGENT.md`: the implementation agent's own page: how to read a brief, the scope hook, and the list of things it never does.
- `areas/`: VLEO's six files, plus six:
  - `simulation.md` for `adcs-sim-core`, `adcs-sim`, `adcs-fsw-abi` and `matlab_sils/`: determinism (§9.7), "the plant never reads a clock", and "the twin follows the platform's equations; a difference is a ledger line, never a silent fix in one of them";
  - `rig.md` for `adcs-rig`, `rig/` and `devices/`: hardware interlocks, mocks and bring-up;
  - `portal.md` for `adcs-portal` and `adcs-worker`: tenant isolation, no engine in the portal, restricted content, no route that changes the software;
  - `evidence.md` for `adcs-evidence` and `catalogue/`: part status rules and the certificate;
  - `solver.md` for `adcs-case`, `adcs-config`, `adcs-tune`, `adcs-solve` and `designs/`: one supplier per value, blanks never guessed, tuned values restricted, candidates never shown to a client and never written to the repository by the software;
  - `intake.md` for `adcs-intake`, `forms/`, `manual/` and `tools/intake.py`: the checker is the authority, the page only helps; a received file's script never runs; interfaces are checked against the repository, never the form.

### 17.2 Human decisions

VLEO's H1–H9 are unchanged. H1 (a relation) and H2 (a test vector) are made in a node form by the engineer under "Checked by", and reviewed on the intake branch. H10 is defined, as F10 requires. Seven more follow:

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

These are added to VLEO's CONTRIBUTING table, which remains the only place the policy is stated:

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

VLEO worked with a fleet of seven agents, each in a lane: one drafted declarations, one filled holes, one recorded fixtures, and so on. The ADCS platform drops the fleet (§3.4). Declarations and test vectors now come from people, through node forms. What remains for an agent is the part a person should not have to type: turning a checked request into code.

**One general agent.** The implementation agent is Claude Code, started by a developer in a checkout, with one brief (§5.11.3). It is not specialised: the same agent implements a declared value, a relation with a new physics function, or a new node, because the brief says exactly what to do and where. It works under:

- `AGENTS.md` and `intake/AGENT.md`, its standing instructions;
- the scope hook in `.claude/hooks/`, which refuses a write to any path the brief does not name. `node.toml`, `fixtures.toml`, `versions.toml`, `versions/` and `derisk/` are refused to the agent's editor altogether: only `cargo xtask intake write` writes them;
- the brief, which names the node, the files, the steps, and the commands that prove the work.

**What proves its work is not the agent.** `intake verify` compares the node with the request field by field. The gate checks the HOLEs (F7's check 10b) and runs every test vector, which people transcribed from cited pages. A developer from the owner group reviews the branch. So an agent's mistake is caught by a check that does not share it.

**What it never does**, whatever a brief or a request says: supply an expected value; write a person's name; set or lower a risk level; widen a tolerance; skip a test; put a formula in a HOLE; edit outside its scope; push, merge or release. When a request cannot be implemented as written, it stops and says why, and the developer replies to the requester.

**Evaluated.** `cargo xtask intake selftest` runs the checker's 66 refusals, and the 82 seed forms through check, write and verify on a scratch tree freshly seeded from `plan/`. H10 requires it green for any change to the agent's instructions, hook, brief or model.

No agent promotes a product, confirms a node or signs anything. `adcs design` saves candidates as a tool run by a person or a scheduled job, with `origin = "designed"` and no name in `[promotion]`. `tools/validate_plan.py` and the gate refuse an `offered` product without a recorded person's decision (§8.6).

### 17.5 Commit scopes

They are derived from crate names as in VLEO, plus `catalogue`, `designs`, `scenarios`, `campaigns`, `rig`, `devices`, `fsw`, `plan`, `forms`, `manual`, `intake`, `matlab_sils` and `deploy`. An intake commit's scope is its node's crate, and its trailer is `Request: <request id>`.

---

## 18. Continuous integration

`gate.yml` keeps VLEO's jobs, renamed. Its `tooling` job keeps VLEO's explicit list of selftests. Not every script has one: `seed_tree.py` refuses arguments by design, `build_tree.py` takes only `--check`, and `check_seed_with_vleo.py` needs a VLEO checkout. So the job names each script it runs, as VLEO's does, and never globs `tools/*.py`. The jobs are: `gate`, `tooling`, `review` (advisory), `mutants`, `panels`, `excluded-faces`, `shipping-profile` (now also building `--profile user`, F4) and `no-std`. The `no-std` job extends to `adcs-sim-core` on `thumbv7em-none-eabihf`. It adds:

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
- VLEO's `mutate` and `bundle verify`.

`release.yml` keeps VLEO's prove → build → publish with its fail-closed approval, preceded by one release-preparation commit: `cargo xtask intake mark published --release <v>` for every node whose request the release carries, so the sheets that are built say `published` and each new version carries its release; and `cargo xtask derisk rollup`, which writes `derisk/rollup.toml` for the Risk management rows. Its artefacts are the CLI, the daemon, the FFI library, `adcs-rig` (Linux only), a container image with `adcs-portal` and `adcs-worker`, `adcs_sils_matlab_<version>.zip`, the node library, the user manual and the quarter's de-risking narrative. Its release notes list every request the release carries, by request id and node, with the belief each rested on, so each requester can find theirs.

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
