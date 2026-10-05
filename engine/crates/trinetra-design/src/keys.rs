//! People's keys and signatures (docs/OPERATING_2_0.md §14): make a key, lock its private half
//! with a passphrase, sign what a file holds, and check a signature through the chain of who
//! registered whom.
//!
//! - **The key** is Ed25519. Its fingerprint is the SHA-256 of the 32-byte public key, as hex.
//! - **The lock** is PBKDF2-HMAC-SHA256 over the passphrase (salt 16 bytes, [`KDF_ITERATIONS`])
//!   giving an AES-256-GCM key, which seals the 32-byte private seed (nonce 12 bytes). The same
//!   algorithms are in every current browser's Web Crypto, so a key made in the page opens here and
//!   the other way round.
//! - **A signature** covers a statement in a fixed text ([`message`]): the role signed as, the
//!   signer, the revision, the content hash (`content::content_hash`), the time and the statement.
//!   Nothing outside that text is signed, so every program that checks builds the same bytes.
//! - **The chain**: a node's signature checks against the keys its group file registers, a group's
//!   seal against the programme file's, and the programme file's against the programme manager's
//!   fingerprint written in START HERE (the anchor). A registry is a list of (name, role, public key).
//!
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};

/// PBKDF2 iterations for a key's lock (OWASP 2023 guidance for PBKDF2-HMAC-SHA256).
pub const KDF_ITERATIONS: u32 = 600_000;
/// The fixed first line of every signed message.
pub const SIGNATURE_SCHEME: &str = "trinetra-signature/1";

#[derive(Debug, Clone, PartialEq)]
pub struct KeyError(pub String);
impl std::fmt::Display for KeyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(&self.0) }
}
impl std::error::Error for KeyError {}

fn hex(b: &[u8]) -> String { b.iter().map(|x| format!("{x:02x}")).collect() }

/// A public key's fingerprint: the hex SHA-256 of its 32 bytes.
pub fn fingerprint(public: &[u8; 32]) -> String { hex(&Sha256::digest(public)) }

/// A fingerprint as people read it: groups of four, the first 32 hex digits.
pub fn readable(fp: &str) -> String {
    fp.chars().take(32).collect::<Vec<_>>().chunks(4).map(|c| c.iter().collect::<String>()).collect::<Vec<_>>().join(" ")
}

/// A key as kept on a person's computer: the public half, and the private half sealed.
#[derive(Debug, Clone, PartialEq)]
pub struct SealedKey {
    pub name: String,
    pub public_key: String,     // base64 of 32 bytes
    pub fingerprint: String,
    pub kdf_iterations: u32,
    pub salt: String,           // base64
    pub nonce: String,          // base64
    pub sealed_private: String, // base64 of AES-GCM(seed) with its tag
}

fn derive(passphrase: &str, salt: &[u8], iterations: u32) -> [u8; 32] {
    let mut k = [0u8; 32];
    pbkdf2::pbkdf2_hmac::<Sha256>(passphrase.as_bytes(), salt, iterations, &mut k);
    k
}

fn random<const N: usize>() -> Result<[u8; N], KeyError> {
    let mut b = [0u8; N];
    getrandom::getrandom(&mut b).map_err(|e| KeyError(format!("no randomness from the system: {e}")))?;
    Ok(b)
}

/// Lock a private seed with a passphrase.
pub fn seal(name: &str, seed: &[u8; 32], passphrase: &str, iterations: u32) -> Result<SealedKey, KeyError> {
    if passphrase.chars().count() < 8 {
        return Err(KeyError("a passphrase of at least 8 characters".into()));
    }
    let (salt, nonce) = (random::<16>()?, random::<12>()?);
    let cipher = Aes256Gcm::new_from_slice(&derive(passphrase, &salt, iterations)).map_err(|e| KeyError(e.to_string()))?;
    let sealed = cipher.encrypt(Nonce::from_slice(&nonce), seed.as_slice()).map_err(|_| KeyError("sealing failed".into()))?;
    let public = SigningKey::from_bytes(seed).verifying_key().to_bytes();
    Ok(SealedKey { name: name.into(), public_key: B64.encode(public), fingerprint: fingerprint(&public), kdf_iterations: iterations,
        salt: B64.encode(salt), nonce: B64.encode(nonce), sealed_private: B64.encode(sealed) })
}

/// Make a new key for a person, locked with their passphrase.
pub fn make(name: &str, passphrase: &str) -> Result<SealedKey, KeyError> {
    seal(name, &random::<32>()?, passphrase, KDF_ITERATIONS)
}

/// Open a sealed key with its passphrase. A wrong passphrase is refused, never guessed past.
pub fn unlock(k: &SealedKey, passphrase: &str) -> Result<SigningKey, KeyError> {
    let b = |s: &str, what: &str| B64.decode(s).map_err(|_| KeyError(format!("the key's {what} is not base64")));
    let (salt, nonce, sealed) = (b(&k.salt, "salt")?, b(&k.nonce, "nonce")?, b(&k.sealed_private, "sealed private half")?);
    if nonce.len() != 12 {
        return Err(KeyError("the key's nonce is not 12 bytes".into()));
    }
    let cipher = Aes256Gcm::new_from_slice(&derive(passphrase, &salt, k.kdf_iterations)).map_err(|e| KeyError(e.to_string()))?;
    let seed = cipher.decrypt(Nonce::from_slice(&nonce), sealed.as_slice())
        .map_err(|_| KeyError(format!("the passphrase does not open {}'s key", k.name)))?;
    let seed: [u8; 32] = seed.try_into().map_err(|_| KeyError("the private half is not 32 bytes".into()))?;
    let sk = SigningKey::from_bytes(&seed);
    if B64.encode(sk.verifying_key().to_bytes()) != k.public_key {
        return Err(KeyError(format!("{}'s key: its private half does not match its public half", k.name)));
    }
    Ok(sk)
}

/// What a signature says, in the fixed text every program builds the same way.
#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    pub role: String,
    pub signer: String,
    pub revision: i64,
    pub content_hash: String,
    pub at: String,
    pub statement: String,
}

/// The bytes a signature covers.
pub fn message(s: &Statement) -> Vec<u8> {
    format!("{SIGNATURE_SCHEME}\nrole {}\nsigner {}\nrevision {}\ncontent {}\nat {}\nstatement {}\n",
        s.role, s.signer, s.revision, s.content_hash, s.at, s.statement.replace('\n', " ")).into_bytes()
}

/// Sign a statement: the base64 Ed25519 signature.
pub fn sign(key: &SigningKey, s: &Statement) -> String { B64.encode(key.sign(&message(s)).to_bytes()) }

fn public(b64: &str) -> Result<VerifyingKey, KeyError> {
    let raw: [u8; 32] = B64.decode(b64).ok().and_then(|v| v.try_into().ok()).ok_or_else(|| KeyError("a public key is not 32 bytes of base64".into()))?;
    VerifyingKey::from_bytes(&raw).map_err(|_| KeyError("not an Ed25519 public key".into()))
}

/// Whether `signature` (base64) is this public key's signature of the statement.
pub fn verify(public_key: &str, s: &Statement, signature: &str) -> Result<(), KeyError> {
    let sig: [u8; 64] = B64.decode(signature).ok().and_then(|v| v.try_into().ok()).ok_or_else(|| KeyError("a signature is not 64 bytes of base64".into()))?;
    public(public_key)?.verify(&message(s), &Signature::from_bytes(&sig))
        .map_err(|_| KeyError(format!("{}'s signature does not match what it says it signed (changed since?)", s.signer)))
}

/// One registered key: who, as what role, their public key.
#[derive(Debug, Clone, PartialEq)]
pub struct Registered {
    pub name: String,
    pub role: String,
    pub public_key: String,
    pub deputy_of: Option<String>,
}

/// Check a signature through a registry: the signer is registered, in a role that may sign as the
/// role the statement claims (themselves, or as the deputy of the person who holds it), and the
/// signature is theirs. Returns the registered entry that signed.
pub fn check_chain<'a>(reg: &'a [Registered], s: &Statement, signature: &str, key_fingerprint: &str) -> Result<&'a Registered, KeyError> {
    let who = reg.iter().find(|r| r.name == s.signer)
        .ok_or_else(|| KeyError(format!("{} is not registered here; only registered people may sign", s.signer)))?;
    let pk = public(&who.public_key)?;
    if fingerprint(&pk.to_bytes()) != key_fingerprint {
        return Err(KeyError(format!("{} signed with key {}, the registered key is {}", s.signer, readable(key_fingerprint), readable(&fingerprint(&pk.to_bytes())))));
    }
    let as_deputy = who.deputy_of.as_ref().and_then(|d| reg.iter().find(|r| &r.name == d)).is_some_and(|d| d.role == s.role);
    if who.role != s.role && !as_deputy {
        return Err(KeyError(format!("{} is registered as {}, not as {} (nor as a deputy for one)", s.signer, who.role, s.role)));
    }
    verify(&who.public_key, s, signature)?;
    Ok(who)
}

/// Check that a registry file's own signer is the anchor (the programme manager's fingerprint,
/// from START HERE).
pub fn check_anchor(public_key: &str, anchor_fingerprint: &str) -> Result<(), KeyError> {
    let fp = fingerprint(&public(public_key)?.to_bytes());
    let norm = |x: &str| x.chars().filter(|c| c.is_ascii_hexdigit()).collect::<String>().to_lowercase();
    if !fp.starts_with(&norm(anchor_fingerprint)) || norm(anchor_fingerprint).len() < 32 {
        return Err(KeyError(format!("the programme file is signed by {}, not by the key START HERE names ({})", readable(&fp), anchor_fingerprint)));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn st(signer: &str, role: &str) -> Statement {
        Statement { role: role.into(), signer: signer.into(), revision: 4, content_hash: "ab".repeat(32), at: "2026-10-06T09:00:00Z".into(), statement: "ready".into() }
    }

    #[test]
    fn a_key_locks_with_its_passphrase_and_signs() {
        let k = seal("Asha", &[7u8; 32], "correct horse battery", 1000).unwrap();
        assert!(unlock(&k, "wrong passphrase").unwrap_err().0.contains("does not open Asha's key"));
        let sk = unlock(&k, "correct horse battery").unwrap();
        let s = st("Asha", "node engineer");
        let sig = sign(&sk, &s);
        verify(&k.public_key, &s, &sig).unwrap();
        let mut changed = s.clone();
        changed.content_hash = "cd".repeat(32);
        assert!(verify(&k.public_key, &changed, &sig).unwrap_err().0.contains("does not match"));
        assert!(seal("x", &[1; 32], "short", 1000).is_err());
        assert_eq!(readable(&k.fingerprint).len(), 39);
    }

    #[test]
    fn the_chain_names_who_may_sign_as_what() {
        let (a, b, c) = ([1u8; 32], [2u8; 32], [3u8; 32]);
        let pk = |s: &[u8; 32]| B64.encode(SigningKey::from_bytes(s).verifying_key().to_bytes());
        let fp = |s: &[u8; 32]| fingerprint(&SigningKey::from_bytes(s).verifying_key().to_bytes());
        let reg = vec![
            Registered { name: "Asha".into(), role: "node engineer".into(), public_key: pk(&a), deputy_of: None },
            Registered { name: "Ravi".into(), role: "subsystem engineer".into(), public_key: pk(&b), deputy_of: None },
            Registered { name: "Meera".into(), role: "node engineer".into(), public_key: pk(&c), deputy_of: Some("Ravi".into()) },
        ];
        let s = st("Asha", "node engineer");
        let sig = sign(&SigningKey::from_bytes(&a), &s);
        assert_eq!(check_chain(&reg, &s, &sig, &fp(&a)).unwrap().name, "Asha");
        // a role they do not hold
        let seal_as = st("Asha", "subsystem engineer");
        assert!(check_chain(&reg, &seal_as, &sign(&SigningKey::from_bytes(&a), &seal_as), &fp(&a)).unwrap_err().0.contains("registered as node engineer"));
        // a deputy signs for the one they stand in for
        let dep = st("Meera", "subsystem engineer");
        assert_eq!(check_chain(&reg, &dep, &sign(&SigningKey::from_bytes(&c), &dep), &fp(&c)).unwrap().name, "Meera");
        // someone not registered, or with another key
        let x = st("Kiran", "node engineer");
        assert!(check_chain(&reg, &x, &sign(&SigningKey::from_bytes(&a), &x), &fp(&a)).unwrap_err().0.contains("not registered"));
        assert!(check_chain(&reg, &s, &sign(&SigningKey::from_bytes(&b), &s), &fp(&b)).unwrap_err().0.contains("the registered key is"));
        // the anchor
        check_anchor(&pk(&b), &readable(&fp(&b))).unwrap();
        assert!(check_anchor(&pk(&a), &readable(&fp(&b))).is_err());
    }
}
