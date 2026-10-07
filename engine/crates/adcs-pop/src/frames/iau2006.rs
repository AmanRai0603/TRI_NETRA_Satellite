//! IAU 2006/2000A CIO-based kernel shared by `frames.eci2ecef_A/B/C`: env's method env_iau2006 (the port of their
//! local functions `fund_args`, `xy06`, `s06`, `sp00`, `c2ixys`, `era00`, `pom00`, `cal2jd`, `mjd_of`, `time_scales`;
//! the three builds share this kernel verbatim and differ only in EOP handling) over env's xys06 series (the IAU
//! 2006/2000A X, Y and s series, IERS Conventions 2010 tables 5.2a, 5.2b, 5.2d), generated from the design into
//! `gen::iau2006` (tools/engine_build.py). What is left here is the crate's names for them.
use crate::gen::iau2006 as g;
use crate::la::M3;

pub use crate::gen::iau2006::{CioParts, TimeScales, AS2R};

/// `fund_args(t)`: the 14 IERS 2003 fundamental arguments (rad), t = TT centuries.
pub fn fund_args(t: f64) -> [f64; 14] { g::fund_args(t) }
/// `xy06(t, fa, tab)`: CIP X, Y (rad) from the IAU 2006/2000A series.
pub fn xy06(t: f64, fa: &[f64; 14]) -> (f64, f64) { g::xy06(t, *fa) }
/// `s06(t, fa, x, y, tab)`: CIO locator s (rad), given X, Y.
pub fn s06(t: f64, fa: &[f64; 14], x: f64, y: f64) -> f64 { g::s06(t, *fa, x, y) }
/// `sp00(t)`: TIO locator s' (rad).
pub fn sp00(t: f64) -> f64 { g::sp00(t) }
/// `c2ixys(x, y, s)`: celestial-to-intermediate matrix from X, Y, s.
pub fn c2ixys(x: f64, y: f64, s: f64) -> M3 { g::c2ixys(x, y, s) }
/// `era00(dj1, dj2)`: Earth rotation angle (rad) from a two-part UT1 JD.
pub fn era00(dj1: f64, dj2: f64) -> f64 { g::era00(dj1, dj2) }
/// `pom00(xp, yp, sp)`: polar-motion matrix W = R1(-yp) R2(-xp) R3(sp).
pub fn pom00(xp: f64, yp: f64, sp: f64) -> M3 { g::pom00(xp, yp, sp) }
/// `cal2jd(iy, im, id)` (SOFA iauCal2jd, local to the builds): `(djm0, djm)`.
pub fn cal2jd_sofa(iy: f64, im: f64, id: f64) -> (f64, f64) { g::cal2jd_sofa(iy, im, id) }
/// `mjd_of(u)`: UTC MJD of a `[Y Mo D H Mi S]` vector.
pub fn mjd_of(u: &[f64; 6]) -> f64 { g::mjd_of(*u) }
/// `time_scales(u, dUT1, dAT)`: two-part UT1/TT JDs and TT centuries.
pub fn time_scales(u: &[f64; 6], dut1: f64, dat: f64) -> TimeScales { g::time_scales(*u, dut1, dat) }
/// The loop body of `eci2ecef_A/B/C` for one epoch, given its EOP: `C = W * R3(era) * Q` (r_ecef = C r_eci). Angles
/// in rad, dut1/dat in s.
pub fn cio_c2t(u: &[f64; 6], dut1: f64, xp: f64, yp: f64, dx: f64, dy: f64, dat: f64) -> (M3, CioParts) { g::cio_c2t(*u, dut1, xp, yp, dx, dy, dat) }
