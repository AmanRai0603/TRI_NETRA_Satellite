//! The groups' generated code as WebAssembly (tools/groupcode.py gen; do not edit): one entry,
//! `call`, that runs a function by its index in NAMES on the doubles in the input buffer.
#![allow(clippy::missing_safety_doc)]

pub const NAMES: &[&str] = &["act::gf_6", "act::gf_7", "act::gf_8", "act::gm_1", "act::gm_2", "act::gm_3", "act::gw_3", "act::gw_4", "act::propellant_per_slew", "act::propellant_per_year", "act::pump_pressure_for_torque", "act::rotor_axes", "act::steer_sr", "ctl::settling_time_2pct", "env::circular_speed", "env::density_at", "env::gd_0", "env::gd_1", "env::gd_2", "env::gd_3", "env::gd_4", "env::gd_5", "env::m2_5", "env::m3_0", "env::m3_1", "env::m3_5", "env::max_magnetic_latitude", "env::mean_motion", "env::radius", "fdir::fdir_sensors", "gdn::guidance", "gdn::yaw_flip", "nav::gmst_rot", "nav::orbit_acc", "nav::quest", "pnt::gp_5", "risk::net_closed", "risk::rk4_0", "risk::share_tested", "shared::fromdcm", "shared::fromrotvec", "shared::qmult"];
static mut BUF_IN: [f64; 256] = [0.0; 256];
static mut BUF_OUT: [f64; 64] = [0.0; 64];

#[no_mangle]
pub extern "C" fn input() -> *mut f64 { core::ptr::addr_of_mut!(BUF_IN) as *mut f64 }
#[no_mangle]
pub extern "C" fn output() -> *const f64 { core::ptr::addr_of!(BUF_OUT) as *const f64 }

/// Run NAMES[which] on the first `n` inputs; the number of outputs written, or -1.
#[no_mangle]
pub extern "C" fn call(which: u32, n: u32) -> i32 {
    let Some(name) = NAMES.get(which as usize) else { return -1 };
    let x = unsafe { core::slice::from_raw_parts(core::ptr::addr_of!(BUF_IN) as *const f64, n.min(256) as usize) };
    match adcs_groups::dispatch::call(name, x) {
        Some(y) => {
            let out = unsafe { core::slice::from_raw_parts_mut(core::ptr::addr_of_mut!(BUF_OUT) as *mut f64, 64) };
            for (o, v) in out.iter_mut().zip(&y) { *o = *v; }
            y.len().min(64) as i32
        }
        None => -1,
    }
}
