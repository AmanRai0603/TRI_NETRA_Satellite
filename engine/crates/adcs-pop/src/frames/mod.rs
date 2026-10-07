//! ECI (GCRF/J2000) <-> ECEF (ITRF) rotations -- port of
//! `matlab_sils/pop/03_frames_time/+frames/` (`eci2ecef.m` dispatcher,
//! `eci2ecefGMST.m`, `eci2ecef_A.m`, `eci2ecef_B.m`, `eci2ecef_C.m`, `utcvec.m`)
//! and of `earthRateECI`, the local function of `01_core/+op/buildWorld.m`.
//!
//! Builds (as the MATLAB headers define them):
//! * `Gmst` -- offline GMST-only rotation (IAU 1982 GMST, optional small-angle
//!   polar motion); no precession/nutation. POP's default and offline fallback.
//! * `A` -- full IAU 2006/2000A CIO chain with finals2000A.all EOP (Bulletin A/B).
//! * `B` -- same chain with EOP 20 C04 spliced with the finals tail.
//! * `C` -- B + 4-point Lagrange EOP interpolation, RG_ZONT2 UT1 regularisation
//!   and the IERS sub-daily tidal EOP model.
//!
//! Like MATLAB, every call returns `(C, Ct)` with `r_ecef = C r_eci` and
//! `Ct = C'` (the ECEF->ECI matrix; *not* a time derivative -- use
//! [`earth_rate_eci`] for the rotation rate, as POP does).
use crate::la::{transpose, M3, V3};
use crate::time::convert_utc;

pub mod iau2006;
pub mod tidal;

pub use iau2006::{c2ixys, cio_c2t, era00, fund_args, pom00, s06, sp00, xy06, CioParts};

/// Frame build selector (`build` argument of `frames.eci2ecef`: 'gmst' | 'A' | 'B' | 'C').
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Build {
    /// 'gmst': GMST-only, offline.
    #[default]
    Gmst,
    /// 'A': IAU 2006/2000A + finals2000A.
    A,
    /// 'B': IAU 2006/2000A + EOP 20 C04 (+ finals tail).
    B,
    /// 'C': build B + Lagrange EOP, zonal UT1 regularisation, sub-daily tides.
    C,
}

impl Build {
    /// Parse the MATLAB build string (case-insensitive, as `upper(build)`).
    pub fn parse(s: &str) -> Option<Build> {
        match s.to_ascii_uppercase().as_str() {
            "A" => Some(Build::A),
            "B" => Some(Build::B),
            "C" => Some(Build::C),
            "GMST" => Some(Build::Gmst),
            _ => None,
        }
    }
}

/// Errors where MATLAB's builds `error(...)`.
#[derive(Clone, Debug, PartialEq)]
pub enum FrameError {
    /// Build A/B/C with no EOP loaded (MATLAB: "could not obtain <file>" when the
    /// cache is empty and the download fails).
    NoEop,
    /// Build B/C with no EOP 20 C04 file in the EOP set.
    NoC04,
    /// `Eop::at` asked for the gmst build (it uses no EOP table).
    GmstHasNoEop,
}

impl std::fmt::Display for FrameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FrameError::NoEop => write!(f, "eci2ecef: could not obtain finals2000A.all (no EOP loaded; set FrameOpt.eop or use build gmst)"),
            FrameError::NoC04 => write!(f, "eci2ecef: could not obtain eopc04_20.1962-now (build B/C need EOP 20 C04)"),
            FrameError::GmstHasNoEop => write!(f, "eci2ecef: build gmst uses no EOP table"),
        }
    }
}
impl std::error::Error for FrameError {}

/// The `opt` struct of `frames.eci2ecef` (fields read by the dispatcher and builds).
#[derive(Clone, Debug)]
pub struct FrameOpt {
    /// `opt.dUT1` UT1-UTC (s) -- gmst build only (default 0).
    pub dut1: f64,
    /// `opt.xp` pole x (rad) -- gmst build only (default 0).
    pub xp: f64,
    /// `opt.yp` pole y (rad) -- gmst build only (default 0).
    pub yp: f64,
    /// `opt.gmst_rad` precomputed GMST (rad) -- gmst build only (default: from utc).
    pub gmst_rad: Option<f64>,
    /// EOP set for builds A/B/C (MATLAB reads it from `opt.data_dir`; see [`crate::eop::Eop::from_dir`]).
    pub eop: Option<crate::eop::Eop>,
    /// `opt.tidal` (builds A/B): add the IERS sub-daily tidal EOP model (default false).
    pub tidal: bool,
    /// `opt.eop_override = [dUT1 xp yp dX dY dAT]` (s, rad, rad, rad, rad, s), builds A/B/C.
    pub eop_override: Option<[f64; 6]>,
}

impl Default for FrameOpt {
    fn default() -> Self {
        FrameOpt { dut1: 0.0, xp: 0.0, yp: 0.0, gmst_rad: None, eop: None, tidal: false, eop_override: None }
    }
}

/// The `info` output of `frames.eci2ecef` (scalar-epoch form).
#[derive(Clone, Copy, Debug, Default)]
pub struct FrameInfo {
    /// build used
    pub build: Build,
    /// `info.gmst_rad` (gmst build)
    pub gmst_rad: f64,
    /// `info.mjd_utc` (A/B/C)
    pub mjd_utc: f64,
    /// `info.dUT1` (s)
    pub dut1: f64,
    /// `info.xp` (rad)
    pub xp: f64,
    /// `info.yp` (rad)
    pub yp: f64,
    /// `info.dX` (rad)
    pub dx: f64,
    /// `info.dY` (rad)
    pub dy: f64,
    /// `info.dAT` TAI-UTC (s)
    pub dat: f64,
    /// `info.X` CIP X incl. dX (rad)
    pub x: f64,
    /// `info.Y` CIP Y incl. dY (rad)
    pub y: f64,
    /// `info.s` CIO locator (rad)
    pub s: f64,
    /// `info.era` Earth rotation angle (rad)
    pub era: f64,
    /// `info.eop_flag`: 0 interpolated, 1 extrapolated, 2 eop_override
    pub eop_flag: i32,
}

/// `frames.utcvec(utc)`: normalise a numeric UTC input to `[Y Mo D H Mi S]`.
/// Panics (MATLAB `error`) with fewer than 6 elements.
pub fn utcvec(utc: &[f64]) -> [f64; 6] {
    assert!(utc.len() >= 6, "frames:utcvec: UTC must be datetime or [Y Mo D H Mi S]");
    [utc[0], utc[1], utc[2], utc[3], utc[4], utc[5]]
}

/// `frames.eci2ecefGMST(gmst_rad, xp, yp)`: `C = W * R3(gmst)` with the small-angle
/// polar-motion matrix `W = [1 0 xp; 0 1 -yp; -xp yp 1]` (only when xp or yp != 0):
/// env's method env_earth_frames (`gen::earthframes`).
pub fn eci2ecef_gmst(gmst_rad: f64, xp: f64, yp: f64) -> (M3, M3) { crate::gen::earthframes::eci2ecef_gmst(gmst_rad, xp, yp) }

/// Body of `frames.eci2ecef_A/B/C` for one epoch (the builds differ only in EOP):
/// EOP from `opt.eop_override` (flag 2) or `eop_interp`, then the CIO chain.
pub fn eci2ecef_cio(utc: [f64; 6], build: Build, opt: &FrameOpt) -> Result<(M3, M3, FrameInfo), FrameError> {
    let mjd = iau2006::mjd_of(&utc);
    let (dut1, xp, yp, dx, dy, dat, ef) = if let Some(eo) = opt.eop_override {
        // MATLAB still loads the EOP files first (and errors offline); the override
        // itself does not use them, so the port accepts it without an EOP set.
        (eo[0], eo[1], eo[2], eo[3], eo[4], eo[5], 2)
    } else {
        let eop = opt.eop.as_ref().ok_or(FrameError::NoEop)?;
        let e = eop.at(mjd, build, opt.tidal)?;
        (e.dut1, e.xp, e.yp, e.dx, e.dy, e.dat, e.flag)
    };
    let (c, p) = cio_c2t(&utc, dut1, xp, yp, dx, dy, dat);
    let info = FrameInfo { build, gmst_rad: 0.0, mjd_utc: mjd, dut1, xp, yp, dx, dy, dat, x: p.x, y: p.y, s: p.s, era: p.era, eop_flag: ef };
    Ok((c, transpose(&c), info))
}

fn eci2ecef_impl(utc: [f64; 6], build: Build, opt: &FrameOpt, gmst: Option<f64>) -> Result<(M3, M3, FrameInfo), FrameError> {
    match build {
        Build::Gmst => {
            let g = match gmst.or(opt.gmst_rad) {
                Some(g) => g,
                None => convert_utc(utc[0], utc[1], utc[2], utc[3], utc[4], utc[5], opt.dut1).gmst_rad,
            };
            let (c, ct) = eci2ecef_gmst(g, opt.xp, opt.yp);
            Ok((c, ct, FrameInfo { build, gmst_rad: g, ..Default::default() }))
        }
        _ => eci2ecef_cio(utc, build, opt),
    }
}

/// `[C, Ct, info] = frames.eci2ecef(utc, build, opt)`, fallible form.
pub fn try_eci2ecef(utc: [f64; 6], build: Build, opt: &FrameOpt) -> Result<(M3, M3, FrameInfo), FrameError> {
    eci2ecef_impl(utc, build, opt, None)
}

/// `[C, Ct] = frames.eci2ecef(utc, build, opt)`: `r_ecef = C r_eci`, `Ct = C'`.
/// Panics where MATLAB errors (build A/B/C without EOP); see [`try_eci2ecef`].
pub fn eci2ecef(utc: [f64; 6], build: Build, opt: &FrameOpt) -> (M3, M3) {
    match eci2ecef_impl(utc, build, opt, None) {
        Ok((c, ct, _)) => (c, ct),
        Err(e) => panic!("{e}"),
    }
}

/// Earth mean rotation rate `de440.constants().omega_earth` (rad/s).
pub const OMEGA_EARTH: f64 = 7.2921150e-5;

/// `earthRateECI(epoch, build, fopt)` (local function of `op.buildWorld`): the true
/// Earth angular-velocity vector in ECI (rad/s), from a +-15 s central difference of
/// `Ct` (`[omega]x = Cdot*C0`). As MATLAB: the gmst build's GMST comes from
/// `convertUTC(...,0)` (dUT1 = 0), and any frame error falls back to `[0,0,omega]`.
pub fn earth_rate_eci(epoch: [f64; 6], build: Build, opt: &FrameOpt) -> V3 {
    let d = 15.0;
    let up = crate::time::addsec(epoch, d);
    let um = crate::time::addsec(epoch, -d);
    let tp = convert_utc(up[0], up[1], up[2], up[3], up[4], up[5], 0.0);
    let tm = convert_utc(um[0], um[1], um[2], um[3], um[4], um[5], 0.0);
    let t0 = convert_utc(epoch[0], epoch[1], epoch[2], epoch[3], epoch[4], epoch[5], 0.0);
    let r = (|| -> Result<V3, FrameError> {
        let (_, ctp, _) = eci2ecef_impl(up, build, opt, Some(tp.gmst_rad))?;
        let (_, ctm, _) = eci2ecef_impl(um, build, opt, Some(tm.gmst_rad))?;
        let (c0, _, _) = eci2ecef_impl(epoch, build, opt, Some(t0.gmst_rad))?;
        Ok(crate::gen::earthframes::earth_rate_from(ctp, ctm, c0, d))
    })();
    // MATLAB warns ('op:buildWorld:earthRate') and falls back
    r.unwrap_or([0.0, 0.0, OMEGA_EARTH])
}
