#!/usr/bin/env python3
"""The release notes' generated part (docs/RELEASE_PLAN.md P14): what ships, group by group, from the
shipping record (tools/delivery.py ship), and the known gaps, from docs/ADCS_GAPS.md, written into
docs/RELEASE_NOTES.md between its markers, so the notes never say less than the records do.

    python3 tools/release_notes.py [--design DIR]    write the generated part
    python3 tools/release_notes.py --check           exit 1 when the notes are not what the records make

DIR is the design folder that ships (default: the design as seeded and carried over, which every
kit's design.tndb is). A group not accepted by its lead is named UNCONFIRMED, with why
(docs/RELEASE_PLAN.md §9).

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import argparse
import pathlib
import re
import sys
import tempfile

from common import ROOT, write_text

NOTES = ROOT / "docs" / "RELEASE_NOTES.md"
GAPS = ROOT / "docs" / "ADCS_GAPS.md"
START, END = "<!-- tn:generated:start (tools/release_notes.py) -->", "<!-- tn:generated:end -->"


def gaps():
    """[(section title, [(id, gap)])] of the gap register."""
    out, cur = [], None
    for line in GAPS.read_text(encoding="utf-8").splitlines():
        m = re.match(r"^## (.+)$", line)
        if m:
            cur = (m.group(1).strip(), [])
            out.append(cur)
            continue
        m = re.match(r"^\| ([A-Z]\d+) \| ([^|]+) \|", line)
        if m and cur:
            cur[1].append((m.group(1), m.group(2).strip()))
    return [x for x in out if x[1]]


def shipping(design):
    import delivery
    if design:
        return delivery.ship(design)[0]
    import carry_over
    import seed_design
    with tempfile.TemporaryDirectory() as d:
        seed_design.seed(pathlib.Path(d) / "Design", sync=False)
        carry_over.carry(pathlib.Path(d) / "Design")
        return delivery.ship(pathlib.Path(d) / "Design")[0]


def generated(design=None):
    rec = shipping(design)
    acc = [g for g in rec["groups"] if g["ships_as"] == "accepted"]
    un = [g for g in rec["groups"] if g["ships_as"] != "accepted"]
    L = [START, "", "## The design in this release", "",
         f"**{len(acc)} of {len(rec['groups'])} groups accepted by their leads; {len(un)} ship visibly UNCONFIRMED**, each with why "
         "(`docs/DELIVERY.md`). A node sealed UNCONFIRMED says so, with its reasons, in the node app and in `design.tndb`.", "",
         "| Wave | Group | Version | Ships as | Accepted by, or why not |", "|---|---|---|---|---|"]
    L += [f"| {g['wave']} | `{g['group']}` | {g['version'] or '—'} | {'accepted' if g['ships_as'] == 'accepted' else 'UNCONFIRMED'} | {g['accepted_by'] or g['why']} |"
          for g in rec["groups"]]
    G = gaps()
    L += ["", "## Known gaps", "", f"{sum(len(x[1]) for x in G)} gaps in the register (`docs/ADCS_GAPS.md`), each with its evidence and what closes it; none gates this release.", ""]
    for title, rows in G:
        L.append(f"- **{title}:** " + "; ".join(f"{i} {g}" for i, g in rows))
    L += ["", END]
    return "\n".join(L)


def apply(text, gen):
    if START in text and END in text:
        return text[:text.index(START)] + gen + text[text.index(END) + len(END):]
    return text.rstrip("\n") + "\n\n" + gen + "\n"


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--design")
    ap.add_argument("--check", action="store_true")
    a = ap.parse_args(argv)
    text = NOTES.read_text(encoding="utf-8")
    new = apply(text, generated(a.design))
    if a.check:
        ok = new == text
        print("release_notes: " + ("current" if ok else "not what the records make (python3 tools/release_notes.py)"))
        return 0 if ok else 1
    write_text(NOTES, new)
    print(f"release_notes: wrote the generated part of {NOTES.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
