# 11_compare — which setup is least wrong?

These are **scripts**, not functions: open one, edit the settings block at the top,
run. All the workflow is visible in one place. Pure comparison — no bias fitting,
no correction, anywhere.

| script | question | truth | you edit at top |
|--------|----------|-------|-----------------|
| `compare_OD.m` | of N force configs, which is least wrong on a real satellite? | ITSG / TU Graz (orbit + accelerometer + density) | `SAT`, `DATE`, `FORCES`, `SWEEP` |
| `compare_density.m` | how close is my density to measured, per model? | TU Delft measured density | `SAT`, dates, `MODELS` |

Helpers: `show_compare_OD.m` (figures), `getm.m` (ragged-table field access),
`save_results.m` (timestamped output dir under `data.root()/results/`).

---

## compare_OD.m

`validate_OD` asks *"is my force model right for THIS setup?"*. `compare_OD` asks
the next question: *"of N setups, which is least wrong?"* — the same four metrics,
evaluated once per sweep value, against the same arc and the same epochs.

Both call **`validation.od_metrics`**. There is exactly one implementation of each
metric, so the two scripts cannot drift apart and disagree.

### The sweep (same pattern as `EXAMPLE_16U` section 9)

```matlab
SWEEP.on     = true;
SWEEP.knob   = 'FORCES.drag.atmos';
SWEEP.values = {'exponential','nrlmsise','jb2008','dtm2020'};
```

One knob, a list of values, one propagation per value. Reference products are
fetched **once** and shared — ITSG products are immutable, so re-reading them per
config would be waste, and would let a transient network failure quietly change
the truth mid-sweep. A config that dies is a **row in the table**, not an abort.

Legal knobs live in `validation.sweep_knob` — an explicit switch, no `eval`, so a
typo fails loudly instead of silently creating a struct field nothing reads.

### Reading the table honestly

```
 SC.Cd          |   rdo|3D|    rdo dv |   kin|3D| |   acc gap  ratio |     rho
 2.4            |     0.948   0.00107 |     0.955 | 1.244e-08  1.090 |   1.000
 3              |     3.785   0.00427 |     3.809 | 4.977e-08  1.362 |   1.000
 3.6            |     6.621   0.00748 |     6.664 | 8.710e-08  1.634 |   1.000
 REFERENCE FLOOR (kinematic - reducedDynamic) = 0.034 m RMS
```

Four things this table is telling you, in decreasing order of how often they get
missed:

1. **The reference has a floor.** `refSpread` (kinematic − reduced-dynamic) is the
   reference's *own* uncertainty, a few cm for ITSG. Two rows closer than that are
   **not distinguishable** and the ordering between them is noise. Do not tune
   below the floor.
2. **`rdo|3D|` is the weakest metric, not the strongest.** The reduced-dynamic
   orbit was itself produced with a force model plus empirical accelerations that
   absorb exactly the mismodelling you are trying to measure. A config can win on
   `rdo|3D|` by resembling TU Graz's model rather than by resembling physics.
   `acc gap` and `rho` are **measurements**. Prefer them when they disagree.
3. **Cd and density are degenerate.** They enter the equations of motion only as a
   product, so position alone cannot separate them — which is the whole reason the
   accelerometer and density comparisons exist. The table above is that
   degeneracy, drawn: `rho` sits at 1.000 while `rdo|3D|` swings 0.9 → 6.6 m. The
   atmosphere never changed; only Cd did. Sweeping `SC.Cd` will happily "improve"
   `rdo|3D|` while making the physics worse.
4. **One arc is one sample.** Three hours on one day at one solar activity level
   does not rank atmosphere models — it tells you what happened on that arc. Sweep
   `DATE` before believing any ordering. The spread *across dates* is your real
   error bar; the spread across models on one date is not.

### What the sweep found on its first run

`SC.Cd` returned three identical rows. Cd was not reaching the drag model:
`forces/drag.m` read it from `cfg.forces.drag.Cd`, but every script writes Cd into
`cfg.spacecraft.Cd` and then replaces `cfg.forces` wholesale with a `FORCES` struct
that has no `Cd` field. So it silently fell back to the built-in 2.2 — CHAMP
propagated at 2.2 instead of its catalog 3.0, a 27% drag error, while
`config.report` printed `Cd : 3` (the report reads `spacecraft`; the physics read
`forces.drag`). `EXAMPLE_16U`'s documented `SC.Cd` sweep knob was a no-op for the
same reason.

Fixed at the root: `forces/drag.m` now resolves Cd as
**`forces.drag.Cd` → `spacecraft.Cd` → 2.2**. Fixing it there rather than in each
script means the next script somebody writes cannot miss it.

> **That fix was only half of it, and the other half was still live.**
> `config.defaultConfig` pre-seeded `cfg.forces.drag.Cd = 2.2`. Since that is the
> *first* entry in the precedence chain, the override was **always present**, so
> the `spacecraft.Cd` fallback could never fire for anyone building a config the
> natural incremental way:
>
> ```matlab
> cfg = config.defaultConfig();
> cfg.spacecraft.Cd = 3.0;     % silently ignored -- physics ran at 2.2
> ```
>
> It only *looked* fixed because the three master scripts (`EXAMPLE_16U`,
> `compare_OD`, `validate_OD`) replace `cfg.forces` wholesale with a `FORCES`
> struct that has no `Cd` field — so the fallback fired for them and for nobody
> else. `config.report` still printed `Cd : 3` throughout, because report reads
> `spacecraft` and the physics reads `forces.drag`.
>
> Caught by `make_OD_fixtures`, which builds a config the natural way: its
> fixtures encoded Cd 2.2 while `compare_OD` propagated at 3.0, and `[3]` came
> back at **ratio 1.362 = 3.0/2.2** instead of 1.000. A round-trip whose answer
> you know in advance turns a silent 36% drag error into an arithmetic tell.
>
> `defaultConfig` no longer seeds `Cd`. The default lives in `drag.m`'s fallback,
> which is the one place that should own it. **Re-baseline again**: any run made
> between the two fixes used 2.2 wherever the config was built incrementally.

> **This changes existing results.** Any run that relied on the silent 2.2 will now
> use the real Cd. That is the point, but re-baseline before comparing to old
> numbers.

### Outputs (under `data.root()/results/<tag>/`)

`compare_OD.csv` (the full table, one row per config) · `compare_OD.mat` ·
`decisions.txt` · `compareOD_{ranking,rtn_vs_time,acc_gap,density,summary}.png`

---

## compare_density.m

Model density vs TU Delft measured density along the real track, for the models you
select. Independent of `compare_OD`: no orbit propagation, no force model — it
evaluates the atmosphere directly at measured points, so it isolates the
atmosphere from Cd entirely. Use it when `compare_OD` metric (4) and metric (1)
disagree.

---

## Gotchas that will cost you an afternoon

- **`GRAV_FIELD='default'` silently caps at degree 6.** Sweep
  `FORCES.gravity.degree` over `[4 20 70]` with the default field and you get three
  identical rows. That sameness *is* the tell. Use `'EGM2008'` (downloads once).
- **`atmos='nrlmsise'` needs the MATLAB Aerospace Toolbox** (`atmosnrlmsise00`).
  Without it that row fails — visibly, as a `FAILED` row, which is intended.
- ~~`atmos='dtm2020'` returns NaN~~ **RESOLVED** — it was a NaN-poisoned `Kp` in
  `atmos.spaceweather`, not a DTM2020 fault. See `09_docs/DRIVERS_AUDIT.md`.
- **The atmosphere sweep was scientifically void before that audit**: JB2008's index
  wire was never connected, NRLMSISE had no storm response and the wrong F10.7 lag.
  All fixed; all change results. Re-baseline.
- **Run the `INTEG_METHOD` sweep once, first.** A correct setup gives the same
  answer on every integrator. If it does not, that is a tolerance/step problem and
  every other row in every other table is meaningless until it is fixed.
