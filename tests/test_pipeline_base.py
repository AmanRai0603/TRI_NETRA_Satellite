"""The design loop's shared helpers: candidate names, the gain grid, hashing, the violation sums.
Copyright (c) 2026 Agastya. All rights reserved."""
import json
import pathlib
import shutil
import tempfile
import unittest

import _path  # puts tools/ on the import path
import pipeline_base as B
import pipeline_design as D
from design_call import Buf, call

_ = _path  # imported for its effect: tools/ on sys.path


class Candidates(unittest.TestCase):
    def test_a_plain_law_has_no_tuning(self):
        self.assertEqual(B.split_alg("mtq_pd"), ("mtq_pd", {}))
        self.assertEqual(B.split_alg(None), (None, {}))
        self.assertEqual(B.split_alg(""), ("", {}))

    def test_a_bandwidth_candidate_tunes_the_wheel_bandwidth(self):
        self.assertEqual(B.split_alg("pid@bw2.5"), ("pid", {"rw_bandwidth": 2.5}))

    def test_a_gain_candidate_gives_every_scale_as_a_number(self):
        self.assertEqual(B.split_alg("mtq_pd@mtq_gain_p=0.25,mtq_gain_d=4,handover_out_dps=1"),
                         ("mtq_pd", {"mtq_gain_p": 0.25, "mtq_gain_d": 4.0, "handover_out_dps": 1.0}))

    def test_a_malformed_gain_is_refused_not_guessed(self):
        with self.assertRaises(ValueError):
            B.split_alg("mtq_pd@mtq_gain_p")
        with self.assertRaises(ValueError):
            B.split_alg("mtq_pd@mtq_gain_p=high")

    def test_the_tune_grid_is_every_law_at_every_grid_point(self):
        for slot, grid in B.TUNE["grids"].items():
            g = B.tune_grid(slot)
            n = 1
            for v in grid.values():
                n *= len(v)
            self.assertEqual(len(g), n * len(B.CANDIDATES[slot]), slot)
            self.assertEqual(len(set(g)), len(g), "no point twice")
            self.assertEqual({B.split_alg(x)[0] for x in g}, set(B.CANDIDATES[slot]))

    def test_each_grid_name_splits_back_to_its_grid_point(self):
        for slot, grid in B.TUNE["grids"].items():
            for name in B.tune_grid(slot):
                law, tune = B.split_alg(name)
                self.assertEqual(list(tune), list(grid), name)
                for k, v in tune.items():
                    self.assertIn(v, grid[k], name)

    def test_an_unknown_slot_is_refused(self):
        with self.assertRaises(KeyError):
            B.tune_grid("no_such_slot")

    def test_the_slot_table_names_registry_candidates(self):
        for slot in set(B.SLOT.values()):
            self.assertIn(slot, B.CANDIDATES)
        self.assertEqual(B.SLOT[("nadir_pointing", "rw+mtq")], "pointing")
        self.assertEqual(B.SLOT[("nadir_pointing", "mtq")], "mtq_pointing")
        self.assertNotIn(("detumble", "rw+mtq"), B.SLOT)


class Parts(unittest.TestCase):
    def test_each_actuator_is_given_authority_by_its_sized_part(self):
        """design_loop_converge's method (design/looprules.pc, S7.15), through the engine: the coils' pointing-grade coil,
        else the actuator's own part."""
        for a, part in (("mtq", "mtqp"), ("rw", "rw"), ("cmg", "cmg"), ("vscmg", "vscmg"), ("fmr", "fmr"), ("rcs", "rcs")):
            p = call("looprules::loop_part", D.ACTUATORS.index(a))[0]
            self.assertEqual(D.PARTS[int(p)], part)

    def test_an_unknown_actuator_is_refused(self):
        with self.assertRaises(ValueError):
            D.ACTUATORS.index("sail")

    def test_the_failure_class_comes_from_the_metric_name(self):
        self.assertEqual([B.cls(m) for m in ("power_mean", "power_peak", "ake_los_p9973", "propellant", "ape_los_p9973", "rate_stability_p9973",
                                             "detumble_time")],
                         ["power", "power", "knowledge", "propellant", "performance", "performance", "performance"])

    def test_an_option_is_usable_only_when_the_family_has_its_actuator_and_its_dump(self):
        self.assertTrue(B.usable({"actuator": "mtq"}, ["mtq"]))
        self.assertTrue(B.usable({"actuator": "rw", "dump": "mtq"}, ["rw", "mtq"]))
        self.assertFalse(B.usable({"actuator": "rw", "dump": "mtq"}, ["rw"]))
        self.assertFalse(B.usable({"actuator": "rw", "dump": "rcs"}, ["rw", "mtq"]))
        self.assertTrue(B.usable({"actuator": "rw", "dump": ""}, ["rw"]), "no dump needs nothing more")


class Hashing(unittest.TestCase):
    def test_the_key_is_short_stable_and_order_free_within_a_dict(self):
        a = B.sha({"x": 1, "y": [1, 2]}, b"bytes", 3)
        self.assertEqual(len(a), 16)
        self.assertEqual(a, B.sha({"y": [1, 2], "x": 1}, b"bytes", 3))

    def test_any_changed_part_changes_the_key(self):
        a = B.sha({"x": 1}, b"case", 1)
        self.assertNotEqual(a, B.sha({"x": 2}, b"case", 1))
        self.assertNotEqual(a, B.sha({"x": 1}, b"case!", 1))
        self.assertNotEqual(a, B.sha({"x": 1}, b"case", 2))
        self.assertNotEqual(a, B.sha(b"case", {"x": 1}, 1), "the order of the parts matters")

    def test_the_case_bytes_are_the_case_file(self):
        self.assertEqual(B.case_bytes("ais_3u"), (B.MS / "cases" / "ais_3u.csv").read_bytes())
        with self.assertRaises(FileNotFoundError):
            B.case_bytes("no_such_case")


class Files(unittest.TestCase):
    def setUp(self):
        self.d = pathlib.Path(tempfile.mkdtemp())

    def tearDown(self):
        shutil.rmtree(self.d, ignore_errors=True)

    def test_write_makes_its_folder_and_reads_back(self):
        p = self.d / "a" / "b" / "x.json"
        B.write(p, {"k": [1, 2.5, None]})
        self.assertEqual(json.loads(p.read_text()), {"k": [1, 2.5, None]})
        self.assertEqual(B.jl_(p), {"k": [1, 2.5, None]})

    def test_a_missing_ledger_reads_as_none(self):
        self.assertIsNone(B.jl_(self.d / "absent.json"))


class Violations(unittest.TestCase):
    """The loop's violation sums are design_loop_converge's (design/looprules.pc, S7.15), asked through the engine."""

    def test_rate_violation_sums_only_rate_stability_each_capped_at_ten(self):
        # the options' violations in order: rate_stability_p9973 0.5, ape_los_p9973 3.0; rate_stability_p9973 40.0
        vval, vrate = [0.5, 3.0, 40.0], [1, 0, 1]
        self.assertEqual(call("looprules::loop_rate_violation", Buf(vval), Buf(vrate), 3)[0], 10.5)
        self.assertEqual(call("looprules::loop_rate_violation", Buf([0.0]), Buf([0]), 0)[0], 0.0)

    def test_family_violation_is_the_best_option_per_mode_and_zero_for_a_passing_mode(self):
        # modes m1, m2, m3; options (m1 a) {p 2, q 1}, (m1 b) {p 0.5}, (m2 a) feasible {p 9}, (m2 b) {p 1}
        mode_ix, feas, use, vstart, vval = [0, 0, 1, 1, 0], [0, 0, 1, 0, 0], [1, 1, 1, 1, 0], [0, 2, 3, 4, 5], [2.0, 1.0, 0.5, 9.0, 1.0]
        fv = lambda n: call("looprules::loop_family_violation", Buf(mode_ix), Buf(feas), Buf(use), 0, Buf(vstart), Buf(vval), Buf([0] * 5), n, 3)[0]
        self.assertEqual(fv(4), 0.5, "m1 best is 0.5, m2 passes, m3 has no option")
        self.assertEqual(fv(0), 0.0)


if __name__ == "__main__":
    unittest.main()
