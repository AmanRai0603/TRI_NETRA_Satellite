//! DTM2020 RESEARCH model (F30 + ap60): env's method env_dtm2020_research (the port of
//! `density_models/dtm2020/research/` -- `DTM2020_coeffs_init.m`, `dtm5.m`, `gldtm_Hp.m`, `bint_oe.m`, `geogm.m`,
//! `dtm2020_density.m` -- the adapter `+atmos/dtm2020_research.m` and
//! `data_sources/spaceweather/solar/f30_to_f107scale.m` / `f30_from_f107.m`) over env's research coefficients and
//! DTM2020's open-ended ap-Kp table, generated from the design into `gen::dtm2020res` (tools/engine_build.py).
//!
//! Despite the `_Hp` in `gldtm_Hp`, this model is driven by the hourly **ap60** index
//! (converted to an open-ended Kp by `bint_oe`), not by Hp60.
use super::dtm2020::{DtmCoeffs, DtmDensity, DtmError, DtmRaw};

/// Research coefficients (`DTM2020_coeffs_init`): the design's table.
pub fn research_coeffs() -> &'static DtmCoeffs {
    static C: DtmCoeffs = DtmCoeffs { set: crate::gen::dtm2020::DTMSET_RESEARCH };
    &C
}

/// `bint_oe`: ap60 -> open-ended Kp by piecewise-linear interpolation over the extended
/// ap-Kp table (extrapolated below 0 and above ap = 657, where MATLAB also warns).
pub use crate::gen::dtm2020res::bint_oe;

/// `geogm`: geographic -> geomagnetic latitude/longitude [deg] (dipole pole at 78.5 N,
/// 291 E). Returns `None` where MATLAB raises `GEOGM: angle error` (NaN quadrant).
pub fn geogm(xlat: f64, xlong: f64) -> Option<(f64, f64)> {
    match crate::gen::dtm2020res::geogm(xlat, xlong) {
        (a, b, true) => Some((a, b)),
        _ => None,
    }
}

/// `dtm5`: the research DTM2020 core. `f`/`fbar` = [F30 on the F10.7 scale, 0],
/// `ap60` the 10-slot ap60 array, other arguments as [`super::dtm2020::dtm3`].
/// `None` only where `geogm` would raise its angle error.
pub fn dtm5(day: f64, f: [f64; 2], fbar: [f64; 2], ap60: &[f64; 10], alti: f64, hl: f64, alat: f64, xlon: f64, st: &DtmCoeffs) -> Option<DtmRaw> {
    assert_eq!(st.set, crate::gen::dtm2020::DTMSET_RESEARCH, "dtm5 takes the research coefficients");
    match crate::gen::dtm2020res::dtm5(day, f[0], fbar[0], *ap60, alti, hl, alat, xlon) {
        (true, o) => Some(o),
        _ => None,
    }
}

/// ap60 input of `dtm2020_density`: a scalar (all 10 slots) or the full array.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Ap60In {
    /// one ap60 in all 10 slots
    Scalar(f64),
    /// the 10-slot ap60 array
    Array([f64; 10]),
}

/// `dtm2020_density(alt_km, lat_deg, lon_deg, lst_hours, doy, F30, F30_bar, ap60, state)`.
/// `F30`/`F30_bar` must already be on the F10.7 scale ([`f30_to_f107scale`]).
pub fn density(alt_km: f64, lat_deg: f64, lon_deg: f64, lst_hours: f64, doy: f64, f30: f64, f30_bar: f64, ap60: Ap60In, st: &DtmCoeffs) -> Result<DtmDensity, DtmError> {
    assert_eq!(st.set, crate::gen::dtm2020::DTMSET_RESEARCH, "dtm2020_density takes the research coefficients");
    let ap = match ap60 {
        Ap60In::Scalar(a) => [a; 10],
        Ap60In::Array(v) => v,
    };
    match crate::gen::dtm2020res::research_density(alt_km, lat_deg, lon_deg, lst_hours, doy, f30, f30_bar, ap) {
        (true, o) => Ok(o),
        _ if alt_km <= 120.0 => Err(DtmError::AltitudeTooLow(alt_km)),
        _ => Err(DtmError::AltitudeTooLow(f64::NAN)),
    }
}

/// `f30_to_f107scale`: native F30 -> F10.7 scale, DTM2020 paper eq. 2 (drift term in the
/// decimal year). `f30_from_f107`: pseudo-F30 derived from F10.7 (the last-resort source of `get_f30`).
pub use crate::gen::dtm2020res::{f30_from_f107, f30_to_f107scale};

/// `decimalYear` of `+atmos/dtm2020_research.m` (leap-year aware, via `datenum`).
pub fn decimal_year(utc: &[f64; 6]) -> f64 {
    crate::gen::dtm2020res::dtm_decimal_year(*utc)
}
