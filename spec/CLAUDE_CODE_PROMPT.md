# Prompts for building the ADCS platform with Claude Code

Give Claude Code one phase at a time. Each phase ends on its own acceptance test and stop points (SPEC.md §19). Start the next phase only after a person has read the phase report and dealt with the stop points.

Before P0, create an empty repository named `ADCS_PLATFORM` and put this whole package in it at `_package/`, unchanged. Each phase copies from `_package/` into the repository's own paths (SPEC.md §0), so the package's README and the new repository's README never collide. Then clone VLEO_SIMULATOR beside the repository, as a scratch checkout outside it, at the pinned commit:

```
git clone https://github.com/AmanRai0603/VLEO_SIMULATOR ../VLEO_SIMULATOR
git -C ../VLEO_SIMULATOR checkout abf79ee
```

---

## The prompt for any build phase

Replace `<N>` with the phase number.

```
You are building the ADCS platform repository described in _package/SPEC.md, phase P<N> only,
for its developer team. The package in _package/ is read-only; copy from it, never edit it.

Read first, in this order: _package/SPEC.md §0 to §4 in full (§1.6 is the operating model),
then §5.10 to §5.13 (the node form, intake, the explanation standard, de-risking), then §19's P<N> section, then every section
P<N> refers to. VLEO_SIMULATOR is checked out at ../VLEO_SIMULATOR (commit abf79ee), outside
this repository. Treat it as read-only: port from it by copying, as §3 says.

Rules that override anything else you would do (SPEC.md §0.2):
- Copy VLEO files verbatim and change only the names in §3.3 and the domain nouns.
  Keep each file's template, ordering and comments. Remove completely every editing path
  §3.4 lists: the released software never writes its own content.
- Node content arrives only through intake: `cargo xtask intake write` from a request that
  passed `intake check`, then `intake verify`. Never write a sheet any other way — not by hand,
  not by script. That includes the 82 seed forms in P1.
- Never write a person's name. `confirmed_by` comes only from a form's "Checked by", through
  `intake write`; otherwise "UNCONFIRMED · <what> · awaiting a person".
- Never produce an expected value. Test vectors come only from a request, which cites the page.
- Never widen a tolerance, skip a test, or edit a generated file outside a HOLE.
- A refusal is never a substitution. Several refusals are this phase's correct result.
- At a stop point in §19 or a decision in §20: prepare what a person needs, then stop.
- Record every disagreement between SPEC.md and VLEO's code in docs/SPEC_DEVIATIONS.md.
- Keep manual/user/ and manual/developer/ true: a command, route, view or case key you add,
  remove or rename is changed on its manual page in the same commit.
- Every page a person reads follows adcs-explain/1 (§5.12): a view or template you write carries
  the kit from tools/explain_kit.py and its marks; tools/explain_check.py must pass.
- The MATLAB twin moves in lockstep (§10.8.7): every SILS element you write in the platform
  (a physics function, a model, an algorithm, a metric, the recorder, the runner) you write in
  matlab_sils/+asils/ in the same commit, from the same relation, and both pass the test the
  twin map names. `cargo xtask twin check` (tools/twin_check.py before it exists) must pass.
- Every change rests on a recorded belief (§5.13). Your own changes record theirs with
  `cargo xtask derisk record` (`python3 tools/derisk.py record` before it exists); never set or lower a risk level
  yourself: a level goes down only by a recorded test, and levels are proposals until D26.

Work: do everything P<N> lists. Run python3 tools/validate_plan.py whenever you touch
plan/, catalogue/, designs/, scenarios/, campaigns/ or devices/.

Finish: every command in P<N>'s Green line passes. Paste the real output of each acceptance
step into docs/BUILD_EVIDENCE.md. Write docs/phases/P<N>.md: what you built, what is
mocked, what you did not do, and what a person must do before P<N+1>. Commit in
type(scope): form (tools/commit_message.py --types).
```

---

## The prompt for implementing one request (after P1, and for each seed form in P1)

A developer runs this in a checkout, on a request that passed `cargo xtask intake check`.

```
Implement intake/requests/<request id>/brief.md, and nothing else.
Your standing instructions are AGENTS.md and intake/AGENT.md. The brief names the node, the
files you may change and the commands, in order. `node.toml` and `fixtures.toml` are written
only by `cargo xtask intake write`; you write the HOLEs and any new physics function the brief
lists, with property tests only, and each new physics function's MATLAB twin in
matlab_sils/+asils/+physics/ in the same commit, run on the request's test vectors. `intake write` also writes the node's next version and the
belief record the request carries; you never write versions.toml or derisk/ yourself. Finish when
`cargo xtask intake verify`, `cargo xtask derisk check`, `cargo xtask twin check` and `cargo xtask gate` pass. Commit on intake/<request id> with the trailer `Request: <request id>`; do not push.
If the request cannot be implemented as written, stop and say why.
```

---

## Notes for the person driving it

- **The operating model (§1.6).** The developer team changes the software; everyone else uses it. A team member's only input is a case CSV, and every request reaches the team as a node form. Build nothing that lets the released software edit itself. The `no-writes` CI job checks this.
- **P0** is the only phase that copies a lot. Check the copy list, the removals (§3.4) and the rename map (§3.3) in its report before anything else is built on it. The daemon must answer no write route.
- **Every page explains itself; every change says why.** The explanation standard (§5.12) and the de-risking ledger (§5.13) start in P0, with the manuals and `derisk/`, and every later phase keeps them: a view without the kit's marks, or a change without its belief record, fails CI. The Risk management branch concludes each release. Risk levels are the package's proposals until a person settles D26.
- **P1** builds intake and proves it on the 82 seed forms, then on one real round trip: a node form exported, edited in a browser, checked, implemented, verified, replied to. It needs two things from a person before all its rows can run: the IGRF-14 coefficients file downloaded from NOAA NCEI, and the review and publication of the `igrf14` and `atmos-density` bundles. Until then, rows reading the field or the density answer `DataMissing` by name. That is correct.
- **Customers are cases, not code.** Every customer is one CSV in the fixed format (`plan/case_template.csv`). Never add a customer to the tree, a product for one customer by hand, or a value the CSV should carry to a scenario. To change what a case can say, edit `CASE_INPUTS` in `tools/build_tree.py`, bump `adcs-case/<n>`, and write the migration (§8.3.6).
- **Every finished run is a result document, beside its case.** It holds the case, metrics, verdicts and kept channels (§13.5). It reopens and compares with no rerun, and opening one made elsewhere files it beside its case in the local store and runs nothing. The files are the record; `index.csv` and `index.sqlite` are caches. A result document is never read back as evidence.
- **The MATLAB twin is built in lockstep, from P1.** `matlab_sils/` implements SILS again in plain MATLAB from the same exported definitions (§10.8). Every element is written in both engines in the same change, as `plan/twin_map.toml` lists them, and the `twin` job refuses a pull request that moves one engine without the other (unless it is labelled `twin:none` with a reason). So the MATLAB SILS zip built on every push is whole for the phase reached, and `TWIN.md` inside it says what it carries. A difference between the engines is a parity-ledger line with a cause. The zip carries the manual, the case editor, the node library, the result viewer and an empty store, and is always built by `tools/pack_matlab.py`.
- **OILS and HILS needs come from the case.** The tree's "OILS rig needs" and "HILS rig needs" rows (layer 2) say what a rig must do for this case; the lab file says what the rig can do (the facility rows, layer 1, supplied by `lab`). `adcs rig fit` compares them and refuses a campaign by name when the lab falls short (§12.10). Never fill a lab value that has not been measured: `nan` is the honest value.
- **Products come from the designer, into the store.** `adcs design` saves candidates as data, never into `catalogue/`. A candidate reaches the catalogue by request, and is offered only on a recorded H14 decision, never one holding a synthetic or placeholder part.
- **P3**: if you hand over the existing detumble C code, it goes behind `adcs_fsw.h` first, before the reference flight software (§19 P3).
- **Low credibility is expected.** UNCONFIRMED on every seeded row until an engineer confirms it by form, InputPedigree 0 on every result that flew a synthetic part (P4), and a certificate that refuses to issue (P8) are acceptance results, not failures.
- **Change the plan with its tools, never by hand.** From P1 the repository owns its copies of `plan/` and the tools. Before any sheet is published, a change to the tree's shape is: edit `tools/build_tree.py`, run it, run `tools/validate_plan.py` and `tools/check_seed_with_vleo.py ../VLEO_SIMULATOR --write`, and reseed. After that, a new group is added with `tools/seed_tree.py --add-group` (§5.7), and its nodes arrive as new-node requests. Node content is never changed either way: it is a request. `_package/` stays as it was delivered, as the record of the starting point.
- **Stop points by phase** (SPEC.md §19–§20): P0 D10, D12 · P1 D4, D5, D9, D11, D25, D26, D27, the IGRF file and two bundles · P2 D1, D7, D13, D18, D21, the default 3U cases' reference requirements (a person) · P3 D5 if still open · P3M D23, D24, twin parity causes (H15) · P4 H15, algorithm bounds (H14), D22 · P5 panel sign-off, D2 · P6 D14 · P7 bring-up and H-rig · P8 D15 · P9 D6, D16, D17, D19, D20 · P10 D3, D8.
