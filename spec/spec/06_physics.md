
---

## 6. Units, physics and reference data

### 6.1 Units and quantities to add

`adcs-units` is VLEO's `vleo-units` plus the following. Each one goes in the `units!` table or the `QUANTITIES` registry. `tests/the_registry_matches_the_types.rs` then keeps the two in step, as it does in VLEO. `plan/units.toml` lists the whole registry, VLEO's and these, with the quantities each unit may state: it is what the node form's pick-lists and the intake checker read in this package (O02, O03), and in the built repository `adcs-intake` reads `adcs-units` itself.

| Add | Kind | SI factor | Why |
|---|---|---|---|
| `MomentOfInertia` | quantity (kg·m²) | — | VLEO declared inertia as `Ratio`/`One`; a unit is a type, and inertia is not a ratio |
| `DynamicViscosity` | quantity (Pa·s) | — | ring fluid |
| `AngularAcceleration` | quantity (rad/s²) | — | slew and wheel dynamics |
| `AngleRandomWalk` | quantity (rad/√s) | — | gyro noise |
| `RateRandomWalk` | quantity (rad/s/√s) | — | gyro bias drift |
| `KilogramSquareMetre` | unit | 1 | inertia |
| `PascalSecond`, `MillipascalSecond` | unit | 1, 1e-3 | viscosity |
| `Millimetre` | unit | 1e-3 | ring bore |
| `Microtesla`, `Nanotesla` | unit | 1e-6, 1e-9 | field, magnetometer noise |
| `MicronewtonMetre`, `MillinewtonMetre` | unit | 1e-6, 1e-3 | disturbance and actuator torque |
| `MillinewtonMetreSecond` | unit | 1e-3 | ring and wheel momentum |
| `Milliwatt` | unit | 1e-3 | holding power |
| `Kilopascal` | unit | 1e3 | pump pressure |
| `Litre` | unit of `Volume` | 1e-3 | volume allocated to the ADCS, and its volume requirement, in the case CSV |
| `DegreePerHour` | unit | 4.848136811095360e-6 | gyro bias |
| `DegreePerSqrtHour` | unit | 2.908882086657216e-4 | gyro angle random walk |
| `DegreePerHourSqrtHour` | unit of `RateRandomWalk` | 8.080228018492267e-8 | gyro rate random walk |
| `RadianPerSecond2` | unit of `AngularAcceleration` | 1 | slew and wheel dynamics |
| `IndianRupee` | unit of `Money` | 1 | layer 1 prices; the currency convention is decision D13 |
| `RiskLevel` | quantity and unit (L) | 1 | the risk rows of layer 1 (§5.13): 0 none open, 1 negligible to 5 would stop delivery. Ordinal: `risk::` compares and takes maxima of it, and nothing adds it |

Attitude types go in `adcs-units/src/frames.rs` beside VLEO's `Body`, `Eci`, `Ecef` and `Lvlh`. `Quat<To, From>` is scalar-first, unit-norm, and typed by its frames, so composing `Quat<Body, Eci>` with `Quat<Eci, Ecef>` compiles and composing it with `Quat<Body, Ecef>` does not. `Dcm<To, From>` works the same way. Every rotation uses `pmath`. These types are shared by `adcs-core` and `adcs-sim-core`, and the Mars Climate Orbiter argument that made units types applies equally to frames.

### 6.2 `adcs-core::physics`

Every relation a sheet or the plant uses lives here (rule 3). Keep `gnc.rs` and `mission.rs` from VLEO, and write the rest fresh in VLEO's style: `no_std`, typed arguments, `pmath` only, a doc comment giving the relation and its source id. Every function that takes a count returns zero at count zero (§5.6).

| Module | Functions (the seed row that calls each) | Source |
|---|---|---|
| `orbit` | `radius(h)` (m2_4) · `circular_period(r)` (m2_5) · `mean_motion(t_orb)` (m2_6) · `circular_speed(r)` (m3_4) · `eclipse_fraction(r, beta)` · `beta_angle(inc, ltan, epoch)` | vallado2013 |
| `env` | `max_magnetic_latitude(i)` (m3_2) · `dipole_field_equator(r)` (m3_0) · `dipole_field_max(r, lambda_max)` (m3_1) · `density_at(h, t_epoch)` (m3_3) · `solar_pressure(t_epoch)` (m3_5) · `sun_distance_au(t_epoch)` | wertz1978, igrf14, nrlmsis2, kopp2011 |
| `gnc` (kept) | VLEO's functions, signatures unchanged: `magnetic_torque(residual_dipole, field)` (gd_3) · `total_disturbance_torque(aerodynamic, gravity_gradient, solar, magnetic)` (gd_4) · `magnetorquer_dipole_required(momentum, field, dump_time)` (gm_2) · `pointing_error_rss(terms: &[Angle])` (gp_5, called with one slice) | wertz1978, smad2011 |
| `gnc` (added) | `gravity_gradient_torque_worst(r, i_max, i_min)` (gd_0; typed inertias, θ = 45°) · `aero_torque(rho, v, c_d, a_fr, c_pa)` (gd_1) · `solar_pressure_torque_at(p_srp, a_sun, q, c_ps)` (gd_2; normal incidence) · `secular_momentum_per_orbit(tau_d, t_orb)` (gd_5). VLEO's `gravity_gradient_torque` and `solar_pressure_torque` stay as they are; their signatures do not fit these rows. | wertz1978, smad2011 |
| `mtq` | `dipole_to_reject(tau_d, b_min)` (gm_1) · `torque_authority(m_av, b_min, n_mtq)` (gm_3) · `coil_dipole(n_turns, current, area)` | smad2011, idmas_v2 |
| `rw` | `bang_bang_peak_momentum(i_max, theta, t_slew)` (gw_3) · `bang_bang_peak_torque(i_max, theta, t_slew)` (gw_4) · `cyclic_momentum_quarter_orbit(tau, t_orb)` | idmas_v2, smad2011 |
| `fmr` | `ring_momentum(d, s, rho, v)` (gf_6) · `spin_down_time(d, rho, mu)` (gf_7) · `holding_power_laminar(h, mu, l, rho, d, s)` (gf_8) · `pump_pressure_for_torque(tau, l, s, d)` (gf_9) · `reynolds(rho, v, d, mu)` · `conduction_pump_pressure(n, i, b, h)` | idmas_v2 |
| `rcs` | `propellant_per_slew(h, r, isp)` (gr_3) · `propellant_per_year(m_p, n_day, n_rcs)` (gr_4; zero when `n_rcs` is zero) | idmas_v2 |
| `ctl` | `settling_time_2pct(w_n, zeta)` (gc_2) | ogata2010 |
| `risk` | `highest_level(levels)` (rk4_0; a slice, the highest open level over the areas) · `net_closed(closed, opened)` (rk4_1) · `share_tested(untested, total)` (rk4_2; zero beliefs recorded is `Undefined`, not 1) | ecss_m_st_80c, adcs_derisk_method |
| `mission` (kept) | `Sense`, `Closure`, `closure(req, ach, sense)`, exactly as in VLEO | — |

`plan/physics.toml` lists every function here with its arguments and the rows that call it; `tools/validate_plan.py` checks that the file and this table name the same functions. That file is the package's registry. From P1 the registry is `adcs-core::physics` itself: `cargo xtask docs` writes a generated table of its public functions, their arguments and their source ids, which `adcs-intake` checks steps against and the form exporter puts in the node form's pick-list. A new function the implementation agent writes therefore appears in the next release's forms with nothing else to edit, and in the twin map's physics family, so the `twin` job asks for its MATLAB twin in the same pull request (§10.8.7). Argument names are the calling rows' binding names, and argument order is their input order. A node form's step names one of these functions, and the implementation agent writes that step's HOLE as exactly `physics::<module>::<fn>(<bindings in order>)`; `intake verify` checks the call.

A kept VLEO function is never changed. Where a node needs something different, its form describes a new function, as the four `gnc` additions above were, and the implementation agent writes it here, in this style, with property tests and its own review.

### 6.3 Reference-data bundles

Each is a VLEO bundle: `bundles/<name>/<version>/` holding a `manifest.toml` (name, version, provenance, `licence_until`, `stale_after_days`, files, `content_hash`), published with `xtask bundle publish` and verified at sync.

| Bundle | Contents | Built by | Read by |
|---|---|---|---|
| `igrf14` | the IGRF-14 Gauss coefficients file from NOAA NCEI (`igrf14coeffs.txt`), unmodified | a person downloads it; `xtask bundle publish` hashes it | `env::*` (the degree-1 terms), `adcs-sim-core` field model (full degree 13) |
| `atmos-density` | a table of log density against altitude, 150–2000 km, at low, mean and high solar activity | `tools/atmos_table.py`, running NRLMSIS 2.0 through the `pymsis` package (ADOPTION.lock) | `env::density_at`, the plant's drag model |
| `catalogue` | every part, product, algorithm and class file and `families.toml`, as published | `xtask bundle publish catalogue/` | solver, loop engine, FSW config generator |

**How bundle data reaches a physics function.** Never as a hole argument. VLEO compiles its measured solar data into `physics/env.rs` as constants, marked "MEASURED DATA, not a published relation". A node's `[data] bundles` makes a run refuse when that bundle is missing from `Case.data`. The ADCS kernel does the same, generated rather than typed:

- `adcs-core/build.rs` reads `bundles/igrf14/<version>/` and `bundles/atmos-density/<version>/` at build time, and emits `const` tables: the IGRF-14 degree-1 terms, and the density table;
- the build records each bundle's content hash as a constant in the kernel;
- at run time, a node that lists the bundle refuses with `DataUnverified` (F8) when the store's verified hash differs from the compiled one, and with `DataMissing` when the store lacks the bundle.

So the kernel stays `no_std` with no files, the data is reviewed as a bundle, and the numbers a run used are the numbers the kernel hash names. The dipole strength is taken at the IGRF-14 model epoch 2025.0, which is an assumption stated on `m3_0`.

The IGRF coefficients file is fetched by a person and recorded with its URL and date in the manifest's `provenance`. The builder does not fetch data at build time; the network rule of VLEO's DELIVERY_PLAN holds, so a campaign makes zero network calls.

The field model in `adcs-sim-core` is a spherical-harmonic synthesis to degree 13 with Schmidt semi-normalised Legendre functions, in `pmath`. Its fixtures come from NOAA's own IGRF calculator, recorded by a person (provenance `independent-tool`), never from this code.
