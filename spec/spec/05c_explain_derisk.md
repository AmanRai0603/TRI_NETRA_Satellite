
### 5.12 The explanation standard, `adcs-explain/1`

**In one line:** every page this platform shows a person (the node form, the case editor, the node library, a result, the quarterly narrative, every manual page) is written to one standard, taken from *Eight Loops, One Beam*, so that each one explains itself to someone who has never seen it.

#### 5.12.1 Why there is a standard

Say it simply first. The software never changes itself, and the people who change what it says are not the people who wrote it (§1.6). So every document is read, and often filled in, by someone new, with nobody beside them to explain it. A document that only its author understands turns into requests that fail the checker, results read the wrong way, and cases edited blind.

*Eight Loops, One Beam* (v2) is a worked example of teaching one hard system, an RF ion thruster and its controller, to three kinds of reader at once. It does two things this platform needs:

- the **learner's loop** (ten moves): map what you know, explain it yourself, find the gaps, predict before you test, go back to the source, ask why until you reach a law, rebuild the number, simplify, repeat, teach it back;
- **thirteen techniques for the explainer**: words beside pictures, live models, concrete before abstract, overview then zoom then details, side-by-side comparison, zoom levels, typed sections, answer first, depth for novice or expert, worked then faded examples, contrasting cases, correcting a common wrong idea, and analogies that say where they stop being true.

The standard below is those, turned into rules a page can be checked against. The example stays the example: the platform's own worked example of the standard is one node's document, the ring spin-down time (`gf_7`), walked through rule by rule in the developer manual ([Explaining](../manual/developer/07_explaining.md)).

Where the simple version breaks: a standard cannot make a page clear. It can only make sure the moves that make pages clear are all there. Whether a reader understood is tested by a reader (the teach-back test of risk R-17, §5.13), not by `tools/explain_check.py`.

#### 5.12.2 The rules

Four groups, as in the example: finding your way, seeing it, learning it, and not fooling yourself. The **Mark** column is what `tools/explain_check.py` looks for (in HTML, a `data-` attribute on the rendered page; in Markdown, the text named). A rule with no mark is reviewed by a person.

| Rule | What the page does | Mark |
|---|---|---|
| **Find your way** | | |
| E01 Answer first | The first sentence under the title answers the reader's question: a result's verdict; what a form is for and what the reader has at the end; a manual page's answer. Everything after it is support. | `data-explain="answer"`, the first block in `main`; in Markdown, the first paragraph starts `**In one line:**` |
| E02 Say what kind it is | Every section is one of four kinds, and shows it: **tutorial** (learn by doing it once), **how-to** (do one task), **reference** (look something up), **explanation** (understand why). A section never mixes two. | `data-kind` on each section, with a visible badge; in Markdown, `<!-- kind: … -->` under the title |
| E03 Three depths | One switch, **Learn · Read · Expert**, on every HTML page. *Learn* keeps every exercise, with answers hidden until tried. *Read*, the default, shows the answer, the explanation and the pictures, and skips the exercises. *Expert* shows the answer, the relation, the limits and the sources, and hides the tutorial text. A Markdown page is written at one depth and says which. | a `data-depth-switch` with three buttons; content carries `data-show`; in Markdown, `depth:` in the kind line |
| E04 Overview, then zoom, then details | A map of the whole comes before any part. Details open on demand (a `<details>`, a hover on a source), never in the way. | `data-explain="overview"` before the first detail section |
| E05 Where you are | A breadcrumb in zoom levels, largest first: Platform › Layer › Group › Node, or Store › Case › Result. | `data-explain="zoom"` |
| **See it** | | |
| E06 Words beside pictures | A label sits on the thing it names, not in a legend elsewhere. A picture has one sentence saying what to see in it. | person |
| E07 Same axes | Things compared are drawn side by side, on the same axes and scales. Never two y-axes. | person |
| E08 Rebuild it | Where the page holds a known answer (a test vector, a campaign's metric), the reader can try to reproduce it before it is shown. | `data-explain="rebuild"` where the page has a test vector |
| E09 Never colour alone | Every state (pass, fail, changed, open, closed) is an icon and a word. Colour only repeats it. | `data-state` elements carry an icon and a word |
| **Learn it** | | |
| E10 Station order | A concept is explained in four steps, in this order: **Say it simply** → **Now the real thing** → **Where the simple version breaks** → **Try it**. | `data-step="simply|real|breaks|try"` in order |
| E11 Worked, faded, solo | An example filled in, then one half filled, then the reader's own. In a form: the example beside the field, the reference value shown, then the blank. | person |
| E12 Two cases, one difference | Where two things are easily confused, they are shown side by side, differing in exactly one thing. | `data-explain="contrast"`, not empty; whether it differs in one thing only is a person's check |
| E13 Common wrong idea | The wrong idea is stated plainly, then why it is wrong. Stating only the right idea leaves the wrong one in place. | `data-explain="wrong-idea"` containing `data-explain="because"` |
| E14 Analogy with its limit | An analogy keeps the relations that matter, and says where the story lies. | `data-explain="analogy"` containing `data-explain="analogy-breaks"` |
| E15 Why, down to a floor | "Why?" is asked until it reaches a named law, a standard or a recorded decision, and stops there, naming it. | `data-explain="why"` with a non-empty last item; that the last item names a floor is a person's check |
| E16 Predict before reveal | In Learn, the reader is asked for a guess before the answer is shown. | `data-explain="predict"`, shown in Learn only |
| E17 One line, then teach back | Each station ends with its one-line summary. Each document ends, in Learn, with a prompt to explain the whole back in a few lines. | `data-explain="one-line"`; `data-explain="teach-back"` shown in Learn |
| **Don't fool yourself** | | |
| E18 Every claim carries its tag | Any number or statement that is not the reader's own says what it is: **sourced** [n] (with the page), **derived** (worked here from sourced relations), **reference** (the reference case's value, UNCONFIRMED until a person confirms it), **example** (a demonstration or a made-up number, labelled so). Fiction is never unlabelled. | `data-claim="sourced|derived|reference|example"` |
| E19 Sources say what they are used for | Every source listed says what this page uses it for. | each source entry has `data-explain="used-for"`; in Markdown, a "Used for" column |
| E20 Say where it breaks | Every document says where it stops being reliable: the assumptions and when each fails, the credibility, the open risks. | `data-explain="breaks"` |
| E21 Plain words first | The plain description comes before the technical name, never instead of it ("say it without the word"). | person |

**Kinds of page, and the sections each must have.** In Markdown (the manuals), `tools/explain_check.py` requires, by kind:

| Kind | Must have |
|---|---|
| tutorial | numbered steps; a "Try it" section; an "Explain it back" section |
| how-to | numbered steps; an "If it goes wrong" section |
| reference | at least one table |
| explanation | "Say it simply"; a section whose heading says where it breaks; "Common wrong idea" |

Every Markdown page has the kind line and the one-line answer. A page may hold several sections of one kind; a section of another kind goes on its own page.

#### 5.12.3 Where each template applies it

| Page | Answer first (E01) | Stations (E10) | Where it breaks (E20) |
|---|---|---|---|
| Node form (§5.10) | what this form is for, what the reader sends, and the node's state in one sentence | the node's document: its own *Say it simply*, the relation, its assumptions, and *Rebuild it* from its test vectors | assumptions and when they fail; UNCONFIRMED; the node's open risks |
| Case editor (§8.3.3) | whether the case is ready to run, and what is missing | each section: what it sets, the reference value, the edit | a blank row blocks named scenarios; ranges are drawn from, not checked |
| Node library (§5.10.5) | how many nodes are written, confirmed and at risk | — | unwritten and unconfirmed nodes named |
| Result viewer (§13b) | the verdict sentence: how many requirements pass, on what engine, at what credibility | each metric: what it measures, the number, the margin, what could make it wrong | the engine, demo or real; the credibility; the assumptions of the rows read |
| Quarterly narrative (§5.13) | the platform's conclusion: highest open level, net risks closed, share of beliefs tested | each belief: believed, tested, now know | untested beliefs; open L4 and L5 risks |
| Manual pages | `**In one line:**` | explanation pages | explanation pages: "Where … breaks" |

**The node's own explanation.** A node form carries an optional `explain` object in `request.proposed`: `simply` (the node in plain words), `one_line`, `wrong_idea` and `wrong_because`, `contrast` (two cases, one difference), `analogy` and `analogy_breaks`, and `why_chain` (a list, its last item naming the law, standard or decision it stops at). `intake write` writes them into the sheet's `[explain]` table, and the node's document shows them as its station. Intake checks X01 to X03 (§5.11.2): an analogy must say where it breaks, a wrong idea must say why it is wrong, and a computed node without `simply` and `one_line` is a warning, because its document then serves experts only.

#### 5.12.4 Checking it

`tools/explain_check.py` (in the repository, `cargo xtask explain check`) renders every HTML template's examples in headless Chromium and reads every manual page, and reports each missing mark by rule and file. It runs in the `explain` CI job (§18) with the form browser check. Its own `--selftest` removes or breaks each mark it checks (E01–E05, E08–E10, E12–E20 in the rendered pages; the kind line, the one-line answer, each kind's sections and E19 in Markdown) and requires the check to fail by that rule.

It checks presence and order, not quality. The quality check is a reader: before a template is released, one person who has not seen it fills it, or reads it and explains it back, and each miss is fixed. That test is how risk R-17 is lowered (§5.13).

#### 5.12.5 Common wrong idea

"The standard makes every page longer." It does not have to. *Answer first* and the depth switch make the page shorter for most readers: Read hides the exercises and Expert hides the tutorial. What grows is what is available, not what is in the way.

---

### 5.13 De-risking: beliefs, versions and the risk branch

**In one line:** every decision the platform rests on (a node, an input, an output, a model, a relation, an algorithm, a picture) is recorded as a *belief* with the test that would settle it; a node gets a new version only because a belief broke or a risk needs lowering; each version keeps what was wrong with the last and what it gives; and the risk branch of layer 1 adds all of it up into one conclusion per release.

#### 5.13.1 Say it simply

Every choice in this platform is a bet: that a dipole field is close enough, that five pointing errors add in quadrature, that one CSV holds a whole case. Most bets are right. The expensive ones are the wrong bets nobody wrote down, because nobody knows which results rest on them.

So each bet is written down as a **belief**, with what would test it. When a test breaks a belief, the node that rested on it gets a **new version**, and the version says what was wrong with the last one and what the new one gives. Each belief that is still untested is carried by a **risk** with a level. And a branch of the tree, **Risk management**, counts it all, so a release can say in one line how much of what it rests on has met evidence.

This is the company's quarterly de-risking narrative (its template's title and columns are regenerated as `derisk/narrative_template.xlsx`: what we believed, what we tested, what we now know, what it cost, what changed in the plan, risks opened and closed), kept for every decision instead of once a quarter, and joined to the nodes it is about.

#### 5.13.2 Now the real thing

| Object | Where it lives | What it is |
|---|---|---|
| Risk register | `derisk/risks.toml`, `adcs-risk-register/1` | every risk `R-nn`: title, area, owner (a team), why it is real, what is done, the **closing test**, its level 0–5, and every move of the level (quarter, from, to, the belief that moved it) |
| Belief record | `derisk/beliefs/<id>.toml`, `adcs-belief/1` | quarter; area; what it is about (node ids, paths); what we believed; status `broke`, `held` or `untested`; what we tested; what we now know; what it cost ($k); what changed in the plan; for a node version, the previous version's issue and this version's benefit; for an untested belief, the test that would settle it; the risks it carries; its moves of risk levels |
| Node version | the node folder: `versions.toml` and `versions/<n>/node.toml`, `fixtures.toml` | one entry per version: number, release, request id, belief id and status, previous issue, benefit. The folder keeps the sheet as each version had it |
| Risk branch | layer 1, `mgt_risk_management` (§5.2): Risk register, Beliefs and versions, Open risk by area, Conclusion | 18 declared rows counted from the ledger (supplier `derisk`, never a form) and 3 computed conclusion rows |
| Narrative | `cargo xtask derisk narrative --quarter Q3-26` | one page per quarter, HTML to the explanation standard, and the same table as `.xlsx` and `.csv` in the template's columns |

**Areas.** A belief and a risk each belong to one of seven areas: *node*, *input* (the case format and what a run reads), *output* (results, certificates, what leaves the company), *model* (the plant, the environment, the rig), *math* (the relations in `adcs-core::physics`), *algorithm* (estimation, control, tuning, the solver, the designer), *visualisation* (every page a person reads). The "Open risk by area" rows are the highest open level in each.

**Levels** (proposed; decision D26 confirms the scale and every starting level at P1):

| Level | Name | If it happens |
|---|---|---|
| 0 | closed | — |
| 1 | negligible | one run is repeated |
| 2 | minor | one node needs a new version |
| 3 | significant | a release is withdrawn or a quote is redone |
| 4 | major | a result already delivered, or a certificate, is in doubt |
| 5 | critical | delivery stops, or something leaves that must not |

The idea is ECSS-M-ST-80C's severity and likelihood; the mapping to these five levels is ours, which is why D26 records it.

**The rules of the ledger** (`cargo xtask derisk check` over the crate `adcs-derisk`, and `tools/derisk.py check` in the package):

- A level moves only through a belief, and the move is written in both files with the same quarter, from, to and belief id.
- A level goes **down** only by a tested belief (`held` or `broke`, with `tested` filled). Nothing is lowered by argument, by review or by time passing.
- A level may go **up** at any time, by any belief, tested or not. Bad news is recorded the day it is known, not saved for a meeting.
- An untested belief names the test that would settle it. An open risk names its closing test.
- Owners are teams (§5.9). The person who recorded a belief is the requester of its request.

**How a version is made.** A change or new-node request carries a **De-risking** section (§5.10.3): the belief record. Intake checks it (D01 to D07, §5.11.2). `intake write` then does three things besides the sheet:

1. copies the node's current sheet to `versions/<n>/` before writing the new one, and appends version *n + 1* to `versions.toml`, with the request id, the belief id (the request id), the previous version's issue and this version's benefit;
2. writes `derisk/beliefs/<request id>.toml`;
3. applies the belief's moves to `derisk/risks.toml`, each as a history entry.

`intake verify` compares all three with the request. `intake mark published --release <v>` stamps the release on the version. A seed form writes version 1, "first build", with no belief record of its own; the package's beliefs about the pilot rows arrive with `derisk/` (P0). A confirmation writes no version: it is a belief that **held**, recorded in `derisk/beliefs/` when the form carries one.

**Decisions outside the tree.** A model in `adcs-sim-core`, a change to the result viewer, a new algorithm: the developer team's own change records its belief with `cargo xtask derisk record` (a belief file, and the moves), and the `derisk` CI job refuses a pull request that touches `crates/adcs-sim-core`, `crates/adcs-sim`, `crates/adcs-tune`, `crates/adcs-solve`, `crates/adcs-result`, `web/`, `forms/`, `results/`, `catalogue/algorithms/` or `scenarios/` without a new belief file, unless it carries the label `derisk:none` and a line saying why nothing believed changed (a refactor, a typo).

**The rollup.** `cargo xtask derisk rollup [--quarter Q]` counts the register and the beliefs into the 18 declared risk rows. The release's preparation commit writes the counts to `derisk/rollup.toml`, and `adcs-mod-management` reads them through the supply map, as a product row reads the catalogue bundle; so the values are what those rows answer in that release, and a form never sets them. The three conclusion rows are computed by the engine, like any other: the highest open level (`risk::highest_level`), the net risks closed this quarter (`risk::net_closed`), and the share of beliefs tested (`risk::share_tested`). Together they are the conclusion the platform reports about itself, on the Risk management page of the web face and the portal (§15), and at the top of the narrative.

**The narrative.** `cargo xtask derisk narrative --quarter Q3-26 --out <dir>` writes `narrative_Q3-26.html` (the conclusion first, then the risks by area on the same scale, then one row per belief of the quarter, then every open L4 and L5 risk), `narrative_Q3-26.xlsx` in the template's seven columns, and `.csv` in the same seven plus three that identify each row (belief, status, area). The paragraph of prose at the top is a person's: the quality team writes it in `derisk/narratives/Q3-26.md` (D27). Until it is written, the page says so, and nothing is written in its place.

#### 5.13.3 Where the simple version breaks

- **A belief is only as good as its test.** "Held" means the named test passed, not that the belief is true beyond it. B-004, for example, held for the reference cases' round trip; whether people find the case editor enough is a different belief.
- **The maximum hides progress.** The highest open level does not move until the last risk at that level closes. That is why the conclusion has three rows, not one (B-014 records this as a belief, untested).
- **Counts can be gamed** by splitting one risk into several. The register is reviewed like code (H-rules, §17), and D26 says who may open and lower a risk.
- **Starting levels are proposals.** The package wrote them from §21 without a test. Every one says so in its history until D26 confirms it.

#### 5.13.4 Common wrong idea

"A new version means the old one was a mistake." No. A version replaces its predecessor because a belief was tested; the old version was the best bet on what was known then, and its sheet is kept. A broken belief is the ledger working: it turns a hidden risk into knowledge. What would be a mistake is changing a node without saying which belief moved.

#### 5.13.5 Try it

Open `gp_5`'s node form (the APE budget total). Before reading its assumptions, predict which risk the row carries and at what level. Then open `derisk/beliefs/B-008.toml` and find the risk it opened, and its level, with `python3 tools/derisk.py table --check` passing and R-15 in SPEC.md §21. Last, run `python3 tools/derisk.py rollup`: the maths area shows L4, not R-15's L3. Why? (Another maths risk, R-03, is higher.)

#### 5.13.6 Acceptance

In this package:

- `python3 tools/derisk.py check` passes on `derisk/`, and `python3 tools/derisk.py selftest` refuses each deliberate mistake in the ledger by its rule;
- `python3 tools/derisk.py rollup` gives the 18 risk rows' values, and `narrative` writes the page, the `.xlsx` and the `.csv`;
- `python3 tools/intake.py selftest` includes a change with a broken belief that moves a risk, and refuses each D and X mistake by its code;
- `python3 tools/explain_check.py` passes on every template's examples and every manual page.
