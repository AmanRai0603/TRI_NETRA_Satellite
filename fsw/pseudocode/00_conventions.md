# 00 — Conventions every implementation follows

**Owner: Agastya.** Copyright (c) 2026 Agastya. All rights reserved.

These pseudocode files are the **contract** for the flight software. Three implementations
follow them line for line, and parity jobs compare them:

| implementation | where | language rules |
|---|---|---|
| embedded C (flight) | `fsw/src/*.c`, `fsw/include/*.h` | C99, no dynamic memory, no recursion, no `time()`/`rand()`, `-ffp-contract=off` |
| Rust (flight) | `fsw-rs/src/*.rs` | `#![no_std]`, no `alloc`, no panics on the step path |
| MATLAB twin | `matlab_sils/+asils/+fsw/*.m` | the prototype; parity measured, not assumed |

A change goes to the pseudocode first, then to all three in the same pull request (SPEC §10.8.7).

## Numbers

- `real` is IEEE-754 double, and the build is double only. There is no single-precision build: `ADCS_REAL_FLOAT` is
  refused at compile time, and the Rust crate has no `f32` feature (DR-fsw-002). A single-precision build waits for the
  board's choice of FPU (H1) and a numerics review; `tests/test_precision_claim.py` fails if a document claims one while
  `gcc -DADCS_REAL_FLOAT -fsyntax-only` does not build clean.
- Angles are in rad, rates in rad/s, time in s (tick time `t = tick · dt`, exact), field in T,
  dipole in A m², torque in N m, momentum in N m s.
- A guarded division `x / max(|y|, eps)` is always written with its `eps`, and it is the same in
  every implementation.

## Frames and attitude

- ECI means GCRF/J2000. ECEF is reached onboard by the IAU 2006 precession, the IAU 2000B nutation, the
  equation of the equinoxes, UT1 = UTC + dUT1 and the polar motion (dUT1, xp, yp are flight parameters set by
  telecommand; the truth uses the POP frame chain, IAU 2006/2000A; the difference is an onboard-model error,
  measured: under 0.003 arcsec).
- Body axes follow the product. For a 3U the long axis is X_B.
- **Quaternion `q = [x, y, z, w]` (scalar last)**, Hamilton product, representing the attitude of
  the body relative to ECI. `dcm(q)` is the *passive* ECI → body DCM, and
  `dcm(q ⊗ p) = dcm(p) · dcm(q)`. The HAL peek structure (`adcs_fsw_state_t`) is scalar first by
  the spec, so conversion happens only there.
- `fromrotvec(θ) = [sin(|θ|/2) θ/|θ|, cos(|θ|/2)]`; for `|θ| < 1e-12` it is `[θ/2, 1]`, normalised.
- The error quaternion is `q_e = conj(q_ref) ⊗ q`, sign-fixed so that `q_e.w ≥ 0`.

## Tick

One call `step(now)` per FSW tick of period `dt` (the scenario's `dt_s`). Inside a tick the order
is fixed (10_mode_manager.md). The flight software keeps all state in one statically allocated
struct. `init` resets every field, so two runs in one process give identical bytes.

## Naming

Pseudocode names match the C functions (`adcs_<module>_<name>`), the Rust functions
(`<module>::<name>`) and the MATLAB twin (`asils.fsw.<name>`). The mapping is in
`fsw/twin_map.toml`.
