//! The engine's source fingerprint (ADCS_ENGINE_SOURCE): FNV-1a over every source file that
//! decides what a run computes (the engine's model crates, the C and the Rust flight software),
//! line endings made uniform, so the same sources give the same fingerprint on every OS. A run
//! records it; `adcs results stale` compares it with the running engine's. The results store's own
//! files (how a run is filed, listed and its replay command printed) compute nothing a run reports,
//! so they are left out: changing them does not make every stored run stale.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use std::path::{Path, PathBuf};

fn walk(d: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(d) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() { walk(&p, out); }
        else if matches!(p.extension().and_then(|x| x.to_str()), Some("rs" | "c" | "h" | "toml")) { out.push(p); }
    }
}

/// Engine sources that file, index and print stored runs, or fetch an input's bytes (each input's own
/// fingerprint is in the run): they decide nothing a run computes.
const NOT_DECIDING: [&str; 3] = ["engine/crates/adcs-sim/src/store.rs", "engine/crates/adcs-sim/src/index.rs", "engine/crates/adcs-sim/src/source.rs"];

fn main() {
    let here = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let repo = here.join("../../..");
    let dirs = ["engine/crates/adcs-sim/src", "engine/crates/adcs-sim-core/src", "engine/crates/adcs-pop/src",
                "engine/crates/adcs-fsw-abi/src", "fsw/src", "fsw/include", "fsw/params", "fsw-rs/src"];
    let mut files = vec![];
    for d in dirs {
        let p = repo.join(d);
        println!("cargo:rerun-if-changed={}", p.display());
        walk(&p, &mut files);
    }
    let rel = |p: &Path| p.strip_prefix(&repo).unwrap_or(p).to_string_lossy().replace('\\', "/");
    files.retain(|p| !NOT_DECIDING.contains(&rel(p).as_str()));
    files.sort_by_key(|p| rel(p));
    let mut h: u64 = 0xcbf29ce484222325;
    let mut eat = |b: &[u8]| for x in b { h ^= *x as u64; h = h.wrapping_mul(0x100000001b3); };
    for f in &files {
        eat(rel(f).as_bytes());
        eat(&[0]);
        let bytes: Vec<u8> = std::fs::read(f).unwrap_or_default().into_iter().filter(|b| *b != b'\r').collect();
        eat(&bytes);
    }
    println!("cargo:rustc-env=ADCS_ENGINE_SOURCE={h:016x}");
}
