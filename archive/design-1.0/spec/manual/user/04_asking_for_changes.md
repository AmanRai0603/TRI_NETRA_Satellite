# Asking for changes: the node form
<!-- kind: how-to; depth: read -->

**In one line:** open the node's form, choose what you are asking for, fill the sections that light up, including the belief your request rests on, fix what the review list flags, save a filled copy and send that file to the developer team; they answer in the same file.

You never change the software yourself, and nobody expects you to write code. The node form is also the node's document, so you read what the node says in the same file you write your request in.

## Get the form

1. Open the **node library** (`forms/index.html` in the MATLAB zip, or **Nodes** in the web app) and find the node by name or id; or press **Ask for a change** on a node's page in the web app.
2. For a node the tree does not have yet, open **Ask for a new node** (`new_node.form.html`).
3. Use a form from the release you are using. If the node has changed since your form was made, the checker says so (F04) and you download a fresh one.

## Ask for a change or a new node

1. **Read the top of the form.** The first sentence says what the node is, its version, whether anyone has confirmed it, and the highest open risk it carries. "Where this node sits" shows what it reads and where its answer goes.
2. **Read the node's document**, "The node, as the software has it now": say it simply, the real thing, where it breaks, and a test vector you can rebuild.
3. **Choose what you are asking for:** change, new node, confirm, feedback, or something else. Only the sections you need light up.
4. **Fill those sections.** Every field says what it asks, why it matters, and gives an example. A field that differs from the node as it is now is highlighted, with its current value under it and a "Reset" button.
   - *The question it answers*: one plain sentence. If you cannot say it in one, it is two nodes.
   - *Its answer*: symbol, quantity, a unit that states it, and the lowest and highest values, **each with a reason**.
   - *What it reads* (computed nodes): each input's binding, the node it reads, and the quantity it must be. Only nodes in the same layer.
   - *The relation*: one line in the bindings' names, its source, and why it is this relation.
   - *How it is computed*: each step and the physics function it calls, or a description of the new one it needs.
   - *What has to be true*: each assumption, and when it stops being true.
   - *Explain it*: the node in plain words and in one line, and, if you can, a common wrong idea, an analogy and where it breaks, and a why-chain down to a law. This is what the next new reader reads first.
   - *Test vectors*: copied from a page of a cited source, never computed with the software, never invented by an assistant.
5. **Fill "De-risking: the belief behind this request".** Every change exists because a belief broke; every new node rests on a belief nobody has tested yet. Say what was believed, whether a test settled it and what that test was, what we now know, what was wrong with the version it replaces and what the new one gives, what changed in the plan, and any risk it opens, raises, lowers or closes. A risk goes down only when you say what was tested. Why this matters is on [Why things change](07_why_things_change.md).
6. **Fill "Who is asking, and why":** your name, team, contact and reason. Under **Checked by**, the engineer who has checked the maths against its source and stands behind it; it can be you. If nobody has, leave it blank: the node then runs as UNCONFIRMED, visibly.
7. In **Learn**, write the one-line summary under "Explain it back". It travels with the request and is what the developer team reads first.
8. **Review and send.** Fix every item marked **!**. Items marked **i** are allowed and recorded. Press **Save filled copy**, which downloads `<node>.request.<date>.html`, and send that file: in the portal (Requests → Send), as an issue with the "Request" template, or by mail or chat to the developer team.

## Confirm a node

1. Choose **Confirm this node**.
2. Give your name, your reason, and your name under **Checked by**. Nothing else changes.
3. If you tested something to confirm it, fill the De-risking section as a belief that **held**, with what you tested: that is how a risk goes down.

## Send feedback on a release

1. Open the file you got back from the developer team (or any form of the node), and choose **Feedback on a release**.
2. Add an entry: the release, your case CSV, the result file, what happened and what you expected. Attach the result file.
3. Save and send it. Feedback may come on an older copy of the form.

## Use an assistant

1. Under "Filling this with an assistant", press **Copy an assistant prompt**, and give the assistant the file and what you want.
2. Open the file it returns and read the review list before you send it.
3. Your name, and the name under "Checked by", must be people's names, never the assistant's. An assistant never supplies a test-vector number.

## What happens after you send it

1. **The check.** A developer runs the checker on your file. It reads only your answers, never runs the page, and applies every rule the page applies and more. If it fails, you get the file back with every finding in its status card.
2. **The implementation.** The developer team implements it exactly as written. Your belief record goes into the risk ledger, and the node's version history gains a version that says what was wrong before and what this one gives.
3. **Verify and review.** The checker compares the finished node with your form, field by field; a second developer reviews it; every test downstream runs.
4. **The reply.** The file comes back with its status: received, check failed, check passed, in progress, in review, released in a version, needs information, feedback noted, or rejected, with the reason.
5. **After the release**, run your own case in the new version and compare the result with your earlier one ([Results](03_results.md)). If it is not right, send feedback in the same file.

## If it goes wrong

| Code | Means | Do |
|---|---|---|
| P01 | no name, or an assistant's name, as the requester | put your own name |
| P02 | no reason | say why |
| F04 | the node changed since your form was made | download a fresh form and carry your request over |
| F05 | the change changes nothing, or a confirmation changes something | pick the right request type |
| O03, O04 | the unit does not state the quantity; a limit has no reason or the limits are out of order | pick a unit from the list; add the reason |
| C02–C05 | an input does not exist, publishes another quantity, crosses a layer, or makes a loop | pick the node from the list; ask the developer team if you need a new connection |
| R01–R03 | the relation, its source, its reason or its steps are missing | fill them |
| S01, T01–T03 | an unknown source; a test vector without provenance or page | add the source; copy the vector from its page |
| D01–D06 | the belief record is missing or incomplete; a risk move is not in the register, or lowers a level without a test | fill "De-risking"; say what was tested |
| X01, X02 | an analogy without where it breaks, or a wrong idea without why | add the missing half |

The developer manual lists every code.
