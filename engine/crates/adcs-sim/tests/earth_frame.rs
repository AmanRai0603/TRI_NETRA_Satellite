//! S12.4b (P2, E4, T5/E3 truth; DR-env-002): the loop's truth orbit is built on the Earth frame the design states
//! (env_earth_frame_build: IAU 2006/2000A build A with the EOP of env_eop_snapshot, linear interpolation), and the field
//! along the orbit is evaluated through that frame as it is, the precession not counted a second time.
use adcs_pop::frames::{self, Build, FrameOpt};
use adcs_sim::config::Config;
use adcs_sim::run::Truth;
use adcs_sim_core::la::*;
use adcs_sim_core::{field, time};
use std::path::PathBuf;

fn root() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../matlab_sils") }

fn config(overrides: &[(&str, &str)]) -> Config {
    let r = root();
    let ov: Vec<(String, String)> = overrides.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
    Config::build(&r, "fine_hold_img", &r.join("cases/ais_img_3u.csv"), 3, &ov).unwrap()
}

/// The loop's ECI -> ECEF rotation at t seconds after the epoch, as the loop gets it (the context between two nodes).
fn loop_c(orb: &mut Truth, t: f64) -> M3 {
    orb.state(t).unwrap();
    match orb { Truth::Pop(o) => o.context(t).c, Truth::Fast(..) => panic!("the precision orbit") }
}

/// build A with the snapshot's EOP, as frames::try_eci2ecef gives it at the epoch plus t.
fn reference_c(c: &Config, t: f64) -> M3 {
    let opt = FrameOpt { eop: Some(adcs_pop::eop::Eop::from_snapshot()), ..Default::default() };
    frames::try_eci2ecef(adcs_pop::time::addsec(c.epoch_utc, t), Build::A, &opt).unwrap().0
}

/// The angle [rad] of the rotation that takes one frame to the other, from the skew part of a b^T (no acos near zero).
fn angle(a: &M3, b: &M3) -> f64 {
    let m = mm(a, &transpose(b));
    let v = [m[2][1] - m[1][2], m[0][2] - m[2][0], m[1][0] - m[0][1]];
    let (s, c) = (0.5*(v[0]*v[0] + v[1]*v[1] + v[2]*v[2]).sqrt(), 0.5*(m[0][0] + m[1][1] + m[2][2] - 1.0));
    s.atan2(c)
}

#[test]
fn the_loop_is_built_on_the_stated_frame_build_a() {
    let c = config(&[("engine.duration_s", "600")]);
    assert_eq!(c.frame_build, Build::A, "env_earth_frame_build states 1");
    let (mut orb, _) = Truth::new(&c).unwrap();
    // at nodes (10 s) and between them (the context rotates node 0's matrix on at the Earth rate), over an hour
    let mut worst: f64 = 0.0;
    for t in [0.0, 3.3, 10.0, 17.5, 123.4, 600.0, 1800.7, 3599.9] {
        worst = worst.max(angle(&loop_c(&mut orb, t), &reference_c(&c, t)));
    }
    eprintln!("the loop's C against try_eci2ecef(A): worst {worst:e} rad");
    assert!(worst < 1e-9, "the loop's C is {worst:e} rad from build A");
}

#[test]
fn the_field_in_eci_is_igrf_at_the_itrf_position() {
    let c = config(&[("engine.duration_s", "600")]);
    let (mut orb, _) = Truth::new(&c).unwrap();
    let gh = field::gh(time::decyear(c.jd0));
    let mut worst: f64 = 0.0;
    for t in [0.0, 7.0, 333.0, 1234.5] {
        let (r, v) = orb.state(t).unwrap();
        let b = orb.env(t, c.jd0, &r, &v, &gh, c.igrf_nmax).b_eci;
        // independently: to ITRF by build A, geodetic position, IGRF north-east-down, to ECEF, back to ECI by C^T
        let ca = reference_c(&c, t);
        let re = mv(&ca, &r);
        let (lat, lon, h) = adcs_pop::geodetic::geodetic(&re);
        let ned = field::ned(&gh, lat, lon, h/1e3, c.igrf_nmax);      // nT
        let (sl, cl, so, co) = (lat.sin(), lat.cos(), lon.sin(), lon.cos());
        let be = [-sl*co*ned[0] - so*ned[1] - cl*co*ned[2], -sl*so*ned[0] + co*ned[1] - cl*so*ned[2], cl*ned[0] - sl*ned[2]];
        let want = mv(&transpose(&ca), &be);
        let d = norm(&sub(&scale(&b, 1e9), &want));
        worst = worst.max(d);
    }
    eprintln!("B in ECI against IGRF at the ITRF position: worst {worst:e} nT");
    assert!(worst < 1.0, "the field differs from IGRF at the ITRF position by {worst} nT");
}

#[test]
fn the_j2_pole_is_the_pole_of_date() {
    let c = config(&[("engine.duration_s", "600")]);
    let (mut orb, _) = Truth::new(&c).unwrap();
    let pole = |m: &M3| m[2];                                      // ITRF z in ECI: the pole gravity's J2 acts about
    let c_loop = loop_c(&mut orb, 0.0);
    // the pole of the day: the CIP in the GCRS from the IAU 2006/2000A series (xy06, held to ERFA's vectors by adcs-pop's
    // published.rs): the precession and the nutation, not the loop's own matrix
    let t = adcs_pop::time::convert_utc(c.epoch_utc[0], c.epoch_utc[1], c.epoch_utc[2], c.epoch_utc[3], c.epoch_utc[4], c.epoch_utc[5], 0.0).t_tt;
    let (x, y) = frames::xy06(t, &frames::fund_args(t));
    let cip = [x, y, (1.0 - x*x - y*y).sqrt()];
    let ang = |a: V3, b: V3| norm(&cross(&a, &b)).atan2(dot(&a, &b)).to_degrees();
    let after = ang(pole(&c_loop), cip);
    // the old frame: GMST about the pole of J2000, the pole of J2000 itself
    let gmst = frames::eci2ecef(c.epoch_utc, Build::Gmst, &FrameOpt::default()).0;
    let before = ang(pole(&gmst), cip);
    // and the mean pole of date of the IAU 1976 precession the field used to be given (no nutation)
    let mean = ang(pole(&c_loop), time::precession(c.jd0)[2]);
    eprintln!("J2 pole against the CIP at {:?}: Gmst {before:.5} deg, build A {after:.6} deg (build A to the IAU 1976 mean pole {mean:.5} deg: the nutation)", c.epoch_utc);
    assert!(before > 0.1, "the GMST-only frame's pole is the pole of J2000, 0.15 deg off the pole of 2027 ({before} deg)");
    assert!(after < 0.001, "the loop's pole is {after} deg from the pole of date");
}

/// The truth frame at three epochs of the span, for tests/test_earth_frame_oracle.py (pyerfa): writes the loop's C where
/// ADCS_FRAME_DUMP names a file, and does nothing otherwise.
#[test]
fn dump_the_truth_frame_for_the_external_oracle() {
    let Some(out) = std::env::var_os("ADCS_FRAME_DUMP") else { return };
    let mut rows = vec![];
    for days in ["0", "-250", "90"] {
        let c = config(&[("engine.duration_s", "600"), ("engine.epoch_days", days)]);
        let (mut orb, _) = Truth::new(&c).unwrap();
        for t in [0.0, 5.5, 17.25] {
            let m = loop_c(&mut orb, t);
            let u = adcs_pop::time::addsec(c.epoch_utc, t);
            rows.push(serde_json::json!({"utc": u, "c": m}));
        }
    }
    std::fs::write(out, serde_json::to_string(&rows).unwrap()).unwrap();
}
