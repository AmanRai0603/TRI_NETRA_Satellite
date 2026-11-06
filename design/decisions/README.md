# Design decision records (S11.1, S11.2)

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

Twenty records: the 15 decisions of `docs/DESIGN_DECISIONS.md` that most need one, and five of the inventory's owner
questions that none of the 15 covers (A5, F5, O12, TS11, M5).
