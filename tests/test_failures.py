"""A failure is never counted as a success, and nothing old is read as new.

Each test makes one failure happen and checks what the tool does with it: a campaign run that
failed to fly counts as a failed run for every judged requirement; engine.py exits non-zero when
any run failed; a case value that is not a number is refused by name; a requirement the case no
longer states loses its old verdict; the V&V report refuses to be built on a missing ledger.

Copyright (c) 2026 Agastya. All rights reserved.
"""
import pathlib
import sys
import tempfile
import unittest
from unittest import mock

import _path  # puts tools/ on the import path
import common
import engine
import engine_campaigns
import rescore
import vv_report

_ = _path  # imported for its effect: tools/ on sys.path


def run(k, value, passed):
    return {"k": k, "metrics": [{"id": "ape", "unit": "deg", "req": 10.0, "value": value, "pass": passed},
                                {"id": "info", "unit": "s", "req": None, "value": value, "pass": None}]}


class Campaigns(unittest.TestCase):
    def test_a_run_that_failed_to_fly_counts_as_a_failed_run(self):
        runs = [run(1, 5.0, 1), run(2, 6.0, 1), {"k": 3, "failed": True, "error": "boom", "metrics": []}]
        ape, info = engine_campaigns.summarise(runs)
        self.assertEqual(ape["pass_rate"], 2 / 3, "the failed run is a failed verdict, not left out")
        self.assertFalse(ape["pass"])
        self.assertEqual(ape["n_failed_runs"], 1)
        self.assertEqual(ape["n_valid"], 2, "its value is not invented")
        self.assertIsNone(info["pass_rate"], "a metric with no requirement is not judged, failed run or not")


class EngineExit(unittest.TestCase):
    def test_engine_py_exits_non_zero_when_a_command_reports_failures(self):
        with mock.patch.object(sys, "argv", ["engine.py", "twin-parity"]), \
             mock.patch.object(engine, "BIN", pathlib.Path(sys.executable)), \
             mock.patch.object(engine, "twin_parity", lambda a: 3):
            with self.assertRaises(SystemExit) as e:
                engine.main()
        self.assertIn("3 failure(s)", str(e.exception.code))

    def test_and_returns_normally_when_nothing_failed(self):
        with mock.patch.object(sys, "argv", ["engine.py", "twin-parity"]), \
             mock.patch.object(engine, "BIN", pathlib.Path(sys.executable)), \
             mock.patch.object(engine, "twin_parity", lambda a: 0):
            engine.main()


class CaseValues(unittest.TestCase):
    def setUp(self):
        self.tmp = pathlib.Path(tempfile.mkdtemp())
        (self.tmp / "matlab_sils" / "cases").mkdir(parents=True)
        self.saved, common.ROOT = common.ROOT, self.tmp

    def tearDown(self):
        common.ROOT = self.saved
        import shutil
        shutil.rmtree(self.tmp, ignore_errors=True)

    def write(self, rows):
        head = "section,key,label,unit,value,lo,hi,level,note\n"
        (self.tmp / "matlab_sils" / "cases" / "c.csv").write_text(head + "".join(f"s,{k},\"a, label\",u,{v},,,,\n" for k, v in rows))

    def test_numbers_blanks_and_meta_text(self):
        self.write([("meta.title", "a title"), ("orbit.alt", "550"), ("req.slew", "")])
        self.assertEqual(common.case_values("c"), {"orbit.alt": 550.0})

    def test_text_where_a_number_belongs_is_refused_by_name(self):
        for bad in ("high", "nan", "inf"):
            self.write([("orbit.alt", bad)])
            with self.assertRaises(SystemExit) as e:
                common.case_values("c")
            self.assertIn("orbit.alt", str(e.exception))

    def test_a_missing_case_is_refused(self):
        with self.assertRaises(SystemExit):
            common.case_values("nope")


class Rescore(unittest.TestCase):
    def test_a_requirement_the_case_no_longer_states_loses_its_verdict(self):
        node = {"case": "c", "metrics": [{"id": "ape", "req_key": "req.ape", "value": 3.0, "req": 10.0, "pass": 1}]}
        with mock.patch.object(rescore, "req", lambda case, key: None):
            n = [0]
            rescore.judge(node, None, n)
        m = node["metrics"][0]
        self.assertEqual((m["req"], m["pass"], n[0]), (None, None, 1))

    def test_a_changed_requirement_is_re_judged(self):
        node = {"case": "c", "metrics": [{"id": "ape", "req_key": "req.ape", "value": 3.0, "req": 10.0, "pass": 1}]}
        with mock.patch.object(rescore, "req", lambda case, key: 2.0):
            rescore.judge(node, None, [0])
        self.assertEqual((node["metrics"][0]["req"], node["metrics"][0]["pass"]), (2.0, 0))


class VVReport(unittest.TestCase):
    def test_a_missing_ledger_is_recorded_unless_optional(self):
        vv_report.MISSING.clear()
        p = common.ROOT / "results" / "no_such_ledger.json"
        self.assertIsNone(vv_report.jl(p))
        self.assertIsNone(vv_report.jl(p, optional=True))
        self.assertEqual(vv_report.MISSING, ["results/no_such_ledger.json"])
        vv_report.MISSING.clear()


if __name__ == "__main__":
    unittest.main()
