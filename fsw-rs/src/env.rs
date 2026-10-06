//! Onboard time, frames and environment models (fsw/pseudocode/02), twin of adcs_env.c.
use crate::alg::frames as alg;
use crate::math::*;

/// ECI -> ECEF rotation about z by GMST.
/// Written from the design: frames::gmst_rot (src/alg).
pub fn gmst_rot(jd: f64) -> M3 { alg::gmst_rot(jd) }
/// J2000 -> mean of date (IAU-76 precession): P = R3(-z) R2(theta) R3(-zeta).
/// Written from the design: frames::prec_rot (src/alg).
pub fn prec_rot(jd: f64) -> M3 { alg::prec_rot(jd) }
/// J2000 -> ECEF: precession to the mean equator of date, then GMST (nutation and polar motion omitted).
/// Written from the design: frames::eci2ecef (src/alg).
pub fn eci2ecef(jd: f64) -> M3 { alg::eci2ecef(jd) }
/// Written from the design: frames::decyear (src/alg).
pub fn decyear(jd: f64) -> f64 { alg::decyear(jd) }

/// Unit Sun direction, J2000 (mean-of-date model precessed to J2000).
/// Written from the design: frames::sun_model (src/alg).
pub fn sun_model(jd: f64) -> V3 { alg::sun_model(jd) }

/// WGS-84 geodetic latitude, longitude [rad] and height [m] of an ECEF position.
/// Written from the design: frames::geodetic (src/alg).
pub fn geodetic(r: &V3) -> (f64, f64, f64) { alg::geodetic(*r) }

/// Gauss coefficients interpolated to a decimal year [nT] (linear, last interval extrapolated).
/// Written from the design: frames::igrf_gh (src/alg).
pub fn igrf_gh(dy: f64) -> [f64; 195] { alg::igrf_gh(dy) }

/// Field in north-east-down [nT] at geodetic lat, lon [rad], altitude [km].
/// Written from the design: frames::igrf_ned (src/alg).
pub fn igrf_ned(gh: &[f64; 195], lat: f64, lon: f64, alt_km: f64, nmax: i32) -> V3 { alg::igrf_ned(*gh, lat, lon, alt_km, nmax as i64) }

/// Field in ECI [T] at r_eci [m].
/// Written from the design: frames::field_eci (src/alg).
pub fn field_eci(r_eci: &V3, jd: f64, gh: &[f64; 195], nmax: i32) -> V3 { alg::field_eci(*r_eci, jd, *gh, nmax as i64) }
