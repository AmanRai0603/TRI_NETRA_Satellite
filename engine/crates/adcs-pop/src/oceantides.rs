//! Ocean tides: ports of `matlab_sils/pop/02_forces/+oceantides/*` (`doodson`,
//! `mainLines`, `fromModel`) and `02_forces/+forces/oceantides.m`: env's method env_ocean_tides over env's
//! ocean-tide tables (env_ocean_tide_tables: the main lines, and FES2004 to degree 10, read into the design by
//! tools/readers.py from `data/gravity_tides/fes2004_deg10.bin`, refgen's bit-exact export of
//! `force_data/fes2004_deg10.mat`), generated from the design into `gen::oceantides` (tools/engine_build.py).
//!
//! As in the MATLAB, `forces.oceantides` uses the 8 representative main lines
//! (`mainLines`), not the FES table; `fromModel` is the "advanced" path that the
//! MATLAB provides but does not wire into the force sum.
//!
//! ctx mapping (`op.accel`): `ctx.T.tt_jd` from `timeconv.convertUTC`,
//! `ctx.r_ecef = C*r_eci`, `ctx.C` from `frames.eci2ecef`, `ctx.grav.mu / Re`.
//!
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use crate::la::{M3, V3};

/// MATLAB/Octave `mod(x, y)` for finite doubles (`x - floor(x/y)*y`, result with
/// the sign of `y`; Octave's near-integer-quotient rule for non-integer `y`): the design's.
pub use crate::gen::oceantides::ocean_mod as octave_mod;

/// Port of `oceantides.doodson(jd_tt)`: the six Doodson fundamental arguments
/// `[tau, s, h, p, N', ps]` [rad] at a TT Julian date.
pub use crate::gen::oceantides::doodson;

/// Port of `oceantides.mainLines(jd_tt)`: degree-2 ocean-tide corrections from
/// the 8 representative lines (`dC += Cp cos th + Sp sin th`, `dS += Sp cos th - Cp sin th`), over the design's
/// table of them.
pub use crate::gen::oceantides::main_lines;

/// The FES table `tbl` of `oceantides.fromModel` (the fields of
/// `force_data/fes2004_deg10.mat`: `doodson, n, m, Cp, Sp, Cm, Sm, note`): the design's table (env_ocean_tide_tables'
/// FES2004), as a view.
#[derive(Clone, Debug)]
pub struct FesTable {
    /// `tbl.doodson`, rows of multipliers `[tau s h p N' ps]`.
    pub doodson: Vec<[i8; 6]>,
    /// `tbl.n`, degree.
    pub n: Vec<u8>,
    /// `tbl.m`, order.
    pub m: Vec<u8>,
    /// `tbl.Cp` (SI, normalised).
    pub cp: Vec<f64>,
    /// `tbl.Sp`.
    pub sp: Vec<f64>,
    /// `tbl.Cm`.
    pub cm: Vec<f64>,
    /// `tbl.Sm`.
    pub sm: Vec<f64>,
    /// `tbl.note`.
    pub note: String,
}

impl FesTable {
    /// `force_data/fes2004_deg10.mat` (FES2004, degree <= 10, 1052 lines): the design's table.
    pub fn fes2004_deg10() -> FesTable {
        let t = &crate::gen::tidelines::DATA_FES2004;
        FesTable {
            doodson: t.iter().map(|r| [r[0] as i8, r[1] as i8, r[2] as i8, r[3] as i8, r[4] as i8, r[5] as i8]).collect(),
            n: t.iter().map(|r| r[6] as u8).collect(),
            m: t.iter().map(|r| r[7] as u8).collect(),
            cp: t.iter().map(|r| r[8]).collect(),
            sp: t.iter().map(|r| r[9]).collect(),
            cm: t.iter().map(|r| r[10]).collect(),
            sm: t.iter().map(|r| r[11]).collect(),
            note: "FES2004 ocean tide Stokes coeffs, degree<=10, scale applied (SI). Source: IERS/orekit-data.".into(),
        }
    }
}

/// A `dcs` of arbitrary size (`(nmax+1)^2`, row-major), as `oceantides.fromModel` returns.
#[derive(Clone, Debug, PartialEq)]
pub struct DcsN {
    /// nmax (the matrices are (nmax+1)x(nmax+1)).
    pub nmax: usize,
    /// `dcs.dC`, row-major.
    pub dc: Vec<f64>,
    /// `dcs.dS`, row-major.
    pub ds: Vec<f64>,
}

impl DcsN {
    /// `(dC(3,:), dS(3,:))`, the degree-2 rows used by `tideutil.accelFromDeg2`.
    pub fn deg2(&self) -> ([f64; 3], [f64; 3]) {
        let st = self.nmax + 1;
        let r = 2 * st;
        ([self.dc[r], self.dc[r + 1], self.dc[r + 2]], [self.ds[r], self.ds[r + 1], self.ds[r + 2]])
    }
}

/// Port of `oceantides.fromModel(jd_tt, tbl)` over the design's FES2004 table (`tbl` is its view, [`FesTable::fes2004_deg10`]):
/// full ocean-tide coefficient corrections (`th = doodson * beta`,
/// `dC += (Cp+Cm) cos th + (Sp+Sm) sin th`, `dS += (Sp-Sm) cos th - (Cp-Cm) sin th`).
pub fn from_model(jd_tt: f64, _tbl: &FesTable) -> DcsN {
    let nmax = crate::gen::oceantides::fes_nmax() as usize;
    let st = nmax + 1;
    let (dc, ds) = crate::gen::oceantides::from_model(jd_tt);
    DcsN { nmax, dc: dc[..st * st].to_vec(), ds: ds[..st * st].to_vec() }
}

/// Inputs of `forces.oceantides(ctx)`, named after the ctx fields they replace.
#[derive(Clone, Copy, Debug)]
pub struct OceanTideInputs {
    /// `ctx.T.tt_jd`, TT Julian date.
    pub tt_jd: f64,
    /// `ctx.r_ecef` = `ctx.C * r_eci` [m].
    pub r_ecef: V3,
    /// `ctx.C`, the ECI->ECEF rotation; `ctx.Ct = C'`.
    pub c_eci2ecef: M3,
    /// `ctx.grav.mu` [m^3/s^2].
    pub mu: f64,
    /// `ctx.grav.Re` [m].
    pub re: f64,
}

/// Port of `forces.oceantides(ctx)`: `mainLines(tt_jd)` degree-2 corrections,
/// `accelFromDeg2` in ECEF, rotated to ECI with `C'`. [m/s^2]
pub fn ocean_tides_accel(inp: &OceanTideInputs) -> V3 {
    crate::gen::oceantides::ocean_tides_accel(inp.tt_jd, inp.r_ecef, inp.c_eci2ecef, inp.mu, inp.re)
}
