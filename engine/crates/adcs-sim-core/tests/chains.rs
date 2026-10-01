//! The component-level sensor chains (comp): the star tracker's rendered frame -> centroids ->
//! identification -> attitude, and the quadrant Sun sensor's currents -> angles, checked
//! against closed forms, against the truth they were rendered from, and against the MATLAB
//! twin's chain on the same noise-free frame (asils.comp.*).
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use adcs_sim_core::comp::star_tracker::{self as stc, Camera, Pair, Work, MAX_SPOTS, NONE};
use adcs_sim_core::comp::sun_sensor::{angles, currents, Head};
use adcs_sim_core::comp::head_frame;
use adcs_sim_core::la::*;
use adcs_sim_core::rng::Rng;
use adcs_sim_core::sensors::{star, St, StDesc, Sun, SunDesc, N_STARS};

/// The twin's baseline camera (asils.comp.star_tracker.camera(0.17)).
fn camera() -> Camera { Camera::new(0.17, 1024, 1.2, 3000.0, 50.0, 8.0, 5.0, 20, 2e-4, 0.25, 1e-4) }
fn catalogue() -> Vec<(V3, f64)> { (0..N_STARS).map(|k| star(k, N_STARS)).collect() }

struct Bufs { img: Vec<f64>, scratch: Vec<(f64, u32)>, pairs: Vec<Pair>, votes: Vec<u32> }
fn bufs(cat: &[(V3, f64)], cam: &Camera) -> Bufs {
    let mut pairs = vec![Pair::default(); stc::pair_count(cat, cam.fov)];
    stc::pairs(cat, cam.fov, &mut pairs);
    Bufs { img: vec![0.0; cam.n*cam.n], scratch: vec![(0.0, 0); cam.n*cam.n], pairs, votes: vec![0; cat.len()] }
}
fn work(b: &mut Bufs) -> Work<'_> { Work { img: &mut b.img, scratch: &mut b.scratch, pairs: &b.pairs, votes: &mut b.votes } }
fn arcsec(a: &Q, b: &Q) -> f64 { qangle(a, b)*180.0/std::f64::consts::PI*3600.0 }

/// A frame of known stars centroids back onto them: noise-free within 0.15 px (the 5 x 5
/// window cuts the 1.2 px PSF's tails: 0.13 px at worst here), with shot and read noise within 0.25 px for every star
/// brighter than 50 000 e- (magnitude 3.8) that has no neighbour within 8 px.
#[test]
fn a_rendered_frame_centroids_onto_its_stars() {
    let (cat, cam) = (catalogue(), camera());
    let mut b = bufs(&cat, &cam);
    let mut r = Rng::new(11, 1);
    for (noisy, tol) in [(false, 0.15), (true, 0.25)] {
        let mut worst: f64 = 0.0;
        let mut checked = 0;
        for _ in 0..4 {
            let q = qnorm(&[r.normal(), r.normal(), r.normal(), r.normal()]);
            let mut truth = [(0.0, 0.0, 0usize); 128];
            let mut g = Rng::new(5, 2);
            let nt = stc::render(&dcm(&q), &cat, &cam, &mut b.img, if noisy { Some(&mut g) } else { None }, &mut truth);
            let mut spots = [[0.0; 3]; MAX_SPOTS];
            let ns = stc::centroid(&b.img, &cam, &mut b.scratch, &mut spots);
            assert!(ns >= 3);
            for &(x, y, k) in &truth[..nt] {
                let fl = cam.flux0*10f64.powf(-0.4*(cat[k].1 - 6.0));
                let alone = truth[..nt].iter().all(|t| t.2 == k || (t.0 - x).abs() >= 8.0 || (t.1 - y).abs() >= 8.0);
                if fl < 5e4 || !alone || x < 8.0 || y < 8.0 || x > 1016.0 || y > 1016.0 { continue; }
                let d = spots[..ns].iter().map(|s| ((s[0] - x).powi(2) + (s[1] - y).powi(2)).sqrt()).fold(f64::INFINITY, f64::min);
                worst = worst.max(d);
                checked += 1;
            }
        }
        assert!(checked >= 20, "only {checked} stars checked");
        assert!(worst < tol, "noise {noisy}: worst centroid {worst} px against {tol}");
    }
}

/// Identification recovers the true attitude: every frame solved, noise-free within 20 arcsec (the
/// centroid window's bias: 3.5-14 arcsec here) and with noise within 60 arcsec (the twin's t_st_chain bound), three axes including roll; the
/// identified stars are the ones drawn there.
#[test]
fn identification_recovers_the_true_attitude() {
    let (cat, cam) = (catalogue(), camera());
    let mut b = bufs(&cat, &cam);
    let mut r = Rng::new(3, 1);
    let mut g = Rng::new(3, 2);
    let eye = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    for (noisy, tol) in [(false, 20.0), (true, 60.0)] {
        for k in 0..6 {
            let q = qnorm(&[r.normal(), r.normal(), r.normal(), r.normal()]);
            let (qm, ok, info) = stc::chain(&q, &eye, &eye, &cat, &cam, &mut work(&mut b), if noisy { Some(&mut g) } else { None });
            assert!(ok, "frame {k} (noise {noisy}) not solved: {info:?}");
            assert!(info.used >= 3);
            assert!(arcsec(&q, &qm) < tol, "frame {k} (noise {noisy}): {} arcsec", arcsec(&q, &qm));
        }
    }
    // the ids themselves: each identified spot is the catalogue star drawn nearest to it
    let q = qnorm(&[0.3, 0.1, -0.4, 0.8]);
    let mut truth = [(0.0, 0.0, 0usize); 128];
    let nt = stc::render(&dcm(&q), &cat, &cam, &mut b.img, None, &mut truth);
    let mut spots = [[0.0; 3]; MAX_SPOTS];
    let ns = stc::centroid(&b.img, &cam, &mut b.scratch, &mut spots);
    let mut bv = [[0.0; 3]; MAX_SPOTS];
    let mut mag = [0.0; MAX_SPOTS];
    for k in 0..ns {
        bv[k] = unit(&[(spots[k][0] - cam.c)/cam.f, (spots[k][1] - cam.c)/cam.f, 1.0]);
        mag[k] = 6.0 - 2.5*(spots[k][2].max(1.0)/cam.flux0).log10();
    }
    let mut id = [NONE; MAX_SPOTS];
    assert!(stc::identify(&bv[..ns], &mag[..ns], &b.pairs, &cat, &cam, &mut b.votes, &mut id[..ns]));
    let mut n_id = 0;
    for k in 0..ns {
        if id[k] == NONE { continue; }
        let near = truth[..nt].iter().min_by(|a, c| {
            let da = (a.0 - spots[k][0]).powi(2) + (a.1 - spots[k][1]).powi(2);
            let dc = (c.0 - spots[k][0]).powi(2) + (c.1 - spots[k][1]).powi(2);
            da.total_cmp(&dc)
        }).unwrap();
        assert_eq!(id[k], near.2, "spot {k}");
        n_id += 1;
    }
    assert!(n_id >= 10, "{n_id} identified");
}

/// A spot that is not a star (a hot pixel) is not identified, and the attitude stands.
#[test]
fn a_false_spot_is_dropped() {
    let (cat, cam) = (catalogue(), camera());
    let mut b = bufs(&cat, &cam);
    let q = qnorm(&[0.1, -0.2, 0.3, 0.9]);
    stc::render(&dcm(&q), &cat, &cam, &mut b.img, None, &mut []);
    b.img[(600 - 1)*1024 + (300 - 1)] += 4e5;         // x 600, y 300: brighter than any star
    let mut spots = [[0.0; 3]; MAX_SPOTS];
    let ns = stc::centroid(&b.img, &cam, &mut b.scratch, &mut spots);
    assert!((spots[0][0] - 600.0).abs() < 1e-9 && (spots[0][1] - 300.0).abs() < 1e-9, "{:?}", spots[0]);
    let mut bv = [[0.0; 3]; MAX_SPOTS];
    let mut mag = [0.0; MAX_SPOTS];
    for k in 0..ns {
        bv[k] = unit(&[(spots[k][0] - cam.c)/cam.f, (spots[k][1] - cam.c)/cam.f, 1.0]);
        mag[k] = 6.0 - 2.5*(spots[k][2].max(1.0)/cam.flux0).log10();
    }
    let mut id = [NONE; MAX_SPOTS];
    assert!(stc::identify(&bv[..ns], &mag[..ns], &b.pairs, &cat, &cam, &mut b.votes, &mut id[..ns]));
    let (qe, ok) = stc::attitude(&bv[..ns], &mut id[..ns], &cat, cam.fit_tol);
    assert!(ok && id[0] == NONE);
    assert!(arcsec(&q, &qe) < 5.0);
}

/// The engine's chain is the twin's on the same noise-free frame: the pair table, the twenty
/// spots and the attitude (asils.comp.star_tracker.* with cam.noise = false at
/// q = norm([0.1 -0.2 0.3 0.9]), the head on the body axes; values printed by Octave 8 with
/// %.17g). Agreement: the pair count exactly, the angles within 1e-14 rad (acos near 1),
/// centroids within 1e-9 px, fluxes within 1e-9
/// relative, the attitude within 1e-6 arcsec.
#[test]
fn the_star_tracker_chain_is_the_twins() {
    let (cat, cam) = (catalogue(), camera());
    let mut b = bufs(&cat, &cam);
    assert_eq!(b.pairs.len(), 452404);
    assert_eq!((b.pairs[0].i + 1, b.pairs[0].j + 1), (3973, 3986));
    assert!((b.pairs[0].ang - 0.030776038627103805).abs() < 1e-14 && (b.pairs[b.pairs.len() - 1].ang - 0.48083169588160557).abs() < 1e-14);
    let q = qnorm(&[0.1, -0.2, 0.3, 0.9]);
    let nt = stc::render(&dcm(&q), &cat, &cam, &mut b.img, None, &mut []);
    assert_eq!(nt, 38);
    let twin: [[f64; 3]; 20] = [
        [402.79524679120351, 342.01110964608188, 275610.86868389987], [452.80796824241332, 461.65305011032717, 255737.29055605855],
        [887.27563194454638, 62.196517181644161, 247718.92747571148], [160.35779582590064, 950.06023120997827, 181863.61288071173],
        [256.03278172498545, 23.612452061314553, 149422.12924737041], [348.8567178665877, 189.65437973316784, 142732.60456918163],
        [773.04976472816952, 546.09222778585467, 107492.36311139169], [14.034319832604874, 1017.9126261522238, 101630.2341468402],
        [186.65686367227065, 188.83508032051378, 97220.97721828823], [566.17373312988741, 309.12228628923657, 86075.180296114893],
        [51.229480525401286, 801.62111311448348, 68964.984490497169], [9.0696438475640502, 663.00032908998685, 62663.335258973821],
        [904.25938247459055, 1003.2842037018354, 48062.898297182306], [706.03213561477514, 423.98417144594674, 42335.81788271133],
        [478.23057030707128, 764.9817621433607, 17649.512797092248], [610.64811128167059, 855.74257170138185, 17459.924640974481],
        [319.36998037063682, 816.17408551664755, 15408.7852604334], [417.34730734840588, 919.34484454577296, 15865.779421626823],
        [224.25127504443947, 699.02664475970425, 14387.963875018171], [773.00827469410081, 916.8667833984988, 14011.925350572936]];
    let mut spots = [[0.0; 3]; MAX_SPOTS];
    let ns = stc::centroid(&b.img, &cam, &mut b.scratch, &mut spots);
    assert_eq!(ns, 20);
    for k in 0..20 {
        assert!((spots[k][0] - twin[k][0]).abs() < 1e-9 && (spots[k][1] - twin[k][1]).abs() < 1e-9, "spot {k}: {:?} vs the twin's {:?}", spots[k], twin[k]);
        assert!((spots[k][2]/twin[k][2] - 1.0).abs() < 1e-9, "spot {k} flux: {} vs the twin's {}", spots[k][2], twin[k][2]);
    }
    let eye = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    let (qm, ok, info) = stc::chain(&q, &eye, &eye, &cat, &cam, &mut work(&mut b), None);
    assert!(ok);
    assert_eq!((info.spots, info.identified, info.used), (20, 20, 20));
    let twin_q = [0.10259925829569796, -0.20519218143710233, 0.30779129307304492, 0.92338187159088136];
    assert!(arcsec(&qm, &twin_q) < 1e-6, "{qm:?} vs the twin's {twin_q:?}: {} arcsec", arcsec(&qm, &twin_q));
}

/// The quadrant currents are the closed form, and the angles invert them exactly while the
/// spot covers the centre (|tan| < a / 2h, 39.8 deg for the baseline head); the engine's
/// currents are the twin's (asils.comp.sun_sensor at 12 deg, -25 deg).
#[test]
fn quadrant_currents_invert_at_known_angles() {
    let p = Head { a: 1.0e-3, h: 0.6e-3, noise: 0.0, min_frac: 0.05 };
    // closed form at alpha 10 deg, beta 0: the spot shifted dx = h tan(10 deg) in x
    let s = unit(&[10f64.to_radians().tan(), 0.0, 1.0]);
    let i = currents(&s, &p, None);
    let dx = p.h*10f64.to_radians().tan();
    let c = s[2];
    let want = [c*(0.5e-3 + dx)*0.5e-3/1e-6, c*(0.5e-3 - dx)*0.5e-3/1e-6, c*(0.5e-3 - dx)*0.5e-3/1e-6, c*(0.5e-3 + dx)*0.5e-3/1e-6];
    for k in 0..4 { assert!((i[k] - want[k]).abs() < 1e-15, "quadrant {k}: {} vs {}", i[k], want[k]); }
    let mut worst: f64 = 0.0;
    for a in (-39..=39).step_by(3) {
        for bb in (-39..=39).step_by(3) {
            let s = unit(&[(a as f64).to_radians().tan(), (bb as f64).to_radians().tan(), 1.0]);
            let (sh, ok) = angles(&currents(&s, &p, None), &p);
            assert!(ok);
            worst = worst.max(dot(&s, &sh).min(1.0).acos());
        }
    }
    assert!(worst < 1e-7, "worst {worst} rad");                    // acos near 1: 1e-7 rad is the floor of the measure
    // past the linear range the spot leaves a quadrant pair and the answer saturates
    let s = unit(&[50f64.to_radians().tan(), 0.0, 1.0]);
    let (sh, _) = angles(&currents(&s, &p, None), &p);
    assert!((sh[0]/sh[2] - p.a/(2.0*p.h)).abs() < 1e-12);
    // the Sun behind the head: no current, not valid
    assert!(!angles(&currents(&[0.0, 0.0, -1.0], &p, None), &p).1);
    // the twin's currents and angles at 12 deg, -25 deg
    let s = unit(&[12f64.to_radians().tan(), (-25f64).to_radians().tan(), 1.0]);
    let i = currents(&s, &p, None);
    let twin = [0.12298383871970874, 0.072995743368318142, 0.25847853897287454, 0.43548680351329888];
    for k in 0..4 { assert!((i[k] - twin[k]).abs() < 1e-15, "quadrant {k}: {} vs the twin's {}", i[k], twin[k]); }
    let (sh, _) = angles(&i, &p);
    let twin_s = [0.18916363324317906, -0.41498813366512194, 0.88994492457420016];
    for k in 0..3 { assert!((sh[k] - twin_s[k]).abs() < 1e-15, "{sh:?} vs the twin's {twin_s:?}"); }
}

/// The Sun sensor's chain level: the head frame on each normal, the current noise from the head,
/// and the answer within the noise's reach of the truth; dark below min_frac.
#[test]
fn the_sun_sensor_chain_level_answers_through_its_head() {
    let mut d = SunDesc { fitted: true, n: 2, fov: 1.05, chain: true, head: Head { a: 1.0e-3, h: 0.6e-3, noise: 0.005, min_frac: 0.05 }, ..Default::default() };
    d.normals[0] = [0.0, 0.0, -1.0];
    d.normals[1] = [1.0, 0.0, 0.0];
    let mut s = Sun::new(d, &mut Rng::new(1, 1), Rng::new(1, 2));
    let mut worst: f64 = 0.0;
    for k in 0..200 {
        let a = 0.6*(k as f64/200.0 - 0.5);
        let sb = unit(&[a.sin(), 0.3*a.cos(), -1.0]);
        let z = s.sample(&sb, 1.0).expect("sunlit, inside the field");
        worst = worst.max(dot(&z, &sb).min(1.0).acos());
    }
    // 0.5 % current noise over a 1 mm aperture at 0.6 mm: about 0.3 deg rms per axis
    assert!(worst < 2f64.to_radians(), "worst {} deg", worst.to_degrees());
    assert!(s.sample(&[1.0, 0.0, 0.0], 1.0).is_some());
    assert!(s.sample(&[0.0, 0.0, -1.0], 0.0).is_none(), "eclipse");
    // the head frame: z on the normal, right-handed
    let r = head_frame(&[1.0, 0.0, 0.0]);
    assert!((det(&r) - 1.0).abs() < 1e-15 && r[2] == [1.0, 0.0, 0.0]);
}

/// The image model in the unit: a valid head answers within 60 arcsec through its true mount
/// (the mount error stays a unit error), and nothing is rendered for a head that is not valid.
#[test]
fn the_image_model_answers_in_the_unit() {
    let mut d = StDesc { fitted: true, nh: 1, noise_cross: 1e-4, noise_roll: 1e-3, rate_hz: 5.0, latency: 0.0, max_rate: 1.0,
        sun_excl: 0.5, earth_excl: 0.3, fov: 0.17, model: 2, moon_excl: 0.26, blind_s: 0.0, noise_rate_ref: 0.01, cam: camera(), ..Default::default() };
    d.bs[0] = [0.0, 1.0, 0.0];
    let mut st = St::new(d, &mut Rng::new(1, 1), Rng::new(1, 3));
    let mut b = bufs(st.catalogue(), &d.cam);
    let q = qnorm(&[0.2, 0.1, -0.3, 0.9]);
    let (sun, moon, nadir) = ([0.0, -1.0, 0.0], [0.0, -1.0, 0.0], [0.0, 0.0, 1.0]);
    let z = st.sample_with(&q, 0.0, &[0.0; 3], &sun, &moon, &nadir, 1.0, Some(&mut work(&mut b)))[0].expect("a clear sky");
    assert!(arcsec(&q, &z) < 60.0, "{} arcsec", arcsec(&q, &z));
    // a mount bias of 100 arcsec about the boresight shows in the answer
    st.q_bias[0] = fromrotvec(&[0.0, 100.0/3600.0*std::f64::consts::PI/180.0, 0.0]);
    let z = st.sample_with(&q, 0.2, &[0.0; 3], &sun, &moon, &nadir, 1.0, Some(&mut work(&mut b)))[0].unwrap();
    assert!((arcsec(&q, &z) - 100.0).abs() < 30.0, "{} arcsec", arcsec(&q, &z));
    // the Sun in the cone: no frame, no answer
    b.img[0] = -1.0;
    assert!(st.sample_with(&q, 0.4, &[0.0; 3], &[0.0, 1.0, 0.0], &moon, &nadir, 1.0, Some(&mut work(&mut b)))[0].is_none());
    assert_eq!(b.img[0], -1.0, "rendered for a head that was not valid");
}
