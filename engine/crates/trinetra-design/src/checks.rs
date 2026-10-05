//! The checks 1.0.0's tools make on a design folder and on a sealed release, in the library
//! (docs/PLAN_2_0.md S2: "the library refuses everything today's tools refuse").
//!
//! - [`check_folder`] is `tools/group.py check`: every group file and node file of a folder against
//!   each other (a node in two groups, a stage that does not exist, an edge kept twice or into
//!   another's node, a read of a node no group holds or one archived, a node without its file, a
//!   node file whose fields disagree with its group, an unfinished structure action).
//! - [`check_release`] is `tools/release.py check`: a sealed release's name, version, every node's
//!   fingerprints and body, how each was sealed (confirmed only when checked by someone other than
//!   its author, and, for a computing node, with a test vector from outside the code), the release
//!   fingerprint, its nodes against the group's, and the lead's seal.
//!
//! The messages are the tools' own, word for word, so either can judge and the test can hold the two
//! to each other (tests/test_library_checks.py).
//!
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use crate::{check, open, Cell, Schema};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// Python's truthiness of a JSON value: null, false, 0, "", [] and {} are false.
fn truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|x| x != 0.0),
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}

fn sha(t: &str) -> String { Sha256::digest(t.as_bytes()).iter().map(|b| format!("{b:02x}")).collect() }

/// Python's repr of a cell, as the tools print it in a message.
fn repr(c: &Cell) -> String {
    match c {
        Cell::Null => "None".into(),
        Cell::Integer(i) => i.to_string(),
        Cell::Real(r) => r.to_string(),
        Cell::Text(t) => if t.contains('\'') && !t.contains('"') { format!("\"{t}\"") } else { format!("'{}'", t.replace('\\', "\\\\").replace('\'', "\\'")) },
        Cell::Blob(_) => "b'...'".into(),
    }
}

fn txt(c: &Cell) -> Option<String> { match c { Cell::Text(t) => Some(t.clone()), Cell::Null => None, Cell::Integer(i) => Some(i.to_string()), Cell::Real(r) => Some(r.to_string()), Cell::Blob(_) => None } }

/// A table's rows as name -> cell maps, in the file's own order.
fn rows(f: &crate::DesignFile, t: &str, s: &Schema) -> Vec<BTreeMap<String, Cell>> {
    let cols: Vec<String> = s.columns(t).into_iter().map(|c| c.0).collect();
    f.rows(t, s).unwrap_or_default().into_iter().map(|r| cols.iter().cloned().zip(r).collect()).collect()
}

struct Group {
    stages: Vec<BTreeMap<String, Cell>>,
    nodes: Vec<BTreeMap<String, Cell>>,
    edges: Vec<BTreeMap<String, Cell>>,
    contracts: Vec<BTreeMap<String, Cell>>,
    members: BTreeSet<String>,
    member_nodes: Vec<BTreeMap<String, Cell>>,
}

impl Group {
    fn node(&self, id: &str) -> Option<&BTreeMap<String, Cell>> { self.nodes.iter().find(|n| txt(&n["id"]).as_deref() == Some(id)) }
}

fn g(m: &BTreeMap<String, Cell>, k: &str) -> String { m.get(k).and_then(txt).unwrap_or_default() }

/// Every problem `tools/group.py check` finds in a design folder, in the same words.
pub fn check_folder(root: &Path, s: &Schema) -> Vec<String> {
    let mut problems = Vec::new();
    let mut groups: Vec<(String, Group)> = Vec::new();
    let mut files: Vec<_> = std::fs::read_dir(root.join("structure")).map(|rd| rd.flatten().map(|e| e.path())
        .filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.ends_with(".group.tndb"))).collect()).unwrap_or_default();
    files.sort();
    for f in files {
        let errs = check(&f, s);
        if !errs.is_empty() { problems.extend(errs); continue; }
        let Ok(df) = open(&f, None, s) else { continue };
        let name = f.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
        let info = rows(&df, "group_info", s);
        let gid = info.first().map(|i| g(i, "id")).filter(|x| !x.is_empty()).unwrap_or_else(|| name.trim_end_matches(".group.tndb").to_string());
        if name != format!("{gid}.group.tndb") {
            problems.push(format!("structure/{name}: its group is {gid}; the file is named for another"));
        }
        groups.push((gid, Group {
            stages: rows(&df, "stage", s), nodes: rows(&df, "group_node", s), edges: rows(&df, "edge", s), contracts: rows(&df, "contract", s),
            members: rows(&df, "member", s).iter().map(|m| g(m, "name")).collect(), member_nodes: rows(&df, "member_node", s),
        }));
    }
    // a later file of the same group replaces the earlier, as a dict would
    let mut by_id: Vec<(String, Group)> = Vec::new();
    for (gid, grp) in groups {
        if let Some(i) = by_id.iter().position(|(x, _)| *x == gid) { by_id[i].1 = grp; } else { by_id.push((gid, grp)); }
    }
    let groups = by_id;
    let mut owner: BTreeMap<String, String> = BTreeMap::new();
    for (gid, grp) in &groups {
        for n in &grp.nodes {
            let nid = g(n, "id");
            if let Some(o) = owner.get(&nid) { problems.push(format!("node {nid} is in two groups: {o} and {gid}")); }
            owner.entry(nid).or_insert_with(|| gid.clone());
        }
    }
    let group = |id: &str| groups.iter().find(|(x, _)| x == id).map(|(_, g)| g);
    for (gid, grp) in &groups {
        let stages: BTreeSet<String> = grp.stages.iter().map(|x| g(x, "id")).collect();
        for n in &grp.nodes {
            let st = g(n, "stage");
            if !st.is_empty() && !stages.contains(&st) { problems.push(format!("{gid}: node {} is in stage {st}, which the group does not have", g(n, "id"))); }
            if st.is_empty() && !stages.is_empty() { problems.push(format!("{gid}: node {} is in no stage; the group has stages", g(n, "id"))); }
        }
        let mut seen = BTreeSet::new();
        for e in &grp.edges {
            let (from, to, kind) = (g(e, "from_node"), g(e, "to_node"), g(e, "kind"));
            if !seen.insert((from.clone(), to.clone(), kind.clone())) { problems.push(format!("{gid}: edge {from} -> {to} ({kind}) twice")); }
            if grp.node(&to).is_none() { problems.push(format!("{gid}: keeps the edge {from} -> {to}, but {to} is not its node")); }
            match owner.get(&from) {
                None => problems.push(format!("{gid}: {to} reads {from}, which is in no group")),
                Some(fg) => {
                    let archived = group(fg).and_then(|x| x.node(&from)).is_some_and(|n| g(n, "state") == "archived");
                    let reader_archived = grp.node(&to).is_some_and(|n| g(n, "state") == "archived");
                    if archived && !reader_archived { problems.push(format!("{gid}: {to} reads {from}, which {fg} archived")); }
                }
            }
            if from == to { problems.push(format!("{gid}: {to} reads itself")); }
        }
        for m in &grp.member_nodes {
            if grp.node(&g(m, "node")).is_none() { problems.push(format!("{gid}: {} authors {}, which is not its node", g(m, "author"), g(m, "node"))); }
        }
        for c in &grp.contracts {
            if grp.node(&g(c, "node")).is_none() { problems.push(format!("{gid}: a contract on {}, which is not its node", g(c, "node"))); }
        }
        for st in &grp.stages {
            let o = g(st, "owner");
            if !o.is_empty() && !grp.members.contains(&o) { problems.push(format!("{gid}: stage {} is owned by {o}, who is not a member", g(st, "id"))); }
        }
        for n in &grp.nodes {
            let nid = g(n, "id");
            let fname = format!("{nid}.node.tndb");
            let f = root.join("nodes").join(&fname);
            if !f.is_file() { problems.push(format!("{gid}: node {nid} has no file nodes/{fname}")); continue; }
            let errs = check(&f, s);
            if !errs.is_empty() { problems.extend(errs); continue; }
            let Ok(df) = open(&f, None, s) else { continue };
            let Some(row) = rows(&df, "node", s).into_iter().next() else { problems.push(format!("nodes/{fname}: no node row")); continue };
            for k in ["id", "group_id", "stage", "label", "state"] {
                let want = if k == "group_id" { Cell::Text(gid.clone()) } else { n.get(k).cloned().unwrap_or(Cell::Null) };
                let have = row.get(k).cloned().unwrap_or(Cell::Null);
                if have != want { problems.push(format!("nodes/{fname}: {k} is {}, its group {gid} says {}", repr(&have), repr(&want))); }
            }
        }
    }
    let mut nodes: Vec<String> = std::fs::read_dir(root.join("nodes")).map(|rd| rd.flatten().filter_map(|e| e.file_name().to_str().map(String::from))
        .filter(|n| n.ends_with(".node.tndb")).collect()).unwrap_or_default();
    nodes.sort();
    for n in nodes {
        if !owner.contains_key(n.trim_end_matches(".node.tndb")) { problems.push(format!("nodes/{n}: in no group")); }
    }
    let mut actions: Vec<_> = std::fs::read_dir(root.join("structure").join("actions")).map(|rd| rd.flatten().map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "json")).collect()).unwrap_or_default();
    actions.sort();
    for a in actions {
        let name = a.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
        match std::fs::read_to_string(&a).ok().and_then(|t| serde_json::from_str::<Value>(&t).ok()) {
            Some(v) if v.is_object() => {
                let done = v.get("done").is_some_and(truthy);
                if !done { problems.push(format!("structure/actions/{name}: a structure action not finished (the group app finishes it)")); }
            }
            _ => problems.push(format!("structure/actions/{name}: not readable")),
        }
    }
    problems
}

const FIXED_KINDS: [&str; 4] = ["interface", "closure_interface", "required", "achieved"];

/// Whether a sealed node computes: the spec's catalogue says so (`catalogue`: id -> "computed" or
/// another kind), or, for a row it has not named, its author chose it (release.py `computing`).
fn computing(body: &Value, catalogue: &BTreeMap<String, String>) -> bool {
    let node = &body["node"];
    let kind = node["kind"].as_str().unwrap_or("");
    if FIXED_KINDS.contains(&kind) || kind.starts_with("closure") { return false; }
    if let Some(k) = node["id"].as_str().and_then(|id| catalogue.get(id)) { return k == "computed"; }
    body["content"].as_array().into_iter().flatten().any(|c| c[0] == "identity" && c[1] == "form_kind" && c[2] == "computed")
}

/// Every problem `tools/release.py check` finds in one release file, in the same words.
pub fn check_release(path: &Path, catalogue: &BTreeMap<String, String>, s: &Schema) -> Vec<String> {
    let p = path.display().to_string();
    let errs = check(path, s);
    if !errs.is_empty() { return errs; }
    let Ok(f) = open(path, None, s) else { return vec![format!("{p}: unreadable")] };
    let rel = rows(&f, "release", s);
    if rel.len() != 1 { return vec![format!("{p}: {} release rows, not one", rel.len())]; }
    let rel = &rel[0];
    let (group_id, version, fp) = (g(rel, "group_id"), g(rel, "version"), g(rel, "fingerprint"));
    let mut problems = Vec::new();
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if name != format!("{group_id}-{version}.tnrel") { problems.push(format!("{p}: holds {group_id} {version}; the file is named for another")); }
    let ok_version = { let mut it = version.split('.'); matches!((it.next(), it.next(), it.next()), (Some(a), Some(b), None) if !a.is_empty() && !b.is_empty() && a.chars().all(|c| c.is_ascii_digit()) && b.chars().all(|c| c.is_ascii_digit())) };
    if !ok_version { problems.push(format!("{p}: version {} is not MAJOR.MINOR", repr(&rel["version"]))); }
    let mut nodes = rows(&f, "release_node", s);
    nodes.sort_by_key(|r| g(r, "id"));
    let (mut lines, mut ids) = (Vec::new(), BTreeSet::new());
    for rn in &nodes {
        let (nid, content, nfp) = (g(rn, "id"), g(rn, "content"), g(rn, "fingerprint"));
        let wh = format!("{p}: {nid}");
        ids.insert(nid.clone());
        lines.push(format!("{nid} {nfp}"));
        if sha(&content) != nfp { problems.push(format!("{wh}: its fingerprint is not the SHA-256 of its content")); }
        let x: Value = match serde_json::from_str(&content) { Ok(v) => v, Err(e) => { problems.push(format!("{wh}: its content is not a sealed node ({e})")); continue } };
        let Some(body_text) = x["body"].as_str() else { problems.push(format!("{wh}: its content is not a sealed node ('body')")); continue };
        let body: Value = match serde_json::from_str(body_text) { Ok(v) => v, Err(e) => { problems.push(format!("{wh}: its content is not a sealed node ({e})")); continue } };
        if Some(sha(body_text).as_str()) != x["body_fingerprint"].as_str() { problems.push(format!("{wh}: its body fingerprint is not the SHA-256 of its body")); }
        if body["node"]["id"].as_str() != Some(nid.as_str()) {
            let about = body["node"]["id"].as_str().map(|v| repr(&Cell::Text(v.into()))).unwrap_or_else(|| "None".into());
            problems.push(format!("{wh}: its body is about {about}"));
        }
        match x["sealed_as"].as_str() {
            Some("unconfirmed") => if !truthy(&x["why"]) { problems.push(format!("{wh}: sealed unconfirmed with no reason given")); },
            Some("confirmed") => {
                let author = body["node"]["author"].as_str().map(String::from);
                let checkers: Vec<String> = body["signature"].as_array().into_iter().flatten()
                    .filter(|s| s[0] == "checked by").filter_map(|s| s[1].as_str().map(String::from)).collect();
                if checkers.last().is_none() || checkers.last() == author.as_ref() {
                    let a = author.clone().unwrap_or_else(|| "None".into());
                    problems.push(format!("{wh}: sealed as confirmed, but nobody other than its author ({a}) checked it"));
                }
                let outside = body["fixture"].as_array().into_iter().flatten().any(|fx| truthy(&fx[5]));
                if computing(&body, catalogue) && !outside {
                    problems.push(format!("{wh}: a computing node sealed as confirmed with no test vector from outside the code"));
                }
            }
            other => {
                let shown = other.map(|v| repr(&Cell::Text(v.into()))).unwrap_or_else(|| "None".into());
                problems.push(format!("{wh}: sealed as {shown}, not confirmed or unconfirmed"));
            }
        }
    }
    lines.sort();
    if sha(&lines.join("\n")) != fp { problems.push(format!("{p}: the release fingerprint is not the SHA-256 of its nodes' fingerprints")); }
    let live: BTreeSet<String> = rows(&f, "group_node", s).iter().filter(|r| g(r, "state") != "archived").map(|r| g(r, "id")).collect();
    if live != ids {
        let show = |v: Vec<&String>| format!("[{}]", v.iter().take(5).map(|x| format!("'{x}'")).collect::<Vec<_>>().join(", "));
        problems.push(format!("{p}: its nodes are not the group's: missing {}, extra {}", show(live.difference(&ids).collect()), show(ids.difference(&live).collect())));
    }
    let seals: Vec<_> = rows(&f, "signature", s).into_iter().filter(|r| g(r, "role") == "sealed").collect();
    let st: Value = seals.last().and_then(|x| serde_json::from_str(&g(x, "statement")).ok()).unwrap_or(Value::Null);
    let sealed_by = g(rel, "sealed_by");
    let good = seals.last().is_some_and(|x| g(x, "name") == sealed_by) && st["version"].as_str() == Some(version.as_str()) && st["fingerprint"].as_str() == Some(fp.as_str());
    if !good { problems.push(format!("{p}: no seal by {sealed_by} naming {version} and its fingerprint")); }
    problems
}

/// Each group's newest release in `root/releases`: group id -> path (`group.py latest_releases`).
pub fn latest_releases(root: &Path) -> BTreeMap<String, std::path::PathBuf> {
    let mut best: BTreeMap<String, ((u64, u64), std::path::PathBuf)> = BTreeMap::new();
    let files = std::fs::read_dir(root.join("releases")).map(|rd| rd.flatten().map(|e| e.path()).collect::<Vec<_>>()).unwrap_or_default();
    for f in files {
        let Some(name) = f.file_name().and_then(|n| n.to_str()).and_then(|n| n.strip_suffix(".tnrel")).map(String::from) else { continue };
        let Some((gid, ver)) = name.rsplit_once('-') else { continue };
        let Some((a, b)) = ver.split_once('.') else { continue };
        let (Ok(a), Ok(b)) = (a.parse::<u64>(), b.parse::<u64>()) else { continue };
        if !(ver.split('.').all(|x| !x.is_empty() && x.chars().all(|c| c.is_ascii_digit()))) { continue; }
        if best.get(gid).is_none_or(|(k, _)| (a, b) > *k) { best.insert(gid.to_string(), ((a, b), f)); }
    }
    best.into_iter().map(|(g, (_, p))| (g, p)).collect()
}

/// Every group's latest release checked against the folder as it is now (`group.py verify`):
/// group id -> its problems; a group with no release says so.
pub fn verify_releases(root: &Path, catalogue: &BTreeMap<String, String>, s: &Schema) -> BTreeMap<String, Vec<String>> {
    // the folder's groups, their live nodes, and who owns each node
    let mut groups: BTreeMap<String, Vec<BTreeMap<String, Cell>>> = BTreeMap::new();
    let mut files: Vec<_> = std::fs::read_dir(root.join("structure")).map(|rd| rd.flatten().map(|e| e.path())
        .filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.ends_with(".group.tndb"))).collect()).unwrap_or_default();
    files.sort();
    for f in files {
        if !check(&f, s).is_empty() { continue; }
        let Ok(df) = open(&f, None, s) else { continue };
        let name = f.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
        let gid = rows(&df, "group_info", s).first().map(|i| g(i, "id")).filter(|x| !x.is_empty()).unwrap_or_else(|| name.trim_end_matches(".group.tndb").to_string());
        groups.insert(gid, rows(&df, "group_node", s));
    }
    let owner: BTreeSet<String> = groups.values().flatten().map(|n| g(n, "id")).collect();
    let rels = latest_releases(root);
    let mut out = BTreeMap::new();
    for (gid, nodes) in &groups {
        let Some(path) = rels.get(gid) else { out.insert(gid.clone(), vec![format!("{gid}: no release yet (the group app seals one)")]); continue };
        let mut p = check_release(path, catalogue, s);
        if let Ok(f) = open(path, None, s) {
            let rel = rows(&f, "release", s);
            let (rgid, ver) = rel.first().map(|r| (g(r, "group_id"), g(r, "version"))).unwrap_or_default();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if rgid != *gid { p.push(format!("{name}: a release of {rgid}, not {gid}")); }
            let sealed: BTreeSet<String> = rows(&f, "release_node", s).iter().map(|r| g(r, "id")).collect();
            let mine: BTreeSet<String> = nodes.iter().filter(|n| g(n, "state") != "archived").map(|n| g(n, "id")).collect();
            for n in sealed.difference(&mine) { p.push(format!("{gid} {ver}: {n} is no longer a node of {gid} (moved or archived since): seal again")); }
            for n in mine.difference(&sealed) { p.push(format!("{gid} {ver}: {n} is a node of {gid} the release does not hold (added since): seal again")); }
            for e in rows(&f, "edge", s) {
                if !owner.contains(&g(&e, "from_node")) { p.push(format!("{gid} {ver}: {} reads {}, which is in no group", g(&e, "to_node"), g(&e, "from_node"))); }
            }
            for c in rows(&f, "contract", s) {
                for r in g(&c, "readers").split(',').filter(|x| !x.is_empty()) {
                    if !groups.contains_key(r) { p.push(format!("{gid} {ver}: the contract on {} names {r}, which is not a group", g(&c, "node"))); }
                }
            }
        }
        out.insert(gid.clone(), p);
    }
    out
}
