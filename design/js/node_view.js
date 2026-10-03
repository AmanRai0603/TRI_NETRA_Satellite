// node_view.js -- a node as the main application will show it (docs/RELEASE_PLAN.md P5 "Preview";
// the same drawing serves the main app's node pages in P11). Built from the node file alone, with
// what the design folder adds when it is there (who reads it, its group, its contracts).
// Answer first (SPEC §5.12): the opening sentence says what the node is, its kind, state and
// version, and what it still owes; then where it sits; then its own explanation; then the rest.
// Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
import { h, section, table, badge, kv } from "./tn_ui.js";
import { KIND_LABEL, FIXED, list, results, evidenceDebt, tryIt } from "./node_model.js";
import { CATALOG } from "./node_catalog.js";

const UNIT_SYM = new Map(CATALOG.units.map(([n, sym]) => [n, sym]));
const SOURCE_TITLE = new Map(CATALOG.sources.map(([i, t]) => [i, t]));
const unitText = (u) => (u ? UNIT_SYM.get(u) || u : "");

/** The expression as it reads: x^2 as a superscript, x_i as a subscript, Greek names as letters. */
export function mathText(expr) {
  const GREEK = { alpha: "α", beta: "β", gamma: "γ", delta: "δ", epsilon: "ε", theta: "θ", lambda: "λ", mu: "μ", pi: "π", rho: "ρ", sigma: "σ", tau: "τ", phi: "φ", omega: "ω", Delta: "Δ", Omega: "Ω" };
  const out = h("span", { class: "tn-math" });
  const re = /([A-Za-z][A-Za-z0-9]*)(_\{?([A-Za-z0-9]+)\}?)?|\^\{?(-?[A-Za-z0-9.]+)\}?|\*|sqrt|[^A-Za-z^*]+/g;
  let m;
  const s = String(expr || "");
  while ((m = re.exec(s))) {
    if (m[4] !== undefined) out.append(h("sup", {}, m[4]));
    else if (m[0] === "*") out.append("·");
    else if (m[1] !== undefined) {
      out.append(GREEK[m[1]] || m[1]);
      if (m[3]) out.append(h("sub", {}, GREEK[m[3]] || m[3]));
    } else out.append(m[0]);
  }
  return out;
}

/** The whole node, previewed. ctx: { group, readers: [{ id, group }], reads: [{ id, label }], contract } */
export function renderNode(doc, ctx = {}, standingInfo = null) {
  const c = doc.content, n = doc.node, k = doc.kind;
  const rev = doc.revisions.length ? `version ${doc.revisions.length}` : "never saved from the apps";
  const debt = evidenceDebt(doc);
  const state = standingInfo ? standingInfo.state : "draft";
  const answer = c["output.symbol"] ? `${c["output.symbol"]}${c["output.unit"] ? ` [${unitText(c["output.unit"])}]` : ""}` : "";
  const parts = [];
  parts.push(h("p", { class: "tn-answer-first", "data-testid": "answer-first" },
    h("b", {}, n.label || n.id), ` is a ${k ? KIND_LABEL[k].split(" (")[0] : "not yet chosen"} node of ${ctx.groupLabel || n.group_id || "its group"}`,
    answer ? `, answering ${answer}` : "", `; ${state}, ${rev}. `,
    FIXED.has(k) ? "Its shape is fixed by the tree." : debt.length ? `It still owes ${debt.length} thing(s): ${debt.map((d) => d.text.split(":")[0]).join("; ")}.` : "It owes nothing: every test it carries is answered from outside the code."));
  if (c["identity.question"]) parts.push(h("p", { class: "tn-question" }, h("b", {}, "The question: "), c["identity.question"]));
  // where it sits
  parts.push(section("Where this node sits", h("div", { class: "tn-sits" },
    h("div", {}, h("div", { class: "tn-dim" }, "It reads"), (doc.inputs.length ? doc.inputs.map((i) => h("div", { class: "tn-mono" }, `${i.from_node} → ${i.name}`)) : h("div", { class: "tn-dim" }, "nothing"))),
    h("div", { class: "tn-sits-me" }, h("b", {}, n.id), h("div", {}, n.label || "")),
    h("div", {}, h("div", { class: "tn-dim" }, "Read by"), ((ctx.readers || []).length ? ctx.readers.map((r) => h("div", { class: "tn-mono" }, `${r.id}${r.group && r.group !== n.group_id ? ` (${r.group})` : ""}`)) : h("div", { class: "tn-dim" }, ctx.readers ? "nobody yet" : "open the design folder to see"))))));
  // its own explanation
  const ex = [["Say it simply", "explain.simply"], ["In one line", "explain.one_line"], ["The real thing", "explain.real_thing"], ["Where it breaks", "explain.where_it_breaks"], ["Try it", "explain.try_it"]]
    .filter(([, key]) => c[key]).map(([t, key]) => h("div", { class: "tn-station" }, h("h3", {}, t), h("p", {}, c[key])));
  if (c["explain.wrong_idea"]) ex.push(h("div", { class: "tn-station" }, h("h3", {}, "Common wrong idea"), h("p", {}, c["explain.wrong_idea"]), c["explain.wrong_because"] ? h("p", {}, h("b", {}, "Why it is wrong: "), c["explain.wrong_because"]) : null));
  if (c["explain.contrast"]) ex.push(h("div", { class: "tn-station" }, h("h3", {}, "Two cases, one difference"), h("p", {}, c["explain.contrast"])));
  if (c["explain.analogy"]) ex.push(h("div", { class: "tn-station" }, h("h3", {}, "An analogy"), h("p", {}, c["explain.analogy"]), c["explain.analogy_breaks"] ? h("p", {}, h("b", {}, "Where it breaks: "), c["explain.analogy_breaks"]) : null));
  const chain = list(doc, "explain.why_chain");
  if (chain.length) ex.push(h("div", { class: "tn-station" }, h("h3", {}, "Why, and why"), h("ol", {}, chain.map((x) => h("li", {}, x)))));
  if (ex.length) parts.push(section("Explanation", ...ex));
  // the relation
  if (k === "computed" && (c["relation.expression"] || c["code.pseudocode"])) {
    const deriv = list(doc, "relation.derivation");
    parts.push(section("The relation",
      c["relation.expression"] ? h("p", { class: "tn-equation", "data-testid": "equation" }, mathText(c["relation.expression"])) : null,
      kv([["Source", sourceText(c["relation.source"])], ["Why this relation", c["relation.why"] || "—"], ["How to read the answer", c["relation.how_to_read"] || "—"]]),
      deriv.length ? h("div", {}, h("h3", {}, "Derivation"), h("ol", {}, deriv.map((x) => h("li", {}, x)))) : null,
      c["code.pseudocode"] ? h("div", {}, h("h3", {}, "Pseudocode"), h("pre", { class: "tn-code" }, c["code.pseudocode"])) : null));
  }
  if (doc.inputs.length) parts.push(section("What it reads", table([{ key: "name", label: "Binding", mono: true }, { key: "from_node", label: "From node", mono: true }, { key: "quantity", label: "Quantity" }], doc.inputs)));
  // the answer
  if (!FIXED.has(k) && c["output.symbol"]) {
    parts.push(section("Its answer", kv([["Symbol", h("span", { class: "tn-mono" }, c["output.symbol"])], ["Quantity", c["output.quantity"] || "—"], ["Unit", c["output.unit"] ? `${c["output.unit"]} (${unitText(c["output.unit"])})` : "—"],
      ["Lowest", c["output.lower"] !== undefined ? `${c["output.lower"]} — ${c["output.reason_lower"] || "no reason given"}` : "—"],
      ["Highest", c["output.upper"] !== undefined ? `${c["output.upper"]} — ${c["output.reason_upper"] || "no reason given"}` : "—"]])));
  }
  if (k === "declared" && c["value.number"]) parts.push(section("Its value", kv([["Value", h("b", { "data-testid": "value" }, `${c["value.number"]} ${unitText(c["output.unit"])}`)], ["Source", sourceText(c["value.source"])]])));
  if (k === "kpi") parts.push(section("The requirement", kv([["Binds", { at_most: "at most (the achieved value must not exceed it)", at_least: "at least (the achieved value must reach it)" }[c["requirement.sense"]] || "—"],
    ["Value", c["requirement.value"] ? `${c["requirement.value"]} ${unitText(c["output.unit"])}` : "set by the case"]])));
  if (k === "evidence") parts.push(section("The evidence", kv([["Metric", c["evidence.metric"] || "—"], ["Rungs", list(doc, "evidence.rungs").join(", ").toUpperCase() || "—"]])));
  const asm = list(doc, "assumptions");
  if (asm.length) parts.push(section("What has to be true", table([{ key: "assumes", label: "Assumed" }, { key: "until", label: "Stops being true when" }], asm)));
  // tests and results
  if (doc.fixtures.length) {
    const t = k === "computed" && c["code.pseudocode"] ? tryIt(doc) : null;
    parts.push(section("Test vectors", table([{ key: "name", label: "Vector", mono: true },
      { key: "inputs", label: "Inputs", mono: true, render: (r) => Object.entries(r.inputs).map(([a, b]) => `${a}=${b}`).join(", ") },
      { key: "expected", label: "Expected", mono: true }, { key: "tolerance", label: "Tolerance" },
      { key: "provenance", label: "From", render: (r) => `${r.provenance}${r.outside ? " (outside the code)" : ""}` },
      { key: "source", label: "Source", render: (r) => `${r.source} ${r.where}` },
      { key: "pass", label: "Pseudocode", render: (r) => { const run = t && t.runs.find((x) => x.name === r.name); return run ? badge(run.pass ? "reproduces it" : "does not", run.pass ? "ok" : "error") : "—"; } }], doc.fixtures)));
  }
  const r = results(doc);
  if (r && r.rows.length) parts.push(section("Results", table(r.columns.map((col, j) => ({ key: String(j), label: col.unit ? `${col.name} [${col.unit}]` : col.name, mono: true, render: (row) => row[j] })), r.rows),
    c["results.from"] ? h("p", { class: "tn-dim" }, `From: ${c["results.from"]}`) : null));
  // pictures
  if (doc.attachments.length && ctx.pictureUrl) {
    parts.push(section("Pictures", h("div", { class: "tn-pictures" }, doc.attachments.map((a) => {
      let meta = {}; try { meta = JSON.parse(c[`pictures.${a.name}`] || "{}"); } catch (e) { meta = {}; }
      return h("figure", { class: "tn-figure" }, a.mime && a.mime.startsWith("image/") ? h("img", { src: ctx.pictureUrl(a.name), alt: meta.alt || a.name }) : h("div", { class: "tn-dim" }, `${a.name} (${a.mime})`),
        h("figcaption", {}, meta.caption || a.name));
    }))));
  }
  // sources, code, belief, versions
  const cited = [...new Set([...list(doc, "sources.cited")])];
  const added = list(doc, "sources.new");
  if (cited.length || added.length) parts.push(section("Sources", h("ul", {}, cited.map((s) => h("li", {}, sourceText(s))), added.map((s) => h("li", {}, `${s.id}: ${s.title}, ${s.where}`)))));
  if (k === "computed" && (c["code.c"] || c["code.rust"] || c["code.twin"])) parts.push(section("Where it is computed", kv([["C flight software", c["code.c"] || "—"], ["Rust", c["code.rust"] || "—"], ["MATLAB twin", c["code.twin"] || "—"]])));
  if (c["belief.believed"]) parts.push(section("Belief record", kv([["Area", c["belief.area"] || "—"], ["Believed", c["belief.believed"]], ["Status", c["belief.status"] || "—"], ["Tested by", c["belief.tested"] || "—"], ["Now we know", c["belief.now_know"] || "—"], ["Changed in the plan", c["belief.plan_change"] || "—"]])));
  if (FIXED.has(k) && (c["feedback.what_happened"] || c["other.subject"])) parts.push(section("Feedback and requests", kv([["What happened", c["feedback.what_happened"] || "—"], ["Expected", c["feedback.expected"] || "—"], ["Something else", c["other.subject"] ? `${c["other.subject"]}: ${c["other.description"] || ""}` : "—"]])));
  if (debt.length) parts.push(section("Where it breaks: what it still owes", h("ul", { "data-testid": "debt" }, debt.map((d) => h("li", {}, d.text)))));
  parts.push(section("Versions", table([{ key: "n", label: "#" }, { key: "at", label: "When", render: (x) => new Date(x.at).toLocaleString() }, { key: "by", label: "Who" }, { key: "summary", label: "What" }],
    [...doc.revisions].reverse(), { empty: "Never saved from the apps." })));
  return h("article", { class: "tn-node-view", "data-testid": "preview" }, parts);
}

function sourceText(id) {
  if (!id) return "—";
  return String(id).split(/[,;]\s*/).map((s) => (SOURCE_TITLE.has(s) ? `${s} — ${SOURCE_TITLE.get(s)}` : s)).join("; ");
}
