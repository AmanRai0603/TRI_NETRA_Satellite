"""A7 (DR-kpi-001): what a metric judges. The window 'last_orbit' is the whole of the last orbit, the window 'eclipse' the samples
within a half width of each entry into or exit from the shadow, and a star tracker blind in the first half of a two-orbit run fails
the verdict that judged only the last half orbit. The relations are the design's (kpi_metric_statistics, read from the regression
copy and run in the interpreter); the scenarios of the fine hold and the slew carry the windows.
Copyright (c) 2026 Agastya. All rights reserved."""
import json
import pathlib
import sqlite3
import tempfile
import unittest

from _path import ROOT
import pcode

SCEN = ROOT / "matlab_sils" / "data" / "scenarios"
ALL, LAST_ORBIT, LAST_HALF, POINTING, AFTER_S, ECLIPSE = range(6)
P99_73 = 4


def kpistats_source():
    with sqlite3.connect(f"file:{ROOT / 'tests' / 'regression' / 'design.tndb'}?mode=ro", uri=True) as c:
        content = json.loads(c.execute("SELECT content FROM design_node WHERE id = 'kpi_metric_statistics'").fetchone()[0])
    return [v for s, f, v, _o in content["body"]["content"] if (s, f) == ("code", "pseudocode")][-1]


class Windows(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.dir = tempfile.TemporaryDirectory()
        cls.file = pathlib.Path(cls.dir.name) / "kpistats.pc"
        cls.file.write_text(kpistats_source())

    @classmethod
    def tearDownClass(cls):
        cls.dir.cleanup()

    def window(self, w, after, t, nu, per):
        n = len(t)
        r = pcode.cli("run", self.file, "--fn", "kpi_window", "--args", json.dumps([w, after, t, [0] * n, nu, n, per, [0] * n]))
        return r[-1][:r[0]]

    def test_last_orbit_is_the_whole_orbit_and_not_the_half(self):
        t = [float(i) for i in range(20)]
        nu = [1.0] * 20
        self.assertEqual(self.window(LAST_ORBIT, 0, t, nu, 10.0), list(range(9, 20)))
        self.assertEqual(self.window(LAST_HALF, 0, t, nu, 10.0), list(range(14, 20)))

    def test_eclipse_window_is_the_neighbourhood_of_each_transition(self):
        t = [float(10 * i) for i in range(10)]
        nu = [1, 1, 1, 0, 0, 0, 0, 1, 1, 1]
        self.assertEqual(self.window(ECLIPSE, 25.0, t, nu, 30.0), list(range(1, 10)))
        self.assertEqual(self.window(ECLIPSE, 10.0, t, nu, 30.0), [2, 3, 4, 6, 7, 8])
        self.assertEqual(self.window(ECLIPSE, 10.0, t, [1.0] * 10, 30.0), [])

    def test_a_tracker_blind_in_the_first_half_fails_a_whole_orbit_and_passed_the_last_half(self):
        # two orbits of 40 samples; the pointing error is 0.001 deg, and 0.05 deg while the tracker is blind in the first half of the
        # second orbit (samples 40 to 57); the limit is 0.01 deg at the 99.73rd percentile
        n, per = 80, 40.0
        t = [float(i) for i in range(n)]
        ape = [0.05 if 40 <= i < 58 else 0.001 for i in range(n)]
        verdict = {}
        for name, w in (("last_half_orbit", LAST_HALF), ("last_orbit", LAST_ORBIT)):
            idx = self.window(w, 0, t, [1.0] * n, per)
            v = pcode.cli("run", self.file, "--fn", "kpi_channel_stat", "--args",
                          json.dumps([ape, idx + [0] * (n - len(idx)), len(idx), P99_73, [0.0] * n, [0.0] * n]))[0]
            verdict[name] = v <= 0.01
        self.assertEqual(verdict, {"last_half_orbit": True, "last_orbit": False})


class Scenarios(unittest.TestCase):
    def test_the_fine_hold_judges_a_whole_orbit_after_one_to_settle(self):
        for f in sorted(SCEN.glob("fine_hold_*.json")):
            s = json.loads(f.read_text())
            self.assertGreaterEqual(s["time"]["duration_s"], 11480.0, f.name)
            self.assertFalse([m["id"] for m in s["metrics"] if m.get("window") == "last_half_orbit"], f.name)
            self.assertIn("last_orbit", {m.get("window") for m in s["metrics"]}, f.name)

    def test_the_slew_judges_from_the_time_it_has_settled(self):
        for f in sorted(SCEN.glob("slew_*.json")):
            s = json.loads(f.read_text())
            g = s["fsw"]["guidance"]
            settle = next(m for m in s["metrics"] if m["id"] == "settle_time_after_slew")
            ape = next(m for m in s["metrics"] if m["id"] == "ape_los_on_target_p9973")
            self.assertEqual(ape["window"], f"after_s:{g['t0'] + g['T_s'] + 20.0:g}", f.name)
            self.assertEqual(settle["from_s"], g["t0"] + g["T_s"], f.name)


if __name__ == "__main__":
    unittest.main()
