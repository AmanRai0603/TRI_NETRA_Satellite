//! The engine refuses what it cannot run, by name, and never guesses: every shipped
//! scenario builds, and each kind of bad input is refused with a message that names it.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use adcs_sim::config::{check_id, Config};
use std::path::PathBuf;

fn root() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../matlab_sils") }

fn case_for(s: &str) -> PathBuf {
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(root().join("data/scenarios").join(format!("{s}.json"))).unwrap()).unwrap();
    root().join("cases").join(format!("{}.csv", v["case"].as_str().unwrap_or("ais_3u")))
}

fn build(s: &str, sets: &[(&str, &str)]) -> Result<Config, String> {
    let sets: Vec<(String, String)> = sets.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
    Config::build(&root(), s, &case_for(s), 1, &sets)
}

fn refused(s: &str, sets: &[(&str, &str)], says: &str) {
    match build(s, sets) {
        Ok(_) => panic!("{sets:?} was accepted"),
        Err(e) => assert!(e.contains(says), "{sets:?}: {e:?} does not say {says:?}"),
    }
}

#[test]
fn every_shipped_scenario_builds() {
    let mut n = 0;
    for f in std::fs::read_dir(root().join("data/scenarios")).unwrap() {
        let p = f.unwrap().path();
        if p.extension().and_then(|x| x.to_str()) != Some("json") { continue; }
        let id = p.file_stem().unwrap().to_str().unwrap().to_string();
        build(&id, &[]).unwrap_or_else(|e| panic!("{id}: {e}"));
        n += 1;
    }
    assert!(n >= 40, "only {n} scenarios");
}

#[test]
fn an_id_never_reaches_outside_its_folder() {
    for bad in ["../x", "a/b", "a\\b", "", ".hidden", "x y"] { assert!(check_id("scenario", bad).is_err(), "{bad:?}"); }
    for ok in ["nadir_hold_ais", "mission-fmr.v2"] { assert!(check_id("scenario", ok).is_ok(), "{ok:?}"); }
    assert!(Config::build(&root(), "../../etc/passwd", &root().join("cases/ais_3u.csv"), 1, &[]).unwrap_err().contains("not an id"));
}

#[test]
fn a_missing_scenario_is_named() {
    assert!(Config::build(&root(), "no_such_scenario", &root().join("cases/ais_3u.csv"), 1, &[]).unwrap_err().contains("no scenario no_such_scenario"));
}

#[test]
fn an_override_keeps_the_type_it_replaces() {
    refused("nadir_hold_ais", &[("time.dt_s", "abc")], "has a number here");
    refused("nadir_hold_ais", &[("time.dt_s", "NaN")], "not a finite number");
    refused("nadir_hold_ais", &[("time.dt_s", "inf")], "not a finite number");
    // a number may replace a "case:<key>" reference: the Monte Carlo draws do exactly that
    build("detumble_ais", &[("initial.rate.magnitude_deg_s", "8.5")]).unwrap();
}

#[test]
fn an_override_outside_the_scenario_is_refused() {
    refused("nadir_hold_ais", &[("bogus.x", "1")], "no section \"bogus\"");
    refused("nadir_hold_ais", &[("fsw..x", "1")], "dotted names");
    refused("nadir_hold_ais", &[("engine.warp", "1")], "unknown engine override engine.warp");
}

#[test]
fn engine_settings_are_finite_and_in_range() {
    refused("nadir_hold_ais", &[("engine.kp", "NaN")], "not a finite number");
    refused("nadir_hold_ais", &[("engine.kp", "12")], "engine.kp = 12");
    refused("nadir_hold_ais", &[("engine.ap", "1510")], "engine.ap = 1510");
    refused("nadir_hold_ais", &[("engine.zonal_max", "-1")], "a whole number from 1 to 6");
    refused("nadir_hold_ais", &[("engine.zonal_max", "2.5")], "a whole number from 1 to 6");
    refused("nadir_hold_ais", &[("engine.igrf_nmax", "40")], "a whole number from 1 to 13");
    refused("nadir_hold_ais", &[("engine.m_res", "[1, NaN, 0]")], "not a [x, y, z] vector");
    refused("nadir_hold_ais", &[("engine.m_res", "[1, 2]")], "needs 3 values");
}

#[test]
fn run_limits_are_named() {
    refused("nadir_hold_ais", &[("engine.duration_s", "0")], "a run lasts more than 0 s");
    refused("nadir_hold_ais", &[("engine.duration_s", "1e9")], "at most 2592000 s");
    refused("nadir_hold_ais", &[("time.dt_s", "0")], "the control step is more than 0");
    refused("nadir_hold_ais", &[("time.record_dt_s", "0.01")], "the record step is at least the control step");
    refused("nadir_hold_ais", &[("engine.mass_kg", "0")], "a mass is positive");
}

#[test]
fn a_case_the_engine_cannot_fly_is_refused_by_name() {
    let d = std::env::temp_dir().join(format!("adcs-inputs-{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    let base = std::fs::read_to_string(root().join("cases/ais_3u.csv")).unwrap();
    let with = |key: &str, val: &str| -> String {
        base.lines().map(|l| {
            let f: Vec<&str> = l.split(',').collect();
            if f.len() > 4 && f[1] == key { let mut g = f.clone(); g[4] = val; g.join(",") } else { l.to_string() }
        }).collect::<Vec<_>>().join("\n") + "\n"
    };
    for (key, val, says) in [("orbit.alt", "36000", "orbit.alt (km) = 36000"), ("orbit.alt", "", "does not state orbit.alt"),
                             ("mass.imin", "-1", "a principal inertia is positive"), ("orbit.alt", "high", "not a finite number")] {
        let f = d.join("case.csv");
        std::fs::write(&f, with(key, val)).unwrap();
        let e = Config::build(&root(), "nadir_hold_ais", &f, 1, &[]).unwrap_err();
        assert!(e.contains(says), "{key}={val}: {e:?}");
    }
    let _ = std::fs::remove_dir_all(&d);
}
