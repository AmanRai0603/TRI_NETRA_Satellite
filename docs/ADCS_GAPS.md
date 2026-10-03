# TRI-NETRA: ADCS gaps and where to study

> **Answer first.** The known ADCS technical gaps that the first release (1.0.0) ships with, each with the evidence
> found so far and what has to be studied or decided to close it. None of these gates the 1.0.0
> release (that is [`RELEASE_PLAN.md`](RELEASE_PLAN.md)); they are the work after it. The release
> notes list this register as the release's known gaps.
>
> **Kind:** register · **For:** the owner (Agastya) and whoever studies the ADCS · **Status:** open
> (2 Oct 2026)

Each row: what is known (the evidence, with where it was found), and what to study or decide.
**Owner** marks a decision only the owner can make; **study** marks technical work. Numbers in
brackets point to the evidence in [`UPGRADE_PLAN.md`](UPGRADE_PLAN.md).

## D. Design (the design loop and the sizing)

| id | gap | what is known | study or decide |
|---|---|---|---|
| D1 | Unconfirmed case values | Most case values are marked UNCONFIRMED (plan references, not stated by the owner): among them `req.pavg` 2.0 W, `req.ppk` 4.0 W, `req.ake`, `req.settle`, `req.detumble`, `mass.*`, `surface.*`, `power.*` | **Owner**: confirm or replace each; `req.pavg` first (D4) |
| D2 | Pointing error budget incomplete | No product states its payload alignment, no case its thermal distortion; the room left for both is 0.0073–0.0080° on the wheel and fluid-ring fine holds, none on `target_img`, not computable on CMG/VSCMG (no stated imbalance); the RPE side is not done (`req.rpe` blank) [B2.2] | **Owner**: alignment and thermal allocations; **study**: RPE budget |
| D3 | Sensor trade and star-tracker outages | Every product gets the same sensor suite; `target_img` misses AKE (0.0114° vs 0.005°); in `mc_fine_img` 51 of 55 failures are both heads out together for about 8 min in sunlight (Sun in head 2's cone, head 1 out for a reason not yet confirmed, the Moon suspected), failures depend on the starting argument of latitude only; the imaging design now carries one head [B2.5, B3.3] | **Study**: confirm the second head's cause; head placement along the orbit, a third head, or a gyro coast budget |
| D4 | Imaging power margin | The selected `ais_img_3u` design (1.79 kg, spare ring) passes, but a variant missed `req.pavg` by 5 % with one coil lost (2.10 W on 2.0 W) and a lighter pump failed mean power in 42 % of dispersed runs; steady budget 3.5 W [c88370c] | **Owner**: confirm `req.pavg`; **study**: pump design margin |
| D5 | Spare ring mass | The four-ring imaging ADCS is 1.79 kg on the owner's 1.85 kg budget (ceiling 2.0); its mass levers are used up (one head, fluid-loop momentum ×0.5625) | **Study**: a lighter ring or pump design |
| D6 | FDIR beyond the rings | Wheels: the windowed FDIR is off for wheels (friction drifts a healthy wheel off its command; a friction-aware test is owed). Thrusters: no FDIR, a stuck valve drives APE to 2.75° (`fault_valve_rcs`). Benchmarks are not flown under faults; `mag_fail` left out of the fault set by design [B2.6, B3.6] | **Study**: wheel friction-aware window, thruster FDIR |
| D7 | Lifetime and momentum dumping | Both cases leave `req.dump` blank (a quarter-orbit default applies); the solar cycle is bounded, not followed over `mission.life` (blank) [B2.3] | **Owner**: dump interval and life; **study**: date-by-date sweep |
| D8 | Thermal and data budgets | Not modelled; `resources.nif`, `resources.vbus`, `mission.duty`, `mass.cm` are refused by name [B2.1, B2.4] | **Study**: each refused key gets a model |
| D9 | Beyond 3U | The demand survey still flies the 3U AIS product (its torques do not depend on the product); products of inertia drawn, not stated [B2.7] | **Study**: survey product from the case |
| D10 | Sizing constants and ranking | Seven sizing constants have no stated source (`docs/SIZING_LAWS.md`); SPEC §8.6 ranks by worst margin, the code by least mass [B2.8] | **Owner**: confirm constants and the ranking rule |
| D11 | Spec package | About 90 of 243 system rows have code, none linked by id [B2.9] | **Study**: spec → code → test matrix |
| D12 | RCS detumble floor | `detumble_rcs` stalls at 0.45–0.7°/s: the 5 ms minimum impulse bit against a 20 s damping time gives a 0.4°/s floor [B3.6] | **Study**: hand over to coil B-dot below 1°/s, or size the damping from the impulse bit |
| D13 | Coils-only modes on AIS | `safe_mode_ais` and `nadir_hold_ais` miss the 10° APE; `sun_mtq_ais` does not hold the Sun (p95 87°) [B3.6] | **Study**: coils-only laws on this product, or accept |
| D14 | Physics relations awaiting their bundles | `spec/physics` (pseudocode v2, P2): the dipole's strength and tilt use IGRF-13 at 2020.0 from the repository (the rows cite IGRF-14 at 2025.0); `env::density_at` is the static exponential atmosphere, with no solar activity (the row cites NRLMSIS 2.0); `env::sun_distance_au` and `orbit::beta_angle` use Vallado's low-precision Sun; `rw::cyclic_momentum_quarter_orbit` leaves out SMAD's 0.707 factor; `orbit::eclipse_fraction` is the cylindrical shadow. 7 of the 39 relations have a sourced test vector (IDMAS v2); the rest are checked for units, translation and range only | **Study**: the igrf14 and nrlmsis2 bundles; sourced test vectors for the other 32 (an engineer, from a page) |

## S. SILS fidelity and statistics

| id | gap | what is known | study or decide |
|---|---|---|---|
| S1 | Monte Carlo claims | Four campaigns fly 1109 runs each; none shows its 99.73 % claim: `mc_fine_img` 95.0 %, `mc_agile_rw_rcs` 91.5 %, `mc_slew_cmg` 91.2 %, `mc_slew_img` 85.8 % (see D3) [B3.3] | **Study**: the failure causes per campaign |
| S2 | Device fidelity left | Coil hysteresis and eddy currents (air-core today), Dahl pre-sliding, outgassing, self-shadowing; the VSCMG rotor keeps the wheel model; no latency compensation in the flight software [B3.5] | **Study**: each, against a stated part |
| S3 | Eclipse transitions | Every orbit-long scenario crosses eclipse; none judges the transition [B3.6] | **Study**: a scenario that judges it |
| S4 | Engine vs twin | Sun-spin hemisphere manoeuvre (3 of 12 seeds miss 95 min); SMC tuning (0.0126–0.0292° vs 0.01°); RCS thrust-scale calibration tried and not shipped (estimates 0.52–0.81 vs ~1.0) [B3.7] | **Study**: each at its traced cause |
| S5 | Twin-only chains | The image and Sun-sensor chains are ported; no shipped part carries their values; the image model costs about 30 ms a frame per head; the earth-sensor chain is not wired into the twin's device [B3.8] | **Study**: characterise a head |
| S6 | Filter consistency | No MEKF consistency (NEES) test; the star-tracker innovation gate was tried and left out (it rejected the corrections after a slew) [B1.6, B8.3] | **Study**: NEES test, then the gate |

## V. Validation and evidence

| id | gap | what is known | study or decide |
|---|---|---|---|
| V1 | Propagator outside validation | Time and frames against ERFA vectors ✅; IGRF-13 against pyIGRF ✅; owed: a real satellite's precise orbit, the atmosphere models' published outputs [B8.2] | **Study**: obtain both references |
| V2 | Algorithms against references | LQR, B-dot, QUEST, TRIAD, MEKF equations, guidance ✅; owed: one published magnetorquer case [B8.3] | **Study** |
| V3 | Catalogue | Datasheet numbers not traced to their pages; 14 synthetic parts [B8.4] | **Owner**: choose real parts |
| V4 | Algorithms confirmed | All 31 algorithms are UNCONFIRMED [B8.6] | **Owner**: confirm |
| V5 | IGRF-14 | The 2027 epoch is past IGRF-13's 2025; the field is extrapolated [B8.6] | **Owner**: adopt IGRF-14 |

## H. Hardware (OILS with a real OBC, HILS)

| id | gap | what is known | study or decide |
|---|---|---|---|
| H1 | Reference board | Only QEMU, POSIX and the link server exist; no board target, real-bus HAL, HAL v2 (watchdog, storage, power switches, bus reset, timestamps), TM/TC, time (PPS/drift) [B5] | **Owner**: choose the board (STM32F4 class or the flight OBC) |
| H2 | Soft-OILS calibration | The CPI is assumed; measured execution times replace it once a board runs [B4.4] | after H1 |
| H3 | Firmware leftovers | The watchdog kick needs a HAL call; one libm for both firmware builds (C = Rust is bit-identical today) [B1.5, B1.8] | after H1 |
| H4 | HILS | Documentation only: interface emulation unit, stimulus drivers (cage, Sun simulator, air bearing), one device at a time [B6] | **Owner**: list the equipment that exists |
| H5 | Safe-mode policy | As built: coils-only detumble after a magnetometer or gyro is silent 60 s; invalid attitude is not a trigger [B1.5] | **Owner**: confirm |

## How a gap is closed

A gap closes with the same rules as the software: the change in the engine, the twin and both flight
softwares where it applies; the runs it changes re-flown; `check_all` and CI green; the row moved
here to a **Closed** section with the commit that closed it.

## Closed

| id | gap | closed by |
|---|---|---|
| — | `ais_img_3u` had no design surviving a single fault (three rings, no spare; the flight software never isolated a dead ring in fine pointing) | spare ring (`redundancy` step) and the windowed fluid-loop FDIR: `505a43d`, `c88370c` |
