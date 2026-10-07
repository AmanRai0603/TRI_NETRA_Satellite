//! Earth radiation pressure (albedo + IR) -- port of
//! `matlab_sils/pop/02_forces/+erp/*` (`zonalCoeffs`, `knocke`, `simple`, `ceres`,
//! `boxwing`, `accel`) and `02_forces/+forces/erp.m` ([`force`]).
//!
//! `knocke`/`simple`/`ceres` are cannonball models (one `CrAoM` number);
//! `boxwing` applies each Earth element's beam to the facets of the
//! [`Spacecraft`](crate::srp::Spacecraft) geometry shared with SRP and drag.
//!
//! The models are env's method env_erp_force, generated from the design into `gen::erp`
//! (tools/engine_build.py); what is left here is the crate's names, the model switch as an enum,
//! the inputs, and the ring loop over a user's albedo grid (a function the run hands in) on the
//! design's cap elements.

use crate::ephem::EphemInputs;
use crate::gen::erp as g;
use crate::la::{M3, V3};
use crate::srp::{Facet, Spacecraft};

/// `[alb, emi] = erp.zonalCoeffs(lat, doy)`: Knocke (1988) zonal albedo and
/// emissivity at geocentric latitude `lat` [rad] and day of year `doy`.
pub fn zonal_coeffs(lat: f64, doy: f64) -> (f64, f64) {
    g::zonal_coeffs(lat, doy)
}

/// Short-wave (albedo) / long-wave (IR) split returned as `comp` [m/s^2] (`sw`, `lw`): the design's record.
pub use crate::gen::erp::ErpComp as Comp;

/// `[a, comp] = erp.knocke(rSat, rSun, CrAoM, doy, nrings, nseg)`: Knocke ring
/// model (MATLAB defaults doy 80, 16 rings x 48 segments).
pub fn knocke(r_sat: &V3, r_sun: &V3, cr_aom: f64, doy: f64, nrings: usize, nseg: usize) -> (V3, Comp) {
    g::erp_knocke(*r_sat, *r_sun, cr_aom, doy, nrings as i64, nseg as i64)
}

/// `[a, comp] = erp.simple(rSat, rSun, CrAoM, doy)`: whole visible Earth as one
/// radial source (sizing only).
pub fn simple(r_sat: &V3, r_sun: &V3, cr_aom: f64, doy: f64) -> (V3, Comp) {
    g::erp_simple(*r_sat, *r_sun, cr_aom, doy)
}

/// Albedo/emissivity grid `gridFcn(lat, lon) -> [albedo, emissivity]` of `erp.ceres`.
pub type GridFn<'a> = &'a dyn Fn(f64, f64) -> (f64, f64);

/// `[a, comp] = erp.ceres(rSat, rSun, CrAoM, gridFcn, nrings, nseg)`: ring model with
/// a user albedo/emissivity grid; `grid = None` falls back to Knocke zonal at doy 80.
pub fn ceres(r_sat: &V3, r_sun: &V3, cr_aom: f64, grid: Option<GridFn>, nrings: usize, nseg: usize) -> (V3, Comp) {
    let Some(grid) = grid else { return knocke(r_sat, r_sun, cr_aom, 80.0, nrings, nseg) };
    let (zhat, e1, e2, shat, rho_max) = g::cap_frame(*r_sat, *r_sun);
    let mut sw = [0.0; 3];
    let mut lw = [0.0; 3];
    for ir in 1..=nrings as i64 {
        for js in 1..=nseg as i64 {
            let (vis, n_el, es, cos_e, da, rho, lat) = g::cap_element(*r_sat, zhat, e1, e2, rho_max, ir, js, nrings as i64, nseg as i64);
            if !vis { continue; }
            let (alb, emi) = grid(lat, n_el[1].atan2(n_el[0]));
            let (msw, mlw, _geo) = g::cap_exitance(alb, emi, n_el, shat, cos_e, da, rho);
            let (ds, dl) = g::cap_cannon(cr_aom, cos_e, da, rho, es, msw, mlw);
            for i in 0..3 {
                sw[i] += ds[i];
                lw[i] += dl[i];
            }
        }
    }
    ([sw[0] + lw[0], sw[1] + lw[1], sw[2] + lw[2]], Comp { sw, lw })
}

/// `[a, comp] = erp.boxwing(rSat, rSun, R_b2i, sc, doy, nrings, nseg)`: ERP on the
/// box-wing facets; each element's beam arrives from `-es` (`uHat_b = R_b2i' (-es)`)
/// and is applied per band through the SRP facet response.
pub fn boxwing(r_sat: &V3, r_sun: &V3, r_b2i: &M3, facets: &[Facet], mass: f64, doy: f64, nrings: usize, nseg: usize) -> (V3, Comp) {
    g::erp_boxwing(*r_sat, *r_sun, *r_b2i, crate::srp::rec(facets), mass, doy, nrings as i64, nseg as i64)
}

/// ERP model and its model-specific arguments (`model` + `varargin` of `erp.accel`).
#[derive(Clone, Copy)]
pub enum Model<'a> {
    /// `'knocke'` (default): cannonball ring model.
    Knocke,
    /// `'simple'`: radial one-source approximation.
    Simple,
    /// `'ceres'` with its optional `gridFcn`.
    Ceres(Option<GridFn<'a>>),
    /// `'boxwing'`: facets + attitude (CrAoM not used).
    Boxwing,
}

impl<'a> std::fmt::Debug for Model<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Model::Knocke => write!(f, "Knocke"),
            Model::Simple => write!(f, "Simple"),
            Model::Ceres(g) => write!(f, "Ceres(grid: {})", g.is_some()),
            Model::Boxwing => write!(f, "Boxwing"),
        }
    }
}

impl<'a> Model<'a> {
    /// Parse the MATLAB name (case-insensitive; `ceres` without a grid);
    /// `None` = `forces:erp:model unknown model`.
    pub fn from_name(name: &str) -> Option<Model<'a>> {
        match name.to_ascii_lowercase().as_str() {
            "knocke" => Some(Model::Knocke),
            "simple" => Some(Model::Simple),
            "ceres" => Some(Model::Ceres(None)),
            "boxwing" => Some(Model::Boxwing),
            _ => None,
        }
    }
}

/// Inputs of `forces.erp(ctx)` and where each comes from in `ctx`.
#[derive(Clone, Copy, Debug)]
pub struct ErpInput<'a> {
    /// `ctx.r_eci` satellite position, ECI [m].
    pub r_eci: V3,
    /// `ctx.E.sun_eci` geocentric Sun, ECI [m].
    pub sun_eci: V3,
    /// `ctx.T.doy` day of year (1..366, integral-valued double).
    pub doy: f64,
    /// `ctx.sc` (mass, Aref, Cr, R_bi, facets).
    pub sc: &'a Spacecraft,
    /// `ctx.cfg.forces.erp.model` (default knocke; `gridFcn` inside `Ceres`).
    pub model: Model<'a>,
    /// `ctx.cfg.forces.erp.CrAoM` (overrides everything below).
    pub cr_aom: Option<f64>,
    /// `ctx.cfg.forces.erp.Cr` (precedence: CrAoM -> this -> `sc.cr` -> 1.3).
    pub cr: Option<f64>,
    /// `ctx.cfg.forces.erp.nrings` (boxwing only; default 16).
    pub nrings: Option<usize>,
    /// `ctx.cfg.forces.erp.nseg` (boxwing only; default 48).
    pub nseg: Option<usize>,
}

impl<'a> ErpInput<'a> {
    /// Fill from `ctx.E` ([`EphemInputs`]) with default cfg (all options None).
    pub fn new(r_eci: V3, e: &EphemInputs, doy: f64, sc: &'a Spacecraft, model: Model<'a>) -> Self {
        ErpInput { r_eci, sun_eci: e.sun_eci, doy, sc, model, cr_aom: None, cr: None, nrings: None, nseg: None }
    }
}

/// Default Knocke ring count of `erp.knocke/ceres/boxwing` (the design's).
pub const NRINGS: usize = g::ERP_NRINGS as usize;
/// Default Knocke segment count of `erp.knocke/ceres/boxwing` (the design's).
pub const NSEG: usize = g::ERP_NSEG as usize;

/// `[a, comp] = erp.accel(rSat, rSun, CrAoM, model, ...)` with the MATLAB
/// defaults for the optional arguments: knocke/simple use `doy`; ceres uses its
/// grid; boxwing uses `r_b2i`, `sc` facets/mass, `doy`, `nrings`, `nseg`.
pub fn accel(r_sat: &V3, r_sun: &V3, cr_aom: f64, model: Model, r_b2i: &M3, sc: &Spacecraft, doy: f64, nrings: usize, nseg: usize) -> (V3, Comp) {
    match model {
        Model::Knocke => knocke(r_sat, r_sun, cr_aom, doy, NRINGS, NSEG),
        Model::Simple => simple(r_sat, r_sun, cr_aom, doy),
        Model::Ceres(g) => ceres(r_sat, r_sun, cr_aom, g, NRINGS, NSEG),
        Model::Boxwing => boxwing(r_sat, r_sun, r_b2i, &sc.facets, sc.mass, doy, nrings, nseg),
    }
}

/// `forces.erp(ctx)`: ERP acceleration, ECI [m/s^2]. Panics (as MATLAB's
/// `erp:boxwing:noFacets` error) if `Boxwing` is asked of a spacecraft without facets.
pub fn force(inp: &ErpInput) -> V3 {
    let sc = inp.sc;
    let cr_aom = g::erp_cr_aom(inp.cr_aom.is_some(), inp.cr_aom.unwrap_or(0.0), inp.cr.is_some(), inp.cr.unwrap_or(0.0), sc.cr.is_some(),
                               sc.cr.unwrap_or(0.0), sc.aref, sc.mass);
    match inp.model {
        Model::Boxwing => {
            assert!(!sc.facets.is_empty(), "erp:boxwing:noFacets -- erp.boxwing needs sc.facets; use knocke");
            boxwing(&inp.r_eci, &inp.sun_eci, &sc.r_bi, &sc.facets, sc.mass, inp.doy,
                    inp.nrings.unwrap_or(NRINGS), inp.nseg.unwrap_or(NSEG)).0
        }
        Model::Knocke => knocke(&inp.r_eci, &inp.sun_eci, cr_aom, inp.doy, NRINGS, NSEG).0,
        Model::Simple => simple(&inp.r_eci, &inp.sun_eci, cr_aom, inp.doy).0,
        Model::Ceres(g) => ceres(&inp.r_eci, &inp.sun_eci, cr_aom, g, NRINGS, NSEG).0,
    }
}
