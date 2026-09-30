//! The results store: every run directory the engine writes (adcs-rec/1) says where it
//! came from, and can be found, read and sent as one file.
//!
//! - **Provenance.** A run's manifest records when it ran, the engine version, and a
//!   fingerprint of exactly what it flew: the case file's bytes, the scenario after
//!   every override, the overrides, the seed and the flight software. `result_id` is
//!   the fingerprint of all of them, so two runs with the same `result_id` flew the same
//!   inputs on the same engine.
//! - **list / show.** `adcs results list [DIR]` finds every run under a folder (the
//!   engine's store by default) and prints one line each; `show` prints one run's
//!   provenance and requirement metrics.
//! - **pin / thin.** `pin` marks a run to keep. `thin --older-than DAYS` removes the time
//!   series (channels.csv, the bulk of a run) from runs older than that which are not pinned;
//!   the manifest stays, so the verdicts and the provenance remain, and `show` gives the
//!   command that flies the same run again.
//! - **export / import.** `adcs results export <run> --out F.trinetra` writes the run's
//!   manifest and channels, with a README, into one zip file any unzip tool opens;
//!   `import` puts one back into a folder.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// FNV-1a, 64 bit: the store's fingerprint (an identity, not a security measure).
pub fn fnv(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in bytes { h ^= *b as u64; h = h.wrapping_mul(0x100000001b3); }
    h
}
pub fn hex(h: u64) -> String { format!("{h:016x}") }

/// The provenance block of a run's manifest.
pub fn provenance(c: &crate::config::Config, fsw: &str) -> Value {
    let case_bytes = std::fs::read(&c.case.file).unwrap_or_default();
    let scen = serde_json::to_string(&c.scenario).unwrap_or_default();
    let (case_h, scen_h) = (fnv(&case_bytes), fnv(scen.as_bytes()));
    let ov: Vec<String> = c.overrides.iter().map(|(k, v)| format!("{k}={v}")).collect();
    let id = fnv(format!("{}|{}|{}|{}|{}|{}", env!("CARGO_PKG_VERSION"), hex(case_h), hex(scen_h), ov.join(";"), c.seed, fsw).as_bytes());
    // paths inside the data folder are recorded relative to it, so a manifest names no machine
    let root = crate::data_root();
    let rel = |p: &str| -> String {
        let pb = Path::new(p);
        let abs = std::fs::canonicalize(pb).unwrap_or_else(|_| pb.to_path_buf());
        let r = std::fs::canonicalize(&root).unwrap_or_else(|_| root.clone());
        abs.strip_prefix(&r).map(|x| x.display().to_string().replace('\\', "/")).unwrap_or_else(|_| p.to_string())
    };
    json!({
        "result_id": hex(id),
        "created_utc": crate::fsio::utc_now().1,
        "engine_version": env!("CARGO_PKG_VERSION"),
        "inputs": {"case_file": rel(&c.case.file), "case_fingerprint": hex(case_h), "scenario_file": rel(&c.scenario_file),
                   "scenario_fingerprint": hex(scen_h), "overrides": ov, "seed": c.seed, "fsw": fsw},
    })
}

/// One run found in a store.
#[derive(Debug, Clone)]
pub struct Found { pub dir: PathBuf, pub m: Value }

/// The file that marks a run as kept: `thin` never touches a run that has it.
pub const PIN: &str = "PINNED";

impl Found {
    pub fn pinned(&self) -> bool { self.dir.join(PIN).is_file() }
    /// Thinned: the manifest without its time series.
    pub fn thinned(&self) -> bool { !self.dir.join("channels.csv").is_file() }
    fn s(&self, k: &str) -> String { self.m.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string() }
    /// (passed, failed) requirement metrics.
    pub fn verdicts(&self) -> (usize, usize) {
        let ms = self.m.get("metrics").and_then(|v| v.as_array()).cloned().unwrap_or_default();
        let p = ms.iter().filter(|x| x["pass"].as_i64() == Some(1)).count();
        let f = ms.iter().filter(|x| x["pass"].as_i64() == Some(0)).count();
        (p, f)
    }
}

/// Every adcs-rec/1 run directory under `root`, sorted by path.
pub fn list(root: &Path) -> Result<Vec<Found>, String> {
    if !root.is_dir() { return Err(format!("{} is not a folder", root.display())); }
    let mut out = vec![];
    let mut stack = vec![root.to_path_buf()];
    while let Some(d) = stack.pop() {
        let rd = match std::fs::read_dir(&d) { Ok(r) => r, Err(_) => continue };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() { stack.push(p); continue; }
            if p.file_name().and_then(|n| n.to_str()) != Some("manifest.json") { continue; }
            let m: Value = match std::fs::read_to_string(&p).ok().and_then(|s| serde_json::from_str(&s).ok()) { Some(m) => m, None => continue };
            if m.get("schema").and_then(|v| v.as_str()) != Some("adcs-rec/1") { continue; }
            out.push(Found { dir: d.clone(), m });
        }
    }
    out.sort_by(|a, b| a.dir.cmp(&b.dir));
    Ok(out)
}

/// One line per run: scenario, case, verdicts, when, where.
pub fn table(found: &[Found], root: &Path) -> String {
    let mut s = format!("{:<34} {:<12} {:>9}  {:<20} {:<7} {}\n", "scenario", "case", "pass/fail", "created (UTC)", "kept", "folder");
    for f in found {
        let (p, x) = f.verdicts();
        let when = f.s("created_utc");
        let rel = f.dir.strip_prefix(root).unwrap_or(&f.dir).display().to_string();
        let kept = if f.pinned() { "pinned" } else if f.thinned() { "thinned" } else { "" };
        s += &format!("{:<34} {:<12} {:>4}/{:<4}  {:<20} {:<7} {}\n", f.s("scenario"), f.s("case"), p, x, if when.is_empty() { "(before provenance)".into() } else { when }, kept, rel);
    }
    s + &format!("{} run(s)\n", found.len())
}

/// A run's provenance and requirement metrics, for a person to read.
pub fn show(dir: &Path) -> Result<String, String> {
    let p = dir.join("manifest.json");
    let m: Value = serde_json::from_str(&std::fs::read_to_string(&p).map_err(|e| format!("{}: {e}", p.display()))?)
        .map_err(|e| format!("{}: {e}", p.display()))?;
    if m.get("schema").and_then(|v| v.as_str()) != Some("adcs-rec/1") { return Err(format!("{} is not a run (schema adcs-rec/1)", p.display())); }
    let g = |k: &str| m.get(k).map(|v| v.as_str().map(String::from).unwrap_or_else(|| v.to_string())).unwrap_or_else(|| "—".into());
    let mut s = format!("{} on {} ({}), product {}, fsw {}\n", g("scenario"), g("case"), g("case_title"), g("product"),
        m["fsw"]["impl"].as_str().unwrap_or("?"));
    s += &format!("  result id     {}\n  created       {}\n  engine        {}\n", g("result_id"), g("created_utc"), g("engine"));
    if let Some(i) = m.get("inputs") {
        s += &format!("  case          {} (fingerprint {})\n  scenario      {} (fingerprint {})\n  seed          {}\n",
            i["case_file"].as_str().unwrap_or("?"), i["case_fingerprint"].as_str().unwrap_or("?"),
            i["scenario_file"].as_str().unwrap_or("?"), i["scenario_fingerprint"].as_str().unwrap_or("?"), i["seed"]);
        let ov: Vec<&str> = i["overrides"].as_array().map(|a| a.iter().filter_map(|x| x.as_str()).collect()).unwrap_or_default();
        s += &format!("  overrides     {}\n", if ov.is_empty() { "none".into() } else { ov.join(" ") });
    } else {
        s += "  (written before runs recorded their inputs: run it again for its provenance)\n";
    }
    if dir.join(PIN).is_file() { s += "  kept          pinned: `adcs results thin` leaves it whole\n"; }
    if !dir.join("channels.csv").is_file() {
        s += "  thinned       the time series was removed; the verdicts and provenance below remain\n";
        if let Some(i) = m.get("inputs") {
            let ov: String = i["overrides"].as_array().map(|a| a.iter().filter_map(|x| x.as_str()).map(|o| format!(" --set {o}")).collect()).unwrap_or_default();
            s += &format!("  fly it again  adcs run {} --case {} --seed {}{ov}\n", g("scenario"), i["case_file"].as_str().unwrap_or("?"), i["seed"]);
        }
    }
    s += "  requirement metrics:\n";
    for x in m["metrics"].as_array().cloned().unwrap_or_default() {
        if x["req"].is_null() { continue; }
        let v = x["value"].as_f64().map(|v| format!("{v:.4}")).unwrap_or("NaN".into());
        let p = match x["pass"].as_i64() { Some(1) => "PASS", Some(_) => "FAIL", None => "" };
        s += &format!("    {:<26} {:>12} {:<6} req {:<8} {}\n", x["id"].as_str().unwrap_or(""), v, x["unit"].as_str().unwrap_or(""), x["req"], p);
    }
    Ok(s)
}

// ---- the .trinetra share file: a zip (stored, no compression), readable by any unzip ----

fn crc32(data: &[u8]) -> u32 {
    let mut table = [0u32; 256];
    for (i, t) in table.iter_mut().enumerate() {
        let mut c = i as u32;
        for _ in 0..8 { c = if c & 1 != 0 { 0xEDB88320 ^ (c >> 1) } else { c >> 1 }; }
        *t = c;
    }
    let mut c = 0xFFFF_FFFFu32;
    for b in data { c = table[((c ^ *b as u32) & 0xFF) as usize] ^ (c >> 8); }
    c ^ 0xFFFF_FFFF
}

fn zip(files: &[(String, Vec<u8>)]) -> Vec<u8> {
    let (mut out, mut central) = (Vec::new(), Vec::new());
    let le16 = |v: &mut Vec<u8>, x: u16| v.extend_from_slice(&x.to_le_bytes());
    let le32 = |v: &mut Vec<u8>, x: u32| v.extend_from_slice(&x.to_le_bytes());
    for (name, data) in files {
        let (crc, n, off) = (crc32(data), data.len() as u32, out.len() as u32);
        // local header: version 20, no flags, stored, a fixed date (1 Jan 2026) so the file is deterministic
        le32(&mut out, 0x04034b50); le16(&mut out, 20); le16(&mut out, 0); le16(&mut out, 0); le16(&mut out, 0); le16(&mut out, 0x5C21);
        le32(&mut out, crc); le32(&mut out, n); le32(&mut out, n); le16(&mut out, name.len() as u16); le16(&mut out, 0);
        out.extend_from_slice(name.as_bytes()); out.extend_from_slice(data);
        le32(&mut central, 0x02014b50); le16(&mut central, 20); le16(&mut central, 20); le16(&mut central, 0); le16(&mut central, 0);
        le16(&mut central, 0); le16(&mut central, 0x5C21); le32(&mut central, crc); le32(&mut central, n); le32(&mut central, n);
        le16(&mut central, name.len() as u16); le16(&mut central, 0); le16(&mut central, 0); le16(&mut central, 0); le16(&mut central, 0);
        le32(&mut central, 0o644 << 16); le32(&mut central, off); central.extend_from_slice(name.as_bytes());
    }
    let (cd_off, cd_len) = (out.len() as u32, central.len() as u32);
    out.extend_from_slice(&central);
    le32(&mut out, 0x06054b50); le16(&mut out, 0); le16(&mut out, 0); le16(&mut out, files.len() as u16); le16(&mut out, files.len() as u16);
    le32(&mut out, cd_len); le32(&mut out, cd_off); le16(&mut out, 0);
    out
}

fn unzip(bytes: &[u8]) -> Result<Vec<(String, Vec<u8>)>, String> {
    let rd16 = |i: usize| -> Result<usize, String> { bytes.get(i..i + 2).map(|b| u16::from_le_bytes([b[0], b[1]]) as usize).ok_or("the file ends early".into()) };
    let rd32 = |i: usize| -> Result<usize, String> { bytes.get(i..i + 4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as usize).ok_or("the file ends early".into()) };
    let (mut i, mut out) = (0usize, vec![]);
    while i + 4 <= bytes.len() && rd32(i)? == 0x04034b50 {
        let (method, size, nlen, xlen) = (rd16(i + 8)?, rd32(i + 18)?, rd16(i + 26)?, rd16(i + 28)?);
        if method != 0 { return Err("a compressed entry: this is not a file `adcs results export` wrote".into()); }
        let name = String::from_utf8(bytes.get(i + 30..i + 30 + nlen).ok_or("the file ends early")?.to_vec()).map_err(|_| "a name that is not text")?;
        let start = i + 30 + nlen + xlen;
        let data = bytes.get(start..start + size).ok_or("the file ends early")?.to_vec();
        if crc32(&data) as usize != rd32(i + 14)? { return Err(format!("{name} is damaged (its checksum does not match)")); }
        out.push((name, data));
        i = start + size;
    }
    if out.is_empty() { return Err("not a .trinetra file".into()); }
    Ok(out)
}

/// Mark a run to keep (`on`), or let `thin` treat it like any other.
pub fn pin(dir: &Path, on: bool) -> Result<(), String> {
    if !dir.join("manifest.json").is_file() { return Err(format!("{} is not a run", dir.display())); }
    let f = dir.join(PIN);
    if on { crate::fsio::write(&f, "kept: `adcs results thin` leaves this run whole\n") } else { match std::fs::remove_file(&f) { Ok(()) => Ok(()), Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()), Err(e) => Err(format!("{}: {e}", f.display())) } }
}

/// Seconds since 1970 of a manifest's `created_utc` (YYYY-MM-DDTHH:MM:SSZ), or None.
pub fn created_secs(m: &Value) -> Option<u64> {
    let s = m.get("created_utc")?.as_str()?;
    let n = |a: usize, b: usize| s.get(a..b)?.parse::<i64>().ok();
    let (y, mo, d, h, mi, se) = (n(0, 4)?, n(5, 7)?, n(8, 10)?, n(11, 13)?, n(14, 16)?, n(17, 19)?);
    // days from civil (Hinnant), the inverse of fsio::utc_now
    let y2 = if mo <= 2 { y - 1 } else { y };
    let era = y2.div_euclid(400);
    let yoe = y2 - era*400;
    let doy = (153*(if mo > 2 { mo - 3 } else { mo + 9 }) + 2)/5 + d - 1;
    let doe = yoe*365 + yoe/4 - yoe/100 + doy;
    let days = era*146097 + doe - 719468;
    u64::try_from(days*86400 + h*3600 + mi*60 + se).ok()
}

/// Thin every run under `root` older than `days` that is not pinned: remove its time series,
/// keep its manifest. Returns the runs thinned and the bytes freed; `dry` changes nothing.
pub fn thin(root: &Path, days: u64, dry: bool) -> Result<(Vec<PathBuf>, u64), String> {
    let now = crate::fsio::utc_now().0;
    let (mut done, mut freed) = (vec![], 0u64);
    for f in list(root)? {
        if f.pinned() || f.thinned() { continue; }
        let Some(t) = created_secs(&f.m) else { continue };   // a run with no date is left alone
        if now.saturating_sub(t) < days*86400 { continue; }
        let c = f.dir.join("channels.csv");
        freed += std::fs::metadata(&c).map(|m| m.len()).unwrap_or(0);
        if !dry { std::fs::remove_file(&c).map_err(|e| format!("{}: {e}", c.display()))?; }
        done.push(f.dir.clone());
    }
    Ok((done, freed))
}

/// Write a run as one `.trinetra` file.
pub fn export(dir: &Path, out: &Path) -> Result<usize, String> {
    let summary = show(dir)?;
    let mut files = vec![];
    for n in ["manifest.json", "channels.csv"] {
        let p = dir.join(n);
        if n == "channels.csv" && !p.is_file() { continue; }   // a thinned run sends its verdicts and provenance
        files.push((n.to_string(), std::fs::read(&p).map_err(|e| format!("{}: {e}", p.display()))?));
    }
    files.insert(0, ("README.txt".into(), format!("A TRI-NETRA ADCS run, exported by `adcs results export`.\n\
        Open it with `adcs results import <file> --out <folder>`, or unzip it: manifest.json is the run's\n\
        provenance and metrics, channels.csv its time series.\n\n{summary}").into_bytes()));
    let z = zip(&files);
    crate::fsio::write(out, &z)?;
    Ok(z.len())
}

/// Put a `.trinetra` file back into a folder, refusing any name that is not a plain file name.
pub fn import(file: &Path, out: &Path) -> Result<usize, String> {
    let files = unzip(&std::fs::read(file).map_err(|e| format!("{}: {e}", file.display()))?).map_err(|e| format!("{}: {e}", file.display()))?;
    for (name, data) in &files {
        if name.contains('/') || name.contains('\\') || name.starts_with('.') || name.is_empty() {
            return Err(format!("{}: refuses entry {name:?}, which is not a plain file name", file.display()));
        }
        crate::fsio::write(&out.join(name), data)?;
    }
    Ok(files.len())
}

#[cfg(test)]
mod t {
    use super::*;
    #[test]
    fn crc32_matches_the_standard() { assert_eq!(crc32(b"123456789"), 0xCBF43926); }
    #[test]
    fn a_zip_round_trips_and_a_damaged_one_is_refused() {
        let files = vec![("a.txt".to_string(), b"hello".to_vec()), ("b.csv".to_string(), vec![1, 2, 3])];
        let mut z = zip(&files);
        assert_eq!(unzip(&z).unwrap(), files);
        let n = z.len();
        z[n / 3] ^= 0xFF;
        assert!(unzip(&z).is_err());
    }
    #[test]
    fn an_entry_that_is_a_path_is_refused_on_import() {
        let d = std::env::temp_dir().join(format!("adcs-store-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        let f = d.join("bad.trinetra");
        std::fs::write(&f, zip(&[("../evil".to_string(), b"x".to_vec())])).unwrap();
        assert!(import(&f, &d.join("out")).unwrap_err().contains("not a plain file name"));
        let _ = std::fs::remove_dir_all(&d);
    }
    #[test]
    fn a_manifest_date_reads_back_to_its_second() {
        let (s, w) = crate::fsio::utc_now();
        assert_eq!(created_secs(&serde_json::json!({"created_utc": w})), Some(s));
        assert_eq!(created_secs(&serde_json::json!({"created_utc": "1970-01-02T00:00:00Z"})), Some(86400));
        assert_eq!(created_secs(&serde_json::json!({})), None);
    }
    #[test]
    fn thin_keeps_the_manifest_and_leaves_pinned_and_recent_runs_whole() {
        let d = std::env::temp_dir().join(format!("adcs-thin-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        for (name, when) in [("old", "2020-01-01T00:00:00Z"), ("kept", "2020-01-01T00:00:00Z"), ("new", crate::fsio::utc_now().1.as_str())] {
            let r = d.join(name);
            std::fs::create_dir_all(&r).unwrap();
            std::fs::write(r.join("manifest.json"), serde_json::json!({"schema": "adcs-rec/1", "created_utc": when, "metrics": []}).to_string()).unwrap();
            std::fs::write(r.join("channels.csv"), "t_s\n0\n").unwrap();
        }
        pin(&d.join("kept"), true).unwrap();
        let (dry, _) = thin(&d, 30, true).unwrap();
        assert!(d.join("old/channels.csv").is_file() && dry.len() == 1, "a dry run changes nothing");
        let (done, freed) = thin(&d, 30, false).unwrap();
        assert_eq!(done, vec![d.join("old")]);
        assert!(freed > 0 && d.join("old/manifest.json").is_file() && !d.join("old/channels.csv").exists());
        assert!(d.join("kept/channels.csv").is_file() && d.join("new/channels.csv").is_file());
        assert!(show(&d.join("old")).unwrap().contains("thinned"));
        let _ = std::fs::remove_dir_all(&d);
    }
    #[test]
    fn the_fingerprint_is_fnv1a() { assert_eq!(hex(fnv(b"a")), "af63dc4c8601ec8c"); }
}
