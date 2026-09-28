//! Derived channels and scenario metrics (asils.metrics.{derive,evaluate,time_to,window}).
use crate::config::{Config, GUID, MODES};
use crate::json;
use crate::run::Record;
use adcs_fsw::ctl::{guidance, Guid};
use adcs_sim_core::la::*;
use serde_json::{json, Value};

#[derive(Clone, Debug, Default)]
pub struct Derived { pub ape_3ax: Vec<f64>, pub ape_los: Vec<f64>, pub ake_3ax: Vec<f64>, pub ake_los: Vec<f64>, pub rate: Vec<f64>, pub rks: Vec<f64>, pub sun_angle: Vec<f64>, pub sun_angle_geo: Vec<f64>, pub spin_z: Vec<f64>, pub rate_err: Vec<f64> }

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

fn acosd(x: f64) -> f64 { x.clamp(-1.0, 1.0).acos().to_degrees() }

pub fn derive(c: &Config, rec: &Record) -> Derived {
    let n = rec.rows.len();
    let bs = c.dev.boresight;
    let p = &c.params;
    let mut d = Derived { ape_3ax: vec![f64::NAN; n], ape_los: vec![f64::NAN; n], ake_3ax: vec![f64::NAN; n], ake_los: vec![f64::NAN; n], rks: vec![f64::NAN; n], ..Default::default() };
    let mut e_vec = vec![[f64::NAN; 3]; n];
    for (j, row) in rec.rows.iter().enumerate() {
        let b_true = mtv(&dcm(&row.q), &bs);
        let k = GUID.get(row.mode as usize).copied().unwrap_or(-1);
        if k >= 0 {
            let gd = Guid { q_off: p.gd_q_off, roll_deg: p.gd_roll_deg, t0: p.gd_t0, t_slew: p.gd_T, axis: p.gd_axis, q_inertial: p.gd_q_inertial,
                sun_axis: c.dev.sun_axis, roll_axis: bs, sun_eci: row.sun_eci };
            let qr = guidance(k, &row.r, &row.v, row.t, &gd).q;
            let mut dq = qmult(&qconj(&qr), &row.q);
            if dq[3] < 0.0 { dq = [-dq[0], -dq[1], -dq[2], -dq[3]]; }
            e_vec[j] = [2.0*dq[0], 2.0*dq[1], 2.0*dq[2]];
            d.ape_3ax[j] = qangle(&row.q, &qr).to_degrees();
            d.ape_los[j] = acosd(dot(&b_true, &mtv(&dcm(&qr), &bs)));
        }
        if let Some(qe) = row.q_est {
            d.ake_3ax[j] = qangle(&row.q, &qe).to_degrees();
            d.ake_los[j] = acosd(dot(&b_true, &mtv(&dcm(&qe), &bs)));
        }
    }
    d.rate = rec.rows.iter().map(|r| norm(&r.w).to_degrees()).collect();
    d.sun_angle_geo = rec.rows.iter().map(|r| acosd(dot(&c.dev.sun_axis, &r.sun_body))).collect();
    d.sun_angle = rec.rows.iter().zip(&d.sun_angle_geo).map(|(r, &a)| if r.nu < 0.5 { f64::NAN } else { a }).collect();
    d.rate_err = vec![f64::NAN; n];
    d.spin_z = rec.rows.iter().map(|r| r.w[2].to_degrees()).collect();
    let lag = ((1.0/c.record_dt.max(1e-9)).round() as usize).max(1);
    for j in lag..n {
        let dv = sub(&e_vec[j], &e_vec[j - lag]);
        d.rks[j] = norm(&dv).to_degrees()/(lag as f64*c.record_dt);
    }
    d
}

/// The first time x stays below thr for hold seconds (NaN: never).
pub fn time_to(t: &[f64], x: &[f64], thr: f64, hold: f64) -> f64 {
    let n = t.len();
    let mut k = 0;
    while k < n {
        if x[k] < thr {
            let mut j = k;
            while j + 1 < n && x[j + 1] < thr { j += 1; }
            if t[j] - t[k] >= hold { return t[k]; }
            k = j + 1;
        } else { k += 1; }
    }
    f64::NAN
}

fn stat(x: impl Iterator<Item = f64>, s: &str) -> f64 {
    let mut v: Vec<f64> = x.filter(|x| x.is_finite()).collect();
    if v.is_empty() { return f64::NAN; }
    match s {
        "max" => v.iter().cloned().fold(f64::MIN, f64::max),
        "rms" => (v.iter().map(|x| x*x).sum::<f64>()/v.len() as f64).sqrt(),
        "mean" => v.iter().sum::<f64>()/v.len() as f64,
        "p99.73" | "p95" => {
            v.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let q = if s == "p95" { 0.95 } else { 0.9973 };
            v[((q*v.len() as f64).ceil() as usize).max(1) - 1]
        }
        _ => f64::NAN,
    }
}

fn window(c: &Config, rec: &Record, spec: &str) -> Vec<usize> {
    let t: Vec<f64> = rec.rows.iter().map(|r| r.t).collect();
    let te = *t.last().unwrap_or(&0.0);
    let per = c.period_s;
    (0..t.len()).filter(|&j| match spec {
        "all" => true,
        "last_orbit" => t[j] >= te - per,
        "last_half_orbit" => t[j] >= te - per/2.0,
        "pointing" => rec.rows[j].mode + 1 > 1,
        s if s.starts_with("after_s:") => t[j] >= s[8..].parse::<f64>().unwrap_or(0.0),
        _ => true,
    }).collect()
}

pub fn evaluate(c: &Config, rec: &Record, d: &Derived) -> Vec<Value> {
    let mut out = vec![];
    let ms = c.scenario.get("metrics").and_then(|m| m.as_array()).cloned().unwrap_or_default();
    let t: Vec<f64> = rec.rows.iter().map(|r| r.t).collect();
    for m in &ms {
        let idx = window(c, rec, json::s(m, "window", "all"));
        let st = json::s(m, "statistic", "max");
        let pick = |x: &Vec<f64>| idx.iter().map(|&j| x[j]).collect::<Vec<_>>();
        let mode_is = |j: usize, name: &str| MODES.get(rec.rows[j].mode as usize).map(|&x| x == name).unwrap_or(false);
        let kind = json::s(m, "kind", "");
        let mut unit = "deg";
        let val = match kind {
            "time_to_rate" => { unit = "min"; time_to(&t, &d.rate, json::f(m, "rate_threshold_deg_s", 0.5), json::f(m, "hold_s", 0.0))/60.0 }
            "ape" => stat(pick(&d.ape_3ax).into_iter(), st),
            "ape_los" => stat(pick(&d.ape_los).into_iter(), st),
            "ake" => stat(pick(&d.ake_3ax).into_iter(), st),
            "ake_los" => stat(pick(&d.ake_los).into_iter(), st),
            "rate_stability" => { unit = "deg/s"; stat(pick(&d.rks).into_iter(), st) }
            "time_to_threshold" => {
                unit = "s";
                let t0 = json::f(m, "from_s", 0.0);
                let name = json::s(m, "channel", "ape_los");
                let Some(ch) = d.channel(name) else { out.push(json!({"id": json::s(m, "id", ""), "kind": kind, "value": Value::Null, "unit": "", "req": Value::Null, "req_key": format!("unknown channel {name}"), "pass": Value::Null})); continue; };
                let ks: Vec<usize> = (0..t.len()).filter(|&j| t[j] >= t0).collect();
                let tt: Vec<f64> = ks.iter().map(|&j| t[j] - t0).collect();
                let xx: Vec<f64> = ks.iter().map(|&j| ch[j]).collect();
                let mut v = time_to(&tt, &xx, json::f(m, "threshold_deg", 1.0), json::f(m, "hold_s", 10.0));
                if json::b(m, "unit_min", false) { v /= 60.0; unit = "min"; }
                v
            }
            "wheel_momentum_peak" => { unit = "N m s"; idx.iter().flat_map(|&j| rec.rows[j].h_w[..rec.nr].iter().map(|x| x.abs())).fold(f64::NAN, f64::max) }
            "time_to_mode" => { unit = "min"; let name = json::s(m, "mode", ""); (0..t.len()).find(|&j| mode_is(j, name)).map(|j| t[j]/60.0).unwrap_or(f64::NAN) }
            "sun_angle" => {
                let sel: Vec<usize> = match json::get(m, "mode").and_then(|x| x.as_str()) { Some(name) => idx.iter().copied().filter(|&j| mode_is(j, name)).collect(), None => idx.clone() };
                stat(sel.iter().map(|&j| d.sun_angle[j]), st)
            }
            "spin_rate_error" => { unit = "deg/s"; stat(idx.iter().map(|&j| (d.spin_z[j].abs() - c.spin_dps).abs()), st) }
            "mode_fraction" => { unit = "%"; let name = json::s(m, "mode", ""); 100.0*idx.iter().filter(|&&j| mode_is(j, name)).count() as f64/idx.len().max(1) as f64 }
            "propellant" => { unit = "g"; 1e3*rec.rows.iter().map(|r| r.prop_kg).fold(0.0, f64::max) }
            "jitter" => { unit = "arcsec"; f64::NAN }
            "power_mean" => { unit = "W"; idx.iter().map(|&j| { let r = &rec.rows[j]; r.p_mtq + r.p_rw + r.p_rcs }).sum::<f64>()/idx.len().max(1) as f64 }
            "power_peak" => { unit = "W"; idx.iter().map(|&j| { let r = &rec.rows[j]; r.p_mtq + r.p_rw + r.p_rcs }).fold(f64::MIN, f64::max) }
            _ => f64::NAN,
        };
        let mut rk = json::s(m, "requirement", "").to_string();
        let mut rv = f64::NAN;
        if !rk.is_empty() { rv = c.case.get(&rk); } else if let Some(l) = json::get(m, "limit").and_then(|x| x.as_f64()) { rv = l; rk = "scenario".into(); }
        let pass = if rv.is_finite() {
            if val.is_nan() { Some(0) } else if json::s(m, "sense", "max") == "min" { Some((val >= rv) as i32) } else { Some((val <= rv) as i32) }
        } else { None };
        let num = |x: f64| if x.is_finite() { json!(x) } else { Value::Null };
        out.push(json!({"id": json::s(m, "id", ""), "kind": kind, "value": num(val), "unit": unit, "req": num(rv), "req_key": rk, "pass": pass}));
    }
    out
}
