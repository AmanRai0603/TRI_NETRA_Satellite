//! End-to-end: the Rust in-loop POP against asils.orbit (MATLAB POP) for both cases
//! (refgen/inloop.m -> tests/data/inloop.txt).
use adcs_pop::accel::{sso_initial, Forces, InLoop, Sc, World};
use adcs_pop::frames::{Build, FrameOpt};
use adcs_pop::gravity::Field;
use adcs_pop::la::I3;
use adcs_pop::spaceweather::ManualIndices;
use adcs_pop::atmos::dtm2020::KpIn;
use std::collections::HashMap;

fn cases() -> Vec<(HashMap<String, Vec<f64>>, Vec<Vec<f64>>)> {
    let txt = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/inloop.txt")).unwrap();
    let mut out: Vec<(HashMap<String, Vec<f64>>, Vec<Vec<f64>>)> = vec![];
    for l in txt.lines() {
        let mut it = l.split_whitespace();
        let k = it.next().unwrap();
        if k == "case" { out.push((HashMap::new(), vec![])); continue; }
        let v: Vec<f64> = it.map(|x| x.parse().unwrap()).collect();
        let c = out.last_mut().unwrap();
        if k == "sample" { c.1.push(v); } else { c.0.insert(k.to_string(), v); }
    }
    out
}

fn rel(a: &[f64], b: &[f64]) -> f64 {
    let n = b.iter().map(|x| x*x).sum::<f64>().sqrt().max(1e-300);
    a.iter().zip(b).map(|(x, y)| (x - y)*(x - y)).sum::<f64>().sqrt()/n
}

#[test]
fn inloop_matches_matlab_pop() {
    for (k, samples) in cases() {
        let g = |n: &str| k[n].clone();
        let e = g("epoch");
        let sw = g("sw");
        let world = World::new([e[0], e[1], e[2], e[3], e[4], e[5]], Build::Gmst, FrameOpt::default(), Field::default_field(), Forces::sils(g("cr")[0]),
            Sc { mass: g("mass")[0], aref: g("aref")[0], cd: Some(g("cd")[0]), cr: Some(g("cr")[0]), r_bi: I3, srp_facets: vec![], drag_facets: None },
            Some(ManualIndices { f107: sw[0], f107a: Some(sw[1]), kp: Some(KpIn::Scalar(sw[2])), ap: Some(sw[3]), ap3: None }), None).unwrap();
        let (r0, v0, raan) = sso_initial(&world, g("alt_km")[0], g("ecc")[0], g("inc_deg")[0], g("ltan_h")[0], g("argp_deg")[0], g("u0_deg")[0]).unwrap();
        assert!((raan - g("raan")[0]).abs() < 1e-15, "raan {raan} vs {}", g("raan")[0]);
        let y0 = g("y0");
        assert!(rel(&[r0[0], r0[1], r0[2], v0[0], v0[1], v0[2]], &y0) < 1e-15, "y0");
        let omega = g("omega_eci");
        assert!(rel(&world.omega_eci, &omega) < 1e-15, "omega_eci");
        let mut o = InLoop::new(world, g("step")[0], r0, v0).unwrap();
        let (a0, p, ..) = o.world.accel(0.0, &r0, &v0).unwrap();
        let mut worst = [0.0f64; 8];
        for (i, (x, name)) in [(a0, "a0"), (p.gravity, "p_gravity"), (p.thirdbody, "p_thirdbody"), (p.drag, "p_drag"), (p.srp, "p_srp")].iter().enumerate() {
            let r = rel(x, &g(name));
            worst[i] = r;
            assert!(r < 1e-12, "{name}: rel {r}");
        }
        for s in &samples {
            let t = s[0];
            let (r, v) = o.state(t).unwrap();
            let x = o.context(t);
            let dr = rel(&r, &s[1..4])*6.9e6;
            let dv = rel(&v, &s[4..7])*7.6e3;
            eprintln!("t = {t:7.1}: |dr| {dr:.3e} m  |dv| {dv:.3e} m/s  sun {:.1e}  rho {:.1e}", rel(&x.sun_eci, &s[7..10]), (x.rho - s[14]).abs()/s[14]);
            assert!(dr < 1e-6 && dv < 1e-9, "state at t = {t}: {dr} m, {dv} m/s");
            assert!(rel(&x.sun_eci, &s[7..10]) < 1e-14 && rel(&x.moon_eci, &s[10..13]) < 1e-14, "ephemeris at {t}");
            assert!((x.p_srp - s[13]).abs() < 1e-15*s[13] && (x.rho - s[14]).abs() < 1e-12*s[14], "p_srp / rho at {t}");
            let c: Vec<f64> = x.c.iter().flatten().copied().collect();
            assert!(rel(&c, &s[15..24]) < 1e-14, "C at {t}");
        }
        eprintln!("parts worst rel: {worst:?}");
    }
}
