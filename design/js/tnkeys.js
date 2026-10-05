// tnkeys.js -- people's keys and signatures in the page (docs/OPERATING_2_0.md §14), with Web Crypto
// only: no library, no network. The same scheme as engine/crates/trinetra-design/src/keys.rs, so a key
// made here opens in the installed application and a signature made there checks here:
//   key        Ed25519; fingerprint = hex SHA-256 of the 32-byte public key
//   lock       PBKDF2-HMAC-SHA256 (salt 16 bytes) -> AES-256-GCM (nonce 12 bytes) over the 32-byte seed
//   signature  over the fixed text of message(): role, signer, revision, content hash, time, statement
// Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.

export const KDF_ITERATIONS = 600000;
export const SIGNATURE_SCHEME = "trinetra-signature/1";
const subtle = globalThis.crypto.subtle;
// PKCS#8 wrapping of a 32-byte Ed25519 seed (RFC 8410): the only form Web Crypto imports a private key in
const PKCS8_PREFIX = Uint8Array.from([0x30, 0x2e, 0x02, 0x01, 0x00, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x04, 0x22, 0x04, 0x20]);

export class KeyError extends Error {}

const b64 = (bytes) => btoa(String.fromCharCode(...new Uint8Array(bytes)));
const unb64 = (s) => Uint8Array.from(atob(s), (c) => c.charCodeAt(0));
const hex = (bytes) => [...new Uint8Array(bytes)].map((x) => x.toString(16).padStart(2, "0")).join("");

/** The hex SHA-256 of a 32-byte public key. */
export async function fingerprint(publicRaw) { return hex(await subtle.digest("SHA-256", publicRaw)); }

/** A fingerprint as people read it: groups of four, the first 32 hex digits. */
export const readable = (fp) => fp.slice(0, 32).match(/.{1,4}/g).join(" ");

async function aesKey(passphrase, salt, iterations) {
  const base = await subtle.importKey("raw", new TextEncoder().encode(passphrase), "PBKDF2", false, ["deriveKey"]);
  return subtle.deriveKey({ name: "PBKDF2", hash: "SHA-256", salt, iterations }, base, { name: "AES-GCM", length: 256 }, false, ["encrypt", "decrypt"]);
}

async function signingKey(seed) {
  const p8 = new Uint8Array(PKCS8_PREFIX.length + 32);
  p8.set(PKCS8_PREFIX); p8.set(seed, PKCS8_PREFIX.length);
  return subtle.importKey("pkcs8", p8, { name: "Ed25519" }, true, ["sign"]);
}

async function publicOf(seed) {
  // the public half of a seed: exported from the private key's JWK (x is the public key)
  const jwk = await subtle.exportKey("jwk", await signingKey(seed));
  return Uint8Array.from(atob(jwk.x.replace(/-/g, "+").replace(/_/g, "/") + "=".repeat((4 - (jwk.x.length % 4)) % 4)), (c) => c.charCodeAt(0));
}

/** Lock a 32-byte seed with a passphrase: { name, public_key, fingerprint, kdf_iterations, salt, nonce, sealed_private }. */
export async function seal(name, seed, passphrase, iterations = KDF_ITERATIONS) {
  if ([...passphrase].length < 8) throw new KeyError("a passphrase of at least 8 characters");
  const salt = crypto.getRandomValues(new Uint8Array(16)), nonce = crypto.getRandomValues(new Uint8Array(12));
  const sealed = await subtle.encrypt({ name: "AES-GCM", iv: nonce }, await aesKey(passphrase, salt, iterations), seed);
  const pub = await publicOf(seed);
  return { name, public_key: b64(pub), fingerprint: await fingerprint(pub), kdf_iterations: iterations, salt: b64(salt), nonce: b64(nonce), sealed_private: b64(sealed) };
}

/** Make a new key for a person, locked with their passphrase. */
export async function make(name, passphrase) { return seal(name, crypto.getRandomValues(new Uint8Array(32)), passphrase); }

/** Open a sealed key: the Web Crypto signing key. A wrong passphrase is refused by name. */
export async function unlock(k, passphrase) {
  let seed;
  try {
    seed = new Uint8Array(await subtle.decrypt({ name: "AES-GCM", iv: unb64(k.nonce) }, await aesKey(passphrase, unb64(k.salt), k.kdf_iterations), unb64(k.sealed_private)));
  } catch { throw new KeyError(`the passphrase does not open ${k.name}'s key`); }
  if (b64(await publicOf(seed)) !== k.public_key) throw new KeyError(`${k.name}'s key: its private half does not match its public half`);
  return signingKey(seed);
}

/** The bytes a signature covers. */
export function message(s) {
  return new TextEncoder().encode(`${SIGNATURE_SCHEME}\nrole ${s.role}\nsigner ${s.signer}\nrevision ${s.revision}\ncontent ${s.content_hash}\nat ${s.at}\nstatement ${String(s.statement).replace(/\n/g, " ")}\n`);
}

/** Sign a statement: the base64 Ed25519 signature. */
export async function sign(key, s) { return b64(await subtle.sign({ name: "Ed25519" }, key, message(s))); }

/** Whether `signature` is this public key's signature of the statement; throws a KeyError naming why not. */
export async function verify(publicKey, s, signature) {
  const pk = await subtle.importKey("raw", unb64(publicKey), { name: "Ed25519" }, false, ["verify"]);
  if (!(await subtle.verify({ name: "Ed25519" }, pk, unb64(signature), message(s)))) {
    throw new KeyError(`${s.signer}'s signature does not match what it says it signed (changed since?)`);
  }
}

/** Check a signature through a registry [{ name, role, public_key, deputy_of }]: the entry that signed. */
export async function checkChain(reg, s, signature, keyFingerprint) {
  const who = reg.find((r) => r.name === s.signer);
  if (!who) throw new KeyError(`${s.signer} is not registered here; only registered people may sign`);
  const fp = await fingerprint(unb64(who.public_key));
  if (fp !== keyFingerprint) throw new KeyError(`${s.signer} signed with key ${readable(keyFingerprint)}, the registered key is ${readable(fp)}`);
  const deputyFor = who.deputy_of && reg.find((r) => r.name === who.deputy_of);
  if (who.role !== s.role && !(deputyFor && deputyFor.role === s.role)) {
    throw new KeyError(`${s.signer} is registered as ${who.role}, not as ${s.role} (nor as a deputy for one)`);
  }
  await verify(who.public_key, s, signature);
  return who;
}
