//! Solar radiation pressure, eclipse and the box-wing spacecraft geometry -- port
//! of `matlab_sils/pop/02_forces/+srp/*` (`cannonball`, `eclipse`, `boxwing`,
//! `buildBox`, `facet`, `addArray`, `arrayNormal`), `02_forces/+forces/srp.m`
//! ([`force`]) and the example geometry `sgeom.sat16u` ([`sat16u`]).
//!
//! [`Spacecraft`] / [`Facet`] are the ONE geometry the POP feeds to SRP, ERP and
//! the panel drag models (`ctx.sc`).
//!
//! The models (the facets' geometry, the cannonball and box-wing forces, the shadow) are env's
//! method env_srp_force, generated from the design into `gen::srp` (tools/engine_build.py); what is
//! left here is the crate's types (the facets as a vector, the models as enums, the inputs) and the
//! calls.

use crate::ephem::EphemInputs;
use crate::gen::srp as g;
use crate::la::{M3, V3, I3};

/// Facet kind (`f.type`): a fixed body panel or a Sun-tracking solar array.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FacetKind {
    /// `'body'`: fixed normal `n` in body axes.
    Body,
    /// `'array'`: pivots about `axis` to face the Sun ([`array_normal`]).
    Array,
}

/// One flat surface -- the struct built by `srp.facet(type,n,A,alpha,rho_s,rho_d,axis,dbl)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Facet {
    /// `f.type`.
    pub kind: FacetKind,
    /// `f.n` unit outward normal, body frame (unused for arrays).
    pub n: V3,
    /// `f.A` area [m^2].
    pub a: f64,
    /// `f.alpha` absorptivity.
    pub alpha: f64,
    /// `f.rho_s` specular reflectivity.
    pub rho_s: f64,
    /// `f.rho_d` diffuse reflectivity (alpha + rho_s + rho_d = 1).
    pub rho_d: f64,
    /// `f.axis` array pivot axis, body frame.
    pub axis: V3,
    /// `f.double`: both sides can be illuminated.
    pub double: bool,
}

/// `srp.facet(type, n, A, alpha, rho_s, rho_d, axis, dbl)`.
pub fn facet(kind: FacetKind, n: V3, a: f64, alpha: f64, rho_s: f64, rho_d: f64, axis: V3, double: bool) -> Facet {
    Facet { kind, n, a, alpha, rho_s, rho_d, axis, double }
}

/// Optical coefficients struct `opt` (`opt.alpha`, `opt.rho_s`, `opt.rho_d`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Optics {
    /// `alpha`.
    pub alpha: f64,
    /// `rho_s`.
    pub rho_s: f64,
    /// `rho_d`.
    pub rho_d: f64,
}

/// The design's facet set (`gen::srp::ScFacets`, at most 16) of the facets.
pub fn rec(f: &[Facet]) -> g::ScFacets {
    assert!(f.len() <= g::SC_NF as usize, "srp: {} facets, at most {}", f.len(), g::SC_NF);
    let mut r = g::ScFacets::default();
    for x in f {
        let k = match x.kind { FacetKind::Body => g::FACETKIND_BODY, FacetKind::Array => g::FACETKIND_ARRAY };
        r = g::facet_add(r, k, x.n, x.a, x.alpha, x.rho_s, x.rho_d, x.axis, x.double);
    }
    r
}

/// The facets of a design's facet set (its plain panels as body panels).
pub fn from_rec(r: &g::ScFacets) -> Vec<Facet> {
    (0..r.nf as usize)
        .map(|j| {
            let kind = if r.kind[j] == g::FACETKIND_ARRAY { FacetKind::Array } else { FacetKind::Body };
            facet(kind, r.n[j], r.a[j], r.alpha[j], r.rho_s[j], r.rho_d[j], r.axis[j], r.dbl[j] == 1)
        })
        .collect()
}

/// `srp.buildBox(Lx, Ly, Lz, opt)`: the six body facets (+x,-x,+y,-y,+z,-z).
pub fn build_box(lx: f64, ly: f64, lz: f64, opt: &Optics) -> Vec<Facet> {
    from_rec(&g::srp_box(lx, ly, lz, opt.alpha, opt.rho_s, opt.rho_d))
}

/// `F = srp.addArray(F, A, axis, opt)`: append one double-sided solar array.
pub fn add_array(f: &mut Vec<Facet>, a: f64, axis: V3, opt: &Optics) {
    *f = from_rec(&g::srp_add_array(rec(f), a, axis, opt.alpha, opt.rho_s, opt.rho_d));
}

/// `srp.arrayNormal(axis, sHat_b)`: best-lighting normal of an array pivoting about `axis`.
pub fn array_normal(axis: &V3, s_hat_b: &V3) -> V3 {
    g::srp_array_normal(*axis, *s_hat_b)
}

/// The spacecraft struct `sc` / `ctx.sc` as the radiation models read it.
#[derive(Debug, Clone, PartialEq)]
pub struct Spacecraft {
    /// `sc.mass` [kg].
    pub mass: f64,
    /// `sc.Aref` reference (cannonball) area [m^2].
    pub aref: f64,
    /// `sc.Cd` (cannonball drag; `None` = field absent).
    pub cd: Option<f64>,
    /// `sc.Cr` (cannonball SRP/ERP; `None` = absent -> 1.3 in forces.srp/erp).
    pub cr: Option<f64>,
    /// `sc.R_bi` body->ECI DCM (the attitude at this epoch; default eye(3)).
    pub r_bi: M3,
    /// `sc.facets` box-wing geometry (empty = none).
    pub facets: Vec<Facet>,
}

/// Box-wing geometry alias: a [`Spacecraft`] with facets.
pub type BoxWing = Spacecraft;

impl Default for Spacecraft {
    fn default() -> Self {
        Spacecraft { mass: 1.0, aref: 1.0, cd: None, cr: None, r_bi: I3, facets: Vec::new() }
    }
}

/// `sgeom.sat16u()`: 16U low LEO smallsat, 0.20 x 0.20 x 0.34 m bus (long axis body z)
/// plus two double-sided arrays pivoting about body +y. Mass 24 kg; `aref` set to the
/// 0.20 x 0.20 ram face and `cr` 1.3 as TEMPLATE_16U does (not part of sgeom).
pub fn sat16u() -> Spacecraft {
    let (mass, aref, cd, cr, f) = g::sat16u();
    Spacecraft { mass, aref, cd: Some(cd), cr: Some(cr), r_bi: I3, facets: from_rec(&f) }
}

/// `srp.cannonball(P, nu, sunUnit_sat2sun, Cr, AoverM)`: `-Cr P (A/m) nu sHat`.
pub fn cannonball(p: f64, nu: f64, sun_unit_sat2sun: &V3, cr: f64, a_over_m: f64) -> V3 {
    g::srp_cannonball(p, nu, *sun_unit_sat2sun, cr, a_over_m)
}

/// Shadow model switch of `srp.eclipse`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EclipseModel {
    /// `'cylindrical'`: hard on/off, spherical Earth.
    Cylindrical,
    /// `'conical'` (default): umbra + penumbra by solar-disk occultation.
    #[default]
    Conical,
    /// `'fine'`: conical on an oblate Earth plus an opaque atmosphere layer.
    Fine,
}

impl EclipseModel {
    /// The design's choice EclipseModel (`gen::srp::ECLIPSEMODEL_*`).
    pub fn choice(self) -> i64 {
        match self {
            EclipseModel::Cylindrical => g::ECLIPSEMODEL_CYLINDRICAL,
            EclipseModel::Conical => g::ECLIPSEMODEL_CONICAL,
            EclipseModel::Fine => g::ECLIPSEMODEL_FINE,
        }
    }
    /// Parse the MATLAB name (case-insensitive); `None` = `srp:eclipse unknown model`.
    pub fn from_name(name: &str) -> Option<EclipseModel> {
        match name.to_ascii_lowercase().as_str() {
            "cylindrical" => Some(EclipseModel::Cylindrical),
            "conical" => Some(EclipseModel::Conical),
            "fine" => Some(EclipseModel::Fine),
            _ => None,
        }
    }
}

/// `opts` of `srp.eclipse` (fields `Re`, `Rp`, `Rsun`, `hAtm`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EclipseOpts {
    /// `Re` Earth equatorial radius [m] (default `Re_earth`).
    pub re: f64,
    /// `Rp` Earth polar radius [m] (default `Rp_earth`).
    pub rp: f64,
    /// `Rsun` solar radius [m] (default 6.957e8).
    pub rsun: f64,
    /// `hAtm` opaque atmosphere height for `'fine'` [m] (default 12000).
    pub h_atm: f64,
}

impl Default for EclipseOpts {
    fn default() -> Self {
        let (re, rp, rsun, h_atm) = g::eclipse_default();
        EclipseOpts { re, rp, rsun, h_atm }
    }
}

/// `srp.eclipse(rSat, rSun, model, opts)`: lighting fraction nu in [0,1]
/// (1 sunlit, 0 umbra). `rSun` geocentric, both ECI [m].
pub fn eclipse(r_sat: &V3, r_sun: &V3, model: EclipseModel, opts: &EclipseOpts) -> f64 {
    g::srp_eclipse(*r_sat, *r_sun, model.choice(), opts.re, opts.rp, opts.rsun, opts.h_atm)
}

/// Euclidean norm with the scaled accumulation of Octave's `norm` (liboctave
/// `norm_accumulator_2`, LAPACK dnrm2 style). Used only inside the eclipse cone
/// test: the lens-area formula of `srp.eclipse` is ill-conditioned at the
/// penumbra edge (`acos` of a number within ~1e-6 of 1), so a one-ulp difference
/// in |r| moves nu by up to ~1e-9 there. Matching the reference norm keeps the
/// Rust fraction bit-comparable with the Octave-generated references.
pub fn norm_scaled(v: &V3) -> f64 {
    crate::gen::gravity::onorm(*v)
}

/// `[a, F] = srp.boxwing(P, nu, sunUnit_sat2sun, R_b2i, sc)`: box-wing SRP.
/// Returns (acceleration ECI [m/s^2], force ECI [N] before the `nu` scaling).
pub fn boxwing(p: f64, nu: f64, sun_unit_sat2sun: &V3, r_b2i: &M3, facets: &[Facet], mass: f64) -> (V3, V3) {
    g::srp_boxwing(p, nu, *sun_unit_sat2sun, *r_b2i, rec(facets), mass)
}

/// SRP spacecraft model (`cfg.forces.srp.model`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SrpModel {
    /// `'cannonball'` (default).
    #[default]
    Cannonball,
    /// `'boxwing'`.
    Boxwing,
}

impl SrpModel {
    /// The design's choice SrpModel (`gen::srp::SRPMODEL_*`).
    pub fn choice(self) -> i64 {
        match self {
            SrpModel::Cannonball => g::SRPMODEL_CANNONBALL,
            SrpModel::Boxwing => g::SRPMODEL_BOXWING,
        }
    }
    /// Parse the MATLAB name (case-insensitive); `None` = `forces:srp unknown model`.
    pub fn from_name(name: &str) -> Option<SrpModel> {
        match name.to_ascii_lowercase().as_str() {
            "cannonball" => Some(SrpModel::Cannonball),
            "boxwing" => Some(SrpModel::Boxwing),
            _ => None,
        }
    }
}

/// Inputs of `forces.srp(ctx)` and where each comes from in `ctx`.
#[derive(Debug, Clone, Copy)]
pub struct SrpInput<'a> {
    /// `ctx.r_eci` satellite position, ECI [m].
    pub r_eci: V3,
    /// `ctx.E.sun_eci` geocentric Sun, ECI [m].
    pub sun_eci: V3,
    /// `ctx.E.P_srp` solar pressure [N/m^2].
    pub p_srp: f64,
    /// `ctx.sc` (mass, Aref, Cr, R_bi, facets).
    pub sc: &'a Spacecraft,
    /// `ctx.cfg.forces.srp.model` (default cannonball).
    pub model: SrpModel,
    /// `ctx.cfg.forces.srp.eclipse` (default conical; called with default opts).
    pub eclipse: EclipseModel,
    /// `ctx.cfg.forces.srp.Cr` (precedence: this -> `sc.cr` -> 1.3).
    pub cr: Option<f64>,
}

impl<'a> SrpInput<'a> {
    /// Fill from `ctx.E` ([`EphemInputs`]) with default cfg (`cr` = None).
    pub fn new(r_eci: V3, e: &EphemInputs, sc: &'a Spacecraft, model: SrpModel, eclipse: EclipseModel) -> Self {
        SrpInput { r_eci, sun_eci: e.sun_eci, p_srp: e.p_srp, sc, model, eclipse, cr: None }
    }
}

/// `forces.srp(ctx)`: SRP acceleration with eclipse, ECI [m/s^2].
pub fn force(inp: &SrpInput) -> V3 {
    let sc = inp.sc;
    let cr = g::srp_cr(inp.cr.is_some(), inp.cr.unwrap_or(0.0), sc.cr.is_some(), sc.cr.unwrap_or(0.0));
    let f = if inp.model == SrpModel::Boxwing { rec(&sc.facets) } else { g::ScFacets::default() };
    g::srp_force(inp.r_eci, inp.sun_eci, inp.p_srp, inp.model.choice(), inp.eclipse.choice(), cr, sc.aref, sc.mass, sc.r_bi, f)
}
