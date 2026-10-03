"""Pseudocode v2 (docs/PSEUDOCODE_V2.md): the checker refuses what it should, by name and line; the
interpreter's features (several outputs, settling loops, tables, state between calls) answer as
written; the physics matches its registry and its sourced test vectors. Runs the one
implementation (design/js/pcode.js) under Node; skipped where Node is not installed.
Copyright (c) 2026 Agastya. All rights reserved.
"""
import json
import shutil
import subprocess
import tempfile
import unittest
import pathlib

import _path  # puts tools/ on the import path
import pcode

_ = _path  # imported for its effect: tools/ on sys.path

NODE = shutil.which("node")


def check(text):
    with tempfile.TemporaryDirectory() as d:
        f = pathlib.Path(d) / "t.pc"
        f.write_text(text)
        r = subprocess.run([NODE, str(pcode.CLI), "check", str(f)], capture_output=True, text=True)
        return r.returncode, r.stderr


def run(text, fn, args):
    with tempfile.TemporaryDirectory() as d:
        f = pathlib.Path(d) / "t.pc"
        f.write_text(text)
        return pcode.cli("run", str(f), "--fn", fn, "--args", json.dumps(args))


@unittest.skipUnless(NODE, "Node.js is not installed")
class Checker(unittest.TestCase):
    def refused(self, text, words):
        code, err = check(text)
        self.assertEqual(code, 1, f"accepted: {text}")
        self.assertIn(words, err)
        self.assertRegex(err, r"t\.pc:\d+:\d+: ")

    def test_units_that_differ_are_refused(self):
        self.refused("fn f(a: real[m], b: real[s]) -> c: real[m]\n    c = a + b\nend\n", "the units differ, [m] and [s]")

    def test_an_output_not_set_on_every_path_is_named(self):
        self.refused("fn f(a: real[m]) -> c: real[m]\n    if a > 0 [m]\n        c = a\n    end\nend\n", "output c is not set on every path")

    def test_a_trig_function_of_a_length_is_refused(self):
        self.refused("fn f(a: real[m]) -> c: real[1]\n    c = sin(a)\nend\n", "sin takes a plain number")

    def test_an_input_is_not_changed(self):
        self.refused("fn f(a: real[m]) -> c: real[m]\n    a = 2*a\n    c = a\nend\n", "a is an input")

    def test_recursion_is_refused(self):
        self.refused("fn f(a: real[1]) -> c: real[1]\n    c = g(a)\nend\nfn g(a: real[1]) -> c: real[1]\n    c = f(a)\nend\n", "recursion: f -> g -> f")

    def test_a_power_that_is_not_a_whole_literal_is_refused(self):
        self.refused("fn f(a: real[m], k: int) -> c: real[m]\n    c = a^k\nend\n", "whole-number literal")

    def test_an_unknown_unit_is_named(self):
        self.refused("fn f(a: real[furlong]) -> c: real[1]\n    c = 1\nend\n", "no unit furlong")

    def test_a_vector_times_a_vector_asks_for_dot_or_cross(self):
        self.refused("fn f(a: vec3[m], b: vec3[m]) -> c: real[m^2]\n    c = a*b\nend\n", "write dot(a, b), cross(a, b)")

    def test_int_takes_a_plain_number(self):
        self.refused("fn f(a: real[m]) -> n: int\n    n = int(a)\nend\n", "plain number")

    def test_state_belongs_to_a_proc(self):
        self.refused("fn f(a: real[1]) -> c: real[1]\n    state k: int = 0\n    c = a\nend\n", "state is kept by a proc")


@unittest.skipUnless(NODE, "Node.js is not installed")
class Interpreter(unittest.TestCase):
    def test_literals_are_si(self):
        self.assertAlmostEqual(run("fn f() -> (a: real[rad], b: real[m])\n    a = 90 [deg]\n    b = 1.5 [km]\nend\n", "f", []), [1.5707963267948966, 1500.0])

    def test_a_settling_loop_stops_when_it_settles_and_says_when_it_does_not(self):
        src = (pcode.ROOT / "design" / "pcode_selftest" / "selftest.pc").read_text()
        r, steps, ok = run(src, "root", [2.0, 30])
        self.assertTrue(ok)
        self.assertAlmostEqual(r, 2 ** 0.5, places=12)
        self.assertEqual(run(src, "root", [2.0, 3]), [-1, 3, False])

    def test_a_linear_table_interpolates_and_holds_its_ends(self):
        src = (pcode.ROOT / "design" / "pcode_selftest" / "selftest.pc").read_text()
        self.assertEqual(run(src, "lookup", [2.5]), [17.5, -0.75])
        self.assertEqual(run(src, "lookup", [-1.0]), [0.0, 1.0])
        self.assertEqual(run(src, "lookup", [50.0]), [0.0, 0.0])

    def test_int_drops_the_fraction_as_a_cast_does(self):
        src = "fn f(x: real[1]) -> (n: int, w: int)\n    n = int(x)\n    w = int(if x < 0 then x - 0.5 else x + 0.5)\nend\n"
        self.assertEqual(run(src, "f", [-2.7]), [-2, -3])
        self.assertEqual(run(src, "f", [2.5]), [2, 3])

    def test_bit_operations_are_exact(self):
        src = (pcode.ROOT / "design" / "pcode_selftest" / "selftest.pc").read_text()
        # the CRC against the same loop in Python, on the bytes of "1234"
        c, hi, lo, sw, r, d, m = run(src, "bits", [[49, 50, 51, 52], 0x1234, -10.5])
        self.assertEqual((hi, lo, sw, r, d, m), (0x12, 0x34, 0x3412, -11, -1, -4))
        crc = 0xFFFF
        for b in b"1234":
            crc ^= b << 8
            for _ in range(8):
                crc = ((crc << 1) ^ 0x1021) & 0xFFFF if crc & 0x8000 else (crc << 1) & 0xFFFF
        self.assertEqual(c, crc)

    def test_inputs_can_be_drawn_through_another_function(self):
        src = ("fn make(a: int in 0 .. 3) -> x: int\n    x = 100 + a\nend\n"
               "## inputs from: make\nfn g(x: int in 0 .. 3, y: int in 0 .. 3) -> z: int\n    z = x + y\nend\n")
        with tempfile.TemporaryDirectory() as d:
            f = pathlib.Path(d) / "t.pc"
            f.write_text(src)
            v = pcode.cli("vectors", str(f), "--n", "6")
        key = next(k for k in v if k.endswith("::g"))
        sets = v[key]["sets"]
        self.assertTrue(sets and all(100 <= s["in"][0] <= 103 and s["out"][0] == s["in"][0] + s["in"][1] for s in sets))

    def test_powers_are_repeated_multiplication(self):
        x = 1.1
        self.assertEqual(run("fn f(x: real[1]) -> y: real[1]\n    y = x^3\nend\n", "f", [x]), [(x * x) * x])


@unittest.skipUnless(NODE, "Node.js is not installed")
class Physics(unittest.TestCase):
    def test_the_physics_is_its_registry(self):
        self.assertEqual(pcode.registry_problems(pcode.cli("signatures", *map(str, pcode.sources()))), [])

    def test_every_sourced_fixture_holds(self):
        res = pcode.fixtures()
        self.assertGreaterEqual(len(res), 7)
        self.assertEqual([line for _, ok, line in res if not ok], [])

    def test_everything_generated_is_current(self):
        self.assertEqual(pcode.gen(check_only=True), [])


if __name__ == "__main__":
    unittest.main()
