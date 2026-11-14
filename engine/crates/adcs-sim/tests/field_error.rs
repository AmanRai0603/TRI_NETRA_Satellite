//! S12.7b (E5; env_field_error, DR-env-005): the truth field has an error the onboard model does not know: the ring current's uniform
//! external field (Dst nT along the geomagnetic dipole axis) and the crust (random Gauss coefficients of the degrees 14 to 20 of a stated
//! root-mean-square value at the orbit, drawn from the run's seed). The magnitudes are DR-env-005's (the developer's, unsigned): 20-50 nT
//! quiet, 100-500 nT at Dst -200 nT. Before: the truth differed from the onboard model by the IGRF's degrees 11 to 13 alone, 9.5 nT RMS
//! over an orbit at 550 km.
use adcs_fsw_abi::Impl;
use adcs_sim::config::Config;
use adcs_sim::run::{self, Truth};
use adcs_sim_core::field::{self, ErrField};
use adcs_sim_core::gen::fielderror;
use adcs_sim_core::la::*;
use adcs_sim_core::time;
use std::path::PathBuf;

fn root() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../matlab_sils") }

fn config(overrides: &[(&str, &str)]) -> Config {
    let r = root();
    let ov: Vec<(String, String)> = overrides.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
    Config::build(&r, "fine_hold_img", &r.join("cases/ais_img_3u.csv"), 3, &ov).unwrap()
}

/// The onboard model at the truth's position: the flight software's own field (IGRF-14 to the degree of fsw_param_igrf_nmax, the Earth
/// frame it computes), at zero sensor noise.
fn onboard(c: &Config, t: f64, r: &V3) -> V3 {
    let p = &c.params;
    let jd = c.jd0 + t/86400.0;
    adcs_fsw::env::field_eci(r, jd, &[p.eop_dut1, p.eop_xp, p.eop_yp], &adcs_fsw::env::igrf_gh(time::decyear(jd)), p.igrf_nmax as i32)
}

/// The root-mean-square [nT] of the truth minus the onboard model over one orbit (every 60 s) with the error given.
fn rms_over_an_orbit(c: &Config, e: Option<&ErrField>) -> f64 {
    let (mut orb, _) = Truth::new(c).unwrap();
    let gh = field::gh(time::decyear(c.jd0));
    let (mut s, mut n) = (0.0, 0.0);
    let mut t = 0.0;
    while t <= c.period_s {
        let (r, v) = orb.state(t).unwrap();
        let truth = orb.env(t, c.jd0, &r, &v, &gh, c.igrf_nmax, e).b_eci;
        let d = scale(&sub(&truth, &onboard(c, t, &r)), 1e9);
        s += dot(&d, &d);
        n += 1.0;
        t += 60.0;
    }
    (s/n).sqrt()
}

fn nominal(c: &Config, seed: u64) -> ErrField { ErrField::draw(seed, c.crust_rms_nt, c.alt_km, c.dst_nt) }

#[test]
fn the_stated_values_are_the_quiet_ones() {
    let c = config(&[]);
    assert!(c.field_error, "env_field_error_on states 1: the nominal scenarios fly the error");
    assert_eq!((c.crust_rms_nt, c.dst_nt), (30.0, -10.0));
    let c = config(&[("engine.dst_nt", "-200")]);
    assert_eq!(c.dst_nt, -200.0);
    assert!(Config::build(&root(), "fine_hold_img", &root().join("cases/ais_img_3u.csv"), 3, &[("engine.dst_nt".to_string(), "-3000".to_string())]).is_err(), "a Dst below -2000 nT is refused");
}

/// Before the change the truth minus the onboard model was the IGRF's degrees 11 to 13 alone, 9.5 nT RMS: below the 20-50 nT band.
#[test]
fn without_the_error_the_truth_is_below_the_quiet_band_from_the_onboard_model() {
    let c = config(&[]);
    let rms = rms_over_an_orbit(&c, None);
    eprintln!("no error field: truth minus the onboard model {rms:.2} nT RMS over an orbit");
    assert!(rms > 0.5 && rms < 15.0, "{rms} nT");
}

#[test]
fn the_quiet_error_is_20_to_50_nt_and_at_dst_minus_200_it_is_100_to_500_nt() {
    let c = config(&[]);
    let (mut quiet, mut storm) = (vec![], vec![]);
    for seed in 1..=12u64 {
        let q = rms_over_an_orbit(&c, Some(&nominal(&c, seed)));
        let s = rms_over_an_orbit(&c, Some(&ErrField { dst: -200.0, ..nominal(&c, seed) }));
        quiet.push(q);
        storm.push(s);
    }
    let mean = |v: &[f64]| v.iter().sum::<f64>()/v.len() as f64;
    eprintln!("quiet (Dst -10 nT, crust 30 nT): truth minus onboard {:.1} nT mean, {:.1} to {:.1} over 12 seeds; Dst -200 nT: {:.1} mean, {:.1} to {:.1}",
        mean(&quiet), quiet.iter().cloned().fold(f64::MAX, f64::min), quiet.iter().cloned().fold(0.0, f64::max),
        mean(&storm), storm.iter().cloned().fold(f64::MAX, f64::min), storm.iter().cloned().fold(0.0, f64::max));
    for (i, q) in quiet.iter().enumerate() { assert!((20.0..=50.0).contains(q), "seed {}: quiet error {q} nT outside 20-50", i + 1); }
    for (i, s) in storm.iter().enumerate() { assert!((100.0..=500.0).contains(s), "seed {}: storm error {s} nT outside 100-500", i + 1); }
    // the run's own seed
    let own = rms_over_an_orbit(&c, Some(&nominal(&c, c.seed)));
    assert!((20.0..=50.0).contains(&own), "the scenario's seed {}: {own} nT", c.seed);
}

#[test]
fn a_run_flies_the_error_and_the_magnetometer_input_is_off_the_onboard_model() {
    // the run's truth field (the body-frame field the sensors and the torques read) against the onboard model, at the truth attitude
    let rms = |on: bool| {
        let mut c = config(&[("engine.duration_s", "5800"), ("time.record_dt_s", "30")]);
        c.field_error = on;
        let rec = run::run(&c, &run::Opts { fsw: Impl::Rust, quiet: true, realtime: false, oils: None }).unwrap();
        let (mut s, mut n) = (0.0, 0.0);
        for r in &rec.rows {
            let want = mv(&dcm(&r.q), &onboard(&c, r.t, &r.r));
            let d = scale(&sub(&r.b, &want), 1e9);
            s += dot(&d, &d);
            n += 1.0;
        }
        (s/n).sqrt()
    };
    let (on, off) = (rms(true), rms(false));
    eprintln!("a run's field minus the onboard model at the truth attitude: {on:.1} nT RMS with the error, {off:.1} nT without");
    assert!((20.0..=50.0).contains(&on) && off < 10.0, "{on} and {off} nT");
}

#[test]
fn two_seeds_draw_different_fields_and_one_seed_the_same() {
    let c = config(&[]);
    let (a, a2, b) = (nominal(&c, 1), nominal(&c, 1), nominal(&c, 2));
    assert_eq!(a.gc, a2.gc, "the same seed draws the same crust");
    assert_ne!(a.gc, b.gc);
    // the fields differ by about the crust itself: sqrt(2) x 30 nT of two independent crusts of 30 nT
    let (mut orb, _) = Truth::new(&c).unwrap();
    let gh = field::gh(time::decyear(c.jd0));
    let (mut s, mut n) = (0.0, 0.0);
    for k in 0..96 {
        let t = 60.0*k as f64;
        let (r, v) = orb.state(t).unwrap();
        let fa = orb.env(t, c.jd0, &r, &v, &gh, c.igrf_nmax, Some(&a)).b_eci;
        let fb = orb.env(t, c.jd0, &r, &v, &gh, c.igrf_nmax, Some(&b)).b_eci;
        let d = scale(&sub(&fa, &fb), 1e9);
        s += dot(&d, &d);
        n += 1.0;
    }
    let rms = (s/n).sqrt();
    eprintln!("two seeds' truth fields differ by {rms:.1} nT RMS");
    assert!(rms > 20.0 && rms < 80.0, "{rms} nT");
}

struct Reference { draws: Vec<(u64, f64, f64, Vec<f64>)>, points: Vec<(u64, f64, f64, f64, V3)> }

fn reference() -> Reference {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../tests/data/field_error_reference.txt")).unwrap();
    let mut lines = text.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty());
    let mut r = Reference { draws: vec![], points: vec![] };
    while let Some(l) = lines.next() {
        let w: Vec<&str> = l.split_whitespace().collect();
        if w[0] == "draw" {
            let n: usize = w[4].parse().unwrap();
            let v: Vec<f64> = lines.next().unwrap().split_whitespace().map(|x| x.parse().unwrap()).collect();
            assert_eq!(v.len(), n);
            r.draws.push((w[1].parse().unwrap(), w[2].parse().unwrap(), w[3].parse().unwrap(), v));
        } else {
            let v: Vec<f64> = w[1..].iter().map(|x| x.parse().unwrap()).collect();
            r.points.push((v[0] as u64, v[1], v[2], v[3], [v[4], v[5], v[6]]));
        }
    }
    r
}

/// The reference (tests/data/field_error_reference.txt) is the standard library of Python alone: the stream (SplitMix64, FNV-1a, Box-Muller)
/// and the crust's field by the potential's closed recurrence of unnormalised Legendre functions; nothing of this repository's code.
#[test]
fn the_draw_and_the_crusts_field_are_the_independent_references() {
    let r = reference();
    assert_eq!(r.draws.len(), 3);
    let mut worst = 0.0f64;
    for (seed, rms, alt, v) in &r.draws {
        let e = ErrField::draw(*seed, *rms, *alt, 0.0);
        assert_eq!(e.gc.len(), v.len());
        for (a, b) in e.gc.iter().zip(v) { worst = worst.max((a - b).abs()/b.abs().max(1e-3)); }
    }
    eprintln!("the draw against the Python reference: worst relative difference {worst:e}");
    assert!(worst < 1e-12, "{worst}");
    let gc = ErrField::draw(1, 30.0, 550.0, 0.0).gc;
    let mut worst_nt = 0.0f64;
    for (_, lat, lon, alt, want) in &r.points {
        let got = fielderror::crust_ned(gc, lat.to_radians(), lon.to_radians(), *alt);
        worst_nt = worst_nt.max(norm(&sub(&got, want)));
    }
    eprintln!("the crust's field against the Python reference: worst {worst_nt:e} nT over {} points", r.points.len());
    assert!(worst_nt < 1e-4, "{worst_nt} nT");
}

/// The mean square of the crust's field over the sphere at the orbit's radius is the stated RMS squared (Lowes's spectrum), here the
/// formula taken from the drawn coefficients, averaged over many draws.
#[test]
fn the_crust_has_the_stated_mean_square_over_the_sphere_at_the_orbit() {
    let (rms, alt) = (30.0, 550.0);
    let (a, r0) = (6371.2, 6378.137 + alt);
    let mut mean = 0.0;
    let n = 400;
    for seed in 1..=n {
        let gc = ErrField::draw(seed, rms, alt, 0.0).gc;
        let (mut ci, mut ms) = (0, 0.0);
        for deg in 14..=20usize {
            let mut sum = 0.0;
            for m in 0..=deg {
                sum += gc[ci]*gc[ci]; ci += 1;
                if m > 0 { sum += gc[ci]*gc[ci]; ci += 1; }
            }
            ms += (deg as f64 + 1.0)*(a/r0).powi(2*deg as i32 + 4)*sum;
        }
        assert_eq!(ci, 245);
        mean += ms/n as f64;
    }
    eprintln!("the mean square of the drawn crust over the sphere, mean of {n} draws: {mean:.1} nT^2 (stated {})", rms*rms);
    assert!((mean/(rms*rms) - 1.0).abs() < 0.03, "{mean}");
}

/// The ring current is Dst along the dipole's axis against the equatorial field: at the geomagnetic equator the dipole's field points
/// along -m (north for the Earth's moment pointing south), and the disturbance of Dst < 0 against it.
#[test]
fn the_ring_current_is_dst_along_the_dipole_axis_against_its_equatorial_field() {
    let c = config(&[]);
    let gh = field::gh(time::decyear(c.jd0));
    let b = fielderror::ring_current_ecef(-200.0, gh);
    assert!((norm(&b) - 200.0).abs() < 1e-9, "uniform field of |Dst| nT: {}", norm(&b));
    let m = [gh[1], gh[2], gh[0]];       // the dipole moment's direction (g11, h11, g10)
    assert!((dot(&b, &m)/norm(&m) - 200.0).abs() < 1e-9, "along the moment by -Dst: {}", dot(&b, &m)/norm(&m));
    assert!(m[2] < 0.0, "the moment points south: g10 < 0");
    // a point on the geomagnetic equator: the position perpendicular to the moment; the dipole's own field (degree 1 only) there
    let mh = scale(&m, 1.0/norm(&m));
    let u = unit(&cross(&mh, &[0.0, 0.0, 1.0]));
    let r = scale(&u, 6378.137e3 + 550e3);
    let (lat, lon, h) = field::geodetic(&r);
    let bd = field::eci_at(lat, lon, h, &[[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]], &gh, 1, None);
    let cos = dot(&bd, &b)/(norm(&bd)*norm(&b));
    eprintln!("the dipole's field at the geomagnetic equator, 550 km: {:.0} nT; against the ring current's: cos {cos:.4}", norm(&bd)*1e9);
    assert!(cos < -0.999, "the ring current's field opposes the equatorial dipole field: cos {cos}");
}
