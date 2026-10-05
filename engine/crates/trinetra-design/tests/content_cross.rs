//! The page (design/js/tncontent.js, with sql.js) and this library compute the same content hash of
//! the same file, whatever its cells hold: reals that print alike in no two languages, a negative
//! zero, text beyond ASCII, a blob, a null. Skipped when Node is not installed.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use std::process::Command;
use trinetra_design::{content, open, write, Cell, Schema};

#[test]
fn the_page_and_the_application_hash_a_file_alike() {
    let s = Schema::embedded().unwrap();
    let p = std::env::temp_dir().join(format!("content-cross-{}.node.tndb", std::process::id()));
    let _ = std::fs::remove_file(&p);
    write::create(&p, "node", "gd_1", &s, |c| {
        write::insert(c, "output", &[vec![Cell::Text("tau".into()), Cell::Text("N m".into()), Cell::Real(0.1), Cell::Real(-0.0),
            Cell::Text("µ-limit ≥ 0".into()), Cell::Null]], &s)?;
        write::insert(c, "attachment", &[vec![Cell::Text("a.png".into()), Cell::Text("image/png".into()), Cell::Integer(3), Cell::Blob(vec![0, 7, 255])]], &s)?;
        write::insert(c, "port", &[vec![Cell::Text("tau".into()), Cell::Text("out".into()), Cell::Text("number".into()), Cell::Text("open".into()),
            Cell::Text("estimated".into()), Cell::Text("<=".into()), Cell::Null, Cell::Null, Cell::Null]], &s)?;
        write::insert(c, "fixture", &[vec![Cell::Text("c1".into()), Cell::Text("{}".into()), Cell::Text("1e-300".into()), Cell::Real(1e-300),
            Cell::Text("Wertz 1978".into()), Cell::Integer(1)]], &s)
    }).unwrap();
    let mine = content::content_hash(&open(&p, Some("node"), &s).unwrap(), &s).unwrap();
    let cli = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../design/js/tncontent_cli.mjs");
    let Ok(o) = Command::new("node").arg(cli).arg(&p).output() else {
        eprintln!("node is not installed: skipped");
        return;
    };
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert_eq!(String::from_utf8_lossy(&o.stdout).trim(), mine);
    let _ = std::fs::remove_file(&p);
}
