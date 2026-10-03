//! The design node of the pipeline (docs/DESIGN_LOOP.md): what the case asks of an actuator
//! (demand survey on the POP orbit, the SILS torque models), every actuator option sized to it
//! (ours: MTQ, fluid loop, N2O RCS, designed; benchmarks: RW, CMG, VSCMG, chosen from the datasheet
//! catalogue), one product per family and
//! its mass / power / volume budget. A port of matlab_sils/+asils/+sizing (demand, mtq, rw,
//! cmg, fmr, rcs, size_all) with the laws unchanged, plus the KNOBS the convergence loop turns
//! (tools/pipeline.py): per-part authority scales, the margins, the fluid loop's electromagnetic pump
//! (mass/power rate lambda, flow-sensor grade), star-tracker heads, gyro grade.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use adcs_sim::Error;
use adcs_fsw::guid::{boresight_offset, guidance, Guid};
use adcs_sim::config::Config;
use adcs_sim::json;
use adcs_sim::run::Truth;
use adcs_sim_core::la::*;
use adcs_sim_core::torques::{self, Facets};
use adcs_sim_core::{field, time};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::f64::consts::PI;
use std::path::Path;

pub mod empump;

/// What the convergence loop may change between iterations.
#[derive(Clone, Debug)]
pub struct Knobs {
    /// authority scale per sized part: mtq, mtqp, rw, cmg, vscmg, fmr, rcs (1 = the law's size)
    pub scale: BTreeMap<String, f64>,
    /// momentum margin (default 2, or 1/(1 - req.hsat)); torque margin (default 1.5)
    pub k_h: Option<f64>, pub k_tau: f64,
    /// fluid loop: mass/power exchange rate of the electromagnetic pump design [kg/W] (empump.rs)
    pub fmr_lambda: f64,
    /// fit the star tracker on a coarse-class product too (knowledge upgrade)
    pub star_tracker: bool,
    /// star-tracker heads (2 by default; 1 saves a head's mass where knowledge allows)
    pub st_heads: u8,
    /// fluid-loop flow sensor noise, 1 sigma [m/s] (the in-house loop's sensor requirement)
    pub fmr_flow_sigma: f64,
    /// gyro grade: noise scale on the precision gyro (1 = TRN-GYRO-P1; 0.3, 0.1 = FOG class)
    pub gyro_grade: f64,
    /// a fourth fluid ring, skewed, that can stand in for any one of the three (single-fault tolerance)
    pub fmr_spare: bool,
}
impl Default for Knobs {
    fn default() -> Self { Knobs { scale: BTreeMap::new(), k_h: None, k_tau: 1.5, fmr_lambda: 0.1, star_tracker: false, st_heads: 2, fmr_flow_sigma: 0.002, gyro_grade: 1.0, fmr_spare: false } }
}
impl Knobs {
    /// The knobs a file states; a key left out keeps its default. A key the sizing does not
    /// read, or a value of the wrong kind, is refused by name.
    pub fn from_json(v: &Value) -> Result<Knobs, Error> {
        const KEYS: [&str; 9] = ["scale", "k_h", "k_tau", "fmr_lambda", "st_heads", "fmr_flow_sigma", "gyro_grade", "star_tracker", "fmr_spare"];
        let o = v.as_object().ok_or_else(|| Error::refused("knobs: must be a JSON object"))?;
        if let Some(k) = o.keys().find(|k| !KEYS.contains(&k.as_str())) {
            return Err(Error::refused(format!("knobs: {k} is not a knob the sizing reads ({})", KEYS.join(", "))));
        }
        let num = |k: &str, lo: f64, hi: f64, d: f64| -> Result<f64, Error> {
            match o.get(k) {
                None | Some(Value::Null) => Ok(d),
                Some(x) => x.as_f64().filter(|x| x.is_finite() && *x >= lo && *x <= hi)
                    .ok_or_else(|| Error::refused(format!("knobs: {k} = {x} is not a number from {lo} to {hi}"))),
            }
        };
        let mut k = Knobs::default();
        if let Some(s) = o.get("scale") {
            let m = s.as_object().ok_or_else(|| Error::refused("knobs: scale must be an object of part kind -> factor"))?;
            for (a, b) in m {
                let x = b.as_f64().filter(|x| x.is_finite() && *x > 0.0).ok_or_else(|| Error::refused(format!("knobs: scale.{a} = {b} is not a positive factor")))?;
                k.scale.insert(a.clone(), x);
            }
        }
        k.k_h = match o.get("k_h") { None | Some(Value::Null) => None, Some(_) => Some(num("k_h", 0.0, 100.0, 0.0)?) };
        k.k_tau = num("k_tau", 0.0, 100.0, 1.5)?;
        k.fmr_lambda = num("fmr_lambda", 0.0, 10.0, 0.1)?;
        k.st_heads = num("st_heads", 1.0, 3.0, 2.0)? as u8;
        k.fmr_flow_sigma = num("fmr_flow_sigma", 0.0, 1.0, 0.002)?;
        k.gyro_grade = num("gyro_grade", 0.0, 100.0, 1.0)?;
        k.star_tracker = match o.get("star_tracker") {
            None | Some(Value::Null) => false,
            Some(Value::Bool(b)) => *b,
            Some(x) => return Err(Error::refused(format!("knobs: star_tracker = {x} is not true or false"))),
        };
        k.fmr_spare = match o.get("fmr_spare") {
            None | Some(Value::Null) => false,
            Some(Value::Bool(b)) => *b,
            Some(x) => return Err(Error::refused(format!("knobs: fmr_spare = {x} is not true or false"))),
        };
        Ok(k)
    }
    pub fn json(&self) -> Value { json!({"scale": self.scale, "k_h": self.k_h, "k_tau": self.k_tau, "fmr_lambda": self.fmr_lambda, "star_tracker": self.star_tracker, "st_heads": self.st_heads, "fmr_flow_sigma": self.fmr_flow_sigma, "gyro_grade": self.gyro_grade, "fmr_spare": self.fmr_spare}) }
    fn s(&self, p: &str) -> f64 { self.scale.get(p).copied().unwrap_or(1.0) }
}

/// asils.sizing.demand
#[derive(Clone, Debug, Default)]
pub struct Demand {
    pub case: String, pub period_s: f64,
    pub tau_peak: [f64; 4], pub h_cyclic: [f64; 4], pub h_secular_orbit: [f64; 4], pub tau_axis_peak: [[f64; 4]; 3],
    pub tau_dist: f64, pub worst_attitude: String, pub h_dist: f64, pub h_secular: f64,
    pub b_min: f64, pub b_mean: f64, pub eclipse_frac: f64,
    pub w0_deg_s: f64, pub j: [f64; 3], pub h_detumble: f64,
    pub slew_deg: f64, pub slew_s: f64, pub w_slew: f64, pub a_slew: f64, pub h_slew: f64, pub tau_slew: f64,
    pub life_yr: f64, pub slews_per_day: f64, pub k_h: f64, pub k_tau: f64, pub h_req: f64, pub tau_req: f64,
    pub req: BTreeMap<String, f64>, pub notes: Vec<String>, pub fine: bool,
    /// the body of the case's class (catalogue/classes.toml): the faces and lever arms the parts are sized on
    pub class: String, pub box_m: [f64; 3],
    /// every season and solar activity the survey flew (the worst of them is sized for)
    pub sweep: Vec<Value>,
}

const ATT: [&str; 4] = ["X_nadir", "Y_nadir", "Z_nadir", "sun"];

/// The environment the survey sweeps over the mission life: four seasons (the Sun's direction
/// against the orbit plane, so the beta angle and the eclipses) and the long-term low and high
/// solar activity (the density, so the aerodynamic torque): ECSS-E-ST-10-04C long-term F10.7.
pub const SURVEY_EPOCH_DAYS: [f64; 4] = [0.0, 91.3, 182.6, 273.9];
pub const SURVEY_F107: [f64; 2] = [65.0, 250.0];

/// One orbit of disturbance at the four attitudes, for one season and one solar activity.
struct Survey { tau_peak: [f64; 4], tau_axis_peak: [[f64; 4]; 3], h_cyclic: [f64; 4], h_secular_orbit: [f64; 4], b_min: f64, b_mean: f64, eclipse_frac: f64 }

fn survey(root: &Path, case_file: &Path, sets: &[(String, String)]) -> Result<(Config, Survey), Error> {
    // the survey scenario of asils.sizing.demand: nadir, 10 s, one orbit
    let s = json!({"schema": "adcs-scenario/1", "id": "sizing_survey", "product": "TRN-P-3U-AIS", "label": "sizing survey",
        "time": {"duration_s": 5740, "dt_s": 10, "record_dt_s": 10}, "initial": {"attitude": {"kind": "nadir"}, "rate": {"kind": "lvlh"}},
        "fsw": {"start_mode": "detumble", "guidance": {"kind": "nadir"}}, "metrics": []});
    static N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let tmp = std::env::temp_dir().join(format!("adcs-survey-{}-{}.json", std::process::id(), N.fetch_add(1, std::sync::atomic::Ordering::Relaxed)));
    std::fs::write(&tmp, s.to_string()).map_err(|e| Error::io(&tmp, e))?;
    let c = Config::build(root, &tmp.display().to_string(), case_file, 1, sets);
    let _ = std::fs::remove_file(&tmp);
    let c = c?;
    let (mut orb, _) = Truth::new(&c)?;
    let gh = field::gh(time::decyear(c.jd0));
    let facets = Facets::boxed(&c.box_m, &c.cm_offset_m, c.sigma_n, c.sigma_t, c.vb_ratio, c.refl, c.spec_frac);
    let t_orb = c.period_s;
    let dt = 10.0;
    let n = (t_orb/dt).floor() as usize + 1;
    let offs = [boresight_offset(&[1.0, 0.0, 0.0]), boresight_offset(&[0.0, 1.0, 0.0]), boresight_offset(&[0.0, 0.0, 1.0])];
    let mut tau = vec![[[0.0; 3]; 4]; n];
    let (mut bm, mut nu) = (vec![0.0; n], vec![0.0; n]);
    for kk in 0..n {
        let t = kk as f64*dt;
        let (r, vv) = orb.state(t)?;
        let e = orb.env(t, c.jd0, &r, &vv, &gh, 13);
        bm[kk] = norm(&e.b_eci); nu[kk] = e.nu;
        for a in 0..4 {
            let g = if a < 3 { Guid { q_off: offs[a], ..Default::default() } }
                    else { Guid { sun_eci: unit(&e.sun_rel), sun_axis: [0.0, 0.0, -1.0], roll_axis: [1.0, 0.0, 0.0], ..Default::default() } };
            let q = guidance(if a < 3 { 0 } else { 4 }, &r, &vv, t, &g).q;
            let p = torques::torques(&q, &r, &e.v_rel, &e.b_eci, &e.sun_rel, e.nu, e.p_srp, e.rho, &c.inertia, &facets, &c.m_res, c.mu, c.env_on);
            tau[kk][a] = add(&add(&p[0], &p[1]), &add(&p[2], &p[3]));
        }
    }
    let mut sv = Survey { tau_peak: [0.0; 4], tau_axis_peak: [[0.0; 4]; 3], h_cyclic: [0.0; 4], h_secular_orbit: [0.0; 4],
        b_min: bm.iter().cloned().fold(f64::MAX, f64::min), b_mean: bm.iter().sum::<f64>()/n as f64,
        eclipse_frac: nu.iter().filter(|x| **x < 0.5).count() as f64/n as f64 };
    for a in 0..4 {
        let mut h = [0.0; 3];
        let mut hs = Vec::with_capacity(n);
        for kk in 0..n { for i in 0..3 { h[i] += tau[kk][a][i]*dt; } hs.push(h); }
        let hend = hs[n - 1];
        sv.tau_peak[a] = tau.iter().map(|x| norm(&x[a])).fold(0.0, f64::max);
        for i in 0..3 { sv.tau_axis_peak[i][a] = tau.iter().map(|x| x[a][i].abs()).fold(0.0, f64::max); }
        sv.h_cyclic[a] = (0..n).map(|kk| { let f = kk as f64*dt/t_orb; norm(&sub(&hs[kk], &scale(&hend, f))) }).fold(0.0, f64::max);
        sv.h_secular_orbit[a] = norm(&hend);
    }
    Ok((c, sv))
}

pub fn demand(root: &Path, case_file: &Path, k: &Knobs) -> Result<Demand, Error> {
    // the worst of every season and solar activity, each attitude and axis on its own
    let mut base: Option<(Config, Survey)> = None;
    let mut sweep = vec![];
    for ep in SURVEY_EPOCH_DAYS {
        for f in SURVEY_F107 {
            let sets = [("engine.epoch_days".to_string(), format!("{ep}")), ("engine.f107".to_string(), format!("{f}")), ("engine.f107a".to_string(), format!("{f}"))];
            let (c, sv) = survey(root, case_file, &sets)?;
            sweep.push(json!({"epoch_days": ep, "f107": f, "tau_peak": sv.tau_peak, "h_secular_orbit": sv.h_secular_orbit, "eclipse_frac": sv.eclipse_frac}));
            base = Some(match base {
                None => (c, sv),
                Some((c0, mut w)) => {
                    for a in 0..4 {
                        w.tau_peak[a] = w.tau_peak[a].max(sv.tau_peak[a]); w.h_cyclic[a] = w.h_cyclic[a].max(sv.h_cyclic[a]);
                        w.h_secular_orbit[a] = w.h_secular_orbit[a].max(sv.h_secular_orbit[a]);
                        for i in 0..3 { w.tau_axis_peak[i][a] = w.tau_axis_peak[i][a].max(sv.tau_axis_peak[i][a]); }
                    }
                    w.b_min = w.b_min.min(sv.b_min); w.eclipse_frac = w.eclipse_frac.max(sv.eclipse_frac);
                    (c0, w)
                }
            });
        }
    }
    let (c, sv) = base.expect("the sweep has at least one survey");
    let v = |key: &str| c.case.get(key);
    let t_orb = c.period_s;
    let mut d = Demand { case: c.case.id.clone(), period_s: t_orb, class: c.case.class.clone(), box_m: c.box_m,
        tau_peak: sv.tau_peak, tau_axis_peak: sv.tau_axis_peak, h_cyclic: sv.h_cyclic, h_secular_orbit: sv.h_secular_orbit,
        b_min: sv.b_min, b_mean: sv.b_mean, eclipse_frac: sv.eclipse_frac, sweep, ..Default::default() };
    let ia = (0..4).fold(0, |b, a| if d.tau_peak[a] > d.tau_peak[b] { a } else { b });
    d.tau_dist = d.tau_peak[ia]; d.worst_attitude = ATT[ia].into();
    // the secular momentum held between dumps: req.dump hours of it, or a quarter orbit when the case is blank
    let dump_h = v("req.dump");
    let orbits_held = if dump_h.is_finite() && dump_h > 0.0 { dump_h*3600.0/t_orb } else { 0.25 };
    d.h_dist = (0..4).map(|a| d.h_cyclic[a] + orbits_held*d.h_secular_orbit[a]).fold(f64::MIN, f64::max);
    d.h_secular = d.h_secular_orbit.iter().cloned().fold(f64::MIN, f64::max);
    let dflt = |x: f64, dv: f64, note: &str, notes: &mut Vec<String>| if x.is_nan() { notes.push(note.into()); dv } else { x };
    let mut notes = vec![];
    if !(dump_h.is_finite() && dump_h > 0.0) { notes.push("req.dump blank: a quarter orbit of secular momentum held between dumps taken".into()); }
    d.w0_deg_s = dflt(v("mission.w0"), 10.0, "mission.w0 blank: 10 deg/s taken", &mut notes);
    d.j = [c.inertia[0][0], c.inertia[1][1], c.inertia[2][2]];
    let jmax = d.j.iter().cloned().fold(f64::MIN, f64::max);
    d.h_detumble = jmax*d.w0_deg_s*PI/180.0;
    d.slew_deg = dflt(v("mission.sangle"), 30.0, "mission.sangle blank: 30 deg taken", &mut notes);
    d.slew_s = dflt(v("req.slew"), 60.0, "req.slew blank: 60 s taken for the reference slew", &mut notes);
    d.w_slew = 2.0*d.slew_deg*PI/180.0/d.slew_s;
    d.a_slew = 2.0*PI*d.slew_deg*PI/180.0/(d.slew_s*d.slew_s);
    d.h_slew = jmax*d.w_slew; d.tau_slew = jmax*d.a_slew;
    d.life_yr = dflt(v("mission.life"), 3.0, "mission.life blank: 3 years taken", &mut notes);
    d.slews_per_day = { let x = v("mission.spd"); if x.is_nan() { 0.0 } else { x } };
    let hsat = v("req.hsat");
    d.k_h = k.k_h.unwrap_or(if hsat.is_finite() && hsat > 0.0 && hsat < 1.0 { 1.0/(1.0 - hsat) } else { 2.0 });
    d.k_tau = k.k_tau;
    d.h_req = d.k_h*d.h_dist.max(d.h_slew);
    d.tau_req = d.k_tau*d.tau_dist.max(d.tau_slew);
    for r in ["ape", "ake", "rks", "mass", "pavg", "ppk", "vol", "detumble", "sunacq"] { d.req.insert(r.into(), v(&format!("req.{r}"))); }
    // what the platform allocates to the ADCS (blank: no allocation stated, nothing checked)
    for r in ["malloc", "palloc", "valloc"] { d.req.insert(r.into(), v(&format!("resources.{r}"))); }
    d.fine = d.req["ake"].is_finite() && d.req["ake"] <= 0.05;
    d.notes = notes;
    Ok(d)
}

impl Demand {
    pub fn json(&self) -> Value {
        let tap: Vec<Vec<f64>> = self.tau_axis_peak.iter().map(|r| r.to_vec()).collect();
        json!({"case": self.case, "orbit_period_s": self.period_s, "attitudes": ATT, "tau_peak": self.tau_peak, "h_cyclic": self.h_cyclic,
            "h_secular_orbit": self.h_secular_orbit, "tau_axis_peak": tap, "tau_dist": self.tau_dist, "worst_attitude": self.worst_attitude,
            "h_dist": self.h_dist, "h_secular": self.h_secular, "B_min": self.b_min, "B_mean": self.b_mean, "eclipse_frac": self.eclipse_frac,
            "w0_deg_s": self.w0_deg_s, "J": self.j, "h_detumble": self.h_detumble, "slew_deg": self.slew_deg, "slew_s": self.slew_s,
            "w_slew": self.w_slew, "a_slew": self.a_slew, "h_slew": self.h_slew, "tau_slew": self.tau_slew, "life_yr": self.life_yr,
            "slews_per_day": self.slews_per_day, "k_h": self.k_h, "k_tau": self.k_tau, "h_req": self.h_req, "tau_req": self.tau_req,
            "req": self.req.iter().map(|(a, b)| (a.clone(), if b.is_finite() { json!(b) } else { Value::Null })).collect::<serde_json::Map<_, _>>(),
            "notes": self.notes, "class": if self.fine { "fine" } else { "coarse" }, "body_class": self.class, "box_m": self.box_m, "survey_sweep": self.sweep})
    }
}

fn part(case: &str, tag: &str, kind: &str, name: &str, made: &str) -> Value {
    json!({"part_number": format!("SZ-{case}-{tag}"), "kind": kind, "name": format!("{name} — {case}"), "status": "sized",
        "source": "adcs-design (asils.sizing laws)", "made": made, "descriptor_version": 1})
}

/// asils.sizing.mtq: (the coil of every family, the pointing-grade coil of the coils-only family)
pub fn mtq(d: &Demand, k: &Knobs) -> (Value, Value) {
    let td = if d.req["detumble"].is_nan() { 3.0*d.period_s/60.0 } else { d.req["detumble"] };
    let m_dump = 2.0*d.tau_dist/(0.5*d.b_min);
    let m_mom = 2.0*d.h_secular/(0.3*d.b_mean*d.period_s);
    let m_det = d.h_detumble/(0.3*d.b_mean*0.5*td*60.0);
    let m = m_dump.max(m_mom).max(m_det).max(0.05);
    let coil = |m: f64, tag: &str, name: &str| {
        let kk = m/0.45;
        let mut p = part(&d.case, tag, "coil_tile", name, "in-house");
        p["nominal"] = json!({"dipole_max_Am2": m, "dipole_per_amp_Am2_per_A": 4.5, "current_max_A": m/4.5, "resistance_ohm": 30,
            "time_constant_s": 0.005, "mass_kg": 0.03*kk, "power_at_max_W": 0.3*kk, "volume_L": 0.012*kk});
        p["dispersion"] = json!({"dipole_scale": {"dist": "normal", "mean": 1, "sigma": 0.02}, "axis_misalignment_rad": {"dist": "normal", "mean": 0, "sigma": 0.005}});
        p["sizing"] = json!({"m_dump_Am2": m_dump, "m_momentum_Am2": m_mom, "m_detumble_Am2": m_det, "scale": k.s(if tag == "MTQ" { "mtq" } else { "mtqp" }),
            "law": "max(dumping, momentum, detumble) dipole; mass and power linear in the dipole (SYN-CT-1 anchor)"});
        p
    };
    (coil(m*k.s("mtq"), "MTQ", "Sized magnetorquer coil (our product)"),
     coil(m.max(2.0*m_dump)*k.s("mtqp"), "MTQP", "Sized pointing-grade magnetorquer coil (our product, coils-only family)"))
}

/// The select_rotor node (docs/NODES.md): the benchmarks' momentum actuators are bought, so they
/// are chosen from the datasheet catalogue (matlab_sils/data/catalogue, tools/catalogue.py), never
/// sized by a law. `which` is rw (three orthogonal wheels) or cmg / vscmg (a four-unit pyramid).
/// Per unit: a wheel must hold h_req and give tau_req; a pyramid unit half of each (two units act on
/// any axis). The lightest selectable model that meets the need wins (then steady power, volume);
/// when none does, the largest is taken and the gap is recorded.
pub fn rotor(root: &Path, d: &Demand, k: &Knobs, which: &str) -> Result<Value, Error> {
    let s = k.s(which);
    let types: &[&str] = if which == "rw" { &["reaction_wheel"] } else { &["cmg", "cmg_cluster"] };
    let share = if which == "rw" { 1.0 } else { 0.5 };
    let (h_need, tau_need) = (share*d.h_req*s, share*d.tau_req*s);
    let mut cands: Vec<Value> = vec![];
    let dir = root.join("data/catalogue");
    let mut files: Vec<_> = std::fs::read_dir(&dir).map_err(|e| Error::io(&dir, e))?.filter_map(|e| e.ok().map(|e| e.path())).collect();
    files.sort();
    for f in files {
        let c = json::read(&f)?;
        if !types.contains(&json::s(&c, "type", "")) || !json::b(&c, "selectable", false) { continue; }
        cands.push(c);
    }
    if cands.is_empty() { return Err(Error::refused(format!("catalogue: no selectable {which} model in {}", dir.display()))); }
    let g = |c: &Value, key: &str| c["derived"][key].as_f64().unwrap_or(f64::NAN);
    let meets = |c: &Value| g(c, "h_max_Nms") >= h_need && g(c, "torque_max_Nm") >= tau_need;
    let key = |c: &Value| (g(c, "mass_kg"), g(c, "power_steady_W"), g(c, "volume_L"));
    let mut ok: Vec<&Value> = cands.iter().filter(|c| meets(c)).collect();
    ok.sort_by(|a, b| key(a).partial_cmp(&key(b)).unwrap());
    let (pick, gap) = match ok.first() {
        Some(c) => ((*c).clone(), Value::Null),
        None => {
            let c = cands.iter().max_by(|a, b| g(a, "h_max_Nms").partial_cmp(&g(b, "h_max_Nms")).unwrap()).unwrap().clone();
            (c, json!(format!("no catalogue {which} meets h {:.3e} N m s, tau {:.3e} N m per unit; the largest is fitted", h_need, tau_need)))
        }
    };
    let x = &pick["derived"];
    let mut nm = x.clone();
    let (kind, suffix, units) = match which { "rw" => ("reaction_wheel", "", 3), "cmg" => ("cmg", "", 4), _ => ("vscmg", "-VSCMG", 4) };
    if which == "vscmg" { nm["rotor_momentum_Nms"] = x["vscmg_rotor_momentum_Nms"].clone(); }
    let pn = format!("{}{suffix}", json::s(&pick, "part_number", ""));
    let mut p = json!({"part_number": pn, "kind": kind, "name": format!("{} {} ({}, one of {units})", json::s(&pick, "vendor", ""), json::s(&pick, "model", ""), which.to_uppercase()),
        "status": "catalogue", "source": pick["source_url"], "made": "bought", "descriptor_version": 1, "vendor": pick["vendor"], "model": pick["model"],
        "verification": pick["verification"], "assumptions": pick["assumptions"]});
    p["nominal"] = nm;
    p["dispersion"] = json!({"torque_scale": {"dist": "normal", "mean": 1, "sigma": 0.01}, "friction_scale": {"dist": "uniform", "lo": 0.5, "hi": 2.0},
        "axis_misalignment_rad": {"dist": "normal", "mean": 0, "sigma": 0.001}});
    p["sizing"] = json!({"node": "select_rotor", "h_req_Nms": d.h_req, "tau_req_Nm": d.tau_req, "scale": s, "units": units,
        "need_per_unit": {"h_Nms": h_need, "tau_Nm": tau_need}, "gap": gap,
        "rule": "lightest selectable catalogue model meeting the per-unit need (then steady power, volume)",
        "candidates": cands.iter().map(|c| json!({"part_number": c["part_number"], "vendor": c["vendor"], "model": c["model"],
            "h_Nms": g(c, "h_max_Nms"), "tau_Nm": g(c, "torque_max_Nm"), "mass_kg": g(c, "mass_kg"), "power_W": g(c, "power_steady_W"),
            "meets": meets(c)})).collect::<Vec<_>>()});
    Ok(p)
}

/// The fluid loop, X, Y, Z rings, each designed with its electromagnetic DC conduction pump
/// (empump.rs): momentum h_req, torque tau_req, for the knob fmr_lambda [kg/W].
pub fn fmr(d: &Demand, k: &Knobs, bx: [f64; 3]) -> Vec<Value> {
    let faces = [bx[1]*bx[2], bx[0]*bx[2], bx[0]*bx[1]];
    let per = [2.0*(bx[1] + bx[2]), 2.0*(bx[0] + bx[2]), 2.0*(bx[0] + bx[1])];
    let h = d.h_req.max(2e-4)*k.s("fmr");
    let tau = d.tau_req.max(1e-5)*k.s("fmr");
    let ax = ["X", "Y", "Z"];
    let mut rings: Vec<Value> = (0..3).map(|i| ring(d, k, h, tau, faces[i], per[i], ax[i], &format!("{} axis", ax[i]))).collect();
    if k.fmr_spare {
        // the spare stands in for any one ring: its axis follows the rings' momenta (all equal here,
        // so the body diagonal) and it carries their root-sum-square, so its projection on each axis
        // holds that ring's whole momentum and torque; its loop lies in the box's cross-section
        // perpendicular to that axis
        let s = spare_axis();
        let (area, perim) = section(bx, s);
        let n = 3f64.sqrt();
        rings.push(ring(d, k, n*h, n*tau, area, perim, "S", "spare, skewed on the body diagonal"));
    }
    rings
}

/// The spare ring's axis: the unit body diagonal (the rings are sized alike).
pub fn spare_axis() -> [f64; 3] { let r = 1.0/3f64.sqrt(); [r, r, r] }

/// Area and perimeter of the cross-section of a centred box (sides `bx`) by the plane through its
/// centre with unit normal `n`: the loop a skewed ring can enclose.
pub fn section(bx: [f64; 3], n: [f64; 3]) -> (f64, f64) {
    let hx = [bx[0]/2.0, bx[1]/2.0, bx[2]/2.0];
    let mut pts: Vec<[f64; 3]> = vec![];
    // every box edge: two coordinates at a corner value, the third free; solve n . p = 0 on it
    for free in 0..3 {
        let (a, b) = ((free + 1) % 3, (free + 2) % 3);
        for sa in [-1.0, 1.0] { for sb in [-1.0, 1.0] {
            if n[free].abs() < 1e-15 { continue; }
            let mut p = [0.0; 3];
            p[a] = sa*hx[a]; p[b] = sb*hx[b];
            p[free] = -(n[a]*p[a] + n[b]*p[b])/n[free];
            if p[free].abs() <= hx[free] + 1e-12 { pts.push(p); }
        } }
    }
    // order them around the normal, then the shoelace area and the perimeter in the plane
    let u = { let t = if n[0].abs() < 0.9 { [1.0, 0.0, 0.0] } else { [0.0, 1.0, 0.0] }; unit(&sub(&t, &scale(&n, t[0]*n[0] + t[1]*n[1] + t[2]*n[2]))) };
    let v = [n[1]*u[2] - n[2]*u[1], n[2]*u[0] - n[0]*u[2], n[0]*u[1] - n[1]*u[0]];
    let mut q: Vec<(f64, f64)> = pts.iter().map(|p| (p[0]*u[0] + p[1]*u[1] + p[2]*u[2], p[0]*v[0] + p[1]*v[1] + p[2]*v[2])).collect();
    q.sort_by(|a, b| a.1.atan2(a.0).partial_cmp(&b.1.atan2(b.0)).unwrap());
    q.dedup_by(|a, b| (a.0 - b.0).abs() < 1e-12 && (a.1 - b.1).abs() < 1e-12);
    let m = q.len();
    let (mut area, mut perim) = (0.0, 0.0);
    for i in 0..m {
        let (a, b) = (q[i], q[(i + 1) % m]);
        area += a.0*b.1 - b.0*a.1;
        perim += ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
    }
    (area.abs()/2.0, perim)
}

/// One fluid ring with its electromagnetic pump, designed for momentum `h` and torque `tau` in a
/// loop of `face` m^2 and `perim` m (80 % of each used).
fn ring(d: &Demand, k: &Knobs, h: f64, tau: f64, face: f64, perim: f64, tag: &str, what: &str) -> Value {
    let (s0, l1) = (0.8*face, 0.8*perim);
    let ds = empump::design(h, tau, s0, l1, k.fmr_lambda).expect("no feasible pump design");
    let mut nm = ds.json(k.fmr_lambda);
    for (key, val) in [("fluid", json!("galinstan")), ("fluid_density_kg_m3", json!(6440.0)), ("fluid_viscosity_Pa_s", json!(0.0024)),
                       ("pump_type", json!("dc-conduction, electromagnet")), ("melt_point_K", json!(254)), ("dipole_max_Am2", json!(0)),
                       ("dipole_per_amp_Am2_per_A", json!(0)), ("current_max_A", json!(0)), ("volume_L", json!(face*0.006*1e3 + 0.02))] {
        nm[key] = val;
    }
    let eta = ds.eta_cruise.max(0.01);
    let mut p = json!({"part_number": format!("SZ-{}-FMR-{}", d.case, tag), "kind": "magneto_fluidic_panel",
        "name": format!("Sized fluid momentum loop with electromagnetic pump, {what} (our product) — {}", d.case), "status": "sized",
        "source": "adcs-design (empump)", "made": "in-house", "descriptor_version": 1});
    p["nominal"] = nm;
    p["dispersion"] = json!({"friction_scale": {"dist": "uniform", "lo": 0.8, "hi": 1.2}, "pump_efficiency": {"dist": "uniform", "lo": 0.7*eta, "hi": 1.3*eta},
        "flow_sensor_noise_m_s": {"dist": "normal", "mean": 0, "sigma": k.fmr_flow_sigma}, "axis_misalignment_rad": {"dist": "normal", "mean": 0, "sigma": 0.005}});
    p["sizing"] = json!({"h_req_Nms": d.h_req, "tau_req_Nm": d.tau_req, "h_ring_Nms": h, "tau_ring_Nm": tau, "flow_sensor_sigma_m_s": k.fmr_flow_sigma, "face_m2": face, "perimeter_m": perim,
        "scale": k.s("fmr"), "lambda_kg_per_W": k.fmr_lambda, "pareto": empump::pareto(h, tau, s0, l1),
        "law": "galinstan loop + DC conduction pump with an electromagnet, designed together: least mass + lambda x steady power (empump.rs)"});
    p
}

/// asils.sizing.rcs
pub fn rcs(d: &Demand, k: &Knobs, bx: [f64; 3]) -> Value {
    let (g0, isp) = (9.80665, 60.0);
    let j = d.j; let jmax = j.iter().cloned().fold(f64::MIN, f64::max);
    let arm = [0.45*bx[1], 0.45*bx[0], 0.45*bx[0]];
    let alpha = (d.tau_req/jmax).max(d.h_detumble/600.0/jmax);
    let freq = (0..3).map(|i| j[i]*alpha/(2.0*arm[i])).fold(f64::MIN, f64::max)*k.s("rcs");
    let cls = [0.005, 0.010, 0.020, 0.050, 0.100];
    let f = cls.iter().copied().find(|c| *c >= freq).unwrap_or((freq/0.05).ceil()*0.05);
    let arm_eff = 3.0/(1.0/arm[0] + 1.0/arm[1] + 1.0/arm[2]);
    let orbits = d.life_yr*365.25*86400.0/d.period_s;
    let it_det = 2.0*d.h_detumble/arm_eff;
    let it_dump = d.h_secular*orbits/arm_eff;
    let it_slew = d.slews_per_day*365.25*d.life_yr*2.0*d.h_slew/arm_eff;
    let mprop = (1.2*(it_det + it_dump)/(isp*g0)).max(0.01);
    let mslew = 1.2*it_slew/(isp*g0);
    let v = 1.25*mprop/745.0;
    let rt = (3.0*v/(4.0*PI)).powf(1.0/3.0); let (pm, sig) = (70e5, 250e6);
    let t = (pm*rt/(2.0*sig)).max(0.5e-3);
    let mtank = 4.0*PI*rt*rt*t*2810.0;
    let mdry = mtank + 12.0*0.010 + 0.050;
    let mut p = part(&d.case, "RCS", "rcs", "Sized N2O cold-gas RCS, 6 couples (our product)", "in-house");
    p["nominal"] = json!({"thrust_N": f, "isp_s": 70, "propellant": "N2O", "thrusters": 12, "mib_s": 0.005, "valve_res_s": 0.001,
        "arm_long_m": arm[1], "arm_short_m": arm[0], "propellant_kg": mprop, "tank_volume_L": v*1e3, "tank_radius_m": rt, "tank_wall_m": t,
        "tank_pressure_bar": 50, "meop_bar": 70, "valve_power_W": 1.0, "power_steady_W": 0.05, "dry_mass_kg": mdry, "mass_kg": mdry + mprop,
        "volume_L": (2.0*rt).powi(3)*1e3 + 0.05});
    p["dispersion"] = json!({"thrust_scale": {"dist": "normal", "mean": 1, "sigma": 0.03}, "isp_s": {"dist": "uniform", "lo": 60, "hi": 80},
        "axis_misalignment_rad": {"dist": "normal", "mean": 0, "sigma": 0.01}});
    p["sizing"] = json!({"F_req_N": freq, "impulse_detumble_Ns": it_det, "impulse_dumping_Ns": it_dump, "impulse_slews_Ns": it_slew,
        "propellant_if_slews_on_rcs_kg": mslew, "life_yr": d.life_yr, "isp_budget_s": isp, "scale": k.s("rcs"),
        "law": "N2O self-pressurised, Isp 60 s budget (60-80 s), Al-7075 sphere at 70 bar MEOP"});
    p
}

fn num(v: &Value, k: &str) -> Option<f64> { v.get(k).and_then(|x| x.as_f64()) }

/// asils.product.load budget_: ADCS mass / nominal power / volume per fill.
fn budget(fill: &[Value], lookup: &dyn Fn(&str) -> Result<Value, Error>) -> Result<Value, Error> {
    let (mut m, mut p, mut vol) = (0.0, 0.0, 0.0);
    let mut items = vec![];
    for f in fill {
        let slot = json::s(f, "slot", "");
        let mut n = 1.0;
        for key in ["axes_body", "spin_axes_body", "boresights_body", "normals_body"] {
            if let Some(a) = f.get(key).and_then(|x| x.as_array()) { n = a.len() as f64; if slot == "coarse_sun_sensors" { n = 1.0; } }
        }
        let pt = lookup(json::s(f, "part", ""))?;
        let nm = pt.get("nominal").cloned().unwrap_or(Value::Null);
        let mm = num(&nm, "mass_kg").unwrap_or(0.0);
        let pp = num(&nm, "power_steady_W").or_else(|| num(&nm, "power_W")).or_else(|| num(&nm, "power_at_max_W")).unwrap_or(0.0);
        let vv = num(&nm, "volume_L").unwrap_or(0.0);
        m += n*mm; p += n*pp; vol += n*vv;
        items.push(json!({"slot": slot, "part": f["part"], "n": n, "mass_kg": n*mm, "power_W": n*pp, "volume_L": n*vv}));
    }
    Ok(json!({"mass_kg": m, "power_W": p, "volume_L": vol, "items": items}))
}

/// asils.sizing.size_all: every part, one product per family, budgets. Writes
/// <out>/parts/*.json, <out>/products/*.json, <out>/sizing.json.
pub fn size_all(root: &Path, case_file: &Path, k: &Knobs, out: &Path) -> Result<Value, Error> {
    let d = demand(root, case_file, k)?;
    let case = d.case.clone();
    let bx = d.box_m;
    let (pm, pmp) = mtq(&d, k);
    let fm = fmr(&d, k, bx);
    let parts: Vec<(&str, Value)> = vec![("mtq", pm), ("mtqp", pmp), ("rw", rotor(root, &d, k, "rw")?), ("cmg", rotor(root, &d, k, "cmg")?), ("vscmg", rotor(root, &d, k, "vscmg")?),
        ("fmr_x", fm[0].clone()), ("fmr_y", fm[1].clone()), ("fmr_z", fm[2].clone()), ("rcs", rcs(&d, k, bx))];
    let mut parts = parts;
    if let Some(sp) = fm.get(3) { parts.push(("fmr_s", sp.clone())); }
    for dir in ["parts", "products"] { std::fs::create_dir_all(out.join(dir)).map_err(|e| Error::io(&out.join(dir), e))?; }
    let mut by_pn: BTreeMap<String, Value> = BTreeMap::new();
    for (_, p) in &parts {
        let pn = json::s(p, "part_number", "").to_string();
        adcs_sim::fsio::write(&out.join("parts").join(format!("{pn}.json")), serde_json::to_string(p).map_err(|e| Error::run(e.to_string()))?)?;
        by_pn.insert(pn, p.clone());
    }
    // a better gyro than the catalogue's precision unit when the loop asks for it: noise x grade,
    // mass and power / grade (anchored on the small fibre-optic class: noise x0.3 ~ 0.2 kg, 1 W)
    let mut gyro_id = "TRN-GYRO-P1".to_string();
    if d.fine && k.gyro_grade < 0.999 {
        let mut g = json::read(&adcs_sim::product::find(root, "parts", "TRN-GYRO-P1")?)?;
        let gr = k.gyro_grade;
        for key in ["arw_rad_per_sqrt_s", "rrw_rad_per_s_sqrt_s", "bias_instability_rad_s"] {
            if let Some(x) = g["nominal"][key].as_f64() { g["nominal"][key] = json!(x*gr); }
        }
        for key in ["mass_kg", "power_W"] { if let Some(x) = g["nominal"][key].as_f64() { g["nominal"][key] = json!(x/gr); } }
        gyro_id = format!("SZ-{case}-GYRO");
        g["part_number"] = json!(gyro_id); g["status"] = json!("sized"); g["source"] = json!("adcs-design (gyro grade)");
        g["name"] = json!(format!("Gyro, noise x{gr} of TRN-GYRO-P1 (fibre-optic class) — {case}"));
        adcs_sim::fsio::write(&out.join("parts").join(format!("{gyro_id}.json")), serde_json::to_string(&g).map_err(|e| Error::run(e.to_string()))?)?;
        by_pn.insert(gyro_id.clone(), g);
    }
    let pn = |key: &str| parts.iter().find(|x| x.0 == key).map(|x| json::s(&x.1, "part_number", "").to_string()).unwrap_or_default();
    let lookup = |id: &str| -> Result<Value, Error> {
        if let Some(p) = by_pn.get(id) { return Ok(p.clone()); }
        json::read(&adcs_sim::product::find(root, "parts", id)?)
    };
    let fams = json::read(&root.join("data/families.json"))?;
    let fam_list: Vec<Value> = match fams.get("family") { Some(Value::Array(a)) => a.clone(), Some(x) => vec![x.clone()], None => vec![] };
    let i3 = json!([[1, 0, 0], [0, 1, 0], [0, 0, 1]]);
    let st_fit = d.fine || k.star_tracker;
    let mut families = serde_json::Map::new();
    for fa in &fam_list {
        let id = json::s(fa, "id", "");
        let acts: Vec<String> = match fa.get("actuators") { Some(Value::Array(a)) => a.iter().filter_map(|x| x.as_str().map(String::from)).collect(),
                                                           Some(Value::String(s)) => vec![s.clone()], _ => vec![] };
        let mut algs = vec!["bdot", "mekf"];
        let coil = if acts == ["mtq"] { pn("mtqp") } else { pn("mtq") };
        let mut fill = vec![json!({"slot": "coils", "part": coil, "axes_body": i3})];
        for a in &acts {
            match a.as_str() {
                "rw" => fill.push(json!({"slot": "wheels", "part": pn("rw"), "axes_body": i3})),
                "fmr" => {
                    for (key, axv) in [("fmr_x", [1, 0, 0]), ("fmr_y", [0, 1, 0]), ("fmr_z", [0, 0, 1])] { fill.push(json!({"slot": "rings", "part": pn(key), "axes_body": [axv]})); }
                    if k.fmr_spare { fill.push(json!({"slot": "rings", "part": pn("fmr_s"), "axes_body": [spare_axis()]})); }
                    algs.push("idmas_split");
                }
                "cmg" | "vscmg" => fill.push(json!({"slot": a, "part": pn(a),
                    "gimbal_axes_body": [[0.8165, 0, 0.5774], [0, 0.8165, 0.5774], [-0.8165, 0, 0.5774], [0, -0.8165, 0.5774]],
                    "spin_axes_body": [[0, 1, 0], [-1, 0, 0], [0, -1, 0], [1, 0, 0]]})),
                "rcs" => { fill.push(json!({"slot": "rcs", "part": pn("rcs")})); algs.push("rcs_pwm"); }
                _ => {}
            }
        }
        let bs = if d.fine { [0, 1, 0] } else { [1, 0, 0] };
        if st_fit {
            // heads away from nadir: on -Y for the imaging class (payload +Y), on -X for the coarse class (payload +X)
            let mut heads = if d.fine { json!([[0, -0.9063, 0.4226], [0, -0.9063, -0.4226]]) } else { json!([[-0.9063, 0, 0.4226], [-0.9063, 0, -0.4226]]) };
            if k.st_heads == 1 { heads = json!([heads[0].clone()]); }
            fill.push(json!({"slot": "star_tracker", "part": "SYN-ST-1", "boresights_body": heads, "calibrated_residual_rad": 1e-5}));
        }
        fill.push(json!({"slot": "magnetometer", "part": "SYN-MAG-1"}));
        fill.push(json!({"slot": "sun_sensors", "part": "SYN-SUN-1", "normals_body": [[1, 0, 0], [-1, 0, 0], [0, 1, 0], [0, -1, 0], [0, 0, 1], [0, 0, -1]]}));
        fill.push(json!({"slot": "gyro", "part": if d.fine { gyro_id.as_str() } else { "SYN-GYRO-1" }}));
        fill.push(json!({"slot": "gnss", "part": "TRN-GPS-1"}));
        fill.push(json!({"slot": "earth_sensor", "part": "SYN-ES-1", "boresight_body": bs}));
        let label = json::s(fa, "label", id);
        let pr = json!({"schema": "adcs-product/1", "id": format!("SZ-{case}-{id}"), "label": format!("{label} — sized to {case}"), "family": id,
            "role": fa["role"], "classes": ["cubesat_3u"], "status": "sized", "origin": "designed", "source": "adcs-design",
            "algorithms": algs, "sun_axis_body": [0, 0, -1], "payload_boresight_body": bs, "fill": fill, "knobs": k.json()});
        adcs_sim::fsio::write(&out.join("products").join(format!("SZ-{case}-{id}.json")), serde_json::to_string(&pr).map_err(|e| Error::run(e.to_string()))?)?;
        let b = budget(pr["fill"].as_array().unwrap(), &lookup)?;
        families.insert(id.to_string(), json!({"product": format!("SZ-{case}-{id}"), "role": fa["role"], "label": label,
            "mass_kg": b["mass_kg"], "power_W": b["power_W"], "volume_L": b["volume_L"], "items": b["items"]}));
    }
    let z = json!({"schema": "adcs-sizing/1", "case": case, "class": if d.fine { "fine" } else { "coarse" }, "star_tracker": st_fit,
        "demand": d.json(), "knobs": k.json(), "families": families,
        "parts": parts.iter().map(|(a, b)| (a.to_string(), b.clone())).collect::<serde_json::Map<_, _>>()});
    adcs_sim::fsio::write(&out.join("sizing.json"), serde_json::to_string_pretty(&z).map_err(|e| Error::run(e.to_string()))?)?;
    Ok(z)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn root() -> std::path::PathBuf { Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../matlab_sils") }

    #[test]
    fn the_box_section_is_the_loop_a_skewed_ring_encloses() {
        // a cube cut through its centre normal to a face: the face itself
        let (a, p) = section([0.1, 0.1, 0.1], [0.0, 0.0, 1.0]);
        assert!((a - 0.01).abs() < 1e-12 && (p - 0.4).abs() < 1e-12);
        // a cube cut normal to its diagonal: the regular hexagon of side s/sqrt(2)
        let (a, p) = section([1.0, 1.0, 1.0], spare_axis());
        let side = 1.0/2f64.sqrt();
        assert!((a - 1.5*3f64.sqrt()*side*side).abs() < 1e-9, "{a}");
        assert!((p - 6.0*side).abs() < 1e-9, "{p}");
    }

    #[test]
    fn the_spare_ring_holds_any_one_rings_momentum_on_its_axis() {
        let d = Demand { case: "t".into(), h_req: 2e-3, tau_req: 6e-5, ..Default::default() };
        let k = Knobs { fmr_spare: true, ..Knobs::default() };
        let r = fmr(&d, &k, [0.34, 0.10, 0.10]);
        assert_eq!(r.len(), 4);
        let h = |p: &Value| p["sizing"]["h_ring_Nms"].as_f64().unwrap();
        let ax = spare_axis();
        for i in 0..3 { assert!((h(&r[3])*ax[i] - h(&r[i])).abs() < 1e-12, "its projection on axis {i} holds that ring's momentum"); }
        assert_eq!(fmr(&d, &Knobs::default(), [0.34, 0.10, 0.10]).len(), 3, "no spare unless the knob asks");
    }

    #[test]
    fn the_survey_sizes_for_the_worst_season_and_solar_activity() {
        let d = demand(&root(), &root().join("cases/ais_3u.csv"), &Knobs::default()).unwrap();
        assert_eq!(d.sweep.len(), SURVEY_EPOCH_DAYS.len()*SURVEY_F107.len());
        let worst = d.sweep.iter().flat_map(|s| s["tau_peak"].as_array().unwrap().iter().map(|x| x.as_f64().unwrap())).fold(0.0, f64::max);
        assert_eq!(d.tau_dist, worst, "the peak torque is the worst of every survey");
        assert_eq!(d.box_m, [0.34, 0.10, 0.10], "the 3U class body");
        // req.dump is blank in the case: the quarter-orbit default is taken and said
        assert!(d.notes.iter().any(|n| n.starts_with("req.dump blank")));
        let h = (0..4).map(|a| d.h_cyclic[a] + 0.25*d.h_secular_orbit[a]).fold(f64::MIN, f64::max);
        assert_eq!(d.h_dist, h);
    }

    #[test]
    fn select_rotor_takes_the_lightest_model_that_meets_the_need() {
        let mut d = Demand { case: "t".into(), h_req: 1.5e-3, tau_req: 5e-5, ..Default::default() };
        let k = Knobs::default();
        let p = rotor(&root(), &d, &k, "rw").unwrap();
        assert_eq!(p["part_number"], "CAT-CUBESPACE-CUBEWHEEL-CW0017");
        assert!(p["sizing"]["gap"].is_null());
        // more momentum than CW0017 holds: the next lightest that meets it
        d.h_req = 5e-3;
        let p = rotor(&root(), &d, &k, "rw").unwrap();
        assert_eq!(p["part_number"], "CAT-CUBESPACE-CUBEWHEEL-CW0057");
        // an authority knob raises the need the same way: x4 asks 6 mNms, above CW0057's 5.7
        d.h_req = 1.5e-3;
        let mut k2 = Knobs::default(); k2.scale.insert("rw".into(), 4.0);
        assert_eq!(rotor(&root(), &d, &k2, "rw").unwrap()["part_number"], "CAT-ROCKET-LAB-RW-0-01");
        // nothing large enough: the largest is fitted and the gap recorded
        d.h_req = 10.0;
        assert!(rotor(&root(), &d, &k, "cmg").unwrap()["sizing"]["gap"].is_string());
    }

    #[test]
    fn vscmg_runs_its_rotor_at_half_momentum() {
        let d = Demand { case: "t".into(), h_req: 1e-3, tau_req: 5e-5, ..Default::default() };
        let v = rotor(&root(), &d, &Knobs::default(), "vscmg").unwrap();
        let n = &v["nominal"];
        assert!((n["rotor_momentum_Nms"].as_f64().unwrap() - n["h_max_Nms"].as_f64().unwrap()/2.0).abs() < 1e-12);
        assert!(v["part_number"].as_str().unwrap().ends_with("-VSCMG"));
    }
}
