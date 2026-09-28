//! NAIF DAF/SPK reader for Chebyshev (type 2) segments -- the pure-Rust port of
//! `matlab_sils/pop/03_frames_time/ephemeris/+de440/open.m`, `state.m` and
//! `chebval.m` (identical to `private/chebval.m`).
//!
//! The kernel is parsed once into a flat word array (DAF word `W`, 1-based as in
//! MATLAB's `D(W)`, lives at `words[W-1]`), plus a segment table. Evaluating a
//! state indexes the record straight out of that array: no allocation, no copy.
//! Segments can be resolved once with [`Kernel::segment`] and evaluated by index
//! with [`Kernel::state_seg`] so the hot path skips the lookup entirely. The whole
//! kernel stays resident (~33 MB for de440s) and each segment's INIT/INTLEN/RSIZE/N
//! footer is pre-read, so a record is found by one division and sliced in place --
//! every record is "cached"; the Chebyshev basis is built once per call and shared
//! by X, Y and Z. `Kernel` is `Send + Sync` (no interior mutability).
//!
//! Frame ICRF, time TDB seconds past J2000, units km and km/s (as the kernel).

use crate::la::V3;
use std::fmt;
use std::path::Path;

/// Errors from opening / parsing a DAF/SPK kernel (`de440:open` assertions).
#[derive(Debug)]
pub enum SpkError {
    /// The file could not be read (`cannot open kernel`).
    Io(std::io::Error),
    /// The file is not a DAF/SPK (`not a DAF/SPK file`) or is truncated/corrupt.
    Format(String),
}

impl fmt::Display for SpkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpkError::Io(e) => write!(f, "de440:open: cannot open kernel: {e}"),
            SpkError::Format(s) => write!(f, "de440:open: {s}"),
        }
    }
}
impl std::error::Error for SpkError {}
impl From<std::io::Error> for SpkError {
    fn from(e: std::io::Error) -> Self { SpkError::Io(e) }
}

/// One array summary of the kernel (`de440.open`: `keys{i}` = "ctr_tgt",
/// `seg(i,:)` = [SA EA]) with the type-2 footer (`de440.state`: INIT, INTLEN,
/// RSIZE, N read from words EA-3..EA).
#[derive(Debug, Clone)]
pub struct Segment {
    /// NAIF id of the target body.
    pub target: i32,
    /// NAIF id of the center body.
    pub center: i32,
    /// Reference frame id (1 = J2000/ICRF in DE kernels).
    pub frame: i32,
    /// SPK data type (2 = Chebyshev position only).
    pub data_type: i32,
    /// Start / end epoch of the segment [TDB s past J2000] (the summary doubles).
    pub start_et: f64,
    /// See `start_et`.
    pub end_et: f64,
    /// First word address of the segment data (1-based, as MATLAB `sa`).
    pub sa: usize,
    /// Last word address of the segment data (1-based, as MATLAB `ea`).
    pub ea: usize,
    /// Epoch of the first record [s] (footer INIT).
    pub init: f64,
    /// Length of each record's interval [s] (footer INTLEN).
    pub intlen: f64,
    /// Words per record (footer RSIZE).
    pub rsize: usize,
    /// Number of records (footer N).
    pub n: usize,
    /// Chebyshev coefficients per component, (RSIZE-2)/3.
    pub ncoef: usize,
}

/// A parsed DAF/SPK kernel (`de440.open` output struct: `.D`, `.keys`, `.seg`).
#[derive(Debug, Clone)]
pub struct Kernel {
    words: Vec<f64>,
    segments: Vec<Segment>,
}

/// Largest Chebyshev degree supported without allocation (DE440 uses 8..14).
pub const MAX_COEF: usize = 64;

fn rd_i32(b: &[u8], off: usize, big: bool) -> i32 {
    let a = [b[off], b[off + 1], b[off + 2], b[off + 3]];
    if big { i32::from_be_bytes(a) } else { i32::from_le_bytes(a) }
}

impl Kernel {
    /// Load a kernel from disk (`de440.open(bspPath)`).
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Kernel, SpkError> {
        let bytes = std::fs::read(path)?;
        Kernel::from_bytes(bytes)
    }

    /// Parse a kernel held in memory (`de440.open` after its `fread`). Handles both
    /// `LTL-IEEE` and `BIG-IEEE` files (MATLAB `swapbytes` branch).
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Kernel, SpkError> {
        if bytes.len() < 1024 || !bytes.starts_with(b"DAF/SPK") {
            return Err(SpkError::Format("not a DAF/SPK file".into()));
        }
        let locfmt = String::from_utf8_lossy(&bytes[88..96]).trim().to_string();
        let big = locfmt == "BIG-IEEE";
        let nd = rd_i32(&bytes, 8, big);
        let ni = rd_i32(&bytes, 12, big);
        let fward = rd_i32(&bytes, 76, big);
        if !(0..=124).contains(&nd) || !(2..=250).contains(&ni) || nd < 2 || ni < 6 {
            return Err(SpkError::Format(format!("bad ND/NI {nd}/{ni}")));
        }
        let (nd, ni) = (nd as usize, ni as usize);
        let nw = bytes.len() / 8;
        let mut words = Vec::with_capacity(nw);
        for w in 0..nw {
            let mut a = [0u8; 8];
            a.copy_from_slice(&bytes[8 * w..8 * w + 8]);
            words.push(if big { f64::from_be_bytes(a) } else { f64::from_le_bytes(a) });
        }
        // integer view of word W's 8 bytes (typecast(summ(nd+1:ss),'int32'))
        let int_at = |word1: usize, k: usize| -> i32 { rd_i32(&bytes, (word1 - 1) * 8 + 4 * k, big) };
        let ss = nd + ni.div_ceil(2);
        let mut segments = Vec::new();
        let mut recno = fward as i64;
        let mut guard = 0usize;
        while recno != 0 {
            guard += 1;
            if recno < 0 || guard > nw / 128 + 1 {
                return Err(SpkError::Format("corrupt summary record chain".into()));
            }
            let base = (recno as usize - 1) * 128; // record start, in words
            if base + 128 > nw {
                return Err(SpkError::Format("summary record beyond end of file".into()));
            }
            let nsum = words[base + 2] as usize;
            for s in 0..nsum {
                let off = base + 3 + s * ss; // summary at words off+1 .. off+ss
                let iw = off + nd + 1; // first integer word (1-based)
                let ints: Vec<i32> = (0..ni).map(|k| int_at(iw, k)).collect();
                let (tgt, ctr, frame, typ) = (ints[0], ints[1], ints[2], ints[3]);
                let (sa, ea) = (ints[4] as usize, ints[5] as usize);
                if sa < 1 || ea > nw || ea < sa + 3 {
                    return Err(SpkError::Format(format!("segment {ctr}->{tgt} out of range")));
                }
                let foot = &words[ea - 4..ea];
                let (init, intlen, rsize, n) = (foot[0], foot[1], foot[2] as usize, foot[3] as usize);
                let ncoef = if rsize >= 2 { (rsize - 2) / 3 } else { 0 };
                if typ == 2 && (ncoef == 0 || ncoef > MAX_COEF || n == 0 || sa - 1 + n * rsize > nw) {
                    return Err(SpkError::Format(format!("bad type-2 footer in segment {ctr}->{tgt}")));
                }
                segments.push(Segment {
                    target: tgt, center: ctr, frame, data_type: typ,
                    start_et: words[off], end_et: words[off + 1],
                    sa, ea, init, intlen, rsize, n, ncoef,
                });
            }
            recno = words[base] as i64; // NEXT record
        }
        Ok(Kernel { words, segments })
    }

    /// All segments in file order (`eph.keys` / `eph.seg`).
    pub fn segments(&self) -> &[Segment] { &self.segments }

    /// Flat word view (`eph.D`); word `W` (1-based) is `words()[W-1]`.
    pub fn words(&self) -> &[f64] { &self.words }

    /// Index of the first segment `center -> target` (`find(strcmp(eph.keys,key),1)`).
    pub fn segment(&self, center: i32, target: i32) -> Option<usize> {
        self.segments.iter().position(|s| s.center == center && s.target == target)
    }

    /// Position [km] and velocity [km/s] of `target` w.r.t. `center` at `et` TDB
    /// seconds past J2000 (`de440.state` below its `jdTDB -> et` line). `None` if the
    /// segment is absent (MATLAB asserts).
    pub fn state(&self, center: i32, target: i32, et: f64) -> Option<(V3, V3)> {
        self.segment(center, target).map(|i| self.state_seg(i, et))
    }

    /// As [`Kernel::state`] for a pre-resolved segment index (hot path).
    /// Record selection clamps to the first/last record exactly as `de440.state`.
    pub fn state_seg(&self, seg: usize, et: f64) -> (V3, V3) {
        let s = &self.segments[seg];
        let mut k = ((et - s.init) / s.intlen).floor();
        let nm1 = (s.n - 1) as f64;
        if k < 0.0 || k.is_nan() { k = 0.0; }
        if k > nm1 { k = nm1; }
        let k = k as usize;
        let start = s.sa - 1 + k * s.rsize; // 0-based index of word sa + k*rsize
        let rec = &self.words[start..start + s.rsize];
        let (mid, radius) = (rec[0], rec[1]);
        let tau = (et - mid) / radius;
        let nc = s.ncoef;
        let mut t = [0.0f64; MAX_COEF];
        let mut dt = [0.0f64; MAX_COEF];
        cheb_basis(tau, &mut t[..nc], &mut dt[..nc]);
        let mut pos = [0.0; 3];
        let mut vel = [0.0; 3];
        for c in 0..3 {
            let coef = &rec[2 + c * nc..2 + (c + 1) * nc];
            let (p, v) = chebval_basis(coef, &t[..nc], &dt[..nc], radius);
            pos[c] = p;
            vel[c] = v;
        }
        (pos, vel)
    }
}

/// Chebyshev polynomials T_i(x) and their derivatives dT_i/dx (the two recurrences
/// of `de440.chebval`). `t` and `dt` must have equal length n >= 1.
pub fn cheb_basis(x: f64, t: &mut [f64], dt: &mut [f64]) {
    let n = t.len();
    t[0] = 1.0;
    if n > 1 { t[1] = x; }
    for i in 2..n { t[i] = 2.0 * x * t[i - 1] - t[i - 2]; }
    dt[0] = 0.0;
    if n > 1 { dt[1] = 1.0; }
    if n > 2 { dt[2] = 4.0 * x; }
    for i in 3..n { dt[i] = 2.0 * x * dt[i - 1] + 2.0 * t[i - 1] - dt[i - 2]; }
}

fn chebval_basis(c: &[f64], t: &[f64], dt: &[f64], radius: f64) -> (f64, f64) {
    let mut val = 0.0;
    let mut der = 0.0;
    for i in 0..c.len() {
        val += c[i] * t[i];
        der += c[i] * dt[i];
    }
    (val, der / radius)
}

/// Chebyshev series value and derivative at `x` in [-1,1]; the derivative is divided
/// by `radius` (per second for a type-2 record half-length). Port of `de440.chebval`.
/// Series longer than [`MAX_COEF`] allocate.
pub fn chebval(c: &[f64], x: f64, radius: f64) -> (f64, f64) {
    let n = c.len();
    if n == 0 { return (0.0, 0.0); }
    if n <= MAX_COEF {
        let mut t = [0.0f64; MAX_COEF];
        let mut dt = [0.0f64; MAX_COEF];
        cheb_basis(x, &mut t[..n], &mut dt[..n]);
        chebval_basis(c, &t[..n], &dt[..n], radius)
    } else {
        let mut t = vec![0.0; n];
        let mut dt = vec![0.0; n];
        cheb_basis(x, &mut t, &mut dt);
        chebval_basis(c, &t, &dt, radius)
    }
}
