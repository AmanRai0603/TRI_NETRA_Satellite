//! Small accessors over serde_json values (asils.util.getf).
use serde_json::Value;
use std::path::Path;

pub fn read(p: &Path) -> Result<Value, crate::Error> {
    let s = std::fs::read_to_string(p).map_err(|e| crate::Error::io(p, e))?;
    serde_json::from_str(&s).map_err(|e| crate::Error::malformed(format!("{}: {e}", p.display())))
}
pub fn get<'a>(v: &'a Value, k: &str) -> Option<&'a Value> { v.get(k).filter(|x| !x.is_null()) }
pub fn f(v: &Value, k: &str, d: f64) -> f64 { get(v, k).and_then(|x| x.as_f64()).unwrap_or(d) }
pub fn s<'a>(v: &'a Value, k: &str, d: &'a str) -> &'a str { get(v, k).and_then(|x| x.as_str()).unwrap_or(d) }
pub fn b(v: &Value, k: &str, d: bool) -> bool {
    match get(v, k) { Some(Value::Bool(x)) => *x, Some(x) if x.is_number() => x.as_f64().unwrap_or(0.0) != 0.0, _ => d }
}
pub fn v3(v: &Value) -> Option<[f64; 3]> {
    let a = v.as_array()?;
    if a.len() != 3 { return None; }
    Some([a[0].as_f64()?, a[1].as_f64()?, a[2].as_f64()?])
}
/// A list of 3-vectors ([[..],[..]] or a single [x,y,z]).
pub fn vecs(v: &Value) -> Vec<[f64; 3]> {
    if let Some(x) = v3(v) { if v.as_array().map(|a| a[0].is_number()).unwrap_or(false) { return vec![x]; } }
    v.as_array().map(|a| a.iter().filter_map(v3).collect()).unwrap_or_default()
}
pub fn unit(a: [f64; 3]) -> [f64; 3] { let n = (a[0]*a[0] + a[1]*a[1] + a[2]*a[2]).sqrt(); [a[0]/n, a[1]/n, a[2]/n] }
/// Set a dotted path in a JSON object (creating objects), value parsed as JSON or kept as a string.
pub fn set_path(root: &mut Value, path: &str, val: &str) {
    let parsed: Value = serde_json::from_str(val).unwrap_or_else(|_| Value::String(val.to_string()));
    let mut cur = root;
    let parts: Vec<&str> = path.split('.').collect();
    for (i, p) in parts.iter().enumerate() {
        if !cur.is_object() { *cur = Value::Object(Default::default()); }
        let o = cur.as_object_mut().unwrap();
        if i + 1 == parts.len() { o.insert(p.to_string(), parsed); return; }
        cur = o.entry(p.to_string()).or_insert(Value::Object(Default::default()));
    }
}
