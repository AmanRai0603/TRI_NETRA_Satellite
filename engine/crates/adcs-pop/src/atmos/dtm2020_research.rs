//! DTM2020 RESEARCH model (F30 + ap60): `density_models/dtm2020/research/`
//! `DTM2020_coeffs_init.m`, `dtm5.m`, `gldtm_Hp.m`, `bint_oe.m`, `geogm.m`,
//! `dtm2020_density.m`, the adapter `+atmos/dtm2020_research.m` and
//! `data_sources/spaceweather/solar/f30_to_f107scale.m` / `f30_from_f107.m`.
//!
//! Despite the `_Hp` in `gldtm_Hp`, this model is driven by the hourly **ap60** index
//! (converted to an open-ended Kp by `bint_oe`), not by Hp60.
use super::dtm2020::{dtm_profile, pack, DtmCoeffs, DtmDensity, DtmError, DtmRaw, Hloc, Plg, FF0};
use super::octave::{datenum, datenum_v, deg2rad, pw};
use std::f64::consts::PI;
use std::sync::OnceLock;

/// Research coefficients (`DTM2020_coeffs_init`), parsed once.
pub fn research_coeffs() -> &'static DtmCoeffs {
    static C: OnceLock<DtmCoeffs> = OnceLock::new();
    C.get_or_init(|| DtmCoeffs::parse(include_str!("../../data/atmos_drag/dtm2020_research_coeffs.txt")))
}

const AAP: [f64; 38] = [
    0.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 9.0, 12.0, 15.0, 18.0, 22.0, 27.0, 32.0, 39.0, 48.0, 56.0, 67.0, 80.0, 94.0, 111.0, 132.0,
    154.0, 179.0, 207.0, 236.0, 265.0, 294.0, 324.0, 355.0, 388.0, 421.0, 456.0, 494.0, 534.0, 574.0, 617.0, 657.0,
];
const KP_TBL: [f64; 38] = [
    0.0, 0.33, 0.66, 1.0, 1.33, 1.66, 2.0, 2.33, 2.66, 3.0, 3.33, 3.66, 4.0, 4.33, 4.66, 5.0, 5.33, 5.66, 6.0, 6.33, 6.66, 7.0,
    7.33, 7.66, 8.0, 8.33, 8.66, 9.0, 9.33, 9.66, 10.0, 10.33, 10.66, 11.0, 11.33, 11.66, 12.0, 12.33,
];

/// `bint_oe`: ap60 -> open-ended Kp by piecewise-linear interpolation over the extended
/// ap-Kp table (extrapolated below 0 and above ap = 657, where MATLAB also warns).
pub fn bint_oe(ap: f64) -> f64 {
    let n = AAP.len();
    if let Some(i) = AAP.iter().position(|&x| x == ap) {
        return KP_TBL[i];
    }
    for i in 0..n {
        if AAP[i] > ap {
            if i == 0 {
                return KP_TBL[0] + ((KP_TBL[1] - KP_TBL[0]) / (AAP[1] - AAP[0])) * (ap - AAP[0]);
            }
            return KP_TBL[i - 1] + ((KP_TBL[i] - KP_TBL[i - 1]) / (AAP[i] - AAP[i - 1])) * (ap - AAP[i - 1]);
        }
    }
    KP_TBL[n - 2] + ((KP_TBL[n - 1] - KP_TBL[n - 2]) / (AAP[n - 1] - AAP[n - 2])) * (ap - AAP[n - 2])
}

/// `geogm`: geographic -> geomagnetic latitude/longitude [deg] (dipole pole at 78.5 N,
/// 291 E). Returns `None` where MATLAB raises `GEOGM: angle error` (NaN quadrant).
pub fn geogm(xlat: f64, xlong: f64) -> Option<(f64, f64)> {
    let rfac = PI / 180.0;
    let platr = 78.5 * rfac;
    let plongr = 291.0 * rfac;
    let xlong = if xlong == 291.0 { 291.1 } else { xlong };
    let spl = platr.sin();
    let cpl = platr.cos();
    let rlat = xlat * rfac;
    let rlong = xlong * rfac;
    let slm = spl * rlat.sin() + cpl * rlat.cos() * (plongr - rlong).cos();
    let clm = (1.0 - pw(slm, 2.0)).sqrt();
    let phim1 = rlat.cos() * (rlong - plongr).sin() / clm;
    let phim2 = (spl * slm - rlat.sin()) / (cpl * clm);
    let gmlat = slm.asin() / rfac;
    let gmlong = if phim1 >= 0.0 && phim2 >= 0.0 {
        phim1.asin() / rfac
    } else if phim1 >= 0.0 && phim2 < 0.0 {
        (PI - phim1.asin().abs()) / rfac
    } else if phim1 < 0.0 && phim2 < 0.0 {
        (PI + phim1.asin().abs()) / rfac
    } else if phim1 < 0.0 && phim2 >= 0.0 {
        (2.0 * PI - phim1.asin().abs()) / rfac
    } else {
        return None;
    };
    Some((gmlat, gmlong))
}

/// `gldtm_Hp`: the research g(L) expansion for one constituent; returns `gdel`.
/// `akp` is the 8-element Kp array built from ap60 by `dtm5`.
fn gldtm_hp(f: [f64; 2], fbar: [f64; 2], akp: &[f64; 8], day: f64, a: &[f64; 97], plg: &Plg, hloc: &Hloc, xlon: f64, ff0: f64) -> f64 {
    let mut da = [0.0f64; 97];
    da[2] = plg.p20;
    da[3] = plg.p40;
    da[74] = plg.p10;
    da[77] = plg.p30;
    da[78] = plg.p50;
    da[79] = plg.p60;
    let fmfb1 = f[0] - fbar[0];
    let fbm150 = fbar[0] - 150.0;
    da[4] = fmfb1;
    da[6] = fbm150;
    da[5] = pw(da[4], 2.0);
    da[69] = pw(da[6], 2.0);
    da[82] = da[4] * plg.p10;
    da[83] = da[4] * plg.p20;
    da[84] = da[4] * plg.p30;
    da[85] = da[6] * plg.p20;
    da[86] = da[6] * plg.p30;
    da[87] = da[6] * plg.p40;
    // ---- Kp saturation during strong storms (1-based akp(k) = akp[k-1])
    let mut m = *akp;
    for k in [1usize, 4, 5, 6, 7, 8] {
        if akp[k - 1] >= 9.0 && akp[2] > 7.5 {
            m[k - 1] = 9.0 + (akp[k - 1] - 9.0) / 5.0;
        }
    }
    let dkp = m[0];
    let dkpm = m[2];
    da[7] = dkp;
    da[8] = plg.p20mg * dkp;
    da[60] = pw(dkp, 2.0);
    da[61] = plg.p20mg * da[60];
    da[62] = plg.p30mg * dkp;
    da[63] = plg.p10mg * dkp;
    da[67] = plg.p60mg * dkp;
    da[68] = plg.p40mg * dkp;
    da[64] = dkpm;
    da[65] = plg.p20mg * dkpm;
    da[66] = pw(dkpm, 2.0);
    da[73] = plg.p20mg * da[66];
    // ---- flux-dependent storm-time high-order Kp terms
    let flux = f[0].max(fbar[0]);
    let iflux = flux.floor();
    let (scale75, scale4) = if iflux >= 200.0 {
        (0.333, 0.1)
    } else if iflux >= 190.0 {
        (0.55, 0.15)
    } else if iflux >= 180.0 {
        (0.733, 0.2)
    } else if iflux >= 160.0 {
        (1.0, 0.4)
    } else if iflux >= 140.0 {
        (1.0, 0.8)
    } else {
        (1.0, 1.0)
    };
    da[75] = scale75 * pw(da[60], 2.0);
    da[71] = scale4 * pw(m[4], 4.0);
    da[72] = scale4 * pw(m[5], 4.0);
    da[76] = scale4 * pw(m[6], 4.0);
    da[79] = scale4 * pw(m[7], 4.0); // overwrites the p60 term (matches Fortran)
    da[70] = m[1];
    let mut f0 = a[4] * da[4] + a[5] * da[5] + a[6] * da[6] + a[69] * da[69]
        + a[82] * da[82] + a[83] * da[83] + a[84] * da[84] + a[85] * da[85]
        + a[86] * da[86] + a[87] * da[87];
    let f1f = 1.0 + f0 * ff0;
    f0 = f0 + a[2] * da[2] + a[3] * da[3] + a[74] * da[74] + a[77] * da[77]
        + a[7] * da[7] + a[8] * da[8] + a[60] * da[60] + a[61] * da[61]
        + a[68] * da[68] + a[64] * da[64] + a[65] * da[65] + a[66] * da[66]
        + a[72] * da[72] + a[73] * da[73] + a[75] * da[75] + a[76] * da[76]
        + a[78] * da[78] + a[79] * da[79] + a[70] * da[70] + a[71] * da[71]
        + a[62] * da[62] + a[63] * da[63] + a[67] * da[67];
    let rot = 0.017214206;
    let rot2 = 0.034428412;
    da[9] = (rot * (day - a[11])).cos();
    da[10] = plg.p20 * da[9];
    da[12] = (rot2 * (day - a[14])).cos();
    da[13] = plg.p20 * da[12];
    let coste = (rot * (day - a[18])).cos();
    da[15] = plg.p10 * coste;
    da[16] = plg.p30 * coste;
    da[17] = da[6] * da[15];
    let cos2te = (rot2 * (day - a[20])).cos();
    da[19] = plg.p10 * cos2te;
    da[39] = plg.p30 * cos2te;
    da[59] = da[6] * da[19];
    let (ch, sh, c2h, s2h, c3h, s3h) = (hloc.ch, hloc.sh, hloc.c2h, hloc.s2h, hloc.c3h, hloc.s3h);
    da[21] = plg.p11 * ch;
    da[22] = plg.p31 * ch;
    da[23] = da[6] * da[21];
    da[24] = da[21] * coste;
    da[25] = plg.p21 * ch * coste;
    da[26] = plg.p11 * sh;
    da[27] = plg.p31 * sh;
    da[28] = da[6] * da[26];
    da[29] = da[26] * coste;
    da[30] = plg.p21 * sh * coste;
    da[94] = plg.p51 * ch;
    da[95] = plg.p51 * sh;
    da[31] = plg.p22 * c2h;
    da[37] = plg.p42 * c2h;
    da[32] = plg.p32 * c2h * coste;
    da[33] = plg.p22 * s2h;
    da[38] = plg.p42 * s2h;
    da[34] = plg.p32 * s2h * coste;
    da[88] = plg.p32 * c2h;
    da[89] = plg.p32 * s2h;
    da[90] = da[6] * da[31];
    da[91] = da[6] * da[33];
    da[92] = plg.p62 * c2h;
    da[93] = plg.p62 * s2h;
    da[35] = plg.p33 * c3h;
    da[36] = plg.p33 * s3h;
    let mut fp = a[9] * da[9] + a[10] * da[10] + a[12] * da[12] + a[13] * da[13]
        + a[15] * da[15] + a[16] * da[16] + a[17] * da[17] + a[19] * da[19]
        + a[21] * da[21] + a[22] * da[22] + a[23] * da[23] + a[24] * da[24]
        + a[25] * da[25] + a[26] * da[26] + a[27] * da[27] + a[28] * da[28]
        + a[29] * da[29] + a[30] * da[30] + a[31] * da[31] + a[32] * da[32]
        + a[33] * da[33] + a[34] * da[34] + a[35] * da[35] + a[36] * da[36]
        + a[37] * da[37] + a[38] * da[38] + a[39] * da[39] + a[59] * da[59]
        + a[88] * da[88] + a[89] * da[89] + a[90] * da[90] + a[91] * da[91]
        + a[92] * da[92] + a[93] * da[93] + a[94] * da[94] + a[95] * da[95];
    da[40] = plg.p10mg * cos2te * dkpm;
    da[41] = plg.p10mg * coste * dkpm;
    da[42] = plg.p10mg * cos2te * dkp;
    da[43] = plg.p11mg * ch * dkp;
    da[44] = plg.p31mg * ch * dkp;
    da[45] = plg.p22mg * c2h * dkp;
    da[46] = plg.p11mg * sh * dkp;
    da[47] = plg.p31mg * sh * dkp;
    da[48] = plg.p22mg * s2h * dkp;
    fp = fp + a[40] * da[40] + a[41] * da[41] + a[42] * da[42] + a[43] * da[43]
        + a[44] * da[44] + a[45] * da[45] + a[46] * da[46] + a[47] * da[47]
        + a[48] * da[48];
    let clfl = xlon.cos();
    let slfl = xlon.sin();
    da[49] = plg.p11 * clfl;
    da[50] = plg.p21 * clfl;
    da[51] = plg.p31 * clfl;
    da[52] = plg.p41 * clfl;
    da[53] = plg.p51 * clfl;
    da[54] = plg.p11 * slfl;
    da[55] = plg.p21 * slfl;
    da[56] = plg.p31 * slfl;
    da[57] = plg.p41 * slfl;
    da[58] = plg.p51 * slfl;
    fp = fp + a[49] * da[49] + a[50] * da[50] + a[51] * da[51] + a[52] * da[52]
        + a[53] * da[53] + a[54] * da[54] + a[55] * da[55] + a[56] * da[56]
        + a[57] * da[57] + a[58] * da[58];
    f0 + fp * f1f
}

/// `dtm5`: the research DTM2020 core. `f`/`fbar` = [F30 on the F10.7 scale, 0],
/// `ap60` the 10-slot ap60 array, other arguments as [`super::dtm2020::dtm3`].
/// `None` only where `geogm` would raise its angle error.
pub fn dtm5(day: f64, f: [f64; 2], fbar: [f64; 2], ap60: &[f64; 10], alti: f64, hl: f64, alat: f64, xlon: f64, st: &DtmCoeffs) -> Option<DtmRaw> {
    let mut plg = Plg::geographic(alat);
    let crd = 180.0 / PI;
    let dlat = alat * crd;
    let dlon = xlon * crd;
    let (gmlatd, _) = geogm(dlat, dlon)?;
    let gmlatr = gmlatd * (PI / 180.0);
    let cm = gmlatr.sin();
    let cm2 = pw(cm, 2.0);
    let cm4 = pw(cm2, 2.0);
    let sm = gmlatr.cos();
    let sm2 = pw(sm, 2.0);
    plg.p10mg = cm;
    plg.p20mg = 1.5 * cm2 - 0.5;
    plg.p30mg = cm * (2.5 * cm2 - 1.5);
    plg.p40mg = 4.375 * cm4 - 3.75 * cm2 + 0.375;
    plg.p50mg = cm * (7.875 * cm4 - 8.75 * cm2 + 1.875);
    plg.p60mg = (5.5 * cm * plg.p50mg - 2.5 * plg.p40mg) / 3.0;
    plg.p11mg = sm;
    plg.p22mg = 3.0 * sm2;
    plg.p31mg = sm * (7.5 * cm2 - 1.5);
    let hloc = Hloc::new(hl);
    // ap60 -> Kp (open-ended); ap60(k) = ap[k-1]
    let ap = |k: usize| ap60[k - 1];
    let latabs = dlat.abs().floor();
    let ap_eff = if latabs >= 70.0 {
        let xl75 = (90.0 - latabs) / 20.0;
        (1.0 - xl75) * ((ap(3) + ap(2) + ap(4)) / 3.0) + xl75 * ((ap(3) + ap(4) + ap(5)) / 3.0)
    } else if latabs >= 30.0 {
        let xl45 = (69.0 - latabs) / 40.0;
        (1.0 - xl45) * ((ap(3) + ap(4) + ap(5)) / 3.0) + xl45 * ((ap(5) + ap(4) + ap(1)) / 3.0)
    } else {
        (ap(4) + ap(5) + ap(1)) / 3.0
    };
    let akp = [
        bint_oe(ap_eff),
        bint_oe(ap(2)) - bint_oe(ap(3)),
        bint_oe(ap(6)),
        0.0,
        bint_oe(ap(7)),
        bint_oe(ap(8)),
        bint_oe(ap(9)),
        bint_oe(ap(10)),
    ];
    let g = |a: &[f64; 97], ff0: f64| gldtm_hp(f, fbar, &akp, day, a, &plg, &hloc, xlon, ff0);
    let gd_tt = g(&st.tt, 1.0);
    let gd_t0 = g(&st.t0, 1.0);
    let gd_tp = g(&st.tp, 1.0);
    let sp = st.species();
    let mut gd = [0.0f64; 6];
    for i in 0..6 {
        gd[i] = g(sp[i], FF0[i]);
    }
    Some(dtm_profile(st, gd_tt, gd_t0, gd_tp, &gd, alti))
}

/// ap60 input of `dtm2020_density`: a scalar (all 10 slots) or the full array.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Ap60In {
    /// one ap60 in all 10 slots
    Scalar(f64),
    /// the 10-slot ap60 array
    Array([f64; 10]),
}

/// `dtm2020_density(alt_km, lat_deg, lon_deg, lst_hours, doy, F30, F30_bar, ap60, state)`.
/// `F30`/`F30_bar` must already be on the F10.7 scale ([`f30_to_f107scale`]).
pub fn density(alt_km: f64, lat_deg: f64, lon_deg: f64, lst_hours: f64, doy: f64, f30: f64, f30_bar: f64, ap60: Ap60In, st: &DtmCoeffs) -> Result<DtmDensity, DtmError> {
    if alt_km <= 120.0 {
        return Err(DtmError::AltitudeTooLow(alt_km));
    }
    let ap = match ap60 {
        Ap60In::Scalar(a) => [a; 10],
        Ap60In::Array(v) => v,
    };
    let hl = lst_hours / 24.0 * 2.0 * PI;
    let alat = deg2rad(lat_deg);
    let xlon = deg2rad(lon_deg);
    let raw = dtm5(doy, [f30, 0.0], [f30_bar, 0.0], &ap, alt_km, hl, alat, xlon, st).ok_or(DtmError::AltitudeTooLow(f64::NAN))?;
    Ok(pack(raw))
}

/// `f30_to_f107scale`: native F30 -> F10.7 scale, DTM2020 paper eq. 2 (drift term in the
/// decimal year).
pub fn f30_to_f107scale(f30: f64, decimal_year: f64) -> f64 {
    -1.5998 + 1.553755 * f30 + (0.22446 * decimal_year - 447.13328)
}

/// `f30_from_f107`: pseudo-F30 derived from F10.7 (the last-resort source of `get_f30`).
pub fn f30_from_f107(f107: f64) -> f64 {
    (f107 + 1.6) / 1.554
}

/// `decimalYear` of `+atmos/dtm2020_research.m` (leap-year aware, via `datenum`).
pub fn decimal_year(utc: &[f64; 6]) -> f64 {
    let y0 = utc[0];
    let d0 = datenum(y0, 1.0, 1.0, 0.0, 0.0, 0.0);
    let d1 = datenum(y0 + 1.0, 1.0, 1.0, 0.0, 0.0, 0.0);
    y0 + (datenum_v(utc) - d0) / (d1 - d0)
}
