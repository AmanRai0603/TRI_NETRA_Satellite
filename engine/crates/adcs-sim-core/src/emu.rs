//! Device emulators' byte codecs (spec §9.5): engineering values -> exactly what
//! each synthetic part puts on its port, and the flight software's writes ->
//! actuator commands. The scaling (each field's counts within its range, the command
//! words' values) is l3_oils_row_07's emucodec, generated from the design
//! (`gen::emucodec`, S7.12): the inverse of the flight software's drivers, its
//! constants taken from the drivers' own conversions. What is left here is the rig's:
//! the packets' layout, sync bytes, CRC and bus ids (fsw-rs/src/devices.rs, the Rust
//! twin of fsw/include/adcs_devices.h, fsw/pseudocode/09_drivers_hal.md).
use crate::gen::emucodec as ec;
use crate::la::{Q, V3};
use crate::{NC, NG, NR};

#[allow(dead_code, clippy::all)]
#[path = "../../../../fsw-rs/src/devices.rs"]
pub mod proto;
use proto::*;

/// The emulators' scaling: one count of each field and the fields' ranges, as the drivers read them.
pub use ec::EmuScale as Scale;
/// The scaling, from the drivers' conversions (emucodec's emu_scale): made once a run.
pub fn scale() -> Scale { ec::emu_scale() }

pub fn crc16(p: &[u8]) -> u16 {
    let mut c: u16 = 0xFFFF;
    for &b in p {
        c ^= (b as u16) << 8;
        for _ in 0..8 { c = if c & 0x8000 != 0 { (c << 1) ^ 0x1021 } else { c << 1 }; }
    }
    c
}

/// I2C register block: status (bit0 valid), three int16 (LE).
fn reg7(ok: bool, c: [i64; 3]) -> [u8; 7] {
    let mut b = [0u8; 7];
    b[0] = ok as u8;
    for i in 0..3 { b[1 + 2*i..3 + 2*i].copy_from_slice(&(c[i] as i16).to_le_bytes()); }
    b
}
pub fn mag_regs(ok: bool, b_t: &V3, s: &Scale) -> [u8; 7] { reg7(ok, ec::mag_counts(*b_t, *s)) }
pub fn unit_regs(ok: bool, u: &V3, s: &Scale) -> [u8; 7] { reg7(ok, ec::unit_counts(*u, *s)) }
pub fn gyro_resp(ok: bool, w: &V3, s: &Scale) -> [u8; 13] {
    let c = ec::gyro_counts(*w, *s);
    let mut b = [0u8; 13];
    b[0] = ok as u8;
    for i in 0..3 { b[1 + 4*i..5 + 4*i].copy_from_slice(&(c[i] as i32).to_le_bytes()); }
    b
}

fn frame(sync2: u8, pl: &[u8], out: &mut [u8; 64]) -> usize {
    let n = pl.len();
    out[0] = ST_SYNC1; out[1] = sync2; out[2] = n as u8;
    out[3..3 + n].copy_from_slice(pl);
    out[3 + n..5 + n].copy_from_slice(&crc16(pl).to_le_bytes());
    n + 5
}
/// Star-tracker UART frame: nh | per head valid u8, q (x y z w) int32 Q30.
pub fn st_frame(heads: &[(bool, Q)], out: &mut [u8; 64], s: &Scale) -> usize {
    let mut pl = [0u8; 1 + 17*2];
    let nh = heads.len().min(2);
    pl[0] = nh as u8;
    for h in 0..nh {
        let e = &mut pl[1 + 17*h..1 + 17*(h + 1)];
        e[0] = heads[h].0 as u8;
        let c = ec::quat_counts(heads[h].1, *s);
        for k in 0..4 { e[1 + 4*k..5 + 4*k].copy_from_slice(&(c[k] as i32).to_le_bytes()); }
    }
    frame(ST_SYNC2, &pl[..1 + 17*nh], out)
}
/// GNSS UART frame: fix u8, r int32 [cm] x3, v int32 [mm/s] x3.
pub fn gps_frame(fix: bool, r: &V3, v: &V3, out: &mut [u8; 64], s: &Scale) -> usize {
    let mut pl = [0u8; 25];
    pl[0] = fix as u8;
    let (cr, cv) = ec::fix_counts(*r, *v, *s);
    for i in 0..3 {
        pl[1 + 4*i..5 + 4*i].copy_from_slice(&(cr[i] as i32).to_le_bytes());
        pl[13 + 4*i..17 + 4*i].copy_from_slice(&(cv[i] as i32).to_le_bytes());
    }
    frame(GPS_SYNC2, &pl, out)
}
/// Rotor telemetry CAN frame: (id, data) with h [1e-9 N m s] and gimbal angle [1e-7 rad].
pub fn rotor_tm(i: usize, h: f64, delta: f64, s: &Scale) -> (u32, [u8; 8]) {
    let mut d = [0u8; 8];
    let (ch, cd) = ec::rotor_counts(h, delta, *s);
    d[..4].copy_from_slice(&(ch as i32).to_le_bytes());
    d[4..].copy_from_slice(&(cd as i32).to_le_bytes());
    (CAN_ROTOR_TM + i as u32, d)
}

/// What the flight software commanded, decoded from its bus writes.
#[derive(Clone, Copy, Debug, Default)]
pub struct Commands { pub m_body: V3, pub cmd_r: [f64; NR], pub cmd_g: [f64; NG], pub duty: [f64; NC] }

pub fn decode_pwm(pwm: &[i16; 8], m_max: f64, s: &Scale, c: &mut Commands) {
    c.m_body = ec::coil_dipole([pwm[0] as i64, pwm[1] as i64, pwm[2] as i64], m_max, *s);
}
/// One CAN frame from the flight software (rotor torque, gimbal rate, valve on-times).
pub fn decode_can(id: u32, data: &[u8; 8], tmax: &[f64; NR], gmax: f64, dt: f64, s: &Scale, c: &mut Commands) {
    let w16 = i16::from_le_bytes([data[0], data[1]]) as i64;
    if id >= CAN_ROTOR_CMD && id < CAN_ROTOR_CMD + NR as u32 { let i = (id - CAN_ROTOR_CMD) as usize; c.cmd_r[i] = ec::command_value(w16, tmax[i], *s); }
    else if id >= CAN_GIMBAL_CMD && id < CAN_GIMBAL_CMD + NG as u32 { c.cmd_g[(id - CAN_GIMBAL_CMD) as usize] = ec::command_value(w16, gmax, *s); }
    else if id == CAN_VALVES { for i in 0..NC { c.duty[i] = ec::valve_duty(data[i] as i64, dt, *s); } }
}
