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
//!     tndb translate rust|c|matlab FILE... [--title T] [--lib L] [--pkg P]
//!     tndb translate matlab-rt      the pseudocode files translated (trinetra-pcode's translators), as
//!                               design/js/pcode_cli.mjs prints them: a JSON object of path to text.
//!                               Exit 1 if the files have problems or cannot be translated
//!     tndb read DESIGN [--engine-inputs | --cases]   the MATLAB twin's inputs from a design database
//!                               (docs/S7_INVENTORY.md S7.18): a JSON array of {path, body}, every engine
//!                               input (data/...) and every case (cases/<id>.csv), the bytes
//!                               tools/from_design.py writes for the twin. Exit 1 if an input is not what
//!                               its fingerprint says, 2 if the file is not a design database
//!     tndb health DESIGN        what a design is and how it stands (its meta, nodes by behaviour, the
//!                               built-in count, its groups' releases, inputs, signatures), as JSON
//!     tndb build-matlab DESIGN OUTDIR [--groups DIR]  every MATLAB function the twin flies, from the design
//!                               (the engine's models +asils/+models, the relations +asils/+relations with the
//!                               groups' wiring DIR, the flight algorithms +asils/+alg, the runtime +asils/+pc),
//!                               byte for byte what tools/engine_build.py and tools/flight_build.py write,
//!                               read-only, with OUTDIR/index.json naming each file's node, release and
//!                               revision; prints what it made as JSON. OUTDIR is new, empty, or one it made
//!                               before (anything else in it is refused, never deleted)
//!
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use std::path::Path;
use std::process::ExitCode;
use trinetra_design::{check, checks, compare, content, node_rules, open, twin, write, Schema};

fn main() -> ExitCode {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let s = match Schema::embedded() {
        Ok(s) => s,
        Err(e) => { eprintln!("tndb: {e}"); return ExitCode::from(2); }
    };
    let usage = || { eprintln!("usage: tndb check FILE... | upgrade FILE... | hash FILE | compare A B | check-folder DIR | check-release FILE [--catalogue JSON] | check-node FILE [--folder DIR] | translate rust|c|matlab|matlab-rt FILE... [--title T] [--lib L] [--pkg P] | read DESIGN [--engine-inputs|--cases] | health DESIGN | build-matlab DESIGN OUTDIR [--groups DIR]"); ExitCode::from(2) };
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
        Some("translate") if a.len() > 1 && ["rust", "c", "matlab", "matlab-rt"].contains(&a[1].as_str()) => translate(&a[1], &a[2..]),
        Some("read") if a.len() == 2 || (a.len() == 3 && ["--engine-inputs", "--cases"].contains(&a[2].as_str())) => {
            let which = match a.get(2).map(String::as_str) {
                Some("--engine-inputs") => twin::Which::EngineInputs,
                Some("--cases") => twin::Which::Cases,
                _ => twin::Which::All,
            };
            read(Path::new(&a[1]), which)
        }
        Some("health") if a.len() == 2 => match twin::health(Path::new(&a[1])) {
            Ok(h) => { println!("{}", serde_json::to_string_pretty(&h).unwrap_or_default()); ExitCode::SUCCESS }
            Err(e) => { eprintln!("tndb: {e}"); ExitCode::from(2) }
        },
        Some("build-matlab") if a.len() == 3 || (a.len() == 5 && a[3] == "--groups") => {
            build_matlab(Path::new(&a[1]), Path::new(&a[2]), a.get(4).map(Path::new))
        }
        _ => usage(),
    }
}

/// `translate`: the files translated, printed as `JSON.stringify(files, null, 1)` (pcode_cli.mjs).
/// An argument `--name` takes the one after it as its value; the others are the files, in order.
fn translate(lang: &str, args: &[String]) -> ExitCode {
    let (mut files, mut opt) = (Vec::new(), std::collections::HashMap::new());
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--no-dispatch" {
            // a flag: it takes no value
            opt.insert("no-dispatch".to_string(), None);
            i += 1;
        } else if let Some(name) = args[i].strip_prefix("--") {
            opt.insert(name.to_string(), args.get(i + 1).cloned());
            i += 2;
        } else {
            files.push(args[i].clone());
            i += 1;
        }
    }
    let o = |k: &str| opt.get(k).and_then(|v| v.as_deref());
    if lang == "matlab-rt" {
        print!("{}", trinetra_pcode::files_json(&trinetra_pcode::matlab_runtime_files()));
        return ExitCode::SUCCESS;
    }
    let mut texts = Vec::new();
    for f in &files {
        match std::fs::read(f) {
            Ok(b) => texts.push(String::from_utf8_lossy(&b).into_owned()),
            Err(e) => { eprintln!("tndb: {f}: {e}"); return ExitCode::from(2); }
        }
    }
    let sources: Vec<(&str, &str)> = files.iter().map(String::as_str).zip(texts.iter().map(String::as_str)).collect();
    let program = match trinetra_pcode::compile(&sources) {
        Ok(p) => p,
        Err(errs) => {
            for e in &errs { eprintln!("{e}"); }
            return ExitCode::from(1);
        }
    };
    let r = match lang {
        "rust" => program.to_rust_files_with(&trinetra_pcode::RustOptions {
            title: o("title"),
            root: o("root"),
            math: o("math"),
            dispatch: !opt.contains_key("no-dispatch"),
        }),
        "c" => program.to_c_files_with(o("title"), o("lib"), !opt.contains_key("no-dispatch")),
        _ => program.to_matlab_files(o("pkg")),
    };
    match r {
        Ok(files) => { print!("{}", trinetra_pcode::files_json(&files)); ExitCode::SUCCESS }
        Err(e) => { eprintln!("tndb: cannot translate: {e}"); ExitCode::from(1) }
    }
}

/// `read`: the twin's inputs, as one JSON array of {path, body}.
fn read(file: &Path, which: twin::Which) -> ExitCode {
    if let Err(e) = twin::connect(file) {
        eprintln!("tndb: {e}");
        return ExitCode::from(2);
    }
    match twin::inputs(file, which).and_then(|x| twin::inputs_json(&x)) {
        Ok(j) => { println!("{j}"); ExitCode::SUCCESS }
        Err(e) => { eprintln!("tndb: {e}"); ExitCode::from(1) }
    }
}

/// `build-matlab`: every MATLAB function the twin flies, written under `out`; what was made, as JSON.
fn build_matlab(file: &Path, out: &Path, groups: Option<&Path>) -> ExitCode {
    let b = match twin::build_matlab(file, groups) {
        Ok(b) => b,
        Err(e) => { eprintln!("tndb: {}: {e}", file.display()); return ExitCode::from(1); }
    };
    if let Err(e) = twin::write_build(out, &b) {
        eprintln!("tndb: {e}");
        return ExitCode::from(1);
    }
    let i = &b.index;
    let summary = serde_json::json!({"out": out.display().to_string(), "files": b.files.len(), "packages": i["packages"],
        "alg_id": i["alg_id"], "fingerprint": i["fingerprint"], "skipped": i["skipped"]});
    println!("{}", serde_json::to_string(&summary).unwrap_or_default());
    ExitCode::SUCCESS
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
