//! Sub-daily and zonal tidal EOP models: env's method env_tidal_eop (the port of
//! `matlab_sils/pop/03_frames_time/tidal_eop_models/` -- `tidal_eop.m`, `tidal_eop_ocean.m`,
//! `tidal_pm_libration.m`, `tidal_ut1_libration.m`, `tidal_ut1_zonal.m` -- themselves the MATLAB ports of the IERS
//! Conventions (2010) ORTHO_EOP/CNMTX, PMSDNUT2, UTLIBR and RG_ZONT2 routines used by `frames.eci2ecef_A/B/C`) over
//! env's tables of their terms, generated from the design into `gen::tidaleop` (tools/engine_build.py).

/// `tidal_eop(mjd)`: full IERS sub-daily EOP correction = ocean tides + PM libration
/// + UT1 libration. Returns `(dxp, dyp, dut1)` in microarcseconds / microseconds.
/// `tidal_eop_ocean(mjd)`: diurnal + semidiurnal ocean-tide variations in polar
/// motion and UT1 (IERS ORTHO_EOP). Returns `(dxp, dyp, dut1)` in uas, uas, us.
/// `tidal_pm_libration(rmjd)`: diurnal lunisolar libration in polar motion
/// (IERS PMSDNUT2). Returns `(dxp, dyp)` in microarcseconds.
/// `tidal_ut1_libration(rmjd)`: subdiurnal libration in UT1 (IERS UTLIBR).
/// Returns `(dut1 [us], dlod [us/day])`.
/// `tidal_ut1_zonal(mjd)`: zonal-tide variation in UT1 (IERS RG_ZONT2, 62 terms),
/// in SECONDS. Used by build C to regularise UT1 before interpolation.
pub use crate::gen::tidaleop::{tidal_eop, tidal_eop_ocean, tidal_pm_libration, tidal_ut1_libration, tidal_ut1_zonal};
