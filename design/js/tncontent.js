// tncontent.js -- a design file's content as canonical bytes (trinetra-content/1) and the hash a
// signature covers, in the page; the same encoding as engine/crates/trinetra-design/src/content.rs,
// held to it by a test on a real file (engine/crates/trinetra-design/tests/content_cross.rs).
//   cell: n null | i<decimal> integer | r<16 hex> real (its IEEE-754 bits) | t<hex> text (UTF-8) | b<hex> blob
//   one "T <table>" line per signed table in the schema's order, then its rows (cells joined by "|"), sorted
// Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
import { TNDB_SCHEMA } from "./tndb_schema.js";

export const ENCODING = "trinetra-content/1";
export const NOT_SIGNED = ["meta", "revision", "comment", "change_request", "signature", "key_signature"];
const hex = (bytes) => [...bytes].map((x) => x.toString(16).padStart(2, "0")).join("");

/** One cell, from SQLite's typeof() and its value. */
export function cell(type, v) {
  if (type === "null") return "n";
  if (type === "integer") return "i" + String(v);
  if (type === "real") { const d = new DataView(new ArrayBuffer(8)); d.setFloat64(0, v); return "r" + hex(new Uint8Array(d.buffer)); }
  if (type === "text") return "t" + hex(new TextEncoder().encode(v));
  return "b" + hex(v);
}

/** The tables of a kind a signature covers, in the schema's order. */
export const signedTables = (kind, schema = TNDB_SCHEMA) => schema.formats[kind].tables.filter((t) => !NOT_SIGNED.includes(t));

/** The canonical text of the tables named, from an open sql.js database. */
export function encode(db, tables, schema = TNDB_SCHEMA) {
  let out = `${ENCODING}\n`;
  for (const t of tables) {
    const cols = schema.tables[t].map((c) => c[0]);
    const sel = cols.map((c) => `typeof("${c}"), "${c}"`).join(", ");
    const lines = [];
    const st = db.prepare(`SELECT ${sel} FROM "${t}"`);
    while (st.step()) {
      const r = st.get();
      const cells = [];
      for (let i = 0; i < r.length; i += 2) cells.push(cell(r[i], r[i + 1]));
      lines.push(cells.join("|"));
    }
    st.free();
    lines.sort();
    out += `T ${t}\n` + lines.map((l) => l + "\n").join("");
  }
  return out;
}

/** SHA-256 of the canonical bytes of what a signature covers, as hex. */
export async function contentHash(db, kind, schema = TNDB_SCHEMA) {
  const bytes = new TextEncoder().encode(encode(db, signedTables(kind, schema), schema));
  return hex(new Uint8Array(await globalThis.crypto.subtle.digest("SHA-256", bytes)));
}
