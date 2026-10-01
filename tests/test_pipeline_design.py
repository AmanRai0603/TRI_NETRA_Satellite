"""The design loop's assess and converge nodes on small made-up results, and the cache key.

assess reads the run manifests the matrix filed (here written into a temporary cache) and keeps
each option's best algorithm by its worst seed; converge turns the failures into knob changes,
and undoes or blocks a change that did not help. Copyright (c) 2026 Agastya. All rights reserved.
"""
import json
import pathlib
import shutil
import tempfile
import unittest
from unittest import mock

import _path  # puts tools/ on the import path
import pipeline_base as B
import pipeline_design as D

_ = _path  # imported for its effect: tools/ on sys.path

MODES = [{"id": "nadir_pointing", "objective": "ape_los_p9973",
          "options": [{"id": "mtq", "actuator": "mtq", "family": "mtq"},
                      {"id": "rw+mtq", "actuator": "rw", "dump": "mtq", "family": "rw"},
                      {"id": "fmr+mtq", "actuator": "fmr", "dump": "mtq", "family": "fmr"},
                      {"id": "rw+rcs", "actuator": "rw", "dump": "rcs", "family": "rw_rcs"}]}]


def manifest(**metrics):
    """A run manifest: metric id -> (value, req, pass)."""
    return {"metrics": [{"id": k, "value": v, "req": q, "pass": p} for k, (v, q, p) in metrics.items()], "algorithms": {"pointing": "pid"}}


class Assess(unittest.TestCase):
    def setUp(self):
        self.cache = pathlib.Path(tempfile.mkdtemp())
        self.p = mock.patch.object(D, "CACHE", self.cache)
        self.p.start()

    def tearDown(self):
        self.p.stop()
        shutil.rmtree(self.cache, ignore_errors=True)

    def file(self, key, man):
        (self.cache / key).mkdir()
        (self.cache / key / "manifest.json").write_text(json.dumps(man))

    def mode_test(self, keys, alg=None, option="mtq"):
        return {"mode": "nadir_pointing", "option": option, "alg": alg, "slot": "mtq_pointing", "keys": keys, "product": "P"}

    def test_every_seed_passing_is_feasible_with_the_worst_seed_as_objective(self):
        self.file("s1", manifest(ape_los_p9973=(3.0, 10.0, 1), power_mean=(0.2, 0.5, 1)))
        self.file("s2", manifest(ape_los_p9973=(7.0, 10.0, 1), power_mean=(0.1, 0.5, 1)))
        r = D.node_assess([self.mode_test(["s1", "s2"])], MODES)[("nadir_pointing", "mtq")]
        self.assertTrue(r["feasible"])
        self.assertEqual((r["objective"], r["objective_req"], r["objective_id"]), (7.0, 10.0, "ape_los_p9973"))
        self.assertEqual(r["metrics"], {"ape_los_p9973": 7.0, "power_mean": 0.2})
        self.assertEqual((r["failing"], r["violation"]), ({}, {}))
        self.assertEqual(r["algorithms"], {"pointing": "pid"})
        self.assertNotIn("variants", r, "one algorithm flown: no variant table")

    def test_a_seed_that_did_not_fly_is_not_a_pass(self):
        self.file("s1", manifest(ape_los_p9973=(3.0, 10.0, 1)))
        r = D.node_assess([self.mode_test(["s1", "never_flown"])], MODES)[("nadir_pointing", "mtq")]
        self.assertFalse(r["feasible"])
        self.assertEqual(r["failing"], {})

    def test_nothing_flown_is_not_feasible_and_has_no_objective(self):
        r = D.node_assess([self.mode_test(["a", "b"])], MODES)[("nadir_pointing", "mtq")]
        self.assertFalse(r["feasible"])
        self.assertIsNone(r["objective"])
        self.assertEqual(r["algorithms"], {})

    def test_a_failure_is_classed_and_its_violation_is_the_worst_seed_over_the_requirement(self):
        self.file("s1", manifest(ape_los_p9973=(15.0, 10.0, 0), power_mean=(0.6, 0.5, 0)))
        self.file("s2", manifest(ape_los_p9973=(20.0, 10.0, 0), power_mean=(0.4, 0.5, 1)))
        r = D.node_assess([self.mode_test(["s1", "s2"])], MODES)[("nadir_pointing", "mtq")]
        self.assertFalse(r["feasible"])
        self.assertEqual(r["failing"], {"ape_los_p9973": "performance", "power_mean": "power"})
        self.assertAlmostEqual(r["violation"]["ape_los_p9973"], 1.0)
        self.assertAlmostEqual(r["violation"]["power_mean"], 0.2)

    def test_a_failure_with_no_usable_number_counts_as_a_large_violation(self):
        self.file("s1", manifest(ape_los_p9973=(None, 10.0, 0), ake_los_p9973=(float("inf"), 5.0, 0), propellant=(1.0, 0.0, 0)))
        r = D.node_assess([self.mode_test(["s1"])], MODES)[("nadir_pointing", "mtq")]
        self.assertEqual(r["violation"], {"ape_los_p9973": 10.0, "ake_los_p9973": 10.0, "propellant": 10.0})

    def test_the_best_algorithm_is_kept_and_every_variant_listed(self):
        self.file("a1", manifest(ape_los_p9973=(15.0, 10.0, 0)))
        self.file("b1", manifest(ape_los_p9973=(8.0, 10.0, 1)))
        self.file("c1", manifest(ape_los_p9973=(4.0, 10.0, 1)))
        self.file("d1", manifest(ape_los_p9973=(30.0, 10.0, 0), power_mean=(1.0, 0.5, 0)))
        tests = [self.mode_test(["a1"], "mtq_pd"), self.mode_test(["b1"], "mtq_lqr"), self.mode_test(["c1"], "mtq_smc"), self.mode_test(["d1"], "mtq_tango2013")]
        r = D.node_assess(tests, MODES)[("nadir_pointing", "mtq")]
        self.assertEqual((r["alg"], r["objective"]), ("mtq_smc", 4.0), "feasible first, then the lower objective")
        self.assertEqual([v["alg"] for v in r["variants"]], ["mtq_pd", "mtq_lqr", "mtq_smc", "mtq_tango2013"])
        self.assertEqual(r["variants"][3]["violation"], 3.0)

    def test_among_failures_the_fewest_failing_metrics_win(self):
        self.file("a1", manifest(ape_los_p9973=(11.0, 10.0, 0), power_mean=(0.6, 0.5, 0)))
        self.file("b1", manifest(ape_los_p9973=(90.0, 10.0, 0)))
        r = D.node_assess([self.mode_test(["a1"], "x"), self.mode_test(["b1"], "y")], MODES)[("nadir_pointing", "mtq")]
        self.assertEqual(r["alg"], "y")

    def test_a_metric_given_as_none_is_left_out_of_the_worst_case(self):
        self.file("s1", manifest(jitter=(None, None, None), ape_los_p9973=(2.0, 10.0, 1)))
        r = D.node_assess([self.mode_test(["s1"])], MODES)[("nadir_pointing", "mtq")]
        self.assertIsNone(r["metrics"]["jitter"])
        self.assertTrue(r["feasible"], "an unjudged metric fails nothing")


def res(option, feasible=False, failing=(), objective=5.0, req=10.0, slot="pointing", violation=None, alg=None):
    f = {m: B.cls(m) for m in failing}
    return {"mode": "nadir_pointing", "option": option, "alg": alg, "slot": slot, "feasible": feasible, "failing": f,
            "violation": violation if violation is not None else {m: 1.0 for m in failing}, "objective": objective,
            "objective_id": "ape_los_p9973", "objective_req": req}


def converge(results, knobs=None, variants=(), history=None, fine=False, sel=None):
    knobs = knobs if knobs is not None else {}
    history = history if history is not None else {}
    k, v, ch, bl = D.node_converge("ais_3u", results, knobs, set(variants), history, fine, MODES, sel)
    return k, v, ch, bl, history


class Converge(unittest.TestCase):
    def test_everything_passing_with_margin_changes_nothing(self):
        r = {("nadir_pointing", "mtq"): res("mtq", True, objective=2.0, slot="mtq_pointing"),
             ("nadir_pointing", "rw+mtq"): res("rw+mtq", True)}
        k, v, ch, bl, h = converge(r, {"scale": {"rw": 1.0}})
        self.assertEqual((k, v, ch, bl), ({"scale": {"rw": 1.0}}, set(), [], []))
        self.assertEqual(h["_last_up"], {})

    def test_the_knobs_given_are_not_changed_in_place(self):
        knobs = {"scale": {"rw": 1.0}}
        converge({("nadir_pointing", "rw+mtq"): res("rw+mtq", failing=["ape_los_p9973"])}, knobs, variants=[("nadir_pointing", "rw+mtq")])
        self.assertEqual(knobs, {"scale": {"rw": 1.0}})

    def test_a_performance_failure_first_flies_every_algorithm_of_the_slot(self):
        k, v, ch, bl, _ = converge({("nadir_pointing", "rw+mtq"): res("rw+mtq", failing=["ape_los_p9973"])})
        self.assertEqual(v, {("nadir_pointing", "rw+mtq")})
        self.assertEqual(ch, ["nadir_pointing/rw+mtq: fly every pointing algorithm"])
        self.assertEqual(k, {})

    def test_then_a_coils_only_option_is_tuned(self):
        key = ("nadir_pointing", "mtq")
        k, v, ch, bl, h = converge({key: res("mtq", failing=["ape_los_p9973"], slot="mtq_pointing")}, variants=[key])
        self.assertEqual(h["_tuned"], [["nadir_pointing", "mtq"]])
        self.assertIn("tune every mtq_pointing law's gains", ch[0])

    def test_then_more_authority_of_the_part_that_serves_it(self):
        key = ("nadir_pointing", "rw+mtq")
        k, v, ch, bl, h = converge({key: res("rw+mtq", failing=["ape_los_p9973"])}, {"scale": {"rw": 2.0}}, variants=[key])
        self.assertEqual(k["scale"]["rw"], 2.0 * B.UP)
        self.assertEqual(h["rw"], ["up"])
        self.assertEqual(list(h["_last_up"]), ["rw"])
        self.assertEqual(h["_last_up"]["rw"][0], 2.0)
        self.assertEqual(bl, [])

    def test_authority_is_never_raised_past_its_bound(self):
        key = ("nadir_pointing", "rw+mtq")
        k, v, ch, bl, _ = converge({key: res("rw+mtq", failing=["ape_los_p9973"])}, {"scale": {"rw": B.SCALE_MAX * 0.9}}, variants=[key])
        self.assertEqual(k["scale"]["rw"], B.SCALE_MAX)
        k, v, ch, bl, _ = converge({key: res("rw+mtq", failing=["ape_los_p9973"])}, {"scale": {"rw": B.SCALE_MAX}}, variants=[key])
        self.assertEqual(k["scale"]["rw"], B.SCALE_MAX)
        self.assertEqual(ch, [])
        self.assertEqual(bl, [f"rw: authority at its bound (x{B.SCALE_MAX:g})"])

    def test_up_after_down_is_a_conflict_not_an_oscillation(self):
        key = ("nadir_pointing", "rw+mtq")
        k, v, ch, bl, _ = converge({key: res("rw+mtq", failing=["ape_los_p9973"])}, {"scale": {"rw": 1.0}}, variants=[key], history={"rw": ["down"]})
        self.assertEqual(k["scale"]["rw"], 1.0)
        self.assertIn("conflict", bl[0])

    def test_performance_and_power_together_are_blocked(self):
        key = ("nadir_pointing", "rw+mtq")
        k, v, ch, bl, _ = converge({key: res("rw+mtq", failing=["ape_los_p9973", "power_mean"])}, variants=[key])
        self.assertTrue(any("performance and power both fail" in b for b in bl))
        self.assertNotIn("rw", k.get("scale", {}))

    def test_a_coils_power_failure_lowers_the_coils_authority(self):
        key = ("nadir_pointing", "mtq")
        k, v, ch, bl, h = converge({key: res("mtq", failing=["power_mean"], slot="mtq_pointing")}, {"scale": {"mtqp": 1.0}})
        self.assertEqual(k["scale"]["mtqp"], B.DOWN)
        self.assertEqual(h["mtqp"], ["down"])
        self.assertEqual(h["_last_up"], {}, "a lowering is not an authority increase to check")

    def test_a_wheel_power_failure_is_blocked_at_the_standby_floor(self):
        k, v, ch, bl, _ = converge({("nadir_pointing", "rw+mtq"): res("rw+mtq", failing=["power_mean"])})
        self.assertEqual(bl, ["nadir_pointing/rw+mtq: power fails at the sized authority (rw); the part's standby power is the floor"])

    def test_knowledge_fits_the_star_tracker_once(self):
        r = {("nadir_pointing", "rw+mtq"): res("rw+mtq", failing=["ake_los_p9973"])}
        k, v, ch, bl, _ = converge(r)
        self.assertTrue(k["star_tracker"])
        k, v, ch, bl, _ = converge(r, {"star_tracker": True})
        self.assertIn("knowledge fails with the star tracker fitted", bl[0])
        k, v, ch, bl, _ = converge(r, fine=True)
        self.assertNotIn("star_tracker", k, "a fine case already flies one")

    def test_propellant_asks_for_more_thruster_authority(self):
        k, v, ch, bl, _ = converge({("nadir_pointing", "rw+rcs"): res("rw+rcs", failing=["propellant"])})
        self.assertEqual(k["scale"]["rcs"], B.UP)

    def test_a_thin_margin_flies_every_algorithm_then_tunes(self):
        key = ("nadir_pointing", "mtq")
        r = {key: res("mtq", True, objective=0.8 * 10.0, slot="mtq_pointing")}
        k, v, ch, bl, h = converge(r)
        self.assertEqual(v, {key})
        self.assertIn("thin margin", ch[0])
        k, v, ch, bl, h = converge(r, variants=[key], history=h)
        self.assertEqual(h["_tuned"], [["nadir_pointing", "mtq"]])
        k, v, ch, bl, h = converge(r, variants=[key], history=h)
        self.assertEqual(ch, [], "once tuned, a thin margin asks nothing more")
        k, v, ch, bl, h = converge({key: res("mtq", True, objective=0.4 * 10.0, slot="mtq_pointing")})
        self.assertEqual(ch, [], "inside the margin: nothing to do")

    def test_more_authority_that_did_not_help_is_undone_and_frozen(self):
        key = ("nadir_pointing", "rw+mtq")
        before = {key: res("rw+mtq", failing=["ape_los_p9973"], violation={"ape_los_p9973": 1.0})}
        now = {key: res("rw+mtq", failing=["ape_los_p9973"], violation={"ape_los_p9973": 0.99})}
        h = {"_last_up": {"rw": (1.0, before)}, "rw": ["up"]}
        k, v, ch, bl, h = converge(now, {"scale": {"rw": 1.5}}, variants=[key], history=h)
        self.assertEqual(k["scale"]["rw"], 1.0)
        self.assertIn("rw: authority back to x1 (no improvement)", ch)
        self.assertIn("rw", h["_frozen"])
        self.assertTrue(any(b.startswith("rw: more authority did not reduce") for b in bl), "and not raised again")

    def test_more_authority_that_helped_is_kept(self):
        key = ("nadir_pointing", "rw+mtq")
        before = {key: res("rw+mtq", failing=["ape_los_p9973"], violation={"ape_los_p9973": 1.0})}
        now = {key: res("rw+mtq", failing=["ape_los_p9973"], violation={"ape_los_p9973": 0.5})}
        h = {"_last_up": {"rw": (1.0, before)}, "rw": ["up"]}
        k, v, ch, bl, h = converge(now, {"scale": {"rw": 1.5}}, variants=[key], history=h)
        self.assertEqual(k["scale"]["rw"], 1.5 * B.UP)
        self.assertNotIn("rw", h["_frozen"])

    def test_rate_stability_in_a_fine_case_asks_a_better_gyro_and_undoes_it_if_useless(self):
        key = ("nadir_pointing", "rw+mtq")
        r = {key: res("rw+mtq", failing=["rate_stability_p9973"], violation={"rate_stability_p9973": 2.0})}
        k, v, ch, bl, h = converge(r, {"gyro_grade": 1.0}, variants=[key], fine=True)
        self.assertEqual(k["gyro_grade"], 0.3)
        self.assertEqual(h["_last_gyro"], (1.0, 2.0))
        k, v, ch, bl, h = converge(r, k, variants=[key], fine=True, history=h)
        self.assertEqual(k["gyro_grade"], 1.0, "no improvement: back")
        self.assertTrue(h["_closed_gyro_grade"])

    def test_the_gyro_is_not_improved_past_its_bound(self):
        key = ("nadir_pointing", "rw+mtq")
        r = {key: res("rw+mtq", failing=["rate_stability_p9973"])}
        k, v, ch, bl, h = converge(r, {"gyro_grade": B.GYRO_MIN}, variants=[key], fine=True)
        self.assertEqual(k["gyro_grade"], B.GYRO_MIN)
        self.assertEqual(k["scale"]["rw"], B.UP, "authority is the next lever")

    def test_a_fluid_loop_power_failure_raises_the_pump_lambda_to_its_bound(self):
        key = ("nadir_pointing", "fmr+mtq")
        r = {key: res("fmr+mtq", failing=["power_mean"])}
        k, v, ch, bl, h = converge(r, {"fmr_lambda": 0.1})
        self.assertAlmostEqual(k["fmr_lambda"], 0.3)
        self.assertTrue(h["_lam_up"])
        k, v, ch, bl, h = converge(r, {"fmr_lambda": B.LAMBDA_MAX})
        self.assertIn("(bound)", bl[0])

    def test_a_mass_gap_of_the_closest_family_takes_one_star_tracker_head(self):
        key = ("nadir_pointing", "rw+mtq")
        r = {key: res("rw+mtq", True)}
        sel = {"families": {"rw": {"role": "solution", "gaps": ["budget: mass_kg 1.7 > 1.6"], "simplicity": 2},
                            "mtq": {"role": "benchmark", "gaps": [], "simplicity": 1}}}
        with mock.patch.object(D, "FAMILIES", [{"id": "rw", "actuators": ["rw", "mtq"]}]):
            k, v, ch, bl, h = converge(r, {}, fine=True, sel=sel)
        self.assertEqual(k["st_heads"], 1)
        self.assertEqual(h["_last_mass"][0], "st_heads")
        # the next iteration finds the option broken: the lever is undone and closed
        broken = {key: res("rw+mtq", failing=["ake_los_p9973"])}
        with mock.patch.object(D, "FAMILIES", [{"id": "rw", "actuators": ["rw", "mtq"]}]):
            k, v, ch, bl, h = converge(broken, k, fine=True, history=h, sel=sel)
        self.assertEqual(k["st_heads"], 2)
        self.assertTrue(h["_closed_st_heads"])
        self.assertTrue(any(c.startswith("mass lever st_heads undone") for c in ch))


class NodeKey(unittest.TestCase):
    def setUp(self):
        self.d = pathlib.Path(tempfile.mkdtemp())
        sized = self.d / "sized"
        (sized / "products").mkdir(parents=True)
        (sized / "parts").mkdir()
        (sized / "products" / "SZ-1.json").write_text(json.dumps({"id": "SZ-1", "knobs": {"a": 1}, "fill": [{"part": "rw1"}, {"part": "absent"}]}))
        (sized / "parts" / "rw1.json").write_text(json.dumps({"mass": 0.1}))
        (self.d / "scen.json").write_text(json.dumps({"product": "SZ-1", "case": "ais_3u", "id": "s"}))
        self.sized, self.disp = sized, {"scenario": str(self.d / "scen.json")}

    def tearDown(self):
        shutil.rmtree(self.d, ignore_errors=True)

    def test_the_key_is_stable(self):
        self.assertEqual(D.node_key(self.disp, self.sized, "b1"), D.node_key(self.disp, self.sized, "b1"))

    def test_the_build_the_purpose_and_a_part_change_the_key(self):
        k = D.node_key(self.disp, self.sized, "b1")
        self.assertNotEqual(k, D.node_key(self.disp, self.sized, "b2"))
        self.assertNotEqual(k, D.node_key(self.disp, self.sized, "b1", "soft_oils"))
        (self.sized / "parts" / "rw1.json").write_text(json.dumps({"mass": 0.2}))
        self.assertNotEqual(k, D.node_key(self.disp, self.sized, "b1"))

    def test_the_product_knobs_do_not_change_the_key(self):
        k = D.node_key(self.disp, self.sized, "b1")
        (self.sized / "products" / "SZ-1.json").write_text(json.dumps({"id": "SZ-1", "knobs": {"a": 2}, "fill": [{"part": "rw1"}, {"part": "absent"}]}))
        self.assertEqual(k, D.node_key(self.disp, self.sized, "b1"))

    def test_the_product_blob_skips_a_part_not_sized(self):
        prod, parts = D.product_blob(self.sized, "SZ-1")
        self.assertEqual(list(parts), ["rw1"])
        self.assertNotIn("knobs", prod)


if __name__ == "__main__":
    unittest.main()
