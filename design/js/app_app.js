// app_app.js -- the desktop app's page (docs/RELEASE_PLAN.md P11), served by trinetra-app on this
// computer only: pick a case and a scenario from the design database, fly it on the engine, read
// the verdicts and the figures (SVG, from the one plotting module, adcs-plot), take the run's
// report as PDF or HTML, and find, show or export every run kept. Built by tools/pages.py into
// engine/crates/trinetra-app/src/page.html, from the one component set (tn_ui.js, tn.css).
// Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
import { h, fill, button, field, banner, badge, table, section, toolbar, kv, shell, choice, tabs, toast, download } from "./tn_ui.js";

const { main, status } = shell("TRI-NETRA ADCS", "Fly a scenario on the engine with the flight software in the loop, and read what it says. Nothing leaves this computer.");
const app = { version: null, cat: { cases: [], scenarios: [] }, runs: [], last: null, busy: false, done: 0 };
const setStatus = (t) => fill(status, h("span", { "data-testid": "status" }, t));
const enc = encodeURIComponent;

/** A request to the app. A request that changes something carries the app's own header. */
async function call(path, body) {
  const r = body === undefined ? await fetch(path)
    : await fetch(path, { method: "POST", headers: { "Content-Type": "application/json", "X-Trinetra": "1" }, body: JSON.stringify(body) });
  const ct = r.headers.get("Content-Type") || "";
  if (!ct.startsWith("application/json")) {
    if (!r.ok) throw new Error(`${r.status} ${r.statusText}`);
    return r;
  }
  const j = await r.json();
  if (!r.ok || j.ok === false) throw new Error(j.error || `${r.status} ${r.statusText}`);
  return j;
}

function fmt(v) {
  if (v === null || v === undefined || Number.isNaN(v)) return "—";
  const a = Math.abs(v);
  return a !== 0 && (a < 1e-3 || a >= 1e5) ? v.toExponential(3) : String(Number(v.toPrecision(4)));
}

function verdicts(ms) {
  return table([{ key: "id", label: "Metric", mono: true }, { key: "value", label: "Value", render: (m) => fmt(m.value) }, { key: "unit", label: "Unit" },
    { key: "req", label: "Requirement", render: (m) => (m.req === null || m.req === undefined ? "—" : fmt(m.req)) },
    { key: "pass", label: "Verdict", render: (m) => (m.pass === 1 ? badge("pass", "ok") : m.pass === 0 ? badge("fail", "error") : "") }],
  ms || [], { empty: "No metric was judged.", testid: "verdicts" });
}

/** A run's figures (SVG drawn by the engine's plotting module) and its report. */
async function figures(run) {
  const box = h("div", { class: "tn-figures", "data-testid": "figures" }, h("p", { class: "tn-dim" }, "Drawing the figures…"));
  call(`/v1/figures?run=${enc(run)}`).then((r) => {
    fill(box, r.figures.length ? r.figures.map((f) => h("figure", { class: "tn-figure" },
      h("img", { src: `/v1/figure?run=${enc(run)}&name=${enc(f.name)}`, alt: f.title, loading: "lazy", "data-figure": f.name }),
      h("figcaption", {}, f.title, " ", button("SVG", () => keep(`/v1/figure?run=${enc(run)}&name=${enc(f.name)}`, `${f.name}.svg`, "image/svg+xml"), { kind: "quiet", title: "Keep this figure as SVG" }))))
      : h("p", { class: "tn-dim" }, "This run kept no time series (its channels were thinned), so it has no figures."));
  }).catch((e) => fill(box, banner("error", "No figures.", e.message)));
  return box;
}

async function keep(path, name, type) {
  try {
    const r = await call(path);
    download(name, new Uint8Array(await r.arrayBuffer()), type);
  } catch (e) { toast(`Not kept: ${e.message}`, "error"); }
}

function reportButtons(run) {
  const stem = run.split("/").pop();
  return toolbar(
    button("Report (PDF)", () => keep(`/v1/report?run=${enc(run)}&format=pdf`, `${stem}.pdf`, "application/pdf"), { testid: "report-pdf", title: "The run's report: its provenance, verdicts and every figure" }),
    button("Report (HTML)", () => keep(`/v1/report?run=${enc(run)}&format=html`, `${stem}.html`, "text/html"), { testid: "report-html", title: "The same report as one page to open in a browser" }),
    button("Export (.trinetra)", () => keep(`/v1/export?run=${enc(run)}`, `${stem}.trinetra`, "application/zip"), { testid: "export", title: "The run as one file to send; adcs results import opens it" }));
}

// ------------------------------------------------------------------ fly
function flyTab() {
  const cases = app.cat.cases;
  let scen = null;
  const out = h("div", { "data-testid": "result" });
  const scenOpts = (c) => app.cat.scenarios.filter((s) => s.case === c).map((s) => [s.id, `${s.id} — ${s.label || ""}`]);
  const hint = h("p", { class: "tn-help", "data-testid": "scen-hint" });
  const showHint = () => {
    const s = app.cat.scenarios.find((x) => x.id === scen.value);
    hint.textContent = s ? `product ${s.product}; ${Math.round(s.duration_s)} s (${(s.duration_s / 60).toFixed(0)} min) of flight` : "";
  };
  const scenBox = h("div", {});
  const pickScen = (c) => { scen = choice("Scenario", scenOpts(c), { value: scenOpts(c)[0]?.[0] || "", testid: "scen", help: "What happens in the flight: the mode, the start, the faults, how long.", onChange: showHint }); fill(scenBox, scen.el); showHint(); };
  const cs = choice("Case", cases.map((c) => [c.id, `${c.id} — ${c.title}`]), { value: cases[0]?.id || "", testid: "case",
    help: "The satellite and its requirements, as the design database holds them.", onChange: pickScen });
  pickScen(cs.value);
  const fsw = choice("Flight software", [["c", "C"], ["rust", "Rust"]], { value: "c", testid: "fsw", help: "Both are built from the same pseudocode and give the same bytes." });
  const seed = field("Seed", { value: "1", testid: "seed", help: "Which draw of the sensor noise; 1 is the scenario's own." });
  const dur = field("Duration, seconds", { testid: "dur", placeholder: "the scenario's own", help: "Leave it blank for the scenario's own length." });
  const go = button("Fly it", async () => {
    if (app.busy) return;
    const body = { scenario: scen.value, case: cs.value, fsw: fsw.value, seed: Number(seed.value || 1) };
    if (dur.value.trim()) body.duration_s = Number(dur.value);
    app.busy = true; go.disabled = true; go.textContent = "Flying…";
    fill(out, banner("info", `Flying ${body.scenario} on ${body.case}…`, ""));
    try {
      const r = await call("/v1/run", body);
      app.last = r.run;
      fill(out, section(`Verdicts — ${r.scenario} on ${r.case}`, verdicts(r.metrics),
        h("p", { class: "tn-help" }, `${fmt(r.duration_s)} s of flight in ${fmt(r.wall_s)} s; flight software ${r.fsw}; kept as ${r.run}.`),
        reportButtons(r.run)), section("Figures", await figures(r.run)));
      setStatus(`flown: ${r.scenario} on ${r.case}`);
    } catch (e) {
      fill(out, banner("error", "Not flown.", e.message));
    } finally { app.busy = false; go.disabled = false; go.textContent = "Fly it"; app.done++; }
  }, { kind: "primary", testid: "go" });
  return h("div", { class: "tn-two" },
    section("Fly a scenario", cs.el, scenBox, hint, fsw.el, seed.el, dur.el, toolbar(go),
      h("p", { class: "tn-help" }, "The engine flies the plant, the environment and the orbit with the flight software in the loop, then judges every metric against the case's requirements.")),
    out);
}

// ------------------------------------------------------------------ runs
function runsTab() {
  const detail = h("div", { "data-testid": "run-detail" });
  const list = h("div", {}, h("p", { class: "tn-dim" }, "Reading the runs…"));
  call("/v1/runs").then((r) => {
    app.runs = r.runs;
    fill(list, table([{ key: "created", label: "When (UTC)", render: (x) => (x.created || "").replace("T", " ").replace("Z", "") }, { key: "scenario", label: "Scenario", mono: true },
      { key: "case", label: "Case", mono: true }, { key: "pass", label: "Pass / fail", render: (x) => [badge(String(x.pass), "ok"), " ", badge(String(x.fail), x.fail ? "error" : "")] }],
    r.runs.slice(0, 200), { empty: `None yet. Runs are kept in ${r.store}.`, testid: "runs", onRow: (x) => show(x.run) }),
    h("p", { class: "tn-help" }, `Kept in ${r.store}.`));
  }).catch((e) => fill(list, banner("error", "The runs could not be read.", e.message)));
  async function show(run) {
    fill(detail, banner("info", "Reading the run…", ""));
    try {
      const r = await call(`/v1/run?run=${enc(run)}`);
      const i = r.inputs || {};
      fill(detail, section(run, kv([["Input hash", i.input_hash || "—"], ["Differs from the scenario", (i.differs || []).join(", ") || "nothing"],
        ["Flown from", i.design ? `the design database (${String(i.design.fingerprint).slice(0, 12)}…)` : "the data folder's files"]]),
      verdicts(r.metrics), reportButtons(run)), section("Figures", await figures(run)));
    } catch (e) { fill(detail, banner("error", "Not shown.", e.message)); }
    app.done++;
  }
  return h("div", {}, section("Your runs", list), detail);
}

// ------------------------------------------------------------------ the design
function designTab() {
  const box = h("div", {}, h("p", { class: "tn-dim" }, "Reading the design…"));
  call("/v1/design").then((d) => {
    if (!d.design) { fill(box, banner("info", "No design database.", "This app reads the data folder's files. A kit carries design.tndb beside its data; TRINETRA_DESIGN names another.")); return; }
    const caseBox = h("div", {});
    const showCase = (id) => call(`/v1/design/case?case=${enc(id)}`).then((c) => fill(caseBox, table([{ key: "key", label: "Key", mono: true }, { key: "label", label: "What" },
      { key: "value", label: "Value" }, { key: "unit", label: "Unit" }, { key: "node", label: "Node", mono: true }], c.rows, { testid: "case-rows" })));
    const pick = choice("Case", app.cat.cases.map((c) => [c.id, c.id]), { value: app.cat.cases[0]?.id || "", testid: "design-case", help: "Every value the case states, and the node that declares it.", onChange: showCase });
    fill(box, section("The design database", kv([["File", d.design.file], ["Inputs fingerprint", d.design.fingerprint], ["Groups", String(d.groups.length)],
      ["Nodes", String(d.groups.reduce((n, g) => n + g.nodes, 0))]])),
    section("Groups", table([{ key: "id", label: "Group", mono: true }, { key: "version", label: "Release" }, { key: "nodes", label: "Nodes" }], d.groups, { testid: "groups" })),
    section("A case, as the design holds it", pick.el, caseBox));
    if (app.cat.cases.length) showCase(pick.value);
  }).catch((e) => fill(box, banner("error", "The design could not be read.", e.message)));
  return box;
}

async function start() {
  try {
    app.version = await call("/v1/version");
    app.cat = await call("/v1/catalogue");
  } catch (e) {
    fill(main, banner("error", "The app did not answer.", `${e.message}. If you pressed Quit, open it again.`));
    return;
  }
  const v = app.version;
  const t = tabs([["Fly", flyTab], ["Runs", runsTab], ["Design", designTab]], { testid: "tabs" });
  fill(main, toolbar(h("span", { class: "tn-dim", "data-testid": "version" }, `version ${v.version} · ${v.engine} · ${v.design ? "from the design database" : "from the data folder"}`),
    button("Quit", async () => {
      await call("/v1/quit", {}).catch(() => {});
      fill(main, banner("info", "TRI-NETRA ADCS has ended.", "You can close this tab. Your runs are kept; open the app again to see them."));
    }, { kind: "quiet", testid: "quit", title: "End the app" })), t.el);
  setStatus(`${app.cat.cases.length} case(s), ${app.cat.scenarios.length} scenario(s)`);
  setInterval(() => call("/v1/ping", {}).catch(() => {}), 20000);
}

app.ready = start();
window.tnApp = app;
