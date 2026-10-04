// node.test.mjs -- TRI-NETRA Node in a real browser (Chromium through Playwright): the proof of
// docs/RELEASE_PLAN.md P5. Every node kind is filled and previewed through the page: declared
// (gm_0), computed (m2_4, with its pseudocode run on a test vector), KPI (p1k_0), evidence (p1a_0,
// with results pasted from a spreadsheet), closure, interface (with a picture too big, made
// smaller by the wizard), and a row the spec has not named yet (its author chooses the kind).
// Along the way: the live checks name what is missing and then clear, the spec's values are taken
// in, the equation helper inserts, a contract change is acknowledged, the lead's comment is shown,
// a node is marked ready and signed by someone other than its author, and an edit takes the
// signature off.
//
//   node tests/browser/node.test.mjs PAGE.html DESIGN_DIR OUT_DIR
// DESIGN_DIR: a seeded design folder, served and copied into the browser's private folder;
// OUT_DIR gets it back, for tools/tndb.py and tools/group.py. tests/test_pages.py runs this.
// Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
import { createServer } from "node:http";
import { readFileSync, writeFileSync, mkdirSync, readdirSync, statSync } from "node:fs";
import path from "node:path";
import { deflateSync } from "node:zlib";

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
  try { await Promise.race([fn(), new Promise((_, rej) => setTimeout(() => rej(new Error("did not finish in 180 s")), 180000))]); passed++; console.log(`PASS ${name}`); }
  catch (e) { failed++; console.log(`FAIL ${name}: ${e.stack || e}`); }
}
function ok(c, msg) { if (!c) throw new Error(msg); }

const ctx = await browser.newContext({ viewport: { width: 1280, height: 900 } });
ctx.setDefaultTimeout(20000);
await ctx.addInitScript(() => { try { localStorage.setItem("trinetra.who", "Asha"); } catch (e) { /* ignore */ } });
const outside = [];
const page = await ctx.newPage();
page.on("request", (r) => { if (!/^(data:|blob:)/.test(r.url()) && !r.url().startsWith(BASE)) outside.push(r.url()); });
page.on("pageerror", (e) => console.log("pageerror:", e.message));
await page.goto(`${BASE}/node.html`);
await page.evaluate(() => window.tnNode.ready);

const doc = () => page.evaluate(() => { const d = window.tnNode.app.doc; return { kind: d.kind, content: d.content, inputs: d.inputs, fixtures: d.fixtures, node: d.node, attachments: d.attachments }; });
const problems = () => page.evaluate(() => [...document.querySelectorAll("[data-testid^=problems-] li")].map((l) => l.textContent));
const open = async (id) => { await page.evaluate(() => { const c = window.tnNode.app.cur; return c && c.dirty && !c.readOnly ? c.save() : null; }); await page.evaluate((n) => window.tnNode.openNode(n), id); await page.locator("[data-testid=steps]").waitFor(); };
const go = (step) => page.evaluate((s) => window.tnNode.goTo(s), step);
const steps = () => page.evaluate(() => window.tnNode.app.cur.undoStack.length);
async function set(testid, value) {
  const before = await steps();
  const el = page.locator(`[data-testid="${testid}"]`).first();
  const tag = await el.evaluate((x) => x.tagName);
  if (tag === "SELECT") await el.selectOption(value);
  else if (tag === "FIELDSET") { for (const v of value) await el.locator(`input[value="${v}"]`).check(); }
  else { await el.fill(value); await el.evaluate((x) => x.dispatchEvent(new Event("change", { bubbles: true }))); }
  await page.waitForFunction((b) => window.tnNode.app.cur.undoStack.length > b && !window.tnNode.app.busy, before);
}
async function fillAt(step, values) { for (const [k, v] of Object.entries(values)) { await go(step); await set(`f-${k}`, v); } }
const BELIEF = { "belief.area": "model", "belief.believed": "The spec's value holds for the reference satellite.", "belief.status": "held",
  "belief.tested": "Compared with the reference case's sizing script.", "belief.now_know": "It holds within 2 %.", "belief.plan_change": "None; the value is kept." };
async function belief() { await fillAt("belief", BELIEF); }
async function save() { await page.locator("[data-testid=save]").click(); await page.waitForFunction(() => !window.tnNode.app.cur.dirty); }
async function preview() {
  await page.evaluate(() => { const t = document.querySelectorAll(".tn-tab"); t[t.length - 1].click(); });
  await page.locator("[data-testid=preview]").waitFor();
  return page.locator("[data-testid=preview]").innerText();
}

await test("the design folder opens and lists the nodes", async () => {
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
    window.__root = root;
    await window.tnNode.useFolder(root);
    return list.length;
  }, BASE);
  ok(n === 754, `${n} files`);
  ok((await page.locator("[data-testid=mine] tbody tr").count()) === 1, "gm_0 is issued to Asha and listed first");
  await page.locator("[data-testid=search]").fill("orbit radius");
  await page.locator("[data-testid=search]").press("Tab");
  await page.locator("[data-testid=nodes] tbody tr", { hasText: "m2_4" }).waitFor();
});

await test("declared (gm_0): the spec's values taken in, the explanation, the belief; ready; previewed; the contract change acknowledged; the lead's comment shown", async () => {
  await open("gm_0");
  ok((await doc()).kind === "declared", "a declared node");
  ok((await page.locator("main").innerText()).includes("The contract changed"), "the contract change is waiting");
  ok((await page.locator("main").innerText()).includes("Check the value against the datasheet"), "the lead's comment is on Home");
  await page.locator("[data-testid=adopt]").click();
  await page.waitForFunction(() => window.tnNode.app.doc.origin["value.number"] === "spec:seed_content.toml" && window.tnNode.app.cur.undoStack.length > 0);
  await page.locator("[data-testid=ack]").click();
  await page.waitForFunction(() => Number(window.tnNode.app.doc.node.contract_version) === 2);
  await go("explain");
  ok((await problems()).some((p) => p.includes("E20")), "the explanation standard asks where it breaks");
  await fillAt("explain", { "explain.where_it_breaks": "Above the coil's rated current the dipole no longer grows linearly.", "explain.analogy": "Like a sail in the wind." });
  ok((await problems()).some((p) => p.includes("X01")), "an analogy must say where it breaks (X01)");
  await fillAt("explain", { "explain.analogy_breaks": "A sail can be trimmed to any angle; a coil's torque is always across the field." });
  await belief();
  await go("review");
  ok(await page.locator("[data-testid=ready]").isEnabled(), "nothing left to fix: it can be marked ready");
  await page.locator("[data-testid=ready]").click();
  await page.waitForFunction(() => window.tnNode.app.standing.state === "ready");
  await save();
  const p = await preview();
  ok(p.includes("Dipole available per axis is a declared node") && p.includes("0.45") && /where it breaks/i.test(p), `the preview: ${p.slice(0, 300)}`);
});

await test("typing on into the next field while the last one is saved loses nothing", async () => {
  await open("gm_0");
  await go("explain");
  const before = await steps();
  const a = page.locator("[data-testid='f-explain.real_thing']"), b = page.locator("[data-testid='f-explain.try_it']");
  await a.fill("m = N I A, the dipole of a flat coil.");
  await b.click();                         // leaving the first field saves it (a change event)
  await page.keyboard.type("Double the turns");
  await page.waitForFunction((n) => window.tnNode.app.cur.undoStack.length > n && !window.tnNode.app.busy, before);
  ok((await page.locator("[data-testid='f-explain.try_it']").inputValue()) === "Double the turns", "the half-typed text is still there");
  ok(await page.evaluate(() => document.activeElement.getAttribute("data-testid") === "f-explain.try_it"), "and still has the focus");
  await page.keyboard.type(": what happens?");
  await page.locator("[data-testid='f-explain.real_thing']").click();
  await page.waitForFunction(() => window.tnNode.app.doc.content["explain.try_it"] === "Double the turns: what happens?" && !window.tnNode.app.busy);
  await save();
});

await test("computed (m2_4): inputs, the relation with the equation helper, pseudocode run on its test vector; ready and checked by another engineer; an edit takes the signature off", async () => {
  await open("m2_4");
  ok((await doc()).kind === "computed", "a computed node");
  await page.locator("[data-testid=adopt]").click();
  await page.waitForFunction(() => window.tnNode.app.cur.undoStack.length > 0);
  ok((await doc()).inputs.length === 1 && (await doc()).inputs[0].from_node === "m2_0", "its input from the spec");
  await go("theory");
  const before = await page.locator("[data-testid='f-relation.expression']").inputValue();
  await page.locator("[data-testid=palette] button", { hasText: "x²" }).click();
  ok((await page.locator("[data-testid='f-relation.expression']").inputValue()) === before + "^2", "the helper inserts at the caret");
  await set("f-relation.expression", "r = R_E + h");
  ok((await page.locator("[data-testid=equation-reads]").innerText()).includes("R"), "the equation is shown as it reads");
  await fillAt("explain", { "explain.simply": "How far the satellite is from the middle of the Earth.", "explain.one_line": "Earth's radius plus the altitude.",
    "explain.where_it_breaks": "For an eccentric orbit the radius changes along the orbit." });
  await go("pseudocode");
  await set("f-code.pseudocode", "fn radius(h: real[m]) -> r: real[m]\n    r = 6378137 [m] + q\nend\n");
  ok((await page.locator("[data-testid=pcode-problems]").innerText()).includes("q"), "the language's checker names the unknown name");
  await set("f-code.pseudocode", "fn radius(h: real[m]) -> r: real[m]\n    r = 6378137 [m] + h\nend\n");
  await go("evidence");
  await set("f-fixtures", "v1 | h=500000 | 6878137 | 1e-12 | published-source | vallado2013 | p. 98, eq. 2-58");
  await go("pseudocode");
  ok((await page.locator("[data-testid=try-it]").innerText()).includes("reproduces it"), "the pseudocode reproduces the test vector");
  await belief();
  await go("review");
  await page.locator("[data-testid=ready]").click();
  await page.waitForFunction(() => window.tnNode.app.standing.state === "ready");
  // the author cannot check their own node; another engineer can
  await page.evaluate(() => window.tnNode.app.cur.change("issue", (db) => db.run("UPDATE node SET author = 'Asha'")).then(() => window.tnNode.refresh()));
  await page.locator("[data-testid=sign]").click();
  await page.locator("[data-testid=f-checker]").fill("Asha");
  await page.locator("[data-testid=f-statement]").fill("Checked the relation against Vallado eq. 2-58.");
  await page.locator("dialog[open]").getByRole("button", { name: "Sign" }).click();
  await page.locator(".tn-toast.error", { hasText: "cannot check their own" }).waitFor();
  await page.evaluate(() => window.tnNode.app.cur.undo().then(() => window.tnNode.refresh()));
  await page.locator("[data-testid=sign]").click();
  await page.locator("[data-testid=f-checker]").fill("Ravi");
  await page.locator("[data-testid=f-statement]").fill("Checked the relation against Vallado eq. 2-58 and the test vector by hand.");
  await page.locator("dialog[open]").getByRole("button", { name: "Sign" }).click();
  await page.waitForFunction(() => window.tnNode.app.standing.state === "checked");
  await save();
  let p = await preview();
  ok(p.includes("reproduces it") && p.includes("vallado2013") && p.includes("It owes nothing"), `the preview: ${p.slice(0, 400)}`);
  await go("theory");
  await set("f-relation.how_to_read", "Subtract the Earth's radius to get the altitude back.");
  ok(await page.evaluate(() => window.tnNode.app.standing.checkedStale), "an edit takes the signature off");
  await save();
});

await test("KPI (p1k_0): the requirement's sense from the spec; previewed", async () => {
  await open("p1k_0");
  ok((await doc()).kind === "kpi", "a KPI requirement");
  await page.locator("[data-testid=adopt]").click();
  await page.waitForFunction(() => window.tnNode.app.doc.origin["requirement.sense"] === "spec:seed_content.toml");
  await fillAt("explain", { "explain.where_it_breaks": "A pointing requirement below the sensors' accuracy cannot be verified." });
  await belief();
  await go("review");
  ok(await page.locator("[data-testid=ready]").isEnabled(), "it can be marked ready");
  await save();
  const p = await preview();
  ok(/at most/i.test(p) && /the requirement/i.test(p), `the preview: ${p.slice(0, 300)}`);
});

await test("evidence (p1a_0): the metric and rungs, results pasted from a spreadsheet; previewed", async () => {
  await open("p1a_0");
  ok((await doc()).kind === "evidence", "an evidence node");
  await fillAt("identity", { "identity.question": "What absolute pointing error does the satellite achieve?" });
  await fillAt("explain", { "explain.where_it_breaks": "Only for the scenarios flown; an untested mode is not covered." });
  await fillAt("io", { "output.symbol": "APE_ach", "output.quantity": "Angle", "output.unit": "Metre" });
  ok((await problems()).some((p) => p.includes("O03") && p.includes("does not state Angle")), "a unit that does not state the quantity is named (O03)");
  await fillAt("io", { "output.unit": "Degree" });
  await fillAt("value", { "evidence.metric": "ape", "evidence.rungs": ["sils", "oils"] });
  await go("results");
  await page.locator("[data-testid=f-results-paste]").fill("scenario\tAPE\trung\n\tdeg\t\nfine_hold\t0,031\tsils\nmission\t0.042\toils\n");
  await page.locator("[data-testid=results-preview] tbody tr").nth(1).waitFor().catch(async (e) => { throw new Error(`${e.message}\n${await page.locator("[data-testid=results-preview]").evaluate((x) => x.outerHTML.slice(0, 600))}\n${await page.locator("[data-testid=f-results-paste]").inputValue()}`); });
  const before = await steps();
  await page.locator("[data-testid=keep-results]").click();
  await page.waitForFunction((b) => window.tnNode.app.cur.undoStack.length > b, before);
  await belief();
  await go("review");
  ok(await page.locator("[data-testid=ready]").isEnabled(), "it can be marked ready");
  await save();
  const p = await preview();
  ok(p.includes("0.031") && /APE \[deg\]/i.test(p) && p.includes("SILS, OILS"), `the preview: ${p.slice(0, 500)}`);
});

await test("closure: fixed by the tree; explained, feedback given; previewed", async () => {
  await open("kpi_detumble_time_analysis");
  ok((await doc()).kind === "closure", "a closure");
  const tabsText = await page.locator("[data-testid=steps]").innerText();
  ok(!tabsText.includes("Inputs and output") && tabsText.includes("Feedback"), "only explanation, pictures and feedback");
  await fillAt("explain", { "explain.simply": "Says whether the satellite detumbles in the time the mission requires.", "explain.where_it_breaks": "Only for the initial rates the scenarios start from." });
  await fillAt("feedback", { "other.subject": "Add the worst initial rate to the closure" });
  ok((await problems()).some((p) => p.includes("B02")), "something else needs its description (B02)");
  await fillAt("feedback", { "other.description": "The closure should say which initial rate it was shown for." });
  await save();
  const p = await preview();
  ok(p.includes("closure node") && p.includes("Its shape is fixed by the tree") && p.includes("worst initial rate"), `the preview: ${p.slice(0, 300)}`);
});

await test("interface: a picture too big is made smaller by the wizard; previewed with the picture", async () => {
  await open("l3_mtq_interface");
  ok((await doc()).kind === "interface", "an interface row");
  await fillAt("explain", { "explain.simply": "Where the magnetic actuation group meets the rest of the design." });
  await go("pictures");
  const png = bigPng(900, 900);
  ok(png.length > 512000, `a ${png.length}-byte picture`);
  await page.locator("[data-testid=add-picture]").setInputFiles({ name: "coil.png", mimeType: "image/png", buffer: png });
  await page.locator("dialog[open]").waitFor();
  ok((await page.locator("dialog[open]").innerText()).includes("Made smaller to fit"), "the wizard says it made it smaller");
  await page.locator("[data-testid=f-caption]").fill("The coil on the long panel");
  await page.locator("[data-testid=f-alt]").fill("A rectangular coil of 200 turns.");
  await page.locator("dialog[open]").getByRole("button", { name: "Add it" }).click();
  await page.waitForFunction(() => window.tnNode.app.doc.attachments.length === 1);
  const a = (await doc()).attachments[0];
  ok(a.size <= 512000 && a.mime === "image/jpeg", `stored ${a.size} bytes as ${a.mime}`);
  await save();
  await preview();
  ok((await page.locator("[data-testid=preview] img").count()) === 1, "the preview shows the picture");
  ok((await page.locator("[data-testid=preview] figcaption").innerText()) === "The coil on the long panel", "with its caption");
});

await test("a row the spec has not named: its author chooses the kind, and its steps follow", async () => {
  await open("l3_mtq_row_01");
  ok((await doc()).kind === null, "no kind yet");
  await go("identity");
  ok((await problems()).some((p) => p.includes("N02")), "it asks for the kind");
  await fillAt("identity", { "identity.form_kind": "computed" });
  ok((await doc()).kind === "computed", "now computed");
  ok((await page.locator("[data-testid=steps]").innerText()).includes("Pseudocode"), "with a computed node's steps");
  await save();
});

await test("the folder after it all: every node file sound, and the structure untouched", async () => {
  ok(outside.length === 0, `nothing fetched from outside: ${outside.slice(0, 3)}`);
  await page.setViewportSize({ width: 375, height: 800 });
  await open("gm_0");
  ok((await page.evaluate(() => document.documentElement.scrollWidth)) <= 375, "phone width");
  await page.evaluate(() => window.tnNode.closeNode());
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
    }
    return r;
  });
  for (const [p, b64] of Object.entries(out)) { mkdirSync(path.dirname(path.join(outDir, p)), { recursive: true }); writeFileSync(path.join(outDir, p), Buffer.from(b64, "base64")); }
});

server.close();
await browser.close();
console.log(`node: ${passed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);

// a PNG of noise (it does not compress), big enough to need shrinking
function bigPng(w, hgt) {
  const crcT = Array.from({ length: 256 }, (_, n) => { let c = n; for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1; return c >>> 0; });
  const crc = (b) => { let c = 0xffffffff; for (const x of b) c = crcT[(c ^ x) & 255] ^ (c >>> 8); return (c ^ 0xffffffff) >>> 0; };
  const chunk = (t, d) => { const len = Buffer.alloc(4); len.writeUInt32BE(d.length); const td = Buffer.concat([Buffer.from(t), d]); const c = Buffer.alloc(4); c.writeUInt32BE(crc(td)); return Buffer.concat([len, td, c]); };
  const ihdr = Buffer.alloc(13); ihdr.writeUInt32BE(w, 0); ihdr.writeUInt32BE(hgt, 4); ihdr[8] = 8; ihdr[9] = 2;
  const raw = Buffer.alloc((w * 3 + 1) * hgt);
  let s = 12345;
  for (let y = 0; y < hgt; y++) { raw[y * (w * 3 + 1)] = 0; for (let i = 1; i <= w * 3; i++) { s = (Math.imul(s, 1103515245) + 12345) >>> 0; raw[y * (w * 3 + 1) + i] = s >>> 24; } }
  return Buffer.concat([Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]), chunk("IHDR", ihdr), chunk("IDAT", deflateSync(raw)), chunk("IEND", Buffer.alloc(0))]);
}
