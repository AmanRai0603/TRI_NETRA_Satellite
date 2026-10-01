//! Rotor-imbalance jitter (= asils.sizing.jitter): the eigenvalue it divides by, and a flight that
//! computes it where the parts state their imbalance and declines where they do not.
use adcs_sim::config::Config;
use adcs_sim::metrics::{eig_min3, jitter};
use adcs_sim::{flight, run};

fn root() -> std::path::PathBuf { std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../matlab_sils") }

#[test]
fn the_smallest_principal_inertia() {
    assert!((eig_min3(&[[3.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 2.0]]) - 1.0).abs() < 1e-15, "diagonal");
    // [[2,1,0],[1,2,0],[0,0,5]]: eigenvalues 1, 3, 5
    assert!((eig_min3(&[[2.0, 1.0, 0.0], [1.0, 2.0, 0.0], [0.0, 0.0, 5.0]]) - 1.0).abs() < 1e-12);
    // [[4,1,1],[1,4,1],[1,1,4]]: eigenvalues 3, 3, 6
    assert!((eig_min3(&[[4.0, 1.0, 1.0], [1.0, 4.0, 1.0], [1.0, 1.0, 4.0]]) - 3.0).abs() < 1e-12);
}

#[test]
fn a_flight_on_wheels_with_stated_imbalance_has_its_jitter_in_closed_form() {
    let r = root();
    let c = Config::build(&r, "fine_hold_img", &flight::case_file(&r, "fine_hold_img", None).unwrap(), 1, &[("engine.duration_s".into(), "60".into())]).unwrap();
    assert!(c.dev.mex.n >= 3 && c.dev.imbalance.len() == c.dev.mex.n, "the imaging product flies wheels");
    assert!(c.dev.imbalance.iter().all(|u| u.is_some()), "the imaging product's wheels state their imbalance");
    let rec = run::run(&c, &run::Opts { fsw: adcs_fsw_abi::Impl::C, quiet: true, realtime: false, oils: None }).unwrap();
    let idx: Vec<usize> = (0..rec.rows.len()).collect();
    let j = jitter(&c, &rec, &idx);
    assert_eq!(j.len(), idx.len());
    // by hand at the last sample, every rotor root-sum-square
    let k = idx.len() - 1;
    let (jmin, d, wbw) = (eig_min3(&c.inertia), c.box_m.iter().cloned().fold(f64::INFINITY, f64::min)/2.0, 0.9);
    let want: f64 = (0..c.dev.mex.n).map(|i| {
        let (us, ud) = c.dev.imbalance[i].unwrap();
        let w = rec.rows[k].h_w[i].abs()/c.dev.mex.jrot[i];
        ((us*d + ud)*w*w/(jmin*(w*w).max(wbw*wbw))).powi(2)
    }).sum::<f64>().sqrt().to_degrees()*3600.0;
    assert!((j[k] - want).abs() <= 1e-12*want.max(1e-30), "{} vs {want}", j[k]);
    assert!(j.iter().all(|x| x.is_finite() && *x >= 0.0));
}
