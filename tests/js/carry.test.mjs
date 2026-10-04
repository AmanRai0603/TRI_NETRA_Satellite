// carry.test.mjs -- the carried-over design (tools/carry_over.py, docs/RELEASE_PLAN.md P8) through
// the node app's own code (design/js/node_model.js): every node file reads, its checks run, every
// carried pseudocode compiles in the node app's checker, and every node with pseudocode and test
// vectors reproduces them in the node app's interpreter ("Try it").
//
//   node tests/js/carry.test.mjs DESIGN_DIR      prints one JSON summary; exit 1 on any failure
// Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
import { createRequire } from "node:module";
import { readFileSync, readdirSync } from "node:fs";
import path from "node:path";

const require = createRequire(import.meta.url);
const ROOT = new URL("../../", import.meta.url).pathname;
const initSqlJs = require(ROOT + "design/vendor/sqljs/sql-wasm.js");
const SQL = await initSqlJs({ wasmBinary: readFileSync(ROOT + "design/vendor/sqljs/sql-wasm.wasm") });
const M = await import(ROOT + "design/js/node_model.js");

const dir = path.join(process.argv[2], "nodes");
const out = { nodes: 0, checked: 0, pseudocode: 0, compiled: 0, symbol_mismatch: [], tried: 0, reproduced: 0, failures: [], kinds: {}, gaps: 0 };
for (const f of readdirSync(dir).filter((x) => x.endsWith(".node.tndb")).sort()) {
  const db = new SQL.Database(readFileSync(path.join(dir, f)));
  const query = (q, p = []) => { const st = db.prepare(q); try { st.bind(p); const o = []; while (st.step()) o.push(st.get()); return o; } finally { st.free(); } };
  try {
    const doc = M.readDoc({ query });
    out.nodes++;
    out.kinds[doc.kind || "unchosen"] = (out.kinds[doc.kind || "unchosen"] || 0) + 1;
    M.check(doc, {});
    out.checked++;
    if (JSON.parse(doc.content["status.gaps"] || "[]").length) out.gaps++;
    const pc = doc.content["code.pseudocode"];
    if (pc) {
      out.pseudocode++;
      const r = M.pcodeCheck(pc, doc.content["output.symbol"], doc.inputs);
      const hard = r.problems.filter((p) => !/R03|has no input/.test(p));
      if (hard.length) out.failures.push(`${doc.node.id}: the carried pseudocode does not compile: ${hard.join("; ")}`);
      else out.compiled++;
      if (r.problems.some((p) => /R03/.test(p))) out.symbol_mismatch.push(doc.node.id);
      if (!hard.length && doc.fixtures.length && r.fn) {
        out.tried++;
        const t = M.tryIt(doc);
        if (t.ok) out.reproduced++;
        else out.failures.push(`${doc.node.id}: the pseudocode does not reproduce its test vectors: ${JSON.stringify(t.runs || t.problems)}`);
      }
    }
  } catch (e) { out.failures.push(`${f}: ${e.message}`); }
  finally { db.close(); }
}
console.log(JSON.stringify(out, null, 1));
process.exit(out.failures.length ? 1 : 0);
