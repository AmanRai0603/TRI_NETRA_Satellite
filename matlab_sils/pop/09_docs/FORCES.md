# Forces

Each force is a file in `+forces/` with the signature `a = forces.<name>(ctx)`
returning a 3x1 ECI acceleration [m/s^2]. `op.accel` builds `ctx` once per
evaluation and sums the enabled terms (gravity is always included).

## The context struct `ctx`
| field | meaning |
|-------|---------|
| `t` | seconds since epoch |
| `utc` | `[Y Mo D H Mi S]` UTC now |
| `T` | `timeconv.convertUTC` struct (all time scales, GMST, doy) |
| `E` | `ephemInputs` struct (Sun/Moon ECI, GMs, P_srp, heliocentric Earth) or `[]` |
| `r_eci,v_eci` | ECI state [m, m/s] |
| `C,Ct` | ECI->ECEF and ECEF->ECI rotations (`r_ecef=C*r_eci`) |
| `r_ecef,v_ecef` | body-fixed state |
| `grav` | loaded gravity field (`mu,Re,Cbar,Sbar,nmax,J`) |
| `sc` | spacecraft (`mass,Aref,Cd,Cr,R_bi`, panels) |
| `cfg` | full config |

## Terms and their switches
| force | config | models / options |
|-------|--------|------------------|
| gravity | `cfg.forces.gravity` | `model`: `twobody`/`j2..j6`/`sphharm`/`toolbox`; `degree`,`order` |
| thirdbody | `cfg.forces.thirdbody` | `model`: `battin`/`direct`/`tidal`/`legendre` (Sun+Moon) |
| drag | `cfg.forces.drag` | `model`: `cannonball`/`sentman`/`dria`/`cll`; `atmos`; `Cd`; `corotate` |
| srp | `cfg.forces.srp` | `model`: `cannonball`/`boxwing`; `eclipse`: `cylindrical`/`conical`/`fine`; `Cr` |
| erp | `cfg.forces.erp` | `model`: `knocke`/`simple`/`ceres` (albedo+IR) |
| relativity | `cfg.forces.relativity` | `terms`: subset of `schwarzschild`,`lensethirring`,`desitter` |
| solidtides | `cfg.forces.solidtides` | IERS2010 degree-2 (Sun+Moon in ECEF) |
| oceantides | `cfg.forces.oceantides` | 8 main lines (FES-based framework) |
| empirical | `cfg.forces.empirical` | `acc=[aR aT aN]` constant RTN biases (for OD) |

## Adding a new force
1. write `+forces/mynew.m` taking `ctx`, returning `a` (3x1, ECI);
2. add `cfg.forces.mynew = struct('on',true, ...)` in `config.defaultConfig`;
3. add `'mynew'` to the `order` list in `op/accel.m`.

## Typical VLEO magnitudes (400 km, one orbit)
Measured by `ex04_force_impact_study.m`: drag >> luni-solar third body > solid
tides > relativity > SRP > ERP > ocean tides. Drag dominates everything at VLEO.
