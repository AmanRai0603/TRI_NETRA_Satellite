//! The engine's truth field against the committed IGRF-14 reference (tests/data/igrf14_reference.txt, S12.6, TS1(f)): 50 points computed by
//! two independent implementations, ppigrf 2.1.0 and pyIGRF14 1.0.4, at degree 13 and cut to degree 10, each held to 1 nT; the two packages
//! to each other, 0.01 nT. The engine's field is `field::gh` (the design's IGRF-14 table, generated) and `field::ned`; the truth's own call
//! (`field::eci_at`, north-east-down turned to J2000 by the Earth-fixed matrix) is held to the same numbers by its magnitude.
use adcs_sim_core::field::{eci_at, gh, ned};
use adcs_sim_core::la::*;

/// (n, year, lat, lon, alt, [pp13 N E D, py13 N E D, pp10 N E D, py10 N E D])
fn reference() -> Vec<(usize, f64, f64, f64, f64, [f64; 12])> {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../tests/data/igrf14_reference.txt")).unwrap();
    text.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()).map(|l| {
        let v: Vec<f64> = l.split_whitespace().map(|x| x.parse().unwrap()).collect();
        assert_eq!(v.len(), 17, "{l}");
        let mut c = [0.0; 12];
        c.copy_from_slice(&v[5..]);
        (v[0] as usize, v[1], v[2], v[3], v[4], c)
    }).collect()
}

#[test]
fn the_reference_is_fifty_points_on_which_the_two_packages_agree() {
    let r = reference();
    assert_eq!(r.len(), 50);
    let mut spread = 0.0f64;
    for (_n, _y, _la, _lo, _h, c) in &r {
        for deg in 0..2 { for k in 0..3 { spread = spread.max((c[6 * deg + k] - c[6 * deg + 3 + k]).abs()); } }
    }
    assert!(spread < 0.01, "the packages differ by {spread} nT");
    // the dates span 2025.0 to 2030.0 and the altitudes 300 to 800 km, and the grid has points near both poles
    assert!(r.iter().any(|p| p.1 == 2025.0) && r.iter().any(|p| p.1 == 2030.0) && r.iter().all(|p| p.1 >= 2025.0 && p.1 <= 2030.0));
    assert!(r.iter().all(|p| p.4 >= 300.0 && p.4 <= 800.0));
    assert!(r.iter().filter(|p| p.2.abs() > 89.0).count() >= 4);
}

#[test]
fn the_truth_field_is_within_1_nt_of_both_packages_at_degree_13_and_10() {
    let mut bad = vec![];
    let mut worst = [0.0f64; 2];
    for (n, year, lat, lon, alt, c) in reference() {
        let g = gh(year);
        for (deg, nmax) in [13usize, 10].into_iter().enumerate() {
            let b = ned(&g, lat.to_radians(), lon.to_radians(), alt, nmax);
            for who in 0..2 {
                for k in 0..3 {
                    let d = (b[k] - c[6 * deg + 3 * who + k]).abs();
                    worst[deg] = worst[deg].max(d);
                    if d >= 1.0 { bad.push(format!("point {n} degree {nmax} package {who} component {k}: {} vs {} nT", b[k], c[6 * deg + 3 * who + k])); }
                }
            }
        }
    }
    println!("worst difference from the references: {:.6} nT at degree 13, {:.6} nT at degree 10", worst[0], worst[1]);
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

/// The truth's own call: the field in J2000 at geodetic (lat, lon, h) through an Earth-fixed rotation has the reference's magnitude.
#[test]
fn the_truth_orbits_field_in_j2000_has_the_references_magnitude() {
    let (cz, sz) = (0.7f64.cos(), 0.7f64.sin());
    let c: M3 = [[cz, sz, 0.0], [-sz, cz, 0.0], [0.0, 0.0, 1.0]];       // any rotation: the magnitude does not depend on it
    for (n, year, lat, lon, alt, v) in reference() {
        let b = eci_at(lat.to_radians(), lon.to_radians(), alt * 1e3, &c, &gh(year), 13, None);
        let mag = (b[0] * b[0] + b[1] * b[1] + b[2] * b[2]).sqrt() * 1e9;
        let want = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        assert!((mag - want).abs() < 1.0, "point {n}: |B| {mag} vs ppigrf {want} nT");
    }
}
