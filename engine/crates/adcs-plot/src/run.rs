//! The figures of one run, engine or MATLAB twin (both write manifest.json and channels.csv with
//! the same columns): 1 attitude, 2 disturbances, 3 actuators and power, 7 Sun spin when the run
//! spun up, and with `full` 4 environment, 5 ground track, 6 modes. The figure set of
//! tools/report_runs.py, which now asks `adcs figures` for it.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use crate::*;
use serde_json::Value;

/// A run's time series: the header's names, a column of numbers each (NaN where empty or not a number).
#[derive(Clone, Debug, Default)]
pub struct Channels { pub names: Vec<String>, pub cols: Vec<Vec<f64>> }

impl Channels {
    pub fn parse(csv: &str) -> Result<Channels, String> {
        let mut lines = csv.lines();
        let names: Vec<String> = lines.next().ok_or("channels.csv is empty")?.split(',').map(|s| s.trim().to_string()).collect();
        let mut cols = vec![Vec::new(); names.len()];
        for (i, l) in lines.enumerate() {
            if l.trim().is_empty() { continue; }
            let mut k = 0;
            for (c, x) in cols.iter_mut().zip(l.split(',')) { c.push(x.trim().parse::<f64>().unwrap_or(f64::NAN)); k += 1; }
            if k != names.len() { return Err(format!("channels.csv line {}: {k} values for {} columns", i + 2, names.len())); }
        }
        Ok(Channels { names, cols })
    }
    pub fn len(&self) -> usize { self.cols.first().map_or(0, Vec::len) }
    pub fn is_empty(&self) -> bool { self.len() == 0 }
    pub fn has(&self, k: &str) -> bool { self.names.iter().any(|n| n == k) }
    /// A column, or NaN throughout when the run has no such channel.
    pub fn col(&self, k: &str) -> Vec<f64> {
        self.names.iter().position(|n| n == k).map(|i| self.cols[i].clone()).unwrap_or_else(|| vec![f64::NAN; self.len()])
    }
    fn any_finite(&self, k: &str) -> bool { self.has(k) && self.col(k).iter().any(|x| x.is_finite()) }
}

/// A manifest's metrics as a list (the twin writes a one-element struct array as an object).
pub fn metrics(man: &Value) -> Vec<Value> {
    match man.get("metrics") { Some(Value::Array(a)) => a.clone(), Some(o @ Value::Object(_)) => vec![o.clone()], _ => vec![] }
}

/// The requirement a metric of the run is judged against (report_base.case_req).
pub fn case_req(man: &Value, key: &str) -> Option<f64> {
    metrics(man).iter().find(|m| m.get("req_key").and_then(Value::as_str) == Some(key) && m.get("req").is_some_and(|r| !r.is_null()))
        .and_then(|m| m["req"].as_f64())
}

fn req_line(p: &mut Panel, y: Option<f64>, label: &str) {
    if let Some(y) = y.filter(|y| y.is_finite()) { p.refs.push(RefLine::h(y, &format!("{label} {}", fmt_g(y, 6)))); }
}

const IN: f64 = 72.0;
pub const MODES: [&str; 7] = ["detumble", "nadir_mtq", "nadir_fine", "target_fine", "slew_fine", "spinup", "sun_spin"];

fn hyp(a: &[f64], b: &[f64], c: Option<&[f64]>) -> Vec<f64> {
    (0..a.len()).map(|i| (a[i] * a[i] + b[i] * b[i] + c.map_or(0.0, |c| c[i] * c[i])).sqrt()).collect()
}

/// The run's figures, each with the name its file ends in ("1_attitude", ...), in report_runs.py's order.
pub fn run_figures(man: &Value, ch: &Channels, full: bool) -> Vec<(String, Figure)> {
    let sv = |k: &str| man.get(k).and_then(Value::as_str).unwrap_or("").to_string();
    let title = format!("{}  ·  {}  ·  {}", sv("scenario"), sv("case"), sv("product"));
    let t: Vec<f64> = ch.col("t_s").iter().map(|x| x / 60.0).collect();
    let tm = "time [min]";
    let c = |k: &str| ch.col(k);
    let mut out = Vec::new();

    // 1 pointing and knowledge, or the body rate when the run does not point
    let f1 = if ch.any_finite("ape_los_deg") {
        let mut a = Panel::new(&format!("{title}\nabsolute pointing error vs the true target (precision orbit)")).logy().ylabel("APE [deg]")
            .with(Series::line(&t, &c("ape_los_deg"), S1).label("payload line of sight"))
            .with(Series::line(&t, &c("ape_3ax_deg"), S2).width(1.1).label("3-axis")).legend(Corner::UpperRight);
        req_line(&mut a, case_req(man, "req.ape"), "APE req");
        let mut b = Panel::new("absolute knowledge error (MEKF vs truth)").logy().ylabel("AKE [deg]").xlabel(tm)
            .with(Series::line(&t, &c("ake_los_deg"), S1).label("payload line of sight"))
            .with(Series::line(&t, &c("ake_3ax_deg"), S2).width(1.1).label("3-axis")).legend(Corner::UpperRight);
        req_line(&mut b, case_req(man, "req.ake"), "AKE req");
        vec![a, b]
    } else {
        let a = Panel::new(&format!("{title}\nbody rate magnitude")).logy().ylabel("|ω| [deg/s]")
            .with(Series::line(&t, &c("rate_degps"), S1)).refl(RefLine::h(0.5, "detumble threshold 0.5 deg/s"));
        let mut b = Panel::new("body rate components").ylabel("ω [deg/s]").xlabel(tm).legend(Corner::UpperRight);
        for (col, n) in [(S1, "x"), (S2, "y"), (S3, "z")] { b = b.with(Series::line(&t, &c(&format!("w_{n}_degps")), col).width(1.0).label(&format!("ω{n}"))); }
        vec![a, b]
    };
    out.push(("1_attitude".to_string(), Figure::stack(10.0 * IN, 6.2 * IN, &[], f1)));

    // 2 disturbance torques by source
    let mut p = Panel::new(&format!("{title}\ndisturbance torques driven by the in-loop precision orbit")).logy().ylabel("|τ| [N m]").xlabel(tm).legend(Corner::LowerRight);
    for (col, n, lab) in [(S1, "gg", "gravity gradient"), (S2, "aero", "aerodynamic (facets)"), (S3, "srp", "solar radiation (facets)"), (S4, "mag", "residual dipole")] {
        let m: Vec<f64> = hyp(&c(&format!("tau_{n}_x_Nm")), &c(&format!("tau_{n}_y_Nm")), Some(&c(&format!("tau_{n}_z_Nm")))).into_iter().map(|v| v.max(1e-14)).collect();
        p = p.with(Series::line(&t, &m, col).width(1.2).label(lab));
    }
    out.push(("2_disturbances".to_string(), Figure::stack(10.0 * IN, 3.8 * IN, &[], vec![p])));

    // 3 actuators and power, one panel per unit
    let pal = [S1, S2, S3, S4];
    let mut ps = Vec::new();
    let mut d = Panel::new(&format!("{title}\nactuators: magnetorquer dipole")).ylabel("dipole [A m²]").legend(Corner::UpperRight);
    for (col, a) in [(S1, "x"), (S2, "y"), (S3, "z")] { d = d.with(Series::line(&t, &c(&format!("m_{a}_Am2")), col).width(0.9).label(&format!("m{a}"))); }
    ps.push(d);
    if ch.any_finite("h_w1_Nms") {
        let mut h = Panel::new("momentum-exchange devices: stored momentum").ylabel("h [mN m s]").legend(Corner::UpperRight);
        let mut i = 1;
        while ch.has(&format!("h_w{i}_Nms")) {
            let v: Vec<f64> = c(&format!("h_w{i}_Nms")).iter().map(|x| x * 1e3).collect();
            h = h.with(Series::line(&t, &v, pal[(i - 1) % 4]).label(&format!("rotor {i}")));
            i += 1;
        }
        ps.push(h);
    }
    if ch.has("gimbal1_rad") {
        let mut g = Panel::new("CMG gimbal angles").ylabel("gimbal [deg]").legend(Corner::UpperRight);
        let mut i = 1;
        while ch.has(&format!("gimbal{i}_rad")) {
            let v: Vec<f64> = c(&format!("gimbal{i}_rad")).iter().map(|x| x.to_degrees()).collect();
            g = g.with(Series::line(&t, &v, pal[(i - 1) % 4]).label(&format!("gimbal {i}")));
            i += 1;
        }
        ps.push(g);
    }
    if ch.has("prop_kg") && c("prop_kg").iter().any(|x| *x > 0.0) {
        let v: Vec<f64> = c("prop_kg").iter().map(|x| x * 1e3).collect();
        ps.push(Panel::new("cold-gas propellant used").ylabel("propellant [g]").with(Series::line(&t, &v, S1)));
    }
    let (pm, pr, pc) = (c("P_mtq_W"), c("P_rw_W"), if ch.has("P_rcs_W") { c("P_rcs_W") } else { vec![0.0; ch.len()] });
    let tot: Vec<f64> = (0..ch.len()).map(|i| pm[i] + pr[i] + pc[i]).collect();
    let mut pw = Panel::new("actuator power (cycle average)").ylabel("ADCS power [W]").xlabel(tm).with(Series::line(&t, &tot, S1));
    req_line(&mut pw, case_req(man, "req.pavg"), "orbit-average req");
    ps.push(pw);
    let h = 2.3 * ps.len() as f64 + 0.6;
    out.push(("3_actuators".to_string(), Figure::stack(10.0 * IN, h * IN, &[], ps)));

    // 7 Sun spin: -Z_B to Sun angle, spin rate, modes
    let mode = c("mode");
    if ch.has("sun_body_z") && mode.iter().any(|m| *m >= 6.0) {
        let (nu, sz) = (c("shadow_nu"), c("sun_body_z"));
        let ang: Vec<f64> = (0..ch.len()).map(|i| if nu[i] > 0.5 { (-sz[i]).clamp(-1.0, 1.0).acos().to_degrees() } else { f64::NAN }).collect();
        let a = Panel::new(&format!("{title}\nSun acquisition with coils only: Sun angle (sunlit samples; dashed θ_ok 20°)")).ylabel("−Z_B to Sun [deg]").ylim(0.0, 180.0)
            .with(Series::line(&t, &ang, S1).width(1.0)).refl(RefLine { width: 1.2, ..RefLine::h(20.0, "") });
        let b = Panel::new("").ylabel("rate [deg/s]").legend(Corner::UpperRight).with(Series::line(&t, &c("w_z_degps"), S1).label("ω_z (spin)"))
            .with(Series::line(&t, &hyp(&c("w_x_degps"), &c("w_y_degps"), None), S2).width(1.0).label("|ω_xy| (nutation)"));
        let mut m = Panel::new("").xlabel(tm).with(Series::step(&t, &mode, S1));
        m.yticks = vec![(1.0, "detumble".into()), (6.0, "spinup".into()), (7.0, "sun_spin".into())];
        out.push(("7_sunspin".to_string(), Figure::stack(10.0 * IN, 7.0 * IN, &[2.0, 2.0, 1.0], vec![a, b, m])));
    }
    if !full { return out; }

    // 4 environment along the precision orbit
    let a = Panel::new(&format!("{title}\nenvironment seen along the POP orbit (DTM2020 density, DE440 Sun, IGRF)")).logy().ylabel("ρ [kg/m³]").with(Series::line(&t, &c("rho_kgm3"), S1));
    let b = Panel::new("").ylabel("sunlit fraction ν").ylim(-0.05, 1.05).with(Series::line(&t, &c("shadow_nu"), S1));
    let bm: Vec<f64> = hyp(&c("B_x_T"), &c("B_y_T"), Some(&c("B_z_T"))).iter().map(|x| x * 1e9).collect();
    let cpan = Panel::new("").ylabel("|B| [nT]").xlabel(tm).with(Series::line(&t, &bm, S1));
    out.push(("4_environment".to_string(), Figure::stack(10.0 * IN, 6.5 * IN, &[], vec![a, b, cpan])));

    // 5 ground track: latitude and longitude from ECI with GMST
    let ep: Vec<f64> = man.get("epoch_utc").and_then(Value::as_array).map(|a| a.iter().map(|x| x.as_f64().unwrap_or(0.0)).collect()).unwrap_or_default();
    let e = |i: usize| ep.get(i).copied().unwrap_or(0.0);
    let tr = |x: f64| x.trunc();
    let jd = 367.0 * e(0) - tr(7.0 * (e(0) + tr((e(1) + 9.0) / 12.0)) / 4.0) + tr(275.0 * e(1) / 9.0) + e(2) + 1721013.5 + ((e(5) / 60.0 + e(4)) / 60.0 + e(3)) / 24.0;
    let (rx, ry, rz, ts, nu) = (c("r_x_m"), c("r_y_m"), c("r_z_m"), c("t_s"), c("shadow_nu"));
    let (mut lit, mut dark) = ((vec![], vec![]), (vec![], vec![]));
    for i in 0..ch.len() {
        let gm = (280.46061837 + 360.98564736629 * (jd + ts[i] / 86400.0 - 2451545.0)).rem_euclid(360.0);
        let lon = (ry[i].atan2(rx[i]).to_degrees() - gm + 180.0).rem_euclid(360.0) - 180.0;
        let lat = (rz[i] / (rx[i] * rx[i] + ry[i] * ry[i] + rz[i] * rz[i]).sqrt()).asin().to_degrees();
        let w = if nu[i] > 0.5 { &mut lit } else { &mut dark };
        w.0.push(lon);
        w.1.push(lat);
    }
    let o = man.get("orbit").cloned().unwrap_or(Value::Null);
    let of = |k: &str| o.get(k).and_then(Value::as_f64).unwrap_or(f64::NAN);
    let mut g = Panel::new(&format!("{title}\nground track — {:.0} km SSO, i {:.3}°, LTAN {:05.2} h", of("alt_km"), of("inc_deg"), of("ltan_h")))
        .xlabel("longitude [deg]").ylabel("latitude [deg]").ylim(-90.0, 90.0).legend(Corner::LowerLeft)
        .with(Series::scatter(&lit.0, &lit.1, S1).label("sunlit")).with(Series::scatter(&dark.0, &dark.1, INK2).label("eclipse"));
    g.xlim = Some((-180.0, 180.0));
    g.xticks = (-180..=180).step_by(60).map(|v| (v as f64, v.to_string())).collect();
    g.yticks = (-90..=90).step_by(30).map(|v| (v as f64, v.to_string())).collect();
    out.push(("5_groundtrack".to_string(), Figure::stack(10.0 * IN, 4.6 * IN, &[], vec![g])));

    // 6 mode timeline and rate
    let mut m = Panel::new(&format!("{title}\nmode timeline and body rate")).with(Series::step(&t, &mode, S1));
    m.yticks = MODES.iter().enumerate().map(|(i, s)| ((i + 1) as f64, s.to_string())).collect();
    let r = Panel::new("").logy().ylabel("|ω| [deg/s]").xlabel(tm).with(Series::line(&t, &c("rate_degps"), S1));
    out.push(("6_modes".to_string(), Figure::stack(10.0 * IN, 4.8 * IN, &[1.0, 2.0], vec![m, r])));
    out
}

#[cfg(test)]
pub(crate) mod t {
    use super::*;
    use serde_json::json;

    pub fn synthetic(n: usize, spin: bool) -> (Value, Channels) {
        let man = json!({"scenario": "detumble_ais", "case": "ais_3u", "product": "TRN-P-3U-AIS", "epoch_utc": [2027, 1, 1, 6, 0, 0],
                         "orbit": {"alt_km": 550.0, "inc_deg": 97.593, "ltan_h": 6.0},
                         "metrics": {"id": "power_mean", "value": 0.02, "unit": "W", "req": 0.5, "req_key": "req.pavg", "pass": 1}});
        let mut csv = String::from("t_s,rate_degps,w_x_degps,w_y_degps,w_z_degps,ape_los_deg,mode,m_x_Am2,m_y_Am2,m_z_Am2,P_mtq_W,P_rw_W,h_w1_Nms,\
                                    r_x_m,r_y_m,r_z_m,shadow_nu,sun_body_z,rho_kgm3,B_x_T,B_y_T,B_z_T,tau_gg_x_Nm,tau_gg_y_Nm,tau_gg_z_Nm\n");
        for i in 0..n {
            let t = i as f64 * 0.2;
            let a = t / 5739.0 * std::f64::consts::TAU;
            let mode = if spin && i > n / 2 { 7 } else { 1 };
            csv += &format!("{t},{},{},0.1,-0.2,,{mode},0.1,-0.1,0,{},0,NaN,{},{},{},{},{},1e-13,2e-5,1e-5,-3e-5,1e-8,0,-2e-9\n",
                            10.0 * (-t / 2000.0).exp(), (t / 50.0).sin(), 0.02 + 0.01 * (t / 300.0).sin(),
                            6.9e6 * a.cos(), 6.9e6 * a.sin() * 0.13, 6.9e6 * a.sin() * 0.99, if a.cos() > -0.3 { 1 } else { 0 }, a.sin());
        }
        (man, Channels::parse(&csv).unwrap())
    }

    #[test]
    fn channels_read_by_name_with_nan_for_empty() {
        let c = Channels::parse("a,b\n1,\n2,x\n").unwrap();
        assert_eq!(c.col("a"), [1.0, 2.0]);
        assert!(c.col("b").iter().all(|x| x.is_nan()) && c.col("zz").iter().all(|x| x.is_nan()) && c.len() == 2);
        assert!(Channels::parse("a,b\n1\n").unwrap_err().contains("line 2"));
    }

    #[test]
    fn a_run_has_its_figure_set() {
        let (man, ch) = synthetic(17000, true);
        let f = run_figures(&man, &ch, true);
        let names: Vec<&str> = f.iter().map(|x| x.0.as_str()).collect();
        assert_eq!(names, ["1_attitude", "2_disturbances", "3_actuators", "7_sunspin", "4_environment", "5_groundtrack", "6_modes"]);
        assert_eq!(run_figures(&man, &ch, false).len(), 4);
        assert_eq!(case_req(&man, "req.pavg"), Some(0.5));
        for (n, fig) in &f {
            let s = fig.to_svg();
            crate::svg::t::well_formed(&s).unwrap_or_else(|e| panic!("{n}: {e}"));
            assert!(s.len() < 400_000, "{n}: {} bytes for 17000 samples", s.len());
        }
        assert!(f[2].1.to_svg().contains("orbit-average req 0.5"), "the requirement line");
        assert!(crate::pdf::t::check(&pdf_of(&f.into_iter().map(|x| x.1).collect::<Vec<_>>())).is_ok());
    }
}
