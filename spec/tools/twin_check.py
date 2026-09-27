#!/usr/bin/env python3
"""The MATLAB twin's lockstep with the platform (SPEC.md §10.8.7): the package's
stand-in for `cargo xtask twin check`.

    python3 tools/twin_check.py                         the map is whole: every SILS element, both sides named
    python3 tools/twin_check.py --list                  the expanded map, one line per element
    python3 tools/twin_check.py --json <file>           the expanded map as JSON (the MATLAB zip carries it)
    python3 tools/twin_check.py --repo <dir> --phase P3 in a checkout: every element built by that phase
                                                        (and the phases it needs) exists on both sides;
                                                        --phase P6,P3M when two branches of §19 are green
    python3 tools/twin_check.py --changed <file> [--label twin:none --reason <text>]
                                                        a pull request's changed paths (one per line): a side
                                                        changed without the other fails
    python3 tools/twin_check.py --selftest              each deliberate break is caught by its rule

plan/twin_map.toml lists every element of SILS: its platform side (Rust, or C
for flight software), its MATLAB twin side, the phase that builds both, the
rungs that reuse it, and the test both must pass. Families expand over the
registries (physics functions, algorithms, metric kinds, part kinds), so an item
added to a registry is in the map at once, and the check asks for both sides.
"""

import copy
import glob
import json
import os
import re
import shutil
import sys
import tempfile

try:
    import tomllib
except ImportError:  # pragma: no cover
    raise SystemExit("the package tools need Python 3.11+ (tomllib)")

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PHASES = ["P0", "P1", "P2", "P3", "P4", "P3M", "P5", "P6", "P7", "P8", "P9", "P10"]
# SPEC.md §19's dependency table: a phase needs these. P3M runs beside P5-P7, so
# reaching P5 does not mean P3M's elements exist.
NEEDS = {"P0": [], "P1": ["P0"], "P2": ["P1"], "P3": ["P2"], "P4": ["P3"], "P3M": ["P4"], "P5": ["P4"],
         "P6": ["P4"], "P7": ["P6"], "P8": ["P7"], "P9": ["P5", "P8"], "P10": ["P9"]}


def covered(reached):
    """The phases whose elements must exist once `reached` (one phase, or several
    separated by commas, such as "P6,P3M") is green."""
    out, todo = set(), [p.strip() for p in str(reached).split(",") if p.strip()]
    while todo:
        p = todo.pop()
        if p not in out:
            out.add(p)
            todo += NEEDS.get(p, [])
    return out


def phases_ok(reached):
    return all(p.strip() in NEEDS for p in str(reached).split(",") if p.strip()) and str(reached).strip() != ""
RUNGS = ["sils", "pil", "oils", "hils"]
RULES = {
    "TW01": "the map is adcs-twin-map/1; every element has a unique id, a phase that exists and rungs from sils, pil, oils, hils",
    "TW02": "an element is on both sides, platform and twin, or is marked only = platform or only = twin, with the reason why",
    "TW03": "a twin side is an asils. name in matlab_sils/+asils/; a platform side is a file under crates/ or fsw/",
    "TW04": "every family's registry yields its items, so each physics function, algorithm, metric kind and part kind is in the map",
    "TW05": "in a checkout, every element built by the phases reached (and the phases they need) exists on both sides",
    "TW06": "a change to one side of an element changes the other in the same pull request, or carries twin:none with a reason; a module file holding several functions changes with at least one of their twins",
}


def load_toml(p):
    with open(p, "rb") as f:
        return tomllib.load(f)


def registry(root, fam):
    src = fam.get("source", "")
    kind = fam.get("kind")
    items = []
    if kind == "physics":
        for f in load_toml(os.path.join(root, src)).get("function", []):
            items.append({"module": f["module"], "name": f["name"]})
    elif kind == "fsw":
        for p in sorted(glob.glob(os.path.join(root, src))):
            d = load_toml(p)
            items.append({"id": d.get("id") or os.path.basename(p)[:-5], "_prototype": bool(d.get("prototype"))})
    elif kind == "metric":
        kinds = set()
        for pat in (src if isinstance(src, list) else [src]):
            for p in sorted(glob.glob(os.path.join(root, pat))):
                d = load_toml(p)
                kinds |= {m["kind"] for m in d.get("metric", []) if m.get("kind")}
                kinds |= {k["metric"] for k in d.get("kpi", []) if k.get("metric")}
        items = [{"kind": k} for k in sorted(kinds - set(fam.get("exclude", [])))]
    elif kind == "device":
        kinds = set()
        for p in sorted(glob.glob(os.path.join(root, src))):
            k = load_toml(p).get("kind")
            if k:
                kinds.add(k)
        items = [{"kind": k} for k in sorted(kinds)]
    return items


def expand(root, tw):
    out = []
    for fam in tw.get("family", []):
        items = registry(root, fam)
        for it in items:
            e = {k: (v.format(**it) if isinstance(v, str) else v) for k, v in fam.items()
                 if k not in ("source", "exclude", "exclude_why")}
            e["family"] = fam.get("kind")
            if it.get("_prototype"):
                # a prototype algorithm lives in the twin until its C lands with prototype = false (SPEC.md §10.8.5)
                e.update({"only": "twin", "why": "a prototype (prototype = true): the twin carries it until its C lands, "
                          "in the change that sets prototype = false"})
            out.append(e)
        if not items:
            out.append({"id": "family." + str(fam.get("kind")), "_empty": fam.get("source")})
        if fam.get("exclude") and not str(fam.get("exclude_why", "")).strip():
            out.append({"id": "family." + str(fam.get("kind")), "_nowhy": True})
    for el in tw.get("element", []):
        out.append(dict(el))
    return out


def twin_files(e):
    return [f for f in [e.get("twin_file")] + list(e.get("twin_also", [])) if f]


def check(root, tw=None):
    F = []
    add = lambda r, where, text: F.append((r, where, text))
    tw = tw if tw is not None else load_toml(os.path.join(root, "plan", "twin_map.toml"))
    if tw.get("schema") != "adcs-twin-map/1":
        add("TW01", "twin_map.toml", "schema is %r" % tw.get("schema"))
    els = expand(root, tw)
    seen = set()
    for e in els:
        eid = e.get("id", "?")
        if "_empty" in e:
            add("TW04", eid, "the family's source %s yields nothing" % e["_empty"])
            continue
        if "_nowhy" in e:
            add("TW04", eid, "the family leaves items out without saying why (exclude_why)")
            continue
        if eid in seen:
            add("TW01", eid, "id used twice")
        seen.add(eid)
        if e.get("phase") not in PHASES:
            add("TW01", eid, "phase %r is not a phase of SPEC.md §19" % e.get("phase"))
        if not e.get("rungs") or any(r not in RUNGS for r in e.get("rungs", [])):
            add("TW01", eid, "rungs %r" % e.get("rungs"))
        only = e.get("only", "")
        if only and only not in ("platform", "twin"):
            add("TW02", eid, "only = %r" % only)
        if only and not str(e.get("why", "")).strip():
            add("TW02", eid, "marked only = %s without saying why" % only)
        if only != "twin" and not (e.get("platform") and e.get("platform_file")):
            add("TW02", eid, "no platform side")
        if only != "platform" and not (e.get("twin") and e.get("twin_file")):
            add("TW02", eid, "no twin side")
        if e.get("twin") and not str(e["twin"]).startswith("asils."):
            add("TW03", eid, "the twin side %r is not an asils. name" % e["twin"])
        for tf in twin_files(e):
            if not str(tf).startswith("matlab_sils/+asils/"):
                add("TW03", eid, "the twin file %r is not in matlab_sils/+asils/" % tf)
        if e.get("platform_file") and not re.match(r"^(crates|fsw)/", str(e["platform_file"])):
            add("TW03", eid, "the platform file %r is not under crates/ or fsw/" % e["platform_file"])
        if not str(e.get("test", "")).strip():
            add("TW01", eid, "no shared test named")
    return F, els


def repo_check(root, repo, phase, els):
    F = []
    upto = covered(phase)
    for e in els:
        if e.get("phase") not in upto or "_empty" in e or "_nowhy" in e:
            continue
        for side in ("platform", "twin"):
            if e.get("only") and e["only"] != side:
                continue
            for f in ([e.get("platform_file")] if side == "platform" else twin_files(e)):
                if f and not os.path.exists(os.path.join(repo, f)):
                    F.append(("TW05", e["id"], "built by %s, but the %s side %s does not exist" % (e["phase"], side, f)))
    return F


def diff_check(els, changed, label, reason=""):
    """A pull request's changed paths. Any change counts, whatever the phase: an
    element written early on one side is one-sided too."""
    ch = set(changed)
    if label == "twin:none":
        return [] if str(reason).strip() else [("TW06", "pull request", "labelled twin:none without a reason")]
    F = []
    groups = {}
    for e in els:
        if e.get("only") or "_empty" in e or "_nowhy" in e:
            continue
        groups.setdefault(e.get("platform_file"), []).append(e)
    for p, es in groups.items():
        twins = [t for e in es for t in twin_files(e)]
        if p in ch and not any(t in ch for t in twins):
            if len(es) == 1:
                F.append(("TW06", es[0]["id"], "%s changed, its twin %s did not" % (p, ", ".join(twin_files(es[0])))))
            else:
                F.append(("TW06", ",".join(e["id"] for e in es), "%s changed (it holds %d elements), none of their twins did"
                          % (p, len(es))))
        for e in es:
            if any(t in ch for t in twin_files(e)) and p not in ch:
                F.append(("TW06", e["id"], "%s changed, its platform side %s did not"
                          % (", ".join(t for t in twin_files(e) if t in ch), p)))
    return F


def selftest():
    F, els = check(ROOT)
    if F:
        print("the map itself fails:", F)
        return 1
    tw0 = load_toml(os.path.join(ROOT, "plan", "twin_map.toml"))
    bad = 0

    def mut(fn):
        tw = copy.deepcopy(tw0)
        fn(tw)
        return {r for r, _, _ in check(ROOT, tw)[0]}
    E = lambda tw, i: tw["element"][i]
    cases = [
        ("TW01", "a phase that does not exist", lambda tw: E(tw, 0).update({"phase": "P99"})),
        ("TW01", "two elements with one id", lambda tw: E(tw, 1).update({"id": E(tw, 0)["id"]})),
        ("TW01", "a rung that does not exist", lambda tw: E(tw, 0).update({"rungs": ["orbit"]})),
        ("TW02", "an element with no twin side", lambda tw: E(tw, 0).update({"twin": "", "twin_file": ""})),
        ("TW02", "a one-sided element with no reason", lambda tw: next(e for e in tw["element"] if e.get("only")).update({"why": ""})),
        ("TW03", "a twin outside +asils", lambda tw: E(tw, 0).update({"twin_file": "matlab/read.m"})),
        ("TW04", "a family leaving items out without saying why", lambda tw: next(f for f in tw["family"] if f.get("exclude")).update({"exclude_why": ""})),
        ("TW04", "a family whose registry is empty", lambda tw: tw["family"][1].update({"source": "catalogue/nothing/*.toml"})),
    ]
    # a prototype algorithm is twin-only; the change that sets prototype = false must bring both sides
    fam_fsw = next(f for f in tw0["family"] if f["kind"] == "fsw")
    tmpc = tempfile.mkdtemp()
    try:
        os.makedirs(os.path.join(tmpc, "catalogue", "algorithms"))
        open(os.path.join(tmpc, "catalogue", "algorithms", "trial.toml"), "w").write('id = "trial"\nprototype = true\n')
        pe = expand(tmpc, {"family": [fam_fsw]})
        if not (pe and pe[0].get("only") == "twin"):
            print("a prototype algorithm is not marked twin-only: %s" % pe)
            bad += 1
    finally:
        shutil.rmtree(tmpc, ignore_errors=True)
    for code, what, fn in cases:
        if code not in mut(fn):
            print("NOT CAUGHT %s: %s" % (code, what))
            bad += 1
    # repo mode: a checkout at P1 with the physics module but not its twin
    tmp = tempfile.mkdtemp()
    try:
        phys = [e for e in els if e.get("family") == "physics"]
        for e in phys:
            os.makedirs(os.path.dirname(os.path.join(tmp, e["platform_file"])), exist_ok=True)
            open(os.path.join(tmp, e["platform_file"]), "a").close()
        for e in els:
            if e["id"] in ("case.read", "case.template"):
                for f in (e["platform_file"], e["twin_file"]):
                    os.makedirs(os.path.dirname(os.path.join(tmp, f)), exist_ok=True)
                    open(os.path.join(tmp, f), "a").close()
        if "TW05" not in {r for r, _, _ in repo_check(ROOT, tmp, "P1", els)}:
            print("NOT CAUGHT TW05: physics written in the platform without its twin")
            bad += 1
        for e in phys:
            os.makedirs(os.path.dirname(os.path.join(tmp, e["twin_file"])), exist_ok=True)
            open(os.path.join(tmp, e["twin_file"]), "a").close()
        if repo_check(ROOT, tmp, "P1", els):
            print("repo mode refuses a checkout with both sides of every P1 element: %s" % repo_check(ROOT, tmp, "P1", els)[:2])
            bad += 1
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    # phases: P3M runs beside P5-P7, so P5 does not cover it, and P3M covers P0-P4
    if "P3M" in covered("P5") or not {"P0", "P1", "P2", "P3", "P4"} <= covered("P3M") or "P3M" not in covered("P6,P3M"):
        print("the phase coverage does not follow SPEC.md §19's dependency table")
        bad += 1
    # diff mode
    plant = next(e for e in els if e["id"] == "plant.dynamics")
    res = next(e for e in els if e["id"] == "result.document")
    if diff_check(els, [res["platform_file"], res["twin_also"][0]], ""):
        print("a change to the result writer and one of its twin files is refused")
        bad += 1
    if "TW06" not in {r for r, _, _ in diff_check(els, [plant["platform_file"]], "twin:none", "")}:
        print("NOT CAUGHT TW06: twin:none with no reason")
        bad += 1
    orbit = [e for e in els if e.get("platform_file", "").endswith("physics/orbit.rs")]
    if len(orbit) > 1 and "TW06" not in {r for r, _, _ in diff_check(els, [orbit[0]["platform_file"]], "")}:
        print("NOT CAUGHT TW06: a physics module changed with none of its twins")
        bad += 1
    if "TW06" not in {r for r, _, _ in diff_check(els, [plant["platform_file"]], "")}:
        print("NOT CAUGHT TW06: the plant changed in Rust only")
        bad += 1
    if diff_check(els, [plant["platform_file"]], "twin:none", "a comment only"):
        print("twin:none is not honoured")
        bad += 1
    if diff_check(els, [plant["platform_file"], plant["twin_file"]], ""):
        print("a lockstep change is refused")
        bad += 1
    if bad:
        return 1
    fam = {}
    for e in els:
        fam[e.get("family", "element")] = fam.get(e.get("family", "element"), 0) + 1
    print("selftest ok: %d breaks in the map, one checkout without a twin, the phase coverage of §19, and one-sided changes (a module, a result file, twin:none without a reason) each caught; "
          "the map holds %d elements (%s)" % (len(cases), len(els), ", ".join("%s %d" % kv for kv in sorted(fam.items()))))
    return 0


def main():
    a = sys.argv[1:]

    def opt(name):
        return a[a.index(name) + 1] if name in a and a.index(name) + 1 < len(a) else None
    if a == ["--selftest"]:
        return selftest()
    if a and a[0] in ("-h", "--help"):
        print(__doc__)
        return 2
    F, els = check(ROOT)
    if "--list" in a:
        for e in els:
            print("%-28s %-5s %-24s %-44s %s" % (e["id"], e.get("phase", ""), ",".join(e.get("rungs", [])),
                                                e.get("platform", "") or "(twin only)", e.get("twin", "") or "(platform only)"))
    if opt("--json"):
        with open(opt("--json"), "w", encoding="utf-8") as f:
            json.dump({"schema": "adcs-twin-map/1", "elements": [{k: v for k, v in e.items() if not k.startswith("_")} for e in els]},
                      f, ensure_ascii=False, indent=1, sort_keys=True)
    if opt("--repo"):
        ph = opt("--phase")
        if not ph or not phases_ok(ph):
            print("--repo needs --phase: one or more of %s, separated by commas" % ", ".join(PHASES))
            return 2
        F += repo_check(ROOT, opt("--repo"), ph, els)
    if opt("--changed"):
        changed = [l.strip() for l in open(opt("--changed"), encoding="utf-8") if l.strip()]
        F += diff_check(els, changed, opt("--label") or "", opt("--reason") or "")
    for r, where, text in F:
        print("%s %s: %s" % (r, where, text))
    n1 = sum(1 for e in els if not e.get("only"))
    print("twin check: %s — %d elements, %d on both sides, %d one-sided by design" % (
        "ok" if not F else "%d finding(s)" % len(F), len(els), n1, len(els) - n1))
    return 1 if F else 0


if __name__ == "__main__":
    sys.exit(main())
