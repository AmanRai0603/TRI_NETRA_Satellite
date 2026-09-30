#!/usr/bin/env python3
"""Every command in the repository, said before it runs: the registry docs/commands.toml.

    python3 tools/trinetra.py list                 every command, one line each, by tool
    python3 tools/trinetra.py explain <command>    what it does, its steps, what it reads and
                                                   writes and which programs it starts
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
