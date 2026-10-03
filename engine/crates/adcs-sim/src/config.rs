//! Case + scenario + product -> everything a run needs (asils.config + asils.fsw.init/select):
//! the plant's truth parameters and the flight software's adcs-fswcfg/1 parameters.
use crate::error::Error;
use crate::case::Case;
use crate::json::{self, get};
use crate::lqr;
use crate::product::Dev;
use adcs_fsw::params::{Params, MODE_NONE};
use adcs_sim_core::actuators::Kind;
use adcs_sim_core::{time, NR};
use serde_json::Value;
use std::collections::BTreeMap;
use std::f64::consts::PI;
use std::path::Path;

/// Controller states in adcs_mode_t order (asils.fsw.modes).
pub const MODES: [&str; 11] = ["detumble", "nadir_mtq", "nadir_fine", "target_fine", "slew_fine", "spinup", "sun_spin", "detumble_rcs", "sun_acq_rotor", "sun_mtq", "sun_fine"];
pub const MISSION: [&str; 11] = ["detumble", "nadir_pointing", "nadir_pointing", "target_pointing", "slew", "sun_acquisition", "sun_acquisition", "detumble", "sun_acquisition", "sun_referencing", "sun_referencing"];
/// Guidance law of each state (-1 none; 0 nadir, 1 target, 2 slew, 3 inertial, 4 sun).
pub const GUID: [i32; 11] = [-1, 0, 0, 1, 2, -1, -1, -1, -1, 4, 4];

pub fn mode_index(name: &str) -> Result<u8, Error> {
    let n = match name { "nadir_rw" => "nadir_fine", "target_rw" => "target_fine", "slew_rw" => "slew_fine", x => x };
    MODES.iter().position(|&m| m == n).map(|i| i as u8).ok_or_else(|| Error::refused(format!("unknown controller state {name}")))
}

#[derive(Clone, Debug)]
pub struct Fault { pub t_s: f64, pub kind: String, pub index: usize, pub value: [f64; 3],
    /// when the device answers again (a magnetometer or a GNSS receiver back from silence); None: for good
    pub end_s: Option<f64> }

#[derive(Clone, Debug)]
pub struct Config {
    pub id: String, pub scenario: Value, pub case: Case, pub dev: Dev, pub seed: u64,
    pub epoch_utc: [f64; 6], pub jd0: f64,
    pub alt_km: f64, pub inc_deg: f64, pub ecc: f64, pub ltan_h: f64, pub u0_deg: f64, pub orbit_step_s: f64, pub period_s: f64, pub mu: f64,
    pub zonal_max: usize, pub third_body: bool, pub drag: bool, pub srp: bool, pub density_scale: f64,
    /// truth orbit/environment: "pop" (the POP port, as the MATLAB twin) or "fast" (analytic)
    pub orbit_model: String,
    /// space weather (asils.config P.env): F10.7, F10.7a, Kp, ap
    pub f107: f64, pub f107a: f64, pub kp: f64, pub ap: f64,
    pub igrf_nmax: usize, pub env_dt_s: f64, pub env_on: [bool; 4],
    pub mass_kg: f64, pub inertia: [[f64; 3]; 3], pub box_m: [f64; 3], pub cm_offset_m: [f64; 3], pub aref_m2: f64, pub cd: f64, pub refl: f64,
    pub sigma_n: f64, pub sigma_t: f64, pub vb_ratio: f64, pub spec_frac: f64, pub m_res: [f64; 3],
    pub duration_s: f64, pub dt: f64, pub record_dt: f64,
    pub params: Params, pub alg: BTreeMap<String, String>, pub faults: Vec<Fault>, pub gd_kind0: i32, pub h_t_rot: [f64; NR],
    pub spin_dps: f64,
    /// the case's flexible mode (section `flex`, all or none); None: a rigid body
    pub flex: Option<adcs_sim_core::plant::Flex>,
    /// the scenario file this run was built from, and the overrides given with it
    pub scenario_file: String, pub overrides: Vec<(String, String)>,
}

/// The algorithms the engine flies, per slot: each id maps to a law in the flight software
/// (the `match` arms in `Config::build`). An algorithm in the registry but not here flies only
/// in the MATLAB twin, and a scenario that asks for it on the engine is refused.
pub const FLOWN: [(&str, &[&str]); 7] = [
    ("detumble", &["bdot_gyro", "bdot_mag", "bdot_bangbang", "genbdot_l1"]),
    ("attitude", &["mekf"]),
    ("pointing", &["pid", "lqr", "smc"]),
    ("mtq_pointing", &["mtq_pd", "mtq_lqr", "mtq_smc", "mtq_rate_damp", "mtq_lovera2004", "mtq_celani2015", "mtq_avanzini2021", "mtq_celani2026", "mtq_tango2013"]),
    ("sun_acquisition", &["sunspin_l1l2", "sunspin_l1l2_e2", "sunspin_damped", "sunspin_deruiter2011", "sun_boresight_celani2026"]),
    ("allocation", &["rotor_pinv", "idmas_split", "cmg_sr", "vscmg_sr"]),
    ("thrusters", &["rcs_pwm"]),
];

fn select(root: &Path, dev: &Dev, s: &Value) -> Result<BTreeMap<String, String>, Error> {
    let has = dev.caps();
    let dflt: [(&str, &[&str]); 7] = [("detumble", &["bdot_gyro", "bdot_mag"]), ("attitude", &["mekf"]), ("pointing", &["pid"]), ("mtq_pointing", &["mtq_pd"]),
        ("sun_acquisition", &["sunspin_l1l2"]), ("allocation", &["cmg_sr", "vscmg_sr", "idmas_split", "rotor_pinv"]), ("thrusters", &["rcs_pwm"])];
    let mut pick: BTreeMap<String, String> = BTreeMap::new();
    for src in [dev.selected.as_ref(), s.get("fsw").and_then(|f| get(f, "algorithms"))].into_iter().flatten() {
        if let Some(o) = src.as_object() { for (k, v) in o { if let Some(x) = v.as_str() { pick.insert(k.clone(), x.into()); } } }
    }
    if let Some(x) = pick.remove("sun_spin") { pick.entry("sun_acquisition".into()).or_insert(x); }   // legacy slot name
    for (slot, id) in &pick {
        let Some((_, ids)) = FLOWN.iter().find(|(sl, _)| sl == slot) else {
            return Err(Error::refused(format!("algorithm slot {slot:?}: no such slot; the slots are {}", FLOWN.iter().map(|x| x.0).collect::<Vec<_>>().join(", "))));
        };
        if !ids.contains(&id.as_str()) {
            return Err(Error::refused(format!("algorithm {id} ({slot}): the engine does not fly it (it flies {}); it runs in the MATLAB twin only", ids.join(", "))));
        }
    }
    let load = |id: &str| -> Result<(String, Vec<String>), Error> {
        let a = json::read(&root.join("data/algorithms").join(format!("{id}.json"))).map_err(|_| Error::refused(format!("no algorithm {id} in the registry")))?;
        let needs = match a.get("needs") { Some(Value::Array(x)) => x.iter().filter_map(|v| v.as_str().map(String::from)).collect(), Some(Value::String(x)) => vec![x.clone()], _ => vec![] };
        Ok((json::s(&a, "slot", "").to_string(), needs))
    };
    let mut alg = BTreeMap::new();
    for (sl, cands) in dflt {
        if let Some(id) = pick.get(sl) {
            let (slot, needs) = load(id)?;
            let miss: Vec<_> = needs.iter().filter(|n| !has.contains(&n.as_str())).cloned().collect();
            if !miss.is_empty() { return Err(Error::refused(format!("algorithm {id} ({sl}) cannot fly on {}: it needs {}", dev.id, miss.join(", ")))); }
            if slot != sl { return Err(Error::refused(format!("algorithm {id} does {slot}, not {sl}"))); }
            alg.insert(sl.to_string(), id.clone());
        } else {
            let mut got = String::new();
            for c in cands { let (_, needs) = load(c)?; if needs.iter().all(|n| has.contains(&n.as_str())) { got = c.to_string(); break; } }
            alg.insert(sl.to_string(), got);
        }
    }
    Ok(alg)
}

/// An id that names a file inside one of the data folders (a scenario, a case): letters,
/// digits, `_`, `-` and `.`, never a path. A path is given as a path (`--case F`,
/// `<scenario>.json`), so an id can never reach outside the folder it names.
pub fn check_id(what: &str, id: &str) -> Result<(), Error> {
    let ok = !id.is_empty() && id.len() <= 128 && !id.starts_with('.')
        && id.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'));
    if ok { Ok(()) } else { Err(Error::refused(format!("{what} {id:?} is not an id: use letters, digits, _ - and . only, or give a path to the file"))) }
}

/// The sections a scenario has; an override outside them would be read by nothing.
pub const SCENARIO_SECTIONS: [&str; 10] = ["case", "fsw", "id", "initial", "label", "metrics", "product", "schema", "time", "faults"];

/// Apply `--set k=v` to the scenario. The key must be a dotted path of names inside a
/// section the scenario has; the value is JSON (a number, true/false, a list) or plain
/// text. It must keep the type of what it replaces, so `time.dt_s=abc` is refused rather
/// than read as "not given" and silently replaced by the default.
pub fn set_override(s: &mut Value, k: &str, v: &str) -> Result<(), Error> {
    let parts: Vec<&str> = k.split('.').collect();
    if parts.iter().any(|p| p.is_empty() || !p.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')) {
        return Err(Error::refused(format!("--set {k}: a key is dotted names (letters, digits, _), such as fsw.rw_bandwidth")));
    }
    if !SCENARIO_SECTIONS.contains(&parts[0]) {
        return Err(Error::refused(format!("--set {k}: a scenario has no section {:?}; it has {}; engine settings are engine.<name>", parts[0], SCENARIO_SECTIONS.join(", "))));
    }
    if !crate::schema::settable(k) {
        return Err(Error::refused(format!("--set {k}: the engine does not read this key (misspelt? the keys are in engine/crates/adcs-sim/src/schema.rs)")));
    }
    let lower = v.trim().to_ascii_lowercase();
    if ["nan", "inf", "+inf", "-inf", "infinity", "+infinity", "-infinity"].contains(&lower.as_str()) {
        return Err(Error::refused(format!("--set {k}={v}: not a finite number")));
    }
    let parsed: Value = serde_json::from_str(v).unwrap_or_else(|_| Value::String(v.to_string()));
    let mut cur = &*s;
    for p in &parts {
        match cur.get(*p) { Some(x) => cur = x, None => { cur = &Value::Null; break; } }
    }
    let kind = |x: &Value| match x { Value::Number(_) => "a number", Value::String(_) => "text", Value::Bool(_) => "true or false", Value::Array(_) => "a list", Value::Object(_) => "a section", Value::Null => "" };
    // "case:<key>" means "read it from the case"; a number given here replaces that reference
    let reference = cur.as_str().map(|x| x.starts_with("case:")).unwrap_or(false) && parsed.is_number();
    if !cur.is_null() && !reference && kind(cur) != kind(&parsed) {
        return Err(Error::refused(format!("--set {k}={v}: the scenario has {} here, and {v:?} is {}", kind(cur), kind(&parsed))));
    }
    json::set_path(s, k, v);
    Ok(())
}

/// A number the engine reads from an override: finite, or refused.
fn finite(k: &str, v: &str) -> Result<f64, Error> {
    match v.trim().parse::<f64>() {
        Ok(x) if x.is_finite() => Ok(x),
        _ => Err(Error::refused(format!("{k}={v}: not a finite number"))),
    }
}

/// A whole number the engine reads from an override, within its range.
fn whole(k: &str, x: f64, lo: usize, hi: usize) -> Result<usize, Error> {
    if x.fract() != 0.0 || x < lo as f64 || x > hi as f64 { return Err(Error::refused(format!("{k}={x}: a whole number from {lo} to {hi}"))); }
    Ok(x as usize)
}

/// The case values every run reads; a blank one is refused here, by name, instead of
/// running the plant on NaN.
pub const CASE_NEEDS: [&str; 14] = ["orbit.alt", "orbit.inc", "orbit.ecc", "orbit.ltan", "mass.m", "mass.imin", "mass.iint", "mass.imax",
    "surface.afr", "surface.cd", "surface.refl", "surface.cpa", "magnetic.dres", "mission.epoch"];

/// The engine settings `--set engine.<name>=` can change.
/// Case keys the models do not use yet: stating one is refused with the reason, so a value is
/// never quietly ignored (docs/UPGRADE_PLAN.md B2.1). Requirements (`req.*`) are covered by the
/// traceability check instead: a stated requirement must be judged by a shipped metric.
pub const CASE_UNMODELLED: [(&str, &str); 4] = [
    ("mission.duty", "the fine-pointing duty cycle is not modelled yet (scenarios fly their mode for their whole duration)"),
    ("mass.cm", "the centre-of-mass offset comes from surface.cpa in the facet model; a separate mass.cm is not modelled"),
    ("resources.vbus", "the bus voltage is not used by any device model"),
    ("resources.nif", "the OBC data interfaces are not checked by the design loop yet (the data budget is B2.4)"),
];

pub const ENGINE_KEYS: [&str; 22] = ["engine.orbit", "engine.inertia_scale", "engine.cm_offset_m", "engine.m_res", "engine.density_scale", "engine.duration_s",
    "engine.orbit_step_s", "engine.zonal_max", "engine.igrf_nmax", "engine.f107", "engine.f107a", "engine.kp", "engine.ap", "engine.accommodation", "engine.refl", "engine.mass_kg",
    "engine.vb_ratio", "engine.spec_frac", "engine.epoch_days", "engine.ltan_h", "engine.alt_km", "engine.inertia_products"];

/// The surface model's settings when no `--set engine.*` changes them (recorded in every run's
/// manifest under "assumptions"): momentum accommodation, the ratio of the re-emitted to the
/// incoming speed, and the specular share of the reflected light. Moe & Moe (2005), LEO.
pub const ACCOMMODATION: f64 = 0.8;
pub const VB_RATIO: f64 = 0.05;
pub const SPEC_FRAC: f64 = 0.5;

/// The body of a satellite class (catalogue/classes.toml, exported to data/classes.json).
pub fn class_box(root: &Path, case: &Case) -> Result<[f64; 3], Error> {
    let f = root.join("data/classes.json");
    let all = json::read(&f)?;
    let ids: Vec<&str> = all["class"].as_array().map(|a| a.iter().filter_map(|c| c["id"].as_str()).collect()).unwrap_or_default();
    if case.class.is_empty() {
        return Err(Error::refused(format!("case {}: meta.class is blank; the engine models the body of the class it names ({})", case.id, ids.join(", "))));
    }
    let c = all["class"].as_array().and_then(|a| a.iter().find(|c| c["id"].as_str() == Some(case.class.as_str())))
        .ok_or_else(|| Error::refused(format!("case {}: meta.class = {:?} is no class in catalogue/classes.toml ({})", case.id, case.class, ids.join(", "))))?;
    let b = json::v3(&c["box_m"]).filter(|b| b.iter().all(|x| x.is_finite() && *x > 0.0))
        .ok_or_else(|| Error::malformed(format!("{}: class {} has no box_m of three positive lengths", f.display(), case.class)))?;
    Ok(b)
}

/// The engine's limits. It models low Earth orbit; the loop runs at most thirty days.
pub const ALT_KM: (f64, f64) = (150.0, 2000.0);
pub const DURATION_MAX_S: f64 = 30.0*86400.0;

/// What the flight-software parameter stages read from the case, the product and the scenario.
struct Knowns<'a> { dev: &'a Dev, fsw: &'a Value, inertia: [[f64; 3]; 3], m_res: [f64; 3], n: f64, inc_deg: f64 }

/// The start mode, the next mode and the schedule; the law each algorithm slot flies.
fn modes_and_laws(p: &mut Params, fsw: &Value, alg: &BTreeMap<String, String>) -> Result<(), Error> {
    let a_ = |k: &str| alg.get(k).cloned().unwrap_or_default();
    p.start_mode = mode_index(json::s(&fsw, "start_mode", "detumble"))?;
    let an = json::s(&fsw, "auto_next", "");
    p.auto_next = if an.is_empty() { MODE_NONE } else { mode_index(an)? };
    if let Some(sc) = get(&fsw, "schedule") {
        let list: Vec<Value> = match sc { Value::Array(x) => x.clone(), x => vec![x.clone()] };
        for (i, e) in list.iter().take(8).enumerate() { p.sched_t[i] = json::f(e, "t_s", 0.0); p.sched_mode[i] = mode_index(json::s(e, "mode", ""))?; }
        p.n_sched = list.len().min(8) as u8;
    }
    p.bdot_law = match a_("detumble").as_str() { "bdot_gyro" => 0, "bdot_bangbang" => 2, "genbdot_l1" => 3, _ => 1 };
    p.rw_law = match a_("pointing").as_str() { "lqr" => 1, "smc" => 2, _ => 0 };
    p.mtq_law = match a_("mtq_pointing").as_str() {
        "mtq_lqr" => 1, "mtq_smc" => 2, "mtq_rate_damp" => 3,
        // magnetorquer-only literature (docs/MTQ_LITERATURE.md)
        "mtq_lovera2004" => 4, "mtq_celani2015" => 5, "mtq_avanzini2021" => 6, "mtq_celani2026" => 7, "mtq_tango2013" => 8,
        _ => 0 };
    p.alloc = match a_("allocation").as_str() { "idmas_split" => 1, "cmg_sr" => 2, "vscmg_sr" => 3, _ => 0 };
    let (ecl, rzf, sl) = match a_("sun_acquisition").as_str() {
        "sunspin_l1l2_e2" => (2, 0.0, 0), "sunspin_damped" => (2, 0.5, 0),
        "sunspin_deruiter2011" => (1, 0.0, 1), "sun_boresight_celani2026" => (2, 0.0, 2),
        _ => (1, 0.0, 0) };
    p.ss_eclipse = ecl; p.ss_rz_floor = rzf; p.ss_law = sl;
    Ok(())
}

/// The guidance the start mode flies, and the payload and power axes.
fn guidance_params(p: &mut Params, k: &Knowns) {
    let (fsw, dev) = (k.fsw, k.dev);
    let g = get(&fsw, "guidance").cloned().unwrap_or(Value::Null);
    p.gd_kind = match json::s(&g, "kind", "nadir") { "target" => 1, "slew" => 2, "inertial" => 3, "sun" => 4, _ => 0 };
    p.gd_q_off = adcs_fsw::ctl::boresight_offset(&dev.boresight);
    p.gd_roll_deg = json::f(&g, "roll_deg", 0.0); p.gd_t0 = json::f(&g, "t0", 0.0); p.gd_T = json::f(&g, "T_s", 1.0);
    p.gd_axis = get(&g, "axis").and_then(json::v3).unwrap_or([0.0; 3]);
    p.gd_q_inertial = get(&g, "q_inertial").and_then(|q| q.as_array().map(|a| [a[0].as_f64().unwrap_or(0.0), a[1].as_f64().unwrap_or(0.0), a[2].as_f64().unwrap_or(0.0), a[3].as_f64().unwrap_or(1.0)])).unwrap_or([0.0, 0.0, 0.0, 1.0]);
    p.sun_axis = dev.sun_axis; p.roll_axis = dev.boresight;
    p.J = k.inertia; p.m_res_est = k.m_res;
}

/// The magnetic pointing gains: one bandwidth for every law, and the literature laws' gains on it.
fn mtq_gains(p: &mut Params, k: &Knowns) {
    let (fsw, dev, inertia, n) = (k.fsw, k.dev, k.inertia, k.n);
    p.mtq_period = 1.0; p.mtq_meas = 0.2;
    p.m_max = if dev.mtq.fitted { dev.mtq.m_max } else { 1.0 };
    let jmin = inertia[0][0].min(inertia[1][1]).min(inertia[2][2]);
    p.bdot_k = json::f(&fsw, "bdot_gain_scale", 3.0)*2.0*n*(1.0 + (k.inc_deg*PI/180.0).sin())*jmin;
    p.detumble_exit = json::f(&fsw, "detumble_exit_deg_s", 0.5)*PI/180.0;
    p.detumble_hold_s = json::f(&fsw, "detumble_hold_s", 60.0);
    let ii = [inertia[0][0], inertia[1][1], inertia[2][2]];
    // magnetic pointing: one bandwidth for every law
    let (wn, z) = (json::f(&fsw, "mtq_wn", 0.005), json::f(&fsw, "mtq_zeta", 2.0));
    for ax in 0..3 {
        p.mtq_Kp[ax] = ii[ax]*wn*wn; p.mtq_Kd[ax] = 2.0*z*ii[ax]*wn; p.mtq_Ki[ax] = 0.0;
        let th = 0.05;
        let q = [1e-12, 1.0/(th*th), 1.0/((wn*th)*(wn*th))];
        let r = 1.0/((ii[ax]*wn*wn*th)*(ii[ax]*wn*wn*th));
        p.mtq_Klqr[ax] = lqr::chain3(1.0/ii[ax], q, r, wn);
    }
    p.mtq_err_max = 0.5; p.mtq_int_max = 0.05;
    // magnetorquer-only literature laws, torque-level gains on the same bandwidth (wn, z) so the
    // laws compare on structure; fsw.mtq_gain_p / mtq_gain_d scale them (the tune node, Bruni & Celani)
    {
        let (gp, gd) = (json::f(&fsw, "mtq_gain_p", 1.0), json::f(&fsw, "mtq_gain_d", 1.0));
        let jm = (ii[0] + ii[1] + ii[2])/3.0;
        let eps = 1e-3;                                  // the papers' time-scale parameter
        p.mtq_eps = eps;
        p.mtq_k1 = gp*jm*wn*wn/(eps*eps);
        // Lovera & Astolfi multiply the rate by J; Celani does not
        p.mtq_k2 = if p.mtq_law == 4 { gd*2.0*z*wn/eps } else { gd*2.0*z*jm*wn/eps };
        // Avanzini 2021: k below 0.5 (1 + 2 sin xi_m) n; the paper flies k ~ 0.84 n, lambda 0.08
        p.mtq_k16 = gd*json::f(&fsw, "avanzini_k_over_n", 0.84)*n;
        p.mtq_lam16 = gp*json::f(&fsw, "avanzini_lambda", 0.08);
        // Celani 2026 boresight
        p.sb_kp = gp*jm*wn*wn; p.sb_kd = gd*2.0*z*jm*wn;
        // weak roll about the payload axis in nadir so the power face is held: its own PD on the roll-axis
        // inertia at roll_wn orbit rates, damping roll_zeta. A bare fraction of the boresight gain is not enough:
        // at the tuned rate gain the Floquet multiplier of the loop grows to 4 per orbit (docs/MTQ_LITERATURE.md)
        let ea = dev.boresight; let je = (0..3).map(|i| ea[i]*(0..3).map(|j| inertia[i][j]*ea[j]).sum::<f64>()).sum::<f64>();
        let wr = json::f(&fsw, "roll_wn_orbits", 3.0)*n; let zr = json::f(&fsw, "roll_zeta", 1.0);
        p.sb_kroll = json::f(&fsw, "roll_gain", 1.0)*je*wr*wr; p.sb_kdroll = 2.0*zr*je*wr;
        p.sb_roll_gate = (json::f(&fsw, "roll_gate_deg", 15.0)*PI/180.0).cos();
        // hand-over from a spinning body (P11 despin to the reference rate, then the law), and the
        // gravity-gradient feed-forward in the Sun state (at nadir the gradient is the restoring spring)
        p.ho_in_dps = json::f(&fsw, "handover_in_dps", 1.0); p.ho_out_dps = json::f(&fsw, "handover_out_dps", 0.5);
        p.ho_hold_s = json::f(&fsw, "handover_hold_s", 60.0);
        // the gradient restores the nadir attitude only inside the Lagrange region J_normal >= J_along > J_nadir;
        // outside it (e.g. a long axis along track) the nadir state cancels it too (bit 1)
        let gg_stable = {
            use adcs_fsw::ctl::{guidance, Guid};
            let (r, v) = ([7.0e6, 0.0, 0.0], [0.0, 7.5e3, 0.0]);
            let q = guidance(0, &r, &v, 0.0, &Guid { q_off: p.gd_q_off, ..Default::default() }).q;
            let a = adcs_fsw::math::dcm(&q);
            let ax = |u: [f64; 3]| { let b = adcs_fsw::math::mat3_vec(&a, &u); let jb = adcs_fsw::math::mat3_vec(&inertia, &b); b[0]*jb[0] + b[1]*jb[1] + b[2]*jb[2] };
            let (jz, ja, jn) = (ax([-1.0, 0.0, 0.0]), ax([0.0, 1.0, 0.0]), ax([0.0, 0.0, 1.0]));
            jn >= ja*(1.0 - 1e-9) && ja > jz*(1.0 + 1e-9)          // equal transverse inertias: neutral, not unstable
        };
        p.mtq_gg_ff = json::f(&fsw, "mtq_gg_ff", if gg_stable { 1.0 } else { 3.0 }) as u8;
        // TANGO frozen Riccati: P from the CARE with the orbit-averaged B_u R^-1 B_u^T; an isotropic field
        // average gives E[Gamma D Gamma]_ii = (7/15) D_i + tr(D)/15, D = J^-2 (per-axis double-integrator
        // CARE). Q is chosen (inverse LQR) so the average axis gets the common bandwidth wn, zeta; each axis
        // then gets the gain its averaged authority calls for. The paper's Q, R are unpublished.
        let r = 1.0;
        let d = [1.0/(ii[0]*ii[0]), 1.0/(ii[1]*ii[1]), 1.0/(ii[2]*ii[2])];
        let trd = d[0] + d[1] + d[2];
        let mi: Vec<f64> = (0..3).map(|ax| ((7.0/15.0)*d[ax] + trd/15.0)/r).collect();
        let mref = (mi[0] + mi[1] + mi[2])/3.0;
        let qt = mref*(jm*wn*wn*r).powi(2);
        let qw = (mref*(2.0*z*jm*wn*r).powi(2) - 2.0*(qt/mref).sqrt()).max(0.0);
        for ax in 0..3 {
            let p12 = (qt/mi[ax]).sqrt();
            let p22 = ((qw + 2.0*p12)/mi[ax]).sqrt();
            p.mtq_Pth[ax][ax] = gp*p12/r; p.mtq_Pw[ax][ax] = gd*p22/r;
        }
    }
    p.mtq_lambda = wn/(2.0*z)*2.0; p.mtq_phi = 5e-4; p.mtq_Gs = [2.0*z*wn*p.mtq_phi; 3];
}

/// The fine-pointing gains (wheels and momentum devices).
fn rw_gains(p: &mut Params, k: &Knowns) {
    let fsw = k.fsw;
    let ii = [k.inertia[0][0], k.inertia[1][1], k.inertia[2][2]];
    // fine pointing
    let (wn, z) = (json::f(&fsw, "rw_bandwidth", 0.9), json::f(&fsw, "rw_damping", 2.0));
    for ax in 0..3 {
        p.rw_Kp[ax] = ii[ax]*wn*wn; p.rw_Kd[ax] = 2.0*z*ii[ax]*wn; p.rw_Ki[ax] = 0.15*ii[ax]*wn*wn*wn;
        let th = 1e-3;
        let q = [(wn/(0.5*th))*(wn/(0.5*th)), 1.0/(th*th), 1.0/((wn*th)*(wn*th))];
        let r = 1.0/((ii[ax]*wn*wn*th)*(ii[ax]*wn*wn*th));
        p.rw_Klqr[ax] = lqr::chain3(1.0/ii[ax], q, r, wn);
    }
    p.rw_err_max = 0.2; p.rw_int_max = 0.02; p.rw_dt = 1.0/json::f(&fsw, "rw_rate_hz", 10.0);
    p.rw_lambda = wn/(2.0*z); p.rw_phi = 2e-4; p.rw_Gs = [2.0*z*wn*p.rw_phi; 3];
    p.capture_deg = json::f(&fsw, "capture_deg", 3.0); p.capture_rate_deg_s = json::f(&fsw, "capture_rate_deg_s", 1.0);
}

/// The Sun-spin and Sun-acquisition laws.
fn spin_params(p: &mut Params, k: &Knowns) {
    let (fsw, inertia) = (k.fsw, k.inertia);
    // Standard Code L1/L2 spin
    p.ss_k_l1 = json::f(&fsw, "l1_gain", 1e6); p.ss_spin_dps = json::f(&fsw, "spin_rate_dps", 6.0); p.ss_sigma0 = 1.0;
    p.ss_z_in_dps = 0.5; p.ss_perp_in_dps = 0.5; p.ss_sun_min = 0.05; p.ss_t_check_s = 60.0; p.ss_omega_max_dps = 100.0;
    p.ss_dwell_in_s = 60.0; p.ss_k1 = 0.01*json::f(&fsw, "ss_gain", 1.0); p.ss_k2 = 0.05*json::f(&fsw, "ss_gain", 1.0);
    // de Ruiter 2011: k1 > 1, k2 > 0 (paper 1.5, 0.5, with k2 in inertia units here: 0.5 J_zz); k like He's k1
    p.ss_dr_k = 0.01*json::f(&fsw, "ss_gain", 1.0); p.ss_dr_k1 = 1.5; p.ss_dr_k2 = 0.5*inertia[2][2]; p.ss_perp_out_dps = json::f(&fsw, "sun_spin_perp_out_dps", 1.0); p.ss_omega_exit_dps = 2.0; p.ss_dwell_out_s = json::f(&fsw, "sun_spin_dwell_out_s", 30.0);
    p.sa_w_max_deg_s = json::f(&fsw, "sun_acq_rate_deg_s", 1.0); p.sa_kd = 0.1; p.sa_done_deg = 10.0; p.sa_done_hold_s = 60.0;
}

/// The momentum devices, in their NOMINAL geometry; returns each rotor's target momentum.
fn rotor_params(p: &mut Params, k: &Knowns) -> [f64; NR] {
    let fsw = k.fsw;
    // momentum devices (NOMINAL geometry)
    let x = &k.dev.mex;
    p.nr = x.n as u8; p.ng = x.ng as u8;
    let h_bias = json::f(&fsw, "wheel_bias_Nms", 2e-3);
    let mut h_t_rot = [0.0; NR];
    for i in 0..x.n {
        p.rot_kind[i] = match x.kind[i] { Kind::Rw => 0, Kind::Fmr => 1, Kind::Cmg => 2, Kind::Vscmg => 3 };
        p.rot_a0[i] = x.a0[i]; p.rot_gi[i] = x.gi[i] as u8;
        p.rot_tmax[i] = x.torque_max[i]; p.rot_hmax[i] = x.h_max[i]; p.rot_h0[i] = x.h0[i];
        h_t_rot[i] = match x.kind[i] { Kind::Rw => h_bias.min(0.25*x.h_max[i]), Kind::Cmg | Kind::Vscmg => x.h0[i], Kind::Fmr => 0.0 };
    }
    for j in 0..x.ng { p.gim_axis[j] = x.g[j]; }
    p.gim_rate_max = if x.ng > 0 { x.gimbal_rate_max } else { 1.0 };
    p.h_bias = h_bias; p.dump_k = json::f(&fsw, "dump_gain", 2e-3);
    p.cmg_lam0 = 1e-9; p.cmg_mu = 10.0; p.cmg_k_null = 0.002; p.fdir_s = 3.0;
    // windowed rotor FDIR (fsw/pseudocode/07): 120 s windows; a commanded change under 0.5 % of h_max is not judged
    p.fdir_win_s = 120.0; p.fdir_h_frac = 0.005;
    h_t_rot
}

/// The thrusters, and the assist, dump and detumble settings.
fn rcs_params(p: &mut Params, k: &Knowns) {
    let (fsw, dev) = (k.fsw, k.dev);
    // thrusters
    if dev.rcs.fitted {
        p.nc = dev.rcs.nc as u8;
        for j in 0..dev.rcs.nc { p.rcs_tau[j] = dev.rcs.tau[j]; }
        p.rcs_mib = dev.rcs.mib; p.rcs_res = dev.rcs.res;
    }
    p.rcs_assist = json::b(&fsw, "rcs_assist", true) as u8; p.rcs_assist_frac = 0.8;
    p.rcs_dump = json::b(&fsw, "rcs_dump", true) as u8; p.rcs_dump_hi = 4e-3; p.rcs_dump_lo = 1e-3; p.rcs_dump_k = 0.05;
    p.rcsd_T_damp_s = json::f(&fsw, "rcs_damp_s", 20.0); p.rcsd_deadband_deg_s = 0.2; p.rcsd_period_s = 1.0;
}

/// The sensors the software is told about, and the estimator's measurement sigmas.
fn sensor_params(p: &mut Params, k: &Knowns) {
    let (fsw, dev) = (k.fsw, k.dev);
    // sensors the software is told about
    p.has_gyro = dev.gyro.fitted as u8; p.has_st = dev.st.fitted as u8; p.has_sun = (dev.sun.fitted || dev.css.fitted) as u8;
    p.has_es = dev.es.fitted as u8; p.has_gps = dev.gps.fitted as u8; p.n_heads = dev.st.nh as u8;
    for h in 0..dev.st.nh { p.st_bs[h] = dev.st.bs[h]; }
    p.st_noise_cross = dev.st.noise_cross; p.st_noise_roll = dev.st.noise_roll; p.st_latency = dev.st.latency; p.st_coast_s = 900.0;
    p.gyro_arw = dev.gyro.arw; p.gyro_rrw = dev.gyro.rrw; p.es_noise = dev.es.noise;
    // measurement sigmas from the fitted devices: the fine Sun sensor's accuracy and bias, the coarse cells'
    // albedo error (0.3 albedo puts the vector up to ~8 deg off); the magnetometer's direction error plus its
    // bias and noise over the field strength (applied onboard, 1-3 deg at 18-50 uT)
    let hyp = |a: f64, b: f64| { let (a, b) = (if a.is_finite() { a } else { 0.0 }, if b.is_finite() { b } else { 0.0 }); (a*a + b*b).sqrt() };
    p.mekf_sig_sun = if dev.sun.fitted { hyp(dev.sun.noise, dev.sun.bias_sigma).max(0.005) } else { (0.5*dev.css.albedo).max(0.1) };
    p.mekf_sig_mag = hyp(dev.mag.misalign, dev.mag.sf_sigma).max(0.01);
    let fin = |x: f64| if x.is_finite() { x } else { 0.0 };
    p.mekf_mag_err_T = (fin(dev.mag.bias_t).powi(2) + 3.0*fin(dev.mag.bias_sigma).powi(2) + 3.0*fin(dev.mag.noise).powi(2)).sqrt();
    p.mekf_gate = json::f(&fsw, "mekf_gate", 16.27); p.mekf_rej_max = json::f(&fsw, "mekf_rej_max", 30.0);
    p.mekf_meas_scale = 1.0;
    p.gnss_ecef = 1;
    p.gps_latency = if dev.gps.fitted { dev.gps.latency } else { 0.0 };
    p.gd_yaw_flip = json::b(&fsw, "yaw_flip", true) as u8; p.gd_flip_hyst = 0.1;
    p.rate_lpf_s = json::f(&fsw, "rate_lpf_s", 0.3); p.igrf_nmax = 10;
    if !p.st_noise_cross.is_finite() { p.st_noise_cross = 0.0; }
    for x in [&mut p.st_noise_roll, &mut p.st_latency, &mut p.gyro_arw, &mut p.gyro_rrw, &mut p.es_noise, &mut p.rcs_mib, &mut p.rcs_res] { if !x.is_finite() { *x = 0.0; } }
}

/// The faults a scenario injects, each naming a device the product carries and a unit it has.
fn faults(s: &Value, dev: &Dev) -> Result<Vec<Fault>, Error> {

    let faults: Vec<Fault> = match s.get("faults") {
        Some(Value::Array(a)) => a.iter().map(|f| Fault { t_s: json::f(f, "t_s", 0.0), kind: json::s(f, "kind", "").into(), index: json::f(f, "index", 0.0) as usize, value: get(f, "value").and_then(json::v3).unwrap_or([0.0; 3]),
            end_s: get(f, "end_s").and_then(|v| v.as_f64()) }).collect(),
        _ => vec![],
    };
    // every fault names a device the product carries, and a unit it has (numbered from 1)
    for (i, f) in faults.iter().enumerate() {
        let raw = s.pointer(&format!("/faults/{i}")).cloned().unwrap_or(Value::Null);
        let (units, what) = match f.kind.as_str() {
            "rotor_fail" => (dev.mex.n, "rotors"), "gimbal_stuck" => (dev.mex.ng, "gimbals"),
            "st_head_fail" => (if dev.st.fitted { dev.st.nh } else { 0 }, "star-tracker heads"),
            "coil_fail" => (if dev.mtq.fitted { dev.mtq.n } else { 0 }, "magnetorquer coils"),
            "rcs_valve_fail" => (if dev.rcs.fitted { dev.rcs.nc } else { 0 }, "thruster couples"),
            "gyro_bias_step" => (dev.gyro.fitted as usize, "gyros"), "gps_outage" => (dev.gps.fitted as usize, "GNSS receivers"),
            "mag_fail" => (dev.mag.fitted as usize, "magnetometers"),
            k => return Err(Error::refused(format!("faults[{i}].kind = {k:?}: no such fault"))),
        };
        if units == 0 { return Err(Error::refused(format!("faults[{i}] {}: product {} has no {what}", f.kind, dev.id))); }
        if matches!(f.kind.as_str(), "rotor_fail" | "gimbal_stuck" | "st_head_fail" | "coil_fail" | "rcs_valve_fail") && !(1..=units).contains(&f.index) {
            return Err(Error::refused(format!("faults[{i}] {}: index {} — product {} has {what} 1 to {units}", f.kind, f.index, dev.id)));
        }
        if let Some(e) = f.end_s {
            if !matches!(f.kind.as_str(), "mag_fail" | "gps_outage") {
                return Err(Error::refused(format!("faults[{i}] {}: end_s is for a device that can answer again (mag_fail, gps_outage)", f.kind)));
            }
            if !(e > f.t_s) { return Err(Error::refused(format!("faults[{i}] {}: end_s {e} s is not after t_s {} s", f.kind, f.t_s))); }
        }
        if f.kind == "gyro_bias_step" && raw.get("value").and_then(json::v3).is_none() {
            return Err(Error::refused(format!("faults[{i}] gyro_bias_step: value must be the bias step [x, y, z] in rad/s")));
        }
    }
    Ok(faults)
}

/// The engine settings given with `--set engine.<name>=` (truth dispersions, models, the surface).
fn apply_engine(cfg: &mut Config, eng: &[(String, String)]) -> Result<(), Error> {
    for (k, val) in eng {
        let (k, val) = (k.clone(), val.clone());
        if k == "engine.orbit" {
            if val != "pop" && val != "fast" { return Err(Error::refused("engine.orbit: pop | fast")); }
            cfg.orbit_model = val.clone();
            continue;
        }
        // truth dispersions (asils.campaign.draw): the plant changes, the flight software keeps
        // the nominal (ground-calibrated) inertia and residual dipole it was loaded with
        if val.trim_start().starts_with('[') {
            let a: Vec<f64> = serde_json::from_str(&val).map_err(|_| Error::refused(format!("{k}: not a [x, y, z] vector")))?;
            if a.len() != 3 { return Err(Error::refused(format!("{k}: needs 3 values"))); }
            if a.iter().any(|x| !x.is_finite()) { return Err(Error::refused(format!("{k}: every value is a finite number"))); }
            match k.as_str() {
                "engine.inertia_scale" => for i in 0..3 { cfg.inertia[i][i] *= a[i]; },
                // products of inertia, each a fraction of sqrt(I_ii I_jj): [xy, xz, yz]; the body stays a body
                "engine.inertia_products" => {
                    for (f, (i, j)) in a.iter().zip([(0, 1), (0, 2), (1, 2)]) {
                        if f.abs() >= 1.0 { return Err(Error::refused(format!("{k}: each product is a fraction of sqrt(I_ii I_jj) below 1, not {f}"))); }
                        let pij = f*(cfg.inertia[i][i]*cfg.inertia[j][j]).sqrt();
                        cfg.inertia[i][j] = pij; cfg.inertia[j][i] = pij;
                    }
                    let m = &cfg.inertia;
                    let d2 = m[0][0]*m[1][1] - m[0][1]*m[1][0];
                    let d3 = m[0][0]*(m[1][1]*m[2][2] - m[1][2]*m[2][1]) - m[0][1]*(m[1][0]*m[2][2] - m[1][2]*m[2][0]) + m[0][2]*(m[1][0]*m[2][1] - m[1][1]*m[2][0]);
                    if !(d2 > 0.0 && d3 > 0.0) { return Err(Error::refused(format!("{k}: {a:?} gives an inertia that is not positive definite"))); }
                }
                "engine.cm_offset_m" => cfg.cm_offset_m = [a[0], a[1], a[2]],
                "engine.m_res" => cfg.m_res = [a[0], a[1], a[2]],
                _ => return Err(Error::refused(format!("unknown engine vector override {k}"))),
            }
            continue;
        }
        let x = finite(&k, &val)?;
        match k.as_str() {
            "engine.density_scale" => cfg.density_scale = x,
            "engine.duration_s" => cfg.duration_s = x,
            "engine.orbit_step_s" => cfg.orbit_step_s = x,
            "engine.zonal_max" => cfg.zonal_max = whole(&k, x, 1, 6)?,
            "engine.igrf_nmax" => cfg.igrf_nmax = whole(&k, x, 1, 13)?,
            "engine.f107" => cfg.f107 = x,
            "engine.f107a" => cfg.f107a = x,
            "engine.kp" => cfg.kp = x,
            "engine.ap" => cfg.ap = x,
            "engine.accommodation" => { cfg.sigma_n = x; cfg.sigma_t = x; }
            "engine.refl" => cfg.refl = x,
            "engine.vb_ratio" => cfg.vb_ratio = x,
            "engine.spec_frac" => cfg.spec_frac = x,
            "engine.mass_kg" => cfg.mass_kg = x,
            // the season: the mission starts this many days later; the date is known on board, so
            // the flight software's epoch moves with the truth's
            "engine.epoch_days" => {
                if x.abs() > 3660.0 { return Err(Error::refused(format!("{k}: {x} days; at most ten years either way"))); }
                cfg.jd0 += x;
                let mut e = time::jd2utc(cfg.jd0);
                e[5] = e[5].round();
                cfg.epoch_utc = e;
                cfg.jd0 = time::jd(&e);
                cfg.params.jd0 = cfg.jd0;
            }
            // the truth orbit's local time of the ascending node (the beta angle) and altitude; the
            // flight software keeps the gains it was tuned with for the nominal orbit
            "engine.ltan_h" => { if !(0.0..24.0).contains(&x) { return Err(Error::refused(format!("{k}: {x} h, from 0 to 24"))); } cfg.ltan_h = x; }
            "engine.alt_km" => {
                if !(150.0..=2000.0).contains(&x) { return Err(Error::refused(format!("{k}: {x} km, from 150 to 2000 (low Earth orbit)"))); }
                cfg.alt_km = x;
                let a = 6378137.0 + 1e3*x;
                cfg.period_s = 2.0*std::f64::consts::PI*(a*a*a/cfg.mu).sqrt();
            }
            _ => return Err(Error::refused(format!("unknown engine override {k}: the engine reads {}", ENGINE_KEYS.join(", ")))),
        }
    }
    Ok(())
}

impl Config {
    /// Every value a run depends on is inside the range the engine models, or the run is
    /// refused with the value and the range: never clamped, never run on NaN.
    pub fn check(&self) -> Result<(), Error> {
        let c = &self.case;
        let miss: Vec<&str> = CASE_NEEDS.iter().copied().filter(|k| !c.get(k).is_finite()).collect();
        if !miss.is_empty() { return Err(Error::refused(format!("case {} does not state {}, which every run needs", c.id, miss.join(", ")))); }
        let within = |what: &str, x: f64, lo: f64, hi: f64| -> Result<(), Error> {
            if x.is_finite() && x >= lo && x <= hi { Ok(()) } else { Err(Error::refused(format!("{what} = {x}: the engine models {lo} to {hi}"))) }
        };
        within("orbit.alt (km)", c.get("orbit.alt"), ALT_KM.0, ALT_KM.1)?;
        within("orbit.inc (deg)", c.get("orbit.inc"), 0.0, 180.0)?;
        within("orbit.ecc", c.get("orbit.ecc"), 0.0, 0.1)?;
        within("orbit.ltan (h)", c.get("orbit.ltan"), 0.0, 24.0)?;
        // the flexible mode: all of it or none of it
        let fk = ["flex.fmode", "flex.mpart", "flex.zeta", "flex.axis"];
        let fs: Vec<&str> = fk.iter().copied().filter(|k| c.get(k).is_finite()).collect();
        if !fs.is_empty() && fs.len() < fk.len() {
            let missing: Vec<&str> = fk.iter().copied().filter(|k| !fs.contains(k)).collect();
            return Err(Error::refused(format!("case {} states part of its flexible mode: {} missing (all of it or none)", c.id, missing.join(", "))));
        }
        if !fs.is_empty() {
            within("flex.fmode (Hz)", c.get("flex.fmode"), 1e-3, 100.0)?;
            within("flex.mpart", c.get("flex.mpart"), 0.0, 0.95)?;
            within("flex.zeta", c.get("flex.zeta"), 0.0, 1.0)?;
            if ![1.0, 2.0, 3.0].contains(&c.get("flex.axis")) { return Err(Error::refused(format!("flex.axis = {}: 1, 2 or 3 (X_B, Y_B, Z_B)", c.get("flex.axis")))); }
        }
        // the power system: all of it or none of it, each value in its range
        let stated: Vec<&str> = crate::metrics::POWER_KEYS.iter().copied().filter(|k| c.get(k).is_finite()).collect();
        if !stated.is_empty() && stated.len() < crate::metrics::POWER_KEYS.len() {
            let missing: Vec<&str> = crate::metrics::POWER_KEYS.iter().copied().filter(|k| !stated.contains(k)).collect();
            return Err(Error::refused(format!("case {} states part of its power system: {} missing (all of it or none)", c.id, missing.join(", "))));
        }
        if !stated.is_empty() {
            for k in &crate::metrics::POWER_KEYS[..6] { within(&format!("{k} (m^2)"), c.get(k), 0.0, 10.0)?; }
            within("power.eff", c.get("power.eff"), 0.0, 1.0)?;
            within("power.batt_wh (Wh)", c.get("power.batt_wh"), 1e-3, 1e5)?;
            within("power.load_w (W)", c.get("power.load_w"), 0.0, 1e4)?;
            within("power.soc0", c.get("power.soc0"), 0.0, 1.0)?;
        }
        // a stated value nothing models is refused, never silently dropped
        for (k, why) in CASE_UNMODELLED {
            if c.get(k).is_finite() { return Err(Error::refused(format!("case {} states {k} = {}: {why}", c.id, c.get(k)))); }
        }
        // the facet model has one centre-of-mass offset (length surface.cpa) for the aerodynamic and the
        // solar torque, and computes the sunlit area from the box: a stated value it cannot honour is refused
        let (cpa, cps) = (c.get("surface.cpa"), c.get("surface.cps"));
        if cps.is_finite() && cps != cpa {
            return Err(Error::refused(format!("case {}: surface.cps = {cps} m differs from surface.cpa = {cpa} m; the facet model has one centre-of-mass offset for both torques", c.id)));
        }
        let asun = c.get("surface.asun");
        let face = [self.box_m[0]*self.box_m[1], self.box_m[1]*self.box_m[2], self.box_m[0]*self.box_m[2]].into_iter().fold(0.0, f64::max);
        if asun.is_finite() && (asun - face).abs() > 1e-6*face.max(1e-12) {
            return Err(Error::refused(format!("case {}: surface.asun = {asun} m^2, but the facet model lights the {} body, largest face {face} m^2 (no deployables are modelled)", c.id, c.class)));
        }
        if !(self.mass_kg > 0.0) { return Err(Error::refused(format!("mass = {} kg: a mass is positive", self.mass_kg))); }
        for (i, row) in self.inertia.iter().enumerate() {
            if !(row[i] > 0.0) { return Err(Error::refused(format!("inertia axis {} = {} kg m^2: a principal inertia is positive", i + 1, row[i]))); }
        }
        if !(self.duration_s > 0.0 && self.duration_s <= DURATION_MAX_S) {
            return Err(Error::refused(format!("duration {} s: a run lasts more than 0 s and at most {} s (30 days)", self.duration_s, DURATION_MAX_S)));
        }
        if !(self.dt > 0.0 && self.dt <= 10.0 && self.dt <= self.duration_s) {
            return Err(Error::refused(format!("time.dt_s = {} s: the control step is more than 0, at most 10 s and at most the duration", self.dt)));
        }
        if !(self.record_dt >= self.dt && self.record_dt <= self.duration_s.max(self.dt)) {
            return Err(Error::refused(format!("time.record_dt_s = {} s: the record step is at least the control step ({} s) and at most the duration", self.record_dt, self.dt)));
        }
        within("engine.orbit_step_s", self.orbit_step_s, 1e-3, 600.0)?;
        within("engine.density_scale", self.density_scale, 0.0, 100.0)?;
        within("engine.f107", self.f107, 0.0, 400.0)?;
        within("engine.f107a", self.f107a, 0.0, 400.0)?;
        within("engine.kp", self.kp, 0.0, 9.0)?;
        within("engine.ap", self.ap, 0.0, 400.0)?;
        within("engine.accommodation", self.sigma_n, 0.0, 1.0)?;
        within("engine.vb_ratio", self.vb_ratio, 0.0, 1.0)?;
        within("engine.spec_frac", self.spec_frac, 0.0, 1.0)?;
        within("engine.refl", self.refl, 0.0, 2.0)?;
        // the flight software checks its parameters at init; the engine says which one first, by name
        self.params.validate().map_err(|f| Error::refused(format!(
            "flight-software parameter {f} is outside its rule in fsw/params/params.toml (the flight software would refuse this configuration)")))?;
        Ok(())
    }

    /// scenario: a data/scenarios id or a path; overrides: dotted paths into the scenario ("fsw.rw_bandwidth=0.5").
    pub fn build(root: &Path, scenario: &str, case_file: &Path, seed: u64, overrides: &[(String, String)]) -> Result<Config, Error> {
        if !scenario.ends_with(".json") { check_id("scenario", scenario)?; }
        let sp = if scenario.ends_with(".json") { scenario.into() } else { root.join("data/scenarios").join(format!("{scenario}.json")) };
        if !sp.is_file() && !scenario.ends_with(".json") {
            return Err(Error::refused(format!("no scenario {scenario}: {} does not exist (the scenarios are data/scenarios/*.json)", sp.display())));
        }
        let mut s = json::read(&sp)?;
        let mut eng: Vec<(String, String)> = vec![];
        for (k, v) in overrides {
            if k.starts_with("engine.") { eng.push((k.clone(), v.clone())); } else { set_override(&mut s, k, v)?; }
        }
        let cf = if case_file.is_file() { case_file.to_path_buf() } else { root.join(case_file) };
        let c = Case::read(&cf)?;
        crate::schema::check_scenario(&s, &c)?;
        let dev = Dev::load(root, json::s(&s, "product", ""))?;
        let fsw = s.get("fsw").cloned().unwrap_or(Value::Null);
        let init = s.get("initial").cloned().unwrap_or(Value::Null);
        let tm = s.get("time").cloned().unwrap_or(Value::Null);
        let v = |k: &str| c.get(k);

        let mut epoch = time::jd2utc(2451545.0 + v("mission.epoch")*365.25);
        epoch[5] = epoch[5].round();
        let jd0 = time::jd(&epoch);
        let mu = 3.986004418e14;
        let a = 6378137.0 + v("orbit.alt")*1e3;
        let n = (mu/(a*a*a)).sqrt();
        let inertia = [[v("mass.imin"), 0.0, 0.0], [0.0, v("mass.iint"), 0.0], [0.0, 0.0, v("mass.imax")]];
        let cpa = v("surface.cpa");
        let cmd = [0.30, 0.70, -0.65];
        let cmn = (cmd[0]*cmd[0] + cmd[1]*cmd[1] + cmd[2]*cmd[2] as f64).sqrt();
        let m_res = [v("magnetic.dres")/3f64.sqrt(); 3];
        let box_m = class_box(root, &c)?;

        let alg = select(root, &dev, &s)?;
        let k = Knowns { dev: &dev, fsw: &fsw, inertia, m_res, n, inc_deg: v("orbit.inc") };
        let mut p = Params::default();
        let dt = json::f(&tm, "dt_s", 0.1);
        p.jd0 = jd0; p.dt = dt; p.mu = mu;
        modes_and_laws(&mut p, &fsw, &alg)?;
        guidance_params(&mut p, &k);
        mtq_gains(&mut p, &k);
        rw_gains(&mut p, &k);
        spin_params(&mut p, &k);
        let h_t_rot = rotor_params(&mut p, &k);
        rcs_params(&mut p, &k);
        sensor_params(&mut p, &k);
        let faults = faults(&s, &dev)?;
        let gd_kind0 = GUID[p.start_mode as usize];
        let mut cfg = Config {
            id: json::s(&s, "id", scenario).into(), case: c.clone(), dev, seed, epoch_utc: epoch, jd0,
            alt_km: v("orbit.alt"), inc_deg: v("orbit.inc"), ecc: v("orbit.ecc"), ltan_h: v("orbit.ltan"), u0_deg: json::f(&init, "arg_lat_deg", 0.0),
            orbit_step_s: 10.0, period_s: 2.0*PI/n, mu, zonal_max: 6, third_body: true, drag: true, srp: true, density_scale: 1.0,
            orbit_model: "pop".into(), f107: 130.0, f107a: 130.0, kp: 2.0, ap: 7.0,
            igrf_nmax: 13, env_dt_s: 1.0, env_on: [true; 4],
            mass_kg: v("mass.m"), inertia, box_m, cm_offset_m: [cpa*cmd[0]/cmn, cpa*cmd[1]/cmn, cpa*cmd[2]/cmn],
            aref_m2: v("surface.afr"), cd: v("surface.cd"), refl: v("surface.refl"), sigma_n: ACCOMMODATION, sigma_t: ACCOMMODATION, vb_ratio: VB_RATIO, spec_frac: SPEC_FRAC, m_res,
            duration_s: json::f(&tm, "duration_s", 600.0), dt, record_dt: json::f(&tm, "record_dt_s", 1.0),
            params: p, alg, faults, gd_kind0, h_t_rot, spin_dps: json::f(&fsw, "spin_rate_dps", 6.0), flex: None, scenario: s,
            scenario_file: sp.display().to_string(), overrides: overrides.to_vec(),
        };
        apply_engine(&mut cfg, &eng)?;
        cfg.check()?;
        // the flexible mode on the truth body: |delta|^2 is mpart of the coupled axis's inertia
        if cfg.case.get("flex.fmode").is_finite() {
            let a = cfg.case.get("flex.axis") as usize - 1;
            let mut delta = [0.0; 3];
            delta[a] = (cfg.case.get("flex.mpart")*cfg.inertia[a][a]).sqrt();
            cfg.flex = Some(adcs_sim_core::plant::Flex { on: true, delta, omega: 2.0*PI*cfg.case.get("flex.fmode"), zeta: cfg.case.get("flex.zeta") });
        }
        Ok(cfg)
    }

    /// The adcs-fswcfg/1 blob the flight software boots from.
    pub fn blob(&self) -> Vec<u8> {
        let mut b = vec![0u8; adcs_fsw::params::BLOB_SIZE];
        self.params.encode(&mut b);
        b
    }
}

