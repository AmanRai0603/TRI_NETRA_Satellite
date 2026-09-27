#!/usr/bin/env python3
"""The plan as one read-only object, shared by tools/forms.py, tools/intake.py
and tools/check_case.py.

It reads plan/tree.json, plan/expected_node_ids.json, plan/seed_content.toml,
plan/case_inputs.toml, plan/kpis.toml, plan/units.toml and plan/physics.toml,
and answers the questions the forms and the checkers ask: which rows exist,
what each one is, what it reads and what reads it, which units a quantity may
be stated in, which physics functions exist.

In the built repository the same questions are answered by the node sheets
themselves (`node.toml`), adcs-units and adcs-core::physics; this module is
the package's stand-in, and it never writes anything.

Row kinds, as the tree names them and as a node form names them:

    tree.json            node form
    "computed"           computed    a relation from other rows
    "set here"           declared    a stated number
    "target — required"  required    a requirement; its value comes from each case
    "achieved — …"       achieved    an evidence row; only a campaign supplies it
    "the door …"         door        the one crossing from layer 1 to layer 2
"""

import copy
import hashlib
import json
import os
import re

try:
    import tomllib
except ImportError:  # pragma: no cover
    raise SystemExit("the package tools need Python 3.11+ (tomllib)")

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

# Row owner by branch (SPEC.md §5.9). The same maps tools/check_seed_with_vleo.py
# patches into VLEO's seeder, so the owner a form shows is the owner the seeder writes.
OWNER = {"svc": "systems", "cpt": "systems", "msn": "environment", "sat": "systems",
         "sub": "gnc", "ver": "verification",
         "sb0": "environment", "sb1": "sensing", "sb2": "actuators", "sb3": "gnc", "sb4": "avionics",
         "cas": "sales", "cat": "systems", "cmr": "sales", "ord": "programme", "fac": "facility",
         "std": "quality", "sup": "programme", "hrt": "programme", "rsk": "quality"}
LAYER3_OWNER = {"dist": "environment", "sens": "sensing", "est": "gnc", "mtq": "actuators",
                "rw": "actuators", "fmr": "actuators", "rcs": "actuators", "ctl": "gnc",
                "pnt": "gnc", "modes": "gnc", "fsw": "avionics", "budget": "systems",
                "oils": "verification", "hils": "verification"}
OWNERS = sorted(set(OWNER.values()) | set(LAYER3_OWNER.values()))
TAGS = ["mtq", "rw", "fmr", "rcs"]
KINDS = ["computed", "declared", "required", "achieved"]
RUNGS = ["sils", "pil", "oils", "hils"]
PROVENANCE_OK = ["independent-derivation", "published-source", "independent-tool", "physical-bound"]
PROVENANCE_REFUSED = ["self-snapshot", "agent-generated"]
TIERS = ["A+", "A", "B", "C"]
# What the plant reads from a case, by environment switch (SPEC.md §10.2). A
# scenario needs these keys as well as its own `case:` references, so a case that
# leaves one blank blocks that scenario by name.
PLANT_READS = {
    "always": ["mass.m", "mass.imax", "mass.iint", "mass.imin"],
    "aerodynamic": ["surface.afr", "surface.cpa", "surface.cd"],
    "solar_pressure": ["surface.asun", "surface.cps", "surface.refl"],
    "residual_dipole": ["magnetic.dres"],
}
LAYER_NAME = {1: "Layer 1 — the company", 2: "Layer 2 — the satellite's ADCS", 3: "Layer 3 — a subsystem"}


def toml(path):
    with open(path, "rb") as f:
        return tomllib.load(f)


def kind_of(tree_kind):
    if tree_kind == "computed":
        return "computed"
    if tree_kind == "set here":
        return "declared"
    if tree_kind.startswith("target"):
        return "required"
    if tree_kind.startswith("achieved"):
        return "achieved"
    if tree_kind.startswith("the door"):
        return "door"
    return "group"


def digest(obj, n=16):
    return hashlib.sha256(json.dumps(obj, sort_keys=True, ensure_ascii=False).encode()).hexdigest()[:n]


BASE_IDENTITY = ["id", "tree_id", "layer", "group", "kind", "owner", "case_key", "supplied_by"]


def base_of(node):
    """A node form's `base`: the node's identity and every field a form may
    propose, and nothing else. The ledger's beliefs and risks, the version list
    and the state are shown on the form but left out, so a risk moved by another
    request does not make every form of every node carrying it stale (F04)."""
    return digest({"identity": {k: node.get(k) for k in BASE_IDENTITY}, "content": proposal_from(node)})


class Plan:
    def __init__(self, root=None):
        root = root or os.environ.get("ADCS_PLAN_ROOT") or ROOT
        self.root = root
        p = lambda *a: os.path.join(root, "plan", *a)
        self.tree = json.load(open(p("tree.json"), encoding="utf-8"))
        self.ids = json.load(open(p("expected_node_ids.json"), encoding="utf-8"))
        self.tree_of = {v: k for k, v in self.ids.items()}
        seed = toml(p("seed_content.toml"))
        self.sources = {s["id"]: s for s in seed.get("source", [])}
        self.seed = {r["tree_id"]: r for r in seed.get("row", []) if "tree_id" in r}
        self.case = toml(p("case_inputs.toml"))
        self.case_key = {i["tree_id"]: i["key"] for i in self.case.get("input", [])}
        # who supplies a declared row that is not the case: product, tuned, evidence (SPEC.md §8.2) or derisk (§5.13)
        self.supplier = {x["tree_id"]: x["by"] for x in self.case.get("supplier", [])}
        self.kpis = toml(p("kpis.toml")).get("kpi", [])
        self.kpi_by = {}
        for k in self.kpis:
            for role in ("requirement", "evidence", "analysis"):
                if k.get(role) and k[role] != "none":
                    self.kpi_by.setdefault(k[role], []).append((role, k))
        self.metrics = sorted({k["metric"] for k in self.kpis})
        u = toml(p("units.toml"))
        self.quantities = [q["name"] for q in u.get("quantity", [])]
        self.units = {x["name"]: x for x in u.get("unit", [])}
        self.physics = {"%s::%s" % (f["module"], f["name"]): f for f in toml(p("physics.toml")).get("function", [])}
        self.derisk = load_ledger(root)
        self.mgt = {r[0] for r in self.tree["HN_MGT"]}
        self.rows = {r[0]: r for r in self.tree["HN_MGT"] + self.tree["HN_SYS"]}
        self.groups = {r[2] for r in self.rows.values() if r[2]}
        self.leaves = [t for t in self.rows if t not in self.groups]
        self.ve_in, self.ve_out, self.ke_out = {}, {}, {}
        for a, b, w in self.tree["VE"]:
            self.ve_in.setdefault(b, []).append((a, w))
            self.ve_out.setdefault(a, []).append((b, w))
        for a, b, w in self.tree["KE"]:
            self.ke_out.setdefault(a, []).append((b, w))

    # ------------------------------------------------------------ rows
    def layer(self, t):
        return 1 if t in self.mgt else 2

    def ancestors(self, t):
        out, cur = [], self.rows[t][2]
        while cur:
            out.append(cur)
            cur = self.rows[cur][2]
        return out

    def owner(self, t):
        anc = self.ancestors(t)
        for a in anc:
            if a.startswith("sb") and a in OWNER:
                return OWNER[a]
        top = anc[-2] if len(anc) >= 2 else (anc[0] if anc else t)
        return OWNER.get(top, "systems")

    def owner_in(self, group):
        """The owner a new node placed in `group` would get."""
        chain = [group] + self.ancestors(group)
        for a in chain:
            if a.startswith("sb") and a in OWNER:
                return OWNER[a]
        top = chain[-2] if len(chain) >= 2 else chain[0]
        return OWNER.get(top, "systems")

    def kind(self, t):
        return "group" if t in self.groups else kind_of(self.rows[t][3])

    def node_id(self, t):
        return self.ids.get(t, t)

    def resolve(self, ref):
        """A tree id or a node id, to a tree id (None when neither)."""
        if ref in self.rows:
            return ref
        return self.tree_of.get(ref)

    def output_of(self, t):
        d = self.seed.get(t, {})
        if d.get("type"):
            return d.get("type"), d.get("unit")
        key = self.case_key.get(t)
        if key:
            unit = next(i["unit"] for i in self.case["input"] if i["key"] == key)
            q = self.units.get(unit, {}).get("quantity", [""])[0]
            return q, unit
        return "", ""

    def group_leaves(self, g):
        return [t for t in self.leaves if self.rows[t][2] == g]

    # ------------------------------------------------------------ the node, as a form shows it
    def skeleton(self, t):
        """What seeding alone gives a row: identity, placement and the edges the
        tree declares. Nothing else is written until a form brings it."""
        row = self.rows[t]
        kind = self.kind(t)
        n = empty_node()
        n.update({
            "id": self.node_id(t), "tree_id": t, "label": row[1], "layer": self.layer(t),
            "group": self.node_id(row[2]), "group_label": self.rows[row[2]][1] if row[2] else "",
            "kind": kind, "owner": self.owner(t), "tags": [x for x in row[4].split(",") if x],
            "state": "seeded", "case_key": self.case_key.get(t, ""), "supplied_by": self.supplier.get(t, ""),
        })
        n["beliefs"], n["risks"] = self.ledger_for(t)
        n["inputs"] = [{"binding": "", "from": self.node_id(a), "type": self.output_of(a)[0]} for a, _ in self.ve_in.get(t, [])]
        for role, k in self.kpi_by.get(t, []):
            if role == "requirement":
                n["sense"] = k["sense"]
            if role == "evidence":
                n["evidence"] = {"metric": k["metric"], "rungs": ["sils", "oils", "hils"]}
        return n

    def current(self, t):
        """The row with its seed content, as the released software would hold it
        once the seed form has been through intake (SPEC.md §5.8)."""
        n = self.skeleton(t)
        d = self.seed.get(t)
        if not d:
            return n
        n.update({"tier": d.get("tier", ""), "state": "unconfirmed", "question": d.get("question", ""),
                  "note": d.get("note", "")})
        n["output"] = {k: d.get(k, "") for k in ("symbol", "type", "unit", "lower", "upper", "reason_lower", "reason_upper")}
        if d.get("kind") == "computed":
            n["relation"] = {"expression": d.get("expression", ""), "source": d.get("source", ""),
                             "why": d.get("why", ""), "reading": d.get("reading", "")}
            steps = d.get("steps", [])
            n["steps"] = [{"text": s, "binds": d["symbol"] if i == len(steps) - 1 else "",
                           "type": d["type"] if i == len(steps) - 1 else "",
                           "physics": d.get("physics", "") if i == len(steps) - 1 else "",
                           "call": d.get("call", "") if i == len(steps) - 1 else "", "new_function": ""}
                          for i, s in enumerate(steps)]
            n["inputs"] = [{"binding": b, "from": self.node_id(a), "type": self.output_of(a)[0]} for b, a in d.get("inputs", [])]
        else:
            n["relation"] = {"expression": d.get("expression", ""), "source": d.get("source", ""), "why": "", "reading": ""}
        if d.get("kind") == "declared" and t not in self.case_key and not d.get("sense"):
            n["value"] = d.get("value", "")
        elif "value" in d:
            n["reference_value"] = d.get("value")
        if d.get("sense"):
            n["sense"] = d["sense"]
        n["assumptions"] = [{"text": a, "fails_when": b} for a, b in d.get("assumptions", [])]
        n["zero_when_absent"] = [self.node_id(x) for x in d.get("zero_when_absent", [])]
        n["bundles"] = list(d.get("bundles", []))
        n["fixtures"] = [{"label": f.get("label", ""), "inputs": f.get("inputs", {}), "expect": f.get("expect", ""),
                          "tolerance": f.get("tolerance", ""), "provenance": f.get("provenance", ""),
                          "source": f.get("source", ""), "where": f.get("note", "")} for f in d.get("fixture", [])]
        n["fixture_wanted"] = d.get("fixture_wanted", "")
        ex = d.get("explain") or {}
        n["explain"] = {k: (list(ex.get(k, [])) if k == "why_chain" else ex.get(k, "")) for k in EXPLAIN_KEYS}
        # Version 1 is the seed content, taken through intake at the first build (SPEC.md §5.13).
        n["versions"] = [{"n": 1, "release": "first build", "request": "seed form", "belief": "", "status": "untested",
                          "previous_issue": "", "benefit": "the node's first content, from %s; UNCONFIRMED until a person confirms it" % d.get("source", "its source")}]
        return n

    def ledger_for(self, t):
        """The beliefs about a row, and the risks they carry (SPEC.md §5.13)."""
        names = {t, self.node_id(t)}
        bel = [b for b in self.derisk["beliefs"] if names & set(b.get("about", []))]
        rids = []
        for b in bel:
            for r in list(b.get("risks", [])) + [m.get("risk") for m in b.get("move", [])]:
                if r not in rids:
                    rids.append(r)
        risks = [self.derisk["risks"][r] for r in rids if r in self.derisk["risks"]]
        return ([{"id": b["id"], "quarter": b.get("quarter", ""), "area": b.get("area", ""), "status": b.get("status", ""),
                  "believed": b.get("believed", ""), "tested": b.get("tested", ""), "would_test": b.get("would_test", "")} for b in bel],
                [{"id": r["id"], "title": r.get("title", ""), "level": r.get("level", 0), "area": r.get("area", ""),
                  "closing_test": r.get("closing_test", "")} for r in risks])

    def consumers(self, t):
        return [{"id": self.node_id(b), "label": self.rows[b][1], "why": w} for b, w in self.ve_out.get(t, [])]

    def kpis_fed(self, t):
        return [{"id": self.node_id(b), "label": self.rows[b][1], "why": w} for b, w in self.ke_out.get(t, [])]

    def catalog(self):
        """Pick-lists for the node form, and what the checker resolves against."""
        nodes = []
        for t in self.leaves:
            q, u = self.output_of(t)
            nodes.append([self.node_id(t), t, self.rows[t][1], q, u, self.layer(t), self.node_id(self.rows[t][2]), self.kind(t)])
        return {
            "units": [[n, u["symbol"], u["quantity"], u["note"]] for n, u in sorted(self.units.items())],
            "quantities": list(self.quantities),
            "nodes": nodes,
            "groups": [[self.node_id(g), g, self.rows[g][1], self.layer(g) if g in self.rows else 0,
                        len(self.group_leaves(g))] for g in sorted(self.groups) if g in self.rows and self.group_leaves(g)],
            "sources": [[s["id"], s["title"], s.get("used_for", "")] for s in self.sources.values()],
            "risks": [[r["id"], r.get("title", ""), r.get("level", 0), r.get("area", "")] for r in self.derisk["risks"].values()],
            "areas": AREAS, "belief_statuses": BELIEF_STATUSES,
            "physics": [[k, f["args"], f["source"]] for k, f in sorted(self.physics.items())],
            "owners": OWNERS, "tags": TAGS, "kinds": KINDS, "metrics": self.metrics, "rungs": RUNGS,
            "provenance": PROVENANCE_OK, "tiers": TIERS,
        }

    def edges(self):
        """The derivation graph as node ids: producer -> [consumers]."""
        g = {}
        for a, b, _ in self.tree["VE"]:
            g.setdefault(self.node_id(a), set()).add(self.node_id(b))
        return g


def empty_node():
    return {
        "id": "", "tree_id": "", "label": "", "layer": 0, "group": "", "group_label": "", "kind": "", "owner": "",
        "tags": [], "tier": "", "state": "new", "criticality": "minor", "case_key": "", "supplied_by": "",
        "question": "", "note": "",
        "output": {"symbol": "", "type": "", "unit": "", "lower": "", "upper": "", "reason_lower": "", "reason_upper": ""},
        "relation": {"expression": "", "source": "", "why": "", "reading": ""},
        "inputs": [], "steps": [], "theory": [], "assumptions": [],
        "value": "", "sense": "", "evidence": {"metric": "", "rungs": []},
        "zero_when_absent": [], "bundles": [], "fixtures": [], "fixture_wanted": "",
        "explain": {k: ([] if k == "why_chain" else "") for k in EXPLAIN_KEYS},
        "versions": [], "beliefs": [], "risks": [],
    }


# A node's own explanation (SPEC.md §5.12.3), shown as its document's station.
EXPLAIN_KEYS = ["simply", "one_line", "wrong_idea", "wrong_because", "contrast", "analogy", "analogy_breaks", "why_chain"]
# The seven areas a belief or a risk belongs to, and a belief's statuses (SPEC.md §5.13).
AREAS = ["node", "input", "output", "model", "math", "algorithm", "visualisation"]
BELIEF_STATUSES = ["broke", "held", "untested"]


def load_ledger(root):
    """derisk/risks.toml and derisk/beliefs/*.toml, read-only."""
    out = {"risks": {}, "beliefs": []}
    rp = os.path.join(root, "derisk", "risks.toml")
    if os.path.exists(rp):
        out["risks"] = {r["id"]: r for r in toml(rp).get("risk", [])}
    bd = os.path.join(root, "derisk", "beliefs")
    if os.path.isdir(bd):
        out["beliefs"] = [toml(os.path.join(bd, f)) for f in sorted(os.listdir(bd)) if f.endswith(".toml")]
    return out


# The part of a node a form may propose. Everything else is placement the tree
# owns (id, layer, group, kind, owner, state) or is computed (consumers, kpis).
EDITABLE = ["label", "tags", "tier", "criticality", "question", "note", "output", "relation", "inputs", "steps",
            "theory", "assumptions", "value", "sense", "evidence", "zero_when_absent", "bundles", "fixtures", "explain"]


def proposal_from(node):
    return {k: copy.deepcopy(node.get(k)) for k in EDITABLE}


AGENT_NAMES = re.compile(r"\b(claude|chatgpt|gpt|copilot|gemini|llama|assistant|agent|bot|ai|llm|openai|anthropic)\b", re.I)


def is_agent_name(s):
    return bool(AGENT_NAMES.search(s or ""))
