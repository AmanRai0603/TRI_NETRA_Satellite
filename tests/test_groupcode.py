"""Each group's code, wired from its nodes (docs/RELEASE_PLAN.md P10, tools/groupcode.py) and generated with the design's
relations into one crate (tools/engine_build.py, S7.16): the wiring in design/groups/ is the design's, the generated Rust,
WebAssembly face and MATLAB are current and the crate a workspace member, every computing row with pseudocode is in the
generated code (no physics written by hand), each relation once, the Rust reproduces the interpreter on every drawn vector
and every node's own test vectors, and every group's test app passes in the browser, in the interpreter and in WebAssembly
alike.

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import json
import os
import pathlib
import shutil
import subprocess
import sys
import tempfile
import unittest

import _path  # puts tools/ on the import path
import engine_build
import groupcode

_ = _path  # imported for its effect: tools/ on sys.path
ROOT = pathlib.Path(__file__).resolve().parents[1]
NODE, CARGO = shutil.which("node"), shutil.which("cargo")
PLAYWRIGHT = next((pathlib.Path(p) for p in (os.environ.get("PLAYWRIGHT_MODULE"), "/opt/node22/lib/node_modules/playwright/index.mjs")
                   if p and pathlib.Path(p).is_file()), None)


def wasm_target():
    r = subprocess.run(["rustup", "target", "list", "--installed"], capture_output=True, text=True) if shutil.which("rustup") else None
    return bool(r and "wasm32-unknown-unknown" in r.stdout)


@unittest.skipUnless(NODE, "Node.js is needed (the pseudocode runs as JavaScript)")
class Generated(unittest.TestCase):
    def test_the_wiring_is_the_designs(self):
        files = groupcode.wire_files()
        for path, text in files.items():
            self.assertEqual(path.read_text(encoding="utf-8") if path.exists() else None, text, f"{path.name} is stale: python3 tools/groupcode.py wire")
        self.assertEqual(sorted(p.name for p in groupcode.SRC.glob("*")), sorted(p.name for p in files))

    def test_the_generated_code_is_current(self):
        stale = [p for p, t in engine_build.relations().items() if not (ROOT / p).is_file() or (ROOT / p).read_text(encoding="utf-8") != t]
        self.assertEqual(stale, [], "python3 tools/engine_build.py gen")
        self.assertIn('"crates/adcs-relations"', (ROOT / "engine" / "Cargo.toml").read_text())
        for gone in ("adcs-physics", "adcs-groups", "adcs-groups-wasm"):        # folded into adcs-relations (S7.16)
            self.assertFalse((ROOT / "engine" / "crates" / gone).exists(), gone)

    def test_every_computing_row_with_pseudocode_is_in_the_generated_code(self):
        wires = [json.loads(p.read_text(encoding="utf-8")) for p in sorted(groupcode.SRC.glob("*.wire.json"))]
        self.assertEqual(len(wires), 20)
        rows = [r for w in wires for r in w["rows"]]
        self.assertGreaterEqual(len(rows), 48)
        rust = "".join(p.read_text(encoding="utf-8") for p in (groupcode.CRATE / "src" / "gen").glob("*.rs"))
        for r in rows:
            self.assertIsNotNone(r["fn"], f"{r['id']}: no function gives its answer")
            self.assertIn(f"pub fn {r['fn']}(", rust, f"{r['id']}: {r['fn']} is not in the generated Rust")

    def test_each_relation_once(self):
        # a group's copy of a library relation is the library's: one definition of each name in the crate
        rust = [ln for p in (groupcode.CRATE / "src" / "gen").glob("*.rs") for ln in p.read_text(encoding="utf-8").split("\n")]
        defs = [ln.split("(")[0].split("<")[0] for ln in rust if ln.startswith("pub fn ")]
        self.assertEqual(sorted(x for x in set(defs) if defs.count(x) > 1), [])
        srcs, moved = engine_build.relations_sources()
        self.assertGreater(len(moved), 0)
        self.assertEqual(len(srcs), len(list(groupcode.SRC.glob("*.pc"))) + 11)

    @unittest.skipUnless(CARGO, "cargo is needed")
    def test_the_rust_reproduces_the_interpreter_and_the_nodes_test_vectors(self):
        r = subprocess.run(["cargo", "test", "--locked", "--release", "-q", "-p", "adcs-relations"], cwd=ROOT / "engine", capture_output=True, text=True, timeout=1800)
        self.assertEqual(r.returncode, 0, r.stdout[-3000:] + r.stderr[-3000:])
        fx = json.loads((groupcode.CRATE / "tests" / "fixtures.json").read_text())
        self.assertGreater(len(fx), 0)


@unittest.skipUnless(NODE and PLAYWRIGHT and CARGO and wasm_target(), "Node.js, Playwright, cargo and the wasm32-unknown-unknown target are needed for the test apps")
class TestApps(unittest.TestCase):
    def test_every_groups_test_app_passes_in_the_browser(self):
        with tempfile.TemporaryDirectory() as d:
            made = groupcode.deliver(d)
            self.assertEqual(len(made), 20)
            r = subprocess.run([NODE, str(ROOT / "tests" / "browser" / "testapp.test.mjs"), d], capture_output=True, text=True, timeout=900,
                               env={**os.environ, "PLAYWRIGHT_MODULE": str(PLAYWRIGHT)})
            sys.stdout.write(r.stdout[-2000:])
            self.assertEqual(r.returncode, 0, r.stdout[-4000:] + r.stderr[-2000:])


if __name__ == "__main__":
    unittest.main()
