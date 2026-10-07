//! adcs-pop -- the Precision Orbit Propagator (matlab_sils/pop, "POP v51") in Rust.
//!
//! Every model is a port of the MATLAB function named in its module docs and is
//! tested against reference vectors that Octave printed from that function
//! (`refgen/*.m` -> `tests/data/*.json`). The engine (adcs-sim) steps it inside
//! the attitude loop exactly as asils.orbit does with the MATLAB POP.
//!
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
#![allow(clippy::needless_range_loop, clippy::too_many_arguments, clippy::excessive_precision, clippy::many_single_char_names)]

pub mod error;
pub use error::{PopError, PopResult};
pub mod la;
/// The published models written from the design (env's methods: time scales, geodetic coordinates, Earth frames, the
/// IAU 2006/2000A kernel, the tidal EOP models, with their tables) by tools/engine_build.py; never edited.
pub mod gen;
// time scales, Earth orientation, frames (matlab_sils/pop/03_frames_time)
pub mod time;
pub mod eop;
pub mod frames;
// ephemerides and radiation / third-body / relativity (03_frames_time/ephemeris, 02_forces)
pub mod spk;
pub mod ephem;
pub mod thirdbody;
pub mod srp;
pub mod erp;
pub mod relativity;
// gravity and tides (01_core/+grav, 02_forces/+solidtides, +oceantides)
pub mod gravity;
pub mod solidtides;
pub mod oceantides;
// atmosphere, space weather, drag (04_atmosphere, 02_forces/+drag)
pub mod atmos;
pub mod spaceweather;
pub mod drag;
// assembly: the force model, the world, integrators (01_core/+op, +integ)
pub mod geodetic;
pub mod accel;
pub mod integ;
