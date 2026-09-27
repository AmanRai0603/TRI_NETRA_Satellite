# Seeding: where r0,v0 comes from, and what each source can achieve

The seed dominates everything. A velocity error `dv` becomes a semi-major-axis
error `da = 2a*dv/v`, which shows up as a radial oscillation of `da` **plus
along-track growth of ~3*dv*t**. Over a 3 h arc, 1 mm/s of seed error is ~32 m of
along-track. So the source of r0,v0 sets the floor, no matter how good the physics.

## The ladder (450 km, 3 h arc)
| source | seed dv | along-track after 3 h | note |
|--------|---------|----------------------|------|
| **SP3 with V records** | **0 (exact)** | 0 | the file's own r AND v are used verbatim |
| SP3, position only | 0.002 m/s | ~65 m | LSQ fit of the 30 s track |
| TU Delft track | 0.026 m/s | ~840 m | limited by the product's metre-level rounding |
| TLE mean elements | 0.56 m/s | ~18 km | limited by the TLE itself (~1 km orbit error) |

**Always prefer SP3.** Everything below it is limited by the DATA, not by the
propagator or the fit.

## Which satellites have what
| satellite | SP3 (best) | TU Delft track | TLE |
|-----------|-----------|----------------|-----|
| CHAMP | GFZ ISDC, **open** | yes | yes |
| GRACE-A/B, GRACE-FO-1/2 | GFZ ISDC, **open** | yes | yes |
| SWARM-A/B/C | swarm-diss, **open** (POD 30 s, or GPSxNAV 1 Hz) | yes | yes |
| GOCE | ESA archive, **login** + XML, not SP3 | yes | yes |
| SLATS, ISS | - | - | yes |

## Why TLE seeding cannot be "fixed"
A TLE is a MEAN element set fitted for SGP4. Its own accuracy is ~1 km at epoch,
degrading ~1-3 km/day. A 1 km semi-major-axis error alone is 0.56 m/s of seed
velocity. So a TLE-seeded run cannot beat ~km level -- that is the TLE, not the
code. `validate_tle` exists to MEASURE that, not to beat it. Use TLE only for
satellites with no better product (SLATS, ISS).

## Why TU Delft seeding is limited
TU Delft publishes a GPS-derived track as geodetic alt/lat/lon with metre-level
rounding, and NO velocity. Differentiating amplifies rounding by ~1/h, so the seed
velocity carries ~0.02-0.03 m/s however the fit is tuned (a local least-squares fit
of degree 7 over ~1000 s is already near the optimum -- see validation.fd_velocity).
Use TU Delft when there is no SP3, or for DENSITY (which is what it is for).

## Caching: nothing is downloaded twice
Precise orbits are IMMUTABLE -- a 2010 SP3 will never change -- so they are cached
with `maxAgeDays = Inf` and a second call never touches the network (measured:
0.01 s from cache). `opts.force = true` overrides. Files live under
`<data.root>/precise_orbit/`. The same holds for gravity fields, EOP and TU Delft
zips; only space weather is re-fetched, because it genuinely updates.
