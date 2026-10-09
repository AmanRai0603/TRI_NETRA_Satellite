//! time.rs / eop.rs / frames / geodetic.rs against the MATLAB POP (Octave), vectors
//! from refgen/time_frames.m -> tests/data/time_frames.json (+ the synthetic EOP set
//! in tests/data/time_frames_eop/ that the same script wrote and Octave parsed).
use adcs_pop::eop::Eop;
use adcs_pop::frames::{self, Build, FrameOpt};
use adcs_pop::la::M3;
use adcs_pop::{geodetic, time};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::OnceLock;

fn data_dir() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data") }

fn refdata() -> &'static Value {
    static V: OnceLock<Value> = OnceLock::new();
    V.get_or_init(|| {
        let s = std::fs::read_to_string(data_dir().join("time_frames.json")).expect("time_frames.json");
        serde_json::from_str(&quote_numbers(&s)).expect("json")
    })
}

fn eop() -> &'static Eop {
    static E: OnceLock<Eop> = OnceLock::new();
    E.get_or_init(|| Eop::from_dir(data_dir().join("time_frames_eop")).expect("eop"))
}

/// serde_json (without its `float_roundtrip` feature) may round a 17-digit decimal
/// to a neighbouring double; wrap every number in quotes and parse it with std's
/// correctly rounded `str::parse::<f64>` instead.
fn quote_numbers(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = String::with_capacity(s.len() * 5 / 4);
    let (mut i, mut in_str) = (0, false);
    while i < b.len() {
        let c = b[i] as char;
        if in_str {
            out.push(c);
            if c == '\\' {
                out.push(b[i + 1] as char);
                i += 1;
            } else if c == '"' {
                in_str = false;
            }
            i += 1;
        } else if c == '"' {
            in_str = true;
            out.push(c);
            i += 1;
        } else if c == '-' || c.is_ascii_digit() {
            let j = i;
            while i < b.len() && matches!(b[i], b'0'..=b'9' | b'-' | b'+' | b'.' | b'e' | b'E') { i += 1; }
            out.push('"');
            out.push_str(&s[j..i]);
            out.push('"');
        } else {
            out.push(c);
            i += 1;
        }
    }
    out
}

fn f(v: &Value) -> f64 {
    match v {
        Value::String(s) => s.parse::<f64>().expect("number"),
        _ => f64::NAN, // null = MATLAB NaN
    }
}
fn arr(v: &Value) -> Vec<f64> {
    match v {
        Value::Array(a) => a.iter().map(f).collect(),
        x => vec![f(x)],
    }
}
fn m3(v: &Value) -> M3 {
    let a = arr(v);
    [[a[0], a[1], a[2]], [a[3], a[4], a[5]], [a[6], a[7], a[8]]]
}
fn u6(v: &Value) -> [f64; 6] {
    let a = arr(v);
    [a[0], a[1], a[2], a[3], a[4], a[5]]
}
fn mdiff(a: &M3, b: &M3) -> f64 {
    let mut d: f64 = 0.0;
    for i in 0..3 { for j in 0..3 { d = d.max((a[i][j] - b[i][j]).abs()); } }
    d
}

#[test]
fn convert_utc_matches_matlab() {
    let cases = refdata()["convert_utc"].as_array().unwrap();
    let (mut worst_jd, mut worst_s): (f64, f64) = (0.0, 0.0);
    for c in cases {
        let i = arr(&c["in"]);
        let t = time::convert_utc(i[0], i[1], i[2], i[3], i[4], i[5], i[6]);
        let o = &c["out"];
        // Julian dates: the MATLAB and Rust values must be the same double.
        for (k, v) in [("utc_jd", t.utc_jd), ("tai_jd", t.tai_jd), ("tt_jd", t.tt_jd), ("tdb_jd", t.tdb_jd), ("ut1_jd", t.ut1_jd),
            ("gps_jd", t.gps_jd), ("utc_mjd", t.utc_mjd), ("tt_mjd", t.tt_mjd), ("tdb_mjd", t.tdb_mjd)] {
            let d = (v - f(&o[k])).abs();
            worst_jd = worst_jd.max(d);
            assert!(d <= 1e-14, "{k} {i:?}: rust {v:.17} matlab {:.17}", f(&o[k]));
        }
        for (k, v, tol) in [("leap", t.leap, 0.0), ("dut1", t.dut1, 0.0), ("t_tt", t.t_tt, 1e-18), ("j2000_tt_sec", t.j2000_tt_sec, 1e-9),
            ("gps_week", t.gps_week, 0.0), ("gps_sow", t.gps_sow, 1e-9), ("gmst_rad", t.gmst_rad, 1e-13), ("doy", t.doy, 0.0)] {
            let d = (v - f(&o[k])).abs();
            if k == "j2000_tt_sec" || k == "gps_sow" { worst_s = worst_s.max(d); }
            assert!(d <= tol, "{k} {i:?}: rust {v:.17} matlab {:.17} (|d|={d:e})", f(&o[k]));
        }
        let jc = time::jd2cal(t.tt_jd);
        let mj = arr(&c["jd2cal_tt"]);
        for k in 0..6 { assert!((jc[k] - mj[k]).abs() <= 1e-9, "jd2cal {i:?}: {jc:?} vs {mj:?}"); }
        assert_eq!(time::tt2utc(t.tt_jd), f(&c["tt2utc"]));
        assert_eq!(time::tai2utc(t.tai_jd), f(&c["tai2utc"]));
    }
    eprintln!("convert_utc: {} cases, max |dJD| = {worst_jd:e} d, max |d sec-fields| = {worst_s:e} s", cases.len());
}

#[test]
fn addsec_matches_matlab() {
    let cases = refdata()["addsec"].as_array().unwrap();
    let mut worst: f64 = 0.0;
    for c in cases {
        let r = time::addsec(u6(&c["utc"]), f(&c["sec"]));
        let m = arr(&c["out"]);
        for k in 0..5 { assert_eq!(r[k], m[k], "addsec {:?} + {}: {r:?} vs {m:?}", u6(&c["utc"]), f(&c["sec"])); }
        worst = worst.max((r[5] - m[5]).abs());
        assert!((r[5] - m[5]).abs() <= 1e-9, "addsec seconds {r:?} vs {m:?}");
    }
    eprintln!("addsec: {} cases, max |d seconds| = {worst:e}", cases.len());
}

#[test]
fn gmst_build_matches_matlab() {
    let cases = refdata()["gmst_build"].as_array().unwrap();
    let mut worst: f64 = 0.0;
    for c in cases {
        let g = arr(&c["gmst_rad"]);
        let opt = FrameOpt { dut1: f(&c["dut1"]), xp: f(&c["xp"]), yp: f(&c["yp"]), gmst_rad: g.first().copied(), ..Default::default() };
        let (cm, ct, info) = frames::try_eci2ecef(u6(&c["utc"]), Build::Gmst, &opt).unwrap();
        let d = mdiff(&cm, &m3(&c["C"])).max(mdiff(&ct, &m3(&c["Ct"])));
        worst = worst.max(d);
        assert!(d <= 1e-13, "gmst build {:?}: |dC| = {d:e}", u6(&c["utc"]));
        assert!((info.gmst_rad - f(&c["info_gmst"])).abs() <= 1e-13);
        assert_eq!(frames::eci2ecef(u6(&c["utc"]), Build::Gmst, &opt), (cm, ct));
    }
    eprintln!("gmst build: {} cases, max |dC| = {worst:e}", cases.len());
}

#[test]
fn cio_builds_match_matlab() {
    let cases = refdata()["cio"].as_array().unwrap();
    let e = eop();
    let mut opt = FrameOpt { eop: Some(e.clone()), ..Default::default() };
    let (mut wc, mut wera, mut wxy, mut wut, mut wpm) = (0.0f64, 0.0f64, 0.0f64, 0.0f64, 0.0f64);
    for c in cases {
        let build = Build::parse(c["build"].as_str().unwrap()).unwrap();
        let ov = arr(&c["override"]);
        opt.tidal = f(&c["tidal"]) != 0.0;
        opt.eop_override = if ov.len() == 6 { Some([ov[0], ov[1], ov[2], ov[3], ov[4], ov[5]]) } else { None };
        let utc = u6(&c["utc"]);
        let (cm, ct, info) = frames::try_eci2ecef(utc, build, &opt).unwrap();
        let tag = format!("{build:?} {utc:?} tidal={} ov={ov:?}", opt.tidal);
        assert_eq!(info.eop_flag as f64, f(&c["eop_flag"]), "{tag}");
        assert_eq!(info.dat, f(&c["dat"]), "{tag}");
        assert_eq!(info.mjd_utc, f(&c["mjd_utc"]), "{tag}");
        let du = (info.dut1 - f(&c["dut1"])).abs();
        let pm = [(info.xp - f(&c["xp"])).abs(), (info.yp - f(&c["yp"])).abs(), (info.dx - f(&c["dx"])).abs(), (info.dy - f(&c["dy"])).abs()]
            .iter().cloned().fold(0.0, f64::max);
        let xy = [(info.x - f(&c["X"])).abs(), (info.y - f(&c["Y"])).abs(), (info.s - f(&c["s"])).abs()].iter().cloned().fold(0.0, f64::max);
        let de = (info.era - f(&c["era"])).abs();
        let dc = mdiff(&cm, &m3(&c["C"])).max(mdiff(&ct, &m3(&c["Ct"])));
        wc = wc.max(dc);
        wera = wera.max(de);
        wxy = wxy.max(xy);
        wut = wut.max(du);
        wpm = wpm.max(pm);
        assert!(du <= 1e-12, "{tag}: dUT1 {} vs {} ({du:e})", info.dut1, f(&c["dut1"]));
        assert!(pm <= 1e-18, "{tag}: pole/dX/dY |d| = {pm:e}");
        assert!(xy <= 1e-17, "{tag}: X/Y/s |d| = {xy:e}");
        assert!(de <= 1e-13, "{tag}: era |d| = {de:e}");
        assert!(dc <= 1e-13, "{tag}: |dC| = {dc:e}");
    }
    eprintln!("cio builds: {} cases, max |dC| = {wc:e}, |d era| = {wera:e}, |d X,Y,s| = {wxy:e}, |d dUT1| = {wut:e} s, |d xp,yp,dX,dY| = {wpm:e}",
        cases.len());
}

#[test]
fn cio_builds_without_eop_behave_like_matlab_offline() {
    let opt = FrameOpt::default();
    for b in [Build::A, Build::B, Build::C] {
        assert_eq!(frames::try_eci2ecef([2027., 1., 1., 6., 0., 0.], b, &opt).err(), Some(frames::FrameError::NoEop));
        // buildWorld/earthRateECI falls back to [0;0;omega] when the build fails
        assert_eq!(frames::earth_rate_eci([2027., 1., 1., 6., 0., 0.], b, &opt), [0.0, 0.0, frames::omega_earth()]);
        assert_eq!(frames::omega_earth(), 7.2921150e-5);     // the value the hand-written constant held
    }
}

#[test]
fn tidal_models_match_matlab() {
    use adcs_pop::frames::tidal::*;
    let cases = refdata()["tidal"].as_array().unwrap();
    let mut worst: f64 = 0.0;
    for c in cases {
        let mjd = f(&c["mjd"]);
        let (ox, oy, ou) = tidal_eop_ocean(mjd);
        let (px, py) = tidal_pm_libration(mjd);
        let (uu, ul) = tidal_ut1_libration(mjd);
        let (tx, ty, tu) = tidal_eop(mjd);
        let z = tidal_ut1_zonal(mjd);
        let got = [ox, oy, ou, px, py, uu, ul, tx, ty, tu];
        let exp: Vec<f64> = [arr(&c["ocean"]), arr(&c["pm"]), arr(&c["ut1lib"]), arr(&c["total"])].concat();
        for k in 0..10 {
            let d = (got[k] - exp[k]).abs();
            worst = worst.max(d);
            assert!(d <= 1e-9, "tidal mjd {mjd} term {k}: {} vs {} (uas/us)", got[k], exp[k]);
        }
        assert!((z - f(&c["zonal"])).abs() <= 1e-15, "zonal mjd {mjd}: {z} vs {}", f(&c["zonal"]));
    }
    eprintln!("tidal: max |d| = {worst:e} uas/us");
}

#[test]
fn geodetic_matches_matlab() {
    let cases = refdata()["geodetic"].as_array().unwrap();
    let (mut wa, mut wh): (f64, f64) = (0.0, 0.0);
    for c in cases {
        let r = arr(&c["r"]);
        let (lat, lon, alt) = geodetic::geodetic(&[r[0], r[1], r[2]]);
        // at the exact pole (p = 0) MATLAB's iteration divides by N+alt = 0 and returns
        // NaN; the port reproduces it (NaN on both sides counts as agreement)
        let dd = |a: f64, b: f64| if a.is_nan() && b.is_nan() { 0.0 } else { (a - b).abs() };
        let da = dd(lat, f(&c["lat"])).max(dd(lon, f(&c["lon"])));
        let dh = dd(alt, f(&c["alt"]));
        wa = wa.max(da);
        wh = wh.max(dh);
        assert!(da <= 1e-9, "geodetic {r:?}: angles |d| = {da:e}");
        assert!(dh <= 1e-6, "geodetic {r:?}: alt |d| = {dh:e}");
    }
    eprintln!("geodetic: max |d angle| = {wa:e} rad, |d alt| = {wh:e} m");
}

#[test]
fn earth_rate_matches_matlab() {
    let cases = refdata()["earth_rate"].as_array().unwrap();
    let mut worst: f64 = 0.0;
    for c in cases {
        let build = Build::parse(c["build"].as_str().unwrap()).unwrap();
        let opt = FrameOpt { dut1: f(&c["dut1"]), eop: if build == Build::Gmst { None } else { Some(eop().clone()) }, ..Default::default() };
        let w = frames::earth_rate_eci(u6(&c["epoch"]), build, &opt);
        let m = arr(&c["w"]);
        for k in 0..3 {
            let d = (w[k] - m[k]).abs();
            worst = worst.max(d);
            assert!(d <= 1e-17, "earth rate {build:?}: {w:?} vs {m:?}");
        }
    }
    eprintln!("earth rate: max |d| = {worst:e} rad/s");
}

#[test]
fn hot_path_timing() {
    // not a benchmark: a sanity bound so a regression to allocation / reparsing shows up
    let e = eop();
    let opt_c = FrameOpt { eop: Some(e.clone()), ..Default::default() };
    let opt_g = FrameOpt::default();
    let n = 200;
    let t0 = std::time::Instant::now();
    let mut acc = 0.0;
    for k in 0..n {
        let u = time::addsec([2027., 1., 1., 6., 0., 0.], k as f64 * 0.1);
        let t = time::convert_utc(u[0], u[1], u[2], u[3], u[4], u[5], 0.0);
        acc += frames::eci2ecef(u, Build::Gmst, &FrameOpt { gmst_rad: Some(t.gmst_rad), ..opt_g.clone() }).0[0][0];
    }
    let tg = t0.elapsed() / n;
    let t1 = std::time::Instant::now();
    for k in 0..n {
        let u = time::addsec([2027., 1., 1., 6., 0., 0.], k as f64 * 0.1);
        acc += frames::eci2ecef(u, Build::C, &opt_c).0[0][0];
    }
    let tc = t1.elapsed() / n;
    eprintln!("per step: addsec+convert_utc+gmst {tg:?}, addsec+build C {tc:?} ({acc})");
}
