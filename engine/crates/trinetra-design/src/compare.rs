//! Comparing two design files of one kind, table by table: the rows only one of them holds.
//! Any two revisions, releases or designs can be compared this way; the application shows the
//! result node by node (a design's rows name their node).
//!
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use crate::content::cell;
use crate::{DesignFile, FormatError, Schema};
use std::collections::BTreeSet;

/// One table's difference: the rows (encoded, as `content` writes them) only in `a`, only in `b`.
#[derive(Debug, Default, PartialEq)]
pub struct TableDiff {
    pub table: String,
    pub only_a: Vec<String>,
    pub only_b: Vec<String>,
}

/// The tables whose rows differ between two files of one kind (empty: the same content).
/// `meta` and the record of work are compared too unless `content_only`.
pub fn compare(a: &DesignFile, b: &DesignFile, s: &Schema, content_only: bool) -> Result<Vec<TableDiff>, FormatError> {
    if a.kind != b.kind {
        return Err(FormatError(format!("a {} file and a {} file: only files of one kind compare", a.kind, b.kind)));
    }
    let tables: Vec<String> = if content_only { crate::content::signed_tables(&a.kind, s) }
        else { s.formats[&a.kind].tables.iter().filter(|t| *t != "meta").cloned().collect() };
    let mut out = Vec::new();
    for t in tables {
        let enc = |f: &DesignFile| -> Result<BTreeSet<String>, FormatError> {
            Ok(f.rows(&t, s)?.iter().map(|r| r.iter().map(cell).collect::<Vec<_>>().join("|")).collect())
        };
        let (x, y) = (enc(a)?, enc(b)?);
        let d = TableDiff { table: t.clone(), only_a: x.difference(&y).cloned().collect(), only_b: y.difference(&x).cloned().collect() };
        if !d.only_a.is_empty() || !d.only_b.is_empty() {
            out.push(d);
        }
    }
    Ok(out)
}
