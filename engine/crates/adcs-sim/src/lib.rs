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

pub const ENGINE: &str = "adcs-engine-rs/1.2.0 (adcs-case/1, POP v51 port in-loop, adcs-design, soft OILS)";

/// The data root (matlab_sils): $ADCS_ROOT, else the first ancestor of the
/// working directory holding matlab_sils/data.
pub fn data_root() -> std::path::PathBuf {
    if let Ok(r) = std::env::var("ADCS_ROOT") { return r.into(); }
    let mut d = std::env::current_dir().unwrap_or_default();
    loop {
        if d.join("matlab_sils/data").is_dir() { return d.join("matlab_sils"); }
        if d.join("data/scenarios").is_dir() { return d; }
        if !d.pop() { return "matlab_sils".into(); }
    }
}

/// The DE440 kernel the POP port reads: $ADCS_DE440, else matlab_sils/pop/03_frames_time/ephemeris/data/de440s.bsp.
pub fn pop_kernel() -> std::path::PathBuf {
    if let Ok(p) = std::env::var("ADCS_DE440") { return p.into(); }
    data_root().join("pop/03_frames_time/ephemeris/data/de440s.bsp")
}
