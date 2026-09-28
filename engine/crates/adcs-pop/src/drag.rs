//! Aerodynamic drag: `02_forces/+forces/drag.m` (the force term op.accel calls) and
//! `02_forces/+drag/*` (cannonball, force = multi-species free-molecular panel model,
//! relVelocity, windAngles, bodyToWind; the GSI coefficients in [`gsi`]; the geometry
//! of `+dgeom`/`+sgeom` in [`geom`]).
//!
//! [`accel`] is `forces.drag(ctx)`. The MATLAB `ctx` is replaced by the explicit
//! [`DragInput`]; the pieces of ctx that belong to other modules (geodetic
//! conversion, time scales, ephemeris) arrive as plain numbers.
//!
//! No allocation on the hot path: facets are borrowed, species arrays are fixed size.
#![deny(missing_docs)]
#![allow(rustdoc::broken_intra_doc_links)] // unit brackets like [K], [m/s] in the docs
pub mod geom;
pub mod gsi;

use crate::atmos::octave::{norm, rad2deg, mod_};
use crate::atmos::{self, AtmosDrivers, AtmosError, AtmosModel, AtmosOut, Composition, Geo};
use crate::atmos::jb2008::JbIndices;
use crate::la::{cross, dot, mtv, mv, sub, M3, V3};
use crate::spaceweather::{self, ManualIndices, ResearchDrivers, ResearchSw, SpaceWeather, SwError, SwOpts, SwTable};
use geom::{array_normal, Facet, FacetKind};
use gsi::{Gsi, PanelModel};

/// Earth rotation rate [rad/s] (`de440.constants().omega_earth`), the default of
/// relVelocity / cannonball / drag.force and of `ctx.omega_eci` in forces.drag.
pub const OMEGA_EARTH: f64 = 7.2921150e-5;

/// `drag.relVelocity(r, v, wind, omega)`: `v - (omega x r) - wind` (the Octave
/// `norm`), returns (vrel, uhat, |vrel|).
pub fn rel_velocity(r: &V3, v: &V3, wind: &V3, omega: &V3) -> (V3, V3, f64) {
    let c = cross(omega, r);
    let vatm = [c[0] + wind[0], c[1] + wind[1], c[2] + wind[2]];
    let vrel = sub(v, &vatm);
    let vm = norm(&vrel);
    (vrel, [vrel[0] / vm, vrel[1] / vm, vrel[2] / vm], vm)
}

/// `drag.windAngles(vrel, R_bi)` -> (alpha, beta, V, vbody): angle of attack
/// `atan2(w,u)` and sideslip `asin(v/V)` of the body-frame relative wind.
#[allow(clippy::manual_clamp)] // max(min(v/V,1),-1): NaN-ignoring like MATLAB
pub fn wind_angles(vrel: &V3, r_bi: &M3) -> (f64, f64, f64, V3) {
    let vb = mtv(r_bi, vrel);
    let vm = norm(&vb);
    let alpha = vb[2].atan2(vb[0]);
    let beta = (vb[1] / vm).min(1.0).max(-1.0).asin();
    (alpha, beta, vm, vb)
}

/// `drag.bodyToWind(alpha, beta)`: body->wind DCM (F_wind = [-D; S; -L]).
pub fn body_to_wind(alpha: f64, beta: f64) -> M3 {
    let (ca, sa, cb, sb) = (alpha.cos(), alpha.sin(), beta.cos(), beta.sin());
    [[cb * ca, sb, cb * sa], [-sb * ca, cb, -sb * sa], [-sa, 0.0, ca]]
}

/// Output of [`cannonball`] (the `out` struct of drag.cannonball).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CannonballOut {
    /// force [N] ECI (`out.F`)
    pub f: V3,
    /// acceleration [m/s^2] ECI (`out.a`)
    pub a: V3,
    /// drag magnitude 1/2 Cd A rho V^2 [N]
    pub drag: f64,
    /// unit relative velocity
    pub uhat: V3,
    /// |v_rel| [m/s]
    pub vrel: f64,
    /// the Cd used
    pub cd: f64,
    /// dynamic pressure 1/2 rho V^2 [Pa]
    pub qd: f64,
    /// reference area used [m^2] (`out.A_proj` = A)
    pub a_proj: f64,
}

/// `drag.cannonball(r, v, rho, Cd, A, opts)`: `a = -1/2 Cd A rho |v_rel| v_rel / m`
/// with `v_rel = v - omega x r - wind` (`omega` the 3-vector; a scalar MATLAB omega is
/// `[0,0,omega]`).
pub fn cannonball(r: &V3, v: &V3, rho: f64, cd: f64, area: f64, mass: f64, wind: &V3, omega: &V3) -> CannonballOut {
    let c = cross(omega, r);
    let t = sub(v, &c);
    let vrel = sub(&t, wind);
    let vm = norm(&vrel);
    let uhat = [vrel[0] / vm, vrel[1] / vm, vrel[2] / vm];
    let k = -0.5 * cd * area * rho * vm;
    let f = [k * vrel[0], k * vrel[1], k * vrel[2]];
    CannonballOut {
        f,
        a: [f[0] / mass, f[1] / mass, f[2] / mass],
        drag: 0.5 * cd * area * rho * crate::atmos::octave::pw(vm, 2.0),
        uhat,
        vrel: vm,
        cd,
        qd: 0.5 * rho * crate::atmos::octave::pw(vm, 2.0),
        a_proj: area,
    }
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

/// Output of [`panel_force`] (the `out` struct of drag.force).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PanelOut {
    /// force [N] ECI (`out.F`)
    pub f: V3,
    /// acceleration [m/s^2] ECI
    pub a: V3,
    /// force in body axes [N]
    pub fbody: V3,
    /// drag D [N] (wind frame)
    pub drag: f64,
    /// side force S [N]
    pub side: f64,
    /// lift L [N]
    pub lift: f64,
    /// force component along the relative velocity [N]
    pub drag_vec: V3,
    /// force component normal to it [N]
    pub lift_vec: V3,
    /// unit relative velocity
    pub uhat: V3,
    /// |v_rel| [m/s]
    pub vrel: f64,
    /// atomic-O speed ratio
    pub s_o: f64,
    /// dynamic pressure of the summed species [Pa]
    pub qd: f64,
    /// angle of attack [rad]
    pub alpha: f64,
    /// sideslip [rad]
    pub beta: f64,
    /// drag coefficient referenced to Aref
    pub cd: f64,
    /// lift coefficient referenced to Aref
    pub cl: f64,
    /// side-force coefficient referenced to Aref
    pub cs: f64,
    /// projected area in the flow [m^2]
    pub a_proj: f64,
    /// drag coefficient referenced to the projected area (NaN if nothing faces the flow)
    pub cd_a: f64,
    /// the Aref the coefficients refer to [m^2]
    pub aref_used: f64,
}

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
    const NA: f64 = 6.02214076e26;
    let (vrel, uhat, vm) = rel_velocity(r, v, &opts.wind, &opts.omega);
    let (alpha, beta, _, _) = wind_angles(&vrel, &opts.r_bi);
    // species list -> (rho_s, M_s)
    let mut rho_s = [0.0f64; 6];
    let mut ms = [0.0f64; 6];
    let ns;
    let n_o;
    match atm.comp {
        Composition::Species(n) => {
            ns = 6;
            for i in 0..6 {
                ms[i] = gsi::M_DTM[i];
                rho_s[i] = n[i] * ms[i] / NA;
            }
            n_o = n[2];
        }
        Composition::Mean { mmol, n_o: no } => {
            ns = 1;
            rho_s[0] = atm.rho;
            ms[0] = mmol;
            n_o = no;
        }
    }
    let a_t = match model {
        PanelModel::Dria | PanelModel::Sesam => gsi::sesam(n_o, atm.t),
        PanelModel::Sentman => gsi.a_t,
        PanelModel::Cll => 0.0,
    };
    let mut f = [0.0f64; 3];
    let mut aproj = 0.0;
    let s_o = gsi::speed_ratio(vm, atm.t, 15.9994);
    for (k, fk) in facets.iter().enumerate() {
        let mut nb = fk.n;
        if fk.kind == FacetKind::Array {
            let sun = opts.sun_eci.ok_or(DragError::ArrayNeedsSun(k + 1))?;
            let ns_ = norm(&sun);
            let sh = [sun[0] / ns_, sun[1] / ns_, sun[2] / ns_];
            let sb = mtv(&opts.r_bi, &sh);
            nb = array_normal(&fk.axis, &sb);
        }
        if norm(&nb) < f64::EPSILON {
            return Err(DragError::ZeroNormal(k + 1));
        }
        let n0 = mv(&opts.r_bi, &nb);
        let nn = norm(&n0);
        let mut n = [n0[0] / nn, n0[1] / nn, n0[2] / nn];
        let mut cosd = dot(&n, &uhat);
        if fk.double && cosd < 0.0 {
            n = [-n[0], -n[1], -n[2]];
            cosd = -cosd;
        }
        if cosd <= 0.0 {
            continue;
        }
        aproj += fk.a * cosd;
        let delta = cosd.min(1.0).acos();
        let sind = (1.0 - crate::atmos::octave::pw(cosd, 2.0)).max(0.0).sqrt();
        let tgas = if sind > 1e-9 {
            [(-uhat[0] + cosd * n[0]) / sind, (-uhat[1] + cosd * n[1]) / sind, (-uhat[2] + cosd * n[2]) / sind]
        } else {
            [0.0; 3]
        };
        for j in 0..ns {
            let s = gsi::speed_ratio(vm, atm.t, ms[j]);
            let (cp, ct) = match model {
                PanelModel::Sentman | PanelModel::Dria | PanelModel::Sesam => gsi::sentman(s, delta, a_t, gsi.tw, atm.t),
                PanelModel::Cll => gsi::cll(s, delta, gsi.sig_n, gsi.sig_t, gsi.tw, atm.t),
            };
            let q = 0.5 * rho_s[j] * crate::atmos::octave::pw(vm, 2.0) * fk.a;
            for i in 0..3 {
                f[i] += q * (ct * tgas[i] - cp * n[i]);
            }
        }
    }
    let fbody = mtv(&opts.r_bi, &f);
    let fwind = mv(&body_to_wind(alpha, beta), &fbody);
    let d = -fwind[0];
    let s = fwind[1];
    let l = -fwind[2];
    let mut rho_tot = 0.0;
    for x in rho_s.iter().take(ns) {
        rho_tot += x;
    }
    let qd = 0.5 * rho_tot * crate::atmos::octave::pw(vm, 2.0);
    let fu = dot(&f, &uhat);
    let drag_vec = [fu * uhat[0], fu * uhat[1], fu * uhat[2]];
    Ok(PanelOut {
        f,
        a: [f[0] / opts.mass, f[1] / opts.mass, f[2] / opts.mass],
        fbody,
        drag: d,
        side: s,
        lift: l,
        drag_vec,
        lift_vec: sub(&f, &drag_vec),
        uhat,
        vrel: vm,
        s_o,
        qd,
        alpha,
        beta,
        cd: d / (qd * opts.aref),
        cl: l / (qd * opts.aref),
        cs: s / (qd * opts.aref),
        a_proj: aproj,
        cd_a: if aproj > 0.0 { d / (qd * aproj) } else { f64::NAN },
        aref_used: opts.aref,
    })
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
    /// The SILS in-loop setting (`asils.orbit.init`): cannonball on DTM2020, co-rotating.
    pub const SILS: DragConfig = DragConfig {
        model: DragModel::Cannonball,
        atmos: AtmosModel::Dtm2020,
        cd: None,
        corotate: true,
        gsi: Gsi { tw: 300.0, a_t: 0.9, sig_n: 0.9, sig_t: 0.9 },
    };
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
    let lon_deg = rad2deg(lon_rad);
    let utsec = utc[3] * 3600.0 + utc[4] * 60.0 + utc[5];
    Geo { alt_km: alt_m / 1000.0, lat_deg: rad2deg(lat_rad), lon_deg, lst_h: mod_(utsec / 3600.0 + lon_deg / 15.0, 24.0), doy, utc: *utc }
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
    let om_ctx = inp.omega_eci.unwrap_or([0.0, 0.0, OMEGA_EARTH]);
    let v_rel = if d.corotate { sub(&inp.v_eci, &cross(&om_ctx, &inp.r_eci)) } else { inp.v_eci };
    let om = if d.corotate { om_ctx } else { [0.0; 3] };
    let sc = &inp.sc;
    let (a, out) = match d.model {
        DragModel::Cannonball => {
            let cd = d.cd.or(sc.cd).unwrap_or(2.2);
            let o = cannonball(&inp.r_eci, &inp.v_eci, atm.rho, cd, sc.aref, sc.mass, &[0.0; 3], &om);
            (o.a, DragOut::Cannonball(o))
        }
        DragModel::Panel(pm) => {
            let plate = [Facet::plain([1.0, 0.0, 0.0], sc.aref)];
            let facets: &[Facet] = match sc.facets {
                Some(f) if !f.is_empty() => f,
                _ => &plate,
            };
            let opts = PanelOpts { mass: sc.mass, aref: sc.aref, r_bi: sc.r_bi, wind: [0.0; 3], omega: om, sun_eci: inp.sun_eci };
            let o = panel_force(&inp.r_eci, &inp.v_eci, &atm, facets, pm, &d.gsi, &opts)?;
            (o.a, DragOut::Panel(o))
        }
    };
    Ok((a, DragInfo { rho: atm.rho, atm, geo, sw: swu, v_rel, out }))
}
