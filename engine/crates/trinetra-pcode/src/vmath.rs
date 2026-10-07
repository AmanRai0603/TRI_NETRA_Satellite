//! The maths library of the interpreter: JavaScript's `Math` functions, to the last bit where that
//! can be had. `pow`, `log`, `log10`, `log2` and `atan2` are ported here line for line from the fdlibm code
//! JavaScript's engine compiles (V8, `src/base/ieee754.cc`), and `hypot` from V8's builtin (`math.tq`); `tan`, `asin`, `acos`, `atan` and `exp`
//! come from the `libm` crate, whose fdlibm port gives the same bits; `sin` and `cos` too, though
//! the engine of Node 22 computes those two with a different algorithm (see lib.rs).
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
//!
//! The functions below are adapted from fdlibm, whose notice is preserved:
//!
//! Copyright (C) 1993 by Sun Microsystems, Inc. All rights reserved.
//! Developed at SunSoft, a Sun Microsystems, Inc. business.
//! Permission to use, copy, modify, and distribute this software is freely granted, provided that
//! this notice is preserved.

// the constants are fdlibm's, written as fdlibm writes them, so they read against the original
#![allow(clippy::excessive_precision, clippy::approx_constant)]

fn hi(x: f64) -> i32 {
    (x.to_bits() >> 32) as u32 as i32
}
fn lo(x: f64) -> u32 {
    x.to_bits() as u32
}
fn words(h: i32, l: u32) -> f64 {
    f64::from_bits(((h as u32 as u64) << 32) | l as u64)
}
fn with_hi(x: f64, h: i32) -> f64 {
    words(h, lo(x))
}
fn with_lo(x: f64, l: u32) -> f64 {
    words(hi(x), l)
}

pub fn sin(x: f64) -> f64 {
    libm::sin(x)
}
pub fn cos(x: f64) -> f64 {
    libm::cos(x)
}
pub fn tan(x: f64) -> f64 {
    libm::tan(x)
}
pub fn asin(x: f64) -> f64 {
    libm::asin(x)
}
pub fn acos(x: f64) -> f64 {
    libm::acos(x)
}
pub fn atan(x: f64) -> f64 {
    libm::atan(x)
}
pub fn exp(x: f64) -> f64 {
    libm::exp(x)
}

/// Math.atan2(y, x).
pub fn atan2(y: f64, x: f64) -> f64 {
    const TINY: f64 = 1.0e-300;
    const PI_O_4: f64 = 7.8539816339744827900E-01;
    const PI_O_2: f64 = 1.5707963267948965580E+00;
    const PI: f64 = 3.1415926535897931160E+00;
    const PI_LO: f64 = 1.2246467991473531772E-16;
    let (hx, lx) = (hi(x), lo(x));
    let ix = hx & 0x7FFFFFFF;
    let (hy, ly) = (hi(y), lo(y));
    let iy = hy & 0x7FFFFFFF;
    if ix > 0x7FF00000 || (ix == 0x7FF00000 && lx != 0) || iy > 0x7FF00000 || (iy == 0x7FF00000 && ly != 0) {
        return x + y; // x or y is NaN
    }
    if (hx.wrapping_sub(0x3FF00000) as u32 | lx) == 0 {
        return atan(y); // x = 1.0
    }
    let mut m = ((hy >> 31) & 1) | ((hx >> 30) & 2); // 2*sign(x)+sign(y)
    if (iy as u32 | ly) == 0 {
        match m {
            0 | 1 => return y,
            2 => return PI + TINY,
            _ => return -PI - TINY,
        }
    }
    if (ix as u32 | lx) == 0 {
        return if hy < 0 { -PI_O_2 - TINY } else { PI_O_2 + TINY };
    }
    if ix == 0x7FF00000 {
        if iy == 0x7FF00000 {
            return match m {
                0 => PI_O_4 + TINY,
                1 => -PI_O_4 - TINY,
                2 => 3.0 * PI_O_4 + TINY,
                _ => -3.0 * PI_O_4 - TINY,
            };
        } else {
            return match m {
                0 => 0.0,
                1 => -0.0,
                2 => PI + TINY,
                _ => -PI - TINY,
            };
        }
    }
    if iy == 0x7FF00000 {
        return if hy < 0 { -PI_O_2 - TINY } else { PI_O_2 + TINY };
    }
    let k = (iy - ix) >> 20;
    let z = if k > 60 {
        m &= 1;
        PI_O_2 + 0.5 * PI_LO
    } else if hx < 0 && k < -60 {
        0.0
    } else {
        atan((y / x).abs())
    };
    match m {
        0 => z,
        1 => -z,
        2 => PI - (z - PI_LO),
        _ => (z - PI_LO) - PI,
    }
}

/// Math.log(x).
pub fn log(x: f64) -> f64 {
    const LN2_HI: f64 = 6.93147180369123816490e-01;
    const LN2_LO: f64 = 1.90821492927058770002e-10;
    const TWO54: f64 = 1.80143985094819840000e+16;
    const LG1: f64 = 6.666666666666735130e-01;
    const LG2: f64 = 3.999999999940941908e-01;
    const LG3: f64 = 2.857142874366239149e-01;
    const LG4: f64 = 2.222219843214978396e-01;
    const LG5: f64 = 1.818357216161805012e-01;
    const LG6: f64 = 1.531383769920937332e-01;
    const LG7: f64 = 1.479819860511658591e-01;
    let mut x = x;
    let mut hx = hi(x);
    let lx = lo(x);
    let mut k: i32 = 0;
    if hx < 0x00100000 {
        if ((hx & 0x7FFFFFFF) as u32 | lx) == 0 {
            return f64::NEG_INFINITY;
        }
        if hx < 0 {
            return f64::NAN;
        }
        k -= 54;
        x *= TWO54;
        hx = hi(x);
    }
    if hx >= 0x7FF00000 {
        return x + x;
    }
    k += (hx >> 20) - 1023;
    hx &= 0x000FFFFF;
    let i = (hx + 0x95F64) & 0x100000;
    x = with_hi(x, hx | (i ^ 0x3FF00000));
    k += i >> 20;
    let f = x - 1.0;
    if (0x000FFFFF & (2 + hx)) < 3 {
        if f == 0.0 {
            if k == 0 {
                return 0.0;
            }
            let dk = k as f64;
            return dk * LN2_HI + dk * LN2_LO;
        }
        let r = f * f * (0.5 - 0.33333333333333333 * f);
        if k == 0 {
            return f - r;
        }
        let dk = k as f64;
        return dk * LN2_HI - ((r - dk * LN2_LO) - f);
    }
    let s = f / (2.0 + f);
    let dk = k as f64;
    let z = s * s;
    let mut i = hx - 0x6147A;
    let w = z * z;
    let j = 0x6B851 - hx;
    let t1 = w * (LG2 + w * (LG4 + w * LG6));
    let t2 = z * (LG1 + w * (LG3 + w * (LG5 + w * LG7)));
    i |= j;
    let r = t2 + t1;
    if i > 0 {
        let hfsq = 0.5 * f * f;
        if k == 0 {
            f - (hfsq - s * (hfsq + r))
        } else {
            dk * LN2_HI - ((hfsq - (s * (hfsq + r) + dk * LN2_LO)) - f)
        }
    } else if k == 0 {
        f - s * (f - r)
    } else {
        dk * LN2_HI - ((s * (f - r) - dk * LN2_LO) - f)
    }
}

/// Math.log10(x).
pub fn log10(x: f64) -> f64 {
    const TWO54: f64 = 1.80143985094819840000e+16;
    const IVLN10: f64 = 4.34294481903251816668e-01;
    const LOG10_2HI: f64 = 3.01029995663611771306e-01;
    const LOG10_2LO: f64 = 3.69423907715893078616e-13;
    let mut x = x;
    let mut hx = hi(x);
    let mut lx = lo(x);
    let mut k: i32 = 0;
    if hx < 0x00100000 {
        if ((hx & 0x7FFFFFFF) as u32 | lx) == 0 {
            return f64::NEG_INFINITY;
        }
        if hx < 0 {
            return f64::NAN;
        }
        k -= 54;
        x *= TWO54;
        hx = hi(x);
        lx = lo(x);
    }
    if hx >= 0x7FF00000 {
        return x + x;
    }
    if hx == 0x3FF00000 && lx == 0 {
        return 0.0;
    }
    k += (hx >> 20) - 1023;
    let i = ((k as u32 & 0x80000000) >> 31) as i32;
    hx = (hx & 0x000FFFFF) | ((0x3FF - i) << 20);
    let y = (k + i) as f64;
    x = words(hx, lx);
    let z = y * LOG10_2LO + IVLN10 * log(x);
    z + y * LOG10_2HI
}

/// fdlibm's k_log1p (k_log.h, as V8 has it): log(1 + f) - f + f*f/2 for 1 + f in [sqrt(2)/2, sqrt(2)].
fn k_log1p(f: f64) -> f64 {
    const LG1: f64 = 6.666666666666735130e-01;
    const LG2: f64 = 3.999999999940941908e-01;
    const LG3: f64 = 2.857142874366239149e-01;
    const LG4: f64 = 2.222219843214978396e-01;
    const LG5: f64 = 1.818357216161805012e-01;
    const LG6: f64 = 1.531383769920937332e-01;
    const LG7: f64 = 1.479819860511658591e-01;
    let s = f / (2.0 + f);
    let z = s * s;
    let w = z * z;
    let t1 = w * (LG2 + w * (LG4 + w * LG6));
    let t2 = z * (LG1 + w * (LG3 + w * (LG5 + w * LG7)));
    let r = t2 + t1;
    let hfsq = 0.5 * f * f;
    s * (hfsq + r)
}

/// Math.log2(x) (V8's ieee754::log2, FreeBSD's e_log2.c).
pub fn log2(x: f64) -> f64 {
    const TWO54: f64 = 1.80143985094819840000e+16;
    const IVLN2HI: f64 = 1.44269504072144627571e+00;
    const IVLN2LO: f64 = 1.67517131648865118353e-10;
    let mut x = x;
    let mut hx = hi(x);
    let lx = lo(x);
    let mut k: i32 = 0;
    if hx < 0x00100000 {
        if ((hx & 0x7FFFFFFF) as u32 | lx) == 0 {
            return f64::NEG_INFINITY;
        }
        if hx < 0 {
            return f64::NAN;
        }
        k -= 54;
        x *= TWO54;
        hx = hi(x);
    }
    if hx >= 0x7FF00000 {
        return x + x;
    }
    if hx == 0x3FF00000 && lx == 0 {
        return 0.0;
    }
    k += (hx >> 20) - 1023;
    hx &= 0x000FFFFF;
    let i = (hx + 0x95F64) & 0x100000;
    x = with_hi(x, hx | (i ^ 0x3FF00000));
    k += i >> 20;
    let y = k as f64;
    let f = x - 1.0;
    let hfsq = 0.5 * f * f;
    let r = k_log1p(f);
    let hi_ = with_lo(f - hfsq, 0);
    let lo_ = (f - hi_) - hfsq + r;
    let mut val_hi = hi_ * IVLN2HI;
    let mut val_lo = (lo_ + hi_) * IVLN2LO + lo_ * IVLN2HI;
    let w = y + val_hi;
    val_lo += (y - w) + val_hi;
    val_hi = w;
    val_lo + val_hi
}

/// Math.hypot(x, y) (V8's builtin, math.tq): both scaled by the larger, the squares summed with Kahan's
/// compensation, the root times the larger.
pub fn hypot(x: f64, y: f64) -> f64 {
    let (ax, ay) = (x.abs(), y.abs());
    let max = if ay > ax { ay } else { ax };
    if ax == f64::INFINITY || ay == f64::INFINITY {
        return f64::INFINITY;
    }
    if x.is_nan() || y.is_nan() {
        return f64::NAN;
    }
    if max == 0.0 {
        return 0.0;
    }
    let (mut sum, mut compensation) = (0.0f64, 0.0f64);
    for v in [ax, ay] {
        let n = v / max;
        let summand = n * n - compensation;
        let preliminary = sum + summand;
        compensation = (preliminary - sum) - summand;
        sum = preliminary;
    }
    sum.sqrt() * max
}

/// Math.pow(x, y).
pub fn pow(x: f64, y: f64) -> f64 {
    const BP: [f64; 2] = [1.0, 1.5];
    const DP_H: [f64; 2] = [0.0, 5.84962487220764160156e-01];
    const DP_L: [f64; 2] = [0.0, 1.35003920212974897128e-08];
    const TWO53: f64 = 9007199254740992.0;
    const HUGE: f64 = 1.0e300;
    const TINY: f64 = 1.0e-300;
    const L1: f64 = 5.99999999999994648725e-01;
    const L2: f64 = 4.28571428578550184252e-01;
    const L3: f64 = 3.33333329818377432918e-01;
    const L4: f64 = 2.72728123808534006489e-01;
    const L5: f64 = 2.30660745775561754067e-01;
    const L6: f64 = 2.06975017800338417784e-01;
    const P1: f64 = 1.66666666666666019037e-01;
    const P2: f64 = -2.77777777770155933842e-03;
    const P3: f64 = 6.61375632143793436117e-05;
    const P4: f64 = -1.65339022054652515390e-06;
    const P5: f64 = 4.13813679705723846039e-08;
    const LG2: f64 = 6.93147180559945286227e-01;
    const LG2_H: f64 = 6.93147182464599609375e-01;
    const LG2_L: f64 = -1.90465429995776804525e-09;
    const OVT: f64 = 8.0085662595372944372e-0017;
    const CP: f64 = 9.61796693925975554329e-01;
    const CP_H: f64 = 9.61796700954437255859e-01;
    const CP_L: f64 = -7.02846165095275826516e-09;
    const IVLN2: f64 = 1.44269504088896338700e+00;
    const IVLN2_H: f64 = 1.44269502162933349609e+00;
    const IVLN2_L: f64 = 1.92596299112661746887e-08;

    let (hx, lx) = (hi(x), lo(x));
    let (hy, ly) = (hi(y), lo(y));
    let mut ix = hx & 0x7fffffff;
    let iy = hy & 0x7fffffff;

    if (iy as u32 | ly) == 0 {
        return 1.0;
    }
    if ix > 0x7ff00000 || (ix == 0x7ff00000 && lx != 0) || iy > 0x7ff00000 || (iy == 0x7ff00000 && ly != 0) {
        return x + y;
    }
    // yisint: 0 y is not an integer, 1 an odd one, 2 an even one
    let mut yisint: i32 = 0;
    if hx < 0 {
        if iy >= 0x43400000 {
            yisint = 2;
        } else if iy >= 0x3ff00000 {
            let k = (iy >> 20) - 0x3ff;
            if k > 20 {
                let j = (ly >> (52 - k)) as i32;
                if j.wrapping_shl((52 - k) as u32) == ly as i32 {
                    yisint = 2 - (j & 1);
                }
            } else if ly == 0 {
                let j = iy >> (20 - k);
                if (j << (20 - k)) == iy {
                    yisint = 2 - (j & 1);
                }
            }
        }
    }
    if ly == 0 {
        if iy == 0x7ff00000 {
            if (ix.wrapping_sub(0x3ff00000) as u32 | lx) == 0 {
                return f64::NAN; // inf**+-1 is NaN (fdlibm's y - y)
            } else if ix >= 0x3ff00000 {
                return if hy >= 0 { y } else { 0.0 };
            } else {
                return if hy < 0 { -y } else { 0.0 };
            }
        }
        if iy == 0x3ff00000 {
            return if hy < 0 { 1.0 / x } else { x };
        }
        if hy == 0x40000000 {
            return x * x;
        }
        if hy == 0x3fe00000 && hx >= 0 {
            return x.sqrt();
        }
    }
    let mut ax = x.abs();
    if lx == 0 && (ix == 0x7ff00000 || ix == 0 || ix == 0x3ff00000) {
        let mut z = ax;
        if hy < 0 {
            z = 1.0 / z;
        }
        if hx < 0 {
            if (ix.wrapping_sub(0x3ff00000) | yisint) == 0 {
                z = f64::NAN;
            } else if yisint == 1 {
                z = -z;
            }
        }
        return z;
    }
    let mut n: i32 = (hx >> 31) + 1;
    if (n | yisint) == 0 {
        return f64::NAN;
    }
    let mut s = 1.0;
    if (n | (yisint - 1)) == 0 {
        s = -1.0;
    }
    let (t1, t2);
    if iy > 0x41e00000 {
        if iy > 0x43f00000 {
            if ix <= 0x3fefffff {
                return if hy < 0 { HUGE * HUGE } else { TINY * TINY };
            }
            if ix >= 0x3ff00000 {
                return if hy > 0 { HUGE * HUGE } else { TINY * TINY };
            }
        }
        if ix < 0x3fefffff {
            return if hy < 0 { s * HUGE * HUGE } else { s * TINY * TINY };
        }
        if ix > 0x3ff00000 {
            return if hy > 0 { s * HUGE * HUGE } else { s * TINY * TINY };
        }
        let t = ax - 1.0;
        let w = (t * t) * (0.5 - t * (0.3333333333333333333333 - t * 0.25));
        let u = IVLN2_H * t;
        let v = t * IVLN2_L - w * IVLN2;
        let a = with_lo(u + v, 0);
        t1 = a;
        t2 = v - (a - u);
    } else {
        n = 0;
        if ix < 0x00100000 {
            ax *= TWO53;
            n -= 53;
            ix = hi(ax);
        }
        n += (ix >> 20) - 0x3ff;
        let j = ix & 0x000fffff;
        ix = j | 0x3ff00000;
        let k: usize;
        if j <= 0x3988E {
            k = 0;
        } else if j < 0xBB67A {
            k = 1;
        } else {
            k = 0;
            n += 1;
            ix -= 0x00100000;
        }
        ax = with_hi(ax, ix);
        let u = ax - BP[k];
        let v = 1.0 / (ax + BP[k]);
        let ss = u * v;
        let s_h = with_lo(ss, 0);
        let t_h = with_hi(0.0, ((ix >> 1) | 0x20000000) + 0x00080000 + ((k as i32) << 18));
        let t_l = ax - (t_h - BP[k]);
        let s_l = v * ((u - s_h * t_h) - s_h * t_l);
        let mut s2 = ss * ss;
        let mut r = s2 * s2 * (L1 + s2 * (L2 + s2 * (L3 + s2 * (L4 + s2 * (L5 + s2 * L6)))));
        r += s_l * (s_h + ss);
        s2 = s_h * s_h;
        let t_h = with_lo(3.0 + s2 + r, 0);
        let t_l = r - ((t_h - 3.0) - s2);
        let u = s_h * t_h;
        let v = s_l * t_h + t_l * ss;
        let p_h = with_lo(u + v, 0);
        let p_l = v - (p_h - u);
        let z_h = CP_H * p_h;
        let z_l = CP_L * p_h + p_l * CP + DP_L[k];
        let t = n as f64;
        let a = with_lo(((z_h + z_l) + DP_H[k]) + t, 0);
        t1 = a;
        t2 = z_l - (((a - t) - DP_H[k]) - z_h);
    }
    // split y into y1 + y2 and compute (y1 + y2)*(t1 + t2)
    let y1 = with_lo(y, 0);
    let p_l = (y - y1) * t1 + y * t2;
    let mut p_h = y1 * t1;
    let mut z = p_l + p_h;
    let j = hi(z);
    let i = lo(z) as i32;
    if j >= 0x40900000 {
        if (j.wrapping_sub(0x40900000) | i) != 0 || p_l + OVT > z - p_h {
            return s * HUGE * HUGE; // overflow
        }
    } else if (j & 0x7fffffff) >= 0x4090cc00 && (((j as u32).wrapping_sub(0xc090cc00) | i as u32) != 0 || p_l <= z - p_h) {
        return s * TINY * TINY; // underflow
    }
    // 2^(p_h + p_l)
    let i = j & 0x7fffffff;
    let mut k = (i >> 20) - 0x3ff;
    let mut n: i32 = 0;
    if i > 0x3fe00000 {
        n = j.wrapping_add(0x00100000 >> (k + 1));
        k = ((n & 0x7fffffff) >> 20) - 0x3ff;
        let t = with_hi(0.0, n & !(0x000fffff >> k));
        n = ((n & 0x000fffff) | 0x00100000) >> (20 - k);
        if j < 0 {
            n = -n;
        }
        p_h -= t;
    }
    let t = with_lo(p_l + p_h, 0);
    let u = t * LG2_H;
    let v = (p_l - (t - p_h)) * LG2 + t * LG2_L;
    z = u + v;
    let w = v - (z - u);
    let t = z * z;
    let t1 = z - t * (P1 + t * (P2 + t * (P3 + t * (P4 + t * P5))));
    let r = (z * t1) / ((t1 - 2.0) - (w + z * w));
    z = 1.0 - (r - z);
    let mut j = hi(z);
    j = j.wrapping_add(((n as u32) << 20) as i32);
    if (j >> 20) <= 0 {
        z = libm::scalbn(z, n);
    } else {
        let tmp = hi(z);
        z = with_hi(z, tmp.wrapping_add(((n as u32) << 20) as i32));
    }
    s * z
}

/// erf(x): fdlibm's s_erf.c (JavaScript has no erf; design/js/pcode.js `rt.erf` is this, line for line), over the
/// `exp` above. The translations call the platform's erf, whose last bits may differ (as exp's).
pub fn erf(x: f64) -> f64 {
    const ERX: f64 = 8.45062911510467529297e-01;
    const EFX: f64 = 1.28379167095512586316e-01;
    const EFX8: f64 = 1.02703333676410069053e+00;
    const PP0: f64 = 1.28379167095512558561e-01;
    const PP1: f64 = -3.25042107247001499370e-01;
    const PP2: f64 = -2.84817495755985104766e-02;
    const PP3: f64 = -5.77027029648944159157e-03;
    const PP4: f64 = -2.37630166566501626084e-05;
    const QQ1: f64 = 3.97917223959155352819e-01;
    const QQ2: f64 = 6.50222499887672944485e-02;
    const QQ3: f64 = 5.08130628187576562776e-03;
    const QQ4: f64 = 1.32494738004321644526e-04;
    const QQ5: f64 = -3.96022827877536812320e-06;
    const PA0: f64 = -2.36211856075265944077e-03;
    const PA1: f64 = 4.14856118683748331666e-01;
    const PA2: f64 = -3.72207876035701323847e-01;
    const PA3: f64 = 3.18346619901161753674e-01;
    const PA4: f64 = -1.10894694282396677476e-01;
    const PA5: f64 = 3.54783043256182359371e-02;
    const PA6: f64 = -2.16637559486879084300e-03;
    const QA1: f64 = 1.06420880400844228286e-01;
    const QA2: f64 = 5.40397917702171048937e-01;
    const QA3: f64 = 7.18286544141962662868e-02;
    const QA4: f64 = 1.26171219808761642112e-01;
    const QA5: f64 = 1.36370839120290507362e-02;
    const QA6: f64 = 1.19844998467991074170e-02;
    const RA0: f64 = -9.86494403484714822705e-03;
    const RA1: f64 = -6.93858572707181764372e-01;
    const RA2: f64 = -1.05586262253232909814e+01;
    const RA3: f64 = -6.23753324503260060396e+01;
    const RA4: f64 = -1.62396669462573470355e+02;
    const RA5: f64 = -1.84605092906711035994e+02;
    const RA6: f64 = -8.12874355063065934246e+01;
    const RA7: f64 = -9.81432934416914548592e+00;
    const SA1: f64 = 1.96512716674392571292e+01;
    const SA2: f64 = 1.37657754143519042600e+02;
    const SA3: f64 = 4.34565877475229228821e+02;
    const SA4: f64 = 6.45387271733267880336e+02;
    const SA5: f64 = 4.29008140027567833386e+02;
    const SA6: f64 = 1.08635005541779435134e+02;
    const SA7: f64 = 6.57024977031928170135e+00;
    const SA8: f64 = -6.04244152148580987438e-02;
    const RB0: f64 = -9.86494292470009928597e-03;
    const RB1: f64 = -7.99283237680523006574e-01;
    const RB2: f64 = -1.77579549177547519889e+01;
    const RB3: f64 = -1.60636384855821916062e+02;
    const RB4: f64 = -6.37566443368389627722e+02;
    const RB5: f64 = -1.02509513161107724954e+03;
    const RB6: f64 = -4.83519191608651397019e+02;
    const SB1: f64 = 3.03380607434824582924e+01;
    const SB2: f64 = 3.25792512996573918826e+02;
    const SB3: f64 = 1.53672958608443695994e+03;
    const SB4: f64 = 3.19985821950859553908e+03;
    const SB5: f64 = 2.55305040643316442583e+03;
    const SB6: f64 = 4.74528541206955367215e+02;
    const SB7: f64 = -2.24409524465858183362e+01;
    let hx = hi(x);
    let ix = hx & 0x7fffffff;
    if ix >= 0x7ff00000 {
        // erf(nan) = nan, erf(+-inf) = +-1
        return (1 - (((hx as u32) >> 31) << 1) as i32) as f64 + 1.0 / x;
    }
    if ix < 0x3feb0000 {
        if ix < 0x3e300000 {
            if ix < 0x00800000 {
                return 0.125 * (8.0 * x + EFX8 * x);
            }
            return x + EFX * x;
        }
        let z = x * x;
        let r = PP0 + z * (PP1 + z * (PP2 + z * (PP3 + z * PP4)));
        let s = 1.0 + z * (QQ1 + z * (QQ2 + z * (QQ3 + z * (QQ4 + z * QQ5))));
        return x + x * (r / s);
    }
    if ix < 0x3ff40000 {
        let s = x.abs() - 1.0;
        let p = PA0 + s * (PA1 + s * (PA2 + s * (PA3 + s * (PA4 + s * (PA5 + s * PA6)))));
        let q = 1.0 + s * (QA1 + s * (QA2 + s * (QA3 + s * (QA4 + s * (QA5 + s * QA6)))));
        return if hx >= 0 { ERX + p / q } else { -ERX - p / q };
    }
    if ix >= 0x40180000 {
        return if hx >= 0 { 1.0 - 1e-300 } else { 1e-300 - 1.0 };
    }
    let ax = x.abs();
    let s = 1.0 / (ax * ax);
    let (r, ss) = if ix < 0x4006db6e {
        (
            RA0 + s * (RA1 + s * (RA2 + s * (RA3 + s * (RA4 + s * (RA5 + s * (RA6 + s * RA7)))))),
            1.0 + s * (SA1 + s * (SA2 + s * (SA3 + s * (SA4 + s * (SA5 + s * (SA6 + s * (SA7 + s * SA8))))))),
        )
    } else {
        (
            RB0 + s * (RB1 + s * (RB2 + s * (RB3 + s * (RB4 + s * (RB5 + s * RB6))))),
            1.0 + s * (SB1 + s * (SB2 + s * (SB3 + s * (SB4 + s * (SB5 + s * (SB6 + s * SB7)))))),
        )
    };
    let z = with_lo(ax, 0);
    let rr = exp(-z * z - 0.5625) * exp((z - ax) * (z + ax) + r / ss);
    if hx >= 0 {
        1.0 - rr / ax
    } else {
        rr / ax - 1.0
    }
}
