#!/usr/bin/env python3
"""Every command in the repository, said before it runs: the registry docs/commands.toml.

    python3 tools/trinetra.py list                 every command, one line each, by tool
    python3 tools/trinetra.py explain <command>    what it does, its steps, what it reads and
                                                   writes and which programs it starts
    python3 tools/trinetra.py why <file>           which command writes that file, and how
    python3 tools/trinetra.py status               the evidence debt first: what is not yet confirmed
                                                   by a person, proven in flight code, or agreed by the twin
    python3 tools/trinetra.py docs [--check]       write docs/COMMANDS.md from the registry
                                                   (--check: exit 1 if it is not current)

A command is named as `tool name` (`engine.py campaign`, `adcs run`) or by its name alone
when that is unique (`campaign`, `pipeline`). engine.py and pipeline.py take --dry-run, which
prints this explanation and runs nothing.

Copyright (c) 2026 Agastya. All rights reserved.
"""
import pathlib
import sys
import tomllib

from common import ROOT, write_text

REGISTRY = ROOT / "docs" / "commands.toml"
DOC = ROOT / "docs" / "COMMANDS.md"


def commands():
    return tomllib.loads(REGISTRY.read_text())["command"]


def find(words):
    """The one command `words` names: `tool name`, `name` alone when unique, or the tool alone when it has one command."""
    cs = commands()
    q = " ".join(words).strip()
    exact = [c for c in cs if q in (f"{c['tool']} {c['name']}", c["name"], f"{c['tool'].removesuffix('.py')} {c['name']}")]
    if len(exact) == 1:
        return exact[0]
    by_tool = [c for c in cs if q in (c["tool"], c["tool"].removesuffix(".py"))]
    if len(by_tool) == 1:
        return by_tool[0]
    pool = exact or by_tool
    if pool:
        raise SystemExit(f"{q!r} names {len(pool)} commands: " + ", ".join(f"{c['tool']} {c['name']}" for c in pool))
    raise SystemExit(f"no command {q!r}; `python3 tools/trinetra.py list` shows them all")


def explain(c):
    """The command in words a person can check before running it."""
    out = [f"{c['tool']} {c['name']}", "", "  " + c["what"], "", f"  usage   {c['usage']}", "", "  steps"]
    out += [f"    {i}. {s}" for i, s in enumerate(c["steps"], 1)]
    for label, key in (("reads", "reads"), ("writes", "writes"), ("starts", "runs")):
        vals = c.get(key) or []
        out.append(f"  {label:<7} " + ("; ".join(vals) if vals else "nothing"))
    return "\n".join(out)


def listing():
    cs = commands()
    out, tool = [], None
    for c in cs:
        if c["tool"] != tool:
            tool = c["tool"]
            out.append(f"\n{tool}")
        out.append(f"  {c['name']:<18} {c['what'].split('. ')[0].split(': ')[0][:96]}")
    return "\n".join(out).lstrip("\n")


def document():
    cs = commands()
    L = ["# Commands", "",
         "> Generated from `docs/commands.toml` by `python3 tools/trinetra.py docs`; never edited by hand.",
         "> `python3 tools/trinetra.py explain <command>` prints one of these; `--dry-run` on `engine.py`",
         "> and `pipeline.py` prints it and runs nothing.", "",
         "| command | what it does |", "|---|---|"]
    for c in cs:
        L.append(f"| [`{c['tool']} {c['name']}`](#{(c['tool'] + '-' + c['name']).replace('.', '').replace(' ', '-').lower()}) | {c['what']} |")
    for c in cs:
        L += ["", f"## {c['tool']} {c['name']}", "", c["what"], "", f"    {c['usage']}", "", "**Steps**", ""]
        L += [f"{i}. {s}" for i, s in enumerate(c["steps"], 1)]
        L.append("")
        for label, key in (("Reads", "reads"), ("Writes", "writes"), ("Starts", "runs")):
            vals = c.get(key) or []
            L.append(f"- **{label}:** " + ("; ".join(f"`{v}`" if "/" in v or "." in v else v for v in vals) if vals else "nothing"))
    return "\n".join(L) + "\n"


def writers(path):
    """The commands whose `writes` name `path` (a file or folder in the repository)."""
    import fnmatch
    import re
    p = pathlib.PurePosixPath(str(path).replace("\\", "/")).as_posix().lstrip("./")
    out = []
    for c in commands():
        for w in c.get("writes") or []:
            for part in re.split(r";\s*", w):
                pat = re.sub(r"\s*\(.*?\)\s*$", "", part).split(":")[-1].strip()   # drop "(or --out)" and "export:" labels
                pat = re.sub(r"<[^>]+>", "*", pat).rstrip("/")
                if pat and (fnmatch.fnmatch(p, pat) or fnmatch.fnmatch(p, pat + "/*") or p.startswith(pat + "/")):
                    out.append(c)
                    break
            else:
                continue
            break
    return out


def stale_runs_named():
    """The stored engine runs `adcs results stale` names, or None when there is no engine to ask."""
    import subprocess
    exe = ROOT / "engine" / "target" / "release" / ("adcs.exe" if sys.platform == "win32" else "adcs")
    if not exe.exists():
        return None
    p = subprocess.run([str(exe), "results", "stale"], capture_output=True, text=True, cwd=ROOT)
    return [ln for ln in p.stdout.splitlines() if ln.strip() and not ln.startswith(" ") and " of " not in ln]


def status():
    """What the repository still owes as evidence, then what stands proven."""
    import glob
    import json
    D = ROOT / "matlab_sils" / "data"
    js = lambda pat: [json.loads(pathlib.Path(f).read_text()) for f in sorted(glob.glob(str(D / pat)))]
    alg, cat, parts = js("algorithms/*.json"), js("catalogue/*.json"), js("parts/*.json")
    unconf = [a["id"] for a in alg if "UNCONFIRMED" in str(a.get("confirmed_by", ""))]
    # flown by the engine (and so by the C and Rust flight software) when the engine maps its id;
    # the data's `prototype` flag is descriptive, and where it disagrees that is itself a finding
    mapped = (ROOT / "engine" / "crates" / "adcs-sim" / "src" / "config.rs").read_text()
    proto = [a["id"] for a in alg if f'"{a["id"]}"' not in mapped]
    stale = [a["id"] for a in alg if a.get("prototype") and f'"{a["id"]}"' in mapped]
    notsel = [c["part_number"] for c in cat if not c.get("selectable")]
    synth = [p.get("part_number", "?") for p in parts if p.get("status") == "synthetic"]
    par = ROOT / "results" / "engine_parity.json"
    rows = json.loads(par.read_text()) if par.exists() else []
    disagree = [f"{r['scenario']}/{r['metric']}" for r in rows if r.get("agree") is False]
    nv = ROOT / "results" / "node_verification.json"
    v = json.loads(nv.read_text()) if nv.exists() else {"passed": 0, "checks": 0}
    debt = [
        (len(unconf), len(alg), "algorithms not yet confirmed by a person (confirmed_by is UNCONFIRMED)"),
        (len(proto), len(alg), "algorithms the engine does not fly (MATLAB twin only: no C or Rust yet)"),
        (len(synth), len(parts), "parts that are synthetic (no bought or built unit behind them)"),
        (len(notsel), len(cat), "catalogue models the datasheet leaves unselectable (a needed number is not stated)"),
        (len(disagree), len(rows), "engine-versus-twin verdicts that disagree"),
        (v["checks"] - v["passed"], v["checks"], "design-loop checks that fail (tools/verify_nodes.py)"),
    ]
    import trace as T
    t = T.build()
    stated = [r for r in t["rows"] if r["status"] != "not stated"]
    debt += [
        (sum(1 for r in stated if r["status"].startswith("owed")), len(stated), "stated requirements nothing checks (tools/trace.py)"),
        (sum(1 for r in stated if r["status"].startswith("not met")), len(stated), "stated requirements a check finds not met (results/TRACEABILITY.md)"),
        (len(t["undecided"]), t["counts"]["metrics"], "shipped metrics that neither judge nor say why they only report"),
    ]
    stale_runs = stale_runs_named()
    if stale_runs is not None:
        debt.append((len(stale_runs), len(list((ROOT / "matlab_sils" / "store" / "results_engine").rglob("manifest.json"))),
                     "stored engine runs another engine or other inputs flew (adcs results stale)"))
    owed = sum(n for n, _, _ in debt)
    L = [f"evidence debt: {owed} item(s) owed" if owed else "evidence debt: none", ""]
    L += [f"  {n:>4} of {of:<4} {what}" for n, of, what in debt]
    L += ["", "the owed items, by name:"]
    for label, items in (("unconfirmed", unconf), ("twin only", proto), ("unselectable", notsel), ("disagree", disagree),
                         ("stale flag", stale)):
        if items:
            L.append(f"  {label:<13} " + ", ".join(items[:12]) + (f" (+{len(items) - 12} more)" if len(items) > 12 else ""))
    if stale:
        L += ["", f"  ({len(stale)} algorithm file(s) say prototype = true although the engine flies them in C and Rust:",
              "   the flag is descriptive only; set it false in catalogue/algorithms/<id>.toml once a person confirms it)"]
    L += ["", f"proven: {v['passed']}/{v['checks']} design-loop checks; the generated files, the registry and the tests: python3 tools/check_all.py"]
    return "\n".join(L)


def dry_run(tool, name):
    """For a tool's --dry-run: print the explanation of `tool name` and stop."""
    print(explain(find([tool, name])))
    print("\n(dry run: nothing was run)")
    raise SystemExit(0)


def main(argv):
    if not argv or argv[0] in ("-h", "--help"):
        print(__doc__.strip())
        return 0
    cmd, rest = argv[0], argv[1:]
    if cmd == "list" and not rest:
        print(listing())
    elif cmd == "explain" and rest:
        print(explain(find(rest)))
    elif cmd == "status" and not rest:
        print(status())
    elif cmd == "why" and len(rest) == 1:
        ws = writers(rest[0])
        if not ws:
            print(f"no command in docs/commands.toml writes {rest[0]}: it is a source, edited by hand (docs/CHANGING.md says where)")
            return 1
        for c in ws:
            print(f"{rest[0]} is written by `{c['tool']} {c['name']}`: {c['what']}\n    {c['usage']}\n")
    elif cmd == "docs" and rest in ([], ["--check"]):
        text = document()
        if rest:
            ok = DOC.exists() and DOC.read_text() == text
            print("docs/COMMANDS.md is current" if ok else "docs/COMMANDS.md is STALE: run python3 tools/trinetra.py docs")
            return 0 if ok else 1
        write_text(DOC, text)
        print(f"wrote {DOC.relative_to(ROOT)} ({len(commands())} commands)")
    else:
        print(__doc__.strip(), file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
