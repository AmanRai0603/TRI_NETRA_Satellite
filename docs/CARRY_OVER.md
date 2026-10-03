# What the design starts with

**In one line:** every node file starts with what the repository already says about it (`tools/carry_over.py`, `docs/RELEASE_PLAN.md` P8), each item marked with where it came from, and with a list of what is still missing and which team owns it; nothing is invented, so most of what is missing is the explanation and the belief record, which only people can write.

## Say it simply

Before the authors start, someone went through the filing cabinets and put into each folder every page that already belonged to it, with a sticker saying which drawer it came from, and a note on the front listing what the folder still needs.

**Where the story lies:** a filing clerk might guess. The carry-over does not: a field the repository does not state stays empty and is on the list.

## Now the real thing

**Where each item comes from** (the origin of a carried field reads `carried:<file>`):

| Source | What it gives | Nodes |
|---|---|---|
| `spec/plan/seed_content.toml` | question, answer (symbol, quantity, unit, bounds and their reasons), relation, source, derivation, value, requirement sense, assumptions, explanation, inputs, test vectors | 82 |
| `spec/plan/physics.toml`, `spec/physics/*.pc` | the relation as pseudocode with the constants and functions it calls, the node's answer named as the node names it; where it runs in Rust (`adcs_physics`) and in the twin | 31 |
| `spec/plan/case_inputs.toml` | which case key, or which supplier (lab, product, de-risking, evidence, tuning), gives a declared value | 152 |
| `spec/plan/kpis.toml` | a requirement's sense, an evidence row's metric | 60 |
| `catalogue/algorithms/*.toml` | a tunable parameter's meaning, range and trade, the algorithm's source | 8 |
| `spec/plan/tree.json` | the tree's own note on a row | 321 |
| `design/carry.toml` | names for the 166 internal rows "to be named", from the code that computes them; 31 rows added (dynamics, onboard navigation, guidance and mode management, FDIR, CMG and VSCMG, each subsystem's sizing), with their flight pseudocode where there is one | 197 |
| `design/groups.toml` | where a computing row's group runs (C, Rust, twin), when nothing more precise says it | — |

**The design after the carry-over:** 765 nodes in 20 groups (734 from the spec, 31 added), about 2,400 fields carried (`--report` counts them per group). Every carried pseudocode (48) compiles in the node app's checker, and every node with pseudocode and test vectors reproduces them in its interpreter.

**What is still missing, across the design:**

| Missing | Nodes | Who writes it |
|---|---:|---|
| an explanation (the one line, said simply) | 764 | the node's author |
| a belief record | 357 | the node's author |
| a question | 244 | the node's author |
| the kind of an internal row (declared or computed) | 166 | the node's author |
| a test vector with an answer from outside the code | 124 | the node's author, from a book, a derivation, another tool or a physical bound |
| a relation or pseudocode | 80 | the node's author |
| a value and its source | 55 | the node's author, from a datasheet, a measurement or a decision |

Each node lists its own gaps on Home in the node app; the group app's **Progress** counts them per node. The owner team is the group's lead team.

**Running it.** The Drive pack's `Design` folder is seeded and carried. To do it yourself: `python3 tools/seed_design.py --out DIR` then `python3 tools/carry_over.py DIR`; `python3 tools/carry_over.py --report DIR` prints the table per group. Carrying again changes nothing: a field already in a node file is never replaced, and an author's own words always stay.

**How it is proven** (`check_all` runs `carry`): `tests/test_carry.py` seeds and carries the whole design, checks every file and every structure rule, checks that items carry their origin, that internal rows are named in both the group file and the node file, that every node lists its gaps and owner team, that carrying twice changes nothing and keeps an author's own field, and runs `tests/js/carry.test.mjs`, which reads every node through the node app's own code, compiles every carried pseudocode and runs every test vector.
