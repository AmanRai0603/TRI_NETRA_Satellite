//! The component-level sensor chains selected by the product (B3.8): the star tracker's
//! `model = "image"` and the Sun sensors' `level = "chain"` read every value from the part and
//! refuse by name one it does not state; an unknown model or level is refused; and a short
//! closed loop flies on them. (Own test binary: it sets ADCS_SIZED_DIR, which the process shares.)
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use adcs_fsw_abi::Impl;
use adcs_sim::{config::Config, product::Dev, run};
use adcs_sim_core::la::*;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn root() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../matlab_sils") }
fn read(p: &Path) -> Value { serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap() }

/// The twin's baseline camera and quadrant head (asils.comp.star_tracker.camera,
/// asils.comp.sun_sensor.head), as a part would state them.
fn camera() -> Value {
    json!({"detector_px": 1024, "psf_sigma_px": 1.2, "flux_mag6_e": 3000.0, "background_e": 50.0, "read_noise_e": 8.0,
           "centroid_k_sigma": 5.0, "max_spots": 20, "id_tol_rad": 2e-4, "id_mag_tol": 0.25, "fit_tol_rad": 1e-4})
}
fn head() -> Value { json!({"aperture_side_m": 1.0e-3, "aperture_height_m": 0.6e-3, "current_noise_frac": 0.005, "current_min_frac": 0.05}) }

struct Sized { d: PathBuf, base: Value }
impl Sized {
    fn new() -> Sized {
        let d = std::env::temp_dir().join(format!("adcs-chains-{}", std::process::id()));
        std::fs::create_dir_all(d.join("products")).unwrap();
        std::fs::create_dir_all(d.join("parts")).unwrap();
        std::env::set_var("ADCS_SIZED_DIR", &d);
        Sized { d, base: read(&root().join("data/products/TRN-P-3U-IMG.json")) }
    }
    /// A part `id` copied from `from` with `add` merged into its nominal values and `drop` removed.
    fn part(&self, id: &str, from: &str, add: &Value, drop: Option<&str>) {
        let mut p = read(&root().join("data/parts").join(format!("{from}.json")));
        for (k, v) in add.as_object().unwrap() { p["nominal"][k] = v.clone(); }
        if let Some(k) = drop { p["nominal"].as_object_mut().unwrap().remove(k); }
        p["part_number"] = id.into();
        std::fs::write(self.d.join("parts").join(format!("{id}.json")), p.to_string()).unwrap();
    }
    /// The imaging product as `id`, its star tracker and Sun sensors on the given parts and settings.
    fn product(&self, id: &str, st: (&str, &str), sun: (&str, &str)) {
        let mut p = self.base.clone();
        p["id"] = id.into();
        for f in p["fill"].as_array_mut().unwrap() {
            if f["slot"] == "star_tracker" { f["part"] = st.0.into(); f["model"] = st.1.into(); }
            if f["slot"] == "sun_sensors" { f["part"] = sun.0.into(); f["level"] = sun.1.into(); }
        }
        std::fs::write(self.d.join("products").join(format!("{id}.json")), p.to_string()).unwrap();
    }
    fn refused(&self, id: &str, says: &str) {
        let m = Dev::load(&root(), id).unwrap_err().message().to_string();
        assert!(m.contains(says), "{id}: {m} does not say {says:?}");
    }
}
impl Drop for Sized {
    fn drop(&mut self) { std::env::remove_var("ADCS_SIZED_DIR"); let _ = std::fs::remove_dir_all(&self.d); }
}

#[test]
fn the_chains_come_from_the_part_and_fly() {
    let s = Sized::new();
    s.part("T-ST-IMG", "SYN-ST-1", &camera(), None);
    s.part("T-SUN-Q", "SYN-SUN-1", &head(), None);
    s.product("T-CHAINS", ("T-ST-IMG", "image"), ("T-SUN-Q", "chain"));
    let d = Dev::load(&root(), "T-CHAINS").unwrap();
    assert_eq!(d.st.model, 2);
    assert_eq!((d.st.cam.n, d.st.cam.max_spots, d.st.cam.psf_px, d.st.cam.fit_tol), (1024, 20, 1.2, 1e-4));
    assert!((d.st.cam.f - 512.0/0.17f64.tan()).abs() < 1e-9 && d.st.cam.c == 512.5);
    assert!(d.sun.chain && d.sun.head.a == 1.0e-3 && d.sun.head.h == 0.6e-3 && d.sun.head.noise == 0.005 && d.sun.head.min_frac == 0.05);
    // the stated levels by their names; the default stays the default
    s.product("T-DEFAULT", ("SYN-ST-1", "quest"), ("SYN-SUN-1", "model"));
    let d = Dev::load(&root(), "T-DEFAULT").unwrap();
    assert!(d.st.model == 1 && !d.sun.chain);
    s.product("T-NOISE", ("SYN-ST-1", "noise"), ("SYN-SUN-1", "model"));
    assert_eq!(Dev::load(&root(), "T-NOISE").unwrap().st.model, 0);

    // each value the chain reads, left out of the part, is named; the shipped parts state none
    s.product("T-SHIPPED", ("SYN-ST-1", "image"), ("SYN-SUN-1", "chain"));
    s.refused("T-SHIPPED", "does not state detector_px");
    for k in camera().as_object().unwrap().keys() {
        s.part("T-ST-EDIT", "SYN-ST-1", &camera(), Some(k));
        s.product("T-EDIT", ("T-ST-EDIT", "image"), ("T-SUN-Q", "chain"));
        s.refused("T-EDIT", &format!("does not state {k}"));
    }
    for k in head().as_object().unwrap().keys() {
        s.part("T-SUN-EDIT", "SYN-SUN-1", &head(), Some(k));
        s.product("T-EDIT", ("T-ST-IMG", "image"), ("T-SUN-EDIT", "chain"));
        s.refused("T-EDIT", &format!("does not state {k}"));
    }
    // out of range: more spots than the engine holds, a fractional detector, a zero aperture
    for (k, v, says) in [("max_spots", json!(40), "max_spots as a whole number from 3 to 32"), ("detector_px", json!(1000.5), "detector_px as a whole number"),
                         ("psf_sigma_px", json!(0.0), "psf_sigma_px above 0"), ("read_noise_e", json!(-1.0), "read_noise_e 0 or more")] {
        let mut c = camera();
        c[k] = v;
        s.part("T-ST-EDIT", "SYN-ST-1", &c, None);
        s.product("T-EDIT", ("T-ST-EDIT", "image"), ("T-SUN-Q", "chain"));
        s.refused("T-EDIT", says);
    }
    let mut h = head();
    h["aperture_height_m"] = json!(0.0);
    s.part("T-SUN-EDIT", "SYN-SUN-1", &h, None);
    s.product("T-EDIT", ("T-ST-IMG", "image"), ("T-SUN-EDIT", "chain"));
    s.refused("T-EDIT", "aperture_height_m above 0");
    // a model or level the engine does not have
    s.product("T-EDIT", ("T-ST-IMG", "pinhole"), ("T-SUN-Q", "chain"));
    s.refused("T-EDIT", "star_tracker model \"pinhole\"; the engine models noise, quest, image");
    s.product("T-EDIT", ("T-ST-IMG", "image"), ("T-SUN-Q", "currents"));
    s.refused("T-EDIT", "sun_sensors level \"currents\"; the engine models model, chain");

    // a closed loop on both chains against the default models: the imaging product under its own
    // id (the sized folder is read first), fine hold for 40 s. The image chain answers on at
    // least 90 % of the frames the star-field model answers on, and the knowledge stays inside
    // the imaging budget's 0.005 deg.
    let fly = |st: (&str, &str), sun: (&str, &str)| {
        s.product("TRN-P-3U-IMG", st, sun);
        let r = root();
        let c = Config::build(&r, "fine_hold_img", &r.join("cases/ais_img_3u.csv"), 1, &[("engine.duration_s".into(), "40".into())]).unwrap();
        let rec = run::run(&c, &run::Opts { fsw: Impl::Rust, quiet: true, realtime: false, oils: None }).unwrap();
        let late: Vec<_> = rec.rows.iter().filter(|x| x.t >= 20.0).collect();
        let answered = late.iter().filter(|x| x.st_ok).count();
        let ake = late.iter().filter_map(|x| x.q_est.map(|e| qangle(&x.q, &e).to_degrees())).fold(0.0, f64::max);
        (c.dev.st.model, answered, ake)
    };
    let (m0, n0, ake0) = fly(("SYN-ST-1", "quest"), ("SYN-SUN-1", "model"));
    let (m1, n1, ake1) = fly(("T-ST-IMG", "image"), ("T-SUN-Q", "chain"));
    assert_eq!((m0, m1), (1, 2));
    assert!(n0 > 0 && n1*10 >= n0*9, "the image chain answered {n1} frames, the star-field model {n0}");
    assert!(ake1 < 0.005, "knowledge {ake1} deg on the image chain ({ake0} deg on the star-field model)");
}
