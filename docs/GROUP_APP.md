# The group app: structure and releases

TRI-NETRA Group (`pages/group.html` in the kit; `design/js/group_app.js`) is the group lead's app.
It has two sides. The structure of a group (`docs/RELEASE_PLAN.md` P4): its nodes, what reads what,
its stages and who signs them, its people and who authors which node, its contracts with other
groups, and change requests both ways. And its releases (P6): every node's progress, the group
assembled and checked across its nodes, stages signed, releases sealed, compared and re-issued from,
node forms imported.

It works on the files of `docs/FILES_IN_THE_BROWSER.md`, with the same rules: Chrome or Edge, a
Drive folder, nothing sent anywhere, someone else's open file refused by name, saves read back and
checked.

## The design folder

```
structure/<group>.group.tndb    one per group: stages, people, nodes, the edges into its nodes, contracts, change requests
structure/actions/<id>.json     every action: what it changed and who did it
nodes/<id>.node.tndb            one per node; a node keeps its file when it changes group
releases/<group>-<v>.tnrel      one per sealed release of a group: frozen, never edited
```

`tools/seed_design.py` writes the first one from the spec (20 groups, 734 nodes).

## For a group lead

Open the design folder, then click your group. Its tabs:

| Tab | What you see and do |
|---|---|
| **Map** | The group's nodes by stage, the arrows between them, and, dashed, the nodes of other groups yours reads. Click a node to see who reads it (inside and outside the group) and to rename it, change its stage, split it, merge it into another, move it to another group, issue it to an author or archive it. **Add a node…** makes a new node file |
| **Nodes** | Every node with its stage, state and author |
| **Stages** | The group's stages and who owns (signs) each; add a stage, set an owner |
| **People** | Members and their roles (lead, stage owner, author); who authors which node, and when it was issued |
| **Contracts** | The nodes other groups read, and the contract published for each (output, unit, version, readers) |
| **Requests** | Change requests to your group (accept or decline) and from it (with their state) |
| **Progress** | Every node: its author, where its work stands (shell, draft, ready, checked), its signatures and whether they still stand, its problems, its evidence debt, what changed since the last release |
| **Assemble** | The checks across the nodes (an input from a node that is gone or archived, a quantity or unit that does not match, a contract that disagrees with its node, a missing arrow); each stage and its owner's signature; every node as it would be sealed, with why it is not confirmed. Click a node to see it as the main application will show it, and to comment on it |
| **Release** | The group's releases (version, when, who, how many confirmed, whether the fingerprints still hold); **Seal**, **Compare**, **Re-issue a node**, **Import node forms** |
| **History** | Every change to the group: who, when, what, how many files |

**Every change shows its impact first.** Each line of the impact check is one of three things:

- a **note**, such as "a new node file nodes/x.node.tndb";
- a **change**, such as "x and y in act lose z as an input";
- a **stop**, such as "kpi reads z: raise a change request to kpi". **Do it** appears only when nothing stops it.

**Another group's file changes only with its agreement.** Three actions touch another group: moving
a node into it, and archiving or merging a node it reads. Each needs a change request to that group,
accepted by its lead in their own file. The impact check offers to raise the request. When it is
accepted, the action goes through, and the request is marked done.

**All or nothing.** An action changes every file it touches, or none:

1. Every file is opened for editing; if someone else has one open, the action stops, naming them.
2. Every change is made and checked.
3. The whole set of new files is written to `structure/actions/<id>.json`, then each file is written.

If the computer stops part-way, the app shows "A structure action did not finish" next time.
**Finish it** completes it from that record. A file someone changed in the meantime is left as it is
and reported.

**Check this group / Check the whole design** holds the folder to its rules:

- every node is in exactly one group;
- its node file is there, and gives the same group, stage, label and state;
- every edge is kept by the group of the node that reads it;
- every edge comes from a node that exists and is not archived;
- every author, contract and stage owner belongs to the group.

## Releasing a group

**Confirmed or UNCONFIRMED.** A release takes every node of the group as it is. A node goes in
as *confirmed* only when:

- someone other than its author checked it (the node app's "Sign as checked"), and it has not
  changed since;
- the checks find nothing in it, and nothing across the nodes points at it;
- its stage, when the stage has an owner, is signed by that owner as it is now;
- **a computing node has a test vector whose answer comes from outside the code** (a book, an
  independent derivation, another tool, a physical bound). An answer the code made proves nothing.

Every other node goes in *UNCONFIRMED*, with its reasons written into the release.

**Who does what.**

1. Authors fill their nodes in the node app and mark them ready; a second engineer signs each as
   checked.
2. A stage owner signs the stage (Assemble → **Sign…**). The signature covers the stage's nodes as
   they are; a change to any of them takes it off.
3. The lead seals (Release → **Seal 1.0…**). Only the lead can, and only when the structure holds
   and every node file opens. The impact check says how many nodes go in confirmed, which do not,
   and what changed since the last release.

**What a seal does**, all or nothing:

- `releases/<group>-<version>.tnrel` is written and never changed again. It keeps every node as it
  was (its content, inputs, test vectors, pictures and signatures), the group's structure, the stage
  signatures and the lead's seal, with SHA-256 fingerprints over all of it.
- Every node file is stamped with the release ("act 1.0", confirmed or not) and **sealed**: the node
  app opens it read-only, saying so. The stamp is a status line, so no signature goes stale.

Versions go 1.0, 1.1, 1.2…. A group with nothing changed since its last release is not sealed again.

**Re-issue** opens a sealed node again for its author (Map → the node → **Re-issue…**, or Release →
**Re-issue a node…**). Choose "the file as it is", or a release: the node file then gets back what
that release sealed, signatures included. A node file that is missing or damaged is made again from
a release this way.

**Compare** shows two releases, or a release and the group now, node by node: which were added,
removed or changed, and which fields changed.

**Comments**: on a node (Map → **Comment…**, or from its view in Progress and Assemble). The comment is
written into the node file; its author sees it on the node's Home in the node app.

**Import node forms** takes today's node forms (`adcs-node-form/1` HTML files, from the node
library or filled by a team member) into the node files. Each form names its node; it must be one of
this group's and not sealed. The form's answers fill the node's fields (marked as the form's, with
its requester), its inputs when the node has none, its test vectors and attachments. A field the
author has typed is kept, and the impact check lists it. A form that names nobody as its requester
is refused.

## Help

**Help** in the header opens the guide for group leads and stage owners, the journey of a node to a
release, the glossary and every other guide, in place and offline. Each screen has a tour, shown once
to someone opening the app for the first time and again from Help. A node's view (Progress, Assemble)
prints alone with **Print**.

## For a developer

| Piece | Where |
|---|---|
| The structure actions, the impact check, the rules, the action records | `design/js/structure.js` (`Workspace`, `impact`, `integrity`, `loadIndex`) |
| A save in two halves (prepare, commit), for actions across files | `FileSession.prepare` / `commit` in `design/js/tnfile.js` |
| The page | `design/js/group_app.js`, `design/pages/group.template.html`; the tabs, map and impact list are components of `design/js/tn_ui.js` |
| The same rules from Python | `tools/group.py check DIR` (written separately, so neither checker is the only judge of the other) |
| The group assembled, the checks across nodes, seal, re-issue, stage signatures, comments, node forms, compare | `design/js/release.js` (`assemble`, `crossChecks`, `verdict`, `Releases`, `compare`, `parseForm`); its actions go through the same `Workspace` (`EXTRA` in `structure.js`) |
| Releases checked from Python | `tools/release.py check PATH` (fingerprints, the confirmed rule, the seal), `tools/release.py list DIR` |

**How it is proven** (`check_all` runs these as `structure`, `release` and `offline-pages`):

- `tests/test_structure.py` runs `tests/js/structure.test.mjs` on the whole seeded design, under Node on a
  folder on disk. It checks that:
  - all 20 groups open;
  - `act` (the largest, 138 nodes) is restructured with every action, and so is `catalogue` (the
    smallest, 8);
  - a node `ctl` did not accept cannot be moved into it, and once accepted it moves;
  - a node another group reads is not archived until that group accepts;
  - a node file someone else has open stops an action with nothing changed;
  - an action cut short after its first file is finished from its record;
  - afterwards every file passes `tools/tndb.py check` and `tools/group.py check` finds nothing.

  Both checkers are also given the same deliberately broken folders, and both must find each break.
- `tests/browser/group.test.mjs` does the same through the page in Chromium: the forms, the impact
  dialog, accepting requests in the other group's Requests tab, the map at phone width. The folder
  the page leaves behind is then checked by `tools/group.py`.
- `tests/test_release.py` runs `tests/js/release.test.mjs` on the whole seeded design under Node:
  - all 20 groups are assembled and seal 1.0; then each re-issues a node, changes it and seals 1.1,
    and every release file checks;
  - only the lead seals, and only a stage's owner signs it;
  - a declared node written, marked ready and checked by someone else is confirmed only once its
    stage is signed;
  - a computing node is not confirmed without a test vector from outside the code;
  - a node is re-issued, changed and checked again, and act seals 1.1; comparing 1.0 with 1.1 names
    that node and the fields that changed;
  - a damaged node file stops the seal and is made again from a release;
  - a node form is imported, with the author's own field kept.

  Then `tools/release.py` checks every release it wrote, and is given releases broken on purpose (a
  node changed, a node dropped, a computing node confirmed without an outside answer, a node checked
  by its own author): it must find each.
- `tests/browser/release.test.mjs` does it through the page: progress and assemble for act, the
  stage signed by its owner, the seal refused to someone else and done by the lead, a node re-issued,
  commented on and renamed, 1.1 sealed and compared, a node form imported into env, catalogue
  sealed; in the node app a sealed node opens read-only. The folder it leaves passes
  `tools/tndb.py`, `tools/group.py` and `tools/release.py`.
