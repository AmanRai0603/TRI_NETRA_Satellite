# Checks and CI
<!-- kind: reference; depth: expert -->

**In one line:** every check this package runs, and the CI jobs in the repository that guard the operating model, the ledger and the explanation standard.

## In this package

| Command | Proves |
|---|---|
| `python3 tools/validate_plan.py` | the package is internally consistent: the tree, the seed content, units and physics, the case format and cases, the ledger supplier, the catalogue, classes, products, algorithms, scenarios, campaigns, designs. Must print `0 finding(s)`. |
| `python3 tools/validate_plan.py --selftest` | each deliberate breakage, applied to a copy, is caught |
| `python3 tools/build_tree.py --check` | the tree, the case registry and the case template are current |
| `python3 tools/intake.py selftest` | each deliberate mistake, at least one per error rule including D01–D06 and X01–X02, is refused by its code; the clean requests pass (a change, a change whose broken belief moves two risks, a new node, a confirmation, feedback, every seed form); every seed sheet verifies; a changed unit, a changed expected value, a changed belief and a move filed against the wrong risk are caught |
| `python3 tools/derisk.py check` | the risk register and the belief records keep the ledger's rules (L01–L09) |
| `python3 tools/derisk.py selftest` | each deliberate mistake in the ledger is refused by its rule; `record` writes a tested move and a new risk cleanly and refuses an untested belief that lowers a level; the rollup gives the 18 risk rows; the narrative is deterministic |
| `python3 tools/derisk.py table --check` | SPEC.md §21's table is the register (in the repository, `--file docs/RISKS.md`) |
| `python3 tools/explain_check.py` | every rendered template example and every manual page carries the marks of `adcs-explain/1` |
| `python3 tools/explain_check.py --selftest` | each mark removed from a page is caught by its rule |
| `python3 tools/explain_kit.py --check` | every template carries the current explanation kit |
| `python3 tools/check_case.py plan/cases/*.csv` | the reference cases pass the format, and the report says what each can run |
| `python3 tools/form_browser_check.py` | in headless Chromium: the node form (with its belief record and depth switch), the case editor, the library and the results work end to end, at desktop and phone width |
| `python3 tools/manual_pages.py --check` | the generated manual parts (case keys, intake codes, ledger codes, twin rules) are current |
| `python3 tools/make_examples.py` | writes every example from the current templates (run before the checks above) |
| `python3 tools/twin_check.py` | the twin map is whole: every SILS element has a platform side and a MATLAB side, or says why it has one (TW01–TW04) |
| `python3 tools/twin_check.py --selftest` | each break in the map, a checkout missing a twin, and a one-sided change are caught (TW01–TW06) |
| `python3 tools/pack_matlab.py --out <dir> [--phase Pn]` | the MATLAB zip builds with the twin map and `TWIN.md`; twice gives identical bytes; with `--phase`, refuses when a twin that phase builds is missing |
| `bash tools/assemble_spec.sh --check` | SPEC.md is its sections joined |

## In the repository

SPEC.md §18 lists every CI job. The ones that guard the operating model:

| Job | Guards |
|---|---|
| `intake` | the checker's selftest; on an `intake/*` branch, check and verify of the request the branch names |
| `no-writes` | no face, route, command or view can change a sheet, a layer, the catalogue, a scenario or a campaign |
| `derisk` | `derisk check`; a pull request touching the engine, the solver, results, the web views, forms, algorithms or scenarios carries a belief record or `derisk:none` with a reason |
| `explain` | `explain check` over every template's rendered examples and every manual page; `explain_kit --check` |
| `twin` | the twin map is whole; every element the phase reached builds exists on both sides; a pull request changes both sides of an element or carries `twin:none` with a reason |
| `matlab-pack` | from P1, the MATLAB SILS zip on every push, whole for the phase reached, identical twice |
| `forms` | the browser check, and two exports of one node compared byte for byte |
| `cases` | one reader for the case format, and every malformed file refused |
| `results` | a result is the same file whichever engine wrote it, and opening or importing one runs nothing |
| `manual` | the manuals say what the software does |
