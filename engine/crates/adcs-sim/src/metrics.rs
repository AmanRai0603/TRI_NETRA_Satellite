//! Derived channels and scenario metrics (asils.metrics.{derive,evaluate,time_to,window}).
use crate::config::{Config, GUID, MODES};
use crate::json;
use crate::run::Record;
use adcs_fsw::guid::{guidance, yaw_flip, Guid};
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

/// The solar constant at 1 AU [W/m^2] (IAU 2015 B3 nominal); the +/-3.3 % of the Earth's
/// eccentric orbit is not modelled.
pub const SOLAR_CONSTANT: f64 = 1361.0;

impl PowerSystem {
    /// The power system the case states, None when it states none (a partial statement is refused
    /// by `Config::check`).
    pub fn from(case: &crate::case::Case) -> Option<PowerSystem> {
        let v: Vec<f64> = POWER_KEYS.iter().map(|k| case.get(k)).collect();
        if !v.iter().all(|x| x.is_finite()) { return None; }
        Some(PowerSystem { area: [v[0], v[1], v[2], v[3], v[4], v[5]], eff: v[6], batt_wh: v[7], load_w: v[8], soc0: v[9] })
    }
    /// Array power [W] with the Sun along `s` (body, unit) and the shadow factor `nu` (1 sunlit).
    pub fn generation(&self, s: &V3, nu: f64) -> f64 {
        let n = [[1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, -1.0, 0.0], [0.0, 0.0, 1.0], [0.0, 0.0, -1.0]];
        let lit: f64 = (0..6).map(|f| self.area[f]*dot(&n[f], s).max(0.0)).sum();
        SOLAR_CONSTANT*nu.clamp(0.0, 1.0)*self.eff*lit
    }
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

/// The smallest eigenvalue of a symmetric 3x3 matrix (the trigonometric closed form).
pub fn eig_min3(m: &[[f64; 3]; 3]) -> f64 {
    let p1 = m[0][1]*m[0][1] + m[0][2]*m[0][2] + m[1][2]*m[1][2];
    let q = (m[0][0] + m[1][1] + m[2][2])/3.0;
    if p1 == 0.0 { return m[0][0].min(m[1][1]).min(m[2][2]); }
    let p2 = (m[0][0] - q).powi(2) + (m[1][1] - q).powi(2) + (m[2][2] - q).powi(2) + 2.0*p1;
    let p = (p2/6.0).sqrt();
    let b = |i: usize, j: usize| (m[i][j] - if i == j { q } else { 0.0 })/p;
    let det = b(0, 0)*(b(1, 1)*b(2, 2) - b(1, 2)*b(2, 1)) - b(0, 1)*(b(1, 0)*b(2, 2) - b(1, 2)*b(2, 0)) + b(0, 2)*(b(1, 0)*b(2, 1) - b(1, 1)*b(2, 0));
    let phi = (det/2.0).clamp(-1.0, 1.0).acos()/3.0;
    q + 2.0*p*(phi + 2.0*std::f64::consts::PI/3.0).cos()
}

/// Pointing jitter from rotor imbalance, frequency domain (= asils.sizing.jitter) [arcsec, each
/// sample of the window]. A rotor at speed W with static imbalance U_s at a lever d from the centre
/// of mass and dynamic imbalance U_d applies (U_s d + U_d) W^2 at W; above the attitude loop's
/// bandwidth w_bw the body answers as a free rigid body, below it the loop rejects it:
///     theta = (U_s d + U_d) W^2 / (J_min max(W^2, w_bw^2))
/// W = |h| / J_rotor from the recorded momentum; the rotors add root-sum-square; fluid rings carry
/// no rotating mass. d is half the class's smallest cross-section (the wheels sit inside it), w_bw
/// the scenario's wheel-loop bandwidth. Empty (not computed) when a rotor's part states no imbalance.
pub fn jitter(c: &Config, rec: &Record, idx: &[usize]) -> Vec<f64> {
    let x = &c.dev.mex;
    if x.n == 0 { return idx.iter().map(|_| 0.0).collect(); }
    if c.dev.imbalance.iter().take(x.n).any(|u| u.is_none()) { return vec![]; }
    let jmin = eig_min3(&c.inertia);
    let d = c.box_m.iter().cloned().fold(f64::INFINITY, f64::min)/2.0;
    let wbw = json::f(&c.scenario["fsw"], "rw_bandwidth", 0.9);
    idx.iter().map(|&j| {
        let th2: f64 = (0..x.n).map(|i| {
            let (us, ud) = c.dev.imbalance[i].unwrap_or((0.0, 0.0));
            if (us == 0.0 && ud == 0.0) || !(x.jrot[i] > 0.0) { return 0.0; }
            let w = rec.rows[j].h_w[i].abs()/x.jrot[i];
            let w = if w.is_finite() { w } else { 0.0 };
            let th = (us*d + ud)*w*w/(jmin*(w*w).max(wbw*wbw));
            th*th
        }).sum();
        th2.sqrt().to_degrees()*3600.0
    }).collect()
}

/// The ECSS-E-ST-60-10C relative, mean and drift indices (see `ecss`).
pub const ECSS_KINDS: [&str; 12] = ["rpe", "rpe_los", "mpe", "mpe_los", "pde", "pde_los", "rke", "rke_los", "mke", "mke_los", "kde", "kde_los"];

fn acosd(x: f64) -> f64 { x.clamp(-1.0, 1.0).acos().to_degrees() }

pub fn derive(c: &Config, rec: &Record) -> Derived {
    let n = rec.rows.len();
    let bs = c.dev.boresight;
    let p = &c.params;
    let mut d = Derived { ape_3ax: vec![f64::NAN; n], ape_los: vec![f64::NAN; n], ake_3ax: vec![f64::NAN; n], ake_los: vec![f64::NAN; n], rks: vec![f64::NAN; n], ..Default::default() };
    let mut e_vec = vec![[f64::NAN; 3]; n];
    d.e_ake = vec![[f64::NAN; 3]; n];
    d.bs = bs;
    let mut flip = false;                      // the nadir-family yaw flip, with the flight software's hysteresis
    for (j, row) in rec.rows.iter().enumerate() {
        if p.gd_yaw_flip != 0 {
            let mut g = Guid { q_off: p.gd_q_off, sun_axis: c.dev.sun_axis, roll_axis: bs, sun_eci: row.sun_eci, flip, ..Default::default() };
            yaw_flip(&mut g, &row.r, &row.v, p.gd_flip_hyst);
            flip = g.flip;
        }
        let b_true = mtv(&dcm(&row.q), &bs);
        let k = GUID.get(row.mode as usize).copied().unwrap_or(-1);
        if k >= 0 {
            let gd = Guid { q_off: p.gd_q_off, roll_deg: p.gd_roll_deg, t0: p.gd_t0, t_slew: p.gd_T, axis: p.gd_axis, q_inertial: p.gd_q_inertial,
                sun_axis: c.dev.sun_axis, roll_axis: bs, sun_eci: row.sun_eci, flip };
            let qr = guidance(k, &row.r, &row.v, row.t, &gd).q;
            let mut dq = qmult(&qconj(&qr), &row.q);
            if dq[3] < 0.0 { dq = [-dq[0], -dq[1], -dq[2], -dq[3]]; }
            e_vec[j] = [2.0*dq[0], 2.0*dq[1], 2.0*dq[2]];
            d.ape_3ax[j] = qangle(&row.q, &qr).to_degrees();
            d.ape_los[j] = acosd(dot(&b_true, &mtv(&dcm(&qr), &bs)));
        }
        if let Some(qe) = row.q_est {
            let mut dk = qmult(&qconj(&qe), &row.q);
            if dk[3] < 0.0 { dk = [-dk[0], -dk[1], -dk[2], -dk[3]]; }
            d.e_ake[j] = [2.0*dk[0], 2.0*dk[1], 2.0*dk[2]];
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
    d.e_ape = e_vec;
    d.p_gen = vec![f64::NAN; n];
    d.soc = vec![f64::NAN; n];
    if let Some(ps) = PowerSystem::from(&c.case) {
        // the battery takes what the arrays give less the platform and the ADCS, up to full; empty is empty
        let mut e = ps.soc0*ps.batt_wh;
        for (j, row) in rec.rows.iter().enumerate() {
            d.p_gen[j] = ps.generation(&unit(&row.sun_body), row.nu);
            if j > 0 {
                let dt_h = (row.t - rec.rows[j - 1].t)/3600.0;
                let load = ps.load_w + row.p_mtq + row.p_rw + row.p_rcs;
                e = (e + (d.p_gen[j] - load)*dt_h).clamp(0.0, ps.batt_wh);
            }
            d.soc[j] = e/ps.batt_wh;
        }
    }
    d
}

/// The ECSS-E-ST-60-10C error indices over a metric window, on the performance error (rpe, mpe,
/// pde) or the knowledge error (rke, mke, kde), three-axis or across the boresight (`_los`). The
/// window is cut into consecutive blocks of `delta_s` from its first sample:
///   rpe / rke   |e(t) - mean of e over t's block|, every sample          (relative error)
///   mpe / mke   |mean of e over a block|, every block                    (mean error)
///   pde / kde   |mean over block k+s - mean over block k|, s = separation_s / delta_s   (drift)
/// The values are in degrees; the metric's statistic is taken over them. A block with no finite
/// sample is skipped; a drift needs both of its blocks.
pub fn ecss(kind: &str, d: &Derived, t: &[f64], idx: &[usize], delta_s: f64, separation_s: f64) -> Vec<f64> {
    let (base, los) = match kind.strip_suffix("_los") { Some(b) => (b, true), None => (kind, false) };
    let src = if matches!(base, "rke" | "mke" | "kde") { &d.e_ake } else { &d.e_ape };
    let bs = unit(&d.bs);
    let e = |j: usize| -> V3 { let v = src[j]; if los { sub(&v, &scale(&bs, dot(&v, &bs))) } else { v } };
    if idx.is_empty() || !(delta_s > 0.0) { return vec![]; }
    let t0 = t[idx[0]];
    let block = |j: usize| ((t[j] - t0)/delta_s + 1e-9).floor() as usize;
    let nb = block(*idx.last().unwrap()) + 1;
    let (mut sum, mut cnt) = (vec![[0.0; 3]; nb], vec![0usize; nb]);
    for &j in idx {
        let v = e(j);
        if v.iter().all(|x| x.is_finite()) { let b = block(j); sum[b] = add(&sum[b], &v); cnt[b] += 1; }
    }
    let mean: Vec<Option<V3>> = (0..nb).map(|b| (cnt[b] > 0).then(|| scale(&sum[b], 1.0/cnt[b] as f64))).collect();
    match base {
        "rpe" | "rke" => idx.iter().filter_map(|&j| { let v = e(j); mean[block(j)].filter(|_| v.iter().all(|x| x.is_finite())).map(|m| norm(&sub(&v, &m)).to_degrees()) }).collect(),
        "mpe" | "mke" => mean.iter().flatten().map(|m| norm(m).to_degrees()).collect(),
        "pde" | "kde" => {
            let s = (separation_s/delta_s).round().max(1.0) as usize;
            (0..nb.saturating_sub(s)).filter_map(|k| match (mean[k], mean[k + s]) { (Some(a), Some(b)) => Some(norm(&sub(&b, &a)).to_degrees()), _ => None }).collect()
        }
        _ => vec![],
    }
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
            "time_to_rate" => {
                unit = "min";
                let thr = json::f(m, "rate_threshold_deg_s", 0.5);
                let mut v = time_to(&t, &d.rate, thr, json::f(m, "hold_s", 0.0));
                // a mission hands over to the next mode when detumble is done; that hand-over (below the
                // threshold) completes the detumble even if the next mode then spins the body up again
                if json::b(m, "end_at_mode_exit", false) && !rec.rows.is_empty() {
                    let m0 = rec.rows[0].mode;
                    if let Some(j) = (0..t.len()).find(|&j| rec.rows[j].mode != m0) {
                        if d.rate[j] < thr && !(v <= t[j]) { v = t[j]; }
                    }
                }
                v/60.0
            }
            k if ECSS_KINDS.contains(&k) => stat(ecss(k, d, &t, &idx, json::f(m, "delta_s", f64::NAN), json::f(m, "separation_s", f64::NAN)).into_iter(), st),
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
            "jitter" => { unit = "arcsec"; stat(jitter(c, rec, &idx).into_iter(), st) }
            // the power budget (needs the case's power system): array power less every load, averaged
            // over the window; the deepest discharge; the lowest state of charge
            "power_margin" => {
                unit = "W";
                let ps = PowerSystem::from(&c.case);
                ps.map(|ps| idx.iter().map(|&j| { let r = &rec.rows[j]; d.p_gen[j] - ps.load_w - r.p_mtq - r.p_rw - r.p_rcs }).sum::<f64>()/idx.len().max(1) as f64)
                    .unwrap_or(f64::NAN)
            }
            "battery_dod" => { unit = "%"; 100.0*(1.0 - idx.iter().map(|&j| d.soc[j]).fold(f64::NAN, f64::min)) }
            "soc_min" => { unit = "%"; 100.0*idx.iter().map(|&j| d.soc[j]).fold(f64::NAN, f64::min) }
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
