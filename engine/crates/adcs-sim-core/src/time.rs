//! Calendar and sidereal time: env's method env_calendar_time (Vallado 2013 algorithms 14 and 22, GMST IAU 1982,
//! IAU 1976 precession), generated from the design into `gen::caltime` (tools/engine_build.py). What is left here is
//! the engine's call of it, by the names the engine has always used.
use crate::gen::caltime as g;
use crate::la::M3;

/// Julian date of a UTC calendar date [Y M D h m s] (valid 1901-2099).
pub fn jd(u: &[f64; 6]) -> f64 { g::jd_utc(*u) }
/// The UTC calendar date of a Julian date.
pub fn jd2utc(jd: f64) -> [f64; 6] { g::utc_of_jd(jd) }
/// The decimal year of a Julian date.
pub fn decyear(jd_: f64) -> f64 { g::decimal_year(jd_) }
/// GMST [rad] (IAU-82, UT1 = UTC).
pub fn gmst(jd_ut1: f64) -> f64 { g::gmst_iau82(jd_ut1) }
/// J2000 -> mean of date (IAU-76 precession).
pub fn precession(jd: f64) -> M3 { g::precession_iau76(jd) }
/// J2000 -> ECEF: IAU-76 precession to the mean equator of date, then GMST (nutation, polar motion omitted).
pub fn eci2ecef(jd_ut1: f64) -> M3 { g::eci_to_ecef(jd_ut1) }
/// TT - UTC [s] for the 2017-2035 leap-second epoch (TAI - UTC = 37 s).
pub const TT_MINUS_UTC_S: f64 = g::TT_MINUS_UTC_S;
