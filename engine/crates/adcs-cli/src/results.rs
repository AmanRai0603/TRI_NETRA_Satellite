//! `adcs results list [DIR] | show <run> | export <run> --out F.trinetra | import F.trinetra --out DIR`
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use adcs_sim::store;
use crate::cli::ResultsCmd;

pub fn main(cmd: &ResultsCmd) -> Result<(), adcs_sim::Error> {
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
    }
    Ok(())
}
