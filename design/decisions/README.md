# Design decision records (S11.1, S11.2, S12.1)

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.

One file per decision, in the format of `docs/DESIGN_DECISIONS.md` ("Decision record (DR) format"). They are the source
text of the design's decision nodes: the developer's revision S11.2 (`design/revisions_2_0.toml`) carries each file into
the group that owns it as a node of kind `decision`, so the regression copy holds them and the library shows them.

- `status = "proposed"`: the developer's proposal, **not signed by a person**. Nobody signs in S11; each owner signs or
  overrides in the ownership stage (S20).
- `current` is what the design and the code do today; `choice` is the proposal the S11 fixes are built on. An empty
  `choice` means the developer makes none: the decision is the owner's and `current` flies until it is made.
- `owner_question` names the question of `docs/S11_INVENTORY.md` §5 whose default this record carries.
- `sets` names where the choice lands: one stated node or one switch, so the owner can change it in one place.
- `signature` stays empty until the owner signs.

Thirty-six records (S11: twenty; S12.1: sixteen new, two amended): the 15 decisions of `docs/DESIGN_DECISIONS.md` that most need one, and five of the inventory's owner
questions that none of the 15 covers (A5, F5, O12, TS11, M5).

**S12.1.** The 24 owner decisions of `docs/S12_INVENTORY.md` section 5 are carried the same way, each on its stated
default, as the developer's proposal, unsigned. A record carries `owner_question = N` (or a list when one decision
answers several) and `owner_question_stage = "S12"`, which tells it from S11's numbering (a record without the stage
field is S11's). DR-design-003 already carries S11's question 8, so its S12 questions (21, 22) are in
`owner_question_s12`. Every question 1-24 is carried by exactly one record (`tests/test_decisions.py`). The amended
records are DR-gdn-002 (19) and DR-design-003 (21, 22), their history kept and "S12.1 (the developer's, unsigned)" added.
Where a choice lands in a node that does not exist yet, `sets` says "to be stated in S12.x as <name>". Values no one has
stated are not invented: question 2's activity table is transcribed in S12.9 from ECSS-E-ST-10-04C with its table
number and checked by the owner; questions 6 and 11 carry the roadmap's cited magnitudes, the developer's.
