//! Ocean tides: ports of `matlab_sils/pop/02_forces/+oceantides/*` (`doodson`,
//! `mainLines`, `fromModel`) and `02_forces/+forces/oceantides.m`, with the FES2004
//! degree-10 table of `force_data/fes2004_deg10.mat` embedded
//! (`data/gravity_tides/fes2004_deg10.bin`, exported bit-exactly by
//! `refgen/gravity_tides_fes_export.m`).
//!
//! As in the MATLAB, `forces.oceantides` uses the 8 representative main lines
//! (`mainLines`), not the FES table; `fromModel` is the "advanced" path that the
//! MATLAB provides but does not wire into the force sum.
//!
//! ctx mapping (`op.accel`): `ctx.T.tt_jd` from `timeconv.convertUTC`,
//! `ctx.r_ecef = C*r_eci`, `ctx.C` from `frames.eci2ecef`, `ctx.grav.mu / Re`.
//!
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use crate::la::{mtv, M3, V3};
use crate::solidtides::{accel_from_deg2, Dcs5};

/// MATLAB/Octave `mod(x, y)` for finite doubles (`x - floor(x/y)*y`, result with
/// the sign of `y`; Octave's near-integer-quotient rule for non-integer `y`).
pub fn octave_mod(x: f64, y: f64) -> f64 {
    if y == 0.0 {
        return x;
    }
    let q = x / y;
    let nint = |v: f64| (v + 0.5).floor();
    let mut r = if nint(y) != y && ((q - nint(q)) / nint(q)).abs() < f64::EPSILON {
        0.0
    } else {
        let n = q.floor();
        x - y * n
    };
    if x != y && r != 0.0 && ((r < 0.0) != (y < 0.0)) {
        r += y;
    }
    r
}

/// Port of `oceantides.doodson(jd_tt)`: the six Doodson fundamental arguments
/// `[tau, s, h, p, N', ps]` [rad] at a TT Julian date.
pub fn doodson(jd_tt: f64) -> [f64; 6] {
    let t = (jd_tt - 2451545.0) / 36525.0;
    let d2r = std::f64::consts::PI / 180.0;
    let s = octave_mod(218.3164477 + 481267.88123421 * t, 360.0);
    let h = octave_mod(280.4664567 + 36000.76982779 * t, 360.0);
    let p = octave_mod(83.3532465 + 4069.0137287 * t, 360.0);
    let np = octave_mod(234.9554736 + 1934.1362608 * t, 360.0);
    let ps = octave_mod(282.9373409 + 1.7195366 * t, 360.0);
    let gmst = octave_mod(280.46061837 + 360.98564736629 * (jd_tt - 2451545.0), 360.0);
    let tau = octave_mod(gmst + 180.0 - s, 360.0);
    [tau * d2r, s * d2r, h * d2r, p * d2r, np * d2r, ps * d2r]
}

/// The 8 constituents of `oceantides.mainLines`: Doodson multipliers, degree,
/// order, `Cplus`, `Splus` (units of 1e-11): M2 S2 N2 K2 K1 O1 P1 Q1.
pub const MAIN_LINES: [([f64; 6], usize, usize, f64, f64); 8] = [
    ([2.0, 0.0, 0.0, 0.0, 0.0, 0.0], 2, 2, -3.10, 0.40),
    ([2.0, 2.0, -2.0, 0.0, 0.0, 0.0], 2, 2, -1.50, 0.20),
    ([2.0, -1.0, 0.0, 1.0, 0.0, 0.0], 2, 2, -0.60, 0.10),
    ([2.0, 2.0, 0.0, 0.0, 0.0, 0.0], 2, 2, -0.40, 0.05),
    ([1.0, 1.0, 0.0, 0.0, 0.0, 0.0], 2, 1, 1.40, -0.30),
    ([1.0, -1.0, 0.0, 0.0, 0.0, 0.0], 2, 1, 1.00, -0.20),
    ([1.0, 1.0, -2.0, 0.0, 0.0, 0.0], 2, 1, 0.45, -0.10),
    ([1.0, -2.0, 0.0, 1.0, 0.0, 0.0], 2, 1, 0.20, -0.05),
];

/// Port of `oceantides.mainLines(jd_tt)`: degree-2 ocean-tide corrections from
/// the 8 representative lines (`dC += Cp cos th + Sp sin th`, `dS += Sp cos th - Cp sin th`).
pub fn main_lines(jd_tt: f64) -> Dcs5 {
    let beta = doodson(jd_tt);
    let mut d = Dcs5::default();
    for (n, deg, m, cp, sp) in MAIN_LINES.iter() {
        let cp = cp * 1e-11;
        let sp = sp * 1e-11;
        let mut th = 0.0;
        for k in 0..6 {
            th += n[k] * beta[k];
        }
        let (st, ct) = (th.sin(), th.cos());
        d.dc[*deg][*m] += cp * ct + sp * st;
        d.ds[*deg][*m] += sp * ct - cp * st;
    }
    d
}

/// The FES table `tbl` of `oceantides.fromModel` (the fields of
/// `force_data/fes2004_deg10.mat`: `doodson, n, m, Cp, Sp, Cm, Sm, note`).
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

static FES2004_BIN: &[u8] = include_bytes!("../data/gravity_tides/fes2004_deg10.bin");

impl FesTable {
    /// The embedded `force_data/fes2004_deg10.mat` (FES2004, degree <= 10, 1052 lines).
    pub fn fes2004_deg10() -> FesTable {
        Self::parse(FES2004_BIN).expect("embedded fes2004_deg10.bin is valid")
    }

    /// Parse the `POPFES01` binary written by `refgen/gravity_tides_fes_export.m`.
    pub fn parse(b: &[u8]) -> Result<FesTable, crate::PopError> {
        let bad = || crate::PopError::Data("fes table: truncated or malformed".to_string());
        if b.len() < 16 || &b[..8] != b"POPFES01" {
            return Err(crate::PopError::Data("fes table: bad magic".into()));
        }
        let u32le = |o: usize| u32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]]) as usize;
        let n = u32le(8);
        let l = u32le(12);
        let mut o = 16;
        let need = o + l + n * 6 + 2 * n + 4 * 8 * n;
        if b.len() != need {
            return Err(bad());
        }
        let note = String::from_utf8_lossy(&b[o..o + l]).into_owned();
        o += l;
        let doodson = (0..n).map(|i| {
            let mut r = [0i8; 6];
            for j in 0..6 {
                r[j] = b[o + 6 * i + j] as i8;
            }
            r
        }).collect();
        o += 6 * n;
        let nn = b[o..o + n].to_vec();
        o += n;
        let mm = b[o..o + n].to_vec();
        o += n;
        let mut col = || {
            let v: Vec<f64> = (0..n).map(|i| {
                let s = o + 8 * i;
                f64::from_le_bytes(b[s..s + 8].try_into().unwrap())
            }).collect();
            o += 8 * n;
            v
        };
        let cp = col();
        let sp = col();
        let cm = col();
        let sm = col();
        Ok(FesTable { doodson, n: nn, m: mm, cp, sp, cm, sm, note })
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

/// Port of `oceantides.fromModel(jd_tt, tbl)`: full ocean-tide coefficient
/// corrections from a model table (`th = doodson * beta`,
/// `dC += (Cp+Cm) cos th + (Sp+Sm) sin th`, `dS += (Sp-Sm) cos th - (Cp-Cm) sin th`).
/// (The MATLAB needs `tbl.doodson` as double; the `.mat` holds int64.)
pub fn from_model(jd_tt: f64, tbl: &FesTable) -> DcsN {
    let beta = doodson(jd_tt);
    let nmax = tbl.n.iter().copied().max().unwrap_or(0) as usize;
    let st = nmax + 1;
    let mut dc = vec![0.0; st * st];
    let mut ds = vec![0.0; st * st];
    for i in 0..tbl.n.len() {
        let dd = &tbl.doodson[i];
        let mut th = 0.0;
        for j in 0..6 {
            th += dd[j] as f64 * beta[j];
        }
        let (stn, ctn) = (th.sin(), th.cos());
        let k = tbl.n[i] as usize * st + tbl.m[i] as usize;
        dc[k] = dc[k] + (tbl.cp[i] + tbl.cm[i]) * ctn + (tbl.sp[i] + tbl.sm[i]) * stn;
        ds[k] = ds[k] + (tbl.sp[i] - tbl.sm[i]) * ctn - (tbl.cp[i] - tbl.cm[i]) * stn;
    }
    DcsN { nmax, dc, ds }
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
    let d = main_lines(inp.tt_jd);
    let (dc2, ds2) = d.deg2();
    let a = accel_from_deg2(&inp.r_ecef, &dc2, &ds2, Some(inp.mu), Some(inp.re));
    mtv(&inp.c_eci2ecef, &a)
}
