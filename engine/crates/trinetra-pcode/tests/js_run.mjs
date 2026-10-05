// js_run.mjs -- the JavaScript interpreter's answers for the Rust interpreter's parity test (tests/interp.rs).
// Reads runs as JSON on stdin, [{files: [[name, text], ...], fn, args}, ...], and writes for each
// {out: [bits...]} (the outputs flattened as pcode_cli.mjs flattens them, each as its 64 bits in hex,
// so -0 and NaN survive) or {error: message} where the run stops.
// Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
import { readFileSync } from "node:fs";
import { compile, makeInterpreter } from "../../../../design/js/pcode.js";

const flatten = (v) => (Array.isArray(v) ? v.flatMap(flatten) : typeof v === "boolean" ? [v ? 1 : 0]
  : v && typeof v === "object" ? Object.values(v).flatMap(flatten) : [v]);
const hex = (x) => { const b = new DataView(new ArrayBuffer(8)); b.setFloat64(0, x); return b.getBigUint64(0).toString(16).padStart(16, "0"); };

const runs = JSON.parse(readFileSync(0, "utf8"));
const out = runs.map((r) => {
  const { program, errors } = compile(r.files.map(([file, text]) => ({ file, text })));
  if (errors.length) return { error: "does not check: " + errors[0].message };
  try {
    return { out: makeInterpreter(program).call(r.fn, r.args).flatMap(flatten).map(hex) };
  } catch (e) {
    return { error: e.message };
  }
});
process.stdout.write(JSON.stringify(out) + "\n");
