// release.js -- a group from its nodes to a sealed release (docs/RELEASE_PLAN.md P6): the group
// assembled (every node file read, its standing and checks, the checks across nodes, the stages and
// their signatures, what changed since the last release), and the actions that take it to a
// release: a stage owner signs a stage, the lead seals the group (a frozen release file, every node
// file stamped and sealed), re-issues a node (from the file as it is, or from any release), comments
// on a node, and imports a node form (adcs-node-form/1) into a node file. They are structure actions
// (structure.js EXTRA): their impact is shown first, and each changes every file it touches or none.
//
// A release file, releases/<group>-<version>.tnrel, keeps every node as it was sealed: its body
// (the node file's rows but its status and history, as JSON text), whether it was sealed as
// confirmed and why not, and fingerprints (SHA-256) that tools/release.py checks again:
//   release_node.fingerprint = sha256(release_node.content)
//   release.fingerprint      = sha256(the lines "<id> <fingerprint>", sorted, joined by "\n")
//   content.body_fingerprint = sha256(content.body)
// A node is sealed as confirmed only when someone other than its author checked it and nothing has
// changed since, the checks find nothing, its stage (when it has an owner) is signed as it is, and,
// for a computing node, a test vector has its answer from outside the code.
// Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
import { checkBytes, sha256 } from "./tnfile.js";
import { EXTRA, NODES, STRUCTURE, subdir, readBytes, integrity, readersOutside } from "./structure.js";
import { readDoc, check, evidenceDebt, standing } from "./node_model.js";
import { CATALOG } from "./node_catalog.js";

export const RELEASES = "releases";
export const FORM_SCHEMA = "adcs-node-form/1";
const relFile = (g, v) => `${g}-${v}.tnrel`;
const nodeFile = (id) => `${id}.node.tndb`;
const DATA = new WeakMap();   // an action -> what was read for it (kept out of the action's record)

function arr(db, sql, p = []) {
  const st = db.prepare(sql);
  try { st.bind(p); const out = []; while (st.step()) out.push(st.get()); return out; } finally { st.free(); }
}
function objs(db, sql, p = []) {
  const st = db.prepare(sql);
  try { st.bind(p); const out = []; while (st.step()) out.push(st.getAsObject()); return out; } finally { st.free(); }
}
export const hashText = (t) => sha256(new TextEncoder().encode(t));
function b64(bytes) {
  let s = "";
  for (let i = 0; i < bytes.length; i += 0x8000) s += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
  return btoa(s);
}
const unb64 = (s) => Uint8Array.from(atob(s), (c) => c.charCodeAt(0));
const docOf = (db) => readDoc({ query: (sql, p) => arr(db, sql, p) });

// ------------------------------------------------------------------ a node's body

/** What a node file says, as the release keeps it: every row but its status, comments, requests and
 *  history; the node row without its state (the group's); pictures as base64. JSON text. */
export function nodeBody(db) {
  const node = objs(db, "SELECT id, sheet, group_id, stage, layer, kind, label, author, contract_version FROM node")[0] || null;
  return JSON.stringify({
    node,
    content: arr(db, "SELECT section, field, value, origin FROM content WHERE section <> 'status' ORDER BY section, field, rowid"),
    input: arr(db, "SELECT name, from_node, from_output, unit FROM input ORDER BY name"),
    output: arr(db, "SELECT name, unit, lower, upper, reason_lower, reason_upper FROM output ORDER BY name"),
    fixture: arr(db, "SELECT name, inputs, expected, tolerance, source, outside FROM fixture ORDER BY name"),
    attachment: arr(db, "SELECT name, mime, size, bytes FROM attachment ORDER BY name").map(([n, m, s, b]) => [n, m, s, b ? b64(b) : ""]),
    signature: arr(db, "SELECT role, name, at, statement FROM signature ORDER BY at, role, name"),
  });
}

/** The SQL that puts a body back into a node file (its content, inputs, output, test vectors,
 *  pictures, signatures, author and contract version); the group's fields stay as the group says. */
export function restoreBody(db, bodyText) {
  const b = JSON.parse(bodyText);
  db.run("DELETE FROM content WHERE section <> 'status'");
  for (const t of ["input", "output", "fixture", "attachment", "signature"]) db.run(`DELETE FROM ${t}`);
  for (const r of b.content) db.run("INSERT INTO content (section, field, value, origin) VALUES (?, ?, ?, ?)", r);
  for (const r of b.input) db.run("INSERT INTO input VALUES (?, ?, ?, ?)", r);
  for (const r of b.output) db.run("INSERT INTO output VALUES (?, ?, ?, ?, ?, ?)", r);
  for (const r of b.fixture) db.run("INSERT INTO fixture VALUES (?, ?, ?, ?, ?, ?)", r);
  for (const [n, m, s, x] of b.attachment) db.run("INSERT INTO attachment VALUES (?, ?, ?, ?)", [n, m, s, x ? unb64(x) : null]);
  for (const r of b.signature) db.run("INSERT INTO signature VALUES (?, ?, ?, ?)", r);
  if (b.node) db.run("UPDATE node SET author = ?, contract_version = ?", [b.node.author, b.node.contract_version ?? 0]);
}

// ------------------------------------------------------------------ the group assembled

export function leadOf(G) { return G.lead || (G.members.find((m) => m.role === "lead") || {}).name || null; }
export function isLead(G, who) { return !!who && (G.lead === who || G.members.some((m) => m.name === who && m.role === "lead")); }

/** What the checks need to know about every node of the design (as the node app builds it). */
export function designContext(index) {
  const nodes = new Map();
  for (const [gid, G] of index.groups) for (const n of G.nodes.values()) {
    const spec = CATALOG.rows[n.id];
    nodes.set(n.id, { label: n.label, group: gid, layer: n.layer, kind: n.kind, quantity: spec ? spec[2] : null, unit: spec ? spec[3] : null });
  }
  return { nodes };
}

/** Every node of the group read and judged: { gid, G, nodes, cross, stages, releases, last, removed,
 *  structure, unreadable, lead, next, signatures }. */
export async function assemble(ws, gid) {
  const idx = await ws.refresh();
  const G = idx.groups.get(gid);
  if (!G) throw new Error(`no group ${gid}`);
  const ndir = await subdir(ws.root, NODES), sdir = await subdir(ws.root, STRUCTURE);
  const ctx = designContext(idx);
  const nodes = [], unreadable = [];
  for (const n of G.nodes.values()) {
    if (n.state === "archived") continue;
    let bytes;
    try { bytes = await readBytes(ndir, nodeFile(n.id)); } catch (e) { unreadable.push(`${n.id}: no file ${NODES}/${nodeFile(n.id)}`); continue; }
    const c = checkBytes(ws.SQL, bytes, { name: `${NODES}/${nodeFile(n.id)}`, expectKind: "node" });
    if (c.problems.length) { unreadable.push(...c.problems); if (c.db) c.db.close(); continue; }
    try {
      const doc = docOf(c.db), body = nodeBody(c.db);
      nodes.push({ id: n.id, row: n, doc, body, bodyFp: await hashText(body), standing: await standing(doc),
        problems: check(doc, ctx).filter((p) => p.level === "!"), debt: evidenceDebt(doc),
        author: (G.memberNodes.get(n.id) || {}).author || doc.node.author || null, sealedIn: doc.content["status.release"] || null,
        comments: doc.comments, gaps: (() => { try { return JSON.parse(doc.content["status.gaps"] || "[]"); } catch (e) { return []; } })() });
    } finally { c.db.close(); }
  }
  let signatures = [];
  { const c = checkBytes(ws.SQL, await readBytes(sdir, `${gid}.group.tndb`), { expectKind: "group" }); if (c.db) { signatures = objs(c.db, "SELECT role, name, at, statement FROM signature ORDER BY at"); c.db.close(); } }
  const cross = crossChecks(G, idx, nodes);
  const stages = [];
  for (const s of G.stages) {
    const mine = nodes.filter((x) => x.row.stage === s.id);
    const fp = await stageFingerprint(mine);
    const sig = [...signatures].reverse().find((x) => x.role === `stage ${s.id}`);
    let st = {}; try { st = JSON.parse(sig ? sig.statement : "{}"); } catch (e) { st = {}; }
    stages.push({ id: s.id, label: s.label, owner: s.owner, nodes: mine.length, fingerprint: fp, sig: sig || null, signed: !!sig && st.fingerprint === fp && sig.name === s.owner });
  }
  for (const x of nodes) { x.cross = cross.filter((c) => c.node === x.id); const v = verdict(x, stages); x.confirmed = v.ok; x.why = v.why; }
  const releases = await listReleases(ws, gid);
  const last = releases.length ? releases[releases.length - 1] : null;
  for (const x of nodes) x.since = !last ? "new" : !last.nodes.has(x.id) ? "new" : last.nodes.get(x.id).bodyFp === x.bodyFp ? "same" : "changed";
  const removed = last ? [...last.nodes.keys()].filter((id) => !nodes.some((x) => x.id === id)) : [];
  const structure = await integrity(ws.SQL, ws.root, { index: idx, scope: [gid] });
  return { gid, G, nodes, cross, stages, releases, last, removed, structure, unreadable, lead: leadOf(G), next: nextVersion(releases), signatures };
}

async function stageFingerprint(nodes) { return hashText(nodes.map((x) => `${x.id} ${x.bodyFp}`).sort().join("\n")); }

/** The checks across the group's nodes: [{ node, level: "!" | "i", text }]. */
export function crossChecks(G, index, nodes) {
  const out = [], byId = new Map(nodes.map((x) => [x.id, x]));
  const say = (node, level, text) => out.push({ node, level, text });
  for (const x of nodes) {
    for (const i of x.doc.inputs) {
      if (!i.from_node) continue;
      const og = index.owner.get(i.from_node);
      if (!og) { say(x.id, "!", `${x.id} reads ${i.from_node}, which is in no group`); continue; }
      const src = index.groups.get(og).nodes.get(i.from_node);
      if (src.state === "archived") say(x.id, "!", `${x.id} reads ${i.from_node}, which ${og} archived`);
      if (og === G.id) {
        const s = byId.get(i.from_node), q = s && s.doc.content["output.quantity"];
        if (q && i.quantity && q !== i.quantity) say(x.id, "!", `${x.id} reads ${i.from_node} as ${i.quantity}; ${i.from_node} answers ${q}`);
      } else if (!index.groups.get(og).contracts.some((c) => c.node === i.from_node)) {
        say(x.id, "i", `${x.id} reads ${i.from_node} of ${og}, which has published no contract on it yet`);
      }
      if (!G.edges.some((e) => e.from_node === i.from_node && e.to_node === x.id)) say(x.id, "i", `${x.id} reads ${i.from_node}, but the map has no arrow ${i.from_node} → ${x.id}`);
    }
  }
  for (const c of G.contracts) {
    const s = byId.get(c.node), u = s && s.doc.content["output.unit"];
    if (u && c.unit && u !== c.unit) say(c.node, "!", `the contract on ${c.node} says ${c.unit}; ${c.node} answers in ${u}`);
    if (s && !readersOutside(index, c.node).size) say(c.node, "i", `a contract on ${c.node}, which no other group reads now`);
  }
  return out;
}

/** Whether a node can be sealed as confirmed, and why not. */
export function verdict(x, stages = []) {
  const why = [], st = x.standing;
  if (st.state !== "checked") {
    why.push(st.state === "ready" ? "marked ready, not yet checked by a second engineer" : st.state === "draft" ? "a draft: not marked ready" : "a shell: nothing written yet");
    if (st.checkedStale) why.push("its check signature is stale: it changed after it was checked");
  }
  if (x.problems.length) why.push(`${x.problems.length} problem(s) the checks find (${[...new Set(x.problems.map((p) => p.code))].slice(0, 6).join(", ")})`);
  for (const c of x.cross || []) if (c.level === "!") why.push(c.text);
  if (x.doc.kind === "computed" && !x.doc.fixtures.some((f) => f.outside)) why.push("a computing node with no test vector whose answer comes from outside the code");
  const s = stages.find((y) => y.id === x.row.stage);
  if (s && s.owner && !s.signed) why.push(`stage ${s.id} is not signed by its owner ${s.owner}${s.sig ? " since it last changed" : ""}`);
  return { ok: !why.length, why };
}

// ------------------------------------------------------------------ releases

const vkey = (v) => v.split(".").map(Number);
export function versionOrder(a, b) { const [x, y] = [vkey(a), vkey(b)]; return x[0] - y[0] || x[1] - y[1]; }
export function nextVersion(releases) {
  if (!releases.length) return "1.0";
  const [maj, min] = vkey(releases[releases.length - 1].version);
  return `${maj}.${min + 1}`;
}

/** The releases of a group in releases/, oldest first, each read and its fingerprints checked. */
export async function listReleases(ws, gid) {
  let rdir;
  try { rdir = await subdir(ws.root, RELEASES); } catch (e) { return []; }
  const out = [];
  for await (const [n, h] of rdir.entries()) {
    if (h.kind !== "file" || !n.startsWith(`${gid}-`) || !n.endsWith(".tnrel")) continue;
    if (!/^\d+\.\d+$/.test(n.slice(gid.length + 1, -6))) continue;
    out.push(await readRelease(ws.SQL, await readBytes(rdir, n), `${RELEASES}/${n}`));
  }
  return out.filter((r) => r.group === gid).sort((a, b) => versionOrder(a.version, b.version));
}

/** One release file: its row, its nodes, and the problems found re-checking its fingerprints. */
export async function readRelease(SQL, bytes, name = "the release") {
  const c = checkBytes(SQL, bytes, { name, expectKind: "release" });
  if (c.problems.length) { if (c.db) c.db.close(); return { file: name, problems: c.problems, nodes: new Map(), version: "0.0" }; }
  try {
    const r = objs(c.db, "SELECT * FROM release")[0] || {};
    const problems = [], nodes = new Map(), lines = [];
    for (const rn of objs(c.db, "SELECT id, fingerprint, content FROM release_node ORDER BY id")) {
      let x = {}; try { x = JSON.parse(rn.content); } catch (e) { problems.push(`${name}: ${rn.id}: its content is not JSON`); }
      if ((await hashText(rn.content)) !== rn.fingerprint) problems.push(`${name}: ${rn.id}: its fingerprint does not match its content`);
      if (x.body !== undefined && (await hashText(x.body)) !== x.body_fingerprint) problems.push(`${name}: ${rn.id}: its body fingerprint does not match its body`);
      lines.push(`${rn.id} ${rn.fingerprint}`);
      nodes.set(rn.id, { fp: rn.fingerprint, bodyFp: x.body_fingerprint, body: x.body, sealed_as: x.sealed_as, why: x.why || [], work: x.work, author: x.author, stage: x.stage });
    }
    if ((await hashText(lines.sort().join("\n"))) !== r.fingerprint) problems.push(`${name}: the release fingerprint does not match its nodes`);
    const signatures = objs(c.db, "SELECT role, name, at, statement FROM signature ORDER BY at");
    return { file: name, group: r.group_id, version: r.version, sealed_at: r.sealed_at, sealed_by: r.sealed_by, fingerprint: r.fingerprint, nodes, signatures, problems,
      confirmed: [...nodes.values()].filter((x) => x.sealed_as === "confirmed").length };
  } finally { c.db.close(); }
}

/** Two releases (or a release and the group now), node by node: [{ id, change, fields }]. */
export function compare(a, b) {
  const out = [];
  for (const id of [...new Set([...a.keys(), ...b.keys()])].sort()) {
    const x = a.get(id), y = b.get(id);
    if (!x) { out.push({ id, change: "added", fields: [] }); continue; }
    if (!y) { out.push({ id, change: "removed", fields: [] }); continue; }
    if (x.bodyFp === y.bodyFp) { out.push({ id, change: "same", fields: [] }); continue; }
    out.push({ id, change: "changed", fields: bodyDiff(x.body, y.body) });
  }
  return out;
}

function bodyDiff(p, q) {
  const A = JSON.parse(p), B = JSON.parse(q), out = [];
  const ca = new Map(A.content.map((r) => [`${r[0]}.${r[1]}`, r[2]])), cb = new Map(B.content.map((r) => [`${r[0]}.${r[1]}`, r[2]]));
  for (const k of [...new Set([...ca.keys(), ...cb.keys()])].sort()) if (ca.get(k) !== cb.get(k)) out.push(k);
  for (const t of ["input", "output", "fixture", "attachment", "signature"]) if (JSON.stringify(A[t]) !== JSON.stringify(B[t])) out.push(t === "fixture" ? "test vectors" : t === "attachment" ? "pictures" : t === "signature" ? "signatures" : t === "input" ? "inputs" : "output");
  for (const k of ["author", "contract_version", "stage", "label"]) if (JSON.stringify(A.node?.[k]) !== JSON.stringify(B.node?.[k])) out.push(`node ${k}`);
  return out;
}

/** The group now, in the shape compare() takes. */
export function nowNodes(asm) { return new Map(asm.nodes.map((x) => [x.id, { bodyFp: x.bodyFp, body: x.body }])); }

// ------------------------------------------------------------------ node forms (adcs-node-form/1)

// written in two pieces: this module is inlined into a page, where the whole tag would end its script
const FORM_OPEN = "<" + 'script type="application/json" id="adcs-node-form">', FORM_CLOSE = "</" + "script>";

/** The JSON block of a node form's HTML; throws, saying why, when it is not one. */
export function parseForm(html) {
  const i = html.indexOf(FORM_OPEN);
  if (i < 0) throw new Error(`holds no ${FORM_SCHEMA} block: it is not a node form`);
  const j = html.indexOf(FORM_CLOSE, i + FORM_OPEN.length);
  const f = JSON.parse(html.slice(i + FORM_OPEN.length, j));
  if (f.schema !== FORM_SCHEMA) throw new Error(`its schema is ${JSON.stringify(f.schema)}, not ${FORM_SCHEMA}`);
  return f;
}

/** The node fields a form's request proposes, in the node app's keys; inputs and test vectors
 *  mapped too (`sheetToId` turns the form's sheet ids into node ids). */
export function formFields(f, sheetToId = (s) => s) {
  const r = f.request || {}, p = r.proposed || {}, out = {};
  const put = (k, v) => { if (v !== undefined && v !== null && String(v).trim() !== "" && v !== "[]") out[k] = typeof v === "string" ? v : String(v); };
  const putJ = (k, v) => { if (Array.isArray(v) ? v.length : v && Object.keys(v).length) out[k] = JSON.stringify(v); };
  put("identity.question", p.question); put("identity.note", p.note); putJ("identity.tags", p.tags || []);
  const o = p.output || {};
  put("output.symbol", o.symbol); put("output.quantity", o.type); put("output.unit", o.unit);
  for (const k of ["lower", "upper"]) put(`output.${k}`, o[k] === "" || o[k] === undefined ? null : String(o[k]).replace(/\.0$/, ""));
  put("output.reason_lower", o.reason_lower); put("output.reason_upper", o.reason_upper);
  const rel = p.relation || {};
  put("relation.expression", rel.expression); put("relation.source", rel.source); put("relation.why", rel.why); put("relation.how_to_read", rel.reading);
  putJ("relation.derivation", (p.steps || []).map((s) => s.text).filter(Boolean));
  putJ("assumptions", (p.assumptions || []).map((a) => ({ assumes: a.text || "", until: a.fails_when || "" })));
  const kind = (f.node || {}).kind;
  if (kind === "required") { put("requirement.sense", { "<=": "at_most", ">=": "at_least" }[p.sense]); put("requirement.value", p.value); }
  else put("value.number", p.value);
  const ev = p.evidence || {};
  put("evidence.metric", ev.metric); putJ("evidence.rungs", ev.rungs || []);
  const ex = p.explain || {};
  for (const [k, v] of Object.entries(ex)) { if (Array.isArray(v)) putJ(`explain.${k}`, v); else put(`explain.${k}`, v); }
  putJ("sources.new", (r.new_sources || []).map((s) => ({ id: s.id || "", title: s.title || "", where: s.where || "" })));
  const d = r.derisk || {};
  for (const k of ["area", "believed", "status", "tested", "now_know", "plan_change", "cost_k"]) put(`belief.${k}`, d[k]);
  const o2 = r.other || {};
  put("other.subject", o2.subject); put("other.description", o2.description);
  const inputs = (p.inputs || []).filter((i) => i.from).map((i) => ({ name: i.binding || "", from_node: sheetToId(i.from), quantity: i.type || null, unit: null }));
  const fixtures = (p.fixtures || []).map((x, k) => ({ name: x.label || `v${k + 1}`, inputs: x.inputs || {}, expected: x.expect, tolerance: x.tolerance, provenance: x.provenance || "", source: x.source || "", where: x.where || "" }));
  const attachments = (r.attachments || []).filter((a) => String(a.data || "").startsWith("data:") && String(a.data).includes(",")).map((a) => {
    const [head, body] = String(a.data).split(",", 2);
    const bytes = head.includes(";base64") ? unb64(body) : new TextEncoder().encode(decodeURIComponent(body));
    return { name: String(a.name || "attachment").replace(/[^A-Za-z0-9._-]/g, "_"), mime: a.type || head.slice(5).split(";")[0] || "application/octet-stream", bytes };
  });
  return { fields: out, inputs, fixtures, attachments, by: r.requested_by || "", team: r.team || "", type: r.type || "" };
}

const OUTSIDE = ["independent-derivation", "published-source", "independent-tool", "physical-bound"];

// ------------------------------------------------------------------ the actions

/** The release side of a design folder: read what an action needs, then plan or apply it through
 *  the Workspace (all or nothing, its impact first). */
export class Releases {
  constructor(ws) { this.ws = ws; }

  async _prepare(a) {
    const ws = this.ws;
    a.by = ws.who;
    const d = {};
    if (a.type === "seal" || a.type === "signStage") {
      d.asm = await assemble(ws, a.group);
      if (a.type === "seal") {
        a.version = a.version || d.asm.next;
        d.rows = [];
        for (const x of d.asm.nodes) {
          const content = JSON.stringify({ body: x.body, body_fingerprint: x.bodyFp, sealed_as: x.confirmed ? "confirmed" : "unconfirmed", why: x.why,
            work: x.standing.state, author: x.author, stage: x.row.stage || null });
          d.rows.push({ id: x.id, content, fp: await hashText(content) });
        }
        d.fp = await hashText(d.rows.map((r) => `${r.id} ${r.fp}`).sort().join("\n"));
      }
    } else {
      await ws.refresh();
    }
    if (a.type === "reissue" || a.type === "import" || a.type === "comment") {
      d.file = "ok";
      try {
        const c = checkBytes(ws.SQL, await readBytes(await subdir(ws.root, NODES), nodeFile(a.id)), { name: `${NODES}/${nodeFile(a.id)}`, expectKind: "node" });
        if (c.problems.length) { d.file = "damaged"; d.fileWhy = c.problems.join("; "); }
        else {
          d.doc = docOf(c.db);
          // what the file itself holds (readDoc also shows the spec's values as starting values)
          d.inputsOwnCount = arr(c.db, "SELECT count(*) FROM input")[0][0];
          d.raw = new Map(arr(c.db, "SELECT section || '.' || field, value, origin FROM content").map(([k, v, o]) => [k, { v, o }]));
        }
        if (c.db) c.db.close();
      } catch (e) { d.file = "missing"; }
    }
    if (a.type === "reissue" && a.from) {
      const rels = await listReleases(ws, a.group);
      const r = rels.find((x) => x.version === a.from);
      d.release = r || null;
      d.body = r && r.nodes.has(a.id) ? r.nodes.get(a.id).body : null;
    }
    if (a.type === "import") {
      // the form itself stays out of the action's record (its pick-lists are large): what it is about is kept
      if (a.form) { d.formRaw = a.form; a.about = (a.form.node || {}).tree_id || null; a.request = (a.form.request || {}).type || null; delete a.form; }
      else d.formRaw = (DATA.get(a) || {}).formRaw || {};
      const sheetToId = new Map();
      for (const G of ws.index.groups.values()) for (const n of G.nodes.values()) if (n.sheet) sheetToId.set(n.sheet, n.id);
      d.form = formFields(d.formRaw, (s) => sheetToId.get(s) || s);
      const raw = d.raw || new Map();
      d.fill = []; d.skip = [];
      for (const [k, v] of Object.entries(d.form.fields)) {
        const have = raw.get(k);
        if (have && String(have.o || "").startsWith("typed by") && have.v !== v) d.skip.push(k);
        else if (have && have.v === v) continue;
        else d.fill.push([k, v]);
      }
      d.addInputs = d.doc && !d.inputsOwnCount ? d.form.inputs : [];
      const have = new Set(d.doc ? d.doc.fixtures.map((x) => x.name) : []);
      d.addFixtures = d.form.fixtures.filter((x) => !have.has(x.name));
      const pics = new Set(d.doc ? d.doc.attachments.map((x) => x.name) : []);
      d.addPictures = d.form.attachments.filter((x) => !pics.has(x.name));
    }
    DATA.set(a, d);
    return d;
  }

  async plan(a) { await this._prepare(a); return this.ws.plan(a); }
  /** Read everything again (the folder may have changed since the plan), then do it. */
  async apply(a) { await this._prepare(a); return this.ws.apply(a); }
  assemble(gid) { return assemble(this.ws, gid); }
  releases(gid) { return listReleases(this.ws, gid); }
}

const say = (a) => `${a.group} ${a.version}`;

EXTRA.set("seal", {
  summary: (a) => `seal ${say(a)}`,
  impact(index, a, { G, block, warn, info }) {
    const d = DATA.get(a);
    if (!d) { block("read the group first (Releases.plan)"); return; }
    const asm = d.asm;
    if (!isLead(G, a.by)) block(`only ${G.id}'s lead seals it: ${a.by} is not its lead (${asm.lead ? `the lead is ${asm.lead}` : "it has no lead yet: People → a member with the role lead"})`);
    if (a.version !== asm.next) block(`the next release of ${G.id} is ${asm.next}, not ${a.version}`);
    for (const p of asm.structure.slice(0, 8)) block(p);
    if (asm.structure.length > 8) block(`and ${asm.structure.length - 8} more structure problem(s)`);
    for (const p of asm.unreadable) block(`${p}: re-issue it from a release, or repair it`);
    const counts = { new: 0, changed: 0, same: 0 };
    for (const x of asm.nodes) counts[x.since]++;
    if (asm.last && !counts.new && !counts.changed && !asm.removed.length) block(`nothing has changed since ${G.id} ${asm.last.version}`);
    info(`${RELEASES}/${relFile(G.id, a.version)}: a new release file, frozen; it keeps every node as sealed, so any node can be re-issued from it`);
    info(`${asm.nodes.length} node file(s) stamped "${say(a)}" and sealed: their authors change them again only when the lead re-issues them; no signature goes stale`);
    if (asm.last) info(`since ${asm.last.version}: ${counts.new} new, ${counts.changed} changed, ${asm.removed.length} gone, ${counts.same} unchanged`);
    const conf = asm.nodes.filter((x) => x.confirmed), un = asm.nodes.filter((x) => !x.confirmed);
    info(`${conf.length} of ${asm.nodes.length} node(s) sealed as confirmed`);
    if (un.length) warn(`${un.length} node(s) sealed UNCONFIRMED, each with its reason (Assemble lists them): ${un.slice(0, 8).map((x) => x.id).join(", ")}${un.length > 8 ? ", …" : ""}`);
    const noOutside = asm.nodes.filter((x) => x.doc.kind === "computed" && !x.doc.fixtures.some((f) => f.outside));
    if (noOutside.length) warn(`${noOutside.length} computing node(s) have no test vector with an answer from outside the code: they cannot be sealed as confirmed`);
    const unsigned = asm.stages.filter((s) => s.owner && s.nodes && !s.signed);
    if (unsigned.length) warn(`stage(s) not signed by their owner as they are now: ${unsigned.map((s) => s.id).join(", ")}`);
  },
  steps(ws, a, { steps, g, n, who, at }) {
    const d = DATA.get(a), asm = d.asm, G = asm.G, v = a.version, stamp = say(a);
    const live = new Set(asm.nodes.map((x) => x.id));
    steps.push({ path: `${RELEASES}/${relFile(G.id, v)}`, kind: "release", id: `${G.id}-${v}`, create: (db) => {
      db.run("INSERT INTO release VALUES (?, ?, ?, ?, ?)", [G.id, v, at, who, d.fp]);
      for (const r of d.rows) db.run("INSERT INTO release_node VALUES (?, ?, ?)", [r.id, r.fp, r.content]);
      db.run("INSERT INTO group_info VALUES (?, ?, ?, ?)", [G.id, G.label, G.lead_team, G.lead]);
      for (const s of G.stages) db.run("INSERT INTO stage VALUES (?, ?, ?)", [s.id, s.label, s.owner]);
      for (const x of G.nodes.values()) db.run("INSERT INTO group_node VALUES (?, ?, ?, ?, ?, ?, ?)", [x.id, x.sheet, x.stage, x.layer, x.kind, x.label, live.has(x.id) ? "sealed" : x.state]);
      for (const e of G.edges) db.run("INSERT INTO edge VALUES (?, ?, ?, ?)", [e.from_node, e.to_node, e.kind, e.label]);
      for (const c of G.contracts) db.run("INSERT INTO contract VALUES (?, ?, ?, ?, ?)", [c.node, c.output, c.unit, c.version, c.readers]);
      for (const s of asm.stages) if (s.signed) db.run("INSERT INTO signature VALUES (?, ?, ?, ?)", [s.sig.role, s.sig.name, s.sig.at, s.sig.statement]);
      db.run("INSERT INTO signature VALUES (?, ?, ?, ?)", ["sealed", who, at, JSON.stringify({ statement: `sealed ${stamp}`, version: v, fingerprint: d.fp })]);
    } });
    g((db) => {
      db.run("INSERT INTO signature VALUES (?, ?, ?, ?)", ["sealed", who, at, JSON.stringify({ statement: `sealed ${stamp}`, version: v, fingerprint: d.fp })]);
      db.run("UPDATE group_node SET state = 'sealed' WHERE state <> 'archived'");
    });
    for (const x of asm.nodes) n(x.id, (db) => {
      db.run("DELETE FROM content WHERE section = 'status' AND field IN ('release', 'sealed_as', 'sealed_body')");
      db.run("INSERT INTO content VALUES ('status', 'release', ?, ?), ('status', 'sealed_as', ?, ?), ('status', 'sealed_body', ?, ?)",
        [stamp, `sealed by ${who} on ${at}`, x.confirmed ? "confirmed" : "unconfirmed", `sealed by ${who} on ${at}`, x.bodyFp, `sealed by ${who} on ${at}`]);
      db.run("UPDATE node SET state = 'sealed'");
    });
  },
});

EXTRA.set("signStage", {
  summary: (a) => `sign stage ${a.stage} of ${a.group}`,
  impact(index, a, { G, block, warn, info }) {
    const d = DATA.get(a), s = d && d.asm.stages.find((x) => x.id === a.stage);
    if (!s) { block(`${G.id} has no stage ${a.stage}`); return; }
    if (!s.owner) block(`stage ${a.stage} has no owner yet: set one under Stages`);
    else if (s.owner !== a.by) block(`stage ${a.stage} is signed by its owner, ${s.owner}; you are ${a.by}`);
    info(`signs the ${s.nodes} node(s) of stage ${a.stage} as they are now; a change to any of them takes the signature off`);
    const open = d.asm.nodes.filter((x) => x.row.stage === a.stage && x.standing.state !== "checked");
    if (open.length) warn(`${open.length} of them are not checked by a second engineer yet: they are sealed UNCONFIRMED whatever the stage's signature`);
  },
  steps(ws, a, { g, who, at }) {
    const s = DATA.get(a).asm.stages.find((x) => x.id === a.stage);
    g((db) => db.run("INSERT INTO signature VALUES (?, ?, ?, ?)", [`stage ${a.stage}`, who, at, JSON.stringify({ statement: `stage ${a.stage} signed`, fingerprint: s.fingerprint })]));
  },
});

EXTRA.set("reissue", {
  summary: (a) => `re-issue ${a.id}${a.from ? ` from ${a.group} ${a.from}` : ""}`,
  impact(index, a, { G, node, mine, block, warn, info }) {
    const d = DATA.get(a);
    if (!mine(a.id)) return;
    if (!isLead(G, a.by)) block(`only ${G.id}'s lead re-issues a node: ${a.by} is not its lead`);
    if (node(a.id).state === "archived") block(`${a.id} is archived`);
    if (a.from) {
      if (!d.release) block(`${G.id} has no release ${a.from}`);
      else if (!d.body) block(`${G.id} ${a.from} does not hold ${a.id}`);
      else info(`${NODES}/${nodeFile(a.id)} gets back what ${G.id} ${a.from} sealed (its signatures too, which stand while nothing else changes)`);
    } else if (node(a.id).state !== "sealed") block(`${a.id} is not sealed: its author can change it already`);
    if (d.file === "missing") (a.from ? warn : block)(`${NODES}/${nodeFile(a.id)} is missing${a.from ? `: it is made again from ${a.from}` : ": re-issue it from a release"}`);
    if (d.file === "damaged") (a.from ? warn : block)(`${NODES}/${nodeFile(a.id)} cannot be opened (${d.fileWhy})${a.from ? `: it is made again from ${a.from}` : ": re-issue it from a release"}`);
    const au = G.memberNodes.get(a.id);
    info(au ? `issued again to ${au.author}, who can change it until the next seal` : `open again; nobody is its author yet (Map → Issue)`);
  },
  steps(ws, a, { steps, g, n, np, who, at, typed }) {
    const d = DATA.get(a), G = ws.index.groups.get(a.group), row = G.nodes.get(a.id);
    const note = `re-issued by ${who} on ${at}${a.from ? ` from ${a.group} ${a.from}` : ""}`;
    g((db) => {
      db.run("UPDATE group_node SET state = 'issued' WHERE id = ?", [a.id]);
      db.run("UPDATE member_node SET issued_at = ? WHERE node = ?", [at, a.id]);
    });
    const mark = (db) => {
      db.run("UPDATE node SET state = 'issued'");
      // a status line: it says what happened to the file, not what the node says, so no signature goes stale
      db.run("DELETE FROM content WHERE section = 'status' AND field = 'reissued'");
      db.run("INSERT INTO content VALUES ('status', 'reissued', ?, ?)", [note, typed]);
    };
    if (d.file === "ok") n(a.id, (db) => { if (a.from) restoreBody(db, d.body); mark(db); });
    else steps.push({ path: np(a.id), kind: "node", id: a.id, overwrite: true, create: (db) => {
      const b = JSON.parse(d.body);
      db.run("INSERT INTO node VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)", [a.id, row.sheet, a.group, row.stage, row.layer, row.kind, row.label, "issued", null, 0]);
      restoreBody(db, d.body);
      if (!b.node) db.run("UPDATE node SET author = ?", [(G.memberNodes.get(a.id) || {}).author || null]);
      mark(db);
    } });
  },
});

EXTRA.set("comment", {
  summary: (a) => `comment on ${a.id}`,
  impact(index, a, { mine, block, info }) {
    const d = DATA.get(a);
    if (!mine(a.id)) return;
    if (!String(a.body || "").trim()) block("a comment needs words");
    if (d.file !== "ok") block(`${NODES}/${nodeFile(a.id)} cannot be opened: re-issue it from a release first`);
    info(`${NODES}/${nodeFile(a.id)}: a comment its author sees on the node's Home${a.parent ? ` (a reply to ${a.parent})` : ""}`);
  },
  steps(ws, a, { n, who, at }) {
    const id = `${a.group}-${at.replace(/[-:.TZ]/g, "").slice(0, 14)}-${Math.random().toString(36).slice(2, 6)}`;
    n(a.id, (db) => db.run("INSERT INTO comment VALUES (?, ?, ?, ?, ?, ?, 0)", [id, at, who, a.id, a.parent || null, a.body.trim()]));
  },
});

EXTRA.set("import", {
  summary: (a) => `import node form ${a.name || ""} into ${a.id}`.replace("  ", " "),
  impact(index, a, { G, node, mine, block, warn, info }) {
    const d = DATA.get(a);
    if (!mine(a.id)) return;
    if (a.about !== a.id) block(`the form is about ${a.about || "another node"}, not ${a.id}`);
    if (a.request === "new") block("a request for a new node: add the node on the Map first, then import its form into it");
    if (node(a.id).state === "sealed") block(`${a.id} is sealed: re-issue it first, then import`);
    if (d.file !== "ok") block(`${NODES}/${nodeFile(a.id)} cannot be opened: re-issue it from a release first`);
    if (!String(d.form.by).trim()) block("the form names nobody as its requester (P01): it cannot be taken in");
    info(`${d.fill.length} field(s) filled from the form (marked as the form's, by ${d.form.by || "?"})`);
    if (d.skip.length) warn(`${d.skip.length} field(s) its author has typed are kept, not replaced: ${d.skip.slice(0, 8).join(", ")}${d.skip.length > 8 ? ", …" : ""}`);
    if (d.addInputs.length) info(`${d.addInputs.length} input(s) from the form`);
    if (d.addFixtures.length) info(`${d.addFixtures.length} test vector(s) from the form`);
    if (d.addPictures.length) info(`${d.addPictures.length} attachment(s) from the form`);
    if (!d.fill.length && !d.addInputs.length && !d.addFixtures.length && !d.addPictures.length) block("the form adds nothing this node does not already have");
  },
  steps(ws, a, { n }) {
    const d = DATA.get(a), origin = `form ${a.name || "node form"} (${d.form.by})`;
    n(a.id, (db) => {
      for (const [k, v] of d.fill) {
        const [sec, ...rest] = k.split(".");
        db.run("DELETE FROM content WHERE section = ? AND field = ?", [sec, rest.join(".")]);
        db.run("INSERT INTO content VALUES (?, ?, ?, ?)", [sec, rest.join("."), v, origin]);
      }
      const sym = d.fill.find(([k]) => k === "output.symbol");
      if (sym) {
        const get = (k) => (d.fill.find(([x]) => x === k) || [null, d.doc.content[k] ?? null])[1];
        const num = (x) => (x === null || x === "" || !Number.isFinite(Number(x)) ? null : Number(x));
        db.run("DELETE FROM output");
        db.run("INSERT INTO output VALUES (?, ?, ?, ?, ?, ?)", [sym[1], get("output.unit"), num(get("output.lower")), num(get("output.upper")), get("output.reason_lower"), get("output.reason_upper")]);
      }
      for (const i of d.addInputs) db.run("INSERT OR REPLACE INTO input VALUES (?, ?, ?, ?)", [i.name, i.from_node, i.quantity, i.unit]);
      for (const x of d.addFixtures) db.run("INSERT INTO fixture VALUES (?, ?, ?, ?, ?, ?)", [x.name, JSON.stringify(x.inputs), x.expected === undefined || x.expected === null || x.expected === "" ? null : String(x.expected),
        x.tolerance === undefined || x.tolerance === null || x.tolerance === "" ? null : Number(x.tolerance), JSON.stringify({ provenance: x.provenance, source: x.source, where: x.where }), OUTSIDE.includes(x.provenance) ? 1 : 0]);
      for (const p of d.addPictures) db.run("INSERT INTO attachment VALUES (?, ?, ?, ?)", [p.name, p.mime, p.bytes.length, p.bytes]);
    });
  },
});
