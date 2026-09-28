//! Engine tests: LQR against the MATLAB twin, determinism (spec §9.6), C = Rust
//! flight software through the bytes, momentum conservation, gravity model.
use adcs_fsw_abi::Impl;
use adcs_sim::{config::Config, lqr, run};
use adcs_sim_core::la::*;
use adcs_sim_core::orbit::{accel, Ctx, OrbitCfg, MU, RE};
use adcs_sim_core::plant::{self, Body, Geometry, State};
use std::path::PathBuf;

fn root() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../matlab_sils") }

fn cfg(s: &str, dur: f64) -> Config {
    let r = root();
    let case = if s.contains("img") || s.contains("cmg") || s.contains("fmr") || s.contains("rw_rcs") { "cases/ais_img_3u.csv" } else { "cases/ais_3u.csv" };
    Config::build(&r, s, &r.join(case), 3, &[("engine.duration_s".into(), dur.to_string())]).unwrap()
}

#[test]
fn lqr_matches_matlab() {
    // asils.fsw.lqr_gain (Hamiltonian eigenvectors) in Octave, printed %.12e
    let (i, wn, th): (f64, f64, f64) = (0.042, 0.9, 1e-3);
    let k = lqr::chain3(1.0/i, [(wn/(0.5*th)).powi(2), 1.0/(th*th), 1.0/(wn*th).powi(2)], 1.0/(i*wn*wn*th).powi(2), wn);
    for (a, b) in k.iter().zip([6.123600000000e-02, 1.194571555706e-01, 1.070665263653e-01]) { assert!((a - b).abs() < 1e-9*b.abs(), "{k:?}"); }
    let (i, wn, th): (f64, f64, f64) = (0.0067, 0.005, 0.05);
    let k = lqr::chain3(1.0/i, [1e-12, 1.0/(th*th), 1.0/(wn*th).powi(2)], 1.0/(i*wn*wn*th).powi(2), wn);
    for (a, b) in k.iter().skip(1).zip([1.675029011767e-07, 5.802403705162e-05]) { assert!((a - b).abs() < 1e-6*b.abs(), "{k:?}"); }
}

#[test]
fn run_twice_identical_and_c_equals_rust() {
    let c = cfg("fine_hold_img", 300.0);
    let a = run::run(&c, &run::Opts { fsw: Impl::C, quiet: true }).unwrap();
    let b = run::run(&c, &run::Opts { fsw: Impl::C, quiet: true }).unwrap();
    let r = run::run(&c, &run::Opts { fsw: Impl::Rust, quiet: true }).unwrap();
    assert_eq!(a.rows.len(), r.rows.len());
    for ((x, y), z) in a.rows.iter().zip(&b.rows).zip(&r.rows) {
        assert_eq!((x.q, x.w, x.h_w, x.m), (y.q, y.w, y.h_w, y.m), "same process, same run, same bytes");
        assert_eq!((x.q, x.w, x.h_w, x.m, x.mode), (z.q, z.w, z.h_w, z.m, z.mode), "C and Rust flight software, t = {}", x.t);
    }
}

#[test]
fn detumble_reduces_rate() {
    let c = cfg("detumble_ais", 3600.0);
    let r = run::run(&c, &run::Opts { fsw: Impl::Rust, quiet: true }).unwrap();
    let (w0, w1) = (norm(&r.rows[0].w), norm(&r.rows.last().unwrap().w));
    assert!(w1 < 0.2*w0, "B-dot: {} -> {} deg/s", w0.to_degrees(), w1.to_degrees());
}

#[test]
fn momentum_conserved_without_external_torque() {
    let geo = Geometry::new(&[[1.0, 0.0, 0.0], [0.0, 0.7071, 0.7071]], &[[0.0, 0.7071, -0.7071]], &[0, 1]);
    let i = [[0.05, 0.001, 0.0], [0.001, 0.04, 0.0], [0.0, 0.0, 0.03]];
    let b = Body { i, iinv: inv(&i), m: geo };
    let mut x = State { q: [0.1, 0.2, 0.3, 0.927], w: [0.05, -0.1, 0.2], ..Default::default() };
    x.q = qnorm(&x.q);
    x.h[0] = 0.003; x.h[1] = 0.004;
    let h_i = |x: &State| mtv(&dcm(&x.q), &plant::momentum(x, &b));
    let h0 = h_i(&x);
    let mut tr = [0.0; 8]; tr[0] = 1e-4; tr[1] = -2e-4;
    let mut gd = [0.0; 4]; gd[0] = 0.3;
    for _ in 0..2000 { x = plant::step(&x, 0.01, &b, &[0.0; 3], &tr, &gd); }
    let e = norm(&sub(&h_i(&x), &h0))/norm(&h0);
    assert!(e < 1e-9, "inertial momentum drift {e}");
}

#[test]
fn zonal_gravity_is_gradient_of_potential() {
    let cfg = OrbitCfg { jd0_utc: 2461407.25, step_s: 10.0, zonal_max: 6, third_body: false, drag: false, srp: false, mass_kg: 1.0, area_m2: 0.0, cd: 0.0, cr: 0.0, density_scale: 0.0 };
    let jn = [0.0, 0.0, 1.08262668e-3, -2.53265649e-6, -1.61962159e-6, -2.27296083e-7, 5.40681239e-7];
    let u = |r: &V3| {
        let rn = norm(r); let s = r[2]/rn;
        let p = [1.0, s, 0.5*(3.0*s*s - 1.0), 0.5*(5.0*s*s*s - 3.0*s), (35.0*s.powi(4) - 30.0*s*s + 3.0)/8.0,
                 (63.0*s.powi(5) - 70.0*s.powi(3) + 15.0*s)/8.0, (231.0*s.powi(6) - 315.0*s.powi(4) + 105.0*s*s - 5.0)/16.0];
        MU/rn*(1.0 - (2..7).map(|n| jn[n]*(RE/rn).powi(n as i32)*p[n]).sum::<f64>())
    };
    let r = [4.1e6, -3.3e6, 4.4e6];
    let a = accel(&cfg, &r, &[0.0; 3], &Ctx::default());
    for i in 0..3 {
        let h = 1.0;
        let (mut rp, mut rm) = (r, r); rp[i] += h; rm[i] -= h;
        let g = (u(&rp) - u(&rm))/(2.0*h);
        assert!((a[i] - g).abs() < 1e-7, "axis {i}: {} vs {}", a[i], g);
    }
}
