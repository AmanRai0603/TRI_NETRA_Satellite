//! Atmosphere density models (matlab_sils/pop/04_atmosphere): the model switch
//! `+atmos/provider.m` and its adapters `+atmos/dtm2020.m` (OPERATIONAL, F10.7 + Kp --
//! the SILS default), `+atmos/dtm2020_research.m` (F30 + ap60), `+atmos/jb2008.m`,
//! `+atmos/exponential.m`. `+atmos/nrlmsise.m` is only an adapter to the MATLAB
//! Aerospace Toolbox `atmosnrlmsise00`, which is not part of the repository, so it is
//! not ported ([`AtmosError::NotPorted`]).
//!
//! The switch (a choice) and its adapters are env's method env_density_model, the models env's methods over env's
//! tables, generated from the design into `gen::densitymodel`, `gen::dtm2020`, `gen::dtm2020res`, `gen::jb2008` and
//! `gen::expatmos` (tools/engine_build.py). What is left here is the crate's types (the drivers as references, the
//! composition as an enum, the errors with their names) and the calls.
#![deny(missing_docs)]
#![allow(rustdoc::broken_intra_doc_links)] // unit brackets like [K], [m/s] in the docs
pub mod octave;
pub mod dtm2020;
pub mod dtm2020_research;
pub mod exponential;
pub mod jb2008;

use crate::gen::densitymodel as dm;
use crate::spaceweather::{ResearchSw, SpaceWeather};
use dtm2020::DtmError;
use jb2008::JbIndices;

/// Avogadro constant [1/mol] (`de440.constants().N_A`): the design's.
pub use exponential::N_A;

/// Density model names of `atmos.provider` / `cfg.forces.drag.atmos`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AtmosModel {
    /// 'exponential' -- altitude only
    Exponential,
    /// 'nrlmsise' -- NOT PORTED (Aerospace Toolbox)
    Nrlmsise,
    /// 'jb2008'
    Jb2008,
    /// 'dtm2020' -- the operational F10.7/Kp model (dtm3)
    Dtm2020,
    /// 'dtm2020_research' / 'dtm2020_res' -- F30/ap60 (dtm5)
    Dtm2020Research,
}

impl AtmosModel {
    /// Parse a MATLAB model name (case-insensitive).
    pub fn from_name(s: &str) -> Option<AtmosModel> {
        match s.to_ascii_lowercase().as_str() {
            "exponential" => Some(AtmosModel::Exponential),
            "nrlmsise" => Some(AtmosModel::Nrlmsise),
            "jb2008" => Some(AtmosModel::Jb2008),
            "dtm2020" => Some(AtmosModel::Dtm2020),
            "dtm2020_research" | "dtm2020_res" => Some(AtmosModel::Dtm2020Research),
            _ => None,
        }
    }
}

/// The geodetic probe point `geo` of forces.drag / atmos.provider.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Geo {
    /// geodetic altitude [km]
    pub alt_km: f64,
    /// geodetic latitude [deg]
    pub lat_deg: f64,
    /// longitude [deg]
    pub lon_deg: f64,
    /// local solar time [h] = mod(UT_h + lon/15, 24)
    pub lst_h: f64,
    /// day of year (`ctx.T.doy`)
    pub doy: f64,
    /// UTC [Y M D h m s]
    pub utc: [f64; 6],
}

/// Species order of the DTM models and of `atm.n` built from them.
pub const SPECIES: [&str; 6] = ["H", "He", "O", "N2", "O2", "N"];

/// Composition carried in the `atm` struct handed to drag.force.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Composition {
    /// `atm.n.{H,He,O,N2,O2,N}` number densities [m^-3] (DTM2020 variants), in
    /// [`SPECIES`] order
    Species([f64; 6]),
    /// single mean species
    Mean {
        /// `atm.Mmol` [kg/kmol]
        mmol: f64,
        /// `atm.nO` [m^-3]
        n_o: f64,
    },
}

/// The `atm` struct of `atmos.provider`: total density, temperature, composition.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AtmosOut {
    /// total mass density [kg/m^3] (`atm.rho`)
    pub rho: f64,
    /// temperature [K] (`atm.T`; 1000 K placeholder for JB2008, as MATLAB)
    pub t: f64,
    /// composition (`atm.n` or `atm.Mmol`/`atm.nO`)
    pub comp: Composition,
}

impl AtmosOut {
    /// Atomic-oxygen number density [m^-3] as drag.force reads it (`atm.n.O` / `atm.nO`).
    pub fn n_o(&self) -> f64 {
        match self.comp {
            Composition::Species(n) => n[2],
            Composition::Mean { n_o, .. } => n_o,
        }
    }
}

/// The driver set handed to [`density`] (the `sw` struct of atmos.provider).
#[derive(Clone, Copy, Debug)]
pub enum AtmosDrivers<'a> {
    /// no drivers (exponential)
    None,
    /// `atmos.spaceweather` output (dtm2020, nrlmsise)
    Sw(&'a SpaceWeather),
    /// `atmos.research_drivers` output (dtm2020_research)
    Research(&'a ResearchSw),
    /// SET index tables (jb2008) + optional `sw.Tinf` placeholder temperature
    Jb(&'a JbIndices, Option<f64>),
}

/// Errors of the atmosphere layer (MATLAB `error(...)` identifiers).
#[derive(Clone, Debug, PartialEq)]
pub enum AtmosError {
    /// the model needs a driver that was not supplied / is non-finite
    /// (`atmos:dtm2020:index`, `atmos:dtm2020_research:driver`, `atmos:jb2008:noIndices`)
    MissingDriver(&'static str),
    /// DTM wrappers: altitude must be > 120 km
    AltitudeTooLow(f64),
    /// `atmos:jb2008:nonfinite`
    NonFinite,
    /// model not available in the port
    NotPorted(&'static str),
}

impl From<DtmError> for AtmosError {
    fn from(e: DtmError) -> Self {
        match e {
            DtmError::AltitudeTooLow(a) => AtmosError::AltitudeTooLow(a),
        }
    }
}

impl AtmosModel {
    /// The design's choice AtmosModel (`gen::densitymodel::ATMOSMODEL_*`).
    pub fn choice(self) -> i64 {
        match self {
            AtmosModel::Exponential => dm::ATMOSMODEL_EXPONENTIAL,
            AtmosModel::Nrlmsise => dm::ATMOSMODEL_NRLMSISE,
            AtmosModel::Jb2008 => dm::ATMOSMODEL_JB2008,
            AtmosModel::Dtm2020 => dm::ATMOSMODEL_DTM2020,
            AtmosModel::Dtm2020Research => dm::ATMOSMODEL_DTM2020_RESEARCH,
        }
    }
}

/// The switch's answer in the crate's types: the atm, or the error the design's status names.
fn answer(st: i64, a: dm::AtmosOut, alt_km: f64) -> Result<AtmosOut, AtmosError> {
    let e = match st {
        dm::ATMOSSTATUS_OK => {
            let comp = if a.species { Composition::Species(a.n) } else { Composition::Mean { mmol: a.mmol, n_o: a.n_o } };
            return Ok(AtmosOut { rho: a.rho, t: a.t, comp });
        }
        dm::ATMOSSTATUS_MISSING_F107 => AtmosError::MissingDriver("F107"),
        dm::ATMOSSTATUS_MISSING_F107A => AtmosError::MissingDriver("F107a"),
        dm::ATMOSSTATUS_MISSING_KP => AtmosError::MissingDriver("Kp"),
        dm::ATMOSSTATUS_MISSING_F30 => AtmosError::MissingDriver("F30"),
        dm::ATMOSSTATUS_MISSING_F30_BAR => AtmosError::MissingDriver("F30_bar"),
        dm::ATMOSSTATUS_MISSING_AP60 => AtmosError::MissingDriver("ap60"),
        dm::ATMOSSTATUS_MISSING_JB => AtmosError::MissingDriver("jb_idx"),
        dm::ATMOSSTATUS_ALTITUDE => AtmosError::AltitudeTooLow(alt_km),
        dm::ATMOSSTATUS_GEOGM => AtmosError::AltitudeTooLow(f64::NAN),
        dm::ATMOSSTATUS_NONFINITE => AtmosError::NonFinite,
        _ => AtmosError::NotPorted("nrlmsise: MATLAB Aerospace Toolbox atmosnrlmsise00 is not in the repository"),
    };
    Err(e)
}

/// `atmos.dtm2020(geo, sw)`: operational DTM2020 with the species converted to m^-3.
pub fn dtm2020_atm(geo: &Geo, sw: &SpaceWeather) -> Result<AtmosOut, AtmosError> {
    let (st, a) = dm::dtm2020_atm(geo.alt_km, geo.lat_deg, geo.lon_deg, geo.lst_h, geo.doy, sw.f107, sw.f107a, sw.kp.akp());
    answer(st, a, geo.alt_km)
}

/// `atmos.dtm2020_research(geo, sw)`: F30 rescaled to the F10.7 scale with the
/// decimal-year drift (unless `f30_is_derived`), ap60 in all slots.
/// NB atmos.dtm2020_research hands dtm2020_density's out.n on UNCONVERTED
/// (cm^-3), unlike atmos.dtm2020 which converts to m^-3 -- ported as is.
pub fn dtm2020_research_atm(geo: &Geo, sw: &ResearchSw) -> Result<AtmosOut, AtmosError> {
    let (st, a) = dm::dtm2020_research_atm(geo.alt_km, geo.lat_deg, geo.lon_deg, geo.lst_h, geo.doy, geo.utc, sw.f30, sw.f30_bar, sw.ap60, sw.f30_is_derived);
    answer(st, a, geo.alt_km)
}

/// `atmos.jb2008(geo, sw)`: JB2008 total density; `Mmol = 16`, `nO = rho*N_A*1000/16`,
/// `T = sw.Tinf` or the 1000 K placeholder.
pub fn jb2008_atm(geo: &Geo, idx: &JbIndices, tinf: Option<f64>) -> Result<AtmosOut, AtmosError> {
    if idx.sol_t.len() < 2 || idx.dtc_t.len() < 2 {
        return Err(AtmosError::MissingDriver("jb_idx"));
    }
    let (st, a) = dm::jb2008_atm(geo.alt_km, geo.lat_deg, geo.lon_deg, geo.utc, tinf.is_some(), tinf.unwrap_or(0.0));
    answer(st, a, geo.alt_km)
}

/// `atmos.exponential(alt_km)` as an [`AtmosOut`].
pub fn exponential_atm(alt_km: f64) -> AtmosOut {
    let e = exponential::exponential(alt_km);
    AtmosOut { rho: e.rho, t: e.t, comp: Composition::Mean { mmol: e.mmol, n_o: e.n_o } }
}

/// `atmos.provider(model, geo, sw)`: the density-model switch (the design's). No silent
/// fallback: a model whose drivers are missing returns an error.
pub fn density(model: AtmosModel, geo: &Geo, drv: &AtmosDrivers) -> Result<AtmosOut, AtmosError> {
    let nan = f64::NAN;
    let (kind, f107, f107a, akp, f30, f30_bar, ap60, derived, tinf) = match drv {
        AtmosDrivers::None => (dm::DRIVERS_NONE, nan, nan, [nan; 4], nan, nan, nan, false, None),
        AtmosDrivers::Sw(sw) => (dm::DRIVERS_SPACEWEATHER, sw.f107, sw.f107a, sw.kp.akp(), nan, nan, nan, false, None),
        AtmosDrivers::Research(sw) => (dm::DRIVERS_RESEARCH, nan, nan, [nan; 4], sw.f30, sw.f30_bar, sw.ap60, sw.f30_is_derived, None),
        AtmosDrivers::Jb(idx, tinf) => {
            if model == AtmosModel::Jb2008 && (idx.sol_t.len() < 2 || idx.dtc_t.len() < 2) {
                return Err(AtmosError::MissingDriver("jb_idx"));
            }
            (dm::DRIVERS_JB2008, nan, nan, [nan; 4], nan, nan, nan, false, *tinf)
        }
    };
    let (st, a) = dm::atmos_density(model.choice(), geo.alt_km, geo.lat_deg, geo.lon_deg, geo.lst_h, geo.doy, geo.utc, kind, f107, f107a, akp,
                                    f30, f30_bar, ap60, derived, tinf.is_some(), tinf.unwrap_or(0.0));
    answer(st, a, geo.alt_km)
}
