// tnfile.js -- the design files (design/schema.toml) in the browser (docs/RELEASE_PLAN.md P3):
// a file opened from a folder (a Drive folder synced to the computer, opened with the browser's
// folder access), checked as tools/tndb.py checks it, edited in memory with SQLite compiled to
// WebAssembly, and written back whole.
//
//   - Open elsewhere: beside the file, "<file>.editing" says who has it open (Drive carries it to
//     the other computers); within one browser a lock does the same at once. A second editor gets
//     the file read-only, with the name of the first. A marker not refreshed for MARKER_STALE_MS
//     is someone who closed the browser without closing the file: it is taken over, and said so.
//   - Conflict copies: Drive's "name (1).ext" and "conflict" copies beside a file are listed, never
//     ignored.
//   - Crash-safe: every change is kept at once in this browser (IndexedDB) until it is saved; a
//     crash, a closed tab, a power cut lose nothing that was shown. A save writes the whole file
//     through the browser's swap file (the old file stays whole until the new one is complete),
//     reads it back and compares, and refuses to overwrite a file someone else changed meanwhile.
//   - History and undo: every change is a step that can be undone and redone; every save adds a
//     revision (who, when, what), and Drive keeps every saved version.
//   - Size caps: a file over its format's max_bytes, a picture over max_attachment_bytes, refused.
//
// Nothing here touches the page: the app (design/js/files_app.js) shows what it reports.
// Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
import { TNDB_SCHEMA } from "./tndb_schema.js";

export const MARKER_SUFFIX = ".editing";
export const MARKER_STALE_MS = 15 * 60 * 1000;
export const BEAT_MS = 60 * 1000;
const SQLITE_HEADER = "SQLite format 3\u0000";

/** A file this program will not open or write, and why (code: a short word the app can test). */
export class FileRefused extends Error {
  constructor(message, code) { super(message); this.code = code; }
}

// ------------------------------------------------------------------ names

/** The format a file name belongs to (the longest extension that ends it), or null. */
export function kindOf(name, schema = TNDB_SCHEMA) {
  let best = null;
  for (const [k, f] of Object.entries(schema.formats)) {
    if (name.endsWith(f.extension) && (!best || f.extension.length > schema.formats[best].extension.length)) best = k;
  }
  return best;
}

/** The original's name when `name` is a conflict copy Drive (or a person) made of it, else null:
 *  "a (1).node.tndb", "a.node (2).tndb", "a.node.tndb (1)", "a_conflict-2026….node.tndb",
 *  "a (conflicted copy 2026-10-03).node.tndb". */
export function conflictBase(name) {
  const pats = [/ \(\d+\)(?=\.|$)/, /[ _-]?\(?conflict(?:ed)?(?: copy)?[^.()]*\)?(?=\.|$)/i];
  for (const p of pats) {
    const s = name.replace(p, "");
    if (s !== name && s.length) return s;
  }
  return null;
}

/** The conflict copies of `name` among `names`. */
export function conflictsOf(name, names) {
  return names.filter((n) => n !== name && conflictBase(n) === name).sort();
}

/** A copy's name that no one will take for a conflict copy: "<stem>-copy-<who>-<yyyymmdd-hhmm><ext>". */
export function copyName(name, who, at, schema = TNDB_SCHEMA) {
  const k = kindOf(name, schema);
  const ext = k ? schema.formats[k].extension : "";
  const stem = name.slice(0, name.length - ext.length);
  const p = (n) => String(n).padStart(2, "0");
  const t = `${at.getFullYear()}${p(at.getMonth() + 1)}${p(at.getDate())}-${p(at.getHours())}${p(at.getMinutes())}`;
  const w = String(who || "someone").toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "") || "someone";
  return `${stem}-copy-${w}-${t}${ext}`;
}

export async function sha256(bytes) {
  const h = new Uint8Array(await crypto.subtle.digest("SHA-256", bytes));
  return Array.from(h, (b) => b.toString(16).padStart(2, "0")).join("");
}

// ------------------------------------------------------------------ checking a file

/** The problems of a design file's bytes against the schema, as tools/tndb.py check finds them.
 *  Returns { db, kind, meta, problems }; db is open (the caller closes it) unless the bytes are
 *  not SQLite at all. */
export function checkBytes(SQL, bytes, { name = "the file", schema = TNDB_SCHEMA, expectKind = null } = {}) {
  const problems = [];
  const head = new TextDecoder().decode(bytes.subarray(0, 16));
  if (bytes.length < 100 || head !== SQLITE_HEADER) return { db: null, kind: null, meta: {}, problems: [`${name}: not a design file (not SQLite)`] };
  let db;
  try { db = new SQL.Database(bytes); } catch (e) { return { db: null, kind: null, meta: {}, problems: [`${name}: SQLite cannot open it (${e.message})`] }; }
  let meta = {};
  try { for (const [k, v] of rows(db, 'SELECT "key", "value" FROM meta')) meta[k] = v; }
  catch (e) { problems.push(`${name}: no meta table: not a design file`); return { db, kind: null, meta, problems }; }
  const kind = meta.format;
  const f = schema.formats[kind];
  if (!f) { problems.push(`${name}: format ${JSON.stringify(kind)} is not a design-file format`); return { db, kind: null, meta, problems }; }
  if (expectKind && kind !== expectKind) problems.push(`${name}: a ${kind} file, not a ${expectKind} file`);
  const have = Number(meta.format_version || 0);
  if (have > f.version) problems.push(`${name}: ${kind} format version ${have} is newer than this page's ${f.version}; open it with a newer TRI-NETRA`);
  if (have < f.version) problems.push(`${name}: ${kind} format version ${have} is older than this page's ${f.version}; upgrade it first (python3 tools/tndb.py check ${name} upgrades it and keeps the original beside it)`);
  if (problems.length) return { db, kind, meta, problems };
  const tables = new Set(rows(db, "SELECT name FROM sqlite_master WHERE type = 'table'").map((r) => r[0]));
  const want = new Set(f.tables);
  for (const t of [...want].sort()) if (!tables.has(t)) problems.push(`${name}: table ${t} missing`);
  for (const t of [...tables].sort()) if (!want.has(t)) problems.push(`${name}: table ${t} is not in the ${kind} format`);
  for (const t of f.tables) {
    if (!tables.has(t)) continue;
    const got = rows(db, `PRAGMA table_info("${t}")`).map((r) => `${r[1]} ${String(r[2]).toUpperCase()}`);
    const exp = schema.tables[t].map(([c, ty]) => `${c} ${ty.toUpperCase()}`);
    if (got.join(", ") !== exp.join(", ")) problems.push(`${name}: table ${t} has columns (${got.join(", ")}), the schema says (${exp.join(", ")})`);
  }
  if (f.max_bytes && bytes.length > f.max_bytes) problems.push(`${name}: ${bytes.length} bytes, over the ${f.max_bytes} a ${kind} file may hold`);
  problems.push(...attachmentProblems(db, kind, schema, name));
  return { db, kind, meta, problems };
}

function attachmentProblems(db, kind, schema, name) {
  const cap = schema.formats[kind].max_attachment_bytes;
  if (!cap || !schema.formats[kind].tables.includes("attachment")) return [];
  return rows(db, "SELECT name, length(bytes), size FROM attachment WHERE length(bytes) > ? OR size > ?", [cap, cap])
    .map(([n, len, size]) => `${name}: attachment ${n} is ${Math.max(len || 0, size || 0)} bytes, over the ${cap} a picture may be`);
}

function rows(db, sql, params = []) {
  const st = db.prepare(sql);
  try {
    st.bind(params);
    const out = [];
    while (st.step()) out.push(st.get());
    return out;
  } finally { st.free(); }
}

/** The bytes of a new design file of `kind`, as tools/tndb.py create makes it: its tables, its meta,
 *  and what fill(db) puts in. */
export function newFileBytes(SQL, kind, id, fill, { schema = TNDB_SCHEMA, writtenBy = "TRI-NETRA apps" } = {}) {
  const db = new SQL.Database();
  try {
    db.run("BEGIN");
    for (const st of schema.ddl[kind]) db.run(st);
    const meta = { format: kind, format_version: String(schema.formats[kind].version), id, written_by: writtenBy };
    for (const k of Object.keys(meta).sort()) db.run('INSERT INTO meta VALUES (?, ?)', [k, meta[k]]);
    if (fill) fill(db);
    db.run("COMMIT");
    return db.export();
  } finally { db.close(); }
}

// ------------------------------------------------------------------ the journal (unsaved work)

/** Unsaved work, kept in this browser's IndexedDB until it is saved or let go. */
export class Journal {
  constructor(idb = globalThis.indexedDB, dbName = "trinetra-files") { this.idb = idb; this.dbName = dbName; this._db = null; }
  async _open() {
    if (this._db) return this._db;
    this._db = await new Promise((res, rej) => {
      const q = this.idb.open(this.dbName, 1);
      q.onupgradeneeded = () => { q.result.createObjectStore("unsaved"); q.result.createObjectStore("prefs"); };
      q.onsuccess = () => res(q.result);
      q.onerror = () => rej(q.error);
    });
    return this._db;
  }
  async _tx(store, mode, fn) {
    const db = await this._open();
    return new Promise((res, rej) => {
      const t = db.transaction(store, mode);
      const r = fn(t.objectStore(store));
      t.oncomplete = () => res(r && "result" in r ? r.result : undefined);
      t.onerror = () => rej(t.error);
      t.onabort = () => rej(t.error || new Error("journal write aborted"));
    });
  }
  get(key) { return this._tx("unsaved", "readonly", (s) => s.get(key)); }
  put(key, entry) { return this._tx("unsaved", "readwrite", (s) => s.put(entry, key)); }
  delete(key) { return this._tx("unsaved", "readwrite", (s) => s.delete(key)); }
  keys() { return this._tx("unsaved", "readonly", (s) => s.getAllKeys()); }
  pref(key) { return this._tx("prefs", "readonly", (s) => s.get(key)); }
  setPref(key, v) { return this._tx("prefs", "readwrite", (s) => s.put(v, key)); }
}

// ------------------------------------------------------------------ undo and redo

// Temporary triggers write, for every change to a table, the SQL that undoes it (the pattern of
// sqlite.org/undoredo.html). They live in the connection, so they are put back after every
// export (sql.js reopens the database to export it).
function installUndo(db, kind, schema) {
  db.run("CREATE TEMP TABLE IF NOT EXISTS _undo (seq INTEGER PRIMARY KEY, sql TEXT)");
  for (const t of schema.formats[kind].tables) {
    const cols = schema.tables[t].map(([c]) => c);
    const q = (c) => `"${c}"`;
    const setOld = cols.map((c) => `'${q(c)}='||quote(old.${q(c)})`).join("||','||");
    const valsOld = cols.map((c) => `quote(old.${q(c)})`).join("||','||");
    db.run(`CREATE TEMP TRIGGER IF NOT EXISTS "_u_${t}_i" AFTER INSERT ON main."${t}" BEGIN
              INSERT INTO _undo(sql) VALUES('DELETE FROM "${t}" WHERE rowid='||new.rowid); END`);
    db.run(`CREATE TEMP TRIGGER IF NOT EXISTS "_u_${t}_u" AFTER UPDATE ON main."${t}" BEGIN
              INSERT INTO _undo(sql) VALUES('UPDATE "${t}" SET '||${setOld}||' WHERE rowid='||old.rowid); END`);
    db.run(`CREATE TEMP TRIGGER IF NOT EXISTS "_u_${t}_d" AFTER DELETE ON main."${t}" BEGIN
              INSERT INTO _undo(sql) VALUES('INSERT INTO "${t}"(rowid,${cols.map(q).join(",")}) VALUES('||old.rowid||','||${valsOld}||')'); END`);
  }
}

function takeUndo(db) {
  const r = rows(db, "SELECT sql FROM _undo ORDER BY seq").map((x) => x[0]);
  db.run("DELETE FROM _undo");
  return r;
}

// ------------------------------------------------------------------ a file open for editing

/**
 * Open `name` in the folder `dir` (a FileSystemDirectoryHandle: the folder the person picked, or,
 * in the tests, the browser's private folder; the same interface).
 *   SQL       sql.js (initSqlJs's result)
 *   who       the person's name, as they typed it (no accounts; Drive's record backs it)
 *   session   an id for this tab
 *   profile   an id of this browser profile (kept in its IndexedDB): its own crashed tab's marker is taken back
 *   journal   a Journal
 *   locks     navigator.locks (or null)
 *   now       the clock (tests move it)
 *   readOnly  open to look only
 *   takeOver  open for editing even though someone else's marker is fresh (they are told on save)
 */
export async function openFile(o) {
  const s = new FileSession(o);
  await s._open();
  return s;
}

export class FileSession {
  constructor({ SQL, dir, name, who, session, profile = null, journal, locks = null, now = () => new Date(), schema = TNDB_SCHEMA,
                readOnly = false, takeOver = false, timers = globalThis }) {
    Object.assign(this, { SQL, dir, name, who, session, profile, journal, locks, now, schema, timers });
    this.wantReadOnly = readOnly;
    this.takeOver = takeOver;
    this.readOnly = readOnly;
    this.readOnlyWhy = readOnly ? "opened to look only" : "";
    this.notes = [];             // things the person should know about this file (taken-over marker, …)
    this.conflicts = [];         // Drive conflict copies beside it
    this.recovery = null;        // unsaved work from before (a crash): { at, actions, bytes, stale }
    this.undoStack = [];         // [{ summary, sql: [...] }]
    this.redoStack = [];
    this.pending = [];           // summaries of the changes since the last save
    this.dirty = false;
    this.lastSaved = null;
    this._release = null;
    this._beat = null;
    this._journalBusy = null;
    this._journalAgain = false;
  }

  get key() { return `${this.dir.name || "folder"}/${this.name}`; }
  get markerName() { return this.name + MARKER_SUFFIX; }
  get format() { return this.schema.formats[this.kind]; }

  async _open() {
    let fh;
    try { fh = await this.dir.getFileHandle(this.name); }
    catch (e) { throw new FileRefused(`${this.name}: no such file in this folder`, "missing"); }
    this.fh = fh;
    const kind = kindOf(this.name, this.schema);
    if (!kind) throw new FileRefused(`${this.name}: not a design file name (${Object.values(this.schema.formats).map((f) => f.extension).join(", ")})`, "name");
    const file = await fh.getFile();
    const cap = this.schema.formats[kind].max_bytes;
    if (cap && file.size > cap) throw new FileRefused(`${this.name}: ${file.size} bytes, over the ${cap} a ${kind} file may hold`, "size");
    const bytes = new Uint8Array(await file.arrayBuffer());
    const c = checkBytes(this.SQL, bytes, { name: this.name, schema: this.schema, expectKind: kind });
    if (c.problems.length) { if (c.db) c.db.close(); throw new FileRefused(c.problems.join("\n"), "format"); }
    this.kind = kind;
    this.meta = c.meta;
    this.db = c.db;
    this.base = { hash: await sha256(bytes), size: file.size, lastModified: file.lastModified };
    installUndo(this.db, kind, this.schema);
    this.conflicts = conflictsOf(this.name, await this._names());
    if (!this.wantReadOnly) await this._claim();
    const j = await this.journal.get(this.key);
    if (j && j.who === this.who) this.recovery = { ...j, stale: j.base_hash !== this.base.hash };
    else if (j) this.notes.push(`unsaved work by ${j.who} from ${j.at} is kept in this browser`);
  }

  async _names() {
    const out = [];
    for await (const [n] of this.dir.entries()) out.push(n);
    return out;
  }

  // ---- open elsewhere
  async _readMarker() {
    try {
      const f = await (await this.dir.getFileHandle(this.markerName)).getFile();
      return JSON.parse(await f.text());
    } catch (e) { return null; }
  }

  async _writeMarker(since) {
    const m = { who: this.who, session: this.session, profile: this.profile, since, beat: this.now().toISOString(), app: "TRI-NETRA files" };
    const w = await (await this.dir.getFileHandle(this.markerName, { create: true })).createWritable();
    await w.write(JSON.stringify(m));
    await w.close();
    this.marker = m;
  }

  async _claim() {
    let locked = false;
    if (this.locks) {
      const got = await new Promise((res) => {
        this.locks.request(`trinetra:${this.key}`, { ifAvailable: true }, (lock) => {
          if (!lock) { res(false); return undefined; }
          res(true);
          return new Promise((done) => { this._release = done; });
        });
      });
      if (!got) { this._readOnly(`open for editing in another tab of this browser`); return; }
      locked = true;
    }
    const m = await this._readMarker();
    // this browser's own marker with no tab of it holding the lock: a tab that crashed or was
    // closed without closing the file; the same person, on the same computer, takes it straight back
    const mine = m && locked && this.profile && m.profile === this.profile;
    if (m && m.session !== this.session && mine) this.notes.push(`it was left open by a tab of this browser that closed without closing it (last seen ${m.beat}); taken back`);
    if (m && m.session !== this.session && !mine) {
      const age = this.now() - new Date(m.beat);
      if (age < MARKER_STALE_MS && !this.takeOver) {
        this._readOnly(`open for editing by ${m.who} since ${m.since} (last seen ${m.beat})`);
        if (this._release) { this._release(); this._release = null; }
        return;
      }
      this.notes.push(age < MARKER_STALE_MS
        ? `taken over from ${m.who}, who still had it open (last seen ${m.beat}); their unsaved changes will not be in it`
        : `${m.who} had it open until ${m.beat} and did not close it; taken over`);
    }
    await this._writeMarker(this.now().toISOString());
    this._beat = this.timers.setInterval(() => { this._heartbeat().catch(() => {}); }, BEAT_MS);
  }

  _readOnly(why) { this.readOnly = true; this.readOnlyWhy = why; }

  async _heartbeat() {
    const m = await this._readMarker();
    if (m && m.session !== this.session) { this.notes.push(`${m.who} took this file over at ${m.since}; saving will make a copy`); return; }
    await this._writeMarker(this.marker ? this.marker.since : this.now().toISOString());
  }

  // ---- reading
  query(sql, params = []) { return rows(this.db, sql, params); }

  // ---- changing
  /** One step: fn(db) runs its statements; all or nothing; undoable. summary says what it was. */
  async change(summary, fn) {
    if (this.readOnly) throw new FileRefused(`${this.name} is read-only here: ${this.readOnlyWhy}`, "readonly");
    this.db.run("SAVEPOINT step");
    try {
      takeUndo(this.db);
      fn(this.db);
      const bad = attachmentProblems(this.db, this.kind, this.schema, this.name);
      if (bad.length) throw new FileRefused(bad.join("\n"), "size");
      const sql = takeUndo(this.db);
      this.db.run("RELEASE step");
      if (sql.length) {
        this.undoStack.push({ summary, sql });
        this.redoStack = [];
        this.pending.push(summary);
        await this._changed();
      }
    } catch (e) {
      this.db.run("ROLLBACK TO step");
      this.db.run("RELEASE step");
      takeUndo(this.db);
      throw e;
    }
  }

  async undo() { return this._replay(this.undoStack, this.redoStack, "undo"); }
  async redo() { return this._replay(this.redoStack, this.undoStack, "redo"); }

  async _replay(from, to, word) {
    if (this.readOnly || !from.length) return null;
    const step = from.pop();
    this.db.run("SAVEPOINT step");
    takeUndo(this.db);
    for (let i = step.sql.length - 1; i >= 0; i--) this.db.run(step.sql[i]);
    const back = takeUndo(this.db);
    this.db.run("RELEASE step");
    to.push({ summary: step.summary, sql: back });
    this.pending.push(`${word}: ${step.summary}`);
    await this._changed();
    return step.summary;
  }

  /** The whole database as bytes (sql.js reopens it to export: the undo triggers go back in). */
  bytes() {
    const b = this.db.export();
    installUndo(this.db, this.kind, this.schema);
    return b;
  }

  async _changed() {
    this.dirty = true;
    await this._journalWrite();
  }

  // every change is kept in IndexedDB before the step returns; writes that overlap are coalesced
  async _journalWrite() {
    if (this._journalBusy) { this._journalAgain = true; return this._journalBusy; }
    this._journalBusy = (async () => {
      do {
        this._journalAgain = false;
        await this.journal.put(this.key, { who: this.who, at: this.now().toISOString(), base_hash: this.base.hash, actions: [...this.pending], bytes: this.bytes() });
      } while (this._journalAgain);
    })();
    try { await this._journalBusy; } finally { this._journalBusy = null; }
  }

  /** Take the unsaved work found on opening. A stale one (the file was saved since) is refused:
   *  save it as a copy instead (restoreAsCopy). */
  async restore() {
    const r = this.recovery;
    if (!r) return;
    if (r.stale) throw new FileRefused(`the unsaved work from ${r.at} was made on an older version of ${this.name}; keep it as a copy and compare`, "stale");
    if (this.readOnly) throw new FileRefused(`${this.name} is read-only here: ${this.readOnlyWhy}`, "readonly");
    const c = checkBytes(this.SQL, r.bytes, { name: this.name, schema: this.schema, expectKind: this.kind });
    if (c.problems.length) { if (c.db) c.db.close(); throw new FileRefused(c.problems.join("\n"), "format"); }
    this.db.close();
    this.db = c.db;
    installUndo(this.db, this.kind, this.schema);
    this.undoStack = []; this.redoStack = [];
    this.pending = [...(r.actions || []), `restored unsaved work from ${r.at}`];
    this.recovery = null;
    await this._changed();
  }

  async restoreAsCopy() {
    const r = this.recovery;
    if (!r) return null;
    const name = await this._writeNew(copyName(this.name, this.who, this.now(), this.schema), r.bytes);
    await this.journal.delete(this.key);
    this.recovery = null;
    return name;
  }

  async discardRecovery() { if (this.recovery) { await this.journal.delete(this.key); this.recovery = null; } }

  // ---- saving
  /** Write the file whole, then read it back and compare. Refused when read-only, when someone
   *  else has taken it over, when it changed on disk since it was opened or saved, or when it is
   *  over its cap; saveCopy() then keeps the work. */
  async save() {
    const p = await this.prepare();
    try { await this.commit(p); } catch (e) { this.unprepare(p); throw e; }
    return this.lastSaved;
  }

  /** The first half of a save, for a change that spans files (structure.js): every check, the
   *  revision row, the bytes to write, checked. Nothing is written. undo it with unprepare(). */
  async prepare(summary = null) {
    if (this.readOnly) throw new FileRefused(`${this.name} is read-only here: ${this.readOnlyWhy}`, "readonly");
    const m = await this._readMarker();
    if (m && m.session !== this.session) throw new FileRefused(`${m.who} took ${this.name} over at ${m.since}; save a copy and compare`, "taken");
    const disk = await this.fh.getFile();
    if (disk.size !== this.base.size || disk.lastModified !== this.base.lastModified) {
      const h = await sha256(new Uint8Array(await disk.arrayBuffer()));
      if (h !== this.base.hash) throw new FileRefused(`${this.name} changed on disk since it was opened (another computer, or Drive); save a copy and compare`, "changed");
    }
    const at = this.now().toISOString();
    const n = (this.query("SELECT coalesce(max(n), 0) FROM revision")[0][0] || 0) + 1;
    this.db.run("INSERT INTO revision (n, at, by, summary) VALUES (?, ?, ?, ?)", [n, at, this.who, summary || this.pending.join("; ") || "saved"]);
    takeUndo(this.db);
    const p = { at, n, bytes: null, hash: null, before: this.base.hash };
    try {
      p.bytes = this.bytes();
      const problems = this._checkOut(p.bytes);
      if (problems.length) throw new FileRefused(problems.join("\n"), problems.some((x) => /over the/.test(x)) ? "size" : "format");
      p.hash = await sha256(p.bytes);
    } catch (e) { this.unprepare(p); throw e; }
    return p;
  }

  /** A prepared save not written after all: its revision row goes again. */
  unprepare(p) {
    if (!this.db) return;
    this.db.run("DELETE FROM revision WHERE n = ?", [p.n]);
    takeUndo(this.db);
  }

  /** The second half: write the prepared bytes, read them back and compare. */
  async commit(p) {
    await this._write(this.fh, p.bytes);
    const back = new Uint8Array(await (await this.fh.getFile()).arrayBuffer());
    if ((await sha256(back)) !== p.hash) throw new FileRefused(`${this.name}: the file read back is not what was written; the work is kept in this browser`, "verify");
    const f = await this.fh.getFile();
    this.base = { hash: p.hash, size: f.size, lastModified: f.lastModified };
    await this.journal.delete(this.key);
    this.pending = [];
    this.dirty = false;
    this.lastSaved = { at: p.at, n: p.n };
  }

  /** The work as a new file beside this one (the original untouched); returns its name. */
  async saveCopy() {
    const bytes = this.bytes();
    const problems = this._checkOut(bytes);
    if (problems.length) throw new FileRefused(problems.join("\n"), "size");
    const name = await this._writeNew(copyName(this.name, this.who, this.now(), this.schema), bytes);
    await this.journal.delete(this.key);
    this.dirty = false;
    return name;
  }

  _checkOut(bytes) {
    const c = checkBytes(this.SQL, bytes, { name: this.name, schema: this.schema, expectKind: this.kind });
    let problems = c.problems;
    if (c.db) {
      if (!problems.length) {
        const ic = rows(c.db, "PRAGMA integrity_check").map((r) => r[0]);
        if (ic.join() !== "ok") problems = [`${this.name}: SQLite integrity check: ${ic.join("; ")}`];
      }
      c.db.close();
    }
    return problems;
  }

  async _writeNew(name, bytes) {
    try { await this.dir.getFileHandle(name); throw new FileRefused(`${name} exists`, "exists"); }
    catch (e) { if (e instanceof FileRefused) throw e; }
    await this._write(await this.dir.getFileHandle(name, { create: true }), bytes);
    return name;
  }

  // the browser writes to a swap file and puts it in place only on close(): until then the file
  // on disk is the old one, whole
  async _write(fh, bytes) {
    const w = await fh.createWritable({ keepExistingData: false });
    try { await w.write(bytes); } catch (e) { await w.abort().catch(() => {}); throw e; }
    await w.close();
  }

  /** The bytes of the file as it is now, for "save by downloading" in a browser without folders. */
  download() { return this.bytes(); }

  async close() {
    if (this._beat) { this.timers.clearInterval(this._beat); this._beat = null; }
    if (!this.readOnly) {
      const m = await this._readMarker();
      if (m && m.session === this.session) await this.dir.removeEntry(this.markerName).catch(() => {});
    }
    if (this._release) { this._release(); this._release = null; }
    if (this.db) { this.db.close(); this.db = null; }
  }
}

/** The design files in a folder, with what to know about each: its kind, its conflict copies, who
 *  has it open (a fresh marker). Conflict copies and markers are not listed as files of their own. */
export async function listFolder(dir, { now = () => new Date(), schema = TNDB_SCHEMA } = {}) {
  const names = [];
  for await (const [n, h] of dir.entries()) if (h.kind === "file") names.push(n);
  const out = [];
  for (const n of names.sort()) {
    if (n.endsWith(MARKER_SUFFIX) || n.endsWith(".crswap")) continue;
    const kind = kindOf(n, schema);
    if (!kind) continue;
    const base = conflictBase(n);
    if (base && names.includes(base)) continue;
    const f = await (await dir.getFileHandle(n)).getFile();
    let editing = null;
    if (names.includes(n + MARKER_SUFFIX)) {
      try {
        const m = JSON.parse(await (await (await dir.getFileHandle(n + MARKER_SUFFIX)).getFile()).text());
        editing = { ...m, stale: now() - new Date(m.beat) >= MARKER_STALE_MS };
      } catch (e) { editing = null; }
    }
    out.push({ name: n, kind, size: f.size, lastModified: f.lastModified, conflicts: conflictsOf(n, names), editing,
               orphanCopyOf: base && !names.includes(base) ? base : null });
  }
  return out;
}

/** A folder held in memory with the same interface as a picked folder: for a browser that cannot
 *  open folders (the file is opened from a file picker and saved by downloading; no other editor
 *  can be seen from there, and the page says so), and for the tests. */
export class MemoryFolder {
  constructor(name = "memory", files = {}) {
    this.name = name;
    this.kind = "directory";
    this.files = new Map(Object.entries(files).map(([n, b]) => [n, { bytes: new Uint8Array(b), lastModified: Date.now() }]));
  }
  async getFileHandle(n, { create = false } = {}) {
    if (!this.files.has(n)) {
      if (!create) { const e = new Error(`${n}: not found`); e.name = "NotFoundError"; throw e; }
      this.files.set(n, { bytes: new Uint8Array(0), lastModified: Date.now() });
    }
    const folder = this;
    return {
      kind: "file", name: n,
      async getFile() {
        const f = folder.files.get(n);
        if (!f) { const e = new Error(`${n}: not found`); e.name = "NotFoundError"; throw e; }
        const b = f.bytes;
        return { name: n, size: b.length, lastModified: f.lastModified,
          async arrayBuffer() { return b.slice().buffer; }, async text() { return new TextDecoder().decode(b); } };
      },
      async createWritable() {
        const parts = [];
        return {
          async write(d) { parts.push(typeof d === "string" ? new TextEncoder().encode(d) : new Uint8Array(d.buffer ? d.buffer.slice(d.byteOffset, d.byteOffset + d.byteLength) : d)); },
          async close() {
            const len = parts.reduce((a, p) => a + p.length, 0), out = new Uint8Array(len);
            let o = 0; for (const p of parts) { out.set(p, o); o += p.length; }
            const prev = folder.files.get(n);
            folder.files.set(n, { bytes: out, lastModified: Math.max(Date.now(), prev ? prev.lastModified + 1 : 0) });
          },
          async abort() { parts.length = 0; },
        };
      },
    };
  }
  async removeEntry(n) { this.files.delete(n); }
  async *entries() { for (const n of [...this.files.keys()]) yield [n, { kind: "file", name: n }]; }
}
