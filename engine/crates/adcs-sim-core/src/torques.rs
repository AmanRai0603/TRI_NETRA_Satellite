//! Environment torques from the orbit state (asils.env.{geometry,torques}):
//! gravity gradient, aerodynamic and solar pressure per box facet, residual dipole.
use crate::la::*;
use crate::pm::*;

#[derive(Clone, Copy, Debug)]
pub struct Facets { pub n: [V3; 6], pub a: [f64; 6], pub rho: [V3; 6], pub sigma_n: f64, pub sigma_t: f64, pub vb_ratio: f64, pub rho_spec: f64, pub rho_diff: f64 }

impl Facets {
    /// A box of edges box_m [m] with the centre of mass offset cm [m].
    pub fn boxed(box_m: &V3, cm: &V3, sigma_n: f64, sigma_t: f64, vb_ratio: f64, refl: f64, spec_frac: f64) -> Facets {
        let (lx, ly, lz) = (box_m[0], box_m[1], box_m[2]);
        let n = [[1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, -1.0, 0.0], [0.0, 0.0, 1.0], [0.0, 0.0, -1.0]];
        let a = [ly*lz, ly*lz, lx*lz, lx*lz, lx*ly, lx*ly];
        let mut rho = [[0.0; 3]; 6];
        for j in 0..6 { for k in 0..3 { rho[j][k] = n[j][k]*box_m[k]/2.0 - cm[k]; } }
        let _ = (lx, ly, lz);
        Facets { n, a, rho, sigma_n, sigma_t, vb_ratio, rho_spec: refl*spec_frac, rho_diff: refl*(1.0 - spec_frac) }
    }
}

/// The Earth's mean Bond albedo and its mean emitted (outgoing long-wave) flux [W/m^2]: the
/// annual global means of the Earth's radiation budget (Kiehl & Trenberth 1997, BAMS 78:197:
/// albedo 0.31, 235 W/m^2; Knocke, Ries & Tapley 1988, AIAA 88-4292, use them for satellite
/// Earth radiation pressure).
pub const EARTH_ALBEDO: f64 = 0.30;
pub const EARTH_IR_W_M2: f64 = 237.0;
const C_LIGHT: f64 = 299792458.0;
const RE: f64 = 6378137.0;

/// The pressures [N/m^2] of the Earth's albedo and infrared on a plate facing the Earth's centre
/// at r: the view factor of a sphere to a plate facing it is (Re/r)^2; the albedo falls as the
/// cosine of the Sun's zenith angle under the satellite (0 over the night side). `p_sun` is the
/// Sun's pressure at the satellite (W/c, so the albedo is a W (Re/r)^2 cos / c). The Earth is
/// taken as a point source in the nadir direction (Knocke et al.'s rings are not modelled).
pub fn earth_pressure(r: &V3, sun_rel: &V3, p_sun: f64) -> (f64, f64) {
    let rn = norm(r);
    let vf = (RE/rn)*(RE/rn);
    let cz = dot(&unit(&add(sun_rel, r)), &scale(r, 1.0/rn)).max(0.0);
    (EARTH_ALBEDO*p_sun*vf*cz, EARTH_IR_W_M2/C_LIGHT*vf)
}

/// The torque of light of pressure p arriving from body direction `sb` (unit, towards the
/// source) on the facets: absorbed, specular and diffuse parts (Wertz 1978, eq. 17-6).
pub fn radiation(g: &Facets, sb: &V3, p: f64) -> V3 {
    let mut out = [0.0; 3];
    for j in 0..6 {
        let c = dot(sb, &g.n[j]);
        if c <= 0.0 { continue; }
        let mut f = [0.0; 3];
        for k in 0..3 { f[k] = -p*g.a[j]*c*((1.0 - g.rho_spec)*sb[k] + 2.0*(g.rho_spec*c + g.rho_diff/3.0)*g.n[j][k]); }
        out = add(&out, &cross(&g.rho[j], &f));
    }
    out
}

/// [gg, aero, radiation (Sun, albedo, Earth IR), mag] torques, body frame.
pub fn torques(q: &Q, r: &V3, v_rel: &V3, b_eci: &V3, sun_rel: &V3, nu: f64, p_srp: f64, rho: f64, i: &M3, g: &Facets, m_res: &V3, mu: f64, on: [bool; 4]) -> [V3; 4] {
    let rm = dcm(q);
    let mut out = [[0.0; 3]; 4];
    if on[0] {
        let rb = mv(&rm, r);
        let rn = norm(&rb);
        out[0] = scale(&cross(&rb, &mv(i, &rb)), 3.0*mu/(rn*rn*rn*rn*rn));
    }
    if on[1] && rho > 0.0 {
        let vb = mv(&rm, v_rel);
        let vv = norm(&vb);
        let vh = scale(&vb, 1.0/vv);
        for j in 0..6 {
            let c = dot(&vh, &g.n[j]);
            if c <= 0.0 { continue; }
            let k2 = g.sigma_n*g.vb_ratio + (2.0 - g.sigma_n - g.sigma_t)*c;
            let mut f = [0.0; 3];
            for k in 0..3 { f[k] = -rho*vv*vv*g.a[j]*c*(g.sigma_t*vh[k] + k2*g.n[j][k]); }
            out[1] = add(&out[1], &cross(&g.rho[j], &f));
        }
    }
    if on[2] {
        // radiation pressure: the Sun, and the Earth's reflected (albedo) and emitted (IR) light
        if nu > 0.0 { out[2] = radiation(g, &unit(&mv(&rm, sun_rel)), nu*p_srp); }
        let (p_alb, p_ir) = earth_pressure(r, sun_rel, p_srp);
        let eb = unit(&scale(&mv(&rm, r), -1.0));
        out[2] = add(&out[2], &radiation(g, &eb, p_alb + p_ir));
    }
    if on[3] { out[3] = cross(m_res, &mv(&rm, b_eci)); }
    let _ = abs(0.0);
    out
}
