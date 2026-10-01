# Sizing laws (node `size`, `engine/crates/adcs-design/src/lib.rs`)

**Owner: Agastya.** What each actuator's size comes from, with every constant, margin and default the
code uses. A constant marked **no source stated** is a working value the code carries without a
reference; it is listed here so it can be confirmed or replaced, not because it is endorsed.
The MATLAB twin's `+asils/+sizing` holds the same laws for one survey; the season and solar-activity sweep (B2.3) is the engine's only.

## 1. Demand survey (`demand`)

One orbit of the case's own orbit, 10 s steps, the engine's environment and torque models (gravity
gradient, aerodynamic, solar pressure, residual dipole), at four attitudes: +X, +Y, +Z boresight to
nadir, and Sun-pointing (−Z to the Sun). The survey is flown at four seasons (the case epoch and
+91.3, +182.6, +273.9 days: the Sun's direction against the orbit plane, so the beta angle and the
eclipses) and at the long-term low and high solar activity (F10.7 = F10.7a = 65 and 250 sfu,
ECSS-E-ST-10-04C), eight surveys in all; every quantity below is the worst of them, each attitude
and axis on its own, and the eight are recorded as `survey_sweep`. At the shipped cases' 550 km the
season moves the peak torque by about 25 % and the solar activity by under 2 %.

| Quantity | Law | Notes |
|---|---|---|
| peak disturbance `tau_dist` | the largest ‖τ‖ over the orbit, worst attitude | |
| cyclic momentum `h_cyclic` | max over the orbit of ‖h(t) − (t/T)·h(T)‖ | the secular trend removed |
| secular momentum per orbit `h_secular` | ‖h(T)‖ | |
| design disturbance momentum `h_dist` | max over attitudes of h_cyclic + n·h_secular, n = `req.dump`·3600/T orbits held between dumps | blank `req.dump`: **0.25 orbit taken** (no source stated), recorded in the notes |
| detumble momentum `h_detumble` | J_max · `mission.w0` | blank `mission.w0`: **10 °/s taken**, recorded in the notes |
| reference slew | bang-bang over `mission.sangle` in `req.slew`: ω = 2θ/T, α = 2πθ/T² | blank: **30° in 60 s taken**, recorded |
| momentum margin `k_h` | 1/(1 − `req.hsat`) when 0 < hsat < 1, else **2** | the factor 2: **no source stated** |
| torque margin `k_tau` | the knob (the converge node raises it) | |
| need | h_req = k_h·max(h_dist, h_slew); τ_req = k_tau·max(τ_dist, τ_slew) | |
| life | `mission.life`; blank: **3 years taken**, recorded | |

## 2. Magnetorquers (`mtq`, our product)

The dipole is the largest of three needs, never under 0.05 A m²:

| Need | Law | Margin |
|---|---|---|
| dumping the peak disturbance | m = 2·τ_dist / (0.5·B_min) | ×2, and only half the weakest field usable: **no source stated** |
| removing the secular momentum each orbit | m = 2·h_secular / (0.3·B_mean·T) | ×2, 30 % average effectiveness: **no source stated** |
| detumble in half the allowed time | m = h_detumble / (0.3·B_mean·0.5·t_det) | t_det = `req.detumble`, blank: 3 orbits |
| coils-only pointing coil (`MTQP`) | max(m, 2·m_dump) | |

Mass, power and volume scale linearly with the dipole from the SYN-CT-1 anchor (0.45 A m²:
0.03 kg, 0.3 W, 0.012 L); 4.5 A m² per A, 30 Ω, 5 ms time constant.

## 3. Wheels, CMG, VSCMG (benchmarks, `rotor`)

Bought, not sized: the lightest selectable catalogue model (`matlab_sils/data/catalogue`,
`docs/CATALOGUE.md`) that holds h_req and gives τ_req per unit, ties broken by steady power, then
volume. Three orthogonal wheels carry the full need each; a four-unit pyramid unit half of it (two
units act on any axis). When no model meets the need the largest is fitted and the gap recorded.

## 4. Fluid momentum loop (`fmr`, our product)

Each axis is a galinstan ring around 80 % of the face perimeter, designed together with its DC
conduction pump (electromagnet, no permanent magnet) by `empump::design`: least mass + λ × steady
power for momentum max(h_req, 2e-4 N m s) and torque max(τ_req, 1e-5 N m). λ [kg/W] is a knob;
`empump::pareto` records the trade.

## 5. Cold-gas RCS (`rcs`, our product)

| Quantity | Law | Notes |
|---|---|---|
| lever arms | 0.45 × the box side across which the couple acts | |
| thrust | F = max_i J_i·α / (2·arm_i), α = max(τ_req/J_max, h_detumble/(600 s·J_max)), rounded up to a class (5, 10, 20, 50, 100 mN) | the 600 s detumble: **no source stated** |
| total impulse | detumble 2·h_detumble/arm + dumping h_secular·orbits/arm over the life | arm: harmonic mean of the three |
| propellant | 1.2 × impulse / (Isp·g0), at least 0.01 kg | ×1.2 margin: **no source stated** |
| Isp | **60 s** budget; the part states 70 s nominal, 60–80 s dispersed | the low end of the dispersion, so the budget holds for every draw |
| slews | **left out** of the propellant; reported as `propellant_if_slews_on_rcs_kg` | slews are flown on the momentum devices |
| tank | 1.25 × propellant volume (N2O 745 kg/m³), Al-7075 sphere, 70 bar MEOP, allowable stress 250 MPa, wall ≥ 0.5 mm | 250 MPa is about half the alloy's T6 yield; the factor is **not stated** in the code |
| dry mass | tank + 12 valves × 10 g + 50 g | |

## 6. Products and budgets (`size_all`)

One product per family from the sized and the bought parts; each family's ADCS mass, steady power
and volume is the sum over its fill. The box is **0.34 × 0.10 × 0.10 m** (3U) for every case today
(B2.7 takes it from the case's class). Every family carries the same sensor suite (B2.5); the gyro
grade and the star tracker are the converge node's knobs.

## 7. Selection

`select` takes the lightest feasible solution family, then the lower steady power, then the smaller
volume (`matlab_sils/data/pipeline/nodes.json`, `select.rank_feasible`). SPEC §8.6 ranks by worst
margin first; that is owed (`docs/UPGRADE_PLAN.md`, B2.8).
