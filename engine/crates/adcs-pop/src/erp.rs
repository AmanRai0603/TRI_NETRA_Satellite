//! Earth radiation pressure (albedo + IR) -- port of
//! `matlab_sils/pop/02_forces/+erp/*` (`zonalCoeffs`, `knocke`, `simple`, `ceres`,
//! `boxwing`, `accel`) and `02_forces/+forces/erp.m` ([`force`]).
//!
//! `knocke`/`simple`/`ceres` are cannonball models (one `CrAoM` number);
//! `boxwing` applies each Earth element's beam to the facets of the
//! [`Spacecraft`](crate::srp::Spacecraft) geometry shared with SRP and drag.

use crate::ephem::{constants, EphemInputs};
use crate::la::{cross, dot, mtv, mv, norm, M3, V3};
use crate::srp::{facet_sum, Facet, Spacecraft};

/// `[alb, emi] = erp.zonalCoeffs(lat, doy)`: Knocke (1988) zonal albedo and
/// emissivity at geocentric latitude `lat` [rad] and day of year `doy`.
pub fn zonal_coeffs(lat: f64, doy: f64) -> (f64, f64) {
    let (a0, a1, a2) = (0.34, 0.10, 0.29);
    let (e0, e1, e2) = (0.68, -0.07, -0.18);
    let w = 2.0 * std::f64::consts::PI / 365.25;
    let t0 = 0.0;
    let s = lat.sin();
    let p1 = s;
    let p2 = 0.5 * (3.0 * s * s - 1.0);
    let ann = (w * (doy - t0)).cos();
    (a0 + a1 * ann * p1 + a2 * p2, e0 + e1 * ann * p1 + e2 * p2)
}

/// Short-wave (albedo) / long-wave (IR) split returned as `comp` [m/s^2].
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Comp {
    /// `comp.sw` albedo part.
    pub sw: V3,
    /// `comp.lw` IR part.
    pub lw: V3,
}

/// One visible-cap element of the Knocke ring integration.
struct Element {
    es: V3,
    /// cos_e * dA / (pi rho^2)
    geo: f64,
    /// cos_e, dA, rho kept for the cannonball `base` product order
    cos_e: f64,
    da: f64,
    rho: f64,
    msw: f64,
    mlw: f64,
}

/// Iterate the Knocke cap elements (shared loop of knocke/ceres/boxwing), calling
/// `f` for every element with `cos_e > 0`. `coeffs(lat, n_el)` gives (alb, emi).
fn cap_loop<C, F>(r_sat: &V3, r_sun: &V3, nrings: usize, nseg: usize, mut coeffs: C, mut f: F)
where
    C: FnMut(f64, &V3) -> (f64, f64),
    F: FnMut(&Element),
{
    use std::f64::consts::PI;
    let k = constants();
    let (re, s) = (k.re_earth, k.tsi);
    let d = norm(r_sat);
    let zhat = [r_sat[0] / d, r_sat[1] / d, r_sat[2] / d];
    let ns = norm(r_sun);
    let shat = [r_sun[0] / ns, r_sun[1] / ns, r_sun[2] / ns];
    let rho_max = (re / d).min(1.0).acos();
    let t = if zhat[0].abs() > 0.9 { [0.0, 1.0, 0.0] } else { [1.0, 0.0, 0.0] };
    let e1 = cross(&zhat, &t);
    let n1 = norm(&e1);
    let e1 = [e1[0] / n1, e1[1] / n1, e1[2] / n1];
    let e2 = cross(&zhat, &e1);
    let (nr, nsg) = (nrings as f64, nseg as f64);
    for ir in 1..=nrings {
        let psi = (ir as f64 - 0.5) / nr * rho_max;
        let dpsi = rho_max / nr;
        let (sp, cp) = psi.sin_cos();
        for js in 1..=nseg {
            let az = 2.0 * PI * (js as f64 - 0.5) / nsg;
            let (sa, ca) = az.sin_cos();
            let mut n_el = [0.0; 3];
            for i in 0..3 { n_el[i] = cp * zhat[i] + sp * (ca * e1[i] + sa * e2[i]); }
            let r_el = [re * n_el[0], re * n_el[1], re * n_el[2]];
            let sv = [r_sat[0] - r_el[0], r_sat[1] - r_el[1], r_sat[2] - r_el[2]];
            let rho = norm(&sv);
            let es = [sv[0] / rho, sv[1] / rho, sv[2] / rho];
            let cos_e = dot(&n_el, &es);
            if cos_e <= 0.0 { continue; }
            let da = re * re * sp * dpsi * (2.0 * PI / nsg);
            #[allow(clippy::manual_clamp)] // MATLAB max(min(x,1),-1) semantics
            let lat = n_el[2].min(1.0).max(-1.0).asin();
            let (alb, emi) = coeffs(lat, &n_el);
            let cz = dot(&n_el, &shat);
            let msw = if cz > 0.0 { alb * s * cz } else { 0.0 };
            let mlw = emi * s / 4.0;
            let geo = cos_e * da / (PI * rho * rho);
            f(&Element { es, geo, cos_e, da, rho, msw, mlw });
        }
    }
}

/// Cannonball accumulation `base = CrAoM/c*cos_e*dA/(pi*rho*rho)`.
fn cannon_sum(r_sat: &V3, r_sun: &V3, cr_aom: f64, nrings: usize, nseg: usize, coeffs: impl FnMut(f64, &V3) -> (f64, f64)) -> (V3, Comp) {
    use std::f64::consts::PI;
    let c = constants().c;
    let mut asw = [0.0; 3];
    let mut alw = [0.0; 3];
    cap_loop(r_sat, r_sun, nrings, nseg, coeffs, |el| {
        let base = cr_aom / c * el.cos_e * el.da / (PI * el.rho * el.rho);
        let (ks, kl) = (base * el.msw, base * el.mlw);
        for i in 0..3 {
            asw[i] += ks * el.es[i];
            alw[i] += kl * el.es[i];
        }
    });
    ([asw[0] + alw[0], asw[1] + alw[1], asw[2] + alw[2]], Comp { sw: asw, lw: alw })
}

/// `[a, comp] = erp.knocke(rSat, rSun, CrAoM, doy, nrings, nseg)`: Knocke ring
/// model (MATLAB defaults doy 80, 16 rings x 48 segments).
pub fn knocke(r_sat: &V3, r_sun: &V3, cr_aom: f64, doy: f64, nrings: usize, nseg: usize) -> (V3, Comp) {
    cannon_sum(r_sat, r_sun, cr_aom, nrings, nseg, |lat, _| zonal_coeffs(lat, doy))
}

/// `[a, comp] = erp.simple(rSat, rSun, CrAoM, doy)`: whole visible Earth as one
/// radial source (sizing only).
pub fn simple(r_sat: &V3, r_sun: &V3, cr_aom: f64, doy: f64) -> (V3, Comp) {
    let k = constants();
    let (re, s, c) = (k.re_earth, k.tsi, k.c);
    let d = norm(r_sat);
    let zhat = [r_sat[0] / d, r_sat[1] / d, r_sat[2] / d];
    let (alb, emi) = zonal_coeffs(0.0, doy);
    let q = re / d;
    let f = q * q;
    let ns = norm(r_sun);
    let cz = dot(&zhat, &[r_sun[0] / ns, r_sun[1] / ns, r_sun[2] / ns]).max(0.0);
    let esw = alb * s * cz * f;
    let elw = emi * (s / 4.0) * f;
    let (ks, kl) = (cr_aom / c * esw, cr_aom / c * elw);
    let sw = [ks * zhat[0], ks * zhat[1], ks * zhat[2]];
    let lw = [kl * zhat[0], kl * zhat[1], kl * zhat[2]];
    ([sw[0] + lw[0], sw[1] + lw[1], sw[2] + lw[2]], Comp { sw, lw })
}

/// Albedo/emissivity grid `gridFcn(lat, lon) -> [albedo, emissivity]` of `erp.ceres`.
pub type GridFn<'a> = &'a dyn Fn(f64, f64) -> (f64, f64);

/// `[a, comp] = erp.ceres(rSat, rSun, CrAoM, gridFcn, nrings, nseg)`: ring model with
/// a user albedo/emissivity grid; `grid = None` falls back to Knocke zonal at doy 80.
pub fn ceres(r_sat: &V3, r_sun: &V3, cr_aom: f64, grid: Option<GridFn>, nrings: usize, nseg: usize) -> (V3, Comp) {
    cannon_sum(r_sat, r_sun, cr_aom, nrings, nseg, |lat, n_el| match grid {
        None => zonal_coeffs(lat, 80.0),
        Some(g) => g(lat, n_el[1].atan2(n_el[0])),
    })
}

/// `[a, comp] = erp.boxwing(rSat, rSun, R_b2i, sc, doy, nrings, nseg)`: ERP on the
/// box-wing facets; each element's beam arrives from `-es` (`uHat_b = R_b2i' (-es)`)
/// and is applied per band through the SRP facet response.
pub fn boxwing(r_sat: &V3, r_sun: &V3, r_b2i: &M3, facets: &[Facet], mass: f64, doy: f64, nrings: usize, nseg: usize) -> (V3, Comp) {
    let c = constants().c;
    let mut fsw = [0.0; 3];
    let mut flw = [0.0; 3];
    cap_loop(r_sat, r_sun, nrings, nseg, |lat, _| zonal_coeffs(lat, doy), |el| {
        let esw = el.msw * el.geo;
        let elw = el.mlw * el.geo;
        if esw <= 0.0 && elw <= 0.0 { return; }
        let u_b = mtv(r_b2i, &[-el.es[0], -el.es[1], -el.es[2]]);
        let s = mv(r_b2i, &facet_sum(facets, &u_b, esw / c));
        let l = mv(r_b2i, &facet_sum(facets, &u_b, elw / c));
        for i in 0..3 {
            fsw[i] += s[i];
            flw[i] += l[i];
        }
    });
    let sw = [fsw[0] / mass, fsw[1] / mass, fsw[2] / mass];
    let lw = [flw[0] / mass, flw[1] / mass, flw[2] / mass];
    ([sw[0] + lw[0], sw[1] + lw[1], sw[2] + lw[2]], Comp { sw, lw })
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

/// Default Knocke ring count of `erp.knocke/ceres/boxwing`.
pub const NRINGS: usize = 16;
/// Default Knocke segment count of `erp.knocke/ceres/boxwing`.
pub const NSEG: usize = 48;

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
    let cr_aom = inp.cr_aom.unwrap_or_else(|| inp.cr.or(sc.cr).unwrap_or(1.3) * sc.aref / sc.mass);
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
