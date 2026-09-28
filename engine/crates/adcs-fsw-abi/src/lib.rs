//! The flight-software boundary of the engine (spec §9.6).
//!
//! [`Bus`] is the emulated hardware: the device emulators put bytes on it
//! (I2C register blocks, the SPI gyro response, UART frames, CAN telemetry) and
//! the flight software's writes (PWM, CAN commands) land on it. It implements
//! the `adcs_fsw::Hal` trait for the Rust flight software and the extern "C"
//! `adcs_hal_*` symbols for the C flight software, so both builds read and
//! write exactly the same bytes.
//!
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use adcs_fsw::hal::{CanFrame, Hal, Status};
use std::cell::Cell;
use std::collections::VecDeque;
use std::sync::{Mutex, MutexGuard};

pub use adcs_fsw::devices as proto;

/// The emulated buses between the device emulators and the flight software.
#[derive(Default, Clone)]
pub struct Bus {
    pub now_ns: u64,
    pub mag: Option<[u8; 7]>,
    pub gyro: Option<[u8; 13]>,
    pub sun: Option<[u8; 7]>,
    pub es: Option<[u8; 7]>,
    pub uart: [VecDeque<u8>; 3],
    pub can_rx: VecDeque<CanFrame>,
    pub can_tx: Vec<CanFrame>,
    pub pwm: [i16; 8],
    pub tm: Vec<(u16, Vec<u8>)>,
}

impl Bus {
    pub fn push_uart(&mut self, port: u8, b: &[u8]) { self.uart[port as usize].extend(b.iter().copied()); }
    pub fn push_can(&mut self, id: u32, data: [u8; 8]) { self.can_rx.push_back(CanFrame { id, extended: 0, dlc: 8, data }); }
}

impl Hal for Bus {
    fn time_ns(&mut self) -> u64 { self.now_ns }
    fn uart_read(&mut self, port: u8, buf: &mut [u8]) -> Result<usize, Status> {
        let q = self.uart.get_mut(port as usize).ok_or(Status::Arg)?;
        let n = q.len().min(buf.len());
        if n == 0 { return Err(Status::Timeout); }
        for (i, b) in q.drain(..n).enumerate() { buf[i] = b; }
        Ok(n)
    }
    fn uart_write(&mut self, _: u8, _: &[u8]) -> Status { Status::Ok }
    fn i2c_xfer(&mut self, bus: u8, addr: u8, _tx: &[u8], rx: &mut [u8]) -> Status {
        let src = match (bus, addr) {
            (proto::MAG_BUS, proto::MAG_ADDR) => self.mag,
            (proto::SUN_BUS, proto::SUN_ADDR) => self.sun,
            (proto::ES_BUS, proto::ES_ADDR) => self.es,
            _ => return Status::Arg,
        };
        match src {
            Some(s) => { let n = rx.len().min(7); rx[..n].copy_from_slice(&s[..n]); Status::Ok }
            None => Status::NoDev,
        }
    }
    fn spi_xfer(&mut self, bus: u8, cs: u8, _tx: &[u8], rx: &mut [u8]) -> Status {
        if bus != proto::GYRO_BUS || cs != proto::GYRO_CS { return Status::Arg; }
        match self.gyro { Some(g) => { let n = rx.len().min(13); rx[..n].copy_from_slice(&g[..n]); Status::Ok } None => Status::NoDev }
    }
    fn can_send(&mut self, _: u8, f: &CanFrame) -> Status { self.can_tx.push(*f); Status::Ok }
    fn can_recv(&mut self, _: u8) -> Result<CanFrame, Status> { self.can_rx.pop_front().ok_or(Status::Timeout) }
    fn pwm_set(&mut self, ch: u8, d: i16) -> Status { if (ch as usize) < 8 { self.pwm[ch as usize] = d; Status::Ok } else { Status::Arg } }
    fn tm_emit(&mut self, apid: u16, p: &[u8]) -> Status { self.tm.push((apid, p.to_vec())); Status::Ok }
}

// ---------------- extern "C" adcs_hal_* for the C flight software ----------------
thread_local! { static CUR: Cell<*mut Bus> = const { Cell::new(std::ptr::null_mut()) }; }

fn with_bus<R>(f: impl FnOnce(&mut Bus) -> R, dflt: R) -> R {
    let p = CUR.with(|c| c.get());
    if p.is_null() { dflt } else { f(unsafe { &mut *p }) }
}
fn code(s: Status) -> i32 { s as i32 }

#[no_mangle] pub extern "C" fn adcs_hal_time_ns() -> u64 { with_bus(|b| b.now_ns, 0) }
#[no_mangle] pub extern "C" fn adcs_hal_uart_write(_port: u8, _buf: *const u8, _len: usize) -> i32 { 0 }
#[no_mangle] pub unsafe extern "C" fn adcs_hal_uart_read(port: u8, buf: *mut u8, cap: usize, got: *mut usize) -> i32 {
    let s = std::slice::from_raw_parts_mut(buf, cap);
    let r = with_bus(|b| b.uart_read(port, s), Err(Status::NoDev));
    match r { Ok(n) => { *got = n; 0 } Err(e) => { *got = 0; code(e) } }
}
#[no_mangle] pub unsafe extern "C" fn adcs_hal_i2c_xfer(bus: u8, addr: u8, tx: *const u8, tx_len: usize, rx: *mut u8, rx_len: usize) -> i32 {
    let (t, r) = (std::slice::from_raw_parts(tx, tx_len), std::slice::from_raw_parts_mut(rx, rx_len));
    code(with_bus(|b| b.i2c_xfer(bus, addr, t, r), Status::NoDev))
}
#[no_mangle] pub unsafe extern "C" fn adcs_hal_spi_xfer(bus: u8, cs: u8, tx: *const u8, rx: *mut u8, len: usize) -> i32 {
    let (t, r) = (std::slice::from_raw_parts(tx, len), std::slice::from_raw_parts_mut(rx, len));
    code(with_bus(|b| b.spi_xfer(bus, cs, t, r), Status::NoDev))
}
#[no_mangle] pub unsafe extern "C" fn adcs_hal_can_send(port: u8, f: *const CanFrame) -> i32 { code(with_bus(|b| b.can_send(port, &*f), Status::NoDev)) }
#[no_mangle] pub unsafe extern "C" fn adcs_hal_can_recv(port: u8, f: *mut CanFrame) -> i32 {
    match with_bus(|b| b.can_recv(port), Err(Status::NoDev)) { Ok(x) => { *f = x; 0 } Err(e) => code(e) }
}
#[no_mangle] pub extern "C" fn adcs_hal_pwm_set(ch: u8, d: i16) -> i32 { code(with_bus(|b| b.pwm_set(ch, d), Status::NoDev)) }
#[no_mangle] pub extern "C" fn adcs_hal_gpio_write(_pin: u16, _l: u8) -> i32 { 0 }
#[no_mangle] pub unsafe extern "C" fn adcs_hal_gpio_read(_pin: u16, l: *mut u8) -> i32 { *l = 0; 0 }
#[no_mangle] pub unsafe extern "C" fn adcs_hal_adc_read(_ch: u8, raw: *mut u16) -> i32 { *raw = 0; 0 }
#[no_mangle] pub unsafe extern "C" fn adcs_hal_tm_emit(apid: u16, p: *const u8, n: usize) -> i32 {
    let s = std::slice::from_raw_parts(p, n);
    code(with_bus(|b| b.tm_emit(apid, s), Status::NoDev))
}

// ---------------- the C flight software ----------------
#[repr(C)]
struct CInit { abi_version: u32, config_blob: *const u8, config_len: usize, start_ns: u64 }

extern "C" {
    fn adcs_fsw_init(init: *const CInit) -> i32;
    fn adcs_fsw_step(now_ns: u64) -> i32;
    fn adcs_fsw_command(tc: *const u8, len: usize) -> i32;
    fn adcs_fsw_peek(out: *mut adcs_fsw::State) -> i32;
    fn adcs_fsw_build_id() -> *const std::os::raw::c_char;
    fn adcs_fsw_debug(out: *mut f64, n: i32) -> i32;
}

/// Which flight software runs behind the bus.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Impl { C, Rust }

impl std::str::FromStr for Impl {
    type Err = String;
    fn from_str(s: &str) -> Result<Impl, String> {
        match s { "c" | "C" => Ok(Impl::C), "rust" | "rs" | "Rust" => Ok(Impl::Rust), _ => Err(format!("unknown flight software '{s}' (c | rust)")) }
    }
}

/// One flight-software instance. The C build keeps its state in statics, so one
/// C instance exists per process at a time (a lock enforces it; spec §9.6).
pub enum Fsw {
    C(MutexGuard<'static, ()>),
    Rust(Box<adcs_fsw::Fsw>),
}

static C_LOCK: Mutex<()> = Mutex::new(());

pub const DEBUG_LEN: usize = 41;

impl Fsw {
    pub fn init(which: Impl, blob: &[u8], start_ns: u64, bus: &mut Bus) -> Result<Fsw, String> {
        match which {
            Impl::C => {
                let g = C_LOCK.lock().unwrap_or_else(|e| e.into_inner());
                let a = CInit { abi_version: adcs_fsw::ABI_VERSION, config_blob: blob.as_ptr(), config_len: blob.len(), start_ns };
                let rc = Self::with(bus, || unsafe { adcs_fsw_init(&a) });
                if rc != 0 { return Err(format!("adcs_fsw_init (C) refused the configuration: {rc}")); }
                Ok(Fsw::C(g))
            }
            Impl::Rust => {
                let mut f = Box::new(adcs_fsw::Fsw::new());
                f.init(adcs_fsw::ABI_VERSION, blob, start_ns).map_err(|e| format!("adcs-fsw (Rust) refused the configuration: {e:?}"))?;
                Ok(Fsw::Rust(f))
            }
        }
    }
    fn with<R>(bus: &mut Bus, f: impl FnOnce() -> R) -> R {
        CUR.with(|c| c.set(bus as *mut Bus));
        let r = f();
        CUR.with(|c| c.set(std::ptr::null_mut()));
        r
    }
    pub fn step(&mut self, bus: &mut Bus, now_ns: u64) -> i32 {
        match self {
            Fsw::C(_) => Self::with(bus, || unsafe { adcs_fsw_step(now_ns) }),
            Fsw::Rust(f) => f.step(bus, now_ns),
        }
    }
    pub fn command(&mut self, tc: &[u8]) -> i32 {
        match self { Fsw::C(_) => unsafe { adcs_fsw_command(tc.as_ptr(), tc.len()) }, Fsw::Rust(f) => f.command(tc) }
    }
    pub fn peek(&self) -> Option<adcs_fsw::State> {
        match self {
            Fsw::C(_) => { let mut s = adcs_fsw::State::default(); (unsafe { adcs_fsw_peek(&mut s) } == 0).then_some(s) }
            Fsw::Rust(f) => f.peek(),
        }
    }
    /// t, mode, ad_ok, q[4], b[3], w_est[3], q_ref[4], tau_req[3], m[3], cmd_r[8], cmd_g[4], duty[6]
    pub fn debug(&self) -> [f64; DEBUG_LEN] {
        let mut v = [0.0; DEBUG_LEN];
        match self { Fsw::C(_) => { unsafe { adcs_fsw_debug(v.as_mut_ptr(), DEBUG_LEN as i32) }; } Fsw::Rust(f) => { f.debug(&mut v); } }
        v
    }
    pub fn build_id(&self) -> String {
        match self {
            Fsw::C(_) => unsafe { std::ffi::CStr::from_ptr(adcs_fsw_build_id()) }.to_string_lossy().into_owned(),
            Fsw::Rust(_) => adcs_fsw::BUILD_ID.to_string(),
        }
    }
}
