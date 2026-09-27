# Review, release and reply
<!-- kind: how-to; depth: read -->

**In one line:** review the intake branch against its request and its belief record, merge it, let the release mark the node published and roll the risk ledger up into the Risk management rows, then reply "released" to every requester in the release.

## Review an intake branch

CODEOWNERS routes the pull request of `intake/<request id>` to the developer group that owns the node. One reviewer is enough; two when the node is `significant`, when a new physics function is added, when the request adds or removes a connection, or when it lowers a risk.

1. Check that `intake verify`, the gate and `derisk check` are green in CI, and that the `intake` job has re-checked the request.
2. Check the diff touches only the node's folder (sheet, fixtures, `versions.toml`, `versions/`, HOLEs), `derisk/` through `intake write`, and `adcs-core::physics` for a new function.
3. Check every HOLE composes physics calls on its bindings and nothing else.
4. Check a new physics function's doc comment matches the request's relation and cites its source, and its tests are property tests.
5. Check the belief record says what the request says, and that any lowered risk names what was tested.
6. Check the request file is in the node folder and the commit trailer names it, and that `confirmed_by` is the "Checked by" name or UNCONFIRMED.
7. Reply "in review" while it is reviewed; merge when approved.

The reviewer does **not** re-judge the physics on the requester's behalf. If the relation looks wrong, reply "needs information" instead of merging.

The intake CI job marks the node `verified` (`intake mark verified`) when verify and the gate pass on the branch.

## Release

1. `release.yml` proves, builds and publishes, with its fail-closed approval. Its artefacts are the CLI, the daemon, the FFI library, the rig runner, the portal image, `adcs_sils_matlab_<version>.zip`, the node library, the user manual, and the quarter's de-risking narrative.
2. Its preparation commit runs `intake mark published --release <version>` for every node whose request the release carries (stamping the release on each new version), and `cargo xtask derisk rollup` into `derisk/rollup.toml`, which the 18 declared Risk management rows read in that release.
3. The **release notes list every request** the release carries, by request id and node, with the belief each rested on, so each requester can find theirs.

## Reply "released"

1. For every request in the release:

<!-- since P1 -->
```
cargo xtask intake reply <file> --status released --release <version> \
    --note "Released. Run your case in <version> and send feedback in this file if it is not right." --out <file>
```

2. Send it back the way it came, or post it in the portal inbox.
3. The team member tests the change with their own case and result, and either stops or sends feedback in the same file.

## If it goes wrong

| Situation | Do |
|---|---|
| the gate fails on a node the request did not touch | a downstream test vector disagrees with the change; do not merge; see [Implementing](02_implementing.md), "a downstream node fails" |
| two intake branches move the same risk | merge the first; the second's `derisk check` fails; its developer re-runs `intake write` on the new register |
| the rollup changes a Risk management row nobody expected | read the ledger diff of the release; the narrative lists every belief of the quarter |
| a requester says the release did not do what they asked | ask for feedback in the same file, with their case and result attached |
