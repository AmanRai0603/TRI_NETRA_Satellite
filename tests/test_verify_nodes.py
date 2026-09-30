"""The design-loop verifier (tools/verify_nodes.py) refuses what it exists to refuse.

A copy of one case's stored design loop (only the files the verifier reads) is checked whole,
where every check must pass, and then broken one rule at a time: a selection that is not the
rule's, a budget that is not the sum of its units, a feasibility that contradicts its failing
list, a certificate that contradicts its multipliers, a short Monte Carlo, a bought wheel that
is not the lightest that meets the need, a firmware overrun. Each break must fail the check
written for it and nothing else in its node.

Copyright (c) 2026 Agastya. All rights reserved.
"""
import json
import pathlib
import shutil
import tempfile
import unittest

from _path import ROOT
import verify_nodes as V

CASE = "ais_3u"


def load(p):
    return json.loads(p.read_text())


def save(p, d):
    p.write_text(json.dumps(d))


class Verifier(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.real = V.PIPE / CASE
        cls.it = load(cls.real / "selection.json")["iterations"]
        cls.cat = {c["part_number"]: c for c in (load(p) for p in sorted((V.MS / "data" / "catalogue").glob("*.json")))}

    def setUp(self):
        self.tmp = pathlib.Path(tempfile.mkdtemp())
        dst = self.tmp / CASE
        files = ["selection.json", "loop.json", "floquet.json", "mc/summary.json", "families.json", "soft_oils.json",
                 f"iter_{self.it}/sized/sizing.json"]
        files += [str(p.relative_to(self.real)) for p in (self.real / f"iter_{self.it}" / "sized" / "products").glob("*.json")]
        for f in files:
            (dst / f).parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(self.real / f, dst / f)
        self.dir = dst
        self.saved, V.PIPE = V.PIPE, self.tmp

    def tearDown(self):
        V.PIPE = self.saved
        shutil.rmtree(self.tmp, ignore_errors=True)

    def failed(self):
        C = V.Check()
        V.verify_case(C, CASE, self.cat)
        return [(r["node"], r["check"]) for r in C.rows if not r["ok"]]

    def assertCaught(self, node, words):
        bad = self.failed()
        self.assertTrue(any(n == node and words in c for n, c in bad), f"the break was not caught by {node}: {words!r}; failed: {bad}")

    def test_the_stored_loop_passes_whole(self):
        self.assertEqual(self.failed(), [])

    def test_a_selection_that_is_not_the_rules_is_caught(self):
        s = load(self.dir / "selection.json")
        s["selected"] = next(f for f, v in s["families"].items() if v["role"] == "solution" and f != s["selected"])
        save(self.dir / "selection.json", s)
        self.assertCaught("select", "selected:")

    def test_a_budget_that_is_not_the_sum_of_its_units_is_caught(self):
        p = self.dir / f"iter_{self.it}" / "sized" / "sizing.json"
        z = load(p)
        f = next(iter(z["families"]))
        z["families"][f]["mass_kg"] += 0.1
        save(p, z)
        self.assertCaught("budget", "sums of the fitted units")

    def test_a_feasibility_that_contradicts_its_failing_list_is_caught(self):
        g = load(self.dir / "loop.json")
        g[-1]["matrix"][0]["feasible"] = not g[-1]["matrix"][0]["feasible"]
        save(self.dir / "loop.json", g)
        self.assertCaught("assess", "feasible exactly when nothing fails")

    def test_a_certificate_that_contradicts_its_multipliers_is_caught(self):
        q = load(self.dir / "floquet.json")
        q["laws"][0]["certified"] = not q["laws"][0]["certified"]
        save(self.dir / "floquet.json", q)
        self.assertCaught("certify", "certified exactly when")

    def test_a_short_monte_carlo_is_caught(self):
        m = load(self.dir / "mc" / "summary.json")
        m["runs"] -= 1
        save(self.dir / "mc" / "summary.json", m)
        self.assertCaught("mc", "Monte Carlo ran every run")

    def test_a_wheel_that_is_not_the_lightest_meeting_the_need_is_caught(self):
        p = self.dir / f"iter_{self.it}" / "sized" / "sizing.json"
        z = load(p)
        fitted = z["parts"]["rw"]["part_number"]
        other = next(k for k, c in self.cat.items() if c["type"] == "reaction_wheel" and c["selectable"] and k != fitted)
        z["parts"]["rw"]["part_number"] = other
        save(p, z)
        self.assertCaught("select_rotor", "RW: lightest selectable catalogue model")

    def test_a_firmware_overrun_is_caught(self):
        s = load(self.dir / "soft_oils.json")
        s["oils"]["oils"]["overruns"] = 1
        save(self.dir / "soft_oils.json", s)
        self.assertCaught("soft_oils", "no overrun")


if __name__ == "__main__":
    unittest.main()
