# Start here
<!-- kind: tutorial; depth: learn -->

**In one line:** you use this software by writing one CSV file, your **case**, and reading the result file it gives back; you never change the software itself, and when you want it different you send the developer team a filled **node form**.

This page walks you through your first hour, once, by doing it. The other pages are for later: [how-to pages](#where-to-go-next) for single tasks, [explanations](06_reading_a_node.md) for why things are the way they are, and [reference](05_case_keys.md) for looking things up.

## The whole platform, in one picture

```
 you ──write──▶ case CSV ──open──▶ the software ──runs──▶ result file ──open anywhere, runs nothing
                                        ▲
 you ──fill──▶ node form ──send──▶ developer team ──check, build, release──┘
```

- **The case** is your whole input: the satellite, its orbit and mission, and what its ADCS must achieve. It is the only thing you edit.
- **The result file** is one HTML page holding every number and plot of a finished run. Opening it runs nothing.
- **The node form** is how you ask for anything else: a correction, a new node, a confirmation, feedback. The developer team answers in the same file.

## Your first hour

1. **Open the case editor.** In the MATLAB zip it is `forms/case_new_case.editor.html`; in the web app, choose **Case**. It opens in any browser, offline.
2. **Load a reference case.** In "Start here", choose **Start from… → ais_3u**. That is a 3U satellite that needs its payload pointed to within 10°.
3. **Read the first sentence.** The blue box at the top answers the question you have: is this case ready, and what will a blank block?
4. **Change one number.** Under "Mass properties", change `mass.m`. Watch "Is it ready?" at the bottom: still ready.
5. **Clear one number.** Clear `mass.imax`. The first sentence now says some scenarios are blocked, and names the key. The software never guesses a number you left blank.
6. **Put it back and download.** Undo the clear, then press **Download CSV**. You have `ais_3u.csv`.
7. **Run it.** In the web app: **Case → Open**, pick the file, then **Run** the scenario `detumble_3u` on the product `SYN-P-3U-MTQ`. In MATLAB: `startup_asils`, then `rec = asils.run('detumble_3u', asils.case.read('ais_3u.csv'), 'product', 'SYN-P-3U-MTQ')`. (`SYN-` products are synthetic: fine for learning, never evidence.)
8. **Open the result.** The result file opens by itself when the run ends; it is also in your store. Read its first sentence: how many requirements pass, on which engine, and how far to trust it.
9. **Read one node.** Open the node library (`forms/index.html`) and search *spin-down*. Open **Ring spin-down time** and switch the page to **Learn** at the top right.

## Try it

On the spin-down node's page, in **Learn**, scroll to **Try it**. The page gives you the inputs of a ring from a printed source. Work out the answer from the relation above it, type it, and press **Show the answer**. If yours is outside the tolerance, check the units first.

## Three rules worth knowing on day one

1. **A blank is never guessed.** Whatever needs it is blocked, by name.
2. **UNCONFIRMED means nobody has put their name to it yet.** It runs, and every result that uses it says so.
3. **Opening a result runs nothing.** Mail it, compare it, reopen it on any computer.

## Where to go next

| You want to | Page | Kind |
|---|---|---|
| write or change your case | [Your case](01_your_case.md) | how-to |
| run a simulation, a Monte Carlo, the solver or the designer | [Running](02_running.md) | how-to |
| see, compare or share a result | [Results](03_results.md) | how-to |
| ask for a change, a new node, a confirmation, or report a problem | [Asking for changes](04_asking_for_changes.md) | how-to |
| know what a case key means | [Case keys](05_case_keys.md) | reference |
| understand what a node says and how far to trust it | [Reading a node](06_reading_a_node.md) | explanation |
| understand why a node changes between releases | [Why things change](07_why_things_change.md) | explanation |
| look up a word | [Glossary](08_glossary.md) | reference |

Every page and every form has the same three depths: **Learn** (every exercise), **Read** (the answer and the explanation) and **Expert** (the relation, the limits and the sources).

## Explain it back

Before you close this page, say to yourself, or to a colleague, in three lines: what you edit, what the software gives back, and what you do when you want the software to behave differently. If one of the three is hard to say, read that part of "The whole platform, in one picture" again.
