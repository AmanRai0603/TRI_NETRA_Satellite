//! adcs -- the engine's command line.
//!
//!   adcs run <scenario> [--case F] [--fsw c|rust] [--seed N] [--out DIR] [--set k=v]... [--alg slot=id]... [--quiet]
//!   adcs params <scenario> [--case F] [--set k=v]... --out blob.bin     the adcs-fswcfg/1 blob (OILS / OBC upload)
//!   adcs size <case> [--knobs k.json] [--out DIR]                        demand survey + every actuator option sized (adcs-design)
//!   adcs parity <scenario> [--fsw A --against B] ...                  two flight-software targets, same loop, same bytes
//!
//! --fsw: c | rust (in-process) | obc-posix | obc-posix-rs (virtual OBC process) | qemu | qemu-rs
//! (virtual Cortex-M4 OBC in QEMU) | spawn:<cmd> | tcp:<host:port> (a real OBC); --realtime paces ticks to wall time.
//! --oils: soft OILS -- each command lands after the OBC's execution (exact QEMU instruction count x
//! --cpi / --obc-mhz) plus its bus time (--i2c-khz, --can-kbps), inside the control period (docs/SOFT_OILS.md).
//!
//! Scenario ids are matlab_sils/data/scenarios/*.json (or a path); the case defaults to
//! cases/<scenario.case>.csv. Overrides: `--set fsw.rw_bandwidth=0.5` edits the scenario,
//! `--set engine.duration_s=600` (also density_scale, orbit_step_s, zonal_max, igrf_nmax) the engine.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use adcs_fsw_abi::Impl;
use adcs_sim::{config::Config, data_root, metrics, rec, run};
use std::path::PathBuf;
use std::process::ExitCode;

struct Args { cmd: String, scenario: String, case: Option<String>, fsw: Impl, fsw_b: Option<Impl>, seed: u64, out: Option<PathBuf>, set: Vec<(String, String)>, quiet: bool, realtime: bool,
    oils: Option<run::OilsModel> }

fn parse() -> Result<Args, String> {
    let mut it = std::env::args().skip(1);
    let cmd = it.next().ok_or_else(usage)?;
    if cmd == "-h" || cmd == "--help" { return Err(usage()); }
    let scenario = it.next().ok_or_else(usage)?;
    let mut a = Args { cmd, scenario, case: None, fsw: Impl::C, fsw_b: None, seed: 1, out: None, set: vec![], quiet: false, realtime: false, oils: None };
    while let Some(k) = it.next() {
        let mut val = || it.next().ok_or(format!("{k} needs a value"));
        let num = |s: String| s.parse::<f64>().map_err(|_| format!("{k}: not a number"));
        match k.as_str() {
            "--oils" => { a.oils.get_or_insert_with(Default::default); }
            "--latency-ms" => { let x = num(val()?)?; a.oils.get_or_insert_with(Default::default).fixed_s = Some(x*1e-3); }
            "--obc-mhz" => { let x = num(val()?)?; a.oils.get_or_insert_with(Default::default).cpu_hz = x*1e6; }
            "--cpi" => { let x = num(val()?)?; a.oils.get_or_insert_with(Default::default).cpi = x; }
            "--i2c-khz" => { let x = num(val()?)?; a.oils.get_or_insert_with(Default::default).i2c_hz = x*1e3; }
            "--can-kbps" => { let x = num(val()?)?; a.oils.get_or_insert_with(Default::default).can_bps = x*1e3; }
            "--case" => a.case = Some(val()?),
            "--fsw" => a.fsw = val()?.parse()?,
            "--against" => a.fsw_b = Some(val()?.parse()?),
            "--realtime" => a.realtime = true,
            "--seed" => a.seed = val()?.parse().map_err(|_| "--seed: not an integer")?,
            "--out" => a.out = Some(val()?.into()),
            "--set" => { let s = val()?; let (k, v) = s.split_once('=').ok_or("--set k=v")?; a.set.push((k.into(), v.into())); }
            "--alg" => { let s = val()?; let (k, v) = s.split_once('=').ok_or("--alg slot=id")?; a.set.push((format!("fsw.algorithms.{k}"), format!("\"{v}\""))); }
            "--knobs" => { let f = val()?; a.set.push(("knobs".into(), f)); }
            "--quiet" | "-q" => a.quiet = true,
            _ => return Err(format!("unknown option {k}\n{}", usage())),
        }
    }
    Ok(a)
}

fn usage() -> String {
    "usage: adcs run|params|parity <scenario> [--case F] [--fsw c|rust|obc-posix|obc-posix-rs|qemu|qemu-rs|spawn:<cmd>|tcp:<host:port>] [--against <fsw>] [--realtime] [--oils [--obc-mhz F] [--cpi C] [--i2c-khz K] [--can-kbps B]] [--seed N] [--out DIR] [--set k=v]... [--alg slot=id]... [--quiet]".into()
}

fn config(a: &Args) -> Result<Config, String> {
    let root = data_root();
    let case = match &a.case {
        Some(c) => PathBuf::from(c),
        None => {
            if !a.scenario.ends_with(".json") { adcs_sim::config::check_id("scenario", &a.scenario)?; }
            let sp = if a.scenario.ends_with(".json") { PathBuf::from(&a.scenario) } else { root.join("data/scenarios").join(format!("{}.json", a.scenario)) };
            let s: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&sp).map_err(|e| format!("{}: {e}", sp.display()))?).map_err(|e| e.to_string())?;
            root.join("cases").join(format!("{}.csv", s["case"].as_str().unwrap_or("ais_3u")))
        }
    };
    Config::build(&root, &a.scenario, &case, a.seed, &a.set)
}

fn print_metrics(ms: &[serde_json::Value]) {
    for m in ms {
        let v = m["value"].as_f64().map(|x| format!("{x:.4}")).unwrap_or("NaN".into());
        let r = m["req"].as_f64().map(|x| format!("{x}")).unwrap_or("-".into());
        let p = match m["pass"].as_i64() { Some(1) => "PASS", Some(_) => "FAIL", None => "" };
        println!("  {:<24} {:>12} {:<6} req {:<8} {}", m["id"].as_str().unwrap_or(""), v, m["unit"].as_str().unwrap_or(""), r, p);
    }
}

fn main() -> ExitCode {
    adcs_sim::fsio::install_crash_report("adcs");
    let a = match parse() { Ok(a) => a, Err(e) => { eprintln!("{e}"); return ExitCode::from(2); } };
    let r = (|| -> Result<(), String> {
        if a.cmd == "size" {
            // adcs size <case> [--out DIR] [--knobs knobs.json]: demand survey + every option sized (adcs-design)
            let root = data_root();
            adcs_sim::config::check_id("case", &a.scenario)?;
            let case_file = root.join("cases").join(format!("{}.csv", a.scenario));
            if !case_file.is_file() { return Err(format!("no case {}: {} does not exist", a.scenario, case_file.display())); }
            let knobs = match a.set.iter().find(|(k, _)| k == "knobs") {
                Some((_, f)) => adcs_design::Knobs::from_json(&adcs_sim::json::read(std::path::Path::new(f))?),
                None => adcs_design::Knobs::default(),
            };
            let out = a.out.clone().unwrap_or_else(|| root.join("store/design").join(&a.scenario).join("sized"));
            let z = adcs_design::size_all(&root, &case_file, &knobs, &out)?;
            let d = &z["demand"];
            println!("[size] {} ({} class): tau_dist {:.3e} N m, h_req {:.3e} N m s, tau_req {:.3e} N m, B_min {:.3e} T -> {}",
                a.scenario, z["class"].as_str().unwrap_or(""), d["tau_dist"].as_f64().unwrap_or(0.0), d["h_req"].as_f64().unwrap_or(0.0),
                d["tau_req"].as_f64().unwrap_or(0.0), d["B_min"].as_f64().unwrap_or(0.0), out.display());
            if let Some(f) = z["families"].as_object() {
                for (k, v) in f { println!("  {:<15} {:>7.3} kg {:>7.3} W {:>7.3} L", k, v["mass_kg"].as_f64().unwrap_or(0.0), v["power_W"].as_f64().unwrap_or(0.0), v["volume_L"].as_f64().unwrap_or(0.0)); }
            }
            return Ok(());
        }
        if !["run", "params", "parity"].contains(&a.cmd.as_str()) { return Err(format!("unknown command {}\n{}", a.cmd, usage())); }
        let c = config(&a)?;
        match a.cmd.as_str() {
            "params" => {
                let out = a.out.clone().ok_or("params needs --out")?;
                adcs_sim::fsio::write(&out, c.blob())?;
                println!("wrote {} ({} bytes, adcs-fswcfg/1)", out.display(), adcs_fsw::params::BLOB_SIZE);
            }
            "run" => {
                if !a.quiet { eprintln!("[adcs] {} on {} ({}), fsw {:?}, seed {}, {:.0} s", c.id, c.case.id, c.dev.id, a.fsw, a.seed, c.duration_s); }
                let r = run::run(&c, &run::Opts { fsw: a.fsw.clone(), quiet: a.quiet, realtime: a.realtime, oils: a.oils.clone() })?;
                let d = metrics::derive(&c, &r);
                let ms = metrics::evaluate(&c, &r, &d);
                let out = a.out.clone().unwrap_or_else(|| data_root().join("store/results_engine").join(&c.id));
                rec::write(&out, &c, &r, &d, &ms)?;
                println!("[adcs] {} done in {:.1} s wall ({:.0}x real time), fsw {} -> {}", c.id, r.wall_s, c.duration_s/r.wall_s.max(1e-9), r.fsw_build, out.display());
                print_metrics(&ms);
                if let Some(s) = &r.oils {
                    let j = s.json(c.dt);
                    let ms_ = |k: &str, q: &str| j[k][q].as_f64().unwrap_or(f64::NAN)*1e3;
                    println!("  soft OILS: latency mean {:.3} ms, max {:.3} ms (exec max {:.3} ms, bus {:.3} ms) in a {:.0} ms period; CPU load max {:.1} %; overruns {}",
                        ms_("latency_s", "mean"), ms_("latency_s", "max"), ms_("exec_s", "max"), ms_("bus_s", "max"), c.dt*1e3,
                        j["cpu_load_max"].as_f64().unwrap_or(f64::NAN)*100.0, s.overruns);
                }
            }
            "parity" => {
                // default: C vs Rust in-process; --fsw A --against B compares any two (e.g. c vs qemu)
                let (ia, ib) = match &a.fsw_b { Some(b) => (a.fsw.clone(), b.clone()), None => (Impl::C, Impl::Rust) };
                let rc = run::run(&c, &run::Opts { fsw: ia, quiet: true, realtime: false, oils: None })?;
                let rr = run::run(&c, &run::Opts { fsw: ib, quiet: true, realtime: false, oils: None })?;
                let mut dq: f64 = 0.0;
                let mut dw: f64 = 0.0;
                let mut first = None;
                for (j, (x, y)) in rc.rows.iter().zip(&rr.rows).enumerate() {
                    let q = (0..4).map(|i| (x.q[i] - y.q[i]).abs()).fold(0.0, f64::max);
                    let w = (0..3).map(|i| (x.w[i] - y.w[i]).abs()).fold(0.0, f64::max);
                    if (q > 0.0 || w > 0.0 || x.mode != y.mode) && first.is_none() { first = Some(j); }
                    dq = dq.max(q); dw = dw.max(w);
                }
                println!("[parity] {} {:.0} s: {} vs {}: max quaternion component difference {:.3e}, max rate difference {:.3e} deg/s, {}",
                    c.id, c.duration_s, rec::impl_label(&rc.fsw_impl), rec::impl_label(&rr.fsw_impl), dq, dw.to_degrees(),
                    match first { None => "bit-identical trajectories".to_string(), Some(j) => format!("first difference at t = {} s", rc.rows[j].t) });
            }
            _ => return Err(usage()),
        }
        Ok(())
    })();
    match r { Ok(()) => ExitCode::SUCCESS, Err(e) => { eprintln!("error: {e}"); ExitCode::FAILURE } }
}
