"""A stale engine binary is refused by name, with the rebuild command, and the tools that call the binary ask first.

Copyright (c) 2026 Agastya. All rights reserved."""
import os
import tempfile
import unittest
import unittest.mock
from pathlib import Path

import _path  # puts tools/ on the import path
import design_call
import engine_fresh as F

_ = _path


def put(p, text="x\n", mtime=None):
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(text)
    if mtime is not None:
        os.utime(p, (mtime, mtime))
    return p


class Toy(unittest.TestCase):
    """engine/crates/cli depends on dep (a path dependency) and includes a file outside its crate; other is not a dependency."""

    def setUp(self):
        self.t = tempfile.TemporaryDirectory()
        self.root = Path(self.t.name).resolve()
        c = self.root / "engine" / "crates"
        put(c / "cli" / "Cargo.toml", '[dependencies]\ndep = { path = "../dep" }\n', 100)
        put(c / "cli" / "src" / "main.rs", 'const D: &str = include_str!("../../../../design/schema.toml");\n', 100)
        put(c / "dep" / "Cargo.toml", "[package]\n", 100)
        put(c / "dep" / "src" / "gen.rs", "// generated\n", 100)
        put(c / "other" / "src" / "lib.rs", "// not a dependency\n", 100)
        put(self.root / "design" / "schema.toml", "s\n", 100)
        put(self.root / "engine" / "Cargo.lock", "l\n", 100)
        self.bin = put(self.root / "engine" / "target" / "release" / "adcs", "b", 200)
        self.start = c / "cli"

    def tearDown(self):
        self.t.cleanup()

    def why(self):
        return F.stale(self.bin, self.start, self.root)

    def test_a_binary_newer_than_its_sources_is_current(self):
        self.assertEqual(self.why(), (None, []))
        F.require_fresh(self.bin, self.start, self.root)

    def test_a_dependency_s_source_newer_than_the_binary_is_refused_by_name_with_the_rebuild_command(self):
        gen = self.root / "engine" / "crates" / "dep" / "src" / "gen.rs"
        os.utime(gen, (300, 300))
        with self.assertRaises(F.EngineStale) as e:
            F.require_fresh(self.bin, self.start, self.root)
        self.assertIn("engine/crates/dep/src/gen.rs", str(e.exception))
        self.assertIn("cargo build --release -p adcs-cli", str(e.exception))
        self.assertIn(F.REBUILD, str(e.exception))

    def test_an_included_file_and_the_lock_count_and_a_crate_it_does_not_use_does_not(self):
        os.utime(self.root / "engine" / "crates" / "other" / "src" / "lib.rs", (300, 300))
        self.assertEqual(self.why(), (None, []), "a crate adcs-cli does not depend on: a rebuild would not touch the binary")
        os.utime(self.root / "design" / "schema.toml", (300, 300))
        self.assertEqual([p.name for p in self.why()[1]], ["schema.toml"])
        os.utime(self.root / "engine" / "Cargo.lock", (400, 400))
        self.assertEqual([p.name for p in self.why()[1]], ["Cargo.lock", "schema.toml"], "newest first")

    def test_a_missing_binary_is_refused_with_the_build_command(self):
        self.bin.unlink()
        with self.assertRaises(F.EngineStale) as e:
            F.require_fresh(self.bin, self.start, self.root)
        self.assertIn("not built", str(e.exception))
        self.assertIn(F.REBUILD, str(e.exception))


class Wired(unittest.TestCase):
    def test_a_tool_that_calls_the_binary_stops_with_the_message_when_it_is_stale(self):
        F.check_once.cache_clear()
        stale = F.EngineStale("the engine is older than its sources. Rebuild it: " + F.REBUILD)
        with unittest.mock.patch.object(F, "require_fresh", side_effect=stale), \
                unittest.mock.patch.object(design_call.subprocess, "run") as run:
            with self.assertRaises(SystemExit) as e:
                design_call.call("looprules::loop_part", 1)
        self.assertIn("cargo build --release", str(e.exception))
        run.assert_not_called()
        F.check_once.cache_clear()

    def test_the_real_binary_s_sources_are_the_crates_it_is_built_from(self):
        names = {str(p.relative_to(F.ROOT)) for p in F.sources()}
        self.assertIn("engine/crates/adcs-design/src/gen/looprules.rs", names)
        self.assertIn("fsw-rs/src/lib.rs" if (F.ROOT / "fsw-rs/src/lib.rs").is_file() else "engine/crates/adcs-sim/src/lib.rs", names)
        self.assertFalse(any(n.startswith("engine/crates/trinetra-app/") for n in names))

    def test_the_repository_s_binary_is_current_after_a_build(self):
        if not F.BIN.is_file():
            self.skipTest("the engine is not built")
        F.require_fresh()


if __name__ == "__main__":
    unittest.main()
