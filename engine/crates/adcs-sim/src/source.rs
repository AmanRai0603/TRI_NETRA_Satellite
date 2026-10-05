//! Where the engine reads its inputs from: the data folder's files, or the design database.
//!
//! With a design database in use (`adcs --design FILE`, the app's own, or `$TRINETRA_DESIGN`),
//! every case (`cases/<id>.csv`) and every input under `data/` (scenarios, products, parts,
//! algorithms, the catalogue, the classes) is read from it, and never from the folder: the
//! cases from its `design_case` rows, line by line as the case states them, the rest from its
//! `engine_input` files, each held to its fingerprint. A path under `cases/` or `data/` that
//! the database does not hold is refused by name, so a run cannot quietly mix the two. The
//! ephemeris and gravity data the orbit reads, and the store, stay files.
//!
//! The bytes are the case's and the files' own, so a run flown from the database has the same
//! fingerprints and the same result id as one flown from the files (tests/design_source.rs).
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use crate::error::Error;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

/// The header every case file starts with (adcs-case/1).
pub const CASE_HEADER: &str = "section,key,label,unit,value,lo,hi,level,note";

/// The toolbox this engine offers a design: the functions its relations and inputs may call.
/// A design names the toolbox it was built for (its meta `toolbox`) and is refused by any other.
pub const TOOLBOX: &str = "trinetra-toolbox/1";

/// This program's version: a design names the oldest application that can run it (its meta
/// `needs_application`), and a newer one is refused by name.
pub const APPLICATION: &str = env!("CARGO_PKG_VERSION");

/// A design database opened for the engine: its inputs, by their path in the data folder.
#[derive(Debug)]
pub struct Design {
    pub file: PathBuf,
    /// the database's own fingerprint (its meta `inputs_fingerprint`): one hash over every input it holds
    pub fingerprint: String,
    /// its version, when it has one (its meta `design_version`, written when a design is released)
    pub version: Option<String>,
    /// the toolbox it was built for
    pub toolbox: String,
    files: BTreeMap<String, Vec<u8>>,
}

/// `a.b.c` as numbers, for comparing versions; None when it is not one.
fn semver(v: &str) -> Option<(u64, u64, u64)> {
    let mut it = v.trim().trim_start_matches('v').split('.').map(|x| x.parse::<u64>().ok());
    let t = (it.next()??, it.next()??, it.next()??);
    it.next().is_none().then_some(t)
}

/// Why this program cannot run a design with this meta (None: it can). A design that names no
/// toolbox or no application version was built before designs said what they need, and is
/// refused too: rebuilding it writes both.
pub fn cannot_run(meta: &BTreeMap<String, String>) -> Option<String> {
    let rebuild = "rebuild it (python3 tools/seed_design.py, or tools/group.py merge)";
    match meta.get("toolbox").map(String::as_str) {
        None | Some("") => return Some(format!("it names no toolbox: built before designs said what they need; {rebuild}")),
        Some(t) if t != TOOLBOX => return Some(format!("it was built for the toolbox {t}; this program offers {TOOLBOX}")),
        _ => {}
    }
    let need = meta.get("needs_application").map(String::as_str).unwrap_or("");
    match (semver(need), semver(APPLICATION)) {
        (None, _) => Some(format!("it names no application version it needs ({need:?}); {rebuild}")),
        (Some(n), Some(me)) if n > me => Some(format!("it needs TRI-NETRA {need} or later; this is {APPLICATION}")),
        _ => None,
    }
}

/// FNV-1a 64 over bytes, as hex: the fingerprint the store gives every input.
pub fn fnv_hex(b: &[u8]) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    for x in b { h ^= *x as u64; h = h.wrapping_mul(0x100000001b3); }
    format!("{h:016x}")
}

impl Design {
    /// Open a design database read-only and take every input it holds; refused, by name, when it
    /// is not a design database of format version 2 or later, or an input is not what its
    /// fingerprint says.
    pub fn open(path: &Path) -> Result<Design, Error> {
        use rusqlite::{Connection, OpenFlags};
        let bad = |m: String| Error::refused(format!("design database {}: {m}", path.display()));
        if path.is_dir() {
            let inside = [path.join("design.tndb"), path.join("Design").join("design.tndb")].into_iter().find(|p| p.is_file());
            return Err(bad(match inside {
                Some(f) => format!("a folder, not a design database; the design database in it is {}", f.display()),
                None => "a folder, not a design database, and it holds no design.tndb".into(),
            }));
        }
        if !path.is_file() { return Err(bad("no such file".into())); }
        let c = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX).map_err(|e| bad(e.to_string()))?;
        let meta: BTreeMap<String, String> = c.prepare(r#"SELECT "key", "value" FROM meta"#)
            .and_then(|mut s| s.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?.unwrap_or_default())))?.collect())
            .map_err(|e| bad(if e.to_string().contains("not a database") { "not a database file (damaged, or not a design file)".into() }
                             else { format!("no meta table: not a design file ({e})") }))?;
        if meta.get("format").map(String::as_str) != Some("design") {
            return Err(bad(format!("a {} file, not a design database", meta.get("format").map(String::as_str).unwrap_or("?"))));
        }
        let v: u32 = meta.get("format_version").and_then(|v| v.parse().ok()).unwrap_or(0);
        if v < 2 { return Err(bad(format!("format version {v} holds no engine inputs: rebuild it (python3 tools/seed_design.py, or tools/group.py merge)"))); }
        if let Some(why) = cannot_run(&meta) { return Err(bad(format!("this program cannot run it: {why}"))); }
        let mut files = BTreeMap::new();
        let q = |sql: &str| -> Result<Vec<(String, String, Vec<u8>)>, Error> {
            c.prepare(sql).and_then(|mut s| s.query_map([], |r| Ok((r.get(0)?, r.get::<_, Option<String>>(1)?.unwrap_or_default(), r.get::<_, Option<Vec<u8>>>(2)?.unwrap_or_default())))?.collect())
                .map_err(|e| bad(e.to_string()))
        };
        for (p, fp, body) in q(r#"SELECT "path", "fingerprint", "body" FROM engine_input ORDER BY "path""#)? {
            if fnv_hex(&body) != fp { return Err(bad(format!("{p} is not what its fingerprint says (changed outside the tools?)"))); }
            files.insert(p, body);
        }
        // each case: its lines in order, under the one header
        let mut cases: BTreeMap<String, String> = BTreeMap::new();
        for (case, line, _) in q(r#"SELECT "case_id", "line", NULL FROM design_case ORDER BY "case_id", "ord""#)? {
            let t = cases.entry(case).or_insert_with(|| format!("{CASE_HEADER}\n"));
            t.push_str(&line);
            t.push('\n');
        }
        for (case, text) in cases { files.insert(format!("cases/{case}.csv"), text.into_bytes()); }
        Ok(Design { file: path.to_path_buf(), fingerprint: meta.get("inputs_fingerprint").cloned().unwrap_or_default(),
                    version: meta.get("design_version").filter(|v| !v.is_empty()).cloned(),
                    toolbox: meta.get("toolbox").cloned().unwrap_or_default(), files })
    }
    /// The case ids it holds.
    pub fn cases(&self) -> Vec<String> {
        self.files.keys().filter_map(|k| k.strip_prefix("cases/").and_then(|x| x.strip_suffix(".csv"))).map(String::from).collect()
    }
    /// The paths it holds under a folder of the data folder (`data/scenarios`), sorted.
    pub fn under(&self, dir: &str) -> Vec<String> {
        let pre = format!("{}/", dir.trim_end_matches('/'));
        self.files.keys().filter(|k| k.starts_with(&pre) && !k[pre.len()..].contains('/')).cloned().collect()
    }
    pub fn get(&self, rel: &str) -> Option<&[u8]> { self.files.get(rel).map(Vec::as_slice) }
}

#[derive(Default)]
struct State { chosen: bool, design: Option<Arc<Design>> }
static STATE: RwLock<State> = RwLock::new(State { chosen: false, design: None });

/// Read from this design database from now on (None: the data folder's files).
pub fn use_design(d: Option<Design>) {
    let mut s = STATE.write().unwrap_or_else(|p| p.into_inner());
    *s = State { chosen: true, design: d.map(Arc::new) };
}

/// The design database in use: the one chosen, else `$TRINETRA_DESIGN` (opened once; a file it
/// names that cannot be opened is refused, never passed over), else none.
pub fn current() -> Result<Option<Arc<Design>>, Error> {
    {
        let s = STATE.read().unwrap_or_else(|p| p.into_inner());
        if s.chosen { return Ok(s.design.clone()); }
    }
    let d = match std::env::var_os("TRINETRA_DESIGN").filter(|v| !v.is_empty()) {
        Some(p) => Some(Design::open(Path::new(&p))?),
        None => None,
    };
    use_design(d);
    Ok(STATE.read().unwrap_or_else(|p| p.into_inner()).design.clone())
}

fn absolute(p: &Path) -> PathBuf {
    if p.is_absolute() { p.to_path_buf() } else { std::env::current_dir().unwrap_or_default().join(p) }
}

/// A path's place in the data folder when it is an input the design database answers for
/// (`cases/...` or `data/...`), else None.
fn key(p: &Path) -> Option<String> {
    let rel = absolute(p).strip_prefix(absolute(&crate::data_root())).ok()?.to_path_buf();
    let k = rel.components().map(|c| c.as_os_str().to_string_lossy().into_owned()).collect::<Vec<_>>().join("/");
    (k.starts_with("cases/") || k.starts_with("data/")).then_some(k)
}

/// An input's bytes: from the design database when one is in use and the path is one it answers
/// for, else from the file.
pub fn read(p: &Path) -> Result<Vec<u8>, Error> {
    if let (Some(d), Some(k)) = (current()?, key(p)) {
        return d.get(&k).map(<[u8]>::to_vec)
            .ok_or_else(|| Error::refused(format!("{k} is not in the design database {} (the engine reads its inputs from it alone)", d.file.display())));
    }
    std::fs::read(p).map_err(|e| Error::io(p, e))
}

pub fn read_to_string(p: &Path) -> Result<String, Error> {
    String::from_utf8(read(p)?).map_err(|_| Error::malformed(format!("{}: not UTF-8 text", p.display())))
}

/// Whether an input exists where the engine would read it.
pub fn is_file(p: &Path) -> bool {
    match (current(), key(p)) {
        (Ok(Some(d)), Some(k)) => d.get(&k).is_some(),
        _ => p.is_file(),
    }
}

/// The inputs in one folder of the data folder (`root/data/scenarios`), sorted by name.
pub fn list(dir: &Path) -> Vec<PathBuf> {
    if let (Ok(Some(d)), Some(k)) = (current(), key(&dir.join("x"))) {
        let k = k.trim_end_matches("/x");
        return d.under(k).iter().filter_map(|x| x.rsplit('/').next()).map(|n| dir.join(n)).collect();
    }
    let mut v: Vec<PathBuf> = std::fs::read_dir(dir).map(|rd| rd.flatten().map(|e| e.path()).filter(|p| p.is_file()).collect()).unwrap_or_default();
    v.sort();
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    fn meta(pairs: &[(&str, &str)]) -> BTreeMap<String, String> { pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect() }
    #[test]
    fn a_design_built_for_another_engine_is_refused() {
        assert_eq!(cannot_run(&meta(&[("toolbox", TOOLBOX), ("needs_application", "1.0.0")])), None);
        assert_eq!(cannot_run(&meta(&[("toolbox", TOOLBOX), ("needs_application", APPLICATION)])), None);
        let other = cannot_run(&meta(&[("toolbox", "trinetra-toolbox/99"), ("needs_application", "1.0.0")])).unwrap();
        assert!(other.contains("trinetra-toolbox/99") && other.contains(TOOLBOX), "{other}");
        let newer = cannot_run(&meta(&[("toolbox", TOOLBOX), ("needs_application", "99.0.0")])).unwrap();
        assert!(newer.contains("needs TRI-NETRA 99.0.0"), "{newer}");
        assert!(cannot_run(&meta(&[("needs_application", "1.0.0")])).unwrap().contains("names no toolbox"));
        assert!(cannot_run(&meta(&[("toolbox", TOOLBOX)])).unwrap().contains("names no application"));
        assert!(cannot_run(&meta(&[("toolbox", TOOLBOX), ("needs_application", "one")])).is_some());
    }
    #[test]
    fn a_folder_or_a_damaged_file_is_refused_by_name() {
        let d = std::env::temp_dir().join(format!("adcs-src-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(d.join("Design")).unwrap();
        let e = Design::open(&d).unwrap_err().message().to_string();
        assert!(e.contains("a folder") && e.contains("holds no design.tndb"), "{e}");
        std::fs::write(d.join("Design/design.tndb"), b"junk").unwrap();
        let e = Design::open(&d).unwrap_err().message().to_string();
        assert!(e.contains("the design database in it is") && e.contains("design.tndb"), "{e}");
        let e = Design::open(&d.join("Design/design.tndb")).unwrap_err().message().to_string();
        assert!(e.contains("not a database file"), "{e}");
        let _ = std::fs::remove_dir_all(&d);
    }
}
