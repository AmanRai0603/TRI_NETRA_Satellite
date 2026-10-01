//! Sensor truth models (asils.devices.*): descriptors from the product, a
//! dispersed unit drawn once per run, and per-sample noise. Engineering values
//! out; `emu` turns them into the bytes the flight drivers read.
use crate::la::*;
use crate::pm::*;
use crate::rng::Rng;
use crate::{NH, NS};

fn mis(r: &mut Rng, s: f64) -> M3 { let e = r.normal3(); small_rot(&scale(&e, s)) }
fn sf(r: &mut Rng, s: f64) -> M3 { diag(&[1.0 + s*r.normal(), 1.0 + s*r.normal(), 1.0 + s*r.normal()]) }

// ---------------- gyro ----------------
#[derive(Clone, Copy, Debug, Default)]
pub struct GyroDesc { pub fitted: bool, pub arw: f64, pub rrw: f64, pub range: f64, pub bias_sigma: f64, pub sf_sigma: f64, pub misalign: f64 }
#[derive(Clone, Copy, Debug, Default)]
pub struct Gyro { pub d: GyroDesc, pub m: M3, pub b: V3, pub brw: V3, pub rng: Rng }
impl Gyro {
    pub fn new(d: GyroDesc, disp: &mut Rng, noise: Rng) -> Gyro {
        let m = mm(&mis(disp, d.misalign), &sf(disp, d.sf_sigma));
        let b = scale(&disp.normal3(), d.bias_sigma);
        Gyro { d, m, b, brw: [0.0; 3], rng: noise }
    }
    pub fn sample(&mut self, w: &V3, dt: f64) -> V3 {
        let n1 = self.rng.normal3();
        for i in 0..3 { self.brw[i] += self.d.rrw*sqrt(dt)*n1[i]; }
        let n2 = self.rng.normal3();
        let mw = mv(&self.m, w);
        let mut o = [0.0; 3];
        for i in 0..3 { o[i] = clamp(mw[i] + self.b[i] + self.brw[i] + self.d.arw/sqrt(dt)*n2[i], -self.d.range, self.d.range); }
        o
    }
}

// ---------------- magnetometer ----------------
#[derive(Clone, Copy, Debug, Default)]
pub struct MagDesc { pub fitted: bool, pub noise: f64, pub bias_t: f64, pub bias_sigma: f64, pub range: f64, pub sf_sigma: f64, pub misalign: f64, pub k_coil: f64 }
#[derive(Clone, Copy, Debug, Default)]
pub struct Mag { pub d: MagDesc, pub m: M3, pub b: V3, pub dead: bool, pub rng: Rng }
impl Mag {
    pub fn new(d: MagDesc, disp: &mut Rng, noise: Rng) -> Mag {
        let m = mm(&mis(disp, d.misalign), &sf(disp, d.sf_sigma));
        let e = disp.normal3();
        let s3 = 1.0/sqrt(3.0);
        let b = [d.bias_t*s3 + d.bias_sigma*e[0], d.bias_t*s3 + d.bias_sigma*e[1], d.bias_t*s3 + d.bias_sigma*e[2]];
        Mag { d, m, b, dead: false, rng: noise }
    }
    /// Body field [T] with the coils' stray field (m_coil, body dipole of the coils).
    pub fn sample(&mut self, b_body: &V3, m_coil: &V3) -> V3 {
        if self.dead { return [1e-9, 0.0, 0.0]; }
        let n = self.rng.normal3();
        let mb = mv(&self.m, b_body);
        let mut o = [0.0; 3];
        for i in 0..3 { o[i] = clamp(mb[i] + self.b[i] + self.d.noise*n[i] + self.d.k_coil*m_coil[i], -self.d.range, self.d.range); }
        o
    }
}

// ---------------- fine Sun sensors ----------------
#[derive(Clone, Copy, Debug, Default)]
pub struct SunDesc { pub fitted: bool, pub n: usize, pub normals: [V3; NS], pub noise: f64, pub fov: f64, pub bias_sigma: f64 }
#[derive(Clone, Copy, Debug, Default)]
pub struct Sun { pub d: SunDesc, pub bias: [V3; NS], pub rng: Rng }
impl Sun {
    pub fn new(d: SunDesc, disp: &mut Rng, noise: Rng) -> Sun {
        let mut bias = [[0.0; 3]; NS];
        for j in 0..d.n { bias[j] = scale(&disp.normal3(), d.bias_sigma); }
        Sun { d, bias, rng: noise }
    }
    pub fn sample(&mut self, s_body: &V3, nu: f64) -> Option<V3> {
        if nu < 0.5 { return None; }
        let (mut cm, mut jm) = (-2.0, 0);
        for j in 0..self.d.n { let c = dot(s_body, &self.d.normals[j]); if c > cm { cm = c; jm = j; } }
        if cm < cos(self.d.fov) { return None; }
        let n = self.rng.normal3();
        let e = add(&self.bias[jm], &scale(&n, self.d.noise));
        Some(unit(&mv(&small_rot(&e), s_body)))
    }
}

// ---------------- coarse Sun sensors (cosine cells with Earth albedo) ----------------
#[derive(Clone, Copy, Debug, Default)]
pub struct CssDesc { pub fitted: bool, pub n: usize, pub normals: [V3; NS], pub noise: f64, pub albedo: f64, pub scale_sigma: f64, pub misalign: f64 }
#[derive(Clone, Copy, Debug, Default)]
pub struct Css { pub d: CssDesc, pub scale: [f64; NS], pub dead: [bool; NS], pub r: M3, pub rng: Rng }
impl Css {
    pub fn new(d: CssDesc, disp: &mut Rng, noise: Rng) -> Css {
        let mut s = [1.0; NS];
        for j in 0..d.n { s[j] = 1.0 + d.scale_sigma*disp.normal(); }
        let r = mis(disp, d.misalign);
        Css { d, scale: s, dead: [false; NS], r, rng: noise }
    }
    pub fn sample(&mut self, s_b: &V3, nu: f64, nadir_b: &V3, earth_ang: f64) -> Option<V3> {
        let geo = sin(earth_ang)*sin(earth_ang)*(-dot(s_b, nadir_b)).max(0.0);
        let mut iv = [0.0; NS];
        for j in 0..self.d.n {
            let nj = &self.d.normals[j];
            iv[j] = self.scale[j]*(nu*dot(nj, s_b).max(0.0) + self.d.albedo*geo*dot(nj, nadir_b).max(0.0)) + self.d.noise*self.rng.normal();
            if self.dead[j] { iv[j] = 0.0; }
        }
        let mut est = [0.0; 3];
        for ax in 0..3 {
            let (mut p, mut m) = (None, None);
            for j in 0..self.d.n {
                if p.is_none() && self.d.normals[j][ax] > 0.9 { p = Some(j); }
                if m.is_none() && self.d.normals[j][ax] < -0.9 { m = Some(j); }
            }
            if let (Some(p), Some(m)) = (p, m) { est[ax] = iv[p] - iv[m]; }
        }
        if nu > 0.5 && norm(&est) > 0.3 { Some(mv(&self.r, &unit(&est))) } else { None }
    }
}

// ---------------- star tracker ----------------
/// The synthetic 4000-star catalogue of asils.devices.star_catalogue, computed on demand.
pub fn star(k: usize, n: usize) -> (V3, f64) {
    let kf = k as f64 + 0.5;
    let nf = n as f64;
    let z = 1.0 - 2.0*kf/nf;
    let ph = PI*(1.0 + sqrt(5.0))*kf;
    let rr = sqrt(1.0 - z*z);
    let r = [rr*cos(ph), rr*sin(ph), z];
    let fr = |x: f64| x - floor(x);
    let h1 = fr(sin(kf*12.9898)*43758.5453) - 0.5;
    let h2 = fr(sin(kf*78.233)*12345.6789) - 0.5;
    let e1 = [-sin(ph), cos(ph), 0.0];
    let e2 = cross(&r, &e1);
    let dsp = 0.35*sqrt(4.0*PI/nf);
    let v = [r[0] + dsp*(h1*e1[0] + h2*e2[0]), r[1] + dsp*(h1*e1[1] + h2*e2[1]), r[2] + dsp*(h1*e1[2] + h2*e2[2])];
    (unit(&v), 1.0 + 5.0*fr(kf*sqrt(2.0) + 0.5*h1))
}
pub const N_STARS: usize = 4000;

#[derive(Clone, Copy, Debug, Default)]
pub struct StDesc {
    pub fitted: bool, pub nh: usize, pub bs: [V3; NH], pub noise_cross: f64, pub noise_roll: f64, pub rate_hz: f64,
    pub latency: f64, pub max_rate: f64, pub sun_excl: f64, pub earth_excl: f64, pub fov: f64,
    /// 0 noise model, 1 onboard QUEST on catalogue stars (the default)
    pub model: u8, pub bias_sigma: f64, pub misalign_sigma: f64,
    /// the Moon's exclusion half-angle [rad]; the time a head stays blind after the Sun or the
    /// Moon leaves its exclusion cone [s]; the body rate at which the noise has doubled [rad/s]
    /// (star smear over the exposure: sigma(w) = sigma_0 (1 + |w|/w_ref))
    pub moon_excl: f64, pub blind_s: f64, pub noise_rate_ref: f64,
}
const HIST: usize = 64;
/// Star tracker unit; holds its catalogue (the onboard star table, 4000 entries).
#[derive(Clone)]
pub struct St { pub d: StDesc, pub q_bias: [Q; NH], pub q_mis: [Q; NH], pub dead: [bool; NH], ht: [f64; HIST], hq: [Q; HIST], hn: usize, pub rng: Rng, cat: [(V3, f64); N_STARS],
    /// each head is blind until this time (the Sun or the Moon was in its exclusion cone)
    pub blind_until: [f64; NH] }
impl St {
    pub fn new(d: StDesc, disp: &mut Rng, noise: Rng) -> St {
        let mut s = St { d, q_bias: [[0.0, 0.0, 0.0, 1.0]; NH], q_mis: [[0.0, 0.0, 0.0, 1.0]; NH], dead: [false; NH], ht: [0.0; HIST], hq: [[0.0; 4]; HIST], hn: 0, rng: noise, cat: [([0.0; 3], 0.0); N_STARS],
            blind_until: [f64::NEG_INFINITY; NH] };
        if d.model == 1 { for k in 0..N_STARS { s.cat[k] = star(k, N_STARS); } }
        for h in 0..d.nh {
            s.q_bias[h] = fromrotvec(&scale(&disp.normal3(), d.bias_sigma));
            s.q_mis[h] = fromrotvec(&scale(&disp.normal3(), d.misalign_sigma));
        }
        s
    }
    /// Attitude history for the latency (asils.devices.st_history, span 0.5 s).
    pub fn history(&mut self, t: f64, q: &Q) {
        if self.hn == HIST { for i in 1..HIST { self.ht[i - 1] = self.ht[i]; self.hq[i - 1] = self.hq[i]; } self.hn -= 1; }
        self.ht[self.hn] = t; self.hq[self.hn] = *q; self.hn += 1;
        let mut k = 0;
        while k < self.hn && self.ht[k] < t - 0.5 { k += 1; }
        if k > 0 { for i in k..self.hn { self.ht[i - k] = self.ht[i]; self.hq[i - k] = self.hq[i]; } self.hn -= k; }
    }
    /// Per head: Some(q_meas) when valid. `moon_b`: the Moon's direction in the body.
    pub fn sample(&mut self, q_true: &Q, t: f64, w: &V3, sun_b: &V3, moon_b: &V3, nadir_b: &V3, earth_ang: f64) -> [Option<Q>; NH] {
        let tl = t - self.d.latency;
        let mut k = None;
        for i in 0..self.hn { if self.ht[i] <= tl + 1e-9 { k = Some(i); } }
        let q_old = match k {
            None => *q_true,
            Some(i) if i + 1 == self.hn => self.hq[i],
            Some(i) => slerp(&self.hq[i], &self.hq[i + 1], (tl - self.ht[i])/(self.ht[i + 1] - self.ht[i])),
        };
        let slow = norm(w) < self.d.max_rate;
        let smear = 1.0 + norm(w)/self.d.noise_rate_ref;
        let rold = dcm(&q_old);
        let mut out = [None; NH];
        for h in 0..self.d.nh {
            let bs = self.d.bs[h];
            // the Sun or the Moon in the exclusion cone blinds the head, and it stays blind for
            // blind_s after the body leaves it
            let dazzled = acos(clamp(dot(&bs, sun_b), -1.0, 1.0)) <= self.d.sun_excl || acos(clamp(dot(&bs, moon_b), -1.0, 1.0)) <= self.d.moon_excl;
            if dazzled { self.blind_until[h] = t + self.d.blind_s; }
            let valid = slow && !self.dead[h] && !dazzled && t >= self.blind_until[h]
                && acos(clamp(dot(&bs, nadir_b), -1.0, 1.0)) > earth_ang + self.d.earth_excl;
            let dq = qmult(&self.q_mis[h], &self.q_bias[h]);
            if self.d.model == 1 {
                let bs_eci = mtv(&rold, &bs);
                let cf = cos(self.d.fov);
                // the 12 brightest catalogue stars in the field
                let mut sel: [(usize, f64); 12] = [(0, 1e9); 12];
                let mut ns = 0;
                for s in 0..N_STARS {
                    let (r, mag) = self.cat[s];
                    if dot(&bs_eci, &r) <= cf { continue; }
                    if ns < 12 { sel[ns] = (s, mag); ns += 1; }
                    else {
                        let mut wi = 0;
                        for i in 1..12 { if sel[i].1 > sel[wi].1 { wi = i; } }
                        if mag < sel[wi].1 { sel[wi] = (s, mag); }
                    }
                }
                if ns < 3 { continue; }
                let rm = mm(&transpose(&dcm(&dq)), &rold);
                let sc = self.d.noise_cross*smear*sqrt(8.0);
                let mut bm = [[0.0; 3]; 12];
                let mut rr = [[0.0; 3]; 12];
                for i in 0..ns {
                    let r = self.cat[sel[i].0].0;
                    let v = mv(&rm, &r);
                    let e0 = scale(&self.rng.normal3(), sc);
                    let e = sub(&e0, &scale(&v, dot(&v, &e0)));
                    bm[i] = unit(&add(&v, &e));
                    rr[i] = r;
                }
                if valid { out[h] = Some(qmethod(&bm[..ns], &rr[..ns])); }
            } else {
                let n0 = self.rng.normal3();
                let nr = self.rng.normal();
                let e0 = scale(&n0, self.d.noise_cross*smear);
                let e = add(&sub(&e0, &scale(&bs, dot(&bs, &e0))), &scale(&bs, self.d.noise_roll*smear*nr));
                if valid { out[h] = Some(qnorm(&qmult(&q_old, &qmult(&dq, &fromrotvec(&e))))); }
            }
        }
        out
    }
}

// ---------------- Earth sensor ----------------
#[derive(Clone, Copy, Debug, Default)]
pub struct EsDesc { pub fitted: bool, pub bs: V3, pub noise: f64, pub fov: f64, pub rate_hz: f64, pub bias_sigma: f64 }
#[derive(Clone, Copy, Debug, Default)]
pub struct Es { pub d: EsDesc, pub bias: Q, pub rng: Rng }
impl Es {
    pub fn new(d: EsDesc, disp: &mut Rng, noise: Rng) -> Es { Es { d, bias: fromrotvec(&scale(&disp.normal3(), d.bias_sigma)), rng: noise } }
    pub fn sample(&mut self, nadir_b: &V3) -> Option<V3> {
        if acos(clamp(dot(&self.d.bs, nadir_b), -1.0, 1.0)) >= self.d.fov { return None; }
        let e = scale(&self.rng.normal3(), self.d.noise);
        Some(unit(&mv(&dcm(&qmult(&self.bias, &fromrotvec(&e))), nadir_b)))
    }
}

// ---------------- GNSS ----------------
/// `latency` [s]: a fix reaches the bus this long after the epoch it solves for, and the
/// receiver reports it as current.
#[derive(Clone, Copy, Debug, Default)]
pub struct GpsDesc { pub fitted: bool, pub pos_sigma: f64, pub vel_sigma: f64, pub rate_hz: f64, pub latency: f64 }
/// The truth states the latency reaches back over (one per loop tick).
pub const GPS_HIST: usize = 256;
#[derive(Clone, Copy, Debug)]
pub struct Gps { pub d: GpsDesc, pub dead: bool, pub rng: Rng, ht: [f64; GPS_HIST], hr: [V3; GPS_HIST], hv: [V3; GPS_HIST], hn: usize }
impl Gps {
    pub fn new(d: GpsDesc, noise: Rng) -> Gps {
        Gps { d, dead: false, rng: noise, ht: [0.0; GPS_HIST], hr: [[0.0; 3]; GPS_HIST], hv: [[0.0; 3]; GPS_HIST], hn: 0 }
    }
    /// Record the truth state at t (every tick); keep what the latency reaches back to.
    pub fn history(&mut self, t: f64, r: &V3, v: &V3) {
        if self.hn == GPS_HIST { self.drop_front(1); }
        self.ht[self.hn] = t; self.hr[self.hn] = *r; self.hv[self.hn] = *v; self.hn += 1;
        // the newest entry at or before t - latency is the oldest one still needed
        let tl = t - self.d.latency;
        let mut k = 0;
        while k + 1 < self.hn && self.ht[k + 1] <= tl + 1e-9 { k += 1; }
        if k > 0 { self.drop_front(k); }
    }
    fn drop_front(&mut self, k: usize) {
        for i in k..self.hn { self.ht[i - k] = self.ht[i]; self.hr[i - k] = self.hr[i]; self.hv[i - k] = self.hv[i]; }
        self.hn -= k;
    }
    /// (epoch, r, v): the truth state at t - latency, linear between ticks; before the history
    /// reaches back that far (the first `latency` of a run), its oldest state and that state's epoch.
    pub fn delayed(&self, t: f64) -> (f64, V3, V3) {
        let tl = t - self.d.latency;
        if self.hn == 0 { return (t, [0.0; 3], [0.0; 3]); }
        let mut k = 0;
        while k + 1 < self.hn && self.ht[k + 1] <= tl + 1e-9 { k += 1; }
        if k + 1 == self.hn || tl <= self.ht[k] { return (self.ht[k].max(tl), self.hr[k], self.hv[k]); }
        let s = (tl - self.ht[k])/(self.ht[k + 1] - self.ht[k]);
        let li = |a: &V3, b: &V3| [a[0] + s*(b[0] - a[0]), a[1] + s*(b[1] - a[1]), a[2] + s*(b[2] - a[2])];
        (tl, li(&self.hr[k], &self.hr[k + 1]), li(&self.hv[k], &self.hv[k + 1]))
    }
    pub fn sample(&mut self, r: &V3, v: &V3) -> (V3, V3) {
        let (a, b) = (self.rng.normal3(), self.rng.normal3());
        (add(r, &scale(&a, self.d.pos_sigma)), add(v, &scale(&b, self.d.vel_sigma)))
    }
}
