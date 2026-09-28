//! The design node of the pipeline (docs/DESIGN_LOOP.md): what the case asks of an actuator
//! (demand survey on the POP orbit, the SILS torque models), every actuator option sized to it
//! (ours: MTQ, fluid loop, N2O RCS; benchmarks: RW, CMG, VSCMG), one product per family and
//! its mass / power / volume budget. A port of matlab_sils/+asils/+sizing (demand, mtq, rw,
//! cmg, fmr, rcs, size_all) with the laws unchanged, plus the KNOBS the convergence loop turns
//! (tools/pipeline.py): per-part authority scales, the margins, the fluid loop's electromagnetic pump
//! (mass/power rate lambda, flow-sensor grade), star-tracker heads, gyro grade.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use adcs_fsw::ctl::{boresight_offset, guidance, Guid};
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
}
impl Default for Knobs {
    fn default() -> Self { Knobs { scale: BTreeMap::new(), k_h: None, k_tau: 1.5, fmr_lambda: 0.1, star_tracker: false, st_heads: 2, fmr_flow_sigma: 0.002, gyro_grade: 1.0 } }
}
impl Knobs {
    pub fn from_json(v: &Value) -> Knobs {
        let mut k = Knobs::default();
        if let Some(o) = v.get("scale").and_then(|x| x.as_object()) { for (a, b) in o { if let Some(x) = b.as_f64() { k.scale.insert(a.clone(), x); } } }
        k.k_h = v.get("k_h").and_then(|x| x.as_f64());
        k.k_tau = json::f(v, "k_tau", 1.5);
        k.fmr_lambda = json::f(v, "fmr_lambda", 0.1);
        k.st_heads = json::f(v, "st_heads", 2.0) as u8;
        k.fmr_flow_sigma = json::f(v, "fmr_flow_sigma", 0.002);
        k.gyro_grade = json::f(v, "gyro_grade", 1.0);
        k.star_tracker = json::b(v, "star_tracker", false);
        k
    }
    pub fn json(&self) -> Value { json!({"scale": self.scale, "k_h": self.k_h, "k_tau": self.k_tau, "fmr_lambda": self.fmr_lambda, "star_tracker": self.star_tracker, "st_heads": self.st_heads, "fmr_flow_sigma": self.fmr_flow_sigma, "gyro_grade": self.gyro_grade}) }
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
}

const ATT: [&str; 4] = ["X_nadir", "Y_nadir", "Z_nadir", "sun"];

pub fn demand(root: &Path, case_file: &Path, k: &Knobs) -> Result<Demand, String> {
    // the survey scenario of asils.sizing.demand: nadir, 10 s, one orbit
    let s = json!({"schema": "adcs-scenario/1", "id": "sizing_survey", "product": "TRN-P-3U-AIS", "label": "sizing survey",
        "time": {"duration_s": 5740, "dt_s": 10, "record_dt_s": 10}, "initial": {"attitude": {"kind": "nadir"}, "rate": {"kind": "lvlh"}},
        "fsw": {"start_mode": "detumble", "guidance": {"kind": "nadir"}}, "metrics": []});
    let tmp = std::env::temp_dir().join(format!("adcs-survey-{}.json", std::process::id()));
    std::fs::write(&tmp, s.to_string()).map_err(|e| e.to_string())?;
    let c = Config::build(root, &tmp.display().to_string(), case_file, 1, &[]);
    let _ = std::fs::remove_file(&tmp);
    let c = c?;
    let v = |key: &str| c.case.get(key);
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
        let e = orb.env(t, &r, &vv, &gh, 13);
        bm[kk] = norm(&e.b_eci); nu[kk] = e.nu;
        for a in 0..4 {
            let g = if a < 3 { Guid { q_off: offs[a], ..Default::default() } }
                    else { Guid { sun_eci: unit(&e.sun_rel), sun_axis: [0.0, 0.0, -1.0], roll_axis: [1.0, 0.0, 0.0], ..Default::default() } };
            let q = guidance(if a < 3 { 0 } else { 4 }, &r, &vv, t, &g).q;
            let p = torques::torques(&q, &r, &e.v_rel, &e.b_eci, &e.sun_rel, e.nu, e.p_srp, e.rho, &c.inertia, &facets, &c.m_res, c.mu, c.env_on);
            tau[kk][a] = add(&add(&p[0], &p[1]), &add(&p[2], &p[3]));
        }
    }
    let mut d = Demand { case: c.case.id.clone(), period_s: t_orb, ..Default::default() };
    for a in 0..4 {
        let mut h = [0.0; 3];
        let mut hs = Vec::with_capacity(n);
        for kk in 0..n { for i in 0..3 { h[i] += tau[kk][a][i]*dt; } hs.push(h); }
        let hend = hs[n - 1];
        d.tau_peak[a] = tau.iter().map(|x| norm(&x[a])).fold(0.0, f64::max);
        for i in 0..3 { d.tau_axis_peak[i][a] = tau.iter().map(|x| x[a][i].abs()).fold(0.0, f64::max); }
        d.h_cyclic[a] = (0..n).map(|kk| { let f = kk as f64*dt/t_orb; norm(&sub(&hs[kk], &scale(&hend, f))) }).fold(0.0, f64::max);
        d.h_secular_orbit[a] = norm(&hend);
    }
    let ia = (0..4).fold(0, |b, a| if d.tau_peak[a] > d.tau_peak[b] { a } else { b });
    d.tau_dist = d.tau_peak[ia]; d.worst_attitude = ATT[ia].into();
    d.h_dist = (0..4).map(|a| d.h_cyclic[a] + 0.25*d.h_secular_orbit[a]).fold(f64::MIN, f64::max);
    d.h_secular = d.h_secular_orbit.iter().cloned().fold(f64::MIN, f64::max);
    d.b_min = bm.iter().cloned().fold(f64::MAX, f64::min);
    d.b_mean = bm.iter().sum::<f64>()/n as f64;
    d.eclipse_frac = nu.iter().filter(|x| **x < 0.5).count() as f64/n as f64;
    let dflt = |x: f64, dv: f64, note: &str, notes: &mut Vec<String>| if x.is_nan() { notes.push(note.into()); dv } else { x };
    let mut notes = vec![];
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
            "notes": self.notes, "class": if self.fine { "fine" } else { "coarse" }})
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

/// asils.sizing.rw
pub fn rw(d: &Demand, k: &Knobs) -> Value {
    let s = k.s("rw");
    let (h, tau) = (d.h_req.max(1e-3)*s, d.tau_req.max(1e-4)*s);
    let w = 6000.0*2.0*PI/60.0; let j = h/w;
    let r = 0.021*(h/0.01).powf(0.2); let mr = j/(0.9*r*r);
    let us = mr*2.5e-3/w; let ud = us*r/2.0;
    let pst = 0.2 + 30.0*h; let ppk = pst + tau*w/0.5;
    let (dia, hgt) = (2.4*r + 0.010, 0.6*r + 0.020);
    let mut p = part(&d.case, "RW", "reaction_wheel", "Sized reaction wheel (benchmark)", "bought");
    p["nominal"] = json!({"h_max_Nms": h, "torque_max_Nm": tau, "speed_max_rad_s": w, "rotor_inertia_kgm2": j, "rotor_radius_m": r, "rotor_mass_kg": mr,
        "friction_coulomb_Nm": 1e-5*(h/0.01).sqrt(), "friction_viscous_Nms": 1e-8, "static_imbalance_kgm": us, "dynamic_imbalance_kgm2": ud,
        "power_steady_W": pst, "power_peak_W": ppk, "mass_kg": 2.2*mr + 0.06, "volume_L": PI*dia*dia/4.0*hgt*1e3});
    p["dispersion"] = json!({"torque_scale": {"dist": "normal", "mean": 1, "sigma": 0.01}, "friction_scale": {"dist": "uniform", "lo": 0.5, "hi": 2.0},
        "axis_misalignment_rad": {"dist": "normal", "mean": 0, "sigma": 0.001}});
    p["sizing"] = json!({"h_req_Nms": d.h_req, "tau_req_Nm": d.tau_req, "scale": s, "floor": "h >= 1 mNms, tau >= 0.1 mNm",
        "law": "rim flywheel 6000 rpm, ISO 1940 G2.5, anchored on SYN-RW-10"});
    p
}

/// asils.sizing.cmg (variable: VSCMG)
pub fn cmg(d: &Demand, k: &Knobs, variable: bool) -> Value {
    let s = k.s(if variable { "vscmg" } else { "cmg" });
    let h0 = (d.h_req/2.0).max(1e-3)*s; let tau = d.tau_req.max(1e-4)*s;
    let w = 800.0; let j = h0/w; let r = 0.015*(h0/0.004).powf(0.2); let mr = j/(0.9*r*r);
    let gr = (1.2*tau/h0).max(0.5).min(3.0);
    let us = mr*2.5e-3/w; let ud = us*r/2.0;
    let (dia, hgt) = (2.4*r + 0.015, 2.4*r + 0.020);
    let (mut m, mut pst) = (2.5*mr + 0.06, 0.2 + 30.0*h0);
    let (tag, kind, name) = if variable { m *= 1.1; pst += 0.1; ("VSCMG", "vscmg", "Sized variable-speed CMG (benchmark, one of 4)") }
                            else { ("CMG", "cmg", "Sized single-gimbal CMG (benchmark, one of 4)") };
    let mut p = part(&d.case, tag, kind, name, "bought");
    p["nominal"] = json!({"rotor_momentum_Nms": h0, "rotor_inertia_kgm2": j, "rotor_radius_m": r, "rotor_mass_kg": mr, "rotor_speed_rad_s": w,
        "rotor_torque_max_Nm": (0.2*tau).max(1e-4), "gimbal_rate_max_rad_s": gr, "gimbal_power_W": 0.3*gr/1.5, "power_steady_W": pst,
        "static_imbalance_kgm": us, "dynamic_imbalance_kgm2": ud, "mass_kg": m, "volume_L": PI*dia*dia/4.0*hgt*1e3});
    p["dispersion"] = json!({"torque_scale": {"dist": "normal", "mean": 1, "sigma": 0.01}, "axis_misalignment_rad": {"dist": "normal", "mean": 0, "sigma": 0.001}});
    if variable {
        p["nominal"]["h_max_Nms"] = json!(2.0*h0); p["nominal"]["friction_coulomb_Nm"] = json!(1e-5); p["nominal"]["friction_viscous_Nms"] = json!(1e-8);
        p["dispersion"]["friction_scale"] = json!({"dist": "uniform", "lo": 0.5, "hi": 2.0});
    }
    p["sizing"] = json!({"h_req_Nms": d.h_req, "tau_req_Nm": d.tau_req, "units": 4, "scale": s,
        "law": "pyramid 54.74 deg, h0 = h_req/2, rotor 800 rad/s, ISO 1940 G2.5, anchored on TRN-CMG-1"});
    p
}

/// The fluid loop, X, Y, Z rings, each designed with its electromagnetic DC conduction pump
/// (empump.rs): momentum h_req, torque tau_req, for the knob fmr_lambda [kg/W].
pub fn fmr(d: &Demand, k: &Knobs, bx: [f64; 3]) -> Vec<Value> {
    let faces = [bx[1]*bx[2], bx[0]*bx[2], bx[0]*bx[1]];
    let per = [2.0*(bx[1] + bx[2]), 2.0*(bx[0] + bx[2]), 2.0*(bx[0] + bx[1])];
    let h = d.h_req.max(2e-4)*k.s("fmr");
    let tau = d.tau_req.max(1e-5)*k.s("fmr");
    let ax = ["X", "Y", "Z"];
    (0..3).map(|i| {
        let (s0, l1) = (0.8*faces[i], 0.8*per[i]);
        let ds = empump::design(h, tau, s0, l1, k.fmr_lambda).expect("no feasible pump design");
        let mut nm = ds.json(k.fmr_lambda);
        for (key, val) in [("fluid", json!("galinstan")), ("fluid_density_kg_m3", json!(6440.0)), ("fluid_viscosity_Pa_s", json!(0.0024)),
                           ("pump_type", json!("dc-conduction, electromagnet")), ("melt_point_K", json!(254)), ("dipole_max_Am2", json!(0)),
                           ("dipole_per_amp_Am2_per_A", json!(0)), ("current_max_A", json!(0)), ("volume_L", json!(faces[i]*0.006*1e3 + 0.02))] {
            nm[key] = val;
        }
        let eta = ds.eta_cruise.max(0.01);
        let mut p = json!({"part_number": format!("SZ-{}-FMR-{}", d.case, ax[i]), "kind": "magneto_fluidic_panel",
            "name": format!("Sized fluid momentum loop with electromagnetic pump, {} axis (our product) — {}", ax[i], d.case), "status": "sized",
            "source": "adcs-design (empump)", "made": "in-house", "descriptor_version": 1});
        p["nominal"] = nm;
        p["dispersion"] = json!({"friction_scale": {"dist": "uniform", "lo": 0.8, "hi": 1.2}, "pump_efficiency": {"dist": "uniform", "lo": 0.7*eta, "hi": 1.3*eta},
            "flow_sensor_noise_m_s": {"dist": "normal", "mean": 0, "sigma": k.fmr_flow_sigma}, "axis_misalignment_rad": {"dist": "normal", "mean": 0, "sigma": 0.005}});
        p["sizing"] = json!({"h_req_Nms": d.h_req, "tau_req_Nm": d.tau_req, "flow_sensor_sigma_m_s": k.fmr_flow_sigma, "face_m2": faces[i], "scale": k.s("fmr"), "lambda_kg_per_W": k.fmr_lambda,
            "pareto": empump::pareto(h, tau, s0, l1),
            "law": "galinstan loop + DC conduction pump with an electromagnet, designed together: least mass + lambda x steady power (empump.rs)"});
        p
    }).collect()
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
fn budget(fill: &[Value], lookup: &dyn Fn(&str) -> Result<Value, String>) -> Result<Value, String> {
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
pub fn size_all(root: &Path, case_file: &Path, k: &Knobs, out: &Path) -> Result<Value, String> {
    let d = demand(root, case_file, k)?;
    let case = d.case.clone();
    let bx = [0.34, 0.10, 0.10];
    let (pm, pmp) = mtq(&d, k);
    let fm = fmr(&d, k, bx);
    let parts: Vec<(&str, Value)> = vec![("mtq", pm), ("mtqp", pmp), ("rw", rw(&d, k)), ("cmg", cmg(&d, k, false)), ("vscmg", cmg(&d, k, true)),
        ("fmr_x", fm[0].clone()), ("fmr_y", fm[1].clone()), ("fmr_z", fm[2].clone()), ("rcs", rcs(&d, k, bx))];
    for dir in ["parts", "products"] { std::fs::create_dir_all(out.join(dir)).map_err(|e| e.to_string())?; }
    let mut by_pn: BTreeMap<String, Value> = BTreeMap::new();
    for (_, p) in &parts {
        let pn = json::s(p, "part_number", "").to_string();
        std::fs::write(out.join("parts").join(format!("{pn}.json")), serde_json::to_string(p).unwrap()).map_err(|e| e.to_string())?;
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
        std::fs::write(out.join("parts").join(format!("{gyro_id}.json")), serde_json::to_string(&g).unwrap()).map_err(|e| e.to_string())?;
        by_pn.insert(gyro_id.clone(), g);
    }
    let pn = |key: &str| parts.iter().find(|x| x.0 == key).map(|x| json::s(&x.1, "part_number", "").to_string()).unwrap_or_default();
    let lookup = |id: &str| -> Result<Value, String> {
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
        std::fs::write(out.join("products").join(format!("SZ-{case}-{id}.json")), serde_json::to_string(&pr).unwrap()).map_err(|e| e.to_string())?;
        let b = budget(pr["fill"].as_array().unwrap(), &lookup)?;
        families.insert(id.to_string(), json!({"product": format!("SZ-{case}-{id}"), "role": fa["role"], "label": label,
            "mass_kg": b["mass_kg"], "power_W": b["power_W"], "volume_L": b["volume_L"], "items": b["items"]}));
    }
    let z = json!({"schema": "adcs-sizing/1", "case": case, "class": if d.fine { "fine" } else { "coarse" }, "star_tracker": st_fit,
        "demand": d.json(), "knobs": k.json(), "families": families,
        "parts": parts.iter().map(|(a, b)| (a.to_string(), b.clone())).collect::<serde_json::Map<_, _>>()});
    std::fs::write(out.join("sizing.json"), serde_json::to_string_pretty(&z).unwrap()).map_err(|e| e.to_string())?;
    Ok(z)
}
