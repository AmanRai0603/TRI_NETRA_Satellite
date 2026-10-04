"""The engine flies from the design database (docs/RELEASE_PLAN.md P11; tools/design_inputs.py,
engine/crates/adcs-sim/src/source.rs): the database holds every case line by line and every input
file under data/, they give back the files' own bytes, and a flight from the database alone, in a
data folder holding no case and no input, has the same result id, metrics and channels as the
flight from the files. A path the database does not hold is refused, never read from the folder.

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import json
import os
import pathlib
import shutil
import sqlite3
import subprocess
import sys
import tempfile
import time
import unittest
import urllib.request

import _path  # puts tools/ on the import path
import design_inputs
import seed_design
import tndb

_ = _path  # imported for its effect: tools/ on sys.path
ROOT = pathlib.Path(__file__).resolve().parents[1]
BIN = ROOT / "engine" / "target" / "release" / "adcs"
APP = ROOT / "engine" / "target" / "release" / "trinetra-app"
NODE = shutil.which("node")
PLAYWRIGHT = next((pathlib.Path(p) for p in (os.environ.get("PLAYWRIGHT_MODULE"), "/opt/node22/lib/node_modules/playwright/index.mjs")
                   if p and pathlib.Path(p).is_file()), None)


class Inputs(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.TemporaryDirectory()
        cls.d = pathlib.Path(cls.tmp.name)
        seed_design.seed(cls.d / "seed", sync=False)
        cls.db = cls.d / "seed" / "design.tndb"

    @classmethod
    def tearDownClass(cls):
        cls.tmp.cleanup()

    def test_every_case_is_held_line_by_line_and_gives_its_bytes_back(self):
        c = sqlite3.connect(self.db)
        for f in sorted((design_inputs.DATA / "cases").glob("*.csv")):
            lines = [r[0] for r in c.execute('SELECT "line" FROM design_case WHERE case_id = ? ORDER BY "ord"', (f.stem,))]
            self.assertEqual(design_inputs.HEADER + "\n" + "".join(x + "\n" for x in lines), f.read_text(encoding="utf-8"), f.name)
        # each declared value names the node that declares it
        node = c.execute('SELECT "node" FROM design_case WHERE case_id = ? AND "key" = ?', ("ais_3u", "req.ape")).fetchone()[0]
        self.assertEqual(node, "p1k_0")
        c.close()

    def test_every_input_file_is_held_with_its_fingerprint(self):
        c = sqlite3.connect(self.db)
        held = {p: (fp, b) for p, fp, b in c.execute('SELECT "path", "fingerprint", "body" FROM engine_input')}
        meta = dict(c.execute('SELECT "key", "value" FROM meta'))
        c.close()
        files = [f for f in (design_inputs.DATA / "data").rglob("*") if f.is_file() and not f.name.endswith(design_inputs.NOT_INPUTS)]
        self.assertEqual(sorted(held), sorted(f.relative_to(design_inputs.DATA).as_posix() for f in files))
        for p, (fp, b) in held.items():
            self.assertEqual(fp, design_inputs.fnv_hex(b), p)
            self.assertEqual(b, (design_inputs.DATA / p).read_bytes(), p)
        self.assertIn("data/scenarios/nadir_hold_ais.json", held)
        self.assertEqual(meta["format_version"], "2")
        self.assertEqual(meta["inputs_fingerprint"], design_inputs.gather()[2])
        self.assertEqual(tndb.check(self.db), [])

    def test_a_version_1_design_upgrades_to_hold_the_inputs(self):
        f = self.d / "old.tndb"
        c = sqlite3.connect(f)
        for st in tndb.ddl("design"):
            if '"design_case"' not in st and '"engine_input"' not in st:
                c.execute(st)
        c.executemany("INSERT INTO meta VALUES (?, ?)", [("format", "design"), ("format_version", "1"), ("id", "design"), ("written_by", "t")])
        c.commit()
        c.close()
        tndb.open_file(f).close()
        self.assertEqual(tndb.check(f), [])
        self.assertTrue((self.d / "old.tndb.v1.bak").is_file())

    def test_the_python_package_reads_the_design_and_the_inputs(self):
        sys.path.insert(0, str(ROOT / "python"))
        try:
            import trinetra_adcs.design as d
        finally:
            sys.path.pop(0)
        with d.open(self.db) as db:
            self.assertEqual(len(db.groups()), 20)
            self.assertEqual(db.node("gd_0")["group_id"], "env")
            self.assertEqual(db.case("ais_3u")["req.ape"], 10.0)
            self.assertIsNone(db.case("ais_3u")["req.hsat"], "a blank value is None, never 0")
            self.assertEqual(db.cases(), ["ais_3u", "ais_img_3u", "case_template"])
            self.assertEqual(db.scenario("nadir_hold_ais"), json.loads((design_inputs.DATA / "data" / "scenarios" / "nadir_hold_ais.json").read_text()))
            self.assertEqual(db.fingerprint, design_inputs.gather()[2])
            self.assertTrue(set(db.readers("gd_0")) <= {n["id"] for n in db.nodes()})
        with self.assertRaises(d.DesignError):
            d.open(self.d / "seed" / "structure" / "env.group.tndb")

    @unittest.skipUnless(BIN.is_file() and os.access(BIN, os.X_OK), "the engine is not built (engine/target/release/adcs)")
    def test_a_flight_from_the_database_alone_is_the_flight_from_the_files(self):
        bare = self.d / "bare"           # a data folder with the orbit's data and no case, no input
        bare.mkdir()
        (bare / "pop").symlink_to(ROOT / "matlab_sils" / "pop")

        def fly(out, **env):
            r = subprocess.run([str(BIN), "run", "nadir_hold_ais", "--set", "engine.duration_s=60", "--out", str(out), "--quiet"],
                               capture_output=True, text=True, timeout=300, cwd=self.d,
                               env={**os.environ, "TRINETRA_STORE": str(self.d / "store"), "TRINETRA_DESIGN": "", **env})
            return r

        a = fly(self.d / "files", ADCS_ROOT=str(ROOT / "matlab_sils"))
        b = fly(self.d / "db", ADCS_ROOT=str(bare), TRINETRA_DESIGN=str(self.db))
        self.assertEqual((a.returncode, b.returncode), (0, 0), a.stderr + b.stderr)
        ma, mb = (json.loads((self.d / x / "manifest.json").read_text()) for x in ("files", "db"))
        self.assertEqual(ma["result_id"], mb["result_id"])
        self.assertEqual(ma["metrics"], mb["metrics"])
        self.assertEqual((self.d / "files" / "channels.csv").read_bytes(), (self.d / "db" / "channels.csv").read_bytes())
        # the run says what it flew from, in one hash, and only what differs from the shipped scenario
        self.assertIsNone(ma["inputs"]["design"])
        self.assertEqual(mb["inputs"]["design"]["fingerprint"], design_inputs.gather()[2])
        self.assertEqual(ma["inputs"]["input_hash"], mb["inputs"]["input_hash"])
        self.assertEqual(mb["inputs"]["differs"], ["engine.duration_s=60"])
        # without the database the bare folder has nothing to fly; with it, a scenario it lacks is refused
        self.assertNotEqual(fly(self.d / "x", ADCS_ROOT=str(bare)).returncode, 0)
        r = subprocess.run([str(BIN), "run", "no_such_scenario", "--quiet"], capture_output=True, text=True, timeout=60, cwd=self.d,
                           env={**os.environ, "ADCS_ROOT": str(bare), "TRINETRA_DESIGN": str(self.db), "TRINETRA_STORE": str(self.d / "store")})
        self.assertEqual(r.returncode, 2, r.stderr)
        self.assertIn("no scenario no_such_scenario", r.stderr)

    @unittest.skipUnless(APP.is_file() and os.access(APP, os.X_OK), "the app is not built (engine/target/release/trinetra-app)")
    def test_the_app_lists_its_cases_and_scenarios_from_the_database(self):
        bare = self.d / "bare_app"
        bare.mkdir()
        port = 7911
        p = subprocess.Popen([str(APP)], cwd=self.d, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
                             env={**os.environ, "ADCS_ROOT": str(bare), "TRINETRA_DESIGN": str(self.db), "TRINETRA_PORT": str(port),
                                  "TRINETRA_NO_BROWSER": "1", "TRINETRA_STORE": str(self.d / "store")})
        try:
            get = lambda path: json.loads(urllib.request.urlopen(f"http://127.0.0.1:{port}{path}", timeout=5).read())
            for _ in range(100):
                try:
                    v = get("/v1/version")
                    break
                except OSError:
                    time.sleep(0.1)
            self.assertEqual(v["design"]["fingerprint"], design_inputs.gather()[2])
            cat = get("/v1/catalogue")
            self.assertEqual(sorted(c["id"] for c in cat["cases"]), ["ais_3u", "ais_img_3u"])
            self.assertIn("nadir_hold_ais", [s["id"] for s in cat["scenarios"]])
            self.assertEqual(len(cat["scenarios"]), len(list((design_inputs.DATA / "data" / "scenarios").glob("*.json"))))
        finally:
            p.terminate()
            p.wait(timeout=10)

    @unittest.skipUnless(APP.is_file() and NODE and PLAYWRIGHT, "the app, Node.js and Playwright are needed for the page's browser test")
    def test_the_app_page_flies_and_draws_from_the_database(self):
        bare = self.d / "bare_page"
        bare.mkdir()
        (bare / "pop").symlink_to(ROOT / "matlab_sils" / "pop")
        port = 7913
        p = subprocess.Popen([str(APP)], cwd=self.d, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
                             env={**os.environ, "ADCS_ROOT": str(bare), "TRINETRA_DESIGN": str(self.db), "TRINETRA_PORT": str(port),
                                  "TRINETRA_NO_BROWSER": "1", "TRINETRA_STORE": str(self.d / "store_page")})
        try:
            for _ in range(100):
                try:
                    urllib.request.urlopen(f"http://127.0.0.1:{port}/v1/version", timeout=2)
                    break
                except OSError:
                    time.sleep(0.1)
            r = subprocess.run([NODE, str(ROOT / "tests" / "browser" / "app.test.mjs"), f"http://127.0.0.1:{port}"], capture_output=True, text=True,
                               timeout=600, env={**os.environ, "PLAYWRIGHT_MODULE": str(PLAYWRIGHT)})
            sys.stdout.write(r.stdout[-2000:])
            self.assertEqual(r.returncode, 0, r.stdout[-4000:] + r.stderr[-2000:])
        finally:
            p.terminate()
            p.wait(timeout=10)


if __name__ == "__main__":
    unittest.main()
