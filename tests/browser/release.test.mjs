// release.test.mjs -- TRI-NETRA Group's release side in a real browser (Chromium through Playwright):
// docs/RELEASE_PLAN.md P6 through the page. act (the largest group) shows its progress and is
// assembled; its stage owner signs a stage; someone who is not the lead is refused the seal; the lead
// seals act 1.0, re-issues a node, comments on it, renames it and seals 1.1, and compares the two;
// env imports a node form and seals 1.0; catalogue (the smallest) seals 1.0; in the node app a
// sealed node opens read-only, saying why. tests/js/release.test.mjs proves the same rules on all
// 20 groups under Node; this proves the page.
//
//   node tests/browser/release.test.mjs GROUP.html NODE.html DESIGN_DIR FORM.html OUT_DIR
// DESIGN_DIR: a seeded design folder, served and copied into the browser's private folder; FORM.html:
// a filled node form for m2_4; OUT_DIR gets the folder back (with releases/), for tools/tndb.py,
// tools/group.py and tools/release.py. tests/test_release.py runs this.
// Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
import { createServer } from "node:http";
import { readFileSync, writeFileSync, mkdirSync, readdirSync, statSync } from "node:fs";
import path from "node:path";

const [groupPage, nodePage, design, formPath, outDir] = process.argv.slice(2);
const pw = await import(process.env.PLAYWRIGHT_MODULE || "playwright").catch(() => import("/opt/node22/lib/node_modules/playwright/index.mjs"));
const browser = await pw.chromium.launch(process.env.CHROME ? { executablePath: process.env.CHROME } : {});
const files = [];
for (const sub of ["structure", "nodes"]) for (const f of readdirSync(path.join(design, sub))) if (statSync(path.join(design, sub, f)).isFile()) files.push(`${sub}/${f}`);
const server = createServer((req, res) => {
  const u = decodeURIComponent(req.url);
  if (u === "/list") { res.writeHead(200, { "content-type": "application/json" }); res.end(JSON.stringify(files)); return; }
  if (u.startsWith("/design/")) { res.writeHead(200); res.end(readFileSync(path.join(design, u.slice(8)))); return; }
  res.writeHead(200, { "content-type": "text/html; charset=utf-8" });
  res.end(readFileSync(u.startsWith("/node") ? nodePage : groupPage));
});
await new Promise((r) => server.listen(0, "127.0.0.1", r));
const BASE = `http://127.0.0.1:${server.address().port}`;

let passed = 0, failed = 0;
async function test(name, fn) {
  try {
    await Promise.race([fn(), new Promise((_, rej) => setTimeout(() => rej(new Error("did not finish in 300 s")), 300000))]);
    passed++; console.log(`PASS ${name}`);
  } catch (e) { failed++; console.log(`FAIL ${name}: ${e.stack || e}`); }
}
function ok(c, msg) { if (!c) throw new Error(msg); }

const ctx = await browser.newContext({ viewport: { width: 1280, height: 900 } });
ctx.setDefaultTimeout(30000);
await ctx.addInitScript(() => { try { localStorage.setItem("trinetra.who", "Lead of act"); } catch (e) { /* ignore */ } });
const outside = [];
async function openPage(url = "/group.html", ready = "tnGroup") {
  const page = await ctx.newPage();
  page.on("request", (r) => { if (!/^(data:|blob:)/.test(r.url()) && !r.url().startsWith(BASE)) outside.push(r.url()); });
  page.on("pageerror", (e) => console.log(`page error: ${e.message}`));
  await page.goto(`${BASE}${url}`);
  await page.evaluate((k) => window[k].ready, ready);
  return page;
}
async function dlg(page, values, press) {
  const d = page.locator("dialog[open]").last();
  await d.waitFor();
  for (const [id, v] of Object.entries(values)) {
    const el = d.locator(`[data-testid=${id}]`);
    if ((await el.evaluate((x) => x.tagName)) === "SELECT") await el.selectOption(v);
    else await el.fill(v);
  }
  await d.getByRole("button", { name: press, exact: true }).click();
}
// the impact dialog: check what it says, then do it (or, when it is blocked, cancel)
async function impact(page, words = [], { blocked = false } = {}) {
  const d = page.locator("dialog[open]").last();
  await d.waitFor({ timeout: 120000 });
  const text = await d.innerText();
  for (const w of words) ok(text.includes(w), `the impact check says "${w}": ${text.slice(0, 1500)}`);
  if (blocked) { ok(text.includes("This cannot be done yet"), `blocked: ${text.slice(0, 500)}`); await d.getByRole("button", { name: "Cancel" }).click(); return text; }
  const before = await page.evaluate(() => window.tnGroup.app.done || 0);
  await d.getByRole("button", { name: "Do it" }).click();
  await page.waitForFunction((n) => (window.tnGroup.app.done || 0) > n, before, { timeout: 240000 });
  return text;
}
const tab = async (page, name) => { await page.locator(`[data-tab="${name}"]`).click(); if (["Progress", "Assemble", "Release"].includes(name)) await page.locator("[data-testid=assembled]").waitFor({ timeout: 120000 }); };
const group = (page, id) => page.evaluate((g) => window.tnGroup.openGroup(g), id);
const as = (page, who) => page.evaluate((w) => { const a = window.tnGroup.app; a.who = w; a.ws.who = w; }, who);
const node = (page, id) => page.evaluate((n) => window.tnGroup.select(n), id);

let page;
await test("the design folder opens; act's progress lists all 138 nodes, none of them confirmed yet", async () => {
  page = await openPage();
  await page.evaluate(async (base) => {
    const root = await (await navigator.storage.getDirectory()).getDirectoryHandle("design", { create: true });
    for (const f of await (await fetch(`${base}/list`)).json()) {
      const [sub, name] = f.split("/");
      const w = await (await (await root.getDirectoryHandle(sub, { create: true })).getFileHandle(name, { create: true })).createWritable();
      await w.write(new Uint8Array(await (await fetch(`${base}/design/${f}`)).arrayBuffer()));
      await w.close();
    }
    window.__root = root;
    await window.tnGroup.useFolder(root);
  }, BASE);
  await group(page, "act");
  await tab(page, "Progress");
  ok((await page.locator("[data-testid=progress] tbody tr").count()) === 138, "138 rows");
  const main = await page.locator("main").innerText();
  ok(/Would be sealed as confirmed\s*0/.test(main) && main.includes("no release yet"), main.slice(0, 600));
});

await test("act assembled: every node with why it is not confirmed; a node shown as the main application shows it", async () => {
  await tab(page, "Assemble");
  ok((await page.locator("[data-testid=assemble] tbody tr").count()) === 138, "138 nodes");
  const row = page.locator("[data-testid=assemble] tbody tr").filter({ has: page.locator("td", { hasText: /^gm_0$/ }) });
  ok((await row.innerText()).includes("a shell: nothing written yet"), "gm_0's reason");
  await row.click();
  const v = await page.locator("[data-testid=node-view]").innerText();
  ok(v.includes("Would be sealed UNCONFIRMED") && /dipole/i.test(v), v.slice(0, 400));
  await page.locator("dialog[open]").last().getByRole("button", { name: "Close" }).click();
});

await test("people and a stage: the lead, and Meera owning stage mtq; only Meera can sign it", async () => {
  await tab(page, "People");
  for (const [n, r] of [["Lead of act", "lead"], ["Meera", "stage owner"]]) {
    await page.locator("[data-testid=add-member]").click();
    await dlg(page, { "f-name": n, "f-role": r }, "Next");
    await impact(page);
  }
  await tab(page, "Stages");
  await page.locator("[data-testid=stages] tbody tr", { hasText: "mtq" }).first().click();
  await dlg(page, { "f-owner": "Meera" }, "Next");
  await impact(page);
  await tab(page, "Assemble");
  await page.locator("[data-testid=sign-mtq]").click();
  await impact(page, ["signed by its owner, Meera"], { blocked: true });
  await as(page, "Meera");
  await page.locator("[data-testid=sign-mtq]").click();
  await impact(page, ["signs the", "not checked by a second engineer yet"]);
  await tab(page, "Assemble");
  ok((await page.locator("[data-testid=stages-sign] tbody tr", { hasText: "mtq" }).innerText()).includes("signed by Meera"), "mtq signed");
});

await test("only the lead seals; the lead seals act 1.0, and every node goes in with its standing", async () => {
  await tab(page, "Release");
  await page.locator("[data-testid=seal]").click();
  await impact(page, ["only act's lead seals it: Meera is not its lead"], { blocked: true });
  await as(page, "Lead of act");
  await tab(page, "Release");
  await page.locator("[data-testid=seal]").click();
  await impact(page, ["releases/act-1.0.tnrel: a new release file", "138 node file(s) stamped", "0 of 138 node(s) sealed as confirmed", "sealed UNCONFIRMED"]);
  await tab(page, "Release");
  const r = await page.locator("[data-testid=releases] tbody tr").innerText();
  ok(r.includes("1.0") && r.includes("intact") && r.includes("0 of 138"), r);
  ok((await page.locator("[data-testid=seal]").innerText()).includes("Seal 1.1"), "the next is 1.1");
  await page.locator("[data-testid=seal]").click();
  await impact(page, ["nothing has changed since act 1.0"], { blocked: true });
});

await test("gm_0 re-issued, commented on and renamed; act seals 1.1; 1.0 against 1.1 names gm_0 alone", async () => {
  await tab(page, "Map");
  await node(page, "gm_0");
  await page.locator("[data-testid=act-reissue]").click();
  await dlg(page, {}, "Next");
  await impact(page, ["open again"]);
  await node(page, "gm_0");
  ok((await page.locator("main").innerText()).includes("issued"), "gm_0 issued again");
  await page.locator("[data-testid=act-comment]").click();
  await dlg(page, { "f-comment": "Say which coil the datasheet value is for." }, "Next");
  await impact(page, ["a comment its author sees"]);
  await node(page, "gm_0");
  await page.locator("[data-testid=act-rename]").click();
  await dlg(page, { "f-label": "Dipole available per axis (per coil)" }, "Next");
  await impact(page);
  await tab(page, "Release");
  await page.locator("[data-testid=seal]").click();
  await impact(page, ["since 1.0: 0 new, 1 changed, 0 gone, 137 unchanged"]);
  await tab(page, "Release");
  ok((await page.locator("[data-testid=releases] tbody tr").count()) === 2, "1.0 and 1.1");
  await page.locator("[data-testid=compare]").click();
  await dlg(page, { "f-from": "1.0", "f-to": "1.1" }, "Compare");
  const c = await page.locator("[data-testid=comparison]").innerText();
  ok(c.includes("gm_0") && c.includes("changed") && c.includes("identity.label") && (c.match(/\n/g) || []).length < 4, c);
  await page.locator("dialog[open]").last().getByRole("button", { name: "Close" }).click();
});

await test("env imports a node form into m2_4 and seals 1.0; catalogue, the smallest, seals 1.0", async () => {
  await group(page, "env");
  await tab(page, "People");
  await as(page, "Lead of env");
  await page.locator("[data-testid=add-member]").click();
  await dlg(page, { "f-name": "Lead of env", "f-role": "lead" }, "Next");
  await impact(page);
  await tab(page, "Release");
  const pending = impact(page, ["field(s) filled from the form", "test vector(s) from the form"]);
  await page.locator("[data-testid=import]").setInputFiles(formPath);
  await pending;
  ok(await page.evaluate(async () => {
    const d = await window.__root.getDirectoryHandle("nodes");
    const t = new TextDecoder().decode(new Uint8Array(await (await (await d.getFileHandle("m2_4.node.tndb")).getFile()).arrayBuffer()));
    return t.includes("Add the Earth's radius to the height.");
  }), "the form's answer is in m2_4's file");
  await tab(page, "Release");
  await page.locator("[data-testid=seal]").click();
  await impact(page, ["releases/env-1.0.tnrel"]);
  await group(page, "catalogue");
  await tab(page, "People");
  await as(page, "Lead of catalogue");
  await page.locator("[data-testid=add-member]").click();
  await dlg(page, { "f-name": "Lead of catalogue", "f-role": "lead" }, "Next");
  await impact(page);
  await tab(page, "Release");
  await page.locator("[data-testid=seal]").click();
  await impact(page, ["releases/catalogue-1.0.tnrel"]);
});

await test("in the node app a sealed node opens read-only, saying it is sealed and who re-opens it", async () => {
  const np = await openPage("/node.html", "tnNode");
  await np.evaluate(async () => { await window.tnNode.useFolder(await (await navigator.storage.getDirectory()).getDirectoryHandle("design")); });
  const id = await page.evaluate(() => [...window.tnGroup.app.ws.index.groups.get("catalogue").nodes.keys()][0]);
  await np.evaluate((n) => window.tnNode.openNode(n), id);
  await np.locator(".tn-banner", { hasText: "Read-only" }).waitFor();
  const t = await np.locator(".tn-banner", { hasText: "Read-only" }).innerText();
  ok(t.includes("sealed in catalogue 1.0") && t.includes("re-issues"), t);
  await np.evaluate(() => window.tnNode.closeNode());
  await np.close();
});

await test("the folder as the page left it, for the Python checkers", async () => {
  const out = await page.evaluate(async () => {
    const r = {};
    const b64 = (b) => { let s = ""; for (let i = 0; i < b.length; i += 0x8000) s += String.fromCharCode(...b.subarray(i, i + 0x8000)); return btoa(s); };
    for (const sub of ["structure", "nodes", "releases"]) {
      const d = await window.__root.getDirectoryHandle(sub);
      for await (const [n, hd] of d.entries()) if (hd.kind === "file" && !n.endsWith(".editing")) r[`${sub}/${n}`] = b64(new Uint8Array(await (await hd.getFile()).arrayBuffer()));
    }
    return r;
  });
  for (const [p, b] of Object.entries(out)) { mkdirSync(path.dirname(path.join(outDir, p)), { recursive: true }); writeFileSync(path.join(outDir, p), Buffer.from(b, "base64")); }
  ok(Object.keys(out).filter((p) => p.startsWith("releases/")).length === 4, Object.keys(out).filter((p) => p.startsWith("releases/")).join());
  ok(outside.length === 0, `nothing fetched from outside: ${outside.slice(0, 3)}`);
});

server.close();
await browser.close();
console.log(`release: ${passed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
