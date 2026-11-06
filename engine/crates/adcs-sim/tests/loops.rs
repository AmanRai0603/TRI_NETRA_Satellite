//! TS3 (docs/S11_INVENTORY.md; G6, DR-ctl-001): the fine-pointing loop checked against its linear design. A harness over the
//! engine with the disturbances off (no environment torque, no residual dipole, no sensor noise or bias): a 0.5 deg step in the
//! attitude, flown by the flight software through the plant, and the loop's frequency sweep, both read against the blob's gains
//! and against the bandwidth the tuning states (fsw_tune_rw_bandwidth, rw_damping).
//!
//! The loop acts on the quaternion's vector part, which is half the angle, so the angle's stiffness is Kp/2, Ki/2 (the gains
//! multiply the vector part) and Kd (the rate). The design places the loop at the bandwidth wn with the damping z:
//! J s^3 + 2 z J wn s^2 + J wn^2 s + 0.15 J wn^3 on the angle (Wie, Weiss and Arapostathis 1989). The test fails when the
//! blob's gains do not give that: the step's rise and settling against the design's within 10 %, the bandwidth within 10 %.
//!
//! The model of the flown loop is the sampled one (the control period dt, the rate's first-order low-pass of rate_lpf_s, the
//! command held over the period), built from the blob's gains only.
use adcs_fsw_abi::Impl;
use adcs_sim::{config::Config, metrics, run};
use std::path::PathBuf;

fn root() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../matlab_sils") }

const STEP_DEG: f64 = 0.5;

/// The loop's parameters as the blob gives them, for the axis of the largest inertia.
struct Loop { j: f64, kp: f64, kd: f64, ki: f64, dt: f64, lpf: f64, wn: f64, z: f64 }

/// The step run: fine nadir hold of the imaging 3U with the reaction wheels, a 0.5 deg error to start, no disturbance and no
/// sensor error. Returns the loop's parameters and the error angle [deg] each record step.
fn step_run() -> (Loop, Vec<(f64, f64)>) {
    let r = root();
    let ov = [("engine.duration_s".to_string(), "240".to_string()), ("initial.attitude.angle_deg".to_string(), STEP_DEG.to_string())];
    let mut c = Config::build(&r, "fine_hold_img", &r.join("cases/ais_img_3u.csv"), 3, &ov).unwrap();
    c.env_on = [false; 4];
    c.m_res = [0.0; 3];
    c.dev.gyro.arw = 0.0; c.dev.gyro.rrw = 0.0; c.dev.gyro.bias_sigma = 0.0; c.dev.gyro.sf_sigma = 0.0; c.dev.gyro.misalign = 0.0;
    c.dev.st.noise_cross = 0.0; c.dev.st.noise_roll = 0.0; c.dev.st.bias_sigma = 0.0; c.dev.st.misalign_sigma = 0.0; c.dev.st.model = 0;
    let rec = run::run(&c, &run::Opts { fsw: Impl::Rust, quiet: true, realtime: false, oils: None }).unwrap();
    let d = metrics::derive(&c, &rec);
    let e: Vec<(f64, f64)> = rec.rows.iter().zip(&d.ape_3ax).map(|(row, a)| (row.t, *a)).filter(|x| x.1.is_finite()).collect();
    let ax = (0..3).max_by(|&a, &b| c.inertia[a][a].partial_cmp(&c.inertia[b][b]).unwrap()).unwrap();
    let f = c.scenario.get("fsw").cloned().unwrap_or_default();
    let l = Loop { j: c.inertia[ax][ax], kp: c.params.rw_Kp[ax], kd: c.params.rw_Kd[ax], ki: c.params.rw_Ki[ax], dt: c.params.rw_dt, lpf: c.params.rate_lpf_s,
                   wn: c.rw_bandwidth, z: f.get("rw_damping").and_then(|x| x.as_f64()).expect("the scenario states the damping") };
    (l, e)
}

/// The sampled step response of the angle error [deg] from `STEP_DEG`, for the angle's stiffness `kth` [N m/rad], integral gain
/// `kti` [N m/(rad s)], the rate gain `kd` and the inertia j: the plant held over the period, the rate through the low-pass.
fn model_step(l: &Loop, kth: f64, kd: f64, kti: f64, secs: f64) -> Vec<(f64, f64)> {
    let (dt, a) = (l.dt, l.dt/(l.lpf + l.dt));
    let (mut th, mut w, mut wf, mut i) = (STEP_DEG, 0.0f64, 0.0f64, 0.0f64);   // degrees and degrees per second
    let mut out = vec![];
    for k in 0..(secs/dt) as usize {
        out.push((k as f64*dt, th.abs()));
        wf += a*(w - wf);
        i += th*dt;
        let u = -(kth*th + kd*wf + kti*i)/l.j;                                    // angular acceleration [deg/s^2]
        th += dt*w + 0.5*dt*dt*u;
        w += dt*u;
    }
    out
}

/// The time the error first falls to `frac` of its start, and the time after which it stays within 2 % (the settling).
fn rise_settle(e: &[(f64, f64)]) -> (f64, f64) {
    let e0 = STEP_DEG;
    let rise = e.iter().find(|x| x.1 <= 0.1*e0).map(|x| x.0).unwrap_or(f64::INFINITY);
    let settle = e.iter().rev().find(|x| x.1 > 0.02*e0).map(|x| x.0).unwrap_or(0.0);
    (rise, settle)
}

/// The design's loop on the angle: stiffness J wn^2, rate gain 2 z J wn, integral gain 0.15 J wn^3.
fn design(l: &Loop) -> (f64, f64, f64) { (l.j*l.wn*l.wn, 2.0*l.z*l.j*l.wn, 0.15*l.j*l.wn.powi(3)) }

#[test]
fn the_step_response_is_the_designs() {
    let (l, e) = step_run();
    assert!(e[0].1 > 0.9*STEP_DEG && e[0].1 < 1.1*STEP_DEG, "the run starts with an error of {} deg, not the step of {STEP_DEG} deg", e[0].1);
    let (rise, settle) = rise_settle(&e);
    // the blob's own poles: the vector part is half the angle, so the stiffness is Kp/2
    let blob = model_step(&l, l.kp/2.0, l.kd, l.ki/2.0, 240.0);
    let (rb, sb) = rise_settle(&blob);
    // the tuning's: the loop placed at the bandwidth wn
    let (kth, kd, kti) = design(&l);
    let want = model_step(&l, kth, kd, kti, 240.0);
    let (rw, sw) = rise_settle(&want);
    println!("step {STEP_DEG} deg: flown rise {rise:.2} s settle {settle:.2} s; blob's poles rise {rb:.2} settle {sb:.2}; design (wn {} z {}) rise {rw:.2} settle {sw:.2}", l.wn, l.z);
    // the harness itself: the flown loop follows the sampled model of its own gains
    assert!((rise/rb - 1.0).abs() < 0.10, "rise {rise:.2} s against the blob's poles' {rb:.2} s");
    assert!((settle/sb - 1.0).abs() < 0.10, "2 % settling {settle:.2} s against the blob's poles' {sb:.2} s");
    // the design: the bandwidth the tuning states, read from the time scale of the step (every time scales as 1/wn)
    let (bw_rise, bw_settle) = (l.wn*rw/rise, l.wn*sw/settle);
    assert!((bw_rise/l.wn - 1.0).abs() < 0.10, "the step's rise gives a bandwidth of {bw_rise:.3} rad/s ({:.2} x the tuning's {} rad/s)", bw_rise/l.wn, l.wn);
    assert!((bw_settle/l.wn - 1.0).abs() < 0.10, "the step's 2 % settling gives a bandwidth of {bw_settle:.3} rad/s ({:.2} x the tuning's {} rad/s)", bw_settle/l.wn, l.wn);
}

#[derive(Clone, Copy)]
struct C(f64, f64);
impl C {
    fn add(self, o: C) -> C { C(self.0 + o.0, self.1 + o.1) }
    fn sub(self, o: C) -> C { C(self.0 - o.0, self.1 - o.1) }
    fn mul(self, o: C) -> C { C(self.0*o.0 - self.1*o.1, self.0*o.1 + self.1*o.0) }
    fn div(self, o: C) -> C { let d = o.0*o.0 + o.1*o.1; C((self.0*o.0 + self.1*o.1)/d, (self.1*o.0 - self.0*o.1)/d) }
    fn sc(self, k: f64) -> C { C(self.0*k, self.1*k) }
    fn abs(self) -> f64 { self.0.hypot(self.1) }
}

/// The loop gain of the sampled loop at the angular frequency w [rad/s]: the plant held over the period (angle and rate
/// outputs), the proportional and integral terms on the angle (the vector part: half of it), the derivative on the low-passed rate.
fn loop_gain(l: &Loop, w: f64) -> C {
    let (dt, a) = (l.dt, l.dt/(l.lpf + l.dt));
    let z = C((w*dt).cos(), (w*dt).sin());
    let one = C(1.0, 0.0);
    let zm1 = z.sub(one);
    let g_th = z.add(one).div(zm1.mul(zm1)).sc(dt*dt/(2.0*l.j));
    let g_w = one.div(zm1).sc(dt/l.j);
    let c_th = one.sc(l.kp/2.0).add(z.div(zm1).sc(l.ki/2.0*dt));
    let c_w = z.div(z.sub(one.sc(1.0 - a))).sc(l.kd*a);
    c_th.mul(g_th).add(c_w.mul(g_w))
}

/// Gain margin [dB] (the least over the phase crossings) and phase margin [deg] (the least over the gain crossings) of the sweep.
fn margins(l: &Loop) -> (f64, f64, f64) {
    let n = 60000;
    let wmax = std::f64::consts::PI/l.dt*0.999;
    let phase = |c: C| { let p = c.1.atan2(c.0).to_degrees(); if p > 0.0 { p - 360.0 } else { p } };
    let (mut gm, mut pm, mut wc_last) = (f64::INFINITY, f64::INFINITY, 0.0);
    let mut prev: Option<(f64, C)> = None;
    for k in 0..n {
        let w = 1e-3*(wmax/1e-3).powf(k as f64/(n - 1) as f64);
        let c = loop_gain(l, w);
        if let Some((_, p)) = prev {
            if (c.abs() - 1.0)*(p.abs() - 1.0) < 0.0 { pm = pm.min(180.0 + phase(c)); wc_last = w; }
            if (phase(c) + 180.0)*(phase(p) + 180.0) < 0.0 { gm = gm.min((20.0*c.abs().log10()).abs()); }
        }
        prev = Some((w, c));
    }
    (gm, pm, wc_last)
}

/// The closed loop's -3 dB bandwidth [rad/s] of the sampled loop, with the stiffness `kth`, rate gain `kd` and integral `kti`
/// given on the angle (the sweep's reference to angle, T = L/(1+L)).
fn closed_bw(l: &Loop, kth: f64, kd: f64, kti: f64) -> f64 {
    let m = Loop { j: l.j, kp: 2.0*kth, kd, ki: 2.0*kti, dt: l.dt, lpf: l.lpf, wn: l.wn, z: l.z };
    let wmax = std::f64::consts::PI/l.dt*0.999;
    let mut w = 1e-3;
    while w < wmax {
        let lg = loop_gain(&m, w);
        let t = lg.div(C(1.0, 0.0).add(lg)).abs();
        if w > 0.05 && t < 0.7071 { return w; }
        w *= 1.002;
    }
    wmax
}

#[test]
fn the_sweep_has_the_margins_and_the_bandwidth() {
    let (l, _) = step_run();
    let (gm, pm, wc) = margins(&l);
    let (kth, kd, kti) = design(&l);
    let (bw, bw_design) = (closed_bw(&l, l.kp/2.0, l.kd, l.ki/2.0), closed_bw(&l, kth, kd, kti));
    println!("sweep: gain margin {gm:.1} dB, phase margin {pm:.1} deg at {wc:.2} rad/s; closed-loop -3 dB {bw:.3} rad/s, the design's {bw_design:.3} rad/s");
    assert!(gm >= 6.0, "gain margin {gm:.1} dB, held at 6 dB");
    assert!(pm >= 30.0, "phase margin {pm:.1} deg, held at 30 deg");
    assert!((bw/bw_design - 1.0).abs() < 0.10, "the loop's -3 dB bandwidth {bw:.3} rad/s is {:.2} x the design's {bw_design:.3} rad/s (the tuning's {} rad/s)", bw/bw_design, l.wn);
}
