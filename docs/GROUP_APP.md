# The group app: structure

TRI-NETRA Group (`pages/group.html` in the kit; `design/js/group_app.js`) is the group lead's app.
This part of it (`docs/RELEASE_PLAN.md` P4) is the structure of a group: its nodes, what reads what,
its stages and who signs them, its people and who authors which node, its contracts with other
groups, and change requests both ways. Assembling and releasing (P6) come next.

It works on the files of `docs/FILES_IN_THE_BROWSER.md`, with the same rules: Chrome or Edge, a
Drive folder, nothing sent anywhere, someone else's open file refused by name, saves read back and
checked.

## The design folder

```
structure/<group>.group.tndb    one per group: stages, people, nodes, the edges into its nodes, contracts, change requests
structure/actions/<id>.json     every structure action: what it changed and who did it
nodes/<id>.node.tndb            one per node; a node keeps its file when it changes group
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
| **History** | Every structure change to the group: who, when, what, how many files |

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

## For a developer

| Piece | Where |
|---|---|
| The structure actions, the impact check, the rules, the action records | `design/js/structure.js` (`Workspace`, `impact`, `integrity`, `loadIndex`) |
| A save in two halves (prepare, commit), for actions across files | `FileSession.prepare` / `commit` in `design/js/tnfile.js` |
| The page | `design/js/group_app.js`, `design/pages/group.template.html`; the tabs, map and impact list are components of `design/js/tn_ui.js` |
| The same rules from Python | `tools/group.py check DIR` (written separately, so neither checker is the only judge of the other) |

**How it is proven** (`check_all` runs both, as `structure` and `offline-pages`):

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
