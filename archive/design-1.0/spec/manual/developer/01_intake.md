# Intake: receiving and checking a request
<!-- kind: how-to; depth: read -->

**In one line:** run the checker on the request file without opening it, read `check.md`, then either reply "check failed" with its findings, or reply "check passed" and hand the brief to the implementation agent.

A request is a saved node form, `<node>.request.<date>.html`. It arrives as an issue with the "Request" template, in the portal's requests inbox, or by mail or chat, which you attach to an issue. Every route ends in the same command.

## Check a request

1. **Run the checker. Never open the file first.**

<!-- since P1 -->
```
cargo xtask intake check <file> --out intake/requests
```

   It reads only the file's JSON block, so a received file's script never runs on your machine. It decides everything from the node as the repository has it now, never from what the file says the node was.

2. **Read `intake/requests/<request id>/check.md` first.** It gives the verdict, every finding with its code, what changes, the connections added and removed, the impact downstream, any new physics function, and the **De-risking** table: the belief the request rests on and the risks it moves.

| File | Is |
|---|---|
| `request.json` | the trusted part of the request: what `intake write` and `intake verify` read |
| `check.md`, `check.json` | the report |
| `attachments/` | the requester's files, decoded |
| `brief.md` | only on a pass of a change or a new node: the implementation agent's task |
| `confirm.md` | only on a pass of a confirmation: the fields `intake write` will set; no agent, no code |

3. **Decide.** A pass means the request is complete and consistent with the tree and the ledger. It does not mean the relation is right: that is what "Checked by" attests and the test vectors test. If nobody is named under "Checked by", the node is released UNCONFIRMED; that is allowed, and visible.

## Reply "check failed"

1. Write the returned file:

<!-- since P1 -->
```
cargo xtask intake reply <file> --status "check failed" --report intake/requests/<id>/check.json \
    --note "Two fixes needed; see the findings." --out <node>.request.<date>.returned.html
```

2. Send it back by the route it came. Its status card lists every finding; the requester fixes the same file and sends it again.

## Reply "check passed" and go on

1. **Change or new node:** reply "check passed", then implement ([Implementing](02_implementing.md)).
2. **Confirmation:** reply "check passed"; on a branch `intake/<request id>`, run `cargo xtask intake write request.json` yourself (in confirm mode it writes only `confirmed_by`, `[request] last`, and the held belief if the form carries one), then `intake verify` and the gate, and open the pull request.
3. **Feedback** and **something else:** `cargo xtask intake issue <file>` opens the issues (`request:feedback`, `request:other`). Reply "received" with the issue link. "Something else" is planned like your own work ([Your own changes](04_own_changes.md)).
4. **Seed** (P1 only): as a change, with `--seed`; the checker refuses a seed form for a node that is no longer `seeded` ([The first build](05_seed_intake.md)).

Feedback may arrive in an older copy of the form: the checker does not hold feedback to the node's current base. A change or a confirmation must come on a fresh form (F04).

## Statuses

`received`, `check failed`, `check passed`, `needs information`, `in progress`, `in review`, `released`, `rejected`, `feedback noted`. Every reply names the release when there is one; a rejection always says why.

## If it goes wrong

| Situation | Do |
|---|---|
| the check passes but the relation looks wrong to you | do not implement it; reply "needs information", say what you doubt, and ask the requester or the engineer under "Checked by" to confirm it with its source |
| the request is complete but not wanted | reply "rejected" with the reason |
| D04: a risk move starts from a level the register no longer has | another release moved that risk; the requester exports a fresh form and re-reads the risk |
| D05: a request lowers a risk with an untested belief | reply "check failed": a level goes down only by a recorded test |
| C07 warns that an input has no answer yet | implementation waits for that input's own request; say so in the reply |
| the file will not parse (F01) | ask for the file again; never repair its JSON yourself |

## Every check

These are the checker's rules, with their codes. The table is generated from the checker, and CI fails if the two differ.

<!-- codes:begin -->
| Code | Level | The rule |
|---|---|---|
| F01 | error | the file holds an adcs-node-form/1 JSON block and nothing larger than 25 MB |
| F02 | error | the request type is change, new, confirm, feedback or other (seed only with --seed) |
| F03 | error | the node named exists and is offered for change (not a door, interface or closure), or the new node's id is free, well formed and prefixed for its layer (mgt_, sys_, l3_<sid>_) |
| F04 | error | a change or confirmation was exported from the node as the software has it now (its base is current); feedback may come on an older copy |
| F05 | error | a change changes something; a confirmation changes nothing |
| P01 | error | the requester is a person, named, and not a tool or an assistant |
| P02 | error | a change, a new node and a confirmation say why |
| P03 | error | whoever is named as having checked the maths is a person, not a tool or an assistant (a seed form names nobody) |
| P04 | error | a confirmation names who checked the node |
| N01 | error | a new node's group exists (holding nodes, or declared in layers/ with none yet), and its model sibling is in that group |
| N02 | error | a new node's kind is computed, declared, required or achieved |
| I01 | error | the kind, layer, group, owner and id of an existing node do not change through a form |
| I02 | error | a node fed by the case format keeps its label, quantity and unit (a case-format change is its own process) |
| I03 | error | hardware tags are mtq, rw, fmr or rcs |
| O01 | error | the answer has a symbol of letters, digits and _ |
| O02 | error | the answer's quantity is one the software knows |
| O03 | error | the answer's unit is one the software knows, and it states that quantity |
| O04 | error | the lowest and highest values are numbers, in order, and each has a written reason |
| C01 | error | a computed node reads at least one input; any other kind reads none |
| C02 | error | each input has a unique binding name and reads a node that exists, other than itself |
| C03 | error | each input's declared quantity is what its producer publishes |
| C04 | error | inputs stay inside the node's layer (layers meet only at the door and the interface nodes) |
| C05 | error | the new connections make no loop |
| C06 | error | zero-when-absent names hardware count nodes the node reads |
| C07 | warning | an input's producer has its answer specified (warning: implementation waits for it) |
| R01 | error | a computed node has a relation, a source, a reason why, and at least one step |
| R02 | error | each step says what it does and names a physics function that exists, or describes the new one it needs |
| R03 | error | the last step binds the answer's symbol |
| R04 | warning | the relation's names are the bindings, the step results and ordinary maths (warning) |
| R05 | warning | a named physics function takes as many arguments as the node has inputs (warning) |
| R06 | error | each assumption says what it assumes and when that stops being true |
| R07 | error | each derivation step has text |
| V01 | error | a declared node that no case, product, tuning, evidence, ledger or lab file supplies has a numeric value inside its bounds, and a source; a product or tuned row may leave its reference value blank; a value the evidence tooling, the de-risking ledger or a lab file supplies is never set by a form |
| V02 | error | a requirement says which way it binds (<= or >=) |
| V03 | error | an evidence node names a metric the simulator computes and at least one rung |
| S01 | error | every cited source is known, or added in the form with an id, a title and exactly where |
| T01 | error | a test vector's provenance is independent-derivation, published-source, independent-tool or physical-bound |
| T02 | error | a test vector cites a known source and the page, table or figure |
| T03 | error | a test vector gives exactly one numeric input per binding, a numeric expected answer and a tolerance above zero |
| T04 | warning | a computed node has at least one test vector (warning: validation stays low) |
| A01 | error | attachments are named files, 5 MB in all |
| B01 | error | feedback entries say what happened |
| B02 | error | an 'other' request has a subject and a description |
| W01 | warning | nobody is named as having checked the maths: the node will run as UNCONFIRMED (warning) |
| D01 | error | a change or a new node carries its belief record: its area (node, input, output, model, math, algorithm, visualisation), what was believed, what we now know, and what changed in the plan |
| D02 | error | a change says what was wrong with the version it replaces and what this version gives |
| D03 | error | the belief's status is broke, held or untested; broke and held say what tested it; untested says the test that would settle it |
| D04 | error | each risk it moves is in the register at the level it moves from, or is new (from 0) with a title, an area, a level 1 to 5 and the test that would close it |
| D05 | error | a risk is lowered or closed only by a tested belief (broke or held) |
| D06 | error | the cost is blank or a number, zero or more, in thousands of dollars |
| D07 | warning | the belief is untested: it is counted under Beliefs not yet tested until a test is recorded (warning) |
| X01 | error | an analogy says where it stops being true |
| X02 | error | a common wrong idea says why it is wrong |
| X03 | warning | a computed node says itself simply and in one line (warning: its document otherwise serves experts only) |
<!-- codes:end -->
