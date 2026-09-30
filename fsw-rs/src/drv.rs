//! Drivers of the synthetic parts over the byte HAL (fsw/pseudocode/09); twin of adcs_drv.c.
use crate::devices::*;
use crate::hal::{CanFrame, Hal, Status};
use crate::params::{Params, MAX_COUPLES, MAX_GIMBALS, MAX_HEADS, MAX_ROTORS};

#[derive(Clone, Copy, Debug, Default)]
pub struct Meas {
    pub mag_ok: bool, pub b: [f64; 3],
    pub gyro_ok: bool, pub w: [f64; 3],
    pub sun_ok: bool, pub sun: [f64; 3],
    pub es_ok: bool, pub nadir: [f64; 3],
    pub st_ok: bool, pub st_valid: [bool; MAX_HEADS], pub q_st: [[f64; 4]; MAX_HEADS],
    pub gps_ok: bool, pub r: [f64; 3], pub v: [f64; 3],
    pub h: [f64; MAX_ROTORS], pub delta: [f64; MAX_GIMBALS],
}

fn rd16(b: &[u8]) -> i16 { i16::from_le_bytes([b[0], b[1]]) }
fn rd32(b: &[u8]) -> i32 { i32::from_le_bytes([b[0], b[1], b[2], b[3]]) }

pub fn crc16(p: &[u8]) -> u16 {
    let mut c: u16 = 0xFFFF;
    for &b in p {
        c ^= (b as u16) << 8;
        for _ in 0..8 { c = if c & 0x8000 != 0 { (c << 1) ^ 0x1021 } else { c << 1 }; }
    }
    c
}

const RXCAP: usize = 256;

/// Frame assembly buffers, one per UART port.
#[derive(Clone, Copy)]
pub struct Drv { rx: [[u8; RXCAP]; 3], len: [usize; 3] }

impl Default for Drv { fn default() -> Self { Drv { rx: [[0; RXCAP]; 3], len: [0; 3] } } }

impl Drv {
    pub fn reset(&mut self) { self.len = [0; 3]; }

    /// Pull available bytes; the payload length of the newest complete, CRC-good frame with sync2.
    fn uart_frame<H: Hal>(&mut self, hal: &mut H, port: u8, sync2: u8, payload: &mut [u8]) -> Option<usize> {
        let pi = port as usize;
        let mut tmp = [0u8; RXCAP];
        while let Ok(got) = hal.uart_read(port, &mut tmp) {
            if got == 0 { break; }
            for &x in &tmp[..got] {
                if self.len[pi] >= RXCAP { self.len[pi] = 0; }
                self.rx[pi][self.len[pi]] = x;
                self.len[pi] += 1;
            }
        }
        let mut found = None;
        let b = &mut self.rx[pi];
        let len = &mut self.len[pi];
        loop {
            let mut s = 0;
            while s + 1 < *len && !(b[s] == ST_SYNC1 && b[s + 1] == sync2) { s += 1; }
            if s + 3 > *len { break; }
            let n = b[s + 2] as usize;
            if s + 3 + n + 2 > *len {
                if s > 0 { b.copy_within(s..*len, 0); *len -= s; }
                break;
            }
            let crc = b[s + 3 + n] as u16 | ((b[s + 4 + n] as u16) << 8);
            if crc16(&b[s + 3..s + 3 + n]) == crc && n <= payload.len() {
                payload[..n].copy_from_slice(&b[s + 3..s + 3 + n]);
                found = Some(n);
            }
            b.copy_within(s + 5 + n..*len, 0);
            *len -= s + 5 + n;
        }
        found
    }

    pub fn read<H: Hal>(&mut self, hal: &mut H, p: &Params, z: &mut Meas) {
        let mut rx = [0u8; 13];
        let tx = [0u8; 1];
        z.mag_ok = hal.i2c_xfer(MAG_BUS, MAG_ADDR, &tx, &mut rx[..7]) == Status::Ok && rx[0] & 1 != 0;
        if z.mag_ok { for i in 0..3 { z.b[i] = rd16(&rx[1 + 2*i..]) as f64*MAG_LSB_T; } }
        z.gyro_ok = false;
        if p.has_gyro != 0 {
            let tx = [GYRO_CMD, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
            if hal.spi_xfer(GYRO_BUS, GYRO_CS, &tx, &mut rx[..13]) == Status::Ok && rx[0] & 1 != 0 {
                z.gyro_ok = true;
                for i in 0..3 { z.w[i] = rd32(&rx[1 + 4*i..]) as f64*GYRO_LSB; }
            }
        }
        z.sun_ok = p.has_sun != 0 && hal.i2c_xfer(SUN_BUS, SUN_ADDR, &tx, &mut rx[..7]) == Status::Ok && rx[0] & 1 != 0;
        if z.sun_ok { for i in 0..3 { z.sun[i] = rd16(&rx[1 + 2*i..]) as f64/Q15; } }
        z.es_ok = p.has_es != 0 && hal.i2c_xfer(ES_BUS, ES_ADDR, &tx, &mut rx[..7]) == Status::Ok && rx[0] & 1 != 0;
        if z.es_ok { for i in 0..3 { z.nadir[i] = rd16(&rx[1 + 2*i..]) as f64/Q15; } }
        let mut pl = [0u8; 64];
        z.st_ok = false;
        z.st_valid = [false; MAX_HEADS];
        if p.has_st != 0 {
            if let Some(n) = self.uart_frame(hal, ST_UART, ST_SYNC2, &mut pl) {
                let nh = pl[0] as usize;
                let mut h = 0;
                while h < nh && h < MAX_HEADS && 17*(h + 1) < n {
                    let e = &pl[1 + 17*h..];
                    z.st_valid[h] = e[0] & 1 != 0;
                    for k in 0..4 { z.q_st[h][k] = rd32(&e[1 + 4*k..]) as f64/Q30; }
                    if z.st_valid[h] { z.st_ok = true; }
                    h += 1;
                }
            }
        }
        z.gps_ok = false;
        if p.has_gps != 0 {
            if let Some(n) = self.uart_frame(hal, GPS_UART, GPS_SYNC2, &mut pl) {
                if n >= 25 && pl[0] & 1 != 0 {
                    z.gps_ok = true;
                    for i in 0..3 { z.r[i] = rd32(&pl[1 + 4*i..]) as f64*0.01; z.v[i] = rd32(&pl[13 + 4*i..]) as f64*0.001; }
                }
            }
        }
        while let Ok(f) = hal.can_recv(CAN_PORT) {
            if f.id >= CAN_ROTOR_TM && f.id < CAN_ROTOR_TM + MAX_ROTORS as u32 && f.dlc >= 8 {
                let r = (f.id - CAN_ROTOR_TM) as usize;
                z.h[r] = rd32(&f.data) as f64*H_LSB;
                if r < p.nr as usize && p.rot_gi[r] > 0 { z.delta[p.rot_gi[r] as usize - 1] = rd32(&f.data[4..]) as f64*DELTA_LSB; }
            }
        }
    }
}

/// A command that is not a finite number drives nothing (as adcs_drv.c: zero, never a saturated output).
fn q15(x: f64) -> i16 {
    if !(x*Q15).is_finite() { return 0; }
    let v = (x*Q15).clamp(-Q15, Q15);
    (if v < 0.0 { v - 0.5 } else { v + 0.5 }) as i16
}

pub fn write<H: Hal>(hal: &mut H, p: &Params, m_body: &[f64; 3], cmd_r: &[f64; MAX_ROTORS], cmd_g: &[f64; MAX_GIMBALS], duty: &[f64; MAX_COUPLES]) {
    for i in 0..3 { hal.pwm_set(i as u8, q15(m_body[i]/p.m_max)); }
    for i in 0..p.nr as usize {
        let mut f = CanFrame { id: CAN_ROTOR_CMD + i as u32, extended: 0, dlc: 2, data: [0; 8] };
        f.data[..2].copy_from_slice(&q15(cmd_r[i]/p.rot_tmax[i]).to_le_bytes());
        hal.can_send(CAN_PORT, &f);
    }
    for i in 0..p.ng as usize {
        let mut f = CanFrame { id: CAN_GIMBAL_CMD + i as u32, extended: 0, dlc: 2, data: [0; 8] };
        f.data[..2].copy_from_slice(&q15(cmd_g[i]/p.gim_rate_max).to_le_bytes());
        hal.can_send(CAN_PORT, &f);
    }
    if p.nc > 0 {
        let mut f = CanFrame { id: CAN_VALVES, extended: 0, dlc: 8, data: [0; 8] };
        for i in 0..(p.nc as usize).min(MAX_COUPLES) {
            let ms = duty[i]*p.dt/VALVE_LSB_S + 0.5;
            f.data[i] = if !ms.is_finite() { 0 } else if ms > 255.0 { 255 } else if ms < 0.0 { 0 } else { ms as u8 };
        }
        hal.can_send(CAN_PORT, &f);
    }
}
