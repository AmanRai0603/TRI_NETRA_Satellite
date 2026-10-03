// node_model.js -- a node file as its author fills it (docs/RELEASE_PLAN.md P5): the kinds, the
// steps each kind has, every field with its question, why it matters and an example, how a node
// file holds them, the checks the spec's checker (spec/tools/intake.py, SPEC §5.11) will apply,
// written here so the author sees them while typing, the evidence debt, and "try it": the node's
// pseudocode run on its own test vectors.
//
// Where things live in a node file (design/schema.toml):
//   content(section, field, value, origin)   every field; a list is JSON
//   input(name, from_node, from_output, unit) what a computed node reads
//   output(name, unit, lower, upper, …)       its answer and the bounds, with their reasons
//   fixture(name, inputs, expected, …)        test vectors; outside = 1 when the answer comes from
//                                             outside the code (a source, an independent tool)
//   attachment(name, mime, size, bytes)       pictures and documents
//   signature(role, name, at, statement)      "ready" by the author, "checked by" an engineer
//   comment, change_request                   from and to the group lead
// The node row (group, stage, label, state) is the group's: the node app never changes it.
//
// Nothing here touches the page: design/js/node_app.js shows it, design/js/node_view.js previews it.
// Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
import { CATALOG } from "./node_catalog.js";
import { compile, makeInterpreter } from "./pcode.js";

// ------------------------------------------------------------------ kinds

export const KIND_LABEL = {
  declared: "declared (a value someone states, with its source)",
  computed: "computed (a relation from other nodes)",
  kpi: "KPI requirement (what the mission requires, and which way it binds)",
  evidence: "evidence (what a test shows: a metric on the verification rungs)",
  closure: "closure (fixed by the tree: requirement against achievement)",
  interface: "interface (fixed by the tree: where groups or layers meet)",
  target: "target row (fixed by the tree: a requirement or an achievement of a subsystem)",
};
export const FIXED = new Set(["closure", "interface", "target"]);
export const CHOOSABLE = ["declared", "computed"];

/** The kind a node is filled as: the tree fixes doors, interfaces, closures and target rows; the
 *  spec's catalogue gives the rest; a row the spec has not named yet is chosen by its author. */
export function kindOf(node, content = {}) {
  const k = node.kind || "";
  if (k === "interface" || k === "closure_interface") return "interface";
  if (k.startsWith("closure")) return "closure";
  if (k === "required" || k === "achieved") return "target";
  const spec = CATALOG.rows[node.id];
  if (spec) return { declared: "declared", computed: "computed", required: "kpi", achieved: "evidence", door: "interface" }[spec[1]] || "declared";
  const chosen = content["identity.form_kind"];
  return CHOOSABLE.includes(chosen) ? chosen : null;
}

// ------------------------------------------------------------------ steps and fields

// f(key, label, ask, why, example, opts): a field. type: text, long, number, choice, list, multi
const f = (key, label, ask, why, example, o = {}) => ({ key, label, ask, why, example, type: "text", ...o });
const ALL = ["declared", "computed", "kpi", "evidence"];

export const STEPS = [
  { id: "identity", title: "Identity", kinds: [...ALL, ...FIXED], fields: [
    f("identity.form_kind", "What kind of node it is", "Is its answer a value someone states, or a relation computed from other nodes?", "The kind decides which steps the node has.", "computed", { type: "choice", choices: CHOOSABLE, only: "unnamed" }),
    f("identity.question", "The question it answers", "What question does this node answer, in one sentence?", "A node that answers no clear question cannot be checked.", "How much magnetic dipole can each coil give?", { need: ALL }),
    f("identity.note", "What it is not for", "What might someone wrongly use this answer for?", "Saying what a node is not stops it being misread.", "Not the dipole needed: that is gm_1."),
    f("identity.tags", "Hardware it depends on", "Which actuator families make this node exist (mtq, rw, fmr, rcs)?", "A node for hardware the case does not fit answers zero (I03).", "mtq", { type: "multi", choices: CATALOG.tags }),
  ] },
  { id: "explain", title: "Explanation", kinds: [...ALL, ...FIXED], fields: [
    f("explain.simply", "Say it simply", "Explain the node to a new team member in plain words.", "The explanation standard: answer first, in plain words.", "Each coil can push on Earth's field only so hard; this is how hard.", { type: "long", need: ["computed"] }),
    f("explain.one_line", "In one line", "The node in one line.", "Shown wherever the node is listed (X03).", "The largest dipole one coil makes.", { need: ["computed"] }),
    f("explain.real_thing", "The real thing", "Now say it exactly, as an expert would.", "The step from the simple picture to the precise one.", "m_max = N I A for a coil of N turns, current I, area A.", { type: "long" }),
    f("explain.where_it_breaks", "Where it breaks", "When does this answer stop being true?", "Every explanation says where it breaks (E20).", "Above the core's saturation the dipole no longer grows with current.", { type: "long", need: ALL }),
    f("explain.try_it", "Try it", "A small exercise the reader can do to check they understood.", "An explanation you can test is one you can trust.", "Double the turns: what happens to the dipole?", { type: "long" }),
    f("explain.wrong_idea", "A common wrong idea", "What do people often get wrong about it?", "Naming the wrong idea is how it stops spreading.", "More dipole is always better."),
    f("explain.wrong_because", "Why it is wrong", "Why is that idea wrong (X02)?", "A wrong idea without the reason teaches nothing.", "Dipole costs power and leaves a residual field the magnetometer sees."),
    f("explain.contrast", "Two cases, one difference", "Two cases that differ in one thing, and what changes.", "Contrast is how the difference is seen.", "Same coil at 400 and 600 km: same dipole, a third less torque."),
    f("explain.analogy", "An analogy", "Something familiar that behaves the same way.", "An analogy carries the idea across.", "Like a sail: the wind is the field, the sail is the coil."),
    f("explain.analogy_breaks", "Where the analogy breaks", "Where does the analogy stop being true (X01)?", "An analogy without its limit misleads.", "A sail can be trimmed to any angle; a coil's torque is always across the field."),
    f("explain.why_chain", "Why, and why, and why", "Why is it so? Keep asking until you reach a law, a standard or a decision.", "The chain shows what the node rests on.", "Torque = m × B, because a dipole in a field feels a torque (Maxwell).", { type: "list" }),
  ] },
  { id: "theory", title: "Theory and equations", kinds: ["computed", "declared"], fields: [
    f("relation.expression", "The relation", "The relation as an equation, in the node's symbols.", "The relation is what the code computes (R01).", "m_max = n_turns * i_max * area", { equation: true, need: ["computed"], only: "computed" }),
    f("relation.source", "Its source", "Where does the relation come from (a source id from the list, or a new one under Evidence)?", "Every relation cites where it comes from (S01).", "wertz1978", { need: ["computed"], only: "computed", sources: true }),
    f("relation.why", "Why this relation", "Why is this the right relation here, rather than another?", "The reason is what a reviewer checks (R01).", "Air-core coils at low current: linear in current.", { type: "long", need: ["computed"], only: "computed" }),
    f("relation.how_to_read", "How to read the answer", "What does a big or a small answer mean?", "So the answer can be used without the derivation.", "Above 0.2 A·m² the coil can reject the expected disturbances.", { type: "long", only: "computed" }),
    f("relation.derivation", "The derivation, step by step", "Each step from the law to the relation (R07).", "A derivation a reader can follow is one they can check.", "Ampère's law gives B inside the coil", { type: "list", only: "computed" }),
    f("assumptions", "What has to be true", "Each assumption: what is assumed, and when it stops being true (R06).", "An assumption without its limit is a hidden risk.", "The core is not saturated | until the current passes 0.3 A", { type: "pairs", pair: ["assumes", "until"] }),
  ] },
  { id: "io", title: "Inputs and output", kinds: ALL, fields: [
    f("inputs", "What it reads", "Each input: the name it has in the relation (binding), the node it reads, the quantity it must be.", "An input is a promise between two nodes (C01–C03).", "n_turns | gm_turns | Count", { type: "inputs", only: "computed" }),
    f("output.symbol", "Its symbol", "The answer's symbol: letters, digits and _ (O01).", "The relation and the code bind this name.", "m_max", { need: ALL }),
    f("output.quantity", "Its quantity", "What kind of thing the answer is (O02).", "A unit is a type: an angle is not a ratio.", "DipoleMoment", { type: "choice", choices: CATALOG.quantities, need: ALL }),
    f("output.unit", "Its unit", "The unit, which must state the quantity (O03).", "The unit decides how every number is read.", "AmpereSquareMetre", { type: "choice", choices: CATALOG.units.map((u) => u[0]), need: ALL }),
    f("output.lower", "Lowest value", "The lowest value that makes sense (O04).", "Bounds catch a wrong number before it flies.", "0", { type: "number" }),
    f("output.reason_lower", "Why that lowest", "Why that lowest value (O04)?", "A bound without its reason cannot be revisited.", "a dipole is a magnitude"),
    f("output.upper", "Highest value", "The highest value that makes sense (O04).", "Bounds catch a wrong number before it flies.", "2", { type: "number" }),
    f("output.reason_upper", "Why that highest", "Why that highest value (O04)?", "A bound without its reason cannot be revisited.", "above 2 A·m² no 3U coil fits the volume"),
  ] },
  { id: "value", title: "Value, requirement or evidence", kinds: ["declared", "kpi", "evidence"], fields: [
    f("value.number", "The value", "The value, in the answer's unit (V01).", "A declared node answers this number.", "0.2", { type: "number", only: "declared", need: ["declared"] }),
    f("value.source", "Where the value comes from", "The source of the value: a datasheet, a measurement, a decision (V01, S01).", "A number without its source cannot be trusted.", "cubespace_mtq_ds", { only: "declared", need: ["declared"], sources: true }),
    f("requirement.sense", "Which way it binds", "Must the achieved value be at most or at least this (V02)?", "A requirement without its sense cannot be closed.", "at_most", { type: "choice", choices: ["at_most", "at_least"], only: "kpi", need: ["kpi"] }),
    f("requirement.value", "The required value", "The value required, in the answer's unit, if the case does not set it.", "Blank means the case decides.", "0.1", { type: "number", only: "kpi" }),
    f("evidence.metric", "The metric", "Which metric of the simulator shows it (V03)?", "Evidence is a number a run produces.", "ape", { type: "choice", choices: CATALOG.metrics, only: "evidence", need: ["evidence"] }),
    f("evidence.rungs", "The rungs", "On which rungs is it shown: SILS, PIL, OILS, HILS (V03)?", "Each rung is a step closer to flight.", "sils", { type: "multi", choices: CATALOG.rungs, only: "evidence", need: ["evidence"] }),
  ] },
  { id: "pseudocode", title: "Pseudocode", kinds: ["computed"], fields: [
    f("code.pseudocode", "The pseudocode", "The relation in pseudocode v2: one fn whose output is the answer's symbol (docs/PSEUDOCODE_V2.md).", "The code of every group is generated from this (P10).", "fn dipole(n_turns: real[1], i_max: real[A], area: real[m^2]) -> m_max: real[A m^2]\n    m_max = n_turns*i_max*area\nend", { type: "code", need: ["computed"] }),
  ] },
  { id: "results", title: "Results", kinds: ["computed", "evidence", "declared"], fields: [
    f("results.table", "Results", "Paste results from Excel or any table: the first row names the columns, an optional second row gives units.", "Results are what the node is checked against.", "", { type: "results" }),
    f("results.from", "Where the results come from", "Which run, tool or test made them?", "Results without their origin are hearsay.", "SILS campaign mtq_detumble, 2026-09-30"),
  ] },
  { id: "evidence", title: "Evidence", kinds: ALL, fields: [
    f("fixtures", "Test vectors", "Inputs, the expected answer, the tolerance, and where the answer comes from (T01–T03).", "An answer from outside the code is what makes a node confirmed.", "", { type: "fixtures", only: "computed" }),
    f("sources.cited", "Sources", "The sources the node rests on (ids from the list).", "Every claim traces to a source (S01).", "wertz1978", { type: "multi", choices: CATALOG.sources.map((s) => s[0]) }),
    f("sources.new", "New sources", "A source not in the list: id | title | exactly where (book page, table, figure).", "A source the checker does not know needs its id, title and place (S01).", "smith2024 | Coil design | p. 41, table 3", { type: "pairs", pair: ["id", "title", "where"] }),
  ] },
  { id: "pictures", title: "Pictures", kinds: [...ALL, ...FIXED], fields: [
    f("pictures", "Pictures and documents", "Figures, plots, a page of a source: up to 500 KB each; a bigger picture is made smaller.", "A picture often explains what a paragraph cannot.", "", { type: "pictures" }),
  ] },
  { id: "code", title: "Code", kinds: ["computed"], fields: [
    f("code.c", "C flight software", "Where it is computed in the C flight software, if it is.", "Each relation has one home in each implementation.", "fsw/src/adcs_ctl.c: adcs_sat_dipole"),
    f("code.rust", "Rust", "Where it is computed in Rust (engine or flight software).", "So the code can be held to the pseudocode.", "engine/crates/adcs-physics: mtq::dipole"),
    f("code.twin", "MATLAB twin", "Where it is computed in the MATLAB twin.", "The twin is checked against the engine.", "+asils/+physics/+mtq/dipole.m"),
  ] },
  { id: "belief", title: "Belief record", kinds: ALL, fields: [
    f("belief.area", "Area", "Which of the seven areas does the belief behind this node sit in (D01)?", "The risk register is kept by area.", "model", { type: "choice", choices: CATALOG.areas, need: ALL }),
    f("belief.believed", "What was believed", "What did we believe before this version (D01)?", "A version is worth what it changes in what we know.", "A 0.2 A·m² coil rejects every disturbance at 500 km.", { type: "long", need: ALL }),
    f("belief.status", "Did a test break it, hold it, or has none been run?", "broke, held or untested (D03).", "Untested beliefs are counted as risk (D07).", "held", { type: "choice", choices: CATALOG.belief_statuses, need: ALL }),
    f("belief.tested", "What tested it", "The test that broke or held it, or the test that would settle it (D03).", "A belief is as good as its test.", "SILS detumble campaign at 500 km, 200 runs"),
    f("belief.now_know", "What we now know", "What do we know now (D01)?", "The point of the version.", "It rejects them with a 40 % margin.", { type: "long", need: ALL }),
    f("belief.plan_change", "What changed in the plan", "What does this change in the plan (D01)?", "So the plan follows what was learned.", "Coil mass can drop by 10 %.", { need: ALL }),
    f("belief.cost_k", "What the test cost", "The test's cost in thousands of dollars (D06).", "So testing can be budgeted.", "0.5", { type: "number" }),
  ] },
  { id: "feedback", title: "Feedback", kinds: [...FIXED], fields: [
    f("feedback.what_happened", "What happened", "If something in a release behaved unexpectedly: what happened (B01)?", "Feedback is how fixed rows still get better.", "The closure showed NotStated for a case that states the requirement.", { type: "long" }),
    f("feedback.expected", "What was expected", "What did you expect instead?", "The difference is the bug report.", "Pass, with a 12 % margin.", { type: "long" }),
    f("other.subject", "Something else: subject", "Anything else the developer team should change (B02).", "Fixed rows change through the developers.", "Add a closure for slew settling"),
    f("other.description", "Something else: description", "Describe it (B02).", "", "", { type: "long" }),
  ] },
];

export function stepsFor(kind) { return STEPS.filter((s) => (kind ? s.kinds.includes(kind) : s.id === "identity")); }

export function fieldShown(fd, kind, node) {
  if (fd.only === "unnamed") return node && !CATALOG.rows[node.id] && !FIXED.has(kind) && ["leaf", "internal", ""].includes(node.kind || "");
  if (fd.only && fd.only !== kind) return false;
  if (FIXED.has(kind) && ["identity.question", "identity.note", "identity.tags"].includes(fd.key)) return false;
  return true;
}

// ------------------------------------------------------------------ reading and writing

const rows = (s, sql, p = []) => s.query(sql, p);

/** Everything the app needs from the open file (a FileSession), as one object. */
export function readDoc(s) {
  const node = Object.fromEntries(["id", "sheet", "group_id", "stage", "layer", "kind", "label", "state", "author", "contract_version"]
    .map((k, i) => [k, (rows(s, "SELECT id, sheet, group_id, stage, layer, kind, label, state, author, contract_version FROM node")[0] || [])[i] ?? null]));
  const content = {};
  const origin = {};
  for (const [sec, fld, val, org] of rows(s, "SELECT section, field, value, origin FROM content ORDER BY rowid")) { content[`${sec}.${fld}`] = val; origin[`${sec}.${fld}`] = org; }
  const doc = {
    node, content, origin,
    inputs: rows(s, "SELECT name, from_node, from_output, unit FROM input ORDER BY rowid").map(([name, from_node, quantity, unit]) => ({ name, from_node, quantity, unit })),
    output: (rows(s, "SELECT name, unit, lower, upper, reason_lower, reason_upper FROM output ORDER BY rowid")[0] || null),
    fixtures: rows(s, "SELECT name, inputs, expected, tolerance, source, outside FROM fixture ORDER BY rowid").map(([name, inputs, expected, tolerance, source, outside]) => {
      let meta = {}; try { meta = JSON.parse(source || "{}"); } catch (e) { meta = { source }; }
      let ins = {}; try { ins = JSON.parse(inputs || "{}"); } catch (e) { ins = {}; }
      return { name, inputs: ins, expected: expected === null || expected === "" ? null : Number(expected), tolerance, provenance: meta.provenance || "", source: meta.source || "", where: meta.where || "", outside: !!outside };
    }),
    attachments: rows(s, "SELECT name, mime, length(bytes) FROM attachment ORDER BY name").map(([name, mime, size]) => ({ name, mime, size })),
    signatures: rows(s, "SELECT role, name, at, statement FROM signature ORDER BY at").map(([role, name, at, statement]) => ({ role, name, at, statement })),
    comments: rows(s, "SELECT id, at, by, about, parent, body, resolved FROM comment ORDER BY at").map(([id, at, by, about, parent, body, resolved]) => ({ id, at, by, about, parent, body, resolved: !!resolved })),
    requests: rows(s, "SELECT id, at, by, about, body, state FROM change_request ORDER BY at").map(([id, at, by, about, body, state]) => ({ id, at, by, about, body, state })),
    revisions: rows(s, "SELECT n, at, by, summary FROM revision ORDER BY n").map(([n, at, by, summary]) => ({ n, at, by, summary })),
  };
  // the output table holds the answer; content holds the quantity and the reasons are in both places
  if (doc.output) {
    content["output.symbol"] ??= doc.output[0];
    content["output.unit"] ??= doc.output[1];
    if (doc.output[2] !== null) content["output.lower"] ??= String(doc.output[2]);
    if (doc.output[3] !== null) content["output.upper"] ??= String(doc.output[3]);
    if (doc.output[4] !== null) content["output.reason_lower"] ??= doc.output[4];
    if (doc.output[5] !== null) content["output.reason_upper"] ??= doc.output[5];
  }
  doc.kind = kindOf(node, content);
  doc.fromSpec = specFields(doc);
  for (const [k, v] of Object.entries(doc.fromSpec)) {
    if (k === "inputs") { if (!doc.inputs.length) doc.inputs = v; continue; }
    if (!(k in content)) { content[k] = v; origin[k] = SPEC_ORIGIN; }
  }
  return doc;
}

export const SPEC_ORIGIN = "spec:seed_content.toml";

/** What the spec package already says about the node (its "spec" section, as tools/seed_design.py
 *  carried it), in the node app's fields: shown as each field's starting value, marked as the
 *  spec's, and written into the node's own fields by adoptSpec(). */
export function specFields(doc) {
  const c = doc.content, out = {};
  const sp = (f) => c[`spec.${f}`];
  const put = (k, v) => { if (v !== undefined && v !== null && v !== "") out[k] = v; };
  put("identity.question", sp("question"));
  put("identity.note", sp("note"));
  put("output.symbol", sp("symbol"));
  put("output.quantity", sp("type"));
  put("output.unit", sp("unit"));
  for (const x of ["lower", "upper", "reason_lower", "reason_upper"]) put(`output.${x}`, sp(x) !== undefined ? String(sp(x)).replace(/\.0$/, "") : undefined);
  if (doc.kind === "computed") {
    put("relation.expression", sp("expression"));
    put("relation.source", sp("source"));
    put("relation.why", sp("why"));
    if (sp("steps")) put("relation.derivation", sp("steps"));
    try {
      const ins = JSON.parse(sp("inputs") || "[]");
      if (ins.length) out.inputs = ins.map(([name, from_node]) => ({ name, from_node, quantity: (CATALOG.rows[from_node] || [])[2] || null, unit: null }));
    } catch (e) { /* a seed without inputs */ }
  } else if (doc.kind === "declared") {
    put("value.number", sp("value") !== undefined ? String(sp("value")) : undefined);
    put("value.source", sp("source"));
  } else if (doc.kind === "kpi") {
    put("requirement.sense", { "<=": "at_most", ">=": "at_least", at_most: "at_most", at_least: "at_least" }[sp("sense")]);
    put("requirement.value", sp("value") !== undefined ? String(sp("value")) : undefined);
    if (sp("source")) put("sources.cited", JSON.stringify([sp("source")]));
  }
  try {
    const a = JSON.parse(sp("assumptions") || "[]");
    if (a.length) put("assumptions", JSON.stringify(a.map((x) => (Array.isArray(x) ? { assumes: x[0], until: x[1] } : x))));
  } catch (e) { /* none */ }
  try {
    const ex = JSON.parse(sp("explain") || "{}");
    for (const [k, v] of Object.entries(ex)) put(`explain.${k}`, typeof v === "string" ? v : JSON.stringify(v));
  } catch (e) { /* none */ }
  return out;
}

/** The SQL that writes the spec's values into the node's own fields (those not written yet). */
export function adoptSpec(db, doc) {
  for (const [k, v] of Object.entries(doc.fromSpec || {})) {
    if (k === "inputs") { const have = db.exec("SELECT count(*) FROM input")[0].values[0][0]; if (!have) setInputs(db, v); continue; }
    if (doc.origin[k] !== SPEC_ORIGIN) continue;
    const [sec, ...rest] = k.split(".");
    db.run("INSERT INTO content (section, field, value, origin) VALUES (?, ?, ?, ?)", [sec, rest.join("."), v, SPEC_ORIGIN]);
  }
  syncOutput(db);
}

export function list(doc, key) { try { const v = JSON.parse(doc.content[key] || "[]"); return Array.isArray(v) ? v : []; } catch (e) { return []; } }

/** The SQL that sets one field (run inside FileSession.change). */
export function setField(db, key, value, who) {
  const [sec, ...rest] = key.split(".");
  const fld = rest.join(".");
  const v = value === null || value === undefined ? null : typeof value === "string" ? value : JSON.stringify(value);
  db.run("DELETE FROM content WHERE section = ? AND field = ?", [sec, fld]);
  if (v !== null && v !== "" && v !== "[]") db.run("INSERT INTO content (section, field, value, origin) VALUES (?, ?, ?, ?)", [sec, fld, v, `typed by ${who}`]);
  if (sec === "output") syncOutput(db);
}

// the output table follows the answer's fields
function syncOutput(db) {
  const get = (f) => { const st = db.prepare("SELECT value FROM content WHERE section = 'output' AND field = ?"); try { st.bind([f]); return st.step() ? st.get()[0] : null; } finally { st.free(); } };
  const num = (x) => (x === null || x === "" || !Number.isFinite(Number(x)) ? null : Number(x));
  db.run("DELETE FROM output");
  const sym = get("symbol");
  if (sym) db.run("INSERT INTO output VALUES (?, ?, ?, ?, ?, ?)", [sym, get("unit"), num(get("lower")), num(get("upper")), get("reason_lower"), get("reason_upper")]);
}

export function setInputs(db, inputs) {
  db.run("DELETE FROM input");
  for (const i of inputs) if (i.name || i.from_node) db.run("INSERT INTO input VALUES (?, ?, ?, ?)", [i.name, i.from_node, i.quantity || null, i.unit || null]);
}

export function setFixtures(db, fixtures) {
  db.run("DELETE FROM fixture");
  fixtures.forEach((x, k) => db.run("INSERT INTO fixture VALUES (?, ?, ?, ?, ?, ?)", [x.name || `v${k + 1}`, JSON.stringify(x.inputs || {}), x.expected === null || x.expected === undefined || x.expected === "" ? null : String(x.expected),
    x.tolerance === "" || x.tolerance === null || x.tolerance === undefined ? null : Number(x.tolerance), JSON.stringify({ provenance: x.provenance || "", source: x.source || "", where: x.where || "" }),
    ["independent-derivation", "published-source", "independent-tool", "physical-bound"].includes(x.provenance) ? 1 : 0]));
}

// ------------------------------------------------------------------ the checks

const UNIT = new Map(CATALOG.units.map(([n, sym, qs]) => [n, { sym, qs }]));
const QUANT = new Set(CATALOG.quantities);
const SOURCE = new Set(CATALOG.sources.map((s) => s[0]));
const PHYS_FN = new Set(CATALOG.physics.map((p) => p[0]));
const filled = (v) => v !== undefined && v !== null && String(v).trim() !== "" && String(v).trim() !== "[]";
const isNum = (v) => filled(v) && Number.isFinite(Number(v));

/** Every problem the checker would find, as the author types: [{ code, level: "!" | "i", step, text }].
 *  ctx: { nodes: Map(id -> { label, quantity, unit, layer, group }) } for inputs (optional). */
export function check(doc, ctx = {}) {
  const k = doc.kind, c = doc.content, out = [];
  const bad = (code, step, text) => out.push({ code, level: "!", step, text });
  const warn = (code, step, text) => out.push({ code, level: "i", step, text });
  if (!k) { bad("N02", "identity", "Say what kind of node this is: declared or computed"); return out; }
  // the explanation (X01–X03), for every kind
  if (filled(c["explain.analogy"]) && !filled(c["explain.analogy_breaks"])) bad("X01", "explain", "The analogy says where it stops being true");
  if (filled(c["explain.wrong_idea"]) && !filled(c["explain.wrong_because"])) bad("X02", "explain", "The common wrong idea says why it is wrong");
  if (k === "computed" && !(filled(c["explain.simply"]) && filled(c["explain.one_line"]))) warn("X03", "explain", "Say it simply and in one line: without them its page serves experts only");
  if (FIXED.has(k)) {
    if (filled(c["other.subject"]) !== filled(c["other.description"])) bad("B02", "feedback", "Something else needs both a subject and a description");
    if (filled(c["feedback.expected"]) && !filled(c["feedback.what_happened"])) bad("B01", "feedback", "Feedback says what happened");
    return out;
  }
  if (!filled(c["identity.question"])) bad("Q01", "identity", "Say the question the node answers");
  if (!filled(c["explain.where_it_breaks"])) bad("E20", "explain", "Say where the answer stops being true (the explanation standard)");
  // its answer (O01–O04)
  const sym = c["output.symbol"], q = c["output.quantity"], u = c["output.unit"];
  if (!filled(sym)) bad("O01", "io", "Give the answer a symbol");
  else if (!/^[A-Za-z_][A-Za-z0-9_]*$/.test(sym)) bad("O01", "io", `The symbol ${sym} has to be letters, digits and _`);
  if (!filled(q)) bad("O02", "io", "Say what quantity the answer is");
  else if (!QUANT.has(q)) bad("O02", "io", `${q} is not a quantity the software knows`);
  if (!filled(u)) bad("O03", "io", "Give the answer's unit");
  else if (!UNIT.has(u)) bad("O03", "io", `${u} is not a unit the software knows`);
  else if (filled(q) && !UNIT.get(u).qs.includes(q)) bad("O03", "io", `${u} does not state ${q} (it states ${UNIT.get(u).qs.join(", ")})`);
  for (const side of ["lower", "upper"]) {
    if (filled(c[`output.${side}`]) && !isNum(c[`output.${side}`])) bad("O04", "io", `The ${side === "lower" ? "lowest" : "highest"} value is not a number`);
    if (filled(c[`output.${side}`]) && !filled(c[`output.reason_${side}`])) bad("O04", "io", `Say why the ${side === "lower" ? "lowest" : "highest"} value is what it is`);
  }
  if (isNum(c["output.lower"]) && isNum(c["output.upper"]) && Number(c["output.lower"]) > Number(c["output.upper"])) bad("O04", "io", "The lowest value is above the highest");
  // inputs (C01–C03)
  if (k === "computed") {
    if (!doc.inputs.length) bad("C01", "io", "A computed node reads at least one input");
    const seen = new Set();
    for (const i of doc.inputs) {
      if (!filled(i.name) || !/^[A-Za-z_][A-Za-z0-9_]*$/.test(i.name)) bad("C02", "io", `An input needs a binding of letters, digits and _ (${i.name || "empty"})`);
      else if (seen.has(i.name)) bad("C02", "io", `The binding ${i.name} is used twice`);
      seen.add(i.name);
      if (i.from_node === doc.node.id) bad("C02", "io", "A node cannot read itself");
      const p = ctx.nodes ? ctx.nodes.get(i.from_node) : null;
      if (ctx.nodes && !p) bad("C02", "io", `${i.from_node || "(empty)"} is not a node of the design`);
      if (p && p.quantity && i.quantity && p.quantity !== i.quantity) bad("C03", "io", `${i.from_node} publishes ${p.quantity}, not ${i.quantity}`);
      if (p && p.layer && doc.node.layer && String(p.layer) !== String(doc.node.layer) && !String(p.kind || "").includes("interface")) warn("C04", "io", `${i.from_node} is in layer ${p.layer}; layers meet only at the door and the interface rows`);
    }
  } else if (doc.inputs.length) bad("C01", "io", "Only a computed node reads inputs");
  // the relation (R01, R06, R07)
  if (k === "computed") {
    for (const [key, what] of [["relation.expression", "the relation"], ["relation.source", "its source"], ["relation.why", "why it is this relation"]]) if (!filled(c[key])) bad("R01", "theory", `Give ${what}`);
    list(doc, "relation.derivation").forEach((s, n) => { if (!filled(s)) bad("R07", "theory", `Derivation step ${n + 1} has no text`); });
    if (filled(c["relation.expression"])) {
      const names = new Set([sym, ...doc.inputs.map((i) => i.name)]);
      const unknown = (String(c["relation.expression"]).match(/[A-Za-z_][A-Za-z0-9_]*/g) || [])
        .filter((n) => !names.has(n) && !MATH_NAMES.has(n) && !PHYS_FN.has(n));
      if (unknown.length) warn("R04", "theory", `The relation uses names that are not its inputs or its answer: ${[...new Set(unknown)].join(", ")}`);
    }
  }
  list(doc, "assumptions").forEach((a, n) => { if (!filled(a.assumes) || !filled(a.until)) bad("R06", "theory", `Assumption ${n + 1} says what it assumes and when that stops being true`); });
  // the pseudocode (R03 through the language's own checker)
  if (k === "computed") {
    if (!filled(c["code.pseudocode"])) bad("P01", "pseudocode", "Write the relation in pseudocode");
    else {
      const p = pcodeCheck(c["code.pseudocode"], sym, doc.inputs);
      for (const e of p.problems) bad("P02", "pseudocode", e);
    }
  }
  // value, requirement, evidence (V01–V03)
  if (k === "declared") {
    if (!isNum(c["value.number"])) bad("V01", "value", "Give the value as a number");
    else {
      const v = Number(c["value.number"]);
      if (isNum(c["output.lower"]) && v < Number(c["output.lower"])) bad("V01", "value", "The value is below the lowest value");
      if (isNum(c["output.upper"]) && v > Number(c["output.upper"])) bad("V01", "value", "The value is above the highest value");
    }
    if (!filled(c["value.source"])) bad("V01", "value", "Say where the value comes from");
  }
  if (k === "kpi" && !["at_most", "at_least"].includes(c["requirement.sense"])) bad("V02", "value", "Say which way the requirement binds: at most or at least");
  if (k === "evidence") {
    if (!CATALOG.metrics.includes(c["evidence.metric"])) bad("V03", "value", "Name the metric the simulator computes");
    if (!list(doc, "evidence.rungs").length) bad("V03", "value", "Name at least one rung (SILS, PIL, OILS, HILS)");
  }
  // sources (S01)
  const newSrc = list(doc, "sources.new");
  newSrc.forEach((s, n) => { if (!filled(s.id) || !filled(s.title) || !filled(s.where)) bad("S01", "evidence", `New source ${n + 1} needs an id, a title and exactly where`); });
  const known = new Set([...SOURCE, ...newSrc.map((s) => s.id)]);
  for (const key of ["relation.source", "value.source"]) {
    if (!filled(c[key])) continue;
    for (const id of String(c[key]).split(/[,;]\s*/).map((x) => x.trim()).filter(Boolean)) if (!known.has(id) && /^[a-z0-9_]+$/.test(id)) bad("S01", key.startsWith("relation") ? "theory" : "value", `The source ${id} is not known: add it under Evidence, New sources`);
  }
  // test vectors (T01–T04)
  if (k === "computed") {
    if (!doc.fixtures.length) warn("T04", "evidence", "No test vector: its validation stays low");
    doc.fixtures.forEach((x, n) => {
      const name = `Test vector ${n + 1}`;
      if (!CATALOG.provenance.includes(x.provenance)) bad("T01", "evidence", `${name}: its answer comes from ${CATALOG.provenance.join(", ")}; never from the code itself`);
      if (!filled(x.source) || !filled(x.where)) bad("T02", "evidence", `${name}: cite the source and the page, table or figure`);
      for (const i of doc.inputs) if (!isNum(x.inputs[i.name])) bad("T03", "evidence", `${name}: give a number for ${i.name}`);
      if (!Number.isFinite(x.expected)) bad("T03", "evidence", `${name}: give the expected answer as a number`);
      if (!(Number(x.tolerance) > 0)) bad("T03", "evidence", `${name}: give a tolerance above zero`);
    });
  }
  // results
  const r = results(doc);
  if (r && r.problems.length) for (const p of r.problems) bad("Y01", "results", p);
  // the belief record (D01, D03, D06, D07)
  for (const [key, what] of [["belief.area", "its area"], ["belief.believed", "what was believed"], ["belief.now_know", "what we now know"], ["belief.plan_change", "what changed in the plan"]]) if (!filled(c[key])) bad("D01", "belief", `The belief record needs ${what}`);
  if (filled(c["belief.status"])) {
    if (!CATALOG.belief_statuses.includes(c["belief.status"])) bad("D03", "belief", "The status is broke, held or untested");
    if (!filled(c["belief.tested"])) bad("D03", "belief", c["belief.status"] === "untested" ? "Say the test that would settle it" : "Say what tested it");
    if (c["belief.status"] === "untested") warn("D07", "belief", "The belief is untested: it is counted under \"Beliefs not yet tested\"");
  } else bad("D03", "belief", "Say whether a test broke the belief, held it, or none has been run");
  if (filled(c["belief.cost_k"]) && !(isNum(c["belief.cost_k"]) && Number(c["belief.cost_k"]) >= 0)) bad("D06", "belief", "The cost is a number, zero or more, in thousands of dollars");
  return out;
}

const MATH_NAMES = new Set(["sqrt", "sin", "cos", "tan", "asin", "acos", "atan", "atan2", "exp", "log", "log10", "abs", "min", "max", "pi", "e", "norm", "dot", "cross", "sum", "pow", "floor", "ceil", "round", "sign", "hypot", "clamp", "if", "then", "else", "and", "or", "not"]);

/** The pseudocode, checked with the language's own checker (design/js/pcode.js): its problems, and
 *  the fn that computes the answer. */
export function pcodeCheck(text, symbol, inputs = []) {
  const { program, errors } = compile([{ file: "node.pc", text: String(text) }]);
  const problems = errors.map((e) => `line ${e.line}: ${e.message}`);
  if (problems.length) return { problems, fn: null, program: null };
  const fns = Object.values(program.fns);
  const fn = fns.find((x) => x.outs.some((o) => o.name === symbol)) || null;
  if (!fns.length) problems.push("The pseudocode has no fn");
  else if (symbol && !fn) problems.push(`No fn has the answer's symbol ${symbol} as an output (R03)`);
  if (fn) {
    const params = new Set(fn.params.map((p) => p.name));
    for (const i of inputs) if (i.name && !params.has(i.name)) problems.push(`The fn ${fn.name} has no input ${i.name}, which the node reads`);
  }
  return { problems, fn, program };
}

/** "Try it": the pseudocode run on every test vector, against the expected answer within its
 *  tolerance (relative to the answer, or absolute when the answer is zero). */
export function tryIt(doc) {
  const c = doc.content;
  const p = pcodeCheck(c["code.pseudocode"] || "", c["output.symbol"], doc.inputs);
  if (p.problems.length || !p.fn) return { ok: false, problems: p.problems, runs: [] };
  const I = makeInterpreter(p.program);
  const at = p.fn.outs.findIndex((o) => o.name === c["output.symbol"]);
  const runs = doc.fixtures.map((x) => {
    try {
      const args = p.fn.params.map((pa) => { const v = Number(x.inputs[pa.name]); if (!Number.isFinite(v)) throw new Error(`no number for ${pa.name}`); return v; });
      const got = I.call(p.fn.name, args)[at];
      const tol = Number(x.tolerance) || 0;
      const err = Math.abs(got - x.expected);
      const pass = Number.isFinite(got) && err <= tol * Math.max(Math.abs(x.expected), x.expected === 0 ? 1 : 0);
      return { name: x.name, got, expected: x.expected, tolerance: tol, pass };
    } catch (e) { return { name: x.name, error: e.message, pass: false }; }
  });
  return { ok: runs.length > 0 && runs.every((r) => r.pass), problems: [], runs };
}

/** The results table pasted in: { columns: [{ name, unit }], rows: [[...]], problems }. */
export function results(doc) {
  if (!filled(doc.content["results.table"])) return null;
  try {
    const t = JSON.parse(doc.content["results.table"]);
    const problems = [];
    t.rows.forEach((r, i) => r.forEach((v, j) => { if (v !== "" && v !== null && !Number.isFinite(Number(v)) && t.numeric && t.numeric[j]) problems.push(`row ${i + 1}, ${t.columns[j].name}: ${v} is not a number`); }));
    return { ...t, problems };
  } catch (e) { return { columns: [], rows: [], problems: ["The results could not be read; paste them again"] }; }
}

/** A table pasted from Excel (tab-separated) or written as CSV, read as the results table: the
 *  first row names the columns; a second row is units when it holds no number. */
export function parsePasted(text) {
  const lines = String(text).replace(/\r/g, "").split("\n").filter((l) => l.trim() !== "");
  if (!lines.length) return null;
  const sep = lines[0].includes("\t") ? "\t" : lines[0].includes(";") ? ";" : ",";
  const cells = lines.map((l) => l.split(sep).map((x) => x.trim()));
  const names = cells[0];
  let body = cells.slice(1), units = names.map(() => "");
  if (body.length && body[0].every((v) => v === "" || !Number.isFinite(Number(v.replace(",", "."))))) { units = body[0]; body = body.slice(1); }
  const width = names.length;
  const rowsOut = body.map((r) => Array.from({ length: width }, (_, j) => (r[j] ?? "").replace(/^(-?\d+),(\d+)$/, "$1.$2")));
  const numeric = names.map((_, j) => rowsOut.length > 0 && rowsOut.filter((r) => r[j] !== "").every((r) => Number.isFinite(Number(r[j]))));
  return { columns: names.map((n, j) => ({ name: n, unit: units[j] || "" })), rows: rowsOut, numeric };
}

// ------------------------------------------------------------------ progress, debt, readiness

/** How much of each step is filled: { step: [filled, wanted] }. */
export function progress(doc) {
  const out = {};
  for (const s of stepsFor(doc.kind)) {
    const fs = s.fields.filter((fd) => fieldShown(fd, doc.kind, doc.node));
    const need = fs.filter((fd) => !fd.need || fd.need.includes(doc.kind));
    let n = 0;
    for (const fd of need) if (has(doc, fd)) n++;
    out[s.id] = [n, need.length];
  }
  return out;
}

function has(doc, fd) {
  if (fd.type === "inputs") return doc.inputs.length > 0;
  if (fd.type === "fixtures") return doc.fixtures.length > 0;
  if (fd.type === "pictures") return doc.attachments.length > 0;
  return filled(doc.content[fd.key]);
}

/** What the node still owes before it can be trusted: [{ text, step }]. */
export function evidenceDebt(doc) {
  const d = [], k = doc.kind, c = doc.content;
  if (FIXED.has(k) || !k) return d;
  if (k === "computed") {
    if (!doc.fixtures.some((x) => x.outside)) d.push({ step: "evidence", text: "no test vector with an answer from outside the code: it cannot be sealed as confirmed (P6)" });
    const t = filled(c["code.pseudocode"]) && doc.fixtures.length ? tryIt(doc) : null;
    if (t && !t.ok) d.push({ step: "pseudocode", text: `the pseudocode does not reproduce ${t.runs.filter((r) => !r.pass).length} of ${t.runs.length} test vector(s)` });
  }
  if (k === "evidence" && !results(doc)) d.push({ step: "results", text: "no results yet: the evidence is a plan, not a finding" });
  if (!list(doc, "sources.cited").length && !filled(c["relation.source"]) && !filled(c["value.source"])) d.push({ step: "evidence", text: "no source cited" });
  if (c["belief.status"] === "untested") d.push({ step: "belief", text: "the belief behind it is untested" });
  if (!checkedBy(doc)) d.push({ step: "review", text: "nobody has checked it (W01): it runs as UNCONFIRMED" });
  return d;
}

/** The fingerprint of what the author wrote: a signature stands behind exactly this. */
export async function fingerprint(doc) {
  const body = JSON.stringify({ content: Object.keys(doc.content).sort().filter((k) => !k.startsWith("status.")).map((k) => [k, doc.content[k]]),
    inputs: doc.inputs, fixtures: doc.fixtures, attachments: doc.attachments.map((a) => [a.name, a.size]) });
  const h = new Uint8Array(await crypto.subtle.digest("SHA-256", new TextEncoder().encode(body)));
  return Array.from(h.slice(0, 12), (b) => b.toString(16).padStart(2, "0")).join("");
}

export function signature(doc, role) {
  const s = [...doc.signatures].reverse().find((x) => x.role === role);
  if (!s) return null;
  let st = {}; try { st = JSON.parse(s.statement || "{}"); } catch (e) { st = { statement: s.statement }; }
  return { ...s, ...st };
}
export function checkedBy(doc) { return signature(doc, "checked by"); }

/** Where the node stands: shell, draft, ready (the author's), checked (an engineer's), and
 *  whether a signature still covers what is written. */
export async function standing(doc) {
  const fp = await fingerprint(doc);
  const ready = signature(doc, "author ready"), checked = checkedBy(doc);
  const written = Object.keys(doc.content).some((k) => !k.startsWith("identity.") && !k.startsWith("status.") && doc.origin[k] && doc.origin[k].startsWith("typed by"));
  const stale = (s) => s && s.fingerprint !== fp;
  return {
    fingerprint: fp,
    state: checked && !stale(checked) ? "checked" : ready && !stale(ready) ? "ready" : written ? "draft" : "shell",
    readyStale: !!stale(ready), checkedStale: !!stale(checked), ready, checked,
  };
}
