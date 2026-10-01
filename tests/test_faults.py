"""Node faults (B2.6) and how select counts it, on small made-up inputs (no engine).

The fault set follows the hardware a product carries; the pass rule asks that every requirement
met without the fault still holds under it, on every seed; select counts a fault the family does
not survive as a gap (fault_policy "gap") or ranks by the failed-fault count ("rank").
Copyright (c) 2026 Agastya.
"""
import json
import unittest
from unittest import mock

import _path  # puts tools/ on the import path
import pipeline_verify as V

_ = _path  # imported for its effect: tools/ on sys.path

COILS = {"slot": "coils", "axes_body": [[1, 0, 0], [0, 1, 0], [0, 0, 1]]}
RINGS = [{"slot": "rings", "axes_body": [a]} for a in ([1, 0, 0], [0, 1, 0], [0, 0, 1])]
CMG = {"slot": "cmg", "spin_axes_body": [[1, 0, 0]] * 4, "gimbal_axes_body": [[0, 0, 1]] * 4}
SENSORS = [{"slot": "gyro"}, {"slot": "gnss"}, {"slot": "magnetometer"}, {"slot": "star_tracker", "boresights_body": [[0, 0, 1], [0, 1, 0]]}]
SET = [{"kind": "coil_fail", "needs": "coils", "index": 1},
       {"kind": "rotor_fail", "needs": "rotors", "index": 1},
       {"kind": "st_head_fail", "needs": "st_heads", "min_units": 2, "index": 2},
       {"kind": "gyro_bias_step", "needs": "gyros", "value": [0.001745, 0.0, 0.0]},
       {"kind": "gps_outage", "needs": "gnss", "duration_s": 1800.0},
       {"kind": "rcs_valve_fail", "needs": "rcs_couples", "index": 1}]


def m(ape, ake=1, power=1):
    return [{"id": "ape_los_p9973", "pass": ape}, {"id": "ake_los_p9973", "pass": ake}, {"id": "power_mean", "pass": power},
            {"id": "power_peak", "pass": None}]


class Units(unittest.TestCase):
    def test_units_are_counted_as_the_engine_counts_them(self):
        u = V.units([COILS, *RINGS, CMG, {"slot": "rcs", "part": "x"}, *SENSORS])
        self.assertEqual(u, {"coils": 3, "rotors": 7, "gimbals": 4, "st_heads": 2, "gyros": 1, "gnss": 1, "magnetometers": 1, "rcs_couples": 6})
        self.assertEqual(V.units([{"slot": "star_tracker", "boresight_body": [0, 0, 1]}])["st_heads"], 1, "one boresight is one head")


class FaultSet(unittest.TestCase):
    def test_the_set_follows_the_hardware_and_says_why_it_skips(self):
        fly, skipped = V.fault_set([COILS, *RINGS, {"slot": "gyro"}, {"slot": "gnss"}, {"slot": "star_tracker", "boresights_body": [[0, 0, 1]]}], SET, 1000)
        self.assertEqual([x["kind"] for x in fly], ["coil_fail", "rotor_fail", "gyro_bias_step", "gps_outage"])
        self.assertEqual({x["kind"]: x["skipped"] for x in skipped},
                         {"st_head_fail": "1 st_heads, needs 2", "rcs_valve_fail": "0 rcs_couples, needs 1"})
        self.assertTrue(all(x["t_s"] == 1000.0 for x in fly))
        gps = next(x for x in fly if x["kind"] == "gps_outage")
        self.assertEqual((gps["end_s"], "needs" in gps, "duration_s" in gps), (2800.0, False, False), "an outage ends duration_s later")
        self.assertEqual(next(x for x in fly if x["kind"] == "gyro_bias_step")["value"], [0.001745, 0.0, 0.0])

    def test_two_star_tracker_heads_fly_the_head_fault_and_rcs_the_valve_fault(self):
        fly, _ = V.fault_set([COILS, {"slot": "rcs"}, *SENSORS], SET, 0)
        self.assertIn("st_head_fail", [x["kind"] for x in fly])
        self.assertIn("rcs_valve_fail", [x["kind"] for x in fly])

    def test_what_cannot_be_flown_is_refused(self):
        for bad, said in (({"kind": "wing_fail", "needs": "coils"}, "not a fault the engine injects"),
                          ({"kind": "coil_fail", "needs": "wings"}, "not a unit"),
                          ({"kind": "coil_fail", "needs": "coils", "index": 4}, "coils 1 to 3")):
            with self.assertRaises(SystemExit) as e:
                V.fault_set([COILS], [bad], 0)
            self.assertIn(said, str(e.exception))


class Verdict(unittest.TestCase):
    def test_a_fault_that_breaks_nothing_passes(self):
        self.assertEqual(V.fault_verdict({1: m(1)}, {1: m(1)}), (True, [], []))

    def test_a_requirement_met_without_the_fault_and_missed_with_it_fails(self):
        self.assertEqual(V.fault_verdict({1: m(1)}, {1: m(0, ake=0)}), (False, ["ape_los_p9973", "ake_los_p9973"], []))

    def test_a_miss_the_fault_free_run_has_too_is_recorded_not_counted(self):
        self.assertEqual(V.fault_verdict({1: m(0)}, {1: m(0)}), (True, [], ["ape_los_p9973"]))

    def test_every_seed_must_hold(self):
        ok, failing, _ = V.fault_verdict({1: m(1), 2: m(1)}, {1: m(1), 2: m(1, power=0)})
        self.assertEqual((ok, failing), (False, ["power_mean"]))

    def test_a_metric_not_computed_under_the_fault_does_not_hold(self):
        faulted = [x for x in m(1) if x["id"] != "ake_los_p9973"]
        self.assertEqual(V.fault_verdict({1: m(1)}, {1: faulted})[1], ["ake_los_p9973"])

    def test_a_run_that_does_not_fly_fails(self):
        self.assertEqual(V.fault_verdict({1: m(1)}, {1: None})[:2], (False, ["did not fly"]))
        self.assertEqual(V.fault_verdict({1: None}, {1: m(1)})[:2], (False, ["fault-free run did not fly"]))

    def test_one_gap_per_fault_not_survived(self):
        rec = {"faults": [{"kind": "rotor_fail", "flown": True, "pass": False, "failing": ["ape_los_p9973", "ake_los_p9973"]},
                          {"kind": "coil_fail", "flown": True, "pass": True, "failing": []},
                          {"kind": "rcs_valve_fail", "flown": False, "skipped": "0 rcs_couples, needs 1"}]}
        self.assertEqual(V.fault_gaps(rec), ["fault: rotor_fail: ape_los_p9973, ake_los_p9973"])


MODES = [{"id": "detumble", "objective": "detumble_time", "options": [{"id": "mtq", "actuator": "mtq"}]}]
FAMILIES = [{"id": "light", "actuators": ["mtq"], "role": "solution", "label": "light", "simplicity": 1},
            {"id": "heavy", "actuators": ["mtq"], "role": "solution", "label": "heavy", "simplicity": 2},
            {"id": "bench", "actuators": ["mtq"], "role": "benchmark", "label": "bench", "simplicity": 3}]
RES = {("detumble", "mtq"): {"mode": "detumble", "option": "mtq", "alg": None, "feasible": True, "failing": {}, "objective": 1.0,
                             "objective_id": "detumble_time", "algorithms": {}, "metrics": {}}}
SIZING = {"demand": {"req": {"mass": 2.0, "vol": 2.0}},
          "families": {f: {"mass_kg": w, "power_W": 0.3, "volume_L": 0.5, "product": f"SZ-c-{f}"} for f, w in (("light", 1.0), ("heavy", 1.5), ("bench", 1.2))}}


def campaign(light=(), heavy=()):
    return {"families": {"light": {"gaps": [f"fault: {k}: ape_los_p9973" for k in light]},
                         "heavy": {"gaps": [f"fault: {k}: ape_los_p9973" for k in heavy]}}}


def with_policy(policy):
    return mock.patch.dict(V.P("select"), {"fault_policy": policy})


class SelectCountsFaults(unittest.TestCase):
    def test_before_the_campaign_nothing_is_counted(self):
        s = V.node_select("c", RES, SIZING, MODES, FAMILIES)
        self.assertEqual((s["selected"], s["fault_policy"], s["families"]["light"]["fault_gaps"]), ("light", None, []))

    def test_gap_a_fault_not_survived_makes_the_family_infeasible(self):
        with with_policy("gap"):
            s = V.node_select("c", RES, SIZING, MODES, FAMILIES, campaign(light=["rotor_fail"]))
        L = s["families"]["light"]
        self.assertEqual((L["feasible"], L["gaps"], L["fault_gaps"]), (False, ["fault: rotor_fail: ape_los_p9973"], ["fault: rotor_fail: ape_los_p9973"]))
        self.assertEqual((s["selected"], s["status"], s["fault_policy"]), ("heavy", "feasible", "gap"))
        self.assertIn("a single fault the family does not survive is a gap", s["rule"])

    def test_gap_with_every_solution_failing_a_fault_the_closest_is_named(self):
        with with_policy("gap"):
            s = V.node_select("c", RES, SIZING, MODES, FAMILIES, campaign(light=["rotor_fail", "coil_fail"], heavy=["coil_fail"]))
        self.assertEqual((s["selected"], s["status"]), ("heavy", "closest (not feasible)"), "fewer gaps first")

    def test_rank_a_fault_gap_leaves_feasibility_and_ranks_the_family_behind(self):
        with with_policy("rank"):
            s = V.node_select("c", RES, SIZING, MODES, FAMILIES, campaign(light=["rotor_fail"]))
        L = s["families"]["light"]
        self.assertEqual((L["feasible"], L["gaps"], L["fault_gaps"]), (True, [], ["fault: rotor_fail: ape_los_p9973"]))
        self.assertEqual((s["selected"], s["status"]), ("heavy", "feasible"), "fewest failed faults before least mass")
        self.assertEqual((s["families"]["heavy"]["rank"], L["rank"]), (1, 2))
        with with_policy("rank"):
            s = V.node_select("c", RES, SIZING, MODES, FAMILIES, campaign(light=["rotor_fail"], heavy=["coil_fail"]))
        self.assertEqual(s["selected"], "light", "an equal fault count: least mass again")

    def test_the_benchmarks_are_not_flown_and_carry_no_fault_gap(self):
        with with_policy("gap"):
            s = V.node_select("c", RES, SIZING, MODES, FAMILIES, campaign())
        self.assertEqual((s["benchmark"], s["families"]["bench"]["fault_gaps"]), ("bench", []))

    def test_a_solution_family_missing_from_the_campaign_is_refused(self):
        fl = campaign()
        del fl["families"]["heavy"]
        with self.assertRaises(SystemExit) as e:
            V.node_select("c", RES, SIZING, MODES, FAMILIES, fl)
        self.assertIn("no record of this family", str(e.exception))

    def test_an_unknown_policy_is_refused(self):
        with with_policy("maybe"), self.assertRaises(SystemExit):
            V.node_select("c", RES, SIZING, MODES, FAMILIES, campaign())

    def test_a_stored_selection_counted_again_drops_its_old_fault_gaps(self):
        with with_policy("gap"):
            s = V.node_select("c", RES, SIZING, MODES, FAMILIES, campaign(light=["rotor_fail"]))
            fams = json.loads(json.dumps(s["families"]))
            again = V.select_pick("c", fams, campaign())
        self.assertEqual((again["selected"], again["families"]["light"]["gaps"]), ("light", []))

    def test_the_registry_states_the_policy_the_set_and_the_rule(self):
        self.assertIn(V.P("select")["fault_policy"], ("gap", "rank"))
        kinds = [x["kind"] for x in V.P("faults")["set"]]
        self.assertTrue(set(kinds) <= set(V.FAULT_KINDS) and len(kinds) == len(set(kinds)))
        self.assertTrue(all(x["needs"] in V.units([]) for x in V.P("faults")["set"]))


if __name__ == "__main__":
    unittest.main()
