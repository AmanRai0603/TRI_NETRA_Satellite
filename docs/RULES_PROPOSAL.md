# The rule change for 1.0.0 (proposed, awaiting the owner's approval)

> **Describes 1.0.0.** Replaced for 2.0.0 by the design's rules in `design/rules_2_0.toml` (described in `docs/OPERATING_2_0.md`) and the code's rules in `docs/CODE_ARCHITECTURE.md` §4.


**In one line:** the spec says a node's content comes only from a node form checked by intake and written by the developer team; this proposes that it comes from its group's sealed release instead, with the node form kept as a way in, and SPEC §3.2 and §5.10–5.11 rewritten to say so (`docs/RELEASE_PLAN.md` P9).

**Status:** proposed. SPEC.md is not changed until the owner approves this page. `CONTRIBUTING.md` and `docs/CHANGING.md` already describe the new way for the apps; the spec's text still describes the old one.

## What changes

| | Today (SPEC §3.2, §5.10–5.11) | Proposed |
|---|---|---|
| Who writes a node's content | the developer team, with `intake write`, from a team member's node form | the node's author, in the node app |
| Who checks it | the intake checker (54 checks), then a reviewer | the same checks, live in the node app; a second engineer signs it as checked; the stage owner signs the stage |
| What makes it official | a merged pull request | the group lead's seal: a frozen release file |
| How it reaches the software | the sheet in the repository | `tools/group.py verify` and `merge` take sealed releases into `design.tndb`; code is generated from it (P10) |
| The node form | the only way in | one way in: the group app imports it into the node file, keeping what the author typed |
| Confirmed | a reviewer's attestation | checked by someone other than the author, stage signed, no problem, and, for a computing node, an outside test vector |

## The five rules, rewritten (SPEC §3.2)

1. **A node's content comes only from its group's sealed release.** The node app writes it, a second engineer checks it, the stage owner signs the stage, the lead seals it; `group merge` is the only way it enters `design.tndb`. Its shape (group, stage, label) comes from the group file.
2. **An expected value may never come from the code under test.** A computing node without an outside test vector is never sealed as confirmed (unchanged in spirit; now enforced at the seal).
3. **Every formula lives in one place:** the physics in pseudocode (`spec/physics/`), translated to Rust and MATLAB; the flight algorithms in `fsw/pseudocode/`, held to C and Rust.
4. **Portable maths only** (unchanged).
5. **A refusal is never a substitution** (unchanged).

## §5.10–5.11, in short

- §5.10 (the node form) becomes "asking for a change": the node form remains for anyone without a node to write in; the group app imports it (P6), and its checker codes stay the node app's live checks.
- §5.11 (intake) becomes "from release to software": verify (`group.py verify`, `release.py check`), merge (`group.py merge`), generate and test (P10), accept (P12).

## What the owner decides

1. Approve, change or refuse this rule change.
2. Once approved, the developer team rewrites SPEC §3.2 and §5.10–5.11 to this text (`bash tools/assemble_spec.sh` regenerates SPEC.md), in one pull request.
