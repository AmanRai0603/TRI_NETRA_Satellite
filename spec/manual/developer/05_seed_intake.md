# The first build: taking the seed forms through intake
<!-- kind: how-to; depth: read -->

**In one line:** at P1, write the 82 seed forms from `plan/seed_content.toml` (the pilot thread's 61 rows and the risk branch's 21) and take each through the same intake a team member's form takes, in dependency order, so the path every later request follows is the path that built the tree.

Seeding gives the tree its shape: every row's identity, place, owner, kind, and the edges the tree declares. It gives no content; every seeded row answers `NotRun`. The seed content was written from cited sources by the planning assistant that wrote the package, which is why none of it is attested.

## Take the seed forms through intake

1. Write the forms:

<!-- since P1 -->
```
python3 tools/forms.py seeds --out intake/seed        # 82 seed forms
```

2. For each form, in dependency order (a row after the rows it reads):

<!-- since P1 -->
```
cargo xtask intake check <form> --seed --out intake/requests
implementation agent on brief.md                   # intake write, the HOLEs, any new physics function
cargo xtask intake verify request.json
cargo xtask gate
one intake branch, reviewed and merged
```

3. `--seed` accepts the request type `seed` and the package's own requester, at the first build only. A seed form never names anyone under "Checked by" (P03), so every seeded row is released UNCONFIRMED, and `xtask ready` holds it.
4. Each seeded row's `versions.toml` starts at version 1, "first build", with no belief record of its own; the beliefs about the pilot rows are the package's `derisk/beliefs/B-*.toml`, which arrive with `derisk/`.
5. The 18 declared risk rows take no value from their seed form: `intake write` gives them their symbol, unit and bounds, and `derisk rollup` their value at the first release.

Four pilot rows carry seven test vectors, transcribed from IDMAS v2 and other cited pages. Five rows say `fixture_wanted` (two in the pilot thread, three in the risk branch's conclusion); an engineer supplies those vectors later, in a node form, with the page.

When P1 is green, engineers start confirming the seeded rows, each with a node form of type "confirm". The next phase does not wait for all of them.

## If it goes wrong

| Situation | Do |
|---|---|
| F03 on a seed form: the node is no longer `seeded` | it has been through intake already; skip it |
| C07: an input's producer has no answer yet | take the producer's seed form first; the order is by dependency |
| a seed row's value disagrees with a source you hold | take the seed through as it is; then send a change with a node form, citing your source, like any team member |
| the risk rows answer `NotRun` after P1 | nothing has rolled the ledger up yet; `cargo xtask derisk rollup` at the first release gives them values |
