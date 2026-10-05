//! A design file's content as canonical bytes (`trinetra-content/1`), and the hash a signature
//! covers.
//!
//! The encoding is the same in every program that signs or checks (this library, the page, the
//! tools), so it avoids anything a language might print differently: a real is written as the hex
//! of its IEEE-754 bits, never as decimal text.
//!
//! ```text
//! trinetra-content/1
//! T <table>                     one block per signed table, in the schema's order for the kind
//! <cell>|<cell>|...             one line per row, the rows sorted by their bytes
//! cell: n            null
//!       i<decimal>   integer
//!       r<16 hex>    real (its 64 bits, big-endian)
//!       t<hex>       text (its UTF-8 bytes, as hex)
//!       b<hex>       blob
//! ```
//!
//! What a signature covers is the file's own content: every table of its kind except `meta` and the
//! record of work around it (revisions, comments, change requests, signatures).
//!
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use crate::{Cell, DesignFile, FormatError, Schema};
use sha2::{Digest, Sha256};

pub const ENCODING: &str = "trinetra-content/1";

/// The tables a signature never covers: the file's description and the record of work around it.
pub const NOT_SIGNED: [&str; 6] = ["meta", "revision", "comment", "change_request", "signature", "key_signature"];

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// One cell, encoded.
pub fn cell(c: &Cell) -> String {
    match c {
        Cell::Null => "n".into(),
        Cell::Integer(v) => format!("i{v}"),
        Cell::Real(v) => format!("r{:016x}", v.to_bits()),
        Cell::Text(v) => format!("t{}", hex(v.as_bytes())),
        Cell::Blob(v) => format!("b{}", hex(v)),
    }
}

/// The tables of a kind a signature covers, in the schema's order.
pub fn signed_tables(kind: &str, s: &Schema) -> Vec<String> {
    s.formats[kind].tables.iter().filter(|t| !NOT_SIGNED.contains(&t.as_str())).cloned().collect()
}

/// The canonical bytes of the tables named, from an open file.
pub fn encode(f: &DesignFile, tables: &[String], s: &Schema) -> Result<Vec<u8>, FormatError> {
    let mut out = format!("{ENCODING}\n");
    for t in tables {
        let mut lines: Vec<String> = f.rows(t, s)?.iter().map(|r| r.iter().map(cell).collect::<Vec<_>>().join("|")).collect();
        lines.sort();
        out.push_str(&format!("T {t}\n"));
        for l in lines {
            out.push_str(&l);
            out.push('\n');
        }
    }
    Ok(out.into_bytes())
}

/// SHA-256 of the canonical bytes of what a signature covers, as hex.
pub fn content_hash(f: &DesignFile, s: &Schema) -> Result<String, FormatError> {
    Ok(hex(&Sha256::digest(encode(f, &signed_tables(&f.kind, s), s)?)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_cell_is_written_the_same_whatever_prints_it() {
        assert_eq!(cell(&Cell::Null), "n");
        assert_eq!(cell(&Cell::Integer(-3)), "i-3");
        assert_eq!(cell(&Cell::Real(0.1)), "r3fb999999999999a");
        assert_eq!(cell(&Cell::Real(-0.0)), "r8000000000000000");
        assert_eq!(cell(&Cell::Text("µ|\n".into())), "tc2b57c0a");
        assert_eq!(cell(&Cell::Blob(vec![0, 255])), "b00ff");
    }
}
