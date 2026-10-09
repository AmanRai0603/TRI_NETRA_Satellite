"""The flight build's done-when (docs/PLAN_2_0.md S6): a change to a flight parameter or an algorithm node, made in the
database, reaches the image and the metrics with no step beyond the flight build.

Each test copies the regression design (tests/regression/design.tndb) into a temporary folder, changes it there, and
follows the change: an algorithm node's method through the flight build (tools/flight_build.py gen --design --out) to
the generated sources, the algorithms' identity and the built library's vectors; a flight parameter through the
configuration blob the engine boots the flight software with (what a seal keeps) to a short run's trajectory and
metrics, read from the design by the engine as it is built (no rebuild). The repository's tree is never written.

Where the flight parameters are today: the parameter blob's layout is the design's (the fsw_param_* blocks), and its
values are what the engine derives from the case and the scenario the design holds (engine/crates/adcs-sim/src/
config.rs) until the relations that derive them are methods (S7). So the parameter changed here is the scenario's
flight-software section in the design (detumble_ais: fsw.bdot_gain_scale, the B-dot gain's scale), as merging a
changed release would write it (tools/design_inputs.py set_input).

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import json
import os
import pathlib
import shutil
import sqlite3
import subprocess
import tempfile
import unittest

import _path  # puts tools/ on the import path
import design_inputs
import flight_build
import fswcfg

_ = _path  # imported for its effect: tools/ on sys.path
ROOT = pathlib.Path(__file__).resolve().parents[1]
DESIGN = ROOT / "tests" / "regression" / "design.tndb"
ADCS = ROOT / "engine" / "target" / "release" / "adcs"
# the algorithm change: a constant of the sliding-mode branch of control_law (fsw_control's method, 05_control.pc)
NODE, OLD, NEW = "fsw_control", "let ed = 0.5*(qe[3]*we[i] + c[i])", "let ed = 0.25*(qe[3]*we[i] + c[i])"
# the parameter change: the B-dot gain's scale in the scenario's flight-software section
SCENARIO, INPUT, PARAM, VALUE = "detumble_ais", "data/scenarios/detumble_ais.json", "bdot_gain_scale", 1.5


def translator():
    return flight_build.TNDB.is_file() or shutil.which("node")


def set_method(db, node, old, new):
    """Change a node's method (its pseudocode) in a design database, in place."""
    with sqlite3.connect(db) as c:
        content = json.loads(c.execute('SELECT "content" FROM design_node WHERE "id" = ?', (node,)).fetchone()[0])
        rows = [r for r in content["body"]["content"] if r[0] == "code" and r[1] == "pseudocode"]
        assert len(rows) == 1 and rows[0][2].count(old) == 1, f"{node}: the method does not hold {old!r} once"
        rows[0][2] = rows[0][2].replace(old, new)
        c.execute('UPDATE design_node SET "content" = ? WHERE "id" = ?', (json.dumps(content, sort_keys=True), node))


def set_scenario_param(db, path, key, value):
    with sqlite3.connect(db) as c:
        d = json.loads(c.execute('SELECT "body" FROM engine_input WHERE "path" = ?', (path,)).fetchone()[0])
        assert key in d["fsw"], f"{path} states no fsw.{key}"
        d["fsw"][key] = value
        design_inputs.set_input(c, path, (json.dumps(d, indent=1, sort_keys=True) + "\n").encode())


class FlightBuild(unittest.TestCase):
    def setUp(self):
        self.tmp = pathlib.Path(tempfile.mkdtemp(prefix="test_flight_build_"))

    def tearDown(self):
        shutil.rmtree(self.tmp, ignore_errors=True)

    def copy(self):
        d = self.tmp / "design.tndb"
        shutil.copy2(DESIGN, d)
        return d

    @unittest.skipUnless(translator(), "needs the library's translator (engine/target/release/tndb) or node")
    def test_the_tree_is_what_the_design_gives(self):
        self.assertEqual(flight_build.main(["gen", "--check"]), 0)
        outs, aid = flight_build.outputs()
        self.assertEqual(aid, flight_build.current_id())
        self.assertIn(f'"{aid}"', (ROOT / flight_build.RS_ID).read_text())

    @unittest.skipUnless(translator(), "needs the library's translator (engine/target/release/tndb) or node")
    def test_an_algorithm_change_reaches_the_sources_the_id_and_the_library(self):
        db = self.copy()
        set_method(db, NODE, OLD, NEW)
        out = self.tmp / "out"
        self.assertEqual(flight_build.main(["gen", "--design", str(db), "--out", str(out)]), 0)
        # the sources: the control module changes in C, Rust and the twin's MATLAB, and nothing else but the identity
        changed = sorted(rel for d in flight_build.GEN_DIRS for p in (out / d).rglob("*") if p.is_file()
                         for rel in [p.relative_to(out).as_posix()] if p.read_text() != (ROOT / rel).read_text())
        self.assertEqual(changed, ["fsw-rs/src/alg/alg_id.rs", "fsw-rs/src/alg/control.rs",
                                   "fsw/alg/include/adcs_alg_id.h", "fsw/alg/src/control.c",
                                   "matlab_sils/+asils/+alg/+control/control_law.m", "matlab_sils/+asils/+alg/alg_id.m"])
        self.assertIn("0.25", (out / "fsw/alg/src/control.c").read_text())
        new_id = flight_build.current_id(out)
        self.assertNotEqual(new_id, flight_build.current_id())
        # the same design again gives the same identity (deterministic: gen --check holds on it)
        self.assertEqual(flight_build.main(["gen", "--check", "--design", str(db), "--out", str(out)]), 0)
        if not (shutil.which("make") and shutil.which("gcc")):
            self.skipTest("needs make and gcc to build the library")
        # the library: the runtime with the algorithms written from the changed design, on the pseudocode's vectors
        fsw = self.tmp / "fsw"
        shutil.copytree(ROOT / "fsw", fsw, ignore=shutil.ignore_patterns("build", "alg"))
        shutil.copytree(out / "fsw" / "alg", fsw / "alg")
        shutil.copy2(out / flight_build.C_DISPATCH, fsw / "tests" / "alg_dispatch.c")
        r = subprocess.run(["make", "-s", "-j4", "build/libadcs_fsw.a", "build/test_pcode"], cwd=fsw, capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertEqual(flight_build.runtime_version((fsw / "build" / "libadcs_fsw.a").read_bytes()),
                         flight_build.expected_id("c", new_id), "the library names the changed algorithms")
        r = subprocess.run(["./build/test_pcode"], cwd=fsw, capture_output=True, text=True)
        fails = [ln for ln in r.stdout.splitlines() if ln.startswith("FAIL")]
        self.assertNotEqual(r.returncode, 0, "the changed algorithm no longer reproduces the vectors drawn before the change")
        # control_law, and the magnetic step law, which flies it as its fallback law on the coil period
        self.assertEqual(sorted({ln.split()[1] for ln in fails}), ["control::control_law", "steplaws::ctl_mtq"], fails[:5])

    @unittest.skipUnless(ADCS.is_file(), "needs the engine (python3 tools/engine.py build)")
    def test_a_parameter_change_reaches_the_blob_and_the_metrics(self):
        db = self.copy()
        set_scenario_param(db, INPUT, PARAM, VALUE)
        before, after = flight_build.blobs(DESIGN), flight_build.blobs(db)
        # the image's blobs (what a seal keeps): this scenario's alone changes, and in it the B-dot gain alone
        self.assertEqual(sorted(s for s in before if before[s] != after[s]), [SCENARIO])
        a, b = fswcfg.decode(before[SCENARIO]), fswcfg.decode(after[SCENARIO])
        self.assertEqual(sorted(k for k in a if a[k] != b[k]), ["bdot_k"])
        self.assertAlmostEqual(b["bdot_k"] / a["bdot_k"], VALUE / 3.0, places=12)
        # a short run, the engine reading each design as it is (not rebuilt): another trajectory, other metrics
        runs = {}
        for name, d in (("before", DESIGN), ("after", db)):
            out = self.tmp / name
            r = subprocess.run([str(ADCS), "run", SCENARIO, "--fsw", "c", "--set", "engine.duration_s=600", "--out", str(out), "--quiet"],
                               cwd=ROOT, capture_output=True, text=True, env={**os.environ, "TRINETRA_DESIGN": str(d)})
            self.assertEqual(r.returncode, 0, r.stderr)
            runs[name] = (json.loads((out / "manifest.json").read_text()), (out / "channels.csv").read_bytes())
        (m0, c0), (m1, c1) = runs["before"], runs["after"]
        self.assertNotEqual(c0, c1, "the trajectory moves with the parameter")
        v0 = {m["id"]: m["value"] for m in m0["metrics"]}
        v1 = {m["id"]: m["value"] for m in m1["metrics"]}
        self.assertTrue(any(v0[k] != v1[k] for k in v0), (v0, v1))
        self.assertEqual(m0["fsw"]["build_id"], m1["fsw"]["build_id"], "the same flight software, booted with another blob")


if __name__ == "__main__":
    unittest.main()
