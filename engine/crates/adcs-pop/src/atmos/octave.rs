//! MATLAB/Octave numerical semantics the ported models depend on, reproduced so the
//! Rust results match the reference interpreter bit for bit where that is possible.
//!
//! * A scalar `x^y` / `x.^y` in Octave is C `pow(x, y)` -- not `x*x` for `y = 2`
//!   (glibc `pow(x, 2)` differs from `x*x` in the last bit for a few inputs in 10^3).
//!   [`pw`] keeps LLVM from rewriting `powf(x, 2.0)` into `x*x`.
//! * `mod` has Octave's near-integer-quotient rule ([`mod_`]).
//! * `norm` of a vector is the scaled (LAPACK-style) 2-norm accumulator ([`norm`]).
//! * `datenum`, `interp1(..,'linear','extrap')`, `interp1(..,'nearest')`, `erf`.
use crate::la::V3;
use std::f64::consts::PI;

/// Octave scalar power `x^y` (and scalar `x.^y`): C `pow`. The exponent goes through
/// `black_box` so the compiler cannot strength-reduce `pow(x, 2.0)` to `x*x`.
#[inline]
pub fn pw(x: f64, y: f64) -> f64 {
    x.powf(std::hint::black_box(y))
}

/// Octave `deg2rad`: `deg * (pi / 180)`.
#[inline]
pub fn deg2rad(d: f64) -> f64 {
    d * (PI / 180.0)
}

/// Octave `rad2deg`: `rad * (180 / pi)`.
#[inline]
pub fn rad2deg(r: f64) -> f64 {
    r * (180.0 / PI)
}

#[inline]
fn x_nint(x: f64) -> f64 {
    if x.is_finite() { (x + 0.5).floor() } else { x }
}

/// Octave `mod(x, y)` (liboctave `octave::math::mod`): `x - floor(x/y)*y`, except that a
/// quotient within one epsilon (relative) of an integer gives exactly 0 when `y` is not
/// an integer, and the result carries the sign of `y`.
pub fn mod_(x: f64, y: f64) -> f64 {
    let mut r;
    if y == 0.0 {
        r = x;
    } else {
        let q = x / y;
        if x_nint(y) != y && ((q - x_nint(q)) / x_nint(q)).abs() < f64::EPSILON {
            r = 0.0;
        } else {
            let n = q.floor();
            let tmp = y * n;
            r = x - tmp;
        }
    }
    if x != y && y != 0.0 {
        r = r.copysign(y);
    }
    r
}

/// Octave `rem(x, y)` for the integer arguments `finddays` uses: `x - fix(x/y)*y`.
pub fn rem_(x: f64, y: f64) -> f64 {
    if y == 0.0 {
        return f64::NAN;
    }
    let q = x / y;
    if x_nint(q) == q {
        return 0.0f64.copysign(x);
    }
    let r = x - y * q.trunc();
    if x != y { r.abs() * x.signum() } else { r }
}

/// Octave `norm(v)` of a vector: the scaled sum-of-squares accumulator
/// (`scl * sqrt(sum)`), not `sqrt(dot(v, v))`.
pub fn norm(v: &V3) -> f64 {
    let mut scl = 0.0f64;
    let mut sum = 1.0f64;
    for &x in v.iter() {
        let t = x.abs();
        if scl == t {
            sum += 1.0;
        } else if scl < t {
            let q = scl / t;
            sum *= q * q;
            sum += 1.0;
            scl = t;
        } else if t != 0.0 {
            let q = t / scl;
            sum += q * q;
        }
    }
    scl * sum.sqrt()
}

/// `v / norm(v)` with the Octave norm.
pub fn unit(v: &V3) -> V3 {
    let n = norm(v);
    [v[0] / n, v[1] / n, v[2] / n]
}

/// Octave `datenum(Y, M, D, h, mi, s)` for integer months (days since year 0).
pub fn datenum(y: f64, mo: f64, d: f64, h: f64, mi: f64, s: f64) -> f64 {
    const MONTHSTART: [f64; 12] = [306.0, 337.0, 0.0, 31.0, 61.0, 92.0, 122.0, 153.0, 184.0, 214.0, 245.0, 275.0];
    let month = if mo < 1.0 { 1.0 } else { mo };
    let year = y + ((month - 14.0) / 12.0).ceil();
    let mut day = d + (MONTHSTART[mod_(month - 1.0, 12.0) as usize] + 60.0);
    day += 365.0 * year + (year / 4.0).floor() - (year / 100.0).floor() + (year / 400.0).floor();
    day + (h + (mi + s / 60.0) / 60.0) / 24.0
}

/// `datenum` of a `[Y M D h m s]` vector.
pub fn datenum_v(u: &[f64; 6]) -> f64 {
    datenum(u[0], u[1], u[2], u[3], u[4], u[5])
}

/// Octave `lookup(x, xi, "lr")` (0-based): the interval index `i` with `x[i] <= xi`,
/// clamped to `0 ..= n-2`.
fn lookup_lr(x: &[f64], xi: f64) -> usize {
    let n = x.len();
    // number of elements <= xi
    let (mut lo, mut hi) = (0usize, n);
    while lo < hi {
        let mid = (lo + hi) / 2;
        if x[mid] <= xi { lo = mid + 1 } else { hi = mid }
    }
    let i = if lo == 0 { 0 } else { lo - 1 };
    i.min(n.saturating_sub(2))
}

/// Octave `interp1(x, y, xi, 'linear', 'extrap')` for sorted `x` without repeated
/// abscissae: `(dy/dx)*(xi - x(i)) + y(i)` on the bracketing (or edge) interval.
pub fn interp1_linear_extrap(x: &[f64], y: &[f64], xi: f64) -> f64 {
    let i = lookup_lr(x, xi);
    let slope = (y[i + 1] - y[i]) / (x[i + 1] - x[i]);
    slope * (xi - x[i]) + y[i]
}

/// Octave `interp1(x, y, xi, 'nearest')` (no extrapolation: NaN outside `[x1, xn]`).
/// Break points are the midpoints `(x(i)+x(i+1))/2`; a tie goes to the upper node.
pub fn interp1_nearest(x: &[f64], y: &[f64], xi: f64) -> f64 {
    let n = x.len();
    if !(xi >= x[0] && xi <= x[n - 1]) {
        return f64::NAN;
    }
    // breaks = [x1, mids..., xn]; interval k (0-based) -> y[k]
    let mut k = 0usize;
    for i in 0..n - 1 {
        if (x[i] + x[i + 1]) / 2.0 <= xi { k = i + 1 } else { break }
    }
    y[k.min(n - 1)]
}

extern "C" {
    #[link_name = "erf"]
    fn c_erf(x: f64) -> f64;
}

/// The error function, from the platform C math library -- the same `erf` Octave calls
/// (Rust std has no stable `erf`).
#[inline]
pub fn erf(x: f64) -> f64 {
    // SAFETY: `erf` is a pure C99 <math.h> function of one double, provided by the
    // system libm that std already links.
    unsafe { c_erf(x) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mod_matches_octave_cases() {
        assert_eq!(mod_(0.9, 0.3), 0.0);
        assert_eq!(mod_(-1e-20, 24.0), 24.0);
        assert_eq!(mod_(48.0, 24.0), 0.0);
        assert_eq!(mod_(0.69999999999999984, 0.7), 0.0);
    }
    #[test]
    fn datenum_known() {
        assert_eq!(datenum(2001.0, 5.0, 19.0, 0.0, 0.0, 0.0), 730990.0);
        assert_eq!(datenum(1858.0, 11.0, 17.0, 0.0, 0.0, 0.0), 678942.0);
    }
}
