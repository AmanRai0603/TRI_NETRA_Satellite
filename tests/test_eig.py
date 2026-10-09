"""The toolbox's eigenvalues (the pseudocode's `eig`, trinetra-toolbox/6, S7.15b; design/js/pcode.js `rt.eig`, which every
translation writes with the same arithmetic) against an implementation it was not written from: numpy's eigvals (LAPACK's
dgeev) on random matrices, and closed forms (a diagonal, a rotation, a companion matrix, a Jordan block's neighbour).
The translations are held to the interpreter bit for bit by the language's self-test (selftest::eigen,
tools/translators.py). Copyright (c) 2026 Agastya. All rights reserved."""
import json
import math
import shutil
import subprocess
import unittest

import numpy as np

import _path
from common import ROOT

_ = _path
NODE = shutil.which("node")
# a nan crosses as the word "nan" (JSON has none)
SCRIPT = ("import { rt } from %s; let s = ''; process.stdin.on('data', (d) => { s += d; });"
          "process.stdin.on('end', () => { const m = JSON.parse(s, (k, v) => (v === 'nan' ? NaN : v)).map((a) => rt.eig(a));"
          " process.stdout.write(JSON.stringify(m, (k, v) => (Number.isNaN(v) ? 'nan' : v))); });")


def eig(ms):
    """[[[re, im], ...], ...] of each matrix, by the interpreter."""
    url = json.dumps((ROOT / "design" / "js" / "pcode.js").as_uri())
    text = json.dumps([[["nan" if isinstance(x, float) and math.isnan(x) else x for x in row] for row in m] for m in ms])
    r = subprocess.run([NODE, "--input-type=module", "-e", SCRIPT % url], input=text, capture_output=True, text=True, check=True)
    return [[complex(float(a), float(b)) for a, b in e] for e in json.loads(r.stdout)]


def matched(got, want):
    """The largest distance from each eigenvalue to its nearest unmatched partner."""
    left, worst = list(want), 0.0
    for z in got:
        j = min(range(len(left)), key=lambda k: abs(left[k] - z))
        worst = max(worst, abs(left.pop(j) - z))
    return worst


@unittest.skipUnless(NODE, "needs node")
class Eig(unittest.TestCase):
    def test_random_matrices_agree_with_lapack(self):
        rng = np.random.default_rng(20261009)
        ms = []
        for n in (1, 2, 3, 4, 5, 6, 6, 6, 7, 8, 10):
            for k in range(12):
                a = rng.normal(size=(n, n))
                if k % 3 == 0:
                    a = a * np.exp(3 * rng.normal(size=(n, n)))       # badly scaled: the balancing's work
                if k % 4 == 1:
                    a = np.triu(a)
                ms.append(a.tolist())
        for m, e in zip(ms, eig(ms)):
            want = np.linalg.eigvals(np.array(m))
            self.assertEqual(len(e), len(m))
            self.assertLessEqual(matched(e, want), 1e-12 * max(1.0, float(np.abs(want).max())), m)

    def test_closed_forms(self):
        th = 0.3
        companion = [[10.0, -35.0, 50.0, -24.0], [1, 0, 0, 0], [0, 1, 0, 0], [0, 0, 1, 0]]       # (x-1)(x-2)(x-3)(x-4)
        got = eig([[[2.0, 0, 0], [0, -1.0, 0], [0, 0, 5.0]], [[math.cos(th), -math.sin(th)], [math.sin(th), math.cos(th)]], companion,
                   [[1.0, 1.0], [1e-20, 1.0]], [[7.5]]])
        self.assertEqual(sorted(z.real for z in got[0]), [-1.0, 2.0, 5.0])
        self.assertTrue(all(z.imag == 0 for z in got[0]))
        self.assertLessEqual(matched(got[1], [complex(math.cos(th), math.sin(th)), complex(math.cos(th), -math.sin(th))]), 1e-15)
        self.assertLessEqual(matched(got[2], [1, 2, 3, 4]), 1e-12)
        self.assertLessEqual(matched(got[3], [1 + 1e-10, 1 - 1e-10]), 1e-15)
        self.assertEqual(got[4], [7.5])

    def test_what_does_not_converge_is_nan(self):
        e = eig([[[float("nan"), 1.0], [1.0, 0.0]]])[0]
        self.assertTrue(all(math.isnan(z.real) and math.isnan(z.imag) for z in e))


if __name__ == "__main__":
    unittest.main()
