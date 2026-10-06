//! TRI-NETRA ADCS flight software in Rust -- the twin of fsw/ (C99).
//!
//! Both implement fsw/pseudocode over the same parameter blob (adcs-fswcfg/1,
//! generated from fsw/params/params.toml) and the same byte-level HAL
//! (fsw/include/adcs_hal.h, here the [`hal::Hal`] trait). `no_std`, no heap.
//! With feature `cabi` the crate exports the adcs_fsw.h symbols, so the
//! static library is a drop-in replacement for libadcs_fsw.a in the SILS
//! engine, OILS or on the OBC.
//!
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::needless_range_loop, clippy::excessive_precision, clippy::too_many_arguments)]

pub mod m;
/// The flight algorithms, written from the design by tools/flight_build.py (never edited).
pub mod alg;
pub mod math;
#[allow(clippy::all)]
pub mod igrf13;
pub mod params;
pub mod env;
pub mod est;
pub mod guid;
pub mod ctl;
pub mod alloc;
pub mod devices;
pub mod hal;
pub mod drv;
pub mod fsw;
#[cfg(feature = "cabi")]
pub mod cabi;

pub use fsw::{Fsw, State, ABI_VERSION, BUILD_ID};
pub use hal::{CanFrame, Hal, Status};
pub use params::{Mode, Params};

/// A panic is a fault the flight software could not contain: the OBC restarts (SYSRESETREQ in the
/// Cortex-M AIRCR) rather than hang with the actuators holding their last command. On a host the
/// process aborts, which the engine reports as the OBC gone.
#[cfg(all(not(feature = "std"), not(test), feature = "cabi"))]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    #[cfg(target_arch = "arm")]
    unsafe {
        core::ptr::write_volatile(0xE000_ED0C as *mut u32, 0x05FA_0004);
        loop { core::arch::asm!("dsb", "wfi"); }
    }
    #[cfg(not(target_arch = "arm"))]
    {
        extern "C" { fn abort() -> !; }
        unsafe { abort() }
    }
}
