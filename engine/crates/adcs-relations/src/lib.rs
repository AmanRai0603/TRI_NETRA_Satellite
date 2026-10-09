//! TRI-NETRA relations: the design's relations (spec/physics) and every group's computing rows (design/groups), each once, written from the design by tools/engine_build.py; do not edit.
//! The translator's modules are under `gen`, re-exported here; `wasm` is the groups' test apps' WebAssembly face.
#![allow(clippy::all)]
pub mod gen;
pub use gen::*;
pub mod wasm;
