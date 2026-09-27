#!/usr/bin/env python3
"""Intake: how a node form becomes software (SPEC.md §5.11). The package's stand-in
for `cargo xtask intake` in the built repository, run by the developer team.

    python3 tools/intake.py check  <form.html> [--seed] [--out <dir>]
    python3 tools/intake.py sheet  <form.html> [--seed] --out <dir>
    python3 tools/intake.py verify <form.html> <node.toml> [<fixtures.toml>] [--seed] [--ledger <dir>]
    python3 tools/intake.py reply  <form.html> --status <status> [--note <text>] [--release <v>]
                                   [--report <check.json>] --out <file.html>
    python3 tools/intake.py codes
    python3 tools/intake.py selftest

check   The checker. It reads ONLY the JSON block of the form (it never opens the
        page), compares the request with the node as the plan has it — never with
        what the form says the node was — and applies every rule in CHECKS. It
        prints the report and exits 0 when the request may go to implementation,
        1 when it may not. With --out it writes <out>/<request id>/: request.json
        (the trusted part), check.md and check.json, the attachments, and — only on
        a pass — brief.md, the implementation agent's brief.
sheet   Writes node.toml and fixtures.toml exactly as the request says, in VLEO's
        sheet shape. In the built repository this is `cargo xtask intake write`, the
        one path by which a request's declarative content reaches a sheet; the
        implementation agent runs it and then writes only the code.
verify  The checker again, after implementation: the sheet (and its fixtures) must
        say exactly what the request says, field by field; with --ledger, so must the
        belief record, the version entry and the risk moves `sheet` wrote there. Exit 1
        on any difference.
reply   Writes the developer team's reply into the form's history and saves the
        copy that goes back to the requester.
codes   Lists every check with its code, for the manual.
selftest  Applies each deliberate mistake in MUTATIONS to a real form and requires
        the checker to refuse it by its code; then checks that clean forms pass and
        that sheet -> verify round-trips.

`--seed` accepts a seed form (request type "seed"): the package's own seed content,
which only the developer team takes through intake, at the first build.

Differences from the repository's `cargo xtask intake`: this stand-in checks against
the plan (tree.json, seed content, units.toml, physics.toml), not against sheets;
`sheet` and `verify` read the form file where the repository reads request.json;
`mark` (state), `issue` (feedback to issues) and verify's check of each HOLE's call
exist only in the repository, and N01's "declared in layers/" case needs layers/.
`sheet` writes what `intake write` adds to the ledger (SPEC.md §5.13) as three files
beside the sheet (belief.toml, versions.entry.toml, risk_moves.toml) instead of
writing into derisk/ and the node folder, which exist only in the repository.
"""

import base64
import copy
import datetime
import json
import os
import re
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import plan_model as pm  # noqa: E402

SCHEMA = "adcs-node-form/1"
OPEN = '<script type="application/json" id="adcs-node-form">'
MAX_FILE = 25 * 1024 * 1024
MAX_ATTACH = 5 * 1024 * 1024
TYPES = ["change", "new", "confirm", "feedback", "other", "seed"]
STATUSES = ["received", "check failed", "check passed", "needs information", "in progress", "in review",
            "released", "rejected", "feedback noted"]
MATH_NAMES = {"pi", "e", "sqrt", "sin", "cos", "tan", "asin", "acos", "atan", "atan2", "exp", "ln", "log", "log10",
              "abs", "min", "max", "pow", "sum", "rss", "sinh", "cosh", "tanh", "floor", "ceil", "sign", "deg", "rad"}

# Every rule, by code. The page applies the ones it can see; this is the authority.
CHECKS = {
    "F01": "the file holds an adcs-node-form/1 JSON block and nothing larger than 25 MB",
    "F02": "the request type is change, new, confirm, feedback or other (seed only with --seed)",
    "F03": "the node named exists and is offered for change (not a door, interface or closure), or the new node's id is free, well formed and prefixed for its layer (mgt_, sys_, l3_<sid>_)",
    "F04": "a change or confirmation was exported from the node as the software has it now (its base is current); feedback may come on an older copy",
    "F05": "a change changes something; a confirmation changes nothing",
    "P01": "the requester is a person, named, and not a tool or an assistant",
    "P02": "a change, a new node and a confirmation say why",
    "P03": "whoever is named as having checked the maths is a person, not a tool or an assistant (a seed form names nobody)",
    "P04": "a confirmation names who checked the node",
    "N01": "a new node's group exists (holding nodes, or declared in layers/ with none yet), and its model sibling is in that group",
    "N02": "a new node's kind is computed, declared, required or achieved",
    "I01": "the kind, layer, group, owner and id of an existing node do not change through a form",
    "I02": "a node fed by the case format keeps its label, quantity and unit (a case-format change is its own process)",
    "I03": "hardware tags are mtq, rw, fmr or rcs",
    "O01": "the answer has a symbol of letters, digits and _",
    "O02": "the answer's quantity is one the software knows",
    "O03": "the answer's unit is one the software knows, and it states that quantity",
    "O04": "the lowest and highest values are numbers, in order, and each has a written reason",
    "C01": "a computed node reads at least one input; any other kind reads none",
    "C02": "each input has a unique binding name and reads a node that exists, other than itself",
    "C03": "each input's declared quantity is what its producer publishes",
    "C04": "inputs stay inside the node's layer (layers meet only at the door and the interface nodes)",
    "C05": "the new connections make no loop",
    "C06": "zero-when-absent names hardware count nodes the node reads",
    "C07": "an input's producer has its answer specified (warning: implementation waits for it)",
    "R01": "a computed node has a relation, a source, a reason why, and at least one step",
    "R02": "each step says what it does and names a physics function that exists, or describes the new one it needs",
    "R03": "the last step binds the answer's symbol",
    "R04": "the relation's names are the bindings, the step results and ordinary maths (warning)",
    "R05": "a named physics function takes as many arguments as the node has inputs (warning)",
    "R06": "each assumption says what it assumes and when that stops being true",
    "R07": "each derivation step has text",
    "V01": "a declared node that no case, product, tuning, evidence, ledger or lab file supplies has a numeric value inside its bounds, and a source; a product or tuned row may leave its reference value blank; a value the evidence tooling, the de-risking ledger or a lab file supplies is never set by a form",
    "V02": "a requirement says which way it binds (<= or >=)",
    "V03": "an evidence node names a metric the simulator computes and at least one rung",
    "S01": "every cited source is known, or added in the form with an id, a title and exactly where",
    "T01": "a test vector's provenance is independent-derivation, published-source, independent-tool or physical-bound",
    "T02": "a test vector cites a known source and the page, table or figure",
    "T03": "a test vector gives exactly one numeric input per binding, a numeric expected answer and a tolerance above zero",
    "T04": "a computed node has at least one test vector (warning: validation stays low)",
    "A01": "attachments are named files, 5 MB in all",
    "B01": "feedback entries say what happened",
    "B02": "an 'other' request has a subject and a description",
    "W01": "nobody is named as having checked the maths: the node will run as UNCONFIRMED (warning)",
    "D01": "a change or a new node carries its belief record: its area (node, input, output, model, math, algorithm, visualisation), what was believed, what we now know, and what changed in the plan",
    "D02": "a change says what was wrong with the version it replaces and what this version gives",
    "D03": "the belief's status is broke, held or untested; broke and held say what tested it; untested says the test that would settle it",
    "D04": "each risk it moves is in the register at the level it moves from, or is new (from 0) with a title, an area, a level 1 to 5 and the test that would close it",
    "D05": "a risk is lowered or closed only by a tested belief (broke or held)",
    "D06": "the cost is blank or a number, zero or more, in thousands of dollars",
    "D07": "the belief is untested: it is counted under Beliefs not yet tested until a test is recorded (warning)",
    "X01": "an analogy says where it stops being true",
    "X02": "a common wrong idea says why it is wrong",
    "X03": "a computed node says itself simply and in one line (warning: its document otherwise serves experts only)",
}
WARNINGS = {"C07", "R04", "R05", "T04", "W01", "D07", "X03"}


# ------------------------------------------------------------------ reading
def read_form(path):
    if os.path.getsize(path) > MAX_FILE:
        raise ValueError("F01: %s is larger than 25 MB" % path)
    text = open(path, encoding="utf-8").read()
    i = text.find(OPEN)
    if i < 0:
        raise ValueError("F01: %s holds no adcs-node-form block; it is not a node form" % path)
    j = text.find("</script>", i)
    try:
        form = json.loads(text[i + len(OPEN):j])
    except json.JSONDecodeError as e:
        raise ValueError("F01: the JSON block does not parse: %s" % e)
    if form.get("schema") != SCHEMA:
        raise ValueError("F01: schema %r; the checker reads %s" % (form.get("schema"), SCHEMA))
    return form, text


def request_id(form):
    r = form.get("request", {})
    target = (form.get("node") or {}).get("tree_id") or r.get("placement", {}).get("id") or "new"
    day = datetime.date.today().isoformat()
    body = {k: v for k, v in r.items() if k != "attachments"}
    return "REQ-%s-%s-%s" % (day, re.sub(r"[^a-z0-9_]", "", str(target).lower())[:40], pm.digest(body, 6))


def s(v):
    return "" if v is None else str(v)


def num(v):
    if isinstance(v, bool) or v is None or v == "":
        return None
    try:
        x = float(v)
    except (TypeError, ValueError):
        return None
    return x if x == x and x not in (float("inf"), float("-inf")) else None


def as_list(v):
    if isinstance(v, str):
        return [x.strip() for x in v.split(",") if x.strip()]
    return list(v or [])


# ------------------------------------------------------------------ the check
class Finding:
    def __init__(self, code, where, text):
        self.code, self.where, self.text = code, where, text
        self.level = "warning" if code in WARNINGS else "error"

    def as_dict(self):
        return {"code": self.code, "level": self.level, "where": self.where, "text": self.text}


def diff_nodes(proposed, current):
    """Field-level differences between two node dicts, over what a form may propose."""
    out = []

    def norm(v):
        # JSON written by a browser drops ".0": 6440.0 comes back as 6440. Compare numbers as numbers, at any depth.
        if isinstance(v, (int, float)) and not isinstance(v, bool):
            return float(v)
        if isinstance(v, dict):
            return {k: norm(x) for k, x in v.items()}
        if isinstance(v, list):
            return [norm(x) for x in v]
        return v
    for k in pm.EDITABLE:
        a, b = proposed.get(k), current.get(k)
        if isinstance(a, dict) and isinstance(b, dict):
            for kk in sorted(set(a) | set(b)):
                x, y = a.get(kk, ""), b.get(kk, "")
                if json.dumps(norm(x), sort_keys=True) != json.dumps(norm(y), sort_keys=True) and not (s(x) == "" and s(y) == ""):
                    out.append(("%s.%s" % (k, kk), y, x))
        else:
            if k == "zero_when_absent":
                a, b = as_list(a), as_list(b)
            empty = lambda v: v in (None, "", [], {})
            if empty(a) and empty(b):
                continue
            if json.dumps(norm(a), sort_keys=True) != json.dumps(norm(b), sort_keys=True):
                out.append((k, b, a))
    return out


def derisk_checks(R, t, plan, add, info):
    """D01-D07: the request's belief record (SPEC.md §5.13). Required for a change
    and a new node; checked when present on a confirmation."""
    D = R.get("derisk") or {}
    filled = any(s(v).strip() for k, v in D.items() if k != "moves") or D.get("moves")
    if t in ("confirm",) and not filled:
        return
    if not filled or not s(D.get("believed")).strip():
        add("D01", "derisk", "the request carries no belief record: say what was believed, and what we now know")
    if D.get("area") not in pm.AREAS:
        add("D01", "derisk", "the belief's area %r is not one of %s" % (D.get("area"), ", ".join(pm.AREAS)))
    for f, what in (("now_know", "what we now know"), ("plan_change", "what changed in the plan")):
        if not s(D.get(f)).strip():
            add("D01", "derisk", "the belief record does not say %s" % what)
    if t == "change" and (not s(D.get("previous_issue")).strip() or not s(D.get("benefit")).strip()):
        add("D02", "derisk", "a change says what was wrong with the version it replaces, and what this version gives")
    st = D.get("status")
    if st not in pm.BELIEF_STATUSES:
        add("D03", "derisk", "the belief's status %r is not broke, held or untested" % st)
    elif st in ("broke", "held") and not s(D.get("tested")).strip():
        add("D03", "derisk", "a %s belief says what tested it" % st)
    elif st == "untested" and not s(D.get("would_test")).strip():
        add("D03", "derisk", "an untested belief says the test that would settle it")
    elif st == "untested":
        add("D07", "derisk", "the belief is untested: it is counted under Beliefs not yet tested until a test is recorded")
    c = D.get("cost_k", "")
    if s(c).strip() != "" and (num(c) is None or num(c) < 0):
        add("D06", "derisk", "the cost %r is not a number, zero or more" % c)
    risks = plan.derisk["risks"]
    for i, m in enumerate(D.get("moves") or [], 1):
        rid, a, z = s(m.get("risk")).strip(), m.get("from"), m.get("to")
        if num(a) is None or num(z) is None or not (0 <= num(a) <= 5 and 0 <= num(z) <= 5) or num(a) != int(num(a)) or num(z) != int(num(z)):
            add("D04", "derisk", "move %d: from and to are whole levels 0 to 5" % i)
            continue
        a, z = int(num(a)), int(num(z))
        if a == z:
            add("D04", "derisk", "move %d: a move changes the level" % i)
        if rid and rid in risks:
            if risks[rid].get("level") != a:
                add("D04", "derisk", "move %d: %s is at L%s in the register, not L%d" % (i, rid, risks[rid].get("level"), a))
        elif a == 0 and z > 0:
            if rid and not re.fullmatch(r"R-\d{2,}", rid):
                add("D04", "derisk", "move %d: a new risk's id is left blank (intake assigns it) or is R-nn" % i)
            for f in ("title", "closing_test"):
                if not s(m.get(f)).strip():
                    add("D04", "derisk", "move %d: a new risk needs its %s" % (i, f.replace("_", " ")))
            if m.get("area") not in pm.AREAS:
                add("D04", "derisk", "move %d: a new risk's area %r is not one of the seven" % (i, m.get("area")))
        else:
            add("D04", "derisk", "move %d: %r is not in the register" % (i, rid))
        if z < a and st not in ("broke", "held"):
            add("D05", "derisk", "move %d lowers %s, but the belief is %s: a level goes down only by a test" % (i, rid or "a risk", st or "unstated"))
        if z < a and st in ("broke", "held") and not s(D.get("tested")).strip():
            add("D05", "derisk", "move %d lowers %s without saying what was tested" % (i, rid))
    info["derisk"] = D


def check(form, plan=None, seed_ok=False):
    plan = plan or pm.Plan()
    F = []
    add = lambda code, where, text: F.append(Finding(code, where, text))
    R = form.get("request") or {}
    P = R.get("proposed") or {}
    t = R.get("type")
    node_in_form = form.get("node")
    info = {"type": t, "target": None, "tree_id": None, "kind": None, "layer": None, "owner": None, "changes": [],
            "inputs_added": [], "inputs_removed": [], "impact": [], "kpis": [], "new_functions": [], "sources_added": []}

    if t not in TYPES or (t == "seed" and not seed_ok):
        add("F02", "request", "request type %r; a form asks for change, new, confirm, feedback or other%s"
            % (t, "" if t != "seed" else " (a seed form is taken through intake only by the developer team, with --seed)"))
        return F, info

    # --------------------------------------------------------- who
    if not s(R.get("requested_by")).strip():
        add("P01", "who", "no requester is named")
    elif pm.is_agent_name(R.get("requested_by")) and not (t == "seed" and seed_ok):
        add("P01", "who", "the requester %r is a tool or an assistant, not a person" % R.get("requested_by"))
    if t in ("change", "new", "confirm", "seed") and not s(R.get("reason")).strip():
        add("P02", "who", "the request does not say why")
    if R.get("attested_by") and pm.is_agent_name(R.get("attested_by")):
        add("P03", "who", "%r is named as having checked the maths; that must be a person" % R.get("attested_by"))
    if t == "confirm" and not s(R.get("attested_by")).strip():
        add("P04", "who", "a confirmation must name who checked the node")
    for i, fb in enumerate(form.get("feedback") or [], 1):
        if not s(fb.get("observed")).strip():
            add("B01", "feedback", "feedback entry %d does not say what happened" % i)
    if t == "feedback" and not (form.get("feedback") or []):
        add("B01", "feedback", "a feedback request carries no feedback entry")
    if t == "other":
        o = R.get("other") or {}
        if not s(o.get("subject")).strip() or not s(o.get("description")).strip():
            add("B02", "other", "an 'other' request needs a subject and a description")
    total = 0
    for a in R.get("attachments") or []:
        total += int(a.get("size") or 0)
        if not a.get("name") or "/" in a["name"] or "\\" in a["name"] or a["name"].startswith("."):
            add("A01", "attach", "attachment name %r is not a plain file name" % a.get("name"))
        if not s(a.get("data")).startswith("data:"):
            add("A01", "attach", "attachment %r carries no data" % a.get("name"))
    if total > MAX_ATTACH:
        add("A01", "attach", "attachments total %.1f MB; the limit is 5 MB" % (total / 1048576))

    # --------------------------------------------------------- the target
    current = None
    if t in ("change", "confirm", "feedback", "seed") or (t == "other" and node_in_form):
        ref = (node_in_form or {}).get("id") or (node_in_form or {}).get("tree_id")
        tid = plan.resolve(ref) if ref else None
        if not tid or tid in plan.groups:
            if t in ("change", "confirm", "seed"):
                add("F03", "request", "the node %r is not a node the software has" % ref)
                return F, info
        else:
            current = plan.skeleton(tid) if t == "seed" else plan.current(tid)
            info.update({"target": current["id"], "tree_id": tid, "kind": current["kind"], "layer": current["layer"],
                         "owner": current["owner"]})
            if form.get("base") != pm.base_of(current) and t in ("change", "confirm", "seed"):
                was = node_in_form or {}
                moved = [k for k, _, _ in diff_nodes(pm.proposal_from(current), pm.proposal_from(was))] if was else []
                add("F04", "request", "the node has changed since this form was exported (base %s, now %s)%s; "
                    "export a fresh form and carry the request over" % (form.get("base"), pm.base_of(current),
                                                                         ": " + ", ".join(moved) if moved else ""))
    if t in ("feedback", "other"):
        return F, info
    if current is not None and current["kind"] not in pm.KINDS:
        add("F03", "request", "%s is a %s row: its content is fixed by the tree's shape, not offered for change; send feedback instead"
            % (current["id"], current["kind"]))
        return F, info
    if t == "seed" and s(R.get("attested_by")).strip():
        add("P03", "who", "a seed form never carries an attestation: the package's author is not a person who can attest")

    kind = current["kind"] if current else None
    if t == "new":
        pl = R.get("placement") or {}
        g = plan.resolve(pl.get("parent")) if pl.get("parent") else None
        if not g or g not in plan.groups or not plan.group_leaves(g):
            add("N01", "place", "the group %r does not exist or holds no nodes" % pl.get("parent"))
        elif pl.get("like") and plan.resolve(pl["like"]) not in plan.group_leaves(g):
            add("N01", "place", "the model sibling %r is not a node of %s" % (pl.get("like"), pl.get("parent")))
        nid = s(pl.get("id"))
        if not re.fullmatch(r"(mgt|sys|l3)_[a-z0-9_]+", nid):
            add("F03", "place", "the new id %r must be lower-case letters, digits and _, starting mgt_, sys_ or l3_" % nid)
        elif plan.resolve(nid):
            add("F03", "place", "a node %s already exists" % nid)
        if pl.get("kind") not in pm.KINDS:
            add("N02", "place", "kind %r is not one of %s" % (pl.get("kind"), ", ".join(pm.KINDS)))
        kind = pl.get("kind")
        if g and g in plan.rows:
            info.update({"target": nid, "tree_id": None, "kind": kind, "layer": plan.layer(g) if g in plan.rows else None,
                         "owner": plan.owner_in(g), "group": plan.node_id(g)})
        if g and nid and not plan.resolve(nid) and g in plan.rows:
            prefix = "mgt" if plan.layer(g) == 1 else "sys"
            if not nid.startswith(prefix + "_"):
                add("F03", "place", "a node in layer %d takes an id starting %s_" % (plan.layer(g), prefix))

    # --------------------------------------------------------- what changes
    if current is not None:
        changes = diff_nodes(P, current)
        info["changes"] = changes
        if t == "change" and not changes:
            add("F05", "request", "the request changes nothing; ask for a confirmation or send feedback instead")
        if t == "confirm" and changes:
            add("F05", "request", "a confirmation changes nothing, but this one changes %s; ask for a change instead"
                % ", ".join(c[0] for c in changes))
        for k in ("id", "tree_id", "kind", "layer", "group", "owner"):
            if node_in_form and k in node_in_form and s(node_in_form.get(k)) != s(current.get(k)) and t != "seed":
                add("I01", "request", "the form's copy of the node says %s = %r, the software %r" % (k, node_in_form.get(k), current.get(k)))
        if current.get("case_key"):
            o, c = P.get("output") or {}, current.get("output") or {}
            if s(P.get("label")) != s(current.get("label")):
                add("I02", "question", "%s is fed by the case key %s: its label is part of the case format" % (current["id"], current["case_key"]))
            if c.get("unit") and (s(o.get("unit")) != s(c.get("unit")) or s(o.get("type")) != s(c.get("type"))):
                add("I02", "answer", "%s is fed by the case key %s: its quantity and unit are part of the case format" % (current["id"], current["case_key"]))
    else:
        info["changes"] = diff_nodes(P, pm.proposal_from(pm.empty_node()))
    if t in ("change", "new", "confirm"):
        derisk_checks(R, t, plan, add, info)
    if t == "confirm":
        return F, info

    # --------------------------------------------------------- the node as proposed
    for tag in as_list(P.get("tags")):
        if tag not in pm.TAGS:
            add("I03", "question", "hardware tag %r is not one of %s" % (tag, ", ".join(pm.TAGS)))
    o = P.get("output") or {}
    if not re.fullmatch(r"[A-Za-z][A-Za-z0-9_]*", s(o.get("symbol"))):
        add("O01", "answer", "the answer's symbol %r must be letters, digits and _" % o.get("symbol"))
    if o.get("type") not in plan.quantities:
        add("O02", "answer", "quantity %r is not one the software knows" % o.get("type"))
    u = plan.units.get(o.get("unit"))
    if not u:
        add("O03", "answer", "unit %r is not one the software knows" % o.get("unit"))
    elif o.get("type") in plan.quantities and o["type"] not in u["quantity"]:
        add("O03", "answer", "%s states %s, not %s" % (o["unit"], "/".join(u["quantity"]), o["type"]))
    lo, hi = o.get("lower", ""), o.get("upper", "")
    for name, v, reason in (("lowest", lo, o.get("reason_lower")), ("highest", hi, o.get("reason_upper"))):
        if s(v) != "" and num(v) is None:
            add("O04", "answer", "the %s value %r is not a number" % (name, v))
        elif s(v) != "" and not s(reason).strip():
            add("O04", "answer", "the %s value has no written reason" % name)
    if num(lo) is not None and num(hi) is not None and not num(lo) < num(hi):
        add("O04", "answer", "the lowest value %s must be below the highest %s" % (lo, hi))
    ex = P.get("explain") or {}
    if s(ex.get("analogy")).strip() and not s(ex.get("analogy_breaks")).strip():
        add("X01", "explain", "the analogy does not say where it stops being true")
    if s(ex.get("wrong_idea")).strip() and not s(ex.get("wrong_because")).strip():
        add("X02", "explain", "the common wrong idea does not say why it is wrong")
    if kind == "computed" and (not s(ex.get("simply")).strip() or not s(ex.get("one_line")).strip()):
        add("X03", "explain", "the node does not say itself simply and in one line: its document serves experts only")

    sources = set(plan.sources)
    for i, x in enumerate(R.get("new_sources") or [], 1):
        if not re.fullmatch(r"[a-z0-9_]+", s(x.get("id"))):
            add("S01", "sources", "new source %d: id %r must be lower-case letters, digits and _" % (i, x.get("id")))
        elif x["id"] in plan.sources:
            add("S01", "sources", "new source %s already exists" % x["id"])
        elif not s(x.get("title")).strip() or not s(x.get("where")).strip():
            add("S01", "sources", "new source %s must say what it is and exactly where" % x["id"])
        else:
            sources.add(x["id"])
            info["sources_added"].append(x["id"])

    inputs = P.get("inputs") or []
    if kind == "computed" and not inputs:
        add("C01", "inputs", "a computed node reads at least one input")
    if kind in ("declared", "required", "achieved") and inputs:
        add("C01", "inputs", "a %s node reads nothing, but this one names %d input(s)" % (kind, len(inputs)))
    bindings = []
    me = info.get("target")
    layer = info.get("layer")
    for i, inp in enumerate(inputs, 1):
        b = s(inp.get("binding"))
        if not re.fullmatch(r"[A-Za-z][A-Za-z0-9_]*", b):
            add("C02", "inputs", "input %d: binding %r must be letters, digits and _" % (i, b))
        elif b in bindings:
            add("C02", "inputs", "input %d: binding %s is used twice" % (i, b))
        bindings.append(b)
        src = plan.resolve(inp.get("from")) if inp.get("from") else None
        if not src or src in plan.groups:
            add("C02", "inputs", "input %d: %r is not a node the software has" % (i, inp.get("from")))
            continue
        if plan.node_id(src) == me:
            add("C02", "inputs", "input %d: a node cannot read itself" % i)
            continue
        if layer and plan.layer(src) != layer:
            add("C04", "inputs", "input %d: %s is in layer %d, this node in layer %d" % (i, plan.node_id(src), plan.layer(src), layer))
        q, _ = plan.output_of(src)
        if not q:
            add("C07", "inputs", "input %d: %s has no answer specified yet" % (i, plan.node_id(src)))
        elif s(inp.get("type")) != q:
            add("C03", "inputs", "input %d: %s publishes %s, not %r" % (i, plan.node_id(src), q, inp.get("type")))
    # connectivity: added and removed edges, and loops
    now_in = {x.get("from") for x in (current or {}).get("inputs", [])} if current else set()
    new_in = {plan.node_id(plan.resolve(x.get("from"))) for x in inputs if plan.resolve(x.get("from") or "")}
    info["inputs_added"] = sorted(new_in - now_in)
    info["inputs_removed"] = sorted(now_in - new_in)
    if me and current is not None:
        g = plan.edges()
        for a in list(g):
            g[a].discard(me)
        for a in new_in:
            g.setdefault(a, set()).add(me)
        seen, stack = set(), [me]
        while stack:
            x = stack.pop()
            for y in g.get(x, ()):
                if y == me:
                    add("C05", "inputs", "reading %s makes a loop: that node already depends on %s" % (", ".join(sorted(new_in & reach_back(g, me))) or "an input", me))
                    stack = []
                    break
                if y not in seen:
                    seen.add(y)
                    stack.append(y)
    for z in as_list(P.get("zero_when_absent")):
        zt = plan.resolve(z) or ""
        if plan.node_id(zt) not in new_in:
            add("C06", "steps", "zero-when-absent names %s, which the node does not read" % z)
        elif not zt.startswith("cf_"):
            add("C06", "steps", "zero-when-absent names %s, which is not a hardware count row" % z)

    rel = P.get("relation") or {}
    steps = P.get("steps") or []
    if kind == "computed":
        if not s(rel.get("expression")).strip():
            add("R01", "relation", "no relation is written")
        if not rel.get("source"):
            add("R01", "relation", "the relation cites no source")
        if not s(rel.get("why")).strip():
            add("R01", "relation", "no reason why it is this relation: the node would answer Undefined")
        if not steps:
            add("R01", "steps", "no step says how it is computed")
        binds = set(bindings)
        for i, st in enumerate(steps, 1):
            if not s(st.get("text")).strip():
                add("R02", "steps", "step %d does not say what it does" % i)
            ph = s(st.get("physics"))
            if ph and ph not in plan.physics:
                add("R02", "steps", "step %d names %s, which is not in the physics library; describe the new function instead" % (i, ph))
            if not ph and not s(st.get("new_function")).strip():
                add("R02", "steps", "step %d names no physics function and describes no new one" % i)
            if not ph and s(st.get("new_function")).strip():
                info["new_functions"].append(st["new_function"])
            if ph in plan.physics and i == len(steps) and st.get("call") != "slice" and len(plan.physics[ph]["args"]) != len(bindings):
                add("R05", "steps", "%s takes %d argument(s); the node has %d input(s)" % (ph, len(plan.physics[ph]["args"]), len(bindings)))
            if st.get("binds"):
                binds.add(st["binds"])
        if steps and s(steps[-1].get("binds")) != s(o.get("symbol")):
            add("R03", "steps", "the last step binds %r, not the answer %r" % (steps[-1].get("binds"), o.get("symbol")))
        expr = s(rel.get("expression"))
        defined = set(binds) | {s(o.get("symbol"))}
        for clause in re.split(r"[,;]", expr):
            if "=" in clause:
                defined |= set(re.findall(r"[A-Za-z_][A-Za-z0-9_]*", clause.split("=")[0]))
        unknown = sorted({n for n in re.findall(r"[A-Za-z_][A-Za-z0-9_]*", expr)} - defined - MATH_NAMES)
        if unknown:
            add("R04", "relation", "the relation uses %s, which are neither bindings nor step results" % ", ".join(unknown))
        for i, x in enumerate(P.get("theory") or [], 1):
            if not s(x.get("text")).strip():
                add("R07", "relation", "derivation step %d is empty" % i)
    elif steps:
        add("C01", "steps", "a %s node has no steps" % kind)
    for i, a in enumerate(P.get("assumptions") or [], 1):
        if not s(a.get("text")).strip() or not s(a.get("fails_when")).strip():
            add("R06", "assumptions", "assumption %d must say what it assumes and when that stops being true" % i)

    case_fed = bool(current and current.get("case_key"))
    supplied = (current or {}).get("supplied_by", "")
    if kind == "declared" and supplied in ("evidence", "derisk", "lab") and num(P.get("value")) is not None:
        add("V01", "value", "%s is supplied by %s; a form cannot set its value" % (current["id"], {
            "evidence": "the evidence tooling from finished campaigns", "derisk": "the de-risking ledger (cargo xtask derisk rollup)",
            "lab": "the lab file the rig runs from (rig/labs/, a facility measurement)"}[supplied]))
    elif kind == "declared" and supplied in ("product", "tuned") and num(P.get("value")) is None:
        pass    # a product or tuned row's sheet value is only the reference for runs without a product (SPEC.md §5.6)
    elif kind == "declared" and not case_fed and supplied not in ("evidence", "derisk", "lab"):
        v = num(P.get("value"))
        if v is None:
            add("V01", "value", "a declared node needs its number")
        else:
            if num(lo) is not None and v < num(lo):
                add("V01", "value", "the value %s is below its lowest value %s" % (P.get("value"), lo))
            if num(hi) is not None and v > num(hi):
                add("V01", "value", "the value %s is above its highest value %s" % (P.get("value"), hi))
        if not rel.get("source"):
            add("V01", "value", "the value cites no source")
    if kind == "required" and P.get("sense") not in ("<=", ">="):
        add("V02", "value", "a requirement must say which way it binds, <= or >=")
    if kind == "achieved":
        ev = P.get("evidence") or {}
        if ev.get("metric") not in plan.metrics:
            add("V03", "value", "metric %r is not one the simulator computes" % ev.get("metric"))
        if not ev.get("rungs") or any(r not in pm.RUNGS for r in ev.get("rungs")):
            add("V03", "value", "rungs %r must be one or more of %s" % (ev.get("rungs"), ", ".join(pm.RUNGS)))
    if rel.get("source") and rel["source"] not in sources:
        add("S01", "relation", "the source %r is not known and not added in the form" % rel["source"])

    fx = P.get("fixtures") or []
    for i, f in enumerate(fx, 1):
        pv = f.get("provenance")
        if pv not in pm.PROVENANCE_OK:
            add("T01", "tests", "test vector %d: provenance %r is refused%s" % (i, pv, " — a number from the code or an assistant proves nothing" if pv in pm.PROVENANCE_REFUSED else ""))
        if not f.get("source") or f["source"] not in sources:
            add("T02", "tests", "test vector %d cites %r, not a known source" % (i, f.get("source")))
        if not s(f.get("where")).strip():
            add("T02", "tests", "test vector %d gives no page, table or figure" % i)
        keys = sorted(k for k, v in (f.get("inputs") or {}).items() if s(v) != "")
        if keys != sorted(bindings) or any(num(v) is None for v in (f.get("inputs") or {}).values() if s(v) != ""):
            add("T03", "tests", "test vector %d must give one number per binding %s; it gives %s" % (i, sorted(bindings), keys))
        if num(f.get("expect")) is None:
            add("T03", "tests", "test vector %d: the expected answer is not a number" % i)
        if num(f.get("tolerance")) is None or num(f.get("tolerance")) <= 0:
            add("T03", "tests", "test vector %d: the tolerance must be a number above zero" % i)
    if kind == "computed" and not fx:
        add("T04", "tests", "no test vector: the node's validation stays low until it has one from a cited page")
    if not s(R.get("attested_by")).strip():
        add("W01", "who", "nobody is named as having checked the maths: the node runs as UNCONFIRMED")

    # --------------------------------------------------------- impact
    if me and current is not None:
        g = plan.edges()
        out, stack = [], [me]
        seen = {me}
        while stack:
            x = stack.pop()
            for y in sorted(g.get(x, ())):
                if y not in seen:
                    seen.add(y)
                    out.append(y)
                    stack.append(y)
        info["impact"] = out
        info["kpis"] = [k["id"] for k in plan.kpis_fed(current["tree_id"])]
    return F, info


def reach_back(g, target):
    """Every node that `target` reaches, i.e. nodes that depend on it."""
    seen, stack = set(), [target]
    while stack:
        x = stack.pop()
        for y in g.get(x, ()):
            if y not in seen:
                seen.add(y)
                stack.append(y)
    return seen


# ------------------------------------------------------------------ reports
def verdict(F):
    return "PASS" if not [f for f in F if f.level == "error"] else "FAIL"


def report_md(form, F, info, rid):
    R = form.get("request") or {}
    lines = ["# Intake check — %s" % rid, ""]
    who = "%s%s" % (R.get("requested_by") or "(nobody)", (" · " + R["team"]) if R.get("team") else "")
    lines += ["| | |", "|---|---|",
              "| Request | %s |" % info["type"],
              "| Node | %s%s |" % (info.get("target") or "—", " (tree row %s)" % info["tree_id"] if info.get("tree_id") else ""),
              "| Kind, layer, owner | %s · %s · %s |" % (info.get("kind") or "—", info.get("layer") or "—", info.get("owner") or "—"),
              "| Requested by | %s |" % who,
              "| Checked by | %s |" % (R.get("attested_by") or "nobody yet (UNCONFIRMED)"),
              "| Verdict | **%s** — %d error(s), %d warning(s) |" % (verdict(F), sum(f.level == "error" for f in F), sum(f.level == "warning" for f in F)),
              ""]
    if R.get("summary"):
        lines += ["**In one line (the requester's):** " + s(R["summary"]).replace("\n", " "), ""]
    if R.get("reason"):
        lines += ["**Why:** " + s(R["reason"]).replace("\n", " "), ""]
    lines += ["## Findings", ""]
    if not F:
        lines += ["None.", ""]
    else:
        lines += ["| | code | section | finding |", "|---|---|---|---|"]
        for f in sorted(F, key=lambda f: (f.level != "error", f.code)):
            lines.append("| %s | %s | %s | %s |" % ("✗" if f.level == "error" else "i", f.code, f.where, f.text.replace("|", "\\|")))
        lines.append("")
    if info.get("changes"):
        lines += ["## What changes", "", "| field | now | proposed |", "|---|---|---|"]
        short = lambda v: (json.dumps(v, ensure_ascii=False) if isinstance(v, (list, dict)) else s(v))[:160].replace("|", "\\|")
        for k, a, b in info["changes"]:
            lines.append("| %s | %s | %s |" % (k, short(a) or "(empty)", short(b) or "(empty)"))
        lines.append("")
    if info.get("inputs_added") or info.get("inputs_removed"):
        lines += ["## Connections", ""]
        lines += ["- reads, newly: %s" % x for x in info["inputs_added"]]
        lines += ["- no longer reads: %s" % x for x in info["inputs_removed"]]
        lines.append("")
    if info.get("impact") or info.get("kpis"):
        lines += ["## Impact", "", "Every node downstream re-runs its tests in the gate; these read this node's answer, directly or through others:", ""]
        lines += ["- %s" % x for x in info["impact"][:60]]
        if len(info["impact"]) > 60:
            lines.append("- …and %d more" % (len(info["impact"]) - 60))
        if info.get("kpis"):
            lines += ["", "KPIs it feeds: " + ", ".join(info["kpis"])]
        lines.append("")
    if info.get("new_functions"):
        lines += ["## New physics functions needed", ""] + ["- " + x for x in info["new_functions"]] + [""]
    D = info.get("derisk") or {}
    if s(D.get("believed")).strip():
        lines += ["## De-risking", "", "| | |", "|---|---|",
                  "| Area | %s |" % s(D.get("area")), "| Believed | %s |" % s(D.get("believed")).replace("|", "\\|"),
                  "| Status | %s |" % s(D.get("status")),
                  "| Tested | %s |" % (s(D.get("tested")) or "not yet; would be: " + s(D.get("would_test"))).replace("|", "\\|"),
                  "| Now know | %s |" % s(D.get("now_know")).replace("|", "\\|"),
                  "| Previous version's issue | %s |" % (s(D.get("previous_issue")) or "—").replace("|", "\\|"),
                  "| This version's benefit | %s |" % (s(D.get("benefit")) or "—").replace("|", "\\|"),
                  "| Risks moved | %s |" % ("; ".join("%s L%s→L%s" % (m.get("risk") or "new", m.get("from"), m.get("to")) for m in D.get("moves") or []) or "none"),
                  ""]
    lines += ["Codes are listed by `python3 tools/intake.py codes` and in the developer manual.", ""]
    return "\n".join(lines)


def brief_md(form, info, rid):
    """The implementation agent's brief: exactly what to do, exactly where, and what not to do."""
    R = form["request"]
    P = R["proposed"]
    crate = {1: "adcs-mod-management", 2: "adcs-mod-system"}.get(info.get("layer"), "adcs-mod-<subsystem>")
    steps = P.get("steps") or []
    L = ["# Implementation brief — %s" % rid, "",
         "You are implementing one checked request. The request is `request.json` beside this brief; the check report is",
         "`check.md`. Implement exactly what the request says: nothing more, nothing less.", "",
         "## Scope", "",
         "- The node: `%s`%s, kind `%s`, layer %s, owner `%s`, in crate `%s`." % (
             info["target"], " (new)" if info["type"] == "new" else "", info["kind"], info["layer"], info["owner"], crate)]
    if info["type"] == "new":
        pl = R.get("placement") or {}
        L.append("- Create it first: `cargo xtask new %s --like %s` under `%s`." % (pl.get("id"), pl.get("like") or "<a sibling in the group>", pl.get("parent")))
    L += ["- Files you may change: that node's folder (`node.toml` only through `cargo xtask intake write`, `model.rs` inside",
          "  its numbered HOLE blocks, `fixtures.toml`, `versions.toml` and `versions/` only through `cargo xtask intake write`),",
          "  and `derisk/` only through `cargo xtask intake write`%s." % (
              ", and `adcs-core/src/physics/` with its MATLAB twin `matlab_sils/+asils/+physics/` for the new function(s) "
              "listed below" if info.get("new_functions") else ""),
          "- Nothing else. Another node, a tolerance, a generated file outside a HOLE, `plan/`, the catalogue: out of scope.", "",
          "## Do, in order", "",
          "1. `cargo xtask intake write request.json` — writes the sheet and its test vectors exactly as requested; keeps the",
          "   previous sheet as `versions/<n>/` and appends the new version to `versions.toml`; writes the belief record",
          "   `derisk/beliefs/<request id>.toml` and applies its risk moves to `derisk/risks.toml` (SPEC.md §5.13).",
          "2. `cargo xtask docs %s` — regenerates the node's artefacts." % info["target"]]
    n = 3
    for i, st in enumerate(steps, 1):
        call = s(st.get("physics"))
        if call:
            args = ", ".join(x.get("binding", "") for x in (P.get("inputs") or []))
            L.append("%d. HOLE %d (%s): call `physics::%s(%s)`%s, binding `%s`." % (
                n, i, s(st.get("text")).strip(), call, "&[" + args + "]" if st.get("call") == "slice" else args,
                " as one slice" if st.get("call") == "slice" else "", s(st.get("binds")) or "(intermediate)"))
        else:
            L.append("%d. HOLE %d (%s): needs a new physics function — %s. Write it in `adcs-core::physics` in VLEO's style: "
                     "`no_std`, typed arguments, `pmath` only, a doc comment giving the relation and the source id `%s`, zero at "
                     "count zero where it takes a count, and property tests only (never an invented expected value). In the same "
                     "commit write its twin `matlab_sils/+asils/+physics/+<module>/<name>.m` (the module and name the Rust "
                     "function has) from the same relation and source, so both engines run this row's test vectors (lockstep, "
                     "SPEC.md §10.8.7). Then call it from the HOLE." % (n, i, s(st.get("text")).strip(), s(st.get("new_function")).strip(), s((P.get("relation") or {}).get("source"))))
        n += 1
    L += ["%d. `cargo xtask intake verify request.json` — the sheet must match the request field by field, and each HOLE must call "
          "the function its step names." % n,
          "%d. `cargo xtask gate` and `cargo test --workspace` — including every downstream node listed in the check report%s." % (
              n + 1, "; and `cargo xtask twin check` with the `matlab-twin` tests, so the new function(s) pass the same vectors in MATLAB"
              if any(not s(st.get("physics")) for st in steps) else ""),
          "%d. Commit on a branch named `intake/%s`, with the request's reason as the body and the trailer `Request: %s`. "
          "Copy the request file into the node folder as `requests/%s.request.html`. Do not push." % (n + 2, rid, rid, rid), "",
          "## Never", "",
          "- supply an expected value: test vectors come only from the request, which cites their pages;",
          "- write a person's name anywhere: `confirmed_by` comes from the request's `attested_by` through `intake write`, or stays UNCONFIRMED;",
          "- widen a tolerance, skip a test, or edit outside the scope above;",
          "- put a formula in a HOLE: a HOLE composes `physics::` calls on its bindings (rule 3).", "",
          "If the request cannot be implemented as written, stop and say why; the developer replies to the requester.", ""]
    return "\n".join(L)


def write_attachments(form, out):
    names = []
    for a in (form.get("request") or {}).get("attachments") or []:
        data = s(a.get("data"))
        if not data.startswith("data:") or "," not in data:
            continue
        name = re.sub(r"[^A-Za-z0-9._-]", "_", s(a.get("name")) or "attachment")
        body = data.split(",", 1)[1]
        raw = base64.b64decode(body) if ";base64" in data.split(",", 1)[0] else body.encode()
        os.makedirs(os.path.join(out, "attachments"), exist_ok=True)
        open(os.path.join(out, "attachments", name), "wb").write(raw)
        names.append(name)
    return names


# ------------------------------------------------------------------ the sheet, as intake write renders it
def q(v):
    return json.dumps(v, ensure_ascii=False)


def tnum(v):
    x = num(v)
    return repr(float(x)) if x is not None else None


def sheet_from(form, info, rid):
    """node.toml and fixtures.toml, in VLEO's sheet shape, exactly as the request
    says. The stand-in for `cargo xtask intake write`."""
    R = form["request"]
    P = R["proposed"]
    o = P.get("output") or {}
    rel = P.get("relation") or {}
    attested = s(R.get("attested_by")).strip()
    confirm = ("%s / %s (via %s)" % (attested, datetime.date.today().isoformat(), rid)) if attested else \
        "UNCONFIRMED · via %s · awaiting a person" % rid
    kind = info["kind"]
    L = ["# The sheet. Written by `cargo xtask intake write` from request %s." % rid,
         "# Change it only through a node form and intake.",
         "id = %s" % q(info["target"]), "label = %s" % q(s(P.get("label")))]
    if kind:
        # A requirement is written as a declared row with a top-level sense (SPEC.md §5.5, gate check 7d).
        L.append("kind = %s" % q("declared" if kind == "required" else kind))
    L.append('state = "specified"')
    L += ["owner = %s" % q(info.get("owner") or ""), "layer = %s" % (info.get("layer") or 0)]
    if P.get("tier"):
        L.append("tier = %s" % q(P["tier"]))
    if P.get("criticality") and P["criticality"] != "minor":
        L.append("criticality = %s" % q(P["criticality"]))
    if as_list(P.get("tags")):
        L.append("tags = [%s]" % ", ".join(q(x) for x in as_list(P.get("tags"))))
    if kind == "required":
        L.append("sense = %s" % q(P.get("sense")))
    if as_list(P.get("zero_when_absent")):
        L.append("zero_when_absent = [%s]" % ", ".join(q(x) for x in as_list(P["zero_when_absent"])))
    if P.get("bundles"):
        L.append("bundles = [%s]" % ", ".join(q(x) for x in P["bundles"]))
    L += ["", "[request]", "last = %s" % q(rid), "",
          "[question]", "text = %s" % q(s(P.get("question")))]
    if s(P.get("note")):
        L.append("note = %s" % q(P["note"]))
    if kind in ("computed", "declared") and (rel.get("expression") or rel.get("source")):
        L += ["", "[maths]", "expression = %s" % q(s(rel.get("expression"))), "source = %s" % q(s(rel.get("source"))),
              "confirmed_by = %s" % q(confirm)]
    if kind == "computed" and (rel.get("why") or rel.get("reading") or P.get("theory")):
        L += ["", "[theory]", "why = %s" % q(s(rel.get("why"))), "reading = %s" % q(s(rel.get("reading")))]
        for x in P.get("theory") or []:
            L += ["", "[[theory.step]]", "text = %s" % q(s(x.get("text")))]
            if s(x.get("math")):
                L.append("math = %s" % q(x["math"]))
    for a in P.get("assumptions") or []:
        L += ["", "[[assumption]]", "text = %s" % q(s(a.get("text"))), "fails_when = %s" % q(s(a.get("fails_when")))]
    ex = P.get("explain") or {}
    if any(s(v).strip() if not isinstance(v, list) else v for v in ex.values()):
        L += ["", "[explain]"]
        for k in pm.EXPLAIN_KEYS:
            if k == "why_chain" and ex.get(k):
                L.append("why_chain = [%s]" % ", ".join(q(s(x)) for x in ex[k]))
            elif k != "why_chain" and s(ex.get(k)).strip():
                L.append("%s = %s" % (k, q(s(ex[k]))))
    L += ["", "[output]", "symbol = %s" % q(s(o.get("symbol"))), "type = %s" % q(s(o.get("type"))), "unit = %s" % q(s(o.get("unit")))]
    for k in ("lower", "upper"):
        if tnum(o.get(k)) is not None:
            L.append("%s = %s" % (k, tnum(o.get(k))))
    for k in ("reason_lower", "reason_upper"):
        if s(o.get(k)):
            L.append("%s = %s" % (k, q(o[k])))
    if kind == "declared" and tnum(P.get("value")) is not None:
        L += ["", "[value]", "number = %s" % tnum(P["value"]), "confirmed_by = %s" % q(confirm)]
    if kind == "achieved":
        ev = P.get("evidence") or {}
        L += ["", "[evidence]", "metric = %s" % q(s(ev.get("metric"))), "rungs = [%s]" % ", ".join(q(x) for x in ev.get("rungs") or [])]
    for inp in P.get("inputs") or []:
        L += ["", "[[input]]", "binding = %s" % q(s(inp.get("binding"))), "var = %s" % q(s(inp.get("from"))), "type = %s" % q(s(inp.get("type")))]
    for i, st in enumerate(P.get("steps") or [], 1):
        L += ["", "[[algorithm.step]]", "number = %d" % i, "text = %s" % q(s(st.get("text")))]
        if s(st.get("binds")):
            L.append("binds = %s" % q(st["binds"]))
        if s(st.get("type")):
            L.append("type = %s" % q(st["type"]))
    fx = ["# Known-good values, and where each came from. Written by `cargo xtask intake write`",
          "# from request %s. An expected value is never produced by the code under test." % rid]
    for f in P.get("fixtures") or []:
        ins = ", ".join("%s = %s" % (k, tnum(v)) for k, v in sorted((f.get("inputs") or {}).items()))
        fx += ["", "[[fixture]]", "label = %s" % q(s(f.get("label"))), "expect = %s" % tnum(f.get("expect")),
               "tolerance = %s" % tnum(f.get("tolerance")), "provenance = %s" % q(s(f.get("provenance"))),
               "source = %s" % q(s(f.get("source"))), "where = %s" % q(s(f.get("where"))), "inputs = { %s }" % ins]
    return "\n".join(L) + "\n", "\n".join(fx) + "\n"


BELIEF_FIELDS = ["area", "believed", "status", "tested", "now_know", "cost_k", "plan_change", "previous_issue", "benefit", "would_test"]


def quarter_of(day):
    return "Q%d-%02d" % ((day.month - 1) // 3 + 1, day.year % 100)


def derisk_files(form, info, rid, plan):  # noqa: C901
    """What `intake write` adds besides the sheet (SPEC.md §5.13): the belief
    record derisk/beliefs/<rid>.toml, the node's next versions.toml entry, and
    the moves it applies to derisk/risks.toml. None for a request with no belief
    record (a seed form, or a confirmation that carries none)."""
    R = form["request"]
    D = R.get("derisk") or {}
    if not s(D.get("believed")).strip():
        if info.get("type") == "seed":
            # version 1, "first build": no belief record of its own (SPEC.md §5.13)
            return None, "\n".join(["", "[[version]]", "n = 1", 'release = ""', "request = %s" % q(rid), 'belief = ""',
                                    'status = ""', 'previous_issue = ""',
                                    "benefit = %s" % q("the node's first content, from the package's seed content; UNCONFIRMED until a person confirms it"),
                                    'snapshot = ""']) + "\n", ""
        return None
    day = datetime.date.today()
    Q = quarter_of(day)
    taken = set(plan.derisk["risks"])
    nxt = max([int(r[2:]) for r in taken if re.fullmatch(r"R-\d+", r)] or [0]) + 1
    moves = []
    for m in D.get("moves") or []:
        rid_m = s(m.get("risk")).strip()
        if not rid_m:
            rid_m = "R-%02d" % nxt
            nxt += 1
        moves.append(dict(m, risk=rid_m, **{"from": int(num(m.get("from"))), "to": int(num(m.get("to")))}))
    node = info.get("target") or ""
    # the node's history as the software has it, never as the form says it was (the form's copy may be edited)
    cur = plan.current(info["tree_id"]).get("versions", []) if info.get("tree_id") else []
    n = (max([int(v.get("n", 0)) for v in cur] or [0]) + 1) if info["type"] != "confirm" else None
    about = [node] + ([info["tree_id"]] if info.get("tree_id") else [])
    cost = num(D.get("cost_k"))
    B = ['schema = "adcs-belief/1"', "id = %s" % q(rid), "quarter = %s" % q(Q), "area = %s" % q(s(D.get("area"))),
         "about = [%s]" % ", ".join(q(x) for x in about), "request = %s" % q(rid),
         "version = %s" % q("%s@%d" % (node, n) if n else ""), "recorded_by = %s" % q(s(R.get("requested_by")))]
    for k in ("believed", "status", "tested", "now_know"):
        B.append("%s = %s" % (k, q(s(D.get(k)))))
    B.append("cost_k = %s" % (repr(float(cost)) if cost is not None else '""'))
    for k in ("plan_change", "previous_issue", "benefit", "would_test"):
        B.append("%s = %s" % (k, q(s(D.get(k)))))
    rs = sorted({m["risk"] for m in moves})
    B.append("risks = [%s]" % ", ".join(q(x) for x in rs))
    for m in moves:
        B += ["", "[[move]]", "risk = %s" % q(m["risk"]), "from = %d" % m["from"], "to = %d" % m["to"]]
    V = None
    if n:
        V = "\n".join(["", "[[version]]", "n = %d" % n, 'release = ""', "request = %s" % q(rid), "belief = %s" % q(rid),
                       "status = %s" % q(s(D.get("status"))), "previous_issue = %s" % q(s(D.get("previous_issue"))),
                       "benefit = %s" % q(s(D.get("benefit"))),
                       "snapshot = %s" % q("versions/%d/" % (n - 1) if n > 1 else "")]) + "\n"
    M = []
    for m in moves:
        M += ["", "[[move]]", "risk = %s" % q(m["risk"]), "quarter = %s" % q(Q), "from = %d" % m["from"], "to = %d" % m["to"], "belief = %s" % q(rid)]
        if m["from"] == 0 and m["risk"] not in taken:
            M += ["title = %s" % q(s(m.get("title"))), "area = %s" % q(s(m.get("area"))), "closing_test = %s" % q(s(m.get("closing_test"))),
                  "owner = %s" % q(info.get("owner") or "")]
    return "\n".join(B) + "\n", V, ("\n".join(M).lstrip("\n") + "\n") if M else ""


def verify_belief(form, rid, belief):
    """The belief record intake wrote, against the request's De-risking section."""
    D = (form["request"].get("derisk") or {})
    d = []
    for k in ("area", "believed", "status", "tested", "now_know", "plan_change", "previous_issue", "benefit", "would_test"):
        if s(D.get(k)) != s(belief.get(k)):
            d.append(("derisk." + k, D.get(k), belief.get(k)))
    c = num(D.get("cost_k"))
    if (c is None) != (belief.get("cost_k") in ("", None)) or (c is not None and abs(c - float(belief["cost_k"])) > 1e-9):
        d.append(("derisk.cost_k", D.get("cost_k"), belief.get("cost_k")))
    want = [(s(m.get("risk")).strip() or None, int(num(m.get("from"))), int(num(m.get("to")))) for m in D.get("moves") or []]
    got = [(m.get("risk"), m.get("from"), m.get("to")) for m in belief.get("move", [])]
    if len(want) != len(got) or any((w[0] is not None and w[0] != g[0]) or w[1:] != g[1:] for w, g in zip(want, got)):
        d.append(("derisk.moves", want, got))
    if belief.get("id") != rid:
        d.append(("derisk.id", rid, belief.get("id")))
    return d


def verify_ledger(form, info, rid, belief, version_entry, moves):
    """What intake write added to the ledger, against the request: the belief
    record, the version entry and the risk moves (SPEC.md §5.13)."""
    d = verify_belief(form, rid, belief) if belief is not None else []
    D = form["request"].get("derisk") or {}
    if s(D.get("believed")).strip() and belief is None:
        d.append(("derisk", "a belief record", "none written"))
    if version_entry is not None:
        v = (version_entry.get("version") or [{}])[0]
        for k in ("previous_issue", "benefit", "status"):
            if s(v.get(k)) != s(D.get(k)):
                d.append(("version." + k, D.get(k), v.get(k)))
        if v.get("request") != rid or v.get("belief") != rid:
            d.append(("version.request", rid, v.get("request")))
    if moves is not None:
        got = [(m.get("risk"), m.get("from"), m.get("to"), m.get("belief")) for m in moves.get("move", [])]
        want = [(s(m.get("risk")).strip() or None, int(num(m.get("from"))), int(num(m.get("to"))), rid) for m in D.get("moves") or []]
        if len(got) != len(want) or any((w[0] is not None and w[0] != g[0]) or w[1:] != g[1:] for w, g in zip(want, got)):
            d.append(("risk moves", want, got))
    return d


def confirm_patch(form, rid):
    """A confirmation writes only the attestation and the request id (SPEC.md §5.8)."""
    who = s(form["request"].get("attested_by")).strip()
    stamp = "%s / %s (via %s)" % (who, datetime.date.today().isoformat(), rid)
    return {"maths.confirmed_by": stamp, "value.confirmed_by": stamp, "request.last": rid}


def verify(form, info, sheet, fixtures):
    """The sheet (and fixtures) against the request, field by field."""
    P = form["request"]["proposed"]
    o, rel = P.get("output") or {}, P.get("relation") or {}
    d = []

    def eq(name, want, got):
        w, g = want, got
        if num(w) is not None and num(g) is not None:
            if abs(num(w) - num(g)) > 1e-12 * max(1.0, abs(num(w))):
                d.append((name, want, got))
        elif s(w) != s(g):
            d.append((name, want, got))
    eq("id", info["target"], sheet.get("id"))
    eq("label", P.get("label"), sheet.get("label"))
    eq("question", P.get("question"), (sheet.get("question") or {}).get("text"))
    eq("note", P.get("note"), (sheet.get("question") or {}).get("note", ""))
    so = sheet.get("output") or {}
    for k in ("symbol", "type", "unit", "lower", "upper", "reason_lower", "reason_upper"):
        eq("output." + k, o.get(k, ""), so.get(k, ""))
    kind = info["kind"]
    if kind in ("computed", "declared") and (rel.get("expression") or rel.get("source")):
        eq("relation.expression", rel.get("expression"), (sheet.get("maths") or {}).get("expression"))
        eq("relation.source", rel.get("source"), (sheet.get("maths") or {}).get("source"))
    if kind == "computed":
        th = sheet.get("theory") or {}
        eq("relation.why", rel.get("why"), th.get("why", ""))
        eq("relation.reading", rel.get("reading"), th.get("reading", ""))
        want = [(s(x.get("text")), s(x.get("math"))) for x in P.get("theory") or []]
        got = [(s(x.get("text")), s(x.get("math"))) for x in th.get("step") or []]
        eq("theory", json.dumps(want), json.dumps(got))
    if kind == "declared" and not (form.get("node") or {}).get("case_key"):
        eq("value", P.get("value"), (sheet.get("value") or {}).get("number"))
    if kind == "required":
        eq("sense", P.get("sense"), sheet.get("sense"))
    if kind == "achieved":
        ev = P.get("evidence") or {}
        eq("evidence.metric", ev.get("metric"), (sheet.get("evidence") or {}).get("metric"))
        eq("evidence.rungs", json.dumps(ev.get("rungs") or []), json.dumps((sheet.get("evidence") or {}).get("rungs") or []))
    eq("zero_when_absent", json.dumps(as_list(P.get("zero_when_absent"))), json.dumps(sheet.get("zero_when_absent") or []))
    eq("tags", json.dumps(as_list(P.get("tags"))), json.dumps(sheet.get("tags") or []))
    want = [(s(x.get("binding")), s(x.get("from")), s(x.get("type"))) for x in P.get("inputs") or []]
    got = [(s(x.get("binding")), s(x.get("var")), s(x.get("type"))) for x in sheet.get("input") or []]
    eq("inputs", json.dumps(want), json.dumps(got))
    want = [(s(x.get("text")), s(x.get("binds")), s(x.get("type"))) for x in P.get("steps") or []]
    got = [(s(x.get("text")), s(x.get("binds")), s(x.get("type"))) for x in (sheet.get("algorithm") or {}).get("step") or []]
    eq("steps", json.dumps(want), json.dumps(got))
    want = [(s(x.get("text")), s(x.get("fails_when"))) for x in P.get("assumptions") or []]
    got = [(s(x.get("text")), s(x.get("fails_when"))) for x in sheet.get("assumption") or []]
    eq("assumptions", json.dumps(want), json.dumps(got))
    ex, sx = P.get("explain") or {}, sheet.get("explain") or {}
    for k in pm.EXPLAIN_KEYS:
        if k == "why_chain":
            eq("explain.why_chain", json.dumps([s(x) for x in ex.get(k) or []]), json.dumps([s(x) for x in sx.get(k) or []]))
        else:
            eq("explain." + k, ex.get(k, ""), sx.get(k, ""))
    attested = s(form["request"].get("attested_by")).strip()
    cb = ((sheet.get("maths") or sheet.get("value") or {}).get("confirmed_by")) or ""
    if attested and not cb.startswith(attested + " / "):
        d.append(("confirmed_by", attested + " / …", cb))
    if not attested and cb and not cb.startswith("UNCONFIRMED"):
        d.append(("confirmed_by", "UNCONFIRMED …", cb))
    if fixtures is not None:
        canon = lambda f: json.dumps([s(f.get("label")), num(f.get("expect")), num(f.get("tolerance")), s(f.get("provenance")),
                                      s(f.get("source")), s(f.get("where")),
                                      sorted((k, num(v)) for k, v in (f.get("inputs") or {}).items())])
        eq("fixtures", json.dumps([canon(f) for f in P.get("fixtures") or []]), json.dumps([canon(f) for f in fixtures.get("fixture") or []]))
    return d


# ------------------------------------------------------------------ reply
def reply(path, status, note, release, report, out):
    form, text = read_form(path)
    if status not in STATUSES:
        raise SystemExit("status must be one of: %s" % ", ".join(STATUSES))
    entry = {"date": datetime.date.today().isoformat(), "who": "developer team", "status": status, "note": note or "",
             "release": release or ""}
    if report:
        r = json.load(open(report, encoding="utf-8"))
        entry["request_id"] = r.get("request_id", "")
        entry["findings"] = ["%s %s: %s" % (f["code"], f["level"], f["text"]) for f in r.get("findings", [])]
    form.setdefault("history", []).append(entry)
    body = json.dumps(form, ensure_ascii=False, indent=1).replace("<", "\\u003c")
    i = text.find(OPEN)
    j = text.find("</script>", i)
    open(out, "w", encoding="utf-8").write(text[:i + len(OPEN)] + "\n" + body + "\n" + text[j:])
    return out


# ------------------------------------------------------------------ commands
def run_check(path, seed_ok, out=None, quiet=False, plan=None):
    try:
        form, _ = read_form(path)
    except ValueError as e:
        code, msg = str(e).split(": ", 1)
        if not quiet:
            print("FAIL  %s  %s" % (code, msg))
        return 1, [Finding(code, "file", msg)], {}, None
    F, info = check(form, plan, seed_ok)
    rid = request_id(form)
    md = report_md(form, F, info, rid)
    if not quiet:
        print(md)
    if out:
        d = os.path.join(out, rid)
        os.makedirs(d, exist_ok=True)
        nd = form.get("node") or {}
        trusted = {"schema": form["schema"], "request_id": rid, "base": form.get("base"),
                   "node": {"id": nd.get("id"), "tree_id": nd.get("tree_id"), "kind": info.get("kind"), "layer": info.get("layer"),
                            "owner": info.get("owner")} if nd else {"id": info.get("target"), "kind": info.get("kind")}, "request": {k: v for k, v in form["request"].items() if k != "attachments"},
                   "feedback": form.get("feedback", [])}
        json.dump(trusted, open(os.path.join(d, "request.json"), "w", encoding="utf-8"), ensure_ascii=False, indent=1)
        open(os.path.join(d, "check.md"), "w", encoding="utf-8").write(md)
        json.dump({"request_id": rid, "verdict": verdict(F), "findings": [f.as_dict() for f in F],
                   "changes": [[a, b, c] for a, b, c in info.get("changes", [])], "impact": info.get("impact", []),
                   "info": {k: v for k, v in info.items() if k not in ("changes",)}},
                  open(os.path.join(d, "check.json"), "w", encoding="utf-8"), ensure_ascii=False, indent=1, default=str)
        write_attachments(form, d)
        if verdict(F) == "PASS" and info.get("type") in ("change", "new", "seed"):
            open(os.path.join(d, "brief.md"), "w", encoding="utf-8").write(brief_md(form, info, rid))
        if verdict(F) == "PASS" and info.get("type") == "confirm":
            p = confirm_patch(form, rid)
            open(os.path.join(d, "confirm.md"), "w", encoding="utf-8").write(
                "# Confirmation — %s\n\nNo code changes and no agent. A developer runs `cargo xtask intake write request.json`, "
                "which in confirm mode writes only these fields of `%s`, then `intake verify` and the gate:\n\n%s\n"
                % (rid, info["target"], "\n".join("- `%s` = `%s`" % (k, v) for k, v in p.items())))
        if not quiet:
            print("wrote %s/" % d)
    return (0 if verdict(F) == "PASS" else 1), F, info, form


def selftest():
    import tomllib
    plan = pm.Plan()
    sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
    import forms
    tmp = tempfile.mkdtemp(prefix="adcs-intake-")

    def form_for(tree_id, mode="change"):
        return json.loads(json.dumps(forms.node_form(plan, tree_id, mode)))

    def save(f, name):
        p = os.path.join(tmp, name)
        open(p, "w", encoding="utf-8").write(forms.fill(forms.NODE_TEMPLATE, f["title"], f, forms.NODE_OPEN))
        return p

    def person(f, reason="the bore was measured again"):
        f["request"].update({"requested_by": "A. Engineer", "team": "GNC", "reason": reason})
        return f

    def belief(f, **kw):
        D = {"area": "math", "believed": "The node's users knew it holds for laminar flow only, so its note did not need to say so.",
             "status": "broke", "tested": "A reader applied the node to a turbulent ring and the result was far too long (selftest data).",
             "now_know": "Readers take the node's scope from its note, not from its assumptions.", "cost_k": 0,
             "plan_change": "The scope is written on the node's note.",
             "previous_issue": "The note did not say the node holds for laminar flow only.",
             "benefit": "A reader sees the node's scope before using it.", "would_test": "",
             "moves": []}
        D.update(kw)
        f["request"]["derisk"] = D
        return f

    def change_gf7():
        f = belief(person(form_for("gf_7")))
        f["request"]["proposed"]["note"] = "Laminar flow only."
        return f

    def change_gf7_moving():
        f = change_gf7()
        f["request"]["derisk"]["moves"] = [{"risk": "R-04", "from": 4, "to": 3},
                                           {"risk": "", "from": 0, "to": 2, "title": "A node's scope is misread", "area": "visualisation",
                                            "closing_test": "a reader test on the node library"}]
        return f
    downstream = None
    for a, b, _ in plan.tree["VE"]:
        if a == "gf_6":
            downstream = plan.node_id(b)
            break

    def m(fn):
        def go():
            f = change_gf7()
            fn(f)
            return f
        return go

    def setp(path, v):
        def fn(f):
            o = f["request"]["proposed"]
            ks = path.split(".")
            for k in ks[:-1]:
                o = o[k]
            o[ks[-1]] = v
        return m(fn)

    def new_form(**pl):
        def go():
            f = json.loads(json.dumps(forms.new_node_form(plan)))
            person(f)
            f["request"]["placement"].update(pl)
            return f
        return go
    def req_form(tid):
        f = person(form_for(tid))
        return f
    MUT = [
        ("F01", "a file that is not a node form", m(lambda f: f.update({"schema": "adcs-form/1"})), False),
        ("V02", "a requirement without its sense", lambda: (lambda f: (f["request"]["proposed"].update({"sense": ""}), f)[-1])(req_form("p1k_0")), False),
        ("V03", "evidence from a metric the simulator does not compute", lambda: (lambda f: (f["request"]["proposed"]["evidence"].update({"metric": "happiness"}), f)[-1])(req_form("p1a_0")), False),
        ("V01", "a form setting a facility capability the lab file supplies", lambda: (lambda f: (f["request"]["proposed"].update({"value": 1e-4}), f)[-1])(req_form("fa3_0")), False),
        ("V01", "a form setting a ledger-supplied risk count", lambda: (lambda f: (f["request"]["proposed"].update({"value": 2}), f)[-1])(req_form("rk1_0")), False),
        ("V01", "a form setting an evidence-supplied value", lambda: (lambda f: (f["request"]["proposed"].update({"value": 3.0}), f)[-1])(req_form("v1_0")), False),
        ("C06", "zero-when-absent naming a row that is not a count", m(lambda f: f["request"]["proposed"].update({"zero_when_absent": [f["request"]["proposed"]["inputs"][0]["from"]]})), False),
        ("F02", "seed form without --seed", lambda: form_for("gf_7", "seed"), False),
        ("F02", "unknown request type", m(lambda f: f["request"].update({"type": "rewrite"})), False),
        ("F04", "stale base", m(lambda f: f.update({"base": "0" * 16})), False),
        ("F05", "a change that changes nothing", lambda: person(form_for("gf_7")), False),
        ("F05", "a confirmation that changes something", m(lambda f: f["request"].update({"type": "confirm", "attested_by": "A. Engineer"})), False),
        ("P01", "no requester", m(lambda f: f["request"].update({"requested_by": ""})), False),
        ("P01", "an assistant as requester", m(lambda f: f["request"].update({"requested_by": "Claude"})), False),
        ("P02", "no reason", m(lambda f: f["request"].update({"reason": ""})), False),
        ("P03", "an assistant as checker", m(lambda f: f["request"].update({"attested_by": "ChatGPT assistant"})), False),
        ("F03", "a change to the door", lambda: person(dict(form_for("ci1_1"), request=dict(form_for("ci1_1")["request"], type="confirm", attested_by="A. Engineer"))), False),
        ("P03", "a seed form with an attestation", lambda: (lambda f: (f["request"].update({"attested_by": "A. Engineer"}), f)[-1])(form_for("gf_7", "seed")), True),
        ("P04", "confirmation without checker", lambda: person(dict(form_for("gf_7"), request=dict(form_for("gf_7")["request"], type="confirm"))), False),
        ("I01", "the form claims another kind", m(lambda f: f["node"].update({"kind": "declared"})), False),
        ("I02", "label change on a case-fed node", lambda: (lambda f: (person(f), f["request"]["proposed"].update({"label": "Orbit tilt"}), f)[-1])(form_for("m2_1")), False),
        ("I03", "unknown hardware tag", setp("tags", ["gyro"]), False),
        ("O01", "bad symbol", setp("output.symbol", "t spin"), False),
        ("O02", "unknown quantity", setp("output.type", "Duration"), False),
        ("O03", "unit that does not state the quantity", setp("output.unit", "Kilogram"), False),
        ("O04", "bounds out of order", setp("output.lower", 5000.0), False),
        ("O04", "bound without a reason", setp("output.reason_upper", ""), False),
        ("C01", "computed node with no input", setp("inputs", []), False),
        ("C02", "input from a node that does not exist", m(lambda f: f["request"]["proposed"]["inputs"].append({"binding": "x", "from": "sys_nowhere", "type": "Ratio"})), False),
        ("C02", "binding used twice", m(lambda f: f["request"]["proposed"]["inputs"].append(dict(f["request"]["proposed"]["inputs"][0]))), False),
        ("C03", "input quantity differs from its producer", m(lambda f: f["request"]["proposed"]["inputs"][0].update({"type": "Mass"})), False),
        ("C04", "input across a layer", m(lambda f: f["request"]["proposed"]["inputs"].append({"binding": "k", "from": plan.node_id("ct1_0"), "type": ""})), False),
        ("C05", "a loop", lambda: (lambda f: (person(f), f["request"]["proposed"]["inputs"].append({"binding": "loop", "from": downstream, "type": plan.output_of(plan.resolve(downstream))[0]}), f)[-1])(form_for("gf_6")), False),
        ("C06", "zero-when-absent names a node not read", setp("zero_when_absent", [plan.node_id("cf_3")]), False),
        ("R01", "no relation", setp("relation.expression", ""), False),
        ("R01", "no reason why", setp("relation.why", ""), False),
        ("R02", "a physics function that does not exist", m(lambda f: f["request"]["proposed"]["steps"][0].update({"physics": "fmr::nothing"})), False),
        ("R02", "no function named or described", m(lambda f: f["request"]["proposed"]["steps"][0].update({"physics": "", "new_function": ""})), False),
        ("R03", "last step does not bind the answer", m(lambda f: f["request"]["proposed"]["steps"][-1].update({"binds": "x"})), False),
        ("R06", "assumption without its failure case", m(lambda f: f["request"]["proposed"]["assumptions"].append({"text": "rigid ring", "fails_when": ""})), False),
        ("R07", "empty derivation step", m(lambda f: f["request"]["proposed"]["theory"].append({"text": "", "math": "x"})), False),
        ("V01", "declared value outside its bounds", lambda: (lambda f: (person(f), f["request"]["proposed"].update({"value": 1e9}), f)[-1])(form_for("gf_0")), False),
        ("S01", "unknown source", setp("relation.source", "someone2099"), False),
        ("S01", "new source without where", m(lambda f: f["request"]["new_sources"].append({"id": "x2026", "title": "X", "where": ""})), False),
        ("T01", "an agent-generated test vector", m(lambda f: f["request"]["proposed"]["fixtures"].append(dict(f["request"]["proposed"]["fixtures"][0], provenance="agent-generated"))), False),
        ("T02", "test vector without a page", m(lambda f: f["request"]["proposed"]["fixtures"][0].update({"where": ""})), False),
        ("T03", "test vector missing a binding", m(lambda f: f["request"]["proposed"]["fixtures"][0]["inputs"].pop(sorted(f["request"]["proposed"]["fixtures"][0]["inputs"])[0])), False),
        ("T03", "zero tolerance", m(lambda f: f["request"]["proposed"]["fixtures"][0].update({"tolerance": 0})), False),
        ("A01", "attachment with a path", m(lambda f: f["request"]["attachments"].append({"name": "../x.m", "size": 3, "data": "data:text/plain;base64,eHh4"})), False),
        ("B01", "feedback with nothing observed", lambda: (lambda f: (person(f), f["request"].update({"type": "feedback"}), f["feedback"].append({"observed": ""}), f)[-1])(form_for("gf_7")), False),
        ("B02", "other without a description", lambda: (lambda f: (person(f), f["request"].update({"type": "other", "other": {"subject": "a part", "description": ""}}), f)[-1])(form_for("gf_7")), False),
        ("N01", "new node in a group that does not exist", new_form(parent="sys_nowhere", id="sys_x_y", label="Y", kind="declared"), False),
        ("F03", "new node with an id already used", new_form(parent=plan.node_id("gf"), id=plan.node_id("gf_7"), label="Y", kind="declared"), False),
        ("N02", "new node of an unknown kind", new_form(parent=plan.node_id("gf"), id="sys_fluid_momentum_rings_new_row", label="New row", kind="lookup"), False),
        ("D01", "a change with no belief record", m(lambda f: f["request"].update({"derisk": {}})), False),
        ("D01", "a belief in an area that is not one of the seven", m(lambda f: f["request"]["derisk"].update({"area": "finance"})), False),
        ("D02", "a change that does not say what was wrong before", m(lambda f: f["request"]["derisk"].update({"previous_issue": ""})), False),
        ("D03", "a broken belief with no test", m(lambda f: f["request"]["derisk"].update({"tested": ""})), False),
        ("D03", "an untested belief with no test that would settle it", m(lambda f: f["request"]["derisk"].update({"status": "untested", "tested": ""})), False),
        ("D04", "a move from a level the risk is not at", m(lambda f: f["request"]["derisk"].update({"moves": [{"risk": "R-04", "from": 2, "to": 1}]})), False),
        ("D04", "a move of a risk that does not exist", m(lambda f: f["request"]["derisk"].update({"moves": [{"risk": "R-99", "from": 3, "to": 2}]})), False),
        ("D04", "a new risk with no closing test", m(lambda f: f["request"]["derisk"].update({"moves": [{"risk": "", "from": 0, "to": 3, "title": "x", "area": "math", "closing_test": ""}]})), False),
        ("D05", "a risk lowered by an untested belief", m(lambda f: f["request"]["derisk"].update({"status": "untested", "tested": "", "would_test": "a bench run", "moves": [{"risk": "R-04", "from": 4, "to": 3}]})), False),
        ("D06", "a cost that is not a number", m(lambda f: f["request"]["derisk"].update({"cost_k": "a lot"})), False),
        ("X01", "an analogy with no limit", setp("explain.analogy_breaks", ""), False),
        ("X02", "a wrong idea with no because", setp("explain.wrong_because", ""), False),
    ]
    bad = []
    for code, what, make, seed_ok in MUT:
        f = make()
        p = save(f, "m.html")
        rc, F, _, _ = run_check(p, seed_ok, quiet=True, plan=plan)
        codes = {x.code for x in F if x.level == "error"}
        if rc == 0 or code not in codes:
            bad.append("%s (%s): not refused by %s; got %s" % (what, code, code, sorted(codes) or "a pass"))
    # clean requests pass
    clean = [("a real change", change_gf7(), False), ("a change whose broken belief lowers one risk and opens another", change_gf7_moving(), False),
             ("a new node", (lambda f: (f["request"]["proposed"].update(json.loads(json.dumps(pm.proposal_from(plan.current("gf_7"))))),
                                        f["request"]["proposed"]["output"].update({"symbol": "T_spin2"}),
                                        f["request"]["proposed"]["steps"][-1].update({"binds": "T_spin2"}),
                                        f["request"]["proposed"]["relation"].update({"expression": "T_spin2 = rho*d^2/(32*mu)"}),
                                        f["request"]["proposed"].update({"fixtures": []}),
                                        belief(f, status="untested", tested="", would_test="a bench run of the second ring", previous_issue="", benefit=""), f)[-1])(
                 new_form(parent=plan.node_id("gf"), like=plan.node_id("gf_7"), id="sys_fluid_momentum_rings_second_ring_spin_down_time", label="Second ring spin-down time", kind="computed")()), False), ("a confirmation", (lambda f: (person(f), f["request"].update({"type": "confirm", "attested_by": "A. Engineer"}), f)[-1])(form_for("gf_7")), False),
             ("feedback", (lambda f: (person(f), f["request"].update({"type": "feedback"}), f["feedback"].append({"observed": "spin-down looks slow", "release": "1.0"}), f)[-1])(form_for("gf_7")), False),
             ("feedback on an older copy of the node", (lambda f: (person(f), f.update({"base": "0" * 16}), f["request"].update({"type": "feedback"}), f["feedback"].append({"observed": "after 1.1.0 the note is still missing", "release": "1.1.0"}), f)[-1])(form_for("gf_7")), False)]
    for t in sorted(plan.seed):
        clean.append(("seed form %s" % t, form_for(t, "seed"), True))
    for what, f, seed_ok in clean:
        rc, F, _, _ = run_check(save(f, "c.html"), seed_ok, quiet=True, plan=plan)
        if rc:
            bad.append("%s: refused: %s" % (what, "; ".join("%s %s" % (x.code, x.text) for x in F if x.level == "error")))
    # sheet -> verify round-trips, and a changed sheet is caught
    for t in sorted(plan.seed):
        f = form_for(t, "seed")
        _, F, info, _ = run_check(save(f, "s.html"), True, quiet=True, plan=plan)
        a, b = sheet_from(f, info, "REQ-selftest")
        sh, fx = tomllib.loads(a), tomllib.loads(b)
        d = verify(f, info, sh, fx)
        if d:
            bad.append("sheet %s does not verify against its own request: %s" % (t, d[:2]))
    f = form_for("gf_7", "seed")
    _, _, info, _ = run_check(save(f, "s.html"), True, quiet=True, plan=plan)
    a, b = sheet_from(f, info, "REQ-selftest")
    sh = tomllib.loads(a.replace('unit = "Second"', 'unit = "Minute"'))
    if not verify(f, info, sh, tomllib.loads(b)):
        bad.append("verify did not notice a changed unit")
    fx = tomllib.loads(b)
    fx["fixture"][0]["expect"] = fx["fixture"][0]["expect"] * 1.5
    if not verify(f, info, tomllib.loads(a), fx):
        bad.append("verify did not notice a changed expected value")
    # the ledger files intake write adds, and verify's check of the belief
    f = change_gf7_moving()
    rc, F, info, _ = run_check(save(f, "d.html"), False, quiet=True, plan=plan)
    files = derisk_files(f, info, "REQ-2026-09-27-gf_7-abcdef", plan)
    if rc or not files:
        bad.append("the moving change did not pass, or wrote no ledger files")
    else:
        bel = tomllib.loads(files[0])
        led = verify_ledger(f, info, "REQ-2026-09-27-gf_7-abcdef", bel, tomllib.loads(files[1]), tomllib.loads(files[2]))
        if led:
            bad.append("the ledger files do not verify against their request: %s" % led)
        wrong = tomllib.loads(files[2])
        wrong["move"][0]["risk"] = "R-03"
        if not verify_ledger(f, info, "REQ-2026-09-27-gf_7-abcdef", bel, tomllib.loads(files[1]), wrong):
            bad.append("verify did not notice a move filed against the wrong risk")
        if verify_belief(f, "REQ-2026-09-27-gf_7-abcdef", bel):
            bad.append("the belief record does not verify: %s" % verify_belief(f, "REQ-2026-09-27-gf_7-abcdef", bel))
        if not files[1] or "n = 2" not in files[1]:
            bad.append("the version entry is not version 2 of gf_7")
        with open(os.path.join(pm.ROOT, "derisk", "risks.toml"), "rb") as fh:
            nxt = "R-%02d" % (max(int(r["id"][2:]) for r in tomllib.load(fh)["risk"]) + 1)
        if nxt not in files[2] or 'risk = "R-04"' not in files[2]:
            bad.append("the moves do not name R-04 and the new risk %s" % nxt)
        bel["status"] = "held"
        if not verify_belief(f, "REQ-2026-09-27-gf_7-abcdef", bel):
            bad.append("verify did not notice a changed belief status")
    for x in bad:
        print("FAIL", x)
    print("selftest %s: %d deliberate mistakes refused by their codes, %d clean requests pass, %d sheets verify"
          % ("ok" if not bad else "FAILED", len(MUT), len(clean), len(plan.seed)))
    return 1 if bad else 0


def main():
    a = sys.argv[1:]
    seed_ok = "--seed" in a
    a = [x for x in a if x != "--seed"]

    def opt(name):
        if name in a:
            k = a.index(name)
            v = a[k + 1]
            del a[k:k + 2]
            return v
        return None
    out = opt("--out")
    ledger_dir = opt("--ledger")
    if a[:1] == ["check"] and len(a) == 2:
        rc, _, _, _ = run_check(a[1], seed_ok, out)
        return rc
    if a[:1] == ["sheet"] and len(a) == 2 and out:
        rc, F, info, form = run_check(a[1], seed_ok, None, quiet=True)
        if rc:
            print("the request does not pass the check; run `check` for the report")
            return 1
        rid = request_id(form)
        sh, fx = sheet_from(form, info, rid)
        os.makedirs(out, exist_ok=True)
        open(os.path.join(out, "node.toml"), "w", encoding="utf-8").write(sh)
        open(os.path.join(out, "fixtures.toml"), "w", encoding="utf-8").write(fx)
        extra = derisk_files(form, info, rid, pm.Plan())
        wrote = []
        if extra:
            for name, text in zip(("belief.toml", "versions.entry.toml", "risk_moves.toml"), extra):
                if text:
                    open(os.path.join(out, name), "w", encoding="utf-8").write(text)
                    wrote.append(name)
        print("wrote %s/node.toml and fixtures.toml for %s%s" % (out, info["target"],
              ", and %s (what intake write adds to the node's history and the ledger)" % ", ".join(wrote) if wrote else ""))
        return 0
    if a[:1] == ["verify"] and len(a) in (3, 4):
        import tomllib
        form, _ = read_form(a[1])
        F, info = check(form, None, seed_ok)
        sheet = tomllib.load(open(a[2], "rb"))
        fx = tomllib.load(open(a[3], "rb")) if len(a) == 4 else None
        d = verify(form, info, sheet, fx)
        led = ledger_dir
        if led:
            rd = lambda n: tomllib.load(open(os.path.join(led, n), "rb")) if os.path.exists(os.path.join(led, n)) else None
            d += verify_ledger(form, info, request_id(form), rd("belief.toml"), rd("versions.entry.toml"), rd("risk_moves.toml"))
        for name, want, got in d:
            print("DIFF %-22s request %r · sheet %r" % (name, want, got))
        print("verify: %s — %d difference(s)" % ("the sheet says what the request says" if not d else "the sheet does NOT match the request", len(d)))
        return 1 if d else 0
    if a[:1] == ["reply"] and len(a) == 2 and out:
        st, note, rel, rep = opt("--status"), opt("--note"), opt("--release"), opt("--report")
        print("wrote", reply(a[1], st, note, rel, rep, out))
        return 0
    if a == ["codes"]:
        for k, v in CHECKS.items():
            print("%s  %-7s  %s" % (k, "warning" if k in WARNINGS else "error", v))
        return 0
    if a == ["selftest"]:
        return selftest()
    print(__doc__.split("\n\n")[1])
    return 2


if __name__ == "__main__":
    sys.exit(main())
