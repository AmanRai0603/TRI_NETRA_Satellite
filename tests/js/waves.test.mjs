// waves.test.mjs -- the group side of docs/RELEASE_PLAN.md P12 under Node, on a design folder on
// disk, with the group app's own code (design/js/release.js): stand-in leads seal their groups, and
// accept what the developer side delivered (tools/delivery.py deliver). tests/test_delivery.py runs
// it a step at a time, delivering between the steps.
//
//   node tests/js/waves.test.mjs DIR seal GROUP...       a stand-in lead for each group, who seals 1.0
//   node tests/js/waves.test.mjs DIR accept GROUP...     each group's lead accepts its delivery of 1.0
//   node tests/js/waves.test.mjs DIR refusals GROUP      who may not accept, and what may not be accepted
// Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
import { createRequire } from "node:module";
import { readFileSync } from "node:fs";
import { NodeFolder, MemoryJournal } from "./node_folder.mjs";

const require = createRequire(import.meta.url);
const ROOT = new URL("../../", import.meta.url).pathname;
const initSqlJs = require(ROOT + "design/vendor/sqljs/sql-wasm.js");
const SQL = await initSqlJs({ wasmBinary: readFileSync(ROOT + "design/vendor/sqljs/sql-wasm.wasm") });
const S = await import(ROOT + "design/js/structure.js");
const R = await import(ROOT + "design/js/release.js");

const [dir, step, ...groups] = process.argv.slice(2);
const root = new NodeFolder(dir);
const timers = { setInterval: () => 0, clearInterval() {} };
const ws = (who) => new S.Workspace({ SQL, root, who, session: who, profile: "test", journal: new MemoryJournal(), timers });
const rel = (who) => new R.Releases(ws(who));
const lead = (g) => `Stand-in lead of ${g}`;
let passed = 0, failed = 0;
async function test(name, fn) {
  try { await fn(); passed++; console.log(`PASS ${name}`); } catch (e) { failed++; console.log(`FAIL ${name}: ${e.stack || e}`); }
}
function ok(c, msg) { if (!c) throw new Error(msg); }
async function refused(p, code, words) {
  try { await p; } catch (e) { ok(e.code === code, `refused with ${e.code}, not ${code}: ${e.message}`); if (words) ok(e.message.includes(words), `the refusal says "${words}": ${e.message}`); return e; }
  throw new Error(`not refused (wanted ${code})`);
}

if (step === "seal") {
  for (const g of groups) {
    await test(`${g}: its stand-in lead joins and seals 1.0`, async () => {
      await ws(lead(g)).apply({ type: "member", group: g, name: lead(g), role: "lead" });
      await rel(lead(g)).apply({ type: "seal", group: g });
      const a = await rel(lead(g)).assemble(g);
      ok(a.releases.length === 1 && a.releases[0].version === "1.0" && !a.releases[0].problems.length, `${g}: ${JSON.stringify(a.releases.map((r) => r.problems))}`);
    });
  }
}

if (step === "accept") {
  for (const g of groups) {
    await test(`${g}: its lead sees the delivery of 1.0 and accepts it`, async () => {
      let a = await rel(lead(g)).assemble(g);
      const d = a.deliveries.find((x) => x.version === "1.0");
      ok(d && !d.accepted, `${g}: deliveries ${JSON.stringify(a.deliveries.map((x) => [x.version, !!x.accepted]))}`);
      const p = await rel(lead(g)).plan({ type: "accept", group: g, version: "1.0" });
      ok(!p.blocked, `${g}: blocked: ${p.impact.filter((x) => x.level === "block").map((x) => x.text).join("; ")}`);
      await rel(lead(g)).apply({ type: "accept", group: g, version: "1.0" });
      a = await rel(lead(g)).assemble(g);
      ok(a.deliveries.find((x) => x.version === "1.0").accepted.by === lead(g), `${g}: not recorded as accepted`);
      await refused(rel(lead(g)).apply({ type: "accept", group: g, version: "1.0" }), "blocked", "already accepted");
    });
  }
}

if (step === "refusals") {
  const g = groups[0];
  await test(`${g}: someone who is not its lead may not accept, nor anyone a version not sealed or not delivered`, async () => {
    await refused(rel("Somebody else").apply({ type: "accept", group: g, version: "1.0" }), "blocked", `only ${g}'s lead accepts`);
    await refused(rel(lead(g)).apply({ type: "accept", group: g, version: "9.9" }), "blocked", "has no release 9.9");
  });
}

console.log(`waves ${step}: ${passed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
