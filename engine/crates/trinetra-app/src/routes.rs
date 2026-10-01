//! What each request does. Reading routes are GET; the three that change something (fly,
//! quit, and the page's heartbeat) are POST and carry the page's own header.
//!
//!   GET  /                 the page          GET  /v1/version     who is answering
//!   GET  /v1/catalogue     cases, scenarios  POST /v1/run         fly one scenario
//!   GET  /v1/runs          every run         GET  /v1/run?run=R   one run's provenance and metrics
//!   GET  /v1/export?run=R  a run as .trinetra                     POST /v1/ping, /v1/quit
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use crate::http::{Request, Response};
use adcs_sim::{config::Config, data_root, run, store, store_root};
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
    // the case given, else the one the scenario names: the same rule as `adcs run` (adcs_sim::flight)
    let case_file = match adcs_sim::flight::case_file(&root, &scenario, body["case"].as_str()) { Ok(f) => f, Err(e) => return failed(&e) };
    let case = case_file.file_stem().and_then(|x| x.to_str()).unwrap_or("").to_string();
    let fsw = match body["fsw"].as_str().unwrap_or("c") { "c" => adcs_fsw_abi::Impl::C, "rust" => adcs_fsw_abi::Impl::Rust, x => return Response::error(400, &format!("fsw {x:?}: c or rust")) };
    let seed = match &body["seed"] { Value::Null => 1, v => match v.as_u64() { Some(s) => s, None => return Response::error(400, "seed: a whole number") } };
    let mut sets = vec![];
    match &body["duration_s"] {
        Value::Null => {}
        v => match v.as_f64() { Some(d) => sets.push(("engine.duration_s".to_string(), d.to_string())), None => return Response::error(400, "duration_s: a number of seconds") },
    }
    let c = match Config::build(&root, &scenario, &case_file, seed, &sets) { Ok(c) => c, Err(e) => return failed(&e) };
    let (_, when) = adcs_sim::fsio::utc_now();
    let rel = format!("app/{scenario}-{case}-{}", when.replace([':', '-'], "").trim_end_matches('Z'));
    let out = store_root().join(&rel);
    let fl = match adcs_sim::flight::fly(&c, &run::Opts { fsw, quiet: true, realtime: false, oils: None }, &out) { Ok(x) => x, Err(e) => return failed(&e) };
    let (rec_, ms) = (&fl.record, &fl.metrics);
    Response::json(200, &json!({"ok": true, "run": rel, "scenario": scenario, "case": case, "metrics": ms,
        "wall_s": rec_.wall_s, "duration_s": c.duration_s, "fsw": rec_.fsw_build}))
}

#[cfg(test)]
mod t {
    use super::*;

    fn req(method: &str, path: &str, query: &str, body: &str) -> Request {
        Request { method: method.into(), path: path.into(), query: query.into(), headers: vec![], body: body.as_bytes().to_vec() }
    }
    fn body(r: &Response) -> Value { serde_json::from_slice(&r.body).unwrap_or(Value::Null) }

    /// Every route against a store of its own: one test, since the store is named by the
    /// environment and the tests of a binary share it.
    #[test]
    fn each_route_answers_or_refuses_as_it_says() {
        let store = std::env::temp_dir().join(format!("trinetra-routes-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&store);
        std::env::set_var("TRINETRA_STORE", &store);

        let page = route(&req("GET", "/", "", ""), 7788);
        assert!(page.status == 200 && page.kind.starts_with("text/html"));
        assert_eq!(route(&req("GET", "/nowhere", "", ""), 7788).status, 404);
        assert_eq!(route(&req("DELETE", "/v1/runs", "", ""), 7788).status, 404, "a method a route does not take");
        let v = body(&route(&req("GET", "/v1/version", "", ""), 7788));
        assert_eq!(v["port"], 7788);
        assert_eq!(v["store"], store.display().to_string());

        let cat = body(&route(&req("GET", "/v1/catalogue", "", ""), 7788));
        assert!(cat["cases"].as_array().unwrap().iter().any(|c| c["id"] == "ais_3u"), "the shipped cases are listed");
        assert!(cat["scenarios"].as_array().unwrap().iter().any(|s| s["id"] == "nadir_hold_ais" && s["case"] == "ais_3u"));

        // a run is named by a folder inside the store, and nothing else
        for q in ["", "run=", "run=..%2Fx", "run=%2Fetc", "run=a%2F..%2F..%2Fx"] {
            assert_eq!(route(&req("GET", "/v1/run", q, ""), 7788).status, 400, "{q:?}");
            assert_eq!(route(&req("GET", "/v1/export", q, ""), 7788).status, 400, "{q:?}");
        }
        assert_eq!(route(&req("GET", "/v1/run", "run=app%2Fno_such", ""), 7788).status, 404);
        assert_eq!(body(&route(&req("GET", "/v1/runs", "", ""), 7788))["runs"], json!([]), "an empty store has no runs");

        // a flight: what is refused is the page's to fix (400)
        for (b, why) in [("not json", "not JSON"), (r#"{"scenario":"no_such_scenario"}"#, "no scenario"), (r#"{"scenario":"../x"}"#, "scenario"),
                         (r#"{"scenario":"nadir_hold_ais","case":"no_such_case"}"#, "no case"), (r#"{"scenario":"nadir_hold_ais","fsw":"ada"}"#, "c or rust"),
                         (r#"{"scenario":"nadir_hold_ais","seed":-1}"#, "seed"), (r#"{"scenario":"nadir_hold_ais","duration_s":"long"}"#, "duration_s")] {
            let r = route(&req("POST", "/v1/run", "", b), 7788);
            assert_eq!(r.status, 400, "{b}: {:?}", body(&r));
            assert!(body(&r)["error"].as_str().unwrap_or("").contains(why), "{b}: {:?}", body(&r));
        }

        // and one that flies: stored, listed, shown and exported
        let r = route(&req("POST", "/v1/run", "", r#"{"scenario":"nadir_hold_ais","fsw":"rust","duration_s":60}"#), 7788);
        let f = body(&r);
        assert_eq!(r.status, 200, "{f:?}");
        assert!(f["ok"] == true && f["case"] == "ais_3u" && f["duration_s"] == 60.0 && !f["metrics"].as_array().unwrap().is_empty());
        let run = f["run"].as_str().unwrap().to_string();
        assert!(store.join(&run).join("manifest.json").is_file(), "the flight is in the store");
        let runs = body(&route(&req("GET", "/v1/runs", "", ""), 7788));
        assert_eq!(runs["runs"].as_array().unwrap().len(), 1);
        assert_eq!(runs["runs"][0]["run"], run.as_str());
        assert_eq!(runs["runs"][0]["scenario"], "nadir_hold_ais");
        let q = format!("run={}", run.replace('/', "%2F"));
        let one = body(&route(&req("GET", "/v1/run", &q, ""), 7788));
        assert!(one["text"].as_str().unwrap().contains("nadir_hold_ais") && one["metrics"] == f["metrics"]);
        let z = route(&req("GET", "/v1/export", &q, ""), 7788);
        assert!(z.status == 200 && z.kind == "application/zip" && z.body.starts_with(b"PK"), "a share file is a zip");
        assert!(z.extra.iter().any(|(k, v)| k == "Content-Disposition" && v.ends_with(".trinetra\"")));

        let _ = std::fs::remove_dir_all(&store);
    }
}
