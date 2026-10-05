//! The units and their dimensions: the table of design/js/pcode.js, a unit's text read to its
//! dimension and its scale to SI, and a dimension written back as text.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.

use crate::error::{ErrorKind, PcodeError, Pos};
use crate::jsfmt;
use crate::vmath;
use std::f64::consts::PI;

/// A dimension: the powers of [length, mass, time, current, temperature]. Powers are whole
/// numbers except after a refused `sqrt` (the checker halves them, as the JavaScript does).
pub type Dim = [f64; 5];
pub const DIMLESS: Dim = [0.0; 5];

const fn d(l: f64, m: f64, t: f64, i: f64, k: f64) -> Dim {
    [l, m, t, i, k]
}

/// Every unit: its name, its dimension, its scale to SI, in the JavaScript's order.
pub const UNITS: &[(&str, Dim, f64)] = &[
    ("1", d(0., 0., 0., 0., 0.), 1.0),
    ("m", d(1., 0., 0., 0., 0.), 1.0),
    ("km", d(1., 0., 0., 0., 0.), 1e3),
    ("cm", d(1., 0., 0., 0., 0.), 1e-2),
    ("mm", d(1., 0., 0., 0., 0.), 1e-3),
    ("um", d(1., 0., 0., 0., 0.), 1e-6),
    ("AU", d(1., 0., 0., 0., 0.), 1.495978707e11),
    ("s", d(0., 0., 1., 0., 0.), 1.0),
    ("ms", d(0., 0., 1., 0., 0.), 1e-3),
    ("min", d(0., 0., 1., 0., 0.), 60.0),
    ("h", d(0., 0., 1., 0., 0.), 3600.0),
    ("day", d(0., 0., 1., 0., 0.), 86400.0),
    ("yr", d(0., 0., 1., 0., 0.), 365.25 * 86400.0),
    ("kg", d(0., 1., 0., 0., 0.), 1.0),
    ("g", d(0., 1., 0., 0., 0.), 1e-3),
    ("N", d(1., 1., -2., 0., 0.), 1.0),
    ("mN", d(1., 1., -2., 0., 0.), 1e-3),
    ("uN", d(1., 1., -2., 0., 0.), 1e-6),
    ("J", d(2., 1., -2., 0., 0.), 1.0),
    ("W", d(2., 1., -3., 0., 0.), 1.0),
    ("mW", d(2., 1., -3., 0., 0.), 1e-3),
    ("Pa", d(-1., 1., -2., 0., 0.), 1.0),
    ("kPa", d(-1., 1., -2., 0., 0.), 1e3),
    ("A", d(0., 0., 0., 1., 0.), 1.0),
    ("mA", d(0., 0., 0., 1., 0.), 1e-3),
    ("C", d(0., 0., 1., 1., 0.), 1.0),
    ("V", d(2., 1., -3., -1., 0.), 1.0),
    ("ohm", d(2., 1., -3., -2., 0.), 1.0),
    ("T", d(0., 1., -2., -1., 0.), 1.0),
    ("uT", d(0., 1., -2., -1., 0.), 1e-6),
    ("nT", d(0., 1., -2., -1., 0.), 1e-9),
    ("K", d(0., 0., 0., 0., 1.), 1.0),
    ("Hz", d(0., 0., -1., 0., 0.), 1.0),
    ("rpm", d(0., 0., -1., 0., 0.), 2.0 * PI / 60.0),
    ("rad", d(0., 0., 0., 0., 0.), 1.0),
    ("deg", d(0., 0., 0., 0., 0.), PI / 180.0),
    ("arcsec", d(0., 0., 0., 0., 0.), PI / 648000.0),
];

const DIM_NAMES: [&str; 5] = ["m", "kg", "s", "A", "K"];

/// A dimension as text: `m/s^2`, `kg m^2`, `1/(m s)`, `1`.
pub fn dim_text(d: &Dim) -> String {
    let (mut up, mut dn) = (Vec::new(), Vec::new());
    for (i, &p) in d.iter().enumerate() {
        if p > 0.0 {
            up.push(format!("{}{}", DIM_NAMES[i], if p != 1.0 { format!("^{}", jsfmt::num(p)) } else { String::new() }));
        }
        if p < 0.0 {
            dn.push(format!("{}{}", DIM_NAMES[i], if p != -1.0 { format!("^{}", jsfmt::num(-p)) } else { String::new() }));
        }
    }
    if up.is_empty() && dn.is_empty() {
        return "1".into();
    }
    let num = if up.is_empty() { "1".to_string() } else { up.join(" ") };
    let den = match dn.len() {
        0 => String::new(),
        1 => format!("/{}", dn[0]),
        _ => format!("/({})", dn.join(" ")),
    };
    num + &den
}

pub(crate) fn dim_eq(a: &Dim, b: &Dim) -> bool {
    a.iter().zip(b).all(|(x, y)| x == y)
}
pub(crate) fn dim_add(a: &Dim, b: &Dim, s: f64) -> Dim {
    let mut r = *a;
    for i in 0..5 {
        r[i] = a[i] + s * b[i];
    }
    r
}
pub(crate) fn dim_mul(a: &Dim, k: f64) -> Dim {
    let mut r = *a;
    for x in r.iter_mut() {
        *x *= k;
    }
    r
}

fn is_ws(c: char) -> bool {
    c.is_whitespace() || c == '\u{feff}'
}

/// `/^[^/]*\/\s*\([^()]*\)\s*$/`: parentheses that hold the whole denominator.
fn denominator_in_parens(text: &str) -> bool {
    let Some(slash) = text.find('/') else { return false };
    let rest = text[slash + 1..].trim_start_matches(is_ws);
    let Some(rest) = rest.strip_prefix('(') else { return false };
    let Some(close) = rest.find(['(', ')']) else { return false };
    if !rest[close..].starts_with(')') {
        return false;
    }
    rest[close + 1..].trim_start_matches(is_ws).is_empty()
}

/// A unit's text to its dimension and its scale to SI: `kg m^2/s`, `1/s`, `kg/(m s)`.
pub fn unit_of(text: &str, pos: &Pos) -> Result<(Dim, f64), PcodeError> {
    let err = |m: String| PcodeError::new(ErrorKind::Check, m, pos);
    let mut dim = DIMLESS;
    let mut scale = 1.0;
    if text.contains('(') && !denominator_in_parens(text) {
        return Err(err(format!("parentheses in a unit group its denominator only: [kg/(m s)], not [{text}]")));
    }
    let flat: String = text.chars().map(|c| if c == '(' || c == ')' { ' ' } else { c }).collect();
    let parts: Vec<&str> = flat.split('/').collect();
    if parts.len() > 2 {
        return Err(err(format!("a unit has at most one /: [{text}]")));
    }
    let den = parts.get(1).copied().unwrap_or("");
    for (part, sign) in [(parts[0], 1.0), (den, -1.0)] {
        for f in part.split(is_ws).filter(|s| !s.is_empty()) {
            let (name, power) = match f.split_once('^') {
                Some((n, p)) => (n, Some(p)),
                None => (f, None),
            };
            let name_ok = name == "1" || (!name.is_empty() && name.chars().all(|c| c.is_ascii_alphabetic()));
            let power_ok = power.is_none_or(|p| {
                let digits = p.strip_prefix('-').unwrap_or(p);
                !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit())
            });
            if !name_ok || !power_ok {
                return Err(err(format!("not a unit factor: {f} in [{text}]")));
            }
            let Some(&(_, udim, uscale)) = UNITS.iter().find(|u| u.0 == name) else {
                let names: Vec<&str> = UNITS.iter().map(|u| u.0).collect();
                return Err(err(format!("no unit {name} (the units: {})", names.join(", "))));
            };
            let k = sign * power.map_or(1.0, |p| p.parse::<f64>().unwrap());
            dim = dim_add(&dim, &dim_mul(&udim, k), 1.0);
            scale *= vmath::pow(uscale, k);
        }
    }
    Ok((dim, scale))
}
