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
> One check is still running: the twin's full re-fly with the final twin, then its comparison against the engine
> (§4). The verdict on that line is filled in when the re-fly ends. Seventy findings for the owning groups are
> listed in `docs/S7_INVENTORY.md` §6 and placed in the next phases by `docs/S11_INVENTORY.md`. None of them
> changes a number today; each is a choice the design's owners make.
>
> **Kind:** report · **For:** the owner · **Status:** waiting for the twin's re-fly (§4), then for the owner's word
> before the next phase

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

## 4 · The twin's full re-fly (running)

Every stored twin run is being flown again by the final twin: 48 scenarios and 8 campaigns (184 runs). It resumes
from its checkpoints and its results are committed as they come. When it ends:
- `python3 tools/engine.py twin-parity` compares the engine with the twin on every scenario;
- `results/ENGINE_PARITY.md` is rewritten from it;
- the result is entered here.

*Pending: the engine against the twin, scenario by scenario, and every difference traced.*

## 5 · Costs you accepted, and what is still open

- **Accepted at S6:** the generated flight code is slower, up to +20 % in C and +26 % in Rust on one step. The
  smallest worst-case margin is still 33.21 ms of 50.
- **Not yet:** a flight image's manifest does not carry the image's hash; runs map to their image by build id.
  This is placed in S15.
- **Your approvals, still open:** S0 (the notes correction) and S1 (with a second reviewer).
- **One browser test** (the group app's offline pages) failed once in the full run and passed alone. It is run
  again after the re-fly, alone, to settle whether it is flaky.

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

When §4 is filled in: are you satisfied with S7? Then S11 starts, as the plan orders it: S11 to S16, then S8,
then S17.
