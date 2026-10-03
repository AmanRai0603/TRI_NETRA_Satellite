# Your own changes
<!-- kind: how-to; depth: read -->

**In one line:** the engine, views, tools, catalogue, scenarios, campaigns, the case format and the MATLAB twin are your own work, on an ordinary branch and review, but each one records the belief it rests on with `cargo xtask derisk record`, and node content is never your own work.

## Make an own change

1. Branch and make the change, following CONTRIBUTING (SPEC.md §17.3) and its reviewer counts.
2. **Record the belief behind it:**

<!-- since P1 -->
```
cargo xtask derisk record --area model --about crates/adcs-sim-core \
    --believed "…" --status broke --tested "…" --now-know "…" --plan-change "…" \
    [--move R-06:3:2] [--risk R-10] [--open "title|area|level|closing test|owner team"]
```

   It writes `derisk/beliefs/B-nnn.toml` and the moves into `derisk/risks.toml`, and writes nothing if `derisk check` would then fail. In this package, `python3 tools/derisk.py record` takes the same arguments.
3. If nothing believed changed (a refactor, a typo, a dependency bump), label the pull request `derisk:none` and say why in one line. The `derisk` CI job refuses a pull request that touches `crates/adcs-sim-core`, `crates/adcs-sim`, `crates/adcs-tune`, `crates/adcs-solve`, `crates/adcs-result`, `web/`, `forms/`, `results/`, `catalogue/algorithms/` or `scenarios/` with neither.
4. A change to a SILS element (a physics function, the plant, the environment, a device emulator, an algorithm, a metric, the recorder, the runner, the campaign runner, the tuner) changes its MATLAB twin in the same branch, or carries the label `twin:none` with a reason; the `twin` CI job checks it ([The MATLAB twin](09_twin.md)).
5. A change to a template or a manual page keeps it to the explanation standard ([Explaining](07_explaining.md)); `cargo xtask explain check` runs in CI.
6. A change that adds, removes or renames a command, a route, a view or a case key updates the manual page that describes it, in the same branch. `cargo xtask manual --check` regenerates the generated pages and fails on any difference.

## Turn a "something else" request into your own work

| Request | What you do | Check |
|---|---|---|
| a new part or a part's values | edit `catalogue/parts/<id>.toml`; two reviewers, one from quality (H14) | `tools/validate_plan.py` |
| a catalogue candidate from the designer | add the attached candidate file to `catalogue/products/` as `candidate`, with its `[design]` table | `tools/validate_plan.py` |
| offering a product | `cargo xtask decision record <record>` copies the H14 decision into `[promotion]`; publish the next catalogue bundle | `validate_plan.py` refuses an offered product with a synthetic part, no design campaign, or unconfirmed algorithm bounds |
| a new algorithm | prototype it in the MATLAB twin (`prototype = true`; the twin map marks it twin-only), then implement it in `fsw/` and set `prototype = false` in the same pull request; from then on the two move together | `twin`, `matlab-parity`, `fsw` |
| a lab capability measured | copy the value from the person's bring-up record into `rig/labs/<lab>.toml`, replacing `nan`; two reviewers, one from verification | `tools/validate_plan.py`, `adcs rig fit` |
| an algorithm's bounds | edit `catalogue/algorithms/<id>.toml`; two reviewers, one from GNC | the tuner's box |
| a scenario or a campaign | write it in `scenarios/` or `campaigns/`; its orbit and epoch are always `case:` references | `tools/validate_plan.py` |
| a new key in the case format | edit `CASE_INPUTS` in `tools/build_tree.py`, run it, bump `adcs-case/<n>`, write the migration; two reviewers | `build_tree.py --check`, the `cases` job |
| clearer help text for a case key | edit `CASE_HELP` in `tools/build_tree.py`; one reviewer | `build_tree.py --check`, `xtask manual --check` |
| a new group in the tree | `tools/seed_tree.py --add-group <id> --under <parent>`, plus the matching change to `tools/build_tree.py`; the group's nodes then arrive as new-node requests | `validate_plan.py`, the seeding assertion of P1 |

Each of these records its belief like any own change, and you reply to the requester on the same file when it is released.

## If it goes wrong

| Situation | Do |
|---|---|
| the `derisk` job refuses the pull request | add the belief record, or the `derisk:none` label with its reason |
| you want to lower a risk and have no test result | you cannot; record the belief as untested with the test that would settle it, and run that test |
| the change touches a node's content | stop: that is a request; send a node form and let another developer take it through intake |
| `explain check` fails on a template you changed | read the rule it names in [Explaining](07_explaining.md); add the missing part |
