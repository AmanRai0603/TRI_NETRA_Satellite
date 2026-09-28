//! Atmosphere density models (matlab_sils/pop/04_atmosphere): the model switch
//! `+atmos/provider.m` and its adapters `+atmos/dtm2020.m` (OPERATIONAL, F10.7 + Kp --
//! the SILS default), `+atmos/dtm2020_research.m` (F30 + ap60), `+atmos/jb2008.m`,
//! `+atmos/exponential.m`. `+atmos/nrlmsise.m` is only an adapter to the MATLAB
//! Aerospace Toolbox `atmosnrlmsise00`, which is not part of the repository, so it is
//! not ported ([`AtmosError::NotPorted`]).
//!
//! Every function is allocation-free (the DTM coefficient tables are parsed once).
#![deny(missing_docs)]
#![allow(rustdoc::broken_intra_doc_links)] // unit brackets like [K], [m/s] in the docs
pub mod octave;
pub mod dtm2020;
pub mod dtm2020_research;
pub mod exponential;
pub mod jb2008;

use crate::spaceweather::{ResearchSw, SpaceWeather};
use dtm2020::{DtmError, KpIn};
use jb2008::JbIndices;

/// Avogadro constant [1/mol] (`de440.constants().N_A`).
pub const N_A: f64 = 6.02214076e23;

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

/// `atmos.dtm2020(geo, sw)`: operational DTM2020 with the species converted to m^-3.
pub fn dtm2020_atm(geo: &Geo, sw: &SpaceWeather) -> Result<AtmosOut, AtmosError> {
    if !sw.f107.is_finite() {
        return Err(AtmosError::MissingDriver("F107"));
    }
    if !sw.f107a.is_finite() {
        return Err(AtmosError::MissingDriver("F107a"));
    }
    let kp_ok = match sw.kp {
        KpIn::Scalar(k) => k.is_finite(),
        KpIn::Akp(a) => a.iter().all(|x| x.is_finite()),
    };
    if !kp_ok {
        return Err(AtmosError::MissingDriver("Kp"));
    }
    let o = dtm2020::oper_density(geo.alt_km, geo.lat_deg, geo.lon_deg, geo.lst_h, geo.doy, sw.f107, sw.f107a, sw.kp, dtm2020::oper_coeffs())?;
    let mut n = [0.0f64; 6];
    for i in 0..6 {
        n[i] = o.n_cm3[i] * 1e6;
    }
    Ok(AtmosOut { rho: o.rho_kgm3, t: o.t_k, comp: Composition::Species(n) })
}

/// `atmos.dtm2020_research(geo, sw)`: F30 rescaled to the F10.7 scale with the
/// decimal-year drift (unless `f30_is_derived`), ap60 in all slots.
pub fn dtm2020_research_atm(geo: &Geo, sw: &ResearchSw) -> Result<AtmosOut, AtmosError> {
    for (v, nm) in [(sw.f30, "F30"), (sw.f30_bar, "F30_bar"), (sw.ap60, "ap60")] {
        if !v.is_finite() {
            return Err(AtmosError::MissingDriver(nm));
        }
    }
    let (fi, fbi) = if sw.f30_is_derived {
        (sw.f30, sw.f30_bar)
    } else {
        let dy = dtm2020_research::decimal_year(&geo.utc);
        (dtm2020_research::f30_to_f107scale(sw.f30, dy), dtm2020_research::f30_to_f107scale(sw.f30_bar, dy))
    };
    let o = dtm2020_research::density(
        geo.alt_km,
        geo.lat_deg,
        geo.lon_deg,
        geo.lst_h,
        geo.doy,
        fi,
        fbi,
        dtm2020_research::Ap60In::Scalar(sw.ap60),
        dtm2020_research::research_coeffs(),
    )?;
    let n = o.n_cm3;
    // NB atmos.dtm2020_research hands dtm2020_density's out.n on UNCONVERTED
    // (cm^-3), unlike atmos.dtm2020 which converts to m^-3 -- ported as is.
    Ok(AtmosOut { rho: o.rho_kgm3, t: o.t_k, comp: Composition::Species(n) })
}

/// `atmos.jb2008(geo, sw)`: JB2008 total density; `Mmol = 16`, `nO = rho*N_A*1000/16`,
/// `T = sw.Tinf` or the 1000 K placeholder.
pub fn jb2008_atm(geo: &Geo, idx: &JbIndices, tinf: Option<f64>) -> Result<AtmosOut, AtmosError> {
    if idx.sol_t.len() < 2 || idx.dtc_t.len() < 2 {
        return Err(AtmosError::MissingDriver("jb_idx"));
    }
    let (rho, _, _) = jb2008::jb2008_density(&geo.utc, geo.lon_deg, geo.lat_deg, geo.alt_km, idx);
    if !rho.is_finite() {
        return Err(AtmosError::NonFinite);
    }
    let avog16 = N_A * 1000.0 / 16.0;
    Ok(AtmosOut { rho, t: tinf.unwrap_or(1000.0), comp: Composition::Mean { mmol: 16.0, n_o: rho * avog16 } })
}

/// `atmos.exponential(alt_km)` as an [`AtmosOut`].
pub fn exponential_atm(alt_km: f64) -> AtmosOut {
    let e = exponential::exponential(alt_km);
    AtmosOut { rho: e.rho, t: e.t, comp: Composition::Mean { mmol: e.mmol, n_o: e.n_o } }
}

/// `atmos.provider(model, geo, sw)`: the density-model switch. No silent fallback: a
/// model whose drivers are missing returns an error.
pub fn density(model: AtmosModel, geo: &Geo, drv: &AtmosDrivers) -> Result<AtmosOut, AtmosError> {
    match model {
        AtmosModel::Exponential => Ok(exponential_atm(geo.alt_km)),
        AtmosModel::Nrlmsise => Err(AtmosError::NotPorted("nrlmsise: MATLAB Aerospace Toolbox atmosnrlmsise00 is not in the repository")),
        AtmosModel::Dtm2020 => match drv {
            AtmosDrivers::Sw(sw) => dtm2020_atm(geo, sw),
            _ => Err(AtmosError::MissingDriver("F107")),
        },
        AtmosModel::Dtm2020Research => match drv {
            AtmosDrivers::Research(sw) => dtm2020_research_atm(geo, sw),
            _ => Err(AtmosError::MissingDriver("F30")),
        },
        AtmosModel::Jb2008 => match drv {
            AtmosDrivers::Jb(idx, tinf) => jb2008_atm(geo, idx, *tinf),
            _ => Err(AtmosError::MissingDriver("jb_idx")),
        },
    }
}
