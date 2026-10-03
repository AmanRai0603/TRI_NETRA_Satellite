# The node app

TRI-NETRA Node (`pages/node.html` in the kit, `Apps/TRI-NETRA Node.html` in the Drive pack;
`design/js/node_app.js`) is the node author's app (`docs/RELEASE_PLAN.md` P5). An author fills a
node step by step, sees what is still missing as they type, previews the node as a reader will see
it, and marks it ready. Someone else then signs it as checked.

It works on the files of `docs/FILES_IN_THE_BROWSER.md`, with the same rules: Chrome or Edge, a
Drive folder, nothing sent anywhere, someone else's open file refused by name, every change one
undoable step, saves read back and checked.

## For a node author

Open the design folder (or a single node file). **Issued to you** lists the nodes your groups'
leads have issued to you; **Find a node** searches every node by id, label, group or author. Open one. Its tabs:

| Tab | What you see and do |
|---|---|
| **Home** | Where the node sits (group, stage, what it reads, who reads it), how far it is filled, its evidence debt, the lead's comments. **Start from the spec** fills the fields from what the spec already says about this row. **Acknowledge** appears when a contract on what the node reads has changed |
| **The steps** | One tab per step, each field with its question, why it is asked and an example (below) |
| **Review** | Every problem the checks find, each with what to do. **Mark ready** stays off while one is left. **Sign as checked** is for someone else: it refuses the node's author. **Ask for a contract change** raises a change request to the group that owns the contract |
| **Preview** | The node as its reader will see it: the answer first, then where it sits, the explanation, the relation, the inputs, the value or requirement or evidence, the test vectors with their results, the pictures, the sources, the code, the belief record, the versions |

**The kind decides the steps.** The tree fixes some kinds; the spec's catalogue gives the rest:

| Kind | What it is | Steps |
|---|---|---|
| declared | a value someone states, with its source | identity, explanation, theory, inputs and output, value, results, evidence, pictures, belief |
| computed | a relation from other nodes | identity, explanation, theory, inputs and output, pseudocode, results, evidence, pictures, code, belief |
| KPI | what the mission requires, and which way it binds | identity, explanation, inputs and output, requirement, evidence, pictures, belief |
| evidence | what a test shows: a metric on the verification rungs | identity, explanation, inputs and output, evidence, results, pictures, belief |
| closure, interface, target row | fixed by the tree | identity, explanation, pictures, feedback |

A row the spec has not named yet has no kind: its author chooses declared or computed first.

**Helpers.**

- *Equation helper*: a palette of symbols and the node's inputs; it inserts at the cursor and shows
  how the equation reads.
- *Pseudocode*: checked as you type (units, inputs, the output), in the language of
  `docs/PSEUDOCODE_V2.md`. **Try it** runs it on every test vector and marks each one passed or failed.
- *Test vectors*: one per line, `name | a=1, b=2 | expected | tolerance | provenance | source | where`,
  in SI units; the source is one of the spec's sources.
- *Results*: paste straight from Excel or Sheets (tab-separated, the first row the headings).
- *Pictures*: PNG, JPEG, GIF or WebP. One over 500 KB goes through the picture wizard, which
  makes it smaller (JPEG, scaled down) and shows the result before it is kept. Pictures are shown
  as images only; nothing in them runs.

**The live checks** are the spec's intake rules (`spec/tools/intake.py`) for one node, run in the
page: units against the quantity, the explanation standard's marks (the one line, the wrong idea and why,
where an analogy breaks), sources named, the pseudocode, the test vectors, the evidence metric and rung,
the belief record. Each problem names its rule (for example `X01`) and the field. Warnings do not
stop **Mark ready**.

**Evidence debt** is what the node claims without evidence yet: shown on Home and in the preview.

**Ready and signed.** Marking ready signs the node's content with a fingerprint (SHA-256 of its
content, inputs, test vectors and pictures). A check signature carries the same fingerprint. Any
later edit makes both stale, and the app says so: the node has to be marked ready and checked again.
The app never changes what the group decides (the node's group, stage, label, author); the only
thing it writes there is the acknowledged contract version.

## For a developer

| Piece | Where |
|---|---|
| The kinds, steps and fields; reading a node file; the checks; try it; the fingerprint and signatures | `design/js/node_model.js` |
| The catalogue it checks against (units, quantities, physics, sources, metrics, rungs, every row of the tree) | `design/js/node_catalog.js`, written by `tools/node_catalog.py` from the spec (`--check` in `check_all`) |
| The preview | `design/js/node_view.js` |
| The page | `design/js/node_app.js`, `design/pages/node.template.html`; fields, choices and the equation palette are components of `design/js/tn_ui.js` |
| The pseudocode checker and interpreter | `design/js/pcode.js` (P2) |

**How it is proven** (`check_all` runs both as part of `offline-pages`):

- `tests/js/node_model.test.mjs`: each check, rule by rule, on a node that breaks it and one that
  keeps it; try it; pasted results; the fingerprint going stale.
- `tests/browser/node.test.mjs` in Chromium on the whole seeded design: every kind filled and
  previewed through the page — declared (`gm_0`: the spec's values taken, a contract acknowledged,
  the lead's comment shown), computed (`m2_4`: equation helper, pseudocode checked and tried on a
  sourced test vector, marked ready, refused to its author, signed by someone else, made stale by an
  edit), KPI (`p1k_0`), evidence (`p1a_0`: results pasted), closure, interface (a picture too big,
  made smaller by the wizard), and an unnamed row whose author chooses its kind. Typing is not lost
  while the page re-renders. The folder it leaves passes `tools/tndb.py check` and
  `tools/group.py check`.
