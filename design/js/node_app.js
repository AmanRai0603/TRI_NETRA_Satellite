// node_app.js -- TRI-NETRA Node, the author's app (docs/RELEASE_PLAN.md P5): open a node file from
// the design folder (or one file on its own), fill it step by step, see every problem the checker
// will find while typing, preview it as the main application will show it, mark it ready, and sign
// it. The steps, fields and checks are node_model.js; the preview is node_view.js; the file is
// tnfile.js; every control is tn_ui.js.
// Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
import { Journal, MemoryFolder, FileRefused, openFile } from "./tnfile.js";
import { loadIndex, readersOutside, readersInside, STRUCTURE, NODES } from "./structure.js";
import { h, fill, button, fileButton, field, banner, badge, table, section, toolbar, spacer, kv, dialog, ask, toast, shell, tabs, choice, checks, insertAt, download } from "./tn_ui.js";
import { KIND_LABEL, FIXED, CHOOSABLE, STEPS, stepsFor, fieldShown, readDoc, list, setField, setInputs, setFixtures, check, progress, evidenceDebt,
  tryIt, pcodeCheck, parsePasted, results, standing, fingerprint, adoptSpec, SPEC_ORIGIN } from "./node_model.js";
import { renderNode, mathText } from "./node_view.js";
import { CATALOG } from "./node_catalog.js";

const canFolders = typeof window.showDirectoryPicker === "function";
const { main, status } = shell("TRI-NETRA Node", "Your node, step by step: what it answers, how, and what shows it is right. Nothing leaves this computer.");
const journal = new Journal();
const session = crypto.getRandomValues(new Uint32Array(2)).join("-");
const app = { SQL: null, root: null, index: null, cur: null, doc: null, tab: 0, who: null, profile: null, viaDownload: false, urls: [] };

function remembered(k) { try { return localStorage.getItem(k); } catch (e) { return null; } }
function remember(k, v) { try { localStorage.setItem(k, v); } catch (e) { /* private window */ } }
function fmtTime(t) { return t ? new Date(t).toLocaleString() : "—"; }
function fmtSize(n) { return n < 1024 ? `${n} B` : n < 1048576 ? `${(n / 1024).toFixed(1)} KB` : `${(n / 1048576).toFixed(2)} MB`; }
function setStatus(t) { fill(status, h("span", { "data-testid": "status" }, t)); }
function render(...parts) { fill(main, ...parts); }

async function who() {
  if (app.who) return app.who;
  app.who = remembered("trinetra.who");
  while (!app.who) app.who = await ask("Your name", "Name", { help: "Written beside every change you save. There are no accounts: Drive's record of who saved each file backs it up." });
  remember("trinetra.who", app.who);
  return app.who;
}

async function guard(what, fn) {
  try { return await fn(); }
  catch (e) {
    const msg = e instanceof FileRefused ? e.message : `${what}: ${e.message || e}`;
    toast(msg.split("\n")[0], "error");
    main.prepend(banner("error", `${what} did not happen.`, msg));
    return undefined;
  }
}

// ------------------------------------------------------------------ opening
async function pickFolder() {
  const dir = await guard("Opening the folder", () => window.showDirectoryPicker({ id: "trinetra-design", mode: "readwrite" }));
  if (dir) await useFolder(dir);
}

async function useFolder(root) {
  await who();
  const ok = await guard("Opening the design folder", async () => { await root.getDirectoryHandle(NODES); await root.getDirectoryHandle(STRUCTURE); return true; });
  if (!ok) { main.prepend(banner("error", "Not a design folder.", `A design folder holds ${STRUCTURE}/ and ${NODES}/. Pick the folder that holds both, or open one node file on its own.`)); return; }
  app.root = root; app.viaDownload = false;
  app.index = await loadIndex(app.SQL, root);
  await showList();
}

async function openOneFile(files) {
  await who();
  const f = files[0];
  app.root = null; app.index = null; app.viaDownload = !canFolders;
  app.single = new MemoryFolder("this computer", { [f.name]: new Uint8Array(await f.arrayBuffer()) });
  await openNode(f.name.replace(/\.node\.tndb$/, ""), { dir: app.single, name: f.name });
}

async function showList(filter = "") {
  if (app.cur) await closeNode();
  const idx = app.index;
  const mine = [];
  for (const [gid, g] of idx.groups) for (const n of g.nodes.values()) {
    const author = g.memberNodes.get(n.id)?.author;
    mine.push({ id: n.id, label: n.label, group: gid, kind: n.kind, state: n.state, author: author || "" });
  }
  const q = filter.trim().toLowerCase();
  const rows = mine.filter((r) => !q || r.id.toLowerCase().includes(q) || (r.label || "").toLowerCase().includes(q) || r.group === q || (r.author || "").toLowerCase() === q);
  const myOwn = mine.filter((r) => r.author && r.author === app.who);
  const search = field("Find a node", { value: filter, placeholder: "id, label, group, or an author's name", testid: "search", onChange: (v) => showList(v) });
  render(toolbar(...topButtons()),
    myOwn.length ? section(`Issued to you (${myOwn.length})`, table(cols(), myOwn, { onRow: (r) => openNode(r.id), testid: "mine" })) : banner("info", "", `No node is issued to ${app.who} yet: your group lead issues nodes in the group app. You can open any node below.`),
    section(`All nodes (${rows.length} of ${mine.length})`, search.el, table(cols(), rows.slice(0, 200), { onRow: (r) => openNode(r.id), testid: "nodes", empty: "No node matches." }),
      rows.length > 200 ? h("p", { class: "tn-dim" }, `${rows.length - 200} more: narrow the search.`) : null));
  setStatus(`${mine.length} nodes in ${idx.groups.size} groups`);
}

const cols = () => [{ key: "id", label: "Node", mono: true }, { key: "label", label: "Label" }, { key: "group", label: "Group" }, { key: "author", label: "Author" }, { key: "state", label: "State" }];

function topButtons() {
  const b = [];
  if (canFolders) b.push(button("Open design folder…", pickFolder, { kind: app.root ? "default" : "primary", testid: "open-folder" }));
  b.push(fileButton("Open one node file…", openOneFile, { accept: ".tndb", testid: "open-file" }));
  b.push(spacer(), button(`You: ${app.who || "?"}`, async () => { const n = await ask("Your name", "Name", { value: app.who || "" }); if (n) { app.who = n; remember("trinetra.who", n); } }, { kind: "quiet" }));
  return b;
}

async function openNode(id, { dir = null, name = null, readOnly = false, takeOver = false } = {}) {
  if (app.cur) await closeNode();
  const nd = dir || await app.root.getDirectoryHandle(NODES);
  const s = await guard(`Opening ${id}`, () => openFile({ SQL: app.SQL, dir: nd, name: name || `${id}.node.tndb`, who: app.who, session, profile: app.profile, journal, locks: navigator.locks || null, readOnly, takeOver }));
  if (!s) return;
  if (s.kind !== "node") { toast("That is not a node file", "error"); await s.close(); return; }
  app.cur = s; app.tab = 0;
  await refresh();
}

async function closeNode() {
  const s = app.cur;
  if (!s) return;
  if (s.dirty && !s.readOnly) {
    const v = await dialog("Unsaved changes", h("p", {}, "This node has changes that are not saved. They stay kept in this browser either way, and are offered again when you open it."), [["Keep them for later", "keep"], ["Save now", "save", "primary"]]);
    if (v === "save") await guard("Saving", () => s.save());
  }
  for (const u of app.urls) URL.revokeObjectURL(u);
  app.urls = [];
  app.cur = null; app.doc = null;
  await s.close();
}

// what the design folder adds: who reads the node, its group, its contract, the lead's comments
function context() {
  const d = app.doc, idx = app.index;
  const ctx = { pictureUrl };
  if (!idx) return ctx;
  const g = idx.groups.get(d.node.group_id);
  ctx.groupLabel = g ? `${g.id} · ${g.label}` : d.node.group_id;
  const inside = readersInside(idx, d.node.id).map((id) => ({ id, group: d.node.group_id }));
  const outside = [...readersOutside(idx, d.node.id)].flatMap(([gid, ids]) => ids.map((id) => ({ id, group: gid })));
  ctx.readers = [...inside, ...outside];
  ctx.contracts = g ? g.contracts.filter((c) => c.node === d.node.id) : [];
  ctx.nodes = new Map();
  for (const [gid, gg] of idx.groups) for (const n of gg.nodes.values()) {
    const spec = CATALOG.rows[n.id];
    ctx.nodes.set(n.id, { label: n.label, group: gid, layer: n.layer, kind: n.kind, quantity: spec ? spec[2] : null, unit: spec ? spec[3] : null });
  }
  return ctx;
}

function pictureUrl(name) {
  const st = app.cur.db.prepare("SELECT mime, bytes FROM attachment WHERE name = ?");
  try {
    st.bind([name]);
    if (!st.step()) return "";
    const [mime, bytes] = st.get();
    const u = URL.createObjectURL(new Blob([bytes], { type: mime || "application/octet-stream" }));
    app.urls.push(u);
    return u;
  } finally { st.free(); }
}

// ------------------------------------------------------------------ the node
async function refresh() {
  app.doc = readDoc(app.cur);
  app.ctx = context();
  app.standing = await standing(app.doc);
  showNode();
}

async function change(summary, fn) {
  app.busy = (app.busy || 0) + 1;
  try {
    const r = await guard(summary, () => app.cur.change(summary, fn));
    await refresh();
    return r;
  } finally { app.busy--; }
}

// a re-render after one field's change must not take what the person is typing in the next one
function keepTyping() {
  const el = document.activeElement;
  const id = el && el.getAttribute && el.getAttribute("data-testid");
  if (!id || !("value" in el) || el.type === "checkbox") return () => {};
  const value = el.value, start = el.selectionStart, end = el.selectionEnd;
  return () => {
    const again = main.querySelector(`[data-testid="${CSS.escape(id)}"]`);
    if (!again || !("value" in again)) return;
    if (again.value !== value) again.value = value;
    again.focus();
    try { again.setSelectionRange(start, end); } catch (e) { /* a select has no caret */ }
  };
}

function showNode() {
  const restore = keepTyping();
  try { drawNode(); } finally { restore(); }
}

function drawNode() {
  const s = app.cur, d = app.doc, k = d.kind;
  const problems = check(d, app.ctx);
  const prog = progress(d);
  const count = (step) => problems.filter((p) => p.step === step && p.level === "!").length;
  const steps = stepsFor(k);
  const items = [["Home", () => homeView(problems, prog)],
    ...steps.map((st) => [`${st.title}${count(st.id) ? ` (${count(st.id)})` : ""}`, () => stepView(st, problems)]),
    [`Review${problems.filter((p) => p.level === "!").length ? ` (${problems.filter((p) => p.level === "!").length})` : ""}`, () => reviewView(problems)],
    ["Preview", () => renderNode(d, app.ctx, app.standing)]];
  const t = tabs(items, { testid: "steps" });
  render(
    toolbar(button("◂ Nodes", async () => { if (app.root) await showList(); else { await closeNode(); start(); } }, { kind: "quiet", testid: "back" }),
      button("Undo", () => app.cur.undo().then(refresh), { disabled: s.readOnly || !s.undoStack.length, testid: "undo" }),
      button("Redo", () => app.cur.redo().then(refresh), { disabled: s.readOnly || !s.redoStack.length, testid: "redo" }),
      spacer(), badge(app.standing.state, { checked: "ok", ready: "ok", draft: "warn", shell: "" }[app.standing.state]),
      button(app.viaDownload ? "Save (download)" : "Save", () => save(), { kind: "primary", disabled: s.readOnly || !s.dirty, testid: "save" })),
    s.readOnly ? banner("warn", "Read-only.", s.readOnlyWhy + ".") : null,
    ...s.notes.map((n) => banner("info", "", n)),
    s.recovery && !s.recovery.stale ? banner("warn", "Unsaved work from before.", `Changes from ${fmtTime(s.recovery.at)} were never saved.`, button("Restore them", async () => { await guard("Restoring", () => s.restore()); await refresh(); }, { kind: "primary", testid: "restore" })) : null,
    !app.index ? banner("info", "", "Opened on its own: who reads it, its group and its contract are shown when the node is opened from the design folder.") : null,
    section(`${d.node.id} · ${d.node.label || ""}`, kv([["Group · stage", `${app.ctx.groupLabel || d.node.group_id || "—"} · ${d.node.stage || "—"}`],
      ["Kind", k ? KIND_LABEL[k] : "not chosen yet: choose it under Identity"], ["Author", d.node.author || "not issued"]])),
    t.el);
  t.show(Math.min(app.tab, items.length - 1));
  for (const [i, b] of [...t.el.querySelectorAll(".tn-tab")].entries()) b.addEventListener("click", () => { app.tab = i; });
  const bad = problems.filter((p) => p.level === "!").length;
  setStatus(`${s.readOnly ? "read-only" : "editing"} · ${bad} problem(s) to fix · ${s.dirty ? `${s.pending.length} unsaved change(s), kept in this browser until saved` : s.lastSaved ? `saved ${fmtTime(s.lastSaved.at)}, read back and checked` : "no changes"}`);
}

function goTo(stepId) {
  const steps = stepsFor(app.doc.kind);
  const i = steps.findIndex((s) => s.id === stepId);
  app.tab = i >= 0 ? i + 1 : steps.length + 1;
  showNode();
}

function homeView(problems, prog) {
  const d = app.doc, s = app.cur;
  const parts = [];
  const nSpec = Object.keys(d.fromSpec || {}).filter((k) => k === "inputs" ? !readDocInputsHave() : d.origin[k] === SPEC_ORIGIN).length;
  if (nSpec && !s.readOnly) parts.push(banner("info", "The spec already says some of this.", `${nSpec} field(s) come from the spec package and are shown as starting values. Take them in to make them this node's own; you can change any of them after.`,
    button("Start from the spec", () => change("start from the spec", (db) => adoptSpec(db, d)), { kind: "primary", testid: "adopt" })));
  // contract changes waiting
  const waiting = (app.ctx.contracts || []).filter((c) => Number(c.version) > Number(d.node.contract_version || 0));
  if (waiting.length) parts.push(banner("warn", "The contract changed.", `Your group lead published ${waiting.map((c) => `${c.output} version ${c.version}`).join(", ")} for this node since you last acknowledged it (you have version ${d.node.contract_version || 0}). Check the output still matches, then acknowledge.`,
    button("Acknowledge", () => change("acknowledge the contract", (db) => db.run("UPDATE node SET contract_version = ?", [Math.max(...waiting.map((c) => Number(c.version)))])), { testid: "ack" })));
  const rows = stepsFor(d.kind).map((st) => ({ id: st.id, step: st.title, filled: prog[st.id] ? `${prog[st.id][0]} of ${prog[st.id][1]}` : "—",
    problems: problems.filter((p) => p.step === st.id && p.level === "!").length }));
  parts.push(section("Your progress", table([{ key: "step", label: "Step" }, { key: "filled", label: "Filled" },
    { key: "problems", label: "To fix", render: (r) => (r.problems ? badge(String(r.problems), "error") : badge("none", "ok")) }], rows, { onRow: (r) => goTo(r.id), testid: "progress" })));
  const debt = evidenceDebt(d);
  if (debt.length) parts.push(section("Evidence it still owes", h("ul", { class: "tn-problems", "data-testid": "home-debt" }, debt.map((x) => h("li", {}, x.text)))));
  // comments from the lead, and the author's replies
  const comments = d.comments;
  parts.push(section(`Comments (${comments.filter((c) => !c.resolved).length} open)`,
    table([{ key: "at", label: "When", render: (r) => fmtTime(r.at) }, { key: "by", label: "Who" }, { key: "body", label: "Comment" }, { key: "resolved", label: "", render: (r) => (r.resolved ? badge("resolved", "ok") : "") }], comments, { empty: "No comments. Your group lead's comments appear here." }),
    s.readOnly ? null : toolbar(button("Write a comment or a reply…", async () => {
      const body = await ask("Comment", "Comment", {});
      if (body) await change("comment", (db) => db.run("INSERT INTO comment VALUES (?, ?, ?, ?, ?, ?, 0)", [`c-${Date.now().toString(36)}`, new Date().toISOString(), app.who, d.node.id, comments.length ? comments[comments.length - 1].id : null, body]));
    }, { testid: "comment" }))));
  return parts;
}

function readDocInputsHave() { return app.cur.query("SELECT count(*) FROM input")[0][0] > 0; }

// ------------------------------------------------------------------ a step
function stepView(st, problems) {
  const d = app.doc, ro = app.cur.readOnly || (FIXED.has(d.kind) && st.id === "identity");
  const mine = problems.filter((p) => p.step === st.id);
  const parts = [];
  if (mine.length) parts.push(h("ul", { class: "tn-problems", "data-testid": `problems-${st.id}` }, mine.map((p) => h("li", {}, badge(p.level === "!" ? "fix" : "note", p.level === "!" ? "error" : "warn"), ` ${p.code} `, p.text))));
  for (const fd of st.fields) if (fieldShown(fd, d.kind, d.node)) parts.push(editor(fd, ro));
  if (st.id === "pseudocode") parts.push(tryItView());
  if (FIXED.has(d.kind) && st.id === "identity") parts.unshift(banner("info", "", `${KIND_LABEL[d.kind]}. Its rows are set by the tree; here you can explain it, add pictures, and send feedback or a request.`));
  return section(st.title, ...parts);
}

function meta(fd) {
  const fromSpec = app.doc.origin[fd.key] === SPEC_ORIGIN;
  return h("div", { class: "tn-field-meta" }, fd.ask, fd.why ? ` ${fd.why}` : "", fd.example ? h("span", {}, " For example: ", h("i", {}, fd.example)) : null,
    fromSpec ? h("span", {}, " ", badge("from the spec", "warn")) : null);
}

function editor(fd, ro) {
  const d = app.doc, v = d.content[fd.key] ?? "";
  const tid = `f-${fd.key}`;
  const set = (val) => change(`${fd.label.toLowerCase()}`, (db) => setField(db, fd.key, val, app.who));
  switch (fd.type) {
    case "choice": {
      const c = choice(fd.label, [["", "—"], ...fd.choices.map((x) => [x, x])], { value: v, testid: tid, onChange: set, disabled: ro });
      return h("div", { class: "tn-stack" }, c.el, meta(fd));
    }
    case "multi": {
      const c = checks(fd.label, fd.choices.map((x) => [x, x]), { values: list(d, fd.key), testid: tid, onChange: (vals) => set(JSON.stringify(vals)), disabled: ro });
      return h("div", { class: "tn-stack" }, c.el, meta(fd));
    }
    case "list": {
      const x = field(fd.label, { value: list(d, fd.key).join("\n"), multiline: true, readOnly: ro, testid: tid, help: "One per line.", onChange: (val) => set(JSON.stringify(val.split("\n").map((l) => l.trim()).filter(Boolean))) });
      return h("div", { class: "tn-stack" }, x.el, meta(fd));
    }
    case "pairs": {
      const keys = fd.pair;
      const x = field(fd.label, { value: list(d, fd.key).map((o) => keys.map((kk) => o[kk] ?? "").join(" | ")).join("\n"), multiline: true, readOnly: ro, testid: tid,
        help: `One per line: ${keys.join(" | ")}.`, onChange: (val) => set(JSON.stringify(val.split("\n").filter((l) => l.trim()).map((l) => Object.fromEntries(keys.map((kk, i) => [kk, (l.split("|")[i] || "").trim()]))))) });
      return h("div", { class: "tn-stack" }, x.el, meta(fd));
    }
    case "inputs": return inputsEditor(fd, ro);
    case "fixtures": return fixturesEditor(fd, ro);
    case "results": return resultsEditor(fd, ro);
    case "pictures": return picturesEditor(fd, ro);
    case "code": {
      const live = h("div", { "data-testid": "pcode-problems" });
      const showLive = (text) => {
        const p = pcodeCheck(text, d.content["output.symbol"], d.inputs);
        fill(live, p.problems.length ? h("ul", { class: "tn-problems" }, p.problems.map((x) => h("li", {}, badge("fix", "error"), " ", x))) : badge("the language's checker finds nothing", "ok"));
      };
      const x = field(fd.label, { value: v, multiline: true, mono: true, readOnly: ro, testid: tid, rows: 10, onInput: showLive, onChange: set });
      showLive(v);
      return h("div", { class: "tn-stack" }, x.el, meta(fd), live);
    }
    default: {
      const x = field(fd.label, { value: v, multiline: fd.type === "long", readOnly: ro, testid: tid, mono: fd.type === "number", onChange: set });
      if (fd.type === "number") x.input.setAttribute("inputmode", "decimal");
      const parts = [x.el, meta(fd)];
      if (fd.equation) parts.splice(1, 0, equationHelper(x.input, ro));
      if (fd.sources) parts.push(h("div", { class: "tn-field-meta" }, "Known sources: ", CATALOG.sources.map((s) => s[0]).join(", ")));
      return h("div", { class: "tn-stack" }, ...parts);
    }
  }
}

// the equation helper: symbols and forms to insert, and the equation as it will read
function equationHelper(input, ro) {
  const reads = h("p", { class: "tn-equation", "data-testid": "equation-reads" }, mathText(input.value));
  input.addEventListener("input", () => fill(reads, mathText(input.value)));
  const syms = [["α", "alpha"], ["β", "beta"], ["γ", "gamma"], ["δ", "delta"], ["θ", "theta"], ["λ", "lambda"], ["μ", "mu"], ["π", "pi"], ["ρ", "rho"], ["σ", "sigma"], ["τ", "tau"], ["φ", "phi"], ["ω", "omega"], ["Δ", "Delta"],
    ["x²", "^2"], ["x³", "^3"], ["xᵢ", "_i"], ["√", "sqrt()"], ["a/b", " / "], ["·", " * "], ["|x|", "abs()"], ["sin", "sin()"], ["cos", "cos()"], ["exp", "exp()"]];
  return h("div", { class: "tn-stack" }, h("div", { class: "tn-palette", "data-testid": "palette" }, syms.map(([label, text]) => button(label, () => insertAt(input, text), { disabled: ro, title: `insert ${text}` }))),
    h("div", { class: "tn-field-meta" }, "It reads:"), reads);
}

function inputsEditor(fd, ro) {
  const d = app.doc, nodes = app.ctx.nodes;
  const x = field(fd.label, { value: d.inputs.map((i) => [i.name, i.from_node, i.quantity || ""].join(" | ")).join("\n"), multiline: true, readOnly: ro, testid: "f-inputs", mono: true,
    help: "One per line: binding | node id | quantity (the quantity is filled from the node when it is known).",
    onChange: (val) => change("inputs", (db) => setInputs(db, val.split("\n").filter((l) => l.trim()).map((l) => {
      const [name, from_node, quantity] = l.split("|").map((p) => (p || "").trim());
      const spec = CATALOG.rows[from_node];
      return { name, from_node, quantity: quantity || (spec ? spec[2] : "") || null };
    }))) });
  const resolved = table([{ key: "name", label: "Binding", mono: true }, { key: "from_node", label: "Reads", mono: true },
    { key: "label", label: "Which is", render: (r) => (nodes && nodes.get(r.from_node) ? `${nodes.get(r.from_node).label} (${nodes.get(r.from_node).group})` : nodes ? badge("not a node", "error") : "—") },
    { key: "quantity", label: "Quantity" }], d.inputs, { empty: "Reads nothing yet." });
  return h("div", { class: "tn-stack" }, x.el, meta(fd), resolved);
}

function fixturesEditor(fd, ro) {
  const d = app.doc;
  const line = (x) => [x.name, Object.entries(x.inputs).map(([a, b]) => `${a}=${b}`).join(", "), x.expected ?? "", x.tolerance ?? "", x.provenance, x.source, x.where].join(" | ");
  const x = field(fd.label, { value: d.fixtures.map(line).join("\n"), multiline: true, readOnly: ro, testid: "f-fixtures", mono: true, rows: 5,
    help: `One per line: name | inputs (a=1, b=2) | expected | tolerance (relative) | where the answer comes from (${CATALOG.provenance.join(", ")}) | source | page, table or figure.`,
    onChange: (val) => change("test vectors", (db) => setFixtures(db, val.split("\n").filter((l) => l.trim()).map((l) => {
      const [name, ins, expected, tolerance, provenance, source, where] = l.split("|").map((p) => (p || "").trim());
      const inputs = Object.fromEntries((ins || "").split(",").map((p) => p.split("=").map((q) => q.trim())).filter((p) => p[0]).map(([a, b]) => [a, b === undefined || b === "" ? null : Number(b)]));
      return { name, inputs, expected: expected === "" ? null : Number(expected), tolerance: tolerance === "" ? null : Number(tolerance), provenance, source, where };
    }))) });
  return h("div", { class: "tn-stack" }, x.el, meta(fd));
}

function tryItView() {
  const d = app.doc;
  if (!d.fixtures.length) return banner("info", "Try it", "Add test vectors under Evidence: the pseudocode is then run on each, here, against its expected answer.");
  const t = tryIt(d);
  if (t.problems.length) return banner("warn", "Try it", "Fix the pseudocode first.");
  return section("Try it: the pseudocode on its test vectors", table([{ key: "name", label: "Vector", mono: true }, { key: "got", label: "It gives", mono: true, render: (r) => (r.error ? r.error : String(r.got)) },
    { key: "expected", label: "Expected", mono: true }, { key: "pass", label: "", render: (r) => badge(r.pass ? "reproduces it" : "does not", r.pass ? "ok" : "error") }], t.runs, { testid: "try-it" }));
}

function resultsEditor(fd, ro) {
  const d = app.doc, r = results(d);
  const paste = field("Paste here", { multiline: true, mono: true, readOnly: ro, testid: "f-results-paste", rows: 5,
    help: "Copy the cells in Excel (or any table) and paste them here: the first row names the columns, a second row of units is optional." });
  const preview = h("div", { "data-testid": "results-preview" });
  const show = (t) => fill(preview, t ? table(t.columns.map((col, j) => ({ key: String(j), label: col.unit ? `${col.name} [${col.unit}]` : col.name, mono: true, render: (row) => row[j] })), t.rows, { empty: "No rows." }) : "");
  paste.input.addEventListener("input", () => show(parsePasted(paste.input.value)));
  show(r);
  return h("div", { class: "tn-stack" }, meta(fd), paste.el, ro ? null : toolbar(
    button("Keep these results", () => { const t = parsePasted(paste.input.value); if (!t) { toast("Nothing pasted", "error"); return; } change("results", (db) => setField(db, "results.table", JSON.stringify(t), app.who)); }, { kind: "primary", testid: "keep-results" }),
    r ? button("Remove the results", () => change("results removed", (db) => setField(db, "results.table", null, app.who)), { kind: "danger" }) : null), preview);
}

function picturesEditor(fd, ro) {
  const d = app.doc;
  const rows = d.attachments.map((a) => { let m = {}; try { m = JSON.parse(d.content[`pictures.${a.name}`] || "{}"); } catch (e) { m = {}; } return { ...a, caption: m.caption || "" }; });
  return h("div", { class: "tn-stack" }, meta(fd),
    table([{ key: "name", label: "File", mono: true }, { key: "caption", label: "Caption" }, { key: "size", label: "Size", render: (r) => fmtSize(r.size) }], rows, { empty: "None yet.", testid: "pictures" }),
    ro ? null : toolbar(fileButton("Add a picture or document…", (files) => pictureWizard(files[0]), { testid: "add-picture" })));
}

// the picture wizard: a picture too big is made smaller (scaled and re-encoded) until it fits the
// cap; then a caption and what it shows, for readers who cannot see it
async function pictureWizard(file) {
  const cap = app.cur.format.max_attachment_bytes;
  let bytes = new Uint8Array(await file.arrayBuffer()), mime = file.type || "application/octet-stream", name = file.name, note = "";
  if (bytes.length > cap) {
    if (!/^image\/(png|jpeg|webp|bmp)$/.test(mime)) { toast(`${file.name} is ${fmtSize(bytes.length)}; a file may be ${fmtSize(cap)} at most`, "error"); return; }
    const small = await shrink(file, cap);
    if (!small) { toast(`${file.name} could not be made smaller than ${fmtSize(cap)}`, "error"); return; }
    note = `Made smaller to fit: ${fmtSize(bytes.length)} → ${fmtSize(small.bytes.length)} (${small.width}×${small.height}, JPEG).`;
    bytes = small.bytes; mime = "image/jpeg"; name = name.replace(/\.[A-Za-z0-9]+$/, "") + ".jpg";
  }
  const cap1 = field("Caption", { testid: "f-caption" }), alt = field("What it shows (for a reader who cannot see it)", { multiline: true, testid: "f-alt" });
  const v = await dialog("Add a picture", h("div", { class: "tn-stack" }, h("p", {}, `${name}, ${fmtSize(bytes.length)}. ${note}`), cap1.el, alt.el), [["Cancel", null], ["Add it", () => [cap1.value, alt.value], "primary"]]);
  if (!v) return;
  await change(`picture ${name}`, (db) => {
    db.run("INSERT OR REPLACE INTO attachment (name, mime, size, bytes) VALUES (?, ?, ?, ?)", [name, mime, bytes.length, bytes]);
    setField(db, `pictures.${name}`, JSON.stringify({ caption: v[0], alt: v[1] }), app.who);
  });
}

async function shrink(file, cap) {
  const bmp = await createImageBitmap(file);
  let scale = Math.min(1, 2000 / Math.max(bmp.width, bmp.height));
  for (let i = 0; i < 8; i++) {
    const w = Math.max(1, Math.round(bmp.width * scale)), hh = Math.max(1, Math.round(bmp.height * scale));
    const cv = new OffscreenCanvas(w, hh);
    cv.getContext("2d").drawImage(bmp, 0, 0, w, hh);
    for (const q of [0.85, 0.7, 0.55]) {
      const blob = await cv.convertToBlob({ type: "image/jpeg", quality: q });
      if (blob.size <= cap) return { bytes: new Uint8Array(await blob.arrayBuffer()), width: w, height: hh };
    }
    scale *= 0.7;
  }
  return null;
}

// ------------------------------------------------------------------ review, ready, sign
function reviewView(problems) {
  const d = app.doc, st = app.standing, s = app.cur;
  const bad = problems.filter((p) => p.level === "!"), notes = problems.filter((p) => p.level === "i");
  const debt = evidenceDebt(d);
  const prob = (list_, kind) => h("ul", { class: "tn-problems" }, list_.map((p) => h("li", {}, badge(kind === "!" ? "fix" : "note", kind === "!" ? "error" : "warn"), " ", `${p.code} `, p.text, " ",
    button("Go there", () => goTo(p.step), { kind: "quiet" }))));
  return [
    section("Where it stands", kv([["State", badge(st.state, st.state === "checked" || st.state === "ready" ? "ok" : "warn")],
      ["Ready", st.ready ? `${st.ready.name}, ${fmtTime(st.ready.at)}${st.readyStale ? " (changed since: mark it ready again)" : ""}` : "not yet"],
      ["Checked by", st.checked ? `${st.checked.name}, ${fmtTime(st.checked.at)}${st.checkedStale ? " (changed since: the signature no longer covers it)" : ""}` : "nobody: it runs as UNCONFIRMED (W01)"]])),
    section(`To fix (${bad.length})`, bad.length ? prob(bad, "!") : banner("ok", "", "Nothing to fix: the checker's rules all hold.")),
    notes.length ? section(`Notes (${notes.length})`, prob(notes, "i")) : null,
    debt.length ? section("Evidence it still owes", h("ul", { class: "tn-problems" }, debt.map((x) => h("li", {}, x.text)))) : null,
    s.readOnly ? null : section("Ready, sign, ask",
      h("p", {}, "Mark it ready when nothing is left to fix: your group lead then sees it as ready. An engineer who has checked the relation, its sources and its values signs it as \"Checked by\"; any later change takes the signature off."),
      toolbar(
        button("Mark ready", () => sign("author ready", app.who, "ready for the group's review"), { kind: "primary", disabled: bad.length > 0 || (st.ready && !st.readyStale), testid: "ready" }),
        button("Sign as checked…", () => signChecked(), { disabled: bad.length > 0, testid: "sign" }),
        button("Ask for a contract change…", () => contractRequest(), { testid: "contract-request" })),
      d.requests.length ? table([{ key: "at", label: "When", render: (r) => fmtTime(r.at) }, { key: "body", label: "Asked" }, { key: "state", label: "State" }], d.requests, { testid: "requests" }) : null),
  ];
}

async function sign(role, name, statement) {
  const fp = await fingerprint(app.doc);
  await change(role === "author ready" ? "marked ready" : `checked by ${name}`, (db) =>
    db.run("INSERT INTO signature VALUES (?, ?, ?, ?)", [role, name, new Date().toISOString(), JSON.stringify({ statement, fingerprint: fp })]));
  toast(role === "author ready" ? "Marked ready" : `Signed by ${name}`);
}

async function signChecked() {
  const name = field("Your name", { value: app.who || "", testid: "f-checker" }), st = field("What you checked", { multiline: true, testid: "f-statement",
    help: "The relation against its source, the values, the test vectors: what you stand behind." });
  const v = await dialog("Checked by", h("div", { class: "tn-stack" }, h("p", {}, "A typed name, not a proof: it stands behind exactly what is written now, and comes off when anything changes."), name.el, st.el),
    [["Cancel", null], ["Sign", () => [name.value.trim(), st.value.trim()], "primary"]]);
  if (!v || !v[0]) return;
  if (app.doc.node.author && v[0].toLowerCase() === String(app.doc.node.author).toLowerCase()) { toast("The author cannot check their own node: another engineer signs", "error"); return; }
  if (!v[1]) { toast("Say what you checked", "error"); return; }
  await sign("checked by", v[0], v[1]);
}

async function contractRequest() {
  const out = field("The output", { value: app.doc.content["output.symbol"] || "", mono: true, testid: "f-output" }), body = field("What should change, and why", { multiline: true, testid: "f-body" });
  const v = await dialog("Ask for a contract change", h("div", { class: "tn-stack" }, h("p", {}, "The contract is what other groups rely on: your group lead decides, and tells its readers."), out.el, body.el),
    [["Cancel", null], ["Send to the lead", () => [out.value.trim(), body.value.trim()], "primary"]]);
  if (!v || !v[1]) return;
  await change("contract change request", (db) => db.run("INSERT INTO change_request VALUES (?, ?, ?, ?, ?, 'open')",
    [`${app.doc.node.id}-${Date.now().toString(36)}`, new Date().toISOString(), app.who, JSON.stringify({ to: app.doc.node.group_id, action: "contract", node: app.doc.node.id, output: v[0] }), v[1]]));
}

async function save() {
  const s = app.cur;
  const r = await guard("Saving", () => s.save());
  if (!r) return;
  if (app.viaDownload) download(s.name, s.bytes());
  toast(`Saved (revision ${r.n}), read back and checked`);
  await refresh();
}

// ------------------------------------------------------------------ start
function start() {
  render(toolbar(...topButtons()),
    canFolders ? banner("info", "Start here.", "Open the design folder on your Drive (the one holding structure/ and nodes/): the nodes issued to you are listed first. Or open one node file on its own.")
      : banner("warn", "This browser cannot open folders.", "Open one node file, fill it, and save it by downloading. Chrome and Edge open the Drive folder directly."));
  setStatus("Ready");
}

window.addEventListener("beforeunload", (e) => { if (app.cur && app.cur.dirty) e.preventDefault(); });

async function profileId() {
  let p = await journal.pref("profile").catch(() => null);
  if (!p) { p = crypto.getRandomValues(new Uint32Array(4)).join("-"); await journal.setPref("profile", p).catch(() => {}); }
  return p;
}

const wasm = Uint8Array.from(atob(document.getElementById("tn-sqlite-wasm").textContent.trim()), (c) => c.charCodeAt(0));
app.ready = Promise.all([window.initSqlJs({ wasmBinary: wasm }), profileId()]).then(([SQL, profile]) => { app.SQL = SQL; app.profile = profile; start(); });

// for the browser tests (tests/browser/node.test.mjs) and for anyone automating the page
window.tnNode = { app, ready: app.ready, useFolder, openNode, closeNode, save, goTo, refresh, STEPS };
