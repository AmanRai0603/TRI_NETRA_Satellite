
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
