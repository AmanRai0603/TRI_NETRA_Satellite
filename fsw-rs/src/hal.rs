//! The hardware abstraction the flight software runs on: adcs_hal.h as a trait.
//! A flight target, the SILS engine's device emulators and the host test stub
//! all implement it; with feature `cabi` the extern "C" adcs_hal_* symbols do.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Status { Ok = 0, Timeout = -1, Bus = -2, Arg = -3, NoDev = -4, Busy = -5 }

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub struct CanFrame { pub id: u32, pub extended: u8, pub dlc: u8, pub data: [u8; 8] }

pub trait Hal {
    fn time_ns(&mut self) -> u64;
    /// Bytes available now (Ok with n > 0), or Timeout when there are none.
    fn uart_read(&mut self, port: u8, buf: &mut [u8]) -> Result<usize, Status>;
    fn uart_write(&mut self, port: u8, buf: &[u8]) -> Status;
    fn i2c_xfer(&mut self, bus: u8, addr7: u8, tx: &[u8], rx: &mut [u8]) -> Status;
    fn spi_xfer(&mut self, bus: u8, cs: u8, tx: &[u8], rx: &mut [u8]) -> Status;
    fn can_send(&mut self, port: u8, f: &CanFrame) -> Status;
    fn can_recv(&mut self, port: u8) -> Result<CanFrame, Status>;
    fn pwm_set(&mut self, ch: u8, duty_q15: i16) -> Status;
    fn tm_emit(&mut self, _apid: u16, _payload: &[u8]) -> Status { Status::Ok }
}
