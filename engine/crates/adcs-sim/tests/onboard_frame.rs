//! S12.5a (T5/E3 onboard; DR-env-002, owner question 4): the flight's Earth frame (env_time_frames eci2ecef: IAU 2006 precession,
//! IAU 2000B nutation, the equation of the equinoxes, UT1 = UTC + dUT1, the pole xp, yp, the three the blob holds as flight
//! parameters and the ground sets by telecommand) against the truth's (build A, IAU 2006/2000A with the EOP of env_eop_snapshot),
//! and the GNSS fix the truth makes on the truth's frame taken to J2000 by the onboard one.
use adcs_pop::frames::{self, Build, FrameOpt};
use adcs_sim::config::Config;
use adcs_sim::run::Truth;
use adcs_sim_core::la::*;
use adcs_sim_core::sensors::Gps;
use std::path::PathBuf;

const ARCSEC: f64 = std::f64::consts::PI/648000.0;

fn root() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../matlab_sils") }

/// The case of the shipped fine-hold scenario, its epoch moved by `days` (the season shift: the blob's date moves with it).
fn config(days: f64) -> Config {
    let r = root();
    let ov = vec![("engine.duration_s".to_string(), "600".to_string()), ("engine.epoch_days".to_string(), format!("{days}"))];
    Config::build(&r, "fine_hold_img", &r.join("cases/ais_img_3u.csv"), 3, &ov).unwrap()
}

fn truth_c(c: &Config, t: f64) -> M3 {
    let opt = FrameOpt { eop: Some(adcs_pop::eop::Eop::from_snapshot()), ..Default::default() };
    frames::try_eci2ecef(adcs_pop::time::addsec(c.epoch_utc, t), Build::A, &opt).unwrap().0
}

fn onboard_c(c: &Config, t: f64) -> M3 {
    let p = &c.params;
    adcs_fsw::env::eci2ecef(c.jd0 + t/86400.0, &[p.eop_dut1, p.eop_xp, p.eop_yp])
}

/// The angle [rad] of the rotation that takes one frame to the other, from the skew part of a b^T (no acos near zero).
fn angle(a: &M3, b: &M3) -> f64 {
    let m = mm(a, &transpose(b));
    let v = [m[2][1] - m[1][2], m[0][2] - m[2][0], m[1][0] - m[0][1]];
    let (s, c) = (0.5*(v[0]*v[0] + v[1]*v[1] + v[2]*v[2]).sqrt(), 0.5*(m[0][0] + m[1][1] + m[2][2] - 1.0));
    s.atan2(c)
}

/// 2027-01-01 06:00 UTC (the shipped case's epoch) and two other epochs inside env_epoch_span (2025-01-01 to 2027-06-28).
const SEASONS: [f64; 3] = [0.0, -400.0, 100.0];

#[test]
fn the_onboard_earth_frame_is_within_an_arcsecond_of_the_truth_frame() {
    let mut worst: f64 = 0.0;
    for days in SEASONS {
        let c = config(days);
        for t in [0.0, 1800.0, 5400.0] {
            let a = angle(&onboard_c(&c, t), &truth_c(&c, t))/ARCSEC;
            eprintln!("epoch {:?} + {t} s: the onboard frame is {a:.5} arcsec from the truth's (blob: dUT1 {:.5} s, xp {:.4}, yp {:.4})",
                c.epoch_utc, c.params.eop_dut1, c.params.eop_xp, c.params.eop_yp);
            worst = worst.max(a);
        }
    }
    assert!(worst < 1.0, "the onboard frame is {worst} arcsec from the truth frame");
}

#[test]
fn dut1_moves_the_onboard_frame_by_the_earth_rotation_angle() {
    // the blob's dUT1 against dUT1 = 0: the frames differ by the Earth's rotation through UT1 - UTC (the ERA's rate 1.00273781191135448
    // turns a day), a rotation about the pole, to the precision of the polar motion's tilt of that axis (1e-5 rad of 1e-5 s)
    for days in SEASONS {
        let c = config(days);
        let p = &c.params;
        assert!(p.eop_dut1.abs() > 0.01, "the snapshot's UT1 - UTC at the epoch ({}) is a number worth checking", p.eop_dut1);
        let jd = c.jd0;
        let with = adcs_fsw::env::eci2ecef(jd, &[p.eop_dut1, p.eop_xp, p.eop_yp]);
        let without = adcs_fsw::env::eci2ecef(jd, &[0.0, p.eop_xp, p.eop_yp]);
        let want = 2.0*std::f64::consts::PI*1.00273781191135448*p.eop_dut1.abs()/86400.0;
        let got = angle(&with, &without);
        eprintln!("dUT1 {:.5} s: the frame turns {got:e} rad, the Earth rotation angle {want:e}", p.eop_dut1);
        assert!((got - want).abs() < 1e-12, "the frame turned {got:e} rad for dUT1 = {} s, the Earth rotation {want:e}", p.eop_dut1);
        // the sign: UT1 ahead of UTC (dUT1 > 0) is the Earth turned further, GAST larger
        let d = mm(&with, &transpose(&without));
        assert!((d[0][1] > 0.0) == (p.eop_dut1 > 0.0), "the rotation's sense follows the sign of dUT1");
    }
}

#[test]
fn the_gnss_fix_the_truth_makes_is_taken_back_by_the_onboard_frame_to_an_arcsecond() {
    // the receiver reports ITRF: the truth makes the fix on its own frame (gnss_ecef, S12.5a), the flight takes it to J2000 by the
    // onboard frame (navorbit's fix_eci): the position returns to the truth's within the frames' difference
    for days in SEASONS {
        let c = config(days);
        let (mut orb, _) = Truth::new(&c).unwrap();
        for t in [0.0, 1234.5] {
            let (r, v) = orb.state(t).unwrap();
            let (ce, w) = orb.earth_frame(t);
            let (re, ve) = Gps::ecef(&ce, &w, &r, &v);
            let p = &c.params;
            let (rb, vb) = adcs_fsw::alg::navorbit::fix_eci(re, ve, c.jd0 + t/86400.0, [p.eop_dut1, p.eop_xp, p.eop_yp]);
            let ang = (dot(&rb, &r)/(norm(&rb)*norm(&r))).clamp(-1.0, 1.0).acos().max(norm(&sub(&rb, &r))/norm(&r))/ARCSEC;
            let dv = norm(&sub(&vb, &v));
            eprintln!("epoch {:?} + {t} s: the position returns {:.1} m and {ang:.4} arcsec off, the velocity {dv:.2e} m/s", c.epoch_utc, norm(&sub(&rb, &r)));
            assert!(ang < 1.0, "the onboard conversion returns the truth position {ang} arcsec off");
            assert!(dv < 2e-3, "and the velocity {dv} m/s off");
        }
    }
}
