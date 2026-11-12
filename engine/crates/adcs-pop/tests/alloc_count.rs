//! P8a: World::accel evaluates the force model without cloning the force set or the
//! facets. A counting global allocator (own test binary, so no other test's allocations
//! count) measures the allocations of one accel call with relativity and ERP on.
use adcs_pop::accel::{Forces, Sc, World};
use adcs_pop::frames::{Build, FrameOpt};
use adcs_pop::gravity::Field;
use adcs_pop::la::I3;
use adcs_pop::{relativity, srp};
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

struct Counting;
static ALLOCS: AtomicUsize = AtomicUsize::new(0);
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 { ALLOCS.fetch_add(1, Ordering::SeqCst); System.alloc(l) }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) { System.dealloc(p, l) }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 { ALLOCS.fetch_add(1, Ordering::SeqCst); System.realloc(p, l, n) }
    unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 { ALLOCS.fetch_add(1, Ordering::SeqCst); System.alloc_zeroed(l) }
}
#[global_allocator]
static A: Counting = Counting;

fn world(model: &str) -> World {
    let mut forces = Forces::sils(1.3);
    forces.drag = None;
    forces.relativity = Some(vec![relativity::Term::Schwarzschild, relativity::Term::LenseThirring, relativity::Term::DeSitter]);
    forces.erp = Some((model.to_string(), None, None, None, None));
    let facets = srp::sat16u().facets;
    let sc = Sc { mass: 12.0, aref: 0.07, cd: Some(2.2), cr: Some(1.3), r_bi: I3, srp_facets: facets, drag_facets: None };
    World::new([2025.0, 3.0, 20.0, 12.0, 0.0, 0.0], Build::Gmst, FrameOpt::default(), Field::default_field(), forces, sc, None, None).unwrap()
}

#[test]
fn accel_allocations_per_call() {
    for model in ["knocke", "boxwing"] {
        let mut w = world(model);
        let (r, v) = ([6.9e6, 1.0e5, 2.0e5], [-100.0, 7.5e3, 300.0]);
        w.accel(0.0, &r, &v).unwrap(); // warm up
        let n = 20;
        let before = ALLOCS.load(Ordering::SeqCst);
        let mut a = [0.0; 3];
        for k in 0..n { a = w.accel(k as f64, &r, &v).unwrap().0; }
        let per = (ALLOCS.load(Ordering::SeqCst) - before) as f64 / n as f64;
        eprintln!("{model}: {per} allocations per accel call (a {a:?})");
        assert_eq!(per, 0.0, "{model}: World::accel allocates {per} times per call");
    }
}
