//! Text as JavaScript writes it, so the messages read the same: a number as `String(x)` does
//! (shortest round trip, `1e+21`, `1e-7`), a string as `JSON.stringify` quotes it.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.

/// `String(x)` of a JavaScript number.
pub fn num(x: f64) -> String {
    if x.is_nan() {
        return "NaN".into();
    }
    if x == 0.0 {
        return "0".into();
    }
    if x.is_infinite() {
        return if x > 0.0 { "Infinity".into() } else { "-Infinity".into() };
    }
    if x < 0.0 {
        return format!("-{}", num(-x));
    }
    // the shortest digits that read back as x, and the exponent: x = 0.d1d2... * 10^n
    let e = format!("{x:e}");
    let (mant, exp) = e.split_once('e').unwrap();
    let digits: String = mant.chars().filter(|c| *c != '.').collect();
    let k = digits.len() as i32;
    let n = exp.parse::<i32>().unwrap() + 1;
    if k <= n && n <= 21 {
        format!("{digits}{}", "0".repeat((n - k) as usize))
    } else if 0 < n && n <= 21 {
        format!("{}.{}", &digits[..n as usize], &digits[n as usize..])
    } else if -6 < n && n <= 0 {
        format!("0.{}{digits}", "0".repeat((-n) as usize))
    } else {
        let sign = if n - 1 < 0 { '-' } else { '+' };
        let m = if k == 1 { digits.clone() } else { format!("{}.{}", &digits[..1], &digits[1..]) };
        format!("{m}e{sign}{}", (n - 1).abs())
    }
}

/// `JSON.stringify(s)` of a string.
pub fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::num;

    #[test]
    fn numbers_read_as_javascript_writes_them() {
        for (x, s) in [(1.0, "1"), (-2.5, "-2.5"), (1e21, "1e+21"), (1e20, "100000000000000000000"), (1e-7, "1e-7"),
                       (1.5e-7, "1.5e-7"), (0.000001, "0.000001"), (123.456, "123.456"), (-0.0, "0"), (0.1 + 0.2, "0.30000000000000004"),
                       (f64::NAN, "NaN"), (f64::NEG_INFINITY, "-Infinity"), (2f64.powi(53), "9007199254740992"), (1.7976931348623157e308, "1.7976931348623157e+308")] {
            assert_eq!(num(x), s, "{x:e}");
        }
    }
}
