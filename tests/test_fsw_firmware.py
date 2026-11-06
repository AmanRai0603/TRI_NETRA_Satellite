"""A stale firmware is refused by name, or rebuilt, and the real Makefile knows the parameter table is a source.

Copyright (c) 2026 Agastya. All rights reserved."""
import os
import subprocess
import tempfile
import time
import unittest
import unittest.mock
from pathlib import Path

import _path  # puts tools/ on the import path
import common
import fsw_firmware as F

_ = _path

TOY = """build/x.elf: src/a.c include/p.h | build
\tcat src/a.c include/p.h > $@
build:
\tmkdir -p build
"""


class Toy(unittest.TestCase):
    def setUp(self):
        self.t = tempfile.TemporaryDirectory()
        self.d = Path(self.t.name)
        (self.d / "src").mkdir()
        (self.d / "include").mkdir()
        (self.d / "src" / "a.c").write_text("a\n")
        (self.d / "include" / "p.h").write_text("p\n")
        (self.d / "Makefile").write_text(TOY)
        subprocess.run(["make", "-C", str(self.d), "build/x.elf"], check=True, capture_output=True)

    def tearDown(self):
        self.t.cleanup()

    def age_header(self):
        (self.d / "include" / "p.h").write_text("p2\n")
        old = time.time() - 100
        os.utime(self.d / "build" / "x.elf", (old, old))

    def test_a_current_firmware_is_accepted(self):
        self.assertEqual(F.stale(("x.elf",), self.d), [])
        F.require_fresh(("x.elf",), self.d)

    def test_a_firmware_older_than_a_header_is_refused_by_name(self):
        self.age_header()
        with self.assertRaises(F.FirmwareStale) as c:
            F.require_fresh(("x.elf",), self.d)
        self.assertIn("fsw/build/x.elf", str(c.exception))
        self.assertIn("adcs_params.h", str(c.exception))

    def test_a_missing_firmware_is_stale(self):
        (self.d / "build" / "x.elf").unlink()
        self.assertEqual(F.stale(("x.elf",), self.d), ["x.elf"])

    def test_ensure_rebuilds_a_stale_firmware(self):
        self.age_header()
        self.assertEqual(F.ensure(("x.elf",), self.d), ["x.elf"])
        self.assertEqual((self.d / "build" / "x.elf").read_text(), "a\np2\n")
        self.assertEqual(F.stale(("x.elf",), self.d), [])

    def test_ensure_refuses_by_name_when_it_cannot_build(self):
        self.age_header()
        (self.d / "src" / "a.c").unlink()
        (self.d / "Makefile").write_text(TOY.replace("cat src/a.c", "false src/a.c"))
        with self.assertRaises(F.FirmwareStale) as c:
            F.ensure(("x.elf",), self.d)
        self.assertIn("fsw/build/x.elf", str(c.exception))


class RealMakefile(unittest.TestCase):
    def test_the_parameter_table_is_a_source_of_every_firmware(self):
        for target in ("build/obc_qemu.elf", "build/obc_posix"):
            r = subprocess.run(["make", "-C", str(common.ROOT / "fsw"), "-n", "-W", "include/adcs_params.h", target], capture_output=True, text=True)
            self.assertIn("-o " + target, r.stdout, f"{target} is not rebuilt when include/adcs_params.h changes")


class Gate(unittest.TestCase):
    def test_the_parity_gate_refuses_a_stale_firmware_by_name_before_it_flies_anything(self):
        import parity_2_0
        was = F.ensure

        def refuse(*a, **k):
            raise F.FirmwareStale("the firmware fsw/build/obc_qemu.elf is older than its sources")
        F.ensure = refuse
        try:
            with unittest.mock.patch.object(parity_2_0, "ADCS", Path(__file__)):   # any existing file: the engine check passes
                with self.assertRaises(SystemExit) as c:
                    parity_2_0.main(["no.tndb", "--oils", "detumble_ais"])
        finally:
            F.ensure = was
        self.assertIn("fsw/build/obc_qemu.elf", str(c.exception))

    def test_the_gate_does_not_look_at_the_firmware_without_oils(self):
        import parity_2_0
        src = Path(parity_2_0.__file__).read_text()
        self.assertIn("if a.oils:\n        try:\n            fsw_firmware.ensure()", src)


if __name__ == "__main__":
    unittest.main()
