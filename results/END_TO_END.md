# Both cases end to end, from the design database

**In one line:** ais_3u, ais_img_3u through the design loop and their campaigns with every engine run reading its inputs from `build/end_to_end/Design/design.tndb` alone; every number is held to the one flown from the files, and every row and KPI closure is evaluated (`tools/end_to_end.py`, `docs/END_TO_END.md`).

| Case | Steps | Runs from the database | Numbers compared | Numbers that differ | Rows with a value | Closures answered | Closures blocked |
|---|---|---|---|---|---|---|---|
| `ais_3u` | the design loop (ok), its campaigns (ok) | 45 (not: 3) | 468 | 0 | 43 of 712 | 4 | 34 |
| `ais_img_3u` | the design loop (ok), its campaigns (ok) | 47 (not: 3) | 555 | 0 | 48 of 712 | 4 | 34 |

Each closure's answer, or the row that blocks it: `results/EVALUATION.md`. Each requirement's check: `results/TRACEABILITY.md`.
