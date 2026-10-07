//! Jacchia-Bowman 2008: env's method env_jb2008 (the port of `density_models/jb2008/JB2008.m` -- the Mahooti core,
//! CIRA integration form -- `jb2008_density.m` (lagged SET indices, Sun position, GMST), `finddays.m`, `invjday.m`,
//! `sign_.m`, and the adapter `+atmos/jb2008.m`) over env's SET index tables (env_jb2008_indices: SOLFSMY.TXT and
//! DTCFILE.TXT, read into the design by tools/readers.py, the parsers of `get_jb2008_indices`), generated from the
//! design into `gen::jb2008` (tools/engine_build.py).
//!
//! Times are MATLAB `datenum` days (`datenum(datetime)` in the MATLAB wrapper).

/// What the JB2008 core gives (`TEMP` = [Tinf, T(z)] [K], `RHO` [kg/m^3]) and what the wrapper hands it (MJD, Sun RA
/// and Dec, the point's RA, latitude and height, F10 F10B S10 S10B M10 M10B Y10 Y10B DSTDTC): the design's records.
pub use crate::gen::jb2008::{JbCoreInputs, JbOut};

/// The SET index tables (`idx` of `get_jb2008_indices`) as the design holds them (env_jb2008_indices): daily
/// `SOLFSMY.TXT` (F10 F81 S10 S81 M10 M81 Y10 Y81) and hourly `DTCFILE.TXT` (DSTDTC [K]), with their epochs
/// [datenum]. The model reads the design's tables itself; this is their view, for a caller that wants the indices.
#[derive(Clone, Debug, Default)]
pub struct JbIndices {
    /// daily epochs [datenum] of the solar indices (sorted)
    pub sol_t: Vec<f64>,
    /// columns F10 F81 S10 S81 M10 M81 Y10 Y81, one row per `sol_t`
    pub sol: Vec<[f64; 8]>,
    /// hourly epochs [datenum] of DSTDTC (sorted)
    pub dtc_t: Vec<f64>,
    /// DSTDTC [K]
    pub dtc: Vec<f64>,
}

impl JbIndices {
    /// The design's SET tables (release 8_1_0, 1997-001 to 2026-137, the copies bundled with the MATLAB toolbox,
    /// which `data.jb2008_indices` falls back to offline).
    pub fn bundled() -> JbIndices {
        use crate::gen::jbset::{DATA_DTCFILE, DATA_SOLFSMY};
        let n = DATA_SOLFSMY.len();
        let sol_t = (0..n).map(|i| crate::gen::jb2008::sol_day(i as i64)).collect();
        let sol = DATA_SOLFSMY.iter().map(|r| [r[2], r[3], r[4], r[5], r[6], r[7], r[8], r[9]]).collect();
        let m = DATA_DTCFILE.len() * 24;
        let dtc_t = (0..m).map(|k| crate::gen::jb2008::dtc_hour(k as i64)).collect();
        let dtc = (0..m).map(|k| DATA_DTCFILE[k / 24][k % 24 + 2]).collect();
        JbIndices { sol_t, sol, dtc_t, dtc }
    }
}

/// `sign_(a, b)`: `abs(a)` with the sign of `b` (`b >= 0` -> positive).
pub use crate::gen::jb2008::jb_sign as sign_;

/// `invjday(Mjd)` (Montenbruck & Gill): calendar date from a modified Julian date.
pub fn invjday(mjd: f64) -> (f64, f64, f64, f64, f64, f64) {
    let u = crate::gen::jb2008::invjday(mjd);
    (u[0], u[1], u[2], u[3], u[4], u[5])
}

/// `finddays(year, month, day, hr, min, sec)` (Vallado): fractional day of year.
pub use crate::gen::jb2008::finddays;

/// `JB2008(MJD, SUN, SAT, F10, F10B, S10, S10B, XM10, XM10B, Y10, Y10B, DSTDTC)`.
/// `sun` = [RA, Dec] [rad]; `sat` = [RA, geocentric latitude [rad], height [km]].
pub use crate::gen::jb2008::jb2008_core;

/// `sun_radec` of `jb2008_density.m`: low-precision Sun RA/Dec (mean equinox of date).
/// `gmst_rad` of `jb2008_density.m`: IAU-82 GMST [rad] from an MJD.
pub use crate::gen::jb2008::{gmst_rad, sun_radec};

/// `jb2008_density(t, lon_deg, lat_deg, alt_km, idx)` for ONE epoch (the in-RHS case:
/// no decimation). `utc` = [Y M D h m s]. Lags per the JB2008 spec: F10/S10 at t-1 d,
/// M10 at t-2 d, Y10 at t-5 d, DSTDTC at the epoch; linear with extrapolation, over the design's SET tables (`idx`
/// is their view).
pub fn jb2008_density(utc: &[f64; 6], lon_deg: f64, lat_deg: f64, alt_km: f64, _idx: &JbIndices) -> (f64, JbOut, JbCoreInputs) {
    crate::gen::jb2008::jb2008_density(*utc, lon_deg, lat_deg, alt_km)
}
