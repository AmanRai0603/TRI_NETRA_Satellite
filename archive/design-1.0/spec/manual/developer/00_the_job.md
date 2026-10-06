# The developer team's job
<!-- kind: explanation; depth: read -->

**In one line:** the developer team is the only way anything changes in the ADCS platform: team members use the released software with their own case CSVs, and everything they want different reaches you as a node form, which you check, implement with the implementation agent, verify, review, release and answer.

## Say it simply

The software is a tool the whole company uses and nobody but you edits. A team member's input is a case; their voice is a node form. Your job is to turn each form into software, or into a clear answer, and to keep the software, the explanations and the risk ledger true while you do.

Think of it like a library's cataloguing desk: anyone may suggest a correction to a catalogue card, but only the desk writes cards, and every card says who asked, why, and which source backs it.

**Where the story lies:** a library card does not compute anything. A node does, and it is tested: a change to one node re-runs the tests of every node downstream of it.

## Now the real thing

**What you own.** The repository: the engine, the tree, the catalogue, scenarios and campaigns, the views, the tools, CI and releases; **intake**, turning every request into software ([Intake](01_intake.md)); the **risk ledger**, which every change feeds ([De-risking](08_derisking.md)); the **MATLAB twin**, which every SILS change updates in the same pull request, so the MATLAB SILS zip is whole at every push ([The MATLAB twin](09_twin.md)); the two manuals and every template, which you keep true and written to the explanation standard ([Explaining](07_explaining.md)).

**Two kinds of work.**

| Kind | Starts from | Path |
|---|---|---|
| a request | a node form from a team member | check → implement (the implementation agent, on a brief) → verify → review → release → reply |
| your own work | the team's plan: the engine, views, tools, catalogue, scenarios, the case format | an ordinary branch and review, with a belief record ([Your own changes](04_own_changes.md)) |

Node content (a node's question, relation, inputs, bounds, assumptions, values, explanation and test vectors) is never your own work. It comes only from a request, through `cargo xtask intake write`. When you want a node different, you send a node form like anyone else, and another developer takes it through intake.

**The rules nothing bends.**

1. **The sheet is the only source,** and `intake write` is the only writer of its content. The seeder and `xtask new` write only a sheet's shape; `intake mark` writes only its state.
2. **An expected value never comes from the code under test.** Test vectors come from a person, transcribed from a cited page. The gate refuses `self-snapshot` and `agent-generated`.
3. **Every formula lives in `adcs-core::physics`.** A HOLE composes physics calls on its bindings, and nothing else.
4. **Portable maths only**: `pmath`.
5. **A refusal is never a substitution.** A blank, a `nan`, a missing theory: refused by name.
6. **Never write a person's name.** `confirmed_by` comes from a form's "Checked by", through `intake write`, or it is UNCONFIRMED. Any other person's decision is copied from their own record by `cargo xtask decision record`.
7. **Never widen a tolerance, skip a test, or edit a generated file outside a HOLE.**
8. **The released software has no write path.** The `no-writes` CI job enforces it.
9. **Every change rests on a recorded belief.** A version is made because a belief broke or a risk needs lowering, and a risk level goes down only by a recorded test.
10. **Every page a person reads follows `adcs-explain/1`.** The `explain` CI job checks the marks; a reader's teach-back checks the rest.

**Tools you will use every day.**

<!-- since P1 -->
```
cargo xtask intake check <request.html> --out intake/requests    # the checker
cargo xtask intake write <request.json>                          # sheet, version, belief, risk moves
cargo xtask intake verify <request.json>                         # the node says what the request says
cargo xtask intake mark verified <request> | published --release <v>
cargo xtask intake reply <request.html> --status <s> --out <file>
cargo xtask derisk check | record | rollup | narrative           # the ledger
cargo xtask explain check                                        # the explanation standard
cargo xtask gate
cargo xtask form export <node> | --library <dir> | --case
```

In this package their stand-ins are `tools/intake.py`, `tools/derisk.py`, `tools/explain_check.py`, `tools/forms.py`, `tools/check_case.py`, `tools/results.py` and `tools/validate_plan.py`.

## Where the simple version breaks

- **A passing check is not a right relation.** The checker proves the request is complete and consistent with the tree. Whether the physics is right is what "Checked by" attests and the test vectors test. When a relation looks wrong to you, reply "needs information"; do not fix it yourself.
- **One team is a bottleneck.** The checker's speed and the brief's narrow scope keep it small; if the queue grows, the team grows, never the number of paths into the software (risk R-02).
- **The agent writes code, not judgement.** It works only inside the brief, and you are responsible for what it commits.

## Common wrong idea

"As a developer I can fix a node's relation directly; it is faster." It is faster once. It also leaves a relation nobody asked for, with nobody's reason, no belief record and no request id, in software every other team trusts because every change has all four. Send the form; another developer takes it through intake in minutes.
