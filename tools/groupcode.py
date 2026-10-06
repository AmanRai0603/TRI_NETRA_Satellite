#!/usr/bin/env python3
"""Each group's code, generated from its nodes (docs/RELEASE_PLAN.md P10): every computing row's
pseudocode wired into one module per group, translated to Rust (engine/crates/adcs-groups) and to
MATLAB (matlab_sils/+asils/+groups), tested against the interpreter on drawn vectors and against the
nodes' own test vectors, and delivered to each group as a test app (one offline page that runs the
group's rows in the browser's interpreter and in WebAssembly built from the generated Rust).

    python3 tools/groupcode.py wire [--design DIR] [--check]
        each group's computing rows into design/groups/<group>.pc and <group>.wire.json, from the
        merged design (DIR/design.tndb after tools/group.py merge) or, without DIR, from the design
        as seeded and carried over (tools/seed_design.py, tools/carry_over.py)
    python3 tools/groupcode.py gen [--check]
        the Rust crate (engine/crates/adcs-groups), its WebAssembly face (adcs-groups-wasm), the
        MATLAB package and the vectors, from design/groups/
    python3 tools/groupcode.py test
        cargo test -p adcs-groups: translator = interpreter on every drawn vector, and every node's
        own test vectors reproduced by the generated Rust
    python3 tools/groupcode.py deliver [--out dist/test-apps]
        a test app per group: build/pages/testapp.html with the group's rows, their pseudocode and the
        WebAssembly module inside

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
import pcode
from common import ROOT, write_text

SRC = ROOT / "design" / "groups"
CRATE = ROOT / "engine" / "crates" / "adcs-groups"
WASM = ROOT / "engine" / "crates" / "adcs-groups-wasm"
MPKG = "asils.groups"
MDIR = ROOT / "matlab_sils" / "+asils" / "+groups"
TWIN_VECTORS = ROOT / "matlab_sils" / "data" / "groups_vectors.json"
ABOUT = "TRI-NETRA groups: every computing row's pseudocode, a module per group, translated by tools/groupcode.py"
N_VECTORS = 8
SCALAR = re.compile(r"^real(\[[^\]]*\])?(\s+in\s.*)?$")     # a plain number with its unit, not an array
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


# ------------------------------------------------------------------ generated code

FIXTURES_RS = """//! Every node's own test vectors (answers from outside the code), through the generated Rust.
//! Generated by tools/groupcode.py gen from design/groups/*.wire.json; do not edit.

#[test]
fn every_node_reproduces_its_own_test_vectors() {
    let v: serde_json::Value = serde_json::from_str(include_str!("fixtures.json")).unwrap();
    let mut n = 0;
    for f in v.as_array().unwrap() {
        let name = f["call"].as_str().unwrap();
        let x: Vec<f64> = f["inputs"].as_array().unwrap().iter().map(|a| a.as_f64().unwrap()).collect();
        let got = adcs_groups::dispatch::call(name, &x).unwrap_or_else(|| panic!("{name}: no such function"))[0];
        let (want, tol) = (f["expected"].as_f64().unwrap(), f["tolerance"].as_f64().unwrap());
        let err = (got - want).abs();
        assert!(err <= tol * want.abs().max(if want == 0.0 { 1.0 } else { 0.0 }),
                "{} {}: Rust gives {got:e}, the node expects {want:e} within {tol:e}", f["node"].as_str().unwrap(), f["vector"].as_str().unwrap());
        n += 1;
    }
    assert!(n > 0, "no test vector at all");
}
"""

WASM_CARGO = """# Generated by tools/groupcode.py gen; do not edit.
[package]
name = "adcs-groups-wasm"
description = "The groups' generated code as WebAssembly, for the test apps' Try it"
version.workspace = true
edition.workspace = true
authors.workspace = true
publish.workspace = true

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
adcs-groups = { path = "../adcs-groups" }
"""

WASM_LIB = """//! The groups' generated code as WebAssembly (tools/groupcode.py gen; do not edit): one entry,
//! `call`, that runs a function by its index in NAMES on the doubles in the input buffer.
#![allow(clippy::missing_safety_doc)]

pub const NAMES: &[&str] = &[NAMES_HERE];
static mut BUF_IN: [f64; 256] = [0.0; 256];
static mut BUF_OUT: [f64; 64] = [0.0; 64];

#[no_mangle]
pub extern "C" fn input() -> *mut f64 { core::ptr::addr_of_mut!(BUF_IN) as *mut f64 }
#[no_mangle]
pub extern "C" fn output() -> *const f64 { core::ptr::addr_of!(BUF_OUT) as *const f64 }

/// Run NAMES[which] on the first `n` inputs; the number of outputs written, or -1.
#[no_mangle]
pub extern "C" fn call(which: u32, n: u32) -> i32 {
    let Some(name) = NAMES.get(which as usize) else { return -1 };
    let x = unsafe { core::slice::from_raw_parts(core::ptr::addr_of!(BUF_IN) as *const f64, n.min(256) as usize) };
    match adcs_groups::dispatch::call(name, x) {
        Some(y) => {
            let out = unsafe { core::slice::from_raw_parts_mut(core::ptr::addr_of_mut!(BUF_OUT) as *mut f64, 64) };
            for (o, v) in out.iter_mut().zip(&y) { *o = *v; }
            y.len().min(64) as i32
        }
        None => -1,
    }
}
"""


def generated():
    files = sorted(SRC.glob("*.pc"))
    wires = [json.loads(f.read_text(encoding="utf-8")) for f in sorted(SRC.glob("*.wire.json"))]
    out = {}
    if not files:
        return out
    for rel, text in pcode.cli("rust", *map(str, files), "--title", ABOUT).items():
        out[CRATE / rel] = text
    out[CRATE / "Cargo.toml"] = pcode.CARGO.format(name="adcs-groups", about=ABOUT)
    out[CRATE / "tests" / "vectors.rs"] = pcode.VECTORS_RS.replace("CRATE", "adcs_groups")
    vec = pcode.cli("vectors", *map(str, files), "--n", N_VECTORS, "--seed", pcode.SEED_VECTORS)
    # a function that takes or gives a record (a flight state) has no plain-numbers call: its vectors
    # are the flight software's own (fsw/tests, C and Rust); here only what the dispatcher can call
    plain = re.compile(r"^(real|int|bool)(\[[^\]]*\])*$")
    callable = {f"{f['module']}::{f['name']}" for f in pcode.cli("signatures", *map(str, files))
                if all(plain.match(x["type"]) for x in f["inputs"] + f["outputs"])}
    vec = {k: v for k, v in vec.items() if k in callable}
    text = json.dumps({"generated_by": "tools/groupcode.py gen (the interpreter, design/js/pcode.js)", "vectors": vec}, indent=1, sort_keys=True) + "\n"
    out[CRATE / "tests" / "vectors.json"] = text
    out[TWIN_VECTORS] = text
    fixtures = [{"node": r["id"], "vector": v["name"], "call": f"{'shared' if r['fn'] in w.get('shared', []) else w['group']}::{r['fn']}", "inputs": v["inputs"], "expected": v["expected"], "tolerance": v["tolerance"]}
                for w in wires for r in w["rows"] for v in r["vectors"]
                if r["fn"] and len(r["outputs"]) == 1 and all(SCALAR.match(t) for _, t in r["params"] + r["outputs"])]
    out[CRATE / "tests" / "fixtures.json"] = json.dumps(fixtures, indent=1, sort_keys=True) + "\n"
    out[CRATE / "tests" / "fixtures.rs"] = FIXTURES_RS
    for rel, text in pcode.cli("matlab", *map(str, files), "--pkg", MPKG).items():
        out[MDIR / rel] = text
    names = sorted({f"{'shared' if r['fn'] in w.get('shared', []) else w['group']}::{r['fn']}" for w in wires for r in w["rows"] if r["fn"]} & callable)
    out[WASM / "Cargo.toml"] = WASM_CARGO
    out[WASM / "src" / "lib.rs"] = WASM_LIB.replace("NAMES_HERE", ", ".join(json.dumps(n) for n in names))
    return out


def _compare(files, check):
    stale = []
    for path, text in sorted(files.items()):
        cur = path.read_text(encoding="utf-8") if path.exists() else None
        if cur != text:
            stale.append(path)
            if not check:
                write_text(path, text)
    return stale


def _members_ok():
    t = (ROOT / "engine" / "Cargo.toml").read_text()
    return all(f'"crates/{c}"' in t for c in ("adcs-groups", "adcs-groups-wasm"))


# ------------------------------------------------------------------ test apps

def build_wasm():
    r = subprocess.run(["cargo", "build", "--locked", "--release", "--target", "wasm32-unknown-unknown", "-p", "adcs-groups-wasm"],
                       cwd=ROOT / "engine", capture_output=True, text=True)
    if r.returncode:
        raise SystemExit("groupcode: the WebAssembly build failed (rustup target add wasm32-unknown-unknown?)\n" + r.stderr[-3000:])
    return (ROOT / "engine" / "target" / "wasm32-unknown-unknown" / "release" / "adcs_groups_wasm.wasm").read_bytes()


def deliver(out):
    import pages
    out = pathlib.Path(out)
    out.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory() as d:
        page = pages.build(pathlib.Path(d))["testapp"][0].read_text(encoding="utf-8")
    wasm = base64.b64encode(build_wasm()).decode()
    names = json.loads(re.search(r"NAMES: &\[&str\] = &\[(.*?)\];", (WASM / "src" / "lib.rs").read_text(), re.S).group(1).join("[]"))
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
    g = sub.add_parser("gen")
    g.add_argument("--check", action="store_true")
    sub.add_parser("test")
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
    if a.cmd == "gen":
        files = generated()
        stale = _compare(files, a.check)
        bad = [] if _members_ok() else ["engine/Cargo.toml does not list crates/adcs-groups and crates/adcs-groups-wasm as members"]
        if a.check and (stale or bad):
            print("groupcode: not current: " + ", ".join(str(p.relative_to(ROOT)) for p in stale[:8]) + ("; " if stale and bad else "") + "; ".join(bad))
            return 1
        for b in bad:
            print("groupcode: " + b)
        print(f"groupcode: {len(files)} generated file(s)" + (" current" if a.check else f", {len(stale)} written"))
        return 1 if bad else 0
    if a.cmd == "test":
        r = subprocess.run(["cargo", "test", "--locked", "--release", "-q", "-p", "adcs-groups"], cwd=ROOT / "engine")
        return r.returncode
    made = deliver(a.out)
    print(f"groupcode: {len(made)} test app(s) in {a.out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
