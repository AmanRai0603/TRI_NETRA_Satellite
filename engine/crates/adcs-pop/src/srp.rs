//! Solar radiation pressure, eclipse and the box-wing spacecraft geometry -- port
//! of `matlab_sils/pop/02_forces/+srp/*` (`cannonball`, `eclipse`, `boxwing`,
//! `buildBox`, `facet`, `addArray`, `arrayNormal`), `02_forces/+forces/srp.m`
//! ([`force`]) and the example geometry `sgeom.sat16u` ([`sat16u`]).
//!
//! [`Spacecraft`] / [`Facet`] are the ONE geometry the POP feeds to SRP, ERP and
//! the panel drag models (`ctx.sc`).

use crate::ephem::{constants, EphemInputs};
use crate::la::{dot, mtv, mv, norm, M3, V3, I3};

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

/// `srp.buildBox(Lx, Ly, Lz, opt)`: the six body facets (+x,-x,+y,-y,+z,-z).
pub fn build_box(lx: f64, ly: f64, lz: f64, opt: &Optics) -> Vec<Facet> {
    let d: [(V3, f64); 6] = [
        ([1.0, 0.0, 0.0], ly * lz), ([-1.0, 0.0, 0.0], ly * lz),
        ([0.0, 1.0, 0.0], lx * lz), ([0.0, -1.0, 0.0], lx * lz),
        ([0.0, 0.0, 1.0], lx * ly), ([0.0, 0.0, -1.0], lx * ly),
    ];
    d.iter()
        .map(|&(n, a)| facet(FacetKind::Body, n, a, opt.alpha, opt.rho_s, opt.rho_d, [0.0; 3], false))
        .collect()
}

/// `F = srp.addArray(F, A, axis, opt)`: append one double-sided solar array.
pub fn add_array(f: &mut Vec<Facet>, a: f64, axis: V3, opt: &Optics) {
    f.push(facet(FacetKind::Array, [0.0; 3], a, opt.alpha, opt.rho_s, opt.rho_d, axis, true));
}

/// `srp.arrayNormal(axis, sHat_b)`: best-lighting normal of an array pivoting about `axis`.
pub fn array_normal(axis: &V3, s_hat_b: &V3) -> V3 {
    let na = norm(axis);
    let ax = [axis[0] / na, axis[1] / na, axis[2] / na];
    let sa = dot(s_hat_b, &ax);
    let mut proj = [s_hat_b[0] - sa * ax[0], s_hat_b[1] - sa * ax[1], s_hat_b[2] - sa * ax[2]];
    if norm(&proj) < 1e-9 {
        let t = if ax[0].abs() > 0.9 { [0.0, 1.0, 0.0] } else { [1.0, 0.0, 0.0] };
        let ta = dot(&t, &ax);
        proj = [t[0] - ta * ax[0], t[1] - ta * ax[1], t[2] - ta * ax[2]];
    }
    let np = norm(&proj);
    [proj[0] / np, proj[1] / np, proj[2] / np]
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
    let bus = Optics { alpha: 0.30, rho_s: 0.30, rho_d: 0.40 };
    let arr = Optics { alpha: 0.85, rho_s: 0.05, rho_d: 0.10 };
    let mut f = build_box(0.20, 0.20, 0.34, &bus);
    add_array(&mut f, 0.34 * 0.20, [0.0, 1.0, 0.0], &arr);
    add_array(&mut f, 0.34 * 0.20, [0.0, 1.0, 0.0], &arr);
    Spacecraft { mass: 24.0, aref: 0.20 * 0.20, cd: Some(2.2), cr: Some(1.3), r_bi: I3, facets: f }
}

/// `srp.cannonball(P, nu, sunUnit_sat2sun, Cr, AoverM)`: `-Cr P (A/m) nu sHat`.
pub fn cannonball(p: f64, nu: f64, sun_unit_sat2sun: &V3, cr: f64, a_over_m: f64) -> V3 {
    let k = -cr * p * a_over_m * nu;
    [k * sun_unit_sat2sun[0], k * sun_unit_sat2sun[1], k * sun_unit_sat2sun[2]]
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
        let k = constants();
        EclipseOpts { re: k.re_earth, rp: k.rp_earth, rsun: 6.957e8, h_atm: 12000.0 }
    }
}

/// `srp.eclipse(rSat, rSun, model, opts)`: lighting fraction nu in [0,1]
/// (1 sunlit, 0 umbra). `rSun` geocentric, both ECI [m].
pub fn eclipse(r_sat: &V3, r_sun: &V3, model: EclipseModel, opts: &EclipseOpts) -> f64 {
    match model {
        EclipseModel::Cylindrical => {
            let d = [r_sun[0] - r_sat[0], r_sun[1] - r_sat[1], r_sun[2] - r_sat[2]];
            let nd = norm(&d);
            let s = [d[0] / nd, d[1] / nd, d[2] / nd];
            let m = [-r_sat[0], -r_sat[1], -r_sat[2]];
            let ms = dot(&m, &s);
            if ms < 0.0 { return 1.0; }
            let perp = norm(&[m[0] - ms * s[0], m[1] - ms * s[1], m[2] - ms * s[2]]);
            if perp >= opts.re { 1.0 } else { 0.0 }
        }
        EclipseModel::Conical => frac_conical(r_sat, r_sun, opts.re, opts.rsun),
        EclipseModel::Fine => {
            let sz = opts.re / opts.rp;
            let rs = [r_sat[0], r_sat[1], r_sat[2] * sz];
            let rn = [r_sun[0], r_sun[1], r_sun[2] * sz];
            frac_conical(&rs, &rn, opts.re + opts.h_atm, opts.rsun)
        }
    }
}

/// Euclidean norm with the scaled accumulation of Octave's `norm` (liboctave
/// `norm_accumulator_2`, LAPACK dnrm2 style). Used only inside the eclipse cone
/// test: the lens-area formula of `srp.eclipse` is ill-conditioned at the
/// penumbra edge (`acos` of a number within ~1e-6 of 1), so a one-ulp difference
/// in |r| moves nu by up to ~1e-9 there. Matching the reference norm keeps the
/// Rust fraction bit-comparable with the Octave-generated references.
pub fn norm_scaled(v: &V3) -> f64 {
    let (mut scl, mut sum) = (0.0f64, 1.0f64);
    for &x in v {
        let t = x.abs();
        if scl == t {
            sum += 1.0;
        } else if scl < t {
            let q = scl / t;
            sum *= q * q;
            sum += 1.0;
            scl = t;
        } else if t != 0.0 {
            let q = t / scl;
            sum += q * q;
        }
    }
    scl * sum.sqrt()
}

fn frac_conical(r_sat: &V3, r_sun: &V3, re: f64, rsun: f64) -> f64 {
    let d = [r_sun[0] - r_sat[0], r_sun[1] - r_sat[1], r_sun[2] - r_sat[2]];
    let ds = norm_scaled(&d);
    let s = [d[0] / ds, d[1] / ds, d[2] / ds];
    let rr = norm_scaled(r_sat);
    let e = [-r_sat[0] / rr, -r_sat[1] / rr, -r_sat[2] / rr];
    let se = dot(&s, &e);
    if se < 0.0 { return 1.0; }
    let th_s = (rsun / ds).min(1.0).asin();
    let th_e = (re / rr).min(1.0).asin();
    #[allow(clippy::manual_clamp)] // MATLAB max(min(x,1),-1): NaN -> 1, not NaN
    let th_sep = se.min(1.0).max(-1.0).acos();
    if th_sep >= th_s + th_e { return 1.0; }
    if th_e - th_s >= th_sep { return 0.0; }
    if th_s - th_e >= th_sep { let q = th_e / th_s; return 1.0 - q * q; }
    let occ = lens_area(th_sep, th_s, th_e);
    1.0 - occ / (std::f64::consts::PI * (th_s * th_s))
}

fn lens_area(d: f64, r1: f64, r2: f64) -> f64 {
    use std::f64::consts::PI;
    if d >= r1 + r2 { return 0.0; }
    if d <= (r1 - r2).abs() { let m = r1.min(r2); return PI * (m * m); }
    let a = (d * d + r1 * r1 - r2 * r2) / (2.0 * d);
    let h = (r1 * r1 - a * a).max(0.0).sqrt();
    r1 * r1 * (a / r1).acos() + r2 * r2 * ((d - a) / r2).acos() - d * h
}

/// Body-frame radiation force of one beam of pressure `p` arriving from direction
/// `u_b` (unit, spacecraft -> source, body axes) on all `facets`: the facet loop
/// of `srp.boxwing` (array: normal from [`array_normal`], flipped when double-sided).
pub(crate) fn facet_sum(facets: &[Facet], u_b: &V3, p: f64) -> V3 {
    let mut fb = [0.0; 3];
    for f in facets {
        let mut n = match f.kind {
            FacetKind::Array => array_normal(&f.axis, u_b),
            FacetKind::Body => f.n,
        };
        let mut cth = dot(&n, u_b);
        if f.double && cth < 0.0 {
            n = [-n[0], -n[1], -n[2]];
            cth = -cth;
        }
        if cth <= 0.0 { continue; }
        let k = p * f.a * cth;
        let ka = f.alpha + f.rho_d;
        let kn = 2.0 * (f.rho_s * cth + f.rho_d / 3.0);
        for i in 0..3 { fb[i] -= k * (ka * u_b[i] + kn * n[i]); }
    }
    fb
}

/// `[a, F] = srp.boxwing(P, nu, sunUnit_sat2sun, R_b2i, sc)`: box-wing SRP.
/// Returns (acceleration ECI [m/s^2], force ECI [N] before the `nu` scaling).
pub fn boxwing(p: f64, nu: f64, sun_unit_sat2sun: &V3, r_b2i: &M3, facets: &[Facet], mass: f64) -> (V3, V3) {
    let s_b = mtv(r_b2i, sun_unit_sat2sun);
    let fb = facet_sum(facets, &s_b, p);
    let f = mv(r_b2i, &fb);
    ([nu * f[0] / mass, nu * f[1] / mass, nu * f[2] / mass], f)
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
    let r = &inp.r_eci;
    let s = &inp.sun_eci;
    let d = [s[0] - r[0], s[1] - r[1], s[2] - r[2]];
    let nd = norm(&d);
    let shat = [d[0] / nd, d[1] / nd, d[2] / nd];
    let nu = eclipse(r, s, inp.eclipse, &EclipseOpts::default());
    match inp.model {
        SrpModel::Cannonball => {
            let cr = inp.cr.or(inp.sc.cr).unwrap_or(1.3);
            cannonball(inp.p_srp, nu, &shat, cr, inp.sc.aref / inp.sc.mass)
        }
        SrpModel::Boxwing => boxwing(inp.p_srp, nu, &shat, &inp.sc.r_bi, &inp.sc.facets, inp.sc.mass).0,
    }
}
