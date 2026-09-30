//! What each request does. Reading routes are GET; the three that change something (fly,
//! quit, and the page's heartbeat) are POST and carry the page's own header.
//!
//!   GET  /                 the page          GET  /v1/version     who is answering
//!   GET  /v1/catalogue     cases, scenarios  POST /v1/run         fly one scenario
//!   GET  /v1/runs          every run         GET  /v1/run?run=R   one run's provenance and metrics
//!   GET  /v1/export?run=R  a run as .trinetra                     POST /v1/ping, /v1/quit
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use crate::http::{Request, Response};
use adcs_sim::{config::Config, data_root, metrics, rec, run, store, store_root};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

static PAGE: &str = include_str!("page.html");
static ICON: &[u8] = include_bytes!("../icon/trinetra.png");
/// one flight at a time: a second request while one flies is told so, not queued
static FLYING: Mutex<()> = Mutex::new(());

pub fn route(r: &Request, port: u16) -> Response {
    match (r.method.as_str(), r.path.as_str()) {
        ("GET", "/") => { crate::seen(); Response { status: 200, kind: "text/html; charset=utf-8", body: PAGE.as_bytes().to_vec(), extra: vec![] } }
        ("GET", "/icon.png") => Response { status: 200, kind: "image/png", body: ICON.to_vec(), extra: vec![] },
        ("GET", "/v1/version") => Response::json(200, &json!({"app": "trinetra", "version": version(), "engine": adcs_sim::ENGINE, "port": port,
            "data": data_root().display().to_string(), "store": store_root().display().to_string()})),
        ("GET", "/v1/catalogue") => catalogue(),
        ("GET", "/v1/runs") => runs(),
        ("GET", "/v1/run") => one_run(r),
        ("GET", "/v1/export") => export(r),
        ("POST", "/v1/run") => fly(r),
        ("POST", "/v1/ping") => { crate::seen(); Response::json(200, &json!({"ok": true})) }
        ("POST", "/v1/quit") => { crate::QUIT.store(true, std::sync::atomic::Ordering::Relaxed); Response::json(200, &json!({"ok": true, "quitting": true})) }
        _ => Response::error(404, "no such page"),
    }
}

fn version() -> String {
    std::fs::read_to_string(data_root().join("VERSION")).ok().and_then(|v| v.lines().next().map(|l| l.trim_start_matches("TRI-NETRA ADCS ").trim().to_string()))
        .or_else(|| std::fs::read_to_string(data_root().join("../VERSION")).ok().map(|v| v.trim().to_string()))
        .unwrap_or_else(|| env!("CARGO_PKG_VERSION").into())
}

/// Every case (with its title) and every scenario (with its label, case and product).
fn catalogue() -> Response {
    let root = data_root();
    let mut cases = vec![];
    if let Ok(rd) = std::fs::read_dir(root.join("cases")) {
        let mut fs: Vec<PathBuf> = rd.flatten().map(|e| e.path()).filter(|p| p.extension().and_then(|x| x.to_str()) == Some("csv")).collect();
        fs.sort();
        for f in fs {
            if let Ok(c) = adcs_sim::case::Case::read(&f) { cases.push(json!({"id": c.id, "title": c.title})); }
        }
    }
    let mut scen = vec![];
    if let Ok(rd) = std::fs::read_dir(root.join("data/scenarios")) {
        let mut fs: Vec<PathBuf> = rd.flatten().map(|e| e.path()).filter(|p| p.extension().and_then(|x| x.to_str()) == Some("json")).collect();
        fs.sort();
        for f in fs {
            let Some(v) = std::fs::read_to_string(&f).ok().and_then(|s| serde_json::from_str::<Value>(&s).ok()) else { continue };
            scen.push(json!({"id": f.file_stem().and_then(|x| x.to_str()).unwrap_or(""), "label": v["label"], "case": v["case"],
                             "product": v["product"], "duration_s": v["time"]["duration_s"]}));
        }
    }
    Response::json(200, &json!({"cases": cases, "scenarios": scen}))
}

/// A run folder named by the page, inside the store or not at all.
fn run_dir(r: &Request) -> Result<PathBuf, Response> {
    let rel = r.param("run").ok_or_else(|| Response::error(400, "which run? (?run=...)"))?;
    let rel_path = Path::new(&rel);
    if rel.is_empty() || rel_path.is_absolute() || rel_path.components().any(|c| !matches!(c, std::path::Component::Normal(_))) {
        return Err(Response::error(400, "a run is named by its folder inside the store"));
    }
    let d = store_root().join(rel_path);
    if !d.join("manifest.json").is_file() { return Err(Response::error(404, "no such run")); }
    // a symbolic link inside the store must not lead out of it
    let (canon, root) = (std::fs::canonicalize(&d), std::fs::canonicalize(store_root()));
    if !matches!((canon, root), (Ok(c), Ok(r)) if c.starts_with(&r)) { return Err(Response::error(404, "no such run")); }
    Ok(d)
}

fn runs() -> Response {
    let root = store_root();
    let found = if root.is_dir() { store::list(&root).unwrap_or_default() } else { vec![] };
    let mut out: Vec<Value> = found.iter().map(|f| {
        let (p, x) = f.verdicts();
        json!({"run": f.dir.strip_prefix(&root).unwrap_or(&f.dir).display().to_string().replace('\\', "/"),
               "scenario": f.m["scenario"], "case": f.m["case"], "created": f.m["created_utc"], "pass": p, "fail": x, "fsw": f.m["fsw"]["impl"]})
    }).collect();
    out.sort_by(|a, b| b["created"].as_str().unwrap_or("").cmp(a["created"].as_str().unwrap_or("")));
    Response::json(200, &json!({"store": root.display().to_string(), "runs": out}))
}

/// An engine error as an answer: a refused input is the page's to fix (400), anything
/// else is a failure here (500).
fn failed(e: &adcs_sim::Error) -> Response {
    Response::error(if e.kind == adcs_sim::Kind::Refused { 400 } else { 500 }, e.message())
}

fn one_run(r: &Request) -> Response {
    let d = match run_dir(r) { Ok(d) => d, Err(e) => return e };
    let m: Value = std::fs::read_to_string(d.join("manifest.json")).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or(Value::Null);
    Response::json(200, &json!({"text": store::show(&d).unwrap_or_else(String::from), "metrics": m["metrics"], "inputs": m["inputs"]}))
}

fn export(r: &Request) -> Response {
    let d = match run_dir(r) { Ok(d) => d, Err(e) => return e };
    static SERIAL: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let tmp = std::env::temp_dir().join(format!("trinetra-export-{}-{}-{}.trinetra", std::process::id(), crate::now(),
        SERIAL.fetch_add(1, std::sync::atomic::Ordering::Relaxed)));
    match store::export(&d, &tmp) {
        Ok(_) => {
            let body = std::fs::read(&tmp).unwrap_or_default();
            let _ = std::fs::remove_file(&tmp);
            let name = d.file_name().and_then(|n| n.to_str()).unwrap_or("run").replace(|c: char| !c.is_ascii_alphanumeric() && c != '-' && c != '_', "_");
            Response { status: 200, kind: "application/zip", body, extra: vec![("Content-Disposition".into(), format!("attachment; filename=\"{name}.trinetra\""))] }
        }
        Err(e) => failed(&e),
    }
}

/// Fly one scenario: {"scenario", "case"?, "fsw": "c" | "rust", "seed"?, "duration_s"?}
fn fly(r: &Request) -> Response {
    // one flight at a time; a flight that panicked leaves the lock poisoned, which is not a flight running
    let _guard = match FLYING.try_lock() {
        Ok(g) => g,
        Err(std::sync::TryLockError::Poisoned(p)) => p.into_inner(),
        Err(std::sync::TryLockError::WouldBlock) => return Response::error(409, "a flight is already running: wait for it to finish"),
    };
    crate::seen();
    let body: Value = match serde_json::from_slice(&r.body) { Ok(v) => v, Err(_) => return Response::error(400, "the request is not JSON") };
    let root = data_root();
    let scenario = body["scenario"].as_str().unwrap_or("").to_string();
    if let Err(e) = adcs_sim::config::check_id("scenario", &scenario) { return failed(&e); }
    let scen_file = root.join("data/scenarios").join(format!("{scenario}.json"));
    let Some(sv) = std::fs::read_to_string(&scen_file).ok().and_then(|s| serde_json::from_str::<Value>(&s).ok()) else { return Response::error(404, &format!("no scenario {scenario}")) };
    let Some(case) = body["case"].as_str().filter(|c| !c.is_empty()).or_else(|| sv["case"].as_str()).map(String::from) else {
        return Response::error(400, &format!("scenario {scenario} names no case: choose one"));
    };
    if let Err(e) = adcs_sim::config::check_id("case", &case) { return failed(&e); }
    let case_file = root.join("cases").join(format!("{case}.csv"));
    if !case_file.is_file() { return Response::error(404, &format!("no case {case}")); }
    let fsw = match body["fsw"].as_str().unwrap_or("c") { "c" => adcs_fsw_abi::Impl::C, "rust" => adcs_fsw_abi::Impl::Rust, x => return Response::error(400, &format!("fsw {x:?}: c or rust")) };
    let seed = match &body["seed"] { Value::Null => 1, v => match v.as_u64() { Some(s) => s, None => return Response::error(400, "seed: a whole number") } };
    let mut sets = vec![];
    match &body["duration_s"] {
        Value::Null => {}
        v => match v.as_f64() { Some(d) => sets.push(("engine.duration_s".to_string(), d.to_string())), None => return Response::error(400, "duration_s: a number of seconds") },
    }
    let c = match Config::build(&root, &scenario, &case_file, seed, &sets) { Ok(c) => c, Err(e) => return failed(&e) };
    let rec_ = match run::run(&c, &run::Opts { fsw, quiet: true, realtime: false, oils: None }) { Ok(x) => x, Err(e) => return failed(&e) };
    let d = metrics::derive(&c, &rec_);
    let ms = metrics::evaluate(&c, &rec_, &d);
    let (_, when) = adcs_sim::fsio::utc_now();
    let rel = format!("app/{scenario}-{case}-{}", when.replace([':', '-'], "").trim_end_matches('Z'));
    let out = store_root().join(&rel);
    if let Err(e) = rec::write(&out, &c, &rec_, &d, &ms) { return failed(&e); }
    Response::json(200, &json!({"ok": true, "run": rel, "scenario": scenario, "case": case, "metrics": ms,
        "wall_s": rec_.wall_s, "duration_s": c.duration_s, "fsw": rec_.fsw_build}))
}
