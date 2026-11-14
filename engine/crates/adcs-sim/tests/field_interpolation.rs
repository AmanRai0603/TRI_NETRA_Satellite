//! S12.7a (E8; env_field_interpolation): the truth environment is refreshed every env_refresh_step (1 s); the field was held
//! between refreshes, though it turns by up to 0.21 deg/s along the orbit, hidden only because the magnetometer samples on the
//! refresh tick. The field is now interpolated linearly in J2000 between the refresh before and the next; the Sun, the Moon, the
//! shadow, the density and the solar pressure stay held, and the numbers below say why that is fair.
//! Before: at t = k + 0.9 s the held field was about 0.19 deg off the field computed there; a field sampled each 0.05 s jumped by the
//! step (0.2 deg) at each refresh.
use adcs_fsw_abi::Impl;
use adcs_sim::config::Config;
use adcs_sim::run::{self, Truth};
use adcs_sim_core::la::*;
use adcs_sim_core::{field, time};
use field::ErrField;
use std::path::PathBuf;

fn root() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../matlab_sils") }

fn config(overrides: &[(&str, &str)]) -> Config {
    let r = root();
    let ov: Vec<(String, String)> = overrides.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
    Config::build(&r, "fine_hold_img", &r.join("cases/ais_img_3u.csv"), 3, &ov).unwrap()
}

/// The angle [deg] between two vectors.
fn deg_between(a: &V3, b: &V3) -> f64 {
    norm(&cross(a, b)).atan2(dot(a, b)).to_degrees()
}

/// A run's true field in J2000 along its rows: (t, B) from the body-frame field and the attitude.
fn fly(overrides: &[(&str, &str)]) -> (Config, Vec<(f64, V3)>) { fly_with(overrides, true) }

/// The same with env_field_interpolation as given (0 holds the field between refreshes, as the engine did until S12.6).
fn fly_with(overrides: &[(&str, &str)], interp: bool) -> (Config, Vec<(f64, V3)>) {
    let mut c = config(overrides);
    c.field_interp = interp;
    let rec = run::run(&c, &run::Opts { fsw: Impl::Rust, quiet: true, realtime: false, oils: None }).unwrap();
    let rows = rec.rows.iter().map(|r| (r.t, mv(&transpose(&dcm(&r.q)), &r.b))).collect();
    (c, rows)
}

/// The run's error field, as the run draws it (the crust from the run's seed): the truth the rows carry includes it.
fn err_of(c: &Config) -> Option<ErrField> { c.field_error.then(|| ErrField::draw(c.seed, c.crust_rms_nt, c.alt_km, c.dst_nt)) }

/// The field a fresh truth computes at t (the same orbit, the field evaluated at that instant: what a refresh at t would hold).
fn field_there(c: &Config, times: &[f64]) -> Vec<V3> {
    let (mut orb, _) = Truth::new(c).unwrap();
    let gh = field::gh(time::decyear(c.jd0));
    let err = err_of(c);
    times.iter().map(|&t| { let (r, v) = orb.state(t).unwrap(); orb.env(t, c.jd0, &r, &v, &gh, c.igrf_nmax, err.as_ref()).b_eci }).collect()
}

#[test]
fn the_field_at_k_plus_0_9_s_is_the_field_there() {
    // dt 0.1 s, refresh 1 s, 550 km; the run's own attitude only rotates the field into the body frame and back
    let (c, rows) = fly(&[("engine.duration_s", "30"), ("time.dt_s", "0.1"), ("time.record_dt_s", "0.1")]);
    assert_eq!((c.dt, c.env_dt_s, c.alt_km), (0.1, 1.0, 550.0));
    assert!(c.field_interp, "env_field_interpolation states 1");
    let times: Vec<f64> = rows.iter().map(|r| r.0).collect();
    let want = field_there(&c, &times);
    let (mut worst, mut worst_mag): (f64, f64) = (0.0, 0.0);
    for (i, (t, b)) in rows.iter().enumerate() {
        if ((t - t.floor()) - 0.9).abs() > 1e-6 { continue; }       // t = k + 0.9
        worst = worst.max(deg_between(b, &want[i]));
        worst_mag = worst_mag.max((norm(b)/norm(&want[i]) - 1.0).abs());
    }
    eprintln!("B at k + 0.9 s against the field computed there: {worst:e} deg, magnitude {worst_mag:e}");
    assert!(worst < 0.005, "the field at k + 0.9 s is {worst} deg from the field there");
    assert!(worst_mag < 1e-4, "its magnitude is {worst_mag:e} off");
}

/// What the held field was: up to 0.19 deg off at t = k + 0.9 s (the switch at 0 is the old behaviour).
#[test]
fn held_between_refreshes_the_field_was_off_by_the_step() {
    let (c, rows) = fly_with(&[("engine.duration_s", "30"), ("time.dt_s", "0.1"), ("time.record_dt_s", "0.1")], false);
    let want = field_there(&c, &rows.iter().map(|r| r.0).collect::<Vec<_>>());
    let worst = rows.iter().enumerate().filter(|(_, (t, _))| ((t - t.floor()) - 0.9).abs() < 1e-6)
        .map(|(i, (_, b))| deg_between(b, &want[i])).fold(0.0, f64::max);
    eprintln!("held: B at k + 0.9 s against the field computed there: {worst:.4} deg");
    assert!(worst > 0.15 && worst < 0.25, "the held field is {worst} deg off at k + 0.9 s");
}

#[test]
fn a_field_sampled_off_the_refresh_tick_shows_no_step() {
    // dt 0.05 s: samples at 0.95 s and 1.0 s straddle a refresh; held, the field stood still and then jumped by the step
    let held = fly_with(&[("engine.duration_s", "30"), ("time.dt_s", "0.05"), ("time.record_dt_s", "0.05")], false).1;
    let step = held.windows(2).map(|w| deg_between(&w[0].1, &w[1].1)).fold(0.0, f64::max);
    eprintln!("held: the largest change between two samples 0.05 s apart is {step:.4} deg (the step at the refresh)");
    assert!(step > 0.15, "the held field's step is {step} deg");
    let (_, rows) = fly(&[("engine.duration_s", "30"), ("time.dt_s", "0.05"), ("time.record_dt_s", "0.05")]);
    let (mut worst, mut mean) = (0.0f64, 0.0);
    for w in rows.windows(2) { let d = deg_between(&w[0].1, &w[1].1); worst = worst.max(d); mean += d; }
    mean /= (rows.len() - 1) as f64;
    eprintln!("the field's change between two samples 0.05 s apart: worst {worst:.5} deg, mean {mean:.5} deg (0.21 deg/s turns 0.0105 deg in 0.05 s)");
    assert!(worst < 0.0125, "a step of {worst} deg between two samples 0.05 s apart");
}

#[test]
fn the_rest_of_the_environment_is_slow_enough_to_hold() {
    let c = config(&[("engine.duration_s", "60")]);
    let (mut orb, _) = Truth::new(&c).unwrap();
    let gh = field::gh(time::decyear(c.jd0));
    let (mut sun, mut moon, mut rho, mut psrp, mut nu): (f64, f64, f64, f64, f64) = (0.0, 0.0, 0.0, 0.0, 0.0);
    // over a refresh step (the held value is the one at the refresh before, 0.9 s earlier): the largest change over a whole orbit
    let mut t = 0.0;
    while t < 5700.0 {
        let (r0, v0) = orb.state(t).unwrap();
        let a = orb.env(t, c.jd0, &r0, &v0, &gh, c.igrf_nmax, None);
        let (r1, v1) = orb.state(t + 1.0).unwrap();
        let b = orb.env(t + 1.0, c.jd0, &r1, &v1, &gh, c.igrf_nmax, None);
        sun = sun.max(deg_between(&a.sun_rel, &b.sun_rel));
        moon = moon.max(deg_between(&a.moon_rel, &b.moon_rel));
        rho = rho.max((b.rho/a.rho - 1.0).abs());
        psrp = psrp.max((b.p_srp/a.p_srp - 1.0).abs());
        nu = nu.max((b.nu - a.nu).abs());
        t += 97.0;
    }
    eprintln!("over a 1 s step, the worst of an orbit: Sun {sun:e} deg, Moon {moon:e} deg, density {rho:e} relative, solar pressure {psrp:e}, sunlit fraction {nu:e}");
    assert!(sun < 2e-5 && moon < 1.5e-3, "the Sun moves {sun} deg and the Moon {moon} deg in a step");
    assert!(rho < 0.01, "the density changes by {rho} in a step");
}

#[test]
fn the_next_refreshs_field_is_the_orbits_own_continuation_to_a_few_centimetres() {
    // the field one step ahead, before the orbit is advanced to it, against the field there afterwards, across node boundaries (10 s)
    let c = config(&[("engine.duration_s", "60")]);
    let (mut orb, _) = Truth::new(&c).unwrap();
    let gh = field::gh(time::decyear(c.jd0));
    let err = err_of(&c);
    let mut worst: f64 = 0.0;
    for k in 0..400 {
        let t = k as f64;
        let (r, v) = orb.state(t).unwrap();
        let _ = orb.env(t, c.jd0, &r, &v, &gh, c.igrf_nmax, err.as_ref());
        let ahead = orb.field_ahead(t + 1.0, c.jd0, &gh, c.igrf_nmax, err.as_ref());
        let (r1, v1) = orb.state(t + 1.0).unwrap();
        let there = orb.env(t + 1.0, c.jd0, &r1, &v1, &gh, c.igrf_nmax, err.as_ref()).b_eci;
        worst = worst.max(norm(&sub(&ahead, &there))*1e9);
        let _ = (r1, v1);
    }
    eprintln!("the field one step ahead against the field there: worst {worst:e} nT");
    assert!(worst < 0.01, "{worst} nT");
}
