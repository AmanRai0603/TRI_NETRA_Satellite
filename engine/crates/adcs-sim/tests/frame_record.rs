//! S12.4a (P9, DR-env-002): a run records the Earth frame it flew and whether the EOP and leap-second snapshot was valid to
//! the end of the run (the manifest's `frame`: eop_flag, leap_status) and warns when it was not; it does not refuse.
use adcs_pop::accel::{Forces, InLoop, Sc, World};
use adcs_pop::frames::{Build, FrameOpt};
use adcs_sim::run::Truth;

fn truth(epoch: [f64; 6]) -> Truth {
    assert!(adcs_sim::pop_kernel_missing().is_none(), "the DE440 kernel ships with the checkout");
    let sc = Sc { mass: 12.0, aref: 0.07, cd: Some(2.2), cr: Some(1.3), r_bi: adcs_pop::la::I3, srp_facets: vec![], drag_facets: None };
    let opt = FrameOpt { eop: Some(adcs_pop::eop::Eop::from_snapshot()), ..Default::default() };
    let w = World::new(epoch, Build::A, opt, adcs_pop::gravity::Field::default_field(), Forces::sils(1.3), sc,
                       Some(adcs_pop::spaceweather::ManualIndices { f107: 150.0, f107a: Some(150.0), kp: Some(adcs_pop::atmos::dtm2020::KpIn::Scalar(2.0)), ap: Some(7.0), ap3: None }),
                       Some(adcs_sim::pop_kernel())).unwrap();
    let (r, v) = ([6.9e6, 0.0, 0.0], [0.0, 5.0e3, 5.5e3]);
    Truth::Pop(Box::new(InLoop::new(w, 10.0, r, v).unwrap()))
}

#[test]
fn a_build_a_run_past_the_snapshots_validity_records_it_and_warns() {
    let inside = truth([2027., 1., 1., 6., 0., 0.]);
    let f = inside.frame_record(3600.0).unwrap();
    assert_eq!((f.build, f.eop_flag, f.leap_status), (Build::A, 0, 0));
    assert!(f.warning().is_none());
    // a run that crosses the leap-second table's expiry (28 June 2027, MJD 61584): leap_status, a warning, the header
    let crossing = truth([2027., 6., 27., 12., 0., 0.]);
    let f = crossing.frame_record(2.0*86400.0).unwrap();
    assert_eq!((f.eop_flag, f.leap_status), (0, 1));
    assert!(f.warning().unwrap().contains("leap-second table expired at MJD 61584"));
    // past the EOP table's last day (MJD 61714): eop_flag 1 in the header
    let past = truth([2027., 11., 20., 0., 0., 0.]);
    let f = past.frame_record(86400.0).unwrap();
    assert_eq!((f.eop_flag, f.leap_status), (1, 1));
    let j = f.json();
    assert_eq!((j["build"].as_str(), j["eop_flag"].as_i64(), j["leap_status"].as_i64(), j["eop_valid_to_mjd"].as_f64()), (Some("A"), Some(1), Some(1), Some(61714.0)));
}
