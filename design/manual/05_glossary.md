# Glossary
<!-- role: everyone; id: glossary -->

**In one line:** the words the apps use, each in one sentence.

| Word | What it means |
|---|---|
| Node | One row of the design tree: a value, a relation, a requirement, a piece of evidence, a closure or an interface, in its own file |
| Node file | `nodes/<id>.node.tndb`: everything its author writes and signs |
| Group | One of the 20 parts of the design, led by one lead; its file is `structure/<group>.group.tndb` |
| Stage | A part of a group with its own owner, who signs it |
| Lead | The person who shapes the group, issues its nodes and seals its releases |
| Stage owner | The person who signs a stage |
| Author | The person a node is issued to; only they change it, until it is sealed |
| Checker | A second engineer, never the author, who signs a node as checked |
| Issue | Give a node to an author |
| Ready | The author's signature: nothing left to fix |
| Checked | The checker's signature over the node as it is |
| Stale | A signature that no longer covers the node, because it changed after |
| Test vector | Inputs, the expected answer, a tolerance, and where the answer comes from |
| Outside answer | An expected value from a book, an independent derivation, another tool or a physical bound, never from the code under test |
| Evidence debt | What a node claims without evidence yet |
| Contract | An output another group reads, with its unit and version |
| Change request | A request to another group to change something it owns |
| Impact check | What an action would change, shown before it is done; a stop blocks it |
| Assemble | The whole group read and checked across its nodes |
| Seal | The lead freezes the group as a numbered release |
| Release | `releases/<group>-<version>.tnrel`: the group as sealed, never edited |
| Confirmed | Sealed with every check passing and signed by a checker and its stage owner |
| UNCONFIRMED | Sealed with at least one reason not to trust it yet, named in the release |
| Re-issue | Open a sealed node again, as it is or as a release had it |
| Node form | Today's request file (`adcs-node-form/1`), which the group app can import |
| Fingerprint | A SHA-256 hash: if one byte changes, the fingerprint changes |
