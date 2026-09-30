# Implementing a request with the implementation agent
<!-- kind: how-to; depth: read -->

**In one line:** on a branch `intake/<request id>`, start Claude Code on the request's `brief.md`, watch it run `intake write` (sheet, version, belief, risk moves), write the HOLEs and any new physics function, verify, run the gate, and commit without pushing.

The brief is the whole task. The implementation agent is Claude Code started in your checkout: one general agent, not a specialist. Its standing instructions are `AGENTS.md` and `intake/AGENT.md`, and the scope hook in `.claude/hooks/` refuses any write outside the brief's scope and refuses `node.toml`, `fixtures.toml`, `versions.toml` and `derisk/` to its editor altogether.

## Start it

1. `git switch -c intake/<request id>`
2. Start Claude Code in the checkout.
3. Tell it: "Implement `intake/requests/<request id>/brief.md`."
4. Watch it. You are responsible for what it commits.

## What it does, in order

1. For a new node: `cargo xtask new <id> --like <sibling>` under the named group.
2. `cargo xtask intake write request.json`. It writes, exactly as requested:
   - the sheet and its test vectors: question, relation and source, theory and assumptions, the output and its bounds with reasons, a declared value, a requirement's sense (`kind = "declared"` with `sense`), an evidence row's metric, inputs, steps, tags, zero-when-absent, the node's `[explain]` table, `[request] last`, and `confirmed_by` (the name under "Checked by", or `UNCONFIRMED · via <request id> · awaiting a person`); the state becomes `specified`;
   - the node's history: the previous sheet copied to `versions/<n>/`, and version *n + 1* appended to `versions.toml` with the request id, the belief, what was wrong before and what this version gives;
   - the ledger: `derisk/beliefs/<request id>.toml`, and each risk move as a history entry in `derisk/risks.toml` (a new risk gets the next free `R-nn`).
3. `cargo xtask docs <node>`, which regenerates the node's artefacts.
4. Each HOLE, one per step: `let <binds>: <Type> = physics::<module>::<fn>(<bindings in declared order>);`, or one slice when the step says so.
5. A **new physics function**, if the brief lists one, in `adcs-core/src/physics/<module>.rs`, in one style: `no_std`, typed arguments, `pmath` only, a doc comment with the relation and its source id, zero at count zero where it takes a count. **Property tests only**: dimensions, monotonicity, zero at count zero; never an invented expected value. **In the same commit, its MATLAB twin** `matlab_sils/+asils/+physics/+<module>/<name>.m`, from the same relation and source, with the same argument order and SI units; the request's test vectors run in both engines ([The MATLAB twin](09_twin.md)).
6. `cargo xtask intake verify request.json`: the sheet, the test vectors, each HOLE's call, the version entry and the belief record must match the request, field by field.
7. `cargo xtask gate`, `cargo test --workspace`, `cargo xtask derisk check` and, when a new function was written, `cargo xtask twin check`, including every node downstream (listed under "Impact" in `check.md`).
8. It copies the request file into the node folder as `requests/<request id>.request.html` and commits with the request's reason as the body and the trailer `Request: <request id>`. It does not push.

## What it never does

- supply an expected value, a relation, a name, or a risk level;
- widen a tolerance, skip a test, or edit outside the brief's scope;
- put a formula in a HOLE;
- push, merge or release.

If the request cannot be implemented as written, it stops and says why. You then reply "needs information" with that reason. Never adjust the request yourself.

## If it goes wrong

| Symptom | Cause | Do |
|---|---|---|
| `intake verify` differs on a field | something edited the sheet, the version entry or the belief besides `intake write` | reset the node folder and `derisk/`, and run `intake write` again |
| a test vector fails | the implementation, the physics function, or the vector is wrong | never change the vector; check the HOLE and the function against the relation; if the vector itself is doubtful, reply "needs information", naming its page |
| a downstream node fails | the change moved a number something else relies on | if the change is right and the downstream test vectors are now wrong, that needs its own request, and that node's owner must know |
| gate check 10b refuses a HOLE | the HOLE computes a relation | move the relation into a physics function |
| `derisk check` fails after `intake write` | another branch moved the same risk | rebase, and re-run `intake write` on the current register; if the move no longer starts where the risk is, reply "needs information" |

Then push the branch and open the pull request ([Review and release](03_review_and_release.md)).
