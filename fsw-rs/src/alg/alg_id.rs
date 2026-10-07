//! The flight algorithms' identity, written by tools/flight_build.py; do not edit.
//! sha256 (first 16 hex) over the generated algorithm sources, C and Rust (fsw/alg, fsw-rs/src/alg), this file and its twin excepted. Both build ids end with it.
/// The identity as a literal (for `concat!`, which takes literals only).
macro_rules! adcs_alg_id { () => { "b2911bb95ae3f634" } }
/// The identity of the flight algorithms this crate carries.
pub const ALG_ID: &str = adcs_alg_id!();
