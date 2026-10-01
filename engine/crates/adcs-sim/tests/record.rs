//! From a product to a stored record: every product loads, a flight's metrics are judged as the
//! case says, the record carries its provenance, and a share file brings it back unchanged.
use adcs_sim::config::Config;
use adcs_sim::product::Dev;
use adcs_sim::{flight, run, store};
use std::path::PathBuf;

fn root() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../matlab_sils") }

fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("adcs-record-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

#[test]
fn every_shipped_product_loads_and_names_its_files() {
    let mut n = 0;
    for e in std::fs::read_dir(root().join("data/products")).unwrap().flatten() {
        let id = e.path().file_stem().unwrap().to_string_lossy().into_owned();
        let d = Dev::load(&root(), &id).unwrap_or_else(|err| panic!("product {id}: {err}"));
        assert!(!d.files.is_empty() && d.files.iter().all(|f| f.is_file()), "{id}: the files it was read from exist");
        assert!(d.mex.n <= adcs_sim_core::NR && d.mex.ng <= adcs_sim_core::NG, "{id}: within capacity");
        for i in 0..d.mex.n {
            let a = d.mex.a0[i];
            assert!(((a[0]*a[0] + a[1]*a[1] + a[2]*a[2]).sqrt() - 1.0).abs() < 1e-9, "{id}: rotor {i} axis is a unit vector");
        }
        n += 1;
    }
    assert!(n > 0, "the products folder holds products");
}

#[test]
fn the_case_a_scenario_flies_is_found_or_refused_by_name() {
    let r = root();
    assert_eq!(flight::case_file(&r, "nadir_hold_ais", None).unwrap(), r.join("cases/ais_3u.csv"));
    assert_eq!(flight::case_file(&r, "nadir_hold_ais", Some("ais_img_3u")).unwrap(), r.join("cases/ais_img_3u.csv"));
    assert!(flight::case_file(&r, "nadir_hold_ais", Some("no_such_case")).unwrap_err().message().contains("no case no_such_case"));
    assert!(flight::case_file(&r, "no_such_scenario", None).unwrap_err().message().contains("no scenario"));
}

#[test]
fn a_flight_is_judged_recorded_and_shared() {
    let r = root();
    let case = flight::case_file(&r, "nadir_hold_ais", None).unwrap();
    let set = [("engine.duration_s".to_string(), "120".to_string())];
    let c = Config::build(&r, "nadir_hold_ais", &case, 1, &set).unwrap();
    let out = tmp("flight");
    let f = flight::fly(&c, &run::Opts { fsw: adcs_fsw_abi::Impl::C, quiet: true, realtime: false, oils: None }, &out).unwrap();

    // every metric the scenario declares, judged against the case's requirement
    let declared = c.scenario["metrics"].as_array().unwrap().len();
    assert_eq!(f.metrics.len(), declared);
    for m in &f.metrics {
        if let (Some(v), Some(req)) = (m["value"].as_f64(), m["req"].as_f64()) {
            let lower_is_better = !matches!(m["kind"].as_str(), Some("mode_fraction"));
            if lower_is_better { assert_eq!(m["pass"].as_i64(), Some((v <= req) as i64), "{}: {v} vs {req}", m["id"]); }
        }
    }

    // the record: channels as many as the rows say, the provenance, the inputs kept
    let man: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(out.join("manifest.json")).unwrap()).unwrap();
    for k in ["result_id", "created_utc", "engine_version", "engine_source", "product_fingerprint", "inputs", "assumptions"] {
        assert!(!man[k].is_null(), "the manifest records {k}");
    }
    assert_eq!(man["engine_source"], store::ENGINE_SOURCE);
    let csv = std::fs::read_to_string(out.join("channels.csv")).unwrap();
    let mut lines = csv.lines();
    let cols = lines.next().unwrap().split(',').count();
    assert!(lines.clone().count() >= 100 && lines.all(|l| l.split(',').count() == cols), "every row has the header's columns");
    assert!(store::stale(&store::Found { dir: out.clone(), m: man.clone() }).is_empty(), "a fresh run is not stale");

    // a share file and back: the same bytes
    let share = out.with_extension("trinetra");
    store::export(&out, &share).unwrap();
    let back = tmp("imported");
    store::import(&share, &back).unwrap();
    for f in ["manifest.json", "channels.csv"] {
        assert_eq!(std::fs::read(out.join(f)).unwrap(), std::fs::read(back.join(f)).unwrap(), "{f} comes back unchanged");
    }
    assert!(store::show(&back).unwrap().contains("nadir_hold_ais"));
    let _ = (std::fs::remove_dir_all(&out), std::fs::remove_dir_all(&back), std::fs::remove_file(&share));
}
