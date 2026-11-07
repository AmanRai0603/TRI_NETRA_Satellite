"""The golden table (tools/golden.py, DR-vv-003): it covers every scenario, a changed metric is named by scenario
and metric, and the tolerance rules hold. The comparison is tested directly; the one re-fly is a short scenario.
Copyright (c) 2026 Agastya. All rights reserved."""
import contextlib
import copy
import io
import json
import pathlib
import tempfile
import unittest
from unittest import mock

import _path  # puts tools/ on the import path
import golden
from engine_base import BIN, DATA

_ = _path  # imported for its effect: tools/ on sys.path

SHORT = "agile_slew_img"


def row(value=1.0, ok=1, tol=1e-3):
    return {"algorithms": {"attitude": "mekf"}, "metrics": {"m": {"kind": "k", "value": value, "pass": ok, "rel_tol": tol}}}


class Table(unittest.TestCase):
    def test_every_scenario_is_in_the_table(self):
        t = json.loads(golden.TABLE.read_text())
        want = sorted(p.stem for p in DATA.glob("*.json"))
        self.assertEqual(len(want), 80)
        self.assertEqual(sorted(t["scenarios"]), want)
        for name, r in t["scenarios"].items():
            self.assertTrue(r["metrics"], name)
            for mid, m in r["metrics"].items():
                self.assertIn("rel_tol", m, f"{name}: {mid}")
        self.assertTrue(all(t["fingerprint"].values()))


class Compare(unittest.TestCase):
    def test_same_run_has_no_difference(self):
        self.assertEqual(golden.compare(row(), row(), True), [])
        self.assertEqual(golden.compare(row(), row(), False), [])

    def test_bit_exact_flags_the_last_bit(self):
        d = golden.compare(row(1.0), row(1.0 + 2 ** -52), True)
        self.assertEqual([x[0] for x in d], ["m"])

    def test_other_fingerprint_inside_and_outside_the_tolerance(self):
        self.assertEqual(golden.compare(row(1.0, tol=1e-3), row(1.0005), False), [])
        d = golden.compare(row(1.0, tol=1e-3), row(1.01), False)
        self.assertEqual(len(d), 1)
        self.assertEqual(d[0][:3], ("m", 1.0, 1.01))

    def test_a_verdict_or_the_algorithms_get_no_tolerance(self):
        self.assertEqual(golden.compare(row(ok=1), row(ok=0), False)[0][3], "verdict")
        other = row()
        other["algorithms"] = {"attitude": "ekf"}
        self.assertEqual(golden.compare(row(), other, False)[0][0], "algorithms")

    def test_a_missing_metric_is_a_difference(self):
        new = row()
        new["metrics"]["extra"] = new["metrics"]["m"]
        self.assertEqual(golden.compare(row(), new, False)[0][0], "extra")


@unittest.skipUnless(BIN.exists(), "engine not built: cd engine && cargo build --release -p adcs-cli")
class Refly(unittest.TestCase):
    def run_check(self, table):
        with tempfile.TemporaryDirectory() as d:
            path = pathlib.Path(d) / "golden.json"
            path.write_text(json.dumps(table))
            out = io.StringIO()
            with mock.patch.object(golden, "TABLE", path), contextlib.redirect_stdout(out):
                rc = golden.main(["check", "--only", SHORT, "--jobs", "1"])
        return rc, out.getvalue()

    def test_a_perturbed_metric_is_named(self):
        table = json.loads(golden.TABLE.read_text())
        rc, _ = self.run_check(table)
        self.assertEqual(rc, 0)
        bad = copy.deepcopy(table)
        mid = next(iter(bad["scenarios"][SHORT]["metrics"]))
        bad["scenarios"][SHORT]["metrics"][mid]["value"] *= 1.5
        bad["scenarios"][SHORT]["metrics"][mid]["value"] += 1.0
        rc, text = self.run_check(bad)
        self.assertEqual(rc, 1)
        self.assertIn(f"DIFF {SHORT}: {mid}:", text)


if __name__ == "__main__":
    unittest.main()
