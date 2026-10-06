
### 5.10 The node form: how anyone asks for anything

#### 5.10.1 The rule

Nobody outside the developer team changes the software, and the software never changes itself (§1.6). A team member who wants something different asks for it with a **node form**: one self-contained HTML file, `adcs-node-form/1`. The file is three things at once:

- **the document**: the node as the released software has it, read-only, written to the explanation standard (§5.12): the answer first, where the node sits, then its own station (say it simply, the relation with its why-chain, where it breaks, and a test vector the reader can rebuild), its versions and the beliefs and risks behind them;
- **the request form**: every field a team member may propose, each with the question it asks, why the answer matters and an example;
- **the request itself**, once saved, which the team member sends to the developer team. They answer in the same file.

It opens straight from disk in any browser. It needs no server, no network and no account, so it can be mailed, attached to an issue, dropped in a shared folder or uploaded in the portal. It carries no code, and none is expected from the person filling it. It carries what the code is written from.

`forms/node_form.html` is the template. `tools/forms.py` in this package fills it from the plan; in the built repository, `cargo xtask form export` fills it from `node.toml`. The examples in `forms/examples/`, written by `tools/make_examples.py`, are the package's output:

- `gf_7` (ring spin-down time), the explanation standard's worked example, `p1k_0` (the APE requirement) and `rk4_0` (the Risk management conclusion) as a team member receives them;
- `gf_7`'s seed form (§5.8);
- a blank new-node request;
- a returned request with its belief record, and the developer team's replies in its history.

#### 5.10.2 What a team member can ask for

The form first asks what the person wants. Each choice lights up only the sections it needs.

| Request | For | What it carries |
|---|---|---|
| **Change this node** | correcting or completing what a node says | the whole node as the person wants it; the checker computes what changed |
| **Ask for a new node** | a question the tree does not answer | where it goes (group, a sibling it is shaped like, label, id, kind) and the whole node |
| **Confirm this node** | an engineer has checked the relation, its source and its values, and stands behind them | a name under "Checked by" and a reason; nothing else changes (§5.8) |
| **Feedback on a release** | the software behaved unexpectedly | what was run (release, case CSV, result file), what happened, what was expected. Feedback may be sent in an older copy of the form, such as the one that came back "released": the checker does not hold it to the node's current base. |
| **Something else** | a part, an algorithm, a scenario, a campaign, the case format, a new group in the tree | a subject and a description, with attachments; the developer team turns it into the right change |

A door, an interface row and a closure are fixed by the tree's shape. Their forms offer only feedback and "something else". Any form may carry feedback entries as well as its request.

#### 5.10.3 The sections, and what each field asks

Every field shows three things: the question it asks, why the answer matters, and an example. The asks and whys of the sheet's fields come from `form.rs` `FIELDS` and `ARRAYS`, so the form and the sheet say the same thing in the same words.

| Section | Fields | Shown for |
|---|---|---|
| Where the new node goes | group, sibling it is shaped like, label, proposed id (filled from group and label), kind | new |
| The question it answers | question; note (what it is NOT for); label; hardware tags | change, new |
| Its answer | symbol; quantity; unit (it must state the quantity); lowest and highest value, each with its reason | change, new |
| What it reads | inputs: binding, the node it reads, the quantity it must be; the producer's label and what it publishes are shown beside each | computed nodes |
| The relation | expression; source; why it is this relation; how to read the answer; the derivation, step by step | computed nodes |
| How it is computed | steps: what each does, what it binds, its quantity, and the physics function it calls, or a description of the new one it needs; the hardware counts that make it zero when absent | computed nodes |
| What has to be true | assumptions: what is assumed and when that stops being true | computed and declared nodes |
| Its value / the requirement / the evidence | a declared number and its source; a requirement's sense; an evidence node's metric and rungs. A value a case supplies is explained and not offered. | declared, required, achieved |
| Test vectors | one row per vector: an input per binding (SI), the expected answer (SI), tolerance, provenance, source and page | computed nodes |
| Sources | a new source's id, author and title, and exactly where | change, new |
| Attachments | any files that help: a page of the source, a plot, a MATLAB prototype, a spreadsheet; 5 MB in all, carried inside the file | all but confirm |
| Your request | subject and description | something else |
| Feedback on a release | release, case CSV, result file, severity, what happened, what was expected | all |
| Explain it | the node in plain words; in one line; a common wrong idea and why it is wrong; two cases, one difference; an analogy and where it breaks; a why-chain down to a law, a standard or a decision (§5.12.3) | change, new |
| De-risking: the belief behind this request | the area (one of seven); what was believed; whether a test broke it, held it, or nobody has tested it; what was tested, or the test that would settle it; what we now know; what was wrong with the version it replaces and what this version gives; what changed in the plan; what the test cost ($k); every risk it opens, raises, lowers or closes (§5.13) | change, new; a confirmation may carry a held belief |
| Explain it back | the request in one or two lines, as the requester would tell a colleague; it travels as `request.summary` | Learn depth |
| Who is asking, and why | name (a person, never a tool), team, contact, needed by, priority, the reason, and "Checked by" | all |
| Review and send | how much of the request is filled; every problem, marked ! (must fix) or i (recorded); every change, now against proposed; Save filled copy; Download answers (JSON) | all |

**The page checks what it can see, with the checker's rules.** A number parses. A unit states its quantity. An input names a node the software has, publishes the quantity declared, and sits in the same layer. The last step binds the answer. A test vector gives one number per binding and cites a page. A requester is not an assistant. Each problem links to its section, and the side navigation counts them per section. The checker (§5.11) is the authority: it applies every rule again, and more, on the developer's side.

**Status.** When the file comes back from the developer team, a "Status of this request" card shows their replies, newest last: received, check failed (with each finding), check passed, in progress, in review, released in a version, needs information, feedback noted, or rejected with the reason. The header shows the latest.

**Before the fields, the page explains the node** (§5.12): a breadcrumb (platform › layer › group › node); one answer-first sentence (what the node is, its kind, version and state, and the highest open risk it carries); "Where this node sits" (what it reads → the node → where its answer goes); the node's document as a station; and "Versions, and what they rest on" (every version with its request, belief, previous issue and benefit, and every belief about the node). Every section shows its kind (tutorial, how-to, reference, explanation), and the depth switch (Learn · Read · Expert) shows the exercises, the explanation, or the relation, limits and sources.

**What else the page does for the person filling it:**

- a "Start here" card that says in six steps what to do and what happens after sending;
- "What happens to this file after you send it", the intake of §5.11 in plain words;
- "Copy an assistant prompt": the exact instruction for editing the file with Claude or another assistant, so that what comes back is still a valid request. It never lets the assistant supply a name or a test-vector number;
- draft recovery: answers kept in the browser as the person types, and offered back if the page is reopened before saving. It is a convenience only; the saved file is the record, and the page works when the browser keeps nothing;
- print, light and dark themes, a layout that works on a phone, and keyboard access throughout.

#### 5.10.4 The file

Everything that matters is the JSON block `<script type="application/json" id="adcs-node-form">`, and the page says so.

| Member | Holds | Who writes it |
|---|---|---|
| `schema`, `title`, `exported`, `mode` | `adcs-node-form/1`; the release it was exported from | the exporter |
| `node` | the node as the release had it: identity, placement, kind, owner, state, and every field of the sheet, including its `explain` table; its `versions`; and, from the ledger, the `beliefs` about it and the `risks` they carry | the exporter |
| `base` | a hash of the node's identity and of every field a form may propose; not its versions, beliefs, risks or state, so a risk another request moves does not make this form stale | the exporter |
| `consumers`, `kpis` | where the node's answer goes | the exporter |
| `catalog` | the pick-lists: every node with the quantity and unit it publishes, groups, units with the quantities each states, quantities, sources with what each is used for, physics functions with their arguments, owners, tags, metrics, rungs, provenances, tiers, the risk register's risks with their levels, the seven areas and the belief statuses | the exporter |
| `request` | `type`; `proposed`, the whole node as wanted (including `explain`); `placement` for a new node; `new_sources`; `reason`, `summary`, `priority`, `needed_by`; `requested_by`, `team`, `contact`; `attested_by`; `derisk`, the belief record (area, believed, status, tested, would_test, now_know, previous_issue, benefit, plan_change, cost_k, moves: risk, from, to, and for a new risk its title, area and closing test); `other`; `attachments` (name, type, size, data URL, note) | the team member |
| `feedback` | entries: release, case, result, severity, observed, expected | the team member |
| `history` | replies: date, who, status, note, release, findings | the developer team, through `cargo xtask intake reply` |

**Save filled copy** writes a copy of the file with the JSON block replaced, named `<node>.request.<date>.html`. The rest is the same template, so a saved copy reopens with every answer in place and can be edited again. Every `<` inside the block is written as `\u003c`, so no value in it can end or open a script.

**Filling it by hand or with an assistant.** A person may type into the page, edit the JSON block in a text editor, or give the file to an assistant with what they want. Only `request` and `feedback` are meant to change. The checker reads those, `base`, and `node.id`, which names the node; it compares the rest of `node` with the repository and reports any difference (I01), and decides everything else from the node as the software has it (§5.11.2). A form whose other members were altered therefore changes nothing it should not.

#### 5.10.5 Where forms come from

- **The node library.** `cargo xtask form export --library <dir>` (`tools/forms.py library` in the package) writes every node's form, the new-node request, the case editor (§8.3.3) and an `index.html`. The index lists every node by layer and group, marks which are written and which are unconfirmed, and has search. Every release carries the library: in the MATLAB zip (`forms/`), and on the workbench's and the portal's internal downloads. It is how a team member reads the tree's documents offline.
- **Single nodes.** `cargo xtask form export <node>... [--layer <sid> | --group <id>] [--out <dir>]`.
- **From the software.** The web face's node page has "Ask for a change", which downloads that node's form from `GET /v1/form/<node>`. The portal serves node forms to internal roles only, because layer 3 is restricted from clients (D1).
- **Seed forms.** `tools/forms.py seeds` writes the 82 seed forms from `plan/seed_content.toml` (§5.8). They exist for the first build only.
- **Determinism.** An exported form of one node at one release is byte-identical every time. Its only date is the release's, when `SOURCE_DATE_EPOCH` is set, as the release sets it. CI compares two exports.

#### 5.10.6 Where a request is sent

A request reaches the developer team by any route that carries a file. Every route ends in the same place: the file, checked by `cargo xtask intake check`.

- **An issue**, with the "Request" issue template (`.github/ISSUE_TEMPLATE/request.yml`), which asks for the file and nothing else.
- **The portal**, for internal roles: `POST /api/v1/internal/requests` stores the file and opens the issue (§15.5).
- **Mail or chat** to the developer team, who attach it to an issue.

#### 5.10.7 Adding a request kind

A node form covers nodes, and "something else" covers the rest by hand. When one kind of "something else" becomes common, it earns its own form: a part's descriptor, an algorithm's bounds, a scenario. A kind is:

- an exporter that fills a template's JSON block;
- a template, or a branch in `node_form.html`, whose page states the kind's rules;
- a checker in `adcs-intake`, with its codes, and a writer and verifier for the files it changes.

None is built until a team needs it. A new kind is two reviewers, because its writer changes the repository.

#### 5.10.8 Acceptance

`tools/form_browser_check.py` runs all of this in headless Chromium, and this package's own run passes:

- A node form (`gf_7`) draws with no script error and reads "unchanged". Two edits, a name and a reason read "2 changes", and the page flags the missing belief record (D01) until the De-risking section is filled; then nothing is left to fix.
- The depth switch hides "Start here" in Expert, and in Learn hides the rebuilt test vector's answer until the reader asks, then judges the reader's answer against the tolerance.
- "Save filled copy" writes a file that reopens with both edits. `tools/intake.py` passes that file, reads back exactly the two changes, and writes a brief.
- An assistant's name as requester is flagged on the page, and refused by the checker (P01).
- A reply written with `intake reply` shows in the returned file's status card and header.
- A new-node request fills its id from the group and label.
- Unsaved answers are offered back after a reload.
- At 390 px, the phone width, the form has no sideways scroll.

- The node library draws, lists every exported node, and its search narrows the list.

The case editor's and the results' checks are in §8.3.6 and §13.5.6.

### 5.11 Intake: how a request becomes software

#### 5.11.1 The pipeline

Intake is the developer team's side of the node form. It is `cargo xtask intake` over the crate `adcs-intake`, run in a checkout. `tools/intake.py` is its stand-in in this package, and runs today on the plan.

```
request file ─▶ 1 CHECK ─▶ 2 IMPLEMENT ─────────────────────────▶ 3 VERIFY ─▶ 4 REVIEW ─▶ 5 RELEASE ─▶ reply
                │           implementation agent, from the brief:               owner,       next
                │           intake write (sheet, test vectors),                 gate,        release
                │           the HOLEs, new physics functions                    downstream
                └─ fail ─▶ reply "check failed" with every finding ─▶ the team member fixes and resends
```

| Step | Command | Who | Writes |
|---|---|---|---|
| 1 Check | `cargo xtask intake check <file> [--out intake/requests]` | a developer | `intake/requests/<request id>/`: `request.json` (the trusted part), `check.md` and `check.json` (the report), `attachments/`, and, on a pass, `brief.md` (change, new) or `confirm.md` (confirm) |
| 2 Implement | the implementation agent, started by a developer with `brief.md` (§17.4); its first step is `cargo xtask intake write request.json` | the agent, watched by the developer | the node's `node.toml`, `fixtures.toml`, `versions.toml` and `versions/<n>/`, `derisk/beliefs/<request id>.toml` and the moves in `derisk/risks.toml` (by `intake write` only); the node's HOLE blocks; new functions in `adcs-core::physics` with their property tests, each with its MATLAB twin in `matlab_sils/+asils/+physics/` (§10.8.7); for a new node, `cargo xtask new <id> --like <sibling>` first |
| 3 Verify | `cargo xtask intake verify request.json`, then `cargo xtask intake mark verified` | the agent, then CI again, which marks the node `verified` when verify and the gate pass | only `state`, through `mark` |
| 4 Review | the pull request of branch `intake/<request id>` | a developer from the node's owner group (CODEOWNERS, §5.9), plus a second reviewer for anything H7 or significant | approval, or a reply "needs information" |
| 5 Release | the ordinary release (§18), which runs `intake mark published --release <v>` | the developer team | the new version, with the request listed in its release notes; the node `published` |
| Reply | `cargo xtask intake reply <file> --status <s> [--note] [--release] [--report check.json] --out <file>` | a developer | the returned file, with the reply in its `history` |

The request id is `REQ-<date>-<node>-<hash>`, from the request's content. It names the working folder, the branch, the commit trailer `Request:`, the copy of the request kept in the node folder (`requests/<request id>.request.html`), and the release note's line. So a node's history is its requests, and every change can be traced to the person who asked, their reason and their test vectors.

**Feedback and "something else"** go through step 1 only. A passing check turns each feedback entry into an issue (`cargo xtask intake issue <file>`, with the node, the release, the case and result named, and the file attached), labelled `request:feedback` and assigned by CODEOWNERS. "Something else" becomes one issue, labelled `request:other`, for the developer team to plan like any of their own work. Both get a reply.

#### 5.11.2 The checker

The checker reads only the JSON block of the file. It never opens the page, so a received file's script never runs on a developer's machine. It decides everything from the node as the repository has it now, never from what the file says the node was. It applies every rule below, prints the report, and exits 0 only when nothing is an error. `python3 tools/intake.py codes` lists them.

| Code | The rule |
|---|---|
| F01 | the file holds an `adcs-node-form/1` block, and is no larger than 25 MB |
| F02 | the request type is change, new, confirm, feedback or other (seed only with `--seed`, at the first build) |
| F03 | the node exists and is offered for change (not a door, an interface or a closure); or a new node's id is free, well formed and prefixed for its layer (`mgt_`, `sys_`, or `l3_<sid>_`) |
| F04 | a change or a confirmation was exported from the node as it is now; otherwise the report names every field that has moved since, and the team member exports a fresh form. Feedback and "something else" may come on an older copy. |
| F05 | a change changes something; a confirmation changes nothing |
| P01 | the requester is a person, named, and not a tool or an assistant |
| P02 | a change, a new node and a confirmation say why |
| P03 | whoever is named as having checked the maths is a person; a seed form names nobody |
| P04 | a confirmation names who checked the node |
| N01 | a new node's group exists (holding nodes, or declared in `layers/` with none yet), and its model sibling is in that group |
| N02 | a new node's kind is computed, declared, required or achieved |
| I01 | an existing node's id, kind, layer, group and owner do not change through a form |
| I02 | a node fed by the case format keeps its label, quantity and unit: that is a case-format change (§8.3.5) |
| I03 | hardware tags are `mtq`, `rw`, `fmr` or `rcs` |
| O01 | the answer has a symbol of letters, digits and _ |
| O02 | the answer's quantity is one the software knows |
| O03 | the answer's unit is one the software knows, and it states that quantity |
| O04 | the lowest and highest values are numbers, in order, and each has a written reason |
| C01 | a computed node reads at least one input; any other kind reads none |
| C02 | each input has a unique binding and reads a node that exists, other than itself |
| C03 | each input's declared quantity is what its producer publishes |
| C04 | inputs stay inside the node's layer: layers meet only at the door and the interface rows |
| C05 | the new connections make no loop |
| C06 | "zero when absent" names hardware count nodes (`cf_*`) the node reads |
| C07 | *warning*: an input's producer has no answer specified yet; implementation waits for it |
| R01 | a computed node has a relation, a source, a reason why, and at least one step |
| R02 | each step says what it does, and names a physics function that exists or describes the new one it needs |
| R03 | the last step binds the answer's symbol |
| R04 | *warning*: the relation uses names that are neither bindings nor step results nor ordinary maths |
| R05 | *warning*: a named physics function takes a different number of arguments than the node has inputs |
| R06 | each assumption says what it assumes and when that stops being true |
| R07 | each derivation step has text |
| V01 | a declared node that no case, product, tuning, evidence, ledger or lab file supplies has a numeric value inside its bounds, and a source; a product or tuned row may leave its reference value blank; a value the evidence tooling, the de-risking ledger or a lab file supplies is never set by a form |
| V02 | a requirement says which way it binds |
| V03 | an evidence node names a metric the simulator computes and at least one rung |
| S01 | every cited source is known, or added in the form with an id, a title and exactly where |
| T01 | a test vector's provenance is independent-derivation, published-source, independent-tool or physical-bound; never self-snapshot or agent-generated |
| T02 | a test vector cites a known source and the page, table or figure |
| T03 | a test vector gives one number per binding, a numeric expected answer and a tolerance above zero |
| T04 | *warning*: a computed node has no test vector, so its validation stays low |
| A01 | attachments are plain file names, 5 MB in all |
| B01 | a feedback entry says what happened |
| B02 | an "other" request has a subject and a description |
| W01 | *warning*: nobody is named under "Checked by", so the node will run as UNCONFIRMED |
| D01 | a change or a new node carries its belief record: its area (one of seven), what was believed, what we now know, and what changed in the plan |
| D02 | a change says what was wrong with the version it replaces and what this version gives |
| D03 | the belief's status is broke, held or untested; broke and held say what tested it; untested says the test that would settle it |
| D04 | each risk it moves is in the register at the level it moves from, or is new (from 0) with a title, an area, a level 1 to 5 and the test that would close it |
| D05 | a risk is lowered or closed only by a tested belief |
| D06 | the cost is blank or a number, zero or more, in thousands of dollars |
| D07 | *warning*: the belief is untested, and is counted under "Beliefs not yet tested" until a test is recorded |
| X01 | an analogy in the node's explanation says where it stops being true |
| X02 | a common wrong idea says why it is wrong |
| X03 | *warning*: a computed node does not say itself simply and in one line, so its document serves experts only |

**The report** (`check.md`) gives the verdict, every finding with its code and section, what changes (field, now, proposed), the connections added and removed, the **impact** (every node downstream of this one, which the gate re-tests, and the KPIs it feeds), any new physics functions needed, and the **de-risking** table: the belief, its status and test, the previous version's issue, this version's benefit, and the risks moved. The developer reads the report before anything else. A pass means the request is complete and consistent with the tree. It does not mean the relation is right: that is what the engineer under "Checked by" attests, and what the test vectors test.

**Interfaces are checked against the tree, not the form.** The producer of every input, the quantity it publishes, its layer, and the loop check all come from the repository. So a form cannot connect a node to something that does not exist, or in a way the tree forbids, however it was edited.

#### 5.11.3 The implementation agent's brief

On a pass, the checker writes `brief.md`. It is the implementation agent's whole task, and nothing outside it is in scope (§17.4):

- **scope**: the node, its crate, and the files it may change: `node.toml`, `fixtures.toml`, `versions.toml`, `versions/` and `derisk/` only through `cargo xtask intake write`, the node's HOLE blocks, and `adcs-core::physics` with its MATLAB twin `matlab_sils/+asils/+physics/` only for the new functions the request describes;
- **in order**: `intake write`; `cargo xtask docs <node>`; each HOLE, one per step, calling the physics function the step names with the bindings in declared order (or writing the new function first, in the style of `adcs-core::physics`, with property tests only, and its MATLAB twin in the same commit, run on the request's test vectors); `intake verify`; the gate and the tests, including every downstream node; a commit on `intake/<request id>` with the trailer `Request:`, the request file copied into the node folder; no push;
- **never**: supply an expected value; write a person's name; widen a tolerance or skip a test; put a formula in a HOLE (F7's check 10b enforces it); edit outside the scope. If the request cannot be implemented as written, the agent stops and says why, and the developer replies to the requester.

#### 5.11.4 The sheet writer and the verifier

**`cargo xtask intake write request.json`** is the only path by which a request's declarative content reaches a sheet, a node's version history, or the risk ledger. A requirement is written `kind = "declared"` with a top-level `sense` (§5.5). It renders `node.toml` and `fixtures.toml` in the sheet shape with `form.rs`'s pure `set`, `normalise` and `value_allowed`: the question and note; the relation and source; the theory, its steps and the assumptions; the output with its bounds and reasons; a declared value; a requirement's sense; an evidence row's metric and rungs; the inputs; the algorithm steps; hardware tags; zero-when-absent; `[request] last = "<request id>"`; and `confirmed_by` from `attested_by`, or `UNCONFIRMED · via <request id> · awaiting a person` (§5.8). It sets the state to `specified`. In confirm mode it writes only the `confirmed_by` fields and `[request] last`. It refuses a request that does not pass the check. `tools/intake.py sheet` is its stand-in; the stand-in reads the form file, where the repository's command reads `request.json`.

Besides the sheet it writes what §5.13 names: the previous sheet copied to `versions/<n>/` and version *n + 1* appended to `versions.toml` (request id, belief, previous issue, benefit, `release` empty until `mark published`); the belief record `derisk/beliefs/<request id>.toml`; and each risk move as a history entry of `derisk/risks.toml`, a new risk taking the next free `R-nn`. The node's `[explain]` table (§5.12.3) is written with the sheet. A confirmation writes no version; a held belief it carries is recorded in `derisk/beliefs/`. The package's `tools/intake.py sheet` writes the three ledger parts as `belief.toml`, `versions.entry.toml` and `risk_moves.toml` beside the sheet.

**`cargo xtask intake verify request.json`** compares the node, as implemented, with the request, field by field:

- every field of the sheet, including `[explain]`;
- the version entry and the belief record, against the request's De-risking section;
- every test vector in `fixtures.toml`;
- each HOLE calls the physics function its step names;
- `confirmed_by` is the attesting person's, or UNCONFIRMED.

Any difference fails it, by field. It never compares `state`. It runs again in CI on the intake branch, and when it and the gate pass, CI runs `intake mark verified`. `tools/intake.py verify` is its stand-in for the sheet and the test vectors.

#### 5.11.5 What intake never does

- It never runs the page of a received file.
- It never writes a node from anything but a passing request, and never changes a node the request does not name.
- It never merges or releases. The only commits it makes itself are the `state` commits of `intake mark`: the intake CI job's on an intake branch, under the CI's own identity, never a person's, and the release's preparation commit. A person reviews every change (H1 for a relation, H2 for a test vector).
- It never supplies a name, a number or a relation. What the software says comes from people, through forms; how the software computes it comes from the implementation agent under a brief, verified against the form.

#### 5.11.6 Acceptance

`python3 tools/intake.py selftest` passes in this package:

- 66 deliberate mistakes, at least one for every rule that is an error (the D and X rules included), are each refused by their code;
- 88 clean requests pass: a real change with its broken belief, a change whose broken belief lowers one risk and opens another, a new node on an untested belief, a confirmation, feedback (including feedback on an older copy), and all 82 seed forms;
- all 82 sheets written from the seed forms verify against their own requests;
- a sheet with one unit changed, a test vector with its expected value changed, and a belief record with its status changed are each caught by verify; the moving change writes version 2 of `gf_7` and names R-04 and the new R-18.

In the built repository, the `intake` CI job runs the same selftest against `adcs-intake`, on a scratch tree freshly seeded from `plan/` (so the seed forms always meet `seeded` nodes, whatever the repository's own nodes now say), and the 82 seed forms go through the full pipeline in P1 (§19).
