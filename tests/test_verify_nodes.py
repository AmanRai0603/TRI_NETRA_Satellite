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

import _path  # puts tools/ on the import path
import verify_nodes as V

_ = _path  # imported for its effect: tools/ on sys.path

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
        # the fault campaign, when the stored loop flew one (node faults, B2.6)
        if (self.real / "faults.json").exists():
            files += ["faults.json"] + [str(p.relative_to(self.real)) for p in (self.real / "faults").rglob("*.json")]
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

    # node faults (B2.6): a made-up campaign on the stored products, counted by select's own code
    def campaign(self, broken=("mtq", "coil_fail")):
        import math
        import pipeline_verify as PV
        sel = load(self.dir / "selection.json")
        FP = V.P["faults"]
        T = 2 * math.pi * math.sqrt((6378137 + V.case_req(CASE)["orbit.alt"] * 1e3) ** 3 / 3.986004418e14)
        dur = 17217
        t = round(dur - (0.5 + FP["lead_orbits"]) * T)
        ok = [{"id": "ape_los_p9973", "pass": 1}, {"id": "ake_los_p9973", "pass": 1}, {"id": "power_peak", "pass": None}]
        miss = [{"id": "ape_los_p9973", "pass": 0}] + ok[1:]
        fams = {}
        for f, v in sel["families"].items():
            if v["role"] not in FP["roles"]:
                continue
            fill = load(self.dir / f"iter_{self.it}" / "sized" / "products" / f"{v['product']}.json")["fill"]
            fly, skipped = PV.fault_set(fill, FP["set"], t)
            for x in fly:
                bad = (f, x["kind"]) == broken
                x.update({"flown": True, "pass": not bad, "failing": ["ape_los_p9973"] if bad else [], "also_fails_without_fault": [],
                          "metrics": {str(sd): (miss if bad else ok) for sd in FP["seeds"]}})
            rec = {"product": v["product"], "t_s": t, "nominal": {str(sd): ok for sd in FP["seeds"]}, "faults": fly + skipped}
            rec["gaps"] = PV.fault_gaps(rec)
            fams[f] = rec
            (self.dir / "faults" / f).mkdir(parents=True, exist_ok=True)
            save(self.dir / "faults" / f / "nominal.json", {"time": {"duration_s": dur}})
        fl = {"families": fams}
        save(self.dir / "faults.json", fl)
        sel.update(PV.select_pick(CASE, sel["families"], fl))
        save(self.dir / "selection.json", sel)
        return sel, fl

    def test_a_counted_fault_campaign_passes_whole(self):
        sel, _ = self.campaign()
        self.assertEqual(sel["families"]["mtq"]["fault_gaps"], ["fault: coil_fail: ape_los_p9973"])
        self.assertEqual(self.failed(), [])

    def test_a_selection_that_predates_the_campaign_is_noted_not_failed(self):
        # a selection made before node faults existed: no fault policy, no fault gaps, no campaign
        sel = load(self.dir / "selection.json")
        sel.pop("fault_policy", None)
        for f in sel["families"].values():
            f.pop("fault_gaps", None)
            f["gaps"] = [g for g in f["gaps"] if not g.startswith("fault:")]
            f["feasible"] = not f["gaps"]
        save(self.dir / "selection.json", sel)
        (self.dir / "faults.json").unlink(missing_ok=True)
        V.NOTES.clear()
        self.assertEqual([x for x in self.failed() if x[0] in ("faults", "select")], [])
        self.assertTrue(any("predates node faults" in n for n in V.NOTES))

    def test_a_fault_verdict_that_contradicts_its_metrics_is_caught(self):
        _, fl = self.campaign()
        x = next(x for x in fl["families"]["mtq"]["faults"] if x["kind"] == "coil_fail")
        x["pass"], x["failing"] = True, []
        save(self.dir / "faults.json", fl)
        self.assertCaught("faults", "mtq / coil_fail: passes exactly when")

    def test_a_fault_gap_select_did_not_count_is_caught(self):
        sel, _ = self.campaign()
        sel["families"]["mtq"]["gaps"] = [g for g in sel["families"]["mtq"]["gaps"] if not g.startswith("fault: ")]
        save(self.dir / "selection.json", sel)
        self.assertCaught("select", "mtq: fault gaps are feasibility gaps")

    def test_a_fault_the_product_carries_left_unflown_is_caught(self):
        _, fl = self.campaign()
        fl["families"]["mtq_fmr"]["faults"] = [x for x in fl["families"]["mtq_fmr"]["faults"] if x["kind"] != "rotor_fail"]
        save(self.dir / "faults.json", fl)
        self.assertCaught("faults", "mtq_fmr: the set's faults its product carries are flown")


if __name__ == "__main__":
    unittest.main()
