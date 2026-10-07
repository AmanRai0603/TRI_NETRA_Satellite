//! Spacecraft drag geometry and attitude helpers: `02_forces/+dgeom/*` (buildBox,
//! addArray, ramAttitude, attitudeFromAoA, sat16u), `02_forces/+sgeom/*` (sat16u,
//! R_lvlh) and the pieces of `+srp` that geometry needs (facet, buildBox, addArray,
//! arrayNormal) -- ported here so drag does not depend on the SRP module. env's method env_drag_force
//! (the drag panels and the attitudes, `gen::drag`) and env_srp_force (the box-wing facets, `gen::srp`),
//! generated from the design (tools/engine_build.py); what is left here is the facets as a vector.
use crate::gen::{drag as gd, srp as gs};
use crate::la::{M3, V3};

/// Facet kind: the drag model only distinguishes tracking solar arrays.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FacetKind {
    /// a dgeom facet (`.n`, `.A` only)
    Plain,
    /// srp.facet type 'body'
    Body,
    /// srp.facet type 'array': normal from [`array_normal`] each call
    Array,
}

/// One flat facet (body frame). dgeom facets carry only `n`/`a`; srp facets add the
/// type, optical coefficients, pivot axis and double-sidedness.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Facet {
    /// facet kind (`.type`)
    pub kind: FacetKind,
    /// outward normal (body frame); `[0,0,0]` for arrays
    pub n: V3,
    /// area [m^2]
    pub a: f64,
    /// srp optics (absorbed / specular / diffuse); 0 for dgeom facets
    pub alpha: f64,
    /// specular fraction (srp)
    pub rho_s: f64,
    /// diffuse fraction (srp)
    pub rho_d: f64,
    /// array pivot axis (body frame)
    pub axis: V3,
    /// both faces exposed (`.double`)
    pub double: bool,
}

impl Facet {
    /// A dgeom facet `struct('n', n, 'A', a)`.
    pub const fn plain(n: V3, a: f64) -> Facet {
        Facet { kind: FacetKind::Plain, n, a, alpha: 0.0, rho_s: 0.0, rho_d: 0.0, axis: [0.0; 3], double: false }
    }
}

/// The design's facet set (`gen::srp::ScFacets`, at most 16) of the facets.
pub fn rec(f: &[Facet]) -> gs::ScFacets {
    assert!(f.len() <= gs::SC_NF as usize, "drag: {} facets, at most {}", f.len(), gs::SC_NF);
    let mut r = gs::ScFacets::default();
    for x in f {
        let k = match x.kind { FacetKind::Plain => gs::FACETKIND_PLAIN, FacetKind::Body => gs::FACETKIND_BODY, FacetKind::Array => gs::FACETKIND_ARRAY };
        r = gs::facet_add(r, k, x.n, x.a, x.alpha, x.rho_s, x.rho_d, x.axis, x.double);
    }
    r
}

/// The facets of a design's facet set.
pub fn from_rec(r: &gs::ScFacets) -> Vec<Facet> {
    (0..r.nf as usize)
        .map(|j| {
            let kind = match r.kind[j] { gs::FACETKIND_ARRAY => FacetKind::Array, gs::FACETKIND_BODY => FacetKind::Body, _ => FacetKind::Plain };
            Facet { kind, n: r.n[j], a: r.a[j], alpha: r.alpha[j], rho_s: r.rho_s[j], rho_d: r.rho_d[j], axis: r.axis[j], double: r.dbl[j] == 1 }
        })
        .collect()
}

/// `dgeom.buildBox(Lx, Ly, Lz)`: six plain facets, +x ram.
pub fn build_box(lx: f64, ly: f64, lz: f64) -> Vec<Facet> {
    from_rec(&gd::drag_box(lx, ly, lz))
}

/// `dgeom.addArray(facets, normal, area)`: both faces of a flat panel as two facets.
pub fn add_array(facets: &mut Vec<Facet>, normal: V3, area: f64) {
    *facets = from_rec(&gd::drag_add_array(rec(facets), normal, area));
}

/// Optical coefficients of an srp facet (absorbed, specular, diffuse; sum 1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Optics {
    /// absorbed fraction
    pub alpha: f64,
    /// specular fraction
    pub rho_s: f64,
    /// diffuse fraction
    pub rho_d: f64,
}

/// `srp.buildBox(Lx, Ly, Lz, opt)`: six 'body' facets with optics.
pub fn srp_build_box(lx: f64, ly: f64, lz: f64, o: Optics) -> Vec<Facet> {
    from_rec(&gs::srp_box(lx, ly, lz, o.alpha, o.rho_s, o.rho_d))
}

/// `srp.addArray(F, A, axis, opt)`: one double-sided tracking array (normal [0,0,0]).
pub fn srp_add_array(f: &mut Vec<Facet>, a: f64, axis: V3, o: Optics) {
    *f = from_rec(&gs::srp_add_array(rec(f), a, axis, o.alpha, o.rho_s, o.rho_d));
}

/// `sgeom.sat16u()`: the example 16U (0.20 x 0.20 x 0.34 m bus, +x ram, two
/// 0.34 x 0.20 m arrays pivoting about body +Y), mass 24 kg. Returns (mass, facets).
/// `dgeom.sat16u` (deprecated) returns the same facets with Aref = 0.04 m^2.
pub fn sat16u() -> (f64, Vec<Facet>) {
    let (mass, _aref, _cd, _cr, f) = gs::sat16u();
    (mass, from_rec(&f))
}

/// `srp.arrayNormal(axis, sHat_b)`: best-lighting normal of an array pivoting about
/// `axis` (body frame), toward the Sun direction `s_hat_b` (body frame).
pub fn array_normal(axis: &V3, s_hat_b: &V3) -> V3 {
    gd::drag_array_normal(*axis, *s_hat_b)
}

/// `dgeom.ramAttitude(vrel)`: body->inertial DCM with body +x along `vrel` and body
/// +y = zref x x (zref = +z, or +y when |x.z| > 0.98).
pub fn ram_attitude(vrel: &V3) -> M3 {
    gd::ram_attitude(*vrel)
}

/// `dgeom.attitudeFromAoA(vrel, aoa_deg, sideslip_deg)`: ram attitude pitched by the
/// angle of attack about body y, then yawed by the sideslip about body z.
pub fn attitude_from_aoa(vrel: &V3, aoa_deg: f64, sideslip_deg: f64) -> M3 {
    gd::attitude_from_aoa(*vrel, aoa_deg, sideslip_deg)
}

/// `sgeom.R_lvlh(r, v)`: body->ECI DCM of a nadir-pointing attitude (z = nadir,
/// y = -orbit normal, x completes, ~ +velocity).
pub fn r_lvlh(r: &V3, v: &V3) -> M3 {
    gd::r_lvlh(*r, *v)
}
