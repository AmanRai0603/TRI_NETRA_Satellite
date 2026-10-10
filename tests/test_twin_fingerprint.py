"""The MATLAB twin names what it flew from (asils.util.fingerprint, docs/MAIN_APP.md): its source,
its case and its data, as Adler-32 over bytes with carriage returns removed, which the engine's
`adcs results stale` recomputes (store.rs, twin_tree_fp). Python's zlib.adler32 is the same sum, so
it holds both sides to one answer.

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import pathlib
import shutil
import subprocess
import unittest
import zlib

ROOT = pathlib.Path(__file__).resolve().parents[1]
MS = ROOT / "matlab_sils"


def fp(b):
    return "a32:%08x" % zlib.adler32(b.replace(b"\r", b""))


def files(top):
    return sorted(p.relative_to(MS).as_posix() for p in (MS / top).rglob("*") if p.is_file()
                  and not any(part.startswith(".") for part in p.relative_to(MS).parts))


def tree(what):
    if what == "source":
        fs = sorted(f for f in files("+asils") if f.endswith(".m"))     # the twin's code (POP is not, since S7.19b)
    else:
        fs = [f for f in files("data") if not f.endswith("_vectors.json")]
    return fp("".join(f"{f} {fp((MS / f).read_bytes())[4:]}\n" for f in fs).encode())


class Fingerprint(unittest.TestCase):
    def test_adler32_as_the_engine_writes_it(self):
        self.assertEqual(fp(b"Wikipedia"), "a32:11e60398")

    @unittest.skipUnless(shutil.which("octave"), "GNU Octave is needed to run the twin")
    def test_the_twin_computes_what_python_and_the_engine_compute(self):
        r = subprocess.run(["octave", "--no-gui", "-q", "--eval",
                            "addpath(pwd); startup_asils; f = @asils.util.fingerprint; "
                            "printf('%s\\n%s\\n%s\\n', f('source'), f('data'), f('file', 'cases/ais_3u.csv'))"],
                           cwd=MS, capture_output=True, text=True, timeout=300)
        got = [x for x in r.stdout.split() if x.startswith("a32:")]
        self.assertEqual(got, [tree("source"), tree("data"), fp((MS / "cases" / "ais_3u.csv").read_bytes())], r.stderr[-2000:])


if __name__ == "__main__":
    unittest.main()
