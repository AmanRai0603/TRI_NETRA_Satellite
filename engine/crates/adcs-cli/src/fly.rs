//! The commands that fly or build a configuration: run, params, parity, size.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use crate::args::Args;
use adcs_fsw_abi::Impl;
use adcs_sim::{config::Config, data_root, rec, run, Error};
use std::path::PathBuf;

pub fn config(a: &Args) -> Result<Config, Error> {
    let root = data_root();
    // --case is a file here (a path or a name with an extension) or an id of a shipped case
    let case = match &a.case {
        Some(c) if std::path::Path::new(c).is_file() => PathBuf::from(c),
        c => adcs_sim::flight::case_file(&root, &a.scenario, c.as_deref())?,
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

pub fn main(a: &Args) -> Result<(), Error> {
        // a design named by $TRINETRA_DESIGN that cannot be read is refused here, by name, before
        // any input is looked for (never passed over as if no design were named)
        adcs_sim::source::current()?;
        if a.cmd == "size" {
            // adcs size <case> [--out DIR] [--knobs knobs.json]: demand survey + every option sized (adcs-design)
            let root = data_root();
            adcs_sim::config::check_id("case", &a.scenario)?;
            let case_file = root.join("cases").join(format!("{}.csv", a.scenario));
            if !adcs_sim::source::is_file(&case_file) { return Err(Error::refused(format!("no case {}: {} does not exist", a.scenario, case_file.display()))); }
            let knobs = match a.set.iter().find(|(k, _)| k == "knobs") {
                Some((_, f)) => adcs_design::Knobs::from_json(&adcs_sim::json::read(std::path::Path::new(f))?)?,
                None => adcs_design::Knobs::default(),
            };
            let out = a.out.clone().unwrap_or_else(|| adcs_sim::store_root().join("design").join(&a.scenario).join("sized"));
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
        let c = config(a)?;
        match a.cmd.as_str() {
            "params" => {
                let out = a.out.clone().ok_or_else(|| Error::refused("params needs --out"))?;
                adcs_sim::fsio::write(&out, c.blob())?;
                println!("wrote {} ({} bytes, adcs-fswcfg/1)", out.display(), adcs_fsw::params::BLOB_SIZE);
            }
            "run" => {
                if !a.quiet { eprintln!("[adcs] {} on {} ({}), fsw {:?}, seed {}, {:.0} s", c.id, c.case.id, c.dev.id, a.fsw, a.seed, c.duration_s); }
                let out = a.out.clone().unwrap_or_else(|| adcs_sim::store_root().join("results_engine").join(&c.id));
                let fl = adcs_sim::flight::fly(&c, &run::Opts { fsw: a.fsw.clone(), quiet: a.quiet, realtime: a.realtime, oils: a.oils.clone() }, &out)?;
                let (r, ms) = (&fl.record, &fl.metrics);
                let (gone, freed) = &fl.retained;
                if !gone.is_empty() && !a.quiet {
                    eprintln!("[adcs] retention: {} run(s) older than {} days lost their time series ({:.1} MB); verdicts and provenance stay",
                        gone.len(), adcs_sim::store::retention_days()?.unwrap_or(0), *freed as f64/1e6);
                }
                println!("[adcs] {} done in {:.1} s wall ({:.0}x real time), fsw {} -> {}", c.id, r.wall_s, c.duration_s/r.wall_s.max(1e-9), r.fsw_build, out.display());
                print_metrics(&ms);
                if let Some(s) = &r.oils {
                    let j = s.json(c.dt);
                    let ms_ = |k: &str, q: &str| j[k][q].as_f64().unwrap_or(f64::NAN)*1e3;
                    println!("  soft OILS: latency mean {:.3} ms, max {:.3} ms (exec max {:.3} ms, bus {:.3} ms) in a {:.0} ms period; CPU load max {:.1} %; overruns {}",
                        ms_("latency_s", "mean"), ms_("latency_s", "max"), ms_("exec_s", "max"), ms_("bus_s", "max"), c.dt*1e3,
                        j["cpu_load_max"].as_f64().unwrap_or(f64::NAN)*100.0, s.overruns);
                    println!("  worst case (CPI {}, {:.0} us of interrupts): margin {:.3} ms to the {:.0} ms deadline",
                        j["model"]["cpi_max"], j["model"]["isr_s"].as_f64().unwrap_or(f64::NAN)*1e6,
                        j["worst_case_margin_s"].as_f64().unwrap_or(f64::NAN)*1e3, j["deadline_s"].as_f64().unwrap_or(f64::NAN)*1e3);
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
                if let Some(j) = first {
                    return Err(Error::run(format!("the two targets differ from t = {} s", rc.rows[j].t)));
                }
            }
            other => return Err(Error::refused(format!("{other} is not a flying command"))),
        }
        Ok(())
}
