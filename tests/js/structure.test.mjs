// structure.test.mjs -- the proof of docs/RELEASE_PLAN.md P4 on the whole seeded design (20 groups,
// 734 nodes): every group opens; the largest (act) and the smallest (catalogue) are restructured
// with every action (add, rename, stage, split, merge, archive, people, stage owners, issue,
// contracts); a node is moved between two groups through a change request its new group accepts;
// what another group reads cannot be taken from it without its agreement; an action cut short by a
// crash is finished from its record; and after all of it no node file is broken.
//
//   node tests/js/structure.test.mjs DESIGN_DIR
// DESIGN_DIR: a folder tools/seed_design.py wrote; changed in place (tests/test_structure.py then
// checks it with tools/tndb.py and tools/group.py, independently of this code).
// Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
import { createRequire } from "node:module";
import { readFileSync } from "node:fs";
import { NodeFolder, MemoryJournal } from "./node_folder.mjs";

const require = createRequire(import.meta.url);
const ROOT = new URL("../../", import.meta.url).pathname;
const initSqlJs = require(ROOT + "design/vendor/sqljs/sql-wasm.js");
const SQL = await initSqlJs({ wasmBinary: readFileSync(ROOT + "design/vendor/sqljs/sql-wasm.wasm") });
const T = await import(ROOT + "design/js/tnfile.js");
const S = await import(ROOT + "design/js/structure.js");

const root = new NodeFolder(process.argv[2]);
const timers = { setInterval: () => 0, clearInterval() {} };
const ws = (who, session = who) => new S.Workspace({ SQL, root, who, session, profile: "test", journal: new MemoryJournal(), timers });
const lead = ws("Lead of act");
let passed = 0, failed = 0;
async function test(name, fn) {
  try { await fn(); passed++; console.log(`PASS ${name}`); } catch (e) { failed++; console.log(`FAIL ${name}: ${e.stack || e}`); }
}
function ok(c, msg) { if (!c) throw new Error(msg); }
async function refused(p, code, words) {
  try { await p; } catch (e) { ok(e.code === code, `refused with ${e.code}, not ${code}: ${e.message}`); if (words) ok(e.message.includes(words), `the refusal says "${words}": ${e.message}`); return e; }
  throw new Error(`not refused (wanted ${code})`);
}
const clean = async (scope = null) => { const p = await S.integrity(SQL, root, { scope }); ok(p.length === 0, `integrity:\n${p.slice(0, 20).join("\n")}`); };

await test("the seeded design holds every rule, and all 20 groups open", async () => {
  const idx = await lead.refresh();
  ok(idx.groups.size === 20, `${idx.groups.size} groups`);
  ok(idx.owner.size === 734, `${idx.owner.size} nodes`);
  await clean();
  const sdir = await S.subdir(root, S.STRUCTURE);
  for (const g of idx.groups.keys()) {
    const s = await T.openFile({ SQL, dir: sdir, name: `${g}.group.tndb`, who: "Lead", session: "x", journal: new MemoryJournal(), timers });
    ok(!s.readOnly, `${g} opens for editing`);
    await s.close();
  }
});

const act = () => lead.index.groups.get("act");
let splitSrc, inReader, outsideRead;

await test("act: people, stage owners, issuing node files", async () => {
  await lead.apply({ type: "member", group: "act", name: "Asha", role: "author" });
  await lead.apply({ type: "member", group: "act", name: "Ravi", role: "stage owner" });
  const st = act().stages[0].id;
  await refused(lead.apply({ type: "stageOwner", group: "act", stage: st, owner: "Nobody" }), "blocked", "not a member");
  await lead.apply({ type: "stageOwner", group: "act", stage: st, owner: "Ravi" });
  ok(act().stages[0].owner === "Ravi", "stage owner set");
  const id = [...act().nodes.keys()][0];
  await lead.apply({ type: "issue", group: "act", id, author: "Asha" });
  ok(act().memberNodes.get(id).author === "Asha", "issued");
  ok((await lead.readNode(id)).node.author === "Asha", "the node file names its author");
});

await test("act: add, rename, move between stages", async () => {
  const [s0, s1] = act().stages.map((s) => s.id);
  await refused(lead.apply({ type: "add", group: "act", id: "Bad Id", label: "x", stage: s0 }), "blocked", "not a node id");
  await refused(lead.apply({ type: "add", group: "act", id: [...act().nodes.keys()][0], label: "x", stage: s0 }), "blocked", "already a node");
  await refused(lead.apply({ type: "add", group: "act", id: "act_new_wheel_model", label: "x", stage: "nowhere" }), "blocked", "no stage");
  const r = await lead.apply({ type: "add", group: "act", id: "act_new_wheel_model", label: "Wheel friction model", stage: s0, kind: "leaf", layer: "3" });
  ok(r.files.includes("nodes/act_new_wheel_model.node.tndb"), "a node file made");
  await lead.apply({ type: "rename", group: "act", id: "act_new_wheel_model", label: "Wheel friction and stiction" });
  await lead.apply({ type: "stage", group: "act", id: "act_new_wheel_model", stage: s1 });
  const n = await lead.readNode("act_new_wheel_model");
  ok(n.node.label === "Wheel friction and stiction" && n.node.stage === s1, "the node file follows");
  ok(n.revision.length === 3, `three revisions: ${n.revision.length}`);
  await clean(["act"]);
});

await test("act: split a node, its readers in the group following the new one", async () => {
  const idx = lead.index;
  splitSrc = [...act().nodes.keys()].find((id) => S.readersInside(idx, id).length && !S.readersOutside(idx, id).size);
  ok(splitSrc, "a node with readers inside act only");
  inReader = S.readersInside(idx, splitSrc)[0];
  const rows = await lead.readNode(splitSrc);
  const fields = rows.content.filter((c) => c.section !== "identity").slice(0, 1).map((c) => `${c.section}.${c.field}`);
  await lead.apply({ type: "split", group: "act", id: splitSrc, newId: "act_split_part", newLabel: "Split part", fields, readers: [inReader] });
  ok(act().edges.some((e) => e.from_node === "act_split_part" && e.to_node === inReader), "the reader reads the new node");
  const n = await lead.readNode("act_split_part");
  ok(fields.every((f) => n.content.some((c) => `${c.section}.${c.field}` === f)), "the fields went with it");
  const o = await lead.readNode(splitSrc);
  ok(fields.every((f) => !o.content.some((c) => `${c.section}.${c.field}` === f)), "and left the original");
  await clean(["act"]);
});

await test("act: merge two nodes", async () => {
  const idx = lead.index;
  const cands = [...act().nodes.values()].filter((n) => n.state !== "archived" && !S.readersOutside(idx, n.id).size && n.stage === act().nodes.get("act_split_part").stage);
  const [keep, gone] = [cands.find((n) => n.id === "act_split_part"), cands.find((n) => n.id !== "act_split_part" && n.id !== splitSrc)];
  ok(keep && gone, "two nodes to merge");
  await lead.apply({ type: "merge", group: "act", keep: keep.id, gone: gone.id });
  ok(act().nodes.get(gone.id).state === "archived", "the merged node is archived");
  ok(!act().edges.some((e) => e.from_node === gone.id || e.to_node === gone.id), "no edge names it");
  ok((await lead.readNode(gone.id)).content.some((c) => c.field === "merged_into" && c.value === keep.id), "its file says where it went");
  await clean(["act"]);
});

await test("act: what another group reads cannot be archived without its agreement; with it, it can", async () => {
  const idx = lead.index;
  outsideRead = [...act().nodes.keys()].find((id) => S.readersOutside(idx, id).size === 1);
  ok(outsideRead, "a node one other group reads");
  const [rg] = S.readersOutside(idx, outsideRead).keys();
  await refused(lead.apply({ type: "archive", group: "act", id: outsideRead }), "blocked", `${rg} reads`);
  await lead.apply({ type: "request", group: "act", to: rg, action: "archive", node: outsideRead, body: "Superseded by the wheel friction model." });
  await refused(lead.apply({ type: "archive", group: "act", id: outsideRead }), "blocked", "is open");
  const theirs = ws(`Lead of ${rg}`);
  await theirs.refresh();
  const req = S.requestsTo(theirs.index, rg).find((r) => r.cr.about.node === outsideRead);
  ok(req && req.state === "open", "the request reaches the other group");
  await theirs.apply({ type: "reply", group: rg, id: req.cr.id, answer: "accepted", body: "We read the new model instead." });
  await lead.refresh();
  await lead.apply({ type: "archive", group: "act", id: outsideRead });
  ok(!lead.index.groups.get(rg).edges.some((e) => e.from_node === outsideRead), `${rg} no longer reads it`);
  ok(S.requestsTo(lead.index, rg).find((r) => r.cr.id === req.cr.id).state === "done", "the request is done");
  await clean();
});

await test("catalogue: the smallest group restructured", async () => {
  const cat = () => lead.index.groups.get("catalogue");
  ok(cat().nodes.size === 8 && !cat().stages.length, "8 nodes, no stages");
  await refused(lead.apply({ type: "add", group: "catalogue", id: "cat_new_part", label: "x", stage: "s" }), "blocked", "no stages");
  await lead.apply({ type: "add", group: "catalogue", id: "cat_new_part", label: "Part datasheet intake" });
  await lead.apply({ type: "rename", group: "catalogue", id: "cat_new_part", label: "Datasheet intake" });
  const idx = lead.index;
  const src = [...cat().nodes.keys()].find((id) => id !== "cat_new_part" && !S.readersOutside(idx, id).size);
  await lead.apply({ type: "split", group: "catalogue", id: src, newId: "cat_split", newLabel: "Catalogue split", fields: [], readers: [] });
  await lead.apply({ type: "merge", group: "catalogue", keep: "cat_new_part", gone: "cat_split" });
  const other = [...cat().nodes.values()].find((n) => n.state !== "archived" && !S.readersOutside(idx, n.id).size && n.id !== "cat_new_part" && n.id !== src);
  await lead.apply({ type: "archive", group: "catalogue", id: other.id });
  ok(cat().nodes.size === 10, `${cat().nodes.size} nodes (8 + 2 added, archived ones kept)`);
  await clean(["catalogue"]);
});

await test("a node moves between two groups once its new group accepts it", async () => {
  const id = "act_new_wheel_model";
  await refused(lead.apply({ type: "move", group: "act", id, to: "ctl" }), "blocked", "needs its lead's agreement");
  await lead.apply({ type: "request", group: "act", to: "ctl", action: "move", node: id, body: "Friction compensation belongs with the controller." });
  const ctl = ws("Lead of ctl");
  await ctl.refresh();
  const req = S.requestsTo(ctl.index, "ctl").find((r) => r.cr.about.node === id);
  await ctl.apply({ type: "reply", group: "ctl", id: req.cr.id, answer: "accepted" });
  await lead.refresh();
  await refused(lead.apply({ type: "move", group: "act", id, to: "ctl", stage: "x" }), "blocked", "no stages");
  await lead.apply({ type: "move", group: "act", id, to: "ctl" });
  ok(lead.index.owner.get(id) === "ctl", "ctl has it");
  ok(!lead.index.groups.get("act").nodes.has(id), "act does not");
  const n = await lead.readNode(id);
  ok(n.node.group_id === "ctl" && n.node.stage === null, "its node file says ctl");
  await clean();
});

await test("a node file someone else has open stops the action, and nothing changes", async () => {
  const ndir = await S.subdir(root, S.NODES);
  const id = [...lead.index.groups.get("catalogue").nodes.values()].find((n) => n.state !== "archived").id;
  const other = await T.openFile({ SQL, dir: ndir, name: `${id}.node.tndb`, who: "Asha", session: "her-laptop", journal: new MemoryJournal(), timers });
  const before = JSON.stringify([...lead.index.groups.get("catalogue").nodes.values()]);
  await refused(lead.apply({ type: "rename", group: "catalogue", id, label: "Renamed" }), "readonly", "Asha");
  await lead.refresh();
  ok(JSON.stringify([...lead.index.groups.get("catalogue").nodes.values()]) === before, "the group file did not change");
  await other.close();
  await clean(["catalogue"]);
});

await test("an action cut short by a crash is finished from its record", async () => {
  // the group file is written, then the computer dies before the node file is
  const id = [...lead.index.groups.get("catalogue").nodes.values()].find((n) => n.state !== "archived").id;
  const proto = T.FileSession.prototype, commit = proto.commit;
  let n = 0;
  proto.commit = async function (p) { if (++n === 2) throw new Error("power cut"); return commit.call(this, p); };
  try { await lead.apply({ type: "rename", group: "catalogue", id, label: "Renamed after a crash" }); throw new Error("not cut short"); }
  catch (e) { ok(e.message === "power cut", e.message); }
  finally { proto.commit = commit; }
  const problems = await S.integrity(SQL, root, { scope: ["catalogue"] });
  ok(problems.some((p) => p.includes("label")), `half done shows: ${problems}`);
  const [u] = await lead.unfinished();
  ok(u && u.action.type === "rename", "the record is there");
  const r = await lead.finish(u.id);
  ok(r.finished, r.report.join("; "));
  ok((await lead.unfinished()).length === 0, "nothing left unfinished");
  await clean();
});

await test("history: every structure action recorded", async () => {
  const h = await lead.history();
  ok(h.length >= 20 && h.every((x) => x.done), `${h.length} actions, all done`);
  ok(h.some((x) => x.summary.startsWith("move act_new_wheel_model from act to ctl")), "the move is in it");
});

console.log(`structure: ${passed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
