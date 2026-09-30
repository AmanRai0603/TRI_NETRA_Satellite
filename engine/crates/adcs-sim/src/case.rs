//! adcs-case/1 reader (asils.case.read): section,key,label,unit,value,lo,hi,level,note.
use crate::error::Error;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Clone, Debug, Default)]
pub struct Case { pub id: String, pub title: String, pub file: String, pub v: BTreeMap<String, f64> }

fn split(line: &str) -> Vec<String> {
    let (mut f, mut cur, mut inq) = (vec![], String::new(), false);
    let c: Vec<char> = line.chars().collect();
    let mut i = 0;
    while i < c.len() {
        let ch = c[i];
        if inq {
            if ch == '"' { if i + 1 < c.len() && c[i + 1] == '"' { cur.push('"'); i += 1; } else { inq = false; } }
            else { cur.push(ch); }
        } else if ch == '"' { inq = true; }
        else if ch == ',' { f.push(std::mem::take(&mut cur)); }
        else { cur.push(ch); }
        i += 1;
    }
    f.push(cur);
    f
}

/// A case value: blank is "not stated" (NaN, which the engine treats as absent);
/// anything else must be a finite number. A value that is neither is refused, never
/// read as blank.
pub fn value(s: &str) -> Result<f64, Error> {
    let s = s.trim();
    if s.is_empty() { return Ok(f64::NAN); }
    match s.parse::<f64>() {
        Ok(x) if x.is_finite() => Ok(x),
        _ => Err(Error::refused(format!("{s:?}: not a finite number (leave it blank if it is not stated)"))),
    }
}

impl Case {
    pub fn read(p: &Path) -> Result<Case, Error> {
        let txt = std::fs::read_to_string(p).map_err(|e| Error::io(p, format!("case: {e}")))?;
        let mut lines = txt.lines();
        let hdr = split(lines.next().unwrap_or(""));
        let need = ["section", "key", "label", "unit", "value", "lo", "hi", "level", "note"];
        if hdr.len() < 9 || hdr[..9].iter().zip(need).any(|(a, b)| a != b) {
            return Err(Error::refused(format!("case {}: header must be {}", p.display(), need.join(","))));
        }
        let mut c = Case { file: p.display().to_string(), ..Default::default() };
        let mut seen = std::collections::BTreeSet::new();
        for (n, l) in lines.enumerate() {
            let line = n + 2;
            if l.trim().is_empty() { continue; }
            let f = split(l.trim_end_matches('\r'));
            if f.len() < 5 { return Err(Error::refused(format!("case {} line {line}: {} field(s); a row has at least section,key,label,unit,value", p.display(), f.len()))); }
            let key = f[1].trim();
            if key.is_empty() { return Err(Error::refused(format!("case {} line {line}: the key is empty", p.display()))); }
            if !seen.insert(key.to_string()) { return Err(Error::refused(format!("case {} line {line}: {key} is given twice", p.display()))); }
            match key {
                "meta.schema" => if f[4] != "adcs-case/1" { return Err(Error::refused(format!("case {}: meta.schema must be adcs-case/1", p.display()))); },
                "meta.case_id" => c.id = f[4].clone(),
                "meta.title" => c.title = f[4].clone(),
                k if k.starts_with("meta.") => {}
                k => { c.v.insert(k.to_string(), value(&f[4]).map_err(|e| Error::refused(format!("case {} line {line}: {k} = {e}", p.display())))?); }
            }
        }
        if c.id.is_empty() { return Err(Error::refused(format!("case {} has no meta.case_id", p.display()))); }
        Ok(c)
    }
    /// A case value by its dotted key (NaN when blank or absent).
    pub fn get(&self, k: &str) -> f64 { self.v.get(k).copied().unwrap_or(f64::NAN) }
}

#[cfg(test)]
mod t {
    use super::*;
    fn read(body: &str) -> Result<Case, Error> {
        // one file per call: the tests run in parallel, so a name must never be shared
        static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let d = std::env::temp_dir().join(format!("adcs-case-test-{}-{n}", std::process::id()));
        std::fs::write(&d, format!("section,key,label,unit,value,lo,hi,level,note\nmeta,meta.schema,,,adcs-case/1,,,,\nmeta,meta.case_id,,,t,,,,\n{body}")).unwrap();
        let r = Case::read(&d);
        let _ = std::fs::remove_file(&d);
        r
    }
    #[test]
    fn a_blank_value_is_not_stated() { assert!(read("orbit,orbit.alt,,,,,,,\n").unwrap().get("orbit.alt").is_nan()); }
    #[test]
    fn a_number_is_read() { assert_eq!(read("orbit,orbit.alt,,,550,,,,\n").unwrap().get("orbit.alt"), 550.0); }
    #[test]
    fn text_where_a_number_belongs_is_refused() { assert!(read("orbit,orbit.alt,,,five hundred,,,,\n").unwrap_err().message().contains("orbit.alt")); }
    #[test]
    fn a_non_finite_value_is_refused() { for v in ["NaN", "inf", "-Infinity"] { assert!(read(&format!("orbit,orbit.alt,,,{v},,,,\n")).is_err(), "{v}"); } }
    #[test]
    fn a_key_given_twice_is_refused() { assert!(read("orbit,orbit.alt,,,550,,,,\norbit,orbit.alt,,,600,,,,\n").unwrap_err().message().contains("twice")); }
    #[test]
    fn a_short_row_is_refused() { assert!(read("orbit,orbit.alt\n").unwrap_err().message().contains("line 4")); }
}
