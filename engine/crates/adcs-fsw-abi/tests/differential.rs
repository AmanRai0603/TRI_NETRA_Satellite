//! Differential fuzzing: the C and the Rust flight software fed the same configuration (mutated
//! from valid ones, kept only when both accept it), the same telecommands and the same random bus
//! bytes, tick after tick. They must agree bit for bit: the same refusal at init, the same return
//! of every step, the same PWM words and CAN frames, the same debug vector. This is the
//! bit-identity of the shipped scenarios extended to inputs no scenario produces.
//!
//! Seeded and stable (no nightly): `ADCS_FUZZ_N` sets the number of configurations (default 60),
//! each flown for 200 ticks. A failure names the seed, the configuration and the tick.
use adcs_fsw::params::{crc32, BLOB_SIZE};
use adcs_fsw::{Mode, Params};
use adcs_fsw_abi::{Bus, Fsw, Impl};

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 { self.0 ^= self.0 >> 12; self.0 ^= self.0 << 25; self.0 ^= self.0 >> 27; self.0.wrapping_mul(0x2545F4914F6CDD1D) }
    fn below(&mut self, n: usize) -> usize { (self.next() % n.max(1) as u64) as usize }
    fn byte(&mut self) -> u8 { self.next() as u8 }
    fn chance(&mut self, p: f64) -> bool { (self.next() >> 11) as f64/(1u64 << 53) as f64 <= p }
    fn uniform(&mut self, lo: f64, hi: f64) -> f64 { lo + (hi - lo)*((self.next() >> 11) as f64/(1u64 << 53) as f64) }
}

/// A configuration within the rules, with its laws, modes and devices drawn at random.
fn draw(r: &mut Rng) -> Params {
    let mut p = Params::default();
    p.jd0 = 2461407.25 + r.uniform(0.0, 365.0); p.dt = [0.05, 0.1, 0.2][r.below(3)]; p.mu = 3.986004418e14;
    p.mtq_period = 1.0; p.mtq_meas = 0.2; p.m_max = r.uniform(0.05, 1.0); p.bdot_k = r.uniform(1e-5, 1e-2);
    p.has_gyro = r.chance(0.8) as u8; p.has_sun = r.chance(0.8) as u8; p.has_gps = r.chance(0.7) as u8; p.gnss_ecef = 1;
    let j = [r.uniform(0.005, 0.1), r.uniform(0.005, 0.1), r.uniform(0.005, 0.1)];
    for i in 0..3 { p.J[i][i] = j[i]; }
    p.igrf_nmax = 1 + r.below(13) as u8; p.rate_lpf_s = r.uniform(0.0, 1.0);
    p.ss_eclipse = 1 + r.below(2) as u8; p.gd_T = r.uniform(10.0, 300.0); p.mtq_phi = 0.01; p.rw_phi = 0.01; p.rw_dt = 0.1; p.fdir_s = 3.0; p.fdir_win_s = 120.0; p.fdir_h_frac = 0.005;
    p.rcsd_T_damp_s = 20.0; p.rcsd_period_s = 1.0; p.st_coast_s = 900.0;
    p.mekf_sig_mag = 0.01; p.mekf_sig_sun = 0.005; p.mekf_meas_scale = 1.0; p.mekf_gate = 16.27; p.mekf_rej_max = 30.0;
    for ax in 0..3 { p.mtq_Kp[ax] = j[ax]*2.5e-5; p.mtq_Kd[ax] = j[ax]*0.02; p.rw_Kp[ax] = j[ax]*0.8; p.rw_Kd[ax] = j[ax]*3.6; p.rw_Ki[ax] = j[ax]*0.1; }
    p.bdot_law = r.below(4) as u8; p.mtq_law = r.below(9) as u8; p.rw_law = r.below(3) as u8; p.ss_law = r.below(3) as u8;
    p.detumble_exit = 0.01; p.detumble_hold_s = r.uniform(1.0, 30.0);
    if r.chance(0.6) {
        p.nr = 3 + r.below(2) as u8;
        let s = 1.0/3f64.sqrt();
        let ax = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [s, s, s]];
        for i in 0..p.nr as usize { p.rot_a0[i] = ax[i]; p.rot_tmax[i] = 2e-3; p.rot_hmax[i] = 0.03; }
        p.h_bias = 2e-3; p.dump_k = 2e-3;
    }
    if r.chance(0.5) { p.has_st = 1; p.n_heads = 1; p.st_bs[0] = [0.0, 0.0, 1.0]; p.st_noise_cross = 1e-5; p.st_noise_roll = 1e-4; }
    let modes: Vec<u8> = (0..11u8).filter(|m| match *m { 2 | 3 | 4 | 10 => p.nr > 0, 8 => p.nr > 0, 7 => false, _ => true }).collect();
    p.start_mode = modes[r.below(modes.len())];
    p.auto_next = if r.chance(0.5) { modes[r.below(modes.len())] } else { 255 };
    p.n_sched = r.below(3) as u8;
    for i in 0..p.n_sched as usize { p.sched_t[i] = r.uniform(1.0, 20.0); p.sched_mode[i] = modes[r.below(modes.len())]; }
    p.gd_kind = r.below(5) as u8;
    let _ = Mode::Detumble;
    p
}

fn blob(p: &Params) -> Vec<u8> { let mut b = vec![0u8; BLOB_SIZE]; p.encode(&mut b); b }

fn mutate(r: &mut Rng, b: &mut [u8]) {
    let n = BLOB_SIZE - 16;
    for _ in 0..r.below(3) {
        let i = 12 + r.below(n);
        if r.chance(0.5) { b[i] ^= 1 << r.below(8); } else { b[i] = r.byte(); }
    }
    let c = crc32(&b[12..12 + n]);
    b[12 + n..16 + n].copy_from_slice(&c.to_le_bytes());
}

/// The same random bus image for both: register images, UART bytes, CAN telemetry.
fn shake(r: &mut Rng, a: &mut Bus, b: &mut Bus) {
    let pick = |r: &mut Rng, n: usize| -> Option<Vec<u8>> { if r.chance(0.05) { None } else { Some((0..n).map(|_| r.byte()).collect()) } };
    let mag = pick(r, 7).map(|mut v| { v[0] |= 1; v });
    let gyro = pick(r, 13).map(|mut v| { v[0] |= 1; v });
    let sun = pick(r, 7);
    let es = pick(r, 7);
    for bus in [&mut *a, &mut *b] {
        bus.mag = mag.as_ref().map(|v| v[..7].try_into().unwrap());
        bus.gyro = gyro.as_ref().map(|v| v[..13].try_into().unwrap());
        bus.sun = sun.as_ref().map(|v| v[..7].try_into().unwrap());
        bus.es = es.as_ref().map(|v| v[..7].try_into().unwrap());
    }
    for port in [1u8, 2] {
        if r.chance(0.3) {
            let mut f = vec![[0xEB, 0x90][r.below(2)], [0x90, 0x91][r.below(2)], r.byte()];
            for _ in 0..r.below(60) { f.push(r.byte()); }
            a.push_uart(port, &f); b.push_uart(port, &f);
        }
    }
    for _ in 0..r.below(5) {
        let id = 0x100 + r.below(0x40) as u32;
        let mut d = [0u8; 8]; for x in d.iter_mut() { *x = r.byte(); }
        a.push_can(id, d); b.push_can(id, d);
    }
}

#[test]
fn c_and_rust_agree_bit_for_bit_on_random_inputs() {
    let n: usize = std::env::var("ADCS_FUZZ_N").ok().and_then(|v| v.parse().ok()).unwrap_or(60);
    let mut r = Rng(0xd1ff_0001);
    let (mut flown, mut refused) = (0, 0);
    for cfg in 0..n {
        let mut bl = blob(&draw(&mut r));
        if r.chance(0.3) { mutate(&mut r, &mut bl); }
        let (mut bc, mut br) = (Bus::default(), Bus::default());
        let c = Fsw::init(Impl::C, &bl, 0, &mut bc);
        let rs = Fsw::init(Impl::Rust, &bl, 0, &mut br);
        let (mut c, mut rs) = match (c, rs) {
            (Ok(c), Ok(rs)) => (c, rs),
            (Err(_), Err(_)) => { refused += 1; continue; }
            (a, b) => panic!("cfg {cfg}: C init {:?}, Rust init {:?}: one accepted what the other refused", a.err(), b.err()),
        };
        flown += 1;
        for k in 0..200u64 {
            shake(&mut r, &mut bc, &mut br);
            if r.chance(0.02) {
                let tc = [0x01, r.below(12) as u8];
                assert_eq!(c.command(&tc), rs.command(&tc), "cfg {cfg} tick {k}: telecommand {tc:?} answered differently");
            }
            let now = k*100_000_000 + 1;
            bc.now_ns = now; br.now_ns = now;
            let (sc, sr) = (c.step(&mut bc, now), rs.step(&mut br, now));
            assert_eq!(sc, sr, "cfg {cfg} tick {k}: step returned {sc} (C) and {sr} (Rust)");
            assert_eq!(bc.pwm, br.pwm, "cfg {cfg} tick {k}: PWM words differ");
            let (tc_, tr) = (std::mem::take(&mut bc.can_tx), std::mem::take(&mut br.can_tx));
            assert_eq!(tc_.len(), tr.len(), "cfg {cfg} tick {k}: CAN frame count differs");
            for (x, y) in tc_.iter().zip(&tr) { assert!(x.id == y.id && x.dlc == y.dlc && x.data == y.data, "cfg {cfg} tick {k}: CAN frame differs: C {:x} {} {:?}, Rust {:x} {} {:?}", x.id, x.dlc, x.data, y.id, y.dlc, y.data); }
            let (dc, dr) = (c.debug(), rs.debug());
            for i in 0..dc.len() {
                assert!(dc[i].to_bits() == dr[i].to_bits() || (dc[i].is_nan() && dr[i].is_nan()), "cfg {cfg} tick {k}: debug[{i}] {} (C) vs {} (Rust)", dc[i], dr[i]);
            }
            bc.can_rx.clear(); br.can_rx.clear();
        }
        drop(c);
    }
    eprintln!("differential: {flown} configurations flown 200 ticks each, {refused} refused by both");
    assert!(flown > n/4, "the harness flies most configurations ({flown} flown, {refused} refused)");
}
