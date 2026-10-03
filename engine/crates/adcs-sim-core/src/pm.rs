//! Portable maths: every transcendental through the pure-Rust libm (bit-identical across targets).
#[inline] pub fn sin(x: f64) -> f64 { libm::sin(x) }
#[inline] pub fn cos(x: f64) -> f64 { libm::cos(x) }
#[inline] pub fn tan(x: f64) -> f64 { libm::tan(x) }
#[inline] pub fn asin(x: f64) -> f64 { libm::asin(x) }
#[inline] pub fn acos(x: f64) -> f64 { libm::acos(x) }
#[inline] pub fn atan2(y: f64, x: f64) -> f64 { libm::atan2(y, x) }
#[inline] pub fn sqrt(x: f64) -> f64 { libm::sqrt(x) }
#[inline] pub fn exp(x: f64) -> f64 { libm::exp(x) }
#[inline] pub fn ln(x: f64) -> f64 { libm::log(x) }
#[inline] pub fn log10(x: f64) -> f64 { libm::log10(x) }
#[inline] pub fn pow(x: f64, y: f64) -> f64 { libm::pow(x, y) }
#[inline] pub fn floor(x: f64) -> f64 { libm::floor(x) }
#[inline] pub fn round(x: f64) -> f64 { libm::round(x) }
#[inline] pub fn fmod(x: f64, y: f64) -> f64 { libm::fmod(x, y) }
#[inline] pub fn abs(x: f64) -> f64 { libm::fabs(x) }
#[inline] pub fn sign(x: f64) -> f64 { if x > 0.0 { 1.0 } else if x < 0.0 { -1.0 } else { 0.0 } }
#[inline] pub fn clamp(x: f64, lo: f64, hi: f64) -> f64 { if x < lo { lo } else if x > hi { hi } else { x } }
pub const PI: f64 = core::f64::consts::PI;
pub const D2R: f64 = PI/180.0;
