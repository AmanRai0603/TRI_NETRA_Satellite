#!/usr/bin/env python3
"""The manuals' generated parts (SPEC.md §16.4); the stand-in for `cargo xtask manual`.

    python3 tools/manual_pages.py            # writes them
    python3 tools/manual_pages.py --check    # exits 1 if either is not current

manual/user/05_case_keys.md   every key of the case format: section, unit, blank
                              policy, range, level and what it means, from
                              plan/case_inputs.toml
manual/developer/01_intake.md the table between <!-- codes:begin --> and
                              <!-- codes:end -->, from tools/intake.py CHECKS
manual/developer/08_derisking.md  the table between <!-- ledger:begin --> and
                              <!-- ledger:end -->, from tools/derisk.py CODES
manual/developer/09_twin.md and spec/10b_matlab_sils.md  the table between
                              <!-- twin:begin --> and <!-- twin:end -->, from
                              tools/twin_check.py RULES

A manual that names a key the format lacks, or a check the checker lacks, tells a
person to do something that cannot work; generating these parts means it cannot.
"""

import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import plan_model as pm  # noqa: E402
import intake  # noqa: E402
import derisk  # noqa: E402
import twin_check  # noqa: E402

ROOT = pm.ROOT
KEYS = os.path.join(ROOT, "manual", "user", "05_case_keys.md")
INTAKE = os.path.join(ROOT, "manual", "developer", "01_intake.md")
LEDGER = os.path.join(ROOT, "manual", "developer", "08_derisking.md")
TWIN = [os.path.join(ROOT, "manual", "developer", "09_twin.md"), os.path.join(ROOT, "spec", "10b_matlab_sils.md")]
SECTIONS = {"meta": "About the case", "req": "What the ADCS must achieve", "mission": "Mission", "orbit": "Orbit",
            "mass": "Mass properties", "surface": "Surfaces", "magnetic": "Magnetic cleanliness", "flex": "Flexible modes",
            "resources": "Resources offered to the ADCS", "pointing": "Pointing budget inputs"}


def case_keys(plan):
    reg = plan.case
    kp = {k["requirement"]: k for k in plan.kpis}
    L = ["# Case keys", "<!-- kind: reference; depth: expert -->", "",
         "**In one line:** every key of the case format `%s`, in the template's order, with its unit, what a blank does,"
         " and what it means; generated from the format itself, so it always matches the release you are using." % reg.get("schema"), "",
         "- **Blank**: *stated* means a blank leaves the value unstated, and whatever needs it is blocked, by name;"
         " *default* means a blank takes the reference value, and the report lists it as assumed.",
         "- **Range**: the row takes `lo` and `hi`. **Level**: the row takes a level in per cent (requirements only;"
         " blank takes %s)." % reg.get("default_level"),
         "- **Sense** (requirements): *at most* means the achieved value must be at or under yours; *at least*, at or over.", ""]
    L += ["## %s" % SECTIONS["meta"], "", "| Key | What | Notes |", "|---|---|---|"]
    for m in reg.get("meta", []):
        L.append("| `%s` | %s | %s |" % (m["key"], m["label"], m.get("note", "")))
    sec = None
    for i in reg.get("input", []):
        if i["section"] != sec:
            sec = i["section"]
            L += ["", "## %s" % SECTIONS.get(sec, sec), ""]
            if sec == "req":
                L += ["| Key | What | Unit | Sense | What it means |", "|---|---|---|---|---|"]
            else:
                L += ["| Key | What | Unit | Blank | Range | What it means |", "|---|---|---|---|---|---|"]
        help_ = i.get("help", "").replace("|", "\\|")
        if sec == "req":
            sense = kp.get(i["tree_id"], {}).get("sense", "")
            L.append("| `%s` | %s | %s | %s | %s |" % (i["key"], i["label"], i["unit"],
                                                    {"<=": "at most", ">=": "at least"}.get(sense, sense), help_))
        else:
            L.append("| `%s` | %s | %s | %s | %s | %s |" % (i["key"], i["label"], i["unit"], i["blank"],
                                                         "yes" if i["range"] else "", help_))
    return "\n".join(L) + "\n"


def codes_table():
    L = ["| Code | Level | The rule |", "|---|---|---|"]
    for k, v in intake.CHECKS.items():
        L.append("| %s | %s | %s |" % (k, "warning" if k in intake.WARNINGS else "error", v))
    return "\n".join(L)


def ledger_table():
    L = ["| Code | The rule |", "|---|---|"]
    for k, v in derisk.CODES.items():
        L.append("| %s | %s |" % (k, v))
    return "\n".join(L)


def twin_table():
    L = ["| Code | The rule |", "|---|---|"]
    for k, v in twin_check.RULES.items():
        L.append("| %s | %s |" % (k, v))
    return "\n".join(L)


def with_codes(text, a="<!-- codes:begin -->", b="<!-- codes:end -->", table=None):
    i, j = text.index(a), text.index(b)
    return text[:i + len(a)] + "\n" + (table or codes_table()) + "\n" + text[j:]


def main():
    check = sys.argv[1:] == ["--check"]
    if sys.argv[1:] not in ([], ["--check"]):
        print(__doc__.split("\n\n")[1])
        return 2
    plan = pm.Plan()
    want = {KEYS: case_keys(plan), INTAKE: with_codes(open(INTAKE, encoding="utf-8").read()),
            LEDGER: with_codes(open(LEDGER, encoding="utf-8").read(), "<!-- ledger:begin -->", "<!-- ledger:end -->", ledger_table())}
    for p in TWIN:
        want[p] = with_codes(open(p, encoding="utf-8").read(), "<!-- twin:begin -->", "<!-- twin:end -->", twin_table())
    stale = []
    for path, text in want.items():
        have = open(path, encoding="utf-8").read() if os.path.exists(path) else None
        if have != text:
            stale.append(os.path.relpath(path, ROOT))
            if not check:
                open(path, "w", encoding="utf-8").write(text)
    if check:
        print("manual pages: %s" % ("current" if not stale else "STALE: %s (run tools/manual_pages.py)" % ", ".join(stale)))
        return 1 if stale else 0
    print("manual pages: %s" % ("wrote " + ", ".join(stale) if stale else "already current"))
    return 0


if __name__ == "__main__":
    sys.exit(main())
