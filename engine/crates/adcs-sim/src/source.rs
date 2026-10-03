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

/// A design database opened for the engine: its inputs, by their path in the data folder.
#[derive(Debug)]
pub struct Design {
    pub file: PathBuf,
    /// the database's own fingerprint (its meta `inputs_fingerprint`): one hash over every input it holds
    pub fingerprint: String,
    files: BTreeMap<String, Vec<u8>>,
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
        if !path.is_file() { return Err(bad("no such file".into())); }
        let c = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX).map_err(|e| bad(e.to_string()))?;
        let meta: BTreeMap<String, String> = c.prepare(r#"SELECT "key", "value" FROM meta"#)
            .and_then(|mut s| s.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?.unwrap_or_default())))?.collect())
            .map_err(|e| bad(format!("no meta table: not a design file ({e})")))?;
        if meta.get("format").map(String::as_str) != Some("design") {
            return Err(bad(format!("a {} file, not a design database", meta.get("format").map(String::as_str).unwrap_or("?"))));
        }
        let v: u32 = meta.get("format_version").and_then(|v| v.parse().ok()).unwrap_or(0);
        if v < 2 { return Err(bad(format!("format version {v} holds no engine inputs: rebuild it (python3 tools/seed_design.py, or tools/group.py merge)"))); }
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
        Ok(Design { file: path.to_path_buf(), fingerprint: meta.get("inputs_fingerprint").cloned().unwrap_or_default(), files })
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
