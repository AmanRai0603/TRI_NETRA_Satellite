//! The flight software's geomagnetic field against an independent implementation of the same
//! model: pyIGRF 0.3.3 (PyPI; IGRF-13 coefficients, its own Python port of the IAGA igrf13syn
//! routine), north-east-down at geodetic latitude, longitude and altitude. Values written by
//! `pyIGRF.igrf_value(lat, lon, alt_km, year)` (X, Y, Z in nT); they agree to well under 1 nT.
use adcs_fsw::env::{igrf_gh, igrf_ned};

const PY_IGRF: [(f64, f64, f64, f64, [f64; 3]); 8] = [
    (45.0, 105.0, 500.0, 2025.0, [19086.817231, -1323.853275, 41013.823279]),
    (-33.9, 18.4, 0.0, 2000.0, [9760.925563, -4241.970038, -24136.639844]),
    (89.0, 0.0, 550.0, 2021.25, [1475.983066, -151.000042, 45215.519863]),
    (-89.0, -120.0, 400.0, 1995.5, [526.463740, 12766.644908, -44874.520690]),
    (0.0, -60.0, 700.0, 2023.7, [18646.618145, -4742.785060, 3748.730254]),
    (51.5, -0.1, 10.0, 2010.0, [19302.624229, -581.813732, 44341.565931]),
    (-30.0, -45.0, 550.0, 2024.9, [12523.298120, -4007.903593, -13200.173089]),
    (10.0, 170.0, 1000.0, 1985.3, [21237.673246, 3183.183938, 3349.692069]),
];

#[test]
fn igrf13_matches_an_independent_implementation() {
    let mut bad = vec![];
    for (lat, lon, alt, year, want) in PY_IGRF {
        let b = igrf_ned(&igrf_gh(year), lat.to_radians(), lon.to_radians(), alt, 13);
        let d = (0..3).map(|k| (b[k] - want[k]).abs()).fold(0.0, f64::max);
        if d >= 0.01 { bad.push(format!("{lat} {lon} {alt} km {year}: {b:?} nT vs pyIGRF {want:?} nT")); }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}
