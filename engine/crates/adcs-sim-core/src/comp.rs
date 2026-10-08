//! Component-level sensor chains (asils.comp): the unit's own processing run on what its
//! detector would read, in place of a noise model on the answer.
//!
//! - [`star_tracker`]: render a pixel frame of the catalogue stars in the field, centroid the
//!   spots, identify them by pair angles, solve the attitude by the q-method with a residual
//!   check (asils.comp.star_tracker.{render,centroid,identify,attitude,chain,pairs}).
//! - [`sun_sensor`]: a square aperture over a four-quadrant photodiode; the four currents and
//!   back to the Sun direction (asils.comp.sun_sensor.{currents,angles}).
//!
//! No allocation: the frame, its scratch copies and the onboard pair table are the caller's slices
//! ([`star_tracker::Work`]). The onboard star table comes as two arrays, its directions `cr` and its magnitudes `cm`
//! (sens_star_catalogue's, `sensors::St::catalogue_mut`); the chain (l3_sens_row_09 to 11), the attitude with its residual
//! check (l3_sens_row_12) and the quadrant Sun sensor (07 and 08) are sens's methods, generated from the design. Pixel coordinates are the twin's: 1-based,
//! x the column and y the row, the frame stored column by column.
//! Twin: matlab_sils/+asils/+comp. Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use crate::la::*;
use crate::rng::Rng;

/// Body -> head rotation (rows x, y, z): head z on the boresight `z`, head x = z x Z_body (or
/// z x X_body when z is along Z_body). The twin's head_ (star tracker) and Rh (Sun sensor); sens's l3_sens_row_07
/// `head_frame`, generated from the design.
pub fn head_frame(z: &V3) -> M3 { crate::gen::sunquad::head_frame(*z) }

pub mod star_tracker {
    //! sens's star-tracker image chain, generated from the design: l3_sens_row_09 (`gen::strender`, the frame), 10
    //! (`gen::stcentroid`, the spots), 11 (`gen::stidentify`, the pair table and the identification), the chain
    //! (`gen::stimage`, sens_star_image) and the attitude (`gen::stattitude`, l3_sens_row_12); here only their call by the
    //! names the engine has always used, the part's camera (the descriptor product.rs fills) and the caller's buffers handed
    //! over.
    use super::*;
    use crate::gen::{stattitude, stcentroid, stidentify, stimage, strender};

    /// The most spots the centroiding keeps (a part's `max_spots` above it is refused): stattitude's ST_SPOTS.
    pub const MAX_SPOTS: usize = stattitude::ST_SPOTS as usize;
    /// No catalogue star.
    pub const NONE: usize = usize::MAX;
    /// The onboard table's stars (sens_star_catalogue's N_STARS).
    pub const N_STARS: usize = crate::gen::starcat::N_STARS as usize;

    /// Detector, optics and the onboard chain's settings, every one from the part
    /// (asils.comp.star_tracker.camera). `f` [px] and `c` (the principal point, 1-based) follow (`st_focal`).
    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    pub struct Camera {
        /// pixels per side, the field's half-angle [rad]
        pub n: usize, pub fov: f64,
        /// PSF sigma [px], electrons of a magnitude-6 star, background and read noise [e-]
        pub psf_px: f64, pub flux0: f64, pub bg: f64, pub read_noise: f64,
        /// detection threshold [robust sigmas above the median], spots kept
        pub k_sigma: f64, pub max_spots: usize,
        /// pair-angle tolerance [rad], magnitude gate, residual a star may keep after the fit [rad]
        pub id_tol: f64, pub mag_tol: f64, pub fit_tol: f64,
        pub f: f64, pub c: f64,
    }
    impl Camera {
        #[allow(clippy::too_many_arguments)]
        pub fn new(fov: f64, n: usize, psf_px: f64, flux0: f64, bg: f64, read_noise: f64, k_sigma: f64, max_spots: usize, id_tol: f64, mag_tol: f64, fit_tol: f64) -> Camera {
            let (f, c) = strender::st_focal(fov, n as i64);
            Camera { n, fov, psf_px, flux0, bg, read_noise, k_sigma, max_spots, id_tol, mag_tol, fit_tol, f, c }
        }
        /// The design's record of the camera.
        pub fn rec(&self) -> strender::StCamera {
            strender::StCamera { n: self.n as i64, fov: self.fov, psf_px: self.psf_px, flux0: self.flux0, bg: self.bg, read_noise: self.read_noise,
                k_sigma: self.k_sigma, max_spots: self.max_spots as i64, id_tol: self.id_tol, mag_tol: self.mag_tol, fit_tol: self.fit_tol, f: self.f, c: self.c }
        }
    }

    /// How many catalogue pairs lie closer than the field diagonal (the pair table's length).
    pub fn pair_count(cr: &mut [V3; N_STARS], fov: f64) -> usize { stidentify::st_pair_count(cr, N_STARS as i64, fov) as usize }

    /// The onboard pair table (asils.comp.star_tracker.pairs): every catalogue pair i < j closer than the field diagonal,
    /// sorted by angle (equal angles by j, then i), into `i`, `j` and `ang`, built in `ti`, `tj` and `ta`: each as long
    /// as [`pair_count`] says. Returns that count.
    #[allow(clippy::too_many_arguments)]
    pub fn pairs(cr: &mut [V3; N_STARS], fov: f64, i: &mut [i64], j: &mut [i64], ang: &mut [f64], ti: &mut [i64], tj: &mut [i64], ta: &mut [f64]) -> usize {
        stidentify::st_pairs(cr, N_STARS as i64, fov, i, j, ang, ti, tj, ta) as usize
    }

    /// The buffers one frame needs: the frame (n*n), its scratch copy and two index lists of the same length, and the
    /// onboard pair table of the catalogue (the two indices and the angle of each pair).
    pub struct Work<'a> {
        pub img: &'a mut [f64], pub work: &'a mut [f64], pub ia: &'a mut [i64], pub ib: &'a mut [i64],
        pub pi: &'a mut [i64], pub pj: &'a mut [i64], pub pa: &'a mut [f64],
    }

    /// What one frame gave: spots found, stars identified, stars the fit kept.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct Info { pub spots: usize, pub identified: usize, pub used: usize }

    /// The stars `render` reports drawn, at most.
    pub const TRUTH: usize = 128;

    /// MODEL SIDE (asils.comp.star_tracker.render): the frame the detector reads at the head
    /// attitude `r_eh` (ECI -> head DCM): every catalogue star within the field diagonal through
    /// a pinhole (head z the boresight), spread by a Gaussian PSF over 9 x 9 pixels, on the
    /// background; with `noise`, shot noise sqrt(signal) and read noise on every pixel.
    /// `truth` receives (x, y, catalogue index) of the stars drawn, as far as it holds them (up to [`TRUTH`]);
    /// returns how many were drawn.
    pub fn render(r_eh: &M3, cr: &mut [V3; N_STARS], cm: &mut [f64; N_STARS], cam: &Camera, img: &mut [f64], noise: Option<&mut Rng>, truth: &mut [(f64, f64, usize)]) -> usize {
        let k = truth.len().min(TRUTH);
        let (mut tx, mut ty, mut tk) = ([0.0; TRUTH], [0.0; TRUTH], [0i64; TRUTH]);
        let drawn = match noise {
            Some(r) => {
                let mut g = r.stream();
                let d = strender::st_render(*r_eh, cr, cm, N_STARS as i64, cam.rec(), img, true, &mut g, &mut tx[..k], &mut ty[..k], &mut tk[..k]);
                *r = Rng::from_stream(&g);
                d
            }
            None => strender::st_render(*r_eh, cr, cm, N_STARS as i64, cam.rec(), img, false, &mut Rng::new(0, 0).stream(), &mut tx[..k], &mut ty[..k], &mut tk[..k]),
        } as usize;
        for m in 0..k.min(drawn) { truth[m] = (tx[m], ty[m], tk[m] as usize); }
        drawn
    }

    /// Pixel frame -> spots (asils.comp.star_tracker.centroid): robust background (median and
    /// 1.4826 MAD), threshold k_sigma above it, brightest pixels first with a 4-pixel exclusion
    /// round every spot found, centre of gravity of the background-subtracted 5 x 5 window.
    /// spots[k] = [x, y, flux (e-)], brightest first; returns how many (at most max_spots). `work`, `ia` and `ib` are
    /// scratch as long as the frame.
    pub fn centroid(img: &mut [f64], cam: &Camera, work: &mut [f64], ia: &mut [i64], ib: &mut [i64], spots: &mut [[f64; 3]; MAX_SPOTS]) -> usize {
        let (s, ns) = stcentroid::st_centroid(img, cam.rec(), work, ia, ib);
        *spots = s;
        ns as usize
    }

    /// Spots -> catalogue indices (asils.comp.star_tracker.identify): every measured pair votes
    /// for the catalogue pairs of the same angle (within id_tol) whose two magnitudes both match
    /// (within mag_tol); each spot takes its most-voted star (the lowest index on a tie); then a
    /// star is kept only while its angles to at least two other kept stars match the catalogue's
    /// (three passes). id[p] = NONE where none. True when three or more are kept.
    #[allow(clippy::too_many_arguments)]
    pub fn identify(b: &[V3], mag: &[f64], pi: &mut [i64], pj: &mut [i64], pa: &mut [f64], cr: &mut [V3; N_STARS], cm: &mut [f64; N_STARS], cam: &Camera, id: &mut [usize]) -> bool {
        let n = b.len();
        let (mut bb, mut mm_) = ([[0.0; 3]; MAX_SPOTS], [0.0; MAX_SPOTS]);
        bb[..n].copy_from_slice(b);
        mm_[..n].copy_from_slice(&mag[..n]);
        let (ig, ok) = stidentify::st_identify(bb, mm_, n as i64, pi, pj, pa, cr, cm, N_STARS as i64, cam.id_tol, cam.mag_tol);
        for p in 0..n { id[p] = if ig[p] < 0 { NONE } else { ig[p] as usize }; }
        ok
    }

    /// Identified stars -> ECI -> head attitude (asils.comp.star_tracker.attitude): the q-method,
    /// then the star whose direction misses its catalogue star by the most is dropped while that
    /// miss is `tol` or more, and the fit repeated. True with three or more stars left. l3_sens_row_12's `st_attitude`,
    /// generated from the design; here its call, each spot's catalogue direction handed over.
    pub fn attitude(b: &[V3], id: &mut [usize], cr: &[V3], tol: f64) -> (Q, bool) {
        let n = b.len();
        let (mut bb, mut rs, mut ig) = ([[0.0; 3]; MAX_SPOTS], [[0.0; 3]; MAX_SPOTS], [-1i64; MAX_SPOTS]);
        for p in 0..n {
            bb[p] = b[p];
            if id[p] != NONE { rs[p] = cr[id[p]]; ig[p] = id[p] as i64; }
        }
        let (q, ok) = stattitude::st_attitude(bb, rs, n as i64, &mut ig, tol);
        for p in 0..n { id[p] = if ig[p] < 0 { NONE } else { ig[p] as usize }; }
        (q, ok)
    }

    /// The whole chain for one head (asils.comp.star_tracker.chain): render -> centroid ->
    /// identify -> attitude. `q_true` the true ECI -> body attitude at the exposure,
    /// `r_body2head` the head's TRUE mount (misalignment included), `r_head_nominal` the mount
    /// the unit reports through. (q_body, solved, info); q_true when not solved.
    #[allow(clippy::too_many_arguments)]
    pub fn chain(q_true: &Q, r_body2head: &M3, r_head_nominal: &M3, cr: &mut [V3; N_STARS], cm: &mut [f64; N_STARS], cam: &Camera, w: &mut Work, noise: Option<&mut Rng>) -> (Q, bool, Info) {
        let (q, ok, spots, identified, used) = match noise {
            Some(r) => {
                let mut g = r.stream();
                let a = stimage::st_chain(*q_true, *r_body2head, *r_head_nominal, cr, cm, N_STARS as i64, cam.rec(), w.img, w.work, w.ia, w.ib, w.pi, w.pj, w.pa, true, &mut g);
                *r = Rng::from_stream(&g);
                a
            }
            None => stimage::st_chain(*q_true, *r_body2head, *r_head_nominal, cr, cm, N_STARS as i64, cam.rec(), w.img, w.work, w.ia, w.ib, w.pi, w.pj, w.pa, false,
                                      &mut Rng::new(0, 0).stream()),
        };
        (q, ok, Info { spots: spots as usize, identified: identified as usize, used: used as usize })
    }
}

pub mod sun_sensor {
    //! sens's l3_sens_row_07 (`sunquad`: the currents) and l3_sens_row_08 (`sunangles`: the angles), generated from the
    //! design; here only their call by the names the engine has always used.
    use super::*;
    use crate::gen::{sunangles, sunquad};

    /// A square aperture of side `a` at height `h` over a four-quadrant photodiode; current
    /// noise and the validity threshold as fractions of the full-Sun current
    /// (asils.comp.sun_sensor.head), every one from the part: the design's record.
    pub use crate::gen::sunquad::SunHead as Head;

    /// MODEL SIDE (asils.comp.sun_sensor.currents): the Sun (unit vector, head frame, z the
    /// boresight) casts the square spot displaced by h tan(alpha), h tan(beta); each quadrant's
    /// current is its share of the spot area times cos(incidence), plus noise. I = [++, -+, --, +-]
    /// (quadrants in x, y). Zero, and no noise drawn, with the Sun behind the head. With no noise
    /// stream, no noise (the generated model with a zero noise, its draws on a stream of no one's).
    pub fn currents(s: &V3, p: &Head, noise: Option<&mut Rng>) -> [f64; 4] {
        match noise {
            Some(r) => {
                let mut g = r.stream();
                let i = sunquad::quad_currents(*s, *p, &mut g);
                *r = Rng::from_stream(&g);
                i
            }
            None => sunquad::quad_currents(*s, Head { noise: 0.0, ..*p }, &mut Rng::new(0, 0).stream()),
        }
    }

    /// Four currents -> the Sun in the head frame (asils.comp.sun_sensor.angles): the normalised
    /// differences give the spot's displacement, tan(alpha) = (a/2) rx / h, likewise beta (exact
    /// while the spot covers the centre). Valid when the total current exceeds min_frac.
    pub fn angles(i: &[f64; 4], p: &Head) -> (V3, bool) { sunangles::quad_angles(*i, *p) }
}
