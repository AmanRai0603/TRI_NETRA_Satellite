
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

The campaign runner expands a campaign into a run list, hashes it, and writes it before running anything. A campaign that is interrupted resumes from its list. A run that fails to start is recorded as refused, with its reason, and never silently dropped ("a sweep records refused points; it never drops them").

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

`campaign_hash = hash(scenario hash, case hash, product id, tuned-set hash, catalogue bundle version and hash (or `unpublished` and the working copy's content hash, §7.2), serial descriptors hash (or `none` before a unit is built), flight software build id, engine kernel hash, adcs-sim-core hash, seed, run list hash)`. Two campaigns with the same hash are the same campaign ("two runs with the same chain hash are the same run"). The evidence row's Reproducibility factor is 4 only when every one of those is present and every bundle verified.

### 10.7 Parity references are not fixtures

`[parity_reference]` holds numbers another tool published: IDMAS v2 §13's MATLAB simulation, or the company's MATLAB SIL. `tools/sil_parity.py` does it for runs. It compares a campaign's metrics with the scenario's parity reference and writes the difference and its cause to the parity ledger (§14.2). The rule: a MATLAB number is a second opinion and never a fixture. "Where the two tools disagree, the disagreement is recorded with its size and its reason rather than tuned away."

The first expected parity result is informative either way. IDMAS v2 §13 reports 0.044° RMS and 210 s to 0.1°, with 10–30 % feed-forward errors, a friction model 20 % low and a pump efficiency of 10 %. The loop engine with the reference flight software will land near or far from that, and the ledger says which and why.
