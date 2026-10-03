// group_app.js -- TRI-NETRA Group, the group lead's app (docs/RELEASE_PLAN.md P4, P6): open the
// design folder, pick a group, and see and change its structure: the map (stages, nodes, what reads
// what, inputs from other groups), its nodes, stages and stage owners, people and who authors what
// (issuing node files), contracts at its boundary, change requests both ways, and its history; and
// take it to a release: every node's progress, the group assembled with the checks across its
// nodes, stages signed by their owners, comments, releases sealed, compared and re-issued from,
// node forms imported. Every change is an action (structure.js, release.js): its impact is shown
// before it is done, a block stops it, and it changes every file it touches or none.
// Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
import { Journal, FileRefused } from "./tnfile.js";
import { Workspace, integrity, summary, readersOutside, readersInside, inputsFromOutside, requestsTo, STRUCTURE, NODES } from "./structure.js";
import { Releases, compare, nowNodes, parseForm, RELEASES } from "./release.js";
import { renderNode } from "./node_view.js";
import { h, fill, button, field, banner, badge, table, section, toolbar, spacer, kv, dialog, ask, toast, shell, tabs, graph, impactList, choice, checks, fileButton } from "./tn_ui.js";

const canFolders = typeof window.showDirectoryPicker === "function";
const { main, status } = shell("TRI-NETRA Group", "A group's structure and its releases: map, nodes, stages, people, contracts, change requests; progress, assemble, seal. Nothing leaves this computer.");
const RELEASE_ACTIONS = new Set(["seal", "signStage", "reissue", "comment", "import"]);
const journal = new Journal();
const session = crypto.getRandomValues(new Uint32Array(2)).join("-");
const app = { SQL: null, root: null, ws: null, rel: null, asm: null, gid: null, sel: null, tab: 0, who: null, profile: null, problems: [] };

function remembered(k) { try { return localStorage.getItem(k); } catch (e) { return null; } }
function remember(k, v) { try { localStorage.setItem(k, v); } catch (e) { /* private window */ } }
function fmtTime(t) { return t ? new Date(t).toLocaleString() : "—"; }
function setStatus(t) { fill(status, h("span", { "data-testid": "status" }, t)); }
function render(...parts) { fill(main, ...parts); }

async function who() {
  if (app.who) return app.who;
  app.who = remembered("trinetra.who");
  while (!app.who) app.who = await ask("Your name", "Name", { help: "Written beside every change you make. There are no accounts: Drive's record of who saved each file backs it up." });
  remember("trinetra.who", app.who);
  return app.who;
}

async function guard(what, fn) {
  try { return await fn(); }
  catch (e) {
    const msg = e instanceof FileRefused ? e.message : `${what}: ${e.message || e}`;
    toast(msg.split("\n")[0], "error");
    main.prepend(banner("error", `${what} did not happen.`, msg));
    return undefined;
  }
}

// ------------------------------------------------------------------ the folder
async function pickFolder() {
  const dir = await guard("Opening the folder", () => window.showDirectoryPicker({ id: "trinetra-design", mode: "readwrite" }));
  if (dir) await useFolder(dir);
}

async function reopenFolder() {
  const dir = await guard("Reopening the folder", () => journal.pref("design-folder"));
  if (!dir) return;
  const ok = (await dir.queryPermission({ mode: "readwrite" })) === "granted" || (await dir.requestPermission({ mode: "readwrite" })) === "granted";
  if (ok) await useFolder(dir);
}

async function useFolder(root) {
  await who();
  const ok = await guard("Opening the design folder", async () => {
    await root.getDirectoryHandle(STRUCTURE);
    await root.getDirectoryHandle(NODES);
    return true;
  });
  if (!ok) { main.prepend(banner("error", "Not a design folder.", `A design folder holds ${STRUCTURE}/ (the group files) and ${NODES}/ (the node files). Pick the folder that holds both.`)); return; }
  app.root = root;
  app.ws = new Workspace({ SQL: app.SQL, root, who: app.who, session, profile: app.profile, journal, locks: navigator.locks || null });
  app.rel = new Releases(app.ws);
  await journal.setPref("design-folder", root).catch(() => {});
  await app.ws.refresh();
  await showGroups();
}

async function unfinishedBanners() {
  const u = await app.ws.unfinished();
  return u.map((r) => banner("error", "A structure action did not finish.", `${r.summary || r.id} by ${r.by || "?"} at ${fmtTime(r.at)}: some of its files were written, some not.`,
    button("Finish it", async () => { const res = await guard("Finishing", () => app.ws.finish(r.id)); if (res) { toast(res.finished ? "Finished" : "Not all of it: see the list"); if (!res.finished) main.prepend(banner("warn", "", res.report.join("\n"))); rerender(); } }, { kind: "primary", testid: "finish" })));
}

async function showGroups() {
  app.gid = null;
  const idx = app.ws.index;
  const rows = [...idx.groups.values()].map((g) => {
    const live = [...g.nodes.values()].filter((n) => n.state !== "archived").length;
    const open = requestsTo(idx, g.id).filter((r) => r.state === "open").length;
    return { id: g.id, label: g.label, lead: g.lead_team || g.lead || "—", nodes: live, stages: g.stages.length, open };
  });
  render(toolbar(...topButtons()), ...(await unfinishedBanners()),
    idx.problems.length ? banner("error", "Problems reading the group files.", idx.problems.join("\n")) : null,
    section(`Design folder: ${app.root.name}`, table([
      { key: "id", label: "Group", mono: true }, { key: "label", label: "Name" }, { key: "lead", label: "Lead team" },
      { key: "nodes", label: "Nodes" }, { key: "stages", label: "Stages" },
      { key: "open", label: "Requests to it", render: (r) => (r.open ? badge(`${r.open} open`, "warn") : "") },
    ], rows, { onRow: (r) => openGroup(r.id), testid: "groups" })),
    toolbar(button("Check the whole design", () => checkAll(), { testid: "check-all" })));
  setStatus(`${rows.length} groups · ${idx.owner.size} nodes`);
}

function topButtons() {
  const b = [];
  if (canFolders) b.push(button("Open design folder…", pickFolder, { kind: app.root ? "default" : "primary", testid: "open-folder" }), button("Reopen last folder", reopenFolder, { kind: "quiet" }));
  b.push(spacer(), button(`You: ${app.who || "?"}`, async () => { const n = await ask("Your name", "Name", { value: app.who || "" }); if (n) { app.who = n; remember("trinetra.who", n); if (app.ws) app.ws.who = n; rerender(); } }, { kind: "quiet" }));
  return b;
}

async function checkAll() {
  setStatus("Checking every group and every node file…");
  const p = await guard("Checking", () => integrity(app.SQL, app.root));
  if (!p) return;
  toast(p.length ? `${p.length} problem(s)` : "Every rule holds: no node file broken");
  main.prepend(p.length ? banner("error", `${p.length} problem(s) in the design.`, p.join("\n")) : banner("ok", "Every rule holds.", `${app.ws.index.owner.size} nodes in ${app.ws.index.groups.size} groups, every node file as its group says.`));
  setStatus(p.length ? `${p.length} problem(s)` : "every rule holds");
}

// ------------------------------------------------------------------ a group
async function openGroup(gid) {
  app.gid = gid;
  app.sel = null;
  app.tab = 0;
  app.asm = null;
  app.problems = await integrity(app.SQL, app.root, { index: app.ws.index, scope: [gid] }).catch((e) => [String(e)]);
  await showGroup();
}

function G() { return app.ws.index.groups.get(app.gid); }

async function showGroup() {
  const g = G();
  const live = [...g.nodes.values()].filter((n) => n.state !== "archived").length;
  const t = tabs([
    ["Map", () => mapView()],
    ["Nodes", () => nodesView()],
    ["Stages", () => stagesView()],
    ["People", () => peopleView()],
    ["Contracts", () => contractsView()],
    ["Requests", () => requestsView()],
    ["Progress", () => assembled(progressView)],
    ["Assemble", () => assembled(assembleView)],
    ["Release", () => assembled(releaseView)],
    ["History", () => historyView()],
  ], { testid: "tabs" });
  render(
    toolbar(button("◂ Groups", () => showGroups(), { kind: "quiet", testid: "groups-back" }), spacer(),
      button("Check this group", async () => { app.problems = await integrity(app.SQL, app.root, { index: app.ws.index, scope: [app.gid] }); await showGroup(); toast(app.problems.length ? `${app.problems.length} problem(s)` : "Every rule holds"); }, { testid: "check" })),
    ...(await unfinishedBanners()),
    app.problems.length ? banner("error", `${app.problems.length} problem(s) in ${g.id}.`, app.problems.join("\n")) : banner("ok", "", `Every rule holds in ${g.id}: every node file as the group says.`),
    section(`${g.id} · ${g.label || ""}`, kv([["Lead team", g.lead_team || "—"], ["Lead", g.lead || "—"], ["Nodes", `${live} (${g.nodes.size - live} archived)`],
      ["Stages", g.stages.map((s) => s.label || s.id).join(", ") || "none"]])),
    t.el);
  t.show(app.tab);
  for (const [i, b] of [...t.el.querySelectorAll(".tn-tab")].entries()) b.addEventListener("click", () => { app.tab = i; });
  setStatus(`${g.id}: ${live} nodes · ${g.edges.length} edges · ${requestsTo(app.ws.index, g.id).filter((r) => r.state === "open").length} open request(s) to it`);
}

async function rerender() {
  if (!app.ws) return;
  if (app.gid) await showGroup(); else await showGroups();
}

function mapView() {
  const g = G(), idx = app.ws.index;
  const outside = new Map();
  for (const e of inputsFromOutside(idx, g.id)) outside.set(e.from_node, { id: e.from_node, label: `${e.from_group}: ${idx.groups.get(e.from_group)?.nodes.get(e.from_node)?.label || e.from_node}`, kind: "outside" });
  const kindOf = (n) => (n.id === app.sel ? "selected" : n.state === "archived" ? "archived" : "");
  const cols = [];
  if (outside.size) cols.push({ label: "From other groups", nodes: [...outside.values()] });
  const nodes = [...g.nodes.values()];
  if (g.stages.length) for (const s of g.stages) cols.push({ label: s.label || s.id, nodes: nodes.filter((n) => n.stage === s.id).map((n) => ({ id: n.id, label: n.label || n.id, kind: kindOf(n) })) });
  else cols.push({ label: "Nodes", nodes: nodes.map((n) => ({ id: n.id, label: n.label || n.id, kind: kindOf(n) })) });
  return [
    toolbar(button("Add a node…", () => addNode(), { testid: "add-node" }), h("span", { class: "tn-dim" }, "Click a node to see it and change it. Dashed: nodes of other groups this group reads.")),
    graph(cols, g.edges.map((e) => ({ from: e.from_node, to: e.to_node })), { onNode: (id) => { if (g.nodes.has(id)) { app.sel = id; app.tab = 0; showGroup(); } }, testid: "map" }),
    app.sel && g.nodes.has(app.sel) ? nodeDetail(app.sel) : null,
  ];
}

function nodeDetail(id) {
  const g = G(), idx = app.ws.index, n = g.nodes.get(id);
  const outR = readersOutside(idx, id), inR = readersInside(idx, id);
  const author = g.memberNodes.get(id);
  const live = n.state !== "archived";
  return section(`Node ${id}`, kv([["Label", n.label || "—"], ["Stage", n.stage || "—"], ["State", n.state], ["Kind · layer", `${n.kind || "—"} · ${n.layer || "—"}`],
    ["Author", author ? `${author.author} (issued ${fmtTime(author.issued_at)})` : "not issued"],
    ["Read in this group by", inR.join(", ") || "—"], ["Read by other groups", [...outR].map(([k, v]) => `${k} (${v.join(", ")})`).join("; ") || "—"],
    ["Reads", g.edges.filter((e) => e.to_node === id).map((e) => e.from_node).join(", ") || "—"]]),
  n.state === "sealed" ? banner("info", "Sealed.", "Its author cannot change it until it is re-issued.", button("Re-issue…", () => reissueNode(id), { testid: "act-reissue" })) : null,
  live ? toolbar(
    button("Comment…", () => commentNode(id), { testid: "act-comment" }),
    button("Rename…", () => renameNode(id), { testid: "act-rename" }),
    g.stages.length ? button("Stage…", () => stageNode(id), { testid: "act-stage" }) : null,
    button("Split…", () => splitNode(id), { testid: "act-split" }),
    button("Merge into…", () => mergeNode(id), { testid: "act-merge" }),
    button("Move to group…", () => moveNode(id), { testid: "act-move" }),
    button("Issue…", () => issueNode(id), { testid: "act-issue" }),
    button("Archive…", () => run({ type: "archive", group: app.gid, id }), { kind: "danger", testid: "act-archive" })) : banner("info", "", "Archived: its file is kept, and nothing reads it."));
}

function nodesView() {
  const g = G();
  return [toolbar(button("Add a node…", () => addNode())),
    table([{ key: "id", label: "Node", mono: true }, { key: "label", label: "Label" }, { key: "stage", label: "Stage" },
      { key: "state", label: "State", render: (r) => badge(r.state, r.state === "archived" ? "" : r.state === "shell" ? "warn" : "ok") },
      { key: "author", label: "Author", render: (r) => g.memberNodes.get(r.id)?.author || "—" }],
    [...g.nodes.values()], { onRow: (r) => { app.sel = r.id; app.tab = 0; showGroup(); }, testid: "nodes" })];
}

function stagesView() {
  const g = G();
  const counts = (sid) => [...g.nodes.values()].filter((n) => n.stage === sid && n.state !== "archived").length;
  return [
    g.stages.length ? table([{ key: "id", label: "Stage", mono: true }, { key: "label", label: "Name" }, { key: "owner", label: "Owner (signs it)", render: (r) => r.owner || "—" },
      { key: "n", label: "Nodes", render: (r) => counts(r.id) }],
    g.stages, { onRow: (r) => stageOwner(r.id), testid: "stages" }) : banner("info", "", `${g.id} has no stages: its nodes are one list. Stages come from design/groups.toml.`),
    g.stages.length ? toolbar(button("Add a stage…", () => addStage(), { testid: "add-stage" }), h("span", { class: "tn-dim" }, "Click a stage to set who owns it.")) : null,
  ];
}

function peopleView() {
  const g = G();
  return [
    section("Members", table([{ key: "name", label: "Name" }, { key: "role", label: "Role" }], g.members, { empty: "Nobody yet.", testid: "members" }),
      toolbar(button("Add or change a member…", () => addMember(), { testid: "add-member" }))),
    section("Who authors what", table([{ key: "node", label: "Node", mono: true }, { key: "author", label: "Author" }, { key: "issued_at", label: "Issued", render: (r) => fmtTime(r.issued_at) }],
      [...g.memberNodes.values()], { empty: "No node file issued yet. Open a node on the map and choose Issue." })),
  ];
}

function contractsView() {
  const g = G(), idx = app.ws.index;
  const boundary = [...g.nodes.keys()].map((id) => ({ id, readers: readersOutside(idx, id) })).filter((x) => x.readers.size)
    .map((x) => ({ id: x.id, readers: [...x.readers.keys()].join(", "), published: g.contracts.filter((c) => c.node === x.id).map((c) => `${c.output} v${c.version}`).join(", ") }));
  return [
    section("Read by other groups", table([{ key: "id", label: "Node", mono: true }, { key: "readers", label: "Read by" },
      { key: "published", label: "Contract", render: (r) => r.published || badge("none yet", "warn") }],
    boundary, { empty: "No other group reads this group's nodes.", onRow: (r) => publishContract(r.id), testid: "boundary" }),
    h("p", { class: "tn-dim" }, "A contract names an output another group reads, its unit and version: the agreement at the boundary. Click a node to publish or change its contract.")),
    section("Contracts", table([{ key: "node", label: "Node", mono: true }, { key: "output", label: "Output", mono: true }, { key: "unit", label: "Unit" },
      { key: "version", label: "Version" }, { key: "readers", label: "Readers" }], g.contracts, { empty: "None published.", testid: "contracts" })),
  ];
}

function requestsView() {
  const g = G(), idx = app.ws.index;
  const incoming = requestsTo(idx, g.id);
  const outgoing = g.crs.filter((c) => c.about.to).map((c) => {
    const T = idx.groups.get(c.about.to), reply = T ? [...T.crs].reverse().find((x) => x.about.reply === c.id) : null;
    return { ...c, to: c.about.to, what: `${c.about.action} ${c.about.node || ""}`, st: c.state === "done" ? "done" : reply ? reply.state : "open" };
  });
  const stBadge = (s) => badge(s, s === "accepted" || s === "done" ? "ok" : s === "declined" ? "error" : "warn");
  return [
    section("To this group", table([{ key: "from", label: "From" }, { key: "what", label: "Asks", render: (r) => `${r.cr.about.action} ${r.cr.about.node || ""}` },
      { key: "body", label: "Why", render: (r) => r.cr.body }, { key: "state", label: "State", render: (r) => stBadge(r.state) },
      { key: "act", label: "", render: (r) => (r.state === "open" ? h("span", {}, button("Accept", () => run({ type: "reply", group: g.id, id: r.cr.id, answer: "accepted" }), { kind: "primary", testid: `accept-${r.cr.id}` }), " ",
        button("Decline", async () => { const why = await ask("Decline", "Why", {}); if (why) run({ type: "reply", group: g.id, id: r.cr.id, answer: "declined", body: why }); })) : "") }],
    incoming, { empty: "None.", testid: "incoming" })),
    section("From this group", table([{ key: "to", label: "To" }, { key: "what", label: "Asks" }, { key: "body", label: "Why" }, { key: "st", label: "State", render: (r) => stBadge(r.st) }],
      outgoing, { empty: "None.", testid: "outgoing" }), toolbar(button("Raise a change request…", () => raiseRequest({}), { testid: "raise" }))),
  ];
}

function historyView() {
  const box = h("div", {}, "Reading…");
  app.ws.history().then((hist) => {
    const mine = hist.filter((x) => x.files.some((f) => f === `${STRUCTURE}/${app.gid}.group.tndb`));
    fill(box, table([{ key: "at", label: "When", render: (r) => fmtTime(r.at) }, { key: "by", label: "Who" }, { key: "summary", label: "What" },
      { key: "files", label: "Files", render: (r) => r.files.length }], mine, { empty: "No structure change yet.", testid: "history" }),
    h("p", { class: "tn-dim" }, "Each change is also a revision in every file it touched, and Drive keeps every saved version."));
  });
  return box;
}

// ------------------------------------------------------------------ progress, assemble, release
// every node file of the group read and judged (release.js assemble), kept until something changes
function assembled(view) {
  const box = h("div", { "data-testid": "assembling" }, "Reading every node file of the group…");
  const gid = app.gid;
  (app.asm && app.asm.gid === gid ? Promise.resolve(app.asm) : app.rel.assemble(gid)).then((asm) => {
    if (app.gid !== gid) return;
    app.asm = asm;
    fill(box, view(asm));
    box.setAttribute("data-testid", "assembled");
  }).catch((e) => fill(box, banner("error", "The group could not be read.", String(e.message || e))));
  return box;
}

const workBadge = (st) => badge(st, st === "checked" ? "ok" : st === "ready" ? "" : "warn");
const sealBadge = (x) => badge(x.confirmed ? "confirmed" : "UNCONFIRMED", x.confirmed ? "ok" : "warn");
const sinceBadge = (s) => badge(s, s === "same" ? "" : "warn");

function progressView(asm) {
  const n = asm.nodes, count = (f) => n.filter(f).length;
  return [
    section("Where the group stands", kv([["Nodes", String(n.length)], ["Checked by a second engineer", String(count((x) => x.standing.state === "checked"))],
      ["Marked ready", String(count((x) => x.standing.state === "ready"))], ["Drafts", String(count((x) => x.standing.state === "draft"))], ["Shells", String(count((x) => x.standing.state === "shell"))],
      ["Would be sealed as confirmed", String(count((x) => x.confirmed))], ["Evidence debt", `${n.reduce((t, x) => t + x.debt.length, 0)} item(s)`],
      ["Since the last release", asm.last ? `${asm.last.version}: ${count((x) => x.since === "new")} new, ${count((x) => x.since === "changed")} changed, ${asm.removed.length} gone` : "no release yet"]])),
    table([{ key: "id", label: "Node", mono: true }, { key: "label", label: "Label", render: (x) => x.row.label || "" }, { key: "stage", label: "Stage", render: (x) => x.row.stage || "" },
      { key: "author", label: "Author", render: (x) => x.author || "—" }, { key: "work", label: "Work", render: (x) => workBadge(x.standing.state) },
      { key: "sig", label: "Signatures", render: (x) => (x.standing.checkedStale || x.standing.readyStale ? badge("stale", "warn") : x.standing.checked ? `checked by ${x.standing.checked.name}` : x.standing.ready ? `ready (${x.standing.ready.name})` : "—") },
      { key: "p", label: "Problems", render: (x) => (x.problems.length ? badge(String(x.problems.length), "warn") : "0") }, { key: "debt", label: "Debt", render: (x) => String(x.debt.length) },
      { key: "since", label: "Since last", render: (x) => sinceBadge(x.since) }],
    n, { onRow: (x) => showNode(x), testid: "progress" }),
  ];
}

function assembleView(asm) {
  const bad = asm.cross.filter((c) => c.level === "!");
  return [
    asm.structure.length || asm.unreadable.length ? banner("error", "The group cannot be sealed as it is.", [...asm.structure, ...asm.unreadable].join("\n")) : null,
    section(`Checks across the nodes (${bad.length} problem(s), ${asm.cross.length - bad.length} note(s))`,
      table([{ key: "node", label: "Node", mono: true }, { key: "level", label: "", render: (c) => badge(c.level === "!" ? "problem" : "note", c.level === "!" ? "error" : "") }, { key: "text", label: "What" }],
        asm.cross, { empty: "Nothing: every input, contract and unit agrees.", testid: "cross" })),
    section("Stages and their signatures", table([{ key: "id", label: "Stage", mono: true }, { key: "owner", label: "Owner", render: (s) => s.owner || "—" }, { key: "nodes", label: "Nodes" },
      { key: "signed", label: "Signed", render: (s) => (s.signed ? badge(`signed by ${s.sig.name}`, "ok") : s.sig ? badge("stale: changed since", "warn") : badge("not signed", "warn")) },
      { key: "act", label: "", render: (s) => button("Sign…", () => run({ type: "signStage", group: asm.gid, stage: s.id }), { testid: `sign-${s.id}` }) }],
    asm.stages, { empty: "This group has no stages: its lead's seal covers it.", testid: "stages-sign" })),
    section("Every node as it would be sealed", h("p", { class: "tn-dim" }, "Click a node to see it as the main application will show it, with what keeps it from being confirmed."),
      table([{ key: "id", label: "Node", mono: true }, { key: "s", label: "Sealed as", render: (x) => sealBadge(x) }, { key: "why", label: "Why not confirmed", render: (x) => x.why.join("; ") }],
        asm.nodes, { onRow: (x) => showNode(x), testid: "assemble" })),
  ];
}

async function showNode(x) {
  const ctx = { groupLabel: `${app.gid} · ${G().label || ""}`, readers: [...readersInside(app.ws.index, x.id).map((id) => ({ id, group: app.gid })), ...[...readersOutside(app.ws.index, x.id)].flatMap(([g, ids]) => ids.map((id) => ({ id, group: g })))],
    pictureUrl: () => "" };
  const v = await dialog(`${x.id} · ${x.row.label || ""}`, h("div", { class: "tn-stack", "data-testid": "node-view" },
    x.confirmed ? banner("ok", "Would be sealed as confirmed.", "") : banner("warn", "Would be sealed UNCONFIRMED.", x.why.join("\n")),
    x.comments.length ? section("Comments", table([{ key: "at", label: "When", render: (c) => fmtTime(c.at) }, { key: "by", label: "Who" }, { key: "body", label: "Comment" }], x.comments)) : null,
    renderNode(x.doc, ctx, x.standing)), [["Close", null], ["Comment…", "comment", "primary"]]);
  if (v === "comment") await commentNode(x.id);
}

function releaseView(asm) {
  const rels = asm.releases;
  return [
    section(`Releases of ${asm.gid}`, table([{ key: "version", label: "Version", mono: true }, { key: "sealed_at", label: "Sealed", render: (r) => fmtTime(r.sealed_at) }, { key: "sealed_by", label: "By" },
      { key: "c", label: "Confirmed", render: (r) => `${r.confirmed} of ${r.nodes.size}` }, { key: "ok", label: "Fingerprints", render: (r) => (r.problems.length ? badge(`${r.problems.length} problem(s)`, "error") : badge("intact", "ok")) },
      { key: "file", label: "File", mono: true }], rels, { empty: "Not sealed yet.", testid: "releases" }),
    toolbar(button(`Seal ${asm.next}…`, () => run({ type: "seal", group: asm.gid, version: asm.next }), { kind: "primary", testid: "seal" }),
      button("Compare…", () => compareDialog(asm), { disabled: !rels.length, testid: "compare" }),
      button("Re-issue a node…", () => reissueNode(null), { testid: "reissue" }),
      fileButton("Import node forms…", (files) => importForms(files), { accept: ".html,text/html", multiple: true, testid: "import" })),
    h("p", { class: "tn-dim" }, `Only the lead (${asm.lead || "none yet: People → a member with the role lead"}) seals. A release is frozen in ${RELEASES}/; every node file is stamped with it and sealed until it is re-issued. A node goes in as confirmed only when someone other than its author checked it, its stage owner signed its stage as it is, the checks find nothing and, for a computing node, a test vector has its answer from outside the code; the rest go in UNCONFIRMED, each with why.`)),
  ];
}

async function compareDialog(asm) {
  const opts = [...asm.releases.map((r) => [r.version, `${asm.gid} ${r.version}`]), ["now", "the group now"]];
  const a = choice("From", opts, { value: asm.releases[asm.releases.length - 1].version, testid: "f-from" });
  const b = choice("To", opts, { value: "now", testid: "f-to" });
  const v = await form("Compare", [a, b], "Compare");
  if (!v) return;
  const pick = (k) => (k === "now" ? nowNodes(asm) : asm.releases.find((r) => r.version === k).nodes);
  const rows = compare(pick(v[0]), pick(v[1])).filter((d) => d.change !== "same");
  await dialog(`${v[0]} → ${v[1]}`, h("div", { "data-testid": "comparison" }, rows.length ? table([{ key: "id", label: "Node", mono: true }, { key: "change", label: "Change", render: (d) => badge(d.change, d.change === "changed" ? "warn" : "") },
    { key: "fields", label: "What changed", render: (d) => d.fields.join(", ") }], rows) : h("p", {}, "No node changed.")), [["Close", null]]);
}

async function reissueNode(id) {
  const asm = app.asm, g = G();
  const node = id ? null : choice("Node", [...g.nodes.values()].filter((n) => n.state !== "archived").map((n) => [n.id, `${n.id} · ${n.label || ""} (${n.state})`]), { testid: "f-node" });
  const from = choice("From", [["", "the file as it is"], ...((asm && asm.gid === g.id ? asm.releases : await app.rel.releases(g.id)).map((r) => [r.version, `as sealed in ${g.id} ${r.version}`]))], { testid: "f-from" });
  const v = await form(`Re-issue ${id || "a node"}`, [...(node ? [node] : []), from]);
  if (!v) return;
  await run({ type: "reissue", group: g.id, id: id || v[0], from: v[node ? 1 : 0] || null });
}

async function commentNode(id) {
  const body = field("Comment", { multiline: true, help: "Its author sees it on the node's Home in the node app.", testid: "f-comment" });
  const v = await form(`Comment on ${id}`, [body], "Next");
  if (v && v[0].trim()) await run({ type: "comment", group: app.gid, id, body: v[0].trim() });
}

async function importForms(files) {
  for (const f of files) {
    let form;
    try { form = parseForm(await f.text()); } catch (e) { toast(`${f.name}: ${e.message}`, "error"); main.prepend(banner("error", `${f.name} was not imported.`, e.message)); continue; }
    const id = (form.node || {}).tree_id;
    if (!id) { toast(`${f.name}: the form names no node`, "error"); continue; }
    await run({ type: "import", group: app.gid, id, name: f.name, form });
  }
}

// ------------------------------------------------------------------ actions
/** Show the impact; do it when nothing stops it. */
async function run(a) {
  const engine = RELEASE_ACTIONS.has(a.type) ? app.rel : app.ws;
  setStatus(`Checking the impact of: ${a.type}…`);
  const p = await guard("Checking the impact", () => engine.plan(a));
  if (!p) return null;
  const acts = [["Cancel", null]];
  const ask4 = p.blocked && requestFor(a, p);
  if (ask4) acts.push(["Raise a change request…", "request"]);
  if (!p.blocked) acts.push(["Do it", "do", "primary"]);
  const v = await dialog(p.blocked ? "This cannot be done yet" : "Impact check", h("div", {}, h("p", {}, h("b", {}, summary(a))), impactList(p.impact)), acts);
  if (v === "request") return raiseRequest(ask4);
  if (v !== "do") return null;
  setStatus(`Doing: ${summary(a)}…`);
  const r = await guard(summary(a), () => engine.apply(a));
  if (!r) return null;
  app.asm = null;
  if (a.type === "move" && a.id === app.sel) app.sel = null;
  app.problems = await integrity(app.SQL, app.root, { index: app.ws.index, scope: [app.gid, ...(a.to ? [a.to] : [])].filter((x) => app.ws.index.groups.has(x)) }).catch((e) => [String(e)]);
  toast(`Done: ${r.summary} (${r.files.length} file(s))`);
  await showGroup();
  app.done = (app.done || 0) + 1;
  return r;
}

// the change request a blocked action needs
function requestFor(a, p) {
  if (a.type === "move") return { to: a.to, action: "move", node: a.id };
  if (a.type === "archive" || a.type === "merge") {
    const id = a.type === "archive" ? a.id : a.gone;
    const [to] = [...readersOutside(app.ws.index, id).keys()].filter((g) => p.impact.some((x) => x.level === "block" && x.text.startsWith(`${g} reads`)));
    return to ? { to, action: a.type, node: id } : null;
  }
  return null;
}

async function form(title, fields, okLabel = "Next") {
  const v = await dialog(title, h("div", { class: "tn-stack" }, fields.map((f) => f.el)), [["Cancel", null], [okLabel, () => fields.map((f) => (f.values || f.value)), "primary"]]);
  return v;
}

async function addNode() {
  const g = G();
  const id = field("Id", { mono: true, help: "lowercase letters, digits and _; it never changes", testid: "f-id" }), label = field("Label", { testid: "f-label" });
  const stage = g.stages.length ? choice("Stage", g.stages.map((s) => [s.id, s.label || s.id]), { testid: "f-stage" }) : null;
  const kind = choice("Kind", [["leaf", "leaf (computed)"], ["kpi", "KPI"], ["closure", "closure"], ["interface", "interface"], ["evidence", "evidence"], ["declared", "declared"]], { testid: "f-kind" });
  const v = await form("Add a node", [id, label, ...(stage ? [stage] : []), kind]);
  if (!v) return;
  await run({ type: "add", group: g.id, id: v[0].trim(), label: v[1].trim(), stage: stage ? v[2] : null, kind: v[stage ? 3 : 2] });
}

async function renameNode(id) {
  const f = field("Label", { value: G().nodes.get(id).label || "", testid: "f-label" });
  const v = await form(`Rename ${id}`, [f]);
  if (v) await run({ type: "rename", group: app.gid, id, label: v[0].trim() });
}

async function stageNode(id) {
  const g = G(), c = choice("Stage", g.stages.map((s) => [s.id, s.label || s.id]), { value: g.nodes.get(id).stage || "", testid: "f-stage" });
  const v = await form(`Stage of ${id}`, [c]);
  if (v) await run({ type: "stage", group: app.gid, id, stage: v[0] });
}

async function splitNode(id) {
  const rows = await guard("Reading the node", () => app.ws.readNode(id));
  if (!rows) return;
  const newId = field("New node's id", { mono: true, testid: "f-id" }), newLabel = field("New node's label", { testid: "f-label" });
  const fields = checks("Fields that go to the new node", rows.content.filter((c) => c.section !== "identity").map((c) => [`${c.section}.${c.field}`, `${c.section} · ${c.field}`]), { testid: "f-fields" });
  const outs = checks("Outputs that go to it", rows.output.map((o) => [o.name, o.name]), { testid: "f-outputs" });
  const readers = checks(`Readers in ${app.gid} that read the new node instead`, readersInside(app.ws.index, id).map((r) => [r, r]), { testid: "f-readers" });
  const v = await form(`Split ${id}`, [newId, newLabel, fields, outs, readers]);
  if (v) await run({ type: "split", group: app.gid, id, newId: v[0].trim(), newLabel: v[1].trim(), fields: v[2], outputs: v[3], readers: v[4] });
}

async function mergeNode(id) {
  const g = G(), others = [...g.nodes.values()].filter((n) => n.id !== id && n.state !== "archived");
  const c = choice(`Merge ${id} into`, others.map((n) => [n.id, `${n.id} · ${n.label || ""}`]), { testid: "f-keep" });
  const v = await form(`Merge ${id}`, [c]);
  if (v) await run({ type: "merge", group: app.gid, keep: v[0], gone: id });
}

async function moveNode(id) {
  const idx = app.ws.index, gs = [...idx.groups.values()].filter((x) => x.id !== app.gid);
  const to = choice("To group", gs.map((x) => [x.id, `${x.id} · ${x.label || ""}`]), { testid: "f-to" });
  const v = await form(`Move ${id} to another group`, [to]);
  if (!v) return;
  const T = idx.groups.get(v[0]);
  let stage = null;
  if (T.stages.length) {
    const st = choice(`Stage in ${T.id}`, T.stages.map((s) => [s.id, s.label || s.id]), { testid: "f-stage" });
    const w = await form(`Stage in ${T.id}`, [st]);
    if (!w) return;
    stage = w[0];
  }
  await run({ type: "move", group: app.gid, id, to: T.id, stage });
}

async function issueNode(id) {
  const g = G();
  if (!g.members.length) { toast("Add the author as a member first (People)", "error"); return; }
  const c = choice("Author", g.members.map((m) => [m.name, `${m.name} (${m.role})`]), { value: g.memberNodes.get(id)?.author || "", testid: "f-author" });
  const v = await form(`Issue ${id}`, [c]);
  if (v) await run({ type: "issue", group: app.gid, id, author: v[0] });
}

async function addStage() {
  const id = field("Id", { mono: true, testid: "f-id" }), label = field("Name", { testid: "f-label" });
  const v = await form("Add a stage", [id, label]);
  if (v) await run({ type: "addStage", group: app.gid, stage: v[0].trim(), label: v[1].trim() });
}

async function stageOwner(sid) {
  const g = G();
  const c = choice("Owner (signs the stage)", [["", "nobody"], ...g.members.map((m) => [m.name, m.name])], { value: g.stages.find((s) => s.id === sid).owner || "", testid: "f-owner" });
  const v = await form(`Owner of stage ${sid}`, [c]);
  if (v) await run({ type: "stageOwner", group: app.gid, stage: sid, owner: v[0] || null });
}

async function addMember() {
  const name = field("Name", { testid: "f-name" }), role = choice("Role", [["author", "author"], ["stage owner", "stage owner"], ["lead", "lead"]], { testid: "f-role" });
  const v = await form("A member of the group", [name, role]);
  if (v) await run({ type: "member", group: app.gid, name: v[0].trim(), role: v[1] });
}

async function publishContract(id) {
  const out = field("Output", { mono: true, value: "*", help: "the output the other groups read (* for the node's whole answer)", testid: "f-output" });
  const unit = field("Unit", { mono: true, testid: "f-unit" });
  const v = await form(`Contract of ${id}`, [out, unit]);
  if (v) await run({ type: "contract", group: app.gid, node: id, output: v[0].trim(), unit: v[1].trim() || null });
}

async function raiseRequest(pre) {
  const idx = app.ws.index;
  const to = choice("To group", [...idx.groups.values()].filter((x) => x.id !== app.gid).map((x) => [x.id, `${x.id} · ${x.label || ""}`]), { value: pre.to || "", testid: "f-to" });
  const act = choice("Asking to", [["move", "take a node into their group"], ["archive", "stop reading a node (it is archived)"], ["merge", "read another node instead (it is merged)"], ["contract", "change a contract"], ["other", "something else"]], { value: pre.action || "other", testid: "f-action" });
  const node = field("Node", { mono: true, value: pre.node || "", testid: "f-node" }), body = field("Why", { multiline: true, testid: "f-body" });
  const v = await form("Raise a change request", [to, act, node, body], "Raise it");
  if (v) await run({ type: "request", group: app.gid, to: v[0], action: v[1], node: v[2].trim() || null, body: v[3].trim() });
}

// ------------------------------------------------------------------ start
function start() {
  render(toolbar(...topButtons()),
    canFolders ? banner("info", "Start here.", "Open the design folder: the one holding structure/ (the group files) and nodes/ (the node files), on your Drive. Then pick your group.")
      : banner("warn", "This browser cannot open folders.", "The group app changes several files at once, so it needs folder access: use Chrome or Edge."));
  setStatus("Ready");
}

async function profileId() {
  let p = await journal.pref("profile").catch(() => null);
  if (!p) { p = crypto.getRandomValues(new Uint32Array(4)).join("-"); await journal.setPref("profile", p).catch(() => {}); }
  return p;
}

const wasm = Uint8Array.from(atob(document.getElementById("tn-sqlite-wasm").textContent.trim()), (c) => c.charCodeAt(0));
app.ready = Promise.all([window.initSqlJs({ wasmBinary: wasm }), profileId()]).then(([SQL, profile]) => { app.SQL = SQL; app.profile = profile; start(); });

// for the browser tests (tests/browser/group.test.mjs) and for anyone automating the page
window.tnGroup = { app, ready: app.ready, useFolder, openGroup, run, showGroups, importForms, select: async (id) => { app.sel = id; app.tab = 0; await showGroup(); } };
