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

## What stays open

- The Sun-acquisition mode test lasts 1.5 orbits. The spin chains (P11 → P5, P11 → P2) finish their spin-up only in
  the full mission (about 1.5 to 2 h after detumble on the coarse case), so the mode test under-rates them against the
  boresight law.
- P12 to P15 are paywalled; their laws cannot be written from the abstracts.
- The certificate is local and numerical, on an aligned-dipole field model, as in the papers.
- On the Cortex-M4F firmware (QEMU), the design loop's coils-only nadir mode tests differ from the host in the last
  bits (newlib's double-precision math functions). This is the same for the pre-existing PD law. The standard
  mission scenario stays bit-identical on all five paths (`results/VIRTUAL_OBC.md`). Soft OILS compares verdicts,
  not bits.
