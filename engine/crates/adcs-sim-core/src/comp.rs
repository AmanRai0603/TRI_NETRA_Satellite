//! Component-level sensor chains (asils.comp): the unit's own processing run on what its
//! detector would read, in place of a noise model on the answer.
//!
//! - [`star_tracker`]: render a pixel frame of the catalogue stars in the field, centroid the
//!   spots, identify them by pair angles, solve the attitude by the q-method with a residual
//!   check (asils.comp.star_tracker.{render,centroid,identify,attitude,chain,pairs}).
//! - [`sun_sensor`]: a square aperture over a four-quadrant photodiode; the four currents and
//!   back to the Sun direction (asils.comp.sun_sensor.{currents,angles}).
//!
//! No allocation: the frame, its scratch copy, the onboard pair table and the vote counts are
//! the caller's slices ([`star_tracker::Work`]). Pixel coordinates are the twin's: 1-based,
//! x the column and y the row, the frame stored column by column.
//! Twin: matlab_sils/+asils/+comp. Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use crate::la::*;
use crate::pm::*;
use crate::rng::Rng;

/// Body -> head rotation (rows x, y, z): head z on the boresight `z`, head x = z x Z_body (or
/// z x X_body when z is along Z_body). The twin's head_ (star tracker) and Rh (Sun sensor).
pub fn head_frame(z: &V3) -> M3 {
    let z = unit(z);
    let mut x = cross(&z, &[0.0, 0.0, 1.0]);
    if norm(&x) < 1e-6 { x = cross(&z, &[1.0, 0.0, 0.0]); }
    let x = unit(&x);
    [x, cross(&z, &x), z]
}

pub mod star_tracker {
    use super::*;

    /// The most spots the centroiding keeps (a part's `max_spots` above it is refused).
    pub const MAX_SPOTS: usize = 32;
    /// No catalogue star.
    pub const NONE: usize = usize::MAX;

    /// Detector, optics and the onboard chain's settings, every one from the part
    /// (asils.comp.star_tracker.camera). `f` [px] and `c` (the principal point, 1-based) follow.
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
            let nf = n as f64;
            Camera { n, fov, psf_px, flux0, bg, read_noise, k_sigma, max_spots, id_tol, mag_tol, fit_tol, f: (nf/2.0)/tan(fov), c: (nf + 1.0)/2.0 }
        }
    }

    /// One onboard pair: catalogue indices i < j (0-based) and their angle [rad].
    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    pub struct Pair { pub i: u16, pub j: u16, pub ang: f64 }

    fn near(a: &V3, b: &V3, fov: f64) -> bool { dot(a, b) > cos(2.0*sqrt(2.0)*fov) }

    /// How many catalogue pairs lie closer than the field diagonal (the pair table's length).
    pub fn pair_count(cat: &[(V3, f64)], fov: f64) -> usize {
        let mut n = 0;
        for j in 0..cat.len() { for i in 0..j { if near(&cat[i].0, &cat[j].0, fov) { n += 1; } } }
        n
    }

    /// The onboard pair table (asils.comp.star_tracker.pairs): every catalogue pair closer than
    /// the field diagonal, sorted by angle (ties as the twin's stable sort leaves them). Writes
    /// `out[..pair_count]` and returns that count; `out` must hold it.
    pub fn pairs(cat: &[(V3, f64)], fov: f64, out: &mut [Pair]) -> usize {
        let mut n = 0;
        for j in 0..cat.len() {
            for i in 0..j {
                if near(&cat[i].0, &cat[j].0, fov) {
                    out[n] = Pair { i: i as u16, j: j as u16, ang: acos(dot(&cat[i].0, &cat[j].0).min(1.0)) };
                    n += 1;
                }
            }
        }
        out[..n].sort_unstable_by(|a, b| a.ang.total_cmp(&b.ang).then(a.j.cmp(&b.j)).then(a.i.cmp(&b.i)));
        n
    }

    /// The buffers one frame needs: the frame (n*n), a scratch of the same length, the pair
    /// table of the catalogue and a vote count per catalogue star.
    pub struct Work<'a> { pub img: &'a mut [f64], pub scratch: &'a mut [(f64, u32)], pub pairs: &'a [Pair], pub votes: &'a mut [u32] }

    /// What one frame gave: spots found, stars identified, stars the fit kept.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct Info { pub spots: usize, pub identified: usize, pub used: usize }

    #[inline] fn px(n: usize, x: usize, y: usize) -> usize { (x - 1)*n + (y - 1) }

    /// MODEL SIDE (asils.comp.star_tracker.render): the frame the detector reads at the head
    /// attitude `r_eh` (ECI -> head DCM): every catalogue star within the field diagonal through
    /// a pinhole (head z the boresight), spread by a Gaussian PSF over 9 x 9 pixels, on the
    /// background; with `noise`, shot noise sqrt(signal) and read noise on every pixel.
    /// `truth` receives (x, y, catalogue index) of the stars drawn, as far as it holds them;
    /// returns how many were drawn.
    pub fn render(r_eh: &M3, cat: &[(V3, f64)], cam: &Camera, img: &mut [f64], noise: Option<&mut Rng>, truth: &mut [(f64, f64, usize)]) -> usize {
        let n = cam.n;
        let nf = n as f64;
        for p in img.iter_mut().take(n*n) { *p = cam.bg; }
        let cmin = cos(cam.fov*sqrt(2.0));
        let s2 = 2.0*cam.psf_px*cam.psf_px;
        let mut drawn = 0;
        for (k, (r, mag)) in cat.iter().enumerate() {
            let v = mv(r_eh, r);
            if v[2] <= cmin { continue; }
            let x = cam.f*v[0]/v[2] + cam.c;
            let y = cam.f*v[1]/v[2] + cam.c;
            if x < 6.0 || y < 6.0 || x > nf - 5.0 || y > nf - 5.0 { continue; }
            let fl = cam.flux0*pow(10.0, -0.4*(mag - 6.0));
            let (ix, iy) = (round(x), round(y));
            let mut w = [[0.0; 9]; 9];          // [column][row]
            let mut sum = 0.0;
            for c in 0..9 {
                for rr in 0..9 {
                    let (gx, gy) = (c as f64 - 4.0, rr as f64 - 4.0);
                    let (dx, dy) = (ix + gx - x, iy + gy - y);
                    w[c][rr] = exp(-(dx*dx + dy*dy)/s2);
                    sum += w[c][rr];
                }
            }
            let (ix, iy) = (ix as usize, iy as usize);
            for c in 0..9 { for rr in 0..9 { img[px(n, ix + c - 4, iy + rr - 4)] += fl*(w[c][rr]/sum); } }
            if drawn < truth.len() { truth[drawn] = (x, y, k); }
            drawn += 1;
        }
        if let Some(g) = noise {
            for p in img.iter_mut().take(n*n) {
                let v = *p;
                *p = v + sqrt(v.max(0.0))*g.normal() + cam.read_noise*g.normal();
            }
        }
        drawn
    }

    /// The median of `s[..m].0` (the mean of the two middle values when m is even); reorders s.
    fn median(s: &mut [(f64, u32)]) -> f64 {
        let m = s.len();
        let cmp = |a: &(f64, u32), b: &(f64, u32)| a.0.total_cmp(&b.0);
        let k = (m - 1)/2;
        let a = s.select_nth_unstable_by(k, cmp).1.0;
        if m % 2 == 1 { return a; }
        let b = s[k + 1..].iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
        (a + b)/2.0
    }

    /// Pixel frame -> spots (asils.comp.star_tracker.centroid): robust background (median and
    /// 1.4826 MAD), threshold k_sigma above it, brightest pixels first with a 4-pixel exclusion
    /// round every spot found, centre of gravity of the background-subtracted 5 x 5 window.
    /// spots[k] = [x, y, flux (e-)], brightest first; returns how many (at most max_spots).
    pub fn centroid(img: &[f64], cam: &Camera, scratch: &mut [(f64, u32)], spots: &mut [[f64; 3]; MAX_SPOTS]) -> usize {
        let n = cam.n;
        let np = n*n;
        let s = &mut scratch[..np];
        for (l, p) in s.iter_mut().enumerate() { *p = (img[l], l as u32); }
        let bg = median(s);
        for (l, p) in s.iter_mut().enumerate() { *p = (abs(img[l] - bg), l as u32); }
        let sg = 1.4826*median(s);
        let thr = bg + cam.k_sigma*sg;
        // the pixels above it, brightest first; equal values in the order the twin's find
        // gives them (column by column)
        let mut m = 0;
        for l in 0..np { if img[l] > thr { s[m] = (img[l], l as u32); m += 1; } }
        s[..m].sort_unstable_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
        let mut ns = 0;
        let cap = cam.max_spots.min(MAX_SPOTS);
        for &(_, l) in s[..m].iter() {
            let (x, y) = (l as usize/n + 1, l as usize % n + 1);
            if x < 3 || y < 3 || x > n - 2 || y > n - 2 { continue; }
            let (xf, yf) = (x as f64, y as f64);
            if spots[..ns].iter().any(|sp| abs(sp[0] - xf) < 4.0 && abs(sp[1] - yf) < 4.0) { continue; }
            let (mut tot, mut sx, mut sy) = (0.0, 0.0, 0.0);
            for c in 0..5 {
                for r in 0..5 {
                    let w = (img[px(n, x + c - 2, y + r - 2)] - bg).max(0.0);
                    tot += w;
                    sx += (c as f64 - 2.0)*w;
                    sy += (r as f64 - 2.0)*w;
                }
            }
            spots[ns] = [xf + sx/tot, yf + sy/tot, tot];
            ns += 1;
            if ns >= cap { break; }
        }
        ns
    }

    /// Spots -> catalogue indices (asils.comp.star_tracker.identify): every measured pair votes
    /// for the catalogue pairs of the same angle (within id_tol) whose two magnitudes both match
    /// (within mag_tol); each spot takes its most-voted star (the lowest index on a tie); then a
    /// star is kept only while its angles to at least two other kept stars match the catalogue's
    /// (three passes). id[p] = NONE where none. True when three or more are kept.
    pub fn identify(b: &[V3], mag: &[f64], pairs: &[Pair], cat: &[(V3, f64)], cam: &Camera, votes: &mut [u32], id: &mut [usize]) -> bool {
        let n = b.len();
        for x in id.iter_mut().take(n) { *x = NONE; }
        if n < 3 { return false; }
        let tol = cam.id_tol;
        let mut cand = [NONE; MAX_SPOTS];
        for p in 0..n {
            for v in votes.iter_mut().take(cat.len()) { *v = 0; }
            for q in 0..n {
                if q == p { continue; }
                let th = acos(dot(&b[p], &b[q]).min(1.0));
                let lo = pairs.partition_point(|x| x.ang < th - tol);
                let hi = pairs.partition_point(|x| x.ang <= th + tol);
                for pr in &pairs[lo..hi.max(lo)] {
                    let (i, j) = (pr.i as usize, pr.j as usize);
                    let (mi, mj) = (cat[i].1, cat[j].1);
                    if abs(mi - mag[p]) < cam.mag_tol && abs(mj - mag[q]) < cam.mag_tol { votes[i] += 1; }
                    if abs(mj - mag[p]) < cam.mag_tol && abs(mi - mag[q]) < cam.mag_tol { votes[j] += 1; }
                }
            }
            let (mut best, mut bi) = (0, NONE);
            for (k, &v) in votes.iter().enumerate().take(cat.len()) { if v > best { best = v; bi = k; } }
            cand[p] = bi;
        }
        let mut keep = [false; MAX_SPOTS];
        for p in 0..n { keep[p] = cand[p] != NONE; }
        for _ in 0..3 {
            let was = keep;
            for p in 0..n {
                if !was[p] { continue; }
                let mut good = 0;
                for q in 0..n {
                    if q == p || !keep[q] { continue; }
                    let th = acos(dot(&b[p], &b[q]).min(1.0));
                    let tc = acos(dot(&cat[cand[p]].0, &cat[cand[q]].0).min(1.0));
                    if abs(th - tc) < 3.0*tol { good += 1; }
                }
                if good < 2 { keep[p] = false; }
            }
        }
        let mut kept = 0;
        for p in 0..n { if keep[p] { id[p] = cand[p]; kept += 1; } }
        kept >= 3
    }

    /// Identified stars -> ECI -> head attitude (asils.comp.star_tracker.attitude): the q-method,
    /// then the star whose direction misses its catalogue star by the most is dropped while that
    /// miss is `tol` or more, and the fit repeated. True with three or more stars left.
    pub fn attitude(b: &[V3], id: &mut [usize], cat: &[(V3, f64)], tol: f64) -> (Q, bool) {
        let mut q = [0.0, 0.0, 0.0, 1.0];
        let n = b.len();
        for _ in 0..n {
            let mut k = [0usize; MAX_SPOTS];
            let mut nk = 0;
            for p in 0..n { if id[p] != NONE { k[nk] = p; nk += 1; } }
            if nk < 3 { return (q, false); }
            let mut bb = [[0.0; 3]; MAX_SPOTS];
            let mut rr = [[0.0; 3]; MAX_SPOTS];
            for i in 0..nk { bb[i] = b[k[i]]; rr[i] = cat[id[k[i]]].0; }
            q = qmethod(&bb[..nk], &rr[..nk]);
            let a = dcm(&q);
            let (mut rm, mut j) = (f64::NEG_INFINITY, 0);
            for i in 0..nk {
                let res = acos(dot(&bb[i], &mv(&a, &rr[i])).min(1.0));
                if res > rm { rm = res; j = i; }
            }
            if rm < tol { return (q, true); }
            id[k[j]] = NONE;
        }
        (q, false)
    }

    /// The whole chain for one head (asils.comp.star_tracker.chain): render -> centroid ->
    /// identify -> attitude. `q_true` the true ECI -> body attitude at the exposure,
    /// `r_body2head` the head's TRUE mount (misalignment included), `r_head_nominal` the mount
    /// the unit reports through. (q_body, solved, info); q_true when not solved.
    #[allow(clippy::too_many_arguments)]
    pub fn chain(q_true: &Q, r_body2head: &M3, r_head_nominal: &M3, cat: &[(V3, f64)], cam: &Camera, w: &mut Work, noise: Option<&mut Rng>) -> (Q, bool, Info) {
        let r_eh = mm(r_body2head, &dcm(q_true));
        render(&r_eh, cat, cam, w.img, noise, &mut []);
        let mut spots = [[0.0; 3]; MAX_SPOTS];
        let ns = centroid(w.img, cam, w.scratch, &mut spots);
        let mut info = Info { spots: ns, ..Default::default() };
        if ns < 3 { return (*q_true, false, info); }
        let mut b = [[0.0; 3]; MAX_SPOTS];
        let mut mag = [0.0; MAX_SPOTS];
        for k in 0..ns {
            b[k] = unit(&[(spots[k][0] - cam.c)/cam.f, (spots[k][1] - cam.c)/cam.f, 1.0]);
            mag[k] = 6.0 - 2.5*log10(spots[k][2].max(1.0)/cam.flux0);
        }
        let mut id = [NONE; MAX_SPOTS];
        let ok = identify(&b[..ns], &mag[..ns], w.pairs, cat, cam, w.votes, &mut id[..ns]);
        info.identified = id[..ns].iter().filter(|&&x| x != NONE).count();
        if !ok { return (*q_true, false, info); }
        let (q_eh, ok) = attitude(&b[..ns], &mut id[..ns], cat, cam.fit_tol);
        info.used = id[..ns].iter().filter(|&&x| x != NONE).count();
        if !ok { return (*q_true, false, info); }
        (fromdcm(&mm(&transpose(r_head_nominal), &dcm(&q_eh))), true, info)
    }
}

pub mod sun_sensor {
    use super::*;

    /// A square aperture of side `a` at height `h` over a four-quadrant photodiode; current
    /// noise and the validity threshold as fractions of the full-Sun current
    /// (asils.comp.sun_sensor.head), every one from the part.
    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    pub struct Head { pub a: f64, pub h: f64, pub noise: f64, pub min_frac: f64 }

    /// MODEL SIDE (asils.comp.sun_sensor.currents): the Sun (unit vector, head frame, z the
    /// boresight) casts the square spot displaced by h tan(alpha), h tan(beta); each quadrant's
    /// current is its share of the spot area times cos(incidence), plus noise. I = [++, -+, --, +-]
    /// (quadrants in x, y). Zero, and no noise drawn, with the Sun behind the head.
    pub fn currents(s: &V3, p: &Head, noise: Option<&mut Rng>) -> [f64; 4] {
        if s[2] <= 0.0 { return [0.0; 4]; }
        let (dx, dy) = (p.h*s[0]/s[2], p.h*s[1]/s[2]);
        let ov = |lo: f64, hi: f64, a: f64, b: f64| (hi.min(b) - lo.max(a)).max(0.0);
        let inf = f64::INFINITY;
        let xp = ov(0.0, inf, dx - p.a/2.0, dx + p.a/2.0);
        let xm = ov(-inf, 0.0, dx - p.a/2.0, dx + p.a/2.0);
        let yp = ov(0.0, inf, dy - p.a/2.0, dy + p.a/2.0);
        let ym = ov(-inf, 0.0, dy - p.a/2.0, dy + p.a/2.0);
        let a2 = p.a*p.a;
        let mut i = [s[2]*(xp*yp/a2), s[2]*(xm*yp/a2), s[2]*(xm*ym/a2), s[2]*(xp*ym/a2)];
        if let Some(g) = noise { for x in i.iter_mut() { *x += p.noise*g.normal(); } }
        i
    }

    /// Four currents -> the Sun in the head frame (asils.comp.sun_sensor.angles): the normalised
    /// differences give the spot's displacement, tan(alpha) = (a/2) rx / h, likewise beta (exact
    /// while the spot covers the centre). Valid when the total current exceeds min_frac.
    pub fn angles(i: &[f64; 4], p: &Head) -> (V3, bool) {
        let tot = i[0] + i[1] + i[2] + i[3];
        if !(tot > p.min_frac) { return ([0.0, 0.0, 1.0], false); }
        let rx = ((i[0] + i[3]) - (i[1] + i[2]))/tot;
        let ry = ((i[0] + i[1]) - (i[2] + i[3]))/tot;
        (unit(&[p.a/2.0*rx/p.h, p.a/2.0*ry/p.h, 1.0]), true)
    }
}
