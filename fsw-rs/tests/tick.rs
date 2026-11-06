//! The tick tests of the flight software (docs/S11_INVENTORY.md TS4, step S11.4): what the tick does with a warm reset, with a
//! step or a jitter in the time it is handed, and with a gyro that has gone quiet. Each test states the BEHAVIOUR THE FIXES OWE
//! (docs/TECHNICAL_ROADMAP.md F1 monotonic time kept through a reset and a step; F2 a silent gyro is flagged and its rate
//! dropped; F3 the rate groups on an integer tick counter). F1 and F3 (S11.4b) and F2 (S11.5) are fixed and run. A test of a
//! fix still to come is `#[ignore]`d with the step that fixes it; that step removes the `#[ignore]` (and the test's name from the C list in
//! fsw/tests/test_fsw.c); `cargo test -- --ignored` shows what each says today. Their scenario twins are `fault_fsw_reset_day30`, `fault_clock_step_back`/`_fwd`,
//! `fault_clock_jitter_1us`/`_5ms` and `fault_gyro_silent`, flown by the engine.
//! The sensors here are exact and the attitude is held (no noise, no dynamics): the test is of the tick, not of the estimate.
#![allow(clippy::needless_range_loop, clippy::field_reassign_with_default)]
use adcs_fsw::devices::*;
use adcs_fsw::drv::crc16;
use adcs_fsw::hal::{CanFrame, Hal, Status};
use adcs_fsw::math::*;
use adcs_fsw::{env, Fsw, Mode, Params, ABI_VERSION};

/// In-memory HAL (= fsw/tests/hal_stub.c), with a gyro that can stop answering.
#[derive(Default)]
struct Stub { now_ns: u64, mag: [u8; 7], gyro: [u8; 13], sun: [u8; 7], es: [u8; 7], uart: [Vec<u8>; 3], pwm: [i16; 8], gyro_silent: bool }

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
        if self.gyro_silent { return Status::Timeout; }
        let n = rx.len().min(13);
        rx[..n].copy_from_slice(&self.gyro[..n]);
        Status::Ok
    }
    fn can_send(&mut self, _: u8, _: &CanFrame) -> Status { Status::Ok }
    fn can_recv(&mut self, _: u8) -> Result<CanFrame, Status> { Err(Status::Timeout) }
    fn pwm_set(&mut self, ch: u8, d: i16) -> Status { if ch < 8 { self.pwm[ch as usize] = d; } Status::Ok }
}

fn put16(b: &mut [u8], v: f64) { b[..2].copy_from_slice(&((if v < 0.0 { v - 0.5 } else { v + 0.5 }) as i16).to_le_bytes()); }
fn put32(b: &mut [u8], v: f64) { b[..4].copy_from_slice(&((if v < 0.0 { v - 0.5 } else { v + 0.5 }) as i32).to_le_bytes()); }

const JD0: f64 = 2461407.25;
const DAY: f64 = 86400.0;
const STALE_LIMIT_S: f64 = 1.0;      // F2: DR-fdir-002 sets 3 ticks (0.3 s); the test looks once a second, so one second is its limit; far shorter than the 60 s fault

/// A configuration that boots: the values params.toml requires to be positive, the fitted parts, the start state.
fn params(start: Mode, sun_gps: bool) -> Params {
    let mut p = Params::default();
    p.jd0 = JD0; p.dt = 0.1; p.mu = 3.986004418e14; p.start_mode = start as u8; p.auto_next = 255;
    p.bdot_law = 0; p.mtq_period = 1.0; p.mtq_meas = 0.2; p.m_max = 0.2; p.bdot_k = 1e-3; p.has_gyro = 1;
    p.J[0][0] = 0.0067; p.J[1][1] = 0.042; p.J[2][2] = 0.042; p.igrf_nmax = 10; p.rate_lpf_s = 0.3;
    p.ss_eclipse = 1; p.gd_T = 1.0; p.mtq_phi = 0.01; p.rw_phi = 0.01; p.rw_dt = 0.1; p.fdir_s = 3.0; p.fdir_win_s = 120.0; p.fdir_h_frac = 0.005; p.rcsd_T_damp_s = 20.0; p.rcsd_period_s = 1.0;
    p.st_coast_s = 900.0; p.mekf_sig_mag = 0.01; p.mekf_sig_sun = 0.005; p.mekf_meas_scale = 1.0; p.clock_step_max_s = 60.0; p.gyro_stale_ticks = 3;
    if sun_gps {
        p.has_sun = 1; p.has_gps = 1; p.gnss_ecef = 0; p.gyro_arw = 1e-6; p.gyro_rrw = 1e-8;
        p.mtq_Kp = [1e-3; 3]; p.mtq_Kd = [1e-2; 3];
    }
    p
}

fn blob(p: &Params) -> Vec<u8> {
    let mut b = vec![0u8; adcs_fsw::params::BLOB_SIZE];
    p.encode(&mut b);
    b
}

fn jit(seed: &mut u64) -> f64 {   // a splitmix64 uniform in (-1, 1), the test's own
    *seed = seed.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = *seed;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^= z >> 31;
    2.0*((z >> 11) as f64 + 0.5)/9007199254740992.0 - 1.0
}

// ---- F3: the rate groups run on the ticks, whatever the timestamps say ----

/// 30 s of detumble with timestamps off by +/- amp (uniform, seeded) against the exact ones: the commands must be the same words.
fn detumble_pwm(amp_s: f64) -> Vec<[i16; 3]> {
    let (p, mut hal, mut f) = (params(Mode::Detumble, false), Stub::default(), Fsw::new());
    f.init(ABI_VERSION, &blob(&p), 0).expect("init");
    let (mut seed, mut out) = (7u64, vec![]);
    for k in 0..300u64 {
        let (b, w) = ([2e-5, -1e-5, 3e-5], [0.05, -0.02, 0.1]);
        let off = if k == 0 { 0.0 } else { amp_s*jit(&mut seed) };
        hal.now_ns = ((k as f64*0.1 + off)*1e9).round() as u64;
        hal.mag[0] = 1; for i in 0..3 { put16(&mut hal.mag[1 + 2*i..], b[i]/MAG_LSB_T); }
        // counts that change, as a gyro's do (a repeated sample is a quiet gyro, F2)
        hal.gyro[0] = 1; for i in 0..3 { put32(&mut hal.gyro[1 + 4*i..], w[i]/GYRO_LSB + ((k*7 + i as u64*3) % 5) as f64); }
        let now = hal.now_ns; f.step(&mut hal, now);
        out.push([hal.pwm[0], hal.pwm[1], hal.pwm[2]]);
    }
    out
}

fn jitter_changes_no_command(amp_s: f64) {
    let (exact, jittered) = (detumble_pwm(0.0), detumble_pwm(amp_s));
    let differ = exact.iter().zip(&jittered).filter(|(a, b)| a != b).count();
    assert_eq!(differ, 0, "timestamps off by +/- {amp_s} s moved the commands of {differ} of {} ticks", exact.len());
}

#[test]
fn timestamps_off_by_1_us_change_no_command() { jitter_changes_no_command(1e-6); }

#[test]
fn timestamps_off_by_5_ms_change_no_command() { jitter_changes_no_command(5e-3); }

// ---- F1: the date the flight software keeps ----

/// The scene a sun sensor, a magnetometer, a gyro at rest and a GNSS receiver give, at the truth date `jd` and the time `t`
/// since the scene began, for the held attitude `q` (ECI to body): a circular orbit, the field and the Sun from the models.
struct Scene { q: Q, gh: [f64; 195] }
impl Scene {
    fn new() -> Scene { Scene { q: qnorm(&[0.2, -0.1, 0.4, 0.8]), gh: env::igrf_gh(env::decyear(JD0)) } }
    fn feed(&self, hal: &mut Stub, t: f64, jd: f64) {
        let (n, rr, inc) = (0.0011f64, 6.9e6f64, 97.0f64.to_radians());
        let (u, du) = (n*t, n);
        let r = [rr*u.cos(), rr*u.sin()*inc.cos(), rr*u.sin()*inc.sin()];
        let v = [-rr*du*u.sin(), rr*du*u.cos()*inc.cos(), rr*du*u.cos()*inc.sin()];
        let a = dcm(&self.q);
        let b = mat3_vec(&a, &env::field_eci(&r, jd, &self.gh, 10));
        let s = mat3_vec(&a, &env::sun_model(jd));
        hal.mag[0] = 1; for i in 0..3 { put16(&mut hal.mag[1 + 2*i..], b[i]/MAG_LSB_T); }
        hal.sun[0] = 1; for i in 0..3 { put16(&mut hal.sun[1 + 2*i..], s[i]*ADCS_Q15); }
        hal.gyro[0] = 1;                                                    // at rest: counts that change a little, as a gyro's do (a repeated sample is a quiet gyro, F2)
        let k = (t*10.0).round() as u64; for i in 0..3 { put32(&mut hal.gyro[1 + 4*i..], ((k*7 + i as u64*3) % 5) as f64); }
        let mut pl = [0u8; 33];
        pl[0] = 1;
        for i in 0..3 { put32(&mut pl[1 + 4*i..], r[i]*100.0); put32(&mut pl[13 + 4*i..], v[i]*1000.0); }
        let ds = (jd - 2451545.0)*86400.0;                                  // the fix's date after its velocity: seconds and microseconds from J2000.0
        put32(&mut pl[25..], ds.floor()); put32(&mut pl[29..], (ds - ds.floor())*1e6);
        let mut fr = vec![ST_SYNC1, GPS_SYNC2, 33];
        fr.extend_from_slice(&pl);
        fr.extend_from_slice(&crc16(&pl).to_le_bytes());
        hal.uart[GPS_UART as usize].extend_from_slice(&fr);
    }
    /// the angle [deg] between the flight software's attitude estimate and the held attitude (180: none)
    fn err_deg(&self, f: &Fsw) -> f64 {
        let mut d = [0.0f64; 8];
        f.debug(&mut d);
        if d[2] == 0.0 { return 180.0; }
        qangle(&self.q, &[d[3], d[4], d[5], d[6]]).to_degrees()
    }
}
const ADCS_Q15: f64 = 32767.0;

/// Run the scene for `secs` of truth time from `t0`, the clock the flight software reads being `clock(t)`; returns the worst
/// attitude error [deg] over the last 20 s.
fn fly(f: &mut Fsw, hal: &mut Stub, sc: &Scene, age_s: f64, t0: f64, secs: f64, clock: &dyn Fn(f64) -> f64) -> f64 {
    let mut worst: f64 = 0.0;
    let n = (secs/0.1).round() as u64;
    for k in 0..n {
        let t = t0 + k as f64*0.1;
        sc.feed(hal, t, JD0 + (age_s + t)/DAY);
        hal.now_ns = (clock(t)*1e9).round() as u64;
        let now = hal.now_ns; f.step(hal, now);
        if k >= n.saturating_sub(200) { worst = worst.max(sc.err_deg(f)); }
    }
    worst
}

/// A warm reset 30 days after the blob's date: the clock goes on, the state starts again, and the date stays the truth's.
#[test]
fn a_warm_reset_30_days_in_keeps_the_date() {
    let (age, p) = (30.0*DAY, params(Mode::NadirMtq, true));
    let (mut hal, mut f, sc) = (Stub::default(), Fsw::new(), Scene::new());
    // the blob is dated at the boot, 30 days before the scene: the software booted with the clock at 0 and reads age + t
    f.init(ABI_VERSION, &blob(&p), 0).expect("init");
    let before = fly(&mut f, &mut hal, &sc, age, 0.0, 120.0, &|t| age + t);
    assert!(before < 1.0, "before the reset the attitude is known to {before} deg (the rig itself)");
    f.init(ABI_VERSION, &blob(&p), ((age + 120.0)*1e9) as u64).expect("reset");
    let after = fly(&mut f, &mut hal, &sc, age, 120.0, 120.0, &|t| age + t);
    assert!(after < 1.0, "120 s after the reset the attitude is known to {after} deg: the date went back by 30 days");
}

/// The time source steps forward by an hour: the date does not (a bounded step, or none).
#[test]
fn a_clock_step_of_plus_1_hour_keeps_the_date() {
    let (p, mut hal, mut f, sc) = (params(Mode::NadirMtq, true), Stub::default(), Fsw::new(), Scene::new());
    f.init(ABI_VERSION, &blob(&p), 0).expect("init");
    let before = fly(&mut f, &mut hal, &sc, 0.0, 0.0, 120.0, &|t| t);
    assert!(before < 1.0, "before the step the attitude is known to {before} deg (the rig itself)");
    let after = fly(&mut f, &mut hal, &sc, 0.0, 120.0, 240.0, &|t| t + 3600.0);
    assert!(after < 1.0, "240 s after a +1 h step the attitude is known to {after} deg: the date moved by an hour");
}

/// The time source steps back by a second: the time the software keeps does not go back.
#[test]
fn a_clock_step_of_minus_1_s_never_runs_the_time_back() {
    let (p, mut hal, mut f) = (params(Mode::Detumble, false), Stub::default(), Fsw::new());
    f.init(ABI_VERSION, &blob(&p), 0).expect("init");
    let mut last = 0.0f64;
    for k in 0..300u64 {
        let clock = k as f64*0.1 - if k >= 150 { 1.0 } else { 0.0 };
        hal.now_ns = (clock*1e9).round() as u64;
        hal.mag[0] = 1; hal.gyro[0] = 1;
        let now = hal.now_ns; f.step(&mut hal, now);
        assert!(f.time() >= last, "the time ran back from {last} s to {} s at tick {k}", f.time());
        last = f.time();
    }
}

/// The software starts at the clock's 100 s (a reset) and the clock steps back by a second: the time stays a few seconds, not centuries.
#[test]
fn a_clock_step_back_past_the_start_does_not_wrap() {
    let (p, mut hal, mut f) = (params(Mode::Detumble, false), Stub::default(), Fsw::new());
    f.init(ABI_VERSION, &blob(&p), 100_000_000_000).expect("init");
    for k in 0..50u64 {
        let clock = 100.0 + k as f64*0.1 - if k >= 5 { 1.0 } else { 0.0 };
        hal.now_ns = (clock*1e9).round() as u64;
        hal.mag[0] = 1; hal.gyro[0] = 1;
        let now = hal.now_ns; f.step(&mut hal, now);
        assert!(f.time() < 3600.0, "the time reads {} s at tick {k} (a wrap of the unsigned difference)", f.time());
    }
}

// ---- F2: a gyro that has gone quiet ----

/// 60 s of detumble at a steady rate; the gyro goes quiet at 10 s (`silent`: it stops answering; else it answers with its last
/// sample, flagged ok). Returns the faults word and the estimated rate's norm [rad/s] at the end of each second after 10 s.
fn quiet_gyro(silent: bool) -> Vec<(u16, f64)> {
    let (p, mut hal, mut f) = (params(Mode::Detumble, false), Stub::default(), Fsw::new());
    f.init(ABI_VERSION, &blob(&p), 0).expect("init");
    let mut out = vec![];
    for k in 0..600u64 {
        let (b, w) = ([2e-5, -1e-5, 3e-5], [0.05, -0.02, 0.1]);
        hal.now_ns = k*100_000_000;
        hal.mag[0] = 1; for i in 0..3 { put16(&mut hal.mag[1 + 2*i..], b[i]/MAG_LSB_T); }
        if k < 100 {
            // a gyro with noise in its counts, so that a frozen one is told from a steady one
            hal.gyro[0] = 1; for i in 0..3 { put32(&mut hal.gyro[1 + 4*i..], w[i]/GYRO_LSB + ((k*7 + i as u64*3) % 5) as f64); }
        } else {
            hal.gyro_silent = silent;
        }
        let now = hal.now_ns; f.step(&mut hal, now);
        if k >= 100 && k % 10 == 9 {
            let mut d = [0.0f64; 13];
            f.debug(&mut d);
            out.push((f.peek().unwrap().faults, norm3(&[d[10], d[11], d[12]])));
        }
    }
    out
}

fn gyro_quiet_is_flagged_and_dropped(silent: bool) {
    let seen = quiet_gyro(silent);
    let flagged = seen.iter().position(|(fl, _)| fl & adcs_fsw::fsw::FAULT_GYRO_QUIET != 0).map(|s| s as f64 + 1.0);
    assert!(flagged.map(|s| s <= STALE_LIMIT_S).unwrap_or(false),
        "the gyro stale fault is raised {} s after the gyro went quiet (the limit is {STALE_LIMIT_S} s)", flagged.map(|s| s.to_string()).unwrap_or("never within 50".into()));
    let w = seen[(STALE_LIMIT_S as usize - 1).min(seen.len() - 1)].1;
    assert!(w < 0.1*0.114, "{STALE_LIMIT_S} s after, the controllers are still fed a rate of {w} rad/s from the quiet gyro");
}

/// A gyro that stops answering (its transfer fails): the driver keeps its last rate; it must be flagged and the rate dropped.
#[test]
fn a_gyro_that_stops_answering_is_flagged_and_its_rate_dropped() { gyro_quiet_is_flagged_and_dropped(true); }

/// A gyro that answers with its last sample, flagged ok (the engine's gyro_fail): the same samples for ten seconds are no rate.
#[test]
fn a_gyro_frozen_on_its_last_sample_is_flagged_and_its_rate_dropped() { gyro_quiet_is_flagged_and_dropped(false); }

// ---- F5: the detumble exit's rate source (DR-gdn-003) ----

/// 40 s of detumble with the exit at 0.01 rad/s held for 5 s, then the next state. The field turns in the body at `spin` rad/s
/// about z (a tumbling body: the B-dot rate is then `spin` times the field's sine to z). The gyro reads `w_gyro` with counts
/// that change, as a gyro's do; `gyro` says whether the product has one and `fail_at` the second at which it stops answering.
/// Returns the second the mode left detumble (None: it did not).
fn detumble_exit(gyro: bool, w_gyro: f64, spin: f64, fail_at: Option<u64>) -> Option<u64> {
    let mut p = params(Mode::Detumble, false);
    p.has_gyro = gyro as u8; p.auto_next = Mode::NadirMtq as u8; p.detumble_exit = 0.01; p.detumble_hold_s = 5.0;
    let (mut hal, mut f) = (Stub::default(), Fsw::new());
    f.init(ABI_VERSION, &blob(&p), 0).expect("init");
    for k in 0..400u64 {
        let t = k as f64*0.1;
        let b0 = [2e-5f64, -1e-5, 3e-5];
        let (c, s) = ((spin*t).cos(), (spin*t).sin());
        let b = [b0[0]*c + b0[1]*s, -b0[0]*s + b0[1]*c, b0[2]];
        hal.now_ns = k*100_000_000;
        hal.mag[0] = 1; for i in 0..3 { put16(&mut hal.mag[1 + 2*i..], b[i]/MAG_LSB_T); }
        if gyro {
            hal.gyro[0] = 1; for i in 0..3 { put32(&mut hal.gyro[1 + 4*i..], w_gyro/GYRO_LSB + ((k*7 + i as u64*3) % 5) as f64); }
            if let Some(s) = fail_at { hal.gyro_silent = k >= s*10; }
        }
        let now = hal.now_ns; f.step(&mut hal, now);
        if f.peek().unwrap().mode != Mode::Detumble as u16 { return Some(k/10); }
    }
    None
}

/// A healthy gyro: the exit is judged on its rate, whatever the field does.
#[test]
fn a_healthy_gyro_leaves_detumble_on_its_rate() {
    assert!(detumble_exit(true, 0.0, 0.0, None).is_some(), "a still body with a healthy gyro never left detumble");
    assert!(detumble_exit(true, 0.1, 0.0, None).is_none(), "a gyro reading 0.1 rad/s let the body leave detumble");
    assert!(detumble_exit(true, 0.0, 0.1, None).is_some(), "the field's turn kept a body the healthy gyro reads still in detumble: the gyro judges");
}

/// No gyro: the exit is judged on the B-dot rate estimate, so a turning field keeps the body in detumble, a still one lets it go.
#[test]
fn without_a_gyro_a_tumbling_body_does_not_leave_detumble() {
    assert!(detumble_exit(false, 0.0, 0.0, None).is_some(), "a still field with no gyro never let the body go");
    assert!(detumble_exit(false, 0.0, 0.1, None).is_none(), "a body whose field turns at 0.1 rad/s left detumble after the hold, with no gyro");
}

/// A gyro that stops answering during detumble reading a still body while the field turns: the exit moves to the B-dot rate.
#[test]
fn a_gyro_failing_in_detumble_does_not_let_a_tumbling_body_go() {
    assert!(detumble_exit(true, 0.0, 0.1, Some(3)).is_none(), "a gyro that failed at 3 s let the tumbling body leave detumble");
}
