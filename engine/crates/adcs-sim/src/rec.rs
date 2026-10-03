//! adcs-rec/1 run directory: channels.csv (the columns of asils.rec.write, %.9g)
//! and manifest.json -- read by tools/report.py like a MATLAB-twin run.
use crate::config::Config;
use crate::metrics::Derived;
use crate::run::Record;
use serde_json::{json, Value};
use std::fmt::Write as _;
use std::path::Path;

/// C/MATLAB "%.9g".
pub fn g9(x: f64) -> String {
    if x.is_nan() { return "NaN".into(); }
    if x.is_infinite() { return if x > 0.0 { "Inf".into() } else { "-Inf".into() }; }
    if x == 0.0 { return if x.is_sign_negative() { "-0".into() } else { "0".into() }; }
    let e = format!("{:.8e}", x);
    let (mant, exp) = e.split_once('e').unwrap();
    let ex: i32 = exp.parse().unwrap();
    let strip = |s: &str| -> String { if s.contains('.') { s.trim_end_matches('0').trim_end_matches('.').to_string() } else { s.to_string() } };
    if (-4..9).contains(&ex) {
        strip(&format!("{:.*}", (8 - ex).max(0) as usize, x))
    } else {
        format!("{}e{}{:02}", strip(mant), if ex < 0 { '-' } else { '+' }, ex.abs())
    }
}

pub fn write(dir: &Path, c: &Config, rec: &Record, d: &Derived, metrics: &[Value]) -> Result<(), crate::Error> {
    std::fs::create_dir_all(dir).map_err(|e| crate::Error::io(dir, e))?;
    let rows = &rec.rows;
    let mut cols: Vec<(String, Vec<f64>)> = vec![];
    let mut col = |name: &str, f: &dyn Fn(usize) -> f64| cols.push((name.to_string(), (0..rows.len()).map(f).collect()));
    col("t_s", &|j| rows[j].t);
    for (i, a) in ["x", "y", "z", "w"].iter().enumerate() { col(&format!("q_{a}"), &|j| rows[j].q[i]); }
    for (i, a) in ["x", "y", "z"].iter().enumerate() { col(&format!("w_{a}_degps"), &|j| rows[j].w[i].to_degrees()); }
    col("rate_degps", &|j| d.rate[j]);
    col("ape_3ax_deg", &|j| d.ape_3ax[j]); col("ape_los_deg", &|j| d.ape_los[j]);
    col("ake_3ax_deg", &|j| d.ake_3ax[j]); col("ake_los_deg", &|j| d.ake_los[j]);
    col("rks_degps", &|j| d.rks[j]);
    col("mode", &|j| rows[j].mode as f64 + 1.0);
    for (i, a) in ["x", "y", "z"].iter().enumerate() { col(&format!("m_{a}_Am2"), &|j| rows[j].m[i]); }
    for (p, name) in ["gg", "aero", "srp", "mag"].iter().enumerate() {
        for (i, a) in ["x", "y", "z"].iter().enumerate() { col(&format!("tau_{name}_{a}_Nm"), &|j| rows[j].tau_dist[p][i]); }
    }
    let nr = rec.nr;
    for (i, a) in ["x", "y", "z"].iter().enumerate() {
        col(&format!("tau_mtq_{a}_Nm"), &|j| rows[j].tau_mtq[i]);
        col(&format!("tau_rw_{a}_Nm"), &|j| if nr > 0 { rows[j].tau_rw[i] } else { f64::NAN });
        col(&format!("B_{a}_T"), &|j| rows[j].b[i]);
        col(&format!("r_{a}_m"), &|j| rows[j].r[i]);
        col(&format!("sun_{a}"), &|j| rows[j].sun_eci[i]);
    }
    for k in 0..nr.max(1) { col(&format!("h_w{}_Nms", k + 1), &|j| if nr > 0 { rows[j].h_w[k] } else { f64::NAN }); }
    for k in 0..rec.ng { col(&format!("gimbal{}_rad", k + 1), &|j| rows[j].delta[k]); }
    for (i, a) in ["x", "y", "z"].iter().enumerate() { col(&format!("tau_rcs_{a}_Nm"), &|j| rows[j].tau_rcs[i]); }
    col("prop_kg", &|j| rows[j].prop_kg);
    col("P_rcs_W", &|j| rows[j].p_rcs);
    col("n_rotors_isolated", &|j| if nr > 0 { rows[j].n_failed as f64 } else { f64::NAN });
    for (i, a) in ["x", "y", "z"].iter().enumerate() { col(&format!("sun_body_{a}"), &|j| rows[j].sun_body[i]); }
    col("P_mtq_W", &|j| rows[j].p_mtq); col("P_rw_W", &|j| rows[j].p_rw);
    col("rho_kgm3", &|j| rows[j].rho); col("shadow_nu", &|j| rows[j].nu);
    col("P_gen_W", &|j| d.p_gen[j]); col("soc", &|j| d.soc[j]);
    col("sun_ok", &|j| rows[j].sun_ok as u8 as f64); col("st_ok", &|j| rows[j].st_ok as u8 as f64); col("ad_ok", &|j| rows[j].ad_ok as u8 as f64);

    let mut s = String::with_capacity(rows.len()*cols.len()*12);
    s.push_str(&cols.iter().map(|c| c.0.as_str()).collect::<Vec<_>>().join(","));
    s.push('\n');
    for j in 0..rows.len() {
        for (i, c) in cols.iter().enumerate() { if i > 0 { s.push(','); } let _ = write!(s, "{}", g9(c.1[j])); }
        s.push('\n');
    }
    crate::fsio::write(&dir.join("channels.csv"), s)?;

    let mut alg = serde_json::Map::new();
    for (k, v) in &c.alg { alg.insert(k.clone(), json!(v)); }
    let mut man = json!({
        "schema": "adcs-rec/1", "engine": crate::ENGINE, "owner": "Agastya",
        "scenario": c.id, "case": c.case.id, "case_title": c.case.title, "product": c.dev.id, "family": c.dev.family,
        "label": crate::json::s(&c.scenario, "label", ""), "algorithms": alg, "seed": c.seed, "epoch_utc": c.epoch_utc,
        "duration_s": c.duration_s, "dt_s": c.dt, "wall_s": rec.wall_s,
        "fsw": {"impl": impl_label(&rec.fsw_impl), "build_id": rec.fsw_build},
        "orbit": {"alt_km": c.alt_km, "inc_deg": c.inc_deg, "ltan_h": c.ltan_h, "raan_deg": rec.raan_rad.to_degrees(), "period_s": c.period_s,
                  "atmosphere": if c.orbit_model == "pop" { format!("dtm2020 (F10.7 {}, Kp {})", c.f107, c.kp) } else { format!("exponential x{}", c.density_scale) },
                  "propagator": if c.orbit_model == "pop" { "POP v51 port (adcs-pop): degree-6 field, DE440 Sun/Moon (Battin), DTM2020 drag, conical SRP, RK4 10 s + Hermite" } else { "analytic (adcs-sim-core): J2-J6, Montenbruck-Gill Sun/Moon, exponential drag, SRP" }},
        "boresight_body": c.dev.boresight, "metrics": metrics,
        // values the engine uses that no case or part states yet (docs/UPGRADE_PLAN.md B2.7): named, not hidden
        "assumptions": {"body_box_m": c.box_m, "cm_offset_m": c.cm_offset_m, "accommodation_normal": c.sigma_n, "accommodation_tangential": c.sigma_t,
                        "vb_ratio": c.vb_ratio, "specular_fraction": c.spec_frac, "magnetometer_coil_coupling_T_per_Am2": c.dev.mag.k_coil,
                        "note": "fixed in the engine for a 3U body; to come from the case and the parts"},
        "mode_log": rec.mode_log.iter().map(|(t, m)| json!({"t": t, "mode": m})).collect::<Vec<_>>(),
        "oils": rec.oils.as_ref().map(|s| s.json(c.dt)),
    });
    if let (Some(o), Value::Object(p)) = (man.as_object_mut(), crate::store::provenance(c, &impl_label(&rec.fsw_impl), &impl_id(&rec.fsw_impl))) { o.extend(p); }
    // the inputs it flew, kept once by fingerprint, so it can be flown again exactly
    let inputs = crate::store::inputs_dir(dir);
    for (kind, file, key, ext) in [("case", &c.case.file, "case_fingerprint", "csv"), ("scenario", &c.scenario_file, "scenario_file_fingerprint", "json")] {
        let bytes = std::fs::read(file).map_err(|e| crate::Error::io(std::path::Path::new(file), e))?;
        let fp = man["inputs"][key].as_str().unwrap_or_default().to_string();
        crate::store::keep_input(&inputs, &crate::store::input_name(kind, &fp, ext), &bytes)?;
    }
    // the manifest last: a run directory with a manifest has its channels too
    crate::fsio::write(&dir.join("manifest.json"), serde_json::to_string(&man).map_err(|e| crate::Error::run(e.to_string()))? + "\n")?;
    Ok(())
}

#[cfg(test)]
mod t {
    #[test]
    fn g9_matches_printf() {
        for (x, s) in [(0.5, "0.5"), (10.0, "10"), (-2.59478359, "-2.59478359"), (4.46144792e-06, "4.46144792e-06"), (-6798619.52, "-6798619.52"),
                       (123456789.0, "123456789"), (1234567891.0, "1.23456789e+09"), (1e-5, "1e-05"), (0.0001, "0.0001")] {
            assert_eq!(super::g9(x), s);
        }
    }
}

/// Short label of where the flight software ran.
/// The --fsw value that flies this flight software again (the named targets by name).
pub fn impl_id(i: &adcs_fsw_abi::Impl) -> String {
    use adcs_fsw_abi::{link::Target, Impl};
    let ends = |x: &str, f: &str| x.replace('\\', "/").ends_with(&format!("fsw/build/{f}"));
    match i {
        Impl::C => "c".into(),
        Impl::Rust => "rust".into(),
        Impl::Obc(Target::Spawn(c)) if c[0].contains("qemu-system") && c.last().is_some_and(|x| ends(x, "obc_qemu.elf")) => "qemu".into(),
        Impl::Obc(Target::Spawn(c)) if c[0].contains("qemu-system") && c.last().is_some_and(|x| ends(x, "obc_qemu_rs.elf")) => "qemu-rs".into(),
        Impl::Obc(Target::Spawn(c)) if c.len() == 1 && ends(&c[0], "obc_posix") => "obc-posix".into(),
        Impl::Obc(Target::Spawn(c)) if c.len() == 1 && ends(&c[0], "obc_posix_rs") => "obc-posix-rs".into(),
        Impl::Obc(Target::Spawn(c)) => format!("spawn:{}", c.join(" ")),
        Impl::Obc(Target::Tcp(a)) => format!("tcp:{a}"),
    }
}

pub fn impl_label(i: &adcs_fsw_abi::Impl) -> String {
    use adcs_fsw_abi::{link::Target, Impl};
    match i {
        Impl::C => "c (in-process)".into(),
        Impl::Rust => "rust (in-process)".into(),
        Impl::Obc(Target::Spawn(c)) if c[0].contains("qemu-system") => format!("virtual OBC: QEMU mps2-an386 Cortex-M4F, {}",
            c.last().map(|x| x.rsplit('/').next().unwrap_or(x)).unwrap_or("?")),
        Impl::Obc(Target::Spawn(c)) => format!("virtual OBC: {}", c.iter().map(|x| x.rsplit('/').next().unwrap_or(x)).collect::<Vec<_>>().join(" ")),
        Impl::Obc(Target::Tcp(a)) => format!("OBC at tcp:{a}"),
    }
}
