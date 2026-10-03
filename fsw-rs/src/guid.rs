//! Guidance (04_guidance.md): the reference attitude and rate of each pointing state, the nadir yaw
//! flip, the boresight offset, and which guidance a controller state flies; twin of adcs_guid.c.
//! Group gdn (design/groups.toml); MATLAB twin: asils.fsw.guidance, yaw_flip, boresight_offset.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use crate::m::*;
use crate::math::*;
use crate::params::Mode;

#[derive(Clone, Copy, Debug, Default)]
pub struct Guid {
    pub q_off: Q, pub roll_deg: f64, pub t0: f64, pub t_slew: f64, pub axis: V3, pub q_inertial: Q,
    pub sun_axis: V3, pub roll_axis: V3, pub sun_eci: V3,
    /// nadir family turned 180 deg about the boresight (power face towards the Sun)
    pub flip: bool,
}

pub struct Ref { pub q: Q, pub w: V3, pub wd: V3 }

/// The guidance a controller state flies: 0 nadir, 1 target, 2 slew, 4 Sun (= adcs_guid_kind).
pub fn kind_of(mode: u8) -> i32 {
    match Mode::from_u8(mode) {
        Some(Mode::TargetFine) => 1,
        Some(Mode::SlewFine) => 2,
        Some(Mode::SunMtq) | Some(Mode::SunFine) => 4,
        _ => 0,
    }
}

/// kind: 0 nadir, 1 target, 2 slew, 3 inertial, 4 sun.
pub fn guidance(kind: i32, r: &V3, v: &V3, t: f64, g: &Guid) -> Ref {
    let rh = unit(r);
    let vh = unit(v);
    let mr = scale3(&rh, -1.0);
    let nrm = unit(&cross(&vh, &mr));
    let ram = unit(&cross(&mr, &nrm));
    let mut rm = [[0.0; 3]; 3];
    for i in 0..3 { rm[0][i] = -ram[i]; rm[1][i] = -rh[i]; rm[2][i] = -nrm[i]; }
    let mut q_nad = fromdcm(&rm);
    if g.q_off.iter().any(|&x| x != 0.0) { q_nad = qmult(&q_nad, &g.q_off); }
    if g.flip {
        let u = if norm3(&g.roll_axis) > 0.0 { unit(&g.roll_axis) } else { [1.0, 0.0, 0.0] };
        q_nad = qmult(&q_nad, &[u[0], u[1], u[2], 0.0]);
    }
    let w_orb = scale3(&cross(r, v), 1.0/dot(r, r));
    let mut wd = [0.0; 3];
    let ax = if norm3(&g.axis) > 0.0 { unit(&g.axis) } else { [1.0, 0.0, 0.0] };
    match kind {
        1 => {
            let q = qmult(&q_nad, &fromrotvec(&scale3(&ax, g.roll_deg*D2R)));
            Ref { q, w: mat3_vec(&dcm(&q), &w_orb), wd }
        }
        2 => {
            let tau = clamp((t - g.t0)/g.t_slew, 0.0, 1.0);
            let ph = g.roll_deg*D2R;
            let (mut sd, mut sdd) = (0.0, 0.0);
            if t > g.t0 && t < g.t0 + g.t_slew {
                sd = (1.0 - cos(2.0*PI*tau))/g.t_slew;
                sdd = 2.0*PI*sin(2.0*PI*tau)/(g.t_slew*g.t_slew);
            }
            let s = tau - sin(2.0*PI*tau)/(2.0*PI);
            let q = qmult(&q_nad, &fromrotvec(&scale3(&ax, ph*s)));
            let wo = mat3_vec(&dcm(&q), &w_orb);
            let mut w = [0.0; 3];
            // the slew frame turns at ax ph s_dot relative to the orbiting frame: transport term on w_orb
            let tr = cross(&scale3(&ax, ph*sd), &wo);
            for i in 0..3 { w[i] = wo[i] + ax[i]*ph*sd; wd[i] = ax[i]*ph*sdd - tr[i]; }
            Ref { q, w, wd }
        }
        3 => Ref { q: g.q_inertial, w: [0.0; 3], wd },
        4 => {
            let a = if norm3(&g.sun_axis) > 0.0 { unit(&g.sun_axis) } else { [0.0, 0.0, -1.0] };
            let mut b = if norm3(&g.roll_axis) > 0.0 { g.roll_axis } else { [1.0, 0.0, 0.0] };
            let ab = dot(&a, &b);
            for i in 0..3 { b[i] -= ab*a[i]; }
            if norm3(&b) < 1e-6 { b = [-a[1]*a[0], 1.0 - a[1]*a[1], -a[1]*a[2]]; }
            let b = unit(&b);
            let s = unit(&g.sun_eci);
            let ns = dot(&nrm, &s);
            let mut e2 = [nrm[0] - ns*s[0], nrm[1] - ns*s[1], nrm[2] - ns*s[2]];
            if norm3(&e2) < 1e-6 { e2 = [-s[2]*s[0], -s[2]*s[1], 1.0 - s[2]*s[2]]; }
            let e2 = unit(&e2);
            let b3 = cross(&a, &b);
            let e3 = cross(&s, &e2);
            let mut m = [[0.0; 3]; 3];
            for i in 0..3 { for j in 0..3 { m[i][j] = a[i]*s[j] + b[i]*e2[j] + b3[i]*e3[j]; } }
            Ref { q: fromdcm(&m), w: [0.0; 3], wd }
        }
        _ => Ref { q: q_nad, w: mat3_vec(&dcm(&q_nad), &w_orb), wd },
    }
}

/// Nadir-family yaw flip with hysteresis: turn 180 deg about the boresight when the power face would
/// look away from the Sun (04_guidance.md).
pub fn yaw_flip(g: &mut Guid, r: &V3, v: &V3, hyst: f64) {
    let mut g0 = *g;
    g0.flip = false;
    let q = guidance(0, r, v, 0.0, &g0).q;
    let sb = mat3_vec(&dcm(&q), &unit(&g.sun_eci));
    let a = if norm3(&g.sun_axis) > 0.0 { unit(&g.sun_axis) } else { [0.0, 0.0, -1.0] };
    let d = dot(&a, &sb);
    if d < -hyst { g.flip = true; } else if d > hyst { g.flip = false; }
}

/// Rotation taking body +y onto the boresight.
pub fn boresight_offset(bs_in: &V3) -> Q {
    let ey = [0.0, 1.0, 0.0];
    let bs = unit(bs_in);
    let ax = cross(&ey, &bs);
    let c = dot(&ey, &bs);
    let s = norm3(&ax);
    if s < 1e-12 { return if c <= 0.0 { [1.0, 0.0, 0.0, 0.0] } else { [0.0, 0.0, 0.0, 1.0] }; }
    let k = scale3(&ax, 1.0/s);
    let kk = skew(&k);
    let k2 = mat3_mul(&kk, &kk);
    let mut a = [[0.0; 3]; 3];
    for i in 0..3 { for j in 0..3 { a[i][j] = (if i == j { 1.0 } else { 0.0 }) + s*kk[i][j] + (1.0 - c)*k2[i][j]; } }
    fromdcm(&a)
}
