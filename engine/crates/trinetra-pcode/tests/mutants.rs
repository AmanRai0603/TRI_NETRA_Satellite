//! The checker on broken sources: the repository's .pc files, each cut and spliced at random places
//! (a deterministic stream), checked by the Rust (which must not panic) and, where Node is installed,
//! by the JavaScript: the same problems, in the same words, at the same places.
//! `TRINETRA_PCODE_MUTANTS=N` breaks each file N times (40 by default).
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.

mod common;

use common::*;
use serde_json::{json, Value as J};
use std::io::Write;
use std::process::{Command, Stdio};
use trinetra_pcode::{ErrorKind, ParamInfo, Prng, Program, Ty, Value};

const MUTANTS_A_FILE: usize = 40;

/// a source broken once: a span deleted, a span copied elsewhere, or one character changed
fn mutate(src: &str, r: &mut Prng) -> String {
    let c: Vec<char> = src.chars().collect();
    let at = |r: &mut Prng| (r.next() * c.len() as f64) as usize;
    let (i, j) = {
        let a = at(r);
        (a, (a + 1 + (r.next() * 12.0) as usize).min(c.len()))
    };
    let mut out: Vec<char> = match (r.next() * 3.0) as usize {
        0 => c[..i].iter().chain(&c[j..]).copied().collect(),
        1 => {
            let k = at(r);
            c[..k].iter().chain(&c[i..j]).chain(&c[k..]).copied().collect()
        }
        _ => {
            let mut v = c.clone();
            const SWAP: &[char] = &['0', '1', '-', '[', ']', '(', ')', ',', ':', '=', '<', '^', '.', 'x', 'n', ' ', '\n', '#', '"', '|', '*', '/'];
            v[i.min(c.len() - 1)] = SWAP[(r.next() * SWAP.len() as f64) as usize];
            v
        }
    };
    if out.is_empty() {
        out.push('\n');
    }
    out.into_iter().collect()
}

fn rust(name: &str, text: &str) -> Vec<String> {
    let errs = trinetra_pcode::check(&[(name, text)]);
    if errs.iter().any(|e| e.kind == ErrorKind::Internal) {
        return vec!["INTERNAL".into()];
    }
    errs.iter().map(|e| e.to_string()).collect()
}

#[test]
fn broken_sources_are_refused_as_the_javascript_refuses_them() {
    let per_file = std::env::var("TRINETRA_PCODE_MUTANTS").ok().and_then(|s| s.parse().ok()).unwrap_or(MUTANTS_A_FILE);
    let mut r = Prng::new(11);
    let mut cases: Vec<(String, String)> = Vec::new();
    for pkg in PACKAGES {
        for (path, text) in sources(pkg.src) {
            let name = path.rsplit('/').next().unwrap().to_string();
            for _ in 0..per_file {
                cases.push((name.clone(), mutate(&text, &mut r)));
            }
        }
    }
    let mine: Vec<Vec<String>> = cases.iter().map(|(n, t)| rust(n, t)).collect();
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/js_check.mjs");
    let input = J::Array(cases.iter().map(|(n, t)| json!({ "files": [[n, t]] })).collect());
    let Ok(mut child) = Command::new("node").arg(script).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn() else {
        println!("Node is not installed: {} broken sources checked, not compared", cases.len());
        return;
    };
    child.stdin.take().unwrap().write_all(input.to_string().as_bytes()).unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success(), "js_check.mjs failed: {}", String::from_utf8_lossy(&out.stderr));
    let js: Vec<Vec<String>> = serde_json::from_slice(&out.stdout).unwrap();
    let mut bad = Vec::new();
    let (mut refused, mut internal) = (0, 0);
    for (((name, text), m), j) in cases.iter().zip(&mine).zip(&js) {
        refused += usize::from(!j.is_empty());
        internal += usize::from(j.first().is_some_and(|x| x == "INTERNAL"));
        if m != j {
            bad.push(format!("{name}:\n  JavaScript:\n    {}\n  Rust:\n    {}\n  source:\n{text}", j.join("\n    "), m.join("\n    ")));
        }
    }
    println!("{} broken sources: {refused} refused ({internal} where the JavaScript throws), the same in both", cases.len());
    assert!(bad.is_empty(), "{} differ; the first:\n{}", bad.len(), bad[..bad.len().min(3)].join("\n"));
}

/// a value as JSON for the JavaScript's interpreter: a record as an object of its fields
fn to_json(prog: &Program, ty: &Ty, v: &Value) -> J {
    match (ty, v) {
        (Ty::Rec(name), Value::Rec(fs)) => {
            let fields = prog.record_fields(name).unwrap();
            J::Object(fields.iter().zip(fs).map(|(f, x)| (f.name.to_string(), to_json(prog, f.ty, x))).collect())
        }
        (Ty::Arr(_, of) | Ty::Buf(of), Value::Arr(xs)) => J::Array(xs.iter().map(|x| to_json(prog, of, x)).collect()),
        (Ty::Stream, Value::Arr(_)) => J::Array(v.flatten().into_iter().map(|x| json!(x)).collect()),
        (_, Value::Bool(b)) => J::Bool(*b),
        (_, v) => json!(v.flatten()[0]),
    }
}

fn bounds(p: &ParamInfo) -> Option<(f64, f64)> {
    p.range.map(|(a, b)| (a.unwrap_or(0.0), b.unwrap_or(0.0)))
}

fn draw(prog: &Program, ty: &Ty, range: Option<(f64, f64)>, r: &mut Prng) -> Value {
    match ty {
        Ty::Rec(name) => Value::Rec(prog.record_fields(name).unwrap().iter().map(|f| draw(prog, f.ty, bounds(f), r)).collect()),
        Ty::Arr(n, of) => Value::Arr((0..*n).map(|_| draw(prog, of, range, r)).collect()),
        // a buffer: as long as the caller makes it (eight here)
        Ty::Buf(of) => Value::Arr((0..8).map(|_| draw(prog, of, range, r)).collect()),
        Ty::Bool => Value::Bool(r.next() < 0.5),
        // a stream: its six numbers, the key and counter whole numbers below 2^32
        Ty::Stream => Value::Arr((0..6).map(|k| Value::Num(if k < 4 { (r.next() * 4294967296.0).floor() } else { r.next() })).collect()),
        Ty::Int => {
            let (lo, hi) = range.unwrap_or((0.0, 10.0));
            Value::Num((lo + r.next() * (hi - lo)).round())
        }
        _ => {
            let (lo, hi) = range.unwrap_or((-10.0, 10.0));
            Value::Num(lo + r.next() * (hi - lo))
        }
    }
}

fn canon(bits: u64) -> u64 {
    // a NaN is a NaN: JavaScript cannot tell one from another
    if f64::from_bits(bits).is_nan() {
        f64::NAN.to_bits()
    } else {
        bits
    }
}

/// a run: the file, its text, the fn, its inputs as JSON, the Rust's answer (its outputs' bits, or the words it stops
/// with), and whether the fn uses sin or cos (a normal draw among them), whose last bits may differ (src/lib.rs)
type Run = (String, String, String, J, Result<Vec<u64>, String>, bool);

/// the same answer: bit for bit, or within 1e-12 relative where the fn uses sin or cos
fn same(got: &Result<Vec<u64>, String>, want: &Result<Vec<u64>, String>, trig: bool) -> bool {
    match (got, want) {
        (Ok(g), Ok(w)) if trig && g.len() == w.len() => g.iter().zip(w).all(|(a, b)| {
            let (x, y) = (f64::from_bits(*a), f64::from_bits(*b));
            a == b || (x - y).abs() <= 1e-12 * y.abs().max(1e-300)
        }),
        _ => got == want,
    }
}

#[test]
fn broken_sources_that_check_run_as_the_javascript_runs_them() {
    let mut r = Prng::new(13);
    let mut runs: Vec<Run> = Vec::new();
    for pkg in &PACKAGES[..2] {
        for (path, text) in sources(pkg.src) {
            let name = path.rsplit('/').next().unwrap().to_string();
            for _ in 0..MUTANTS_A_FILE * 3 {
                let m = mutate(&text, &mut r);
                let Ok(prog) = trinetra_pcode::compile(&[(name.as_str(), m.as_str())]) else { continue };
                for f in prog.functions() {
                    let args: Vec<Value> = f.inputs.iter().map(|p| draw(&prog, p.ty, bounds(p), &mut r)).collect();
                    let js_args: Vec<J> = f.inputs.iter().zip(&args).map(|(p, v)| to_json(&prog, p.ty, v)).collect();
                    let got = prog.call(f.name, &args).map(|o| o.iter().flat_map(Value::flatten).map(|x| canon(x.to_bits())).collect()).map_err(|e| e.0);
                    let trig = prog.uses(f.name, &["sin", "cos"]);
                    runs.push((name.clone(), m.clone(), f.name.to_string(), J::Array(js_args), got, trig));
                }
            }
        }
    }
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/js_run.mjs");
    let input = J::Array(runs.iter().map(|(n, t, f, a, _, _)| json!({ "files": [[n, t]], "fn": f, "args": a })).collect());
    let Ok(mut child) = Command::new("node").arg(script).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn() else {
        println!("Node is not installed: {} runs of broken sources, not compared", runs.len());
        return;
    };
    child.stdin.take().unwrap().write_all(input.to_string().as_bytes()).unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success(), "js_run.mjs failed: {}", String::from_utf8_lossy(&out.stderr));
    let js: Vec<J> = serde_json::from_slice(&out.stdout).unwrap();
    let (mut stopped, mut bad) = (0, Vec::new());
    for ((name, text, f, a, got, trig), j) in runs.iter().zip(&js) {
        let want: Result<Vec<u64>, String> = match j.get("out") {
            Some(o) => Ok(o.as_array().unwrap().iter().map(|h| canon(u64::from_str_radix(h.as_str().unwrap(), 16).unwrap())).collect()),
            None => Err(j["error"].as_str().unwrap().to_string()),
        };
        stopped += usize::from(want.is_err());
        if !same(got, &want, *trig) {
            bad.push(format!("{name} {f}({a}):\n  JavaScript {want:?}\n  Rust       {got:?}\n  source:\n{text}"));
        }
    }
    println!("{} runs of broken sources that check ({stopped} stop), the same in both", runs.len());
    assert!(bad.is_empty(), "{} differ; the first:\n{}", bad.len(), bad[..bad.len().min(3)].join("\n"));
}
