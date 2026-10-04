# Contributing to TRI-NETRA ADCS

**In one line:** the design is written by people in the apps (authors in the node app, leads in the group app), the software is written by the developer team in this repository, and the two meet only through sealed group releases; every change to the repository goes through a pull request that passes `python3 tools/check_all.py`.

## Say it simply

Two workshops share one building. In the drawing office, engineers draw and sign each part; the chief of each section stamps a numbered edition. In the machine shop, the developers build exactly what the stamped editions say, and test it against the drawings' own numbers. The machine shop never redraws a part; it sends it back to the drawing office.

**Where the story lies:** a machine shop can measure a part and adjust it. Here the developers may not adjust a number to make a test pass: an expected value never comes from the code under test.

## Who changes what

| You are | You change | Where | How it is checked |
|---|---|---|---|
| A node's author | the node's content: question, relation, value, test vectors, explanation, belief record | TRI-NETRA Node (`docs/NODE_APP.md`) | the live checks on Review; a second engineer signs it as checked |
| A stage owner | the signature on your stage | TRI-NETRA Group, Assemble | the signature covers the stage as it is |
| A group lead | the group's structure, its people and contracts; its releases | TRI-NETRA Group (`docs/GROUP_APP.md`) | every action's impact check; `tools/group.py check`; the seal |
| The developer team | the code, the tools, the data, the apps, the manuals | this repository, by pull request | `python3 tools/check_all.py` and CI |

A team member who has no node or group to write in asks for a change with a node form (`spec/manual/user/04_asking_for_changes.md`); the group app imports node forms into node files.

## In the repository

1. **Branch, change, check.** Work on a branch. `docs/CHANGING.md` says, for every kind of change, which file is the source, which command regenerates what follows, and which check proves it. Run `python3 tools/check_all.py` before you push; CI runs the same.
2. **Say before you run.** `python3 tools/trinetra.py explain <command>` (or `dry-run <command>`) says what any command does, what it reads and writes, what it checks, how to undo it and where its code is (`docs/COMMANDS.md`).
3. **Never edit a generated file.** Each says so in its header, and its `--check` finds a hand edit.
4. **The twin moves with the engine**, and the C flight software with the Rust, in the same change.
5. **Node content is never edited here.** It arrives only as a sealed release: `python3 tools/group.py verify DIR` checks every group's latest release, `merge DIR` takes them into `design.tndb`.
6. **Pull requests** follow `.github/pull_request_template.md`; one human review is needed on `main`.

## The rules nothing bends

1. **A node's content comes only from its group's sealed release.**
2. **An expected value never comes from the code under test.** A computing node without a test vector whose answer comes from outside the code is never sealed as confirmed.
3. **Every formula lives in one place** (the physics in `spec/physics/`, translated by `tools/pcode.py`; the flight algorithms in `fsw/pseudocode/`, held to C and Rust).
4. **A refusal is never a substitution.** A value that cannot be used is refused by name, never clamped, defaulted or guessed.
5. **What is not known is shown, not hidden.** A gap stays a visible gap, with its owner team, until a person fills it.
