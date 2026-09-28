//! ECEF -> geodetic -- port of `matlab_sils/pop/01_core/+op/geodetic.m`
//! (Bowring-style fixed 5-iteration solution; WGS84 defaults from
//! `de440.constants`: `Re_earth`, `f_earth`). POP has no inverse (geodetic -> ECEF).
use crate::la::V3;

/// `de440.constants().Re_earth`: WGS84 equatorial radius (m).
pub const RE_EARTH: f64 = 6378137.0;
/// `de440.constants().f_earth`: WGS84 flattening.
pub const F_EARTH: f64 = 1.0 / 298.257223563;

/// `[lat, lon, alt] = op.geodetic(r_ecef)`: WGS84 geodetic latitude, longitude (rad)
/// and altitude (m).
pub fn geodetic(r_ecef: &V3) -> (f64, f64, f64) { geodetic_with(r_ecef, RE_EARTH, F_EARTH) }

/// `[lat, lon, alt] = op.geodetic(r_ecef, Re, f)` with an explicit ellipsoid.
pub fn geodetic_with(r_ecef: &V3, re: f64, f: f64) -> (f64, f64, f64) {
    let (x, y, z) = (r_ecef[0], r_ecef[1], r_ecef[2]);
    let e2 = f * (2.0 - f);
    let lon = y.atan2(x);
    let p = x.hypot(y);
    let mut lat = z.atan2(p * (1.0 - e2));
    for _ in 0..5 {
        let sph = lat.sin();
        let n = re / (1.0 - e2 * sph.powf(2.0)).sqrt();
        let alt = p / lat.cos() - n;
        lat = z.atan2(p * (1.0 - e2 * n / (n + alt)));
    }
    let sph = lat.sin();
    let n = re / (1.0 - e2 * sph.powf(2.0)).sqrt();
    let alt = p / lat.cos() - n;
    (lat, lon, alt)
}
