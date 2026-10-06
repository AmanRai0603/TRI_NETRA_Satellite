# Reading a node
<!-- kind: explanation; depth: read -->

**In one line:** a node is one question with one answer, worked out from other nodes by a cited relation, and its page tells you, in order, what it means in plain words, the real relation, where that relation stops being true, and a printed number you can rebuild yourself.

## Say it simply

The software's knowledge is a tree of small, separate answers. "How long does a ring take to spin down?" is one. "What is the absolute pointing error?" is another. Each answer is worked out from a few others, so the tree is a chain of reasoning you can follow link by link, and every link says where it came from and who checked it.

Think of it like a spreadsheet in which every cell has its own page: what it means, the formula, where the formula comes from, and the cells it reads.

**Where the story lies:** a spreadsheet cell can hold any formula anyone typed. A node's relation may only call the physics library, cites a source, and cannot change unless a request goes through the developer team's checks.

## Now the real thing

**Four layers, one door between each.**

| Layer | Holds |
|---|---|
| 1 — the company | case intake, the catalogue, commercial, the order, the test facility, standards, supply, heritage, and risk management |
| 2 — the satellite's ADCS | the pointing service asked for, the hardware fitted, mission and orbit, the satellite as the ADCS sees it, the subsystems at system level, verification, and what the OILS and HILS rigs must do for this case |
| 3 — a subsystem | fourteen subsystem layers, the OILS and HILS rigs among them, plus the KPI closures (restricted to the developer team) |
| 4 — the run | what one run or campaign produced |

Layers meet only at the door (layer 1 to 2) and at each subsystem's interface node (2 to 3). A node reads only nodes in its own layer.

**Kinds.**

| Kind | Means |
|---|---|
| computed | a relation from other nodes |
| declared | a stated number; many come from your case, a product, tuning, or the risk ledger |
| required | a requirement; its value comes from each case |
| achieved | evidence: only a campaign supplies it |
| door, interface, closure | the tree's shape; fixed, not offered for change |

**States.**

| The library says | Means |
|---|---|
| not written yet | the node exists but nobody has specified it; it answers `NotRun` |
| unconfirmed | it is specified, checked against its request and tested, and it runs, but nobody has put their name to its relation or values |
| confirmed | an engineer has checked it and stands behind it; their name is on it |

**Trust.** Every answer carries eight credibility scores from 0 to 4: Mathematics, Assumptions, Verification, Validation, InputPedigree, Uncertainty, Understanding, Reproducibility. The lowest one limits the answer, and the page names it. Mathematics 1 means nobody's name is on the relation. InputPedigree 0 means it was flown on a synthetic part. InputPedigree 1 means it reads a value marked UNCONFIRMED.

**The page, in order.** The first sentence answers "what is this, and how far can I trust it?". "Where this node sits" shows what it reads and where its answer goes. Then its document, as four steps: *Say it simply*, *Now the real thing* (the relation, its source, a why-chain down to a law, the inputs, the steps, the answer and its bounds, the test vectors), *Where the simple version breaks* (its assumptions, whether it is confirmed, its open risks), and *Try it*. Below that, "Versions, and what they rest on" shows every version and the beliefs behind it ([Why things change](07_why_things_change.md)).

## Where the simple version breaks

- **A confirmed node can still be wrong.** Confirmation means a named engineer checked the relation against its source; it is not proof. The test vectors and the campaigns are what test it.
- **A number is only as good as what it reads.** A confirmed node that reads an unconfirmed one inherits the low score, and the page says so.
- **The relation holds only inside its assumptions.** The ring spin-down time assumes laminar flow; above a Reynolds number of about 2000 the real ring stops sooner. The page lists each assumption with when it stops being true.
- **Layer 3 is not in the library.** Subsystem detail is restricted to the developer team; what reaches you is its interface node in layer 2.

## Common wrong idea

"A low credibility score means the number is wrong." It does not. It means nobody has yet done the thing that would raise it: put a name to the relation, measure the part, run the higher rung. The page names which score limits the answer, and so whose work would raise it. A high score with a wrong number is the dangerous case, and the ledger of beliefs exists to find those ([Why things change](07_why_things_change.md)).

## Try it

1. Open the node library and find **Ring spin-down time**.
2. Switch the page to **Learn**.
3. Before reading its relation, predict: if the ring's bore doubles, does it coast twice as long, four times as long, or half as long?
4. Read "Two cases, one difference" on the page and check.
5. Scroll to **Try it** and rebuild the printed 0.75 s from the inputs given.
