"""The engine orchestration's pure parts: a campaign's dispersed draws, the campaign statistics,
a mode test's scenario, the orbit period, the parity line, and the Floquet tool's attitude maths.
Nothing here runs the engine. Copyright (c) 2026 Agastya. All rights reserved."""
import json
import math
import subprocess
import unittest
from unittest import mock

import _path  # puts tools/ on the import path
import common
import engine_campaigns as C
import engine_runs
import engine_solutions as S
import floquet as F
import numpy as np
import rescore
import run_matrix

_ = _path  # imported for its effect: tools/ on sys.path

ALT = common.case_values("ais_3u")["orbit.alt"]
PERIOD = 2 * math.pi * math.sqrt((6378137 + ALT * 1e3) ** 3 / 3.986004418e14)


def sets(lst):
    return {s.split("=", 1)[0]: json.loads(s.split("=", 1)[1]) for s in lst}


class Draws(unittest.TestCase):
    MC = {"case": "ais_3u", "seed": 7, "type": "mc", "duration_s": 600.0,
          "dispersions": [{"kind": "solar_flux", "lo": 70.0, "hi": 220.0}, {"kind": "kp", "lo": 0.0, "hi": 6.0},
                          {"kind": "inertia", "frac": 0.1}, {"kind": "cm_offset", "lo": 0.5, "hi": 1.5}, {"kind": "arg_lat_deg"}]}

    def test_a_run_draws_the_same_every_time_and_another_run_differently(self):
        self.assertEqual(C.draw(self.MC, 3), C.draw(self.MC, 3))
        self.assertNotEqual(C.draw(self.MC, 3)[1], C.draw(self.MC, 4)[1])

    def test_every_draw_is_inside_its_bounds_and_named(self):
        for k in range(1, 30):
            s, d = C.draw(self.MC, k)
            o = sets(s)
            self.assertEqual(o["engine.duration_s"], 600.0)
            self.assertTrue(70.0 <= o["engine.f107"] <= 220.0)
            self.assertEqual(o["engine.f107"], o["engine.f107a"])
            self.assertTrue(0.0 <= o["engine.kp"] <= 6.0)
            self.assertEqual(o["engine.ap"], C.kp2ap(o["engine.kp"]), "ap follows the drawn Kp")
            self.assertTrue(all(0.9 <= f <= 1.1 for f in o["engine.inertia_scale"]))
            self.assertTrue(0 <= o["initial.arg_lat_deg"] <= 360)
            m = math.sqrt(sum(x * x for x in o["engine.cm_offset_m"]))
            self.assertAlmostEqual(m * 1000, d["cm_offset_mm"], places=9)
            self.assertEqual(set(d), {"F107", "Kp", "inertia_scale_x", "inertia_scale_y", "inertia_scale_z", "cm_offset_mm", "arg_lat_deg"})

    def test_an_edge_campaign_puts_one_dispersion_at_a_time_on_its_bounds_then_all_adverse(self):
        E = {"case": "ais_3u", "seed": 1, "type": "edge",
             "dispersions": [{"kind": "solar_flux", "lo": 70.0, "hi": 220.0}, {"kind": "accommodation", "lo": 0.7, "hi": 1.0}]}
        want = {1: ({"engine.f107": 70.0}, 1, 0), 2: ({"engine.f107": 220.0}, 1, 1),
                3: ({"engine.accommodation": 0.7}, 2, 0), 4: ({"engine.accommodation": 1.0}, 2, 1)}
        for k, (val, which, hi) in want.items():
            s, d = C.draw(E, k)
            o = sets(s)
            for key, v in val.items():
                self.assertEqual(o[key], v, k)
            self.assertEqual(len(o), 2 if "engine.f107" in val else 1, "the other dispersion stays nominal")
            self.assertEqual((d["edge_case"], d["edge_high"]), (which, hi))
        o = sets(C.draw(E, 5)[0])
        self.assertEqual((o["engine.f107"], o["engine.accommodation"]), (220.0, 1.0))

    def test_a_single_dispersion_need_not_be_a_list(self):
        one = {"case": "ais_3u", "seed": 1, "dispersions": {"kind": "reflectivity", "lo": 0.3, "hi": 0.9}}
        s, d = C.draw(one, 1)
        self.assertEqual(list(sets(s)), ["engine.refl"])

    def test_an_unknown_dispersion_is_refused(self):
        with self.assertRaisesRegex(ValueError, "unknown dispersion gravity"):
            C.draw({"case": "ais_3u", "seed": 1, "dispersions": [{"kind": "gravity"}]}, 1)


class Stats(unittest.TestCase):
    def test_statistics_over_the_finite_values_only(self):
        runs = [{"metrics": [{"id": "a", "value": v, "req": 2.0, "pass": int(v <= 2.0) if math.isfinite(v) else 0, "unit": "deg"}]}
                for v in (1.0, 3.0, float("nan"))]
        a, = C.summarise(runs)
        self.assertEqual((a["mean"], a["min"], a["max"], a["n_valid"], a["unit"]), (2.0, 1.0, 3.0, 2, "deg"))
        self.assertAlmostEqual(a["std"], math.sqrt(2))
        self.assertAlmostEqual(a["pass_rate"], 1 / 3)
        self.assertFalse(a["pass"])

    def test_a_metric_missing_from_a_run_is_not_a_verdict(self):
        runs = [{"metrics": [{"id": "a", "value": 1.0, "req": 2.0, "pass": 1}]}, {"metrics": [{"id": "b", "value": 5.0}]}]
        a, b = C.summarise(runs)
        self.assertEqual((a["values"], a["pass_rate"], a["pass"], a["std"]), ([1.0, None], 1.0, True, 0.0))
        self.assertEqual((b["values"], b["mean"], b["pass_rate"]), ([None, 5.0], 5.0, None))

    def test_no_runs_no_statistics(self):
        self.assertEqual(C.summarise([]), [])


class ModeScenario(unittest.TestCase):
    def mode(self, name):
        return json.loads((S.MODES_DIR / f"{name}.json").read_text())

    def test_a_mode_test_is_the_options_scenario_on_the_sized_product(self):
        M = self.mode("nadir_pointing")
        o = next(x for x in M["options"] if x["id"] == "rw+rcs")
        s = S.mode_scenario("ais_3u", M, o)
        self.assertEqual(s["id"], "ais_3u__nadir_pointing__rw_rcs")
        self.assertEqual(s["product"], f"SZ-ais_3u-{o['family']}")
        self.assertEqual(s["time"]["duration_s"], round(M["test"]["duration_orbits"] * PERIOD))
        self.assertEqual(s["time"]["dt_s"], o["dt_s"])
        self.assertEqual(s["fsw"], {"start_mode": o["fsw_mode"], "guidance": {"kind": "nadir"}, "rcs_dump": 1.0})
        self.assertEqual(s["initial"], {"attitude": M["test"]["attitude"], "rate": M["test"]["rate"]})
        self.assertEqual([m["id"] for m in s["metrics"]], [m["id"] for m in M["metrics"]])

    def test_an_option_may_fly_longer_on_its_own_window_and_initial_state(self):
        M = self.mode("nadir_pointing")
        o = next(x for x in M["options"] if x["id"] == "mtq")
        s = S.mode_scenario("ais_3u", M, o)
        self.assertEqual(s["time"]["duration_s"], round(o["duration_orbits"] * PERIOD))
        self.assertEqual(s["initial"], o["test_initial"])
        self.assertEqual(s["fsw"]["rcs_dump"], 0.0)

    def test_a_fluid_loop_gets_its_dump_gain(self):
        M = self.mode("nadir_pointing")
        o = next(x for x in M["options"] if x["actuator"] == "fmr")
        self.assertEqual(S.mode_scenario("ais_3u", M, o)["fsw"]["dump_gain"], 0.03)

    def test_a_value_the_case_does_not_state_is_refused_by_name(self):
        with self.assertRaisesRegex(SystemExit, "req.rks"):
            S.case_value("ais_3u", "req.rks")
        self.assertEqual(S.case_value("ais_3u", "orbit.alt"), ALT)


class Small(unittest.TestCase):
    def test_the_orbit_period_from_the_altitude(self):
        self.assertAlmostEqual(run_matrix.orbit_period("ais_3u"), PERIOD)
        self.assertTrue(5600 < PERIOD < 5800, "550 km: about 95.6 min")
        with mock.patch.object(common, "case_values", lambda case: {}), self.assertRaisesRegex(SystemExit, "orbit.alt"):
            run_matrix.orbit_period("ais_3u")

    def test_the_parity_line_is_found_or_the_last_line_given(self):
        p = lambda out, err="": subprocess.CompletedProcess([], 1, stdout=out, stderr=err)
        self.assertEqual(engine_runs.parity_line(p("a\n[parity] 0 differ\nb\n")), "[parity] 0 differ")
        self.assertEqual(engine_runs.parity_line(p("a\n", "[parity] in stderr\n")), "[parity] in stderr")
        self.assertEqual(engine_runs.parity_line(p("a\nlast\n")), "last")
        self.assertEqual(engine_runs.parity_line(p("")), "no output")

    def test_rescore_reads_the_case_and_refuses_a_case_that_is_not_there(self):
        self.assertEqual(rescore.req("ais_3u", "req.ape"), common.case_values("ais_3u")["req.ape"])
        self.assertIsNone(rescore.req("ais_3u", "req.rks"), "an unstated requirement")
        with self.assertRaisesRegex(SystemExit, "no_such_case"):
            rescore.req("no_such_case", "req.ape")

    def test_rescore_judges_with_the_nearest_enclosing_case(self):
        ape = common.case_values("ais_3u")["req.ape"]
        node = {"case": "ais_3u", "runs": [{"metrics": [{"id": "ape_los_p9973", "req_key": "req.ape", "value": ape * 0.5, "req": 1.0, "pass": 0},
                                                        {"id": "x", "req_key": "req.ape", "value": float("nan"), "req": 1.0, "pass": 1},
                                                        {"id": "y", "req_key": "req.ape", "value": 1.0, "req": ape, "pass": 1}]}]}
        n = [0]
        rescore.judge(node, None, n)
        m = node["runs"][0]["metrics"]
        self.assertEqual((m[0]["req"], m[0]["pass"]), (ape, 1))
        self.assertEqual(m[1]["pass"], 0, "no number is no pass")
        self.assertEqual(n[0], 2, "an unchanged requirement is not counted")


class Floquet(unittest.TestCase):
    def setUp(self):
        rng = np.random.default_rng(7)
        self.q = [x / np.linalg.norm(x) for x in rng.normal(size=(6, 4))]

    def test_the_dcm_is_a_rotation_and_identity_at_the_identity(self):
        np.testing.assert_allclose(F.dcm(np.array([0, 0, 0, 1.0])), np.eye(3))
        for q in self.q:
            A = F.dcm(q)
            np.testing.assert_allclose(A @ A.T, np.eye(3), atol=1e-12)
            self.assertAlmostEqual(np.linalg.det(A), 1.0)
            np.testing.assert_allclose(F.dcm(-q), A, atol=1e-12, err_msg="q and -q are one attitude")

    def test_the_dcm_is_passive_and_composes_as_the_flight_software(self):
        q = F.rv2q(np.array([0, 0, math.pi / 2]))
        np.testing.assert_allclose(F.dcm(q) @ [1, 0, 0], [0, -1, 0], atol=1e-12)
        for a, b in zip(self.q[:3], self.q[3:]):
            np.testing.assert_allclose(F.dcm(F.qmul(a, b)), F.dcm(b) @ F.dcm(a), atol=1e-12)

    def test_qmul_has_the_identity_and_the_conjugate_inverse(self):
        e = np.array([0, 0, 0, 1.0])
        for q in self.q:
            np.testing.assert_allclose(F.qmul(q, e), q)
            np.testing.assert_allclose(F.qmul(e, q), q)
            np.testing.assert_allclose(F.qmul(q, np.array([-q[0], -q[1], -q[2], q[3]])), e, atol=1e-12)

    def test_rv2q_is_a_unit_quaternion_of_that_angle_and_safe_at_zero(self):
        for v in ([0.3, -0.2, 0.1], [math.pi, 0, 0], [0, 0, 1e-17], [0, 0, 0]):
            q = F.rv2q(np.array(v, dtype=float))
            self.assertAlmostEqual(np.linalg.norm(q), 1.0, places=12)
            self.assertAlmostEqual(2 * math.acos(min(1.0, q[3])), np.linalg.norm(v), places=7)
        np.testing.assert_allclose(F.rv2q(np.array([math.pi, 0, 0])), [1, 0, 0, 0], atol=1e-12)

    P = {"J": np.diag([0.01, 0.04, 0.04]).tolist(), "mtq_Kp": [1e-4] * 3, "mtq_Kd": [2e-3] * 3, "mtq_eps": 0.1, "mtq_k1": 1.0, "mtq_k2": 1.0,
         "mtq_lam16": 0.5, "mtq_k16": 1e-3, "sb_kp": 1e-4, "sb_kd": 1e-3, "sb_kroll": 0.0, "sb_roll_gate": 0.9, "sun_axis": [0, 0, 1],
         "mtq_Pth": (1e-4 * np.eye(3)).tolist(), "mtq_Pw": (1e-3 * np.eye(3)).tolist()}

    def test_every_law_asks_no_torque_on_the_reference(self):
        n = 1.1e-3
        wref = np.array([0, -n, 0])                    # about the orbit normal, a principal axis
        e3 = np.array([0, 0, 1.0])
        qe = np.array([0, 0, 0, 1.0])
        for law in F.LAWS:
            np.testing.assert_allclose(F.law_torque(law, self.P, qe, wref.copy(), wref, e3, None), 0, atol=1e-15, err_msg=F.LAWS[law])

    def test_the_pd_law_restores_an_error_and_damps_a_rate(self):
        qe = F.rv2q(np.array([0.1, 0, 0]))
        tau = F.law_torque(0, self.P, qe, np.zeros(3), np.zeros(3), np.array([0, 0, 1.0]), None)
        self.assertLess(tau[0], 0)
        tau = F.law_torque(3, self.P, np.array([0, 0, 0, 1.0]), np.array([0, 0.01, 0]), np.zeros(3), np.array([0, 0, 1.0]), None)
        np.testing.assert_allclose(tau, [0, -2e-5, 0])
        # the short way round: q and -q ask the same torque
        np.testing.assert_allclose(F.law_torque(0, self.P, -qe, np.zeros(3), np.zeros(3), None, None),
                                   F.law_torque(0, self.P, qe, np.zeros(3), np.zeros(3), None, None))

    def test_an_unknown_law_is_refused(self):
        with self.assertRaises(ValueError):
            F.law_torque(1, self.P, np.array([0, 0, 0, 1.0]), np.zeros(3), np.zeros(3), np.array([0, 0, 1.0]), None)

    def test_the_laws_are_the_registrys_candidates(self):
        import pipeline_base
        self.assertLessEqual(set(F.LAWS.values()), set(pipeline_base.CANDIDATES["mtq_pointing"]))


if __name__ == "__main__":
    unittest.main()


class Interpretations(unittest.TestCase):
    """The ECSS temporal / ensemble / mixed interpretations over a campaign's runs."""

    def runs(self, series):
        import tempfile, pathlib
        tmp = pathlib.Path(tempfile.mkdtemp())
        self.addCleanup(lambda: __import__("shutil").rmtree(tmp, ignore_errors=True))
        dirs = []
        for k, x in enumerate(series):
            d = tmp / f"run_{k:04d}"
            d.mkdir()
            (d / "channels.csv").write_text("t_s,ape_los_deg\n" + "".join(f"{i},{v}\n" for i, v in enumerate(x)))
            (d / "manifest.json").write_text(json.dumps({"orbit": {"period_s": 100.0}}))
            dirs.append(d)
        return dirs

    def test_the_three_interpretations_of_a_max(self):
        # run A is large early, run B large late: each run's worst is 5, but never both at once
        A, B = [5, 1, 1, 1], [1, 1, 1, 5]
        x = C.interpretations(self.runs([A, B]), [{"id": "e", "kind": "ape_los", "statistic": "max"}])[0]
        self.assertEqual((x["temporal"], x["ensemble"], x["mixed"], x["runs"]), (5.0, 5.0, 5.0, 2))

    def test_a_percentile_separates_them(self):
        # 3 runs, 4 instants; at 50 %: per run the 2nd smallest, across runs the 2nd smallest at each instant
        series = [[1, 2, 3, 9], [2, 2, 2, 2], [8, 1, 1, 1]]
        x = C.interpretations(self.runs(series), [{"id": "e", "kind": "ape_los", "statistic": "p95"}])[0]
        self.assertEqual(x["temporal"], 9.0, "the worst run's p95")
        self.assertEqual(x["ensemble"], 9.0, "the p95 across 3 runs is their max; worst instant 9")
        self.assertEqual(x["mixed"], 9.0, "pooled p95 of 12 samples is the 12th smallest")
        C.INTERP_Q["p50"] = 0.5
        try:
            y = C.interpretations(self.runs(series), [{"id": "e", "kind": "ape_los", "statistic": "p50"}])[0]
        finally:
            del C.INTERP_Q["p50"]
        self.assertEqual((y["temporal"], y["ensemble"], y["mixed"]), (2.0, 2.0, 2.0))

    def test_a_window_and_a_metric_it_does_not_cover(self):
        x = C.interpretations(self.runs([[9, 9, 1, 1]]), [{"id": "e", "kind": "ape_los", "statistic": "max", "window": "after_s:2"},
                                                         {"id": "p", "kind": "power_mean"}])
        self.assertEqual([i["id"] for i in x], ["e"])
        self.assertEqual(x[0]["temporal"], 1.0, "only the window's samples")


class Claims(unittest.TestCase):
    """How many runs a probability claim needs, and the reliability a campaign's runs show."""

    def test_the_success_run_count(self):
        self.assertEqual(C.success_runs(0.9973, 0.95), 1109, "99.73 % at 95 % confidence: 1108 runs show only 99.72999 %")
        self.assertEqual(C.success_runs(0.99, 0.90), 230)

    def test_the_lower_bound_is_clopper_pearsons(self):
        # SciPy 1.17.1: scipy.stats.beta.ppf(1 - confidence, n - failures, failures + 1)
        for n, f, conf, want in [(10, 1, 0.9, 0.6631522766932753), (1108, 0, 0.95, 0.9972999222959197),
                                 (1108, 3, 0.95, 0.9930171138530246), (24, 2, 0.95, 0.7601989838711982)]:
            self.assertAlmostEqual(C.reliability_lower(n, f, conf), want, places=9, msg=f"{n} runs, {f} failed")
        self.assertIsNone(C.reliability_lower(0, 0, 0.95))

    def test_a_campaign_s_claim_comes_from_the_case_level(self):
        c = {"case": "ais_img_3u", "scenario": "fine_hold_img", "runs": 24, "confidence": 0.95}
        self.assertEqual(C.claim(c), (0.9973, 1109), "req.ape is stated at 99.73 %")
        self.assertEqual(C.claim({**c, "confidence": None}), (None, 24))
        del c["confidence"]
        self.assertEqual(C.claim(c), (None, 24), "no confidence, no claim")

    def test_a_summary_says_whether_the_runs_meet_the_claim(self):
        runs = [{"metrics": [{"id": "ape", "value": 0.005, "req": 0.01, "pass": 1}]} for _ in range(1109)]
        s = C.summarise(runs, claim=(0.9973, 0.95))[0]
        self.assertTrue(s["claim_met"])
        runs[0]["metrics"][0]["pass"] = 0
        s = C.summarise(runs, claim=(0.9973, 0.95))[0]
        self.assertFalse(s["claim_met"], "one failure in 1109 no longer shows 99.73 %")
        self.assertNotIn("claim_met", C.summarise(runs)[0], "no claim, no verdict on it")


class KeepInterp(unittest.TestCase):
    def test_a_campaign_run_keeps_the_interpretation_channels_and_the_same_answer(self):
        import json, pathlib, tempfile
        import engine_campaigns as EC
        with tempfile.TemporaryDirectory() as t:
            d = pathlib.Path(t) / "run_0001"
            d.mkdir()
            rows = ["t_s,q_x,ape_los_deg,ake_los_deg,P_rw_W"] + [f"{i},0.1,{0.001 * i},{0.0005 * i},2" for i in range(10)]
            (d / "channels.csv").write_text("\n".join(rows) + "\n")
            (d / "manifest.json").write_text(json.dumps({"orbit": {"period_s": 5.0}}))
            m = [{"id": "a", "kind": "ape_los", "statistic": "max", "window": "all"}]
            before = EC.interpretations([d], m)
            EC.keep_interp(d)
            self.assertFalse((d / "channels.csv").exists())
            self.assertEqual((d / "interp.csv").read_text().splitlines()[0], "t_s,ape_los_deg,ake_los_deg")
            self.assertEqual(EC.interpretations([d], m), before)
