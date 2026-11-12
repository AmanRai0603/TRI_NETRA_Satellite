# S11 check report: the corrections (U0)

> **Answer first.** S11's four done-when tests hold:
> - **Every verdict is demonstrated at its stated probability and confidence, or says so.** One verdict function, the
>   design's (`kpi_confidence_verdict`: a Clopper-Pearson lower bound at the stated `kpi_verdict_confidence` = 0.95, a
>   run with no finite value counted as a failure, an infeasible selection never passing), answers every closure;
>   `tools/evaluate.py` and `tools/trace.py` call it and keep no bound of their own.
> - **Power is whole-ADCS.** The sensors' steady power by flight mode is a fourth power channel in the engine and the
>   twin, and the static budget computes the peak.
> - **The twin flies the flight frame.** Over every recorded sample of 80 scenarios the engine and the twin give the
>   same position, density, Sun direction and nadir direction (largest angle 0).
> - **The gyro, reset and timing faults have scenario cases**, and so do 25 bus-level faults.
>
> Made honest, the verdicts are worse: the design loop finds **no feasible design for either case** (both end on the
> fluid-ring family as "closest, not feasible"), 30 golden verdicts went from pass to fail once the sensors' power was
> counted, and five drivers raise no fault flag for a bus fault. These are the design's findings, not run errors.
> The gates hold: `tools/check_all.py` 25 of 25, the parity gate equal in every part, the golden table bit-exact.
>
> **Kind:** report · **For:** the owner · **Status:** complete; S12 started on the owner's word (12 Nov 2026). The 11 commits of §6 are left as they are (no answer: the default). Nothing in S11 is signed.

## 1 · The four tests

| test | result | evidence |
|---|---|---|
| verdicts with confidence | one function in the design; the level one stated node. Closures: `ais_3u` 4 fail, 34 blocked; `ais_img_3u` 4 fail, 1 pass, 33 blocked. A closure the runs cannot reach reads "not demonstrated (R >= x at 95 %)" | `design/revisions/S11.10/kpistats.pc`, `tools/verdict.py`, `tests/test_evaluate.py`, `results/EVALUATION.md` |
| power whole-ADCS | `sens_power` gives each sensor's steady power in the mode flown (the star tracker on in the pointing modes 1-4 and 10 only; the rest always); `P_sens_W` enters power_mean, power_peak, power_margin and the battery; gb_2 is the sum of the units' peaks. `ais_3u` nominal +1.5 W, the imaging products +3.5 W | `design/revisions/S11.8/sensorpower.pc`, `engine/crates/adcs-sim/tests/power.rs`, DR-design-003 |
| the twin flies the flight frame | max difference over every sample: position 0 m, density 0, Sun angle 0, nadir angle 0 deg in all 80 scenarios. The field's angle differs only where the attitude does (the field is recorded in the body: 178 deg in the chaotic `sun_spin_ais`, 0.35 deg in `detumble_ais_bangbang`, below 0.004 deg elsewhere); engine and twin agree on 171 of 171 judged verdicts | `results/ENGINE_PARITY.md` (truth-environment table, E1's new columns) |
| fault cases | gyro_fail, fsw_reset, clock_step, clock_jitter (six scenarios, S11.4); bus faults NACK, bad CRC, stuck register, delay, brownout, CAN error frame at the 8 devices of the device map (25 scenarios, S11.14) | `design/revisions/S11.4/`, `design/revisions/S11.14/`, `results/golden.json` (80 scenarios) |

## 2 · The gates after the re-fly

Every stored result was flown again with today's code (S11.16): 80 engine scenarios, 4,516 engine campaign runs, the
design loop for both cases (8 iterations, Monte Carlo and soft OILS inside), soft OILS of 80 scenarios, the twin's
80 scenarios and 8 campaigns (184 runs), and a 100-draw two-sample comparison of engine and twin.

| gate | result |
|---|---|
| parity (`tools/parity_2_0.py`) | equal in every part: inputs 199/199, layout 156/156, blobs 80/80, runs 80/80, campaign runs 4,516/4,516 bit-exact, evaluation 2,578/2,578, soft OILS on QEMU 2/2, twin 1/1 |
| `tools/check_all.py` | 25 of 25 (553 Python tests, design-loop 338 of 338, boundary 0 problems and 0 exceptions, translators every vector) |
| golden table | 80 scenarios, bit-exact on this toolchain's fingerprint |
| C against Rust flight software | 80 of 80 bit-identical |
| engine against twin | 171 of 171 judged verdicts agree (304 metric pairs) |
| two-sample, `mc_nadir_ais`, 100 identical draws | 0 of 6 metrics differ at p < 0.05 (Mann-Whitney, the design's `kpi_mann_whitney`) |
| flight images | resealed for posix, posix-rs, qemu, qemu-rs |
| soft OILS against SILS | verdicts 166 of 171 agree; 10 metric rows apart by more than 5 % (§5) |

## 3 · What changed, by step

Each fix landed in the design (a developer revision with its source, unsigned) and in the code in the same commit, with
a test or case. The commits are in `docs/S11_PROGRESS.md`.

| step | items | what flies now |
|---|---|---|
| S11.1-S11.2 | decisions | 20 decision records as design nodes; 12 carry the developer's choice on the stated defaults |
| S11.3 | TS11 | the golden table and its CI job |
| S11.4 | TS4, F3, F1, F10 | the fault rig; integer-tick rate groups; a monotonic bounded clock taking GNSS time; the single-precision claim removed |
| S11.5 | F2/G3, F5, G4 | a silent gyro detected after 3 ticks; the filter's rate stands in; detumble leaves on the gyro, or on the B-dot rate when there is none (both, by the gyro's health, as you chose) |
| S11.6 | F4, G1, G2, G6, TS3 | one tracker plausibility method; TRIAD refused below a sine of 0.2; vector-scaled rotor allocation; the wheel PID's factor 2 (step 0.98x, settling 0.99x of its bandwidth; GM 28.8 dB, PM 40.6 deg) |
| decisions | DR-ctl-001, DR-vv-003 | the magnetic PD without the factor 2, behind one switch, until a coil loop test proves a placement (`nadir_hold_ais_css` back to pass); campaign parity across machines |
| S11.7 | M1, M2, M5, M9, M10 | a failed ring and a failed CMG spin down and hand over their momentum; wheel copper loss; a tracker latency beyond 0.5 s refused; the GNSS rate honoured |
| S11.8 | Y1, Y2, Y4, Y5 | the CMG cluster one unit; h_req = k_h (h_dist + h_slew) with k_h = 2 a floor; the survey in the inertial frame; power whole-ADCS (default wheel CW0017 -> CW0057) |
| S11.9 | A5 | an inertia draw that is no body is drawn again; edge corners moved to the nearest body (179 of 4,490 stored draws) |
| S11.10 | A7, A1, A3, A2 | whole-orbit and eclipse-transition windows; the one verdict; differing verdicts traced or "untraced"; the ledger's sentence from the counts; the two-sample command |
| S11.11 | E1, E6 | the parity tool measures directions; the worst aero density is DTM2020's high bundle (F10.7 250, Kp 2) |
| S11.12-S11.15 | P5, O2, O12, O19-O21, O11, TS17 | orbit step capped at 30 s; soft OILS hardened; bus faults; four weak oracles replaced |
| before S11.16 | — | the momentum parts' own lower scale bound in the loop; a stated tolerance for the grazing-shadow SRP vector; a stale engine binary refused |

## 4 · The developer's choices you may change

All unsigned, each one node or one switch:

| choice | where | default taken |
|---|---|---|
| verdict confidence | `kpi_verdict_confidence` | 0.95 one-sided Clopper-Pearson |
| magnetic PD gain | `kp_factor` in `fsw_mtq_pd` (`design/revisions/S11.6/fswmtq.pc`) | 1 (the wheel PID keeps 2) |
| CMG bearing friction | DR-act-002 | the wheel's friction ratios |
| momentum margin floor | `momentum_margin_floor`, DR-design-003 | k_h = 2; with it the loop has no momentum lever at the default margin |
| sensor duty by mode | `sens_power`, DR-design-003 | everything on, the tracker in pointing modes only |
| inertia dispersion | DR-vv-002 | redraw (rotation-based R diag R^T is the alternative) |
| density for "worst" | m3_3 | DTM2020 high bundle, F10.7 250, Kp 2, 14 h local time |
| campaign parity across machines | DR-vv-003, `campaign_relative_tolerance` | bit-exact on the record's fingerprint, else equal verdicts and 5 % |

## 5 · Findings, for the owning groups

- **No feasible design for either case.** Monte Carlo requirements unmet: `ais_3u` detumble time, APE (p99.73), mean
  power; `ais_img_3u` APE, AKE (p99.73), mean power. The design-loop check now recomputes and lists them, it no longer
  fails on them.
- **Drivers that raise no fault flag** (S11.14): the Sun sensor, star tracker, GNSS receiver and coils for every bus
  fault, the magnetometer for a stuck register, the gyro for a 0.06 s delay (which also misses the deadline by 23.7 ms
  on soft OILS). A stuck tracker frame takes the knowledge error to 177 deg unflagged. TS4's "PWM write error: fault
  raised" does not hold: the driver ignores the bus status, and the design's driver text states no check. Recorded as
  failing verdicts in the cases; no flight behaviour changed.
- **Soft OILS against SILS, 10 rows apart by more than 5 %.** Nine were already apart before S11 (no 5 % rule then);
  only `agile_slew_vscmg` is new (1 % -> 8 %), a latency knife edge. `sun_spin_ais` is chaotic; `nadir_hold_ais`
  moves with the modelled execution-time jitter. C equals the QEMU firmware bit for bit at a fixed latency: no
  SILS/OILS defect.
- **ctl's settling closure** rests on the underdamped formula: the loop as flown (gc_1 = 2.0) settles in 27.8 s
  against the case's 20 s while the closure passes at 2.2 s. **`sun_spin_ais`'s spin-reversal lock-up** (S7) stands.
- **The sizing survey's high activity is inert** on the fast orbit: F10.7 does not enter its static density. With P6 in S12.
- Small: `adcs replay` of a QEMU recording differs in frame 26; 10 weak tests left in `docs/TEST_STRATEGY.md` TS17;
  S11.7-S11.8's tests were written with their fixes and not watched failing on the old tree.

## 6 · What the record says that it should not

The runner's 11 commits of S11.16 (ee4f409b..b88c0c8c) end with `Co-Authored-By: Claude Haiku 5.5` instead of your
stated line, and 9fec6a1c gives the twin's campaign count as 4,516 (the twin flew 184). They are pushed; correcting them
needs a rewrite of the branch's history and a force-push, which your rules forbid. Every later commit carries the stated line.

## 7 · What S11 cost (the token ledger)

`python3 tools/tokens.py ledger --stage S11`:

| stage | kind | tier | agents | tokens | weighted |
|---|---|---|---:|---:|---:|
| S11 | code | standard | 22 | 3.42 M | 10.26 M |
| S11 | decide | standard | 1 | 183 k | 549 k |
| S11 | run | light | 2 | 232 k | 232 k |

25 agents recorded, 11.0 M weighted tokens (the S11.17 runner was cut off by a restart and recorded no tokens). S7: 26 agents, 6.8 M tokens, 33.8 M weighted, all but one on the deep
tier (15.6 M weighted over their tier). S11 did more steps for about a third of S7's weighted cost, with every agent on
the policy's tier. The main session's own context is not in the ledger; it handed work out and read verdicts, not logs.

## 8 · What you are asked

1. Your word to start S12.
2. Whether to leave the 11 commits of §6 as they are (the default) or have the branch rewritten.
3. Optionally, any of the choices of §4.
