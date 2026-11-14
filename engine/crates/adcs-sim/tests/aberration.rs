//! S12.5b (E2; DR-nav-001): the aberration of starlight, truth and onboard. The truth tracker sees the stars aberrated by the
//! spacecraft's geometric velocity relative to the solar-system barycentre (its orbital velocity plus the Earth's from the truth's
//! ephemeris); the flight software takes it out of the tracker's attitude with the GNSS velocity plus the Earth's from the Sun
//! model's derivative (fsw_estimation's st_corrected, frames' earth_velocity). Before: the tracker took no velocity (0 shift) and the
//! flight software had no correction.
use adcs_sim::config::Config;
use adcs_sim::run::Truth;
use adcs_sim_core::la::*;
use adcs_sim_core::rng::Rng;
use adcs_sim_core::sensors::{St, StDesc};
use adcs_sim_core::{field, time};
use std::path::PathBuf;

const C: f64 = 299_792_458.0;
const LIMIT_DEG: f64 = 0.0005;

fn root() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../matlab_sils") }

fn config(days: f64) -> Config {
    let r = root();
    let ov = vec![("engine.duration_s".to_string(), "600".to_string()), ("engine.epoch_days".to_string(), format!("{days}"))];
    Config::build(&r, "fine_hold_img", &r.join("cases/ais_img_3u.csv"), 3, &ov).unwrap()
}

/// A tracker with no noise and no mount error, so the one difference between two samples is the velocity.
fn tracker() -> St {
    let mut d = StDesc { fitted: true, nh: 1, noise_cross: 0.0, noise_roll: 0.0, rate_hz: 5.0, latency: 0.0, max_rate: 1.0,
        sun_excl: 0.0, earth_excl: 0.0, fov: 0.17, model: 1, moon_excl: 0.0, blind_s: 0.0, noise_rate_ref: 0.01, ..Default::default() };
    d.bs[0] = [0.0, 0.0, 1.0];
    St::new(d, &mut Rng::new(1, 1), Rng::new(1, 3))
}

fn attitude(q: &Q, vb: &V3) -> Q {
    let mut s = tracker();
    s.history(0.0, q);
    let far = [0.0, 0.0, -1.0];
    s.sample(q, 0.0, &[0.0; 3], &far, &far, &far, 0.01, vb)[0].expect("a clear sky")
}

fn turn(a: &Q, b: &Q) -> f64 { let d = qmult(a, &qconj(b)); 2.0*(d[0]*d[0] + d[1]*d[1] + d[2]*d[2]).sqrt() }

fn attitudes(n: usize) -> Vec<Q> {
    let mut g = Rng::new(7, 3);
    (0..n).map(|_| { let q = [g.normal(), g.normal(), g.normal(), g.normal()]; let m = (q[0]*q[0] + q[1]*q[1] + q[2]*q[2] + q[3]*q[3]).sqrt(); [q[0]/m, q[1]/m, q[2]/m, q[3]/m] }).collect()
}

#[test]
fn the_truth_tracker_is_shifted_by_the_barycentric_velocity_and_the_flight_software_takes_it_out_to_0_0005_degrees() {
    let (mut worst_raw, mut worst_res, mut worst_off): (f64, f64, f64) = (0.0, 0.0, 0.0);
    for days in [0.0, -400.0, 100.0] {
        let c = config(days);
        let (mut orb, _) = Truth::new(&c).unwrap();
        let gh = field::gh(time::decyear(c.jd0));
        for t in [0.0, 300.0] {
            let (r, v) = orb.state(t).unwrap();
            let v_earth = orb.env(t, c.jd0, &r, &v, &gh, c.igrf_nmax, None).v_earth;
            let vb = [v[0] + v_earth[0], v[1] + v_earth[1], v[2] + v_earth[2]];
            let jd = c.jd0 + t/86400.0;
            assert!((norm(&v_earth) - 29.8e3).abs() < 1.0e3, "the Earth's speed from the ephemeris: {} m/s", norm(&v_earth));
            // the onboard Earth velocity, from the Sun model's derivative, against the ephemeris: the eccentricity's 1.7 per cent
            let ve = adcs_fsw::alg::frames::earth_velocity(jd);
            let off = norm(&sub(&ve, &v_earth));
            worst_off = worst_off.max(off);
            assert!(off < 700.0, "the onboard Earth velocity is {off} m/s from the ephemeris's");
            for q in attitudes(25) {
                let geometric = attitude(&q, &[0.0; 3]);
                let seen = attitude(&q, &vb);
                let a = dcm(&q);
                let u = [a[2][0], a[2][1], a[2][2]];
                let raw = turn(&seen, &geometric);
                let want = norm(&cross(&u, &vb))/C;
                // the truth tracker's report differs from the geometric one by (|v|/c) sin a, up to 1.2e-4 rad, to the fit's residual (the 12 stars' scatter, up to about 6e-6 rad)
                assert!((raw - want).abs() < 0.08*want + 7e-6, "the aberration turns the report by {raw:e} rad, (v/c) sin a = {want:e}");
                // the onboard estimator: the GNSS velocity (the orbit's) plus the Earth's from the Sun model's derivative
                let corrected = adcs_fsw::alg::estimation::st_corrected(seen, [0.0, 0.0, 1.0], jd, true, v, 1);
                let untouched = adcs_fsw::alg::estimation::st_corrected(seen, [0.0, 0.0, 1.0], jd, true, v, 0);
                assert_eq!(untouched, seen, "source 0 takes no correction");
                let res = turn(&corrected, &geometric);
                worst_raw = worst_raw.max(raw);
                worst_res = worst_res.max(res);
            }
        }
    }
    eprintln!("the report shifted by up to {worst_raw:e} rad ({:.5} deg); corrected onboard: {worst_res:e} rad ({:.5} deg); the Earth velocity off by {worst_off:.0} m/s at most",
        worst_raw.to_degrees(), worst_res.to_degrees());
    assert!(worst_raw.to_degrees() > 0.002, "the uncorrected shift is a visible fraction of 0.0071 degrees ({} deg)", worst_raw.to_degrees());
    assert!(worst_res.to_degrees() < LIMIT_DEG, "the corrected residual is {} deg, above {LIMIT_DEG}", worst_res.to_degrees());
}
