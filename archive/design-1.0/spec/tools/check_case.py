#!/usr/bin/env python3
"""The case checker (SPEC.md §8.3): what `adcs case check <file.csv>` prints in the
built repository, and what the case editor shows under "Is it ready?".

    python3 tools/check_case.py <case.csv>... [--json]

For each file it reports, in this order:

1. the format: the fixed columns, every key once and in order, units as the format
   states them, numbers that parse, ranges and levels only where the row takes
   them. Any failure here and the software refuses the file, by line and key;
2. what the case says: requirements written, inputs stated, inputs left unstated
   (each blocks what needs it, by name), blanks that take the reference value
   (listed as assumed), requirements taking the default level, values marked
   UNCONFIRMED;
3. what can run: for every scenario in this release, whether the case states
   every key it reads, and which of its requirements the scenario can judge.

Exit status 1 when any file breaks the format; 0 otherwise. A case with blanks
is not an error: nothing is guessed, and the report says what they block.
"""

import csv
import glob
import io
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import plan_model as pm  # noqa: E402
import validate_plan as vp  # noqa: E402
import forms  # noqa: E402


def load_rules(plan):
    vp.check_catalogue()
    vp.check_classes()
    vp.CASE.clear()
    vp.CASE.update(plan.case)


def examine(path, plan):
    text = open(path, encoding="utf-8").read()
    errs, meta = vp.check_case_csv(os.path.basename(path), text)
    rows = {r[1]: r for r in list(csv.reader(io.StringIO(text)))[1:] if len(r) >= 9}
    val = lambda k: rows.get(k, [""] * 9)[4]
    reg = plan.case["input"]
    inputs = [i for i in reg if i["section"] != "req"]
    reqs = [i for i in reg if i["section"] == "req"]
    out = {
        "file": path, "format_errors": errs, "case_id": meta.get("case_id", ""), "title": meta.get("title", ""),
        "requirements_written": [i["key"] for i in reqs if val(i["key"])],
        "requirements_blank": [i["key"] for i in reqs if not val(i["key"])],
        "inputs_stated": [i["key"] for i in inputs if val(i["key"])],
        "unstated": [i["key"] for i in inputs if not val(i["key"]) and i["blank"] == "stated"],
        "assumed": [i["key"] for i in inputs if not val(i["key"]) and i["blank"] == "default"],
        "level_assumed": [i["key"] for i in reqs if val(i["key"]) and not rows.get(i["key"], [""] * 9)[7]],
        "unconfirmed": [k for k, r in rows.items() if r[4] and r[8].startswith("UNCONFIRMED")],
        "scenarios": [],
    }
    for p in sorted(glob.glob(os.path.join(plan.root, "scenarios", "*.toml"))):
        s = pm.toml(p)
        need = forms.scenario_reads(s)
        missing = [k for k in need if k in out["unstated"]]
        judges = []
        for m in s.get("metric", []):
            k = plan.case_key.get(m.get("requirement", ""))
            if k and val(k) and k not in judges:
                judges.append(k)
        out["scenarios"].append({"id": s.get("id"), "label": s.get("label", ""), "can_run": not missing,
                                 "blocked_by": missing, "judges": judges})
    return out


def text_report(r, default_level):
    L = ["%s — %s (%s)" % (r["file"], r["case_id"] or "no id", r["title"] or "no title")]
    if r["format_errors"]:
        L.append("  FORMAT: %d problem(s); the software refuses this file until they are fixed:" % len(r["format_errors"]))
        L += ["    - " + e for e in r["format_errors"]]
    else:
        L.append("  FORMAT: ok")
    L.append("  requirements written: %d of %d%s" % (len(r["requirements_written"]), len(r["requirements_written"]) + len(r["requirements_blank"]),
                                                    (" — " + ", ".join(r["requirements_written"])) if r["requirements_written"] else " — nothing can pass or fail"))
    L.append("  inputs stated: %d; unstated: %d%s" % (len(r["inputs_stated"]), len(r["unstated"]),
                                                     (" — each blocks what needs it: " + ", ".join(r["unstated"])) if r["unstated"] else ""))
    if r["assumed"]:
        L.append("  blank, will take the reference value (listed as assumed): " + ", ".join(r["assumed"]))
    if r["level_assumed"]:
        L.append("  requirements taking the default level %s %%: %s" % (default_level, ", ".join(r["level_assumed"])))
    if r["unconfirmed"]:
        L.append("  values marked UNCONFIRMED (results that use them say so): %d" % len(r["unconfirmed"]))
    L.append("  scenarios in this release:")
    for s in r["scenarios"]:
        L.append("    %-24s %s%s" % (s["id"], "can run" if s["can_run"] else "blocked — needs " + ", ".join(s["blocked_by"]),
                                     ("; judges " + ", ".join(s["judges"])) if s["judges"] else ""))
    return "\n".join(L)


def main():
    a = sys.argv[1:]
    as_json = "--json" in a
    files = [x for x in a if x != "--json"]
    if not files:
        print(__doc__.split("\n\n")[1])
        return 2
    plan = pm.Plan()
    load_rules(plan)
    reports = [examine(f, plan) for f in files]
    if as_json:
        print(json.dumps(reports, indent=1))
    else:
        print("\n\n".join(text_report(r, plan.case.get("default_level")) for r in reports))
    return 1 if any(r["format_errors"] for r in reports) else 0


if __name__ == "__main__":
    sys.exit(main())
