
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
