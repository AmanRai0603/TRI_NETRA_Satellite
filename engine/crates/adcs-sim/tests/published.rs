//! The engine's design-time algorithms against outside references: the LQR gain against SciPy's
//! continuous algebraic Riccati solver (scipy.linalg.solve_continuous_are 1.17.1; one case is the
//! textbook closed form for a triple integrator), and the B-dot gain against the formula of
//! Avanzini & Giulietti (2012), "Magnetic detumbling of a rigid spacecraft", JGCD 35(4):
//! k = 2 n (1 + sin xi_m) J_min.
use adcs_sim::config::Config;
use adcs_sim::{flight, lqr};
use std::path::PathBuf;

fn root() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../matlab_sils") }

/// (b, Q diagonal, R, start, K from SciPy) for A = [0 1 0; 0 0 1; 0 0 0], B = [0; 0; b].
const SCIPY: [(f64, [f64; 3], f64, f64, [f64; 3]); 4] = [
    (149.2537313432836, [1e-06, 1.0, 0.5], 2.0, 0.1, [0.0007071067811865876, 0.7076159916476974, 0.5093938106102973]),
    (20.0, [0.0032, 0.81, 0.9], 1.0, 0.05, [0.05656854249492395, 0.9606843723022889, 0.9980322826593482]),
    // Q = I, R = 1, b = 1: K = [1, 1 + sqrt 2, 1 + sqrt 2] in closed form
    (1.0, [1.0, 1.0, 1.0], 1.0, 0.5, [1.0000000000000049, 2.414213562373101, 2.414213562373098]),
    (33.333333333333336, [1e-09, 0.0001, 0.01], 1000.0, 0.02, [9.999999999999324e-07, 0.0003330943300132181, 0.005475916343480034]),
];

#[test]
fn lqr_gains_are_scipys_riccati_solution() {
    for (b, q, r, p0, want) in SCIPY {
        let k = lqr::chain3(b, q, r, p0);
        for i in 0..3 { assert!(((k[i] - want[i])/want[i]).abs() < 1e-9, "b {b} Q {q:?} R {r}: K[{i}] {} vs SciPy {}", k[i], want[i]); }
    }
    let s = 1.0 + 2f64.sqrt();
    let k = lqr::chain3(1.0, [1.0; 3], 1.0, 0.5);
    assert!((k[0] - 1.0).abs() < 1e-12 && (k[1] - s).abs() < 1e-12 && (k[2] - s).abs() < 1e-12, "{k:?}");
}

#[test]
fn the_bdot_gain_is_avanzini_and_giuliettis() {
    let r = root();
    let set = [("fsw.bdot_gain_scale".to_string(), "1".to_string())];
    let c = Config::build(&r, "detumble_ais", &flight::case_file(&r, "detumble_ais", None).unwrap(), 1, &set).unwrap();
    let (mu, a) = (3.986004418e14, 6378137.0 + 1e3*c.case.get("orbit.alt"));
    let n = (mu/(a*a*a)).sqrt();
    let j = c.params.J;
    let jmin = j[0][0].min(j[1][1]).min(j[2][2]);
    let want = 2.0*n*(1.0 + c.case.get("orbit.inc").to_radians().sin())*jmin;
    assert!(((c.params.bdot_k - want)/want).abs() < 1e-12, "bdot_k {} vs 2 n (1 + sin i) J_min = {want}", c.params.bdot_k);
    // the shipped scenarios fly three times it (bdot_gain_scale = 3, the paper's margin for a slow first orbit)
    let c3 = Config::build(&r, "detumble_ais", &flight::case_file(&r, "detumble_ais", None).unwrap(), 1, &[]).unwrap();
    assert!(((c3.params.bdot_k - 3.0*want)/want).abs() < 1e-12);
}
