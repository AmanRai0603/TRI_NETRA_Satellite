# OD validation — architecture, and the bugs that were in it

How the OD validation path fits together, and a record of what was wrong. The
second part matters more than the first: every bug below **failed quietly**, and
that is the property worth internalising, not the individual fixes.

## The path

```
data.itsg(SAT, DATE, product)     ITSG / TU Graz. No credentials. Cache-first.
        |                          r,v are ALREADY CELESTIAL -> no ITRF->ECI rotation,
        |                          which removes the bug class that dominated the project.
        v
validate_OD.m   (script: every knob at the top)      one setup, four metrics
compare_OD.m    (script: + SWEEP)                    N setups, same four metrics
        |
        +----> validation.od_metrics(sol, W, REF, opts)   <- THE metrics. One copy.
                     |
                     +-- validation.interp_state    Hermite on (r,v). NEVER pchip.
                     +-- validation.quat_continuous Sign-unwrap before interpolating.
                     +-- validation.quat2dcm        Scalar-first, sat -> celestial.
                     +-- validation.rtn_stats       Project a difference onto RTN.
                     +-- validation.mjd2utc         MJD -> [Y Mo D H Mi S].
                     +-- validation.itsg_catalog    One row of itsg_catalog.csv.
```

`validate_OD` and `compare_OD` both call `od_metrics`. There is one implementation
of each metric, so the two cannot drift apart and disagree.

## The four metrics, and what each is actually worth

| # | metric | what it really tests | trust |
|---|--------|----------------------|-------|
| 1 | position vs `reducedDynamicOrbit` | agreement with **TU Graz's force model**, not with physics — their solution absorbs mismodelling into empirical accelerations | weakest |
| 2 | position vs `kinematicOrbit` | GPS geometry only, no force model — genuinely independent | good |
| — | `refSpread` = (2) − (1) | the reference's **own uncertainty**. A residual below this is not resolvable | the floor |
| 3 | accelerometer vs modelled drag+SRP+ERP | Cd·A/m and density **directly**, in m/s², at the source | strongest |
| 4 | measured vs modelled density | the atmosphere **alone**, free of Cd | strongest |

**Cd and density are degenerate on (1) and (2)** — they enter the equations of
motion only as a product. That degeneracy is the entire reason (3) and (4) exist.

## The bugs. All of them failed quietly.

| bug | what it did | why nobody saw it |
|-----|-------------|-------------------|
| **`Cd` never reached drag** | `forces/drag.m` read `cfg.forces.drag.Cd`; every script writes `cfg.spacecraft.Cd` then replaces `cfg.forces` wholesale. Silently fell back to 2.2 — CHAMP ran at 2.2, not 3.0 (**27% drag error**) | `config.report` printed `Cd : 3`. The report reads `spacecraft`; the physics read `forces.drag`. **The decisions file lied.** `EXAMPLE_16U`'s `SC.Cd` sweep knob was a no-op for the same reason. Three scripts do `cfg.forces.srp.Cr = cfg.spacecraft.Cr` — somebody hit this for SRP and never came back for drag. |
| **`pchip` on orbit states** | 72.6 m RMS of pure interpolant in metric (2), against a 3 cm floor | It looked like a plausible physics residual. pchip is *shape-preserving*: its limiter clips the slope at turning points, and an orbit turns over twice per rev in every component. Radial-dominated and **biased** (+1.2 m mean), so it does not average away. Same grid: pchip 72.644 m, spline 0.032 m, Hermite 0.028 m. |
| **`W.sw` does not exist** | Every density model call threw; an empty `catch` ate it; `rho_mod` stayed NaN → printed **ratio NaN** | `buildWorld` makes `swmanual`/`swtable`; `forces.drag` turns them into `sw` per step via `atmos.spaceweather`. The same mistake in the provenance block silently omitted the space-weather line from **every run ever made**. |
| **Wrong density product** | Hardcoded `'neutralDensity'`; CHAMP only has `neutralDensity_ACC` → 404 → swallowed | Density never ran on the one satellite the header recommends. |
| **`op.geodetic` returns radians** | Written into `.lat_deg`/`.lon_deg` and `lon/15` for LST | Never throws. Just pins the atmosphere near the equator at the wrong local time, forever. A wrong number that looks plausible. |
| **`C` shadowed** | Catalog struct, then reused for the ECI→ECEF rotation matrix | Latent. Fires only when `DO_TLE=true` **and** the r1.0 density branch runs. |
| **Quaternion sign flips** | `q` and `−q` are the same rotation; real star cameras flip branch. Interpolating across a flip drags the quaternion through zero; renormalising then gives a confidently wrong attitude. Measured: **177.8° max error, 13.0° RMS** | Lands as isolated spikes in (3). Barely moves the RMS — reads as sensor noise. |
| **Local functions in a script** | `'mjd2utc' undefined` | MATLAB hoists a script's local functions; **Octave does not** — it defines them only when execution reaches them, so helpers at the end of a script are never defined. Fine on MATLAB, fatal under Octave. |

### The pattern
Not one of these threw an error a user would see. Five produced *plausible wrong
numbers*; three were caught by `catch` blocks and reported as `NaN` or a skip. When
a validation script disagrees with reality, the first hypothesis should be the
script.

## The round-trip test — and what it cannot do

`06_validation/realsat/make_OD_fixtures.m` writes synthetic ITSG files **built from
the engine's own output**, in real GROOPS layout (including the bare-number header
lines a naive parser eats as data). Then every answer is known in advance:

```
[1] ~0.03 m   [2] ~0.05 m   refSpread ~0.03 m   [3] ratio 1.000   [4] ratio 1.000
```

Any deviation is a plumbing bug. That is how the 72 m `pchip` artifact was caught —
[2] read 72.64 m against a reference that is truth + 2 cm noise.

**Its blind spots, stated plainly:**

- **A consistently wrong parameter is invisible.** The fixtures were generated with
  the same config they are tested against, so both truth and model used the silent
  Cd = 2.2. The round-trip returned a perfect 1.000 with a 27% drag error in it.
  Only the **sweep** caught that — `SC.Cd` returned three identical rows.
- **`exponential` depends on altitude only**, so lat/lon/LST never enter. The
  radians→degrees fix is *reasoned* (from `op.geodetic`'s own header), not
  *demonstrated*. It needs a real run with `nrlmsise`.
- **Query epochs land on data nodes**, so the quaternion path is not exercised by
  the round-trip. It is proven separately, at mid-interval.
- Real densities, real Cd, real GROOPS quirks, leap seconds, data gaps: **not
  tested**. Those need real cached files on real MATLAB.

Round-trip consistency proves **plumbing**. It cannot prove **physics**. Both
matter; do not confuse one for the other.

## Known open

- ~~`atmos='dtm2020'` returns NaN~~ **RESOLVED, and it was never a DTM2020 bug**: `atmos.spaceweather` injected a NaN `Kp` field when you gave `ap` and no `Kp`, and `getf` never defaults on NaN. See `09_docs/DRIVERS_AUDIT.md`.
- **`atmos='nrlmsise'`** requires the MATLAB Aerospace Toolbox (`atmosnrlmsise00`).
- **`+de440/private/chebval`** is MATLAB-legal and Octave-invisible (Octave does not
  resolve `private/` inside `+package/`). Irrelevant on MATLAB; fatal for an Octave
  test suite. An identical public copy sits at `+de440/chebval.m`.
- **`EXAMPLE_16U.m`** still has local functions at the end of a script. Fine on
  MATLAB, fails under Octave. `validation.sweep_knob` already covers `applyKnob`'s
  job if you lift them.
- **`FRAME='C'`** (full IAU 2006/2000A CIO chain), `EGM2008` degree 70, the live
  space-weather fetch, and the TU Delft / TLE overlay branches are all unexercised
  by the round-trip.

---

## The order of operations (validate_OD section 9, and compare_OD section 7)

Both scripts now run the same sequence. The order is the point:

```
9a  THE PLAN            print every decision BEFORE acting on any of it
9b  EPOCH + SATELLITE   everything the satellite gives us FOR THAT EPOCH, up front:
    DATA                the seed r,v AND every product we will later compare to
9c  SATELLITE           mass/area/Cd/Cr from itsg_catalog.csv, or typed into the
    PROPERTIES          SC_* knobs for a satellite the catalog does not know
9d  DRIVER DATA         op.buildWorld resolves space weather / EOP / gravity /
    + THE WORLD         ephemeris FOR THAT EPOCH -- the ONLY fetch point
9e  PROPAGATE           everything is decided by now; this is just integration
9f  COMPARE             our state vs the data fetched in 9b
```

**Why it had to change.** `validate_OD` used to do `buildWorld` + `propagate` at
line 157 and only then fetch `kinematicOrbit`, the accelerometer, attitude and
density at lines 168-217. So a missing product cost you a full arc before it
surfaced, and the flow did not match `compare_OD` (which already fetched its
reference up front, because a sweep must not refetch per config).

Now a data gap fails in seconds, at 9b, before any physics runs.

**Nothing runs silently.** Section 9a prints the seed product and index, which
comparisons are on, the gravity field *and* the fact that `'default'` caps at
degree 6, the drag model and atmosphere, every other force's on/off, whether space
weather is MEASURED or something you typed, the frame, the integrator and its
tolerance, the output spacing, whether the spacecraft came from the catalog, and the
cache policy. Then 9b prints what was actually fetched, 9d prints the resolved driver
window and gravity/DE440/EOP status, and 9e prints the integration.

**New satellite?** Set `USE_CATALOG=false` and fill `SC_MASS_KG`, `SC_AREA_M2`,
`SC_CD`, `SC_CR`. If the satellite IS in `itsg_catalog.csv`, leave them `[]` and its
properties are used. `itsg.list` prints the catalog with coverage.
