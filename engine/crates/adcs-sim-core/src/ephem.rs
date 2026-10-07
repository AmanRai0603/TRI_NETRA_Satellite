//! The fast orbit's Sun and Moon, the Sun's pressure and the Earth's shadow: env's methods, generated from the design
//! (tools/engine_build.py): the low-precision Sun and its pressure (`gen::sunfast`, l3_dist_row_09), the Moon
//! (`gen::moonfast`, env_moon_fast), both Montenbruck and Gill (2000) section 3.3.2, mean equator and equinox of J2000,
//! metres, and the conical shadow (`gen::shadow`, l3_dist_row_10). What is left here is the engine's call of them, by
//! the names the engine has always used. The MATLAB twin reads DE440 through the POP; the difference (Sun ~0.01 deg,
//! Moon ~0.1 deg) is a parity-ledger line.
use crate::gen::{moonfast, shadow as sh, sunfast};
use crate::la::V3;

pub const AU: f64 = crate::gen::constants::AU_M;

/// Geocentric Sun position [m], J2000, at a TT Julian date.
pub fn sun(jd_tt: f64) -> V3 { sunfast::sun_position_fast(jd_tt) }

/// Geocentric Moon position [m], J2000, at a TT Julian date.
pub fn moon(jd_tt: f64) -> V3 { moonfast::moon_position_fast(jd_tt) }

/// Solar radiation pressure at distance d [m] from the Sun [N/m^2].
pub fn p_srp(d: f64) -> f64 { sunfast::solar_pressure_at(d) }

/// Conical shadow fraction nu (1 sunlit, 0 umbra) (asils.env.shadow).
pub fn shadow(r_sat: &V3, r_sun: &V3) -> f64 { sh::shadow_fraction(*r_sat, *r_sun) }
