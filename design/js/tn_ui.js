// tn_ui.js -- the one component set of every TRI-NETRA page (docs/RELEASE_PLAN.md P3): buttons,
// fields, banners, badges, tables, dialogs, toasts, the page shell. A page builds its controls
// only from these (tools/pages.py refuses a page that makes a button, an input or a style of its
// own), so every page looks and behaves the same, in light and dark, at phone width.
// Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.

/** An element: h("div", { class: "x", onclick: f }, child, "text", [more]). Layout only: a page
 *  makes its controls with the functions below. */
export function h(tag, props = {}, ...children) {
  const el = document.createElement(tag);
  for (const [k, v] of Object.entries(props || {})) {
    if (v === undefined || v === null || v === false) continue;
    if (k.startsWith("on") && typeof v === "function") el.addEventListener(k.slice(2), v);
    else if (k === "class") el.className = v;
    else if (k === "text") el.textContent = v;
    else el.setAttribute(k, v === true ? "" : v);
  }
  add(el, children);
  return el;
}

function add(el, children) {
  for (const c of children.flat(Infinity)) {
    if (c === null || c === undefined || c === false) continue;
    el.append(c instanceof Node ? c : document.createTextNode(String(c)));
  }
}

/** Replace an element's children. */
export function fill(el, ...children) { el.replaceChildren(); add(el, children); return el; }

/** kind: "primary" | "default" | "danger" | "quiet". */
export function button(label, onClick, { kind = "default", disabled = false, title = "", testid = "" } = {}) {
  return h("button", { class: `tn-button ${kind}`, type: "button", onclick: onClick, disabled, title: title || null, "data-testid": testid || null }, label);
}

let fieldSeq = 0;
/** A labelled input (or textarea when multiline). Returns { el, input, value, setError }. */
export function field(label, { value = "", multiline = false, help = "", readOnly = false, mono = false, placeholder = "", testid = "", onInput = null } = {}) {
  const id = `tn-f${++fieldSeq}`;
  const input = h(multiline ? "textarea" : "input", { id, class: `tn-input${mono ? " tn-mono" : ""}`, readonly: readOnly, placeholder: placeholder || null,
    "data-testid": testid || null, type: multiline ? null : "text", oninput: onInput ? (e) => onInput(e.target.value) : null });
  input.value = value;
  const err = h("div", { class: "tn-error", role: "alert" });
  const el = h("div", { class: "tn-field" }, h("label", { for: id }, label), input, help ? h("div", { class: "tn-help" }, help) : null, err);
  return { el, input, get value() { return input.value; }, setError(t) { err.textContent = t || ""; } };
}

/** A button that picks files: onFiles(FileList). */
export function fileButton(label, onFiles, { accept = "", multiple = false, testid = "" } = {}) {
  const input = h("input", { type: "file", accept: accept || null, multiple, hidden: true, "data-testid": testid || null,
    onchange: (e) => { if (e.target.files.length) onFiles(e.target.files); e.target.value = ""; } });
  const b = button(label, () => input.click());
  return h("span", {}, b, input);
}

/** kind: "info" | "ok" | "warn" | "error". actions: buttons. */
export function banner(kind, title, text, ...actions) {
  return h("div", { class: `tn-banner ${kind}`, role: kind === "error" ? "alert" : "status" },
    h("div", { class: "tn-text" }, title ? h("b", {}, title + " ") : null, text || ""), actions);
}

export function badge(text, kind = "") { return h("span", { class: `tn-badge ${kind}` }, text); }

/** columns: [{ key, label, mono, render(row) }]; onRow(row) makes rows clickable. */
export function table(columns, rows, { empty = "Nothing here.", onRow = null, testid = "" } = {}) {
  if (!rows.length) return h("div", { class: "tn-empty", "data-testid": testid || null }, empty);
  const head = h("tr", {}, columns.map((c) => h("th", { scope: "col" }, c.label)));
  const body = rows.map((r) => h("tr", { class: onRow ? "tn-clickable" : null, onclick: onRow ? () => onRow(r) : null, tabindex: onRow ? "0" : null,
    onkeydown: onRow ? (e) => { if (e.key === "Enter") onRow(r); } : null },
  columns.map((c) => h("td", { class: c.mono ? "tn-mono" : null }, c.render ? c.render(r) : r[c.key] ?? ""))));
  return h("div", { class: "tn-table-wrap" }, h("table", { class: "tn-table", "data-testid": testid || null }, h("thead", {}, head), h("tbody", {}, body)));
}

export function section(title, ...children) { return h("section", { class: "tn-section" }, title ? h("h2", {}, title) : null, children); }
export function toolbar(...items) { return h("div", { class: "tn-toolbar", role: "toolbar" }, items); }
export function spacer() { return h("span", { class: "tn-spacer" }); }
export function kv(pairs) { return h("dl", { class: "tn-kv" }, pairs.map(([k, v]) => [h("dt", {}, k), h("dd", {}, v)])); }

/** A modal dialog; resolves with the value of the action chosen (or null on Escape). */
export function dialog(title, body, actions) {
  return new Promise((resolve) => {
    const d = h("dialog", { class: "tn-dialog", "aria-label": title });
    const done = (v) => { d.close(); d.remove(); resolve(v); };
    d.addEventListener("cancel", (e) => { e.preventDefault(); done(null); });
    fill(d, h("h2", {}, title), body, h("div", { class: "tn-actions" },
      actions.map(([label, value, kind]) => button(label, () => done(typeof value === "function" ? value() : value), { kind: kind || "default" }))));
    document.body.append(d);
    d.showModal();
  });
}

/** Ask for one line of text; resolves with it, or null. */
export async function ask(title, label, { value = "", help = "" } = {}) {
  const f = field(label, { value, help });
  setTimeout(() => f.input.focus(), 0);
  return dialog(title, f.el, [["Cancel", null], ["OK", () => f.value.trim() || null, "primary"]]);
}

let toasts = null;
export function toast(text, kind = "") {
  if (!toasts) { toasts = h("div", { class: "tn-toasts", "aria-live": "polite" }); document.body.append(toasts); }
  const t = h("div", { class: `tn-toast ${kind}` }, text);
  toasts.append(t);
  setTimeout(() => t.remove(), kind === "error" ? 8000 : 3500);
}

/** Hand the person a file to keep (the browser's download). */
export function download(name, bytes, type = "application/octet-stream") {
  const url = URL.createObjectURL(new Blob([bytes], { type }));
  const a = h("a", { href: url, download: name });
  document.body.append(a);
  a.click();
  a.remove();
  setTimeout(() => URL.revokeObjectURL(url), 10000);
}

/** The page: a header (title, line under it) and the main area; a status line fixed at the foot. */
export function shell(title, subtitle) {
  const main = h("main", {});
  const status = h("footer", { class: "tn-status", "aria-live": "polite" });
  document.body.append(h("div", { class: "tn-shell" }, h("header", { class: "tn-head" }, h("h1", {}, title), h("p", {}, subtitle)), main), status);
  return { main, status };
}

/** Tabs: [[label, render()]]; render runs when its tab is chosen. Returns { el, show(i) }. */
export function tabs(items, { testid = "" } = {}) {
  const body = h("div", { class: "tn-tabbody" });
  const bar = h("div", { class: "tn-tabs", role: "tablist", "data-testid": testid || null });
  const btns = items.map(([label], i) => h("button", { class: "tn-tab", type: "button", role: "tab", onclick: () => show(i), "data-tab": label }, label));
  bar.append(...btns);
  let cur = 0;
  function show(i) {
    cur = i;
    btns.forEach((b, j) => b.setAttribute("aria-selected", j === i ? "true" : "false"));
    fill(body, items[i][1]());
  }
  show(0);
  return { el: h("div", {}, bar, body), show, get current() { return cur; } };
}

const SVG = "http://www.w3.org/2000/svg";
function s(tag, attrs = {}, ...kids) {
  const el = document.createElementNS(SVG, tag);
  for (const [k, v] of Object.entries(attrs)) if (v !== null && v !== undefined) el.setAttribute(k, v);
  for (const k of kids.flat()) if (k !== null && k !== undefined) el.append(k instanceof Node ? k : document.createTextNode(String(k)));
  return el;
}

/** A map: columns of boxes (a group's stages, with the inputs from other groups first) and the
 *  arrows between them. columns: [{ label, nodes: [{ id, label, kind: "" | "outside" | "archived" | "selected" }] }];
 *  edges: [{ from, to }]; onNode(id). Scrolls sideways inside itself at phone width. */
export function graph(columns, edges, { onNode = null, testid = "" } = {}) {
  const W = 210, H = 30, GX = 70, GY = 8, TOP = 34, PAD = 10;
  const pos = new Map();
  columns.forEach((c, i) => c.nodes.forEach((n, j) => pos.set(n.id, { x: PAD + i * (W + GX), y: TOP + j * (H + GY) })));
  const width = PAD * 2 + columns.length * W + (columns.length - 1) * GX;
  const height = TOP + Math.max(1, ...columns.map((c) => c.nodes.length)) * (H + GY) + PAD;
  const svg = s("svg", { class: "tn-graph", viewBox: `0 0 ${width} ${height}`, width, height, role: "img", "aria-label": "map of the group's nodes and what each reads" });
  svg.append(s("defs", {}, s("marker", { id: "tn-arrow", viewBox: "0 0 10 10", refX: "9", refY: "5", markerWidth: "7", markerHeight: "7", orient: "auto-start-reverse" },
    s("path", { d: "M0,0 L10,5 L0,10 z", class: "tn-graph-head" }))));
  columns.forEach((c, i) => svg.append(s("text", { x: PAD + i * (W + GX), y: 18, class: "tn-graph-col" }, c.label)));
  for (const e of edges) {
    const a = pos.get(e.from), b = pos.get(e.to);
    if (!a || !b) continue;
    const back = b.x <= a.x;
    const x1 = back ? a.x : a.x + W, y1 = a.y + H / 2, x2 = back ? b.x + W : b.x, y2 = b.y + H / 2;
    const dx = back ? -Math.max(40, Math.abs(x2 - x1) / 2) - 40 : Math.max(30, (x2 - x1) / 2);
    svg.append(s("path", { d: `M${x1},${y1} C${x1 + dx},${y1} ${x2 - dx},${y2} ${x2},${y2}`, class: "tn-graph-edge", "marker-end": "url(#tn-arrow)" }));
  }
  for (const c of columns) for (const n of c.nodes) {
    const p = pos.get(n.id);
    const g = s("g", { class: `tn-graph-node ${n.kind || ""}`, tabindex: onNode ? "0" : null, role: onNode ? "button" : null, "data-node": n.id },
      s("title", {}, `${n.id}: ${n.label}`),
      s("rect", { x: p.x, y: p.y, width: W, height: H, rx: 5 }),
      s("text", { x: p.x + 8, y: p.y + 19 }, n.label.length > 30 ? n.label.slice(0, 29) + "…" : n.label));
    if (onNode) { g.addEventListener("click", () => onNode(n.id)); g.addEventListener("keydown", (ev) => { if (ev.key === "Enter") onNode(n.id); }); }
    svg.append(g);
  }
  return h("div", { class: "tn-graph-wrap", "data-testid": testid || null }, svg);
}

/** An impact check: its lines, worst first. */
export function impactList(items) {
  const order = { block: 0, warn: 1, info: 2 }, label = { block: "Stops it", warn: "Changes", info: "Note" }, cls = { block: "error", warn: "warn", info: "" };
  return h("ul", { class: "tn-impact", "data-testid": "impact" }, [...items].sort((a, b) => order[a.level] - order[b.level])
    .map((i) => h("li", { class: `tn-impact-${i.level}` }, badge(label[i.level], cls[i.level]), " ", i.text)));
}

/** A choice from a list: { el, value }. options: [[value, label]]. */
export function choice(label, options, { value = "", testid = "", help = "" } = {}) {
  const id = `tn-f${++fieldSeq}`;
  const sel = h("select", { id, class: "tn-input", "data-testid": testid || null }, options.map(([v, l]) => h("option", { value: v }, l)));
  sel.value = value;
  const el = h("div", { class: "tn-field" }, h("label", { for: id }, label), sel, help ? h("div", { class: "tn-help" }, help) : null);
  return { el, input: sel, get value() { return sel.value; } };
}

/** Several choices: { el, values }. */
export function checks(label, options, { testid = "" } = {}) {
  const boxes = options.map(([v, l]) => { const b = h("input", { type: "checkbox", value: v }); return [b, h("label", { class: "tn-check" }, b, " ", l)]; });
  const el = h("fieldset", { class: "tn-field tn-checks", "data-testid": testid || null }, h("legend", {}, label), boxes.map(([, l]) => l));
  return { el, get values() { return boxes.filter(([b]) => b.checked).map(([b]) => b.value); } };
}
