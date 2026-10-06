#!/usr/bin/env python3
"""Writes every example file the package ships, from the plan, the ledger and
the templates, so the examples always match the templates they show:

    python3 tools/make_examples.py

forms/examples/
  <gf_7>.form.html                  a computed node, as a team member receives it: the
                                    standard's worked example (SPEC.md §5.12)
  <p1k_0>.form.html                 a requirement row
  <rk4_0>.form.html                 a risk-branch conclusion row (SPEC.md §5.13)
  <gf_7>.seed.form.html             a seed form (SPEC.md §5.8)
  new_node.form.html                the blank new-node request
  <gf_7>.request.returned.html      a filled change with its belief record, as the developer
                                    team returned it: "check passed", then "released 1.1.0"
  case_ais_img_3u.editor.html, case_new_case.editor.html   the case editor, filled and blank
results/examples/                   the two demonstration results (tools/results.py demo)

Nothing here is evidence. The requester in the returned example is a
placeholder, never a person's name.
"""

import json
import os
import subprocess
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import forms  # noqa: E402
import intake  # noqa: E402
import plan_model as pm  # noqa: E402

ROOT = pm.ROOT
OUT = os.path.join(ROOT, "forms", "examples")


def returned(plan):
    f = json.loads(json.dumps(forms.node_form(plan, "gf_7")))
    R = f["request"]
    R["proposed"]["note"] = ("Laminar flow only: above a Reynolds number of about 2000 the turbulent losses shorten it; "
                             "use the ring's Reynolds-number node to check.")
    R.update({"requested_by": "<team member>", "team": "GNC", "contact": "<email>",
              "reason": "The spin-down time is quoted without its validity limit; the note should say when the laminar relation stops holding.",
              "summary": "Put the laminar limit on the node's note, where a reader looks first.",
              "derisk": {"area": "node",
                         "believed": "A reader can tell from the node's page when the spin-down relation holds.",
                         "status": "broke",
                         "tested": "Read the node's page in release 1.0 as a new reader would: the note gives no validity limit, and the laminar assumption appears only lower down, under the assumptions.",
                         "now_know": "The limit has to be on the note, where a reader looks first; an assumption further down is read by experts only.",
                         "cost_k": "", "plan_change": "The note states the laminar limit and points to the Reynolds-number check.",
                         "previous_issue": "The note did not say when the relation stops holding.",
                         "benefit": "A reader sees the laminar limit before using the spin-down time.",
                         "would_test": "", "moves": []}})
    tmp = tempfile.mkdtemp()
    p = os.path.join(tmp, "req.html")
    open(p, "w", encoding="utf-8").write(forms.fill(forms.NODE_TEMPLATE, f["title"], f, forms.NODE_OPEN))
    rc, F, info, form = intake.run_check(p, False, os.path.join(tmp, "intake"), quiet=True, plan=plan)
    if rc:
        raise SystemExit("the returned example does not pass the checker: %s" % [(x.code, x.text) for x in F if x.level == "error"])
    rid = intake.request_id(form)
    rep = os.path.join(tmp, "intake", rid, "check.json")
    warn = ", ".join("%s" % x.code for x in F if x.level == "warning")
    a = os.path.join(tmp, "a.html")
    intake.reply(p, "check passed", "Passed the checker%s. Implementation next." % (" with warnings (%s)" % warn if warn else ""), "", rep, a)
    out = os.path.join(OUT, "%s.request.returned.html" % plan.node_id("gf_7"))
    intake.reply(a, "released", "Released. Run your case in this version, and add feedback to this file if the node page does not say what you asked.", "1.1.0", None, out)
    return out


def main():
    plan = pm.Plan()
    os.makedirs(OUT, exist_ok=True)
    for f in os.listdir(OUT):
        if f.endswith(".html"):
            os.remove(os.path.join(OUT, f))
    written = [forms.export_node(plan, t, OUT) for t in ("gf_7", "p1k_0", "rk4_0")]
    written.append(forms.export_node(plan, "gf_7", OUT, "seed"))
    written.append(forms.export_new(plan, OUT))
    written.append(returned(plan))
    written.append(forms.export_case(plan, os.path.join(ROOT, "plan", "cases", "ais_img_3u.csv"), OUT))
    written.append(forms.export_case(plan, None, OUT))
    subprocess.run([sys.executable, os.path.join(ROOT, "tools", "results.py"), "demo", "--out", os.path.join(ROOT, "results", "examples")],
                   check=True, stdout=subprocess.DEVNULL)
    for w in written:
        print("wrote", os.path.relpath(w, ROOT))
    print("wrote results/examples/ (two demonstration results)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
