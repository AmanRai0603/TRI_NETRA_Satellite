//! adcs-sim-core -- the loop engine's pure core (spec §9.1): the plant state and
//! dynamics, orbit, environment, sensor and actuator models and the byte codecs
//! of the synthetic parts. `no_std`, no allocation, no clock, no files; every
//! maths call goes through `pm` (the pure-Rust libm), so a trajectory is the
//! same on every target. Randomness is counter-based (`rng`).
//!
//! Twin: matlab_sils/+asils/{+plant,+env,+orbit,+devices} (the MATLAB SILS).
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
#![no_std]
#![allow(clippy::needless_range_loop, clippy::too_many_arguments, clippy::excessive_precision)]

pub mod pm;
pub mod la;
/// The environment's published models, written from the design (env's methods) by tools/engine_build.py; never edited.
pub mod gen;
pub mod rng;
pub mod time;
pub mod ephem;
pub mod field;
pub mod atmos;
pub mod orbit;
pub mod plant;
pub mod torques;
pub mod comp;
pub mod sensors;
pub mod actuators;
pub mod emu;

/// Capacities (fixed arrays).
pub const NR: usize = 8;
pub const NG: usize = 4;
pub const NC: usize = 6;
pub const NH: usize = 2;
pub const NS: usize = 8;
