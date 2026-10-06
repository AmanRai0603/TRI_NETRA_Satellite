// pcode_cli.mjs -- the pseudocode v2 tools under Node, driven by tools/pcode.py.
//   node pcode_cli.mjs check FILE...                  problems as file:line:col: message; exit 1 on any
//   node pcode_cli.mjs run FILE... --fn NAME --args JSON   the outputs (SI) as JSON
//   node pcode_cli.mjs vectors FILE... [--n N] [--seed S]  test vectors from the interpreter, as JSON
//   node pcode_cli.mjs rust FILE... [--title T]        {path: text} of the Rust crate's sources, as JSON
//   node pcode_cli.mjs matlab FILE... [--pkg P]        {path: text} of the MATLAB package, as JSON
//   node pcode_cli.mjs matlab-rt                       {path: text} of the shared MATLAB runtime, as JSON
//   node pcode_cli.mjs signatures FILE...              every fn: module, name, inputs, outputs, as JSON
// Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
import { readFileSync } from "node:fs";
import { compile, makeInterpreter, prng, typeText } from "./pcode.js";
import { toRust, toMatlab, matlabRuntime } from "./pcode_gen.js";
import { toC } from "./pcode_c.js";

const argv = process.argv.slice(2);
const cmd = argv.shift();
const files = [], opt = {};
for (let i = 0; i < argv.length; i++) {
  if (argv[i] === "--no-dispatch") opt["no-dispatch"] = true;          // a flag: it takes no value
  else if (argv[i].startsWith("--")) opt[argv[i].slice(2)] = argv[++i];
  else files.push(argv[i]);
}
const out = (x) => process.stdout.write(JSON.stringify(x, null, 1) + "\n");

if (cmd === "matlab-rt") { out(matlabRuntime()); process.exit(0); }

const { program, errors } = compile(files.map((f) => ({ file: f, text: readFileSync(f, "utf8") })));
if (errors.length) {
  for (const e of errors) process.stderr.write(`${e.where()}: ${e.message}\n`);
  if (cmd === "check") process.stdout.write(`pcode: ${files.length} file(s), ${errors.length} problem(s)\n`);
  process.exit(1);
}

// a value as numbers: arrays element by element, a record field by field in declaration order
const flatten = (v) => (Array.isArray(v) ? v.flatMap(flatten) : typeof v === "boolean" ? [v ? 1 : 0]
  : v && typeof v === "object" ? Object.values(v).flatMap(flatten) : [v]);

// a double's exact bits as [high, low] 32-bit words: for readers whose decimal parsing is not
// correctly rounded (Octave's jsondecode)
function bits(x) {
  const b = new DataView(new ArrayBuffer(8)); b.setFloat64(0, x);
  return [b.getUint32(0), b.getUint32(4)];
}

// does a fn (or anything it calls) use a function whose last bits differ between maths libraries?
const TRANSCENDENTAL = new Set(["sin", "cos", "tan", "asin", "acos", "atan", "atan2", "exp", "log", "log10", "pow"]);
function usesTranscendental(f, seen = new Set()) {
  if (seen.has(f.name)) return false;
  seen.add(f.name);
  let found = false;
  const walk = (n) => {
    if (found || !n || typeof n !== "object") return;
    if (Array.isArray(n)) { n.forEach(walk); return; }
    if (n.e === "call") {
      if (n.builtin && TRANSCENDENTAL.has(n.f)) { found = true; return; }
      if (n.target && n.target.kind !== "table" && usesTranscendental(n.target, seen)) { found = true; return; }
    }
    for (const k of Object.keys(n)) if (!["target", "ty", "pos", "vty"].includes(k)) walk(n[k]);
  };
  walk(f.body);
  return found;
}

switch (cmd) {
  case "check": {
    const n = Object.values(program.fns).length, t = Object.values(program.tables).length;
    process.stdout.write(`pcode: ${files.length} file(s), ${n} fn/proc, ${t} table(s), 0 problems\n`);
    break;
  }
  case "signatures": {
    out(Object.values(program.fns).map((f) => ({
      module: f.module, name: f.name, kind: f.kind,
      inputs: f.params.map((p) => ({ name: p.name, type: typeText(p.ty) })),
      outputs: f.outs.map((o) => ({ name: o.name, type: typeText(o.ty) })),
      doc: f.doc,
    })));
    break;
  }
  case "run": {
    const I = makeInterpreter(program);
    const r = I.call(opt.fn, JSON.parse(opt.args || "[]"));
    out(r);
    break;
  }
  case "vectors": {
    // per fn: n input sets drawn inside each input's range (log-uniform for a positive range),
    // and the interpreter's outputs; a set that makes the interpreter fail is left out
    const I = makeInterpreter(program);
    // n vectors a function, or fewer where its inputs and outputs are many (--budget values a function)
    const nWant = +(opt.n || 12), budget = +(opt.budget || 1e9), rand = prng(+(opt.seed || 1));
    const width = (t) => (t.k === "arr" ? t.n * width(t.of) : t.k === "rec" ? program.records[t.name].fields.reduce((a, f) => a + width(f.ty), 0) : 1);
    const res = {};
    for (const f of Object.values(program.fns)) {
      // a function may ask for its own count in its documentation (`## vectors: 96`): a branchy one
      const asked = (f.doc || []).map((d) => /^vectors:\s*(\d+)\s*$/.exec(d)).find(Boolean);
      const n = asked ? +asked[1] : Math.max(4, Math.min(nWant, Math.floor(budget / [...f.params, ...f.outs].reduce((a, x) => a + width(x.ty), 0))));
      const draw = (p) => {
        const t = p.ty;
        const scalar = (lo, hi, isI) => {
          let v = lo > 0 && hi / lo > 20 ? Math.exp(Math.log(lo) + rand() * (Math.log(hi) - Math.log(lo))) : lo + rand() * (hi - lo);
          return isI ? Math.round(v) : v;
        };
        const range = p.range ? p.range.map((r) => r.bound) : null;
        const one = (tt) => {
          if (tt.k === "rec") return Object.fromEntries(program.records[tt.name].fields.map((fl) => [fl.name, draw(fl)]));
          if (tt.k === "arr") return Array.from({ length: tt.n }, () => one(tt.of));
          if (tt.k === "bool") return rand() < 0.5;
          if (tt.k === "int") return scalar(range ? range[0] : 0, range ? range[1] : 10, true);
          return scalar(range ? range[0] : 0.1, range ? range[1] : 10, false);
        };
        return one(t);
      };
      const sets = [];
      if (f.kind === "proc") {
        // a proc: runs of 8 consecutive calls, the state carried from each to the next
        for (let k = 0; k < n * 3 && sets.length < Math.max(2, n / 4); k++) {
          const st = I.newState(f.name), calls = [];
          try {
            for (let c = 0; c < 8; c++) {
              const ins = f.params.map(draw);
              const fo = I.call(f.name, ins, st).flatMap(flatten), fi = ins.flatMap(flatten);
              if (fo.some((x) => !Number.isFinite(x))) throw new Error("not finite");
              calls.push({ in: fi, out: fo, in_bits: fi.map(bits), out_bits: fo.map(bits) });
            }
          } catch (e) { continue; }
          sets.push({ calls });
        }
        res[`${f.module}::${f.name}`] = { exact: !usesTranscendental(f), proc: true, sets };
        continue;
      }
      // or that its inputs be drawn through another fn (`## inputs from: uart_stream`): that fn's
      // outputs give the inputs of the same name, the rest are drawn as usual
      const from = (f.doc || []).map((d) => /^inputs from:\s*(\w+)\s*$/.exec(d)).find(Boolean);
      const g = from ? program.fns[from[1]] : null;
      if (from && !g) { process.stderr.write(`pcode_cli: ${f.name}: inputs from ${from[1]}, which is no fn\n`); process.exit(1); }
      const drawAll = () => {
        const ins = f.params.map(draw);
        if (g) {
          const go = I.call(g.name, g.params.map(draw));
          g.outs.forEach((o, j) => { const i = f.params.findIndex((p) => p.name === o.name); if (i >= 0) ins[i] = go[j]; });
        }
        return ins;
      };
      for (let k = 0; k < n * 3 && sets.length < n; k++) {
        let ins;
        try { ins = drawAll(); } catch (e) { continue; }
        try {
          const o = I.call(f.name, ins);
          const fo = o.flatMap(flatten);
          if (fo.some((x) => !Number.isFinite(x))) continue;
          const fi = ins.flatMap(flatten);
          sets.push({ in: fi, out: fo, in_bits: fi.map(bits), out_bits: fo.map(bits) });
        } catch (e) { continue; }
      }
      res[`${f.module}::${f.name}`] = { exact: !usesTranscendental(f), sets };
    }
    out(res);
    break;
  }
  case "outkinds": {
    // per fn and proc: for each of its outputs flattened (records field by field), whether it is a whole number or a
    // yes/no (no sign of zero) rather than a real
    const kinds = (t) => (t.k === "arr" ? Array.from({ length: t.n }, () => kinds(t.of)).flat()
      : t.k === "rec" ? program.records[t.name].fields.flatMap((f) => kinds(f.ty)) : [t.k === "int" || t.k === "bool"]);
    out(Object.fromEntries(Object.values(program.fns).map((f) => [`${f.module}::${f.name}`, f.outs.flatMap((o) => kinds(o.ty))])));
    break;
  }
  // --root and --math embed the Rust as a module of a no_std crate (the flight build); --no-dispatch leaves out the
  // vector dispatcher, a test aid
  case "rust": out(toRust(program, { title: opt.title, root: opt.root, math: opt.math, dispatch: !("no-dispatch" in opt) })); break;
  case "c": out(toC(program, { title: opt.title, lib: opt.lib, dispatch: !("no-dispatch" in opt) })); break;
  case "matlab": out(toMatlab(program, { pkg: opt.pkg })); break;
  default:
    process.stderr.write(`pcode_cli: no command ${cmd} (check, run, vectors, outkinds, rust, c, matlab, matlab-rt, signatures)\n`);
    process.exit(2);
}

