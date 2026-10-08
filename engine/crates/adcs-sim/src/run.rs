//! One closed-loop SILS run (asils.run): the tick order of spec §9.3 --
//! environment, sensors -> device emulators -> bytes, flight software step,
//! bytes -> actuator commands, actuators, torques, plant, recorder.
use crate::error::Error;
use crate::config::Config;
use adcs_fsw::guid::{guidance, Guid};
use adcs_fsw_abi::{Bus, Fsw, Impl};
use adcs_sim_core::actuators::{coil_torque, Mex, Mtq, Rcs};
use adcs_sim_core::comp::star_tracker as stc;
use adcs_sim_core::emu::{self, proto, Commands};
use adcs_sim_core::la::*;
use adcs_sim_core::orbit::{self, Orbit, OrbitCfg};
use adcs_sim_core::plant::{self, Body, Geometry, State};
use adcs_sim_core::rng::{stream_id, Rng};
use adcs_sim_core::sensors::*;
use adcs_sim_core::torques::{self, Facets};
use adcs_sim_core::{ephem, field, time, NC, NG, NR};

/// The environment the attitude loop reads at an env tick.
pub struct Env { pub b_eci: V3, pub sun_rel: V3, pub moon_rel: V3, pub nu: f64, pub v_rel: V3, pub rho: f64, pub p_srp: f64 }

/// The truth orbit and environment. `Pop`: the Rust port of POP stepped as asils.orbit
/// does (bit-identical to the MATLAB twin's orbit, Sun, Moon, density, frame). `Fast`:
/// the analytic models of adcs-sim-core (J2-J6, Montenbruck-Gill, exponential density).
pub enum Truth { Pop(Box<adcs_pop::accel::InLoop>), Fast(Orbit, f64) }

impl Truth {
    pub fn new(c: &Config) -> Result<(Truth, f64), Error> {
        if c.orbit_model == "pop" {
            if let Some(e) = crate::pop_kernel_missing() { return Err(e); }
            use adcs_pop::accel::{sso_initial, Forces, InLoop, Sc, World};
            let cr = adcs_pop::gen::forcemodel::sils_cr(c.refl); // env_force_model: Cr = 1 + the reflectivity
            let w = World::new(c.epoch_utc, adcs_pop::frames::Build::Gmst, adcs_pop::frames::FrameOpt::default(),
                adcs_pop::gravity::Field::default_field(), Forces::sils(cr),
                Sc { mass: c.mass_kg, aref: c.aref_m2, cd: Some(c.cd), cr: Some(cr), r_bi: adcs_pop::la::I3, srp_facets: vec![], drag_facets: None },
                Some(adcs_pop::spaceweather::ManualIndices { f107: c.f107, f107a: Some(c.f107a), kp: Some(adcs_pop::atmos::dtm2020::KpIn::Scalar(c.kp)), ap: Some(c.ap), ap3: None }),
                Some(crate::pop_kernel()))?;
            let (r0, v0, raan) = sso_initial(&w, c.alt_km, c.ecc, c.inc_deg, c.ltan_h, 0.0, c.u0_deg)?;
            Ok((Truth::Pop(Box::new(InLoop::new(w, c.orbit_step_s, r0, v0)?)), raan))
        } else {
            // env_orbit_start: the RAAN from the LTAN and the Sun at the epoch, a, i and u0 (gen::orbitstart)
            let (raan, a, inc, u0) = crate::gen::orbitstart::fast_orbit_start(orbit::start_sun(c.jd0), c.ltan_h, c.alt_km, c.inc_deg, c.u0_deg);
            let (r0, v0) = orbit::coe2rv(a, c.ecc, inc, raan, 0.0, u0);
            let o = Orbit::new(OrbitCfg { jd0_utc: c.jd0, step_s: c.orbit_step_s, zonal_max: c.zonal_max, third_body: c.third_body, drag: c.drag, srp: c.srp,
                mass_kg: c.mass_kg, area_m2: c.aref_m2, cd: c.cd, cr: adcs_pop::gen::forcemodel::sils_cr(c.refl), density_scale: c.density_scale }, r0, v0);
            Ok((Truth::Fast(o, c.jd0), raan))
        }
    }
    pub fn state(&mut self, t: f64) -> Result<(V3, V3), Error> {
        match self { Truth::Pop(o) => o.state(t).map_err(Error::from), Truth::Fast(o, _) => Ok(o.state(t)) }
    }
    /// asils.run's env refresh: field at the POP position (op.geodetic), Sun, shadow,
    /// atmosphere-relative velocity, density and SRP pressure.
    pub fn env(&self, t: f64, jd0: f64, r: &V3, v: &V3, gh: &field::Gh, nmax: usize) -> Env {
        match self {
            Truth::Pop(o) => {
                let x = o.context(t);
                // the field in J2000: precession to the equator of date, then POP's Earth rotation
                let cp = mm(&x.c, &time::precession(jd0 + t/86400.0));
                let re = mv(&cp, r);
                let (lat, lon, h) = adcs_pop::geodetic::geodetic(&re);
                let w = o.omega_e;
                Env { b_eci: field::eci_at(lat, lon, h, &cp, gh, nmax), sun_rel: sub(&x.sun_eci, r), moon_rel: sub(&x.moon_eci, r), nu: ephem::shadow(r, &x.sun_eci),
                      v_rel: [v[0] + w*r[1], v[1] - w*r[0], v[2]], rho: x.rho, p_srp: x.p_srp }
            }
            Truth::Fast(o, jd0) => {
                let xc = o.context(t);
                let cm = time::eci2ecef(jd0 + t/86400.0);
                Env { b_eci: field::eci(&mv(&cm, r), &cm, gh, nmax), sun_rel: sub(&xc.sun, r), moon_rel: sub(&xc.moon, r), nu: ephem::shadow(r, &xc.sun),
                      v_rel: orbit::corotating(r, v), rho: xc.rho, p_srp: xc.p_srp }
            }
        }
    }
    pub fn set_attitude(&mut self, r_bi: M3) { if let Truth::Pop(o) = self { o.set_attitude(r_bi); } }
}

/// One recorded sample (the channels of asils.rec.write).
#[derive(Clone, Debug, Default)]
pub struct Row {
    pub t: f64, pub q: Q, pub w: V3, pub q_est: Option<Q>, pub w_meas: V3, pub tau_req: V3,
    pub h_w: [f64; NR], pub tau_rw: V3, pub cmd_r: [f64; NR], pub n_failed: u32, pub delta: [f64; NG],
    pub tau_rcs: V3, pub prop_kg: f64, pub m: V3, pub tau_dist: [V3; 4], pub tau_mtq: V3,
    pub mode: u8, pub p_mtq: f64, pub p_rw: f64, pub p_rcs: f64,
    pub r: V3, pub v: V3, pub rho: f64, pub nu: f64, pub b: V3, pub b_meas: V3,
    pub sun_ok: bool, pub st_ok: bool, pub ad_ok: bool, pub sun_eci: V3, pub sun_body: V3,
}

pub struct Record { pub rows: Vec<Row>, pub nr: usize, pub ng: usize, pub raan_rad: f64, pub mode_log: Vec<(f64, String)>, pub wall_s: f64, pub fsw_build: String, pub fsw_impl: Impl,
    pub oils: Option<OilsStats> }

pub struct Opts { pub fsw: Impl, pub quiet: bool, /// pace ticks to wall-clock time (OILS / HILS with a real OBC)
    pub realtime: bool,
    /// soft OILS: the OBC's execution and bus time delay each command inside its control period
    pub oils: Option<OilsModel> }

/// Soft-OILS timing model (docs/SOFT_OILS.md). Per tick the command reaches the actuators
///   latency = sensor reads on the buses + OBC execution + CAN command frames
/// after the sample; until then the previous command holds. The OBC execution is the step's
/// exact instruction count on QEMU (-icount) x CPI / core clock, or the host-measured time on a
/// process OBC. A latency of a whole period or more is an overrun: the command lands a tick late.
///
/// The deadline is judged on the worst case, not the nominal one: the execution at `cpi_max`
/// (wait states, cache misses) plus `isr_s` of interrupts that may preempt the step, plus the
/// bus time; it must land within `deadline_frac` of the control period. The trajectory flies the
/// nominal latency; the worst case only judges.
#[derive(Clone, Debug)]
pub struct OilsModel { pub cpu_hz: f64, pub cpi: f64, pub i2c_hz: f64, pub spi_hz: f64, pub can_bps: f64,
    /// a fixed command latency [s] instead of the OBC model (latency studies; works with the in-process builds)
    pub fixed_s: Option<f64>,
    /// the worst cycles per instruction the deadline is judged at
    pub cpi_max: f64,
    /// the interrupt time that may preempt one step [s]
    pub isr_s: f64,
    /// the share of the control period the command must land within
    pub deadline_frac: f64 }
impl Default for OilsModel {
    /// A Cortex-M4F OBC at 168 MHz (STM32F4 class, flash accelerator on: CPI ~1.25, 2.0 with the
    /// accelerator missing), I2C fast mode, SPI 1 MHz, CAN 1 Mbit/s; 50 us of interrupts a step;
    /// the command within half the period.
    fn default() -> Self {
        OilsModel { cpu_hz: 168e6, cpi: 1.25, i2c_hz: 400e3, spi_hz: 1e6, can_bps: 1e6, fixed_s: None, cpi_max: 2.0, isr_s: 50e-6, deadline_frac: 0.5 }
    }
}

#[derive(Clone, Debug, Default)]
pub struct OilsStats {
    pub model: Option<OilsModel>,
    pub ticks: u64, pub overruns: u64,
    pub exec_s: Vec<f64>, pub io_s: Vec<f64>, pub lat_s: Vec<f64>, pub insn: Vec<f64>,
    /// the worst-case latency of each tick (execution at cpi_max, interrupts, buses)
    pub worst_s: Vec<f64>,
}
impl OilsStats {
    fn pct(v: &[f64], p: f64) -> f64 {
        if v.is_empty() { return f64::NAN; }
        let mut s = v.to_vec(); s.sort_by(|a, b| a.partial_cmp(b).unwrap());
        s[((p/100.0*(s.len() - 1) as f64).round() as usize).min(s.len() - 1)]
    }
    fn mean(v: &[f64]) -> f64 { if v.is_empty() { f64::NAN } else { v.iter().sum::<f64>()/v.len() as f64 } }
    fn max(v: &[f64]) -> f64 { v.iter().cloned().fold(f64::NAN, f64::max) }
    pub fn json(&self, dt: f64) -> serde_json::Value {
        let s = |v: &[f64]| serde_json::json!({"mean": Self::mean(v), "p99": Self::pct(v, 99.0), "max": Self::max(v)});
        let m = self.model.clone().unwrap_or_default();
        let (deadline, worst) = (m.deadline_frac*dt, Self::max(&self.worst_s));
        serde_json::json!({
            "model": {"cpu_hz": m.cpu_hz, "cpi": m.cpi, "i2c_hz": m.i2c_hz, "spi_hz": m.spi_hz, "can_bps": m.can_bps,
                      "cpi_max": m.cpi_max, "isr_s": m.isr_s, "deadline_frac": m.deadline_frac, "fixed_s": m.fixed_s},
            "ticks": self.ticks, "overruns": self.overruns, "control_period_s": dt,
            "exec_s": s(&self.exec_s), "bus_s": s(&self.io_s), "latency_s": s(&self.lat_s), "worst_latency_s": s(&self.worst_s),
            "instructions": if self.insn.is_empty() { serde_json::Value::Null } else { s(&self.insn) },
            "cpu_load_mean": Self::mean(&self.exec_s)/dt, "cpu_load_max": Self::max(&self.exec_s)/dt,
            "deadline_margin_min_s": dt - Self::max(&self.lat_s),
            "deadline_s": deadline, "worst_case_margin_s": deadline - worst,
        })
    }

    /// The two soft-OILS verdicts, judged like any metric: no command lands a period late, and
    /// the worst-case latency leaves a margin before the deadline.
    pub fn metrics(&self, dt: f64) -> Vec<serde_json::Value> {
        let m = self.model.clone().unwrap_or_default();
        let margin = m.deadline_frac*dt - Self::max(&self.worst_s);
        let num = |x: f64| if x.is_finite() { serde_json::json!(x) } else { serde_json::Value::Null };
        vec![
            serde_json::json!({"id": "oils_overruns", "kind": "oils_overruns", "value": self.overruns, "unit": "", "req": 0.0,
                               "req_key": "oils", "pass": (self.overruns == 0) as i32}),
            serde_json::json!({"id": "oils_worst_case_margin", "kind": "oils_deadline", "value": num(margin*1e3), "unit": "ms", "req": 0.0,
                               "req_key": "oils", "pass": (margin.is_finite() && margin >= 0.0) as i32}),
        ]
    }
}

fn mode_changes(log: &mut Vec<(f64, String)>, t: f64, m: u8) {
    let name = crate::config::MODES.get(m as usize).copied().unwrap_or("none").to_string();
    if log.last().map(|x| x.1 != name).unwrap_or(true) { log.push((t, name)); }
}

/// The units a product carries, each drawn once per run (its dispersion), in a fixed order so a
/// seed always gives the same units.
struct Units {
    gyro: Option<Gyro>, mag: Mag, sun: Option<Sun>, st: Option<St>, mtq: Mtq, es: Option<Es>, css: Option<Css>,
    mex: Mex, rcs: Option<Rcs>, gps: Gps, tlm: Rng,
    /// the frame buffers of the image star-tracker model (model 2), built once per run
    st_frame: Option<StFrame>,
}

/// The image star-tracker model's buffers: the frame, its scratch copy and two index lists as long as it, and the onboard
/// pair table of the unit's catalogue, each as long as the part makes it (adcs-sim-core does not allocate).
pub struct StFrame { img: Vec<f64>, work: Vec<f64>, ia: Vec<i64>, ib: Vec<i64>, pi: Vec<i64>, pj: Vec<i64>, pa: Vec<f64> }
impl StFrame {
    pub fn new(st: &mut St) -> StFrame {
        let (fov, n) = (st.d.cam.fov, st.d.cam.n);
        let (cr, _) = st.catalogue_mut();
        let np = stc::pair_count(cr, fov);
        let (mut pi, mut pj, mut pa) = (vec![0; np], vec![0; np], vec![0.0; np]);
        stc::pairs(cr, fov, &mut pi, &mut pj, &mut pa, &mut vec![0; np], &mut vec![0; np], &mut vec![0.0; np]);
        StFrame { img: vec![0.0; n*n], work: vec![0.0; n*n], ia: vec![0; n*n], ib: vec![0; n*n], pi, pj, pa }
    }
    pub fn work(&mut self) -> stc::Work<'_> {
        stc::Work { img: &mut self.img, work: &mut self.work, ia: &mut self.ia, ib: &mut self.ib, pi: &mut self.pi, pj: &mut self.pj, pa: &mut self.pa }
    }
}

impl Units {
    fn new(d: &crate::product::Dev, rs: &dyn Fn(&str) -> Rng) -> Units {
        let mut disp = rs("dispersion");
        let gyro = if d.gyro.fitted { Some(Gyro::new(d.gyro, &mut disp, rs("gyro"))) } else { None };
        let mag = Mag::new(d.mag, &mut disp, rs("mag"));
        let sun = if d.sun.fitted { Some(Sun::new(d.sun, &mut disp, rs("sun"))) } else { None };
        let mut st = if d.st.fitted { Some(St::new(d.st, &mut disp, rs("st"))) } else { None };
        let mtq = Mtq::new(d.mtq, &mut disp);
        let es = if d.es.fitted { Some(Es::new(d.es, &mut disp, rs("es"))) } else { None };
        let css = if d.css.fitted { Some(Css::new(d.css, &mut disp, rs("css"))) } else { None };
        let mex = Mex::new(d.mex, &mut disp, rs("mex"));
        let rcs = if d.rcs.fitted { Some(Rcs::new(d.rcs, &mut disp)) } else { None };
        let st_frame = st.as_mut().filter(|s| s.d.model == 2).map(StFrame::new);
        Units { gyro, mag, sun, st, mtq, es, css, mex, rcs, gps: Gps::new(d.gps, rs("gps")), tlm: rs("telemetry"), st_frame }
    }

    /// Inject every fault whose time has come (once each), clear the ones that end, and log both.
    fn faults(&mut self, c: &Config, t: f64, done: &mut [bool], log: &mut Vec<(f64, String)>) -> Result<(), Error> {
        for (i, f) in c.faults.iter().enumerate() {
            // a device back from silence (end_s): it answers again from then on
            if done[i] {
                if let Some(e) = f.end_s {
                    let dead = match f.kind.as_str() { "gps_outage" => &mut self.gps.dead, _ => &mut self.mag.dead };
                    if t >= e && *dead { *dead = false; log.push((t, format!("FAULT cleared: {} {}", f.kind, f.index))); }
                }
                continue;
            }
            if t < f.t_s { continue; }
            done[i] = true;
            let ix = f.index.saturating_sub(1);
            match f.kind.as_str() {
                "rotor_fail" => self.mex.failed[ix] = true,
                "gimbal_stuck" => self.mex.gfailed[ix] = true,
                "st_head_fail" => if let Some(s) = self.st.as_mut() { s.dead[ix] = true },
                "coil_fail" => self.mtq.dead[ix] = true,
                "gyro_bias_step" => if let Some(g) = self.gyro.as_mut() { g.b = add(&g.b, &f.value) },
                "gps_outage" => self.gps.dead = true,
                "rcs_valve_fail" => if let Some(r) = self.rcs.as_mut() { r.failed[ix] = true },
                "mag_fail" => self.mag.dead = true,
                other => return Err(Error::run(format!("unknown fault {other}"))),
            }
            log.push((t, format!("FAULT injected: {} {}", f.kind, f.index)));
        }
        Ok(())
    }
}

/// How often each part of the loop runs, in ticks.
struct Rates { n: u64, every: u64, off: u64, env: u64, st: u64, gps: u64, es: u64 }

impl Rates {
    fn new(c: &Config) -> Rates {
        let (d, dt) = (&c.dev, c.dt);
        let every = ((c.record_dt/dt).round() as u64).max(1);
        Rates {
            n: (c.duration_s/dt).round() as u64, every, off: every/2,
            env: ((c.env_dt_s/dt).round() as u64).max(1),
            st: if d.st.fitted { ((1.0/(d.st.rate_hz*dt)).round() as u64).max(1) } else { 1 },
            gps: ((1.0/dt).round() as u64).max(1),
            es: if d.es.fitted { ((1.0/(d.es.rate_hz*dt)).round() as u64).max(1) } else { 1 },
        }
    }
}

/// The initial attitude and rate the scenario asks for (asils.run initial_), and the rotor momenta.
fn initial_state(c: &Config, r: &V3, v: &V3, gd: &Guid, ir: &mut Rng, nr: usize) -> State {
    let ini = c.scenario.get("initial").cloned().unwrap_or_default();
    let att = ini.get("attitude").cloned().unwrap_or_default();
    let rate = ini.get("rate").cloned().unwrap_or_default();
    let q_nad = guidance(0, r, v, 0.0, gd).q;
    let q0 = match crate::json::s(&att, "kind", "") {
        "random" => { let x = [ir.normal(), ir.normal(), ir.normal(), ir.normal()]; qnorm(&x) }
        k @ ("error_from_target" | "error_from_guidance") => {
            let qr = if k == "error_from_guidance" && c.gd_kind0 >= 0 { guidance(c.gd_kind0, r, v, 0.0, gd).q } else { q_nad };
            let ax = unit(&att.get("axis_body").and_then(crate::json::v3).unwrap_or([1.0, 0.0, 0.0]));
            qnorm(&qmult(&qr, &fromrotvec(&scale(&ax, crate::json::f(&att, "angle_deg", 0.0).to_radians()))))
        }
        _ => q_nad,
    };
    let w0 = match crate::json::s(&rate, "kind", "") {
        "random_direction" => {
            let mag_ = match rate.get("magnitude_deg_s") {
                Some(serde_json::Value::String(s)) => c.case.get(s.trim_start_matches("case:")),
                Some(x) => x.as_f64().unwrap_or(0.0), None => 0.0,
            };
            scale(&unit(&ir.normal3()), mag_.to_radians())
        }
        "lvlh" => mv(&dcm(&q0), &scale(&cross(r, v), 1.0/dot(r, r))),
        "guidance" => {
            // the reference rate is in the reference frame: the body at q0 turns with it at dcm(q_e) w_ref
            let mut w = if c.gd_kind0 >= 0 {
                let g = guidance(c.gd_kind0, r, v, 0.0, gd);
                mv(&dcm(&qmult(&qconj(&g.q), &q0)), &g.w)
            } else { [0.0; 3] };
            w = add(&w, &scale(&unit(&ir.normal3()), crate::json::f(&rate, "extra_deg_s", 0.0).to_radians()));
            w
        }
        _ => rate.get("value_deg_s").and_then(crate::json::v3).map(|x| scale(&x, std::f64::consts::PI/180.0)).unwrap_or([0.0; 3]),
    };
    let mut x = State { q: q0, w: w0, ..Default::default() };
    let h0 = crate::json::f(&ini, "wheel_momentum_Nms", f64::NAN);
    for i in 0..nr { x.h[i] = if h0.is_nan() { c.h_t_rot[i] } else { h0 }; }
    x
}

/// What the sensors measured this tick (the truth's view the recorder keeps).
struct Sensed { w_meas: V3, b_meas: V3, sun_meas: Option<V3>, st_ok: bool }

/// The geometry a tick's sensors see: body field, Sun, Moon and nadir directions, Earth's half-angle.
struct Sky { b_b: V3, sb: V3, mb: V3, nb: V3, earth_ang: f64 }

/// Every sensor sampled and written onto the bus as its device's bytes (registers, UART frames,
/// CAN telemetry), in the order the units draw their noise.
#[allow(clippy::too_many_arguments)]
fn sense(u: &mut Units, c: &Config, bus: &mut Bus, x: &State, sky: &Sky, m_b: &V3, nu: f64, t: f64, k: u64, rt: &Rates, r: &V3, v: &V3) -> Sensed {
    let (d, dt) = (&c.dev, c.dt);
    bus.now_ns = (t*1e9).round() as u64;
    let w_meas = match u.gyro.as_mut() { Some(g) => g.sample(&x.w, dt), None => x.w };
    bus.gyro = u.gyro.as_ref().map(|_| emu::gyro_resp(true, &w_meas));
    let b_meas = u.mag.sample(&sky.b_b, m_b);
    bus.mag = if d.mag.fitted { Some(emu::mag_regs(true, &b_meas)) } else { None };
    let sun_meas = if let Some(s) = u.sun.as_mut() { s.sample(&sky.sb, nu) } else if let Some(s) = u.css.as_mut() { s.sample(&sky.sb, nu, &sky.nb, sky.earth_ang) } else { None };
    bus.sun = if d.sun.fitted || d.css.fitted { Some(emu::unit_regs(sun_meas.is_some(), &sun_meas.unwrap_or([0.0; 3]))) } else { None };
    let mut st_ok = false;
    if let Some(s) = u.st.as_mut() {
        s.history(t, &x.q);
        if k % rt.st == 0 {
            let mut work = u.st_frame.as_mut().map(|f| f.work());
            let heads = s.sample_with(&x.q, t, &x.w, &sky.sb, &sky.mb, &sky.nb, sky.earth_ang, work.as_mut());
            let mut hv = [(false, [0.0, 0.0, 0.0, 1.0]); 2];
            for h in 0..d.st.nh { if let Some(q) = heads[h] { hv[h] = (true, q); st_ok = true; } }
            let mut buf = [0u8; 64];
            let len = emu::st_frame(&hv[..d.st.nh], &mut buf);
            bus.push_uart(proto::ST_UART, &buf[..len]);
        }
    }
    if let Some(e) = u.es.as_mut() {
        let z = if k % rt.es == 0 { e.sample(&sky.nb) } else { None };
        bus.es = Some(emu::unit_regs(z.is_some(), &z.unwrap_or([0.0; 3])));
    }
    if d.gps.fitted { u.gps.history(t, r, v); }
    if d.gps.fitted && k % rt.gps == 0 && !u.gps.dead {
        // a receiver fixes in ECEF (WGS-84): r_e = C r, v_e = C v - w_E x r_e; the fix it reports
        // now solves for the epoch `latency` ago
        let (te, r, v) = u.gps.delayed(t);
        let (re_, ve_) = Gps::ecef(c.jd0, te, &r, &v);
        let (rg, vg) = u.gps.sample(&re_, &ve_);
        let mut buf = [0u8; 64];
        let len = emu::gps_frame(true, &rg, &vg, &mut buf);
        bus.push_uart(proto::GPS_UART, &buf[..len]);
    }
    // the rotors' telemetry: act_rotor_telemetry's rotor_telemetry, act's stated noises, from the telemetry stream
    let mut gi = [0i64; NR];
    for i in 0..NR { gi[i] = d.mex.gi[i] as i64; }
    let mut g = u.tlm.stream();
    let (hm, dm) = adcs_sim_core::gen::rotortlm::rotor_telemetry(d.mex.n as i64, x.h, x.d, gi, &mut g);
    u.tlm = Rng::from_stream(&g);
    for i in 0..d.mex.n {
        let (id, data) = emu::rotor_tm(i, hm[i], dm[i]);
        bus.push_can(id, data);
    }
    Sensed { w_meas, b_meas, sun_meas, st_ok }
}

/// Soft OILS: when this tick's command reaches the actuators (the OBC's execution time plus the
/// synchronous bus reads and the CAN frames it sent), counted into the statistics.
fn oils_latency(st: &mut OilsStats, m: &OilsModel, fsw: &Fsw, bus: &Bus, dt: f64) -> f64 {
    let (exec, insn) = match fsw.obc_exec() {
        Some((_, Some(n))) => (n*m.cpi/m.cpu_hz, Some(n)),
        Some((s, None)) => (s, None),
        None => (0.0, None),
    };
    // synchronous reads inside the step: I2C (addr + reg, restart, 7 bytes), SPI gyro (13 bytes)
    let i2c = [bus.mag.is_some(), bus.sun.is_some(), bus.es.is_some()].iter().filter(|x| **x).count() as f64;
    let io = i2c*10.0*9.0/m.i2c_hz + if bus.gyro.is_some() { 13.0*8.0/m.spi_hz } else { 0.0 }
        + bus.can_tx.len() as f64*130.0/m.can_bps;
    let lat = match m.fixed_s { Some(f) => f, None => io + exec };
    let worst = match m.fixed_s { Some(f) => f, None => io + exec*m.cpi_max/m.cpi + m.isr_s };
    st.ticks += 1; st.exec_s.push(exec); st.io_s.push(io); st.lat_s.push(lat); st.worst_s.push(worst);
    if let Some(n) = insn { st.insn.push(n); }
    if lat >= dt { st.overruns += 1; }
    lat
}

/// What the actuators deliver this tick, and the power they draw.
#[derive(Default)]
struct Actuation { m_b: V3, m_coil: V3, hdot: [f64; NR], gdot: [f64; NG], tau_rcs: V3, p_mtq: f64, p_mex: f64, p_rcs: f64 }

/// The commands applied to the coils, the momentum devices and the thrusters (propellant counted;
/// an empty tank fails every valve). Unfitted devices keep what they last delivered.
fn actuate(u: &mut Units, c: &Config, cmd: &Commands, x: &State, a: &mut Actuation, prop: &mut f64) {
    let d = &c.dev;
    let (nr, nc) = (d.mex.n, if d.rcs.fitted { d.rcs.nc } else { 0 });
    // the coils' dipole averaged over the tick (the torque) and at its end (what the magnetometer sees next)
    if d.mtq.fitted { let (m, me, pw) = u.mtq.apply(&cmd.m_body, c.dt); a.m_b = m; a.m_coil = me; a.p_mtq = pw; }
    if nr > 0 { let (hd, gd_, pw) = u.mex.apply(&cmd.cmd_r, &cmd.cmd_g, &x.h, c.dt); a.hdot = hd; a.gdot = gd_; a.p_mex = pw; }
    if let Some(rc_) = u.rcs.as_mut() {
        let mut duty = [0.0; NC];
        duty[..nc].copy_from_slice(&cmd.duty[..nc]);
        let (tq, mdot, pw) = rc_.apply(&duty, c.dt);
        a.tau_rcs = tq; a.p_rcs = pw;
        *prop = rc_.spend(*prop, mdot, c.dt);   // act's l3_rcs_row_08: an empty tank fails every valve
    }
}

/// The actuation that holds until this tick's command lands (soft OILS).
#[derive(Default)]
struct Held { m: V3, hdot: [f64; NR], gdot: [f64; NG], rcs: V3 }

/// The rest of what a recorded row holds that the tick computed.
struct Tick { t: f64, r: V3, v: V3, rho: f64, nu: f64, sun_rel: V3, mode: u8, ad_ok: bool, pavg: [f64; 3], faults: u16, parts: [V3; 4], tau_mtq: V3, prop: f64 }

/// One recorded row: the truth, the flight software's view (debug vector), the actuation, the environment.
#[allow(clippy::too_many_arguments)]
fn row(k: &Tick, x: &State, geo: &Geometry, nr: usize, a: &Actuation, cmd: &Commands, dbg: &[f64; adcs_fsw_abi::DEBUG_LEN], z: &Sensed, sky: &Sky) -> Row {
    let ax = geo.axes(&x.d);
    let mut tau_rw = [0.0; 3];
    for i in 0..nr { for j in 0..3 { tau_rw[j] -= ax[i][j]*a.hdot[i]; } }
    Row {
        t: k.t, q: x.q, w: x.w, q_est: if k.ad_ok { Some([dbg[3], dbg[4], dbg[5], dbg[6]]) } else { None }, w_meas: z.w_meas,
        tau_req: [dbg[17], dbg[18], dbg[19]], h_w: x.h, tau_rw, cmd_r: cmd.cmd_r, n_failed: k.faults.count_ones(), delta: x.d,
        tau_rcs: a.tau_rcs, prop_kg: k.prop, m: a.m_b, tau_dist: k.parts, tau_mtq: k.tau_mtq, mode: k.mode, p_mtq: k.pavg[0], p_rw: k.pavg[1], p_rcs: k.pavg[2],
        r: k.r, v: k.v, rho: k.rho, nu: k.nu, b: sky.b_b, b_meas: z.b_meas, sun_ok: z.sun_meas.is_some(), st_ok: z.st_ok, ad_ok: k.ad_ok, sun_eci: unit(&k.sun_rel), sun_body: sky.sb,
    }
}

/// The plant over one tick. With soft OILS (`lat` given) the previous actuation holds for the
/// command latency, then this tick's command acts (a whole period or more: it lands next tick).
#[allow(clippy::too_many_arguments)]
fn step_plant(x: &State, dt: f64, body: &Body, tau_d: &V3, tau_mtq: &V3, b_b: &V3, a: &Actuation, lat: Option<f64>, held: &mut Held) -> State {
    let tau_ext = add(&add(tau_d, tau_mtq), &a.tau_rcs);
    let Some(lat) = lat else { return plant::step(x, dt, body, &tau_ext, &a.hdot, &a.gdot) };
    let held_ext = add(&add(tau_d, &coil_torque(&held.m, b_b)), &held.rcs);
    let x = if lat >= dt {
        plant::step(x, dt, body, &held_ext, &held.hdot, &held.gdot)
    } else {
        let x1 = if lat > 0.0 { plant::step(x, lat, body, &held_ext, &held.hdot, &held.gdot) } else { *x };
        plant::step(&x1, dt - lat, body, &tau_ext, &a.hdot, &a.gdot)
    };
    *held = Held { m: a.m_b, hdot: a.hdot, gdot: a.gdot, rcs: a.tau_rcs };
    x
}

pub fn run(c: &Config, o: &Opts) -> Result<Record, Error> {
    if let Some(m) = &o.oils {
        if m.cpi_max < m.cpi { return Err(Error::refused(format!("soft OILS: the worst CPI {} is below the nominal {}", m.cpi_max, m.cpi))); }
    }
    let wall = std::time::Instant::now();
    let seed = c.seed;
    let rs = |name: &str| Rng::new(seed, stream_id(name));
    let d = &c.dev;
    let p = &c.params;

    // ---- orbit and environment truth: the POP port (as asils.orbit) or the analytic model ----
    let (mut orb, raan) = Truth::new(c)?;
    let gh = field::gh(time::decyear(c.jd0));
    let facets = Facets::boxed(&c.box_m, &c.cm_offset_m, c.sigma_n, c.sigma_t, c.vb_ratio, c.refl, c.spec_frac);

    // ---- dispersed units (one draw per run) ----
    let mut u = Units::new(d, &rs);

    // plant: TRUE (misaligned) rotor geometry; the software holds the nominal one
    let (nr, ng) = (d.mex.n, d.mex.ng);
    let geo = Geometry::new(&u.mex.a0[..nr], &d.mex.g[..ng], &d.mex.gi[..nr]);
    let body = match c.flex { Some(f) => Body::flexible(c.inertia, geo, f), None => Body::rigid(c.inertia, geo) };

    // ---- flight software behind the bus ----
    let mut bus = Bus::default();
    let mut fsw = Fsw::init(o.fsw.clone(), &c.blob(), 0, &mut bus)?;
    let fsw_build = fsw.build_id();

    // ---- initial state ----
    let (r, v) = orb.state(0.0)?;
    let mut gd = Guid { q_off: p.gd_q_off, roll_deg: p.gd_roll_deg, t0: p.gd_t0, t_slew: p.gd_T, axis: p.gd_axis, q_inertial: p.gd_q_inertial,
        sun_axis: p.sun_axis, roll_axis: p.roll_axis, sun_eci: adcs_fsw::env::sun_model(c.jd0), flip: false };
    if p.gd_yaw_flip != 0 { adcs_fsw::guid::yaw_flip(&mut gd, &r, &v, p.gd_flip_hyst); }
    let mut x = initial_state(c, &r, &v, &gd, &mut rs("initial"), nr);

    let dt = c.dt;
    let rt = Rates::new(c);
    if d.gps.fitted && d.gps.latency/dt + 2.0 > GPS_HIST as f64 {
        return Err(Error::refused(format!("GNSS latency {} s is {:.0} control steps; the engine keeps {} steps of history", d.gps.latency, d.gps.latency/dt, GPS_HIST - 2)));
    }
    let mut tmax = [0.0; NR];
    for i in 0..nr { tmax[i] = d.mex.torque_max[i]; }

    let mut rows = Vec::with_capacity((rt.n/rt.every + 2) as usize);
    let mut log = vec![];
    let mut a = Actuation::default();
    let mut prop = 0.0;
    let mut eacc = [0.0; 3];
    let mut fault_done = vec![false; c.faults.len()];
    let (mut b_eci, mut sun_rel, mut moon_rel, mut nu, mut v_rel, mut rho, mut psrp) = ([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 1.0, [0.0; 3], 0.0, 0.0);
    let mut last_print = std::time::Instant::now();
    let mut oils = o.oils.as_ref().map(|m| OilsStats { model: Some(m.clone()), ..Default::default() });
    if oils.is_some() && o.oils.as_ref().map(|m| m.fixed_s.is_none()).unwrap_or(false) && !matches!(o.fsw, Impl::Obc(_)) { return Err(Error::run("soft OILS needs the flight software on a virtual OBC (--fsw qemu | qemu-rs | obc-posix ...)")); }
    let mut held = Held::default();
    for k in 0..=rt.n {
        let t = k as f64*dt;
        let (r, v) = orb.state(t)?;
        if k % rt.env == 0 {
            let ev = orb.env(t, c.jd0, &r, &v, &gh, c.igrf_nmax);
            b_eci = ev.b_eci; sun_rel = ev.sun_rel; moon_rel = ev.moon_rel; nu = ev.nu; v_rel = ev.v_rel; rho = ev.rho; psrp = ev.p_srp;
            orb.set_attitude(transpose(&dcm(&x.q)));      // attitude -> POP (box-wing / panel models read it)
        }
        u.faults(c, t, &mut fault_done, &mut log)?;
        let rb = dcm(&x.q);
        // what the sensors see of the sky: sens_sky_view's sky_view (the Earth's half-angle from the constants' R_E)
        let (b_b, sb, mb, nb, earth_ang) = crate::gen::skyview::sky_view(rb, b_eci, sun_rel, moon_rel, r);
        let sky = Sky { b_b, sb, mb, nb, earth_ang };

        // ---- sensors -> bytes ----
        let z = sense(&mut u, c, &mut bus, &x, &sky, &a.m_coil, nu, t, k, &rt, &r, &v);

        // ---- flight software ----
        let now = bus.now_ns;
        let rc = fsw.step(&mut bus, now);
        if rc != 0 {
            let why = fsw.link_error().map(|e| format!(": {e}")).unwrap_or_default();
            return Err(Error::run(format!("flight software step returned {rc} at t = {t}{why}")));
        }
        // soft OILS: when does this command reach the actuators?
        let lat = match (oils.as_mut(), o.oils.as_ref()) { (Some(st), Some(m)) => oils_latency(st, m, &fsw, &bus, dt), _ => 0.0 };
        let mut cmd = Commands::default();
        emu::decode_pwm(&bus.pwm, if d.mtq.fitted { d.mtq.m_max } else { 1.0 }, &mut cmd);
        for f in bus.can_tx.drain(..) { emu::decode_can(f.id, &f.data, &tmax, p.gim_rate_max, dt, &mut cmd); }
        bus.can_rx.clear();
        let dbg = fsw.debug();
        let mode = dbg[1] as u8;
        let ad_ok = dbg[2] != 0.0;
        mode_changes(&mut log, t, mode);

        // ---- actuators ----
        actuate(&mut u, c, &cmd, &x, &mut a, &mut prop);

        // ---- torques, plant ----
        let parts = torques::torques(&x.q, &r, &v_rel, &b_eci, &sun_rel, nu, psrp, rho, &c.inertia, &facets, &c.m_res, c.mu, c.env_on);
        let tau_d = add(&add(&parts[0], &parts[1]), &add(&parts[2], &parts[3]));
        let tau_mtq = coil_torque(&a.m_b, &sky.b_b);
        eacc = [eacc[0] + a.p_mtq, eacc[1] + a.p_mex, eacc[2] + a.p_rcs];
        if k % rt.every == rt.off || k == 0 {
            let pavg = if k > 0 { [eacc[0]/rt.every as f64, eacc[1]/rt.every as f64, eacc[2]/rt.every as f64] } else { [a.p_mtq, a.p_mex, a.p_rcs] };
            eacc = [0.0; 3];
            let faults = fsw.peek().map(|s| s.faults).unwrap_or(0);
            let tick = Tick { t, r, v, rho, nu, sun_rel, mode, ad_ok, pavg, faults, parts, tau_mtq, prop };
            rows.push(row(&tick, &x, &geo, nr, &a, &cmd, &dbg, &z, &sky));
        }
        if k == rt.n { break; }
        if o.realtime {
            let due = std::time::Duration::from_secs_f64(t + dt);
            let el = wall.elapsed();
            if due > el { std::thread::sleep(due - el); }
        }
        x = step_plant(&x, dt, &body, &tau_d, &tau_mtq, &sky.b_b, &a, oils.is_some().then_some(lat), &mut held);
        if !o.quiet && last_print.elapsed().as_secs_f64() > 10.0 {
            last_print = std::time::Instant::now();
            eprintln!("  t = {:7.0} / {:.0} s  mode {:<13} |w| {:.3} deg/s  ({:.0} s wall)", t, c.duration_s,
                crate::config::MODES.get(mode as usize).unwrap_or(&"?"), norm(&x.w).to_degrees(), wall.elapsed().as_secs_f64());
        }
    }
    Ok(Record { rows, nr, ng, raan_rad: raan, mode_log: log, wall_s: wall.elapsed().as_secs_f64(), fsw_build, fsw_impl: o.fsw.clone(), oils })
}
