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

// ---------------- device fidelity (B3.5) ----------------

#[test]
fn a_coil_lags_its_command_by_its_time_constant_and_saturates() {
    use adcs_sim_core::actuators::{Mtq, MtqDesc};
    let mut axes = [[0.0; 3]; adcs_sim_core::NS];
    axes[0] = [1.0, 0.0, 0.0]; axes[1] = [0.0, 1.0, 0.0]; axes[2] = [0.0, 0.0, 1.0];
    let tau = 0.01;
    let d = MtqDesc { fitted: true, n: 3, axes, m_max: 0.4, p_max: 0.3, scale_sigma: 0.0, misalign: 0.0, tau };
    let mut c = Mtq::new(d, &mut Rng::new(1, 1));
    // one time constant: 1 - 1/e = 63.2 % of the step at its end, 1/e of it averaged over it
    let (avg, end, _) = c.apply(&[0.2, 0.0, 0.0], tau);
    assert!(close(end[0], 0.2*(1.0 - (-1.0f64).exp()), 1e-14) && (end[0]/0.2 - 0.632).abs() < 1e-3, "{}", end[0]);
    assert!(close(avg[0], 0.2*(-1.0f64).exp(), 1e-14), "{}", avg[0]);
    // the next step starts from where it got to
    let (_, end2, _) = c.apply(&[0.2, 0.0, 0.0], tau);
    assert!(close(end2[0], 0.2*(1.0 - (-2.0f64).exp()), 1e-14));
    // a command past the limit saturates at m_max and the dipole settles there
    for _ in 0..50 { c.apply(&[0.0, 0.0, 5.0], tau); }
    let (avg, end, p) = c.apply(&[0.0, 0.0, 5.0], tau);
    assert!(close(end[2], 0.4, 1e-12) && close(avg[2], 0.4, 1e-12) && close(p, 0.3, 1e-9), "{end:?} {p}");
    // a failed coil is open: no current at once
    c.dead[2] = true;
    let (avg, end, _) = c.apply(&[0.0, 0.0, 5.0], tau);
    assert!(avg[2] == 0.0 && end[2] == 0.0);
}

fn one_wheel() -> adcs_sim_core::actuators::MexDesc {
    use adcs_sim_core::actuators::{Kind, MexDesc};
    let mut m = MexDesc { n: 1, torque_noise: 0.0, friction_comp: 0.95, eta: 0.8, ..Default::default() };
    m.kind[0] = Kind::Rw; m.a0[0] = [1.0, 0.0, 0.0]; m.h_max[0] = 0.01; m.torque_max[0] = 1e-3; m.jrot[0] = 1.6e-5;
    m.flo[0] = 1.0; m.fhi[0] = 1.0; m.coulomb[0] = 1e-5; m.viscous[0] = 1e-8;
    // SYN-RW-10's motor: k_t 4 mN m/A, 12 ohm, 5 V -> stall 1.667 mN m, no-load 1250 rad/s
    let (kt, r, v) = (4e-3, 12.0, 5.0);
    m.t_stall[0] = kt*v/r; m.w_nl[0] = v/kt; m.speed_max[0] = 600.0; m.f_static[0] = 1.5e-5; m.w_stribeck[0] = 1.0;
    m
}

#[test]
fn a_wheels_torque_follows_its_motor_line_and_stops_at_its_speed_limit() {
    use adcs_sim_core::actuators::Mex;
    let mut m = one_wheel();
    m.f_static[0] = 0.0; m.coulomb[0] = 0.0; m.viscous[0] = 0.0;     // no friction: the motor alone
    let ts = 4e-3*5.0/12.0;
    let mut x = Mex::new(m, &mut Rng::new(1, 1), Rng::new(1, 2));
    let hd = |x: &mut Mex, cmd: f64, w: f64| { let mut c = [0.0; NR]; c[0] = cmd; let mut h = [0.0; NR]; h[0] = w*1.6e-5; x.apply(&c, &[0.0; NG], &h, 0.1).0[0] };
    // below the knee the drive's current limit: the full torque
    assert!(close(hd(&mut x, 1e-3, 300.0), 1e-3, 1e-12));
    // on the line: T_s (1 - w/w_nl), speeding up
    for w in [520.0f64, 580.0, -550.0] {
        let want = ts*(1.0 - w.abs()/1250.0)*w.signum();
        assert!(close(hd(&mut x, 1e-3*w.signum(), w), want, 1e-12), "w {w}: {} vs {want}", hd(&mut x, 1e-3*w.signum(), w));
    }
    // braking, the back-EMF helps: the current limit again
    assert!(close(hd(&mut x, -1e-3, 580.0), -1e-3, 1e-12));
    // at the speed limit no torque that speeds it up, all that slows it
    assert!(hd(&mut x, 1e-3, 600.0) == 0.0 && close(hd(&mut x, -1e-3, 600.0), -1e-3, 1e-12));
    assert!(adcs_sim_core::actuators::wheel_motor(&m, 0, 1e-3, 2000.0) == 0.0);
}

#[test]
fn a_wheel_at_rest_breaks_away_only_above_its_static_friction() {
    use adcs_sim_core::actuators::Mex;
    let m = one_wheel();
    let mut x = Mex::new(m, &mut Rng::new(1, 1), Rng::new(1, 2));
    let hd = |x: &mut Mex, cmd: f64, h0: f64| { let mut c = [0.0; NR]; c[0] = cmd; let mut h = [0.0; NR]; h[0] = h0; x.apply(&c, &[0.0; NG], &h, 0.1).0[0] };
    // at rest, a demand under the breakaway torque does not move it
    assert!(hd(&mut x, 1.4e-5, 0.0) == 0.0 && hd(&mut x, -1.4e-5, 0.0) == 0.0);
    // over it, it turns
    assert!(close(hd(&mut x, 1.6e-5, 0.0), 1.6e-5, 1e-12));
    // creeping with no demand, static friction stops it within the step (h + hdot dt = 0)
    let h0 = 1e-7;
    assert!(close(hd(&mut x, 0.0, h0)*0.1 + h0, 0.0, 1e-15));
    // turning at the Stribeck speed: the driver leaves 5 % of Coulomb + viscous, and the
    // breakaway excess (Fs - Fc) e^-(w/ws)^2 is not compensated
    let w = 1.0;
    let fr = 1e-5 + 1e-8*w;
    let want = 1e-4 - (1.0 - 0.95)*fr - (1.5e-5 - 1e-5)*(-1.0f64).exp();
    assert!(close(hd(&mut x, 1e-4, w*1.6e-5), want, 1e-12), "{} vs {want}", hd(&mut x, 1e-4, w*1.6e-5));
}

fn tracker(blind: f64) -> adcs_sim_core::sensors::St {
    use adcs_sim_core::sensors::{St, StDesc};
    let mut d = StDesc { fitted: true, nh: 1, noise_cross: 1e-4, noise_roll: 1e-3, rate_hz: 5.0, latency: 0.0, max_rate: 1.0,
        sun_excl: 0.5, earth_excl: 0.3, fov: 0.17, model: 0, moon_excl: 0.26, blind_s: blind, noise_rate_ref: 0.01, ..Default::default() };
    d.bs[0] = [0.0, 0.0, 1.0];
    St::new(d, &mut Rng::new(1, 1), Rng::new(1, 3))
}

#[test]
fn a_star_tracker_refuses_inside_the_moon_exclusion_and_recovers_after_its_blind_time() {
    let mut s = tracker(5.0);
    let q = [0.0, 0.0, 0.0, 1.0];
    let (sun, nadir) = ([1.0, 0.0, 0.0], [0.0, 0.0, -1.0]);
    let moon_at = |deg: f64| { let a = deg.to_radians(); [a.sin(), 0.0, a.cos()] };
    assert!(s.sample(&q, 0.0, &[0.0; 3], &sun, &moon_at(40.0), &nadir, 1.0)[0].is_some());
    // inside the Moon's 15 deg cone: no attitude
    assert!(s.sample(&q, 1.0, &[0.0; 3], &sun, &moon_at(14.0), &nadir, 1.0)[0].is_none());
    // the Moon has left the cone, the head is still blind for 5 s after it was last in it
    assert!(s.sample(&q, 1.2, &[0.0; 3], &sun, &moon_at(16.0), &nadir, 1.0)[0].is_none());
    assert!(s.sample(&q, 5.9, &[0.0; 3], &sun, &moon_at(20.0), &nadir, 1.0)[0].is_none());
    assert!(s.sample(&q, 6.0, &[0.0; 3], &sun, &moon_at(20.0), &nadir, 1.0)[0].is_some());
    // the Sun blinds it the same way (its own cone), and with no blind time it is back at once
    let mut s = tracker(0.0);
    assert!(s.sample(&q, 0.0, &[0.0; 3], &moon_at(25.0), &moon_at(90.0), &nadir, 1.0)[0].is_none());
    assert!(s.sample(&q, 0.2, &[0.0; 3], &moon_at(35.0), &moon_at(90.0), &nadir, 1.0)[0].is_some());
}

#[test]
fn a_star_trackers_noise_grows_with_the_body_rate() {
    // cross-boresight error rms = noise_cross (1 + |w|/w_ref): 1x at rest, 3x at 2 w_ref
    let q = [0.0, 0.0, 0.0, 1.0];
    for (w, k) in [(0.0, 1.0), (0.02, 3.0)] {
        let mut s = tracker(0.0);
        let (mut ss, n) = (0.0, 4000);
        for i in 0..n {
            let z = s.sample(&q, i as f64*0.2, &[0.0, 0.0, w], &[1.0, 0.0, 0.0], &[-1.0, 0.0, 0.0], &[0.0, 0.0, -1.0], 1.0)[0].unwrap();
            let e = [2.0*z[0], 2.0*z[1]];          // small rotation: twice the vector part
            ss += e[0]*e[0] + e[1]*e[1];
        }
        let rms = (ss/(2.0*n as f64)).sqrt();
        assert!((rms/(1e-4*k) - 1.0).abs() < 0.05, "w {w}: rms {rms} vs {}", 1e-4*k);
    }
}

#[test]
fn a_gnss_fix_is_the_position_of_its_latency_ago() {
    use adcs_sim_core::sensors::{Gps, GpsDesc};
    let mut g = Gps::new(GpsDesc { fitted: true, pos_sigma: 0.0, vel_sigma: 0.0, rate_hz: 1.0, latency: 0.25 }, Rng::new(1, 4));
    let r = |t: f64| [7e6 + 100.0*t, -3.0*t, 2.0*t];
    let v = |t: f64| [100.0, -3.0, 2.0 + t];
    for k in 0..=30 { let t = k as f64*0.1; g.history(t, &r(t), &v(t)); }
    let (te, rd, vd) = g.delayed(3.0);
    assert!(close(te, 2.75, 1e-12));
    for i in 0..3 { assert!(close(rd[i], r(2.75)[i], 1e-12) && close(vd[i], v(2.75)[i], 1e-12), "{rd:?} {vd:?}"); }
    // the fix is what it was 0.25 s ago, not now: 25 m behind along x at 100 m/s
    assert!(close(r(3.0)[0] - rd[0], 25.0, 1e-9));
    // before the history reaches back that far: the first state and its epoch
    let mut g2 = Gps::new(GpsDesc { latency: 0.25, ..Default::default() }, Rng::new(1, 4));
    g2.history(0.0, &r(0.0), &v(0.0)); g2.history(0.1, &r(0.1), &v(0.1));
    let (te, rd, _) = g2.delayed(0.1);
    assert!(te == 0.0 && rd == r(0.0));
}

#[test]
fn the_earths_albedo_and_infrared_press_as_their_closed_form() {
    use adcs_sim_core::torques::{earth_pressure, EARTH_ALBEDO, EARTH_IR_W_M2};
    let rn = 6378137.0 + 500e3;
    let vf = (6378137.0/rn)*(6378137.0/rn);
    // the Sun overhead: the full albedo; the Sun behind the Earth: none, the infrared stays
    let (pa, pi) = earth_pressure(&[rn, 0.0, 0.0], &[1.5e11, 0.0, 0.0], 4.56e-6);
    assert!(close(pa, EARTH_ALBEDO*4.56e-6*vf, 1e-12) && close(pi, EARTH_IR_W_M2/299792458.0*vf, 1e-12));
    let (pa, _) = earth_pressure(&[rn, 0.0, 0.0], &[-1.5e11, 0.0, 0.0], 4.56e-6);
    assert!(pa == 0.0);
    // in eclipse (nu 0) only the Earth's infrared acts: on the -X facet of a box whose centre of
    // mass is 1 cm off along +Y, F = p A (1 + rho_s + 2 rho_d/3) along +X, torque = 0.01 F about Z
    let g = Facets::boxed(&[0.1, 0.1, 0.34], &[0.0, 0.01, 0.0], 0.8, 0.8, 0.05, 0.6, 0.5);
    let t = torques::torques(&[0.0, 0.0, 0.0, 1.0], &[rn, 0.0, 0.0], &[1.0, 0.0, 0.0], &[0.0; 3], &[-1.5e11, 0.0, 0.0], 0.0, 4.56e-6, 0.0,
        &[[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]], &g, &[0.0; 3], MU, [false, false, true, false]);
    let f = pi*0.1*0.34*(1.0 + 0.3 + 2.0*0.3/3.0);
    assert!(close(t[2][2], 0.01*f, 1e-12) && t[2][0].abs() < 1e-25 && t[2][1].abs() < 1e-25, "{:?} vs {}", t[2], 0.01*f);
}
