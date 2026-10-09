"""The built-in count (docs/PLAN_2_0.md S7; docs/S7_INVENTORY.md S7.1b, S7.3, S7.3b-e, S7.4, S7.5, S7.6, S7.7, S7.8, S7.9, S7.10, S7.11, S7.12, S7.13, S7.14, S7.14b, S7.15, S7.15b): the nodes of the regression copy whose
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
import re
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

BUILT_IN = {}     # S7.15: the sizing's 27 (act 8, design 19) were the last
BOUNDARY = {"l3_fsw_row_01": "runtime", "l3_fsw_row_02": "runtime", "l3_fsw_row_03": "runtime", "l3_fsw_row_04": "runtime",
            "l3_fsw_row_10": "test"}


class BuiltIn(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        with sqlite3.connect(f"file:{REG / 'design.tndb'}?mode=ro", uri=True) as c:
            cls.nodes = {nid: (kind, json.loads(content)) for nid, kind, content in c.execute("SELECT id, kind, content FROM design_node")}

    def beh(self, nid):
        return self.nodes[nid][1]["body"]["block"][0][3]

    def test_the_count_is_zero(self):
        bi = health.built_in(REG)
        self.assertEqual(bi["by_group"], BUILT_IN)
        self.assertEqual(bi["count"], 0)
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
        and distance) was the fast orbit's analytic Sun (adcs-sim-core ephem.rs), S7.4's."""
        for nid, beh in (("env_de440", "method"), ("env_de440_slice", "lookup")):
            _k, x = self.nodes[nid]
            rows = {(s, f): (v, o) for s, f, v, o in x["body"]["content"]}
            self.assertEqual(self.beh(nid), beh, nid)
            if beh == "method":
                self.assertIn("the developer's revision S7.3c", rows[("code", "pseudocode")][1])
                self.assertEqual(rows[("code", "generate")][0], "adcs-pop")
            self.assertEqual(x["sealed_as"], "unconfirmed", nid)
            self.assertTrue(any(w.startswith("the developer's revision S7.3c") and "not yet signed by a person" in w for w in x["why"]), nid)

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

    def test_the_environment_and_disturbances_are_methods_the_developer_transcribed(self):
        """S7.4: the disturbance torques (gravity gradient, aerodynamic per face with the box's faces, radiation of the Sun
        and the Earth's albedo and infrared, the residual dipole), the fast orbit's Sun and its pressure, its Moon (a new
        node), the conical shadow and the eclipse fraction are env's methods, each the developer's unsigned transcription,
        generated into adcs-sim-core; env has no built-in node left."""
        moved = {"l3_dist_row_01", "l3_dist_row_02", "l3_dist_row_03", "l3_dist_row_04", "l3_dist_row_05", "l3_dist_row_06",
                 "l3_dist_row_09", "l3_dist_row_10", "m2_7"}
        for nid in moved | {"env_moon_fast"}:
            _k, x = self.nodes[nid]
            rows = {(s, f): (v, o) for s, f, v, o in x["body"]["content"]}
            self.assertEqual(self.beh(nid), "method", nid)
            self.assertIn("the developer's revision S7.4", rows[("code", "pseudocode")][1], nid)
            self.assertTrue(rows[("code", "transcribes")][0], nid)
            self.assertEqual(rows[("code", "generate")][0], "adcs-sim-core", nid)
            self.assertEqual(x["sealed_as"], "unconfirmed", nid)
            self.assertTrue(any(w.startswith("the developer's revision S7.4") and "not yet signed by a person" in w for w in x["why"]), nid)
        self.assertNotIn("env", health.built_in(REG)["by_group"])

    def test_the_plant_is_methods_the_developer_transcribed(self):
        """S7.5: the rotors' geometry and coupling, the flexible mode, the rigid body's rate with its momentum devices and
        the total momentum are dyn's methods, each the developer's unsigned transcription, generated into adcs-sim-core;
        the integrator (RK4, the flexible mode's sub-steps) stays code. dyn has no built-in node left."""
        for nid in ("dyn_rotor_coupling", "dyn_flexible_mode", "dyn_rigid_body", "dyn_total_momentum"):
            _k, x = self.nodes[nid]
            rows = {(s, f): (v, o) for s, f, v, o in x["body"]["content"]}
            self.assertEqual(self.beh(nid), "method", nid)
            self.assertIn("the developer's revision S7.5", rows[("code", "pseudocode")][1], nid)
            self.assertTrue(rows[("code", "transcribes")][0], nid)
            self.assertEqual(rows[("code", "generate")][0], "adcs-sim-core", nid)
            self.assertEqual(x["sealed_as"], "unconfirmed", nid)
            self.assertTrue(any(w.startswith("the developer's revision S7.5") and "not yet signed by a person" in w for w in x["why"]), nid)
        self.assertNotIn("dyn", health.built_in(REG)["by_group"])

    def test_the_orbit_is_methods_the_developer_transcribed(self):
        """S7.6: the fast orbit's forces and node context and its start from the LTAN, and the precision orbit's
        spacecraft force models (the third body, the gas-surface interaction, drag, solar and Earth radiation pressure)
        with its force set, sum and sun-synchronous start, are env's methods, each the developer's unsigned
        transcription, generated into the engine. They had no node (the inventory's §1.3) or were adcs-pop's relations,
        never built-in: the count does not move."""
        targets = {"env_orbit_fast": "adcs-sim-core", "env_orbit_start": "adcs-sim", "env_third_body": "adcs-pop",
                   "env_gas_surface": "adcs-pop", "env_drag_force": "adcs-pop", "env_srp_force": "adcs-pop",
                   "env_erp_force": "adcs-pop", "env_force_model": "adcs-pop"}
        for nid, target in targets.items():
            _k, x = self.nodes[nid]
            rows = {(s, f): (v, o) for s, f, v, o in x["body"]["content"]}
            self.assertEqual(self.beh(nid), "method", nid)
            self.assertEqual(x["body"]["node"]["group_id"], "env", nid)
            self.assertIn("the developer's revision S7.6", rows[("code", "pseudocode")][1], nid)
            self.assertTrue(rows[("code", "transcribes")][0], nid)
            self.assertEqual(rows[("code", "generate")][0], target, nid)
            self.assertEqual(x["sealed_as"], "unconfirmed", nid)
            self.assertTrue(any(w.startswith("the developer's revision S7.6") and "not yet signed by a person" in w for w in x["why"]), nid)

    def test_the_actuators_are_methods_the_developer_transcribed(self):
        """S7.7: the coils, the wheels, the rings, the CMG's and VSCMG's rotors and gimbals, the momentum devices as the
        engine flies them (a new node) and the thrusters are act's methods, each the developer's unsigned transcription,
        generated into adcs-sim-core; the torque of a dipole in the field (l3_mtq_row_06, open until now) too. act keeps
        only its sizing rows (S7.15) built-in."""
        moved = {"l3_mtq_row_02", "l3_mtq_row_03", "l3_mtq_row_04", "l3_mtq_row_05", "l3_mtq_row_12", "l3_mtq_row_13",
                 "l3_rw_row_01", "l3_rw_row_02", "l3_rw_row_03", "l3_rw_row_04", "l3_rw_row_05", "l3_rw_row_06", "l3_rw_row_07",
                 "l3_fmr_row_08", "l3_fmr_row_09", "l3_fmr_row_10", "l3_fmr_row_11", "l3_fmr_row_12", "l3_fmr_row_13",
                 "act_cmg_model", "act_vscmg_model", "act_vscmg_gimbal_limits",
                 "l3_rcs_row_01", "l3_rcs_row_02", "l3_rcs_row_03", "l3_rcs_row_04", "l3_rcs_row_08", "l3_rcs_row_09",
                 "l3_rcs_row_10", "l3_rcs_row_11"}
        self.assertEqual(len(moved), 30)
        for nid in moved | {"act_rotor_set", "l3_mtq_row_06"}:
            _k, x = self.nodes[nid]
            rows = {(s, f): (v, o) for s, f, v, o in x["body"]["content"]}
            self.assertEqual(self.beh(nid), "method", nid)
            self.assertEqual(x["body"]["node"]["group_id"], "act", nid)
            self.assertIn("the developer's revision S7.7", rows[("code", "pseudocode")][1], nid)
            self.assertTrue(rows[("code", "transcribes")][0], nid)
            self.assertEqual(rows[("code", "generate")][0], "adcs-sim-core", nid)
            self.assertEqual(x["sealed_as"], "unconfirmed", nid)
            self.assertTrue(any(w.startswith("the developer's revision S7.7") and "not yet signed by a person" in w for w in x["why"]), nid)
        self.assertEqual(set(health.built_in(REG)["by_group"].get("act", [])) & moved, set())

    def test_the_simple_sensors_are_methods_the_developer_transcribed(self):
        """S7.8: the gyro, the magnetometer, the coarse and fine Sun sensors (the noise model and the quadrant chain), the
        Earth sensor and the GNSS receiver are sens's methods, each the developer's unsigned transcription, generated into
        adcs-sim-core; the fine Sun sensors as the engine flies them (a new node), the sky the sensors see (a new node,
        into adcs-sim, the Earth's half-angle from the constants' R_E in place of run.rs's literal) and the rotors'
        telemetry (a new node of act) too. sens keeps only the star tracker's rows (S7.9 and S7.10) built-in."""
        moved = {"l3_sens_row_01", "l3_sens_row_02", "l3_sens_row_03", "l3_sens_row_04", "l3_sens_row_05", "l3_sens_row_06",
                 "l3_sens_row_07", "l3_sens_row_08", "l3_sens_row_13", "l3_sens_row_14"}
        added = {"sens_fine_sun": ("sens", "adcs-sim-core"), "sens_sky_view": ("sens", "adcs-sim"), "act_rotor_telemetry": ("act", "adcs-sim-core")}
        for nid in moved | set(added):
            _k, x = self.nodes[nid]
            rows = {(s, f): (v, o) for s, f, v, o in x["body"]["content"]}
            group, target = added.get(nid, ("sens", "adcs-sim-core"))
            self.assertEqual(self.beh(nid), "method", nid)
            self.assertEqual(x["body"]["node"]["group_id"], group, nid)
            self.assertIn("the developer's revision S7.8", rows[("code", "pseudocode")][1], nid)
            self.assertTrue(rows[("code", "transcribes")][0], nid)
            self.assertEqual(rows[("code", "generate")][0], target, nid)
            self.assertEqual(x["sealed_as"], "unconfirmed", nid)
            self.assertTrue(any(w.startswith("the developer's revision S7.8") and "not yet signed by a person" in w for w in x["why"]), nid)
        self.assertEqual(set(health.built_in(REG)["by_group"].get("sens", [])) & moved, set())
        run = (ROOT / "engine" / "crates" / "adcs-sim" / "src" / "run.rs").read_text(encoding="utf-8")
        self.assertNotIn("6378137", run, "the Earth's radius is the constants' (sens_sky_view), not a literal in run.rs")

    def stated_file(self):
        """data/stated.json as the regression copy gives the engine (its engine_input), and the data folder's copy."""
        with sqlite3.connect(f"file:{REG / 'design.tndb'}?mode=ro", uri=True) as c:
            body = c.execute('SELECT "body" FROM engine_input WHERE "path" = ?', ("data/stated.json",)).fetchone()[0]
        self.assertEqual(bytes(body), (ROOT / "matlab_sils" / "data" / "stated.json").read_bytes())
        return json.loads(bytes(body))["value"]

    def assert_stated(self, want, group, revision, source):
        """Each node of want is stated by group with the value and source the 1.0.0 code gave, and the engine's input
        (data/stated.json) gives that value."""
        stated = self.stated_file()
        for nid, v in want.items():
            _k, x = self.nodes[nid]
            rows = {(s, f): (val, o) for s, f, val, o in x["body"]["content"]}
            self.assertEqual(self.beh(nid), "stated", nid)
            self.assertEqual(x["body"]["node"]["group_id"], group, nid)
            self.assertEqual(float(rows[("value", "number")][0]), v, nid)
            self.assertEqual(stated[nid], v, nid)
            self.assertTrue(rows[("value", "source")][0].startswith(source), nid)
            self.assertEqual(rows[("value", "number")][1], f"the developer's revision {revision}", nid)

    def test_the_device_values_the_code_used_with_no_node_are_stated_as_it_gave_them(self):
        """S7.7: the product's device defaults (product.rs Dev::load) and the rotors' telemetry noises (run.rs sense), which
        the engine used with no node, are stated by act with the value and source the 1.0.0 code gave. Since S7.11 the
        engine reads them from the design (data/stated.json) and keeps no copy: product.rs names the nodes, and the rotors'
        telemetry (act_rotor_telemetry) takes the two noises as inputs."""
        self.assert_stated({"act_rw_torque_noise": 0.001, "act_rw_friction_comp": 0.95, "act_rw_drive_efficiency": 0.8,
                            "act_cmg_speed_gain": 1.0, "act_fmr_flow_gain": 2.0, "act_fmr_flow_tau": 0.3,
                            "act_rotor_tlm_noise": 1e-7, "act_gimbal_tlm_noise": 1e-5}, "act", "S7.7", "1.0.0 code: engine/crates/adcs-sim/src/")
        prod = (ROOT / "engine" / "crates" / "adcs-sim" / "src" / "product.rs").read_text(encoding="utf-8")
        self.assertNotIn("torque_noise: 0.001", prod)
        for nid in ("act_rw_torque_noise", "act_rw_friction_comp", "act_rw_drive_efficiency", "act_cmg_speed_gain", "act_fmr_flow_gain",
                    "act_fmr_flow_tau", "act_rotor_tlm_noise", "act_gimbal_tlm_noise"):
            self.assertIn(f'st.get("{nid}")', prod, nid)
        tlm = from_design.text("act/rotortlm.pc", REG / "design.tndb")
        self.assertNotIn("const ROTOR_TLM_NOISE", tlm)
        self.assertIn("h_noise: real[1]", tlm)

    def test_the_star_tracker_unit_is_methods_the_developer_transcribed(self):
        """S7.9: the star tracker's attitude from its stars (the q-method with the residual check, l3_sens_row_12), its
        synthetic onboard star table (a new node, a method: the engine's own catalogue, not a published one) and the unit
        as the engine flies it (a new node: the mounts, the latency, the exclusion and blinding, the noise and QUEST models)
        are sens's methods, each the developer's unsigned transcription, generated into adcs-sim-core. The image model's
        chain (l3_sens_row_09 to 11) is still comp.rs's (S7.10)."""
        for nid in ("l3_sens_row_12", "sens_star_catalogue", "sens_star_tracker"):
            _k, x = self.nodes[nid]
            rows = {(s, f): (v, o) for s, f, v, o in x["body"]["content"]}
            self.assertEqual(self.beh(nid), "method", nid)
            self.assertEqual(x["body"]["node"]["group_id"], "sens", nid)
            self.assertIn("the developer's revision S7.9", rows[("code", "pseudocode")][1], nid)
            self.assertTrue(rows[("code", "transcribes")][0], nid)
            self.assertEqual(rows[("code", "generate")][0], "adcs-sim-core", nid)
            self.assertEqual(x["sealed_as"], "unconfirmed", nid)
            self.assertTrue(any(w.startswith("the developer's revision S7.9") and "not yet signed by a person" in w for w in x["why"]), nid)
        self.assertNotIn("l3_sens_row_12", health.built_in(REG)["by_group"].get("sens", []))

    def test_the_star_tracker_image_chain_is_methods_the_developer_transcribed(self):
        """S7.10: the star tracker's image chain, the frame the detector reads (l3_sens_row_09), its spots (10), their
        identification by pair angles against the onboard pair table (11) and the image model as the engine flies it (a new
        node: the chain for each head that answers, over l3_sens_row_12's attitude), are sens's methods, each the developer's
        unsigned transcription, generated into adcs-sim-core over buffers whose length is the caller's (trinetra-toolbox/5).
        sens has no built-in node left."""
        for nid in ("l3_sens_row_09", "l3_sens_row_10", "l3_sens_row_11", "sens_star_image"):
            _k, x = self.nodes[nid]
            rows = {(s, f): (v, o) for s, f, v, o in x["body"]["content"]}
            self.assertEqual(self.beh(nid), "method", nid)
            self.assertEqual(x["body"]["node"]["group_id"], "sens", nid)
            self.assertIn("the developer's revision S7.10", rows[("code", "pseudocode")][1], nid)
            self.assertTrue(rows[("code", "transcribes")][0], nid)
            self.assertEqual(rows[("code", "generate")][0], "adcs-sim-core", nid)
            self.assertEqual(x["sealed_as"], "unconfirmed", nid)
            self.assertTrue(any(w.startswith("the developer's revision S7.10") and "not yet signed by a person" in w for w in x["why"]), nid)
        self.assertNotIn("sens", health.built_in(REG)["by_group"])
        comp = (ROOT / "engine" / "crates" / "adcs-sim-core" / "src" / "comp.rs").read_text(encoding="utf-8")
        for gone in ("select_nth_unstable", "sort_unstable_by", "partition_point", "exp(-("):
            self.assertNotIn(gone, comp, "the chain's relations are the design's, comp.rs only calls them")

    def test_the_product_axes_the_code_used_with_no_node_are_stated_as_it_gave_them(self):
        """S7.8: the payload boresight and the Sun axis a product flies when it states none (product.rs Dev::load), which
        the engine used with no node, are stated by gdn with the value and source the 1.0.0 code gave; since S7.11 the
        engine reads them from the design (data/stated.json)."""
        self.assert_stated({"gdn_payload_boresight_x": 0.0, "gdn_payload_boresight_y": 1.0, "gdn_payload_boresight_z": 0.0,
                            "gdn_sun_axis_x": 0.0, "gdn_sun_axis_y": 0.0, "gdn_sun_axis_z": -1.0}, "gdn", "S7.8",
                           "1.0.0 code: engine/crates/adcs-sim/src/product.rs Dev::load")
        prod = (ROOT / "engine" / "crates" / "adcs-sim" / "src" / "product.rs").read_text(encoding="utf-8")
        self.assertNotIn("boresight: [0.0, 1.0, 0.0]", prod)
        self.assertIn('st.v3("gdn_payload_boresight")', prod)
        self.assertIn('st.v3("gdn_sun_axis")', prod)

    def test_the_values_the_code_used_with_no_node_are_stated_as_it_gave_them(self):
        """S7.4: the centre-of-mass direction and the surface's constants, which adcs-sim's config.rs used with no node,
        are stated by dyn with the value and source the 1.0.0 code gave; since S7.11 the engine reads them from the
        design (data/stated.json) and config.rs keeps no copy."""
        self.assert_stated({"dyn_cm_direction_x": 0.30, "dyn_cm_direction_y": 0.70, "dyn_cm_direction_z": -0.65,
                            "dyn_surface_accommodation": 0.8, "dyn_surface_vb_ratio": 0.05, "dyn_surface_specular_share": 0.5},
                           "dyn", "S7.4", "1.0.0 code: engine/crates/adcs-sim/src/config.rs")
        cfg = (ROOT / "engine" / "crates" / "adcs-sim" / "src" / "config.rs").read_text(encoding="utf-8")
        for gone in ("0.30, 0.70", "ACCOMMODATION: f64", "VB_RATIO: f64", "SPEC_FRAC: f64"):
            self.assertNotIn(gone, cfg)
        self.assertIn('st.v3("dyn_cm_direction")', cfg)

    def test_the_descriptors_and_the_plant_are_the_designs(self):
        """S7.11: what adcs-sim's config.rs, product.rs and run.rs computed for the plant and the devices is methods (the
        centre of mass's offset, s1_4, open until now; the truth plant from the case and the plant's state at the start
        of a run, new nodes of dyn; the case's orbit and epoch, a new node of env; in their rows' modules the ring flow
        sensor's noise in momentum, the thrusters' arms, the Earth sensor's boresight, a calibrated tracker's mount errors),
        each the developer's unsigned transcription; the values they used with no node are stated by env as the 1.0.0
        code gave them; the engine reads every stated value from the design (data/stated.json, the design's stated
        values exactly) and the code holds no copy of them, nor the Earth's radius or mu."""
        targets = {"s1_4": ("dyn", "adcs-sim"), "dyn_truth_plant": ("dyn", "adcs-sim"), "env_case_orbit": ("env", "adcs-sim"),
                   "dyn_initial_state": ("dyn", "adcs-sim-core")}
        for nid, (group, target) in targets.items():
            _k, x = self.nodes[nid]
            rows = {(s, f): (v, o) for s, f, v, o in x["body"]["content"]}
            self.assertEqual(self.beh(nid), "method", nid)
            self.assertEqual(x["body"]["node"]["group_id"], group, nid)
            self.assertIn("the developer's revision S7.11", rows[("code", "pseudocode")][1], nid)
            self.assertTrue(rows[("code", "transcribes")][0], nid)
            self.assertEqual(rows[("code", "generate")][0], target, nid)
            self.assertEqual(x["sealed_as"], "unconfirmed", nid)
            self.assertTrue(any(w.startswith("the developer's revision S7.11") and "not yet signed by a person" in w for w in x["why"]), nid)
        for path, fn in (("act/ringnoise.pc", "ring_flow_noise"), ("act/rcstorque.pc", "couple_arms"), ("sens/earthsensor.pc", "es_boresight"),
                         ("sens/sttracker.pc", "st_calibrated")):
            self.assertIn(f"\nfn {fn}(", from_design.text(path, REG / "design.tndb"), path)
        self.assert_stated({"env_f107_default": 130.0, "env_f107a_default": 130.0, "env_kp_default": 2.0, "env_ap_default": 7.0,
                            "env_orbit_step": 10.0, "env_fast_zonal_degree": 6.0, "env_field_degree": 13.0}, "env", "S7.11",
                           "1.0.0 code: engine/crates/adcs-sim/src/config.rs build")
        # data/stated.json: every stated value of the design, and nothing else
        want = {}
        for nid, (_k, x) in self.nodes.items():
            rows = {(s, f): v for s, f, v, _o in x["body"]["content"]}
            if self.beh(nid) == "stated" and ("value", "number") in rows:
                want[nid] = float(rows[("value", "number")])
            elif self.beh(nid) == "stated" and ("value", "list") in rows:     # since S7.13, adcs-stated/2
                want[nid] = [float(v) for v in json.loads(rows[("value", "list")])]
        self.assertEqual(self.stated_file(), want)
        src = ROOT / "engine" / "crates" / "adcs-sim" / "src"
        cfg, prod, run = ((src / f).read_text(encoding="utf-8") for f in ("config.rs", "product.rs", "run.rs"))
        for gone in ("6378137", "3.986004418e14", "f107: 130.0", "kp: 2.0", "ap: 7.0", "orbit_step_s: 10.0", "zonal_max: 6,", "igrf_nmax: 13,",
                     "365.25", "3f64.sqrt()", ".round();"):
            self.assertNotIn(gone, cfg, gone)
        self.assertNotIn('n("arm_long_m"), n("arm_long_m")', prod)
        for fn in ("ring_flow_noise(", "couple_arms(", "st_calibrated(", "es_boresight("):
            self.assertIn(fn, prod, fn)
        for gone in ("ir.normal()", "fromrotvec(", "1.0/dot(r, r)", "w*r[1]"):
            self.assertNotIn(gone, run, gone)

    def test_the_device_codecs_scaling_is_the_drivers(self):
        """S7.12, as the owner decided: the device emulators' scaling (each sensor's counts within its field's range, each
        command word's value) is l3_oils_row_07's method, the developer's unsigned transcription, generated into
        adcs-sim-core; it takes the drivers' constants from the drivers' own functions (one count of each field by drv_read,
        the ranges by rd16 and rd32, the words' full scale by q15), so they are the protocol table's (fsw-rs devices.rs),
        and its one stated constant, the valve word's 1 ms, gives back every word drv_write writes. emu.rs keeps the
        packets (layout, sync, CRC, bus) and no scaling. oils has no built-in node left."""
        nid = "l3_oils_row_07"
        _k, x = self.nodes[nid]
        rows = {(s, f): (v, o) for s, f, v, o in x["body"]["content"]}
        self.assertEqual(self.beh(nid), "method")
        self.assertEqual(x["body"]["node"]["group_id"], "oils")
        self.assertIn("the developer's revision S7.12", rows[("code", "pseudocode")][1])
        self.assertEqual(rows[("code", "generate")][0], "adcs-sim-core")
        self.assertEqual(json.loads(rows[("code", "uses")][0]), ["fsw/pseudocode/09_drivers.pc"])
        self.assertEqual(x["sealed_as"], "unconfirmed")
        self.assertNotIn("oils", health.built_in(REG)["by_group"])
        import tempfile, pcode
        with tempfile.TemporaryDirectory() as d:
            files = []
            for p in ("fsw/pseudocode/01_math.pc", "fsw/pseudocode/02_time_frames_models.pc", "fsw/pseudocode/02_igrf13.pc",
                      "fsw/pseudocode/09_drivers.pc", "oils/emucodec.pc"):
                f = pathlib.Path(d) / pathlib.PurePosixPath(p).name
                f.write_text((ROOT / p).read_text(encoding="utf-8") if p.startswith("fsw/pseudocode/01") else from_design.text(p, REG / "design.tndb"),
                             encoding="utf-8")   # the toolbox is the repository's
                files.append(f)
            # every valve word k drv_write writes for the duty valve_duty gives it, at two control steps
            (pathlib.Path(d) / "valves.pc").write_text("""module valvecheck
fn valve_round_trip(dt: real[1] in 0.05 .. 0.5) -> bad: int
    let s = emu_scale()
    bad = 0
    for k in 0 .. 256
        let duty: real[1][6] = 0
        duty[0] = valve_duty(k, dt*(1 [s]), s)
        let cr: real[1][8] = 0
        let tm: real[1][8] = 0
        let cg: vec4[1] = 0
        let pwm, rot, gim, valves = drv_write([0.0, 0.0, 0.0], 0.2, cr, tm, 0, cg, 1.0, 0, duty, 1, dt)
        if valves[0] != k
            bad = bad + 1
        end
    end
end
""", encoding="utf-8")
            files.append(pathlib.Path(d) / "valves.pc")
            s = pcode.cli("run", *files, "--fn", "emu_scale", "--args", "[]")[0]
            for dt in (0.1, 0.25):
                self.assertEqual(pcode.cli("run", *files, "--fn", "valve_round_trip", "--args", json.dumps([dt])), [0], dt)
        dev = (ROOT / "fsw-rs" / "src" / "devices.rs").read_text(encoding="utf-8")
        const = {k: eval(v) for k, v in re.findall(r"pub const (\w+): f64 = ([0-9.e/+-]+);", dev)}
        want = {"mag": const["MAG_LSB_T"], "gyro": const["GYRO_LSB"], "unit": 1.0/const["Q15"], "q": 1.0/const["Q30"], "pos": 0.01, "vel": 0.001,
                "h": const["H_LSB"], "delta": const["DELTA_LSB"], "word": const["Q15"], "valve": const["VALVE_LSB_S"],
                "lo16": -32768, "hi16": 32767, "lo32": -2147483648, "hi32": 2147483647}
        self.assertEqual(s, want)
        emu = (ROOT / "engine" / "crates" / "adcs-sim-core" / "src" / "emu.rs").read_text(encoding="utf-8")
        emu = "\n".join(ln.split("//")[0] for ln in emu.splitlines())     # the code, not its comments
        for gone in ("MAG_LSB_T", "GYRO_LSB", "Q30", "Q15", "H_LSB", "DELTA_LSB", "VALVE_LSB_S", "fn q16", "fn q32", "/0.01", "/0.001"):
            self.assertNotIn(gone, emu, gone)

    def test_the_flight_parameters_are_the_designs(self):
        """S7.13 (closes S6's finding): every one of the 147 fsw_param_* nodes has its value from the design. 77 are methods
        of fsw, the developer's unsigned transcriptions generated into adcs-sim (the laws, the choices of law, the copies of
        the product's, the case's and the constants' values); 67 are stated as the 1.0.0 code gave them (the constants, and
        the value a run takes when its scenario states none); the schedule's three state none (a run whose scenario states
        no schedule flies none). The tuning a law takes when the scenario states none (fsw_tune_*) and the run's defaults
        the engine held with no node are stated too; the engine reads them from data/stated.json, and config.rs holds no
        law and no constant of the flight software."""
        params = sorted(n for n in self.nodes if n.startswith("fsw_param_"))
        self.assertEqual(len(params), 147)
        stated = self.stated_file()
        methods, values, none = [], [], []
        for nid in params:
            _k, x = self.nodes[nid]
            rows = {(s, f): (v, o) for s, f, v, o in x["body"]["content"]}
            self.assertEqual(x["body"]["node"]["group_id"], "fsw", nid)
            if self.beh(nid) == "method":
                methods.append(nid)
                self.assertIn("the developer's revision S7.13", rows[("code", "pseudocode")][1], nid)
                self.assertTrue(rows[("code", "transcribes")][0], nid)
                self.assertEqual(rows[("code", "generate")][0], "adcs-sim", nid)
                self.assertEqual(x["sealed_as"], "unconfirmed", nid)
                self.assertTrue(any(w.startswith("the developer's revision S7.13") and "not yet signed by a person" in w for w in x["why"]), nid)
            else:
                self.assertEqual(self.beh(nid), "stated", nid)
                src = rows[("value", "source")][0]
                self.assertTrue(src.startswith(("1.0.0 code: engine/crates/adcs-sim/src/config.rs", "the scenario's ")), nid)
                self.assertIn("engine/crates/adcs-sim/src/config.rs", src, nid)
                if ("value", "number") in rows or ("value", "list") in rows:
                    values.append(nid)
                    self.assertIn(nid, stated, nid)
                else:
                    none.append(nid)
        self.assertEqual((len(methods), len(values)), (77, 67))
        self.assertEqual(none, ["fsw_param_n_sched", "fsw_param_sched_mode", "fsw_param_sched_t"])
        self.assert_stated({"fsw_param_mtq_period": 1.0, "fsw_param_mtq_meas": 0.2, "fsw_param_mtq_eps": 1e-3, "fsw_param_mtq_phi": 5e-4,
                            "fsw_param_rw_phi": 2e-4,
                            "fsw_tune_bdot_gain_scale": 3.0, "fsw_tune_mtq_wn": 0.005, "fsw_tune_avanzini_k_over_n": 0.84,
                            "fsw_tune_rw_bandwidth": 0.9, "fsw_tune_rw_rate_hz": 10.0, "fsw_tune_ss_gain": 1.0},
                           "fsw", "S7.13", "1.0.0 code: engine/crates/adcs-sim/src/config.rs")
        self.assert_stated({"fsw_param_detumble_hold_s": 60.0, "fsw_param_dt": 0.1, "fsw_param_mekf_gate": 16.27, "fsw_param_start_mode": 0.0,
                            "fsw_param_auto_next": 255.0}, "fsw", "S7.13", "the scenario's ")
        self.assert_stated({"env_density_scale_default": 1.0, "env_force_drag": 1.0, "env_torque_magnetic": 1.0, "env_refresh_step": 1.0,
                            "env_start_arg_lat_default": 0.0}, "env", "S7.13", "1.0.0 code: engine/crates/adcs-sim/src/config.rs")
        self.assert_stated({"vv_run_duration_default": 600.0, "vv_record_step_default": 1.0}, "vv", "S7.13", "1.0.0 code: engine/crates/adcs-sim/src/config.rs")
        self.assertEqual((stated["dyn_initial_error_axis"], stated["dyn_initial_rate_value"]), ([1.0, 0.0, 0.0], [0.0, 0.0, 0.0]))
        self.assertEqual(len([n for n in self.nodes if n.startswith("fsw_tune_")]), 16)
        for path, fn in (("fsw/params/fswchoice.pc", "fsw_mtq_law"), ("fsw/params/fswmtq.pc", "fsw_mtq_lqr_weights"), ("fsw/params/fswrw.pc", "fsw_rw_pid"),
                         ("fsw/params/fswbody.pc", "fsw_gg_feedforward"), ("fsw/params/fswsens.pc", "fsw_mekf_sigmas"), ("fsw/params/fswrotor.pc", "fsw_rotor_targets")):
            self.assertIn(f"\nfn {fn}(", from_design.text(path, REG / "design.tndb"), path)
        src = ROOT / "engine" / "crates" / "adcs-sim" / "src"
        cfg = (src / "config.rs").read_text(encoding="utf-8")
        cfg = "\n".join(ln.split("//")[0] for ln in cfg.splitlines())     # the code, not its comments
        for gone in ("0.84", "0.08", "0.15*", "[1e-12", "7.0/15.0", "th = ", "5e-4", "2e-4", "1e6", "wn*wn", "0.5*inertia", "1.0/json", "PI/180",
                     "gg_stable", '"sunspin_damped" =>', '"mtq_lqr" =>', "json::f(&fsw, \"", "unwrap_or([0.0, 0.0, 0.0, 1.0])"):
            self.assertFalse(gone in cfg, gone)
        self.assertEqual(re.findall(r"json::f\([^;()\n]*, -?[0-9][0-9.e-]*\)", cfg), ['json::f(e, "t_s", 0.0)', 'json::f(f, "t_s", 0.0)', 'json::f(f, "index", 0.0)'],
                         "a scenario's number with a default in code: only the events' clock origin and the fault's unit")
        self.assertIn("lqr::chain3(", cfg)       # the Riccati solve stays the toolbox's

    def test_the_pointing_budget_jitter_and_power_are_methods_the_developer_transcribed(self):
        """S7.14: the pointing budget's knowledge, control and alignment contributions (gp_0, gp_1, gp_2) and its total, room
        and verdict (l3_pnt_row_09, open until now) are pnt's methods, the rotors' jitter with each rotor's imbalance gp_4's,
        and the platform's power system (arrays, battery, the solar constant) a new node of design; each the developer's
        unsigned transcription, generated into the engine (adcs-sim). tools/pointing_budget.py and the engine's metrics.rs
        and product.rs keep no relation of them: the tool asks the engine (adcs results budget)."""
        moved = {"gp_0": "built-in", "gp_1": "built-in", "gp_2": "built-in", "gp_4": "built-in", "l3_pnt_row_09": "open", "design_power_system": None}
        for nid in moved:
            _k, x = self.nodes[nid]
            rows = {(s, f): (v, o) for s, f, v, o in x["body"]["content"]}
            self.assertEqual(self.beh(nid), "method", nid)
            self.assertIn("the developer's revision S7.14", rows[("code", "pseudocode")][1], nid)
            self.assertTrue(rows[("code", "transcribes")][0], nid)
            self.assertEqual(rows[("code", "generate")][0], "adcs-sim", nid)
            self.assertEqual(x["sealed_as"], "unconfirmed", nid)
            self.assertTrue(any(w.startswith("the developer's revision S7.14") and "not yet signed by a person" in w for w in x["why"]), nid)
        self.assertEqual(self.nodes["design_power_system"][1]["body"]["node"]["group_id"], "design")
        for path, fn in (("pnt/pntbudget.pc", "pointing_control"), ("pnt/pntbudget.pc", "pointing_budget"), ("pnt/jitter.pc", "rotor_jitter"),
                         ("pnt/jitter.pc", "rotor_imbalance"), ("design/powersys.pc", "array_power"), ("design/powersys.pc", "battery_step")):
            self.assertIn(f"\nfn {fn}(", from_design.text(path, REG / "design.tndb"), path)
        code = lambda rel: "\n".join(ln.split("//")[0] for ln in (ROOT / rel).read_text(encoding="utf-8").splitlines())
        met = code("engine/crates/adcs-sim/src/metrics.rs")
        for gone in ("1361", "powi(2)", "to_degrees()*3600", "(us*d + ud)", ".clamp(0.0, ps.batt_wh)", "\"rw_bandwidth\", 0.9"):
            self.assertNotIn(gone, met, gone)
        self.assertNotIn('("rings", _) => Some((0.0, 0.0))', code("engine/crates/adcs-sim/src/product.rs"))
        tool = (ROOT / "tools" / "pointing_budget.py").read_text(encoding="utf-8")
        for gone in ("math.sqrt", "math.degrees", "/3600", "def rss", "def control_part"):
            self.assertNotIn(gone, tool, gone)
        self.assertIn('"results", "budget"', tool)

    def test_the_metrics_are_kpis_methods_the_developer_transcribed(self):
        """S7.14b (the owner's ruling of 7 Oct 2026: the metrics are relations of kpi): how each metric is measured from a
        run, its channels, windows, statistics, the ECSS indices, each kind's value, unit, defaults and verdict, are the
        methods of four new nodes of kpi, each the developer's unsigned transcription of adcs-sim metrics.rs, generated
        into the engine (the channels into adcs-sim-core too, for the three-axis angle's portable maths). metrics.rs keeps
        no relation: it walks the record, reads the scenario's metric sections and hands the channels over; the choices'
        options are the scenario's words in the schema's order."""
        mods = {"kpi_metric_channels": ("kpi/kpichannels.pc", "adcs-sim, adcs-sim-core"), "kpi_metric_statistics": ("kpi/kpistats.pc", "adcs-sim"),
                "kpi_metric_ecss": ("kpi/kpiecss.pc", "adcs-sim"), "kpi_metric_evaluate": ("kpi/kpimetrics.pc", "adcs-sim")}
        for nid, (path, gen) in mods.items():
            _k, x = self.nodes[nid]
            rows = {(s, f): (v, o) for s, f, v, o in x["body"]["content"]}
            self.assertEqual(x["body"]["node"]["group_id"], "kpi", nid)
            self.assertEqual(self.beh(nid), "method", nid)
            self.assertTrue(rows[("code", "pseudocode")][1].startswith(path + " (the developer's revision S7.14b"), nid)
            self.assertTrue(rows[("code", "transcribes")][0], nid)
            self.assertEqual(rows[("code", "generate")][0], gen, nid)
            self.assertEqual(x["sealed_as"], "unconfirmed", nid)
            self.assertTrue(any(w.startswith("the developer's revision S7.14b") and "not yet signed by a person" in w for w in x["why"]), nid)
        src = ROOT / "engine" / "crates" / "adcs-sim" / "src"
        met = "\n".join(ln.split("//")[0] for ln in (src / "metrics.rs").read_text(encoding="utf-8").splitlines())
        for gone in ("0.9973", "0.95", "1e-9", "sort_by", ".acos()", "to_degrees", "qangle", "/60.0", "100.0*", "1e3*", "fold(", ".sum::<f64>()",
                     "\"rate_threshold_deg_s\", 0.5", "\"hold_s\", 10.0", "\"threshold_deg\", 1.0", "adcs_fsw::guid", "Guid {", "clamp("):
            self.assertFalse(gone in met, gone)
        schema = (src / "schema.rs").read_text(encoding="utf-8")
        words = lambda name: re.findall(r'"([^"]+)"', re.search(rf"pub const {name}: \[&str; \d+\] = \[(.*?)\];", schema, re.S).group(1))
        gen = lambda mod, choice: [k.lower() for k, _v in sorted(re.findall(rf"pub const {choice}_(\w+): i64 = (\d+);", (src / "gen" / f"{mod}.rs").read_text(encoding="utf-8")), key=lambda x: int(x[1]))]
        self.assertEqual(gen("kpimetrics", "METRICKIND"), words("METRIC_KINDS") + ["none"])
        self.assertEqual(gen("kpistats", "METRICWINDOW"), words("WINDOWS") + ["after_s"])
        self.assertEqual(gen("kpistats", "METRICSTAT"), [w.replace(".", "") for w in words("STATISTICS")] + ["none"])
        self.assertEqual(gen("kpimetrics", "METRICCHANNEL"), words("CHANNELS"))
        units = re.findall(r'"([^"]+)"', re.search(r"const UNITS: \[&str; \d+\] = \[(.*?)\];", met).group(1))
        self.assertEqual(len(units), len(gen("kpimetrics", "METRICUNIT")))

    def test_the_sizing_is_methods_the_developer_transcribed(self):
        """S7.15: the sizing's 27 built-in nodes (act's gm_4, gm_5, gw_2, gw_5, gw_6, l3_fmr_row_07, 14, 15; design's
        sizing, budget and survey rows) are methods of design and act, the catalogue's derive rule a new node of catalogue
        and the design loop's rules three new nodes of design (the owner's word), each the developer's unsigned
        transcription, generated into adcs-design. adcs-design keeps the survey's step order and the files' formats, and
        the tools ask the engine (adcs design derive, adcs design call): none keeps a law of its own."""
        mods = {"design/sizedemand.pc": ["l3_budget_row_01", "gw_2"],
                "design/sizemtq.pc": ["design_sizing_mtq", "l3_budget_row_02", "gm_4", "gm_5"],
                "design/sizerotor.pc": ["design_sizing_rw", "design_sizing_cmg", "design_sizing_vscmg", "l3_budget_row_03", "gw_5", "gw_6"],
                "act/sizepump.pc": ["l3_fmr_row_07"], "act/sizering.pc": ["l3_fmr_row_14", "l3_fmr_row_15"],
                "design/sizefmr.pc": ["design_sizing_fmr", "l3_budget_row_04"], "design/sizercs.pc": ["design_sizing_rcs", "l3_budget_row_05"],
                "design/sizebudget.pc": ["l3_budget_row_06", "l3_budget_row_07", "l3_budget_row_08", "gb_0", "gb_1", "gb_2", "gb_3"],
                "design/sizesensors.pc": ["design_sizing_sensors"], "catalogue/catderive.pc": ["catalogue_datasheet_derive"],
                "design/looprules.pc": ["design_loop_converge", "design_loop_robustness", "design_loop_redundancy"]}
        self.assertEqual(sum(len(v) for k, v in mods.items() if k not in ("catalogue/catderive.pc", "design/looprules.pc")), 27)
        for path, nids in mods.items():
            for nid in nids:
                _k, x = self.nodes[nid]
                rows = {(s, f): (v, o) for s, f, v, o in x["body"]["content"]}
                self.assertEqual(self.beh(nid), "method", nid)
                self.assertTrue(rows[("code", "pseudocode")][1].startswith(path + " (the developer's revision S7.15"), nid)
                self.assertTrue(rows[("code", "transcribes")][0], nid)
                self.assertEqual(rows[("code", "generate")][0], "adcs-design", nid)
                self.assertEqual(x["sealed_as"], "unconfirmed", nid)
                self.assertTrue(any(w.startswith("the developer's revision S7.15") and "not yet signed by a person" in w for w in x["why"]), nid)
        self.assertEqual(self.nodes["catalogue_datasheet_derive"][1]["body"]["node"]["group_id"], "catalogue")
        for nid in mods["design/looprules.pc"]:
            self.assertEqual(self.nodes[nid][1]["body"]["node"]["group_id"], "design", nid)
        code = lambda rel: "\n".join(ln.split("//")[0] for ln in (ROOT / rel).read_text(encoding="utf-8").splitlines())
        src = ROOT / "engine" / "crates" / "adcs-design" / "src"
        self.assertFalse((src / "empump.rs").exists())
        lib = code("engine/crates/adcs-design/src/lib.rs").split("#[cfg(test)]")[0]       # the code, not its tests
        for gone in ("0.45", "4.5", "6440", "9.80665", "745.0", "2810", "70e5", "250e6", "0.8165", "0.9063", "91.3", "250.0", "5740", "0.25",
                     "2e-4", "0.999", "0.006", "0.8*", "1.2*", "fold(", ".max(", ".min(", "powf", "powi", ".sqrt()", "PI", "sort_by", "atan2"):
            self.assertNotIn(gone, lib, gone)
        cat = (ROOT / "tools" / "catalogue.py").read_text(encoding="utf-8")
        for gone in ("math.", "RPM", "ASSUMED_RPM", "1e-3", "re.findall", "def wheel_motor", "def volume_L", "(1 - top"):
            self.assertNotIn(gone, cat, gone)
        self.assertIn('"design", "derive"', cat)
        # the notes' and the assumptions' words name the methods' constants, as the code wrote them
        self.assertIn("a quarter orbit of secular momentum held between dumps", lib)
        self.assertIn("\n    orbits_held = 0.25\n", from_design.text("design/sizedemand.pc", REG / "design.tndb"))
        rules = from_design.text("catalogue/catderive.pc", REG / "design.tndb")
        rule = lambda name: float(re.search(rf"\n    {name} = ([0-9.e-]+)\n", rules).group(1))
        for name, words in (("balance", "G2.5"), ("coulomb", "Coulomb 1e-5 sqrt"), ("coulomb_ref", "h / 0.01 Nms"), ("viscous", "viscous 1e-8 N m s"),
                            ("peak_efficiency", "speed / 0.5"), ("supply", "motor supply 5 V"), ("tuna_volume", "volume: 0.2 L per unit")):
            self.assertIn(words, cat, name)
            self.assertEqual(rule(name), {"balance": 2.5e-3, "coulomb": 1e-5, "coulomb_ref": 0.01, "viscous": 1e-8, "peak_efficiency": 0.5,
                                          "supply": 5.0, "tuna_volume": 0.2}[name], name)
        loop = (ROOT / "tools" / "pipeline_design.py").read_text(encoding="utf-8") + (ROOT / "tools" / "pipeline_verify.py").read_text(encoding="utf-8")
        for gone in ("round(g0", "lam * 3", "sg / 4", "IMPROVE *", "s * UP", "s * DOWN", "s0 * DOWN", "GYRO_MIN * 1.01", "def node_robust(sel, fails, knobs, history):\n    \"\"\"Node mc's feedback: the Monte Carlo of the dispatched mission fails a requirement in some\n    dispersed run. The selected family's lever for that failure class is moved and the lever that\n    works against it is closed, then the loop runs on from these knobs.\"\"\""):
            self.assertNotIn(gone, loop, gone)
        base = (ROOT / "tools" / "pipeline_base.py").read_text(encoding="utf-8")
        for gone in ("def auth_part", "def rate_violation", "def fam_violation"):
            self.assertNotIn(gone, base, gone)
        # the choices' options are the words the code reads and writes, in order
        gen = lambda mod, choice: [k.lower() for k, _v in sorted(re.findall(rf"pub const {choice}_(\w+): i64 = (\d+);", (src / "gen" / f"{mod}.rs").read_text(encoding="utf-8")), key=lambda x: int(x[1]))]
        import pipeline_design
        self.assertEqual(gen("looprules", "PART"), pipeline_design.PARTS)
        self.assertEqual(gen("looprules", "ACTUATOR"), pipeline_design.ACTUATORS)
        self.assertEqual(gen("looprules", "LOOPEVENT"), pipeline_design.EVENTS)
        self.assertEqual(gen("looprules", "MASSLEVER"), [w.replace(".", "_") for w in pipeline_design.LEVERS])
        slots = re.findall(r'"(\w+)" => gb::SLOT_(\w+)', lib)
        self.assertEqual([w for w, _c in slots], [c.lower() for _w, c in slots])
        self.assertEqual([w for w, _c in slots], gen("sizebudget", "SLOT")[:-1])
        self.assertEqual(gen("sizedemand", "SURVEYATTITUDE"), [w.lower() for w in re.findall(r'"(\w+)"', re.search(r"const ATT: \[&str; 4\] = \[(.*?)\];", lib).group(1))])
        self.assertEqual(gen("sizedemand", "DIST"), ["normal", "uniform"])

    def test_the_floquet_certificate_is_ctls_method_the_developer_transcribed(self):
        """S7.15b (decision 3, as proposed): the Floquet certificate is an analysis method of ctl, a new node, the developer's
        unsigned transcription of tools/floquet.py, generated into adcs-design; its multipliers are the toolbox's eig
        (trinetra-toolbox/6). The tool reads, asks the engine (adcs design call) and writes: it keeps no law of its own."""
        _k, x = self.nodes["ctl_floquet_certificate"]
        rows = {(s, f): (v, o) for s, f, v, o in x["body"]["content"]}
        self.assertEqual(x["body"]["node"]["group_id"], "ctl")
        self.assertEqual(self.beh("ctl_floquet_certificate"), "method")
        self.assertTrue(rows[("code", "pseudocode")][1].startswith("ctl/floquet.pc (the developer's revision S7.15b"))
        self.assertIn("tools/floquet.py", rows[("code", "transcribes")][0])
        self.assertEqual(rows[("code", "generate")][0], "adcs-design")
        self.assertEqual(x["sealed_as"], "unconfirmed")
        self.assertTrue(any(w.startswith("the developer's revision S7.15b") and "not yet signed by a person" in w for w in x["why"]))
        pc = from_design.text("ctl/floquet.pc", REG / "design.tndb")
        self.assertIn("eig(phi)", pc)
        self.assertIn("\nconst FLOQUET_STEPS = 1500\n", pc)
        self.assertTrue((ROOT / "engine" / "crates" / "adcs-design" / "src" / "gen" / "floquet.rs").is_file())
        tool = (ROOT / "tools" / "floquet.py").read_text(encoding="utf-8").split('"""', 2)[2]     # the code, not its words
        for gone in ("numpy", "np.", "math.", "def monodromy", "def law_torque", "def dcm", "def rv2q", "def body_from_orbit", "3.986004418e14",
                     "6378137", "1500", "1e-7", "< 1.0", "eigvals", "radians"):
            self.assertNotIn(gone, tool, gone)
        self.assertIn('"floquet::fq_certify"', tool)

    def test_every_behaviour_is_one_the_schema_knows(self):
        self.assertEqual(tndb.check(REG / "design.tndb"), [])
        known = set(tndb.schema()["behaviours"])
        self.assertEqual({self.beh(n) for n in self.nodes} - known, set())


if __name__ == "__main__":
    unittest.main()
