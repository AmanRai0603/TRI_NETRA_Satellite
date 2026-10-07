//! Time scales: env's method env_time_scales (the port of the Octave POP's `+timeconv` -- cal2jd, jd2cal, jd2mjd,
//! mjd2jd, leapTable, taiMinusUTC, utc2tai, tai2utc, tai2tt, tt2tai, tt2utc, utc2tt, tt2tdb, utc2tdb, utc2ut1, tai2gps,
//! utc2gps, tt2jc, j2000sec, gpsWeekSow, gmst82, doy, convertUTC -- and of `01_core/+op/addsec.m` with the Octave
//! `datenum`/`datevec` it relies on), over env's leap-second table, generated from the design into `gen::timescales`
//! (tools/engine_build.py). Every function keeps the MATLAB formula and order of operations, so results agree with
//! Octave to the last bit. What is left here is the crate's names for them.

pub use crate::gen::timescales::{
    addsec, cal2jd, convert_utc, datenum, datevec, deg2rad, doy, gmst82, gps_week_sow, j2000sec, jd2cal, jd2mjd, mjd2jd, omod,
    tai2gps, tai2tt, tai2utc, tai_minus_utc, tt2jc, tt2tai, tt2tdb, tt2utc, utc2gps, utc2tai, utc2tdb, utc2tt, utc2ut1, Times,
};

/// `timeconv.leapTable()`: `[year month TAI-UTC(s)]` at 0h UTC of the 1st of that month (env's table, IERS Bulletin C).
pub use crate::gen::leapsec::DATA_LEAP_SECONDS as LEAP_TABLE;
