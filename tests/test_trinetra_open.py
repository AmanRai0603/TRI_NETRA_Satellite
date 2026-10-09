"""The twin opens the design database itself (docs/PLAN_2_0.md S7; docs/S7_INVENTORY.md S7.18): the library's commands
the MATLAB twin calls over system() give what the tools give, and the twin flies from them as from the files.

  - `tndb read DESIGN` is every input tools/from_design.py exports for the twin (matlab_sils/data, matlab_sils/cases),
    byte for byte, and no other; `--engine-inputs` and `--cases` the two halves;
  - `tndb build-matlab DESIGN OUTDIR --groups design/groups` is every file tools/engine_build.py and tools/flight_build.py
    write for the twin (+asils/+models, +relations, +alg, +pc) byte for byte, and no other, read-only, with an index
    naming each file's node; a folder holding other files is refused;
  - `tndb health DESIGN` counts the nodes as the inventory does, and the built-in count is tools/health.py's;
  - the Octave test (matlab_sils/tests/test_trinetra_open.m, also in the twin's suite) opens, builds and flies the
    regression copy in a temporary workspace with +trinetra and holds the runs to the committed tree's, bit for bit.

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import json
import os
import pathlib
import shutil
import subprocess
import tempfile
import unittest

import _path  # puts tools/ on the import path
import from_design
import health

_ = _path  # imported for its effect: tools/ on sys.path
ROOT = pathlib.Path(__file__).resolve().parents[1]
REG = ROOT / "tests" / "regression" / "design.tndb"
MS = ROOT / "matlab_sils"
TNDB = pathlib.Path(os.environ.get("TNDB_BIN") or ROOT / "engine" / "target" / "release" / "tndb")
ADCS = pathlib.Path(os.environ.get("ADCS_BIN") or ROOT / "engine" / "target" / "release" / "adcs")
PACKAGES = ("+asils/+models", "+asils/+relations", "+asils/+alg", "+asils/+pc")


def tndb_reads():
    """Is the library's command built with `read` (an older build has no such command)?"""
    if not (TNDB.is_file() and os.access(TNDB, os.X_OK)):
        return False
    r = subprocess.run([str(TNDB), "read", str(REG), "--cases"], capture_output=True, timeout=120)
    return r.returncode == 0


def tndb(*args, check=True):
    r = subprocess.run([str(TNDB), *map(str, args)], capture_output=True, text=True, timeout=600)
    if check and r.returncode:
        raise AssertionError(f"tndb {args[0]}: {r.stderr.strip()[-2000:]}")
    return r


def tree(top):
    return {p.relative_to(top).as_posix() for p in top.rglob("*") if p.is_file()}


@unittest.skipUnless(tndb_reads(), "the library's command is not built with `read` (cargo build --release -p trinetra-design)")
class LibraryCommands(unittest.TestCase):
    def test_read_is_what_from_design_exports_for_the_twin(self):
        got = {x["path"]: x["body"].encode() for x in json.loads(tndb("read", REG).stdout)}
        want = {p[len("matlab_sils/"):]: b for p, (b, _how) in from_design.outputs().items() if p.startswith("matlab_sils/")}
        self.assertEqual(sorted(got), sorted(want))
        for k in want:
            self.assertEqual(got[k], want[k], k)
        eng = {x["path"] for x in json.loads(tndb("read", REG, "--engine-inputs").stdout)}
        cases = {x["path"] for x in json.loads(tndb("read", REG, "--cases").stdout)}
        self.assertEqual(eng | cases, set(got))
        self.assertTrue(all(p.startswith("data/") for p in eng) and all(p.startswith("cases/") for p in cases))
        self.assertIn("data/stated.json", eng)

    def test_read_refuses_what_is_not_a_design(self):
        with tempfile.TemporaryDirectory() as d:
            f = pathlib.Path(d) / "x.tndb"
            f.write_bytes(b"not a database")
            r = tndb("read", f, check=False)
            self.assertEqual(r.returncode, 2, r.stderr)
            self.assertIn("x.tndb", r.stderr)

    def test_build_matlab_is_what_the_engine_and_flight_builds_write_for_the_twin(self):
        with tempfile.TemporaryDirectory() as d:
            out = pathlib.Path(d) / "generated"
            s = json.loads(tndb("build-matlab", REG, out, "--groups", ROOT / "design" / "groups").stdout)
            n = 0
            for pk in PACKAGES:
                built, committed = tree(out / pk), tree(MS / pk)
                self.assertEqual(built, committed, pk)
                for f in built:
                    self.assertEqual((out / pk / f).read_bytes(), (MS / pk / f).read_bytes(), f"{pk}/{f}")
                    n += 1
            self.assertEqual(n, s["files"])
            self.assertEqual((out / "+asils" / "+pc" / "clamp.m").stat().st_mode & 0o222, 0, "the generated files are read-only")
            idx = json.loads((out / "index.json").read_text())
            self.assertEqual(idx["fingerprint"], from_design.fingerprint())
            aero = [e for e in idx["files"] if e["file"] == "+asils/+models/+facets/aero_torque.m"]
            self.assertEqual(aero[0]["node"], "l3_dist_row_02")
            self.assertEqual(aero[0]["revision"], "S7.4")
            # built again into its own folder: replaced; a folder with other files: refused, and nothing deleted
            tndb("build-matlab", REG, out)
            self.assertFalse((out / "+asils" / "+relations").exists(), "without the groups' wiring the relations are not built")
            mine = pathlib.Path(d) / "mine"
            mine.mkdir()
            (mine / "keep.m").write_text("x")
            r = tndb("build-matlab", REG, mine, check=False)
            self.assertEqual(r.returncode, 1)
            self.assertEqual((mine / "keep.m").read_text(), "x")
            for p in out.rglob("*"):
                p.chmod(0o755 if p.is_dir() else 0o644)

    def test_health_counts_as_the_inventory_and_health_py(self):
        h = json.loads(tndb("health", REG).stdout)
        self.assertEqual(h["nodes"], sum(h["behaviours"].values()))
        self.assertEqual(h["built_in"]["count"], health.built_in(REG.parent)["count"])
        self.assertEqual(h["fingerprint"], from_design.fingerprint())
        self.assertEqual(h["inputs"]["not_their_fingerprint"], [])
        self.assertEqual(sorted(h["cases"]), sorted(p.stem for p in (MS / "cases").glob("*.csv")))


@unittest.skipUnless(shutil.which("octave-cli") and tndb_reads() and ADCS.is_file(),
                     "GNU Octave, the library's command (tndb) and the engine (adcs) are needed to fly the twin")
class TwinOpensTheDesign(unittest.TestCase):
    def test_open_build_and_run_in_octave(self):
        env = dict(os.environ, TNDB_BIN=str(TNDB), ADCS_BIN=str(ADCS))
        r = subprocess.run(["octave-cli", "--no-gui", "-q", "--eval",
                            "startup_asils; addpath tests; try, disp(['RESULT ' test_trinetra_open()]); "
                            "catch e, disp(['FAILED ' e.message]); exit(1); end"],
                           cwd=MS, env=env, capture_output=True, text=True, timeout=1800)
        self.assertEqual(r.returncode, 0, (r.stdout + r.stderr)[-3000:])
        self.assertIn("RESULT ", r.stdout)
        self.assertIn("bit for bit", r.stdout)


if __name__ == "__main__":
    unittest.main()
