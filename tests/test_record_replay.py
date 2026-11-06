"""The link's recording (adcs run --record) and its replay (adcs replay), on one short soft-OILS scenario.

The recording holds every frame both ways with wall time and the run's verdict; replaying it on the same
firmware reproduces the OBC's answers and so the verdict, and a recording whose answers were altered is
refused. Needs the engine (cargo build --release -p adcs-cli) and the firmware (make -C fsw obc); skipped
without them. Copyright (c) 2026 Agastya. All rights reserved.
"""
import json
import shutil
import struct
import subprocess
import tempfile
import unittest
from pathlib import Path

import _path  # puts tools/ on the import path
import common

_ = _path

ADCS = common.ROOT / "engine" / "target" / "release" / "adcs"
FW = common.ROOT / "fsw" / "build"
MAGIC = b"ADCSREC1\n"
OUT, TICK = 0x82, 0x02


def records(b):
    assert b.startswith(MAGIC)
    k, out = len(MAGIC), []
    while k < len(b):
        wall, d, n = struct.unpack_from("<QBI", b, k)
        out.append((wall, d, k + 13, n))
        k += 13 + n
    return out


@unittest.skipUnless(ADCS.exists() and (FW / "obc_qemu.elf").exists() and (FW / "insn_count.so").exists() and shutil.which("qemu-system-arm"),
                     "needs the engine, the firmware and QEMU")
class RecordReplay(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.TemporaryDirectory()
        cls.rec = Path(cls.tmp.name) / "run.rec"
        cls.run_ = subprocess.run([str(ADCS), "run", "nadir_hold_ais", "--fsw", "qemu", "--oils", "--quiet", "--set", "engine.duration_s=3",
                                   "--out", str(Path(cls.tmp.name) / "out"), "--record", str(cls.rec)], capture_output=True, text=True)

    @classmethod
    def tearDownClass(cls):
        cls.tmp.cleanup()

    def test_the_recording_holds_the_tick_out_stream_with_wall_time(self):
        self.assertEqual(self.run_.returncode, 0, self.run_.stderr)
        b = self.rec.read_bytes()
        rs = records(b)
        types = [b[o + 2] for (_, d, o, _) in rs if d in (0, 1)]
        self.assertGreater(types.count(TICK), 10)
        self.assertEqual(types.count(OUT), types.count(TICK), "an OUT answers every TICK")
        walls = [w for (w, _, _, _) in rs]
        self.assertEqual(walls, sorted(walls))
        self.assertGreater(walls[-1], 0, "wall time is recorded")
        note = [json.loads(b[o:o + n]) for (_, d, o, n) in rs if d == 2]
        self.assertIn(note[0]["verdict"], ("PASS", "FAIL"))

    def test_the_replay_reproduces_the_verdict(self):
        r = subprocess.run([str(ADCS), "replay", str(self.rec), "--fsw", "qemu"], capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        b = self.rec.read_bytes()
        (o, n), = [(o, n) for (_, d, o, n) in records(b) if d == 2]
        verdict = json.loads(b[o:o + n])["verdict"]
        self.assertIn("0 differ", r.stdout)
        self.assertIn(f"verdict reproduced: {verdict}", r.stdout)

    def test_a_recording_with_altered_answers_is_not_reproduced(self):
        b = bytearray(self.rec.read_bytes())
        outs = [o for (_, d, o, n) in records(bytes(b)) if d == 1 and b[o + 2] == OUT]
        b[outs[3] + 5 + 6] ^= 0x7F          # a PWM command of the fourth OUT
        bad = Path(self.tmp.name) / "bad.rec"
        bad.write_bytes(bytes(b))
        r = subprocess.run([str(ADCS), "replay", str(bad), "--fsw", "qemu"], capture_output=True, text=True)
        self.assertEqual(r.returncode, 1, r.stdout + r.stderr)
        self.assertIn("not reproduced", r.stderr)


if __name__ == "__main__":
    unittest.main()
