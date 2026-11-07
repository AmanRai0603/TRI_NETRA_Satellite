"""The one verdict of a requirement that holds at a level (A1, DR-kpi-001): the design's, asked of the engine.

Of n runs, k are successes (the metric finite and within the limit: a run with no finite value is a failure, never left out).
kpi_metric_statistics's kpi_confidence_verdict gives pass, fail or not demonstrated from the exact (Clopper-Pearson) bounds at
the confidence the design states (the node kpi_verdict_confidence). tools/evaluate.py (the closures) and tools/trace.py (the
matrix) call this and keep no bound or level of their own; this file formats the design's answer and reads the stated value.

    from verdict import verdict, count, level, confidence
    verdict(12, 12, 0.9973)["text"]      # 'not demonstrated (R >= 0.78 at 95 %)'

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import json
import math

from common import ROOT
from design_call import call

STATED = ROOT / "matlab_sils" / "data" / "stated.json"
PASS, FAIL, NOT_DEMONSTRATED = 0, 1, 2           # kpistats' Verdict, in its order
NAMES = {PASS: "pass", FAIL: "fail", NOT_DEMONSTRATED: "not demonstrated"}


def confidence():
    """The confidence the design states for a verdict (kpi_verdict_confidence)."""
    return float(json.loads(STATED.read_text())["value"]["kpi_verdict_confidence"])


def level(claim=None, stated=""):
    """The probability a run must meet the requirement with: the campaign's claim, else the level the case states for it (the
    percentage of its level column), else 0 (each run must meet it)."""
    if claim:
        return float(claim)
    return float(stated) / 100.0 if str(stated or "").strip() else 0.0


def count(values, limit, sense="<="):
    """How many of the runs' values meet the limit: a value that is not a finite number is a failure."""
    ok = (lambda x: x <= limit) if sense == "<=" else (lambda x: x >= limit)
    return sum(1 for x in values if x is not None and math.isfinite(x) and ok(x))


def verdict(n, k, p, g=None, feasible=True):
    """The design's verdict of k successes among n runs at the level p and the confidence g (the design's when not given):
    {answer: pass | fail | not demonstrated, text, lower, upper, n, k, level, confidence}. A selection marked infeasible
    never passes: it is a fail."""
    g = confidence() if g is None else g
    v, lower, upper = call("kpistats::kpi_confidence_verdict", n, k, p, g)
    v = int(v)
    if v == NOT_DEMONSTRATED:
        text = f"not demonstrated (R >= {lower:.2f} at {100 * g:g} %)"
    elif v == PASS:
        text = "pass" if p <= 0 else f"pass (R >= {lower:.4f} at {100 * g:g} %)"
    else:
        text = "fail" if p <= 0 or n <= 0 else f"fail (R <= {upper:.4f} at {100 * g:g} %)"
    if not feasible and v == PASS:
        v, text = FAIL, "fail (the selected design is marked infeasible)"
    return {"answer": NAMES[v], "text": text, "lower": lower, "upper": upper, "n": n, "k": k, "level": p, "confidence": g}
