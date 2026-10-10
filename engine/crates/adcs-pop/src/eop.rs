//! Earth-orientation parameters -- port of the EOP half of
//! `matlab_sils/pop/03_frames_time/+frames/eci2ecef_A.m / _B.m / _C.m`
//! (`get_eop`, `eop_paths`, `parse_leap`, `parse_finals`, `parse_c04`,
//! `read_c04_numeric`, `locate_mjd_col`, `splice`, `finalize_eop`, `eop_interp`,
//! `lagr`, `leap_at`). The readers of the three files are the code's; the splice, the
//! leap-second lookup and the interpolation of every build are env's method env_eop,
//! generated from the design into `gen::eop` (tools/engine_build.py, S7.19b), which the
//! functions below hand the files' columns to.
//!
//! POP caches three IERS files in one folder (`cfg.frame.data_dir`, wired by
//! `op.buildWorld` to `data.eop_dir()` = `<data.root>/eop`):
//! `Leap_Second.dat` (or IANA `leap-seconds.list` content), `finals2000A.all`
//! and `eopc04_20.1962-now`. MATLAB downloads them when missing; this port never
//! touches the network: [`Eop::from_dir`] reads what is there. With no EOP files
//! MATLAB's A/B/C builds stop with "could not obtain <file>"; the Rust builds
//! return [`crate::frames::FrameError::NoEop`] in that case (see `frames`). The
//! offline default of POP is the `gmst` build, which needs no EOP at all.
use crate::frames::{Build, FrameError};
use crate::gen::eop as gen;
use std::path::Path;

/// One EOP table after `finalize_eop` (per-row, daily; x/y/dX/dY in arcsec, UT1-UTC in s).
#[derive(Clone, Debug, Default)]
pub struct EopTable {
    /// `E.mjd` UTC MJD of each row (ascending).
    pub mjd: Vec<f64>,
    /// `E.xp` pole x (arcsec).
    pub xp: Vec<f64>,
    /// `E.yp` pole y (arcsec).
    pub yp: Vec<f64>,
    /// `E.dut1` UT1-UTC (s).
    pub dut1: Vec<f64>,
    /// `E.dX` celestial pole offset dX (arcsec).
    pub dx: Vec<f64>,
    /// `E.dY` celestial pole offset dY (arcsec).
    pub dy: Vec<f64>,
    /// `E.dut1_tai` = UT1-UTC - (TAI-UTC): leap-continuous UT1-TAI (s).
    pub dut1_tai: Vec<f64>,
    /// `E.source` description.
    pub source: String,
}

/// Loaded EOP: what `get_eop('A')` and `get_eop('B')` build from the cache folder.
#[derive(Clone, Debug, Default)]
pub struct Eop {
    /// `E.leap` = `parse_leap` rows `[MJD, TAI-UTC]`, sorted by MJD.
    pub leap: Vec<[f64; 2]>,
    /// build A table: finals2000A.all (Bulletin B where present, else A).
    pub finals: EopTable,
    /// build B/C table: EOP 20 C04 spliced with the finals tail (None if no C04 file).
    pub c04: Option<EopTable>,
}

/// Output of `eop_interp(mjd, E)`: `[dUT1, xp, yp, dX, dY, dAT, flag]`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EopAt {
    /// UT1-UTC (s)
    pub dut1: f64,
    /// pole x (rad)
    pub xp: f64,
    /// pole y (rad)
    pub yp: f64,
    /// dX (rad)
    pub dx: f64,
    /// dY (rad)
    pub dy: f64,
    /// TAI-UTC (s) from the leap file
    pub dat: f64,
    /// 1 if the MJD is outside the table (extrapolated; MATLAB warns), else 0
    pub flag: i32,
}

/// Options that change how the files are parsed (`opt.c04_native_dxdy`).
#[derive(Clone, Copy, Debug, Default)]
pub struct EopLoadOpt {
    /// `opt.c04_native_dxdy = [iX iY]`: 1-based C04 data columns to take dX,dY from.
    pub c04_native_dxdy: Option<[usize; 2]>,
}

/// File names of `eop_paths(opt)`.
pub const LEAP_FILE: &str = "Leap_Second.dat";
/// finals file name (`eop_paths`).
pub const FINALS_FILE: &str = "finals2000A.all";
/// C04 file name (`eop_paths`).
pub const C04_FILE: &str = "eopc04_20.1962-now";

impl Eop {
    /// `get_eop` without the download: read `Leap_Second.dat`, `finals2000A.all` and
    /// (if present) `eopc04_20.1962-now` from `dir` (MATLAB `opt.data_dir`).
    pub fn from_dir(dir: impl AsRef<Path>) -> Result<Eop, crate::PopError> {
        Self::from_dir_opt(dir, &EopLoadOpt::default())
    }

    /// [`Eop::from_dir`] with parse options.
    pub fn from_dir_opt(dir: impl AsRef<Path>, opt: &EopLoadOpt) -> Result<Eop, crate::PopError> {
        let d = dir.as_ref();
        let c04 = d.join(C04_FILE);
        Self::from_files(d.join(LEAP_FILE), d.join(FINALS_FILE), if c04.is_file() { Some(c04) } else { None }, opt)
    }

    /// Build from explicit files: `parse_leap`, `parse_finals`, and (for B/C)
    /// `parse_c04` + `splice`; each table then goes through `finalize_eop`.
    pub fn from_files(leap: impl AsRef<Path>, finals: impl AsRef<Path>, c04: Option<impl AsRef<Path>>, opt: &EopLoadOpt) -> Result<Eop, crate::PopError> {
        let rd = |p: &Path| std::fs::read(p).map(|b| String::from_utf8_lossy(&b).into_owned()).map_err(|e| crate::PopError::Data(format!("could not obtain {}: {e}", p.display())));
        let lp = leap.as_ref();
        let ls = parse_leap(&rd(lp)?, lp.to_string_lossy().contains("leap-seconds.list"));
        let fin = parse_finals(&rd(finals.as_ref())?);
        let c04t = match c04 {
            Some(p) => {
                let c = parse_c04(&rd(p.as_ref())?, opt)?;
                Some(finalize_eop(splice(&c, &fin), &ls))
            }
            None => None,
        };
        Ok(Eop { finals: finalize_eop(fin, &ls), c04: c04t, leap: ls })
    }

    /// `eop_interp(mjd, E)` of the given build: A = finals, linear (`interp1`
    /// 'linear','extrap'); B = C04+finals, linear; C = C04+finals, 4-point Lagrange
    /// with RG_ZONT2 regularisation and the sub-daily tidal model always on.
    /// `tidal` is `opt.tidal` of builds A/B (ignored by C, which always applies it).
    pub fn at(&self, mjd_utc: f64, build: Build, tidal: bool) -> Result<EopAt, FrameError> {
        match build {
            Build::A => Ok(interp_linear(&self.finals, &self.leap, mjd_utc, tidal)),
            Build::B => self.c04.as_ref().map(|e| interp_linear(e, &self.leap, mjd_utc, tidal)).ok_or(FrameError::NoC04),
            Build::C => self.c04.as_ref().map(|e| interp_c(e, &self.leap, mjd_utc)).ok_or(FrameError::NoC04),
            Build::Gmst => Err(FrameError::GmstHasNoEop),
        }
    }

    /// `leap_at(E.leap, mjd)`: TAI-UTC from the leap file at a UTC MJD.
    pub fn leap_at(&self, mjd: f64) -> f64 { leap_at(&self.leap, mjd) }
}

/// `leap_at(LS, mjd)`: last row with MJD <= mjd, else the first row's value (env's `eop::leap_at`).
pub fn leap_at(ls: &[[f64; 2]], mjd: f64) -> f64 {
    let (mut lm, mut ld) = leap_cols(ls);
    gen::leap_at(&mut lm, &mut ld, mjd)
}

/// MATLAB `sscanf(L,'%f')`: whitespace-separated numbers until the first that fails.
fn sscanf_f(line: &str) -> Vec<f64> {
    let mut v = Vec::new();
    for tok in line.split_whitespace() {
        match tok.parse::<f64>() {
            Ok(x) => v.push(x),
            Err(_) => {
                // %f reads a numeric prefix, then stops at the first non-numeric character
                if let Some(x) = numeric_prefix(tok) { v.push(x); }
                break;
            }
        }
    }
    v
}

fn numeric_prefix(tok: &str) -> Option<f64> {
    let b = tok.as_bytes();
    let mut end = 0;
    let mut best = None;
    while end < b.len() {
        end += 1;
        if let Ok(x) = tok[..end].parse::<f64>() { best = Some(x); }
    }
    best
}

/// `str2double` of a trimmed substring (NaN on failure / empty).
fn str2double(s: &str) -> f64 {
    let t = s.trim();
    if t.is_empty() { return f64::NAN; }
    t.parse::<f64>().unwrap_or(f64::NAN)
}

fn split_lines(txt: &str) -> impl Iterator<Item = &str> {
    txt.split('\n').map(|l| l.strip_suffix('\r').unwrap_or(l))
}

/// `parse_leap(file)`: IERS `Leap_Second.dat` (MJD .. TAI-UTC in column 5) or IANA
/// `leap-seconds.list` (NTP seconds, TAI-UTC); returns sorted `[MJD, TAI-UTC]`.
pub fn parse_leap(txt: &str, name_is_iana: bool) -> Vec<[f64; 2]> {
    let is_iana = txt.contains("#@") || txt.contains("#$") || name_is_iana;
    let mut rows: Vec<[f64; 2]> = Vec::new();
    for l in split_lines(txt) {
        let l = l.trim();
        if l.is_empty() || l.starts_with('#') { continue; }
        let v = sscanf_f(l);
        if is_iana {
            if v.len() >= 2 { rows.push([v[0] / 86400.0 + 15020.0, v[1]]); }
        } else if v.len() >= 5 {
            rows.push([v[0], v[4]]);
        }
    }
    rows.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap_or(std::cmp::Ordering::Equal));
    if rows.is_empty() {
        // MATLAB: warning('leap parse empty; default')
        rows = vec![[41317.0, 10.0], [57754.0, 37.0]];
    }
    rows
}

/// `parse_finals(file)`: IERS finals2000A.all fixed columns. Bulletin B x/y/UT1
/// (and dX,dY, falling back to A's) where present, else Bulletin A. dX,dY mas -> arcsec.
pub fn parse_finals(txt: &str) -> EopTable {
    let g = |s: &[u8], a: usize, b: usize| -> f64 {
        if s.len() >= b { str2double(&String::from_utf8_lossy(&s[a - 1..b])) } else { f64::NAN }
    };
    let pk = |a: f64, b: f64| if !a.is_nan() { a } else { b };
    let mut e = EopTable { source: "finals2000A.all".into(), ..Default::default() };
    let mut lines: Vec<&str> = txt.split('\n').collect();
    if lines.last().map(|l| l.is_empty()).unwrap_or(false) { lines.pop(); }
    for l in lines {
        let s = l.as_bytes();
        if s.len() < 68 { continue; }
        let mjd = g(s, 8, 15);
        let (xa, ya, ua, dxa, dya) = (g(s, 19, 27), g(s, 38, 46), g(s, 59, 68), g(s, 98, 106), g(s, 117, 125));
        let (xb, yb, ub, dxb, dyb) = (g(s, 135, 144), g(s, 145, 154), g(s, 155, 165), g(s, 166, 175), g(s, 176, 185));
        let (xp, yp, du, dx, dy) = if !xb.is_nan() && !ub.is_nan() { (xb, yb, ub, pk(dxb, dxa), pk(dyb, dya)) } else { (xa, ya, ua, dxa, dya) };
        if mjd.is_nan() || xp.is_nan() || du.is_nan() { continue; }
        e.mjd.push(mjd);
        e.xp.push(xp);
        e.yp.push(yp);
        e.dut1.push(du);
        e.dx.push(if dx.is_nan() { 0.0 } else { dx } / 1000.0);
        e.dy.push(if dy.is_nan() { 0.0 } else { dy } / 1000.0);
    }
    e
}

fn median(v: &mut [f64]) -> f64 {
    if v.is_empty() || v.iter().any(|x| x.is_nan()) { return f64::NAN; }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = v.len();
    if n % 2 == 1 { v[n / 2] } else { (v[n / 2 - 1] + v[n / 2]) / 2.0 }
}

/// `read_c04_numeric(file)`: numeric rows (of the modal length, >= 7 values) and
/// the last header line mentioning MJD|UT1|dX|Xcip.
fn read_c04_numeric(txt: &str) -> Result<(Vec<Vec<f64>>, String), crate::PopError> {
    let mut rows: Vec<Vec<f64>> = Vec::new();
    let mut hdr = String::new();
    for l in split_lines(txt) {
        let l = l.trim();
        if l.is_empty() { continue; }
        let c0 = l.as_bytes()[0];
        if c0 == b'#' || c0.is_ascii_alphabetic() {
            let u = l.to_ascii_uppercase();
            if u.contains("MJD") || u.contains("UT1") || u.contains("DX") || u.contains("XCIP") { hdr = l.to_string(); }
            continue;
        }
        let v = sscanf_f(l);
        if v.len() >= 7 { rows.push(v); }
    }
    if rows.is_empty() { return Err(crate::PopError::Data("parse_c04: no numeric rows".into())); }
    // mode of the row lengths (smallest on ties, as MATLAB/Octave mode)
    let mut lens: Vec<usize> = rows.iter().map(|r| r.len()).collect();
    lens.sort_unstable();
    let (mut best, mut bestn, mut i) = (lens[0], 0usize, 0usize);
    while i < lens.len() {
        let mut j = i;
        while j < lens.len() && lens[j] == lens[i] { j += 1; }
        if j - i > bestn { bestn = j - i; best = lens[i]; }
        i = j;
    }
    rows.retain(|r| r.len() == best);
    Ok((rows, hdr))
}

/// `locate_mjd_col(R)` (1-based; 0 if none).
fn locate_mjd_col(r: &[Vec<f64>]) -> usize {
    let nc = r[0].len();
    let in_range = |c: usize| r.iter().all(|row| row[c] >= 15000.0 && row[c] <= 99000.0);
    let (mut mc, mut best) = (0usize, f64::INFINITY);
    for c in 0..nc {
        if in_range(c) {
            let mut d: Vec<f64> = r.windows(2).map(|w| w[1][c] - w[0][c]).collect();
            let sc = (median(&mut d) - 1.0).abs();
            if sc < best && sc < 0.6 { best = sc; mc = c + 1; }
        }
    }
    if mc == 0 {
        for c in 0..nc { if in_range(c) { return c + 1; } }
    }
    mc
}

/// Parsed C04 (before splice): x, y, UT1-UTC and optionally its own dX, dY.
#[derive(Clone, Debug, Default)]
pub struct C04 {
    /// MJD
    pub mjd: Vec<f64>,
    /// x (arcsec)
    pub xp: Vec<f64>,
    /// y (arcsec)
    pub yp: Vec<f64>,
    /// UT1-UTC (s)
    pub dut1: Vec<f64>,
    /// dX (arcsec), when resolved
    pub dx: Option<Vec<f64>>,
    /// dY (arcsec), when resolved
    pub dy: Option<Vec<f64>>,
}

/// `parse_c04(file, opt)` of builds B/C (format-adaptive EOP 20 C04 reader; dX,dY
/// by header name, then positional sign-changing sub-mas pair, with mas/arcsec scaling).
pub fn parse_c04(txt: &str, opt: &EopLoadOpt) -> Result<C04, crate::PopError> {
    let (r, hdr) = read_c04_numeric(txt)?;
    let mc = locate_mjd_col(&r);
    if mc == 0 { return Err(crate::PopError::Data("parse_c04: could not locate an MJD column in C04 file".into())); }
    let nc = r[0].len();
    if nc < mc + 3 { return Err(crate::PopError::Data("parse_c04: C04 rows too short (need x,y,UT1 after MJD)".into())); }
    let col = |c: usize| -> Vec<f64> { r.iter().map(|row| row[c - 1]).collect() };
    let mut e = C04 { mjd: col(mc), xp: col(mc + 1), yp: col(mc + 2), dut1: col(mc + 3), dx: None, dy: None };
    let (mut ix, mut iy): (Option<i64>, Option<i64>) = (None, None);
    if let Some([a, b]) = opt.c04_native_dxdy {
        ix = Some(a as i64);
        iy = Some(b as i64);
    } else if !hdr.is_empty() {
        // tok=regexp(regexprep(hdr,'^[#\s]+',''),'\s+','split'); ht=lower(regexprep(tok,'[^a-z0-9]',''))
        let h = hdr.trim_start_matches(|c: char| c == '#' || c.is_whitespace());
        let ht: Vec<String> = h.split_whitespace()
            .map(|t| t.chars().filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit()).collect::<String>().to_ascii_lowercase())
            .collect();
        let hm = ht.iter().position(|t| t == "mjd");
        let hx = ht.iter().position(|t| ["dx", "dxcip", "xcip", "dx2000a"].contains(&t.as_str()));
        let hy = ht.iter().position(|t| ["dy", "dycip", "ycip", "dy2000a"].contains(&t.as_str()));
        if let (Some(hm), Some(hx), Some(hy)) = (hm, hx, hy) {
            let off = mc as i64 - (hm as i64 + 1);
            ix = Some(hx as i64 + 1 + off);
            iy = Some(hy as i64 + 1 + off);
        }
    }
    if ix.is_none() {
        let mut j = mc + 4;
        while j < nc {
            let a: Vec<f64> = col(j).into_iter().filter(|x| !x.is_nan()).collect();
            let b: Vec<f64> = col(j + 1).into_iter().filter(|x| !x.is_nan()).collect();
            if !a.is_empty() && !b.is_empty() {
                let mina = a.iter().cloned().fold(f64::INFINITY, f64::min);
                let minb = b.iter().cloned().fold(f64::INFINITY, f64::min);
                let mut aa: Vec<f64> = a.iter().map(|x| x.abs()).collect();
                let mut ab: Vec<f64> = b.iter().map(|x| x.abs()).collect();
                if median(&mut aa) < 5.0 && median(&mut ab) < 5.0 && mina < 0.0 && minb < 0.0 {
                    ix = Some(j as i64);
                    iy = Some(j as i64 + 1);
                    break;
                }
            }
            j += 1;
        }
    }
    if let (Some(ix), Some(iy)) = (ix, iy) {
        if ix >= 1 && iy >= 1 && (nc as i64) >= ix.max(iy) {
            let vx = col(ix as usize);
            let vy = col(iy as usize);
            let mut ax: Vec<f64> = vx.iter().filter(|x| !x.is_nan()).map(|x| x.abs()).collect();
            let mad = median(&mut ax);
            if mad < 5e-3 {
                e.dx = Some(vx);
                e.dy = Some(vy);
            } else if mad < 5.0 {
                e.dx = Some(vx.iter().map(|x| x / 1000.0).collect());
                e.dy = Some(vy.iter().map(|x| x / 1000.0).collect());
            }
        }
    }
    Ok(e)
}

/// Octave `interp1(x, y, xi, 'linear', extrap)` (x ascending): env's `eop::interp1_linear` (env_eop); outside [x1, xn]
/// either the end segment extrapolated (`None`) or the given value.
pub fn interp1_linear(x: &[f64], y: &[f64], xi: f64, extrap: Option<f64>) -> f64 {
    gen::interp1_linear(&mut x.to_vec(), &mut y.to_vec(), xi, extrap.is_some(), extrap.unwrap_or(0.0))
}

/// `splice(C04, FIN, opt)`: C04 x/y/UT1 over its span + finals rows beyond it; dX,dY from C04 if it carried them, else
/// finals interpolated (0 outside): env's `eop::splice` (env_eop) over the readers' columns.
pub fn splice(c04: &C04, fin: &EopTable) -> EopTable {
    let native = c04.dx.is_some() && c04.dy.is_some();
    let n = c04.mjd.len() + fin.mjd.len();
    let (mut cdx, mut cdy) = match (&c04.dx, &c04.dy) { (Some(x), Some(y)) => (x.clone(), y.clone()), _ => (vec![], vec![]) };
    let (mut om, mut ox, mut oy, mut ou, mut odx, mut ody) = (vec![0.0; n], vec![0.0; n], vec![0.0; n], vec![0.0; n], vec![0.0; n], vec![0.0; n]);
    let (mut ix, mut tmp, mut w) = (vec![0i64; n], vec![0i64; n], vec![0.0; n]);
    let (k, mmax) = gen::splice(&mut c04.mjd.clone(), &mut c04.xp.clone(), &mut c04.yp.clone(), &mut c04.dut1.clone(), &mut cdx, &mut cdy, native,
        &mut fin.mjd.clone(), &mut fin.xp.clone(), &mut fin.yp.clone(), &mut fin.dut1.clone(), &mut fin.dx.clone(), &mut fin.dy.clone(),
        &mut om, &mut ox, &mut oy, &mut ou, &mut odx, &mut ody, &mut ix, &mut tmp, &mut w);
    let k = k as usize;
    for v in [&mut om, &mut ox, &mut oy, &mut ou, &mut odx, &mut ody] { v.truncate(k); }
    EopTable { mjd: om, xp: ox, yp: oy, dut1: ou, dx: odx, dy: ody, dut1_tai: Vec::new(),
        source: format!("C04(<={:.0})+finals tail; dX,dY={}", mmax, if native { "C04-native" } else { "finals" }) }
}

/// The leap-second rows as env's methods take them: their MJDs, their TAI - UTC.
fn leap_cols(ls: &[[f64; 2]]) -> (Vec<f64>, Vec<f64>) {
    (ls.iter().map(|r| r[0]).collect(), ls.iter().map(|r| r[1]).collect())
}

/// `finalize_eop(E, LS, build)`: attach `dut1_tai = dut1 - leap_at(LS, mjd)` (env's `eop::ut1_tai`).
pub fn finalize_eop(mut e: EopTable, ls: &[[f64; 2]]) -> EopTable {
    let (mut lm, mut ld) = leap_cols(ls);
    let n = e.mjd.len().min(e.dut1.len());
    let mut out = vec![0.0; n];
    gen::ut1_tai(&mut e.mjd[..n].to_vec(), &mut e.dut1[..n].to_vec(), &mut lm, &mut ld, &mut out);
    e.dut1_tai = out;
    e
}

/// The table's columns as env's methods take them.
fn table_cols(e: &EopTable) -> [Vec<f64>; 6] {
    [e.mjd.clone(), e.xp.clone(), e.yp.clone(), e.dx.clone(), e.dy.clone(), e.dut1_tai.clone()]
}

fn eop_at(r: (f64, f64, f64, f64, f64, f64, i64)) -> EopAt {
    EopAt { dut1: r.0, xp: r.1, yp: r.2, dx: r.3, dy: r.4, dat: r.5, flag: r.6 as i32 }
}

/// `eop_interp` of builds A/B (linear interp1 with extrapolation, optional tidal): env's `eop::eop_linear`.
pub fn interp_linear(e: &EopTable, ls: &[[f64; 2]], mjd: f64, tidal: bool) -> EopAt {
    let [mut m, mut x, mut y, mut dx, mut dy, mut u] = table_cols(e);
    let (mut lm, mut ld) = leap_cols(ls);
    eop_at(gen::eop_linear(&mut m, &mut x, &mut y, &mut dx, &mut dy, &mut u, &mut lm, &mut ld, mjd, tidal))
}

/// `lagr(x, f, xi)`: N-point Lagrange interpolation (build C): env's `eop::lagr`.
pub fn lagr(x: &[f64], f: &[f64], xi: f64) -> f64 {
    gen::lagr(&mut x.to_vec(), &mut f.to_vec(), xi)
}

/// `eop_interp` of build C: 4-point Lagrange, RG_ZONT2-regularised UT1 and the sub-daily tidal model (ocean + PM
/// libration + UT1 libration): env's `eop::eop_lagrange`. (`E.zonal` is never set by eci2ecef_C, so the RG_ZONT2 branch
/// is the one MATLAB always takes.)
pub fn interp_c(e: &EopTable, ls: &[[f64; 2]], mjd: f64) -> EopAt {
    let [mut m, mut x, mut y, mut dx, mut dy, mut u] = table_cols(e);
    let (mut lm, mut ld) = leap_cols(ls);
    eop_at(gen::eop_lagrange(&mut m, &mut x, &mut y, &mut dx, &mut dy, &mut u, &mut lm, &mut ld, mjd))
}
