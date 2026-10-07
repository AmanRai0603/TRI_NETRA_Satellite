//! The flight algorithms against their pseudocode (fsw/pseudocode/*.pc, docs/PSEUDOCODE_V2.md):
//! every vector the interpreter drew (fsw/tests/pcode_vectors.txt, written by tools/pcode.py gen)
//! through the runtime's Rust signatures (which hand the values to the algorithms written from the design,
//! fsw-rs/src/alg), and the functions it has no signature for by name through the generated dispatcher. A function that uses no transcendental must agree
//! bit for bit; one that does, to 1e-12 relative (maths libraries differ in their last bits).
//! A proc's vectors are a run of calls, its state carried from each to the next.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use adcs_fsw::devices::*;
use adcs_fsw::{alloc, ctl, drv, env, est, guid, CanFrame, Hal, Params, Status};
use adcs_fsw::math as hmath;
// The translator's vector dispatcher (tests/alg/dispatch.rs, test code written beside the algorithms by
// tools/flight_build.py, never in the flight crate). It names the algorithms' modules crate::<module>, so they are
// brought in at this test's root.
#[allow(unused_imports)]
use adcs_fsw::alg::{allocation, control, drivers, estimation, frames, guidance, math, modes, steplaws};
#[allow(dead_code)]
#[path = "alg/dispatch.rs"]
mod dispatch;

/// One call of the vector file: `name exact set nx ny`, then the values' IEEE-754 bits in hex.
struct Vector { name: &'static str, exact: bool, x: Vec<f64>, y: Vec<f64> }

fn vectors() -> Vec<Vector> {
    include_str!("../../fsw/tests/pcode_vectors.txt").lines().filter(|l| !l.starts_with('#') && !l.is_empty()).map(|l| {
        let w: Vec<&'static str> = l.split(' ').collect();
        let (nx, ny): (usize, usize) = (w[3].parse().unwrap(), w[4].parse().unwrap());
        let v: Vec<f64> = w[5..5 + nx + ny].iter().map(|h| f64::from_bits(u64::from_str_radix(h, 16).unwrap())).collect();
        Vector { name: w[0], exact: w[1] == "1", x: v[..nx].to_vec(), y: v[nx..].to_vec() }
    }).collect()
}

/// Functions of the pseudocode the Rust flight software does not check, and why: none. Every function the pseudocode
/// declares is in the flight build (fsw-rs/src/alg, written by tools/flight_build.py), so every one is checked.
const NOT_IN_RUST: &[(&str, &str)] = &[];

/// Functions the runtime has no signature of its own for, and why: each is checked as the translator wrote it, called
/// by its name through the translator's vector dispatcher.
const BY_NAME: &[(&str, &str)] = &[
    ("math::qconj", "the runtime writes it inline where it is used ([-q.x, -q.y, -q.z, q.w])"),
    ("modes::modes_enter", "the runtime calls it from a method on its private state (fsw::modes, fsw::fdir): no public signature"),
    ("modes::modes_feasible", "the runtime calls it from a method on its private state (fsw::modes, fsw::fdir): no public signature"),
    ("modes::spin_guards", "called only by the generated modes_step"),
    ("modes::modes_schedule", "the runtime calls it from a method on its private state (fsw::modes, fsw::fdir): no public signature"),
    ("modes::modes_step", "the runtime calls it from a method on its private state (fsw::modes, fsw::fdir): no public signature"),
    ("modes::fdir_safe", "the runtime calls it from a method on its private state (fsw::modes, fsw::fdir): no public signature"),
    ("modes::fdir_sensors", "the runtime calls it from a method on its private state (fsw::modes, fsw::fdir): no public signature"),
    ("modes::fdir_rotors", "the runtime calls it from a method on its private state (fsw::modes, fsw::fdir): no public signature"),
    ("steplaws::ctl_mtq", "the runtime calls it from a method on its private state (fsw.rs): no public signature"),
    ("steplaws::ctl_capture", "the runtime calls it from a method on its private state (fsw.rs): no public signature"),
    ("steplaws::ctl_sun_acq", "the runtime calls it from a method on its private state (fsw.rs): no public signature"),
    ("steplaws::alloc_rotors", "the runtime calls it from a method on its private state (fsw.rs): no public signature"),
    ("steplaws::alloc_idle", "the runtime calls it from a method on its private state (fsw.rs): no public signature"),
    ("steplaws::orbit_acc", "the runtime calls it from a method on its private state (fsw.rs): no public signature"),
    ("frames::mod360", "called only by the generated sun_model"),
    ("control::mtq_err", "called only by the generated magnetic laws (Lovera, Celani, Avanzini, TANGO)"),
    ("estimation::update3", "called only by the generated mekf_vector and mekf_quat"),
    ("drivers::rd16", "called only by the generated drv_read"),
    ("drivers::rd32", "called only by the generated drv_read"),
    ("drivers::q15", "called only by the generated drv_write"),
    ("drivers::uart_frame", "called only by the generated drv_read, on both UARTs"),
    ("drivers::uart_stream", "the vectors' byte-stream builder: no flight function calls it"),
    ("drivers::read_streams", "the vectors' byte-stream builder: no flight function calls it"),
];

/// A call by the runtime's signature, or by name through the dispatcher for the functions it has none for.
fn call_any(name: &str, x: &[f64]) -> Option<Vec<f64>> {
    if BY_NAME.iter().any(|(m, _)| *m == name) { dispatch::call(name, x) } else { call(name, x) }
}


/// An in-memory HAL: what the devices would put on their ports, and what the drivers wrote.
#[derive(Default)]
struct Stub { mag: [u8; 7], gyro: [u8; 13], sun: [u8; 7], es: [u8; 7], uart: [Vec<u8>; 3], can_rx: Vec<CanFrame>, can_tx: Vec<CanFrame>, pwm: [i16; 8] }
impl Hal for Stub {
    fn time_ns(&mut self) -> u64 { 0 }
    fn uart_read(&mut self, port: u8, buf: &mut [u8]) -> Result<usize, Status> {
        let q = &mut self.uart[port as usize];
        let n = q.len().min(buf.len());
        if n == 0 { return Err(Status::Timeout); }
        buf[..n].copy_from_slice(&q[..n]);
        q.drain(..n);
        Ok(n)
    }
    fn uart_write(&mut self, _port: u8, _buf: &[u8]) -> Status { Status::Ok }
    fn i2c_xfer(&mut self, bus: u8, addr7: u8, _tx: &[u8], rx: &mut [u8]) -> Status {
        let src = if bus == MAG_BUS && addr7 == MAG_ADDR { &self.mag } else if bus == SUN_BUS && addr7 == SUN_ADDR { &self.sun }
                  else if bus == ES_BUS && addr7 == ES_ADDR { &self.es } else { return Status::NoDev };
        let n = rx.len().min(7);
        rx[..n].copy_from_slice(&src[..n]);
        Status::Ok
    }
    fn spi_xfer(&mut self, bus: u8, cs: u8, _tx: &[u8], rx: &mut [u8]) -> Status {
        if bus != GYRO_BUS || cs != GYRO_CS { return Status::NoDev; }
        let n = rx.len().min(13);
        rx[..n].copy_from_slice(&self.gyro[..n]);
        Status::Ok
    }
    fn can_send(&mut self, _port: u8, f: &CanFrame) -> Status { self.can_tx.push(*f); Status::Ok }
    fn can_recv(&mut self, _port: u8) -> Result<CanFrame, Status> { if self.can_rx.is_empty() { Err(Status::Timeout) } else { Ok(self.can_rx.remove(0)) } }
    fn pwm_set(&mut self, ch: u8, duty_q15: i16) -> Status { if ch < 8 { self.pwm[ch as usize] = duty_q15; } Status::Ok }
}

struct In<'a> { x: &'a [f64], at: usize }
impl In<'_> {
    fn f(&mut self) -> f64 { self.at += 1; self.x[self.at - 1] }
    fn i(&mut self) -> i64 { self.f() as i64 }
    fn b(&mut self) -> bool { self.f() != 0.0 }
    fn v3(&mut self) -> [f64; 3] { [self.f(), self.f(), self.f()] }
    fn q(&mut self) -> [f64; 4] { [self.f(), self.f(), self.f(), self.f()] }
    fn m3(&mut self) -> [[f64; 3]; 3] { [self.v3(), self.v3(), self.v3()] }
    fn u8s<const N: usize>(&mut self) -> [u8; N] { let mut r = [0u8; N]; for x in r.iter_mut() { *x = self.i() as u8; } r }
    fn fs<const N: usize>(&mut self) -> [f64; N] { let mut r = [0.0; N]; for x in r.iter_mut() { *x = self.f(); } r }
    fn m<const R: usize, const C: usize>(&mut self) -> [[f64; C]; R] {
        let mut m = [[0.0; C]; R];
        for r in m.iter_mut() { for c in r.iter_mut() { *c = self.f(); } }
        m
    }
}

#[derive(Default)]
struct Out(Vec<f64>);
impl Out {
    fn f(&mut self, v: f64) -> &mut Self { self.0.push(v); self }
    fn b(&mut self, v: bool) -> &mut Self { self.0.push(if v { 1.0 } else { 0.0 }); self }
    fn s(&mut self, v: &[f64]) -> &mut Self { self.0.extend_from_slice(v); self }
    fn m<const C: usize>(&mut self, m: &[[f64; C]]) -> &mut Self { for r in m { self.0.extend_from_slice(r); } self }
}

/// One call of a pseudocode function through the Rust flight software (None: not adapted).
fn call(name: &str, x: &[f64]) -> Option<Vec<f64>> {
    let mut a = In { x, at: 0 };
    let mut o = Out::default();
    match name {
        "math::maxabs3" => { o.f(hmath::maxabs3(&a.v3())); }
        "math::mat3t_vec" => { let m = a.m3(); o.s(&hmath::mat3t_vec(&m, &a.v3())); }
        "math::skew" => { o.m(&hmath::skew(&a.v3())); }
        "math::det3" => { o.f(hmath::det3(&a.m3())); }
        "math::inv3" => { let (r, ok) = hmath::inv3(&a.m3()); o.m(&r).b(ok); }
        "math::pinv_rows" => { let m = a.m::<3, 8>(); let n = a.i() as usize; o.m(&hmath::pinv_rows(&m, n)); }
        "math::jacobi_eig4" => { let (l, v) = hmath::jacobi_eig4(&a.m::<4, 4>()); o.s(&l).m(&v); }
        "math::qmult" => { let p = a.q(); o.s(&hmath::qmult(&p, &a.q())); }
        "math::qnorm" => { o.s(&hmath::qnorm(&a.q())); }
        "math::dcm" => { o.m(&hmath::dcm(&a.q())); }
        "math::fromdcm" => { o.s(&hmath::fromdcm(&a.m3())); }
        "math::fromrotvec" => { o.s(&hmath::fromrotvec(&a.v3())); }
        "math::qangle" => { let p = a.q(); o.f(hmath::qangle(&p, &a.q())); }
        "math::qerr" => { let p = a.q(); o.s(&hmath::qerr(&p, &a.q())); }
        "guidance::guid_kind" => { o.f(guid::kind_of(a.i() as u8) as f64); }
        "guidance::guidance" => {
            let kind = a.i() as i32; let (r, v, t) = (a.v3(), a.v3(), a.f());
            let g = guid::Guid { q_off: a.q(), roll_deg: a.f(), t0: a.f(), t_slew: a.f(), axis: a.v3(), q_inertial: a.q(),
                                 sun_axis: a.v3(), roll_axis: a.v3(), sun_eci: a.v3(), flip: a.b() };
            let rf = guid::guidance(kind, &r, &v, t, &g);
            o.s(&rf.q).s(&rf.w).s(&rf.wd);
        }
        "guidance::yaw_flip" => {
            let (r, v) = (a.v3(), a.v3());
            let mut g = guid::Guid { q_off: a.q(), sun_axis: a.v3(), roll_axis: a.v3(), sun_eci: a.v3(), flip: a.b(), ..Default::default() };
            let h = a.f();
            guid::yaw_flip(&mut g, &r, &v, h);
            o.b(g.flip);
        }
        "guidance::boresight_offset" => { o.s(&guid::boresight_offset(&a.v3())); }
        "estimation::mekf_init" => { let q = a.q(); let (sa, sb) = (a.f(), a.f()); let k = est::Mekf::new(&q, sa, sb, 0.0, 0.0); o.s(&k.q).s(&k.b).m(&k.p); }
        "estimation::mekf_predict" => {
            let (q, b, p, arw, rrw) = (a.q(), a.v3(), a.m::<6, 6>(), a.f(), a.f());
            let mut k = est::Mekf { q, b, p, arw, rrw };
            let (wm, dt) = (a.v3(), a.f());
            k.predict(&wm, dt);
            o.s(&k.q).m(&k.p);
        }
        "estimation::mekf_vector" => {
            let mut k = est::Mekf { q: a.q(), b: a.v3(), p: a.m::<6, 6>(), arw: 0.0, rrw: 0.0 };
            let (bm, rr, sg, gate) = (a.v3(), a.v3(), a.f(), a.f());
            let took = k.vector(&bm, &rr, sg, gate);
            o.s(&k.q).s(&k.b).m(&k.p).b(took);
        }
        "estimation::mekf_quat" => {
            let mut k = est::Mekf { q: a.q(), b: a.v3(), p: a.m::<6, 6>(), arw: 0.0, rrw: 0.0 };
            let (qm, sc, sr, bs, gate) = (a.q(), a.f(), a.f(), a.v3(), a.f());
            let took = k.quat(&qm, sc, sr, &bs, gate);
            o.s(&k.q).s(&k.b).m(&k.p).b(took);
        }
        "estimation::triad" => {
            let (b1, b2, r1, r2) = (a.v3(), a.v3(), a.v3(), a.v3());
            match est::triad(&b1, &b2, &r1, &r2) { Some(q) => { o.s(&q).b(true); } None => { o.s(&[0.0; 4]).b(false); } }
        }
        "estimation::quest" => {
            let (b, r) = (a.m::<16, 3>(), a.m::<16, 3>());
            let w: Vec<f64> = (0..16).map(|_| a.f()).collect();
            let n = a.i() as usize;
            let (q, loss) = est::quest(&b[..n], &r[..n], Some(&w[..n]));
            o.s(&q).f(loss);
        }
        "control::control_law" => {
            let (q, w, qr, wr) = (a.q(), a.v3(), a.q(), a.v3());
            let mut iq = a.v3();
            let dt = a.f();
            let g = ctl::Gains { law: a.i() as u8, kp: a.v3(), kd: a.v3(), ki: a.v3(), klqr: a.m3(), lambda: a.f(), phi: a.f(), gs: a.v3(), err_max: a.f(), int_max: a.f() };
            let (j, hs, wd) = (a.m3(), a.v3(), a.v3());
            let tau = ctl::control_law(&q, &w, &qr, &wr, &mut iq, dt, &g, &j, &hs, &wd);
            o.s(&tau).s(&iq);
        }
        "control::mtq_pd" => {
            let (q, w, qr, wr) = (a.q(), a.v3(), a.q(), a.v3());
            let g = ctl::Gains { kp: a.v3(), kd: a.v3(), ..Default::default() };
            o.s(&ctl::mtq_pd(&q, &w, &qr, &wr, &g));
        }
        "control::sat_dipole" => { let m = a.v3(); o.s(&ctl::sat_dipole(&m, a.f())); }
        "control::torque2dipole" => { let (t, b) = (a.v3(), a.v3()); o.s(&ctl::torque2dipole(&t, &b, a.f())); }
        "control::bdot" => { let (b1, b2) = (a.v3(), a.v3()); let (dt, bn, k, mm) = (a.f(), a.f(), a.f(), a.f()); o.s(&ctl::bdot(&b1, &b2, dt, bn, k, mm)); }
        "control::gen_bdot" => { let (b, bd, wd) = (a.v3(), a.v3(), a.v3()); o.s(&ctl::gen_bdot(&b, &bd, &wd, a.f())); }
        "control::sun_spin" => {
            let (b, w, sv, ecl, j) = (a.v3(), a.v3(), a.v3(), a.b(), a.m3());
            let (sp, k1, k2, fl) = (a.f(), a.f(), a.f(), a.f());
            o.s(&ctl::sun_spin(&b, &w, &sv, ecl, &j, sp, k1, k2, fl));
        }
        "control::mtq_lovera" => {
            let (q, w, qr, wr, j) = (a.q(), a.v3(), a.q(), a.v3(), a.m3());
            let (e, kp, kv) = (a.f(), a.f(), a.f());
            o.s(&ctl::mtq_lovera(&q, &w, &qr, &wr, &j, e, kp, kv));
        }
        "control::mtq_celani" => {
            let (q, w, qr, wr) = (a.q(), a.v3(), a.q(), a.v3());
            let (e, k1, k2) = (a.f(), a.f(), a.f());
            o.s(&ctl::mtq_celani(&q, &w, &qr, &wr, e, k1, k2));
        }
        "control::mtq_avanzini" => {
            let (q, w, qr, wr, j) = (a.q(), a.v3(), a.q(), a.v3(), a.m3());
            let (k, lam) = (a.f(), a.f());
            match ctl::mtq_avanzini(&q, &w, &qr, &wr, &j, k, lam) { Some(t) => { o.s(&t).b(true); } None => { o.s(&[0.0; 3]).b(false); } }
        }
        "control::mtq_boresight" => { let (e3, t, we) = (a.v3(), a.v3(), a.v3()); let (kp, kd) = (a.f(), a.f()); o.s(&ctl::mtq_boresight(&e3, &t, &we, kp, kd)); }
        "control::mtq_tango" => {
            let (q, w, qr, wr, pth, pw) = (a.q(), a.v3(), a.q(), a.v3(), a.m3(), a.m3());
            o.s(&ctl::mtq_tango(&q, &w, &qr, &wr, &pth, &pw));
        }
        "control::sun_spin_deruiter" => {
            let (b, w, sv, ecl, j) = (a.v3(), a.v3(), a.v3(), a.b(), a.m3());
            let (sp, k, k1, k2) = (a.f(), a.f(), a.f(), a.f());
            o.s(&ctl::sun_spin_deruiter(&b, &w, &sv, ecl, &j, sp, k, k1, k2));
        }
        "allocation::rotor_axes" => {
            let mut p = Params::default();
            p.nr = a.i() as u8; p.rot_a0 = a.m::<8, 3>(); p.rot_gi = a.u8s::<8>(); p.gim_axis = a.m::<4, 3>();
            let d = a.fs::<4>();
            o.m(&alloc::rotor_axes(&p, &d));
        }
        "allocation::steer_sr" => {
            let (tau, am, h) = (a.v3(), a.m::<3, 8>(), a.fs::<8>());
            let mut p = Params::default();
            p.nr = a.i() as u8; p.ng = a.i() as u8; p.rot_gi = a.u8s::<8>(); p.gim_axis = a.m::<4, 3>();
            p.gim_rate_max = a.f(); p.cmg_lam0 = a.f(); p.cmg_mu = a.f();
            let wheels = a.b();
            let (g, hd) = alloc::steer_sr(&tau, &am, &h, &p, wheels);
            o.s(&g).s(&hd);
        }
        "allocation::dump" => { let (hd, ht, b) = (a.v3(), a.v3(), a.v3()); let (k, mm) = (a.f(), a.f()); o.s(&alloc::dump(&hd, &ht, &b, k, mm)); }
        "allocation::rcs_duty" => {
            let req = a.v3();
            let mut p = Params::default();
            p.nc = a.i() as u8; p.rcs_tau = a.m::<6, 3>(); p.rcs_mib = a.f(); p.rcs_res = a.f();
            let (d, tau) = alloc::rcs_duty(&req, &p, a.f());
            o.s(&d).s(&tau);
        }
        "frames::gmst_rot" => { o.m(&env::gmst_rot(a.f())); }
        "frames::prec_rot" => { o.m(&env::prec_rot(a.f())); }
        "frames::eci2ecef" => { o.m(&env::eci2ecef(a.f())); }
        "frames::decyear" => { o.f(env::decyear(a.f())); }
        "frames::sun_model" => { o.s(&env::sun_model(a.f())); }
        "frames::geodetic" => { let (la, lo, h) = env::geodetic(&a.v3()); o.f(la).f(lo).f(h); }
        "frames::igrf_gh" => { o.s(&env::igrf_gh(a.f())); }
        "frames::igrf_ned" => {
            let gh = a.fs::<195>(); let (la, lo, alt) = (a.f(), a.f(), a.f()); let n = a.i() as i32;
            o.s(&env::igrf_ned(&gh, la, lo, alt, n));
        }
        "frames::field_eci" => {
            let (r, jd) = (a.v3(), a.f()); let gh = a.fs::<195>(); let n = a.i() as i32;
            o.s(&env::field_eci(&r, jd, &gh, n));
        }
        "estimation::latency" => { let (q, w, l) = (a.q(), a.v3(), a.f()); o.s(&est::latency(&q, &w, l)); }
        "drivers::crc16" => { let b = a.u8s::<64>(); let n = a.i() as usize; o.f(drv::crc16(&b[..n]) as f64); }
        "drivers::drv_write" => {
            // through the public path: the words the driver puts on the stub HAL's PWM and CAN
            let mut p = Params::default();
            let m = a.v3(); p.m_max = a.f(); let cr = a.fs::<8>(); p.rot_tmax = a.fs::<8>(); p.nr = a.i() as u8; let cg = a.fs::<4>();
            p.gim_rate_max = a.f(); p.ng = a.i() as u8; let du = a.fs::<6>(); p.nc = a.i() as u8; p.dt = a.f();
            let mut h = Stub::default();
            drv::write(&mut h, &p, &m, &cr, &cg, &du);
            let (mut rot, mut gim, mut val) = ([0.0; 8], [0.0; 4], [0.0; 6]);
            for f in &h.can_tx {
                let w = i16::from_le_bytes([f.data[0], f.data[1]]) as f64;
                if (CAN_ROTOR_CMD..CAN_ROTOR_CMD + 8).contains(&f.id) { rot[(f.id - CAN_ROTOR_CMD) as usize] = w; }
                else if (CAN_GIMBAL_CMD..CAN_GIMBAL_CMD + 4).contains(&f.id) { gim[(f.id - CAN_GIMBAL_CMD) as usize] = w; }
                else if f.id == CAN_VALVES { for i in 0..6 { val[i] = f.data[i] as f64; } }
            }
            for i in 0..3 { o.f(h.pwm[i] as f64); }
            o.s(&rot).s(&gim).s(&val);
        }
        "drivers::drv_read" => {
            // through the public path: the bytes on the stub HAL's buses, read by Drv::read
            let mut h = Stub { mag: a.u8s(), gyro: a.u8s(), sun: a.u8s(), es: a.u8s(), ..Stub::default() };
            let st = a.u8s::<96>(); let n_st = a.i() as usize;
            let gps = a.u8s::<96>(); let n_gps = a.i() as usize;
            let (ids, dlc) = (a.fs::<8>(), a.fs::<8>());
            let mut frames = Vec::new();
            for i in 0..8 { frames.push(CanFrame { id: ids[i] as u32, extended: 0, dlc: dlc[i] as u8, data: a.u8s() }); }
            let n_can = a.i() as usize;
            h.can_rx = frames[..n_can].to_vec();
            let mut p = Params::default();
            p.has_gyro = a.i() as u8; p.has_sun = a.i() as u8; p.has_es = a.i() as u8; p.has_st = a.i() as u8; p.has_gps = a.i() as u8;
            p.nr = a.i() as u8; p.rot_gi = a.u8s();
            h.uart[ST_UART as usize] = st[..n_st].to_vec();
            h.uart[GPS_UART as usize] = gps[..n_gps].to_vec();
            let mut z = drv::Meas::default();
            drv::Drv::default().read(&mut h, &p, &mut z);
            o.b(z.mag_ok).s(&z.b).b(z.gyro_ok).s(&z.w).b(z.sun_ok).s(&z.sun).b(z.es_ok).s(&z.nadir).b(z.st_ok).b(z.st_valid[0]).b(z.st_valid[1])
             .m(&z.q_st).b(z.gps_ok).s(&z.r).s(&z.v).s(&z.h).s(&z.delta);
        }
        _ => return None,
    }
    assert_eq!(a.at, x.len(), "{name}: the adapter read {} of {} inputs", a.at, x.len());
    Some(o.0)
}

#[test]
fn the_rust_flight_software_reproduces_the_pseudocode() {
    let vs = vectors();
    assert!(!vs.is_empty(), "no vectors (tools/pcode.py gen writes fsw/tests/pcode_vectors.txt)");
    let (mut values, mut exact, mut worst) = (0usize, 0usize, 0.0f64);
    let mut names: Vec<&str> = vs.iter().map(|v| v.name).collect();
    names.dedup();
    let missing: Vec<&str> = names.iter().copied()
        .filter(|n| !NOT_IN_RUST.iter().any(|(m, _)| m == n) && call_any(n, &vs.iter().find(|v| v.name == *n).unwrap().x).is_none()).collect();
    assert!(missing.is_empty(), "pseudocode functions with no Rust adapter and no stated reason: {missing:?}");
    for v in &vs {
        if NOT_IN_RUST.iter().any(|(m, _)| *m == v.name) { continue; }
        let got = call_any(v.name, &v.x).unwrap();
        assert_eq!(got.len(), v.y.len(), "{}: output count", v.name);
        for (k, (g, w)) in got.iter().zip(&v.y).enumerate() {
            let err = (g - w).abs() / w.abs().max(1e-300);
            values += 1;
            if g.to_bits() == w.to_bits() || (*g == 0.0 && *w == 0.0) { exact += 1; } else { worst = worst.max(err); }
            assert!(g.to_bits() == w.to_bits() || (*g == 0.0 && *w == 0.0) || (!v.exact && err < 1e-12),
                    "{} output {k} for {:?}: Rust {g:e}, pseudocode {w:e}{}", v.name, v.x, if v.exact { " (no transcendental: must agree bit for bit)" } else { "" });
        }
    }
    println!("{} functions ({} by name through the dispatcher), {} calls, {values} values, {exact} bit for bit, worst relative difference {worst:e}",
             names.len(), names.iter().filter(|n| BY_NAME.iter().any(|(m, _)| m == *n)).count(), vs.len());
}
