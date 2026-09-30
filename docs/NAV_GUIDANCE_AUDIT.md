# Navigation and guidance audit

**Owner: Agastya.** This page records an audit of the flight software's navigation and guidance assumptions: C
(`fsw/src`), Rust (`fsw-rs/src`), the pseudocode (`fsw/pseudocode`), the engine's truth side, and the MATLAB twin.
It gives each finding, what was changed, and what stays open.

Every change is in C and Rust together. The two stay bit-identical on every dispatched mission. The
pseudocode was updated to match (02, 03, 04, 05, 09).

## Navigation

| # | finding | severity | fix |
|---|---|---|---|
| N1 | **The magnetometer update never ran while the coils were working.** It was gated on the first tick of the coil cycle and on "the previous tick's dipole was zero". The coils hold their dipole to the end of the cycle, so that tick is never clean. Without a star tracker, in NADIR_MTQ and SUN_MTQ, the filter lived on Sun (and Earth-sensor) updates alone. Rotation about the Sun line and its gyro bias were unobservable. | bug | The field update runs on the first clean tick of each cycle (latched once per cycle). On the coils-only ais_3u mission, knowledge (AKE, p99.73) went from 0.25° to 0.13°. On the MATLAB twin's nadir hold it went from 3.9° to 1.2°. |
| N2 | **The attitude-valid flag was never cleared.** DETUMBLE, DETUMBLE_RCS, SPINUP and SUN_SPIN skip the filter, so the attitude froze with a small covariance. A later pointing state flew that stale attitude, which could be up to 180° off. | bug | Entering those states drops the estimate, so TRIAD or the star tracker re-initialises it. |
| N3 | **No innovation gating, and a fixed Sun σ of 0.012 rad even with coarse cells.** Coarse cells carry up to about 8° of albedo error, which is 12σ at that value. | risky | Vector updates are gated at χ² > 16.27 (3 dof, 99.9 %). After 30 consecutive gated updates the estimate is dropped and re-initialised. The Sun σ comes from the fitted device: fine sensor hypot(accuracy, bias) ≥ 0.005 rad; coarse cells max(0.1, 0.5·albedo). |
| N4 | **The magnetometer σ ignored its bias and the field strength.** About 0.9 µT of error is 1–3° of direction at 18–50 µT. | risky | σ_B = sqrt(σ_dir² + (err/\|B\|)²). σ_dir = hypot(misalignment, scale factor) ≥ 0.01 rad; err = sqrt(bias² + 3σ_bias² + 3σ_noise²). |
| N6 | **Onboard orbit between fixes.** It used two-body only with a first-order velocity update, giving 55 km of error per orbit (32 km from the missing J2, 23 km from the integrator). | risky | Two-body + J2 by velocity Verlet. |
| N7 | **Frames.** The GNSS fix was taken as J2000, but receivers output ECEF. The field reference used GMST only (mean equator of date), while the Sun and star tracker are J2000: a 0.37° mismatch in 2027. The truth model had the same simplification, so no simulation could see it. | risky (flight) | The GNSS fix is in ECEF, as a real receiver outputs, and is converted onboard: r = Cᵀr_e, v = Cᵀ(v_e + ω_E ẑ × r_e). `eci2ecef` = GMST × IAU-76 precession, used onboard for the field and the GNSS fix, and by the truth model for the field and the GNSS emulator. |
| N8 | **IGRF coefficients** were interpolated once at boot and never refreshed. | minor | They are re-interpolated once a day. |
| N9 | **Earth-sensor update and sample rate.** The update needed a sample on exactly the cycle's first tick, which works at 2 Hz only by alignment. | minor | It uses the latest valid sample of the cycle. |

## Guidance

| # | finding | severity | fix |
|---|---|---|---|
| G1 | **In nadir the −Z power face pointed away from the Sun for the whole orbit.** The base frame puts body −Z on the orbit normal whatever the sign of β. Both cases have β < 0 (dawn–dusk −59°, 10:00 −24°), so the face was 147° and 112° from the Sun in every nadir orbit. | bug (mission) | **Yaw flip:** a 180° turn about the payload axis when the power face would look away from the Sun, with 0.1 hysteresis. It is applied in the flight software and to the truth reference of the metrics. On the dispatched missions the face is now 30° and 65° from the Sun, the geometric best of 90° − \|β\|. |
| G2 | **The gravity gradient at nadir.** The spec called it restoring, which is only true inside the Lagrange region. ais_img_3u flies its long axis along track, so pitch is gradient-unstable. | risky | Bit 1 of `mtq_gg_ff` (the nadir-state feed-forward) is set on the ground when J_normal ≥ J_along > J_nadir fails. That is ais_img_3u; ais_3u stays inside the region. The certificate models the feed-forward. |
| G3 | **The Floquet certificate took its reference frame from a mean of telemetry,** with a transpose "fix" that could mask a wrong attitude. | risky | It now uses the exact guidance frame: base frame, boresight offset, and the yaw flip if the mission flew it. |
| G4 | **The slew reference acceleration missed the transport term** −(ax φ ṡ) × ω_o (about 2 % of the peak). | minor | Added. |
| G6 | **The `guidance` initial rate** set ω_ref (reference frame) as the body rate. | minor | Now dcm(q_e) ω_ref. |

## What stays open

- **The Celani 2026 boresight law leaves the rotation about the payload axis free** (it is the paper's design,
  and why it certifies). With it, the power face in nadir goes wherever that rotation settles: on the coils-only
  ais_3u mission, 29–149° from the Sun over the nadir orbits, while the payload stays on nadir. If the energy
  balance needs the face held, a weak roll term, or a three-axis law in nadir, is the lever.
- **Gyro scale factor.** At the Sun-spin rate, 0.2 % of scale factor looks like 2e-4 rad/s of bias, and the bias
  state re-adapts slowly after the despin. The bias process noise does not use the catalogue's bias instability.
- **IGRF-13** is extrapolated for 2027. Load IGRF-14 for flight.
- **Frame model:** nutation (≤ 0.005°) and polar motion are omitted onboard and in the truth field. The POP gravity
  field keeps its GMST orientation.
- **Nadir is geocentric** (−r̂), not geodetic (up to 0.19°), and there is no yaw steering for Earth rotation.
  Against the 0.01° imaging requirement, the payload team must agree the definition.
- **The MATLAB twin** carries N1, N2, N6, G1, G4 and G6. It keeps J2000 GNSS, GMST-only frames, fixed
  sigmas and no gating (N3, N4, N7, N9). Its engine-parity table (`results/ENGINE_PARITY.md`) predates this
  audit.
- **Small items not changed:**
  - `ape_los` in the Sun states measures the payload axis; Sun pointing is scored by `sun_ape`.
  - The scenario's `guidance.kind` is not used by the flight software, which picks guidance from the state.
  - The slew timing is absolute (`gd_t0`).
