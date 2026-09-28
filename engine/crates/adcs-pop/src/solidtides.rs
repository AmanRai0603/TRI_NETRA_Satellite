//! Solid-Earth tides: ports of `matlab_sils/pop/02_forces/+solidtides/*`
//! (`iers2010`, `elastic2`, `freqDependent`, `normLegendre`),
//! `02_forces/+tideutil/accelFromDeg2.m` and `02_forces/+forces/solidtides.m`.
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
use crate::gravity::octave_norm;
use crate::la::{mtv, mv, M3, V3};

/// `de440.constants().GM_sun` [m^3/s^2].
pub const GM_SUN: f64 = 1.32712440041279419e20;
/// `de440.constants().GM_moon` [m^3/s^2].
pub const GM_MOON: f64 = 4.902800118e12;
/// `de440.constants().GM_earth` [m^3/s^2] (the tide functions' default `mu`).
pub const GM_EARTH: f64 = 3.98600435507e14;
/// `de440.constants().Re_earth` [m] (WGS84; the tide functions' default `Re`).
pub const RE_EARTH: f64 = 6378137.0;

/// A `dcs` struct with 5x5 `dC, dS` (normalised coefficient corrections,
/// `dc[n][m]` = MATLAB `dcs.dC(n+1, m+1)`), as returned by `solidtides.*` and
/// `oceantides.mainLines`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Dcs5 {
    /// `dcs.dC`.
    pub dc: [[f64; 5]; 5],
    /// `dcs.dS`.
    pub ds: [[f64; 5]; 5],
}

impl Dcs5 {
    /// The degree-2 rows `(dC(3,:), dS(3,:))` that `tideutil.accelFromDeg2` reads.
    pub fn deg2(&self) -> ([f64; 3], [f64; 3]) {
        ([self.dc[2][0], self.dc[2][1], self.dc[2][2]], [self.ds[2][0], self.ds[2][1], self.ds[2][2]])
    }
}

/// Port of `solidtides.normLegendre(nmax, x)` with `N = nmax + 1`: fully
/// normalised `P[n][m] = Pbar_nm(x)`, lower triangular.
pub fn norm_legendre<const N: usize>(x: f64) -> [[f64; N]; N] {
    let nmax = N - 1;
    let mut p = [[0.0; N]; N];
    let s = (1.0 - x * x).sqrt();
    p[0][0] = 1.0;
    if nmax >= 1 {
        p[1][0] = 3f64.sqrt() * x;
        p[1][1] = 3f64.sqrt() * s;
    }
    for n in 2..=nmax {
        let nf = n as f64;
        for m in 0..=n {
            let mf = m as f64;
            p[n][m] = if m == n {
                s * ((2.0 * nf + 1.0) / (2.0 * nf)).sqrt() * p[n - 1][m - 1]
            } else if m == n - 1 {
                x * (2.0 * nf + 1.0).sqrt() * p[n - 1][m]
            } else {
                let a = ((2.0 * nf + 1.0) * (2.0 * nf - 1.0) / ((nf - mf) * (nf + mf))).sqrt();
                let b = ((2.0 * nf + 1.0) * (nf + mf - 1.0) * (nf - mf - 1.0) / ((2.0 * nf - 3.0) * (nf - mf) * (nf + mf))).sqrt();
                a * x * p[n - 1][m] - b * p[n - 2][m]
            };
        }
    }
    p
}

/// `(r, phi, lam)` of a body as the MATLAB forms them (`norm`, `asin`, `atan2`).
#[inline]
fn body_angles(rb: &V3) -> (f64, f64, f64) {
    let r = octave_norm(rb);
    (r, (rb[2] / r).asin(), rb[1].atan2(rb[0]))
}

/// Port of `solidtides.iers2010(rMoon_ecef, rSun_ecef, mu, Re)`: IERS 2010 step 1
/// (anelastic nominal Love numbers, degrees 2 and 3 plus degree 4 from degree 2).
/// `mu = None` uses `GM_EARTH` and `RE_EARTH` (MATLAB `nargin<3 || isempty(mu)`);
/// `re = None` with a given `mu` uses `RE_EARTH` (MATLAB `nargin<4`).
pub fn iers2010(r_moon_ecef: &V3, r_sun_ecef: &V3, mu: Option<f64>, re: Option<f64>) -> Dcs5 {
    let (mu, re) = match mu {
        None => (GM_EARTH, RE_EARTH),
        Some(m) => (m, re.unwrap_or(RE_EARTH)),
    };
    let mut k = [[0.0f64; 4]; 4];
    k[2][0] = 0.29525;
    k[2][1] = 0.29470;
    k[2][2] = 0.29801;
    k[3][0] = 0.093;
    k[3][1] = 0.093;
    k[3][2] = 0.093;
    k[3][3] = 0.094;
    let kp = [-0.00087, -0.00079, -0.00057];
    let mut d = Dcs5::default();
    for body in 0..2 {
        let (rb, gmb) = if body == 0 { (r_moon_ecef, GM_MOON) } else { (r_sun_ecef, GM_SUN) };
        let (r, phi, lam) = body_angles(rb);
        let p = norm_legendre::<5>(phi.sin());
        for n in 2..=3usize {
            for m in 0..=n {
                let fac = k[n][m] / (2 * n + 1) as f64 * (gmb / mu) * (re / r).powf((n + 1) as f64) * p[n][m];
                d.dc[n][m] += fac * (m as f64 * lam).cos();
                d.ds[n][m] += fac * (m as f64 * lam).sin();
            }
        }
        for m in 0..=2usize {
            let fac = kp[m] / 5.0 * (gmb / mu) * (re / r).powf(3.0) * p[2][m];
            d.dc[4][m] += fac * (m as f64 * lam).cos();
            d.ds[4][m] += fac * (m as f64 * lam).sin();
        }
    }
    d
}

/// Port of `solidtides.elastic2(rMoon_ecef, rSun_ecef, k2, mu, Re)`: degree-2
/// elastic tide with one real Love number (`None` -> 0.30, `GM_EARTH`, `RE_EARTH`).
pub fn elastic2(r_moon_ecef: &V3, r_sun_ecef: &V3, k2: Option<f64>, mu: Option<f64>, re: Option<f64>) -> Dcs5 {
    let k2 = k2.unwrap_or(0.30);
    let mu = mu.unwrap_or(GM_EARTH);
    let re = re.unwrap_or(RE_EARTH);
    let gm = [GM_MOON, GM_SUN];
    let bodies = [r_moon_ecef, r_sun_ecef];
    let mut d = Dcs5::default();
    for b in 0..2 {
        let (r, phi, lam) = body_angles(bodies[b]);
        let p = norm_legendre::<3>(phi.sin());
        for m in 0..=2usize {
            let fac = k2 / 5.0 * (gm[b] / mu) * (re / r).powf(3.0) * p[2][m];
            d.dc[2][m] += fac * (m as f64 * lam).cos();
            d.ds[2][m] += fac * (m as f64 * lam).sin();
        }
    }
    d
}

/// Port of `solidtides.freqDependent(gmst, dcs0)`: the two representative
/// step-2 lines of the MATLAB (K1 on (2,1) with argument `gmst + pi/2`, and a
/// constant 0.47e-11 on C20). Not used by `forces.solidtides`.
pub fn freq_dependent(gmst: f64, dcs0: &Dcs5) -> Dcs5 {
    let mut d = *dcs0;
    let th = gmst + std::f64::consts::PI / 2.0;
    d.dc[2][1] += -0.29e-11 * th.sin();
    d.ds[2][1] += -0.29e-11 * th.cos();
    d.dc[2][0] += 0.47e-11;
    d
}

/// Port of `tideutil.accelFromDeg2(rSat_ecef, dcs, mu, Re)`: ECEF acceleration
/// [m/s^2] of the degree-2 potential with `dC(3,:) = dc2`, `dS(3,:) = ds2`, by the
/// MATLAB's central difference with h = 1 m. `mu = None` -> `GM_EARTH, RE_EARTH`;
/// `re = None` -> `RE_EARTH`.
pub fn accel_from_deg2(r_sat_ecef: &V3, dc2: &[f64; 3], ds2: &[f64; 3], mu: Option<f64>, re: Option<f64>) -> V3 {
    let (mu, re) = match mu {
        None => (GM_EARTH, RE_EARTH),
        Some(m) => (m, re.unwrap_or(RE_EARTH)),
    };
    let u2 = |x: &V3| -> f64 {
        let xr = octave_norm(x);
        let phi = (x[2] / xr).asin();
        let lam = x[1].atan2(x[0]);
        let p = norm_legendre::<3>(phi.sin());
        let mut u = 0.0;
        for m in 0..=2usize {
            let ml = m as f64 * lam;
            u += mu / xr * (re / xr).powf(2.0) * p[2][m] * (dc2[m] * ml.cos() + ds2[m] * ml.sin());
        }
        u
    };
    let h = 1.0;
    let mut a = [0.0; 3];
    for k in 0..3 {
        let mut rp = *r_sat_ecef;
        let mut rm = *r_sat_ecef;
        rp[k] += h;
        rm[k] -= h;
        a[k] = (u2(&rp) - u2(&rm)) / (2.0 * h);
    }
    a
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
    let r_sun = mv(&inp.c_eci2ecef, &inp.sun_eci);
    let r_moon = mv(&inp.c_eci2ecef, &inp.moon_eci);
    let d = iers2010(&r_moon, &r_sun, Some(inp.mu), Some(inp.re));
    let (dc2, ds2) = d.deg2();
    let a = accel_from_deg2(&inp.r_ecef, &dc2, &ds2, Some(inp.mu), Some(inp.re));
    mtv(&inp.c_eci2ecef, &a)
}
