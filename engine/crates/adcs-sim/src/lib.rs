//! adcs-sim -- the SILS engine (spec §9): loads a case (adcs-case/1 CSV), a
//! scenario (adcs-scenario/1 JSON) and a product with its parts (the MATLAB
//! twin's data/ tree), derives every parameter as asils.config does, runs the
//! closed loop with the flight software (C or Rust) behind the byte HAL, and
//! writes an adcs-rec/1 run directory the Python report reads.
//!
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
#![allow(clippy::needless_range_loop, clippy::too_many_arguments)]

pub mod json;
pub mod case;
pub mod product;
pub mod lqr;
pub mod config;
pub mod run;
pub mod metrics;
pub mod rec;
pub mod fsio;
pub mod store;
pub mod index;
pub mod flight;
pub mod error;
pub mod schema;
pub mod source;
/// The engine's set-up relations written from the design with the platform's maths (env_orbit_start; tools/engine_build.py):
/// generated, never edited.
pub mod gen;

pub use error::{Error, Kind};

pub const ENGINE: &str = concat!("adcs-engine-rs/", env!("CARGO_PKG_VERSION"), " (adcs-case/1, POP v51 port in-loop, adcs-design, soft OILS)");

/// The data root (matlab_sils): $ADCS_ROOT, else the first ancestor of the
/// working directory holding matlab_sils/data.
pub fn data_root() -> std::path::PathBuf {
    if let Some(r) = std::env::var_os("ADCS_ROOT").filter(|r| !r.is_empty()) { return r.into(); }
    let mut d = std::env::current_dir().unwrap_or_default();
    loop {
        if d.join("matlab_sils/data").is_dir() { return d.join("matlab_sils"); }
        if d.join("data/scenarios").is_dir() { return d; }
        if !d.pop() { break; }
    }
    // a released kit: the data sits beside the program, whatever folder it is run from
    if let Some(k) = kit_root() { return k; }
    "matlab_sils".into()
}

/// The folder of a released kit or installed package: the program's own folder (or one
/// above it, or a bundle's Contents/Resources) when it holds `data/scenarios` and the kit's
/// `VERSION` file.
pub fn kit_root() -> Option<std::path::PathBuf> {
    let exe = std::env::current_exe().ok()?.canonicalize().ok()?;
    let is_kit = |d: &std::path::Path| d.join("data/scenarios").is_dir() && d.join("VERSION").is_file();
    // a kit folder, or a macOS app bundle (the program in Contents/MacOS, the data in Contents/Resources)
    exe.ancestors().skip(1).take(3).flat_map(|d| [d.to_path_buf(), d.join("Resources")]).find(|d| is_kit(d))
}

/// Where runs and sized designs are written: `$TRINETRA_STORE`; else, for a released kit,
/// `.trinetra/store` in the home folder, so the kit itself is never written to and a new
/// version finds the old results; else `store/` in the data folder (a checkout).
pub fn store_root() -> std::path::PathBuf {
    if let Some(s) = std::env::var_os("TRINETRA_STORE").filter(|s| !s.is_empty()) { return s.into(); }
    let root = data_root();
    if root.join("VERSION").is_file() {
        let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")).filter(|h| !h.is_empty());
        if let Some(h) = home { return std::path::PathBuf::from(h).join(".trinetra").join("store"); }
    }
    root.join("store")
}

/// Where the DE440 kernel sits under a data folder.
const DE440_REL: &str = "pop/03_frames_time/ephemeris/data/de440s.bsp";

/// The places the DE440 kernel is looked for, in order: $ADCS_DE440; the data folder; beside the
/// program (a released kit); the checkout this program was built from. The physics data ships
/// with the program, so a design read from a database, run from any folder, still finds it.
pub fn pop_kernel_candidates() -> Vec<std::path::PathBuf> {
    if let Some(p) = std::env::var_os("ADCS_DE440").filter(|p| !p.is_empty()) { return vec![p.into()]; }
    let mut v = vec![data_root().join(DE440_REL)];
    if let Some(k) = kit_root() { v.push(k.join(DE440_REL)); }
    v.push(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../matlab_sils").join(DE440_REL));
    v.dedup();
    v
}

/// The DE440 kernel the POP port reads: the first of [`pop_kernel_candidates`] that exists, else
/// the first (so the error names where it was looked for; see [`pop_kernel_missing`]).
pub fn pop_kernel() -> std::path::PathBuf {
    let c = pop_kernel_candidates();
    c.iter().find(|p| p.is_file()).cloned().unwrap_or_else(|| c[0].clone())
}

/// The refusal when no DE440 kernel is found, naming every place looked in.
pub fn pop_kernel_missing() -> Option<Error> {
    let c = pop_kernel_candidates();
    (!c.iter().any(|p| p.is_file())).then(|| Error::refused(format!(
        "the DE440 kernel (de440s.bsp) is not found; looked in: {}. It ships with the program (a kit's data folder) or is named by $ADCS_DE440",
        c.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join("; "))))
}
