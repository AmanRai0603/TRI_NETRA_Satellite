# S11 progress: where the corrections stand (hand-over record)

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.

The working record of S11 (`docs/PLAN_2_0.md`, S11 Corrections (U0); the steps are those of `docs/S11_INVENTORY.md` §6).
Written at a hand-over between sessions; the next session keeps it current and `docs/S11_REPORT.md` is written from it at
the phase's end. Everything here is the developer's and unsigned.

## Steps done (on `claude/brave-dijkstra-rshrb7`, last commit at this writing b8727d79)

| step | items | commits | what it shows |
|---|---|---|---|
| S11.1, S11.2 | the 9 owner questions, the 15 decisions | cf397d1, dc157c50, 73aecf6a | 20 records in `design/decisions/` (DR format of `DESIGN_DECISIONS.md`), carried in as nodes of kind `decision` (`[[revision.decision]]`, revision S11.2; `tests/test_decisions.py`). 12 carry the developer's choice (the 9 defaults of §5, FDIR staleness DR-fdir-002, the gain method DR-ctl-001, the run count DR-vv-001); 8 make none (DD2, DD1+38, DD39+15, DD12, DD59+60, DD14+19, DD22+23, DD52). The verdict method for A1 is DR-kpi-001's `method`. |
| S11.3 | TS11 | aac2aa03 (picked) | `tools/golden.py`, `results/golden.json` (55 scenarios now), CI job `golden`; bit-exact per toolchain fingerprint, per-metric tolerance otherwise. |
| S11.4 rig | TS4 | 7ac4b9dd, 5fd1bdb4 | fault kinds gyro_fail, fsw_reset, clock_step, clock_jitter (engine and twin); six scenarios (`design/revisions/S11.4/`, `[[revision.case]]`); three metric kinds (fine_law_fraction, date_error, schedule_replays); tick tests in C and Rust. |
| S11.4b | F3, F1, F10/G8 | afad7643, 30a4cafe, c35cc8d9 (picked) | integer-tick rate groups; monotonic bounded clock with GNSS date and no schedule replay (`fsw_param_clock_step_max_s`, `fsw/pseudocode/09_time.pc`); double only, `tests/test_precision_claim.py`. |
| S11.5 | F2/G3, F5, G4 | 13f05030..4b01552a (picked) | quiet gyro after 3 ticks (fault bit 10), filter attitude rate stands in (`att_rate`), damping beyond 2e-3 rad/s (`damp_excess`), B-dot rate for the gyroless detumble exit; `fault_st_blind`. fault_gyro_silent knowledge error 179.98 -> 0.020 deg; fault_st_blind rate 229 -> 0.096 deg/s. DR-fdir-002 amended to what flies (73aecf6a). |
| S11.6 | F4, G1, G2, G6, TS3 | b1138aff..b8727d79 | TS3 harness `engine/crates/adcs-sim/tests/loops.rs` (rise 0.62x -> 0.98x, settling 0.44x -> 0.99x of rw_bandwidth; GM 28.8 dB, PM 40.6 deg); gains x2 (DR-ctl-001); `st_plausible`, `st_head`, TRIAD sine floor 0.2 with `triad_cov`; per-sensor rejection counts; vector-scaled rotor allocation. Golden: G6 moved 175 rows in 44 scenarios, 4 fail -> pass, 1 pass -> fail (below). |
| S11.12 | P5 | cd64add3 | orbit step capped at 30 s. |
| S11.13 | O2, O12, O19, O20, O21 | 0b529c76..6026be2a (picked) | plugin refused by name; `fsw/devices/example_3u.toml` held to the header (`tools/device_map.py`); 5 % numeric SILS/OILS tolerance; CI `oils-deadline` (`results/oils_deadline_min.json`, smallest 32.585 ms); `--record`/`adcs replay`. |
| S11.15 | TS17 | b19f8a13 | four oracles replaced, each failing a mutation the old passed; 10 more weak tests listed in `docs/TEST_STRATEGY.md` TS17, left. |
| gate fixes | — | 496c4936, baa2874b, b1cfc6e6, b634b6e0, c32676f4, 0c7e313d | stale firmware refused or rebuilt (`tools/fsw_firmware.py`); evaluation rewritten (no closure verdict changed); campaign parity: bit-exact on the record's fingerprint, else verdicts equal and every metric within 5 % (`campaign_relative_tolerance` in `results/golden.json`). |

## The developer's decisions taken during S11 (to amend into the records, unsigned)

- **Campaign parity across machines** (refines DR-vv-003): the 4,516 stored campaign runs encode the libm of the machine
  that flew them (439 differ on this machine at cf397d1 already; switching glibc's FMA path moves which). Across
  fingerprints a campaign run passes on equal verdicts and every metric within 5 % relative (largest seen 1.41e-2, no
  verdict differs). **DR-vv-003's text still has to say so** (a design edit: rebuild the regression copy).
- **The magnetic PD's factor 2 (to do next).** G6's factor 2 is proven for the wheel PID by TS3, but the magnetic loop is
  not in the harness, and flown it is worse: `nadir_hold_ais` ape_los 10.67 -> 16.62 deg (already failing) and
  `nadir_hold_ais_css` `ape_los_last_orbit` 9.71 -> 10.76 deg against 10 (pass -> fail). The magnetic loop is periodic
  and its placement rests on an averaged field, so the stiffness the placement assumes is not what flies. **Decision:**
  keep the factor 2 on the wheel PID; take it off the magnetic PD (S7.13's `fsw_mtq_pd` as before) until a magnetic loop
  test (TS3 for coils, with S13's T1 margins) proves a placement; amend DR-ctl-001 with this and the numbers; rewrite the
  golden rows that move back. One switch: the factor on `fsw_mtq_pd`.

## Open, found on the way

- ctl's settling closure: gc_1 = 2.0 (stated as flown) lies outside ctl's `settling_time_2pct` range 0.3-1.5, whose formula
  is the underdamped one; the analysis closure passes at 2.2 s while the flown 2 % settling is 27.8 s against the case's
  20 s. ctl's node; record it for ctl, or state gc_0/gc_1 to the flown settling.
- `safe_mode_ais` recovery_to_ape 66.6 -> 128.4 s with F4/G1 (TRIAD refused at low sine): no verdict, but read it in the review.
- A stale engine binary (`engine/target/release/adcs` older than its sources or the blob) makes tests fail with misleading
  errors; the firmware has a guard (`tools/fsw_firmware.py`), the engine none yet. Give it the same guard.
- `adcs replay` of a QEMU recording on `obc-posix` differs in one frame (frame 26): not looked at.
- O19 is covered by unit tests only; `engine.py oils` has not been re-flown for the ledger.
- The full `verify_work.py --since cf397d1` has not run green in one piece since S11.4b: run it detached once the next
  steps land (the parity gate is an hour; use a watchdog, `setsid -f nohup`).
- The flight images were not resealed after S11.5-S11.6 (`flight-image` passed; confirm in S11.16).
- `tools/tokens.py checked`: no agent has been marked yet.

## Steps left, in order

S11.7 device models (M1, M2, M5 with DR-act-002, M9, M10) · S11.8 power and sizing (Y2, Y4, Y5, then Y1; DR-design-003)
· S11.9 A5 (DR-vv-002) · S11.10 verdicts (A7 windows, A1 on DR-kpi-001 with a stated `kpi_verdict_confidence`, A3, A2)
· S11.11 E6 and E1 · S11.14 O11 · S11.16 regenerate, gates, reseal, re-fly (the twin re-fly waits for the S7 re-fly's last
commit and the filled §4 of `docs/S7_REPORT.md`) · S11.17 counts and tables · S11.18 the phase review and
`docs/S11_REPORT.md`, ending with `python3 tools/tokens.py ledger --stage S11`.
