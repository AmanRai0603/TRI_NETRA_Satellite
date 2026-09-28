//! One closed-loop SILS run (asils.run): the tick order of spec §9.3 --
//! environment, sensors -> device emulators -> bytes, flight software step,
//! bytes -> actuator commands, actuators, torques, plant, recorder.
use crate::config::{Config, GUID};
use adcs_fsw::ctl::{guidance, Guid};
use adcs_fsw_abi::{Bus, Fsw, Impl};
use adcs_sim_core::actuators::{Mex, Mtq, Rcs};
use adcs_sim_core::emu::{self, proto, Commands};
use adcs_sim_core::la::*;
use adcs_sim_core::orbit::{self, Orbit, OrbitCfg};
use adcs_sim_core::plant::{self, Body, Geometry, State};
use adcs_sim_core::rng::{stream_id, Rng};
use adcs_sim_core::sensors::*;
use adcs_sim_core::torques::{self, Facets};
use adcs_sim_core::{ephem, field, time, NC, NG, NR};

/// The environment the attitude loop reads at an env tick.
pub struct Env { pub b_eci: V3, pub sun_rel: V3, pub nu: f64, pub v_rel: V3, pub rho: f64, pub p_srp: f64 }

/// The truth orbit and environment. `Pop`: the Rust port of POP stepped as asils.orbit
/// does (bit-identical to the MATLAB twin's orbit, Sun, Moon, density, frame). `Fast`:
/// the analytic models of adcs-sim-core (J2-J6, Montenbruck-Gill, exponential density).
pub enum Truth { Pop(Box<adcs_pop::accel::InLoop>), Fast(Orbit, f64) }

impl Truth {
    pub fn new(c: &Config) -> Result<(Truth, f64), String> {
        if c.orbit_model == "pop" {
            use adcs_pop::accel::{sso_initial, Forces, InLoop, Sc, World};
            let cr = 1.0 + c.refl;
            let w = World::new(c.epoch_utc, adcs_pop::frames::Build::Gmst, adcs_pop::frames::FrameOpt::default(),
                adcs_pop::gravity::Field::default_field(), Forces::sils(cr),
                Sc { mass: c.mass_kg, aref: c.aref_m2, cd: Some(c.cd), cr: Some(cr), r_bi: adcs_pop::la::I3, srp_facets: vec![], drag_facets: None },
                Some(adcs_pop::spaceweather::ManualIndices { f107: c.f107, f107a: Some(c.f107a), kp: Some(adcs_pop::atmos::dtm2020::KpIn::Scalar(c.kp)), ap: Some(c.ap), ap3: None }),
                Some(crate::pop_kernel()))?;
            let (r0, v0, raan) = sso_initial(&w, c.alt_km, c.ecc, c.inc_deg, c.ltan_h, 0.0, c.u0_deg)?;
            Ok((Truth::Pop(Box::new(InLoop::new(w, c.orbit_step_s, r0, v0)?)), raan))
        } else {
            let jd_tt0 = c.jd0 + time::TT_MINUS_UTC_S/86400.0;
            let s0 = ephem::sun(jd_tt0);
            let raan = (s0[1].atan2(s0[0]) + (c.ltan_h - 12.0)*15.0*std::f64::consts::PI/180.0).rem_euclid(2.0*std::f64::consts::PI);
            let a = orbit::RE + c.alt_km*1e3;
            let (r0, v0) = orbit::coe2rv(a, c.ecc, c.inc_deg.to_radians(), raan, 0.0, c.u0_deg.to_radians());
            let o = Orbit::new(OrbitCfg { jd0_utc: c.jd0, step_s: c.orbit_step_s, zonal_max: c.zonal_max, third_body: c.third_body, drag: c.drag, srp: c.srp,
                mass_kg: c.mass_kg, area_m2: c.aref_m2, cd: c.cd, cr: 1.0 + c.refl, density_scale: c.density_scale }, r0, v0);
            Ok((Truth::Fast(o, c.jd0), raan))
        }
    }
    pub fn state(&mut self, t: f64) -> Result<(V3, V3), String> {
        match self { Truth::Pop(o) => o.state(t), Truth::Fast(o, _) => Ok(o.state(t)) }
    }
    /// asils.run's env refresh: field at the POP position (op.geodetic), Sun, shadow,
    /// atmosphere-relative velocity, density and SRP pressure.
    pub fn env(&self, t: f64, r: &V3, v: &V3, gh: &field::Gh, nmax: usize) -> Env {
        match self {
            Truth::Pop(o) => {
                let x = o.context(t);
                let re = mv(&x.c, r);
                let (lat, lon, h) = adcs_pop::geodetic::geodetic(&re);
                let w = o.omega_e;
                Env { b_eci: field::eci_at(lat, lon, h, &x.c, gh, nmax), sun_rel: sub(&x.sun_eci, r), nu: ephem::shadow(r, &x.sun_eci),
                      v_rel: [v[0] + w*r[1], v[1] - w*r[0], v[2]], rho: x.rho, p_srp: x.p_srp }
            }
            Truth::Fast(o, jd0) => {
                let xc = o.context(t);
                let cm = time::eci2ecef(jd0 + t/86400.0);
                Env { b_eci: field::eci(&mv(&cm, r), &cm, gh, nmax), sun_rel: sub(&xc.sun, r), nu: ephem::shadow(r, &xc.sun),
                      v_rel: [v[0] + orbit::OMEGA_E*r[1], v[1] - orbit::OMEGA_E*r[0], v[2]], rho: xc.rho, p_srp: xc.p_srp }
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
#[derive(Clone, Debug)]
pub struct OilsModel { pub cpu_hz: f64, pub cpi: f64, pub i2c_hz: f64, pub spi_hz: f64, pub can_bps: f64 }
impl Default for OilsModel {
    /// A Cortex-M4F OBC at 168 MHz (STM32F4 class, flash accelerator on: CPI ~1.25),
    /// I2C fast mode, SPI 1 MHz, CAN 1 Mbit/s.
    fn default() -> Self { OilsModel { cpu_hz: 168e6, cpi: 1.25, i2c_hz: 400e3, spi_hz: 1e6, can_bps: 1e6 } }
}

#[derive(Clone, Debug, Default)]
pub struct OilsStats {
    pub model: Option<(f64, f64, f64, f64, f64)>,
    pub ticks: u64, pub overruns: u64,
    pub exec_s: Vec<f64>, pub io_s: Vec<f64>, pub lat_s: Vec<f64>, pub insn: Vec<f64>,
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
        let (hz, cpi, i2c, spi, can) = self.model.unwrap_or_default();
        serde_json::json!({
            "model": {"cpu_hz": hz, "cpi": cpi, "i2c_hz": i2c, "spi_hz": spi, "can_bps": can},
            "ticks": self.ticks, "overruns": self.overruns, "control_period_s": dt,
            "exec_s": s(&self.exec_s), "bus_s": s(&self.io_s), "latency_s": s(&self.lat_s),
            "instructions": if self.insn.is_empty() { serde_json::Value::Null } else { s(&self.insn) },
            "cpu_load_mean": Self::mean(&self.exec_s)/dt, "cpu_load_max": Self::max(&self.exec_s)/dt,
            "deadline_margin_min_s": dt - Self::max(&self.lat_s),
        })
    }
}

fn mode_changes(log: &mut Vec<(f64, String)>, t: f64, m: u8) {
    let name = crate::config::MODES.get(m as usize).copied().unwrap_or("none").to_string();
    if log.last().map(|x| x.1 != name).unwrap_or(true) { log.push((t, name)); }
}

pub fn run(c: &Config, o: &Opts) -> Result<Record, String> {
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
    let mut disp = rs("dispersion");
    let mut gyro = if d.gyro.fitted { Some(Gyro::new(d.gyro, &mut disp, rs("gyro"))) } else { None };
    let mut mag = Mag::new(d.mag, &mut disp, rs("mag"));
    let mut sun = if d.sun.fitted { Some(Sun::new(d.sun, &mut disp, rs("sun"))) } else { None };
    let mut st = if d.st.fitted { Some(St::new(d.st, &mut disp, rs("st"))) } else { None };
    let mut mtq = Mtq::new(d.mtq, &mut disp);
    let mut es = if d.es.fitted { Some(Es::new(d.es, &mut disp, rs("es"))) } else { None };
    let mut css = if d.css.fitted { Some(Css::new(d.css, &mut disp, rs("css"))) } else { None };
    let mut mex = Mex::new(d.mex, &mut disp, rs("mex"));
    let mut rcs = if d.rcs.fitted { Some(Rcs::new(d.rcs, &mut disp)) } else { None };
    let mut gps = Gps { d: d.gps, dead: false, rng: rs("gps") };
    let mut tlm = rs("telemetry");

    // plant: TRUE (misaligned) rotor geometry; the software holds the nominal one
    let (nr, ng, nc) = (d.mex.n, d.mex.ng, if d.rcs.fitted { d.rcs.nc } else { 0 });
    let geo = Geometry::new(&mex.a0[..nr], &d.mex.g[..ng], &d.mex.gi[..nr]);
    let body = Body { i: c.inertia, iinv: inv(&c.inertia), m: geo };

    // ---- flight software behind the bus ----
    let mut bus = Bus::default();
    let mut fsw = Fsw::init(o.fsw.clone(), &c.blob(), 0, &mut bus)?;
    let fsw_build = fsw.build_id();

    // ---- initial state (asils.run initial_) ----
    let (r, v) = orb.state(0.0)?;
    let gd = Guid { q_off: p.gd_q_off, roll_deg: p.gd_roll_deg, t0: p.gd_t0, t_slew: p.gd_T, axis: p.gd_axis, q_inertial: p.gd_q_inertial,
        sun_axis: p.sun_axis, roll_axis: p.roll_axis, sun_eci: adcs_fsw::env::sun_model(c.jd0) };
    let mut ir = rs("initial");
    let ini = c.scenario.get("initial").cloned().unwrap_or_default();
    let att = ini.get("attitude").cloned().unwrap_or_default();
    let rate = ini.get("rate").cloned().unwrap_or_default();
    let q_nad = guidance(0, &r, &v, 0.0, &gd).q;
    let q0 = match crate::json::s(&att, "kind", "") {
        "random" => { let x = [ir.normal(), ir.normal(), ir.normal(), ir.normal()]; qnorm(&x) }
        k @ ("error_from_target" | "error_from_guidance") => {
            let qr = if k == "error_from_guidance" && c.gd_kind0 >= 0 { guidance(c.gd_kind0, &r, &v, 0.0, &gd).q } else { q_nad };
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
        "lvlh" => mv(&dcm(&q0), &scale(&cross(&r, &v), 1.0/dot(&r, &r))),
        "guidance" => {
            let mut w = if c.gd_kind0 >= 0 { guidance(c.gd_kind0, &r, &v, 0.0, &gd).w } else { [0.0; 3] };
            w = add(&w, &scale(&unit(&ir.normal3()), crate::json::f(&rate, "extra_deg_s", 0.0).to_radians()));
            w
        }
        _ => rate.get("value_deg_s").and_then(crate::json::v3).map(|x| scale(&x, std::f64::consts::PI/180.0)).unwrap_or([0.0; 3]),
    };
    let mut x = State { q: q0, w: w0, ..Default::default() };
    let h0 = crate::json::f(&ini, "wheel_momentum_Nms", f64::NAN);
    for i in 0..nr { x.h[i] = if h0.is_nan() { c.h_t_rot[i] } else { h0 }; }

    // ---- rates ----
    let dt = c.dt;
    let n = (c.duration_s/dt).round() as u64;
    let every = ((c.record_dt/dt).round() as u64).max(1);
    let off = every/2;
    let env_every = ((c.env_dt_s/dt).round() as u64).max(1);
    let st_every = if d.st.fitted { ((1.0/(d.st.rate_hz*dt)).round() as u64).max(1) } else { 1 };
    let gps_every = ((1.0/dt).round() as u64).max(1);
    let es_every = if d.es.fitted { ((1.0/(d.es.rate_hz*dt)).round() as u64).max(1) } else { 1 };
    let mut tmax = [0.0; NR];
    for i in 0..nr { tmax[i] = d.mex.torque_max[i]; }

    let mut rows = Vec::with_capacity((n/every + 2) as usize);
    let mut log = vec![];
    let (mut m_b, mut p_mtq, mut p_mex, mut p_rcs) = ([0.0; 3], 0.0, 0.0, 0.0);
    let (mut hdot, mut gdot, mut tau_rcs, mut prop) = ([0.0; NR], [0.0; NG], [0.0; 3], 0.0);
    let mut eacc = [0.0; 3];
    let mut fault_done = vec![false; c.faults.len()];
    let (mut b_eci, mut sun_rel, mut nu, mut v_rel, mut rho, mut psrp) = ([0.0; 3], [1.0, 0.0, 0.0], 1.0, [0.0; 3], 0.0, 0.0);
    let mut last_print = std::time::Instant::now();
    let mut can_rx_count = 0usize;
    // soft OILS: the actuation that holds until this tick's command lands
    let mut oils = o.oils.as_ref().map(|m| OilsStats { model: Some((m.cpu_hz, m.cpi, m.i2c_hz, m.spi_hz, m.can_bps)), ..Default::default() });
    if oils.is_some() && !matches!(o.fsw, Impl::Obc(_)) { return Err("soft OILS needs the flight software on a virtual OBC (--fsw qemu | qemu-rs | obc-posix ...)".into()); }
    let (mut held_m, mut held_hdot, mut held_gdot, mut held_rcs) = ([0.0; 3], [0.0; NR], [0.0; NG], [0.0; 3]);
    for k in 0..=n {
        let t = k as f64*dt;
        let (r, v) = orb.state(t)?;
        if k % env_every == 0 {
            let ev = orb.env(t, &r, &v, &gh, c.igrf_nmax);
            b_eci = ev.b_eci; sun_rel = ev.sun_rel; nu = ev.nu; v_rel = ev.v_rel; rho = ev.rho; psrp = ev.p_srp;
            orb.set_attitude(transpose(&dcm(&x.q)));      // attitude -> POP (box-wing / panel models read it)
        }
        for (i, f) in c.faults.iter().enumerate() {
            if fault_done[i] || t < f.t_s { continue; }
            fault_done[i] = true;
            let ix = f.index.saturating_sub(1);
            match f.kind.as_str() {
                "rotor_fail" => mex.failed[ix] = true,
                "gimbal_stuck" => mex.gfailed[ix] = true,
                "st_head_fail" => if let Some(s) = st.as_mut() { s.dead[ix] = true },
                "coil_fail" => mtq.dead[ix] = true,
                "gyro_bias_step" => if let Some(g) = gyro.as_mut() { g.b = add(&g.b, &f.value) },
                "gps_outage" => gps.dead = true,
                "rcs_valve_fail" => if let Some(r) = rcs.as_mut() { r.failed[ix] = true },
                "mag_fail" => mag.dead = true,
                other => return Err(format!("unknown fault {other}")),
            }
            log.push((t, format!("FAULT injected: {} {}", f.kind, f.index)));
        }
        let rb = dcm(&x.q);
        let b_b = mv(&rb, &b_eci);
        let sb = unit(&mv(&rb, &sun_rel));
        let nb = scale(&mv(&rb, &r), -1.0/norm(&r));
        let earth_ang = (6378137.0/norm(&r)).asin();

        // ---- sensors -> bytes ----
        bus.now_ns = (t*1e9).round() as u64;
        let w_meas = match gyro.as_mut() { Some(g) => g.sample(&x.w, dt), None => x.w };
        bus.gyro = gyro.as_ref().map(|_| emu::gyro_resp(true, &w_meas));
        let b_meas = mag.sample(&b_b, &m_b);
        bus.mag = if d.mag.fitted { Some(emu::mag_regs(true, &b_meas)) } else { None };
        let sun_meas = if let Some(s) = sun.as_mut() { s.sample(&sb, nu) } else if let Some(s) = css.as_mut() { s.sample(&sb, nu, &nb, earth_ang) } else { None };
        bus.sun = if d.sun.fitted || d.css.fitted { Some(emu::unit_regs(sun_meas.is_some(), &sun_meas.unwrap_or([0.0; 3]))) } else { None };
        let mut st_ok = false;
        if let Some(s) = st.as_mut() {
            s.history(t, &x.q);
            if k % st_every == 0 {
                let heads = s.sample(&x.q, t, &x.w, &sb, &nb, earth_ang);
                let mut hv = [(false, [0.0, 0.0, 0.0, 1.0]); 2];
                for h in 0..d.st.nh { if let Some(q) = heads[h] { hv[h] = (true, q); st_ok = true; } }
                let mut buf = [0u8; 64];
                let len = emu::st_frame(&hv[..d.st.nh], &mut buf);
                bus.push_uart(proto::ST_UART, &buf[..len]);
            }
        }
        if let Some(e) = es.as_mut() {
            let z = if k % es_every == 0 { e.sample(&nb) } else { None };
            bus.es = Some(emu::unit_regs(z.is_some(), &z.unwrap_or([0.0; 3])));
        }
        if d.gps.fitted && k % gps_every == 0 && !gps.dead {
            let (rg, vg) = gps.sample(&r, &v);
            let mut buf = [0u8; 64];
            let len = emu::gps_frame(true, &rg, &vg, &mut buf);
            bus.push_uart(proto::GPS_UART, &buf[..len]);
        }
        for i in 0..nr {
            let hm = x.h[i] + 1e-7*tlm.normal();
            let dm = if d.mex.gi[i] > 0 { x.d[d.mex.gi[i] - 1] + 1e-5*tlm.normal() } else { 0.0 };
            let (id, data) = emu::rotor_tm(i, hm, dm);
            bus.push_can(id, data);
        }
        can_rx_count += nr;

        // ---- flight software ----
        let now = bus.now_ns;
        let rc = fsw.step(&mut bus, now);
        if rc != 0 { return Err(format!("flight software step returned {rc} at t = {t}")); }
        // soft OILS: when does this command reach the actuators?
        let mut lat = 0.0;
        if let (Some(st), Some(m)) = (oils.as_mut(), o.oils.as_ref()) {
            let (exec, insn) = match fsw.obc_exec() {
                Some((_, Some(n))) => (n*m.cpi/m.cpu_hz, Some(n)),
                Some((s, None)) => (s, None),
                None => (0.0, None),
            };
            // synchronous reads inside the step: I2C (addr + reg, restart, 7 bytes), SPI gyro (13 bytes)
            let i2c = [bus.mag.is_some(), bus.sun.is_some(), bus.es.is_some()].iter().filter(|x| **x).count() as f64;
            let io = i2c*10.0*9.0/m.i2c_hz + if bus.gyro.is_some() { 13.0*8.0/m.spi_hz } else { 0.0 }
                + bus.can_tx.len() as f64*130.0/m.can_bps;
            lat = io + exec;
            st.ticks += 1; st.exec_s.push(exec); st.io_s.push(io); st.lat_s.push(lat);
            if let Some(n) = insn { st.insn.push(n); }
            if lat >= dt { st.overruns += 1; }
        }
        let mut cmd = Commands::default();
        emu::decode_pwm(&bus.pwm, if d.mtq.fitted { d.mtq.m_max } else { 1.0 }, &mut cmd);
        for f in bus.can_tx.drain(..) { emu::decode_can(f.id, &f.data, &tmax, p.gim_rate_max, dt, &mut cmd); }
        bus.can_rx.clear();
        let dbg = fsw.debug();
        let mode = dbg[1] as u8;
        let ad_ok = dbg[2] != 0.0;
        mode_changes(&mut log, t, mode);

        // ---- actuators ----
        if d.mtq.fitted { let (m, pw) = mtq.apply(&cmd.m_body); m_b = m; p_mtq = pw; }
        if nr > 0 { let (hd, gd_, pw) = mex.apply(&cmd.cmd_r, &cmd.cmd_g, &x.h, dt); hdot = hd; gdot = gd_; p_mex = pw; }
        if let Some(rc_) = rcs.as_mut() {
            let mut duty = [0.0; NC];
            duty[..nc].copy_from_slice(&cmd.duty[..nc]);
            let (tq, mdot, pw) = rc_.apply(&duty, dt);
            tau_rcs = tq; p_rcs = pw;
            prop += mdot*dt;
            if prop >= d.rcs.prop_kg { rc_.failed = [true; NC]; }
        }

        // ---- torques, plant ----
        let parts = torques::torques(&x.q, &r, &v_rel, &b_eci, &sun_rel, nu, psrp, rho, &c.inertia, &facets, &c.m_res, c.mu, c.env_on);
        let tau_d = add(&add(&parts[0], &parts[1]), &add(&parts[2], &parts[3]));
        let tau_mtq = cross(&m_b, &b_b);
        eacc = [eacc[0] + p_mtq, eacc[1] + p_mex, eacc[2] + p_rcs];
        if k % every == off || k == 0 {
            let pavg = if k > 0 { [eacc[0]/every as f64, eacc[1]/every as f64, eacc[2]/every as f64] } else { [p_mtq, p_mex, p_rcs] };
            eacc = [0.0; 3];
            let ax = geo.axes(&x.d);
            let mut tau_rw = [0.0; 3];
            for i in 0..nr { for j in 0..3 { tau_rw[j] -= ax[i][j]*hdot[i]; } }
            let faults = fsw.peek().map(|s| s.faults).unwrap_or(0);
            rows.push(Row {
                t, q: x.q, w: x.w, q_est: if ad_ok { Some([dbg[3], dbg[4], dbg[5], dbg[6]]) } else { None }, w_meas,
                tau_req: [dbg[17], dbg[18], dbg[19]], h_w: x.h, tau_rw, cmd_r: cmd.cmd_r, n_failed: faults.count_ones(), delta: x.d,
                tau_rcs, prop_kg: prop, m: m_b, tau_dist: parts, tau_mtq, mode, p_mtq: pavg[0], p_rw: pavg[1], p_rcs: pavg[2],
                r, v, rho, nu, b: b_b, b_meas, sun_ok: sun_meas.is_some(), st_ok, ad_ok, sun_eci: unit(&sun_rel), sun_body: sb,
            });
        }
        if k == n { break; }
        if o.realtime {
            let due = std::time::Duration::from_secs_f64(t + dt);
            let el = wall.elapsed();
            if due > el { std::thread::sleep(due - el); }
        }
        let tau_ext = add(&add(&tau_d, &tau_mtq), &tau_rcs);
        if oils.is_some() {
            // the previous actuation holds for the latency, then this tick's command acts
            let held_ext = add(&add(&tau_d, &cross(&held_m, &b_b)), &held_rcs);
            if lat >= dt {
                x = plant::step(&x, dt, &body, &held_ext, &held_hdot, &held_gdot);
            } else {
                if lat > 0.0 { x = plant::step(&x, lat, &body, &held_ext, &held_hdot, &held_gdot); }
                x = plant::step(&x, dt - lat, &body, &tau_ext, &hdot, &gdot);
            }
            held_m = m_b; held_hdot = hdot; held_gdot = gdot; held_rcs = tau_rcs;
        } else {
            x = plant::step(&x, dt, &body, &tau_ext, &hdot, &gdot);
        }
        if !o.quiet && last_print.elapsed().as_secs_f64() > 10.0 {
            last_print = std::time::Instant::now();
            eprintln!("  t = {:7.0} / {:.0} s  mode {:<13} |w| {:.3} deg/s  ({:.0} s wall)", t, c.duration_s,
                crate::config::MODES.get(mode as usize).unwrap_or(&"?"), norm(&x.w).to_degrees(), wall.elapsed().as_secs_f64());
        }
    }
    let _ = (can_rx_count, GUID);
    Ok(Record { rows, nr, ng, raan_rad: raan, mode_log: log, wall_s: wall.elapsed().as_secs_f64(), fsw_build, fsw_impl: o.fsw.clone(), oils })
}
