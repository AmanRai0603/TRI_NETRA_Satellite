//! Sun/Moon point-mass third-body acceleration -- port of
//! `matlab_sils/pop/02_forces/+thirdbody/*` (`battin`, `direct`, `tidal`,
//! `legendre`, `accel`, `total`, `+secular/*` in [`secular`]) and
//! `02_forces/+forces/thirdbody.m` ([`force`]).
//!
//! All vectors Earth-centred ECI [m]; accelerations [m/s^2].

pub mod secular;

use crate::ephem::EphemInputs;
use crate::la::{dot, norm, V3};
use crate::srp::norm_scaled;

/// Third-body model switch (`model` argument of `thirdbody.accel`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Model {
    /// `'battin'` (default): stable Battin f(q) form.
    #[default]
    Battin,
    /// `'direct'`: textbook difference of two inverse squares.
    Direct,
    /// `'tidal'`: degree-2 gradient approximation.
    Tidal,
    /// `'legendre'`: Legendre series to degree `nmax` (MATLAB default 4).
    Legendre(usize),
}

impl Model {
    /// Parse the MATLAB model name (case-insensitive, `lower(model)`); `'legendre'`
    /// gets `nmax` = 4 as in `thirdbody.accel`/`total`. `None` = MATLAB's
    /// `thirdbody:accel unknown model` error.
    pub fn from_name(name: &str) -> Option<Model> {
        match name.to_ascii_lowercase().as_str() {
            "battin" => Some(Model::Battin),
            "direct" => Some(Model::Direct),
            "tidal" => Some(Model::Tidal),
            "legendre" => Some(Model::Legendre(4)),
            _ => None,
        }
    }
}

/// `thirdbody.battin(rSat, rBody, GM)`: numerically stable third-body acceleration.
pub fn battin(r_sat: &V3, r_body: &V3, gm: f64) -> V3 {
    let (s, b) = (r_sat, r_body);
    let rb2 = dot(b, b);
    let s2b = [s[0] - 2.0 * b[0], s[1] - 2.0 * b[1], s[2] - 2.0 * b[2]];
    let q = dot(s, &s2b) / rb2;
    let f = q * (3.0 + 3.0 * q + q * q) / (1.0 + (1.0 + q).powf(1.5));
    let d3 = rb2 * rb2.sqrt() * (1.0 + q).powf(1.5);
    let k = -gm / d3;
    [k * (s[0] + f * b[0]), k * (s[1] + f * b[1]), k * (s[2] + f * b[2])]
}

/// `thirdbody.direct(rSat, rBody, GM)`: `GM*[(b-s)/|b-s|^3 - b/|b|^3]`.
pub fn direct(r_sat: &V3, r_body: &V3, gm: f64) -> V3 {
    let (s, b) = (r_sat, r_body);
    let d = [b[0] - s[0], b[1] - s[1], b[2] - s[2]];
    // Octave-style scaled norm: the two terms cancel to ~5 digits for the Sun, so
    // the norm's last bit is amplified ~1e5 (see srp::norm_scaled)
    let nd3 = norm_scaled(&d).powf(3.0);
    let nb3 = norm_scaled(b).powf(3.0);
    let mut a = [0.0; 3];
    for i in 0..3 { a[i] = gm * (d[i] / nd3 - b[i] / nb3); }
    a
}

/// `thirdbody.tidal(rSat, rBody, GM)`: `-GM/|b|^3 (s - 3(s.bhat) bhat)`.
pub fn tidal(r_sat: &V3, r_body: &V3, gm: f64) -> V3 {
    let (s, b) = (r_sat, r_body);
    let rb = norm(b);
    let rh = [b[0] / rb, b[1] / rb, b[2] / rb];
    let k = -gm / rb.powf(3.0);
    let sr = 3.0 * dot(s, &rh);
    [k * (s[0] - sr * rh[0]), k * (s[1] - sr * rh[1]), k * (s[2] - sr * rh[2])]
}

/// `thirdbody.legendre(rSat, rBody, GM, nmax)`: Legendre series to degree `nmax`.
pub fn legendre(r_sat: &V3, r_body: &V3, gm: f64, nmax: usize) -> V3 {
    let (s, b) = (r_sat, r_body);
    let rho = norm(s);
    let rb = norm(b);
    let rs = [s[0] / rho, s[1] / rho, s[2] / rho];
    let rbh = [b[0] / rb, b[1] / rb, b[2] / rb];
    let u = dot(&rs, &rbh);
    // P[k] = P_k, dP[k] = P_k' (stack storage up to degree 32)
    let mut pa = [0.0f64; 33];
    let mut dpa = [0.0f64; 33];
    let (mut pv, mut dpv);
    let (p, dp): (&mut [f64], &mut [f64]) = if nmax < 33 {
        (&mut pa[..nmax + 1], &mut dpa[..nmax + 1])
    } else {
        pv = vec![0.0; nmax + 1];
        dpv = vec![0.0; nmax + 1];
        (&mut pv[..], &mut dpv[..])
    };
    p[0] = 1.0;
    if nmax >= 1 { p[1] = u; dp[1] = 1.0; }
    for n in 2..=nmax {
        let nf = n as f64;
        p[n] = ((2.0 * nf - 1.0) * u * p[n - 1] - (nf - 1.0) * p[n - 2]) / nf;
        dp[n] = u * dp[n - 1] + nf * p[n - 1];
    }
    let mut a = [0.0; 3];
    for n in 2..=nmax {
        let nf = n as f64;
        let k = gm / rb.powf(nf + 1.0) * rho.powf(nf - 1.0);
        for i in 0..3 {
            a[i] += k * (nf * p[n] * rs[i] + dp[n] * (rbh[i] - u * rs[i]));
        }
    }
    a
}

/// `thirdbody.accel(rSat, rBody, GM, model, nmax)`: model switch.
pub fn accel(r_sat: &V3, r_body: &V3, gm: f64, model: Model) -> V3 {
    match model {
        Model::Battin => battin(r_sat, r_body, gm),
        Model::Direct => direct(r_sat, r_body, gm),
        Model::Tidal => tidal(r_sat, r_body, gm),
        Model::Legendre(nmax) => legendre(r_sat, r_body, gm, nmax),
    }
}

/// Per-body split returned as `parts` by `thirdbody.total`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Parts {
    /// `parts.sun` [m/s^2].
    pub sun: V3,
    /// `parts.moon` [m/s^2].
    pub moon: V3,
}

/// `[a, parts] = thirdbody.total(rSat, E, model, nmax)`: Sun + Moon, with the
/// `E` fields passed explicitly (`E.sun_eci, E.GM_sun, E.moon_eci, E.GM_moon`).
pub fn total(r_sat: &V3, sun_eci: &V3, gm_sun: f64, moon_eci: &V3, gm_moon: f64, model: Model) -> (V3, Parts) {
    let sun = accel(r_sat, sun_eci, gm_sun, model);
    let moon = accel(r_sat, moon_eci, gm_moon, model);
    ([sun[0] + moon[0], sun[1] + moon[1], sun[2] + moon[2]], Parts { sun, moon })
}

/// Inputs of `forces.thirdbody(ctx)`, with where each comes from in `ctx`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThirdBodyInput {
    /// `ctx.r_eci` satellite position, ECI [m].
    pub r_eci: V3,
    /// `ctx.E.sun_eci` [m].
    pub sun_eci: V3,
    /// `ctx.E.moon_eci` [m].
    pub moon_eci: V3,
    /// `ctx.E.GM_sun` [m^3/s^2].
    pub gm_sun: f64,
    /// `ctx.E.GM_moon` [m^3/s^2].
    pub gm_moon: f64,
    /// `ctx.cfg.forces.thirdbody.model` (default `'battin'`; `'legendre'` uses nmax 4).
    pub model: Model,
}

impl ThirdBodyInput {
    /// Fill from an [`EphemInputs`] (`ctx.E`) and the satellite position.
    pub fn new(r_eci: V3, e: &EphemInputs, model: Model) -> Self {
        ThirdBodyInput { r_eci, sun_eci: e.sun_eci, moon_eci: e.moon_eci, gm_sun: e.gm_sun, gm_moon: e.gm_moon, model }
    }
}

/// `forces.thirdbody(ctx)`: Sun + Moon third-body acceleration, ECI [m/s^2].
pub fn force(inp: &ThirdBodyInput) -> V3 {
    total(&inp.r_eci, &inp.sun_eci, inp.gm_sun, &inp.moon_eci, inp.gm_moon, inp.model).0
}
