//! The ECSS-E-ST-60-10C error indices on signals whose answers are known by hand: a constant bias
//! (all mean, no relative error), a sine faster than the window (all relative), a ramp (drift).
use adcs_sim::metrics::{ecss, Derived};

fn signal(f: impl Fn(f64) -> [f64; 3]) -> (Derived, Vec<f64>, Vec<usize>) {
    let t: Vec<f64> = (0..1000).map(|k| k as f64*0.1).collect();       // 100 s at 10 Hz
    let e: Vec<[f64; 3]> = t.iter().map(|&x| f(x)).collect();
    let d = Derived { e_ape: e.clone(), e_ake: e, bs: [0.0, 0.0, 1.0], ..Default::default() };
    (d, t, (0..1000).collect())
}
fn max(v: &[f64]) -> f64 { v.iter().cloned().fold(f64::MIN, f64::max) }
fn close(a: f64, b: f64, tol: f64) -> bool { (a - b).abs() <= tol }

#[test]
fn a_bias_is_all_mean_error() {
    let b = 1e-3;                                                        // rad about x
    let (d, t, i) = signal(|_| [b, 0.0, 0.0]);
    assert!(max(&ecss("rpe", &d, &t, &i, 10.0, f64::NAN)) < 1e-12, "no relative error");
    let m = ecss("mpe", &d, &t, &i, 10.0, f64::NAN);
    assert_eq!(m.len(), 10, "ten 10 s blocks");
    assert!(m.iter().all(|&x| close(x, b.to_degrees(), 1e-12)));
    assert!(max(&ecss("pde", &d, &t, &i, 10.0, 30.0)) < 1e-12, "no drift");
}

#[test]
fn a_fast_sine_is_all_relative_error() {
    let a = 2e-3;                                                        // 2.5 Hz: sampled at its peaks, 25 periods per 10 s block
    let (d, t, i) = signal(|x| [0.0, a*(2.0*std::f64::consts::PI*2.5*x).sin(), 0.0]);
    assert!(close(max(&ecss("rpe", &d, &t, &i, 10.0, f64::NAN)), a.to_degrees(), 1e-9), "the relative error is the amplitude");
    assert!(max(&ecss("mpe", &d, &t, &i, 10.0, f64::NAN)) < 1e-12, "whole periods average to zero");
}

#[test]
fn a_ramp_is_drift_and_the_boresight_component_is_dropped_for_los() {
    let r = 1e-5;                                                        // rad/s about x, and as much about the boresight z
    let (d, t, i) = signal(|x| [r*x, 0.0, r*x]);
    // block means at 4.95 + 10k s; 30 s apart they differ by 30 r on each of two axes
    let pde = ecss("pde", &d, &t, &i, 10.0, 30.0);
    assert_eq!(pde.len(), 7);
    assert!(pde.iter().all(|&x| close(x, (30.0*r*2f64.sqrt()).to_degrees(), 1e-9)), "{pde:?}");
    let los = ecss("pde_los", &d, &t, &i, 10.0, 30.0);
    assert!(los.iter().all(|&x| close(x, (30.0*r).to_degrees(), 1e-9)), "about the boresight is not a line-of-sight error");
    // within a block a ramp leaves at most half its span about the mean
    assert!(close(max(&ecss("rpe_los", &d, &t, &i, 10.0, f64::NAN)), (4.95*r).to_degrees(), 1e-9));
}

#[test]
fn knowledge_indices_read_the_knowledge_error_and_gaps_are_skipped() {
    let (mut d, t, i) = signal(|_| [0.0; 3]);
    d.e_ake = t.iter().map(|&x| if x < 20.0 { [f64::NAN; 3] } else { [0.0, 1e-3, 0.0] }).collect();
    let m = ecss("mke", &d, &t, &i, 10.0, f64::NAN);
    assert_eq!(m.len(), 8, "the two blocks with no estimate are skipped");
    assert!(max(&ecss("mpe", &d, &t, &i, 10.0, f64::NAN)) < 1e-12, "performance error untouched");
    assert!(ecss("rpe", &d, &t, &[], 10.0, f64::NAN).is_empty() && ecss("rpe", &d, &t, &i, 0.0, f64::NAN).is_empty());
}
