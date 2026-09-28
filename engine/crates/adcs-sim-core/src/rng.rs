//! Counter-based randomness (spec §9.7): a stream per (seed, stream id) is
//! SplitMix64 over a counter, so a draw never depends on evaluation order
//! elsewhere. Normal draws by Box-Muller.
use crate::pm::*;

#[inline]
pub fn splitmix64(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// FNV-1a over a stream name, for stable stream ids.
pub const fn stream_id(name: &str) -> u64 {
    let b = name.as_bytes();
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut i = 0;
    while i < b.len() { h ^= b[i] as u64; h = h.wrapping_mul(0x100_0000_01b3); i += 1; }
    h
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Rng { key: u64, n: u64, spare: Option<f64> }

impl Rng {
    pub fn new(seed: u64, stream: u64) -> Rng { Rng { key: splitmix64(seed ^ splitmix64(stream)), n: 0, spare: None } }
    pub fn next_u64(&mut self) -> u64 { self.n += 1; splitmix64(self.key ^ splitmix64(self.n)) }
    /// Uniform on (0, 1).
    pub fn uniform(&mut self) -> f64 { ((self.next_u64() >> 11) as f64 + 0.5)*(1.0/9007199254740992.0) }
    pub fn normal(&mut self) -> f64 {
        if let Some(s) = self.spare.take() { return s; }
        let (u1, u2) = (self.uniform(), self.uniform());
        let r = sqrt(-2.0*ln(u1));
        self.spare = Some(r*sin(2.0*PI*u2));
        r*cos(2.0*PI*u2)
    }
    pub fn normal3(&mut self) -> [f64; 3] { [self.normal(), self.normal(), self.normal()] }
}
