// files.test.mjs -- TRI-NETRA Files in a real browser (Chromium through Playwright): the proof of
// docs/RELEASE_PLAN.md P3. A node file saves to a folder, reopens, survives a crash (before a save
// and in the middle of one) and refuses a second editor; conflict copies are reported; a file
// changed on disk is never overwritten; the caps hold; a newer format is refused; the page runs
// from disk with nothing fetched from anywhere, and at phone width.
//
//   node tests/browser/files.test.mjs PAGE.html FIXTURES_DIR
// FIXTURES_DIR holds n1.node.tndb (a node file), newer.node.tndb (format version 2) and is where
// the test writes saved.node.tndb (what the page saved, for tools/tndb.py check). tests/test_pages.py
// makes them and runs this.
//
// The folder: a picked folder (a Drive folder) needs a person at the browser's folder dialog, so
// the tests hand the page the browser's private folder (navigator.storage.getDirectory()), which
// has the same interface; the page cannot use it from disk (file://), so those tests serve the
// page from 127.0.0.1. Everything after the picker is the code a person runs.
// Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
import { createServer } from "node:http";
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

const [pagePath, fixtures] = process.argv.slice(2);
const pw = await import(process.env.PLAYWRIGHT_MODULE || "playwright").catch(() => import("/opt/node22/lib/node_modules/playwright/index.mjs"));
const { chromium } = pw;
const browser = await chromium.launch(process.env.CHROME ? { executablePath: process.env.CHROME } : {});
const html = readFileSync(pagePath);
const node1 = readFileSync(path.join(fixtures, "n1.node.tndb")).toString("base64");
const newer = readFileSync(path.join(fixtures, "newer.node.tndb")).toString("base64");

const server = createServer((req, res) => { res.writeHead(200, { "content-type": "text/html; charset=utf-8" }); res.end(html); });
await new Promise((r) => server.listen(0, "127.0.0.1", r));
const URL_ = `http://127.0.0.1:${server.address().port}/files.html`;

let passed = 0, failed = 0;
async function test(name, fn) {
  if (process.env.TN_TRACE) console.log(`.. ${name}`);
  try {
    await Promise.race([fn(), new Promise((_, rej) => setTimeout(() => rej(new Error("did not finish in 90 s")), 90000))]);
    passed++; console.log(`PASS ${name}`);
  }
  catch (e) { failed++; console.log(`FAIL ${name}: ${e.stack || e}`); }
}
function ok(c, msg) { if (!c) throw new Error(msg); }

// a context = one browser profile (its private folder and IndexedDB persist across its pages)
async function profile(who = "Asha") {
  const ctx = await browser.newContext();
  ctx.setDefaultTimeout(15000);
  await ctx.addInitScript((w) => { try { localStorage.setItem("trinetra.who", w); } catch (e) { /* ignore */ } }, who);
  return ctx;
}

async function openPage(ctx, { url = URL_ } = {}) {
  const page = await ctx.newPage();
  const outside = [];
  page.on("request", (r) => { if (!/^(data:|blob:|file:)/.test(r.url()) && !r.url().startsWith(URL_)) outside.push(r.url()); });
  page.on("pageerror", (e) => { throw e; });
  await page.goto(url);
  await page.evaluate(() => window.tnFiles.ready);
  page.outside = outside;
  return page;
}

// the browser's private folder as the design-file folder; files put in it from base64
async function folder(page, files = {}, name = "group-act") {
  await page.evaluate(async ([name, files]) => {
    const root = await navigator.storage.getDirectory();
    const dir = await root.getDirectoryHandle(name, { create: true });
    for (const [n, b64] of Object.entries(files)) {
      const w = await (await dir.getFileHandle(n, { create: true })).createWritable();
      await w.write(Uint8Array.from(atob(b64), (c) => c.charCodeAt(0)));
      await w.close();
    }
    window.__dir = dir;
    await window.tnFiles.useFolder(dir);
  }, [name, files]);
}

const status = (page) => page.locator("[data-testid=status]").innerText();
const valueOf = (page, field) => page.evaluate((f) => window.tnFiles.app.cur.query("SELECT value FROM content WHERE field = ?", [f])[0][0], field);
async function editValue(page, field, value) {
  await page.locator("[data-testid=content] tbody tr").filter({ has: page.locator("td:nth-child(2)", { hasText: new RegExp(`^${field}$`) }) }).click();
  await page.locator("[data-testid=value]").fill(value);
  await page.getByRole("button", { name: "Apply" }).click();
  await page.waitForFunction(() => window.tnFiles.app.cur && window.tnFiles.app.cur.dirty);
}
async function crash(page) {
  // the renderer dies at once (as in a crash or a power cut): no unload handler, no last write
  // (some Chromium builds, as CI's headless shell, never report the crash: after 5 s the tab is
  // shut without its unload handlers, which leaves the same state behind)
  const cdp = await page.context().newCDPSession(page);
  const dead = new Promise((r) => page.once("crash", r));
  cdp.send("Page.crash").catch(() => {});
  await Promise.race([dead, new Promise((r) => setTimeout(r, 5000))]);
  await Promise.race([page.close({ runBeforeUnload: false }).catch(() => {}), new Promise((r) => setTimeout(r, 5000))]);
}
async function fileBytes(page, name) {
  return page.evaluate(async (n) => {
    const f = await (await window.__dir.getFileHandle(n)).getFile();
    let s = ""; const b = new Uint8Array(await f.arrayBuffer());
    for (let i = 0; i < b.length; i += 0x8000) s += String.fromCharCode(...b.subarray(i, i + 0x8000));
    return btoa(s);
  }, name);
}

await test("a node file saves to the folder, reopens with the change, and its history says who and what", async () => {
  const ctx = await profile();
  let page = await openPage(ctx);
  await folder(page, { "n1.node.tndb": node1 });
  ok((await page.locator("[data-testid=folder]").innerText()).includes("n1.node.tndb"), "the folder lists the file");
  await page.locator("[data-testid=folder] tbody tr", { hasText: "n1.node.tndb" }).click();
  await page.waitForSelector("[data-testid=content]");
  await editValue(page, "value", "0.25");
  await page.locator("[data-testid=save]").click();
  await page.waitForFunction(() => !window.tnFiles.app.cur.dirty);
  ok((await status(page)).includes("read back and checked"), "the save is verified");
  writeFileSync(path.join(fixtures, "saved.node.tndb"), Buffer.from(await fileBytes(page, "n1.node.tndb"), "base64"));
  await page.locator("[data-testid=back]").click();
  await page.close();
  page = await openPage(ctx);
  await page.evaluate(async () => { const root = await navigator.storage.getDirectory(); window.__dir = await root.getDirectoryHandle("group-act"); await window.tnFiles.useFolder(window.__dir); await window.tnFiles.openName("n1.node.tndb"); });
  ok((await valueOf(page, "value")) === "0.25", "the change is in the reopened file");
  const hist = await page.locator("[data-testid=history]").innerText();
  ok(hist.includes("Asha") && hist.includes("theory.value"), `the history names who and what: ${hist}`);
  ok(page.outside.length === 0, `nothing fetched from outside: ${page.outside}`);
  await ctx.close();
});

await test("undo and redo step back and forth through the changes", async () => {
  const ctx = await profile();
  const page = await openPage(ctx);
  await folder(page, { "n1.node.tndb": node1 });
  await page.evaluate(() => window.tnFiles.openName("n1.node.tndb"));
  await editValue(page, "value", "1");
  await editValue(page, "value", "2");
  await page.locator("[data-testid=undo]").click();
  ok((await valueOf(page, "value")) === "1", "undo once");
  await page.locator("[data-testid=undo]").click();
  ok((await valueOf(page, "value")) === "0.1", "undo twice");
  await page.locator("[data-testid=redo]").click();
  ok((await valueOf(page, "value")) === "1", "redo");
  // a real stored at full precision comes back exactly on undo
  const back = await page.evaluate(async () => {
    const s = window.tnFiles.app.cur;
    await s.change("fixture", (db) => db.run("INSERT INTO fixture (name, tolerance) VALUES ('f', ?)", [0.1 + 0.2]));
    await s.change("tolerance", (db) => db.run("UPDATE fixture SET tolerance = 1e-300 WHERE name = 'f'"));
    await s.undo();
    return s.query("SELECT tolerance FROM fixture")[0][0];
  });
  ok(back === 0.1 + 0.2, `undo restores a real exactly (${back})`);
  await ctx.close();
});

await test("a crash before saving loses nothing: the work is offered back and restored", async () => {
  const ctx = await profile();
  let page = await openPage(ctx);
  await folder(page, { "n1.node.tndb": node1 });
  await page.evaluate(() => window.tnFiles.openName("n1.node.tndb"));
  await editValue(page, "equation", "tau = J*wdot + w x Jw");
  await crash(page);
  page = await openPage(ctx);
  await page.evaluate(async () => { const root = await navigator.storage.getDirectory(); window.__dir = await root.getDirectoryHandle("group-act"); await window.tnFiles.useFolder(window.__dir); });
  await page.locator("[data-testid=folder] tbody tr", { hasText: "n1.node.tndb" }).click();
  await page.waitForSelector("[data-testid=restore]");
  ok((await valueOf(page, "equation")) === "tau = J*wdot", "the file on disk is the last saved one");
  const back = await page.evaluate(() => ({ ro: window.tnFiles.app.cur.readOnly, notes: window.tnFiles.app.cur.notes.join() }));
  ok(!back.ro && back.notes.includes("taken back"), `the person whose tab crashed gets the file back for editing: ${JSON.stringify(back)}`);
  await page.locator("[data-testid=restore]").click();
  await page.waitForFunction(() => window.tnFiles.app.cur.dirty);
  ok((await valueOf(page, "equation")) === "tau = J*wdot + w x Jw", "the unsaved change is back");
  await page.locator("[data-testid=save]").click();
  await page.waitForFunction(() => !window.tnFiles.app.cur.dirty);
  await ctx.close();
});

await test("a crash in the middle of a write leaves the file whole", async () => {
  const ctx = await profile();
  let page = await openPage(ctx);
  await folder(page, { "n1.node.tndb": node1 });
  const before = await fileBytes(page, "n1.node.tndb");
  await page.evaluate(async () => {
    const w = await (await window.__dir.getFileHandle("n1.node.tndb")).createWritable();
    await w.write(new Uint8Array(4096).fill(7));   // half a file, never closed
    window.__w = w;
  });
  await crash(page);
  page = await openPage(ctx);
  await page.evaluate(async () => { const root = await navigator.storage.getDirectory(); window.__dir = await root.getDirectoryHandle("group-act"); await window.tnFiles.useFolder(window.__dir); await window.tnFiles.openName("n1.node.tndb"); });
  ok((await fileBytes(page, "n1.node.tndb")) === before, "the file is byte for byte what it was");
  ok((await valueOf(page, "value")) === "0.1", "and opens");
  await ctx.close();
});

await test("a second editor gets the file read-only, with the first one's name", async () => {
  const ctx = await profile();
  const a = await openPage(ctx);
  await folder(a, { "n1.node.tndb": node1 });
  await a.evaluate(() => window.tnFiles.openName("n1.node.tndb"));
  const b = await openPage(ctx);
  await b.evaluate(async () => { const root = await navigator.storage.getDirectory(); window.__dir = await root.getDirectoryHandle("group-act"); await window.tnFiles.useFolder(window.__dir); await window.tnFiles.openName("n1.node.tndb"); });
  ok(await b.evaluate(() => window.tnFiles.app.cur.readOnly), "the second tab is read-only");
  ok((await b.locator(".tn-banner.warn").first().innerText()).includes("another tab"), "and says why");
  ok(await b.locator("[data-testid=save]").isDisabled(), "it cannot save");
  // the marker another computer left (Drive synced it): fresh, it refuses; stale, it is taken over
  await a.evaluate(() => window.tnFiles.closeFile());
  await b.evaluate(() => window.tnFiles.closeFile());
  const marker = (beatAgoMin) => b.evaluate(async (m) => {
    const w = await (await window.__dir.getFileHandle("n1.node.tndb.editing", { create: true })).createWritable();
    await w.write(JSON.stringify({ who: "Ravi", session: "other-computer", since: new Date(Date.now() - 7200e3).toISOString(), beat: new Date(Date.now() - m * 60e3).toISOString() }));
    await w.close();
    await window.tnFiles.openName("n1.node.tndb");
    const s = window.tnFiles.app.cur;
    return { ro: s.readOnly, why: s.readOnlyWhy, notes: s.notes };
  }, beatAgoMin);
  const fresh = await marker(2);
  ok(fresh.ro && fresh.why.includes("Ravi"), `a fresh marker from another computer refuses: ${JSON.stringify(fresh)}`);
  await b.evaluate(() => window.tnFiles.closeFile());
  const stale = await marker(60);
  ok(!stale.ro && stale.notes.join().includes("Ravi"), `a stale marker is taken over and said so: ${JSON.stringify(stale)}`);
  const list = await b.evaluate(async () => { await window.tnFiles.closeFile(); return (await window.__dir.getFileHandle("n1.node.tndb.editing").then(() => "marker left", () => "marker removed")); });
  ok(list === "marker removed", "closing removes the marker");
  await ctx.close();
});

await test("conflict copies are reported in the folder and in the file", async () => {
  const ctx = await profile();
  const page = await openPage(ctx);
  await folder(page, { "n1.node.tndb": node1, "n1 (1).node.tndb": node1 });
  const row = await page.locator("[data-testid=folder] tbody tr", { hasText: "n1.node.tndb" }).first().innerText();
  ok(row.includes("1 conflict copy"), `the folder flags it: ${row}`);
  ok(!(await page.locator("[data-testid=folder]").innerText()).includes("n1 (1).node.tndb"), "the copy is not listed as a file of its own");
  await page.evaluate(() => window.tnFiles.openName("n1.node.tndb"));
  ok((await page.locator(".tn-banner.error").innerText()).includes("n1 (1).node.tndb"), "the file names its copy");
  await ctx.close();
});

await test("a file changed on disk since it was opened is never overwritten; the work goes to a copy", async () => {
  const ctx = await profile();
  const page = await openPage(ctx);
  await folder(page, { "n1.node.tndb": node1 });
  await page.evaluate(() => window.tnFiles.openName("n1.node.tndb"));
  await editValue(page, "value", "9");
  const theirs = await page.evaluate(async () => {     // Drive brings in another computer's save
    const fh = await window.__dir.getFileHandle("n1.node.tndb");
    const b = new Uint8Array(await (await fh.getFile()).arrayBuffer()); b[b.length - 1] ^= 1;
    const w = await fh.createWritable(); await w.write(b); await w.close();
    return b.length;
  });
  await page.locator("[data-testid=save]").click();
  await page.waitForSelector(".tn-banner.error");
  ok((await page.locator("main").innerText()).includes("changed on disk"), "the save is refused and says why");
  ok((await fileBytes(page, "n1.node.tndb")).length > 0 && theirs > 0, "their file is still there");
  await page.getByRole("button", { name: "Save a copy" }).first().click();
  await page.locator(".tn-toast", { hasText: "Saved as" }).waitFor();
  const names = await page.evaluate(async () => { const out = []; for await (const [n] of window.__dir.entries()) out.push(n); return out; });
  ok(names.some((n) => /^n1-copy-asha-\d{8}-\d{4}\.node\.tndb$/.test(n)), `the copy is beside it: ${names}`);
  await ctx.close();
});

await test("the caps hold: a picture over 500 KB and a file over 50 MB are refused", async () => {
  const ctx = await profile();
  const page = await openPage(ctx);
  await folder(page, { "n1.node.tndb": node1 });
  await page.evaluate(() => window.tnFiles.openName("n1.node.tndb"));
  await page.locator("[data-testid=add-picture]").setInputFiles({ name: "big.png", mimeType: "image/png", buffer: Buffer.alloc(600 * 1024, 1) });
  await page.locator(".tn-toast.error").waitFor();
  ok((await page.evaluate(() => window.tnFiles.app.cur.query("SELECT count(*) FROM attachment")[0][0])) === 0, "no picture added");
  await page.locator("[data-testid=add-picture]").setInputFiles({ name: "small.png", mimeType: "image/png", buffer: Buffer.alloc(10 * 1024, 1) });
  await page.waitForFunction(() => window.tnFiles.app.cur.query("SELECT count(*) FROM attachment")[0][0] === 1);
  await page.locator("[data-testid=save]").click();
  await page.waitForFunction(() => !window.tnFiles.app.cur.dirty);
  // the file layer refuses it too, whatever the page does
  const code = await page.evaluate(() => window.tnFiles.app.cur.change("big", (db) => db.run("INSERT INTO attachment VALUES ('x', 'image/png', 600000, zeroblob(600000))")).then(() => "taken", (e) => e.code));
  ok(code === "size", `the file layer refuses a big picture: ${code}`);
  const big = await page.evaluate(async () => {
    await window.tnFiles.closeFile();
    const w = await (await window.__dir.getFileHandle("huge.node.tndb", { create: true })).createWritable();
    await w.write(new Uint8Array(51 * 1024 * 1024)); await w.close();
    await window.tnFiles.openName("huge.node.tndb");
    return { cur: !!window.tnFiles.app.cur, text: document.body.innerText };
  });
  ok(!big.cur && big.text.includes("over the 52428800"), "a file over 50 MB is not opened, and the page says why");
  await ctx.close();
});

await test("a file of a newer format is refused by name", async () => {
  const ctx = await profile();
  const page = await openPage(ctx);
  await folder(page, { "newer.node.tndb": newer });
  await page.evaluate(() => window.tnFiles.openName("newer.node.tndb"));
  ok(!(await page.evaluate(() => !!window.tnFiles.app.cur)), "not opened");
  ok((await page.locator("body").innerText()).includes("newer than this page"), "and says so");
  await ctx.close();
});

await test("from disk with nothing fetched: open one file, change it, save by downloading", async () => {
  const ctx = await profile();
  const page = await ctx.newPage();
  const outside = [];
  page.on("request", (r) => { if (!/^(data:|blob:|file:)/.test(r.url())) outside.push(r.url()); });
  await page.goto("file://" + path.resolve(pagePath));
  await page.evaluate(() => window.tnFiles.ready);
  ok((await status(page)).startsWith("Ready"), "the page starts from disk");
  await page.locator("[data-testid=open-file]").setInputFiles({ name: "n1.node.tndb", mimeType: "application/octet-stream", buffer: Buffer.from(node1, "base64") });
  await page.waitForSelector("[data-testid=content]");
  await editValue(page, "value", "0.5");
  const [dl] = await Promise.all([page.waitForEvent("download"), page.locator("[data-testid=save]").click()]);
  const saved = readFileSync(await dl.path());
  ok(dl.suggestedFilename() === "n1.node.tndb" && saved.subarray(0, 15).toString() === "SQLite format 3", "the download is the file");
  ok(outside.length === 0, `nothing fetched from outside: ${outside}`);
  await ctx.close();
});

await test("phone width: nothing runs off the side", async () => {
  const ctx = await browser.newContext({ viewport: { width: 375, height: 760 } });
  ctx.setDefaultTimeout(15000);
  await ctx.addInitScript(() => localStorage.setItem("trinetra.who", "Asha"));
  const page = await openPage(ctx);
  await folder(page, { "n1.node.tndb": node1 });
  await page.evaluate(() => window.tnFiles.openName("n1.node.tndb"));
  const w = await page.evaluate(() => document.documentElement.scrollWidth);
  ok(w <= 375, `page ${w}px wide`);
  await ctx.close();
});

server.close();
await browser.close();
console.log(`files: ${passed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
