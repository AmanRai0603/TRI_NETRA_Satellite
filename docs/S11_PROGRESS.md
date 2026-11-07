# S11 progress: where the corrections stand (hand-over record)

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.

The working record of S11 (`docs/PLAN_2_0.md`, S11 Corrections (U0); the steps are those of `docs/S11_INVENTORY.md` §6).
Written at a hand-over between sessions; the next session keeps it current and `docs/S11_REPORT.md` is written from it at
the phase's end. Everything here is the developer's and unsigned.

## Steps done (on `claude/brave-dijkstra-rshrb7`, last commit at this writing: S11.14)

| step | items | commits | what it shows |
|---|---|---|---|
| S11.1, S11.2 | the 9 owner questions, the 15 decisions | cf397d1, dc157c50, 73aecf6a | 20 records in `design/decisions/` (DR format of `DESIGN_DECISIONS.md`), carried in as nodes of kind `decision` (`[[revision.decision]]`, revision S11.2; `tests/test_decisions.py`). 12 carry the developer's choice (the 9 defaults of §5, FDIR staleness DR-fdir-002, the gain method DR-ctl-001, the run count DR-vv-001); 8 make none (DD2, DD1+38, DD39+15, DD12, DD59+60, DD14+19, DD22+23, DD52). The verdict method for A1 is DR-kpi-001's `method`. |
| S11.3 | TS11 | aac2aa03 (picked) | `tools/golden.py`, `results/golden.json` (55 scenarios now), CI job `golden`; bit-exact per toolchain fingerprint, per-metric tolerance otherwise. |
| S11.4 rig | TS4 | 7ac4b9dd, 5fd1bdb4 | fault kinds gyro_fail, fsw_reset, clock_step, clock_jitter (engine and twin); six scenarios (`design/revisions/S11.4/`, `[[revision.case]]`); three metric kinds (fine_law_fraction, date_error, schedule_replays); tick tests in C and Rust. |
| S11.4b | F3, F1, F10/G8 | afad7643, 30a4cafe, c35cc8d9 (picked) | integer-tick rate groups; monotonic bounded clock with GNSS date and no schedule replay (`fsw_param_clock_step_max_s`, `fsw/pseudocode/09_time.pc`); double only, `tests/test_precision_claim.py`. |
| S11.5 | F2/G3, F5, G4 | 13f05030..4b01552a (picked) | quiet gyro after 3 ticks (fault bit 10), filter attitude rate stands in (`att_rate`), damping beyond 2e-3 rad/s (`damp_excess`), B-dot rate for the gyroless detumble exit; `fault_st_blind`. fault_gyro_silent knowledge error 179.98 -> 0.020 deg; fault_st_blind rate 229 -> 0.096 deg/s. DR-fdir-002 amended to what flies (73aecf6a). |
| S11.6 | F4, G1, G2, G6, TS3 | b1138aff..b8727d79 | TS3 harness `engine/crates/adcs-sim/tests/loops.rs` (rise 0.62x -> 0.98x, settling 0.44x -> 0.99x of rw_bandwidth; GM 28.8 dB, PM 40.6 deg); gains x2 (DR-ctl-001); `st_plausible`, `st_head`, TRIAD sine floor 0.2 with `triad_cov`; per-sensor rejection counts; vector-scaled rotor allocation. Golden: G6 moved 175 rows in 44 scenarios, 4 fail -> pass, 1 pass -> fail (below). |
| decisions | DR-ctl-001, DR-vv-003 | 5d6aecb5, a65a1db5 | factor 2 kept on the wheel PID, off the magnetic PD behind one switch (`kp_factor` in `fsw_mtq_pd`, design/revisions/S11.6/fswmtq.pc); golden: nadir_hold_ais_css ape_los_last_orbit 10.94 -> 9.71 deg (fail -> pass), nadir_hold_ais 17.06 -> 10.61 (still fails), safe_mode_ais recovery_to_ape 128.4 -> 85.9 s. DR-vv-003 carries the campaign rule across machines. |
| S11.7 | M1, M5, M2, M9, M10 | e0da8c19..ea446034 | failed ring spins down with T_sd; failed CMG spins down on the wheel's friction ratios (DR-act-002) and its gimbal stops; wheel copper loss tau^2 w_nl/tau_stall (17 golden values, no verdict); tracker latency over 0.5 s refused by name; GNSS rate_Hz in the schedule. |
| S11.8 | Y2, Y4, Y5, Y1 | 40f97886..6a715573 | CMG cluster one unit on its own figures, a stated zero is not stated; h_req = k_h (h_dist + h_slew), k_h = 2 floor refused by name, achieved margin in sizing.json, no panic; survey in the inertial frame; `sens_power` (sensorpower.pc) a fourth power channel `P_sens_W` and gb_2 the peak (duty: the tracker in modes 1-4 and 10 only, DR-design-003). Golden: 30 verdicts pass -> fail (power_mean 23, power_peak 4, power_margin 3), none the other way. Default selection: wheel CW0017 -> CW0057, h_req 1.71e-3 -> 2.81e-3 N m s; stored pipeline selection.json not regenerated (S11.16). |
| S11.9 | A5 | 347e677c | `inertia_realisable`, `realisable_scale` (dyn_truth_plant, S11.9): a Monte Carlo draw that is not a body is drawn again from its own stream, an edge corner is moved to the nearest body (AIS 10 % corner 1.0867/0.9133/1.0867), a stated one refused by name; DR-vv-002. 179 of 4,490 stored draws change (173 MC, all 6 edge corners): re-flown in S11.16. No golden row. |
| S11.10 | A7, A1, A3, A2 | 785619f4, 1e1d533b, bcefe05a, 7bbd8c35, 3da67a66, 1119317e, 25fb884e, 751eaeda | windows: fine holds fly 2 orbits judged `last_orbit`, slews from 280 s, `eclipse_s:` window (16 golden rows, no verdict). One verdict `kpi_confidence_verdict` (Clopper-Pearson at the stated `kpi_verdict_confidence` = 0.95, NaN a failure, infeasible selection a fail; DR-kpi-001), called by evaluate and trace (`tools/verdict.py`). EVALUATION: ais_img_3u APE, AKE pass -> not demonstrated (R >= 0.78 at 95 %); ais_3u APE, AKE, detumble, power pass -> fail (selection closest, not feasible). Cause table (scenario, metric) or "untraced"; the ledger sentence from the run counts; `engine.py twosample` with the design's `kpi_mann_whitney` (not flown: S11.16). |
| S11.11 | E1, E6 | 01c10e1c, 21824211 | parity tool: max angle of B, Sun, nadir directions. m3_3 the DTM2020 bundle (F10.7 70/130/250, Kp 2; high for worst, unsigned), spec text off NRLMSIS; gd_1, gd_4 move, no closure or selection (the survey flies the fast orbit's table). |
| S11.12 | P5 | cd64add3 | orbit step capped at 30 s. |
| S11.13 | O2, O12, O19, O20, O21 | 0b529c76..6026be2a (picked) | plugin refused by name; `fsw/devices/example_3u.toml` held to the header (`tools/device_map.py`); 5 % numeric SILS/OILS tolerance; CI `oils-deadline` (`results/oils_deadline_min.json`, smallest 32.585 ms); `--record`/`adcs replay`. |
| S11.14 | O11 | the three S11.14 commits after 21824211 | bus faults at the emulated bus (`bus_nack`, `bus_crc`, `bus_stuck`, `bus_delay`, `bus_brownout`, `bus_can_error`, at a device of `fsw/devices/example_3u.toml`, each device listing its `bus_faults`): engine `busfault.rs`, twin `+asils/+faults/bus_*.m`, the TICK's present bits 4 (PWM write) and 5 (CAN send); kpi's `fault_raised`; the fault word appended to `adcs_fsw_debug` (DEBUG_LEN 42); 25 cases (`design/revisions/S11.14/`, golden 80 scenarios, existing rows unmoved); HAL tests in C (19 skipped as findings) and Rust (19 ignored, 6 pass). Flags raised: magnetometer NACK and brownout, gyro NACK/brownout/stuck, momentum devices all four. **Not raised (findings, failing verdicts by case):** magnetometer stuck; gyro delay 0.06 s (deadline margin -23.7 ms on soft OILS); Sun sensor, star tracker, GNSS, coils every fault (a stuck tracker frame: knowledge error 177 deg, no flag); TS4's "PWM write error: fault raised" does not hold (the driver ignores the bus status; the design's driver text states no check). Earth sensor: no product carries one. |
| S11.15 | TS17 | b19f8a13 | four oracles replaced, each failing a mutation the old passed; 10 more weak tests listed in `docs/TEST_STRATEGY.md` TS17, left. |
| gate fixes | — | 496c4936, baa2874b, b1cfc6e6, b634b6e0, c32676f4, 0c7e313d | stale firmware refused or rebuilt (`tools/fsw_firmware.py`); evaluation rewritten (no closure verdict changed); campaign parity: bit-exact on the record's fingerprint, else verdicts equal and every metric within 5 % (`campaign_relative_tolerance` in `results/golden.json`). |

## The developer's decisions taken during S11

Both carried into their records (5d6aecb5, a65a1db5): DR-ctl-001 (the magnetic PD without the factor 2 until a coil loop test
proves a placement) and DR-vv-003 (campaign parity across machines). S11.7 amended DR-act-002, S11.8 DR-design-003 (k_h floor,
the sensors' duty by mode), unsigned.

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

- The pipeline loop's fluid-loop mass lever (x0.75 from scale 1) is now refused by the sizing's k_h = 2 floor, which stops the
  loop: the momentum parts need their own lower scale bound, apart from the coils'. It lives in `looprules.pc` (sealed vectors);
  DR-design-003 notes it. Settle before the S11.16 pipeline re-fly.
- S11.7 and S11.8's new tests were written with the fix and not watched failing on the old tree (the generated code was rebuilt
  in place); each asserts a value the old code could not give.
- Step 1's single python-tests failure was a flaky browser click (a `tn-dialog` over the button in group.test.mjs): the suite
  passed in full on a65a1db (484, 5 skipped).

- The sizing survey's "high" activity (F10.7 = 250) is inert on the fast orbit (its density is the static table times
  `density_scale`); making the survey's high truly high (fly it on the POP orbit or DTM2020) belongs with P6 (S12).
- results/ENGINE_CAMPAIGNS.md still carries the old fixed sentence until `engine.py campaign-ledger` is run (S11.16/S11.17).

- The drivers' missing fault flags of S11.14 (Sun sensor, star tracker, GNSS, coils, magnetometer stuck, gyro delay) are
  findings for the owner's list (S20 or the FDIR phase): the design's driver text states no bus-status check, so nothing
  was changed in flight behaviour.
- `translators` (check_all) fails on one vector, `env::srp::srp_force`, relative 2.79e-11 against 1e-12, in Rust and C
  alike; present before S11.14 (measured at the S11.11 tip). Not looked at.

## Steps left, in order

S11.16 regenerate, gates, reseal, re-fly (the twin re-fly waits for the S7 re-fly's last
commit and the filled §4 of `docs/S7_REPORT.md`) · S11.17 counts and tables · S11.18 the phase review and
`docs/S11_REPORT.md`, ending with `python3 tools/tokens.py ledger --stage S11`.
