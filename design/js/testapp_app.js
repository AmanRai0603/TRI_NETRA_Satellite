// testapp_app.js -- a group's test app (docs/RELEASE_PLAN.md P10; tools/groupcode.py deliver): the
// group's computing rows, each run on its own test vectors twice, in the pseudocode's interpreter
// (design/js/pcode.js, what the node app's "Try it" runs) and in WebAssembly built from the Rust the
// pseudocode was translated to (engine/crates/adcs-relations), side by side; any row can be tried on
// numbers of one's own. The rows the group has no code for yet are listed with their owner team.
// Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
import { compile, makeInterpreter } from "./pcode.js";
import { h, fill, button, field, banner, badge, table, section, toolbar, kv, shell, choice } from "./tn_ui.js";

const { main, status } = shell("TRI-NETRA Test app", "A group's computing rows, run in the interpreter and in WebAssembly from the generated code. Nothing leaves this computer.");
const app = { data: null, I: null, wasm: null, results: [], ready: null };
const setStatus = (t) => fill(status, h("span", { "data-testid": "status" }, t));

function modOf(w, fn) { return (w.shared || []).includes(fn) ? "shared" : w.group; }

/** Run `module::fn` in the WebAssembly module on flat inputs; null when it cannot. */
function wasmCall(name, xs) {
  const k = app.data.names.indexOf(name);
  if (!app.wasm || k < 0) return null;
  const ex = app.wasm.exports;
  new Float64Array(ex.memory.buffer, ex.input(), xs.length).set(xs);
  const n = ex.call(k, xs.length);
  return n < 0 ? null : Array.from(new Float64Array(ex.memory.buffer, ex.output(), n));
}

function interpCall(fn, xs) {
  const y = app.I.call(fn, xs);
  return (Array.isArray(y) ? y : [y]).flat(Infinity).map(Number);
}

const close = (got, want, tol) => Number.isFinite(got) && Math.abs(got - want) <= tol * Math.max(Math.abs(want), want === 0 ? 1 : 0);
const fmt = (x) => (x === null || x === undefined ? "—" : Number(x).toPrecision(10));

function runAll() {
  const w = app.data.wire, out = [];
  for (const r of w.rows) {
    for (const v of r.vectors) {
      let ip = null, wa = null, err = null;
      try { ip = interpCall(r.fn, v.inputs)[0]; } catch (e) { err = e.message; }
      wa = (wasmCall(`${modOf(w, r.fn)}::${r.fn}`, v.inputs) || [null])[0];
      const agree = ip !== null && wa !== null && Math.abs(ip - wa) <= 1e-12 * Math.max(1e-300, Math.abs(ip));
      out.push({ node: r.id, label: r.label, vector: v.name, expected: v.expected, tolerance: v.tolerance, interp: ip, wasm: wa, err,
        pass: close(ip, v.expected, v.tolerance) && close(wa, v.expected, v.tolerance) && agree, outside: v.outside });
    }
  }
  app.results = out;
  return out;
}

function tryIt(r) {
  const w = app.data.wire;
  const fields = r.params.map(([n, ty]) => field(`${n} (${ty})`, { mono: true, help: "In SI units, as the pseudocode takes it; a vector as numbers separated by commas.", testid: `in-${n}` }));
  const out = h("div", { "data-testid": "try-out" });
  const go = () => {
    try {
      const xs = fields.flatMap((f) => f.value.split(",").map((x) => Number(x.trim())));
      if (xs.some((x) => !Number.isFinite(x))) throw new Error("every input needs a number");
      const a = interpCall(r.fn, xs.length === r.params.length ? xs : xs), b = wasmCall(`${modOf(w, r.fn)}::${r.fn}`, xs);
      fill(out, kv([["Interpreter", a.map(fmt).join(", ")], ["WebAssembly (generated Rust)", b ? b.map(fmt).join(", ") : "not callable with plain numbers (a flight-state function)"]]));
    } catch (e) { fill(out, banner("error", "Not run.", e.message)); }
  };
  return section(`Try ${r.id} · ${r.label}`, ...fields.map((f) => f.el), toolbar(button("Run", go, { kind: "primary", testid: "try-run" })), out);
}

function draw() {
  const w = app.data.wire, res = runAll();
  const ok = res.filter((x) => x.pass).length;
  let tried = null;
  const pick = choice("Row", w.rows.map((r) => [r.id, `${r.id} · ${r.label}`]), { help: "Run one row on numbers of your own.", testid: "pick",
    onChange: (id) => { fill(tried, tryIt(w.rows.find((r) => r.id === id))); } });
  tried = h("div", {});
  fill(main,
    section(`${w.group} · ${w.label || ""}`, kv([["Owner team", w.owner_team || "—"], ["From", w.source], ["Computing rows with code", String(w.rows.length)],
      ["Rows with no code yet", String(w.without_code.length)], ["Test vectors", String(res.length)]])),
    !res.length ? banner("info", "No test vector yet.", "Its rows run, but none has a test vector with an answer to compare: their authors add them in the node app.")
      : ok === res.length ? banner("ok", `All ${res.length} test vectors pass,`, "in the interpreter and in WebAssembly, and the two agree.")
        : banner("error", `${res.length - ok} of ${res.length} test vectors fail.`, "See the table: each row says where it differs."),
    section("Test vectors", table([{ key: "node", label: "Node", mono: true }, { key: "vector", label: "Vector" }, { key: "expected", label: "Expected", render: (x) => fmt(x.expected) },
      { key: "interp", label: "Interpreter", render: (x) => fmt(x.interp) }, { key: "wasm", label: "WebAssembly", render: (x) => fmt(x.wasm) },
      { key: "pass", label: "", render: (x) => badge(x.pass ? "pass" : x.err || "fail", x.pass ? "ok" : "error") }], res, { empty: "None yet.", testid: "vectors" })),
    w.rows.length ? section("Try a row", pick.el, tried) : null,
    section("Rows", table([{ key: "id", label: "Node", mono: true }, { key: "label", label: "Label" }, { key: "fn", label: "Function", mono: true, render: (r) => r.fn || "—" },
      { key: "release", label: "From" }], w.rows, { empty: "No computing row has pseudocode yet.", testid: "rows" })),
    w.without_code.length ? section("No code yet", table([{ key: "id", label: "Node", mono: true }, { key: "label", label: "Label" }], w.without_code, { testid: "without" }),
      h("p", { class: "tn-dim" }, `Their authors write the pseudocode in the node app; owner team: ${w.owner_team || "—"}.`)) : null);
  if (w.rows.length) fill(tried, tryIt(w.rows[0]));
  setStatus(`${w.group}: ${ok} of ${res.length} test vectors pass`);
}

async function start() {
  app.data = JSON.parse(document.getElementById("tn-testapp").textContent);
  if (!app.data) { fill(main, banner("info", "An empty test app.", "tools/groupcode.py deliver puts a group into this page.")); return; }
  const srcs = (app.data.sources || []).filter((s) => s.text);
  if (srcs.length) {
    const { program, errors } = compile(srcs);
    if (errors.length) { fill(main, banner("error", "The pseudocode does not compile.", errors.map((e) => e.message).join("\n"))); return; }
    app.I = makeInterpreter(program);
  }
  const b64 = document.getElementById("tn-groups-wasm").textContent.trim();
  if (b64) {
    const bytes = Uint8Array.from(atob(b64), (c) => c.charCodeAt(0));
    app.wasm = (await WebAssembly.instantiate(bytes, {})).instance;
  }
  draw();
}

app.ready = start();
window.tnTestApp = { app, ready: app.ready, runAll, wasmCall, interpCall };
