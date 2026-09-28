//! adcs -- the engine's command line.
//!
//!   adcs run <scenario> [--case F] [--fsw c|rust] [--seed N] [--out DIR] [--set k=v]... [--alg slot=id]... [--quiet]
//!   adcs params <scenario> [--case F] [--set k=v]... --out blob.bin     the adcs-fswcfg/1 blob (OILS / OBC upload)
//!   adcs parity <scenario> [--fsw A --against B] ...                  two flight-software targets, same loop, same bytes
//!
//! --fsw: c | rust (in-process) | obc-posix | obc-posix-rs (virtual OBC process) | qemu | qemu-rs
//! (virtual Cortex-M4 OBC in QEMU) | spawn:<cmd> | tcp:<host:port> (a real OBC); --realtime paces ticks to wall time.
//!
//! Scenario ids are matlab_sils/data/scenarios/*.json (or a path); the case defaults to
//! cases/<scenario.case>.csv. Overrides: `--set fsw.rw_bandwidth=0.5` edits the scenario,
//! `--set engine.duration_s=600` (also density_scale, orbit_step_s, zonal_max, igrf_nmax) the engine.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use adcs_fsw_abi::Impl;
use adcs_sim::{config::Config, data_root, metrics, rec, run};
use std::path::PathBuf;
use std::process::ExitCode;

struct Args { cmd: String, scenario: String, case: Option<String>, fsw: Impl, fsw_b: Option<Impl>, seed: u64, out: Option<PathBuf>, set: Vec<(String, String)>, quiet: bool, realtime: bool }

fn parse() -> Result<Args, String> {
    let mut it = std::env::args().skip(1);
    let cmd = it.next().ok_or_else(usage)?;
    if cmd == "-h" || cmd == "--help" { return Err(usage()); }
    let scenario = it.next().ok_or_else(usage)?;
    let mut a = Args { cmd, scenario, case: None, fsw: Impl::C, fsw_b: None, seed: 1, out: None, set: vec![], quiet: false, realtime: false };
    while let Some(k) = it.next() {
        let mut val = || it.next().ok_or(format!("{k} needs a value"));
        match k.as_str() {
            "--case" => a.case = Some(val()?),
            "--fsw" => a.fsw = val()?.parse()?,
            "--against" => a.fsw_b = Some(val()?.parse()?),
            "--realtime" => a.realtime = true,
            "--seed" => a.seed = val()?.parse().map_err(|_| "--seed: not an integer")?,
            "--out" => a.out = Some(val()?.into()),
            "--set" => { let s = val()?; let (k, v) = s.split_once('=').ok_or("--set k=v")?; a.set.push((k.into(), v.into())); }
            "--alg" => { let s = val()?; let (k, v) = s.split_once('=').ok_or("--alg slot=id")?; a.set.push((format!("fsw.algorithms.{k}"), format!("\"{v}\""))); }
            "--quiet" | "-q" => a.quiet = true,
            _ => return Err(format!("unknown option {k}\n{}", usage())),
        }
    }
    Ok(a)
}

fn usage() -> String {
    "usage: adcs run|params|parity <scenario> [--case F] [--fsw c|rust|obc-posix|obc-posix-rs|qemu|qemu-rs|spawn:<cmd>|tcp:<host:port>] [--against <fsw>] [--realtime] [--seed N] [--out DIR] [--set k=v]... [--alg slot=id]... [--quiet]".into()
}

fn config(a: &Args) -> Result<Config, String> {
    let root = data_root();
    let case = match &a.case {
        Some(c) => PathBuf::from(c),
        None => {
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
    let a = match parse() { Ok(a) => a, Err(e) => { eprintln!("{e}"); return ExitCode::from(2); } };
    let r = (|| -> Result<(), String> {
        let c = config(&a)?;
        match a.cmd.as_str() {
            "params" => {
                let out = a.out.clone().ok_or("params needs --out")?;
                std::fs::write(&out, c.blob()).map_err(|e| e.to_string())?;
                println!("wrote {} ({} bytes, adcs-fswcfg/1)", out.display(), adcs_fsw::params::BLOB_SIZE);
            }
            "run" => {
                if !a.quiet { eprintln!("[adcs] {} on {} ({}), fsw {:?}, seed {}, {:.0} s", c.id, c.case.id, c.dev.id, a.fsw, a.seed, c.duration_s); }
                let r = run::run(&c, &run::Opts { fsw: a.fsw.clone(), quiet: a.quiet, realtime: a.realtime })?;
                let d = metrics::derive(&c, &r);
                let ms = metrics::evaluate(&c, &r, &d);
                let out = a.out.clone().unwrap_or_else(|| data_root().join("store/results_engine").join(&c.id));
                rec::write(&out, &c, &r, &d, &ms)?;
                println!("[adcs] {} done in {:.1} s wall ({:.0}x real time), fsw {} -> {}", c.id, r.wall_s, c.duration_s/r.wall_s.max(1e-9), r.fsw_build, out.display());
                print_metrics(&ms);
            }
            "parity" => {
                // default: C vs Rust in-process; --fsw A --against B compares any two (e.g. c vs qemu)
                let (ia, ib) = match &a.fsw_b { Some(b) => (a.fsw.clone(), b.clone()), None => (Impl::C, Impl::Rust) };
                let rc = run::run(&c, &run::Opts { fsw: ia, quiet: true, realtime: false })?;
                let rr = run::run(&c, &run::Opts { fsw: ib, quiet: true, realtime: false })?;
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
