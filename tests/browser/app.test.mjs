// app.test.mjs -- the desktop app's page (docs/MAIN_APP.md, docs/RELEASE_PLAN.md P11) in Chromium,
// against a running trinetra-app that reads a design database: it starts without an error and
// fetches nothing from outside; it flies a scenario and shows the verdicts and the figures the
// plotting module drew; the run is listed and shown; the Design tab shows a case value by value
// with the node that declares it.
//
//   node tests/browser/app.test.mjs http://127.0.0.1:PORT     (tests/test_design_source.py runs this)
// Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
const base = process.argv[2];
const pw = await import(process.env.PLAYWRIGHT_MODULE || "playwright").catch(() => import("/opt/node22/lib/node_modules/playwright/index.mjs"));
const browser = await pw.chromium.launch(process.env.CHROME ? { executablePath: process.env.CHROME } : {});
const page = await (await browser.newContext()).newPage();
const errors = [], outside = [];
page.on("pageerror", (e) => errors.push(e.message));
page.on("request", (r) => { if (!r.url().startsWith(base) && !/^(data:|blob:)/.test(r.url())) outside.push(r.url()); });
let passed = 0, failed = 0;
async function test(name, fn) {
  try { await fn(); passed++; console.log(`PASS ${name}`); } catch (e) { failed++; console.log(`FAIL ${name}: ${e.stack || e}`); }
}
function ok(c, msg) { if (!c) throw new Error(msg); }
const done = () => page.evaluate(() => window.tnApp.done);
async function waitDone(n) { await page.waitForFunction((k) => window.tnApp.done > k, n, { timeout: 120000 }); }

await page.goto(base);
await page.evaluate(() => window.tnApp.ready);

await test("it opens on the design database's cases and scenarios", async () => {
  const v = await page.textContent('[data-testid="version"]');
  ok(v.includes("from the design database"), v);
  const cases = await page.$$eval('[data-testid="case"] option', (o) => o.map((x) => x.value));
  ok(cases.includes("ais_3u") && cases.includes("ais_img_3u"), cases.join());
});

await test("it flies a scenario and shows the verdicts and the drawn figures", async () => {
  await page.selectOption('[data-testid="case"]', "ais_3u");
  await page.selectOption('[data-testid="scen"]', "nadir_hold_ais");
  await page.fill('[data-testid="dur"]', "120");
  const n = await done();
  await page.click('[data-testid="go"]');
  await waitDone(n);
  ok(await page.$('[data-testid="verdicts"]'), "a verdict table");
  await page.waitForSelector("img[data-figure]", { timeout: 60000 });
  await page.waitForFunction(() => [...document.querySelectorAll("img[data-figure]")].every((i) => i.complete && i.naturalWidth > 0), null, { timeout: 60000 });
  const k = await page.$$eval("img[data-figure]", (x) => x.length);
  ok(k >= 3, `${k} figures`);
  ok(await page.$('[data-testid="report-pdf"]'), "the report can be taken as PDF");
});

await test("the report comes as PDF and HTML", async () => {
  const run = await page.evaluate(() => window.tnApp.last);
  const r = await page.evaluate(async (run) => {
    const a = await fetch(`/v1/report?run=${encodeURIComponent(run)}&format=pdf`), b = await fetch(`/v1/report?run=${encodeURIComponent(run)}&format=html`);
    return [a.headers.get("Content-Type"), new TextDecoder().decode((await a.arrayBuffer()).slice(0, 5)), (await b.text()).includes("<svg")];
  }, run);
  ok(r[0] === "application/pdf" && r[1] === "%PDF-" && r[2], JSON.stringify(r));
});

await test("the run is listed, and shows its input hash and figures", async () => {
  await page.click('[data-tab="Runs"]');
  await page.waitForSelector('[data-testid="runs"] tbody tr');
  const n = await done();
  await page.click('[data-testid="runs"] tbody tr');
  await waitDone(n);
  const t = await page.textContent('[data-testid="run-detail"]');
  ok(t.includes("Input hash") && t.includes("engine.duration_s=120") && t.includes("the design database"), t.slice(0, 300));
  await page.waitForSelector('[data-testid="run-detail"] img[data-figure]');
});

await test("the Design tab shows a case value by value, with its node", async () => {
  await page.click('[data-tab="Design"]');
  await page.waitForSelector('[data-testid="case-rows"]');
  const rows = await page.$$eval('[data-testid="case-rows"] tbody tr', (r) => r.map((x) => x.textContent));
  ok(rows.some((r) => r.includes("req.ape") && r.includes("p1k_0")), rows.slice(0, 5).join(" | "));
  ok((await page.$$eval('[data-testid="groups"] tbody tr', (r) => r.length)) === 20, "20 groups");
});

await test("nothing failed and nothing was fetched from outside", async () => {
  ok(!errors.length, errors.join("; "));
  ok(!outside.length, outside.join(", "));
});

await browser.close();
console.log(`app: ${passed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
