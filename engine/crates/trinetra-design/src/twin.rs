//! The MATLAB twin's face of a design database (docs/PLAN_2_0.md S7, "the twin opens the database itself";
//! docs/S7_INVENTORY.md S7.18): what `trinetra.open`, `trinetra.build` and `trinetra.run` ask of the library over
//! `system()`, so the twin needs no toolbox and no Python at run time.
//!
//! - [`inputs`]: the twin's inputs, every engine input (`data/...`, each held to its fingerprint) and every case
//!   (`cases/<id>.csv`, its lines under the one header), the same bytes `tools/from_design.py` writes into
//!   `matlab_sils/data` and `matlab_sils/cases` and the engine reads from the database (`adcs-sim` source.rs);
//! - [`health`]: what the design is and how it stands (its meta, its nodes by behaviour, the built-in count, its
//!   groups' releases, its inputs, its signatures), for `trinetra.open` to show;
//! - [`build_matlab`]: every MATLAB function the twin flies, translated from the design by the library's own
//!   translator (trinetra-pcode), byte for byte what `tools/engine_build.py` (`+asils/+models`, `+asils/+relations`)
//!   and `tools/flight_build.py` (`+asils/+alg`) write, with the language's runtime (`+asils/+pc`), and an index
//!   naming the node, release and revision each file came from (`trinetra.which`).
//!
//! The selection of what each package takes is the tools': a module is the text a node's origin names
//! (from_design.py `_held`), a target takes every method block whose `code.generate` names it and what its
//! `code.uses` names, in turn (engine_build.py `modules`); the relations are the design's library and the groups'
//! wiring, each relation once (engine_build.py `relations_sources`); the flight algorithms are the toolbox and the
//! design's `fsw/pseudocode/02` to `09` blocks (flight_build.py). The toolbox (`fsw/pseudocode/01_math.pc`) is code
//! and is compiled in. tests/test_trinetra_open.py holds the two to one answer.
//!
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use rusqlite::{Connection, OpenFlags};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// The header every case file starts with (adcs-case/1; adcs-sim source.rs, tools/design_inputs.py).
pub const CASE_HEADER: &str = "section,key,label,unit,value,lo,hi,level,note";
/// The toolbox's pseudocode (code, not design: engine_build.py TOOLBOX), as this program was built with it.
pub const TOOLBOX: &[(&str, &str)] = &[("fsw/pseudocode/01_math.pc", include_str!("../../../../fsw/pseudocode/01_math.pc"))];
/// The engine's targets whose modules the twin's `+asils/+models` takes (engine_build.py TARGETS, TWIN).
pub const MODEL_TARGETS: &[&str] = &["adcs-sim-core", "adcs-pop", "adcs-sim", "adcs-design"];
/// Where each package goes under the generated folder, and its MATLAB package.
pub const MODELS: (&str, &str) = ("+asils/+models", "asils.models");
pub const RELATIONS: (&str, &str) = ("+asils/+relations", "asils.relations");
pub const ALG: (&str, &str) = ("+asils/+alg", "asils.alg");
pub const RUNTIME: &str = "+asils/+pc";
/// The relations library's folder in the design (engine_build.py RELATIONS["library"]).
pub const LIBRARY: &str = "spec/physics/";
/// The flight build's C and Rust options, whose sources the algorithms' identity is the hash of (flight_build.py).
pub const ALG_TITLE: &str = "TRI-NETRA flight algorithms, written from the design by tools/flight_build.py";
pub const ALG_LIB: &str = "adcs_alg";
/// The index the build writes beside the packages, and what says a folder is one the build made.
pub const INDEX: &str = "index.json";
pub const MADE_BY: &str = "tndb build-matlab";
/// The paths a node's origin may name for a text the design keeps whole (from_design.py `_held`).
const HELD_PREFIXES: &[&str] = &["spec/", "catalogue/", "fsw/", "matlab_sils/", "env/", "dyn/", "act/", "sens/", "oils/", "pnt/",
                                 "design/", "kpi/", "ctl/"];
const DECL_KINDS: &[&str] = &["fn", "proc", "record", "table", "const", "data", "choice"];

/// FNV-1a 64 over bytes, as hex: the fingerprint the database gives every engine input (adcs-sim source.rs).
pub fn fnv_hex(b: &[u8]) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    for x in b {
        h ^= *x as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    format!("{h:016x}")
}

fn sha_hex(b: &[u8]) -> String {
    Sha256::digest(b).iter().map(|x| format!("{x:02x}")).collect()
}

/// A design database, opened read-only; refused by name when it is not one that holds the engine's inputs.
pub fn connect(path: &Path) -> Result<Connection, String> {
    let p = path.display();
    if path.is_dir() {
        return Err(format!("{p}: a folder, not a design database"));
    }
    if !path.is_file() {
        return Err(format!("{p}: no such file"));
    }
    let c = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX)
        .map_err(|e| format!("{p}: {e}"))?;
    let m = meta(&c).map_err(|e| format!("{p}: {e}"))?;
    if m.get("format").map(String::as_str) != Some("design") {
        return Err(format!("{p}: a {} file, not a design database", m.get("format").map(String::as_str).unwrap_or("?")));
    }
    let v: u32 = m.get("format_version").and_then(|v| v.parse().ok()).unwrap_or(0);
    if v < 2 {
        return Err(format!("{p}: format version {v} holds no engine inputs: rebuild it (python3 tools/seed_design.py)"));
    }
    Ok(c)
}

fn meta(c: &Connection) -> Result<BTreeMap<String, String>, String> {
    c.prepare(r#"SELECT "key", "value" FROM meta"#)
        .and_then(|mut s| s.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?.unwrap_or_default())))?.collect())
        .map_err(|e| if e.to_string().contains("not a database") { "not a database file (damaged, or not a design file)".into() }
                     else { format!("no meta table: not a design file ({e})") })
}

/// Which of the twin's inputs to give.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Which {
    All,
    EngineInputs,
    Cases,
}

/// The twin's inputs, by their path in the data folder (`data/...`, `cases/<id>.csv`), sorted by path: the bytes
/// tools/from_design.py writes for the twin and the engine reads from the database. An engine input that is not
/// what its fingerprint says is refused by name.
pub fn inputs(path: &Path, which: Which) -> Result<Vec<(String, Vec<u8>)>, String> {
    let c = connect(path)?;
    let p = path.display();
    let mut out: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    if which != Which::Cases {
        let rows: Vec<(String, String, Vec<u8>)> = c.prepare(r#"SELECT "path", "fingerprint", "body" FROM engine_input ORDER BY "path""#)
            .and_then(|mut s| s.query_map([], |r| Ok((r.get(0)?, r.get::<_, Option<String>>(1)?.unwrap_or_default(),
                                                        r.get::<_, Option<Vec<u8>>>(2)?.unwrap_or_default())))?.collect())
            .map_err(|e| format!("{p}: {e}"))?;
        for (k, fp, body) in rows {
            if fnv_hex(&body) != fp {
                return Err(format!("{p}: {k} is not what its fingerprint says (changed outside the tools?)"));
            }
            out.insert(k, body);
        }
    }
    if which != Which::EngineInputs {
        let rows: Vec<(String, String)> = c.prepare(r#"SELECT "case_id", "line" FROM design_case ORDER BY "case_id", "ord""#)
            .and_then(|mut s| s.query_map([], |r| Ok((r.get(0)?, r.get::<_, Option<String>>(1)?.unwrap_or_default())))?.collect())
            .map_err(|e| format!("{p}: {e}"))?;
        let mut cases: BTreeMap<String, String> = BTreeMap::new();
        for (case, line) in rows {
            let t = cases.entry(case).or_insert_with(|| format!("{CASE_HEADER}\n"));
            t.push_str(&line);
            t.push('\n');
        }
        for (case, text) in cases {
            out.insert(format!("cases/{case}.csv"), text.into_bytes());
        }
    }
    Ok(out.into_iter().collect())
}

/// `[{"path": ..., "body": ...}, ...]`: the inputs as the twin decodes them (jsondecode: one struct array).
pub fn inputs_json(items: &[(String, Vec<u8>)]) -> Result<String, String> {
    let mut v = Vec::with_capacity(items.len());
    for (k, b) in items {
        let body = String::from_utf8(b.clone()).map_err(|_| format!("{k}: not UTF-8 text"))?;
        v.push(json!({"path": k, "body": body}));
    }
    serde_json::to_string(&Value::Array(v)).map_err(|e| e.to_string())
}

/// One node of the design: its group, its release and its content's rows.
struct Node {
    id: String,
    group: String,
    release: String,
    /// [section, field, value, origin]
    rows: Vec<(String, String, String, String)>,
    /// its first block's behaviour, and whether any block is built-in
    behaviour: String,
    built_in: bool,
    content: String,
}

fn s(v: &Value) -> String {
    v.as_str().map(String::from).unwrap_or_default()
}

/// Every node, in the order the query gives them (`ORDER BY id` when `by_id`, else the table's own, as the tools ask).
fn nodes(c: &Connection, by_id: bool) -> Result<Vec<Node>, String> {
    let sql = if by_id {
        r#"SELECT "id", "group_id", "release", "content" FROM design_node ORDER BY "id""#
    } else {
        r#"SELECT "id", "group_id", "release", "content" FROM design_node"#
    };
    let raw: Vec<(String, String, String, String)> = c.prepare(sql)
        .and_then(|mut st| st.query_map([], |r| Ok((r.get(0)?, r.get::<_, Option<String>>(1)?.unwrap_or_default(),
                                                     r.get::<_, Option<String>>(2)?.unwrap_or_default(),
                                                     r.get::<_, Option<String>>(3)?.unwrap_or_default())))?.collect())
        .map_err(|e| format!("design_node: {e}"))?;
    let mut out = Vec::with_capacity(raw.len());
    for (id, group, release, content) in raw {
        let x: Value = serde_json::from_str(&content).map_err(|e| format!("node {id}: its content is not JSON ({e})"))?;
        let b = &x["body"];
        let rows = b["content"].as_array().map(|a| a.iter().map(|r| (s(&r[0]), s(&r[1]), s(&r[2]), s(&r[3]))).collect()).unwrap_or_default();
        let blocks = b["block"].as_array().cloned().unwrap_or_default();
        let behaviour = blocks.first().map(|blk| s(&blk[3])).unwrap_or_default();
        let built_in = blocks.iter().any(|blk| blk[3].as_str() == Some("built-in"));
        out.push(Node { id, group, release, rows, behaviour, built_in, content });
    }
    Ok(out)
}

/// A text the design keeps whole, by the path its node's origin names: the text, the node, the whole origin.
#[derive(Clone, Debug)]
pub struct Held {
    pub text: String,
    pub node: String,
    pub origin: String,
}

/// {path: held} of every file the design keeps whole (a lookup block's table, a method's pseudocode and its notes,
/// a data block's module), by the path its origin names; the first node to name a path holds it (from_design.py `_held`).
fn held(ns: &[Node]) -> BTreeMap<String, Held> {
    let mut out = BTreeMap::new();
    for n in ns {
        for (sec, field, value, origin) in &n.rows {
            if origin.is_empty() {
                continue;
            }
            let src = origin.split(' ').next().unwrap_or("");
            let kept = (sec == "table" && src.contains('/')) || sec == "data"
                || (sec == "code" && field == "pseudocode") || (sec == "explain" && field == "theory");
            if kept && HELD_PREFIXES.iter().any(|p| src.starts_with(p)) && !value.is_empty() {
                out.entry(src.to_string()).or_insert_with(|| Held { text: value.clone(), node: n.id.clone(), origin: origin.clone() });
            }
        }
    }
    out
}

/// (module path, targets, uses) of every method block that names a target, by node id (engine_build.py `blocks`).
fn blocks(ns_by_id: &[Node]) -> Result<Vec<(String, Vec<String>, Vec<String>)>, String> {
    let mut out = Vec::new();
    for n in ns_by_id {
        let mut rows: BTreeMap<(&str, &str), (&str, &str)> = BTreeMap::new();
        for (sec, field, value, origin) in &n.rows {
            rows.insert((sec.as_str(), field.as_str()), (value.as_str(), origin.as_str()));
        }
        let (Some(gen), Some(pc)) = (rows.get(&("code", "generate")), rows.get(&("code", "pseudocode"))) else { continue };
        let path = pc.1.split(' ').next().unwrap_or("").to_string();
        let targets = gen.0.split(',').map(str::trim).filter(|t| !t.is_empty()).map(String::from).collect();
        let uses = match rows.get(&("code", "uses")) {
            Some((u, _)) => serde_json::from_str::<Vec<String>>(u).map_err(|e| format!("node {}: code.uses is not a list of paths ({e})", n.id))?,
            None => Vec::new(),
        };
        out.push((path, targets, uses));
    }
    Ok(out)
}

fn module_text(path: &str, held: &BTreeMap<String, Held>) -> Result<String, String> {
    if path.starts_with("fsw/pseudocode/01") {
        return TOOLBOX.iter().find(|(p, _)| *p == path).map(|(_, t)| t.to_string())
            .ok_or_else(|| format!("the toolbox module {path} is not this program's toolbox"));
    }
    held.get(path).map(|h| h.text.clone()).ok_or_else(|| format!("the design holds no {path}"))
}

fn join_beside(user: &str, used: &str) -> String {
    // a module named bare (the flight algorithms' own uses, "01_math.pc") is its user's neighbour
    if used.contains('/') {
        return used.to_string();
    }
    match user.rfind('/') {
        Some(i) => format!("{}/{used}", &user[..i]),
        None => used.to_string(),
    }
}

/// {path: text} of every module a target takes: the blocks that name it and, in turn, what they use.
fn modules(target: &str, bs: &[(String, Vec<String>, Vec<String>)], held: &BTreeMap<String, Held>) -> Result<BTreeMap<String, String>, String> {
    let mut uses: BTreeMap<&str, &Vec<String>> = BTreeMap::new();
    for (p, _g, u) in bs {
        uses.insert(p.as_str(), u);
    }
    let mut todo: std::collections::VecDeque<String> = bs.iter().filter(|(_, g, _)| g.iter().any(|t| t == target)).map(|(p, _, _)| p.clone()).collect();
    if todo.is_empty() {
        return Err(format!("the design names no method for {target}"));
    }
    let mut got = BTreeMap::new();
    while let Some(p) = todo.pop_front() {
        if got.contains_key(&p) {
            continue;
        }
        got.insert(p.clone(), module_text(&p, held)?);
        if let Some(u) = uses.get(p.as_str()) {
            for q in u.iter() {
                todo.push_back(join_beside(&p, q));
            }
        }
    }
    Ok(got)
}

/// The kind and name a top-level line declares (`fn name(`, `const NAME = ...`), as engine_build.py's DECL.
fn decl(line: &str) -> Option<(&str, &str)> {
    let kind = DECL_KINDS.iter().find(|k| line.starts_with(**k))?;
    let rest = &line[kind.len()..];
    let name_at = rest.len() - rest.trim_start().len();
    if name_at == 0 {
        return None;
    }
    let rest = &rest[name_at..];
    let first = rest.chars().next()?;
    if !(first.is_ascii_alphabetic() || first == '_') {
        return None;
    }
    let end = rest.find(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_')).unwrap_or(rest.len());
    Some((kind, &rest[..end]))
}

/// (the module's head, [(name, the item's text with the comments above it)]) of a module's top-level items
/// (engine_build.py `_items`).
fn items(text: &str) -> Result<(Vec<String>, Vec<(String, String)>), String> {
    let lines: Vec<&str> = text.trim_end_matches('\n').split('\n').collect();
    let (mut head, mut items, mut notes): (Vec<String>, Vec<(String, String)>, Vec<String>) = (Vec::new(), Vec::new(), Vec::new());
    let mut i = 0;
    while i < lines.len() {
        let ln = lines[i];
        if ln.starts_with("##") {
            notes.push(ln.to_string());
        } else if let Some((kind, name)) = decl(ln) {
            let mut body = vec![ln.to_string()];
            if kind == "const" || kind == "choice" {
                while body.last().map(|b| b.trim_end().ends_with('\\')).unwrap_or(false) && i + 1 < lines.len() {
                    i += 1;
                    body.push(lines[i].to_string());
                }
            } else {
                while i + 1 < lines.len() && lines[i] != "end" {
                    i += 1;
                    body.push(lines[i].to_string());
                }
            }
            let mut t = std::mem::take(&mut notes);
            t.extend(body);
            items.push((name.to_string(), t.join("\n")));
        } else if !ln.trim().is_empty() {
            if !items.is_empty() {
                return Err(format!("a top-level line the relations cannot place: {ln:?}"));
            }
            head.append(&mut notes);
            head.push(ln.to_string());
        } else if !notes.is_empty() && items.is_empty() {
            head.append(&mut notes);
        }
        i += 1;
    }
    Ok((head, items))
}

fn module_of(head: &[String]) -> Option<String> {
    head.iter().find(|x| x.starts_with("module ")).and_then(|x| x.split_whitespace().nth(1)).map(String::from)
}

fn same_item(a: &str, b: &str) -> bool {
    // how an item is drawn (`## inputs from:`) is its package's vectors', not the relation
    let strip = |t: &str| t.split('\n').filter(|x| !x.starts_with("## inputs from:")).collect::<Vec<_>>().join("\n");
    strip(a) == strip(b)
}

/// ([(design path, text)] the relations are translated from, in order): the library's modules whole, then each
/// group's module without the items the library holds word for word (engine_build.py `relations_sources`).
fn relations_sources(held: &BTreeMap<String, Held>, groups: &Path) -> Result<Vec<(String, String)>, String> {
    let lib: Vec<(String, String)> = held.iter().filter(|(p, _)| p.starts_with(LIBRARY)).map(|(p, h)| (p.clone(), h.text.clone())).collect();
    if lib.is_empty() {
        return Err(format!("the design holds no relations under {LIBRARY}"));
    }
    let mut known: BTreeMap<String, (String, String)> = BTreeMap::new();
    for (p, t) in &lib {
        let (head, its) = items(t).map_err(|e| format!("{p}: {e}"))?;
        let m = module_of(&head).ok_or_else(|| format!("{p}: no module line"))?;
        for (name, text) in its {
            known.insert(name, (m.clone(), text));
        }
    }
    let mut out = lib.clone();
    let mut files: Vec<PathBuf> = std::fs::read_dir(groups).map_err(|e| format!("{}: {e}", groups.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_file() && p.extension().and_then(|x| x.to_str()) == Some("pc"))
        .collect();
    files.sort();
    for f in files {
        let t = std::fs::read_to_string(&f).map_err(|e| format!("{}: {e}", f.display()))?;
        let name_of = f.file_name().and_then(|x| x.to_str()).unwrap_or("").to_string();
        let (head, its) = items(&t).map_err(|e| format!("{}: {e}", f.display()))?;
        let mut kept = Vec::new();
        for (name, text) in its {
            match known.get(&name) {
                None => kept.push(text),
                Some((_, lt)) if same_item(lt, &text) => {}
                Some((lm, _)) => return Err(format!("{name} is written one way in {LIBRARY}{lm}.pc and another in design/groups/{name_of}")),
            }
        }
        out.push((format!("design/groups/{name_of}"), format!("{}\n\n{}\n", head.join("\n"), kept.join("\n\n"))));
    }
    Ok(out)
}

fn translate_matlab(sources: &[(String, String)], pkg: &str) -> Result<Vec<(String, String)>, String> {
    let src: Vec<(&str, &str)> = sources.iter().map(|(p, t)| (p.as_str(), t.as_str())).collect();
    let prog = trinetra_pcode::compile(&src).map_err(|errs| errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n"))?;
    prog.to_matlab_files(Some(pkg)).map_err(|e| format!("cannot translate: {e}"))
}

/// The flight algorithms' sources: the toolbox and the design's `fsw/pseudocode/02`..`09` blocks, by file name.
fn flight_sources(held: &BTreeMap<String, Held>) -> Vec<(String, String)> {
    let mut out: BTreeMap<String, String> = TOOLBOX.iter().map(|(p, t)| (p.to_string(), t.to_string())).collect();
    for (p, h) in held {
        let Some(name) = p.strip_prefix("fsw/pseudocode/") else { continue };
        if name.contains('/') || !name.ends_with(".pc") {
            continue;
        }
        if ["02", "03", "04", "05", "06", "07", "08", "09"].iter().any(|k| name.starts_with(k)) {
            out.insert(p.clone(), h.text.clone());
        }
    }
    out.into_iter().collect()
}

/// The algorithms' identity (flight_build.py `alg_id`): sha256 (first 16 hex) over the generated C and Rust sources.
fn alg_id(sources: &[(String, String)]) -> Result<String, String> {
    let src: Vec<(&str, &str)> = sources.iter().map(|(p, t)| (p.as_str(), t.as_str())).collect();
    let prog = trinetra_pcode::compile(&src).map_err(|errs| errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n"))?;
    let mut alg: BTreeMap<String, String> = BTreeMap::new();
    for (rel, t) in prog.to_c_files_with(Some(ALG_TITLE), Some(ALG_LIB), false)? {
        alg.insert(format!("fsw/alg/{rel}"), t);
    }
    let ro = trinetra_pcode::RustOptions { title: Some(ALG_TITLE), root: Some("crate::alg"), math: Some("crate::m"), dispatch: false };
    for (rel, t) in prog.to_rust_files_with(&ro)? {
        let base = rel.rsplit('/').next().unwrap_or(&rel).to_string();
        alg.insert(format!("fsw-rs/src/alg/{base}"), t);
    }
    let mut h = Sha256::new();
    for (p, t) in &alg {
        h.update(p.as_bytes());
        h.update(b"\0");
        h.update(t.as_bytes());
        h.update(b"\0");
    }
    Ok(h.finalize().iter().map(|x| format!("{x:02x}")).collect::<String>()[..16].to_string())
}

fn alg_id_m(aid: &str) -> String {
    format!("function id = alg_id()\n%ALG_ID  The flight algorithms' identity (the C and Rust sources' sha256, first 16 hex), written by\n\
             %   tools/flight_build.py; do not edit. This package is the MATLAB translation of the same design.\n    id = '{aid}';\nend\n")
}

/// The design's content hash (from_design.py `fingerprint`): sha256 over its inputs' fingerprint and every node's
/// content, by id. Any change to a node, a case or an input changes it; the file's layout on disk does not.
fn content_fingerprint(m: &BTreeMap<String, String>, ns_by_id: &[Node]) -> String {
    let mut h = Sha256::new();
    h.update(format!("inputs {}\n", m.get("inputs_fingerprint").map(String::as_str).unwrap_or("")).as_bytes());
    for n in ns_by_id {
        h.update(format!("node {} {}\n", n.id, sha_hex(n.content.as_bytes())).as_bytes());
    }
    h.finalize().iter().map(|x| format!("{x:02x}")).collect()
}

/// What the design is and how it stands, for `trinetra.open` to show.
pub fn health(path: &Path) -> Result<Value, String> {
    let c = connect(path)?;
    let m = meta(&c)?;
    let ns = nodes(&c, true)?;
    let mut beh: BTreeMap<String, u64> = BTreeMap::new();
    let mut built: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for n in &ns {
        *beh.entry(if n.behaviour.is_empty() { "none".into() } else { n.behaviour.clone() }).or_default() += 1;
        if n.built_in {
            built.entry(n.group.clone()).or_default().push(n.id.clone());
        }
    }
    let groups: BTreeMap<String, String> = c.prepare(r#"SELECT "id", "version" FROM design_group ORDER BY "id""#)
        .and_then(|mut s| s.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?.unwrap_or_default())))?.collect())
        .unwrap_or_default();
    let release: Option<String> = c.query_row(r#"SELECT "version" FROM design_release ORDER BY "built_at" DESC LIMIT 1"#, [], |r| r.get(0)).ok();
    let signatures: i64 = c.query_row(r#"SELECT count(*) FROM key_signature"#, [], |r| r.get(0)).unwrap_or(0);
    // every input held to its fingerprint (a problem is named, never passed over)
    let (mut n_inputs, mut bad) = (0, Vec::new());
    let rows: Vec<(String, String, Vec<u8>)> = c.prepare(r#"SELECT "path", "fingerprint", "body" FROM engine_input ORDER BY "path""#)
        .and_then(|mut s| s.query_map([], |r| Ok((r.get(0)?, r.get::<_, Option<String>>(1)?.unwrap_or_default(),
                                                    r.get::<_, Option<Vec<u8>>>(2)?.unwrap_or_default())))?.collect())
        .map_err(|e| e.to_string())?;
    let mut by_dir: BTreeMap<String, u64> = BTreeMap::new();
    for (k, fp, body) in &rows {
        n_inputs += 1;
        if fnv_hex(body) != *fp {
            bad.push(k.clone());
        }
        let parts: Vec<&str> = k.split('/').collect();
        if parts.len() > 2 {
            *by_dir.entry(parts[1].to_string()).or_default() += 1;
        }
    }
    let cases: Vec<String> = c.prepare(r#"SELECT DISTINCT "case_id" FROM design_case ORDER BY "case_id""#)
        .and_then(|mut s| s.query_map([], |r| r.get::<_, String>(0))?.collect()).unwrap_or_default();
    let methods = blocks(&ns)?.len();
    let parse = |k: &str| m.get(k).and_then(|v| serde_json::from_str::<Value>(v).ok()).unwrap_or(Value::Null);
    let version = m.get("design_version").filter(|v| !v.is_empty()).cloned().or(release).unwrap_or_else(|| m.get("id").cloned().unwrap_or_default());
    Ok(json!({
        "file": path.display().to_string(),
        "id": m.get("id"), "kind": m.get("design_kind"), "version": version,
        "design_version": m.get("design_version").filter(|v| !v.is_empty()),
        "format_version": m.get("format_version"), "written_by": m.get("written_by"),
        "toolbox": m.get("toolbox"), "needs_application": m.get("needs_application"), "library": env!("CARGO_PKG_VERSION"),
        "fingerprint": content_fingerprint(&m, &ns), "inputs_fingerprint": m.get("inputs_fingerprint"),
        "nodes": ns.len(), "behaviours": beh,
        "built_in": {"count": built.values().map(Vec::len).sum::<usize>(), "by_group": built},
        "generated_methods": methods,
        "groups": groups, "signatures": signatures,
        "inputs": {"count": n_inputs, "by_folder": by_dir, "not_their_fingerprint": bad},
        "cases": cases,
        "refused": parse("refused"), "cycles_refused": parse("cycles_refused"),
    }))
}

/// Where a generated file came from: its module's design path and the node that holds it.
fn provenance(rel: &str, decls: &BTreeMap<(String, String), String>, mods: &BTreeMap<String, Vec<String>>,
              held: &BTreeMap<String, Held>, info: &BTreeMap<String, (String, String)>, wired: &BTreeMap<String, String>) -> Value {
    // rel: +asils/+models/+facets/facets_box.m
    let parts: Vec<&str> = rel.split('/').collect();
    let file = parts.last().copied().unwrap_or("");
    let func = file.strip_suffix(".m").unwrap_or(file);
    let pkg: Vec<String> = parts[..parts.len() - 1].iter().map(|p| p.trim_start_matches('+').to_string()).collect();
    let mut e = Map::new();
    e.insert("file".into(), json!(rel));
    e.insert("name".into(), json!(func));      // not "function": a MATLAB keyword, which jsondecode renames
    e.insert("package".into(), json!(pkg.join(".")));
    if rel.starts_with(RUNTIME) {
        e.insert("from".into(), json!("the language's runtime (trinetra-pcode), code"));
        return Value::Object(e);
    }
    if rel == format!("{}/alg_id.m", ALG.0) {
        e.insert("from".into(), json!("the flight algorithms' identity: sha256 over their C and Rust translations (tools/flight_build.py)"));
        return Value::Object(e);
    }
    if parts.len() < 4 {
        e.insert("from".into(), json!("the translator's dispatcher, code"));
        return Value::Object(e);
    }
    let module = pkg.last().cloned().unwrap_or_default();
    let base = func.strip_suffix("_zero").unwrap_or(func);
    let path = decls.get(&(module.clone(), func.to_string())).or_else(|| decls.get(&(module.clone(), base.to_string())))
        .cloned().or_else(|| mods.get(&module).and_then(|v| v.first().cloned()));
    e.insert("module".into(), json!(module));
    if let Some(p) = path {
        e.insert("source".into(), json!(p));
        if let Some(h) = held.get(&p) {
            e.insert("node".into(), json!(h.node));
            if let Some((g, r)) = info.get(&h.node) {
                e.insert("group".into(), json!(g));
                e.insert("release".into(), json!(r));
            }
            e.insert("origin".into(), json!(h.origin));
            let rev = h.origin.find("revision ").map(|i| {
                let t = &h.origin[i + "revision ".len()..];
                t[..t.find([':', ')', ' ']).unwrap_or(t.len())].to_string()
            });
            e.insert("revision".into(), json!(rev.unwrap_or_default()));
        } else if TOOLBOX.iter().any(|(t, _)| *t == p) {
            e.insert("from".into(), json!("the toolbox, code"));
        } else if p.starts_with("design/groups/") {
            let node = wired.get(&format!("{module}::{func}")).or_else(|| wired.get(&format!("{module}::{base}")));
            if let Some(n) = node {
                e.insert("node".into(), json!(n));
                if let Some((g, r)) = info.get(n) {
                    e.insert("group".into(), json!(g));
                    e.insert("release".into(), json!(r));
                }
            }
            e.insert("from".into(), json!("the groups' wiring (tools/groupcode.py wire), not the design file"));
        }
    }
    Value::Object(e)
}

/// What a build made.
pub struct Built {
    pub files: Vec<(String, String)>,
    pub index: Value,
}

/// Every MATLAB function the twin flies, from the design: {path under the generated folder: text}, and the index.
/// `groups`: the groups' wiring (design/groups, tools/groupcode.py), which the relations package takes beside the
/// design's library; without it the relations are not built (the index says so).
pub fn build_matlab(path: &Path, groups: Option<&Path>) -> Result<Built, String> {
    let c = connect(path)?;
    let m = meta(&c)?;
    let ns_table = nodes(&c, false)?;
    let ns_id = nodes(&c, true)?;
    let held = held(&ns_table);
    let bs = blocks(&ns_id)?;
    let info: BTreeMap<String, (String, String)> = ns_id.iter().map(|n| (n.id.clone(), (n.group.clone(), n.release.clone()))).collect();
    let mut files: Vec<(String, String)> = Vec::new();
    let mut packages = Map::new();
    let mut skipped = Vec::new();
    let mut all_sources: Vec<(String, String)> = Vec::new();
    // the engine's models, every target's modules once
    let mut twin: BTreeMap<String, String> = BTreeMap::new();
    for t in MODEL_TARGETS {
        twin.extend(modules(t, &bs, &held)?);
    }
    let twin: Vec<(String, String)> = twin.into_iter().collect();
    let made = translate_matlab(&twin, MODELS.1).map_err(|e| format!("the translator refused the engine's models:\n{e}"))?;
    packages.insert(MODELS.1.into(), json!(made.len()));
    files.extend(made.into_iter().map(|(r, t)| (format!("{}/{r}", MODELS.0), t)));
    all_sources.extend(twin);
    // the relations: the design's library and the groups' wiring, each relation once
    let mut wired: BTreeMap<String, String> = BTreeMap::new();
    match groups {
        Some(g) => {
            let srcs = relations_sources(&held, g)?;
            let made = translate_matlab(&srcs, RELATIONS.1).map_err(|e| format!("the translator refused the relations:\n{e}"))?;
            packages.insert(RELATIONS.1.into(), json!(made.len()));
            files.extend(made.into_iter().map(|(r, t)| (format!("{}/{r}", RELATIONS.0), t)));
            all_sources.extend(srcs);
            if let Ok(rd) = std::fs::read_dir(g) {
                for f in rd.flatten().map(|e| e.path()).filter(|p| p.to_string_lossy().ends_with(".wire.json")) {
                    let Ok(w) = std::fs::read_to_string(&f).map_err(|_| ()).and_then(|t| serde_json::from_str::<Value>(&t).map_err(|_| ())) else { continue };
                    let gid = s(&w["group"]);
                    let shared: BTreeSet<String> = w["shared"].as_array().map(|a| a.iter().map(s).collect()).unwrap_or_default();
                    for r in w["rows"].as_array().cloned().unwrap_or_default() {
                        let f = s(&r["fn"]);
                        if f.is_empty() {
                            continue;
                        }
                        let m = if shared.contains(&f) { "shared".to_string() } else { gid.clone() };
                        wired.entry(format!("{m}::{f}")).or_insert_with(|| s(&r["id"]));
                    }
                }
            }
        }
        None => skipped.push(json!({"package": RELATIONS.1, "why": "the groups' wiring (design/groups, tools/groupcode.py wire) is not in the design file: give --groups DIR"})),
    }
    // the flight software's algorithms, and their identity
    let fsw = flight_sources(&held);
    let made = translate_matlab(&fsw, ALG.1).map_err(|e| format!("the translator refused the flight algorithms:\n{e}"))?;
    let aid = alg_id(&fsw).map_err(|e| format!("the flight algorithms' C and Rust: {e}"))?;
    packages.insert(ALG.1.into(), json!(made.len() + 1));
    files.extend(made.into_iter().map(|(r, t)| (format!("{}/{r}", ALG.0), t)));
    files.push((format!("{}/alg_id.m", ALG.0), alg_id_m(&aid)));
    all_sources.extend(fsw);
    // the language's runtime every package shares
    let rt = trinetra_pcode::matlab_runtime_files();
    packages.insert("asils.pc".into(), json!(rt.len()));
    files.extend(rt.into_iter().map(|(r, t)| (format!("{RUNTIME}/{r}"), t)));
    files.sort();
    // where each file came from
    let mut decls: BTreeMap<(String, String), String> = BTreeMap::new();
    let mut mods: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (p, t) in &all_sources {
        let mut module = String::new();
        for ln in t.split('\n') {
            if let Some(x) = ln.strip_prefix("module ") {
                module = x.split_whitespace().next().unwrap_or("").to_string();
                mods.entry(module.clone()).or_default().push(p.clone());
            } else if let Some((_, name)) = decl(ln) {
                decls.entry((module.clone(), name.to_string())).or_insert_with(|| p.clone());
            }
        }
    }
    let entries: Vec<Value> = files.iter().map(|(rel, _)| provenance(rel, &decls, &mods, &held, &info, &wired)).collect();
    let index = json!({
        "generated_by": MADE_BY, "library": env!("CARGO_PKG_VERSION"),
        "design": path.display().to_string(), "fingerprint": content_fingerprint(&m, &ns_id),
        "inputs_fingerprint": m.get("inputs_fingerprint"), "toolbox": m.get("toolbox"),
        "alg_id": aid, "packages": packages, "skipped": skipped,
        "groups": groups.map(|g| g.display().to_string()),
        "files": entries,
    });
    Ok(Built { files, index })
}

/// Write a build into `out` (made, or a folder this command made before, emptied first): every file read-only.
/// A folder that holds anything else is refused, so a build never deletes a person's files.
pub fn write_build(out: &Path, b: &Built) -> Result<(), String> {
    let o = out.display();
    if out.exists() {
        if !out.is_dir() {
            return Err(format!("{o}: not a folder"));
        }
        let names: Vec<String> = std::fs::read_dir(out).map_err(|e| format!("{o}: {e}"))?
            .filter_map(|e| e.ok().map(|e| e.file_name().to_string_lossy().into_owned())).collect();
        if !names.is_empty() {
            let idx = std::fs::read_to_string(out.join(INDEX)).ok().and_then(|t| serde_json::from_str::<Value>(&t).ok());
            if idx.as_ref().map(|v| v["generated_by"] == MADE_BY) != Some(true) {
                return Err(format!("{o}: holds files this command did not make (no {INDEX} of {MADE_BY}); give an empty or new folder"));
            }
            if let Some(x) = names.iter().find(|n| !["+asils", INDEX].contains(&n.as_str())) {
                return Err(format!("{o}: holds {x}, which this command did not make; give an empty or new folder"));
            }
            remove_tree(&out.join("+asils"))?;
            writable(&out.join(INDEX));
            std::fs::remove_file(out.join(INDEX)).map_err(|e| format!("{o}/{INDEX}: {e}"))?;
        }
    }
    for (rel, text) in &b.files {
        let f = out.join(rel);
        if let Some(d) = f.parent() {
            std::fs::create_dir_all(d).map_err(|e| format!("{}: {e}", d.display()))?;
        }
        std::fs::write(&f, text).map_err(|e| format!("{}: {e}", f.display()))?;
        read_only(&f);
    }
    let f = out.join(INDEX);
    std::fs::write(&f, serde_json::to_string_pretty(&b.index).map_err(|e| e.to_string())? + "\n").map_err(|e| format!("{}: {e}", f.display()))?;
    read_only(&f);
    Ok(())
}

fn read_only(f: &Path) {
    if let Ok(md) = std::fs::metadata(f) {
        let mut p = md.permissions();
        p.set_readonly(true);
        let _ = std::fs::set_permissions(f, p);
    }
}

#[allow(clippy::permissions_set_readonly_false)]
fn writable(f: &Path) {
    if let Ok(md) = std::fs::metadata(f) {
        let mut p = md.permissions();
        p.set_readonly(false);
        let _ = std::fs::set_permissions(f, p);
    }
}

fn remove_tree(d: &Path) -> Result<(), String> {
    if !d.exists() {
        return Ok(());
    }
    for e in std::fs::read_dir(d).map_err(|e| format!("{}: {e}", d.display()))?.flatten() {
        let p = e.path();
        if p.is_dir() {
            remove_tree(&p)?;
        } else {
            writable(&p);
            std::fs::remove_file(&p).map_err(|e| format!("{}: {e}", p.display()))?;
        }
    }
    std::fs::remove_dir(d).map_err(|e| format!("{}: {e}", d.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_declaration_is_read_as_the_tools_read_it() {
        assert_eq!(decl("fn aero_torque(q: vec4) -> t: vec3"), Some(("fn", "aero_torque")));
        assert_eq!(decl("const  MU = 3"), Some(("const", "MU")));
        assert_eq!(decl("record Plant"), Some(("record", "Plant")));
        assert_eq!(decl("fnord x"), None);
        assert_eq!(decl("fn (x)"), None);
        assert_eq!(decl("  fn x()"), None);
    }

    #[test]
    fn a_modules_items_keep_their_notes_and_the_head_its_lines() {
        let t = "## about\nmodule env\n\n## a note\nfn f(x: real) -> y: real\n    y = x\nend\n\nconst K = 1 \\\n  + 2\n";
        let (head, its) = items(t).unwrap();
        assert_eq!(head, vec!["## about", "module env"]);
        assert_eq!(its[0], ("f".to_string(), "## a note\nfn f(x: real) -> y: real\n    y = x\nend".to_string()));
        assert_eq!(its[1], ("K".to_string(), "const K = 1 \\\n  + 2".to_string()));
        assert!(items("module m\nfn f()\nend\nstray").is_err());
    }

    #[test]
    fn a_bare_use_is_its_users_neighbour() {
        assert_eq!(join_beside("fsw/pseudocode/04_guidance.pc", "01_math.pc"), "fsw/pseudocode/01_math.pc");
        assert_eq!(join_beside("act/coil.pc", "env/field.pc"), "env/field.pc");
    }

    #[test]
    fn fnv_is_the_engines() {
        assert_eq!(fnv_hex(b""), "cbf29ce484222325");
        assert_eq!(fnv_hex(b"a"), "af63dc4c8601ec8c");
    }

    fn regression() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../tests/regression/design.tndb")
    }

    #[test]
    fn the_inputs_are_the_files_the_twin_reads() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../matlab_sils");
        let got = inputs(&regression(), Which::All).unwrap();
        assert!(got.len() > 100);
        for (k, b) in &got {
            assert_eq!(&std::fs::read(root.join(k)).unwrap(), b, "{k} differs from matlab_sils/{k}");
        }
        let cases = inputs(&regression(), Which::Cases).unwrap();
        assert!(cases.iter().all(|(k, _)| k.starts_with("cases/")) && !cases.is_empty());
        let eng = inputs(&regression(), Which::EngineInputs).unwrap();
        assert_eq!(eng.len() + cases.len(), got.len());
        let j: Value = serde_json::from_str(&inputs_json(&got).unwrap()).unwrap();
        assert_eq!(j.as_array().unwrap().len(), got.len());
    }

    #[test]
    fn the_build_is_the_committed_twin_packages() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let b = build_matlab(&regression(), Some(&root.join("design/groups"))).unwrap();
        let ms = root.join("matlab_sils");
        let mut n = 0;
        for (rel, t) in &b.files {
            assert_eq!(&std::fs::read_to_string(ms.join(rel)).unwrap_or_default(), t, "{rel} differs from matlab_sils/{rel}");
            n += 1;
        }
        for pkg in ["+asils/+models", "+asils/+relations", "+asils/+alg", "+asils/+pc"] {
            let mut have = Vec::new();
            fn walk(d: &Path, base: &Path, out: &mut Vec<String>) {
                for e in std::fs::read_dir(d).unwrap().flatten() {
                    let p = e.path();
                    if p.is_dir() { walk(&p, base, out) } else { out.push(p.strip_prefix(base).unwrap().to_string_lossy().replace('\\', "/")) }
                }
            }
            walk(&ms.join(pkg), &ms, &mut have);
            for h in have {
                assert!(b.files.iter().any(|(r, _)| *r == h), "{h} is committed but the build does not give it");
            }
        }
        assert!(n > 500, "{n} files");
        let idx = &b.index["files"];
        let aero: Vec<&Value> = idx.as_array().unwrap().iter().filter(|e| e["name"] == "facets_box").collect();
        assert!(aero.iter().any(|e| e["node"].as_str().is_some_and(|n| !n.is_empty())), "{aero:?}");
    }

    #[test]
    fn without_the_groups_wiring_the_relations_are_not_built_and_say_so() {
        let b = build_matlab(&regression(), None).unwrap();
        assert!(b.files.iter().all(|(r, _)| !r.starts_with(RELATIONS.0)));
        assert_eq!(b.index["skipped"][0]["package"], RELATIONS.1);
    }

    #[test]
    fn health_counts_the_nodes_and_holds_every_input_to_its_fingerprint() {
        let h = health(&regression()).unwrap();
        let n = h["nodes"].as_u64().unwrap();
        let sum: u64 = h["behaviours"].as_object().unwrap().values().map(|v| v.as_u64().unwrap()).sum();
        assert_eq!(n, sum);
        assert_eq!(h["built_in"]["count"], 0);
        assert!(h["inputs"]["not_their_fingerprint"].as_array().unwrap().is_empty());
        assert_eq!(h["fingerprint"].as_str().unwrap().len(), 64);
    }

    #[test]
    fn a_folder_with_other_files_is_never_emptied() {
        let d = std::env::temp_dir().join(format!("tndb_twin_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("mine.m"), "x").unwrap();
        let b = Built { files: vec![("+asils/+pc/a.m".into(), "a".into())], index: json!({"generated_by": MADE_BY}) };
        assert!(write_build(&d, &b).unwrap_err().contains("did not make"));
        std::fs::remove_file(d.join("mine.m")).unwrap();
        write_build(&d, &b).unwrap();
        assert!(std::fs::metadata(d.join("+asils/+pc/a.m")).unwrap().permissions().readonly());
        write_build(&d, &b).unwrap();      // a build of its own is replaced
        std::fs::write(d.join("other.txt"), "x").unwrap();
        assert!(write_build(&d, &b).unwrap_err().contains("other.txt"));
        writable(&d.join("+asils/+pc/a.m"));
        writable(&d.join(INDEX));
        std::fs::remove_dir_all(&d).unwrap();
    }
}
