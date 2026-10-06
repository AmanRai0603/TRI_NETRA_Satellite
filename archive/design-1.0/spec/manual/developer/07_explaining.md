# Explaining: the standard every page follows
<!-- kind: explanation; depth: read -->

**In one line:** every page a person reads (the node form, the case editor, the node library, a result, the quarterly narrative, every manual page) is written to `adcs-explain/1`, taken from *Eight Loops, One Beam*: answer first, overview before detail, one kind per section, three depths, and each concept told as say it simply, the real thing, where it breaks, try it.

## Say it simply

The people who read this software's pages did not write it, and nobody sits beside them. So each page has to do what a good teacher does: say the answer first, show the whole before the parts, tell the simple version and then the real one, say plainly where the simple version lies, and let the reader try it before being told.

*Eight Loops, One Beam* is one worked example of that, for an RF ion thruster. It keeps the learner's loop (explain it yourself, find the gaps, predict before you test, ask why until you reach a law, rebuild the number, teach it back) and adds thirteen techniques for the explainer. `adcs-explain/1` turns both into 21 rules, E01 to E21 (SPEC.md §5.12.2), in four groups: find your way, see it, learn it, and don't fool yourself.

## Now the real thing: the worked example

The platform's own worked example is one node, **Ring spin-down time** (`gf_7`). Open `forms/examples/sys_fluid_momentum_rings_ring_spin_down_time.form.html` beside this page.

| Rule | Where the page does it |
|---|---|
| E01 answer first | the blue box: what the node is, in one line; its kind, version, state and highest open risk; what the reader does next |
| E02 kind | every section's badge: "Start here" is a tutorial, "Where this node sits" and the node's document are explanations, "Versions" is reference, every field section is a how-to |
| E03 depth | the switch at the top: Learn keeps the exercises and hides the rebuilt answer; Expert hides "Start here" |
| E04 overview | "Where this node sits": what it reads → the node → where its answer goes |
| E05 where you are | the breadcrumb: ADCS platform › Layer 2 › Fluid momentum rings › Ring spin-down time |
| E10 station | the node's document: 1 Say it simply, 2 Now the real thing, 3 Where the simple version breaks, 4 Try it |
| E14 analogy | stirred tea, and "where the story lies": a closed, full, laminar ring is not a cup |
| E15 why-chain | four whys down to the Hagen-Poiseuille law of laminar pipe flow |
| E12 contrast | two rings, one difference, the bore: 21 s at 16 mm, 0.75 s at 3 mm (IDMAS v2 §03B) |
| E13 wrong idea | "a ring stores momentum like a wheel", and why not |
| E08 rebuild | the 3 mm test vector's inputs; the reader computes, then sees the printed 0.75 s |
| E18 claim tags | `[idmas_v2]` beside the relation and the test vector: sourced |
| E19 used for | "Sources this node cites", each with what the platform uses it for |
| E20 where it breaks | the laminar assumption, UNCONFIRMED, and risk R-04 |
| E17 one line | "In one line:" at the end of the document; "Explain it back" in Learn |

The node's own explanation lives in its sheet, as the `[explain]` table, and reaches it through a node form's "Explain it" section. Intake checks it: X01 (an analogy says where it breaks), X02 (a wrong idea says why), X03 (a computed node without "simply" and "one line" is a warning).

**Where each template applies it** is in SPEC.md §5.12.3. The one copy of the depth switch, badges, claim chips and boxes is `tools/explain_kit.py`, copied into every template between markers; `explain_kit.py --check` fails when a copy drifts.

**Manual pages** carry a kind line under the title (`<!-- kind: how-to; depth: read -->`) and open with `**In one line:**`. A tutorial has numbered steps, "Try it" and "Explain it back"; a how-to has numbered steps and "If it goes wrong"; a reference has a table; an explanation has "Say it simply", a section on where it breaks, and "Common wrong idea". This page is an explanation.

## Where the standard breaks

- **Marks are not understanding.** `explain_check.py` sees that a page has an answer box, badges and a station. It cannot see whether the answer is the right one, or whether a reader understood. That test is a reader's: before a template is released, someone who has not seen it fills it or explains it back, and each miss is fixed. Until that has been done for every template, risk R-17 stays open.
- **Not every rule fits every page.** A list of case keys has no station; a result has no analogy. The checker requires only the marks that apply, and a person judges the rest (E06, E07, E11, E21).
- **The standard can make a page longer.** The depth switch is the answer: Read hides exercises, Expert hides tutorials. When a page is still too long at Read, it is two pages.

## Common wrong idea

"The explanation standard is about style." It is about honesty as much as clarity. Four of its rules, E18 to E21, exist so a reader never takes an example for a measurement, a stand-in for a confirmed value, or a simple version for the whole truth. A clear page that hides where it breaks is worse than an unclear one, because it is believed.

## Try it

1. Open the `gf_7` example and switch to **Learn**.
2. Before scrolling, predict which of the 21 rules the page cannot meet for a declared node (one with no relation). Then open `forms/examples/sys_kpi_required_pointing_performance_absolute_pointing_error_ape.form.html` and check.
3. In your checkout, delete this page's `**In one line:**` paragraph, run `python3 tools/explain_check.py --md`, read the finding (E01), then restore the paragraph.
