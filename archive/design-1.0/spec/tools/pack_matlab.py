#!/usr/bin/env python3
"""Builds the MATLAB SILS twin's download: dist/adcs_sils_matlab_<version>.zip
(SPEC.md §10.8.2).

    python3 tools/pack_matlab.py                     # writes dist/adcs_sils_matlab_<version>.zip
    python3 tools/pack_matlab.py --out <dir>         # somewhere else
    python3 tools/pack_matlab.py --version <text>    # instead of the git commit
    python3 tools/pack_matlab.py --rng <file.json>   # include the platform's reference draws
    python3 tools/pack_matlab.py --engine-hash <h>   # the platform's engine hash, recorded in VERSION
    python3 tools/pack_matlab.py --phase P3          # refuse unless every element built by P3 (and what it needs) has its twin
    python3 tools/pack_matlab.py --phase P6,P3M      # when two branches of SPEC.md §19 are green

What goes in:
* matlab_sils/ (the twin's code, examples and tests), when it exists;
* manual/user/*.md: the user manual, as manual/;
* forms/: the node library (every node's document and request form, the
  new-node request) and the case editor; viewer/: the result template;
  store/: the empty local store, with its README;
* plan/case_template.csv and plan/cases/*.csv, unchanged: the twin reads the
  case CSV directly;
* data/*.json: every TOML the twin needs, exported, because MATLAB has no TOML
  reader. A nan in TOML becomes JSON null ("not measured"): jsondecode turns a
  null inside a numeric array into NaN and a scalar null into [], which the
  twin's loader reads as NaN;
* data/param_ids.json: the parameter-id table (SPEC.md §7.5), from each
  algorithm's `number` and each parameter's `number`, which are assigned once
  and never renumbered;
* nothing that catalogue/restricted.toml names, once D1 has written it: whole
  files under `exclude`, and single keys under `[strip]` ("<file>" = ["table.key"]);
* no labtwin campaign: the lab twin belongs to the rig, not to SILS;
* data/twin_map.json: the twin map (plan/twin_map.toml, expanded), each element
  marked with whether its twin is in this zip; TWIN.md says the same in words;
* VERSION, MANIFEST.sha256 and startup_asils.m.

The zip is deterministic: sorted entries, a fixed timestamp, fixed
permissions, one compression level. The same commit gives the same bytes.
It refuses to pack when the twin map fails its check; when a twin folder
exists (matlab_sils/+asils/+fsw/, +physics/+orbit/, …) and an element the map
puts in it has no file there; and, with --phase, when any element built by that
phase has no twin (SPEC.md §10.8.7). So the zip, built on every push from P1, is
never behind the platform.
"""

import glob
import hashlib
import io
import json
import math
import os
import subprocess
import sys
import zipfile

try:
    import tomllib
except ImportError:  # pragma: no cover
    sys.exit("pack_matlab.py needs Python 3.11+ (tomllib)")

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
STAMP = (1980, 1, 1, 0, 0, 0)


def nulls(x):
    if isinstance(x, float) and math.isnan(x):
        return None
    if isinstance(x, dict):
        return {k: nulls(v) for k, v in x.items()}
    if isinstance(x, list):
        return [nulls(v) for v in x]
    return x


def jtext(obj):
    return json.dumps(nulls(obj), ensure_ascii=False, indent=1, sort_keys=True, allow_nan=False) + "\n"


def toml(path):
    with open(path, "rb") as f:
        return tomllib.load(f)


def tree_hash(paths):
    h = hashlib.sha256()
    for p in sorted(paths):
        h.update(os.path.relpath(p, ROOT).encode())
        h.update(open(p, "rb").read())
    return h.hexdigest()[:16]


def git_commit():
    try:
        return subprocess.run(["git", "-C", ROOT, "rev-parse", "--short=12", "HEAD"], capture_output=True,
                              text=True, check=True).stdout.strip()
    except (OSError, subprocess.CalledProcessError):
        return "package"


def restricted():
    p = os.path.join(ROOT, "catalogue", "restricted.toml")
    if not os.path.exists(p):
        return set(), {}
    d = toml(p)
    return set(d.get("exclude", [])), d.get("strip", {})


def strip(obj, keys):
    for k in keys:
        cur, parts = obj, k.split(".")
        for part in parts[:-1]:
            cur = cur.get(part, {}) if isinstance(cur, dict) else {}
        if isinstance(cur, dict):
            cur.pop(parts[-1], None)
    return obj


def collect(rng_file, phase=None):
    files = {}   # path inside the zip -> bytes
    excluded, stripped = restricted()

    def exported(path):
        rel = os.path.relpath(path, ROOT)
        return strip(toml(path), stripped.get(rel, []))
    add = lambda name, data: files.__setitem__(name, data.encode() if isinstance(data, str) else data)

    twin = os.path.join(ROOT, "matlab_sils")
    for p in sorted(glob.glob(os.path.join(twin, "**", "*"), recursive=True)):
        if os.path.isfile(p):
            add(os.path.relpath(p, twin), open(p, "rb").read())

    add("cases/case_template.csv", open(os.path.join(ROOT, "plan", "case_template.csv"), "rb").read())

    # What a team member uses beside the engine: the case editor, the node
    # library (every node's document and request form), the new-node request,
    # and the result-document template the twin saves its results into.
    import tempfile
    sys.path.insert(0, os.path.join(ROOT, "tools"))
    import forms
    import plan_model
    with tempfile.TemporaryDirectory() as tmp:
        forms.export_library(plan_model.Plan(), tmp)
        for p in sorted(glob.glob(os.path.join(tmp, "**", "*.html"), recursive=True)):
            add("forms/" + os.path.relpath(p, tmp).replace(os.sep, "/"), open(p, "rb").read())
    add("viewer/result_template.html", open(os.path.join(ROOT, "results", "template.html"), "rb").read())
    add("store/README.md", "# Your local store\n\n"
        "Every finished run or campaign is saved here by `asils.result.save` as one result document, filed beside the case it "
        "was run on:\n\n    store/cases/<case id>/<sha12>.csv\n    store/results/<case id>/<sha12>/<scenario>_<date>.result.html\n"
        "    store/index.csv\n\nOpen a result in a browser, or with `asils.result.open`, to see everything again without "
        "re-running. `asils.result.import(file)` files a result someone sent you the same way, with its case. index.csv is "
        "rebuilt from the files by `asils.result.index`; delete it any time. The same layout is used by the web app and the "
        "workbench (~/.adcs/store/), so a store can be copied between them.\n")
    for p in sorted(glob.glob(os.path.join(ROOT, "manual", "user", "*.md"))):
        add("manual/" + os.path.basename(p), open(p, "rb").read())
    for p in sorted(glob.glob(os.path.join(ROOT, "plan", "cases", "*.csv"))):
        raw = open(p, "rb").read()
        add("cases/" + os.path.basename(p), raw)
        # the store starts with the reference cases, filed as the store files every case (SPEC.md §13.5.4)
        add("store/cases/%s/%s.csv" % (os.path.basename(p)[:-4], hashlib.sha256(raw).hexdigest()[:12]), raw)
    for p in sorted(glob.glob(os.path.join(ROOT, "catalogue", "classes", "*.csv"))):
        add("data/classes/" + os.path.basename(p), open(p, "rb").read())

    add("data/case_inputs.json", jtext(toml(os.path.join(ROOT, "plan", "case_inputs.toml"))))
    add("data/kpis.json", jtext(toml(os.path.join(ROOT, "plan", "kpis.toml"))))
    for name in ("families", "classes", "schema"):
        add("data/%s.json" % name, jtext(toml(os.path.join(ROOT, "catalogue", name + ".toml"))))
    for kind in ("parts", "products", "algorithms"):
        for p in sorted(glob.glob(os.path.join(ROOT, "catalogue", kind, "*.toml"))):
            if os.path.relpath(p, ROOT) not in excluded:
                add("data/%s/%s.json" % (kind, os.path.basename(p)[:-5]), jtext(exported(p)))
    for kind in ("scenarios", "campaigns"):
        for p in sorted(glob.glob(os.path.join(ROOT, kind, "*.toml"))):
            d = exported(p)
            if kind == "campaigns" and d.get("type") == "labtwin":
                continue
            add("data/%s/%s.json" % (kind, os.path.basename(p)[:-5]), jtext(d))

    seed = toml(os.path.join(ROOT, "plan", "seed_content.toml"))
    fixtures = [{"tree_id": r["tree_id"], "physics": r.get("physics"), "call": r.get("call"),
                 "inputs": r.get("inputs", []), "fixture": r["fixture"]}
                for r in seed.get("row", []) if r.get("fixture")]
    add("data/fixtures.json", jtext(fixtures))

    table = []
    for p in sorted(glob.glob(os.path.join(ROOT, "catalogue", "algorithms", "*.toml"))):
        d = toml(p)
        table.append({"algorithm": d["id"], "id": d["number"],
                      "params": [{"name": pr["name"], "id": pr["number"]} for pr in d.get("param", [])]})
    table.sort(key=lambda t: t["id"])
    add("data/param_ids.json", jtext(table))

    fsw = os.path.join(twin, "+asils", "+fsw")
    if os.path.isdir(fsw):
        missing = [t["algorithm"] for t in table if not os.path.exists(os.path.join(fsw, t["algorithm"] + ".m"))]
        if missing:
            raise SystemExit("refused: no twin implementation for algorithm(s) %s in matlab_sils/+asils/+fsw/" % missing)

    # the lockstep (SPEC.md §10.8.7): the map, and which of its twins this zip carries
    import twin_check
    tf, els = twin_check.check(ROOT)
    if tf:
        raise SystemExit("refused: the twin map fails its check: %s" % "; ".join("%s %s %s" % f for f in tf[:5]))
    twin_rows, gaps = [], []
    for e in els:
        rels = [t[len("matlab_sils/"):] for t in twin_check.twin_files(e) if t.startswith("matlab_sils/")]
        rel = rels[0] if rels else ""
        here = bool(rels) and all(r in files for r in rels)
        for r in rels:
            if r not in files and os.path.isdir(os.path.join(twin, os.path.dirname(r))) and os.path.dirname(r) != "+asils":
                gaps.append(e["id"])
        if rels and not here and phase and e.get("phase") in twin_check.covered(phase):
            gaps.append(e["id"])
        twin_rows.append({"id": e["id"], "phase": e.get("phase"), "rungs": e.get("rungs", []), "platform": e.get("platform", ""),
                          "twin": e.get("twin", ""), "twin_file": rel, "only": e.get("only", ""), "test": e.get("test", ""),
                          "in_zip": here if rel else None})
    if gaps:
        raise SystemExit("refused: no twin in matlab_sils/ for %s (SPEC.md §10.8.7)" % sorted(set(gaps)))
    add("data/twin_map.json", jtext({"schema": "adcs-twin-map/1", "phase": phase or "not given", "elements": twin_rows}))
    need = [r for r in twin_rows if r["only"] != "platform"]
    have = [r for r in need if r["in_zip"]]
    lines = ["# What this zip carries of the SILS twin", "",
             "**In one line:** %d of the %d SILS elements that have a MATLAB side are in this zip; the rest arrive with the "
             "phase named beside them, in the same change as their platform side (SPEC.md §10.8.7)." % (len(have), len(need)), "",
             "| Element | Phase | MATLAB | In this zip | The test both engines must pass |", "|---|---|---|---|---|"]
    for r in need:
        lines.append("| %s | %s | `%s` | %s | %s |" % (r["id"], r["phase"], r["twin"], "yes" if r["in_zip"] else "not yet",
                                                      r["test"]))
    one = [r for r in twin_rows if r["only"] == "platform"]
    if one:
        lines += ["", "In the platform only, by design:", ""] + ["* %s: %s" % (r["id"], next(
            (e.get("why", "") for e in els if e["id"] == r["id"]), "")) for r in one]
    add("TWIN.md", "\n".join(lines) + "\n")

    if rng_file:
        add("data/rng_reference.json", open(rng_file, "rb").read())

    cat_files = [p for p in glob.glob(os.path.join(ROOT, "catalogue", "**", "*"), recursive=True) if os.path.isfile(p)]
    reg = toml(os.path.join(ROOT, "plan", "case_inputs.toml"))
    return files, {"commit": git_commit(), "catalogue": "unpublished " + tree_hash(cat_files),
                   "case_format": reg.get("schema"), "algorithms": len(table), "twin": "%d of %d" % (len(have), len(need))}


def main():
    args = sys.argv[1:]
    opts = {"--out": os.path.join(ROOT, "dist"), "--version": None, "--rng": None, "--engine-hash": "not given", "--phase": None}
    while args:
        a = args.pop(0)
        if a not in opts or not args:
            print(__doc__.split("\n\n")[1])
            return 2
        opts[a] = args.pop(0)
    if opts["--phase"]:
        sys.path.insert(0, os.path.join(ROOT, "tools"))
        import twin_check
        if not twin_check.phases_ok(opts["--phase"]):
            print("--phase is one or more of %s, separated by commas" % ", ".join(twin_check.PHASES))
            return 2
    files, ver = collect(opts["--rng"], opts["--phase"])
    version = opts["--version"] or ver["commit"]
    top = "adcs_sils_matlab_%s/" % version
    enc = lambda v: v.encode() if isinstance(v, str) else v
    files.setdefault("startup_asils.m", "% Adds the ADCS SILS twin to the MATLAB path. Generated by tools/pack_matlab.py.\n"
                     "addpath(fileparts(mfilename('fullpath')));\n")
    files.setdefault("README.md", "# ADCS SILS twin for MATLAB\n\nSee SPEC.md §10.8. Run `startup_asils`, then the examples/.\n")
    files = {k: enc(v) for k, v in files.items()}
    files["VERSION"] = ("version = %s\ncommit = %s\ncatalogue = %s\ncase_format = %s\nengine = %s\nalgorithms = %d\n"
                        "twin_elements = %s\n"
                        % (version, ver["commit"], ver["catalogue"], ver["case_format"], opts["--engine-hash"],
                           ver["algorithms"], ver["twin"])).encode()
    manifest = "".join("%s  %s\n" % (hashlib.sha256(files[k]).hexdigest(), k) for k in sorted(files))
    files["MANIFEST.sha256"] = manifest.encode()
    os.makedirs(opts["--out"], exist_ok=True)
    out = os.path.join(opts["--out"], "adcs_sils_matlab_%s.zip" % version)
    buf = io.BytesIO()
    with zipfile.ZipFile(buf, "w") as z:
        for name in sorted(files):
            info = zipfile.ZipInfo(top + name, date_time=STAMP)
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o644 << 16
            z.writestr(info, files[name], compresslevel=9)
    open(out, "wb").write(buf.getvalue())
    print("wrote %s: %d files, sha256 %s" % (os.path.relpath(out, ROOT), len(files),
                                             hashlib.sha256(buf.getvalue()).hexdigest()[:16]))
    return 0


if __name__ == "__main__":
    sys.exit(main())
