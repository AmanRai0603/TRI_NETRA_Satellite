# Driver data audit — space weather, EOP, and what each atmosphere model actually eats

Traced end-to-end: does the propagator fetch the right driver data, for the right
epoch, and hand each model the indices *that model* was fitted on?

## 0. FIRST: the measured space-weather subsystem is NOT IN THE TREE

`05_data/` holds only `+data  +itsg  +sat  sat_data`. **There is no `data_sources/`.**
`setup_paths.m`'s own comment expects it: `05_data  fetch+cache  (+data +sat data_sources sat_data)`.

Missing, and absent from the original upload (not deleted by me):

| missing | called by | consequence |
|---|---|---|
| `get_f107` | `data.spaceweather` | **all measured F10.7 is unavailable** |
| `get_gfz_hpo` | `data.spaceweather` | **all measured Kp/ap is unavailable** |
| `get_jb2008_indices` | `data.jb2008_indices` | **JB2008 can never run** |

So `data.spaceweather()` throws `undefined function`. `buildWorld` calls it inside a
`try ... catch` that swallows the error and leaves `W.swtable = []`. Then
`forces.drag` calls `atmos.spaceweather(utc, swopts)` with no table and no manual
values, which errors **per RHS evaluation**, at t=0, with a message about passing
`opts.manual` — nowhere near the actual cause.

**`SW_MANUAL = []` is the shipped default of `validate_OD` and `compare_OD`, and
`SW_MODE = 'measured'` is the shipped default of `EXAMPLE_16U`. None of them can
run as shipped.** Every run in this tree must supply manual indices — meaning every
run to date has been on a fixed, hand-typed F10.7, with no measured space weather
at any epoch.

This reorders everything below. The lag fix (3b) and the ap-history fix (3c) only
bite on the **table** path, which does not currently exist. They are correct and
they are inert until `data_sources/` is restored. On the manual path F10.7 is
whatever you typed (no lag to apply) and `aph` is necessarily flat.

**Action: find `data_sources/spaceweather/` in the source project and ship it.**
Until then, `sw.aph_source` will read `manual-flat (no storm history)` on every run,
which is at least now visible rather than silent.

---

**Verdict on the rest: the per-step plumbing is correct. The index CONTENT was not.**
`op.accel` advances the epoch properly and `forces.drag` resolves space weather per
step at that epoch. But the resolver returned one index set — NRLMSISE's — to three
models that need three different ones, at the wrong lag, with the storm history
discarded.

---

## 1. Per-step plumbing — CORRECT

```
op.accel:18   ctx.utc = op.addsec(W.epoch, t)           % epoch advances per step ✓
op.accel:32   frames.eci2ecef(ctx.utc, W.frame.build)   % rotation at THIS epoch ✓
forces.drag   sw = atmos.spaceweather(ctx.utc, swopts)  % indices at THIS epoch ✓
forces.drag   geo.lat_deg = rad2deg(lat)                % degrees ✓
```
Space weather is resolved per RHS evaluation at the correct epoch, from a
pre-loaded table. No caching bug, no frozen-epoch bug.

## 2. EOP — CORRECT, with two caveats

- `eci2ecef_{A,B,C}` hold `persistent TAB` and `persistent CACHE`: the IERS tables
  are parsed **once**, not per step. `eci2ecef` is called from every RHS evaluation
  (thousands per arc) — without those persistents this would be unusable. ✓
- EOP is fetched at **`buildWorld`** time, not mid-integration: `buildWorld:21`
  calls `earthRateECI`, which calls `eci2ecef` three times. So a missing network
  fails at setup, not 40% into an arc. ✓ (This is load-bearing and accidental —
  don't remove `earthRateECI` without replacing the pre-fetch.)

**Caveat 1 — the unified cache is a promise the code does not keep.**
`data.eop_dir()`'s header says to pass `cfg.frame.data_dir = data.eop_dir()`. The
A/B/C builds default `data_dir` to `''` and fall back to a folder beside the
function. **Nothing** — not `buildWorld`, `validate_OD`, `compare_OD`, or
`EXAMPLE_16U` — sets it. EOP therefore lands outside `data.root()`, in a second
cache. Not a correctness bug; it does mean "clear the cache" won't.

**Caveat 2 — `dUT1` is 0 everywhere.**
`op.accel:19` does `timeconv.convertUTC(..., getf(W.frame,'dUT1',0))` and nobody
sets `cfg.frame.dUT1`. So **UT1 = UTC** for the derived time scales.
- For `build='C'`/`'B'`/`'A'`: harmless. Those read ERA/UT1 from EOP themselves,
  and TDB depends on TAI–UTC (leap seconds), not UT1.
- For `build='gmst'`: `ctx.T.gmst_rad` is passed straight through and used. |dUT1|
  reaches 0.9 s → 6.6e-5 rad → **~450 m** of ECEF error. Irrelevant for density
  (LST error ~0.00025 h), fatal for anything comparing against real ITRF.

## 3. The index sets — WRONG. Three models, one struct.

`atmos.provider(model, geo, sw)` hands the same `sw` to all three, making them look
interchangeable. They are not:

| model | what it was fitted on | what it was given |
|---|---|---|
| NRLMSISE-00 | F10.7 (t−24h), F10.7a, **aph(1..7): 57 h ap history** | F10.7 (same day), F10.7a, `Ap*ones(1,7)` — and `flags(9)=1`, which discards `aph` anyway |
| DTM2020 **operational** | F10.7 (t−24h), F10.7 81-d mean, Kp | F10.7 (same day), daily-mean Kp scalar |
| JB2008 | F10, S10, M10, Y10 (+81-d means), DSTDTC — **SOLFSMY.TXT + DTCFILE.TXT** | `[]` |

**Correction to an earlier claim of mine:** DTM2020's *operational* variant genuinely
is F10.7+Kp driven (`DTM2020_F107_coeffs_init` ← `DTM_2020_F107_Kp.dat`). The
F30/ap60 port is the **research** variant, `research/dtm2020_density.m`, and is not
on the `atmos.provider` path at all. The operational contract was right.

### 3a. JB2008 could never run. Fixed.
`atmos.jb2008` read `idx = getf(sw,'jb_idx',[])`; `atmos.spaceweather` never sets
`jb_idx`; `jb2008_density:67` does `datenum(idx.sol.Time)` on that `[]` and dies.
`data.jb2008_indices` — the correct fetcher — **existed with zero callers**. The
wire was never connected. Now auto-loads with a `persistent` cache (it runs inside
the RHS; it must not hit disk per step).

The JB2008 *core* was always right: `jb2008_density` lags F10/S10 by `dnum-1`, M10
by `dnum-2`, Y10 by `dnum-5`, DSTDTC hourly — exactly per spec.

### 3b. The F10.7 lag. Fixed.
`atmos.spaceweather` looked up `T.mjd == floor(datenum(utc))` — the **same day**.
Every model wants the previous day:
- NRLMSISE-00 spec: *"f107 — DAILY F10.7 FLUX FOR PREVIOUS DAY"*
- `dtm2020_oper_density` header: *"F107 — F10.7 flux at t−24h [sfu]"*
- `jb2008_density`: already did it right

This is physical, not clerical: the thermosphere lags EUV, and the models were
**fitted** with the lagged index. The error scales with dF10.7/dt — smallest at
solar minimum (when you'd test it), largest on a rising-phase storm (when you care).

Now returns t−24h, with `sw.F107_today` carried alongside and `sw.F107_lag` naming
what was used. `opts.lag_f107=false` reproduces old numbers.

### 3c. The storm history. Fixed.
`atmos/nrlmsise.m` built `aph = Ap*ones(1,7)` from the **daily** Ap. Two faults:
1. The daily Ap in all seven slots erases the 57 h structure the array exists for.
   During a storm `ap3` can exceed 200 while daily Ap is 50 — the model was being
   told the storm didn't happen.
2. `flags(9) = 1` tells `atmosnrlmsise00` to use the daily Ap and **ignore `aph`
   entirely**. So even a correct `aph` would have been discarded. `flags(9) = -1`
   is the switch.

The tell: `atmos.spaceweather` computed `sw.ap3` and **nothing anywhere consumed
it**. Now `sw.aph` is built as a real history (`aph(1)` daily; `aph(2..5)` now/−3/−6/−9 h;
`aph(6)` mean −12..−33 h; `aph(7)` mean −36..−57 h), walking the flattened 3-hourly
series so it crosses day boundaries, and `flags(9) = -1`.

At VLEO this is not a refinement: density can double in hours.

### 3d. The NaN poisoning — and the truth about "DTM2020 returns NaN". Fixed.

I previously reported DTM2020 as returning NaN. **That was wrong.** DTM2020 is fine:

```
F107+F107a+ap  (NO Kp)       sw.Kp=NaN  ->  dtm2020 rho = NaN
F107+F107a+Kp+ap (complete)  sw.Kp=2    ->  dtm2020 rho = 4.66e-13
```

`atmos.spaceweather`'s manual branch did `sw.Kp = getf(m,'Kp', NaN)` — inserting a
NaN **field**. Downstream reads use `getf(sw,'Kp',3)`, and `getf` defaults on
*missing or empty*, **never on NaN**. So the field existed, the default never fired,
and NaN flowed to rho → acceleration → integrator.

The model that broke depended only on **which index you omitted**: DTM2020 reads
`Kp`, NRLMSISE reads `ap`. Give `ap` and no `Kp` — as any reasonable person would —
and DTM2020 dies while NRLMSISE sails on. `manual = struct('F107',90)` alone made
Kp, ap **and** ap3 all NaN, silently.

Now: ap and Kp are the same activity on two scales, so either derives the other
(`kp2ap` / new `ap2kp`); giving neither **errors**, which is what the file's header
always promised and never delivered on that path. Plus `assertFinite` at both exits —
a NaN index becomes a one-line message at the source instead of a step-size collapse
thousands of evaluations later.

### 3e. Silent stand-ins. Fixed.
`atmos/nrlmsise.m` had `F107=getf(sw,'F107',150); F107a=getf(sw,'F107a',150); Ap=getf(sw,'ap',4)`.
Call it with `sw=struct()` and you silently got **solar-max-ish air**. The
`spaceweather.m` header's "NO HARDCODED DEFAULTS" was true of that file and
defeated one layer up. Now `req()` errors.

---

## What this means for the comparator

`compare_OD`'s headline sweep is `FORCES.drag.atmos` across
`{exponential, nrlmsise, jb2008, dtm2020}`. Before these fixes that sweep was
**scientifically void**:
- `jb2008` → hard error (index wire absent)
- `dtm2020` → NaN if you omitted Kp
- `nrlmsise` → no storm response, wrong F10.7 lag
- so the only rows that "worked" were wrong in different ways

Re-run it now. **All four fixes change results** — re-baseline before comparing to
any earlier number.

## Still open

- **`atm.T` for JB2008 is a 1000 K placeholder.** The core returns `TEMP`;
  `jb2008_density` drops it. Single-species drag reading `atm.T` gets the
  placeholder. Doesn't affect `atm.rho`.
- **`jb2008_density` extrapolates with `'extrap'`** past the end of SOLFSMY/DTCFILE
  (currently 2026-137) — silently, no error.
- **DTM2020 gets a scalar daily-mean Kp** where the model accepts
  `[Kp_3hdelay; 0; Kp_24hmean; 0]`. `sw.ap3` now exists and could drive the
  3h-delayed slot properly. Not done.
- **`cfg.frame.data_dir`** is never set to `data.eop_dir()`.
- **`cfg.frame.dUT1`** is never set; only matters for `build='gmst'`.
- **None of this is verified against real MATLAB**, and `nrlmsise` needs the
  Aerospace Toolbox, which Octave does not have — the `flags(9)=-1` and `aph`
  changes are **reasoned from the NRLMSISE-00 interface spec, not executed**.
  Verify on your first real run.

---

# Round 2 — the fetchers, ported and fixed

`data_sources/` was recovered from `GOCE_density_study/2_spaceweather/` and now lives
at `05_data/data_sources/spaceweather/{solar,geomag,model_inputs}`.

**Finding the files was not enough — every wrapper was broken against the thing it
wraps.** None of these three had ever been run end-to-end:

| # | fault | consequence |
|---|---|---|
| 1 | **`setup_paths` never added `data_sources`** — line 14 has always listed it; line 30 adds only `05_data`, non-recursively | even with the files present, `get_f107`/`get_gfz_hpo`/`get_jb2008_indices` are undefined |
| 2 | **`data.jb2008_indices` passed `'force'`, `'startDate'`, `'endDate'`** — `get_jb2008_indices`'s inputParser knows only `cacheDir`, `forceDownload`, `solfsmyFile`, `dtcFile`, `maxAgeDays`, and does **not** set `KeepUnmatched` | every call died on *"the name 'force' is not a valid parameter name"* |
| 3 | **`get_omni2` never cached**: `datfile = [tempname '.dat']` | ~20 MB of OMNI2 re-downloaded on **every** call. An N-config `compare_OD` sweep = N re-downloads. `data.spaceweather`'s header claimed "both sub-fetchers cache into `<data.root>/spaceweather`" — **false end-to-end**: `get_f107` never forwarded a cache dir, and `get_gfz_hpo` has no cache layer at all (`webread` every call) |
| 4 | **`padDays` default 2** | NRLMSISE's `aph(7)` is the mean ap over t−36..−57h = **2.375 days** back. For an epoch at 00:00 UTC a 2-day pad puts that bin *before* the table starts, so the storm history silently degraded to the daily Ap — defeating the `aph` fix from round 1. Now **3**. |

## The timeline fault you asked about — `F107_bar` near the present

`get_f107`:
```matlab
f107bar = movmean(allF, 81, 'omitnan', 'Endpoints','shrink');
```
`'shrink'` does **not** return NaN at the edges — it silently shortens the window.
Mid-history that is harmless: the function deliberately pulls whole OMNI2 year files
spanning `[start−41d, end+41d]`, so the window is full.

But at the **end of the OMNI2 archive — i.e. today** — there is no future to average
over. The last ~40 days get a progressively shrinking window, and at the newest
sample the *"81-day **centred** mean"* is really a **~41-day trailing mean**, returned
under the same name with no flag.

For a hindcast this never bites. For a run at or near the present epoch — exactly the
case where you are predicting — `F107_bar` is quietly a different quantity than the
models were **fitted on**, and it is biased in a known direction: a trailing mean lags
a rising solar cycle and leads a falling one.

**Fix:** `get_f107` now returns `F107_bar_n` — the actual sample count per day — and
warns when any day's window was clipped. `F107_bar_n < 81` means that day's mean is
not a true centred value. Downstream can decide; it is no longer lied to.

> Not fixed: nothing yet *consumes* `F107_bar_n`. `data.spaceweather` drops it. The
> honest options are (a) refuse epochs with a clipped window, or (b) splice
> `data.spaceweather_forecast`'s predicted F10.7 onto the tail so the window closes.
> (b) is the right answer for a forecast run and is not done.

`data.spaceweather_forecast` (NOAA/SWPC) already exists and `buildWorld` auto-selects
it for epochs >3 days out — that design is sound and untouched.

## JB2008 now runs offline

The tree already ships `SOLFSMY.TXT` + `DTCFILE.TXT` (release 8_1_0, coverage to
2026-137) next to `jb2008_density`. `data.jb2008_indices` now points
`get_jb2008_indices` at those via `solfsmyFile`/`dtcFile` when no refresh is forced —
so JB2008 needs no network, which matters because it is reached from inside the RHS.

> Bundled files are **static**. Past their coverage, or for 2024–2025 work (SET
> re-derived S10 in May 2025), pass `force=true`. `jb2008_density` interpolates with
> `'extrap'` and will **not** tell you. A coverage warning is now raised, including
> the Y10 t−120h lag margin.

## Satellite properties vs magic numbers — the Cd bug had a twin

Same fault, found by looking for the same shape:

```matlab
forces/srp.m :  getf(c,'Cr',1.3)                      % cfg.forces.srp.Cr only
forces/erp.m :  getf(c,'Cr',1.3)*sc.Aref/sc.mass      % cfg.forces.erp.Cr only
```
Neither fell back to `sc.Cr`. Three scripts carry `cfg.forces.srp.Cr =
cfg.spacecraft.Cr` purely to paper over it — which is the tell. **Nothing patches
`erp`**, so ERP used **Cr = 1.3 for every satellite ever propagated**, ignoring
`cfg.spacecraft.Cr`. CHAMP's catalog Cr *is* 1.3, so on CHAMP it was accidentally
right — which is exactly how this survives.

Fixed at the root, mirroring the Cd fix: **`forces.<f>.Cr` → `spacecraft.Cr` → 1.3.**

**Retyped WGS84 removed.** `de440.constants()` already ships `Re_earth` and `f_earth`,
yet code called `de440.constants()` and then hardcoded `6378137` *on the same line*.
Two sources of truth that can drift. Now sourced from the constants in
`validate_OD`, `od_metrics`, `compare_OD`, `EXAMPLE_16U`, `show_16U`,
`track_reference`. (Left alone: `08_test/*` and `TEMPLATE_propagation.m`, where
independent literals are arguably the point of a test, and `make_OD_fixtures`, where
it defines a synthetic orbit.)

## Still open after round 2

- **`atmos.jb2008` hardcodes `Mmol = 16`** and `atm.T = 1000 K`. The 16 is a defensible
  VLEO modelling choice (atomic-O dominated, same as `dgeom/vleo16u`), but it is a
  *geometry/GSI* property sitting in an atmosphere adapter. The 1000 K is a
  placeholder: JB2008's core returns `TEMP` and `jb2008_density` drops it.
- **`get_gfz_hpo` has no cache layer.** One GFZ request per `buildWorld`, so N per sweep.
- **`F107_bar_n` has no consumer** (see above).
- **None of the fetchers can run under Octave** — they use MATLAB `timetable`/`datetime`
  by design. So the port is verified only as far as *path resolution, argument
  correctness, and reaching the download/parse stage*. The parse and the numbers need
  your MATLAB.

---

# Round 3 — the op pipeline

The fetchers existed and the websites existed. What was missing was the **wiring**:
`op.buildWorld` is the only place allowed to touch disk or network (everything below
it runs inside the integrator's RHS, thousands of times per arc), and it was not
resolving all the drivers — nor the *right* ones.

## The pipeline now

```
op.buildWorld(cfg)          <-- the ONLY fetch point, driven by cfg.epoch + cfg.tspan
  |  window = [epoch - padDays, epoch + dur + padDays]     (pad = 3, not 2)
  |
  +-- W.grav      op.gravLoad          (date-independent)
  +-- W.eph       de440.open           (date-independent)
  +-- W.omega_eci earthRateECI         (also pre-fetches EOP -- load-bearing)
  +-- W.frame.data_dir = data.eop_dir()          <-- NEW: unified EOP cache
  |
  +-- per ATMOSPHERE MODEL, because each eats a different driver:
  |     exponential        -> nothing
  |     nrlmsise, dtm2020  -> W.swtable   data.spaceweather      (OMNI2 + GFZ)
  |                                    or data.spaceweather_forecast (NOAA/SWPC, auto >3d out)
  |     jb2008             -> W.jbidx     data.jb2008_indices    (SET SOLFSMY + DTCFILE)
  |
  +-- W.drivers   provenance: window, model, what was resolved, eop dir, frame
        |
        v
op.accel   ctx.swtable / ctx.swmanual / ctx.jbidx      (no I/O below this line)
        |
        v
forces.drag   switch model:
                exponential -> sw = struct()
                jb2008      -> sw.jb_idx = ctx.jbidx        <-- NEW
                otherwise   -> sw = atmos.spaceweather(ctx.utc, {table|manual})
        |
        v
atmos.provider -> atmos.{exponential,nrlmsise,jb2008,dtm2020}
```

## What was wrong

**One-size-fits-all driver fetch.** `buildWorld` fetched the F10.7/ap table for
*every* non-exponential model. That is right for NRLMSISE and DTM2020 and **wrong for
JB2008**, which never reads F10.7 or ap. So a `jb2008` run downloaded a table it
would never open, *and never fetched the one it needed* — then `forces.drag` called
`atmos.spaceweather` anyway and **errored demanding an F10.7 table for a model that
does not use F10.7**. `atmos.provider`'s header has documented `.jb_idx` all along;
nothing ever supplied it.

**My own round-2 fix was wrong too, and this replaces it.** I had `atmos.jb2008`
auto-load the indices behind a `persistent`. That does file I/O from inside the RHS,
and a `persistent` is keyed to nothing — so **sweeping `DATE` would silently reuse the
first date's fetch for every later date**. Exactly the class of bug this audit exists
to kill. Driver data is date-dependent; it belongs in `buildWorld`, resolved once per
run against the actual epoch window, and visible in `W.drivers`.

**EOP cache.** `cfg.frame.data_dir` now defaults to `data.eop_dir()` in `buildWorld`,
so every A/B/C build shares one cache under `data.root()` instead of writing beside
the function. No script has to remember.

**`W.drivers`.** Every space-weather bug in this toolbox has been silent, so the
resolved drivers are now recorded: window, model, which table was loaded, the EOP
dir, the frame build. Verified date-driven:

```
atmos        | swtable | jbidx | manual | resolved
exponential  |    0    |   0   |   1    | window 2006-12-29..2007-01-04  eop=unified
nrlmsise     |    0    |   0   |   1    | window 2006-12-29..2007-01-04  eop=unified
dtm2020      |    0    |   0   |   1    | window 2006-12-29..2007-01-04  eop=unified
jb2008       | goes for its OWN SET files, ignores manual -- as it must
```
(manual set, so the table fetch is correctly skipped). JB2008 now fails at **setup**
with a message naming the real dependency, not 40% into an arc with a message about
F10.7.

## Cr — the Cd bug's twin, now proven

`forces/srp.m` and `forces/erp.m` read `cfg.forces.*.Cr` with a hardcoded 1.3 default
and no `sc.Cr` fallback. Three scripts hand-patch SRP; **nothing patched ERP**, so ERP
used Cr = 1.3 for every satellite ever propagated. Measured, `spacecraft.Cr` 1.3 -> 2.0
with `forces.*.Cr` unset:

```
srp |a|: 8.969038e-09 -> 1.379852e-08   ratio 1.5385   (= 2.0/1.3)
erp |a|: 1.733294e-09 -> 2.666607e-09   ratio 1.5385   <- was FROZEN at 1.3
```

## Still open

- **DTM2020 *research* (F30/ap60) is not on the `atmos.provider` path.**
  `research/dtm2020_density.m`, `get_f30`, `get_f30_cls`, `f30_from_f107` all exist
  and are wired to nothing. Adding `'dtm2020_research'` needs a fourth driver branch
  in `buildWorld` (F30 + ap60) — the pipeline now has the right shape for it.
- **`F107_bar_n` still has no consumer** — the clipped-window flag is raised and
  dropped by `data.spaceweather`. For a forecast run the right fix is splicing
  `spaceweather_forecast`'s predicted F10.7 onto the tail so the centred window closes.
- **`get_gfz_hpo` has no cache layer** — one GFZ request per `buildWorld`, N per sweep.
- **`atmos.jb2008` hardcodes `Mmol=16` and `T=1000 K`** (the core returns `TEMP`;
  `jb2008_density` drops it).
- **Verified under Octave only.** The fetchers are MATLAB-only (`timetable`/`datetime`),
  so the table-fetch branches are verified as far as dispatch, window construction, and
  argument correctness — not as far as parsed numbers. That needs your MATLAB.

---

# Round 4 — wired to the proven segmentation (this supersedes round 3)

Round 3 hand-rolled an if-ladder in `buildWorld`. That was still me deciding the
model→driver mapping. **The mapping was already decided, proven, and validated** in
`GOCE_density_study` — `2_spaceweather/` (segmented by role) plus
`4_comparison/compute/align_drivers_to_track.m`, driven by `run_comparison_study.m`.
The propagator now mirrors that instead of reinventing it.

## The segmentation — taken from the study, not invented

| model | solar driver | geomagnetic driver | core |
|---|---|---|---|
| `exponential` | — | — | altitude only |
| `nrlmsise` | F10.7, F10.7a | ap + 57 h `aph` history | Aerospace Toolbox |
| `dtm2020` | F10.7, F10.7_bar | Kp, ap | **dtm3**, `DTM_2020_F107_Kp.dat` |
| `dtm2020_research` | **F30, F30_bar** → rescaled | **ap60** (GFZ) | **dtm5**, `DTM_2020_F30_ap60.dat` |
| `jb2008` | F10/S10/M10/Y10 (+81d) | DSTDTC | SET SOLFSMY + DTCFILE |

New: **`data.drivers(model, s0, s1, opts)`** — one place, one mapping.
`op.buildWorld` makes exactly one call to it. The if-ladder is gone.

```
op.buildWorld -> data.drivers(model, window)  -> W.drv (+ W.swtable/W.jbidx legacy)
op.accel      -> ctx.drv
forces.drag   -> per model:  jb2008 -> sw.jb_idx
                             dtm2020_research -> atmos.research_drivers(ctx.drv, utc)
                             otherwise -> atmos.spaceweather(utc, {table|manual})
```

## The research model is now reachable — it never was

`dtm2020_research` (F30/ap60, `dtm5`) — the variant ported bit-for-bit against the
CNES Fortran — **was not on the `atmos.provider` path at all**. `get_f30`,
`get_f30_cls`, `f30_from_f107`, `f30_to_f107scale`, `research/dtm2020_density.m`,
`DTM_2020_F30_ap60.dat` all shipped, all wired to nothing. Now:
`atmos.dtm2020_research` + `atmos.research_drivers`, exposed in `atmos.provider` and
in both sweep knob lists.

**The rescaling is the part that is easy to get silently wrong.** `dtm5` is driven by
F30 **rescaled to the F10.7 scale** (DTM2020 paper eq. 2, `f30_to_f107scale`), and
that regression carries a **drift term in decimal year** — so it is date-dependent and
cannot be a constant. But if `get_f30` cannot reach CLS/LISIRD it falls back to a
pseudo-F30 *derived from F10.7*; rescaling that would apply the drift twice. SWAMI's
guidance is to feed F10.7 directly in that case. So `data.drivers` carries
`f30_is_derived` and `f30src` rather than letting anything guess from magnitude —
mirroring `run_comparison_study.m` lines 141-150.

Verified: `dtm2020_research` reaches `get_f30` → tries real LISIRD F30 → falls back to
F10.7-derived **and says so** → fails at *setup* on Octave's missing `datetime`. And
`cfg.spaceweather.manual` with F10.7/ap correctly does **not** satisfy it: manual F10.7
is not F30.

## Sampling rule

`atmos.research_drivers` samples **`'previous'`** — most recent value at or before the
epoch — the same rule as `align_drivers_to_track`: daily F30 by most-recent-day,
hourly ap60 by most-recent-sample. Deliberately **not** `'linear'`: these are stepwise
measurements with a validity period, not a continuous signal. Interpolating them
invents values never observed and smears storm onsets into the hour before they
happened. Out of window it **errors** rather than extrapolating — a driver outside its
fetched window is a window bug, and reusing the nearest edge value is how a storm goes
missing.

## Still open

- **`research_f107`** — the study's fifth variant (research model driven by
  F10.7-derived F30) is reachable via `f30source='f107'`, but not exposed as its own
  sweep value.
- **`get_gfz_hpo` has no cache** — one GFZ request per `buildWorld`.
- **`F107_bar_n`** (clipped centred-window flag) still has no consumer.
- **Octave-verified only** — the fetchers are MATLAB-only (`timetable`/`datetime`).
  Dispatch, window construction, per-model routing and argument correctness are
  verified; parsed numbers need your MATLAB.

---

# Round 5 — end-to-end, force by force

Every force fired at once, with its data source traced. Magnitudes match the ladder
`EXAMPLE_16U` documents (450 km, quiet):

```
FORCE            |a| [m/s^2]   wired from
gravity          8.5615e+00   W.grav  <- op.gravLoad / data.gravity
thirdbody        8.2106e-07   ctx.E   <- DE440 (de440s.bsp)
drag             7.2130e-08   ctx.drv <- data.drivers(atmos, window)
srp              8.9690e-09   ctx.E.P_srp + sun_eci <- DE440
erp              1.7333e-09   ctx.E.sun_eci <- DE440 + doy
relativity       1.6638e-08   ctx.grav.mu + ctx.E (deSitter)
solidtides       2.2578e-07   ctx.E sun/moon + ctx.C (EOP)
oceantides       1.5654e-09   ctx.T.tt_jd + ctx.grav
```

## Verified sound

- **Ephemeris gate.** `needE = anyOn(F,{thirdbody,srp,erp,relativity,solidtides,oceantides})`
  exactly matches the set of forces that read `ctx.E`. No force reads an ephemeris
  that was not built; none pays for one it does not use. `relativity` additionally
  guards `isempty(ctx.E)` before deSitter.
- **EOP freshness.** `fetch_one(urls, localfile, max_age_days, opt)` re-downloads when
  `(now - file_date) > max_age_days`; the finals rapid tail is on a 7-day check. The
  C04 series is fetched from three mirrors with fallback to the existing file. This
  was already right.
- **EOP cache is now unified**: `eop_dir = <data.root>/eop`, set by `buildWorld`.
- **DE440** loads once into `W.eph` and is reused; `buildWorld` warns if a force needs
  Sun/Moon and the kernel failed.

## Found and fixed

**`relativity.deSitter` could never run.** Line 6 was
`GMs = de440.constants().GM_sun` — **chained indexing on a function-call result**,
which is illegal in Octave and in MATLAB before R2019b. Unlike the five other
occurrences (all inside `nargin` guards that a normal call never trips), this one is
unconditional: it fires on **every** call. So the de Sitter term threw the moment it
was switched on. Fixed here and in the five latent twins
(`schwarzschild`, `lenseThirring`, `accelFromDeg2`, `iers2010`, `phiQuad`).

**The CSV parser dropped empty fields.** Both catalog readers used
`strsplit(L, ',')`, whose default **collapses consecutive delimiters** — so an empty
field (`,,`) vanishes and every later column shifts left. Adding `has_tudelft` /
`tudelft_name` exposed it: rows with no TU Delft name came back with
`tudelft_name` = the coverage string. Rows with every field populated parsed fine,
which is why it survived. Now `'CollapseDelimiters', false`.

## TU Delft measured density — metric (5)

`data.tudelft_density` was **another dead wrapper**: it calls `fetch_tudelft_density`,
which was never shipped (same failure as `get_f107` et al). That silently killed
`compare_density.m` and the TU Delft overlay in `validation.track_reference`.
And `sat.catalog` had no `has_tudelft` / `tudelft_name` at all, while
`compare_density.m:19` asserts on `cat.has_tudelft` — a guaranteed
"non-existent field" error.

Fixed: `fetch_tudelft_density` + `read_tudelft_density_file` ported to
`05_data/data_sources/density_reference/`; `has_tudelft` + `tudelft_name` added to
`itsg_catalog.csv` (9 satellites) and surfaced by both readers;
`validation.tudelft_to_ref` adapts the timetable to the `REF` shape; metric (5) added
to `validation.od_metrics`, wired into `validate_OD` and `compare_OD` behind
`DO_TUDELFT_DENSITY` (deliberately distinct from the pre-existing `DO_TUDELFT`, which
is the *position* overlay — two different products, two different questions).

**Why it is worth having two truths.** ITSG's `neutralDensity` and TU Delft's density
are derived by different groups, from different processing of the *same* accelerometer,
under different Cd and gas-surface assumptions. Neither is "measured density" in an
absolute sense — both are retrievals. So:

- model agrees with **both** -> the model is probably right
- model matches **one and not the other** -> look at the truth, not the model
- **the two truths disagree with each other** -> that gap, reported as
  `R.densitySpread`, is the real floor under any density claim — exactly what
  `refSpread` is for position

TU Delft also ships the geodetic track *with* the density, so metric (5) evaluates the
model at **TU Delft's own points** — our position error never enters it. That is the
opposite of the ITSG r1.0 `neutralDensity_ACC` product, which is MJD+rho only and must
borrow our state.

---

# Round 6 — the OP loop, proven epoch-driven end to end

## 1. Does the driver window follow `cfg.epoch`?

```
epoch 2007-01-01 -> window 2006-12-29 .. 2007-01-04
epoch 2007-07-01 -> window 2007-06-28 .. 2007-07-04
epoch 2010-03-15 -> window 2010-03-12 .. 2010-03-18
```
Yes. Change the date, the fetch window moves with it. A DATE sweep refetches.

## 2. Within one arc, does every epoch-dependent input actually move?

```
 t [s] | ctx.utc               | sun_eci x [m] | GMST [rad] | rho [kg/m^3]
     0 | 2007-01-01 00:00:00.0 | +2.5748e+10   |  1.75001   | 5.6167e-13
  1350 | 2007-01-01 00:22:30.0 | +2.5789e+10   |  1.84845   | 1.8475e-13
  2700 | 2007-01-01 00:45:00.0 | +2.5829e+10   |  1.94690   | 1.9134e-13
  4050 | 2007-01-01 01:07:30.0 | +2.5869e+10   |  2.04534   | 3.5671e-13
  5400 | 2007-01-01 01:30:00.0 | +2.5909e+10   |  2.14378   | 5.6046e-13
```
UTC advances, the Sun moves, GMST rotates, and density cycles 5.6e-13 -> 1.8e-13 ->
5.6e-13 over one revolution -- the day/night and latitude cycle. **The loop is
epoch-driven at every stage**: epoch -> drivers -> forces -> integrator -> state ->
comparison against the fetched satellite data.

Noted in passing: **DTM2020 returns `NA` rather than erroring** when handed an
out-of-range altitude (it surfaced when a test extrapolated off the output grid).
It is not silent -- `NA` propagates visibly -- but it is not an error either.

## 3. The figures — `show_OD`, rebuilt

Every figure has a switch; nothing is drawn behind your back:

```matlab
show_OD(R, sol, ttl, outdir, struct('rtn',1,'overlay_r',1,'overlay_v',1, ...
        'growth',1,'hist',1,'acc',1,'density',1,'stats',1,'movavg_s',300))
```

| fig | what it answers |
|---|---|
| `rtn` | residual per RTN axis, each with the reference-floor band, both references, and a moving mean. Titles carry RMS **and** mean, because those are different diagnoses |
| `overlay_r` | `\|r\|` ours vs measured **overlaid** — then the radial error and the 3D error underneath. The overlay always looks perfect at metres-on-6800-km; that is exactly why it is never shown alone |
| `overlay_v` | `\|v\|` + the velocity residual level |
| `growth` | running RMS + along-track envelope. **Climbing = a force is mismodelled; flat = noise.** The most diagnostic figure here |
| `hist` | distribution per axis, mean marked in red. **Offset = systematic** (averaging will never remove it); **wide and centred = noise** |
| `acc` | accelerometer measured vs modelled per axis + error + ratio |
| `density` | ITSG **and** TU Delft measured vs modelled, + ratio, + the truth-vs-truth gap |
| `stats` | all five metrics, biases, and the floors |

`movavg_s` defaults to 300 s: ~1/18 of a LEO revolution — long enough to kill
per-sample noise, short enough to leave once-per-rev signal intact, which is the
signal you are hunting. The moving mean is computed over a **time** window, not a
sample count, because kinematic epochs are irregular and a fixed count would be a
different physical window at different points in the arc.

Round-trip after the rebuild, unchanged: `[1] 0.03 m · [2] 0.05 m · floor 0.03 m ·
[3] ratio 1.000 · [4] ratio 1.000`.
