// node_model.test.mjs -- each check of design/js/node_model.js, the node app's port of the spec's
// checker rules (SPEC §5.11.2, spec/tools/intake.py), made to fire by one mistake and to stay quiet
// on a node with none; the kinds; reading a pasted table; "try it".
// Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
const ROOT = new URL("../../", import.meta.url).pathname;
const M = await import(ROOT + "design/js/node_model.js");
let passed = 0, failed = 0;
function t(name, fn) { try { fn(); passed++; } catch (e) { failed++; console.log(`FAIL ${name}: ${e.message}`); } }
function ok(c, m) { if (!c) throw new Error(m); }

const good = () => {
  const d = { node: { id: "x_new", kind: "leaf", layer: "2" }, origin: {}, attachments: [], signatures: [], comments: [], requests: [], revisions: [],
    content: { "identity.form_kind": "computed", "identity.question": "How big?", "explain.where_it_breaks": "Never.", "explain.simply": "s", "explain.one_line": "o",
      "output.symbol": "m_max", "output.quantity": "DipoleMoment", "output.unit": "AmpereSquareMetre", "output.lower": "0", "output.reason_lower": "a magnitude",
      "relation.expression": "m_max = n * i * a", "relation.source": "wertz1978", "relation.why": "linear",
      "code.pseudocode": "fn dipole(n: real[1], i: real[A], a: real[m^2]) -> m_max: real[A m^2]\n    m_max = n*i*a\nend\n",
      "belief.area": "model", "belief.believed": "b", "belief.status": "held", "belief.tested": "t", "belief.now_know": "n", "belief.plan_change": "p" },
    inputs: [{ name: "n", from_node: "gm_0" }, { name: "i", from_node: "gm_4" }, { name: "a", from_node: "gm_5" }],
    fixtures: [{ name: "v1", inputs: { n: 200, i: 0.1, a: 0.01 }, expected: 0.2, tolerance: 1e-9, provenance: "published-source", source: "wertz1978", where: "p. 204", outside: true }] };
  d.kind = M.kindOf(d.node, d.content);
  return d;
};
const codes = (d) => M.check(d).filter((p) => p.level === "!").map((p) => p.code);
const fires = (code, mutate) => { const d = good(); mutate(d); d.kind = M.kindOf(d.node, d.content); ok(codes(d).includes(code), `${code} not found in ${codes(d)}`); };

t("a node with no mistake has nothing to fix", () => ok(codes(good()).length === 0, codes(good()).join(",")));
t("O01 symbol", () => fires("O01", (d) => { d.content["output.symbol"] = "m max"; }));
t("O02 quantity", () => fires("O02", (d) => { d.content["output.quantity"] = "Wobble"; }));
t("O03 unit that does not state the quantity", () => fires("O03", (d) => { d.content["output.unit"] = "Metre"; }));
t("O04 bound without its reason", () => fires("O04", (d) => { d.content["output.upper"] = "5"; }));
t("O04 bounds out of order", () => fires("O04", (d) => { d.content["output.upper"] = "-1"; d.content["output.reason_upper"] = "r"; }));
t("C01 computed reads nothing", () => fires("C01", (d) => { d.inputs = []; }));
t("C02 binding twice", () => fires("C02", (d) => { d.inputs[1].name = "n"; }));
t("C02 reads itself", () => fires("C02", (d) => { d.inputs[0].from_node = "x_new"; }));
t("R01 relation without its source", () => fires("R01", (d) => { delete d.content["relation.source"]; }));
t("R06 assumption without its limit", () => fires("R06", (d) => { d.content.assumptions = JSON.stringify([{ assumes: "linear", until: "" }]); }));
t("P02 pseudocode the language refuses", () => fires("P02", (d) => { d.content["code.pseudocode"] = "fn f(n: real[1]) -> m_max: real[m]\n    m_max = n\nend\n"; }));
t("P02 pseudocode without the node's input", () => fires("P02", (d) => { d.content["code.pseudocode"] = "fn f(n: real[1], i: real[A]) -> m_max: real[A]\n    m_max = n*i\nend\n"; }));
t("T01 a test vector from the code itself", () => fires("T01", (d) => { d.fixtures[0].provenance = "self-snapshot"; }));
t("T02 a test vector without its page", () => fires("T02", (d) => { d.fixtures[0].where = ""; }));
t("T03 a test vector missing a binding", () => fires("T03", (d) => { delete d.fixtures[0].inputs.i; }));
t("T03 a tolerance of zero", () => fires("T03", (d) => { d.fixtures[0].tolerance = 0; }));
t("S01 an unknown source", () => fires("S01", (d) => { d.content["relation.source"] = "nosuchbook"; }));
t("S01 a new source without where", () => fires("S01", (d) => { d.content["sources.new"] = JSON.stringify([{ id: "x", title: "t", where: "" }]); }));
t("X01 analogy without where it breaks", () => fires("X01", (d) => { d.content["explain.analogy"] = "a sail"; }));
t("X02 wrong idea without why", () => fires("X02", (d) => { d.content["explain.wrong_idea"] = "bigger is better"; }));
t("E20 where it breaks", () => fires("E20", (d) => { delete d.content["explain.where_it_breaks"]; }));
t("D01 belief record", () => fires("D01", (d) => { delete d.content["belief.now_know"]; }));
t("D03 status without its test", () => fires("D03", (d) => { delete d.content["belief.tested"]; }));
t("D06 a cost that is not a number", () => fires("D06", (d) => { d.content["belief.cost_k"] = "lots"; }));
t("V01 declared value outside its bounds", () => fires("V01", (d) => { d.content["identity.form_kind"] = "declared"; d.inputs = []; d.content["value.number"] = "-3"; d.content["value.source"] = "wertz1978"; }));
t("V02 KPI without its sense", () => { const d = good(); d.node = { id: "p1k_0", kind: "leaf" }; d.inputs = []; d.kind = M.kindOf(d.node, d.content); ok(d.kind === "kpi", d.kind); ok(codes(d).includes("V02"), codes(d)); });
t("V03 evidence without metric and rungs", () => { const d = good(); d.node = { id: "p1a_0", kind: "leaf" }; d.inputs = []; d.kind = M.kindOf(d.node, d.content); ok(d.kind === "evidence", d.kind); ok(codes(d).includes("V03"), codes(d)); });
t("fixed kinds: closure and interface take only the explanation and feedback", () => {
  for (const [kind, want] of [["closure_analysis", "closure"], ["interface", "interface"], ["closure_interface", "interface"], ["required", "target"]]) {
    const k = M.kindOf({ id: "z", kind }, {});
    ok(k === want, `${kind} -> ${k}`);
    ok(M.stepsFor(k).map((s) => s.id).join() === "identity,explain,pictures,feedback", M.stepsFor(k).map((s) => s.id).join());
  }
});
t("an unnamed row asks its author for the kind (N02)", () => { const k = M.kindOf({ id: "l3_x_row_01", kind: "internal" }, {}); ok(k === null && M.check({ kind: k, content: {} })[0].code === "N02", String(k)); });
t("try it: the pseudocode reproduces its vector, and a wrong expected answer is caught", () => {
  const d = good(); ok(M.tryIt(d).ok, JSON.stringify(M.tryIt(d)));
  d.fixtures[0].expected = 0.3; ok(!M.tryIt(d).ok, "a wrong answer passes");
  ok(M.evidenceDebt(d).some((x) => x.text.includes("does not reproduce")), "and it is evidence debt");
});
t("a table pasted from Excel: tabs, a units row, decimal commas", () => {
  const r = M.parsePasted("t\tAPE\ns\tdeg\n0\t0,5\n10\t1.25\n");
  ok(r.columns[1].unit === "deg" && r.rows[0][1] === "0.5" && r.numeric.every(Boolean), JSON.stringify(r));
});
t("evidence debt: no outside vector, untested belief, unconfirmed", () => {
  const d = good(); d.fixtures[0].outside = false; d.content["belief.status"] = "untested";
  const debt = M.evidenceDebt(d).map((x) => x.text).join(" | ");
  ok(debt.includes("outside the code") && debt.includes("untested") && debt.includes("UNCONFIRMED"), debt);
});
console.log(`node_model: ${passed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
