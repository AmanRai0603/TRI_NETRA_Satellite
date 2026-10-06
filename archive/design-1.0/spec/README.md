# ADCS platform — build package

Everything needed to build the ADCS platform repository with Claude Code. It goes into the new repository at `_package/` (see `CLAUDE_CODE_PROMPT.md`).

The platform is a sheet-driven engine (SPEC.md §3) with an ADCS tree in it, and:

- case import from a fixed CSV;
- a solver and designer over a catalogue of products;
- a time-stepping loop engine and a real-time rig, whose OILS and HILS needs the tree derives from each case and compares with the lab's measured capabilities (SPEC.md §12.10);
- a client portal;
- a plain-MATLAB twin of the SILS engine, built in lockstep with the platform: every SILS element is written in both engines in the same change, so the MATLAB SILS zip is whole at every push (SPEC.md §10.8.7);
- an intake that turns a team member's node form into checked, implemented and released software;
- one explanation standard, `adcs-explain/1`, that every page a person reads follows (SPEC.md §5.12);
- a de-risking ledger: every decision rests on a recorded belief, every node version says why it exists, and a Risk management branch concludes at every release (SPEC.md §5.13).

A customer's case arrives as one CSV. The solver tunes each catalogue product in SILS against it and selects the ones that meet every requirement. When none does, the design team's runs of the same engine find candidates. The platform tests that ADCS in SILS, PIL, OILS and HILS on one scenario template, and certifies it. Clients get SILS with visualisation before the order, and OILS and HILS with visualisation and certification after it.

## How it is run and changed

- **The developer team** builds, maintains and upgrades the software. Only they change it. The released software never changes itself: it has no edit mode and no write route.
- **Everyone else uses it** and gets two things:
  - **The released software.** Their only input is a case CSV, written in the case editor. Every finished run is saved as a result document beside its case, and reopens without re-running.
  - **The node form.** To ask for anything: a changed node, a new node, a confirmation, feedback on a release, or something else. The developer team checks the form, has the implementation agent implement it, verifies the result against the form, reviews and releases it, and answers in the same file.

Two threads run through all of it:

- **Every page explains itself.** The node form, the case editor, the node library, every result and the quarterly narrative are written to `adcs-explain/1`, taken from *Eight Loops, One Beam*: the answer first, an overview before detail, each section's kind shown, a Learn · Read · Expert switch, and each concept told as say it simply → the real thing → where it breaks → try it. The spin-down node (`gf_7`) is the worked example.
- **Every change says why.** A change or a new node carries its belief record: what was believed, what tested it, what we now know, what was wrong with the version it replaces and what this one gives, and the risks it opens or closes. The ledger is `derisk/`; the Risk management branch counts it; the quarterly narrative explains it in the company template's columns.

`manual/user/` is what a team member reads. `manual/developer/` is what the developer team reads. Both ship with the software, and both follow the standard.

## Check the package

```
python3 tools/validate_plan.py --selftest                        # 28 deliberate breakages, each caught
python3 tools/validate_plan.py                                   # must print 0 finding(s)
python3 tools/build_tree.py --check                              # the tree, case registry and case template are current
python3 tools/intake.py selftest                                 # 66 refusals by code; 88 clean requests; 82 seed sheets verify
python3 tools/derisk.py check                                    # the risk register and belief records keep their rules
python3 tools/derisk.py selftest                                 # each ledger mistake refused; rollup; narrative deterministic
python3 tools/derisk.py table --check                            # SPEC.md §21 is the register
python3 tools/derisk.py narrative --quarter Q3-26 --out /tmp/nar # the quarterly narrative: .html, .xlsx, .csv
python3 tools/check_case.py plan/cases/*.csv                     # the reference cases, and what each can run
python3 tools/manual_pages.py --check                            # the manuals' generated parts are current
python3 tools/make_examples.py                                   # every example, from the current templates
python3 tools/explain_kit.py --check                             # every template carries the current explanation kit
python3 tools/explain_check.py                                   # every page carries the explanation standard's marks
python3 tools/explain_check.py --selftest                        # each removed mark is caught
python3 tools/form_browser_check.py                              # node form, case editor, library, results, in Chromium
bash tools/assemble_spec.sh --check                              # SPEC.md is the spec/ sections joined
python3 tools/twin_check.py                                      # the twin map is whole: every SILS element, both sides
python3 tools/twin_check.py --selftest                           # each break in the lockstep caught (TW01-TW06)
python3 tools/pack_matlab.py --out /tmp/twin                     # the MATLAB SILS zip, deterministic, with TWIN.md
```

These need Python 3.11 or newer (`tomllib`); the browser and explanation checks need Playwright with Chromium; the narrative's `.xlsx` needs `openpyxl`. The flight software headers compile clean as C99 and C++17:

```
printf '#include "adcs_fsw.h"\nint main(void){return 0;}\n' > /tmp/h.c
gcc -std=c99 -Wall -Wextra -Wpedantic -Werror -Ifsw/include -fsyntax-only /tmp/h.c
```

## What is where

| Path | What |
|---|---|
| `RELEASE.md` | this release: what it adds, what the package holds, the checks run and their results, what it does not do |
| `SPEC.md` | the build specification, §0 to §22; §1.6 is the operating model |
| `CLAUDE_CODE_PROMPT.md` | the per-phase prompt, and notes for the person driving it |
| `manual/user/`, `manual/developer/` | the two manuals, complete; shipped with every release |
| `plan/tree.json` | the ADCS tree (418 rows, 307 edges, one door), built by `tools/build_tree.py` |
| `plan/case_template.csv`, `plan/case_inputs.toml` | the fixed case format `adcs-case/1`, every key explained, and its registry (generated) |
| `plan/cases/*.csv` | four reference cases: the default 3U AIS (10°) and 3U AIS + imaging (0.01°) cases, a 150 kg bus, an unstated 12U |
| `plan/seed_content.toml` | the content of the 82 seed forms: the pilot thread and the Risk management branch, from 26 cited sources, with 7 transcribed test vectors and the worked example's explanation |
| `derisk/` | the risk register (18 risks, levels proposed until D26), 16 belief records for this package's own decisions, the narrative template, and where a quarter's prose goes |
| `plan/units.toml`, `plan/physics.toml` | the units and quantities a node may declare; the physics functions a step may call |
| `plan/kpis.toml` | the 22 KPIs: each requirement row, its evidence row, its analysis row and metric kind |
| `plan/expected_node_ids.json` | the node id the seeder gives each row |
| `catalogue/` | the module descriptor standard, 4 families with their slots and algorithms, 13 parts (6 IDMAS, 7 synthetic), 5 seeded products, 4 algorithms with tuning bounds, 7 satellite classes |
| `scenarios/`, `campaigns/`, `designs/` | 5 scenarios, 7 campaigns (every campaign type), two design jobs |
| `devices/`, `rig/`, `fsw/include/` | an example device protocol; the OILS/HILS device map and two labs, as plants and as facilities (rig host, IEU, field cage, air bearing, stimulators, test stands, safety); `adcs_hal.h` and `adcs_fsw.h` |
| `forms/` | `node_form.html` (the node's document and the request form), `case_editor.html` (writes the case CSV), `library.html` (the node library's index), and examples written by `tools/make_examples.py`: node forms (`gf_7`, the standard's worked example; `p1k_0`; `rk4_0`), a seed form, a new-node request, a returned request with its belief record, the case editor |
| `results/` | `template.html`, the result document every finished run is saved as, and two demonstration results from a toy model (never evidence) |
| `matlab_sils/`, `plan/twin_map.toml` | the MATLAB SILS twin's README and package contents, and the twin map: every SILS element's platform side and MATLAB side, the phase that writes both, the test both pass |
| `tools/` | `plan_model.py` (the plan, read-only), `build_tree.py`, `validate_plan.py`, `forms.py` (forms, seed forms, library, case editor), `intake.py` (checker, sheet writer, verifier, replies), `derisk.py` (the ledger: check, rollup, narrative), `explain_kit.py` and `explain_check.py` (the explanation standard), `make_examples.py`, `check_case.py`, `results.py` (result documents and the local store), `manual_pages.py`, `form_browser_check.py`, `pack_matlab.py`, `twin_check.py` (the lockstep), `assemble_spec.sh` |
| `spec/` | the sections `SPEC.md` is assembled from; edit these, then run `tools/assemble_spec.sh` |
