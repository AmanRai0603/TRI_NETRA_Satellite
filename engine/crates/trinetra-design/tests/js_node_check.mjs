// js_node_check.mjs -- the node app's live checks (design/js/node_model.js) for the Rust port's parity
// test (tests/node_rules.rs). Reads {folder, files} as JSON on stdin: each node file is read as the
// node app reads it (readDoc over sql.js) and checked with ctx.nodes built from every node file of
// folder/nodes (as release.js designContext builds it from the design's index); without a folder, ctx
// is {}. Writes, for each file, {problems: [[code, level, step, text], ...]} or {throws: message}.
// Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
import { createRequire } from "node:module";
import { readFileSync, readdirSync } from "node:fs";
import path from "node:path";

const require = createRequire(import.meta.url);
const ROOT = new URL("../../../../", import.meta.url).pathname;
const initSqlJs = require(ROOT + "design/vendor/sqljs/sql-wasm.js");
const SQL = await initSqlJs({ wasmBinary: readFileSync(ROOT + "design/vendor/sqljs/sql-wasm.wasm") });
const M = await import(ROOT + "design/js/node_model.js");
const { CATALOG } = await import(ROOT + "design/js/node_catalog.js");

function open(file) {
  const db = new SQL.Database(readFileSync(file));
  const query = (q, p = []) => { const st = db.prepare(q); try { st.bind(p); const o = []; while (st.step()) o.push(st.get()); return o; } finally { st.free(); } };
  return { db, query };
}

const req = JSON.parse(readFileSync(0, "utf8"));
const ctx = {};
if (req.folder) {
  ctx.nodes = new Map();
  const dir = path.join(req.folder, "nodes");
  for (const f of readdirSync(dir).filter((x) => x.endsWith(".node.tndb")).sort()) {
    const { db, query } = open(path.join(dir, f));
    try {
      for (const [id, label, group, layer, kind] of query("SELECT id, label, group_id, layer, kind FROM node")) {
        const spec = CATALOG.rows[id];
        ctx.nodes.set(id, { label, group, layer, kind, quantity: spec ? spec[2] : null, unit: spec ? spec[3] : null });
      }
    } finally { db.close(); }
  }
}
const out = req.files.map((file) => {
  const { db, query } = open(file);
  try {
    const doc = M.readDoc({ query });
    return { problems: M.check(doc, ctx).map((p) => [p.code, p.level, p.step, p.text]) };
  } catch (e) {
    return { throws: String(e && e.message) };
  } finally { db.close(); }
});
process.stdout.write(JSON.stringify(out) + "\n");
