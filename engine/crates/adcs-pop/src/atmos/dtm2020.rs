//! DTM2020 OPERATIONAL model (F10.7 + Kp): env's method env_dtm2020_operational (the port of
//! `density_models/dtm2020/operational/` -- `DTM2020_F107_coeffs_init.m`, `dtm3.m`, `gldtm.m`,
//! `dtm2020_oper_density.m` -- and the adapter `+atmos/dtm2020.m`) over env's DTM2020 coefficient tables
//! (env_dtm2020_coefficients, read from refgen's exports by tools/readers.py), generated from the design into
//! `gen::dtm2020` (tools/engine_build.py). This is the density of the SILS in-loop orbit. What is left here is the
//! crate's names: the coefficient set a call names, the Kp input, the wrapper's error.

/// The DTM2020 output records of the design: `dtm3`/`dtm5`'s raw output and `dtm2020_oper_density`/`dtm2020_density`'s.
pub use crate::gen::dtm2020::{DtmDensity, DtmRaw};

/// A DTM2020 coefficient set: the design's operational or research table (`gen::dtm2020coeffs`), as the MATLAB
/// `state` struct of `DTM2020_F107_coeffs_init` / `DTM2020_coeffs_init` names it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DtmCoeffs {
    /// the design's choice DtmSet (`gen::dtm2020::DTMSET_*`)
    pub set: i64,
}

/// Operational coefficients (`DTM2020_F107_coeffs_init`): the design's table.
pub fn oper_coeffs() -> &'static DtmCoeffs {
    static C: DtmCoeffs = DtmCoeffs { set: crate::gen::dtm2020::DTMSET_OPERATIONAL };
    &C
}

/// Particle masses [g] of H, He, O, N2, O2, N (`vma`).
pub fn vma() -> [f64; 6] {
    crate::gen::dtm2020::dtm_vma()
}

/// `dtm3`: the operational DTM2020 core. `f = [F10.7(t-24h), 0]`, `fbar = [F10.7 81-day
/// mean, 0]`, `akp` 4-element Kp array, `alti` [km] (> 120), `hl` local solar time
/// [rad], `alat`/`xlon` geographic latitude/longitude [rad]; the design's operational set.
pub fn dtm3(day: f64, f: [f64; 2], fbar: [f64; 2], akp: [f64; 4], alti: f64, hl: f64, alat: f64, xlon: f64, st: &DtmCoeffs) -> DtmRaw {
    assert_eq!(st.set, crate::gen::dtm2020::DTMSET_OPERATIONAL, "dtm3 takes the operational coefficients");
    crate::gen::dtm2020::dtm3(day, f[0], fbar[0], akp, alti, hl, alat, xlon)
}

/// Kp input of the operational model: a scalar (used for both the 3 h-delayed and the
/// 24 h-mean slots) or the full 4-element `akp` array `[Kp_3h; 0; Kp_24h; 0]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum KpIn {
    /// one Kp for both slots
    Scalar(f64),
    /// `[Kp_3h; 0; Kp_24h; 0]`
    Akp([f64; 4]),
}

impl KpIn {
    /// The 4-element `akp` array the model takes (`[Kp; 0; Kp; 0]` for a scalar).
    pub fn akp(&self) -> [f64; 4] {
        match *self {
            KpIn::Scalar(k) => [k, 0.0, k, 0.0],
            KpIn::Akp(a) => a,
        }
    }
}

/// Error of the DTM wrappers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DtmError {
    /// `alt_km <= 120` (the model's lower boundary).
    AltitudeTooLow(f64),
}

/// `dtm2020_oper_density(alt_km, lat_deg, lon_deg, lst_hours, doy, F107, F107_bar, Kp, state)`.
pub fn oper_density(alt_km: f64, lat_deg: f64, lon_deg: f64, lst_hours: f64, doy: f64, f107: f64, f107_bar: f64, kp: KpIn, st: &DtmCoeffs) -> Result<DtmDensity, DtmError> {
    assert_eq!(st.set, crate::gen::dtm2020::DTMSET_OPERATIONAL, "dtm2020_oper_density takes the operational coefficients");
    match crate::gen::dtm2020::oper_density(alt_km, lat_deg, lon_deg, lst_hours, doy, f107, f107_bar, kp.akp()) {
        (true, o) => Ok(o),
        (false, _) => Err(DtmError::AltitudeTooLow(alt_km)),
    }
}
