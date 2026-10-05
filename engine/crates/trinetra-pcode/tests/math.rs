//! The maths library against JavaScript's Math, where Node is installed: tan, asin, acos, atan,
//! atan2, exp, log, log10 and pow bit for bit on every argument; sin and cos (glibc's in Node,
//! fdlibm's here: see src/lib.rs) within one unit in the last place, and how often they differ.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.

use serde_json::{json, Value as J};
use std::collections::BTreeMap;
use std::io::Write;
use std::process::{Command, Stdio};
use trinetra_pcode::{math, Prng};

const N: usize = 40_000;

fn args(r: &mut Prng, i: usize) -> f64 {
    match i % 5 {
        0 => (r.next() - 0.5) * 20.0,
        1 => (r.next() - 0.5) * 2.0,
        2 => math::exp((r.next() - 0.5) * 60.0) * if r.next() < 0.5 { -1.0 } else { 1.0 },
        3 => (r.next() - 0.5) * 1e6,
        _ => r.next() * 700.0,
    }
}

type F = fn(f64, f64) -> f64;

#[test]
fn the_maths_is_javascript_s() {
    let fns: [(&str, F, bool); 11] = [
        ("sin", |x, _| math::sin(x), true),
        ("cos", |x, _| math::cos(x), true),
        ("tan", |x, _| math::tan(x), false),
        ("asin", |x, _| math::asin(x), false),
        ("acos", |x, _| math::acos(x), false),
        ("atan", |x, _| math::atan(x), false),
        ("atan2", math::atan2, false),
        ("exp", |x, _| math::exp(x), false),
        ("log", |x, _| math::log(x), false),
        ("log10", |x, _| math::log10(x), false),
        ("pow", math::pow, false),
    ];
    let mut r = Prng::new(7);
    let mut req = BTreeMap::new();
    let mut xs = BTreeMap::new();
    for (name, _, _) in &fns {
        let v: Vec<(f64, f64)> = (0..N)
            .map(|i| {
                let (x, y) = (args(&mut r, i), args(&mut r, i + 1));
                match *name {
                    "log" | "log10" => (x.abs(), y),
                    // a whole power of a negative number too, now and then
                    "pow" if i % 7 == 0 => (x, (y % 8.0).trunc()),
                    "pow" => (x.abs(), y / 100.0),
                    _ => (x, y),
                }
            })
            .collect();
        req.insert(*name, J::Array(v.iter().map(|(x, y)| json!([format!("{:016x}", x.to_bits()), format!("{:016x}", y.to_bits())])).collect()));
        xs.insert(*name, v);
    }
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/js_math.mjs");
    let Ok(mut child) = Command::new("node").arg(script).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn() else {
        println!("Node is not installed: the maths is not compared");
        return;
    };
    child.stdin.take().unwrap().write_all(serde_json::to_string(&req).unwrap().as_bytes()).unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success(), "js_math.mjs failed: {}", String::from_utf8_lossy(&out.stderr));
    let js: BTreeMap<String, Vec<String>> = serde_json::from_slice(&out.stdout).unwrap();
    for (name, f, trig) in fns {
        let (mut differ, mut worst) = (0, 0u64);
        for ((x, y), h) in xs[name].iter().zip(&js[name]) {
            let (want, got) = (f64::from_bits(u64::from_str_radix(h, 16).unwrap()), f(*x, *y));
            if got.to_bits() == want.to_bits() || (got.is_nan() && want.is_nan()) {
                continue;
            }
            differ += 1;
            let ulps = if got.is_sign_negative() == want.is_sign_negative() { got.to_bits().abs_diff(want.to_bits()) } else { u64::MAX };
            worst = worst.max(ulps);
            assert!(trig, "{name}({x:e}, {y:e}): Rust {got:e}, JavaScript {want:e}");
        }
        println!("{name}: {N} arguments, {differ} not bit for bit{}", if differ > 0 { format!(" (at most {worst} ulp)") } else { String::new() });
        assert!(worst <= 1, "{name}: {worst} ulp from JavaScript's");
    }
}
