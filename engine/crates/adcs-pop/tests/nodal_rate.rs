//! TS1(d): the nodal regression of a J2-only orbit against its closed form.
//!
//! Oracle: Vallado, Fundamentals of Astrodynamics and Applications, 4th ed., Section 9.6 (the secular rates of J2):
//!   dOmega/dt = -3/2 n J2 (Re/p)^2 cos i,  p = a (1 - e^2),  n = sqrt(mu/a^3),
//! with the mean elements of the orbit (Kozai 1959, first-order mean elements: the time average of the
//! osculating elements over whole orbits). The sun-synchronous rate is 360 deg per tropical year,
//! 0.98565 deg/day (Vallado Section 9.6, sun-synchronous orbits; tropical year 365.2422 d).
//! The test integrates the engine's J2-only gravity (adcs-pop World, Zonal(2)) with rk78 over one day.
use adcs_pop::accel::{coe2rv, Forces, Sc, World};
use adcs_pop::frames::{Build, FrameOpt};
use adcs_pop::gravity::{Field, GravityForceCfg, GravityModel};
use adcs_pop::integ::{self, Opts};
use adcs_pop::la::I3;

/// (measured node rate, closed-form node rate [deg/day], mean a, mean e, GM, Re, J2)
fn run(inc_deg: f64) -> (f64, f64, f64, f64, f64, f64, f64) {
    let mut f = Forces::sils(1.3);
    f.gravity = GravityForceCfg { model: GravityModel::Zonal(2), degree: 2, order: None };
    f.thirdbody = None; f.drag = None; f.srp = None; f.erp = None; f.relativity = None; f.solidtides = false; f.oceantides = false; f.empirical = None;
    let field = Field::default_field();
    let (mu, re, j2) = (field.gm, field.re, field.j[0]);
    let mut w = World::new([2026.0, 3.0, 20.0, 12.0, 0.0, 0.0], Build::Gmst, FrameOpt::default(), field, f,
        Sc { mass: 4.0, aref: 0.03, cd: None, cr: None, r_bi: I3, srp_facets: vec![], drag_facets: None }, None, None).unwrap();
    let a0 = re + 550e3;
    let (r0, v0) = coe2rv(a0, 0.0, inc_deg.to_radians(), 0.3, 0.0, 0.0, mu);
    let y0 = [r0[0], r0[1], r0[2], v0[0], v0[1], v0[2]];
    let day = 86400.0;
    let o = Opts { rtol: 1e-12, atol: 1e-9, ..Opts::default() };
    let s = integ::rk78(|t, y| { let (a, ..) = w.accel(t, &[y[0], y[1], y[2]], &[y[3], y[4], y[5]]).unwrap(); [y[3], y[4], y[5], a[0], a[1], a[2]] }, 0.0, day, y0, &o).unwrap();
    // osculating elements on a 10 s grid by dense output
    let (mut raan, mut prev, mut ts, mut om) = (0.0f64, f64::NAN, vec![], vec![]);
    let (mut sa, mut se, mut si, mut n) = (0.0, 0.0, 0.0, 0.0);
    let mut t = 0.0;
    while t <= day {
        let y = integ::hermite(&s, t);
        let (r, v) = ([y[0], y[1], y[2]], [y[3], y[4], y[5]]);
        let h = [r[1]*v[2] - r[2]*v[1], r[2]*v[0] - r[0]*v[2], r[0]*v[1] - r[1]*v[0]];
        let hn = (h[0]*h[0] + h[1]*h[1] + h[2]*h[2]).sqrt();
        let rn = (r[0]*r[0] + r[1]*r[1] + r[2]*r[2]).sqrt();
        let v2 = v[0]*v[0] + v[1]*v[1] + v[2]*v[2];
        let a = 1.0/(2.0/rn - v2/mu);
        let ev = [(v2 - mu/rn)*r[0]/mu - (r[0]*v[0] + r[1]*v[1] + r[2]*v[2])*v[0]/mu, (v2 - mu/rn)*r[1]/mu - (r[0]*v[0] + r[1]*v[1] + r[2]*v[2])*v[1]/mu, (v2 - mu/rn)*r[2]/mu - (r[0]*v[0] + r[1]*v[1] + r[2]*v[2])*v[2]/mu];
        let e = (ev[0]*ev[0] + ev[1]*ev[1] + ev[2]*ev[2]).sqrt();
        let om_now = h[0].atan2(-h[1]);
        let om_u = if prev.is_nan() { om_now } else { let mut d = om_now - prev; while d > 3.0 { d -= 2.0*std::f64::consts::PI; } while d < -3.0 { d += 2.0*std::f64::consts::PI; } raan + d };
        raan = om_u; prev = om_now;
        ts.push(t); om.push(om_u);
        sa += a; se += e; si += (h[2]/hn).acos(); n += 1.0;
        t += 10.0;
    }
    let (a, e, i) = (sa/n, se/n, si/n);
    let p = a*(1.0 - e*e);
    let nm = (mu/(a*a*a)).sqrt();
    let want = -1.5*nm*j2*(re/p).powi(2)*i.cos();            // rad/s
    // least-squares slope of the unwrapped node over the day
    let m = ts.len() as f64;
    let (mt, mo) = (ts.iter().sum::<f64>()/m, om.iter().sum::<f64>()/m);
    let got = ts.iter().zip(&om).map(|(t, o)| (t - mt)*(o - mo)).sum::<f64>()/ts.iter().map(|t| (t - mt).powi(2)).sum::<f64>();
    eprintln!("i0 {inc_deg} deg: mean a {a:.1} e {e:.2e} i {:.4} deg; node rate {:.6} deg/day, closed form {:.6} deg/day ({:.4} %)",
        i.to_degrees(), got.to_degrees()*day, want.to_degrees()*day, 100.0*(got/want - 1.0));
    (got.to_degrees()*day, want.to_degrees()*day, a, e, mu, re, j2)
}

#[test]
fn j2_nodal_rate_matches_vallado_within_0_1_percent() {
    let (got, want, ..) = run(97.6);
    assert!((got/want - 1.0).abs() < 1e-3, "node rate {got} deg/day vs {want}");
}

#[test]
fn sun_synchronous_inclination_regresses_at_0_9856_deg_per_day() {
    // the SSO inclination from the closed form at the orbit's mean a and e (cos i = -rate/(1.5 n J2 (Re/p)^2)),
    // then the engine's propagated node rate at that inclination against 360 deg per tropical year
    let sso: f64 = 360.0/365.2422;
    assert!((sso - 0.9856).abs() < 1e-4);
    let (_, _, a, e, mu, re, j2) = run(97.6);
    let (n, p) = ((mu/(a*a*a)).sqrt(), a*(1.0 - e*e));
    let ci = -(sso.to_radians()/86400.0)/(1.5*n*j2*(re/p).powi(2));
    let inc = ci.acos().to_degrees();
    let (got, ..) = run(inc);
    eprintln!("SSO inclination {inc:.4} deg: node rate {got:.5} deg/day vs {sso:.5}");
    assert!((got/sso - 1.0).abs() < 5e-3, "node rate {got} deg/day vs {sso} at i = {inc}");
}
