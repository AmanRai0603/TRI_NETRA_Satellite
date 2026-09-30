//! Product + parts -> device descriptors (asils.product.load).
use crate::json::{self, get};
use adcs_sim_core::actuators::{Kind, MexDesc, MtqDesc, RcsDesc};
use adcs_sim_core::sensors::*;
use serde_json::Value;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default)]
pub struct Dev {
    pub id: String, pub label: String, pub family: String, pub algorithms: Vec<String>, pub selected: Option<Value>,
    pub boresight: [f64; 3], pub sun_axis: [f64; 3],
    pub gyro: GyroDesc, pub mag: MagDesc, pub sun: SunDesc, pub css: CssDesc, pub st: StDesc, pub es: EsDesc, pub gps: GpsDesc,
    pub mtq: MtqDesc, pub mex: MexDesc, pub rcs: RcsDesc,
}

/// data/<kind>/<id>.json, else store/sized/*/<kind>/<id>.json.
/// $ADCS_SIZED_DIR/<kind>/<id>.json first when set: the design loop's current iteration
/// (tools/pipeline.py) flies its own sized products without touching the MATLAB ones.
pub fn find(root: &Path, kind: &str, id: &str) -> Result<PathBuf, String> {
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
    Err(format!("no {} record {id} (data/{kind} or store/sized/*/{kind})", kind.trim_end_matches('s')))
}

fn sig(ds: &Value, k: &str) -> f64 { ds.get(k).and_then(|x| x.get("sigma")).and_then(|x| x.as_f64()).unwrap_or(0.0) }
fn lohi(ds: &Value, k: &str, d: f64) -> (f64, f64) {
    let e = ds.get(k);
    (e.and_then(|x| x.get("lo")).and_then(|x| x.as_f64()).unwrap_or(d), e.and_then(|x| x.get("hi")).and_then(|x| x.as_f64()).unwrap_or(d))
}
fn axes(v: Option<&Value>) -> Vec<[f64; 3]> { v.map(json::vecs).unwrap_or_default().into_iter().map(json::unit).collect() }

impl Dev {
    pub fn load(root: &Path, id: &str) -> Result<Dev, String> {
        let pr = json::read(&find(root, "products", id)?)?;
        let mut d = Dev {
            id: json::s(&pr, "id", id).into(), label: json::s(&pr, "label", "").into(), family: json::s(&pr, "family", "").into(),
            algorithms: pr.get("algorithms").and_then(|a| a.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default(),
            selected: pr.get("selected").cloned(), boresight: [0.0, 1.0, 0.0], sun_axis: [0.0, 0.0, -1.0], ..Default::default()
        };
        if let Some(b) = get(&pr, "payload_boresight_body").and_then(json::v3) { d.boresight = json::unit(b); }
        if let Some(b) = get(&pr, "sun_axis_body").and_then(json::v3) { d.sun_axis = json::unit(b); }
        let fills: Vec<Value> = match pr.get("fill") { Some(Value::Array(a)) => a.clone(), Some(x) => vec![x.clone()], None => vec![] };
        let mut x = MexDesc { torque_noise: 0.001, friction_comp: 0.95, eta: 0.8, k_speed: 1.0, k_flow: 2.0, flow_tau: 0.3, ..Default::default() };
        let add = |x: &mut MexDesc, kind: Kind, a: [f64; 3], gi: usize, hmax: f64, tmax: f64, j: f64, cou: f64, vis: f64, pst: f64, tsig: f64, flo: f64, fhi: f64, mis: f64| {
            let i = x.n;
            x.kind[i] = kind; x.a0[i] = a; x.gi[i] = gi; x.h_max[i] = hmax; x.torque_max[i] = tmax; x.jrot[i] = j;
            x.coulomb[i] = cou; x.viscous[i] = vis; x.p_steady[i] = pst; x.tsig[i] = tsig; x.flo[i] = flo; x.fhi[i] = fhi; x.misalign[i] = mis;
            x.t_sd[i] = f64::INFINITY; x.k_hv[i] = 1.0; x.ac[i] = 1.0; x.s[i] = 1.0; x.l[i] = 1.0; x.eta_lo[i] = 1.0; x.eta_hi[i] = 1.0;
            x.n += 1;
        };
        for f in &fills {
            let part = json::s(f, "part", "");
            let p = json::read(&find(root, "parts", part)?)?;
            let nm = p.get("nominal").cloned().unwrap_or(Value::Null);
            let ds = p.get("dispersion").cloned().unwrap_or(Value::Null);
            let n = |k: &str| json::f(&nm, k, f64::NAN);
            match json::s(f, "slot", "") {
                "coils" => {
                    let a = axes(f.get("axes_body"));
                    let mut m = MtqDesc { fitted: true, n: a.len(), m_max: n("dipole_max_Am2"), p_max: n("power_at_max_W"), scale_sigma: sig(&ds, "dipole_scale"), misalign: sig(&ds, "axis_misalignment_rad"), ..Default::default() };
                    for (j, v) in a.iter().enumerate() { m.axes[j] = *v; }
                    d.mtq = m;
                }
                "wheels" => {
                    let (flo, fhi) = lohi(&ds, "friction_scale", 1.0);
                    for a in axes(f.get("axes_body")) {
                        add(&mut x, Kind::Rw, a, 0, n("h_max_Nms"), n("torque_max_Nm"), n("rotor_inertia_kgm2"), n("friction_coulomb_Nm"), n("friction_viscous_Nms"), n("power_steady_W"), sig(&ds, "torque_scale"), flo, fhi, sig(&ds, "axis_misalignment_rad"));
                    }
                }
                "rings" => {
                    let ac = std::f64::consts::PI*n("bore_m").powi(2)/4.0;
                    let s = n("enclosed_area_m2");
                    let k_hv = n("fluid_density_kg_m3")*ac*2.0*s;
                    let tsd = n("fluid_density_kg_m3")*n("bore_m").powi(2)/(32.0*n("fluid_viscosity_Pa_s"));
                    let hmax = k_hv*n("v_max_m_s");
                    let (flo, fhi) = lohi(&ds, "friction_scale", 1.0);
                    let (elo, ehi) = lohi(&ds, "pump_efficiency", 1.0);
                    let fp = json::f(&nm, "field_power_W", 0.0);
                    for a in axes(f.get("axes_body")) {
                        // an electromagnetic pump designed by adcs-design states its pressure-limited torque
                        let tq = json::f(&nm, "pump_torque_max_Nm", f64::NAN);
                        add(&mut x, Kind::Fmr, a, 0, hmax, if tq.is_finite() && tq > 0.0 { tq } else { 2.0*hmax/tsd }, k_hv, 0.0, 0.0, 0.0, 0.0, flo, fhi, sig(&ds, "axis_misalignment_rad"));
                        let i = x.n - 1;
                        x.t_sd[i] = tsd; x.k_hv[i] = k_hv; x.ac[i] = ac; x.s[i] = s; x.l[i] = n("channel_length_m");
                        x.flow_noise_h[i] = k_hv*sig(&ds, "flow_sensor_noise_m_s");
                        x.field_power[i] = if fp.is_nan() { 0.0 } else { fp };
                        x.eta_lo[i] = elo; x.eta_hi[i] = ehi;
                    }
                }
                slot @ ("cmg" | "vscmg") => {
                    let g = axes(f.get("gimbal_axes_body"));
                    let a = axes(f.get("spin_axes_body"));
                    let g0 = x.ng;
                    for (j, v) in g.iter().enumerate() { x.g[g0 + j] = *v; }
                    x.ng += g.len();
                    let (flo, fhi) = lohi(&ds, "friction_scale", 1.0);
                    for (k, v) in a.iter().enumerate() {
                        if slot == "cmg" {
                            add(&mut x, Kind::Cmg, *v, g0 + k + 1, n("rotor_momentum_Nms"), n("rotor_torque_max_Nm"), n("rotor_inertia_kgm2"), 0.0, 0.0, n("power_steady_W"), sig(&ds, "torque_scale"), 1.0, 1.0, sig(&ds, "axis_misalignment_rad"));
                        } else {
                            add(&mut x, Kind::Vscmg, *v, g0 + k + 1, n("h_max_Nms"), n("rotor_torque_max_Nm"), n("rotor_inertia_kgm2"), n("friction_coulomb_Nm"), n("friction_viscous_Nms"), n("power_steady_W"), sig(&ds, "torque_scale"), flo, fhi, sig(&ds, "axis_misalignment_rad"));
                        }
                        x.h0[x.n - 1] = n("rotor_momentum_Nms");
                    }
                    x.gimbal_rate_max = n("gimbal_rate_max_rad_s");
                    x.gimbal_power = n("gimbal_power_W");
                }
                "rcs" => {
                    let fth = n("thrust_N");
                    let arm = [n("arm_short_m"), n("arm_long_m"), n("arm_long_m")];
                    let mut r = RcsDesc { fitted: true, nc: 6, thrust: fth, isp: n("isp_s"), mib: n("mib_s"), res: n("valve_res_s"), prop_kg: n("propellant_kg"), valve_power: n("valve_power_W"), thrust_sigma: sig(&ds, "thrust_scale"), misalign: sig(&ds, "axis_misalignment_rad"), ..Default::default() };
                    (r.isp_lo, r.isp_hi) = lohi(&ds, "isp_s", n("isp_s"));
                    for ax in 0..3 { r.tau[2*ax][ax] = 2.0*fth*arm[ax]; r.tau[2*ax + 1][ax] = -2.0*fth*arm[ax]; }
                    d.rcs = r;
                }
                "star_tracker" => {
                    let bs = if f.get("boresights_body").is_some() { axes(f.get("boresights_body")) } else { axes(f.get("boresight_body")) };
                    let mut s = StDesc { fitted: true, nh: bs.len().min(2), noise_cross: n("noise_cross_rad"), noise_roll: n("noise_roll_rad"), rate_hz: n("rate_Hz"),
                        latency: n("latency_s"), max_rate: n("max_rate_rad_s"), sun_excl: n("sun_exclusion_rad"), earth_excl: n("earth_exclusion_rad"),
                        fov: n("fov_half_angle_rad"), model: if json::s(f, "model", "quest") == "noise" { 0 } else { 1 },
                        bias_sigma: sig(&ds, "bias_rad"), misalign_sigma: sig(&ds, "axis_misalignment_rad"), ..Default::default() };
                    for h in 0..s.nh { s.bs[h] = bs[h]; }
                    if let Some(c) = get(f, "calibrated_residual_rad").and_then(|v| v.as_f64()) { s.bias_sigma = c; s.misalign_sigma = 0.0; }
                    d.st = s;
                }
                "magnetometer" => d.mag = MagDesc { fitted: true, noise: n("noise_T_rms"), bias_t: n("bias_T"), bias_sigma: sig(&ds, "bias_T"), range: n("range_T"), sf_sigma: sig(&ds, "scale_factor"), misalign: sig(&ds, "axis_misalignment_rad"), k_coil: 5e-6 },
                "sun_sensors" => {
                    let a = axes(f.get("normals_body"));
                    let mut s = SunDesc { fitted: true, n: a.len(), noise: n("accuracy_rad"), fov: n("fov_half_angle_rad"), bias_sigma: sig(&ds, "bias_rad"), ..Default::default() };
                    for (j, v) in a.iter().enumerate() { s.normals[j] = *v; }
                    d.sun = s;
                }
                "coarse_sun_sensors" => {
                    let a = axes(f.get("normals_body"));
                    let mut s = CssDesc { fitted: true, n: a.len(), noise: n("noise_frac"), albedo: n("albedo"), scale_sigma: sig(&ds, "scale"), misalign: sig(&ds, "axis_misalignment_rad"), ..Default::default() };
                    for (j, v) in a.iter().enumerate() { s.normals[j] = *v; }
                    d.css = s;
                }
                "gyro" => d.gyro = GyroDesc { fitted: true, arw: n("arw_rad_per_sqrt_s"), rrw: n("rrw_rad_per_s_sqrt_s"), range: n("range_rad_s"), bias_sigma: sig(&ds, "bias_rad_s"), sf_sigma: sig(&ds, "scale_factor"), misalign: sig(&ds, "axis_misalignment_rad") },
                "earth_sensor" => {
                    let bs = get(f, "boresight_body").and_then(json::v3).map(json::unit).unwrap_or(d.boresight);
                    d.es = EsDesc { fitted: true, bs, noise: n("accuracy_rad"), fov: n("fov_half_angle_rad"), rate_hz: n("rate_Hz"), bias_sigma: sig(&ds, "bias_rad") };
                }
                "gnss" => d.gps = GpsDesc { fitted: true, pos_sigma: n("pos_sigma_m"), vel_sigma: n("vel_sigma_m_s"), rate_hz: n("rate_Hz") },
                other => return Err(format!("product {id}: unknown slot {other}")),
            }
        }
        d.mex = x;
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
