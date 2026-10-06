//! Drivers of the synthetic parts over the byte HAL (fsw/pseudocode/09); twin of adcs_drv.c.
//! The decoding and the command words are the design's (drivers::drv_read, drv_write, crc16); what
//! stays here is the runtime around them: the bus transfers, the UART frame assembly that keeps a
//! frame not yet whole for the next read, and the last reading held when a read brings none.
use crate::alg::drivers as alg;
use crate::devices::*;
use crate::hal::{CanFrame, Hal, Status};
use crate::math::ints;
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

/// The longest payload the design's CRC and frame search take [bytes].
const PAYLOAD_MAX: usize = 64;

/// CRC-16/CCITT of at most PAYLOAD_MAX bytes.
/// Written from the design: drivers::crc16 (src/alg).
pub fn crc16(p: &[u8]) -> u16 {
    assert!(p.len() <= PAYLOAD_MAX, "crc16 takes at most 64 bytes");
    let mut b = [0i64; PAYLOAD_MAX];
    for (x, &y) in b.iter_mut().zip(p) { *x = y as i64; }
    alg::crc16(b, p.len() as i64) as u16
}

const RXCAP: usize = 256;
/// The stream the design's frame search takes: one frame: a payload of at most PAYLOAD_MAX
/// bytes and its five framing bytes, with room to spare.
const STREAM: usize = 96;

/// Frame assembly buffers, one per UART port.
#[derive(Clone, Copy)]
pub struct Drv { rx: [[u8; RXCAP]; 3], len: [usize; 3] }

impl Default for Drv { fn default() -> Self { Drv { rx: [[0; RXCAP]; 3], len: [0; 3] } } }

/// No bytes on a port: the design's frame search finds no frame.
const NO_STREAM: ([i64; STREAM], i64) = ([0; STREAM], 0);

/// The CAN frames the design's read takes at a time.
const CAN_BATCH: usize = 8;

impl Drv {
    pub fn reset(&mut self) { self.len = [0; 3]; }

    /// Pull available bytes; the newest complete, CRC-good frame with sync2, as the stream of that one
    /// frame (EB sync2 n payload CRC) the design's frame search reads. The bytes of a frame not yet
    /// whole stay in the buffer for the next read.
    fn uart_frame<H: Hal>(&mut self, hal: &mut H, port: u8, sync2: u8) -> ([i64; STREAM], i64) {
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
        let mut found = NO_STREAM;
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
            if n <= PAYLOAD_MAX && crc16(&b[s + 3..s + 3 + n]) == crc {
                let mut f = [0i64; STREAM];
                for (x, &y) in f.iter_mut().zip(&b[s..s + 5 + n]) { *x = y as i64; }
                found = (f, (5 + n) as i64);
            }
            b.copy_within(s + 5 + n..*len, 0);
            *len -= s + 5 + n;
        }
        found
    }

    /// A read of every sensor: the bus transfers here, the decoding the design's. A reading the read
    /// does not bring (no transfer, status bit clear, no frame) keeps its last value, its flag false.
    /// Written from the design: drivers::drv_read (src/alg).
    pub fn read<H: Hal>(&mut self, hal: &mut H, p: &Params, z: &mut Meas) {
        let tx = [0u8; 1];
        // a transfer that fails brings no reading: its status byte is cleared
        let mut mag = [0u8; 7];
        if hal.i2c_xfer(MAG_BUS, MAG_ADDR, &tx, &mut mag) != Status::Ok { mag[0] = 0; }
        let mut gyro = [0u8; 13];
        if p.has_gyro != 0 {
            let gtx = [GYRO_CMD, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
            if hal.spi_xfer(GYRO_BUS, GYRO_CS, &gtx, &mut gyro) != Status::Ok { gyro[0] = 0; }
        }
        let mut sun = [0u8; 7];
        if p.has_sun != 0 && hal.i2c_xfer(SUN_BUS, SUN_ADDR, &tx, &mut sun) != Status::Ok { sun[0] = 0; }
        let mut es = [0u8; 7];
        if p.has_es != 0 && hal.i2c_xfer(ES_BUS, ES_ADDR, &tx, &mut es) != Status::Ok { es[0] = 0; }
        let (st, st_len) = if p.has_st != 0 { self.uart_frame(hal, ST_UART, ST_SYNC2) } else { NO_STREAM };
        let (gps, gps_len) = if p.has_gps != 0 { self.uart_frame(hal, GPS_UART, GPS_SYNC2) } else { NO_STREAM };
        let mut can = [CanFrame::default(); CAN_BATCH];
        let mut n_can = can_batch(hal, &mut can);

        let (has_gyro, has_sun, has_es, has_st, has_gps) = (p.has_gyro != 0, p.has_sun != 0, p.has_es != 0, p.has_st != 0, p.has_gps != 0);
        let (ids, dlc, data) = can_words(&can[..n_can], false);
        let (mag_ok, b, gyro_ok, w, sun_ok, s, es_ok, nadir, st_ok, st_valid, q_st, gps_ok, r, v, h, delta) =
            alg::drv_read(ints(&mag), ints(&gyro), ints(&sun), ints(&es), st, st_len, gps, gps_len, ids, dlc, data, n_can as i64,
                          has_gyro, has_sun, has_es, has_st, has_gps, p.nr as i64, ints(&p.rot_gi));
        z.mag_ok = mag_ok;
        if mag_ok { z.b = b; }
        z.gyro_ok = gyro_ok;
        if gyro_ok { z.w = w; }
        z.sun_ok = sun_ok;
        if sun_ok { z.sun = s; }
        z.es_ok = es_ok;
        if es_ok { z.nadir = nadir; }
        z.st_ok = st_ok;
        z.st_valid = st_valid;
        if st_len > 0 { z.q_st = q_st; }
        z.gps_ok = gps_ok;
        if gps_ok { z.r = r; z.v = v; }
        // the momentum devices' telemetry: every frame waiting, CAN_BATCH at a time in the order they came
        hold_telemetry(p, z, &can[..n_can], h, delta);
        while n_can == CAN_BATCH {
            n_can = can_batch(hal, &mut can);
            let (h, delta) = telemetry(p, &can[..n_can], false);
            hold_telemetry(p, z, &can[..n_can], h, delta);
        }
    }
}

/// The CAN frames waiting, at most CAN_BATCH.
fn can_batch<H: Hal>(hal: &mut H, can: &mut [CanFrame; CAN_BATCH]) -> usize {
    let mut n = 0;
    while n < CAN_BATCH { match hal.can_recv(CAN_PORT) { Ok(f) => { can[n] = f; n += 1; } Err(_) => break } }
    n
}

/// The momentum and gimbal angle the design's read decodes from CAN frames alone.
fn telemetry(p: &Params, f: &[CanFrame], invert: bool) -> ([f64; MAX_ROTORS], [f64; MAX_GIMBALS]) {
    let (ids, dlc, data) = can_words(f, invert);
    let (.., h, delta) = alg::drv_read([0; 7], [0; 13], [0; 7], [0; 7], NO_STREAM.0, 0, NO_STREAM.0, 0, ids, dlc, data, f.len() as i64,
                                       false, false, false, false, false, p.nr as i64, ints(&p.rot_gi));
    (h, delta)
}

/// The values frames f carried (h, delta as decoded) replace the held ones; the rest are held. Which
/// they carried: the same frames with every data byte inverted decode each carried value to another,
/// so a carried value is not 0 in both reads (a value not carried is 0 in both).
fn hold_telemetry(p: &Params, z: &mut Meas, f: &[CanFrame], h: [f64; MAX_ROTORS], delta: [f64; MAX_GIMBALS]) {
    if f.is_empty() { return; }
    let (hi, di) = telemetry(p, f, true);
    for i in 0..MAX_ROTORS { if h[i] != 0.0 || hi[i] != 0.0 { z.h[i] = h[i]; } }
    for j in 0..MAX_GIMBALS { if delta[j] != 0.0 || di[j] != 0.0 { z.delta[j] = delta[j]; } }
}

/// CAN frames as the design's ids, lengths and data bytes (inverted when asked).
fn can_words(f: &[CanFrame], invert: bool) -> ([i64; CAN_BATCH], [i64; CAN_BATCH], [[i64; 8]; CAN_BATCH]) {
    let (mut ids, mut dlc, mut data) = ([0i64; CAN_BATCH], [0i64; CAN_BATCH], [[0i64; 8]; CAN_BATCH]);
    for (k, x) in f.iter().enumerate() {
        ids[k] = x.id as i64;
        dlc[k] = x.dlc as i64;
        for i in 0..8 { data[k][i] = (if invert { !x.data[i] } else { x.data[i] }) as i64; }
    }
    (ids, dlc, data)
}

/// The command words on the buses: coils' PWM, rotor and gimbal words (CAN), valve on-times.
/// Written from the design: drivers::drv_write (src/alg).
pub fn write<H: Hal>(hal: &mut H, p: &Params, m_body: &[f64; 3], cmd_r: &[f64; MAX_ROTORS], cmd_g: &[f64; MAX_GIMBALS], duty: &[f64; MAX_COUPLES]) {
    let nc = (p.nc as usize).min(MAX_COUPLES);
    let (pwm, rot, gim, valves) = alg::drv_write(*m_body, p.m_max, *cmd_r, p.rot_tmax, p.nr as i64, *cmd_g, p.gim_rate_max, p.ng as i64,
                                                 *duty, nc as i64, p.dt);
    for i in 0..3 { hal.pwm_set(i as u8, pwm[i] as i16); }
    for i in 0..p.nr as usize {
        let mut f = CanFrame { id: CAN_ROTOR_CMD + i as u32, extended: 0, dlc: 2, data: [0; 8] };
        f.data[..2].copy_from_slice(&(rot[i] as i16).to_le_bytes());
        hal.can_send(CAN_PORT, &f);
    }
    for i in 0..p.ng as usize {
        let mut f = CanFrame { id: CAN_GIMBAL_CMD + i as u32, extended: 0, dlc: 2, data: [0; 8] };
        f.data[..2].copy_from_slice(&(gim[i] as i16).to_le_bytes());
        hal.can_send(CAN_PORT, &f);
    }
    if p.nc > 0 {
        let mut f = CanFrame { id: CAN_VALVES, extended: 0, dlc: 8, data: [0; 8] };
        for i in 0..nc { f.data[i] = valves[i] as u8; }
        hal.can_send(CAN_PORT, &f);
    }
}
