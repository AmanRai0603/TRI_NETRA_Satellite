"""The flight parity ledger (tools/flight_parity.py) finds what it says it finds: identical results pass,
and a moved value, a changed verdict, an overrun or a missing run is a regression, named.
Copyright (c) 2026 Agastya. All rights reserved."""
import copy
import json
import pathlib
import tempfile
import unittest

import _path  # puts tools/ on the import path
import flight_parity as fp

_ = _path  # imported for its effect: tools/ on sys.path

SUMMARY = {"id": "mc_x", "scenario": "x", "fsw": "c", "runs": 2, "failed_runs": 0, "wall_s": 1.0,
           "stats": [{"id": "ape", "values": [0.5, 0.25], "mean": 0.375, "pass_rate": 1.0}],
           "per_run": [{"k": k, "wall_s": 0.1 * k, "draws": {"inertia": 1.0 + k},
                        "metrics": [{"id": "ape", "value": v, "pass": 1}]} for k, v in ((1, 0.5), (2, 0.25))],
           "interpretations": [{"id": "ape", "temporal": 0.5, "ensemble": 0.5, "mixed": 0.4}]}
SILS = {"fsw": {"impl": "c (in-process)"}, "wall_s": 3.0, "metrics": [{"id": "ape", "value": 0.01, "pass": 1}]}
OILS = {"fsw": {"impl": "qemu"}, "wall_s": 30.0, "metrics": [{"id": "ape", "value": 0.0101, "pass": 1}],
        "oils": {"ticks": 100, "overruns": 0, "instructions": {"mean": 1000.0, "max": 2000.0},
                 "deadline_margin_min_s": 0.09, "worst_case_margin_s": 0.03, "cpu_load_max": 0.07}}


def lay(root, summary, sils, oils):
    e = pathlib.Path(root) / fp.ENG
    for p, x in ((e / "campaigns/mc_x/summary.json", summary), (e / "soft_oils/x/sils/manifest.json", sils),
                 (e / "soft_oils/x/oils/manifest.json", oils)):
        if x is not None:
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_text(json.dumps(x))
    return pathlib.Path(root)


class FlightParity(unittest.TestCase):
    def check(self, summary=SUMMARY, sils=SILS, oils=OILS):
        with tempfile.TemporaryDirectory() as a, tempfile.TemporaryDirectory() as b:
            base, cur = lay(a, SUMMARY, SILS, OILS), lay(b, summary, sils, oils)
            c, f1 = fp.campaigns("t", base, cur)
            o, f2 = fp.soft_oils("t", base, cur, [])
            return c, o, f1 + f2

    def test_the_same_results_pass_whatever_their_wall_time(self):
        s = copy.deepcopy(SUMMARY)
        s["wall_s"], s["per_run"][0]["wall_s"], s["fsw"] = 99.0, 5.0, "rust"
        c, o, faults = self.check(summary=s)
        self.assertEqual(faults, [])
        self.assertEqual(c[0]["differ"], 0)
        self.assertEqual(o[0]["oils"]["insn_mean_pct"], 0.0)

    def test_a_moved_campaign_value_is_named(self):
        s = copy.deepcopy(SUMMARY)
        s["per_run"][1]["metrics"][0]["value"] = 0.25 * (1 + 1e-9)
        _, _, faults = self.check(summary=s)
        self.assertTrue(any("per_run[2].metrics[ape].value" in x for x in faults), faults)

    def test_a_changed_verdict_and_an_overrun_are_named(self):
        o = copy.deepcopy(OILS)
        o["metrics"][0]["pass"], o["oils"]["overruns"] = 0, 3
        o["oils"]["instructions"]["mean"] = 1100.0
        _, rows, faults = self.check(oils=o)
        self.assertTrue(any("verdict" in x for x in faults), faults)
        self.assertTrue(any("3 overrun" in x for x in faults), faults)
        self.assertAlmostEqual(rows[0]["oils"]["insn_mean_pct"], 10.0)

    def test_soft_oils_metrics_may_move_but_sils_may_not(self):
        o = copy.deepcopy(OILS)
        o["metrics"][0]["value"] = 0.0102
        self.assertEqual(self.check(oils=o)[2], [])
        s = copy.deepcopy(SILS)
        s["metrics"][0]["value"] = 0.0102
        self.assertTrue(any("sils" in x for x in self.check(sils=s)[2]))

    def test_a_run_the_baseline_lacks_is_a_note_judged_on_its_deadline(self):
        with tempfile.TemporaryDirectory() as a, tempfile.TemporaryDirectory() as b:
            base, cur = lay(a, SUMMARY, SILS, None), lay(b, SUMMARY, SILS, OILS)
            notes = []
            rows, faults = fp.soft_oils("t", base, cur, notes)
        self.assertEqual(faults, [])
        self.assertTrue(rows[0]["oils"]["new"] and notes, notes)

    def test_a_missing_run_is_named(self):
        _, _, faults = self.check(oils=None)
        self.assertTrue(any("only in the baseline" in x for x in faults), faults)


if __name__ == "__main__":
    unittest.main()
