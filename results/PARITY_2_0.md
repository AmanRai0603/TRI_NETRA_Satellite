# The parity gate: today's design gives today's answers

**In one line:** every check holds: today's design, built from the drive (`tools/design_build.py`), read alone by the engine (an empty data folder), gives the repository's own inputs, parameter blobs, runs, campaigns and evaluation (`tools/parity_2_0.py`, `docs/PLAN_2_0.md` S4).

Design: `tests/regression/design.tndb`.

| Check | What is held | Equal | Differ |
|---|---|---|---|
| inputs | every engine input file and every case line, byte for byte | 167 of 167 | 0 |
| layout | the parameter table from its nodes, and the C and Rust it writes | 152 of 152 | 0 |
| blobs | every scenario's parameter blob (adcs-fswcfg/1) | 48 of 48 | 0 |
| runs | every stored scenario run, flown again from the design: inputs and metrics | 48 of 48 | 0 |
| campaigns | every run of the 8 stored campaigns, its dispersions drawn again: draws and metrics | 4516 of 4516 | 0 |
| evaluate | every row and closure of results/evaluation.json (a node of the dissolved group `case` names its new group) | 2520 of 2520 | 0 |
| oils | soft OILS on QEMU (Cortex-M4F firmware), from the files and from the design: metrics and timing | 2 of 2 | 0 |
| twin | the MATLAB twin flown from the data folder the design exports, against its stored run | 1 of 1 | 0 |

The design loop and each case's campaigns flown end to end from the design: `results/END_TO_END.md`. The health map, range verdicts and tornadoes of the same design: `results/HEALTH.md`.

The twin's check was flown on its own (`Gate.twin`, the same code) after the full run's stopped: since S7.17 the twin flies the blob the engine builds (`adcs params`), and the gate's copy of the twin had no engine beside it (asils.util.engine); the gate now tells it which (ADCS_BIN, tools/parity_2_0.py). Every other check is the full run's (`python3 tools/parity_2_0.py tests/regression/design.tndb --oils detumble_ais fine_hold_img --twin detumble_ais --jobs 2`, S7.19).
