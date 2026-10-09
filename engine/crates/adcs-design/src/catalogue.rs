//! A bought wheel's or CMG's model parameters from its datasheet (adcs-datasheet/1): catalogue's method
//! (catalogue/catderive.pc, generated into `gen::catderive`), which tools/catalogue.py asks for through
//! `adcs design derive`. This module reads the datasheet's numbers and its two text fields (the supply's and the
//! dimensions', the numbers written in them) and hands the method's values back with what it assumed; the tool writes
//! the file's derived block and its list of assumptions.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use crate::gen::catderive as cd;
use adcs_sim::Error;
use serde_json::{json, Value};

/// The numbers written in a text, as the datasheet's reader takes them: each run of digits with its decimals.
fn numbers(s: &str) -> Vec<f64> {
    let b = s.as_bytes();
    let mut out = vec![];
    let mut i = 0;
    while i < b.len() {
        if b[i].is_ascii_digit() {
            let st = i;
            while i < b.len() && b[i].is_ascii_digit() { i += 1; }
            if i + 1 < b.len() && b[i] == b'.' && b[i + 1].is_ascii_digit() {
                i += 1;
                while i < b.len() && b[i].is_ascii_digit() { i += 1; }
            }
            if let Ok(x) = s[st..i].parse::<f64>() { out.push(x); }
        } else {
            i += 1;
        }
    }
    out
}

/// A datasheet's number, nan where it states none.
fn stated(d: &Value, key: &str) -> f64 { d.get(key).and_then(|x| x.as_f64()).unwrap_or(f64::NAN) }

/// The supply the datasheet states (the first number of its voltage_V), nan when it states none.
fn supply(d: &Value) -> f64 {
    match d.get("voltage_V") {
        Some(Value::String(s)) if !s.is_empty() => numbers(s).first().copied().unwrap_or(f64::NAN),
        Some(Value::Number(n)) if n.as_f64() != Some(0.0) => n.as_f64().unwrap_or(f64::NAN),
        _ => f64::NAN,
    }
}

/// The datasheet's box (sides in mm, the first three numbers before any bracket when it is written a x b x c) and
/// whether it says the unit is a 0.2U tuna can.
fn dims(d: &Value) -> ([f64; 3], bool, bool) {
    match d.get("dims_mm") {
        Some(Value::String(s)) if !s.is_empty() => {
            let n = numbers(s.split('(').next().unwrap_or(""));
            if s.contains('x') && n.len() >= 3 { ([n[0], n[1], n[2]], true, false) } else { ([f64::NAN; 3], false, s.contains("0.2U")) }
        }
        _ => ([f64::NAN; 3], false, false),
    }
}

/// One datasheet's derivation, as tools/catalogue.py writes it into the file: whether it is selectable and which of
/// the four numbers it misses; the derived values (in the file's order) when it is; what was assumed; and the rule's
/// constants the file's list of assumptions names.
pub fn derive(c: &Value) -> Result<Value, Error> {
    let d = c.get("datasheet").filter(|x| x.is_object()).ok_or_else(|| Error::malformed("a datasheet file (adcs-datasheet/1) has no datasheet block"))?;
    let (h, tq, mass, ps, rpm, pk) = (stated(d, "h_mNms"), stated(d, "torque_mNm"), stated(d, "mass_g"), stated(d, "power_steady_W"),
                                      stated(d, "h_speed_rpm"), stated(d, "power_peak_W"));
    let (selectable, mh, mt, mm, mp) = cd::catalogue_selectable(h, tq, mass, ps);
    let missing: Vec<&str> = [("h_mNms", mh), ("torque_mNm", mt), ("mass_g", mm), ("power_steady_W", mp)].iter().filter(|x| x.1).map(|x| x.0).collect();
    let (assumed_rpm, rotor_share, _g, _b, _c, _cr, _v, _pe, no_load, static_fraction, stribeck, _s, cmg_rotor_torque, _t) = cd::catalogue_rules();
    let rules = json!({"assumed_rpm": assumed_rpm, "rotor_share": rotor_share, "no_load": no_load, "static_fraction": static_fraction,
                       "stribeck": stribeck, "cmg_rotor_torque": cmg_rotor_torque});
    if !selectable {
        return Ok(json!({"selectable": false, "missing": missing, "rules": rules}));
    }
    let (bx, boxed, tuna) = dims(d);
    let (volume, tuna_assumed) = cd::catalogue_volume(bx[0], bx[1], bx[2], boxed, tuna);
    let (hh, tau, w, j, radius, rotor_mass, coulomb, viscous, us, ud, peak, m, rpm_assumed, peak_assumed) = cd::catalogue_rotor(h, tq, rpm, mass, ps, pk);
    // a stated value is written as the datasheet wrote it
    let as_stated = |key: &str| d.get(key).cloned().unwrap_or(Value::Null);
    let vol = if volume.is_nan() { Value::Null } else { json!(volume) };
    let mut x = json!({"h_max_Nms": hh, "torque_max_Nm": tau, "speed_max_rad_s": w, "rotor_inertia_kgm2": j, "rotor_radius_m": radius,
        "rotor_mass_kg": rotor_mass, "friction_coulomb_Nm": coulomb, "friction_viscous_Nms": viscous, "static_imbalance_kgm": us,
        "dynamic_imbalance_kgm2": ud, "power_steady_W": as_stated("power_steady_W"), "power_peak_W": if peak_assumed { json!(peak) } else { as_stated("power_peak_W") },
        "mass_kg": m, "volume_L": vol});
    let kind = c.get("type").and_then(|t| t.as_str()).unwrap_or("");
    let mut assumed = json!({"rpm": rpm_assumed, "tuna": tuna_assumed, "peak": peak_assumed});
    if kind == "reaction_wheel" {
        let (kt, res, v, fs, st, supply_assumed) = cd::catalogue_wheel_motor(supply(d), stated(d, "speed_max_rpm"), w, tau, coulomb);
        for (k, v) in [("motor_kt_Nm_per_A", kt), ("motor_resistance_ohm", res), ("bus_voltage_V", v), ("friction_static_Nm", fs), ("stribeck_speed_rad_s", st)] {
            x[k] = json!(v);
        }
        assumed["supply"] = json!(supply_assumed);
    }
    if kind == "cmg" || kind == "cmg_cluster" {
        let (rm, rs, rt, gr, gp, vm) = cd::catalogue_cmg(hh, tau, w);
        for (k, v) in [("rotor_momentum_Nms", rm), ("rotor_speed_rad_s", rs), ("rotor_torque_max_Nm", rt), ("gimbal_rate_max_rad_s", gr),
                       ("gimbal_power_W", gp), ("vscmg_rotor_momentum_Nms", vm)] {
            x[k] = json!(v);
        }
    }
    Ok(json!({"selectable": true, "missing": missing, "derived": x, "assumed": assumed, "rules": rules}))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_numbers_of_a_text_are_read_as_the_reader_reads_them() {
        assert_eq!(numbers("8 (6.4-16.8)"), [8.0, 6.4, 16.8]);
        assert_eq!(numbers("3.3 and 5"), [3.3, 5.0]);
        assert_eq!(numbers("0.2U+ tuna can per unit, >64 mm dia"), [0.2, 64.0]);
        assert_eq!(numbers("33.5 x 33.5 x 17"), [33.5, 33.5, 17.0]);
        assert_eq!(numbers("1.2.3 x."), [1.2, 3.0]);
        assert!(numbers("none").is_empty());
    }
}
