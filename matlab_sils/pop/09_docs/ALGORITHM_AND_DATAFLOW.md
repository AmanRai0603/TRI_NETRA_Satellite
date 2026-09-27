# Algorithm & data flow — how it fits together, and where to change it

Read this before modifying anything. It is the map: what calls what, what data enters
where, and which line to edit for a given change.

---

## 0. The one rule

> **`op.buildWorld` is the only place that touches disk or network.
> Nothing below `op.accel` may fetch anything.**

`op.accel` is the integrator's right-hand side. It runs 13× per step for rk78,
thousands of steps per arc. Any I/O there is a bug, no matter how well cached.
Every driver-data defect in this codebase traces back to something reaching for data
from the wrong layer.

---

## 1. Layers

```
  SCRIPTS (open, knob-driven; you edit these)
    EXAMPLE_16U.m          design study: sweep a knob, see the orbit respond
    validate_OD.m          are we right? -> real satellite data
    compare_OD.m           which setting is better? -> sweep vs the same truth
    compare_density.m      density models vs measured
       |
  PIPELINE (+op)                    op.buildWorld -> op.propagate -> op.accel
       |
  PHYSICS (+forces, +atmos, +grav, +srp, +drag, +thirdbody, +relativity, +tides)
       |
  DATA (+data, data_sources/)       fetch + cache + parse
       |
  FRAMES/TIME (+frames, +timeconv, +de440)
```

---

## 2. The propagation pipeline

```
cfg  (epoch, r0, v0, tspan, spacecraft, forces, gravityField, frame, integrator,
      spaceweather, output)
  |
  v
W = op.buildWorld(cfg)                  <-- THE ONLY FETCH POINT
  |   W.grav      op.gravLoad(cfg.gravityField)   -> data.gravity  (EGM2008/EIGEN .gfc)
  |   W.eph       de440.open()                    -> de440s.bsp
  |   W.frame     + data_dir = data.eop_dir()     (unified EOP cache)
  |   W.omega_eci earthRateECI(epoch, build)      (ALSO pre-fetches EOP -- load-bearing)
  |   W.drv       data.drivers(atmos, window)     <-- per-model; see section 3
  |   W.drivers   provenance: window, model, what resolved, eop dir, frame
  v
sol = op.propagate(cfg)
  |   integ.rk78 / rk45 / odesuite     (cfg.integrator.method, .rtol, .atol)
  |     repeatedly calls:
  v
[a, parts] = op.accel(t, r, v, W)       <-- the RHS. NO I/O HERE.
  |
  |   ctx.utc   = op.addsec(W.epoch, t)                 epoch advances
  |   ctx.T     = timeconv.convertUTC(ctx.utc, dUT1)    UT1/TT/TDB/GMST
  |   ctx.E     = ephemInputs(ctx.T.tdb_jd, W.eph)      Sun/Moon -- ONLY if a force needs it
  |   ctx.C,Ct  = frames.eci2ecef(ctx.utc, build)       rotation at THIS epoch
  |   ctx.r_ecef= ctx.C * r ;  ctx.v_ecef = ctx.C*(v - omega x r)
  |   ctx.sc/grav/eph/swtable/swmanual/jbidx/drv        handed down from W
  |
  |   a = forces.gravity(ctx)                            ALWAYS
  |     + forces.{thirdbody,drag,srp,erp,relativity,solidtides,oceantides,empirical}
  |       for each with cfg.forces.<name>.on == true
  v
sol.t, sol.r, sol.v   (on cfg.output.times)
```

**To add a force:** write `02_forces/+forces/myforce.m` taking `ctx`, returning a 3×1
ECI acceleration; add its name to the `order` list in `op.accel`; if it needs Sun/Moon,
add it to the `needE` list too, or `ctx.E` will be `[]`.

---

## 3. Drivers — each density model eats a different set

This is the part that was wrong for a long time. **The models are not interchangeable.**

```
data.drivers(model, startDate, endDate, opts) -> DRV
  exponential        -> (nothing; altitude only)
  nrlmsise           -> DRV.swtable   data.spaceweather      F10.7 + ap (+aph history)
  dtm2020            -> DRV.swtable   data.spaceweather      F10.7 + Kp   (dtm3, OPERATIONAL)
  dtm2020_research   -> DRV.f30TT     get_f30                F30 + F30_bar
                        DRV.ap60TT    get_gfz_hpo{'ap60'}    ap60         (dtm5, RESEARCH)
  jb2008             -> DRV.jbidx     data.jb2008_indices    F10/S10/M10/Y10 + DSTDTC
```

Mirrors `GOCE_density_study/2_spaceweather/` + `align_drivers_to_track.m`. Sources:
OMNI2 (F10.7, Kp, ap), GFZ (Kp/ap/ap60), CLS/LISIRD (F30), SET (JB2008), NOAA/SWPC
(forecast).

```
buildWorld: window = [epoch - padDays, epoch + tspan + padDays]     padDays = 3
                     ^ 3 not 2: aph(7) reaches back 57 h = 2.375 d
            W.drv = data.drivers(model, window)
op.accel  : ctx.drv = W.drv ; ctx.jbidx = W.jbidx
forces.drag: switch model
               exponential      -> sw = struct()
               jb2008           -> sw.jb_idx = ctx.jbidx        (lags applied INSIDE
                                                                 jb2008_density)
               dtm2020_research -> sw = atmos.research_drivers(ctx.drv, ctx.utc)
               otherwise        -> sw = atmos.spaceweather(ctx.utc, {table|manual})
             atm = atmos.provider(model, geo, sw)
```

### Conventions that are OPT-IN, and why

`atmos.spaceweather` faces a genuine fork, and refuses to pick silently:

| option | default | spec says | why the default |
|---|---|---|---|
| `lag_f107` | `false` (same day) | NRLMSISE & DTM2020 want **t−24h** | `align_drivers_to_track` samples `'previous'`, i.e. same day. The density models here were validated against GOCE **measured** density with that alignment. Turning the lag on makes the propagator spec-correct **and simultaneously makes it disagree with the study you trust.** |
| `aph_mode` | `'flat'` (Ap×ones) | aph is a **57 h history** | `run_comparison_study.m` uses flat. `'history'` gives a real storm response but needs `flags(9) = -1`, or `atmosnrlmsise00` ignores the array. |

Either way `sw.F107_lag` and `sw.aph_source` record what was used. **This is a decision
for you, not for the function.** Both paths are verified (§6).

---

## 4. Validation — `validate_OD`

```
9a  THE PLAN         print every decision BEFORE acting
9b  SATELLITE DATA   data.itsg(SAT, DATE, product) for THIS epoch, ALL up front:
                       reducedDynamicOrbit  -> the seed r0,v0 (the file's own, no
                                               derivation, no rotation) + truth [1]
                       kinematicOrbit       -> truth [2] (GPS geometry; INDEPENDENT
                                               of any force model)
                       nonConservativeForces+ attitude -> truth [3]
                       neutralDensity_ACC   -> truth [4]
                       data.tudelft_density -> truth [5] (independent second density)
9c  PROPERTIES       itsg_catalog.csv (mass/area/Cd/Cr), or SC_* knobs for a new satellite
9d  DRIVERS+WORLD    op.buildWorld  (section 2)
9e  PROPAGATE        op.propagate
9f  COMPARE          validation.od_metrics(sol, W, REF, opts) -> R
```

**Order matters:** data first means a missing product fails in seconds, not after an arc.

```
validation.od_metrics
  [1] vs RDO   interp_state(Hermite) -> rtn_stats -> radial/along/cross/3D + dv
  [2] vs KIN   same, on the kinematic epochs
      refSpread = RMS|RDO(at KIN epochs) - KIN|      <-- THE FLOOR
  [3] vs ACC   quat_continuous -> quat2dcm(sat->celestial); compare to
               parts.drag+srp+erp
  [4] vs DEN   atmos.provider at the reference's own geodetic point
  [5] vs TUD   same, at TU Delft's points (their track ships with the density,
               so OUR position error never enters)
      densitySpread = |log median(DEN) - log median(TUD)|   <-- the density floor
```

**What each metric is worth.** [1] ranks agreement with *TU Graz's force model*, not
with physics. [2] is geometry only. **[3], [4], [5] are measurements — prefer them when
they disagree.** A residual below `refSpread` is not resolvable; do not tune there.

---

## 5. The comparator — `compare_OD`

```
7a  fetch REF ONCE (never per config -- a sweep must not refetch its own truth)
7b  for each value v in SWEEP.values:
      cfg <- validation.sweep_knob(SWEEP.knob, v, FORCES, SC, INTEG, SW)
      W   <- op.buildWorld(cfg)          (drivers re-resolve: a DATE or ATMOS sweep
                                          MUST refetch; a Cd sweep re-fetches
                                          harmlessly)
      sol <- op.propagate(cfg)
      R(v)<- validation.od_metrics(sol, W, REF, ...)   <-- SAME metric engine as
                                                            validate_OD. One truth,
                                                            one ruler.
7c  rank by the chosen metric; print the table
```

Knobs: `FORCES.<f>.<field>`, `SC.{mass_kg,Aref_m2,Cd,Cr}`, `INTEG_METHOD`,
`SW_MANUAL.<field>`. A typo **errors** rather than silently creating a field (verified).

**Why the sweep is the only thing that catches a wrong constant.** Fixtures built from
the engine's own output are structurally blind to a consistently-wrong parameter — that
is exactly how `Cd = 2.2` hid inside a perfect `ratio 1.000`. A sweep varies the
parameter against a *fixed external truth*, so a wrong value shows up as "the optimum
is not where you set it."

---

## 6. Mathematical verification (measured, not asserted)

| what | result |
|---|---|
| `interp_state` exact on a cubic | max err **7.1e-15** |
| `interp_state` convergence order | **4.00, 4.01, 4.00** → confirms O(h⁴) |
| `interp_state` vq = d(rq)/dt | analytic vs finite-diff agree to 3e-9 → the `1/h` scaling is right |
| RTN triad orthonormal / right-handed | **1.1e-16** / det **+1.000000000000** |
| pure radial offset → `.rad` only | rad=+0.5000, alo=0, cro=0 |
| `quat2dcm` vs analytic Rz | **0.00e+00**; orthonormal; det=1; q and −q identical |
| `quat_continuous` | 7 flips unwrapped, **every rotation unchanged**, all dots ≥ 0 |
| aph index arithmetic (`history`) | `[300 21 20 19 18 13.5 5.5]` = exactly now/−3/−6/−9h + means over −12..−33h and −36..−57h, **crossing the day boundary** |
| F10.7 lag, both conventions | default 92 (same-day), `lag_f107=true` → 91 (t−24h) |
| two-body 1-period closure | **0.065 m** |
| two-body energy drift | **8.3e-14** |
| integrator invariance rk78/rk45/ode45 | agree to **1e-5 m** — a correct setup must not depend on the integrator |
| `sweep_knob` | `gravity.degree` sets degree **and** order; `SC.Cd` isolated; typo errors |

---

## 7. Where to change what

| I want to... | edit |
|---|---|
| add a force | `+forces/myforce.m`; add to `order` in `op.accel`; add to `needE` if it uses Sun/Moon |
| add a density model | `+atmos/mymodel.m`; case in `atmos.provider`; **driver branch in `data.drivers`**; branch in `forces.drag` |
| add a satellite | row in `05_data/sat_data/itsg_catalog.csv` (13 columns; empty fields are fine now) |
| run a satellite not in the catalog | `USE_CATALOG=false` + `SC_MASS_KG/AREA_M2/CD/CR` |
| change the truth | `SEED_PRODUCT`, `DO_RDO/KIN/ACC/DENSITY/TUDELFT_DENSITY` |
| add a sweep knob | `validation.sweep_knob` |
| add a figure | `show_OD`; give it a switch in `defaults()` |
| use spec-correct space weather | `cfg.spaceweather.lag_f107 = true`, `.aph_mode = 'history'` |
| future epoch | `cfg.spaceweather.source = 'forecast'` (auto for >3 d out) |

## 8. Known gaps

`jb2008` `Mmol=16` / `T=1000 K` hardcoded · `dUT1=0` (only bites `build='gmst'`, ~450 m)
· DTM2020 returns `NA` (not an error) out of range · `jb2008_density` extrapolates
silently past SET coverage · `get_gfz_hpo` uncached · `F107_bar_n` (clipped centred-mean
flag) unconsumed · `research_f107` not its own sweep value · `+de440/chebval.m` duplicated
in `private/` (edit one, edit both).

**Everything here is Octave-verified against SYNTHETIC fixtures.** Wiring, units, frames,
ordering, dispatch and the mathematics above are proven. **The numbers are not** — the
fetchers are MATLAB-only (`timetable`/`datetime`) and `nrlmsise` needs the Aerospace
Toolbox. Real validation needs your MATLAB, your CHAMP cache, and a network.
