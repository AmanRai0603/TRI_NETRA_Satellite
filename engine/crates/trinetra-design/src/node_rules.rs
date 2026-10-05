//! The node app's live checks (design/js/node_model.js) in the library: a node file read as the
//! node app reads it (`readDoc`: its kind, its content with the spec's values beneath what the
//! author wrote, its inputs, its answer and its test vectors), and every check the app shows while
//! the author types (`check`): the same codes, levels, steps and words, in the same order. The
//! pseudocode is checked by the language's own checker (`trinetra-pcode`), as `pcodeCheck` does.
//!
//! The catalogue is the node app's own, design/js/node_catalog.js (written by tools/node_catalog.py,
//! held current by `node_catalog.py --check`), compiled in as it is: nothing here can drift from it.
//!
//! A node's fields are JavaScript values in the app (text, numbers from the database, whatever JSON a
//! list holds), and the checks read them as JavaScript does: `String(v)`, `Number(v)`, `JSON.parse`,
//! `v.field` of a parsed list item. [`J`] is such a value, and the checks use it so a mistyped field
//! is judged as the app judges it. Where the app itself would stop with a TypeError (a list item that
//! is `null`, say), [`check`] answers [`Throw`].
//!
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use std::collections::HashMap;
use std::path::Path;
use std::sync::OnceLock;

use rusqlite::{types::ValueRef, Connection, OpenFlags};
use trinetra_pcode::js::{json_string, number_text};

use crate::FormatError;

// ------------------------------------------------------------------ JavaScript values

/// A JavaScript value as the node app holds it.
#[derive(Clone, Debug, PartialEq)]
pub enum J {
    Undef,
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<J>),
    /// an object: its own keys in the order they were made
    Obj(Vec<(String, J)>),
    /// a function every object has (`constructor`, `toString`, ...)
    Func,
}

/// The node app would stop here with a TypeError: what it would say.
#[derive(Clone, Debug, PartialEq)]
pub struct Throw(pub String);

/// What every JavaScript object answers to.
const OBJECT_PROTO: &[&str] = &[
    "constructor", "__defineGetter__", "__defineSetter__", "hasOwnProperty", "__lookupGetter__", "__lookupSetter__", "isPrototypeOf",
    "propertyIsEnumerable", "toString", "valueOf", "__proto__", "toLocaleString",
];

/// JavaScript's white space (String.prototype.trim, `\s`).
fn js_ws(c: char) -> bool {
    matches!(c, '\t' | '\n' | '\u{b}' | '\u{c}' | '\r' | ' ' | '\u{a0}' | '\u{1680}' | '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}' | '\u{202f}' | '\u{205f}' | '\u{3000}' | '\u{feff}')
}
fn js_trim(s: &str) -> &str {
    s.trim_matches(js_ws)
}

/// `Number(s)` of a string.
fn str_to_number(s: &str) -> f64 {
    let t = js_trim(s);
    if t.is_empty() {
        return 0.0;
    }
    match t {
        "Infinity" | "+Infinity" => return f64::INFINITY,
        "-Infinity" => return f64::NEG_INFINITY,
        _ => {}
    }
    let radix = |digits: &str, r: u32| -> f64 {
        if digits.is_empty() || !digits.chars().all(|c| c.is_digit(r)) {
            return f64::NAN;
        }
        digits.chars().fold(0.0, |a, c| a * r as f64 + c.to_digit(r).unwrap() as f64)
    };
    let lower = t.get(..2).map(str::to_ascii_lowercase);
    match lower.as_deref() {
        Some("0x") => return radix(&t[2..], 16),
        Some("0o") => return radix(&t[2..], 8),
        Some("0b") => return radix(&t[2..], 2),
        _ => {}
    }
    // [+-]? (digits [. digits?] | . digits) ([eE] [+-]? digits)?
    let b = t.as_bytes();
    let mut i = usize::from(matches!(b[0], b'+' | b'-'));
    let int = b[i..].iter().take_while(|c| c.is_ascii_digit()).count();
    i += int;
    let mut frac = 0;
    if b.get(i) == Some(&b'.') {
        i += 1;
        frac = b[i..].iter().take_while(|c| c.is_ascii_digit()).count();
        i += frac;
    }
    if int + frac == 0 {
        return f64::NAN;
    }
    if matches!(b.get(i), Some(b'e') | Some(b'E')) {
        i += 1;
        if matches!(b.get(i), Some(b'+') | Some(b'-')) {
            i += 1;
        }
        let e = b[i..].iter().take_while(|c| c.is_ascii_digit()).count();
        if e == 0 {
            return f64::NAN;
        }
        i += e;
    }
    if i != b.len() {
        return f64::NAN;
    }
    t.parse::<f64>().unwrap_or(f64::NAN)
}

/// An array index as JavaScript names one: "0", "17", never "01".
fn array_index(k: &str) -> Option<usize> {
    if k.is_empty() || (k.len() > 1 && k.starts_with('0')) || !k.bytes().all(|c| c.is_ascii_digit()) {
        return None;
    }
    k.parse::<u64>().ok().filter(|v| *v < u32::MAX as u64).map(|v| v as usize)
}

impl J {
    fn from_cell(v: ValueRef) -> J {
        match v {
            ValueRef::Null => J::Null,
            ValueRef::Integer(i) => J::Num(i as f64),
            ValueRef::Real(f) => J::Num(f),
            ValueRef::Text(t) => J::Str(String::from_utf8_lossy(t).into_owned()),
            // sql.js hands a blob as a Uint8Array, which reads as its bytes joined by commas
            ValueRef::Blob(b) => J::Arr(b.iter().map(|x| J::Num(*x as f64)).collect()),
        }
    }
    fn from_serde(v: &serde_json::Value) -> J {
        match v {
            serde_json::Value::Null => J::Null,
            serde_json::Value::Bool(b) => J::Bool(*b),
            serde_json::Value::Number(n) => J::Num(n.as_f64().unwrap_or(f64::NAN)),
            serde_json::Value::String(s) => J::Str(s.clone()),
            serde_json::Value::Array(a) => J::Arr(a.iter().map(J::from_serde).collect()),
            serde_json::Value::Object(o) => J::Obj(o.iter().map(|(k, v)| (k.clone(), J::from_serde(v))).collect()),
        }
    }
    fn s(x: &str) -> J {
        J::Str(x.to_string())
    }
    /// `String(v)`
    pub fn text(&self) -> String {
        match self {
            J::Undef => "undefined".into(),
            J::Null => "null".into(),
            J::Bool(b) => b.to_string(),
            J::Num(x) => number_text(*x),
            J::Str(s) => s.clone(),
            J::Arr(a) => a.iter().map(|x| if matches!(x, J::Undef | J::Null) { String::new() } else { x.text() }).collect::<Vec<_>>().join(","),
            J::Obj(_) => "[object Object]".into(),
            J::Func => "function () { [native code] }".into(),
        }
    }
    /// `Number(v)`
    pub fn number(&self) -> f64 {
        match self {
            J::Undef | J::Obj(_) | J::Func => f64::NAN,
            J::Null => 0.0,
            J::Bool(b) => f64::from(u8::from(*b)),
            J::Num(x) => *x,
            J::Str(s) => str_to_number(s),
            J::Arr(_) => str_to_number(&self.text()),
        }
    }
    fn truthy(&self) -> bool {
        match self {
            J::Undef | J::Null => false,
            J::Bool(b) => *b,
            J::Num(x) => *x != 0.0 && !x.is_nan(),
            J::Str(s) => !s.is_empty(),
            _ => true,
        }
    }
    /// `a === b` (two objects are never the same here: each was made apart)
    fn strict_eq(&self, o: &J) -> bool {
        match (self, o) {
            (J::Undef, J::Undef) | (J::Null, J::Null) => true,
            (J::Bool(a), J::Bool(b)) => a == b,
            (J::Num(a), J::Num(b)) => a == b,
            (J::Str(a), J::Str(b)) => a == b,
            _ => false,
        }
    }
    /// SameValueZero: `===`, but NaN is NaN (Set.has, Array.includes)
    fn same(&self, o: &J) -> bool {
        matches!((self, o), (J::Num(a), J::Num(b)) if a.is_nan() && b.is_nan()) || self.strict_eq(o)
    }
    fn is(&self, s: &str) -> bool {
        matches!(self, J::Str(x) if x == s)
    }
    fn as_str(&self) -> Option<&str> {
        match self {
            J::Str(s) => Some(s),
            _ => None,
        }
    }
    /// `v[key]`
    fn get(&self, key: &str) -> Result<J, Throw> {
        let proto = || if OBJECT_PROTO.contains(&key) { J::Func } else { J::Undef };
        Ok(match self {
            J::Undef | J::Null => return Err(Throw(format!("Cannot read properties of {} (reading '{key}')", self.text()))),
            J::Obj(kv) => kv.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone()).unwrap_or_else(proto),
            J::Arr(a) => match (key, array_index(key)) {
                ("length", _) => J::Num(a.len() as f64),
                (_, Some(i)) => a.get(i).cloned().unwrap_or(J::Undef),
                _ => proto(),
            },
            J::Str(s) => {
                let units: Vec<u16> = s.encode_utf16().collect();
                match (key, array_index(key)) {
                    ("length", _) => J::Num(units.len() as f64),
                    (_, Some(i)) => units.get(i).map(|u| J::Str(String::from_utf16_lossy(&[*u]))).unwrap_or(J::Undef),
                    _ => proto(),
                }
            }
            _ => proto(),
        })
    }
    /// an object's keys in JavaScript's order: array indices rising, then the rest as made
    fn ordered(kv: &[(String, J)]) -> Vec<&(String, J)> {
        let mut idx: Vec<&(String, J)> = kv.iter().filter(|(k, _)| array_index(k).is_some()).collect();
        idx.sort_by_key(|(k, _)| array_index(k).unwrap());
        idx.extend(kv.iter().filter(|(k, _)| array_index(k).is_none()));
        idx
    }
    /// `Object.entries(v)`
    fn entries(&self) -> Result<Vec<(String, J)>, Throw> {
        Ok(match self {
            J::Undef | J::Null => return Err(Throw("Cannot convert undefined or null to object".into())),
            J::Obj(kv) => J::ordered(kv).into_iter().cloned().collect(),
            J::Arr(a) => a.iter().enumerate().map(|(i, v)| (i.to_string(), v.clone())).collect(),
            J::Str(s) => s.encode_utf16().enumerate().map(|(i, u)| (i.to_string(), J::Str(String::from_utf16_lossy(&[u])))).collect(),
            _ => Vec::new(),
        })
    }
    /// `JSON.stringify(v)`; None where JavaScript gives undefined
    pub fn stringify(&self) -> Option<String> {
        Some(match self {
            J::Undef | J::Func => return None,
            J::Null => "null".into(),
            J::Bool(b) => b.to_string(),
            J::Num(x) => if x.is_finite() { number_text(*x) } else { "null".into() },
            J::Str(s) => json_string(s),
            J::Arr(a) => format!("[{}]", a.iter().map(|x| x.stringify().unwrap_or_else(|| "null".into())).collect::<Vec<_>>().join(",")),
            J::Obj(kv) => format!(
                "{{{}}}",
                J::ordered(kv).into_iter().filter_map(|(k, v)| v.stringify().map(|s| format!("{}:{s}", json_string(k)))).collect::<Vec<_>>().join(",")
            ),
        })
    }
}

/// `JSON.parse(text)`
pub fn json_parse(text: &str) -> Result<J, Throw> {
    struct P<'a> {
        s: &'a [u8],
        i: usize,
    }
    fn fail<T>(p: &P) -> Result<T, Throw> {
        Err(Throw(format!("JSON.parse: unexpected input at {}", p.i)))
    }
    impl P<'_> {
        fn ws(&mut self) {
            while matches!(self.s.get(self.i), Some(b' ' | b'\t' | b'\n' | b'\r')) {
                self.i += 1;
            }
        }
        fn lit(&mut self, w: &str, v: J) -> Result<J, Throw> {
            if self.s[self.i..].starts_with(w.as_bytes()) {
                self.i += w.len();
                Ok(v)
            } else {
                fail(self)
            }
        }
        fn value(&mut self) -> Result<J, Throw> {
            self.ws();
            match self.s.get(self.i) {
                Some(b'n') => self.lit("null", J::Null),
                Some(b't') => self.lit("true", J::Bool(true)),
                Some(b'f') => self.lit("false", J::Bool(false)),
                Some(b'"') => self.string().map(J::Str),
                Some(b'[') => {
                    self.i += 1;
                    let mut a = Vec::new();
                    self.ws();
                    if self.s.get(self.i) == Some(&b']') {
                        self.i += 1;
                        return Ok(J::Arr(a));
                    }
                    loop {
                        a.push(self.value()?);
                        self.ws();
                        match self.s.get(self.i) {
                            Some(b',') => self.i += 1,
                            Some(b']') => {
                                self.i += 1;
                                return Ok(J::Arr(a));
                            }
                            _ => return fail(self),
                        }
                    }
                }
                Some(b'{') => {
                    self.i += 1;
                    let mut kv: Vec<(String, J)> = Vec::new();
                    self.ws();
                    if self.s.get(self.i) == Some(&b'}') {
                        self.i += 1;
                        return Ok(J::Obj(kv));
                    }
                    loop {
                        self.ws();
                        if self.s.get(self.i) != Some(&b'"') {
                            return fail(self);
                        }
                        let k = self.string()?;
                        self.ws();
                        if self.s.get(self.i) != Some(&b':') {
                            return fail(self);
                        }
                        self.i += 1;
                        let v = self.value()?;
                        // a key given twice keeps its first place and its last value
                        match kv.iter_mut().find(|(x, _)| *x == k) {
                            Some(e) => e.1 = v,
                            None => kv.push((k, v)),
                        }
                        self.ws();
                        match self.s.get(self.i) {
                            Some(b',') => self.i += 1,
                            Some(b'}') => {
                                self.i += 1;
                                return Ok(J::Obj(kv));
                            }
                            _ => return fail(self),
                        }
                    }
                }
                Some(b'-' | b'0'..=b'9') => {
                    let st = self.i;
                    if self.s[self.i] == b'-' {
                        self.i += 1;
                    }
                    let digits = |p: &mut Self| {
                        let n = p.s[p.i..].iter().take_while(|c| c.is_ascii_digit()).count();
                        p.i += n;
                        n
                    };
                    match self.s.get(self.i) {
                        Some(b'0') => self.i += 1,
                        Some(b'1'..=b'9') => {
                            digits(self);
                        }
                        _ => return fail(self),
                    }
                    if self.s.get(self.i) == Some(&b'.') {
                        self.i += 1;
                        if digits(self) == 0 {
                            return fail(self);
                        }
                    }
                    if matches!(self.s.get(self.i), Some(b'e' | b'E')) {
                        self.i += 1;
                        if matches!(self.s.get(self.i), Some(b'+' | b'-')) {
                            self.i += 1;
                        }
                        if digits(self) == 0 {
                            return fail(self);
                        }
                    }
                    let t = std::str::from_utf8(&self.s[st..self.i]).unwrap();
                    Ok(J::Num(t.parse::<f64>().unwrap_or(f64::NAN)))
                }
                _ => fail(self),
            }
        }
        fn hex4(&mut self) -> Result<u16, Throw> {
            let h = self.s.get(self.i..self.i + 4).and_then(|b| std::str::from_utf8(b).ok()).and_then(|t| u16::from_str_radix(t, 16).ok());
            match h {
                Some(v) => {
                    self.i += 4;
                    Ok(v)
                }
                None => fail(self),
            }
        }
        fn string(&mut self) -> Result<String, Throw> {
            self.i += 1;
            let mut units: Vec<u16> = Vec::new();
            loop {
                let Some(&c) = self.s.get(self.i) else { return fail(self) };
                match c {
                    b'"' => {
                        self.i += 1;
                        return Ok(String::from_utf16_lossy(&units));
                    }
                    b'\\' => {
                        self.i += 1;
                        let e = self.s.get(self.i).copied();
                        self.i += 1;
                        match e {
                            Some(b'"') => units.push(b'"' as u16),
                            Some(b'\\') => units.push(b'\\' as u16),
                            Some(b'/') => units.push(b'/' as u16),
                            Some(b'b') => units.push(8),
                            Some(b'f') => units.push(12),
                            Some(b'n') => units.push(10),
                            Some(b'r') => units.push(13),
                            Some(b't') => units.push(9),
                            Some(b'u') => units.push(self.hex4()?),
                            _ => return fail(self),
                        }
                    }
                    0..=0x1f => return fail(self),
                    _ => {
                        // one UTF-8 character
                        let rest = std::str::from_utf8(&self.s[self.i..]).map_err(|_| Throw("JSON.parse: bad text".into()))?;
                        let ch = rest.chars().next().unwrap();
                        let mut b = [0u16; 2];
                        units.extend_from_slice(ch.encode_utf16(&mut b));
                        self.i += ch.len_utf8();
                    }
                }
            }
        }
    }
    let mut p = P { s: text.as_bytes(), i: 0 };
    let v = p.value()?;
    p.ws();
    if p.i != p.s.len() {
        return fail(&p);
    }
    Ok(v)
}

// ------------------------------------------------------------------ the catalogue

/// The node app's catalogue, design/js/node_catalog.js, compiled in.
pub const CATALOG_JS: &str = include_str!("../../../../design/js/node_catalog.js");

/// What a node may declare and read (tools/node_catalog.py).
#[derive(Debug)]
pub struct Catalog {
    /// unit name -> (symbol, the quantities it states)
    pub units: HashMap<String, (String, Vec<String>)>,
    pub quantities: Vec<String>,
    pub sources: Vec<String>,
    pub physics: Vec<String>,
    pub metrics: Vec<String>,
    pub rungs: Vec<String>,
    pub provenance: Vec<String>,
    pub areas: Vec<String>,
    pub belief_statuses: Vec<String>,
    pub tags: Vec<String>,
    /// tree id -> [label, spec kind, quantity, unit, layer]
    pub rows: HashMap<String, Vec<J>>,
}

/// The catalogue (read once).
pub fn catalog() -> &'static Catalog {
    static C: OnceLock<Catalog> = OnceLock::new();
    C.get_or_init(|| {
        let body = CATALOG_JS.split_once("export const CATALOG = ").expect("node_catalog.js: no CATALOG").1;
        let body = body.trim_end().trim_end_matches(';');
        let v: serde_json::Value = serde_json::from_str(body).expect("node_catalog.js: not JSON");
        let strs = |k: &str| -> Vec<String> { v[k].as_array().unwrap().iter().map(|x| x.as_str().unwrap().to_string()).collect() };
        let firsts = |k: &str| -> Vec<String> { v[k].as_array().unwrap().iter().map(|x| x[0].as_str().unwrap().to_string()).collect() };
        Catalog {
            units: v["units"]
                .as_array()
                .unwrap()
                .iter()
                .map(|u| (u[0].as_str().unwrap().to_string(), (u[1].as_str().unwrap_or("").to_string(), u[2].as_array().unwrap().iter().map(|q| q.as_str().unwrap().to_string()).collect())))
                .collect(),
            quantities: strs("quantities"),
            sources: firsts("sources"),
            physics: firsts("physics"),
            metrics: strs("metrics"),
            rungs: strs("rungs"),
            provenance: strs("provenance"),
            areas: strs("areas"),
            belief_statuses: strs("belief_statuses"),
            tags: strs("tags"),
            rows: v["rows"].as_object().unwrap().iter().map(|(k, r)| (k.clone(), r.as_array().unwrap().iter().map(J::from_serde).collect())).collect(),
        }
    })
}

fn in_list(list: &[String], v: &J) -> bool {
    v.as_str().is_some_and(|s| list.iter().any(|x| x == s))
}

// ------------------------------------------------------------------ the node, as the app reads it

/// An input of a computed node.
#[derive(Clone, Debug)]
pub struct Input {
    pub name: J,
    pub from_node: J,
    pub quantity: J,
    pub unit: J,
}

/// A test vector.
#[derive(Clone, Debug)]
pub struct Fixture {
    pub name: J,
    pub inputs: J,
    pub expected: Option<f64>,
    pub tolerance: J,
    pub provenance: J,
    pub source: J,
    pub where_: J,
    pub outside: bool,
}

/// A node file as the node app reads it (node_model.js `readDoc`).
#[derive(Clone, Debug)]
pub struct NodeDoc {
    /// the node row: id, sheet, group_id, stage, layer, kind, label, state, author, contract_version
    pub node: HashMap<&'static str, J>,
    /// every field, `section.field`, the spec's values beneath the author's
    pub content: HashMap<String, J>,
    pub inputs: Vec<Input>,
    pub fixtures: Vec<Fixture>,
    /// declared, computed, kpi, evidence, closure, interface, target; None for a row whose author
    /// has not said yet
    pub kind: Option<&'static str>,
}

const SPEC_ORIGIN_KEYS: [&str; 10] = ["id", "sheet", "group_id", "stage", "layer", "kind", "label", "state", "author", "contract_version"];

/// The kind a node is filled as (node_model.js `kindOf`).
pub fn kind_of(node: &HashMap<&'static str, J>, content: &HashMap<String, J>) -> Option<&'static str> {
    let k = match node.get("kind") {
        Some(v) if v.truthy() => v.text(),
        _ => String::new(),
    };
    if k == "interface" || k == "closure_interface" {
        return Some("interface");
    }
    if k.starts_with("closure") {
        return Some("closure");
    }
    if k == "required" || k == "achieved" {
        return Some("target");
    }
    let id = node.get("id").cloned().unwrap_or(J::Undef).text();
    if let Some(spec) = catalog().rows.get(&id) {
        return Some(match spec.get(1).and_then(J::as_str) {
            Some("declared") => "declared",
            Some("computed") => "computed",
            Some("required") => "kpi",
            Some("achieved") => "evidence",
            Some("door") => "interface",
            _ => "declared",
        });
    }
    match content.get("identity.form_kind") {
        Some(J::Str(s)) if s == "declared" => Some("declared"),
        Some(J::Str(s)) if s == "computed" => Some("computed"),
        _ => None,
    }
}

fn query(conn: &Connection, sql: &str) -> Result<Vec<Vec<J>>, String> {
    let mut st = conn.prepare(sql).map_err(|e| e.to_string())?;
    let n = st.column_count();
    let rows = st
        .query_map([], |r| (0..n).map(|i| r.get_ref(i).map(J::from_cell)).collect::<Result<Vec<_>, _>>())
        .and_then(|it| it.collect::<Result<Vec<_>, _>>())
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

/// `JSON.parse(v)`, v made text as JavaScript makes it
fn parse_text(v: &J) -> Result<J, Throw> {
    json_parse(&v.text())
}

/// `v || "[]"` and friends: v when truthy, else the default
fn or<'a>(v: &'a J, d: &'a J) -> &'a J {
    if v.truthy() {
        v
    } else {
        d
    }
}

/// A node file, read as the node app reads it. `Err` where the app's reading would throw.
pub fn read_doc(conn: &Connection) -> Result<NodeDoc, Throw> {
    let q = |sql: &str| query(conn, sql).map_err(Throw);
    let row = q("SELECT id, sheet, group_id, stage, layer, kind, label, state, author, contract_version FROM node")?.into_iter().next().unwrap_or_default();
    let node: HashMap<&'static str, J> = SPEC_ORIGIN_KEYS.iter().enumerate().map(|(i, k)| (*k, row.get(i).cloned().unwrap_or(J::Null))).collect();
    let mut content: HashMap<String, J> = HashMap::new();
    for r in q("SELECT section, field, value, origin FROM content ORDER BY rowid")? {
        content.insert(format!("{}.{}", r[0].text(), r[1].text()), r[2].clone());
    }
    let mut inputs: Vec<Input> = q("SELECT name, from_node, from_output, unit FROM input ORDER BY rowid")?
        .into_iter()
        .map(|r| Input { name: r[0].clone(), from_node: r[1].clone(), quantity: r[2].clone(), unit: r[3].clone() })
        .collect();
    let output = q("SELECT name, unit, lower, upper, reason_lower, reason_upper FROM output ORDER BY rowid")?.into_iter().next();
    let mut fixtures = Vec::new();
    for r in q("SELECT name, inputs, expected, tolerance, source, outside FROM fixture ORDER BY rowid")? {
        let meta = parse_text(or(&r[4], &J::s("{}"))).unwrap_or_else(|_| J::Obj(vec![("source".into(), r[4].clone())]));
        let ins = parse_text(or(&r[1], &J::s("{}"))).unwrap_or(J::Obj(Vec::new()));
        let pick = |k: &str| -> Result<J, Throw> {
            let v = meta.get(k)?;
            Ok(if v.truthy() { v } else { J::s("") })
        };
        let expected = if matches!(r[2], J::Null) || r[2].is("") { None } else { Some(r[2].number()) };
        fixtures.push(Fixture {
            name: r[0].clone(),
            inputs: ins,
            expected,
            tolerance: r[3].clone(),
            provenance: pick("provenance")?,
            source: pick("source")?,
            where_: pick("where")?,
            outside: r[5].truthy(),
        });
    }
    // the output table holds the answer: it fills what content does not say (`??=`)
    if let Some(o) = &output {
        let mut fill = |k: &str, v: J| {
            let e = content.entry(k.to_string()).or_insert(J::Undef);
            if matches!(e, J::Undef | J::Null) {
                *e = v;
            }
        };
        fill("output.symbol", o[0].clone());
        fill("output.unit", o[1].clone());
        for (i, k) in [(2, "output.lower"), (3, "output.upper")] {
            if !matches!(o[i], J::Null) {
                fill(k, J::Str(o[i].text()));
            }
        }
        for (i, k) in [(4, "output.reason_lower"), (5, "output.reason_upper")] {
            if !matches!(o[i], J::Null) {
                fill(k, o[i].clone());
            }
        }
    }
    let kind = kind_of(&node, &content);
    let (spec, spec_inputs) = spec_fields(&content, kind)?;
    if let Some(si) = spec_inputs {
        if inputs.is_empty() {
            inputs = si;
        }
    }
    for (k, v) in spec {
        content.entry(k).or_insert(v);
    }
    Ok(NodeDoc { node, content, inputs, fixtures, kind })
}

/// What the spec says about the node, in the app's fields (node_model.js `specFields`), with the
/// inputs it names.
#[allow(clippy::type_complexity)]
fn spec_fields(c: &HashMap<String, J>, kind: Option<&'static str>) -> Result<(Vec<(String, J)>, Option<Vec<Input>>), Throw> {
    let mut out: Vec<(String, J)> = Vec::new();
    let mut inputs = None;
    let sp = |f: &str| c.get(&format!("spec.{f}")).cloned().unwrap_or(J::Undef);
    let mut put = |k: &str, v: J| {
        if !matches!(v, J::Undef | J::Null) && !v.is("") {
            match out.iter_mut().find(|(x, _)| x == k) {
                Some(e) => e.1 = v,
                None => out.push((k.to_string(), v)),
            }
        }
    };
    put("identity.question", sp("question"));
    put("identity.note", sp("note"));
    put("output.symbol", sp("symbol"));
    put("output.quantity", sp("type"));
    put("output.unit", sp("unit"));
    for x in ["lower", "upper", "reason_lower", "reason_upper"] {
        let v = sp(x);
        put(&format!("output.{x}"), if matches!(v, J::Undef) { J::Undef } else { J::Str(v.text().strip_suffix(".0").map(str::to_string).unwrap_or_else(|| v.text())) });
    }
    match kind {
        Some("computed") => {
            put("relation.expression", sp("expression"));
            put("relation.source", sp("source"));
            put("relation.why", sp("why"));
            if sp("steps").truthy() {
                put("relation.derivation", sp("steps"));
            }
            // a seed without (readable) inputs has none
            if let Ok(J::Arr(ins)) = parse_text(or(&sp("inputs"), &J::s("[]"))) {
                if !ins.is_empty() {
                    let mut got = Vec::new();
                    let mut ok = true;
                    for x in &ins {
                        // `([name, from_node]) =>`: an array or a string gives its first two; anything else throws
                        let (name, from) = match x {
                            J::Arr(a) => (a.first().cloned().unwrap_or(J::Undef), a.get(1).cloned().unwrap_or(J::Undef)),
                            // a string gives its first two characters (code points, as a string iterates)
                            J::Str(s) => {
                                let mut ch = s.chars().map(|c| J::Str(c.to_string()));
                                (ch.next().unwrap_or(J::Undef), ch.next().unwrap_or(J::Undef))
                            }
                            _ => {
                                ok = false;
                                break;
                            }
                        };
                        let q = catalog().rows.get(&from.text()).and_then(|r| r.get(2)).cloned().filter(J::truthy).unwrap_or(J::Null);
                        got.push(Input { name, from_node: from, quantity: q, unit: J::Null });
                    }
                    if ok {
                        inputs = Some(got);
                    }
                }
            }
        }
        Some("declared") => {
            let v = sp("value");
            put("value.number", if matches!(v, J::Undef) { J::Undef } else { J::Str(v.text()) });
            put("value.source", sp("source"));
        }
        Some("kpi") => {
            let s = sp("sense");
            // an object's key is String(v)
            let sense = match s.text().as_str() {
                "<=" | "at_most" => J::s("at_most"),
                ">=" | "at_least" => J::s("at_least"),
                _ => J::Undef,
            };
            put("requirement.sense", sense);
            let v = sp("value");
            put("requirement.value", if matches!(v, J::Undef) { J::Undef } else { J::Str(v.text()) });
            if sp("source").truthy() {
                put("sources.cited", J::Str(J::Arr(vec![sp("source")]).stringify().unwrap()));
            }
        }
        _ => {}
    }
    if let Ok(a) = parse_text(or(&sp("assumptions"), &J::s("[]"))) {
        if a.get("length").map(|l| l.truthy()).unwrap_or(false) {
            if let J::Arr(items) = &a {
                let mapped: Vec<J> = items
                    .iter()
                    .map(|x| match x {
                        J::Arr(p) => J::Obj(vec![("assumes".into(), p.first().cloned().unwrap_or(J::Undef)), ("until".into(), p.get(1).cloned().unwrap_or(J::Undef))]),
                        x => x.clone(),
                    })
                    .collect();
                put("assumptions", J::Str(J::Arr(mapped).stringify().unwrap()));
            }
        }
    }
    if let Ok(ex) = parse_text(or(&sp("explain"), &J::s("{}"))) {
        if let Ok(es) = ex.entries() {
            for (k, v) in es {
                let t = match &v {
                    J::Str(_) => v.clone(),
                    v => v.stringify().map(J::Str).unwrap_or(J::Undef),
                };
                put(&format!("explain.{k}"), t);
            }
        }
    }
    Ok((out, inputs))
}

/// node_model.js `list`: a field's JSON list, or none.
fn list(doc: &NodeDoc, key: &str) -> Vec<J> {
    match parse_text(or(doc.content.get(key).unwrap_or(&J::Undef), &J::s("[]"))) {
        Ok(J::Arr(a)) => a,
        _ => Vec::new(),
    }
}

// ------------------------------------------------------------------ the design around the node

/// What the design says of another node: its label, quantity, unit, layer, group and kind.
#[derive(Clone, Debug)]
pub struct OtherNode {
    pub label: J,
    pub group: J,
    pub layer: J,
    pub kind: J,
    pub quantity: J,
    pub unit: J,
}

/// What the checks know beyond the node (node_model.js `ctx`).
#[derive(Clone, Debug, Default)]
pub struct Context {
    /// the design's nodes by id; None when the design is not known (C02's "not a node of the
    /// design" and C03, C04 are then not checked)
    pub nodes: Option<HashMap<String, OtherNode>>,
}

impl Context {
    /// The design's nodes, from every node file of `dir/nodes`, as the node app builds them from
    /// the design (release.js `designContext`): each row's quantity and unit are the spec's.
    pub fn from_folder(dir: &Path) -> Result<Context, FormatError> {
        let nd = dir.join("nodes");
        let mut files: Vec<_> = std::fs::read_dir(&nd)
            .map_err(|e| FormatError(format!("{}: {e}", nd.display())))?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.to_string_lossy().ends_with(".node.tndb"))
            .collect();
        files.sort();
        let mut nodes = HashMap::new();
        for f in files {
            let conn = Connection::open_with_flags(&f, OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(|e| FormatError(format!("{}: {e}", f.display())))?;
            for r in query(&conn, "SELECT id, label, group_id, layer, kind FROM node").map_err(|e| FormatError(format!("{}: {e}", f.display())))? {
                let spec = catalog().rows.get(&r[0].text());
                let at = |i: usize| spec.map(|s| s.get(i).cloned().unwrap_or(J::Undef)).unwrap_or(J::Null);
                nodes.insert(r[0].text(), OtherNode { label: r[1].clone(), group: r[2].clone(), layer: r[3].clone(), kind: r[4].clone(), quantity: at(2), unit: at(3) });
            }
        }
        Ok(Context { nodes: Some(nodes) })
    }
}

// ------------------------------------------------------------------ the checks

/// One problem: its rule, its level (`!` to fix, `i` to know), the step of the app it belongs to,
/// and what the app says.
#[derive(Clone, Debug, PartialEq)]
pub struct Problem {
    pub code: &'static str,
    pub level: char,
    pub step: &'static str,
    pub text: String,
}

fn filled(v: &J) -> bool {
    if matches!(v, J::Undef | J::Null) {
        return false;
    }
    let t = v.text();
    let t = js_trim(&t);
    !t.is_empty() && t != "[]"
}
fn is_num(v: &J) -> bool {
    filled(v) && v.number().is_finite()
}
/// /^[A-Za-z_][A-Za-z0-9_]*$/.test(String(v))
fn ident(v: &J) -> bool {
    let t = v.text();
    let mut c = t.chars();
    c.next().is_some_and(|f| f.is_ascii_alphabetic() || f == '_') && c.all(|x| x.is_ascii_alphanumeric() || x == '_')
}

const FIXED: [&str; 3] = ["closure", "interface", "target"];

const MATH_NAMES: &[&str] = &[
    "sqrt", "sin", "cos", "tan", "asin", "acos", "atan", "atan2", "exp", "log", "log10", "abs", "min", "max", "pi", "e", "norm", "dot", "cross", "sum",
    "pow", "floor", "ceil", "round", "sign", "hypot", "clamp", "if", "then", "else", "and", "or", "not",
];

/// Every problem the node app shows for the node, in its order (node_model.js `check`).
pub fn check(doc: &NodeDoc, ctx: &Context) -> Result<Vec<Problem>, Throw> {
    let cat = catalog();
    let c = |k: &str| doc.content.get(k).cloned().unwrap_or(J::Undef);
    let mut out: Vec<Problem> = Vec::new();
    // a problem to fix (!), and one to know (i), each in its place
    macro_rules! bad {
        ($code:expr, $step:expr, $text:expr) => {
            out.push(Problem { code: $code, level: '!', step: $step, text: $text.into() })
        };
    }
    macro_rules! warn {
        ($code:expr, $step:expr, $text:expr) => {
            out.push(Problem { code: $code, level: 'i', step: $step, text: $text.into() })
        };
    }
    let Some(k) = doc.kind else {
        bad!("N02", "identity", "Say what kind of node this is: declared or computed");
        return Ok(out);
    };
    // the explanation (X01–X03), for every kind
    if filled(&c("explain.analogy")) && !filled(&c("explain.analogy_breaks")) {
        bad!("X01", "explain", "The analogy says where it stops being true");
    }
    if filled(&c("explain.wrong_idea")) && !filled(&c("explain.wrong_because")) {
        bad!("X02", "explain", "The common wrong idea says why it is wrong");
    }
    if k == "computed" && !(filled(&c("explain.simply")) && filled(&c("explain.one_line"))) {
        warn!("X03", "explain", "Say it simply and in one line: without them its page serves experts only");
    }
    if FIXED.contains(&k) {
        if filled(&c("other.subject")) != filled(&c("other.description")) {
            bad!("B02", "feedback", "Something else needs both a subject and a description");
        }
        if filled(&c("feedback.expected")) && !filled(&c("feedback.what_happened")) {
            bad!("B01", "feedback", "Feedback says what happened");
        }
        return Ok(out);
    }
    if !filled(&c("identity.question")) {
        bad!("Q01", "identity", "Say the question the node answers");
    }
    if !filled(&c("explain.where_it_breaks")) {
        bad!("E20", "explain", "Say where the answer stops being true (the explanation standard)");
    }
    // its answer (O01–O04)
    let (sym, q, u) = (c("output.symbol"), c("output.quantity"), c("output.unit"));
    if !filled(&sym) {
        bad!("O01", "io", "Give the answer a symbol");
    } else if !ident(&sym) {
        bad!("O01", "io", format!("The symbol {} has to be letters, digits and _", sym.text()));
    }
    if !filled(&q) {
        bad!("O02", "io", "Say what quantity the answer is");
    } else if !in_list(&cat.quantities, &q) {
        bad!("O02", "io", format!("{} is not a quantity the software knows", q.text()));
    }
    let unit = u.as_str().and_then(|s| cat.units.get(s));
    if !filled(&u) {
        bad!("O03", "io", "Give the answer's unit");
    } else if let Some((_, qs)) = unit {
        if filled(&q) && !in_list(qs, &q) {
            bad!("O03", "io", format!("{} does not state {} (it states {})", u.text(), q.text(), qs.join(", ")));
        }
    } else {
        bad!("O03", "io", format!("{} is not a unit the software knows", u.text()));
    }
    for (side, word) in [("lower", "lowest"), ("upper", "highest")] {
        let v = c(&format!("output.{side}"));
        if filled(&v) && !is_num(&v) {
            bad!("O04", "io", format!("The {word} value is not a number"));
        }
        if filled(&v) && !filled(&c(&format!("output.reason_{side}"))) {
            bad!("O04", "io", format!("Say why the {word} value is what it is"));
        }
    }
    if is_num(&c("output.lower")) && is_num(&c("output.upper")) && c("output.lower").number() > c("output.upper").number() {
        bad!("O04", "io", "The lowest value is above the highest");
    }
    // inputs (C01–C04)
    if k == "computed" {
        if doc.inputs.is_empty() {
            bad!("C01", "io", "A computed node reads at least one input");
        }
        let mut seen: Vec<J> = Vec::new();
        let id = doc.node.get("id").cloned().unwrap_or(J::Undef);
        let my_layer = doc.node.get("layer").cloned().unwrap_or(J::Undef);
        for i in &doc.inputs {
            if !filled(&i.name) || !ident(&i.name) {
                bad!("C02", "io", format!("An input needs a binding of letters, digits and _ ({})", if i.name.truthy() { i.name.text() } else { "empty".into() }));
            } else if seen.iter().any(|s| s.same(&i.name)) {
                bad!("C02", "io", format!("The binding {} is used twice", i.name.text()));
            }
            seen.push(i.name.clone());
            if i.from_node.strict_eq(&id) {
                bad!("C02", "io", "A node cannot read itself");
            }
            let p = ctx.nodes.as_ref().and_then(|m| i.from_node.as_str().and_then(|s| m.get(s)));
            if ctx.nodes.is_some() && p.is_none() {
                bad!("C02", "io", format!("{} is not a node of the design", if i.from_node.truthy() { i.from_node.text() } else { "(empty)".into() }));
            }
            if let Some(p) = p {
                if p.quantity.truthy() && i.quantity.truthy() && !p.quantity.strict_eq(&i.quantity) {
                    bad!("C03", "io", format!("{} publishes {}, not {}", i.from_node.text(), p.quantity.text(), i.quantity.text()));
                }
                let pk = if p.kind.truthy() { p.kind.text() } else { String::new() };
                if p.layer.truthy() && my_layer.truthy() && p.layer.text() != my_layer.text() && !pk.contains("interface") {
                    warn!("C04", "io", format!("{} is in layer {}; layers meet only at the door and the interface rows", i.from_node.text(), p.layer.text()));
                }
            }
        }
    } else if !doc.inputs.is_empty() {
        bad!("C01", "io", "Only a computed node reads inputs");
    }
    // the relation (R01, R07, R04)
    if k == "computed" {
        for (key, what) in [("relation.expression", "the relation"), ("relation.source", "its source"), ("relation.why", "why it is this relation")] {
            if !filled(&c(key)) {
                bad!("R01", "theory", format!("Give {what}"));
            }
        }
        for (n, s) in list(doc, "relation.derivation").iter().enumerate() {
            if !filled(s) {
                bad!("R07", "theory", format!("Derivation step {} has no text", n + 1));
            }
        }
        let expr = c("relation.expression");
        if filled(&expr) {
            let text = expr.text();
            let mut unknown: Vec<String> = Vec::new();
            let b = text.as_bytes();
            let mut i = 0;
            while i < b.len() {
                if b[i].is_ascii_alphabetic() || b[i] == b'_' {
                    let st = i;
                    while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                        i += 1;
                    }
                    let n = &text[st..i];
                    let named = sym.is(n) || doc.inputs.iter().any(|x| x.name.is(n));
                    if !named && !MATH_NAMES.contains(&n) && !cat.physics.iter().any(|p| p == n) && !unknown.iter().any(|x| x == n) {
                        unknown.push(n.to_string());
                    }
                } else {
                    i += 1;
                }
            }
            if !unknown.is_empty() {
                warn!("R04", "theory", format!("The relation uses names that are not its inputs or its answer: {}", unknown.join(", ")));
            }
        }
    }
    for (n, a) in list(doc, "assumptions").iter().enumerate() {
        if !filled(&a.get("assumes")?) || !filled(&a.get("until")?) {
            bad!("R06", "theory", format!("Assumption {} says what it assumes and when that stops being true", n + 1));
        }
    }
    // the pseudocode (R03 through the language's own checker)
    if k == "computed" {
        let pc = c("code.pseudocode");
        if !filled(&pc) {
            bad!("P01", "pseudocode", "Write the relation in pseudocode");
        } else {
            for e in pcode_check(&pc.text(), &sym, &doc.inputs)? {
                bad!("P02", "pseudocode", e);
            }
        }
    }
    // value, requirement, evidence (V01–V03)
    if k == "declared" {
        let v = c("value.number");
        if !is_num(&v) {
            bad!("V01", "value", "Give the value as a number");
        } else {
            let v = v.number();
            if is_num(&c("output.lower")) && v < c("output.lower").number() {
                bad!("V01", "value", "The value is below the lowest value");
            }
            if is_num(&c("output.upper")) && v > c("output.upper").number() {
                bad!("V01", "value", "The value is above the highest value");
            }
        }
        if !filled(&c("value.source")) {
            bad!("V01", "value", "Say where the value comes from");
        }
    }
    if k == "kpi" && !(c("requirement.sense").is("at_most") || c("requirement.sense").is("at_least")) {
        bad!("V02", "value", "Say which way the requirement binds: at most or at least");
    }
    if k == "evidence" {
        if !in_list(&cat.metrics, &c("evidence.metric")) {
            bad!("V03", "value", "Name the metric the simulator computes");
        }
        if list(doc, "evidence.rungs").is_empty() {
            bad!("V03", "value", "Name at least one rung (SILS, PIL, OILS, HILS)");
        }
    }
    // sources (S01)
    let new_src = list(doc, "sources.new");
    for (n, s) in new_src.iter().enumerate() {
        if !filled(&s.get("id")?) || !filled(&s.get("title")?) || !filled(&s.get("where")?) {
            bad!("S01", "evidence", format!("New source {} needs an id, a title and exactly where", n + 1));
        }
    }
    let mut known: Vec<J> = cat.sources.iter().map(|s| J::s(s)).collect();
    for s in &new_src {
        known.push(s.get("id")?);
    }
    for key in ["relation.source", "value.source"] {
        let v = c(key);
        if !filled(&v) {
            continue;
        }
        let text = v.text();
        // String(v).split(/[,;]\s*/).map(trim).filter(Boolean)
        let mut parts: Vec<&str> = Vec::new();
        let mut rest = text.as_str();
        while let Some(at) = rest.find([',', ';']) {
            parts.push(&rest[..at]);
            rest = rest[at + 1..].trim_start_matches(js_ws);
        }
        parts.push(rest);
        for id in parts.into_iter().map(js_trim).filter(|s| !s.is_empty()) {
            let lower_id = id.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');
            if !known.iter().any(|x| x.is(id)) && lower_id {
                let step = if key.starts_with("relation") { "theory" } else { "value" };
                bad!("S01", step, format!("The source {id} is not known: add it under Evidence, New sources"));
            }
        }
    }
    // test vectors (T01–T04)
    if k == "computed" {
        if doc.fixtures.is_empty() {
            warn!("T04", "evidence", "No test vector: its validation stays low");
        }
        for (n, x) in doc.fixtures.iter().enumerate() {
            let name = format!("Test vector {}", n + 1);
            if !in_list(&cat.provenance, &x.provenance) {
                bad!("T01", "evidence", format!("{name}: its answer comes from {}; never from the code itself", cat.provenance.join(", ")));
            }
            if !filled(&x.source) || !filled(&x.where_) {
                bad!("T02", "evidence", format!("{name}: cite the source and the page, table or figure"));
            }
            for i in &doc.inputs {
                if !is_num(&x.inputs.get(&i.name.text())?) {
                    bad!("T03", "evidence", format!("{name}: give a number for {}", i.name.text()));
                }
            }
            if !x.expected.is_some_and(f64::is_finite) {
                bad!("T03", "evidence", format!("{name}: give the expected answer as a number"));
            }
            if x.tolerance.number().partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater) {
                bad!("T03", "evidence", format!("{name}: give a tolerance above zero"));
            }
        }
    }
    // results
    if let Some(problems) = results(doc) {
        for p in problems {
            bad!("Y01", "results", p);
        }
    }
    // the belief record (D01, D03, D07, D06)
    for (key, what) in [("belief.area", "its area"), ("belief.believed", "what was believed"), ("belief.now_know", "what we now know"), ("belief.plan_change", "what changed in the plan")] {
        if !filled(&c(key)) {
            bad!("D01", "belief", format!("The belief record needs {what}"));
        }
    }
    let status = c("belief.status");
    if filled(&status) {
        if !in_list(&cat.belief_statuses, &status) {
            bad!("D03", "belief", "The status is broke, held or untested");
        }
        if !filled(&c("belief.tested")) {
            bad!("D03", "belief", if status.is("untested") { "Say the test that would settle it" } else { "Say what tested it" });
        }
        if status.is("untested") {
            warn!("D07", "belief", "The belief is untested: it is counted under \"Beliefs not yet tested\"");
        }
    } else {
        bad!("D03", "belief", "Say whether a test broke the belief, held it, or none has been run");
    }
    let cost = c("belief.cost_k");
    if filled(&cost) && !(is_num(&cost) && cost.number() >= 0.0) {
        bad!("D06", "belief", "The cost is a number, zero or more, in thousands of dollars");
    }
    Ok(out)
}

/// The pseudocode, checked with the language's own checker (node_model.js `pcodeCheck`): its problems.
pub fn pcode_check(text: &str, symbol: &J, inputs: &[Input]) -> Result<Vec<String>, Throw> {
    let program = match trinetra_pcode::compile(&[("node.pc", text)]) {
        Ok(p) => p,
        Err(errs) => {
            if let Some(e) = errs.iter().find(|e| e.kind == trinetra_pcode::ErrorKind::Internal) {
                return Err(Throw(e.message.clone()));
            }
            return Ok(errs.iter().map(|e| format!("line {}: {}", if e.pos.line == 0 { "undefined".into() } else { e.pos.line.to_string() }, e.message)).collect());
        }
    };
    let mut problems = Vec::new();
    let fns = program.functions();
    let f = fns.iter().find(|f| f.outputs.iter().any(|o| symbol.is(o.name)));
    if fns.is_empty() {
        problems.push("The pseudocode has no fn".to_string());
    } else if symbol.truthy() && f.is_none() {
        problems.push(format!("No fn has the answer's symbol {} as an output (R03)", symbol.text()));
    }
    if let Some(f) = f {
        for i in inputs {
            if i.name.truthy() && !f.inputs.iter().any(|p| i.name.is(p.name)) {
                problems.push(format!("The fn {} has no input {}, which the node reads", f.name, i.name.text()));
            }
        }
    }
    Ok(problems)
}

/// The results table's problems (node_model.js `results`): None when there is no table.
fn results(doc: &NodeDoc) -> Option<Vec<String>> {
    let tv = doc.content.get("results.table").cloned().unwrap_or(J::Undef);
    if !filled(&tv) {
        return None;
    }
    let read = || -> Result<Vec<String>, Throw> {
        let t = parse_text(&tv)?;
        let mut problems = Vec::new();
        let J::Arr(rows) = t.get("rows")? else { return Err(Throw("t.rows.forEach is not a function".into())) };
        for (i, r) in rows.iter().enumerate() {
            let J::Arr(cells) = r else { return Err(Throw("r.forEach is not a function".into())) };
            for (j, v) in cells.iter().enumerate() {
                if v.is("") || matches!(v, J::Null) || v.number().is_finite() {
                    continue;
                }
                let numeric = t.get("numeric")?;
                if !numeric.truthy() || !numeric.get(&j.to_string())?.truthy() {
                    continue;
                }
                let name = t.get("columns")?.get(&j.to_string())?.get("name")?;
                problems.push(format!("row {}, {}: {} is not a number", i + 1, name.text(), v.text()));
            }
        }
        Ok(problems)
    };
    Some(read().unwrap_or_else(|_| vec!["The results could not be read; paste them again".into()]))
}

/// A node file, read and checked: what `tndb check-node` prints. `Err` names a file the node app
/// could not read or check.
pub fn check_file(path: &Path, ctx: &Context) -> Result<Vec<Problem>, String> {
    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(|e| format!("{}: {e}", path.display()))?;
    let doc = read_doc(&conn).map_err(|e| format!("{}: the node app cannot read it: {}", path.display(), e.0))?;
    check(&doc, ctx).map_err(|e| format!("{}: the node app's checks stop: {}", path.display(), e.0))
}
