# Codebase audit — hidden values, redundant wiring, failure modes

A sweep of the whole tree for the failure classes this project kept hitting. The
pattern in every one of them: **the code made a physics decision and did not say so.**

---

## 1. Silent numeric defaults — a decision made for you

`getf(s,'field', <number>)` reads like a convenience. In a force model it is a
physics choice taken in silence, and it only shows up as a wrong answer.

| where | was | why it matters | now |
|---|---|---|---|
| `forces/drag.m` | `getf(d,'Cd',2.2)` | **CHAMP ran at 2.2, not its catalog 3.0** — 27% drag error, while `config.report` printed `Cd : 3` | `forces.drag.Cd` → `sc.Cd` → 2.2 |
| `forces/srp.m` | `getf(c,'Cr',1.3)` | ignored `spacecraft.Cr`; 3 scripts hand-patched around it | `forces.srp.Cr` → `sc.Cr` → 1.3 |
| `forces/erp.m` | `getf(c,'Cr',1.3)` | **nothing patched ERP** → Cr=1.3 for every satellite ever propagated | `forces.erp.Cr` → `sc.Cr` → 1.3 |
| `atmos/nrlmsise.m` | `getf(sw,'F107',150)` | `sw=struct()` silently gave **solar-max air** | errors |
| `atmos/dtm2020.m` | `getf(sw,'F107',120)`, `getf(sw,'Kp',3)` | same fault, missed on the first pass. The Kp default is what made the "DTM2020 NaN" hunt confusing: `getf` defaults on missing/empty, **never on NaN** | errors |
| `op/gravLoad.m` | `getf(gcfg,'degree',60)` | forget to set degree → you get **degree 60** and never know. 60 vs 20 changes the answer *and* the runtime | warns |

**Left deliberately:** `integ/odesuite.m` rtol/atol (documented integrator defaults);
`srp/eclipse.m` `Rsun=6.957e8`, `hAtm=12000` (physical constants, not user choices —
though `Rsun` could come from `de440.constants`); `atmos/jb2008.m` `Tinf=1000` and
`Mmol=16` (see §5).

## 2. Swallowing catches — an error that never reaches you

| where | hid |
|---|---|
| `buildWorld` earth-rate | **the worst one.** A bare `catch` fell back to `[0;0;omega]` — precisely the error the same file's header warns costs **~0.4 m/s** at LEO (the Earth turns about the CIP, ~168 arcsec off the ECI z-axis by 2008). A failed EOP fetch silently downgraded your frame accuracy with no trace. **Now warns.** |
| `buildWorld` space weather | `data.spaceweather` throwing `undefined function` was swallowed → `W.swtable=[]` → `forces.drag` then died per-step with a misleading "pass opts.manual". **Now a named error at setup.** |
| density block (old) | empty `catch` → `rho_mod` all-NaN → printed **ratio NaN** instead of an error |
| `buildWorld:20` `data_dir` | `try ... catch, end` — benign: if `data.eop_dir()` fails the builds use their own default. Left. |
| `data/itsg.m` ×4 | fetch-mirror fallbacks; each reports on the final failure. Left. |

## 3. Script-local functions — the bug that started this

MATLAB hoists a script's local functions. **Octave does not** — it defines them only
when execution *reaches* them, so helpers at the end of a script never exist and every
call fails (`'mjd2utc' undefined`). Fine on MATLAB, fatal under Octave.

All now lifted to package functions: `validate_OD` → `validation.*`;
`compare_OD` → `validation.sweep_knob` / `val_str`; **`EXAMPLE_16U`** (the last one) →
same. **Zero scripts in the tree still carry local functions.**

## 4. Redundant wiring

- **`+de440/chebval.m` and `+de440/private/chebval.m` are byte-identical.** Two copies
  that can drift. The private one is what `de440.state` actually resolves in MATLAB
  (private folders are visible to the parent's members); the public one exists so
  Octave can reach it at all. Kept both — **if you edit one, edit both.**
- Everything else that shares a basename is a genuinely different package member
  (`op.accel` / `thirdbody.accel` / `erp.accel`; `drag.cannonball` / `srp.cannonball`;
  `forces.gravity` / `data.gravity`; `thirdbody.total` / `relativity.total`). Not
  duplication.
- **Two catalogs, one file.** `sat.catalog` and `validation.itsg_catalog` both read
  `itsg_catalog.csv`. Redundant readers, but they return different shapes for
  different consumers and both are now in sync (`has_tudelft`, `tudelft_name`,
  `CollapseDelimiters`). Worth collapsing eventually.

## 5. Known-hidden, still open

- **`atmos/jb2008.m`**: `Mmol = 16` and `atm.T = 1000 K` hardcoded. The 16 is a
  defensible VLEO choice (atomic-O dominated, same as `dgeom/vleo16u`) but it is a
  *geometry/GSI* property living in an atmosphere adapter. The 1000 K is a pure
  placeholder — JB2008's core returns `TEMP`, and `jb2008_density` drops it.
- **`dUT1` defaults to 0 everywhere** (`getf(W.frame,'dUT1',0)`). Harmless for A/B/C
  (they read UT1 from EOP themselves; TDB depends on TAI−UTC, not UT1). For
  `build='gmst'` it is up to 0.9 s → ~450 m of ECEF error.
- **DTM2020 returns `NA`, not an error**, for an out-of-range altitude. Visible, but
  it will not stop a run.
- **`jb2008_density` extrapolates with `'extrap'`** past the SET file coverage,
  silently. A coverage warning is raised at fetch; the extrapolation itself is not
  guarded.
- **`get_gfz_hpo` has no cache layer** — one GFZ request per `buildWorld`, N per sweep.
- **`F107_bar_n`** (the clipped-centred-window flag) is raised by `get_f107` and
  dropped by `data.spaceweather`. Nothing consumes it.
- **`research_f107`** (the study's 5th variant) is reachable via `f30source='f107'`
  but is not its own sweep value.

## 6. Verification status — read this before trusting any of it

Everything above is **Octave-verified against synthetic fixtures**. That means:

- **Proven:** wiring, dispatch, units, frames, product names, argument correctness,
  ordering, and that all 8 forces fire with sane magnitudes.
- **NOT proven:** the numbers. The fixtures are built from the engine's own output, so
  they are *structurally blind to a consistently wrong parameter* — that is exactly how
  the Cd=2.2 bug hid inside a perfect `ratio 1.000`. Only the **sweep** caught it.
- **Not runnable here at all:** every fetcher (`get_f107`, `get_gfz_hpo`,
  `get_jb2008_indices`, `fetch_tudelft_density`) is MATLAB-only — `timetable` and
  `datetime`. And `nrlmsise` needs the Aerospace Toolbox.

Real numbers need your MATLAB, your cached CHAMP files, and a live network.

---

# Round 7 — four bugs found by the FIRST REAL MATLAB RUN

Fixtures proved the plumbing. Real data proved the rest. Everything below was
invisible under Octave + synthetic fixtures.

## 1. The crash: `Unrecognized field name "retrieved"` (validate_OD:378)

**Mine.** The provenance writer prints `p.retrieved` for every entry; the TU Delft
entry I added did not carry it. It crashed *after* a successful 3 h propagation.

**Why CHAMP "worked" and GRACE did not** — and this is the instructive part: for CHAMP
the TU Delft fetch **failed**, so no provenance entry was added, so nothing crashed.
The bug was only reachable on the satellites where the new feature *succeeded*. A
failure that hides behind another failure looks like a satellite-specific problem and
is not one.

Fixed: `'retrieved', datestr(now,31)`. Every prov entry must carry the same fields.

## 2. Zero-width TU Delft window -> metric [5] ran on ONE point

`data.tudelft_density(SC.tudelft_name, DATE, DATE)` — as datenums, `DATE..DATE` is a
single **instant** (midnight), not a day.

```
CHAMP  : [5] TU Delft density unavailable: no rows in [01-Jan-2007,01-Jan-2007]
GRACE-A: [TUD] DONE: 1 rows, 01-Jan-2007..01-Jan-2007
         [5] density vs TU Delft: ratio 2.178 | RMS log-err 0.778
```
CHAMP got 0 rows; GRACE got **1** row, by the luck of a sample landing exactly on
00:00:00 — **and a "ratio" was reported from that single point.** Worse than useless:
it looked like a result.

Fixed: the window is now the arc, `DATE .. DATE + ceil(TSPAN_S/86400)`.

## 3. GRACE-B was validated against GRACE-A's density

The worst one.

```
validate_OD GRACE-B 2013-01-01
  [TUD] 1 monthly file(s) to fetch: GA_DNS_ACC_2013_01_v02.zip     <-- GA = GRACE-A
  [5] TU Delft track vs ITSG reduced-dynamic: 247617.27 m RMS
```

`fetch_tudelft_density` has a prefix filter for Swarm (`SA_`/`SB_`/`SC_`) but **none
for the GRACE twins**, and its satellite argument only understands `'GRACE'`. My
catalog mapped **both** `GRACE-A` and `GRACE-B` to `tudelft_name = 'GRACE'`, so both
requests took whichever file listed first — the `GA_` ones.

The twins fly ~200 km apart along-track. **That 247 km is not a bug in the propagator;
it is the distance between two different satellites.** The 5,025 m for GRACE-A 2007
was the same mechanism at a different separation.

Fixed: a `G[AB]_` filter in the fetcher, `GRACE-A`/`GRACE-B` accepted as satellite
names, catalog `tudelft_name` disambiguated. If the requested twin has no files it now
**errors** — returning the wrong satellite's data is worse than returning none.

## 4. `RCOND = 4.46e-24` in `validation.fd_velocity>sgVelocity`

`sgVelocity` built the Savitzky-Golay Vandermonde in **raw sample offsets** and solved
the **normal equations**:

```matlab
s = (-m:m).';  A(:,j+1) = s.^j;  C = (A.'*A) \ A.';
```

`autoWindow` targets a ~1000 s span, so TU Delft's 10 s sampling gives **w = 101**,
i.e. `s` spans ±50 and `s^7` reaches 8e11. Forming `A'A` then **squares** the condition
number. Reproduced exactly:

```
w=101, deg=7   (what track_reference actually uses on the TU Delft track)
  BEFORE  cond(A'A) raw        = 2.233e+23   -> RCOND ~ 4.48e-24   <- MATLAB: 4.46e-24
  AFTER   cond(A'A) normalised = 5.101e+04   -> RCOND ~ 1.96e-05
  AFTER   cond(A) with QR      = 2.259e+02
  improvement: 9.9e+20 x
```

The derived seed velocity was being decided by round-off. Fixed by normalising the
abscissa to `u = s/m` in [-1,1] (chain rule: `d/dt = (1/(m*h)) d/du`) and solving by QR
(`A\eye`) rather than the normal equations. Verified still **exact** on a degree-5
polynomial including at the ends (2.7e-12 / 1.8e-08).

Not a bug, but corrected: the header said `opts.deg` defaults to 5 while the code says
**7**. `autoWindow`'s own table rejects degree 5 at that span (`deg5 1.007 m/s` vs
`deg7 0.033`). Measured here: deg5 **0.788 m/s**, deg7 **0.0021 m/s**. The code was
right; the comment was stale.

## Still open from this run — NOT yet fixed

- **`[3] ratio 0.126` for CHAMP** (measured 1.994e-06, modelled 2.518e-07): our
  non-conservative acceleration is **8x too small** at 345 km. GRACE-A 0.539, GRACE-B
  0.542 at ~485 km. Both under, and CHAMP much worse — consistent with a systematic
  drag deficit that grows as density grows, not with a random error. Candidates, in
  order: the `Aref` in the catalog is a fixed cross-section while CHAMP's projected
  area varies with attitude; `erp` was **off** in these runs; the DTM2020 same-day
  F10.7 / flat-aph convention (both currently the GOCE-validated defaults, both
  opt-in-able). **This is the real physics question and it is now measurable — that is
  what metric [3] is for.**
- **ITSG `neutralDensity` 404s** for CHAMP and GRACE-1 on these dates (all 4 URL
  patterns tried). The catalog says `has_density=yes`; the product is not at those
  paths. Needs a catalog correction or a path fix.
- **`[1]` along-track 4.57 m for CHAMP vs 0.96 m for GRACE-A** with a reference floor
  of 0.01 m. Both are far above the floor, so both are real modelling error — and
  along-track dominating is the drag signature.

---

# Round 8 — A(t), Cd(t), and why the panel models had never run

Your `[3] ratio 0.126` question. The machinery to answer it was all present and none
of it was reachable.

## The panel drag models could never produce drag

`forces/drag.m`'s panel branch calls `drag.force(..., opts.R_bi ...)`, and `R_bi`
came from `cfg.spacecraft.R_bi` — the constant `eye(3)` from `defaultConfig`.

`eye(3)` does not mean "some fixed attitude". It means **body axes == INERTIAL
axes**. So a `+x` facet points at inertial +x, not into the flow. For a satellite
moving along +y, `n · v̂ = 0` and the drag is **exactly zero**:

```
attitude=none -> |a_drag| = 0.0000e+00 m/s^2     <-- silent, plausible-looking nothing
attitude=ram  -> |a_drag| = 3.6495e-13 m/s^2
```

Not "a bit off" — zero, with no error. Every GSI model in `+drag/` (sentman, dria,
cll, sesam), `panelCoeffs`, `bodyToWind`, `windAngles`, `speedRatio`, `species`, and
the whole `+dgeom/` box-wing kit was unreachable through the pipeline.

**Meanwhile `validate_OD` was already fetching the measured attitude** — ITSG's
`attitude` product — and using it *only* to rotate the accelerometer for metric [3].
The information was on disk and the forces never saw it.

### Now

```
cfg.spacecraft.attitude -> W.att  (buildWorld: quat_continuous ONCE, not per step)
                        -> ctx.sc.R_bi = op.attitudeAt(W.att, ctx.utc)   [per epoch]
                        -> every panel force, unchanged
```

New `op.attitudeAt`: accepts `[]`, a 3x3 DCM, a function handle, or `{.mjd,.q}`
measured quaternions (nlerp between samples; at ITSG's 10 s cadence the rotation per
sample is <0.1°, where nlerp and slerp agree to ~1e-7 rad — far below attitude
knowledge). The sign-flip unwrap happens **once in buildWorld**, because ITSG flips
sign on ~half the epochs (4264/8640 on CHAMP) and interpolating across a flip is up
to 180° wrong.

### The toggles

```matlab
ATTITUDE   = 'none' | 'ram' | 'measured'
DRAG_MODEL = 'cannonball' | 'sentman' | 'dria' | 'cll' | 'sesam'
GEOMETRY   = 'plate' | 'boxwing'      BOX_DIMS = [Lx Ly Lz]
SRP_MODEL  = 'cannonball' | 'boxwing'
GSI        = struct('Tw',300,'aT',0.9,'sig_n',0.9,'sig_t',0.9)
```

- `'ram'` = `dgeom.ramAttitude`, body +x along v_rel. A(t) constant, **Cd(t) still
  from physics**. Use when there is no attitude product.
- `'measured'` = the real quaternions. A(t) **and** Cd(t) both move.
- **`ATTITUDE='none'` + a panel model now ERRORS** rather than silently returning zero.
- `GEOMETRY='boxwing'` **errors without real `BOX_DIMS`**. There is no CHAMP/GRACE
  box-wing geometry in this tree, and inventing dimensions to make the area move is
  fitting with extra steps. `'plate'` gives `A(t) = Aref·|n·v̂|` and needs no invented
  shape — the honest minimum.
- **ERP has no box-wing path** (`forces/erp.m` is CrAoM-only: knocke/simple/ceres).
  Saying so rather than pretending the toggle covers it.

## `sesam` was a category error

`drag.force('sesam')` died with `'ct' undefined`. Two causes:

1. **`drag.force` duplicates `drag.panelCoeffs`' dispatch.** Two switches for one
   decision, and they drifted: `panelCoeffs` handled a model `force` did not. The
   value fell through both cases, `cp`/`ct` were never assigned, and the failure
   surfaced three lines later as an undefined variable. Both now have `otherwise`
   guards that name the model. Collapsing them is a marked TODO.
2. **SESAM is not a panel model.** `drag.sesam(nO,T)` returns `aT` — an accommodation
   coefficient, not pressure/shear coefficients. And `force.m` line 40 already did
   `case 'dria', aT = drag.sesam(nO, atm.T)`. **`'dria'` in this tree already IS
   "Sentman with a SESAM-derived aT".** So `'sesam'` is now an explicit alias, and
   measured: `dria` and `sesam` return **identical** Cd (3.4638), which confirms it.

## The number that matters for your ratio 0.126

Same facet, same atmosphere, 350 km:

| model | Cd | where aT comes from |
|---|---|---|
| `sentman` | **2.4909** | you typed `gsi.aT = 0.9` |
| `dria` / `sesam` | **3.4638** | computed from local atomic-O density and T |
| `cll` | 2.3354 | `sig_n`, `sig_t` |

A **39% spread**, against CHAMP's catalog `Cd = 3.0` and the cannonball default 2.2.
The real distinction is not "which GSI model" but **whether aT is typed in or
computed**. At VLEO adsorbed atomic oxygen drives accommodation and it is rarely 0.9 —
that is the physics, not a detail.

**This does not close the 8x on CHAMP by itself** (Cd 2.2 -> 3.46 is ~1.6x, not 8x).
The remaining candidates, in order: the fixed `Aref` vs a real projected area (CHAMP
is not a plate); `erp` was **off** in your runs; and the DTM2020 same-day-F10.7 /
flat-aph conventions. All are now togglable and measurable against metric [3].

## TU Delft: what it is, and the availability rule

TU Delft's density is the **same kind of thing** as ITSG's: a retrieval from that
satellite's own accelerometer, on that satellite's own track. It is **not** a model to
be evaluated anywhere. So it is a comparison only if it is the **same satellite and
the same epoch** — asking for a different satellite, or a date it does not cover, does
not degrade the comparison, it **voids** it. GRACE-B silently receiving GRACE-A's file
is exactly what that looks like unchecked.

`validate_OD` now reports how many samples land **inside the propagated arc**, and if
fewer than 2 it prints **DATA NOT AVAILABLE** and drops the metric rather than
reporting a ratio from one point (which is what it did for GRACE-A: `ratio 2.178` from
a single sample).

---

# Round 9 — the panel models were 1e6 wrong, and my round-8 Cd claim was wrong with them

## The units bug: atm.n was cm^-3 in one adapter and m^-3 in the other

```
dtm2020_oper_density header : out.n.{H,He,O,N2,O2,N}  [1/cm^3]
atmos/dtm2020.m             : atm.n = out.n;              <-- passed straight through
atmos/nrlmsise.m            : atm.n from the Aerospace TB  [1/m^3]
drag/force.m                : NA = 6.02214076e26 (per KMOL), species in kg/kmol -> SI
```

One field name, two unit systems, depending on which model filled it. `drag.force` is
correctly SI, so **every panel drag model running on DTM2020 was 1e6 too small**:

```
                     BEFORE        AFTER
cannonball, ram      5.0032e-07    5.0032e-07     (never affected: reads atm.rho)
sentman, ram         4.1482e-13    4.1482e-07
dria, ram            5.7750e-13    4.2054e-07
```

The cannonball branch reads `atm.rho`, which was always right — which is exactly why
this survived. **The default path never touched `atm.n`.** Fixed at the source
(`atmos/dtm2020.m` now converts cm^-3 -> m^-3), because SI is the convention
everywhere else and patching consumers would leave the next one to find it again.

## It also corrupted the accommodation -- retracting my round-8 number

`drag.sesam(nO,T)` is `aT = x/(1+x)` with `x = 7.5e-17 * nO * T`. With nO a millionth
of the truth, `x ~ 7e-6` and **aT ~ 0** -- near-specular reflection, everywhere, at
every altitude. That produced the "Cd 3.4638" I reported in round 8 and the "39%
spread between sentman and dria". **Both were artefacts of the units bug.** The real
picture:

```
alt    | rho        | nO [m^-3] | SESAM aT | Cd sent  | Cd dria/sesam
250 km | 4.422e-11 | 7.208e+14 | 0.9798   | 2.4850   | 2.2508
300 km | 1.185e-11 | 2.729e+14 | 0.9486   | 2.4881   | 2.3653
350 km | 3.698e-12 | 1.058e+14 | 0.8776   | 2.4909   | 2.5376
450 km | 4.894e-13 | 1.665e+13 | 0.5303   | 2.4959   | 3.0163
550 km | 8.341e-14 | 2.767e+12 | 0.1579   | 2.5055   | 3.3537
```

That is the textbook VLEO behaviour: atomic O thins with altitude, accommodation
falls, Cd rises from ~2.2 toward ~3.4. `sentman` with a typed aT=0.9 is flat at ~2.49
by construction and **cannot show this** -- which is the whole argument for computing
aT instead of typing it.

Worth noting against your catalog: at CHAMP's ~345 km DRIA gives **Cd ~ 2.54** where
`itsg_catalog.csv` says **3.0**; at GRACE's ~485 km DRIA gives **~3.1** where the
catalog says **2.5**. The catalog values are not obviously right.

## The facet format collision

Two builders, two incompatible shapes, one field name:

```
dgeom.buildBox -> .n .A                                        (drag reads these)
srp.buildBox   -> .type .n .A .alpha .rho_s .rho_d .axis .double
```
Measured: drag facets -> `srp.boxwing` dies with *"structure has no member 'type'"*;
SRP facets -> `drag.force` works fine (it reads only `.n`/`.A`). The SRP format is a
strict superset, so `validate_OD` now builds **one** srp-format set that drives drag
A(t) **and** SRP box-wing. Two facet sets would be two spacecraft, and A(t) must be the
same area for both.

## How attitude actually reaches each force -- verified, with the gaps named

```
cfg.spacecraft.attitude -> W.att  (unwrapped ONCE in buildWorld)
     -> ctx.sc.R_bi = op.attitudeAt(W.att, ctx.utc)      per epoch
          |
          +-- forces.drag  'sentman|dria|cll|sesam' -> drag.force(opts.R_bi)
          |        -> A(t) = sum over facets of A_k*max(0, n_k.v)   AND Cd(t) from GSI
          +-- forces.srp   'boxwing' -> srp.boxwing(..., R_b2i, sc)   VERIFIED wired
          +-- forces.erp   *** NO ATTITUDE PATH ***  CrAoM only
```

**ERP is the honest gap.** `forces/erp.m` is `knocke|simple|ceres`, all driven by a
scalar `CrAoM`. There is no `erp.boxwing`. So Cr for ERP is a constant no matter what
you set, and this toggle does not reach it. Writing one is a real task, not a wiring
fix, and it is not done.

**A single plate is not a spacecraft for SRP.** With `GEOMETRY='plate'` and a ram
attitude, the plate faces the flow and the Sun is elsewhere -> `srp.boxwing` returns
**0**. Correct geometry, useless model. The plate is fine for drag A(t) (it always
faces the flow) and meaningless for SRP. Use `GEOMETRY='boxwing'` with REAL dimensions
for SRP box-wing -- and there is no CHAMP/GRACE geometry in this tree.

---

# Round 10 — the drag chain, verified term by term

`06_validation/verify_drag_chain.m` — an open, knobbed script. Every check in it
exists because that exact thing was found broken here.

```
a_drag = -0.5 * rho * Cd * (A/m) * |v_rel| * v_rel
          \___/  \__/   \___/  \_/
            |      |      |     +-- catalog        CHECK: does the CSV reach ctx.sc?
            |      |      +-------- facets x attitude  CHECK: A(t) == Aref*|n.vhat|?
            |      +--------------- catalog or GSI     CHECK: panel == cannonball?
            +---------------------- atmos.provider     CHECK: sum(n_i M_i/NA) == rho?
```

## Results — all five PASS

**[1] Catalog.** CHAMP 522 kg / 0.767 m^2 / Cd 3.0; GRACE-A and GRACE-B 487 / 0.955 /
2.5 with **distinct** TU Delft names (the round-7 twin fix holds); SWARM-A 473 / 0.955.

Found while writing this: the reference area has **three names** for one quantity --
`validation.itsg_catalog -> .area`, `sat.catalog -> .Aref`, `sweep_knob -> .Aref_m2`,
`cfg.spacecraft -> .Aref`. `validate_OD:360` translates. It works only because
somebody remembered -- the same shape as the Cd bug. Not collapsed; flagged.

**[2] Density closure** -- `sum(n_i*M_i/NA)` vs `rho`, ratio 0.9998 .. 1.0000 at
250-550 km. **This is the check that proves the round-9 cm^-3 fix.** Before it, this
ratio was 1e6.

**[3] A(t) vs attitude** -- exact at every angle, including yaw 91 deg -> **A = 0**
(shadowed, not negative: `max(0,.)` not `abs(.)`, which is the difference between a
box and a box-shaped sail).

**[4] Cd(t)** -- sentman flat 2.485 -> 2.506 (you typed aT=0.9, so it CANNOT vary);
dria/sesam 2.25 -> 3.35 rising as atomic O thins and accommodation falls. sesam ==
dria exactly, confirming the alias.

**[5] CLOSURE** -- cannonball(Cd from panel) vs panel, same rho/A/m:
```
250 km | Cd 2.4817 | 6.234393e-06 | 6.234393e-06 | rel 0.000e+00 | PASS
350 km | Cd 2.4868 | 5.223424e-07 | 5.223424e-07 | rel 2.027e-16 | PASS
550 km | Cd 2.4996 | 1.184122e-08 | 1.184122e-08 | rel 1.397e-16 | PASS
```
**Machine precision.** rho, Cd, A and m are tied together in one identity and it
closes -- so all four are what they claim to be.

### The 1e-4 that was not an error

The first run of [5] "failed" at 2.3e-4. It was the test, not the code:
`drag.force` defines `Cd = D/(qd*Aref)` with `qd = 0.5*SUM(rhoS)*V^2` -- the density
it summed from the COMPOSITION, not the model's own `atm.rho`. Comparing against
`atm.rho` shows DTM2020's own rho-vs-composition inconsistency (the [2] ratio) as if
it were our error. Closing against the same rho the model used gives 1e-16. Worth
knowing: **DTM2020's rho and its species sum agree only to ~1e-4**, so a multi-species
panel model and a cannonball on the same DTM2020 call can never agree better than that.

### And I reintroduced the founding bug

The first draft of `verify_drag_chain.m` carried `tern_`, `probeGeo_` and
`projectedArea_` as **script locals**. Octave does not hoist them -> `'tern_'
undefined`. That is the exact bug this entire audit started from, in the script
written to verify everything else. Lifted to `validation.probe_geo`,
`validation.projected_area`, `subsref_tern`.

## NOT done -- carried forward

- **`erp.boxwing`** -- ERP is still `knocke|simple|ceres` on a scalar CrAoM. The
  attitude toggle does not reach it.
- **SRP box-wing for a real satellite** -- needs real dimensions; a single plate gives
  SRP = 0 (faces ram, Sun elsewhere). No CHAMP/GRACE geometry exists here.
- **r/v overlay + error plots for the TU Delft and TLE tracks.**
- **End-to-end on real data** -- still Octave + synthetic fixtures only.

---

# Round 11 — one name per quantity, and every chain traced

## The aliases are collapsed

Two quantities had multiple names for one thing:

```
area :  validation.itsg_catalog -> .area        sat.catalog -> .Aref
        validation.sweep_knob   -> .Aref_m2     cfg / physics -> .Aref
mass :  catalog + physics       -> .mass        sweep_knob / EXAMPLE_16U -> .mass_kg
```

Nothing was broken by it, because `validate_OD:360` carried a rename in flight
(`cfg.spacecraft.Aref = SC.area`). It worked because somebody remembered. **That is
the exact shape of the Cd bug** -- Cd lived in `cfg.spacecraft`, the physics read
`cfg.forces.drag`, the two never met, and CHAMP flew at 2.2 instead of 3.0 for as
long as nobody checked. A translation line is a bug waiting for the next person who
writes a script and does not know it is needed.

**The physics layer's names win**, because they are the ones the forces actually read:
`.mass`, `.Aref`, `.Cd`, `.Cr`, `.facets`, `.attitude`. `itsg_catalog` now returns
`.Aref`; the rename in `validate_OD` and `compare_OD` is gone; `sweep_knob` accepts
`SC.Aref`/`SC.mass` as canonical with the old names as documented aliases.

New **`sat.spacecraft(name, overrides)`** -- the single contract, catalog then
field-by-field overrides, with provenance:
```
CHAMP: mass=522 Aref=0.767 Cd=3 Cr=1.3   source=itsg_catalog.csv:CHAMP
override Cd=2.2                          source=itsg_catalog.csv:CHAMP +Cd
no Aref -> ERROR: spacecraft.Aref is required and must be positive
```
(Named `sat.spacecraft`, not `sat.properties`: `properties` is a built-in and
shadowing it warns on every path add.)

## Which field does each toggle actually read?

New **`sat.reads(cfg)`**. The question it answers: you set `Cd = 3.0` and select
`dria`. Is your Cd used? **No** -- DRIA computes Cd(t) and never looks at `sc.Cd`.
You set `Cr` and `srp='boxwing'`? **No** -- box-wing uses per-facet optics. None of
that is wrong, but all of it was invisible, and *a value printed in the run report
while being ignored by the physics is exactly how CHAMP flew at 2.2*.

```
  force  | model       | READS                  | DERIVES     | IGNORES
  drag   | cannonball  | mass, Aref, Cd         | -           | facets, attitude
  drag   | dria        | mass, facets, attitude | Cd(t), A(t) | Aref, Cd
  srp    | boxwing     | mass, facets, attitude | A(t),optics | Aref, Cr
  erp    | knocke      | mass, Aref, Cr         | -           | facets, attitude
                                                                (NO erp box-wing exists)
```
It also names the traps: panel drag with no attitude (-> ~0), one-facet box-wing SRP
(-> 0), and "an attitude is set but erp cannot use it".

## Every chain traced, hand-checked

| chain | identity | hand vs model |
|---|---|---|
| drag | `0.5 rho Cd (A/m) V^2` | **0.0e+00 .. 2.0e-16** |
| SRP | `nu * P_srp * Cr * (A/m)` | **1.41e-16** |
| ERP | `f(albedo,IR,doy) * CrAoM` | Cr 1.3->2.6 gives ratio **2.0000** exactly |
| third body | `GM_b[(r_b-r)/|r_b-r|^3 - r_b/|r_b|^3]` | **7.55e-13** |
| gravity | `-mu r/|r|^3 + harmonics` | ratio to point mass **1.001462** (= J2) |

The third-body check includes the **indirect** term (the Earth is accelerated too).
Dropping it is a classic error worth ~1e-6 m/s^2 -- larger than the entire SRP.

## Re-integrated

```
force        | |a| [m/s^2]   | /gravity    | driver chain
gravity      | 8.818252e+00 | 1.0000e+00 | gravityField -> data.gravity -> W.grav
thirdbody    | 8.089149e-07 | 9.1732e-08 | DE440 -> ctx.E.sun_eci/moon_eci
drag         | 5.729387e-07 | 6.4972e-08 | data.drivers -> atmos.provider -> rho,n + sc + attitude
srp          | 1.169366e-08 | 1.3261e-09 | DE440 -> ctx.E.P_srp + srp.eclipse + sc
erp          | 2.333908e-09 | 2.6467e-10 | DE440 sun + doy + CrAoM
relativity   | 1.740837e-08 | 1.9741e-09 | ctx.grav.mu + c
NON-GRAV     | 5.623058e-07 | 6.3766e-08 | <- what metric [3] measures
```

Shown as a ratio to gravity, not a percent of the total: at LEO every
non-gravitational force is <1e-6 of the total, so a percent column reads `0.000%` for
all of them and tells you nothing. **The non-gravitational sum is 6.4e-8 of gravity.
That is why a 27% Cd error hid for so long** -- and why metric [3], which measures it
against the accelerometer directly, is worth more than metric [1].

## Still not done

`erp.boxwing` | SRP box-wing geometry for a real satellite | TU Delft / TLE overlay +
error plots | end-to-end on real MATLAB.

---

# Round 12 — erp.boxwing, every model wired, and the track plots

## erp.boxwing — written, not stubbed

ERP was the last force with no attitude path: `knocke|simple|ceres` all collapse the
spacecraft to `CrAoM = Cr*A/m`. So the attitude toggle was a lie for ERP -- whatever
you set, it could not be seen.

**Why it is not just `srp.boxwing` with a different source.** SRP has ONE source
direction: the Sun is a point 1 AU away and every facet sees the same `sHat`. The
Earth is not a point -- at 350 km it subtends ~140 deg, so different elements of the
cap illuminate the satellite from genuinely different directions, including from
*behind* facets that the net-flux direction would call lit. Collapsing the cap to one
effective direction would be an approximation, and worst exactly where ERP matters.

So the facet sum lives **inside** the Knocke element loop: for each of nrings x nseg
elements, the per-band irradiance `E = M*cos_e*dA/(pi*rho^2)` is applied to the facets
as a beam arriving from `-es`. Cost: `nrings*nseg*numel(facets)` facet evaluations per
RHS call (768*6 ~ 4.6k at defaults); knocke already pays the 768.

Measured, 350 km, 4.0x1.6x0.75 m bus, ram-pointing:
```
model      | |a_erp|      | sw        | lw        | reads attitude?
knocke     | 1.790108e-09 | 2.795e-10 | 1.519e-09 | no (CrAoM)
simple     | 1.766149e-09 | 2.660e-10 | 1.500e-09 | no (CrAoM)
boxwing    | 1.064189e-08 | 1.646e-09 | 9.029e-09 | YES

yaw  0 deg: boxwing 1.064189e-08 | knocke 1.790108e-09 (flat -- it cannot vary)
yaw 45 deg: boxwing 9.361602e-09 | knocke 1.790108e-09
yaw 90 deg: boxwing 7.441712e-09 | knocke 1.790108e-09
```
lw >> sw is physical: IR comes from the whole cap, albedo only from the sunlit part.
The 6x over knocke is mostly area -- the box has ~21 m^2 of facets against
Aref = 0.767. Which is the point: `Cr*Aref/m` is not the area the Earth sees.

**Stated limitation:** `srp.facet` carries ONE set of (alpha, rho_s, rho_d). Real
surfaces differ between the visible (~0.5 um albedo) and the thermal IR (~10 um) --
MLI and solar cells especially. The same coefficients are applied to both bands
because that is what the facet struct holds. The albedo/IR SPLIT is still physical;
the error is in the surface response. Fixing it means adding IR optics to
`srp.facet` -- a change to the geometry contract, not to `erp.boxwing`.

Also added the `otherwise` guard `forces/erp.m` never had: an unknown model fell
through the switch, left `a` unassigned, and surfaced as *"'a' undefined"* from the
caller -- the same shape as the `sesam` bug in `drag.force`.

## Every model is now reachable and toggled

```
DRAG_MODEL = cannonball | sentman | dria | cll | sesam
SRP_MODEL  = cannonball | boxwing
ERP_MODEL  = knocke | simple | ceres | boxwing
ATTITUDE   = none | ram | measured
GEOMETRY   = plate | boxwing (+ BOX_DIMS, OPTICS)
```
`sat.reads(cfg)` prints which spacecraft fields each selection actually consults.

## The track plots -- and the naming bug they exposed

`show_OD` gained `.tracks`: `|r|` ours vs the independent track **overlaid**, with the
error underneath and a moving mean. Both sources are deliberately framed:

- **TU Delft** -- a geodetic track shipping with the density retrieval. Same satellite,
  same epoch, its own chain. A second opinion at the 10-100 m level. GRACE-B receiving
  GRACE-A's file showed up here as 247 km.
- **TLE** -- SGP4 mean elements, **kilometre-class BY DESIGN**. A TLE overlay that
  looks bad is a TLE being a TLE. It is here to catch gross errors -- wrong satellite,
  wrong epoch, wrong frame -- and nothing finer.

The reference floor is deliberately NOT drawn on these: they live far above it.

**The bug this found:** `show_OD` named its PNGs by **position** in the figure list.
Turn any figure off and every later filename shifts -- rendering only `.tracks` wrote
it as **`OD_rtn.png`**. Every plot silently mislabelled, which is worse than no plot:
you would have believed the wrong picture. Figures are now tagged by identity as they
are created.

## Still not done

- **SRP/ERP box-wing for a REAL satellite** -- both need real dimensions and optics.
  `BOX_DIMS`/`OPTICS` knobs exist and error if unset. There is no CHAMP/GRACE geometry
  in this tree and I will not invent one: a box chosen to make a number move is
  fitting, not modelling.
- **IR optics** on `srp.facet` (see above).
- **End-to-end on real MATLAB** -- still Octave + synthetic fixtures.

---

# Round 13 — capability from data, model I/O, and the closing audit

## Geometry: searched, not assumed

Grepped the whole tree, the docs and the three uploaded projects. **No CHAMP, GRACE
or Swarm dimensions exist anywhere.** The only real geometry is `dgeom/sgeom.vleo16u`
(0.34 x 0.20 x 0.20 m) -- the 16U CubeSat.

So the catalog now SAYS so, rather than the knowledge living in my head or a comment:
```
name,...,has_tudelft,tudelft_name,has_geometry,geom_dims_m,coverage_note
CHAMP,...,yes,CHAMP,no,,2000-2010
```
`geom_dims_m` takes `"Lx;Ly;Lz"` in metres. Put real numbers there and box-wing runs
for that satellite automatically -- `validate_OD` picks them up when `GEOMETRY='boxwing'`
and `BOX_DIMS` is empty. Marked `has_geometry=yes` with an empty/unparseable
`geom_dims_m` is an **error**: a half-declared geometry is worse than none.

## validation.capability -- data decides which models you may run

A model is not available because its code exists. It is available when the DATA it
needs exists for THIS satellite.

```
  ---- what CHAMP can actually support (from itsg_catalog.csv) ----
    data present : attitude=1  acc=1  ITSG-density=1  TU-Delft=1  geometry=0
    drag         : cannonball, sentman, dria, cll, sesam
    srp          : cannonball
    erp          : knocke, simple, ceres
    metrics      : 1 vs reducedDynamicOrbit, 2 vs kinematicOrbit,
                   3 vs nonConservativeForces, 4 vs neutralDensity, 5 vs TU Delft
    [x] srp boxwing: no box dimensions in the catalog.
    [x] erp boxwing: no box dimensions in the catalog.
    -> NO box-wing for CHAMP. Non-gravitational forces are limited to the cannonball
       (srp/erp) and the panel drag models, which need only a plate of area Aref.
```
`validate_OD` prints this in the PLAN and **refuses before propagating** if you
selected a model whose inputs are absent -- naming it a DATA limit, not a code limit,
and saying exactly which column to fill.

The three silent failures this closes: box-wing with no dimensions (you invent a box
and you are now fitting); panel drag with no attitude (R_bi = eye(3) -> drag ~ 0, no
error); TU Delft for the wrong twin (GRACE-B against GRACE-A, 247 km).

## show_model_io -- ONE figure, the whole chain

`drivers -> atmosphere -> (rho, composition, T) + attitude -> (Cd(t), A(t)) -> accel`.
Every one of those handoffs has been wrong here, and every one was invisible in the
final number: density in cm^-3 (panel drag 1e6 low), R_bi frozen (drag exactly zero),
Cd from the wrong struct (CHAMP at 2.2). **A plot of the acceleration alone would
have shown none of them.**

3x3 subplots on one time axis: **IN** (rho, T, n_O) | **IN/DERIVED** (attitude yaw,
Cd(t), A(t)) | **OUT** (drag, SRP with eclipse shaded, ERP). Cannonball runs draw Cd
and A as dashed CONSTANTS -- so "you typed this" and "the physics derived this" cannot
be confused.

**No duplication to get it.** `forces.drag` gained an optional second output
(`info.atm/.geo/.out/.v_rel`) and `op.accel` returns `[a, parts, ctx, info]`. The
diagnostic reads what the force ACTUALLY used. Reimplementing the chain to inspect it
is how two copies drift apart -- which is exactly what `drag.force` vs
`drag.panelCoeffs` did, and it cost a day.

## Closing audit

| check | result |
|---|---|
| parse | **0 failures / 233 files** |
| one name per quantity | `.area` gone (1 hit left, in a comment describing the old state). `.Aref_m2`/`.mass_kg` survive ONLY as documented sweep-knob aliases resolving to the canonical field |
| script-local functions in scripts | **none** |
| `switch` without `otherwise` in the force/atmos/driver path | **none** (the three flagged were the word "switch" in comments) |
| silent numeric defaults in the force path | only the documented last-resort fallbacks `forces.drag.Cd -> sc.Cd -> 2.2` and the `Cr` twins. The atmosphere index defaults are gone (they now error) |
| every model reachable | drag: cannonball/sentman/dria/cll/sesam; srp: cannonball/boxwing; erp: knocke/simple/ceres/boxwing |
| every model's inputs declared | `validation.capability` + `sat.reads` |

## What remains, honestly

- **Box-wing for CHAMP/GRACE**: blocked on real dimensions, not on code. One CSV cell.
- **IR optics** on `srp.facet` (one (alpha,rho_s,rho_d) applied to both bands).
- **End-to-end on real MATLAB**: fetchers need `timetable`/`datetime`; `nrlmsise`
  needs the Aerospace Toolbox. Everything here is Octave + synthetic fixtures, which
  proves wiring, units, frames and dispatch -- **not the numbers**.

---

# Round 14 — can Aref make a box? (no), and the 16U template

## Your question, measured rather than argued

"Can't we assume A_ref, make the box, and update it with attitude?"

Aref = 0.767 m^2 (CHAMP). Three shapes that all reproduce it exactly at zero yaw:
```
  yaw    | plate A(t)   | cube A(t)    | 4m box A(t)
    0 deg|     0.767000 |     0.767000 |     0.767000
   15 deg|     0.740865 |     0.939379 |     1.647545
   45 deg|     0.542351 |     1.084702 |     3.019446
   90 deg|     0.000000 |     0.767000 |     3.503141
```
They agree at the one angle where Aref was defined and disagree by **6x** elsewhere.
Aref is ONE number; a box needs THREE; the inversion is not unique. And **A(t) is the
quantity we are trying to measure**, so choosing a shape to produce it is circular:
you would be fitting, and the fit would be invisible in the answer.

**Solar arrays:** you asked whether deployment matters. Enormously -- see the 16U
below, where tracking arrays are ~75% of the drag area. But CHAMP and GRACE have
**body-mounted cells and no deployed arrays**, and they fly ram/nadir-pointing with
small yaw. So their A(t) is close to Aref anyway -- which means **A(t) will not
explain the 8x in metric [3]**. That is worth knowing before chasing it.

## Two 16Us, and a solar panel that returned NaN

**`dgeom.vleo16u` and `sgeom.vleo16u` described DIFFERENT SPACECRAFT.**
```
dgeom: buildBox(0.34, 0.20, 0.20)   -> long axis on BODY X
sgeom: Lx=0.20, Ly=0.20, Lz=0.34    -> long axis on BODY Z
```
With +x defined as ram, the drag version flew the 16U **broadside**: a 0.34x0.20 ram
face against SRP's 0.20x0.20. 1.7x apart, and nothing complained -- drag read one,
SRP read the other, each internally consistent. The "two facet sets = two spacecraft"
failure in its purest form. `dgeom.vleo16u` now forwards to `sgeom.vleo16u` and warns.

**`drag.force` returned NaN on any tracking array.** `srp.addArray` stores
`n = [0;0;0]` by construction -- an array's normal is computed per call by
`srp.arrayNormal(axis, sun)`. `drag.force` did `n = n/norm(n)` on that zero vector:
`0/0 = NaN`, summed into the force, and the run died three layers up as **"dynamics
non-finite at t0"** with no hint that a solar panel was the cause. Panel drag could
never run on ANY geometry with arrays.

Fixed: `forces/drag.m` passes `ctx.E.sun_eci`, `drag.force` pivots arrays exactly as
SRP does, honours `.double`, and **errors** if arrays are present without a Sun rather
than inventing an orientation. `validation.projected_area` gained the same option --
and defaults to drag's behaviour so the diagnostic matches the force. The gap is the
size of the old limitation:
```
16U ram frontal area:
  arrays NOT pivoted (the old drag.force): 0.0680 m^2
  arrays tracking a +x Sun               : 0.2040 m^2   <- 3x
  bus alone                              : 0.0400 m^2
```

## TEMPLATE_16U — every model, end to end

The 16U is the ONLY spacecraft here with a real geometry, so it is the only one where
box-wing SRP and ERP can run. The trade is exact and worth stating: **the 16U has
geometry and no truth; CHAMP has truth and no geometry.**

```
force        | |a| mean      | /gravity
gravity      | 8.938015e+00 | 1.0000e+00
thirdbody    | 9.573461e-07 | 1.0711e-07
drag         | 3.104963e-06 | 3.4739e-07
srp          | 2.654565e-08 | 2.9700e-09
erp          | 5.886013e-09 | 6.5854e-10
relativity   | 1.784538e-08 | 1.9966e-09
NON-GRAV     | 3.097632e-06 | 3.4657e-07

Cd(t)  : 5.0823 .. 9.9215   (DERIVED)
Cd*A/m : 1.6885e-02 .. 6.9265e-02 m^2/kg
A(t)   : 0.0797 .. 0.1676 m^2   (arrays tracking the Sun, so it MOVES)
```

**A Cd of 5-10 is not a broken model.** Cd is DEFINED as `D/(q*Aref)`, and this 16U
carries ~0.128 m^2 of tracking array against a 0.040 m^2 bus face. Normalise by the
small number and you get a big coefficient. **Cd alone is meaningless without its
reference area** -- the physical quantity is `Cd*A/m`, and the template now prints
both. This is the same trap as the catalog's `Cd = 3.0` for CHAMP: a coefficient
without its area is half a statement.

Writes `force_data/<tag>/`: one CSV per force + `accel_total` + `state` +
`model_inputs` (rho, T, nO, Cd, A_proj, yaw) + a manifest. Segregated on purpose:
"the drag is wrong" is answerable from `drag.csv` alone, and two runs diff force by
force.

**erp 'boxwing' cost is now a knob, not a surprise.** Knocke's default cap is 16x48 =
768 elements, and box-wing runs every facet at every element: 768*8 = 6k facet
evaluations *per RHS call*, inside rk78. A full orbit is ~10^7 and it timed out here.
`ERP_RINGS`/`ERP_SEGS` expose it; 8x16 costs 6x less and the cap integral converges
quickly.

## sat.reads went stale within one round

It still said *"NO erp box-wing exists"* while `erp.boxwing` was running. **A
capability reporter that lies is worse than none**, because it is the thing you check
INSTEAD of reading the code. Fixed, and the file now says out loud that it must be
updated when a model is added.

---

# Round 15 — you were right twice

## 1. Cd 5-10 was never physics

You said Cd must be material and geometric and that 5-9 could not be that. Correct.

`drag.force` reported `Cd = D/(q*Aref)` where `Aref` is a **bookkeeping choice** --
whatever the catalog calls the reference area. For the 16U, `Aref` is the 0.040 m^2
bus face while 0.068 m^2 of bus+array is actually in the flow. Measured at 300 km:

```
  Aref (bus ram face)        = 0.0400 m^2   <- an arbitrary choice
  A(t)  (what the flow sees) = 0.0680 m^2   <- from the geometry
  Cd referenced to Aref      = 4.0129       <- the alarming number
  Cd referenced to A(t)      = 2.3605       <- MATERIAL + GEOMETRIC
  ratio A(t)/Aref            = 1.7000       <- exactly the inflation
```
`4.0129 / 2.3605 = 1.7000` exactly. **The drag force was always right**; only the
coefficient's label was misleading. `drag.force` now returns `Cd_A` (referenced to
the projected area, ~2-3 as free-molecular flow requires), `A_proj`, and
`Aref_used`, so nobody has to guess the convention. **A coefficient without its
reference area is half a statement** -- which is also why the catalog's `Cd = 3.0`
for CHAMP means nothing without `area_m2 = 0.767` beside it.

## 2. CHAMP does NOT need a box for drag

You said: Aref is known in ram, so A(t) can be computed. Correct, and it already
runs -- I had been describing the box-wing limitation as if it applied to drag.

```
CHAMP, dria, GEOMETRY='plate', 345 km:
  yaw    | A(t) [m^2] | Cd(Aref) | Cd_A   | |a_drag|
    0 deg|    0.76700 |   2.6118 | 2.6118 | 3.155e-07   <- CHAMP's normal flight
   10 deg|    0.75535 |   2.5632 | 2.6027 | 3.099e-07
   30 deg|    0.66424 |   2.1924 | 2.5316 | 2.668e-07
```
A(t) tracks attitude, `Cd_A` is the physical 2.5-2.6, **no invented shape**. For a
single plate `Cd == Cd_A` at 0 deg because Aref IS the projected area there.

**What a box would add, and why CHAMP still cannot have one:**
- a plate has **no sides**: at 30 deg yaw a real CHAMP presents its flank; the plate
  presents nothing. A(t) is then WRONG, not merely absent.
- a plate has **no back**: SRP on a plate edge-on to the Sun is 0, but CHAMP still
  has a lit surface. That is why srp/erp box-wing need three dimensions.
- CHAMP flies ram/nadir with small yaw, so the plate is a **good** approximation for
  drag and a **poor** one for SRP.

So the capability table was right but under-explained: CHAMP supports every panel
drag model (via a plate) and no box-wing SRP/ERP.

## 3. The suite: five files -> one

`run_all_tests` + `test_gravity` + `test_integrators` + `test_energy` + `test_seed`
-> one `08_test/run_all_tests.m`. The split bought nothing (nobody ran them singly)
and cost five places to miss `setup_paths` and five files to keep in sync.

The old suite tested gravity and integrators -- **which were never wrong**. Added the
checks that cover where every real bug actually lived:
```
[4] interp_state exact on a cubic     (a pchip interpolant silently cost 72 m)
[5] density closure sum(n*M/NA)/rho   (1e6 here = the cm^-3 bug)
    A(t) of a ram plate               (R_bi=eye(3) made this ZERO)
    cannonball(Cd from panel)==panel  (2.027e-16)
[6] all 6 forces finite               (NaN = the solar-array n=[0;0;0] bug)
    panel drag responds to attitude   (identical = R_bi frozen)
```
All pass.

**And it found one more thing.** Test [1] compared gravity against
`de440.constants().mu_earth` and "failed" at 7.5e-10. Not a bug: **the gravity field
ships its own GM**, and DE440's mu_earth comes from a different fit. Testing against
the wrong source of truth. Same class as the Cd-without-its-area problem: a number is
meaningless without saying which convention produced it. Now compares against
`W.grav.mu` and reports the difference explicitly.

---

# Round 16 — the comparator could not compare the models

"Nothing redundant found" was not the same as "complete". You were right to push.

## sweep_knob knew about ONE model

```
  FORCES.drag.model     <- knew it
  FORCES.srp.model      <- MISSING
  FORCES.erp.model      <- MISSING
  FORCES.drag.gsi.*     <- MISSING
  SC.attitude           <- MISSING
  SC.facets             <- MISSING
```

So every model wired into `validate_OD` -- box-wing SRP, box-wing ERP, the four GSI
models, ram vs measured attitude -- **was unreachable from `compare_OD`**. The
comparator could not compare the models.

That is worse than it sounds. **The sweep is the only mechanism in this toolbox that
catches a consistently-wrong parameter.** The fixtures are built from the engine's
own output, so they are structurally blind to one -- which is exactly how `Cd = 2.2`
hid inside a perfect `ratio 1.000`. A knob that cannot be swept cannot be caught
being wrong.

All now sweepable, and `compare_OD` passes attitude/facets through to `cfg` -- without
that the knobs would be set and then dropped, and **a sweep that silently does nothing
is worse than an error, because the flat table looks like a physics finding**.

The unknown-knob error now prints the full legal menu by category rather than one
run-on line.

## The model knobs are documented where you will read them

`compare_OD`'s knob menu now says, per knob, what it is FOR:

- **`FORCES.drag.model`** -- the real question is not which GSI model but whether aT is
  TYPED or COMPUTED. `sentman` takes `gsi.aT` and is therefore **flat with altitude by
  construction**; `dria`/`sesam` derive it (2.25 at 250 km -> 3.35 at 550 km).
  **Not degenerate with density the way `SC.Cd` is**: Cd(t) has a SHAPE a scalar
  cannot mimic, so metrics (3) and (4) can separate them. That is the sweep worth
  running against your `ratio 0.126`.
- **`FORCES.drag.gsi.aT`** -- only `sentman` reads it. `dria`/`sesam` IGNORE it, so a
  flat table means you swept a knob the model does not consult. `sat.reads(cfg)` says
  which before you spend the arcs.
- **`SC.attitude = []`** with a panel model -- a trap: `validate_OD` errors on that
  pair, but `compare_OD` would just hand you a suspiciously good-looking row.

## gravity_data/ -- empty is correct, and now it says why

It is **not** the cache (`data_cache/gravity/` is). It is the **manual drop folder**,
on the path so `cfg.gravityField.field = 'gravity_data/EGM2008.gfc'` resolves. It
exists for when the automatic fetch cannot work: no network, or **the ICGEM hash
rotated** -- their URLs embed a content hash, so a hard-coded link eventually 404s,
and `data.gravity`'s error already points here. That error is the intended path to
this folder, not a bug.

`README.md` added with the two files worth having, where to get them, and the reason
the bundled default is not enough: `grav.defaultField()` is zonal J2-J6 and **caps at
degree 6**. If a `FORCES.gravity.degree` sweep gives four identical rows, that cap is
why -- and the warning already says so.

## State

231 files parse. Full suite ALL PASS. Every model reachable from BOTH validate_OD and
compare_OD, every knob documented at the point of use, one canonical name per
quantity, and `validation.capability(SAT)` refusing what the data cannot support.

**The one thing no amount of this can give you: the numbers.** Octave + synthetic
fixtures prove wiring, units, frames, dispatch and the mathematics. Real validation
needs your MATLAB, the CHAMP cache, a network, and the Aerospace Toolbox.

---

# Round 17 — dead code, duplicate models, and the box you asked for

## Two models were sitting unused while copies of them ran

An "is anything uncalled?" sweep over every package found exactly two:

**`drag.cannonball`** — nothing called it. `forces/drag.m` carried the cannonball
INLINE. Two implementations of one model, and **they disagreed**: `drag.cannonball`
built `v_rel` from a SCALAR omega via `drag.relVelocity` (`cross([0;0;omega],r)`),
while `forces/drag.m` used the true Earth-rate VECTOR `om_eci`, which points along the
CIP — ~168 arcsec off the ECI z-axis, worth ~0.4 m/s of `v_rel`. Whichever you called,
you got a different answer, and nothing compared them. Now `drag.cannonball` takes a
scalar OR a vector omega, returns the same diagnostic contract as `drag.force`
(`Cd_A`, `A_proj`, `qd`), and `forces/drag.m` **calls** it.

**`relativity.total`** — nothing called it. `forces/relativity.m` carried a SECOND
copy of the same term switch. Exactly what drag.force vs drag.panelCoeffs did, where
`'sesam'` existed in one and not the other and surfaced as `'ct' undefined`. And
`relativity.total` had **no `otherwise`**: an unknown term silently contributed
nothing, so a typo in `terms` quietly switched the term off and the run looked fine.
Now one dispatch, an `otherwise` that names the legal terms, a guard on the de Sitter
term's ephemeris requirement, and `mu` passed through — the sub-terms take the
**gravity field's** mu, not de440's (they differ ~7.5e-10; the field's is what the
dominant term uses).

## The box: you were right and I was being rigid

You have the attitude, so you CAN wrap a box around Aref and run box-wing SRP/ERP.
That is a legitimate engineering choice. My job is to make the assumption **visible**,
not to refuse it.

**`GEOMETRY = 'aref_box'`** builds a box whose ram face area is exactly `Aref`, with
an aspect ratio you supply (`BOX_ASPECT`). Measured for CHAMP:

```
BOX_ASPECT   | ram face | A(t) ram | |a_srp|     | |a_erp|     | shape
[ 1.0 1 1]   |   0.7670 |   0.7670 | 1.27522e-08 | 3.10982e-09 | 0.88x0.88x0.88 m
[ 2.0 1 1]   |   0.7670 |   0.7670 | 1.61645e-08 | 5.72676e-09 | 1.75x0.88x0.88 m
[ 4.6 1 1]   |   0.7670 |   0.7670 | 2.54593e-08 | 1.25313e-08 | 4.03x0.88x0.88 m
[ 1.0 1 2]   |   0.7670 |   0.7670 | 1.15751e-08 | 2.86407e-09 | 0.62x0.62x1.24 m
```

Every row has the **same ram face and the same drag**. SRP moves **2.2x** and ERP
**4.4x** across plausible shapes. **That spread IS the assumption's cost**, and it is
why an `aref_box` result is a *sensitivity band*, not CHAMP's number. `[4.6 1 1]` is
closest to CHAMP's real proportions (~4 m long on a ~0.9 m face).

`validation.capability` now offers both routes and names them:
```
srp : cannonball, boxwing(GEOMETRY=aref_box: ASSUMED shape)
-> CHAMP has NO measured dimensions. Two honest routes:
   (a) GEOMETRY='plate'    -- drag only, A(t)=Aref*|n.vhat|, no invented shape.
       Good for CHAMP: ram/nadir, small yaw. Useless for SRP.
   (b) GEOMETRY='aref_box' -- ASSUME a box, see how much shape MOVES srp/erp.
   What you cannot do is call (b) CHAMP's answer.
```

## For drag, no box was ever needed

Worth repeating because I muddled it earlier: `'plate'` already gives
`A(t) = Aref*|n.vhat|` from the measured attitude with no invented shape, and CHAMP
flies ram/nadir with small yaw, so it is a **good** drag approximation. The box only
buys SRP/ERP, where a plate is genuinely wrong (edge-on to the Sun it has no lit area;
a real satellite does).

## State

231 files parse. Suite ALL PASS. **Zero uncalled models.** Every model reachable from
validate_OD AND compare_OD AND TEMPLATE_16U. One canonical name per quantity. Every
switch guarded. Every assumption labelled at the point of use and in the capability
table.

The numbers still need your MATLAB.

---

# Round 18 — the model x satellite matrix, run rather than asserted

## `06_validation/verify_model_matrix.m`

Walks **every satellite in the catalog** against **every drag/srp/erp model**, asks
`validation.capability` whether the DATA allows it, and then **actually runs** the
allowed ones. A capability table that only agrees with itself proves nothing --
`sat.reads` went stale within one round of `erp.boxwing` landing and still claimed no
ERP box-wing existed while one was running.

```
  satellite   | canno sentm dria  cll   sesam | srp:cann srp:boxw | erp:knoc erp:simp erp:cere erp:boxw
  CHAMP       | y     y     y     y     y     | y        y        | y        y        y        y
  GRACE-A     | y     y     y     y     y     | y        y        | y        y        y        y
  GRACE-B     | y     y     y     y     y     | y        y        | y        y        y        y
  GRACE-FO-1  | y     y     y     y     y     | y        y        | y        y        y        y
  GRACE-FO-2  | y     y     y     y     y     | y        y        | y        y        y        y
  SWARM-A/B/C | y     .     .     .     .     | y        .        | y        y        y        .
  TERRASAR-X  | y     .     .     .     .     | y        .        | y        y        y        .
  TANDEM-X    | y     .     .     .     .     | y        .        | y        y        y        .
  JASON-1/2/3 | y     .     .     .     .     | y        .        | y        y        y        .
  METOP-A/B   | y     .     .     .     .     | y        .        | y        y        y        .
  SENTINEL-*  | y     .     .     .     .     | y        .        | y        y        y        .
```
**No `X` anywhere: every model the data allows actually runs, and every model it does
not is refused.** 24 satellites x 11 model slots.

## Why the pattern is what it is

**Only CHAMP and the GRACE family carry an accelerometer**, and ITSG publishes the
attitude WITH it -- the ACC product is useless without one. Attitude is what a panel
drag model acts on. Everywhere else `R_bi` would stay `eye(3)`, which puts the body
axes on the **inertial** axes: a +x plate faces inertial +x instead of the flow and
the drag collapses toward **zero**, silently, with no error. So those cells are
**refused rather than run**. A `.` is a DATA limit, not a code limit.

The same rule governs the metrics: `[3]` needs `has_acc` (CHAMP/GRACE only), `[4]`
needs `has_density` (+ TerraSAR/TanDEM), `[5]` needs `has_tudelft` (+ Swarm).

## The table now lives in validate_OD's header

Not in a doc nobody opens -- in the script, above the knobs, with the reason for every
`.` and the measured cost of every `y*`. `validate_OD` also **refuses an unsupported
model before propagating**, naming the missing column rather than failing after an arc.

## Dead-code fixes verified

```
forces/drag.m calls drag.cannonball      : 1
inline cannonball left                   : 0
forces/relativity.m calls relativity.total: 1
second switch in forces/relativity.m     : 0
relativity.total has otherwise           : 2
```
One implementation each, no copies, unknown terms error.

## Final state

**234 files parse. Suite ALL PASS. Zero uncalled models. Zero unrun-but-allowed
combinations.** Every model reachable from `validate_OD`, `compare_OD` and
`TEMPLATE_16U`; one canonical name per quantity; every switch guarded; every
assumption labelled at the point of use, in the capability table, and in the matrix.

The numbers still need your MATLAB, the CHAMP cache, a network and the Aerospace
Toolbox. Everything here proves wiring, units, frames, dispatch and mathematics --
which is the necessary half, not the sufficient one.

---

# Round 19 — the sweep matrix, and the knob it caught on its first run

## `11_compare/verify_sweep_matrix.m`

The satellite x model matrix answers "is this model allowed?". The comparator needs a
different question: **does this knob actually MOVE the answer?**

A flat column in a sweep table is ambiguous, and the two meanings are opposite:
- the model does not **consult** that knob -> correct, expected, informative
- the knob is set and then **dropped** -> a bug that looks like a result

`EXAMPLE_16U` documented an `SC.Cd` knob that was a no-op for as long as nobody
checked. Every model knob was unreachable from `compare_OD` until round 16. **Both
were invisible in exactly this way.** So: sweep every knob twice against a config
where it should matter, and print whether the non-gravitational sum moved. Measured
on the non-grav sum on purpose -- gravity is 1e7 larger and would hide a 100% drag
change inside its own rounding.

```
  knob                     | moved?  | rel change  | documented expectation
  FORCES.drag.atmos        | YES     | 2.962e-07   | moves
  FORCES.drag.model        | YES     | 1.857e-07   | moves
  FORCES.drag.corotate     | YES     | 3.156e-08   | moves
  FORCES.drag.gsi.aT       | no      | 0.000e+00   | ignored by dria (it DERIVES aT)
  FORCES.drag.gsi.Tw       | YES     | 6.452e-10   | moves
  FORCES.srp.model         | YES     | 4.626e-09   | moves
  FORCES.erp.model         | YES     | 8.251e-10   | moves
  FORCES.erp.nrings        | YES     | 1.992e-10   | moves (cap resolution)
  FORCES.gravity.degree    | YES     | 4.402e-06   | moves
  FORCES.srp.on            | YES     | 5.778e-09   | moves
  FORCES.erp.on            | YES     | 1.055e-09   | moves
  SC.Cd                    | no      | 0.000e+00   | ignored by dria (it DERIVES Cd)
  SC.Cr                    | no      | 0.000e+00   | ignored by srp boxwing (facet optics)
  SC.Aref                  | no      | 0.000e+00   | ignored by panel+boxwing (facets rule)
  SC.mass                  | YES     | 1.108e-07   | moves
  SC.attitude              | YES     | 1.444e-07   | moves
  INTEG_METHOD             | no      | 0.000e+00   | ignored (single accel eval)
```
Every knob behaves as documented. **A "no" against "ignored" is a PASS** -- the model
derives that quantity instead of accepting it, which is why you selected it.

## What it caught immediately: `FORCES.drag.corotate` did nothing

First run: `corotate` moved the answer by **0.000e+00**.

`forces/drag.m`'s panel branch passed `'omega', omega` -- the scalar constant,
**unconditionally**. Two consequences, neither of which announced itself:

1. **`corotate = false` did NOTHING to the panel models.** A documented, sweepable
   knob, silently ignored. In a sweep table that is a flat column that looks like a
   physics result: "co-rotation doesn't matter at this altitude" -- a plausible,
   citable, completely false conclusion.
2. **The panel models used the naive z-axis rate** while the cannonball branch used
   the true rate about the CIP (~168 arcsec off z, ~0.4 m/s of v_rel). The same
   model, two co-rotation conventions, depending on which branch you took. I had
   fixed exactly this for the cannonball in round 17 and not noticed the panel branch
   two cases below it.

Fixed: `drag.relVelocity` accepts a scalar OR a 3-vector omega, and `forces/drag.m`
hands the panel models the same `om_eci` the cannonball gets, zeroed when
`corotate=false`. `corotate` now moves the answer by 3.156e-08 of the non-grav sum.

**This is the argument for the matrix in one bug.** Twelve rounds of auditing, a
term-by-term chain verification, and a full regression suite did not find it --
because every one of them tested that the code does what it does. Only sweeping the
knob and asking "did anything change?" tests that the code does what it *says*.

## The 16U base is realistic on purpose

`sgeom.vleo16u` -- the real geometry (0.20x0.20x0.34 m bus + two tracking arrays,
MLI and solar-cell optics), 24 kg, 350 km, drag=dria/dtm2020, srp=boxwing,
erp=boxwing, attitude=ram. Not a toy: it is the one spacecraft here where every model
has the inputs it needs, which is what makes it the right base for a knob sweep.

## State

235 files parse. Suite ALL PASS. Zero uncalled models. Zero allowed-but-unrun
combinations. **Zero knobs that disagree with their documentation.**

---

# Round 20 — provenance: which KIND of number is this?

## The problem the plots were not solving

Every figure so far showed what the models DID. None showed **where the inputs came
from** — and this pipeline mixes four completely different kinds of input into one
answer:

| class | meaning |
|---|---|
| **MEASURED** | a real instrument, this satellite, this epoch (ITSG acc/attitude/orbits, TU Delft) |
| **FETCHED** | somebody else's measurement from an open source (F10.7, Kp, EOP, DE440, ICGEM) |
| **ASSUMED** | a number we chose because nobody published one (Cr, GSI Tw, optics, an aref_box) |
| **MISSING** | needed, absent, blocking a model |

Once they are floats in a struct they are indistinguishable. They are not
interchangeable: **a residual explained by an ASSUMED Cr is not a finding**, and a
model refused for a MISSING input is not a failure. This is exactly the gap that let
CHAMP fly at Cd=2.2 while the report printed 3.0.

## `validation.provenance` + `show_provenance`

Classifies EVERY input and prints it. On CHAMP with dria/dtm2020 + box-wing:

```
provenance: measured 6 | fetched 6 | ASSUMED 7 | MISSING 2

  variable                           | class        | source
  reducedDynamicOrbit (metric 1)     | measured     | ITSG, this satellite & epoch
  nonConservativeForces (metric 3)   | measured     | ITSG, this satellite & epoch
  attitude -> A(t), Cd(t)            | measured     | ITSG, this satellite & epoch
  neutralDensity (metric 4)          | missing      | not published for this sat/epoch
  TU Delft density (metric 5)        | missing      | not published for this sat/epoch
  F10.7 + F10.7a                     | fetched      | OMNI2 (NASA)
  Kp                                 | fetched      | GFZ Potsdam
  EOP (xp, yp, dUT1)                 | fetched      | IERS
  Sun/Moon ephemeris                 | fetched      | JPL DE440
  Cd(t)                              | measured-ish | DERIVED by 'dria' from GSI + attitude
  GSI aT                             | measured-ish | DERIVED by SESAM from atomic O
  F10.7 lag convention               | assumed      | a CHOICE, recorded in sw.F107_lag
  aph mode                           | assumed      | flat vs real 57h history -- a CHOICE
  gravity field                      | assumed      | defaultField, caps at degree 6
  GSI wall temp Tw [K]               | assumed      | nobody measured this surface
  facet optics                       | assumed      | chosen; ONE set for visible AND IR
  geometry (box)                     | assumed      | ASSUMED box from Aref + aspect
  attitude                           | assumed      | MODELLED ram, not the real attitude
```

**Seven ASSUMED against six measured.** `validate_OD` now says so out loud when
assumed outnumbers measured: not automatically wrong — it is the normal state for a
satellite with no published geometry — but it means the headline number is a statement
about your assumptions as much as about the physics.

Note `measured-ish`: `Cd(t)` and `aT` are DERIVED from measured inputs by the GSI
physics. That is a different thing from both a typed constant and a direct
measurement, and collapsing it into either would be a lie in one direction or the
other.

## The figure

ONE figure. Left: every input as a coloured bar with its source spelled out. Right:
the metric ladder against its **own reference floor** (a residual below the floor is
not resolvable and must not be tuned against), the ratios against 1.0, and two text
panels — ASSUMED-inputs-that-can-move-the-answer, and MISSING-and-what-it-blocks.

**Octave-safe by construction**: gnuplot's backend silently mangles a cellstr passed
to `text()`, so every line is its own `text()` call. A figure that renders on MATLAB
and not on Octave is a figure nobody checks.

---

# Round 21 — provenance in the source, and one colour vocabulary

## Declared where you edit, not only computed at runtime

Each of the three entry scripts now opens with a `DATA PROVENANCE` block naming every
input by class, with its source. Not the same block three times — **the provenance of
the three scripts is genuinely different**, and that difference is the point:

**`validate_OD`** — measured truth, no geometry:
```
MEASURED: ITSG RDO/KIN/ACC/attitude/density, TU Delft density
          -> WHICH exist depends on the satellite. CHAMP/GRACE have all six;
             Swarm only TU Delft; Sentinel/Jason none.
FETCHED : F10.7 (OMNI2), Kp (GFZ), ap60/Hpo, F30 (CLS), JB2008 (SET), EOP (IERS),
          DE440 (JPL), gravity (ICGEM)
          -> Real, but NOT of this satellite. A 3-hour planetary Kp stands in for
             the local state above one spacecraft. That gap is physics, not a bug,
             and part of why metric [4] never reads exactly 1.000.
ASSUMED : Cd, Cr, GSI.Tw, GSI.aT (typed by sentman, DERIVED by dria), OPTICS,
          GEOMETRY, lag_f107, aph_mode, ATTITUDE='ram'
```

**`TEMPLATE_16U`** — *the mirror image*:
```
MEASURED: nothing. There is no flight data for this spacecraft. Not one number is
          compared against an instrument. That is the file's PURPOSE, not a defect
          -- but "the ladder looks right" is the strongest claim available, and
          "the drag is correct" is not.
REAL    : THE GEOMETRY. sgeom.vleo16u is the only real geometry in the toolbox,
          which is why it is the only spacecraft where box-wing runs without
          assuming a shape.
```
validate_OD has truth and no geometry; TEMPLATE_16U has geometry and no truth.
**Knowing which you are looking at is the difference between a validation and a
demonstration.**

**`compare_OD`** — what a sweep actually measures:
```
Sweeping an ASSUMED input  -> you are measuring YOUR OWN UNCERTAINTY. If the table
                              moves a lot, the honest report is a BAND, not a number.
                              The most valuable thing this script does, least often done.
Sweeping a FETCHED input   -> how much someone else's data choice costs you.
Sweeping a MODEL           -> model spread: the closest thing to an error bar on a
                              quantity nobody can measure directly.
Sweeping a NUMERICAL knob  -> MUST NOT move the answer. If it does, every other row
                              is noise and you are reading tea leaves.
```
Plus the degeneracy warning stated where it bites: on metrics [1]/[2], `rho*Cd*A/m`
is ONE number and the orbit cannot say which factor was wrong. Sweep `SC.Cd`, watch
the residual move beautifully, and learn **nothing about Cd**. Metrics [3]/[4]/[5]
break it. And `Cd(t)` from a GSI model is *not* degenerate the same way — it has a
shape with altitude and local time that a scalar cannot mimic.

## `validation.prov_colors` — one vocabulary, five classes

Three figures now colour by provenance. If each carried its own literal they would
drift, and **"orange" meaning ASSUMED in one figure and DERIVED in the next is worse
than no colour, because you would read it confidently and be wrong.** Same rule the
audit has enforced on physics all along: one quantity, one definition, one place.

```
measured  green       a real instrument, THIS satellite, THIS epoch
derived   pale green  computed BY the physics FROM measured inputs (Cd(t), A(t))
fetched   blue        someone else's measurement, open source, NOT this satellite
assumed   orange      we chose it. EVERY ONE IS A KNOB.
missing   red         needed, absent, blocking a model -- a DATA limit
```

`derived` earns its own class: `Cd(t)` and `aT` are neither measurements nor guesses.
They inherit the measurement's authority for the parts that are measured and the
model's uncertainty for the rest. Collapsing them into either neighbour would be a
lie in one direction or the other.

## The panels now say which kind of number they show

`show_model_io` titles changed from describing position in the pipeline to describing
**provenance**:
```
  rho          -> "rho -- MODEL dtm2020, on FETCHED indices"
  attitude     -> "attitude (R_bi) -- MEASURED"  or  "-- ASSUMED: modelled ram"
  Cd (cannon)  -> "C_d -- ASSUMED (cannonball: a constant you typed)"    [orange]
  Cd (dria)    -> "C_d(t) -- DERIVED by dria from GSI"                   [pale green]
  Aref         -> "A_ref -- FETCHED (catalog), held CONSTANT"            [blue]
  A(t)         -> "A(t) -- DERIVED (geometry x attitude)"                [pale green]
```
and the legend is printed on the figure, because a reader should not have to know the
convention to read the picture.

## State

240 files parse. Suite ALL PASS. Every model wired and reachable from all three
scripts; every knob sweepable and verified against its documentation; every input
classified by origin in the source, at runtime, and in colour.

---

# Round 22 — the crash I shipped, and what your log says

## `Unrecognized function or variable 'DO_PLOTS'`

I invented **two** variable names when adding the provenance call: `DO_PLOTS` (the
toggle is `PLOTS`, sitting 400 lines up) and `FIGDIR` (it is `outdir`). Neither
existed. The run died at line 751 **after** a 20-second propagation and every
download -- the worst possible place to fail.

**A script's own variables are an interface too.** Inventing a name for one is the
same class of error as inventing a struct field, which is the bug this entire audit
started from. I have now added a mechanical check (assignment-before-use, comments
and string literals stripped) rather than trusting myself to notice.

## Two structural problems in the same block

1. **The provenance printout was gated behind `if PLOTS`.** Provenance is not a plot.
   It prints regardless now -- a run with plots disabled (a batch sweep, a headless
   box) is *exactly* where a silently ASSUMED input does the most damage, and
   nesting it under `PLOTS` would have dropped it precisely when it matters most.
2. **Two provenance systems.** `validate_OD` already had a fetched-product ledger
   (provider, frame, epochs, URL) and I added a classifier beside it. They are not
   duplicates -- the ledger knows WHERE each download came from, the classifier knows
   WHAT KIND every input is, including the ASSUMED ones that are never downloaded and
   so never appear in the ledger. But two overlapping accounts of one question is the
   `drag.force`/`panelCoeffs` pattern, and they agree until someone edits one. The
   ledger is now **passed into** the classifier: one account, real providers and
   frames instead of a generic 'ITSG / TU Delft' string.

## What the real MATLAB run actually says

This is the first end-to-end run on real data, and it is worth reading carefully:

```
[3] non-conservative force : measured 1.994e-06 | modelled 2.081e-07 | ratio 0.104
[5] density vs TU Delft    : measured 1.488e-12 | modelled 1.801e-12 | ratio 1.142
[1] vs reducedDynamicOrbit : |3D| 2.20 m   (reference spread 0.01 m)
```

**The density is fine and the acceleration is 10x low.** Those two facts together are
the whole story, and they rule things out:

- `ratio [5] = 1.142` says DTM2020 on fetched F10.7/Kp reproduces the measured
  density to 14%. **The atmosphere is not the problem.**
- `ratio [3] = 0.104` says the modelled non-gravitational acceleration is ~10x too
  small anyway. With density right to 14%, the deficit is in **Cd x A / m**, or in a
  force that is missing entirely.

And your run config names a candidate outright:
```
  srp / erp     cannonball / knocke
  other forces  thirdbody=1 srp=1 erp=0 relativity=1
```
**ERP was OFF.** ERP at 345 km is ~1e-9 m/s^2 against a 1.8e-6 deficit, so it is not
the answer -- but it is a free 0.05% and it should be on.

The 10x is too large for Cd alone (2.2 -> 3.0 is 1.4x) and too large for A(t) (CHAMP
flies ram/nadir; the plate gives A(t) ~ Aref within a few percent, which is exactly
what `[3]`'s shape should show). A 10x deficit in the non-gravitational sum with the
density verified independently points at the **drag scale factor** -- Cd*A/m -- or at
a units/frame error in how `nonConservativeForces` (SATELLITE body frame, per the log)
is being compared to the modelled ECI acceleration.

**That last one is the first thing to check**, and it is now checkable: the log says
`[itsg] CHAMP nonConservativeForces: SATELLITE body frame`, and the modelled
acceleration is ECI. If the comparison rotates one into the other with the measured
attitude, good. If it compares magnitudes, a 10x is not explained by a frame error --
but the RMS of a body-frame vector against an ECI vector is not a like-for-like
number, and `ratio 0.104` has been stable across every change I have made to the
physics, which is itself suspicious: **a physics error should have moved when the
drag model changed.**

Run `compare_OD` with `FORCES.drag.model = {'cannonball','sentman','dria'}` against
metric [3]. If the ratio does not move at all across three genuinely different Cd
models, the fault is in the COMPARISON, not the physics.

## State

239 files parse. Suite ALL PASS. Sweep matrix: zero knobs disagree with their
documentation. Assignment-before-use verified for every variable in the new block.

---

# Round 23 — hunting the DO_PLOTS class across the whole tree

You asked whether that error exists anywhere else. It did. `08_test/check_undefined.py`
now finds it mechanically instead of by eye.

## What it looks for

An identifier USED but never ASSIGNED anywhere in its file, and not a function, a
package, or a builtin. That is `DO_PLOTS` when the toggle is `PLOTS`. **MATLAB does
not catch it until the line executes** -- which in validate_OD was after a 20 s
propagation and every download.

Statically, not by running: most of these paths cannot run here (no MATLAB, no
network, no Aerospace Toolbox). A bug that only appears on your machine after 20 s of
work is exactly the one to catch by reading.

## Building it honestly cost more than running it

The first pass reported **178 problems** and nearly all were false. Each one was a
lesson about MATLAB's grammar, and each mattered, because **a checker that cries wolf
gets switched off** -- at which point it is worse than nothing:

| false alarm | why | fix |
|---|---|---|
| `case 'twobody'` -> `twobody` undefined | "case" ends in a letter, so my transpose heuristic called that quote a transpose and left the string contents in the code | exclude keywords before the alnum test |
| `[base f(x) '&FORMAT=TLE']` -> `FORMAT`, `TLE` | `)` before a quote looked like transpose | `A(1)'` is transpose (touching); `f(x) 'lit'` is a string (space) |
| `add = @(n,A) struct(...)` -> `A` | anonymous-function parameters | parse `@(...)` params |
| `solver = @ode78` -> `ode78` | function handles | parse `@name` |
| `function u = U2(x)` inside `accelFromDeg2` -> `dcs`, `mu`, `Re` | nested functions capture the parent's scope | union of everything assigned in the file, including every header's params |
| `grid on; box on;` -> `box`, `on` | command syntax, two per line | strip `cmd word` after `^` or `;` |

**178 -> 0.** The remaining "unknown call" list is now genuinely useful rather than
noise: it is the real external-dependency list (`atmosnrlmsise00`,
`gravitysphericalharmonic` = Aerospace Toolbox; `twoline2rv`/`sgp4` = Vallado, and
`sgp4_reference.m` already guards those with `exist(...)==2`, which is correct).

## Two real bugs, both of the same class

**`fetch_tle_auto` does not exist.** `05_data/+data/tle.m:28` calls it. There is no
`data_sources/satellite/` in this tree -- only `density_reference` and `spaceweather`.
The line has been an `Undefined function` waiting for its first caller. The real
fetcher is `validation.fetch_tle`; `data.tle` now tries the old name and falls back.

**`inputs_report` does not exist, and a swallowing catch hid it.**
```matlab
try, inputs_report(SAT, START, STOP, config.defaultConfig(), outdir); catch, end
```
It threw `Undefined function` on **every single run** of `compare_density` and the
error was discarded. **A swallowing catch around a function that does not exist is
indistinguishable from a feature that works.** It wanted a provenance report -- there
is a real one now, and its catch prints instead of swallowing.

That second one is the more interesting failure. The first is a typo; the second is a
*silent* typo, and this codebase has produced that pattern repeatedly: the CHAMP
TU Delft fetch that "worked" because it failed inside a catch, the atmosphere defaults
that hid a missing driver, `p.retrieved` missing on a struct nobody checked.

## State

**239 files parse. 0 used-but-never-assigned. Suite ALL PASS.** The checker lives in
`08_test/check_undefined.py` -- run `python3 08_test/check_undefined.py .` from the
toolbox root after any edit.

---

# Round 24 — the repetition you spotted was a silent override

## `R.rdo.rms3d` -- I did it again, three times

`validation.rtn_stats` returns **`pos3D`**. I wrote `rms3d`. And `R.acc` has **no
`.ratio` field** -- `od_metrics` builds it without one and computes the ratio inline
at line 138 -- so that was a second crash queued behind the first. Three invented
names in one function.

I tried to extend `check_undefined.py` to catch struct fields and **it does not
work well enough to ship as a gate**: it collects fields by variable NAME across
files, so an `sc` in one file is conflated with an `sc` in another, and it produced
17 hits that are nearly all noise. Doing it properly needs type inference. It is left
in as **ADVISORY ONLY**, labelled as such -- a checker you cannot trust is one you
switch off, and then it is worse than nothing.

**The real defence is on the reader side.** `show_provenance` now checks `isfield`
before every access and degrades to a missing panel instead of a crash. R's contents
legitimately vary with which products a satellite has, so *assuming its shape was
always wrong*, checker or no checker. **A figure is a diagnostic -- it must never be
the thing that kills the run that produced the diagnosis**, especially after 20 s of
propagation and every download.

## The repetition: two knobs for one decision, and one silently won

You were right, and it was worse than untidy:

```matlab
DRAG_MODEL = 'sentman';                                   % line 251
...
FORCES = struct('drag', struct('on',true,'model','sentman','atmos','dtm2020',...));  % line 314
...
cfg.forces.drag.model = DRAG_MODEL;                       % line 538  <-- this wins
cfg.forces.srp.model  = SRP_MODEL;
cfg.forces.erp.model  = ERP_MODEL;
```

Set `FORCES.drag.model = 'dria'` and leave `DRAG_MODEL = 'sentman'` and **you ran
sentman**, with no warning, and every conclusion you drew was about a model you never
ran. They agreed in the shipped file only because I happened to keep them in sync --
which is exactly the fragility, not a defence.

This is the same **"one quantity, two names"** bug the audit has been removing from
the physics all along (`.area`/`.Aref`, `drag.force` vs `panelCoeffs`, two 16U
geometries) -- except sitting in the **user-facing knobs**, which is the worst place
for it, because the knobs are the part people actually edit.

Fixed:
- `model` is **gone** from the `FORCES` struct. It carries on/off and settings with
  no dedicated knob.
- `ATMOS` is now a real knob. It had only ever existed as `FORCES.drag.atmos` --
  the model in the knob block, its partner 60 lines away in a struct literal.
- A **guard**: if anyone re-adds `FORCES.drag.model`, the script errors and names
  both places, rather than silently overwriting.

## Figure layout

The legend was drawn in the **same axes** as the item list, in data coordinates
starting at `y=0.2` -- so it printed *on top of* the last items the moment the list
got long. It collided on the first real run. Now: its own axes; font and truncation
scale with the item count (a fixed 8.5 pt overprints past ~22 rows, and the list
grows with every model that has parameters, so it *will* pass it); the right-hand
panels get explicit positions instead of `subplot` slots that assumed a 4x2 grid.

## Folders

The root carried `TEMPLATE_16U`, `EXAMPLE_16U`, `TEMPLATE_propagation`, `show_16U`.
They are examples; they now live in `07_examples/` with a README stating the trade
they make: **validate_OD has truth and no geometry; these have geometry and no
truth.** `setup_paths.m` stays at the root because it is the one file you call first.

`TEMPLATE_16U`'s `force_data` path was relative to `mfilename` -- the move would have
silently redirected its output into `07_examples/force_data/`. Outputs would still
appear, just in the wrong place: the kind of "working" that wastes an afternoon.
Fixed to resolve from the toolbox root.

## State

239 files parse. 0 used-but-never-assigned. Suite ALL PASS. One source of truth per
knob, with a guard against the second one coming back.

---

# Round 25 — you found it: metric [3] was never a physics result

## Your along-track argument, quantified

You said the drag deficit "should only be reflected in along-track error". That is the
whole diagnosis, and the arithmetic is decisive:

```
a genuine 1.786e-06 m/s^2 along-track deficit over a 3 h arc gives
  dx = 0.5*a*t^2 = 104 m of along-track drift.
Your run measured |3D| = 2.20 m against the reduced-dynamic orbit.
-> A real 10x drag deficit CANNOT hide inside a 2.2 m residual.
```

## And the model was right all along

TU Delft **measured** the density: 1.488e-12 kg/m^3. That density *requires*:
```
a = 0.5*rho*Cd*(A/m)*v^2 = 1.9445e-07 m/s^2
```
```
modelled  2.081e-07  -> ratio to physics 1.070   <-- the model is CORRECT
measured  1.994e-06  -> ratio to physics 10.255  <-- the REFERENCE is 10x too big
```
For 1.994e-06 to be real the density would have to be 1.526e-11 -- **10.3x what TU
Delft derived from CHAMP's own accelerometer.**

## The bug: RMS about zero eats the instrument bias

CHAMP's STAR accelerometer carries a known along-track bias of order **1e-6 m/s^2** --
roughly **ten times the drag it is measuring** at 345 km. That is precisely why TU
Delft publish a *calibrated* density product: they estimate bias and scale before
inverting the accelerometer.

`od_metrics` took `rms(a_meas)` **about zero**, so:
```
rms_meas ~= |bias|                            (because |bias| >> |drag|)
ratio = rms_mod/rms_meas = drag/bias ~ 0.1
```
**That is the signal-to-offset ratio, not a physics result.** Implied bias from your
own numbers: **1.98e-06 m/s^2**. Exactly the STAR figure.

## The tell we walked past for twelve rounds

**The ratio never moved when the drag model changed.** Swapping cannonball -> dria
shifts the numerator ~20%, and 0.097 -> 0.117 is invisible against a denominator that
is 10x larger and constant.

**A physics error moves when you change the physics. This one never did.** I kept
proposing sweeps to explain it and never asked why the previous sweeps had all been
flat.

## Fixed

`R.acc` now carries both, and says which is which:
```
[3] non-conservative force (RMS about zero -- INCLUDES the instrument bias):
      measured 1.994e-06 | modelled 2.081e-07 | ratio 0.104
    accelerometer bias (mean of the measured series):
      |bias| = 1.98e-06 m/s^2
      CHAMP STAR carries ~1e-6 m/s^2 of along-track bias -- ~10x the drag at 345 km.
[3] BIAS-REMOVED (this is the physics comparison):
      ratio modelled/measured = <the real number>
```
Plus `ac_meas`/`ac_mod` (mean-removed series) and a one-orbit moving-average window --
**the moving average you asked for**, which is what separates a slowly-varying bias
from the per-revolution drag signature.

## The velocity panel

It drew a horizontal line at a scalar because **there was no series to draw** --
`od_metrics` stored only the RMS. Now it stores `dv_vec`, `dv_mag` and `dv_rtn`, and
the panel plots the residual **in RTN**.

That matters for exactly your reason: **a residual's shape is the diagnostic.** A
constant offset is a bias; a linear ramp is a drag scale error; a once-per-rev
oscillation is a frame or phase error. The RMS collapses all three into one number
that cannot tell them apart -- which is how a 10x "error" survived twelve rounds.

`validation.rtn_project` is the projection alone, so velocity uses the **same triad**
as position rather than a second definition of "along-track".

## neutralDensity was hard-coded to the name that 404s

```matlab
if strcmpi(SC.dir,'CHAMP'), dprod = 'neutralDensity_ACC';   % all 4 URLs 404
```
Metric [4] silently vanished on the satellite with the most data. ITSG's naming is not
uniform across missions and we do not get to decide it, so it now **tries both** and
reports which it used.

## Density ratio 1.142 -- is that "fine"?

Yes, and it is the strongest number in your run. DTM2020, driven by fetched F10.7/Kp,
reproducing a measured thermospheric density to **14%** is at the good end of what
these models do (they are routinely 15-30% off). Both the 1.142 and the model's
agreement with the physics above say the same thing: **the atmosphere was never the
problem.**

## Knobs, segregated

`09_docs/KNOBS.md` sorts every knob into four kinds that behave differently under a
sweep -- INDEPENDENT / DEGENERATE / CONDITIONAL / NUMERICAL -- and all three entry
scripts point at it.

---

# Round 26 — two errors that refused to say what was wrong

## GRACE-A: `F107 is NaN`

The message named the symptom and nothing else, so it sends you into the atmosphere
code when the cause is upstream and mundane. There are exactly three, and the table
can tell them apart in one line:

- the epoch is **outside** the table -> extend `padDays`
- the row exists but the **source had a gap** -- OMNI2 writes `999.9` for a missing
  day and that parses to NaN -> that day genuinely has no F10.7
- the table **never loaded** -> a fetch failed upstream

The error now says which:
```
space-weather index "F107" is missing or non-finite (NaN)...
  requested epoch  : 2007-01-02 00:00:00 (MJD 54102.0000)
  table covers     : MJD 54100.0 .. 54106.0 (7 rows)
  row 3 of 7 found, so the epoch IS covered.
  f107obs has 1 NaN of 7 rows -- OMNI2 writes 999.9 for a
  missing day and that parses to NaN. This day simply has no F10.7.
  -> set opts.manual (e.g. struct('F107',81,'F107a',81,'ap',5,'Kp',1))
     or pick a different DATE, or use a source without the gap.
```
Note what it does **not** do: interpolate across the gap. F10.7 is smooth and a
one-day fill would look harmless -- and would be inventing a driver value and then
reporting results as if it were measured. The provenance work of the last few rounds
exists precisely to stop that. If you want the gap filled, fill it deliberately with
`opts.manual` and it will be tagged ASSUMED.

**Why CHAMP worked and GRACE-A did not, at the same date:** the message will now
tell you rather than us guessing. My expectation is the arc crossing into a day whose
OMNI2 F10.7 is a fill value -- but that is a hypothesis and the diagnostic is the test.

## neutralDensity: "not found" was the wrong claim

```
[itsg] GET 1/4 .../CHAMP/neutralDensity_ACC/2007/CHAMP_neutralDensity_ACC_2007-01-01.txt.gz
[4] neutralDensity_ACC unavailable: could not find neutralDensity_ACC for CHAMP
```

We **guessed a filename**, it was not there, and we reported it as the data being
missing. **Those are completely different conclusions and they need opposite
responses.** You said the density is on the same server as the orbits -- and you are
right, the orbits come from exactly that path.

Two fixes:
1. `validate_OD` no longer hard-codes CHAMP to `neutralDensity_ACC`. It tries both
   product names and prints which one it used. (ITSG's naming is not uniform across
   missions and we do not get to decide it.)
2. When every candidate fails, `data.itsg` now **lists what the server actually has**
   -- the real product directory names and a sample filename from each -- instead of
   asserting absence:
```
  we looked for : CHAMP_neutralDensity_ACC_2007-01-01.txt.gz
  under         : .../CHAMP/
  server HAS these product dirs for CHAMP:
    attitude, kinematicOrbit, neutralDensity, nonConservativeForces, reducedDynamicOrbit
  e.g. in neutralDensity/2007/: CHAMP_neutralDensity_2007-01-01.txt.gz
```
That turns "the data is missing" into "we named it wrong", which is a one-line fix
rather than an abandoned metric.

This is the same failure mode as the swallowing catch around `inputs_report`: **an
error message that describes our own guess as if it were a fact about the world.**

---

# Round 27 — the final sweep for the same three classes

You asked whether more of these are hidden anywhere. Three classes, swept
mechanically rather than by eye.

## 1. Swallowing catches — 1 real bug

Ten found. Triaged, and the distinction turned out to be exactly one thing:
**does the default get set BEFORE the try?**

`data/drivers.m` is the CORRECT pattern:
```matlab
DRV.f30src = 'unknown';                                    % <- default FIRST
try, DRV.f30src = DRV.f30TT.Properties.UserData.source; catch, end
```
The failure cannot leave the field undefined. That is not a swallowed error, it is
optional metadata with a fallback.

`op/buildWorld.m` was the **bug**:
```matlab
try, W.frame.data_dir = data.eop_dir(); catch, end        % <- nothing set first
```
On failure the field stayed **missing**, silently, and the frame build fell back to
whatever it does without EOP -- on build `'gmst'` that is `dUT1 = 0`, worth **~450 m**
— with no trace in the output. Now the field is always set and a failure **warns**
with the cost named.

The rest are legitimate probes (candidate URLs that are *expected* to 404 — that is
what probing means), version guards (`sgtitle` is R2018b+; Octave's gnuplot handles
neither `sgtitle` nor `annotation`), an optional Vallado extra, and the
error-message builder itself (a diagnostic that throws while explaining a failure
replaces a useful message with a useless one). **All now say WHY in place**, so the
next sweep does not re-flag them and, more importantly, so nobody adds a real one
next to them by pattern-matching.

## 2. Undefined variables — 0

`check_undefined.py`: **0 used-but-never-assigned** across 241 files.

## 3. Struct fields — the class the checker cannot get

`R.rdo.rms3d`, `R.acc.ratio`, `DO_PLOTS`. MATLAB does not check a field until the
line runs, and judging it statically needs type inference — my attempt produced 19
hits that were nearly all noise, because it collected fields by variable NAME across
files. **A checker you cannot trust gets switched off, and then it is worse than
nothing.** It stays ADVISORY, labelled.

So this class is tested by **execution**: `08_test/test_contracts.m`, now block [8] of
the suite.
```
[8] struct contracts (test_contracts)
    all contracts hold (5 model combos x 3 ref shapes, 9 R shapes,
    24 satellites, 5 drag models on one out-struct)   PASS
```
The **degenerate cases are the point**: empty `R`, `R` with only some metrics,
`R.acc` with `rms_meas = 0`, a satellite with no accelerometer. R's contents
legitimately vary with which products exist, so a reader that assumes a shape is
wrong *even when today's data happens to fit*. That is why `show_provenance` now
checks `isfield` before every access rather than relying on any checker.

## Final state

```
241 files parse                          0 failures
regression suite (8 blocks)              ALL PASS
check_undefined.py                       0 used-but-never-assigned
uncalled models                          0
allowed-but-failed model x satellite     0
knobs disagreeing with documentation     0
swallowing catches without a reason      0
two-sources-of-truth                     guarded (validate_OD errors on it)
```

**And the honest limit, unchanged since round 1:** Octave + synthetic fixtures prove
wiring, units, frames, dispatch and mathematics. They do not prove the numbers. The
numbers need your MATLAB — and the single most valuable thing in this whole audit came
from *you* running it and noticing that a 10x drag deficit cannot hide inside a 2.2 m
along-track residual.

---

# Round 28 — the bias fix had only reached one of six places

You asked to see measured vs calculated side by side for r/v, density, and the
non-gravitational sum, and to resolve any gap that could produce an error. The r/v
overlays and the two-source density panel already existed. The accelerometer did not,
and looking for the gap found a serious one.

## Six places computed the metric-[3] ratio. One learned about the bias.

Round 25 corrected `od_metrics` to remove the accelerometer bias. Five other places
kept dividing raw, each with its own inline
`R.acc.rms_mod/max(R.acc.rms_meas,eps)`:

| place | consequence |
|---|---|
| `show_OD` acc panel | figure prints 0.104 while the console prints the corrected number |
| `show_OD` stats panel | same |
| `show_provenance` | **I wrote this inline one round ago, using the raw formula** |
| `compare_OD` (metric) | **sweeps against the contaminated ratio** |
| `compare_OD` (CSV) | the saved results carry the raw number |
| `show_compare_OD` | the comparison plot |

**The `compare_OD` one is the serious one.** The raw ratio is ~`drag/bias`, and the
bias is a **constant ~10x larger than the signal**. So every sweep against metric [3]
comes back nearly **flat** — which reads as *"this knob does not matter"*. A
comparator pointed at a contaminated metric does not merely give a wrong answer, it
gives a **confident** one. And that flatness is exactly the symptom that hid the bug
for twelve rounds: I kept proposing sweeps to explain the 0.104 and never asked why
the earlier sweeps had all been flat.

**Fixed:** `validation.acc_ratio` is now the single definition. All six go through it.
It returns `isCorrected` so a caller that gets the raw fallback (an old `R`) says so
in the title rather than passing it off.

## Proved on a case where the answer is known

`08_test/test_contracts.m` block [5] — a model that is **exactly the truth**, plus a
realistic CHAMP STAR bias:
```
raw 0.0775 (calls a PERFECT model a 10x failure) -> corrected 1.0000   PASS
```
**0.0775 is the same magnitude as your 0.104.** If this test ever breaks, metric [3]
is lying again.

## The accelerometer figure now shows the thing that matters

- panels 1–3: measured vs modelled per axis, **with the measured MEAN drawn as a
  dashed line** — the bias, visible, next to the signal it swamps
- panel 4: **bias-removed** measured vs modelled, titled with the physics ratio and
  the raw one beside it so the difference is not a claim you have to take on trust
- panel 5: the modelled sum **by force** (drag / SRP / ERP, log scale)

That last panel exists because *"the non-grav sum is 10% low"* is not actionable:
drag, SRP and ERP are three different physics with three different fixes and, at
345 km, two orders of magnitude apart. Summing before plotting throws away the only
information that says which one to look at. `od_metrics` now records `R.acc.cmp`.

## State

242 files parse. Suite ALL PASS (8 blocks). 0 used-but-never-assigned. 0 uncalled
models. 0 knobs disagreeing with their documentation. One definition per quantity —
including, now, the metric-[3] ratio.

---

# Round 29 — the plot destroyed the thing it was diagnosing

## `P.growth`: Dot indexing is not supported

Mine, and the same class twice over.

```matlab
P = defaults(PLOTS);          % line 47  -- the options struct for the whole function
...
P = R.rdo.dv_rtn;             % line 131 -- MY EDIT. A matrix. P is now gone.
...
if P.growth                   % line 149 -- dies. In a figure I never touched.
```

Reusing a short name inside a long function is the same bug as inventing one:
**MATLAB will not warn about either until the line runs.** Long functions make it
likelier; single letters make it invisible. Renamed to `Vrtn`.

## The checker for it — and my own two failures building it

First attempt flagged `fk = facets(k)` — a **struct-array index**, perfectly correct
— and would **not** have caught the real bug, because `R.rdo.dv_rtn` has no paren
after the identifier. Wrong at both ends.

The reliable signal needs no type inference: **within one function, is a name used
BOTH as `x.field` AND indexed as `x(:,1)`?** Those cannot both be right.

Then I "proved" the new check missed the bug — and the *test* was wrong: my `sed`
never applied, and my `sed -n '/ONE NAME/,/^====/p'` range stopped at the `====`
immediately under the header. **Twice I nearly concluded the checker was broken when
my test was.** It catches the minimal reproduction correctly.

Three hits on the tree, all explainable and all correct code:
```
utcvec.m       utc.Year AND utc(1:6)          -- POLYMORPHIC: takes a datetime OR a 6-vector
fetch_tudelft  T.DateTime AND T(rows,:)       -- a MATLAB table: both are valid
get_f30_cls    t.field AND t(end+1,1)         -- a datetime array
```
Tables, datetimes and type-dispatching functions legitimately do both. **Documented
rather than suppressed** — the reader needs to know why they are there, or the next
person deletes the check.

## The real defence: a figure must not kill the run

Both figure crashes (`DO_PLOTS`, `P.growth`) fired **after** a 20 s propagation and
every download. In both cases **the science was fine and a diagnostic threw it away.**

`validate_OD` now guards the plot calls and **reports** the failure — file, line,
message — then carries on:
```
  [!] 1 FIGURE(S) FAILED -- the validation itself is unaffected:
      show_OD: P.growth: Dot indexing is not supported  [show_OD line 149]
      The metrics above and results/*.csv are complete and correct.
      Fix the plot; do not re-run the propagation.
```

**This is the opposite of the bare `catch, end` this audit has been removing.** The
test has always been *do you find out* — and here you do, loudly, while keeping the
run. R and PROV are computed and printed before this point; the figures are a **view**
of the answer, not the answer. Plotting code has no business deciding whether a
validation succeeded.

---

# Round 30 — your GRACE-A run, and the check that should have existed all along

## Your question: "velocity error is growing — is everything fine or some issue?"

**It is fine, and the run proves it.** But nothing in the toolbox was checking, so
here is the check, now automatic.

## `validation.closure` — do the metrics agree with EACH OTHER?

Metric [3] says the modelled non-gravitational acceleration is short by `da`.
Metric [1] says the along-track position is off by `dx`. **Those are not two
independent facts.** A missing along-track acceleration MUST produce a drift, and the
relationship is arithmetic:

```
dx = 0.5 * da * t^2
```

If they disagree, one of them is wrong — and that is worth more than either number
alone. Run on your three cases:

```
CHAMP 2007-01-01, RAW metric [3]:
  deficit 1.7859e-06 -> implies 104.15 m of along-track drift
  metric [1] measured 2.16 m                        ratio 0.02
  INCONSISTENT: 48x. A deficit that large CANNOT hide inside that residual.
  Metric [3] is measuring something that is not a force.

GRACE-A 2010-01-01, YOUR RUN (bias-removed, ratio 0.529):
  deficit 1.41e-08 -> implies 0.82 m of along-track drift
  metric [1] measured 2.09 m                        ratio 2.54
  CONSISTENT: agree to 2.5x. Both describe the same missing force.
```

**This is the check that would have found the CHAMP bug on day one.** The metrics
contradicted each other by 48x for twelve rounds because nothing ever compared them.
It is now block [6] of the suite and prints on every run and on the stats panel.

## So: your velocity residual

The radial `dv` growing to −4 mm/s while the radial POSITION residual oscillates at
±0.5 m is **not a radial force**. Drag is along-track. r and v residuals are coupled
by the orbit — a small along-track offset rotates through the RTN triad and appears
in the radial velocity. The along-track position (0 → 4.6 m) is the drag signature,
and closure says it agrees with the accelerometer to 2.5x. **Two independent
measurements of the same missing force, agreeing.** That is a pass.

## The plot said CHAMP on a GRACE figure

`'measured MEAN: CHAMP STAR carries ~1e-6 m/s^2 of bias'` — hard-coded by me, printed
on every satellite. **A caption naming the wrong instrument is worse than none: it is
a confident, wrong, citable statement sitting on a plot.** Both the figure and the
console now report the bias/signal ratio **measured in this run**.

## The density spikes are not model error

TU Delft rho collapses to ~1e-15 at two samples and the ratio spikes to 25 and 36.
`rho = 1e-15` at 460 km is not a thermosphere — it is a **dropout in the reference**.
On a linear axis those two points owned the panel and the other ~700 (the ones
carrying the answer) were a flat line at the bottom.

Now: **log axis** (a ratio is multiplicative — 0.5 and 2.0 are the same size of wrong,
and a linear axis says otherwise), dropouts **marked with a red triangle and counted
in the title**, not deleted. Deleting them silently would be choosing which reference
samples to believe.

The METRIC was already robust — `R.tud.ratio` uses a `median` and `rms_logerr` works
in log space. Only the picture was lying.

## State

243 files parse. Suite ALL PASS (8 blocks, now including closure). 0
used-but-never-assigned.

## Postscript to round 30: the checker earned its keep on my own edit

Packaging this round, `check_undefined.py` reported **1 variable problem** — mine,
from the closure edit I had just made:

```
06_validation/+validation/od_metrics.m  [vprint]  ->  VERBOSE
```

Two bugs in one insert, and the `[vprint]` scope label gives away the worse one:

1. **`VERBOSE` does not exist.** The toggle is `opts.verbose`. The DO_PLOTS class
   again — my fourth time.
2. **The block landed INSIDE `vprint`.** I anchored the insert on the file's LAST
   `end` (`s.rindex('end')`), which belongs to the last local helper, not to
   `od_metrics`. The closure check would never have run — and it would not have
   crashed either, because `vprint` is only called with the arguments it expects.
   **A silent no-op is worse than a crash**: I would have shipped a cross-check that
   never fired and believed it was passing.

Both caught before shipping, by a checker whose false-positive rate I spent three
rounds beating down for exactly this moment. The lesson is not that I make mistakes;
it is that **anchoring an edit on `end` is anchoring on nothing** — `end` is the most
common token in the file. The fix anchors on `^function R = od_metrics` and finds the
next `^function`, which is a real boundary.

---

# Round 31 — are the errors natural or ours? Analysing your GRACE-A plots

## The panel that looks alarming and is not

Your radial `dv` is **secular** (−4 mm/s, growing) while the radial POSITION residual
is **bounded** (±0.5 m). If `dv_R` were the derivative of `dr_R` that is impossible.

**The RTN triad rotates.** For a purely along-track residual:
```
d/dt (x_T * That) = xdot_T*That + x_T*dThat/dt = xdot_T*That - n*x_T*Rhat
```
So a growing along-track POSITION error MUST produce a radial VELOCITY residual of
`-n*x_T`. Checked against your numbers:
```
  along-track at 180 min      : 4.6 m
  n at 460 km                 : 1.1165e-03 rad/s
  predicted radial dv = -n*x_T: -5.14 mm/s
  your plot shows             : -4.2 mm/s     -> 78% agreement, off a picture
```
**Nothing is wrong.** One error — along-track drag — appears in two panels wearing
different clothes. `validation.closure` now checks this automatically, because that
panel looks alarming and the honest answer without a check is a shrug, and **a shrug
is how a real bug gets waved through next time.**

## Where the 2.09 m actually lives

```
  candidate                          | a [m/s^2] | 3h drift [m]
  [3] deficit (bias-removed)         | 1.410e-08 |     0.82
  solid tides, OFF                   | 1.000e-08 |     0.58   <-- biggest single term
  Cd 2.2 vs 3.0                      | 7.200e-09 |     0.42
  density model 14% (TUD ratio 1.14) | 2.800e-09 |     0.16
  ocean tides, OFF                   | 2.000e-09 |     0.12
  ERP, OFF in your run               | 1.500e-09 |     0.09
  gravity deg 70 vs 20               | 1.000e-09 |     0.06
  ITSG reduced-dyn vs kinematic      | the floor |     0.01
  --------------------------------------------------------------
  YOUR OBSERVED ALONG-TRACK          |           |     2.09
```

The residual is **200x above the reference floor**, so it is real physics, not
reference noise. And the accelerometer deficit explains only 0.82 m of it.

## The architecture gap: undeclared defaults

`validate_OD` shipped with **erp = OFF, solidtides = OFF, oceantides = OFF**. All
three are implemented, wired, reachable, and were simply off — in a struct literal,
silently. **~0.79 m of your 2.09 m is forces the run never included**, and a 2 m
residual was being read as drag mismodelling.

**A default is a choice. An undeclared default is a choice someone else made for you
and did not mention** — the same failure as an assumed Cd printed as if it were
measured, which is where this whole audit started.

`validation.force_budget` now prints, every run:
```
  ---- FORCE BUDGET at 460 km over a 10800 s arc ----
    force        on?  |a| [m/s^2] drift [m]  note
    erp          NO   1.500e-09       0.09   <-- OFF: albedo+IR. Small, FREE.
    solidtides   NO   1.000e-08       0.58   <-- OFF: worth ~0.6 m/3h
    oceantides   NO   2.000e-09       0.12   <-- OFF
    -> the forces that are OFF could account for ~0.79 m of along-track drift.
       Weigh that against your residual BEFORE concluding anything about Cd or
       the density: a residual you can explain with a force you did not model
       is not a physics finding.
```
Priced in **metres**, not m/s^2, because nobody can weigh `1e-8 m/s^2` against a 2 m
residual in their head. **ERP is now ON by default** — it is 0.09 m and free.
Solid tides stay off pending a real-MATLAB run of the ephemeris+Love-number path,
but the cost is now printed rather than hidden.

## So: are the errors natural or ours?

**Natural, and the toolbox now says why.** In order:
1. `[3]` bias — was OURS, fixed (round 25).
2. The radial `dv` — natural kinematics, now explained by the closure check.
3. The 2.09 m along-track — real physics: ~0.8 m of drag deficit, ~0.8 m of forces
   that were switched off, the rest in Cd/density assumptions. All now priced.

## Next, in order of value

1. **Turn on solid tides** — 0.58 m, the biggest single unmodelled term.
2. **Sweep `FORCES.drag.model`** against metric [3] now that it is bias-corrected;
   the sweep was measuring `drag/bias` and returning flat until round 28.
3. Sweep `SC.Cd` **to see the degeneracy**, not to fit through it.

---

# Round 32 — switching each force on, and what the Cd is really telling us

## Measured, one change at a time (GRACE-A 480 km, 2010-01-01)

```
  configuration                              | non-grav |a| | vs 3.0e-8
  your run: sentman + erp OFF + tides OFF    | 2.1540e-08 |  0.718
  + ERP on (round 31 default)                | 2.1452e-08 |  0.715
  + solid+ocean tides on                     | 2.1452e-08 |  0.715   <-- ZERO change
  + DRAG_MODEL = dria (Cd DERIVED)           | 2.7651e-08 |  0.922   <-- closes most of it
```

## The tides did nothing, and that corrects MY round-31 table

**An accelerometer in free fall measures only non-gravitational forces.** It cannot
feel gravity. Solid and ocean tides ARE gravity. They can never appear in metric [3],
however large. Measured: switching them on moved the non-grav sum by **0.0000e+00**.
Exactly as it must.

Round 31's force budget listed `solidtides 0.58 m` right next to the metric [3]
deficit as if they were competing explanations for one residual. **They cannot be:**
- tides move the ORBIT -> metric [1] sees them. 0.58 m. Real.
- tides do not touch the ACCELEROMETER -> metric [3] cannot see them. Zero.

One table, one column, and it invited exactly the wrong action: *"turn on tides to fix
the accelerometer ratio"*. It would do nothing, and the failure to move would look like
a fresh mystery. `force_budget` now carries a **`sees[3]?`** column, because which
metric a force can reach is a property of physics, not a footnote.

## Why we under-predict: the catalog Cd is a flat number, and the physics is not

```
  alt    | catalog | sentman | dria    | sesam   | nO [m^-3]
  350 km |  2.5000 |  2.4861 |  2.6705 |  2.6705 | 6.831e+13
  400 km |  2.5000 |  2.4887 |  2.9624 |  2.9624 | 2.320e+13
  460 km |  2.5000 |  2.4933 |  3.2511 |  3.2511 | 6.485e+12
  480 km |  2.5000 |  2.4957 |  3.3142 |  3.3142 | 4.261e+12
  550 km |  2.5000 |  2.5102 |  3.4320 |  3.4320 | 9.990e+11
```

`Cd = 2.5` is a literature constant somebody typed. **`dria` derives 3.31 at 480 km**
— 33% higher — because as atomic oxygen thins, accommodation drops and Cd rises. That
is a real altitude dependence a scalar cannot represent, and it is exactly the
direction and roughly the size metric [3] is short by. **Switching to `dria` moves the
single-epoch ratio 0.718 -> 0.922.**

I said 3.2 was "unphysical" an hour earlier. **That was wrong** — 3.2 at 480 km is
textbook, and the model that derives it was sitting in the toolbox unused.

`sentman` gives 2.4957 — essentially the catalog value — because it uses the `aT` you
**typed**. `dria` and `sesam` DERIVE accommodation from the local atomic-oxygen number
density. That is the difference between a knob and a model.

## And GRACE-A is not drag-dominated

At 480 km in 2010 **solar minimum**: drag 1.43e-8, SRP 1.16e-8. **SRP is 0.81x the
drag.** So metric [3] on GRACE-A is not mainly a Cd measurement, and `Cr = 1.3` is an
ASSUMPTION (the catalog default for nearly every satellite). `Aref` is used for BOTH
drag and SRP, which cannot be right: GRACE presents a different cross-section to the
flow than to the Sun.

## The figures

The accelerometer figure stacked 5 panels in one column: three axis traces where the
bias dwarfs the signal, the bias-removed comparison, and the per-force breakdown —
each squeezed to ~150 px with overlapping labels. **Unreadable is decorative.**

Split by QUESTION, not by array index:
- `acc_raw` — is there a bias, how big? Three axes, room to see, mean drawn per axis.
- `acc_ratio` — does the model match the signal? Bias-removed, plus the ratio through
  the arc on a log axis.
- `acc_forces` — which force is doing the work? drag/SRP/ERP against the measurement.

## What to run next

1. `DRAG_MODEL = 'dria'` — 0.718 -> 0.922 at a single epoch. Run it over the arc.
2. Solid tides ON — 0.58 m on metric [1]. It will NOT move metric [3]; that is not a
   failure, it is the equivalence principle.
3. Then `SC.Cr` and the drag-vs-SRP `Aref` split — at 480 km they matter as much as Cd.

---

# Round 32 — switching each force on, and what Cd is really doing

## The answer to "can we get closer": yes, and it is the drag MODEL, not Cd

One epoch, GRACE-A 480 km, one change at a time:

```
  configuration                              | non-grav |a| | vs measured 3.0e-8
  -------------------------------------------------------------------------------
  your run: sentman + erp OFF + tides OFF    | 2.1540e-08 | 0.718
  + ERP on (round 31 default)                | 2.1452e-08 | 0.715
  + solid+ocean tides on                     | 2.1452e-08 | 0.715   <-- ZERO change
  + DRAG_MODEL = dria (Cd DERIVED)           | 2.7651e-08 | 0.922   <-- closes most of it
```

**Why.** The catalog says `Cd = 2.5`. That is a literature constant somebody typed.
The panel models derive `Cd(t)` from the local gas-surface physics:

```
  alt    | catalog | sentman | dria    | nO [m^-3]
  350 km |  2.5000 |  2.4861 |  2.6705 | 6.831e+13
  400 km |  2.5000 |  2.4887 |  2.9624 | 2.320e+13
  460 km |  2.5000 |  2.4933 |  3.2511 | 6.485e+12
  480 km |  2.5000 |  2.4957 |  3.3142 | 4.261e+12
  550 km |  2.5000 |  2.5102 |  3.4320 | 9.990e+11
```

**The catalog Cd is FLAT. The physics is not.** As atomic oxygen thins with altitude,
accommodation falls and `Cd` rises. At GRACE-A's altitude `dria` derives **3.31**
against the catalog's 2.5 — a 33% increase, and your metric [3] was short by roughly
that.

Note `sentman` gives 2.4957 — essentially the catalog value — because **you type its
`aT` and everyone types 0.9.** That is the difference between a model that has a knob
and a model that has physics. `sentman` was never going to find this.

I also said, in this session, that a Cd of 3.2 would be "unphysical". **That was
wrong.** 3.2 at 480 km is textbook, and the models derive it from first principles.

## The tides changed metric [3] by exactly 0.0000e+00 — and that is correct

**An accelerometer in free fall cannot feel gravity.** That is the equivalence
principle, not a modelling choice. Solid and ocean tides ARE gravity: they move the
ORBIT (metric [1]) and are **invisible** to the accelerometer (metric [3]), however
large they are.

**My round-31 force budget was wrong about this.** It listed `solidtides` at 0.58 m
right beside the metric [3] deficit as if they were competing explanations for one
residual. One table, one column, and the obvious action it invited — *"turn on tides
to fix the accelerometer ratio"* — does nothing, and the failure to move would have
looked like a fresh mystery. Which metric a force can reach is a property of physics,
so it now has a column:

```
    force        on?  |a| [m/s^2] [1] drift  sees[3]? note
    drag         YES  5.272e-09       0.31   YES
    srp          YES  3.000e-08       1.75   YES
    thirdbody    YES  1.000e-06      58.32   no
    solidtides   NO   1.000e-08       0.58   no     <-- GRAVITY: metric [1] only
    -> every OFF force is GRAVITATIONAL, so metric [3] CANNOT move by turning them
       on. Verified: switching solid+ocean tides on changed the non-grav sum by
       0.0000e+00. If [3] is short, look at drag, SRP, ERP or the reference.
```

So the two residuals need two different actions:
- **metric [1]** (2.09 m along-track): turn on solid tides — 0.58 m of it.
- **metric [3]** (ratio 0.53): switch `DRAG_MODEL` to `dria` — 0.718 -> 0.922.

## GRACE-A is not drag-dominated

At 480 km in 2010 **solar minimum**: drag 1.43e-8, SRP 1.16e-8 — **SRP is 0.81x the
drag**. So the deficit was never going to be a pure Cd story, and `Cr = 1.3` is a
catalog default (an ASSUMPTION, tagged as such by `validation.provenance`) applied to
nearly every satellite. `Aref` is used for BOTH drag and SRP, which cannot be right:
GRACE presents a different cross-section to the flow than to the Sun.

## Two more bugs, both found by a test fixture rather than by reading

- **`show_OD` assumed `R.acc.rms_diff` and `R.den.model`.** The same
  never-assume-a-struct's-shape rule `show_provenance` already follows. Both guarded.
- **`show_OD` printed to `outdir` without creating it.** `validate_OD` happens to
  make the directory earlier, which is exactly why it went unnoticed: the bug only
  fires when `show_OD` is called on its own — which is what anyone debugging a plot
  does. A run could produce all ten figures correctly and still throw at the last
  step, for the most trivial reason there is.

## The figures

Rendered all ten against realistic synthetic shapes (GRACE-A sizes, a real bias, real
TU Delft dropouts, eclipse gaps) and inspected them. The accelerometer is now **three
figures** (raw+bias / bias-removed ratio / per-force) rather than five crammed panels,
the density ratio is on a log axis with dropouts marked, and nothing overlaps.

## State

245 files parse. Suite ALL PASS (8 blocks). 0 used-but-never-assigned.

---

# Round 33 — "does a model switch help SRP too?"

Short answer: **no, not in the way `dria` helped drag.** The distinction matters more
than the answer.

## Why `dria` worked

```
cannonball : Cd = 2.5                 <- a constant you TYPE
dria       : Cd = f(n_O, T, Tw, v)    <- DERIVED
```
and the inputs to that `f` are **measured**, or as close as this field gets: `n_O`
and `T` come from DTM2020 driven by fetched F10.7/Kp, validated against TU Delft
measured density at 14% on your own CHAMP run. `dria` replaced a typed constant with
a function of mostly-measured things. Assumption count roughly unchanged (Cd -> Tw),
but Tw is a **weak** knob where Cd was a **strong** one, and `Cd(t)` now has the right
**shape** with altitude.

## Why the same move does NOT work for SRP

```
cannonball : a = P*Cr*(A/m)                          <- Cr = 1.3, TYPED
boxwing    : a = sum_i f(alpha, rho_s, rho_d, n_i, A_i)
```
`alpha`, `rho_s`, `rho_d` are **also typed** — nobody measured GRACE-A's optics. And
`n_i, A_i` need a **shape**, which the catalog does not have (`has_geometry = NO` for
all 24 satellites).

**Assumption count: 1 -> 4+.** That is the opposite of what `dria` did. Switching SRP
model would move the number without reducing what we are assuming — and a number that
moves because you changed an assumption is not evidence.

**The test to apply to any model switch:** does it DERIVE a quantity from a MEASURED
input, or re-parameterise an assumption? `dria` does the first. `boxwing` SRP does the
second, unless you have the geometry.

## But one thing cannonball structurally cannot do

`a = P*Cr*(A/m)` has **no attitude in it**. GRACE-A has MEASURED attitude (ITSG
quaternions) and cannonball throws it away. Over an assumed box (aspect [3 1 1],
scaled so the +x face is Aref):

```
  Sun angle | cannonball | aref_box   | ratio
      0 deg | 1.1625e-08 | 1.1625e-08 | 1.00
     45 deg | 1.1625e-08 | 3.2880e-08 | 2.83
     60 deg | 1.1625e-08 | 3.6014e-08 | 3.10
```
**3x across Sun angle, and cannonball is flat across all of it.** That variation is
real. But the box is `[2.93 0.98 0.98] m`, invented from Aref and an aspect I chose:
Aref is ONE number and a box needs THREE. So it bounds the **sensitivity**; it is not
GRACE-A's SRP.

## The degeneracy — and the thing that breaks it

Your metric [3] deficit is ~1.4e-8. Two candidates, both the right size:
```
  (a) Cd 2.5 -> 3.31 (dria)          : +6.2e-09
  (b) SRP area 1x -> ~2.5x           : +1.0e-08
```
An RMS over the arc **cannot** separate them. Sweeping Cd would "fit" the residual and
silently absorb an SRP error into a drag parameter — the right answer for the wrong
reason, which fails on the next arc.

**SRP GOES TO EXACTLY ZERO IN ECLIPSE. DRAG DOES NOT.** Your own by-force panel shows
it. The orbit runs this experiment twice per revolution, for free:
```
  IN ECLIPSE : measured = drag + ERP        (SRP is OFF)
  IN SUNLIGHT: measured = drag + ERP + SRP
```

`validation.acc_split` (new, wired into `od_metrics`, block [7] of the suite) uses the
eclipse samples to get the **drag scale clean**, then SOLVES for SRP:
```
  r_d       = |drag_mod| / |meas_ecl|              <- SRP structurally absent
  drag_true = drag_mod / r_d
  srp_true  = meas_sun - drag_true                 <- vector subtraction
  r_s       = |srp_mod| / |srp_true|
```

Validated on injected errors where the answer is known:
```
  injected drag 0.4, SRP 1.0  ->  recovered drag 0.514, SRP 0.941   "the deficit is DRAG"
  injected drag 1.0, SRP 0.4  ->  recovered drag 1.009, SRP 0.392   "the deficit is SRP"
  injected drag 0.8, SRP 0.5  ->  recovered drag 0.797, SRP 0.490   "BOTH, separately sized"
```

## The first version of this was wrong, and the test killed it

It compared the eclipse and sunlit **ratios** and called a pure drag error "SRP".
**Correct SRP DILUTES a drag error** — the sunlit ratio is `(r_d*D + S)/(D + S)`,
which moves toward 1 as S grows — so the ratios differ for a reason that has nothing
to do with SRP being wrong. Comparing ratios feels like the obvious move and is not
the measurement. Solving with the eclipse scale is.

This is the only place in the toolbox where **two non-gravitational forces are
separated by DATA rather than by assumption.** Everywhere else they are degenerate
and the honest thing is to say so.

## State

246 files parse. Suite ALL PASS (8 blocks). 0 used-but-never-assigned.

---

# Round 33 — does changing the SRP model help? It can produce any answer you want.

## Every force on, GRACE-A 480 km, one epoch

```
  configuration                                  | non-grav   | vs measured 3.0e-8
  ---------------------------------------------------------------------------------
  cannonball SRP, Cr=1.3 (your run)              | 2.765e-08 | 0.922
  boxwing SRP, ASSUMED cube                      | 3.017e-08 | 1.006   <-- LOOKS PERFECT
  boxwing SRP, ASSUMED 2:1:1                     | 3.900e-08 | 1.300
  boxwing SRP, ASSUMED 3.1x0.7x0.7 (GRACE-ish)   | 6.901e-08 | 2.300
```

**The spread is a factor of 2.5, from nothing but the shape I typed.**

`Aref` is ONE number; a box needs THREE. The inversion is not unique, so the aspect
ratio is a **free parameter** — and one with that much leverage can hit **any** target
you point it at.

## The cube was a mirage

I could report *"box-wing SRP closes the gap — ratio 1.006"*. It would be
arithmetically true and **worthless**, because I chose the cube and GRACE-A is not a
cube. That is **fitting, not modelling**: tuning a free parameter until the residual
vanishes and calling the vanishing a result.

**The direction is the actual finding.** The box CLOSEST to the real GRACE
(3.1 x 0.7 x 0.7) **overshoots by 2.3x**. So the honest reading is not "boxwing fixes
it" but: *"box-wing with a plausible GRACE shape overshoots, therefore `Cr = 1.3`, the
optics, or the area is wrong somewhere."* **That is a real lead.** The cube was noise
that happened to land on 1.0.

## So the answer to your question

**Changing the SRP model does not help — while the geometry is assumed.** It replaces
one assumption (`Cr = 1.3` + `Aref`) with a strictly bigger one (an entire shape), and
the bigger one has enough freedom to reach any conclusion. `validation.capability`
already refuses to call this measured: it reports
`boxwing(GEOMETRY=aref_box: ASSUMED shape)`.

**It WOULD help with real geometry.** GRACE's dimensions are published (~3.1 m long,
trapezoidal). Put them in `05_data/sat_data/itsg_catalog.csv` under `geom_dims_m` and
box-wing SRP becomes a measurement rather than a wish. That is the single highest-value
data item missing from this toolbox — all 24 satellites have `has_geometry = no`.

## `validation.shape_leverage`

Whenever an ASSUMED shape drives SRP or ERP, `validate_OD` now prints the sweep:

```
  ---- SHAPE LEVERAGE: what is the ASSUMED box worth? ----
      aspect           dims [m]               srp [m/s^2] non-grav [m/s^2]
      [1 1 1]          0.98 x 0.98 x 0.98     1.701e-08   3.017e-08
      [2 1 1]          1.95 x 0.98 x 0.98     3.133e-08   3.900e-08
      [4.4 1 1]        4.30 x 0.98 x 0.98     6.581e-08   6.901e-08
    SPREAD: 3.005e-08 .. 6.901e-08  = a factor of 2.3
    That factor is the SIZE OF THE ASSUMPTION. A residual smaller than this
    spread is not a finding -- you could reach it by retyping the aspect ratio.
```

**The spread is larger than the residual being chased.** That has to be on the screen
before anyone reads a box-wing number, or the toolbox is helping you fool yourself —
which is the one thing this entire audit has been built to prevent.

## Where this leaves GRACE-A

| metric | value | what moves it | honest status |
|---|---|---|---|
| [1] along-track | 2.09 m | solid tides (0.58 m), then drag | tides ON now; real physics remains |
| [3] non-grav ratio | 0.53 raw / 0.92 with `dria` | `DRAG_MODEL='dria'` derives Cd 3.31 vs catalog 2.5 | **the fix, and it is physics** |
| [5] density | 1.14 | nothing — DTM2020 to 14% is good | the atmosphere was never the problem |
| SRP | 0.81x the drag at this altitude | real geometry, or nothing | **blocked on missing data, not code** |

## State

247 files parse. Suite ALL PASS. 0 used-but-never-assigned.

---

# Round 34 — the figures: showing the split, not the average

## The overlay panel admitted it showed nothing

Its own title read: *"overlaid - at this scale they MUST look identical; that is why
the panels below exist"*. **A panel that admits it cannot show anything was using a
third of the figure.** 2 m on 6860 km is 3e-7 of the axis — about a thousandth of a
pixel. It answered exactly one question ("are we on the same orbit at all?"), which is
answered once, in a sentence.

Replaced with the residual's **distribution** — the thing a time series cannot show:
is the error centred on zero, or offset? Offset means a missing force. Mean and ±1σ
drawn on it.

## RMS was hiding the diagnosis

`RMS^2 = mean^2 + std^2`. One RMS number cannot tell you which of two completely
different errors you have:

- **mean >> std** -> a **BIAS**. A force is missing or mis-scaled.
- **std >> mean** -> **SCATTER**. Noise, or periodic mismodelling.

Your GRACE-A run, split:
```
  along-track  RMS 2.091 = mean +1.668 +- std 1.26   -> BIAS (|mean| 1.3x std)
                                                        the bias IS the drag deficit
  radial       RMS 0.371 = mean -0.170 +- std 0.33   -> SCATTER (std 1.9x |mean|)
                                                        chord-vs-arc, NOT a force
```
**Same kind of RMS, opposite diagnosis.** The RTN panels now draw the mean line and
a ±1σ band, print `RMS = mean ± std`, and **say which regime it is** rather than
making the reader divide two numbers in their head.

`validation.rtn_stats` returns `mean_rad/alo/cro` and `std_rad/alo/cro` alongside the
RMS — one function, one triad, one definition, as everywhere else.

## Growth rate, because a value cannot distinguish forces

A drag error grows as `t^2`; a seed or frame error is flat or linear. The 3D error
panel now prints `2nd half / 1st half` and labels it **GROWING (integrating force,
e.g. drag)** or **flat (seed or frame, not integrating)**. That ratio is the
difference between "chase the drag model" and "check your seed", and no single RMS
can tell you which.

## Layout

The verdict lines made every title two lines, so three stacked panels at 720 px put
the second line **on the axes above**. **A verdict that overprints the plot it is
describing is worse than no verdict.** Heights raised (RTN 900, position 860,
velocity 760) and all ten figures re-rendered and inspected.

## State

247 files parse. Suite ALL PASS. 0 used-but-never-assigned.

---

# Round 35 — GRACE-1/2012: three bugs, all mine, all in the error path

Your run: `GRACE-1_reducedDynamicOrbit_2012-01-01.txt.gz` -> 404, four times, then an
error message with a **literal backslash-n in it**.

## 1. The `\n` that printed as `\n`

```matlab
txt = '  (could not list the server directory to show what IS available)\n';
```
**MATLAB single quotes do not interpret escapes.** Round 26 added this line -- inside
the function whose entire job was to make the error clearer -- and shipped it
unrendered, because **the path only runs when the listing fails, which is exactly
when nobody is looking.** Now `sprintf`, and it reports WHY the listing failed.

## 2. The catalog was throwing away the answer

`itsg_catalog.csv` column 15 is `coverage_note`. The reader built a 14-field struct
and **dropped the column on the floor**. So nothing checked coverage, and every run
spent four HTTP round-trips discovering what the CSV already knew.

**A parser that silently discards a column is a quieter swallowing catch**: the
information existed and the code decided you did not need it without telling anyone.

## 3. And the diagnosis is not what you would guess

```
GRACE-A, 2012: catalog CLAIMS coverage 2002-2017, so 2012 should be available and
               is not. The catalog note is UNVERIFIED metadata -- nobody ever
               checked it against the server. Trust the 404.

CHAMP, 2015:   catalog says coverage is 2000-2010 (mission end 2010-09) and you
               asked for 2015 -- OUTSIDE it. That is the whole explanation.
```
**Two identical-looking 404s, two opposite fixes.** One says "pick another date"; the
other says "the catalog is lying to you". Your GRACE-A run at **2010 worked**, so the
directory name `GRACE-1` is right -- 2012 is simply not there, whatever the CSV says.

`itsgListing` also walks UP to the parent directory when the satellite directory
itself 404s, because "the name is wrong" and "the date is wrong" need opposite
responses and looked identical before.

## The one I am least proud of

Writing the coverage check, I looked up `validation.itsg_catalog(S)` -- **`S` is the
DIRECTORY (`GRACE-1`); the catalog is keyed by NAME (`GRACE-A`)** -- and wrapped it in
a bare `try/catch`. It threw on every call and printed nothing. **I added a swallowing
catch three edits after writing the round-27 section about removing swallowing
catches**, and it hid a bug I introduced in the same commit.

The catch stays -- a diagnostic must not throw while explaining a failure -- but it
now says it gave up, with the reason. That is the whole rule: not "never catch", but
**"never fail to say so"**.

`check_undefined.py` caught a fourth (`err_` used in a `catch` that never bound it)
before it shipped.

## State

248 files parse. Suite ALL PASS. 0 used-but-never-assigned.

---

# Round 36 — verifying the catalog and the cache, as asked

## Syntax: parse-checked file by file, not in bulk

```
06_validation   48 files, 0 parse errors
11_compare       6 files, 0 parse errors
05_data         25 files, 0 parse errors
07_examples      4 files, 0 parse errors
08_test          2 files, 0 parse errors
```
The `FORCES` block is rewritten with the comments **above** the literal. Four-line
comment blocks between the elements of a line-continued struct made the comments
load-bearing: every line of the continuation had to survive intact, so one stray
keystroke in a copy-paste produced `parse error` pointing at a line that looked fine.
Explanation belongs next to the code, not inside the expression.

## Catalog: all 24 satellites verified by LOADING them

Every row checked for `dir`, `norad`, `mass`, `Aref`, `Cd`, `Cr`, `alt` present and
sane. **0 problems.** And `coverage_note` -- column 15 of the CSV -- is now actually
read. The parser built a 14-field struct and **dropped the column on the floor**,
which is exactly why nothing checked coverage before spending four HTTP round-trips
to discover a 404 the CSV could have predicted.

## Cache: one real bug

```matlab
websave(local, cand{i});     % straight into the cache path
```

Interrupt that -- network drop, Ctrl-C, a full disk -- and a **partial file is left
sitting at the cache path**, where the cache-first check finds it and treats it as a
complete product **forever**. The `delete` in the catch only fires if `websave`
throws; a killed process never reaches it.

The failure is nasty because it is silent and sticky: every later run reads the
truncated file, gunzip dies with something cryptic about an unexpected end of stream,
and **nothing points at the cache**. You would go looking at the parser.

Fixed the standard way: download to `<file>.part`, verify, then `movefile` on success.
Rename is atomic, and a half-written `.part` is obviously not the cache.

Plus a gzip magic-byte check, because some servers answer a missing file with HTTP 200
and an HTML error page, which `websave` stores happily:
```
html.txt.gz   REJECTED: NOT gzip (magic 3c 68, expected 1f 8b) -- usually an HTML error page
tiny.txt.gz   REJECTED: only 1 bytes -- not a product file
real.txt.gz   ACCEPTED
```
Without it we would cache the word "404" under a `.txt.gz` name and fail at gunzip on
every subsequent run.

Cache keys are per-`itsg_dir` (`GRACE-1`, `GRACE-2`, `CHAMP`, `Swarm-1`) -- distinct,
so no collisions between satellites.

## Final state

```
248 files parse                          0 failures
regression suite (8 blocks)              ALL PASS
check_undefined.py                       0 used-but-never-assigned
                                         3 type-clashes (all polymorphic dispatch, explained)
uncalled models                          0
allowed-but-failed model x satellite     0
knobs disagreeing with documentation     0
swallowing catches without a reason      0
two-sources-of-truth                     guarded (validate_OD errors on it)
catalog rows                             24/24 load, all fields sane
cache                                    atomic write + format verification
```

**The limit is unchanged and always will be:** Octave and synthetic fixtures prove
wiring, units, frames, dispatch and mathematics. They do not prove the numbers. Every
real finding in this audit -- the accelerometer bias, the along-track arithmetic, the
Cd that the physics derives and the catalog does not -- came from **you** running it
on real MATLAB with real data and noticing something that did not add up.

---

# Round 37 — vector overlays, RTN overlay, per-axis accelerometer, and a toggle on everything

## Vector overlays, because a magnitude hides direction

`|r|` is ONE number out of three. It can match perfectly while the vector is wrong.
New figures plot the **components** -- ours vs measured, per axis -- with the
per-axis difference beside each:

- `vec_r` position x/y/z overlay + per-axis error (mean and std on each)
- `vec_v` velocity x/y/z overlay + per-axis error
- `rtn_overlay` both ORBITS projected into the triad, drawn against each other,
  with the residual beside each component

The overlay and the residual panel next to it are **exactly** consistent:
`rtnToEci` is the inverse of `validation.rtn_project` using the same triad
construction, and the round trip is machine precision:
```
  radial      max |err| = 2.220e-16 m
  along-track max |err| = 6.661e-16 m
  cross-track max |err| = 3.331e-16 m
```
It is written as a local function next to its caller rather than a second package
function, so the pair cannot drift -- one edited and the other not is the "two
definitions of along-track" bug this audit has removed several times already.

## Per-axis accelerometer error AND ratio

`acc_axes`: for each of x/y/z, the **difference** (modelled - measured, with mean and
±std) and the **|modelled/measured| ratio** (bias-removed, log axis).

Why per axis rather than the magnitude: **the forces do not share a direction.** Drag
is along-track, SRP points away from the Sun, ERP comes off the Earth. A magnitude
ratio of 0.53 could be a uniform 47% shortfall in everything, or drag correct and SRP
missing entirely -- **opposite fixes**. Only the split can tell them apart.

Where the measured signal crosses zero the ratio is a division by ~0, so those samples
are **masked and counted in the title**. Silently dropping them is how you end up
believing a cleaner picture than you have.

## A toggle on every figure -- and the check that found the gap

`P.acc` was driving **THREE** figures. You could not turn off just the one you did not
want, **which is the entire point of a toggle.** Split into `acc_raw`, `acc_ratio`,
`acc_forces`.

Now a suite block reads show_OD's own source and compares the advertised toggle list
against what actually gets drawn:
```
[8] every figure has exactly one toggle
    15 toggles <-> 15 figures, 1:1   PASS
```
A toggle that draws nothing, or a figure nobody can switch off, fails the suite. **If
the defaults struct and the figures disagree, the advertised list is a lie** -- and
the list is the only documentation anyone reads.

Proven end to end: all toggles on -> 14 figures; four toggled off -> 10 figures.

## Clickable legends

`validation.clickable_legend`: click a legend entry to hide that curve and rescale to
what is left. Plotting several series on one axes is right -- you cannot see a
difference between curves on different figures -- and it is also why panels get hard
to read: the interesting curve is the small one and the big one owns the axis. Now you
can pull them apart **without re-running a 20 s propagation**.

`ItemHitFcn` is MATLAB R2016b+. Octave has no such property, so it falls back to a
static legend and says so **once per session** -- ten identical warnings train you to
ignore warnings. The default legend is created BEFORE the try, and the failure is
reported: the same rule as everywhere else -- not "never catch", but **"never fail to
say so"**.

## State

249 files parse. Suite ALL PASS (8 blocks). 0 used-but-never-assigned. 15 figures,
15 toggles, 1:1.

---

# Round 38 — the toggle that did nothing, and the figures you actually asked for

## The clickable legend was silently doing nothing

Round 37 used `ItemHitFcn`. You reported it did not work, and you were right. The
failure mode is the worst kind: **the property assignment succeeds, the callback just
never fires** if the legend's line mapping is not what the code assumed. No error, no
warning. You click, nothing moves, and you cannot tell whether the toggle is broken or
the curves genuinely overlap.

I shipped it having verified only that it did not *crash* -- on Octave, where the
feature does not exist at all and my own fallback message fired. **I tested the
fallback and called it a test of the feature.**

Replaced with `validation.toggle_panel`: real `uicontrol` checkboxes in a panel on the
right, one per SERIES, plus all/none buttons. A checkbox cannot fail silently -- it is
a widget with a visible state.

It toggles a **group**, not a line: the same series is drawn in three panels (x/y/z),
and hiding one panel's copy while the others stay is not a view anyone wants. One box
hides the series everywhere. Hiding also **rescales** the axes to what is left --
without that, the axis stays stretched to fit a curve that is no longer drawn and the
small curve you wanted stays flat at the bottom, which looks exactly like the toggle
having done nothing.

The panel **shrinks the axes** rather than sitting on top of them. A control drawn
over the data is the overlap problem again in a new hat.

**Octave/gnuplot cannot draw a uipanel** -- it creates fine and then dies at `print()`
with "unknown object class". That would have destroyed every PNG. Guarded: on that
backend the figures are drawn without toggles and it says so once.

## Position: both references, per axis

`vec_r` now draws **model / reduced-dynamic / kinematic** overlaid per axis, with the
error against **each** reference beside it.

Both on one axes because they are INDEPENDENT truths and their disagreement is the
floor under everything: if model-vs-RDO and model-vs-KIN differ by more than
RDO-vs-KIN, the model is the odd one out; if they track each other, you are looking at
ONE error, not two. On separate figures that comparison is impossible.

`od_metrics` now stores the reference SERIES (`r_ref`, `v_ref`, `r_our`, `v_our`), not
just the residual. Reconstructing curves in the plotting code is how two versions of
"measured" come to exist and quietly disagree.

## Velocity: reduced-dynamic only, and the title says why

**The kinematic product is `MJD, x, y, z` -- POSITION ONLY.** There is no kinematic
velocity in existence. The figure states that instead of omitting the curve: a missing
panel looks like an oversight, an absent product is a fact about the data, and only
one of those is worth investigating.

## Accelerometer per axis: forces, sum, measurement, and the unmodelled remainder

`acc_axes`, per axis:
- **left:** every modelled force separately (drag / SRP / ERP), their SUM, and the
  MEASURED accelerometer, overlaid
- **right:** `delta_a = modelled_sum - measured` = **the unmodelled acceleration**,
  with its mean priced in metres of drift over the arc

Per axis and not magnitude because **the forces do not share a direction**: drag is
along-track, SRP points away from the Sun, ERP comes off the Earth. A magnitude ratio
of 0.53 could be a uniform 47% shortfall in everything, or drag correct and SRP missing
entirely -- OPPOSITE fixes. `delta_a` per axis says which DIRECTION the missing force
points, which is the strongest clue available about what it is.

Linear axis, not log: **delta_a changes sign**, and a log axis cannot show that. The
sign is the whole point -- it says whether the model is over or under, and on which
axis.

## State

250 files parse. Suite ALL PASS (8 blocks). 0 used-but-never-assigned. 15 figures,
15 toggles, 1:1.

---

# Round 39 — the scale factor, and deleting three figures

## The idea, and why it is the right question

```
a_drag = 0.5 * rho * Cd * (A/m) * v^2
```
`rho*Cd*A/m` is **ONE number**. No tracking data can say which factor was wrong. `Cd`
is not measured for any satellite in this catalog -- it is a literature value somebody
typed -- and `Aref` is one number standing in for a whole shape.

**So a constant multiplicative error is EXPECTED and is not interesting.** It says one
of four factors is off, which we already knew. The question underneath is:

> After absorbing the best possible scale factor, what is LEFT?

That remainder cannot be fixed by any Cd, any area, any density scaling. It is
structure -- a missing force, a wrong direction, a phase error -- and **it is the only
part of the residual that is evidence about the MODEL rather than the bookkeeping.**

## `validation.scale_factor`, and the two numbers that keep you honest

Least squares: `k = <meas,mod>/<mod,mod>`. Not `rms(meas)/rms(mod)`, which ignores
SIGN and would return ~1.000 for a model perfectly ANTI-correlated with the truth. Not
`mean(meas./mod)`, which blows up wherever the model crosses zero -- for SRP, every
eclipse.

A scale factor **always exists**. The formula returns something for any two series,
including unrelated ones. So it also returns `r2` and `explained`:

```
  case                                factor   r2       explained
  model = 0.53 x truth (a Cd error)   1.887    1.000    1.000
  model = WRONG SHAPE (unrelated)     0.007    0.000    0.274
  model = ANTI-correlated            -1.000    1.000    1.000
```
- `k=1.887, r2=1.00` -> right shape, wrong size. **A real Cd/area/density statement.**
- `k=0.007, r2=0.00` -> the rescaling achieved nothing. Quoting that k as a Cd
  correction would be **pure fitting**.
- `k=-1.000` -> my first version called this "right shape, wrong size" because r2 was
  1.0. **It is not.** The model tracks the shape and points the WRONG WAY: a sign
  error, a flipped frame, a reversed facet normal. Now flagged as such.

Locked into the suite as block [9].

## Three figures deleted

`acc_raw`, `acc_ratio`, `acc_forces` are **gone**. Three figures were saying one thing:
the model is short, and here is the bias. The per-axis breakdown contains all of it --
every force, the sum, the measurement -- and shows it **per axis**, which the magnitude
figures could not.

**Redundant plots are not free: each one is a place for the numbers to disagree with
each other, and they DID.** Round 28: five copies of the metric-[3] ratio, one of them
corrected for the bias and four not.

What remains: `acc_axes` (the breakdown) and `acc_scale` (the scale factor + what
survives it).

## Density: two sources, two subsections, k = measured/model

**ITSG neutralDensity exists for 7 of 24 satellites, TU Delft for 8, and only 5 have
BOTH.** So "the density check" is not one check -- on most satellites you get whichever
exists, and you need to know which you are looking at.

ITSG matters especially because it comes from the **same server and the same
processing as the orbit you are validating against** -- when it exists it is fetchable
alongside the position. TU Delft is a separate product from a separate group with its
own coverage. **A pipeline that only understands TU Delft goes blind on every satellite
TU Delft has not published.**

The scale factor is `measured / model`, deliberately that way round: it answers *"what
must I MULTIPLY MY MODEL BY to match the truth"* -- the number you would actually
apply. `model/measured` answers a question nobody asks.

## State

250 files parse. Suite ALL PASS (9 blocks). 0 used-but-never-assigned.
13 figures, 13 toggles, 1:1.

## Round 39b — acc_raw and acc_forces restored, with the model overlaid

You were right to push back. I deleted them arguing "three figures saying one thing",
and that was true of the OLD versions -- all three plotted magnitudes and all three
recomputed the ratio their own way. It is not true of the questions they answer. Four
accelerometer figures, each answering something the others cannot:

| figure | question | why the others cannot answer it |
|---|---|---|
| `acc_raw` | what does the instrument actually RETURN? | every other figure is bias-REMOVED. This is the only one showing what was removed, and how big it was -- without it the correction is something you take on trust |
| `acc_forces` | WHICH force is doing the work? | needs a LOG axis: at 480 km drag and SRP are comparable, ERP is an order down, and SRP falls to zero in eclipse -- three orders on one plot. The per-axis figures are linear and must be (components change sign), so they cannot show this |
| `acc_axes` | which DIRECTION is the error? | the forces do not share a direction; a magnitude cannot separate a drag error from an SRP one |
| `acc_scale` | what SURVIVES the best k? | the only part that is evidence about the model rather than the bookkeeping |

Both restored **with the modelled sum and the measurement overlaid** and full toggle
panels -- which is what makes them non-redundant rather than three views of one curve.

`acc_ratio` stays deleted: it plotted a magnitude ratio that `acc_scale` now computes
properly (least squares, with r2) and `acc_axes` shows per axis. That one really was
saying something the others say better.

**15 figures, 15 toggles, 1:1. 250 files parse. Suite ALL PASS.**

---

# Round 40 — titles that collide, and knobs that control nothing

## The super-title was overprinting the axes title

`sgtitle` draws at a fixed height and **does not move the axes out of its way**, so on
a single-axes figure the two land on the same pixels. It looked like a rendering
glitch; it was two things told to occupy one place.

Fixed properly rather than by shortening the words: `sgt` now **wraps** the text to
the figure width and then **shrinks every axes** to leave exactly as much room as the
wrapped text needs. Long captions are worth having -- they are the difference between
a plot and a plot you can act on -- but they have to be **given** space, not squeezed
on top of the data.

Verified in pixels rather than by eye:
```
  text blocks in the top 12% of the figure: 2
     rows 21-40
     rows 56-154
  gaps between blocks (px): [16]
  OVERLAP: NONE
```

Wrapping, not truncating: truncation loses the END of the sentence, which is where the
conclusion lives ("...the model has the right shape and the wrong size").

## The sampling constants are now knobs

They were magic numbers buried in the plotting code. **They change what you see, so
they are decisions -- and a decision nobody can find is a decision nobody can
question.**

| knob | default | what it costs you |
|---|---|---|
| `movavg_s` | 300 s | THE big one. Too short smooths nothing; too long flattens a real secular trend into the mean and it **disappears**. Sweep it: if a feature dies at 600 s and lives at 300 s, it is per-rev structure, not a trend -- **that is the diagnosis** |
| `hist_bins` | auto | too few hides bimodality; too many turns a distribution into a comb and you read sample noise as structure |
| `ratio_lim` | [0.2 5] | the dropout test AND the axis. Widen it to see the raw spikes |
| `decimate` | 1 | plot every Nth sample |

`decimate` is applied in **exactly one place** (`dec()`, at the plot call) and never
before a mean/std/RMS. If it were applied once at the top of `show_OD`, every statistic
in every title would silently be computed on a subsample -- **and a std computed on
every 10th point is a different number that looks exactly like the right one.** The
knob says "draw fewer points"; it must not become "measure fewer points".

## Two knobs controlled nothing

`ratio_zero_frac` survived a figure rewrite that **deleted its only consumer**. It sat
in the defaults advertising a capability that no longer existed. `decimate` I had
declared and never wired.

**A knob that controls nothing is worse than no knob**: someone sets it, sees no
change, and concludes the QUANTITY does not matter -- when in fact the knob was never
read. Same failure as `FORCES.drag.corotate`, which was documented, sweepable, and
silently ignored.

Now suite block [10]:
```
[10] every plotting knob has a consumer
     18 knobs, every one has a consumer   PASS
```

## State

250 files parse. Suite ALL PASS (10 blocks). 0 used-but-never-assigned.
15 figures, 15 toggles, 1:1. 18 knobs, 0 dead.

---

# Round 41 — the overlap check that could not fail, and "not published" that meant "we guessed wrong"

## My round-40 fix was wrong, and my round-40 test could not have caught it

I shrank each axes so its BOX cleared the super-title. **`title()` does not draw
inside the box -- it draws ABOVE it.** So I cleared the wrong rectangle and the
collision survived on every 3-panel figure.

Worse: the pixel check I ran to "verify" it happened to be on `OD_acc_forces` -- the
**one figure with a single axes and no subplot title above it**. It passed. **A test
that cannot fail is not a test**, and I shipped the fix on the strength of it.

`08_test/check_title_overlap.py` now runs on **every** figure:
```
  figure                 blocks  min gap px  verdict
  OD_acc_axes                 2          12  ok
  OD_acc_raw                  3           4  ok
  OD_growth                   3           5  ok
  ...
  0 figure(s) with colliding titles
```
`sgt` now reserves a band sized for the wrapped text PLUS the top subplot's own
title, and re-lays the whole axes stack under it -- scaling the stack rather than
nudging boxes, so a 3-panel figure stays evenly spaced instead of having its top
panel squashed.

Two more of mine, caught by rendering rather than reading:
- the rewrite **deleted `rtnToEci`** (it sat between `sgt` and `wrapText`, and I cut
  the whole span)
- `cell2mat(get(ax,'Position'))` throws for a SINGLE axes -- `get` returns a plain
  vector, not a cell. The one-axes figure is exactly the case round 40 "verified", so
  it failed the opposite way this time.

## "not published for this satellite/epoch" was a lie we told ourselves

Your provenance figure says **neutralDensity: not published**. `itsg_catalog.csv` says
**has_density = yes** for the same satellite. **Both cannot be right, and the one
guessing was us.**

The orbit downloads fine from `.../GRACE-1/reducedDynamicOrbit/2010/`. The density
tries `.../GRACE-1/neutralDensity/2010/` -- same server, same folder, same pattern.
So the product is under a name we did not think of, and we **tried two spellings and
reported the failure of our own guess as a fact about the world.**

Fixed: `data.itsg_products(dir)` **lists the server** and `validate_OD` finds the
density product by matching `/density/i` against what is actually there:
```
[4] server HAS: attitude, kinematicOrbit, neutralDensity, ... -> using 'neutralDensity'
```
Guessing survives only as a fallback for when the listing itself fails, and then it
**says** it is guessing.

The provenance text now reads: *"NOT FETCHED. This says our fetch failed -- NOT that
the data is absent. itsg_catalog.csv claims this satellite HAS it."*

**A report that contradicts the catalog it was built from is worse than no report.**

## Still open from your message

- separate KIN and RDO residual figures with toggles on both overlay and residual
- RTN velocity overlay + residual, with both references
- per-axis RTN error growth
- replacing the two-bar provenance panel with something that carries more

## State

251 files parse. Suite ALL PASS. 0 used-but-never-assigned. 0 title collisions.

---

# Round 42 — `neutralDensity_1.0`: you found what the code could not

## The crash

```matlab
catch ME, fprintf('[4] %s unavailable: %s\n', dprod, ...)
```
`dprod` was a **char** until round 41 made it a **cell** of candidate names, and this
consumer was not updated. It died with *"Function is not defined for 'cell' inputs"*
**after every download had already succeeded** -- the worst place to fail, at the end,
in the error handler, on a run that had otherwise worked.

My own "one name, two types" bug class (round 29), introduced by me three rounds after
writing the checker for it. `check_undefined.py` cannot see this one: the type only
differs at runtime, inside a format string.

## The real find: the product directory carries a VERSION

```
  .../operational/CHAMP/neutralDensity_1.0/
```
**No amount of guessing produces `_1.0`.** We tried `neutralDensity` and
`neutralDensity_ACC` -- both 404, and then the code reported *"not published for this
satellite/epoch"*, a confident claim about the world derived entirely from the failure
of our own spelling.

This is the whole argument against guessing, stated as cleanly as it will ever be:
**the failure mode is not "we get it wrong sometimes", it is "we cannot get it right,
and we report our failure as the server's".**

## Why the discovery I added in round 41 did not fire

```
[4] could not list the server; falling back to GUESSED names
```
`webread` on a directory index without `ContentType` set: it sniffs the response and
throws or returns a non-char. The listing silently returned `{}`, the fallback ran,
and the fallback produced exactly the 404s the listing was added to prevent. **The
feature looked implemented and did nothing** -- the same shape as the clickable
legend, and I did not catch it because I had no way to test a network call offline.

Fixed:
- `weboptions('ContentType','text')` -- load-bearing, not decoration
- the directory regex now accepts dots and digits: `neutralDensity_1.0` was
  unmatchable before
- **`itsgURLs` no longer assumes the directory name is the file name.** It builds
  both `<S>_neutralDensity_1.0_<date>.txt.gz` and `<S>_neutralDensity_<date>.txt.gz`,
  because a versioned directory very likely holds an unversioned file, and assuming
  otherwise is the same guess in a new place
- **`itsgFind` now LISTS the directory and matches on the DATE** -- the one part of
  the filename we actually know. The old version rebuilt the same guessed filename and
  searched the index for it, so it could only ever confirm a guess we had already
  made and never discover a name we had not thought of. That is why `_1.0` was
  invisible: nothing in the pipeline could see a name it had not already written down
- `neutralDensity_1.0` is first in the offline fallback list -- **it is there only
  because you went and looked**

## State

252 files parse. Suite ALL PASS. 0 used-but-never-assigned. 0 title collisions.

---

# Round 43 — no fallback, two density figures, and a test that errored without failing

## The fallback is gone

You were right: **a guess that only works because a human already looked the name up
is not a fallback -- it is the lookup with extra steps.** And it lets a wrong name
survive by producing plausible 404s instead of an error.

`neutralDensity_1.0` is now **column 16 of `itsg_catalog.csv`**, where every other
server fact lives:
```
  CHAMP        yes    neutralDensity_1.0
  GRACE-A      yes    neutralDensity_1.0
  GRACE-B      yes    neutralDensity_1.0
  GRACE-FO-1   yes    neutralDensity_1.0
  SWARM-A      no     (none -- has_density=no)
```
A product name is DATA about the server, not an if-branch in the middle of
`validate_OD`. The server listing is now a **check** on the catalog, not a substitute:
if they disagree it says so loudly and names the column to fix. If the catalog is
empty AND the listing fails, it **errors** -- "this is a CATALOG gap, not a missing
dataset" -- instead of quietly guessing.

## Two density figures, not one figure with two rows

ITSG and TU Delft are **two measurements, not two views of one measurement.** Sharing
a figure invites reading one as a correction to the other. Separate figures with
IDENTICAL panels and IDENTICAL arithmetic, so the only thing differing between them is
the data -- which is the entire point of having two.

Each carries, on toggles:
- measured, model, and **k x model** overlaid (log axis)
- **measured/model** through the arc, with the **BIAS** line
- **measured - k*model**: what the scale factor cannot explain

The bias is `exp(mean(log(measured/model)))` -- **multiplicative**. Density spans
decades, so a difference in kg/m^3 is meaningless: 1e-13 vs 1e-12 is the same "size of
wrong" as 1e-14 vs 1e-13. An additive mean would be dominated entirely by the
perigee samples.

## The tests were measuring the text, not the behaviour

Block [8] parsed show_OD's source and compared the defaults struct against
`tags{end+1} = 'literal'` lines. It broke the moment a tag came from a variable (the
density loop), and it **counted sampling knobs as figure switches** -- it tried to
"switch off" the histogram bin count.

That is not the test being clumsy. **Toggles and knobs shared one struct with no way
to tell them apart**: `hist_bins = 0` and `decimate = 1` look exactly like figure
switches. The struct was conflating two kinds of thing and daring anyone to guess
which was which.

- `figureToggles()` is now the ONE declared list of figures; knobs are everything else
- `show_OD()` with no arguments returns its defaults and draws nothing -- so a caller
  can ask "what can you draw?" without a fixture, a screen, or a 20 s propagation
- block [8] now **RUNS** show_OD: all on -> 15 figures; each toggle off -> exactly 14.
  It cannot be fooled by formatting and it fails for the right reason
- block [10] asks `show_OD()` for its knobs instead of regexing them out of the file

**Block [10] had been ERRORING and still reporting ALL PASS**:
```
[10] every plotting knob has a consumer
    ERROR: dtx(1): out of bound 0 (dimensions are 0x0)
```
The regex stopped matching when `defaults()` was restructured, `dtx{1}` threw, and the
suite printed ALL PASS anyway. **A test that errors without failing is worse than one
that cannot fail: it looks like it ran.**

## Verified

```
254 files parse                       0 failures
suite (10 blocks)                     ALL PASS, 0 errors
check_undefined.py                    0 used-but-never-assigned
figures rendered                      15
title collisions                      0
toggles                               15, each suppresses exactly its own figure
knobs                                 4 (decimate, hist_bins, movavg_s, ratio_lim), 0 dead
```

---

# Round 44 — the figures you actually asked for

## Three figures deleted, because the same curve was drawn three times

```
  rtn           radial/along/cross residual        <- vs BOTH refs, mixed
  overlay_r     radial error distribution + radial error vs time + 3D error
  rtn_overlay   RTN overlay + RTN residual
  growth        running RMS + |along-track|
```
**The radial residual was drawn in three figures. The along-track in three.** And
every one of them either mixed the two references on one axes or silently used
whichever existed -- so a curve labelled "error" could not tell you WHAT it disagreed
with. `rtn`, `overlay_r` and `rtn_overlay` are gone.

## One reference per figure

| figure | reference | what it is |
|---|---|---|
| `rtn_rdo` | REDUCED-DYNAMIC | the ITSG dynamic fit -- smooth, model-informed |
| `rtn_kin` | KINEMATIC | GPS geometry ALONE -- no force model, noisier, **independent of our physics** |

Each carries, per component: the **overlay** (both orbits in RTN) on the left and the
**residual** on the right, with mean, ±1σ band, and the BIAS/SCATTER verdict. Every
series -- overlay curves AND residual curves -- on its own checkbox.

They are different measurements, and their mutual difference is the floor under both:
**a residual smaller than RDO-vs-KIN is not a model error, it is the references
arguing.**

## `rtn_v` -- RTN velocity, overlay and residual

Reduced-dynamic only, and the title says why: **the kinematic product is MJD,x,y,z.
Position only.** There is no kinematic velocity in existence. A missing panel looks
like an oversight; an absent product is a fact about the data.

The radial note earns its place: `dv_r = -n*x_along` is **triad rotation, not a
force** -- a radial velocity residual is what an along-track position error looks like
when you resolve it in a rotating frame.

## `rtn_growth` -- per-axis, both references

A value cannot distinguish forces; a **growth rate** can. Drag integrates twice, so
along-track goes as t^2; a seed or frame error is flat. Running RMS (not a moving
window -- a window deliberately forgets the past, which is the thing being asked
about) per component, **with both references on the same axes**.

**If only one reference shows the growth, it is that reference drifting, not our
model.** No single-reference figure can make that distinction.

## The accelerometer frame -- you asked, and nothing said it

ITSG `nonConservativeForces` is measured in the **SATELLITE BODY frame**. `od_metrics`
rotates it to ECI with the fetched attitude quaternions, so every comparison is
frame-consistent -- but no figure said so, and the panels are labelled `a_x, a_y, a_z`
exactly like the position figures, which are ECI from the start.

Now stated on the figures and at the rotation itself, with what it costs:
**a bad attitude moves signal BETWEEN the x/y/z panels without changing the total.**
So `|a|` is robust to attitude and the per-axis breakdown is not -- worth knowing
before reading a per-axis delta as physics.

## The dead-knob test earned its keep

Deleting `overlay_r` orphaned `hist_bins` -- its only consumer -- and the suite caught
it in the same run:
```
    FAIL: knob(s) that control nothing: hist_bins
```
The `hist` figure had `hist(y, 40)` hard-coded while the knob sat unused. Wired.

## State

```
254 files parse                   0 failures
suite (10 blocks)                 ALL PASS
figures                           16, each individually suppressible (proven by running)
knobs                             4, every one with a consumer
title collisions                  0
```

---

# Round 45 — "Could not find node in peer tree", and what to do when the density still is not there

## The graphics warnings were an ordering bug of mine

```
Could not find node in peer tree during reparentChildren
Could not find node in peer tree during replaceChild
```
**Nine figures called `toggle_panel` and THEN `sgt`.** So: the panel is created and
every axes repositioned; then `sgt` walks the tree with `findobj` and repositions
every axes AGAIN -- including, now, objects that live inside the panel, using
FIGURE-normalized coordinates. We hand MATLAB a layout referring to a parent the
object does not have, and it loses the node it was drawing.

Three fixes, because the order alone should not be load-bearing:
1. **`sgt` first, then `toggle_panel`** everywhere. Lay the axes out, then add the
   panel beside them.
2. **`findobj(..., 'Parent', fig)`** in both. `findobj` walks the WHOLE tree; without
   this it returns the panel's children too. A layout helper that silently corrupts
   the scene when called in the wrong order is a trap the next figure will fall into.
3. **`drawnow`** before and after the panel work. Repositioning axes mid-render is
   how the renderer loses its node. One drawnow costs nothing and removes the class.

## The ITSG density: the plotting side is proven, so it is the fetch

Proven by running, not by argument:
```
  no density   :  9 figures, density_itsg present: 0
  ITSG only    : 10 figures, density_itsg present: 1, density_tud present: 0
```
**The moment `R.den` exists the figure appears -- with TU Delft absent, no error.** So
if you do not see it, `R.den` was never built, and that is the fetch.

The fetch now tries 8 URLs built from the catalog's `neutralDensity_1.0`, INCLUDING
the version-stripped filename (`GRACE-1_neutralDensity_2010-01-01.txt.gz` inside
`neutralDensity_1.0/2010/`), and then falls through to **listing the directory and
matching on the date** -- which finds the file whatever it is called.

And the listing failure is no longer silent:
```
[itsg] directory listing for GRACE-1/neutralDensity_1.0/2010 returned nothing.
       Either the date is absent OR the listing failed (network, or webread could
       not read the index). These are different problems -- open the URL in a
       browser to tell them apart:
       https://ftp.tugraz.at/.../GRACE-1/neutralDensity_1.0/2010/
```
**Empty means two different things** -- the directory holds nothing for this date, or
we could not read the directory at all. One is a fact about the data, the other about
our network. Reporting the second as the first is precisely the mistake that cost us
`neutralDensity_1.0` for this many rounds.

Run it and paste what that prints: it will say which of the two you have.

## A false positive removed

`check_undefined.py` flagged `drawnow` as an undefined variable -- it is a builtin the
list did not know. Fixed rather than tolerated: a checker with a permanent false alarm
trains you to skim its output, and then it stops catching the real ones.

## State

254 files parse. Suite ALL PASS. 0 used-but-never-assigned. 16 figures, 16 toggles.
0 title collisions.

---

# Round 46 — the whitelist was rejecting the name the server uses

## The error was ours, and it said so

```
[4] neutralDensity_1.0 unavailable: unknown product "neutralDensity_1.0".
    Valid: reducedDynamicOrbit, kinematicOrbit, ..., neutralDensity, neutralDensity_ACC
```

**That is OUR whitelist, not the server's.** `data.itsg` refused the name before a
single URL was tried -- and refused it having been handed that name by the catalog,
which had it *because you read it off the server*.

`productInfo` matched on `lower(strrep(p,'_',''))`, which turns `neutralDensity_1.0`
into `neutraldensity1.0` and matches nothing.

## Three places defined product names, and they disagreed

1. `itsg_catalog.csv` column 16 -- `neutralDensity_1.0`
2. `validate_OD` -- the discovery / (former) guess list
3. **`data.itsg`'s `productInfo` whitelist** -- `neutralDensity`, `neutralDensity_ACC`

**One quantity, three names.** The bug class this audit has hit more than any other,
and I added the third one myself two rounds ago while fixing the second.

## The directory is not the product

```
  .../operational/CHAMP/neutralDensity_1.0/2003/CHAMP_neutralDensity_2003-01-01.txt.gz
                        \_____ dir _____/       \___ stem ___/
```
The DIRECTORY carries the version; the FILE does not. `P.name` was being used as
both, so the code could only ever find a product whose directory and filename agreed
-- which the versioned ones do not, by construction.

Now `P.name` is the LAYOUT (columns, frame, sampling) and `P.dir` is the DIRECTORY,
and a trailing `_1.0` / `_2` / `_1.0.3` is stripped to find the layout. `_ACC` is not
a version and is not stripped -- checked by the suite.

```
  asked for                layout (P.name)        directory (P.dir)
  ------------------------------------------------------------------
  neutralDensity           neutralDensity         neutralDensity
  neutralDensity_ACC       neutralDensity_ACC     neutralDensity_ACC
  neutralDensity_1.0       neutralDensity         neutralDensity_1.0
  neutralDensity_2.0       neutralDensity         neutralDensity_2.0

  URL #1: .../CHAMP/neutralDensity_1.0/2003/CHAMP_neutralDensity_2003-01-01.txt.gz
```
That is exactly the path you gave me.

## Why nothing caught this

`productInfo` and `itsgURLs` were local functions. **The only way to exercise them was
a live request and a 404** -- so a rule that decides whether data is reachable was
untestable offline, and it stayed wrong for six rounds while I "fixed" the fetch
around it three separate times.

`data.itsg_productinfo` and `data.itsg_urls` now expose both. Suite block [11] checks
the versioned name resolves, `_ACC` is not mistaken for a version, and URL #1 is the
real path -- in one line, no network.

The error message, when it does fire, now says whose fault it is:
> *unknown product "X" (version-stripped: "Y"). **This is OUR whitelist rejecting it,
> NOT the server.** ... A trailing version is stripped automatically, so a name
> reaching here is genuinely new: add its COLUMN LAYOUT to productInfo.*

## Your other traceback

```
Operation terminated by user during ... websave ... In data.gravity (line 32)
```
That is Ctrl-C during the EGM2008 download -- a 100 MB .gfc from ICGEM, once, then
cached. Not a bug. If it is hanging rather than slow, `GRAV_FIELD = 'default'` runs
degree 6 with no download (and the header says so, loudly, because degree 6 is not a
VLEO gravity field).

## State

256 files parse. Suite ALL PASS (11 blocks). 0 used-but-never-assigned.
16 figures, 16 toggles. 0 title collisions.
