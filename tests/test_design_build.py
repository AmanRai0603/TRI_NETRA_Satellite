"""Today's design runs (docs/PLAN_2_0.md S4): tools/design_build.py builds the one database every
program reads from the converted drive, and what the engine reads from it is what it reads today:
  - every engine input file, generated from the design's blocks and case files, is the repository's
    data file byte for byte, and no file is missing or extra;
  - every case the engine flies has the same lines, and the inputs' fingerprint is the repository's;
  - the database checks, names its toolbox and application, and the releases it used;
  - a refused release changes nothing: the group keeps its last good release, and says so;
  - the flight software's parameter table, read from its nodes, is fsw/params/params.toml and gives the
    same generated C and Rust; every scenario's parameter blob from the design is the files' byte for byte;
  - scenarios flown from the design alone (an empty data folder) give the files' result id and metrics;
  - the design loop is declared on its block, and any other cycle in the wires is refused by name;
  - the health map: every node's health rolls up to its group and the ADCS; every closure has an answer
    and a range verdict; a blocked one names the node that causes it; nothing converted shows as signed;
  - building twice gives the same engine inputs.

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import json
import os
import pathlib
import shutil
import sqlite3
import subprocess
import tempfile
import tomllib
import unittest

import _path  # puts tools/ on the import path
import carry_over
import convert_2_0 as conv
import design_build
import design_inputs
import gen_fsw_params
import health
import seed_design
import tndb

_ = _path  # imported for its effect: tools/ on sys.path
ROOT = pathlib.Path(__file__).resolve().parents[1]
ADCS = ROOT / "engine" / "target" / "release" / "adcs"
HAVE_ENGINE = ADCS.is_file() and os.access(ADCS, os.X_OK)


class DesignBuild(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.TemporaryDirectory()
        d = pathlib.Path(cls.tmp.name)
        src = d / "src"
        seed_design.seed(src, sync=False)
        carry_over.carry(src)
        cls.drive = d / "drive"
        conv.convert(cls.drive, src)
        cls.db = d / "design.tndb"
        cls.export = d / "export"
        cls.summary = design_build.build(cls.drive, cls.db, export=cls.export)
        cls.rows, cls.files, cls.fp = design_inputs.gather()

    @classmethod
    def tearDownClass(cls):
        cls.tmp.cleanup()

    def held(self, sql):
        with sqlite3.connect(self.db) as c:
            return c.execute(sql).fetchall()

    def test_every_engine_input_is_the_repositorys(self):
        mine = {p: b for p, b in self.held('SELECT "path", "body" FROM engine_input')}
        want = {p: b for p, _f, b in self.files}
        self.assertEqual(sorted(mine), sorted(want))
        self.assertEqual([p for p in want if mine[p] != want[p]], [])
        for p, b in want.items():
            self.assertEqual((self.export / p).read_bytes(), b, p)

    def test_every_flown_case_is_the_repositorys_and_so_is_the_fingerprint(self):
        mine = sorted(self.held("SELECT * FROM design_case"))
        self.assertEqual(mine, sorted(self.rows))
        for f in sorted((design_inputs.DATA / "cases").glob("*.csv")):
            self.assertEqual((self.export / "cases" / f.name).read_bytes(), f.read_bytes(), f.name)
        meta = dict(self.held('SELECT "key", "value" FROM meta'))
        self.assertEqual(meta["inputs_fingerprint"], self.fp)
        self.assertEqual(design_inputs.differences(self.db), [])

    def test_the_database_checks_and_says_what_it_needs(self):
        self.assertEqual(tndb.check(self.db), [])
        meta = dict(self.held('SELECT "key", "value" FROM meta'))
        self.assertEqual(meta["toolbox"], design_inputs.TOOLBOX)
        self.assertEqual(meta["needs_application"], design_inputs.needs_application())
        self.assertEqual(meta["design_kind"], "today")
        self.assertEqual(self.summary["groups"], 21)
        self.assertEqual(self.summary["refused"], [])
        self.assertEqual(self.held("SELECT count(*) FROM design_node")[0][0], self.summary["nodes"])

    def test_a_refused_release_keeps_the_last_good_one(self):
        d = pathlib.Path(self.tmp.name) / "refused"
        shutil.copytree(self.drive, d)
        rels = sorted((d / "groups" / "attitude").glob("releases/*.tnrel")) or sorted(d.glob("groups/*/releases/*.tnrel"))
        good = rels[0]
        bad = good.with_name(good.name.replace("-0.1.", "-0.2."))
        shutil.copy(good, bad)
        with sqlite3.connect(bad) as c:
            c.execute("UPDATE release_node SET content = '{}' WHERE rowid = (SELECT min(rowid) FROM release_node)")
        used = design_build.releases_used(d)
        g = good.parents[1].name
        self.assertEqual(used[g][0].name, good.name)
        self.assertTrue(any(bad.name in x for x in used[g][1]), used[g][1])
        s = design_build.build(d, d / "design.tndb")
        self.assertTrue(any(bad.name in x for x in s["refused"]))
        with sqlite3.connect(d / "design.tndb") as c:
            self.assertEqual(sorted(c.execute('SELECT "path", "body" FROM engine_input')),
                             sorted(self.held('SELECT "path", "body" FROM engine_input')))

    def test_building_again_gives_the_same_inputs(self):
        again = pathlib.Path(self.tmp.name) / "again.tndb"
        design_build.build(self.drive, again)
        self.assertEqual(again.read_bytes(), self.db.read_bytes(), "two builds of the same drive are the same bytes")
        with sqlite3.connect(again) as c:
            self.assertEqual(sorted(c.execute("SELECT * FROM engine_input")), sorted(self.held("SELECT * FROM engine_input")))
            self.assertEqual(dict(c.execute("SELECT * FROM meta"))["inputs_fingerprint"], self.fp)


    def test_the_flight_parameter_table_comes_from_its_nodes(self):
        layout = json.loads(dict(self.held('SELECT "key", "value" FROM meta'))["flight_layout"])
        self.assertEqual(layout, tomllib.loads((ROOT / "fsw" / "params" / "params.toml").read_text()))
        want = gen_fsw_params.outputs()
        try:
            gen_fsw_params.use(layout)
            self.assertEqual(gen_fsw_params.outputs(), want)
        finally:
            gen_fsw_params.use(tomllib.loads((ROOT / "fsw" / "params" / "params.toml").read_text()))

    def engine(self, *args, from_design):
        bare = pathlib.Path(self.tmp.name) / "bare"
        bare.mkdir(exist_ok=True)
        env = dict(os.environ, TRINETRA_STORE=str(pathlib.Path(self.tmp.name) / ("store_db" if from_design else "store_files")))
        env.pop("TRINETRA_DESIGN", None)
        if from_design:
            env.update(ADCS_ROOT=str(bare), TRINETRA_DESIGN=str(self.db))
        r = subprocess.run([str(ADCS), *args], env=env, capture_output=True, text=True, timeout=600, cwd=ROOT)
        self.assertEqual(r.returncode, 0, r.stderr[-1000:])

    @unittest.skipUnless(HAVE_ENGINE, "the engine is not built (engine/target/release/adcs)")
    def test_every_parameter_blob_from_the_design_is_the_files(self):
        d = pathlib.Path(self.tmp.name)
        for sc in sorted(p.stem for p in (design_inputs.DATA / "data" / "scenarios").glob("*.json")):
            self.engine("params", sc, "--out", str(d / "a.bin"), from_design=False)
            self.engine("params", sc, "--out", str(d / "b.bin"), from_design=True)
            self.assertEqual((d / "a.bin").read_bytes(), (d / "b.bin").read_bytes(), sc)

    @unittest.skipUnless(HAVE_ENGINE, "the engine is not built (engine/target/release/adcs)")
    def test_scenarios_flown_from_the_design_give_the_same_runs(self):
        d = pathlib.Path(self.tmp.name)
        for sc in ("detumble_ais", "nadir_hold_ais", "fine_hold_img", "slew_cmg"):
            for fsw in ("c", "rust"):
                got = []
                for way in (False, True):
                    out = d / f"run_{sc}_{fsw}_{way}"
                    self.engine("run", sc, "--fsw", fsw, "--out", str(out), "-q", from_design=way)
                    m = json.loads((out / "manifest.json").read_text())
                    got.append((m["result_id"], m["metrics"]))
                    shutil.rmtree(out)
                self.assertEqual(got[0], got[1], f"{sc} {fsw}")


    def test_the_design_loop_is_declared_and_no_other_cycle_runs(self):
        meta = dict(self.held('SELECT "key", "value" FROM meta'))
        loops = json.loads(meta["loops"])
        self.assertEqual([(x["block"], x["id"], x["settles"]) for x in loops], [("gb", "design_loop", ["mass", "inertia", "demand"])])
        self.assertEqual(json.loads(meta["cycles_refused"]), [])

    def test_an_undeclared_cycle_is_refused_by_name(self):
        def node(nid, parent, reads=(), loop=()):
            return {"block": [[nid, parent, "subsystem", "method", None, None]], "input": [[f"x{i}", r, "y", None] for i, r in enumerate(reads)],
                    "loop": list(loop)}
        bodies = {"top": node("top", None), "box": node("box", "top"), "a": node("a", "box", ["b"]), "b": node("b", "box", ["a"]), "c": node("c", "top", ["a"])}
        _loops, refused = design_build.loop_check(bodies, [])
        self.assertEqual(len(refused), 1)
        self.assertIn("a -> b", refused[0])
        self.assertIn("box", refused[0])
        bodies["box"]["loop"] = [["settle_ab", json.dumps(["y"]), 1e-6, 20, "a and b settle together"]]
        loops, refused = design_build.loop_check(bodies, [])
        self.assertEqual(refused, [])
        self.assertEqual(loops[0]["id"], "settle_ab")
        bodies["top"]["loop"], bodies["box"]["loop"] = bodies["box"]["loop"], []
        self.assertEqual(len(design_build.loop_check(bodies, [])[1]), 1, "a loop belongs to the smallest block that contains it")


    def test_the_health_map(self):
        d = pathlib.Path(self.tmp.name) / "health"
        d.mkdir(exist_ok=True)
        shutil.copy(self.db, d / "design.tndb")
        for case in ("ais_3u", "ais_img_3u"):
            h = health.health(d, case)
            ids = {n["id"] for n in h["nodes"]}
            self.assertEqual(len(ids), self.summary["nodes"])
            for g in h["groups"]:
                mine = [n["health"] for n in h["nodes"] if n["group"] == g["group"]]
                self.assertEqual(g["health"], health.worst(mine), g["group"])
            self.assertEqual(h["adcs"], health.worst([g["health"] for g in h["groups"]]))
            self.assertEqual([n["id"] for n in h["nodes"] if n["health"] == "refused"], [], "every computed row lies in its own range")
            for n in h["nodes"]:
                if n["health"] in ("closes", "tight"):
                    self.fail(f"{n['id']}: shows {n['health']}, but its release is a converted baseline nobody has signed")
            answered = [c for c in h["closures"] if c["answer"] != "blocked"]
            self.assertEqual(sorted(c["id"] for c in answered), sorted(f"kpi_{k}_verified" for k in (
                "absolute_pointing_error_ape", "absolute_knowledge_error_ake", "detumble_time", "adcs_orbit_average_power")))
            for c in answered:
                self.assertEqual(c["answer"], "pass", c["id"])
                self.assertIn(c["verdict"], ("whole", "part"), c["id"])
                self.assertTrue(any("Monte Carlo" in x for x in c["range_from"]), c["id"])
            for c in h["closures"]:
                if c["verdict"] == "blocked":
                    self.assertTrue(c["cause"], c["id"])
                    self.assertTrue(all(x["node"] in ids for x in c["cause"]), c["id"])
            for c in answered:
                if c["tornado"]:
                    widths = [t["width"] for t in c["tornado"]]
                    self.assertEqual(widths, sorted(widths, reverse=True), "the widest bar first")


    def test_a_broken_node_is_traced_to_by_name(self):
        # m2_4 (the orbit radius from the altitude) computes today; its method broken, it refuses, and every node
        # that reads it, at any depth, is blocked and names m2_4 as its cause
        d = pathlib.Path(self.tmp.name) / "broken"
        d.mkdir(exist_ok=True)
        shutil.copy(self.db, d / "design.tndb")
        with sqlite3.connect(d / "design.tndb") as c:
            x = json.loads(c.execute("SELECT content FROM design_node WHERE id = 'm2_4'").fetchone()[0])
            x["body"]["content"] = [r if (r[0], r[1]) != ("code", "pseudocode") else [r[0], r[1], r[2].replace("fn ", "fn (", 1), r[3]]
                                    for r in x["body"]["content"]]
            c.execute("UPDATE design_node SET content = ? WHERE id = 'm2_4'", (json.dumps(x),))
        h = health.health(d, "ais_3u")
        by = {n["id"]: n for n in h["nodes"]}
        self.assertEqual(by["m2_4"]["health"], "refused", by["m2_4"]["why"])
        downstream = [n for n in h["nodes"] if n["health"] == "blocked" and "m2_4" in n["cause"]]
        self.assertGreaterEqual({n["id"] for n in downstream}, {"m2_5", "m3_0", "m3_1", "m3_4", "gd_0"})
        whole = pathlib.Path(self.tmp.name) / "whole"
        whole.mkdir(exist_ok=True)
        shutil.copy(self.db, whole / "design.tndb")
        before = {n["id"]: n["health"] for n in health.health(whole, "ais_3u")["nodes"]}
        for n in ("m2_5", "m3_0", "m3_1", "m3_4", "gd_0"):
            self.assertNotEqual(before[n], "blocked", f"{n} computed before m2_4 broke")


if __name__ == "__main__":
    unittest.main()
