# The parity gate: today's design gives today's answers

**In one line:** something differs: today's design, built from the drive (`tools/design_build.py`), read alone by the engine (an empty data folder), gives the repository's own inputs, parameter blobs, runs, campaigns and evaluation (`tools/parity_2_0.py`, `docs/PLAN_2_0.md` S4).

Design: `tests/regression/design.tndb`.

| Check | What is held | Equal | Differ |
|---|---|---|---|
| inputs | every engine input file and every case line, byte for byte | 166 of 166 | 0 |
| layout | the parameter table from its nodes, and the C and Rust it writes | 151 of 151 | 0 |
| blobs | every scenario's parameter blob (adcs-fswcfg/1) | 48 of 48 | 0 |
| runs | every stored scenario run, flown again from the design: inputs and metrics | 48 of 48 | 0 |
| campaigns | every run of the 8 stored campaigns, its dispersions drawn again: draws and metrics | 4516 of 4516 | 0 |
| evaluate | every row and closure of results/evaluation.json (a node of the dissolved group `case` names its new group) | 1496 of 1500 | 4 |
| oils | soft OILS on QEMU (Cortex-M4F firmware), from the files and from the design: metrics and timing | 2 of 2 | 0 |
| twin | the MATLAB twin flown from the data folder the design exports, against its stored run | 1 of 1 | 0 |

**evaluate, what differs:**

- ais_3u: l3_dist_row_07
- ais_3u: l3_dist_row_08
- ais_img_3u: l3_dist_row_07
- ais_img_3u: l3_dist_row_08

The design loop and each case's campaigns flown end to end from the design: `results/END_TO_END.md`. The health map, range verdicts and tornadoes of the same design: `results/HEALTH.md`.
