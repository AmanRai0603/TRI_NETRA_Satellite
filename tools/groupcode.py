#!/usr/bin/env python3
"""Each group's code, wired from its nodes (docs/RELEASE_PLAN.md P10): every computing row's pseudocode
wired into one module per group (design/groups), and delivered to each group as a test app (one offline
page that runs the group's rows in the browser's interpreter and in WebAssembly built from the generated
Rust). The code is generated from the wiring by tools/engine_build.py (S7.16), folded with the design's
relations into one crate and one MATLAB package (engine/crates/adcs-relations, matlab_sils/+asils/+relations),
each relation once, and tested there against the interpreter's vectors and the nodes' own.

    python3 tools/groupcode.py wire [--design DIR] [--check]
        each group's computing rows into design/groups/<group>.pc and <group>.wire.json, from the
        merged design (DIR/design.tndb after tools/group.py merge) or, without DIR, from the design
        as seeded and carried over (tools/seed_design.py, tools/carry_over.py)
    python3 tools/groupcode.py deliver [--out dist/test-apps]
        a test app per group: build/pages/testapp.html with the group's rows, their pseudocode and the
        WebAssembly module (adcs-relations, built for wasm32-unknown-unknown) inside

After `wire`, `python3 tools/engine_build.py gen` writes the code and `cargo test -p adcs-relations` holds it.
No row's physics is written by hand here: a row with no pseudocode is listed in its group's test app
as having no code yet, with its owner team. Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import argparse
import base64
import json
import pathlib
import re
import shutil
import sqlite3
import subprocess
import sys
import tempfile

import carry_over
from common import ROOT, write_text

SRC = ROOT / "design" / "groups"
CRATE = ROOT / "engine" / "crates" / "adcs-relations"     # the groups' code, folded with the relations (tools/engine_build.py)
CARRIED = "the design as seeded and carried over (tools/seed_design.py, tools/carry_over.py); not yet sealed by its group"


# ------------------------------------------------------------------ the design's nodes

def _body_from_file(path):
    with sqlite3.connect(f"file:{path}?mode=ro", uri=True) as c:
        node = dict(zip(("id", "group_id", "label", "kind"), c.execute("SELECT id, group_id, label, kind FROM node").fetchone()))
        content = {(f"{s}.{f}" if f else s): v for s, f, v in c.execute("SELECT section, field, value FROM content")}
        fixtures = [dict(zip(("name", "inputs", "expected", "tolerance", "source", "outside"), r)) for r in c.execute("SELECT name, inputs, expected, tolerance, source, outside FROM fixture ORDER BY name")]
        return node, content, fixtures


def design_nodes(design_dir):
    """[(node, content, fixtures, release)] of the design: merged releases when there are, else the node files."""
    d = pathlib.Path(design_dir)
    db = d / "design.tndb"
    merged = []
    if db.exists():
        with sqlite3.connect(f"file:{db}?mode=ro", uri=True) as c:
            for nid, gid, label, kind, content, rel in c.execute("SELECT id, group_id, label, kind, content, release FROM design_node ORDER BY id"):
                try:
                    x = json.loads(content)
                except ValueError:
                    continue
                if not isinstance(x, dict) or "body" not in x:
                    merged = []
                    break
                b = x["body"]
                merged.append(({"id": nid, "group_id": gid, "label": label, "kind": kind},
                               {(f"{s}.{f}" if f else s): v for s, f, v, _o in b["content"]},
                               [dict(zip(("name", "inputs", "expected", "tolerance", "source", "outside"), r)) for r in b["fixture"]], rel))
    if merged:
        return merged
    out = []
    for f in sorted((d / "nodes").glob("*.node.tndb")):
        n, c, fx = _body_from_file(f)
        out.append((n, c, fx, None))
    return out


def _header(text, fn):
    """(params [(name, type)], outputs [(name, type)]) of `fn` in a pseudocode text."""
    m = re.search(rf"^fn\s+{fn}\((.*?)\)\s*->\s*(.+)$", re.sub(r"\\\n\s*", " ", text), re.M)
    if not m:
        return None, None
    split = lambda s: [x.strip() for x in re.split(r",(?![^\[]*\])", s) if x.strip()]  # noqa: E731
    params = [(p.split(":")[0].strip(), p.split(":", 1)[1].strip()) for p in split(m.group(1))]
    outs = m.group(2).strip()
    outs = outs[1:-1] if outs.startswith("(") else outs
    return params, [(o.split(":")[0].strip(), o.split(":", 1)[1].strip()) for o in split(outs)]


def wire(design_dir=None):
    """{group: (pc text, wire dict)} for every group."""
    tmp = None
    if design_dir is None:
        import seed_design
        tmp = tempfile.TemporaryDirectory()
        seed_design.seed(tmp.name, sync=False)
        carry_over.carry(tmp.name)
        design_dir, source = tmp.name, CARRIED
    else:
        source = None
    try:
        teams = {}
        for gf in sorted(pathlib.Path(design_dir, "structure").glob("*.group.tndb")):
            with sqlite3.connect(f"file:{gf}?mode=ro", uri=True) as c:
                teams[gf.name.split(".")[0]] = (c.execute("SELECT lead_team, label FROM group_info").fetchone() or (None, None))
        nodes = design_nodes(design_dir)
    finally:
        if tmp:
            tmp.cleanup()
    out = {}
    for gid in sorted(teams):
        mine = [x for x in nodes if x[0]["group_id"] == gid]
        blocks, conflicts, rows, without = {}, [], [], []
        for n, c, fx, rel in mine:
            text = c.get("code.pseudocode")
            if not text:
                if c.get("identity.form_kind") == "computed" or (n["kind"] in ("leaf", "internal", "added") and c.get("relation.expression")):
                    without.append({"id": n["id"], "label": n["label"]})
                continue
            p = pathlib.Path(tempfile.mkdtemp()) / "n.pc"
            p.write_text(text, encoding="utf-8")
            bs = carry_over.pc_blocks([p])
            shutil.rmtree(p.parent)
            for k, v in bs.items():
                if k in blocks and blocks[k][2] != v[2]:
                    conflicts.append(f"{gid}: {k} is written two ways (in {n['id']} and an earlier node)")
                blocks.setdefault(k, v)
            sym = c.get("output.symbol")
            fn = n["id"] if n["id"] in bs else next((k for k, v in bs.items() if v[0] == "fn" and carry_over.fn_outputs(v[2]) == sym), None)
            if fn is None:      # the node names its answer otherwise: the function its text defines last (what it calls comes first)
                fn = next((k for k, v in reversed(list(bs.items())) if v[0] == "fn"), None)
            params, outs = _header(bs[fn][2], fn) if fn else (None, None)
            vecs = []
            for x in fx:
                try:
                    ins = json.loads(x["inputs"] or "{}")
                    src = json.loads(x["source"] or "{}")
                except ValueError:
                    continue
                if x["expected"] in (None, "") or not params or any(pn not in ins for pn, _ in params):
                    continue
                vecs.append({"name": x["name"], "inputs": [ins[pn] for pn, _ in params], "expected": float(x["expected"]),
                             "tolerance": float(x["tolerance"] or 0), "outside": bool(x["outside"]), "provenance": src.get("provenance", "")})
            rows.append({"id": n["id"], "label": n["label"], "fn": fn, "symbol": sym, "release": rel or "carried",
                         "params": params or [], "outputs": outs or [], "vectors": vecs})
        if conflicts:
            raise SystemExit("groupcode: " + "; ".join(conflicts))
        w = {"group": gid, "label": teams[gid][1], "owner_team": teams[gid][0], "source": source or "the merged releases (DIR/design.tndb)",
             "rows": sorted(rows, key=lambda r: r["id"]), "without_code": sorted(without, key=lambda r: r["id"])}
        out[gid] = (blocks, w)
    return out


def wire_files(design_dir=None):
    """design/groups/: a module per group, and `shared.pc` for what several groups' rows call alike
    (the language has one namespace: a function written once, wherever it is)."""
    groups = wire(design_dir)
    users = {}
    for gid, (blocks, _) in groups.items():
        for k, v in blocks.items():
            users.setdefault(k, {}).setdefault(v[2], []).append(gid)
    clash = [f"{k} ({' / '.join(', '.join(g) for g in t.values())})" for k, t in users.items() if len(t) > 1]
    if clash:
        raise SystemExit("groupcode: written differently in different groups: " + "; ".join(sorted(clash)))
    shared = sorted(k for k, t in users.items() if len(next(iter(t.values()))) > 1)
    files = {}
    head = "## {what}. Generated by tools/groupcode.py wire from the design's nodes; do not edit.\nmodule {mod}\n\n"
    if shared:
        any_blocks = {k: v for _, (b, _) in groups.items() for k, v in b.items()}
        files[SRC / "shared.pc"] = head.format(what="What several groups' rows call alike, written once", mod="shared") + \
            "\n\n".join(any_blocks[k][2] for k in shared) + "\n"
    for gid, (blocks, w) in groups.items():
        own = [k for k in blocks if k not in shared]
        if own:
            files[SRC / f"{gid}.pc"] = head.format(what=f"{gid}: every computing row of the group, as its nodes state it", mod=gid) + \
                "\n\n".join(blocks[k][2] for k in own) + "\n"
        w["shared"] = sorted(k for k in blocks if k in shared)
        files[SRC / f"{gid}.wire.json"] = json.dumps(w, indent=1, sort_keys=True, ensure_ascii=False) + "\n"
    return files


def _compare(files, check):
    stale = []
    for path, text in sorted(files.items()):
        cur = path.read_text(encoding="utf-8") if path.exists() else None
        if cur != text:
            stale.append(path)
            if not check:
                write_text(path, text)
    return stale


# ------------------------------------------------------------------ test apps

def build_wasm():
    r = subprocess.run(["cargo", "build", "--locked", "--release", "--target", "wasm32-unknown-unknown", "-p", "adcs-relations"],
                       cwd=ROOT / "engine", capture_output=True, text=True)
    if r.returncode:
        raise SystemExit("groupcode: the WebAssembly build failed (rustup target add wasm32-unknown-unknown?)\n" + r.stderr[-3000:])
    return (ROOT / "engine" / "target" / "wasm32-unknown-unknown" / "release" / "adcs_relations.wasm").read_bytes()


def deliver(out):
    import pages
    out = pathlib.Path(out)
    out.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory() as d:
        page = pages.build(pathlib.Path(d))["testapp"][0].read_text(encoding="utf-8")
    wasm = base64.b64encode(build_wasm()).decode()
    names = json.loads(re.search(r"NAMES: &\[&str\] = &\[(.*?)\];", (CRATE / "src" / "wasm.rs").read_text(), re.S).group(1).join("[]"))
    made = []
    for wf in sorted(SRC.glob("*.wire.json")):
        w = json.loads(wf.read_text(encoding="utf-8"))
        srcs = [p for p in (SRC / "shared.pc", SRC / f"{w['group']}.pc") if p.exists() and (p.name != "shared.pc" or w.get("shared"))]
        data = {"wire": w, "sources": [{"file": p.name, "text": p.read_text(encoding="utf-8")} for p in srcs], "names": names}
        blob = json.dumps(data, ensure_ascii=False).replace("<", "\\u003c")
        html = page.replace('<script type="application/json" id="tn-testapp">null</script>', f'<script type="application/json" id="tn-testapp">{blob}</script>', 1) \
                   .replace('<script type="text/plain" id="tn-groups-wasm"></script>', f'<script type="text/plain" id="tn-groups-wasm">{wasm}</script>', 1)
        if blob not in html:
            raise SystemExit("groupcode: the test app page has no place for the group's data")
        p = out / f"{w['group']}.test-app.html"
        p.write_text(html, encoding="utf-8")
        made.append(p)
    return made


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    w = sub.add_parser("wire")
    w.add_argument("--design")
    w.add_argument("--check", action="store_true")
    d = sub.add_parser("deliver")
    d.add_argument("--out", default=str(ROOT / "dist" / "test-apps"))
    a = ap.parse_args(argv)
    if a.cmd == "wire":
        files = wire_files(a.design)
        stale = _compare(files, a.check)
        gone = [p for p in SRC.glob("*") if p not in files] if SRC.exists() else []
        if not a.check:
            for p in gone:
                p.unlink()
        if a.check and (stale or gone):
            print("groupcode: design/groups/ is not the design's wiring: " + ", ".join(p.name for p in stale + gone) + " (python3 tools/groupcode.py wire)")
            return 1
        rows = sum(len(json.loads(t)["rows"]) for p, t in files.items() if p.suffix == ".json")
        print(f"groupcode: {sum(1 for p in files if p.suffix == '.pc')} group module(s), {rows} computing row(s) wired" + (" (current)" if a.check else ""))
        return 0
    made = deliver(a.out)
    print(f"groupcode: {len(made)} test app(s) in {a.out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
