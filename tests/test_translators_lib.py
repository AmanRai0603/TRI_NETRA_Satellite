"""The translators in the library (docs/PLAN_2_0.md S5): trinetra-pcode's Rust ports of the pseudocode
translators (engine/crates/trinetra-pcode/src/gen/, driven by `tndb translate`) write, for every package
the tools translate and for each of Rust, C and MATLAB, byte for byte the files the JavaScript writes
(design/js/pcode_gen.js, pcode_c.js, driven by design/js/pcode_cli.mjs), with the options the tools use.

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import json
import os
import pathlib
import shutil
import subprocess
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
NODE = shutil.which("node")
CLI = ROOT / "design" / "js" / "pcode_cli.mjs"
TNDB = ROOT / "engine" / "target" / "release" / "tndb"
PHYSICS_TITLE = ("TRI-NETRA physics: the relations of spec/plan/physics.toml, translated from the pseudocode "
                 "(spec/physics/*.pc) by tools/pcode.py")

# each package as the tools translate it: its files, in the order they are given, and each language's options
PACKAGES = {
    "physics": (sorted((ROOT / "archive" / "design-1.0" / "spec" / "physics").glob("*.pc")),
                {"rust": ["--title", PHYSICS_TITLE], "c": ["--lib", "pcode", "--title", "x"], "matlab": ["--pkg", "asils.physics"]}),
    "selftest": (sorted((ROOT / "design" / "pcode_selftest").glob("*.pc")),
                 {"rust": ["--title", "translator check"], "c": ["--lib", "pcode", "--title", "x"], "matlab": ["--pkg", "tc"]}),
    "fsw": (sorted((ROOT / "fsw" / "pseudocode").glob("*.pc")),
            {"rust": ["--title", "translator check"], "c": ["--lib", "pcode", "--title", "x"], "matlab": ["--pkg", "tc"]}),
    "groups": (sorted((ROOT / "design" / "groups").glob("*.pc")),
               {"rust": ["--title", "translator check"], "c": ["--lib", "pcode", "--title", "x"], "matlab": ["--pkg", "tc"]}),
}


def tndb_translates():
    """Is the library's command built, and does it translate?"""
    if not (TNDB.is_file() and os.access(TNDB, os.X_OK)):
        return False
    r = subprocess.run([str(TNDB), "translate", "matlab-rt"], capture_output=True, timeout=120)
    return r.returncode == 0


def first_difference(js, rs):
    """Where two outputs first differ: a file one lacks, or a file's first differing line."""
    a, b = json.loads(js), json.loads(rs)
    if list(a) != list(b):
        return f"the files differ: JavaScript {list(a)}, Rust {list(b)}"
    for path in a:
        if a[path] != b[path]:
            la, lb = a[path].split("\n"), b[path].split("\n")
            n = next((i for i, (x, y) in enumerate(zip(la, lb)) if x != y), min(len(la), len(lb)))
            return (f"{path}, line {n + 1}:\n  JavaScript: {la[n] if n < len(la) else '<end>'!r}\n"
                    f"  Rust:       {lb[n] if n < len(lb) else '<end>'!r}")
    return "the same files and text, but not the same JSON"


@unittest.skipUnless(NODE, "Node.js is needed (the JavaScript translators are the reference)")
@unittest.skipUnless(tndb_translates(), "the library's command is not built with `translate` "
                     "(cd engine && cargo build --release -p trinetra-design)")
class TranslatorsInTheLibrary(unittest.TestCase):
    def same(self, args, what):
        js = subprocess.run(["node", str(CLI), *args], capture_output=True, timeout=600)
        rs = subprocess.run([str(TNDB), "translate", *args], capture_output=True, timeout=600)
        self.assertEqual(js.returncode, 0, f"{what}: the JavaScript fails: {js.stderr.decode()[-2000:]}")
        self.assertEqual(rs.returncode, 0, f"{what}: the Rust fails: {rs.stderr.decode()[-2000:]}")
        if js.stdout != rs.stdout:
            self.fail(f"{what}: {first_difference(js.stdout, rs.stdout)}")

    def test_every_package_in_every_language(self):
        for name, (files, opts) in PACKAGES.items():
            self.assertTrue(files, f"{name}: no .pc files")
            for lang in ("rust", "c", "matlab"):
                with self.subTest(package=name, lang=lang):
                    self.same([lang, *map(str, files), *opts[lang]], f"{name} in {lang}")

    def test_the_flight_build_options(self):
        # the flight build embeds the Rust as a module of the no_std flight crate and leaves out the dispatchers
        files = list(map(str, sorted((ROOT / "fsw" / "pseudocode").glob("*.pc"))))
        for args in (["rust", *files, "--root", "crate::alg", "--math", "crate::m", "--no-dispatch", "--title", "t"],
                     ["c", *files, "--lib", "adcs_alg", "--no-dispatch", "--title", "t"]):
            with self.subTest(lang=args[0]):
                self.same(args, f"the flight build's {args[0]}")

    def test_the_matlab_runtime(self):
        self.same(["matlab-rt"], "the MATLAB runtime")


if __name__ == "__main__":
    unittest.main()
