// structure.js -- the structure of the design (docs/RELEASE_PLAN.md P4): the group files and the
// node files of a design folder, read as one index, held to their rules, and changed by structure
// actions that check their impact first and change every file they touch or none.
//
// A design folder (what tools/seed_design.py writes; on Drive, the shared drive of the design):
//   structure/<group>.group.tndb   one per group: its stages, people, nodes, the edges into its
//                                  nodes, its contracts, its change requests
//   structure/actions/<id>.json    every structure action: what it changed, by whom; while one is
//                                  being written, everything needed to finish it
//   nodes/<id>.node.tndb           one per node; a node never changes file when it changes group
//
// The rules (integrity()): every node in exactly one group, its node file there and saying the
// same group, stage and label; every edge kept by the group of the node that reads, from a node
// that exists and is not archived; every stage, author and contract about a node of the group.
//
// Another group's file is changed only when that group agreed: a change request to it, accepted
// by its lead in its own file (moving a node into it, archiving or merging a node it reads).
// Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
import { checkBytes, openFile, newFileBytes, sha256, FileRefused } from "./tnfile.js";

export const STRUCTURE = "structure";
export const NODES = "nodes";
export const ACTIONS = "actions";
const ID = /^[a-z][a-z0-9_]{1,63}$/;

/** Actions other modules add (release.js: seal, re-issue, stage signatures, comments, imports):
 *  type -> { impact(index, a, x), steps(ws, a, x), summary(a) }, with x the helpers each side uses. */
export const EXTRA = new Map();

export async function subdir(root, name, create = false) { return root.getDirectoryHandle(name, { create }); }
const groupFile = (g) => `${g}.group.tndb`;
const nodeFile = (id) => `${id}.node.tndb`;

export async function readBytes(dir, name) {
  return new Uint8Array(await (await (await dir.getFileHandle(name)).getFile()).arrayBuffer());
}

function all(db, sql, params = []) {
  const st = db.prepare(sql);
  try { st.bind(params); const out = []; while (st.step()) out.push(st.getAsObject()); return out; } finally { st.free(); }
}

export function parseAbout(about) { try { const o = JSON.parse(about); return o && typeof o === "object" ? o : {}; } catch (e) { return {}; } }

// ------------------------------------------------------------------ the index

/** Every group file of the folder, read: { groups: Map(id -> group), owner: Map(node -> group id), problems }. */
export async function loadIndex(SQL, root) {
  const sdir = await subdir(root, STRUCTURE);
  const groups = new Map(), owner = new Map(), problems = [];
  const names = [];
  for await (const [n, h] of sdir.entries()) if (h.kind === "file" && n.endsWith(".group.tndb")) names.push(n);
  for (const name of names.sort()) {
    const c = checkBytes(SQL, await readBytes(sdir, name), { name: `${STRUCTURE}/${name}`, expectKind: "group" });
    if (c.problems.length) { problems.push(...c.problems); if (c.db) c.db.close(); continue; }
    const db = c.db;
    try {
      const info = all(db, "SELECT * FROM group_info")[0] || {};
      const g = {
        id: info.id || name.replace(/\.group\.tndb$/, ""), file: name, label: info.label, lead_team: info.lead_team, lead: info.lead,
        stages: all(db, "SELECT * FROM stage ORDER BY rowid"),
        nodes: new Map(all(db, "SELECT * FROM group_node ORDER BY rowid").map((r) => [r.id, r])),
        edges: all(db, "SELECT * FROM edge ORDER BY rowid"),
        contracts: all(db, "SELECT * FROM contract ORDER BY rowid"),
        members: all(db, "SELECT * FROM member ORDER BY name"),
        memberNodes: new Map(all(db, "SELECT * FROM member_node").map((r) => [r.node, r])),
        crs: all(db, "SELECT * FROM change_request ORDER BY at, id").map((r) => ({ ...r, about: parseAbout(r.about) })),
        revisions: all(db, "SELECT * FROM revision ORDER BY n"),
      };
      if (`${g.id}.group.tndb` !== name) problems.push(`${STRUCTURE}/${name}: its group is ${g.id}; the file is named for another`);
      if (groups.has(g.id)) problems.push(`group ${g.id} has two files`);
      groups.set(g.id, g);
      for (const id of g.nodes.keys()) {
        if (owner.has(id)) problems.push(`node ${id} is in two groups: ${owner.get(id)} and ${g.id}`);
        else owner.set(id, g.id);
      }
    } finally { db.close(); }
  }
  return { groups, owner, problems };
}

/** The nodes of other groups that read `id` (edges kept by their groups), by group. */
export function readersOutside(index, id) {
  const g = index.owner.get(id), out = new Map();
  for (const [gid, G] of index.groups) {
    if (gid === g) continue;
    for (const e of G.edges) if (e.from_node === id) { if (!out.has(gid)) out.set(gid, []); out.get(gid).push(e.to_node); }
  }
  return out;
}

/** The nodes of its own group that read `id`. */
export function readersInside(index, id) {
  const G = index.groups.get(index.owner.get(id));
  return G ? G.edges.filter((e) => e.from_node === id).map((e) => e.to_node) : [];
}

/** Inputs of a group's nodes from other groups: [{ from_node, from_group, to_node }]. */
export function inputsFromOutside(index, gid) {
  const G = index.groups.get(gid);
  return G.edges.filter((e) => index.owner.get(e.from_node) !== gid).map((e) => ({ ...e, from_group: index.owner.get(e.from_node) || "?" }));
}

/** A change request from group `from` to group `to` about (action, node), and its state there:
 *  open, accepted, declined, done. */
export function requestState(index, from, to, action, node) {
  const F = index.groups.get(from), T = index.groups.get(to);
  if (!F || !T) return null;
  const cr = [...F.crs].reverse().find((c) => c.about.to === to && c.about.action === action && c.about.node === node && c.state !== "done");
  if (!cr) return null;
  const reply = [...T.crs].reverse().find((c) => c.about.reply === cr.id);
  return { cr, reply, state: reply ? reply.state : "open" };
}

/** Change requests to group `gid` from the others, each with its state. */
export function requestsTo(index, gid) {
  const out = [];
  for (const [from, F] of index.groups) {
    if (from === gid) continue;
    for (const cr of F.crs) {
      if (cr.about.to !== gid || cr.about.reply) continue;
      const reply = [...index.groups.get(gid).crs].reverse().find((c) => c.about.reply === cr.id);
      out.push({ from, cr, reply, state: cr.state === "done" ? "done" : reply ? reply.state : "open" });
    }
  }
  return out;
}

// ------------------------------------------------------------------ the rules

/** Every problem with the folder's structure. Reads the node files of `scope` (group ids; all when
 *  null): each must pass its format check and say the group, stage and label its group says. */
export async function integrity(SQL, root, { index = null, scope = null } = {}) {
  index = index || await loadIndex(SQL, root);
  const problems = [...index.problems];
  const ndir = await subdir(root, NODES);
  for (const [gid, G] of index.groups) {
    const stages = new Set(G.stages.map((s) => s.id));
    for (const n of G.nodes.values()) {
      if (n.stage && !stages.has(n.stage)) problems.push(`${gid}: node ${n.id} is in stage ${n.stage}, which the group does not have`);
      if (!n.stage && stages.size) problems.push(`${gid}: node ${n.id} is in no stage; the group has stages`);
    }
    const seen = new Set();
    for (const e of G.edges) {
      const k = `${e.from_node}>${e.to_node}>${e.kind}`;
      if (seen.has(k)) problems.push(`${gid}: edge ${e.from_node} -> ${e.to_node} (${e.kind}) twice`);
      seen.add(k);
      if (!G.nodes.has(e.to_node)) problems.push(`${gid}: keeps the edge ${e.from_node} -> ${e.to_node}, but ${e.to_node} is not its node (an edge is kept by the group of the node that reads)`);
      const fg = index.owner.get(e.from_node);
      if (!fg) problems.push(`${gid}: ${e.to_node} reads ${e.from_node}, which is in no group`);
      else if (index.groups.get(fg).nodes.get(e.from_node).state === "archived" && G.nodes.get(e.to_node)?.state !== "archived") problems.push(`${gid}: ${e.to_node} reads ${e.from_node}, which ${fg} archived`);
      if (e.from_node === e.to_node) problems.push(`${gid}: ${e.to_node} reads itself`);
    }
    for (const m of G.memberNodes.values()) if (!G.nodes.has(m.node)) problems.push(`${gid}: ${m.author} authors ${m.node}, which is not its node`);
    for (const c of G.contracts) if (!G.nodes.has(c.node)) problems.push(`${gid}: a contract on ${c.node}, which is not its node`);
    for (const s of G.stages) if (s.owner && !G.members.some((m) => m.name === s.owner)) problems.push(`${gid}: stage ${s.id} is owned by ${s.owner}, who is not a member`);
    if (scope && !scope.includes(gid)) continue;
    for (const n of G.nodes.values()) {
      let bytes;
      try { bytes = await readBytes(ndir, nodeFile(n.id)); } catch (e) { problems.push(`${gid}: node ${n.id} has no file ${NODES}/${nodeFile(n.id)}`); continue; }
      const c = checkBytes(SQL, bytes, { name: `${NODES}/${nodeFile(n.id)}`, expectKind: "node" });
      if (c.problems.length) { problems.push(...c.problems); if (c.db) c.db.close(); continue; }
      const row = all(c.db, "SELECT * FROM node")[0];
      c.db.close();
      if (!row) { problems.push(`${NODES}/${nodeFile(n.id)}: no node row`); continue; }
      for (const [k, want] of [["id", n.id], ["group_id", gid], ["stage", n.stage], ["label", n.label], ["state", n.state]]) {
        if ((row[k] ?? null) !== (want ?? null)) problems.push(`${NODES}/${nodeFile(n.id)}: ${k} is ${JSON.stringify(row[k])}, its group ${gid} says ${JSON.stringify(want)}`);
      }
    }
  }
  if (!scope) {
    for await (const [n, h] of ndir.entries()) {
      if (h.kind !== "file" || !n.endsWith(".node.tndb")) continue;
      const id = n.replace(/\.node\.tndb$/, "");
      if (!index.owner.has(id)) problems.push(`${NODES}/${n}: in no group`);
    }
  }
  return problems;
}

// ------------------------------------------------------------------ impact

/** What an action would do beyond its own node, before it is done:
 *  [{ level: "block" | "warn" | "info", text }]. A block stops it. */
export function impact(index, a) {
  const out = [], G = index.groups.get(a.group);
  const block = (t) => out.push({ level: "block", text: t }), warn = (t) => out.push({ level: "warn", text: t }), info = (t) => out.push({ level: "info", text: t });
  if (!G) { block(`no group ${a.group}`); return out; }
  const node = (id) => G.nodes.get(id);
  const mine = (id) => { if (!node(id)) block(`${id} is not a node of ${a.group}`); return !!node(id); };
  const stageOk = (Gx, st) => {
    if (Gx.stages.length && !Gx.stages.some((s) => s.id === st)) block(`${Gx.id} has no stage ${st || "(none)"}; its stages: ${Gx.stages.map((s) => s.id).join(", ")}`);
    if (!Gx.stages.length && st) block(`${Gx.id} has no stages; leave the stage empty`);
  };
  const newId = (id) => {
    if (!ID.test(id || "")) block(`${id || "(empty)"} is not a node id (a lowercase letter, then lowercase letters, digits and _; 2 to 64)`);
    else if (index.owner.has(id)) block(`${id} is already a node of ${index.owner.get(id)}`);
  };
  const outsideReaders = (id, action, what) => {
    for (const [rg, nodes] of readersOutside(index, id)) {
      const r = requestState(index, a.group, rg, action, id);
      if (r && r.state === "accepted") warn(`${rg} reads ${id} (${nodes.join(", ")}) and accepted change request ${r.cr.id}: ${what} in its file`);
      else block(`${rg} reads ${id} (${nodes.join(", ")}): raise a change request to ${rg} and wait for its lead to accept it${r ? ` (request ${r.cr.id} is ${r.state})` : ""}`);
    }
  };
  switch (a.type) {
    case "add":
      newId(a.id); stageOk(G, a.stage || null);
      if (!a.label) block("a new node needs a label");
      info(`a new node file ${NODES}/${nodeFile(a.id)}, a shell for its author`);
      break;
    case "rename":
      if (mine(a.id)) {
        if (!a.label) block("a label cannot be empty");
        const r = readersOutside(index, a.id);
        if (r.size) info(`${[...r.keys()].join(", ")} read ${a.id}; they see the new label (the id does not change)`);
      }
      break;
    case "stage":
      if (mine(a.id)) stageOk(G, a.stage || null);
      break;
    case "archive":
      if (mine(a.id)) {
        if (node(a.id).state === "archived") block(`${a.id} is archived already`);
        const inside = readersInside(index, a.id).filter((r) => node(r)?.state !== "archived");
        if (inside.length) warn(`${inside.join(", ")} in ${a.group} lose ${a.id} as an input: the edges go`);
        outsideReaders(a.id, "archive", "the edges reading it go");
        info(`${NODES}/${nodeFile(a.id)} is kept, marked archived`);
      }
      break;
    case "split":
      if (mine(a.id)) {
        newId(a.newId);
        if (!a.newLabel) block("the new node needs a label");
        for (const r of a.readers || []) if (!G.edges.some((e) => e.from_node === a.id && e.to_node === r)) block(`${r} does not read ${a.id} inside ${a.group}`);
        const out = readersOutside(index, a.id);
        if (out.size) info(`${[...out.keys()].join(", ")} read ${a.id} and keep reading it; to move them to ${a.newId}, raise a change request`);
        info(`${a.newId} gets ${(a.fields || []).length} field(s) and ${(a.outputs || []).length} output(s) of ${a.id}, a copy of its inputs, and ${(a.readers || []).length} of its readers in ${a.group}`);
      }
      break;
    case "merge":
      if (mine(a.keep) && mine(a.gone)) {
        if (a.keep === a.gone) block("a node cannot be merged into itself");
        outsideReaders(a.gone, "merge", `the edges reading it read ${a.keep}`);
        const inside = readersInside(index, a.gone);
        if (inside.length) info(`${inside.join(", ")} read ${a.keep} instead`);
        info(`${a.gone}'s fields, inputs and outputs go into ${a.keep}; ${NODES}/${nodeFile(a.gone)} is kept, archived, saying where it went`);
      }
      break;
    case "move": {
      if (!mine(a.id)) break;
      const T = index.groups.get(a.to);
      if (!T) { block(`no group ${a.to}`); break; }
      if (a.to === a.group) { block(`${a.id} is in ${a.group} already`); break; }
      stageOk(T, a.stage || null);
      const r = requestState(index, a.group, a.to, "move", a.id);
      if (!r || r.state !== "accepted") block(`moving ${a.id} into ${a.to} needs its lead's agreement: raise a change request to ${a.to}${r ? ` (request ${r.cr.id} is ${r.state})` : ""}`);
      const inside = readersInside(index, a.id);
      if (inside.length) warn(`${inside.join(", ")} in ${a.group} will read ${a.id} from ${a.to}: a boundary, so a contract is published for it in ${a.to}`);
      const inputs = G.edges.filter((e) => e.to_node === a.id);
      if (inputs.length) info(`its ${inputs.length} input edge(s) go with it to ${a.to}`);
      info(`${NODES}/${nodeFile(a.id)} stays where it is, saying group ${a.to}`);
      break;
    }
    case "addStage":
      if (!ID.test(a.stage || "")) block(`${a.stage || "(empty)"} is not a stage id`);
      else if (G.stages.some((s) => s.id === a.stage)) block(`${a.group} has a stage ${a.stage} already`);
      if (!G.stages.length && G.nodes.size) block(`${a.group} has no stages yet: its ${G.nodes.size} nodes would be in none. Ask for stages to be set in design/groups.toml (P9), or add the first stage with nodes: move every node`);
      break;
    case "stageOwner":
      if (!G.stages.some((s) => s.id === a.stage)) block(`${a.group} has no stage ${a.stage}`);
      if (a.owner && !G.members.some((m) => m.name === a.owner)) block(`${a.owner} is not a member of ${a.group}; add them under People first`);
      break;
    case "member":
      if (!a.name) block("a member needs a name");
      if (!["lead", "stage owner", "author"].includes(a.role)) block(`a role is lead, stage owner or author, not ${a.role}`);
      break;
    case "issue":
      if (mine(a.id)) {
        if (!G.members.some((m) => m.name === a.author)) block(`${a.author} is not a member of ${a.group}`);
        if (node(a.id).state === "archived") block(`${a.id} is archived`);
        if (node(a.id).state === "sealed") block(`${a.id} is sealed in a release of ${a.group}: re-issue it (Release) to open it again`);
        const prev = G.memberNodes.get(a.id);
        if (prev && prev.author !== a.author) warn(`${a.id} was issued to ${prev.author}; it goes to ${a.author}`);
        info(`${NODES}/${nodeFile(a.id)} names ${a.author} as its author`);
      }
      break;
    case "contract":
      if (mine(a.node)) {
        const r = readersOutside(index, a.node);
        if (!r.size) warn(`no other group reads ${a.node} yet`);
        const prev = G.contracts.find((c) => c.node === a.node && c.output === a.output);
        if (prev) warn(`${a.node}.${a.output} goes from version ${prev.version} to ${prev.version + 1}: ${[...r.keys()].join(", ") || "its readers"} are told by its change`);
        if (!a.output) block("a contract names an output");
      }
      break;
    case "request":
      if (!index.groups.has(a.to)) block(`no group ${a.to}`);
      if (a.to === a.group) block("a change request goes to another group");
      if (!["move", "archive", "merge", "contract", "other"].includes(a.action)) block(`no such request: ${a.action}`);
      if (!a.body) block("say what is asked, and why");
      break;
    case "reply": {
      const req = requestsTo(index, a.group).find((r) => r.cr.id === a.id);
      if (!req) block(`no change request ${a.id} to ${a.group}`);
      else if (req.state !== "open") block(`change request ${a.id} is ${req.state} already`);
      if (!["accepted", "declined"].includes(a.answer)) block("the answer is accepted or declined");
      break;
    }
    default:
      if (EXTRA.has(a.type)) EXTRA.get(a.type).impact(index, a, { G, node, mine, block, warn, info });
      else block(`no structure action ${a.type}`);
  }
  return out;
}

// ------------------------------------------------------------------ acting

/**
 * The design folder, to change. Every action: impact(); refused on any block; then every file it
 * touches is opened for editing (refused if anyone else has one open), changed, prepared (checked,
 * its revision row in), the whole set written to structure/actions/<id>.json, each file written,
 * and the record marked done. A crash part-way leaves the record: finish() completes it, file by
 * file, from the bytes in it (a file someone changed since is not overwritten: reported).
 */
export class Workspace {
  constructor({ SQL, root, who, session, profile = null, journal, locks = null, now = () => new Date(), timers = globalThis }) {
    Object.assign(this, { SQL, root, who, session, profile, journal, locks, now, timers });
    this.index = null;
  }

  async refresh() { this.index = await loadIndex(this.SQL, this.root); return this.index; }

  async plan(a) {
    if (!this.index) await this.refresh();
    const imp = impact(this.index, a);
    return { action: a, impact: imp, blocked: imp.some((x) => x.level === "block") };
  }

  /** Do it; returns { id, summary, files }. */
  async apply(a) {
    const p = await this.plan(a);
    if (p.blocked) throw new FileRefused(p.impact.filter((x) => x.level === "block").map((x) => x.text).join("\n"), "blocked");
    if (a.type === "split") await this.readNode(a.id);
    if (a.type === "merge") { await this.readNode(a.keep); await this.readNode(a.gone); }
    const steps = this._steps(a);
    const sdir = await subdir(this.root, STRUCTURE), dirs = new Map();
    for (const st of steps) { const top = st.path.split("/")[0]; if (!dirs.has(top)) dirs.set(top, await subdir(this.root, top, !!st.create)); }
    const dirOf = (path) => dirs.get(path.split("/")[0]);
    const base = (path) => path.split("/").pop();
    const sessions = new Map(), creates = [];
    const close = async () => { for (const s of sessions.values()) { await this.journal.delete(s.key).catch(() => {}); await s.close(); } };
    try {
      for (const st of steps) {
        if (st.create) {
          // a new file; or, with overwrite, one that cannot be opened (missing or damaged), made again
          let before = null;
          try { before = await sha256(await readBytes(dirOf(st.path), base(st.path))); } catch (e) { before = null; }
          if (before !== null && !st.overwrite) throw new FileRefused(`${st.path} exists`, "exists");
          // a release file is frozen: it has no history of its own
          const made = (db) => { st.create(db); if (st.kind !== "release") db.run("INSERT INTO revision VALUES (1, ?, ?, ?)", [this.now().toISOString(), this.who, `made: ${summary(a)}`]); };
          const bytes = newFileBytes(this.SQL, st.kind, st.id, made, { writtenBy: `TRI-NETRA group app (${this.who})` });
          creates.push({ path: st.path, bytes, hash: await sha256(bytes), before });
          continue;
        }
        let s = sessions.get(st.path);
        if (!s) {
          s = await openFile({ SQL: this.SQL, dir: dirOf(st.path), name: base(st.path), who: this.who, session: this.session, profile: this.profile,
            journal: this.journal, locks: this.locks, now: this.now, timers: this.timers });
          sessions.set(st.path, s);
          if (s.readOnly) throw new FileRefused(`${st.path} is read-only here: ${s.readOnlyWhy}`, "readonly");
        }
        await s.change(st.summary || summary(a), st.edit);
      }
      const prepared = [];
      try {
        for (const [path, s] of sessions) prepared.push({ path, s, p: await s.prepare(summary(a)) });
      } catch (e) { for (const x of prepared) x.s.unprepare(x.p); throw e; }
      const at = this.now().toISOString();
      const id = `${at.replace(/[-:.TZ]/g, "").slice(0, 14)}-${Math.random().toString(36).slice(2, 8)}`;
      const record = { id, action: a, summary: summary(a), by: this.who, at, impact: p.impact, done: false,
        files: [...creates.map((c) => ({ path: c.path, before: c.before, after: c.hash, bytes: b64(c.bytes) })),
          ...prepared.map((x) => ({ path: x.path, before: x.p.before, after: x.p.hash, bytes: b64(x.p.bytes) }))] };
      const adir = await subdir(sdir, ACTIONS, true);
      await writeFile(adir, `${id}.json`, JSON.stringify(record));
      for (const c of creates) await writeFile(dirOf(c.path), base(c.path), c.bytes);
      for (const x of prepared) await x.s.commit(x.p);
      await writeFile(adir, `${id}.json`, JSON.stringify({ ...record, done: true, files: record.files.map(({ bytes, ...f }) => f) }));
      await close();
      await this.refresh();
      return { id, summary: record.summary, files: record.files.map((f) => f.path) };
    } catch (e) {
      await close();
      throw e;
    }
  }

  /** Structure actions recorded but not finished (a crash part-way). */
  async unfinished() {
    const out = [];
    let adir;
    try { adir = await subdir(await subdir(this.root, STRUCTURE), ACTIONS); } catch (e) { return out; }
    for await (const [n, h] of adir.entries()) {
      if (h.kind !== "file" || !n.endsWith(".json")) continue;
      try { const r = JSON.parse(new TextDecoder().decode(await readBytes(adir, n))); if (!r.done) out.push(r); } catch (e) { out.push({ id: n, broken: true }); }
    }
    return out.sort((x, y) => String(x.at).localeCompare(String(y.at)));
  }

  /** The history of structure actions, newest first. */
  async history() {
    const out = [];
    let adir;
    try { adir = await subdir(await subdir(this.root, STRUCTURE), ACTIONS); } catch (e) { return out; }
    for await (const [n, h] of adir.entries()) {
      if (h.kind !== "file" || !n.endsWith(".json")) continue;
      try { const r = JSON.parse(new TextDecoder().decode(await readBytes(adir, n))); out.push({ id: r.id, at: r.at, by: r.by, summary: r.summary, done: r.done, files: r.files.map((f) => f.path) }); } catch (e) { /* listed by unfinished() */ }
    }
    return out.sort((x, y) => String(y.at).localeCompare(String(x.at)));
  }

  /** Complete an unfinished action: every file still as it was before gets its new bytes; one
   *  already new is left; one changed by someone since is reported and left. */
  async finish(id) {
    const sdir = await subdir(this.root, STRUCTURE), adir = await subdir(sdir, ACTIONS);
    const r = JSON.parse(new TextDecoder().decode(await readBytes(adir, `${id}.json`)));
    const report = [];
    for (const f of r.files) {
      const dir = await subdir(this.root, f.path.split("/")[0], true), name = f.path.split("/").pop();
      let now = null;
      try { now = await sha256(await readBytes(dir, name)); } catch (e) { now = null; }
      if (now === f.after) { report.push(`${f.path}: done already`); continue; }
      if (now !== f.before) { report.push(`${f.path}: changed by someone since; left as it is (compare it with the action's record)`); continue; }
      const bytes = unb64(f.bytes);
      if ((await sha256(bytes)) !== f.after) { report.push(`${f.path}: the record's bytes do not match it; left`); continue; }
      await writeFile(dir, name, bytes);
      report.push(`${f.path}: written`);
    }
    const left = report.filter((x) => !/written|done already/.test(x));
    if (!left.length) await writeFile(adir, `${id}.json`, JSON.stringify({ ...r, done: true, finished_by: this.who, files: r.files.map(({ bytes, ...f }) => f) }));
    await this.refresh();
    return { finished: !left.length, report };
  }

  // the files an action changes, and how: [{ path, edit(db) } | { path, create(db), kind, id }]
  _steps(a) {
    const idx = this.index, G = idx.groups.get(a.group), gpath = `${STRUCTURE}/${groupFile(a.group)}`, np = (id) => `${NODES}/${nodeFile(id)}`;
    const who = this.who, at = this.now().toISOString();
    const typed = `typed by ${who}`;
    const steps = [];
    const g = (edit) => steps.push({ path: gpath, edit });
    const n = (id, edit) => steps.push({ path: np(id), edit });
    switch (a.type) {
      case "add":
        steps.push({ path: np(a.id), kind: "node", id: a.id, create: (db) => {
          db.run("INSERT INTO node VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)", [a.id, a.sheet || a.id, a.group, a.stage || null, a.layer || null, a.kind || "leaf", a.label, "shell", null, 0]);
          db.run("INSERT INTO content VALUES ('identity', 'label', ?, ?), ('identity', 'sheet', ?, ?)", [a.label, typed, a.sheet || a.id, typed]);
        } });
        g((db) => db.run("INSERT INTO group_node VALUES (?, ?, ?, ?, ?, ?, ?)", [a.id, a.sheet || a.id, a.stage || null, a.layer || null, a.kind || "leaf", a.label, "shell"]));
        break;
      case "rename":
        g((db) => db.run("UPDATE group_node SET label = ? WHERE id = ?", [a.label, a.id]));
        n(a.id, (db) => {
          db.run("UPDATE node SET label = ?", [a.label]);
          db.run("UPDATE content SET value = ?, origin = ? WHERE section = 'identity' AND field = 'label'", [a.label, typed]);
          if (!db.getRowsModified()) db.run("INSERT INTO content VALUES ('identity', 'label', ?, ?)", [a.label, typed]);
        });
        break;
      case "stage":
        g((db) => db.run("UPDATE group_node SET stage = ? WHERE id = ?", [a.stage || null, a.id]));
        n(a.id, (db) => db.run("UPDATE node SET stage = ?", [a.stage || null]));
        break;
      case "archive":
        g((db) => {
          db.run("UPDATE group_node SET state = 'archived' WHERE id = ?", [a.id]);
          db.run("DELETE FROM edge WHERE from_node = ?", [a.id]);
          db.run("DELETE FROM contract WHERE node = ?", [a.id]);
        });
        n(a.id, (db) => { db.run("UPDATE node SET state = 'archived'"); db.run("INSERT INTO content VALUES ('identity', 'archived', ?, ?)", [`archived by ${who} on ${at}`, typed]); });
        this._outsideEdits(a, steps, (db) => db.run("DELETE FROM edge WHERE from_node = ?", [a.id]));
        break;
      case "split": {
        const fields = new Set(a.fields || []), outs = new Set(a.outputs || []);
        const src = this._nodeRows(a.id);
        steps.push({ path: np(a.newId), kind: "node", id: a.newId, create: (db) => {
          const o = src.node;
          db.run("INSERT INTO node VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)", [a.newId, a.newId, a.group, o.stage, o.layer, o.kind, a.newLabel, "shell", null, 0]);
          db.run("INSERT INTO content VALUES ('identity', 'label', ?, ?), ('identity', 'sheet', ?, ?), ('identity', 'split_from', ?, ?)", [a.newLabel, typed, a.newId, typed, a.id, typed]);
          for (const c of src.content) if (fields.has(`${c.section}.${c.field}`)) db.run("INSERT INTO content VALUES (?, ?, ?, ?)", [c.section, c.field, c.value, c.origin]);
          for (const i of src.input) db.run("INSERT INTO input VALUES (?, ?, ?, ?)", [i.name, i.from_node, i.from_output, i.unit]);
          for (const o2 of src.output) if (outs.has(o2.name)) db.run("INSERT INTO output VALUES (?, ?, ?, ?, ?, ?)", [o2.name, o2.unit, o2.lower, o2.upper, o2.reason_lower, o2.reason_upper]);
        } });
        n(a.id, (db) => {
          for (const f of fields) { const [sec, ...rest] = f.split("."); db.run("DELETE FROM content WHERE section = ? AND field = ?", [sec, rest.join(".")]); }
          for (const o2 of outs) db.run("DELETE FROM output WHERE name = ?", [o2]);
          db.run("INSERT INTO content VALUES ('identity', 'split_into', ?, ?)", [a.newId, typed]);
        });
        g((db) => {
          const o = G.nodes.get(a.id);
          db.run("INSERT INTO group_node VALUES (?, ?, ?, ?, ?, ?, ?)", [a.newId, a.newId, o.stage, o.layer, o.kind, a.newLabel, "shell"]);
          for (const e of G.edges.filter((x) => x.to_node === a.id)) db.run("INSERT INTO edge VALUES (?, ?, ?, ?)", [e.from_node, a.newId, e.kind, e.label]);
          for (const r of a.readers || []) db.run("UPDATE edge SET from_node = ? WHERE from_node = ? AND to_node = ?", [a.newId, a.id, r]);
        });
        break;
      }
      case "merge": {
        const gone = this._nodeRows(a.gone), keep = this._nodeRows(a.keep);
        const have = new Set(keep.content.map((c) => `${c.section}.${c.field}`)), ins = new Set(keep.input.map((i) => i.name)), outs = new Set(keep.output.map((o) => o.name));
        n(a.keep, (db) => {
          for (const c of gone.content) {
            if (c.section === "identity") continue;
            const f = have.has(`${c.section}.${c.field}`) ? `${c.field} (from ${a.gone})` : c.field;
            db.run("INSERT INTO content VALUES (?, ?, ?, ?)", [c.section, f, c.value, c.origin]);
          }
          for (const i of gone.input) if (!ins.has(i.name)) db.run("INSERT INTO input VALUES (?, ?, ?, ?)", [i.name, i.from_node, i.from_output, i.unit]);
          for (const o of gone.output) if (!outs.has(o.name)) db.run("INSERT INTO output VALUES (?, ?, ?, ?, ?, ?)", [o.name, o.unit, o.lower, o.upper, o.reason_lower, o.reason_upper]);
          db.run("INSERT INTO content VALUES ('identity', 'merged_from', ?, ?)", [a.gone, typed]);
        });
        n(a.gone, (db) => { db.run("UPDATE node SET state = 'archived'"); db.run("INSERT INTO content VALUES ('identity', 'merged_into', ?, ?)", [a.keep, typed]); });
        g((db) => {
          db.run("UPDATE group_node SET state = 'archived' WHERE id = ?", [a.gone]);
          db.run("UPDATE edge SET from_node = ? WHERE from_node = ?", [a.keep, a.gone]);
          db.run("UPDATE edge SET to_node = ? WHERE to_node = ?", [a.keep, a.gone]);
          dedupeEdges(db);
          db.run("UPDATE contract SET node = ? WHERE node = ?", [a.keep, a.gone]);
          db.run("DELETE FROM member_node WHERE node = ?", [a.gone]);
        });
        this._outsideEdits(a, steps, (db) => { db.run("UPDATE edge SET from_node = ? WHERE from_node = ?", [a.keep, a.gone]); dedupeEdges(db); }, a.gone);
        break;
      }
      case "move": {
        const T = idx.groups.get(a.to), row = G.nodes.get(a.id);
        const incoming = G.edges.filter((e) => e.to_node === a.id), contracts = G.contracts.filter((c) => c.node === a.id), mn = G.memberNodes.get(a.id);
        const readersHere = G.edges.filter((e) => e.from_node === a.id).map((e) => e.to_node);
        const req = requestState(idx, a.group, a.to, "move", a.id);
        g((db) => {
          db.run("DELETE FROM group_node WHERE id = ?", [a.id]);
          db.run("DELETE FROM edge WHERE to_node = ?", [a.id]);
          db.run("DELETE FROM contract WHERE node = ?", [a.id]);
          db.run("DELETE FROM member_node WHERE node = ?", [a.id]);
          db.run("UPDATE change_request SET state = 'done' WHERE id = ?", [req.cr.id]);
        });
        steps.push({ path: `${STRUCTURE}/${groupFile(a.to)}`, edit: (db) => {
          db.run("INSERT INTO group_node VALUES (?, ?, ?, ?, ?, ?, ?)", [a.id, row.sheet, a.stage || null, row.layer, row.kind, row.label, row.state]);
          for (const e of incoming) db.run("INSERT INTO edge VALUES (?, ?, ?, ?)", [e.from_node, a.id, e.kind, e.label]);
          for (const c of contracts) db.run("INSERT INTO contract VALUES (?, ?, ?, ?, ?)", [c.node, c.output, c.unit, c.version, c.readers]);
          if (readersHere.length && !contracts.length) db.run("INSERT INTO contract VALUES (?, ?, ?, ?, ?)", [a.id, "*", null, 1, a.group]);
          if (mn) db.run("INSERT OR REPLACE INTO member_node VALUES (?, ?, ?)", [mn.node, mn.author, mn.issued_at]);
          if (mn && !T.members.some((m) => m.name === mn.author)) db.run("INSERT INTO member VALUES (?, 'author')", [mn.author]);
        } });
        n(a.id, (db) => db.run("UPDATE node SET group_id = ?, stage = ?", [a.to, a.stage || null]));
        break;
      }
      case "addStage":
        g((db) => db.run("INSERT INTO stage VALUES (?, ?, ?)", [a.stage, a.label || a.stage, a.owner || null]));
        break;
      case "stageOwner":
        g((db) => db.run("UPDATE stage SET owner = ? WHERE id = ?", [a.owner || null, a.stage]));
        break;
      case "member":
        g((db) => db.run("INSERT OR REPLACE INTO member VALUES (?, ?)", [a.name, a.role]));
        break;
      case "issue":
        g((db) => db.run("INSERT OR REPLACE INTO member_node VALUES (?, ?, ?)", [a.id, a.author, at]));
        n(a.id, (db) => { db.run("UPDATE node SET author = ?", [a.author]); db.run("INSERT INTO content VALUES ('identity', 'issued', ?, ?)", [`issued to ${a.author} by ${who} on ${at}`, typed]); });
        break;
      case "contract": {
        const prev = G.contracts.find((c) => c.node === a.node && c.output === a.output);
        const readers = [...readersOutside(idx, a.node).keys()].join(",");
        g((db) => {
          if (prev) db.run("UPDATE contract SET unit = ?, version = version + 1, readers = ? WHERE node = ? AND output = ?", [a.unit || prev.unit, readers, a.node, a.output]);
          else db.run("INSERT INTO contract VALUES (?, ?, ?, 1, ?)", [a.node, a.output, a.unit || null, readers]);
        });
        n(a.node, (db) => db.run("UPDATE node SET contract_version = contract_version + 1"));
        break;
      }
      case "request": {
        const id = `${a.group}-${at.replace(/[-:.TZ]/g, "").slice(0, 14)}-${Math.random().toString(36).slice(2, 6)}`;
        g((db) => db.run("INSERT INTO change_request VALUES (?, ?, ?, ?, ?, 'open')", [id, at, who, JSON.stringify({ to: a.to, action: a.action, node: a.node || null }), a.body]));
        break;
      }
      case "reply":
        g((db) => db.run("INSERT INTO change_request VALUES (?, ?, ?, ?, ?, ?)", [`${a.id}.reply`, at, who, JSON.stringify({ reply: a.id }), a.body || a.answer, a.answer]));
        break;
      default:
        if (!EXTRA.has(a.type)) throw new FileRefused(`no structure action ${a.type}`, "blocked");
        EXTRA.get(a.type).steps(this, a, { steps, g, n, np, gpath, who, at, typed });
    }
    return steps;
  }

  // the edits in other groups' files an action makes with their accepted change request
  _outsideEdits(a, steps, edit, id = a.id) {
    for (const rg of readersOutside(this.index, id).keys()) {
      const r = requestState(this.index, a.group, rg, a.type, id);
      steps.push({ path: `${STRUCTURE}/${groupFile(rg)}`, summary: `${summary(a)} (change request ${r.cr.id}, accepted)`, edit });
      steps.push({ path: `${STRUCTURE}/${groupFile(a.group)}`, edit: (db) => db.run("UPDATE change_request SET state = 'done' WHERE id = ?", [r.cr.id]) });
    }
  }

  // the rows of a node file, read now (split and merge copy them)
  _nodeRows(id) {
    const rows = this._nodeCache && this._nodeCache.get(id);
    if (!rows) throw new FileRefused(`${id}: read its node file first (Workspace.readNode)`, "internal");
    return rows;
  }

  /** Read a node file's rows (for the map's detail, and before split or merge). */
  async readNode(id) {
    const bytes = await readBytes(await subdir(this.root, NODES), nodeFile(id));
    const c = checkBytes(this.SQL, bytes, { name: `${NODES}/${nodeFile(id)}`, expectKind: "node" });
    if (c.problems.length) { if (c.db) c.db.close(); throw new FileRefused(c.problems.join("\n"), "format"); }
    try {
      const rows = { node: all(c.db, "SELECT * FROM node")[0], content: all(c.db, "SELECT * FROM content ORDER BY rowid"),
        input: all(c.db, "SELECT * FROM input"), output: all(c.db, "SELECT * FROM output"), revision: all(c.db, "SELECT * FROM revision ORDER BY n") };
      if (!this._nodeCache) this._nodeCache = new Map();
      this._nodeCache.set(id, rows);
      return rows;
    } finally { c.db.close(); }
  }
}

/** One line saying what an action is. */
export function summary(a) {
  switch (a.type) {
    case "add": return `add ${a.id} "${a.label}"${a.stage ? ` in ${a.stage}` : ""}`;
    case "rename": return `rename ${a.id} to "${a.label}"`;
    case "stage": return `move ${a.id} to stage ${a.stage || "(none)"}`;
    case "archive": return `archive ${a.id}`;
    case "split": return `split ${a.newId} "${a.newLabel}" out of ${a.id}`;
    case "merge": return `merge ${a.gone} into ${a.keep}`;
    case "move": return `move ${a.id} from ${a.group} to ${a.to}${a.stage ? ` (stage ${a.stage})` : ""}`;
    case "addStage": return `add stage ${a.stage}`;
    case "stageOwner": return `stage ${a.stage} owned by ${a.owner || "nobody"}`;
    case "member": return `${a.name} is ${a.role}`;
    case "issue": return `issue ${a.id} to ${a.author}`;
    case "contract": return `contract ${a.node}.${a.output}`;
    case "request": return `change request to ${a.to}: ${a.action} ${a.node || ""}`.trim();
    case "reply": return `change request ${a.id}: ${a.answer}`;
    default: return EXTRA.has(a.type) ? EXTRA.get(a.type).summary(a) : a.type;
  }
}

function dedupeEdges(db) {
  db.run("DELETE FROM edge WHERE from_node = to_node");
  db.run("DELETE FROM edge WHERE rowid NOT IN (SELECT min(rowid) FROM edge GROUP BY from_node, to_node, kind)");
}

async function writeFile(dir, name, data) {
  const w = await (await dir.getFileHandle(name, { create: true })).createWritable();
  try { await w.write(data); } catch (e) { await w.abort().catch(() => {}); throw e; }
  await w.close();
}

function b64(bytes) {
  let s = "";
  for (let i = 0; i < bytes.length; i += 0x8000) s += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
  return btoa(s);
}
function unb64(s) { return Uint8Array.from(atob(s), (c) => c.charCodeAt(0)); }
