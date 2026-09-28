//! DTM2020 OPERATIONAL model (F10.7 + Kp): `density_models/dtm2020/operational/`
//! `DTM2020_F107_coeffs_init.m`, `dtm3.m`, `gldtm.m`, `dtm2020_oper_density.m`, and the
//! adapter `+atmos/dtm2020.m`. This is the density of the SILS in-loop orbit.
//!
//! The coefficient tables are the MATLAB transcription exported by
//! `refgen/atmos_drag_coeffs.m` (`%.17g`, so the parsed doubles are identical).
use super::octave::{deg2rad, pw};
use std::f64::consts::PI;
use std::sync::OnceLock;

/// One DTM2020 coefficient set: nine 96-term vectors (1-based, index 0 unused), as the
/// MATLAB `state` struct from `DTM2020_F107_coeffs_init` / `DTM2020_coeffs_init`.
#[derive(Clone, Debug)]
pub struct DtmCoeffs {
    /// exospheric temperature
    pub tt: [f64; 97],
    /// H, He, O, N2 (`az2`), O2, N (`az`) densities at 120 km
    pub h: [f64; 97],
    /// He
    pub he: [f64; 97],
    /// O
    pub o: [f64; 97],
    /// N2
    pub az2: [f64; 97],
    /// O2
    pub o2: [f64; 97],
    /// N
    pub az: [f64; 97],
    /// temperature and its gradient at 120 km
    pub t0: [f64; 97],
    /// temperature gradient at 120 km
    pub tp: [f64; 97],
}

impl DtmCoeffs {
    /// Parse the 96 x 9 text table written by `refgen/atmos_drag_coeffs.m`
    /// (columns tt h he o az2 o2 az t0 tp; `#` comment lines).
    pub fn parse(text: &str) -> DtmCoeffs {
        let mut c = [[0.0f64; 97]; 9];
        let mut row = 0usize;
        for line in text.lines() {
            let l = line.trim();
            if l.is_empty() || l.starts_with('#') {
                continue;
            }
            row += 1;
            assert!(row <= 96, "DTM2020 coefficient table has more than 96 rows");
            for (j, tok) in l.split_whitespace().enumerate() {
                c[j][row] = tok.parse::<f64>().expect("DTM2020 coefficient");
            }
        }
        assert_eq!(row, 96, "DTM2020 coefficient table must have 96 rows");
        DtmCoeffs { tt: c[0], h: c[1], he: c[2], o: c[3], az2: c[4], o2: c[5], az: c[6], t0: c[7], tp: c[8] }
    }
    /// The six species vectors in DTM order H, He, O, N2, O2, N.
    pub fn species(&self) -> [&[f64; 97]; 6] {
        [&self.h, &self.he, &self.o, &self.az2, &self.o2, &self.az]
    }
}

/// Operational coefficients (`DTM2020_F107_coeffs_init`), parsed once.
pub fn oper_coeffs() -> &'static DtmCoeffs {
    static C: OnceLock<DtmCoeffs> = OnceLock::new();
    C.get_or_init(|| DtmCoeffs::parse(include_str!("../../data/atmos_drag/dtm2020_oper_coeffs.txt")))
}

// ---- shared DTM constants (dtm3 / dtm5) --------------------------------------
pub(crate) const RE: f64 = 6356.77;
pub(crate) const GSURF: f64 = 980.665;
pub(crate) const RGAS: f64 = 831.4;
pub(crate) const ZLB0: f64 = 120.0;
pub(crate) const MA: [f64; 6] = [1.0, 4.0, 16.0, 28.0, 32.0, 14.0];
pub(crate) const ALEFA: [f64; 6] = [-0.40, -0.38, 0.0, 0.0, 0.0, 0.0];
/// Particle masses [g] of H, He, O, N2, O2, N (`vma`).
pub const VMA: [f64; 6] = [1.6606e-24, 6.6423e-24, 26.569e-24, 46.4958e-24, 53.1381e-24, 23.2479e-24];
pub(crate) const FF0: [f64; 6] = [0.0, 0.0, 1.0, 1.0, 1.0, 1.0];

/// Geographic Legendre functions of `dtm3`/`dtm5` (`plg.*`), plus the geomagnetic ones.
#[derive(Clone, Copy, Default, Debug)]
pub(crate) struct Plg {
    pub p10: f64, pub p20: f64, pub p30: f64, pub p40: f64, pub p50: f64, pub p60: f64,
    pub p11: f64, pub p21: f64, pub p31: f64, pub p41: f64, pub p51: f64,
    pub p22: f64, pub p32: f64, pub p42: f64, pub p52: f64, pub p62: f64, pub p33: f64,
    pub p10mg: f64, pub p20mg: f64, pub p30mg: f64, pub p40mg: f64, pub p50mg: f64, pub p60mg: f64,
    pub p11mg: f64, pub p22mg: f64, pub p31mg: f64,
}

impl Plg {
    /// The geographic block common to dtm3 and dtm5.
    pub(crate) fn geographic(alat: f64) -> Plg {
        let c = alat.sin();
        let c2 = pw(c, 2.0);
        let c4 = pw(c2, 2.0);
        let s = alat.cos();
        let s2 = pw(s, 2.0);
        let mut p = Plg::default();
        p.p10 = c;
        p.p20 = 1.5 * c2 - 0.5;
        p.p30 = c * (2.5 * c2 - 1.5);
        p.p40 = 4.375 * c4 - 3.75 * c2 + 0.375;
        p.p50 = c * (7.875 * c4 - 8.75 * c2 + 1.875);
        p.p60 = (5.5 * c * p.p50 - 2.5 * p.p40) / 3.0;
        p.p11 = s;
        p.p21 = 3.0 * c * s;
        p.p31 = s * (7.5 * c2 - 1.5);
        p.p41 = c * s * (17.5 * c2 - 7.5);
        p.p51 = s * (39.375 * c4 - 26.25 * c2 + 1.875);
        p.p22 = 3.0 * s2;
        p.p32 = 15.0 * c * s2;
        p.p42 = s2 * (52.5 * c2 - 7.5);
        p.p52 = 3.0 * c * p.p42 - 2.0 * p.p32;
        p.p62 = 2.75 * c * p.p52 - 1.75 * p.p42;
        p.p33 = 15.0 * s * s2;
        p
    }
}

/// Local-time harmonics (`hloc.*`).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Hloc {
    pub ch: f64, pub sh: f64, pub c2h: f64, pub s2h: f64, pub c3h: f64, pub s3h: f64,
}

impl Hloc {
    pub(crate) fn new(hl: f64) -> Hloc {
        let ch = hl.cos();
        let sh = hl.sin();
        let c2h = pw(ch, 2.0) - pw(sh, 2.0);
        let s2h = 2.0 * ch * sh;
        let c3h = c2h * ch - s2h * sh;
        let s3h = s2h * ch + c2h * sh;
        Hloc { ch, sh, c2h, s2h, c3h, s3h }
    }
}

/// `gldtm`: the operational g(L) expansion for one constituent; returns `gdel`.
/// `akp` = [Kp 3 h delayed, 0, mean Kp 24 h, 0]; `ff0` = 1 temperature, 0 density.
pub(crate) fn gldtm(f: [f64; 2], fbar: [f64; 2], akp: [f64; 4], day: f64, a: &[f64; 97], plg: &Plg, hloc: &Hloc, xlon: f64, ff0: f64) -> f64 {
    let mut da = [0.0f64; 97];
    let akp = |i: usize| akp[i - 1];
    // ---- Legendre terms
    da[2] = plg.p20;
    da[3] = plg.p40;
    da[74] = plg.p10;
    da[77] = plg.p30;
    da[78] = plg.p50;
    da[79] = plg.p60;
    // ---- solar flux terms
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
    // ---- Kp
    let ikp = 62usize;
    let ikpm = 67usize;
    let c2fi = 1.0 - pw(plg.p10mg, 2.0);
    let dkp = akp(1) + (a[ikp] + c2fi * a[ikp + 1]) * akp(2);
    let dakp = a[7] + a[8] * plg.p20mg + a[68] * plg.p40mg
        + 2.0 * dkp * (a[60] + a[61] * plg.p20mg + a[75] * 2.0 * dkp * dkp);
    da[ikp] = dakp * akp(2);
    da[ikp + 1] = da[ikp] * c2fi;
    let dkpm = akp(3) + a[ikpm] * akp(4);
    let dakpm = a[64] + a[65] * plg.p20mg + a[72] * plg.p40mg
        + 2.0 * dkpm * (a[66] + a[73] * plg.p20mg + a[76] * 2.0 * dkpm * dkpm);
    da[ikpm] = dakpm * akp(4);
    da[7] = dkp;
    da[8] = plg.p20mg * dkp;
    da[68] = plg.p40mg * dkp;
    da[60] = pw(dkp, 2.0);
    da[61] = plg.p20mg * da[60];
    da[75] = pw(da[60], 2.0);
    da[64] = dkpm;
    da[65] = plg.p20mg * dkpm;
    da[72] = plg.p40mg * dkpm;
    da[66] = pw(dkpm, 2.0);
    da[73] = plg.p20mg * da[66];
    da[76] = pw(da[66], 2.0);
    // ---- static part
    let mut f0 = a[4] * da[4] + a[5] * da[5] + a[6] * da[6] + a[69] * da[69]
        + a[82] * da[82] + a[83] * da[83] + a[84] * da[84] + a[85] * da[85]
        + a[86] * da[86] + a[87] * da[87];
    let f1f = 1.0 + f0 * ff0;
    f0 = f0 + a[2] * da[2] + a[3] * da[3] + a[74] * da[74] + a[77] * da[77]
        + a[7] * da[7] + a[8] * da[8] + a[60] * da[60] + a[61] * da[61]
        + a[68] * da[68] + a[64] * da[64] + a[65] * da[65] + a[66] * da[66]
        + a[72] * da[72] + a[73] * da[73] + a[75] * da[75] + a[76] * da[76]
        + a[78] * da[78] + a[79] * da[79];
    // ---- seasonal
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
    // ---- diurnal
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
    // ---- semi-diurnal
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
    // ---- ter-diurnal
    da[35] = plg.p33 * c3h;
    da[36] = plg.p33 * s3h;
    // ---- periodic part
    let mut fp = a[9] * da[9] + a[10] * da[10] + a[12] * da[12] + a[13] * da[13]
        + a[15] * da[15] + a[16] * da[16] + a[17] * da[17] + a[19] * da[19]
        + a[21] * da[21] + a[22] * da[22] + a[23] * da[23] + a[24] * da[24]
        + a[25] * da[25] + a[26] * da[26] + a[27] * da[27] + a[28] * da[28]
        + a[29] * da[29] + a[30] * da[30] + a[31] * da[31] + a[32] * da[32]
        + a[33] * da[33] + a[34] * da[34] + a[35] * da[35] + a[36] * da[36]
        + a[37] * da[37] + a[38] * da[38] + a[39] * da[39] + a[59] * da[59]
        + a[88] * da[88] + a[89] * da[89] + a[90] * da[90] + a[91] * da[91]
        + a[92] * da[92] + a[93] * da[93] + a[94] * da[94] + a[95] * da[95];
    // ---- storm-time coupling (geographic Legendre, dkp)
    da[40] = plg.p10 * coste * dkp;
    da[41] = plg.p30 * coste * dkp;
    da[42] = plg.p50 * coste * dkp;
    da[43] = plg.p11 * ch * dkp;
    da[44] = plg.p31 * ch * dkp;
    da[45] = plg.p51 * ch * dkp;
    da[46] = plg.p11 * sh * dkp;
    da[47] = plg.p31 * sh * dkp;
    da[48] = plg.p51 * sh * dkp;
    fp = fp + a[40] * da[40] + a[41] * da[41] + a[42] * da[42] + a[43] * da[43]
        + a[44] * da[44] + a[45] * da[45] + a[46] * da[46] + a[47] * da[47]
        + a[48] * da[48];
    // (second Kp adjustment only touches da(62), da(63) -- not used by gdel)
    // ---- non-migrating tides
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

/// Raw output of `dtm3` / `dtm5`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DtmRaw {
    /// temperature at altitude [K]
    pub tz: f64,
    /// exospheric temperature [K]
    pub tinf: f64,
    /// total density [g/cm^3]
    pub ro: f64,
    /// partial densities [g/cm^3]: H, He, O, N2, O2, N
    pub d: [f64; 6],
    /// mean molecular mass [amu]
    pub wmm: f64,
}

/// The Bates profile + species integration shared by `dtm3` and `dtm5`, given the nine
/// `gdel` values (tt, t0, tp, then the six species).
pub(crate) fn dtm_profile(st: &DtmCoeffs, gd_tt: f64, gd_t0: f64, gd_tp: f64, gd_sp: &[f64; 6], alti: f64) -> DtmRaw {
    let zlb = ZLB0;
    let tinf = st.tt[1] * (1.0 + gd_tt);
    let t120 = st.t0[1] * (1.0 + gd_t0);
    let tp120 = st.tp[1] * (1.0 + gd_tp);
    let sigma = tp120 / (tinf - t120);
    let dzeta = (RE + zlb) / (RE + alti);
    let zeta = (alti - zlb) * dzeta;
    let sigzeta = sigma * zeta;
    let expsz = (-sigzeta).exp();
    let tz = tinf - (tinf - t120) * expsz;
    let sp = st.species();
    let mut dbase = [0.0f64; 6];
    for i in 0..6 {
        dbase[i] = sp[i][1] * gd_sp[i].exp();
    }
    let mut glb = GSURF / pw(1.0 + zlb / RE, 2.0);
    glb /= sigma * RGAS * tinf;
    let t120tz = t120 / tz;
    let mut d = [0.0f64; 6];
    let mut cc = [0.0f64; 6];
    let mut ro = 0.0;
    for i in 0..6 {
        let gamma = MA[i] * glb;
        let upapg = 1.0 + ALEFA[i] + gamma;
        let fz_i = pw(t120tz, upapg) * (-sigzeta * gamma).exp();
        cc[i] = dbase[i] * fz_i;
        d[i] = cc[i] * VMA[i];
        ro += d[i];
    }
    let mut sum_cc = 0.0;
    for c in cc.iter() {
        sum_cc += c;
    }
    let wmm = ro / (VMA[0] * sum_cc);
    DtmRaw { tz, tinf, ro, d, wmm }
}

/// `dtm3`: the operational DTM2020 core. `f = [F10.7(t-24h), 0]`, `fbar = [F10.7 81-day
/// mean, 0]`, `akp` 4-element Kp array, `alti` [km] (> 120), `hl` local solar time
/// [rad], `alat`/`xlon` geographic latitude/longitude [rad].
pub fn dtm3(day: f64, f: [f64; 2], fbar: [f64; 2], akp: [f64; 4], alti: f64, hl: f64, alat: f64, xlon: f64, st: &DtmCoeffs) -> DtmRaw {
    let cpmg = 0.19081;
    let spmg = 0.98163;
    let xlmg = -1.2392;
    let mut plg = Plg::geographic(alat);
    let c = alat.sin();
    let s = alat.cos();
    // geomagnetic latitude, dipole approximation
    let clmlmg = (xlon - xlmg).cos();
    let sp = s * cpmg * clmlmg + c * spmg;
    let cmg = sp;
    let cmg2 = pw(cmg, 2.0);
    let cmg4 = pw(cmg2, 2.0);
    plg.p10mg = cmg;
    plg.p20mg = 1.5 * cmg2 - 0.5;
    plg.p40mg = 4.375 * cmg4 - 3.75 * cmg2 + 0.375;
    let hloc = Hloc::new(hl);
    let g = |a: &[f64; 97], ff0: f64| gldtm(f, fbar, akp, day, a, &plg, &hloc, xlon, ff0);
    let gd_tt = g(&st.tt, 1.0);
    let gd_t0 = g(&st.t0, 1.0);
    let gd_tp = g(&st.tp, 1.0);
    let sp = st.species();
    let mut gd = [0.0f64; 6];
    for i in 0..6 {
        gd[i] = g(sp[i], FF0[i]);
    }
    dtm_profile(st, gd_tt, gd_t0, gd_tp, &gd, alti)
}

/// Kp input of the operational model: a scalar (used for both the 3 h-delayed and the
/// 24 h-mean slots) or the full 4-element `akp` array `[Kp_3h; 0; Kp_24h; 0]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum KpIn {
    /// one Kp for both slots
    Scalar(f64),
    /// `[Kp_3h; 0; Kp_24h; 0]`
    Akp([f64; 4]),
}

/// Output of `dtm2020_oper_density` / `dtm2020_density`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DtmDensity {
    /// total density [g/cm^3]
    pub rho_gcm3: f64,
    /// total density [kg/m^3]
    pub rho_kgm3: f64,
    /// temperature at altitude [K]
    pub t_k: f64,
    /// exospheric temperature [K]
    pub tinf_k: f64,
    /// number densities [1/cm^3], H He O N2 O2 N
    pub n_cm3: [f64; 6],
    /// partial mass densities [g/cm^3]
    pub rho_species: [f64; 6],
    /// mean molecular mass [amu]
    pub mbar_amu: f64,
}

/// Error of the DTM wrappers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DtmError {
    /// `alt_km <= 120` (the model's lower boundary).
    AltitudeTooLow(f64),
}

pub(crate) fn pack(o: DtmRaw) -> DtmDensity {
    let mut n = [0.0f64; 6];
    for i in 0..6 {
        n[i] = o.d[i] / VMA[i];
    }
    DtmDensity { rho_gcm3: o.ro, rho_kgm3: o.ro * 1000.0, t_k: o.tz, tinf_k: o.tinf, n_cm3: n, rho_species: o.d, mbar_amu: o.wmm }
}

/// `dtm2020_oper_density(alt_km, lat_deg, lon_deg, lst_hours, doy, F107, F107_bar, Kp, state)`.
pub fn oper_density(alt_km: f64, lat_deg: f64, lon_deg: f64, lst_hours: f64, doy: f64, f107: f64, f107_bar: f64, kp: KpIn, st: &DtmCoeffs) -> Result<DtmDensity, DtmError> {
    if alt_km <= 120.0 {
        return Err(DtmError::AltitudeTooLow(alt_km));
    }
    let f = [f107, 0.0];
    let fbar = [f107_bar, 0.0];
    let akp = match kp {
        KpIn::Scalar(k) => [k, 0.0, k, 0.0],
        KpIn::Akp(a) => a,
    };
    let hl = lst_hours / 24.0 * 2.0 * PI;
    let alat = deg2rad(lat_deg);
    let xlon = deg2rad(lon_deg);
    Ok(pack(dtm3(doy, f, fbar, akp, alt_km, hl, alat, xlon, st)))
}
