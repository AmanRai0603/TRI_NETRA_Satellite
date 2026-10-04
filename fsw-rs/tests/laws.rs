//! The control laws (fsw/pseudocode/05_control.md), the detumble and Sun-spin laws
//! (06_detumble_sunspin.md) and the corner cases of guidance (04_guidance.md), each against what its
//! definition says: the closed form of the law written out here from the pseudocode and the papers it
//! cites (Lovera & Astolfi 2004, Celani 2015 and 2026, Avanzini, de Angelis & Giulietti 2021, Chasset
//! et al. 2013 (TANGO), de Ruiter 2011, Avanzini & Giulietti 2012), with the attitude error built from a
//! rotation chosen here rather than read back; then the invariants (q and -q are one attitude, the coils
//! only make torque across B, saturation keeps the direction), the equilibria each law is built to hold,
//! the thresholds (`<` against `<=`) and the law selectors. Written from the definitions, not from a run.
#![allow(clippy::needless_range_loop)]
use adcs_fsw::ctl::{self, Gains};
use adcs_fsw::guid::{self, Guid};
use adcs_fsw::math::*;

const D2R_: f64 = std::f64::consts::PI/180.0;

/// Equal to a relative 1e-12 of the larger of the two, or of `scale` when that is larger.
fn near3(got: &V3, want: &V3, scale: f64, what: &str) {
    let s = norm3(got).max(norm3(want)).max(scale);
    assert!(norm3(&sub3(got, want)) <= 1e-12*s, "{what}: got {got:?}, want {want:?}");
}
fn mv(m: &M3, v: &V3) -> V3 { [0, 1, 2].map(|i| m[i][0]*v[0] + m[i][1]*v[1] + m[i][2]*v[2]) }
fn neg(q: &Q) -> Q { [-q[0], -q[1], -q[2], -q[3]] }

/// A full, symmetric inertia, so that every off-diagonal term matters.
const J: M3 = [[0.031, 0.002, -0.001], [0.002, 0.028, 0.0015], [-0.001, 0.0015, 0.012]];

/// The reference, and the attitude a known rotation `phi` (body) away from it: q = q_ref (x) delta,
/// so the error quaternion conj(q_ref) (x) q is delta, with a positive scalar part.
struct Case { q_ref: Q, q: Q, delta: Q, w: V3, w_ref: V3 }
fn case() -> Case {
    let q_ref = qnorm(&[0.2, -0.4, 0.1, 0.85]);
    let delta = fromrotvec(&[0.12, -0.15, 0.2]);
    Case { q_ref, q: qmult(&q_ref, &delta), delta, w: [0.011, -0.023, 0.017], w_ref: [0.0013, -0.0011, 0.0004] }
}
impl Case {
    fn qv(&self) -> V3 { [self.delta[0], self.delta[1], self.delta[2]] }
    /// the rate error: the body rate less the reference rate seen in the body
    fn we(&self) -> V3 { sub3(&self.w, &mv(&dcm(&self.delta), &self.w_ref)) }
}

// ---------------------------------------------------------------- control_law (momentum devices)

const HS: V3 = [1e-3, -2e-3, 5e-4];
const WD_REF: V3 = [1e-4, -2e-4, 3e-4];
fn gyro_ff(w: &V3) -> V3 { add3(&cross(w, &add3(&mv(&J, w), &HS)), &mv(&J, &WD_REF)) }
fn gains(law: u8) -> Gains {
    Gains { law, kp: [0.5, 0.7, 0.3], kd: [2.0, 1.5, 1.1], ki: [0.01, 0.02, 0.03],
            klqr: [[0.03, 0.9, 2.1], [0.05, 1.1, 1.7], [0.02, 0.6, 0.9]],
            lambda: 0.4, phi: 0.04, gs: [0.02, 0.03, 0.04], err_max: 0.065, int_max: 0.02 }
}
/// The error clamped per component at +-err_max: here x is inside, y below -err_max, z above +err_max.
fn clamped(c: &Case, g: &Gains) -> V3 {
    let qv = c.qv();
    assert!(qv[0].abs() < g.err_max && qv[1] < -g.err_max && qv[2] > g.err_max, "the case covers every side of the clamp");
    qv.map(|x| x.clamp(-g.err_max, g.err_max))
}

#[test]
fn pid_is_minus_kp_e_minus_kd_rate_error_minus_ki_integral_plus_gyroscopic_and_feed_forward() {
    let c = case();
    let g = gains(0);
    let dt = 0.1;
    let e = clamped(&c, &g);
    let i0 = [0.019, -0.0195, 0.0];
    // the integral is clamped at +-int_max: x over the top, y under the bottom, z free
    let iq: V3 = [0, 1, 2].map(|i| (i0[i] + e[i]*dt).clamp(-g.int_max, g.int_max));
    assert!(iq[0] == g.int_max && iq[1] == -g.int_max && iq[2].abs() < g.int_max);
    let we = c.we();
    let mut want = gyro_ff(&c.w);
    for i in 0..3 { want[i] += -g.kp[i]*e[i] - g.kd[i]*we[i] - g.ki[i]*iq[i]; }
    for (q, label) in [(c.q, "q"), (neg(&c.q), "-q, the same attitude")] {
        let mut i_q = i0;
        let tau = ctl::control_law(&q, &c.w, &c.q_ref, &c.w_ref, &mut i_q, dt, &g, &J, &HS, &WD_REF);
        near3(&tau, &want, 0.0, label);
        near3(&i_q, &iq, 1e-3, "the integral state");
    }
    // any selector other than 1 and 2 is the PID law
    let mut i_q = i0;
    let tau = ctl::control_law(&c.q, &c.w, &c.q_ref, &c.w_ref, &mut i_q, dt, &Gains { law: 7, ..g }, &J, &HS, &WD_REF);
    near3(&tau, &want, 0.0, "law 7");
}

#[test]
fn lqr_integrates_twice_the_error_and_weighs_integral_angle_and_rate_by_its_three_gain_columns() {
    let c = case();
    let g = gains(1);
    let dt = 0.1;
    let e = clamped(&c, &g);
    let i0 = [0.01, -0.012, 0.003];
    let iq: V3 = [0, 1, 2].map(|i| (i0[i] + 2.0*e[i]*dt).clamp(-g.int_max, g.int_max));
    assert!(iq[0] == g.int_max && iq[1] == -g.int_max && iq[2].abs() < g.int_max);
    let we = c.we();
    let mut want = gyro_ff(&c.w);
    for i in 0..3 { want[i] -= g.klqr[i][0]*iq[i] + g.klqr[i][1]*2.0*e[i] + g.klqr[i][2]*we[i]; }
    for (q, label) in [(c.q, "q"), (neg(&c.q), "-q")] {
        let mut i_q = i0;
        let tau = ctl::control_law(&q, &c.w, &c.q_ref, &c.w_ref, &mut i_q, dt, &g, &J, &HS, &WD_REF);
        near3(&tau, &want, 0.0, label);
        near3(&i_q, &iq, 1e-3, "the integral state");
    }
}

#[test]
fn sliding_mode_is_minus_j_times_lambda_e_dot_plus_gs_sat_s_over_phi_and_leaves_the_integral_alone() {
    let c = case();
    let g = gains(2);
    let e = clamped(&c, &g);
    let we = c.we();
    let qw = c.delta[3];
    let s: V3 = [0, 1, 2].map(|i| we[i] + g.lambda*e[i]);
    // one axis inside the boundary layer, one saturated each way
    assert!((s[0]/g.phi).abs() < 1.0 && s[1]/g.phi < -1.0 && s[2]/g.phi > 1.0, "{s:?}");
    let ec = cross(&e, &we);
    let x: V3 = [0, 1, 2].map(|i| {
        let edot = 0.5*(qw*we[i] + ec[i]);
        g.lambda*edot + g.gs[i]*(s[i]/g.phi).clamp(-1.0, 1.0)
    });
    let want = sub3(&gyro_ff(&c.w), &mv(&J, &x));
    for (q, label) in [(c.q, "q"), (neg(&c.q), "-q")] {
        let mut i_q = [0.003, -0.004, 0.005];
        let tau = ctl::control_law(&q, &c.w, &c.q_ref, &c.w_ref, &mut i_q, 0.1, &g, &J, &HS, &WD_REF);
        near3(&tau, &want, 0.0, label);
        assert_eq!(i_q, [0.003, -0.004, 0.005], "no integral in the sliding-mode law");
    }
}

#[test]
fn every_law_commands_nothing_at_rest_on_the_reference() {
    let c = case();
    for law in [0, 1, 2] {
        let mut i_q = [0.0; 3];
        let tau = ctl::control_law(&c.q_ref, &[0.0; 3], &c.q_ref, &[0.0; 3], &mut i_q, 0.1, &gains(law), &J, &[0.0; 3], &[0.0; 3]);
        assert!(norm3(&tau) < 1e-15 && norm3(&i_q) < 1e-15, "law {law}: {tau:?}");
    }
    // on the reference the rate error vanishes, so only the gyroscopic and feed-forward terms remain
    let w = [0.002, -0.001, 0.003];
    for law in [0, 1, 2] {
        let mut i_q = [0.0; 3];
        let tau = ctl::control_law(&c.q_ref, &w, &c.q_ref, &w, &mut i_q, 0.1, &gains(law), &J, &HS, &WD_REF);
        near3(&tau, &gyro_ff(&w), 0.0, &format!("law {law} on the reference"));
    }
}

#[test]
fn a_larger_attitude_error_pushes_back_harder_and_the_rate_term_damps() {
    // single-axis sign conventions: a positive error about x, or a positive rate, gives a negative torque
    let g = Gains { law: 0, kp: [1.0; 3], kd: [1.0; 3], ki: [0.0; 3], err_max: 1.0, int_max: 1.0, ..Default::default() };
    let jd = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    let id = [0.0, 0.0, 0.0, 1.0];
    let mut i_q = [0.0; 3];
    let small = ctl::control_law(&fromrotvec(&[0.1, 0.0, 0.0]), &[0.0; 3], &id, &[0.0; 3], &mut i_q, 0.0, &g, &jd, &[0.0; 3], &[0.0; 3]);
    let big = ctl::control_law(&fromrotvec(&[0.3, 0.0, 0.0]), &[0.0; 3], &id, &[0.0; 3], &mut i_q, 0.0, &g, &jd, &[0.0; 3], &[0.0; 3]);
    assert!(small[0] < 0.0 && big[0] < small[0] && small[1] == 0.0 && small[2] == 0.0);
    assert!((small[0] + (0.05f64).sin()).abs() < 1e-15, "Kp sin(theta/2)");
    let damp = ctl::control_law(&id, &[0.2, 0.0, 0.0], &id, &[0.0; 3], &mut i_q, 0.0, &g, &jd, &[0.0; 3], &[0.0; 3]);
    assert!((damp[0] + 0.2).abs() < 1e-15 && damp[1] == 0.0 && damp[2] == 0.0, "Kd opposes the rate: {damp:?}");
}

// ---------------------------------------------------------------- magnetorquer laws

#[test]
fn mtq_pd_is_minus_kp_signed_vector_part_minus_kd_rate_error() {
    let c = case();
    let g = Gains { kp: [0.5, 0.7, 0.3], kd: [2.0, 1.5, 1.1], ..Default::default() };
    let (qv, we) = (c.qv(), c.we());
    let want: V3 = [0, 1, 2].map(|i| -g.kp[i]*qv[i] - g.kd[i]*we[i]);
    near3(&ctl::mtq_pd(&c.q, &c.w, &c.q_ref, &c.w_ref, &g), &want, 0.0, "q");
    near3(&ctl::mtq_pd(&neg(&c.q), &c.w, &c.q_ref, &c.w_ref, &g), &want, 0.0, "-q: the shorter way round");
    // a half turn about z: the scalar part is zero and the sign is taken as +1
    let half = [0.0, 0.0, 1.0, 0.0];
    let wr = mv(&dcm(&half), &c.w_ref);
    let want: V3 = [0, 1, 2].map(|i| -g.kp[i]*half[i] - g.kd[i]*(c.w[i] - wr[i]));
    near3(&ctl::mtq_pd(&half, &c.w, &[0.0, 0.0, 0.0, 1.0], &c.w_ref, &g), &want, 0.0, "half turn");
}

/// The common error of the literature laws, its three terms visible separately: the sign of q_e.w
/// (q and -q), the half turn where it is zero (taken as +1), and the reference rate carried into the body.
fn literature_cases(c: &Case) -> [(Q, Q, V3, f64, &'static str); 3] {
    let half = [0.0, 1.0, 0.0, 0.0];
    [(c.q, c.q_ref, c.qv(), 1.0, "q"), (neg(&c.q), c.q_ref, c.qv(), 1.0, "-q"), (half, [0.0, 0.0, 0.0, 1.0], [0.0, 1.0, 0.0], 0.0, "half turn")]
}

#[test]
fn lovera_astolfi_is_minus_eps_squared_kp_q_minus_eps_kv_j_rate_error() {
    let c = case();
    let (eps, kp, kv) = (0.01, 2.0, 0.9);
    for (q, q_ref, qv, _, label) in literature_cases(&c) {
        let qe = qmult(&qconj(&q_ref), &q);
        let we = sub3(&c.w, &mv(&dcm(&qe), &c.w_ref));
        let jw = mv(&J, &we);
        let want: V3 = [0, 1, 2].map(|i| -(eps*eps*kp*qv[i] + eps*kv*jw[i]));
        near3(&ctl::mtq_lovera(&q, &c.w, &q_ref, &c.w_ref, &J, eps, kp, kv), &want, 0.0, label);
    }
}

#[test]
fn celani_2015_is_the_same_law_without_the_inertia() {
    let c = case();
    let (eps, k1, k2) = (0.01, 2.0, 0.03);
    for (q, q_ref, qv, _, label) in literature_cases(&c) {
        let qe = qmult(&qconj(&q_ref), &q);
        let we = sub3(&c.w, &mv(&dcm(&qe), &c.w_ref));
        let want: V3 = [0, 1, 2].map(|i| -(eps*eps*k1*qv[i] + eps*k2*we[i]));
        near3(&ctl::mtq_celani(&q, &c.w, &q_ref, &c.w_ref, eps, k1, k2), &want, 0.0, label);
    }
}

#[test]
fn tango_is_minus_p_theta_times_twice_q_minus_p_omega_times_rate_error() {
    let c = case();
    let pth = [[0.002, 0.0003, -0.0001], [0.0004, 0.0015, 0.0002], [-0.0002, 0.0001, 0.001]];
    let pw = [[0.03, -0.004, 0.001], [0.002, 0.025, -0.003], [0.001, 0.002, 0.02]];
    for (q, q_ref, qv, _, label) in literature_cases(&c) {
        let qe = qmult(&qconj(&q_ref), &q);
        let we = sub3(&c.w, &mv(&dcm(&qe), &c.w_ref));
        let want = scale3(&add3(&mv(&pth, &scale3(&qv, 2.0)), &mv(&pw, &we)), -1.0);
        near3(&ctl::mtq_tango(&q, &c.w, &q_ref, &c.w_ref, &pth, &pw), &want, 0.0, label);
    }
}

#[test]
fn avanzini_spins_the_reference_axis_and_corrects_the_pitch_through_eta() {
    let c = case();
    let (k, lam) = (0.5, 0.08);
    let n = norm3(&c.w_ref);
    let ep = scale3(&c.w_ref, 1.0/n);
    let jp = dot(&ep, &mv(&J, &ep));
    let jw = mv(&J, &c.w);
    for (q, q_ref, qv, _, label) in literature_cases(&c) {
        let qe = qmult(&qconj(&q_ref), &q);
        // the axis meant to spin, seen in the body now; the pitch error to first order
        let sigma = mv(&dcm(&qe), &ep);
        let theta = 2.0*dot(&qv, &ep);
        let eta = jp*n*(1.0 - lam*theta);
        let want: V3 = [0, 1, 2].map(|i| k*(eta*sigma[i] - jw[i]) + k*(eta*ep[i] - jw[i]));
        let got = ctl::mtq_avanzini(&q, &c.w, &q_ref, &c.w_ref, &J, k, lam).expect("a rotating reference");
        near3(&got, &want, 0.0, label);
    }
    // on the reference, spinning at its rate about a principal axis: nothing to do
    let jd = [[0.031, 0.0, 0.0], [0.0, 0.028, 0.0], [0.0, 0.0, 0.012]];
    let w_ref = [0.0, -0.0011, 0.0];
    let t = ctl::mtq_avanzini(&c.q_ref, &w_ref, &c.q_ref, &w_ref, &jd, k, lam).unwrap();
    assert!(norm3(&t) < 1e-18, "{t:?}");
}

#[test]
fn avanzini_needs_a_reference_rate_of_at_least_1e_minus_9() {
    let c = case();
    assert!(ctl::mtq_avanzini(&c.q, &c.w, &c.q_ref, &[0.0; 3], &J, 0.5, 0.08).is_none());
    assert!(ctl::mtq_avanzini(&c.q, &c.w, &c.q_ref, &[0.0, 0.9e-9, 0.0], &J, 0.5, 0.08).is_none());
    assert!(ctl::mtq_avanzini(&c.q, &c.w, &c.q_ref, &[0.0, 1.1e-9, 0.0], &J, 0.5, 0.08).is_some());
    // the test is strict: a rate of exactly 1e-9 (representable, and its own norm) already counts
    assert!(ctl::mtq_avanzini(&c.q, &c.w, &c.q_ref, &[0.0, 1e-9, 0.0], &J, 0.5, 0.08).is_some(), "at 1e-9");
}

#[test]
fn celani_2026_boresight_law_is_kp_e3_cross_a_minus_kd_rate_error() {
    let e3 = [0.0, 0.0, -1.0];
    let a = unit(&[0.2, -0.3, -0.9]);
    let we = [0.01, -0.02, 0.005];
    let (kp, kd) = (0.003, 0.04);
    let x = cross(&e3, &a);
    let want: V3 = [0, 1, 2].map(|i| kp*x[i] - kd*we[i]);
    near3(&ctl::mtq_boresight(&e3, &a, &we, kp, kd), &want, 0.0, "law");
    // the boresight on the target and at rest: nothing; a turn about the boresight is not resisted by kp
    assert_eq!(ctl::mtq_boresight(&e3, &e3, &[0.0; 3], kp, kd), [0.0; 3]);
    let t = ctl::mtq_boresight(&e3, &a, &[0.0; 3], kp, kd);
    assert!(dot(&t, &e3).abs() < 1e-18 && dot(&t, &x) > 0.0, "turns e3 toward a");
}

// ---------------------------------------------------------------- torque to dipole, saturation

#[test]
fn the_dipole_is_square_to_b_and_makes_the_commanded_torque_across_b() {
    let b = [2.1e-5, -1.3e-5, 3.4e-5];
    let tau = [1.0e-6, 2.0e-6, -1.5e-6];
    let m = ctl::torque2dipole(&tau, &b, 1.0);
    let bb = dot(&b, &b);
    near3(&m, &scale3(&cross(&b, &tau), 1.0/bb), 0.0, "B x tau/|B|^2");
    assert!(dot(&m, &b).abs() < 1e-12*norm3(&m)*norm3(&b), "square to B");
    // m x B is tau less its component along B: the coils cannot make the rest
    let bh = unit(&b);
    near3(&cross(&m, &b), &sub3(&tau, &scale3(&bh, dot(&tau, &bh))), 0.0, "m x B");
}

#[test]
fn the_dipole_saturates_on_its_largest_coil_keeping_its_direction() {
    let b = [2.1e-5, -1.3e-5, 3.4e-5];
    let tau = [1.0e-6, 2.0e-6, -1.5e-6];
    let free = scale3(&cross(&b, &tau), 1.0/dot(&b, &b));
    let m_max = 0.5*maxabs3(&free);
    let m = ctl::torque2dipole(&tau, &b, m_max);
    near3(&m, &scale3(&free, 0.5), 0.0, "halved");
    assert!((maxabs3(&m) - m_max).abs() <= 1e-15*m_max);
    // sat_dipole by itself: every coil within m_max, the direction kept; nothing done below it
    let d = [0.3, -0.9, 0.6];
    near3(&ctl::sat_dipole(&d, 0.45), &scale3(&d, 0.5), 0.0, "sat");
    assert_eq!(ctl::sat_dipole(&d, 0.9), d, "at the limit: unchanged");
    assert_eq!(ctl::sat_dipole(&d, 2.0), d);
    assert_eq!(ctl::sat_dipole(&[0.0; 3], 0.2), [0.0; 3]);
}

#[test]
fn no_field_no_dipole_below_a_squared_field_of_1e_minus_18() {
    let tau = [1.0e-6, 2.0e-6, -1.5e-6];
    assert_eq!(ctl::torque2dipole(&tau, &[0.9e-9, 0.0, 0.0], 1e9), [0.0; 3]);
    let m = ctl::torque2dipole(&tau, &[1.1e-9, 0.0, 0.0], 1e9);
    near3(&m, &scale3(&cross(&[1.1e-9, 0.0, 0.0], &tau), 1.0/(1.1e-9*1.1e-9)), 0.0, "just above");
    // the test is strict: |B|^2 of exactly 1e-18 (1e-9 squared is exact in binary64) already counts
    let edge = [1e-9, 0.0, 0.0];
    assert_eq!(dot(&edge, &edge), 1e-18);
    near3(&ctl::torque2dipole(&tau, &edge, 1e9), &scale3(&cross(&edge, &tau), 1e18), 0.0, "at 1e-18");
}

// ---------------------------------------------------------------- detumble

#[test]
fn bdot_is_minus_k_over_b_times_the_unit_field_derivative() {
    let b1 = [2.0e-5, -1.0e-5, 3.0e-5];
    let b2 = [2.1e-5, -1.2e-5, 2.9e-5];
    let (dt, bn, k) = (0.8, 3.5e-5, 2.0e-5);
    let d = sub3(&unit(&b2), &unit(&b1));
    let want = scale3(&d, -(k/bn)/dt);
    near3(&ctl::bdot(&b1, &b2, dt, bn, k, 1.0), &want, 0.0, "unsaturated");
    let m_max = 0.25*maxabs3(&want);
    near3(&ctl::bdot(&b1, &b2, dt, bn, k, m_max), &scale3(&want, 0.25), 0.0, "saturated");
}

#[test]
fn generalised_bdot_is_minus_k_times_the_field_derivative_plus_the_desired_rate_cross_b() {
    let b = [2.0e-5, -1.0e-5, 3.0e-5];
    let bd = [1.0e-7, 3.0e-7, -2.0e-7];
    let wd = [0.01, -0.02, -0.07];
    let k = 3.0e3;
    let want = scale3(&add3(&bd, &cross(&wd, &b)), -k);
    near3(&ctl::gen_bdot(&b, &bd, &wd, k), &want, 0.0, "law");
    // spinning at the desired rate in an inertially fixed field the body sees db/dt = -w x B: no dipole
    let still = ctl::gen_bdot(&b, &scale3(&cross(&wd, &b), -1.0), &wd, k);
    assert!(norm3(&still) < 1e-20, "{still:?}");
}

// ---------------------------------------------------------------- Sun spin (He et al.) and de Ruiter

const JS1: M3 = [[0.030, 0.001, -0.002], [0.001, 0.035, 0.0005], [-0.002, 0.0005, 0.040]];
/// x long: J_zz - J_xx negative, so the x floor holds and the y one does not.
const JS2: M3 = [[0.050, 0.001, -0.002], [0.001, 0.020, 0.0005], [-0.002, 0.0005, 0.040]];
const B: V3 = [2.1e-5, -1.3e-5, 3.4e-5];

#[allow(clippy::too_many_arguments)]
fn sun_spin_ref(b: &V3, w: &V3, s: &V3, j: &M3, spin_dps: f64, k1: f64, k2: f64, rz_floor: f64) -> V3 {
    let ws = -(spin_dps*D2R_).abs();
    let sg = if w[2] < 0.0 { -1.0 } else { 1.0 };
    let h = mv(j, w);
    let ht: V3 = [0, 1, 2].map(|i| h[i] - sg*j[2][2]*ws*s[i]);
    let fl = rz_floor*j[2][2];
    let rz = [(j[2][2] - j[0][0]).max(fl), (j[2][2] - j[1][1]).max(fl), 0.0];
    let x: V3 = [0, 1, 2].map(|i| k1*ht[i] + k2*rz[i]*w[i]);
    scale3(&cross(b, &x), -1.0/dot(b, b))
}

#[test]
fn sun_spin_is_minus_b_cross_k1_momentum_error_plus_k2_rz_w_over_b_squared() {
    let s = unit(&[0.1, -0.2, -0.95]);
    let (k1, k2, floor) = (0.8, 0.5, 0.2);
    for (j, label) in [(JS1, "y floored"), (JS2, "x floored")] {
        let fl = floor*j[2][2];
        assert!((j[2][2] - j[0][0] < fl) != (j[2][2] - j[1][1] < fl), "{label}: one axis on the floor");
        for w in [[0.01, -0.02, 0.06], [0.01, -0.02, -0.06], [0.01, -0.02, 0.0]] {
            for spin in [4.0, -4.0] {
                let want = sun_spin_ref(&B, &w, &s, &j, spin, k1, k2, floor);
                let m = ctl::sun_spin(&B, &w, &s, false, &j, spin, k1, k2, floor);
                near3(&m, &want, 0.0, &format!("{label}, w {w:?}, spin {spin}"));
                assert!(dot(&m, &B).abs() < 1e-12*norm3(&m)*norm3(&B), "square to B");
            }
        }
    }
}

#[test]
fn sun_spin_holds_minus_z_on_the_sun_spinning_at_the_set_rate() {
    let jd = [[0.030, 0.0, 0.0], [0.0, 0.035, 0.0], [0.0, 0.0, 0.040]];
    let s = [0.0, 0.0, -1.0];
    for wz in [4.0*D2R_, -4.0*D2R_] {
        let m = ctl::sun_spin(&B, &[0.0, 0.0, wz], &s, false, &jd, 4.0, 0.8, 0.5, 0.2);
        assert!(norm3(&m) < 1e-12, "spin {wz}: {m:?}");
        let m = ctl::sun_spin_deruiter(&B, &[0.0, 0.0, wz], &s, false, &jd, -4.0, 2.0, 1.5, 0.3);
        assert!(norm3(&m) < 1e-12, "de Ruiter, spin {wz}: {m:?}");
    }
    // spinning the wrong speed, or tilted, is not
    assert!(norm3(&ctl::sun_spin(&B, &[0.0, 0.0, 3.0*D2R_], &s, false, &jd, 4.0, 0.8, 0.5, 0.2)) > 1e-3);
    assert!(norm3(&ctl::sun_spin_deruiter(&B, &[0.0, 0.0, 3.0*D2R_], &s, false, &jd, 4.0, 2.0, 1.5, 0.3)) > 1e-3);
}

#[test]
fn de_ruiter_is_minus_k_b_cross_the_lyapunov_gradient_over_b_squared() {
    let s = unit(&[0.1, -0.2, -0.95]);
    let (k, k1, k2) = (2.0, 1.5, 0.3);
    for j in [JS1, JS2] {
        for w in [[0.01, -0.02, 0.06], [0.01, -0.02, -0.06], [0.01, -0.02, 0.0]] {
            for spin in [4.0, -4.0] {
                let ws = (spin*D2R_).abs();
                let sg = if w[2] < 0.0 { -1.0 } else { 1.0 };
                let h = mv(&j, &w);
                let ehz = h[2] - sg*j[2][2]*ws;
                // h - h_d + k1 e_hz z + k2 P w, h_d = -sg J_zz ws s, P = diag(1, 1, 0)
                let x = [h[0] + sg*j[2][2]*ws*s[0] + k2*w[0],
                         h[1] + sg*j[2][2]*ws*s[1] + k2*w[1],
                         h[2] + sg*j[2][2]*ws*s[2] + k1*ehz];
                let want = scale3(&cross(&B, &x), -k/dot(&B, &B));
                let m = ctl::sun_spin_deruiter(&B, &w, &s, false, &j, spin, k, k1, k2);
                near3(&m, &want, 0.0, &format!("w {w:?}, spin {spin}"));
            }
        }
    }
}

#[test]
fn both_sun_laws_are_off_in_eclipse_and_without_a_field() {
    let s = unit(&[0.1, -0.2, -0.95]);
    let w = [0.01, -0.02, 0.06];
    assert_eq!(ctl::sun_spin(&B, &w, &s, true, &JS1, 4.0, 0.8, 0.5, 0.2), [0.0; 3]);
    assert_eq!(ctl::sun_spin_deruiter(&B, &w, &s, true, &JS1, 4.0, 2.0, 1.5, 0.3), [0.0; 3]);
    let tiny = [0.9e-9, 0.0, 0.0];
    assert_eq!(ctl::sun_spin(&tiny, &w, &s, false, &JS1, 4.0, 0.8, 0.5, 0.2), [0.0; 3]);
    assert_eq!(ctl::sun_spin_deruiter(&tiny, &w, &s, false, &JS1, 4.0, 2.0, 1.5, 0.3), [0.0; 3]);
    let small = [1.1e-9, 0.0, 0.0];
    near3(&ctl::sun_spin(&small, &w, &s, false, &JS1, 4.0, 0.8, 0.5, 0.2),
          &sun_spin_ref(&small, &w, &s, &JS1, 4.0, 0.8, 0.5, 0.2), 0.0, "just above the field threshold");
    assert!(norm3(&ctl::sun_spin_deruiter(&small, &w, &s, false, &JS1, 4.0, 2.0, 1.5, 0.3)) > 0.0);
    // |B|^2 of exactly 1e-18 is not below the threshold: both laws still run
    let edge = [1e-9, 0.0, 0.0];
    assert_eq!(dot(&edge, &edge), 1e-18);
    near3(&ctl::sun_spin(&edge, &w, &s, false, &JS1, 4.0, 0.8, 0.5, 0.2),
          &sun_spin_ref(&edge, &w, &s, &JS1, 4.0, 0.8, 0.5, 0.2), 0.0, "at the field threshold");
    assert!(norm3(&ctl::sun_spin_deruiter(&edge, &w, &s, false, &JS1, 4.0, 2.0, 1.5, 0.3)) > 0.0, "de Ruiter at the threshold");
}

// ---------------------------------------------------------------- boresight offset, guidance corners, yaw flip

#[test]
fn the_boresight_offset_is_the_shortest_rotation_taking_plus_y_onto_the_boresight() {
    for bs in [[0.3, 0.8, -0.2], [-2.0, 0.5, 1.0], [0.0, -0.7, 0.7], [1.0, 0.0, 0.0], [0.1, -3.0, 0.2]] {
        let q = guid::boresight_offset(&bs);
        let a = dcm(&q);
        let u = unit(&bs);
        near3(&mv(&a, &[0.0, 1.0, 0.0]), &u, 1.0, &format!("{bs:?}: +y onto it"));
        assert!((det3(&a) - 1.0).abs() < 1e-12 && (q.iter().map(|x| x*x).sum::<f64>() - 1.0).abs() < 1e-12, "{bs:?}: a proper rotation");
        // the shortest: the angle between them, about their common normal (which it leaves alone)
        assert!((qangle(&q, &[0.0, 0.0, 0.0, 1.0]) - u[1].clamp(-1.0, 1.0).acos()).abs() < 1e-7, "{bs:?}: angle");
        let n = unit(&cross(&[0.0, 1.0, 0.0], &u));
        near3(&mv(&a, &n), &n, 1.0, &format!("{bs:?}: the axis is fixed"));
    }
}

#[test]
fn the_boresight_offset_on_the_y_axis_is_exactly_none_or_a_half_turn_about_x() {
    assert_eq!(guid::boresight_offset(&[0.0, 3.0, 0.0]), [0.0, 0.0, 0.0, 1.0]);
    assert_eq!(guid::boresight_offset(&[0.0, -3.0, 0.0]), [1.0, 0.0, 0.0, 0.0]);
    // within 1e-12 of the axis counts as on it
    assert_eq!(guid::boresight_offset(&[1e-13, 1.0, 0.0]), [0.0, 0.0, 0.0, 1.0]);
    assert_eq!(guid::boresight_offset(&[1e-13, -1.0, 0.0]), [1.0, 0.0, 0.0, 0.0]);
}

#[test]
fn sun_pointing_with_the_roll_axis_on_the_sun_axis_falls_back_to_plus_y_made_normal() {
    let (r, v) = ([5.1e6, -3.2e6, 3.0e6], [2.9e3, 6.4e3, 1.8e3]);
    let rh = unit(&r);
    let nrm = unit(&cross(&unit(&v), &scale3(&rh, -1.0)));
    let sun = [0.3e11, 1.4e11, 0.6e11];
    let s = unit(&sun);
    let e2 = unit(&sub3(&nrm, &scale3(&s, dot(&nrm, &s))));
    let a = unit(&[0.3, 0.5, -0.8]);
    let perp = unit(&cross(&a, &[1.0, 0.0, 0.0]));
    let want_b = unit(&sub3(&[0.0, 1.0, 0.0], &scale3(&a, a[1])));
    // the roll axis within 1e-8 of the Sun axis: parallel for the guidance (threshold 1e-6)
    let roll = add3(&scale3(&a, 2.0), &scale3(&perp, 1e-8));
    let g = guid::guidance(4, &r, &v, 0.0, &Guid { sun_axis: a, roll_axis: roll, sun_eci: sun, ..Default::default() });
    let m = dcm(&g.q);
    assert!((det3(&m) - 1.0).abs() < 1e-12);
    near3(&mv(&m, &s), &a, 1.0, "the Sun on the axis");
    near3(&mv(&m, &e2), &want_b, 1.0, "+y made normal to the axis, toward the orbit normal");
}

/// An equatorial orbit on the x axis, where the nadir frame is exact in floating point: the body
/// sees an inertial x Sun with no z component at all.
fn equatorial() -> (V3, V3) { ([7.0e6, 0.0, 0.0], [0.0, 7.5e3, 0.0]) }

#[test]
fn the_yaw_flip_holds_its_state_with_the_sun_exactly_on_the_threshold() {
    let (r, v) = equatorial();
    for start in [false, true] {
        let mut g = Guid { sun_eci: [1.0, 0.0, 0.0], flip: start, ..Default::default() };
        guid::yaw_flip(&mut g, &r, &v, 0.0);
        assert_eq!(g.flip, start, "d = 0, hysteresis 0: neither d < -h nor d > h");
    }
}

#[test]
fn the_yaw_flip_uses_the_configured_power_face() {
    let (r, v) = equatorial();
    // nadir frame here: body -z on inertial +z (the orbit normal)
    let nad = guid::guidance(0, &r, &v, 0.0, &Guid::default()).q;
    assert!(norm3(&sub3(&mv(&dcm(&nad), &[0.0, 0.0, 1.0]), &[0.0, 0.0, -1.0])) < 1e-12);
    let sun = [0.3, 0.0, -0.95]; // below the orbit plane: on the body +z side
    let mut g = Guid { sun_eci: sun, ..Default::default() };
    guid::yaw_flip(&mut g, &r, &v, 0.1);
    assert!(g.flip, "the default -z face looks away from it");
    let mut g = Guid { sun_eci: sun, sun_axis: [0.0, 0.0, 2.0], flip: true, ..Default::default() };
    guid::yaw_flip(&mut g, &r, &v, 0.1);
    assert!(!g.flip, "a +z face already looks at it");
}
