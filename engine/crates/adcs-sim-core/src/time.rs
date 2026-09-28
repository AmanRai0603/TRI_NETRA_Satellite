//! Calendar and sidereal time (asils.util.jd / jd2utc / decyear; GMST IAU-82).
use crate::la::M3;
use crate::pm::*;

/// Julian date of a UTC calendar date [Y M D h m s] (valid 1901-2099).
pub fn jd(u: &[f64; 6]) -> f64 {
    let (y, m, d) = (u[0], u[1], u[2]);
    367.0*y - floor(7.0*(y + floor((m + 9.0)/12.0))/4.0) + floor(275.0*m/9.0) + d + 1721013.5 + ((u[5]/60.0 + u[4])/60.0 + u[3])/24.0
}
pub fn jd2utc(jd: f64) -> [f64; 6] {
    let z = floor(jd + 0.5);
    let f = jd + 0.5 - z;
    let a = floor((z - 1867216.25)/36524.25);
    let aa = z + 1.0 + a - floor(a/4.0);
    let b = aa + 1524.0;
    let c = floor((b - 122.1)/365.25);
    let d = floor(365.25*c);
    let e = floor((b - d)/30.6001);
    let day = b - d - floor(30.6001*e);
    let mon = if e < 14.0 { e - 1.0 } else { e - 13.0 };
    let yr = if mon > 2.0 { c - 4716.0 } else { c - 4715.0 };
    let mut s = f*86400.0;
    let h = floor(s/3600.0); s -= 3600.0*h;
    let mi = floor(s/60.0); s -= 60.0*mi;
    [yr, mon, day, h, mi, s]
}
pub fn decyear(jd_: f64) -> f64 {
    let u = jd2utc(jd_);
    let j0 = jd(&[u[0], 1.0, 1.0, 0.0, 0.0, 0.0]);
    let j1 = jd(&[u[0] + 1.0, 1.0, 1.0, 0.0, 0.0, 0.0]);
    u[0] + (jd_ - j0)/(j1 - j0)
}
/// GMST [rad] (IAU-82, UT1 = UTC).
pub fn gmst(jd_ut1: f64) -> f64 {
    let t = (jd_ut1 - 2451545.0)/36525.0;
    let mut g = fmod(67310.54841 + (876600.0*3600.0 + 8640184.812866)*t + 0.093104*t*t - 6.2e-6*t*t*t, 86400.0);
    if g < 0.0 { g += 86400.0; }
    g/240.0*D2R
}
/// ECI -> ECEF.
pub fn eci2ecef(jd_ut1: f64) -> M3 {
    let g = gmst(jd_ut1);
    [[cos(g), sin(g), 0.0], [-sin(g), cos(g), 0.0], [0.0, 0.0, 1.0]]
}
/// TT - UTC [s] for the 2017-2035 leap-second epoch (TAI - UTC = 37 s).
pub const TT_MINUS_UTC_S: f64 = 69.184;
