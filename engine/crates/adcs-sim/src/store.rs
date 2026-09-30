//! The results store: every run directory the engine writes (adcs-rec/1) says where it
//! came from, and can be found, read and sent as one file.
//!
//! - **Provenance.** A run's manifest records when it ran, the engine version, and a
//!   fingerprint of exactly what it flew: the case file's bytes, the scenario after
//!   every override, the overrides, the seed and the flight software. `result_id` is
//!   the fingerprint of all of them, so two runs with the same `result_id` flew the same
//!   inputs on the same engine.
//! - **Inputs kept once.** A run keeps a copy of the case file and the scenario file it flew,
//!   named by their fingerprints, in the store's `inputs/` folder (or the run's own
//!   `inputs/` outside a store): a hundred runs of one case keep one copy, and a run can be
//!   flown again exactly after the case or the scenario has changed.
//! - **list / show.** `adcs results list [DIR]` finds every run under a folder (the
//!   engine's store by default) and prints one line each; `show` prints one run's
//!   provenance and requirement metrics. `list` keeps an index (`.adcs-index.json`) of what
//!   it read from each manifest, keyed by the manifest's size and time, so it reads only
//!   runs that are new or changed; the index can never be stale, only slower.
//! - **pin / thin.** `pin` marks a run to keep. `thin --older-than DAYS` removes the time
//!   series (channels.csv, the bulk of a run) from runs older than that which are not pinned;
//!   the manifest stays, so the verdicts and the provenance remain, and `show` gives the
//!   command that flies the same run again.
//! - **export / import.** `adcs results export <run> --out F.trinetra` writes the run's
//!   manifest and channels, with a README, into one zip file any unzip tool opens;
//!   `import` puts one back into a folder.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use crate::error::Error;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// FNV-1a, 64 bit: the store's fingerprint (an identity, not a security measure).
pub fn fnv(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in bytes { h ^= *b as u64; h = h.wrapping_mul(0x100000001b3); }
    h
}
pub fn hex(h: u64) -> String { format!("{h:016x}") }

/// The name a kept input has: `case-<fingerprint>.csv`, `scenario-<fingerprint>.json`.
pub fn input_name(kind: &str, fp: &str, ext: &str) -> String { format!("{kind}-{fp}.{ext}") }

/// Where a run under `dir` keeps its inputs: the store's `inputs/` when `dir` is in the
/// engine's store, else `dir/inputs`.
pub fn inputs_dir(dir: &Path) -> PathBuf {
    let store = crate::store_root();
    let canon = |p: &Path| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    let (d, s) = (canon(dir), canon(&store));
    if d.starts_with(&s) { s.join("inputs") } else { dir.join("inputs") }
}

/// Keep one copy of `bytes` as `name` in `inputs`: written once, and refused if a file of
/// that name is already there with other contents (a fingerprint that names two files).
pub fn keep_input(inputs: &Path, name: &str, bytes: &[u8]) -> Result<PathBuf, Error> {
    let f = inputs.join(name);
    match std::fs::read(&f) {
        Ok(have) if have == bytes => Ok(f),
        Ok(_) => Err(Error::malformed(format!("{}: already holds other contents under the same fingerprint; refusing to overwrite it", f.display()))),
        Err(_) => { crate::fsio::write(&f, bytes)?; Ok(f) }
    }
}

/// A kept input of the run in `dir`: `dir/<name>` (an imported run) or `<ancestor>/inputs/<name>`.
pub fn find_input(dir: &Path, name: &str) -> Option<PathBuf> {
    if dir.join(name).is_file() { return Some(dir.join(name)); }
    dir.ancestors().map(|a| a.join("inputs").join(name)).find(|p| p.is_file())
}

/// The provenance block of a run's manifest.
pub fn provenance(c: &crate::config::Config, fsw: &str) -> Value {
    let case_bytes = std::fs::read(&c.case.file).unwrap_or_default();
    let scen_file_h = fnv(&std::fs::read(&c.scenario_file).unwrap_or_default());
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
                   "scenario_fingerprint": hex(scen_h), "scenario_file_fingerprint": hex(scen_file_h),
                   "overrides": ov, "seed": c.seed, "fsw": fsw},
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

/// The index `list` keeps in the folder it lists.
pub const INDEX: &str = ".adcs-index.json";

/// What `list` keeps of a manifest: enough for the table, `thin` and the app.
fn summary(m: &Value) -> Value {
    let metrics: Vec<Value> = m["metrics"].as_array().map(|a| a.iter().map(|x| json!({"pass": x["pass"]})).collect()).unwrap_or_default();
    json!({"schema": m["schema"], "scenario": m["scenario"], "case": m["case"], "created_utc": m["created_utc"],
           "result_id": m["result_id"], "fsw": {"impl": m["fsw"]["impl"]}, "metrics": metrics})
}

/// A manifest's identity for the index: its size and modification time.
fn stamp(p: &Path) -> Option<String> {
    let md = std::fs::metadata(p).ok()?;
    let t = md.modified().ok()?.duration_since(std::time::UNIX_EPOCH).ok()?;
    Some(format!("{}:{}.{:09}", md.len(), t.as_secs(), t.subsec_nanos()))
}

/// Every adcs-rec/1 run directory under `root`, sorted by path. A manifest whose size and
/// time the index already holds is not read again; the index is rewritten when it changed
/// (and left alone where the folder cannot be written).
pub fn list(root: &Path) -> Result<Vec<Found>, Error> {
    if !root.is_dir() { return Err(Error::refused(format!("{} is not a folder", root.display()))); }
    let ix_path = root.join(INDEX);
    let old: serde_json::Map<String, Value> = std::fs::read_to_string(&ix_path).ok()
        .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        .filter(|v| v["schema"] == "adcs-index/1")
        .and_then(|v| v["runs"].as_object().cloned()).unwrap_or_default();
    let mut new = serde_json::Map::new();
    let mut out = vec![];
    let mut stack = vec![root.to_path_buf()];
    while let Some(d) = stack.pop() {
        let rd = match std::fs::read_dir(&d) { Ok(r) => r, Err(_) => continue };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() { stack.push(p); continue; }
            if p.file_name().and_then(|n| n.to_str()) != Some("manifest.json") { continue; }
            let key = d.strip_prefix(root).unwrap_or(&d).display().to_string().replace('\\', "/");
            let Some(st) = stamp(&p) else { continue };
            let known = old.get(&key).filter(|v| v["stamp"].as_str() == Some(st.as_str())).map(|v| v["m"].clone());
            let m = match known {
                Some(m) => m,
                None => match std::fs::read_to_string(&p).ok().and_then(|s| serde_json::from_str::<Value>(&s).ok()) {
                    Some(m) => summary(&m),
                    None => continue,
                },
            };
            if m.get("schema").and_then(|v| v.as_str()) != Some("adcs-rec/1") { continue; }
            new.insert(key, json!({"stamp": st, "m": m.clone()}));
            out.push(Found { dir: d.clone(), m });
        }
    }
    if new != old {
        let _ = crate::fsio::write(&ix_path, serde_json::to_string(&json!({"schema": "adcs-index/1", "runs": new})).unwrap_or_default() + "\n");
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
pub fn show(dir: &Path) -> Result<String, Error> {
    let p = dir.join("manifest.json");
    if !p.is_file() { return Err(Error::refused(format!("{} is not a run: it has no manifest.json", dir.display()))); }
    let m: Value = serde_json::from_str(&std::fs::read_to_string(&p).map_err(|e| Error::io(&p, e))?)
        .map_err(|e| Error::malformed(format!("{}: {e}", p.display())))?;
    if m.get("schema").and_then(|v| v.as_str()) != Some("adcs-rec/1") { return Err(Error::refused(format!("{} is not a run (schema adcs-rec/1)", p.display()))); }
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
        if m.get("inputs").is_some() { s += &fly_again(dir, &m); }
    }
    if dir.join("channels.csv").is_file() && m.get("inputs").is_some() { s += &fly_again(dir, &m); }
    s += "  requirement metrics:\n";
    for x in m["metrics"].as_array().cloned().unwrap_or_default() {
        if x["req"].is_null() { continue; }
        let v = x["value"].as_f64().map(|v| format!("{v:.4}")).unwrap_or("NaN".into());
        let p = match x["pass"].as_i64() { Some(1) => "PASS", Some(_) => "FAIL", None => "" };
        s += &format!("    {:<26} {:>12} {:<6} req {:<8} {}\n", x["id"].as_str().unwrap_or(""), v, x["unit"].as_str().unwrap_or(""), x["req"], p);
    }
    Ok(s)
}

/// The command that flies the run in `dir` again: from its kept inputs when they are there,
/// else from the files it names (which may have changed since).
fn fly_again(dir: &Path, m: &Value) -> String {
    let i = &m["inputs"];
    let ov: String = i["overrides"].as_array().map(|a| a.iter().filter_map(|x| x.as_str()).map(|o| format!(" --set {o}")).collect()).unwrap_or_default();
    let kept = |kind: &str, key: &str, ext: &str| i[key].as_str().and_then(|fp| find_input(dir, &input_name(kind, fp, ext)));
    let (case, scen) = (kept("case", "case_fingerprint", "csv"), kept("scenario", "scenario_file_fingerprint", "json"));
    match (case, scen) {
        (Some(c), Some(sc)) => format!("  fly it again  adcs run {} --case {} --seed {}{ov}\n                (the inputs it flew, kept by fingerprint)\n",
            sc.display(), c.display(), i["seed"]),
        _ => format!("  fly it again  adcs run {} --case {} --seed {}{ov}\n                (the files it names: they may have changed since; this run kept no copy)\n",
            m["scenario"].as_str().unwrap_or("?"), i["case_file"].as_str().unwrap_or("?"), i["seed"]),
    }
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
pub fn pin(dir: &Path, on: bool) -> Result<(), Error> {
    if !dir.join("manifest.json").is_file() { return Err(Error::refused(format!("{} is not a run", dir.display()))); }
    let f = dir.join(PIN);
    if on { crate::fsio::write(&f, "kept: `adcs results thin` leaves this run whole\n") } else { match std::fs::remove_file(&f) { Ok(()) => Ok(()), Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()), Err(e) => Err(Error::io(&f, e)) } }
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
pub fn thin(root: &Path, days: u64, dry: bool) -> Result<(Vec<PathBuf>, u64), Error> {
    let now = crate::fsio::utc_now().0;
    let (mut done, mut freed) = (vec![], 0u64);
    for f in list(root)? {
        if f.pinned() || f.thinned() { continue; }
        let Some(t) = created_secs(&f.m) else { continue };   // a run with no date is left alone
        if now.saturating_sub(t) < days*86400 { continue; }
        let c = f.dir.join("channels.csv");
        freed += std::fs::metadata(&c).map(|m| m.len()).unwrap_or(0);
        if !dry { std::fs::remove_file(&c).map_err(|e| Error::io(&c, e))?; }
        done.push(f.dir.clone());
    }
    Ok((done, freed))
}

/// Write a run as one `.trinetra` file.
pub fn export(dir: &Path, out: &Path) -> Result<usize, Error> {
    let summary = show(dir)?;
    let mut files = vec![];
    for n in ["manifest.json", "channels.csv"] {
        let p = dir.join(n);
        if n == "channels.csv" && !p.is_file() { continue; }   // a thinned run sends its verdicts and provenance
        files.push((n.to_string(), std::fs::read(&p).map_err(|e| Error::io(&p, e))?));
    }
    if let Ok(m) = serde_json::from_slice::<Value>(&files[0].1) {
        for (kind, key, ext) in [("case", "case_fingerprint", "csv"), ("scenario", "scenario_file_fingerprint", "json")] {
            let Some(fp) = m["inputs"][key].as_str() else { continue };
            let name = input_name(kind, fp, ext);
            if let Some(p) = find_input(dir, &name) { files.push((name, std::fs::read(&p).map_err(|e| Error::io(&p, e))?)); }
        }
    }
    files.insert(0, ("README.txt".into(), format!("A TRI-NETRA ADCS run, exported by `adcs results export`.\n\
        Open it with `adcs results import <file> --out <folder>`, or unzip it: manifest.json is the run's\n\
        provenance and metrics, channels.csv its time series, case-*.csv and scenario-*.json the inputs it flew.\n\n{summary}").into_bytes()));
    let z = zip(&files);
    crate::fsio::write(out, &z)?;
    Ok(z.len())
}

/// Put a `.trinetra` file back into a folder, refusing any name that is not a plain file name.
pub fn import(file: &Path, out: &Path) -> Result<usize, Error> {
    let files = unzip(&std::fs::read(file).map_err(|e| Error::io(file, e))?).map_err(|e| Error::malformed(format!("{}: {e}", file.display())))?;
    for (name, data) in &files {
        if name.contains('/') || name.contains('\\') || name.starts_with('.') || name.is_empty() {
            return Err(Error::refused(format!("{}: refuses entry {name:?}, which is not a plain file name", file.display())));
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
        assert!(import(&f, &d.join("out")).unwrap_err().message().contains("not a plain file name"));
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
    fn an_input_is_kept_once_and_a_clash_is_refused() {
        let d = std::env::temp_dir().join(format!("adcs-keep-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        let f = keep_input(&d, "case-1.csv", b"a,b\n").unwrap();
        assert_eq!(keep_input(&d, "case-1.csv", b"a,b\n").unwrap(), f, "the same bytes again keep the one copy");
        assert!(keep_input(&d, "case-1.csv", b"other").unwrap_err().message().contains("refusing to overwrite"));
        assert_eq!(std::fs::read(&f).unwrap(), b"a,b\n");
        let run = d.join("results/r1");
        std::fs::create_dir_all(&run).unwrap();
        assert_eq!(find_input(&run, "case-1.csv"), None, "inputs/ is looked for in the ancestors, not beside them");
        std::fs::create_dir_all(d.join("results/inputs")).unwrap();
        std::fs::write(d.join("results/inputs/case-1.csv"), "x").unwrap();
        assert_eq!(find_input(&run, "case-1.csv"), Some(d.join("results/inputs/case-1.csv")));
        std::fs::write(run.join("case-1.csv"), "y").unwrap();
        assert_eq!(find_input(&run, "case-1.csv"), Some(run.join("case-1.csv")), "an imported run's own copy comes first");
        let _ = std::fs::remove_dir_all(&d);
    }
    #[test]
    fn the_index_is_reused_and_never_stale() {
        let d = std::env::temp_dir().join(format!("adcs-index-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        let put = |name: &str, scen: &str| {
            std::fs::create_dir_all(d.join(name)).unwrap();
            std::fs::write(d.join(name).join("manifest.json"), json!({"schema": "adcs-rec/1", "scenario": scen, "metrics": [{"pass": 1}]}).to_string()).unwrap();
        };
        put("a", "first");
        assert_eq!(list(&d).unwrap()[0].m["scenario"], "first");
        let ix = std::fs::read_to_string(d.join(INDEX)).unwrap();
        assert!(ix.contains("adcs-index/1") && ix.contains("\"a\""));
        // a planted index entry whose stamp matches is used as it is: the manifest is not read again
        let st = stamp(&d.join("a/manifest.json")).unwrap();
        std::fs::write(d.join(INDEX), json!({"schema": "adcs-index/1", "runs": {"a": {"stamp": st, "m": {"schema": "adcs-rec/1", "scenario": "from the index"}}}}).to_string()).unwrap();
        assert_eq!(list(&d).unwrap()[0].m["scenario"], "from the index");
        // a changed manifest, a new run and a removed run are all seen
        std::thread::sleep(std::time::Duration::from_millis(20));
        put("a", "second longer");
        put("b", "new");
        let got: Vec<String> = list(&d).unwrap().iter().map(|f| f.m["scenario"].as_str().unwrap().to_string()).collect();
        assert_eq!(got, ["second longer", "new"]);
        std::fs::remove_dir_all(d.join("b")).unwrap();
        assert_eq!(list(&d).unwrap().len(), 1);
        assert!(!std::fs::read_to_string(d.join(INDEX)).unwrap().contains("\"b\""), "a removed run leaves the index");
        let _ = std::fs::remove_dir_all(&d);
    }
    #[test]
    fn the_fingerprint_is_fnv1a() { assert_eq!(hex(fnv(b"a")), "af63dc4c8601ec8c"); }
}
