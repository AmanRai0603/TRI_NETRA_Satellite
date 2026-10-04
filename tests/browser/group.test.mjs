// group.test.mjs -- TRI-NETRA Group in a real browser (Chromium through Playwright): the proof of
// docs/RELEASE_PLAN.md P4 through the page. All 20 groups open; act (the largest) and catalogue
// (the smallest) are restructured through the forms and the impact check; what another group
// reads is not taken from it until its lead accepts a change request; a node moves between two
// groups; a node file someone else has open stops an action; and at the end every rule holds.
//
//   node tests/browser/group.test.mjs PAGE.html DESIGN_DIR OUT_DIR
// DESIGN_DIR: a seeded design folder (tools/seed_design.py), served to the page, which copies it
// into the browser's private folder (the folder interface a picked Drive folder has); OUT_DIR gets
// the folder as the page left it, for tools/group.py check. tests/test_pages.py runs this.
// Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
import { createServer } from "node:http";
import { readFileSync, writeFileSync, mkdirSync, readdirSync, statSync } from "node:fs";
import path from "node:path";

const [pagePath, design, outDir] = process.argv.slice(2);
const pw = await import(process.env.PLAYWRIGHT_MODULE || "playwright").catch(() => import("/opt/node22/lib/node_modules/playwright/index.mjs"));
const browser = await pw.chromium.launch(process.env.CHROME ? { executablePath: process.env.CHROME } : {});
const html = readFileSync(pagePath);

const files = [];
for (const sub of ["structure", "nodes"]) for (const f of readdirSync(path.join(design, sub))) if (statSync(path.join(design, sub, f)).isFile()) files.push(`${sub}/${f}`);
const server = createServer((req, res) => {
  const u = decodeURIComponent(req.url);
  if (u === "/list") { res.writeHead(200, { "content-type": "application/json" }); res.end(JSON.stringify(files)); return; }
  if (u.startsWith("/design/")) { res.writeHead(200); res.end(readFileSync(path.join(design, u.slice(8)))); return; }
  res.writeHead(200, { "content-type": "text/html; charset=utf-8" }); res.end(html);
});
await new Promise((r) => server.listen(0, "127.0.0.1", r));
const BASE = `http://127.0.0.1:${server.address().port}`;

let passed = 0, failed = 0;
async function test(name, fn) {
  try {
    await Promise.race([fn(), new Promise((_, rej) => setTimeout(() => rej(new Error("did not finish in 180 s")), 180000))]);
    passed++; console.log(`PASS ${name}`);
  } catch (e) { failed++; console.log(`FAIL ${name}: ${e.stack || e}`); }
}
function ok(c, msg) { if (!c) throw new Error(msg); }

const ctx = await browser.newContext({ viewport: { width: 1280, height: 900 } });
ctx.setDefaultTimeout(20000);
await ctx.addInitScript(() => { try { localStorage.setItem("trinetra.who", "Lead"); } catch (e) { /* ignore */ } });
const outside = [];
async function openPage(c = ctx) {
  const page = await c.newPage();
  page.on("request", (r) => { if (!/^(data:|blob:)/.test(r.url()) && !r.url().startsWith(BASE)) outside.push(r.url()); });
  await page.goto(`${BASE}/group.html`);
  await page.evaluate(() => window.tnGroup.ready);
  return page;
}
const useDesign = (page) => page.evaluate(async () => {
  const root = await (await navigator.storage.getDirectory()).getDirectoryHandle("design", { create: true });
  window.__root = root;
  await window.tnGroup.useFolder(root);
});
// a dialog: fill fields by test id, then press a button by its name
async function dlg(page, values, press) {
  const d = page.locator("dialog[open]").last();
  await d.waitFor();
  for (const [id, v] of Object.entries(values)) {
    const el = d.locator(`[data-testid=${id}]`);
    if ((await el.evaluate((x) => x.tagName)) === "SELECT") await el.selectOption(v);
    else if ((await el.evaluate((x) => x.tagName)) === "FIELDSET") { for (const one of v) await el.locator(`input[value="${one}"]`).check(); }
    else await el.fill(v);
  }
  await d.getByRole("button", { name: press, exact: true }).click();
}
async function impactThenDo(page, words = null) {
  const d = page.locator("dialog[open]").last();
  await d.waitFor();
  if (words) ok((await d.innerText()).includes(words), `the impact check says "${words}": ${await d.innerText()}`);
  const before = await page.evaluate(() => window.tnGroup.app.done || 0);
  await d.getByRole("button", { name: "Do it" }).click();
  await page.waitForFunction((n) => (window.tnGroup.app.done || 0) > n, before, { timeout: 60000 });
}
const tab = (page, name) => page.locator(`[data-tab="${name}"]`).click();
const group = (page, id) => page.evaluate((g) => window.tnGroup.openGroup(g), id);
const idx = (page, fn, arg) => page.evaluate(([f, a]) => new Function("app", "a", `return (${f})(app, a)`)(window.tnGroup.app, a), [fn.toString(), arg]);

let page;
await test("the design folder opens: all 20 groups, and every rule holds", async () => {
  page = await openPage();
  const n = await page.evaluate(async (base) => {
    const root = await (await navigator.storage.getDirectory()).getDirectoryHandle("design", { create: true });
    const list = await (await fetch(`${base}/list`)).json();
    for (const f of list) {
      const [sub, name] = f.split("/");
      const d = await root.getDirectoryHandle(sub, { create: true });
      const w = await (await d.getFileHandle(name, { create: true })).createWritable();
      await w.write(new Uint8Array(await (await fetch(`${base}/design/${f}`)).arrayBuffer()));
      await w.close();
    }
    return list.length;
  }, BASE);
  ok(n === 754, `${n} files copied`);
  await useDesign(page);
  const rows = await page.locator("[data-testid=groups] tbody tr").count();
  ok(rows === 20, `${rows} groups listed`);
  await page.locator("[data-testid=check-all]").click();
  await page.locator(".tn-banner.ok", { hasText: "Every rule holds" }).waitFor({ timeout: 120000 });
  for (const g of await idx(page, (app) => [...app.ws.index.groups.keys()])) {
    await group(page, g);
    ok((await page.locator(".tn-banner.ok").first().innerText()).includes(`Every rule holds in ${g}`), `${g} opens and holds`);
  }
});

await test("act: the map, a member, a new node added, renamed and moved between stages", async () => {
  await page.locator("[data-testid=groups-back]").click();
  await page.locator("[data-testid=groups] tbody tr", { hasText: "act" }).first().click();
  await page.locator("[data-testid=map] .tn-graph-node").first().waitFor();
  const boxes = await page.locator("[data-testid=map] .tn-graph-node").count();
  ok(boxes >= 138, `${boxes} boxes on the map (138 nodes and the inputs from other groups)`);
  await tab(page, "People");
  await page.locator("[data-testid=add-member]").click();
  await dlg(page, { "f-name": "Asha", "f-role": "author" }, "Next");
  await impactThenDo(page);
  await tab(page, "Map");
  await page.locator("[data-testid=add-node]").first().click();
  const stages = await idx(page, (app) => app.ws.index.groups.get("act").stages.map((s) => s.id));
  await dlg(page, { "f-id": "act_friction_model", "f-label": "Wheel friction", "f-stage": stages[0] }, "Next");
  await impactThenDo(page, "a new node file nodes/act_friction_model.node.tndb");
  await page.locator('[data-node="act_friction_model"]').click();
  await page.locator("[data-testid=act-rename]").click();
  await dlg(page, { "f-label": "Wheel friction and stiction" }, "Next");
  await impactThenDo(page);
  await page.locator('[data-node="act_friction_model"]').click();
  await page.locator("[data-testid=act-stage]").click();
  await dlg(page, { "f-stage": stages[1] }, "Next");
  await impactThenDo(page);
  await page.locator('[data-node="act_friction_model"]').click();
  await page.locator("[data-testid=act-issue]").click();
  await dlg(page, { "f-author": "Asha" }, "Next");
  await impactThenDo(page, "names Asha as its author");
  const n = await idx(page, (app) => { const x = app.ws.index.groups.get("act"); return [x.nodes.get("act_friction_model"), x.memberNodes.get("act_friction_model")]; });
  ok(n[0].label === "Wheel friction and stiction" && n[0].stage === stages[1] && n[1].author === "Asha", JSON.stringify(n));
});

await test("act: split a node through the form, then merge the new part back", async () => {
  const src = await idx(page, (app) => { const S = app.ws.index; const g = S.groups.get("act");
    return [...g.nodes.keys()].find((id) => g.edges.some((e) => e.from_node === id && g.nodes.has(e.to_node)) && !S.groups.get("act").edges.some(() => false)
      && ![...S.groups.values()].some((o) => o.id !== "act" && o.edges.some((e) => e.from_node === id))); });
  const reader = await idx(page, (app, id) => app.ws.index.groups.get("act").edges.find((e) => e.from_node === id).to_node, src);
  await page.locator(`[data-node="${src}"]`).first().click();
  await page.locator("[data-testid=act-split]").click();
  await dlg(page, { "f-id": "act_split_out", "f-label": "Split out", "f-readers": [reader] }, "Next");
  await impactThenDo(page, "act_split_out gets");
  ok(await idx(page, (app, r) => app.ws.index.groups.get("act").edges.some((e) => e.from_node === "act_split_out" && e.to_node === r), reader), "the reader follows the new node");
  await page.locator('[data-node="act_split_out"]').click();
  await page.locator("[data-testid=act-merge]").click();
  await dlg(page, { "f-keep": src }, "Next");
  await impactThenDo(page, "go into");
  ok(await idx(page, (app) => app.ws.index.groups.get("act").nodes.get("act_split_out").state === "archived"), "the part is archived");
});

await test("act: a node another group reads is archived only after that group accepts", async () => {
  const [id, rg] = await idx(page, (app) => { const S = app.ws.index;
    for (const id of S.groups.get("act").nodes.keys()) { const r = [...S.groups.values()].filter((o) => o.id !== "act" && o.edges.some((e) => e.from_node === id)); if (r.length === 1) return [id, r[0].id]; }
    return [null, null]; });
  ok(id, "a node read by one other group");
  await page.locator(`[data-node="${id}"]`).first().click();
  await page.locator("[data-testid=act-archive]").click();
  const d = page.locator("dialog[open]").last();
  ok((await d.innerText()).includes(`${rg} reads`), "the impact check names the reader");
  ok(!(await d.getByRole("button", { name: "Do it" }).count()), "and it cannot be done");
  await d.getByRole("button", { name: "Raise a change request…" }).click();
  await dlg(page, { "f-body": "Superseded by the friction model." }, "Raise it");
  await impactThenDo(page);
  await group(page, rg);
  await tab(page, "Requests");
  await page.locator("[data-testid=incoming] tr", { hasText: id }).getByRole("button", { name: "Accept" }).click();
  await impactThenDo(page);
  await group(page, "act");
  await page.locator(`[data-node="${id}"]`).first().click();
  await page.locator("[data-testid=act-archive]").click();
  await impactThenDo(page, "accepted change request");
  ok(!(await idx(page, (app, a) => app.ws.index.groups.get(a[1]).edges.some((e) => e.from_node === a[0]), [id, rg])), `${rg} no longer reads it`);
});

await test("a node moves from act to ctl once ctl accepts it", async () => {
  await group(page, "act");
  await page.locator('[data-node="act_friction_model"]').click();
  await page.locator("[data-testid=act-move]").click();
  await dlg(page, { "f-to": "ctl" }, "Next");
  const d = page.locator("dialog[open]").last();
  ok((await d.innerText()).includes("needs its lead's agreement"), "blocked until ctl agrees");
  await d.getByRole("button", { name: "Raise a change request…" }).click();
  await dlg(page, { "f-body": "Friction compensation belongs with the controller." }, "Raise it");
  await impactThenDo(page);
  await group(page, "ctl");
  await tab(page, "Requests");
  await page.locator("[data-testid=incoming] tr", { hasText: "act_friction_model" }).getByRole("button", { name: "Accept" }).click();
  await impactThenDo(page);
  await group(page, "act");
  await page.locator('[data-node="act_friction_model"]').click();
  await page.locator("[data-testid=act-move]").click();
  await dlg(page, { "f-to": "ctl" }, "Next");
  await impactThenDo(page, "stays where it is, saying group ctl");
  await group(page, "ctl");
  ok(await page.locator('[data-node="act_friction_model"]').count() === 1, "ctl's map shows it");
});

await test("catalogue, the smallest group: add, split, merge, archive", async () => {
  await group(page, "catalogue");
  ok(await idx(page, (app) => app.ws.index.groups.get("catalogue").nodes.size) === 8, "8 nodes");
  await page.locator("[data-testid=add-node]").first().click();
  await dlg(page, { "f-id": "cat_datasheets", "f-label": "Datasheet intake" }, "Next");
  await impactThenDo(page);
  const src = await idx(page, (app) => { const S = app.ws.index; return [...S.groups.get("catalogue").nodes.keys()].find((id) => id !== "cat_datasheets" && ![...S.groups.values()].some((o) => o.id !== "catalogue" && o.edges.some((e) => e.from_node === id))); });
  await page.locator(`[data-node="${src}"]`).click();
  await page.locator("[data-testid=act-split]").click();
  await dlg(page, { "f-id": "cat_split", "f-label": "Catalogue split" }, "Next");
  await impactThenDo(page);
  await page.locator('[data-node="cat_split"]').click();
  await page.locator("[data-testid=act-merge]").click();
  await dlg(page, { "f-keep": "cat_datasheets" }, "Next");
  await impactThenDo(page);
  await page.locator('[data-node="cat_datasheets"]').click();
  await page.locator("[data-testid=act-archive]").click();
  await impactThenDo(page);
  ok(await idx(page, (app) => [...app.ws.index.groups.get("catalogue").nodes.values()].filter((n) => n.state === "archived").length) === 2, "two archived");
});

await test("a node file someone else has open stops the action, by name", async () => {
  const id = await idx(page, (app) => [...app.ws.index.groups.get("catalogue").nodes.values()].find((n) => n.state !== "archived").id);
  const ctx2 = await browser.newContext();
  const other = await ctx2.newPage();
  await other.goto(`${BASE}/group.html`);
  // another computer: its marker beside the node file, fresh (Drive brought it in)
  await page.evaluate(async (n) => {
    const d = await window.__root.getDirectoryHandle("nodes");
    const w = await (await d.getFileHandle(`${n}.node.tndb.editing`, { create: true })).createWritable();
    await w.write(JSON.stringify({ who: "Asha", session: "her-laptop", profile: "hers", since: new Date().toISOString(), beat: new Date().toISOString() }));
    await w.close();
  }, id);
  await page.evaluate((n) => window.tnGroup.select(n), id);
  ok((await page.locator(".tn-section h2", { hasText: `Node ${id}` }).count()) === 1, `the detail shows ${id}`);
  await page.locator("[data-testid=act-rename]").click();
  await dlg(page, { "f-label": "Renamed" }, "Next");
  await page.locator("dialog[open]").last().getByRole("button", { name: "Do it" }).click();
  await page.locator(".tn-banner.error", { hasText: "Asha" }).first().waitFor({ timeout: 60000 }).catch(async (e) => { throw new Error(`${e.message}\n${await page.locator("dialog[open]").count()} dialog(s) open: ${await page.locator("dialog[open]").allInnerTexts()}\ntoasts: ${await page.locator(".tn-toast").allInnerTexts()}\nsel ${await page.evaluate(() => window.tnGroup.app.sel)}\nstatus ${await page.locator("[data-testid=status]").innerText()}`); });
  ok(await idx(page, (app, n) => app.ws.index.groups.get("catalogue").nodes.get(n).label !== "Renamed", id), "nothing changed");
  await page.evaluate(async (n) => (await window.__root.getDirectoryHandle("nodes")).removeEntry(`${n}.node.tndb.editing`), id);
  await ctx2.close();
});

await test("at the end every rule holds: no node file broken; history and phone width", async () => {
  await page.evaluate(() => window.tnGroup.showGroups());
  await page.locator("[data-testid=check-all]").click();
  await page.locator(".tn-banner.ok", { hasText: "Every rule holds" }).waitFor({ timeout: 120000 });
  await group(page, "act");
  await tab(page, "History");
  await page.locator("[data-testid=history] tbody tr").first().waitFor();
  ok((await page.locator("[data-testid=history] tbody tr").count()) >= 8, "act's history lists its actions");
  await page.setViewportSize({ width: 375, height: 800 });
  await tab(page, "Map");
  ok((await page.evaluate(() => document.documentElement.scrollWidth)) <= 375, "the page fits; the map scrolls inside itself");
  // the folder as the page left it, for tools/group.py
  const out = await page.evaluate(async () => {
    const r = {};
    for (const sub of ["structure", "nodes"]) {
      const d = await window.__root.getDirectoryHandle(sub);
      for await (const [n, hd] of d.entries()) {
        if (hd.kind !== "file") continue;
        const b = new Uint8Array(await (await hd.getFile()).arrayBuffer());
        let s = ""; for (let i = 0; i < b.length; i += 0x8000) s += String.fromCharCode(...b.subarray(i, i + 0x8000));
        r[`${sub}/${n}`] = btoa(s);
      }
      if (sub === "structure") {
        const a = await d.getDirectoryHandle("actions");
        for await (const [n, hd] of a.entries()) r[`structure/actions/${n}`] = btoa(unescape(encodeURIComponent(await (await hd.getFile()).text())));
      }
    }
    return r;
  });
  for (const [p, b64] of Object.entries(out)) { mkdirSync(path.dirname(path.join(outDir, p)), { recursive: true }); writeFileSync(path.join(outDir, p), Buffer.from(b64, "base64")); }
  ok(outside.length === 0, `nothing fetched from outside: ${outside.slice(0, 3)}`);
});

server.close();
await browser.close();
console.log(`group: ${passed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
