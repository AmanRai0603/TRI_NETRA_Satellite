//! Product + parts -> device descriptors (asils.product.load).
use crate::error::Error;
use crate::json::{self, get};
use adcs_sim_core::actuators::{Kind, MexDesc, MtqDesc, RcsDesc};
use adcs_sim_core::comp::star_tracker::{Camera, MAX_SPOTS};
use adcs_sim_core::comp::sun_sensor::Head;
use adcs_sim_core::sensors::*;
use serde_json::Value;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default)]
pub struct Dev {
    pub id: String, pub label: String, pub family: String, pub algorithms: Vec<String>, pub selected: Option<Value>,
    pub boresight: [f64; 3], pub sun_axis: [f64; 3],
    pub gyro: GyroDesc, pub mag: MagDesc, pub sun: SunDesc, pub css: CssDesc, pub st: StDesc, pub es: EsDesc, pub gps: GpsDesc,
    pub mtq: MtqDesc, pub mex: MexDesc, pub rcs: RcsDesc,
    /// each rotor's imbalance as its part states it (static [kg m], dynamic [kg m^2]), in rotor order;
    /// None where the part does not state both (jitter is then not computed); a fluid ring has none
    pub imbalance: Vec<Option<(f64, f64)>>, pub rotor_part: Vec<String>,
    /// the product and part files it was read from (their fingerprint goes in the run's provenance)
    pub files: Vec<PathBuf>,
}

/// data/<kind>/<id>.json, else store/sized/*/<kind>/<id>.json.
/// $ADCS_SIZED_DIR/<kind>/<id>.json first when set: the design loop's current iteration
/// (tools/pipeline.py) flies its own sized products without touching the MATLAB ones.
pub fn find(root: &Path, kind: &str, id: &str) -> Result<PathBuf, Error> {
    if let Some(d) = std::env::var_os("ADCS_SIZED_DIR").filter(|d| !d.is_empty()) {
        let g = Path::new(&d).join(kind).join(format!("{id}.json"));
        if g.is_file() { return Ok(g); }
    }
    let f = root.join("data").join(kind).join(format!("{id}.json"));
    if f.is_file() { return Ok(f); }
    if let Ok(rd) = std::fs::read_dir(crate::store_root().join("sized")).or_else(|_| std::fs::read_dir(root.join("store/sized"))) {
        let mut ds: Vec<_> = rd.flatten().map(|e| e.path()).collect();
        ds.sort();
        for d in ds { let g = d.join(kind).join(format!("{id}.json")); if g.is_file() { return Ok(g); } }
    }
    Err(Error::refused(format!("no {} record {id} (data/{kind} or store/sized/*/{kind})", kind.trim_end_matches('s'))))
}

fn sig(ds: &Value, k: &str) -> f64 { ds.get(k).and_then(|x| x.get("sigma")).and_then(|x| x.as_f64()).unwrap_or(0.0) }
fn lohi(ds: &Value, k: &str, d: f64) -> (f64, f64) {
    let e = ds.get(k);
    (e.and_then(|x| x.get("lo")).and_then(|x| x.as_f64()).unwrap_or(d), e.and_then(|x| x.get("hi")).and_then(|x| x.as_f64()).unwrap_or(d))
}
fn axes(v: Option<&Value>) -> Vec<[f64; 3]> { v.map(json::vecs).unwrap_or_default().into_iter().map(json::unit).collect() }

/// One fitted part: where the product places it (`f`), its nominal values and dispersions. Every
/// value a fitted device reads must be stated by the part: one it does not state is noted (and
/// the product refused), never read as 0.
struct Part<'a> { f: &'a Value, nm: Value, ds: Value, missing: std::cell::RefCell<Vec<String>> }

impl<'a> Part<'a> {
    fn new(f: &'a Value, p: &Value) -> Part<'a> {
        Part { f, nm: p.get("nominal").cloned().unwrap_or(Value::Null), ds: p.get("dispersion").cloned().unwrap_or(Value::Null), missing: Default::default() }
    }
    fn n(&self, k: &str) -> f64 { let v = json::f(&self.nm, k, f64::NAN); if !v.is_finite() { self.missing.borrow_mut().push(k.to_string()); } v }
    /// A value that must be stated and above zero (a time constant of zero would divide; a
    /// negative one has no meaning).
    fn pos(&self, k: &str) -> f64 { self.bounded(k, |v| v > 0.0, "above 0") }
    fn nonneg(&self, k: &str) -> f64 { self.bounded(k, |v| v >= 0.0, "0 or more") }
    fn bounded(&self, k: &str, ok: impl Fn(f64) -> bool, what: &str) -> f64 {
        let v = json::f(&self.nm, k, f64::NAN);
        if !v.is_finite() { self.missing.borrow_mut().push(k.to_string()); }
        else if !ok(v) { self.missing.borrow_mut().push(format!("{k} {what} (it states {v})")); }
        v
    }
    fn sig(&self, k: &str) -> f64 { sig(&self.ds, k) }
    fn axes(&self, k: &str) -> Vec<[f64; 3]> { axes(self.f.get(k)) }
}

/// What the engine and the flight software hold, refused by name, never overrun or cut.
fn capacity(id: &str, part: &str, slot: &str, f: &Value, pt: &Part, x: &MexDesc) -> Result<(), Error> {
    use adcs_sim_core::{NG, NH, NR, NS};
    let count = |key: &str| f.get(key).map(json::vecs).map(|v| v.len()).unwrap_or(0);
    let over = |what: &str, have: usize, cap: usize| -> Result<(), Error> {
        if have > cap { Err(Error::refused(format!("product {id}: {have} {what}; the engine holds at most {cap}"))) } else { Ok(()) }
    };
    match slot {
        "coils" => over("magnetorquer coils", count("axes_body"), NS),
        "wheels" | "rings" => over("rotors", x.n + count("axes_body"), NR),
        "cmg" | "vscmg" => { over("rotors", x.n + count("spin_axes_body"), NR)?; over("gimbals", x.ng + count("gimbal_axes_body"), NG) }
        "star_tracker" => over("star-tracker heads", count("boresights_body").max(count("boresight_body")), NH),
        "sun_sensors" | "coarse_sun_sensors" => over("Sun sensor heads", count("normals_body"), NS),
        "rcs" => {
            let t = json::f(&pt.nm, "thrusters", f64::NAN);
            if t != 12.0 { Err(Error::refused(format!("part {part}: {t} thrusters; the engine models 12 (six couples)"))) } else { Ok(()) }
        }
        _ => Ok(()),
    }
}

/// The model level a fill selects, refused by name when the engine has no such model.
fn selector(id: &str, slot: &str, f: &Value) -> Result<(), Error> {
    let (key, have): (&str, &[&str]) = match slot {
        "star_tracker" => ("model", &["noise", "quest", "image"]),
        "sun_sensors" => ("level", &["model", "chain"]),
        _ => return Ok(()),
    };
    match f.get(key) {
        None => Ok(()),
        Some(Value::String(v)) if have.contains(&v.as_str()) => Ok(()),
        Some(v) => Err(Error::refused(format!("product {id}: {slot} {key} {v}; the engine models {}", have.join(", ")))),
    }
}

/// One rotor added to the momentum-device set.
#[allow(clippy::too_many_arguments)]
fn add_rotor(x: &mut MexDesc, kind: Kind, a: [f64; 3], gi: usize, hmax: f64, tmax: f64, j: f64, cou: f64, vis: f64, pst: f64, tsig: f64, flo: f64, fhi: f64, mis: f64) {
    let i = x.n;
    x.kind[i] = kind; x.a0[i] = a; x.gi[i] = gi; x.h_max[i] = hmax; x.torque_max[i] = tmax; x.jrot[i] = j;
    x.coulomb[i] = cou; x.viscous[i] = vis; x.p_steady[i] = pst; x.tsig[i] = tsig; x.flo[i] = flo; x.fhi[i] = fhi; x.misalign[i] = mis;
    x.t_sd[i] = f64::INFINITY; x.k_hv[i] = 1.0; x.ac[i] = 1.0; x.s[i] = 1.0; x.l[i] = 1.0; x.eta_lo[i] = 1.0; x.eta_hi[i] = 1.0;
    x.n += 1;
}

/// The actuator slots: coils, wheels, rings, CMG/VSCMG, thrusters. False when `slot` is not one.
fn fit_actuator(d: &mut Dev, x: &mut MexDesc, slot: &str, pt: &Part) -> bool {
    let n0 = x.n;
    let fitted = fit_actuator_(d, x, slot, pt);
    // the imbalance of every rotor this slot added (jitter): stated, or none for a fluid ring
    let stated = (json::get(&pt.nm, "static_imbalance_kgm").and_then(|v| v.as_f64()), json::get(&pt.nm, "dynamic_imbalance_kgm2").and_then(|v| v.as_f64()));
    for _ in n0..x.n {
        d.imbalance.push(match (slot, stated) { ("rings", _) => Some((0.0, 0.0)), (_, (Some(a), Some(b))) => Some((a, b)), _ => None });
        d.rotor_part.push(json::s(pt.f, "part", "?").to_string());
    }
    fitted
}

fn fit_actuator_(d: &mut Dev, x: &mut MexDesc, slot: &str, pt: &Part) -> bool {
    let (n, ds) = (|k: &str| pt.n(k), &pt.ds);
    match slot {
        "coils" => {
            let a = pt.axes("axes_body");
            let mut m = MtqDesc { fitted: true, n: a.len(), m_max: n("dipole_max_Am2"), p_max: n("power_at_max_W"), scale_sigma: pt.sig("dipole_scale"), misalign: pt.sig("axis_misalignment_rad"),
                tau: pt.nonneg("time_constant_s"), ..Default::default() };
            for (j, v) in a.iter().enumerate() { m.axes[j] = *v; }
            d.mtq = m;
        }
        "wheels" => {
            let (flo, fhi) = lohi(ds, "friction_scale", 1.0);
            // the motor's torque-speed line from its constants: stall torque k_t V / R, no-load speed V / k_t
            let (kt, rw, vb) = (pt.pos("motor_kt_Nm_per_A"), pt.pos("motor_resistance_ohm"), pt.pos("bus_voltage_V"));
            let (wmax, fst, wst) = (pt.pos("speed_max_rad_s"), pt.pos("friction_static_Nm"), pt.pos("stribeck_speed_rad_s"));
            if fst < n("friction_coulomb_Nm") { pt.missing.borrow_mut().push(format!("friction_static_Nm of at least friction_coulomb_Nm (it states {fst})")); }
            for a in pt.axes("axes_body") {
                add_rotor(x, Kind::Rw, a, 0, n("h_max_Nms"), n("torque_max_Nm"), n("rotor_inertia_kgm2"), n("friction_coulomb_Nm"), n("friction_viscous_Nms"), n("power_steady_W"), pt.sig("torque_scale"), flo, fhi, pt.sig("axis_misalignment_rad"));
                let i = x.n - 1;
                x.speed_max[i] = wmax; x.t_stall[i] = kt*vb/rw; x.w_nl[i] = vb/kt; x.f_static[i] = fst; x.w_stribeck[i] = wst;
            }
        }
        "rings" => {
            let ac = std::f64::consts::PI*n("bore_m").powi(2)/4.0;
            let s = n("enclosed_area_m2");
            let k_hv = n("fluid_density_kg_m3")*ac*2.0*s;
            let tsd = n("fluid_density_kg_m3")*n("bore_m").powi(2)/(32.0*n("fluid_viscosity_Pa_s"));
            let hmax = k_hv*n("v_max_m_s");
            let (flo, fhi) = lohi(ds, "friction_scale", 1.0);
            let (elo, ehi) = lohi(ds, "pump_efficiency", 1.0);
            let fp = json::f(&pt.nm, "field_power_W", 0.0);
            for a in pt.axes("axes_body") {
                // an electromagnetic pump designed by adcs-design states its pressure-limited torque
                let tq = json::f(&pt.nm, "pump_torque_max_Nm", f64::NAN);
                add_rotor(x, Kind::Fmr, a, 0, hmax, if tq.is_finite() && tq > 0.0 { tq } else { 2.0*hmax/tsd }, k_hv, 0.0, 0.0, 0.0, 0.0, flo, fhi, pt.sig("axis_misalignment_rad"));
                let i = x.n - 1;
                x.t_sd[i] = tsd; x.k_hv[i] = k_hv; x.ac[i] = ac; x.s[i] = s; x.l[i] = n("channel_length_m");
                x.flow_noise_h[i] = k_hv*pt.sig("flow_sensor_noise_m_s");
                x.field_power[i] = if fp.is_nan() { 0.0 } else { fp };
                x.eta_lo[i] = elo; x.eta_hi[i] = ehi;
            }
        }
        "cmg" | "vscmg" => {
            let g = pt.axes("gimbal_axes_body");
            let a = pt.axes("spin_axes_body");
            let g0 = x.ng;
            for (j, v) in g.iter().enumerate() { x.g[g0 + j] = *v; }
            x.ng += g.len();
            let (flo, fhi) = lohi(ds, "friction_scale", 1.0);
            for (k, v) in a.iter().enumerate() {
                if slot == "cmg" {
                    add_rotor(x, Kind::Cmg, *v, g0 + k + 1, n("rotor_momentum_Nms"), n("rotor_torque_max_Nm"), n("rotor_inertia_kgm2"), 0.0, 0.0, n("power_steady_W"), pt.sig("torque_scale"), 1.0, 1.0, pt.sig("axis_misalignment_rad"));
                } else {
                    add_rotor(x, Kind::Vscmg, *v, g0 + k + 1, n("h_max_Nms"), n("rotor_torque_max_Nm"), n("rotor_inertia_kgm2"), n("friction_coulomb_Nm"), n("friction_viscous_Nms"), n("power_steady_W"), pt.sig("torque_scale"), flo, fhi, pt.sig("axis_misalignment_rad"));
                }
                x.h0[x.n - 1] = n("rotor_momentum_Nms");
            }
            x.gimbal_rate_max = n("gimbal_rate_max_rad_s");
            x.gimbal_power = n("gimbal_power_W");
        }
        "rcs" => {
            let fth = n("thrust_N");
            let arm = [n("arm_short_m"), n("arm_long_m"), n("arm_long_m")];
            let mut r = RcsDesc { fitted: true, nc: 6, thrust: fth, isp: n("isp_s"), mib: n("mib_s"), res: n("valve_res_s"), prop_kg: n("propellant_kg"), valve_power: n("valve_power_W"), thrust_sigma: pt.sig("thrust_scale"), misalign: pt.sig("axis_misalignment_rad"), ..Default::default() };
            (r.isp_lo, r.isp_hi) = lohi(ds, "isp_s", n("isp_s"));
            for ax in 0..3 { r.tau[2*ax][ax] = 2.0*fth*arm[ax]; r.tau[2*ax + 1][ax] = -2.0*fth*arm[ax]; }
            d.rcs = r;
        }
        _ => return false,
    }
    true
}

/// The sensor slots. False when `slot` is not one.
fn fit_sensor(d: &mut Dev, slot: &str, pt: &Part) -> bool {
    let (n, f) = (|k: &str| pt.n(k), pt.f);
    match slot {
        "star_tracker" => {
            let bs = if f.get("boresights_body").is_some() { pt.axes("boresights_body") } else { pt.axes("boresight_body") };
            let mut s = StDesc { fitted: true, nh: bs.len(), noise_cross: n("noise_cross_rad"), noise_roll: n("noise_roll_rad"), rate_hz: n("rate_Hz"),
                latency: n("latency_s"), max_rate: n("max_rate_rad_s"), sun_excl: n("sun_exclusion_rad"), earth_excl: n("earth_exclusion_rad"),
                fov: n("fov_half_angle_rad"), model: match json::s(f, "model", "quest") { "noise" => 0, "image" => 2, _ => 1 },
                bias_sigma: pt.sig("bias_rad"), misalign_sigma: pt.sig("axis_misalignment_rad"),
                moon_excl: pt.nonneg("moon_exclusion_rad"), blind_s: pt.nonneg("blind_recovery_s"), noise_rate_ref: pt.pos("noise_doubling_rate_rad_s"), ..Default::default() };
            for h in 0..s.nh { s.bs[h] = bs[h]; }
            if let Some(c) = get(f, "calibrated_residual_rad").and_then(|v| v.as_f64()) { s.bias_sigma = c; s.misalign_sigma = 0.0; }
            if s.model == 2 {
                // the rendered-frame chain: the camera and the onboard chain's settings, from the part
                let whole = |k: &str, lo: f64, hi: f64| {
                    let v = pt.pos(k);
                    if v.is_finite() && (v != v.round() || v < lo || v > hi) {
                        pt.missing.borrow_mut().push(format!("{k} as a whole number from {lo} to {hi} (it states {v})"));
                    }
                    v as usize
                };
                s.cam = Camera::new(s.fov, whole("detector_px", 16.0, 8192.0), pt.pos("psf_sigma_px"), pt.pos("flux_mag6_e"), pt.nonneg("background_e"),
                    pt.nonneg("read_noise_e"), pt.pos("centroid_k_sigma"), whole("max_spots", 3.0, MAX_SPOTS as f64), pt.pos("id_tol_rad"),
                    pt.pos("id_mag_tol"), pt.pos("fit_tol_rad"));
            }
            d.st = s;
        }
        "magnetometer" => d.mag = MagDesc { fitted: true, noise: n("noise_T_rms"), bias_t: n("bias_T"), bias_sigma: pt.sig("bias_T"), range: n("range_T"), sf_sigma: pt.sig("scale_factor"), misalign: pt.sig("axis_misalignment_rad"), k_coil: n("coil_coupling_T_per_Am2") },
        "sun_sensors" => {
            let a = pt.axes("normals_body");
            let mut s = SunDesc { fitted: true, n: a.len(), noise: n("accuracy_rad"), fov: n("fov_half_angle_rad"), bias_sigma: pt.sig("bias_rad"), ..Default::default() };
            if json::s(f, "level", "model") == "chain" {
                // quadrant currents -> angles: the head's aperture, height, current noise and threshold
                s.chain = true;
                s.head = Head { a: pt.pos("aperture_side_m"), h: pt.pos("aperture_height_m"), noise: pt.nonneg("current_noise_frac"), min_frac: pt.pos("current_min_frac") };
            }
            for (j, v) in a.iter().enumerate() { s.normals[j] = *v; }
            d.sun = s;
        }
        "coarse_sun_sensors" => {
            let a = pt.axes("normals_body");
            let mut s = CssDesc { fitted: true, n: a.len(), noise: n("noise_frac"), albedo: n("albedo"), scale_sigma: pt.sig("scale"), misalign: pt.sig("axis_misalignment_rad"), ..Default::default() };
            for (j, v) in a.iter().enumerate() { s.normals[j] = *v; }
            d.css = s;
        }
        "gyro" => d.gyro = GyroDesc { fitted: true, arw: n("arw_rad_per_sqrt_s"), rrw: n("rrw_rad_per_s_sqrt_s"), range: n("range_rad_s"), bias_sigma: pt.sig("bias_rad_s"), sf_sigma: pt.sig("scale_factor"), misalign: pt.sig("axis_misalignment_rad") },
        "earth_sensor" => {
            let bs = get(f, "boresight_body").and_then(json::v3).map(json::unit).unwrap_or(d.boresight);
            d.es = EsDesc { fitted: true, bs, noise: n("accuracy_rad"), fov: n("fov_half_angle_rad"), rate_hz: n("rate_Hz"), bias_sigma: pt.sig("bias_rad") };
        }
        "gnss" => d.gps = GpsDesc { fitted: true, pos_sigma: n("pos_sigma_m"), vel_sigma: n("vel_sigma_m_s"), rate_hz: n("rate_Hz"), latency: pt.nonneg("latency_s") },
        _ => return false,
    }
    true
}

impl Dev {
    pub fn load(root: &Path, id: &str) -> Result<Dev, Error> {
        let pf = find(root, "products", id)?;
        let pr = json::read(&pf)?;
        let mut d = Dev {
            id: json::s(&pr, "id", id).into(), label: json::s(&pr, "label", "").into(), family: json::s(&pr, "family", "").into(),
            algorithms: pr.get("algorithms").and_then(|a| a.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default(),
            selected: pr.get("selected").cloned(), boresight: [0.0, 1.0, 0.0], sun_axis: [0.0, 0.0, -1.0], ..Default::default()
        };
        if let Some(b) = get(&pr, "payload_boresight_body").and_then(json::v3) { d.boresight = json::unit(b); }
        if let Some(b) = get(&pr, "sun_axis_body").and_then(json::v3) { d.sun_axis = json::unit(b); }
        let fills: Vec<Value> = match pr.get("fill") { Some(Value::Array(a)) => a.clone(), Some(x) => vec![x.clone()], None => vec![] };
        let mut x = MexDesc { torque_noise: 0.001, friction_comp: 0.95, eta: 0.8, k_speed: 1.0, k_flow: 2.0, flow_tau: 0.3, ..Default::default() };
        let mut files = vec![pf];
        for f in &fills {
            let part = json::s(f, "part", "");
            let partf = find(root, "parts", part)?;
            let pt = Part::new(f, &json::read(&partf)?);
            files.push(partf);
            let slot = json::s(f, "slot", "");
            capacity(id, part, slot, f, &pt, &x)?;
            selector(id, slot, f)?;
            if !fit_actuator(&mut d, &mut x, slot, &pt) && !fit_sensor(&mut d, slot, &pt) {
                return Err(Error::refused(format!("product {id}: unknown slot {slot}")));
            }
            let miss = pt.missing.into_inner();
            if !miss.is_empty() {
                return Err(Error::refused(format!("part {part} ({slot} of product {id}) does not state {}, which the engine needs", miss.join(", "))));
            }
        }
        d.mex = x;
        d.files = files;
        Ok(d)
    }

    /// Capabilities the algorithm registry checks (asils.fsw.select caps_).
    pub fn caps(&self) -> Vec<&'static str> {
        let mut c = vec![];
        if self.mtq.fitted { c.push("coils"); }
        if self.mag.fitted { c.push("magnetometer"); }
        if self.gyro.fitted { c.push("gyro"); }
        let sun = self.sun.fitted || self.css.fitted;
        if sun { c.push("sun"); }
        if self.st.fitted { c.push("star_tracker"); }
        if self.gyro.fitted && (self.st.fitted || (self.mag.fitted && sun)) { c.push("attitude"); }
        if self.rcs.fitted { c.push("rcs"); }
        let x = &self.mex;
        if x.n > 0 {
            c.push("momentum");
            let k = &x.kind[..x.n];
            if k.iter().any(|&k| k == Kind::Rw || k == Kind::Fmr) { c.push("wheels_or_rings"); }
            if k.contains(&Kind::Fmr) { c.push("rings"); }
            if k.contains(&Kind::Cmg) { c.push("cmg"); }
            if k.contains(&Kind::Vscmg) { c.push("vscmg"); }
        }
        c
    }
}
