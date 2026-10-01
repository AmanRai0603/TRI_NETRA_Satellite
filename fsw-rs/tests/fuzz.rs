//! Fuzzing the Rust flight software on stable Rust: seeded, in-tree harnesses (no nightly, no
//! libFuzzer) that run in every test pass and as long as asked (`ADCS_FUZZ_N`, default 3,000 per
//! harness). Each feeds what the flight software takes from outside — a parameter blob, a
//! telecommand, the bus bytes of every sensor — and requires that nothing panics, that a step
//! never answers with a non-finite command, and that a refused input changes nothing.
//!
//! The inputs are structured mutations of valid ones (a byte flipped, a run overwritten, a field
//! set to an edge value, the CRC repaired so the mutation reaches the rule checks) and fully
//! random ones. A failure prints the harness, the seed and the iteration, which replay it.
#![allow(clippy::needless_range_loop, clippy::field_reassign_with_default)]
use adcs_fsw::devices::*;
use adcs_fsw::hal::{CanFrame, Hal, Status};
use adcs_fsw::params::{crc32, BLOB_SIZE};
use adcs_fsw::{Fsw, Mode, Params, ABI_VERSION};

/// xorshift64*: small, seeded, the same everywhere.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 { self.0 ^= self.0 >> 12; self.0 ^= self.0 << 25; self.0 ^= self.0 >> 27; self.0.wrapping_mul(0x2545F4914F6CDD1D) }
    fn below(&mut self, n: usize) -> usize { (self.next() % n.max(1) as u64) as usize }
    fn byte(&mut self) -> u8 { self.next() as u8 }
    fn chance(&mut self, p: f64) -> bool { (self.next() >> 11) as f64/(1u64 << 53) as f64 <= p }
}

fn iterations() -> usize { std::env::var("ADCS_FUZZ_N").ok().and_then(|v| v.parse().ok()).unwrap_or(3000) }

/// A bus that serves whatever bytes the harness puts on it.
#[derive(Default)]
struct Bus { now_ns: u64, mag: [u8; 7], gyro: [u8; 13], sun: [u8; 7], es: [u8; 7], uart: [Vec<u8>; 3], can_rx: Vec<CanFrame>, pwm: [i16; 8], can_tx: Vec<CanFrame>, absent: u8 }

impl Hal for Bus {
    fn time_ns(&mut self) -> u64 { self.now_ns }
    fn uart_read(&mut self, port: u8, buf: &mut [u8]) -> Result<usize, Status> {
        let q = &mut self.uart[(port as usize).min(2)];
        let n = q.len().min(buf.len());
        if n == 0 { return Err(Status::Timeout); }
        buf[..n].copy_from_slice(&q[..n]);
        q.drain(..n);
        Ok(n)
    }
    fn uart_write(&mut self, _: u8, _: &[u8]) -> Status { Status::Ok }
    fn i2c_xfer(&mut self, bus: u8, addr: u8, _tx: &[u8], rx: &mut [u8]) -> Status {
        let src = if bus == MAG_BUS && addr == MAG_ADDR { if self.absent & 1 != 0 { return Status::NoDev; } &self.mag }
            else if bus == SUN_BUS && addr == SUN_ADDR { if self.absent & 4 != 0 { return Status::NoDev; } &self.sun }
            else if bus == ES_BUS && addr == ES_ADDR { if self.absent & 8 != 0 { return Status::NoDev; } &self.es }
            else { return Status::NoDev };
        let n = rx.len().min(7);
        rx[..n].copy_from_slice(&src[..n]);
        Status::Ok
    }
    fn spi_xfer(&mut self, bus: u8, cs: u8, _tx: &[u8], rx: &mut [u8]) -> Status {
        if bus != GYRO_BUS || cs != GYRO_CS || self.absent & 2 != 0 { return Status::NoDev; }
        let n = rx.len().min(13);
        rx[..n].copy_from_slice(&self.gyro[..n]);
        Status::Ok
    }
    fn can_send(&mut self, _: u8, f: &CanFrame) -> Status { self.can_tx.push(*f); Status::Ok }
    fn can_recv(&mut self, _: u8) -> Result<CanFrame, Status> { if self.can_rx.is_empty() { Err(Status::Timeout) } else { Ok(self.can_rx.remove(0)) } }
    fn pwm_set(&mut self, ch: u8, d: i16) -> Status { if ch < 8 { self.pwm[ch as usize] = d; } Status::Ok }
}

/// Two valid configurations: a magnetic detumble-to-pointing satellite, and a wheeled one with a
/// star tracker and GNSS flying fine pointing.
fn valid(which: usize) -> Params {
    let mut p = Params::default();
    p.jd0 = 2461407.25; p.dt = 0.1; p.mu = 3.986004418e14;
    p.mtq_period = 1.0; p.mtq_meas = 0.2; p.m_max = 0.2; p.bdot_k = 1e-3; p.has_gyro = 1; p.has_sun = 1;
    p.J[0][0] = 0.0067; p.J[1][1] = 0.042; p.J[2][2] = 0.042; p.igrf_nmax = 10; p.rate_lpf_s = 0.3;
    p.ss_eclipse = 1; p.gd_T = 1.0; p.mtq_phi = 0.01; p.rw_phi = 0.01; p.rw_dt = 0.1; p.fdir_s = 3.0; p.rcsd_T_damp_s = 20.0; p.rcsd_period_s = 1.0;
    p.st_coast_s = 900.0; p.mekf_sig_mag = 0.01; p.mekf_sig_sun = 0.005; p.mekf_meas_scale = 1.0; p.mekf_gate = 16.27; p.mekf_rej_max = 30.0;
    p.mtq_Kp = [1e-6; 3]; p.mtq_Kd = [1e-4; 3]; p.rw_Kp = [1e-3; 3]; p.rw_Kd = [1e-2; 3];
    p.start_mode = Mode::Detumble as u8; p.auto_next = Mode::NadirMtq as u8; p.detumble_exit = 0.01; p.detumble_hold_s = 5.0;
    if which == 1 {
        p.nr = 4; p.has_st = 1; p.n_heads = 1; p.has_gps = 1; p.gnss_ecef = 1;
        p.st_bs[0] = [0.0, 0.0, 1.0]; p.st_noise_cross = 1e-5; p.st_noise_roll = 1e-4;
        let s = 1.0/3f64.sqrt();
        p.rot_a0 = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [s, s, s], [0.0; 3], [0.0; 3], [0.0; 3], [0.0; 3]];
        for i in 0..4 { p.rot_tmax[i] = 2e-3; p.rot_hmax[i] = 0.03; }
        p.start_mode = Mode::NadirFine as u8; p.auto_next = 255;
        p.sched_t[0] = 30.0; p.sched_mode[0] = Mode::SlewFine as u8; p.n_sched = 1;
    }
    p
}

fn blob(p: &Params) -> Vec<u8> { let mut b = vec![0u8; BLOB_SIZE]; p.encode(&mut b); b }

/// Repair the CRC so a mutated payload reaches decode's rule checks instead of the CRC check.
fn reseal(b: &mut [u8]) {
    let n = BLOB_SIZE - 16;
    let c = crc32(&b[12..12 + n]);
    b[12 + n..16 + n].copy_from_slice(&c.to_le_bytes());
}

fn mutate(r: &mut Rng, b: &mut [u8]) {
    for _ in 0..1 + r.below(8) {
        let i = 12 + r.below(BLOB_SIZE - 16);
        match r.below(5) {
            0 => b[i] ^= 1 << r.below(8),
            1 => b[i] = r.byte(),
            2 => { let e: [f64; 6] = [0.0, -1.0, f64::NAN, f64::INFINITY, 1e300, -1e-300];
                   let x = e[r.below(6)].to_le_bytes(); let j = i.min(12 + BLOB_SIZE - 16 - 8); b[j..j + 8].copy_from_slice(&x); }
            3 => b[i] = [0u8, 1, 7, 8, 9, 10, 11, 254, 255][r.below(9)],
            _ => { let n = r.below(32); for k in i..(i + n).min(12 + BLOB_SIZE - 16) { b[k] = r.byte(); } }
        }
    }
}

/// Random register images, UART bytes (often near-valid frames), CAN telemetry and absent devices.
fn shake(r: &mut Rng, bus: &mut Bus) {
    for x in bus.mag.iter_mut().chain(bus.gyro.iter_mut()).chain(bus.sun.iter_mut()).chain(bus.es.iter_mut()) { if r.chance(0.3) { *x = r.byte(); } }
    if r.chance(0.5) { bus.mag[0] |= 1; bus.gyro[0] |= 1; }
    bus.absent = if r.chance(0.1) { r.byte() & 0x0f } else { 0 };
    for port in [ST_UART, GPS_UART] {
        let q = &mut bus.uart[port as usize];
        if r.chance(0.3) {
            // the start of a frame of either device's sync, then a length and random bytes
            q.extend_from_slice(&[[0xEB, 0x90][r.below(2)], [0x90, 0x4E][r.below(2)], r.byte()]);
            for _ in 0..r.below(80) { q.push(r.byte()); }
        }
        if q.len() > 4096 { q.clear(); }
    }
    for _ in 0..r.below(4) {
        let mut d = [0u8; 8]; for x in d.iter_mut() { *x = r.byte(); }
        bus.can_rx.push(CanFrame { id: 0x100 + r.below(0x200) as u32, extended: 0, dlc: r.below(9) as u8, data: d });
    }
}

fn assert_finite(f: &Fsw, bus: &Bus, what: &str) {
    let mut d = [0.0; 48];
    let n = f.debug(&mut d);
    // the commands written: dipole (debug 20..23), rotor (23..31), gimbal (31..35), duty (35..41)
    for (i, x) in d[20..n.min(41)].iter().enumerate() { assert!(x.is_finite(), "{what}: command {} is {x}", 20 + i); }
    let _ = bus;
}

#[test]
fn fuzz_parameter_blobs() {
    let mut r = Rng(0x5eed_0001);
    let (mut accepted, mut refused) = (0, 0);
    for it in 0..iterations() {
        let mut b = blob(&valid(it % 2));
        if r.chance(0.05) { for x in b.iter_mut() { *x = r.byte(); } } else { mutate(&mut r, &mut b); if r.chance(0.9) { reseal(&mut b); } }
        let mut f = Fsw::new();
        match f.init(ABI_VERSION, &b, 0) {
            Err(_) => { refused += 1; assert!(f.peek().is_none(), "blobs it={it}: a refused blob left the flight software running"); }
            Ok(()) => {
                accepted += 1;
                let mut bus = Bus::default();
                for k in 0..20u64 { shake(&mut r, &mut bus); bus.now_ns = k*100_000_000; let now = bus.now_ns; f.step(&mut bus, now); }
                assert_finite(&f, &bus, &format!("blobs it={it}"));
            }
        }
    }
    assert!(accepted > 0 && refused > 0, "the harness reaches both sides: {accepted} accepted, {refused} refused");
}

#[test]
fn fuzz_telecommands() {
    let mut r = Rng(0x5eed_0002);
    let mut f = Fsw::new();
    f.init(ABI_VERSION, &blob(&valid(1)), 0).expect("a valid configuration");
    let mut bus = Bus::default();
    for it in 0..iterations() {
        let n = r.below(12);
        let tc: Vec<u8> = (0..n).map(|_| if r.chance(0.5) { [0x01, 0x00, 0x02, 0x0a, 0x0b, 0xff][r.below(6)] } else { r.byte() }).collect();
        let before = f.peek().map(|s| s.mode);
        let rc = f.command(&tc);
        if rc != 0 { assert_eq!(f.peek().map(|s| s.mode), before, "tc it={it} {tc:?}: refused ({rc}) yet the mode changed"); }
        shake(&mut r, &mut bus);
        bus.now_ns += 100_000_000;
        let now = bus.now_ns;
        f.step(&mut bus, now);
        assert_finite(&f, &bus, &format!("tc it={it}"));
    }
}

#[test]
fn fuzz_bus_bytes() {
    let mut r = Rng(0x5eed_0003);
    for cfg in 0..2 {
        let mut f = Fsw::new();
        f.init(ABI_VERSION, &blob(&valid(cfg)), 0).expect("a valid configuration");
        let mut bus = Bus::default();
        for it in 0..iterations() {
            shake(&mut r, &mut bus);
            bus.now_ns += 100_000_000;
            let now = bus.now_ns;
            assert_eq!(f.step(&mut bus, now), 0, "bus cfg={cfg} it={it}: a step failed");
            assert_finite(&f, &bus, &format!("bus cfg={cfg} it={it}"));
        }
    }
}

#[test]
#[ignore]
fn replay_blob() {
    let target: usize = std::env::var("ADCS_FUZZ_AT").unwrap().parse().unwrap();
    let mut r = Rng(0x5eed_0001);
    for it in 0..=target {
        let mut b = blob(&valid(it % 2));
        if r.chance(0.05) { for x in b.iter_mut() { *x = r.byte(); } } else { mutate(&mut r, &mut b); if r.chance(0.9) { reseal(&mut b); } }
        let mut f = Fsw::new();
        if f.init(ABI_VERSION, &b, 0).is_ok() {
            let mut bus = Bus::default();
            for k in 0..20u64 { shake(&mut r, &mut bus); bus.now_ns = k*100_000_000; let now = bus.now_ns; f.step(&mut bus, now); }
        }
        if it == target {
            let v = valid(it % 2); let good = blob(&v);
            let diff: Vec<usize> = (0..BLOB_SIZE).filter(|&i| b[i] != good[i]).collect();
            println!("differs at {diff:?}");
            let p = format!("{:#?}", Params::decode(&b).unwrap());
            let q = format!("{:#?}", Params::decode(&good).unwrap());
            for (a, c) in p.lines().zip(q.lines()) { if a != c { println!("NOW {a}   WAS {c}"); } }
        }
    }
}
