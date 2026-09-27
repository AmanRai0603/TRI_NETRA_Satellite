# Code audit — findings and fixes

A two-pass audit: (1) **basic** — hardcoded values, portability, dead paths; and
(2) **advanced** — name collisions, variable shadowing, data mismatches. Every
item below was checked by grep across the whole tree; fixes are applied unless
marked as an accepted exception.

## Pass 1 — basic

### 1.1 Hardcoded physical constants
| where | value | verdict |
|-------|-------|---------|
| `+drag/{relVelocity,force,cannonball}.m` | ω = 7.2921159e-5 | **MISMATCH — FIXED** → 7.2921150e-5 to match `de440.constants().omega_earth` (see 2.3) |
| `+erp/{ceres,simple,knocke}.m` | Re, S=1361, c | consistent across files (no mismatch); centralising into the hub is a nice-to-have, left as literals to avoid a per-step package call |
| `+relativity/{schwarzschild,lenseThirring}.m` | c=299792458 | exact SI value, identical everywhere — accepted |
| `+srp/eclipse.m` | Re, Rp | getf-defaults, overridable — accepted |
| `08_test/*.m`, `TEMPLATE_propagation.m` | μ, Re | **intentional** — tests need independent reference values; the template is user-facing illustration |

All *operational* constants resolve through `de440.constants()`; the remaining
literals are either test references, user templates, or consistent SI values.

### 1.2 Hardcoded paths
Only a `C:\path\...` string inside a **help comment** (`get_jb2008_indices.m`),
illustrating manual override. No real absolute paths in code. Runtime paths use
`data.root()` / `tempdir`. **Clean.**

## Pass 2 — advanced

### 2.1 Duplicate file names across packages
Most are **safe** because they live in different `+packages` and are only ever
called package-qualified:
`op.accel` / `thirdbody.accel` / `erp.accel`; `drag.cannonball` / `srp.cannonball`;
`forces.gravity` / `data.gravity`; `atmos.spaceweather` / `data.spaceweather`;
`thirdbody.total` / `relativity.total`; `op.propagate` / `thirdbody.secular.propagate`.
No action needed — the package prefix disambiguates.

### 2.2 Bare-name collisions on the path (real risk)
| name | locations | verdict |
|------|-----------|---------|
| `vleo16u.m` | `+dgeom/` **and** `+sgeom/` — both bare, both on path, **different signatures** (`[facets,opts]` vs `sc`) | **BUG (latent) — FIXED**: renamed to `vleo16u_drag.m` and `vleo16u_srp.m`. No callers existed, so nothing breaks. |
| `read_tle_file.m` | `+validation/` (package) **and** `data_sources/satellite/` (bare, your parser) | **known collision — handled**: a bare `read_tle_file` resolves to your element-parser; the raw-line reader the pipeline needs is called explicitly as `validation.read_tle_file` (fixed earlier in `data.tle`). Documented so it stays that way. |
| `addArray.m`,`buildBox.m` | `+dgeom/` (bare) + `+srp/` (package) | **safe** — only one *bare* copy each; the SRP ones are `srp.addArray`/`srp.buildBox`. |
| `chebval.m` | `+de440/` and `+de440/private/` | intentional (MATLAB resolves `private/` inside the package; the extra copy is the Octave-compat shim location). Accepted. |

### 2.3 Data mismatch (the important class)
- **Earth rotation rate ω**: the drag defaults used 7.2921159e-5 while the
  constants hub uses 7.2921150e-5. In the live force path ω comes from the shared
  context (correct value), so this only bit if a drag sub-function was called
  standalone — but it was a genuine inconsistency. **Aligned to the hub.**
- No other numeric mismatches found: μ, Re, c, solar constant are identical
  wherever duplicated.

### 2.4 Overwritten variables / shadowing
Checked the hot paths (`op.accel`, `op.rhs`, integrators, `forces.*`). The
per-evaluation context `ctx` is built once and read-only downstream; force
wrappers write only their local `a`. Every helper uses a private `getf`; these
are file-local (subfunctions), so no cross-file shadowing. No loop index or state
variable is reused across scopes in a way that changes results.

## Post-fix verification
`run_all_tests` → gravity / integrators / energy all **PASS** after the fixes
(energy relative drift 7e-12 over 10 orbits unchanged). The two renamed geometry
helpers and the ω alignment do not affect any numeric result.

## Residual recommendations (low priority, not blocking)
1. Centralise `c` and the solar constant into `de440.constants()` and reference
   them in `+erp`/`+relativity` (cosmetic; values are already consistent).
2. Consider moving `+dgeom/` and `+sgeom/` helpers into `+dgeom` /
   `+sgeom` packages to remove bare names from the global path entirely.

## Fetch/parse robustness pass (data download & readers)
Triggered by a real CHAMP SP3 failure: the GFZ portal serves a gzip file under a
`.raw` name, so extension-based "is it compressed?" logic never unzipped it and
`read_sp3` then choked on binary. Fixes:
- **`precise_orbit.maybeUnzip`** now detects gzip/zip by **magic bytes** (1f 8b /
  'PK'), not by filename — portals often mislabel. gunzip is done via a temp `.gz`.
- **`precise_orbit` `opts.file`** is now unzip-aware: a stored `.sp3` OR `.sp3.gz`
  both work ("download & store, or auto-download if absent").
- **`read_sp3`** guards against an empty/partial epoch (skips a P-record before any
  valid `*` epoch) and errors clearly if nothing parsed, instead of an assignment
  crash. Accepts only full 6-field epoch lines.
- **`fetch_tudelft_density`** unzips on name OR zip magic bytes (defensive).
- `fetch_attitude` already sniffed gzip by content (`is_gzip`) with try/catch — OK.
Verified: a gzip SP3 named `.sp3.raw` now flows through
`precise_orbit -> maybeUnzip -> read_sp3 -> gps_reference` with 61 epochs, no crash.

## Package-vs-path pass (MATLAB "Unrecognized function" class)
Triggered by `tidal_eop` failing on MATLAB. Root cause: the eci2ecef toolbox was
vendored with its tidal-EOP helpers INSIDE the `+frames` package, but
`eci2ecef_A/B/C` call them UNQUALIFIED (`tidal_eop(...)`, `tidal_ut1_zonal(...)`)
as in the original path-based layout. A package file calling a sibling package
file unqualified works in Octave but FAILS on MATLAB. Fix:
- Moved `tidal_eop`, `tidal_eop_ocean`, `tidal_pm_libration`, `tidal_ut1_libration`,
  `tidal_ut1_zonal`, `example_tidal` OUT of `+frames` into
  `03_frames_time/tidal_eop_models/` (on the path), matching the original toolbox.
  `+frames` now holds only `eci2ecef*` + `utcvec`. Added the folder to `setup_paths`.
- Verified `xys06_tables.mat` (required by eci2ecef_C, `mfilename`-relative) is
  present in `+frames`; Leap_Second/finals/C04 are runtime downloads.
Then AUDITED every package for the same pattern (sibling package file called
unqualified): the only other hit is `de440/state.m -> chebval()`, which is SAFE
because `chebval` is a PRIVATE function (`+de440/private/chebval.m`) and MATLAB
resolves private functions for unqualified package calls. No other cases exist.
Frames: `gmst` works offline; `C`/`B` work on MATLAB (auto-download EOP+leap sec).

## Vector-shape pass (MATLAB row/column broadcast)
Triggered by `rtn_residual` crashing ("1-by-3 = 3-by-3"): `sol.stateAt` returned a
1x6 ROW (Hermite output is row-per-time), but its docstring promised `[r;v]` and
callers did column math, so `row - column` broadcast into a 3x3 matrix. Fixes:
- **`sol.stateAt`** now returns `[r;v]` as a COLUMN (6x1 for one time, 6xM for M
  times) — matching its documentation. Root fix.
- **`rtn_residual`** also forces `rv(:)` defensively.
- Audited the other state consumers: `op.rv2coe` and `op.geodetic` are shape-safe
  (verified identical results for row and column inputs); the two `stateAt` demos
  (ex06, TEMPLATE) only index-and-print, unaffected. No other row/column hazards
  in the runtime path.

## Fetcher robustness pass (cache-first before any network)
Triggered by CHAMP failing with "no built-in SP3 URL" even though the file had
already downloaded: the cache check lived INSIDE data.ensure, which was only
reached AFTER a URL resolved. When the GFZ directory-listing `webread` hiccuped,
no URL was produced, so the already-downloaded file was never reused. Audited
every fetcher for "network/listing before cache":
- **precise_orbit** — now CACHE-FIRST at the top: a cached `.sp3` or `.raw` is
  reused before any URL work (so a stored file always works, no network). The
  CHAMP GFZ URL is now built DETERMINISTICALLY (`GFZOP_RSO_L06_G_<d-1>_220000_
  <d>_120000_v01.sp3.gz`) — the fragile directory listing is only a fallback.
  `opts.force=true` bypasses cache.
- **data.tudelft_density** — now CACHE-FIRST on the saved `.mat` (reused with NO
  web directory listing); also writes the `.mat` into `data.root()/density_
  reference/` instead of the toolbox root.
- **data.tle** — already cache-first (checks the cached `.tle` before SatChecker). OK.
- **data.spaceweather** — sub-fetchers cache raw OMNI2/GFZ into `spaceweather/`. OK.
- **fetch_attitude** — has its own persistent `CacheDir` (re-runs never re-download). OK.
Result: once any product is downloaded it is reused offline; fresh fetches use
deterministic URLs where possible, with live listings only as a fallback.

## Deep technical audit (latent failure modes)
A full-codebase pass (224 .m files) for errors lying beneath the surface —
silent degradations, unguarded numerics, and mismanaged toggles. Findings + fixes:

### Fixed — HIGH severity
1. **Integrator NaN -> infinite loop / silent NaN.** In `rk78`/`rk45` a NaN in the
   dynamics made `err<=1` false forever, so the step was rejected endlessly, `t`
   never advanced, and `maxsteps` (which counted only *accepted* steps) never
   tripped -> a hang; or a NaN state propagated silently to the end. Now: an
   `isfinite` guard on the state and derivative **errors immediately** with the
   likely cause, and a **total-iteration cap** stops repeated rejection.
2. **Silent NaN from spacecraft config.** `drag`/`srp` use `sc.Aref/sc.mass`
   directly; a zero/absent mass gave `Inf/NaN` that poisoned the whole run with no
   message. `buildWorld` now **validates** `mass>0` and `Aref>0` (clear error)
   whenever drag or SRP is on.

### Fixed — MEDIUM severity (silent degradation of results)
3. **Ephemeris kernel failure was swallowed.** `buildWorld` did
   `try W.eph=de440.open(); catch W.eph=[]; end` — if the DE440 kernel failed to
   load, third-body/SRP/ERP/tides ran on degraded Sun/Moon with no warning. Now it
   **warns loudly** when the kernel is missing *and* a force needs it.
4. **Gravity degree silently truncated.** `nmax=min(g.degree, fld.nmax)` quietly
   capped the requested degree to the loaded field (default field is only J2-J6),
   so "gravity 20x20" and "40x40" gave identical results. Now `buildWorld`
   **warns once** and tells you to load EGM2008/EIGEN-6C4 for higher degree.
5. **Top-level input validation.** `op.propagate` now checks `epoch/r0/v0/tspan`
   are present, `r0/v0` have 3 finite elements, `tf>t0`, and warns if `|r0|` looks
   like km-not-metres — instead of cryptic field/'size' errors deep in the stack.

### Reviewed — OK (no change needed)
- `op.accel` dispatch guards each force's `.on` (so tide funcs need no internal
  guard) and forces column vectors. Ephemeris built only when a force needs it.
- Density models do **not** silently fall back — `atmos.provider` errors on an
  unavailable model; `atmos.spaceweather` errors ("noData") rather than
  extrapolating past the loaded window.
- Time system: `convertUTC` applies leap seconds (`taiMinusUTC`) and `dUT1`
  (UTC->TAI->TT->TDB, UT1/GMST) correctly.
- `/norm()` sites: the vectors involved (position, velocity, Sun-distance,
  angular momentum for real orbits) are non-zero in normal operation; any
  degenerate NaN is now caught by the integrator finite-guard with a clear error.
- All integrators re-verified: gravity/integrator/energy tests PASS after changes.

## Integration-edge pass (field-name & endpoint drift)
Two concrete runtime breaks a user hit, fixed at the source:
- **DTM2020 wrong output fields.** `atmos.dtm2020` read `out.rho`/`out.Tz`/`out.nO`,
  but `dtm2020_oper_density` actually returns `out.rho_kgm3`, `out.T_K`, and
  `out.n.{O,N2,...}`. Fixed the adapter to the real field names -> DTM2020 drag
  now works.
- **EIGEN-6C4 URL 404.** ICGEM migrated to `icgem.gfz.de` and its download hashes
  change, so the registry link 404'd. Now each model lists both hosts (gfz.de +
  gfz-potsdam.de) as candidates, and a registry-download failure raises an
  ACTIONABLE error: use EGM2008 (degree 2190, works, equivalent for orbit
  propagation to << um/s^2), or download the .gfc from icgem.gfz.de/tom_longtime
  and place it in data_cache/gravity/ (or pass the URL). EGM2008 auto-download is
  verified working; EIGEN-6C4 is optional and equivalent for propagation.

## Interface pass 2 (seed accuracy + integrator options)
- **TU Delft seed velocity was ~2% low (ROOT CAUSE of the 107 km "altitude < 120"
  crash).** `track_reference` finite-differenced the velocity AFTER thinning to
  ~5 min spacing, so the chord velocity was -1.96% in magnitude and off-tangent.
  A 2% seed-velocity error drops perigee hundreds of km -> the propagation dove
  below the DTM2020 floor and died. Now the velocity is finite-differenced at
  FULL 10 s resolution and thinned afterwards -> error -0.002% (verified on a
  synthetic orbit). The seed is now a valid orbit; no low-altitude crash.
- **No altitude clamp** (by request): the density-model floor stays a hard signal,
  not masked -- with a correct seed it simply isn't hit.
- **Added ODE-suite integrators**: `ode45`, `ode78`, `ode89`, `ode113` (MATLAB
  built-ins; ode78/89 need R2021b+) alongside rk4/rk45/rk78/rk6luther/nystrom4/
  gaussJackson8. `integ.run` already supplies a default step `h` for the
  fixed-step methods when only adaptive opts are given.
- DTM2020 adapter field names fixed (previous pass) -> DTM2020 drag runs.

## Methodology fix (TLE seed diagnostic compared at the wrong time)
The example printed |dr0| between the TLE-derived state and the GPS state, but
`tle2eci` built the TLE state at the TLE's OWN epoch, while the GPS state is at
its first sample -- often hours apart. The huge difference (e.g. ~1800 km) was
mostly orbital PHASE, not real TLE error, and looked alarming. Fix: `tle2eci`
now takes an optional targetEpoch and advances the mean elements to it with J2
secular rates (RAAN/argp/M), so the comparison is at the SAME instant (true TLE
accuracy, tens of km). The validation itself was always correct -- it seeds from
the GPS/SP3 state and reports the propagated-vs-measured drift; only the TLE
diagnostic line was comparing across times.

## Added: propagated-vs-measured state overlay (requested figure)
The comparison scripts previously showed only RTN residuals. Added
`11_compare/state_overlay.m`: plots the RAW state r,v (the fundamental quantity;
altitude/speed/RTN are conversions from it). Fig 1: r_{x,y,z} and v_{x,y,z},
measured (solid) vs propagated (dashed). Fig 2: the per-component differences
dr,dv (propagated - measured). Both from the SAME seed. Wired into compare_orbit (best
config, vs SP3 and vs TU Delft) and example_orbit_validation. Drift is 0 at t0
(same seed) and grows with the missing physics -- the picture of "our
propagation vs reality".

## Overlay extrapolation + per-source scripts + robust TU Delft cache
- **The 1e11 m "explosion" was Hermite EXTRAPOLATION, not physics.** The propagation
  runs for TSPAN_S (e.g. 3 h) but the SP3 reference spans the full day (~14 h);
  `state_overlay` sampled the dense output at ALL reference epochs, so past the last
  integrated node the Hermite polynomial blew up. Verified a 14 h propagation is
  bounded. Fix: `state_overlay` now clips to `sol.t(end)` -- it never samples past
  the propagated arc (max drift went from 1e11 m to a physical ~30 m in the test).
- **Separate per-source scripts** (clearer initialisation): `validate_sp3.m`
  (seed = SP3 r,v, direct), `validate_tudelft.m` (seed = reconstructed track r,v,
  full-res velocity), `validate_tle.m` (seed = TLE mean elements advanced to the
  SP3 epoch, compared vs SP3). Shared figure helper `show_validation.m`.
  `example_orbit_validation.m` kept as the combined SP3/TU Delft script but now
  fetches the reference ONCE (uses res.ref) with the TLE diagnostic optional.
- **TU Delft corrupt-zip self-heal.** A partial/corrupt cached .zip used to fail
  every run ("Invalid ZIP"). `fetch_tudelft_density` now validates the cached file
  (zip magic 'PK' + size) and re-downloads if it is bad, with a one-time unzip retry.

## Fixes: overlay extrapolation, corrupt cache, separate scripts
- **state_overlay blow-up (10^10 m) was Hermite EXTRAPOLATION, not a bad
  propagation.** It sampled sol.stateAt at EVERY reference epoch, but the SP3
  reference spans a full day while the arc is only TSPAN_S; past the last node
  the dense-output polynomial diverges. Now clipped to `ref.t <= sol.t(end)`.
  (The propagation and the RTN residual over the arc were always correct.)
- **Corrupt TU Delft zip cache now self-heals.** A partial download poisoned the
  cache and failed every run ("Invalid ZIP"). fetch_tudelft_density now validates
  the cached file (zip magic 'PK' + size) and re-downloads if invalid, with a
  retry around unzip.
- **example_orbit_validation double-fetched** the reference (once for the
  diagnostic, once inside validate_against_gps). Restructured to fetch ONCE
  (use res.ref); the TLE seed diagnostic is now OPTIONAL (SHOW_TLE_SEED=false).
- **Separate per-source scripts added**: validate_sp3.m, validate_tudelft.m,
  validate_tle.m -- each with its own clean initialisation, all using
  show_validation (RTN + r,v overlay + differences).

## Residual analysis (why ~7 km over 180 min for CHAMP vs SP3) + fixes
Setup verified CORRECT: CHAMP uses its real params (522 kg, 0.767 m^2, Cd 3.0 from
the catalog via sat.import), the seed-velocity fit is OFF (pure result), frame C.
Force-contribution test over 180 min at CHAMP altitude (RMS |dr| if a term is dropped):
  gravity 6->2 = 188 m ; drag = 83 m ; third-body = 5.6 m ; SRP = 5.0 m ; relativity = 1.4 m.
=> Among forces, GRAVITY DEGREE is by far the biggest lever; drag/3rd-body/SRP are small
   over this arc. No modeled force explains 7 km at degree 20, so the residual is
   dominated by either (a) gravity TRUNCATION (degree 20 at 340 km leaves km-level
   signal) or (b) the SEED VELOCITY.
Two fixes:
- **Seed velocity**: if the SP3 has no V records, gps_reference finite-differenced
  with `gradient`, whose 1-sided estimate at the FIRST point (the seed) has a ~130 m/s
  error at 30 s -> dominates everything. Replaced with a 4th-order stencil (incl.
  high-order one-sided ends): seed error 132 m/s -> 0.002 m/s. gps_reference now also
  PRINTS whether velocity came from SP3 V records or was finite-differenced.
- **Gravity degree**: the precision validation scripts now default to 70x70 EGM2008
  (was 20); at CHAMP altitude degree 20 truncation is a large part of the residual.
Recommendation to the user: re-run; the "[ref] SP3 velocity: ..." line tells you if
the seed was FD'd (now fixed) or from V records (then raise gravity degree to 70-120).

## GLOBAL seed rule: every propagation starts exactly at the measured state
Verified end-to-end that the SP3 path is already exact: SP3 record -> ECI seed ->
op.propagate initial state match to machine precision (|dr0|=|dv0|=0, RTN residual
at t=0 = 0). The ECEF->ECI law is correct (r: Ct*r_ecef ; v: Ct*v_ecef + omega x r).
The remaining gap was HOW velocity is obtained when the measurement has none:
- **TU Delft (position-only) had the same seed bug as SP3-without-V-records.**
  track_reference used a 2nd-order central difference with a ONE-SIDED FIRST-ORDER
  estimate at the FIRST point -- but the first point IS the seed. Leading error
  (h/2)*|r''| ~= 44 m/s at 10 s (and ~130 m/s at 30 s). That single number dominates
  the whole residual (radial oscillation + secular along-track growth).
- **Fix: new shared `validation.fd_velocity`** -- 4th-order central in the interior
  AND 4th-order ONE-SIDED at both ends, so the seed is mm/s accurate. Now used by
  BOTH `validation.gps_reference` (SP3 without V records) and
  `validation.track_reference` (TU Delft). Measured seed error:
     280/340/450 km at 10 s -> ~3e-5 m/s ; at 30 s -> ~2e-3 m/s
  (was 44 m/s and 130 m/s respectively).
  Non-uniform or <5-sample series fall back to `gradient` WITH A WARNING (the seed
  then carries more error) rather than silently.
- TLE path needs no FD: tle2eci produces r,v analytically from mean elements.
- **New regression test `08_test/test_seed.m`** (in run_all_tests) locks this in:
  it checks fd_velocity seed accuracy at 280/340/450 km at 10 s and 30 s, and that
  the propagator's initial state equals the reference's first state to machine
  precision. Any future regression of the seed rule now fails the suite.

## SP3 seed: use the FILE's own r AND v (priority), report honestly
The earlier "seed is exact (0.00e+00)" claim was verified on a SYNTHETIC SP3 that
was written WITH V records -- it did not prove anything about a real file that may
be position-only. Corrected:
- **read_sp3 now reads the SP3 HEADER flag** (`#dV` = position+velocity, `#dP` =
  position only) and counts P and V records, so what the file actually provides is
  known rather than assumed.
- **V-record parsing hardened**: skips a V line with no preceding P, skips a
  malformed/short V record (leaves NaN), and `hasVel` is now true ONLY if every
  epoch has a usable V (partial V is not silently mixed with FD).
  Warns if the header declares #dV but no V records parse, or if V covers only
  part of the epochs.
- **gps_reference now states the seed source explicitly**, e.g.
    `[ref] SP3 L09: r AND v taken from the FILE (1681 V records, dm/s)`
    `[ref] SP3 L09: NO usable V records (header #dP, 0/1681) -> velocity DERIVED ... 4th-order FD`
- **New `06_validation/realsat/verify_seed.m`** runs on the REAL file and prints:
  file contents (header flag, P/V counts), a SELF-CONSISTENCY cross-check of the
  file's own V records against a 4th-order FD of the file's own P records, the seed
  identity (|r0_prop-r0_ref|, |v0_prop-v0_ref| must be 0), and the da<->dv mapping
  so a residual plot can be read quantitatively.
Verified on both a position+velocity SP3 and a position-only SP3: the two seed
paths (file V records vs 4th-order FD of the same file's positions) agree to
0.005 m/s, so the seed is consistent either way.

## What each source actually provides (r and v) -- checked, and the fix
| source | position | velocity | seed |
|--------|----------|----------|------|
| SatChecker TLE | mean elements | mean elements | r,v ANALYTIC via tle2eci -- both present |
| SP3 (GFZ/AIUB/ESA) | P records | V records IF header is `#dV` | uses the FILE's r AND v when present |
| TU Delft track | alt/lat/lon | **NONE** (12-col product: date,time,tsys,alt,lon,lat,LST,arglat,density,densmean,flags) | v MUST be derived |
So two of three carry both; TU Delft is position-only BY DESIGN -- that is not a
bug to fix but a property to handle correctly.
**The real problem with deriving it**: differentiating a quantised/noisy track
amplifies position error by ~1/h, and a high-order FD stencil makes it WORSE
(bigger coefficients). Measured seed error at 340 km, 10 s sampling:
| track quality | 4th-order FD | least-squares fit (deg 5, window 21) |
|---|---|---|
| clean            | 0.00003 m/s | 0.00025 m/s |
| quantised 1 m    | 0.305 m/s   | **0.009 m/s** |
| quantised 10 m   | 2.823 m/s   | **0.434 m/s** |
| noise 2 m rms    | 2.635 m/s   | **0.197 m/s** |
A 2.8 m/s seed error is da = 2*a*dv/v ~ 4.9 km of semi-major axis -> exactly the
km-level residual with radial oscillation + along-track growth.
**Fix**: `validation.fd_velocity` now defaults to a sliding least-squares
polynomial fit (Savitzky-Golay, deg 5, window 21) -- exact for smooth motion AND
noise-suppressing, valid at the ENDS (the seed) by shifting the window and
evaluating the fitted derivative at the point's offset. Falls back to the
4th-order stencil / gradient (with warnings) for non-uniform or short series.

## Deep audit of the CHAMP 7.6 km residual -- what is and is NOT the cause
Driven by verify_seed on the real file: SP3 header `#dV`, 1681/1681 V records,
seed taken FROM the file, |r0_prop-r0_ref| = |v0_prop-v0_ref| = 0. So the seed is
exact and the error is created during propagation. Audited each candidate:
- **GRAVITY IS CORRECT.** Cross-checked grav.sphericalHarmonic (Cunningham,
  denormalised) against grav.potential (INDEPENDENT normalised-Legendre code) with
  a full TESSERAL field: relative acceleration error ~1e-9 at degrees 4..70.
  The "degree 70 tesserals may lose precision" warning was over-conservative and
  is raised to degree 120. NOTE: test_gravity only exercised the default
  ZONAL-only field, so tesserals had never been tested -- now they are.
- **DENSE OUTPUT WAS A REAL DEFECT (fixed).** The adaptive high-order integrators
  are accurate at their NODES, but the dense output between nodes is a CUBIC
  Hermite whose error grows like h^4. rk78 at rtol=1e-11 takes long steps on a
  smooth orbit, so the values AT THE REQUESTED OUTPUT TIMES were wrong. Two-body
  vs an ANALYTIC truth, outputs every 30 s: no cap -> RMS 14.0 / max 27.3 m ;
  hmax=120 s -> 5.9 m ; hmax=30 s -> 0.13 m. op.propagate now caps hmax at the
  output spacing whenever output times are requested -> 27.3 m becomes 0.13 m.
- **PIPELINE IS EXACT (closed-loop proof).** Wrote a known J2 truth out as an SP3
  with V records, read it back through read_sp3/gps_reference, re-seeded and
  re-propagated with the SAME model: SP3 round-trip position 0.6 mm, velocity
  exactly 0 (so the ECEF->ECI `Ct*v_ecef + omega x r` transform is right), and the
  end-to-end residual is **0.023 m over 2 h**. Locked in as a regression in
  08_test/test_seed.m.
- Drag formula verified: `v_rel = v_eci - omega x r_eci` (proper corotation);
  NRLMSISE adapter takes rho(6) = total mass density (correct). `ctx.v_ecef` is
  `C*v_eci` (NOT the ECEF velocity) but is unused by any force -- harmless.
CONCLUSION: the pipeline, seed, gravity and force formulas are sound, so a
residual of km scale is a FORCE-MODEL/configuration gap, not a code bug. New
`06_validation/realsat/check_forces.m` prints the force magnitudes at the seed and
a drop-one test (how far the orbit moves if each term is removed), which localises
it on the real data in one run.

## Per-integrator + per-source pipeline verification
Asked: does the pipeline hold for EVERY integrator, and are the TU Delft and TLE
paths sound? Verified by closed loop (write a known J2 truth as an SP3 with V
records, read it back, re-seed, re-propagate with the SAME model -> residual must
be ~0). RMS |3D|:
  rk4 0.0236 | nystrom4 0.0236 | rk6luther 0.0236 | gaussJackson8 0.0236 |
  rk45 0.0231 | rk78 0.0253   (all metres, i.e. ~2 cm)
All sit at the SP3 file's own write precision (position %14.6f km = mm), so every
integrator is sound and **changing integrator does NOT change the residual** --
if a real residual moves when you switch integrator, that is a red flag.
Fixed-step methods are safe because integ.run's default h (span/1000, capped at
30 s) is finer than the output grid. CAUTION: if you SET cfg.integrator.h COARSER
than the output spacing you reintroduce dense-output error -- keep h <= output dt.
Locked in: 08_test/test_seed.m now runs the closed loop for all six native methods.
Other two reference paths audited the same way:
- **TU Delft** (geodetic -> ECEF -> ECI, the track_reference math): round-trip
  |dr| = 0.000000 m -- exact.
- **TLE** (mean elements -> r,v via tle2eci): round-trip |dr| = 0.000 m,
  |dv| = 0.00000 m/s -- exact.
Side benefit of the dense-output (hmax) fix: test_energy relative drift over 10
orbits improved from 7e-12 to **9e-15**.
All three sources and all integrators verified: residuals are physics, not plumbing.

## ODE SUITE (ode45/ode78/ode89/ode113) -- real divergence bug, FIXED
You were right to ask. The ODE-suite wrapper returned the SOLVER'S NODES, which
op.propagate then resampled with a CUBIC HERMITE to reach the output times. That
throws away MATLAB's own high-order interpolant and injects the h^4 dense-output
error -- and it is WORST exactly for the solvers you use: ode113 is a variable-
order Adams method that deliberately takes very long steps, and ode89 likewise.
Their nodes are sparse BY DESIGN, so cubic resampling of them diverges.
**Fix**: integ.odesuite now receives the output times and passes them to the
solver as the tspan vector, so MATLAB reports the state AT those times using its
OWN interpolant (the one deval uses). integ.run routes tout to the ode wrappers.
Verified: ode45 vs an ANALYTIC two-body, outputs every 30 s -> RMS 0.075 m,
max 0.128 m (identical to the fixed native path); closed loop 0.0236 m.
test_seed now runs the closed loop for rk4, nystrom4, rk6luther, gaussJackson8,
rk45, rk78, ode45, ode78, ode89, ode113 (the last three SKIP cleanly where the
MATLAB release lacks them). All PASS at ~2 cm.
=> Switching integrator does not change the residual; if it ever does, that is a
   red flag, not physics.

## ROOT CAUSE of the CHAMP ~7.6 km residual: omega assumed along the ECI z-axis
Decisive evidence from the user: swapping the integrator (rk78 -> ode45) AND the
atmosphere (NRLMSISE -> DTM2020) changed the residual by 0.1% (7650 -> 7641 m).
So it was neither the integrator nor drag. The one thing every closed-loop test
had missed: those tests ran with frame 'gmst'; the user runs frame 'C'.
**The bug**: `gps_reference` built the inertial seed velocity as
    v_eci = Ct*v_ecef + omega x r_eci,  with omega = [0;0;omega_earth]
which assumes the Earth's rotation vector lies along the ECI z-axis. That is true
for the 'gmst' (TEME-like) build -- which is exactly why the closed loop passed --
but the IAU 2006/2000A builds ('B','C') rotate about the **CIP**, which by 2008
has precessed ~168 arcsec (8.14e-4 rad) off the GCRF z-axis. The resulting seed
velocity error is
    dv = omega * theta * r = 7.29e-5 * 8.14e-4 * 6.72e6 = **0.399 m/s**
matching the 0.4 m/s independently inferred from the observed radial amplitude.
Consequences (all observed): da = 2a*dv/v ~ 700 m -> radial oscillation ~1400 m at
the orbital period, along-track growth ~13 km over 3 h, cross-track small, residual
identical for any integrator or atmosphere, and seed identity still exactly 0
(because the propagation is seeded FROM the wrong ref.v -- position was never wrong).
**Fix (frame-agnostic)**: from r_eci = Ct(t)*r_ecef it follows exactly that
    v_eci = Ct*v_ecef + (dCt/dt)*r_ecef
`gps_reference` now forms dCt/dt by central difference of the ECEF->ECI rotation
(1 s step; truncation ~1e-14 rad/s, i.e. sub-um/s). This captures the true rotation
vector -- precession-nutation, ERA rate and polar motion -- for ANY build, so 'gmst',
'B' and 'C' are all correct with no special-casing.
NOTE the same reasoning corrected `test_seed`'s SP3 writer: v_ecef is BY DEFINITION
d/dt[C(t) r_eci(t)] = dC/dt*r_eci + C*v_eci, not C*(v_eci - omega x r_eci). With the
exact definition the closed loop returns to ~2 cm for every integrator
(rk4/nystrom4/rk6luther/gaussJackson8/rk45/rk78/ode45; ode78/89/113 skip where the
release lacks them).
LESSON: a validation harness that only exercises the simplified frame cannot see a
frame-dependent bug. The closed loop should also be run with frame 'C' on a machine
with EOP access.

## Sweep: every place that assumed omega = [0;0;omega_earth]
The CIP bug was one instance of a CLASS. Swept the codebase for the assumption
"the Earth's rotation vector lies along the ECI z-axis" and interfaced them all
through one true rotation vector:
1. **validation.gps_reference** (the SP3 seed) -- FIXED via v_eci = Ct*v_ecef +
   (dCt/dt)*r_ecef. This was the 0.4 m/s bug. **Builds A and B are covered by the
   SAME code**: `frame` is passed straight through to frames.eci2ecef inside the
   dCt/dt helper, so gmst/A/B/C are all exact with no special-casing.
2. **op.buildWorld** -- now computes the TRUE Earth rate ONCE per run:
   from r_eci = Ct*r_ecef it follows [omega]x = (dCt/dt)*C, so
   omega = [M(3,2); M(1,3); M(2,1)]. Stored as W.omega_eci. Computing it once is
   legitimate: the CIP moves ~20 arcsec/YEAR, i.e. it is fixed over any arc.
   Falls back to [0;0;omega_earth] offline. Verified: for 'gmst' it returns
   [0;0;7.2921e-5] with 0.000 arcsec tilt (as it must).
3. **forces.drag** (both the cannonball and the multi-species path via
   drag.relVelocity) -- the co-rotating atmosphere now uses ctx.omega_eci instead
   of [0;0;omega]. Small for drag (0.4 m/s out of ~7.6 km/s) but the same class.
4. **op.accel ctx.v_ecef WAS A LANDMINE** -- it was `C*v_eci`, which is the
   INERTIAL velocity expressed in ECEF axes, NOT the Earth-fixed velocity (they
   differ by omega x r ~ 490 m/s). Nothing consumed it, so it did no harm, but any
   future use would have been a 490 m/s error. Now correct:
   ctx.v_ecef = C*(v_eci - omega_eci x r_eci).
5. **validation.track_reference** -- immune by construction: it differentiates the
   reconstructed ECI positions, so no rotation vector enters.
6. **validation.tle2eci** -- immune: r,v come analytically from mean elements.

### Finite-difference step: chosen by error analysis, not by feel
dCt/dt error = round-off ~A/d + truncation ~(d^2/6)*omega^3. The round-off is real:
the GMST angle is a polynomial in a ~2.45e6 Julian date, so a double loses ~2e-5 s
of time resolution. Measured A ~ 3.9e-10 rad/s at d=1 s; d=120 s gave 9.4e-10
(truncation-dominated). Optimum d = (A/(2B))^(1/3) ~ 15 s. Verified against the
exact GMST rate (1.00273790935 rev/day):
    d=1 s   -> 3.9e-10 rad/s (2.6e-3 m/s at LEO)
    d=15 s  -> **1.9e-11 rad/s (1.3e-4 m/s)**   <- 20x better
    d=120 s -> 9.4e-10 rad/s (6.3e-3 m/s)
Both gps_reference and buildWorld use d=15 s. NOTE: test_seed's SP3 writer must use
the SAME convention -- v_ecef is BY DEFINITION dC/dt*r_eci + C*v_eci; when the
writer and reader disagreed on the step the closed loop read 1.7 m, which is itself
a useful sanity signal.
Closed loop after all of the above: 0.023 m for rk4/nystrom4/rk6luther/
gaussJackson8/rk45/rk78/ode45 (ode78/89/113 skip where unavailable). User reports
gmst now converging to 30-60 m over 3 h -- consistent with a real force-model gap
rather than a seed defect.

## RESULT of the CIP fix + remaining SP3 source gaps
CHAMP 2008-06-01, frame C, EGM2008 70x70 + drag + 3rd body + SRP + relativity,
seeded from the SP3's own r,v:
    BEFORE (omega assumed along ECI z): |3D| 7650 m  (radial 840, along 7595)
    AFTER  (true CIP rotation vector) : **|3D| 5.16 m** (radial 0.95, along 5.06,
                                         cross 0.40, velocity RMS 0.006 m/s)
i.e. a 1500x improvement, and now at genuine precise-propagator quality.
CHAMP 2010-06-01 gives |3D| 68 m (radial 10.0, along 67.5, cross 0.26). That is
almost certainly PHYSICS, not a defect: by 2010 CHAMP had decayed to a much lower
altitude and F10.7 had risen 69 -> 75, so drag is far stronger; the residual is
~99% along-track, which is the drag/energy signature. Use check_forces.m to
confirm the drag share, and expect a Cd/density-model gap to dominate there.

### SP3 source gaps closed (GOCE, Swarm)
- **"no built-in SP3 URL for Swarm"**: the URL builder only matched 'SWARM-A/B/C',
  so SAT='Swarm' silently fell through. Added `normaliseSat`: 'Swarm A',
  'swarm-a', 'SwarmA', 'SW-A' -> 'SWARM-A' (and GRACE-FO variants). A BARE
  'SWARM' now raises a clear error -- Swarm is a 3-satellite constellation and
  each spacecraft has its own SP3, so silently picking one would be wrong.
- **"no built-in SP3 URL for GOCE"**: there was no builder at all. Added the ESA
  GOCE Virtual Archive PSO URLs (GO_CONS_SST_PSO_2__<d0>T235942_<d1>T235942_<cc>.TGZ,
  counters 0001/0002 tried in order).
- **GOCE now routes through the SAME stored ESA credential as Swarm** (both sit
  behind the free ESA EO Sign-In), so `data.credentials('set','esa',...)` once
  serves both. Registry updated: GOCE orbit is `esa_goce`/login, not `aiub_sp3`/open.
- The "no source" error now lists exactly which satellites are wired, the exact
  spelling required, the one-line credential command, and the three overrides
  (refSource='tudelft', opts.file, opts.url).
CAVEAT (honest): the GOCE/Swarm URL patterns are built from the documented naming
conventions and could not be exercised against the live gated servers from here.
If a filename differs, the error now names the exact URL tried -- send it and it is
a one-line fix. CHAMP's deterministic URL is confirmed working, and note the 2010
run already exercised the listing fallback (the file ended ...120030 not ...120000).

## Restructure: the OPEN template (nothing chosen behind your back)
Complaint: the scripts are too closed -- step-size selection, parameter choice and
data handling all happen invisibly in the engine. Inventory of what WAS hidden:
  integ.adopt : rtol 1e-9, atol 1e-12, hmax span/2, hmin 1e-6, h0 min(span/100,10),
                facmin 0.2, facmax 5, maxsteps 2e6
  integ.run   : fixed-step h = min(30, span/1000)   (warned, but not settable per-run)
  op.propagate: hmax silently capped at the output spacing (the dense-output guard)
  track_reference : build 'gmst', thin 30
  gps/track_reference : fd_velocity called with NO options at all
  fd_velocity : method 'auto', deg 5, window auto (~210 s span)
  sat.import  : mass/area/Cd taken from the catalog with no override path
New `TEMPLATE_validation.m` surfaces ALL of it: 9 sections (CASE, DATA/SOURCE,
SPACECRAFT, FORCES, INTEGRATOR, REFERENCE, FRAME/TIME, OUTPUT/FIGURES, RUN), every
knob written out AT its default, with what it does and what breaks if you change it.
Deleting a line falls back to the engine default (also stated), so it prunes down
to only what you care about. The short scripts remain the quick path.
Plumbing added so the template is not a lie (these were previously IGNORED):
- `opts.fdVelocity` -> forwarded by gps_reference AND track_reference into
  validation.fd_velocity (method/deg/window).
- `opts.spacecraft` -> MERGED on top of the catalog entry, so overriding mass alone
  keeps the catalogued Aref/Cd. Verified: mass 522->999, Cd 3.0->2.2, Aref kept.
  Prints '[cfg] spacecraft override: ...' so it is never silent.
- `opts.spaceweather` -> accepted alongside the legacy 'manualSW'.
- `opts.output_dt` -> custom output grid, but SNAPPED to the reference epochs,
  because the residual pairs sol.t(k) with ref.t(k); inventing times the reference
  cannot be compared at would silently corrupt the pairing. Warns if the requested
  step does not land on the reference grid.
Two hidden defaults that had already bitten us are now documented AT the knob:
  * fixed-step h must stay <= the output spacing (else dense output degrades);
  * fd_velocity must stay on the least-squares fit for quantised tracks
    (bare FD -> 0.31 m/s seed error on a 1 m-quantised track vs 0.009 m/s).

## Openness pass: satellite list, decisions report, propagation example
1. **`sat.list`** — prints every satellite the toolbox accepts with NORAD, altitude,
   VALID DATE RANGE, TU Delft availability, SP3 portal and SP3 access, read live
   from sat_catalog.csv so it can never drift from what the fetchers take. The same
   table is pasted into EXAMPLE_propagate_satellite.m as a copy-paste comment block.
   Date ranges matter: a DATE outside them 404s, which previously looked like a bug.
2. **`config.report(cfg, W, sol[, file])`** — the answer to "what got decided at the
   back?". Prints/saves EVERY value in force and marks each `[set]` (you chose it)
   or `(default)` (the engine chose it), covering: spacecraft mass/Aref/Cd/Cr; every
   force on/off + model + degree/order + atmosphere; requested vs LOADED gravity
   degree with an explicit `<-- TRUNCATED from N!` flag; the full integrator step
   control (rtol/atol/h/h0/hmin/hmax/facmin/facmax/maxsteps) with the defaults
   spelled out; frame build; the recovered Earth-rotation vector INCLUDING its tilt
   off the z-axis in arcsec (the CIP offset that caused the 7.6 km bug); the output
   grid; and what the integrator actually did (nodes, wall time).
   Verified live: it caught `field=default, requested degree=70 -> LOADED 6
   <-- TRUNCATED` -- exactly the silent trap that used to make degree sweeps no-ops.
3. **`EXAMPLE_propagate_satellite.m`** — end-to-end PROPAGATION ONLY (no comparison):
   pick a satellite + date, seed from SP3/TU Delft/TLE/manual, propagate, and get
   altitude/speed/3D-trajectory/sma-change plots plus a saved decisions.txt and
   propagation.mat. Every knob is on the surface at its default with the consequence
   documented next to it, including the two that already bit us (fixed-step h must
   stay <= the output spacing; fd_velocity must stay on the LSQ fit for quantised
   tracks). SEED_FROM='manual' allows an arbitrary state.
4. **Naming-change audit**: looped ALL 11 catalog names through data.precise_orbit --
   11 resolve cleanly, 0 unexpected failures (only the expected offline outcomes:
   needs-credentials / no-SP3-for-this-satellite / no-network). sat.catalog still
   accepts CHAMP, GOCE, SWARM-A, GRACE-FO-1, ISS. Regression suite: test_gravity,
   test_energy, test_seed (fd_velocity, quantised track, seed identity, closed loop
   over all integrators) all PASS.

## Credentials folder + web-source registry (open vs login)
- **`05_data/credentials/web_sources.csv`** — EVERY website the toolbox downloads
  from (17): source name, site key, access, product, base URL, sign-up URL, which
  function uses it, notes. Excel-readable. Split: **14 OPEN, 3 LOGIN** (and the two
  ESA entries share ONE account, so it is really 2 accounts, and Earthdata is only
  needed if you enable ERP).
- **`data.sources()`** — prints them grouped OPEN vs LOGIN with the sign-up links;
  `data.sources('open'|'login')` filters; returns a struct array if asked. Reads the
  SAME csv the setup wizard walks, so docs and behaviour cannot drift apart.
- **`data.credentials('setup')`** — interactive: walks only the LOGIN sites, shows
  what each is used for and where to sign up, ENTER skips, and it never asks about
  the 14 open ones because there is nothing to ask.
- **The store moved into its own folder**: `<data.root>/credentials/credentials.json`,
  i.e. outside the shipped tree so it is never zipped/shared. An existing
  `<data.root>/credentials.json` is migrated automatically and silently on first use.
- `credentials.template.json` moved into the same folder next to a README that says
  in the first table that it is a FORMAT EXAMPLE ONLY and is never read — this is the
  file a user naturally edits and then wonders why nothing loads.
- The "no credentials" error now leads with `data.credentials('setup')` and states
  that only 3 of 17 sources need anything.

## GOCE download failure: my bug (ftp:// forced onto an HTTPS host)
User error: "authenticated download failed for
https://goce-ds.eo.esa.int/oads/data/GOCE_PSO/GO_CONS_SST_PSO_2__...TGZ (site 'esa')"
-- with valid credentials. The error was PAST the credential check, so the login was
never the problem.
**Cause (mine)**: data.fetch_auth's 'esa' branch did
    u = regexprep(url,'^https?://','ftp://');   % force ftp endpoint
That branch was written for swarm-diss.eo.esa.int, which has an FTP-over-TLS
endpoint. When I later routed GOCE through the same site key ('esa'), every GOCE
HTTPS URL got rewritten to ftp://goce-ds.eo.esa.int/... -- a server that does not
exist. The real link was never tried.
**Fix**: route by HOST, not by site key. swarm-diss -> FTP-over-TLS; anything else
on the 'esa' account (goce-ds / OADS) -> plain HTTPS with -L --location-trusted and
a cookie jar, since EO Sign-In bounces through an identity provider.
**Honest limits, stated in the code and the error**: the ESA OADS is designed around
the browser EO Sign-In flow, and its file names carry a per-file version counter, so
the built-in GOCE URL remains a best-effort guess that may still not resolve. The
error now says credentials were found, lists the three likely causes (wrong file
name / OADS wants the browser flow / collection not activated) and points at the
tested path (download once in a browser -> opts.file).
**GOCE is the one satellite where the no-login route is better anyway**: TU Delft
carries GOCE's GPS-derived track OPENLY (data_availability.csv: track=tudelft/open),
so validate_tudelft.m needs no ESA account at all. precise_orbit now says so at the
GOCE URL builder.

## GOCE SP3: the actual protocol (verified against ESA Earth Online)
Two of my errors, both now fixed:
1. **Wrong protocol.** fetch_auth rewrote https->ftp:// for site 'esa'. ESA's page
   states plainly: "Open ftps://goce-ds.eo.esa.int/ via an FTP client, using
   IMPLICIT FTP over TLS. Log in with an active EO Sign In account." Implicit FTPS
   is port 990; plain ftp:// is port 21 -- which is EXACTLY the user's error
   ("Failed to connect to goce-ds.eo.esa.int port 21"). fetch_auth now passes an
   ftps:// URL through untouched and lets curl speak implicit TLS.
2. **Wrong baseline.** I guessed counters 0001/0002. ESA states SST_PSO_2 latest
   baseline is **_0201** (0002/0001 remain for some days). Now tried newest-first.
Also recorded: the **PSO collection covers 2009-09-01..2012-07-31**, which is
SHORTER than the GOCE mission (2009-03-17..2013-11-11). A date inside the mission
but outside the PSO window will never resolve. (The user's 2010-06-01 is fine.)
**New `data.browse(site, url)`** -- lists a gated archive directory with the stored
credentials, e.g. data.browse('esa','ftps://goce-ds.eo.esa.int/'). The built-in URLs
for gated archives are best-effort guesses from published naming conventions and
cannot be tested from this sandbox; browse turns "guess again" into "look it up",
and whatever is found can be passed straight back via opts.url / SP3_FILE.

## curl 530 on Windows: the password was never actually sent (REAL BUG)
Symptom: `data.browse('esa','ftps://goce-ds.eo.esa.int/')` -> `curl: (67) Access
denied: 530` with correct credentials. Note the progress: port 990 + implicit TLS
were REACHED (so the ftps:// protocol fix was right); the LOGIN was refused.
**Cause (mine)**: every authenticated curl call was built as
    curl -u "user:$POP_CURL_PW" ...
`$POP_CURL_PW` is SHELL syntax. MATLAB's system() runs **cmd.exe on Windows**, which
does NOT expand `$VAR` -- so curl received the LITERAL string `$POP_CURL_PW` as the
password and the server correctly answered 530. This affected EVERY gated site
(esa, gfz, earthdata) on Windows; it silently worked on Linux/macOS, which is why it
was never caught here. (`%VAR%` would expand on Windows but not on sh -- there is no
portable inline form, so the whole approach was wrong.)
**Fix**: new `data.curlcfg(user, pass, extra)` writes a temporary curl `-K` config
file (`user = "u:p"`), expanded by CURL ITSELF -> identical on Windows/macOS/Linux.
Quoting makes special characters (@ : # spaces ") safe. All call sites in
fetch_auth (ftps, swarm-diss, OADS/https, gfz, earthdata) and browse now use it;
zero `$POP_CURL_PW` uses remain in any command line.
**Security bonus**: the password no longer appears on the command line, so it cannot
be read from the process list, shell history or a MATLAB error dump. The temp config
is chmod'd to the owner and removed by an onCleanup.
browse's error now decodes the status: 530 = reached but login refused -> (1) is the
stored password CURRENT (did you rotate it?), (2) the EO Sign In USERNAME may not be
your e-mail, (3) collection not activated; while curl 7/timeout = wrong protocol.

## GOCE: stop guessing paths -- discover them (curl 9 fix)
The curl -K config fix WORKED: `data.browse('esa','ftps://goce-ds.eo.esa.int/')` now
authenticates and lists. That listing settled the last question:
    dr-x------ ... GOCE          <-- the root holds /GOCE/
    -r-------- ... readme.txt
i.e. NOT the /GOCE_Level_2/ implied by ESA's catalogue page, which is why every
guessed URL died with `curl (9) Server denied you to change to the given directory`.
**Fix**: new `data.find_esa_file(site, root, pattern[, maxDepth])` -- breadth-first
walk of the FTPS tree that LISTS directories and regex-matches the file, instead of
hard-coding a path. Subdirectories are ranked so obviously-relevant names (SST_PSO,
ORBIT, LEVEL, the year in question) are visited first, so the file is normally found
in a handful of listings; depth is capped (default 4) and unreadable branches are
skipped. precise_orbit now DISCOVERS the GOCE PSO url for the requested date
(pattern `^GO_CONS_SST_PSO_2__<d0>T\d{6}_<d1>T\d{6}_\d{4}\.TGZ$`, so any baseline
counter matches -- 0201/0002/0001 alike) and only falls back to guessed paths if the
walk fails. The listing parser was verified against the EXACT lines the user's server
returned (dr-x/-r-- unix style -> GOCE=dir, readme.txt=file).
Note the walk needs credentials, which now work on Windows thanks to the curlcfg fix;
the two bugs were stacked, which is why this took several rounds.

## ESA dissemination layout: found the CONVENTION (GOCE + Swarm)
ESA's own FTPS scripting FAQ gives real worked examples:
    /MERIS/MER_FRS_1P/2/2002/0010
    /SMOS/AUX_Dynamic/AUX_ECMWF_/2010/06/01/SM_REPR_AUX_ECMWF__20100601T232050_...zip
=> convention: **/<MISSION>/<PRODUCT>/[version]/<YYYY>/<MM>/<DD>/<file>**
=> and the **PATH DATE IS THE GRANULE'S START DATE** (the SMOS file starting
   20100601 sits in /2010/06/01/). A GOCE PSO granule for day D starts at
   (D-1)T235942, so it lives under **<D-1>**'s folder -- e.g. 2010-06-01 -> /2010/05/31/.
   That subtlety would have broken a naive /YYYY/MM/DD built from the requested day.
The user's own root listing confirmed the mission level is **/GOCE/** (not the
/GOCE_Level_2/ the catalogue page implies).
GOCE candidates now tried, newest baseline first:
    ftps://goce-ds.eo.esa.int/GOCE/SST_PSO_2/2010/05/31/GO_CONS_SST_PSO_2__20100531T235942_20100601T235942_0201.TGZ
plus /GOCE/GOCE_Level_2/SST_PSO_2/..., a /2/ version segment variant, and shallower
parents -- then the WALKER as the final authority.
Swarm moved to the same family: the old https '?do=download&file=...' URL was the
web UI, not a scriptable endpoint. Now ftps://swarm-diss.eo.esa.int with
Level2daily/{Entire_mission_data,Latest_baselines}/POD/RD/Sat_<A|B|C>/... plus walker.
find_esa_file now ranks EXACT <YYYY>/<MM>/<DD> directory names (+6 each) so the walk
descends the date tree directly instead of breadth-firsting the whole archive, and
maxDepth was raised to 6 to reach mission/product/version/Y/M/D.
HONEST STATUS: the convention and the /GOCE/ root are CONFIRMED; the GOCE-specific
subtree below /GOCE/ is NOT (this sandbox cannot reach ESA, and it needs the user's
EO Sign In). That is exactly why the walker exists and why a FOUND url overrides
every guess.

## "It's taking too long" -- my ordering bug, plus no timeouts
Cause (mine): I put the archive WALK inside besteffortURLs, so it ran BEFORE the
direct candidate URLs. Every listing is a TLS round-trip, and the walk was
breadth-first over the whole /GOCE/ tree to depth 6 -> minutes of crawling before a
single obvious URL was ever tried. Also, no curl timeout existed anywhere, so one
stalled listing could hang MATLAB indefinitely.
Fixes:
- **Direct candidates first.** besteffortURLs now only builds URLs (one request
  each, and the ESA-convention path should hit). The walk moved to `walkFallback`,
  invoked ONLY after every direct URL has failed.
- **curl timeouts everywhere**: connect-timeout 15 s, max-time 60 s.
- **Hard search budget** in find_esa_file: max 40 listings AND 120 s wall clock,
  then it stops and tells you which directory to inspect by hand.
- **Only the top 3 ranked children per level** are followed. rankDirs scores an
  exact YYYY / MM / DD folder (+6 each) or a product-name match (+3) far above the
  rest, so the walk goes almost straight down the date tree instead of exploring the
  archive.
- **Progress is printed** (`[esa] listing (3/40): ...`) so a slow search is visibly
  making progress instead of looking hung.
- On success it prints `[esa] FOUND: <url>  <-- paste this back so it can be
  hard-wired`, which is how the guess gets retired permanently.

## Full sweep of the session's changes -- two more real bugs found
1. **GATED PATH TRIED ONLY ONE URL (major).** precise_orbit's gated branch did
       cand = besteffortURLs(...); gurl = cand{1};
   i.e. it took the FIRST candidate and silently discarded the rest. All the
   carefully built fallbacks (baselines 0201/0002/0001, alternative directory
   layouts, shallower parents) were never tried -- one wrong guess killed the fetch.
   This is why GOCE failed instantly on a single path. FIXED: it now loops over
   EVERY candidate, prints `[esa] try k/N: <url>` so the search is visible, keeps the
   last error, and only then falls through to the archive walk. Confirmed counts:
   GOCE 15 candidates, SWARM-A 12, CHAMP 1 (deterministic).
   Also added CACHE-FIRST to the gated branch (it fetched even when the .raw already
   existed) with an opts.force override.
2. **`getf` used but never defined in precise_orbit.** My cache-first check called
   getf(opts,'force',false); that helper does not exist in this file, so it would
   have thrown at runtime the moment a gated fetch ran. Replaced with an inline
   isfield test. A sweep of every file touched this session (precise_orbit,
   fetch_auth, browse, find_esa_file, curlcfg, sources, config.report, sat.list) now
   reports NO local helper used-but-undefined.
Verification after all of it: 15/15 changed functions parse; all 11 catalog
satellites route to an EXPECTED outcome (0 unexpected); test_gravity, test_energy
and test_seed (fd_velocity across altitudes/rates, 1 m-quantised track, seed
identity 0.00e+00, closed loop 0.0236 m on every integrator) all PASS.
Swarm first candidate:
  ftps://swarm-diss.eo.esa.int/Level2daily/Entire_mission_data/POD/RD/Sat_A/2010/06/SW_OPER_SP3ACOM_2__20100601T000000_20100601T235930_0202.ZIP
GOCE first candidate:
  ftps://goce-ds.eo.esa.int/GOCE/SST_PSO_2/2010/05/31/GO_CONS_SST_PSO_2__20100531T235942_20100601T235942_0201.TGZ

## GOCE path SOLVED by the walk: the product dir has a TRAILING UNDERSCORE
The user's walk output decoded the archive:
    listing  3 : /GOCE/GOCE_Level_2/
    listing  9 : /GOCE/GOCE_Level_2/SST_PSO_2_/     <-- trailing underscore!
    listing 24 : /GOCE/GOCE_Level_2/SST_PSO_2_/2010/
**RULE: ESA pads product-type codes to 10 characters with underscores** --
SST_PSO_2_, EGG_NOM_2_, SST_AUX_2_, EGM_GCF_2_ (SST_NOM_1B, ACC_DF1_1B already fit).
ESA's own SMOS example had it too (AUX_ECMWF_) and I read straight past it. Every
guessed URL used SST_PSO_2 (no underscore) -> curl 9 on all 15.
Hard-wired now, newest baseline first:
    ftps://goce-ds.eo.esa.int/GOCE/GOCE_Level_2/SST_PSO_2_/2010/05/31/GO_CONS_SST_PSO_2__20100531T235942_20100601T235942_0201.TGZ
(both the granule START day <D-1> and the stop day D are tried, plus YYYY/MM and the
product root, x baselines 0201/0002/0001 -> 15 candidates.)

## The walker was breadth-first -- that was why it burned its budget
It listed every sibling before descending, so it spent 33 listings on
GOCE_Level_1/*/2009..2011 and never reached GOCE_Level_2/SST_PSO_2_/2010/05.
**Rewritten DEPTH-FIRST**: score the children, walk INTO the best, back-track only on
a dead end. Scoring now encodes the two ESA facts learned here:
  * product code de-padded before comparison, so 'SST_PSO_2' matches 'SST_PSO_2_' (+20);
  * exact YYYY / MM / DD folder (+15 each), taken from the granule START date in the
    file pattern; Level_2 (+5); POD|ORBIT|SP3 (+4).
Once anything scores, zero-scoring siblings are skipped, so the walk stops wandering
into other products/years. The same route now needs ~6 listings instead of 40+.
Swarm gets the same treatment (ftps + walker from the server root); its first
candidate is
    ftps://swarm-diss.eo.esa.int/Level2daily/Entire_mission_data/POD/RD/Sat_A/2010/06/SW_OPER_SP3ACOM_2__20100601T000000_20100601T235930_0202.ZIP

## Audit of the UNPACK path -- 3 real bugs (GOCE/Swarm would have failed anyway)
I cannot fetch from ESA here: the sandbox proxy returns `x-deny-reason:
host_not_allowed` for goce-ds/swarm-diss/isdc (403). So the download itself is only
verifiable on the user's machine. What IS verifiable offline is everything AFTER the
download -- and that is where the next failures were waiting:
1. **No TAR support.** GOCE PSO is a `.TGZ` = gzip-wrapped TAR holding several
   products. maybeUnzip only knew gzip/zip: it gunzipped to `payload.sp3`, which is
   actually a TAR, matched its own `\.sp3$` test and handed BINARY TAR to read_sp3.
   Added: `ustar` magic detection at byte 257 -> untar -> pick the SP3 from the
   contents. `pickSP3` now verifies by CONTENT (first line '#' + P/V) rather than
   trusting a name, so decoys (QUALITY.dat, README.txt, manifest.xml) are ignored.
2. **unzip/untar path convention (this one bit twice).** They return ABSOLUTE paths
   on some releases and RELATIVE ('./x.sp3') on others. The exist() filter then
   dropped every extracted file, leaving nothing, so maybeUnzip returned the RAW
   archive -- which reads as "the download is broken" when the download was fine.
   Hit the TGZ path first, then the ZIP path again after the first fix. Now handled
   once, centrally, by `absPaths`.
3. **Silent catch.** The unpack failure was swallowed by a bare `catch` -> the raw
   file passed through with no explanation. Now warns with the actual reason.
VERIFIED offline against archives built to mimic the real products:
   ZIP (Swarm-like, sp3 + manifest.xml decoy) -> 61 epochs, hasVel=1  OK
   TGZ (GOCE-like, sp3 + QUALITY.dat + README decoys) -> 61 epochs, hasVel=1  OK
   plain .sp3 (CHAMP-like)                    -> 61 epochs, hasVel=1  OK
Note a false pass along the way: a tar written WITHOUT gzip appeared to work, because
read_sp3 skipped the 512-byte tar headers and parsed the embedded text. Only a
genuine gzip+tar exposed the bug -- worth remembering when testing archive handling.
Regression after all of it: test_gravity PASS, test_energy PASS, test_seed PASS,
routing 11/11 expected, 0 unexpected.

## Why ONLY CHAMP worked -- the honest tally
CHAMP is open (GFZ ISDC, plain HTTPS GET, deterministic filename), so ONE thing had
to be right and it was. GOCE/Swarm needed FIVE things right at once, and every one
of them was wrong. Each fix only exposed the next:
  1. credentials never reached curl on Windows -- `-u "user:$POP_CURL_PW"` is shell
     syntax; cmd.exe does not expand $VAR, so curl sent the literal text -> 530.
     FIXED via data.curlcfg (a curl -K config file, expanded by curl itself).
  2. wrong protocol -- https rewritten to ftp:// (port 21). ESA serves IMPLICIT
     FTPS on port 990 -> ftps://. FIXED (routed by host, ftps:// passed through).
  3. wrong GOCE path -- product dir is SST_PSO_2_ **with a trailing underscore**
     (ESA pads product codes to 10 chars). Proved by the user's own ftps listing:
     /GOCE/GOCE_Level_2/SST_PSO_2_/2010/. FIXED.
  4. wrong Swarm path -- the tree root segment is **swarm/** (lowercase). The web UI
     fragment #swarm%2FLevel1b%2FEntire_mission_data%2FGPSxNAV%2FSat_A decodes to
     swarm/Level1b/Entire_mission_data/GPSxNAV/Sat_A, so precise orbits are
     swarm/Level2daily/Entire_mission_data/POD/RD/Sat_<A|B|C>/<YYYY>/<MM>/.
     My URLs started at Level2daily/... and died at the first segment. FIXED.
  5. only ONE candidate was ever tried (`gurl = cand{1}`), so a single bad guess
     killed the fetch and all the fallbacks were dead code. FIXED (loops all).

## Archive unpacking was broken for BOTH gated products (found by test, not by luck)
Even a successful download would have failed:
- **GOCE .TGZ**: maybeUnzip handled gzip and zip but NOT tar. A .TGZ gunzips to a
  TAR; the code named it payload.sp3, matched its own '\.sp3$' test and handed
  BINARY TAR to read_sp3. Added tar detection (the 'ustar' magic at byte 257) +
  untar + a CONTENT-based SP3 pick (first line '#' + 'P'/'V'), so the real SP3 is
  chosen out of a multi-file PSO archive instead of a decoy.
- **Swarm .ZIP and the TGZ**: unzip/untar return ABSOLUTE paths on some releases and
  RELATIVE ('./x.sp3') on others. The exist() filter silently dropped every
  extracted file -> maybeUnzip returned the raw archive, which looks exactly like a
  broken download. Added `absPaths` to normalise both.
- The catch that swallowed all of this now WARNS with the real reason.
Verified end-to-end offline against genuine archives:
  ZIP (Swarm-like)  -> 61 epochs OK
  TGZ (GOCE-like)   -> 61 epochs OK   (decoy README/QUALITY files correctly ignored)
  plain .sp3        -> 61 epochs OK
HONEST LIMIT: this sandbox cannot reach ESA (proxy: x-deny-reason host_not_allowed),
so the DOWNLOAD itself is unverified from here; the unpack/parse chain after it now
is verified. Also worth noting: https://gssc.esa.int/portal/datasets hosts SP3 for
several missions and may be an OPEN alternative worth wiring next.

## Swarm: use the endpoint the WEB UI actually uses
The user can download manually in a browser -- so the browser's request is the
ground truth worth copying. The UI at swarm-diss.eo.esa.int is a front end whose
'#<path>' fragment IS the tree, and whose download button hits a PLAIN HTTPS
endpoint:
    https://swarm-diss.eo.esa.int/?do=download&file=<url-encoded path>
So Swarm now tries, in order:
  1. that HTTPS endpoint  (no FTPS, and for Swarm usually no login at all)
  2. the same file over implicit FTPS, with and without a YYYY/MM level
for baselines 0202/0201/0101 and both Entire_mission_data and Latest_baselines
-> 18 candidates. Ironically the ORIGINAL code used this endpoint; it failed only
because the path was wrong (missing the lowercase 'swarm/' root), and replacing it
with FTPS was solving the wrong problem.
Routing re-checked after the change: 11/11 catalog satellites expected, 0 unexpected.

## TU Delft 15.9 km: it was the FRAME, not the seed (my bug)
GOCE 2010-06-01 vs TU Delft gave |dr| 15.9 km, |dv| 18 m/s. Cause: when I generated
validate_tudelft.m from validate_sp3.m with a sed replace, I set **FRAME = 'gmst'**.
The TU Delft product is a REAL ITRF geodetic track (alt/lat/lon), so rotating it to
ECI with 'gmst' drops precession/nutation entirely. By 2010 the CIP sits ~208 arcsec
(1.01e-3 rad) off the GCRF z-axis:
    position error = 1.01e-3 * 6.63e6 = **6.7 km** at GOCE's radius
and it ROTATES WITH THE ORBIT, so the mismatch sweeps and grows to ~15 km over a 3 h
arc -- exactly the observed magnitude and the oscillating-in-all-three-axes shape.
The seed was never the problem here: the velocity is differentiated from the ECI
track, so it is immune to the omega/CIP seed bug fixed earlier; it simply inherited
the badly-rotated positions.
Fixes:
- validate_tudelft.m: FRAME = 'C', with a comment saying WHY it must stay 'C'.
- validation.track_reference: default build changed 'gmst' -> 'C' (it always
  converts a real ITRF track), and it now WARNS if anyone forces 'gmst', because
  that silently costs kilometres. 'gmst' stays available for synthetic/offline work,
  where it is self-consistent (the SP3 closed loop deliberately uses it).
Regression after the change: test_gravity PASS, test_energy PASS, test_seed PASS
(seed identity 0.00e+00, closed loop 0.0253 m rk78 / 0.0236 m ode45).
LESSON (twice now): 'gmst' is self-consistent for SYNTHETIC data and silently wrong
for REAL ITRF data. Both the CHAMP 7.6 km and this GOCE 15.9 km trace to that
distinction.

## TU Delft 9.4 km: track ROUNDING amplified by differentiation (the real cause)
After the frame fix (gmst -> C) GOCE/TU Delft went 15.9 km -> 9.4 km, so the frame
was ONE cause, not the only one. Inverting the remaining residual:
    along 9368 m over 3 h  ->  dv = 9368/(3t) = **0.289 m/s**
    => da = 2a*dv/v = 496 m  -> radial ~500 m (measured RMS 1029). CONSISTENT.
So the seed velocity was ~0.3 m/s off. Reproduced exactly: a track quantised to 5 m,
differentiated with the then-default LSQ fit (deg 5, 21-point / 210 s window), gives
**0.339 m/s** -- matching the inferred 0.289. TU Delft publishes metre-rounded
positions, and differentiating amplifies that by ~1/h; my window was far too short
to average it out. Measured at 265 km on a 5 m-quantised 10 s track:
    span  3.9% (w=21)  deg5 -> 0.339 m/s   <- old default
    span 18.7% (w=101) deg5 -> 1.007 m/s   <- long window, degree too low
    span 18.7% (w=101) deg7 -> **0.033 m/s** <- chosen
    span 29.9% (w=161) deg9 -> 0.021 m/s
FIX: fd_velocity now defaults to **degree 7** with the window auto-sized to a
**~1000 s span** (~18% of a LEO orbit; 101 points at 10 s, 33 at 30 s), capped at
121. Truncation is carried by the higher degree, noise by the longer window.
Result on a 5 m-quantised track: 0.339 -> **0.026 m/s at 255 km** (13x better);
clean-track accuracy stays ~0.003 m/s.
test_seed now checks BOTH 1 m and 5 m quantisation (0.0136 / 0.0386 m/s, tol 0.05).
Regression: test_gravity PASS, test_energy PASS, test_seed PASS (seed identity
0.00e+00, closed loop 0.0253 m).
NOTE for GOCE specifically: it flew DRAG-FREE (ion thruster compensating drag), so
even a perfect propagator carries an un-modelled along-track term -- ~170 m at 1 mN
and ~3.3 km at 20 mN over 3 h. Expect GOCE to stay worse than CHAMP for that
physical reason, and use CHAMP/TU Delft as the clean control.

## Swarm is OPEN, and BOTH its r,v products now auto-download
Read the user's two real files and they settled the design:
**SWARM  SW_OPER_GPSANAV_1B_...ZIP** contains a genuine SP3:
    #cV2013 11 25 11 0 9.00000000  46807  d ITRF FIT GMV
    +  1  L47      -> Swarm A ITSELF (not the GPS constellation)
    PL47/VL47      -> r AND v, 1 Hz, ITRF, 46807 epochs, 13.00 h
  Verified through our reader: 46807 epochs, hasVel=1, alt 484.9 km, |v| 7604.8 m/s,
  dt 1.0 s, first epoch 2013-11-25 11:00:09.
**Routing changed: Swarm needs NO LOGIN.** Its download endpoint
(?do=download&file=...) serves without EO Sign In, so Swarm was removed from
gatedSite and now goes down the OPEN path (data.ensure tries every candidate),
exactly like CHAMP. Registry: 16 OPEN / 2 LOGIN (only the GOCE archive and NASA
Earthdata remain).
**Both products supported via opts.product:**
  'pod'    (default) swarm/Level2daily/{Entire_mission_data|Latest_baselines}/POD/RD/
                     Sat_X/SW_OPER_SP3<X>COM_2__<d>T000000_<d>T235930_<bl>.ZIP
                     -> times are ALWAYS T000000..T235930, so the name is
                        CONSTRUCTIBLE and it automates exactly like CHAMP.
  'gpsnav'           swarm/Level1b/Entire_mission_data/GPSxNAV/Sat_X/
                     SW_OPER_GPSANAV_1B_<d>T<hhmmss>_<d>T<hhmmss>_<bl>.ZIP
                     -> start/stop are the RECEIVER's on/off times, so the name
                        CANNOT be constructed. New data.swarm_list() lists the
                        folder via the public ?do=list endpoint and matches the date.
Both are listed in credentials/web_sources.csv with their exact paths and why one
needs listing, so it is visible where each satellite's data comes from.

## read_sp3 was O(n^2) -- a 1 Hz day never finished
The user's 1 Hz GPSxNAV file (46807 epochs, ~94k records) hung the reader: it grew
utc/R/V/ids with `R(end+1,:)` on every record. Preallocated + geometric growth ->
**46807 epochs in 20.5 s**. Found only because a REAL 1 Hz file was tried; every
synthetic test used ~100 epochs and hid it.
Bug found while fixing it: my counter was named `n`, which the epoch parser already
uses for its sscanf result (`n=sscanf(ln(2:end),'%f',6)`), so the counter silently
became a 6-vector -> "op1 is 6x6". Renamed to nRec.

## GOCE PSO is NOT SP3 (this is why it needs its own reader)
GO_CONS_SST_PSO_2__...TGZ unpacks to .HDR + .DBL, and the DBL is **447 MB of XML**:
    <SST_PRD_2>  <List_of_SP3c_Records count="8640">   <- reduced-dynamic orbit
      <SP3c_Record><Time_Information><GPS_Time>...      <- GPS TIME, not UTC!
        <L15><Position unit="km"><Velocity unit="dm/s">
    <SST_PKI_2>  kinematic orbit ; <SST_PCV_2> covariance ; rotation records
So GOCE needs: streaming XML parse of SST_PRD_2 (xmlread cannot take 447 MB), r,v
extraction, and a **GPS->UTC** correction (15 s in 2009 -- ignoring it is ~115 km of
along-track). read_sp3 cannot touch this file; PSO coverage is also 2009-09-01..
2012-07-31 only. Manual download + a dedicated reader is the honest route.

## Cache, Swarm A/B/C coverage, and the seeding ladder (all verified)
- **Cache proven**: seeded the cache by hand, then called data.precise_orbit with a
  URL that would 403 in this sandbox -> returned in **0.01 s from cache, no network**.
  Precise orbits use maxAgeDays = **Inf** (correct: a 2010 SP3 is immutable);
  opts.force overrides. So nothing re-downloads.
- **Swarm A/B/C, arbitrary dates**: URL generation verified for A/B/C x
  2014-03-15 / 2020-07-04 -> SW_OPER_SP3{A,B,C}COM_2__<d>T000000_<d>T235930_0202.ZIP.
  No login (open path, like CHAMP).
- **New 09_docs/SEEDING.md** documents the ladder measured here (450 km, 3 h):
    SP3 with V records  dv = 0 (exact)      -> the CHAMP 5.16 m result
    SP3 position-only   dv = 0.002 m/s      -> ~65 m
    TU Delft track      dv = 0.026 m/s      -> ~840 m
    TLE mean elements   dv = 0.56 m/s       -> ~18 km
- **TLE cannot be improved**: a TLE is a MEAN element set for SGP4, ~1 km at epoch,
  degrading 1-3 km/day. A 1 km sma error alone IS 0.56 m/s of seed velocity. So a
  TLE-seeded residual is the TLE's own error, not ours -- validate_tle MEASURES it
  rather than beating it. Use TLE only where nothing better exists (SLATS, ISS).
- **TU Delft cannot be improved much further**: the product is metre-rounded and
  carries no velocity; differentiation amplifies rounding by ~1/h. The deg-7 /
  ~1000 s LSQ fit already sits near the optimum (0.021 m/s at deg 9 / 161 pts vs
  0.026 at the default) -- the remaining error is the DATA, not the fit.
=> Practical rule now documented everywhere: prefer SP3 (open for CHAMP, GRACE,
   GRACE-FO and Swarm); TU Delft for density or when no SP3 exists; TLE last.

## r-vs-r+v capability table (sat.rv) + end-to-end audit
Added **05_data/sat_data/rv_capability.csv** (Excel-readable) and **sat.rv** to print
it: for every satellite x product it states the source, access, whether it gives r,
whether it gives v, and the resulting VELOCITY QUALITY -- because that is exactly
what sets the seed floor (09_docs/SEEDING.md).
Key distinction made explicit:
  * **YES**      -> the file ships velocity; we seed EXACTLY. Confirmed from real
                    files: Swarm GPSxNAV (#cV, 46807 V records) and GOCE PSO XML
                    (<Velocity unit="dm/s">). TLE also yields r,v via SGP4, but its
                    MEAN elements cap it at ~0.56 m/s.
  * **runtime**  -> the SP3 header decides (#dV = position+velocity, #dP = position
                    only). Deliberately NOT hard-coded here: read_sp3 reports hasVel
                    per file and warns if a #dV file has missing/partial V records.
                    The FILE is the authority, not a table that can go stale.
  * **no**       -> TU Delft is position-only (alt/lat/lon, metre-rounded); velocity
                    is DERIVED (LSQ deg 7 / ~1000 s) at ~0.026 m/s -> the floor.
sat.list now points at sat.rv, so the path is: sat.list -> which satellites exist,
data.sources -> which websites, sat.rv -> what each gives (r or r+v).
GFZ ISDC (isdc-data.gfz.de) is the OPEN host for CHAMP + GRACE + GRACE-FO and is
recorded as such for all five satellites.
END-TO-END AUDIT (this build):
  1) parse: 15/15 changed/new functions OK
  2) routing: 11 expected / 0 unexpected (all 11 catalog satellites)
  3) rv table: 11/11 satellites covered
  4) real 1 Hz Swarm file: 46807 epochs, hasVel=1, #dV -> seeds exactly
  5) test_gravity PASS, test_energy PASS, test_seed PASS

## CHAMP DOES ship velocity -- upgraded from "runtime" to YES
Evidence is this project's own bug history: gps_reference converts the file's OWN
v_ecef via v_eci = Ct*v_ecef + (dCt/dt)*r_ecef, and the entire CIP/omega bug (which
took CHAMP from 7650 m to 5.16 m) was about converting that FILE-SUPPLIED v_ecef.
A position-only product would have had no v_ecef to convert. So CHAMP RSO L06 is
#dV and seeds EXACTLY. rv_capability.csv updated; GRACE/GRACE-FO (same GFZ precise-
orbit family) marked "expected #dV", still confirmed per file by read_sp3 at runtime.

## ONE validation script (validate_satellite.m) replaces the three
validate_sp3 / validate_tudelft / validate_tle were copies of each other -- which is
precisely how validate_tudelft drifted onto FRAME='gmst' and cost ~6.7 km. There is
now ONE script: set SAT + DATE + SOURCE ('sp3'|'tudelft'|'tle') and run. It prints
the implied seed dv (along/3t) next to the residual so the seed floor is visible.
The three old names remain as one-line stubs that explain the move and open the new
script, so nobody is stranded.

## Two robustness bugs found by running the REAL 1 Hz file end-to-end
1. **1 Hz products were unusably slow.** gps_reference rotated EVERY epoch through
   the full IAU chain before any thinning -- 46807 rotations for one Swarm GPSxNAV
   day. Now it thins BEFORE the frame chain, choosing the factor from opts.output_dt
   vs the native sampling (30 s wanted from 1 Hz -> keep every 30th) with a 4000-epoch
   hard cap. Epoch 1 (the seed) is always kept and the file's own r,v are untouched;
   only the comparison grid coarsens. Measured: 46792 -> 1560 epochs, whole
   validation in **1 s** (previously timed out).
   Sub-bug: thinN used floor(want/dt) and dt=1.0000001 made floor give 29 -> round().
2. **A TLE outage killed SP3 validation.** validate_against_gps did `cfg = S.cfg`,
   but sat.import only returns .cfg when its TLE fetch SUCCEEDS. Seeding from SP3 or
   TU Delft needs NO TLE, yet a SatChecker hiccup aborted the run with the cryptic
   "structure has no member 'cfg'". Now the config is built from the CATALOG
   (mass/Aref/Cd/Cr) and the TLE is optional, with a one-line note when it is absent.
   Verified: SWARM-A validated from the user's 1 Hz file with NO network at all ->
   |3D| 23.48 m (J2-only model, 15 min arc) in 1 s.
AUDIT: parse OK; routing 11 expected / 0 unexpected; test_gravity, test_energy,
test_seed all PASS.
