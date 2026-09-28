//! Scalar maths: the platform libm on host builds (same results as the C build),
//! the pure-Rust libm crate on no_std targets.
#[cfg(feature = "std")]
mod imp {
    #[inline] pub fn sin(x: f64) -> f64 { x.sin() }
    #[inline] pub fn cos(x: f64) -> f64 { x.cos() }
    #[inline] pub fn sqrt(x: f64) -> f64 { x.sqrt() }
    #[inline] pub fn acos(x: f64) -> f64 { x.acos() }
    #[inline] pub fn atan2(y: f64, x: f64) -> f64 { y.atan2(x) }
    #[inline] pub fn exp(x: f64) -> f64 { x.exp() }
    #[inline] pub fn round(x: f64) -> f64 { x.round() }
    #[inline] pub fn fmod(x: f64, y: f64) -> f64 { x % y }
}
#[cfg(not(feature = "std"))]
mod imp {
    #[inline] pub fn sin(x: f64) -> f64 { libm::sin(x) }
    #[inline] pub fn cos(x: f64) -> f64 { libm::cos(x) }
    #[inline] pub fn sqrt(x: f64) -> f64 { libm::sqrt(x) }
    #[inline] pub fn acos(x: f64) -> f64 { libm::acos(x) }
    #[inline] pub fn atan2(y: f64, x: f64) -> f64 { libm::atan2(y, x) }
    #[inline] pub fn exp(x: f64) -> f64 { libm::exp(x) }
    #[inline] pub fn round(x: f64) -> f64 { libm::round(x) }
    #[inline] pub fn fmod(x: f64, y: f64) -> f64 { libm::fmod(x, y) }
}
pub use imp::*;
#[inline] pub fn fabs(x: f64) -> f64 { if x < 0.0 { -x } else if x == 0.0 { 0.0 } else { x } }
