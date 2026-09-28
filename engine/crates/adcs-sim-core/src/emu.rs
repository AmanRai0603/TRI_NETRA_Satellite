//! Device emulators' byte codecs (spec §9.5): engineering values -> exactly what
//! each synthetic part puts on its port, and the flight software's writes ->
//! actuator commands. The protocol numbers are fsw-rs/src/devices.rs (the Rust
//! twin of fsw/include/adcs_devices.h, fsw/pseudocode/09_drivers_hal.md).
use crate::la::{Q, V3};
use crate::pm::*;
use crate::{NC, NG, NR};

#[allow(dead_code, clippy::all)]
#[path = "../../../../fsw-rs/src/devices.rs"]
pub mod proto;
use proto::*;

pub fn crc16(p: &[u8]) -> u16 {
    let mut c: u16 = 0xFFFF;
    for &b in p {
        c ^= (b as u16) << 8;
        for _ in 0..8 { c = if c & 0x8000 != 0 { (c << 1) ^ 0x1021 } else { c << 1 }; }
    }
    c
}

fn q16(x: f64) -> i16 { round(clamp(x, -32768.0, 32767.0)) as i16 }
fn q32(x: f64) -> i32 { round(clamp(x, -2147483648.0, 2147483647.0)) as i32 }

/// I2C register block: status (bit0 valid), three int16 (LE).
fn reg7(ok: bool, v: &V3, lsb: f64) -> [u8; 7] {
    let mut b = [0u8; 7];
    b[0] = ok as u8;
    for i in 0..3 { b[1 + 2*i..3 + 2*i].copy_from_slice(&q16(v[i]/lsb).to_le_bytes()); }
    b
}
pub fn mag_regs(ok: bool, b_t: &V3) -> [u8; 7] { reg7(ok, b_t, MAG_LSB_T) }
pub fn unit_regs(ok: bool, u: &V3) -> [u8; 7] { reg7(ok, u, 1.0/Q15) }
pub fn gyro_resp(ok: bool, w: &V3) -> [u8; 13] {
    let mut b = [0u8; 13];
    b[0] = ok as u8;
    for i in 0..3 { b[1 + 4*i..5 + 4*i].copy_from_slice(&q32(w[i]/GYRO_LSB).to_le_bytes()); }
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
pub fn st_frame(heads: &[(bool, Q)], out: &mut [u8; 64]) -> usize {
    let mut pl = [0u8; 1 + 17*2];
    let nh = heads.len().min(2);
    pl[0] = nh as u8;
    for h in 0..nh {
        let e = &mut pl[1 + 17*h..1 + 17*(h + 1)];
        e[0] = heads[h].0 as u8;
        for k in 0..4 { e[1 + 4*k..5 + 4*k].copy_from_slice(&q32(heads[h].1[k]*Q30).to_le_bytes()); }
    }
    frame(ST_SYNC2, &pl[..1 + 17*nh], out)
}
/// GNSS UART frame: fix u8, r int32 [cm] x3, v int32 [mm/s] x3.
pub fn gps_frame(fix: bool, r: &V3, v: &V3, out: &mut [u8; 64]) -> usize {
    let mut pl = [0u8; 25];
    pl[0] = fix as u8;
    for i in 0..3 {
        pl[1 + 4*i..5 + 4*i].copy_from_slice(&q32(r[i]/0.01).to_le_bytes());
        pl[13 + 4*i..17 + 4*i].copy_from_slice(&q32(v[i]/0.001).to_le_bytes());
    }
    frame(GPS_SYNC2, &pl, out)
}
/// Rotor telemetry CAN frame: (id, data) with h [1e-9 N m s] and gimbal angle [1e-7 rad].
pub fn rotor_tm(i: usize, h: f64, delta: f64) -> (u32, [u8; 8]) {
    let mut d = [0u8; 8];
    d[..4].copy_from_slice(&q32(h/H_LSB).to_le_bytes());
    d[4..].copy_from_slice(&q32(delta/DELTA_LSB).to_le_bytes());
    (CAN_ROTOR_TM + i as u32, d)
}

/// What the flight software commanded, decoded from its bus writes.
#[derive(Clone, Copy, Debug, Default)]
pub struct Commands { pub m_body: V3, pub cmd_r: [f64; NR], pub cmd_g: [f64; NG], pub duty: [f64; NC] }

pub fn decode_pwm(pwm: &[i16; 8], m_max: f64, c: &mut Commands) {
    for i in 0..3 { c.m_body[i] = pwm[i] as f64/Q15*m_max; }
}
/// One CAN frame from the flight software (rotor torque, gimbal rate, valve on-times).
pub fn decode_can(id: u32, data: &[u8; 8], tmax: &[f64; NR], gmax: f64, dt: f64, c: &mut Commands) {
    let v16 = i16::from_le_bytes([data[0], data[1]]) as f64/Q15;
    if id >= CAN_ROTOR_CMD && id < CAN_ROTOR_CMD + NR as u32 { let i = (id - CAN_ROTOR_CMD) as usize; c.cmd_r[i] = v16*tmax[i]; }
    else if id >= CAN_GIMBAL_CMD && id < CAN_GIMBAL_CMD + NG as u32 { c.cmd_g[(id - CAN_GIMBAL_CMD) as usize] = v16*gmax; }
    else if id == CAN_VALVES { for i in 0..NC { c.duty[i] = data[i] as f64*VALVE_LSB_S/dt; } }
}
