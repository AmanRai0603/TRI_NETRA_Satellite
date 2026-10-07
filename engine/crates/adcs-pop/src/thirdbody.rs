//! Sun/Moon point-mass third-body acceleration -- port of
//! `matlab_sils/pop/02_forces/+thirdbody/*` (`battin`, `direct`, `tidal`,
//! `legendre`, `accel`, `total`, `+secular/*` in [`secular`]) and
//! `02_forces/+forces/thirdbody.m` ([`force`]).
//!
//! The models are env's method env_third_body, generated from the design into `gen::thirdbody`
//! (tools/engine_build.py); what is left here is the crate's names, the model switch as an
//! enum and the inputs.
//!
//! All vectors Earth-centred ECI [m]; accelerations [m/s^2].

pub mod secular;

use crate::ephem::EphemInputs;
use crate::gen::thirdbody as tb;
use crate::la::V3;

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

impl Model {
    /// The design's choice ThirdBodyModel (`gen::thirdbody::THIRDBODYMODEL_*`) and the Legendre degree.
    fn choice(self) -> (i64, i64) {
        match self {
            Model::Battin => (tb::THIRDBODYMODEL_BATTIN, 0),
            Model::Direct => (tb::THIRDBODYMODEL_DIRECT, 0),
            Model::Tidal => (tb::THIRDBODYMODEL_TIDAL, 0),
            Model::Legendre(n) => {
                assert!(n <= 64, "thirdbody.legendre: degree {n}, at most 64");
                (tb::THIRDBODYMODEL_LEGENDRE, n as i64)
            }
        }
    }
}

/// `thirdbody.battin(rSat, rBody, GM)`: numerically stable third-body acceleration.
pub fn battin(r_sat: &V3, r_body: &V3, gm: f64) -> V3 {
    tb::tb_battin(*r_sat, *r_body, gm)
}

/// `thirdbody.direct(rSat, rBody, GM)`: `GM*[(b-s)/|b-s|^3 - b/|b|^3]` (Octave's scaled norms).
pub fn direct(r_sat: &V3, r_body: &V3, gm: f64) -> V3 {
    tb::tb_direct(*r_sat, *r_body, gm)
}

/// `thirdbody.tidal(rSat, rBody, GM)`: `-GM/|b|^3 (s - 3(s.bhat) bhat)`.
pub fn tidal(r_sat: &V3, r_body: &V3, gm: f64) -> V3 {
    tb::tb_tidal(*r_sat, *r_body, gm)
}

/// `thirdbody.legendre(rSat, rBody, GM, nmax)`: Legendre series to degree `nmax` (at most 64, the design's).
pub fn legendre(r_sat: &V3, r_body: &V3, gm: f64, nmax: usize) -> V3 {
    tb::tb_legendre(*r_sat, *r_body, gm, Model::Legendre(nmax).choice().1)
}

/// `thirdbody.accel(rSat, rBody, GM, model, nmax)`: model switch.
pub fn accel(r_sat: &V3, r_body: &V3, gm: f64, model: Model) -> V3 {
    let (m, n) = model.choice();
    tb::tb_accel(*r_sat, *r_body, gm, m, n)
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
    let (m, n) = model.choice();
    let (a, sun, moon) = tb::tb_total(*r_sat, *sun_eci, gm_sun, *moon_eci, gm_moon, m, n);
    (a, Parts { sun, moon })
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
