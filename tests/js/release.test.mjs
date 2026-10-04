// release.test.mjs -- the proof of docs/RELEASE_PLAN.md P6 on the whole seeded design (20 groups,
// 734 nodes), under Node on a folder on disk: every group assembles and seals 1.0; a node is
// re-issued, changed, checked again and the group seals 1.1, which compares against 1.0 node by node;
// a computing node with no test vector from outside the code cannot be sealed as confirmed; a stage
// is signed only by its owner and a group sealed only by its lead; a damaged node file is made again
// from a release; a lead's comment reaches the node file; a node form (adcs-node-form/1) is imported.
//
//   node tests/js/release.test.mjs DESIGN_DIR FORM.html
// DESIGN_DIR: a folder tools/seed_design.py wrote; changed in place (tests/test_release.py then
// checks it, and every release file, with tools/tndb.py, tools/group.py and tools/release.py).
// FORM.html: a filled node form for m2_4 (tests/test_release.py makes it with spec/tools/forms.py).
// Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
import { createRequire } from "node:module";
import { readFileSync } from "node:fs";
import path from "node:path";
import { NodeFolder, MemoryJournal } from "./node_folder.mjs";

const require = createRequire(import.meta.url);
const ROOT = new URL("../../", import.meta.url).pathname;
const initSqlJs = require(ROOT + "design/vendor/sqljs/sql-wasm.js");
const SQL = await initSqlJs({ wasmBinary: readFileSync(ROOT + "design/vendor/sqljs/sql-wasm.wasm") });
const T = await import(ROOT + "design/js/tnfile.js");
const S = await import(ROOT + "design/js/structure.js");
const R = await import(ROOT + "design/js/release.js");
const M = await import(ROOT + "design/js/node_model.js");

const [designDir, formPath] = process.argv.slice(2);
const root = new NodeFolder(designDir);
const timers = { setInterval: () => 0, clearInterval() {} };
const ws = (who) => new S.Workspace({ SQL, root, who, session: who, profile: "test", journal: new MemoryJournal(), timers });
const rel = (who) => new R.Releases(ws(who));
let passed = 0, failed = 0;
async function test(name, fn) {
  try { await fn(); passed++; console.log(`PASS ${name}`); } catch (e) { failed++; console.log(`FAIL ${name}: ${e.stack || e}`); }
}
function ok(c, msg) { if (!c) throw new Error(msg); }
async function refused(p, code, words) {
  try { await p; } catch (e) { ok(e.code === code, `refused with ${e.code}, not ${code}: ${e.message}`); if (words) ok(e.message.includes(words), `the refusal says "${words}": ${e.message}`); return e; }
  throw new Error(`not refused (wanted ${code})`);
}
const clean = async () => { const p = await S.integrity(SQL, root); ok(p.length === 0, `integrity:\n${p.slice(0, 20).join("\n")}`); };
const lead = (g) => `Lead of ${g}`;

/** Open a node file as `who`, change it with fn(db, doc), save it. */
async function edit(who, id, summary, fn) {
  const s = await T.openFile({ SQL, dir: await S.subdir(root, S.NODES), name: `${id}.node.tndb`, who, session: who, profile: "test", journal: new MemoryJournal(), timers });
  try {
    ok(!s.readOnly, `${id} opens for ${who}: ${s.readOnlyWhy}`);
    await s.change(summary, (db) => fn(db, M.readDoc(s)));
    await s.save();
    return M.readDoc(s);
  } finally { await s.close(); }
}
async function signNode(who, id, role, statement) {
  const fp = await M.fingerprint((await readNode(id)).doc);
  return edit(who, id, role, (db) => db.run("INSERT INTO signature VALUES (?, ?, ?, ?)", [role, who, new Date().toISOString(), JSON.stringify({ statement, fingerprint: fp })]));
}
async function readNode(id) {
  const c = T.checkBytes(SQL, await S.readBytes(await S.subdir(root, S.NODES), `${id}.node.tndb`), { expectKind: "node" });
  try { return { doc: M.readDoc({ query: (q, p) => { const st = c.db.prepare(q); try { st.bind(p || []); const o = []; while (st.step()) o.push(st.get()); return o; } finally { st.free(); } } }), problems: c.problems }; }
  finally { if (c.db) c.db.close(); }
}
const BELIEF = { "belief.area": "model", "belief.believed": "The spec's value holds for the reference satellite.", "belief.status": "held",
  "belief.tested": "Compared with the reference case's sizing script.", "belief.now_know": "It holds within 2 %.", "belief.plan_change": "None; the value is kept." };

let groups = [];
await test("every group assembles: every node read and judged, nothing sealed yet, the next release 1.0", async () => {
  const idx = await ws("x").refresh();
  groups = [...idx.groups.keys()];
  ok(groups.length === 20, `${groups.length} groups`);
  let n = 0;
  for (const g of groups) {
    const a = await rel("x").assemble(g);
    ok(!a.structure.length && !a.unreadable.length, `${g}: ${a.structure.concat(a.unreadable).slice(0, 5).join("; ")}`);
    ok(a.next === "1.0" && !a.releases.length, `${g}: next ${a.next}`);
    ok(a.nodes.every((x) => !x.confirmed && x.why.length), `${g}: a seeded shell is not confirmed`);
    n += a.nodes.length;
  }
  ok(n === 734, `${n} nodes assembled`);
});

await test("only the lead seals: someone else is refused, by name", async () => {
  const r = rel("Asha");
  await refused(r.apply({ type: "seal", group: "act" }), "blocked", "only act's lead seals it");
});

await test("people: a lead for every group; in act an author, a checker and the owner of stage mtq", async () => {
  for (const g of groups) await ws(lead(g)).apply({ type: "member", group: g, name: lead(g), role: "lead" });
  const w = ws(lead("act"));
  for (const [name, role] of [["Asha", "author"], ["Ravi", "author"], ["Meera", "stage owner"]]) await w.apply({ type: "member", group: "act", name, role });
  await w.apply({ type: "stageOwner", group: "act", stage: "mtq", owner: "Meera" });
  await w.apply({ type: "issue", group: "act", id: "gm_0", author: "Asha" });
  await clean();
});

await test("gm_0 written, marked ready by Asha and checked by Ravi; confirmed only once Meera signs stage mtq", async () => {
  await edit("Asha", "gm_0", "fill", (db, doc) => {
    M.adoptSpec(db, doc);
    for (const [k, v] of Object.entries({ "explain.where_it_breaks": "Above the coil's rated current the dipole no longer grows linearly.", ...BELIEF })) M.setField(db, k, v, "Asha");
  });
  let { doc } = await readNode("gm_0");
  ok(M.check(doc, R.designContext((await ws("x").refresh()))).filter((p) => p.level === "!").length === 0, "gm_0: nothing left to fix");
  await signNode("Asha", "gm_0", "author ready", "ready for the group's review");
  await signNode("Ravi", "gm_0", "checked by", "the value against the datasheet");
  let a = await rel(lead("act")).assemble("act");
  let x = a.nodes.find((y) => y.id === "gm_0");
  ok(x.standing.state === "checked", `gm_0 is ${x.standing.state}`);
  ok(!x.confirmed && x.why.some((w) => w.includes("stage mtq is not signed by its owner Meera")), `why: ${x.why.join("; ")}`);
  await refused(rel("Asha").apply({ type: "signStage", group: "act", stage: "mtq" }), "blocked", "signed by its owner, Meera");
  await rel("Meera").apply({ type: "signStage", group: "act", stage: "mtq" });
  a = await rel(lead("act")).assemble("act");
  x = a.nodes.find((y) => y.id === "gm_0");
  ok(x.confirmed, `gm_0 confirmed: ${x.why.join("; ")}`);
  ok(a.stages.find((s) => s.id === "mtq").signed, "stage mtq signed as it is");
});

await test("a computing node with no test vector from outside the code cannot be sealed as confirmed", async () => {
  let x = (await rel("x").assemble("env")).nodes.find((y) => y.id === "m2_4");
  ok(x.doc.kind === "computed", "m2_4 computes");
  ok(x.why.some((w) => w.includes("no test vector whose answer comes from outside the code")), `why: ${x.why.join("; ")}`);
  await edit("Asha", "m2_4", "a test vector from a book", (db) => M.setFixtures(db, [{ name: "v1", inputs: { h: 500000 }, expected: 6878137, tolerance: 1e-12, provenance: "published-source", source: "vallado2013", where: "p. 98, eq. 2-58" }]));
  x = (await rel("x").assemble("env")).nodes.find((y) => y.id === "m2_4");
  ok(!x.why.some((w) => w.includes("outside the code")), `the reason goes once a published answer is there: ${x.why.join("; ")}`);
  await edit("Asha", "m2_4", "a code-made answer only", (db) => M.setFixtures(db, [{ name: "v1", inputs: { h: 500000 }, expected: 6878137, tolerance: 1e-12, provenance: "code-output", source: "vallado2013", where: "p. 98" }]));
  x = (await rel("x").assemble("env")).nodes.find((y) => y.id === "m2_4");
  ok(x.why.some((w) => w.includes("outside the code")), "an answer the code made proves nothing");
});

await test("all 20 groups seal 1.0: frozen release files that check, every node file stamped and sealed", async () => {
  for (const g of groups) {
    const r = rel(lead(g));
    const p = await r.plan({ type: "seal", group: g });
    ok(!p.blocked, `${g}: ${p.impact.filter((i) => i.level === "block").map((i) => i.text).join("; ")}`);
    const done = await r.apply({ type: "seal", group: g });
    ok(done.files.includes(`releases/${g}-1.0.tnrel`), `${g}: ${done.files.slice(0, 3)}`);
  }
  for (const g of groups) {
    const [r1] = await rel("x").releases(g);
    ok(r1 && r1.version === "1.0" && !r1.problems.length, `${g}: ${r1 && r1.problems.join("; ")}`);
  }
  const act = (await rel("x").releases("act"))[0];
  ok(act.nodes.get("gm_0").sealed_as === "confirmed", "gm_0 sealed as confirmed");
  ok(act.nodes.get("gm_1").sealed_as === "unconfirmed" && act.nodes.get("gm_1").why.some((w) => w.includes("a shell")), "a shell sealed UNCONFIRMED, with why");
  ok(act.confirmed === 1 && act.nodes.size === 138, `act 1.0: ${act.confirmed} confirmed of ${act.nodes.size}`);
  const { doc } = await readNode("gm_0");
  ok(doc.node.state === "sealed" && doc.content["status.release"] === "act 1.0" && doc.content["status.sealed_as"] === "confirmed", JSON.stringify(doc.node));
  ok((await M.standing(doc)).state === "checked", "the stamp takes no signature off");
  await clean();
});

await test("a sealed node is not issued again by mistake, and an unchanged group is not sealed twice", async () => {
  await refused(ws(lead("act")).apply({ type: "issue", group: "act", id: "gm_0", author: "Ravi" }), "blocked", "re-issue it");
  await refused(rel(lead("act")).apply({ type: "seal", group: "act" }), "blocked", "nothing has changed since act 1.0");
  await refused(rel("Asha").apply({ type: "reissue", group: "act", id: "gm_0" }), "blocked", "only act's lead re-issues");
});

await test("gm_0 re-issued, changed, checked again; act seals 1.1, and 1.0 against 1.1 shows exactly that", async () => {
  await rel(lead("act")).apply({ type: "reissue", group: "act", id: "gm_0" });
  let { doc } = await readNode("gm_0");
  ok(doc.node.state === "issued", `state ${doc.node.state}`);
  await edit("Asha", "gm_0", "try it", (db) => M.setField(db, "explain.try_it", "Double the turns: what happens to the dipole?", "Asha"));
  let x = (await rel("x").assemble("act")).nodes.find((y) => y.id === "gm_0");
  ok(x.since === "changed" && !x.confirmed && x.standing.checkedStale, `after the edit: ${x.since}, ${x.why.join("; ")}`);
  await signNode("Asha", "gm_0", "author ready", "ready again");
  await signNode("Ravi", "gm_0", "checked by", "the new question too");
  x = (await rel("x").assemble("act")).nodes.find((y) => y.id === "gm_0");
  ok(!x.confirmed && x.why.some((w) => w.includes("since it last changed")), "stage mtq's signature no longer covers it");
  await rel("Meera").apply({ type: "signStage", group: "act", stage: "mtq" });
  const p = await rel(lead("act")).plan({ type: "seal", group: "act" });
  ok(p.action.version === "1.1" && !p.blocked, `seal ${p.action.version}: ${p.impact.map((i) => i.text).join("; ")}`);
  ok(p.impact.some((i) => i.text.includes("since 1.0: 0 new, 1 changed, 0 gone, 137 unchanged")), p.impact.map((i) => i.text).join("\n"));
  await rel(lead("act")).apply({ type: "seal", group: "act" });
  const [r10, r11] = await rel("x").releases("act");
  ok(r11.version === "1.1" && !r11.problems.length && r11.nodes.get("gm_0").sealed_as === "confirmed", "act 1.1 sealed, gm_0 confirmed");
  const diff = R.compare(r10.nodes, r11.nodes).filter((d) => d.change !== "same");
  ok(diff.length === 1 && diff[0].id === "gm_0" && diff[0].fields.includes("explain.try_it") && diff[0].fields.includes("signatures"), JSON.stringify(diff));
  await clean();
});

await test("a damaged node file stops the seal, and is made again from a release", async () => {
  const ndir = await S.subdir(root, S.NODES);
  const w = await (await ndir.getFileHandle("gm_1.node.tndb")).createWritable();
  await w.write(new TextEncoder().encode("not a database any more")); await w.close();
  await rel(lead("act")).apply({ type: "reissue", group: "act", id: "gm_0" });
  await edit("Asha", "gm_0", "a note", (db) => M.setField(db, "identity.note", "Per axis.", "Asha"));
  await refused(rel(lead("act")).apply({ type: "seal", group: "act" }), "blocked", "gm_1");
  await refused(rel(lead("act")).apply({ type: "reissue", group: "act", id: "gm_1" }), "blocked", "re-issue it from a release");
  await rel(lead("act")).apply({ type: "reissue", group: "act", id: "gm_1", from: "1.0" });
  const { doc, problems } = await readNode("gm_1");
  ok(!problems.length && doc.node.state === "issued" && doc.node.group_id === "act", `gm_1 again: ${problems.join("; ")}`);
  const [r10] = await rel("x").releases("act");
  const x = (await rel("x").assemble("act")).nodes.find((y) => y.id === "gm_1");
  ok(x.bodyFp === r10.nodes.get("gm_1").bodyFp, "gm_1 is what 1.0 sealed");
  await clean();
});

await test("the lead's comment reaches the node file", async () => {
  await rel(lead("act")).apply({ type: "comment", group: "act", id: "gm_0", body: "Say which coil the datasheet value is for." });
  const { doc } = await readNode("gm_0");
  ok(doc.comments.some((c) => c.by === lead("act") && c.body.includes("which coil")), JSON.stringify(doc.comments));
});

await test("a node form is imported: refused while the node is sealed, then its answers fill the node, the author's own kept", async () => {
  const form = R.parseForm(readFileSync(formPath, "utf8"));
  const r = rel(lead("env"));
  await refused(r.apply({ type: "import", group: "env", id: "m2_4", name: path.basename(formPath), form }), "blocked", "re-issue it first");
  await r.apply({ type: "reissue", group: "env", id: "m2_4" });
  await edit("Asha", "m2_4", "her own why", (db) => M.setField(db, "relation.why", "Asha's own words on why.", "Asha"));
  const a = { type: "import", group: "env", id: "m2_4", name: path.basename(formPath), form: R.parseForm(readFileSync(formPath, "utf8")) };
  const p = await r.plan(a);
  ok(!p.blocked, p.impact.map((i) => i.text).join("; "));
  ok(p.impact.some((i) => i.level === "warn" && i.text.includes("relation.why")), "the author's own field is kept, and the impact says so");
  await r.apply(a);
  const { doc } = await readNode("m2_4");
  ok(doc.content["explain.simply"] === "Add the Earth's radius to the height.", `explain.simply: ${doc.content["explain.simply"]}`);
  ok(doc.origin["explain.simply"].startsWith("form "), doc.origin["explain.simply"]);
  ok(doc.content["relation.why"] === "Asha's own words on why.", "Asha's words kept");
  ok(doc.fixtures.some((f) => f.name === "book" && f.outside), JSON.stringify(doc.fixtures));
  ok(doc.content["belief.area"] === "model", "the belief record taken in");
  await refused(r.apply({ ...a, id: "m2_5" }), "blocked", "the form is about m2_4, not m2_5");
  await clean();
});

await test("env seals 1.1 with the imported node; every release still checks", async () => {
  await rel(lead("env")).apply({ type: "seal", group: "env" });
  for (const g of groups) for (const r of await rel("x").releases(g)) ok(!r.problems.length, `${g} ${r.version}: ${r.problems.join("; ")}`);
  const rs = await rel("x").releases("env");
  ok(rs.map((r) => r.version).join() === "1.0,1.1", rs.map((r) => r.version).join());
  const d = R.compare(rs[0].nodes, rs[1].nodes).filter((x) => x.change !== "same");
  ok(d.length === 1 && d[0].id === "m2_4", JSON.stringify(d.map((x) => x.id)));
  await clean();
});

await test("every other group re-issues a node, changes it and seals 1.1", async () => {
  for (const g of groups.filter((x) => x !== "act" && x !== "env")) {
    const G = (await ws("x").refresh()).groups.get(g);
    const n = [...G.nodes.values()].find((x) => x.state === "sealed");
    await rel(lead(g)).apply({ type: "reissue", group: g, id: n.id });
    await ws(lead(g)).apply({ type: "rename", group: g, id: n.id, label: `${n.label} (revised)` });
    await rel(lead(g)).apply({ type: "seal", group: g });
    const rs = await rel("x").releases(g);
    ok(rs.map((r) => r.version).join() === "1.0,1.1" && rs.every((r) => !r.problems.length), `${g}: ${rs.map((r) => r.version)}`);
    const d = R.compare(rs[0].nodes, rs[1].nodes).filter((x) => x.change !== "same");
    ok(d.length === 1 && d[0].id === n.id && d[0].fields.includes("identity.label"), `${g}: ${JSON.stringify(d)}`);
  }
  await clean();
});

console.log(`release: ${passed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
