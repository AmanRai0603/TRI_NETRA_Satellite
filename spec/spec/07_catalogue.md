
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

