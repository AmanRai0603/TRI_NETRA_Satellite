//! Rust checker = JavaScript checker: the same sources accepted, the same refused, with the same
//! problems (words, file, line and column, in the same order).
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
//!
//! tests/checker_cases.json holds the cases (the refusals of tests/test_pcode.py and one or more of
//! every problem the checker names, the lexer's and the parser's too) with the JavaScript's answer
//! to each; `INTERNAL` where the JavaScript checker itself throws. Where Node is installed, the
//! answers are asked of design/js/pcode.js again (tests/js_check.mjs), so they cannot go stale, and
//! every .pc file of the repository is checked by both, alone and with its package.
//! `TRINETRA_PCODE_BLESS=1` rewrites the answers from the JavaScript.

mod common;

use common::*;
use serde_json::{json, Value as J};
use std::io::Write;
use std::process::{Command, Stdio};
use trinetra_pcode::ErrorKind;

const CASES: &str = "tests/checker_cases.json";

struct Case {
    label: String,
    files: Vec<(String, String)>,
    js: Vec<String>,
}

fn cases() -> Vec<Case> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(CASES);
    let v: J = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    v.as_array()
        .unwrap()
        .iter()
        .map(|c| Case {
            label: c["label"].as_str().unwrap().to_string(),
            files: c["files"].as_array().unwrap().iter().map(|f| (f[0].as_str().unwrap().to_string(), f[1].as_str().unwrap().to_string())).collect(),
            js: c["js"].as_array().map(|a| a.iter().map(|l| l.as_str().unwrap().to_string()).collect()).unwrap_or_default(),
        })
        .collect()
}

/// The Rust checker's answer, as the JavaScript's is written.
fn rust(files: &[(String, String)]) -> Vec<String> {
    let refs: Vec<(&str, &str)> = files.iter().map(|(f, t)| (f.as_str(), t.as_str())).collect();
    let errs = trinetra_pcode::check(&refs);
    if errs.iter().any(|e| e.kind == ErrorKind::Internal) {
        assert_eq!(errs.len(), 1, "an internal failure stands alone");
        return vec!["INTERNAL".into()];
    }
    errs.iter().map(|e| e.to_string()).collect()
}

/// The JavaScript checker's answers, if Node is here.
fn javascript(groups: &[Vec<(String, String)>]) -> Option<Vec<Vec<String>>> {
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/js_check.mjs");
    let input = J::Array(groups.iter().map(|files| json!({ "files": files.iter().map(|(f, t)| json!([f, t])).collect::<Vec<_>>() })).collect());
    let mut child = Command::new("node").arg(script).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().ok()?;
    child.stdin.take().unwrap().write_all(input.to_string().as_bytes()).unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success(), "js_check.mjs failed: {}", String::from_utf8_lossy(&out.stderr));
    Some(serde_json::from_slice(&out.stdout).unwrap())
}

fn compare(what: &str, rust: &[String], js: &[String]) -> Option<String> {
    (rust != js).then(|| format!("{what}:\n  JavaScript:\n    {}\n  Rust:\n    {}", js.join("\n    "), rust.join("\n    ")))
}

#[test]
fn the_rust_checker_answers_as_the_javascript_does() {
    let cases = cases();
    assert!(cases.len() >= 90, "the cases are all here");
    let (mut refused, mut internal, mut accepted, mut bad) = (0, 0, 0, Vec::new());
    for c in &cases {
        let r = rust(&c.files);
        match c.js.as_slice() {
            [] => accepted += 1,
            [x] if x == "INTERNAL" => internal += 1,
            _ => refused += 1,
        }
        bad.extend(compare(&c.label, &r, &c.js));
    }
    println!("{} cases: {accepted} accepted, {refused} refused with the same problems, {internal} where the JavaScript throws", cases.len());
    assert!(bad.is_empty(), "{} case(s) differ:\n{}", bad.len(), bad.join("\n"));
}

#[test]
fn every_package_of_the_repository_checks() {
    for pkg in PACKAGES {
        let srcs = sources(pkg.src);
        assert!(!srcs.is_empty(), "{}: no sources", pkg.src);
        let r = rust(&srcs);
        assert!(r.is_empty(), "{}: {}", pkg.src, r.join("\n"));
        println!("{}: {} file(s), no problems", pkg.src, srcs.len());
    }
}

#[test]
fn the_answers_are_the_javascript_s_today() {
    let cases = cases();
    let groups: Vec<Vec<(String, String)>> = cases.iter().map(|c| c.files.clone()).collect();
    let Some(js) = javascript(&groups) else {
        println!("Node is not installed: the answers of {CASES} are taken as written");
        return;
    };
    if std::env::var_os("TRINETRA_PCODE_BLESS").is_some() {
        let out = J::Array(
            cases
                .iter()
                .zip(&js)
                .map(|(c, a)| json!({ "label": c.label, "files": c.files.iter().map(|(f, t)| json!([f, t])).collect::<Vec<_>>(), "js": a }))
                .collect(),
        );
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(CASES);
        std::fs::write(path, serde_json::to_string_pretty(&out).unwrap() + "\n").unwrap();
        println!("{CASES}: {} answers written from the JavaScript", js.len());
        return;
    }
    let bad: Vec<String> = cases.iter().zip(&js).filter_map(|(c, a)| compare(&c.label, &c.js, a)).collect();
    assert!(bad.is_empty(), "{CASES} is not the JavaScript's answer now (TRINETRA_PCODE_BLESS=1 rewrites it):\n{}", bad.join("\n"));
}

#[test]
fn every_pc_file_alone_and_by_package_checks_the_same_in_both() {
    // each file alone (it may call into the others: the problems that makes are compared too), and each package
    let mut groups: Vec<Vec<(String, String)>> = Vec::new();
    for pkg in PACKAGES {
        let srcs = sources(pkg.src);
        groups.extend(srcs.iter().map(|s| vec![s.clone()]));
        groups.push(srcs);
    }
    let Some(js) = javascript(&groups) else {
        println!("Node is not installed: not compared");
        return;
    };
    let mut refused = 0;
    let mut bad = Vec::new();
    for (g, a) in groups.iter().zip(&js) {
        let r = rust(g);
        refused += usize::from(!a.is_empty());
        let names: Vec<&str> = g.iter().map(|(f, _)| f.rsplit('/').next().unwrap()).collect();
        bad.extend(compare(&names.join(" "), &r, a));
    }
    println!("{} checks of the repository's .pc files ({refused} refused, alone), the same in both", groups.len());
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}
