// js_check.mjs -- the JavaScript checker's answers for the Rust checker's parity test (tests/checker.rs).
// Reads cases as JSON on stdin, [{label, files: [[name, text], ...]}, ...], and writes each case's
// problems as `file:line:col: message` lines (design/js/pcode.js's `compile`, as pcode_cli.mjs prints
// them), or ["INTERNAL"] where the checker itself throws.
// Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
import { readFileSync } from "node:fs";
import { compile } from "../../../../design/js/pcode.js";

const cases = JSON.parse(readFileSync(0, "utf8"));
const out = cases.map((c) => {
  try {
    const { errors } = compile(c.files.map(([file, text]) => ({ file, text })));
    return errors.map((e) => `${e.where()}: ${e.message}`);
  } catch (e) {
    return ["INTERNAL"];
  }
});
process.stdout.write(JSON.stringify(out) + "\n");
