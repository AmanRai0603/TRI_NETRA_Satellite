//! Post-Newtonian (IERS 2010) relativistic accelerations -- port of
//! `matlab_sils/pop/02_forces/+relativity/*` (`schwarzschild`, `lenseThirring`,
//! `deSitter`, `total`) and `02_forces/+forces/relativity.m` ([`force`]).
//!
//! ECI position/velocity [m, m/s]; accelerations [m/s^2].

use crate::ephem::{constants, EphemInputs};
use crate::la::{cross, dot, norm, V3};

/// Default Earth angular momentum per unit mass `Jvec = [0;0;9.8e8]` [m^2/s].
pub const J_EARTH: V3 = [0.0, 0.0, 9.8e8];

/// `relativity.schwarzschild(rSat, vSat, mu, gamma, beta)` (MATLAB defaults:
/// mu = `mu_earth`, gamma = beta = 1).
pub fn schwarzschild(r: &V3, v: &V3, mu: f64, gamma: f64, beta: f64) -> V3 {
    let c = constants().c;
    let rn = norm(r);
    let k = mu / (c * c * rn.powf(3.0));
    let kr = 2.0 * (beta + gamma) * mu / rn - gamma * dot(v, v);
    let kv = 2.0 * (1.0 + gamma) * dot(r, v);
    [k * (kr * r[0] + kv * v[0]), k * (kr * r[1] + kv * v[1]), k * (kr * r[2] + kv * v[2])]
}

/// `relativity.lenseThirring(rSat, vSat, mu, Jvec, gamma)` (defaults: `Jvec` =
/// [`J_EARTH`], gamma = 1).
pub fn lense_thirring(r: &V3, v: &V3, mu: f64, jvec: &V3, gamma: f64) -> V3 {
    let c = constants().c;
    let rn = norm(r);
    let k = (1.0 + gamma) * mu / (c * c * rn.powf(3.0));
    let rxv = cross(r, v);
    let vxj = cross(v, jvec);
    let q = 3.0 / (rn * rn);
    let rj = dot(r, jvec);
    [k * (q * rxv[0] * rj + vxj[0]), k * (q * rxv[1] * rj + vxj[1]), k * (q * rxv[2] * rj + vxj[2])]
}

/// `relativity.deSitter(rSat, vSat, earthHelioPos, earthHelioVel, gamma)`:
/// geodesic precession; `earth_helio_*` = Earth w.r.t. Sun [m, m/s] (default gamma 1).
pub fn de_sitter(_r: &V3, v: &V3, earth_helio_pos: &V3, earth_helio_vel: &V3, gamma: f64) -> V3 {
    let k = constants();
    let (c, gms) = (k.c, k.gm_sun);
    let re = earth_helio_pos;
    let den = c * c * norm(re).powf(3.0);
    let ae = [-gms * re[0] / den, -gms * re[1] / den, -gms * re[2] / den];
    let w = cross(&cross(earth_helio_vel, &ae), v);
    let g = 1.0 + 2.0 * gamma;
    [g * w[0], g * w[1], g * w[2]]
}

/// A switchable term of `relativity.total` (`terms` cell entries).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Term {
    /// `'schwarzschild'`.
    Schwarzschild,
    /// `'lensethirring'`.
    LenseThirring,
    /// `'desitter'`.
    DeSitter,
}

impl Term {
    /// Parse a MATLAB term name (case-insensitive); `None` = `relativity:total:term`.
    pub fn from_name(name: &str) -> Option<Term> {
        match name.to_ascii_lowercase().as_str() {
            "schwarzschild" => Some(Term::Schwarzschild),
            "lensethirring" => Some(Term::LenseThirring),
            "desitter" => Some(Term::DeSitter),
            _ => None,
        }
    }
}

/// `relativity.total` default term list (all three).
pub const ALL_TERMS: [Term; 3] = [Term::Schwarzschild, Term::LenseThirring, Term::DeSitter];

/// Per-term split returned as `parts` by `relativity.total` (None = term not requested).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Parts {
    /// `parts.schwarzschild`.
    pub schwarzschild: Option<V3>,
    /// `parts.lensethirring`.
    pub lensethirring: Option<V3>,
    /// `parts.desitter`.
    pub desitter: Option<V3>,
}

/// Error of `relativity.total`: de Sitter asked for without the Earth heliocentric
/// state (`relativity:total:noEphem`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoEphem;

impl std::fmt::Display for NoEphem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "relativity:total:noEphem -- the de Sitter term needs the Earth's heliocentric state (E.earth_helio_pos/vel)")
    }
}
impl std::error::Error for NoEphem {}

/// `[a, parts] = relativity.total(rSat, vSat, E, terms, mu)`. `earth_helio` =
/// (`E.earth_helio_pos`, `E.earth_helio_vel`) or None when `E` is empty. Terms are
/// applied in the order given (Lense-Thirring with the default `Jvec`, gamma = beta = 1).
pub fn total(r: &V3, v: &V3, earth_helio: Option<(&V3, &V3)>, terms: &[Term], mu: f64) -> Result<(V3, Parts), NoEphem> {
    let mut a = [0.0; 3];
    let mut parts = Parts::default();
    for t in terms {
        let p = match t {
            Term::Schwarzschild => {
                let p = schwarzschild(r, v, mu, 1.0, 1.0);
                parts.schwarzschild = Some(p);
                p
            }
            Term::LenseThirring => {
                let p = lense_thirring(r, v, mu, &J_EARTH, 1.0);
                parts.lensethirring = Some(p);
                p
            }
            Term::DeSitter => {
                let (pos, vel) = earth_helio.ok_or(NoEphem)?;
                let p = de_sitter(r, v, pos, vel, 1.0);
                parts.desitter = Some(p);
                p
            }
        };
        for i in 0..3 { a[i] += p[i]; }
    }
    Ok((a, parts))
}

/// Inputs of `forces.relativity(ctx)` and where each comes from in `ctx`.
#[derive(Debug, Clone, Copy)]
pub struct RelativityInput<'a> {
    /// `ctx.r_eci` [m].
    pub r_eci: V3,
    /// `ctx.v_eci` [m/s].
    pub v_eci: V3,
    /// (`ctx.E.earth_helio_pos`, `ctx.E.earth_helio_vel`); None when `ctx.E` is empty.
    pub earth_helio: Option<(V3, V3)>,
    /// `ctx.cfg.forces.relativity.terms` (forces default: `{'schwarzschild'}`).
    pub terms: &'a [Term],
    /// `ctx.grav.mu`: the GRAVITY FIELD's mu (not de440's).
    pub mu: f64,
}

impl<'a> RelativityInput<'a> {
    /// Fill from `ctx.E` ([`EphemInputs`]).
    pub fn new(r_eci: V3, v_eci: V3, e: Option<&EphemInputs>, terms: &'a [Term], mu: f64) -> Self {
        RelativityInput { r_eci, v_eci, earth_helio: e.map(|e| (e.earth_helio_pos, e.earth_helio_vel)), terms, mu }
    }
}

/// `forces.relativity` default term list.
pub const FORCE_DEFAULT_TERMS: [Term; 1] = [Term::Schwarzschild];

/// `forces.relativity(ctx)`: post-Newtonian acceleration, ECI [m/s^2].
pub fn force(inp: &RelativityInput) -> Result<V3, NoEphem> {
    let eh = inp.earth_helio.as_ref().map(|(p, v)| (p, v));
    total(&inp.r_eci, &inp.v_eci, eh, inp.terms, inp.mu).map(|x| x.0)
}
