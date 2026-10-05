//! The node app's live checks in the library (src/node_rules.rs) against the node app's own
//! (design/js/node_model.js, run under Node by tests/js_node_check.mjs): every node file of the
//! seeded and carried design (tools/seed_design.py, tools/carry_over.py), each checked with the
//! design's nodes around it, and a few hundred broken copies (fields blanked or garbled, units and
//! quantities the software does not know, pseudocode the language refuses, test vectors without
//! their source, unknown sources, beliefs without their test, results that are not numbers, ...)
//! so that every rule fires in both. The same problems, codes, levels, steps and words, in the
//! same order; where the node app stops (a TypeError), the library says so too.
//! Skipped (with a line saying so) where python3 or Node is not installed.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.

use rusqlite::Connection;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use trinetra_design::node_rules::{self, Context};

const RULES: &[&str] = &[
    "N02", "X01", "X02", "X03", "B01", "B02", "Q01", "E20", "O01", "O02", "O03", "O04", "C01", "C02", "C03", "C04", "R01", "R04", "R06", "R07", "P01",
    "P02", "V01", "V02", "V03", "S01", "T01", "T02", "T03", "T04", "Y01", "D01", "D03", "D06", "D07",
];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..").canonicalize().unwrap()
}

/// a seeded, carried design in a fresh folder, or None without python3
fn design(dir: &Path) -> Option<PathBuf> {
    let d = dir.join("Design");
    let py = "import sys, pathlib; sys.path.insert(0, 'tools'); import seed_design, carry_over; d = pathlib.Path(sys.argv[1]); seed_design.seed(d, sync=False); carry_over.carry(d)";
    let out = Command::new("python3").arg("-c").arg(py).arg(&d).current_dir(root()).output().ok()?;
    assert!(out.status.success(), "seeding the design failed: {}", String::from_utf8_lossy(&out.stderr));
    Some(d)
}

/// the library's answer for a file, as the harness writes the app's
fn rust(file: &Path, ctx: &Context) -> Result<Vec<[String; 4]>, ()> {
    let conn = Connection::open(file).unwrap();
    let doc = node_rules::read_doc(&conn).map_err(|_| ())?;
    let p = node_rules::check(&doc, ctx).map_err(|_| ())?;
    Ok(p.into_iter().map(|p| [p.code.to_string(), p.level.to_string(), p.step.to_string(), p.text]).collect())
}

/// the node app's answers, or None without Node
fn javascript(folder: &Path, files: &[PathBuf]) -> Option<Vec<Result<Vec<[String; 4]>, ()>>> {
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/js_node_check.mjs");
    let mut child = Command::new("node").arg(script).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().ok()?;
    let req = json!({ "folder": folder, "files": files });
    child.stdin.take().unwrap().write_all(req.to_string().as_bytes()).unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success(), "js_node_check.mjs failed: {}", String::from_utf8_lossy(&out.stderr));
    let v: Vec<Value> = serde_json::from_slice(&out.stdout).unwrap();
    Some(
        v.into_iter()
            .map(|r| match r.get("problems") {
                Some(p) => Ok(p
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|x| {
                        let a = x.as_array().unwrap();
                        [0, 1, 2, 3].map(|i| a[i].as_str().unwrap().to_string())
                    })
                    .collect()),
                None => Err(()),
            })
            .collect(),
    )
}

fn show(r: &Result<Vec<[String; 4]>, ()>) -> String {
    match r {
        Ok(p) => p.iter().map(|x| format!("    {} {} {}: {}", x[0], x[1], x[2], x[3])).collect::<Vec<_>>().join("\n"),
        Err(()) => "    (the checks stop: TypeError)".into(),
    }
}

/// a broken copy: SQL run on a copy of a node file. `{id}` is the node's id, `{other_layer}` a node
/// of another layer that is no interface, `{publisher}` a node whose spec gives its quantity.
struct Recipe {
    name: &'static str,
    kinds: &'static [&'static str],
    sql: &'static [&'static str],
}

const COMPUTED: &[&str] = &["computed"];
const VALUED: &[&str] = &["declared", "computed", "kpi", "evidence"];
const FIXED: &[&str] = &["closure", "interface", "target"];
const ANY: &[&str] = &["declared", "computed", "kpi", "evidence", "closure", "interface", "target"];

/// set a field: `field = value` as SQL
macro_rules! set {
    ($sec:literal, $f:literal, $v:literal) => {
        concat!("DELETE FROM content WHERE section = '", $sec, "' AND field = '", $f, "'; INSERT INTO content (section, field, value, origin) VALUES ('", $sec, "', '", $f, "', ", $v, ", 'typed by test')")
    };
}
macro_rules! del {
    ($sec:literal, $f:literal) => {
        concat!("DELETE FROM content WHERE section = '", $sec, "' AND field = '", $f, "'")
    };
}

const RECIPES: &[Recipe] = &[
    Recipe { name: "unnamed row, no kind", kinds: ANY, sql: &["UPDATE node SET id = 'l3_new_row_9', kind = 'leaf'", del!("identity", "form_kind")] },
    Recipe { name: "unnamed row chosen computed", kinds: VALUED, sql: &["UPDATE node SET id = 'l3_new_row_8', kind = 'internal'", set!("identity", "form_kind", "'computed'")] },
    Recipe { name: "unnamed row chosen declared", kinds: VALUED, sql: &["UPDATE node SET id = 'l3_new_row_7', kind = ''", set!("identity", "form_kind", "'declared'")] },
    Recipe { name: "X01 analogy", kinds: ANY, sql: &[set!("explain", "analogy", "'a sail'"), del!("explain", "analogy_breaks")] },
    Recipe { name: "X02 wrong idea", kinds: ANY, sql: &[set!("explain", "wrong_idea", "'bigger is better'"), set!("explain", "wrong_because", "'  '")] },
    Recipe { name: "X03 simply", kinds: COMPUTED, sql: &[set!("explain", "simply", "''"), set!("explain", "one_line", "'one'")] },
    Recipe { name: "B01 B02 feedback", kinds: FIXED, sql: &[set!("feedback", "expected", "'pass'"), set!("other", "subject", "'add a closure'")] },
    Recipe { name: "B02 description only", kinds: FIXED, sql: &[set!("other", "description", "'why'")] },
    Recipe { name: "Q01 E20 blank", kinds: VALUED, sql: &[set!("identity", "question", "' '"), set!("explain", "where_it_breaks", "NULL"), set!("spec", "question", "''")] },
    Recipe { name: "nbsp is blank, NEL is not", kinds: VALUED, sql: &[set!("identity", "question", "char(160)"), set!("explain", "where_it_breaks", "char(133)")] },
    Recipe { name: "O01 symbol", kinds: VALUED, sql: &[set!("output", "symbol", "'m max'")] },
    Recipe { name: "O01 symbol a number", kinds: VALUED, sql: &[set!("output", "symbol", "3"), "DELETE FROM output"] },
    Recipe { name: "O01 none", kinds: VALUED, sql: &[del!("output", "symbol"), del!("spec", "symbol"), "DELETE FROM output"] },
    Recipe { name: "O02 O03 unknown", kinds: VALUED, sql: &[set!("output", "quantity", "'Wobble'"), set!("output", "unit", "'Furlong'")] },
    Recipe { name: "O03 unit not of the quantity", kinds: VALUED, sql: &[set!("output", "quantity", "'Angle'"), set!("output", "unit", "'Metre'")] },
    Recipe { name: "O03 none", kinds: VALUED, sql: &[set!("output", "unit", "''"), "UPDATE output SET unit = NULL"] },
    Recipe { name: "O04 bounds", kinds: VALUED, sql: &[set!("output", "lower", "'abc'"), set!("output", "upper", "'5'"), del!("output", "reason_upper")] },
    Recipe { name: "O04 order", kinds: VALUED, sql: &[set!("output", "lower", "' 0x10 '"), set!("output", "upper", "'1e1'"), set!("output", "reason_lower", "'r'"), set!("output", "reason_upper", "'r'")] },
    Recipe { name: "O04 Infinity and dots", kinds: VALUED, sql: &[set!("output", "lower", "'.5'"), set!("output", "upper", "'Infinity'"), set!("output", "reason_lower", "'r'"), set!("output", "reason_upper", "'r'")] },
    Recipe { name: "output table only", kinds: VALUED, sql: &[del!("output", "lower"), del!("output", "upper"), del!("spec", "lower"), del!("spec", "upper"), "UPDATE output SET lower = 1e-7, upper = 2.0"] },
    Recipe { name: "C01 no inputs", kinds: COMPUTED, sql: &["DELETE FROM input", del!("spec", "inputs")] },
    Recipe { name: "C01 inputs on a declared", kinds: &["declared", "kpi", "evidence"], sql: &["INSERT INTO input VALUES ('x', 'gm_0', NULL, NULL)"] },
    Recipe { name: "C02 bindings", kinds: COMPUTED, sql: &["DELETE FROM input", "INSERT INTO input VALUES ('1x', 'gm_0', NULL, NULL)", "INSERT INTO input VALUES ('', 'nosuch_node', NULL, NULL)", "INSERT INTO input VALUES (NULL, NULL, NULL, NULL)", "INSERT INTO input VALUES ('me', '{id}', NULL, NULL)"] },
    // a file holds a binding once; the spec's list can name one twice
    Recipe { name: "C02 twice", kinds: COMPUTED, sql: &["DELETE FROM input", set!("spec", "inputs", "'[[\"a\", \"gm_0\"], [\"a\", \"gm_1\"], [null, \"gm_2\"], [null, \"gm_3\"]]'")] },
    Recipe { name: "C03 C04 publisher", kinds: COMPUTED, sql: &["DELETE FROM input", "INSERT INTO input VALUES ('q', '{publisher}', 'Wobble', NULL)", "INSERT INTO input VALUES ('l', '{other_layer}', NULL, NULL)"] },
    Recipe { name: "R01 R07 R04", kinds: COMPUTED, sql: &[del!("relation", "source"), del!("spec", "source"), set!("relation", "derivation", "'[\"Ampere\", \"\", \" \", null, 3, []]'"), set!("relation", "expression", "'y = foo*bar + sqrt(foo) + mtq::dipole'")] },
    Recipe { name: "R01 all", kinds: COMPUTED, sql: &[set!("relation", "expression", "''"), set!("relation", "why", "'[]'"), set!("relation", "source", "NULL")] },
    Recipe { name: "R06 assumptions", kinds: VALUED, sql: &[set!("assumptions", "", "'[{\"assumes\": \"linear\", \"until\": \"\"}, {\"assumes\": \"a\", \"until\": \"b\"}, [\"x\"], \"text\", 5]'")] },
    Recipe { name: "R06 null assumption", kinds: VALUED, sql: &[set!("assumptions", "", "'[null]'")] },
    Recipe { name: "P01 no pseudocode", kinds: COMPUTED, sql: &[del!("code", "pseudocode")] },
    Recipe { name: "P02 does not parse", kinds: COMPUTED, sql: &[set!("code", "pseudocode", "'fn f(' || char(10) || 'end'")] },
    Recipe { name: "P02 no fn", kinds: COMPUTED, sql: &[set!("code", "pseudocode", "'const A = 1'")] },
    Recipe { name: "P02 other symbol", kinds: COMPUTED, sql: &[set!("code", "pseudocode", "'fn f(zz: real[1]) -> other: real[1]' || char(10) || '    other = zz' || char(10) || 'end'")] },
    Recipe { name: "P02 units", kinds: COMPUTED, sql: &[set!("code", "pseudocode", "'fn f(a: real[m], b: real[s]) -> c: real[m]' || char(10) || '    c = a + b' || char(10) || 'end' || char(10) || 'fn g() -> d: real[furlong]' || char(10) || '    d = 1' || char(10) || 'end'")] },
    Recipe { name: "P02 the app stops", kinds: COMPUTED, sql: &[set!("code", "pseudocode", "'fn f() -> y: real[1]' || char(10) || '    y = min()' || char(10) || 'end'")] },
    Recipe { name: "V01 declared", kinds: &["declared"], sql: &[set!("value", "number", "'abc'"), del!("value", "source"), del!("spec", "source")] },
    Recipe { name: "V01 bounds", kinds: &["declared"], sql: &[set!("value", "number", "'-1e9'"), set!("output", "lower", "'0'"), set!("output", "reason_lower", "'r'"), set!("output", "upper", "'1'"), set!("output", "reason_upper", "'r'")] },
    Recipe { name: "V01 above", kinds: &["declared"], sql: &[set!("value", "number", "'1e9'"), set!("output", "upper", "'1'"), set!("output", "reason_upper", "'r'")] },
    Recipe { name: "V02 sense", kinds: &["kpi"], sql: &[set!("requirement", "sense", "'sideways'")] },
    Recipe { name: "V02 sense from the spec", kinds: &["kpi"], sql: &[del!("requirement", "sense"), set!("spec", "sense", "'>='")] },
    Recipe { name: "V03 evidence", kinds: &["evidence"], sql: &[set!("evidence", "metric", "'nope'"), set!("evidence", "rungs", "'[]'")] },
    Recipe { name: "V03 evidence good", kinds: &["evidence"], sql: &[set!("evidence", "metric", "'ape'"), set!("evidence", "rungs", "'[\"sils\"]'")] },
    Recipe { name: "S01 new and unknown", kinds: VALUED, sql: &[set!("sources", "new", "'[{\"id\": \"mine\", \"title\": \"\", \"where\": \"p. 4\"}, {\"id\": \"good\", \"title\": \"t\", \"where\": \"w\"}]'"), set!("relation", "source", "'nosuchbook, good; Upper Case;mine,,wertz1978'"), set!("value", "source", "'unknown_src'")] },
    Recipe { name: "S01 null source", kinds: VALUED, sql: &[set!("sources", "new", "'[null]'")] },
    Recipe { name: "T01 T02 T03 vectors", kinds: COMPUTED, sql: &["DELETE FROM fixture", "INSERT INTO fixture VALUES ('v1', '{}', 'abc', 0, '{\"provenance\": \"self-snapshot\"}', 0)", "INSERT INTO fixture VALUES ('v2', 'not json', NULL, 'x', 'not json', 1)", "INSERT INTO fixture VALUES ('v3', '[1,2]', '', -1, '5', 1)", "INSERT INTO fixture VALUES ('v4', '{\"a\": \"1\"}', ' 2 ', 1e-9, '{\"provenance\": \"published-source\", \"source\": \"wertz1978\", \"where\": \"p. 1\"}', 1)"] },
    Recipe { name: "T03 inputs null", kinds: COMPUTED, sql: &["UPDATE fixture SET inputs = 'null'", "INSERT INTO fixture VALUES ('vx', 'null', '1', 1, '{}', 0)"] },
    Recipe { name: "readDoc stops on a null source", kinds: COMPUTED, sql: &["INSERT INTO fixture VALUES ('vn', '{}', '1', 1, 'null', 0)"] },
    Recipe { name: "T04 no vectors", kinds: COMPUTED, sql: &["DELETE FROM fixture"] },
    Recipe { name: "Y01 not numbers", kinds: VALUED, sql: &[set!("results", "table", "'{\"columns\": [{\"name\": \"a\"}, {\"name\": \"b\"}], \"rows\": [[\"1\", \"x\"], [\"\", null], [[], {}]], \"numeric\": [true, true]}'")] },
    Recipe { name: "Y01 unreadable", kinds: VALUED, sql: &[set!("results", "table", "'not json'")] },
    Recipe { name: "Y01 no columns", kinds: VALUED, sql: &[set!("results", "table", "'{\"rows\": [[\"x\"]], \"numeric\": \"t\"}'")] },
    Recipe { name: "Y01 rows a number", kinds: VALUED, sql: &[set!("results", "table", "'{\"rows\": 5}'")] },
    Recipe { name: "D01 D03 belief", kinds: VALUED, sql: &[del!("belief", "area"), set!("belief", "status", "'maybe'"), set!("belief", "believed", "'b'")] },
    Recipe { name: "D03 held untested", kinds: VALUED, sql: &[set!("belief", "status", "'held'"), del!("belief", "tested"), set!("belief", "cost_k", "'-1'")] },
    Recipe { name: "D07 untested", kinds: VALUED, sql: &[set!("belief", "status", "'untested'"), set!("belief", "cost_k", "'lots'")] },
    Recipe { name: "D07 untested with its test", kinds: VALUED, sql: &[set!("belief", "status", "'untested'"), set!("belief", "tested", "'a campaign'"), set!("belief", "cost_k", "'0'")] },
    Recipe { name: "spec inputs", kinds: COMPUTED, sql: &["DELETE FROM input", set!("spec", "inputs", "'[[\"a\", \"gm_0\"], \"xy\", [\"c\"]]'")] },
    Recipe { name: "spec inputs that stop", kinds: COMPUTED, sql: &["DELETE FROM input", set!("spec", "inputs", "'[[\"a\", \"gm_0\"], 5]'")] },
    Recipe { name: "spec assumptions and explain", kinds: VALUED, sql: &[del!("assumptions", ""), set!("spec", "assumptions", "'[[\"a\", \"b\"], [\"c\"], {\"assumes\": \"d\"}]'"), set!("spec", "explain", "'{\"simply\": \"s\", \"1\": 2, \"analogy\": [1, 2], \"wrong_idea\": \"w\"}'"), del!("explain", "simply"), del!("explain", "analogy")] },
    Recipe { name: "spec explain a string", kinds: VALUED, sql: &[set!("spec", "explain", "'\"ab\"'")] },
    Recipe { name: "spec bounds", kinds: VALUED, sql: &[del!("output", "lower"), del!("output", "upper"), "DELETE FROM output", set!("spec", "lower", "'1.0'"), set!("spec", "upper", "NULL")] },
    Recipe { name: "a blob", kinds: VALUED, sql: &[set!("identity", "question", "X'3132'"), set!("output", "lower", "X'35'"), set!("output", "reason_lower", "'r'")] },
    Recipe { name: "numbers as text", kinds: VALUED, sql: &[set!("output", "lower", "'-0'"), set!("output", "upper", "'1_000'"), set!("output", "reason_lower", "'r'"), set!("output", "reason_upper", "'r'"), set!("belief", "cost_k", "'0b101'")] },
];

/// a copy of `src` broken by these SQL statements
fn mutate(src: &Path, dst: &Path, sql: &[&str], subst: &BTreeMap<&str, String>) {
    std::fs::copy(src, dst).unwrap();
    let c = Connection::open(dst).unwrap();
    for s in sql {
        let mut s = s.to_string();
        for (k, v) in subst {
            s = s.replace(k, v);
        }
        c.execute_batch(&s).unwrap_or_else(|e| panic!("{s}: {e}"));
    }
}

#[test]
fn the_library_checks_every_node_as_the_node_app_does() {
    let tmp = std::env::temp_dir().join(format!("trinetra-node-rules-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    let Some(d) = design(&tmp) else {
        println!("python3 is not installed: not compared");
        return;
    };
    let ctx = Context::from_folder(&d).unwrap();
    let mut files: Vec<PathBuf> = std::fs::read_dir(d.join("nodes")).unwrap().map(|e| e.unwrap().path()).filter(|p| p.to_string_lossy().ends_with(".node.tndb")).collect();
    files.sort();
    assert_eq!(files.len(), 765, "the carried design's nodes");

    // the broken copies: each recipe on up to six nodes of a kind it applies to, then 200 of two at once
    let kinds: Vec<(PathBuf, String, String)> = files
        .iter()
        .map(|f| {
            let doc = node_rules::read_doc(&Connection::open(f).unwrap()).unwrap();
            (f.clone(), doc.kind.unwrap_or("none").to_string(), doc.node["layer"].text())
        })
        .collect();
    let ids: Vec<(&String, &node_rules::OtherNode)> = ctx.nodes.as_ref().unwrap().iter().collect();
    let publisher = ids.iter().filter(|(_, n)| matches!(&n.quantity, node_rules::J::Str(q) if !q.is_empty())).map(|(id, _)| id.to_string()).min().unwrap();
    let mdir = tmp.join("mutants");
    std::fs::create_dir_all(&mdir).unwrap();
    let mut mutants: Vec<(PathBuf, String)> = Vec::new();
    let mut make = |base: &(PathBuf, String, String), recipes: &[&Recipe], n: usize| {
        let id = base.0.file_name().unwrap().to_string_lossy().trim_end_matches(".node.tndb").to_string();
        let other_layer = ids
            .iter()
            .filter(|(_, o)| o.layer.text() != base.2 && !o.kind.text().contains("interface") && o.layer.text() != "null")
            .map(|(i, _)| i.to_string())
            .min()
            .unwrap_or_else(|| "gm_0".into());
        let subst = BTreeMap::from([("{id}", id.clone()), ("{publisher}", publisher.clone()), ("{other_layer}", other_layer)]);
        let dst = mdir.join(format!("m{n:04}_{id}.node.tndb"));
        let sql: Vec<&str> = recipes.iter().flat_map(|r| r.sql.iter().copied()).collect();
        mutate(&base.0, &dst, &sql, &subst);
        mutants.push((dst, format!("{} on {id}", recipes.iter().map(|r| r.name).collect::<Vec<_>>().join(" + "))));
    };
    let mut n = 0;
    for r in RECIPES {
        let bases: Vec<&(PathBuf, String, String)> = kinds.iter().filter(|k| r.kinds.contains(&k.1.as_str())).collect();
        for j in 0..6.min(bases.len()) {
            make(bases[j * bases.len() / 6.min(bases.len())], &[r], n);
            n += 1;
        }
    }
    let mut seed: u64 = 20261005;
    let mut rnd = |m: usize| {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        (seed % m as u64) as usize
    };
    for _ in 0..200 {
        let (a, b) = (&RECIPES[rnd(RECIPES.len())], &RECIPES[rnd(RECIPES.len())]);
        let bases: Vec<&(PathBuf, String, String)> = kinds.iter().filter(|k| a.kinds.contains(&k.1.as_str()) && b.kinds.contains(&k.1.as_str())).collect();
        if bases.is_empty() {
            continue;
        }
        make(bases[rnd(bases.len())], &[a, b], n);
        n += 1;
    }

    let all: Vec<PathBuf> = files.iter().cloned().chain(mutants.iter().map(|m| m.0.clone())).collect();
    let mine: Vec<Result<Vec<[String; 4]>, ()>> = all.iter().map(|f| rust(f, &ctx)).collect();
    let Some(js) = javascript(&d, &all) else {
        println!("Node is not installed: {} node files checked, not compared", all.len());
        return;
    };
    let mut bad = Vec::new();
    let mut fired: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    let (mut stopped, mut problems) = (0, 0);
    for (k, (m, j)) in mine.iter().zip(&js).enumerate() {
        let what = if k < files.len() { files[k].file_name().unwrap().to_string_lossy().to_string() } else { mutants[k - files.len()].1.clone() };
        match j {
            Ok(p) => {
                problems += p.len();
                for x in p {
                    let e = fired.entry(x[0].clone()).or_default();
                    if k < files.len() { e.0 += 1 } else { e.1 += 1 }
                }
            }
            Err(()) => stopped += 1,
        }
        if m != j {
            bad.push(format!("{what}:\n  node app:\n{}\n  library:\n{}", show(j), show(m)));
        }
    }
    println!(
        "{} node files of the carried design and {} broken copies: {problems} problems, {stopped} where the checks stop, the same in both",
        files.len(),
        mutants.len()
    );
    println!("rule: (in the design, in the broken copies)");
    for (c, (a, b)) in &fired {
        println!("  {c}: ({a}, {b})");
    }
    assert!(bad.is_empty(), "{} differ; the first:\n{}", bad.len(), bad[..bad.len().min(5)].join("\n"));
    let missing: Vec<&&str> = RULES.iter().filter(|r| !fired.contains_key(**r)).collect();
    assert!(missing.is_empty(), "rules that never fired: {missing:?}");
    assert!(stopped > 0, "a node the app stops on is among the broken copies");
    let _ = std::fs::remove_dir_all(&tmp);
}
