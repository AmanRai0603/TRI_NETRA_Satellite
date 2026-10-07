//! Post-Newtonian (IERS 2010) relativistic accelerations -- port of
//! `matlab_sils/pop/02_forces/+relativity/*` (`schwarzschild`, `lenseThirring`,
//! `deSitter`, `total`) and `02_forces/+forces/relativity.m` ([`force`]): env's method env_relativity, generated from
//! the design into `gen::relativity` (tools/engine_build.py). What is left here is the crate's names for the terms and
//! the inputs.
//!
//! ECI position/velocity [m, m/s]; accelerations [m/s^2].

use crate::ephem::EphemInputs;
use crate::gen::relativity as rel;
use crate::la::V3;

/// Default Earth angular momentum per unit mass `Jvec = [0;0;9.8e8]` [m^2/s] (the design's, `gen::relativity::j_earth`).
pub fn j_earth() -> V3 {
    rel::j_earth()
}

/// `relativity.schwarzschild(rSat, vSat, mu, gamma, beta)` (MATLAB defaults:
/// mu = `mu_earth`, gamma = beta = 1).
pub fn schwarzschild(r: &V3, v: &V3, mu: f64, gamma: f64, beta: f64) -> V3 {
    rel::schwarzschild(*r, *v, mu, gamma, beta)
}

/// `relativity.lenseThirring(rSat, vSat, mu, Jvec, gamma)` (defaults: `Jvec` =
/// [`j_earth`], gamma = 1).
pub fn lense_thirring(r: &V3, v: &V3, mu: f64, jvec: &V3, gamma: f64) -> V3 {
    rel::lense_thirring(*r, *v, mu, *jvec, gamma)
}

/// `relativity.deSitter(rSat, vSat, earthHelioPos, earthHelioVel, gamma)`:
/// geodesic precession; `earth_helio_*` = Earth w.r.t. Sun [m, m/s] (default gamma 1).
pub fn de_sitter(r: &V3, v: &V3, earth_helio_pos: &V3, earth_helio_vel: &V3, gamma: f64) -> V3 {
    rel::de_sitter(*r, *v, *earth_helio_pos, *earth_helio_vel, gamma)
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

impl Term {
    /// The design's choice RelTerm (`gen::relativity::RELTERM_*`).
    pub fn choice(self) -> i64 {
        match self {
            Term::Schwarzschild => rel::RELTERM_SCHWARZSCHILD,
            Term::LenseThirring => rel::RELTERM_LENSETHIRRING,
            Term::DeSitter => rel::RELTERM_DESITTER,
        }
    }
}

/// `[a, parts] = relativity.total(rSat, vSat, E, terms, mu)`. `earth_helio` =
/// (`E.earth_helio_pos`, `E.earth_helio_vel`) or None when `E` is empty. Terms are
/// applied in the order given, at most 8 (Lense-Thirring with the default `Jvec`, gamma = beta = 1): the design's.
pub fn total(r: &V3, v: &V3, earth_helio: Option<(&V3, &V3)>, terms: &[Term], mu: f64) -> Result<(V3, Parts), NoEphem> {
    assert!(terms.len() <= 8, "relativity.total: {} terms, at most 8", terms.len());
    let mut t = [0i64; 8];
    for (k, x) in terms.iter().enumerate() {
        t[k] = x.choice();
    }
    let (hp, hv) = earth_helio.map(|(p, v)| (*p, *v)).unwrap_or(([0.0; 3], [0.0; 3]));
    let (ok, a, p) = rel::rel_total(*r, *v, earth_helio.is_some(), hp, hv, t, terms.len() as i64, mu);
    if !ok {
        return Err(NoEphem);
    }
    let parts = Parts {
        schwarzschild: p.has_schwarzschild.then_some(p.schwarzschild),
        lensethirring: p.has_lensethirring.then_some(p.lensethirring),
        desitter: p.has_desitter.then_some(p.desitter),
    };
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
