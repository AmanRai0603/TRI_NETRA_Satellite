# The journey of a node
<!-- role: everyone; id: journey -->

**In one line:** an author writes a node in the node app, a second engineer checks it, the stage owner signs the stage, the group lead seals the group's release, and the developer team turns sealed releases into tested software that the lead accepts before it reaches everyone.

![The journey](journey)

## Say it simply

Think of a chapter of a reference book. One person writes a section, a colleague reads it against the sources, the chapter editor signs the chapter, and the editor in chief prints the edition. Nothing reaches the printed book without passing every desk, and every desk leaves its name.

**Where the story lies:** a book's section only has to read well. A node also computes: its code is generated from what the author wrote and tested against the answers the author gave from outside the code. So "checked" here means the numbers hold, not only the words.

## Now the real thing

| Step | Who | Where | What leaves it |
|---|---|---|---|
| 1 Write | the node's author | node app | a node marked ready, with no problem left on Review |
| 2 Check | a second engineer (never the author) | node app, Review | a "checked by" signature over the node's content |
| 3 Sign the stage | the stage's owner | group app, Assemble | a stage signature over all its nodes as they are |
| 4 Seal | the group lead | group app, Release | a frozen release, `releases/<group>-<version>.tnrel`; every node file sealed |
| 5 Build | the developer team | the repository | the group's code generated from its pseudocode, tested against its test vectors, in a test app |
| 6 Accept | the group lead | the test app | the group accepted, or sent back with what is wrong |
| 7 Release | the owner | `main` | a release of the software everyone uses, naming each group's release |

Every step can be undone by the one before it: an edit takes a signature off, a re-issue opens a sealed node again, and a group that is not accepted goes back with its reasons.

**Confirmed or UNCONFIRMED.** A node is sealed as confirmed only when steps 1 to 3 hold for it as it is now, the checks find nothing, and, if it computes, a test vector has its answer from outside the code. Every other node is sealed UNCONFIRMED with its reasons, and the software shows it as such. Nothing is hidden: a gap is visible, not missing.

## Where to go next

- If you write nodes: **Guide for authors**.
- If you lead a group or own a stage: **Guide for group leads**.
- If you build the software: **Guide for developers**.
- If you use the software's results: **Guide for users**.
