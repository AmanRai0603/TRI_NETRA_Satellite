"""Safe ground (docs/PLAN_2_0.md, S0): a design names the toolbox it was built for and the oldest
application that can run it, and the engine, the app and the Python package refuse one they cannot
run, by name; every run names the design it flew, and is stale once another design is in use; and
the four faults found in reading a design (`adcs size` under a database, DE440 from an empty working
folder, `evaluate` on the pack's root, a folder or a damaged file named as the design) each have a
test that failed before the fix.

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
import unittest

import _path  # puts tools/ on the import path
import design_inputs
import evaluate
import seed_design

_ = _path  # imported for its effect: tools/ on sys.path
ROOT = pathlib.Path(__file__).resolve().parents[1]
BIN = ROOT / "engine" / "target" / "release" / "adcs"
HAVE_BIN = BIN.is_file() and os.access(BIN, os.X_OK)


def set_meta(db, **kv):
    c = sqlite3.connect(db)
    c.executemany('INSERT OR REPLACE INTO meta VALUES (?, ?)', list(kv.items()))
    c.commit()
    c.close()


class SafeGround(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.TemporaryDirectory()
        cls.d = pathlib.Path(cls.tmp.name)
        seed_design.seed(cls.d / "pack" / "Design", sync=False)   # laid out as the Drive pack is
        cls.db = cls.d / "pack" / "Design" / "design.tndb"
        cls.bare = cls.d / "bare"                                 # a data folder with nothing in it
        cls.bare.mkdir()
        cls.empty = cls.d / "empty"                               # a working folder with nothing in it
        cls.empty.mkdir()

    @classmethod
    def tearDownClass(cls):
        cls.tmp.cleanup()

    def copy(self, name, **meta):
        f = self.d / name
        shutil.copy(self.db, f)
        set_meta(f, **meta)
        return f

    def adcs(self, *args, design=None, **env):
        e = {**os.environ, "ADCS_ROOT": str(self.bare), "TRINETRA_STORE": str(self.d / "store"),
             "TRINETRA_DESIGN": str(design if design is not None else self.db), "ADCS_DE440": "", **env}
        return subprocess.run([str(BIN), *args], capture_output=True, text=True, timeout=600, cwd=self.empty, env=e)

    # -- what a design needs ------------------------------------------------------------------------

    def test_a_design_names_its_toolbox_and_the_application_it_needs(self):
        c = sqlite3.connect(self.db)
        meta = dict(c.execute('SELECT "key", "value" FROM meta'))
        c.close()
        self.assertEqual(meta["toolbox"], design_inputs.TOOLBOX)
        self.assertEqual(meta["needs_application"], (ROOT / "VERSION").read_text().strip())

    def test_the_python_package_refuses_a_design_it_cannot_run(self):
        sys.path.insert(0, str(ROOT / "python"))
        try:
            import trinetra_adcs.design as d
        finally:
            sys.path.pop(0)
        with d.open(self.db) as db:
            self.assertTrue(db.groups())
        for name, meta, want in [("other.tndb", {"toolbox": "trinetra-toolbox/99"}, "trinetra-toolbox/99"),
                                 ("newer.tndb", {"needs_application": "99.0.0"}, "needs TRI-NETRA 99.0.0"),
                                 ("unnamed.tndb", {"toolbox": ""}, "names no toolbox")]:
            with self.assertRaises(d.DesignError) as e:
                d.open(self.copy(name, **meta))
            self.assertIn(want, str(e.exception))
        with self.assertRaises(d.DesignError) as e:
            d.open(self.d / "pack")
        self.assertIn("the design database in it is", str(e.exception))
        self.assertEqual(d.cannot_run({"toolbox": d.TOOLBOX, "needs_application": "1.0.0"}, version="1.0.0"), None)
        self.assertIn("needs TRI-NETRA 1.1.0", d.cannot_run({"toolbox": d.TOOLBOX, "needs_application": "1.1.0"}, version="1.0.0"))

    @unittest.skipUnless(HAVE_BIN, "the engine is not built (engine/target/release/adcs)")
    def test_the_engine_refuses_a_design_built_for_another_engine(self):
        for name, meta, want in [("e_other.tndb", {"toolbox": "trinetra-toolbox/99"}, "built for the toolbox trinetra-toolbox/99"),
                                 ("e_newer.tndb", {"needs_application": "99.0.0"}, "needs TRI-NETRA 99.0.0")]:
            r = self.adcs("run", "detumble_ais", "--set", "engine.duration_s=10", "--quiet", design=self.copy(name, **meta))
            self.assertEqual(r.returncode, 2, r.stderr)
            self.assertIn(want, r.stderr)

    # -- the four faults ----------------------------------------------------------------------------

    @unittest.skipUnless(HAVE_BIN, "the engine is not built (engine/target/release/adcs)")
    def test_a_folder_or_a_damaged_file_named_as_the_design_is_refused_by_name(self):
        r = self.adcs("run", "detumble_ais", "--quiet", design=self.d / "pack")
        self.assertEqual(r.returncode, 2, r.stderr)
        self.assertIn("a folder, not a design database; the design database in it is", r.stderr)
        junk = self.d / "junk.tndb"
        junk.write_text("not a database")
        r = self.adcs("run", "detumble_ais", "--quiet", design=junk)
        self.assertEqual(r.returncode, 2, r.stderr)
        self.assertIn("not a database file", r.stderr)
        self.assertNotIn("no scenario", r.stderr, "the design's fault, not a missing scenario")

    @unittest.skipUnless(HAVE_BIN, "the engine is not built (engine/target/release/adcs)")
    def test_size_reads_its_case_from_the_design(self):
        r = self.adcs("size", "ais_3u", "--out", str(self.d / "sized"))
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("[size] ais_3u", r.stdout)

    @unittest.skipUnless(HAVE_BIN, "the engine is not built (engine/target/release/adcs)")
    def test_de440_is_found_from_any_folder_or_named_where_it_was_looked_for(self):
        r = self.adcs("run", "detumble_ais", "--set", "engine.duration_s=60", "--out", str(self.d / "de440_ok"), "--quiet")
        self.assertEqual(r.returncode, 0, r.stderr)
        missing = self.d / "nowhere" / "de440s.bsp"
        r = self.adcs("run", "detumble_ais", "--set", "engine.duration_s=60", "--out", str(self.d / "de440_no"), "--quiet",
                      ADCS_DE440=str(missing))
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("DE440 kernel (de440s.bsp) is not found; looked in", r.stderr)
        self.assertIn(str(missing), r.stderr)

    def test_evaluate_takes_the_pack_root_and_refuses_what_is_not_a_design(self):
        out = self.d / "ev"
        evaluate.main([str(self.d / "pack"), "ais_3u", "--out", str(out)])
        self.assertEqual(json.loads((out / "evaluation.json").read_text())[0]["case"], "ais_3u")
        with self.assertRaises(SystemExit) as e:
            evaluate.main([str(self.empty), "ais_3u", "--out", str(out)])
        self.assertIn("not a design folder", str(e.exception))

    # -- results name their design ------------------------------------------------------------------

    @unittest.skipUnless(HAVE_BIN, "the engine is not built (engine/target/release/adcs)")
    def test_a_run_names_its_design_and_is_stale_once_another_is_in_use(self):
        store = self.d / "stale_store"
        r = self.adcs("run", "detumble_ais", "--set", "engine.duration_s=60", "--quiet", TRINETRA_STORE=str(store))
        self.assertEqual(r.returncode, 0, r.stderr)
        m = json.loads(next(store.rglob("manifest.json")).read_text())
        self.assertEqual(m["inputs"]["design"]["fingerprint"], design_inputs.gather()[2])
        self.assertEqual(m["inputs"]["design"]["toolbox"], design_inputs.TOOLBOX)
        fresh = self.adcs("results", "stale", str(store), TRINETRA_STORE=str(store))
        self.assertEqual(fresh.returncode, 0, fresh.stdout + fresh.stderr)
        other = self.copy("other_design.tndb", inputs_fingerprint="f" * 64, design_version="2026.11.1")
        r = self.adcs("results", "stale", str(store), design=other, TRINETRA_STORE=str(store))
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("the design in use is design 2026.11.1", r.stdout)


if __name__ == "__main__":
    unittest.main()
