#!/usr/bin/env python3
"""The design leaves the repository (docs/PLAN_2_0.md S3): the one conversion of TRI-NETRA's design
into the 2.0.0 layout of the shared drive, run once (and again only as a corrected conversion,
zip 1.x, until the first released design).

    python3 tools/convert_2_0.py --out DIR        write the converted design into DIR
    python3 tools/convert_2_0.py --out DIR --check   and check every file it wrote (exit 1 on a problem)

What it starts from: the 1.0.0 design as the seed and the carry-over make it (765 node files, 20
group files; tools/seed_design.py, tools/carry_over.py), so every field 1.0.0 holds travels as it is,
and the repository's library data (the catalogue, the cases, scenarios, campaigns and trades, the
flight software's algorithms, parameters and IGRF table, the KPI definitions, the delivery waves).

What it writes (docs/OPERATING_2_0.md §15):

    groups/<group>/<group>.group.tndb   21 groups (design/tree_2_0.toml): today's 20 less `case`,
                                        plus `programme` and `systems`; each with its mounts
    groups/<group>/nodes/*.node.tndb    every node, with its block (parent, perspective, behaviour),
                                        its ports (state, maturity, sense), its closures and loops
    groups/<group>/releases/<g>-0.1.tnrel  the baseline: sealed by the conversion, every node marked
                                        unconfirmed: "converted, not yet signed by a person"
    cases/*.tncase                      the reference cases, the scenarios, campaigns and trades
    readable/*.csv                      Groups, Nodes, Ports, Wires, Closures, and Conversion.csv
    design/ daily/ integration/ issues/ results/   empty: the application fills them

The rules that invent nothing (docs/PLAN_2_0.md S3, step 2): a value stated with its source is
decided; a required row's bound is allocated; a row computed by pseudocode or code is achieved; any
other is open. Maturity is "estimated" for a stated or allocated value, "calculated" for a computed
one; nothing is "measured" until a person says so with its source. A range is the one the node
already carries (its output's lower and upper with their reasons); an open value without one is
listed in Conversion.csv.

Behaviour: method (it has pseudocode), stated (a declared value), children (a branch of the tree), or
built-in (its relation is still in code, named by node id; S7 removes every one). A row with none
of these is open.

The developer's revisions (design/revisions_2_0.toml) are applied on top, in order: each moves the nodes it
names from one behaviour to another, adds the rows it states, writes itself into each node's history (its
revision table) and its reason into the node's release (`why`), marked the developer's and not yet signed by a
person. S7.1b moves the built-in nodes that are not relations in code: achieved holders and the KPIs' evidence
rows to evidence, the KPI closures to closure, the owner pointers to open (or evidence where a run measures
them), the flight software's runtime rows to stated (code by the boundary). A revision may also add nodes
(`[[revision.node]]`), load published data into a node through a reader of tools/readers.py (`[[revision.data]]`,
S7.2b: the leap seconds, the IGRF table, the IAU 2006 series, the tidal EOP terms) and give a node its method, the
developer's transcription kept under design/revisions/ (`[[revision.method]]`, S7.3: env's time, frames and field; one module
may be several nodes' method, `nodes`, S7.13: the flight software's parameters).

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import argparse
import csv
import hashlib
import io
import json
import pathlib
import re
import shutil
import sqlite3
import sys
import tempfile
import tomllib

import carry_over
import design_inputs
import release
import seed_design
import tndb
from common import ROOT, V1, v1_root

TREE = ROOT / "design" / "tree_2_0.toml"
REVISIONS = ROOT / "design" / "revisions_2_0.toml"
SPEC_TREE = V1 / "spec" / "plan" / "tree.json"
KPIS = V1 / "spec" / "plan" / "kpis.toml"
GROUPS_1 = ROOT / "design" / "groups.toml"
BY = "the conversion (tools/convert_2_0.py)"
AT = "2026-10-06T00:00:00Z"            # fixed, so the conversion is the same bytes every time it runs
WHY = "converted from 1.0.0, not yet signed by a person"
FSW_MODULES = {"03_estimation.pc": "nav", "04_guidance.pc": "gdn", "05_control.pc": "ctl", "06_step_laws.pc": "ctl",
               "07_allocation.pc": "ctl", "08_mode_manager.pc": "gdn", "09_drivers.pc": "fsw"}
TOOLBOX_MODULES = ["01_math.pc", "02_time_frames_models.pc", "02_igrf13.pc"]
# Library data: (folder, glob, owning group, what it is). Each file becomes a lookup block of its group.
LIBRARY = [("catalogue", "**/*.toml", "catalogue", "the catalogue"),
           ("spec/plan", "kpis.toml", "kpi", "the KPI definitions"),
           ("spec/plan", "units.toml", "programme", "the units and quantities the design may state"),
           ("fsw/params", "params.toml", "fsw", "the flight software's parameter table (adcs-fswcfg/1)"),
           ("matlab_sils/data/catalogue", "*.json", "catalogue", "the datasheet catalogue (adcs-datasheet/1)"),
           ("matlab_sils/data/pipeline", "nodes.json", "design", "the design loop's node registry (adcs-nodes/1)"),
           ("matlab_sils/data", "scenario_schema.json", "programme", "the scenario format (adcs-scenario/1)"),
           # the rest of 1.0.0's plan that the code still reads until its relations are methods (S7); held whole, so
           # the repository can archive spec/ and read them from the design (tools/from_design.py)
           ("spec/plan", "case_inputs.toml", "programme", "which node declares each case line"),
           ("spec/plan", "case_template.csv", "programme", "the case template (adcs-case/1)"),
           ("spec/plan", "physics.toml", "systems", "the relations' registry (signatures, units, sources)"),
           ("spec/plan", "seed_content.toml", "systems", "the relations' worked examples (test vectors)"),
           ("spec/plan", "tree.json", "systems", "1.0.0's tree of rows"),
           ("spec/plan", "expected_node_ids.json", "systems", "1.0.0's row ids"),
           ("spec/plan", "twin_map.toml", "vv", "which twin function implements each row"),
           ("spec/physics", "*.pc", "systems", "the relations in pseudocode (adcs-physics is generated from them)")]
CASES = [("matlab_sils/cases", "*.csv", "case"), ("spec/plan/cases", "*.csv", "case"),
         ("scenarios", "*.toml", "scenario"), ("campaigns", "*.toml", "campaign"), ("trades", "*.toml", "trade")]


def sha(b):
    return hashlib.sha256(b if isinstance(b, bytes) else b.encode()).hexdigest()


def rows(conn, sql, args=()):
    conn.row_factory = sqlite3.Row
    return [dict(r) for r in conn.execute(sql, args)]


# ------------------------------------------------------------------ the tree's blocks

def tree():
    t = json.loads(SPEC_TREE.read_text())
    rows_ = {r[0]: {"id": r[0], "label": r[1], "parent": r[2], "about": r[3], "layer": 1 if k == "HN_MGT" else 2}
             for k in ("HN_MGT", "HN_SYS") for r in t[k]}
    headers = {r["parent"] for r in rows_.values() if r["parent"]}
    shapes = {s["id"]: s for s in t["layer3_shape"]}
    return rows_, headers, shapes


def parent_of(nid, node, rows_, shapes, kpi_req):
    """The block a node hangs under (docs/SYSTEM_MODEL.md §2)."""
    if nid in rows_:
        return rows_[nid]["parent"]
    m = re.match(r"l3_(.+?)_(interface|row_\d+|.+_(?:achieved|required))$", nid)
    if nid.startswith("l3_"):
        for sid in sorted(shapes, key=len, reverse=True):
            if nid.startswith(f"l3_{sid}_"):
                return f"l3_{sid}"
        if nid == "l3_x_closure_interface":
            return "x_closure"
    if nid in kpi_req:
        return rows_.get(kpi_req[nid], {}).get("parent")
    if node.get("stage"):
        return f"{node['group_id']}__{node['stage']}"
    return f"{node['group_id']}__carried"
    _ = m


def behaviour(kind, content):
    if kind in ("group", "branch", "interface", "closure_interface"):
        return "children"
    if content.get("code.pseudocode"):
        return "method"
    if kind in ("required",) or content.get("spec.kind") == "declared" or content.get("value.number") or content.get("spec.value"):
        return "stated"
    if kind in ("achieved", "closure_analysis", "closure_verified") or any(content.get(k) for k in ("code.rust", "code.c", "code.twin")):
        return "built-in"
    return "open"


def revisions(path=REVISIONS):
    """The developer's revisions, in order (design/revisions_2_0.toml)."""
    return tomllib.loads(pathlib.Path(path).read_text(encoding="utf-8"))["revision"]


def _matches(m, nid, kind, content):
    if "nodes" in m:
        return nid in m["nodes"]
    if "kind" in m:
        return kind in (m["kind"] if isinstance(m["kind"], list) else [m["kind"]])
    if "field" in m:
        return bool(content.get(m["field"]))
    raise SystemExit(f"convert: a revision matches by nodes, kind or field, not {sorted(m)}")


def add_nodes(gdir, groups, revs, report):
    """The nodes the developer's revisions add (`[[revision.node]]`), each a new block of its group under the parent it
    names, with the behaviour it says and any content rows it gives (a stated value and its source, S7.4): written
    before the group files, so each group lists them."""
    have = {f.name[:-len(".node.tndb")] for d in gdir.values() for f in (d / "nodes").glob("*.node.tndb")}
    why = {}
    for r in revs:
        origin = f"the developer's revision {r['id']}"
        for n in r.get("node", []):
            if n["id"] in have:
                raise SystemExit(f"convert: revision {r['id']} adds {n['id']}, which the design already has")
            if n["group"] not in groups or n["parent"] not in have:
                raise SystemExit(f"convert: revision {r['id']} adds {n['id']} to group {n['group']} under {n['parent']}: no such group or block")
            new_node(gdir[n["group"]] / "nodes" / f"{n['id']}.node.tndb", n["id"], n["group"], n.get("kind", "leaf"), n["label"], n.get("layer", 2),
                     content=[("identity", "question", n["question"], origin)] + [(a, b, v, origin) for a, b, v in n.get("content", [])],
                     block=(n["id"], n["parent"], n.get("perspective", "system"), n["behaviour"], None, None))
            with sqlite3.connect(gdir[n["group"]] / "nodes" / f"{n['id']}.node.tndb") as c:
                c.execute("INSERT INTO revision VALUES (?, ?, ?, ?)", (1, r["at"], r["by"], f"{r['id']}: added ({n['behaviour']}): {n['why']}"))
            have.add(n["id"])
            why.setdefault(n["id"], []).append(f"{origin} ({r['at'][:10]}), not yet signed by a person: added, {n['behaviour']}: {n['why']}")
            report.append(["revision", n["id"], "(none)", f"groups/{n['group']}/nodes/{n['id']}.node.tndb", f"{origin}: added, {n['behaviour']}: {n['why']}"])
    return why


def data_rows(r, d, origin):
    """The content rows a `[[revision.data]]` gives its node: the module the reader (tools/readers.py) reads from the
    file (or files), every table citing its publication and its file (with the file's sha256), and the publication."""
    import readers
    tables, docs, files = {}, {}, []
    for one in d.get("files") or [d]:
        got = readers.read({"reader": d["reader"], **one})
        h = readers.sha256(one["file"])
        files.append(f"{one['file']}, sha256 {h}")
        for name, t in got.items():
            tables[name] = t
            docs[name] = [f"{name}: {d['publication']}", f"read by tools/readers.py ({d['reader']}) from {one['file']} (sha256 {h[:16]}), {origin}"]
    text = readers.module_text(d["module"], d["about"], tables, docs)
    where = f"{d['path']} ({origin}: " if d.get("path") else f"{origin} ("
    return [("data", f"{d['module']}.pc", text, f"{where}reader {d['reader']} of {'; '.join(files)})"),
            ("sources", f"published.{d['module']}", d["publication"], origin)]


def method_rows(m, origin):
    """The content rows a `[[revision.method]]` gives its node: the pseudocode the developer transcribed (a file under
    design/revisions/, kept as the revision's source) and what it transcribes, the node's module and what it calls."""
    text = (ROOT / m["pseudocode"]).read_text(encoding="utf-8")
    where = f"{m['path']} ({origin}: " if m.get("path") else f"{origin} ("
    rows = [("code", "pseudocode", text, f"{where}{m['pseudocode']})"),
            ("code", "transcribes", m["transcribes"], origin),
            ("code", "generate", ", ".join(m["generate"]), origin)]
    if m.get("uses"):
        rows.append(("code", "uses", json.dumps(m["uses"]), origin))
    if m.get("source"):
        rows.append(("sources", "transcribed", m["source"], origin))
    return rows


def revise(gdir, revs, report):
    """Apply the developer's revisions to the node files: {node: [why, ...]} for their releases. A change whose
    node has another behaviour than its `from`, or that names a node the design does not have, is refused.
    Besides behaviour changes (`[[revision.change]]`), a revision may load published data into a node
    (`[[revision.data]]`: a reader of tools/readers.py on a file of the repository) and give a node its method
    (`[[revision.method]]`: the developer's transcription of code and of the source it cites, unsigned)."""
    files = {f.name[:-len(".node.tndb")]: f for d in gdir.values() for f in sorted((d / "nodes").glob("*.node.tndb"))}
    why = {}
    for r in revs:
        origin = f"the developer's revision {r['id']}"
        for kind, items in (("data", r.get("data", [])), ("method", r.get("method", []))):
            # a method may be several nodes' (`nodes`): one module whose functions give each of them its value (S7.13)
            for d, nid in ((d, nid) for d in items for nid in (d["nodes"] if "nodes" in d else [d["node"]])):
                f = files.get(nid)
                if f is None:
                    raise SystemExit(f"convert: revision {r['id']} gives {kind} to {nid}, which the design does not have")
                rows = data_rows(r, d, origin) if kind == "data" else method_rows(d, origin)
                with sqlite3.connect(f) as c:
                    beh = c.execute("SELECT behaviour FROM block").fetchone()[0]
                    to = d.get("to", beh)
                    if d.get("from", beh) != beh:
                        raise SystemExit(f"convert: revision {r['id']}: {nid} is {beh}, not {d['from']}")
                    c.execute("UPDATE block SET behaviour = ?", (to,))
                    c.executemany("INSERT INTO content VALUES (?, ?, ?, ?)", rows)
                    n = c.execute("SELECT coalesce(max(n), 0) + 1 FROM revision").fetchone()[0]
                    what = f"{kind} {rows[0][1] if kind == 'data' else 'pseudocode'}" + (f"; behaviour {beh} -> {to}" if to != beh else "")
                    c.execute("INSERT INTO revision VALUES (?, ?, ?, ?)", (n, r["at"], r["by"], f"{r['id']}: {what}: {d['why']}"))
                why.setdefault(nid, []).append(f"{origin} ({r['at'][:10]}), not yet signed by a person: {what}: {d['why']}")
                report.append(["revision", nid, beh, to, f"{origin}: {what}: {d['why']}"])
        for ch in r.get("change", []):
            named = set(ch["match"].get("nodes", []))
            if named - set(files):
                raise SystemExit(f"convert: revision {r['id']} names {sorted(named - set(files))}, which the design does not have")
            hit = 0
            for nid, f in sorted(files.items()):
                with sqlite3.connect(f) as c:
                    kind = c.execute("SELECT kind FROM node").fetchone()[0]
                    content = {f"{a}.{b}" if b else a: v for a, b, v in c.execute("SELECT section, field, value FROM content")}
                    if not _matches(ch["match"], nid, kind, content):
                        continue
                    beh = c.execute("SELECT behaviour FROM block").fetchone()[0]
                    if beh != ch["from"]:
                        raise SystemExit(f"convert: revision {r['id']}: {nid} is {beh}, not {ch['from']}")
                    c.execute("UPDATE block SET behaviour = ?", (ch["to"],))
                    c.executemany("INSERT INTO content VALUES (?, ?, ?, ?)", [(a, b, v, origin) for a, b, v in ch.get("content", [])])
                    n = c.execute("SELECT coalesce(max(n), 0) + 1 FROM revision").fetchone()[0]
                    c.execute("INSERT INTO revision VALUES (?, ?, ?, ?)", (n, r["at"], r["by"], f"{r['id']}: behaviour {ch['from']} -> {ch['to']}: {ch['why']}"))
                hit += 1
                why.setdefault(nid, []).append(f"{origin} ({r['at'][:10]}), not yet signed by a person: behaviour {ch['from']} -> {ch['to']}: {ch['why']}")
                report.append(["revision", nid, ch["from"], ch["to"], f"{origin}: {ch['why']}"])
            if not hit:
                raise SystemExit(f"convert: revision {r['id']}: a change to {ch['to']} matches no node")
    return why


def state_maturity(kind, beh, content):
    if kind == "required":
        return "allocated", "estimated"
    if beh == "stated":
        return ("decided", "estimated") if (content.get("value.source") or content.get("spec.source")) else ("open", "estimated")
    if beh in ("method", "built-in") or kind == "achieved":
        return "achieved", "calculated"
    return "open", None


SENSE = {"at_most": "<=", "at_least": ">=", "<=": "<=", ">=": ">="}


def node_doc(path):
    with sqlite3.connect(path) as c:
        n = rows(c, "SELECT * FROM node")[0]
        content = {}
        for r in rows(c, "SELECT section, field, value FROM content"):
            content[f"{r['section']}.{r['field']}" if r["field"] else r["section"]] = r["value"]
        outs = rows(c, "SELECT * FROM output")
        ins = rows(c, "SELECT * FROM input")
    return n, content, outs, ins


# ------------------------------------------------------------------ writing node files

def new_node(path, nid, group, kind, label, layer, *, content=(), outputs=(), fixtures=(), block, ports=(), loops=(), closures=()):
    def fill(c):
        c.execute("INSERT INTO node VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)", (nid, nid, group, None, str(layer), kind, label, "draft", None, 1))
        c.executemany("INSERT INTO content VALUES (?, ?, ?, ?)", content)
        c.executemany("INSERT INTO output VALUES (?, ?, ?, ?, ?, ?)", outputs)
        c.executemany("INSERT INTO fixture VALUES (?, ?, ?, ?, ?, ?)", fixtures)
        add_2_0(c, block, ports, loops, closures)
    tndb.create(path, "node", nid, written_by=BY, fill=fill, sync=False)


def add_2_0(c, block, ports=(), loops=(), closures=()):
    c.execute("INSERT INTO block VALUES (?, ?, ?, ?, ?, ?)", block)
    c.executemany("INSERT INTO port VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)", ports)
    c.executemany('INSERT INTO "loop" VALUES (?, ?, ?, ?, ?)', loops)
    c.executemany("INSERT INTO closure VALUES (?, ?, ?, ?, ?, ?, ?)", closures)


# ------------------------------------------------------------------ the conversion

def convert(out, src=None):
    out = pathlib.Path(out)
    if out.exists() and any(out.iterdir()):
        raise SystemExit(f"convert: {out} is not empty; the conversion writes a new folder")
    out.mkdir(parents=True, exist_ok=True)
    report = []          # Conversion.csv: what, where it was, where it is now, how
    tmp = None
    if src is None:
        tmp = tempfile.TemporaryDirectory()
        src = pathlib.Path(tmp.name) / "Design"
        seed_design.seed(src, sync=False)
        carry_over.carry(src)
    src = pathlib.Path(src)
    t2 = tomllib.loads(TREE.read_text())
    groups = {g["id"]: g for g in t2["group"]}
    rows_, headers, shapes = tree()
    kpis = tomllib.loads(KPIS.read_text())["kpi"]
    kpi_req = {}
    for k in kpis:
        for role in ("verified", "analysis"):
            kpi_req[f"kpi_{k['slug']}_{role}"] = k["requirement"]
    g1 = {g["id"]: g for g in tomllib.loads(GROUPS_1.read_text())["group"]}
    stage_label = {(g["id"], s["id"]): s.get("label", s["id"]) for g in g1.values() for s in g.get("stages", [])}

    # 1 · every 1.0.0 node: where it goes
    nodes = {}
    for f in sorted((src / "nodes").glob("*.node.tndb")):
        n, content, outs, ins = node_doc(f)
        nid, g = n["id"], n["group_id"]
        if g == "case":        # dissolved: case intake to the programme, mission and hardware fitted to the system
            g = "programme" if (rows_.get(nid, {}).get("layer") == 1 or nid.startswith("ci1")) else "systems"
        nodes[nid] = {"file": f, "node": n, "content": content, "outs": outs, "ins": ins, "group": g,
                      "parent": parent_of(nid, n, rows_, shapes, kpi_req)}

    # 2 · the blocks the tree implies but 1.0.0 kept as structure: its branches, the subsystem layers'
    #     roots, the groups' stages and their carried rows
    blocks = {}
    for h in headers:
        r = rows_[h]
        blocks[h] = {"label": r["label"], "parent": r["parent"] or ("mgm" if h == "prg" else None), "layer": r["layer"], "about": r["about"]}
    for sid, s in shapes.items():
        blocks[f"l3_{sid}"] = {"label": s["label"], "parent": s["group"], "layer": 3, "about": f"the {s['label'].lower()} subsystem layer"}
    blocks["x_closure"] = {"label": "KPI closures", "parent": "svc", "layer": "closure", "about": "every KPI closed by analysis and by evidence"}
    for x in nodes.values():
        p = x["parent"]
        if p and p not in blocks and p not in nodes and "__" in p:
            gid, rest = p.split("__", 1)
            mount = groups[x["group"]]["mounts"][0]["on"] if groups[x["group"]]["mounts"] else None
            blocks[p] = {"label": stage_label.get((gid, rest), "Rows carried from the code" if rest == "carried" else rest),
                         "parent": mount, "layer": 3, "about": f"stage {rest} of {gid}" if rest != "carried" else f"the rows {gid} carried from the code"}
    # who owns a branch: the group holding most of its rows; else programme (layer 1) or systems
    def leaves_under(b):
        out_ = []
        for nid, x in nodes.items():
            p = x["parent"]
            while p:
                if p == b:
                    out_.append(x["group"])
                    break
                p = blocks.get(p, {}).get("parent") or (nodes.get(p, {}).get("parent") if p in nodes else None)
        return out_
    frame = {gid: set(g.get("frame", [])) for gid, g in groups.items()}
    for b, x in blocks.items():
        if b in frame["programme"]:
            x["group"] = "programme"
        elif b in frame["systems"]:
            x["group"] = "systems"
        else:
            under = leaves_under(b)
            x["group"] = max(set(under), key=under.count) if under else ("programme" if x["layer"] == 1 else "systems")

    # 3 · the flight software's design: each algorithm module a block of its group, its vectors its cases
    vectors = {}
    for line in (ROOT / "fsw" / "tests" / "pcode_vectors.txt").read_text().splitlines():
        if line.startswith("#") or not line.strip():
            continue
        name = line.split()[0]
        vectors.setdefault(name.split("::")[0], []).append(line)
    fsw = {}
    for fname, gid in FSW_MODULES.items():
        text = (ROOT / "fsw" / "pseudocode" / fname).read_bytes().decode("utf-8")
        mod = re.search(r"^module\s+(\w+)", text, re.M).group(1)
        nid = f"fsw_{mod}"
        fsw[nid] = {"group": gid, "file": fname, "module": mod, "text": text, "doc": next(iter(sorted((ROOT / "fsw" / "pseudocode").glob(fname[:3] + "*.md"))), ROOT / "fsw" / "pseudocode" / fname.replace(".pc", ".md")),
                    "vectors": vectors.get(mod, [])}
    params = tomllib.loads((ROOT / "fsw" / "params" / "params.toml").read_text())

    # 4 · the library data and the cases
    library = []
    for folder, pat, gid, what in LIBRARY:
        for f in sorted((v1_root(folder) / folder).glob(pat)):
            rel = f.relative_to(v1_root(folder)).as_posix()
            library.append({"id": "lib_" + re.sub(r"[^a-z0-9]+", "_", rel.lower().rsplit(".", 1)[0]), "group": gid, "file": f, "rel": rel, "what": what})
    igrf = ROOT / "matlab_sils" / "data" / "igrf13coeffs.txt"

    # ------------------------------------------------------------ write the groups' folders
    gdir = {gid: out / "groups" / gid for gid in groups}
    for d in gdir.values():
        (d / "nodes").mkdir(parents=True)
        (d / "releases").mkdir()
    for d in ("cases", "readable", "design", "daily", "integration", "issues", "results"):
        (out / d).mkdir(exist_ok=True)

    ports_csv, closures_csv = [], []
    # 1.0.0 nodes: copied whole, then the 2.0.0 tables added
    for nid, x in sorted(nodes.items()):
        dst = gdir[x["group"]] / "nodes" / x["file"].name
        shutil.copy(x["file"], dst)
        n, content = x["node"], x["content"]
        beh = behaviour(n["kind"], content)
        st, mat = state_maturity(n["kind"], beh, content)
        persp = {"1": "programme", "2": "system", "3": "subsystem", "closure": "system", "added": "subsystem"}.get(str(n["layer"]), "subsystem")
        sense = SENSE.get(content.get("requirement.sense") or content.get("spec.sense") or "")
        ports = [(o["name"], "out", "number", st, mat, sense, None, None, None) for o in x["outs"]]
        ports += [(i["name"], "in", "number", None, None, None, None, None, None) for i in x["ins"]]
        closures = []
        if n["kind"].startswith("closure_") and nid in kpi_req:
            k = next(k for k in kpis if nid.startswith(f"kpi_{k['slug']}_"))
            by = "evidence" if nid.endswith("_verified") else "analysis"
            ach = k["evidence"] if by == "evidence" else k["analysis"]
            closures.append((nid, k["requirement"], ach if ach and ach != "none" else None, k.get("sense"), by, k.get("metric") if by == "evidence" else None, None))
            closures_csv.append([nid, k["requirement"], ach, k.get("sense"), by, k.get("metric")])
        with sqlite3.connect(dst) as c:
            if n["group_id"] != x["group"]:
                c.execute("UPDATE node SET group_id = ?", (x["group"],))
            add_2_0(c, (nid, x["parent"], persp, beh, None, None), ports, (), closures)
        ports_csv += [[nid, p[0], p[1], p[3], p[4], p[5]] for p in ports]
        report.append(["node", nid, f"nodes/{x['file'].name} ({n['group_id']})", f"groups/{x['group']}/nodes/{x['file'].name}", f"copied whole; block {beh}, parent {x['parent']}"])
        if n["group_id"] != x["group"]:
            report.append(["node group", nid, n["group_id"], x["group"], "case dissolved (decision S1-6)"])
    # the tree's blocks
    for b, x in sorted(blocks.items()):
        persp = {1: "programme", 2: "system"}.get(x["layer"], "subsystem")
        new_node(gdir[x["group"]] / "nodes" / f"{b}.node.tndb", b, x["group"], "group", x["label"], x["layer"],
                 content=[("identity", "question", x["about"], "spec:tree.json" if b in rows_ else BY)],
                 block=(b, x["parent"], persp, "children", None, None))
        report.append(["block", b, "spec/plan/tree.json" if b in rows_ else "the tree's structure", f"groups/{x['group']}/nodes/{b}.node.tndb", "a branch of the tree, now a block"])
    # the design loop, declared on the block that holds it
    loop_block = "gb"
    with sqlite3.connect(gdir[blocks[loop_block]["group"]] / "nodes" / f"{loop_block}.node.tndb") as c:
        c.execute('INSERT INTO "loop" VALUES (?, ?, ?, ?, ?)', ("design_loop", json.dumps(["mass", "inertia", "demand"]), None, None,
                  "sizing -> mass and inertia -> disturbance and slew demand -> sizing; iterated today by tools/pipeline_design.py (node_converge); its tolerance is to be stated by its owner"))
    report.append(["loop", "design_loop", "tools/pipeline_design.py (outside the tree)", f"groups/{blocks[loop_block]['group']}/nodes/{loop_block}.node.tndb", "declared on its block; tolerance open"])
    # the flight software's algorithms
    for nid, x in sorted(fsw.items()):
        fx = []
        for i, line in enumerate(x["vectors"]):
            parts = line.split()
            fx.append((f"{parts[0]} {parts[1]} {parts[2]}", json.dumps(parts[5:5 + int(parts[3])]), json.dumps(parts[5 + int(parts[3]):]), 0.0,
                       json.dumps({"provenance": "interpreter", "source": "fsw/tests/pcode_vectors.txt", "where": f"line {i + 1} of {parts[0]}"}), 0))
        doc = x["doc"].read_bytes().decode("utf-8") if x["doc"].is_file() else ""
        new_node(gdir[x["group"]] / "nodes" / f"{nid}.node.tndb", nid, x["group"], "leaf", f"Flight algorithms: {x['module']}", 3,
                 content=[("code", "pseudocode", x["text"], f"fsw/pseudocode/{x['file']}"), ("code", "uses", json.dumps(TOOLBOX_MODULES), BY),
                          ("explain", "theory", doc, f"fsw/pseudocode/{x['doc'].name}"),
                          ("identity", "question", f"What the flight software's {x['module']} algorithms compute, operation for operation", BY)],
                 fixtures=fx, block=(nid, groups[x["group"]]["mounts"][0]["on"], "subsystem", "method", None, None))
        report.append(["flight algorithms", x["file"], f"fsw/pseudocode/{x['file']}", f"groups/{x['group']}/nodes/{nid}.node.tndb", f"method; {len(fx)} vectors as its cases"])
    # the flight parameters: each a stated block of the flight software's group, with its rules
    for i, f in enumerate(params["field"]):
        nid = f"fsw_param_{f['name']}"
        rules = {k: v for k, v in f.items() if k not in ("name", "doc")}
        unit = re.search(r"\[([^\]]+)\]", f.get("doc", ""))
        lo = f.get("min")
        hi = f.get("max") if isinstance(f.get("max"), (int, float)) else None
        new_node(gdir["fsw"] / "nodes" / f"{nid}.node.tndb", nid, "fsw", "leaf", f.get("doc", f["name"]), 3,
                 content=[("identity", "question", f.get("doc", ""), "fsw/params/params.toml"), ("spec", "kind", "declared", "fsw/params/params.toml"),
                          ("spec", "rules", json.dumps(rules, sort_keys=True), "fsw/params/params.toml"), ("spec", "order", str(i), "fsw/params/params.toml")],
                 outputs=[(f["name"], unit.group(1) if unit else None, lo, hi, "params.toml min" if lo is not None else None, "params.toml max" if hi is not None else None)],
                 block=(nid, "gx", "subsystem", "stated", None, None),
                 ports=[(f["name"], "out", "number" if f["type"] == "f64" and not f.get("shape") else ("whole number" if not f.get("shape") else "list"),
                         "open", "estimated", None, None, None, None)])
        report.append(["flight parameter", f["name"], "fsw/params/params.toml", f"groups/fsw/nodes/{nid}.node.tndb", "stated block; its value comes from the case and the product (adcs_sim::config), as today"])
        ports_csv.append([nid, f["name"], "out", "open", "estimated", None])
    # the IGRF coefficients: a lookup of the environment group
    new_node(gdir["env"] / "nodes" / "env_igrf13_coefficients.node.tndb", "env_igrf13_coefficients", "env", "leaf", "IGRF-13 coefficients", 2,
             content=[("table", "igrf13coeffs.txt", igrf.read_bytes().decode("utf-8"), "matlab_sils/data/igrf13coeffs.txt (IAGA)"),
                      ("identity", "question", "The geomagnetic reference field's Gauss coefficients (IAGA IGRF-13)", BY)],
             block=("env_igrf13_coefficients", "m3", "system", "lookup", None, None))
    report.append(["table", "igrf13coeffs.txt", "matlab_sils/data/igrf13coeffs.txt", "groups/env/nodes/env_igrf13_coefficients.node.tndb", "lookup block"])
    # the library data
    for x in library:
        parent = groups[x["group"]]["mounts"][0]["on"] if groups[x["group"]]["mounts"] else "mgm"
        new_node(gdir[x["group"]] / "nodes" / f"{x['id']}.node.tndb", x["id"], x["group"], "leaf", f"{x['what']}: {x['rel']}", 1 if x["group"] in ("programme", "catalogue") else 2,
                 content=[("table", pathlib.Path(x["rel"]).name, x["file"].read_bytes().decode("utf-8"), x["rel"]), ("identity", "question", f"{x['what']}, as {x['rel']} states it", BY)],
                 block=(x["id"], parent, "programme" if x["group"] in ("programme", "catalogue") else "system", "lookup", None, None))
        report.append(["library", x["rel"], x["rel"], f"groups/{x['group']}/nodes/{x['id']}.node.tndb", "lookup block, the file's text kept whole"])
    # the delivery waves: a stated block of the programme
    waves = {g: x.get("wave") for g, x in g1.items()}
    new_node(gdir["programme"] / "nodes" / "programme_delivery_waves.node.tndb", "programme_delivery_waves", "programme", "leaf", "Delivery waves (1.0.0)", 1,
             content=[("table", "waves", json.dumps(waves, sort_keys=True), "design/groups.toml"), ("identity", "question", "The order groups were delivered in 1.0.0", BY)],
             block=("programme_delivery_waves", "mgm", "programme", "lookup", None, None))
    report.append(["library", "design/groups.toml (wave)", "design/groups.toml", "groups/programme/nodes/programme_delivery_waves.node.tndb", "lookup block"])

    # the nodes the developer's revisions add (their content follows with the revisions, below)
    added = add_nodes(gdir, groups, revisions(), report)

    # ------------------------------------------------------------ the group files
    placed = {}
    for gid in groups:
        placed[gid] = sorted(f.name[:-len(".node.tndb")] for f in (gdir[gid] / "nodes").glob("*.node.tndb"))
    old_groups = {}
    for f in sorted((src / "structure").glob("*.group.tndb")):
        with sqlite3.connect(f) as c:
            old_groups[f.name[:-len(".group.tndb")]] = {t: rows(c, f'SELECT * FROM "{t}"') for t in ("group_info", "stage", "member", "member_node", "edge", "contract")}
    owner = {nid: gid for gid, ns in placed.items() for nid in ns}
    edges = [e for g in old_groups.values() for e in g["edge"]]
    contracts = [x for g in old_groups.values() for x in g["contract"]]
    wires_csv = []
    for gid, g in groups.items():
        og = old_groups.get(gid, {})
        info = (og.get("group_info") or [{"id": gid, "label": g["label"], "lead_team": None, "lead": None}])[0]

        def fill(c, gid=gid, g=g, og=og, info=info):
            c.execute("INSERT INTO group_info VALUES (?, ?, ?, ?)", (gid, info.get("label") or g["label"], info.get("lead_team"), info.get("lead")))
            c.executemany("INSERT INTO stage VALUES (?, ?, ?)", [(s["id"], s["label"], s["owner"]) for s in og.get("stage", [])])
            c.executemany("INSERT INTO member VALUES (?, ?)", [(m["name"], m["role"]) for m in og.get("member", [])])
            for nid in placed[gid]:
                with sqlite3.connect(gdir[gid] / "nodes" / f"{nid}.node.tndb") as nc:
                    n = rows(nc, "SELECT * FROM node")[0]
                c.execute("INSERT INTO group_node VALUES (?, ?, ?, ?, ?, ?, ?)", (nid, n["sheet"], n["stage"], n["layer"], n["kind"], n["label"], n["state"]))
            mine = set(placed[gid])
            for e in edges:
                if e["to_node"] in mine:
                    c.execute("INSERT INTO edge VALUES (?, ?, ?, ?)", (e["from_node"], e["to_node"], e["kind"], e["label"]))
                    wires_csv.append([e["from_node"], owner.get(e["from_node"]), e["to_node"], gid, e["kind"]])
            for x in contracts:
                if x["node"] in mine:
                    c.execute("INSERT INTO contract VALUES (?, ?, ?, ?, ?)", (x["node"], x["output"], x["unit"], x["version"], x["readers"]))
            parent_group = None
            if g["mounts"]:
                on = g["mounts"][0]["on"]
                parent_group = owner.get(on) or blocks.get(on, {}).get("group")
            c.execute("INSERT INTO group_frame VALUES (?, ?, ?, ?, ?, ?)", (gid, parent_group, None, None, None, 0))
            c.executemany("INSERT INTO mount VALUES (?, ?, ?)", [(m["on"], m["via"], 1) for m in g["mounts"]])
        tndb.create(gdir[gid] / f"{gid}.group.tndb", "group", gid, written_by=BY, fill=fill, sync=False)
        report.append(["group", gid, f"structure/{gid}.group.tndb" if gid in old_groups else "new (decision S1-4)", f"groups/{gid}/{gid}.group.tndb",
                       f"{len(placed[gid])} nodes, {len(g['mounts'])} mount(s)"])
    report.append(["group", "case", "structure/case.group.tndb", "dissolved", "its nodes to programme and systems; each customer a case file"])

    # ------------------------------------------------------------ the cases
    for folder, pat, kind in CASES:
        for f in sorted((v1_root(folder) / folder).glob(pat)):
            rel = f.relative_to(v1_root(folder)).as_posix()
            cid = f.stem if kind == "case" else f"{kind}_{f.stem}"
            path = out / "cases" / f"{cid}.tncase"
            if path.exists():                 # a reference case held twice (spec and matlab_sils): the same name, kept once
                if (out / "cases" / f"{cid}.tncase").exists() and _case_text(path) == f.read_bytes().decode("utf-8"):
                    report.append(["case", rel, rel, f"cases/{cid}.tncase", "the same case as already converted"])
                    continue
                cid = f"{cid}_{folder.split('/')[0]}"
                path = out / "cases" / f"{cid}.tncase"
            text = f.read_bytes().decode("utf-8")

            def fill(c, text=text, kind=kind, cid=cid, rel=rel, f=f, folder=folder):
                c.execute("INSERT INTO case_info VALUES (?, ?, ?, ?, ?, ?)", (cid, None, cid, f.stem if kind == "scenario" else None,
                                                                              f.stem if kind == "campaign" else None, f"{kind}, from {rel}"))
                c.execute("INSERT INTO case_source VALUES (?, ?, ?)", (f.name, kind, text))
                # which cases the engine flies: today, the cases under matlab_sils/cases and every
                # scenario, campaign and trade; the plan's own copies (spec/plan/cases) are kept, not flown
                c.execute('INSERT OR REPLACE INTO meta VALUES (?, ?)', ("flown", "no" if folder == "spec/plan/cases" else "yes"))
                if kind == "case":
                    # each line with the node that declares its value (spec/plan/case_inputs.toml), as the
                    # engine's inputs hold it today (tools/design_inputs.py); its own text kept as written
                    try:
                        lines = [r[1:] for r in design_inputs.case_rows(f)]
                    except SystemExit:
                        node_of = design_inputs._nodes_of_keys()
                        lines = []
                        for i, ln in enumerate([x for x in text.split("\n")[1:] if x.strip()], 1):
                            r = (next(csv.reader(io.StringIO(ln))) + [""] * 9)[:9]
                            lines.append((i, *r, node_of.get(r[1].strip()), ln))
                    c.executemany("INSERT INTO case_line VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)", lines)
            tndb.create(path, "case", cid, written_by=BY, fill=fill, sync=False)
            report.append(["case", rel, rel, f"cases/{cid}.tncase", f"{kind}; its text kept whole" + ("; its lines one by one" if kind == "case" else "")])

    # ------------------------------------------------------------ the developer's revisions, then the baseline releases
    revised = revise(gdir, revisions(), report)
    for nid, ws in added.items():
        revised[nid] = ws + revised.get(nid, [])
    for gid in groups:
        seal_baseline(gdir[gid], gid, revised)

    # ------------------------------------------------------------ readable copies
    def write_csv(name, head, body):
        with open(out / "readable" / name, "w", newline="", encoding="utf-8") as fh:
            w = csv.writer(fh)
            w.writerow(head)
            w.writerows(body)
    write_csv("Groups.csv", ["group", "label", "nodes", "mounts on"], [[gid, g["label"], len(placed[gid]), " ".join(m["on"] for m in g["mounts"])] for gid, g in groups.items()])
    nodes_csv = []
    for gid in groups:
        for nid in placed[gid]:
            with sqlite3.connect(gdir[gid] / "nodes" / f"{nid}.node.tndb") as c:
                n = rows(c, "SELECT * FROM node")[0]
                b = rows(c, "SELECT * FROM block")[0]
            nodes_csv.append([nid, gid, n["label"], n["kind"], b["parent"], b["perspective"], b["behaviour"]])
    write_csv("Nodes.csv", ["node", "group", "label", "kind", "parent", "perspective", "behaviour"], nodes_csv)
    write_csv("Ports.csv", ["node", "port", "direction", "state", "maturity", "sense"], ports_csv)
    write_csv("Wires.csv", ["from node", "from group", "to node", "to group", "kind"], wires_csv)
    write_csv("Closures.csv", ["closure", "requirement", "achieved", "sense", "by", "metric"], closures_csv)
    # what a person must still state: a node with no port (1.0.0 declared no output for it), and an
    # open value with no range (the rules invent neither)
    with_port = {r[0] for r in ports_csv}
    for gid in groups:
        for nid in placed[gid]:
            if nid not in with_port and nid not in blocks:
                report.append(["to state", nid, "no output declared in 1.0.0", f"groups/{gid}/nodes/{nid}.node.tndb", "no port yet: its owner states its answer's name, unit and range"])
    ranged = set()
    for gid in groups:
        for nid in placed[gid]:
            with sqlite3.connect(gdir[gid] / "nodes" / f"{nid}.node.tndb") as c:
                ranged |= {(nid, r[0]) for r in c.execute("SELECT name FROM output WHERE lower IS NOT NULL OR upper IS NOT NULL")}
    for r in ports_csv:
        if r[2] == "out" and r[3] == "open" and (r[0], r[1]) not in ranged:
            report.append(["to state", f"{r[0]}.{r[1]}", "open, no range in 1.0.0", r[0], "an open value with no range: its owner states the range it may still take"])
    builtin = []
    for gid in groups:
        for nid in placed[gid]:
            with sqlite3.connect(gdir[gid] / "nodes" / f"{nid}.node.tndb") as c:
                if c.execute("SELECT behaviour FROM block").fetchone()[0] != "built-in":
                    continue
                code = {f"{r[0]}.{r[1]}": r[2] for r in c.execute("SELECT section, field, value FROM content WHERE section = 'code'")}
                kind = c.execute("SELECT kind FROM node").fetchone()[0]
            builtin.append([nid, gid, kind, code.get("code.rust", ""), code.get("code.c", ""), code.get("code.twin", "")])
    write_csv("BuiltIn.csv", ["node", "group", "kind", "rust", "c", "twin"], builtin)
    write_csv("Conversion.csv", ["what", "id", "was", "is now", "how"], report)
    if tmp:
        tmp.cleanup()
    return {"groups": len(groups), "nodes": sum(len(v) for v in placed.values()), "cases": len(list((out / "cases").glob("*.tncase"))),
            "converted_1_0_nodes": len(nodes), "blocks": len(blocks), "fsw": len(fsw), "params": len(params["field"]), "library": len(library)}


def _case_text(path):
    with sqlite3.connect(path) as c:
        r = c.execute("SELECT text FROM case_source").fetchone()
    return r[0] if r else None


def seal_baseline(gd, gid, revised=None):
    """Release 0.1 of a group, sealed by the conversion: every node as it was converted, unconfirmed,
    with why (tools/release.py's rules hold for it), and, for a node the developer revised, each revision's why."""
    gf = gd / f"{gid}.group.tndb"
    rel_rows = []
    for f in sorted((gd / "nodes").glob("*.node.tndb")):
        with sqlite3.connect(f) as c:
            node = rows(c, "SELECT id, sheet, group_id, stage, layer, kind, label, author, contract_version FROM node")[0]
            body = {"node": node,
                    "content": [list(r.values()) for r in rows(c, "SELECT section, field, value, origin FROM content WHERE section <> 'status' ORDER BY section, field, rowid")],
                    "input": [list(r.values()) for r in rows(c, "SELECT name, from_node, from_output, unit FROM input ORDER BY name")],
                    "output": [list(r.values()) for r in rows(c, "SELECT name, unit, lower, upper, reason_lower, reason_upper FROM output ORDER BY name")],
                    "fixture": [list(r.values()) for r in rows(c, "SELECT name, inputs, expected, tolerance, source, outside FROM fixture ORDER BY name")],
                    "attachment": [], "signature": [list(r.values()) for r in rows(c, "SELECT role, name, at, statement FROM signature ORDER BY at, role, name")],
                    "block": [list(r.values()) for r in rows(c, "SELECT * FROM block")],
                    "port": [list(r.values()) for r in rows(c, "SELECT * FROM port ORDER BY name, direction")],
                    "loop": [list(r.values()) for r in rows(c, 'SELECT * FROM "loop" ORDER BY id')],
                    "closure": [list(r.values()) for r in rows(c, "SELECT * FROM closure ORDER BY id")]}
        body_text = json.dumps(body, ensure_ascii=False, separators=(",", ":"))
        content = json.dumps({"body": body_text, "body_fingerprint": sha(body_text), "sealed_as": "unconfirmed", "why": [WHY, *(revised or {}).get(node["id"], [])],
                              "work": "converted", "author": None, "stage": node["stage"]}, ensure_ascii=False, separators=(",", ":"))
        rel_rows.append((node["id"], sha(content), content))
    fp = sha("\n".join(sorted(f"{i} {h}" for i, h, _ in rel_rows)))
    with sqlite3.connect(gf) as c:
        info = rows(c, "SELECT * FROM group_info")[0]
        stages, gnodes = rows(c, "SELECT * FROM stage"), rows(c, "SELECT * FROM group_node")
        edges, contracts = rows(c, "SELECT * FROM edge"), rows(c, "SELECT * FROM contract")
        frame, mounts = rows(c, "SELECT * FROM group_frame"), rows(c, "SELECT * FROM mount")

    def fill(c):
        c.execute("INSERT INTO release VALUES (?, ?, ?, ?, ?)", (gid, "0.1", AT, BY, fp))
        c.executemany("INSERT INTO release_node VALUES (?, ?, ?)", rel_rows)
        c.execute("INSERT INTO group_info VALUES (?, ?, ?, ?)", tuple(info.values()))
        c.executemany("INSERT INTO stage VALUES (?, ?, ?)", [tuple(s.values()) for s in stages])
        c.executemany("INSERT INTO group_node VALUES (?, ?, ?, ?, ?, ?, ?)", [tuple(n.values()) for n in gnodes])
        c.executemany("INSERT INTO edge VALUES (?, ?, ?, ?)", [tuple(e.values()) for e in edges])
        c.executemany("INSERT INTO contract VALUES (?, ?, ?, ?, ?)", [tuple(x.values()) for x in contracts])
        c.executemany("INSERT INTO group_frame VALUES (?, ?, ?, ?, ?, ?)", [tuple(x.values()) for x in frame])
        c.executemany("INSERT INTO mount VALUES (?, ?, ?)", [tuple(x.values()) for x in mounts])
        c.execute("INSERT INTO signature VALUES (?, ?, ?, ?)", ("sealed", BY, AT, json.dumps({"statement": "sealed 0.1, the converted baseline", "version": "0.1", "fingerprint": fp})))
    tndb.create(gd / "releases" / f"{gid}-0.1.tnrel", "release", f"{gid}-0.1", written_by=BY, fill=fill, sync=False)


def check_folder(out):
    """Every problem with a converted folder: each file against the schema; each group's baseline
    release by tools/release.py; the tree's shape (every node in one group, every parent a block,
    every mount a block, every group file listing exactly its folder's nodes)."""
    out = pathlib.Path(out)
    problems = []
    files = sorted(out.rglob("*.tndb")) + sorted(out.rglob("*.tnrel")) + sorted(out.rglob("*.tncase"))
    for f in files:
        problems += tndb.check(f)
    ids, parents, mounts = {}, {}, []
    for gd in sorted((out / "groups").iterdir()):
        gid = gd.name
        with sqlite3.connect(gd / f"{gid}.group.tndb") as c:
            listed = {r[0] for r in c.execute("SELECT id FROM group_node")}
            mounts += [(gid, r[0]) for r in c.execute("SELECT on_block FROM mount")]
        held = {f.name[:-len(".node.tndb")] for f in (gd / "nodes").glob("*.node.tndb")}
        if listed != held:
            problems.append(f"groups/{gid}: its group file lists {sorted(listed - held)[:5]} it does not hold, and holds {sorted(held - listed)[:5]} it does not list")
        for nid in held:
            if nid in ids:
                problems.append(f"node {nid} is in two groups: {ids[nid]} and {gid}")
            ids[nid] = gid
            with sqlite3.connect(gd / "nodes" / f"{nid}.node.tndb") as c:
                b = c.execute("SELECT parent FROM block").fetchall()
                g = c.execute("SELECT group_id FROM node").fetchone()[0]
            if len(b) != 1:
                problems.append(f"groups/{gid}/nodes/{nid}: {len(b)} block rows, not one")
            else:
                parents[nid] = b[0][0]
            if g != gid:
                problems.append(f"groups/{gid}/nodes/{nid}: names group {g}")
        for r in sorted((gd / "releases").glob("*.tnrel")):
            problems += release.check(r, rows={})
    for nid, p in parents.items():
        if p is not None and p not in ids:
            problems.append(f"{nid}: its parent {p} is no block of the design")
    roots = sorted(n for n, p in parents.items() if p is None)
    if roots != ["mgm"]:
        problems.append(f"the tree has {len(roots)} roots, not one (mgm): {roots[:8]}")
    for gid, on in mounts:
        if on not in ids:
            problems.append(f"groups/{gid}: mounts on {on}, which is no block of the design")
    return problems


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--out", required=True)
    ap.add_argument("--check", action="store_true")
    a = ap.parse_args(argv)
    s = convert(a.out)
    print("convert: " + ", ".join(f"{k} {v}" for k, v in s.items()))
    if a.check:
        p = check_folder(a.out)
        for x in p:
            print("convert: " + x, file=sys.stderr)
        print(f"convert: checked, {len(p)} problem(s)")
        return 1 if p else 0
    return 0


if __name__ == "__main__":
    sys.exit(main())
