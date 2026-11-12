//! TS1 analytic oracles for the plant and the environment models: each is a published closed form, cited at
//! the test, against the engine's own function (the generated model or the plant), which is called, not changed.
use adcs_sim_core::gen::constants::{AU_M, R_E};
use adcs_sim_core::gen::shadow::shadow_fraction;
use adcs_sim_core::la::*;
use adcs_sim_core::plant::{self, Body, Geometry, State};
use adcs_sim_core::torques::{self, Facets};
use adcs_sim_core::{NG, NR};

const MU: f64 = 3.986004418e14;

fn rigid(i: V3) -> Body { Body::rigid(diag(&i), Geometry::new(&[], &[], &[])) }

/// TS1(a). Torque-free axisymmetric body, Hughes, Spacecraft Attitude Dynamics (Wiley 1986), Chapter 3
/// (Section 3.3, torque-free motion of an axisymmetric body): the axial rate w3 is constant and the transverse rate
/// rotates in the body frame at lambda = (I3 - I1)/I1 * w3; the angle between the angular momentum and the
/// symmetry axis, tan(theta) = I1 w_perp/(I3 w3), is constant.
#[test]
fn torque_free_axisymmetric_body_precesses_at_hughes_rate() {
    let (i1, i3) = (0.02, 0.05);
    let b = rigid([i1, i1, i3]);
    let mut x = State { q: [0.0, 0.0, 0.0, 1.0], w: [0.01, 0.0, 0.1], ..Default::default() };
    let lambda = (i3 - i1)/i1*0.1;
    assert!((lambda - 0.15).abs() < 1e-15);
    let nut = |x: &State| (i1*x.w[0].hypot(x.w[1])).atan2(i3*x.w[2]);
    let th0 = nut(&x);
    let (dt, t_end): (f64, f64) = (0.005, 600.0);
    let n = (t_end/dt).round() as usize;
    let (mut ang_prev, mut ang) = (0.0f64, 0.0f64);
    let mut worst_nut = 0.0f64;
    for _ in 0..n {
        x = plant::step(&x, dt, &b, &[0.0; 3], &[0.0; NR], &[0.0; NG]);
        let a = x.w[1].atan2(x.w[0]);
        let mut d = a - ang_prev;
        while d > std::f64::consts::PI { d -= 2.0*std::f64::consts::PI; }
        while d < -std::f64::consts::PI { d += 2.0*std::f64::consts::PI; }
        ang += d; ang_prev = a;
        worst_nut = worst_nut.max((nut(&x) - th0).abs());
    }
    let rate = ang/t_end;
    eprintln!("precession rate {rate:.9} rad/s (Hughes {lambda}), nutation angle drift {worst_nut:.2e} rad, w3 {:.12}", x.w[2]);
    assert!((rate - lambda).abs() < 1e-6, "body-frame precession {rate} vs {lambda}");
    assert!(worst_nut < 1e-9, "nutation angle drift {worst_nut} rad");
}

/// TS1(b). Gravity-gradient pitch libration of a nadir-pointing body on a circular orbit, Hughes (1986) Chapter 9
/// (gravity-gradient stabilisation, pitch libration): small pitch angle about the orbit normal obeys theta'' + 3 n^2 (Ix - Iz)/Iy theta = 0, so the
/// libration period is 2 pi/(n sqrt(3 (Ix - Iz)/Iy)). x along the velocity, y the pitch axis, z nadir.
#[test]
fn gravity_gradient_pitch_libration_period() {
    let a = R_E + 550e3;
    let n = (MU/(a*a*a)).sqrt();
    let (ix, iy, iz) = (0.04, 0.05, 0.01);
    let b = rigid([ix, iy, iz]);
    let i = diag(&[ix, iy, iz]);
    let rn = |t: f64| [a*(n*t).cos(), a*(n*t).sin(), 0.0];
    // inertial to LVLH-body: rows velocity, -orbit normal, nadir (right handed)
    let lv = |t: f64| -> M3 { let (s, c) = (n*t).sin_cos(); [[-s, c, 0.0], [0.0, 0.0, -1.0], [-c, -s, 0.0]] };
    let d0 = 0.5f64.to_radians();
    let (sd, cd) = d0.sin_cos();
    let ry = [[cd, 0.0, -sd], [0.0, 1.0, 0.0], [sd, 0.0, cd]];
    let mut x = State { q: fromdcm(&mm(&ry, &lv(0.0))), w: [0.0, -n, 0.0], ..Default::default() };
    let want = 2.0*std::f64::consts::PI/(n*(3.0*(ix - iz)/iy).sqrt());
    let (dt, t_end) = (0.1, 4.0*want);
    let steps = (t_end/dt) as usize;
    let mut up: Vec<f64> = vec![];
    let mut prev = d0;
    for k in 0..steps {
        let t = k as f64*dt;
        let tau = torques::torques(&x.q, &rn(t), &[0.0; 3], &[0.0; 3], &[1.0, 0.0, 0.0], 1.0, 0.0, 0.0, &i, &Facets::boxed(&[0.1, 0.1, 0.1], &[0.0; 3], 0.8, 0.8, 0.05, 0.6, 0.5), &[0.0; 3], MU, [true, false, false, false])[0];
        x = plant::step(&x, dt, &b, &tau, &[0.0; NR], &[0.0; NG]);
        let rel = mm(&dcm(&x.q), &transpose(&lv(t + dt)));
        let th = (-rel[0][2]).asin();
        if prev < 0.0 && th >= 0.0 { up.push(t + dt*(-prev)/(th - prev)); }
        prev = th;
    }
    assert!(up.len() >= 3, "{} upward crossings", up.len());
    let period = (up[up.len() - 1] - up[0])/(up.len() - 1) as f64;
    eprintln!("libration period {period:.3} s, Hughes {want:.3} s ({:.4} %)", 100.0*(period/want - 1.0));
    assert!((period/want - 1.0).abs() < 0.005, "period {period} vs {want}");
}

/// TS1(d)/(e) helper: the sunlit fraction at the satellite angle phi from the anti-Sun direction, circular orbit
/// radius a, Sun in the orbit plane (beta 0) at one AU.
fn nu_at(a: f64, phi: f64) -> f64 { shadow_fraction([-a*phi.cos(), a*phi.sin(), 0.0], [AU_M, 0.0, 0.0]) }

/// TS1(e). Umbra duration of a 550 km circular orbit at beta 0, Vallado, Fundamentals of Astrodynamics and
/// Applications 4th ed., Section 5.3 (conical shadow geometry): the umbra cone has half angle
/// alpha = asin((Rsun - Re)/D); the satellite at angle phi from the anti-Sun axis is in umbra when
/// a sin(phi + alpha) / cos(alpha) < Re, so the umbra arc is 2 (asin(Re cos(alpha)/a) - alpha) and the duration arc/n.
#[test]
fn umbra_duration_matches_the_conical_closed_form() {
    let a = R_E + 550e3;
    let n = (MU/(a*a*a)).sqrt();
    let rs = adcs_sim_core::gen::shadow::SHADOW_R_SUN;
    let alpha = ((rs - R_E)/AU_M).asin();
    let phi_u = (R_E*alpha.cos()/a).asin() - alpha;
    let want = 2.0*phi_u/n;
    // the engine's umbra edge: nu == 0 inside, bisected from the anti-Sun axis outward
    let (mut lo, mut hi) = (0.0f64, 1.5f64);
    assert!(nu_at(a, lo) == 0.0 && nu_at(a, hi) > 0.0);
    for _ in 0..80 { let m = 0.5*(lo + hi); if nu_at(a, m) == 0.0 { lo = m } else { hi = m } }
    let got = 2.0*lo/n;
    eprintln!("umbra {got:.3} s, Vallado {want:.3} s ({:.3} s)", got - want);
    assert!((got - want).abs() < 2.0, "umbra {got} s vs {want} s");
}

/// TS1(g). Flat plate, normal incidence, -X facet 0.1 x 0.34 m, centre of mass 1 cm along +Y: the facet centre is at
/// (-0.05, -0.01, 0) from the centre of mass, so a force F along +X gives a torque 0.01 F about +Z.
/// SRP: Wertz (ed.), Spacecraft Attitude Determination and Control (1978), Eq. 17-6 at normal incidence, F = p A (1 + rho_s + 2 rho_d/3)
/// (Montenbruck and Gill, Satellite Orbits, Eq. 3.73 for the flat plate).
fn box_facets() -> Facets { Facets::boxed(&[0.1, 0.1, 0.34], &[0.0, 0.01, 0.0], 0.8, 0.8, 0.05, 0.6, 0.5) }

#[test]
fn flat_plate_srp_torque() {
    let (rho_s, rho_d) = (0.6*0.5, 0.6*0.5); // refl 0.6, spec_frac 0.5: Facets::boxed's rho_s = refl spec_frac, rho_d = refl (1 - spec_frac)
    let p = 4.56e-6;
    let (aa, f) = (0.1*0.34, p*0.1*0.34*(1.0 + rho_s + 2.0*rho_d/3.0));
    let t = torques::radiation(&box_facets(), &[-1.0, 0.0, 0.0], p);
    eprintln!("srp torque {:?} want {:.6e} (A {aa})", t, 0.01*f);
    assert!((t[2] - 0.01*f).abs() < 1e-12*1.0 && t[0].abs() < 1e-12 && t[1].abs() < 1e-12, "{t:?} vs {}", 0.01*f);
}

/// TS1(g). Drag on the same plate at normal incidence: F = 1/2 rho v^2 Cd A. The coefficient follows from the
/// momentum accommodation of the gas-surface model: normal incidence gives Cd = 2 (2 - sigma_n + sigma_n v_b/v)
/// (specular, sigma = 0: Cd = 4; fully diffuse with cold re-emission, sigma = 1: Cd = 2 (1 + v_b/v)); Moe and Moe,
/// Planetary and Space Science 53 (2005) 793, on the gas-surface momentum accommodation, and Schaaf and Chambre (1961).
#[test]
fn flat_plate_drag_torque() {
    let (sn, vb) = (0.8, 0.05);
    let cd = 2.0*(2.0 - sn + sn*vb);
    let (rho, v) = (3.0e-12, 7600.0);
    let f = 0.5*rho*v*v*cd*0.1*0.34;
    let i = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    let t = torques::torques(&[0.0, 0.0, 0.0, 1.0], &[7.0e6, 0.0, 0.0], &[-v, 0.0, 0.0], &[0.0; 3], &[1.0, 0.0, 0.0], 0.0, 0.0, rho, &i, &box_facets(), &[0.0; 3], MU, [false, true, false, false])[1];
    eprintln!("drag torque {:?} want {:.6e}", t, 0.01*f);
    assert!((t[2] - 0.01*f).abs() < 1e-12 && t[0].abs() < 1e-12 && t[1].abs() < 1e-12, "{t:?} vs {}", 0.01*f);
}
