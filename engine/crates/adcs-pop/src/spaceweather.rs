//! Space-weather drivers of the density models: `04_atmosphere/+atmos/spaceweather.m`
//! (manual indices and the per-epoch table lookup), `+atmos/research_drivers.m`,
//! `05_data/+data/drivers.m` (which model eats which index), `+data/spaceweather.m`
//! (`mergeToStruct`), `+data/spaceweather_forecast.m`, and the parsers of the cached
//! source formats (`get_omni2` / `get_f107`, `get_gfz_hpo`, the SWPC forecast files).
//! Downloads are out of scope: every parser takes the file TEXT.
//!
//! What the density models receive:
//! * DTM2020 operational (`atmos.dtm2020`): `F107` (the table's F10.7 of the epoch's
//!   day -- same-day by default, t-24 h with `lag_f107`), `F107a` (81-day centred
//!   mean), and a SCALAR `Kp` (daily mean of the 3-hourly Kp for a table; the manual
//!   value for manual input), which `dtm2020_oper_density` expands to
//!   `akp = [Kp; 0; Kp; 0]` (the same Kp in the 3 h-delayed and 24 h-mean slots).
//!   With the SILS manual set F10.7 = F10.7a = 130, Kp = 2, ap = 7 that is exactly
//!   `dtm3(doy, [130;0], [130;0], [2;0;2;0], ...)`; `ap` is carried but unused.
//! * DTM2020 research (`atmos.dtm2020_research`): `F30`, `F30_bar` (rescaled to the
//!   F10.7 scale unless the F30 was itself derived from F10.7) and ONE `ap60` value
//!   (most recent hourly sample at/before the epoch) put in all 10 ap60 slots. The
//!   Hp60 index is not used anywhere in the pipeline.
//! * NRLMSISE-00: `F107`, `F107a`, `ap` + 7-slot `aph` (not ported: Aerospace Toolbox).
//! * JB2008: its own SET tables (see [`crate::atmos::jb2008::JbIndices`]).
#![deny(missing_docs)]
#![allow(rustdoc::broken_intra_doc_links)] // unit brackets like [K], [m/s] in the docs
use crate::atmos::dtm2020::KpIn;
use crate::atmos::octave::{datenum, interp1_linear_extrap};
use crate::gen::swindex as swi;

/// `kp2ap` (atmos.spaceweather): nearest node of the standard table, Kp clamped to [0, 9]. `ap2kp`: the inverse,
/// nearest node, ap clamped to [0, 400]. `ap2kp_forecast` (data.spaceweather_forecast): LINEAR with extrapolation on
/// Kp = (0:27)/3, ap clamped above at 400 only. env's method env_space_weather over env's Kp-ap table, generated
/// from the design into `gen::swindex`.
pub use crate::gen::swindex::{ap2kp, ap2kp_forecast, kp2ap};

/// Manual space weather (`opts.manual` / `cfg.spaceweather.manual`). `F107` is
/// required; `F107a` defaults to `F107`; at least one of `Kp`/`ap` is required (the
/// other is derived); `ap3` defaults to `ap`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ManualIndices {
    /// `F107` [sfu] (required)
    pub f107: f64,
    /// `F107a` [sfu] (default F107)
    pub f107a: Option<f64>,
    /// `Kp`
    pub kp: Option<KpIn>,
    /// `ap`
    pub ap: Option<f64>,
    /// `ap3` (default ap)
    pub ap3: Option<f64>,
}

impl ManualIndices {
    /// The SILS setting of `asils.config`: F10.7 = F10.7a = 130, Kp = 2, ap = 7.
    pub const SILS: ManualIndices = ManualIndices { f107: 130.0, f107a: Some(130.0), kp: Some(KpIn::Scalar(2.0)), ap: Some(7.0), ap3: None };
}

/// Where a [`SpaceWeather`] came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SwOrigin {
    /// `sw.source = 'manual'`
    Manual,
    /// a table (`sw.source = T.source`, see [`SwTable::source`])
    Table,
}

/// The `sw` struct returned by `atmos.spaceweather`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpaceWeather {
    /// F10.7 handed to the models [sfu]
    pub f107: f64,
    /// 81-day centred mean [sfu]
    pub f107a: f64,
    /// Kp (scalar, or the 4-element akp a manual input may give)
    pub kp: KpIn,
    /// daily ap
    pub ap: f64,
    /// 3-hourly ap at the epoch
    pub ap3: f64,
    /// NRLMSISE-00 7-slot ap array
    pub aph: [f64; 7],
    /// same-day F10.7 (always carried)
    pub f107_today: f64,
    /// `sw.source` manual / table
    pub origin: SwOrigin,
    /// `sw.aph_source`
    pub aph_source: &'static str,
    /// `sw.F107_lag` (table path only)
    pub f107_lag: Option<&'static str>,
}

impl SpaceWeather {
    /// Scalar Kp (the first element of an akp array).
    pub fn kp_scalar(&self) -> f64 {
        match self.kp {
            KpIn::Scalar(k) => k,
            KpIn::Akp(a) => a[0],
        }
    }
}

/// Errors of the space-weather resolution (the MATLAB `error(...)` identifiers).
#[derive(Clone, Debug, PartialEq)]
pub enum SwError {
    /// `atmos:spaceweather:manualGeomag` -- manual input without Kp or ap
    ManualGeomag,
    /// `atmos:spaceweather:manualMissing`
    ManualMissing(&'static str),
    /// `atmos:spaceweather:noTable` -- neither manual values nor a table
    NoTable,
    /// `atmos:spaceweather:noData` -- epoch day (MJD) not in the table
    NoData(f64),
    /// `atmos:spaceweather:nonfinite` -- named index missing / non-finite
    NonFinite(&'static str),
    /// a manual akp array without an explicit ap (MATLAB would produce a vector ap)
    AkpNeedsAp,
    /// `atmos:research_drivers:noDrivers` / `:coverage`
    ResearchCoverage(&'static str),
    /// parse error of a cached source file
    Parse(String),
}

fn assert_finite(sw: &SpaceWeather) -> Result<(), SwError> {
    if !sw.f107.is_finite() {
        return Err(SwError::NonFinite("F107"));
    }
    if !sw.f107a.is_finite() {
        return Err(SwError::NonFinite("F107a"));
    }
    let kp_ok = match sw.kp {
        KpIn::Scalar(k) => k.is_finite(),
        KpIn::Akp(a) => a.iter().all(|x| x.is_finite()),
    };
    if !kp_ok {
        return Err(SwError::NonFinite("Kp"));
    }
    if !sw.ap.is_finite() {
        return Err(SwError::NonFinite("ap"));
    }
    Ok(())
}

/// The manual branch of `atmos.spaceweather(utc, struct('manual', m))`: env's method (`gen::swindex::from_manual`).
pub fn from_manual(m: &ManualIndices) -> Result<SpaceWeather, SwError> {
    let (kind, akp) = match m.kp {
        None => (0, [f64::NAN; 4]),
        Some(KpIn::Scalar(k)) => (1, [k, 0.0, k, 0.0]),
        Some(KpIn::Akp(a)) => (2, a),
    };
    let (st, w) = swi::from_manual(m.f107, m.f107a.is_some(), m.f107a.unwrap_or(f64::NAN), kind, akp, m.ap.is_some(), m.ap.unwrap_or(f64::NAN),
                                   m.ap3.is_some(), m.ap3.unwrap_or(f64::NAN));
    match st {
        swi::SWSTATUS_SW_OK => Ok(SpaceWeather {
            f107: w.f107,
            f107a: w.f107a,
            kp: if w.kp_is_array { KpIn::Akp(w.akp) } else { KpIn::Scalar(w.kp) },
            ap: w.ap,
            ap3: w.ap3,
            aph: w.aph,
            f107_today: w.f107_today,
            origin: SwOrigin::Manual,
            aph_source: "manual-flat (no storm history)",
            f107_lag: None,
        }),
        swi::SWSTATUS_MANUAL_GEOMAG => Err(SwError::ManualGeomag),
        swi::SWSTATUS_AKP_NEEDS_AP => Err(SwError::AkpNeedsAp),
        swi::SWSTATUS_NONFINITE_F107 => Err(SwError::NonFinite("F107")),
        swi::SWSTATUS_NONFINITE_F107A => Err(SwError::NonFinite("F107a")),
        swi::SWSTATUS_NONFINITE_KP => Err(SwError::NonFinite("Kp")),
        _ => Err(SwError::NonFinite("ap")),
    }
}

/// The uniform daily space-weather table of `data.spaceweather` /
/// `data.spaceweather_forecast` (`SW.mjd f107obs f107c81 kp apDaily ap(Nx8) source`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SwTable {
    /// UTC MJD of each day (00:00)
    pub mjd: Vec<f64>,
    /// observed F10.7 [sfu]
    pub f107obs: Vec<f64>,
    /// 81-day centred mean [sfu]
    pub f107c81: Vec<f64>,
    /// daily mean Kp
    pub kp: Vec<f64>,
    /// daily Ap
    pub ap_daily: Vec<f64>,
    /// the eight 3-hourly ap of each day
    pub ap: Vec<[f64; 8]>,
    /// `SW.source`
    pub source: String,
}

/// Options of the table branch of `atmos.spaceweather`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SwOpts {
    /// `opts.lag_f107` (default false = same-day F10.7, the GOCE-validated convention)
    pub lag_f107: bool,
    /// `opts.aph_mode = 'history'` (default 'flat')
    pub aph_history: bool,
}

/// The table branch of `atmos.spaceweather(utc, struct('table', T, ...))`.
pub fn from_table(t: &SwTable, utc: &[f64; 6], opts: SwOpts) -> Result<SpaceWeather, SwError> {
    let mjd = (datenum(utc[0], utc[1], utc[2], 0.0, 0.0, 0.0) - 678942.0).floor();
    let k = match t.mjd.iter().position(|&m| m == mjd) {
        Some(k) => k,
        None => return Err(SwError::NoData(mjd)),
    };
    let hourbin = 8.0f64.min(((utc[3] + utc[4] / 60.0) / 3.0).floor() + 1.0) as usize;
    let kprev = t.mjd.iter().position(|&m| m == mjd - 1.0);
    let f107_today = t.f107obs[k];
    let (f107, lag) = match kprev {
        Some(kp) if opts.lag_f107 && t.f107obs[kp].is_finite() => (t.f107obs[kp], "previous day (t-24h, per model spec)"),
        _ => {
            let msg = if opts.lag_f107 {
                "SAME DAY (lag requested but no t-24h record -- extend padDays)"
            } else {
                "same day ('previous' sample -- matches align_drivers_to_track)"
            };
            (t.f107obs[k], msg)
        }
    };
    let ap = t.ap_daily[k];
    let ap3 = t.ap[k][hourbin - 1];
    let (aph, src) = if opts.aph_history {
        (build_aph(t, k, hourbin), "history (real 57 h, spec form; needs flags(9)=-1)")
    } else {
        ([ap; 7], "flat (matches run_comparison_study.m -- the validated convention)")
    };
    let sw = SpaceWeather {
        f107,
        f107a: t.f107c81[k],
        kp: KpIn::Scalar(t.kp[k]),
        ap,
        ap3,
        aph,
        f107_today,
        origin: SwOrigin::Table,
        aph_source: src,
        f107_lag: Some(lag),
    };
    assert_finite(&sw)?;
    Ok(sw)
}

/// `buildAph`: NRLMSISE-00's 7-slot ap history from the 3-hourly table.
fn build_aph(t: &SwTable, k: usize, hourbin: usize) -> [f64; 7] {
    let nflat = t.ap.len() * 8;
    let flat = |i: isize| -> f64 {
        if i >= 1 && (i as usize) <= nflat {
            let j = i as usize - 1;
            t.ap[j / 8][j % 8]
        } else {
            f64::NAN
        }
    };
    let now = (k * 8 + hourbin) as isize;
    let mean_bins = |b0: isize, b1: isize| -> f64 {
        let mut s = 0.0;
        let mut n = 0usize;
        let mut b = b1;
        while b >= b0 {
            let v = flat(now - b);
            if v.is_finite() {
                s += v;
                n += 1;
            }
            b -= 1;
        }
        if n == 0 { f64::NAN } else { s / n as f64 }
    };
    let mut aph = [
        t.ap_daily[k],
        flat(now),
        flat(now - 1),
        flat(now - 2),
        flat(now - 3),
        mean_bins(4, 11),
        mean_bins(12, 19),
    ];
    for v in aph.iter_mut() {
        if !v.is_finite() {
            *v = t.ap_daily[k];
        }
    }
    aph
}

/// `atmos.spaceweather(utc, opts)`: manual values win over a table; neither is an error.
pub fn resolve(utc: &[f64; 6], manual: Option<&ManualIndices>, table: Option<&SwTable>, opts: SwOpts) -> Result<SpaceWeather, SwError> {
    if let Some(m) = manual {
        return from_manual(m);
    }
    match table {
        Some(t) => from_table(t, utc, opts),
        None => Err(SwError::NoTable),
    }
}

// ---------------------------------------------------------------------------------
// DTM2020 research drivers
// ---------------------------------------------------------------------------------

/// The research-model driver bundle of `data.drivers('dtm2020_research')`:
/// `.f30TT` (daily F30, F30_bar), `.ap60TT` (hourly ap60), `.f30_is_derived`.
/// Times are `datenum` days.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ResearchDrivers {
    /// F30 epochs [datenum]
    pub f30_t: Vec<f64>,
    /// F30 [sfu]
    pub f30: Vec<f64>,
    /// F30 81-day mean [sfu]
    pub f30_bar: Vec<f64>,
    /// ap60 epochs [datenum]
    pub ap60_t: Vec<f64>,
    /// hourly ap60
    pub ap60: Vec<f64>,
    /// F30 is an F10.7-derived pseudo-F30 (no rescaling)
    pub f30_is_derived: bool,
}

/// The `sw` of `atmos.research_drivers`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResearchSw {
    /// `sw.F30`
    pub f30: f64,
    /// `sw.F30_bar`
    pub f30_bar: f64,
    /// `sw.ap60`
    pub ap60: f64,
    /// `sw.f30_is_derived`
    pub f30_is_derived: bool,
}

/// `prevSample`: most recent finite sample at or before `tq`.
fn prev_sample(t: &[f64], v: &[f64], tq: f64, name: &'static str) -> Result<f64, SwError> {
    let mut found = None;
    for i in 0..t.len().min(v.len()) {
        if t[i] <= tq && v[i].is_finite() {
            found = Some(v[i]);
        }
    }
    found.ok_or(SwError::ResearchCoverage(name))
}

/// `atmos.research_drivers(DRV, utc)`: sample the research drivers at one epoch
/// ('previous' rule, never interpolated).
pub fn research_sample(d: &ResearchDrivers, utc: &[f64; 6]) -> Result<ResearchSw, SwError> {
    let tq = datenum(utc[0], utc[1], utc[2], utc[3], utc[4], utc[5]);
    Ok(ResearchSw {
        f30: prev_sample(&d.f30_t, &d.f30, tq, "F30")?,
        f30_bar: prev_sample(&d.f30_t, &d.f30_bar, tq, "F30_bar")?,
        ap60: prev_sample(&d.ap60_t, &d.ap60, tq, "ap60")?,
        f30_is_derived: d.f30_is_derived,
    })
}

// ---------------------------------------------------------------------------------
// data.drivers: which model eats which index
// ---------------------------------------------------------------------------------

/// The driver set a density model needs (`data.drivers`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DriverSet {
    /// exponential: altitude only
    None,
    /// nrlmsise, dtm2020: F10.7 + Kp/ap table (or manual values)
    F107Kp,
    /// dtm2020_research: F30 + ap60 (no manual form)
    F30Ap60,
    /// jb2008: SET F10/S10/M10/Y10 + DSTDTC files (no manual form)
    Set,
}

/// `data.drivers(model, ..)` segmentation (env's method, `gen::swindex::drivers_for`); `None` for an unknown model
/// name.
pub fn drivers_for(model: &str) -> Option<DriverSet> {
    let m = crate::atmos::AtmosModel::from_name(model)?;
    Some(match swi::drivers_for(m.choice()) {
        swi::DRIVERSET_NO_DRIVERS => DriverSet::None,
        swi::DRIVERSET_F107_KP => DriverSet::F107Kp,
        swi::DRIVERSET_F30_AP60 => DriverSet::F30Ap60,
        _ => DriverSet::Set,
    })
}

// ---------------------------------------------------------------------------------
// Cached source formats
// ---------------------------------------------------------------------------------

/// Column names of an OMNI2 hourly record (`get_omni2`, 55 columns).
pub const OMNI2_NAMES: [&str; 55] = [
    "year", "doy", "hour", "bartels", "id_imf", "id_sw", "n_imf", "n_plasma", "B_mag_avg", "B_vec_mag", "B_lat", "B_long", "Bx_gse",
    "By_gse", "Bz_gse", "By_gsm", "Bz_gsm", "sigma_B_mag", "sigma_B", "sigma_Bx", "sigma_By", "sigma_Bz", "proton_temp",
    "proton_density", "flow_speed", "flow_long", "flow_lat", "na_np", "flow_pressure", "sigma_T", "sigma_N", "sigma_V",
    "sigma_phi_V", "sigma_theta_V", "sigma_na_np", "E_field", "plasma_beta", "alfven_mach", "Kp", "R_sunspot", "Dst", "AE",
    "pflux_gt1", "pflux_gt2", "pflux_gt4", "pflux_gt10", "pflux_gt30", "pflux_gt60", "flag", "ap", "f107", "pcn", "AL", "AU",
    "magnetosonic_mach",
];
/// Fill values of the OMNI2 columns (NaN = none).
const OMNI2_FILL: [f64; 55] = [
    f64::NAN, f64::NAN, f64::NAN, 9999.0, 99.0, 99.0, 999.0, 999.0, 999.9, 999.9, 999.9, 999.9, 999.9, 999.9, 999.9, 999.9, 999.9,
    999.9, 999.9, 999.9, 999.9, 999.9, 9999999.0, 999.9, 9999.0, 999.9, 999.9, 9.999, 99.99, 9999999.0, 999.9, 9999.0, 999.9,
    999.9, 9.999, 999.99, 999.99, 999.9, 99.0, 999.0, 99999.0, 9999.0, 999999.99, 99999.99, 99999.99, 99999.99, 99999.99,
    99999.99, f64::NAN, 999.0, 999.9, 999.9, 99999.0, 99999.0, 99.9,
];

/// `get_omni2` parse of an `omni2_YYYY.dat` text: rows of the 55 columns with fill
/// values -> NaN and Kp/10 (`keepFill = false`). Short rows are all-NaN (as MATLAB).
pub fn parse_omni2(text: &str) -> Vec<[f64; 55]> {
    let mut rows = Vec::new();
    let mut v = Vec::with_capacity(64);
    for line in text.split(['\n', '\r']) {
        if line.trim().is_empty() {
            continue;
        }
        v.clear();
        for tok in line.split_whitespace() {
            match tok.parse::<f64>() {
                Ok(x) => v.push(x),
                Err(_) => break,
            }
        }
        let mut row = [f64::NAN; 55];
        if v.len() >= 55 {
            row.copy_from_slice(&v[..55]);
            for k in 0..55 {
                if !OMNI2_FILL[k].is_nan() && row[k] == OMNI2_FILL[k] {
                    row[k] = f64::NAN;
                }
            }
            row[38] /= 10.0;
        }
        rows.push(row);
    }
    rows
}

/// One day of `get_f107`: F10.7, its centred mean and the number of samples in it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct F107Day {
    /// day [datenum, 00:00 UTC]
    pub t: f64,
    /// daily F10.7 [sfu]
    pub f107: f64,
    /// centred mean [sfu]
    pub f107_bar: f64,
    /// samples in the mean
    pub f107_bar_n: f64,
}

/// `get_f107` on already-parsed OMNI2 years: hourly F10.7 -> daily mean (NaN-omitting,
/// the `retime(...,'daily','mean')` step), sorted/deduplicated, then the centred
/// `window`-day mean with a shrinking window at the edges (`movmean(...,'omitnan',
/// 'Endpoints','shrink')`), cropped to `[d0, d1]` (datenum days).
pub fn f107_from_omni2(years: &[Vec<[f64; 55]>], window: usize, d0: f64, d1: f64) -> Vec<F107Day> {
    let mut daily: Vec<(f64, f64, usize)> = Vec::new(); // (day, sum, n) ...
    let mut all: Vec<(f64, f64)> = Vec::new();
    for rows in years {
        daily.clear();
        for r in rows {
            if !r[0].is_finite() || !r[1].is_finite() {
                continue;
            }
            let day = datenum(r[0], 1.0, 1.0, 0.0, 0.0, 0.0) + r[1] - 1.0;
            match daily.last_mut() {
                Some(last) if last.0 == day => {
                    if r[50].is_finite() {
                        last.1 += r[50];
                        last.2 += 1;
                    }
                }
                _ => daily.push((day, if r[50].is_finite() { r[50] } else { 0.0 }, usize::from(r[50].is_finite()))),
            }
        }
        for d in &daily {
            all.push((d.0, if d.2 > 0 { d.1 / d.2 as f64 } else { f64::NAN }));
        }
    }
    all.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    all.dedup_by(|a, b| a.0 == b.0);
    let n = all.len();
    let (back, fwd) = if window % 2 == 1 { ((window - 1) / 2, (window - 1) / 2) } else { (window / 2, window / 2 - 1) };
    let mut out = Vec::new();
    for i in 0..n {
        let lo = i.saturating_sub(back);
        let hi = (i + fwd).min(n - 1);
        let (mut s, mut c) = (0.0, 0usize);
        for x in &all[lo..=hi] {
            if !x.1.is_nan() {
                s += x.1;
                c += 1;
            }
        }
        let t = all[i].0;
        if t >= d0 && t <= d1 {
            out.push(F107Day { t, f107: all[i].1, f107_bar: if c > 0 { s / c as f64 } else { f64::NAN }, f107_bar_n: c as f64 });
        }
    }
    out
}

/// One GFZ index series (`get_gfz_hpo`): times [datenum] and values (-1 -> NaN).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GfzSeries {
    /// index name
    pub name: String,
    /// epochs [datenum]
    pub t: Vec<f64>,
    /// values
    pub v: Vec<f64>,
}

fn parse_iso_z(s: &str) -> Option<f64> {
    // yyyy-MM-ddTHH:mm:ssZ
    let b = s.trim();
    if b.len() < 19 {
        return None;
    }
    let num = |a: usize, e: usize| b.get(a..e).and_then(|x| x.parse::<f64>().ok());
    Some(datenum(num(0, 4)?, num(5, 7)?, num(8, 10)?, num(11, 13)?, num(14, 16)?, num(17, 19)?))
}

/// `get_gfz_hpo` decode of one `kp.gfz.de/app/json` response for index `idx`: the
/// field whose name contains "datetime" gives the times, the field named `idx`
/// (case-insensitive; else the first numeric array of the same length) the values.
pub fn parse_gfz_json(text: &str, idx: &str) -> Result<GfzSeries, SwError> {
    let j: serde_json::Value = serde_json::from_str(text).map_err(|e| SwError::Parse(e.to_string()))?;
    let obj = j.as_object().ok_or_else(|| SwError::Parse("GFZ JSON is not an object".into()))?;
    let tf = obj.keys().find(|k| k.to_ascii_lowercase().contains("datetime")).ok_or_else(|| SwError::Parse("no datetime field".into()))?;
    let times: Vec<f64> = obj[tf]
        .as_array()
        .ok_or_else(|| SwError::Parse("datetime is not an array".into()))?
        .iter()
        .map(|x| x.as_str().and_then(parse_iso_z).unwrap_or(f64::NAN))
        .collect();
    let num_arr = |v: &serde_json::Value| -> Option<Vec<f64>> {
        let a = v.as_array()?;
        let mut out = Vec::with_capacity(a.len());
        for x in a {
            out.push(x.as_f64()?);
        }
        Some(out)
    };
    let mut vals = obj.iter().find(|(k, _)| k.eq_ignore_ascii_case(idx)).and_then(|(_, v)| num_arr(v));
    if vals.is_none() {
        for (k, v) in obj.iter() {
            if k != tf {
                if let Some(a) = num_arr(v) {
                    if a.len() == times.len() {
                        vals = Some(a);
                        break;
                    }
                }
            }
        }
    }
    let mut v = vals.ok_or_else(|| SwError::Parse(format!("no values for {idx}")))?;
    for x in v.iter_mut() {
        if *x == -1.0 {
            *x = f64::NAN;
        }
    }
    Ok(GfzSeries { name: idx.to_string(), t: times, v })
}

/// `data.spaceweather`'s `mergeToStruct`: the F10.7 days and the GFZ series
/// (synchronised on the union of their times, as `synchronize(...,'union')`) folded
/// into the uniform daily [`SwTable`].
pub fn merge_to_table(f107: &[F107Day], mag: &[GfzSeries]) -> SwTable {
    let mut times: Vec<f64> = mag.iter().flat_map(|s| s.t.iter().copied()).filter(|t| t.is_finite()).collect();
    times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    times.dedup();
    let col = |name: &str| -> Option<Vec<f64>> {
        let s = mag.iter().find(|s| s.name == name)?;
        Some(times.iter().map(|t| s.t.iter().position(|x| x == t).map(|i| s.v[i]).unwrap_or(f64::NAN)).collect())
    };
    let (kp, ap, apd) = (col("Kp"), col("ap"), col("Ap"));
    let dayof = |t: f64| t.floor();
    let mut days: Vec<f64> = times.iter().map(|&t| dayof(t)).collect();
    days.dedup();
    let mut sw = SwTable { source: "OMNI2(F10.7)+GFZ(Kp/ap)".into(), ..Default::default() };
    for &d in &days {
        sw.mjd.push(d - 678942.0);
        let (mut fo, mut fc) = (f64::NAN, f64::NAN);
        if let Some(r) = f107.iter().find(|r| r.t == d) {
            fo = r.f107;
            fc = r.f107_bar;
        }
        sw.f107obs.push(fo);
        sw.f107c81.push(fc);
        let sel: Vec<usize> = (0..times.len()).filter(|&i| dayof(times[i]) == d).collect();
        let mean_omit = |v: &[f64]| {
            let (mut s, mut n) = (0.0, 0usize);
            for &x in sel.iter().map(|&i| &v[i]) {
                if !x.is_nan() {
                    s += x;
                    n += 1;
                }
            }
            if n > 0 { s / n as f64 } else { f64::NAN }
        };
        sw.kp.push(kp.as_ref().map(|v| mean_omit(v)).unwrap_or(f64::NAN));
        let mut apdaily = f64::NAN;
        if let Some(v) = &apd {
            if let Some(&i) = sel.iter().find(|&&i| !v[i].is_nan()) {
                apdaily = v[i];
            }
        }
        let mut row = [f64::NAN; 8];
        if let Some(v) = &ap {
            for (j, &i) in sel.iter().take(8).enumerate() {
                row[j] = v[i];
            }
            if apdaily.is_nan() {
                apdaily = mean_omit(v);
            }
        }
        sw.ap_daily.push(apdaily);
        sw.ap.push(row);
    }
    sw
}

/// One month of the NOAA predicted solar cycle (`parsePredicted`): mid-month datenum
/// and the predicted / high / low F10.7.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PredictedMonth {
    /// mid-month [datenum]
    pub t: f64,
    /// predicted F10.7
    pub predicted: f64,
    /// high band
    pub high: f64,
    /// low band
    pub low: f64,
}

/// `parsePredicted` of `predicted-solar-cycle.json` (sorted by time).
pub fn parse_predicted_json(text: &str) -> Result<Vec<PredictedMonth>, SwError> {
    let j: serde_json::Value = serde_json::from_str(text).map_err(|e| SwError::Parse(e.to_string()))?;
    let arr = j.as_array().ok_or_else(|| SwError::Parse("predicted-solar-cycle is not an array".into()))?;
    let num = |v: &serde_json::Value| -> f64 {
        match v {
            serde_json::Value::Number(n) => n.as_f64().unwrap_or(f64::NAN),
            serde_json::Value::String(s) => s.trim().parse::<f64>().unwrap_or(f64::NAN),
            _ => f64::NAN,
        }
    };
    let get = |o: &serde_json::Map<String, serde_json::Value>, names: &[&str]| -> Option<serde_json::Value> {
        for n in names {
            let nm = n.replace('-', "_");
            if let Some((_, v)) = o.iter().find(|(k, _)| k.replace('-', "_").eq_ignore_ascii_case(&nm)) {
                return Some(v.clone());
            }
        }
        None
    };
    let mut out = Vec::new();
    for e in arr {
        let o = e.as_object().ok_or_else(|| SwError::Parse("entry is not an object".into()))?;
        let tt = get(o, &["time_tag", "time-tag"]).and_then(|v| v.as_str().map(str::to_string)).unwrap_or_default();
        let y = tt.get(0..4).and_then(|x| x.parse::<f64>().ok()).unwrap_or(f64::NAN);
        let m = tt.get(5..7).and_then(|x| x.parse::<f64>().ok()).unwrap_or(f64::NAN);
        let p = get(o, &["predicted_f10_7", "predicted_f107", "f10_7"]).map(|v| num(&v)).unwrap_or(f64::NAN);
        let h = get(o, &["high_f10_7", "high_f107"]).map(|v| num(&v)).unwrap_or(p);
        let l = get(o, &["low_f10_7", "low_f107"]).map(|v| num(&v)).unwrap_or(p);
        out.push(PredictedMonth { t: datenum(y, m, 15.0, 0.0, 0.0, 0.0), predicted: p, high: h, low: l });
    }
    out.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap_or(std::cmp::Ordering::Equal));
    Ok(out)
}

/// `fetch45day` parse of the SWPC `45-day-forecast.txt`: lines `dd Mon yyyy F107 Ap`
/// -> (datenum, F10.7, Ap).
pub fn parse_45day(text: &str) -> Vec<(f64, f64, f64)> {
    const MON: [&str; 12] = ["jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec"];
    let mut out = Vec::new();
    for line in text.lines() {
        let t: Vec<&str> = line.split_whitespace().collect();
        if t.len() < 5 {
            continue;
        }
        let ok = t[0].len() <= 2 && t[0].bytes().all(|c| c.is_ascii_digit())
            && t[1].len() == 3
            && t[2].len() == 4 && t[2].bytes().all(|c| c.is_ascii_digit())
            && t[3].bytes().all(|c| c.is_ascii_digit())
            && t[4].bytes().all(|c| c.is_ascii_digit());
        if !ok {
            continue;
        }
        let Some(mi) = MON.iter().position(|m| m.eq_ignore_ascii_case(t[1])) else { continue };
        let (d, y): (f64, f64) = (t[0].parse().unwrap_or(f64::NAN), t[2].parse().unwrap_or(f64::NAN));
        out.push((datenum(y, (mi + 1) as f64, d, 0.0, 0.0, 0.0), t[3].parse().unwrap_or(f64::NAN), t[4].parse().unwrap_or(f64::NAN)));
    }
    out
}

/// Which F10.7 curve of the predicted cycle (`opts.band`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Band {
    /// 'predicted'
    Predicted,
    /// 'high'
    High,
    /// 'low'
    Low,
}

/// `data.spaceweather_forecast(startDate, endDate, opts)` from already-read sources:
/// `d0`/`d1` window [datenum], the monthly prediction, the optional 45-day forecast
/// (used when `d0 <= now + 45`), `ap_nominal` (default 15) beyond it.
pub fn forecast_table(d0: f64, d1: f64, predicted: &[PredictedMonth], band: Band, ap_nominal: f64, forty_five: Option<&[(f64, f64, f64)]>, now: f64) -> SwTable {
    let days: Vec<f64> = {
        let (a, b) = (d0.floor(), d1.ceil());
        let mut v = Vec::new();
        let mut x = a;
        while x <= b {
            v.push(x);
            x += 1.0;
        }
        v
    };
    let mt: Vec<f64> = predicted.iter().map(|p| p.t).collect();
    let mf: Vec<f64> = predicted
        .iter()
        .map(|p| match band {
            Band::High => p.high,
            Band::Low => p.low,
            Band::Predicted => p.predicted,
        })
        .collect();
    let mut sw = SwTable { source: String::new(), ..Default::default() };
    let mut ap_daily = vec![ap_nominal; days.len()];
    for &d in &days {
        sw.mjd.push(d - 678942.0);
        let f = if mt.len() >= 2 { interp1_linear_extrap(&mt, &mf, d) } else { f64::NAN };
        sw.f107obs.push(f);
        sw.f107c81.push(f);
    }
    if let Some(ff) = forty_five {
        if d0 <= now + 45.0 {
            for (k, &d) in days.iter().enumerate() {
                if let Some(r) = ff.iter().find(|r| r.0 == d) {
                    sw.f107obs[k] = r.1;
                    sw.f107c81[k] = r.1;
                    ap_daily[k] = r.2;
                }
            }
        }
    }
    sw.kp = ap_daily.iter().map(|&a| ap2kp_forecast(a)).collect();
    sw.ap = ap_daily.iter().map(|&a| [a; 8]).collect();
    sw.ap_daily = ap_daily;
    let b = match band {
        Band::Predicted => "predicted",
        Band::High => "high",
        Band::Low => "low",
    };
    sw.source = format!("NOAA/SWPC forecast (F10.7 {b} band, Ap nominal={ap_nominal})");
    sw
}

/// The hard-override path of `data.spaceweather_forecast` (`opts.f107` [+ `opts.ap`]).
pub fn forecast_override(d0: f64, d1: f64, f107: f64, ap: f64) -> SwTable {
    let mut sw = SwTable { source: "manual forecast override".into(), ..Default::default() };
    let mut x = d0.floor();
    while x <= d1.ceil() {
        sw.mjd.push(x - 678942.0);
        sw.f107obs.push(f107);
        sw.f107c81.push(f107);
        sw.ap_daily.push(ap);
        sw.ap.push([ap; 8]);
        sw.kp.push(ap2kp_forecast(ap));
        x += 1.0;
    }
    sw
}
