//! `adcs results list [DIR] | show <run> | pin|unpin <run> | thin | export | import | query | stale | refly`
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use adcs_sim::{config::Config, run, store, Error};
use crate::cli::ResultsCmd;

pub fn main(cmd: &ResultsCmd) -> Result<(), Error> {
    let default = || adcs_sim::store_root().join("results_engine");
    match cmd {
        ResultsCmd::List { dir } => {
            let root = dir.clone().unwrap_or_else(default);
            print!("{}", store::table(&store::list(&root)?, &root));
        }
        ResultsCmd::Show { run } => print!("{}", store::show(run)?),
        ResultsCmd::Export { run, out } => {
            let n = store::export(run, out)?;
            println!("wrote {} ({n} bytes): send it as it is; `adcs results import` opens it", out.display());
        }
        ResultsCmd::Import { file, out } => {
            let n = store::import(file, out)?;
            println!("imported {n} file(s) into {}; `adcs results show {}` reads it", out.display(), out.display());
        }
        ResultsCmd::Pin { run } | ResultsCmd::Unpin { run } => {
            let on = matches!(cmd, ResultsCmd::Pin { .. });
            store::pin(run, on)?;
            println!("{} {}", if on { "pinned" } else { "unpinned" }, run.display());
        }
        ResultsCmd::Thin { older_than: days, dir, dry_run: dry } => {
            let root = dir.clone().unwrap_or_else(default);
            let (done, freed) = store::thin(&root, *days, *dry)?;
            for d in &done { println!("  {}", d.strip_prefix(&root).unwrap_or(d).display()); }
            println!("{} {} run(s) older than {days} days, {:.1} MB of time series{}; pinned runs and every manifest stay",
                if *dry { "would thin" } else { "thinned" }, done.len(), freed as f64/1e6, if *dry { " (dry run: nothing changed)" } else { " freed" });
        }
        ResultsCmd::Query { sql, dir } => {
            let root = dir.clone().unwrap_or_else(default);
            store::list(&root)?;                                  // bring the index up to date first
            let (cols, rows) = adcs_sim::index::query(&root, sql)?;
            println!("{}", cols.join("\t"));
            for r in &rows { println!("{}", r.join("\t")); }
            eprintln!("{} row(s)", rows.len());
        }
        ResultsCmd::Stale { dir } => {
            let root = dir.clone().unwrap_or_else(default);
            let found = store::list(&root)?;
            let mut n = 0;
            for f in &found {
                let why = store::stale(f);
                if why.is_empty() { continue; }
                n += 1;
                println!("{}\n    {}", f.dir.strip_prefix(&root).unwrap_or(&f.dir).display(), why.join("\n    "));
            }
            println!("{n} of {} run(s) stale (engine source {}); `adcs results refly <run>` flies one again from the inputs it kept", found.len(), store::ENGINE_SOURCE);
            if n > 0 { return Err(Error::run(format!("{n} stored run(s) do not stand for today's engine and inputs"))); }
        }
        ResultsCmd::Refly { run: dir, out, fsw: on } => {
            let kept = store::kept_inputs(dir);
            let (scen, case, seed, ov, fsw) = match (kept, on) {
                (Ok(k), None) => k,
                (Ok((s, c, seed, ov, _)), Some(f)) => (s, c, seed, ov, f.clone()),
                // a run whose flight software has no name can still be flown on the one given
                (Err(e), Some(f)) if e.message().contains("--fsw") => { let (s, c, seed, ov, _) = store::kept_inputs_any(dir)?; (s, c, seed, ov, f.clone()) }
                (Err(e), _) => return Err(e),
            };
            let name = dir.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "run".into());
            let out = out.clone().unwrap_or_else(|| adcs_sim::store_root().join("refly").join(&name));
            let c = Config::build(&adcs_sim::data_root(), &scen.display().to_string(), &case, seed, &ov)?;
            let which: adcs_fsw_abi::Impl = fsw.parse().map_err(|e: adcs_fsw_abi::FswError| Error::refused(format!("the run flew flight software {fsw:?}: {e}")))?;
            eprintln!("[refly] {} from its kept inputs (seed {seed}, fsw {fsw}) -> {}", c.id, out.display());
            let ms = adcs_sim::flight::fly(&c, &run::Opts { fsw: which, quiet: true, realtime: false, oils: None }, &out)?.metrics;
            let old: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(dir.join("manifest.json")).map_err(|e| Error::io(&dir.join("manifest.json"), e))?)
                .map_err(|e| Error::malformed(format!("{}: {e}", dir.display())))?;
            print!("{}", diff(&old, &ms));
        }
    }
    Ok(())
}

/// Stored against new, metric by metric: value, requirement and verdict.
fn diff(old: &serde_json::Value, new: &[serde_json::Value]) -> String {
    let olds = old["metrics"].as_array().cloned().unwrap_or_default();
    let v = |x: &serde_json::Value| x["value"].as_f64().map(|v| format!("{v:.4}")).unwrap_or("NaN".into());
    let p = |x: &serde_json::Value| match x["pass"].as_i64() { Some(1) => "PASS", Some(_) => "FAIL", None => "" };
    let mut s = format!("  {:<26} {:>12} {:>12}  {:<5} {:<5}\n", "metric", "stored", "now", "was", "is");
    let (mut changed, mut flipped) = (0, 0);
    for n in new {
        let id = n["id"].as_str().unwrap_or("");
        let o = olds.iter().find(|x| x["id"].as_str() == Some(id));
        let ov = o.map(v).unwrap_or("—".into());
        let op = o.map(p).unwrap_or("");
        if ov != v(n) { changed += 1; }
        if op != p(n) { flipped += 1; }
        let mark = if op != p(n) { "  <- verdict changed" } else if ov != v(n) { "  <- value changed" } else { "" };
        s += &format!("  {:<26} {:>12} {:>12}  {:<5} {:<5}{mark}\n", id, ov, v(n), op, p(n));
    }
    s + &format!("{changed} value(s) and {flipped} verdict(s) changed since the stored run\n")
}
