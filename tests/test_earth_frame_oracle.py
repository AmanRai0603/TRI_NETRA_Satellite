"""An external oracle for the truth Earth frame (docs/S12_INVENTORY.md S12.4, T5/E3 truth side; DR-env-002): the engine's
truth GCRF -> ITRF rotation, the one the loop gives the orbit's gravity, drag and field (adcs-sim Truth::Pop's context),
at three epochs of the span, against ERFA (pyerfa, the BSD release of the IAU SOFA library) given the same finals2000A.all
values.

ERFA's `c2t06a` takes no celestial pole offsets, and build A adds the file's dX and dY to the CIP; the oracle therefore
builds the same chain from ERFA's own pieces (xy06, s06, c2ixys, era00, sp00, pom00, c2tcio) with the offsets, and also
compares with `c2t06a` itself (the offsets, 0.3 mas or less, are inside its 1 mas). The EOP is read here from the committed
finals2000A.all by its fixed columns (Bulletin B where the day has it, else A), interpolated linearly, UT1 - UTC through the
leap-continuous UT1 - TAI; none of it is the engine's code. The old truth frame (the GMST rotation) is held to the same
oracle, so the test can fail: it is about 25 arcseconds off.

The engine's rotations are written by `cargo test -p adcs-sim --test earth_frame dump_the_truth_frame_for_the_external_oracle`
(ADCS_FRAME_DUMP names the file). The test is skipped, with its reason, only when pyerfa is not importable.

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import json
import math
import os
import pathlib
import subprocess
import tempfile
import unittest

import _path  # puts tools/ on the import path

_ = _path
try:
    import erfa
    import numpy as np
    SKIP = None
except ImportError as e:        # pragma: no cover
    erfa = np = None
    SKIP = f"pyerfa is not importable ({e}); pip install pyerfa (tools/requirements-ci.txt)"

ROOT = pathlib.Path(__file__).resolve().parents[1]
FINALS = ROOT / "engine" / "crates" / "adcs-pop" / "data" / "time_frames" / "finals2000A.all"
MAS = math.pi / 180 / 3600 / 1000
AS2R = math.pi / 180 / 3600


def field(line, a, b):
    t = line[a - 1:b].strip()
    return float(t) if t else None


def finals_rows():
    """[(mjd, x", y", UT1-UTC s, dX mas, dY mas)] of finals2000A.all, Bulletin B where the day has it, else A (dX, dY 0 when the
    day has none), by the columns of the file's own format note."""
    rows = []
    for line in FINALS.read_text(encoding="ascii").splitlines():
        line = line.ljust(187)
        mjd = field(line, 8, 15)
        if mjd is None:
            continue
        b = [field(line, *c) for c in ((135, 144), (145, 154), (155, 165), (166, 175), (176, 185))]
        a = [field(line, 19, 27), field(line, 38, 46), field(line, 59, 68), field(line, 98, 106), field(line, 117, 125)]
        got = b if b[0] is not None and b[2] is not None else a
        if got[0] is None or got[2] is None:
            continue
        dx = got[3] if got[3] is not None else (a[3] if a[3] is not None else 0.0)
        dy = got[4] if got[4] is not None else (a[4] if a[4] is not None else 0.0)
        rows.append((mjd, got[0], got[1], got[2], dx, dy))
    return rows


def eop_at(rows, mjd):
    """x, y [rad], UT1 - UTC [s], dX, dY [rad] at a UTC MJD: linear between the two days around it."""
    m = [r[0] for r in rows]
    i = max(0, min(len(rows) - 2, int(np.searchsorted(m, mjd, side="right")) - 1))
    f = (mjd - m[i]) / (m[i + 1] - m[i])
    lerp = lambda k: rows[i][k] + f * (rows[i + 1][k] - rows[i][k])          # noqa: E731
    # UT1 - UTC through UT1 - TAI, which no leap second breaks: the days' TAI - UTC from ERFA's own table
    dat = lambda r: erfa.dat(*erfa.jd2cal(2400000.5, r[0])[:3], 0.0)          # noqa: E731
    ut1_tai = [r[3] - dat(r) for r in (rows[i], rows[i + 1])]
    dut1 = ut1_tai[0] + f * (ut1_tai[1] - ut1_tai[0]) + dat((mjd,))
    return lerp(1) * AS2R, lerp(2) * AS2R, dut1, lerp(4) * 1e-3 * AS2R, lerp(5) * 1e-3 * AS2R


def erfa_frames(utc, rows, gmst_only=False):
    """(the chain with the offsets, c2t06a) GCRF -> ITRF at a UTC [Y Mo D h m s]; or, gmst_only, the old truth frame."""
    y, mo, d, h, mi, s = utc
    u1, u2 = erfa.dtf2d("UTC", int(y), int(mo), int(d), int(h), int(mi), float(s))
    mjd = u1 - 2400000.5 + u2
    xp, yp, dut1, dx, dy = eop_at(rows, mjd)
    if gmst_only:
        g = erfa.gmst82(u1, u2)
        c, s_ = math.cos(g), math.sin(g)
        r3 = np.array([[c, s_, 0], [-s_, c, 0], [0, 0, 1]])
        # the old truth: the orbit's frame was R3(gmst); the field's was R3(gmst) after the IAU 1976 precession
        return r3, r3 @ erfa.pmat76(u1, u2)
    ut11, ut12 = erfa.utcut1(u1, u2, dut1)
    t1, t2 = erfa.taitt(*erfa.utctai(u1, u2))
    x0, y0 = erfa.xy06(t1, t2)
    x, yy = x0 + dx, y0 + dy
    s06 = erfa.s06(t1, t2, x, yy)
    rc2i = erfa.c2ixys(x, yy, s06)
    rpom = erfa.pom00(xp, yp, erfa.sp00(t1, t2))
    full = erfa.c2tcio(rc2i, erfa.era00(ut11, ut12), rpom)
    return full, erfa.c2t06a(t1, t2, ut11, ut12, xp, yp)


def angle_mas(a, b):
    """The angle [mas] of the rotation that takes a to b."""
    m = np.asarray(a) @ np.asarray(b).T
    v = np.array([m[2, 1] - m[1, 2], m[0, 2] - m[2, 0], m[1, 0] - m[0, 1]])
    return math.atan2(0.5 * np.linalg.norm(v), 0.5 * (np.trace(m) - 1.0)) / MAS


@unittest.skipIf(SKIP is not None, SKIP)
class TruthFrameAgainstErfa(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        with tempfile.TemporaryDirectory() as d:
            out = pathlib.Path(d) / "frames.json"
            r = subprocess.run(["cargo", "test", "--release", "-q", "-p", "adcs-sim", "--test", "earth_frame",
                                "dump_the_truth_frame_for_the_external_oracle", "--", "--exact"],
                               cwd=ROOT / "engine", capture_output=True, text=True, env={**os.environ, "ADCS_FRAME_DUMP": str(out)})
            if r.returncode:
                raise RuntimeError(f"the engine's frames could not be written:\n{r.stdout[-1500:]}{r.stderr[-1500:]}")
            cls.dump = json.loads(out.read_text())
        cls.rows = finals_rows()

    def test_the_truth_frame_is_erfa_within_one_milliarcsecond_at_three_epochs(self):
        self.assertEqual(len({tuple(r["utc"][:3]) for r in self.dump}), 3, "three epochs")
        worst_full = worst_plain = 0.0
        for r in self.dump:
            full, plain = erfa_frames(r["utc"], self.rows)
            worst_full = max(worst_full, angle_mas(r["c"], full))
            worst_plain = max(worst_plain, angle_mas(r["c"], plain))
        print(f"\nthe truth GCRF -> ITRF against ERFA: {worst_full:.5f} mas (chain with dX, dY), {worst_plain:.5f} mas (c2t06a)")
        self.assertLess(worst_full, 1.0)
        self.assertLess(worst_plain, 1.0)

    def test_the_epochs_include_2027_01_01_and_the_old_frame_fails_the_same_oracle(self):
        self.assertTrue(any(tuple(r["utc"][:3]) == (2027.0, 1.0, 1.0) for r in self.dump), "2027-01-01 is one of the epochs")
        r = next(r for r in self.dump if tuple(r["utc"][:3]) == (2027.0, 1.0, 1.0))
        old_orbit, old_field = erfa_frames(r["utc"], self.rows, gmst_only=True)
        want = erfa_frames(r["utc"], self.rows)[0]
        orbit, field_ = angle_mas(old_orbit, want) / 1000.0, angle_mas(old_field, want) / 1000.0
        print(f"\nthe old truth frames against ERFA at 2027-01-01: the orbit's GMST rotation {orbit:.1f} arcsec, the field's GMST after precession {field_:.1f} arcsec")
        self.assertGreater(orbit, 1000.0, "the GMST-only rotation is more than 1000 arcseconds off (precession alone is about 1360)")
        self.assertGreater(field_, 5.0, "the GMST rotation after precession is several arcseconds off (nutation, UT1, polar motion; up to about 25 over the span)")


if __name__ == "__main__":
    unittest.main()
