//! Jacchia-Bowman 2008: `density_models/jb2008/JB2008.m` (the Mahooti core, CIRA
//! integration form), `jb2008_density.m` (lagged SET indices, Sun position, GMST),
//! `finddays.m`, `invjday.m`, `sign_.m`, the SET file parsers of
//! `data_sources/spaceweather/model_inputs/get_jb2008_indices.m`, and the adapter
//! `+atmos/jb2008.m`.
//!
//! Times are MATLAB `datenum` days (`datenum(datetime)` in the MATLAB wrapper).
use super::octave::{datenum, datenum_v, deg2rad, interp1_linear_extrap, mod_, pw, rem_};
use std::f64::consts::PI;

/// The SET index tables (`idx` of `get_jb2008_indices`): daily `SOLFSMY.TXT`
/// (F10 F81 S10 S81 M10 M81 Y10 Y81) and hourly `DTCFILE.TXT` (DSTDTC [K]).
#[derive(Clone, Debug, Default)]
pub struct JbIndices {
    /// daily epochs [datenum] of the solar indices (sorted)
    pub sol_t: Vec<f64>,
    /// columns F10 F81 S10 S81 M10 M81 Y10 Y81, one row per `sol_t`
    pub sol: Vec<[f64; 8]>,
    /// hourly epochs [datenum] of DSTDTC (sorted)
    pub dtc_t: Vec<f64>,
    /// DSTDTC [K]
    pub dtc: Vec<f64>,
    // column-major copies for the interpolation (built by `finish`)
    cols: Vec<Vec<f64>>,
}

/// Parse error of the SET files.
#[derive(Clone, Debug, PartialEq)]
pub enum JbParseError {
    /// no SOLFSMY data line parsed (`parse_solfsmy:empty`)
    EmptySolfsmy,
    /// no DTC line parsed (`parse_dtcfile:empty`)
    EmptyDtc,
}

/// `sscanf(L, '%f')`: the leading run of numbers, stopping at the first token that is
/// not one (a token like `1B11` contributes its numeric prefix `1` and stops the scan).
fn scan_numbers(l: &str, out: &mut Vec<f64>) {
    out.clear();
    for tok in l.split_whitespace() {
        if let Ok(v) = tok.parse::<f64>() {
            out.push(v);
            continue;
        }
        // numeric prefix, then stop
        let b = tok.as_bytes();
        let mut end = 0;
        for k in (1..=b.len()).rev() {
            if tok.is_char_boundary(k) && tok[..k].parse::<f64>().is_ok() {
                end = k;
                break;
            }
        }
        if end > 0 {
            out.push(tok[..end].parse::<f64>().unwrap_or(f64::NAN));
        }
        break;
    }
}

impl JbIndices {
    /// `parse_solfsmy` + `parse_dtcfile` of `get_jb2008_indices` on the two file texts.
    pub fn parse(solfsmy: &str, dtcfile: &str) -> Result<JbIndices, JbParseError> {
        let mut rows: Vec<(f64, [f64; 8])> = Vec::new();
        let mut t = Vec::with_capacity(16);
        for line in solfsmy.lines() {
            let l = line.trim();
            if l.is_empty() || l.starts_with('#') || l.starts_with('%') {
                continue;
            }
            scan_numbers(&l.replace('\t', " "), &mut t);
            if t.len() >= 11 && t[0] > 1900.0 && t[0] < 2100.0 {
                let dn = datenum(t[0], 1.0, 1.0, 0.0, 0.0, 0.0) + t[1] - 1.0;
                rows.push((dn, [t[3], t[4], t[5], t[6], t[7], t[8], t[9], t[10]]));
            }
        }
        if rows.is_empty() {
            return Err(JbParseError::EmptySolfsmy);
        }
        rows.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        let mut dt: Vec<(f64, f64)> = Vec::new();
        for line in dtcfile.lines() {
            let l = line.trim();
            if l.len() < 3 || !l[..3].eq_ignore_ascii_case("DTC") {
                continue;
            }
            scan_numbers(&l[3..].replace('\t', " "), &mut t);
            if t.len() >= 26 {
                let base = datenum(t[0], 1.0, 1.0, 0.0, 0.0, 0.0) + t[1] - 1.0;
                for h in 0..24 {
                    dt.push((base + h as f64 / 24.0, t[2 + h]));
                }
            }
        }
        if dt.is_empty() {
            return Err(JbParseError::EmptyDtc);
        }
        dt.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        let mut idx = JbIndices {
            sol_t: rows.iter().map(|r| r.0).collect(),
            sol: rows.iter().map(|r| r.1).collect(),
            dtc_t: dt.iter().map(|r| r.0).collect(),
            dtc: dt.iter().map(|r| r.1).collect(),
            cols: Vec::new(),
        };
        idx.finish();
        Ok(idx)
    }

    /// Build the per-column copies used by [`jb2008_density`] (call after filling the
    /// public fields by hand).
    pub fn finish(&mut self) {
        self.cols = (0..8).map(|j| self.sol.iter().map(|r| r[j]).collect()).collect();
    }

    /// The copies of SOLFSMY.TXT / DTCFILE.TXT bundled with the MATLAB toolbox
    /// (`04_atmosphere/density_models/jb2008`, release 8_1_0, to 2026-137), which
    /// `data.jb2008_indices` falls back to offline. Parsed on each call.
    pub fn bundled() -> JbIndices {
        JbIndices::parse(include_str!("../../data/atmos_drag/SOLFSMY.TXT"), include_str!("../../data/atmos_drag/DTCFILE.TXT"))
            .expect("bundled SET files parse")
    }

    fn col(&self, j: usize) -> &[f64] {
        &self.cols[j]
    }
}

// ---------------------------------------------------------------------------------
// JB2008 core
// ---------------------------------------------------------------------------------

/// Output of the JB2008 core: `TEMP(1)` exospheric temperature above the point [K],
/// `TEMP(2)` temperature at the point [K], `RHO` total mass density [kg/m^3].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct JbOut {
    /// `TEMP` = [Tinf, T(z)] [K]
    pub temp: [f64; 2],
    /// density [kg/m^3]
    pub rho: f64,
}

/// `sign_(a, b)`: `abs(a)` with the sign of `b` (`b >= 0` -> positive).
pub fn sign_(a: f64, b: f64) -> f64 {
    if b >= 0.0 { a.abs() } else { -a.abs() }
}

fn xambar(z: f64) -> f64 {
    let c = [28.15204, -8.5586e-2, 1.2840e-4, -1.0056e-5, -1.0210e-5, 1.5044e-6, 9.9826e-8];
    let dz = z - 100.0;
    let mut amb = c[6];
    for i in 1..=6 {
        let j = 7 - i;
        amb = dz * amb + c[j - 1];
    }
    amb
}

fn xgrav(z: f64) -> f64 {
    9.80665 / pw(1.0 + z / 6356.766, 2.0)
}

fn xlocal(z: f64, tc: &[f64; 4]) -> f64 {
    let dz = z - 125.0;
    if dz > 0.0 {
        return tc[0] + tc[2] * (tc[3] * dz * (1.0 + 4.5e-6 * pw(dz, 2.5))).atan();
    }
    ((-9.8204695e-6 * dz - 7.3039742e-4) * pw(dz, 2.0) + 1.0) * dz * tc[1] + tc[0]
}

/// `DTSUB`: dTc correction for local solar time and latitude.
fn dtsub(f10: f64, xlst: f64, xlat: f64, zht: f64) -> f64 {
    const B: [f64; 19] = [
        -0.457512297e1, -0.512114909e1, -0.693003609e2, 0.203716701e3, 0.703316291e3, -0.194349234e4, 0.110651308e4,
        -0.174378996e3, 0.188594601e4, -0.709371517e4, 0.922454523e4, -0.384508073e4, -0.645841789e1, 0.409703319e2,
        -0.482006560e3, 0.181870931e4, -0.237389204e4, 0.996703815e3, 0.361416936e2,
    ];
    const C: [f64; 23] = [
        -0.155986211e2, -0.512114909e1, -0.693003609e2, 0.203716701e3, 0.703316291e3, -0.194349234e4, 0.110651308e4,
        -0.220835117e3, 0.143256989e4, -0.318481844e4, 0.328981513e4, -0.135332119e4, 0.199956489e2, -0.127093998e2,
        0.212825156e2, -0.275555432e1, 0.110234982e2, 0.148881951e3, -0.751640284e3, 0.637876542e3, 0.127093998e2,
        -0.212825156e2, 0.275555432e1,
    ];
    let b = |i: usize| B[i - 1];
    let c = |i: usize| C[i - 1];
    let mut dtc = 0.0;
    let tx = xlst / 24.0;
    let ycs = xlat.cos();
    let f = (f10 - 100.0) / 100.0;
    let p = |x: f64, n: f64| pw(x, n);
    if (120.0..=200.0).contains(&zht) {
        let dtc200 = c(17) + c(18) * tx * ycs + c(19) * p(tx, 2.0) * ycs
            + c(20) * p(tx, 3.0) * ycs + c(21) * f * ycs + c(22) * tx * f * ycs
            + c(23) * p(tx, 2.0) * f * ycs;
        let sum = c(1) + b(2) * f + c(3) * tx * f + c(4) * p(tx, 2.0) * f
            + c(5) * p(tx, 3.0) * f + c(6) * p(tx, 4.0) * f + c(7) * p(tx, 5.0) * f
            + c(8) * tx * ycs + c(9) * p(tx, 2.0) * ycs + c(10) * p(tx, 3.0) * ycs
            + c(11) * p(tx, 4.0) * ycs + c(12) * p(tx, 5.0) * ycs + c(13) * ycs
            + c(14) * f * ycs + c(15) * tx * f * ycs + c(16) * p(tx, 2.0) * f * ycs;
        let dtc200dz = sum;
        let cc = 3.0 * dtc200 - dtc200dz;
        let dd = dtc200 - cc;
        let zp = (zht - 120.0) / 80.0;
        dtc = cc * zp * zp + dd * zp * zp * zp;
    }
    if zht > 200.0 && zht <= 240.0 {
        let h = (zht - 200.0) / 50.0;
        let sum = c(1) * h + b(2) * f * h + c(3) * tx * f * h + c(4) * p(tx, 2.0) * f * h
            + c(5) * p(tx, 3.0) * f * h + c(6) * p(tx, 4.0) * f * h + c(7) * p(tx, 5.0) * f * h
            + c(8) * tx * ycs * h + c(9) * p(tx, 2.0) * ycs * h + c(10) * p(tx, 3.0) * ycs * h
            + c(11) * p(tx, 4.0) * ycs * h + c(12) * p(tx, 5.0) * ycs * h + c(13) * ycs * h
            + c(14) * f * ycs * h + c(15) * tx * f * ycs * h + c(16) * p(tx, 2.0) * f * ycs * h
            + c(17) + c(18) * tx * ycs + c(19) * p(tx, 2.0) * ycs
            + c(20) * p(tx, 3.0) * ycs + c(21) * f * ycs + c(22) * tx * f * ycs
            + c(23) * p(tx, 2.0) * f * ycs;
        dtc = sum;
    }
    if zht > 240.0 && zht <= 300.0 {
        let h = 40.0 / 50.0;
        let sum = c(1) * h + b(2) * f * h + c(3) * tx * f * h + c(4) * p(tx, 2.0) * f * h
            + c(5) * p(tx, 3.0) * f * h + c(6) * p(tx, 4.0) * f * h + c(7) * p(tx, 5.0) * f * h
            + c(8) * tx * ycs * h + c(9) * p(tx, 2.0) * ycs * h + c(10) * p(tx, 3.0) * ycs * h
            + c(11) * p(tx, 4.0) * ycs * h + c(12) * p(tx, 5.0) * ycs * h + c(13) * ycs * h
            + c(14) * f * ycs * h + c(15) * tx * f * ycs * h + c(16) * p(tx, 2.0) * f * ycs * h
            + c(17) + c(18) * tx * ycs + c(19) * p(tx, 2.0) * ycs
            + c(20) * p(tx, 3.0) * ycs + c(21) * f * ycs + c(22) * tx * f * ycs
            + c(23) * p(tx, 2.0) * f * ycs;
        let aa = sum;
        let bb = c(1) + b(2) * f + c(3) * tx * f + c(4) * p(tx, 2.0) * f
            + c(5) * p(tx, 3.0) * f + c(6) * p(tx, 4.0) * f + c(7) * p(tx, 5.0) * f
            + c(8) * tx * ycs + c(9) * p(tx, 2.0) * ycs + c(10) * p(tx, 3.0) * ycs
            + c(11) * p(tx, 4.0) * ycs + c(12) * p(tx, 5.0) * ycs + c(13) * ycs
            + c(14) * f * ycs + c(15) * tx * f * ycs + c(16) * p(tx, 2.0) * f * ycs;
        let h = 300.0 / 100.0;
        let sum = b(1) + b(2) * f + b(3) * tx * f + b(4) * p(tx, 2.0) * f
            + b(5) * p(tx, 3.0) * f + b(6) * p(tx, 4.0) * f + b(7) * p(tx, 5.0) * f
            + b(8) * tx * ycs + b(9) * p(tx, 2.0) * ycs + b(10) * p(tx, 3.0) * ycs
            + b(11) * p(tx, 4.0) * ycs + b(12) * p(tx, 5.0) * ycs + b(13) * h * ycs
            + b(14) * tx * h * ycs + b(15) * p(tx, 2.0) * h * ycs + b(16) * p(tx, 3.0) * h * ycs
            + b(17) * p(tx, 4.0) * h * ycs + b(18) * p(tx, 5.0) * h * ycs + b(19) * ycs;
        let dtc300 = sum;
        let sum = b(13) * ycs
            + b(14) * tx * ycs + b(15) * p(tx, 2.0) * ycs + b(16) * p(tx, 3.0) * ycs
            + b(17) * p(tx, 4.0) * ycs + b(18) * p(tx, 5.0) * ycs;
        let dtc300dz = sum;
        let cc = 3.0 * dtc300 - dtc300dz - 3.0 * aa - 2.0 * bb;
        let dd = dtc300 - aa - bb - cc;
        let zp = (zht - 240.0) / 60.0;
        dtc = aa + bb * zp + cc * zp * zp + dd * zp * zp * zp;
    }
    if zht > 300.0 && zht <= 600.0 {
        let h = zht / 100.0;
        let sum = b(1) + b(2) * f + b(3) * tx * f + b(4) * p(tx, 2.0) * f
            + b(5) * p(tx, 3.0) * f + b(6) * p(tx, 4.0) * f + b(7) * p(tx, 5.0) * f
            + b(8) * tx * ycs + b(9) * p(tx, 2.0) * ycs + b(10) * p(tx, 3.0) * ycs
            + b(11) * p(tx, 4.0) * ycs + b(12) * p(tx, 5.0) * ycs + b(13) * h * ycs
            + b(14) * tx * h * ycs + b(15) * p(tx, 2.0) * h * ycs + b(16) * p(tx, 3.0) * h * ycs
            + b(17) * p(tx, 4.0) * h * ycs + b(18) * p(tx, 5.0) * h * ycs + b(19) * ycs;
        dtc = sum;
    }
    if zht > 600.0 && zht <= 800.0 {
        let zp = (zht - 600.0) / 100.0;
        let hp = 600.0 / 100.0;
        let aa = b(1) + b(2) * f + b(3) * tx * f + b(4) * p(tx, 2.0) * f
            + b(5) * p(tx, 3.0) * f + b(6) * p(tx, 4.0) * f + b(7) * p(tx, 5.0) * f
            + b(8) * tx * ycs + b(9) * p(tx, 2.0) * ycs + b(10) * p(tx, 3.0) * ycs
            + b(11) * p(tx, 4.0) * ycs + b(12) * p(tx, 5.0) * ycs + b(13) * hp * ycs
            + b(14) * tx * hp * ycs + b(15) * p(tx, 2.0) * hp * ycs + b(16) * p(tx, 3.0) * hp * ycs
            + b(17) * p(tx, 4.0) * hp * ycs + b(18) * p(tx, 5.0) * hp * ycs + b(19) * ycs;
        let bb = b(13) * ycs
            + b(14) * tx * ycs + b(15) * p(tx, 2.0) * ycs + b(16) * p(tx, 3.0) * ycs
            + b(17) * p(tx, 4.0) * ycs + b(18) * p(tx, 5.0) * ycs;
        let cc = -(3.0 * aa + 4.0 * bb) / 4.0;
        let dd = (aa + bb) / 4.0;
        dtc = aa + bb * zp + cc * zp * zp + dd * zp * zp * zp;
    }
    dtc
}

/// `SEMIAN08`: semiannual variation -> (FZZ, GTZ, DRLOG).
fn semian08(day: f64, ht: f64, f10b: f64, s10b: f64, xm10b: f64) -> (f64, f64, f64) {
    let twopi = 2.0 * PI;
    let fzm = [0.2689, -0.1176e-1, 0.2782e-1, -0.2782e-1, 0.3470e-3];
    let gtm = [-0.3633, 0.8506e-1, 0.2401, -0.1897, -0.2554, -0.1790e-1, 0.5650e-3, -0.6407e-3, -0.3418e-2, -0.1252e-2];
    let fsmb = f10b - 0.70 * s10b - 0.04 * xm10b;
    let htz = ht / 1000.0;
    let mut fzz = fzm[0] + fzm[1] * fsmb + fzm[2] * fsmb * htz + fzm[3] * fsmb * pw(htz, 2.0) + fzm[4] * pw(fsmb, 2.0) * htz;
    let fsmb = f10b - 0.75 * s10b - 0.37 * xm10b;
    let tau = (day - 1.0) / 365.0;
    let sin1p = (twopi * tau).sin();
    let cos1p = (twopi * tau).cos();
    let sin2p = (2.0 * twopi * tau).sin();
    let cos2p = (2.0 * twopi * tau).cos();
    let gtz = gtm[0] + gtm[1] * sin1p + gtm[2] * cos1p + gtm[3] * sin2p + gtm[4] * cos2p
        + gtm[5] * fsmb
        + gtm[6] * fsmb * sin1p + gtm[7] * fsmb * cos1p
        + gtm[8] * fsmb * sin2p + gtm[9] * fsmb * cos2p;
    if fzz < 1e-6 {
        fzz = 1e-6;
    }
    (fzz, gtz, fzz * gtz)
}

/// `invjday(Mjd)` (Montenbruck & Gill): calendar date from a modified Julian date.
pub fn invjday(mjd: f64) -> (f64, f64, f64, f64, f64, f64) {
    let a = (mjd + 2400001.0).trunc();
    let c = if a < 2299161.0 {
        a + 1524.0
    } else {
        let b = ((a - 1867216.25) / 36524.25).trunc();
        a + b - (b / 4.0).trunc() + 1525.0
    };
    let d = ((c - 122.1) / 365.25).trunc();
    let e = 365.0 * d + (d / 4.0).trunc();
    let f = ((c - e) / 30.6001).trunc();
    let day = c - e - (30.6001 * f).trunc();
    let month = f - 1.0 - 12.0 * (f / 14.0).trunc();
    let year = d - 4715.0 - ((7.0 + month) / 10.0).trunc();
    let hours = 24.0 * (mjd - mjd.floor());
    let hour = hours.trunc();
    let x = (hours - hour) * 60.0;
    let minute = x.trunc();
    let sec = (x - minute) * 60.0;
    (year, month, day, hour, minute, sec)
}

/// `finddays(year, month, day, hr, min, sec)` (Vallado): fractional day of year.
pub fn finddays(year: f64, month: f64, day: f64, hr: f64, min: f64, sec: f64) -> f64 {
    let mut lmonth = [31.0f64; 12];
    lmonth[1] = 28.0;
    for i in [4usize, 6, 9, 11] {
        lmonth[i - 1] = 30.0;
    }
    if rem_(year, 4.0) == 0.0 {
        lmonth[1] = 29.0;
        if rem_(year, 100.0) == 0.0 && rem_(year, 400.0) != 0.0 {
            lmonth[1] = 28.0;
        }
    }
    let mut i = 1.0;
    let mut days = 0.0;
    while i < month && i < 12.0 {
        days += lmonth[i as usize - 1];
        i += 1.0;
    }
    days + day + hr / 24.0 + min / 1440.0 + sec / 86400.0
}

/// `TMOUTD(MJD)` = `finddays(invjday(MJD + 2400000.5))`. NOTE (faithful port): the
/// MATLAB passes a JULIAN date to `invjday`, which expects an MJD, so the calendar
/// date used for the semiannual phase is offset by 2400000.5 days, exactly as in MATLAB.
fn tmoutd(mjd: f64) -> f64 {
    let (y, mo, d, h, mi, s) = invjday(mjd + 2400000.5);
    finddays(y, mo, d, h, mi, s)
}

/// `JB2008(MJD, SUN, SAT, F10, F10B, S10, S10B, XM10, XM10B, Y10, Y10B, DSTDTC)`.
/// `sun` = [RA, Dec] [rad]; `sat` = [RA, geocentric latitude [rad], height [km]].
#[allow(clippy::approx_constant)] // AL10 = 2.3025851, PIOV2 = 1.5707963: the model's own truncated constants
pub fn jb2008_core(mjd: f64, sun: [f64; 2], sat: [f64; 3], f10: f64, f10b: f64, s10: f64, s10b: f64, xm10: f64, xm10b: f64, y10: f64, y10b: f64, dstdtc: f64) -> JbOut {
    let alpha = [0.0, 0.0, 0.0, 0.0, -0.38];
    let al10 = 2.3025851;
    let amw = [28.0134, 31.9988, 15.9994, 39.9480, 4.0026, 1.00797];
    let avogad = 6.02257e26;
    let twopi = 2.0 * PI;
    let piov2 = 1.5707963;
    let frac = [0.78110, 0.20955, 9.3400e-3, 1.2890e-5];
    let rstar = 8314.32;
    let r1 = 0.010;
    let r2 = 0.025;
    let r3 = 0.075;
    let wt = [0.311111111111111, 1.422222222222222, 0.533333333333333, 1.422222222222222, 0.311111111111111];
    let cht = [0.22, -0.20e-2, 0.115e-2, -0.211e-5];
    let degrad = PI / 180.0;
    // Equation (14)
    let mut fn_ = pw(f10b / 240.0, 1.0 / 4.0);
    if fn_ > 1.0 {
        fn_ = 1.0;
    }
    let fsb = f10b * fn_ + s10b * (1.0 - fn_);
    let tsubc = 392.4 + 3.227 * fsb + 0.298 * (f10 - f10b) + 2.259 * (s10 - s10b) + 0.312 * (xm10 - xm10b) + 0.178 * (y10 - y10b);
    // Equation (15)
    let eta = 0.5 * (sat[1] - sun[1]).abs();
    let theta = 0.5 * (sat[1] + sun[1]).abs();
    // Equation (16)
    let h = sat[0] - sun[0];
    let tau = h - 0.64577182 + 0.10471976 * (h + 0.75049158).sin();
    let glat = sat[1];
    let zht = sat[2];
    let glst = h + PI;
    let mut glsthr = (glst / degrad) * (24.0 / 360.0);
    if glsthr >= 24.0 {
        glsthr -= 24.0;
    }
    if glsthr < 0.0 {
        glsthr += 24.0;
    }
    // Equation (17)
    let c = pw(eta.cos(), 2.5);
    let s = pw(theta.sin(), 2.5);
    let df = s + (c - s) * pw((0.5 * tau).cos().abs(), 3.0);
    let tsubl = tsubc * (1.0 + 0.31 * df);
    let dtclst = dtsub(f10, glsthr, glat, zht);
    let mut temp = [0.0f64; 2];
    temp[0] = tsubl + dstdtc;
    let tinf = tsubl + dstdtc + dtclst;
    // Equation (9)
    let tsubx = 444.3807 + 0.02385 * tinf - 392.8292 * (-0.0021357 * tinf).exp();
    // Equation (11)
    let gsubx = 0.054285714 * (tsubx - 183.0);
    let mut tc = [0.0f64; 4];
    tc[0] = tsubx;
    tc[1] = gsubx;
    tc[2] = (tinf - tsubx) / piov2;
    tc[3] = gsubx / tc[2];
    // Equation (5)
    let z1 = 90.0;
    let z2 = sat[2].min(105.0);
    let mut al = (z2 / z1).ln();
    let mut n = (al / r1).floor() + 1.0;
    let mut zr = (al / n).exp();
    let ambar1 = xambar(z1);
    let tloc1 = xlocal(z1, &tc);
    let mut zend = z1;
    let mut sum2 = 0.0;
    let mut ain = ambar1 * xgrav(z1) / tloc1;
    let mut z = 0.0;
    let mut ambar2 = 0.0;
    let mut tloc2 = 0.0;
    let mut gravl = 0.0;
    let mut i = 1.0;
    while i <= n {
        z = zend;
        zend = zr * z;
        let dz = 0.25 * (zend - z);
        let mut sum1 = wt[0] * ain;
        for j in 1..5 {
            z += dz;
            ambar2 = xambar(z);
            tloc2 = xlocal(z, &tc);
            gravl = xgrav(z);
            ain = ambar2 * gravl / tloc2;
            sum1 += wt[j] * ain;
        }
        sum2 += dz * sum1;
        i += 1.0;
    }
    let fact1 = 1000.0 / rstar;
    let mut rho = 3.46e-6 * ambar2 * tloc1 * (-fact1 * sum2).exp() / ambar1 / tloc2;
    // Equation (2)
    let anm = avogad * rho;
    let an = anm / ambar2;
    // Equation (3)
    let mut fact2 = anm / 28.960;
    let mut aln = [0.0f64; 6];
    aln[0] = (frac[0] * fact2).ln();
    aln[3] = (frac[2] * fact2).ln();
    aln[4] = (frac[3] * fact2).ln();
    // Equation (4)
    aln[1] = (fact2 * (1.0 + frac[1]) - an).ln();
    aln[2] = (2.0 * (an - fact2)).ln();

    let finish = |aln: &mut [f64; 6], z: f64| -> f64 {
        // Equation (24) - J70 seasonal-latitudinal variation
        let trash = (mjd - 36204.0) / 365.2422;
        let capphi = mod_(trash, 1.0);
        let dlrsl = 0.02 * (sat[2] - 90.0) * (-0.045 * (sat[2] - 90.0)).exp()
            * sign_(1.0, sat[1]) * (twopi * capphi + 1.72).sin()
            * pw(sat[1].sin(), 2.0);
        // Equation (23) - semiannual variation
        let mut dlrsa = 0.0;
        if z < 2000.0 {
            let yrday = tmoutd(mjd);
            let (fzz, _gtz, d) = semian08(yrday, zht, f10b, s10b, xm10b);
            dlrsa = d;
            if fzz < 0.0 {
                dlrsa = 0.0;
            }
        }
        let dlr = al10 * (dlrsl + dlrsa);
        for v in aln.iter_mut() {
            *v += dlr;
        }
        let mut sumnm = 0.0;
        for i in 0..6 {
            let an = aln[i].exp();
            sumnm += an * amw[i];
        }
        let mut rho = sumnm / avogad;
        // high-altitude exospheric correction
        let mut fex = 1.0;
        if (1000.0..1500.0).contains(&zht) {
            let zeta = (zht - 1000.0) * 0.002;
            let zeta2 = zeta * zeta;
            let zeta3 = zeta * zeta2;
            let f15c = cht[0] + cht[1] * f10b + cht[2] * 1500.0 + cht[3] * f10b * 1500.0;
            let f15c_zeta = (cht[2] + cht[3] * f10b) * 500.0;
            let fex2 = 3.0 * f15c - f15c_zeta - 3.0;
            let fex3 = f15c_zeta - 2.0 * f15c + 2.0;
            fex = 1.0 + fex2 * zeta2 + fex3 * zeta3;
        }
        if zht >= 1500.0 {
            fex = cht[0] + cht[1] * f10b + cht[2] * zht + cht[3] * f10b * zht;
        }
        rho *= fex;
        rho
    };

    if sat[2] <= 105.0 {
        temp[1] = tloc2;
        aln[5] = aln[4] - 25.0;
        rho = finish(&mut aln, z);
        return JbOut { temp, rho };
    }
    // Equation (6)
    let z3 = sat[2].min(500.0);
    al = (z3 / z).ln();
    n = (al / r2).floor() + 1.0;
    zr = (al / n).exp();
    sum2 = 0.0;
    ain = gravl / tloc2;
    let mut tloc3 = 0.0;
    let mut i = 1.0;
    while i <= n {
        z = zend;
        zend = zr * z;
        let dz = 0.25 * (zend - z);
        let mut sum1 = wt[0] * ain;
        for j in 1..5 {
            z += dz;
            tloc3 = xlocal(z, &tc);
            gravl = xgrav(z);
            ain = gravl / tloc3;
            sum1 += wt[j] * ain;
        }
        sum2 += dz * sum1;
        i += 1.0;
    }
    let z4 = sat[2].max(500.0);
    al = (z4 / z).ln();
    let r = if sat[2] > 500.0 { r3 } else { r2 };
    n = (al / r).floor() + 1.0;
    zr = (al / n).exp();
    let mut sum3 = 0.0;
    let mut tloc4 = 0.0;
    let mut i = 1.0;
    while i <= n {
        z = zend;
        zend = zr * z;
        let dz = 0.25 * (zend - z);
        let mut sum1 = wt[0] * ain;
        for j in 1..5 {
            z += dz;
            tloc4 = xlocal(z, &tc);
            gravl = xgrav(z);
            ain = gravl / tloc4;
            sum1 += wt[j] * ain;
        }
        sum3 += dz * sum1;
        i += 1.0;
    }
    let altr;
    let hsign;
    if sat[2] > 500.0 {
        temp[1] = tloc4;
        altr = (tloc4 / tloc2).ln();
        fact2 = fact1 * (sum2 + sum3);
        hsign = -1.0;
    } else {
        temp[1] = tloc3;
        altr = (tloc3 / tloc2).ln();
        fact2 = fact1 * sum2;
        hsign = 1.0;
    }
    for i in 0..5 {
        aln[i] = aln[i] - (1.0 + alpha[i]) * altr - fact2 * amw[i];
    }
    // Equation (7)
    let al10t5 = tinf.log10();
    let alnh5 = (5.5 * al10t5 - 39.40) * al10t5 + 73.13;
    aln[5] = al10 * (alnh5 + 6.0) + hsign * ((tloc4 / tloc3).ln() + fact1 * sum3 * amw[5]);
    rho = finish(&mut aln, z);
    JbOut { temp, rho }
}

/// Inputs the JB2008 core received from the wrapper (for diagnostics / tests).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct JbCoreInputs {
    /// MJD (UTC)
    pub mjd: f64,
    /// Sun RA, Dec [rad]
    pub sun: [f64; 2],
    /// RA [rad], latitude [rad], height [km]
    pub sat: [f64; 3],
    /// F10 F10B S10 S10B M10 M10B Y10 Y10B DSTDTC
    pub ind: [f64; 9],
}

/// `sun_radec` of `jb2008_density.m`: low-precision Sun RA/Dec (mean equinox of date).
pub fn sun_radec(mjd: f64) -> (f64, f64) {
    let t = (mjd - 51544.5) / 36525.0;
    let l = mod_(280.460 + 36000.771 * t, 360.0);
    let m = deg2rad(mod_(357.528 + 35999.050 * t, 360.0));
    let lam = deg2rad(l + 1.915 * m.sin() + 0.020 * (2.0 * m).sin());
    let eps = deg2rad(23.439 - 0.0130 * t);
    let ra = mod_((eps.cos() * lam.sin()).atan2(lam.cos()), 2.0 * PI);
    let dec = (eps.sin() * lam.sin()).asin();
    (ra, dec)
}

/// `gmst_rad` of `jb2008_density.m`: IAU-82 GMST [rad] from an MJD.
pub fn gmst_rad(mjd: f64) -> f64 {
    let tu = (mjd.floor() - 51544.5) / 36525.0;
    let gmst0 = 24110.54841 + 8640184.812866 * tu + 0.093104 * pw(tu, 2.0) - 6.2e-6 * pw(tu, 3.0);
    let frac = mod_(mjd, 1.0);
    let gsec = mod_(gmst0 + 1.0027379093 * 86400.0 * frac, 86400.0);
    deg2rad(gsec / 240.0)
}

/// `jb2008_density(t, lon_deg, lat_deg, alt_km, idx)` for ONE epoch (the in-RHS case:
/// no decimation). `utc` = [Y M D h m s]. Lags per the JB2008 spec: F10/S10 at t-1 d,
/// M10 at t-2 d, Y10 at t-5 d, DSTDTC at the epoch; linear with extrapolation.
pub fn jb2008_density(utc: &[f64; 6], lon_deg: f64, lat_deg: f64, alt_km: f64, idx: &JbIndices) -> (f64, JbOut, JbCoreInputs) {
    let dnum = datenum_v(utc);
    let mjd = dnum - 678942.0;
    let st = &idx.sol_t;
    let li = |j: usize, x: f64| interp1_linear_extrap(st, idx.col(j), x);
    let f10 = li(0, dnum - 1.0);
    let f10b = li(1, dnum - 1.0);
    let s10 = li(2, dnum - 1.0);
    let s10b = li(3, dnum - 1.0);
    let m10 = li(4, dnum - 2.0);
    let m10b = li(5, dnum - 2.0);
    let y10 = li(6, dnum - 5.0);
    let y10b = li(7, dnum - 5.0);
    let dstdtc = interp1_linear_extrap(&idx.dtc_t, &idx.dtc, dnum);
    let (ra_sun, dec_sun) = sun_radec(mjd);
    let gmst = gmst_rad(mjd);
    let ra_sat = mod_(deg2rad(lon_deg) + gmst, 2.0 * PI);
    let lat_r = deg2rad(lat_deg);
    let sun = [ra_sun, dec_sun];
    let sat = [ra_sat, lat_r, alt_km];
    let out = jb2008_core(mjd, sun, sat, f10, f10b, s10, s10b, m10, m10b, y10, y10b, dstdtc);
    (out.rho, out, JbCoreInputs { mjd, sun, sat, ind: [f10, f10b, s10, s10b, m10, m10b, y10, y10b, dstdtc] })
}
