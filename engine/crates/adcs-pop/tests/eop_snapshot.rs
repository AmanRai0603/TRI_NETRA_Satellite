//! The engine's own readers of the IERS files (eop.rs parse_finals, parse_leap) on the committed S12.2 snapshot
//! (data/time_frames/finals2000A.all and Leap_Second.dat, astropy-iers-data 0.2026.11.9.0.35.9). The test prints what they
//! read as one JSON line, which tests/test_eop_snapshot.py compares with the design's data table (env_eop_snapshot) read from
//! the same files by tools/readers.py; here it only checks that the engine reads the files at all.
use adcs_pop::eop::{parse_finals, parse_leap};
use std::path::PathBuf;

fn data(name: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/time_frames").join(name);
    String::from_utf8_lossy(&std::fs::read(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))).into_owned()
}

fn list(v: &[f64]) -> String {
    format!("[{}]", v.iter().map(|x| format!("{x:?}")).collect::<Vec<_>>().join(","))
}

#[test]
fn engine_reads_the_snapshot_files() {
    let f = parse_finals(&data("finals2000A.all"));
    let l = parse_leap(&data("Leap_Second.dat"), false);
    assert!(f.mjd.len() > 20000 && f.mjd.windows(2).all(|w| w[0] < w[1]));
    assert_eq!(l.len(), 28);
    let lm: Vec<f64> = l.iter().map(|r| r[0]).collect();
    let ld: Vec<f64> = l.iter().map(|r| r[1]).collect();
    println!(
        "ENGINE_EOP_JSON {{\"mjd\":{},\"xp\":{},\"yp\":{},\"dut1\":{},\"dx\":{},\"dy\":{},\"leap_mjd\":{},\"leap_dat\":{}}}",
        list(&f.mjd), list(&f.xp), list(&f.yp), list(&f.dut1), list(&f.dx), list(&f.dy), list(&lm), list(&ld)
    );
}
