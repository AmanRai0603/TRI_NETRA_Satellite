// tnkeys_cli.mjs -- the page's key scheme from the command line, for the cross-language test
// (engine/crates/trinetra-design/tests/keys_cross.rs): the installed application and the page must
// open each other's keys and check each other's signatures.
//   node tnkeys_cli.mjs make NAME PASSPHRASE ITERATIONS        -> {key, statement, signature}
//   node tnkeys_cli.mjs check KEYJSON PASSPHRASE STATEMENTJSON SIGNATURE   -> "ok" or the refusal
// Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
import * as K from "./tnkeys.js";

const [cmd, ...a] = process.argv.slice(2);
try {
  if (cmd === "make") {
    const [name, pass, it] = a;
    const key = await K.seal(name, crypto.getRandomValues(new Uint8Array(32)), pass, Number(it));
    const statement = { role: "node engineer", signer: name, revision: 3, content_hash: "ef".repeat(32), at: "2026-10-06T10:00:00Z", statement: "the day's work" };
    const signature = await K.sign(await K.unlock(key, pass), statement);
    console.log(JSON.stringify({ key, statement, signature }));
  } else if (cmd === "check") {
    const [keyJson, pass, stJson, sig] = a;
    const key = JSON.parse(keyJson), st = JSON.parse(stJson);
    await K.unlock(key, pass);                      // the page opens the application's key
    await K.verify(key.public_key, st, sig);        // and checks its signature
    await K.checkChain([{ name: key.name, role: st.role, public_key: key.public_key }], st, sig, key.fingerprint);
    console.log("ok");
  } else {
    throw new Error(`no command ${cmd}`);
  }
} catch (e) { console.log(`refused: ${e.message}`); process.exitCode = 1; }
