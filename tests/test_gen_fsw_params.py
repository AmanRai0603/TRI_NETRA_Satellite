"""The parameter generator refuses a definition it cannot write as C and Rust, by field.

Every field name becomes a struct member in both languages, every doc sits inside /* */ and
after ///, every shape becomes array bounds: a definition that would break any of them is
refused with the field named, before a file is written.

Copyright (c) 2026 Agastya. All rights reserved.
"""
import copy
import json
import unittest

import _path  # puts tools/ on the import path
import gen_fsw_params as G

_ = _path  # imported for its effect: tools/ on sys.path


class Definition(unittest.TestCase):
    def broken(self, i, **change):
        P = copy.deepcopy(G.P)
        P["field"][i].update(change)
        return G.check_definition(P)

    def test_the_committed_definition_is_writable(self):
        self.assertEqual(G.check_definition(G.P), [])

    def test_each_break_is_refused_by_field(self):
        cases = [
            ({"name": "bad name"}, "name must be"),
            ({"name": "struct"}, "keyword"),
            ({"name": "fn"}, "keyword"),
            ({"name": G.P["field"][0]["name"]}, "given twice"),
            ({"type": "f32"}, "is not one of"),
            ({"shape": [0]}, "shape"),
            ({"shape": [2, 2, 2]}, "shape"),
            ({"shape": [True]}, "shape"),
            ({"doc": "ends the comment */ early"}, "doc must be"),
            ({"doc": "two\nlines"}, "doc must be"),
            ({"unit": "m"}, "unknown key"),
        ]
        for change, words in cases:
            with self.subTest(change=change):
                bad = self.broken(3, **change)
                self.assertTrue(bad and all("field 4 " in b for b in bad), bad)
                self.assertTrue(any(words in b for b in bad), bad)

    def test_a_limit_out_of_range_is_refused(self):
        P = copy.deepcopy(G.P)
        P["max_rotors"] = 0
        self.assertTrue(any("max_rotors" in b for b in G.check_definition(P)))


class Igrf(unittest.TestCase):
    def test_the_committed_table_is_writable_and_breaks_are_refused(self):
        d = json.loads((G.ROOT / "matlab_sils" / "data" / "igrf13.json").read_text())
        self.assertEqual(G.check_igrf(d), [])
        short = copy.deepcopy(d)
        short["epochs"][0]["gh"] = short["epochs"][0]["gh"][:-1]
        self.assertTrue(any("194 coefficients" in b for b in G.check_igrf(short)))
        nan = copy.deepcopy(d)
        nan["epochs"][-1]["gh"][5] = float("nan")
        self.assertTrue(any("not a finite number" in b for b in G.check_igrf(nan)))


if __name__ == "__main__":
    unittest.main()
