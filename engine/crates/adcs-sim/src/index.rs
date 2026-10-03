//! The results index: one SQLite database (`.adcs-index.sqlite`) in each folder `list` reads.
//!
//! The run folders stay the record: every manifest is a file a person can open, and the index
//! only remembers what was read from each one, keyed by the manifest's size and time, so a
//! listing reads only runs that are new or changed and the index can never be stale, only
//! slower. Being SQLite it also answers questions no file walk does quickly: every failed
//! requirement of one scenario, the APE of every run of one case across engine versions, the
//! runs older than the engine (`adcs results query --sql`, read only).
//!
//! Tables (schema `adcs-index/2`):
//! - `runs`: one row per run folder (relative to the listed folder): the manifest's stamp,
//!   scenario, case, product, flight software, when, result id, engine version and source,
//!   the inputs it flew (files, fingerprints, seed), passed and failed requirement counts,
//!   pinned, thinned, and the summary the listing shows (JSON).
//! - `metrics`: one row per metric of every run: id, kind, unit, value, requirement, verdict.
//!
//! A damaged or foreign file is rebuilt (it is a cache); a folder that cannot be written gets
//! an index in memory for that listing. Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use crate::error::Error;
use rusqlite::{params, Connection, OpenFlags, OptionalExtension};
use serde_json::Value;
use std::path::Path;

pub const FILE: &str = ".adcs-index.sqlite";
const SCHEMA: &str = "adcs-index/2";

const DDL: &str = "
CREATE TABLE IF NOT EXISTS meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS runs (
    folder TEXT PRIMARY KEY, stamp TEXT NOT NULL,
    scenario TEXT, case_id TEXT, product TEXT, fsw TEXT, created_utc TEXT, result_id TEXT,
    engine_version TEXT, engine_source TEXT, product_fingerprint TEXT,
    case_file TEXT, case_fingerprint TEXT, scenario_file TEXT, scenario_file_fingerprint TEXT, seed INTEGER,
    n_pass INTEGER NOT NULL, n_fail INTEGER NOT NULL, pinned INTEGER NOT NULL, thinned INTEGER NOT NULL,
    summary TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS metrics (
    folder TEXT NOT NULL REFERENCES runs(folder) ON DELETE CASCADE,
    id TEXT NOT NULL, kind TEXT, unit TEXT, value REAL, req REAL, pass INTEGER,
    PRIMARY KEY (folder, id));
CREATE INDEX IF NOT EXISTS runs_scenario ON runs(scenario);
CREATE INDEX IF NOT EXISTS metrics_id ON metrics(id);
";

/// One run as the index keeps it.
pub struct Row { pub folder: String, pub stamp: String, pub summary: Value, pub pinned: bool, pub thinned: bool }

pub struct Index { conn: Connection }

fn sql_err(what: &str, e: rusqlite::Error) -> Error { Error::run(format!("results index {what}: {e}")) }

impl Index {
    /// The index of `root`: opened, created, or rebuilt when it is damaged or of another schema.
    pub fn open(root: &Path) -> Index {
        let f = root.join(FILE);
        for _ in 0..2 {
            if let Ok(c) = Connection::open(&f) {
                if let Some(ix) = Self::ready(c) { return ix; }
            }
            // a damaged file, or one of another schema: it is only a cache, so start it again
            for x in ["", "-journal", "-wal", "-shm"] { let _ = std::fs::remove_file(root.join(format!("{FILE}{x}"))); }
        }
        // the folder cannot be written: this listing keeps its index in memory
        Self::ready(Connection::open_in_memory().expect("an in-memory SQLite database")).expect("an in-memory index")
    }

    fn ready(c: Connection) -> Option<Index> {
        c.busy_timeout(std::time::Duration::from_secs(10)).ok()?;
        c.execute_batch("PRAGMA foreign_keys = ON;").ok()?;
        c.execute_batch(DDL).ok()?;
        let have: Option<String> = c.query_row("SELECT value FROM meta WHERE key = 'schema'", [], |r| r.get(0)).optional().ok()?;
        match have.as_deref() {
            Some(SCHEMA) => {}
            None => { c.execute("INSERT INTO meta (key, value) VALUES ('schema', ?1)", [SCHEMA]).ok()?; }
            Some(_) => return None,
        }
        Some(Index { conn: c })
    }

    /// Every run the index holds: folder -> (stamp, summary).
    pub fn known(&self) -> std::collections::HashMap<String, (String, Value)> {
        let mut out = std::collections::HashMap::new();
        if let Ok(mut st) = self.conn.prepare("SELECT folder, stamp, summary FROM runs") {
            if let Ok(rows) = st.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?))) {
                for (f, s, j) in rows.flatten() {
                    if let Ok(v) = serde_json::from_str(&j) { out.insert(f, (s, v)); }
                }
            }
        }
        out
    }

    /// Make the index hold exactly `rows`, in one transaction; rows whose stamp and kept state
    /// are unchanged are not rewritten.
    pub fn sync(&mut self, rows: &[Row]) -> Result<(), Error> {
        let tx = self.conn.transaction().map_err(|e| sql_err("begin", e))?;
        {
            let mut have = std::collections::HashMap::new();
            {
                let mut st = tx.prepare("SELECT folder, stamp, pinned, thinned FROM runs").map_err(|e| sql_err("read", e))?;
                let it = st.query_map([], |r| Ok((r.get::<_, String>(0)?, (r.get::<_, String>(1)?, r.get::<_, bool>(2)?, r.get::<_, bool>(3)?))))
                    .map_err(|e| sql_err("read", e))?;
                for x in it.flatten() { have.insert(x.0, x.1); }
            }
            let now: std::collections::HashSet<&str> = rows.iter().map(|r| r.folder.as_str()).collect();
            for gone in have.keys().filter(|k| !now.contains(k.as_str())) {
                tx.execute("DELETE FROM runs WHERE folder = ?1", [gone]).map_err(|e| sql_err("delete", e))?;
            }
            for r in rows {
                if have.get(&r.folder) == Some(&(r.stamp.clone(), r.pinned, r.thinned)) { continue; }
                let m = &r.summary;
                let s = |v: &Value| v.as_str().map(String::from);
                let i = &m["inputs"];
                let ms = m["metrics"].as_array().cloned().unwrap_or_default();
                let n_pass = ms.iter().filter(|x| x["pass"].as_i64() == Some(1)).count() as i64;
                let n_fail = ms.iter().filter(|x| x["pass"].as_i64() == Some(0)).count() as i64;
                tx.execute("DELETE FROM runs WHERE folder = ?1", [&r.folder]).map_err(|e| sql_err("replace", e))?;
                tx.execute("INSERT INTO runs VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21)",
                    params![r.folder, r.stamp, s(&m["scenario"]), s(&m["case"]), s(&m["product"]), s(&m["fsw"]["impl"]), s(&m["created_utc"]),
                            s(&m["result_id"]), s(&m["engine_version"]), s(&m["engine_source"]), s(&m["product_fingerprint"]),
                            s(&i["case_file"]), s(&i["case_fingerprint"]), s(&i["scenario_file"]), s(&i["scenario_file_fingerprint"]), i["seed"].as_i64(),
                            n_pass, n_fail, r.pinned, r.thinned, m.to_string()])
                    .map_err(|e| sql_err("insert", e))?;
                for x in &ms {
                    let Some(id) = x["id"].as_str() else { continue };
                    tx.execute("INSERT OR REPLACE INTO metrics VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                        params![r.folder, id, s(&x["kind"]), s(&x["unit"]), x["value"].as_f64(), x["req"].as_f64(), x["pass"].as_i64()])
                        .map_err(|e| sql_err("insert", e))?;
                }
            }
        }
        tx.commit().map_err(|e| sql_err("commit", e))
    }
}

/// Run one read-only SQL statement on the index of `root` (after `store::list` brought it up to
/// date): the column names and every row, each value as text (NULL as empty).
pub fn query(root: &Path, sql: &str) -> Result<(Vec<String>, Vec<Vec<String>>), Error> {
    let f = root.join(FILE);
    let c = Connection::open_with_flags(&f, OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX)
        .map_err(|e| sql_err(&format!("open {}", f.display()), e))?;
    c.busy_timeout(std::time::Duration::from_secs(10)).map_err(|e| sql_err("open", e))?;
    let mut st = c.prepare(sql).map_err(|e| Error::refused(format!("the query does not read: {e}")))?;
    if !st.readonly() { return Err(Error::refused("a query only reads: the index is kept by `adcs results list`, never written by hand")); }
    let cols: Vec<String> = st.column_names().iter().map(|s| s.to_string()).collect();
    let n = cols.len();
    let rows = st.query_map([], |r| {
        (0..n).map(|k| Ok(match r.get_ref(k)? {
            rusqlite::types::ValueRef::Null => String::new(),
            rusqlite::types::ValueRef::Integer(i) => i.to_string(),
            rusqlite::types::ValueRef::Real(x) => format!("{x}"),
            rusqlite::types::ValueRef::Text(t) => String::from_utf8_lossy(t).into_owned(),
            rusqlite::types::ValueRef::Blob(b) => format!("<{} bytes>", b.len()),
        })).collect::<Result<Vec<String>, rusqlite::Error>>()
    }).map_err(|e| Error::refused(format!("the query failed: {e}")))?;
    let mut out = vec![];
    for r in rows { out.push(r.map_err(|e| Error::refused(format!("the query failed: {e}")))?); }
    Ok((cols, out))
}
