//! Sensor truth models (asils.devices.*): sens's methods, generated from the design (tools/engine_build.py): the gyro
//! (`gen::gyro`, l3_sens_row_01, over the rows' gyrobias and gyroaxes), the magnetometer (`gen::mag`, l3_sens_row_05, over
//! magnoise), the coarse Sun sensors (`gen::css`, l3_sens_row_06), the fine Sun sensors (`gen::finesun`, sens_fine_sun, over
//! the quadrant chain's sunquad and sunangles), the Earth sensor (`gen::earthsensor`, l3_sens_row_13) and the GNSS receiver
//! (`gen::gnss`, l3_sens_row_14); their random draws are the language's streams (rng.rs's, value for value). What is left
//! here is the engine's descriptors as product.rs fills them from the parts (code: the reading), handed to the generated
//! models, and the units in flight as the generated state, by the names the engine has always used. The star tracker
//! (`gen::sttracker`, sens_star_tracker, over l3_sens_row_12's q-method and sens_star_catalogue's table) likewise, and its
//! image model's heads (`gen::stimage`'s st_image, sens_star_image, over the chain l3_sens_row_09 to 12), the frame's buffers the caller's.
//! Engineering values out; `emu` turns them into the bytes the flight drivers read.
use crate::la::*;
use crate::comp::{star_tracker as stc, sun_sensor as ssc};
use crate::gen::{css, earthsensor, finesun, gnss, gyro, mag, starcat, stimage, sttracker};
use crate::rng::Rng;
use crate::{NH, NS};

/// A generated unit from a description: the dispersion stream handed over and taken back (its draws rng.rs's).
fn drawn<U>(disp: &mut Rng, f: impl FnOnce(&mut crate::gen::rt::Stream) -> U) -> U {
    let mut g = disp.stream();
    let u = f(&mut g);
    *disp = Rng::from_stream(&g);
    u
}

/// A unit in flight: the generated state, read and written through `Deref`, beside the engine's descriptor.
macro_rules! unit {
    ($name:ident, $desc:ident, $gdesc:ty, $state:ty) => {
        #[derive(Clone, Copy, Debug, Default)]
        pub struct $name { pub d: $desc, dg: $gdesc, s: $state }
        impl core::ops::Deref for $name { type Target = $state; fn deref(&self) -> &$state { &self.s } }
        impl core::ops::DerefMut for $name { fn deref_mut(&mut self) -> &mut $state { &mut self.s } }
    };
}
fn opt(r: (bool, V3)) -> Option<V3> { if r.0 { Some(r.1) } else { None } }

// ---------------- gyro ----------------
#[derive(Clone, Copy, Debug, Default)]
pub struct GyroDesc { pub fitted: bool, pub arw: f64, pub rrw: f64, pub range: f64, pub bias_sigma: f64, pub sf_sigma: f64, pub misalign: f64 }
impl GyroDesc {
    /// The design's record of the gyro.
    pub fn rec(&self) -> gyro::GyroDesc {
        gyro::GyroDesc { arw: self.arw, rrw: self.rrw, range: self.range, bias_sigma: self.bias_sigma, sf_sigma: self.sf_sigma, misalign: self.misalign }
    }
}
unit!(Gyro, GyroDesc, gyro::GyroDesc, gyro::GyroUnit);
impl Gyro {
    pub fn new(d: GyroDesc, disp: &mut Rng, noise: Rng) -> Gyro {
        let dg = d.rec();
        Gyro { d, dg, s: drawn(disp, |g| gyro::gyro_new(dg, g, noise.stream())) }
    }
    pub fn sample(&mut self, w: &V3, dt: f64) -> V3 { gyro::gyro_sample(&mut self.s, self.dg, *w, dt) }
}

// ---------------- magnetometer ----------------
#[derive(Clone, Copy, Debug, Default)]
pub struct MagDesc { pub fitted: bool, pub noise: f64, pub bias_t: f64, pub bias_sigma: f64, pub range: f64, pub sf_sigma: f64, pub misalign: f64, pub k_coil: f64 }
impl MagDesc {
    /// The design's record of the magnetometer.
    pub fn rec(&self) -> mag::MagDesc {
        mag::MagDesc { noise: self.noise, bias_t: self.bias_t, bias_sigma: self.bias_sigma, range: self.range, sf_sigma: self.sf_sigma,
                       misalign: self.misalign, k_coil: self.k_coil }
    }
}
unit!(Mag, MagDesc, mag::MagDesc, mag::MagUnit);
impl Mag {
    pub fn new(d: MagDesc, disp: &mut Rng, noise: Rng) -> Mag {
        let dg = d.rec();
        Mag { d, dg, s: drawn(disp, |g| mag::mag_new(dg, g, noise.stream())) }
    }
    /// Body field [T] with the coils' stray field (m_coil, body dipole of the coils).
    pub fn sample(&mut self, b_body: &V3, m_coil: &V3) -> V3 { mag::mag_sample(&mut self.s, self.dg, *b_body, *m_coil) }
}

// ---------------- fine Sun sensors ----------------
/// `chain`: the component level (the product's `level = "chain"`): the quadrant currents of
/// `head` and back to the Sun direction (comp::sun_sensor) in place of the noise model.
#[derive(Clone, Copy, Debug, Default)]
pub struct SunDesc { pub fitted: bool, pub n: usize, pub normals: [V3; NS], pub noise: f64, pub fov: f64, pub bias_sigma: f64,
    pub chain: bool, pub head: ssc::Head }
impl SunDesc {
    /// The design's record of the fine Sun sensors.
    pub fn rec(&self) -> finesun::SunDesc {
        finesun::SunDesc { n: self.n as i64, normals: self.normals, noise: self.noise, fov: self.fov, bias_sigma: self.bias_sigma, chain: self.chain, head: self.head }
    }
}
unit!(Sun, SunDesc, finesun::SunDesc, finesun::SunUnit);
impl Sun {
    pub fn new(d: SunDesc, disp: &mut Rng, noise: Rng) -> Sun {
        let dg = d.rec();
        Sun { d, dg, s: drawn(disp, |g| finesun::sun_new(dg, g, noise.stream())) }
    }
    pub fn sample(&mut self, s_body: &V3, nu: f64) -> Option<V3> { opt(finesun::sun_sample(&mut self.s, self.dg, *s_body, nu)) }
}

// ---------------- coarse Sun sensors (cosine cells with Earth albedo) ----------------
#[derive(Clone, Copy, Debug, Default)]
pub struct CssDesc { pub fitted: bool, pub n: usize, pub normals: [V3; NS], pub noise: f64, pub albedo: f64, pub scale_sigma: f64, pub misalign: f64 }
impl CssDesc {
    /// The design's record of the coarse Sun sensors.
    pub fn rec(&self) -> css::CssDesc {
        css::CssDesc { n: self.n as i64, normals: self.normals, noise: self.noise, albedo: self.albedo, scale_sigma: self.scale_sigma, misalign: self.misalign }
    }
}
unit!(Css, CssDesc, css::CssDesc, css::CssUnit);
impl Css {
    pub fn new(d: CssDesc, disp: &mut Rng, noise: Rng) -> Css {
        let dg = d.rec();
        Css { d, dg, s: drawn(disp, |g| css::css_new(dg, g, noise.stream())) }
    }
    pub fn sample(&mut self, s_b: &V3, nu: f64, nadir_b: &V3, earth_ang: f64) -> Option<V3> {
        opt(css::css_sample(&mut self.s, self.dg, *s_b, nu, *nadir_b, earth_ang))
    }
}

// ---------------- star tracker ----------------
/// Star k of the synthetic 4000-star catalogue (asils.devices.star_catalogue): sens_star_catalogue's `star_entry`.
pub fn star(k: usize, n: usize) -> (V3, f64) { starcat::star_entry(k as i64, n as i64) }
pub const N_STARS: usize = starcat::N_STARS as usize;

#[derive(Clone, Copy, Debug, Default)]
pub struct StDesc {
    pub fitted: bool, pub nh: usize, pub bs: [V3; NH], pub noise_cross: f64, pub noise_roll: f64, pub rate_hz: f64,
    pub latency: f64, pub max_rate: f64, pub sun_excl: f64, pub earth_excl: f64, pub fov: f64,
    /// 0 noise model, 1 onboard QUEST on catalogue stars (the default), 2 the rendered-frame
    /// chain (comp::star_tracker; needs a [`stc::Work`] per frame, see [`St::sample_with`])
    pub model: u8, pub bias_sigma: f64, pub misalign_sigma: f64,
    /// the Moon's exclusion half-angle [rad]; the time a head stays blind after the Sun or the
    /// Moon leaves its exclusion cone [s]; the body rate at which the noise has doubled [rad/s]
    /// (star smear over the exposure: sigma(w) = sigma_0 (1 + |w|/w_ref))
    pub moon_excl: f64, pub blind_s: f64, pub noise_rate_ref: f64,
    /// the camera and onboard chain of model 2, from the part
    pub cam: stc::Camera,
}
impl StDesc {
    /// The design's record of the star tracker.
    pub fn rec(&self) -> sttracker::StDesc {
        let model = match self.model { 0 => sttracker::STMODEL_NOISE, 1 => sttracker::STMODEL_QUEST, _ => sttracker::STMODEL_IMAGE };
        sttracker::StDesc { nh: self.nh as i64, bs: self.bs, noise_cross: self.noise_cross, noise_roll: self.noise_roll, latency: self.latency,
            max_rate: self.max_rate, sun_excl: self.sun_excl, earth_excl: self.earth_excl, fov: self.fov, model, bias_sigma: self.bias_sigma,
            misalign_sigma: self.misalign_sigma, moon_excl: self.moon_excl, blind_s: self.blind_s, noise_rate_ref: self.noise_rate_ref }
    }
}
const HIST: usize = sttracker::ST_HIST as usize;
/// Star tracker unit (sens_star_tracker's `sttracker`): the generated state (each head's mount bias and misalignment,
/// whether it has failed, until when it is blind, the noise stream), read and written through `Deref`; the attitude
/// history the latency reaches back over; the onboard star table (sens_star_catalogue, filled for models 1 and 2).
#[derive(Clone)]
pub struct St { pub d: StDesc, dg: sttracker::StDesc, s: sttracker::StUnit, ht: [f64; HIST], hq: [Q; HIST], hn: usize,
    cr: [V3; N_STARS], cm: [f64; N_STARS] }
impl core::ops::Deref for St { type Target = sttracker::StUnit; fn deref(&self) -> &sttracker::StUnit { &self.s } }
impl core::ops::DerefMut for St { fn deref_mut(&mut self) -> &mut sttracker::StUnit { &mut self.s } }
impl St {
    pub fn new(d: StDesc, disp: &mut Rng, noise: Rng) -> St {
        let dg = d.rec();
        let mut s = St { d, dg, s: drawn(disp, |g| sttracker::st_new(dg, g, noise.stream())), ht: [0.0; HIST], hq: [[0.0; 4]; HIST], hn: 0,
                         cr: [[0.0; 3]; N_STARS], cm: [0.0; N_STARS] };
        if d.model >= 1 { starcat::star_catalogue(&mut s.cr, &mut s.cm); }
        s
    }
    /// Attitude history for the latency (asils.devices.st_history, span 0.5 s).
    pub fn history(&mut self, t: f64, q: &Q) { self.hn = sttracker::st_history(&mut self.ht, &mut self.hq, self.hn as i64, t, *q) as usize; }
    /// The onboard star table (filled for models 1 and 2): each star's direction and magnitude.
    pub fn catalogue(&self) -> (&[V3], &[f64]) { (&self.cr, &self.cm) }
    /// The onboard star table as the generated chain takes it (by reference).
    pub fn catalogue_mut(&mut self) -> (&mut [V3; N_STARS], &mut [f64; N_STARS]) { (&mut self.cr, &mut self.cm) }
    /// Per head: Some(q_meas) when valid. `moon_b`: the Moon's direction in the body.
    /// Models 0 and 1; model 2 needs [`St::sample_with`].
    pub fn sample(&mut self, q_true: &Q, t: f64, w: &V3, sun_b: &V3, moon_b: &V3, nadir_b: &V3, earth_ang: f64) -> [Option<Q>; NH] {
        self.sample_with(q_true, t, w, sun_b, moon_b, nadir_b, earth_ang, None)
    }
    /// As [`St::sample`], with the frame buffers model 2 renders into (it panics without them:
    /// a run never flies the image model on nothing). Model 2 renders a frame only for a head
    /// that is valid (its noise is drawn from this unit's stream); a frame the chain cannot
    /// solve gives no attitude.
    pub fn sample_with(&mut self, q_true: &Q, t: f64, w: &V3, sun_b: &V3, moon_b: &V3, nadir_b: &V3, earth_ang: f64, work: Option<&mut stc::Work>) -> [Option<Q>; NH] {
        let (ok, q, valid, dq, q_old) = sttracker::st_sample(&mut self.s, self.dg, &mut self.ht, &mut self.hq, self.hn as i64, &mut self.cr, &mut self.cm,
                                                             *q_true, t, *w, *sun_b, *moon_b, *nadir_b, earth_ang);
        let mut out = [None; NH];
        for h in 0..self.d.nh { if ok[h] { out[h] = Some(q[h]); } }
        if self.d.model == 2 && (0..self.d.nh).any(|h| valid[h]) {
            // COMPONENT LEVEL: each valid head's chain on a rendered frame, through its true mount; it reports the body
            // attitude through its nominal mount (sens_star_image's st_image)
            let wk = work.expect("the image star-tracker model needs its frame buffers (St::sample_with)");
            let (okc, qc) = stimage::st_image(&mut self.s, self.dg, self.d.cam.rec(), valid, dq, q_old, &mut self.cr, &mut self.cm, N_STARS as i64,
                                              wk.img, wk.work, wk.ia, wk.ib, wk.pi, wk.pj, wk.pa);
            for h in 0..self.d.nh { if okc[h] { out[h] = Some(qc[h]); } }
        }
        out
    }
}

// ---------------- Earth sensor ----------------
#[derive(Clone, Copy, Debug, Default)]
pub struct EsDesc { pub fitted: bool, pub bs: V3, pub noise: f64, pub fov: f64, pub rate_hz: f64, pub bias_sigma: f64 }
impl EsDesc {
    /// The design's record of the Earth sensor.
    pub fn rec(&self) -> earthsensor::EsDesc { earthsensor::EsDesc { bs: self.bs, noise: self.noise, fov: self.fov, bias_sigma: self.bias_sigma } }
}
unit!(Es, EsDesc, earthsensor::EsDesc, earthsensor::EsUnit);
impl Es {
    pub fn new(d: EsDesc, disp: &mut Rng, noise: Rng) -> Es {
        let dg = d.rec();
        Es { d, dg, s: drawn(disp, |g| earthsensor::es_new(dg, g, noise.stream())) }
    }
    pub fn sample(&mut self, nadir_b: &V3) -> Option<V3> { opt(earthsensor::es_sample(&mut self.s, self.dg, *nadir_b)) }
}

// ---------------- GNSS ----------------
/// `latency` [s]: a fix reaches the bus this long after the epoch it solves for, and the
/// receiver reports it as current.
#[derive(Clone, Copy, Debug, Default)]
pub struct GpsDesc { pub fitted: bool, pub pos_sigma: f64, pub vel_sigma: f64, pub rate_hz: f64, pub latency: f64 }
/// The truth states the latency reaches back over (one per loop tick).
pub const GPS_HIST: usize = gnss::GPS_HIST as usize;
/// The receiver in flight: whether it is silent (a fault), its noise stream, and the history of the truth the latency
/// reaches back over (gnss's gps_history and gps_delayed keep it).
#[derive(Clone, Copy, Debug)]
pub struct Gps { pub d: GpsDesc, pub dead: bool, pub g: crate::gen::rt::Stream, ht: [f64; GPS_HIST], hr: [V3; GPS_HIST], hv: [V3; GPS_HIST], hn: usize }
impl Gps {
    pub fn new(d: GpsDesc, noise: Rng) -> Gps {
        Gps { d, dead: false, g: noise.stream(), ht: [0.0; GPS_HIST], hr: [[0.0; 3]; GPS_HIST], hv: [[0.0; 3]; GPS_HIST], hn: 0 }
    }
    /// Record the truth state at t (every tick); keep what the latency reaches back to.
    pub fn history(&mut self, t: f64, r: &V3, v: &V3) {
        self.hn = gnss::gps_history(&mut self.ht, &mut self.hr, &mut self.hv, self.hn as i64, self.d.latency, t, *r, *v) as usize;
    }
    /// (epoch, r, v): the truth state at t - latency, linear between ticks; before the history
    /// reaches back that far (the first `latency` of a run), its oldest state and that state's epoch.
    pub fn delayed(&self, t: f64) -> (f64, V3, V3) { gnss::gps_delayed(self.ht, self.hr, self.hv, self.hn as i64, self.d.latency, t) }
    /// The fix with its noise.
    pub fn sample(&mut self, r: &V3, v: &V3) -> (V3, V3) { gnss::gps_sample(*r, *v, self.d.pos_sigma, self.d.vel_sigma, &mut self.g) }
    /// The truth (r, v; J2000) at the epoch te [s] of a run that starts at the Julian date jd0, in ECEF (WGS-84).
    pub fn ecef(jd0: f64, te: f64, r: &V3, v: &V3) -> (V3, V3) { gnss::gnss_ecef(jd0, te, *r, *v) }
}
