//! adcs-case/1 reader (asils.case.read): section,key,label,unit,value,lo,hi,level,note.
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

impl Case {
    pub fn read(p: &Path) -> Result<Case, String> {
        let txt = std::fs::read_to_string(p).map_err(|e| format!("case {}: {e}", p.display()))?;
        let mut lines = txt.lines();
        let hdr = split(lines.next().unwrap_or(""));
        let need = ["section", "key", "label", "unit", "value", "lo", "hi", "level", "note"];
        if hdr.len() < 9 || hdr[..9].iter().zip(need).any(|(a, b)| a != b) {
            return Err(format!("case {}: header must be {}", p.display(), need.join(",")));
        }
        let mut c = Case { file: p.display().to_string(), ..Default::default() };
        for l in lines {
            if l.trim().is_empty() { continue; }
            let f = split(l.trim_end_matches('\r'));
            if f.len() < 5 { continue; }
            match f[1].as_str() {
                "meta.schema" => if f[4] != "adcs-case/1" { return Err(format!("case {}: meta.schema must be adcs-case/1", p.display())); },
                "meta.case_id" => c.id = f[4].clone(),
                "meta.title" => c.title = f[4].clone(),
                k if k.starts_with("meta.") => {}
                k => { c.v.insert(k.to_string(), f[4].trim().parse().unwrap_or(f64::NAN)); }
            }
        }
        if c.id.is_empty() { return Err(format!("case {} has no meta.case_id", p.display())); }
        Ok(c)
    }
    /// A case value by its dotted key (NaN when blank or absent).
    pub fn get(&self, k: &str) -> f64 { self.v.get(k).copied().unwrap_or(f64::NAN) }
}
