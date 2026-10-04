# Guide for group leads
<!-- role: lead; id: lead; app: group -->

**In one line:** you shape your group in TRI-NETRA Group (its nodes, stages, people and contracts), issue nodes to authors, follow their progress, and seal the group's release when it is ready; stage owners sign their stages before you seal.

## Say it simply

You are the editor of one chapter. The map shows every section and what reads what. You hand sections to writers, see who is done, and when the chapter is ready you print an edition of it. An edition never changes; the next one gets a new number.

**Where the story lies:** a printed edition cannot be corrected. A sealed release cannot either, but any node in it can be re-issued for the next release, and a node file that is lost can be made again from any release.

## Your first week

1. **Open the app.** Open `Apps/TRI-NETRA Group.html` in Chrome or Edge from the Drive folder on your computer, type your name, choose **Open design folder** and pick `Design`.
2. **Open your group.** Click it in the list. The banner says whether every structure rule holds.
3. **Add your people.** **People → Add or change a member**: yourself as **lead**, your stage owners, your authors. Names must be typed the same way everyone types them in the apps.
4. **Set the stage owners.** **Stages**, click a stage, choose its owner.
5. **Issue nodes.** On the **Map**, click a node, **Issue…**, choose the author. Their node app lists it first.
6. **Follow progress.** **Progress** shows each node's work (shell, draft, ready, checked), its signatures, its problems and its evidence debt.
7. **Comment.** Click a node (Map, Progress or Assemble) and **Comment…**: the author sees it on Home.

## Now the real thing

**Every change shows its impact first.** Each line is a note, a change, or a stop. **Do it** appears only when nothing stops it, and the action changes every file it touches or none.

**Another group's file changes only with its agreement.** Moving a node into another group, or archiving or merging a node another group reads, needs a change request accepted by that group's lead. The impact check offers to raise it.

**Releasing.**

1. **Assemble** lists the checks across your nodes and, for every node, why it is not yet confirmed.
2. Each **stage owner** signs their stage (**Assemble → Sign…**). A change to any of its nodes takes the signature off.
3. You **seal** (**Release → Seal 1.0…**). Only the lead can. The impact check says how many nodes go in confirmed and which do not.

A node is sealed as confirmed only when a second engineer checked it as it is, the checks find nothing, its stage is signed and, if it computes, a test vector has its answer from outside the code. The rest go in UNCONFIRMED, each with why. Sealing writes `releases/<group>-<version>.tnrel`, frozen, and seals every node file.

**Accepting the delivery.** After you seal, the developer side tests your release and delivers it: your group's code generated from your nodes, its tests run, a test app, and a note, all in `deliveries/`. Open the test app from disk and see every test vector pass in it. Then **Release → Deliveries → Accept…**. Your acceptance names this release and this delivery; a later release needs its own. A group whose lead has not accepted ships visibly UNCONFIRMED, named in the release notes. Groups go through in five waves, the groups yours reads first.

**After a release.** **Re-issue** opens a sealed node again (as it is, or as any release had it). **Compare** shows what changed between two releases, or since the last one. **Import node forms** takes today's node forms into your node files, keeping what authors typed.

## For a stage owner

You sign your stage on **Assemble** when its nodes are as they should be. Your signature covers them exactly as they are; any change takes it off, and you sign again. Only you can sign your stage.

## When something goes wrong

| What you see | What it means | What to do |
|---|---|---|
| An action stops: someone has a file open | An author is editing a node the action touches | Ask them to close it, then do it again |
| A structure action did not finish | The computer stopped in the middle | **Finish it** completes it from its record |
| Seal stops: nothing changed | No node changed since the last release | Nothing to do |
| Seal stops: a node file cannot be opened | A file is missing or damaged | **Re-issue** it from a release |
| Fingerprints: problems | A release file was changed after sealing | Tell the developer team; do not edit release files |
