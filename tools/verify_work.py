#!/usr/bin/env python3
"""Check a piece of work by the files it changed: the checks those files need, and nothing else.

    python3 tools/verify_work.py --since REV            run the checks the changes since REV need
    python3 tools/verify_work.py --since REV --list     only say which checks, and for which files
    python3 tools/verify_work.py --since REV --kind code   also the checks of that kind of work
    python3 tools/verify_work.py --since REV --agent ID    and mark the agent checked in the token ledger

Which checks each path needs is .claude/model_policy.toml [[paths]] (the first glob of a path wins; a
path no glob matches needs the Python tests); the kind's checks are its [[work]] entry. A check is a
check of tools/check_all.py, or one under [checks] (the parity gate). This is how the work of a
light or standard agent is proven: by the deterministic checks, which cost no tokens, before anyone
reads it. Uncommitted changes count as changed. Exit status 1 when a check fails.

Copyright (c) 2026 Agastya. All rights reserved.
"""
import argparse
import fnmatch
import subprocess
import sys

from common import ROOT
import check_all
import tokens

ORDER = [c[0] for c in check_all.CHECKS + check_all.OCTAVE + check_all.PAGES + check_all.MUTATION]


def changed(since):
    out = subprocess.run(["git", "diff", "--name-only", since], cwd=ROOT, capture_output=True, text=True, check=True).stdout
    new = subprocess.run(["git", "ls-files", "--others", "--exclude-standard"], cwd=ROOT, capture_output=True, text=True, check=True).stdout
    return sorted(set(out.split()) | set(new.split()))


def needs(path, pol):
    for p in pol["paths"]:
        if fnmatch.fnmatch(path, p["glob"]):
            return p["checks"], p["glob"]
    return ["python-tests"], "(no glob)"


def plan(files, pol, kind=None):
    want, why = {}, {}
    for f in files:
        cks, g = needs(f, pol)
        for c in cks:
            want.setdefault(c, set()).add(f)
            why.setdefault(c, set()).add(g)
    if kind:
        w = [w for w in pol["work"] if w["kind"] == kind]
        if not w:
            raise SystemExit(f"verify_work: no kind of work {kind!r}; the kinds are " + ", ".join(x["kind"] for x in pol["work"]))
        for c in w[0]["checks"]:
            want.setdefault(c, set()).add(f"(kind {kind})")
    extra = list(pol.get("checks", {}))
    names = [c for c in ORDER if c in want] + [c for c in extra if c in want]
    unknown = set(want) - set(names)
    if unknown:
        raise SystemExit("verify_work: the policy names checks that do not exist: " + ", ".join(sorted(unknown)))
    return [(c, sorted(want[c])) for c in names]


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--since", required=True, help="the revision the work started from")
    ap.add_argument("--kind", help="the kind of work (.claude/model_policy.toml [[work]])")
    ap.add_argument("--list", action="store_true", help="only list the checks")
    ap.add_argument("--agent", help="mark this agent's row of results/token_ledger.csv with the verdict")
    a = ap.parse_args(argv)
    pol = tokens.policy()
    files = changed(a.since)
    todo = plan(files, pol, a.kind)
    print(f"verify_work: {len(files)} files changed since {a.since}; {len(todo)} checks")
    for c, fs in todo:
        print(f"  {c:<18} for {', '.join(fs[:4])}{' and %d more' % (len(fs) - 4) if len(fs) > 4 else ''}")
    if a.list or not todo:
        return 0
    lib = [c for c, _ in todo if c not in pol.get("checks", {})]
    failed = []
    if lib and check_all.main(["--only", *lib]):
        failed.append("check_all")
    for c, _ in todo:
        if c in pol.get("checks", {}):
            print(f"== {c}: {' '.join(pol['checks'][c])}", flush=True)
            if subprocess.run(pol["checks"][c], cwd=ROOT).returncode:
                failed.append(c)
    verdict = "failed" if failed else "ok"
    print(f"\nverify_work: {verdict}" + (f" ({', '.join(failed)})" if failed else ""))
    if a.agent:
        tokens.main(["checked", a.agent, verdict])
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
