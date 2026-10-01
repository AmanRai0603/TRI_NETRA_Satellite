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

fn build(s: &str, sets: &[(&str, &str)]) -> Result<Config, adcs_sim::Error> {
    let sets: Vec<(String, String)> = sets.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
    Config::build(&root(), s, &case_for(s), 1, &sets)
}

fn refused(s: &str, sets: &[(&str, &str)], says: &str) {
    match build(s, sets) {
        Ok(_) => panic!("{sets:?} was accepted"),
        Err(e) => {
            assert!(e.message().contains(says), "{sets:?}: {e:?} does not say {says:?}");
            assert_eq!(e.kind, adcs_sim::Kind::Refused, "{sets:?}: refused, not failed");
        }
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
    assert!(Config::build(&root(), "../../etc/passwd", &root().join("cases/ais_3u.csv"), 1, &[]).unwrap_err().message().contains("not an id"));
}

#[test]
fn a_missing_scenario_is_named() {
    assert!(Config::build(&root(), "no_such_scenario", &root().join("cases/ais_3u.csv"), 1, &[]).unwrap_err().message().contains("no scenario no_such_scenario"));
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
                             ("mass.imin", "-1", "a principal inertia is positive"), ("orbit.alt", "high", "not a finite number"),
                             ("power.eff", "", "states part of its power system: power.eff missing"), ("power.eff", "1.5", "power.eff = 1.5"),
                             ("power.soc0", "-0.1", "power.soc0 = -0.1"),
                             // a stated value nothing models: refused, never quietly dropped
                             ("mass.iunc", "0.1", "states mass.iunc = 0.1"), ("pointing.et", "0.001", "states pointing.et"),
                             ("surface.cps", "0.05", "one centre-of-mass offset for both torques"),
                             ("surface.asun", "0.1", "no deployables are modelled")] {
        let f = d.join("case.csv");
        std::fs::write(&f, with(key, val)).unwrap();
        let e = Config::build(&root(), "nadir_hold_ais", &f, 1, &[]).unwrap_err();
        assert!(e.message().contains(says) && e.kind == adcs_sim::Kind::Refused, "{key}={val}: {e:?}");
    }
    let _ = std::fs::remove_dir_all(&d);
}

/// A copy of a shipped scenario with one change, flown from a temporary file.
fn edited(name: &str, edit: impl Fn(&mut serde_json::Value)) -> Result<Config, adcs_sim::Error> {
    let mut v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(root().join("data/scenarios").join(format!("{name}.json"))).unwrap()).unwrap();
    edit(&mut v);
    static N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let f = std::env::temp_dir().join(format!("adcs-scen-{}-{}.json", std::process::id(), N.fetch_add(1, std::sync::atomic::Ordering::Relaxed)));
    std::fs::write(&f, v.to_string()).unwrap();
    let r = Config::build(&root(), &f.display().to_string(), &case_for(name), 1, &[]);
    let _ = std::fs::remove_file(&f);
    r
}

fn says(r: Result<Config, adcs_sim::Error>, words: &str) {
    match r {
        Ok(_) => panic!("accepted; expected a refusal saying {words:?}"),
        Err(e) => {
            assert!(e.message().contains(words), "{e:?} does not say {words:?}");
            assert_eq!(e.kind, adcs_sim::Kind::Refused, "{e:?}: refused, not failed");
        }
    }
}

#[test]
fn a_scenario_key_the_engine_does_not_read_is_refused() {
    says(edited("nadir_hold_ais", |v| { v["fsw"]["rw_bandwith"] = 0.5.into(); }), "fsw.rw_bandwith: the engine does not read this key");
    says(edited("nadir_hold_ais", |v| { v["tiem"] = serde_json::json!({"dt_s": 0.1}); }), "tiem: the engine does not read this key");
    says(edited("nadir_hold_ais", |v| { v["time"]["dt_s"] = "0.5".into(); }), "time.dt_s = \"0.5\": must be a finite number");
    says(edited("nadir_hold_ais", |v| { v["fsw"]["start_mode"] = "nadir".into(); }), "fsw.start_mode = \"nadir\": must be a mode");
    says(edited("nadir_hold_ais", |v| { v["fsw"]["yaw_flip"] = 0.5.into(); }), "fsw.yaw_flip = 0.5: must be true or false");
    says(edited("nadir_hold_ais", |v| { v["initial"]["rate"]["kind"] = "spin".into(); }), "initial.rate.kind");
}

#[test]
fn an_override_on_a_key_the_engine_does_not_read_is_refused() {
    refused("nadir_hold_ais", &[("fsw.rw_bandwith", "0.5")], "the engine does not read this key");
    refused("nadir_hold_ais", &[("fsw.algorithms.pointng", "\"lqr\"")], "the engine does not read this key");
    build("nadir_hold_ais", &[("fsw.rw_bandwidth", "0.5"), ("fsw.algorithms.mtq_pointing", "\"mtq_lqr\"")]).unwrap();
}

#[test]
fn a_metric_that_cannot_be_judged_as_written_is_refused() {
    says(edited("nadir_hold_ais", |v| { v["metrics"][0]["requirement"] = "req.apee".into(); }), "has no such requirement");
    says(edited("nadir_hold_ais", |v| { v["metrics"][0]["kind"] = "apee".into(); }), "must be one of");
    says(edited("nadir_hold_ais", |v| { v["metrics"][0]["window"] = "last_orbits".into(); }), "after_s:<seconds>");
    says(edited("nadir_hold_ais", |v| { v["metrics"][0]["window"] = "after_s:soon".into(); }), "after_s:<seconds>");
    says(edited("nadir_hold_ais", |v| { v["metrics"][0]["statistic"] = "p99".into(); }), "must be one of");
    // jitter is computed now (adcs_sim::metrics::jitter), so it may be judged
    edited("nadir_hold_ais", |v| { v["metrics"].as_array_mut().unwrap().push(serde_json::json!({"id": "j", "kind": "jitter", "limit": 1.0})); }).unwrap();
    // a diagnostic reports without judging: one that also judges is refused, its reason must be text
    says(edited("nadir_hold_ais", |v| { v["metrics"][1]["diagnostic"] = "just looking".into(); }), "judged, so not a diagnostic");
    says(edited("nadir_hold_ais", |v| { v["metrics"][0]["diagnostic"] = 1.0.into(); }), "diagnostic");
}

#[test]
fn an_algorithm_the_engine_does_not_fly_is_refused() {
    says(edited("nadir_hold_ais", |v| { v["fsw"]["algorithms"] = serde_json::json!({"pointng": "pid"}); }), "no such slot");
    says(edited("nadir_hold_ais", |v| { v["fsw"]["algorithms"] = serde_json::json!({"allocation": "pd_alloc"}); }), "the engine does not fly it");
}

#[test]
fn limits_of_the_flight_software_are_refused_not_cut() {
    says(edited("nadir_hold_ais", |v| {
        v["fsw"]["schedule"] = serde_json::Value::Array((0..9).map(|k| serde_json::json!({"t_s": k as f64 * 10.0, "mode": "nadir_mtq"})).collect());
    }), "the flight software holds at most 8");
    says(edited("fault_gyro_ais", |v| { v["faults"][0]["value"] = 0.001.into(); }), "value must be the bias step");
    says(edited("nadir_hold_ais", |v| { v["faults"] = serde_json::json!([{"t_s": 10.0, "kind": "rotor_fail", "index": 1}]); }), "has no rotors");
    says(edited("nadir_hold_ais", |v| { v["faults"] = serde_json::json!([{"t_s": 10.0, "kind": "coil_fail", "index": 9}]); }), "magnetorquer coils 1 to 3");
    says(edited("nadir_hold_ais", |v| { v["faults"] = serde_json::json!([{"t_s": 10.0, "kind": "coil_fail", "index": 0}]); }), "a unit number from 1");
}

#[test]
fn a_product_beyond_the_engines_capacity_or_missing_a_value_is_refused() {
    use adcs_sim::product::Dev;
    let d = std::env::temp_dir().join(format!("adcs-sized-{}", std::process::id()));
    std::fs::create_dir_all(d.join("products")).unwrap();
    std::fs::create_dir_all(d.join("parts")).unwrap();
    std::env::set_var("ADCS_SIZED_DIR", &d);
    let base: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(root().join("data/products/TRN-P-3U-RW-RCS.json")).unwrap()).unwrap();
    let put = |id: &str, v: &serde_json::Value| std::fs::write(d.join("products").join(format!("{id}.json")), v.to_string()).unwrap();
    // nine wheels
    let mut p = base.clone();
    p["id"] = "T-NINE".into();
    for f in p["fill"].as_array_mut().unwrap() {
        if f["slot"] == "wheels" { f["axes_body"] = serde_json::Value::Array((0..9).map(|_| serde_json::json!([1, 0, 0])).collect()); }
    }
    put("T-NINE", &p);
    assert!(Dev::load(&root(), "T-NINE").unwrap_err().message().contains("9 rotors; the engine holds at most 8"));
    // a wheel part that does not state its torque
    let wheel = base["fill"].as_array().unwrap().iter().find(|f| f["slot"] == "wheels").unwrap()["part"].as_str().unwrap().to_string();
    let mut part: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(root().join("data/parts").join(format!("{wheel}.json"))).unwrap()).unwrap();
    part["nominal"].as_object_mut().unwrap().remove("torque_max_Nm");
    std::fs::write(d.join("parts").join("T-WHEEL.json"), part.to_string()).unwrap();
    let mut p = base.clone();
    p["id"] = "T-NOTORQUE".into();
    for f in p["fill"].as_array_mut().unwrap() { if f["slot"] == "wheels" { f["part"] = "T-WHEEL".into(); } }
    put("T-NOTORQUE", &p);
    assert!(Dev::load(&root(), "T-NOTORQUE").unwrap_err().message().contains("does not state torque_max_Nm"));
    std::env::remove_var("ADCS_SIZED_DIR");
    let _ = std::fs::remove_dir_all(&d);
}

/// The body is the class's (catalogue/classes.toml): a case naming no class, or one not there, is
/// refused; the surface settings are refused outside [0, 1].
#[test]
fn the_body_comes_from_the_class_the_case_names() {
    let c = Config::build(&root(), "nadir_hold_ais", &case_for("nadir_hold_ais"), 1, &[]).unwrap();
    assert_eq!(c.box_m, [0.34, 0.10, 0.10]);
    let text = std::fs::read_to_string(case_for("nadir_hold_ais")).unwrap();
    for (class, words) in [("", "meta.class is blank"), ("cubesat_9u", "no class in catalogue/classes.toml")] {
        let f = std::env::temp_dir().join(format!("adcs-class-{}-{}.csv", std::process::id(), class.len()));
        let edited: String = text.lines().map(|l| if l.starts_with("meta,meta.class,") {
            let mut x: Vec<String> = l.split(',').map(String::from).collect(); x[4] = class.into(); x.join(",")
        } else { l.to_string() }).collect::<Vec<_>>().join("\n");
        std::fs::write(&f, edited).unwrap();
        let r = Config::build(&root(), "nadir_hold_ais", &f, 1, &[]);
        let _ = std::fs::remove_file(&f);
        says(r, words);
    }
    let set = [("engine.vb_ratio".to_string(), "1.5".to_string())];
    says(Config::build(&root(), "nadir_hold_ais", &case_for("nadir_hold_ais"), 1, &set), "engine.vb_ratio");
}
