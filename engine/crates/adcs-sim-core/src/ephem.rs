//! Low-precision Sun and Moon (Montenbruck & Gill 2000, §3.3.2), mean equator and
//! equinox of J2000, metres. The MATLAB twin reads DE440 through the POP; the
//! difference (Sun ~0.01 deg, Moon ~0.1 deg) is a parity-ledger line.
use crate::la::V3;
use crate::pm::*;

pub const AU: f64 = 149_597_870_700.0;
pub const EPS_J2000: f64 = 23.43929111*D2R;

fn ecl2eq(l: f64, b: f64, r: f64) -> V3 {
    let (x, y, z) = (r*cos(b)*cos(l), r*cos(b)*sin(l), r*sin(b));
    [x, cos(EPS_J2000)*y - sin(EPS_J2000)*z, sin(EPS_J2000)*y + cos(EPS_J2000)*z]
}

/// Geocentric Sun position [m], J2000, at a TT Julian date.
pub fn sun(jd_tt: f64) -> V3 {
    let t = (jd_tt - 2451545.0)/36525.0;
    let m = fmod(357.5256 + 35999.049*t, 360.0)*D2R;
    let l = (282.9400*D2R) + m + (6892.0*sin(m) + 72.0*sin(2.0*m))/3600.0*D2R;
    let r = (149.619 - 2.499*cos(m) - 0.021*cos(2.0*m))*1e9;
    ecl2eq(l, 0.0, r)
}

/// Geocentric Moon position [m], J2000, at a TT Julian date.
pub fn moon(jd_tt: f64) -> V3 {
    let t = (jd_tt - 2451545.0)/36525.0;
    let d = |x: f64| fmod(x, 360.0)*D2R;
    let l0 = d(218.31617 + 481267.88088*t - 1.3972*t);
    let l = d(134.96292 + 477198.86753*t);
    let lp = d(357.52543 + 35999.04944*t);
    let f = d(93.27283 + 483202.01873*t);
    let dd = d(297.85027 + 445267.11135*t);
    let lam = l0 + (22640.0*sin(l) + 769.0*sin(2.0*l) - 4586.0*sin(l - 2.0*dd) + 2370.0*sin(2.0*dd) - 668.0*sin(lp)
        - 412.0*sin(2.0*f) - 212.0*sin(2.0*l - 2.0*dd) - 206.0*sin(l + lp - 2.0*dd) + 192.0*sin(l + 2.0*dd)
        - 165.0*sin(lp - 2.0*dd) + 148.0*sin(l - lp) - 125.0*sin(dd) - 110.0*sin(l + lp) - 55.0*sin(2.0*f - 2.0*dd))/3600.0*D2R;
    let beta = (18520.0*sin(f + lam - l0 + (412.0*sin(2.0*f) + 541.0*sin(lp))/3600.0*D2R) - 526.0*sin(f - 2.0*dd)
        + 44.0*sin(l + f - 2.0*dd) - 31.0*sin(-l + f - 2.0*dd) - 25.0*sin(-2.0*l + f) - 23.0*sin(lp + f - 2.0*dd)
        + 21.0*sin(-l + f) + 11.0*sin(-lp + f - 2.0*dd))/3600.0*D2R;
    let r = (385000.0 - 20905.0*cos(l) - 3699.0*cos(2.0*dd - l) - 2956.0*cos(2.0*dd) - 570.0*cos(2.0*l)
        + 246.0*cos(2.0*l - 2.0*dd) - 205.0*cos(lp - 2.0*dd) - 171.0*cos(l + 2.0*dd) - 152.0*cos(l + lp - 2.0*dd))*1e3;
    ecl2eq(lam, beta, r)
}

/// Solar radiation pressure at distance d [m] from the Sun [N/m^2].
pub fn p_srp(d: f64) -> f64 { 4.56e-6*(AU/d)*(AU/d) }

/// Conical shadow fraction nu (1 sunlit, 0 umbra) (asils.env.shadow).
pub fn shadow(r_sat: &V3, r_sun: &V3) -> f64 {
    use crate::la::*;
    let rs = 6.957e8;
    let re = 6378137.0;
    let d = sub(r_sun, r_sat);
    let a = asin(rs/norm(&d));
    let b = asin(re/norm(r_sat));
    let c = acos(clamp(-dot(r_sat, &d)/(norm(r_sat)*norm(&d)), -1.0, 1.0));
    if c >= a + b { 1.0 }
    else if c < b - a { 0.0 }
    else {
        let x = (c*c + a*a - b*b)/(2.0*c);
        let y = sqrt(if a*a - x*x > 0.0 { a*a - x*x } else { 0.0 });
        let ar = a*a*acos(clamp(x/a, -1.0, 1.0)) + b*b*acos(clamp((c - x)/b, -1.0, 1.0)) - c*y;
        1.0 - ar/(PI*a*a)
    }
}
