//! What the tests share: the repository's pseudocode packages, their sources and the vectors the
//! JavaScript interpreter (design/js/pcode.js) drew for each, read to the exact bits.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
#![allow(dead_code)]

use std::path::{Path, PathBuf};

pub fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..").canonicalize().unwrap()
}

/// A directory's `.pc` files, sorted as tools/pcode.py sorts them, as (path, text).
pub fn sources(dir: &str) -> Vec<(String, String)> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(root().join(dir))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "pc"))
        .collect();
    files.sort();
    files.into_iter().map(|p| (p.display().to_string(), std::fs::read_to_string(&p).unwrap())).collect()
}

pub fn compile(srcs: &[(String, String)]) -> trinetra_pcode::Program {
    let refs: Vec<(&str, &str)> = srcs.iter().map(|(f, t)| (f.as_str(), t.as_str())).collect();
    match trinetra_pcode::compile(&refs) {
        Ok(p) => p,
        Err(errs) => panic!("{}", errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")),
    }
}

/// One call: its inputs and outputs as numbers.
pub struct Call {
    pub input: Vec<f64>,
    pub output: Vec<f64>,
}

/// A vector: one call of a fn, or a run of calls of a proc, its state carried.
pub struct Set {
    pub calls: Vec<Call>,
}

/// Every vector of one function.
pub struct Entry {
    /// `module::name`
    pub name: String,
    pub exact: bool,
    pub proc: bool,
    pub sets: Vec<Set>,
}

/// A package: where its sources are and where the JavaScript's vectors of it are.
pub struct Package {
    pub name: &'static str,
    pub src: &'static str,
    pub vectors: &'static str,
}

pub const PACKAGES: &[Package] = &[
    // 1.0.0's relations, archived (archive/design-1.0); the same text the design holds (tests/test_from_design.py)
    Package { name: "physics", src: "archive/design-1.0/spec/physics", vectors: "matlab_sils/data/physics_vectors.json" },
    Package { name: "selftest", src: "design/pcode_selftest", vectors: "matlab_sils/data/pcselftest_vectors.json" },
    Package { name: "flight software", src: "fsw/pseudocode", vectors: "fsw/tests/pcode_vectors.txt" },
    // the groups' computing rows as tools/groupcode.py wires them; their vectors drawn by tools/engine_build.py (adcs-relations)
    Package { name: "groups", src: "design/groups", vectors: "matlab_sils/data/groups_vectors.json" },
];

fn bits_of(v: &serde_json::Value) -> f64 {
    let w = v.as_array().unwrap();
    let (hi, lo) = (w[0].as_u64().unwrap(), w[1].as_u64().unwrap());
    f64::from_bits((hi << 32) | lo)
}
fn all_bits(v: &serde_json::Value) -> Vec<f64> {
    v.as_array().unwrap().iter().map(bits_of).collect()
}

/// The vectors of a package, in the file's order.
pub fn vectors(path: &str) -> Vec<Entry> {
    let text = std::fs::read_to_string(root().join(path)).unwrap();
    if path.ends_with(".json") {
        let v: serde_json::Value = serde_json::from_str(&text).unwrap();
        let mut out = Vec::new();
        for (name, e) in v["vectors"].as_object().unwrap() {
            let proc = e["proc"].as_bool().unwrap_or(false);
            let sets = e["sets"]
                .as_array()
                .unwrap()
                .iter()
                .map(|s| {
                    let calls = if proc { s["calls"].as_array().unwrap().iter().collect::<Vec<_>>() } else { vec![s] };
                    Set { calls: calls.into_iter().map(|c| Call { input: all_bits(&c["in_bits"]), output: all_bits(&c["out_bits"]) }).collect() }
                })
                .collect();
            out.push(Entry { name: name.clone(), exact: e["exact"].as_bool().unwrap(), proc, sets });
        }
        return out;
    }
    // the flight software's text: `name exact set nx ny`, then the nx + ny values' bits
    let mut out: Vec<Entry> = Vec::new();
    let mut last_set = None;
    for line in text.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()) {
        let w: Vec<&str> = line.split_whitespace().collect();
        let (name, exact, set) = (w[0], w[1] == "1", w[2].parse::<u64>().unwrap());
        let (nx, ny) = (w[3].parse::<usize>().unwrap(), w[4].parse::<usize>().unwrap());
        let vals: Vec<f64> = w[5..].iter().map(|h| f64::from_bits(u64::from_str_radix(h, 16).unwrap())).collect();
        assert_eq!(vals.len(), nx + ny, "{name}: a line of {} values, not {}", vals.len(), nx + ny);
        let call = Call { input: vals[..nx].to_vec(), output: vals[nx..].to_vec() };
        if out.last().is_none_or(|e| e.name != name) {
            out.push(Entry { name: name.to_string(), exact, proc: false, sets: Vec::new() });
            last_set = None;
        }
        let e = out.last_mut().unwrap();
        if last_set == Some(set) {
            e.sets.last_mut().unwrap().calls.push(call);
            e.proc = true;
        } else {
            e.sets.push(Set { calls: vec![call] });
            last_set = Some(set);
        }
    }
    out
}

/// A function's bare name from `module::name`.
pub fn bare(name: &str) -> &str {
    name.rsplit("::").next().unwrap()
}
