#!/usr/bin/env python3
"""Test, deliver, accept, ship: every group through the loop in five waves (docs/RELEASE_PLAN.md
P12, docs/DELIVERY.md). The developer side delivers a group's sealed release; its lead accepts the
delivery in the group app (Release → Deliveries → Accept); what ships says, group by group, which
lead accepted which version, and which group goes visibly UNCONFIRMED and why.

    python3 tools/delivery.py deliver DIR [GROUP ...] [--wave X] [--no-test]
            each named group's latest release (or every released group of the wave): verified, the
            generated code checked to be the release's, the group's tests run, its test app built,
            and DIR/deliveries/<group>-<version>.delivery.json and .md written beside the test app
    python3 tools/delivery.py status DIR [--json]
            wave by wave, group by group: released, delivered, accepted (by whom), or what is missing
    python3 tools/delivery.py ship DIR [--out FILE] [--require-accepted]
            the shipping record (JSON, and Markdown beside it): every group accepted, or UNCONFIRMED
            with why; with --require-accepted, exit 1 unless every group is accepted

The rules:
  - a group is delivered in its wave's order: every group of an earlier wave that has a release has
    its latest release delivered first (`--out-of-order` names a reason and records it);
  - a delivery is of a release that `tools/group.py verify` passes, whose wiring (tools/groupcode.py)
    is the repository's generated code: the functions its nodes state and every computing row's
    function, inputs and test vectors are those of design/groups/ (else: wire, gen and test it first);
  - the group's tests are the generated Rust reproducing the interpreter and every node's own test
    vectors (`cargo test -p adcs-relations`, the crate tools/engine_build.py writes from the wiring), recorded pass
    or fail in the delivery;
  - an acceptance is the lead's signature in the group file, naming the version, the release's
    fingerprint and the delivery's (SHA-256 of its JSON); one that names another fingerprint, or
    is by someone who is not the group's lead, does not count;
  - shipping never hides a group: one not released, not delivered or not accepted ships UNCONFIRMED
    with the reason (the owner's decision, docs/RELEASE_PLAN.md §9).

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import argparse
import datetime
import hashlib
import json
import pathlib
import shutil
import sqlite3
import subprocess
import sys
import tempfile

import groups as G
from common import ROOT, write_text

DELIVERIES = "deliveries"
SCHEMA = "trinetra-delivery/1"


def _sha(b):
    return hashlib.sha256(b if isinstance(b, bytes) else b.encode("utf-8")).hexdigest()


def utc_now():
    return datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def waves():
    """[(wave, label, [groups])] in order."""
    w = G.wave_of(G.load())
    return [(k, lab, sorted(g for g, x in w.items() if x == k)) for k, lab in G.WAVES.items()]


def _release_row(path):
    with sqlite3.connect(f"file:{path}?mode=ro", uri=True) as c:
        c.row_factory = sqlite3.Row
        r = dict(c.execute("SELECT * FROM release").fetchone())
        sealed = [json.loads(x)["sealed_as"] for (x,) in c.execute("SELECT content FROM release_node")]
    r["nodes"], r["confirmed"] = len(sealed), sealed.count("confirmed")
    return r


def _group_people(root, gid):
    """(lead, [signatures]) of a group file."""
    f = pathlib.Path(root) / "structure" / f"{gid}.group.tndb"
    with sqlite3.connect(f"file:{f}?mode=ro", uri=True) as c:
        info = c.execute("SELECT lead FROM group_info").fetchone()
        leads = [r[0] for r in c.execute("SELECT name FROM member WHERE role = 'lead'")]
        sigs = [dict(zip(("role", "name", "at", "statement"), r)) for r in c.execute("SELECT role, name, at, statement FROM signature ORDER BY at")]
    lead = (info[0] if info and info[0] else None) or (leads[0] if leads else None)
    return lead, set(leads) | ({info[0]} if info and info[0] else set()), sigs


def delivery_path(root, gid, version):
    return pathlib.Path(root) / DELIVERIES / f"{gid}-{version}.delivery.json"


def _code_differences(root, gid):
    """Where the release's wiring of `gid` differs from the repository's generated code (empty: the same)."""
    import carry_over
    import groupcode
    now = groupcode.wire(str(root)).get(gid)
    if now is None:
        return [f"{gid} has no nodes in the design"]
    blocks, w = now
    have_w = groupcode.SRC / f"{gid}.wire.json"
    if not have_w.is_file():
        return [f"design/groups/{gid}.wire.json does not exist"]
    hw = json.loads(have_w.read_text(encoding="utf-8"))
    files = [p for p in (groupcode.SRC / f"{gid}.pc", groupcode.SRC / "shared.pc") if p.is_file()]
    committed = {}
    for p in files:
        bs = carry_over.pc_blocks([p])
        committed.update({k: v[2] for k, v in bs.items() if p.name != "shared.pc" or k in hw.get("shared", [])})
    out = []
    mine = {k: v[2] for k, v in blocks.items()}
    for k in sorted(set(mine) | set(committed)):
        if mine.get(k) != committed.get(k):
            out.append(f"function {k}: " + ("not in the generated code" if k not in committed else "no longer stated by the nodes" if k not in mine else "stated otherwise than generated"))
    key = lambda r: (r["id"], r["fn"], json.dumps(r["params"]), json.dumps(r["outputs"]), json.dumps(r["vectors"], sort_keys=True))  # noqa: E731
    a, b = {r["id"]: key(r) for r in w["rows"]}, {r["id"]: key(r) for r in hw["rows"]}
    for n in sorted(set(a) | set(b)):
        if a.get(n) != b.get(n):
            out.append(f"row {n}: its function, inputs or test vectors differ from the generated code's")
    return out


def _tests():
    r = subprocess.run(["cargo", "test", "--locked", "--release", "-q", "-p", "adcs-relations"], cwd=ROOT / "engine", capture_output=True, text=True)
    tail = (r.stdout + r.stderr).strip().splitlines()[-6:]
    return {"command": "cargo test -p adcs-relations", "passed": r.returncode == 0, "tail": tail}


def deliver(root, gids=None, wave=None, *, test=True, out_of_order=None):
    """Deliver groups; returns ([(group, version, path)], problems)."""
    import group as GR
    import groupcode
    root = pathlib.Path(root)
    order = waves()
    wave_of = {g: k for k, _, gs in order for g in gs}
    rels = GR.latest_releases(root)
    if wave:
        if wave not in G.WAVES:
            return [], [f"no wave {wave}: the waves are {', '.join(G.WAVES)}"]
        gids = [g for g in next(gs for k, _, gs in order if k == wave) if g in rels]
        if not gids:
            return [], [f"wave {wave}: no group of it has a release yet"]
    problems = []
    for g in gids or []:
        if g not in wave_of:
            problems.append(f"no group {g}")
        elif g not in rels:
            problems.append(f"{g}: no release yet (its lead seals one in the group app)")
    if problems:
        return [], problems
    ver = GR.verify(root)
    for g in gids:
        problems += ver.get(g, [])
        earlier = [x for k, _, gs in order if k < wave_of[g] for x in gs if x in rels]
        late = [x for x in earlier if not delivery_path(root, x, _release_row(rels[x])["version"]).is_file()]
        if late and not out_of_order:
            problems.append(f"{g} (wave {wave_of[g]}): deliver wave {', '.join(sorted({wave_of[x] for x in late}))} first: {', '.join(late)} "
                            "not delivered (or --out-of-order REASON)")
    if problems:
        return [], problems
    summary, mp = GR.merge(root)
    if mp:
        return [], mp
    for g in gids:
        d = _code_differences(root, g)
        if d:
            problems.append(f"{g}: the generated code is not this release's ({'; '.join(d[:4])}{'; …' if len(d) > 4 else ''}): "
                            f"python3 tools/groupcode.py wire --design {root} && python3 tools/engine_build.py gen && "
                            "(cd engine && cargo test --release -p adcs-relations), then commit")
    if problems:
        return [], problems
    tests = _tests() if test else {"command": "not run (--no-test)", "passed": None, "tail": []}
    with tempfile.TemporaryDirectory() as tmp:
        apps = {p.name.split(".")[0]: p for p in groupcode.deliver(tmp)}
        made = []
        for g in gids:
            r = _release_row(rels[g])
            w = json.loads((groupcode.SRC / f"{g}.wire.json").read_text(encoding="utf-8"))
            vectors = sum(len(x["vectors"]) for x in w["rows"])
            outside = sum(1 for x in w["rows"] for v in x["vectors"] if v["outside"])
            app_name = f"{g}-{r['version']}.test-app.html"
            ddir = root / DELIVERIES
            ddir.mkdir(exist_ok=True)
            shutil.copy2(apps[g], ddir / app_name)
            with sqlite3.connect(f"file:{root / 'design.tndb'}?mode=ro", uri=True) as c:
                readers = sorted({x for (rs,) in c.execute("SELECT readers FROM catalogue_output WHERE node IN (SELECT id FROM design_node WHERE group_id = ?)", (g,))
                                  for x in (rs or "").split(",") if x})
            rec = {"schema": SCHEMA, "group": g, "wave": wave_of[g], "version": r["version"], "release_file": rels[g].name,
                   "release_fingerprint": r["fingerprint"], "sealed_by": r["sealed_by"], "sealed_at": r["sealed_at"],
                   "nodes": r["nodes"], "confirmed": r["confirmed"], "delivered_at": utc_now(),
                   "code": {"rows_with_code": len(w["rows"]), "rows_without_code": len(w["without_code"]), "functions": sorted(x["fn"] for x in w["rows"]),
                            "test_vectors": vectors, "from_outside_the_code": outside},
                   "tests": tests, "test_app": app_name, "test_app_sha256": _sha((ddir / app_name).read_bytes()),
                   "read_by": readers, "out_of_order": out_of_order}
            text = json.dumps(rec, indent=1, sort_keys=True) + "\n"
            p = delivery_path(root, g, r["version"])
            write_text(p, text)
            write_text(p.with_suffix("").with_suffix(".md"), note(rec, _sha(text)))
            made.append((g, r["version"], p))
    return made, []


def note(rec, fp):
    t = rec["tests"]
    ok = "passed" if t["passed"] else "not run" if t["passed"] is None else "FAILED"
    c = rec["code"]
    return "\n".join([
        f"# Delivery: {rec['group']} {rec['version']} (wave {rec['wave']})", "",
        f"**In one line:** the release {rec['group']} {rec['version']}, sealed by {rec['sealed_by']} on {rec['sealed_at']}, "
        f"tested and delivered; its lead accepts it in the group app (Release → Deliveries → Accept).", "",
        "| | |", "|---|---|",
        f"| Release | `{rec['release_file']}`, fingerprint `{rec['release_fingerprint'][:16]}…` |",
        f"| Nodes | {rec['nodes']}, {rec['confirmed']} sealed as confirmed, {rec['nodes'] - rec['confirmed']} UNCONFIRMED |",
        f"| Code | {c['rows_with_code']} computing row(s) generated (Rust, the twin, WebAssembly); {c['rows_without_code']} without code yet |",
        f"| Test vectors | {c['test_vectors']}, {c['from_outside_the_code']} with an answer from outside the code |",
        f"| Tests | `{t['command']}`: {ok} |",
        f"| Test app | `{rec['test_app']}`: open it from disk; every vector runs in the interpreter and in WebAssembly |",
        f"| Read by | {', '.join(rec['read_by']) or 'no other group'}: their leads see this delivery in the catalogue |",
        f"| Delivered | {rec['delivered_at']}" + (f"; out of wave order: {rec['out_of_order']}" if rec["out_of_order"] else "") + " |", "",
        "**What accepting says:** the lead has opened the test app, the group's vectors pass in it, and the release is what the group meant to ship. "
        "An acceptance names this release's fingerprint and this delivery's; a later release needs its own delivery and acceptance.", "",
        f"Delivery fingerprint (SHA-256 of the JSON beside this note): `{fp}`", ""])


def acceptance(root, gid, rec):
    """The lead's acceptance of a delivery, or None, and why not."""
    lead, leads, sigs = _group_people(root, gid)
    text = delivery_path(root, gid, rec["version"]).read_text(encoding="utf-8")
    fp = _sha(text)
    for s in reversed(sigs):
        if s["role"] != "accepted":
            continue
        try:
            st = json.loads(s["statement"] or "{}")
        except ValueError:
            continue
        if st.get("version") != rec["version"]:
            continue
        if s["name"] not in leads:
            return None, f"accepted by {s['name']}, who is not {gid}'s lead"
        if st.get("fingerprint") != rec["release_fingerprint"] or st.get("delivery") != fp:
            return None, f"the acceptance by {s['name']} names another release or delivery"
        return {"by": s["name"], "at": s["at"]}, None
    return None, f"not accepted yet by its lead ({lead or 'none named: People → a member with the role lead'})"


def status(root):
    """[{group, wave, state, version, ...}] wave by wave."""
    import group as GR
    root = pathlib.Path(root)
    rels = GR.latest_releases(root)
    out = []
    for k, lab, gs in waves():
        for g in gs:
            row = {"group": g, "wave": k, "wave_label": lab, "version": None, "state": "not released", "why": "no release yet (its lead seals one in the group app)",
                   "accepted_by": None}
            if g in rels:
                r = _release_row(rels[g])
                row.update(version=r["version"], nodes=r["nodes"], confirmed=r["confirmed"], state="released",
                           why=f"{g} {r['version']} sealed; not delivered yet (python3 tools/delivery.py deliver)")
                p = delivery_path(root, g, r["version"])
                if p.is_file():
                    rec = json.loads(p.read_text(encoding="utf-8"))
                    if rec.get("release_fingerprint") != r["fingerprint"]:
                        row.update(state="released", why=f"the delivery of {r['version']} is of another release: deliver it again")
                    else:
                        acc, why = acceptance(root, g, rec)
                        if acc:
                            row.update(state="accepted", why="", accepted_by=acc["by"], accepted_at=acc["at"])
                        else:
                            row.update(state="delivered", why=why, tests_passed=rec["tests"]["passed"])
            out.append(row)
    return out


def ship(root, out=None, require_accepted=False):
    rows = status(root)
    rec = {"schema": "trinetra-shipping/1", "at": utc_now(), "design": str(root), "groups": []}
    for r in rows:
        rec["groups"].append({"group": r["group"], "wave": r["wave"], "version": r["version"],
                              "ships_as": "accepted" if r["state"] == "accepted" else "UNCONFIRMED",
                              "accepted_by": r.get("accepted_by"), "why": r["why"] or None})
    acc = [g for g in rec["groups"] if g["ships_as"] == "accepted"]
    lines = ["# What ships", "", f"**In one line:** {len(acc)} of {len(rows)} group(s) accepted by their leads; "
             f"{len(rows) - len(acc)} ship visibly UNCONFIRMED, each with why (docs/RELEASE_PLAN.md §9).", "",
             "| Wave | Group | Version | Ships as | Accepted by / why not |", "|---|---|---|---|---|"]
    for g in rec["groups"]:
        lines.append(f"| {g['wave']} | `{g['group']}` | {g['version'] or '—'} | {'accepted' if g['ships_as'] == 'accepted' else '**UNCONFIRMED**'} | "
                     f"{g['accepted_by'] or g['why']} |")
    if out:
        out = pathlib.Path(out)
        write_text(out, json.dumps(rec, indent=1) + "\n")
        write_text(out.with_suffix(".md"), "\n".join(lines) + "\n")
    bad = require_accepted and len(acc) < len(rows)
    return rec, "\n".join(lines), bad


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    d = sub.add_parser("deliver", help="deliver groups' latest releases")
    d.add_argument("dir")
    d.add_argument("groups", nargs="*")
    d.add_argument("--wave", choices=list(G.WAVES))
    d.add_argument("--no-test", action="store_true", help="record the tests as not run")
    d.add_argument("--out-of-order", metavar="REASON", help="deliver before an earlier wave, for the reason given (recorded)")
    s = sub.add_parser("status", help="released, delivered, accepted: wave by wave")
    s.add_argument("dir")
    s.add_argument("--json", action="store_true")
    sh = sub.add_parser("ship", help="the shipping record")
    sh.add_argument("dir")
    sh.add_argument("--out")
    sh.add_argument("--require-accepted", action="store_true")
    a = ap.parse_args(argv)
    if a.cmd == "deliver":
        if not a.groups and not a.wave:
            print("delivery: name the groups, or --wave A..E", file=sys.stderr)
            return 2
        made, problems = deliver(a.dir, a.groups or None, a.wave, test=not a.no_test, out_of_order=a.out_of_order)
        for p in problems:
            print("delivery: " + p, file=sys.stderr)
        for g, v, p in made:
            print(f"delivery: {g} {v} -> {p}")
        return 1 if problems else 0
    if a.cmd == "status":
        rows = status(a.dir)
        if a.json:
            print(json.dumps(rows, indent=1))
            return 0
        for r in rows:
            print(f"{r['wave']}  {r['group']:10} {r['version'] or '—':6} {r['state']:12} {r.get('accepted_by') or r['why']}")
        n = sum(r["state"] == "accepted" for r in rows)
        print(f"delivery: {n} of {len(rows)} group(s) accepted")
        return 0
    rec, md, bad = ship(a.dir, a.out, a.require_accepted)
    print(md)
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
