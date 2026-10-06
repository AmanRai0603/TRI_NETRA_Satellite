# De-risking: the ledger, versions and the quarterly narrative
<!-- kind: how-to; depth: read -->

**In one line:** keep the risk register and the belief records true (intake writes most of them; you record your own changes' beliefs), check the ledger in every pull request, roll it up into the Risk management rows at every release, and write the quarterly narrative from it.

Why the ledger exists, and what a belief, a risk and a version are, is on the user manual's [Why things change](../user/07_why_things_change.md) and in SPEC.md §5.13.

## Know where it lives

| File | Holds | Written by |
|---|---|---|
| `derisk/risks.toml` | the register: every risk, its level 0–5, its closing test, every move | `intake write` (moves in a request), `cargo xtask derisk record` (your own changes) |
| `derisk/beliefs/<id>.toml` | one belief record each, `adcs-belief/1` | the same two |
| `crates/adcs-mod-*/nodes/<node>/versions.toml`, `versions/<n>/` | every version of a node, why it was made, and its sheet as it was | `intake write`; `intake mark published` stamps the release |
| `derisk/narratives/<Q>.md` | a quarter's paragraph of prose | a person on the quality team (D27) |
| `derisk/rollup.toml` | the values of the 18 declared Risk management rows in a release | the release's preparation commit |

## Record a belief for your own change

1. Run `cargo xtask derisk record` (`python3 tools/derisk.py record` in this package) with the area, what it is about, what was believed, the status, what was tested (or would test it), what we now know, what changed in the plan, and any moves ([Your own changes](04_own_changes.md)).
2. `cargo xtask derisk check` must pass. A level goes down only with `--status held` or `--status broke` and `--tested`.
3. Commit the belief file and the register together with the change they belong to.

## Check the ledger

1. Run the check:

<!-- since P1 -->
```
cargo xtask derisk check          # python3 tools/derisk.py check in this package
```

2. Read each finding by its code (below) and fix the file it names. Never lower a level to make a check pass.

## Roll it up at a release

1. The release's preparation commit runs:

<!-- since P1 -->
```
cargo xtask derisk rollup --quarter Q3-26     # python3 tools/derisk.py rollup in this package
```

2. It writes `derisk/rollup.toml`: the 18 declared risk rows' values. The three conclusion rows (highest open level, net closed this quarter, share of beliefs tested) are computed by the engine from them, like any computed node.

## Write the quarterly narrative

1. At the end of a quarter, generate the page and the tables:

<!-- since P1 -->
```
cargo xtask derisk narrative --quarter Q3-26 --out dist/narrative     # tools/derisk.py narrative in this package
```

   It writes `narrative_Q3-26.html` (to the explanation standard: the conclusion first, the areas on one scale, one row per belief in the company template's columns, the open L4 and L5 risks, where the narrative breaks), `narrative_Q3-26.xlsx` in the template's seven columns, and `narrative_Q3-26.csv` in the same seven plus belief, status and area.
2. The quality team writes the paragraph of prose in `derisk/narratives/Q3-26.md`, and the page is generated again. Until it is written the page says so; nobody writes it for them.
3. The release carries the narrative. Bad news appears there first and between meetings, never sprung at the table.

## If it goes wrong

| Finding | Means | Do |
|---|---|---|
| L03 | a risk's history does not add up to its level, or its opened and closed quarters | fix the history; never edit the level alone |
| L04 | a move is in the register but not in the belief that made it, or the other way round | write it in both, with the same quarter, from, to and belief id |
| L05 | a level went down without a tested belief | put the level back, or record the test that lowers it |
| L07 | a belief says neither what tested it nor what would | write the test |
| two branches moved the same risk | the second's check fails after the first merges | rebase and re-apply the move from the new level |

## Every rule

The table is generated from `tools/derisk.py`, and CI fails if the two differ.

<!-- ledger:begin -->
| Code | The rule |
|---|---|
| L01 | the register is adcs-risk-register/1; every risk id is R-nn and unique; every belief is adcs-belief/1 with a unique id, B-nnn or a request id, named as its file |
| L02 | a risk's area is one of the seven; its owner is a team, never a person; its level is a whole number 0 to 5 |
| L03 | a risk's history starts from 0, each move starts where the last ended, the last ends at its level, and opened and closed are the quarters of the first move and of the move to 0 |
| L04 | every move in the register names the belief that made it (only a risk's first entry, made at planning, may name none), is written in that belief, and every move a belief makes is written in the register |
| L05 | a level goes down only by a tested belief: its status is held or broke and it says what was tested |
| L06 | a belief's area is one of the seven; its status is broke, held or untested; it says what was believed, what we now know and what changed in the plan |
| L07 | a held or broke belief says what tested it; an untested belief says the test that would settle it |
| L08 | quarters are Qn-YY; a cost is blank or a number, zero or more, in thousands of dollars |
| L09 | every risk a belief names exists; every open risk says the test that would close it |
<!-- ledger:end -->
