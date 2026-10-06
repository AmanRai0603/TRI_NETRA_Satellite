//! Guidance (04_guidance.md): the reference attitude and rate of each pointing state, the nadir yaw
//! flip, the boresight offset, and which guidance a controller state flies; twin of adcs_guid.c.
//! Group gdn (design/groups.toml); MATLAB twin: asils.fsw.guidance, yaw_flip, boresight_offset.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use crate::alg::guidance as alg;
use crate::math::*;

#[derive(Clone, Copy, Debug, Default)]
pub struct Guid {
    pub q_off: Q, pub roll_deg: f64, pub t0: f64, pub t_slew: f64, pub axis: V3, pub q_inertial: Q,
    pub sun_axis: V3, pub roll_axis: V3, pub sun_eci: V3,
    /// nadir family turned 180 deg about the boresight (power face towards the Sun)
    pub flip: bool,
}

pub struct Ref { pub q: Q, pub w: V3, pub wd: V3 }

/// The guidance a controller state flies: 0 nadir, 1 target, 2 slew, 4 Sun (= adcs_guid_kind).
/// Written from the design: guidance::guid_kind (src/alg).
pub fn kind_of(mode: u8) -> i32 { alg::guid_kind(mode as i64) as i32 }

/// kind: 0 nadir, 1 target, 2 slew, 3 inertial, 4 sun.
/// Written from the design: guidance::guidance (src/alg).
pub fn guidance(kind: i32, r: &V3, v: &V3, t: f64, g: &Guid) -> Ref {
    let (q, w, wd) = alg::guidance(kind as i64, *r, *v, t, g.q_off, g.roll_deg, g.t0, g.t_slew, g.axis, g.q_inertial,
                                   g.sun_axis, g.roll_axis, g.sun_eci, g.flip);
    Ref { q, w, wd }
}

/// Nadir-family yaw flip with hysteresis: turn 180 deg about the boresight when the power face would
/// look away from the Sun (04_guidance.md).
/// Written from the design: guidance::yaw_flip (src/alg).
pub fn yaw_flip(g: &mut Guid, r: &V3, v: &V3, hyst: f64) {
    g.flip = alg::yaw_flip(*r, *v, g.q_off, g.sun_axis, g.roll_axis, g.sun_eci, g.flip, hyst);
}

/// Rotation taking body +y onto the boresight.
/// Written from the design: guidance::boresight_offset (src/alg).
pub fn boresight_offset(bs_in: &V3) -> Q { alg::boresight_offset(*bs_in) }
