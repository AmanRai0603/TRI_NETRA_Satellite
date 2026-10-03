# 07_examples/

Everything here **runs**. Nothing here is **validated**.

| file | what it is |
|---|---|
| `TEMPLATE_16U.m` | the 16U end-to-end: every model on, per-force CSVs to `force_data/`, the model-I/O figure. Start here. |
| `EXAMPLE_16U.m` | a design sweep over the same 16U (altitude/Cd/area bands). |
| `TEMPLATE_propagation.m` | the minimal propagation: no satellite, no validation, just the engine. |
| `show_16U.m` | the plot suite for the two 16U scripts. |

## The trade these make, stated plainly

`06_validation/realsat/validate_OD.m` runs CHAMP: **measured truth, no geometry.**
These run a 16U: **real geometry, no measured truth.**

That is not a defect of either. It is the shape of what exists: `sgeom.sat16u` is the
only real geometry in the toolbox, so box-wing SRP/ERP can only run here without
assuming a shape -- and no instrument ever flew on this spacecraft, so nothing here
is checked against reality.

**Every number these files print is a model talking to itself.** That is genuinely
useful -- it is how you watch a model behave without a measurement arguing back --
and it is not evidence about the world. For evidence, use `validate_OD`.

## Why they were in the root

They were, until round 24. `setup_paths.m` stays there because it is the one file you
call before anything else; the rest are examples and belong with the examples.
