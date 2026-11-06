//! M10: the receiver's rate_Hz is honoured in the run schedule: 0.1 Hz is one fix per 10 s, 1 Hz one per second (it was one per
//! second whatever the part stated), and a part stating no rate above 0 is refused by name. (Own test binary: it sets ADCS_SIZED_DIR,
//! which the process shares.)
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use adcs_sim::{config::Config, product::Dev, run};
use serde_json::Value;
use std::path::{Path, PathBuf};

fn root() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../matlab_sils") }
fn read(p: &Path) -> Value { serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap() }

#[test]
fn the_gnss_rate_sets_the_fix_period_of_the_run() {
    let d = std::env::temp_dir().join(format!("adcs-gnss-{}", std::process::id()));
    std::fs::create_dir_all(d.join("products")).unwrap();
    std::fs::create_dir_all(d.join("parts")).unwrap();
    std::env::set_var("ADCS_SIZED_DIR", &d);
    let r = root();
    let base = read(&r.join("data/products/TRN-P-3U-IMG.json"));
    // the imaging product with its receiver replaced by a copy of the shipped one with the given rate (None: no rate stated)
    let product_with = |id: &str, rate: Option<f64>| {
        let mut part = read(&r.join("data/parts/TRN-GPS-1.json"));
        part["part_number"] = id.into();
        match rate { Some(x) => part["nominal"]["rate_Hz"] = x.into(), None => { part["nominal"].as_object_mut().unwrap().remove("rate_Hz"); } }
        std::fs::write(d.join("parts").join(format!("{id}.json")), part.to_string()).unwrap();
        let mut p = base.clone();
        p["id"] = format!("P-{id}").into();
        for f in p["fill"].as_array_mut().unwrap() { if f["slot"] == "gnss" { f["part"] = id.into(); } }
        std::fs::write(d.join("products").join(format!("P-{id}.json")), p.to_string()).unwrap();
    };
    let period = |id: &str, rate: f64| -> f64 {
        product_with(id, Some(rate));
        // the scenario flown with the sized product under its own id
        let mut sc = read(&r.join("data/scenarios/fine_hold_img.json"));
        sc["product"] = format!("P-{id}").into();
        let f = d.join(format!("{id}.json"));
        std::fs::write(&f, sc.to_string()).unwrap();
        let c = Config::build(&r, &f.display().to_string(), &r.join("cases/ais_img_3u.csv"), 1, &[("engine.duration_s".into(), "40".into())]).unwrap();
        run::gnss_fix_period_s(&c)
    };
    assert_eq!(period("T-GPS-1HZ", 1.0), 1.0);
    assert_eq!(period("T-GPS-SLOW", 0.1), 10.0, "rate_Hz 0.1: one fix per 10 s");
    assert_eq!(period("T-GPS-FAST", 2.0), 0.5);
    // a receiver stating no rate, or none above 0, is refused by name, not given the loop's rate
    product_with("T-GPS-NONE", None);
    let m = Dev::load(&r, "P-T-GPS-NONE").unwrap_err().message().to_string();
    assert!(m.contains("does not state rate_Hz"), "{m}");
    product_with("T-GPS-ZERO", Some(0.0));
    let m = Dev::load(&r, "P-T-GPS-ZERO").unwrap_err().message().to_string();
    assert!(m.contains("rate_Hz above 0"), "{m}");
    std::env::remove_var("ADCS_SIZED_DIR");
    let _ = std::fs::remove_dir_all(&d);
}
