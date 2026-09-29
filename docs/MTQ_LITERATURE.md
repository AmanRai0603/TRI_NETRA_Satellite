# Magnetorquer-only control: the literature in the flight software

**Owner: Agastya.** Source: *Magnetorquer-Only Attitude Control: Sixteen Papers, One Mode Chain*.
The review (literature_review_v2.pptx, Agastya Research Collective, 19 Sep 2026) reads each paper for what it proves,
what it costs and what it leaves open. This page records what each paper became in TRI NETRa: a flight-software law, a
pipeline tool, or a documented decision not to implement it.

Every implemented law is:
- written once in the pseudocode (`fsw/pseudocode/05_control.md`, `06_detumble_sunspin.md`);
- coded in C (`fsw/src/adcs_ctl.c`) and Rust (`fsw-rs/src/ctl.rs`), bit for bit identical;
- registered in `matlab_sils/data/algorithms/` with its citation;
- given its gains on the ground (`engine/crates/adcs-sim/src/config.rs`).

The design loop flies every law of a slot as a candidate like any other (`docs/NODES.md`, node `matrix`). It tunes the
coils-only laws' gains (node `tune`) and certifies the coils-only nadir loop (node `certify`). The per-case ledgers
(`results/DESIGN_<case>.md`) and the V&V report carry the results.

## Paper by paper

| # | paper | review verdict | in TRI NETRa |
|---|---|---|---|
| P1 | Lovera & Astolfi 2004, Automatica | drop (needs exact inertia) | Implemented anyway as `mtq_lovera2004` (law 4): u = −(ε²k_p q_v + ε k_v J ω). It is the baseline P4 improves on. |
| P2 | de Ruiter 2011, Acta Astronautica | drop the law (axisymmetric proof), keep its outage argument | Implemented as `sunspin_deruiter2011` (ss_law 1): the momentum-error spin law on the Sun line, after the P11 spin-up. Whole-vector saturation (P5 rule) instead of componentwise. |
| P3 | Chasset et al. 2013, TANGO/PRISMA (flown) | reference; LQR gains unpublished | Implemented as `mtq_tango2013` (law 8). P is frozen at the solution of the averaged Riccati equation, so the gain varies only through the measured field. Q, R are ours: inverse LQR to the common bandwidth, with an isotropic field average. |
| P4 | Celani 2015, Acta Astronautica | implement (group B) | `mtq_celani2015` (law 5): u = −(ε²k₁ q_v + ε k₂ ω), no inertia in the law. It is also the fallback of P16 when the reference does not rotate. |
| P5 | He et al. 2023, Adv. Space Res. | implement (group A, 2nd) | Already flown in the chain `sunspin_l1l2`: the Sun-pointing spin after the P11 spin-up, with an R_z rate term and whole-vector saturation. |
| P6 | Celani 2016, JGCD | tool: duty-cycle band | The coil cycle (1 s, 0.2 s measure window) sits inside every law and inside the certificate (duty 0.8). The Δm*–Δa band itself is not computed. |
| P7 | Bruni & Celani 2017, JOTA | tool: min–max gains | Node `tune`: every coils-only law of a failing option is flown on a gain grid and extra seeds; the worst seed decides. |
| P8 | Celani 2026, JGCD | tool (Floquet); law dropped | Both. Node `certify` gives the Floquet multipliers of the coils-only nadir loop for every law. The boresight law is implemented too: `mtq_celani2026` (law 7) for the payload axis, and `sun_boresight_celani2026` (ss_law 2) for the power face on the Sun with no spin. |
| P9 | Roldugin et al. 2023/24 | spec + switch band | The rate band of the spin-up → Sun-spin switch (existing). The spin rate is a tuning variable of node `tune` (wobble grows with it). The balancing spec is a structures item. |
| P10 | Cubas & de Ruiter 2020 | drop (abstract only) | Not implemented: the law is not in the accessible text. |
| P11 | Porras-Hermoso et al. 2024, UPMSat-2 (flown) | implement (group A, 1st) | Already flown: the spin-up m = −k(Ḃ + ω_d × B) (`genbdot`), the first stage of every Sun chain. |
| P12 | Yang et al. 2025, IEEE TAES | hold (paywalled) | Not implemented: the law is not in the accessible text. It becomes a candidate once the full text is read. |
| P13 | Bahu & Modenini 2021, CEAS | drop for now | Not implemented (paywalled hybrid H∞). |
| P14 | Sofyalı et al. 2018, AST | drop for now | Not implemented (paywalled sliding mode). |
| P15 | Yang et al. 2025, CEP | drop | Not implemented (paywalled, needs an onboard QP). |
| P16 | Avanzini, de Angelis & Giulietti 2021, AST | implement (group C) | `mtq_avanzini2021` (law 6), the two-timescale nadir law. The pitch axis is spun at the orbit rate and aligned with the orbit normal, and the pitch error is removed through η(θ). The pitch error is taken to first order from the error quaternion. |

## Slots after this change

| slot | candidates (the loop picks per case and option) |
|---|---|
| `sun_acquisition` (coils only) | `sunspin_l1l2` (P11 → P5), `sunspin_l1l2_e2`, `sunspin_damped`, `sunspin_deruiter2011` (P11 → P2), `sun_boresight_celani2026` (P8) |
| `mtq_pointing` (Sun referencing and nadir, coils only) | `mtq_pd`, `mtq_lqr`, `mtq_smc`, `mtq_rate_damp`, `mtq_lovera2004` (P1), `mtq_celani2015` (P4), `mtq_avanzini2021` (P16), `mtq_celani2026` (P8), `mtq_tango2013` (P3) |

## Gains

All magnetic laws are compared at one bandwidth, ω_n and ζ (fsw `mtq_wn`, `mtq_zeta`), at torque level after the
normalised projection. Only their structure differs. The papers' own gains belong to their own spacecraft (for
example 50–100 A·m² and J in the thousands of kg·m²), so they are not transferable. Node `tune` then scales each law's
proportional and rate gains by 0.25, 1 or 4, and the Sun laws' spin rate and gain the same way, and keeps the best
worst case.

## Results (both cases, `results/DESIGN_<case>.md`, V&V report section 7)

- **Nadir pointing, coils only: Celani 2026 (P8) is the one law that holds nadir.** With the tuned gains
  (proportional ×4, rate ×0.25) its worst seed out of four is 3.73° on ais_3u, against the 10° AIS requirement, so the
  AIS coils-only nadir option is now feasible. On ais_img_3u it reaches 3.4°, not near the 0.01° imaging requirement.
  - The next best on ais_3u are the baseline SMC (11.9°), TANGO (12.9°) and Lovera & Astolfi (12.9°); Celani 2015 gives
    17.8° and Avanzini 54°.
  - Its certificate agrees: at the dispatched gains the largest Floquet multiplier is 0.017 on ais_3u and 0.04 on
    ais_img_3u.
  - Of the others, only the baseline PD, Celani 2015 and TANGO are certified at nominal gains on ais_3u, and none on
    ais_img_3u.
- **Sun acquisition, coils only: no law meets the cases' line within the 1.5-orbit mode test.** That line is the power
  face within 20° inside 95 min and held at the 95th percentile.
  - P11 → P5 (He et al.) at a 2 °/s spin reaches the Sun in 56 min on ais_3u, but not on every seed.
  - Celani 2026's boresight reaches it in 32 min on its best seed, without a spin.
  - Lowering the spin rate helped every spin law, as Roldugin's wobble analysis predicts.
- **Sun referencing (three-axis on the Sun), coils only: none holds 5°.** The best is Celani 2026 (27.6° imaging,
  84.7° AIS).
- **Coils-only mission (dispatched with the best laws):** detumble, then Sun spin, then nadir by schedule at
  2 orbits, with the hand-over below. The final figures of each case are in `results/DESIGN_<case>.md` and in the
  V&V report.

## The Sun-spin → nadir hand-over (coils only)

The review leaves this transition open for P16. Before this change the coils-only mission reached only 33.7° in its
last half orbit. The mode test, which starts 10° off nadir, gives 3.7°. There were three causes, found on the
ais_3u trace:

1. **The body arrives spinning at about 4 °/s,** and every pointing law was tuned from a 10° error at the orbit
   rate. The magnetic states now despin first. When the rate error ω_e exceeds 1 °/s, the coils run the B-dot law on
   the rate error, m = (k/|B|)(ω_e × b̂). The pointing law takes over once |ω_e| < 0.5 °/s has held 60 s
   (`fsw/pseudocode/05_control.md`; C, Rust and the config blob fields `ho_in_dps`, `ho_out_dps`, `ho_hold_s`).
2. **The despin gain matters (P11, and Avanzini & Giulietti 2012).**
   - With the spin-up's high gain, the coils remove the rate across B at once. The rate along the field line is
     only carried round as the field turns: 3.9 → 0.3 °/s in one orbit, and the despin never handed over.
   - With the detumble's optimal gain, k = 3·2n(1 + sin i)J_min, the rate falls to 0.5 °/s in 0.4 orbit.
   - A 0.2 °/s exit is below what the field's own rotation leaves, so the exit is 0.5 °/s.
3. **The magnetic capture from an arbitrary attitude takes about 1.5 orbits,** because the coils have no authority
   along B. The mission ended one orbit after the nadir command, so its last half orbit still held the transient.
   Node `dispatch` now gives a coils-only nadir three orbits after the command (`nadir_orbits.coils_only`). The
   rotor families keep one.

On the ais_3u coils-only mission (Celani 2026 with the tuned gains, four seeds, five orbits) nadir holds 2.1–6.0°
p99.73, against the 10° AIS requirement.

| law | no despin (4 seeds) | despin, hand-over at 0.5 °/s |
|---|---|---|
| Celani 2026, tuned | 2.0–3.9° | 2.1–6.0° |
| PD | 22–40° | 7.3–14.3° |
| SMC | 7.6–57.6° | 4.2–12.7° |

The boresight law already captures from the spin by itself, since its rotation about the boresight is free. The
despin is what makes the three-axis laws usable after the spin, so it stays on for every law.

## Why Sun referencing is worse than nadir with coils only

The trace shows the coils are not short of dipole: they peak at 0.044 A·m² of their rating, and the residual dipole
is cancelled by `m_res_est`. The difference is the gravity gradient.

- **At nadir, the gravity gradient helps.** The long, minimum-inertia +X axis sits at its equilibrium, so the
  gradient is a restoring stiffness of order 3n²ΔJ. The residual torque is about 4·10⁻⁹ N·m, and the loop holds
  1–2°.
- **Under Sun referencing, the gravity gradient is a disturbance.** The attitude is inertial, so the gradient
  becomes a periodic forcing at twice the orbit rate, about 4.5·10⁻⁸ N·m. A magnetic torque is always
  perpendicular to B, so the component of that forcing along B cannot be rejected at that instant, only later, as
  the field turns. The error swings by tens of degrees.
- **The literature agrees.** TANGO/PRISMA (P3) flew 16° with coils only, and Celani 2026 (P8) reports 24°. The review
  recommends a Sun spin, not three-axis Sun referencing, as the coils-only power attitude.
- **What was added:** a gravity-gradient feed-forward in the Sun state (`mtq_gg_ff`, bit 0). It cancels the modelled
  3μ/|r|⁵ (r_b × J r_b) inside the law's torque, so only the along-B part is left to the loop. On ais_3u with Celani
  2026 it takes the Sun-pointing error from 80.6° to 66.8°. It is still not the 5° line, and the physics above says
  it will not be.
- **At nadir the feed-forward stays off,** because cancelling the restoring gradient would remove the help it gives.
- **The coils-only power attitude remains the Sun spin (P11 → P5).**

## What stays open

- The Sun-acquisition mode test lasts 1.5 orbits. The spin chains (P11 → P5, P11 → P2) finish their spin-up only in
  the full mission (about 1.5 to 2 h after detumble on the coarse case), so the mode test under-rates them against the
  boresight law.
- P12 to P15 are paywalled; their laws cannot be written from the abstracts.
- The hand-over thresholds (1 and 0.5 °/s, 60 s) are fixed, not tuned by node `tune`. The mode tests start at 10°,
  so the tuned gains are chosen for holding, not for the capture after the spin.
- The certificate is local and numerical, on an aligned-dipole field model, as in the papers.
- On the Cortex-M4F firmware (QEMU), the design loop's coils-only nadir mode tests differ from the host in the last
  bits (newlib's double-precision math functions). This is the same for the pre-existing PD law. The standard
  mission scenario stays bit-identical on all five paths (`results/VIRTUAL_OBC.md`). Soft OILS compares verdicts,
  not bits.
