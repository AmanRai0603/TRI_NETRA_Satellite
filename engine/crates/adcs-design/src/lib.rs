//! The design node of the pipeline (docs/DESIGN_LOOP.md): what the case asks of an actuator (the demand survey on the
//! engine's orbit and torque models), every actuator option sized to it (ours: MTQ, fluid loop, N2O RCS, designed;
//! benchmarks: RW, CMG, VSCMG, chosen from the datasheet catalogue), one product per family and its mass / power /
//! volume budget, with the KNOBS the convergence loop turns (tools/pipeline.py): per-part authority scales, the
//! margins, the fluid loop's electromagnetic pump (mass/power rate lambda, flow-sensor grade), star-tracker heads, gyro
//! grade.
//!
//! Every law is the design's (docs/S7_INVENTORY.md S7.15): the methods of design and act (design/sizedemand.pc,
//! sizemtq.pc, sizerotor.pc, sizefmr.pc, sizercs.pc, sizebudget.pc, sizesensors.pc; act/sizepump.pc, sizering.pc),
//! generated into `gen` by tools/engine_build.py. This crate keeps the survey's step order (the orbit flown and the
//! torques taken by the engine's generated models), the reading of the case, the catalogue and the parts, and the
//! writing of the parts, the products and sizing.json in their formats.
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
use std::path::Path;

pub mod catalogue;
pub mod gen;
// the translator's dispatcher (gen/dispatch.rs, which `adcs design call` serves) names the methods' modules
// crate::<module>, as the flight software's vector test does: they are brought in at the crate's root
pub use gen::*;
use gen::{sizebudget as gb, sizedemand as gd, sizefmr as gf, sizemtq as gm, sizepump as gp, sizercs as gc, sizering as gs,
          sizerotor as gr, sizesensors as gx};

/// What the convergence loop may change between iterations.
#[derive(Clone, Debug)]
pub struct Knobs {
    /// authority scale per sized part: mtq, mtqp, rw, cmg, vscmg, fmr, rcs (1 = the law's size)
    pub scale: BTreeMap<String, f64>,
    /// momentum margin (the design's default, or 1/(1 - req.hsat)); torque margin
    pub k_h: Option<f64>, pub k_tau: f64,
    /// fluid loop: mass/power exchange rate of the electromagnetic pump design [kg/W] (act/sizepump.pc)
    pub fmr_lambda: f64,
    /// fit the star tracker on a coarse-class product too (knowledge upgrade)
    pub star_tracker: bool,
    /// star-tracker heads (1 saves a head's mass where knowledge allows)
    pub st_heads: u8,
    /// fluid-loop flow sensor noise, 1 sigma [m/s] (the in-house loop's sensor requirement)
    pub fmr_flow_sigma: f64,
    /// gyro grade: noise scale on the precision gyro (1 = TRN-GYRO-P1; 0.3, 0.1 = FOG class)
    pub gyro_grade: f64,
    /// a fourth fluid ring, skewed, that can stand in for any one of the three (single-fault tolerance)
    pub fmr_spare: bool,
}
impl Default for Knobs {
    /// The knobs where a file states none: the design's (design/sizedemand.pc sizing_knobs).
    fn default() -> Self {
        let (k_tau, fmr_lambda, st_heads, fmr_flow_sigma, gyro_grade, ..) = gd::sizing_knobs();
        Knobs { scale: BTreeMap::new(), k_h: None, k_tau, fmr_lambda, star_tracker: false, st_heads: st_heads as u8, fmr_flow_sigma, gyro_grade, fmr_spare: false }
    }
}
impl Knobs {
    /// The knobs a file states; a key left out keeps its default. A key the sizing does not
    /// read, or a value of the wrong kind or outside the design's range, is refused by name.
    pub fn from_json(v: &Value) -> Result<Knobs, Error> {
        const KEYS: [&str; 9] = ["scale", "k_h", "k_tau", "fmr_lambda", "st_heads", "fmr_flow_sigma", "gyro_grade", "star_tracker", "fmr_spare"];
        let o = v.as_object().ok_or_else(|| Error::refused("knobs: must be a JSON object"))?;
        if let Some(k) = o.keys().find(|k| !KEYS.contains(&k.as_str())) {
            return Err(Error::refused(format!("knobs: {k} is not a knob the sizing reads ({})", KEYS.join(", "))));
        }
        let (_k_tau, _lambda, _heads, _sigma, _grade, k_lo, k_hi, lambda_lo, lambda_hi, heads_lo, heads_hi, sigma_lo, sigma_hi, grade_lo, grade_hi) = gd::sizing_knobs();
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
        k.k_h = match o.get("k_h") { None | Some(Value::Null) => None, Some(_) => Some(num("k_h", k_lo, k_hi, 0.0)?) };
        k.k_tau = num("k_tau", k_lo, k_hi, k.k_tau)?;
        k.fmr_lambda = num("fmr_lambda", lambda_lo, lambda_hi, k.fmr_lambda)?;
        k.st_heads = num("st_heads", heads_lo, heads_hi, k.st_heads as f64)? as u8;
        k.fmr_flow_sigma = num("fmr_flow_sigma", sigma_lo, sigma_hi, k.fmr_flow_sigma)?;
        k.gyro_grade = num("gyro_grade", grade_lo, grade_hi, k.gyro_grade)?;
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
    /// A part's authority scale: the loop's, or 1 (the law's size) where it sets none.
    fn s(&self, p: &str) -> f64 { self.scale.get(p).copied().unwrap_or(1.0) }
}

/// What the case asks of an actuator (design/sizedemand.pc).
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

/// The survey's attitudes as sizing.json names them, in the order of sizedemand's SurveyAttitude.
const ATT: [&str; 4] = ["X_nadir", "Y_nadir", "Z_nadir", "sun"];

/// One orbit of disturbance at the four attitudes, for one season and one solar activity.
struct Survey { tau_peak: [f64; 4], tau_axis_peak: [[f64; 4]; 3], h_cyclic: [f64; 4], h_secular_orbit: [f64; 4], b_min: f64, b_mean: f64, eclipse_frac: f64 }

/// One survey: the orbit flown by the engine (Truth), at each sample the environment, the four attitudes' reference by
/// the flight software's guidance and the disturbance torques by the engine's models; the design reduces them.
fn survey(root: &Path, case_file: &Path, sets: &[(String, String)]) -> Result<(Config, Survey), Error> {
    let (duration, dt, record_dt) = gd::survey_setup();
    let s = json!({"schema": "adcs-scenario/1", "id": "sizing_survey", "product": "TRN-P-3U-AIS", "label": "sizing survey",
        "time": {"duration_s": duration, "dt_s": dt, "record_dt_s": record_dt}, "initial": {"attitude": {"kind": "nadir"}, "rate": {"kind": "lvlh"}},
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
    let n = gd::survey_samples(t_orb, dt) as usize;
    let att: Vec<_> = (0..4).map(gd::survey_attitude).collect();
    let offs: Vec<_> = att.iter().map(|x| if x.1 { None } else { Some(boresight_offset(&x.0)) }).collect();
    let mut tau = vec![0.0; 12*n];
    let (mut bm, mut nu) = (vec![0.0; n], vec![0.0; n]);
    for kk in 0..n {
        let t = kk as f64*dt;
        let (r, vv) = orb.state(t)?;
        let e = orb.env(t, c.jd0, &r, &vv, &gh, c.igrf_nmax);
        bm[kk] = norm(&e.b_eci); nu[kk] = e.nu;
        for a in 0..4 {
            let (_, _, mode, sun_axis, roll_axis) = att[a];
            let g = match offs[a] { Some(q_off) => Guid { q_off, ..Default::default() },
                                    None => Guid { sun_eci: unit(&e.sun_rel), sun_axis, roll_axis, ..Default::default() } };
            let q = guidance(mode as i32, &r, &vv, t, &g).q;
            let p = torques::torques(&q, &r, &e.v_rel, &e.b_eci, &e.sun_rel, e.nu, e.p_srp, e.rho, &c.inertia, &facets, &c.m_res, c.mu, c.env_on);
            tau[12*kk + 3*a..12*kk + 3*a + 3].copy_from_slice(&add(&add(&p[0], &p[1]), &add(&p[2], &p[3])));
        }
    }
    let (tau_peak, tau_axis_peak, h_cyclic, h_secular_orbit) = gd::survey_orbit(&mut tau, n as i64, dt, t_orb);
    let (b_min, b_mean) = gd::survey_field(&mut bm, n as i64);
    Ok((c, Survey { tau_peak, tau_axis_peak, h_cyclic, h_secular_orbit, b_min, b_mean, eclipse_frac: eclipse_fraction(&nu)? }))
}

/// The eclipse fraction of one orbit's samples of the sunlit fraction: env's method of m2_7, generated from the design
/// (`adcs_sim_core::gen::eclipse`), which holds at most ECLIPSE_NMAX samples.
fn eclipse_fraction(nu: &[f64]) -> Result<f64, Error> {
    use adcs_sim_core::gen::eclipse::{eclipse_fraction, ECLIPSE_NMAX};
    let cap = ECLIPSE_NMAX as usize;
    if nu.is_empty() || nu.len() > cap {
        return Err(Error::refused(format!("the sizing survey's orbit has {} samples; the eclipse fraction (m2_7) takes 1 to {cap}", nu.len())));
    }
    let mut a = [0.0; ECLIPSE_NMAX as usize];
    a[..nu.len()].copy_from_slice(nu);
    Ok(eclipse_fraction(a, nu.len() as i64))
}

/// What the case asks of an actuator: every survey of the design's sweep flown, the worst of them, and the case's lines
/// with the design's defaults where it is blank (design/sizedemand.pc).
pub fn demand(root: &Path, case_file: &Path, k: &Knobs) -> Result<Demand, Error> {
    let mut base: Option<(Config, Survey)> = None;
    let mut sweep = vec![];
    for i in 0..gd::survey_runs() {
        let (ep, f) = gd::survey_sweep(i);
        let sets = [("engine.epoch_days".to_string(), format!("{ep}")), ("engine.f107".to_string(), format!("{f}")), ("engine.f107a".to_string(), format!("{f}"))];
        let (c, sv) = survey(root, case_file, &sets)?;
        sweep.push(json!({"epoch_days": ep, "f107": f, "tau_peak": sv.tau_peak, "h_secular_orbit": sv.h_secular_orbit, "eclipse_frac": sv.eclipse_frac}));
        base = Some(match base {
            None => (c, sv),
            Some((c0, w)) => {
                let (tau_peak, tau_axis_peak, h_cyclic, h_secular_orbit, b_min, eclipse_frac) = gd::survey_worst(w.tau_peak, sv.tau_peak,
                    w.tau_axis_peak, sv.tau_axis_peak, w.h_cyclic, sv.h_cyclic, w.h_secular_orbit, sv.h_secular_orbit, w.b_min, sv.b_min,
                    w.eclipse_frac, sv.eclipse_frac);
                (c0, Survey { tau_peak, tau_axis_peak, h_cyclic, h_secular_orbit, b_min, b_mean: w.b_mean, eclipse_frac })
            }
        });
    }
    let (c, sv) = base.expect("the sweep has at least one survey");
    let v = |key: &str| c.case.get(key);
    let t_orb = c.period_s;
    let j = [c.inertia[0][0], c.inertia[1][1], c.inertia[2][2]];
    let (worst, tau_dist, h_dist, h_secular, dump_dflt, w0, w0_dflt, h_detumble, slew_deg, sangle_dflt, slew_s, slew_dflt, w_slew, a_slew,
         h_slew, tau_slew, life_yr, life_dflt, slews_per_day, k_h, h_req, tau_req, fine) = gd::demand(sv.tau_peak, sv.h_cyclic, sv.h_secular_orbit,
        t_orb, v("req.dump"), v("mission.w0"), v("mission.sangle"), v("req.slew"), v("mission.life"), v("mission.spd"), v("req.hsat"),
        k.k_h.unwrap_or(f64::NAN), k.k_tau, j, v("req.ake"));
    // each default the design took where the case is blank, said in the case's terms
    let mut notes = vec![];
    if dump_dflt { notes.push("req.dump blank: a quarter orbit of secular momentum held between dumps taken".to_string()); }
    if w0_dflt { notes.push(format!("mission.w0 blank: {w0} deg/s taken")); }
    if sangle_dflt { notes.push(format!("mission.sangle blank: {slew_deg} deg taken")); }
    if slew_dflt { notes.push(format!("req.slew blank: {slew_s} s taken for the reference slew")); }
    if life_dflt { notes.push(format!("mission.life blank: {life_yr} years taken")); }
    let mut req = BTreeMap::new();
    for r in ["ape", "ake", "rks", "mass", "pavg", "ppk", "vol", "detumble", "sunacq"] { req.insert(r.into(), v(&format!("req.{r}"))); }
    // what the platform allocates to the ADCS (blank: no allocation stated, nothing checked)
    for r in ["malloc", "palloc", "valloc"] { req.insert(r.into(), v(&format!("resources.{r}"))); }
    Ok(Demand { case: c.case.id.clone(), period_s: t_orb, class: c.case.class.clone(), box_m: c.box_m,
        tau_peak: sv.tau_peak, tau_axis_peak: sv.tau_axis_peak, h_cyclic: sv.h_cyclic, h_secular_orbit: sv.h_secular_orbit,
        b_min: sv.b_min, b_mean: sv.b_mean, eclipse_frac: sv.eclipse_frac, sweep,
        tau_dist, worst_attitude: ATT[worst as usize].into(), h_dist, h_secular, w0_deg_s: w0, j, h_detumble, slew_deg, slew_s, w_slew, a_slew,
        h_slew, tau_slew, life_yr, slews_per_day, k_h, k_tau: k.k_tau, h_req, tau_req, req, notes, fine })
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

/// A dispersion as the part files write it: the design's distribution (sizedemand's Dist) and its two values.
fn spread(dist: i64, a: Value, b: Value) -> Value {
    if dist == gd::DIST_UNIFORM { json!({"dist": "uniform", "lo": a, "hi": b}) } else { json!({"dist": "normal", "mean": a, "sigma": b}) }
}

/// A direction as the part and product files write it: a whole component as a whole number.
fn dir(v: &[f64; 3]) -> Value {
    let c = |x: f64| if x.fract() == 0.0 && x.abs() < 1e15 { json!(x as i64) } else { json!(x) };
    json!([c(v[0]), c(v[1]), c(v[2])])
}
fn dirs(v: &[[f64; 3]]) -> Value { Value::Array(v.iter().map(dir).collect()) }

/// The coil of every family and the pointing-grade coil of the coils-only family (design/sizemtq.pc).
pub fn mtq(d: &Demand, k: &Knobs) -> (Value, Value) {
    let (m_dump, m_mom, m_det, m) = gm::mtq_dipoles(d.tau_dist, d.b_min, d.h_secular, d.b_mean, d.period_s, d.h_detumble, d.req["detumble"]);
    let coil = |m: f64, tag: &str, name: &str| {
        let (dipole_max, per_amp, current_max, resistance, time_constant, mass, power, volume) = gm::mtq_coil(m);
        let (sd, sm, ss, md, mm, ms) = gm::mtq_dispersion();
        let mut p = part(&d.case, tag, "coil_tile", name, "in-house");
        p["nominal"] = json!({"dipole_max_Am2": dipole_max, "dipole_per_amp_Am2_per_A": per_amp, "current_max_A": current_max, "resistance_ohm": resistance,
            "time_constant_s": time_constant, "mass_kg": mass, "power_at_max_W": power, "volume_L": volume});
        p["dispersion"] = json!({"dipole_scale": spread(sd, json!(sm), json!(ss)), "axis_misalignment_rad": spread(md, json!(mm), json!(ms))});
        p["sizing"] = json!({"m_dump_Am2": m_dump, "m_momentum_Am2": m_mom, "m_detumble_Am2": m_det, "scale": k.s(if tag == "MTQ" { "mtq" } else { "mtqp" }),
            "law": "max(dumping, momentum, detumble) dipole; mass and power linear in the dipole (SYN-CT-1 anchor)"});
        p
    };
    (coil(m*k.s("mtq"), "MTQ", "Sized magnetorquer coil (our product)"),
     coil(gm::mtq_pointing_dipole(m, m_dump)*k.s("mtqp"), "MTQP", "Sized pointing-grade magnetorquer coil (our product, coils-only family)"))
}

/// The select_rotor node (docs/NODES.md; design/sizerotor.pc): the benchmarks' momentum actuators are bought, so they
/// are chosen from the datasheet catalogue (matlab_sils/data/catalogue, tools/catalogue.py), never sized by a law.
/// `which` is rw (three orthogonal wheels) or cmg / vscmg (a four-unit pyramid).
pub fn rotor(root: &Path, d: &Demand, k: &Knobs, which: &str) -> Result<Value, Error> {
    let u = match which { "rw" => gr::ROTORUSE_RW, "cmg" => gr::ROTORUSE_CMG, _ => gr::ROTORUSE_VSCMG };
    let s = k.s(which);
    let (h_need, tau_need, units) = gr::rotor_need(u, d.h_req, d.tau_req, s);
    let mut cands: Vec<Value> = vec![];
    let dir = root.join("data/catalogue");
    let files = adcs_sim::source::list(&dir);
    if files.is_empty() { return Err(Error::refused(format!("catalogue: {} holds no model", dir.display()))); }
    for f in files {
        let c = json::read(&f)?;
        let kind = match json::s(&c, "type", "") { "reaction_wheel" => gr::CATALOGUEKIND_REACTION_WHEEL, "cmg" => gr::CATALOGUEKIND_CMG,
                                                   "cmg_cluster" => gr::CATALOGUEKIND_CMG_CLUSTER, _ => gr::CATALOGUEKIND_OTHER };
        if !gr::rotor_fits(u, kind, json::b(&c, "selectable", false)) { continue; }
        cands.push(c);
    }
    if cands.is_empty() { return Err(Error::refused(format!("catalogue: no selectable {which} model in {}", dir.display()))); }
    let g = |c: &Value, key: &str| c["derived"][key].as_f64().unwrap_or(f64::NAN);
    let col = |key: &str| cands.iter().map(|c| g(c, key)).collect::<Vec<f64>>();
    let (mut h, mut tau, mut mass, mut pw, mut vol) = (col("h_max_Nms"), col("torque_max_Nm"), col("mass_kg"), col("power_steady_W"), col("volume_L"));
    let (pick, met) = gr::rotor_pick(&mut h, &mut tau, &mut mass, &mut pw, &mut vol, cands.len() as i64, h_need, tau_need);
    let pick = cands[pick as usize].clone();
    let gap = if met { Value::Null } else {
        json!(format!("no catalogue {which} meets h {:.3e} N m s, tau {:.3e} N m per unit; the largest is fitted", h_need, tau_need))
    };
    let x = &pick["derived"];
    let mut nm = x.clone();
    let (kind, suffix) = match which { "rw" => ("reaction_wheel", ""), "cmg" => ("cmg", ""), _ => ("vscmg", "-VSCMG") };
    if gr::rotor_vscmg(u) { nm["rotor_momentum_Nms"] = x["vscmg_rotor_momentum_Nms"].clone(); }
    let pn = format!("{}{suffix}", json::s(&pick, "part_number", ""));
    let mut p = json!({"part_number": pn, "kind": kind, "name": format!("{} {} ({}, one of {units})", json::s(&pick, "vendor", ""), json::s(&pick, "model", ""), which.to_uppercase()),
        "status": "catalogue", "source": pick["source_url"], "made": "bought", "descriptor_version": 1, "vendor": pick["vendor"], "model": pick["model"],
        "verification": pick["verification"], "assumptions": pick["assumptions"]});
    p["nominal"] = nm;
    let (td, tm, ts, fd, flo, fhi, md, mm, ms) = gr::rotor_dispersion();
    p["dispersion"] = json!({"torque_scale": spread(td, json!(tm), json!(ts)), "friction_scale": spread(fd, json!(flo), json!(fhi)),
        "axis_misalignment_rad": spread(md, json!(mm), json!(ms))});
    p["sizing"] = json!({"node": "select_rotor", "h_req_Nms": d.h_req, "tau_req_Nm": d.tau_req, "scale": s, "units": units,
        "need_per_unit": {"h_Nms": h_need, "tau_Nm": tau_need}, "gap": gap,
        "rule": "lightest selectable catalogue model meeting the per-unit need (then steady power, volume)",
        "candidates": cands.iter().map(|c| json!({"part_number": c["part_number"], "vendor": c["vendor"], "model": c["model"],
            "h_Nms": g(c, "h_max_Nms"), "tau_Nm": g(c, "torque_max_Nm"), "mass_kg": g(c, "mass_kg"), "power_W": g(c, "power_steady_W"),
            "meets": gr::rotor_meets(g(c, "h_max_Nms"), g(c, "torque_max_Nm"), h_need, tau_need)})).collect::<Vec<_>>()});
    Ok(p)
}

/// The fluid loop, X, Y, Z rings, each designed with its electromagnetic DC conduction pump (design/sizefmr.pc,
/// act/sizepump.pc), and the spare when the loop asks for it (act/sizering.pc).
pub fn fmr(d: &Demand, k: &Knobs, bx: [f64; 3]) -> Vec<Value> {
    let (faces, per) = gf::fmr_faces(bx);
    let (h, tau) = gf::fmr_need(d.h_req, d.tau_req, k.s("fmr"));
    let ax = ["X", "Y", "Z"];
    let mut rings: Vec<Value> = (0..3).map(|i| ring(d, k, h, tau, faces[i], per[i], ax[i], &format!("{} axis", ax[i]))).collect();
    if k.fmr_spare {
        let (area, perim) = section(bx, spare_axis());
        let (hs, taus) = gf::fmr_spare_need(h, tau);
        rings.push(ring(d, k, hs, taus, area, perim, "S", "spare, skewed on the body diagonal"));
    }
    rings
}

/// The spare ring's axis (act/sizering.pc).
pub fn spare_axis() -> [f64; 3] { gs::spare_axis() }

/// Area and perimeter of the cross-section of a centred box (sides `bx`) by the plane through its centre with unit
/// normal `n`: the loop a skewed ring can enclose (act/sizering.pc).
pub fn section(bx: [f64; 3], n: [f64; 3]) -> (f64, f64) { gs::ring_section(bx, n) }

/// A pump design as the ring part writes it.
fn design_json(ds: &gp::PumpDesign, lambda: f64) -> Value {
    json!({"bore_m": ds.d, "loops": ds.loops, "v_max_m_s": ds.v_max, "h_max_Nms": ds.h_max, "enclosed_area_m2": ds.s,
        "channel_length_m": ds.l, "reynolds_cruise": ds.re, "v_cruise_m_s": ds.v_cruise,
        "pump": {"type": "DC conduction pump, electromagnet C-core", "B_gap_T": ds.b, "active_length_m": ds.lp, "gap_m": ds.gap,
                 "ampere_turns": ds.ni, "electrode_current_design_A": ds.i_design, "electrode_current_cruise_A": ds.i_cruise,
                 "dp_design_Pa": ds.dp_design, "dp_cruise_Pa": ds.dp_cruise, "coil_power_W": ds.p_coil,
                 "electrode_power_cruise_W": ds.p_elec_cruise, "copper_kg": ds.m_cu, "iron_kg": ds.m_fe, "lambda_kg_per_W": lambda,
                 "efficiency_cruise": ds.eta_cruise},
        "pump_torque_max_Nm": ds.tau_max, "field_power_W": ds.p_coil, "power_steady_W": ds.p_steady, "power_peak_W": ds.p_peak,
        "pump_efficiency": ds.eta, "fluid_mass_kg": ds.m_fluid, "mass_kg": ds.mass})
}

/// One fluid ring with its electromagnetic pump, designed for momentum `h` and torque `tau` in a loop of `face` m^2 and
/// `perim` m.
fn ring(d: &Demand, k: &Knobs, h: f64, tau: f64, face: f64, perim: f64, tag: &str, what: &str) -> Value {
    let (s0, l1) = gf::fmr_loop(face, perim);
    let ds = Some(gp::pump_design(h, tau, s0, l1, k.fmr_lambda)).filter(|x| x.found).expect("no feasible pump design");
    let mut nm = design_json(&ds, k.fmr_lambda);
    let (rho, mu, _rho_e, melt) = gp::pump_fluid();
    let (dipole_max, per_amp, current_max) = gf::fmr_no_dipole();
    for (key, val) in [("fluid", json!("galinstan")), ("fluid_density_kg_m3", json!(rho)), ("fluid_viscosity_Pa_s", json!(mu)),
                       ("pump_type", json!("dc-conduction, electromagnet")), ("melt_point_K", json!(melt)), ("dipole_max_Am2", json!(dipole_max)),
                       ("dipole_per_amp_Am2_per_A", json!(per_amp)), ("current_max_A", json!(current_max)), ("volume_L", json!(gf::fmr_volume(face)))] {
        nm[key] = val;
    }
    let (fd, flo, fhi, pd, plo, phi, wd, wm, ws, md, mm, ms) = gf::fmr_dispersion(ds.eta, k.fmr_flow_sigma);
    let mut p = json!({"part_number": format!("SZ-{}-FMR-{}", d.case, tag), "kind": "magneto_fluidic_panel",
        "name": format!("Sized fluid momentum loop with electromagnetic pump, {what} (our product) — {}", d.case), "status": "sized",
        "source": "adcs-design (empump)", "made": "in-house", "descriptor_version": 1});
    p["nominal"] = nm;
    p["dispersion"] = json!({"friction_scale": spread(fd, json!(flo), json!(fhi)), "pump_efficiency": spread(pd, json!(plo), json!(phi)),
        "flow_sensor_noise_m_s": spread(wd, json!(wm), json!(ws)), "axis_misalignment_rad": spread(md, json!(mm), json!(ms))});
    let (ok, lam, mass, power, coil, v_max, bore, b, loops) = gp::pump_pareto(h, tau, s0, l1);
    let pareto: Vec<Value> = (0..ok.len()).filter(|&i| ok[i] == 1).map(|i| json!({"lambda": lam[i], "mass_kg": mass[i], "power_W": power[i],
        "coil_W": coil[i], "v_max": v_max[i], "bore_m": bore[i], "B_T": b[i], "loops": loops[i]})).collect();
    p["sizing"] = json!({"h_req_Nms": d.h_req, "tau_req_Nm": d.tau_req, "h_ring_Nms": h, "tau_ring_Nm": tau, "flow_sensor_sigma_m_s": k.fmr_flow_sigma, "face_m2": face, "perimeter_m": perim,
        "scale": k.s("fmr"), "lambda_kg_per_W": k.fmr_lambda, "pareto": pareto,
        "law": "galinstan loop + DC conduction pump with an electromagnet, designed together: least mass + lambda x steady power (empump.rs)"});
    p
}

/// The N2O cold-gas thrusters (design/sizercs.pc).
pub fn rcs(d: &Demand, k: &Knobs, bx: [f64; 3]) -> Value {
    let (thrust, f_req, arm_long, arm_short, it_det, it_dump, it_slew, mprop, mslew, tank_volume, tank_radius, tank_wall, dry_mass, mass, volume,
         isp_budget) = gc::rcs_size(d.tau_req, d.h_detumble, d.h_secular, d.slews_per_day, d.h_slew, d.life_yr, d.period_s, d.j, bx, k.s("rcs"));
    let (isp, thrusters, mib, valve_res, tank_pressure, meop, valve_power, power_steady) = gc::rcs_part();
    let mut p = part(&d.case, "RCS", "rcs", "Sized N2O cold-gas RCS, 6 couples (our product)", "in-house");
    p["nominal"] = json!({"thrust_N": thrust, "isp_s": isp, "propellant": "N2O", "thrusters": thrusters, "mib_s": mib, "valve_res_s": valve_res,
        "arm_long_m": arm_long, "arm_short_m": arm_short, "propellant_kg": mprop, "tank_volume_L": tank_volume, "tank_radius_m": tank_radius, "tank_wall_m": tank_wall,
        "tank_pressure_bar": tank_pressure, "meop_bar": meop, "valve_power_W": valve_power, "power_steady_W": power_steady, "dry_mass_kg": dry_mass, "mass_kg": mass,
        "volume_L": volume});
    let (td, tm, ts, id, ilo, ihi, md, mm, ms) = gc::rcs_dispersion();
    p["dispersion"] = json!({"thrust_scale": spread(td, json!(tm), json!(ts)), "isp_s": spread(id, json!(ilo), json!(ihi)),
        "axis_misalignment_rad": spread(md, json!(mm), json!(ms))});
    p["sizing"] = json!({"F_req_N": f_req, "impulse_detumble_Ns": it_det, "impulse_dumping_Ns": it_dump, "impulse_slews_Ns": it_slew,
        "propellant_if_slews_on_rcs_kg": mslew, "life_yr": d.life_yr, "isp_budget_s": isp_budget, "scale": k.s("rcs"),
        "law": "N2O self-pressurised, Isp 60 s budget (60-80 s), Al-7075 sphere at 70 bar MEOP"});
    p
}

fn num(v: &Value, k: &str) -> f64 { v.get(k).and_then(|x| x.as_f64()).unwrap_or(f64::NAN) }

/// A fill slot's word (adcs-product/1) as sizebudget's Slot.
fn slot_of(s: &str) -> i64 {
    match s {
        "coils" => gb::SLOT_COILS, "wheels" => gb::SLOT_WHEELS, "rings" => gb::SLOT_RINGS, "cmg" => gb::SLOT_CMG, "vscmg" => gb::SLOT_VSCMG,
        "rcs" => gb::SLOT_RCS, "star_tracker" => gb::SLOT_STAR_TRACKER, "magnetometer" => gb::SLOT_MAGNETOMETER, "sun_sensors" => gb::SLOT_SUN_SENSORS,
        "gyro" => gb::SLOT_GYRO, "gnss" => gb::SLOT_GNSS, "earth_sensor" => gb::SLOT_EARTH_SENSOR, "coarse_sun_sensors" => gb::SLOT_COARSE_SUN_SENSORS,
        _ => gb::SLOT_OTHER,
    }
}

/// The ADCS's mass / power / volume per fill (design/sizebudget.pc).
fn budget(fill: &[Value], lookup: &dyn Fn(&str) -> Result<Value, Error>) -> Result<Value, Error> {
    let mut items = vec![];
    let (mut ms, mut ps, mut vs) = (vec![], vec![], vec![]);
    for f in fill {
        let slot = json::s(f, "slot", "");
        let count = |key: &str| f.get(key).and_then(|x| x.as_array()).map(|a| a.len() as i64).unwrap_or(-1);
        let pt = lookup(json::s(f, "part", ""))?;
        let nm = pt.get("nominal").cloned().unwrap_or(Value::Null);
        let (n, m, p, v) = gb::budget_line(slot_of(slot), count("axes_body"), count("spin_axes_body"), count("boresights_body"), count("normals_body"),
            num(&nm, "mass_kg"), num(&nm, "power_steady_W"), num(&nm, "power_W"), num(&nm, "power_at_max_W"), num(&nm, "volume_L"));
        items.push(json!({"slot": slot, "part": f["part"], "n": n, "mass_kg": m, "power_W": p, "volume_L": v}));
        ms.push(m); ps.push(p); vs.push(v);
    }
    let (m, p, vol) = gb::budget_total(&mut ms, &mut ps, &mut vs, fill.len() as i64);
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
    // a better gyro than the catalogue's precision unit when the loop asks for it (design/sizesensors.pc)
    let mut gyro_id = "TRN-GYRO-P1".to_string();
    if gx::gyro_graded(d.fine, k.gyro_grade) {
        let mut g = json::read(&adcs_sim::product::find(root, "parts", "TRN-GYRO-P1")?)?;
        let gr = k.gyro_grade;
        for key in ["arw_rad_per_sqrt_s", "rrw_rad_per_s_sqrt_s", "bias_instability_rad_s"] {
            if let Some(x) = g["nominal"][key].as_f64() { g["nominal"][key] = json!(gx::gyro_noise(x, gr)); }
        }
        for key in ["mass_kg", "power_W"] { if let Some(x) = g["nominal"][key].as_f64() { g["nominal"][key] = json!(gx::gyro_load(x, gr)); } }
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
    // how each kind of unit is mounted and which sensors a product carries (design/sizemtq.pc, sizerotor.pc, sizefmr.pc, sizesensors.pc)
    let coil_axes = dirs(&gm::mtq_axes());
    let (wheel_axes, gimbal_axes, spin_axes) = gr::rotor_mounting();
    let ring_axes = gf::fmr_axes();
    let st_fit = gx::sensor_star_tracker(d.fine, k.star_tracker);
    let (n_heads, heads, residual) = gx::sensor_heads(d.fine, k.st_heads as i64);
    let bs = dir(&gx::sensor_boresight(d.fine));
    let (normals, sun_axis) = gx::sensor_sun();
    let mut families = serde_json::Map::new();
    for fa in &fam_list {
        let id = json::s(fa, "id", "");
        let acts: Vec<String> = match fa.get("actuators") { Some(Value::Array(a)) => a.iter().filter_map(|x| x.as_str().map(String::from)).collect(),
                                                           Some(Value::String(s)) => vec![s.clone()], _ => vec![] };
        let mut algs = vec!["bdot", "mekf"];
        let coil = if acts == ["mtq"] { pn("mtqp") } else { pn("mtq") };
        let mut fill = vec![json!({"slot": "coils", "part": coil, "axes_body": coil_axes})];
        for a in &acts {
            match a.as_str() {
                "rw" => fill.push(json!({"slot": "wheels", "part": pn("rw"), "axes_body": dirs(&wheel_axes)})),
                "fmr" => {
                    for (i, key) in ["fmr_x", "fmr_y", "fmr_z"].into_iter().enumerate() { fill.push(json!({"slot": "rings", "part": pn(key), "axes_body": [dir(&ring_axes[i])]})); }
                    if k.fmr_spare { fill.push(json!({"slot": "rings", "part": pn("fmr_s"), "axes_body": [dir(&spare_axis())]})); }
                    algs.push("idmas_split");
                }
                "cmg" | "vscmg" => fill.push(json!({"slot": a, "part": pn(a), "gimbal_axes_body": dirs(&gimbal_axes), "spin_axes_body": dirs(&spin_axes)})),
                "rcs" => { fill.push(json!({"slot": "rcs", "part": pn("rcs")})); algs.push("rcs_pwm"); }
                _ => {}
            }
        }
        if st_fit {
            fill.push(json!({"slot": "star_tracker", "part": "SYN-ST-1", "boresights_body": dirs(&heads[..n_heads as usize]), "calibrated_residual_rad": residual}));
        }
        fill.push(json!({"slot": "magnetometer", "part": "SYN-MAG-1"}));
        fill.push(json!({"slot": "sun_sensors", "part": "SYN-SUN-1", "normals_body": dirs(&normals)}));
        fill.push(json!({"slot": "gyro", "part": if d.fine { gyro_id.as_str() } else { "SYN-GYRO-1" }}));
        fill.push(json!({"slot": "gnss", "part": "TRN-GPS-1"}));
        fill.push(json!({"slot": "earth_sensor", "part": "SYN-ES-1", "boresight_body": bs}));
        let label = json::s(fa, "label", id);
        let pr = json!({"schema": "adcs-product/1", "id": format!("SZ-{case}-{id}"), "label": format!("{label} — sized to {case}"), "family": id,
            "role": fa["role"], "classes": ["cubesat_3u"], "status": "sized", "origin": "designed", "source": "adcs-design",
            "algorithms": algs, "sun_axis_body": dir(&sun_axis), "payload_boresight_body": bs, "fill": fill, "knobs": k.json()});
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
        assert_eq!(d.sweep.len(), gd::survey_runs() as usize);
        let worst = d.sweep.iter().flat_map(|s| s["tau_peak"].as_array().unwrap().iter().map(|x| x.as_f64().unwrap())).fold(0.0, f64::max);
        assert_eq!(d.tau_dist, worst, "the peak torque is the worst of every survey");
        assert_eq!(d.box_m, [0.34, 0.10, 0.10], "the 3U class body");
        // req.dump is blank in the case: the design's default is taken and said
        assert!(d.notes.iter().any(|n| n.starts_with("req.dump blank")));
        let held = gd::demand_defaults().6;
        let h = (0..4).map(|a| d.h_cyclic[a] + held*d.h_secular_orbit[a]).fold(f64::MIN, f64::max);
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
