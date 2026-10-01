"""The design loop's select and robustness nodes, and the design ledger, on small made-up inputs.

select ranks the families by the registry's rule (feasible first, then mass, power, volume,
simplicity) and says when the pick is only the closest; robust moves the selected family's
lever for a Monte Carlo failure; the ledger writes what was decided. Copyright (c) 2026 Agastya.
"""
import contextlib
import io
import pathlib
import shutil
import tempfile
import unittest
from unittest import mock

import _path  # puts tools/ on the import path
import pipeline_base as B
import pipeline_ledger as L
import pipeline_verify as V

_ = _path  # imported for its effect: tools/ on sys.path

MODES = [{"id": "detumble", "objective": "detumble_time", "options": [{"id": "mtq", "actuator": "mtq"}]},
         {"id": "nadir_pointing", "objective": "ape_los_p9973",
          "options": [{"id": "mtq", "actuator": "mtq"}, {"id": "rw+mtq", "actuator": "rw", "dump": "mtq"}]}]

FAMILIES = [{"id": "mtq", "actuators": ["mtq"], "role": "solution", "label": "coils only", "simplicity": 1},
            {"id": "rw", "actuators": ["rw", "mtq"], "role": "solution", "label": "wheels", "simplicity": 2},
            {"id": "cmg", "actuators": ["cmg", "mtq"], "role": "benchmark", "label": "CMGs", "simplicity": 3}]


def r(mode, option, feasible, failing=(), objective=1.0, alg=None):
    return {"mode": mode, "option": option, "alg": alg, "feasible": feasible, "failing": {m: B.cls(m) for m in failing},
            "objective": objective, "objective_id": "obj", "algorithms": {"x": "y"}, "metrics": {"obj": objective}}


def sizing(**mass):
    fam = {f: {"mass_kg": m, "power_W": 0.3, "volume_L": 0.5, "product": f"SZ-c-{f}"} for f, m in mass.items()}
    return {"demand": {"req": {"mass": 1.6, "vol": 1.0}}, "families": fam}


class Select(unittest.TestCase):
    def test_the_lightest_feasible_solution_is_selected_and_the_benchmark_ranked_apart(self):
        res = {("detumble", "mtq"): r("detumble", "mtq", True),
               ("nadir_pointing", "mtq"): r("nadir_pointing", "mtq", True, objective=8.0),
               ("nadir_pointing", "rw+mtq"): r("nadir_pointing", "rw+mtq", True, objective=0.1)}
        s = V.node_select("c", res, sizing(mtq=0.9, rw=1.2, cmg=1.5), MODES, FAMILIES)
        self.assertEqual((s["selected"], s["status"]), ("mtq", "feasible"))
        self.assertEqual((s["benchmark"], s["benchmark_status"]), ("cmg", "feasible"))
        self.assertEqual((s["families"]["mtq"]["rank"], s["families"]["rw"]["rank"], s["families"]["cmg"]["rank"]), (1, 2, 1))
        self.assertEqual(s["families"]["rw"]["modes"]["nadir_pointing"]["option"], "rw+mtq", "the family's best usable option")
        self.assertEqual(s["families"]["mtq"]["modes"]["nadir_pointing"]["option"], "mtq", "a coils-only family cannot use the wheels")
        self.assertIn("least mass_kg", s["rule"])

    def test_a_budget_over_the_requirement_is_a_gap(self):
        res = {("detumble", "mtq"): r("detumble", "mtq", True), ("nadir_pointing", "mtq"): r("nadir_pointing", "mtq", True)}
        s = V.node_select("c", res, sizing(mtq=1.7, rw=1.2, cmg=1.5), MODES, FAMILIES)
        self.assertEqual(s["families"]["mtq"]["gaps"], ["budget: mass_kg 1.7 > 1.6"])
        self.assertFalse(s["families"]["mtq"]["feasible"])

    def test_a_mode_with_no_usable_option_is_a_gap(self):
        res = {("detumble", "mtq"): r("detumble", "mtq", True)}
        s = V.node_select("c", res, sizing(mtq=0.9, rw=1.2, cmg=1.5), MODES, FAMILIES)
        self.assertEqual(s["families"]["mtq"]["gaps"], ["nadir_pointing: no option"])
        self.assertIsNone(s["families"]["mtq"]["modes"]["nadir_pointing"])

    def test_with_no_feasible_solution_the_closest_is_named_not_called_feasible(self):
        res = {("detumble", "mtq"): r("detumble", "mtq", True),
               ("nadir_pointing", "mtq"): r("nadir_pointing", "mtq", False, ["ape_los_p9973", "power_mean"]),
               ("nadir_pointing", "rw+mtq"): r("nadir_pointing", "rw+mtq", False, ["power_mean"])}
        s = V.node_select("c", res, sizing(mtq=0.9, rw=1.2, cmg=1.5), MODES, FAMILIES)
        self.assertEqual(s["status"], "closest (not feasible)")
        self.assertEqual(s["selected"], "mtq", "equal gap count: the lighter")
        self.assertEqual(s["families"]["rw"]["gaps"], ["nadir_pointing: power_mean (power)"])
        self.assertNotIn("rank", s["families"]["mtq"], "an infeasible family is not ranked")

    def test_simplicity_breaks_an_equal_budget(self):
        res = {("detumble", "mtq"): r("detumble", "mtq", True), ("nadir_pointing", "mtq"): r("nadir_pointing", "mtq", True),
               ("nadir_pointing", "rw+mtq"): r("nadir_pointing", "rw+mtq", True)}
        fams = [dict(FAMILIES[1], simplicity=1), dict(FAMILIES[0], simplicity=5), FAMILIES[2]]
        s = V.node_select("c", res, sizing(mtq=1.0, rw=1.0, cmg=1.5), MODES, fams)
        self.assertEqual(s["selected"], "rw")

    def test_no_benchmark_family_gives_no_benchmark(self):
        res = {("detumble", "mtq"): r("detumble", "mtq", True), ("nadir_pointing", "mtq"): r("nadir_pointing", "mtq", True)}
        s = V.node_select("c", res, sizing(mtq=0.9), MODES, FAMILIES[:1])
        self.assertIsNone(s["benchmark"])


class Robust(unittest.TestCase):
    def robust(self, family, fails, knobs=None, cls_="coarse"):
        knobs = knobs if knobs is not None else {}
        h = {}
        k, ch, bl = V.node_robust({"selected": family, "class": cls_}, [{"id": f} for f in fails], knobs, h)
        return k, ch, bl, h

    def test_nothing_failing_changes_nothing(self):
        self.assertEqual(self.robust("mtq", [], {"a": 1})[:3], ({"a": 1}, [], []))

    def test_a_coils_performance_failure_raises_the_coils_and_closes_the_way_back(self):
        knobs = {"scale": {"mtqp": 2.0}}
        k, ch, bl, h = self.robust("mtq", ["ape_los_p9973"], knobs)
        self.assertEqual(k["scale"]["mtqp"], 2.0 * B.UP)
        self.assertTrue(h["_closed_scale.mtqp"])
        self.assertEqual(knobs, {"scale": {"mtqp": 2.0}}, "the knobs given are not changed in place")

    def test_a_family_without_an_authority_lever_is_blocked(self):
        k, ch, bl, h = self.robust("rw", ["ape_los_p9973"])
        self.assertEqual(ch, [])
        self.assertIn("no authority is left", bl[0])
        k, ch, bl, h = self.robust("mtq", ["ape_los_p9973"], {"scale": {"mtqp": B.SCALE_MAX}})
        self.assertIn("no authority is left", bl[0])

    def test_a_fluid_loop_power_failure_takes_more_copper(self):
        k, ch, bl, h = self.robust("rw_fmr", ["power_mean"], {"fmr_lambda": 0.1})
        self.assertAlmostEqual(k["fmr_lambda"], 0.3)
        self.assertTrue(h["_lam_up"] and h["_closed_fmr_lambda"])
        k, ch, bl, h = self.robust("mtq", ["power_mean"])
        self.assertIn("no power lever is left", bl[0])

    def test_a_knowledge_failure_fits_the_star_tracker_unless_fine_or_fitted(self):
        self.assertTrue(self.robust("mtq", ["ake_los_p9973"])[0]["star_tracker"])
        self.assertIn("with the star tracker fitted", self.robust("mtq", ["ake_los_p9973"], cls_="fine")[2][0])
        self.assertIn("with the star tracker fitted", self.robust("mtq", ["ake_los_p9973"], {"star_tracker": True})[2][0])


def variant(alg, feasible, objective, failing=()):
    return {"alg": alg, "feasible": feasible, "failing": {m: "performance" for m in failing}, "objective": objective, "violation": 0.0}


class Ledger(unittest.TestCase):
    LOG = [{"iteration": 1, "knobs": {"scale": {"mtqp": 1.5}, "star_tracker": True, "fmr_lambda": 0.3, "gyro_grade": 0.3},
            "feasible_options": 2, "options": 3, "selected": "mtq", "status": "feasible", "changes": ["mtqp: authority x1 -> x1.5"], "blocked": ["b"],
            "matrix": [{"mode": "nadir_pointing", "option": "mtq", "alg": "mtq_pd@mtq_gain_p=1,mtq_gain_d=4", "objective_id": "ape_los_p9973",
                        "feasible": True, "objective": 4.0, "failing": {},
                        "variants": [variant("mtq_pd", False, 12.0, ["ape_los_p9973"]), variant("mtq_pd@mtq_gain_p=1,mtq_gain_d=4", True, 4.0),
                                     variant("mtq_lovera2004", True, 6.0), variant("mtq_lovera2004@mtq_gain_p=4,mtq_gain_d=4", True, 2.0),
                                     variant(None, True, 5.0)]},
                       {"mode": "nadir_pointing", "option": "rw+mtq", "alg": None, "objective_id": "ape_los_p9973", "feasible": True, "objective": 0.1,
                        "failing": {}, "variants": [variant("pid", True, 0.1), variant("lqr", True, 0.2)]},
                       {"mode": "detumble", "option": "mtq", "alg": None, "objective_id": "detumble_time", "feasible": True, "objective": 100.0,
                        "failing": {}}]}]

    def test_the_literature_table_is_each_laws_best_gains_for_the_coils_only_option(self):
        rows = L.literature_table(self.LOG)
        self.assertEqual([x["law"] for x in rows], ["mtq_lovera2004", "mtq_pd", "default"], "best first; wheel options left out")
        lov = rows[0]
        self.assertEqual((lov["gains"], lov["objective"], lov["paper"]), ("mtq_gain_p 4, mtq_gain_d 4", 2.0, "P1 Lovera & Astolfi 2004"))
        self.assertEqual((rows[1]["gains"], rows[1]["feasible"]), ("mtq_gain_p 1, mtq_gain_d 4", True), "a tuned feasible point beats the nominal failure")
        self.assertEqual((rows[2]["gains"], rows[2]["paper"]), ("nominal", "—"))

    def test_the_literature_table_reads_only_the_last_iteration(self):
        self.assertEqual(L.literature_table(self.LOG + [{"matrix": []}]), [])

    def test_the_ledger_is_written_with_its_decisions(self):
        d = pathlib.Path(tempfile.mkdtemp())
        try:
            fam = lambda role, feas, mass, gaps: {"role": role, "feasible": feas, "label": "L", "gaps": gaps, "rank": 1,
                                                  "budget": {"mass_kg": mass, "power_W": 0.3, "volume_L": 0.5},
                                                  "modes": {"nadir_pointing": {"option": "mtq", "feasible": feas, "objective": 4.0, "objective_id": "ape",
                                                                               "algorithms": {"mtq_pointing": "mtq_pd"}, "failing": {}},
                                                            "detumble": None}}
            sel = {"selected": "mtq", "status": "feasible", "converged": True, "iterations": 1, "class": "coarse", "sensors": ["mag", "sun"],
                   "rule": "R", "benchmark": "cmg", "benchmark_status": "feasible",
                   "families": {"mtq": fam("solution", True, 0.9, []), "cmg": fam("benchmark", False, 1.9, ["budget: mass_kg 1.9 > 1.6"])},
                   "robustness": [{"after_iteration": 1, "family": "mtq", "mc_failing": {"ape": 0.9}, "changes": ["c1"], "blocked": []}]}
            sizing = {"families": {"mtq": {"items": [{"slot": "coils", "part": "mtq1", "n": 3}]},
                                   "cmg": {"items": [{"slot": "cmg", "part": "cmg1", "n": 4}]}}}
            disp = {"dir": "dist/dispatch/c/mtq/converged", "check": {"c_equals_rust_bitwise": True}}
            with mock.patch.object(L, "OUT", d), mock.patch.object(L, "PIPE", d / "pipe"), \
                    contextlib.redirect_stdout(io.StringIO()):
                L.ledger("c", sel, self.LOG, disp, None, None, sizing)
            text = (d / "DESIGN_c.md").read_text()
        finally:
            shutil.rmtree(d, ignore_errors=True)
        self.assertIn("**Selected: `mtq` — L — feasible**, converged after 1 iteration(s)", text)
        self.assertIn("| cmg | benchmark | 1 | no | 1.900 |", text)
        self.assertIn("cmg1 x4", text)
        self.assertIn("coils only", text)
        self.assertIn("mtqp x1.5, star_tracker, pump lambda 0.3 kg/W, gyro noise x0.3", text)
        self.assertIn("the Monte Carlo of `mtq` failed ape (90 % of runs pass). c1", text)
        self.assertIn("- b", text, "why the loop stopped")
        self.assertIn("| nadir_pointing | mtq_lovera2004 | P1 Lovera & Astolfi 2004 |", text)
        self.assertIn("C = Rust flight software bitwise on the engine: True.", text)
        self.assertNotIn("Monte Carlo of the dispatched mission", text, "no Monte Carlo given, none reported")


if __name__ == "__main__":
    unittest.main()
