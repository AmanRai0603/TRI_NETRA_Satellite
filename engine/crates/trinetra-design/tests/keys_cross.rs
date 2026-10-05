//! The page (design/js/tnkeys.js, Web Crypto) and this library open each other's keys and check each
//! other's signatures, and a changed statement is refused by both. Skipped when Node is not installed.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use std::process::Command;
use trinetra_design::keys::*;

fn node(args: &[&str]) -> Option<(bool, String)> {
    let cli = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../design/js/tnkeys_cli.mjs");
    let o = Command::new("node").arg(cli).args(args).output().ok()?;
    Some((o.status.success(), String::from_utf8_lossy(&o.stdout).trim().to_string()))
}

#[test]
fn the_page_and_the_application_share_keys_and_signatures() {
    let Some((ok, out)) = node(&["make", "Asha", "correct horse battery", "2000"]) else {
        eprintln!("node is not installed: skipped");
        return;
    };
    assert!(ok, "{out}");
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    let k = &v["key"];
    let sealed = SealedKey {
        name: k["name"].as_str().unwrap().into(), public_key: k["public_key"].as_str().unwrap().into(),
        fingerprint: k["fingerprint"].as_str().unwrap().into(), kdf_iterations: k["kdf_iterations"].as_u64().unwrap() as u32,
        salt: k["salt"].as_str().unwrap().into(), nonce: k["nonce"].as_str().unwrap().into(), sealed_private: k["sealed_private"].as_str().unwrap().into(),
    };
    // the page's key opens here, with the same public half and fingerprint
    let sk = unlock(&sealed, "correct horse battery").unwrap();
    assert_eq!(fingerprint(&sk.verifying_key().to_bytes()), sealed.fingerprint);
    assert!(unlock(&sealed, "not the passphrase").is_err());
    let s = &v["statement"];
    let st = Statement { role: s["role"].as_str().unwrap().into(), signer: s["signer"].as_str().unwrap().into(),
        revision: s["revision"].as_i64().unwrap(), content_hash: s["content_hash"].as_str().unwrap().into(),
        at: s["at"].as_str().unwrap().into(), statement: s["statement"].as_str().unwrap().into() };
    // the page's signature checks here; changed, it does not
    verify(&sealed.public_key, &st, v["signature"].as_str().unwrap()).unwrap();
    let mut other = st.clone();
    other.revision = 4;
    assert!(verify(&sealed.public_key, &other, v["signature"].as_str().unwrap()).is_err());
    // the other way: a key sealed and a statement signed here open and check in the page
    let mine = seal("Ravi", &[9u8; 32], "another long passphrase", 2000).unwrap();
    let st2 = Statement { signer: "Ravi".into(), ..st.clone() };
    let sig = sign(&unlock(&mine, "another long passphrase").unwrap(), &st2);
    let kj = serde_json::json!({"name": mine.name, "public_key": mine.public_key, "fingerprint": mine.fingerprint, "kdf_iterations": mine.kdf_iterations,
        "salt": mine.salt, "nonce": mine.nonce, "sealed_private": mine.sealed_private}).to_string();
    let sj = |s: &Statement| serde_json::json!({"role": s.role, "signer": s.signer, "revision": s.revision, "content_hash": s.content_hash, "at": s.at, "statement": s.statement}).to_string();
    assert_eq!(node(&["check", &kj, "another long passphrase", &sj(&st2), &sig]).unwrap(), (true, "ok".into()));
    let changed = Statement { content_hash: "00".repeat(32), ..st2.clone() };
    let (ok, why) = node(&["check", &kj, "another long passphrase", &sj(&changed), &sig]).unwrap();
    assert!(!ok && why.contains("does not match"), "{why}");
    let (ok, why) = node(&["check", &kj, "wrong passphrase!", &sj(&st2), &sig]).unwrap();
    assert!(!ok && why.contains("does not open"), "{why}");
}
