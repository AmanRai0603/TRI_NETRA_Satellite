//! Solid-Earth tides: ports of `matlab_sils/pop/02_forces/+solidtides/*`
//! (`iers2010`, `elastic2`, `freqDependent`, `normLegendre`),
//! `02_forces/+tideutil/accelFromDeg2.m` and `02_forces/+forces/solidtides.m`: env's method env_solid_tides,
//! generated from the design into `gen::solidtides` (tools/engine_build.py). What is left here is the crate's names
//! and the MATLAB's defaults (`de440.constants()`: GM and radius of the Earth).
//!
//! The MATLAB acceleration is a central finite difference (h = 1 m) of the
//! degree-2 potential, which amplifies last-bit differences by ~1e6; every
//! expression is therefore kept in the MATLAB evaluation order, with Octave's
//! `norm` ([`crate::gravity::octave_norm`]) and libm `pow/asin/atan2/sin/cos`, so
//! the port reproduces Octave to ~1e-15 rather than to the FD noise level.
//!
//! ctx mapping (`op.accel`): `ctx.r_ecef = C*r_eci`, `ctx.C` = ECI->ECEF matrix
//! from `frames.eci2ecef`, `ctx.E.sun_eci / moon_eci` from `ephemInputs(T.tdb_jd)`
//! (DE440, m), `ctx.grav.mu / Re` from `op.gravLoad`.
//!
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use crate::la::{M3, V3};

/// A `dcs` struct with 5x5 `dC, dS` (normalised coefficient corrections,
/// `dc[n][m]` = MATLAB `dcs.dC(n+1, m+1)`), as returned by `solidtides.*` and
/// `oceantides.mainLines`: the design's record.
pub use crate::gen::solidtides::Dcs5;

impl Dcs5 {
    /// The degree-2 rows `(dC(3,:), dS(3,:))` that `tideutil.accelFromDeg2` reads.
    pub fn deg2(&self) -> ([f64; 3], [f64; 3]) {
        ([self.dc[2][0], self.dc[2][1], self.dc[2][2]], [self.ds[2][0], self.ds[2][1], self.ds[2][2]])
    }
}

/// `(mu, Re)` as the tide functions default them: `de440.constants()`'s GM_earth and Re_earth.
fn defaults(mu: Option<f64>, re: Option<f64>) -> (f64, f64) {
    let k = crate::ephem::constants();
    match mu {
        None => (k.gm_earth, k.re_earth),
        Some(m) => (m, re.unwrap_or(k.re_earth)),
    }
}

/// Port of `solidtides.normLegendre(nmax, x)` with `N = nmax + 1` (at most 5): fully
/// normalised `P[n][m] = Pbar_nm(x)`, lower triangular.
pub fn norm_legendre<const N: usize>(x: f64) -> [[f64; N]; N] {
    assert!((1..=5).contains(&N), "normLegendre: degree {} > 4", N as i64 - 1);
    let q = crate::gen::solidtides::norm_legendre5(x, N as i64 - 1);
    let mut p = [[0.0; N]; N];
    for n in 0..N {
        p[n].copy_from_slice(&q[n][..N]);
    }
    p
}

/// Port of `solidtides.iers2010(rMoon_ecef, rSun_ecef, mu, Re)`: IERS 2010 step 1
/// (anelastic nominal Love numbers, degrees 2 and 3 plus degree 4 from degree 2).
/// `mu = None` uses `GM_earth` and `Re_earth` (MATLAB `nargin<3 || isempty(mu)`);
/// `re = None` with a given `mu` uses `Re_earth` (MATLAB `nargin<4`).
pub fn iers2010(r_moon_ecef: &V3, r_sun_ecef: &V3, mu: Option<f64>, re: Option<f64>) -> Dcs5 {
    let (mu, re) = defaults(mu, re);
    crate::gen::solidtides::iers2010(*r_moon_ecef, *r_sun_ecef, mu, re)
}

/// Port of `solidtides.elastic2(rMoon_ecef, rSun_ecef, k2, mu, Re)`: degree-2
/// elastic tide with one real Love number (`None` -> 0.30, `GM_earth`, `Re_earth`).
pub fn elastic2(r_moon_ecef: &V3, r_sun_ecef: &V3, k2: Option<f64>, mu: Option<f64>, re: Option<f64>) -> Dcs5 {
    let k = crate::ephem::constants();
    crate::gen::solidtides::elastic2(*r_moon_ecef, *r_sun_ecef, k2.unwrap_or(0.30), mu.unwrap_or(k.gm_earth), re.unwrap_or(k.re_earth))
}

/// Port of `solidtides.freqDependent(gmst, dcs0)`: the two representative
/// step-2 lines of the MATLAB (K1 on (2,1) with argument `gmst + pi/2`, and a
/// constant 0.47e-11 on C20). Not used by `forces.solidtides`.
pub fn freq_dependent(gmst: f64, dcs0: &Dcs5) -> Dcs5 {
    crate::gen::solidtides::freq_dependent(gmst, *dcs0)
}

/// Port of `tideutil.accelFromDeg2(rSat_ecef, dcs, mu, Re)`: ECEF acceleration
/// [m/s^2] of the degree-2 potential with `dC(3,:) = dc2`, `dS(3,:) = ds2`, by the
/// MATLAB's central difference with h = 1 m. `mu = None` -> `GM_earth, Re_earth`;
/// `re = None` -> `Re_earth`.
pub fn accel_from_deg2(r_sat_ecef: &V3, dc2: &[f64; 3], ds2: &[f64; 3], mu: Option<f64>, re: Option<f64>) -> V3 {
    let (mu, re) = defaults(mu, re);
    crate::gen::solidtides::accel_from_deg2(*r_sat_ecef, *dc2, *ds2, mu, re)
}

/// Inputs of `forces.solidtides(ctx)`, named after the ctx fields they replace.
#[derive(Clone, Copy, Debug)]
pub struct SolidTideInputs {
    /// `ctx.r_ecef` = `ctx.C * r_eci` [m].
    pub r_ecef: V3,
    /// `ctx.C`, the ECI->ECEF rotation (`r_ecef = C * r_eci`); `ctx.Ct = C'`.
    pub c_eci2ecef: M3,
    /// `ctx.E.sun_eci`, geocentric Sun [m] (ICRF/GCRF).
    pub sun_eci: V3,
    /// `ctx.E.moon_eci`, geocentric Moon [m].
    pub moon_eci: V3,
    /// `ctx.grav.mu` [m^3/s^2].
    pub mu: f64,
    /// `ctx.grav.Re` [m].
    pub re: f64,
}

/// Port of `forces.solidtides(ctx)`: IERS2010 step-1 coefficients from Sun/Moon
/// in ECEF, degree-2 acceleration in ECEF, rotated to ECI with `C'`. [m/s^2]
pub fn solid_tides_accel(inp: &SolidTideInputs) -> V3 {
    crate::gen::solidtides::solid_tides_accel(inp.r_ecef, inp.c_eci2ecef, inp.sun_eci, inp.moon_eci, inp.mu, inp.re)
}
