//! `adcs results list [DIR] | show <run> | export <run> --out F.trinetra | import F.trinetra --out DIR`
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use adcs_sim::{data_root, store};
use std::path::{Path, PathBuf};

pub fn main(args: &[String]) -> Result<(), String> {
    let usage = crate::help::usage("results");
    let out_of = |rest: &[String]| -> Result<PathBuf, String> {
        match rest { [flag, v] if flag == "--out" => Ok(PathBuf::from(v)), _ => Err(format!("needs --out <path>\n{usage}")) }
    };
    match args.first().map(String::as_str) {
        Some("list") => {
            if args.len() > 2 { return Err(usage); }
            let root = args.get(1).map(PathBuf::from).unwrap_or_else(|| data_root().join("store/results_engine"));
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
        _ => return Err(usage),
    }
    Ok(())
}
