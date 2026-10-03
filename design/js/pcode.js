// pcode.js -- TRI-NETRA pseudocode v2: units, parser, checker and interpreter (docs/PSEUDOCODE_V2.md).
// One implementation for the browser (the checker page, the node app) and for Node (tools/pcode.py).
// No dependencies. Every value is held in SI; a unit states a dimension, which the checker holds.
// Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.

export const VERSION = "trinetra-pcode/2";

// ------------------------------------------------------------------ units
// dimension vector: [L, M, T, I, Theta]
const D = (l = 0, m = 0, t = 0, i = 0, k = 0) => [l, m, t, i, k];
export const UNITS = {
  "1": [D(), 1],
  m: [D(1), 1], km: [D(1), 1e3], cm: [D(1), 1e-2], mm: [D(1), 1e-3], um: [D(1), 1e-6], AU: [D(1), 1.495978707e11],
  s: [D(0, 0, 1), 1], ms: [D(0, 0, 1), 1e-3], min: [D(0, 0, 1), 60], h: [D(0, 0, 1), 3600], day: [D(0, 0, 1), 86400],
  yr: [D(0, 0, 1), 365.25 * 86400],
  kg: [D(0, 1), 1], g: [D(0, 1), 1e-3],
  N: [D(1, 1, -2), 1], mN: [D(1, 1, -2), 1e-3], uN: [D(1, 1, -2), 1e-6],
  J: [D(2, 1, -2), 1], W: [D(2, 1, -3), 1], mW: [D(2, 1, -3), 1e-3],
  Pa: [D(-1, 1, -2), 1], kPa: [D(-1, 1, -2), 1e3],
  A: [D(0, 0, 0, 1), 1], mA: [D(0, 0, 0, 1), 1e-3], C: [D(0, 0, 1, 1), 1],
  V: [D(2, 1, -3, -1), 1], ohm: [D(2, 1, -3, -2), 1],
  T: [D(0, 1, -2, -1), 1], uT: [D(0, 1, -2, -1), 1e-6], nT: [D(0, 1, -2, -1), 1e-9],
  K: [D(0, 0, 0, 0, 1), 1],
  Hz: [D(0, 0, -1), 1], rpm: [D(0, 0, -1), 2 * Math.PI / 60],
  rad: [D(), 1], deg: [D(), Math.PI / 180], arcsec: [D(), Math.PI / 648000],
};
const DIM_NAMES = ["m", "kg", "s", "A", "K"];

export function dimText(d) {
  const up = [], dn = [];
  d.forEach((p, i) => { if (p > 0) up.push(DIM_NAMES[i] + (p !== 1 ? "^" + p : "")); if (p < 0) dn.push(DIM_NAMES[i] + (p !== -1 ? "^" + -p : "")); });
  if (!up.length && !dn.length) return "1";
  return (up.length ? up.join(" ") : "1") + (dn.length ? "/" + (dn.length > 1 ? `(${dn.join(" ")})` : dn[0]) : "");
}
const dimEq = (a, b) => a.every((x, i) => x === b[i]);
const dimAdd = (a, b, s = 1) => a.map((x, i) => x + s * b[i]);
const dimMul = (a, k) => a.map((x) => x * k);
const DIMLESS = D();

// ------------------------------------------------------------------ errors
export class PcodeError extends Error {
  constructor(msg, line, col, file) { super(msg); this.line = line; this.col = col; this.file = file; }
  where() { return `${this.file || "<input>"}:${this.line || 0}:${this.col || 0}`; }
}

// ------------------------------------------------------------------ lexer
const KEYWORDS = new Set(["module", "use", "const", "fn", "proc", "table", "record", "end", "let", "state", "if", "then", "elif", "else",
  "for", "in", "settle", "until", "and", "or", "not", "true", "false", "step", "linear"]);
const OPS = ["->", "..", "==", "!=", "<=", ">=", "+", "-", "*", "/", "^", "(", ")", "[", "]", ",", ":", "=", "<", ">", ".", "|"];

export function lex(src, file) {
  const toks = [];
  let i = 0, line = 1, col = 1, depth = 0;
  const push = (t, v, l, c, extra = {}) => toks.push({ t, v, line: l, col: c, file, ...extra });
  while (i < src.length) {
    const ch = src[i];
    if (ch === "#") {
      const doc = src[i + 1] === "#";
      let j = i; while (j < src.length && src[j] !== "\n") j++;
      if (doc) push("doc", src.slice(i + 2, j).trim(), line, col);
      col += j - i; i = j; continue;
    }
    if (ch === "\n") {
      if (depth === 0) push("nl", "\n", line, col);
      i++; line++; col = 1; continue;
    }
    if (ch === " " || ch === "\t" || ch === "\r") { i++; col++; continue; }
    if (ch === "\\" && src[i + 1] === "\n") { i += 2; line++; col = 1; continue; }   // line continuation
    if (/[0-9]/.test(ch) || (ch === "." && /[0-9]/.test(src[i + 1] || ""))) {
      const m = /^(?:[0-9]+(?:\.[0-9]*)?|\.[0-9]+)(?:[eE][+-]?[0-9]+)?/.exec(src.slice(i));
      if (src[i + m[0].length] === "." && src[i + m[0].length + 1] === "." && m[0].endsWith(".")) {
        const s = m[0].slice(0, -1); push("num", s, line, col, { isInt: !/[.eE]/.test(s) }); i += s.length; col += s.length; continue;
      }
      push("num", m[0], line, col, { isInt: !/[.eE]/.test(m[0]) }); i += m[0].length; col += m[0].length; continue;
    }
    if (/[A-Za-z_]/.test(ch)) {
      const m = /^[A-Za-z_][A-Za-z_0-9]*/.exec(src.slice(i));
      push(KEYWORDS.has(m[0]) ? "kw" : "id", m[0], line, col); i += m[0].length; col += m[0].length; continue;
    }
    if (ch === '"') {
      let j = i + 1; while (j < src.length && src[j] !== '"' && src[j] !== "\n") j++;
      if (src[j] !== '"') throw new PcodeError("a string is not closed on its line", line, col, file);
      push("str", src.slice(i + 1, j), line, col); col += j + 1 - i; i = j + 1; continue;
    }
    const op = OPS.find((o) => src.startsWith(o, i));
    if (!op) throw new PcodeError(`a character the language does not use: ${JSON.stringify(ch)}`, line, col, file);
    if (op === "(" || op === "[") depth++;
    if (op === ")" || op === "]") depth = Math.max(0, depth - 1);
    push("op", op, line, col); i += op.length; col += op.length;
  }
  push("nl", "\n", line, col); push("eof", "", line, col);
  return toks;
}

// ------------------------------------------------------------------ parser
export function parse(src, file = "<input>") {
  const toks = lex(src, file);
  let p = 0;
  const peek = (k = 0) => toks[p + k];
  const err = (msg, t = peek()) => { throw new PcodeError(msg, t.line, t.col, file); };
  const is = (t, v) => peek().t === t && (v === undefined || peek().v === v);
  const eat = (t, v) => { if (!is(t, v)) err(`expected ${v || t}, found ${peek().v === "\n" ? "the end of the line" : JSON.stringify(peek().v)}`); return toks[p++]; };
  const opt = (t, v) => (is(t, v) ? toks[p++] : null);
  const skipNl = () => { while (is("nl")) p++; };
  const pos = (t) => ({ line: t.line, col: t.col, file });
  let docs = [];

  function unitText() {   // inside [ ... ]: rebuild the unit expression from its tokens
    eat("op", "[");
    const parts = [];
    while (!is("op", "]")) { if (is("eof") || is("nl")) err("a unit is not closed with ]"); parts.push(toks[p++].v); }
    eat("op", "]");
    return parts.join(" ").replace(/ \^ /g, "^").replace(/\^ - /g, "^-").replace(/ \/ /g, "/").replace(/ \* /g, " ");
  }
  function typ() {
    const t = peek();
    if (opt("id", "int")) return arrSuffix({ k: "int" });
    if (opt("id", "bool")) return arrSuffix({ k: "bool" });
    if (opt("id", "quat")) return arrSuffix({ k: "arr", n: 4, of: { k: "real", unit: "1" } });
    for (const [name, n, mat] of [["vec3", 3, false], ["mat3", 3, true], ["vec4", 4, false], ["vec2", 2, false]]) {
      if (opt("id", name)) {
        const u = is("op", "[") ? unitText() : "1";
        let ty = { k: "arr", n, of: { k: "real", unit: u } };
        if (mat) ty = { k: "arr", n, of: ty };
        return arrSuffix(ty);
      }
    }
    if (opt("id", "real")) return arrSuffix({ k: "real", unit: is("op", "[") ? unitText() : "1" });
    if (is("id")) return arrSuffix({ k: "rec", name: eat("id").v, pos: pos(t) });
    err("expected a type (real[unit], int, bool, vec3[unit], mat3[unit], quat, or a record)");
  }
  function arrSuffix(ty) {
    while (is("op", "[")) {
      eat("op", "["); const n = eat("num"); eat("op", "]");
      if (!n.isInt || +n.v < 1) err("an array's length is a whole number from 1", n);
      ty = { k: "arr", n: +n.v, of: ty };
    }
    return ty;
  }
  function param() {
    const t = eat("id"); eat("op", ":"); const ty = typ();
    let range = null;
    if (opt("kw", "in")) { const lo = expr(); eat("op", ".."); const hi = expr(); range = [lo, hi]; }
    return { name: t.v, type: ty, range, pos: pos(t) };
  }
  function outputs() {
    if (opt("op", "(")) {
      const outs = [param()];
      while (opt("op", ",")) outs.push(param());
      eat("op", ")"); return outs;
    }
    return [param()];
  }
  function block(stops) {
    const body = [];
    for (;;) {
      skipNl();
      if (is("eof")) err(`the block is not closed (expected ${stops.join(" or ")})`);
      if (is("doc")) { p++; continue; }
      if (is("kw") && stops.includes(peek().v)) return body;
      body.push(stmt());
    }
  }
  function endLine() { if (!is("eof")) eat("nl"); }
  function stmt() {
    const t = peek();
    if (opt("kw", "let")) {
      const names = [eat("id").v]; let ty = null;
      if (opt("op", ":")) ty = typ();
      while (opt("op", ",")) names.push(eat("id").v);
      eat("op", "="); const e = expr(); endLine();
      return { s: "let", names, type: ty, e, pos: pos(t) };
    }
    if (opt("kw", "state")) {
      const n = eat("id"); eat("op", ":"); const ty = typ(); eat("op", "="); const e = expr(); endLine();
      return { s: "state", name: n.v, type: ty, e, pos: pos(t) };
    }
    if (opt("kw", "if")) {
      const arms = []; let c = expr(); endLine();
      let body = block(["elif", "else", "end"]); arms.push({ c, body });
      let els = null;
      for (;;) {
        if (opt("kw", "elif")) { c = expr(); endLine(); body = block(["elif", "else", "end"]); arms.push({ c, body }); continue; }
        if (opt("kw", "else")) { endLine(); els = block(["end"]); }
        eat("kw", "end"); endLine(); break;
      }
      return { s: "if", arms, els, pos: pos(t) };
    }
    if (opt("kw", "for")) {
      const v = eat("id"); eat("kw", "in"); const a = expr(); eat("op", ".."); const b = expr(); endLine();
      const body = block(["end"]); eat("kw", "end"); endLine();
      return { s: "for", v: v.v, a, b, body, pos: pos(t) };
    }
    if (opt("kw", "settle")) {
      eat("id", "max"); const n = expr(); eat("kw", "until"); const c = expr(); endLine();
      const body = block(["else", "end"]); let els = null;
      if (opt("kw", "else")) { endLine(); els = block(["end"]); }
      eat("kw", "end"); endLine();
      return { s: "settle", n, c, body, els, pos: pos(t) };
    }
    // assignment: target[, target] = expr
    const targets = [lvalue()];
    while (opt("op", ",")) targets.push(lvalue());
    eat("op", "="); const e = expr(); endLine();
    return { s: "set", targets, e, pos: pos(t) };
  }
  function lvalue() {
    const t = eat("id"); let lv = { e: "var", name: t.v, pos: pos(t) };
    for (;;) {
      if (is("op", "[")) { const b = eat("op", "["); const i = expr(); eat("op", "]"); lv = { e: "index", a: lv, i, pos: pos(b) }; continue; }
      if (is("op", ".")) { const b = eat("op", "."); const f = eat("id"); lv = { e: "field", a: lv, f: f.v, pos: pos(b) }; continue; }
      return lv;
    }
  }
  // expressions, lowest precedence first
  function expr() {
    if (is("kw", "if")) {
      const t = eat("kw", "if"); const c = expr(); eat("kw", "then"); const a = expr(); eat("kw", "else"); const b = expr();
      return { e: "ifx", c, a, b, pos: pos(t) };
    }
    return orx();
  }
  function bin(next, ops) {
    return function f() {
      let a = next();
      for (;;) {
        const t = peek();
        const op = ops.find((o) => (t.t === "op" || t.t === "kw") && t.v === o);
        if (!op) return a;
        p++; const b = next(); a = { e: "bin", op, a, b, pos: pos(t) };
      }
    };
  }
  function notx() { if (is("kw", "not")) { const t = eat("kw", "not"); return { e: "un", op: "not", a: notx(), pos: pos(t) }; } return cmpx(); }
  function cmpx() {
    const a = addx(); const t = peek();
    const op = ["==", "!=", "<", "<=", ">", ">="].find((o) => t.t === "op" && t.v === o);
    if (!op) return a;
    p++; const b = addx();
    if (["==", "!=", "<", "<=", ">", ">="].some((o) => is("op", o))) err("comparisons do not chain: write (a < b) and (b < c)");
    return { e: "bin", op, a, b, pos: pos(t) };
  }
  const mulx = bin(unx, ["*", "/"]);
  const addx = bin(mulx, ["+", "-"]);
  const andx = bin(notx, ["and"]);
  const orx = bin(andx, ["or"]);
  function unx() {
    if (is("op", "-")) { const t = eat("op", "-"); return { e: "un", op: "-", a: unx(), pos: pos(t) }; }
    if (is("op", "+")) { eat("op", "+"); return unx(); }
    return powx();
  }
  function powx() {
    const a = postfix();
    if (is("op", "^")) { const t = eat("op", "^"); const b = unx(); return { e: "bin", op: "^", a, b, pos: pos(t) }; }
    return a;
  }
  function postfix() {
    let a = atom();
    for (;;) {
      if (is("op", "(") && (a.e === "var" || a.e === "qual")) {
        const t = eat("op", "("); const args = [];
        if (!is("op", ")")) { args.push(expr()); while (opt("op", ",")) args.push(expr()); }
        eat("op", ")"); a = { e: "call", f: a.e === "var" ? a.name : a.mod + "." + a.name, args, pos: pos(t) }; continue;
      }
      if (is("op", "[")) { const t = eat("op", "["); const i = expr(); eat("op", "]"); a = { e: "index", a, i, pos: pos(t) }; continue; }
      if (is("op", ".") && peek(1).t === "id") {
        const t = eat("op", "."); const f = eat("id");
        if (a.e === "var" && is("op", "(")) { a = { e: "qual", mod: a.name, name: f.v, pos: pos(t) }; continue; }
        a = { e: "field", a, f: f.v, pos: pos(t) }; continue;
      }
      return a;
    }
  }
  function atom() {
    const t = peek();
    if (is("num")) {
      p++; let unit = null;
      if (is("op", "[") && !(peek(1).t === "num" && peek(2).t === "op" && peek(2).v === "]")) unit = unitText();
      return { e: "num", v: +t.v, isInt: t.isInt && unit === null, unit, text: t.v, pos: pos(t) };
    }
    if (opt("kw", "true")) return { e: "bool", v: true, pos: pos(t) };
    if (opt("kw", "false")) return { e: "bool", v: false, pos: pos(t) };
    if (is("id")) { p++; return { e: "var", name: t.v, pos: pos(t) }; }
    if (opt("op", "(")) { const e = expr(); eat("op", ")"); return e; }
    if (opt("op", "|")) { const e = expr(); eat("op", "|"); return { e: "call", f: "abs", args: [e], pos: pos(t), bars: true }; }
    if (opt("op", "[")) {
      const items = [];
      if (!is("op", "]")) { items.push(expr()); while (opt("op", ",")) items.push(expr()); }
      eat("op", "]");
      let unit = null;
      if (is("op", "[") && !(peek(1).t === "num")) unit = unitText();
      return { e: "arr", items, unit, pos: pos(t) };
    }
    err(`expected a value, found ${t.v === "\n" ? "the end of the line" : JSON.stringify(t.v)}`);
  }

  // top level
  const prog = { file, module: null, uses: [], items: [] };
  for (;;) {
    skipNl();
    if (is("eof")) break;
    if (is("doc")) { docs.push(eat("doc").v); continue; }
    const t = peek();
    const doc = docs; docs = [];
    if (opt("kw", "module")) { prog.module = eat("id").v; endLine(); prog.doc = doc; continue; }
    if (opt("kw", "use")) { prog.uses.push(eat("id").v); endLine(); continue; }
    if (opt("kw", "const")) {
      const n = eat("id"); eat("op", "="); const e = expr(); endLine();
      prog.items.push({ kind: "const", name: n.v, e, doc, pos: pos(t) }); continue;
    }
    if (is("kw", "fn") || is("kw", "proc")) {
      const kind = toks[p++].v; const n = eat("id"); eat("op", "(");
      const params = [];
      if (!is("op", ")")) { params.push(param()); while (opt("op", ",")) params.push(param()); }
      eat("op", ")"); eat("op", "->"); const outs = outputs(); endLine();
      const body = block(["end"]); eat("kw", "end"); endLine();
      prog.items.push({ kind, name: n.v, params, outs, body, doc, pos: pos(t) }); continue;
    }
    if (opt("kw", "record")) {
      const n = eat("id"); endLine(); const fields = [];
      for (;;) { skipNl(); if (is("doc")) { p++; continue; } if (opt("kw", "end")) break; const f = eat("id"); eat("op", ":"); fields.push({ name: f.v, type: typ(), pos: pos(f) }); endLine(); }
      endLine(); prog.items.push({ kind: "record", name: n.v, fields, doc, pos: pos(t) }); continue;
    }
    if (opt("kw", "table")) {
      const n = eat("id"); eat("op", "("); const key = param(); eat("op", ")"); eat("op", "->"); const outs = outputs();
      let mode = "linear"; if (opt("kw", "step")) mode = "step"; else if (opt("kw", "linear")) mode = "linear";
      endLine(); const rows = [];
      for (;;) {
        skipNl(); if (is("doc")) { p++; continue; } if (opt("kw", "end")) break;
        const rt = peek(); const row = [];
        const num = () => { const neg = !!opt("op", "-"); const v = eat("num"); return (neg ? -1 : 1) * +v.v; };
        row.push(num()); while (opt("op", ",")) row.push(num());
        rows.push({ v: row, pos: pos(rt) }); endLine();
      }
      endLine(); prog.items.push({ kind: "table", name: n.v, key, outs, mode, rows, doc, pos: pos(t) }); continue;
    }
    err("expected module, use, const, fn, proc, record or table at the top level");
  }
  return prog;
}

// ------------------------------------------------------------------ types
export const T_INT = { k: "int" }, T_BOOL = { k: "bool" };
export const tReal = (dim) => ({ k: "real", dim });
export function typeText(t) {
  if (!t) return "?";
  switch (t.k) {
    case "int": return "int"; case "bool": return "bool";
    case "real": return `real[${dimText(t.dim)}]`;
    case "arr": return `${typeText(t.of)}[${t.n}]`;
    case "rec": return t.name;
    case "tuple": return `(${t.items.map(typeText).join(", ")})`;
    default: return t.k;
  }
}
// a type as it was written: real[kg m^2], vec3[m/s], int[7]
export function declText(t) {
  if (!t) return "?";
  switch (t.k) {
    case "real": return `real[${t.unit || "1"}]`;
    case "arr": return `${declText(t.of)}[${t.n}]`;
    case "rec": return t.name;
    default: return t.k;
  }
}
function typeEq(a, b) {
  if (a.k !== b.k) return false;
  if (a.k === "real") return dimEq(a.dim, b.dim);
  if (a.k === "arr") return a.n === b.n && typeEq(a.of, b.of);
  if (a.k === "rec") return a.name === b.name;
  return true;
}
function unitOf(text, pos) {
  // "kg m^2/s", "1/s", "m/s^2"
  let dim = DIMLESS, scale = 1;
  if (/\(/.test(text) && !/^[^/]*\/\s*\([^()]*\)\s*$/.test(text)) throw new PcodeError(`parentheses in a unit group its denominator only: [kg/(m s)], not [${text}]`, pos.line, pos.col, pos.file);
  const [num, den, extra] = text.replace(/[()]/g, " ").split("/");
  if (extra !== undefined) throw new PcodeError(`a unit has at most one /: [${text}]`, pos.line, pos.col, pos.file);
  for (const [part, sign] of [[num, 1], [den || "", -1]]) {
    for (const f of part.trim().split(/\s+/).filter(Boolean)) {
      const m = /^([A-Za-z]+|1)(?:\^(-?[0-9]+))?$/.exec(f);
      if (!m) throw new PcodeError(`not a unit factor: ${f} in [${text}]`, pos.line, pos.col, pos.file);
      const u = UNITS[m[1]];
      if (!u) throw new PcodeError(`no unit ${m[1]} (the units: ${Object.keys(UNITS).join(", ")})`, pos.line, pos.col, pos.file);
      const k = sign * (m[2] ? +m[2] : 1);
      dim = dimAdd(dim, dimMul(u[0], k)); scale *= Math.pow(u[1], k);
    }
  }
  return { dim, scale };
}

// ------------------------------------------------------------------ checker
const BUILTINS = new Set(["sqrt", "abs", "sin", "cos", "tan", "asin", "acos", "atan", "atan2", "exp", "log", "log10", "min", "max", "clamp",
  "floor", "ceil", "round", "sign", "fmod", "pow", "dot", "cross", "norm", "unit", "transpose", "real", "len", "hypot", "div", "rem"]);
const CONSTS = { pi: Math.PI };

// Check a set of parsed files together (they may call one another). Returns { program, errors }.
// program.fns / tables / records / consts by name; every expression node gets .ty
export function check(files) {
  const errors = [];
  const E = (msg, pos) => { errors.push(new PcodeError(msg, pos && pos.line, pos && pos.col, pos && pos.file)); };
  const prog = { fns: {}, tables: {}, records: {}, consts: {}, modules: {}, order: [] };
  // 1 declarations
  for (const f of files) {
    const mod = f.module || "main";
    prog.modules[mod] = prog.modules[mod] || { name: mod, doc: f.doc || [], items: [], file: f.file };
    for (const it of f.items) {
      it.module = mod;
      const all = { ...prog.fns, ...prog.tables, ...prog.records, ...prog.consts };
      if (all[it.name] || BUILTINS.has(it.name) || CONSTS[it.name] !== undefined) { E(`${it.name} is defined twice (or is a builtin)`, it.pos); continue; }
      ({ fn: prog.fns, proc: prog.fns, table: prog.tables, record: prog.records, const: prog.consts })[it.kind][it.name] = it;
      prog.modules[mod].items.push(it); prog.order.push(it);
    }
  }
  // 2 resolve types
  const resolve = (t, pos) => {
    if (!t) return t;
    if (t.k === "real" && t.unit !== undefined) { try { return { k: "real", dim: unitOf(t.unit, pos || t.pos || {}).dim, unit: t.unit }; } catch (e) { errors.push(e); return tReal(DIMLESS); } }
    if (t.k === "arr") return { k: "arr", n: t.n, of: resolve(t.of, pos) };
    if (t.k === "rec" && !prog.records[t.name]) { E(`no record ${t.name}`, t.pos || pos); }
    return t;
  };
  for (const r of Object.values(prog.records)) for (const f of r.fields) f.ty = resolve(f.type, f.pos);
  for (const it of prog.order) {
    if (it.kind === "fn" || it.kind === "proc") {
      for (const p of it.params) p.ty = resolve(p.type, p.pos);
      for (const o of it.outs) o.ty = resolve(o.type, o.pos);
      const names = [...it.params, ...it.outs].map((x) => x.name);
      const dup = names.find((n, i) => names.indexOf(n) !== i);
      if (dup) E(`${it.name}: ${dup} is named twice among the inputs and outputs`, it.pos);
    }
    if (it.kind === "table") {
      it.key.ty = resolve(it.key.type, it.key.pos);
      for (const o of it.outs) { o.ty = resolve(o.type, o.pos); if (o.ty.k !== "real") E(`table ${it.name}: every column is real`, o.pos); }
      if (it.key.ty.k !== "real") E(`table ${it.name}: the key is real`, it.key.pos);
      else if (!dimEq(it.key.ty.dim, it.outs[0].ty.dim)) E(`table ${it.name}: the first column is the key, so it has the key's dimension`, it.outs[0].pos);
      it.si = [];
      const scales = it.outs.map((o) => unitOf(o.type.unit, o.pos).scale);
      it.rows.forEach((r, ri) => {
        if (r.v.length !== it.outs.length) E(`table ${it.name}: the row has ${r.v.length} values, the table ${it.outs.length} columns`, r.pos);
        it.si.push(r.v.map((x, i) => x * (scales[i] || 1)));
        if (ri > 0 && !(r.v[0] > it.rows[ri - 1].v[0])) E(`table ${it.name}: keys rise strictly from row to row`, r.pos);
      });
      if (it.rows.length < (it.mode === "linear" ? 2 : 1)) E(`table ${it.name}: too few rows`, it.pos);
    }
  }
  // 3 bodies
  const scopeStack = [];
  const lookupVar = (name) => { for (let i = scopeStack.length - 1; i >= 0; i--) if (scopeStack[i].has(name)) return scopeStack[i].get(name); return null; };
  let cur = null;

  const numType = (t) => t && (t.k === "real" || t.k === "int");
  const scalarDim = (t) => (t.k === "int" ? DIMLESS : t.dim);
  const elem = (t) => { while (t.k === "arr") t = t.of; return t; };
  const isZero = (e) => (e.e === "num" && e.v === 0 && !e.unit) || (e.e === "arr" && e.items.every(isZero)) || (e.e === "un" && e.op === "-" && isZero(e.a));
  // can a value of type `got` (from expression e) go where `want` is wanted?
  const fits = (want, got, e) => {
    if (typeEq(want, got)) return true;
    if (e && isZero(e) && shapeEq(want, got)) return true;          // a bare 0 (or array of them) takes any dimension
    if (e && e.e === "num" && e.v === 0 && !e.unit && (want.k === "arr" || want.k === "rec")) { e.fill = want; return true; }   // 0 fills an array
    if (e && e.e === "arr" && !e.unit && want.k === "arr" && got.k === "arr" && want.n === got.n &&
        e.items.every((x) => fits(want.of, x.ty, x))) { e.ty = want; return true; }   // a literal takes the element type it is given to
    if (want.k === "real" && got.k === "int" && dimEq(want.dim, DIMLESS)) return true;
    if (want.k === "real" && got.k === "real" && e && e.e === "num" && !e.unit && dimEq(want.dim, DIMLESS)) return true;
    return false;
  };
  const shapeEq = (a, b) => {
    if (a.k === "arr" || b.k === "arr") return a.k === b.k && a.n === b.n && shapeEq(a.of, b.of);
    return numType(a) && numType(b);
  };
  function unify(a, b, ea, eb, op, pos) {   // for + - and comparisons: same dimension (a bare 0 adopts the other)
    if (isZero(ea) && shapeEq(a, b)) return b;
    if (isZero(eb) && shapeEq(a, b)) return a;
    if (a.k === "arr" || b.k === "arr") {
      if (!(a.k === "arr" && b.k === "arr" && a.n === b.n)) { E(`${op}: ${typeText(a)} and ${typeText(b)} differ in shape`, pos); return a; }
      return { k: "arr", n: a.n, of: unify(a.of, b.of, { e: "x" }, { e: "x" }, op, pos) };
    }
    if (a.k === "int" && b.k === "int") return T_INT;
    if (!numType(a) || !numType(b)) { E(`${op} needs numbers, not ${typeText(a)} and ${typeText(b)}`, pos); return tReal(DIMLESS); }
    const da = scalarDim(a), db = scalarDim(b);
    if (!dimEq(da, db)) E(`${op}: the units differ, [${dimText(da)}] and [${dimText(db)}]`, pos);
    return tReal(da);
  }

  function ty(e) { const t = tyInner(e); e.ty = t; return t; }
  function tyInner(e) {
    switch (e.e) {
      case "num": {
        if (e.unit !== null) { try { const u = unitOf(e.unit, e.pos); e.si = e.v * u.scale; return tReal(u.dim); } catch (er) { errors.push(er); return tReal(DIMLESS); } }
        e.si = e.v;
        return e.isInt ? T_INT : tReal(DIMLESS);
      }
      case "bool": return T_BOOL;
      case "var": {
        const v = lookupVar(e.name);
        if (v) { e.kind = v.kind; return v.ty; }
        if (prog.consts[e.name]) { e.kind = "const"; return prog.consts[e.name].ty || tReal(DIMLESS); }
        if (CONSTS[e.name] !== undefined) { e.kind = "builtin_const"; return tReal(DIMLESS); }
        E(`${e.name} is not defined here (a let declares a name)`, e.pos); return tReal(DIMLESS);
      }
      case "arr": {
        if (!e.items.length) { E("an empty array", e.pos); return tReal(DIMLESS); }
        const ts = e.items.map(ty);
        let base = ts.find((t, i) => !isZero(e.items[i])) || ts[0];
        if (e.unit) {
          let u; try { u = unitOf(e.unit, e.pos); } catch (er) { errors.push(er); u = { dim: DIMLESS, scale: 1 }; }
          e.scale = u.scale;
          if (ts.some((t) => !numType(t))) E("a unit after an array applies to numbers", e.pos);
          if (ts.some((t, i) => !isZero(e.items[i]) && !(t.k === "int" || (t.k === "real" && dimEq(t.dim, DIMLESS)))))
            E("an array with a unit holds plain numbers: [1, 2, 3] [m]", e.pos);
          return { k: "arr", n: e.items.length, of: tReal(u.dim) };
        }
        ts.forEach((t, i) => {
          if (!isZero(e.items[i]) && !(typeEq(t, base) || (numType(t) && numType(base) && dimEq(scalarDim(t), scalarDim(base)))))
            E(`array items differ: ${typeText(base)} and ${typeText(t)}`, e.items[i].pos);
        });
        if (base.k === "int" && !ts.every((t) => t.k === "int")) base = tReal(DIMLESS);   // all ints: an int array
        return { k: "arr", n: e.items.length, of: base };
      }
      case "index": {
        const a = ty(e.a), i = ty(e.i);
        if (i.k !== "int") E(`an index is an int, not ${typeText(i)}`, e.pos);
        if (a.k !== "arr") { E(`${typeText(a)} cannot be indexed`, e.pos); return tReal(DIMLESS); }
        if (e.i.e === "num" && (e.i.v < 0 || e.i.v >= a.n)) E(`index ${e.i.v} is outside 0..${a.n - 1}`, e.pos);
        return a.of;
      }
      case "field": {
        const a = ty(e.a);
        if (a.k !== "rec") { E(`${typeText(a)} has no fields`, e.pos); return tReal(DIMLESS); }
        const f = prog.records[a.name] && prog.records[a.name].fields.find((x) => x.name === e.f);
        if (!f) { E(`${a.name} has no field ${e.f}`, e.pos); return tReal(DIMLESS); }
        return f.ty;
      }
      case "un": {
        const a = ty(e.a);
        if (e.op === "not") { if (a.k !== "bool") E(`not needs a bool, not ${typeText(a)}`, e.pos); return T_BOOL; }
        if (!(numType(a) || (a.k === "arr" && numType(elem(a))))) E(`- needs a number or an array of them, not ${typeText(a)}`, e.pos);
        return a;
      }
      case "ifx": {
        const c = ty(e.c), a = ty(e.a), b = ty(e.b);
        if (c.k !== "bool") E(`if needs a bool, not ${typeText(c)}`, e.pos);
        if (a.k === "bool" || b.k === "bool") { if (!(a.k === "bool" && b.k === "bool")) E("if: both branches are bools or neither is", e.pos); return T_BOOL; }
        if (a.k === "rec" || b.k === "rec") { if (!typeEq(a, b)) E("if: the branches differ", e.pos); return a; }
        const t = unify(a, b, e.a, e.b, "if", e.pos);
        return t.k === "int" ? T_INT : t;
      }
      case "bin": return binTy(e);
      case "call": return callTy(e);
      default: E(`cannot use ${e.e} here`, e.pos); return tReal(DIMLESS);
    }
  }
  function binTy(e) {
    const op = e.op;
    if (op === "and" || op === "or") {
      const a = ty(e.a), b = ty(e.b);
      if (a.k !== "bool" || b.k !== "bool") E(`${op} needs bools, not ${typeText(a)} and ${typeText(b)}`, e.pos);
      return T_BOOL;
    }
    if (op === "^") {
      const a = ty(e.a); const b = ty(e.b);
      if (!numType(a)) { E(`^ needs a number, not ${typeText(a)}`, e.pos); return a; }
      const k = constInt(e.b);
      if (k === null) {
        E("the power after ^ is a whole-number literal (x^2, x^-3); for any other power write pow(x, y)", e.pos);
        return a;
      }
      e.k = k;
      if (k === 0) return a.k === "int" ? T_INT : tReal(DIMLESS);
      if (a.k === "int") return k > 0 ? T_INT : tReal(DIMLESS);
      return tReal(dimMul(a.dim, k));
    }
    const a = ty(e.a), b = ty(e.b);
    if (["==", "!=", "<", "<=", ">", ">="].includes(op)) {
      if (a.k === "bool" && b.k === "bool" && (op === "==" || op === "!=")) return T_BOOL;
      if (a.k === "arr" || b.k === "arr") E(`${op} compares numbers, not arrays`, e.pos);
      else unify(a, b, e.a, e.b, op, e.pos);
      return T_BOOL;
    }
    if (op === "+" || op === "-") {
      const t = unify(a, b, e.a, e.b, op, e.pos);
      e.shape = t.k === "arr" ? (t.of.k === "arr" ? "mat" : "vec") : "scalar";
      return t;
    }
    // * and /
    if (a.k === "arr" && b.k === "arr") {
      if (op === "/") { E("/ between arrays: divide by a number, or solve", e.pos); return a; }
      if (a.of.k === "arr" && b.of.k === "arr") {          // matrix x matrix
        if (a.of.n !== b.n) E(`matrix product: ${a.n}x${a.of.n} times ${b.n}x${b.of.n}`, e.pos);
        e.shape = "mm"; return { k: "arr", n: a.n, of: { k: "arr", n: b.of.n, of: tReal(dimAdd(elem(a).dim, elem(b).dim)) } };
      }
      if (a.of.k === "arr" && b.of.k !== "arr") {          // matrix x vector
        if (a.of.n !== b.n) E(`matrix times vector: ${a.n}x${a.of.n} times ${b.n}`, e.pos);
        e.shape = "mv"; return { k: "arr", n: a.n, of: tReal(dimAdd(elem(a).dim, scalarDim(elem(b)))) };
      }
      E("vector * vector: write dot(a, b), cross(a, b), or index them", e.pos); return a;
    }
    if (a.k === "arr" || b.k === "arr") {
      const [arr, sc, scE] = a.k === "arr" ? [a, b, e.b] : [b, a, e.a];
      if (!numType(sc)) { E(`${op}: ${typeText(sc)} does not scale an array`, e.pos); return arr; }
      if (op === "/" && b.k === "arr") { E("a number divided by an array", e.pos); return arr; }
      e.shape = arr.of.k === "arr" ? "ms" : "vs"; e.arrLeft = a.k === "arr";
      void scE;
      const sd = scalarDim(sc);
      const scaleEl = (t) => (t.k === "arr" ? { k: "arr", n: t.n, of: scaleEl(t.of) } : tReal(dimAdd(scalarDim(t), sd, op === "/" ? -1 : 1)));
      return scaleEl(arr);
    }
    if (!numType(a) || !numType(b)) { E(`${op} needs numbers, not ${typeText(a)} and ${typeText(b)}`, e.pos); return tReal(DIMLESS); }
    e.shape = "scalar";
    if (op === "*" && a.k === "int" && b.k === "int") return T_INT;
    return tReal(dimAdd(scalarDim(a), scalarDim(b), op === "/" ? -1 : 1));
  }
  function constInt(e) {
    if (e.e === "num" && e.isInt) return e.v;
    if (e.e === "un" && e.op === "-" && e.a.e === "num" && e.a.isInt) return -e.a.v;
    return null;
  }
  function want(e, t, what) {
    const got = ty(e);
    if (!fits(t, got, e)) E(`${what}: wants ${typeText(t)}, gets ${typeText(got)}`, e.pos);
    return got;
  }
  function callTy(e) {
    const name = e.f.includes(".") ? e.f.split(".")[1] : e.f;
    if (e.f.includes(".") && !prog.modules[e.f.split(".")[0]]) E(`no module ${e.f.split(".")[0]}`, e.pos);
    const args = e.args;
    if (BUILTINS.has(name) && !e.f.includes(".")) { e.builtin = true; return builtinTy(e, name, args); }
    const fn = prog.fns[name], tab = prog.tables[name];
    if (fn) {
      e.target = fn;
      if (fn.kind === "proc") E(`${name} is a proc (it keeps state): call it from a proc, as a statement-level let with its state`, e.pos);
      if (args.length !== fn.params.length) E(`${name} takes ${fn.params.length} inputs (${fn.params.map((p) => p.name).join(", ")}), given ${args.length}`, e.pos);
      args.forEach((a, i) => fn.params[i] && want(a, fn.params[i].ty, `${name}: input ${fn.params[i].name}`));
      return fn.outs.length === 1 ? fn.outs[0].ty : { k: "tuple", items: fn.outs.map((o) => o.ty) };
    }
    if (tab) {
      e.target = tab; e.table = true;
      if (args.length !== 1) E(`table ${name} takes one key`, e.pos);
      else want(args[0], tab.key.ty, `table ${name}: key`);
      return tab.outs.length === 1 ? tab.outs[0].ty : { k: "tuple", items: tab.outs.map((o) => o.ty) };
    }
    if (prog.records[name]) {
      if (args.length) E(`${name}() makes a record with every field zero; it takes no inputs`, e.pos);
      e.record = name; return { k: "rec", name };
    }
    E(`no function ${e.f}`, e.pos); args.forEach(ty); return tReal(DIMLESS);
  }
  function builtinTy(e, name, args) {
    const n = (k) => { if (args.length !== k) E(`${name} takes ${k} input(s), given ${args.length}`, e.pos); };
    const ts = args.map(ty);
    const sc = (t, i) => { if (!t || !numType(t)) { E(`${name}: input ${i + 1} is a number, not ${typeText(t)}`, e.pos); return DIMLESS; } return scalarDim(t); };
    const dl = (t, i) => { const d = sc(t, i); if (!dimEq(d, DIMLESS)) E(`${name} takes a plain number (an angle in rad is one), not [${dimText(d)}]`, e.pos); };
    const vec = (t, i) => { if (!t || t.k !== "arr" || t.of.k === "arr" || !numType(t.of)) { E(`${name}: input ${i + 1} is a vector, not ${typeText(t)}`, e.pos); return { n: 3, dim: DIMLESS }; } return { n: t.n, dim: scalarDim(t.of) }; };
    switch (name) {
      case "sqrt": { n(1); const d = sc(ts[0], 0); if (d.some((x) => x % 2)) E(`sqrt of [${dimText(d)}] has no unit`, e.pos); return tReal(dimMul(d, 0.5)); }
      case "abs": case "floor": case "ceil": case "round": { n(1); if (ts[0] && ts[0].k === "int") return T_INT; return tReal(sc(ts[0], 0)); }
      case "sign": n(1); sc(ts[0], 0); return tReal(DIMLESS);
      case "sin": case "cos": case "tan": case "asin": case "acos": case "atan": case "exp": case "log": case "log10":
        n(1); dl(ts[0], 0); return tReal(DIMLESS);
      case "atan2": case "hypot": case "fmod": {
        n(2); const d = sc(ts[0], 0), d2 = sc(ts[1], 1);
        if (!dimEq(d, d2) && !isZero(args[1]) && !isZero(args[0])) E(`${name}: the inputs differ in unit, [${dimText(d)}] and [${dimText(d2)}]`, e.pos);
        return name === "atan2" ? tReal(DIMLESS) : tReal(d);
      }
      case "pow": n(2); dl(ts[0], 0); dl(ts[1], 1); return tReal(DIMLESS);
      case "min": case "max": {
        if (args.length < 2) E(`${name} takes two inputs or more`, e.pos);
        let t = ts[0]; for (let i = 1; i < ts.length; i++) t = unify(t, ts[i], args[0], args[i], name, e.pos);
        if (t.k === "arr") E(`${name} takes numbers`, e.pos);
        e.allInt = ts.every((x) => x.k === "int");
        return e.allInt ? T_INT : t;
      }
      case "clamp": { n(3); let t = unify(ts[0], ts[1], args[0], args[1], name, e.pos); t = unify(t, ts[2], args[0], args[2], name, e.pos); return t.k === "int" ? T_INT : t; }
      case "real": n(1); if (!ts[0] || ts[0].k !== "int") E("real() turns an int into a real", e.pos); return tReal(DIMLESS);
      case "div": case "rem": n(2); if (!(ts[0] && ts[0].k === "int" && ts[1] && ts[1].k === "int")) E(`${name} takes two ints (whole-number division, truncated as in C)`, e.pos); return T_INT;
      case "len": n(1); if (!ts[0] || ts[0].k !== "arr") E("len() takes an array", e.pos); return T_INT;
      case "dot": { n(2); const a = vec(ts[0], 0), b = vec(ts[1], 1); if (a.n !== b.n) E(`dot: lengths ${a.n} and ${b.n}`, e.pos); return tReal(dimAdd(a.dim, b.dim)); }
      case "cross": { n(2); const a = vec(ts[0], 0), b = vec(ts[1], 1); if (a.n !== 3 || b.n !== 3) E("cross takes two 3-vectors", e.pos); return { k: "arr", n: 3, of: tReal(dimAdd(a.dim, b.dim)) }; }
      case "norm": { n(1); const a = vec(ts[0], 0); return tReal(a.dim); }
      case "unit": { n(1); const a = vec(ts[0], 0); return { k: "arr", n: a.n, of: tReal(DIMLESS) }; }
      case "transpose": {
        n(1); const t = ts[0];
        if (!t || t.k !== "arr" || t.of.k !== "arr") { E("transpose takes a matrix", e.pos); return t; }
        return { k: "arr", n: t.of.n, of: { k: "arr", n: t.n, of: t.of.of } };
      }
    }
    E(`no builtin ${name}`, e.pos); return tReal(DIMLESS);
  }

  // statements; `assigned` tracks outputs certainly set on every path
  function checkBlock(stmts, assigned, inLoop) {
    scopeStack.push(new Map());
    for (const s of stmts) checkStmt(s, assigned, inLoop);
    scopeStack.pop();
  }
  function declare(name, t, pos, kind = "local") {
    if (lookupVar(name)) { E(`${name} is already defined here; a name is declared once`, pos); }
    scopeStack[scopeStack.length - 1].set(name, { ty: t, kind });
  }
  function checkStmt(s, assigned, inLoop) {
    switch (s.s) {
      case "let": {
        const t = ty(s.e);
        let declTy = s.type ? resolve(s.type, s.pos) : null;
        if (s.names.length > 1) {
          if (t.k !== "tuple" || t.items.length !== s.names.length) { E(`let ${s.names.join(", ")}: the right side gives ${t.k === "tuple" ? t.items.length : 1} value(s)`, s.pos); s.names.forEach((n) => declare(n, tReal(DIMLESS), s.pos)); return; }
          s.names.forEach((n, i) => declare(n, t.items[i], s.pos)); return;
        }
        if (t.k === "tuple") { E(`${s.e.f} gives ${t.items.length} values: let one name for each`, s.pos); declare(s.names[0], t.items[0], s.pos); return; }
        if (declTy && !fits(declTy, t, s.e)) E(`let ${s.names[0]}: declared ${typeText(declTy)}, given ${typeText(t)}`, s.pos);
        let vt = declTy || t;
        if (!declTy && vt.k === "int" && s.e.e === "num") vt = T_INT;
        if (!declTy && isZero(s.e) && vt.k !== "int") E(`let ${s.names[0]} = 0 needs its type: let ${s.names[0]}: real[unit] = 0`, s.pos);
        s.vty = vt;
        declare(s.names[0], vt, s.pos); return;
      }
      case "state": {
        if (!cur || cur.kind !== "proc") E("state is kept by a proc, not a fn", s.pos);
        if (scopeStack.length !== 2) E("state is declared at the top of the proc's body", s.pos);
        const st = resolve(s.type, s.pos); s.vty = st;
        const t = ty(s.e);
        if (!fits(st, t, s.e)) E(`state ${s.name}: declared ${typeText(st)}, starts at ${typeText(t)}`, s.pos);
        if (!isConstExpr(s.e)) E(`state ${s.name} starts at a constant`, s.pos);
        cur.states.push(s);
        declare(s.name, st, s.pos, "state"); return;
      }
      case "set": {
        const t = ty(s.e);
        const tts = s.targets.map((lv) => lvTy(lv));
        if (s.targets.length > 1) {
          if (t.k !== "tuple" || t.items.length !== s.targets.length) E(`${s.targets.length} targets, the right side gives ${t.k === "tuple" ? t.items.length : 1}`, s.pos);
          else tts.forEach((tt, i) => tt && !typeEq(tt, t.items[i]) && E(`target ${i + 1}: ${typeText(tt)}, given ${typeText(t.items[i])}`, s.pos));
        } else if (tts[0] && !fits(tts[0], t, s.e)) E(`${lvName(s.targets[0])}: is ${typeText(tts[0])}, given ${typeText(t)}`, s.pos);
        for (const lv of s.targets) if (lv.e === "var") assigned.add(lv.name); else if (rootVar(lv)) touched.add(rootVar(lv));
        return;
      }
      case "if": {
        const sets = [];
        for (const arm of s.arms) {
          const c = ty(arm.c); if (c.k !== "bool") E(`if needs a bool, not ${typeText(c)}`, arm.c.pos);
          const a = new Set(assigned); checkBlock(arm.body, a, inLoop); sets.push(a);
        }
        if (s.els) { const a = new Set(assigned); checkBlock(s.els, a, inLoop); sets.push(a); }
        if (s.els) for (const n of sets[0]) if (sets.every((x) => x.has(n))) assigned.add(n);
        return;
      }
      case "for": {
        const a = ty(s.a), b = ty(s.b);
        if (a.k !== "int" || b.k !== "int") E("for runs over ints: for i in 0 .. n", s.pos);
        scopeStack.push(new Map()); declare(s.v, T_INT, s.pos, "loop");
        checkBlock(s.body, new Set(assigned), true); scopeStack.pop(); return;
      }
      case "settle": {
        const n = ty(s.n); if (n.k !== "int") E("settle max N: N is an int", s.pos);
        const inner = new Set(assigned);
        checkBlock(s.body, inner, true);
        for (const x of inner) assigned.add(x);          // the body runs at least once
        scopeStack.push(new Map());
        const c = ty(s.c); if (c.k !== "bool") E(`until needs a bool, not ${typeText(c)}`, s.pos);
        scopeStack.pop();
        if (s.els) checkBlock(s.els, new Set(assigned), inLoop);
        return;
      }
    }
  }
  let touched = new Set();
  const rootVar = (lv) => (lv.e === "var" ? lv.name : rootVar(lv.a));
  const lvName = (lv) => (lv.e === "var" ? lv.name : lv.e === "field" ? `${lvName(lv.a)}.${lv.f}` : `${lvName(lv.a)}[…]`);
  function lvTy(lv) {
    if (lv.e === "var") {
      const v = lookupVar(lv.name);
      if (!v) { E(`${lv.name} is not declared: let ${lv.name} = …`, lv.pos); return null; }
      if (v.kind === "input") E(`${lv.name} is an input; inputs are not changed (copy it: let x = ${lv.name})`, lv.pos);
      if (v.kind === "loop") E(`${lv.name} is the loop's counter`, lv.pos);
      lv.kind = v.kind; lv.ty = v.ty; return v.ty;
    }
    const base = lvTy(lv.a); if (!base) return null;
    if (lv.e === "index") {
      const i = ty(lv.i); if (i.k !== "int") E("an index is an int", lv.pos);
      if (base.k !== "arr") { E(`${typeText(base)} cannot be indexed`, lv.pos); return null; }
      lv.ty = base.of; return base.of;
    }
    if (base.k !== "rec") { E(`${typeText(base)} has no fields`, lv.pos); return null; }
    const f = prog.records[base.name] && prog.records[base.name].fields.find((x) => x.name === lv.f);
    if (!f) { E(`${base.name} has no field ${lv.f}`, lv.pos); return null; }
    lv.ty = f.ty; return f.ty;
  }
  function constNum(e) {   // the SI value of a constant bound (a literal, maybe negated, or pi)
    if (e.e === "num") return e.si;
    if (e.e === "un" && e.op === "-") { const v = constNum(e.a); return v === null ? null : -v; }
    if (e.e === "var" && e.kind === "builtin_const") return Math.PI;
    if (e.e === "bin" && ["*", "/"].includes(e.op)) { const a = constNum(e.a), b = constNum(e.b); return a === null || b === null ? null : e.op === "*" ? a * b : a / b; }
    E("a range bound is a constant", e.pos); return null;
  }
  function isConstExpr(e) {
    switch (e.e) {
      case "num": case "bool": return true;
      case "arr": return e.items.every(isConstExpr);
      case "un": return isConstExpr(e.a);
      case "bin": return isConstExpr(e.a) && isConstExpr(e.b);
      case "var": return e.kind === "const" || e.kind === "builtin_const";
      case "call": return !!e.record || (e.builtin && e.args.every(isConstExpr));
      default: return false;
    }
  }

  for (const c of Object.values(prog.consts)) { scopeStack.length = 0; scopeStack.push(new Map()); c.ty = ty(c.e); if (!isConstExpr(c.e)) E(`const ${c.name} is a constant expression`, c.pos); }
  for (const it of prog.order) {
    if (it.kind !== "fn" && it.kind !== "proc") continue;
    cur = it; it.states = []; touched = new Set();
    scopeStack.length = 0;
    const top = new Map();
    for (const p of it.params) top.set(p.name, { ty: p.ty, kind: "input" });
    for (const o of it.outs) top.set(o.name, { ty: o.ty, kind: "output" });
    scopeStack.push(top);
    const assigned = new Set();
    checkBlock(it.body, assigned, false);
    for (const o of it.outs) {
      if (!assigned.has(o.name) && !(touched.has(o.name) && o.ty.k !== "real" && o.ty.k !== "int" && o.ty.k !== "bool"))
        E(`${it.name}: output ${o.name} is not set on every path`, it.pos);
    }
    for (const p of it.params) if (p.range) {
      // a bound with no unit is in the input's own unit: t_slew: real[s] in 1 .. 600
      scopeStack.length = 0; scopeStack.push(new Map());
      // on an array the range bounds each element
      let baseType = p.type; while (baseType.k === "arr") baseType = baseType.of;
      const baseTy = elem(p.ty);
      const scale = baseType.k === "real" ? (() => { try { return unitOf(baseType.unit, p.pos).scale; } catch (er) { return 1; } })() : 1;
      p.range.forEach((r) => {
        const t = ty(r);
        const plain = (x) => (x.e === "num" && !x.unit) || (x.e === "un" && x.op === "-" && plain(x.a));
        if (plain(r) && numType(baseTy)) { r.bound = constNum(r) * scale; return; }
        if (!fits(baseTy, t, r)) E(`${it.name}: the range of ${p.name} is ${typeText(baseTy)}, not ${typeText(t)}`, r.pos);
        r.bound = constNum(r);
      });
      if (p.range[0].bound !== null && !(p.range[0].bound < p.range[1].bound)) E(`${it.name}: the range of ${p.name} runs low .. high`, p.pos);
    }
  }
  cur = null;
  // calls between fns: no recursion (flight code)
  const calls = {};
  const walkCalls = (node, out) => {
    if (!node || typeof node !== "object") return;
    if (Array.isArray(node)) { node.forEach((n) => walkCalls(n, out)); return; }
    if (node.e === "call" && node.target && node.target.kind !== "table") out.add(node.target.name);
    for (const k of Object.keys(node)) if (!["target", "ty", "pos", "vty"].includes(k)) walkCalls(node[k], out);
  };
  for (const it of prog.order) if (it.kind === "fn" || it.kind === "proc") { const s = new Set(); walkCalls(it.body, s); calls[it.name] = s; }
  const seen = {}, onStack = {};
  const dfs = (n, path) => {
    if (onStack[n]) { E(`recursion: ${[...path, n].join(" -> ")} (flight code does not recurse)`, prog.fns[n].pos); return; }
    if (seen[n]) return; seen[n] = onStack[n] = true;
    for (const m of calls[n] || []) dfs(m, [...path, n]);
    onStack[n] = false;
  };
  Object.keys(calls).forEach((n) => dfs(n, []));
  prog.calls = calls;
  return { program: prog, errors };
}

// ------------------------------------------------------------------ interpreter
// Semantics every translation follows (docs/PSEUDOCODE_V2.md, "Numbers"):
//   x^k is repeated multiplication left to right (x^-k = 1/(x^k)); min/max/abs/clamp are written out;
//   dot and matrix products sum left to right; unit(a) = a / max(norm(a), 1e-30).
export const rt = {
  ipow(x, k) { if (k === 0) return 1; let r = x; for (let i = 1; i < Math.abs(k); i++) r = r * x; return k < 0 ? 1 / r : r; },
  min(a, b) { return b < a ? b : a; },
  max(a, b) { return b > a ? b : a; },
  abs(x) { return x < 0 ? -x : x === 0 ? 0 : x; },
  clamp(x, lo, hi) { return rt.min(rt.max(x, lo), hi); },
  sign(x) { return x > 0 ? 1 : x < 0 ? -1 : 0; },
  dot(a, b) { let s = a[0] * b[0]; for (let i = 1; i < a.length; i++) s = s + a[i] * b[i]; return s; },
  cross(a, b) { return [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]; },
  norm(a) { return Math.sqrt(rt.dot(a, a)); },
  unit(a) { const n = rt.max(rt.norm(a), 1e-30); return a.map((x) => x / n); },
  mv(M, v) { return M.map((row) => rt.dot(row, v)); },
  mm(A, B) { return A.map((row) => B[0].map((_, j) => { let s = row[0] * B[0][j]; for (let k = 1; k < B.length; k++) s = s + row[k] * B[k][j]; return s; })); },
  tr(M) { return M[0].map((_, j) => M.map((row) => row[j])); },
  round(x) { return x < 0 ? -Math.round(-x) : Math.round(x); },          // half away from zero, as C round()
  fmod(x, y) { return x % y; },
  lookup(tab, x) {
    const r = tab.si, k = r.map((row) => row[0]);
    if (tab.mode === "step") { let i = 0; while (i + 1 < r.length && k[i + 1] <= x) i++; return r[i].slice(); }
    const xc = rt.clamp(x, k[0], k[k.length - 1]);
    let i = 0; while (i + 2 < r.length && k[i + 1] <= xc) i++;
    const t = (xc - k[i]) / (k[i + 1] - k[i]);
    return r[i].map((a, j) => a + t * (r[i + 1][j] - a));
  },
};
const clone = (v) => (Array.isArray(v) ? v.map(clone) : v && typeof v === "object" ? Object.fromEntries(Object.entries(v).map(([k, x]) => [k, clone(x)])) : v);
function zeroOf(t, prog) {
  switch (t.k) {
    case "int": case "real": return 0; case "bool": return false;
    case "arr": return Array.from({ length: t.n }, () => zeroOf(t.of, prog));
    case "rec": return Object.fromEntries(prog.records[t.name].fields.map((f) => [f.name, zeroOf(f.ty, prog)]));
  }
  return 0;
}
export class RunError extends Error {}

export function makeInterpreter(prog) {
  const constVals = {};
  const ev = (e, env) => {
    switch (e.e) {
      case "num": return e.fill ? zeroOf(e.fill, prog) : e.si;
      case "bool": return e.v;
      case "var": {
        if (e.kind === "const") { if (!(e.name in constVals)) constVals[e.name] = ev(prog.consts[e.name].e, new Map()); return constVals[e.name]; }
        if (e.kind === "builtin_const") return Math.PI;
        const v = env.get(e.name); if (v === undefined) throw new RunError(`${e.name} has no value yet (line ${e.pos.line})`); return v;
      }
      case "arr": { const v = e.items.map((x) => ev(x, env)); return e.scale !== undefined ? v.map((x) => x * e.scale) : v; }
      case "index": {
        const a = ev(e.a, env), i = ev(e.i, env);
        if (!(i >= 0 && i < a.length)) throw new RunError(`index ${i} outside 0..${a.length - 1} (line ${e.pos.line})`);
        return clone(a[i]);
      }
      case "field": return clone(ev(e.a, env)[e.f]);
      case "un": { const a = ev(e.a, env); if (e.op === "not") return !a; return neg(a); }
      case "ifx": return ev(e.c, env) ? ev(e.a, env) : ev(e.b, env);
      case "bin": return bin(e, env);
      case "call": return call(e, env);
    }
    throw new RunError(`cannot evaluate ${e.e}`);
  };
  const neg = (a) => (Array.isArray(a) ? a.map(neg) : -a);
  const ew = (a, b, f) => (Array.isArray(a) ? a.map((x, i) => ew(x, b[i], f)) : f(a, b));
  const sc = (a, s, f) => (Array.isArray(a) ? a.map((x) => sc(x, s, f)) : f(a, s));
  function bin(e, env) {
    const op = e.op;
    if (op === "and") return ev(e.a, env) && ev(e.b, env);
    if (op === "or") return ev(e.a, env) || ev(e.b, env);
    const a = ev(e.a, env);
    if (op === "^") return rt.ipow(a, e.k);
    const b = ev(e.b, env);
    switch (op) {
      case "==": return a === b; case "!=": return a !== b; case "<": return a < b; case "<=": return a <= b; case ">": return a > b; case ">=": return a >= b;
      case "+": return Array.isArray(a) || Array.isArray(b) ? ew(zeroLike(a, b), zeroLike(b, a), (x, y) => x + y) : a + b;
      case "-": return Array.isArray(a) || Array.isArray(b) ? ew(zeroLike(a, b), zeroLike(b, a), (x, y) => x - y) : a - b;
    }
    if (e.shape === "mv") return rt.mv(a, b);
    if (e.shape === "mm") return rt.mm(a, b);
    if (e.shape === "vs" || e.shape === "ms") {
      if (op === "/") return sc(a, b, (x, s) => x / s);
      return e.arrLeft ? sc(a, b, (x, s) => x * s) : sc(b, a, (x, s) => s * x);
    }
    return op === "*" ? a * b : a / b;
  }
  const zeroLike = (a, other) => (Array.isArray(a) || !Array.isArray(other) ? a : other.map((o) => zeroLike(a, o)));
  function call(e, env) {
    if (e.record) return zeroOf({ k: "rec", name: e.record }, prog);
    const args = e.args.map((a) => ev(a, env));
    if (e.builtin) {
      const f = e.f;
      switch (f) {
        case "sqrt": return Math.sqrt(args[0]);
        case "abs": return rt.abs(args[0]);
        case "floor": return Math.floor(args[0]); case "ceil": return Math.ceil(args[0]); case "round": return rt.round(args[0]);
        case "sign": return rt.sign(args[0]);
        case "sin": case "cos": case "tan": case "asin": case "acos": case "atan": case "exp": case "log": case "log10": return Math[f](args[0]);
        case "atan2": return Math.atan2(args[0], args[1]);
        case "hypot": return Math.sqrt(args[0] * args[0] + args[1] * args[1]);
        case "fmod": return rt.fmod(args[0], args[1]);
        case "pow": return Math.pow(args[0], args[1]);
        case "min": return args.reduce((m, x) => rt.min(m, x));
        case "max": return args.reduce((m, x) => rt.max(m, x));
        case "clamp": return rt.clamp(args[0], args[1], args[2]);
        case "real": return args[0];
        case "div": if (args[1] === 0) throw new RunError(`div by zero (line ${e.pos.line})`); return Math.trunc(args[0] / args[1]);
        case "rem": if (args[1] === 0) throw new RunError(`rem by zero (line ${e.pos.line})`); return args[0] % args[1];
        case "len": return args[0].length;
        case "dot": return rt.dot(args[0], args[1]);
        case "cross": return rt.cross(args[0], args[1]);
        case "norm": return rt.norm(args[0]);
        case "unit": return rt.unit(args[0]);
        case "transpose": return rt.tr(args[0]);
      }
    }
    if (e.table) { const r = rt.lookup(e.target, args[0]); return e.target.outs.length === 1 ? r[0] : r; }
    const r = runFn(e.target, args, null);
    return e.target.outs.length === 1 ? r[0] : r;
  }
  function exec(stmts, env, st) {
    const local = new Map(env);
    const writeBack = () => { for (const k of env.keys()) env.set(k, local.get(k)); };
    for (const s of stmts) {
      switch (s.s) {
        case "let": {
          const v = ev(s.e, local);
          if (s.names.length > 1) s.names.forEach((n, i) => local.set(n, clone(v[i])));
          else local.set(s.names[0], clone(v));
          break;
        }
        case "state": local.set(s.name, st.vals[s.name]); break;
        case "set": {
          const v = ev(s.e, local);
          if (s.targets.length > 1) s.targets.forEach((lv, i) => assign(lv, clone(v[i]), local));
          else assign(s.targets[0], clone(v), local);
          break;
        }
        case "if": {
          let done = false;
          for (const arm of s.arms) if (ev(arm.c, local)) { exec(arm.body, local, st); done = true; break; }
          if (!done && s.els) exec(s.els, local, st);
          break;
        }
        case "for": {
          const a = ev(s.a, local), b = ev(s.b, local);
          for (let i = a; i < b; i++) { const m = new Map(local); m.set(s.v, i); exec(s.body, m, st); for (const k of local.keys()) local.set(k, m.get(k)); }
          break;
        }
        case "settle": {
          const n = ev(s.n, local); let k = 0;
          for (;;) {
            exec(s.body, local, st); k++;
            if (ev(s.c, local)) break;
            if (k >= n) { if (s.els) exec(s.els, local, st); break; }
          }
          break;
        }
      }
    }
    if (st) for (const name of Object.keys(st.vals)) if (local.has(name)) st.vals[name] = local.get(name);
    writeBack();
  }
  function assign(lv, v, env) {
    if (lv.e === "var") { env.set(lv.name, v); return; }
    const path = []; let x = lv;
    while (x.e !== "var") { path.unshift(x); x = x.a; }
    const root = env.get(x.name);
    let o = root;
    for (let i = 0; i < path.length; i++) {
      const p = path[i]; const key = p.e === "index" ? ev(p.i, env) : p.f;
      if (p.e === "index" && !(key >= 0 && key < o.length)) throw new RunError(`index ${key} outside 0..${o.length - 1} (line ${p.pos.line})`);
      if (i === path.length - 1) o[key] = v; else o = o[key];
    }
    env.set(x.name, root);
  }
  function runFn(fn, args, state) {
    const env = new Map();
    fn.params.forEach((p, i) => env.set(p.name, clone(args[i])));
    fn.outs.forEach((o) => env.set(o.name, zeroOf(o.ty, prog)));
    let st = null;
    if (fn.kind === "proc") {
      st = state || newState(fn);
      for (const s of fn.states) env.set(s.name, st.vals[s.name]);
    }
    exec(fn.body, env, st);
    return fn.outs.map((o) => env.get(o.name));
  }
  function newState(fn) {
    const vals = {};
    for (const s of fn.states) vals[s.name] = clone(ev(s.e, new Map()));
    return { fn: fn.name, vals };
  }
  return {
    // call a fn with SI inputs; returns the outputs, in order
    call(name, args, state = null) {
      const fn = prog.fns[name];
      if (!fn) throw new RunError(`no fn ${name}`);
      if (args.length !== fn.params.length) throw new RunError(`${name} takes ${fn.params.length} inputs`);
      return runFn(fn, args, state);
    },
    newState(name) { return newState(prog.fns[name]); },
    lookup(name, x) { return rt.lookup(prog.tables[name], x); },
  };
}

// ------------------------------------------------------------------ one call: parse and check
export function compile(sources) {   // [{ file, text }]
  const parsed = [], errors = [];
  for (const s of sources) {
    try { parsed.push(parse(s.text, s.file)); } catch (e) { if (e instanceof PcodeError) errors.push(e); else throw e; }
  }
  if (errors.length) return { program: null, errors };
  const r = check(parsed);
  return r;
}

// a deterministic generator for test vectors (xorshift64*)
export function prng(seed) {
  let s = BigInt.asUintN(64, BigInt(seed) * 0x9E3779B97F4A7C15n + 1n);
  return () => {
    s ^= s >> 12n; s ^= BigInt.asUintN(64, s << 25n); s ^= s >> 27n;
    const r = BigInt.asUintN(64, s * 0x2545F4914F6CDD1Dn);
    return Number(r >> 11n) / 9007199254740992;
  };
}
