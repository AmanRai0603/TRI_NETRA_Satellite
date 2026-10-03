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
