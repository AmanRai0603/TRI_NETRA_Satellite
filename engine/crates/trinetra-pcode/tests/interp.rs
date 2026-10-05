//! The interpreter's answers: tests/test_pcode.py's interpreter checks, here in Rust, and runs at
//! the edges (minus zero, NaN and infinity, int casts and their limit, bit operations and their
//! range, indices outside their array, division of ints by zero, short circuits, tables that step)
//! compared bit for bit, and message for message, with the JavaScript's where Node is installed.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.

mod common;

use serde_json::{json, Value as J};
use std::io::Write;
use std::process::{Command, Stdio};
use trinetra_pcode::{compile, Value};

const SELFTEST: &str = include_str!("../../../../design/pcode_selftest/selftest.pc");

const EDGES: &str = "\
fn edges(x: real[1], y: real[1], n: int, m: int) -> (a: real[1], r: real[1], i: int, f: real[1], mn: real[1], mx: real[1], \
s: real[1], h: real[1], cl: real[1], p: real[1], e: real[1], l: real[1], l10: real[1], t: real[1], at: real[1], d: int, q: int)
    a = abs(x)
    r = round(x)
    i = int(x)
    f = fmod(x, y)
    mn = min(x, y)
    mx = max(x, y, 0)
    s = sign(x)
    h = hypot(x, y)
    cl = clamp(x, -1, 1)
    p = pow(y, x)
    e = exp(x)
    l = log(y)
    l10 = log10(y)
    t = tan(x) + asin(clamp(x, -1, 1)) + acos(clamp(y, -1, 1)) + atan(x)
    at = atan2(x, y)
    d = div(n, m)
    q = rem(n, m)
end
fn bitsx(n: int, m: int) -> (a: int, o: int, x: int, l: int, r: int)
    a = band(n, m)
    o = bor(n, m)
    x = bxor(n, m)
    r = shr(n, m)
    l = shl(n, m)
end
fn idx(i: int, j: int) -> y: real[1]
    let v = [1.0, 2, 3]
    v[j] = 5
    y = v[i]
end
fn short(n: int) -> (y: int, z: bool)
    y = if n != 0 and div(10, n) > 1 then 1 else 0
    z = n == 0 or rem(10, n) == 0
end
const BIG = int(1e300)
fn big(x: real[1]) -> y: int
    y = BIG + int(x)
end
fn powers(x: real[1], n: int) -> (a: real[1], b: real[1], c: int, d: real[1], e: real[1])
    a = x^3
    b = x^-3
    c = n^2
    d = n^-1
    e = x^0
end
table st(x: real[s]) -> (x0: real[s], y: real[m]) step
    0, 1
    2, 5
end
fn stepped(t: real[s]) -> y: real[m]
    let a, b = st(t)
    y = b
end
fn literals() -> (a: real[rad], b: real[m], c: real[m/s], d: vec3[mm], e: real[1/s])
    a = 90 [deg]
    b = 1.5 [km]
    c = 3 [km/h]
    d = [1, 2, 3] [mm]
    e = 60 [rpm]
end
";

fn num(v: &J) -> Value {
    match v {
        J::Bool(b) => Value::Bool(*b),
        J::Array(a) => Value::Arr(a.iter().map(num).collect()),
        x => Value::Num(x.as_f64().unwrap()),
    }
}

/// the runs: (source, fn, inputs)
fn runs() -> Vec<(&'static str, &'static str, J)> {
    let mut r = vec![
        (SELFTEST, "root", json!([2.0, 30])),
        (SELFTEST, "root", json!([2.0, 3])),
        (SELFTEST, "lookup", json!([2.5])),
        (SELFTEST, "lookup", json!([-1.0])),
        (SELFTEST, "lookup", json!([50.0])),
        (SELFTEST, "bits", json!([[49, 50, 51, 52], 0x1234, -10.5])),
        (SELFTEST, "numbers", json!([-0.0, 0.0, 0, 1])),
        (SELFTEST, "numbers", json!([-2.5, 2.5, -6, 5])),
        (SELFTEST, "algebra", json!([[1.0, -2.0, 0.5], [0.0, 0.0, 0.0], 0.5])),
        (SELFTEST, "algebra", json!([[1.0, -2.0, 0.5], [1.0, -2.0, 0.5], 0.5])),
        (SELFTEST, "loops", json!([[-1.0, 2.0, -0.0, 3.0, 0.0, 1.0], 5])),
        (SELFTEST, "reuse", json!([2.0])),
        (SELFTEST, "dwell", json!([1.5, 0.1])),
        (EDGES, "literals", json!([])),
        (EDGES, "powers", json!([1.1, 3])),
        (EDGES, "powers", json!([-0.0, 0])),
        (EDGES, "stepped", json!([-1.0])),
        (EDGES, "stepped", json!([2.0])),
        (EDGES, "stepped", json!([9.0])),
        (EDGES, "short", json!([0])),
        (EDGES, "short", json!([3])),
        (EDGES, "big", json!([1.0])),
        (EDGES, "idx", json!([1, 1])),
        (EDGES, "idx", json!([3, 0])),
        (EDGES, "idx", json!([0, -1])),
        (EDGES, "bitsx", json!([5, 3])),
        (EDGES, "bitsx", json!([9007199254740991_i64, 9007199254740990_i64])),
        (EDGES, "bitsx", json!([4503599627370496_i64, 1])),
        (EDGES, "bitsx", json!([0, 2000])),
        (EDGES, "bitsx", json!([1, 64])),
        (EDGES, "bitsx", json!([-1, 3])),
        (EDGES, "bitsx", json!([1.5, 3])),
    ];
    for args in [
        json!([-0.0, 0.0, -7, 2]),
        json!([-0.4, 3.0, 7, -2]),
        json!([2.5, -1.5, -7, 3]),
        json!([-2.5, 1e-300, 0, 5]),
        json!([0.49999999999999994, 2.0, 1, 1]),
        json!([700.0, -1.0, 1, 1]),
        json!([-745.5, 1e308, 1, 1]),
        json!([1e22, 7.0, 9007199254740991_i64, 2]),
        json!([-1.5, -2.0, 3, 0]),
        json!([1e300, 7.0, 1, 1]),
    ] {
        r.push((EDGES, "edges", args));
    }
    r
}

fn rust(src: &str, f: &str, args: &J) -> Result<Vec<u64>, String> {
    let p = compile(&[("t.pc", src)]).map_err(|e| format!("does not check: {}", e[0].message))?;
    let args: Vec<Value> = args.as_array().unwrap().iter().map(num).collect();
    let out = p.call(f, &args).map_err(|e| e.0)?;
    Ok(out.iter().flat_map(Value::flatten).map(f64::to_bits).collect())
}

#[test]
fn test_pcode_py_s_interpreter_checks() {
    let f = |src: &str, f: &str, a: J| rust(src, f, &a).unwrap().into_iter().map(f64::from_bits).collect::<Vec<f64>>();
    // literals are SI
    assert_eq!(f(EDGES, "literals", json!([]))[..2], [std::f64::consts::FRAC_PI_2, 1500.0]);
    // a settling loop stops when it settles, and says when it does not
    let r = f(SELFTEST, "root", json!([2.0, 30]));
    assert!((r[0] - 2f64.sqrt()).abs() < 1e-12 && r[2] == 1.0);
    assert_eq!(f(SELFTEST, "root", json!([2.0, 3])), [-1.0, 3.0, 0.0]);
    // a linear table interpolates and holds its ends
    assert_eq!(f(SELFTEST, "lookup", json!([2.5])), [17.5, -0.75]);
    assert_eq!(f(SELFTEST, "lookup", json!([-1.0])), [0.0, 1.0]);
    assert_eq!(f(SELFTEST, "lookup", json!([50.0])), [0.0, 0.0]);
    // int drops the fraction as a cast does
    let cast = "fn f(x: real[1]) -> (n: int, w: int)\n    n = int(x)\n    w = int(if x < 0 then x - 0.5 else x + 0.5)\nend\n";
    assert_eq!(f(cast, "f", json!([-2.7])), [-2.0, -3.0]);
    assert_eq!(f(cast, "f", json!([2.5])), [2.0, 3.0]);
    // bit operations are exact: the CRC-16/CCITT of "1234"
    let mut crc: u32 = 0xFFFF;
    for b in b"1234" {
        crc ^= u32::from(*b) << 8;
        for _ in 0..8 {
            crc = if crc & 0x8000 != 0 { ((crc << 1) ^ 0x1021) & 0xFFFF } else { (crc << 1) & 0xFFFF };
        }
    }
    assert_eq!(f(SELFTEST, "bits", json!([[49, 50, 51, 52], 0x1234, -10.5])), [f64::from(crc), 18.0, 52.0, 13330.0, -11.0, -1.0, -4.0]);
    // powers are repeated multiplication
    let x = 1.1f64;
    assert_eq!(f(EDGES, "powers", json!([1.1, 3]))[0].to_bits(), ((x * x) * x).to_bits());
    // the run stops with the JavaScript's words
    assert_eq!(rust(EDGES, "idx", &json!([3, 0])), Err("index 3 outside 0..2 (line 30)".into()));
    assert_eq!(rust(EDGES, "big", &json!([1.0])), Err("int(1e+300): not a finite number below 2^53 (line 36)".into()));
    assert_eq!(rust(EDGES, "bitsx", &json!([-1, 3])), Err("band(-1, 3): bit operations take non-negative ints below 2^53 (line 21)".into()));
    assert_eq!(rust(EDGES, "edges", &json!([-1.5, -2.0, 3, 0])), Err("div by zero (line 17)".into()));
}

#[test]
fn every_run_answers_as_the_javascript_does() {
    let runs = runs();
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/js_run.mjs");
    let input = J::Array(runs.iter().map(|(s, f, a)| json!({ "files": [["t.pc", s]], "fn": f, "args": a })).collect());
    let Ok(mut child) = Command::new("node").arg(script).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn() else {
        println!("Node is not installed: the runs are not compared");
        for (s, f, a) in &runs {
            let _ = rust(s, f, a);
        }
        return;
    };
    child.stdin.take().unwrap().write_all(input.to_string().as_bytes()).unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success(), "js_run.mjs failed: {}", String::from_utf8_lossy(&out.stderr));
    let js: Vec<J> = serde_json::from_slice(&out.stdout).unwrap();
    let (mut values, mut errors, mut bad) = (0, 0, Vec::new());
    for ((s, f, a), j) in runs.iter().zip(&js) {
        let want: Result<Vec<u64>, String> = match j.get("out") {
            Some(o) => Ok(o.as_array().unwrap().iter().map(|h| u64::from_str_radix(h.as_str().unwrap(), 16).unwrap()).collect()),
            None => Err(j["error"].as_str().unwrap().to_string()),
        };
        // a NaN is a NaN: JavaScript cannot tell one from another (V8 writes its own pattern)
        let canon = |r: Result<Vec<u64>, String>| r.map(|v| v.into_iter().map(|b| if f64::from_bits(b).is_nan() { f64::NAN.to_bits() } else { b }).collect::<Vec<u64>>());
        let (got, want) = (canon(rust(s, f, a)), canon(want));
        match &want {
            Ok(v) => values += v.len(),
            Err(_) => errors += 1,
        }
        if got != want {
            let show = |r: &Result<Vec<u64>, String>| match r {
                Ok(v) => format!("{:?}", v.iter().map(|b| f64::from_bits(*b)).collect::<Vec<_>>()),
                Err(e) => format!("stops: {e}"),
            };
            bad.push(format!("{f}({a}):\n  JavaScript {}\n  Rust       {}", show(&want), show(&got)));
        }
    }
    println!("{} runs: {values} values bit for bit, {errors} stopped with the same words", runs.len());
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}
