# Guide for authors
<!-- role: author; id: author; app: node -->

**In one line:** you fill the node your lead issued to you, step by step, in TRI-NETRA Node; when Review shows nothing left to fix you mark it ready, and a second engineer signs it as checked.

## Say it simply

The node app is a form that checks itself as you type. Each field asks one question, says why it is asked and shows an example. What is still missing is listed on Review, each item with a button that takes you to the field.

**Where the story lies:** a form usually only wants its boxes filled. These checks also run your pseudocode on your test vectors and compare units, so a filled form can still have problems: Review is the judge, not the number of filled boxes.

## Your first node

1. **Open the app.** Open `Apps/TRI-NETRA Node.html` in Chrome or Edge, from the Drive folder on your computer (Drive for desktop). It runs offline; nothing leaves your computer.
2. **Say who you are.** Type your name the first time. It goes beside every change and signature.
3. **Open the design folder.** Choose **Open design folder** and pick the `Design` folder. The nodes issued to you are listed first.
4. **Open your node.** Click it. **Home** shows where it sits (its group, what it reads, who reads it), how far it is filled, and your lead's comments.
5. **Start from the spec.** When the spec already says something about this row, Home offers **Start from the spec**. Take it: those become your fields, and you can change any of them.
6. **Go through the steps.** Each tab is one step. Fill the fields; leaving a field saves it into the file's history (Undo takes it back).
7. **Look at Review.** Every problem is listed with its rule (for example `X01`) and a **Go there** button. Fix them until the list is empty.
8. **Mark ready.** On Review, **Mark ready**. Then **Save**.
9. **Ask a colleague to check it.** They open the same node, read it on **Preview**, and choose **Sign as checked**. The app refuses your own name there.

## Now the real thing

**The kind decides the steps.** A declared node states a value with its source; a computed node is a relation from other nodes, with pseudocode and test vectors; a KPI states a requirement and which way it binds; an evidence node states what a test shows. Closures, interfaces and target rows are fixed by the tree: you write only their explanation and feedback. A row the spec has not named yet asks you to choose declared or computed first.

**Test vectors are the heart of a computed node.** Each one gives inputs, the expected answer, a tolerance and where the answer comes from: a page of a book, an independent derivation, another tool, a physical bound. An answer the code made proves nothing, and a computed node without an outside answer is never sealed as confirmed. **Try it** runs your pseudocode on every vector.

**Signatures cover what is written.** Marking ready and checking each record a fingerprint of the node's content. Any later edit makes them stale, and the app says so: mark it ready again and ask for the check again.

**Sealed means frozen.** When your lead seals the group, the node opens read-only and says which release sealed it. Your lead re-issues it when it is to change.

**Contracts.** When a node you read changes its contract, Home shows **Acknowledge**. When you need a contract changed, ask on Review (**Ask for a contract change**); the owning group's lead decides.

## When something goes wrong

| What you see | What it means | What to do |
|---|---|---|
| Read-only: open for editing by someone | Someone else has the node open | Wait, or ask them to close it; the banner names them |
| Read-only: sealed in a release | Your lead sealed the group | Ask your lead to re-issue it |
| Unsaved work from before | The browser or computer stopped before a save | **Restore them**, then Save |
| A conflict copy beside the file | Drive saw two versions at once | Open both, keep the right one, tell your lead |
| Signatures stale | You edited after ready or check | Mark ready again; ask for the check again |
