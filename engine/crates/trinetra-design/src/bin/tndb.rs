//! tndb -- the design library from the command line (the tools and CI call it; people use the
//! application):
//!
//!     tndb check FILE...        every problem with each file against the schema (exit 1 if any)
//!     tndb upgrade FILE...      bring older files to this format version, a copy kept beside each
//!     tndb hash FILE            the content hash a signature covers
//!     tndb compare A B          the rows only one of two files of one kind holds (exit 1 if any)
//!
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use std::path::Path;
use std::process::ExitCode;
use trinetra_design::{check, compare, content, open, write, Schema};

fn main() -> ExitCode {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let s = match Schema::embedded() {
        Ok(s) => s,
        Err(e) => { eprintln!("tndb: {e}"); return ExitCode::from(2); }
    };
    let usage = || { eprintln!("usage: tndb check FILE... | upgrade FILE... | hash FILE | compare A B"); ExitCode::from(2) };
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
        _ => usage(),
    }
}
