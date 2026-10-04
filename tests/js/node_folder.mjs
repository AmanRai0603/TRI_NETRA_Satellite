// node_folder.mjs -- a folder on disk with the browser's folder interface (FileSystemDirectoryHandle),
// for running the pages' file and structure code under Node in the tests. A write goes to a
// temporary file renamed into place on close(), as the browser's swap file does.
// Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
import { promises as fs } from "node:fs";
import path from "node:path";

export class NodeFolder {
  constructor(dir) { this.dir = dir; this.name = path.basename(dir); this.kind = "directory"; }
  async getDirectoryHandle(n, { create = false } = {}) {
    const p = path.join(this.dir, n);
    try { if (!(await fs.stat(p)).isDirectory()) throw notFound(n); }
    catch (e) { if (!create) throw notFound(n); await fs.mkdir(p, { recursive: true }); }
    return new NodeFolder(p);
  }
  async getFileHandle(n, { create = false } = {}) {
    const p = path.join(this.dir, n);
    try { await fs.stat(p); } catch (e) { if (!create) throw notFound(n); await fs.writeFile(p, new Uint8Array(0)); }
    return {
      kind: "file", name: n,
      async getFile() {
        const st = await fs.stat(p).catch(() => { throw notFound(n); });
        return { name: n, size: st.size, lastModified: Math.floor(st.mtimeMs),
          async arrayBuffer() { const b = await fs.readFile(p); return b.buffer.slice(b.byteOffset, b.byteOffset + b.byteLength); },
          async text() { return fs.readFile(p, "utf8"); } };
      },
      async createWritable() {
        const tmp = `${p}.crswap`, parts = [];
        return {
          async write(d) { parts.push(typeof d === "string" ? Buffer.from(d) : Buffer.from(d.buffer ? new Uint8Array(d.buffer, d.byteOffset, d.byteLength) : d)); },
          async close() { await fs.writeFile(tmp, Buffer.concat(parts)); await fs.rename(tmp, p); },
          async abort() { parts.length = 0; },
        };
      },
    };
  }
  async removeEntry(n) { await fs.rm(path.join(this.dir, n)); }
  async *entries() {
    for (const d of await fs.readdir(this.dir, { withFileTypes: true })) yield [d.name, { kind: d.isDirectory() ? "directory" : "file", name: d.name }];
  }
}

function notFound(n) { const e = new Error(`${n}: not found`); e.name = "NotFoundError"; return e; }

/** The Journal's interface, in memory. */
export class MemoryJournal {
  constructor() { this.m = new Map(); this.p = new Map(); }
  async get(k) { return this.m.get(k); }
  async put(k, v) { this.m.set(k, v); }
  async delete(k) { this.m.delete(k); }
  async keys() { return [...this.m.keys()]; }
  async pref(k) { return this.p.get(k); }
  async setPref(k, v) { this.p.set(k, v); }
}
