//! Case + scenario + product -> everything a run needs (asils.config + asils.fsw.init/select):
//! the plant's truth parameters and the flight software's adcs-fswcfg/1 parameters.
use crate::error::Error;
use crate::case::Case;
use crate::json::{self, get};
use crate::gen::{fswbody as fb, fswchoice as fc, fswmtq as fm, fswrcs as frc, fswrotor as fro, fswrw as fr, fswsens as fse, fswspin as fs};
use crate::lqr;
use crate::product::Dev;
use crate::stated::Stated;
use adcs_fsw::params::{Params, MODE_NONE};
use adcs_sim_core::NR;
use serde_json::Value;
use std::collections::BTreeMap;
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
    /// how a run starts where its scenario's initial section states nothing (dyn's stated values)
    pub start: Start,
    /// the scenario file this run was built from, and the overrides given with it
    pub scenario_file: String, pub overrides: Vec<(String, String)>,
}

/// How a run starts where its scenario's initial section states nothing: dyn's stated values (dyn_initial_*), read by
/// run.rs's initial_state: the attitude's and the rate's kinds (dyn_initial_state's AttStart and RateStart), the error's
/// axis (body) and angle [deg], the body rate [deg/s], a random direction's rate [deg/s] and the rate added [deg/s].
#[derive(Clone, Debug, Default)]
pub struct Start { pub att_kind: i64, pub axis: [f64; 3], pub angle_deg: f64, pub rate_kind: i64, pub rate_deg_s: [f64; 3], pub magnitude_deg_s: f64, pub extra_deg_s: f64 }

impl Start {
    fn load(st: &Stated) -> Result<Start, Error> {
        Ok(Start { att_kind: st.whole("dyn_initial_attitude_kind_default", 0, 3)? as i64, axis: st.list("dyn_initial_error_axis")?,
                   angle_deg: st.get("dyn_initial_error_angle")?, rate_kind: st.whole("dyn_initial_rate_kind_default", 0, 3)? as i64,
                   rate_deg_s: st.list("dyn_initial_rate_value")?, magnitude_deg_s: st.get("dyn_initial_rate_magnitude")?,
                   extra_deg_s: st.get("dyn_initial_rate_extra")? })
    }
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

/// The option of slot `slot` that algorithm `id` is: its place in FLOWN (fswchoice's choices list the slot's algorithms in
/// this order, then none); none (the slot's length) for "" or an id the engine does not fly in it.
fn option(slot: &str, id: &str) -> i64 {
    let ids = FLOWN.iter().find(|(s, _)| *s == slot).map(|x| x.1).unwrap_or(&[]);
    ids.iter().position(|x| *x == id).unwrap_or(ids.len()) as i64
}

/// A slice as an array of its length (a generated function's input).
fn arr<T: Copy + Default, const N: usize>(x: &[T]) -> [T; N] { let mut a = [T::default(); N]; a.copy_from_slice(&x[..N]); a }

fn select(root: &Path, dev: &Dev, s: &Value) -> Result<BTreeMap<String, String>, Error> {
    let has = dev.caps();
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
    // whether the product can fly each algorithm of a slot (the registry's needs, its devices), in FLOWN's order
    let can = |ids: &[&str]| -> Result<Vec<i64>, Error> {
        ids.iter().map(|c| load(c).map(|(_, needs)| needs.iter().all(|n| has.contains(&n.as_str())) as i64)).collect()
    };
    let mut alg = BTreeMap::new();
    for (sl, ids) in FLOWN {
        if let Some(id) = pick.get(sl) {
            let (slot, needs) = load(id)?;
            let miss: Vec<_> = needs.iter().filter(|n| !has.contains(&n.as_str())).cloned().collect();
            if !miss.is_empty() { return Err(Error::refused(format!("algorithm {id} ({sl}) cannot fly on {}: it needs {}", dev.id, miss.join(", ")))); }
            if slot != sl { return Err(Error::refused(format!("algorithm {id} does {slot}, not {sl}"))); }
            alg.insert(sl.to_string(), id.clone());
        } else {
            // the slot's default, fswchoice's: the first of its defaults the product can fly, else none ("")
            let c = can(ids)?;
            let o = match sl {
                "detumble" => fc::fsw_default_detumble(arr(&c)),
                "attitude" => fc::fsw_default_attitude(arr(&c)),
                "pointing" => fc::fsw_default_pointing(arr(&c)),
                "mtq_pointing" => fc::fsw_default_mtq_pointing(arr(&c)),
                "sun_acquisition" => fc::fsw_default_sun_acquisition(arr(&c)),
                "allocation" => fc::fsw_default_allocation(arr(&c)),
                "thrusters" => fc::fsw_default_thrusters(arr(&c)),
                _ => ids.len() as i64,
            } as usize;
            alg.insert(sl.to_string(), ids.get(o).copied().unwrap_or("").to_string());
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
/// incoming speed, and the specular share of the reflected light, dyn's stated values (the design's
/// data/stated.json, S7.11): dyn_surface_accommodation, dyn_surface_vb_ratio, dyn_surface_specular_share.
pub const SURFACE: [&str; 3] = ["dyn_surface_accommodation", "dyn_surface_vb_ratio", "dyn_surface_specular_share"];

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

/// What the flight-software parameter stages read from the case, the product, the scenario and the design's stated values.
struct Knowns<'a> { dev: &'a Dev, fsw: &'a Value, st: &'a Stated, inertia: [[f64; 3]; 3], m_res: [f64; 3], n: f64, inc_deg: f64 }

impl Knowns<'_> {
    /// The principal inertia [kg m^2] on each axis.
    fn ii(&self) -> [f64; 3] { [self.inertia[0][0], self.inertia[1][1], self.inertia[2][2]] }
    /// A flight parameter the scenario's fsw section gives (key), else the value fsw_param_<name> states.
    fn param(&self, name: &str, key: &str) -> Result<f64, Error> { Ok(json::f(self.fsw, key, self.st.get(&format!("fsw_param_{name}"))?)) }
    /// A yes or no parameter: the scenario's fsw section (key), else what fsw_param_<name> states.
    fn param_flag(&self, name: &str, key: &str) -> Result<u8, Error> { Ok(json::b(self.fsw, key, self.st.flag(&format!("fsw_param_{name}"))?) as u8) }
    /// A law's tuning the scenario's fsw section gives (key), else the value fsw_tune_<key> states.
    fn tune(&self, key: &str) -> Result<f64, Error> { Ok(json::f(self.fsw, key, self.st.get(&format!("fsw_tune_{key}"))?)) }
    /// A flight parameter the design states (fsw_param_<name>): a constant of the flight software.
    fn constant(&self, name: &str) -> Result<f64, Error> { self.st.get(&format!("fsw_param_{name}")) }
}

/// The start mode, the next mode and the schedule; the law each algorithm slot flies (fswchoice).
fn modes_and_laws(p: &mut Params, k: &Knowns, alg: &BTreeMap<String, String>) -> Result<(), Error> {
    let fsw = k.fsw;
    let o = |slot: &str| option(slot, alg.get(slot).map(String::as_str).unwrap_or(""));
    p.start_mode = match get(fsw, "start_mode").and_then(Value::as_str) { Some(m) => mode_index(m)?, None => k.st.whole("fsw_param_start_mode", 0, 255)? as u8 };
    p.auto_next = match get(fsw, "auto_next").and_then(Value::as_str) {
        Some("") => MODE_NONE, Some(m) => mode_index(m)?, None => k.st.whole("fsw_param_auto_next", 0, 255)? as u8 };
    if let Some(sc) = get(fsw, "schedule") {
        let list: Vec<Value> = match sc { Value::Array(x) => x.clone(), x => vec![x.clone()] };
        for (i, e) in list.iter().take(p.sched_t.len()).enumerate() { p.sched_t[i] = json::f(e, "t_s", 0.0); p.sched_mode[i] = mode_index(json::s(e, "mode", ""))?; }
        p.n_sched = list.len().min(p.sched_t.len()) as u8;
    }
    p.bdot_law = fc::fsw_bdot_law(o("detumble")) as u8;
    p.rw_law = fc::fsw_rw_law(o("pointing")) as u8;
    p.mtq_law = fc::fsw_mtq_law(o("mtq_pointing")) as u8;
    p.alloc = fc::fsw_alloc(o("allocation")) as u8;
    let (ecl, rzf, sl) = fc::fsw_sun_acquisition(o("sun_acquisition"));
    p.ss_eclipse = ecl as u8; p.ss_rz_floor = rzf; p.ss_law = sl as u8;
    Ok(())
}

/// The guidance the start mode flies, the payload and power axes, the body's inertia and residual dipole (fswbody).
fn guidance_params(p: &mut Params, k: &Knowns) -> Result<(), Error> {
    let dev = k.dev;
    let g = get(k.fsw, "guidance").cloned().unwrap_or(Value::Null);
    // the scenario's word for its guidance, fswchoice's GuidKind; when it states none, fsw_param_gd_kind
    let kind = match get(&g, "kind").and_then(Value::as_str) {
        Some("nadir") => fc::GUIDKIND_NADIR, Some("target") => fc::GUIDKIND_TARGET, Some("slew") => fc::GUIDKIND_SLEW,
        Some("inertial") => fc::GUIDKIND_INERTIAL, Some("sun") => fc::GUIDKIND_SUN, _ => k.st.whole("fsw_param_gd_kind", 0, 4)? as i64 };
    p.gd_kind = fc::fsw_guidance_kind(kind) as u8;
    p.gd_q_off = fb::fsw_payload_offset(dev.boresight);
    let gd = |name: &str, key: &str| -> Result<f64, Error> { Ok(json::f(&g, key, k.st.get(&format!("fsw_param_{name}"))?)) };
    p.gd_roll_deg = gd("gd_roll_deg", "roll_deg")?; p.gd_t0 = gd("gd_t0", "t0")?; p.gd_T = gd("gd_T", "T_s")?;
    p.gd_axis = match get(&g, "axis").and_then(json::v3) { Some(a) => a, None => k.st.list("fsw_param_gd_axis")? };
    let q0: [f64; 4] = k.st.list("fsw_param_gd_q_inertial")?;
    p.gd_q_inertial = get(&g, "q_inertial").and_then(|q| q.as_array().map(|a| [a[0].as_f64().unwrap_or(q0[0]), a[1].as_f64().unwrap_or(q0[1]), a[2].as_f64().unwrap_or(q0[2]), a[3].as_f64().unwrap_or(q0[3])])).unwrap_or(q0);
    (p.J, p.m_res_est, p.sun_axis, p.roll_axis) = fb::fsw_body_model(k.inertia, k.m_res, dev.sun_axis, dev.boresight);
    Ok(())
}

/// The magnetic gains: one bandwidth for every law, and the literature laws' gains on it (fswmtq); the LQR's gain the
/// toolbox's Riccati solve (lqr.rs) of the weights fswmtq gives.
fn mtq_gains(p: &mut Params, k: &Knowns) -> Result<(), Error> {
    let (dev, ii, n) = (k.dev, k.ii(), k.n);
    p.mtq_period = k.constant("mtq_period")?; p.mtq_meas = k.constant("mtq_meas")?;
    p.m_max = fm::fsw_coil_limit(dev.mtq.fitted, dev.mtq.m_max);
    p.bdot_k = fm::fsw_bdot_gain(k.tune("bdot_gain_scale")?, n, k.inc_deg, ii);
    p.detumble_exit = fm::fsw_rate_rad(k.tune("detumble_exit_deg_s")?);
    p.detumble_hold_s = k.param("detumble_hold_s", "detumble_hold_s")?;
    let (wn, z) = (k.tune("mtq_wn")?, k.tune("mtq_zeta")?);
    (p.mtq_Kp, p.mtq_Kd, p.mtq_Ki) = fm::fsw_mtq_pd(ii, wn, z);
    for ax in 0..3 {
        let (b, q, r, p0) = fm::fsw_mtq_lqr_weights(ii[ax], wn);
        p.mtq_Klqr[ax] = lqr::chain3(b, q, r, p0);
    }
    p.mtq_err_max = k.constant("mtq_err_max")?; p.mtq_int_max = k.constant("mtq_int_max")?;
    let (gp, gd) = (k.tune("mtq_gain_p")?, k.tune("mtq_gain_d")?);
    p.mtq_eps = k.constant("mtq_eps")?;
    (p.mtq_k1, p.mtq_k2, p.mtq_k16, p.mtq_lam16, p.sb_kp, p.sb_kd) =
        fm::fsw_mtq_literature(p.mtq_law as i64, ii, wn, z, p.mtq_eps, gp, gd, k.tune("avanzini_k_over_n")?, k.tune("avanzini_lambda")?, n);
    (p.sb_kroll, p.sb_kdroll) = fm::fsw_mtq_roll(dev.boresight, k.inertia, n, k.tune("roll_wn_orbits")?, k.tune("roll_zeta")?, k.tune("roll_gain")?);
    p.sb_roll_gate = fm::fsw_roll_gate(k.tune("roll_gate_deg")?);
    p.ho_in_dps = k.param("ho_in_dps", "handover_in_dps")?; p.ho_out_dps = k.param("ho_out_dps", "handover_out_dps")?;
    p.ho_hold_s = k.param("ho_hold_s", "handover_hold_s")?;
    // the gravity-gradient feed-forward: the scenario's, else fswbody's from the nadir attitude's inertia
    p.mtq_gg_ff = json::f(k.fsw, "mtq_gg_ff", fb::fsw_gg_feedforward(p.gd_q_off, k.inertia) as f64) as u8;
    (p.mtq_Pth, p.mtq_Pw) = fm::fsw_mtq_tango(ii, wn, z, gp, gd);
    p.mtq_phi = k.constant("mtq_phi")?;
    (p.mtq_lambda, p.mtq_Gs) = fm::fsw_mtq_smc(wn, z, p.mtq_phi);
    Ok(())
}

/// The fine-pointing gains, wheels and momentum devices (fswrw); the LQR's gain the toolbox's Riccati solve (lqr.rs).
fn rw_gains(p: &mut Params, k: &Knowns) -> Result<(), Error> {
    let ii = k.ii();
    let (wn, z) = (k.tune("rw_bandwidth")?, k.tune("rw_damping")?);
    (p.rw_Kp, p.rw_Kd, p.rw_Ki) = fr::fsw_rw_pid(ii, wn, z);
    for ax in 0..3 {
        let (b, q, r, p0) = fr::fsw_rw_lqr_weights(ii[ax], wn);
        p.rw_Klqr[ax] = lqr::chain3(b, q, r, p0);
    }
    p.rw_err_max = k.constant("rw_err_max")?; p.rw_int_max = k.constant("rw_int_max")?;
    p.rw_dt = fr::fsw_rw_period(k.tune("rw_rate_hz")?);
    p.rw_phi = k.constant("rw_phi")?;
    (p.rw_lambda, p.rw_Gs) = fr::fsw_rw_smc(wn, z, p.rw_phi);
    p.capture_deg = k.param("capture_deg", "capture_deg")?; p.capture_rate_deg_s = k.param("capture_rate_deg_s", "capture_rate_deg_s")?;
    Ok(())
}

/// The Sun-spin and Sun-acquisition laws (fswspin).
fn spin_params(p: &mut Params, k: &Knowns) -> Result<(), Error> {
    p.ss_k_l1 = k.param("ss_k_l1", "l1_gain")?; p.ss_spin_dps = k.param("ss_spin_dps", "spin_rate_dps")?;
    for (x, name) in [(&mut p.ss_sigma0, "ss_sigma0"), (&mut p.ss_z_in_dps, "ss_z_in_dps"), (&mut p.ss_perp_in_dps, "ss_perp_in_dps"),
                      (&mut p.ss_sun_min, "ss_sun_min"), (&mut p.ss_t_check_s, "ss_t_check_s"), (&mut p.ss_omega_max_dps, "ss_omega_max_dps"),
                      (&mut p.ss_dwell_in_s, "ss_dwell_in_s"), (&mut p.ss_dr_k1, "ss_dr_k1"), (&mut p.ss_omega_exit_dps, "ss_omega_exit_dps"),
                      (&mut p.sa_kd, "sa_kd"), (&mut p.sa_done_deg, "sa_done_deg"), (&mut p.sa_done_hold_s, "sa_done_hold_s")] {
        *x = k.constant(name)?;
    }
    (p.ss_k1, p.ss_k2, p.ss_dr_k, p.ss_dr_k2) = fs::fsw_sun_spin_gains(k.tune("ss_gain")?, k.inertia[2][2]);
    p.ss_perp_out_dps = k.param("ss_perp_out_dps", "sun_spin_perp_out_dps")?; p.ss_dwell_out_s = k.param("ss_dwell_out_s", "sun_spin_dwell_out_s")?;
    p.sa_w_max_deg_s = k.param("sa_w_max_deg_s", "sun_acq_rate_deg_s")?;
    Ok(())
}

/// The momentum devices, in their NOMINAL geometry (fswrotor); returns each rotor's momentum when the scenario states none.
fn rotor_params(p: &mut Params, k: &Knowns) -> Result<[f64; NR], Error> {
    let x = &k.dev.mex;
    let kind = x.kind.map(|c| c.choice());
    let gi = x.gi.map(|g| g as i64);
    let (nr, ng, rk, a0, rgi, ga, tmax, hmax, h0, grate) =
        fro::fsw_rotors(x.n as i64, x.ng as i64, kind, x.a0, gi, x.g, x.torque_max, x.h_max, x.h0, x.gimbal_rate_max);
    p.nr = nr as u8; p.ng = ng as u8;
    p.rot_kind = rk.map(|c| c as u8); p.rot_a0 = a0; p.rot_gi = rgi.map(|g| g as u8); p.gim_axis = ga;
    p.rot_tmax = tmax; p.rot_hmax = hmax; p.rot_h0 = h0; p.gim_rate_max = grate;
    p.h_bias = k.param("h_bias", "wheel_bias_Nms")?; p.dump_k = k.param("dump_k", "dump_gain")?;
    for (v, name) in [(&mut p.cmg_lam0, "cmg_lam0"), (&mut p.cmg_mu, "cmg_mu"), (&mut p.cmg_k_null, "cmg_k_null"), (&mut p.fdir_s, "fdir_s"),
                      (&mut p.fdir_win_s, "fdir_win_s"), (&mut p.fdir_h_frac, "fdir_h_frac")] {
        *v = k.constant(name)?;
    }
    Ok(fro::fsw_rotor_targets(x.n as i64, kind, x.h_max, x.h0, p.h_bias))
}

/// The thrusters (fswrcs), and the assist, dump and detumble settings.
fn rcs_params(p: &mut Params, k: &Knowns) -> Result<(), Error> {
    let r = &k.dev.rcs;
    let (nc, tau, mib, res) = frc::fsw_thrusters(r.fitted, r.nc as i64, r.tau, r.mib, r.res);
    p.nc = nc as u8; p.rcs_tau = tau; p.rcs_mib = mib; p.rcs_res = res;
    p.rcs_assist = k.param_flag("rcs_assist", "rcs_assist")?; p.rcs_assist_frac = k.constant("rcs_assist_frac")?;
    p.rcs_dump = k.param_flag("rcs_dump", "rcs_dump")?;
    p.rcs_dump_hi = k.constant("rcs_dump_hi")?; p.rcs_dump_lo = k.constant("rcs_dump_lo")?; p.rcs_dump_k = k.constant("rcs_dump_k")?;
    p.rcsd_T_damp_s = k.param("rcsd_T_damp_s", "rcs_damp_s")?;
    p.rcsd_deadband_deg_s = k.constant("rcsd_deadband_deg_s")?; p.rcsd_period_s = k.constant("rcsd_period_s")?;
    Ok(())
}

/// The sensors the software is told about, and the estimator's measurement sigmas (fswsens).
fn sensor_params(p: &mut Params, k: &Knowns) -> Result<(), Error> {
    let d = k.dev;
    let (hg, hs, hn, he, hp, nh) = fse::fsw_sensor_set(d.gyro.fitted, d.st.fitted, d.sun.fitted, d.css.fitted, d.es.fitted, d.gps.fitted, d.st.nh as i64);
    p.has_gyro = hg as u8; p.has_st = hs as u8; p.has_sun = hn as u8; p.has_es = he as u8; p.has_gps = hp as u8; p.n_heads = nh as u8;
    (p.st_bs, p.st_noise_cross, p.st_noise_roll, p.st_latency) = fse::fsw_star_tracker(d.st.nh as i64, d.st.bs, d.st.noise_cross, d.st.noise_roll, d.st.latency);
    p.st_coast_s = k.constant("st_coast_s")?;
    (p.gyro_arw, p.gyro_rrw, p.es_noise) = fse::fsw_rate_noises(d.gyro.arw, d.gyro.rrw, d.es.noise);
    (p.mekf_sig_sun, p.mekf_sig_mag, p.mekf_mag_err_T) = fse::fsw_mekf_sigmas(d.sun.fitted, d.sun.noise, d.sun.bias_sigma, d.css.albedo,
        d.mag.misalign, d.mag.sf_sigma, d.mag.bias_t, d.mag.bias_sigma, d.mag.noise);
    p.mekf_gate = k.param("mekf_gate", "mekf_gate")?; p.mekf_rej_max = k.param("mekf_rej_max", "mekf_rej_max")?;
    p.mekf_meas_scale = k.constant("mekf_meas_scale")?;
    p.gnss_ecef = k.st.whole("fsw_param_gnss_ecef", 0, 1)? as u8;
    p.gps_latency = fse::fsw_gnss_latency(d.gps.fitted, d.gps.latency);
    p.gd_yaw_flip = k.param_flag("gd_yaw_flip", "yaw_flip")?; p.gd_flip_hyst = k.constant("gd_flip_hyst")?;
    p.rate_lpf_s = k.param("rate_lpf_s", "rate_lpf_s")?; p.igrf_nmax = k.st.whole("fsw_param_igrf_nmax", 1, 13)? as u8;
    Ok(())
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
                "engine.inertia_scale" => cfg.inertia = crate::gen::truthplant::scale_inertia(cfg.inertia, [a[0], a[1], a[2]]),
                // products of inertia, each a fraction of sqrt(I_ii I_jj): [xy, xz, yz]; the body stays a body
                "engine.inertia_products" => {
                    if let Some(f) = a.iter().find(|f| f.abs() >= 1.0) {
                        return Err(Error::refused(format!("{k}: each product is a fraction of sqrt(I_ii I_jj) below 1, not {f}")));
                    }
                    // dyn_truth_plant: the products, and whether the body is still a body
                    let (m, pd) = crate::gen::truthplant::inertia_products(cfg.inertia, [a[0], a[1], a[2]]);
                    if !pd { return Err(Error::refused(format!("{k}: {a:?} gives an inertia that is not positive definite"))); }
                    cfg.inertia = m;
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
                // env_case_orbit: the epoch moved, its seconds rounded
                (cfg.epoch_utc, cfg.jd0) = crate::gen::caseorbit::shifted_epoch(cfg.jd0, x);
                cfg.params.jd0 = cfg.jd0;
            }
            // the truth orbit's local time of the ascending node (the beta angle) and altitude; the
            // flight software keeps the gains it was tuned with for the nominal orbit
            "engine.ltan_h" => { if !(0.0..24.0).contains(&x) { return Err(Error::refused(format!("{k}: {x} h, from 0 to 24"))); } cfg.ltan_h = x; }
            "engine.alt_km" => {
                if !(150.0..=2000.0).contains(&x) { return Err(Error::refused(format!("{k}: {x} km, from 150 to 2000 (low Earth orbit)"))); }
                cfg.alt_km = x;
                cfg.period_s = crate::gen::caseorbit::dispersed_period(1e3*x);   // env_case_orbit
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
        if !crate::source::is_file(&sp) && !scenario.ends_with(".json") {
            return Err(Error::refused(format!("no scenario {scenario}: {} does not exist (the scenarios are data/scenarios/*.json)", sp.display())));
        }
        let mut s = json::read(&sp)?;
        let mut eng: Vec<(String, String)> = vec![];
        for (k, v) in overrides {
            if k.starts_with("engine.") { eng.push((k.clone(), v.clone())); } else { set_override(&mut s, k, v)?; }
        }
        let cf = if crate::source::is_file(case_file) { case_file.to_path_buf() } else { root.join(case_file) };
        let c = Case::read(&cf)?;
        crate::schema::check_scenario(&s, &c)?;
        let dev = Dev::load(root, json::s(&s, "product", ""))?;
        let fsw = s.get("fsw").cloned().unwrap_or(Value::Null);
        let init = s.get("initial").cloned().unwrap_or(Value::Null);
        let tm = s.get("time").cloned().unwrap_or(Value::Null);
        let v = |k: &str| c.get(k);

        // the values the case and the scenario leave to the design: dyn's, env's, vv's, fsw's (data/stated.json)
        let st = Stated::load(root)?;
        // env_case_orbit: the mission's epoch (its seconds rounded), the orbit's mean motion and period
        let (epoch, jd0) = crate::gen::caseorbit::mission_epoch(v("mission.epoch"));
        let mu = crate::gen::constants::MU_E;
        let (n, period_s) = crate::gen::caseorbit::case_mean_motion(v("orbit.alt")*1e3);
        // dyn_truth_plant and s1_4: the inertia, the residual dipole on each axis, the centre of mass's offset
        let inertia = crate::gen::truthplant::principal_inertia(v("mass.imin"), v("mass.iint"), v("mass.imax"));
        let cm_offset_m = crate::gen::cmoffset::cm_offset(v("surface.cpa"), st.v3("dyn_cm_direction")?);
        let m_res = crate::gen::truthplant::residual_dipole_axes(v("magnetic.dres"));
        let surf = [st.get(SURFACE[0])?, st.get(SURFACE[1])?, st.get(SURFACE[2])?];
        let box_m = class_box(root, &c)?;

        let alg = select(root, &dev, &s)?;
        let k = Knowns { dev: &dev, fsw: &fsw, st: &st, inertia, m_res, n, inc_deg: v("orbit.inc") };
        let mut p = Params::default();
        // the flight software's tick: the scenario's control step, else fsw_param_dt; its epoch and Earth (fswbody)
        let dt = json::f(&tm, "dt_s", st.get("fsw_param_dt")?);
        (p.jd0, p.mu) = fb::fsw_epoch_earth(jd0); p.dt = dt;
        modes_and_laws(&mut p, &k, &alg)?;
        guidance_params(&mut p, &k)?;
        mtq_gains(&mut p, &k)?;
        rw_gains(&mut p, &k)?;
        spin_params(&mut p, &k)?;
        let h_t_rot = rotor_params(&mut p, &k)?;
        rcs_params(&mut p, &k)?;
        sensor_params(&mut p, &k)?;
        let faults = faults(&s, &dev)?;
        let gd_kind0 = GUID[p.start_mode as usize];
        let spin_dps = p.ss_spin_dps;
        // the run's defaults where the case and the scenario state none: env's, vv's and dyn's stated values
        let mut cfg = Config {
            id: json::s(&s, "id", scenario).into(), case: c.clone(), dev, seed, epoch_utc: epoch, jd0,
            alt_km: v("orbit.alt"), inc_deg: v("orbit.inc"), ecc: v("orbit.ecc"), ltan_h: v("orbit.ltan"),
            u0_deg: json::f(&init, "arg_lat_deg", st.get("env_start_arg_lat_default")?),
            orbit_step_s: st.get("env_orbit_step")?, period_s, mu, zonal_max: st.whole("env_fast_zonal_degree", 1, 6)?,
            third_body: st.flag("env_force_third_body")?, drag: st.flag("env_force_drag")?, srp: st.flag("env_force_srp")?,
            density_scale: st.get("env_density_scale_default")?,
            orbit_model: if st.flag("env_orbit_precision_default")? { "pop" } else { "fast" }.into(),
            f107: st.get("env_f107_default")?, f107a: st.get("env_f107a_default")?, kp: st.get("env_kp_default")?, ap: st.get("env_ap_default")?,
            igrf_nmax: st.whole("env_field_degree", 1, 13)?, env_dt_s: st.get("env_refresh_step")?,
            env_on: [st.flag("env_torque_gravity_gradient")?, st.flag("env_torque_aero")?, st.flag("env_torque_radiation")?, st.flag("env_torque_magnetic")?],
            mass_kg: v("mass.m"), inertia, box_m, cm_offset_m,
            aref_m2: v("surface.afr"), cd: v("surface.cd"), refl: v("surface.refl"), sigma_n: surf[0], sigma_t: surf[0], vb_ratio: surf[1], spec_frac: surf[2], m_res,
            duration_s: json::f(&tm, "duration_s", st.get("vv_run_duration_default")?), dt,
            record_dt: json::f(&tm, "record_dt_s", st.get("vv_record_step_default")?),
            params: p, alg, faults, gd_kind0, h_t_rot, spin_dps, flex: None, start: Start::load(&st)?, scenario: s,
            scenario_file: sp.display().to_string(), overrides: overrides.to_vec(),
        };
        apply_engine(&mut cfg, &eng)?;
        cfg.check()?;
        // the flexible mode on the truth body (dyn_truth_plant): |delta|^2 is mpart of the coupled axis's inertia
        if cfg.case.get("flex.fmode").is_finite() {
            let k = |x: &str| cfg.case.get(x);
            let (delta, omega, zeta) = crate::gen::truthplant::flexible_mode(k("flex.fmode"), k("flex.mpart"), k("flex.zeta"), k("flex.axis") as i64, cfg.inertia);
            cfg.flex = Some(adcs_sim_core::plant::Flex { on: true, delta, omega, zeta });
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

