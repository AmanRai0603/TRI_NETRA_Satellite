//! Truth geomagnetic field: env's methods, generated from the design (tools/engine_build.py): IGRF-13 by env's
//! synthesis (`gen::frames::igrf_gh`, `igrf_ned`: the flight software's model and table, one copy) at the engine's
//! geodetic coordinates, back to J2000 (`gen::truthfield`, the method of l3_dist_row_08). What is left here is the
//! engine's call of it, by the names the engine has always used.
use crate::gen::{frames, truthfield};
use crate::la::*;

/// The Gauss coefficients at a date, 195 [nT].
pub type Gh = [f64; 195];

/// Gauss coefficients at a decimal year [nT] (linear between epochs, last interval extrapolated).
pub fn gh(dy: f64) -> Gh { frames::igrf_gh(dy) }

/// WGS-84 geodetic latitude, longitude [rad], height [m].
pub fn geodetic(r: &V3) -> (f64, f64, f64) { truthfield::geodetic5(*r) }

/// Field north-east-down [nT] (Schmidt semi-normalised recursion, geodetic input).
pub fn ned(g: &Gh, lat: f64, lon: f64, alt_km: f64, nmax: usize) -> V3 { frames::igrf_ned(*g, lat, lon, alt_km, nmax as i64) }

/// Field in ECI [T] at an ECEF position, given the ECI->ECEF matrix.
pub fn eci(r_ecef: &V3, c_eci2ecef: &M3, g: &Gh, nmax: usize) -> V3 { truthfield::field_eci_ecef(*r_ecef, *c_eci2ecef, *g, nmax as i64) }

/// Field in ECI [T] at geodetic (lat, lon [rad], h [m]) -- asils.env.field with the geodetic coordinates supplied by
/// the caller (the engine passes POP's op.geodetic).
pub fn eci_at(lat: f64, lon: f64, h: f64, c_eci2ecef: &M3, g: &Gh, nmax: usize) -> V3 {
    truthfield::field_eci_at(lat, lon, h, *c_eci2ecef, *g, nmax as i64)
}
