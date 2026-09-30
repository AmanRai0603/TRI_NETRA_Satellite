//! `adcs results list [DIR] | show <run> | export <run> --out F.trinetra | import F.trinetra --out DIR`
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use adcs_sim::store;
use std::path::{Path, PathBuf};

pub fn main(args: &[String]) -> Result<(), String> {
    let usage = crate::help::usage("results");
    let out_of = |rest: &[String]| -> Result<PathBuf, String> {
        match rest { [flag, v] if flag == "--out" => Ok(PathBuf::from(v)), _ => Err(format!("needs --out <path>\n{usage}")) }
    };
    match args.first().map(String::as_str) {
        Some("list") => {
            if args.len() > 2 { return Err(usage); }
            let root = args.get(1).map(PathBuf::from).unwrap_or_else(|| adcs_sim::store_root().join("results_engine"));
            print!("{}", store::table(&store::list(&root)?, &root));
        }
        Some("show") => {
            if args.len() != 2 { return Err(usage); }
            print!("{}", store::show(Path::new(&args[1]))?);
        }
        Some("export") => {
            let d = args.get(1).ok_or(usage.clone())?;
            let out = out_of(&args[2..])?;
            let n = store::export(Path::new(d), &out)?;
            println!("wrote {} ({n} bytes): send it as it is; `adcs results import` opens it", out.display());
        }
        Some("import") => {
            let f = args.get(1).ok_or(usage.clone())?;
            let out = out_of(&args[2..])?;
            let n = store::import(Path::new(f), &out)?;
            println!("imported {n} file(s) into {}; `adcs results show {}` reads it", out.display(), out.display());
        }
        Some(v @ ("pin" | "unpin")) => {
            if args.len() != 2 { return Err(usage); }
            store::pin(Path::new(&args[1]), v == "pin")?;
            println!("{} {}", if v == "pin" { "pinned" } else { "unpinned" }, args[1]);
        }
        Some("thin") => {
            let (mut days, mut dry, mut dir) = (None, false, None);
            let mut it = args[1..].iter();
            while let Some(a) = it.next() {
                match a.as_str() {
                    "--older-than" => days = Some(it.next().and_then(|d| d.parse::<u64>().ok()).ok_or("--older-than DAYS: a whole number of days")?),
                    "--dry-run" => dry = true,
                    d if dir.is_none() && !d.starts_with("--") => dir = Some(PathBuf::from(d)),
                    _ => return Err(usage),
                }
            }
            let days = days.ok_or("thin needs --older-than DAYS")?;
            let root = dir.unwrap_or_else(|| adcs_sim::store_root().join("results_engine"));
            let (done, freed) = store::thin(&root, days, dry)?;
            for d in &done { println!("  {}", d.strip_prefix(&root).unwrap_or(d).display()); }
            println!("{} {} run(s) older than {days} days, {:.1} MB of time series{}; pinned runs and every manifest stay",
                if dry { "would thin" } else { "thinned" }, done.len(), freed as f64/1e6, if dry { " (dry run: nothing changed)" } else { " freed" });
        }
        _ => return Err(usage),
    }
    Ok(())
}
