#!/usr/bin/env python3
"""Re-score stored runs against the case files as they are now.

A requirement is a threshold on a recorded metric: changing one (a relaxed req.rks, say) does not
change a trajectory, so the stored runs are re-judged instead of flown again. Every metric record
({"id", "value", "req", "req_key", "pass"}) whose req_key names a case requirement gets the case's
current value and a fresh verdict (value <= req, or >= for sense "min" metrics); campaign results
get their statistics recomputed from their re-judged runs.

    python3 tools/rescore.py            # every store under matlab_sils/store and results/
    python3 tools/rescore.py --dry-run
"""
import argparse, json, math, sys

from common import write_text, ROOT
sys.path.insert(0, str(ROOT / "tools"))
import engine as E                                   # noqa: E402  (summarise, case_values)

STORES = ["matlab_sils/store/results", "matlab_sils/store/results_engine", "matlab_sils/store/solutions",
          "matlab_sils/store/solutions_engine", "matlab_sils/store/pipeline", "dist/dispatch"]
SKIP = ("pipeline/cache",)
MIN_SENSE = {m["id"] for p in (ROOT / "matlab_sils/data/scenarios").glob("*.json")
             for m in json.loads(p.read_text()).get("metrics", []) if m.get("sense") == "min"}
_cases = {}


def req(case, key):
    if case not in _cases:
        p = ROOT / "matlab_sils/cases" / f"{case}.csv"
        if not p.exists():
            raise SystemExit(f"rescore: a result names case {case!r}, and {p.relative_to(ROOT)} does not exist")
        _cases[case] = E.case_values(case)
    v = _cases[case].get(key)
    return v if isinstance(v, (int, float)) and math.isfinite(v) else None


def judge(node, case, n):
    """Re-judge every metric record under node; case comes from the nearest enclosing 'case'."""
    if isinstance(node, dict):
        case = node.get("case") if isinstance(node.get("case"), str) else case
        k = node.get("req_key")
        if isinstance(k, str) and k.startswith("req.") and case and "value" in node:
            r = req(case, k)
            if r is None and (node.get("req") is not None or node.get("pass") is not None):
                # the case no longer states this requirement: the old verdict goes with it
                node["req"], node["pass"] = None, None
                n[0] += 1
            elif r is not None and node.get("req") != r:
                v = node.get("value")
                ok = isinstance(v, (int, float)) and math.isfinite(v)
                node["req"] = r
                node["pass"] = int(ok and (v >= r if node.get("id") in MIN_SENSE else v <= r))
                n[0] += 1
        for x in node.values():
            judge(x, case, n)
    elif isinstance(node, list):
        for x in node:
            judge(x, case, n)


def rejudge_stats(d):
    """A campaign summary without per-run records (the MATLAB twin's): re-judge its per-metric values,
    naming each metric's requirement from the campaign's scenario."""
    sp = ROOT / "matlab_sils/data/scenarios" / f"{d.get('scenario')}.json"
    if not (sp.exists() and isinstance(d.get("case"), str)):
        return 0
    keys = {m["id"]: m.get("requirement") for m in json.loads(sp.read_text()).get("metrics", [])}
    st = d["stats"] if isinstance(d["stats"], list) else [d["stats"]]
    n = 0
    for x in st:
        k = keys.get(x.get("id"))
        r = req(d["case"], k) if isinstance(k, str) and k.startswith("req.") else None
        if r is None or x.get("req") == r:
            continue
        ok = [isinstance(v, (int, float)) and math.isfinite(v) and (v >= r if x["id"] in MIN_SENSE else v <= r) for v in x.get("values", [])]
        x["req"], x["pass_rate"], x["pass"] = r, (sum(ok) / len(ok) if ok else None), (all(ok) if ok else None)
        n += 1
    return n


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--dry-run", action="store_true")
    a = ap.parse_args()
    files = changed = 0
    unreadable = []
    for s in STORES:
        for p in sorted((ROOT / s).rglob("*.json")):
            if any(x in str(p) for x in SKIP):
                continue
            try:
                txt = p.read_text()
                d = json.loads(txt)
            except (ValueError, UnicodeDecodeError) as e:
                unreadable.append(f"{p.relative_to(ROOT)}: {e}")
                continue
            n = [0]
            judge(d, None, n)
            if isinstance(d, dict) and "stats" in d and "per_run" not in d:
                n[0] += rejudge_stats(d)
            if not n[0]:
                continue
            if isinstance(d, dict) and isinstance(d.get("per_run"), list) and "stats" in d:
                d["stats"] = E.summarise(d["per_run"])
            files += 1
            changed += n[0]
            if not a.dry_run:
                # keep each file's own layout (compact records, indented ledgers) so the diff is the verdicts
                write_text(p, json.dumps(d, indent=1) if txt.startswith("{\n") or txt.startswith("[\n")
                             else json.dumps(d, separators=(",", ":")))
    print(f"rescore: {changed} verdict(s) in {files} file(s){' (dry run)' if a.dry_run else ''}")
    if unreadable:
        print("\n".join(f"  [FAIL] unreadable: {u}" for u in unreadable))
        sys.exit(f"rescore: {len(unreadable)} result file(s) could not be read, so their verdicts were not checked")


if __name__ == "__main__":
    main()
