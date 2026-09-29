//! Guidance (04), control laws (05), detumble and Sun spin (06); twin of adcs_ctl.c.
use crate::m::*;
use crate::math::*;

#[derive(Clone, Copy, Debug, Default)]
pub struct Guid {
    pub q_off: Q, pub roll_deg: f64, pub t0: f64, pub t_slew: f64, pub axis: V3, pub q_inertial: Q,
    pub sun_axis: V3, pub roll_axis: V3, pub sun_eci: V3,
    /// nadir family turned 180 deg about the boresight (power face towards the Sun)
    pub flip: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Gains {
    /// 0 pid, 1 lqr, 2 smc
    pub law: u8,
    pub kp: V3, pub kd: V3, pub ki: V3, pub klqr: M3, pub lambda: f64, pub phi: f64, pub gs: V3, pub err_max: f64, pub int_max: f64,
}

pub struct Ref { pub q: Q, pub w: V3, pub wd: V3 }

/// kind: 0 nadir, 1 target, 2 slew, 3 inertial, 4 sun.
pub fn guidance(kind: i32, r: &V3, v: &V3, t: f64, g: &Guid) -> Ref {
    let rh = unit(r);
    let vh = unit(v);
    let mr = scale3(&rh, -1.0);
    let nrm = unit(&cross(&vh, &mr));
    let ram = unit(&cross(&mr, &nrm));
    let mut rm = [[0.0; 3]; 3];
    for i in 0..3 { rm[0][i] = -ram[i]; rm[1][i] = -rh[i]; rm[2][i] = -nrm[i]; }
    let mut q_nad = fromdcm(&rm);
    if g.q_off.iter().any(|&x| x != 0.0) { q_nad = qmult(&q_nad, &g.q_off); }
    if g.flip {
        let u = if norm3(&g.roll_axis) > 0.0 { unit(&g.roll_axis) } else { [1.0, 0.0, 0.0] };
        q_nad = qmult(&q_nad, &[u[0], u[1], u[2], 0.0]);
    }
    let w_orb = scale3(&cross(r, v), 1.0/dot(r, r));
    let mut wd = [0.0; 3];
    let ax = if norm3(&g.axis) > 0.0 { unit(&g.axis) } else { [1.0, 0.0, 0.0] };
    match kind {
        1 => {
            let q = qmult(&q_nad, &fromrotvec(&scale3(&ax, g.roll_deg*D2R)));
            Ref { q, w: mat3_vec(&dcm(&q), &w_orb), wd }
        }
        2 => {
            let tau = clamp((t - g.t0)/g.t_slew, 0.0, 1.0);
            let ph = g.roll_deg*D2R;
            let (mut sd, mut sdd) = (0.0, 0.0);
            if t > g.t0 && t < g.t0 + g.t_slew {
                sd = (1.0 - cos(2.0*PI*tau))/g.t_slew;
                sdd = 2.0*PI*sin(2.0*PI*tau)/(g.t_slew*g.t_slew);
            }
            let s = tau - sin(2.0*PI*tau)/(2.0*PI);
            let q = qmult(&q_nad, &fromrotvec(&scale3(&ax, ph*s)));
            let wo = mat3_vec(&dcm(&q), &w_orb);
            let mut w = [0.0; 3];
            // the slew frame turns at ax ph s_dot relative to the orbiting frame: transport term on w_orb
            let tr = cross(&scale3(&ax, ph*sd), &wo);
            for i in 0..3 { w[i] = wo[i] + ax[i]*ph*sd; wd[i] = ax[i]*ph*sdd - tr[i]; }
            Ref { q, w, wd }
        }
        3 => Ref { q: g.q_inertial, w: [0.0; 3], wd },
        4 => {
            let a = if norm3(&g.sun_axis) > 0.0 { unit(&g.sun_axis) } else { [0.0, 0.0, -1.0] };
            let mut b = if norm3(&g.roll_axis) > 0.0 { g.roll_axis } else { [1.0, 0.0, 0.0] };
            let ab = dot(&a, &b);
            for i in 0..3 { b[i] -= ab*a[i]; }
            if norm3(&b) < 1e-6 { b = [-a[1]*a[0], 1.0 - a[1]*a[1], -a[1]*a[2]]; }
            let b = unit(&b);
            let s = unit(&g.sun_eci);
            let ns = dot(&nrm, &s);
            let mut e2 = [nrm[0] - ns*s[0], nrm[1] - ns*s[1], nrm[2] - ns*s[2]];
            if norm3(&e2) < 1e-6 { e2 = [-s[2]*s[0], -s[2]*s[1], 1.0 - s[2]*s[2]]; }
            let e2 = unit(&e2);
            let b3 = cross(&a, &b);
            let e3 = cross(&s, &e2);
            let mut m = [[0.0; 3]; 3];
            for i in 0..3 { for j in 0..3 { m[i][j] = a[i]*s[j] + b[i]*e2[j] + b3[i]*e3[j]; } }
            Ref { q: fromdcm(&m), w: [0.0; 3], wd }
        }
        _ => Ref { q: q_nad, w: mat3_vec(&dcm(&q_nad), &w_orb), wd },
    }
}

/// Nadir-family yaw flip with hysteresis: turn 180 deg about the boresight when the power face would
/// look away from the Sun (04_guidance.md).
pub fn yaw_flip(g: &mut Guid, r: &V3, v: &V3, hyst: f64) {
    let mut g0 = *g;
    g0.flip = false;
    let q = guidance(0, r, v, 0.0, &g0).q;
    let sb = mat3_vec(&dcm(&q), &unit(&g.sun_eci));
    let a = if norm3(&g.sun_axis) > 0.0 { unit(&g.sun_axis) } else { [0.0, 0.0, -1.0] };
    let d = dot(&a, &sb);
    if d < -hyst { g.flip = true; } else if d > hyst { g.flip = false; }
}

/// Rotation taking body +y onto the boresight.
pub fn boresight_offset(bs_in: &V3) -> Q {
    let ey = [0.0, 1.0, 0.0];
    let bs = unit(bs_in);
    let ax = cross(&ey, &bs);
    let c = dot(&ey, &bs);
    let s = norm3(&ax);
    if s < 1e-12 { return if c <= 0.0 { [1.0, 0.0, 0.0, 0.0] } else { [0.0, 0.0, 0.0, 1.0] }; }
    let k = scale3(&ax, 1.0/s);
    let kk = skew(&k);
    let k2 = mat3_mul(&kk, &kk);
    let mut a = [[0.0; 3]; 3];
    for i in 0..3 { for j in 0..3 { a[i][j] = (if i == j { 1.0 } else { 0.0 }) + s*kk[i][j] + (1.0 - c)*k2[i][j]; } }
    fromdcm(&a)
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
