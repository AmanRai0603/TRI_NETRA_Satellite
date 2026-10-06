# Why things change
<!-- kind: explanation; depth: read -->

**In one line:** a node gets a new version only because a belief it rested on broke, or to lower a risk; every version says what was wrong with the last one and what it gives; and the Risk management branch adds all the beliefs and risks up into one conclusion per release.

## Say it simply

Every choice in the software is a bet: that a simple field model is close enough, that five pointing errors add up a certain way, that one CSV holds a whole case. Most bets are right. The expensive ones are the wrong bets nobody wrote down, because then nobody knows which results rest on them.

So each bet is written down as a **belief**, with the test that would settle it. While it is untested, the risk it carries is counted. When a test breaks it, the node that rested on it gets a new **version**, which says what was wrong and what the new version gives. A test that holds is recorded too: that is how a risk goes down.

Think of it like a navigator's log that records every assumption made about the ship's position, and crosses each one out only when a landmark confirms or refutes it.

**Where the story lies:** a landmark is certain; our tests are not. A belief a test held can still break under a test nobody has run yet.

## Now the real thing

**A belief record** has the columns of the company's quarterly de-risking narrative: what we believed, what we tested, what we now know, what it cost, what changed in the plan, and the risks it opened or closed. It belongs to one of seven **areas**: nodes, case inputs, outputs and results, models, maths, algorithms, visualisation. Its status is *broke* (a test showed it wrong), *held* (a test confirmed it) or *untested*.

**A risk** has an id (R-01, R-02…), a level from L1 (negligible: one run is repeated) to L5 (critical: delivery stops, or something leaves that must not), and the test that would close it. A level goes **down** only when a tested belief says what was tested. It can go **up** at any time: bad news is recorded the day it is known.

**A version.** On every node page, "Versions, and what they rest on" lists each version: the release, the request that made it, the belief behind it, what was wrong with the version before, and what this one gives. Every earlier version's sheet is kept.

**The conclusion.** The Risk management branch of layer 1 counts the ledger at every release: risks open, open at L4 or L5, closed, opened and closed this quarter; beliefs broken, held, untested; node versions released; the highest open level in each of the seven areas. Three rows conclude it:

| Row | Says |
|---|---|
| Highest open risk level | the worst open risk anywhere; one open L5 stops delivery whatever the rest say |
| Net risks closed this quarter | closed less opened; whether the quarter turned unknowns into knowledge |
| Share of beliefs tested | how much of what the platform rests on has met evidence |

The quarterly narrative puts these first, then a row per belief of the quarter, then every open L4 and L5 risk. Its paragraph of prose is written by a person on the quality team; the page never writes it for them.

## Where the simple version breaks

- **"Held" means the named test passed,** not that the belief is true beyond that test.
- **The highest level hides progress** below it: it does not move until the last risk at that level closes. That is why two more rows sit beside it.
- **The starting levels are proposals.** The package that set up the register wrote them without a test, and each says "proposed" until the quality owner confirms the scale (decision D26).
- **Counts can be moved** by splitting or merging risks; the register is reviewed like code.

## Common wrong idea

"A new version means the old one was a mistake." No. The old version was the best bet on what was known then, and its sheet is kept. A version replaces it because a belief was tested. A broken belief is the ledger working: it turns a hidden risk into knowledge. The mistake would be changing a node without saying which belief moved.

## Try it

1. Open the node library, tick "only nodes with an open risk", and open **APE budget total** (the pointing budget's total).
2. Before scrolling, predict which risk it carries and at what level.
3. Scroll to "Where the simple version breaks" and to "Versions, and what they rest on", and check.
4. Then open **Highest open risk level** in Risk management and read how it is computed.
