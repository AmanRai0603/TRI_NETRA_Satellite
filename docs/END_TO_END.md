# Both cases end to end, from the design database

**In one line:** `ais_3u` and `ais_img_3u` go through everything, the design loop, their campaigns, every row of the design and every KPI closure, with every engine run reading its inputs from `design.tndb` alone, and every number held to the one flown from the files (`tools/end_to_end.py`, `tools/evaluate.py`, `docs/RELEASE_PLAN.md` P13).

## Say it simply

The cookbook moved into one binder. To be sure nothing was lost in the move, the whole menu is cooked again from the binder alone, every dish weighed against the one cooked from the old loose pages, and for every dish on the menu the card says either how it turned out or which ingredient is still missing.

**Where the story lies:** the binder is filled from the loose pages by a tool, and the run refuses to start if the two differ, so "the same numbers" says the move lost nothing, not that the design is right.

## What runs, in order

1. **The database holds the files' inputs.** `tools/design_inputs.py differences`: every case line by line and every input file byte for byte, else the run is refused, naming each file that differs.
2. **The design loop** (`tools/pipeline.py`): sizing, the mode matrix, convergence, selection, faults, dispatch, the selected family's Monte Carlo and soft OILS, every engine run from the database (`TRINETRA_DESIGN`); each run's manifest names the database's fingerprint.
3. **The case's campaigns** (`tools/engine.py campaign`), the same way.
4. **Every number compared** with the one stored before the run (the selection and its budgets, every statistic of the Monte Carlo and the campaigns), and each one that differs named.
5. **Every row and closure** (`tools/evaluate.py`): a row's value is stated by the case (in SI), computed by its pseudocode in the interpreter from the rows its inputs name, or supplied as evidence by the selected design's Monte Carlo at the campaign's claimed probability; otherwise it says why not. Each KPI's verified closure compares its requirement with its evidence, its analysis closure with its analysis row, in the requirement's sense; otherwise it is blocked, naming the row with no value.
6. **The traceability** (`tools/trace.py`): every stated requirement and what checks it.

## What it shows

`results/END_TO_END.md`: per case, the steps, how many runs read from the database, how many numbers were compared and which differ (none, when the database is the files). `results/EVALUATION.md`: every closure's answer or what blocks it, and every row's value or why not. `results/TRACEABILITY.md`.

## Why so many rows have no value yet

A row has a value only when the case states it, its pseudocode can run, or a campaign supplies it. Most rows are still shells or relations without pseudocode (`docs/CARRY_OVER.md`: 80 rows without a relation or pseudocode, 166 internal rows whose kind is still to be set), and many inputs are supplied by a lab, a product or a supplier, not by the case. Each is listed with its reason; as authors write the pseudocode and leads release their groups (`docs/DELIVERY.md`), rows gain values and closures answer, and `tools/evaluate.py --check` refuses a closure that answered before and is blocked now.
