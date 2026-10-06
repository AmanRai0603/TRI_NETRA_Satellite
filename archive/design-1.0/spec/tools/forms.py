#!/usr/bin/env python3
"""The two documents a team member fills, written from the plan (SPEC.md §5.10, §8.3):
the node form (adcs-node-form/1) and the case editor (adcs-case-editor/1). This is
the package's stand-in for `cargo xtask form export` in the built repository.

    python3 tools/forms.py node <tree_id> [--out <dir>]        a node's form, as the release holds the node
    python3 tools/forms.py seed <tree_id> [--out <dir>]        a node's seed form: its seed content as a request
    python3 tools/forms.py seeds [--out <dir>]                 every seed form (the pilot thread and the risk branch, 82 rows)
    python3 tools/forms.py new [--out <dir>]                   the blank request for a new node
    python3 tools/forms.py case [<case.csv>] [--out <dir>]     the case editor, blank or filled from a CSV
    python3 tools/forms.py library [--only a,b] [--out <dir>]  every node's form (or only those tree ids),
                                                               the new-node request, the case editor and an index page
    python3 tools/forms.py case-csv <saved editor.html> <out.csv>   the CSV a saved editor copy holds

A node form is forms/node_form.html with its JSON block filled in; the case editor
is forms/case_editor.html the same way. Both templates are the files the built
repository ships unchanged. Nothing here writes to the plan: reading a filled node
form is tools/intake.py's job, the stand-in for `cargo xtask intake`.
"""

import csv
import datetime
import glob
import io
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import plan_model as pm  # noqa: E402

ROOT = pm.ROOT
NODE_TEMPLATE = os.path.join(ROOT, "forms", "node_form.html")
CASE_TEMPLATE = os.path.join(ROOT, "forms", "case_editor.html")
LIBRARY_TEMPLATE = os.path.join(ROOT, "forms", "library.html")
NODE_SCHEMA = "adcs-node-form/1"
CASE_EDITOR_SCHEMA = "adcs-case-editor/1"
NODE_OPEN = '<script type="application/json" id="adcs-node-form">'
CASE_OPEN = '<script type="application/json" id="adcs-case-editor">'
SEED_REQUESTER = "ADCS-SPEC-01 package (seed content)"


def exported():
    # SOURCE_DATE_EPOCH (the reproducible-builds convention) fixes the date, so an
    # export repeated at one commit is byte-identical.
    t = os.environ.get("SOURCE_DATE_EPOCH")
    day = datetime.datetime.fromtimestamp(int(t), datetime.timezone.utc).date() if t else datetime.date.today()
    return {"commit": "package", "software": "plan (not yet built)", "date": day.isoformat(),
            "tool": "tools/forms.py (stand-in for cargo xtask form export)"}


def fill(template, title, block, open_tag):
    tpl = open(template, encoding="utf-8").read()
    body = json.dumps(block, ensure_ascii=False, separators=(",", ":")).replace("<", "\\u003c")
    esc = title.replace("&", "&amp;").replace("<", "&lt;")
    if open_tag not in tpl:
        raise SystemExit("%s has no %s block" % (template, open_tag))
    return tpl.replace("__TITLE__", esc, 1).replace("__FORM_JSON__", body, 1)


def blank_request(rtype, proposed):
    return {"type": rtype, "proposed": proposed, "placement": {}, "new_sources": [], "reason": "", "priority": "normal",
            "needed_by": "", "requested_by": "", "team": "", "contact": "", "attested_by": "",
            "other": {"subject": "", "description": ""}, "attachments": [], "summary": "",
            "derisk": {"area": "", "believed": "", "status": "", "tested": "", "now_know": "", "cost_k": "", "plan_change": "",
                       "previous_issue": "", "benefit": "", "would_test": "", "moves": []}}


def node_form(plan, tree_id, mode="change"):
    """mode "change": the node as the release holds it (its seed content, once
    that has been through intake), for a team member to request a change.
    mode "seed": the node as seeding alone leaves it, with its seed content as
    the request, for the developer team's first intake (SPEC.md §5.8)."""
    if tree_id not in plan.rows or tree_id in plan.groups:
        raise SystemExit("%s is not a leaf of plan/tree.json" % tree_id)
    if mode == "seed":
        if tree_id not in plan.seed:
            raise SystemExit("%s has no seed content in plan/seed_content.toml" % tree_id)
        node = plan.skeleton(tree_id)
        req = blank_request("seed", pm.proposal_from(plan.current(tree_id)))
        req.update({"requested_by": SEED_REQUESTER, "team": "developer team",
                    "reason": "Seed content for the pilot thread, written from the cited sources for the first build. "
                              "Nobody has confirmed it: it runs as UNCONFIRMED until an engineer attests it in a node form."})
        title = "Seed form — %s" % plan.rows[tree_id][1]
    else:
        node = plan.current(tree_id)
        req = blank_request("change" if node["kind"] in pm.KINDS else "feedback", pm.proposal_from(node))
        title = "%s — node form" % plan.rows[tree_id][1]
    return {
        "schema": NODE_SCHEMA, "title": title, "exported": exported(), "mode": mode,
        "node": node, "base": pm.base_of(node), "consumers": plan.consumers(tree_id), "kpis": plan.kpis_fed(tree_id),
        "catalog": plan.catalog(), "request": req, "feedback": [], "history": [],
    }


def new_node_form(plan):
    proposed = pm.proposal_from(pm.empty_node())
    req = blank_request("new", proposed)
    req["placement"] = {"parent": "", "like": "", "label": "", "id": "", "kind": "computed"}
    return {"schema": NODE_SCHEMA, "title": "A new node — request to the developer team", "exported": exported(), "mode": "new",
            "node": None, "base": "new", "consumers": [], "kpis": [], "catalog": plan.catalog(), "request": req,
            "feedback": [], "history": []}


def write(path, text):
    os.makedirs(os.path.dirname(path) or ".", exist_ok=True)
    open(path, "w", encoding="utf-8").write(text)
    return path


def export_node(plan, tree_id, out, mode="change"):
    f = node_form(plan, tree_id, mode)
    name = plan.node_id(tree_id) + (".seed" if mode == "seed" else "") + ".form.html"
    return write(os.path.join(out, name), fill(NODE_TEMPLATE, f["title"], f, NODE_OPEN))


def export_new(plan, out):
    f = new_node_form(plan)
    return write(os.path.join(out, "new_node.form.html"), fill(NODE_TEMPLATE, f["title"], f, NODE_OPEN))


# ------------------------------------------------------------------ the case editor

SECTIONS = {
    "meta": ("About the case", "Who and what this case is. The id names the file and every result made from it."),
    "req": ("What the ADCS must achieve", "Each requirement in its unit, and the per cent of cases it must hold for (level). "
            "A blank requirement is simply not judged."),
    "mission": ("Mission", "How long, when, and how the satellite will be pointed."),
    "orbit": ("Orbit", "A circular orbit by default. The orbit is always the case's: no scenario restates it."),
    "mass": ("Mass properties", "The whole satellite as the ADCS must turn it. Give lo and hi where they are uncertain."),
    "surface": ("Surfaces", "What drag and sunlight push on. Offsets are between the centre of pressure and the centre of mass."),
    "magnetic": ("Magnetic cleanliness", "The satellite's own magnetic dipole, which the field turns into a torque."),
    "flex": ("Flexible modes", "Only if the satellite has flexible panels or booms."),
    "resources": ("Resources offered to the ADCS", "What the platform can give the ADCS. The solver refuses a product that needs more."),
    "pointing": ("Pointing budget inputs", "Contributions the ADCS cannot see for itself."),
}


def scenario_reads(scn):
    """The case keys a scenario needs: its own `case:` references, plus what the
    plant reads for each environment switch that is on (SPEC.md §10.2)."""
    keys = set()

    def walk(v):
        if isinstance(v, str) and v.startswith("case:"):
            keys.add(v[5:])
        elif isinstance(v, dict):
            for x in v.values():
                walk(x)
        elif isinstance(v, list):
            for x in v:
                walk(x)
    walk(scn)
    env = scn.get("environment", {})
    keys |= set(pm.PLANT_READS["always"])
    for switch, ks in pm.PLANT_READS.items():
        if switch != "always" and env.get(switch):
            keys |= set(ks)
    return sorted(keys)


def case_editor(plan, case_csv=None):
    reg = plan.case
    kp = {k["requirement"]: k for k in plan.kpis}
    got = {}
    if case_csv:
        got = {r[1]: r for r in list(csv.reader(open(case_csv, encoding="utf-8")))[1:] if len(r) >= 9}
    rows, values = [], {}
    for m in reg.get("meta", []):
        rows.append({"section": "meta", "key": m["key"], "label": m["label"], "unit": "", "blank": "stated",
                     "range": False, "level": False, "help": m.get("note", "")})
    for i in reg.get("input", []):
        rows.append({"section": i["section"], "key": i["key"], "label": i["label"], "unit": i["unit"], "blank": i["blank"],
                     "range": i["range"], "level": i["level"], "help": i.get("help", ""),
                     "sense": kp.get(i["tree_id"], {}).get("sense") if i["section"] == "req" else None,
                     "fraction": i["unit"] == "One" and "0 to 1" in i.get("help", "")})
    for k, r in got.items():
        if k == "meta.schema":
            continue
        v = {"value": r[4], "lo": r[5], "hi": r[6], "level": r[7], "note": r[8]}
        if any(v.values()):
            values[k] = v
    presets = []
    for p in sorted(glob.glob(os.path.join(plan.root, "plan", "cases", "*.csv"))):
        pr = {r[1]: r for r in list(csv.reader(open(p, encoding="utf-8")))[1:] if len(r) >= 9}
        presets.append({"id": os.path.basename(p)[:-4], "title": pr.get("meta.title", [""] * 9)[4],
                        "values": {k: {"value": r[4], "lo": r[5], "hi": r[6], "level": r[7], "note": r[8]}
                                   for k, r in pr.items() if k != "meta.schema" and any(r[4:9])}})
    scen = []
    for p in sorted(glob.glob(os.path.join(plan.root, "scenarios", "*.toml"))):
        s = pm.toml(p)
        binds = []
        for m in s.get("metric", []):
            k = plan.case_key.get(m.get("requirement", ""))
            if k and k not in binds:
                binds.append(k)
        scen.append({"id": s.get("id"), "label": s.get("label", ""), "reads": scenario_reads(s), "binds": binds})
    classes = [c["id"] for c in pm.toml(os.path.join(plan.root, "catalogue", "classes.toml")).get("class", [])] \
        if os.path.exists(os.path.join(plan.root, "catalogue", "classes.toml")) else []
    families = [f["id"] for f in pm.toml(os.path.join(plan.root, "catalogue", "families.toml")).get("family", [])] \
        if os.path.exists(os.path.join(plan.root, "catalogue", "families.toml")) else []
    cid = got.get("meta.case_id", [""] * 9)[4]
    title = got.get("meta.title", [""] * 9)[4] or "A new case"
    return {"schema": CASE_EDITOR_SCHEMA, "title": "Case editor — %s" % (cid or "new case"), "exported": exported(),
            "format": reg.get("schema"), "columns": reg["columns"], "default_level": reg.get("default_level"),
            "sections": {k: {"title": a, "lead": b} for k, (a, b) in SECTIONS.items()},
            "rows": rows, "values": values, "presets": presets, "scenarios": scen, "classes": classes, "families": families,
            "loaded_from": os.path.basename(case_csv) if case_csv else "blank", "case_title": title}


def export_case(plan, case_csv, out):
    f = case_editor(plan, case_csv)
    cid = f["values"].get("meta.case_id", {}).get("value") or "new_case"
    return write(os.path.join(out, "case_" + cid + ".editor.html"), fill(CASE_TEMPLATE, f["title"], f, CASE_OPEN))


def read_block(path, open_tag):
    text = open(path, encoding="utf-8").read()
    i = text.find(open_tag)
    if i < 0:
        raise SystemExit("%s: no %s block" % (path, open_tag))
    j = text.find("</script>", i)
    return json.loads(text[i + len(open_tag):j])


def editor_csv(block):
    out = io.StringIO()
    w = csv.writer(out, lineterminator="\n")
    w.writerow(block["columns"])
    V = block.get("values", {})
    for r in block["rows"]:
        v = V.get(r["key"], {})
        g = lambda k: "" if r["section"] == "meta" and k != "value" else str(v.get(k, "") or "")
        val = block["format"] if r["key"] == "meta.schema" else g("value")
        w.writerow([r["section"], r["key"], r["label"], r.get("unit", ""), val, g("lo"), g("hi"), g("level"), g("note")])
    return out.getvalue()


# ------------------------------------------------------------------ the library

def export_library(plan, out, only=None):
    """Every layer-1 and layer-2 node's form, the new-node request and the case
    editor, with an index: the offline library a release carries (SPEC.md §5.10.5).
    `only` (tree ids) exports a few, for a quick look or a check."""
    entries = []
    for t in sorted(plan.leaves, key=lambda x: (plan.layer(x), list(plan.rows).index(x))):
        if only and t not in only:
            continue
        export_node(plan, t, os.path.join(out, "nodes"))
        n = plan.current(t)
        entries.append({"layer": pm.LAYER_NAME[plan.layer(t)], "group": plan.rows[plan.rows[t][2]][1], "tree_id": t,
                        "node": plan.node_id(t), "label": plan.rows[t][1], "kind": n["kind"], "state": n["state"],
                        "file": "nodes/%s.form.html" % plan.node_id(t),
                        "risk": max([r["level"] for r in n.get("risks", [])] or [0]),
                        "one_line": (n.get("explain") or {}).get("one_line", ""), "versions": len(n.get("versions", []))})
    export_new(plan, out)
    export_case(plan, None, out)
    body = json.dumps({"schema": "adcs-library/1", "exported": exported(), "entries": entries,
                       "extra": [["Ask for a new node", "new_node.form.html", "a node the tree does not have yet"],
                                 ["Write a case", "case_new_case.editor.html", "your input to the software, as a CSV"]]},
                      ensure_ascii=False, indent=1).replace("<", "\\u003c")
    idx = open(LIBRARY_TEMPLATE, encoding="utf-8").read().replace("__LIBRARY_JSON__", body, 1)
    write(os.path.join(out, "index.html"), idx)
    return os.path.join(out, "index.html"), len(entries)


def main():
    a = sys.argv[1:]
    out = os.path.join(ROOT, "forms", "out")
    if "--out" in a:
        k = a.index("--out")
        out = a[k + 1]
        del a[k:k + 2]
    plan = pm.Plan()
    if a[:1] == ["node"] and len(a) == 2:
        print("wrote", export_node(plan, a[1], out))
    elif a[:1] == ["seed"] and len(a) == 2:
        print("wrote", export_node(plan, a[1], out, "seed"))
    elif a == ["seeds"]:
        for t in plan.seed:
            export_node(plan, t, out, "seed")
        print("wrote %d seed forms to %s" % (len(plan.seed), out))
    elif a == ["new"]:
        print("wrote", export_new(plan, out))
    elif a[:1] == ["case"] and len(a) in (1, 2):
        print("wrote", export_case(plan, a[1] if len(a) == 2 else None, out))
    elif a[:1] == ["library"] and (len(a) == 1 or (len(a) == 3 and a[1] == "--only")):
        path, n = export_library(plan, out, set(a[2].split(",")) if len(a) == 3 else None)
        print("wrote %s and %d node forms" % (path, n))
    elif a[:1] == ["case-csv"] and len(a) == 3:
        b = read_block(a[1], CASE_OPEN)
        if b.get("schema") != CASE_EDITOR_SCHEMA:
            raise SystemExit("%s is not a case editor (%s)" % (a[1], b.get("schema")))
        write(a[2], editor_csv(b))
        print("wrote", a[2])
    else:
        print(__doc__.split("\n\n")[1])
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
