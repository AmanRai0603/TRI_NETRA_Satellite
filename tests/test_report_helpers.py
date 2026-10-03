"""The report tools' small helpers: reading a filed run, finding a metric and its requirement,
the verdict and number formats the pages print. Copyright (c) 2026 Agastya. All rights reserved."""
import json
import math
import pathlib
import shutil
import tempfile
import unittest

import _path  # puts tools/ on the import path
import report_base as R
import report_trades
import vv_report

_ = _path  # imported for its effect: tools/ on sys.path

MAN = {"metrics": [{"id": "ape_los_p9973", "value": 3.5, "req": 10.0, "req_key": "req.ape", "pass": 1},
                   {"id": "ape_los_mean", "value": 1.0, "req": None, "req_key": "req.ape"},
                   {"id": "power_mean", "value": 0.4, "req": 0.5, "req_key": "req.pavg", "pass": 1},
                   {"id": "jitter", "value": None}]}


class LoadRun(unittest.TestCase):
    def setUp(self):
        self.d = pathlib.Path(tempfile.mkdtemp())

    def tearDown(self):
        shutil.rmtree(self.d, ignore_errors=True)

    def test_a_run_reads_as_its_manifest_and_one_array_per_channel(self):
        (self.d / "manifest.json").write_text(json.dumps(MAN))
        (self.d / "channels.csv").write_text("t_s,w_x,mode\n0,0.5,1\n1,-0.25,2\n2,1e-3,2\n")
        man, ch = R.load_run(self.d)
        self.assertEqual(man, MAN)
        self.assertEqual(list(ch), ["t_s", "w_x", "mode"])
        self.assertEqual(ch["w_x"].tolist(), [0.5, -0.25, 1e-3])
        self.assertEqual(ch["t_s"].dtype.kind, "f")

    def test_a_one_element_struct_array_is_read_as_a_list(self):
        (self.d / "manifest.json").write_text(json.dumps({"metrics": MAN["metrics"][0], "mode_log": {"t_s": 0, "mode": "detumble"}}))
        (self.d / "channels.csv").write_text("t_s\n0\n")
        man, _ = R.load_run(self.d)
        self.assertEqual(man["metrics"], [MAN["metrics"][0]])
        self.assertEqual(man["mode_log"], [{"t_s": 0, "mode": "detumble"}])

    def test_a_run_without_its_time_series_is_refused(self):
        (self.d / "manifest.json").write_text(json.dumps(MAN))
        with self.assertRaises(FileNotFoundError):
            R.load_run(self.d)

    def test_a_value_that_is_not_a_number_is_refused(self):
        (self.d / "manifest.json").write_text(json.dumps(MAN))
        (self.d / "channels.csv").write_text("t_s\nnot-a-number\n")
        with self.assertRaises(ValueError):
            R.load_run(self.d)


class Metrics(unittest.TestCase):
    def test_metric_is_the_first_whose_id_starts_with_the_prefix(self):
        self.assertEqual(R.metric(MAN, "ape_los")["id"], "ape_los_p9973")
        self.assertEqual(R.metric(MAN, "power")["value"], 0.4)
        self.assertIsNone(R.metric(MAN, "detumble"))

    def test_case_req_is_the_first_stated_requirement_of_that_key(self):
        self.assertEqual(R.case_req(MAN, "req.ape"), 10.0)
        self.assertEqual(R.case_req({"metrics": MAN["metrics"][1:2]}, "req.ape"), None, "an unstated requirement is none")
        self.assertIsNone(R.case_req(MAN, "req.rks"))

    def test_mval_is_the_exact_ids_value_else_nan(self):
        self.assertEqual(R.mval(MAN, "power_mean"), 0.4)
        self.assertTrue(math.isnan(R.mval(MAN, "power")), "no prefix match")
        self.assertIsNone(R.mval(MAN, "jitter"))

    def test_verdict(self):
        self.assertEqual(R.verdict(1), ("✔ PASS", "pass"))
        self.assertEqual(R.verdict(True), ("✔ PASS", "pass"))
        self.assertEqual(R.verdict(0), ("✖ FAIL", "fail"))
        self.assertEqual(R.verdict(None), ("—", ""))
        self.assertEqual(R.verdict(float("nan")), ("—", ""), "an unjudged metric is neither")

    def test_fnum_and_aslist(self):
        self.assertTrue(math.isnan(R.fnum(None)))
        self.assertEqual(R.fnum("2.5"), 2.5)
        self.assertEqual(R.fnum(3), 3.0)
        self.assertEqual(R.aslist(None), [])
        self.assertEqual(R.aslist({"a": 1}), [{"a": 1}])
        self.assertEqual(R.aslist([1, 2]), [1, 2])
        self.assertEqual(R.aslist(0), [0], "a falsy value is still a value")

    def test_trade_seeds_are_a_list_of_numbers(self):
        self.assertEqual(report_trades.seeds_of({}), [])
        self.assertEqual(report_trades.seeds_of({"obj_seeds": 2}), [2.0])
        s = report_trades.seeds_of({"obj_seeds": [1, None, "3"]})
        self.assertEqual((s[0], s[2]), (1.0, 3.0))
        self.assertTrue(math.isnan(s[1]))


class VVFormats(unittest.TestCase):
    def test_fmt_gives_significant_figures_and_a_dash_for_no_number(self):
        self.assertEqual(vv_report.fmt(3.14159265), "3.142")
        self.assertEqual(vv_report.fmt(3.14159265, 2), "3.1")
        self.assertEqual(vv_report.fmt(12345678), "1.235e+07")
        for x in (None, float("nan"), float("inf")):
            self.assertEqual(vv_report.fmt(x), "—")

    def test_fmt_escapes_text(self):
        self.assertEqual(vv_report.fmt("<b>&"), "&lt;b&gt;&amp;")

    def test_verdict_spans(self):
        self.assertIn('class="pass"', vv_report.verdict(1))
        self.assertIn('class="pass"', vv_report.verdict(True))
        self.assertIn('class="fail"', vv_report.verdict(0))
        self.assertIn('class="fail"', vv_report.verdict(False))
        for x in (None, 0.5, "x"):
            self.assertIn('class="na"', vv_report.verdict(x))

    def test_table_marks_the_numeric_columns(self):
        t = vv_report.table(["a", "b"], [["x", "1"], ["y", "2"]], num=(1,))
        self.assertEqual(t.count("<tr>"), 3)
        self.assertIn('<th class="">a</th><th class="n">b</th>', t)
        self.assertIn('<td class="">y</td><td class="n">2</td>', t)

    def test_a_missing_figure_is_named_not_invented(self):
        self.assertIn("missing", vv_report.svg_inline("docs/figures/no_such_figure.svg"))


if __name__ == "__main__":
    unittest.main()
