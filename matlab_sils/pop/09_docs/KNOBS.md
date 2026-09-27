# The knobs, segregated by what they are

A knob is only useful if you know **which kind** it is. These four kinds behave
completely differently under a sweep, and mixing them in one list is why sweep tables
get over-read.

---

## 1. INDEPENDENT — change one, the answer moves, and it means something

These are not degenerate with anything else. A change here is a real, separable claim.

| knob | where | what it controls |
|---|---|---|
| `DRAG_MODEL` | validate_OD, TEMPLATE_16U | cannonball / sentman / dria / cll / sesam |
| `ATMOS` | validate_OD, TEMPLATE_16U | exponential / nrlmsise / dtm2020 / dtm2020_research / jb2008 |
| `SRP_MODEL` | validate_OD, TEMPLATE_16U | cannonball / boxwing |
| `ERP_MODEL` | validate_OD, TEMPLATE_16U | knocke / simple / ceres / boxwing |
| `ATTITUDE` | validate_OD, TEMPLATE_16U | none / ram / measured |
| `GEOMETRY` | validate_OD | plate / boxwing / aref_box |
| `GRAV_FIELD`, `degree` | validate_OD, TEMPLATE_16U | which field, to what degree |
| `FORCES.<x>.on` | both | is the force in the sum at all |

**`DRAG_MODEL` is the one worth your time on metric [3].** Not because the GSI
formulations differ much, but because `Cd(t)` has a **shape** with altitude and local
time that a scalar cannot mimic — so it is *not* degenerate with density the way
`SC.Cd` is.

---

## 2. DEGENERATE — change one, the answer moves, and it means almost nothing

On the **position** metrics [1]/[2], `rho * Cd * A / m` is **ONE number**. The orbit
cannot tell you which factor was wrong.

| knob | degenerate with |
|---|---|
| `SC.Cd` | density, `SC.Aref`, `SC.mass` |
| `SC.Aref` | density, `SC.Cd`, `SC.mass` |
| `SC.mass` | the same product |

Sweep them to **see** the degeneracy, not to fit through it. Metrics [3]
(accelerometer) and [4]/[5] (density) break it, because they see the factors
separately.

---

## 3. CONDITIONAL — read by some models, ignored by others

A flat sweep here is **correct behaviour**, not a bug. `sat.reads(cfg)` prints which
of these your current toggles actually consult, before you spend the arcs.

| knob | read by | IGNORED by |
|---|---|---|
| `SC.Cd` | drag cannonball | sentman/dria/cll/sesam — they **derive** `Cd(t)` |
| `SC.Cr` | srp cannonball, erp knocke/simple/ceres | srp/erp boxwing — facet optics instead |
| `SC.Aref` | all cannonballs | panel + boxwing — the facets rule |
| `GSI.aT` | **sentman only** (you type it) | dria/sesam — they **derive** aT from atomic O |
| `GSI.sig_n`, `GSI.sig_t` | cll only | everything else |
| `GSI.Tw` | all panel models | cannonball |
| `BOX_ASPECT` | `GEOMETRY='aref_box'` only | plate, boxwing |
| `ERP_RINGS`, `ERP_SEGS` | erp boxwing/knocke | srp, drag |

---

## 4. NUMERICAL — these MUST NOT move the answer

| knob | rule |
|---|---|
| `INTEG_METHOD` | a correct setup gives the same residual on every integrator (~2 cm apart) |
| `INTEG.rtol` / `.atol` | tighten until the answer stops moving; it should already have |
| `OUT_DT` | output spacing must not touch the dynamics |

**If one of these moves the answer, every other row in your table is noise.** That is
a step-size/tolerance problem, not physics, and nothing else is worth reading until it
is fixed.

---

## Verifying this list rather than trusting it

`11_compare/verify_sweep_matrix.m` sweeps **every** knob twice against a realistic 16U
and checks whether the non-gravitational sum moved, against the expectation documented
here. It caught `FORCES.drag.corotate` doing **nothing** — the panel models were
passed a hard-coded omega, so the knob was documented, sweepable, and silently ignored.
In a sweep table that reads as *"co-rotation doesn't matter at this altitude"*:
plausible, citable, false.

Run it after any change to a model or a knob.
