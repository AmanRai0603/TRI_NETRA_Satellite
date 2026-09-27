# OD validation — one master script

    edit validate_OD          % set SAT + DATE at the top, run it.

That is the whole interface. Everything else here supports it.

| file | role |
|------|------|
| `validate_OD.m` | **THE master.** Every knob at the top, satellite list in the header, nothing decided behind your back. Section 10 prints and saves the decisions + provenance of every input. |
| `make_OD_fixtures.m` | synthetic ITSG files built from the engine's own output -> runs `validate_OD` offline with KNOWN answers. See `09_docs/OD_VALIDATION.md`. |
| `show_OD.m` | the figures: position residual vs both references, non-conservative force measured-vs-modelled, density, a statistics table, and external overlays. |

## What it validates
1. **position/velocity** vs `reducedDynamicOrbit` (the seed source)
2. **position** vs `kinematicOrbit` — an INDEPENDENT solution (GPS geometry only, no
   force model). Their difference gives the **reference's own uncertainty**:
   measured at **0.074 m RMS** on CHAMP 2003-01-01. A residual below that is noise.
3. **non-conservative force** — the accelerometer measures drag+SRP+ERP directly in
   m/s^2 every 10 s. Rotated to inertial through the attitude quaternion and compared
   against our model. This tests Cd*A/m and the density model AT THE SOURCE, instead
   of inferring them from a position error hours later. The most diagnostic plot here.
4. **density** — measured neutral density vs our atmosphere model.
5. **overlays** (optional) — TU Delft track (position only) and a TLE-seeded run,
   interpolated onto the SAME epochs, plotted against the same reference so their
   quality is directly visible. They are NOT the reference.

## Data
ONE source: ITSG / TU Graz. No credentials, no FTPS, no archive walking.
Positions and velocities are already in the CELESTIAL (inertial) frame, so this path
performs NO ITRF->ECI rotation — which removes the bug class that produced this
project's two worst errors (CHAMP 7.6 km from a CIP/omega seed error, GOCE 15.9 km
from 'gmst' on a real ITRF track).

    itsg.list                                   % 24 satellites, what each offers
    data.itsg('CHAMP','2003-01-01','reducedDynamicOrbit')

Retired scripts are in `../_retired/` — they were pre-ITSG and are kept only for
reference.


## Where the metrics live
`validate_OD` does not compute its own metrics. It fetches the ITSG products and
hands them to **`validation.od_metrics`**, which `compare_OD.m` also calls — one
implementation, so the two scripts cannot drift apart.

## Before you trust a number
Read `09_docs/OD_VALIDATION.md`. Every bug this path has had failed *quietly*, and
the reference has a floor (`refSpread`, a few cm) below which residuals are not
resolvable. Metric (1) ranks agreement with TU Graz's force model, not with
physics; metrics (3) and (4) are measurements. Prefer them when they disagree.
