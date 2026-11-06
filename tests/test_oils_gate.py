"""The soft-OILS deadline gate (tools/oils_gate.py): it fails when a margin falls below its stored minimum.
Nothing here flies. Copyright (c) 2026 Agastya. All rights reserved."""
import json
import unittest

import _path  # puts tools/ on the import path
import common
import oils_gate

_ = _path

PLUGIN = "qemu insn_count plugin (exact guest instructions)"


class Gate(unittest.TestCase):
    stored = {"minimum_margin_s": {"a": 0.030, "b": 0.080}}

    def meas(self, a=0.040, b=0.090, tb=PLUGIN, ov=0):
        return {"a": {"margin_s": a, "overruns": ov, "time_base": tb}, "b": {"margin_s": b, "overruns": 0, "time_base": PLUGIN}}

    def test_margins_above_their_minima_pass(self):
        self.assertEqual(oils_gate.judge(self.meas(), self.stored), [])

    def test_a_margin_below_its_minimum_fails_by_name(self):
        bad = oils_gate.judge(self.meas(a=0.029), self.stored)
        self.assertEqual(len(bad), 1)
        self.assertTrue(bad[0].startswith("a: deadline margin 29.000 ms is below the stored minimum 30.000 ms"), bad)

    def test_an_overrun_a_missing_scenario_and_a_wrong_time_base_fail(self):
        self.assertTrue(any("overrun" in x for x in oils_gate.judge(self.meas(ov=2), self.stored)))
        self.assertTrue(any("not flown" in x for x in oils_gate.judge({"a": self.meas()["a"]}, self.stored)))
        self.assertTrue(any("instruction-count plugin" in x for x in oils_gate.judge(self.meas(tb="qemu without the instruction-count plugin"), self.stored)))

    def test_the_stored_file_names_at_least_five_scenarios_and_how_it_was_set(self):
        s = json.loads(oils_gate.STORE.read_text())
        self.assertGreaterEqual(len(s["minimum_margin_s"]), 5)
        self.assertTrue(s["set"]["how"])
        for scen in s["minimum_margin_s"]:
            self.assertTrue((common.ROOT / "matlab_sils" / "data" / "scenarios" / f"{scen}.json").exists(), scen)


if __name__ == "__main__":
    unittest.main()
