// js_math.mjs -- JavaScript's Math for the maths library's parity test (tests/math.rs).
// Reads {name: [[x, y], ...]} on stdin, every number as its 64 bits in hex, and writes {name: [r, ...]},
// r = Math[name](x) (or Math[name](x, y) for atan2 and pow), in hex too.
// Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
import { readFileSync } from "node:fs";

const dv = new DataView(new ArrayBuffer(8));
const num = (h) => { dv.setBigUint64(0, BigInt("0x" + h)); return dv.getFloat64(0); };
const hex = (x) => { dv.setFloat64(0, x); return dv.getBigUint64(0).toString(16).padStart(16, "0"); };

const req = JSON.parse(readFileSync(0, "utf8"));
const out = {};
for (const [name, args] of Object.entries(req)) out[name] = args.map(([x, y]) => hex(Math[name](num(x), num(y))));
process.stdout.write(JSON.stringify(out) + "\n");
