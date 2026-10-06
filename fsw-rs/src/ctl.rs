//! Control laws (05), detumble and Sun spin (06); twin of adcs_ctl.c. Guidance (04) is guid.rs.
use crate::alg::control as alg;
use crate::math::*;

#[derive(Clone, Copy, Debug, Default)]
pub struct Gains {
    /// 0 pid, 1 lqr, 2 smc
    pub law: u8,
    pub kp: V3, pub kd: V3, pub ki: V3, pub klqr: M3, pub lambda: f64, pub phi: f64, pub gs: V3, pub err_max: f64, pub int_max: f64,
}

/// Written from the design: control::control_law (src/alg).
#[allow(clippy::too_many_arguments)]
pub fn control_law(q: &Q, w: &V3, q_ref: &Q, w_ref: &V3, i_q: &mut V3, dt: f64, g: &Gains, j: &M3, hs: &V3, wd_ref: &V3) -> V3 {
    let tau;
    (tau, *i_q) = alg::control_law(*q, *w, *q_ref, *w_ref, *i_q, dt, g.law as i64, g.kp, g.kd, g.ki, g.klqr, g.lambda, g.phi, g.gs,
                                   g.err_max, g.int_max, *j, *hs, *wd_ref);
    tau
}

/// Written from the design: control::mtq_pd (src/alg).
pub fn mtq_pd(q: &Q, w: &V3, q_ref: &Q, w_ref: &V3, g: &Gains) -> V3 { alg::mtq_pd(*q, *w, *q_ref, *w_ref, g.kp, g.kd) }

/// Scale the dipole so no coil exceeds m_max (direction kept).
/// Written from the design: control::sat_dipole (src/alg).
pub fn sat_dipole(m: &V3, m_max: f64) -> V3 { alg::sat_dipole(*m, m_max) }

/// Written from the design: control::torque2dipole (src/alg).
pub fn torque2dipole(tau: &V3, b: &V3, m_max: f64) -> V3 { alg::torque2dipole(*tau, *b, m_max) }

/// Written from the design: control::bdot (src/alg).
pub fn bdot(b1: &V3, b2: &V3, dt: f64, bn: f64, k: f64, m_max: f64) -> V3 { alg::bdot(*b1, *b2, dt, bn, k, m_max) }

/// Written from the design: control::gen_bdot (src/alg).
pub fn gen_bdot(b: &V3, bd: &V3, wd: &V3, k: f64) -> V3 { alg::gen_bdot(*b, *bd, *wd, k) }

/// Written from the design: control::sun_spin (src/alg).
#[allow(clippy::too_many_arguments)]
pub fn sun_spin(b: &V3, w: &V3, s: &V3, eclipse: bool, j: &M3, spin_dps: f64, k1: f64, k2: f64, rz_floor: f64) -> V3 {
    alg::sun_spin(*b, *w, *s, eclipse, *j, spin_dps, k1, k2, rz_floor)
}

// ---- magnetorquer-only literature laws (05_control.md, docs/MTQ_LITERATURE.md) ----

/// P1 Lovera & Astolfi 2004, Prop. 1: u = -(eps^2 k_p q_v + eps k_v J w).
/// Written from the design: control::mtq_lovera (src/alg).
#[allow(clippy::too_many_arguments)]
pub fn mtq_lovera(q: &Q, w: &V3, q_ref: &Q, w_ref: &V3, j: &M3, eps: f64, kp: f64, kv: f64) -> V3 {
    alg::mtq_lovera(*q, *w, *q_ref, *w_ref, *j, eps, kp, kv)
}

/// P4 Celani 2015, Thm 2: u = -(eps^2 k1 q_v + eps k2 w), no inertia in the law.
/// Written from the design: control::mtq_celani (src/alg).
pub fn mtq_celani(q: &Q, w: &V3, q_ref: &Q, w_ref: &V3, eps: f64, k1: f64, k2: f64) -> V3 {
    alg::mtq_celani(*q, *w, *q_ref, *w_ref, eps, k1, k2)
}

/// P16 Avanzini, de Angelis & Giulietti 2021 (see adcs_ctl.c); None when the reference does not rotate.
/// Written from the design: control::mtq_avanzini (src/alg).
#[allow(clippy::too_many_arguments)]
pub fn mtq_avanzini(q: &Q, w: &V3, q_ref: &Q, w_ref: &V3, j: &M3, k: f64, lam: f64) -> Option<V3> {
    let (t, ok) = alg::mtq_avanzini(*q, *w, *q_ref, *w_ref, *j, k, lam);
    if ok { Some(t) } else { None }
}

/// P8 Celani 2026: boresight e3 onto the target a (body), rotation about e3 free.
/// Written from the design: control::mtq_boresight (src/alg).
pub fn mtq_boresight(e3: &V3, a: &V3, we: &V3, kp: f64, kd: f64) -> V3 { alg::mtq_boresight(*e3, *a, *we, kp, kd) }

/// P3 TANGO frozen-Riccati LQR: tau = -(P21/r theta + P22/r w), theta = 2 q_v.
/// Written from the design: control::mtq_tango (src/alg).
pub fn mtq_tango(q: &Q, w: &V3, q_ref: &Q, w_ref: &V3, pth: &M3, pw: &M3) -> V3 { alg::mtq_tango(*q, *w, *q_ref, *w_ref, *pth, *pw) }

/// P2 de Ruiter 2011 on the Sun line (see adcs_ctl.c).
/// Written from the design: control::sun_spin_deruiter (src/alg).
#[allow(clippy::too_many_arguments)]
pub fn sun_spin_deruiter(b: &V3, w: &V3, s: &V3, eclipse: bool, j: &M3, spin_dps: f64, k: f64, k1: f64, k2: f64) -> V3 {
    alg::sun_spin_deruiter(*b, *w, *s, eclipse, *j, spin_dps, k, k1, k2)
}
