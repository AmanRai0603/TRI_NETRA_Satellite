# 2_spaceweather — solar & geomagnetic drivers (inputs to the density models)

> File-by-file Input/Process/Output map: [`CODE_MAP.md`](../CODE_MAP.md) at the repository root. Every `.m` file carries the same contract in its header.


Grouped by role:
- `solar/`        solar flux indices + conversions:
    - `get_f107.m` (F10.7), `get_f30.m`/`get_f30_cls.m` (F30), `get_omni2.m` (OMNI2 bundle),
      `f30_from_f107.m`, `f30_to_f107scale.m` (convert between F30 and the F10.7 scale).
- `geomag/`       geomagnetic activity: `get_gfz_hpo.m` (Kp/Hpo/ap, incl. `ap60`; GFZ JSON web service).
- `model_inputs/` driver bundle for JB2008:
    - `get_jb2008_indices.m` (SOLFSMY + DTCFILE for JB2008; caches in `jb2008_data/`),

**Inputs:** date range (internet needed for first fetch; then cached).

**Run sequence:** these are called by `4_comparison/run/run_comparison_study.m` and by `6_decay/decay_study.m`;
you rarely call them directly. If you do: pick the getter for the index you need, pass a date range,
get back a timetable.

**Outputs / gives:** timetables of F10.7, F30, ap/Kp/ap60, and the JB2008 index struct — the drivers the
models are evaluated with. `ap60` in particular is what the decay study overlays to explain storm-driven
sharp decay points (Fig 6d).
