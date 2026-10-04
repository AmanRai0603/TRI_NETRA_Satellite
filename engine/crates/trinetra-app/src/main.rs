//! TRI-NETRA ADCS -- the desktop app.
//!
//! A person opens it like any program. It serves one page on this machine only
//! (127.0.0.1), opens the browser on it, and there the person picks a case and a scenario,
//! flies it on the engine (in this process, the same code as `adcs run`), reads the
//! requirement verdicts, and finds, shows or exports every run they have flown.
//!
//! - **One copy.** Opening it again while it runs opens the running one's page.
//! - **Ending.** Quit on the page ends it; so does no page having been open for
//!   `TRINETRA_APP_IDLE_MINUTES` (5) minutes.
//! - **Safety.** Only this machine can reach it; a request whose Host is not this app's is
//!   refused (DNS rebinding), and every request that changes something must carry the
//!   page's own header, so another web site cannot drive it. Request heads and bodies are
//!   capped; one flight runs at a time.
//! - **Failure.** A start that fails shows a page saying why (kept in the log folder); an
//!   internal error in a request answers 500 and leaves a crash report.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
#![cfg_attr(windows, windows_subsystem = "windows")]

mod http;
mod routes;

use std::io::Read;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::time::Duration;

pub static QUIT: AtomicBool = AtomicBool::new(false);
pub static LAST_SEEN: AtomicU64 = AtomicU64::new(0);
static OPEN: AtomicUsize = AtomicUsize::new(0);
/// connections served at once; more wait in the kernel's queue
const MAX_OPEN: usize = 16;
pub const FIRST_PORT: u16 = 7788;

pub fn now() -> u64 { std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0) }
pub fn seen() { LAST_SEEN.store(now(), Ordering::Relaxed); }

/// How long the app waits with no page open before it ends: $TRINETRA_APP_IDLE_MINUTES
/// (a whole number 1..=10080), else 5 minutes. Anything else is refused, never guessed.
fn idle_limit_s() -> Result<u64, Start> {
    match std::env::var("TRINETRA_APP_IDLE_MINUTES").ok().filter(|m| !m.trim().is_empty()) {
        None => Ok(5 * 60),
        Some(m) => match m.trim().parse::<u64>() {
            Ok(n) if (1..=10080).contains(&n) => Ok(n * 60),
            _ => Err(Start::Failed(format!("TRINETRA_APP_IDLE_MINUTES={m} is not a whole number of minutes from 1 to 10080"))),
        },
    }
}

/// Is a TRI-NETRA app already answering on `port`? (It says so in /v1/version.)
fn running_on(port: u16) -> bool {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let Ok(mut s) = TcpStream::connect_timeout(&addr, Duration::from_millis(300)) else { return false };
    let _ = s.set_read_timeout(Some(Duration::from_secs(1)));
    use std::io::Write;
    if s.write_all(format!("GET /v1/version HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n").as_bytes()).is_err() { return false; }
    let mut b = Vec::new();
    let _ = s.take(8192).read_to_end(&mut b);
    String::from_utf8_lossy(&b).contains("\"app\":\"trinetra\"")
}

pub fn open_browser(url: &str) {
    if std::env::var_os("TRINETRA_NO_BROWSER").is_some_and(|v| !v.is_empty()) { return; }
    #[cfg(target_os = "windows")]
    let r = std::process::Command::new("rundll32").args(["url.dll,FileProtocolHandler", url]).spawn();
    #[cfg(target_os = "macos")]
    let r = std::process::Command::new("open").arg(url).spawn();
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let r = std::process::Command::new("xdg-open").arg(url).spawn();
    let _ = r;
}

/// Say why the app could not start, on a page kept in the log folder, and open it.
fn start_failed(why: &str) {
    eprintln!("TRI-NETRA ADCS: {why}");
    let page = format!("<!doctype html><meta charset=utf-8><title>TRI-NETRA ADCS could not start</title>\
        <body style=\"font:16px system-ui;max-width:40em;margin:3em auto;padding:0 1em\">\
        <h1>TRI-NETRA ADCS could not start</h1><p><b>{}</b></p>\
        <p>Nothing was changed. See FIRST_RUN.md beside the program for what to do; this page is kept in {}.</p>",
        http::esc(why), http::esc(&adcs_sim::fsio::log_dir().display().to_string()));
    let f = adcs_sim::fsio::log_dir().join("start-failed.html");
    if adcs_sim::fsio::write(&f, page).is_ok() { open_browser(&format!("file://{}", f.display())); }
}

/// Why the app does not start serving: another copy already answers (open it instead), or a
/// reason to show the person.
enum Start { AlreadyRunning(u16), Failed(String) }

fn bind() -> Result<(TcpListener, u16), Start> {
    let first = match std::env::var("TRINETRA_PORT").ok().filter(|p| !p.trim().is_empty()) {
        Some(p) => p.trim().parse::<u16>().map_err(|_| Start::Failed(format!("TRINETRA_PORT={p} is not a port number")))?,
        None => FIRST_PORT,
    };
    for port in first..first.saturating_add(12) {
        if running_on(port) { return Err(Start::AlreadyRunning(port)); }
        if let Ok(l) = TcpListener::bind(("127.0.0.1", port)) { return Ok((l, port)); }
    }
    Err(Start::Failed(format!("nothing in {}..{} was free on 127.0.0.1. Set TRINETRA_PORT to choose another.", first, first.saturating_add(11))))
}

/// The design database the app reads its cases and scenarios from: $TRINETRA_DESIGN, else
/// `design.tndb` beside the data (a kit, the Python package); none (a checkout), and it reads the
/// data folder's files.
pub fn design_file(root: &std::path::Path) -> Option<std::path::PathBuf> {
    if let Some(p) = std::env::var_os("TRINETRA_DESIGN").filter(|p| !p.is_empty()) { return Some(p.into()); }
    Some(root.join("design.tndb")).filter(|p| p.is_file())
}

fn main() {
    adcs_sim::fsio::install_crash_report("trinetra-app");
    let root = adcs_sim::data_root();
    if let Some(f) = design_file(&root) {
        match adcs_sim::source::Design::open(&f) {
            Ok(d) => adcs_sim::source::use_design(Some(d)),
            Err(e) => { start_failed(&format!("{e}. Nothing was flown.")); std::process::exit(1); }
        }
    }
    if adcs_sim::source::current().ok().flatten().is_none() && !root.join("data/scenarios").is_dir() {
        start_failed(&format!("the tool's data is not beside the program (looked for data/scenarios in {}). Unzip the whole kit and open the program from it.", root.display()));
        std::process::exit(1);
    }
    let limit = match idle_limit_s() {
        Ok(l) => l,
        Err(Start::Failed(e)) => { start_failed(&e); std::process::exit(1); }
        Err(Start::AlreadyRunning(_)) => unreachable!("only bind() finds another copy running"),
    };
    let (listener, port) = match bind() {
        Ok(x) => x,
        Err(Start::AlreadyRunning(port)) => { open_browser(&format!("http://127.0.0.1:{port}")); return; }
        Err(Start::Failed(e)) => { start_failed(&e); std::process::exit(1); }
    };
    let url = format!("http://127.0.0.1:{port}");
    println!("TRI-NETRA ADCS at {url} -- Quit on the page ends it");
    seen();
    open_browser(&url);
    let _ = listener.set_nonblocking(true);
    loop {
        if QUIT.load(Ordering::Relaxed) { std::thread::sleep(Duration::from_millis(300)); return; }
        if now().saturating_sub(LAST_SEEN.load(Ordering::Relaxed)) > limit { return; }
        // at the cap, nothing more is accepted: the next connection waits in the kernel's queue
        // until one finishes (only this loop adds to OPEN, so the count cannot overshoot)
        if OPEN.load(Ordering::Acquire) >= MAX_OPEN { std::thread::sleep(Duration::from_millis(20)); continue; }
        match listener.accept() {
            Ok((s, _)) => {
                OPEN.fetch_add(1, Ordering::AcqRel);
                let spawned = std::thread::Builder::new().spawn(move || {
                    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| http::serve(s, port)));
                    OPEN.fetch_sub(1, Ordering::AcqRel);
                });
                // no thread to serve it: the connection is closed and the slot given back
                if spawned.is_err() { OPEN.fetch_sub(1, Ordering::AcqRel); }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => std::thread::sleep(Duration::from_millis(50)),
            Err(_) => std::thread::sleep(Duration::from_millis(200)),
        }
    }
}
