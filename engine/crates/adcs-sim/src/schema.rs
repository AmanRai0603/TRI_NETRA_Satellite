//! The scenario schema: every key the engine reads from an adcs-scenario/1 file, with its type.
//!
//! A scenario is checked against it before anything is built. A key the engine does not read
//! (a misspelling, a key from another tool) is refused by name, and so is a value of the wrong
//! type, a name outside its list (a mode, a metric kind, a window), a requirement the case does
//! not have, and a list longer than the flight software holds. A key that is absent keeps the
//! default the engine documents where it reads it. `--set` goes through the same table, so an
//! override can only change something the engine reads.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use crate::case::Case;
use crate::config::MODES;
use crate::error::Error;
use serde_json::Value;

/// What a value must be.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum T {
    Num,
    /// a number, or "case:<key>" read from the case
    NumOrCase,
    Str,
    Bool,
    /// true or false, also written 1 or 0 (as the dispatch and mode generators write it)
    Flag,
    Vec3,
    /// a number or a 3-vector (a fault's value)
    NumOrVec3,
    Quat,
    OneOf(&'static [&'static str]),
    /// a flight-software mode name
    Mode,
    /// a list of sections, each checked under `<path>[]`
    List,
}

pub const ALGORITHM_SLOTS: [&str; 8] = ["detumble", "attitude", "pointing", "mtq_pointing", "sun_acquisition", "allocation", "thrusters",
    "sun_spin" /* the old name of sun_acquisition */];
pub const METRIC_KINDS: [&str; 16] = ["time_to_rate", "ape", "ape_los", "ake", "ake_los", "rate_stability", "time_to_threshold",
    "wheel_momentum_peak", "time_to_mode", "sun_angle", "spin_rate_error", "mode_fraction", "propellant", "power_mean", "power_peak",
    "jitter"];
pub const WINDOWS: [&str; 4] = ["all", "last_orbit", "last_half_orbit", "pointing"];
pub const STATISTICS: [&str; 5] = ["max", "rms", "mean", "p95", "p99.73"];
pub const CHANNELS: [&str; 10] = ["ape_3ax", "ape_los", "ake_3ax", "ake_los", "rate", "rks", "sun_angle", "sun_angle_geo", "spin_z", "rate_err"];
pub const FAULTS: [&str; 8] = ["rotor_fail", "gimbal_stuck", "st_head_fail", "coil_fail", "gyro_bias_step", "gps_outage", "rcs_valve_fail", "mag_fail"];
/// Metric kinds the engine accepts in a file but does not compute yet: refused with the reason.
pub const NOT_COMPUTED: [(&str, &str); 1] = [("jitter", "jitter is not computed by the engine yet (a wheel-imbalance model is needed)")];

pub const SCENARIO: &[(&str, T)] = &[
    ("schema", T::OneOf(&["adcs-scenario/1"])), ("id", T::Str), ("label", T::Str), ("case", T::Str), ("product", T::Str),
    ("time.dt_s", T::Num), ("time.duration_s", T::Num), ("time.record_dt_s", T::Num),
    ("initial.arg_lat_deg", T::Num), ("initial.wheel_momentum_Nms", T::Num),
    ("initial.attitude.kind", T::OneOf(&["nadir", "random", "error_from_target", "error_from_guidance"])),
    ("initial.attitude.angle_deg", T::Num), ("initial.attitude.axis_body", T::Vec3),
    ("initial.rate.kind", T::OneOf(&["random_direction", "lvlh", "guidance", "body"])),
    ("initial.rate.magnitude_deg_s", T::NumOrCase), ("initial.rate.extra_deg_s", T::Num), ("initial.rate.value_deg_s", T::Vec3),
    ("fsw.start_mode", T::Mode), ("fsw.auto_next", T::Mode),
    ("fsw.schedule", T::List), ("fsw.schedule[].t_s", T::Num), ("fsw.schedule[].mode", T::Mode),
    ("fsw.guidance.kind", T::OneOf(&["nadir", "target", "slew", "inertial", "sun"])), ("fsw.guidance.T_s", T::Num), ("fsw.guidance.t0", T::Num),
    ("fsw.guidance.roll_deg", T::Num), ("fsw.guidance.axis", T::Vec3), ("fsw.guidance.q_inertial", T::Quat),
    ("fsw.rcs_assist", T::Flag), ("fsw.rcs_dump", T::Flag), ("fsw.yaw_flip", T::Flag),
    ("fsw.avanzini_k_over_n", T::Num), ("fsw.avanzini_lambda", T::Num), ("fsw.bdot_gain_scale", T::Num), ("fsw.capture_deg", T::Num),
    ("fsw.capture_rate_deg_s", T::Num), ("fsw.detumble_exit_deg_s", T::Num), ("fsw.detumble_hold_s", T::Num), ("fsw.dump_gain", T::Num),
    ("fsw.handover_hold_s", T::Num), ("fsw.handover_in_dps", T::Num), ("fsw.handover_out_dps", T::Num), ("fsw.l1_gain", T::Num),
    ("fsw.mekf_gate", T::Num), ("fsw.mekf_rej_max", T::Num), ("fsw.mtq_gain_d", T::Num), ("fsw.mtq_gain_p", T::Num), ("fsw.mtq_gg_ff", T::Num),
    ("fsw.mtq_wn", T::Num), ("fsw.mtq_zeta", T::Num), ("fsw.rate_lpf_s", T::Num), ("fsw.rcs_damp_s", T::Num), ("fsw.roll_gain", T::Num),
    ("fsw.roll_gate_deg", T::Num), ("fsw.roll_wn_orbits", T::Num), ("fsw.roll_zeta", T::Num), ("fsw.rw_bandwidth", T::Num),
    ("fsw.rw_damping", T::Num), ("fsw.rw_rate_hz", T::Num), ("fsw.spin_rate_dps", T::Num), ("fsw.ss_gain", T::Num),
    ("fsw.sun_acq_rate_deg_s", T::Num), ("fsw.sun_spin_dwell_out_s", T::Num), ("fsw.sun_spin_perp_out_dps", T::Num), ("fsw.wheel_bias_Nms", T::Num),
    ("faults", T::List), ("faults[].t_s", T::Num), ("faults[].kind", T::OneOf(&FAULTS)), ("faults[].index", T::Num), ("faults[].value", T::NumOrVec3),
    ("metrics", T::List), ("metrics[].id", T::Str), ("metrics[].kind", T::OneOf(&METRIC_KINDS)), ("metrics[].window", T::Str),
    ("metrics[].statistic", T::OneOf(&STATISTICS)), ("metrics[].channel", T::OneOf(&CHANNELS)), ("metrics[].mode", T::Mode),
    ("metrics[].from_s", T::Num), ("metrics[].hold_s", T::Num), ("metrics[].rate_threshold_deg_s", T::Num), ("metrics[].threshold_deg", T::Num),
    ("metrics[].requirement", T::Str), ("metrics[].limit", T::Num), ("metrics[].sense", T::OneOf(&["max", "min"])),
    ("metrics[].unit_min", T::Flag), ("metrics[].end_at_mode_exit", T::Flag),
];

/// The schedule entries the flight software holds (fsw/params/params.toml max_schedule).
pub const MAX_SCHEDULE: usize = adcs_fsw::params::MAX_SCHEDULE;

fn entry(path: &str) -> Option<T> { SCENARIO.iter().find(|(p, _)| *p == path).map(|(_, t)| *t) }

/// Is `path` a key `--set` may change: a leaf the engine reads, or an algorithm slot.
pub fn settable(path: &str) -> bool {
    if let Some(slot) = path.strip_prefix("fsw.algorithms.") { return ALGORITHM_SLOTS.contains(&slot); }
    matches!(entry(path), Some(t) if t != T::List)
}

fn is_num(v: &Value) -> bool { v.as_f64().map(f64::is_finite).unwrap_or(false) }
fn is_vec(v: &Value, n: usize) -> bool { v.as_array().map(|a| a.len() == n && a.iter().all(is_num)).unwrap_or(false) }

fn check_value(path: &str, t: T, v: &Value, case: &Case, bad: &mut Vec<String>) {
    let ok = match t {
        T::Num => is_num(v),
        T::NumOrCase => is_num(v) || v.as_str().and_then(|s| s.strip_prefix("case:")).map(|k| {
            if case.v.get(k).map(|x| x.is_finite()).unwrap_or(false) { true } else { bad.push(format!("{path} = {v}: the case does not state {k}")); true }
        }).unwrap_or(false),
        T::Str => v.is_string(),
        T::Bool => v.is_boolean(),
        T::Flag => v.is_boolean() || v.as_f64().map(|x| x == 0.0 || x == 1.0).unwrap_or(false),
        T::Vec3 => is_vec(v, 3),
        T::NumOrVec3 => is_num(v) || is_vec(v, 3),
        T::Quat => is_vec(v, 4),
        T::OneOf(list) => v.as_str().map(|s| list.contains(&s)).unwrap_or(false),
        T::Mode => v.as_str().map(|s| s.is_empty() || MODES.contains(&s)).unwrap_or(false),
        T::List => v.as_array().map(|a| a.iter().all(Value::is_object)).unwrap_or(false),
    };
    if !ok {
        let want = match t {
            T::Num => "a finite number".to_string(), T::NumOrCase => "a finite number or \"case:<key>\"".into(), T::Str => "text".into(),
            T::Bool => "true or false".into(), T::Flag => "true or false (or 1 or 0)".into(), T::Vec3 => "[x, y, z] of finite numbers".into(), T::NumOrVec3 => "a number or [x, y, z]".into(),
            T::Quat => "[x, y, z, w] of finite numbers".into(), T::OneOf(l) => format!("one of {}", l.join(", ")),
            T::Mode => format!("a mode: {}", MODES.join(", ")), T::List => "a list of sections".into(),
        };
        bad.push(format!("{path} = {v}: must be {want}"));
    }
}

fn walk(v: &Value, path: &str, case: &Case, bad: &mut Vec<String>) {
    let Some(o) = v.as_object() else { return };
    for (k, x) in o {
        let p = if path.is_empty() { k.clone() } else { format!("{path}.{k}") };
        if p == "fsw.algorithms" {
            match x.as_object() {
                Some(a) => for (slot, id) in a {
                    if !ALGORITHM_SLOTS.contains(&slot.as_str()) { bad.push(format!("fsw.algorithms.{slot}: no such slot; the slots are {}", ALGORITHM_SLOTS[..7].join(", "))); }
                    else if !id.is_string() { bad.push(format!("fsw.algorithms.{slot} = {id}: must be an algorithm id (text)")); }
                },
                None => bad.push("fsw.algorithms: must be a section of slot = algorithm id".into()),
            }
            continue;
        }
        match entry(&p) {
            Some(T::List) => {
                check_value(&p, T::List, x, case, bad);
                for e in x.as_array().into_iter().flatten() { walk(e, &format!("{p}[]"), case, bad); }
            }
            Some(t) => check_value(&p, t, x, case, bad),
            None if x.is_object() && SCENARIO.iter().any(|(q, _)| q.starts_with(&format!("{p}."))) => walk(x, &p, case, bad),
            None => bad.push(format!("{p}: the engine does not read this key (misspelt, or from another tool)")),
        }
    }
}

/// Check a scenario against the schema and the case: every problem named, none substituted.
pub fn check_scenario(s: &Value, case: &Case) -> Result<(), Error> {
    let mut bad = vec![];
    if !s.is_object() { return Err(Error::refused("a scenario is a JSON section of keys")); }
    walk(s, "", case, &mut bad);
    if let Some(n) = s.pointer("/fsw/schedule").and_then(Value::as_array).map(Vec::len) {
        if n > MAX_SCHEDULE { bad.push(format!("fsw.schedule: {n} entries; the flight software holds at most {MAX_SCHEDULE}")); }
    }
    for (i, f) in s.get("faults").and_then(Value::as_array).into_iter().flatten().enumerate() {
        if let Some(ix) = f.get("index") {
            if !ix.as_f64().map(|x| x >= 1.0 && x.fract() == 0.0).unwrap_or(false) { bad.push(format!("faults[{i}].index = {ix}: a unit number from 1")); }
        }
    }
    for (i, m) in s.get("metrics").and_then(Value::as_array).into_iter().flatten().enumerate() {
        let id = m.get("id").and_then(Value::as_str).unwrap_or("?");
        if let Some(k) = m.get("kind").and_then(Value::as_str) {
            // listed for information it records null; judged, it could only ever fail or pass by accident
            let judged = m.get("requirement").is_some() || m.get("limit").is_some();
            if let Some((_, why)) = NOT_COMPUTED.iter().find(|(n, _)| *n == k).filter(|_| judged) {
                bad.push(format!("metrics[{i}] ({id}): judged against a requirement, but {why}"));
            }
        } else { bad.push(format!("metrics[{i}] ({id}): has no kind")); }
        if let Some(w) = m.get("window").and_then(Value::as_str) {
            let ok = WINDOWS.contains(&w) || w.strip_prefix("after_s:").map(|x| x.parse::<f64>().map(f64::is_finite).unwrap_or(false)).unwrap_or(false);
            if !ok { bad.push(format!("metrics[{i}] ({id}).window = {w:?}: one of {} or after_s:<seconds>", WINDOWS.join(", "))); }
        }
        if let Some(r) = m.get("requirement").and_then(Value::as_str) {
            if !r.starts_with("req.") || !case.v.contains_key(r) {
                bad.push(format!("metrics[{i}] ({id}).requirement = {r:?}: the case {} has no such requirement", case.id));
            }
        }
        if m.get("requirement").is_some() && m.get("limit").is_some() {
            bad.push(format!("metrics[{i}] ({id}): a requirement from the case or a limit, not both"));
        }
    }
    if bad.is_empty() { Ok(()) } else { Err(Error::refused(format!("scenario {}: {}", s.get("id").and_then(Value::as_str).unwrap_or("?"), bad.join("; ")))) }
}

/// The schema as JSON, for the MATLAB twin (matlab_sils/data/scenario_schema.json), so both
/// read a scenario by the same rules.
pub fn to_json() -> Value {
    // a list of {path, type[, values]}: MATLAB turns keys with dots into other names, so paths are values here
    let keys: Vec<Value> = SCENARIO.iter().map(|(p, t)| match t {
        T::OneOf(l) => serde_json::json!({"path": p, "type": "one_of", "values": l}),
        T::Mode => serde_json::json!({"path": p, "type": "one_of", "values": MODES.iter().copied().chain([""]).collect::<Vec<_>>()}),
        other => serde_json::json!({"path": p, "type": format!("{other:?}").to_ascii_lowercase()}),
    }).collect();
    serde_json::json!({
        "schema": "adcs-scenario-schema/1",
        "generated_by": "engine/crates/adcs-sim/src/schema.rs (ADCS_WRITE_SCHEMA=1 cargo test -p adcs-sim schema_json)",
        "keys": keys, "algorithm_slots": ALGORITHM_SLOTS, "metric_windows": WINDOWS, "max_schedule": MAX_SCHEDULE,
        "not_computed": NOT_COMPUTED.iter().map(|(k, why)| serde_json::json!({"kind": k, "why": why})).collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod t {
    #[test]
    fn schema_json_is_current() {
        let f = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../matlab_sils/data/scenario_schema.json");
        let text = serde_json::to_string_pretty(&super::to_json()).unwrap() + "\n";
        if std::env::var_os("ADCS_WRITE_SCHEMA").is_some() {
            crate::fsio::write(&f, &text).unwrap();
        }
        assert_eq!(std::fs::read_to_string(&f).unwrap_or_default(), text,
            "matlab_sils/data/scenario_schema.json is not the engine's schema: ADCS_WRITE_SCHEMA=1 cargo test -p adcs-sim schema_json");
    }
}
