# S7 check report: every relation from the database

> **Answer first.** S7's three done-when tests hold:
> - **0 built-in nodes:** no relation of the design is computed in code. At the start there were 271 built-in
>   nodes, 147 flight parameters with no value and 23 published models in code.
> - **The parity gate holds:** today's design, read alone, gives every one of 1.0.0's answers.
> - **The boundary check finds no relation or published model in code:** 1,916 source files, 0 problems,
>   0 declared exceptions.
>
> The MATLAB twin opens the database itself and flies only code generated from it.
>
> The twin's re-fly with the final S7 twin is in: the engine and the twin agree on all 129 judged verdicts in
> 48 scenarios (§4). Seventy findings for the owning groups are
> listed in `docs/S7_INVENTORY.md` §6 and placed in the next phases by `docs/S11_INVENTORY.md`. None of them
> changes a number today; each is a choice the design's owners make.
>
> **Kind:** report · **For:** the owner · **Status:** complete; S11 started on the owner's word (6 Nov 2026)

## 1 · The three tests

| test | result | evidence |
|---|---|---|
| built-in nodes | **0** of 1,275 nodes. Their behaviours: open 301, stated 284, method 273, children 134, lookup 126, evidence 119, closure 38 | `tests/test_built_in.py`, `python3 tools/health.py tests/regression --built-in` |
| parity gate | **equal everywhere**. Inputs 167/167, parameter layout 152/152, blobs 48/48, runs 48/48, campaign runs 4,516/4,516, evaluation 2,520/2,520, soft OILS on QEMU 2/2, the twin 1/1. C equals Rust bit for bit | `results/PARITY_2_0.md`, `tools/parity_2_0.py` |
| boundary | **0 problems, 0 exceptions** in 1,916 source files: 1,203 generated, 315 tests, 26 toolbox and the rest code that enables (core, runtime, rigs, readers, translators, tools, application) | `tools/boundary.py`, `tools/boundary.toml`, the `boundary` check of `tools/check_all.py` |

The evaluation's baseline moved once, at S7.19, for a reason: the evaluation now reads the stated values and the
wiring the design gives, where before it guessed. Each moved row is named with its reason in `results/S7_19_CHANGES.md`.

## 2 · What moved into the database

- **The relations.** All 91 relations computed in code are now methods: act 38, design 19, sens 14, env 11, dyn 4,
  pnt 4, oils 1. Each is transcribed from the code it replaces and the source that code cites, marked as the
  developer's, and not yet signed.
- **The non-relations.** The 180 built-in nodes that were not relations (achieved holders, closures, owner
  pointers, runtime rows) were reclassified.
- **The flight parameters.** The 147 flight parameters now have their values from the design: 77 methods and
  70 stated. The engine no longer derives them in `config.rs`.
- **The published models, with their data as tables** (your ruling of 7 Oct):
  - time scales, frames and the IAU 2006/2000A kernel, and Earth orientation with its tides;
  - IGRF, DE440, DTM2020 and JB2008, gravity, solid and ocean tides, relativity, and the space-weather indices;
  - the metrics (how each KPI is measured).

  Code keeps only the file readers (SPK, EOP, ICGEM, FES, space weather).
- **The sizing laws, the catalogue's derive rule, the Floquet certificate and the design-loop rules.**
- **The generated code.** The engine's models, the sizing, the relations crate and the twin's functions are
  generated from the design (`tools/engine_build.py gen`). The per-group crates retired into one.

**What stays code**, the enabler only:
- the maths toolbox (vectors, quaternions, random streams, integrators, the Riccati and eigenvalue solvers);
- the engine's step order and recorder;
- the flight software's runtime;
- the rigs and the file readers;
- the translators, the library and the application.

## 3 · The MATLAB twin

- `trinetra.open` reads a `.tndb` through the library's own `tndb read`, with no toolbox; this works in MATLAB
  and Octave.
- `trinetra.build` generates every MATLAB function from it: the models, the relations and the flight
  algorithms.
- `trinetra.run` flies it; there are also `campaign`, `which`, `use` and `health`.
- The twin flies the generated flight software with the engine's parameter blob, and draws the same random
  streams. The POP is kept only as a test reference.
- Faster MATLAB code generation made the twin 1.26 to 1.69 times faster, with bit-identical results.

## 4 · The twin's full re-fly

Every stored twin scenario is being flown again by the final S7 twin (48 scenarios; 46 in at 11:20 UTC on 6 Nov).
It resumes from its checkpoints and its results are committed as they come.

The 8 twin campaigns (184 runs) are not flown again on the S7 twin. S11 already changes the engine code the twin's
parameter blob comes from (`adcs-sim/src/config.rs`), so campaigns flown now would be stale when they land. They
are flown once, with the S11 twin, in S11.16. The engine's campaigns are already held by the parity gate (4,516 of
4,516), and the comparison below is per scenario.

The re-fly ended at 12:40 UTC on 6 Nov: all 48 scenarios flown by the final S7 twin. `python3 tools/engine.py twin-parity`
wrote `results/ENGINE_PARITY.md`. Before S7 it said 115 of 116 verdicts over 43 scenarios.

**The engine and the twin agree on every verdict: 129 of 129 judged metrics**, over 207 metrics in 48 scenarios.

| how close the 203 numeric pairs are | pairs |
|---|---|
| identical | 79 |
| within 0.1 % | 177 |
| within 1 % | 199 |
| more than 1 % apart | 4, all in `sun_spin_ais` |

- **`sun_spin_ais`.** This is the one scenario whose numbers part: Sun angle 111 against 142 deg, spin-rate error
  1.39 against 1.95 deg/s, Sun-spin share 0.73 against 0 %, power 1.6 % apart. Both sides fail the same three
  requirements, so no verdict differs. It was traced in S7.17, and it is not a model difference. The two sides
  first differ in the 9th digit at 235 s: Octave's maths library against the engine's. The tumble amplifies that,
  and the spin-up enters 12 s apart. Then the flight logic's spin-reversal lock-up decides one of them. That
  lock-up is a GNC finding: the sign-flip guard needs a hemisphere manoeuvre, not a spin reversal.
- **The rest are within 0.71 %.** The largest is `mission_cmg`'s pointing over the last half orbit, which is the
  same last-bits effect over a long run.

**The browser test:** the group app's offline pages pass alone, 9 of 9, run after the re-fly (155 s). They had
failed once inside the full test run while the re-fly held all four processors. That points to a timing margin
under load, not a fault in the pages. It is kept as an item for S8, where the application's browser tests are
rebuilt.

**Verdict on section 4:** the twin flown from the database reproduces the engine on every verdict, and on every
number to within the known last-bits effect.

## 5 · Costs you accepted, and what is still open

- **Accepted at S6:** the generated flight code is slower, up to +20 % in C and +26 % in Rust on one step. The
  smallest worst-case margin is still 33.21 ms of 50.
- **Not yet:** a flight image's manifest does not carry the image's hash; runs map to their image by build id.
  This is placed in S15.
- **Your approvals, still open:** S0 (the notes correction) and S1 (with a second reviewer).
- **One browser test** (the group app's offline pages) failed once under load and passes alone (§4); kept for S8.

## 6 · Found on the way, for the owning groups

`docs/S7_INVENTORY.md` §6 has 70 findings. Every relation was transcribed as flown, so no run moved; each finding
is a question for the group that owns the node. The ones that change a number when answered:
- **env:**
  - albedo and Earth IR: flown 0.30 and 237 W/m², cited 0.31 and 235 W/m²;
  - solar pressure: two models 0.46 % apart;
  - two low-precision Sun series kept.
- **ctl:** the gc_2 closure fails by analysis.
- **evaluation:** 13 values where the evaluation and the engine differ, each with its reason in
  `results/S7_19_CHANGES.md`: the AIS products' +X boresight against gdn's default +Y in five flight parameters of
  ais_3u, and 1.0.0's analysis rows gd_4, gd_5, gm_1 and gm_2 against the sizing's survey.
- **fsw:** the design's default control law, and the gravity-gradient reference.
- **dyn, act:** constants the code carried, now stated nodes with the 1.0.0 code as their source.

`docs/S11_INVENTORY.md` places each finding in the phase that answers it (S11 to S16), or leaves it with its group
for the ownership stage.

## 7 · What S7 cost (the token ledger)

`python3 tools/tokens.py ledger --stage S7`: 26 agents, every one but the last on the deep tier.
- Migration: 15 agents, 5.0 M tokens.
- Code: 4 agents, 1.4 M.
- Transcription: 2 agents, 1.4 M.

On the policy's tiers this would have been about 15.6 M weighted tokens less. The main session's cold wakes cost
more than all the agents together. From S11 on the rules of `docs/PLAN_2_0.md` "Working economy" apply, and each
phase's report shows its ledger against this one.

## 8 · What you are asked

Nothing: S7 is complete. You said to go ahead with S11, and it is running in its own session (S11 to S16, then S8,
then S17). The twin's campaigns are flown there, with the S11 twin (S11.16).
