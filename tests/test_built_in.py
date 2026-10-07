"""The built-in count (docs/PLAN_2_0.md S7; docs/S7_INVENTORY.md S7.1b, S7.3, S7.3b-e): the nodes of the regression copy whose
relation is still compiled code, by group. S7 lowers it to zero; each step that writes a method takes its nodes
out of BUILT_IN here, in the same change.
  - the count and the list are exactly these, and tools/health.py reports them;
  - nothing else is built-in: an achieved holder or a KPI's evidence row is evidence, a KPI closure is a closure
    the library compares from the design's own rows, the flight software's runtime rows are stated and marked
    code by the boundary;
  - every node the developer's revision S7.1b moved says so in its release, unsigned, with its reason;
  - every block's behaviour is one the schema knows.

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import json
import pathlib
import sqlite3
import tomllib
import unittest

import _path  # puts tools/ on the import path
import from_design
import health
import tndb

_ = _path  # imported for its effect: tools/ on sys.path
ROOT = pathlib.Path(__file__).resolve().parents[1]
REG = ROOT / "tests" / "regression"

BUILT_IN = {
    "act": ["act_cmg_model", "act_vscmg_gimbal_limits", "act_vscmg_model", "gm_4", "gm_5", "gw_2", "gw_5", "gw_6",
            "l3_fmr_row_07", "l3_fmr_row_08", "l3_fmr_row_09", "l3_fmr_row_10", "l3_fmr_row_11", "l3_fmr_row_12", "l3_fmr_row_13",
            "l3_fmr_row_14", "l3_fmr_row_15", "l3_mtq_row_02", "l3_mtq_row_03", "l3_mtq_row_04", "l3_mtq_row_05", "l3_mtq_row_12",
            "l3_mtq_row_13", "l3_rcs_row_01", "l3_rcs_row_02", "l3_rcs_row_03", "l3_rcs_row_04", "l3_rcs_row_08", "l3_rcs_row_09",
            "l3_rcs_row_10", "l3_rcs_row_11", "l3_rw_row_01", "l3_rw_row_02", "l3_rw_row_03", "l3_rw_row_04", "l3_rw_row_05",
            "l3_rw_row_06", "l3_rw_row_07"],
    "design": ["design_sizing_cmg", "design_sizing_fmr", "design_sizing_mtq", "design_sizing_rcs", "design_sizing_rw",
               "design_sizing_sensors", "design_sizing_vscmg", "gb_0", "gb_1", "gb_2", "gb_3", "l3_budget_row_01", "l3_budget_row_02",
               "l3_budget_row_03", "l3_budget_row_04", "l3_budget_row_05", "l3_budget_row_06", "l3_budget_row_07", "l3_budget_row_08"],
    "dyn": ["dyn_flexible_mode", "dyn_rigid_body", "dyn_rotor_coupling", "dyn_total_momentum"],
    "env": ["l3_dist_row_01", "l3_dist_row_02", "l3_dist_row_03", "l3_dist_row_04", "l3_dist_row_05", "l3_dist_row_06",
            "l3_dist_row_09", "l3_dist_row_10", "m2_7"],
    "oils": ["l3_oils_row_07"],
    "pnt": ["gp_0", "gp_1", "gp_2", "gp_4"],
    "sens": ["l3_sens_row_01", "l3_sens_row_02", "l3_sens_row_03", "l3_sens_row_04", "l3_sens_row_05", "l3_sens_row_06",
             "l3_sens_row_07", "l3_sens_row_08", "l3_sens_row_09", "l3_sens_row_10", "l3_sens_row_11", "l3_sens_row_12",
             "l3_sens_row_13", "l3_sens_row_14"],
}
BOUNDARY = {"l3_fsw_row_01": "runtime", "l3_fsw_row_02": "runtime", "l3_fsw_row_03": "runtime", "l3_fsw_row_04": "runtime",
            "l3_fsw_row_10": "test"}


class BuiltIn(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        with sqlite3.connect(f"file:{REG / 'design.tndb'}?mode=ro", uri=True) as c:
            cls.nodes = {nid: (kind, json.loads(content)) for nid, kind, content in c.execute("SELECT id, kind, content FROM design_node")}

    def beh(self, nid):
        return self.nodes[nid][1]["body"]["block"][0][3]

    def test_the_count_is_89_and_these(self):
        bi = health.built_in(REG)
        self.assertEqual(bi["by_group"], BUILT_IN)
        self.assertEqual(bi["count"], 89)
        self.assertEqual(bi["boundary"], BOUNDARY)

    def test_what_is_not_a_relation_in_code_is_not_built_in(self):
        for nid, (kind, x) in self.nodes.items():
            content = {(s, f) for s, f, _v, _o in x["body"]["content"]}
            if kind == "achieved" or ("evidence", "metric") in content:
                self.assertEqual(self.beh(nid), "evidence", nid)
            if kind in ("closure_analysis", "closure_verified"):
                self.assertEqual(self.beh(nid), "closure", nid)
            if nid in BOUNDARY:
                self.assertEqual(self.beh(nid), "stated", nid)

    def test_a_closure_is_compared_from_the_designs_own_rows(self):
        """tools/evaluate.py answers each KPI from the design's KPI table; every closure block holds the same
        requirement, achieved row and sense, so the comparison is the design's and no relation in code."""
        kpis = tomllib.loads(from_design.text("spec/plan/kpis.toml", REG / "design.tndb"))["kpi"]
        want = {}
        for k in kpis:
            want[f"kpi_{k['slug']}_verified"] = (k["requirement"], k["evidence"], k["sense"], "evidence")
            if k["analysis"] != "none":
                want[f"kpi_{k['slug']}_analysis"] = (k["requirement"], k["analysis"], k["sense"], "analysis")
        have = {nid: tuple(x["body"]["closure"][0][1:5]) for nid, (_k, x) in self.nodes.items() if self.beh(nid) == "closure"}
        self.assertEqual(have, want)

    def test_every_revised_node_says_so_unsigned(self):
        moved = 0
        for nid, (_k, x) in self.nodes.items():
            mine = [w for w in x["why"] if w.startswith("the developer's revision S7.1b")]
            if mine:
                moved += 1
                self.assertEqual(x["sealed_as"], "unconfirmed", nid)
                self.assertIn("not yet signed by a person", mine[0])
        self.assertEqual(moved, 180 + 22, "the 180 built-in nodes that are not relations in code, and the 22 KPI evidence rows")

    def test_a_relation_out_of_code_is_a_method_the_developer_transcribed(self):
        """S7.3: the density (l3_dist_row_07) and the main field (l3_dist_row_08) are methods now, each the developer's
        unsigned transcription of the code it replaced and of its source, generated into the engine
        (tools/engine_build.py), as are the published models S7.3 added to env."""
        moved = {"l3_dist_row_07", "l3_dist_row_08", "env_time_frames", "env_calendar_time", "env_two_body_elements",
                 "env_time_scales", "env_geodetic", "env_earth_frames", "env_iau2006", "env_tidal_eop"}
        for nid in moved:
            _k, x = self.nodes[nid]
            self.assertEqual(self.beh(nid), "method", nid)
            rows = {(s, f): (v, o) for s, f, v, o in x["body"]["content"]}
            self.assertIn("the developer's revision S7.3", rows[("code", "pseudocode")][1], nid)
            self.assertTrue(rows[("code", "transcribes")][0], nid)
            self.assertEqual(x["sealed_as"], "unconfirmed", nid)
            self.assertTrue(any(w.startswith("the developer's revision S7.3") and "not yet signed by a person" in w for w in x["why"]), nid)
        self.assertEqual(self.beh("nav_time_frames"), "stated")

    def test_the_atmosphere_is_methods_the_developer_transcribed(self):
        """S7.3b: the precision orbit's atmosphere (DTM2020 operational and research, JB2008, the exponential
        atmosphere, the density switch and which index each model takes) is env's methods over env's tables, each the
        developer's unsigned transcription, generated into adcs-pop (tools/engine_build.py). No built-in node moved:
        the inventory counted these models toolbox until the ruling of 7 Oct, so they were never built-in."""
        added = {"env_dtm2020_operational", "env_dtm2020_research", "env_jb2008", "env_exponential_atmosphere",
                 "env_density_model", "env_space_weather"}
        data = {"env_dtm2020_coefficients", "env_jb2008_indices", "env_kp_ap_table"}
        for nid in added | data:
            _k, x = self.nodes[nid]
            rows = {(s, f): (v, o) for s, f, v, o in x["body"]["content"]}
            self.assertEqual(self.beh(nid), "method" if nid in added else "lookup", nid)
            if nid in added:
                self.assertIn("the developer's revision S7.3b", rows[("code", "pseudocode")][1], nid)
                self.assertTrue(rows[("code", "transcribes")][0], nid)
                self.assertEqual(rows[("code", "generate")][0], "adcs-pop", nid)
            self.assertEqual(x["sealed_as"], "unconfirmed", nid)
            self.assertTrue(any(w.startswith("the developer's revision S7.3b") and "not yet signed by a person" in w for w in x["why"]), nid)

    def test_the_ephemeris_is_a_method_the_developer_transcribed(self):
        """S7.3c: the Sun and Moon from DE440 and the per-step bundle are env's method over env's slice of DE440's
        records, the developer's unsigned transcription, generated into adcs-pop. l3_dist_row_09 (the Sun's direction
        and distance) stays built-in: the fast orbit's analytic Sun (adcs-sim-core ephem.rs) is S7.4's."""
        for nid, beh in (("env_de440", "method"), ("env_de440_slice", "lookup")):
            _k, x = self.nodes[nid]
            rows = {(s, f): (v, o) for s, f, v, o in x["body"]["content"]}
            self.assertEqual(self.beh(nid), beh, nid)
            if beh == "method":
                self.assertIn("the developer's revision S7.3c", rows[("code", "pseudocode")][1])
                self.assertEqual(rows[("code", "generate")][0], "adcs-pop")
            self.assertEqual(x["sealed_as"], "unconfirmed", nid)
            self.assertTrue(any(w.startswith("the developer's revision S7.3c") and "not yet signed by a person" in w for w in x["why"]), nid)
        self.assertEqual(self.beh("l3_dist_row_09"), "built-in")

    def test_gravity_and_tides_are_methods_the_developer_transcribed(self):
        """S7.3d: the Earth's gravity (the field's normalisation and zonals, the spherical harmonics, the potential, the
        zonal J2..J6, the force's frames), the solid-Earth tides (IERS 2010) and the ocean tides are env's methods over
        env's tables, the developer's unsigned transcriptions, generated into adcs-pop; the ICGEM .gfc reader stays
        code. No built-in node moved (they were toolbox in the inventory until the ruling of 7 Oct)."""
        for nid, beh in (("env_gravity_field", "method"), ("env_solid_tides", "method"), ("env_ocean_tides", "method"),
                         ("env_gravity_default_field", "lookup"), ("env_ocean_tide_tables", "lookup")):
            _k, x = self.nodes[nid]
            rows = {(s, f): (v, o) for s, f, v, o in x["body"]["content"]}
            self.assertEqual(self.beh(nid), beh, nid)
            if beh == "method":
                self.assertIn("the developer's revision S7.3d", rows[("code", "pseudocode")][1], nid)
                self.assertEqual(rows[("code", "generate")][0], "adcs-pop", nid)
            self.assertEqual(x["sealed_as"], "unconfirmed", nid)
            self.assertTrue(any(w.startswith("the developer's revision S7.3d") and "not yet signed by a person" in w for w in x["why"]), nid)

    def test_relativity_is_a_method_the_developer_transcribed(self):
        """S7.3e: the IERS 2010 post-Newtonian terms (Schwarzschild, Lense-Thirring, de Sitter) and their sum are env's
        method, the developer's unsigned transcription, generated into adcs-pop. The count stays 89 through S7.3b-e:
        the published models were toolbox in the inventory until the ruling of 7 Oct, never built-in nodes; each step
        added env's nodes for them."""
        _k, x = self.nodes["env_relativity"]
        rows = {(s, f): (v, o) for s, f, v, o in x["body"]["content"]}
        self.assertEqual(self.beh("env_relativity"), "method")
        self.assertIn("the developer's revision S7.3e", rows[("code", "pseudocode")][1])
        self.assertEqual(rows[("code", "generate")][0], "adcs-pop")
        self.assertEqual(x["sealed_as"], "unconfirmed")
        self.assertTrue(any(w.startswith("the developer's revision S7.3e") and "not yet signed by a person" in w for w in x["why"]))

    def test_every_behaviour_is_one_the_schema_knows(self):
        self.assertEqual(tndb.check(REG / "design.tndb"), [])
        known = set(tndb.schema()["behaviours"])
        self.assertEqual({self.beh(n) for n in self.nodes} - known, set())


if __name__ == "__main__":
    unittest.main()
