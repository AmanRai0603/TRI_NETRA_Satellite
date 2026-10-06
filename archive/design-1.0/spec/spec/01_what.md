
---

## 1. What is being built

### 1.1 The product

The company sells attitude determination and control systems built from a fixed catalogue. It also runs the facility that proves each one: SILS, PIL, OILS and HILS.

A customer states what their satellite must do in **one CSV file in a fixed format**: pointing accuracy, knowledge, stability, slew time, detumble time, faults to survive, the mass and power they can spare, and the satellite and orbit behind them. That file is **the case**. The platform imports it, puts every value where it is needed, and turns it into three things:

1. a shortlist of catalogue **products** that meet every written requirement, each tuned in SILS for this case, with its margins. When no product does, the answer says so, with the gap, and the case becomes a design request for the design team (§8);
2. SILS evidence for each shortlisted product, with visualisation, run on the scenarios that bind the case's requirements;
3. after the purchase order, a built and calibrated unit tested in OILS and HILS on those same scenarios, with visualisation. It ships with a verification matrix, a parity ledger and a certificate of conformance.

A product is a configuration of catalogue parts plus the flight algorithms it carries. The design team finds new products by running the same engine in **designer mode**: every part combination a family allows is swept against a satellite class's standard case, and every combination that passes is saved as a candidate. So the catalogue grows with the company's design work, and no customer is ever a branch of the tree or a line of code.

One platform serves the company's own teams and its clients. The client sees a restricted view of the same engine, never a second implementation.

### 1.2 Who uses it

Two groups meet the platform, and they meet it differently (§1.6).

| Who | Uses | Gives the platform |
|---|---|---|
| **Developer team** | the repository: the tree, the engine, the tools, the intake, CI and releases | code, sheets (only through intake), catalogue files, scenarios, releases, and a reply to every request |
| **Team members**: physics, GNC and ADCS engineers, the design team, flight software, test facility operators, production, sales, quality, management, trainees and students | the released software: the web app and workbench, the MATLAB SILS tool, the portal's internal pages, the node library | case CSVs; node forms (requests and feedback); their own saved results. Some roles also make a person's decision in the software: a promotion (H14), a price (H-quote), a certificate signature (H-cert), a parity cause (H15). |
| **Client engineers** | the portal: their project, the solver, SILS, the witness view | their case CSV, their choice of product and campaign, their own model as an FMU |

What each team member mainly does with the released software:

| Team | Uses it for | Asks the developer team for, by node form |
|---|---|---|
| Physics and GNC engineers | runs, campaigns, the tree's documents and chains | a relation, a bound, a test vector, a new node, a confirmation |
| Design team | the designer (§8.6), class standards, candidate products | a new part, algorithm or product in the catalogue ("something else") |
| Flight software engineers | SILS against their own flight build | an algorithm's parameters, a telemetry change ("something else") |
| Test facility operators | the rig, campaigns, the facility view | a scenario or a campaign ("something else") |
| Production | calibration records: each unit's measured descriptor, uploaded on the order in the portal (§7.4) | a new descriptor field ("something else") |
| Sales | the solver's shortlists, quotes | — |
| Quality | the verification matrix, parity causes, certificates | a confirmation |
| Management | layer 1 of the tree | — |

### 1.3 What the client sees, and when

| Stage | The client sees | The client can do | Stays in-house |
|---|---|---|---|
| Enquiry | the portal, the four families described, the blank case template and the case editor | create a project, upload the case CSV (or fill the case editor on screen), read the case report | everything below layer 2 |
| Sizing | the shortlist of offered products with margins after tuning, refusals named in plain words; or "no catalogue product meets this case" with the gap | pick a product to test, or request a design | L3 internals, restricted rows (D1), tuned values, candidate products |
| SILS | live run view and campaign dashboards (§13) for their case, within a run quota; every finished run as a result document | switch between shortlisted products; run the release's nominal, edge, Monte Carlo, sweep and fault campaigns; upload a model as an FMU; download and re-open results | flight software source, plant internals, controller gains |
| Quote | the requirement package (§14.1): ADCS spec, ICD, verification matrix; the quote, frozen by hash | accept, or change the case and re-quote | cost, margin, price table |
| Purchase order | the order record and its stages | upload the PO | — |
| OILS, HILS | the witness view: the same visualisation, streamed live from the rig, then replayable, and saved as result documents | watch, comment, request a re-run through sales | the rig console, fault-injection controls, facility telemetry |
| Delivery | the evidence package and the certificate of conformance | download | the run ledger across all clients |

OILS and HILS results reach the client only after the purchase order, and always with visualisation. The live witness view is the product feature, not a courtesy.

### 1.4 What the repository contains, in twelve parts

1. **The engine**: units, kernel, sheet generators, gate, bus, data bundles, faces (§3), with no in-software editing path (§3.4).
2. **The ADCS tree**: four layers, 608 node sheets once seeded, 22 KPIs each closed by evidence and 16 of them also by analysis (§5). Every node's content arrives through intake.
3. **The node form and intake** (§5.10, §5.11): the one way the software's content changes. A team member fills a form. The developer team runs the checker, the implementation agent writes the change, the checker verifies it, a person reviews, and the release carries it.
4. **Units and physics** for attitude work (§6).
5. **The catalogue**: the module descriptor standard, parts, families, products, algorithms and satellite classes; per-serial descriptors are recorded against each order as operational data, not in the repository (§7).
6. **Cases, the solver and the designer**: the fixed case CSV, its editor, checker and importer; screening, SILS tuning and selection for a case; design sweeps that save candidate products (§8).
7. **The loop engine**: a deterministic, time-stepping SILS with the flight C code linked in and device emulators on every port (§9, §10).
8. **The loop contract and the rig**: one message set; PIL, OILS and HILS on a real-time host with an interface emulation unit and an environment simulator (§11, §12).
9. **Visualisation and result documents**: one set of views for every rung, live and replayed, and every finished run saved as one file that re-opens without re-running (§13).
10. **Evidence and certification**: verification matrix, parity ledger, reports, certificate (§14).
11. **The portal**: multi-tenant, client and internal roles, run queue, quote to purchase order (§15).
12. **The MATLAB SILS twin**: the SILS engine again in plain MATLAB, released as one zip with the case editor, the node library, the result viewer and the user manual (§10.8). It is built in lockstep with the platform: every SILS element is written in both engines in the same change, from P1, so the zip is whole at every push (§10.8.7).

The flight software itself is the company's own product and lives behind the two headers. This repository ships a **reference flight software** in `fsw/` so that SILS runs out of the box: B-dot, sun acquisition, a MEKF, the IDMAS split-projection law and allocation. The company's flight build replaces it without any change to the platform.

### 1.5 Not in scope

- A flight operating system or the OBC's board support package. The repository defines `adcs_hal.h`; each OBC implements it.
- Replacing MATLAB for design. MATLAB stays the design tool, and the MATLAB SILS twin (§10.8) is where developers try new algorithms first. The loop engine is the platform's SIL, checked against MATLAB (§10.7).
- Editing the software from inside the software. There is no edit mode, no write route and no in-app sheet editor (§3.4). A change is a node form.
- Hardware drivers proven on hardware. Every rig device has a mock backend and a driver written to its interface document. Bring-up on the real device is a checklist a person runs (§12.9).
- Building a GNSS RF signal simulator. HILS buys one. OILS emulates the receiver's output at protocol level.
- Code signing, until decision D3 (§20).

### 1.6 Who does what

This is the operating model. Every other part of this document serves it.

**The developer team owns the software.** They build it, maintain it and upgrade it over time. Only they change the repository, and they change it in one of two ways: their own engineering work (the engine, the tools, the views, the catalogue, scenarios), or a request from a team member, taken through intake. The released software never changes itself. It has no edit mode and no write route.

**Everyone else uses it.** A team member gets exactly two things.

**1. The released software, as a user.** It comes as the web app and workbench, the portal's internal pages, and the MATLAB SILS tool: one zip, rebuilt on every push and attached to every release, always holding exactly the SILS the platform has, because each element is written in both engines at once (§10.8.7). With it, a team member:

- writes their **input** as a **case CSV**, in the case editor or any spreadsheet. The case is the only thing they edit (§8.3);
- checks the case: the software reports what it states, what it leaves blank, and which scenarios it can run;
- runs SILS on it: single runs, Monte Carlo, edge cases, sweeps, faults, the solver and the designer;
- sees the result in the views of §13, reads any node's document in the node library, and compares runs;
- has every finished run saved as a **result document** in their local store, filed beside the case it was run on (§13.5). They reopen it later, share it, or open one a colleague sent, and see everything again without running anything.

Visualisation, documents, interaction and saved results are features of the software, not things anyone edits.

**2. The node form, to ask for anything.** A team member who wants a node changed, a new node, a node confirmed, or who has feedback on a release, fills that node's form (§5.10) and sends the saved file to the developer team. Something that is not a node, such as a part, an algorithm, a scenario or the case format, goes in the same form as "something else". The form is written for them: every field says what it asks, why it matters and gives an example. It carries no code. It carries the relation, the connections, the bounds, the test vectors and the sources, in enough detail that the code can be written from it.

**The loop between them.**

```
TEAM MEMBER                                        DEVELOPER TEAM
───────────                                        ──────────────
case CSV ──▶ released software ──▶ result documents (local store, beside their case; re-open, compare, share)
                                        │
node form: change · new · confirm ·     │ names the case and result
feedback · something else ◀─────────────┘
     │ saved file
     ▼
                                          1. check     cargo xtask intake check <form>
                                             fail ──▶ reply in the form's history ──▶ back to the team member
                                          2. implement the implementation agent, from the checked request's brief:
                                                       intake write (sheet, test vectors, version, belief record),
                                                       code in the HOLEs,
                                                       new physics functions
                                          3. verify    cargo xtask intake verify: the node says what the form says
                                          4. review    a second developer (the node's owner group); gate, tests, downstream nodes
                                          5. release   CLI, workbench, portal, MATLAB zip, node library, manuals
     ▲                                                 │
     └───────── the form comes back: "released in <version>" ◀─┘
     then: run your own case in that version, save the result, and send feedback in the same form
```

**Every page explains itself, and every change says why.** Nobody sits beside a team member to explain a form, a result or a case: each page does it itself, to one standard (§5.12). And every request that changes a node carries the belief it rests on, what tested it, and the risks it opens or closes (§5.13). The developer team's own changes record theirs too. So each node's version history says why every version exists, and the Risk management branch of layer 1 concludes, at every release, how far the platform as a whole has been de-risked.

**What this rules out.**

- There is no in-software editing path: no workbench write route, no draft-and-confirm in the app, no form that writes a sheet by itself (§3.4).
- There is no fleet of specialised agents drafting nodes. Content comes from people, through forms. Code is written by one general implementation agent, run by a developer, under a brief the checker writes and a verify step that compares the result with the form (§5.11, §17.4).
- There is no second copy of an input. A scenario never restates a case value; a case is never typed into a scenario.

Clients outside the company use the portal (§15). It is the same software behind their tenant, with the same case CSV and the same result documents. Clients have no node form: what they want changed reaches the company through sales.
