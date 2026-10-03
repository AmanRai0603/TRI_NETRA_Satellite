// files_app.js -- TRI-NETRA Files (docs/RELEASE_PLAN.md P3): open a folder of design files (a
// Drive folder synced to this computer), see who has what open and which conflict copies Drive
// made, open a node file, change it, undo, save, see its history. The groundwork the node app
// and the group app (P4-P6) are built on; it uses only the file layer (tnfile.js) and the
// component set (tn_ui.js).
// Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
import { openFile, listFolder, Journal, MemoryFolder, FileRefused, MARKER_STALE_MS } from "./tnfile.js";
import { h, fill, button, fileButton, field, banner, badge, table, section, toolbar, spacer, kv, dialog, ask, toast, shell, download } from "./tn_ui.js";

const canFolders = typeof window.showDirectoryPicker === "function";
const { main, status } = shell("TRI-NETRA Files", "Design files on your Drive folder: open, change, save. Nothing leaves this computer.");
const journal = new Journal();
const session = crypto.getRandomValues(new Uint32Array(2)).join("-");
const app = { SQL: null, dir: null, viaDownload: false, cur: null, who: null, folderNote: null };

function remembered(k) { try { return localStorage.getItem(k); } catch (e) { return null; } }
function remember(k, v) { try { localStorage.setItem(k, v); } catch (e) { /* private window: asked again next time */ } }

async function who() {
  if (app.who) return app.who;
  app.who = remembered("trinetra.who");
  while (!app.who) {
    app.who = await ask("Your name", "Name", { help: "Written beside every change you save, and shown to anyone who opens a file you have open. There are no accounts: Drive's record of who saved each file backs it up." });
  }
  remember("trinetra.who", app.who);
  return app.who;
}

function fmtSize(n) { return n < 1024 ? `${n} B` : n < 1048576 ? `${(n / 1024).toFixed(1)} KB` : `${(n / 1048576).toFixed(2)} MB`; }
function fmtTime(t) { return new Date(t).toLocaleString(); }

async function guard(what, fn) {
  try { return await fn(); }
  catch (e) {
    const msg = e instanceof FileRefused ? e.message : `${what}: ${e.message || e}`;
    toast(msg, "error");
    main.prepend(banner("error", `${what} did not happen.`, msg));
    return undefined;
  }
}

// ------------------------------------------------------------------ the folder
async function pickFolder() {
  const dir = await guard("Opening the folder", () => window.showDirectoryPicker({ id: "trinetra", mode: "readwrite" }));
  if (dir) await useFolder(dir);
}

async function reopenFolder() {
  const dir = await guard("Reopening the folder", () => journal.pref("folder"));
  if (!dir) return;
  const ok = (await dir.queryPermission({ mode: "readwrite" })) === "granted" || (await dir.requestPermission({ mode: "readwrite" })) === "granted";
  if (ok) await useFolder(dir);
}

async function useFolder(dir, { viaDownload = false } = {}) {
  await who();
  if (app.cur) await closeFile();
  app.dir = dir;
  app.viaDownload = viaDownload;
  if (!viaDownload) await journal.setPref("folder", dir).catch(() => {});
  await showFolder();
}

async function openOneFile(files) {
  const f = files[0];
  const dir = new MemoryFolder("this computer", { [f.name]: new Uint8Array(await f.arrayBuffer()) });
  await useFolder(dir, { viaDownload: true });
  await openName(f.name);
}

async function showFolder() {
  const list = await guard("Reading the folder", () => listFolder(app.dir));
  if (!list) return;
  const rows = list.map((f) => ({ ...f, flags: flags(f) }));
  render(
    toolbar(...topButtons()),
    app.viaDownload ? banner("warn", "Saving by downloading.", "This browser cannot open folders, so the file opened here is saved by downloading it, and nobody else editing it can be seen from here. Chrome and Edge open Drive folders directly.") : null,
    section(`Folder: ${app.dir.name}`,
      table([
        { key: "name", label: "File", mono: true },
        { key: "kind", label: "Kind" },
        { key: "size", label: "Size", render: (r) => fmtSize(r.size) },
        { key: "lastModified", label: "Saved", render: (r) => fmtTime(r.lastModified) },
        { key: "flags", label: "", render: (r) => r.flags },
      ], rows, { empty: "No design files here (.node.tndb, .group.tndb, .tnrel, .tndb).", onRow: (r) => openName(r.name), testid: "folder" })));
  setStatus(`${list.length} design file(s) in ${app.dir.name}`);
}

function flags(f) {
  const out = [];
  if (f.editing && !f.editing.stale) out.push(badge(`open by ${f.editing.who}`, "warn"));
  if (f.editing && f.editing.stale) out.push(badge(`left open by ${f.editing.who}`, ""));
  if (f.conflicts.length) out.push(badge(`${f.conflicts.length} conflict cop${f.conflicts.length > 1 ? "ies" : "y"}`, "error"));
  if (f.orphanCopyOf) out.push(badge(`copy of missing ${f.orphanCopyOf}`, "error"));
  return h("span", { class: "tn-stack" }, out);
}

function topButtons() {
  const b = [];
  if (canFolders) b.push(button("Open folder…", pickFolder, { kind: app.dir ? "default" : "primary", testid: "open-folder" }));
  if (canFolders) b.push(button("Reopen last folder", reopenFolder, { kind: "quiet" }));
  b.push(fileButton(canFolders ? "Open one file…" : "Open a file…", openOneFile, { accept: ".tndb,.tnrel", testid: "open-file" }));
  b.push(spacer(), button(`You: ${app.who || "?"}`, async () => { const n = await ask("Your name", "Name", { value: app.who || "" }); if (n) { app.who = n; remember("trinetra.who", n); rerender(); } }, { kind: "quiet" }));
  return b;
}

// ------------------------------------------------------------------ a file
async function openName(name, { readOnly = false, takeOver = false } = {}) {
  await who();
  if (app.cur) await closeFile();
  const s = await guard(`Opening ${name}`, () => openFile({ SQL: app.SQL, dir: app.dir, name, who: app.who, session, profile: app.profile, journal, locks: navigator.locks || null, readOnly, takeOver }));
  if (!s) return;
  app.cur = s;
  showFile();
}

async function closeFile() {
  if (!app.cur) return;
  const s = app.cur;
  if (s.dirty && !s.readOnly) {
    const v = await dialog("Unsaved changes", h("p", {}, `${s.name} has changes that are not saved. They stay kept in this browser either way, and are offered again when you open it.`),
      [["Keep them for later", "keep"], ["Save now", "save", "primary"]]);
    if (v === "save") await guard("Saving", () => s.save());
  }
  app.cur = null;
  await s.close();
}

function showFile() {
  const s = app.cur;
  const parts = [];
  parts.push(toolbar(
    button("◂ Folder", async () => { await closeFile(); await showFolder(); }, { kind: "quiet", testid: "back" }),
    button("Undo", () => act(() => s.undo()), { disabled: s.readOnly || !s.undoStack.length, testid: "undo", title: s.undoStack.length ? `Undo: ${s.undoStack.at(-1).summary}` : "" }),
    button("Redo", () => act(() => s.redo()), { disabled: s.readOnly || !s.redoStack.length, testid: "redo" }),
    spacer(),
    button("Save a copy", () => act(async () => { const n = await s.saveCopy(); toast(`Saved as ${n}`); }), { disabled: s.readOnly || app.viaDownload }),
    button(app.viaDownload ? "Save (download)" : "Save", () => save(), { kind: "primary", disabled: s.readOnly || !s.dirty, testid: "save" })));
  if (s.readOnly) parts.push(banner("warn", "Read-only.", s.readOnlyWhy + ".",
    !s.wantReadOnly ? button("Open for editing anyway", () => confirmTakeOver(s.name)) : null));
  for (const n of s.notes) parts.push(banner("info", "", n));
  if (s.conflicts.length) parts.push(banner("error", "Conflict copies.", `Drive made ${s.conflicts.join(", ")} beside this file: two computers saved it at once. Open each, compare, carry over what is missing, then delete the copy in Drive. They are never ignored.`,
    ...s.conflicts.map((c) => button(`Look at ${c}`, () => openName(c, { readOnly: true })))));
  if (s.recovery) {
    parts.push(s.recovery.stale
      ? banner("warn", "Unsaved work from before.", `Changes made on ${fmtTime(s.recovery.at)} (${(s.recovery.actions || []).join("; ") || "—"}) were never saved, and the file has been saved since. Keep them as a copy to compare, or let them go.`,
        button("Keep as a copy", () => act(async () => { const n = await s.restoreAsCopy(); toast(`Kept as ${n}`); })), button("Let them go", () => act(() => s.discardRecovery()), { kind: "danger" }))
      : banner("warn", "Unsaved work from before.", `Changes made on ${fmtTime(s.recovery.at)} (${(s.recovery.actions || []).join("; ") || "—"}) were never saved (the browser closed or crashed).`,
        button("Restore them", () => act(() => s.restore()), { kind: "primary", testid: "restore" }), button("Let them go", () => act(() => s.discardRecovery()), { kind: "danger" })));
  }
  parts.push(section("File", kv([["Name", h("span", { class: "tn-mono" }, s.name)], ["Kind", `${s.kind} (format version ${s.meta.format_version})`],
    ["Written by", s.meta.written_by || "—"], ["Folder", app.dir.name]])));
  if (s.kind === "node") parts.push(...nodeView(s));
  else parts.push(section("Tables", table([{ key: "t", label: "Table", mono: true }, { key: "n", label: "Rows" }],
    s.format.tables.map((t) => ({ t, n: s.query(`SELECT count(*) FROM "${t}"`)[0][0] })))), banner("info", "", "This kind of file is edited in the group app (P4–P6); here it is shown only."));
  parts.push(section("History",
    table([{ key: "n", label: "#" }, { key: "at", label: "When", render: (r) => fmtTime(r.at) }, { key: "by", label: "Who" }, { key: "summary", label: "What" }],
      s.query("SELECT n, at, by, summary FROM revision ORDER BY n DESC").map(([n, at, by, summary]) => ({ n, at, by, summary })), { empty: "Never saved from the apps yet.", testid: "history" }),
    h("p", { class: "tn-dim" }, "Drive keeps every saved version of the file as well: right-click it in Drive, Manage versions.")));
  render(...parts);
  setStatus(stateLine(s));
}

function nodeView(s) {
  const node = s.query("SELECT id, label, kind, layer, stage, state, author FROM node")[0] || [];
  const content = s.query("SELECT rowid, section, field, value, origin FROM content ORDER BY section, field").map(([rowid, sec, f, value, origin]) => ({ rowid, sec, f, value, origin }));
  const pics = s.query("SELECT name, mime, length(bytes) FROM attachment ORDER BY name").map(([name, mime, size]) => ({ name, mime, size }));
  const cap = s.format.max_attachment_bytes;
  return [
    section("Node", kv([["Id", h("span", { class: "tn-mono" }, node[0] || "—")], ["Label", node[1] || "—"], ["Kind", node[2] || "—"],
      ["Layer · stage", `${node[3] || "—"} · ${node[4] || "—"}`], ["State", node[5] || "—"], ["Author", node[6] || "—"]])),
    section("Content",
      table([{ key: "sec", label: "Section" }, { key: "f", label: "Field", mono: true }, { key: "value", label: "Value" }, { key: "origin", label: "From" }],
        content, { empty: "No content yet.", onRow: s.readOnly ? null : (r) => editValue(r), testid: "content" }),
      s.readOnly ? null : toolbar(button("Add a field…", () => addField(), { testid: "add-field" }))),
    section("Pictures",
      table([{ key: "name", label: "Name", mono: true }, { key: "mime", label: "Type" }, { key: "size", label: "Size", render: (r) => fmtSize(r.size) }],
        pics, { empty: "No pictures." }),
      s.readOnly ? null : toolbar(fileButton("Add a picture…", (fl) => addPicture(fl[0]), { accept: "image/png,image/jpeg,image/svg+xml,image/webp", testid: "add-picture" }),
        h("span", { class: "tn-dim" }, `Up to ${fmtSize(cap)} each; a file holds up to ${fmtSize(s.format.max_bytes)}.`))),
  ];
}

async function editValue(r) {
  const f = field(`${r.sec} · ${r.f}`, { value: r.value ?? "", multiline: true, testid: "value" });
  const v = await dialog("Change a value", f.el, [["Cancel", null], ["Apply", () => f.value, "primary"]]);
  if (v === null || v === r.value) return;
  await act(() => app.cur.change(`${r.sec}.${r.f}`, (db) => db.run("UPDATE content SET value = ?, origin = ? WHERE rowid = ?", [v, `typed by ${app.who}`, r.rowid])));
}

async function addField() {
  const sec = field("Section", { testid: "new-section" }), fl = field("Field", { mono: true, testid: "new-field" }), val = field("Value", { multiline: true, testid: "new-value" });
  const v = await dialog("Add a field", h("div", { class: "tn-stack" }, sec.el, fl.el, val.el), [["Cancel", null], ["Add", () => [sec.value.trim(), fl.value.trim(), val.value], "primary"]]);
  if (!v || !v[0] || !v[1]) return;
  await act(() => app.cur.change(`add ${v[0]}.${v[1]}`, (db) => db.run("INSERT INTO content (section, field, value, origin) VALUES (?, ?, ?, ?)", [v[0], v[1], v[2], `typed by ${app.who}`])));
}

async function addPicture(file) {
  const cap = app.cur.format.max_attachment_bytes;
  if (file.size > cap) { toast(`${file.name} is ${fmtSize(file.size)}; a picture may be ${fmtSize(cap)} at most. Make it smaller (crop, or save as JPEG) and add it again.`, "error"); return; }
  const bytes = new Uint8Array(await file.arrayBuffer());
  await act(() => app.cur.change(`picture ${file.name}`, (db) => db.run("INSERT OR REPLACE INTO attachment (name, mime, size, bytes) VALUES (?, ?, ?, ?)", [file.name, file.type, bytes.length, bytes])));
}

async function save() {
  const s = app.cur;
  const r = await guard("Saving", () => s.save());
  if (!r) { showFileKeepBanner(); return; }
  if (app.viaDownload) download(s.name, s.bytes());
  toast(`Saved ${s.name} (revision ${r.n}), read back and checked`);
  showFile();
}

function showFileKeepBanner() {
  // a refused save: the work is kept in this browser; offer the copy
  const s = app.cur;
  if (!s) return;
  main.prepend(banner("error", "Not saved.", "Your changes are kept in this browser. Save them as a copy beside the file and compare.",
    button("Save a copy", () => act(async () => { const n = await s.saveCopy(); toast(`Saved as ${n}`); }), { kind: "primary" })));
}

async function confirmTakeOver(name) {
  const v = await dialog("Open for editing anyway?", h("p", {}, "Someone else has this file open. If you both save, one of you loses work, or Drive makes a conflict copy. Do this only when you know they have stopped (they closed the browser without closing the file, or their computer is off)."),
    [["Cancel", null], ["Open for editing", true, "danger"]]);
  if (v) await openName(name, { takeOver: true });
}

async function act(fn) { await guard("That change", fn); if (app.cur) showFile(); }

function stateLine(s) {
  const bits = [s.readOnly ? `read-only: ${s.readOnlyWhy}` : "editing"];
  if (s.dirty) bits.push(`${s.pending.length} unsaved change(s), kept in this browser until saved`);
  else if (s.lastSaved) bits.push(`saved ${fmtTime(s.lastSaved.at)}, read back and checked`);
  else bits.push("no changes");
  return bits.join(" · ");
}

function setStatus(t) { fill(status, h("span", { "data-testid": "status" }, t)); }
function render(...parts) { fill(main, ...parts); }
function rerender() { if (app.cur) showFile(); else if (app.dir) showFolder(); else start(); }

function start() {
  render(toolbar(...topButtons()),
    canFolders ? banner("info", "Start here.", "Open the Drive folder of your group (Drive for desktop shows it as a folder on this computer). Files open one at a time; anyone else who opens one you have open sees your name.")
      : banner("warn", "This browser cannot open folders.", "You can open a file, change it and save it by downloading. To open Drive folders and save in place, use Chrome or Edge."));
  setStatus(`Ready · stale markers are taken over after ${MARKER_STALE_MS / 60000} min`);
}

window.addEventListener("beforeunload", (e) => { if (app.cur && app.cur.dirty) e.preventDefault(); });

// SQLite in WebAssembly, from the bytes inlined in this page (tools/pages.py): no outside hosts
const wasm = Uint8Array.from(atob(document.getElementById("tn-sqlite-wasm").textContent.trim()), (c) => c.charCodeAt(0));
async function profileId() {
  let p = await journal.pref("profile").catch(() => null);
  if (!p) { p = crypto.getRandomValues(new Uint32Array(4)).join("-"); await journal.setPref("profile", p).catch(() => {}); }
  return p;
}
app.ready = Promise.all([window.initSqlJs({ wasmBinary: wasm }), profileId()]).then(([SQL, profile]) => { app.SQL = SQL; app.profile = profile; start(); });

// for the browser tests (tests/browser/files.test.mjs) and for anyone automating the page
window.tnFiles = { app, ready: app.ready, useFolder, openName, closeFile, save: () => save() };
