//! The power budget's array model against its closed form: the Sun on one face, between two, in
//! eclipse; the case's power system read all or none.
use adcs_sim::metrics::{PowerSystem, SOLAR_CONSTANT};

fn ps() -> PowerSystem { PowerSystem { area: [0.0, 0.0, 0.03, 0.0, 0.02, 0.0], eff: 0.25, batt_wh: 30.0, load_w: 2.5, soc0: 1.0 } }

#[test]
fn the_arrays_give_the_sun_on_their_faces() {
    let p = ps();
    assert!((p.generation(&[0.0, 1.0, 0.0], 1.0) - SOLAR_CONSTANT*0.25*0.03).abs() < 1e-12, "+Y face square to the Sun");
    let s = 1.0/2f64.sqrt();
    assert!((p.generation(&[0.0, s, s], 1.0) - SOLAR_CONSTANT*0.25*(0.03 + 0.02)*s).abs() < 1e-12, "between +Y and +Z");
    assert_eq!(p.generation(&[0.0, -1.0, 0.0], 1.0), 0.0, "a face without cells, the others edge-on or away");
    assert_eq!(p.generation(&[0.0, 1.0, 0.0], 0.0), 0.0, "eclipse");
    assert!((p.generation(&[0.0, 1.0, 0.0], 0.4) - 0.4*SOLAR_CONSTANT*0.25*0.03).abs() < 1e-12, "penumbra");
}

#[test]
fn the_shipped_cases_state_a_whole_power_system() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../matlab_sils/cases");
    for c in ["ais_3u", "ais_img_3u"] {
        let case = adcs_sim::case::Case::read(&root.join(format!("{c}.csv"))).unwrap();
        assert!(PowerSystem::from(&case).is_some(), "{c}");
    }
}

/// The ADCS's power is the whole ADCS's (S11.8, Y1; DR-design-003): the sensors' steady power, by flight mode, is in the
/// run's power channel and in power_mean, which counted only the coils, rotors and thrusters.
#[test]
fn the_sensors_steady_power_is_in_the_runs_power() {
    use adcs_fsw_abi::Impl;
    use adcs_sim::{config::Config, metrics, run};
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../matlab_sils");
    let c = Config::build(&root, "nadir_hold_ais", &root.join("cases/ais_3u.csv"), 1, &[("engine.duration_s".into(), "120".into())]).unwrap();
    // TRN-P-3U-AIS fits a magnetometer (0.1 W), six Sun sensors (6 x 0.05 W), a gyro (0.3 W) and a GNSS receiver (0.8 W): 1.5 W
    let w = 0.1 + 6.0*0.05 + 0.3 + 0.8;
    assert!((c.dev.sens_w.iter().sum::<f64>() - w).abs() < 1e-12, "{:?}", c.dev.sens_w);
    let rec = run::run(&c, &run::Opts { fsw: Impl::Rust, quiet: true, realtime: false, oils: None }).unwrap();
    assert!(rec.rows.iter().all(|r| (r.p_sens - w).abs() < 1e-12), "every sample, in the mode the run flies, carries the sensors");
    // power_mean is the actuators' mean plus the sensors'
    let acts = rec.rows.iter().map(|r| r.p_mtq + r.p_rw + r.p_rcs).sum::<f64>()/rec.rows.len() as f64;
    let d = metrics::derive(&c, &rec);
    let v = metrics::evaluate(&c, &rec, &d);
    let mean = v.iter().find(|m| m["id"] == "power_mean").and_then(|m| m["value"].as_f64()).unwrap();
    assert!((mean - (acts + w)).abs() < 1e-9, "power_mean {mean} against actuators {acts} + sensors {w}");
    assert!(mean - acts >= 1.5 - 1e-9, "at least the sensors' 1.5 W more than before");
}
