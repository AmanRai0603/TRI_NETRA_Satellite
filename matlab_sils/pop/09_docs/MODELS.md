# Models — selection, theory, and references

Every physical effect in the propagator is a *selectable* model. This document
lists what you can choose for each, how to select it, the governing equations
(with short derivations), and the canonical reference so anyone can trace the
math. Standard references used throughout:

- **[MG]** Montenbruck & Gill, *Satellite Orbits*, Springer 2000.
- **[Val]** Vallado, *Fundamentals of Astrodynamics and Applications*, 4th ed.
- **[IERS]** IERS Conventions (2010), Petit & Luzum, TN 36.

---

## 0. Equation of motion (Cowell formulation)

The propagator integrates the full second-order ODE in an inertial (ECI) frame:

    r'' = -μ r/|r|³  +  a_grav(r,t)  +  a_drag  +  a_3body  +  a_SRP  +  a_ERP
                      +  a_relativity  +  a_solidtide  +  a_oceantide

Two-body term explicit, everything else a **perturbing acceleration**. State
y = [r; v], y' = [v; Σa]. This is *Cowell's method*: integrate the total
acceleration directly (as opposed to Encke or variation-of-parameters). It keeps
every oscillatory and secular term — which is exactly why the output must be
compared to an osculating (GPS) reference, not to averaged TLEs (see VALIDATION.md).
Reference: [MG] §3.2.

Forces are summed in `op.accel`; each is toggled by `cfg.forces.<name>.on`.

---

## 1. Gravity

| model | select | use when |
|-------|--------|----------|
| two-body | `forces.gravity.model='twobody'` | analytic checks, high orbits |
| zonal J2–J6 | `='j2'`..`'j6'` | fast, oblateness-dominated LEO |
| full spherical harmonics | `='sphharm'`, `gravityField.field=...`, `.degree`,`.order` | precise LEO |
| Aerospace Toolbox | `='toolbox'` | cross-check against MathWorks |

**Field selection** (row 3): `'default'` (embedded J2–J6, offline), `'EGM2008'`,
`'EIGEN-6C4'`, `'GOCO06s'`, or any ICGEM `.gfc` path. See DATA_SOURCES.md row 7.

**Theory.** The geopotential outside the Earth is a solid-harmonic expansion:

    U(r,φ,λ) = (μ/r) Σ_{n=0}^N Σ_{m=0}^n (R_E/r)^n P̄_nm(sinφ) [C̄_nm cos mλ + S̄_nm sin mλ]

with fully-normalised associated Legendre functions P̄_nm and Stokes coefficients
(C̄_nm, S̄_nm) from the `.gfc` file. Acceleration is a = ∇U, evaluated in the
Earth-fixed frame then rotated to ECI. We use the **Cunningham/Montenbruck–Gill**
recursions for the gradient (pole-safe, avoids the tan φ singularity of the naive
form). The zonal-only shortcut keeps m=0 terms, giving the classic J_n form
(J_n = -C̄_n0 √(2n+1)); e.g. the J2 acceleration

    a_J2 = -(3/2) J2 (μ/r²)(R_E/r)² [ (1-5(z/r)²) x/r ; (1-5(z/r)²) y/r ; (3-5(z/r)²) z/r ]

Validated to 1e-16 against the analytic J2 at all latitudes incl. the pole.
Reference: [MG] §3.2, Cunningham (1970); coefficients ICGEM.

---

## 2. Atmosphere and drag

Two independent choices: the **density model** (atmosphere) and the **aerodynamic
model** (how density becomes force).

### 2a. Density model
| model | select (`forces.drag.atmos=`) | inputs | note |
|-------|------|--------|------|
| exponential | `'exponential'` | altitude only | offline, no space weather |
| NRLMSISE-00 | `'nrlmsise'` | F10.7, F10.7a, ap | multi-species |
| JB2008 | `'jb2008'` | SOLFSMY, DTCFILE | best thermosphere at storm times |
| DTM2020 | `'dtm2020'` | F10.7/F30, Kp | CNES operational/research |

No silent fallback: choosing a real model without its space weather **errors**
with guidance (supply `data.spaceweather(...)` or `cfg.spaceweather.manual`).

### 2b. Aerodynamic model
| model | select (`forces.drag.model=`) | physics |
|-------|------|---------|
| cannonball | `'cannonball'` | isotropic, single Cd·A/m |
| Sentman | `'sentman'` | flat-plate free-molecular, diffuse |
| DRIA | `'dria'` | Sentman + SESAM adsorption (atomic-O) |
| CLL | `'cll'` | Cercignani–Lampis–Lord GSI |

**Theory.** Drag acceleration:

    a_drag = -(1/2) ρ (C_D A / m) |v_rel| v_rel,     v_rel = v - ω_E × r  (co-rotating)

Cannonball takes C_D, A as given. The panel models compute the *directional*
coefficients from gas–surface interaction in the free-molecular regime, where the
incoming flux is a drifting Maxwellian with **speed ratio** s = |v_rel|/√(2kT/m).
Sentman's closed form for a flat plate of normal n̂ at angle θ (cosθ = n̂·v̂_rel):

    C_D, C_L = f(s, θ, T_wall, α_T)     [Sentman 1961; MG §3.5]

DRIA augments this with **SESAM** atomic-oxygen adsorption setting the accommodation
α_T from the local O number density (why atomic-O from the density model matters at
low LEO). When no facet geometry is supplied, a single ram-facing plate of area A_ref
is synthesised so mass/area/Cd-only spacecraft still run. References: [MG] §3.5;
Sentman (1961); Doornbos, *Thermospheric Density and Wind Determination from
Satellite Dynamics* (2011); Walker et al. (DRIA, 2014).

---

## 3. Third-body (Sun, Moon)

| method | select | note |
|--------|--------|------|
| direct | `forces.thirdbody.model='direct'` | plain inverse-square |
| Battin | `='battin'` (default) | cancellation-safe |

**Theory.** A third body of mass parameter μ_b at s perturbs the satellite at r:

    a_3b = μ_b ( (s - r)/|s - r|³  -  s/|s|³ )

The two terms nearly cancel for |r| ≪ |s|; direct subtraction loses precision, so
we use **Battin's F(q) formulation** which reformulates the difference to avoid the
cancellation. Positions of Sun/Moon from the bundled **DE440** ephemeris.
Reference: [MG] §3.3; Battin, *An Introduction to the Mathematics and Methods of
Astrodynamics* (1999).

---

## 4. Solar radiation pressure (SRP)

| model | select | eclipse |
|-------|--------|---------|
| cannonball | `forces.srp.model='cannonball'` | `eclipse='conical'` or `'cylindrical'` |
| box-wing | `='boxwing'` | conical shadow, per-panel |

**Theory.** With solar flux Φ, speed of light c, Sun direction ŝ:

    a_SRP = -ν C_R (A/m) (Φ/c) (AU/|r_sat_sun|)² ŝ

ν ∈ [0,1] is the **shadow function**. Cylindrical model: ν = 0/1 hard shadow.
Conical model: ν is the illuminated fraction of the solar disk from umbra/penumbra
geometry (partial during penumbra). Box-wing sums per-panel specular+diffuse
reflection with the panel normals. Reference: [MG] §3.4; [Val] §8.6.6.

---

## 5. Earth radiation pressure (ERP: albedo + IR)

Select via `forces.erp.on=true`. Reflected shortwave (albedo) + emitted longwave
(IR) from Earth push the satellite. Each surface element of the sunlit/emitting
Earth contributes

    da = (C_R A/m c) (E_i/π) cosθ_i dΩ_i

integrated over the visible cap. **Currently the code uses a constant mean albedo
(~0.3) and IR unless a CERES flux grid is supplied** (DATA_SOURCES.md row 15).
Provide a CERES grid for the spatially/temporally varying field. Reference:
Knocke, Ries & Tapley (1988); [MG] §3.6.

---

## 6. Relativistic corrections

Select terms in `forces.relativity.terms = {'schwarzschild',...}`.
| term | select | size at LEO |
|------|--------|-------------|
| Schwarzschild | `'schwarzschild'` | ~0.1–0.3 m/orbit |
| Lense–Thirring | `'lense'`/`'thirring'` | frame-dragging, small |
| de Sitter | `'desitter'` | geodetic precession, small |

**Theory (Schwarzschild, dominant).** The IERS post-Newtonian point-mass term:

    a = (μ/c²r³) [ (4μ/r - v²) r + 4(r·v) v ]

(with β=γ=1). Lense–Thirring adds the Earth-spin frame-dragging term (∝ J, Earth
angular momentum); de Sitter adds geodetic precession from the Earth's motion
around the Sun. Each sub-term is called with an explicit μ so the code is portable.
Reference: [IERS] §10.3; [MG] §3.7.

---

## 7. Solid Earth and ocean tides

| effect | select | drivers |
|--------|--------|---------|
| solid tide | `forces.solidtides.on=true` | Sun+Moon, Love numbers k_nm |
| ocean tide | `forces.oceantides.on=true` | FES2004 coefficients |

**Theory.** Tidal potentials perturb the Stokes coefficients:

    ΔC̄_nm, ΔS̄_nm = f( k_nm, tide-generating potential )   [IERS §6.2]

Solid tide uses the frequency-dependent Love-number scheme (step 1 + step 2 of the
IERS algorithm). Ocean tide adds the FES2004 harmonic constituents (DATA_SOURCES.md
row 16). These re-enter the gravity acceleration as small time-varying corrections
(sub-metre at LEO). Reference: [IERS] Ch. 6.

---

## 8. Time systems and reference frames

| build | select (`cfg.frame.build=`) | fidelity | needs EOP |
|-------|------|----------|-----------|
| GMST-only | `'gmst'` (default) | ~arcsec | no (offline) |
| CIO A/B/C | `'A'`/`'B'`/`'C'` | full IAU 2006/2000A | yes (auto-cached) |

**Theory.** ECI↔ECEF uses the IAU 2006/2000A **CIO-based** chain:

    [ECEF] = W(t) · R(θ_ERA) · Q(t) · [GCRS]

where Q = precession-nutation (CIO X,Y,s), R = Earth rotation (ERA from UT1), and
W = polar motion. GMST-only collapses Q,W to identity and uses Greenwich sidereal
time — good to arcseconds, adequate for many LEO studies, and fully offline. The
A/B/C builds differ in their tidal/EOP corrections and are validated against Orekit
and IERS test cases. Time scales: UTC→TAI (leap seconds) →TT→ for precession,
UT1=UTC+ΔUT1 for Earth rotation. Reference: [IERS] Ch. 5; [MG] §5.

---

## 9. Numerical integrators

| method | select (`cfg.integrator.method=`) | order | kind |
|--------|------|-------|------|
| RK4 | `'rk4'` | 4 | fixed-step |
| Nyström-4 | `'nystrom4'` | 4 | fixed, 2nd-order ODE |
| RK6 (Luther) | `'rk6luther'` | 6 | fixed |
| Gauss–Jackson 8 | `'gaussJackson8'` | 8 | multistep, 2nd-order ODE |
| RK45 (Dormand–Prince) | `'rk45'` | 5(4) | adaptive |
| RK78 (Dormand–Prince) | `'rk78'` | 8(7) | adaptive (**default/workhorse**) |
| Hermite | `'hermite'` | — | dense output/interp |

**Notes.** Gauss–Jackson is a predictor-corrector tuned for orbit propagation
(directly integrates r'' = a), Kahan-summed, self-started with RK6-Luther — very
low energy drift over many orbits. RK78 gives ~40 accepted steps/orbit at
rtol 1e-11 and is the default. Validated: energy relative drift 7e-12 over 10
orbits. Reference: [MG] §4; Berry & Healy (Gauss–Jackson, 2004).

---

## 10. How selection flows through the code

`cfg.forces.<name>.on/.model` → `op.accel` builds a shared per-evaluation context
(one ECI↔ECEF transform, ephemeris, density) and sums the enabled forces →
`op.propagate` runs the chosen integrator → `sol` with a Hermite `stateAt`
interpolator. Presets bundle common stacks: `config.forcePresets('twobody' |
'j2' | 'leo_precise' | 'leo_full' | 'gnss_meo')`. Start from a preset, then flip
individual models per the tables above.
