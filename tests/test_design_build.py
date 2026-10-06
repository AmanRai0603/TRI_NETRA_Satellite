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


if __name__ == "__main__":
    unittest.main()
