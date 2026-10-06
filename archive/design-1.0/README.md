# 1.0.0's design sources, archived

**In one line:** the files 1.0.0 kept its design in. They are history now. The design lives in the database
(`Trinetra Database/groups/*` on the shared drive), and the repository builds and tests against one copy
of it, `tests/regression/design.tndb` (`docs/PLAN_2_0.md` S3 and S4).

| folder | was | is now |
|---|---|---|
| `spec/` | the platform specification: the plan (tree, KPIs, case inputs, units, the relations' registry and worked examples), the relations in pseudocode, the de-risking ledger, forms, manuals and the spec tools | the plan and the relations are lookup and method blocks of the design; every file the code still reads is held whole and read from the regression copy (`tools/from_design.py`) |
| `catalogue/` | parts, products, algorithms, modes, components, classes, families, dispersions (TOML) | lookup blocks of the `catalogue` group; the engine's JSON is generated from them |
| `scenarios/`, `campaigns/`, `trades/` | the 48 scenarios, 8 campaigns and 12 trades (TOML) | case files (`cases/*.tncase`) |

Not here because they are still read in place, and are now **generated from the design** and checked
against it (`python3 tools/from_design.py --check`):
- `matlab_sils/data/` (the engine's inputs) and `matlab_sils/cases/` (the cases it flies);
- `fsw/params/params.toml` (the flight software's parameter table);
- `fsw/pseudocode/03`–`09` (the flight algorithms and their notes).

They stay generated until the flight build (S6) and the engine build (S7) generate code from the nodes.

**Who still reads this folder.** Only the 1.0.0 tools that made the conversion: `tools/seed_design.py`,
`tools/carry_over.py`, `tools/convert_2_0.py`, `tools/export_catalogue.py`, `tools/node_catalog.py` and
`tools/design_rows.py` (through `common.V1`), and the tests of the 1.0.0 node and group files, which use it as
their fixture until the example group (`tests/fixtures/`, S8) replaces it. The interpreter's Rust parity test
reads the relations' pseudocode here; `tests/test_from_design.py` holds that text to the design's.

**Never edit it.** A change to the design is made in the application and released by its group.
