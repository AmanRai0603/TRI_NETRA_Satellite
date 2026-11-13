//! S12.5b (E2; DR-nav-001): the truth star tracker sees the stars aberrated by the spacecraft's geometric velocity relative to
//! the solar-system barycentre (its orbital velocity plus the Earth's), to first order in v/c: a star at angle a from the velocity
//! is displaced by (|v|/c) sin a towards it, up to 1.2e-4 rad at 7.6 + 29.8 km/s. Before, the tracker took no velocity and the
//! displacement was 0.
use adcs_sim_core::comp::star_tracker::{Camera, Work};
use adcs_sim_core::gen::starcat;
use adcs_sim_core::la::*;
use adcs_sim_core::rng::Rng;
use adcs_sim_core::sensors::{star, St, StDesc, N_STARS};

const C: f64 = 299_792_458.0;

fn unit(v: V3) -> V3 { let n = norm(&v); [v[0]/n, v[1]/n, v[2]/n] }

/// A deterministic spread of directions over the sphere.
fn directions(n: usize) -> Vec<V3> {
    (0..n).map(|k| { let z = 1.0 - 2.0*(k as f64 + 0.5)/n as f64; let a = 2.399963229728653*k as f64; let r = (1.0 - z*z).sqrt(); [r*a.cos(), r*a.sin(), z] }).collect()
}

#[test]
fn a_star_appears_displaced_by_v_over_c_sin_the_angle_towards_the_velocity() {
    // 7.6 km/s of orbit and 29.8 km/s of the Earth along one line, the worst case, and across it
    for v in [[37.4e3, 0.0, 0.0], [7.6e3, 29.8e3, 0.0], [-20.0e3, 5.0e3, 28.0e3]] {
        let (vn, mut worst): (f64, f64) = (norm(&v), 0.0);
        for s in directions(2000) {
            let p = starcat::st_aberrate(s, v);
            assert!((norm(&p) - 1.0).abs() < 1e-15, "the apparent direction is a unit vector");
            let d = sub(&p, &s);
            let sin_a = norm(&cross(&s, &unit(v)));
            let want = vn/C*sin_a;
            // to first order: the displacement is (v/c) sin a, and it points towards the velocity
            assert!((norm(&d) - want).abs() < 1e-8*vn/C + 1e-12, "star {s:?}: displaced {:e}, (v/c) sin a = {want:e}", norm(&d));
            assert!(dot(&d, &v) >= -1e-18, "towards the velocity");
            worst = worst.max(norm(&d));
        }
        eprintln!("v = {v:?}: the largest displacement of 2000 stars {worst:.4e} rad (|v|/c {:.4e})", vn/C);
        assert!((worst - vn/C).abs() < 1e-3*vn/C, "the largest displacement is |v|/c");
    }
    let worst = 37.4e3/C;
    assert!(worst > 1.24e-4 && worst < 1.25e-4, "the figure of the inventory, 1.24e-4 rad: {worst:e}");
    assert_eq!(starcat::st_aberrate([0.0, 0.0, 1.0], [0.0; 3]), [0.0, 0.0, 1.0], "no velocity, no displacement");
}

fn tracker(model: u8) -> St {
    // no noise, no mount error: the one difference between two samples is the velocity
    let mut d = StDesc { fitted: true, nh: 1, noise_cross: 0.0, noise_roll: 0.0, rate_hz: 5.0, latency: 0.0, max_rate: 1.0,
        sun_excl: 0.0, earth_excl: 0.0, fov: 0.17, model, moon_excl: 0.0, blind_s: 0.0, noise_rate_ref: 0.01, ..Default::default() };
    d.bs[0] = [0.0, 0.0, 1.0];
    St::new(d, &mut Rng::new(1, 1), Rng::new(1, 3))
}

fn attitude(s: &mut St, q: &Q, vb: &V3) -> Q {
    s.history(0.0, q);
    let far = [0.0, 0.0, -1.0];      // the Sun, the Moon and the Earth behind the tracker, nothing in its way
    s.sample(q, 0.0, &[0.0; 3], &far, &far, &far, 0.01, vb)[0].expect("a tracker with a clear sky answers")
}

#[test]
fn the_attitude_the_tracker_reports_is_the_true_one_turned_by_the_boresight_crossed_with_v_over_c() {
    // the QUEST model fits the aberrated stars to the catalogue's geometric directions; the noise model turns the attitude by the
    // rotation the displacement is equivalent to. Both: |turn| = (|v|/c) sin(angle between the boresight and v), about axis
    // u x v, the field's radial scale term turning the fit by nothing
    let v = [7.6e3, 29.8e3, 0.0];
    for model in [0u8, 1] {
        let mut worst_abs: f64 = 0.0;
        for (i, qa) in [[0.0f64, 0.0, 0.0, 1.0], [0.3, -0.2, 0.5, 0.78], [-0.6, 0.1, 0.2, 0.75], [0.1, 0.9, -0.3, 0.3]].iter().enumerate() {
            let q = { let n = (qa[0]*qa[0] + qa[1]*qa[1] + qa[2]*qa[2] + qa[3]*qa[3]).sqrt(); [qa[0]/n, qa[1]/n, qa[2]/n, qa[3]/n] };
            let geometric = attitude(&mut tracker(model), &q, &[0.0; 3]);
            let seen = attitude(&mut tracker(model), &q, &v);
            let d = qmult(&seen, &qconj(&geometric));
            let turn = 2.0*(d[0]*d[0] + d[1]*d[1] + d[2]*d[2]).sqrt();      // the angle from the vector part: no acos near 1
            // the boresight in the inertial frame, from the attitude
            let a = dcm(&q);
            let u = [a[2][0], a[2][1], a[2][2]];       // A^T times the body z axis: the third row of the ECI-to-body matrix
            let want = norm(&cross(&u, &v))/C;
            eprintln!("model {model}, attitude {i}: the reported attitude is {turn:.4e} rad from the geometric one, (|v|/c) sin a = {want:.4e}");
            if model == 0 { assert!((turn - want).abs() < 1e-9*want + 1e-14, "the noise model turns by (v/c) sin a"); }
            worst_abs = worst_abs.max((turn - want).abs());
            assert!(turn > 0.0 || want < 1e-9, "the aberration changes what the tracker reports");
        }
        // the QUEST fit of 12 scattered stars turns by (v/c) sin a to the scatter of the field: a few per cent of 1.2e-4 rad
        if model == 1 { assert!(worst_abs < 7e-6, "the QUEST fit of the aberrated field turns by (v/c) sin a to {worst_abs:e} rad"); }
    }
}

#[test]
fn the_rendered_frame_shows_the_stars_where_the_velocity_puts_them() {
    // the image model: each catalogue star is projected from its apparent direction. A star's pixel position moves by the focal
    // length times the displacement across the boresight: up to f (|v|/c) with f = 512/tan(0.17) = 2,970 px, 0.37 px at 37.4 km/s
    use adcs_sim_core::comp::star_tracker as stc;
    let cam = Camera::new(0.17, 1024, 1.2, 3000.0, 50.0, 8.0, 5.0, 20, 2e-4, 0.25, 1e-4);
    let (mut cr, mut cm) = (Box::new([[0.0; 3]; N_STARS]), Box::new([0.0; N_STARS]));
    for k in 0..N_STARS { (cr[k], cm[k]) = star(k, N_STARS); }
    let mut img = vec![0.0; cam.n*cam.n];
    let r = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    let v = [37.4e3*0.6, 37.4e3*0.8, 0.0];
    let (mut t0, mut t1) = ([(0.0, 0.0, 0usize); 128], [(0.0, 0.0, 0usize); 128]);
    let n0 = stc::render(&r, &mut cr, &mut cm, &cam, &mut img, None, &mut t0, &[0.0; 3]);
    let n1 = stc::render(&r, &mut cr, &mut cm, &cam, &mut img, None, &mut t1, &v);
    assert!(n0 >= 10 && n1 >= 10);
    let f = 512.0/0.17f64.tan();
    let (mut worst, mut compared): (f64, usize) = (0.0, 0);
    for &(x0, y0, k0) in &t0[..n0.min(128)] {
        if let Some(&(x1, y1, _)) = t1[..n1.min(128)].iter().find(|t| t.2 == k0) {
            let dpx = ((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt();
            // across the boresight (z) the shift is the velocity's x, y part over c, the focal length times it, to the radial term
            let across = (v[0]*v[0] + v[1]*v[1]).sqrt()/C*f;
            assert!(dpx < across*1.05 + 0.02, "star {k0} moved {dpx} px, at most {across} px");
            worst = worst.max(dpx); compared += 1;
        }
    }
    eprintln!("{compared} stars compared: the largest shift {worst:.3} px of {:.3} px", (v[0]*v[0] + v[1]*v[1]).sqrt()/C*f);
    assert!(compared >= 8 && worst > 0.3, "the stars of a frame move by the aberration: {worst} px");
}
