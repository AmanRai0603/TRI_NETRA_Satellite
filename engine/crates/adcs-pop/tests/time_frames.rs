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
        assert_eq!(time::tt2utc(t.tt_jd).0, f(&c["tt2utc"]));
        assert_eq!(time::tai2utc(t.tai_jd).0, f(&c["tai2utc"]));
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
        // P8c: the Earth rate of a build that fails is the build's error, never [0;0;omega]
        assert_eq!(frames::earth_rate_eci([2027., 1., 1., 6., 0., 0.], b, &opt), Err(frames::FrameError::NoEop));
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
        let w = frames::earth_rate_eci(u6(&c["epoch"]), build, &opt).unwrap();
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

/// P8c: a world on build A (or B, C) with no EOP is refused with the frame's error; it is not flown on [0, 0, omega].
#[test]
fn world_on_a_cio_build_without_eop_is_refused() {
    use adcs_pop::accel::{Forces, Sc, World};
    for b in [Build::A, Build::B, Build::C] {
        let sc = Sc { mass: 12.0, aref: 0.07, cd: Some(2.2), cr: Some(1.3), r_bi: adcs_pop::la::I3, srp_facets: vec![], drag_facets: None };
        let r = World::new([2027., 1., 1., 6., 0., 0.], b, FrameOpt::default(), adcs_pop::gravity::Field::default_field(), Forces::sils(1.3), sc, None, None);
        assert_eq!(r.err(), Some(adcs_pop::PopError::Frame(frames::FrameError::NoEop)), "{b:?}");
    }
}

// ---- S12.4a, P9 (DR-env-002): the leap-second table's validity end, the EOP flag, the refusal of a published-vector case ----

/// The leap-second table carries its validity end (Leap_Second.dat's expiry, 28 June 2027, MJD 61584): past it
/// `tai_minus_utc` says so with a status, and holds the last row; it did not (37 s, silently, in 2036).
#[test]
fn tai_minus_utc_says_when_it_is_beyond_the_tables_validity() {
    let jd = |y, m, d| time::cal2jd(y, m, d, 0.0, 0.0, 0.0);
    assert_eq!(time::tai_minus_utc(jd(2027.0, 6.0, 27.0)), (37.0, 0), "the day before the expiry");
    assert_eq!(time::tai_minus_utc(jd(2027.0, 6.0, 28.0)), (37.0, 0), "0 h UTC of the expiry day itself");
    assert_eq!(time::tai_minus_utc(jd(2027.0, 6.0, 28.0) + 1e-3), (37.0, 1), "after it");
    assert_eq!(time::tai_minus_utc(jd(2036.0, 1.0, 1.0)), (37.0, 1), "2036-01-01 is beyond validity");
    // every function that looks the leap up carries the status, and convert_utc's Times has it
    assert_eq!(time::utc2tai(jd(2036.0, 1.0, 1.0)).1, 1);
    assert_eq!(time::tai2utc(jd(2036.0, 1.0, 1.0)).1, 1);
    assert_eq!(time::utc2tdb(jd(2027.0, 1.0, 1.0)).1, 0);
    assert_eq!(time::convert_utc(2036.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0).leap_status, 1);
    assert_eq!(time::convert_utc(2027.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0).leap_status, 0);
}

/// The design's own tables agree on the validity end: the leap table's expiry is the stated env_leap_second_expiry_mjd
/// and the EOP table's last day the stated env_eop_finals_expiry_mjd.
#[test]
fn the_snapshots_validity_ends_are_the_stated_nodes() {
    let st: serde_json::Value = serde_json::from_slice(&std::fs::read(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../matlab_sils/data/stated.json")).unwrap()).unwrap();
    let e = adcs_pop::eop::Eop::from_snapshot();
    assert_eq!(e.leap_expiry_mjd, st["value"]["env_leap_second_expiry_mjd"].as_f64());
    assert_eq!(e.finals.mjd.last().copied(), st["value"]["env_eop_finals_expiry_mjd"].as_f64());
}

/// The snapshot assembled from the design's tables is the table `parse_finals` reads from the committed file, row for row
/// over the span, bit for bit (Bulletin B where the day has it, else A).
#[test]
fn eop_from_the_design_snapshot_is_what_the_file_reads() {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/time_frames");
    let f = adcs_pop::eop::parse_finals(&String::from_utf8_lossy(&std::fs::read(dir.join("finals2000A.all")).unwrap()));
    let e = adcs_pop::eop::Eop::from_snapshot();
    let k0 = f.mjd.iter().position(|&m| m == e.finals.mjd[0]).unwrap();
    assert_eq!(e.finals.mjd.len(), f.mjd.len() - k0);
    for (i, k) in (k0..f.mjd.len()).enumerate() {
        assert_eq!((e.finals.mjd[i], e.finals.xp[i], e.finals.yp[i], e.finals.dut1[i], e.finals.dx[i], e.finals.dy[i]),
                   (f.mjd[k], f.xp[k], f.yp[k], f.dut1[k], f.dx[k], f.dy[k]), "row {k}, MJD {}", f.mjd[k]);
    }
}

/// A build-A frame past the EOP table's last day is extrapolated and says so (eop_flag 1), inside it is not (0); past the
/// leap table's expiry leap_status is 1. A published-vector case (refuse_beyond_validity) is refused by name past either.
#[test]
fn build_a_records_eop_flag_and_a_published_vector_case_refuses() {
    let opt = FrameOpt { eop: Some(adcs_pop::eop::Eop::from_snapshot()), ..Default::default() };
    let at = |u: [f64; 6], o: &FrameOpt| frames::try_eci2ecef(u, Build::A, o);
    let inside = at([2027., 1., 1., 6., 0., 0.], &opt).unwrap().2;
    assert_eq!((inside.eop_flag, inside.leap_status), (0, 0));
    let leap_only = at([2027., 8., 1., 0., 0., 0.], &opt).unwrap().2;          // after 28 June 2027, before the finals' MJD 61714
    assert_eq!((leap_only.eop_flag, leap_only.leap_status), (0, 1));
    let past = at([2028., 3., 1., 0., 0., 0.], &opt).unwrap().2;               // past MJD 61714 (November 2027)
    assert_eq!((past.eop_flag, past.leap_status), (1, 1));
    let strict = FrameOpt { refuse_beyond_validity: true, ..opt.clone() };
    assert!(at([2027., 1., 1., 6., 0., 0.], &strict).is_ok());
    assert_eq!(at([2027., 8., 1., 0., 0., 0.], &strict).err(), Some(frames::FrameError::BeyondValidity { eop_flag: 0, leap_status: 1 }));
    assert_eq!(at([2028., 3., 1., 0., 0., 0.], &strict).err(), Some(frames::FrameError::BeyondValidity { eop_flag: 1, leap_status: 1 }));
}

/// A world records the validity at the end of its span: the run's header reads it.
#[test]
fn a_worlds_validity_at_the_end_of_a_run() {
    use adcs_pop::accel::{Forces, Sc, World, Validity};
    let sc = Sc { mass: 12.0, aref: 0.07, cd: Some(2.2), cr: Some(1.3), r_bi: adcs_pop::la::I3, srp_facets: vec![], drag_facets: None };
    let opt = FrameOpt { eop: Some(adcs_pop::eop::Eop::from_snapshot()), ..Default::default() };
    let w = World::new([2027., 6., 27., 12., 0., 0.], Build::A, opt, adcs_pop::gravity::Field::default_field(), Forces::sils(1.3), sc, None, None).unwrap();
    assert_eq!(w.validity(0.0), Validity { eop_flag: 0, leap_status: 0 });
    assert_eq!(w.validity(86400.0), Validity { eop_flag: 0, leap_status: 1 }, "a run that crosses the leap table's expiry");
    assert_eq!(w.validity(200.0*86400.0), Validity { eop_flag: 1, leap_status: 1 });
}
