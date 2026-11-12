//! P4: the six integrators and the Hermite dense output against analytic references.
//! The integrators are the toolbox (`src/integ.rs`); no design is involved, so every oracle here is a
//! published closed form, not the code's own output.
//!
//! References
//! - Kepler two-body motion in closed form: Vallado, Fundamentals of Astrodynamics and Applications, 4th ed.,
//!   Algorithm 2 (Kepler's equation, M = E - e sin E) and Section 2.2 (perifocal r, v).
//! - Order of a Runge-Kutta method, global error ~ h^p: Hairer, Norsett, Wanner, Solving Ordinary Differential
//!   Equations I, Section II.3 (Theorem 3.1).
//! - J2 perturbing acceleration: Vallado 4th ed., Eq. 8-30 (Section 8.6.2).
//! - Cubic Hermite interpolation error h^4/384 max|y''''|: Burden and Faires, Numerical Analysis, Section 3.4.
use adcs_pop::integ::{self, Opts, Sol, Y};

const MU: f64 = 3.986004418e14;
const A: f64 = 7.0e6;
const E: f64 = 0.1;

fn n_mean() -> f64 { (MU/(A*A*A)).sqrt() }
fn period() -> f64 { 2.0*std::f64::consts::PI/n_mean() }

/// Closed-form Kepler state at time t after periapsis (perifocal frame, motion in the x-y plane).
fn kepler(t: f64) -> Y {
    let m = n_mean()*t;
    let mut ea = m;
    for _ in 0..100 { ea -= (ea - E*ea.sin() - m)/(1.0 - E*ea.cos()); }
    let (se, ce) = ea.sin_cos();
    let r = A*(1.0 - E*ce);
    let k = (MU*A).sqrt()/r;
    [A*(ce - E), A*(1.0 - E*E).sqrt()*se, 0.0, -k*se, k*(1.0 - E*E).sqrt()*ce, 0.0]
}

fn two_body(_t: f64, y: &Y) -> Y {
    let r = (y[0]*y[0] + y[1]*y[1] + y[2]*y[2]).sqrt();
    let k = -MU/(r*r*r);
    [y[3], y[4], y[5], k*y[0], k*y[1], k*y[2]]
}

fn pos_err(a: &Y, b: &Y) -> f64 { ((a[0]-b[0]).powi(2) + (a[1]-b[1]).powi(2) + (a[2]-b[2]).powi(2)).sqrt() }
fn last(s: &Sol) -> Y { *s.y.last().unwrap() }

fn fixed_h(h: f64) -> Opts {
    // an accept-everything adaptive run with the step pinned to h: the method's own order shows
    Opts { h, rtol: 1e30, atol: 1e30, hmax: Some(h), hmin: h, h0: Some(h), facmin: 1.0, facmax: 1.0, ..Opts::default() }
}

#[test]
fn rk78_matches_kepler_closed_form_within_1_mm() {
    let t = period();
    let o = Opts { rtol: 1e-13, atol: 1e-13, ..Opts::default() };
    let s = integ::rk78(two_body, 0.0, t, kepler(0.0), &o).unwrap();
    let e = pos_err(&last(&s), &kepler(t));
    eprintln!("rk78 one period: {e:.3e} m");
    assert!(e < 1e-3, "rk78 position error {e} m");
    let mut worst = 0.0f64;
    for (tk, yk) in s.t.iter().zip(&s.y) { worst = worst.max(pos_err(yk, &kepler(*tk))); }
    eprintln!("rk78 worst node: {worst:.3e} m");
    assert!(worst < 1e-3, "rk78 worst node error {worst} m");
}

#[test]
fn rk4_global_error_ratio_is_16() {
    let t = (period()/4.0/20.0).round()*20.0;
    let r = |h: f64| last(&integ::rk4(two_body, 0.0, t, kepler(0.0), h));
    let (e1, e2) = (pos_err(&r(20.0), &kepler(t)), pos_err(&r(10.0), &kepler(t)));
    eprintln!("rk4 ratio {:.2} ({e1:.3e} -> {e2:.3e})", e1/e2);
    assert!((e1/e2 - 16.0).abs() < 1.0, "ratio {}", e1/e2);
}

#[test]
fn nystrom4_is_fourth_order() {
    let t = (period()/4.0/20.0).round()*20.0;
    let r = |h: f64| last(&integ::nystrom4(two_body, 0.0, t, kepler(0.0), h));
    let (e1, e2) = (pos_err(&r(20.0), &kepler(t)), pos_err(&r(10.0), &kepler(t)));
    eprintln!("nystrom4 order {:.2} ({e1:.3e} -> {e2:.3e})", (e1/e2).log2());
    // fourth order for a general y'' = f(t, y, y'); for the force-only y'' = f(y) of two-body the Nystrom
    // conditions are weaker (Hairer, Norsett, Wanner I, Section II.14), so the position converges at order 5.
    // Never below 4: a wrong coefficient drops the order.
    assert!((e1/e2).log2() > 3.8 && (e1/e2).log2() < 5.3, "order {}", (e1/e2).log2());
}

#[test]
fn rk6_luther_is_sixth_order() {
    let t = (period()/4.0/30.0).round()*30.0;
    let r = |h: f64| last(&integ::rk6luther(two_body, 0.0, t, kepler(0.0), h));
    let (e1, e2) = (pos_err(&r(30.0), &kepler(t)), pos_err(&r(15.0), &kepler(t)));
    eprintln!("rk6 order {:.2} ({e1:.3e} -> {e2:.3e})", (e1/e2).log2());
    assert!((e1/e2).log2() > 5.8 && (e1/e2).log2() < 6.2, "order {}", (e1/e2).log2());
}

#[test]
fn rk45_is_fifth_order_at_pinned_step() {
    // local error h^6 per step, global h^5; the pinned run takes the 5th-order solution each step
    let t = (period()/4.0/40.0).round()*40.0;
    let r = |h: f64| last(&integ::rk45(two_body, 0.0, t, kepler(0.0), &fixed_h(h)).unwrap());
    let (e1, e2) = (pos_err(&r(40.0), &kepler(t)), pos_err(&r(20.0), &kepler(t)));
    eprintln!("rk45 order {:.2} ({e1:.3e} -> {e2:.3e})", (e1/e2).log2());
    assert!((e1/e2).log2() > 4.8 && (e1/e2).log2() < 5.3, "order {}", (e1/e2).log2());
}

#[test]
fn rk78_is_eighth_order_at_pinned_step() {
    // 1200 s = 4 x 300 s: coarse enough to stay above the 5e-9 m round-off floor at the finer step
    let t = 1200.0;
    let r = |h: f64| last(&integ::rk78(two_body, 0.0, t, kepler(0.0), &fixed_h(h)).unwrap());
    let (e1, e2) = (pos_err(&r(300.0), &kepler(t)), pos_err(&r(150.0), &kepler(t)));
    eprintln!("rk78 order {:.2} ({e1:.3e} -> {e2:.3e})", (e1/e2).log2());
    assert!((e1/e2).log2() > 7.7 && (e1/e2).log2() < 8.5, "order {}", (e1/e2).log2());
}

fn energy(y: &Y) -> f64 {
    let r = (y[0]*y[0] + y[1]*y[1] + y[2]*y[2]).sqrt();
    0.5*(y[3]*y[3] + y[4]*y[4] + y[5]*y[5]) - MU/r
}

#[test]
fn energy_drift_over_ten_orbits_below_1e_10() {
    let (t, e0) = (10.0*period(), energy(&kepler(0.0)));
    let o = Opts { rtol: 1e-13, atol: 1e-13, ..Opts::default() };
    let s = integ::rk78(two_body, 0.0, t, kepler(0.0), &o).unwrap();
    let d78 = s.y.iter().map(|y| ((energy(y) - e0)/e0).abs()).fold(0.0, f64::max);
    let g = integ::gauss_jackson8(two_body, 0.0, t, kepler(0.0), 30.0);
    let dgj = g.y.iter().map(|y| ((energy(y) - e0)/e0).abs()).fold(0.0, f64::max);
    eprintln!("energy drift: rk78 {d78:.2e}, gj8 {dgj:.2e}");
    assert!(d78 < 1e-10, "rk78 energy drift {d78}");
    assert!(dgj < 1e-10, "gauss-jackson 8 energy drift {dgj}");
}

/// Two-body plus J2, Vallado Eq. 8-30, with the J2 and Re of the test.
fn with_j2(t: f64, y: &Y) -> Y {
    const RE: f64 = 6.378137e6;
    const J2: f64 = 1.08262668e-3;
    let mut d = two_body(t, y);
    let r2 = y[0]*y[0] + y[1]*y[1] + y[2]*y[2];
    let r = r2.sqrt();
    let k = -1.5*J2*MU*RE*RE/(r2*r2);
    let z2 = y[2]*y[2]/r2;
    d[3] += k*y[0]/r*(1.0 - 5.0*z2);
    d[4] += k*y[1]/r*(1.0 - 5.0*z2);
    d[5] += k*y[2]/r*(3.0 - 5.0*z2);
    d
}

#[test]
fn gauss_jackson8_matches_rk78_on_a_j2_day_within_1_cm() {
    // tilt the orbit by 51.6 deg about x so J2 acts on every component
    let k = kepler(0.0);
    let (s, c) = (51.6f64.to_radians()).sin_cos();
    let y0: Y = [k[0], k[1]*c, k[1]*s, k[3], k[4]*c, k[4]*s];
    let day = 86400.0;
    let o = Opts { rtol: 1e-13, atol: 1e-13, ..Opts::default() };
    let r = integ::rk78(with_j2, 0.0, day, y0, &o).unwrap();
    let g = integ::gauss_jackson8(with_j2, 0.0, day, y0, 30.0);
    let e = pos_err(&last(&g), &last(&r));
    eprintln!("gj8 vs rk78 after a J2 day: {e:.3e} m");
    assert!(e < 1e-2, "gj8 vs rk78 {e} m");
}

#[test]
fn hermite_dense_output_matches_closed_form_between_nodes() {
    // nodes are exact Kepler states and derivatives; the cubic Hermite error is O(h^4)
    let make = |h: f64| {
        let n = (600.0/h).round() as usize;
        let mut s = Sol::default();
        for k in 0..=n { let t = k as f64*h; let y = kepler(t); s.t.push(t); s.y.push(y); s.d.push(two_body(t, &y)); }
        s
    };
    let worst = |h: f64| {
        let s = make(h);
        let mut w = 0.0f64;
        for k in 0..((600.0/h) as usize*7) {
            let tq = 3.0 + k as f64*(h/7.0)*0.99;
            if tq < 600.0 { w = w.max(pos_err(&integ::hermite(&s, tq), &kepler(tq))); }
        }
        let tn = 5.0*h;
        assert!(pos_err(&integ::hermite(&s, tn), &kepler(tn)) < 1e-9, "not exact at a node");
        w
    };
    let (e60, e30) = (worst(60.0), worst(30.0));
    eprintln!("hermite: h=60 {e60:.3e} m, h=30 {e30:.3e} m, ratio {:.2}", e60/e30);
    // the textbook bound h^4/384 max|y''''|; |y''''| is of order n^4 a (1+e)/(1-e)^3 at periapsis, 3x margin
    let bound = 60f64.powi(4)/384.0*n_mean().powi(4)*A*(1.0 + E)/(1.0 - E).powi(3)*3.0;
    assert!(e60 < bound, "hermite {e60} vs bound {bound}");
    assert!((e60/e30 - 16.0).abs() < 2.0, "hermite is cubic: ratio {}", e60/e30);
}

// ---- P8b: hermite finds its interval by a search, not a scan ----

/// The old hermite: every node scanned per query (the reference the search must equal).
fn hermite_linear(s: &integ::Sol, tq: f64) -> [f64; 6] {
    let n = s.t.len();
    let mut i = 0;
    for k in 0..n { if s.t[k] <= tq { i = k; } }
    let i = i.min(n - 2);
    let h = s.t[i + 1] - s.t[i];
    let u = (tq - s.t[i])/h;
    let (u2, u3) = (u*u, u*u*u);
    let (h00, h10, h01, h11) = (2.0*u3 - 3.0*u2 + 1.0, u3 - 2.0*u2 + u, -2.0*u3 + 3.0*u2, u3 - u2);
    let mut o = [0.0; 6];
    for c in 0..6 { o[c] = h00*s.y[i][c] + h10*h*s.d[i][c] + h01*s.y[i + 1][c] + h11*h*s.d[i + 1][c]; }
    o
}

fn lcg(state: &mut u64) -> f64 {
    *state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    (*state >> 11) as f64/(1u64 << 53) as f64
}

fn synthetic(n: usize, uneven: bool) -> integ::Sol {
    let mut st = 12345u64;
    let mut s = integ::Sol::default();
    let mut t = 0.0;
    for k in 0..n {
        s.t.push(t);
        let q = k as f64;
        s.y.push([q.sin(), q.cos(), 0.1*q, 1.0, -q, 2.0]);
        s.d.push([q.cos(), -q.sin(), 0.1, 0.0, -1.0, 0.5]);
        t += if uneven { 0.5 + lcg(&mut st) } else { 60.0 };
    }
    s
}

#[test]
fn hermite_bracket_probes_are_logarithmic() {
    let n = 100_000;
    let s = synthetic(n, true);
    let t_end = s.t[n - 1];
    let bound = (2.0*(n as f64).log2()).ceil() as usize;
    let mut st = 7u64;
    let mut worst = 0;
    for q in 0..2000 {
        let tq = if q == 0 { -5.0 } else if q == 1 { t_end + 5.0 } else { lcg(&mut st)*t_end };
        let (_, probes) = integ::hermite_bracket(&s.t, tq);
        worst = worst.max(probes);
    }
    eprintln!("hermite_bracket: n {n}, worst probes {worst}, bound {bound}");
    assert!(worst <= bound, "{worst} probes per query for {n} nodes (bound {bound})");
}

#[test]
fn hermite_equals_the_linear_scan_bit_for_bit() {
    for (n, uneven) in [(2usize, false), (3, true), (50, true), (1000, false), (4097, true)] {
        let s = synthetic(n, uneven);
        let mut st = 99u64;
        let mut qs: Vec<f64> = vec![s.t[0] - 3.0, s.t[0], s.t[n - 1], s.t[n - 1] + 3.0, f64::NAN];
        qs.extend(s.t.iter().copied());
        qs.extend(s.t.iter().map(|&x| f64::from_bits(x.to_bits() + 1)));
        qs.extend(s.t.iter().skip(1).map(|&x| f64::from_bits(x.to_bits() - 1)));
        for _ in 0..3000 { qs.push(s.t[0] + lcg(&mut st)*(s.t[n - 1] - s.t[0])); }
        for tq in qs {
            let (a, b) = (integ::hermite(&s, tq), hermite_linear(&s, tq));
            for c in 0..6 { assert_eq!(a[c].to_bits(), b[c].to_bits(), "n {n} tq {tq} comp {c}: {} vs {}", a[c], b[c]); }
        }
    }
}
