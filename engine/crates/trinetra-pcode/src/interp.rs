//! The interpreter: design/js/pcode.js's `makeInterpreter` and `rt`, operation for operation, so
//! the same inputs give the same bits. Every number is an f64 as in JavaScript (an int too: the
//! same rounding, the same 2^53 limit); a value is copied wherever the JavaScript copies it, so
//! value semantics here are the JavaScript's reference semantics as the checked language sees them.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.

use crate::ast::*;
use crate::check::{Checked, Shape, Target, VarKind};
use crate::error::RunError;
use crate::jsfmt;
use crate::vmath;
use std::borrow::Cow;
use std::cell::{Cell, OnceCell};
use std::f64::consts::PI;

/// A value, in SI: a number (a real or an int), a bool, an array, or a record's fields in the
/// order the record declares them. Several outputs of one call travel as an `Arr`.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Num(f64),
    Bool(bool),
    Arr(Vec<Value>),
    Rec(Vec<Value>),
}

impl Value {
    /// The value as numbers, as the vectors write it: arrays element by element, a record field by
    /// field, a bool as 1 or 0.
    pub fn flatten(&self) -> Vec<f64> {
        let mut out = Vec::new();
        self.flatten_into(&mut out);
        out
    }
    fn flatten_into(&self, out: &mut Vec<f64>) {
        match self {
            Value::Num(x) => out.push(*x),
            Value::Bool(b) => out.push(if *b { 1.0 } else { 0.0 }),
            Value::Arr(v) | Value::Rec(v) => v.iter().for_each(|x| x.flatten_into(out)),
        }
    }
    /// The number this is, as JavaScript would make one of it.
    pub(crate) fn num(&self) -> f64 {
        match self {
            Value::Num(x) => *x,
            Value::Bool(b) => f64::from(u8::from(*b)),
            // an array as a number: through its text ("" is 0, "5" is 5, "1,2" is not a number)
            Value::Arr(v) => match v.len() {
                0 => 0.0,
                1 => v[0].num(),
                _ => f64::NAN,
            },
            Value::Rec(_) => f64::NAN,
        }
    }
    fn truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Num(x) => *x != 0.0 && !x.is_nan(),
            _ => true,
        }
    }
    fn len(&self) -> usize {
        match self {
            Value::Arr(v) | Value::Rec(v) => v.len(),
            _ => 0,
        }
    }
    /// `a[i]` as JavaScript reads it: past the end, or of what is not an array, is undefined (here NaN).
    fn at(&self, i: usize) -> Cow<'_, Value> {
        match self {
            Value::Arr(v) | Value::Rec(v) if i < v.len() => Cow::Borrowed(&v[i]),
            _ => Cow::Owned(Value::Num(f64::NAN)),
        }
    }
}

/// What a proc keeps from one call to the next.
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    pub(crate) func: usize,
    /// each state variable by name, in the order the proc declares them
    pub vals: Vec<(String, Value)>,
}

type Env = Vec<(String, Value)>;
type R<T> = Result<T, RunError>;

// ------------------------------------------------------------------ rt
// Semantics every translation follows (docs/PSEUDOCODE_V2.md, "Numbers").
pub(crate) fn ipow(x: f64, k: f64) -> f64 {
    if k == 0.0 {
        return 1.0;
    }
    let mut r = x;
    let n = k.abs();
    let mut i = 1.0;
    while i < n {
        r *= x;
        i += 1.0;
    }
    if k < 0.0 {
        1.0 / r
    } else {
        r
    }
}
fn rmin(a: f64, b: f64) -> f64 {
    if b < a {
        b
    } else {
        a
    }
}
fn rmax(a: f64, b: f64) -> f64 {
    if b > a {
        b
    } else {
        a
    }
}
fn rabs(x: f64) -> f64 {
    if x < 0.0 {
        -x
    } else if x == 0.0 {
        0.0
    } else {
        x
    }
}
fn rclamp(x: f64, lo: f64, hi: f64) -> f64 {
    rmin(rmax(x, lo), hi)
}
fn rsign(x: f64) -> f64 {
    if x > 0.0 {
        1.0
    } else if x < 0.0 {
        -1.0
    } else {
        0.0
    }
}
/// JavaScript's Math.round: the nearest whole number, a half up (called on x >= 0 or -0 only)
fn js_round(x: f64) -> f64 {
    if !x.is_finite() {
        return x;
    }
    let f = x.floor();
    if x - f >= 0.5 {
        f + 1.0
    } else {
        f
    }
}
/// half away from zero, as C round()
fn rround(x: f64) -> f64 {
    if x < 0.0 {
        -js_round(-x)
    } else {
        js_round(x)
    }
}
fn nums(v: &Value) -> Vec<f64> {
    match v {
        Value::Arr(x) => x.iter().map(Value::num).collect(),
        _ => Vec::new(),
    }
}
fn dot(a: &[f64], b: &[f64]) -> f64 {
    let at = |v: &[f64], i: usize| v.get(i).copied().unwrap_or(f64::NAN);
    let mut s = at(a, 0) * at(b, 0);
    for (i, x) in a.iter().enumerate().skip(1) {
        s += x * at(b, i);
    }
    s
}
fn arr(v: Vec<f64>) -> Value {
    Value::Arr(v.into_iter().map(Value::Num).collect())
}
fn cross(a: &[f64], b: &[f64]) -> Vec<f64> {
    let g = |v: &[f64], i: usize| v.get(i).copied().unwrap_or(f64::NAN);
    vec![
        g(a, 1) * g(b, 2) - g(a, 2) * g(b, 1),
        g(a, 2) * g(b, 0) - g(a, 0) * g(b, 2),
        g(a, 0) * g(b, 1) - g(a, 1) * g(b, 0),
    ]
}
fn norm(a: &[f64]) -> f64 {
    dot(a, a).sqrt()
}
fn unit(a: &[f64]) -> Vec<f64> {
    let n = rmax(norm(a), 1e-30);
    a.iter().map(|x| x / n).collect()
}
fn mv(m: &Value, v: &Value) -> Value {
    let v = nums(v);
    match m {
        Value::Arr(rows) => Value::Arr(rows.iter().map(|r| Value::Num(dot(&nums(r), &v))).collect()),
        _ => Value::Arr(Vec::new()),
    }
}
fn mm(a: &Value, b: &Value) -> Value {
    let rows_b: Vec<Vec<f64>> = match b {
        Value::Arr(r) => r.iter().map(nums).collect(),
        _ => Vec::new(),
    };
    let cols = rows_b.first().map_or(0, Vec::len);
    let at = |r: &Vec<f64>, j: usize| r.get(j).copied().unwrap_or(f64::NAN);
    match a {
        Value::Arr(rows) => Value::Arr(
            rows.iter()
                .map(|row| {
                    let row = nums(row);
                    arr((0..cols)
                        .map(|j| {
                            let mut s = at(&row, 0) * at(&rows_b[0], j);
                            for (k, rb) in rows_b.iter().enumerate().skip(1) {
                                s += at(&row, k) * at(rb, j);
                            }
                            s
                        })
                        .collect())
                })
                .collect(),
        ),
        _ => Value::Arr(Vec::new()),
    }
}
fn tr(m: &Value) -> Value {
    let Value::Arr(rows) = m else { return Value::Arr(Vec::new()) };
    let cols = rows.first().map_or(0, Value::len);
    Value::Arr((0..cols).map(|j| Value::Arr(rows.iter().map(|r| r.at(j).into_owned()).collect())).collect())
}
/// the indices of a vector in ascending order, equal values in their order: an insertion sort (pcode.js `rt.argsort`)
pub(crate) fn argsort(v: &[f64]) -> Vec<usize> {
    let mut ix: Vec<usize> = (0..v.len()).collect();
    for i in 1..ix.len() {
        let k = ix[i];
        let mut j = i as i64 - 1;
        while j >= 0 && v[ix[j as usize]] > v[k] {
            ix[(j + 1) as usize] = ix[j as usize];
            j -= 1;
        }
        ix[(j + 1) as usize] = k;
    }
    ix
}
fn eig_sgn(x: f64, y: f64) -> f64 {
    if y >= 0.0 {
        rabs(x)
    } else {
        -rabs(x)
    }
}
/// The eigenvalues of a real square matrix, each [re, im] (pcode.js `rt.eig`, trinetra-toolbox/6): EISPACK's balanc by
/// powers of 2 without its permutations, orthes and hqr, sequential, the arithmetic every translation writes.
pub(crate) fn eig(rows: &[Vec<f64>]) -> Vec<[f64; 2]> {
    let n = rows.len();
    let mut a: Vec<Vec<f64>> = rows.to_vec();
    let mut wr = vec![0.0f64; n];
    let mut wi = vec![0.0f64; n];
    let mut ort = vec![0.0f64; n];
    let mut done = false;
    let mut sweep = 0;
    while sweep < 100 && !done {
        done = true;
        for i in 0..n {
            let mut c = 0.0;
            let mut r = 0.0;
            for j in 0..n {
                if j != i {
                    c = c + rabs(a[j][i]);
                    r = r + rabs(a[i][j]);
                }
            }
            if c != 0.0 && r != 0.0 && (c + r).is_finite() {
                let mut g = r / 2.0;
                let mut f = 1.0;
                let s = c + r;
                while c < g {
                    f = f * 2.0;
                    c = c * 4.0;
                }
                g = r * 2.0;
                while c > g {
                    f = f / 2.0;
                    c = c / 4.0;
                }
                if (c + r) / f < 0.95 * s {
                    done = false;
                    g = 1.0 / f;
                    for j in 0..n {
                        a[i][j] = a[i][j] * g;
                    }
                    for j in 0..n {
                        a[j][i] = a[j][i] * f;
                    }
                }
            }
        }
        sweep += 1;
    }
    for m in 1..n.saturating_sub(1) {
        let mut h = 0.0;
        let mut scale = 0.0;
        ort[m] = 0.0;
        for i in m..n {
            scale = scale + rabs(a[i][m - 1]);
        }
        if scale != 0.0 {
            for i in (m..n).rev() {
                ort[i] = a[i][m - 1] / scale;
                h = h + ort[i] * ort[i];
            }
            let g = -eig_sgn(f64::sqrt(h), ort[m]);
            h = h - ort[m] * g;
            ort[m] = ort[m] - g;
            for j in m..n {
                let mut f = 0.0;
                for i in (m..n).rev() {
                    f = f + ort[i] * a[i][j];
                }
                f = f / h;
                for i in m..n {
                    a[i][j] = a[i][j] - f * ort[i];
                }
            }
            for i in 0..n {
                let mut f = 0.0;
                for j in (m..n).rev() {
                    f = f + ort[j] * a[i][j];
                }
                f = f / h;
                for j in m..n {
                    a[i][j] = a[i][j] - f * ort[j];
                }
            }
            ort[m] = scale * ort[m];
            a[m][m - 1] = scale * g;
        }
    }
    for i in 2..n {
        for j in 0..i - 1 {
            a[i][j] = 0.0;
        }
    }
    let mut norm = 0.0;
    let mut k0 = 0;
    for i in 0..n {
        for j in k0..n {
            norm = norm + rabs(a[i][j]);
        }
        k0 = i;
    }
    let mut left = n;
    let mut t = 0.0;
    let mut itn = 30 * n;
    while left > 0 {
        let en = left - 1;
        let mut its = 0;
        loop {
            let mut l = en;
            while l > 0 {
                let mut s = rabs(a[l - 1][l - 1]) + rabs(a[l][l]);
                if s == 0.0 {
                    s = norm;
                }
                if s + rabs(a[l][l - 1]) == s {
                    break;
                }
                l -= 1;
            }
            let mut x = a[en][en];
            if l == en {
                wr[en] = x + t;
                wi[en] = 0.0;
                left -= 1;
                break;
            }
            let na = en - 1;
            let mut y = a[na][na];
            let mut w = a[en][na] * a[na][en];
            if l == na {
                let p = (y - x) / 2.0;
                let q = p * p + w;
                let mut zz = f64::sqrt(rabs(q));
                x = x + t;
                if q >= 0.0 {
                    zz = p + eig_sgn(zz, p);
                    wr[na] = x + zz;
                    wr[en] = wr[na];
                    if zz != 0.0 {
                        wr[en] = x - w / zz;
                    }
                    wi[na] = 0.0;
                    wi[en] = 0.0;
                } else {
                    wr[na] = x + p;
                    wr[en] = x + p;
                    wi[na] = zz;
                    wi[en] = -zz;
                }
                left -= 2;
                break;
            }
            if itn == 0 {
                for i in 0..=en {
                    wr[i] = f64::NAN;
                    wi[i] = f64::NAN;
                }
                left = 0;
                break;
            }
            let enm2 = na - 1;
            if its == 10 || its == 20 {
                t = t + x;
                for i in 0..=en {
                    a[i][i] = a[i][i] - x;
                }
                let s = rabs(a[en][na]) + rabs(a[na][enm2]);
                x = 0.75 * s;
                y = x;
                w = -0.4375 * s * s;
            }
            its += 1;
            itn -= 1;
            let mut m = enm2;
            let mut p;
            let mut q;
            let mut r;
            let mut zz;
            loop {
                zz = a[m][m];
                r = x - zz;
                let mut s = y - zz;
                p = (r * s - w) / a[m + 1][m] + a[m][m + 1];
                q = a[m + 1][m + 1] - zz - r - s;
                r = a[m + 2][m + 1];
                s = rabs(p) + rabs(q) + rabs(r);
                p = p / s;
                q = q / s;
                r = r / s;
                if m == l {
                    break;
                }
                let tst1 = rabs(p) * (rabs(a[m - 1][m - 1]) + rabs(zz) + rabs(a[m + 1][m + 1]));
                if tst1 + rabs(a[m][m - 1]) * (rabs(q) + rabs(r)) == tst1 {
                    break;
                }
                m -= 1;
            }
            for i in m + 2..=en {
                a[i][i - 2] = 0.0;
                if i != m + 2 {
                    a[i][i - 3] = 0.0;
                }
            }
            for k in m..=na {
                let notlas = k != na;
                if k != m {
                    p = a[k][k - 1];
                    q = a[k + 1][k - 1];
                    r = 0.0;
                    if notlas {
                        r = a[k + 2][k - 1];
                    }
                    x = rabs(p) + rabs(q) + rabs(r);
                    if x == 0.0 {
                        continue;
                    }
                    p = p / x;
                    q = q / x;
                    r = r / x;
                }
                let s = eig_sgn(f64::sqrt(p * p + q * q + r * r), p);
                if k != m {
                    a[k][k - 1] = -s * x;
                } else if l != m {
                    a[k][k - 1] = -a[k][k - 1];
                }
                p = p + s;
                x = p / s;
                y = q / s;
                zz = r / s;
                q = q / p;
                r = r / p;
                let jm = if k + 3 < en { k + 3 } else { en };
                if notlas {
                    for j in k..=en {
                        p = a[k][j] + q * a[k + 1][j] + r * a[k + 2][j];
                        a[k][j] = a[k][j] - p * x;
                        a[k + 1][j] = a[k + 1][j] - p * y;
                        a[k + 2][j] = a[k + 2][j] - p * zz;
                    }
                    for i in l..=jm {
                        p = x * a[i][k] + y * a[i][k + 1] + zz * a[i][k + 2];
                        a[i][k] = a[i][k] - p;
                        a[i][k + 1] = a[i][k + 1] - p * q;
                        a[i][k + 2] = a[i][k + 2] - p * r;
                    }
                } else {
                    for j in k..=en {
                        p = a[k][j] + q * a[k + 1][j];
                        a[k][j] = a[k][j] - p * x;
                        a[k + 1][j] = a[k + 1][j] - p * y;
                    }
                    for i in l..=jm {
                        p = x * a[i][k] + y * a[i][k + 1];
                        a[i][k] = a[i][k] - p;
                        a[i][k + 1] = a[i][k + 1] - p * q;
                    }
                }
            }
        }
    }
    wr.iter().zip(wi.iter()).map(|(r, i)| [*r, *i]).collect()
}
fn fmod(x: f64, y: f64) -> f64 {
    x % y
}
pub(crate) fn lookup(tab: &Table, x: f64) -> Vec<f64> {
    let r = &tab.si;
    let k: Vec<f64> = r.iter().map(|row| row[0]).collect();
    if tab.mode == TableMode::Step {
        let mut i = 0;
        while i + 1 < r.len() && k[i + 1] <= x {
            i += 1;
        }
        return r[i].clone();
    }
    let xc = rclamp(x, k[0], k[k.len() - 1]);
    let mut i = 0;
    while i + 2 < r.len() && k[i + 1] <= xc {
        i += 1;
    }
    let t = (xc - k[i]) / (k[i + 1] - k[i]);
    r[i].iter().enumerate().map(|(j, a)| a + t * (r[i + 1].get(j).copied().unwrap_or(f64::NAN) - a)).collect()
}

// element by element, and a number over an array
fn neg(a: &Value) -> Value {
    match a {
        Value::Arr(v) => Value::Arr(v.iter().map(neg).collect()),
        x => Value::Num(-x.num()),
    }
}
fn ew(a: &Value, b: &Value, f: fn(f64, f64) -> f64) -> Value {
    match a {
        Value::Arr(v) => Value::Arr(v.iter().enumerate().map(|(i, x)| ew(x, &b.at(i), f)).collect()),
        x => Value::Num(f(x.num(), b.num())),
    }
}
fn zero_like<'a>(a: &'a Value, other: &Value) -> Cow<'a, Value> {
    match (a, other) {
        (Value::Arr(_), _) => Cow::Borrowed(a),
        (_, Value::Arr(o)) => Cow::Owned(Value::Arr(o.iter().map(|x| zero_like(a, x).into_owned()).collect())),
        _ => Cow::Borrowed(a),
    }
}
fn sc(a: &Value, s: f64, f: fn(f64, f64) -> f64) -> Value {
    match a {
        Value::Arr(v) => Value::Arr(v.iter().map(|x| sc(x, s, f)).collect()),
        x => Value::Num(f(x.num(), s)),
    }
}

// ------------------------------------------------------------------ random streams (the toolbox's)
// adcs-sim-core rng.rs, value for value (pcode.js `rt.stream`): a stream per (seed, id) is SplitMix64 over a
// counter, a normal draw by Box-Muller with the spare kept. A stream is held as six numbers: its key's and its
// counter's high and low 32 bits, the spare, whether there is one.
pub(crate) fn splitmix64(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}
/// FNV-1a over a name's UTF-8 bytes (rng.rs stream_id), as its two 32-bit halves
pub(crate) fn fnv1a(name: &str) -> (f64, f64) {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in name.as_bytes() {
        h = (h ^ u64::from(*b)).wrapping_mul(0x100_0000_01b3);
    }
    ((h >> 32) as f64, (h & 0xFFFF_FFFF) as f64)
}
/// a stream's word: a whole number from 0 below 2^32 (anything else reads as 0)
fn u32w(x: f64) -> u64 {
    if x.is_finite() && x >= 0.0 && x < 4294967296.0 {
        x.trunc() as u64
    } else {
        0
    }
}
/// a number as JavaScript's `BigInt.asUintN(64, BigInt(Math.trunc(x)))`: its whole part modulo 2^64 (0 if not finite)
fn wrap64(x: f64) -> u64 {
    if !x.is_finite() {
        return 0;
    }
    let t = x.trunc();
    if t.abs() < 9.223372036854776e18 {
        return (t as i64) as u64;
    }
    // |t| >= 2^63: its mantissa shifted up by its exponent, modulo 2^64
    let bits = t.abs().to_bits();
    let e = ((bits >> 52) & 0x7ff) as i64 - 1075;
    let m = (bits & 0x000f_ffff_ffff_ffff) | 0x0010_0000_0000_0000;
    let u = if e >= 64 { 0 } else { m << e };
    if t < 0.0 {
        u.wrapping_neg()
    } else {
        u
    }
}
fn stream_words(key: u64, n: u64, spare: f64, has: f64) -> Value {
    arr(vec![(key >> 32) as f64, (key & 0xFFFF_FFFF) as f64, (n >> 32) as f64, (n & 0xFFFF_FFFF) as f64, spare, has])
}
fn new_stream(seed: f64, id: &Value) -> Value {
    let i = match id {
        Value::Arr(w) => (u32w(w.first().map_or(f64::NAN, Value::num)) << 32) | u32w(w.get(1).map_or(f64::NAN, Value::num)),
        v => wrap64(v.num()),
    };
    stream_words(splitmix64(wrap64(seed) ^ splitmix64(i)), 0, 0.0, 0.0)
}
/// a draw: its value and the stream advanced
fn draw(f: &str, s: &Value) -> (Value, Value) {
    let w: Vec<f64> = (0..6).map(|k| s.at(k).num()).collect();
    let uniform = |w: &[f64]| -> (f64, Vec<f64>) {
        let key = (u32w(w[0]) << 32) | u32w(w[1]);
        let n = ((u32w(w[2]) << 32) | u32w(w[3])).wrapping_add(1);
        let z = splitmix64(key ^ splitmix64(n));
        let u = ((z >> 11) as f64 + 0.5) * (1.0 / 9007199254740992.0);
        (u, vec![(key >> 32) as f64, (key & 0xFFFF_FFFF) as f64, (n >> 32) as f64, (n & 0xFFFF_FFFF) as f64, w[4], w[5]])
    };
    let normal = |w: &[f64]| -> (f64, Vec<f64>) {
        if w[5] != 0.0 {
            return (w[4], vec![w[0], w[1], w[2], w[3], 0.0, 0.0]);
        }
        let (u1, w1) = uniform(w);
        let (u2, w2) = uniform(&w1);
        // the stream is the engine's toolbox (rng.rs): its log is the libm crate's, as rng.rs's is (not V8's, which
        // the language's own log follows), so a normal draw here is rng.rs's bit for bit; the JavaScript's may differ
        // in the last bit (its log, and Node's glibc sin and cos), as any transcendental may
        let r = (-2.0 * libm::log(u1)).sqrt();
        (r * vmath::cos(2.0 * PI * u2), vec![w2[0], w2[1], w2[2], w2[3], r * vmath::sin(2.0 * PI * u2), 1.0])
    };
    match f {
        "uniform" => {
            let (u, w) = uniform(&w);
            (Value::Num(u), arr(w))
        }
        "normal" => {
            let (z, w) = normal(&w);
            (Value::Num(z), arr(w))
        }
        _ => {
            let (a, w1) = normal(&w);
            let (b, w2) = normal(&w1);
            let (c, w3) = normal(&w2);
            (arr(vec![a, b, c]), arr(w3))
        }
    }
}

/// integer bit operations on non-negative ints below 2^53, exact
fn bitop(f: &str, a: f64, b: f64, line: u32) -> R<f64> {
    let two53 = 9007199254740992.0;
    let whole = |x: f64| x.is_finite() && x.trunc() == x;
    if !(whole(a) && whole(b) && a >= 0.0 && b >= 0.0 && a < two53 && b < two53) {
        return Err(RunError(format!("{f}({}, {}): bit operations take non-negative ints below 2^53 (line {line})", jsfmt::num(a), jsfmt::num(b))));
    }
    match f {
        "shl" => {
            let r = a * vmath::pow(2.0, b);
            if r >= two53 {
                return Err(RunError(format!("shl({}, {}) passes 2^53 (line {line})", jsfmt::num(a), jsfmt::num(b))));
            }
            Ok(r)
        }
        "shr" => Ok((a / vmath::pow(2.0, b)).floor()),
        "band" => Ok(((a as u64) & (b as u64)) as f64),
        "bor" => Ok(((a as u64) | (b as u64)) as f64),
        _ => Ok(((a as u64) ^ (b as u64)) as f64),
    }
}

pub(crate) fn zero_of(c: &Checked, t: &Ty) -> Value {
    match t {
        Ty::Int | Ty::Real(_) | Ty::Tuple(_) | Ty::Choice(_) | Ty::Str => Value::Num(0.0),
        Ty::Stream => arr(vec![0.0; 6]),
        Ty::Bool => Value::Bool(false),
        Ty::Arr(n, of) => Value::Arr((0..*n).map(|_| zero_of(c, of)).collect()),
        // a buffer is the caller's: it has no zero of its own (as the JavaScript's zeroOf, 0)
        Ty::Buf(_) => Value::Num(0.0),
        Ty::Rec(name) => match c.records.iter().find(|r| &r.name == name) {
            Some(r) => Value::Rec(r.fields.iter().map(|f| zero_of(c, &f.ty)).collect()),
            None => Value::Num(0.0),
        },
    }
}

/// The interpreter over a checked program; it keeps the constants it has evaluated.
pub(crate) struct Interp {
    pub c: Checked,
    consts: Vec<OnceCell<Value>>,
    busy: Vec<Cell<bool>>,
    /// each data table's value: nested arrays of its SI values
    data: Vec<Value>,
}

/// A data table's value from its SI values (the outer index first) and its type.
pub(crate) fn data_value(d: &Data) -> Value {
    let si = d.si.as_deref().unwrap_or(&[]);
    match &d.ty {
        Ty::Arr(n, of) => match &**of {
            Ty::Arr(m, _) => Value::Arr((0..*n).map(|i| arr(si.get(i * m..(i + 1) * m).unwrap_or(&[]).to_vec())).collect()),
            _ => arr(si.to_vec()),
        },
        _ => Value::Num(f64::NAN),
    }
}

enum Step {
    Index(ExprId),
    Field(usize),
}

impl Interp {
    pub(crate) fn new(c: Checked) -> Self {
        let n = c.consts.len();
        let data = c.data.iter().map(data_value).collect();
        Interp { c, consts: (0..n).map(|_| OnceCell::new()).collect(), busy: (0..n).map(|_| Cell::new(false)).collect(), data }
    }

    fn line(&self, e: ExprId) -> u32 {
        self.c.exprs[e].pos.line
    }

    fn const_val(&self, ci: usize) -> R<&Value> {
        if let Some(v) = self.consts[ci].get() {
            return Ok(v);
        }
        if self.busy[ci].get() {
            return Err(RunError(format!("const {} refers to itself", self.c.consts[ci].name)));
        }
        self.busy[ci].set(true);
        let v = self.ev(self.c.consts[ci].e, &Vec::new()).map(Cow::into_owned);
        self.busy[ci].set(false);
        let v = v?;
        let _ = self.consts[ci].set(v);
        Ok(self.consts[ci].get().unwrap())
    }

    pub(crate) fn ev<'a>(&'a self, e: ExprId, env: &'a Env) -> R<Cow<'a, Value>> {
        let ann = &self.c.ann[e];
        match &self.c.exprs[e].kind {
            ExprKind::Num { .. } => Ok(Cow::Owned(match &ann.fill {
                Some(t) => zero_of(&self.c, t),
                None => Value::Num(ann.si),
            })),
            ExprKind::Bool(b) => Ok(Cow::Owned(Value::Bool(*b))),
            ExprKind::Str(s) => {
                let (hi, lo) = fnv1a(s);
                Ok(Cow::Owned(arr(vec![hi, lo])))
            }
            ExprKind::Var(name) => match ann.var {
                VarKind::Const(ci) => Ok(Cow::Borrowed(self.const_val(ci)?)),
                VarKind::Pi => Ok(Cow::Owned(Value::Num(PI))),
                VarKind::Inf => Ok(Cow::Owned(Value::Num(f64::INFINITY))),
                VarKind::Nan => Ok(Cow::Owned(Value::Num(f64::NAN))),
                VarKind::Data(di) => Ok(Cow::Borrowed(&self.data[di])),
                _ => match env.iter().rev().find(|(n, _)| n == name) {
                    Some((_, v)) => Ok(Cow::Borrowed(v)),
                    None => Err(RunError(format!("{name} has no value yet (line {})", self.line(e)))),
                },
            },
            ExprKind::Arr { items, .. } => {
                let mut v = Vec::with_capacity(items.len());
                for &x in items {
                    v.push(self.ev(x, env)?.into_owned());
                }
                if let Some(s) = ann.scale {
                    v = v.into_iter().map(|x| Value::Num(x.num() * s)).collect();
                }
                Ok(Cow::Owned(Value::Arr(v)))
            }
            ExprKind::Index { a, i } => {
                let av = self.ev(*a, env)?;
                let iv = self.ev(*i, env)?.num();
                let len = av.len();
                if !(iv >= 0.0 && iv < len as f64) {
                    return Err(RunError(format!("index {} outside 0..{} (line {})", jsfmt::num(iv), len as i64 - 1, self.line(e))));
                }
                if iv.trunc() != iv {
                    return Err(RunError(format!("index {} is not a whole number (line {})", jsfmt::num(iv), self.line(e))));
                }
                let k = iv as usize;
                Ok(match av {
                    Cow::Borrowed(v) => v.at(k),
                    Cow::Owned(v) => Cow::Owned(v.at(k).into_owned()),
                })
            }
            ExprKind::Field { a, .. } => {
                if let Some((_, i)) = ann.choice {
                    return Ok(Cow::Owned(Value::Num(i as f64)));
                }
                let av = self.ev(*a, env)?;
                Ok(match av {
                    Cow::Borrowed(v) => v.at(ann.field),
                    Cow::Owned(v) => Cow::Owned(v.at(ann.field).into_owned()),
                })
            }
            ExprKind::Not(a) => Ok(Cow::Owned(Value::Bool(!self.ev(*a, env)?.truthy()))),
            ExprKind::Neg(a) => Ok(Cow::Owned(neg(&*self.ev(*a, env)?))),
            ExprKind::Ifx { c, a, b } => {
                if self.ev(*c, env)?.truthy() {
                    self.ev(*a, env)
                } else {
                    self.ev(*b, env)
                }
            }
            ExprKind::Bin { op, a, b } => self.bin(e, *op, *a, *b, env),
            ExprKind::Call { args, .. } => self.call(e, args, env).map(Cow::Owned),
        }
    }

    fn bin<'a>(&'a self, e: ExprId, op: BinOp, ea: ExprId, eb: ExprId, env: &'a Env) -> R<Cow<'a, Value>> {
        let ann = &self.c.ann[e];
        match op {
            BinOp::And => {
                let a = self.ev(ea, env)?;
                return if a.truthy() { self.ev(eb, env) } else { Ok(a) };
            }
            BinOp::Or => {
                let a = self.ev(ea, env)?;
                return if a.truthy() { Ok(a) } else { self.ev(eb, env) };
            }
            _ => {}
        }
        let a = self.ev(ea, env)?;
        if op == BinOp::Pow {
            return Ok(Cow::Owned(Value::Num(ipow(a.num(), ann.k))));
        }
        let b = self.ev(eb, env)?;
        let (a, b) = (&*a, &*b);
        let is_arr = |v: &Value| matches!(v, Value::Arr(_));
        let strict_eq = |a: &Value, b: &Value| match (a, b) {
            (Value::Num(x), Value::Num(y)) => x == y,
            (Value::Bool(x), Value::Bool(y)) => x == y,
            _ => false,
        };
        let v = match op {
            BinOp::Eq => Value::Bool(strict_eq(a, b)),
            BinOp::Ne => Value::Bool(!strict_eq(a, b)),
            BinOp::Lt => Value::Bool(a.num() < b.num()),
            BinOp::Le => Value::Bool(a.num() <= b.num()),
            BinOp::Gt => Value::Bool(a.num() > b.num()),
            BinOp::Ge => Value::Bool(a.num() >= b.num()),
            BinOp::Add | BinOp::Sub => {
                let f: fn(f64, f64) -> f64 = if op == BinOp::Add { |x, y| x + y } else { |x, y| x - y };
                if is_arr(a) || is_arr(b) {
                    ew(&zero_like(a, b), &zero_like(b, a), f)
                } else {
                    Value::Num(f(a.num(), b.num()))
                }
            }
            _ => match ann.shape {
                Shape::Mv => mv(a, b),
                Shape::Mm => mm(a, b),
                Shape::Vs | Shape::Ms => {
                    if op == BinOp::Div {
                        sc(a, b.num(), |x, s| x / s)
                    } else if ann.arr_left {
                        sc(a, b.num(), |x, s| x * s)
                    } else {
                        sc(b, a.num(), |x, s| s * x)
                    }
                }
                _ => {
                    if op == BinOp::Mul {
                        Value::Num(a.num() * b.num())
                    } else {
                        Value::Num(a.num() / b.num())
                    }
                }
            },
        };
        Ok(Cow::Owned(v))
    }

    fn call(&self, e: ExprId, args: &[ExprId], env: &Env) -> R<Value> {
        let target = &self.c.ann[e].target;
        if let Target::Record(ri) = target {
            return Ok(zero_of(&self.c, &Ty::Rec(self.c.records[*ri].name.clone())));
        }
        let mut av = Vec::with_capacity(args.len());
        for &a in args {
            av.push(self.ev(a, env)?.into_owned());
        }
        let line = self.line(e);
        let x = |i: usize| av.get(i).map_or(f64::NAN, Value::num);
        match target {
            Target::Builtin(f) => {
                let n = |v: f64| Ok(Value::Num(v));
                match *f {
                    "sqrt" => n(x(0).sqrt()),
                    "abs" => n(rabs(x(0))),
                    "floor" => n(x(0).floor()),
                    "ceil" => n(x(0).ceil()),
                    "round" => n(rround(x(0))),
                    "trunc" => n(x(0).trunc()),
                    "sign" => n(rsign(x(0))),
                    "sin" => n(vmath::sin(x(0))),
                    "cos" => n(vmath::cos(x(0))),
                    "tan" => n(vmath::tan(x(0))),
                    "asin" => n(vmath::asin(x(0))),
                    "acos" => n(vmath::acos(x(0))),
                    "atan" => n(vmath::atan(x(0))),
                    "exp" => n(vmath::exp(x(0))),
                    "log" => n(vmath::log(x(0))),
                    "log10" => n(vmath::log10(x(0))),
                    "log2" => n(vmath::log2(x(0))),
                    "erf" => n(vmath::erf(x(0))),
                    "atan2" => n(vmath::atan2(x(0), x(1))),
                    "hypot" => n(vmath::hypot(x(0), x(1))),
                    "fmod" => n(fmod(x(0), x(1))),
                    "pow" => n(vmath::pow(x(0), x(1))),
                    "min" => n(av.iter().map(Value::num).reduce(rmin).unwrap_or(f64::NAN)),
                    "max" => n(av.iter().map(Value::num).reduce(rmax).unwrap_or(f64::NAN)),
                    "clamp" => n(rclamp(x(0), x(1), x(2))),
                    "real" => Ok(av.into_iter().next().unwrap_or(Value::Num(f64::NAN))),
                    "isnan" => Ok(Value::Bool(x(0).is_nan())),
                    "isfinite" => Ok(Value::Bool(x(0).is_finite())),
                    "int" => {
                        let v = x(0).trunc();
                        if !(v.abs() < 9007199254740992.0) {
                            return Err(RunError(format!("int({}): not a finite number below 2^53 (line {line})", jsfmt::num(x(0)))));
                        }
                        n(if v == 0.0 { 0.0 } else { v })
                    }
                    "band" | "bor" | "bxor" | "shl" | "shr" => n(bitop(f, x(0), x(1), line)?),
                    "div" => {
                        if x(1) == 0.0 {
                            return Err(RunError(format!("div by zero (line {line})")));
                        }
                        n((x(0) / x(1)).trunc())
                    }
                    "rem" => {
                        if x(1) == 0.0 {
                            return Err(RunError(format!("rem by zero (line {line})")));
                        }
                        n(x(0) % x(1))
                    }
                    "len" => n(av.first().map_or(0, Value::len) as f64),
                    "dot" => n(dot(&nums(&av[0]), &nums(&av[1]))),
                    "cross" => Ok(arr(cross(&nums(&av[0]), &nums(&av[1])))),
                    "norm" => n(norm(&nums(&av[0]))),
                    "unit" => Ok(arr(unit(&nums(&av[0])))),
                    "transpose" => Ok(tr(&av[0])),
                    "argsort" => Ok(arr(argsort(&nums(&av[0])).into_iter().map(|i| i as f64).collect())),
                    "sort" => Ok(Value::Arr(argsort(&nums(&av[0])).into_iter().map(|i| av[0].at(i).into_owned()).collect())),
                    "eig" => {
                        let rows: Vec<Vec<f64>> = match &av[0] {
                            Value::Arr(r) => r.iter().map(nums).collect(),
                            _ => Vec::new(),
                        };
                        Ok(Value::Arr(eig(&rows).into_iter().map(|e| arr(e.to_vec())).collect()))
                    }
                    "stream" => Ok(new_stream(x(0), av.get(1).unwrap_or(&Value::Num(f64::NAN)))),
                    "uniform" | "normal" | "normal3" => Ok(draw(f, &av[0]).0),
                    _ => Err(RunError(format!("no builtin {f}"))),
                }
            }
            Target::Table(ti) => {
                let tab = &self.c.tables[*ti];
                let r = lookup(tab, x(0));
                Ok(if tab.outs.len() == 1 { Value::Num(r[0]) } else { arr(r) })
            }
            Target::Fn(fi) => {
                let mut r = self.run_fn(*fi, av, None)?;
                r.truncate(self.c.fns[*fi].outs.len());
                Ok(if self.c.fns[*fi].outs.len() == 1 { r.swap_remove(0) } else { Value::Arr(r) })
            }
            _ => Err(RunError(format!("cannot evaluate a call of {}", self.c.exprs[e].pos.line))),
        }
    }

    /// The right side of a let or an assignment: a call that changes its inout inputs gives their values back
    /// into the caller's variables (copy in, copy out: no other input names them), before the names are set.
    fn ev_stmt(&self, e: ExprId, env: &mut Env) -> R<Value> {
        if !self.c.ann[e].inout {
            return Ok(self.ev(e, env)?.into_owned());
        }
        // a draw: the stream drawn from goes back into the caller's variable
        if let (ExprKind::Call { args, .. }, Target::Builtin(f)) = (&self.c.exprs[e].kind, &self.c.ann[e].target) {
            let s = self.ev(args[0], env)?.into_owned();
            let (v, s) = draw(f, &s);
            if let ExprKind::Var(name) = &self.c.exprs[args[0]].kind {
                match env.iter().rposition(|(n, _)| n == name) {
                    Some(slot) => env[slot].1 = s,
                    None => env.push((name.clone(), s)),
                }
            }
            return Ok(v);
        }
        let (ExprKind::Call { args, .. }, Target::Fn(fi)) = (&self.c.exprs[e].kind, &self.c.ann[e].target) else {
            return Ok(self.ev(e, env)?.into_owned());
        };
        let mut av = Vec::with_capacity(args.len());
        for &a in args {
            av.push(self.ev(a, env)?.into_owned());
        }
        let f = &self.c.fns[*fi];
        let mut r = self.run_fn(*fi, av, None)?;
        let inouts = r.split_off(f.outs.len());
        let mut k = 0;
        for (p, &a) in f.params.iter().zip(args) {
            if !p.inout {
                continue;
            }
            if let ExprKind::Var(name) = &self.c.exprs[a].kind {
                let v = inouts.get(k).cloned().unwrap_or(Value::Num(f64::NAN));
                match env.iter().rposition(|(n, _)| n == name) {
                    Some(slot) => env[slot].1 = v,
                    None => env.push((name.clone(), v)),
                }
            }
            k += 1;
        }
        Ok(if f.outs.len() == 1 { r.swap_remove(0) } else { Value::Arr(r) })
    }

    fn exec(&self, stmts: &[Stmt], env: &mut Env) -> R<()> {
        let mark = env.len();
        for s in stmts {
            match &s.kind {
                StmtKind::Let { names, e, .. } => {
                    let v = self.ev_stmt(*e, env)?;
                    if names.len() > 1 {
                        for (i, n) in names.iter().enumerate() {
                            let vi = v.at(i).into_owned();
                            env.push((n.clone(), vi));
                        }
                    } else {
                        env.push((names[0].clone(), v));
                    }
                }
                // a state's value is in the environment from the start of the call
                StmtKind::State { .. } => {}
                StmtKind::Set { targets, e } => {
                    let v = self.ev_stmt(*e, env)?;
                    if targets.len() > 1 {
                        for (i, lv) in targets.iter().enumerate() {
                            self.assign(*lv, v.at(i).into_owned(), env)?;
                        }
                    } else {
                        self.assign(targets[0], v, env)?;
                    }
                }
                StmtKind::If { arms, els } => {
                    let mut done = false;
                    for (c, body) in arms {
                        if self.ev(*c, env)?.truthy() {
                            self.exec(body, env)?;
                            done = true;
                            break;
                        }
                    }
                    if !done {
                        if let Some(els) = els {
                            self.exec(els, env)?;
                        }
                    }
                }
                StmtKind::For { v, a, b, body } => {
                    let a = self.ev(*a, env)?.num();
                    let b = self.ev(*b, env)?.num();
                    let mut i = a;
                    while i < b {
                        env.push((v.clone(), Value::Num(i)));
                        let r = self.exec(body, env);
                        env.pop();
                        r?;
                        i += 1.0;
                    }
                }
                StmtKind::Settle { n, c, body, els } => {
                    let n = self.ev(*n, env)?.num();
                    let mut k = 0.0;
                    loop {
                        self.exec(body, env)?;
                        k += 1.0;
                        if self.ev(*c, env)?.truthy() {
                            break;
                        }
                        if k >= n {
                            if let Some(els) = els {
                                self.exec(els, env)?;
                            }
                            break;
                        }
                    }
                }
            }
        }
        env.truncate(mark);
        Ok(())
    }

    fn assign(&self, lv: ExprId, v: Value, env: &mut Env) -> R<()> {
        let mut steps = Vec::new();
        let mut x = lv;
        loop {
            match &self.c.exprs[x].kind {
                ExprKind::Index { a, i } => {
                    steps.push((Step::Index(*i), x));
                    x = *a;
                }
                ExprKind::Field { a, .. } => {
                    steps.push((Step::Field(self.c.ann[x].field), x));
                    x = *a;
                }
                _ => break,
            }
        }
        steps.reverse();
        let ExprKind::Var(root) = &self.c.exprs[x].kind else { return Err(RunError("cannot assign".into())) };
        let Some(slot) = env.iter().rposition(|(n, _)| n == root) else {
            if steps.is_empty() {
                env.push((root.clone(), v));
                return Ok(());
            }
            return Err(RunError(format!("{root} has no value yet (line {})", self.line(x))));
        };
        // the places, evaluated from the root before anything changes
        let mut keys = Vec::with_capacity(steps.len());
        {
            let mut o: &Value = &env[slot].1;
            for (si, (step, sx)) in steps.iter().enumerate() {
                let k = match step {
                    Step::Index(ie) => {
                        let key = self.ev(*ie, env)?.num();
                        if !(key >= 0.0 && key < o.len() as f64) {
                            return Err(RunError(format!("index {} outside 0..{} (line {})", jsfmt::num(key), o.len() as i64 - 1, self.line(*sx))));
                        }
                        if key.trunc() != key {
                            return Err(RunError(format!("index {} is not a whole number (line {})", jsfmt::num(key), self.line(*sx))));
                        }
                        key as usize
                    }
                    Step::Field(fi) => *fi,
                };
                keys.push(k);
                if si + 1 < steps.len() {
                    o = match o {
                        Value::Arr(v) | Value::Rec(v) if k < v.len() => &v[k],
                        _ => return Err(RunError(format!("cannot assign into this value (line {})", self.line(*sx)))),
                    };
                }
            }
        }
        let mut o = &mut env[slot].1;
        for (si, k) in keys.iter().enumerate() {
            let place = match o {
                Value::Arr(v) | Value::Rec(v) if *k < v.len() => &mut v[*k],
                _ => return Err(RunError(format!("cannot assign into this value (line {})", self.line(lv)))),
            };
            if si + 1 == keys.len() {
                *place = v;
                return Ok(());
            }
            o = place;
        }
        env[slot].1 = v;
        Ok(())
    }

    pub(crate) fn run_fn(&self, fi: usize, args: Vec<Value>, state: Option<&mut State>) -> R<Vec<Value>> {
        let f = &self.c.fns[fi];
        let mut env: Env = Vec::with_capacity(f.params.len() + f.outs.len() + 8);
        for (p, a) in f.params.iter().zip(args) {
            env.push((p.name.clone(), a));
        }
        for o in &f.outs {
            env.push((o.name.clone(), zero_of(&self.c, &o.ty)));
        }
        let mut fresh;
        let st = if f.kind == FnKind::Proc {
            let st = match state {
                Some(s) => s,
                None => {
                    fresh = self.new_state(fi)?;
                    &mut fresh
                }
            };
            for (name, v) in &st.vals {
                env.push((name.clone(), v.clone()));
            }
            Some(st)
        } else {
            None
        };
        self.exec(&f.body, &mut env)?;
        if let Some(st) = st {
            for (name, v) in st.vals.iter_mut() {
                if let Some((_, x)) = env.iter().rev().find(|(n, _)| n == name) {
                    *v = x.clone();
                }
            }
        }
        // the outputs, then each inout input's value after the call
        let find = |n: &str| env.iter().rev().find(|(m, _)| m == n).map(|(_, v)| v.clone()).unwrap_or(Value::Num(f64::NAN));
        Ok(f.outs.iter().map(|o| find(&o.name)).chain(f.params.iter().filter(|p| p.inout).map(|p| find(&p.name))).collect())
    }

    /// A constant expression's value, as the JavaScript's `constOf` evaluates one (a const's value,
    /// a proc's starting state): in an empty environment.
    pub(crate) fn eval_closed(&self, e: ExprId) -> R<Value> {
        Ok(self.ev(e, &Vec::new())?.into_owned())
    }

    pub(crate) fn new_state(&self, fi: usize) -> R<State> {
        let f = &self.c.fns[fi];
        let mut vals = Vec::new();
        for (name, e) in &f.states {
            vals.push((name.clone(), self.ev(*e, &Vec::new())?.into_owned()));
        }
        Ok(State { func: fi, vals })
    }
}
