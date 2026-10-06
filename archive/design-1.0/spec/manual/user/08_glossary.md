# Glossary
<!-- kind: reference; depth: expert -->

**In one line:** the words this manual and the software use, each in a line; the page that explains a word further is named where there is one.

| Word | Means |
|---|---|
| ADCS | attitude determination and control system |
| APE, AKE, RPE, PDE, RKE | absolute pointing, absolute knowledge, relative pointing, pointing drift and relative knowledge errors (ECSS-E-ST-60-10C) |
| case | your whole input: one CSV in the fixed format `adcs-case/1` |
| case editor | the offline page that writes a case CSV and says what it can run |
| unstated | a case value left blank under the "stated" policy; whatever needs it is blocked, by name |
| assumed | a case value left blank under the "default" policy; the reference value is used and listed |
| level | the per cent of cases a requirement must hold for, such as 99.73 for 3σ |
| UNCONFIRMED | a value or relation nobody has put their name to yet; it runs, and says so |
| node | one question with one answer in the software's tree |
| node library | every node's document and form, offline |
| node form | the file you use to ask for a change, a new node, a confirmation or to give feedback |
| request | a filled node form you send to the developer team |
| Checked by | the engineer who stands behind a node's relation and values; their name goes on the node |
| family | a configuration of hardware: `mtq` (coils only), `mtq_rw` (coils and wheels), `mtq_fmr` (coils and fluid rings), `mtq_fmr_rcs` (coils, rings and thrusters) |
| product | a catalogue configuration of parts plus the flight algorithms it carries; candidate, offered or retired |
| candidate | a product found by the designer, not yet offered to clients |
| scenario | one test: the orbit and epoch from your case, the initial conditions, the environment, the modes, and the metrics it judges |
| campaign | a set of runs of a scenario: nominal, Monte Carlo, edge, sweep or fault |
| result document | one HTML file holding a finished run or campaign; opens anywhere and runs nothing |
| store | the folder where your cases and results are kept, each result beside the case it ran |
| SILS, PIL, OILS, HILS | software-, processor-, OBC- and hardware-in-the-loop simulation: the four test rungs |
| rig needs | what an OILS or HILS rig must do for your case (field range, bearing torque allowed, truth accuracy, ...), computed from the case |
| NotMeasured | a lab capability nobody has measured yet; whatever needs it waits, by name |
| rig fit | the comparison of your case's rig needs with what the lab has measured; a campaign that does not fit is not run |
| `TWIN.md` | the page in the MATLAB zip that lists every part of SILS and whether this version carries it |
| synthetic part | a made-up part (`SYN-*`) for learning and pilots; never evidence |
| credibility | eight 0–4 scores saying how far to trust a number; the lowest one limits it |
| developer team | the people who build, maintain and release the software, and answer your requests |
| belief | a bet a node, an input, an output, a model, a relation, an algorithm or a page rests on, written down with the test that would settle it ([Why things change](07_why_things_change.md)) |
| broke, held, untested | a belief's status: a test showed it wrong, a test confirmed it, or nobody has tested it yet |
| risk, R-nn | something that could go wrong because a belief may be wrong, with the test that would close it |
| risk level, L1–L5 | how much it would cost if it happened: from one run repeated (L1) to delivery stopped (L5); 0 means closed |
| area | one of seven: nodes, case inputs, outputs and results, models, maths, algorithms, visualisation |
| version | one state of a node, with the request and belief that made it, what was wrong before, and what it gives |
| Risk management | the branch of layer 1 that counts the beliefs and risks at every release, and concludes |
| de-risking narrative | the quarterly page: the conclusion, the beliefs of the quarter in the company template's columns, and the open L4 and L5 risks |
| Learn, Read, Expert | the three depths of every page: every exercise; the answer and the explanation; the relation, limits and sources |
| tutorial, how-to, reference, explanation | the four kinds of section: learn by doing it once; do one task; look something up; understand why |
