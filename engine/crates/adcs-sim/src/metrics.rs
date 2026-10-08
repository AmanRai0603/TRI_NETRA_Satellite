//! Derived channels and scenario metrics (asils.metrics.{derive,evaluate,time_to,window}): how each is measured is
//! kpi's (S7.14b: kpi_metric_channels, _statistics, _ecss, _evaluate), the power system design's, the jitter and the
//! pointing budget pnt's (S7.14), all generated into the engine (crate::gen); this file walks the record, reads the
//! scenario's metric sections and hands the channels over.
use crate::config::{Config, GUID, MODES};
use crate::json;
use crate::run::Record;
use adcs_sim_core::la::*;
use serde_json::{json, Value};

#[derive(Clone, Debug, Default)]
pub struct Derived { pub ape_3ax: Vec<f64>, pub ape_los: Vec<f64>, pub ake_3ax: Vec<f64>, pub ake_los: Vec<f64>, pub rate: Vec<f64>, pub rks: Vec<f64>, pub sun_angle: Vec<f64>, pub sun_angle_geo: Vec<f64>, pub spin_z: Vec<f64>, pub rate_err: Vec<f64>,
    /// the small-angle error vectors [rad, body]: performance (true reference to true attitude) and
    /// knowledge (estimate to true attitude), NaN where there is none; and the boresight they project on
    pub e_ape: Vec<V3>, pub e_ake: Vec<V3>, pub bs: V3,
    /// the power budget (`power`): array generation [W] and battery state of charge [0..1]; NaN when
    /// the case states no power system
    pub p_gen: Vec<f64>, pub soc: Vec<f64> }

/// The platform's power system as the case states it (section `power`): the illuminated cell area
/// on each body face [m^2], the cell-to-bus efficiency, the battery [Wh], the platform's own load
/// [W, the ADCS not included] and the state of charge at the start. All or none.
#[derive(Clone, Debug)]
pub struct PowerSystem { pub area: [f64; 6], pub eff: f64, pub batt_wh: f64, pub load_w: f64, pub soc0: f64 }

/// The case's power keys, in the order `PowerSystem::from` reads them.
pub const POWER_KEYS: [&str; 10] = ["power.area_px", "power.area_mx", "power.area_py", "power.area_my", "power.area_pz", "power.area_mz",
    "power.eff", "power.batt_wh", "power.load_w", "power.soc0"];

/// The solar constant at 1 AU [W/m^2]: design_power_system's (gen::powersys).
pub const SOLAR_CONSTANT: f64 = crate::gen::powersys::SOLAR_CONSTANT;

impl PowerSystem {
    /// The power system the case states, None when it states none (a partial statement is refused
    /// by `Config::check`).
    pub fn from(case: &crate::case::Case) -> Option<PowerSystem> {
        let v: Vec<f64> = POWER_KEYS.iter().map(|k| case.get(k)).collect();
        if !v.iter().all(|x| x.is_finite()) { return None; }
        Some(PowerSystem { area: [v[0], v[1], v[2], v[3], v[4], v[5]], eff: v[6], batt_wh: v[7], load_w: v[8], soc0: v[9] })
    }
    /// Array power [W] with the Sun along `s` (body, unit) and the shadow factor `nu` (1 sunlit):
    /// design_power_system's array_power (gen::powersys).
    pub fn generation(&self, s: &V3, nu: f64) -> f64 { crate::gen::powersys::array_power(self.area, self.eff, *s, nu) }
}

impl Derived {
    /// A derived channel by its asils.metrics.derive name.
    pub fn channel(&self, name: &str) -> Option<&Vec<f64>> {
        Some(match name {
            "ape_3ax" => &self.ape_3ax, "ape_los" => &self.ape_los, "ake_3ax" => &self.ake_3ax, "ake_los" => &self.ake_los,
            "rate" => &self.rate, "rks" => &self.rks, "sun_angle" => &self.sun_angle, "sun_angle_geo" => &self.sun_angle_geo,
            "spin_z" => &self.spin_z, "rate_err" => &self.rate_err, _ => return None,
        })
    }
}

/// The smallest eigenvalue of a symmetric 3x3 matrix: gp_4's (gen::jitter).
pub fn eig_min3(m: &[[f64; 3]; 3]) -> f64 { crate::gen::jitter::jitter_eig_min3(*m) }

/// Pointing jitter from rotor imbalance, frequency domain [arcsec, each sample of the window]: gp_4's
/// rotor_jitter (gen::jitter) over the recorded momenta, the body's smallest principal moment and lever
/// (jitter_body), the wheel loop's bandwidth the flight software flies. Zero with no rotor; empty (not
/// computed) when a rotor's part states no imbalance.
pub fn jitter(c: &Config, rec: &Record, idx: &[usize]) -> Vec<f64> {
    use crate::gen::jitter::{jitter_body, rotor_jitter, JIT_NR};
    let x = &c.dev.mex;
    if x.n == 0 { return idx.iter().map(|_| 0.0).collect(); }
    if c.dev.imbalance.iter().take(x.n).any(|u| u.is_none()) { return vec![]; }
    let (jmin, d) = jitter_body(c.inertia, c.box_m);
    let (mut jrot, mut us, mut ud) = ([0.0; JIT_NR as usize], [0.0; JIT_NR as usize], [0.0; JIT_NR as usize]);
    for i in 0..x.n {
        jrot[i] = x.jrot[i];
        (us[i], ud[i]) = c.dev.imbalance[i].unwrap_or((0.0, 0.0));
    }
    idx.iter().map(|&j| {
        let mut h = [0.0; JIT_NR as usize];
        h[..x.n].copy_from_slice(&rec.rows[j].h_w[..x.n]);
        rotor_jitter(x.n as i64, h, jrot, us, ud, jmin, d, c.rw_bandwidth)
    }).collect()
}

/// The ECSS-E-ST-60-10C relative, mean and drift indices (see `ecss`).
pub const ECSS_KINDS: [&str; 12] = ["rpe", "rpe_los", "mpe", "mpe_los", "pde", "pde_los", "rke", "rke_los", "mke", "mke_los", "kde", "kde_los"];

/// How each kind of metric is measured is kpi's (kpi_metric_channels, _statistics, _ecss, _evaluate: gen::kpichannels,
/// kpistats, kpiecss, kpimetrics). Their choices' options are the scenario's words, in the schema's order: a kind
/// (schema::METRIC_KINDS, then none), a window (schema::WINDOWS, then after_s), a statistic (schema::STATISTICS, then
/// none), a channel (schema::CHANNELS); a unit's option is shown as its label (UNITS).
const UNITS: [&str; 9] = ["deg", "min", "s", "deg/s", "N m s", "%", "g", "W", "arcsec"];

/// The option of a choice the word names: its place in `words`, else `other`.
fn option(words: &[&str], word: &str, other: i64) -> i64 { words.iter().position(|w| *w == word).map(|i| i as i64).unwrap_or(other) }

/// The mode a metric names (its place in MODES), or -1: no sample is in a mode that is not one.
fn mode_number(name: &str) -> i64 { option(&MODES, name, -1) }

/// The derived channels of a run, sample by sample: kpi_metric_channels's (gen::kpichannels: the reference the flight
/// software's guidance gives in each sample's mode, GUID its kind, with its yaw flip carried; the errors; the rate, the
/// Sun's angle, the spin and the rate stability), the three-axis angles with the engine's portable maths
/// (adcs-sim-core's generation of the same method), and the power system's (design_power_system: gen::powersys).
pub fn derive(c: &Config, rec: &Record) -> Derived {
    use crate::gen::kpichannels as kc;
    use adcs_sim_core::gen::kpichannels::kpi_three_axis;
    let n = rec.rows.len();
    let bs = c.dev.boresight;
    let p = &c.params;
    let mut d = Derived { ape_3ax: vec![f64::NAN; n], ape_los: vec![f64::NAN; n], ake_3ax: vec![f64::NAN; n], ake_los: vec![f64::NAN; n], rks: vec![f64::NAN; n],
                          rate: vec![f64::NAN; n], sun_angle: vec![f64::NAN; n], sun_angle_geo: vec![f64::NAN; n], spin_z: vec![f64::NAN; n], ..Default::default() };
    let mut e_vec = vec![[f64::NAN; 3]; n];
    d.e_ake = vec![[f64::NAN; 3]; n];
    d.bs = bs;
    let mut flip = false;
    for (j, row) in rec.rows.iter().enumerate() {
        flip = kc::kpi_yaw_flip(p.gd_yaw_flip != 0, row.r, row.v, p.gd_q_off, c.dev.sun_axis, bs, row.sun_eci, flip, p.gd_flip_hyst);
        let k = GUID.get(row.mode as usize).copied().unwrap_or(-1);
        let (has, qr) = kc::kpi_reference(k as i64, row.r, row.v, row.t, p.gd_q_off, p.gd_roll_deg, p.gd_t0, p.gd_T, p.gd_axis, p.gd_q_inertial,
                                          c.dev.sun_axis, bs, row.sun_eci, flip);
        if has {
            (e_vec[j], d.ape_los[j]) = kc::kpi_error(row.q, qr, bs);
            d.ape_3ax[j] = kpi_three_axis(row.q, qr);
        }
        if let Some(qe) = row.q_est {
            (d.e_ake[j], d.ake_los[j]) = kc::kpi_error(row.q, qe, bs);
            d.ake_3ax[j] = kpi_three_axis(row.q, qe);
        }
        (d.rate[j], d.spin_z[j]) = kc::kpi_rates(row.w);
        (d.sun_angle_geo[j], d.sun_angle[j]) = kc::kpi_sun_angle(c.dev.sun_axis, row.sun_body, row.nu);
    }
    d.rate_err = vec![f64::NAN; n];
    let lag = kc::kpi_rks_lag(c.record_dt) as usize;
    for j in lag..n { d.rks[j] = kc::kpi_rks(e_vec[j], e_vec[j - lag], lag as i64, c.record_dt); }
    d.e_ape = e_vec;
    d.p_gen = vec![f64::NAN; n];
    d.soc = vec![f64::NAN; n];
    if let Some(ps) = PowerSystem::from(&c.case) {
        // design_power_system (gen::powersys): the arrays, and the battery taking what they give less the load
        use crate::gen::powersys::{battery_soc, battery_start, battery_step, power_load, power_sun_unit};
        let mut e = battery_start(ps.soc0, ps.batt_wh);
        for (j, row) in rec.rows.iter().enumerate() {
            d.p_gen[j] = ps.generation(&power_sun_unit(row.sun_body), row.nu);
            if j > 0 {
                let load = power_load(ps.load_w, row.p_mtq, row.p_rw, row.p_rcs);
                e = battery_step(e, d.p_gen[j], load, rec.rows[j - 1].t, row.t, ps.batt_wh);
            }
            d.soc[j] = battery_soc(e, ps.batt_wh);
        }
    }
    d
}

fn usize_ix(idx: &[usize]) -> Vec<i64> { idx.iter().map(|&j| j as i64).collect() }

/// The ECSS-E-ST-60-10C error indices of `kind` (an ECSS kind; any other gives none) over the window's samples `idx`
/// (times `t`), in blocks of `delta_s` (a drift over `separation_s`), in degrees: kpi_metric_ecss's (gen::kpiecss), the
/// kind's index and error kpi_metric_evaluate's (kpi_ecss_kind). The metric's statistic is taken over them.
pub fn ecss(kind: &str, d: &Derived, t: &[f64], idx: &[usize], delta_s: f64, separation_s: f64) -> Vec<f64> {
    use crate::gen::{kpiecss as ke, kpimetrics as km};
    let (is, k, knowledge, los) = km::kpi_ecss_kind(option(&crate::schema::METRIC_KINDS, kind, km::METRICKIND_NONE));
    if !is { return vec![]; }
    let mut e: Vec<f64> = if knowledge { &d.e_ake } else { &d.e_ape }.iter().flat_map(|v| v.iter().copied()).collect();
    let (mut tt, mut ix) = (t.to_vec(), usize_ix(idx));
    let ni = ix.len() as i64;
    let nb = ke::kpi_ecss_blocks(&mut tt, &mut ix, ni, delta_s).max(0) as usize;
    let (mut sums, mut cnt, mut out) = (vec![0.0; 3*nb], vec![0i64; nb], vec![0.0; idx.len().max(nb)]);
    let nout = ke::kpi_ecss(k, los, &mut e, d.bs, &mut tt, &mut ix, ni, delta_s, separation_s, &mut sums, &mut cnt, &mut out);
    out.truncate(nout.max(0) as usize);
    out
}

/// The first time x stays below thr for hold seconds (NaN: never): kpi_metric_statistics's kpi_time_to.
pub fn time_to(t: &[f64], x: &[f64], thr: f64, hold: f64) -> f64 {
    crate::gen::kpistats::kpi_time_to(&mut t.to_vec(), &mut x.to_vec(), t.len() as i64, thr, hold)
}

/// The statistic `st` (a MetricStat) of the finite values among x: kpi_metric_statistics's kpi_statistic.
fn stat(mut x: Vec<f64>, st: i64) -> f64 {
    let n = x.len();
    let (mut w, mut tmp) = (vec![0.0; n], vec![0.0; n]);
    crate::gen::kpistats::kpi_statistic(&mut x, n as i64, st, &mut w, &mut tmp)
}

/// The samples of a run a metric's window (the scenario's word; after_s:<seconds>) takes: kpi_metric_statistics's
/// kpi_window over the times, the modes and the orbit's period.
fn window(c: &Config, t: &mut [f64], mode: &mut [i64], spec: &str) -> Vec<usize> {
    use crate::gen::kpistats as ks;
    let (w, after) = match spec.strip_prefix("after_s:") {
        Some(x) => (ks::METRICWINDOW_AFTER_S, x.parse::<f64>().unwrap_or(0.0)),
        None => (option(&crate::schema::WINDOWS, spec, ks::METRICWINDOW_ALL), 0.0),
    };
    let mut idx = vec![0i64; t.len()];
    let ni = ks::kpi_window(w, after, t, mode, t.len() as i64, c.period_s, &mut idx);
    idx[..ni.max(0) as usize].iter().map(|&j| j as usize).collect()
}

/// Every metric the scenario asks for, measured on the run (kpi_metric_evaluate: gen::kpimetrics): its value, the unit
/// it is stated in, and its verdict against the case's requirement or the scenario's limit. This reads the scenario's
/// metric sections (the defaults kpi's where a section states none) and hands the run's channels over.
pub fn evaluate(c: &Config, rec: &Record, d: &Derived) -> Vec<Value> {
    use crate::gen::{kpimetrics as km, kpistats as ks};
    use crate::schema::{CHANNELS, METRIC_KINDS, STATISTICS, WINDOWS};
    let mut out = vec![];
    let ms = c.scenario.get("metrics").and_then(|m| m.as_array()).cloned().unwrap_or_default();
    let (dw, dst, dch, rate_thr, rate_hold, thr, thr_hold, from_s, dsense_min, dunit_min, dend) = km::kpi_metric_defaults();
    let n = rec.rows.len();
    let col = |f: &dyn Fn(&crate::run::Row) -> f64| -> Vec<f64> { rec.rows.iter().map(f).collect() };
    let mut t = col(&|r| r.t);
    let mut mode: Vec<i64> = rec.rows.iter().map(|r| r.mode as i64).collect();
    let (mut prop, mut p_mtq, mut p_rw, mut p_rcs) = (col(&|r| r.prop_kg), col(&|r| r.p_mtq), col(&|r| r.p_rw), col(&|r| r.p_rcs));
    let mut h: Vec<f64> = rec.rows.iter().flat_map(|r| r.h_w[..rec.nr].iter().copied()).collect();
    // the channels, in MetricChannel's order
    let mut ch: Vec<Vec<f64>> = CHANNELS.iter().map(|name| d.channel(name).cloned().unwrap_or_default()).collect();
    let (mut sun, mut spin, mut p_gen, mut soc) = (d.sun_angle.clone(), d.spin_z.clone(), d.p_gen.clone(), d.soc.clone());
    let (mut w, mut tmp, mut tt, mut xx) = (vec![0.0; n], vec![0.0; n], vec![0.0; n], vec![0.0; n]);
    let ps = PowerSystem::from(&c.case);
    for m in &ms {
        let spec = json::get(m, "window").and_then(|x| x.as_str()).unwrap_or(WINDOWS[dw as usize]);
        let idx = window(c, &mut t, &mut mode, spec);
        let mut ix = usize_ix(&idx);
        let ni = ix.len() as i64;
        let st = match json::get(m, "statistic").and_then(|x| x.as_str()) { Some(s) => option(&STATISTICS, s, ks::METRICSTAT_NONE), None => dst };
        let kind = json::s(m, "kind", "");
        let k = option(&METRIC_KINDS, kind, km::METRICKIND_NONE);
        let unit_min = json::b(m, "unit_min", dunit_min);
        let mode_named = || mode_number(json::s(m, "mode", ""));
        let val = match k {
            km::METRICKIND_TIME_TO_RATE => km::kpi_time_to_rate(&mut t, &mut ch[km::METRICCHANNEL_RATE as usize], &mut mode, n as i64,
                json::f(m, "rate_threshold_deg_s", rate_thr), json::f(m, "hold_s", rate_hold), json::b(m, "end_at_mode_exit", dend)),
            km::METRICKIND_TIME_TO_THRESHOLD => {
                let name = json::s(m, "channel", CHANNELS[dch as usize]);
                let Some(c_ix) = CHANNELS.iter().position(|x| *x == name) else {
                    out.push(json!({"id": json::s(m, "id", ""), "kind": kind, "value": Value::Null, "unit": "", "req": Value::Null, "req_key": format!("unknown channel {name}"), "pass": Value::Null}));
                    continue;
                };
                km::kpi_time_to_threshold(&mut t, &mut ch[c_ix], n as i64, json::f(m, "from_s", from_s), json::f(m, "threshold_deg", thr),
                    json::f(m, "hold_s", thr_hold), unit_min, &mut tt, &mut xx)
            }
            km::METRICKIND_WHEEL_MOMENTUM_PEAK => km::kpi_wheel_peak(&mut h, rec.nr as i64, &mut ix, ni),
            km::METRICKIND_TIME_TO_MODE => km::kpi_time_to_mode(&mut t, &mut mode, n as i64, mode_named()),
            km::METRICKIND_SUN_ANGLE => {
                let by = json::get(m, "mode").and_then(|x| x.as_str());
                km::kpi_sun_angle_stat(&mut sun, &mut mode, &mut ix, ni, by.is_some(), by.map(mode_number).unwrap_or(-1), st, &mut w, &mut tmp)
            }
            km::METRICKIND_SPIN_RATE_ERROR => km::kpi_spin_error_stat(&mut spin, &mut ix, ni, c.spin_dps, st, &mut w, &mut tmp),
            km::METRICKIND_MODE_FRACTION => km::kpi_mode_fraction(&mut mode, &mut ix, ni, mode_named()),
            km::METRICKIND_PROPELLANT => km::kpi_propellant(&mut prop, n as i64),
            km::METRICKIND_JITTER => stat(jitter(c, rec, &idx), st),
            km::METRICKIND_POWER_MARGIN => km::kpi_power_margin(ps.is_some(), &mut p_gen, &mut p_mtq, &mut p_rw, &mut p_rcs, &mut ix, ni,
                ps.as_ref().map(|p| p.load_w).unwrap_or(f64::NAN)),
            km::METRICKIND_BATTERY_DOD => km::kpi_battery(&mut soc, &mut ix, ni).0,
            km::METRICKIND_SOC_MIN => km::kpi_battery(&mut soc, &mut ix, ni).1,
            km::METRICKIND_POWER_MEAN => km::kpi_adcs_power(&mut p_mtq, &mut p_rw, &mut p_rcs, &mut ix, ni).0,
            km::METRICKIND_POWER_PEAK => km::kpi_adcs_power(&mut p_mtq, &mut p_rw, &mut p_rcs, &mut ix, ni).1,
            _ => match (km::kpi_kind_channel(k), km::kpi_ecss_kind(k).0) {
                ((true, c_ix), _) => ks::kpi_channel_stat(&mut ch[c_ix as usize], &mut ix, ni, st, &mut w, &mut tmp),
                (_, true) => stat(ecss(kind, d, &t, &idx, json::f(m, "delta_s", f64::NAN), json::f(m, "separation_s", f64::NAN)), st),
                _ => f64::NAN,
            },
        };
        let unit = UNITS[km::kpi_metric_unit(k, unit_min) as usize];
        // which requirement: the case's (a requirement key) or the scenario's own limit
        let mut rk = json::s(m, "requirement", "").to_string();
        let mut rv = f64::NAN;
        if !rk.is_empty() { rv = c.case.get(&rk); } else if let Some(l) = json::get(m, "limit").and_then(|x| x.as_f64()) { rv = l; rk = "scenario".into(); }
        let sense_min = json::get(m, "sense").and_then(|x| x.as_str()).map(|s| s == "min").unwrap_or(dsense_min);
        let pass = match km::kpi_verdict(val, rv, sense_min) { -1 => Value::Null, p => json!(p) };
        let num = |x: f64| if x.is_finite() { json!(x) } else { Value::Null };
        out.push(json!({"id": json::s(m, "id", ""), "kind": kind, "value": num(val), "unit": unit, "req": num(rv), "req_key": rk, "pass": pass}));
    }
    out
}

/// The absolute pointing error budget of a stored fine-pointing run (SPEC rows gp_0 to gp_5, ECSS-E-ST-60-10C), as
/// pnt's methods give it (gen::pntbudget): the knowledge (gp_0) and control (gp_1) contributions from the run's
/// metrics `flown` (the APE across the boresight, p99.73) and `knowledge` (the AKE likewise), the alignment (gp_2) the
/// product states (payload_alignment_rad), the thermal distortion (gp_3) the case states (pointing.et), the jitter
/// (gp_4) from the metric `jitter` [arcsec]; the total, the room req.ape leaves for gp_2 and gp_3, and the verdict.
/// A metric not named, or not computed, and a value not stated, are null. Degrees. Read by tools/pointing_budget.py.
pub fn pointing_budget(dir: &std::path::Path, flown: Option<&str>, knowledge: Option<&str>, jitter: Option<&str>) -> Result<Value, crate::Error> {
    use crate::gen::pntbudget as pb;
    let f = dir.join("manifest.json");
    let man: Value = serde_json::from_str(&crate::source::read_to_string(&f)?).map_err(|e| crate::Error::malformed(format!("{}: {e}", f.display())))?;
    let ms = man["metrics"].as_array().cloned().unwrap_or_default();
    let metric = |id: Option<&str>| id.and_then(|id| ms.iter().find(|m| m["id"].as_str() == Some(id))).and_then(|m| m["value"].as_f64()).unwrap_or(f64::NAN);
    let root = crate::data_root();
    let case = crate::case::Case::read(&crate::flight::case_file(&root, json::s(&man, "scenario", ""), Some(json::s(&man, "case", "")))?)?;
    let product: Value = serde_json::from_str(&crate::source::read_to_string(&crate::product::find(&root, "products", json::s(&man, "product", ""))?)?)
        .map_err(|e| crate::Error::malformed(format!("product {}: {e}", json::s(&man, "product", ""))))?;
    let (ape, ake) = (metric(flown), metric(knowledge));
    let terms = [pb::pointing_knowledge(ake), pb::pointing_control(ape, ake), pb::pointing_alignment(json::f(&product, "payload_alignment_rad", f64::NAN)),
                 case.get("pointing.et"), pb::pointing_jitter(metric(jitter))];
    let req = case.get("req.ape");
    let (total, room, verdict) = pb::pointing_budget(ape, terms[2], terms[3], terms[4], req);
    let num = |x: f64| if x.is_finite() { json!(x) } else { Value::Null };
    let verdict = match verdict { pb::BUDGETVERDICT_INCOMPLETE => "incomplete", pb::BUDGETVERDICT_CLOSES => "closes", _ => "does not close" };
    Ok(json!({"scenario": json::s(&man, "scenario", ""), "case": case.id, "product": json::s(&man, "product", ""), "flown_ape_deg": num(ape),
              "terms_deg": {"gp_0": num(terms[0]), "gp_1": num(terms[1]), "gp_2": num(terms[2]), "gp_3": num(terms[3]), "gp_4": num(terms[4])},
              "total_deg": num(total), "req_ape_deg": num(req), "room_gp2_gp3_deg": num(room), "verdict": verdict}))
}
