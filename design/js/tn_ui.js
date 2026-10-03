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
