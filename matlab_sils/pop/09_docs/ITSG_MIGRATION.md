# ITSG (TU Graz) — the single open source. Migration spec.

Verified from https://ftp.tugraz.at/pub/ITSG/satelliteOrbitProducts/operational/
and its readme.txt (fetched 2026-07-16). This supersedes GFZ / ESA / swarm-diss /
goce-ds for orbit validation.

## Why this replaces everything
> "Access is granted without any registration and free of charge."

Plain HTTPS + Apache directory listing. **No credentials, no FTPS, no EO Sign In,
no archive walking, no baseline counters.** The entire credential subsystem
(credentials.json, curlcfg, fetch_auth's gated path, browse, find_esa_file,
swarm_list) becomes DEAD CODE for orbit data.

## ★ The big one: positions are already in the CELESTIAL frame
> kinematicOrbit: "Position coordinates are given in the celestial reference frame."
> reducedDynamicOrbit: "Position coordinates and velocities are given in the
>   celestial reference frame."

No ITRF→ECI rotation is needed. That **deletes the entire class of bugs that has
dominated this project**: the CIP/omega seed error (CHAMP 7.6 km), 'gmst' on a real
ITRF track (GOCE 15.9 km), dCt/dt velocity conversion, EOP downloads for the
reference. The reference becomes what it should always have been: read the file,
seed, propagate, compare — all in one frame.

## 23 satellites, all open
CHAMP · GRACE-1 · GRACE-2 · GRACEFO-1 · GRACEFO-2 · Swarm-1 · Swarm-2 · Swarm-3 ·
Jason-1/2/3 · MetOp-A/B · Sentinel-1A/1B/1C · Sentinel-2A/2B/2C · Sentinel-3A/3B ·
Sentinel-6A · TerraSAR-X · TanDEM-X
(NOTE: no GOCE. GOCE stays a manual ESA download — see the XML reader task.)
(NOTE: Swarm-1/2/3 = Swarm A/B/C.)

## Products (all `<SAT>_<product>_<YYYY-MM-DD>.txt.gz` — CONSTRUCTIBLE)
| product | columns | units | sampling |
|---------|---------|-------|----------|
| `kinematicOrbit` | MJD, x, y, z | day, m | **measurement epochs — NOT equidistant, NOT integer seconds** |
| `kinematicOrbitCovariance` | MJD, xx, yy, zz, xy, xz, yz | day, m² | as kinematic |
| `reducedDynamicOrbit` | MJD, x, y, z, vx, vy, vz | day, m, m/s | **10 s**, both midnights included |
| `attitude` | MJD, q0, qx, qy, qz | day, - | 10 s (satellite→celestial) |
| `nonConservativeForces` | MJD, ax, ay, az | day, m/s² | 10 s (**satellite frame**) |
| `neutralDensity_ACC` r1.0 | MJD, rho | day, kg/m³ | 60 s |
| `neutralDensity` r1.5/2.0 | MJD, lon, lat, alt, LST, rho | day, deg, deg, m, h, kg/m³ | CHAMP 10 s (r2.0), GRACE/GRACE-FO 5 s (r1.5), TDX/TSX 30 s |

Format: GROOPS ASCII, gzipped. Epoch is **MJD in column 1 everywhere** — so every
product joins on one key. That is what makes the comparisons trivial.

## What this enables (the OD validation the user actually wants)
1. **r,v seed, exact, in ECI** — reducedDynamicOrbit cols 2-7. No derivation, no
   rotation. This should beat CHAMP's current 5.16 m.
2. **kinematic vs reduced-dynamic** — two independent orbit solutions of the SAME
   satellite/day. Their difference is the OD's own uncertainty floor: kinematic is
   geometry-only (GPS), reduced-dynamic is geometry + a force model. Comparing our
   propagator against BOTH separates "our physics is wrong" from "the reference
   itself is uncertain".
3. **non-conservative force validation** — the killer feature. `nonConservativeForces`
   is the MEASURED (accelerometer) drag+SRP+ERP acceleration. Our model computes the
   same thing. Plot measured vs modelled, per axis → this directly validates the drag
   coefficient, the density model and the SRP model, instead of inferring them from a
   position residual 3 hours later. Needs `attitude` (quaternion) to rotate the
   satellite-frame measurement into ECI.
4. **density** — the measured rho alongside our NRLMSISE/JB2008/DTM2020 call.

## Migration plan (next session)
1. `data.itsg(sat, date, product)` — one fetcher: build URL, cache-first (Inf),
   gunzip, parse GROOPS ASCII → struct with .mjd + columns. ~80 lines, no auth.
2. `itsg_catalog.csv` — 23 satellites x 7 products x date coverage, from the live
   listing (each satellite dir must be listed once to get its real date range).
3. Rewire `validation.gps_reference` → ITSG reducedDynamicOrbit (drop the frame
   chain entirely for this path).
4. **One master validation** `validate_OD.m`: seed from reducedDynamicOrbit →
   propagate → compare vs reduced-dynamic AND kinematic → model non-conservative
   forces and compare vs the accelerometer → compare density. All knobs surfaced,
   one file.
5. **One master example** `EXAMPLE_16U.m` for the 16U propagator, same open style.
6. Keep TU Delft (r only) and TLE (r,v ~km) ONLY as extra overlay curves on the
   position plot, on the SAME epochs, for comparison. Nothing else.
7. Delete: the credential subsystem for orbits, the ESA walkers, the three
   validate_* scripts, and any source not used by the above.
8. Save every fetched input with a SOURCE TAG + retrieval date (EOP, space weather,
   gravity field, ITSG products) in a segmented cache, so a run's provenance is
   reproducible.

## Honest status
Site + readme VERIFIED by fetch. The parsing/fetching code is NOT yet written —
this document is the spec, not a claim that it works.

---
# VERIFIED against real files (CHAMP 2003-01-01, all 6 products)

## The real GROOPS format is NOT what the readme implies
    1: groops instrument version=20200123      <- version line, no '#'
    2: # ORBIT | STARCAMERA | ACCELEROMETER | COVARIANCE3D | MISCVALUE
    3:        -6          1                    <- type code + count  (BARE NUMBERS)
    4: # Time [MJD]  data0: pos x [m] ... data8: acc z [m/s^2]
    5: # =====================================
    6:       8640                              <- epoch count        (BARE NUMBER)
    7+: data
**Lines 3 and 6 are bare numbers with no '#'**, so a naive "skip comments, sscanf the
rest" reader eats them as data, injecting fake epochs at MJD -6 and MJD 8640. The
parser now requires the exact column count declared by the 'dataN:' labels AND an
MJD in 4e4..7e4. Both fakes are rejected. (MISCVALUE declares no labels -> the width
is inferred from the first row that carries a plausible MJD.)

## ORBIT files carry NINE data columns, not six
    data0..data8 = pos x,y,z [m] | vel x,y,z [m/s] | acc x,y,z [m/s^2]
The readme documents only pos+vel. reducedDynamicOrbit ALSO ships the acceleration
of its own solution (exposed as D.a_rdo) -- an independent cross-check on our force
model. kinematicOrbit declares the same width (D.v_kin / D.a_kin when present).

## Measured facts (CHAMP 2003-01-01)
| product | type | cols | rows | dt |
|---|---|---|---|---|
| reducedDynamicOrbit | ORBIT | 10 | 8640 | 10 s |
| kinematicOrbit | ORBIT | 10 | 7464 | 10 s (measurement epochs -> fewer, as documented) |
| kinematicOrbitCovariance | COVARIANCE3D | 7 | 7464 | 10 s |
| attitude | STARCAMERA | 5 | 8640 | 10 s |
| nonConservativeForces | ACCELEROMETER | 4 | 8640 | 10 s |
| neutralDensity_ACC | MISCVALUE | 2 | 1440 | 60 s |

## Physics checks (all pass)
- altitude 400.1 km, |v| 7670.3 m/s, |h| 5.199e10 -- a real CHAMP orbit at MJD 52640.
- the file's own v matches a central difference of its own r to **0.16 m/s** (a
  10 s FD truncation on a real orbit) -> internally consistent, and NOTHING needs
  deriving: use cols 5-7 directly.
- attitude |q| = 1.000000 exactly.
- non-conservative acceleration 2.42e-06 m/s^2 -- right order for drag at 400 km.
- density 2.54e-12 kg/m^3 -- right order for 400 km.

## ★ THE REFERENCE'S OWN UNCERTAINTY
    kinematic - reducedDynamic = **0.074 m RMS (3D)** over 7464 common epochs
Two independent solutions (geometry-only vs geometry+dynamics) agree to **7 cm**.
That is the floor: a propagator residual below ~0.07 m is meaningless. It also means
this reference is ~70x tighter than CHAMP's best GFZ SP3 result so far (5.16 m), so
it can actually resolve our force-model errors.

# API bugs in validate_OD, found by checking the real signatures
Writing the master against the readme was not enough -- three calls were wrong:
1. **`op.accel`** is `[a, parts] = op.accel(t, r, v, W)` -- SEPARATE r and v, and no
   cfg. I had written `op.accel(t,[r;v],W,cfg)`. Fixed. Better still, `parts.<force>`
   already carries each force's own vector (parts.gravity, parts.drag, parts.srp...),
   so the non-conservative model acceleration is simply
       sum over {drag, srp, erp} of parts.<f>
   -- no gravity-only re-run needed (that hack is deleted).
2. **`op.density` DOES NOT EXIST.** The atmosphere is reached via
   `atm = atmos.provider(model, geo, sw)` with GEODETIC inputs
   (.alt_km .lat_deg .lon_deg .lst_h .doy .utc), as 11_compare/compare_density.m
   already does. Rewired.
   * For neutralDensity release 1.5/2.0 the product ships its OWN lon/lat/alt/LST ->
     use those, so model and measurement are evaluated at exactly the same point.
   * For release 1.0 (the user's CHAMP neutralDensity_ACC: MJD + rho only) the point
     must come from our own state: ECI -> ECEF -> geodetic. That folds our position
     error into the density comparison, and the code says so.
3. **`frames.eci2geodetic` does not exist either**; the real function is
   `op.geodetic(r_ecef, Re, f) -> [lat, lon, alt]`. Fixed.
Retired (they referenced removed scripts): TEMPLATE_validation.m, show_validation.m,
validate_against_gps.m -> 06_validation/_retired/.
Regression after all of it: test_gravity PASS, test_energy PASS.
STILL OPEN: 11_compare/compare_orbit.m and 05_data/+data/precise_orbit.m still
reference the retired path; EXAMPLE_16U.m is not written; validate_OD has still not
been executed end-to-end.

# Cleanup: the codebase is now ITSG-only
DELETED (the whole credential/multi-source subsystem -- dead under ITSG):
  05_data/credentials/            (store, template, web_sources.csv, README)
  05_data/data_sources/
  05_data/+data/: credentials.m curlcfg.m fetch_auth.m browse.m find_esa_file.m
                  swarm_list.m precise_orbit.m sources.m availability.m selftest.m
                  fetch.m  (a dispatcher for the GFZ/ESA path)
  05_data/sat_data/: sat_catalog.csv data_availability.csv rv_capability.csv
                     web_sources.csv
  06_validation/: verify_access.m, _retired/, +validation/gps_reference.m
  06_validation/realsat/: the 3 validate_* scripts, run_gps_validation, val_figures,
                          check_forces, verify_seed, the old examples
  11_compare/: compare_orbit.m get_reference.m (both GFZ/ESA-seeded)
  07_examples/: ex01, ex02, ex04, ex05, ex06
  09_docs/: LOGINS DATA_SOURCES DATA_AVAILABILITY DATA_CATALOG SATELLITES DATA
            MANIFEST VALIDATION 00_START_HERE  (all described the old architecture)
  TEMPLATE_validation.m, EXAMPLE_propagate_satellite.m
KEPT + REWIRED:
  05_data/+data/: itsg.m (the one fetcher) ensure.m root.m eop_dir.m gravity.m
                  spaceweather.m spaceweather_forecast.m jb2008_indices.m
                  tle.m tudelft_density.m  (the last two ONLY feed the overlays)
  05_data/sat_data/itsg_catalog.csv  -- now the ONLY catalog, with a NORAD column
  05_data/+sat/catalog.m -- a thin shim reading itsg_catalog.csv, so track_reference /
                  compare_density / test_seed keep working against ONE truth instead
                  of a second list that could drift.
  06_validation/realsat/: validate_OD.m + show_OD.m + README.md  (that is all)
test_seed used sat.import('GOCE') -- GOCE is not on ITSG at all, and sat.import is
gone; it now takes CHAMP's parameters from sat.catalog. The test is about the SEED,
not the vehicle. Regression after the whole cleanup: test_gravity PASS,
test_energy PASS.

# EXAMPLE_16U.m — the master propagator example
One file, every knob, ordered by what actually matters at low LEO (drag > gravity
degree; Cd*A/m > either; area/attitude ~ drag model). Sections:
  1 spacecraft (16U: 24 kg, 0.08 m^2, Cd 2.2 -> B = Cd*A/m = 7.3e-3 m^2/kg, i.e.
    ~1.7x draggier per kg than CHAMP's 4.4e-3 -- stated in the file so the number
    means something)
  2 orbit  3 forces  4 space weather  5 integrator  6 frame  7 output  8 plots
  9 **SWEEP**: give ONE knob a list of values -> one run per value, overlaid. This
    is the point of the file: see an EFFECT, not a number. Legal knobs are explicit
    (no eval), so a typo fails loudly:
      ALT_KM [250 300 350 400]              the drag cliff
      SC.Cd / SC.Aref_m2 / SC.mass_kg       ballistic coefficient sensitivity
      SW_MANUAL.F107 [70 150 250]           solar cycle -> the lifetime RANGE
      FORCES.gravity.degree [4 20 70]       when does degree stop mattering
      FORCES.drag.atmos {nrlmsise,jb2008,dtm2020}   model spread
      INTEG_METHOD {rk4,rk78,ode45}         must NOT change the answer
  11 decisions printed + saved   12 plots
NEW: **op.coe2rv(a,e,inc,RAAN,argp,nu,mu)** -- did not exist (the old ex03 hand-built
a circular state). Mirrors op.rv2coe, radians throughout, perifocal construction.
VERIFIED: round-trip through op.rv2coe gives da=-2.8e-09 m, de=4e-17, di=0,
dRAAN=6e-15 deg -- machine precision.
DELETED with it: ex03_leo_full_stack.m, compare_16u.m, accelerations_timeseries.m,
state_overlay.m, inputs_report.m (all superseded).
STILL OPEN: show_16U.m (the plotter EXAMPLE_16U calls) is NOT written; the master
comparator in 11_compare is NOT built; validate_OD has still never been executed.

# show_16U.m — written and EXECUTED
The plotter EXAMPLE_16U calls. One figure per QUESTION; when a sweep is on, every
figure OVERLAYS the runs on the same axes (the point of a sweep is watching the
curves separate):
  1 altitude + **\Delta sma** (the secular decay, free of the short-period wobble)
  2 elements (sma as \Delta, ecc, inc, RAAN)
  3 **force ladder** (log |a| per force, from op.accel's parts.<force>) -- the most
    instructive plot: it shows WHICH force is worth arguing about at this altitude
  4 energy drift -- labelled honestly: with drag ON this is PHYSICS, not integrator
    error; switch drag/SRP off to read it as pure integrator error
  5 ground track (with the longitude wrap broken so it does not draw across the map)
  6 3D ECI trajectory with the Earth
VERIFIED BY RUNNING IT (Octave, gnuplot): 5 figures, no errors.
  16U at 300 km, B = Cd*A/m = 7.33e-03 m^2/kg  ->  300.0 -> 292.5 km in 30 MINUTES.
  That is not a bug: a 24 kg / 0.08 m^2 body at 300 km really is that draggy, which
  is exactly what EXAMPLE_16U exists to make visible.
Compatibility: subtitle() is R2020b+, so it is wrapped in try/catch and degrades to
an xlabel note on older MATLAB and on Octave.

# validate_OD: first execution attempt (partial)
Ran with the user's REAL CHAMP 2003-01-01 files seeded into the cache.
CONFIRMED WORKING:
  * data.itsg resolves from cache inside validate_OD, no network, no credentials:
      [itsg] cache: CHAMP_reducedDynamicOrbit_2003-01-01.txt.gz
      [itsg] CHAMP reducedDynamicOrbit 2003-01-01: 8640 epochs, celestial (inertial)
  * all 6 products load: r(1)=[4768938.3 -1163812.9 4674070.4] m,
    v(1)=[5256.12 -761.08 -5534.21] m/s, |a_nc|=2.42e-06 m/s^2, |q|=1.000000
  * provenance struct populated (provider/product/frame/url/retrieved)
FIXED ALONG THE WAY:
  * setup_paths still addpath'd 05_data/data_sources/{satellite,spaceweather,
    density_reference}, which the cleanup deleted -> four warnings on every startup.
    Removed.
BLOCKED (sandbox, not necessarily a real bug):
  * 'mjd2utc undefined' at the first local-function call. Local functions in SCRIPTS
    are MATLAB R2016b+; Octave's run() does not resolve them. The user is on real
    MATLAB so this may simply work there. IF IT DOES NOT, the fix is to move the
    local helpers (mjd2utc, rtnStats, doy_, quat2dcm_local, itsgCat) out of
    validate_OD.m into +validation/ as proper functions.
STILL NEVER EXECUTED END-TO-END. Next session: run validate_OD on real MATLAB with
the cached CHAMP files and fix what surfaces.
