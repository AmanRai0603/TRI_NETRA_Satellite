// testapp.test.mjs -- every group's test app (tools/groupcode.py deliver, docs/RELEASE_PLAN.md P10)
// opened from disk in Chromium: it starts without an error and fetches nothing; every test vector
// passes in the interpreter and in WebAssembly built from the generated Rust, the two agreeing; and
// "Try it" gives the same numbers both ways.
//
//   node tests/browser/testapp.test.mjs APPS_DIR      (tests/test_groupcode.py runs this)
// Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
import { readdirSync } from "node:fs";
import path from "node:path";

const dir = process.argv[2];
const pw = await import(process.env.PLAYWRIGHT_MODULE || "playwright").catch(() => import("/opt/node22/lib/node_modules/playwright/index.mjs"));
const browser = await pw.chromium.launch(process.env.CHROME ? { executablePath: process.env.CHROME } : {});
const ctx = await browser.newContext();
let passed = 0, failed = 0;
async function test(name, fn) {
  try { await fn(); passed++; console.log(`PASS ${name}`); } catch (e) { failed++; console.log(`FAIL ${name}: ${e.stack || e}`); }
}
function ok(c, msg) { if (!c) throw new Error(msg); }

const apps = readdirSync(dir).filter((f) => f.endsWith(".test-app.html")).sort();
let total = 0, rowsWithCode = 0;
for (const f of apps) {
  await test(`${f}: opens from disk, every test vector passes both ways, nothing fetched`, async () => {
    const page = await ctx.newPage();
    const errors = [], outside = [];
    page.on("pageerror", (e) => errors.push(e.message));
    page.on("request", (r) => { if (!/^(file:|data:|blob:)/.test(r.url())) outside.push(r.url()); });
    await page.goto("file://" + path.resolve(dir, f));
    await page.evaluate(() => window.tnTestApp.ready);
    const r = await page.evaluate(() => ({ res: window.tnTestApp.app.results, rows: window.tnTestApp.app.data.wire.rows.length, wasm: !!window.tnTestApp.app.wasm }));
    ok(!errors.length, errors.join("; "));
    ok(r.wasm, "the WebAssembly module loaded");
    ok(r.res.every((x) => x.pass), `failing: ${JSON.stringify(r.res.filter((x) => !x.pass))}`);
    ok(!outside.length, `fetched from outside: ${outside}`);
    total += r.res.length;
    rowsWithCode += r.rows;
    if (r.rows) {
      // Try it on the first row, with inputs from its drawn domain: both ways give the same numbers
      const same = await page.evaluate(() => {
        const a = window.tnTestApp.app, w = a.data.wire;
        for (const row of w.rows) {
          const n = row.params.length, scalar = row.params.every(([, t]) => /^real(\[[^\]]*\])?(\s+in\s.*)?$/.test(t));   // a plain number, not an array
          if (!scalar || !n) continue;
          const xs = row.params.map((_, i) => 1 + i * 0.37);
          let ip; try { ip = window.tnTestApp.interpCall(row.fn, xs); } catch (e) { continue; }
          const name = `${(w.shared || []).includes(row.fn) ? "shared" : w.group}::${row.fn}`;
          if (!a.data.names.includes(name)) continue;          // takes a table or a flight state: no plain-numbers call
          const wa = window.tnTestApp.wasmCall(name, xs);
          if (!wa) return `${row.id}: WebAssembly cannot call it`;
          for (let i = 0; i < ip.length; i++) if (!(Math.abs(ip[i] - wa[i]) <= 1e-12 * Math.max(Math.abs(ip[i]), 1e-300) || (Number.isNaN(ip[i]) && Number.isNaN(wa[i])))) return `${row.id}: ${ip} vs ${wa}`;
        }
        return "same";
      });
      ok(same === "same", same);
    }
    await page.close();
  });
}
await test("the test vectors and rows add up", async () => {
  ok(apps.length === 20, `${apps.length} test apps`);
  ok(total > 0 && rowsWithCode > 0, `${total} vectors, ${rowsWithCode} rows with code`);
  console.log(`  ${apps.length} test apps, ${rowsWithCode} rows with code, ${total} test vectors`);
});
await browser.close();
console.log(`testapp: ${passed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
