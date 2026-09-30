//! adcs_fsw.h exported over extern "C" adcs_hal_* (feature `cabi`): the same
//! symbols as libadcs_fsw.a, so a host links either implementation unchanged.
use crate::fsw::{Fsw, State};
use crate::hal::{CanFrame, Hal, Status};
use core::cell::UnsafeCell;

extern "C" {
    fn adcs_hal_time_ns() -> u64;
    fn adcs_hal_uart_write(port: u8, buf: *const u8, len: usize) -> i32;
    fn adcs_hal_uart_read(port: u8, buf: *mut u8, cap: usize, got: *mut usize) -> i32;
    fn adcs_hal_i2c_xfer(bus: u8, addr7: u8, tx: *const u8, tx_len: usize, rx: *mut u8, rx_len: usize) -> i32;
    fn adcs_hal_spi_xfer(bus: u8, cs: u8, tx: *const u8, rx: *mut u8, len: usize) -> i32;
    fn adcs_hal_can_send(port: u8, f: *const CanFrame) -> i32;
    fn adcs_hal_can_recv(port: u8, f: *mut CanFrame) -> i32;
    fn adcs_hal_pwm_set(ch: u8, duty: i16) -> i32;
    fn adcs_hal_tm_emit(apid: u16, p: *const u8, n: usize) -> i32;
}

fn st(s: i32) -> Status {
    match s { 0 => Status::Ok, -1 => Status::Timeout, -2 => Status::Bus, -3 => Status::Arg, -4 => Status::NoDev, _ => Status::Busy }
}

/// The platform HAL behind the C symbols.
pub struct ExternHal;

impl Hal for ExternHal {
    fn time_ns(&mut self) -> u64 { unsafe { adcs_hal_time_ns() } }
    fn uart_read(&mut self, port: u8, buf: &mut [u8]) -> Result<usize, Status> {
        let mut got = 0usize;
        match st(unsafe { adcs_hal_uart_read(port, buf.as_mut_ptr(), buf.len(), &mut got) }) {
            Status::Ok => Ok(got),
            e => Err(e),
        }
    }
    fn uart_write(&mut self, port: u8, buf: &[u8]) -> Status { st(unsafe { adcs_hal_uart_write(port, buf.as_ptr(), buf.len()) }) }
    fn i2c_xfer(&mut self, bus: u8, addr7: u8, tx: &[u8], rx: &mut [u8]) -> Status {
        st(unsafe { adcs_hal_i2c_xfer(bus, addr7, tx.as_ptr(), tx.len(), rx.as_mut_ptr(), rx.len()) })
    }
    fn spi_xfer(&mut self, bus: u8, cs: u8, tx: &[u8], rx: &mut [u8]) -> Status {
        st(unsafe { adcs_hal_spi_xfer(bus, cs, tx.as_ptr(), rx.as_mut_ptr(), rx.len().min(tx.len())) })
    }
    fn can_send(&mut self, port: u8, f: &CanFrame) -> Status { st(unsafe { adcs_hal_can_send(port, f) }) }
    fn can_recv(&mut self, port: u8) -> Result<CanFrame, Status> {
        let mut f = CanFrame::default();
        match st(unsafe { adcs_hal_can_recv(port, &mut f) }) { Status::Ok => Ok(f), e => Err(e) }
    }
    fn pwm_set(&mut self, ch: u8, duty: i16) -> Status { st(unsafe { adcs_hal_pwm_set(ch, duty) }) }
    fn tm_emit(&mut self, apid: u16, p: &[u8]) -> Status { st(unsafe { adcs_hal_tm_emit(apid, p.as_ptr(), p.len()) }) }
}

/// The single flight-software instance (the C build keeps one static struct too).
struct Cell(UnsafeCell<Option<Fsw>>);
// SAFETY: adcs_fsw.h is single-threaded by contract (one caller, the scheduler).
unsafe impl Sync for Cell {}
static FSW: Cell = Cell(UnsafeCell::new(None));

#[allow(clippy::mut_from_ref)]
fn fsw() -> &'static mut Option<Fsw> { unsafe { &mut *FSW.0.get() } }

#[repr(C)]
pub struct InitArgs { pub abi_version: u32, pub config_blob: *const u8, pub config_len: usize, pub start_ns: u64 }

#[no_mangle]
pub extern "C" fn adcs_fsw_init(init: *const InitArgs) -> i32 {
    let slot = fsw();
    *slot = Some(Fsw::new());
    if init.is_null() { return -10; }
    let a = unsafe { &*init };
    if a.config_blob.is_null() { return -11; }
    let blob = unsafe { core::slice::from_raw_parts(a.config_blob, a.config_len) };
    match slot.as_mut().unwrap().init(a.abi_version, blob, a.start_ns) {
        Ok(()) => 0,
        Err(crate::fsw::InitError::Abi) => -10,
        Err(crate::fsw::InitError::Config) => -11,
        Err(crate::fsw::InitError::Invalid(_)) => -12,
    }
}

#[no_mangle]
pub extern "C" fn adcs_fsw_step(now_ns: u64) -> i32 {
    match fsw() { Some(f) => f.step(&mut ExternHal, now_ns), None => -1 }
}

#[no_mangle]
pub extern "C" fn adcs_fsw_command(tc: *const u8, len: usize) -> i32 {
    if tc.is_null() { return -1; }
    match fsw() { Some(f) => f.command(unsafe { core::slice::from_raw_parts(tc, len) }), None => -1 }
}

#[no_mangle]
pub extern "C" fn adcs_fsw_peek(out: *mut State) -> i32 {
    if out.is_null() { return -1; }
    match fsw().as_ref().and_then(|f| f.peek()) { Some(s) => { unsafe { *out = s; } 0 } None => -1 }
}

#[no_mangle]
pub extern "C" fn adcs_fsw_build_id() -> *const u8 { b"trinetra-fsw-rs/1.0.0 (adcs-fswcfg/1)\0".as_ptr() }

/// Extension for the SILS and the parity ledger (as adcs_fsw_debug in C).
#[no_mangle]
pub extern "C" fn adcs_fsw_debug(out: *mut f64, n: i32) -> i32 {
    match fsw() {
        Some(f) if !out.is_null() && n >= 0 => f.debug(unsafe { core::slice::from_raw_parts_mut(out, n as usize) }) as i32,
        _ => -1,
    }
}

/// Hosted no_std builds (a POSIX virtual OBC) link against a `core` compiled with
/// unwinding tables; with panic = "abort" the personality routine is never called.
#[cfg(all(not(feature = "std"), not(target_os = "none")))]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
