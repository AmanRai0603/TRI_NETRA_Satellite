//! Host unit tests of the Rust flight software: the same vectors as fsw/tests/test_fsw.c
//! (reference numbers marked "twin" come from the MATLAB twin asils.*).
#![allow(clippy::needless_range_loop, clippy::field_reassign_with_default)]
use adcs_fsw::devices::*;
use adcs_fsw::drv::{crc16, Drv, Meas};
use adcs_fsw::hal::{CanFrame, Hal, Status};
use adcs_fsw::math::*;
use adcs_fsw::{alloc, ctl, env, est, Fsw, Mode, Params, ABI_VERSION};

/// In-memory HAL (= fsw/tests/hal_stub.c).
#[derive(Default)]
struct Stub { now_ns: u64, mag: [u8; 7], gyro: [u8; 13], sun: [u8; 7], es: [u8; 7], uart: [Vec<u8>; 3], can_rx: Vec<CanFrame>, can_tx: Vec<CanFrame>, pwm: [i16; 8] }

impl Hal for Stub {
    fn time_ns(&mut self) -> u64 { self.now_ns }
    fn uart_read(&mut self, port: u8, buf: &mut [u8]) -> Result<usize, Status> {
        let q = &mut self.uart[port as usize];
        let n = q.len().min(buf.len());
        if n == 0 { return Err(Status::Timeout); }
        buf[..n].copy_from_slice(&q[..n]);
        q.drain(..n);
        Ok(n)
    }
    fn uart_write(&mut self, _: u8, _: &[u8]) -> Status { Status::Ok }
    fn i2c_xfer(&mut self, bus: u8, addr: u8, _tx: &[u8], rx: &mut [u8]) -> Status {
        let src = if bus == MAG_BUS && addr == MAG_ADDR { &self.mag } else if bus == SUN_BUS && addr == SUN_ADDR { &self.sun } else if bus == ES_BUS && addr == ES_ADDR { &self.es } else { return Status::NoDev };
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
    fn can_send(&mut self, _: u8, f: &CanFrame) -> Status { self.can_tx.push(*f); Status::Ok }
    fn can_recv(&mut self, _: u8) -> Result<CanFrame, Status> { if self.can_rx.is_empty() { Err(Status::Timeout) } else { Ok(self.can_rx.remove(0)) } }
    fn pwm_set(&mut self, ch: u8, d: i16) -> Status { if ch < 8 { self.pwm[ch as usize] = d; } Status::Ok }
}

#[test]
fn math() {
    let a = qnorm(&[0.1, -0.3, 0.2, 0.9]);
    let b = qnorm(&[-0.4, 0.2, 0.5, 0.7]);
    let ab = dcm(&qmult(&a, &b));
    let ba = mat3_mul(&dcm(&b), &dcm(&a));
    let mut e = 0.0;
    for i in 0..3 { for j in 0..3 { e += (ab[i][j] - ba[i][j]).abs(); } }
    assert!(e < 1e-14, "dcm(a*b) = dcm(b) dcm(a): {e}");
    let q = fromdcm(&dcm(&a));
    assert!(((q[0]*a[0] + q[1]*a[1] + q[2]*a[2] + q[3]*a[3]).abs() - 1.0).abs() < 1e-14);
    let mut p = [[0.0; 8]; 3];
    p[0][..4].copy_from_slice(&[0.8165, 0.0, -0.8165, 0.0]);
    p[1][..4].copy_from_slice(&[0.0, 0.8165, 0.0, -0.8165]);
    p[2][..4].copy_from_slice(&[0.5774; 4]);
    let pi = pinv_rows(&p, 4);
    let mut i3 = 0.0;
    for i in 0..3 { for j in 0..3 {
        let s: f64 = (0..4).map(|k| p[i][k]*pi[k][j]).sum();
        i3 += (s - if i == j { 1.0 } else { 0.0 }).abs();
    } }
    assert!(i3 < 1e-9, "A pinv(A) = I: {i3}");
    let k = [[4.0, 1.0, 0.0, 0.0], [1.0, 3.0, 0.0, 0.0], [0.0, 0.0, 2.0, 0.5], [0.0, 0.0, 0.5, 1.0]];
    let (lam, _) = jacobi_eig4(&k);
    let mx = lam.iter().cloned().fold(-1e9, f64::max);
    assert!((mx - (3.5 + 1.25f64.sqrt())).abs() < 1e-12);
}

#[test]
fn environment_vs_twin() {
    let gh = env::igrf_gh(2027.1);
    let b = env::igrf_ned(&gh, 0.3, 1.2, 550.0, 13);
    assert!((b[0] - 29155.707342).abs() < 1e-5 && (b[1] + 187.025096).abs() < 1e-5 && (b[2] - 13232.587581).abs() < 1e-5, "IGRF {b:?}");
    let s = env::sun_model(2461407.25);
    assert!((s[0] - 0.185796268321).abs() < 1e-11 && (s[1] + 0.901530663479).abs() < 1e-11 && (s[2] + 0.390796890321).abs() < 1e-11, "Sun {s:?}");
}

#[test]
fn estimation() {
    let q = qnorm(&[0.2, -0.1, 0.4, 0.8]);
    let a = dcm(&q);
    let mut b = [[0.0; 3]; 8];
    let mut r = [[0.0; 3]; 8];
    for i in 0..8 {
        let f = i as f64;
        r[i] = unit(&[(1.3*f + 0.2).sin(), (0.7*f + 1.1).cos(), (2.1*f - 0.4).sin()]);
        b[i] = mat3_vec(&a, &r[i]);
    }
    let (qm, _) = est::quest(&b, &r, None);
    assert!(qangle(&q, &qm) < 1e-9, "QUEST");
    let qt = est::triad(&b[0], &b[1], &r[0], &r[1]).unwrap();
    assert!(qangle(&q, &qt) < 1e-9, "TRIAD");
    assert!(est::triad(&b[0], &scale3(&b[0], 2.0), &r[0], &r[1]).is_none(), "TRIAD on parallel vectors fixes no attitude");
    let q0 = qmult(&q, &fromrotvec(&[0.1, -0.12, 0.08]));
    let mut k = est::Mekf::new(&q0, 0.2, 1e-3, 1e-6, 1e-8);
    for _ in 0..200 {
        k.predict(&[0.0; 3], 0.1);
        k.vector(&b[0], &r[0], 1e-3, 0.0);
        k.vector(&b[1], &r[1], 1e-3, 0.0);
    }
    assert!(qangle(&q, &k.q)*180.0/PI < 0.01, "MEKF");
}

#[test]
fn laws() {
    let bb = [2e-5, -1e-5, 3e-5];
    let w = [0.05, -0.02, 0.1];
    let wd = [0.0, 0.0, 0.1];
    let bd = scale3(&cross(&w, &bb), -1.0);
    let m = ctl::gen_bdot(&bb, &bd, &wd, 1e6);
    assert!(dot(&cross(&m, &bb), &sub3(&w, &wd)) < 0.0, "L1 drives w toward w_d");
    let m = ctl::torque2dipole(&w, &bb, 10.0);
    assert!(dot(&m, &bb).abs() < 1e-18);
    let j = [[0.0067, 0.0, 0.0], [0.0, 0.042, 0.0], [0.0, 0.0, 0.042]];
    let s = unit(&[0.3, -0.5, -0.8]);
    for k in 0..50 {
        let kf = k as f64;
        let bk = [3e-5*kf.sin(), 3e-5*(1.7*kf).cos(), 2e-5*(0.3*kf + 1.0).sin()];
        let wk = [0.1*(2.0*kf).sin(), 0.1*kf.cos(), 0.1*(kf + 2.0).sin()];
        let m = ctl::sun_spin(&bk, &wk, &s, false, &j, 6.0, 0.01, 0.05, 0.0);
        let h = mat3_vec(&j, &wk);
        let rz = [0.035, 0.0, 0.0];
        let mut x = [0.0; 3];
        for i in 0..3 {
            let hd = (if wk[2] >= 0.0 { 1.0 } else { -1.0 })*0.042*(-6.0*D2R)*s[i];
            x[i] = 0.01*(h[i] - hd) + 0.05*rz[i]*wk[i];
        }
        assert!(dot(&cross(&bk, &x), &m) <= 1e-20, "Sun spin L2 A.m0 <= 0 at {k}");
    }
}

#[test]
fn a_value_outside_its_rule_is_refused_at_init() {
    let good = detumble_blob();
    let mut f = Fsw::new();
    assert!(f.init(ABI_VERSION, &good, 0).is_ok());
    for (field, set) in [("nr", Box::new(|p: &mut Params| p.nr = 9) as Box<dyn Fn(&mut Params)>),
                         ("m_max", Box::new(|p: &mut Params| p.m_max = f64::NAN)),
                         ("start_mode", Box::new(|p: &mut Params| p.start_mode = 11)),
                         ("rot_gi", Box::new(|p: &mut Params| { p.nr = 1; p.rot_a0[0] = [1.0, 0.0, 0.0]; p.rot_tmax[0] = 0.01; p.rot_gi[0] = 1; })),
                         // finite but no physical setting: it overflows inside the laws (fuzz.rs found it)
                         ("dump_k", Box::new(|p: &mut Params| p.dump_k = 2.8e272)),
                         ("bdot_k", Box::new(|p: &mut Params| p.bdot_k = 1e-309)),
                         ("rot_a0", Box::new(|p: &mut Params| { p.nr = 1; p.rot_a0[0] = [2.0, 0.0, 0.0]; p.rot_tmax[0] = 0.01; })),
                         ("sun_axis", Box::new(|p: &mut Params| p.sun_axis = [0.0, 0.0, 0.5]))] {
        let mut p = Params::decode(&good).unwrap();
        set(&mut p);
        let mut blob = vec![0u8; adcs_fsw::params::BLOB_SIZE];
        p.encode(&mut blob);
        match f.init(ABI_VERSION, &blob, 0) {
            Err(adcs_fsw::fsw::InitError::Invalid(name)) => assert_eq!(name, field),
            other => panic!("{field}: {other:?}"),
        }
    }
}

#[test]
fn rcs_duty_with_mib() {
    let mut p = Params::default();
    p.nc = 6; p.rcs_mib = 0.005; p.rcs_res = 0.001;
    p.rcs_tau[0][0] = 4.5e-4; p.rcs_tau[1][0] = -4.5e-4; p.rcs_tau[2][1] = 1.5e-3; p.rcs_tau[3][1] = -1.5e-3; p.rcs_tau[4][2] = 1.5e-3; p.rcs_tau[5][2] = -1.5e-3;
    let (d, _) = alloc::rcs_duty(&[1e-4, 0.0, -2e-6], &p, 1.0);
    assert!((d[0] - 0.222).abs() < 1e-12 && d[5] == 0.0, "{d:?}");
}

fn put16(b: &mut [u8], v: f64) { b[..2].copy_from_slice(&((if v < 0.0 { v - 0.5 } else { v + 0.5 }) as i16).to_le_bytes()); }
fn put32(b: &mut [u8], v: f64) { b[..4].copy_from_slice(&((if v < 0.0 { v - 0.5 } else { v + 0.5 }) as i32).to_le_bytes()); }

fn detumble_blob() -> Vec<u8> {
    let mut p = Params::default();
    p.jd0 = 2461407.25; p.dt = 0.1; p.mu = 3.986004418e14; p.start_mode = Mode::Detumble as u8; p.auto_next = 255;
    p.bdot_law = 0; p.mtq_period = 1.0; p.mtq_meas = 0.2; p.m_max = 0.2; p.bdot_k = 1e-3; p.has_gyro = 1;
    p.J[0][0] = 0.0067; p.J[1][1] = 0.042; p.J[2][2] = 0.042; p.igrf_nmax = 10; p.rate_lpf_s = 0.3;
    // the values params.toml requires to be positive (the flight software refuses a blob without them)
    p.ss_eclipse = 1; p.gd_T = 1.0; p.mtq_phi = 0.01; p.rw_phi = 0.01; p.rw_dt = 0.1; p.fdir_s = 3.0; p.fdir_win_s = 120.0; p.fdir_h_frac = 0.005; p.rcsd_T_damp_s = 20.0; p.rcsd_period_s = 1.0;
    p.st_coast_s = 900.0; p.mekf_sig_mag = 0.01; p.mekf_sig_sun = 0.005; p.mekf_meas_scale = 1.0;
    let mut blob = vec![0u8; adcs_fsw::params::BLOB_SIZE];
    p.encode(&mut blob);
    blob
}

fn run_detumble() -> Vec<[i16; 3]> {
    let blob = detumble_blob();
    let mut hal = Stub::default();
    let mut f = Fsw::new();
    f.init(ABI_VERSION, &blob, 0).expect("init");
    let mut log = vec![];
    for k in 0..30u64 {
        let (b, w) = ([2e-5, -1e-5, 3e-5], [0.05, -0.02, 0.1]);
        hal.now_ns = k*100_000_000;
        hal.mag[0] = 1; for i in 0..3 { put16(&mut hal.mag[1 + 2*i..], b[i]/MAG_LSB_T); }
        hal.gyro[0] = 1; for i in 0..3 { put32(&mut hal.gyro[1 + 4*i..], w[i]/GYRO_LSB); }
        let now = hal.now_ns; f.step(&mut hal, now);
        log.push([hal.pwm[0], hal.pwm[1], hal.pwm[2]]);
    }
    log
}

/// The same detumble through the bytes; the PWM words are pinned to the C build's (test_fsw.c prints them).
#[test]
fn abi_detumble() {
    let a = run_detumble();
    assert_eq!(a, run_detumble(), "deterministic");
    let mut on = false;
    for (k, m) in a.iter().enumerate() {
        let ph = (k as f64*0.1 + 1e-9) % 1.0;
        if ph < 0.2 - 1e-9 { assert_eq!(*m, [0, 0, 0], "coils off in the measurement window, k={k}"); }
        if ph > 0.25 && *m != [0, 0, 0] { on = true; }
    }
    let (b, w) = ([2e-5, -1e-5, 3e-5], [0.05, -0.02, 0.1]);
    let bd = scale3(&cross(&w, &b), -1.0);
    let m = [a[5][0] as f64, a[5][1] as f64, a[5][2] as f64];
    assert!(on && dot(&m, &bd) < 0.0, "B-dot dipole opposes dB/dt");
    assert_eq!(a[2], [25936, 32767, -6826], "PWM words equal the C build (fsw/tests/test_fsw.c)");
    if let Ok(s) = std::env::var("ADCS_PRINT_PWM") { if s == "1" { println!("PWM {:?}", a); } }
    let mut blob = vec![0u8; adcs_fsw::params::BLOB_SIZE];
    Params::default().encode(&mut blob);
    blob[100] ^= 1;
    assert!(Fsw::new().init(ABI_VERSION, &blob, 0).is_err(), "bad CRC refused");
}

#[test]
fn st_frame() {
    let mut p = Params::default();
    p.has_st = 1; p.n_heads = 1;
    let q = [0.1, 0.2, -0.3, 0.927];
    let mut pl = [0u8; 18];
    pl[0] = 1; pl[1] = 1;
    for k in 0..4 { put32(&mut pl[2 + 4*k..], q[k]*Q30); }
    let c = crc16(&pl);
    let mut f = vec![ST_SYNC1, ST_SYNC2, 18];
    f.extend_from_slice(&pl);
    f.extend_from_slice(&c.to_le_bytes());
    let mut hal = Stub::default();
    hal.uart[ST_UART as usize].extend_from_slice(&[0x00, 0x13]);
    hal.uart[ST_UART as usize].extend_from_slice(&f);
    let mut drv = Drv::default();
    let mut z = Meas::default();
    drv.read(&mut hal, &p, &mut z);
    assert!(z.st_ok && z.st_valid[0] && (z.q_st[0][2] + 0.3).abs() < 1e-8);
}

/// A NaN or an infinite command drives nothing (= t_nonfinite in test_fsw.c).
#[test]
fn a_non_finite_command_drives_nothing() {
    let mut p = Params::default();
    p.m_max = 0.2; p.nr = 1; p.rot_tmax[0] = 1e-3; p.nc = 2; p.dt = 0.1;
    let (mut r, g, mut d) = ([0.0; adcs_fsw::params::MAX_ROTORS], [0.0; adcs_fsw::params::MAX_GIMBALS], [0.0; adcs_fsw::params::MAX_COUPLES]);
    r[0] = f64::NAN; d[0] = f64::NAN; d[1] = f64::NEG_INFINITY;
    let mut hal = Stub::default();
    adcs_fsw::drv::write(&mut hal, &p, &[f64::NAN, f64::INFINITY, 0.1], &r, &g, &d);
    assert_eq!(&hal.pwm[..3], &[0, 0, 16384]);
    assert_eq!(hal.can_tx.len(), 2);
    assert_eq!(&hal.can_tx[0].data[..2], &[0, 0]);
    assert_eq!(&hal.can_tx[1].data[..2], &[0, 0]);
}

/// The magnetometer stops answering (= t_safe in test_fsw.c): in detumble the coils act on the field,
/// then stop; a pointing state holds for 60 s, then it is magnetorquer detumble with the fault raised;
/// the fault clears when the magnetometer answers again.
#[test]
fn a_silent_magnetometer_brings_safe_mode() {
    let (mut on_before, mut on_after, mut mode59) = (false, false, None);
    for start in [Mode::Detumble as u8, Mode::NadirMtq as u8] {
        let mut p = Params::default();
        p.jd0 = 2461407.25; p.dt = 0.1; p.mu = 3.986004418e14; p.start_mode = start; p.auto_next = 255;
        p.mtq_period = 1.0; p.mtq_meas = 0.2; p.m_max = 0.2; p.bdot_k = 1e-3; p.has_gyro = 1;
        p.J[0][0] = 0.0067; p.J[1][1] = 0.042; p.J[2][2] = 0.042; p.igrf_nmax = 10; p.rate_lpf_s = 0.3;
        p.ss_eclipse = 1; p.gd_T = 1.0; p.mtq_phi = 0.01; p.rw_phi = 0.01; p.rw_dt = 0.1; p.fdir_s = 3.0; p.fdir_win_s = 120.0; p.fdir_h_frac = 0.005; p.rcsd_T_damp_s = 20.0; p.rcsd_period_s = 1.0;
        p.st_coast_s = 900.0; p.mekf_sig_mag = 0.01; p.mekf_sig_sun = 0.005; p.mekf_meas_scale = 1.0;
        let mut blob = vec![0u8; adcs_fsw::params::BLOB_SIZE];
        p.encode(&mut blob);
        let mut hal = Stub::default();
        let mut f = Fsw::new();
        f.init(ABI_VERSION, &blob, 0).expect("init");
        for k in 0..720u64 {
            let (b, w) = ([2e-5, -1e-5, 3e-5], [0.05, -0.02, 0.1]);
            hal.now_ns = k*100_000_000;
            hal.mag[0] = (k < 30 || k == 719) as u8; for i in 0..3 { put16(&mut hal.mag[1 + 2*i..], b[i]/MAG_LSB_T); }
            hal.gyro[0] = 1; for i in 0..3 { put32(&mut hal.gyro[1 + 4*i..], w[i]/GYRO_LSB); }
            let now = hal.now_ns; f.step(&mut hal, now);
            let on = hal.pwm[..3].iter().any(|x| *x != 0);
            if start == Mode::Detumble as u8 && k < 30 && on { on_before = true; }
            if start == Mode::Detumble as u8 && (30..719).contains(&k) && on { on_after = true; }
            if start == Mode::NadirMtq as u8 && k == 620 { mode59 = Some(f.peek().unwrap().mode); }
            if k == 718 {
                let s = f.peek().unwrap();
                assert!(s.mode == Mode::Detumble as u16 && s.faults & 0x100 != 0, "silent 60 s: mode {} faults {:x}", s.mode, s.faults);
            }
        }
        assert_eq!(f.peek().unwrap().faults & 0x100, 0, "the fault clears when the magnetometer answers");
    }
    assert!(on_before && !on_after, "coils act on the field, then stop ({on_before} {on_after})");
    // no rotors, no thrusters: fine pointing is refused by telecommand and at init
    let mut p = Params::default();
    p.jd0 = 2461407.25; p.dt = 0.1; p.mu = 3.986004418e14; p.start_mode = Mode::NadirMtq as u8; p.auto_next = 255;
    p.mtq_period = 1.0; p.mtq_meas = 0.2; p.m_max = 0.2; p.bdot_k = 1e-3; p.has_gyro = 1;
    p.J[0][0] = 0.0067; p.J[1][1] = 0.042; p.J[2][2] = 0.042; p.igrf_nmax = 10; p.rate_lpf_s = 0.3;
    p.ss_eclipse = 1; p.gd_T = 1.0; p.mtq_phi = 0.01; p.rw_phi = 0.01; p.rw_dt = 0.1; p.fdir_s = 3.0; p.fdir_win_s = 120.0; p.fdir_h_frac = 0.005; p.rcsd_T_damp_s = 20.0; p.rcsd_period_s = 1.0;
    p.st_coast_s = 900.0; p.mekf_sig_mag = 0.01; p.mekf_sig_sun = 0.005; p.mekf_meas_scale = 1.0;
    let mut blob = vec![0u8; adcs_fsw::params::BLOB_SIZE];
    p.encode(&mut blob);
    let mut f = Fsw::new();
    f.init(ABI_VERSION, &blob, 0).expect("init");
    assert_eq!(f.command(&[0x01, Mode::NadirFine as u8]), -3);
    assert_eq!(f.command(&[0x01, Mode::NadirMtq as u8]), 0);
    p.start_mode = Mode::NadirFine as u8;
    p.encode(&mut blob);
    assert!(matches!(f.init(ABI_VERSION, &blob, 0), Err(adcs_fsw::fsw::InitError::Infeasible(_))));
    assert_eq!(mode59, Some(Mode::NadirMtq as u16));
}
