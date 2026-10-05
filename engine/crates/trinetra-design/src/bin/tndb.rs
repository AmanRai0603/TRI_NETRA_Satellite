//! tndb -- the design library from the command line (the tools and CI call it; people use the
//! application):
//!
//!     tndb check FILE...        every problem with each file against the schema (exit 1 if any)
//!     tndb upgrade FILE...      bring older files to this format version, a copy kept beside each
//!     tndb hash FILE            the content hash a signature covers
//!     tndb compare A B          the rows only one of two files of one kind holds (exit 1 if any)
//!     tndb check-folder DIR     every problem `tools/group.py check` finds in a design folder
//!     tndb verify-releases DIR [--catalogue JSON]  each group's latest release against the folder
//!                               (`tools/group.py verify`): one line per problem, "<group>: <problem>"
//!     tndb check-release FILE [--catalogue JSON]   every problem `tools/release.py check` finds
//!                               (JSON: {row id: kind}, the spec's catalogue, saying which rows compute)
//!     tndb check-node FILE [--folder DIR]           the node app's live checks on a node file, one
//!                               line each, "CODE level step: text" (level ! to fix, i to know); with
//!                               the design folder, its inputs are checked against the design's nodes.
//!                               Exit 1 if any is to fix, 2 if the file cannot be read or checked
//!
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use std::path::Path;
use std::process::ExitCode;
use trinetra_design::{check, checks, compare, content, node_rules, open, write, Schema};

fn main() -> ExitCode {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let s = match Schema::embedded() {
        Ok(s) => s,
        Err(e) => { eprintln!("tndb: {e}"); return ExitCode::from(2); }
    };
    let usage = || { eprintln!("usage: tndb check FILE... | upgrade FILE... | hash FILE | compare A B | check-folder DIR | check-release FILE [--catalogue JSON] | check-node FILE [--folder DIR]"); ExitCode::from(2) };
    match a.first().map(String::as_str) {
        Some("check") if a.len() > 1 => {
            let errs: Vec<String> = a[1..].iter().flat_map(|f| check(Path::new(f), &s)).collect();
            for e in &errs { eprintln!("tndb: {e}"); }
            println!("tndb: {} file(s), {} problem(s)", a.len() - 1, errs.len());
            if errs.is_empty() { ExitCode::SUCCESS } else { ExitCode::from(1) }
        }
        Some("upgrade") if a.len() > 1 => {
            let (mut up, mut bad) = (0, 0);
            for f in &a[1..] {
                match write::upgrade(Path::new(f), None, &s) {
                    Ok(u) if u.from.is_some() => up += 1,
                    Ok(_) => {}
                    Err(e) => { eprintln!("tndb: {e}"); bad += 1; }
                }
            }
            println!("tndb: {} file(s), {up} upgraded, {bad} refused", a.len() - 1);
            if bad == 0 { ExitCode::SUCCESS } else { ExitCode::from(1) }
        }
        Some("hash") if a.len() == 2 => match open(Path::new(&a[1]), None, &s).and_then(|f| content::content_hash(&f, &s)) {
            Ok(h) => { println!("{h}"); ExitCode::SUCCESS }
            Err(e) => { eprintln!("tndb: {e}"); ExitCode::from(1) }
        },
        Some("compare") if a.len() == 3 => {
            let r = open(Path::new(&a[1]), None, &s).and_then(|x| open(Path::new(&a[2]), None, &s).and_then(|y| compare::compare(&x, &y, &s, false)));
            match r {
                Ok(d) => {
                    for t in &d { println!("{}: {} row(s) only in the first, {} only in the second", t.table, t.only_a.len(), t.only_b.len()); }
                    if d.is_empty() { println!("tndb: the same content"); ExitCode::SUCCESS } else { ExitCode::from(1) }
                }
                Err(e) => { eprintln!("tndb: {e}"); ExitCode::from(2) }
            }
        }
        Some("check-folder") if a.len() == 2 => report(checks::check_folder(Path::new(&a[1]), &s)),
        Some("verify-releases") if a.len() == 2 || (a.len() == 4 && a[2] == "--catalogue") => {
            let cat = match catalogue(a.get(3)) { Some(c) => c, None => return ExitCode::from(2) };
            let v = checks::verify_releases(Path::new(&a[1]), &cat, &s);
            println!("{}", serde_json::to_string(&v).unwrap_or_default());
            if v.values().all(|p| p.is_empty()) { ExitCode::SUCCESS } else { ExitCode::from(1) }
        }
        Some("check-release") if a.len() == 2 || (a.len() == 4 && a[2] == "--catalogue") => {
            let cat = match catalogue(a.get(3)) { Some(c) => c, None => return ExitCode::from(2) };
            report(checks::check_release(Path::new(&a[1]), &cat, &s))
        }
        Some("check-node") if a.len() == 2 || (a.len() == 4 && a[2] == "--folder") => check_node(Path::new(&a[1]), a.get(3).map(Path::new)),
        _ => usage(),
    }
}

/// `check-node`: the node app's checks, one line each.
fn check_node(file: &Path, folder: Option<&Path>) -> ExitCode {
    let ctx = match folder.map(node_rules::Context::from_folder).transpose() {
        Ok(c) => c.unwrap_or_default(),
        Err(e) => { eprintln!("tndb: {e}"); return ExitCode::from(2); }
    };
    match node_rules::check_file(file, &ctx) {
        Ok(problems) => {
            for p in &problems { println!("{} {} {}: {}", p.code, p.level, p.step, p.text); }
            let fix = problems.iter().filter(|p| p.level == '!').count();
            eprintln!("tndb: {} problem(s), {fix} to fix", problems.len());
            if fix == 0 { ExitCode::SUCCESS } else { ExitCode::from(1) }
        }
        Err(e) => { eprintln!("tndb: {e}"); ExitCode::from(2) }
    }
}

/// Print each problem on its own line (stdout, for the tools to read) and the count; exit 1 if any.
fn report(problems: Vec<String>) -> ExitCode {
    for p in &problems { println!("{p}"); }
    eprintln!("tndb: {} problem(s)", problems.len());
    if problems.is_empty() { ExitCode::SUCCESS } else { ExitCode::from(1) }
}

/// The spec's catalogue of rows, {row id: kind}, from a JSON file (empty when none is given).
fn catalogue(f: Option<&String>) -> Option<std::collections::BTreeMap<String, String>> {
    match f {
        None => Some(Default::default()),
        Some(f) => {
            let c = std::fs::read_to_string(f).ok().and_then(|t| serde_json::from_str(&t).ok());
            if c.is_none() { eprintln!("tndb: {f}: not a JSON object of row id to kind"); }
            c
        }
    }
}
