//! Signatures on files, checked through the chain (docs/OPERATING_2_0.md §14; rules R01, R02).
//!
//! - [`registry`] reads who may sign from a group file (its `member_key` table): the programme
//!   file registers the system engineer, the subsystem engineers and the deputies; each group file
//!   registers its node engineers and checkers.
//! - [`sign_file`] adds a signature to a file's `key_signature` table: the content hash of what the
//!   file holds now, signed with the person's key.
//! - [`verify_file`] checks every signature on a file: the signer is registered in the role they
//!   signed as (R01, one writer per file: only the assigned writer's role may sign it), the key is
//!   the registered one, the signature is theirs (R02), and the content is still what was signed.
//!
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use crate::keys::{check_chain, fingerprint, sign, KeyError, Registered, Statement};
use crate::{content, open, write, Cell, DesignFile, FormatError, Schema};
use ed25519_dalek::SigningKey;
use rusqlite::Connection;
use std::path::Path;

fn text(c: &Cell) -> String {
    match c { Cell::Text(t) => t.clone(), Cell::Integer(i) => i.to_string(), Cell::Real(r) => r.to_string(), _ => String::new() }
}

/// Who a group file registers, from its `member_key` table.
pub fn registry(group: &DesignFile, s: &Schema) -> Result<Vec<Registered>, FormatError> {
    Ok(group.rows("member_key", s)?.iter().map(|r| Registered {
        name: text(&r[0]), role: text(&r[1]), public_key: text(&r[2]),
        deputy_of: Some(text(&r[4])).filter(|d| !d.is_empty()),
    }).collect())
}

/// Who signs, as what, and what they say: everything of a signature but the content it covers.
#[derive(Debug, Clone)]
pub struct Signing<'a> {
    pub signer: &'a str,
    pub role: &'a str,
    /// the file's revision the person signs (0 for a file that keeps none)
    pub revision: i64,
    pub at: &'a str,
    pub statement: &'a str,
}

/// Sign what a file holds now: a row in its `key_signature` table.
pub fn sign_file(path: &Path, key: &SigningKey, w: &Signing, s: &Schema) -> Result<Statement, FormatError> {
    let f = open(path, None, s)?;
    let revision = w.revision;
    let st = Statement { role: w.role.into(), signer: w.signer.into(), revision, content_hash: content::content_hash(&f, s)?, at: w.at.into(), statement: w.statement.into() };
    let n = f.rows("key_signature", s)?.len() as i64 + 1;
    drop(f);
    let c = Connection::open(path).map_err(|e| FormatError(format!("{}: {e}", path.display())))?;
    write::insert(&c, "key_signature", &[vec![Cell::Integer(n), Cell::Text(st.role.clone()), Cell::Text(st.signer.clone()),
        Cell::Text(fingerprint(&key.verifying_key().to_bytes())), Cell::Integer(revision), Cell::Text(st.content_hash.clone()),
        Cell::Text(st.at.clone()), Cell::Text(st.statement.clone()), Cell::Text(sign(key, &st))]], s)?;
    Ok(st)
}

/// One signature's verdict.
#[derive(Debug, PartialEq)]
pub struct Verdict {
    pub n: i64,
    pub signer: String,
    pub role: String,
    /// None: it holds; Some(why): it does not
    pub problem: Option<String>,
}

/// Check every signature on a file against a registry. A signature over content the file no longer
/// holds is named as such: the file changed after it was signed.
pub fn verify_file(f: &DesignFile, reg: &[Registered], s: &Schema) -> Result<Vec<Verdict>, FormatError> {
    let now = content::content_hash(f, s)?;
    Ok(f.rows("key_signature", s)?.iter().map(|r| {
        let n = match r[0] { Cell::Integer(i) => i, _ => 0 };
        let st = Statement { role: text(&r[1]), signer: text(&r[2]), revision: match r[4] { Cell::Integer(i) => i, _ => 0 },
            content_hash: text(&r[5]), at: text(&r[6]), statement: text(&r[7]) };
        let problem = match check_chain(reg, &st, &text(&r[8]), &text(&r[3])) {
            Err(KeyError(e)) => Some(e),
            Ok(_) if st.content_hash != now => Some(format!("{} signed revision {}; the file has changed since (its content is not what was signed)", st.signer, st.revision)),
            Ok(_) => None,
        };
        Verdict { n, signer: st.signer, role: st.role, problem }
    }).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose::STANDARD as B64, Engine};

    #[test]
    fn a_node_signed_by_its_registered_engineer_holds_until_it_changes() {
        let s = Schema::embedded().unwrap();
        let d = std::env::temp_dir().join(format!("chain-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        let (asha, ravi, kiran) = (SigningKey::from_bytes(&[1; 32]), SigningKey::from_bytes(&[2; 32]), SigningKey::from_bytes(&[3; 32]));
        let pk = |k: &SigningKey| B64.encode(k.verifying_key().to_bytes());
        // the group file registers its node engineer and its subsystem engineer
        let g = d.join("env.group.tndb");
        write::create(&g, "group", "env", &s, |c| write::insert(c, "member_key", &[
            vec![Cell::Text("Asha".into()), Cell::Text("node engineer".into()), Cell::Text(pk(&asha)), Cell::Text(fingerprint(&asha.verifying_key().to_bytes())), Cell::Null, Cell::Text("2026-10-06".into()), Cell::Text("Ravi".into())],
            vec![Cell::Text("Ravi".into()), Cell::Text("subsystem engineer".into()), Cell::Text(pk(&ravi)), Cell::Text(fingerprint(&ravi.verifying_key().to_bytes())), Cell::Null, Cell::Text("2026-10-06".into()), Cell::Text("programme manager".into())],
        ], &s)).unwrap();
        let reg = registry(&open(&g, Some("group"), &s).unwrap(), &s).unwrap();
        assert_eq!(reg.len(), 2);
        // a node, signed by its engineer
        let n = d.join("gd_1.node.tndb");
        write::create(&n, "node", "gd_1", &s, |c| write::insert(c, "output", &[vec![Cell::Text("tau".into()), Cell::Text("N m".into()), Cell::Real(0.0), Cell::Real(1e-3), Cell::Null, Cell::Null]], &s)).unwrap();
        sign_file(&n, &asha, &Signing { signer: "Asha", role: "node engineer", revision: 3, at: "2026-10-06T10:00:00Z", statement: "the day's work" }, &s).unwrap();
        let v = verify_file(&open(&n, None, &s).unwrap(), &reg, &s).unwrap();
        assert_eq!(v, vec![Verdict { n: 1, signer: "Asha".into(), role: "node engineer".into(), problem: None }]);
        // someone not registered signs: named; the engineer's own signature still holds
        sign_file(&n, &kiran, &Signing { signer: "Kiran", role: "node engineer", revision: 3, at: "2026-10-06T10:05:00Z", statement: "me too" }, &s).unwrap();
        let v = verify_file(&open(&n, None, &s).unwrap(), &reg, &s).unwrap();
        assert!(v[0].problem.is_none());
        assert!(v[1].problem.as_ref().unwrap().contains("Kiran is not registered"));
        // the node engineer may not seal as the subsystem engineer
        let n2 = d.join("gd_2.node.tndb");
        write::create(&n2, "node", "gd_2", &s, |_| Ok(())).unwrap();
        sign_file(&n2, &asha, &Signing { signer: "Asha", role: "subsystem engineer", revision: 1, at: "t", statement: "sealed" }, &s).unwrap();
        assert!(verify_file(&open(&n2, None, &s).unwrap(), &reg, &s).unwrap()[0].problem.as_ref().unwrap().contains("registered as node engineer"));
        // the content changes after signing: the signature no longer holds, by name
        let c = Connection::open(&n).unwrap();
        c.execute("UPDATE output SET upper = 2e-3", []).unwrap();
        drop(c);
        let v = verify_file(&open(&n, None, &s).unwrap(), &reg, &s).unwrap();
        assert!(v[0].problem.as_ref().unwrap().contains("changed since"));
        let _ = std::fs::remove_dir_all(&d);
    }
}
