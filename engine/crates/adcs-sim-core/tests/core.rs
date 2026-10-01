//! The plant, environment and device models checked against what physics and their own
//! definitions say, not against a stored run: conservation laws, closed forms, round trips.
use adcs_sim_core::emu::{self, Commands};
use adcs_sim_core::la::*;
use adcs_sim_core::orbit::{coe2rv, Orbit, OrbitCfg};
use adcs_sim_core::plant::{self, Body, Geometry, State};
use adcs_sim_core::rng::Rng;
use adcs_sim_core::time;
use adcs_sim_core::torques::{self, Facets};
use adcs_sim_core::{NG, NR};

const MU: f64 = 3.986004418e14;

fn close(a: f64, b: f64, tol: f64) -> bool { (a - b).abs() <= tol*(1.0 + a.abs().max(b.abs())) }

#[test]
fn quaternions_and_dcms_round_trip() {
    let mut r = Rng::new(7, 1);
    for _ in 0..200 {
        let q = qnorm(&[r.normal(), r.normal(), r.normal(), r.normal()]);
        let back = fromdcm(&dcm(&q));
        assert!(qangle(&q, &back) < 1e-7, "fromdcm(dcm(q)) is q (up to sign; qangle resolves ~1e-8 rad near 0)");
        let a = dcm(&q);
        let i = mm(&a, &transpose(&a));
        for (j, row) in i.iter().enumerate() { for (k, x) in row.iter().enumerate() { assert!(close(*x, if j == k { 1.0 } else { 0.0 }, 1e-12)); } }
        assert!(close(det(&a), 1.0, 1e-12), "a rotation, not a reflection");
        let th = scale(&r.normal3(), 0.3);
        assert!(close(qangle(&[0.0, 0.0, 0.0, 1.0], &fromrotvec(&th)), norm(&th), 1e-7), "a rotation vector's angle is its length");
    }
    let m = [[2.0, 1.0, 0.0], [1.0, 3.0, 1.0], [0.0, 1.0, 4.0]];
    let p = mm(&m, &inv(&m));
    for (j, row) in p.iter().enumerate() { for (k, x) in row.iter().enumerate() { assert!(close(*x, if j == k { 1.0 } else { 0.0 }, 1e-14)); } }
}

#[test]
fn time_has_its_epochs() {
    assert!(close(time::jd(&[2000.0, 1.0, 1.0, 12.0, 0.0, 0.0]), 2451545.0, 1e-15), "J2000.0");
    let u = time::jd2utc(2461407.25);
    assert!(close(time::jd(&u), 2461407.25, 1e-14), "jd2utc inverts jd");
    let c = time::eci2ecef(2451545.0);
    assert!(close(det(&c), 1.0, 1e-12), "ECI to ECEF is a rotation");
}

#[test]
fn the_noise_is_seeded_and_standard() {
    let (mut a, mut b) = (Rng::new(42, 3), Rng::new(42, 3));
    for _ in 0..100 { assert_eq!(a.next_u64(), b.next_u64(), "the same seed and stream give the same draws"); }
    let mut c = Rng::new(42, 4);
    assert_ne!(Rng::new(42, 3).next_u64(), c.next_u64(), "another stream gives other draws");
    let n = 200_000;
    let (mut s, mut s2) = (0.0, 0.0);
    for _ in 0..n { let x = c.normal(); s += x; s2 += x*x; }
    let (mean, var) = (s/n as f64, s2/n as f64 - (s/n as f64).powi(2));
    assert!(mean.abs() < 0.01 && (var - 1.0).abs() < 0.02, "standard normal: mean {mean}, variance {var}");
}

fn body(i: V3, wheels: usize) -> Body {
    let s = 1.0/3f64.sqrt();
    let ax = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [s, s, s]];
    let a0: Vec<V3> = ax[..wheels].to_vec();
    let gi = vec![0usize; wheels];
    let inertia = diag(&i);
    Body::rigid(inertia, Geometry::new(&a0, &[], &gi))
}

#[test]
fn a_torque_free_body_keeps_its_momentum_and_energy() {
    let b = body([0.02, 0.03, 0.05], 0);
    let mut x = State { q: [0.0, 0.0, 0.0, 1.0], w: [0.1, 0.02, -0.05], ..Default::default() };
    let h_eci = |x: &State| mtv(&dcm(&x.q), &plant::momentum(x, &b));
    let e = |x: &State| 0.5*dot(&x.w, &mv(&b.i, &x.w));
    let (h0, e0) = (h_eci(&x), e(&x));
    for _ in 0..6000 { x = plant::step(&x, 0.1, &b, &[0.0; 3], &[0.0; NR], &[0.0; NG]); }
    let (h1, e1) = (h_eci(&x), e(&x));
    for k in 0..3 { assert!(close(h1[k], h0[k], 1e-7), "inertial momentum {k}: {} -> {}", h0[k], h1[k]); }
    assert!(close(e1, e0, 1e-7), "kinetic energy {e0} -> {e1}");
    assert!(close(norm(&[x.q[0], x.q[1], x.q[2]]).hypot(x.q[3]), 1.0, 1e-12), "the attitude stays a unit quaternion");
}

#[test]
fn a_wheel_trades_momentum_with_the_body() {
    let b = body([0.02, 0.03, 0.05], 3);
    let mut x = State { q: [0.0, 0.0, 0.0, 1.0], ..Default::default() };
    let mut tau = [0.0; NR];
    tau[2] = 1e-4;                              // the z wheel spun up at 0.1 mN m
    for _ in 0..100 { x = plant::step(&x, 0.1, &b, &[0.0; 3], &tau, &[0.0; NG]); }
    assert!(close(x.h[2], 1e-3, 1e-9), "the wheel gained tau t = 1 mN m s: {}", x.h[2]);
    let h = plant::momentum(&x, &b);
    assert!(h.iter().all(|v| v.abs() < 1e-12), "the total stays zero: {h:?}");
    assert!(close(x.w[2], -1e-3/0.05, 1e-6), "the body turned the other way: {}", x.w[2]);
}

#[test]
fn the_gravity_gradient_is_its_closed_form() {
    let i = diag(&[0.0067, 0.042, 0.050]);
    let g = Facets::boxed(&[0.34, 0.1, 0.1], &[0.0; 3], 0.8, 0.8, 0.05, 0.6, 0.5);
    let rr = 6.9e6;
    let u = [1.0/2f64.sqrt(), 0.0, 1.0/2f64.sqrt()];
    let t = torques::torques(&[0.0, 0.0, 0.0, 1.0], &scale(&u, rr), &[0.0; 3], &[0.0; 3], &[1.0, 0.0, 0.0], 1.0, 0.0, 0.0, &i, &g, &[0.0; 3], MU, [true, false, false, false]);
    // 3 mu / r^3 (u x J u): about y, 0.5 (Jxx - Jzz) with u at 45 deg in x-z
    let want = 3.0*MU/(rr*rr*rr)*0.5*(0.0067 - 0.050);
    assert!(close(t[0][1], want, 1e-12) && t[0][0].abs() < 1e-20 && t[0][2].abs() < 1e-20, "{:?} vs {want}", t[0]);
    let along = torques::torques(&[0.0, 0.0, 0.0, 1.0], &[0.0, 0.0, rr], &[0.0; 3], &[0.0; 3], &[1.0, 0.0, 0.0], 1.0, 0.0, 0.0, &i, &g, &[0.0; 3], MU, [true, false, false, false]);
    assert!(along[0].iter().all(|v| v.abs() < 1e-20), "a principal axis along the radius feels none");
}

#[test]
fn a_circular_orbit_keeps_its_radius_and_period() {
    let a = 6378137.0 + 550e3;
    let (r0, v0) = coe2rv(a, 0.0, 97.6f64.to_radians(), 0.3, 0.0, 0.0);
    let cfg = OrbitCfg { jd0_utc: 2461407.25, step_s: 10.0, zonal_max: 0, third_body: false, drag: false, srp: false, mass_kg: 4.0, area_m2: 0.03, cd: 2.2, cr: 1.3, density_scale: 1.0 };
    let mut o = Orbit::new(cfg, r0, v0);
    let period = 2.0*std::f64::consts::PI*(a*a*a/MU).sqrt();
    let mut t = 0.0;
    while t < period { let (r, _) = o.state(t); assert!(close(norm(&r), a, 1e-7), "radius at t = {t}: {}", norm(&r)); t += 60.0; }
    let (r1, _) = o.state(period);
    assert!(norm(&sub(&r1, &r0)) < 5.0, "back where it started after one period ({} m away)", norm(&sub(&r1, &r0)));
}

#[test]
fn the_bus_codecs_round_trip() {
    let b = [2.1e-5, -3.4e-5, 4.0e-6];
    let regs = emu::mag_regs(true, &b);
    assert_eq!(regs[0] & 1, 1, "the valid flag");
    for k in 0..3 {
        let raw = i16::from_le_bytes([regs[1 + 2*k], regs[2 + 2*k]]) as f64*emu::proto::MAG_LSB_T;
        assert!((raw - b[k]).abs() <= emu::proto::MAG_LSB_T, "axis {k}: {raw} vs {}", b[k]);
    }
    let mut c = Commands::default();
    emu::decode_pwm(&[16384, -32767, 0, 0, 0, 0, 0, 0], 0.2, &mut c);
    assert!(close(c.m_body[0], 0.1, 1e-4) && close(c.m_body[1], -0.2, 1e-4) && c.m_body[2] == 0.0, "{:?}", c.m_body);
    let mut tmax = [0.0; NR];
    tmax[1] = 2e-3;
    let mut d = [0u8; 8];
    d[..2].copy_from_slice(&(-16384i16).to_le_bytes());
    emu::decode_can(emu::proto::CAN_ROTOR_CMD + 1, &d, &tmax, 1.0, 0.1, &mut c);
    assert!(close(c.cmd_r[1], -1e-3, 1e-4), "{}", c.cmd_r[1]);
    let mut buf = [0u8; 64];
    let n = emu::gps_frame(true, &[7e6, 1.0, -2.0], &[1.0, 7.5e3, 0.5], &mut buf);
    assert!(n > 6 && buf[..2] == [emu::proto::ST_SYNC1, emu::proto::GPS_SYNC2], "a GNSS frame starts with its sync");
    let len = buf[2] as usize;
    let crc = u16::from_le_bytes([buf[3 + len], buf[4 + len]]);
    assert_eq!(crc, emu::crc16(&buf[3..3 + len]), "and carries the CRC of its payload");
}

fn flex_body(p: f64, f_hz: f64, zeta: f64) -> Body {
    let i = diag(&[0.02, 0.03, 0.05]);
    let delta = [0.0, 0.0, (p*0.05f64).sqrt()];
    Body::flexible(i, Geometry::new(&[], &[], &[]), plant::Flex { on: true, delta, omega: 2.0*std::f64::consts::PI*f_hz, zeta })
}

#[test]
fn a_flexible_body_keeps_its_momentum_and_energy_and_rings_at_its_free_free_frequency() {
    let (p, f) = (0.3, 0.5);
    let b = flex_body(p, f, 0.0);
    let mut x = State { q: [0.0, 0.0, 0.0, 1.0], w: [0.01, -0.02, 0.0], eta: 0.01, ..Default::default() };
    let d = b.flex.delta;
    let energy = |x: &State| 0.5*dot(&x.w, &mv(&b.i, &x.w)) + dot(&x.w, &d)*x.etad + 0.5*x.etad*x.etad + 0.5*b.flex.omega.powi(2)*x.eta*x.eta;
    let h_eci = |x: &State| mtv(&dcm(&x.q), &plant::momentum(x, &b));
    let (h0, e0) = (h_eci(&x), energy(&x));
    let dt = 0.01;
    let mut crossings = vec![];
    let mut last = x.eta;
    for k in 0..20000 {
        x = plant::step(&x, dt, &b, &[0.0; 3], &[0.0; NR], &[0.0; NG]);
        if last > 0.0 && x.eta <= 0.0 { crossings.push(k as f64*dt); }
        last = x.eta;
    }
    let h1 = h_eci(&x);
    for k in 0..3 { assert!(close(h1[k], h0[k], 1e-8), "inertial momentum {k}: {} -> {}", h0[k], h1[k]); }
    assert!(close(energy(&x), e0, 1e-8), "energy {e0} -> {}", energy(&x));
    // about z the mode and the body share the motion: the free-free frequency is Omega / sqrt(1 - p)
    let period = (crossings[crossings.len() - 1] - crossings[0])/(crossings.len() - 1) as f64;
    let want = 1.0/(f/(1.0 - p).sqrt());
    assert!((period - want).abs() < 0.01*want, "period {period} s vs {want} s");
}

#[test]
fn a_damped_mode_dies_away_and_a_rigid_body_is_untouched() {
    let b = flex_body(0.3, 0.5, 0.05);
    let mut x = State { q: [0.0, 0.0, 0.0, 1.0], eta: 0.01, ..Default::default() };
    for _ in 0..6000 { x = plant::step(&x, 0.1, &b, &[0.0; 3], &[0.0; NR], &[0.0; NG]); }
    assert!(x.eta.abs() < 1e-6 && x.etad.abs() < 1e-6, "eta {} after 600 s", x.eta);
    // a rigid body's step is the step it always was
    let r = body([0.02, 0.03, 0.05], 0);
    let x0 = State { q: [0.0, 0.0, 0.0, 1.0], w: [0.1, 0.02, -0.05], ..Default::default() };
    let y = plant::step(&x0, 0.1, &r, &[1e-6, 0.0, 0.0], &[0.0; NR], &[0.0; NG]);
    assert!(y.eta == 0.0 && y.etad == 0.0);
}

#[test]
fn a_flexible_step_is_the_twins() {
    // the same step in matlab_sils (asils.plant.step with M.flex), printed to 17 digits
    let i = [[0.02, 1e-3, 0.0], [1e-3, 0.03, 0.0], [0.0, 0.0, 0.05]];
    let delta = [0.01, 0.0, (0.2f64*0.05).sqrt()];
    let geo = Geometry::new(&[[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]], &[], &[0, 0, 0]);
    let b = Body::flexible(i, geo, plant::Flex { on: true, delta, omega: 2.0*std::f64::consts::PI*1.5, zeta: 0.01 });
    let q = qnorm(&[0.1, -0.2, 0.3, 0.9]);
    let mut h = [0.0; NR]; h[0] = 1e-3; h[1] = -2e-3; h[2] = 5e-4;
    let mut tr = [0.0; NR]; tr[0] = 1e-5; tr[2] = -1e-5;
    let x = State { q, w: [0.01, -0.02, 0.03], h, eta: 0.004, etad: -0.01, ..Default::default() };
    let y = plant::step(&x, 0.1, &b, &[1e-6, -2e-6, 3e-7], &tr, &[0.0; NG]);
    let twin = [0.10309608651184834, -0.20618708796330151, 0.31095700362999107, 0.92204328726834361, 0.024792743262954071, -0.020675163732826785,
                0.092338264706772713, 0.0010010000000000002, -0.002, 0.00049900000000000009, 0.0011694761227773595, -0.041131871922693644];
    let got = [y.q[0], y.q[1], y.q[2], y.q[3], y.w[0], y.w[1], y.w[2], y.h[0], y.h[1], y.h[2], y.eta, y.etad];
    for k in 0..12 { assert!((got[k] - twin[k]).abs() < 1e-14*(1.0 + twin[k].abs()), "state {k}: {} vs the twin's {}", got[k], twin[k]); }
}
