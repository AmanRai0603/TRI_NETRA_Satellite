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
pub mod math;
#[allow(clippy::all)]
pub mod igrf13;
pub mod params;
pub mod env;
pub mod est;
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

#[cfg(all(not(feature = "std"), not(test), feature = "cabi"))]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! { loop {} }
