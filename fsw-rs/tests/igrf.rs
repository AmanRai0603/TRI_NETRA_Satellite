//! The flight software's geomagnetic field against two independent implementations of the same model, IGRF-14
//! (IAGA V-MOD, 2024): ppigrf 2.1.0 (PyPI, MIT; reads IGRF14.shc, its own Legendre recursion and its own WGS-84 conversion)
//! and pyIGRF14 1.0.4 (PyPI, MIT; the IAGA synthesis routine in Python), north-east-down at geodetic latitude, longitude and
//! altitude, degree 13. Values written by `ppigrf.igrf(lon, lat, alt_km, date)` (Bn, Be, -Bu) and by
//! `pyIGRF14.igrf_value(lat, lon, alt_km, year)` (X, Y, Z), in nT. IGRF interpolates its coefficients linearly in the
//! decimal year between its epochs (pyIGRF14 and this software do); ppigrf interpolates in calendar time, so its date is
//! the calendar date that is the same fraction of the way between the two epochs' 1 January 0 h (the epochs' own dates).
//! The two packages agree with each other to 0.0064 nT on these points (their WGS-84 constants differ in the seventh digit).
//! The 50-point reference, at degree 13 and at degree 10 (the onboard degree), is `fifty_points_at_degree_13_and_10` below (S12.6, TS1(f)).
use adcs_fsw::env::{igrf_gh, igrf_ned};

// (lat deg, lon deg, alt km, decimal year, ppigrf [N, E, D], pyIGRF14 [X, Y, Z])
const POINTS: [(f64, f64, f64, f64, [f64; 3], [f64; 3]); 9] = [
    (45.0, 105.0, 500.0, 2027.0, [19098.503040, -1277.925661, 41013.558808], [19098.507867, -1277.925814, 41013.559434]),
    (45.0, 105.0, 500.0, 2025.0, [19113.125119, -1260.924282, 40938.843754], [19113.129942, -1260.924436, 40938.844372]),
    (-33.9, 18.4, 0.0, 2000.0, [9760.924225, -4241.970288, -24136.639111], [9760.925563, -4241.970038, -24136.639844]),
    (89.0, 0.0, 550.0, 2021.25, [1482.303385, -153.089844, 45213.761796], [1482.303850, -153.089717, 45213.767431]),
    (-89.0, -120.0, 400.0, 1995.5, [526.463947, 12766.642518, -44874.514400], [526.463740, 12766.644908, -44874.520690]),
    (0.0, -60.0, 700.0, 2023.7, [18629.894126, -4742.927202, 3769.956969], [18629.894120, -4742.927200, 3769.956968]),
    (51.5, -0.1, 10.0, 2010.0, [19302.618912, -581.813880, 44341.564511], [19302.624229, -581.813732, 44341.565931]),
    (-30.0, -45.0, 550.0, 2024.9, [12539.096193, -3980.558473, -13144.841449], [12539.097272, -3980.558904, -13144.840851]),
    (10.0, 170.0, 1000.0, 1985.3, [21237.673033, 3183.183880, 3349.692974], [21237.673246, 3183.183938, 3349.692069]),
];

#[test]
fn igrf14_matches_two_independent_implementations() {
    let mut bad = vec![];
    for (lat, lon, alt, year, pp, py) in POINTS {
        let b = igrf_ned(&igrf_gh(year), lat.to_radians(), lon.to_radians(), alt, 13);
        for (who, want) in [("ppigrf 2.1.0", pp), ("pyIGRF14 1.0.4", py)] {
            let d = (0..3).map(|k| (b[k] - want[k]).abs()).fold(0.0, f64::max);
            if d >= 0.01 { bad.push(format!("{lat} {lon} {alt} km {year}: {b:?} nT vs {who} {want:?} nT ({d:.4} nT)")); }
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

/// The first point of the table, the one the inventory names: 2027.0, 45 N 105 E, 500 km, to 0.01 nT.
/// (IGRF-13 extrapolated to this date was tens of nT away.)
#[test]
fn the_field_at_2027_45n_105e_500km_is_igrf14s() {
    let b = igrf_ned(&igrf_gh(2027.0), 45f64.to_radians(), 105f64.to_radians(), 500.0, 13);
    for (k, want) in [19098.503040, -1277.925661, 41013.558808].into_iter().enumerate() {
        assert!((b[k] - want).abs() < 0.01, "component {k}: {} vs ppigrf {want}", b[k]);
    }
}

/// The coefficient g(1,0) at 2025.0 is the file's: -29350.0 nT as igrf14coeffs.txt (matlab_sils/data) states it, read here from the
/// file (not from the generated table) so a table that is not the file's fails.
#[test]
fn g10_at_2025_is_the_files() {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../matlab_sils/data/igrf14coeffs.txt")).unwrap();
    let head = text.lines().find(|l| l.starts_with("g/h")).unwrap().split_whitespace().collect::<Vec<_>>();
    let col = head.iter().position(|x| *x == "2025.0").unwrap() - 3;
    let row = text.lines().find(|l| l.starts_with("g ") && l.split_whitespace().nth(1) == Some("1") && l.split_whitespace().nth(2) == Some("0")).unwrap();
    let want: f64 = row.split_whitespace().nth(3 + col).unwrap().parse().unwrap();
    assert_eq!(igrf_gh(2025.0)[0], want);
}

/// tests/data/igrf14_reference.txt (S12.6, TS1(f)): 50 points, 2025.0 to 2030.0, 300 to 800 km and near the poles, each computed by ppigrf 2.1.0
/// and pyIGRF14 1.0.4 at degree 13 and cut to degree 10 (their versions and the grid are in the file's header and its generator). The
/// software's field at degree 13 is held to the degree-13 values and at degree 10, the onboard degree (fsw_param_igrf_nmax), to the
/// degree-10 values, each package and component within 1 nT; the packages within 0.01 nT of each other. (Degree 10 against the degree-13
/// values would be up to 35.9 nT off: the truncation is the reference's, not allowed for by a wider limit.)
#[test]
fn fifty_points_at_degree_13_and_10() {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../tests/data/igrf14_reference.txt")).unwrap();
    let (mut n, mut bad, mut worst, mut spread) = (0, vec![], [0.0f64; 2], 0.0f64);
    for l in text.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()) {
        let v: Vec<f64> = l.split_whitespace().map(|x| x.parse().unwrap()).collect();
        assert_eq!(v.len(), 17, "{l}");
        n += 1;
        let gh = igrf_gh(v[1]);
        for (deg, nmax) in [13i32, 10].into_iter().enumerate() {
            let b = igrf_ned(&gh, v[2].to_radians(), v[3].to_radians(), v[4], nmax);
            for who in 0..2 {
                for k in 0..3 {
                    let want = v[5 + 6 * deg + 3 * who + k];
                    let d = (b[k] - want).abs();
                    worst[deg] = worst[deg].max(d);
                    if d >= 1.0 { bad.push(format!("point {} degree {nmax} package {who} component {k}: {} vs {want} nT", v[0], b[k])); }
                }
            }
            for k in 0..3 { spread = spread.max((v[5 + 6 * deg + k] - v[5 + 6 * deg + 3 + k]).abs()); }
        }
    }
    println!("worst difference from the references: {:.6} nT at degree 13, {:.6} nT at degree 10; the packages differ by {spread:.6} nT", worst[0], worst[1]);
    assert_eq!(n, 50);
    assert!(spread < 0.01, "the packages differ by {spread} nT");
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}
