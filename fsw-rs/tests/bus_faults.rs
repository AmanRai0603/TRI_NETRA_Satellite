//! Bus faults at the HAL (docs/S11_INVENTORY.md O11, step S11.14): one test per fault point of the device map
//! (fsw/devices/example_3u.toml: the magnetometer, Sun sensor and Earth sensor on I2C, the gyro on SPI, the star tracker and the GNSS
//! receiver on UART, the coils on PWM, the momentum devices on CAN) and per bus fault (a NACK, a CRC wrong, a register stuck, a
//! supply drop, a CAN error frame, a write that fails). Each asks whether the flight driver raises its fault flag. Where one does
//! (the magnetometer's and the gyro's stale flags, a rotor's isolation flag) the test runs; where the drivers raise none (the Sun
//! sensor, the Earth sensor, the star tracker, the GNSS receiver, the coils' and the CAN's writes) the test is `#[ignore]`d as a
//! FINDING of S11.14: `cargo test --test bus_faults -- --ignored` shows what each says today, and the step that gives the driver
//! its flag removes the `#[ignore]` (and the name from the C list of fsw/tests/test_fsw.c). Their scenario twins are
//! `fault_bus_<device>_<kind>`, flown by the engine and the twin on the same bytes.
//! The scene is the tick tests': exact sensors, a held attitude (the test is of the drivers' flags, not of the estimate).
#![allow(clippy::needless_range_loop, clippy::field_reassign_with_default)]
use adcs_fsw::devices::*;
use adcs_fsw::drv::crc16;
use adcs_fsw::hal::{CanFrame, Hal, Status};
use adcs_fsw::math::*;
use adcs_fsw::{env, Fsw, Mode, Params, ABI_VERSION};

#[derive(Clone, Copy, PartialEq, Debug)]
enum Dev { Mag, Sun, Es, St, Gyro, Gnss, Coils, Can }
#[derive(Clone, Copy, PartialEq, Debug)]
enum Kind { Nack, Crc, Stuck, Brownout, CanError }

/// The in-memory HAL with the rig's bus fault at one device (from `from_s` of the run's time on).
struct Stub { now_ns: u64, mag: [u8; 7], gyro: [u8; 13], sun: [u8; 7], es: [u8; 7], uart: [Vec<u8>; 3], can_rx: Vec<CanFrame>, pwm: [i16; 8],
              fault: Option<(Dev, Kind)>, on: bool, last: std::collections::HashMap<u8, Vec<u8>>, write_status: Status, writes_failed: u32 }
impl Stub {
    fn new() -> Stub {
        Stub { now_ns: 0, mag: [0; 7], gyro: [0; 13], sun: [0; 7], es: [0; 7], uart: Default::default(), can_rx: vec![], pwm: [0; 8], fault: None, on: false,
               last: Default::default(), write_status: Status::Ok, writes_failed: 0 }
    }
    fn hit(&self, d: Dev, k: &[Kind]) -> bool { self.on && self.fault.map(|(fd, fk)| fd == d && k.contains(&fk)).unwrap_or(false) }
    /// A register block as the bus delivers it: not at all (a NACK, no supply), or the last good one again (stuck).
    fn block(&mut self, d: Dev, id: u8, fresh: &[u8], rx: &mut [u8]) -> Status {
        if self.hit(d, &[Kind::Nack, Kind::Brownout]) { return Status::NoDev; }
        let held = self.last.entry(id).or_insert_with(|| fresh.to_vec()).clone();
        let src = if self.hit(d, &[Kind::Stuck]) { held } else { self.last.insert(id, fresh.to_vec()); fresh.to_vec() };
        let n = rx.len().min(src.len());
        rx[..n].copy_from_slice(&src[..n]);
        Status::Ok
    }
}
impl Hal for Stub {
    fn time_ns(&mut self) -> u64 { self.now_ns }
    fn uart_read(&mut self, port: u8, buf: &mut [u8]) -> Result<usize, Status> {
        let d = if port == ST_UART { Dev::St } else { Dev::Gnss };
        let mut q = std::mem::take(&mut self.uart[port as usize]);
        if self.hit(d, &[Kind::Nack, Kind::Brownout]) { q.clear(); }
        if self.hit(d, &[Kind::Crc]) && !q.is_empty() { let n = q.len(); q[n - 1] ^= 1; }
        if !q.is_empty() {
            if self.hit(d, &[Kind::Stuck]) { q = self.last.get(&(10 + port)).cloned().unwrap_or(q); } else { self.last.insert(10 + port, q.clone()); }
        }
        let n = q.len().min(buf.len());
        if n == 0 { return Err(Status::Timeout); }
        buf[..n].copy_from_slice(&q[..n]);
        q.drain(..n);
        self.uart[port as usize] = q;
        Ok(n)
    }
    fn uart_write(&mut self, _: u8, _: &[u8]) -> Status { Status::Ok }
    fn i2c_xfer(&mut self, bus: u8, addr: u8, _tx: &[u8], rx: &mut [u8]) -> Status {
        let (d, id, fresh) = if bus == MAG_BUS && addr == MAG_ADDR { (Dev::Mag, 1, self.mag.to_vec()) }
            else if bus == SUN_BUS && addr == SUN_ADDR { (Dev::Sun, 2, self.sun.to_vec()) }
            else if bus == ES_BUS && addr == ES_ADDR { (Dev::Es, 3, self.es.to_vec()) } else { return Status::NoDev };
        self.block(d, id, &fresh, rx)
    }
    fn spi_xfer(&mut self, bus: u8, cs: u8, _tx: &[u8], rx: &mut [u8]) -> Status {
        if bus != GYRO_BUS || cs != GYRO_CS { return Status::NoDev; }
        let fresh = self.gyro.to_vec();
        self.block(Dev::Gyro, 4, &fresh, rx)
    }
    fn can_send(&mut self, _: u8, _: &CanFrame) -> Status {
        if self.hit(Dev::Can, &[Kind::Nack, Kind::CanError]) { self.writes_failed += 1; return Status::Bus; }
        Status::Ok
    }
    fn can_recv(&mut self, _: u8) -> Result<CanFrame, Status> {
        if self.hit(Dev::Can, &[Kind::CanError]) { return Err(Status::Bus); }
        if self.hit(Dev::Can, &[Kind::Nack, Kind::Brownout]) { self.can_rx.clear(); }
        if self.can_rx.is_empty() { Err(Status::Timeout) } else { Ok(self.can_rx.remove(0)) }
    }
    fn pwm_set(&mut self, ch: u8, d: i16) -> Status {
        if self.hit(Dev::Coils, &[Kind::Nack]) { self.writes_failed += 1; return Status::Bus; }
        if ch < 8 { self.pwm[ch as usize] = d; }
        self.write_status
    }
}

fn put16(b: &mut [u8], v: f64) { b[..2].copy_from_slice(&((if v < 0.0 { v - 0.5 } else { v + 0.5 }) as i16).to_le_bytes()); }
fn put32(b: &mut [u8], v: f64) { b[..4].copy_from_slice(&((if v < 0.0 { v - 0.5 } else { v + 0.5 }) as i32).to_le_bytes()); }
fn frame(sync2: u8, pl: &[u8]) -> Vec<u8> {
    let mut fr = vec![ST_SYNC1, sync2, pl.len() as u8];
    fr.extend_from_slice(pl);
    fr.extend_from_slice(&crc16(pl).to_le_bytes());
    fr
}

const JD0: f64 = 2461407.25;
const ADCS_Q15: f64 = 32767.0;
/// The run: 150 s of nadir-by-coils with a Sun sensor, an Earth sensor, a star tracker and a GNSS receiver and three wheels' telemetry, the
/// fault in force from 10 s, the scene exact (a circular orbit, the field and the Sun from the models, a gyro at rest).
/// Returns the flight software's fault word at the end and how many output writes failed.
fn fly(fault: Option<(Dev, Kind)>) -> (u16, u32) {
    let mut p = Params::default();
    p.jd0 = JD0; p.dt = 0.1; p.mu = 3.986004418e14; p.start_mode = Mode::NadirMtq as u8; p.auto_next = 255;
    p.bdot_law = 0; p.mtq_period = 1.0; p.mtq_meas = 0.2; p.m_max = 0.2; p.bdot_k = 1e-3; p.has_gyro = 1;
    p.J[0][0] = 0.0067; p.J[1][1] = 0.042; p.J[2][2] = 0.042; p.igrf_nmax = 10; p.rate_lpf_s = 0.3;
    p.ss_eclipse = 1; p.gd_T = 1.0; p.mtq_phi = 0.01; p.rw_phi = 0.01; p.rw_dt = 0.1; p.fdir_s = 3.0; p.fdir_win_s = 120.0; p.fdir_h_frac = 0.005;
    p.rcsd_T_damp_s = 20.0; p.rcsd_period_s = 1.0; p.st_coast_s = 900.0; p.mekf_sig_mag = 0.01; p.mekf_sig_sun = 0.005; p.mekf_meas_scale = 1.0;
    p.clock_step_max_s = 60.0; p.gyro_stale_ticks = 3; p.st_norm_tol = 1e-3; p.triad_sin_min = 0.2;
    p.has_sun = 1; p.has_gps = 1; p.has_es = 1; p.has_st = 1; p.gnss_ecef = 0; p.gyro_arw = 1e-6; p.gyro_rrw = 1e-8;
    p.mtq_Kp = [1e-3; 3]; p.mtq_Kd = [1e-2; 3];
    let mut b = vec![0u8; adcs_fsw::params::BLOB_SIZE];
    p.encode(&mut b);
    let (mut hal, mut f) = (Stub::new(), Fsw::new());
    hal.fault = fault;
    f.init(ABI_VERSION, &b, 0).expect("init");
    let (q, gh) = (qnorm(&[0.2, -0.1, 0.4, 0.8]), env::igrf_gh(env::decyear(JD0)));
    let a = dcm(&q);
    for k in 0..1500u64 {
        let t = k as f64*0.1;
        hal.on = t >= 10.0;
        let (n, rr, inc) = (0.0011f64, 6.9e6f64, 97.0f64.to_radians());
        let u = n*t;
        let r = [rr*u.cos(), rr*u.sin()*inc.cos(), rr*u.sin()*inc.sin()];
        let v = [-rr*n*u.sin(), rr*n*u.cos()*inc.cos(), rr*n*u.cos()*inc.sin()];
        let jd = JD0 + t/86400.0;
        let bf = mat3_vec(&a, &env::field_eci(&r, jd, &gh, 10));
        let s = mat3_vec(&a, &env::sun_model(jd));
        hal.mag[0] = 1; for i in 0..3 { put16(&mut hal.mag[1 + 2*i..], bf[i]/MAG_LSB_T); }
        hal.sun[0] = 1; for i in 0..3 { put16(&mut hal.sun[1 + 2*i..], s[i]*ADCS_Q15); }
        let nadir = mat3_vec(&a, &[-r[0]/rr, -r[1]/rr, -r[2]/rr]);
        hal.es[0] = 1; for i in 0..3 { put16(&mut hal.es[1 + 2*i..], nadir[i]*ADCS_Q15); }
        hal.gyro[0] = 1; for i in 0..3 { put32(&mut hal.gyro[1 + 4*i..], ((k*7 + i as u64*3) % 5) as f64); }
        if k % 10 == 0 {
            let mut pl = [0u8; 33];
            pl[0] = 1;
            for i in 0..3 { put32(&mut pl[1 + 4*i..], r[i]*100.0); put32(&mut pl[13 + 4*i..], v[i]*1000.0); }
            let ds = (jd - 2451545.0)*86400.0;
            put32(&mut pl[25..], ds.floor()); put32(&mut pl[29..], (ds - ds.floor())*1e6);
            hal.uart[GPS_UART as usize].extend_from_slice(&frame(GPS_SYNC2, &pl));
            let mut sp = [0u8; 18];
            sp[0] = 1; sp[1] = 1;
            for i in 0..4 { put32(&mut sp[2 + 4*i..], q[i]*ADCS_Q30); }
            hal.uart[ST_UART as usize].extend_from_slice(&frame(ST_SYNC2, &sp));
        }
        for i in 0..3u32 {
            let mut d = [0u8; 8];
            put32(&mut d[..4], 1e-3/1e-9*f64::from(i + 1));
            hal.can_rx.push(CanFrame { id: CAN_ROTOR_TM + i, extended: 0, dlc: 8, data: d });
        }
        hal.now_ns = k*100_000_000;
        let now = hal.now_ns;
        f.step(&mut hal, now);
    }
    (f.peek().unwrap().faults, hal.writes_failed)
}
const ADCS_Q30: f64 = 1073741824.0;

fn raised(fault: (Dev, Kind), mask: u16) -> bool {
    let clean = fly(None).0;
    assert_eq!(clean & mask, 0, "the scene itself raises a flag of {mask:#x} (word {clean:#x}): the rig is not clean");
    fly(Some(fault)).0 & mask != 0
}
const ANY: u16 = 0xFFFF;
const MAG_STALE: u16 = adcs_fsw::fsw::FAULT_MAG_STALE;
const GYRO: u16 = adcs_fsw::fsw::FAULT_GYRO_STALE | adcs_fsw::fsw::FAULT_GYRO_QUIET;
const ROTOR: u16 = 0b111;

macro_rules! point {
    ($name:ident, $dev:expr, $kind:expr, $mask:expr) => {
        #[test]
        fn $name() { assert!(raised(($dev, $kind), $mask), "{:?} {:?}: the flight software raised no fault flag of {:#x}", $dev, $kind, $mask); }
    };
    ($name:ident, $dev:expr, $kind:expr, $mask:expr, ignore $why:expr) => {
        #[test]
        #[ignore = $why]
        fn $name() { assert!(raised(($dev, $kind), $mask), "{:?} {:?}: the flight software raised no fault flag of {:#x}", $dev, $kind, $mask); }
    };
}

// magnetometer (I2C bus 0): a NACK and a lost supply go stale after 60 s; a register stuck at its last value is no fault to the driver
point!(mag_nack, Dev::Mag, Kind::Nack, MAG_STALE);
point!(mag_brownout, Dev::Mag, Kind::Brownout, MAG_STALE);
point!(mag_stuck, Dev::Mag, Kind::Stuck, MAG_STALE, ignore "S11.14 finding: a magnetometer repeating its last field is not flagged (only silence is)");
// gyro (SPI bus 0 CS 0): silence goes stale, a repeated sample is the quiet flag
point!(gyro_nack, Dev::Gyro, Kind::Nack, GYRO);
point!(gyro_brownout, Dev::Gyro, Kind::Brownout, GYRO);
point!(gyro_stuck, Dev::Gyro, Kind::Stuck, GYRO);
// Sun sensor and Earth sensor (I2C bus 1), star tracker and GNSS receiver (UART 1, 2): the drivers keep the flag of the read (ok false) and raise none
point!(sun_nack, Dev::Sun, Kind::Nack, ANY, ignore "S11.14 finding: no driver flag exists for this fault point");
point!(sun_stuck, Dev::Sun, Kind::Stuck, ANY, ignore "S11.14 finding: no driver flag exists for this fault point");
point!(sun_brownout, Dev::Sun, Kind::Brownout, ANY, ignore "S11.14 finding: no driver flag exists for this fault point");
point!(es_nack, Dev::Es, Kind::Nack, ANY, ignore "S11.14 finding: no driver flag exists for this fault point");
point!(es_stuck, Dev::Es, Kind::Stuck, ANY, ignore "S11.14 finding: no driver flag exists for this fault point");
point!(es_brownout, Dev::Es, Kind::Brownout, ANY, ignore "S11.14 finding: no driver flag exists for this fault point");
point!(st_nack, Dev::St, Kind::Nack, ANY, ignore "S11.14 finding: no driver flag exists for this fault point");
point!(st_crc, Dev::St, Kind::Crc, ANY, ignore "S11.14 finding: no driver flag exists for this fault point");
point!(st_stuck, Dev::St, Kind::Stuck, ANY, ignore "S11.14 finding: no driver flag exists for this fault point");
point!(st_brownout, Dev::St, Kind::Brownout, ANY, ignore "S11.14 finding: no driver flag exists for this fault point");
point!(gnss_nack, Dev::Gnss, Kind::Nack, ANY, ignore "S11.14 finding: no driver flag exists for this fault point");
point!(gnss_crc, Dev::Gnss, Kind::Crc, ANY, ignore "S11.14 finding: no driver flag exists for this fault point");
point!(gnss_stuck, Dev::Gnss, Kind::Stuck, ANY, ignore "S11.14 finding: no driver flag exists for this fault point");
point!(gnss_brownout, Dev::Gnss, Kind::Brownout, ANY, ignore "S11.14 finding: no driver flag exists for this fault point");
// momentum devices (CAN 0): silent telemetry or an error frame isolates a rotor only in a momentum state, which this run (coils) is not
point!(can_nack, Dev::Can, Kind::Nack, ROTOR, ignore "S11.14 finding: silent rotor telemetry raises no flag in a state without momentum devices; see the engine's fault_bus_mex_nack");
point!(can_error, Dev::Can, Kind::CanError, ROTOR, ignore "S11.14 finding: a CAN error frame raises no flag in a state without momentum devices; see the engine's fault_bus_mex_can_error");
// the output side: a write the bus refuses (PWM, CAN) is not looked at by the drivers (TS4: 'PWM write error: fault raised')
point!(pwm_write_error, Dev::Coils, Kind::Nack, ANY, ignore "S11.14 finding (TS4): a PWM write that fails raises no fault flag; the design's driver does not check the write's status");
point!(can_write_error, Dev::Can, Kind::Nack, ANY, ignore "S11.14 finding: a CAN send that fails raises no fault flag; the design's driver does not check the send's status");

/// The rig itself: with no fault the scene raises no flag, and a failing write is a status the driver gets (so that the findings above are of the driver).
#[test]
fn the_rig_is_clean_and_its_write_faults_reach_the_hal() {
    assert_eq!(fly(None), (0, 0));
    let (_, failed) = fly(Some((Dev::Coils, Kind::Nack)));
    assert!(failed > 1000, "the coils' writes failed {failed} times: the fault does not reach the HAL");
}
