//! IAU 2006/2000A CIO-based kernel shared by `frames.eci2ecef_A/B/C` -- port of
//! their local functions `fund_args`, `xy06`, `s06`, `sp00`, `c2ixys`, `era00`,
//! `pom00`, `rot`, `cal2jd`, `mjd_of`, `time_scales`, `get_tables` (the three
//! builds share this kernel verbatim; they differ only in EOP handling).
//!
//! The `xys06_tables.mat` series are exported by `refgen/time_frames_xys06_export.m`
//! to `data/time_frames/xys06.bin` and embedded with `include_bytes!`.
use crate::la::{mm, r1, r2, r3, M3};
use crate::time::omod;
use std::sync::OnceLock;

/// Arcseconds to radians (value used by the MATLAB kernel).
pub const AS2R: f64 = 4.848136811095359935899141e-6;
const TURNAS: f64 = 1296000.0;
/// MATLAB `D2PI=6.283185307179586476925287` (the same double as `TAU`).
const D2PI: f64 = std::f64::consts::TAU;

/// One X/Y series term of `xy_terms`. The argument `M(row,:)*fa` is shared by many
/// rows (3082 terms, 1309 distinct multiplier rows), so each distinct argument is
/// evaluated once (`arg` indexes `Tables::xy_args`); identical rows give the
/// identical double, so this does not change a single bit of the result.
struct XyTerm {
    /// 0 -> X, 1 -> Y (`jxy`)
    jxy: u8,
    /// power of t (`p`, 0..=5)
    p: usize,
    /// index into `Tables::xy_args`
    arg: usize,
    a_s: f64,
    a_c: f64,
}

/// Maximum number of distinct X/Y arguments (the table has 1309).
const MAX_XY_ARGS: usize = 1536;

/// One s-series term (8 multipliers on fa([1 2 3 4 5 7 8 14]) + sin/cos amplitudes).
struct STerm {
    m: Vec<(usize, f64)>,
    a_s: f64,
    a_c: f64,
}

/// The contents of `xys06_tables.mat` (`xyp`, `s_poly`, `xy_terms`, `s0..s4`).
pub struct Tables {
    xyp: [[f64; 6]; 2],
    s_poly: [f64; 6],
    xy: Vec<XyTerm>,
    /// distinct sparse argument rows: nonzero (index, multiplier) pairs, in column order
    xy_args: Vec<Vec<(usize, f64)>>,
    s: [Vec<STerm>; 5],
}

static XYS06_BIN: &[u8] = include_bytes!("../../data/time_frames/xys06.bin");

fn parse_tables(bytes: &[u8]) -> Tables {
    let v: Vec<f64> = bytes.chunks_exact(8).map(|c| f64::from_le_bytes(c.try_into().unwrap())).collect();
    let n_xy = v[0] as usize;
    let ns = [v[1] as usize, v[2] as usize, v[3] as usize, v[4] as usize, v[5] as usize];
    let mut k = 8;
    let mut xyp = [[0.0; 6]; 2];
    for r in 0..2 { for c in 0..6 { xyp[r][c] = v[k]; k += 1; } }
    let mut s_poly = [0.0; 6];
    for c in 0..6 { s_poly[c] = v[k]; k += 1; }
    let mut xy = Vec::with_capacity(n_xy);
    let mut xy_args: Vec<Vec<(usize, f64)>> = Vec::new();
    let mut seen: std::collections::HashMap<Vec<u64>, usize> = std::collections::HashMap::new();
    for _ in 0..n_xy {
        let row = &v[k..k + 18];
        k += 18;
        let key: Vec<u64> = row[2..16].iter().map(|x| x.to_bits()).collect();
        let arg = *seen.entry(key).or_insert_with(|| {
            xy_args.push((0..14).filter(|&i| row[2 + i] != 0.0).map(|i| (i, row[2 + i])).collect());
            xy_args.len() - 1
        });
        let p = row[1];
        assert!((0.0..=5.0).contains(&p) && p == p.trunc(), "xys06.bin: bad power {p}");
        xy.push(XyTerm { jxy: row[0] as u8, p: p as usize, arg, a_s: row[16], a_c: row[17] });
    }
    assert!(xy_args.len() <= MAX_XY_ARGS, "xys06.bin: {} distinct arguments", xy_args.len());
    let mut s: [Vec<STerm>; 5] = Default::default();
    for (b, &n) in ns.iter().enumerate() {
        for _ in 0..n {
            let row = &v[k..k + 10];
            k += 10;
            let m = (0..8).filter(|&i| row[i] != 0.0).map(|i| (i, row[i])).collect();
            s[b].push(STerm { m, a_s: row[8], a_c: row[9] });
        }
    }
    assert_eq!(k, v.len(), "xys06.bin: size mismatch");
    Tables { xyp, s_poly, xy, xy_args, s }
}

/// `get_tables()`: the embedded xys06 tables (parsed once, then shared).
pub fn tables() -> &'static Tables {
    static T: OnceLock<Tables> = OnceLock::new();
    T.get_or_init(|| parse_tables(XYS06_BIN))
}

/// `fund_args(t)`: the 14 IERS 2003 fundamental arguments (rad), t = TT centuries.
pub fn fund_args(t: f64) -> [f64; 14] {
    let mut fa = [0.0; 14];
    fa[0] = omod(485868.249036 + t * (1717915923.2178 + t * (31.8792 + t * (0.051635 + t * (-0.00024470)))), TURNAS) * AS2R;
    fa[1] = omod(1287104.793048 + t * (129596581.0481 + t * (-0.5532 + t * (0.000136 + t * (-0.00001149)))), TURNAS) * AS2R;
    fa[2] = omod(335779.526232 + t * (1739527262.8478 + t * (-12.7512 + t * (-0.001037 + t * (0.00000417)))), TURNAS) * AS2R;
    fa[3] = omod(1072260.703692 + t * (1602961601.2090 + t * (-6.3706 + t * (0.006593 + t * (-0.00003169)))), TURNAS) * AS2R;
    fa[4] = omod(450160.398036 + t * (-6962890.5431 + t * (7.4722 + t * (0.007702 + t * (-0.00005939)))), TURNAS) * AS2R;
    fa[5] = omod(4.402608842 + 2608.7903141574 * t, D2PI);
    fa[6] = omod(3.176146697 + 1021.3285546211 * t, D2PI);
    fa[7] = omod(1.753470314 + 628.3075849991 * t, D2PI);
    fa[8] = omod(6.203480913 + 334.0612426700 * t, D2PI);
    fa[9] = omod(0.599546497 + 52.9690962641 * t, D2PI);
    fa[10] = omod(0.874016757 + 21.3299104960 * t, D2PI);
    fa[11] = omod(5.481293872 + 7.4781598567 * t, D2PI);
    fa[12] = omod(5.311886287 + 3.8133035638 * t, D2PI);
    fa[13] = (0.024381750 + 0.00000538691 * t) * t;
    fa
}

/// `xy06(t, fa, tab)`: CIP X, Y (rad) from the IAU 2006/2000A series.
pub fn xy06(t: f64, fa: &[f64; 14]) -> (f64, f64) {
    let tab = tables();
    // pt = t.^(0:5), also the t.^p factor of every term (same pow calls as MATLAB)
    let mut pt = [0.0; 6];
    for j in 0..6 { pt[j] = t.powf(j as f64); }
    let mut sc = [(0.0f64, 0.0f64); MAX_XY_ARGS];
    for (a, m) in tab.xy_args.iter().enumerate() {
        let mut arg = 0.0;
        for &(i, c) in m.iter() { arg += c * fa[i]; }
        sc[a] = (arg.sin(), arg.cos());
    }
    let (mut sx, mut sy) = (0.0, 0.0);
    for term in tab.xy.iter() {
        let (sn, cs) = sc[term.arg];
        let comp = pt[term.p] * (term.a_s * sn + term.a_c * cs);
        if term.jxy == 0 { sx += comp; } else { sy += comp; }
    }
    let (mut px, mut py) = (0.0, 0.0);
    for j in 0..6 {
        px += tab.xyp[0][j] * pt[j];
        py += tab.xyp[1][j] * pt[j];
    }
    (AS2R * (px + sx / 1e6), AS2R * (py + sy / 1e6))
}

/// `s06(t, fa, x, y, tab)`: CIO locator s (rad), given X, Y.
pub fn s06(t: f64, fa: &[f64; 14], x: f64, y: f64) -> f64 {
    let tab = tables();
    let fa8 = [fa[0], fa[1], fa[2], fa[3], fa[4], fa[6], fa[7], fa[13]];
    let mut w = tab.s_poly;
    for k in 0..5 {
        if tab.s[k].is_empty() { continue; }
        let mut acc = 0.0;
        for term in tab.s[k].iter() {
            let mut arg = 0.0;
            for &(i, c) in term.m.iter() { arg += c * fa8[i]; }
            acc += term.a_s * arg.sin() + term.a_c * arg.cos();
        }
        w[k] += acc;
    }
    let poly = w[0] + (w[1] + (w[2] + (w[3] + (w[4] + w[5] * t) * t) * t) * t) * t;
    poly * AS2R - x * y / 2.0
}

/// `sp00(t)`: TIO locator s' (rad).
#[inline]
pub fn sp00(t: f64) -> f64 { -47e-6 * t * 4.848136811095359935899141e-6 }

/// `c2ixys(x, y, s)`: celestial-to-intermediate matrix from X, Y, s.
pub fn c2ixys(x: f64, y: f64, s: f64) -> M3 {
    let r2v = x * x + y * y;
    let e = if r2v > 0.0 { y.atan2(x) } else { 0.0 };
    let d = (r2v / (1.0 - r2v)).sqrt().atan();
    mm(&mm(&r3(-(e + s)), &r2(d)), &r3(e))
}

/// `era00(dj1, dj2)`: Earth rotation angle (rad) from a two-part UT1 JD.
pub fn era00(dj1: f64, dj2: f64) -> f64 {
    let (d1, d2) = if dj1 < dj2 { (dj1, dj2) } else { (dj2, dj1) };
    let t = d1 + (d2 - 2451545.0);
    let f = omod(d1, 1.0) + omod(d2, 1.0);
    let mut th = omod(D2PI * (f + 0.7790572732640 + 0.00273781191135448 * t), D2PI);
    if th < 0.0 { th += D2PI; }
    th
}

/// `pom00(xp, yp, sp)`: polar-motion matrix W = R1(-yp) R2(-xp) R3(sp).
pub fn pom00(xp: f64, yp: f64, sp: f64) -> M3 { mm(&mm(&r1(-yp), &r2(-xp)), &r3(sp)) }

/// `cal2jd(iy, im, id)` (SOFA iauCal2jd, local to the builds): `(djm0, djm)`.
#[inline]
pub fn cal2jd_sofa(iy: f64, im: f64, id: f64) -> (f64, f64) {
    let djm0 = 2400000.5;
    let my = ((im - 14.0) / 12.0).trunc();
    let iypmy = iy + my;
    let djm = ((1461.0 * (iypmy + 4800.0)) / 4.0).trunc() + ((367.0 * (im - 2.0 - 12.0 * my)) / 12.0).trunc()
        - ((3.0 * ((iypmy + 4900.0) / 100.0).trunc()) / 4.0).trunc() + id - 2432076.0;
    (djm0, djm)
}

/// `mjd_of(u)`: UTC MJD of a `[Y Mo D H Mi S]` vector.
#[inline]
pub fn mjd_of(u: &[f64; 6]) -> f64 {
    let (_, djm) = cal2jd_sofa(u[0], u[1], u[2]);
    djm + (u[3] * 3600.0 + u[4] * 60.0 + u[5]) / 86400.0
}

/// `time_scales(u, dUT1, dAT)` output.
#[derive(Clone, Copy, Debug)]
pub struct TimeScales {
    /// two-part UT1 JD
    pub jd_ut1: [f64; 2],
    /// two-part TT JD
    pub jd_tt: [f64; 2],
    /// TT Julian centuries since J2000
    pub t: f64,
}

/// `time_scales(u, dUT1, dAT)`: two-part UT1/TT JDs and TT centuries.
pub fn time_scales(u: &[f64; 6], dut1: f64, dat: f64) -> TimeScales {
    const DJ00: f64 = 2451545.0;
    const TTMTAI: f64 = 32.184;
    let (djm0, djm) = cal2jd_sofa(u[0], u[1], u[2]);
    let fd = (u[3] * 3600.0 + u[4] * 60.0 + u[5]) / 86400.0;
    let utc2 = djm + fd;
    let jd_ut1 = [djm0, utc2 + dut1 / 86400.0];
    let jd_tt = [djm0, utc2 + (dat + TTMTAI) / 86400.0];
    let t = ((jd_tt[0] - DJ00) + jd_tt[1]) / 36525.0;
    TimeScales { jd_ut1, jd_tt, t }
}

/// Everything the kernel computes for one epoch (the scalar `info` of the builds).
#[derive(Clone, Copy, Debug, Default)]
pub struct CioParts {
    /// CIP X including dX (rad)
    pub x: f64,
    /// CIP Y including dY (rad)
    pub y: f64,
    /// CIO locator s (rad)
    pub s: f64,
    /// Earth rotation angle (rad)
    pub era: f64,
    /// celestial-to-intermediate matrix Q
    pub q: M3,
    /// polar-motion matrix W
    pub w: M3,
}

/// The loop body of `eci2ecef_A/B/C` for one epoch, given its EOP:
/// `C = W * R3(era) * Q` (r_ecef = C r_eci). Angles in rad, dut1/dat in s.
pub fn cio_c2t(u: &[f64; 6], dut1: f64, xp: f64, yp: f64, dx: f64, dy: f64, dat: f64) -> (M3, CioParts) {
    let ts = time_scales(u, dut1, dat);
    let fa = fund_args(ts.t);
    let (x0, y0) = xy06(ts.t, &fa);
    let x = x0 + dx;
    let y = y0 + dy;
    let s = s06(ts.t, &fa, x, y);
    let q = c2ixys(x, y, s);
    let era = era00(ts.jd_ut1[0], ts.jd_ut1[1]);
    let w = pom00(xp, yp, sp00(ts.t));
    let c = mm(&mm(&w, &r3(era)), &q);
    (c, CioParts { x, y, s, era, q, w })
}
