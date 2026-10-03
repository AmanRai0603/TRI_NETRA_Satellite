//! Guidance (fsw/pseudocode/04_guidance.md) against what each reference frame is defined to be:
//! which body axis looks where, how fast the frame turns, and that the slew's feed-forward
//! acceleration is the derivative of its rate. Written from the definitions, not from a run.
#![allow(clippy::needless_range_loop)]
use adcs_fsw::ctl::{self, Guid};
use adcs_fsw::math::*;

fn near(a: &V3, b: &V3, tol: f64) -> bool { norm3(&sub3(a, b)) <= tol }
fn body(q: &Q, v: &V3) -> V3 { mat3_vec(&dcm(q), v) }

/// An inclined, not quite circular orbit point, so no axis lines up by accident.
fn orbit() -> (V3, V3) { ([5.1e6, -3.2e6, 3.0e6], [2.9e3, 6.4e3, 1.8e3]) }

fn frame(r: &V3, v: &V3) -> (V3, V3, V3, V3) {
    let rh = unit(r);
    let nrm = unit(&cross(&unit(v), &scale3(&rh, -1.0)));
    let ram = unit(&cross(&scale3(&rh, -1.0), &nrm));
    (rh, nrm, ram, scale3(&cross(r, v), 1.0/dot(r, r)))
}

#[test]
fn nadir_puts_plus_y_on_the_earth_and_turns_with_the_orbit() {
    let (r, v) = orbit();
    let (rh, nrm, ram, w_orb) = frame(&r, &v);
    let g = ctl::guidance(0, &r, &v, 0.0, &Guid::default());
    assert!(near(&body(&g.q, &scale3(&rh, -1.0)), &[0.0, 1.0, 0.0], 1e-12), "+y to nadir");
    assert!(near(&body(&g.q, &ram), &[-1.0, 0.0, 0.0], 1e-12), "-x along the ram");
    assert!(near(&body(&g.q, &nrm), &[0.0, 0.0, -1.0], 1e-12), "-z along the orbit normal");
    assert!(near(&g.w, &body(&g.q, &w_orb), 1e-18) && g.wd == [0.0; 3]);
    // an offset is applied in the body
    let off = fromrotvec(&[0.1, -0.2, 0.05]);
    let go = ctl::guidance(0, &r, &v, 0.0, &Guid { q_off: off, ..Default::default() });
    assert!(qangle(&go.q, &qmult(&g.q, &off)) < 1e-7);
    assert!(near(&go.w, &body(&go.q, &w_orb), 1e-18), "the rate follows the offset frame");
}

#[test]
fn the_yaw_flip_turns_half_a_turn_about_the_boresight() {
    let (r, v) = orbit();
    let (rh, _, ram, _) = frame(&r, &v);
    let flipped = ctl::guidance(0, &r, &v, 0.0, &Guid { flip: true, roll_axis: [0.0, 2.0, 0.0], ..Default::default() });
    assert!(near(&body(&flipped.q, &scale3(&rh, -1.0)), &[0.0, 1.0, 0.0], 1e-12), "the boresight still on nadir");
    assert!(near(&body(&flipped.q, &ram), &[1.0, 0.0, 0.0], 1e-12), "+x now along the ram");
    let dflt = ctl::guidance(0, &r, &v, 0.0, &Guid { flip: true, ..Default::default() });
    assert!(near(&body(&dflt.q, &ram), &[-1.0, 0.0, 0.0], 1e-12) && near(&body(&dflt.q, &scale3(&rh, -1.0)), &[0.0, -1.0, 0.0], 1e-12),
            "no roll axis given: about x");
}

#[test]
fn target_pointing_rolls_about_its_axis_from_nadir() {
    let (r, v) = orbit();
    let (.., w_orb) = frame(&r, &v);
    let nad = ctl::guidance(0, &r, &v, 0.0, &Guid::default()).q;
    let g = Guid { roll_deg: 25.0, axis: [0.0, 0.0, 3.0], ..Default::default() };
    let t = ctl::guidance(1, &r, &v, 0.0, &g);
    assert!(qangle(&t.q, &qmult(&nad, &fromrotvec(&[0.0, 0.0, 25f64.to_radians()]))) < 1e-7);
    assert!(near(&t.w, &body(&t.q, &w_orb), 1e-18) && t.wd == [0.0; 3]);
    let x = ctl::guidance(1, &r, &v, 0.0, &Guid { roll_deg: 25.0, ..Default::default() }).q;
    assert!(qangle(&x, &qmult(&nad, &fromrotvec(&[25f64.to_radians(), 0.0, 0.0]))) < 1e-7, "no axis given: x");
}

#[test]
fn a_slew_goes_from_nadir_to_its_angle_and_its_acceleration_is_the_rate_derivative() {
    let (r, v) = orbit();
    let (.., w_orb) = frame(&r, &v);
    let nad = ctl::guidance(0, &r, &v, 0.0, &Guid::default()).q;
    let ax = unit(&[1.0, -1.0, 2.0]);
    let g = Guid { roll_deg: 40.0, axis: ax, t0: 100.0, t_slew: 60.0, ..Default::default() };
    let at = |t: f64| ctl::guidance(2, &r, &v, t, &g);
    let ph = 40f64.to_radians();
    for (t, frac) in [(50.0, 0.0), (100.0, 0.0), (130.0, 0.5), (160.0, 1.0), (300.0, 1.0)] {
        let s = at(t);
        assert!(qangle(&s.q, &qmult(&nad, &fromrotvec(&scale3(&ax, ph*frac)))) < 1e-7, "t = {t}: {frac} of the way");
    }
    for t in [50.0, 100.0, 160.0, 300.0] {
        let s = at(t);
        assert!(near(&s.w, &body(&s.q, &w_orb), 1e-18) && s.wd == [0.0; 3], "t = {t}: at rest in the orbiting frame");
    }
    // halfway the profile's rate peaks at 2/T
    let m = at(130.0);
    assert!(near(&sub3(&m.w, &body(&m.q, &w_orb)), &scale3(&ax, ph*2.0/60.0), 1e-15));
    // the feed-forward is d/dt of the reference rate, transport term included
    for t in [107.0, 118.5, 130.0, 141.0, 155.0] {
        let h = 1e-3;
        let (a, b) = (at(t - h).w, at(t + h).w);
        let fd = scale3(&sub3(&b, &a), 0.5/h);
        let wd = at(t).wd;
        assert!(norm3(&sub3(&fd, &wd)) < 1e-8*(1.0 + norm3(&wd)), "t = {t}: wd {wd:?}, finite difference {fd:?}");
    }
}

#[test]
fn inertial_holds_its_quaternion_at_rest() {
    let (r, v) = orbit();
    let q = qnorm(&[0.1, 0.2, -0.3, 0.9]);
    let g = ctl::guidance(3, &r, &v, 5.0, &Guid { q_inertial: q, ..Default::default() });
    assert_eq!((g.q, g.w, g.wd), (q, [0.0; 3], [0.0; 3]));
}

#[test]
fn sun_pointing_puts_its_axis_on_the_sun_and_its_roll_axis_toward_the_orbit_normal() {
    let (r, v) = orbit();
    let (_, nrm, ..) = frame(&r, &v);
    let sun = [0.3e11, 1.4e11, 0.6e11];
    let s = unit(&sun);
    let e2 = unit(&sub3(&nrm, &scale3(&s, dot(&nrm, &s))));
    let a = unit(&[0.0, 1.0, 1.0]);
    let g = ctl::guidance(4, &r, &v, 0.0, &Guid { sun_axis: scale3(&a, 3.0), roll_axis: [1.0, 0.0, 0.5], sun_eci: sun, ..Default::default() });
    assert!(near(&body(&g.q, &s), &a, 1e-12), "the Sun on the axis");
    let b = unit(&sub3(&[1.0, 0.0, 0.5], &scale3(&a, dot(&a, &[1.0, 0.0, 0.5]))));
    assert!(near(&body(&g.q, &e2), &b, 1e-12), "the roll axis, made square to it, toward the orbit normal");
    assert!(g.w == [0.0; 3] && g.wd == [0.0; 3]);
    // defaults: -z on the Sun, x toward the normal
    let d = ctl::guidance(4, &r, &v, 0.0, &Guid { sun_eci: sun, ..Default::default() });
    assert!(near(&body(&d.q, &s), &[0.0, 0.0, -1.0], 1e-12) && near(&body(&d.q, &e2), &[1.0, 0.0, 0.0], 1e-12));
    // a roll axis along the Sun axis, and a Sun along the orbit normal, still give a proper rotation
    for g in [Guid { sun_axis: [0.0, 0.0, 1.0], roll_axis: [0.0, 0.0, 2.0], sun_eci: sun, ..Default::default() },
              Guid { sun_eci: scale3(&nrm, 1.5e11), ..Default::default() }] {
        let q = ctl::guidance(4, &r, &v, 0.0, &g).q;
        let a = if norm3(&g.sun_axis) > 0.0 { g.sun_axis } else { [0.0, 0.0, -1.0] };
        assert!(near(&body(&q, &unit(&g.sun_eci)), &a, 1e-9) && (det3(&dcm(&q)) - 1.0).abs() < 1e-12, "{g:?}");
    }
}

#[test]
fn the_yaw_flip_switches_with_hysteresis() {
    let (r, v) = orbit();
    let nad = ctl::guidance(0, &r, &v, 0.0, &Guid::default()).q;
    // the power face is -z; place the Sun at a chosen body elevation over it
    let sun_at = |d: f64| mat3t_vec(&dcm(&nad), &[0.0, (1.0 - d*d).sqrt(), -d]);
    let mut g = Guid::default();
    for (d, want) in [(0.05, false), (-0.05, false), (-0.2, true), (-0.05, true), (0.05, true), (0.2, false), (0.0, false)] {
        g.sun_eci = sun_at(d);
        ctl::yaw_flip(&mut g, &r, &v, 0.1);
        assert_eq!(g.flip, want, "Sun at {d} over the power face");
    }
}
