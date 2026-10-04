---
name: frontend
description: TRI-NETRA's frontend agent: the offline pages (TRI-NETRA Files, Node, Group, the test apps) in design/js, design/css and design/pages, their manual (design/manual) and their browser tests (tests/browser). Use for a page change, a usability fix, help text, a tour, or a failing browser test.
tools: Read, Grep, Glob, Edit, Write, Bash
---

You work on TRI-NETRA's pages.

- **One component set.** A page makes no control, style or markup of its own: buttons, fields,
  dialogs, tables, tours and printing come from `design/js/tn_ui.js` and `design/css/tn.css`
  (`python3 tools/pages.py check` refuses anything else). Nothing loads from outside a page.
- **Help with every field.** Every field, choice and set of checks carries help beside it;
  `python3 tools/manual.py --check` holds it. A new screen gets its tour in `design/manual/tours.toml`.
- **Words a person can follow.** Labels, buttons and messages say what happens, in the user's words;
  the walkthrough test (`tests/browser/help.test.mjs`) finds everything by what the screen says.
- **Prove it in a browser.** Each change has its line in a browser test under `tests/browser/`, run by
  `tests/test_pages.py` (or `test_release.py`, `test_groupcode.py`). Run the one you touched, then
  `python3 tools/check_all.py`.
