// help.test.mjs -- the manual, the tours and printing in a real browser (docs/RELEASE_PLAN.md P7),
// as a newcomer meets them: someone opening an app for the first time is shown its tour, screen by
// screen, once; Help opens every guide in place, prints a page, and shows the tour again; every
// field of every group-app form has help beside it; a node prints alone; the manual fits a phone.
// Then a walkthrough as a newcomer would do it, finding everything by what the screen says (labels,
// tab names, button names; never the pages' test ids): an author fills a node and marks it ready, a
// colleague checks it, the stage owner signs the stage and the lead seals the group with that node
// confirmed. It does not replace the usability session with real members (docs/USABILITY_SESSION.md);
// it proves the screens can be followed by what they say.
//
//   node tests/browser/help.test.mjs FILES.html NODE.html GROUP.html DESIGN_DIR
// tests/test_pages.py runs this.
// Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
import { createServer } from "node:http";
import { readFileSync, readdirSync, statSync } from "node:fs";
import path from "node:path";

const [filesPage, nodePage, groupPage, design] = process.argv.slice(2);
const pw = await import(process.env.PLAYWRIGHT_MODULE || "playwright").catch(() => import("/opt/node22/lib/node_modules/playwright/index.mjs"));
const browser = await pw.chromium.launch(process.env.CHROME ? { executablePath: process.env.CHROME } : {});
const files = [];
for (const sub of ["structure", "nodes"]) for (const f of readdirSync(path.join(design, sub))) if (statSync(path.join(design, sub, f)).isFile()) files.push(`${sub}/${f}`);
const pagesBy = { "/files.html": filesPage, "/node.html": nodePage, "/group.html": groupPage };
const server = createServer((req, res) => {
  const u = decodeURIComponent(req.url);
  if (u === "/list") { res.writeHead(200, { "content-type": "application/json" }); res.end(JSON.stringify(files)); return; }
  if (u.startsWith("/design/")) { res.writeHead(200); res.end(readFileSync(path.join(design, u.slice(8)))); return; }
  res.writeHead(200, { "content-type": "text/html; charset=utf-8" });
  res.end(readFileSync(pagesBy[u] || groupPage));
});
await new Promise((r) => server.listen(0, "127.0.0.1", r));
const BASE = `http://127.0.0.1:${server.address().port}`;

let passed = 0, failed = 0;
async function test(name, fn) {
  try { await Promise.race([fn(), new Promise((_, rej) => setTimeout(() => rej(new Error("did not finish in 300 s")), 300000))]); passed++; console.log(`PASS ${name}`); }
  catch (e) { failed++; console.log(`FAIL ${name}: ${e.stack || e}`); }
}
function ok(c, msg) { if (!c) throw new Error(msg); }

// a newcomer: nothing remembered in this browser (no name, no tour seen)
const ctx = await browser.newContext({ viewport: { width: 1280, height: 900 } });
ctx.setDefaultTimeout(30000);
const outside = [];
async function open(url, ready) {
  const page = await ctx.newPage();
  page.on("request", (r) => { if (!/^(data:|blob:)/.test(r.url()) && !r.url().startsWith(BASE)) outside.push(r.url()); });
  page.on("pageerror", (e) => console.log(`page error: ${e.message}`));
  await page.addInitScript(() => { window.print = () => { const p = document.getElementById("tn-print"); window.__printed = (window.__printed || []).concat(p ? p.innerText : "(nothing marked to print)"); }; });
  await page.goto(`${BASE}${url}`);
  await page.evaluate((k) => window[k].ready, ready);
  return page;
}
const tourCard = (page) => page.locator("[data-testid=tour]");
async function nameDialog(page, name) {
  const d = page.locator("dialog[open]").last();
  await d.waitFor();
  await d.getByLabel("Name").fill(name);
  await d.getByRole("button", { name: "OK" }).click();
}

let node;
await test("a newcomer opening the node app is shown its tour, pointing at each thing in turn, once", async () => {
  node = await open("/node.html", "tnNode");
  await tourCard(node).waitFor();
  ok((await tourCard(node).innerText()).includes("Open the design folder"), await tourCard(node).innerText());
  ok(await node.locator('[data-testid="open-folder"].tn-tour-target').count() === 1, "the button it speaks of is marked");
  await tourCard(node).getByRole("button", { name: "Next" }).click();
  ok((await tourCard(node).innerText()).includes("Help is always here"), "step 2");
  ok(await node.getByRole("button", { name: "Help" }).evaluate((b) => b.classList.contains("tn-tour-target")), "Help is marked");
  await tourCard(node).getByRole("button", { name: "Done" }).click();
  ok(await tourCard(node).count() === 0, "the tour ends");
  await node.reload();
  await node.evaluate(() => window.tnNode.ready);
  await node.waitForTimeout(300);
  ok(await tourCard(node).count() === 0, "and is not shown again");
});

await test("Help: every guide in place, the journey drawn, a page printed alone, the tour again", async () => {
  await node.getByRole("button", { name: "Help" }).click();
  const d = node.locator("dialog[open]").last();
  await d.waitFor();
  const navs = await d.locator(".tn-manual-nav button").allInnerTexts();
  for (const t of ["The journey of a node", "Guide for authors", "Guide for group leads", "Guide for developers", "Guide for users", "Glossary", "Guide to TRI-NETRA Files"]) ok(navs.includes(t), `${t} listed: ${navs}`);
  ok((await d.locator("[data-testid=manual-page]").innerText()).startsWith("Guide for authors"), "the author's guide opens first in the node app");
  await d.getByRole("button", { name: "The journey of a node" }).click();
  ok(await d.locator("[data-testid=manual-page] img").evaluate((i) => i.complete && i.naturalWidth > 500), "the journey diagram is drawn");
  await d.getByRole("button", { name: "Glossary" }).click();
  ok((await d.locator("[data-testid=manual-page]").innerText()).includes("Outside answer"), "the glossary");
  await d.getByRole("button", { name: "Print this page" }).click();
  const printed = await node.evaluate(() => window.__printed);
  ok(printed && printed[0].startsWith("Glossary") && !printed[0].includes("Open design folder"), `only the page is printed: ${printed && printed[0].slice(0, 80)}`);
  await node.getByRole("button", { name: "Help" }).click();
  await node.locator("dialog[open]").last().getByRole("button", { name: "Take the tour of this screen" }).click();
  await tourCard(node).waitFor();
  ok((await tourCard(node).innerText()).includes("1 of 2"), "the tour of this screen, again");
  await node.keyboard.press("Escape");
  ok(await tourCard(node).count() === 0, "Escape ends it");
});

await test("the manual fits a phone", async () => {
  await node.setViewportSize({ width: 375, height: 800 });
  await node.getByRole("button", { name: "Help" }).click();
  const d = node.locator("dialog[open]").last();
  await d.waitFor();
  ok(await d.evaluate((x) => x.scrollWidth <= x.clientWidth + 1), "no sideways scroll in the manual");
  ok((await node.evaluate(() => document.documentElement.scrollWidth)) <= 375, "nor on the page");
  await d.getByRole("button", { name: "Close" }).click();
  await node.setViewportSize({ width: 1280, height: 900 });
});

let group;
await test("the group app: its tour on each screen; every field of every form has help beside it", async () => {
  group = await open("/group.html", "tnGroup");
  ok(await tourCard(group).count() === 0 || (await tourCard(group).innerText()).includes("Open the design folder"), "the start tour");
  if (await tourCard(group).count()) await tourCard(group).getByRole("button", { name: "Close the tour" }).click();
  const named = group.evaluate(async (base) => {
    const root = await (await navigator.storage.getDirectory()).getDirectoryHandle("design", { create: true });
    for (const f of await (await fetch(`${base}/list`)).json()) {
      const [sub, name] = f.split("/");
      const w = await (await (await root.getDirectoryHandle(sub, { create: true })).getFileHandle(name, { create: true })).createWritable();
      await w.write(new Uint8Array(await (await fetch(`${base}/design/${f}`)).arrayBuffer()));
      await w.close();
    }
    await window.tnGroup.useFolder(root);
  }, BASE);
  await nameDialog(group, "Lead of act");
  await named;
  await tourCard(group).waitFor();
  ok((await tourCard(group).innerText()).includes("The groups"), "the list's tour");
  await tourCard(group).getByRole("button", { name: "Close the tour" }).click();
  await group.getByRole("row", { name: /^act\b/ }).click();
  await tourCard(group).waitFor();
  ok((await tourCard(group).innerText()).includes("Your group"), "the group's tour");
  for (let i = 0; i < 5 && await tourCard(group).count(); i++) await tourCard(group).getByRole("button", { name: /Next|Done/ }).click();
  const forms = [["People", "Add or change a member…"], ["Map", "Add a node…"], ["Stages", "Add a stage…"], ["Requests", "Raise a change request…"]];
  for (const [tabName, btn] of forms) {
    await group.getByRole("tab", { name: tabName }).click();
    await group.getByRole("button", { name: btn }).first().click();
    const d = group.locator("dialog[open]").last();
    await d.waitFor();
    const bare = await d.locator(".tn-field").evaluateAll((fs) => fs.filter((f) => !f.querySelector(".tn-help")).map((f) => f.innerText.split("\n")[0]));
    ok(!bare.length, `${btn}: fields without help: ${bare}`);
    await d.getByRole("button", { name: "Cancel" }).click();
  }
});

// ---- the walkthrough, by what the screen says (waiting on the page's own count of finished
// actions, so a step starts only when the last one is done)
async function doIt(page, timeout = 240000) {
  const d = page.locator("dialog[open]").last();
  const before = await page.evaluate(() => window.tnGroup.app.done || 0);
  await d.getByRole("button", { name: "Do it" }).click();
  await page.waitForFunction((n) => (window.tnGroup.app.done || 0) > n, before, { timeout });
}
async function fieldByLabel(page, label, value) {
  const f = page.getByLabel(label, { exact: true });
  if ((await f.evaluate((x) => x.tagName)) === "SELECT") await f.selectOption({ label: value });
  else { await f.fill(value); await f.press("Tab"); }
  await page.waitForFunction(() => !window.tnNode.app.busy);
}

await test("walkthrough: Asha fills gm_0 from what the screens say, and marks it ready", async () => {
  // the lead issues gm_0 to Asha, as the lead's guide says: People, then Map → Issue
  await group.getByRole("tab", { name: "People" }).click();
  for (const [n, r] of [["Lead of act", "lead"], ["Asha", "author"], ["Meera", "stage owner"]]) {
    await group.getByRole("button", { name: "Add or change a member…" }).click();
    const d = group.locator("dialog[open]").last();
    await d.getByLabel("Name").fill(n);
    await d.getByLabel("Role").selectOption(r);
    await d.getByRole("button", { name: "Next" }).click();
    await doIt(group);
  }
  await group.getByRole("tab", { name: "Stages" }).click();
  await group.getByRole("row", { name: /^mtq\b/ }).click();
  await group.locator("dialog[open]").last().getByLabel("Owner (signs the stage)").selectOption("Meera");
  await group.locator("dialog[open]").last().getByRole("button", { name: "Next" }).click();
  await doIt(group);
  await group.getByRole("tab", { name: "Map" }).click();
  await group.locator('[data-node="gm_0"]').click();
  await group.getByRole("button", { name: "Issue…" }).click();
  await group.locator("dialog[open]").last().getByLabel("Author").selectOption("Asha");
  await group.locator("dialog[open]").last().getByRole("button", { name: "Next" }).click();
  await doIt(group);

  await node.evaluate(() => { localStorage.setItem("trinetra.who", "Asha"); });
  await node.reload();
  await node.evaluate(async () => { await window.tnNode.ready; await window.tnNode.useFolder(await (await navigator.storage.getDirectory()).getDirectoryHandle("design")); });
  if (await tourCard(node).count()) await tourCard(node).getByRole("button", { name: "Close the tour" }).click();
  await node.getByRole("row", { name: /gm_0/ }).first().click();
  await node.getByRole("button", { name: "Start from the spec" }).click();
  await node.waitForFunction(() => !window.tnNode.app.busy && window.tnNode.app.cur.undoStack.length > 0);
  await node.getByRole("tab", { name: /^Explanation/ }).click();
  await fieldByLabel(node, "Where it breaks", "Above the coil's rated current the dipole no longer grows linearly.");
  await node.getByRole("tab", { name: /^Belief record/ }).click();
  await fieldByLabel(node, "Area", "model");
  await fieldByLabel(node, "What was believed", "The spec's value holds for the reference satellite.");
  await fieldByLabel(node, "Did a test break it, hold it, or has none been run?", "held");
  await fieldByLabel(node, "What tested it", "Compared with the reference case's sizing script.");
  await fieldByLabel(node, "What we now know", "It holds within 2 %.");
  await fieldByLabel(node, "What changed in the plan", "None; the value is kept.");
  await node.getByRole("tab", { name: /^Review/ }).click();
  const mark = node.getByRole("button", { name: "Mark ready" });
  ok(await mark.isEnabled(), `nothing left to fix: ${await node.locator("main").innerText()}`);
  await mark.click();
  await node.waitForFunction(() => window.tnNode.app.standing && window.tnNode.app.standing.state === "ready");
  await node.getByRole("button", { name: "Save" }).click();
  await node.waitForFunction(() => !window.tnNode.app.cur.dirty);
  await node.getByRole("tab", { name: "Preview" }).click();
  await node.getByRole("button", { name: "Print" }).click();
  const printed = await node.evaluate(() => window.__printed.at(-1));
  ok(/dipole/i.test(printed) && !printed.includes("Mark ready"), `the node printed alone: ${printed.slice(0, 120)}`);
  await node.getByRole("button", { name: /Nodes/ }).first().click();
});

await test("walkthrough: Ravi checks it; the author could not", async () => {
  await node.evaluate(() => { localStorage.setItem("trinetra.who", "Ravi"); });
  await node.reload();
  await node.evaluate(async () => { await window.tnNode.ready; await window.tnNode.useFolder(await (await navigator.storage.getDirectory()).getDirectoryHandle("design")); });
  await node.getByRole("row", { name: /gm_0/ }).first().click();
  await node.getByRole("tab", { name: /^Review/ }).click();
  await node.getByRole("button", { name: "Sign as checked…" }).click();
  const d = node.locator("dialog[open]").last();
  await d.getByLabel("Your name").fill("Ravi");
  await d.getByLabel("What you checked").fill("The value against the coil's datasheet, and the explanation.");
  await d.getByRole("button", { name: /Sign/ }).click();
  await node.waitForFunction(() => window.tnNode.app.standing && window.tnNode.app.standing.state === "checked");
  await node.getByRole("button", { name: "Save" }).click();
  await node.waitForFunction(() => !window.tnNode.app.cur.dirty);
  await node.getByRole("button", { name: /Nodes/ }).first().click();
});

await test("walkthrough: Meera signs stage mtq, the lead seals act 1.0, gm_0 goes in confirmed", async () => {
  const as = (who) => group.evaluate((w) => { const a = window.tnGroup.app; a.who = w; a.ws.who = w; }, who);
  await as("Meera");
  await group.getByRole("tab", { name: "Assemble" }).click();
  await group.getByRole("row", { name: /^mtq\b/ }).getByRole("button", { name: "Sign…" }).click();
  await doIt(group);
  await as("Lead of act");
  await group.getByRole("tab", { name: "Release" }).click();
  await group.getByRole("button", { name: /^Seal 1\.0/ }).click();
  const d = group.locator("dialog[open]").last();
  await d.waitFor({ timeout: 120000 });
  ok((await d.innerText()).includes("1 of 138 node(s) sealed as confirmed"), (await d.innerText()).slice(0, 800));
  await doIt(group);
  await group.getByRole("tab", { name: "Release" }).click();
  ok((await group.getByRole("row", { name: /^1\.0/ }).innerText()).includes("1 of 138"), "act 1.0, with gm_0 confirmed");
  ok(outside.length === 0, `nothing fetched from outside: ${outside.slice(0, 3)}`);
});

await test("TRI-NETRA Files shows its tour and its guide too", async () => {
  const ctx2 = await browser.newContext();
  const p = await ctx2.newPage();
  await p.goto(`${BASE}/files.html`);
  await p.evaluate(() => window.tnFiles.ready);
  await p.locator("[data-testid=tour]").waitFor();
  ok((await p.locator("[data-testid=tour]").innerText()).includes("Open the folder"), "the files tour");
  await p.locator("[data-testid=tour]").getByRole("button", { name: "Close the tour" }).click();
  await p.getByRole("button", { name: "Help" }).click();
  ok((await p.locator("[data-testid=manual-page]").innerText()).startsWith("Guide to TRI-NETRA Files"), "its guide first");
  await ctx2.close();
});

server.close();
await browser.close();
console.log(`help: ${passed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
