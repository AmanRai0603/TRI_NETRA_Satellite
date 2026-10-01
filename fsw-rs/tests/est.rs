//! The estimators against their own definitions (fsw/pseudocode/03_estimation.md), written out
//! here independently: the MEKF's propagation and Joseph update in closed form, the gate, the
//! star-tracker noise shape, TRIAD's and the q-method's answers and losses, the latency hop.
//! These are what mutation testing found the flight tests alone did not pin down.
#![allow(clippy::needless_range_loop)]
use adcs_fsw::est::{self, Mekf};
use adcs_fsw::math::*;

type M6 = [[f64; 6]; 6];

fn close(a: f64, b: f64, tol: f64) -> bool { (a - b).abs() <= tol*(1.0 + a.abs().max(b.abs())) }
fn mm6(a: &M6, b: &M6) -> M6 { let mut c = [[0.0; 6]; 6]; for i in 0..6 { for j in 0..6 { for l in 0..6 { c[i][j] += a[i][l]*b[l][j]; } } } c }
fn tr6(a: &M6) -> M6 { let mut c = [[0.0; 6]; 6]; for i in 0..6 { for j in 0..6 { c[i][j] = a[j][i]; } } c }
fn rot(axis: &V3, deg: f64) -> Q { fromrotvec(&scale3(&unit(axis), deg.to_radians())) }

/// A full, symmetric, positive covariance, so that every entry of the propagation matters.
fn covariance() -> M6 {
    let mut a = [[0.0; 6]; 6];
    for i in 0..6 { for j in 0..6 { a[i][j] = (0.37*(i*6 + j) as f64 + 0.11).sin()*if j < 3 { 0.05 } else { 1e-3 }; } }
    let mut p = mm6(&tr6(&a), &a);
    for i in 0..6 { p[i][i] += if i < 3 { 1e-3 } else { 1e-7 }; }
    p
}

#[test]
fn propagation_is_phi_p_phi_t_plus_the_discrete_noise() {
    let (arw, rrw, dt) = (3e-4, 2e-5, 0.2);
    let mut k = Mekf::new(&rot(&[1.0, 2.0, 3.0], 20.0), 0.1, 1e-3, arw, rrw);
    k.b = [1e-3, -2e-3, 5e-4];
    k.p = covariance();
    let (q0, p0) = (k.q, k.p);
    let wm = [0.05, -0.08, 0.03];
    k.predict(&wm, dt);

    // the attitude turns by the measured rate less the bias
    let w = sub3(&wm, &k.b);
    let want_q = qnorm(&qmult(&q0, &fromrotvec(&scale3(&w, dt))));
    for i in 0..4 { assert!((k.q[i] - want_q[i]).abs() < 1e-15, "q[{i}]"); }
    // Phi = [[I - [w]dt + [w]^2 dt^2/2, -I dt], [0, I]]
    let s = skew(&w);
    let s2 = mat3_mul(&s, &s);
    let mut phi = [[0.0; 6]; 6];
    for i in 0..3 {
        for j in 0..3 { phi[i][j] = (i == j) as u8 as f64 - s[i][j]*dt + 0.5*s2[i][j]*dt*dt; }
        phi[i][i + 3] = -dt;
        phi[i + 3][i + 3] = 1.0;
    }
    let mut want = mm6(&mm6(&phi, &p0), &tr6(&phi));
    let (sv2, su2) = (arw*arw, rrw*rrw);
    for i in 0..3 {
        want[i][i] += sv2*dt + su2*dt.powi(3)/3.0;
        want[i][i + 3] -= su2*dt*dt/2.0;
        want[i + 3][i] -= su2*dt*dt/2.0;
        want[i + 3][i + 3] += su2*dt;
    }
    for i in 0..6 { for j in 0..6 { assert!(close(k.p[i][j], want[i][j], 1e-12), "P[{i}][{j}] {} vs {}", k.p[i][j], want[i][j]); } }
}

#[test]
fn a_constant_rate_integrates_to_its_rotation() {
    let mut k = Mekf::new(&[0.0, 0.0, 0.0, 1.0], 0.1, 1e-3, 0.0, 0.0);
    k.b = [0.0, 0.0, 0.01];
    for _ in 0..100 { k.predict(&[0.0, 0.0, 0.11], 0.1); }
    assert!(qangle(&k.q, &fromrotvec(&[0.0, 0.0, 1.0])) < 1e-12, "0.1 rad/s for 10 s about z is 1 rad");
    assert!(close(norm3(&[k.q[0], k.q[1], k.q[2]]).hypot(k.q[3]), 1.0, 1e-15));
}

#[test]
fn a_vector_update_is_the_kalman_gain_in_closed_form() {
    // reference z, attitude identity: the measurement sees rotations about x and y, not z
    let (sa, sb, sig) = (0.1, 1e-3, 0.02);
    let mut k = Mekf::new(&[0.0, 0.0, 0.0, 1.0], sa, sb, 0.0, 0.0);
    let eps = 1e-4f64;
    let truth = rot(&[1.0, 0.0, 0.0], eps.to_degrees());
    let r = [0.0, 0.0, 1.0];
    let b = mat3_vec(&dcm(&truth), &r);
    assert!(k.vector(&b, &r, sig, 0.0));
    let gain = sa*sa/(sa*sa + sig*sig);
    let post = sa*sa*sig*sig/(sa*sa + sig*sig);
    assert!(close(k.p[0][0], post, 1e-9) && close(k.p[1][1], post, 1e-9), "x and y shrink: {} {}", k.p[0][0], k.p[1][1]);
    assert!(close(k.p[2][2], sa*sa, 1e-12), "z is unseen: {}", k.p[2][2]);
    for i in 3..6 { assert!(close(k.p[i][i], sb*sb, 1e-12), "the bias is uncorrelated, so unchanged"); }
    assert!(k.b.iter().all(|x| x.abs() < 1e-18));
    let moved = 2.0*k.q[0].atan2(k.q[3]);
    assert!(close(moved, gain*eps, 1e-6), "the estimate moves by K y about x: {moved} vs {}", gain*eps);
    assert!(k.q[1].abs() < 1e-15 && k.q[2].abs() < 1e-15);
}

#[test]
fn the_gate_rejects_an_innovation_beyond_it_and_leaves_the_state() {
    let mut k = Mekf::new(&[0.0, 0.0, 0.0, 1.0], 0.01, 1e-3, 0.0, 0.0);
    let r = [0.0, 0.0, 1.0];
    let far = mat3_vec(&dcm(&rot(&[1.0, 0.0, 0.0], 10.0)), &r);
    let before = k;
    assert!(!k.vector(&far, &r, 0.01, 9.0), "a 10 deg miss with 0.6 deg uncertainty is out of the gate");
    assert_eq!((k.q, k.b, k.p), (before.q, before.b, before.p));
    assert!(k.vector(&far, &r, 0.01, 0.0), "no gate: taken");
    // chi^2 of y'S^-1 y for the 0.01 rad miss with S = (0.01^2 + 0.01^2) I is about 0.5
    let mut k = before;
    let near = mat3_vec(&dcm(&rot(&[1.0, 0.0, 0.0], 0.01f64.to_degrees())), &r);
    assert!(k.vector(&near, &r, 0.01, 0.6) && { let mut k2 = before; !k2.vector(&near, &r, 0.01, 0.4) }, "the gate sits at chi^2 = 0.5");
    let mut k = before;
    assert!(!k.vector(&[f64::NAN, 0.0, 1.0], &r, 0.01, 0.0), "a measurement that is not a number is refused");
    assert_eq!(k.q, before.q);
}

#[test]
fn a_star_tracker_update_trusts_cross_boresight_more_than_roll() {
    let (sa, sc, sr) = (0.1, 1e-4, 1e-2);
    let bs = [0.0, 0.0, 1.0];
    let eps = 1e-3f64;
    for (axis, sig) in [([1.0, 0.0, 0.0], sc), ([0.0, 0.0, 1.0], sr)] {
        let mut k = Mekf::new(&[0.0, 0.0, 0.0, 1.0], sa, 1e-3, 0.0, 0.0);
        assert!(k.quat(&rot(&axis, eps.to_degrees()), sc, sr, &bs, 0.0));
        let moved = 2.0*norm3(&[k.q[0], k.q[1], k.q[2]]);
        let gain = sa*sa/(sa*sa + sig*sig);
        assert!(close(moved, gain*eps, 1e-6), "about {axis:?}: moved {moved}, want {}", gain*eps);
        let i = if axis[0] == 1.0 { 0 } else { 2 };
        assert!(close(k.p[i][i], sa*sa*sig*sig/(sa*sa + sig*sig), 1e-9), "about {axis:?}: P {}", k.p[i][i]);
    }
}

#[test]
fn the_filter_learns_the_gyro_bias() {
    let bias = [2e-4, -3e-4, 1e-4];
    let wt = [0.01, -0.02, 0.015];
    let mut truth = rot(&[1.0, 1.0, 0.0], 30.0);
    let mut k = Mekf::new(&qmult(&truth, &rot(&[0.0, 1.0, 1.0], 3.0)), 0.1, 1e-3, 1e-6, 1e-8);
    let (r1, r2) = ([1.0, 0.0, 0.0], unit(&[0.3, 1.0, -0.2]));
    for _ in 0..3000 {
        truth = qnorm(&qmult(&truth, &fromrotvec(&scale3(&wt, 0.1))));
        k.predict(&add3(&wt, &bias), 0.1);
        assert!(k.vector(&mat3_vec(&dcm(&truth), &r1), &r1, 1e-3, 0.0) && k.vector(&mat3_vec(&dcm(&truth), &r2), &r2, 1e-3, 0.0));
    }
    assert!(qangle(&truth, &k.q).to_degrees() < 1e-3, "attitude {} deg", qangle(&truth, &k.q).to_degrees());
    for i in 0..3 { assert!((k.b[i] - bias[i]).abs() < 1e-6, "bias {i}: {} vs {}", k.b[i], bias[i]); }
    for i in 0..6 { for j in 0..6 { assert!(close(k.p[i][j], k.p[j][i], 1e-9), "P stays symmetric"); } assert!(k.p[i][i] > 0.0); }
}

#[test]
fn triad_refuses_pairs_closer_than_its_angle_whatever_their_length() {
    let x = [100.0, 0.0, 0.0];
    let tilt = |s: f64| scale3(&[(1.0 - s*s).sqrt(), s, 0.0], 100.0);
    let r = ([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    assert!(est::triad(&x, &tilt(5e-4), &r.0, &r.1).is_none(), "0.03 deg apart in the body");
    assert!(est::triad(&x, &tilt(2e-3), &r.0, &r.1).is_some());
    assert!(est::triad(&r.0, &r.1, &x, &tilt(5e-4)).is_none(), "0.03 deg apart in the reference");
    assert!(est::triad(&r.0, &r.1, &x, &tilt(2e-3)).is_some());
}

#[test]
fn the_q_method_weighs_its_pairs_and_reports_its_loss() {
    let q = rot(&[0.2, -1.0, 0.5], 70.0);
    let r = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], unit(&[1.0, 1.0, 1.0])];
    let b: Vec<V3> = r.iter().map(|v| mat3_vec(&dcm(&q), v)).collect();
    let (qm, loss) = est::quest(&b, &r, Some(&[0.5, 2.0, 1.0]));
    assert!(qangle(&q, &qm) < 1e-10 && loss.abs() < 1e-10, "consistent pairs: the truth and no loss ({loss})");
    // the angle between the pairs off by a: the loss of two unit pairs weighted w1, w2 is w1 + w2 - sqrt(w1^2 + w2^2 + 2 w1 w2 cos a)
    let a = 0.1f64;
    let b2 = [b[0], mat3_vec(&dcm(&rot(&cross(&b[0], &b[1]), a.to_degrees())), &b[1])];
    let r2 = [r[0], r[1]];
    for (w1, w2) in [(1.0, 1.0), (1.0, 3.0)] {
        let (_, l) = est::quest(&b2, &r2, Some(&[w1, w2]));
        let want = w1 + w2 - (w1*w1 + w2*w2 + 2.0*w1*w2*a.cos()).sqrt();
        assert!(close(l, want, 1e-9), "weights {w1} {w2}: loss {l} vs {want}");
    }
    let (_, l1) = est::quest(&b2, &r2, None);
    assert!(close(l1, 2.0 - (2.0 + 2.0*a.cos()).sqrt(), 1e-9), "no weights is weights of 1");
    // the heavier pair wins: with w2 >> w1 the answer fits pair 2
    let (qh, _) = est::quest(&b2, &r2, Some(&[1e-6, 1.0]));
    assert!(norm3(&sub3(&mat3_vec(&dcm(&qh), &r2[1]), &b2[1])) < 1e-5);
}

#[test]
fn latency_carries_the_attitude_forward_by_the_rate() {
    let q = rot(&[1.0, 2.0, -1.0], 40.0);
    let w = [0.0, 0.0, 0.5];
    let q2 = est::latency(&q, &w, 0.2);
    assert!(close(qangle(&q, &q2), 0.1, 1e-12), "0.5 rad/s for 0.2 s");
    assert!(qangle(&q2, &qmult(&q, &fromrotvec(&[0.0, 0.0, 0.1]))) < 1e-15, "about the body axis of the rate");
    assert_eq!(est::latency(&q, &w, 0.0), qnorm(&q));
}
