"""The Python decoder of the flight software's config blob (adcs-fswcfg/1) reads what the engine
writes, field for field, and refuses a blob that is not whole rather than mis-reading it.
Copyright (c) 2026 Agastya. All rights reserved."""
import math
import os
import shutil
import struct
import subprocess
import tempfile
import unittest
import zlib

import _path  # puts tools/ on the import path
import fswcfg
from _path import ROOT

_ = _path  # imported for its effect: tools/ on sys.path

BIN = ROOT / "engine" / "target" / "release" / "adcs"


def encode(fields):
    """The layout of fsw/params/params.toml, written from a decoded dict: the inverse of decode."""
    pay = b""
    for f in fswcfg.SPEC["field"]:
        fmt = fswcfg.FMT[f["type"]][0]
        v = fields[f["name"]]
        flat = [x for row in v for x in row] if len(f.get("shape", [])) == 2 else (v if f.get("shape") else [v])
        pay += b"".join(struct.pack(fmt, x) for x in flat)
    return b"ADCSCFG1" + struct.pack("<I", len(pay)) + pay + struct.pack("<I", zlib.crc32(pay))


def blob_with(pay, n=None):
    return b"ADCSCFG1" + struct.pack("<I", len(pay) if n is None else n) + pay + struct.pack("<I", zlib.crc32(pay))


class Layout(unittest.TestCase):
    def test_the_layout_size_is_the_sum_of_its_fields(self):
        n = 0
        for f in fswcfg.SPEC["field"]:
            n += fswcfg.FMT[f["type"]][1] * math.prod(f.get("shape", []))
        self.assertEqual(fswcfg.SIZE, n)
        self.assertEqual(fswcfg.SPEC["schema"], "adcs-fswcfg/1")

    def test_a_synthetic_blob_round_trips(self):
        vals, k = {}, 0
        for f in fswcfg.SPEC["field"]:
            shape = f.get("shape", [])
            def one():
                nonlocal k
                k += 1
                return k * 0.5 if f["type"] == "f64" else k % 251
            if not shape:
                vals[f["name"]] = one()
            elif len(shape) == 1:
                vals[f["name"]] = [one() for _ in range(shape[0])]
            else:
                vals[f["name"]] = [[one() for _ in range(shape[1])] for _ in range(shape[0])]
        b = encode(vals)
        self.assertEqual(len(b), 16 + fswcfg.SIZE)
        self.assertEqual(fswcfg.decode(b), vals)


class Refusals(unittest.TestCase):
    def setUp(self):
        self.good = blob_with(bytes(fswcfg.SIZE))
        fswcfg.decode(self.good)

    def test_a_wrong_magic_is_refused(self):
        for b in (b"", b"ADCSCFG", b"ADCSCFG2" + self.good[8:], b"x" * 100):
            with self.assertRaisesRegex(ValueError, "not an adcs-fswcfg/1 blob"):
                fswcfg.decode(b)

    def test_a_truncated_blob_is_refused_as_a_bad_blob(self):
        for n in (8, 10, 12, 20, len(self.good) - 4, len(self.good) - 1):
            with self.assertRaises(ValueError, msg=n):
                fswcfg.decode(self.good[:n])

    def test_a_payload_length_that_is_not_the_layout_is_refused(self):
        for pay in (b"", bytes(fswcfg.SIZE - 1), bytes(fswcfg.SIZE + 8)):
            with self.assertRaisesRegex(ValueError, "payload length", msg=len(pay)):
                fswcfg.decode(blob_with(pay))

    def test_a_flipped_byte_fails_the_crc(self):
        for i in (12, 12 + fswcfg.SIZE // 2, 11 + fswcfg.SIZE, len(self.good) - 1):
            b = bytearray(self.good)
            b[i] ^= 0x01
            with self.assertRaisesRegex(ValueError, "CRC-32", msg=i):
                fswcfg.decode(bytes(b))


@unittest.skipUnless(BIN.is_file() and os.access(BIN, os.X_OK), "the engine is not built (engine/target/release/adcs)")
class EngineBlob(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.d = tempfile.mkdtemp()
        out = os.path.join(cls.d, "x.bin")
        subprocess.run([str(BIN), "params", "nadir_hold_ais", "--out", out], check=True, capture_output=True,
                       env=dict(os.environ, ADCS_ROOT=str(ROOT / "matlab_sils")), cwd=cls.d, timeout=60)
        with open(out, "rb") as f:
            cls.blob = f.read()

    @classmethod
    def tearDownClass(cls):
        shutil.rmtree(cls.d, ignore_errors=True)

    def test_the_engine_blob_decodes_every_field(self):
        d = fswcfg.decode(self.blob)
        self.assertEqual(list(d), [f["name"] for f in fswcfg.SPEC["field"]])
        self.assertEqual(len(self.blob), 16 + fswcfg.SIZE)

    def test_the_decoded_values_are_the_scenarios(self):
        d = fswcfg.decode(self.blob)
        self.assertEqual(d["dt"], 0.2)                       # nadir_hold_ais: time.dt_s
        self.assertEqual(d["mu"], 3.986004418e14)
        self.assertEqual(len(d["J"]), 3)
        self.assertTrue(all(d["J"][i][i] > 0 for i in range(3)))
        self.assertTrue(all(math.isfinite(x) for f in fswcfg.SPEC["field"] if f["type"] == "f64"
                            for x in (lambda v: [y for r in v for y in r] if isinstance(v, list) and v and isinstance(v[0], list)
                                      else (v if isinstance(v, list) else [v]))(d[f["name"]])))

    def test_decode_then_encode_is_the_engine_blob_byte_for_byte(self):
        self.assertEqual(encode(fswcfg.decode(self.blob)), self.blob)

    def test_the_engine_blob_damaged_is_refused(self):
        for bad in (self.blob[:-1], self.blob[:12], self.blob[:-4] + b"\0\0\0\0"):
            with self.assertRaises(ValueError):
                fswcfg.decode(bad)


if __name__ == "__main__":
    unittest.main()
