# Guide for developers
<!-- role: developer; id: developer -->

**In one line:** the developer team turns each group's sealed release into tested software: it checks the release, generates every computing row's code from its pseudocode, tests it against the release's own test vectors, and hands the group a test app to accept.

## Say it simply

The groups write the recipes; you run the kitchen. You never change a recipe yourself: when one is wrong you send it back to its group. What you own is the kitchen, the tools and the checks that prove each dish matches its recipe.

**Where the story lies:** a kitchen can taste and adjust. You cannot adjust a node's numbers to make a test pass; an expected value never comes from the code under test.

## Now the real thing

**What arrives.** A sealed release file, `releases/<group>-<version>.tnrel`. `python3 tools/release.py check` checks its fingerprints, that its nodes are the group's, that each confirmed node was checked by someone other than its author and, when it computes, has an outside test vector, and that the lead sealed it.

**What you do with it.** The release is merged into `design.tndb`, every computing row's code is generated from its pseudocode (Rust for the engine, MATLAB for the twin) and tested against the release's test vectors, the C and Rust flight software are held to the pseudocode interpreter, and the group gets a test app to accept.

**Where things are.** The design files and their schema: `design/schema.toml`, `tools/tndb.py`. The structure rules: `tools/group.py`. Releases: `tools/release.py`. The pseudocode: `docs/PSEUDOCODE_V2.md`, `tools/pcode.py`. Every command: `docs/COMMANDS.md`. Every check in one command: `python3 tools/check_all.py`.

**The rules nothing bends.** Node content comes only from its group's release. An expected value never comes from the code under test. A computing node without an outside answer is never confirmed. Every check that can fail fails by name.

The developer manual in full is in `spec/manual/developer/`.
