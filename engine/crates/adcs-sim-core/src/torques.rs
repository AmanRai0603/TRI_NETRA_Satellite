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

/// [gg, aero, srp, mag] torques, body frame.
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
    if on[2] && nu > 0.0 {
        let sb = unit(&mv(&rm, sun_rel));
        for j in 0..6 {
            let c = dot(&sb, &g.n[j]);
            if c <= 0.0 { continue; }
            let mut f = [0.0; 3];
            for k in 0..3 { f[k] = -nu*p_srp*g.a[j]*c*((1.0 - g.rho_spec)*sb[k] + 2.0*(g.rho_spec*c + g.rho_diff/3.0)*g.n[j][k]); }
            out[2] = add(&out[2], &cross(&g.rho[j], &f));
        }
    }
    if on[3] { out[3] = cross(m_res, &mv(&rm, b_eci)); }
    let _ = abs(0.0);
    out
}
