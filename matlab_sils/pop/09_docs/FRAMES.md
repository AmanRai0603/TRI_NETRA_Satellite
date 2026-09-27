# Frames (ECI <-> ECEF) and time

## Time
`timeconv.convertUTC(Y,Mo,D,h,m,s,dUT1)` returns every scale you need
(`utc/tai/tt/tdb/ut1/gps` JDs, `T_tt`, `gmst_rad`, `doy`). Feed `tdb_jd` to the
ephemeris, `ut1_jd`/`T_tt` to the frame chain. Leap seconds live in
`ephemeris/+timeconv/leapTable.m` (TAI-UTC currently 37 s).

## Rotation
```matlab
[C,Ct] = frames.eci2ecef(utc, build, opt);   % r_ecef = C * r_eci
```
| build | source | accuracy | needs internet |
|-------|--------|----------|----------------|
| `gmst` | GMST only (no precession/nutation/EOP) | ~arcsec (tens of m at LEO) | no |
| `A` | finals2000A (Bulletin A/B) | matches `dcmeci2ecef` | yes (once, cached) |
| `B` | EOP 20 C04 (ITRF2020/ICRF3) | precise | yes (once, cached) |
| `C` | C04 + sub-daily tidal + zonal + cubic | maximum | yes (once, cached) |

Builds A/B/C implement the full IAU 2006/2000A CIO chain and download IERS EOP
once, caching for ~7 days. They accept an `opt.eop_override=[dUT1 xp yp dX dY dAT]`
and vectorised `Nx6` UTC input. The shipped **examples default to `gmst`** so
they run with no network; switch `cfg.frame.build='B'` for precise runs.

`op.accel` computes **one** transform per evaluation and passes `C/Ct` to every
force, so gravity (evaluated in ECEF), tides (bodies rotated to ECEF), and drag
(geodetic altitude) all share it.
