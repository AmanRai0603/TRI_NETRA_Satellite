//! Aerodynamic drag: `02_forces/+forces/drag.m` (the force term op.accel calls) and
//! `02_forces/+drag/*` (cannonball, force = multi-species free-molecular panel model,
//! relVelocity, windAngles, bodyToWind; the GSI coefficients in [`gsi`]; the geometry
//! of `+dgeom`/`+sgeom` in [`geom`]).
//!
//! [`accel`] is `forces.drag(ctx)`. The MATLAB `ctx` is replaced by the explicit
//! [`DragInput`]; the pieces of ctx that belong to other modules (geodetic
//! conversion, time scales, ephemeris) arrive as plain numbers.
//!
//! The models (the relative velocity, the wind frame, the cannonball, the panel force, the probe point,
//! the drag after the density) are env's method env_drag_force, generated from the design into
//! `gen::drag` (tools/engine_build.py); what is left here is the crate's types (the inputs, the
//! space-weather sources, the models as enums, the errors), the density switch's call with the space
//! weather the run holds, and the calls.
#![deny(missing_docs)]
#![allow(rustdoc::broken_intra_doc_links)] // unit brackets like [K], [m/s] in the docs
pub mod geom;
pub mod gsi;

use crate::atmos::{self, AtmosDrivers, AtmosError, AtmosModel, AtmosOut, Composition, Geo};
use crate::atmos::jb2008::JbIndices;
use crate::gen::drag as gd;
use crate::la::{M3, V3};
use crate::spaceweather::{self, ManualIndices, ResearchDrivers, ResearchSw, SpaceWeather, SwError, SwOpts, SwTable};
use geom::Facet;
use gsi::{Gsi, PanelModel};

/// Earth rotation rate [rad/s] (`de440.constants().omega_earth`), the default of
/// relVelocity / cannonball / drag.force and of `ctx.omega_eci` in forces.drag.
pub use crate::frames::omega_earth;

/// `drag.relVelocity(r, v, wind, omega)`: `v - (omega x r) - wind` (the Octave
/// `norm`), returns (vrel, uhat, |vrel|).
pub fn rel_velocity(r: &V3, v: &V3, wind: &V3, omega: &V3) -> (V3, V3, f64) {
    gd::rel_velocity(*r, *v, *wind, *omega)
}

/// `drag.windAngles(vrel, R_bi)` -> (alpha, beta, V, vbody): angle of attack
/// `atan2(w,u)` and sideslip `asin(v/V)` of the body-frame relative wind.
pub fn wind_angles(vrel: &V3, r_bi: &M3) -> (f64, f64, f64, V3) {
    gd::wind_angles(*vrel, *r_bi)
}

/// `drag.bodyToWind(alpha, beta)`: body->wind DCM (F_wind = [-D; S; -L]).
pub fn body_to_wind(alpha: f64, beta: f64) -> M3 {
    gd::body_to_wind(alpha, beta)
}

/// Output of [`cannonball`] (the `out` struct of drag.cannonball): the force [N] and acceleration [m/s^2] ECI
/// (`f`, `a`), the drag 1/2 Cd A rho V^2 [N], the unit relative velocity, |v_rel| [m/s], the Cd used, the dynamic
/// pressure [Pa] and the reference area [m^2] (`a_proj`): the design's record.
pub use crate::gen::drag::CannonballOut;

/// `drag.cannonball(r, v, rho, Cd, A, opts)`: `a = -1/2 Cd A rho |v_rel| v_rel / m`
/// with `v_rel = v - omega x r - wind` (`omega` the 3-vector; a scalar MATLAB omega is
/// `[0,0,omega]`).
pub fn cannonball(r: &V3, v: &V3, rho: f64, cd: f64, area: f64, mass: f64, wind: &V3, omega: &V3) -> CannonballOut {
    gd::drag_cannonball(*r, *v, rho, cd, area, mass, *wind, *omega)
}

/// Options of [`panel_force`] (`opts` of drag.force).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PanelOpts {
    /// mass [kg] (`opts.mass`)
    pub mass: f64,
    /// reference area [m^2] of the coefficients (`opts.Aref`)
    pub aref: f64,
    /// body->inertial DCM
    pub r_bi: M3,
    /// wind [m/s] ECI
    pub wind: V3,
    /// Earth rate [rad/s] ECI (3-vector)
    pub omega: V3,
    /// Sun direction (any length) ECI, needed when a facet is a tracking array
    pub sun_eci: Option<V3>,
}

/// Output of [`panel_force`] (the `out` struct of drag.force): the force [N] and acceleration ECI, the body-axes
/// force, drag, side force and lift [N], the force along and across the relative velocity, its unit vector and size,
/// the atomic-O speed ratio, the dynamic pressure [Pa], the angle of attack and sideslip [rad], the drag, lift and
/// side-force coefficients on Aref, the projected area [m^2], the drag coefficient on it (NaN if nothing faces the
/// flow) and the Aref used: the design's record.
pub use crate::gen::drag::PanelOut;

/// Errors of drag.force / forces.drag.
#[derive(Clone, Debug, PartialEq)]
pub enum DragError {
    /// `drag:force:arrayNeedsSun`
    ArrayNeedsSun(usize),
    /// `drag:force:zeroNormal`
    ZeroNormal(usize),
    /// the density model failed
    Atmos(AtmosError),
    /// the space-weather resolution failed
    SpaceWeather(SwError),
}

impl From<AtmosError> for DragError {
    fn from(e: AtmosError) -> Self {
        DragError::Atmos(e)
    }
}
impl From<SwError> for DragError {
    fn from(e: SwError) -> Self {
        DragError::SpaceWeather(e)
    }
}

/// `drag.force(r, v, atm, facets, model, gsi, opts)`: total free-molecular force
/// (drag + lift + side) on a faceted body, summed over the species of `atm`.
pub fn panel_force(r: &V3, v: &V3, atm: &AtmosOut, facets: &[Facet], model: PanelModel, gsi: &Gsi, opts: &PanelOpts) -> Result<PanelOut, DragError> {
    let (st, k, o) = gd::panel_force(*r, *v, atm_rec(atm), geom::rec(facets), model.choice(), gsi.rec(), opts.mass, opts.aref, opts.r_bi,
        opts.wind, opts.omega, opts.sun_eci.is_some(), opts.sun_eci.unwrap_or([0.0; 3]));
    status(st, k).map(|_| o)
}

/// The air as the design's record (`gen::densitymodel::AtmosOut`).
fn atm_rec(atm: &AtmosOut) -> crate::gen::densitymodel::AtmosOut {
    let mut a = crate::gen::densitymodel::AtmosOut { rho: atm.rho, t: atm.t, ..Default::default() };
    match atm.comp {
        Composition::Species(n) => {
            a.species = true;
            a.n = n;
        }
        Composition::Mean { mmol, n_o } => {
            a.mmol = mmol;
            a.n_o = n_o;
        }
    }
    a
}

/// The error of a status of the design's drag (`gen::drag::DRAGSTATUS_*`), at facet k (from 1).
fn status(st: i64, k: i64) -> Result<(), DragError> {
    match st {
        gd::DRAGSTATUS_ARRAY_NEEDS_SUN => Err(DragError::ArrayNeedsSun(k as usize)),
        gd::DRAGSTATUS_ZERO_NORMAL => Err(DragError::ZeroNormal(k as usize)),
        _ => Ok(()),
    }
}

// ---------------------------------------------------------------------------------
// forces.drag
// ---------------------------------------------------------------------------------

/// `cfg.forces.drag.model`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DragModel {
    /// constant-Cd isotropic drag (the SILS default)
    Cannonball,
    /// a free-molecular panel model through drag.force
    Panel(PanelModel),
}

impl DragModel {
    /// Parse a MATLAB model name ('cannonball' | 'sentman' | 'dria' | 'sesam' | 'cll').
    pub fn from_name(s: &str) -> Option<DragModel> {
        match s.to_ascii_lowercase().as_str() {
            "cannonball" => Some(DragModel::Cannonball),
            "sentman" => Some(DragModel::Panel(PanelModel::Sentman)),
            "dria" => Some(DragModel::Panel(PanelModel::Dria)),
            "sesam" => Some(DragModel::Panel(PanelModel::Sesam)),
            "cll" => Some(DragModel::Panel(PanelModel::Cll)),
            _ => None,
        }
    }
}

/// `cfg.forces.drag`: `.model`, `.atmos`, `.Cd` (cannonball; overrides the spacecraft
/// Cd), `.corotate` (default true), `.gsi` (panel models).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DragConfig {
    /// `.model`
    pub model: DragModel,
    /// `.atmos`
    pub atmos: AtmosModel,
    /// `.Cd` (cannonball; takes precedence over the spacecraft Cd)
    pub cd: Option<f64>,
    /// `.corotate` (co-rotating atmosphere)
    pub corotate: bool,
    /// `.gsi` (panel models)
    pub gsi: Gsi,
}

impl DragConfig {
    /// The SILS in-loop setting (`asils.orbit.init`): cannonball on DTM2020, co-rotating (the design's force set,
    /// env_force_model's `sils_forces`).
    pub fn sils() -> DragConfig {
        crate::accel::Forces::sils(1.0).drag.expect("the SILS force set has drag")
    }
    /// The design's choices (`gen::drag::DRAGMODEL_*`, `gen::gsi::PANELMODEL_*`) of the model.
    fn choice(&self) -> (i64, i64) {
        match self.model {
            DragModel::Cannonball => (gd::DRAGMODEL_CANNONBALL, crate::gen::gsi::PANELMODEL_SENTMAN),
            DragModel::Panel(p) => (gd::DRAGMODEL_PANEL, p.choice()),
        }
    }
}

/// `ctx.sc`: the spacecraft as drag sees it.
#[derive(Clone, Copy, Debug)]
pub struct Spacecraft<'a> {
    /// mass [kg]
    pub mass: f64,
    /// reference area [m^2] (cannonball area; Cd reference of the panel models)
    pub aref: f64,
    /// `sc.Cd` (cannonball fallback; 2.2 when absent)
    pub cd: Option<f64>,
    /// body->inertial DCM (`sc.R_bi`, op.accel sets it per step for 'ram' attitude)
    pub r_bi: M3,
    /// `sc.facets`; `None` -> a single +x plate of area Aref for the panel models
    pub facets: Option<&'a [Facet]>,
}

/// The space-weather sources carried in ctx (`ctx.swmanual`, `ctx.swtable`,
/// `ctx.drv`, `ctx.jbidx`).
#[derive(Clone, Copy, Debug, Default)]
pub struct SwSources<'a> {
    /// `ctx.swmanual` (wins over the table)
    pub manual: Option<&'a ManualIndices>,
    /// `ctx.swtable`
    pub table: Option<&'a SwTable>,
    /// table options (`lag_f107`, `aph_mode`)
    pub opts: SwOpts,
    /// `ctx.drv` for dtm2020_research
    pub research: Option<&'a ResearchDrivers>,
    /// `ctx.jbidx` for jb2008
    pub jb: Option<&'a JbIndices>,
}

/// The explicit form of the MATLAB `ctx` that `forces.drag(ctx)` reads:
///
/// | field | ctx |
/// |---|---|
/// | `r_eci`, `v_eci` | `ctx.r_eci`, `ctx.v_eci` [m, m/s] (GCRF) |
/// | `lat_rad`, `lon_rad`, `alt_m` | `op.geodetic(ctx.r_ecef)` |
/// | `utc` | `ctx.utc` = op.addsec(W.epoch, t) [Y M D h m s] |
/// | `doy` | `ctx.T.doy` (timeconv.convertUTC) |
/// | `omega_eci` | `ctx.omega_eci` (W.omega_eci; `None` -> [0,0,omega_earth]) |
/// | `sun_eci` | `ctx.E.sun_eci` (only needed by tracking arrays) |
/// | `sc` | `ctx.sc` (mass, Aref, Cd, R_bi, facets) |
/// | `cfg` | `ctx.cfg.forces.drag` |
/// | `sw` | `ctx.swmanual`, `ctx.swtable`, `ctx.drv`, `ctx.jbidx` |
#[derive(Clone, Copy, Debug)]
pub struct DragInput<'a> {
    /// `ctx.r_eci` [m]
    pub r_eci: V3,
    /// `ctx.v_eci` [m/s]
    pub v_eci: V3,
    /// geodetic latitude [rad] of `op.geodetic(ctx.r_ecef)`
    pub lat_rad: f64,
    /// longitude [rad]
    pub lon_rad: f64,
    /// geodetic altitude [m]
    pub alt_m: f64,
    /// `ctx.utc` [Y M D h m s]
    pub utc: [f64; 6],
    /// `ctx.T.doy`
    pub doy: f64,
    /// `ctx.omega_eci` [rad/s]
    pub omega_eci: Option<V3>,
    /// `ctx.E.sun_eci`
    pub sun_eci: Option<V3>,
    /// `ctx.sc`
    pub sc: Spacecraft<'a>,
    /// `ctx.cfg.forces.drag`
    pub cfg: &'a DragConfig,
    /// `ctx.swmanual` / `swtable` / `drv` / `jbidx`
    pub sw: SwSources<'a>,
}

/// The drivers forces.drag actually handed to the density model (`info.sw`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SwUsed {
    /// no drivers (exponential)
    None,
    /// atmos.spaceweather output
    Sw(SpaceWeather),
    /// atmos.research_drivers output
    Research(ResearchSw),
    /// the SET tables
    Jb,
}

/// The model output inside [`DragInfo`] (`info.out`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DragOut {
    /// drag.cannonball output
    Cannonball(CannonballOut),
    /// drag.force output
    Panel(PanelOut),
}

/// `info` of `[a, info] = forces.drag(ctx)`: what the model actually used.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DragInfo {
    /// `info.atm.rho` [kg/m^3] (what asils reads)
    pub rho: f64,
    /// `info.atm`
    pub atm: AtmosOut,
    /// `info.geo`
    pub geo: Geo,
    /// `info.sw`
    pub sw: SwUsed,
    /// `info.v_rel` [m/s]
    pub v_rel: V3,
    /// `info.out`
    pub out: DragOut,
}

/// The geodetic probe point of forces.drag: degrees, km, LST = mod(UT_h + lon/15, 24).
pub fn probe_geo(lat_rad: f64, lon_rad: f64, alt_m: f64, utc: &[f64; 6], doy: f64) -> Geo {
    let (alt_km, lat_deg, lon_deg, lst_h) = gd::probe_geo(lat_rad, lon_rad, alt_m, *utc);
    Geo { alt_km, lat_deg, lon_deg, lst_h, doy, utc: *utc }
}

/// `atmos.provider(model, geo, sw)` with the drivers resolved the way forces.drag
/// resolves them for `model` (exponential: none; jb2008: `jb_idx`; dtm2020_research:
/// atmos.research_drivers; dtm2020/nrlmsise: atmos.spaceweather manual/table).
pub fn density(model: AtmosModel, geo: &Geo, sw: &SwSources) -> Result<(AtmosOut, SwUsed), DragError> {
    Ok(match model {
        AtmosModel::Exponential => (atmos::density(model, geo, &AtmosDrivers::None)?, SwUsed::None),
        AtmosModel::Jb2008 => {
            let idx = sw.jb.ok_or(AtmosError::MissingDriver("jb_idx"))?;
            (atmos::density(model, geo, &AtmosDrivers::Jb(idx, None))?, SwUsed::Jb)
        }
        AtmosModel::Dtm2020Research => {
            let d = sw.research.ok_or(SwError::ResearchCoverage("DRV"))?;
            let r = spaceweather::research_sample(d, &geo.utc)?;
            (atmos::density(model, geo, &AtmosDrivers::Research(&r))?, SwUsed::Research(r))
        }
        AtmosModel::Dtm2020 | AtmosModel::Nrlmsise => {
            let s = spaceweather::resolve(&geo.utc, sw.manual, sw.table, sw.opts)?;
            (atmos::density(model, geo, &AtmosDrivers::Sw(&s))?, SwUsed::Sw(s))
        }
    })
}

/// `[a, info] = forces.drag(ctx)`: drag acceleration [m/s^2] ECI and what produced it.
pub fn accel(inp: &DragInput) -> Result<(V3, DragInfo), DragError> {
    let d = inp.cfg;
    let geo = probe_geo(inp.lat_rad, inp.lon_rad, inp.alt_m, &inp.utc, inp.doy);
    let (atm, swu) = density(d.atmos, &geo, &inp.sw)?;
    let sc = &inp.sc;
    let (model, pm) = d.choice();
    let facets = match (d.model, sc.facets) {
        (DragModel::Panel(_), Some(f)) => geom::rec(f),
        _ => crate::gen::srp::ScFacets::default(),
    };
    let (st, k, a, v_rel, cb, po) = gd::drag_force(model, pm, inp.r_eci, inp.v_eci, atm_rec(&atm), inp.omega_eci.is_some(), inp.omega_eci.unwrap_or([0.0; 3]),
        d.corotate, d.cd.is_some(), d.cd.unwrap_or(0.0), sc.cd.is_some(), sc.cd.unwrap_or(0.0), sc.aref, sc.mass, sc.r_bi, facets, d.gsi.rec(),
        inp.sun_eci.is_some(), inp.sun_eci.unwrap_or([0.0; 3]));
    status(st, k)?;
    let out = match d.model { DragModel::Cannonball => DragOut::Cannonball(cb), DragModel::Panel(_) => DragOut::Panel(po) };
    Ok((a, DragInfo { rho: atm.rho, atm, geo, sw: swu, v_rel, out }))
}
