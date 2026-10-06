
---

## 5. The ADCS tree

### 5.1 Four layers, one door between each

| Layer | Holds | Rows | Crate |
|---|---|---|---|
| 1 company | case intake, the catalogue, commercial, the order lifecycle, the facility, standards, supply, heritage, risk management | 133 leaves in 42 groups | `adcs-mod-management` |
| 2 the satellite's ADCS | the pointing service asked for, the fitted hardware, mission and orbit, the satellite as the ADCS sees it, the ADCS subsystems at system level, verification | 194 leaves in 49 groups | `adcs-mod-system` |
| 3 subsystem | 14 subsystem layers (12 for the ADCS, 2 for its test rigs), plus the closure addition | 368 rows, plus 38 closures and the closure layer's interface row | `adcs-mod-<sid>`, `adcs-mod-closure` |
| 4 the run | what one evaluation or one campaign produced | — | — |

Every case reaches layer 2 through one row, `Satellite ADCS` under Case intake; the note "the door into this case's engineering layer" makes it cross to `sys_satellite_adcs`. There is exactly one door, and `tools/validate_plan.py` refuses a second. Each subsystem reaches layer 2 through its one `l3_<sid>_interface` row. `tools/validate_plan.py` checks the shape. Seeding it (`install`, `LAYER3_SOURCE` and `install_edges`, §5.7) wires 46 relations, 238 derivation edges and 23 contribution edges, skips none, creates one crossing, and passes all 14 layer-3 target assertions. In all, 734 node sheets once seeded: 327 in layers 1 and 2, 368 in the subsystem layers, and 39 in the closure addition.

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

No customer is a branch of this tree. A tree that carries its reference customers as layer-1 groups does not scale past a handful, so here every customer is a **case**. It is uploaded as one CSV in the fixed format (§8.3), imported into the case store, and runs through the single door (CD-06 §33, "one architecture + case id"). The reference cases are four such files in `plan/cases/`. The two defaults are one 3U satellite with two missions: `ais_3u` (AIS, 10° pointing) and `ais_img_3u` (AIS and imaging, 0.01° pointing) (§8.9). The other two are `ref_c2_150kg`, a 150 kg bus, and `ref_c3_12u`, a 12U not yet stated.

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

An analysis row is narrower than a contribution edge. `tree.json`'s KE edges say which variables feed a KPI, which is the coverage graph, and several evidence-only KPIs keep theirs: settling time feeds rate stability, for instance, without answering it. Only `plan/kpis.toml` says what answers a KPI.

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

Each layer holds its interface row, one required and one achieved row per target (the seeder derives these), and `rows − 1 − 2·targets` internal rows labelled "to be named". The internal budget is 166 rows across the fourteen layers. The fluid-ring and control layers get the most, 15 each. IDMAS v2 §03–§07 already names the ring's internal relations: pump pressure per stage, conduction and induction pump laws, Reynolds number, turbulent loss, freeze and thaw. §12 names the control stack's: the split projection, allocation weights, dump gain and the mode laws L1–L6. The two rig layers hold the lab's own models, which the lab twin (§12.6) runs: the cage's coil and field-error model, the bearing's residual-torque and drag model, the stimulators' rendering and latency, and each test stand's measurement model.

### 5.5 Closures, and rows that only evidence can answer

Every KPI is closed by evidence, and the 16 with an analysis row are also closed by analysis: 38 closures.

- **The analysis closure**, `kpi_<slug>_analysis` in `l3_x_closure`, compares the requirement row with the KPI's analysis row (`plan/kpis.toml`). It runs whenever the tree does. `<slug>` is the `slug` field of `plan/kpis.toml`, which is the seeder's `slug()` of the KPI's label: `kpi_absolute_pointing_error_ape_analysis`, for example.
- **The evidence closure**, `kpi_<slug>_verified`, compares the requirement row with the achieved row (`p1a_0` and so on). An achieved KPI row is an **evidence row**. Only a campaign can give it a value.

Both closures use `mission::closure(req, ach, Sense::AtMost | Sense::AtLeast)` exactly as every KPI row does. The requirement row is written as `kind = "declared"` with a top-level `sense` (the convention for written requirements; gate check 7d).

**Who writes the closures.** The seeder writes all 38 in full, and every requirement row's shape with them, from `plan/kpis.toml` and `plan/case_inputs.toml`, with the seeder's `KPI()` (§0.2 rule 6). A closure's content is fixed by the KPI list; nothing in it is a person's statement, so it needs no form. The seeder writes each closure `specified`, and the gate's closure checks (7d, 7e) and a `cargo xtask intake mark verified --closures` at the end of seeding make it `verified`, so closures run from P1. It does not wait for a requirement value, because the value comes from each case, not the sheet.

- inputs `req` and `ach`, one step, and the hole `Ratio::new(mission::closure(req.get(), ach.get(), mission::Sense::AtMost).margin)`, with `AtMost` or `AtLeast` read from the requirement's `sense` (gate 7e);
- a fixed `[theory]` text stating that a closure compares achieved with required in the requirement's sense;
- bounds −100 to 1000, each with its reason.

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

- **Seeding** leaves every group's `cases` list empty, which means every case. Which groups are in play depends on the product, not the case, so it is decided at run time from the counts.
- **The face** hides a group that is not in play. It filters by tags as F14 describes, so the same rule works for the portal's cases too.
- **The resolver** answers `NotFitted { slot }` for every `product` or `tuned` row the candidate does not supply: the per-unit rows of an empty slot, and a tuned row no algorithm of the product sets. `NotFitted` is not a number. A row that reads it answers `NotFitted` too, naming the slot, so "knowledge error with the star tracker" on a product with no star tracker says so rather than answering with zero noise. The one exception is the zero-answer rule below. A sheet may declare `zero_when_absent = ["cf_1"]`, naming the count rows it reads. When those counts are zero, the `NotFitted` inputs whose hardware tags match those counts contribute exactly zero. This is an H7 kernel extension, with gate check 7h: a `zero_when_absent` row must read the counts it names. `ge_5`, knowledge error with the sensors fitted, uses the same mechanism: it declares `zero_when_absent = ["cf_4"]`, so with no star tracker it reads the magnetometer-and-sun knowledge, and a coils-only product still has an AKE to judge.
- **A run without a product** (the tree alone, as P1's acceptance runs it) reads the `product` and `tuned` rows' sheet values. Those are the reference configuration, IDMAS V2 as the seed content states it for the reference 3U satellite (source `adcs_ref_c1`), and the run says so. A candidate run replaces every one of them (§8.2).

**The zero-answer rule.** A row that describes one unit answers whatever is fitted: one ring's momentum at cruise speed, one slew's propellant if thrusters did all of it. A row that totals, aggregates or feeds a KPI reads the relevant count and answers exactly zero when it is zero. That covers every authority row, every budget total, and propellant per year. Such rows have `lower = 0` with the reason "zero is a real answer: … is not fitted", and their physics function returns zero at count zero. So a family without thrusters reports zero propellant per year, not the three grams a year a 500 kg bus's thruster arm would imply. The gap pass notes a count-fed row whose lower bound is above zero.

### 5.7 Seeding the tree

The seeder is `tools/seed_tree.py`, with its maps in `tools/plan_rows.py`.

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

With these maps the node ids come out exactly as recorded in `plan/expected_node_ids.json`, and P1 checks that they still do.

`tools/seed_tree.py`:

- the root label is "ADCS products and test facility";
- it has no `REPARENT`, no `amend()` calls and no `l3_x_envorbit` layer;
- it has the `l3_x_closure` layer, which the seeder gives an interface row, `l3_x_closure_interface`, and generate the 38 closure rows of §5.5 in it, seeded;
- `SOURCES` come from `plan/seed_content.toml [[source]]`; later sources arrive through intake, each from a request's `new_sources`;
- the seeder writes no cases. Drop `CASES` and the case-writing half of `emit_supporting()`. After seeding, `adcs case import plan/cases/*.csv --into cases/` writes them through the one importer the portal also uses (§8.3), so there is one implementation of the format;
- the ADCS tree declares no cycle yet. Remove `CYCLE`, and remove the code in `emit_supporting()` that writes an `[[iterate]]` block into every case from it. `layers/cycles.toml` holds only its header comment;
- `owner_crate()` returns `adcs-mod-management`, `adcs-mod-system` and `adcs-mod-<sid>`. `CRATE_ALIAS` comes from `crate_skeleton.ALIAS`, which is set to `{"x_closure": "closure"}` (§3.4);

`seed_tree.py` still refuses to run on a tree that has a published sheet. It runs once, and what it writes is the tree's shape: every row's identity, place, owner, kind and the edges the tree declares, with no content. The one exception is the closures and requirement rows of §5.5, which the KPI list fixes. Every other seeded row answers `NotRun` until its content arrives through intake (§5.8).

**Changing the shape later** is the developer team's own work, never a form's. A new group is added with `tools/seed_tree.py --add-group <id> --under <parent>`, which writes that one group into `layers/` and nothing else, and refuses a group that exists. It is reviewed like any layer change, and the group's nodes then arrive as new-node requests (N01 accepts a group `layers/` declares, even with no nodes yet). A full reseed is possible only before any sheet is published, as the seeder enforces.

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

**States.** A row moves through these states, and intake decides which:

| State | Means | Reached by |
|---|---|---|
| `seeded` | shape only; answers `NotRun` | seeding |
| `specified` | content written from a request; not yet proven | `intake write` |
| `verified` | the gate passes, the regenerated artefacts match, every test vector passes, and `intake verify` found the sheet equal to its request | `cargo xtask intake mark verified <request>`, run by the intake CI job when verify and the gate pass on the branch |
| `published` | reviewed, merged and released | `cargo xtask intake mark published --release <version>`, run by `release.yml` for every node whose request the release carries |

`state` is the one field besides the content that intake writes, and only through `mark`. `mark` never lowers a state, so a confirmation of a `published` node leaves it `published`. `intake verify` never compares it.

Only `verified`, `published` and `deprecated` rows run (`State::runnable`).

**Attestation.** A node form has a field "Checked by": the engineer who has checked the relation against its source, and the values, and stands behind them. `intake write` copies that name, with the date and the request id, into `[maths] confirmed_by` and, for a declared row, `[value] confirmed_by`. Nothing else ever writes a name there: not the builder, not the implementation agent, not a script (rule 2). A request with the field empty writes `UNCONFIRMED · via <request id> · awaiting a person`. Seed forms always leave it empty, because the package's author is not a person who can attest.

A relation or value with nobody's name against it is not a failure; it is honest:

- the gap pass notes "the value has nobody's name against it";
- `xtask ready` holds the row;
- the row's own slot scores Mathematics 1 (for a relation) or InputPedigree 1 (for a value), so every number that reads it carries that low score and a client sees it.

It becomes somebody's when an engineer sends a node form of type **confirm**: no change, their name under "Checked by", and their reason. The checker refuses a confirmation that changes anything (F05). A confirmation needs no code and no agent: the checker writes `confirm.md` instead of a brief, and a developer runs `cargo xtask intake write request.json`, which in confirm mode writes only `[maths] confirmed_by`, `[value] confirmed_by` and `[request] last`, leaves the state as it is, and is verified and reviewed like any request.

**What credibility a seeded-then-specified row shows.** Take `gf_7` after its seed form is verified, scored by `credibility.rs`:

| Factor | Score | Why |
|---|---|---|
| Mathematics | 1 | the relation has nobody's name against it |
| Assumptions | 3 | one assumption |
| Verification | 4 | the row is verified |
| Validation | 2 | tier B, test vectors pass |
| InputPedigree | 1 | it reads UNCONFIRMED declared values |
| Uncertainty | 1 | inherited: its declared inputs score 1 under the proxy, and the rollup takes the minimum |
| Understanding | 3 | one step |
| Reproducibility | 4 | data ok |

The lowest is 1, shared by Mathematics, InputPedigree and Uncertainty. Ties go to the earliest factor, so Mathematics governs. A client looking at a margin in the portal therefore sees that it rests on a relation and values nobody has confirmed, until an engineer confirms them by form.

### 5.9 Owners

`CODEOWNERS` is generated from each sheet's `owner` (F9). The owners are `systems`, `environment`, `sensing`, `actuators`, `gnc`, `avionics`, `verification`, `sales`, `programme`, `facility` and `quality`. Mapping each to a GitHub team is decision D12. The generator's fixed header gains the new crates (§4), `catalogue/`, `scenarios/`, `rig/`, `forms/`, `manual/` and `intake/`.

Each owner maps to a reviewer group inside the developer team (D12). That group reviews the intake branches of the nodes it owns (§5.11, step 4): that the implementation is what the form asked for, that the HOLEs compose physics functions and nothing else, and that the tests and downstream nodes pass. Whether the relation itself is right is not the reviewer's to assume. It is attested in the form by the engineer under "Checked by" (§5.8), and a relation nobody has checked is released as UNCONFIRMED, visibly, until someone does.
