//! Control laws (05), detumble and Sun spin (06); twin of adcs_ctl.c. Guidance (04) is guid.rs.
use crate::m::*;
use crate::math::*;

#[derive(Clone, Copy, Debug, Default)]
pub struct Gains {
    /// 0 pid, 1 lqr, 2 smc
    pub law: u8,
    pub kp: V3, pub kd: V3, pub ki: V3, pub klqr: M3, pub lambda: f64, pub phi: f64, pub gs: V3, pub err_max: f64, pub int_max: f64,
}

#[allow(clippy::too_many_arguments)]
pub fn control_law(q: &Q, w: &V3, q_ref: &Q, w_ref: &V3, i_q: &mut V3, dt: f64, g: &Gains, j: &M3, hs: &V3, wd_ref: &V3) -> V3 {
    let qe = qerr(q_ref, q);
    let mut e = [0.0; 3];
    for i in 0..3 { e[i] = clamp(qe[i], -g.err_max, g.err_max); }
    let we = sub3(w, &mat3_vec(&dcm(&qe), w_ref));
    let gyro = cross(w, &add3(&mat3_vec(j, w), hs));
    let ff = mat3_vec(j, wd_ref);
    let mut tau = [0.0; 3];
    match g.law {
        1 => {
            for i in 0..3 {
                i_q[i] = clamp(i_q[i] + 2.0*e[i]*dt, -g.int_max, g.int_max);
                tau[i] = -(g.klqr[i][0]*i_q[i] + g.klqr[i][1]*(2.0*e[i]) + g.klqr[i][2]*we[i]);
            }
        }
        2 => {
            let c = cross(&e, &we);
            let mut x = [0.0; 3];
            for i in 0..3 {
                let s = we[i] + g.lambda*e[i];
                let sat = clamp(s/g.phi, -1.0, 1.0);
                let ed = 0.5*(qe[3]*we[i] + c[i]);
                x[i] = g.lambda*ed + g.gs[i]*sat;
            }
            tau = scale3(&mat3_vec(j, &x), -1.0);
        }
        _ => {
            for i in 0..3 {
                i_q[i] = clamp(i_q[i] + e[i]*dt, -g.int_max, g.int_max);
                tau[i] = -g.kp[i]*e[i] - g.kd[i]*we[i] - g.ki[i]*i_q[i];
            }
        }
    }
    for i in 0..3 { tau[i] += gyro[i] + ff[i]; }
    tau
}

pub fn mtq_pd(q: &Q, w: &V3, q_ref: &Q, w_ref: &V3, g: &Gains) -> V3 {
    let qe = qmult(&qconj(q_ref), q);
    let mut s = sign(qe[3]);
    if s == 0.0 { s = 1.0; }
    let wr = mat3_vec(&dcm(&qe), w_ref);
    let mut tau = [0.0; 3];
    for i in 0..3 { tau[i] = -g.kp[i]*(s*qe[i]) - g.kd[i]*(w[i] - wr[i]); }
    tau
}

/// Scale the dipole so no coil exceeds m_max (direction kept).
pub fn sat_dipole(m: &V3, m_max: f64) -> V3 {
    let mut a = 1.0;
    for i in 0..3 {
        let mut d = fabs(m[i]);
        if d < 1e-30 { d = 1e-30; }
        if m_max/d < a { a = m_max/d; }
    }
    scale3(m, a)
}

pub fn torque2dipole(tau: &V3, b: &V3, m_max: f64) -> V3 {
    let bs = dot(b, b);
    if bs < 1e-18 { return [0.0; 3]; }
    sat_dipole(&scale3(&cross(b, tau), 1.0/bs), m_max)
}

pub fn bdot(b1: &V3, b2: &V3, dt: f64, bn: f64, k: f64, m_max: f64) -> V3 {
    let u1 = unit(b1);
    let u2 = unit(b2);
    let mut m = [0.0; 3];
    for i in 0..3 { m[i] = -(k/bn)*(u2[i] - u1[i])/dt; }
    sat_dipole(&m, m_max)
}

pub fn gen_bdot(b: &V3, bd: &V3, wd: &V3, k: f64) -> V3 {
    let c = cross(wd, b);
    [-k*(bd[0] + c[0]), -k*(bd[1] + c[1]), -k*(bd[2] + c[2])]
}

#[allow(clippy::too_many_arguments)]
pub fn sun_spin(b: &V3, w: &V3, s: &V3, eclipse: bool, j: &M3, spin_dps: f64, k1: f64, k2: f64, rz_floor: f64) -> V3 {
    if eclipse { return [0.0; 3]; }
    let fl = rz_floor*j[2][2];
    let ws = -fabs(spin_dps*D2R);
    let mut sg = sign(w[2]);
    if sg == 0.0 { sg = 1.0; }
    let h = mat3_vec(j, w);
    let mut ht = [0.0; 3];
    for i in 0..3 { ht[i] = h[i] - sg*j[2][2]*ws*s[i]; }
    let mut rz = [j[2][2] - j[0][0], j[2][2] - j[1][1], 0.0];
    if rz[0] < fl { rz[0] = fl; }
    if rz[1] < fl { rz[1] = fl; }
    let mut x = [0.0; 3];
    for i in 0..3 { x[i] = k1*ht[i] + k2*rz[i]*w[i]; }
    let a = cross(b, &x);
    let bs = dot(b, b);
    if bs < 1e-18 { return [0.0; 3]; }
    [-a[0]/bs, -a[1]/bs, -a[2]/bs]
}

// ---- magnetorquer-only literature laws (05_control.md, docs/MTQ_LITERATURE.md) ----
// Mirrors fsw/src/adcs_ctl.c operation for operation (C = Rust bit for bit).
fn mtq_err(q: &Q, w: &V3, q_ref: &Q, w_ref: &V3) -> (Q, f64, V3) {
    let qe = qmult(&qconj(q_ref), q);
    let mut s = sign(qe[3]);
    if s == 0.0 { s = 1.0; }
    let wr = mat3_vec(&dcm(&qe), w_ref);
    let mut we = [0.0; 3];
    for i in 0..3 { we[i] = w[i] - wr[i]; }
    (qe, s, we)
}

/// P1 Lovera & Astolfi 2004, Prop. 1: u = -(eps^2 k_p q_v + eps k_v J w).
#[allow(clippy::too_many_arguments)]
pub fn mtq_lovera(q: &Q, w: &V3, q_ref: &Q, w_ref: &V3, j: &M3, eps: f64, kp: f64, kv: f64) -> V3 {
    let (qe, s, we) = mtq_err(q, w, q_ref, w_ref);
    let jw = mat3_vec(j, &we);
    let mut t = [0.0; 3];
    for i in 0..3 { t[i] = -(eps*eps*kp*(s*qe[i]) + eps*kv*jw[i]); }
    t
}

/// P4 Celani 2015, Thm 2: u = -(eps^2 k1 q_v + eps k2 w), no inertia in the law.
pub fn mtq_celani(q: &Q, w: &V3, q_ref: &Q, w_ref: &V3, eps: f64, k1: f64, k2: f64) -> V3 {
    let (qe, s, we) = mtq_err(q, w, q_ref, w_ref);
    let mut t = [0.0; 3];
    for i in 0..3 { t[i] = -(eps*eps*k1*(s*qe[i]) + eps*k2*we[i]); }
    t
}

/// P16 Avanzini, de Angelis & Giulietti 2021 (see adcs_ctl.c); None when the reference does not rotate.
#[allow(clippy::too_many_arguments)]
pub fn mtq_avanzini(q: &Q, w: &V3, q_ref: &Q, w_ref: &V3, j: &M3, k: f64, lam: f64) -> Option<V3> {
    let n = norm3(w_ref);
    if n < 1e-9 { return None; }
    let (qe, s, _we) = mtq_err(q, w, q_ref, w_ref);
    let mut ep = [0.0; 3];
    for i in 0..3 { ep[i] = w_ref[i]/n; }
    let sg = mat3_vec(&dcm(&qe), &ep);
    let jep = mat3_vec(j, &ep);
    let jp = dot(&ep, &jep);
    let th = 2.0*s*dot(&[qe[0], qe[1], qe[2]], &ep);
    let eta = jp*n*(1.0 - lam*th);
    let jw = mat3_vec(j, w);
    let mut t = [0.0; 3];
    for i in 0..3 { t[i] = k*(eta*sg[i] - jw[i]) + k*(eta*ep[i] - jw[i]); }
    Some(t)
}

/// P8 Celani 2026: boresight e3 onto the target a (body), rotation about e3 free.
pub fn mtq_boresight(e3: &V3, a: &V3, we: &V3, kp: f64, kd: f64) -> V3 {
    let c = cross(e3, a);
    let mut t = [0.0; 3];
    for i in 0..3 { t[i] = kp*c[i] - kd*we[i]; }
    t
}

/// P3 TANGO frozen-Riccati LQR: tau = -(P21/r theta + P22/r w), theta = 2 q_v.
pub fn mtq_tango(q: &Q, w: &V3, q_ref: &Q, w_ref: &V3, pth: &M3, pw: &M3) -> V3 {
    let (qe, s, we) = mtq_err(q, w, q_ref, w_ref);
    let mut th = [0.0; 3];
    for i in 0..3 { th[i] = 2.0*s*qe[i]; }
    let a = mat3_vec(pth, &th);
    let b = mat3_vec(pw, &we);
    [-(a[0] + b[0]), -(a[1] + b[1]), -(a[2] + b[2])]
}

/// P2 de Ruiter 2011 on the Sun line (see adcs_ctl.c).
#[allow(clippy::too_many_arguments)]
pub fn sun_spin_deruiter(b: &V3, w: &V3, s: &V3, eclipse: bool, j: &M3, spin_dps: f64, k: f64, k1: f64, k2: f64) -> V3 {
    if eclipse { return [0.0; 3]; }
    let ws = fabs(spin_dps*D2R);
    let mut sg = sign(w[2]);
    if sg == 0.0 { sg = 1.0; }
    let h = mat3_vec(j, w);
    let ehz = h[2] - sg*j[2][2]*ws;
    let mut x = [0.0; 3];
    for i in 0..3 { x[i] = h[i] + sg*j[2][2]*ws*s[i]; }
    x[2] += k1*ehz;
    x[0] += k2*w[0]; x[1] += k2*w[1];
    let a = cross(b, &x);
    let bs = dot(b, b);
    if bs < 1e-18 { return [0.0; 3]; }
    [-k*a[0]/bs, -k*a[1]/bs, -k*a[2]/bs]
}
