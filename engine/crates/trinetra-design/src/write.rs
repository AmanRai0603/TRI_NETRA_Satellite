//! Making and upgrading design files, as tools/tndb.py does: a new file is built whole under a
//! temporary name and renamed into place; an older file is upgraded in place after a copy of it is
//! kept beside it (`<file>.v<N>.bak`); a newer one is refused by name.
//!
//! Every upgrade step so far only adds the tables a format gained (2.0.0's block model, the
//! engine's inputs), so an upgrade never changes or moves anything a file holds.
//!
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use crate::{meta, open, Cell, DesignFile, FormatError, Schema};
use rusqlite::{params_from_iter, types::Value, Connection};
use std::path::{Path, PathBuf};

/// The program a file names as its writer when this library writes it.
pub const PROGRAM: &str = concat!("trinetra-design/", env!("CARGO_PKG_VERSION"));

fn sql_err(p: &Path, e: rusqlite::Error) -> FormatError {
    FormatError(format!("{}: {e}", p.display()))
}

/// Make a new file of this kind, whole: its tables, its meta, and what `fill` puts in, in one
/// transaction under a temporary name, then renamed into place. Refuses to overwrite.
pub fn create(path: &Path, kind: &str, id: &str, s: &Schema, fill: impl FnOnce(&Connection) -> Result<(), FormatError>)
    -> Result<PathBuf, FormatError> {
    let f = s.formats.get(kind).ok_or_else(|| FormatError(format!("no format {kind:?}; the formats are {}",
        s.formats.keys().cloned().collect::<Vec<_>>().join(", "))))?;
    if path.exists() {
        return Err(FormatError(format!("{} exists; a design file is never overwritten by create", path.display())));
    }
    let tmp = path.with_file_name(format!("{}.tmp", path.file_name().and_then(|n| n.to_str()).unwrap_or("file")));
    let _ = std::fs::remove_file(&tmp);
    let build = || -> Result<(), FormatError> {
        let c = Connection::open(&tmp).map_err(|e| sql_err(&tmp, e))?;
        c.execute_batch("PRAGMA journal_mode = OFF; PRAGMA synchronous = OFF; BEGIN").map_err(|e| sql_err(&tmp, e))?;
        for st in s.ddl(kind)? {
            c.execute(&st, []).map_err(|e| sql_err(&tmp, e))?;
        }
        let version = f.version.to_string();
        for (k, v) in [("format", kind), ("format_version", version.as_str()), ("id", id), ("written_by", PROGRAM)] {
            c.execute("INSERT INTO meta VALUES (?1, ?2)", [k, v]).map_err(|e| sql_err(&tmp, e))?;
        }
        fill(&c)?;
        c.execute_batch("COMMIT").map_err(|e| sql_err(&tmp, e))?;
        Ok(())
    };
    if let Err(e) = build() {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }
    std::fs::File::open(&tmp).and_then(|f| f.sync_all()).map_err(|e| FormatError(format!("{}: {e}", tmp.display())))?;
    std::fs::rename(&tmp, path).map_err(|e| FormatError(format!("{}: {e}", path.display())))?;
    Ok(path.to_path_buf())
}

/// Insert rows into one table, each in the schema's column order.
pub fn insert(c: &Connection, table: &str, rows: &[Vec<Cell>], s: &Schema) -> Result<(), FormatError> {
    let cols = s.columns(table);
    let sql = format!("INSERT INTO \"{table}\" ({}) VALUES ({})",
        cols.iter().map(|x| format!("\"{}\"", x.0)).collect::<Vec<_>>().join(", "),
        (1..=cols.len()).map(|i| format!("?{i}")).collect::<Vec<_>>().join(", "));
    let mut st = c.prepare(&sql).map_err(|e| FormatError(format!("{table}: {e}")))?;
    for r in rows {
        if r.len() != cols.len() {
            return Err(FormatError(format!("{table}: a row of {} cells, the table has {} columns", r.len(), cols.len())));
        }
        let vals = r.iter().map(|x| match x {
            Cell::Null => Value::Null,
            Cell::Integer(v) => Value::Integer(*v),
            Cell::Real(v) => Value::Real(*v),
            Cell::Text(v) => Value::Text(v.clone()),
            Cell::Blob(v) => Value::Blob(v.clone()),
        });
        st.execute(params_from_iter(vals)).map_err(|e| FormatError(format!("{table}: {e}")))?;
    }
    Ok(())
}

/// What an upgrade did: the version it came from (None: it was current), and the copy kept.
#[derive(Debug, PartialEq)]
pub struct Upgraded {
    pub from: Option<u32>,
    pub backup: Option<PathBuf>,
}

/// Bring a file to this program's format version in place: a copy of the original is kept beside it,
/// then each missing table of its format is added, empty, and its version set. A file already
/// current is left untouched; a newer one, or another kind than asked, is refused by name.
pub fn upgrade(path: &Path, kind: Option<&str>, s: &Schema) -> Result<Upgraded, FormatError> {
    let p = path.display();
    if !path.is_file() {
        return Err(FormatError(format!("{p}: no such file")));
    }
    let c = Connection::open(path).map_err(|e| sql_err(path, e))?;
    let m = meta(&c)?;
    let got = m.get("format").cloned().unwrap_or_default();
    let Some(f) = s.formats.get(&got) else {
        return Err(FormatError(format!("{p}: format {got:?} is not a design-file format")));
    };
    if let Some(k) = kind {
        if k != got {
            return Err(FormatError(format!("{p}: a {got} file, not a {k} file")));
        }
    }
    let have: u32 = m.get("format_version").and_then(|v| v.parse().ok()).unwrap_or(0);
    if have > f.version {
        return Err(FormatError(format!("{p}: {got} format version {have} is newer than this program's {}; open it with a newer TRI-NETRA", f.version)));
    }
    if have == f.version {
        return Ok(Upgraded { from: None, backup: None });
    }
    drop(c);
    let backup = path.with_file_name(format!("{}.v{have}.bak", path.file_name().and_then(|n| n.to_str()).unwrap_or("file")));
    std::fs::copy(path, &backup).map_err(|e| FormatError(format!("{}: {e}", backup.display())))?;
    let c = Connection::open(path).map_err(|e| sql_err(path, e))?;
    let tables: Vec<String> = c.prepare("SELECT name FROM sqlite_master WHERE type = 'table'")
        .and_then(|mut st| st.query_map([], |r| r.get(0))?.collect())
        .map_err(|e| sql_err(path, e))?;
    let tx = c.unchecked_transaction().map_err(|e| sql_err(path, e))?;
    for (t, st) in f.tables.iter().zip(s.ddl(&got)?) {
        if !tables.contains(t) {
            tx.execute(&st, []).map_err(|e| sql_err(path, e))?;
        }
    }
    tx.execute("UPDATE meta SET \"value\" = ?1 WHERE \"key\" = 'format_version'", [f.version.to_string()]).map_err(|e| sql_err(path, e))?;
    tx.commit().map_err(|e| sql_err(path, e))?;
    Ok(Upgraded { from: Some(have), backup: Some(backup) })
}

/// Open a file for reading, upgrading it first when it is older (the copy kept beside it).
pub fn open_current(path: &Path, kind: Option<&str>, s: &Schema) -> Result<(DesignFile, Upgraded), FormatError> {
    let u = upgrade(path, kind, s)?;
    Ok((open(path, kind, s)?, u))
}
