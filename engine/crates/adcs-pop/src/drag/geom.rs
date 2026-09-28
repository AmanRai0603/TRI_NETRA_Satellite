//! Spacecraft drag geometry and attitude helpers: `02_forces/+dgeom/*` (buildBox,
//! addArray, ramAttitude, attitudeFromAoA, vleo16u), `02_forces/+sgeom/*` (vleo16u,
//! R_lvlh) and the pieces of `+srp` that geometry needs (facet, buildBox, addArray,
//! arrayNormal) -- ported here so drag does not depend on the SRP module.
use crate::atmos::octave::{deg2rad, norm, unit};
use crate::la::{cross, dot, mm, M3, V3};

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

/// `dgeom.buildBox(Lx, Ly, Lz)`: six plain facets, +x ram.
pub fn build_box(lx: f64, ly: f64, lz: f64) -> Vec<Facet> {
    vec![
        Facet::plain([1.0, 0.0, 0.0], ly * lz),
        Facet::plain([-1.0, 0.0, 0.0], ly * lz),
        Facet::plain([0.0, 1.0, 0.0], lx * lz),
        Facet::plain([0.0, -1.0, 0.0], lx * lz),
        Facet::plain([0.0, 0.0, 1.0], lx * ly),
        Facet::plain([0.0, 0.0, -1.0], lx * ly),
    ]
}

/// `dgeom.addArray(facets, normal, area)`: both faces of a flat panel as two facets.
pub fn add_array(facets: &mut Vec<Facet>, normal: V3, area: f64) {
    facets.push(Facet::plain(normal, area));
    facets.push(Facet::plain([-normal[0], -normal[1], -normal[2]], area));
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
    let d: [(V3, f64); 6] = [
        ([1.0, 0.0, 0.0], ly * lz),
        ([-1.0, 0.0, 0.0], ly * lz),
        ([0.0, 1.0, 0.0], lx * lz),
        ([0.0, -1.0, 0.0], lx * lz),
        ([0.0, 0.0, 1.0], lx * ly),
        ([0.0, 0.0, -1.0], lx * ly),
    ];
    d.iter()
        .map(|&(n, a)| Facet { kind: FacetKind::Body, n, a, alpha: o.alpha, rho_s: o.rho_s, rho_d: o.rho_d, axis: [0.0; 3], double: false })
        .collect()
}

/// `srp.addArray(F, A, axis, opt)`: one double-sided tracking array (normal [0,0,0]).
pub fn srp_add_array(f: &mut Vec<Facet>, a: f64, axis: V3, o: Optics) {
    f.push(Facet { kind: FacetKind::Array, n: [0.0; 3], a, alpha: o.alpha, rho_s: o.rho_s, rho_d: o.rho_d, axis, double: true });
}

/// `sgeom.vleo16u()`: the example 16U (0.20 x 0.20 x 0.34 m bus, +x ram, two
/// 0.34 x 0.20 m arrays pivoting about body +Y), mass 24 kg. Returns (mass, facets).
/// `dgeom.vleo16u` (deprecated) returns the same facets with Aref = 0.04 m^2.
pub fn vleo16u() -> (f64, Vec<Facet>) {
    let (lx, ly, lz) = (0.20, 0.20, 0.34);
    let bus = Optics { alpha: 0.30, rho_s: 0.30, rho_d: 0.40 };
    let arr = Optics { alpha: 0.85, rho_s: 0.05, rho_d: 0.10 };
    let mut f = srp_build_box(lx, ly, lz, bus);
    srp_add_array(&mut f, 0.34 * 0.20, [0.0, 1.0, 0.0], arr);
    srp_add_array(&mut f, 0.34 * 0.20, [0.0, 1.0, 0.0], arr);
    (24.0, f)
}

/// `srp.arrayNormal(axis, sHat_b)`: best-lighting normal of an array pivoting about
/// `axis` (body frame), toward the Sun direction `s_hat_b` (body frame).
pub fn array_normal(axis: &V3, s_hat_b: &V3) -> V3 {
    let ax = unit(axis);
    let d = dot(s_hat_b, &ax);
    let mut proj = [s_hat_b[0] - d * ax[0], s_hat_b[1] - d * ax[1], s_hat_b[2] - d * ax[2]];
    if norm(&proj) < 1e-9 {
        let t = if ax[0].abs() > 0.9 { [0.0, 1.0, 0.0] } else { [1.0, 0.0, 0.0] };
        let d = dot(&t, &ax);
        proj = [t[0] - d * ax[0], t[1] - d * ax[1], t[2] - d * ax[2]];
    }
    unit(&proj)
}

fn from_cols(x: V3, y: V3, z: V3) -> M3 {
    [[x[0], y[0], z[0]], [x[1], y[1], z[1]], [x[2], y[2], z[2]]]
}

/// `dgeom.ramAttitude(vrel)`: body->inertial DCM with body +x along `vrel` and body
/// +y = zref x x (zref = +z, or +y when |x.z| > 0.98).
pub fn ram_attitude(vrel: &V3) -> M3 {
    let x = unit(vrel);
    let mut zref = [0.0, 0.0, 1.0];
    if dot(&x, &zref).abs() > 0.98 {
        zref = [0.0, 1.0, 0.0];
    }
    let y = unit(&cross(&zref, &x));
    let z = cross(&x, &y);
    from_cols(x, y, z)
}

/// `dgeom.attitudeFromAoA(vrel, aoa_deg, sideslip_deg)`: ram attitude pitched by the
/// angle of attack about body y, then yawed by the sideslip about body z.
pub fn attitude_from_aoa(vrel: &V3, aoa_deg: f64, sideslip_deg: f64) -> M3 {
    let rram = ram_attitude(vrel);
    let a = deg2rad(aoa_deg);
    let b = deg2rad(sideslip_deg);
    let ry = [[a.cos(), 0.0, a.sin()], [0.0, 1.0, 0.0], [-a.sin(), 0.0, a.cos()]];
    let rz = [[b.cos(), -b.sin(), 0.0], [b.sin(), b.cos(), 0.0], [0.0, 0.0, 1.0]];
    mm(&mm(&rram, &ry), &rz)
}

/// `sgeom.R_lvlh(r, v)`: body->ECI DCM of a nadir-pointing attitude (z = nadir,
/// y = -orbit normal, x completes, ~ +velocity).
pub fn r_lvlh(r: &V3, v: &V3) -> M3 {
    let nr = norm(r);
    let zb = [-r[0] / nr, -r[1] / nr, -r[2] / nr];
    let h = cross(r, v);
    let nh = norm(&h);
    let yb = [-h[0] / nh, -h[1] / nh, -h[2] / nh];
    let xb = unit(&cross(&yb, &zb));
    from_cols(xb, yb, zb)
}
