"""The OBC answers every frame it cannot take, so the engine never waits on a reply that will not come.

Each test speaks adcs-link/1 (fsw/targets/link/adcs_link.h) to the POSIX virtual OBC and sends
one bad frame: a bad CRC, a length over the limit, a CONFIG too short to hold its start time, a
TICK whose counts run past its payload or ask for more than the OBC holds, an unknown type. The
OBC must answer each with ACK and its LINK_E_* code, and still say goodbye afterwards.

Copyright (c) 2026 Agastya. All rights reserved.
"""
import struct
import subprocess
import unittest

import _path  # puts tools/ on the import path
import common

_ = _path

OBC = common.ROOT / "fsw" / "build" / "obc_posix"
ACK = 0x81
E_CRC, E_LONG, E_SHORT, E_FULL, E_TYPE = -91, -92, -93, -94, -99


def crc(b, c=0xFFFF):
    for x in b:
        c ^= x << 8
        for _ in range(8):
            c = ((c << 1) ^ 0x1021) & 0xFFFF if c & 0x8000 else (c << 1) & 0xFFFF
    return c


def frame(ty, p, n=None, good=True):
    h = bytes([ty]) + struct.pack("<H", len(p) if n is None else n) + p
    return b"\xA5\x5A" + h + struct.pack("<H", crc(h) ^ (0 if good else 1))


def tick(uart1=b"", n1=None, ncan=0, cans=b""):
    p = struct.pack("<Q", 0) + bytes([0]) + bytes(34)
    p += struct.pack("<H", len(uart1) if n1 is None else n1) + uart1
    p += struct.pack("<H", 0) + bytes([ncan]) + cans
    return p


@unittest.skipUnless(OBC.is_file(), "make -C fsw obc builds the virtual OBC")
class Link(unittest.TestCase):
    def talk(self, *frames):
        """Send the frames then BYE; return (type, rc) of every reply."""
        data = b"".join(frames) + frame(0x04, b"")
        out = subprocess.run([str(OBC)], input=data, capture_output=True, timeout=30).stdout
        replies, k = [], 0
        while k < len(out):
            self.assertEqual(out[k:k + 2], b"\xA5\x5A")
            ty, n = out[k + 2], struct.unpack_from("<H", out, k + 3)[0]
            replies.append((ty, struct.unpack_from("<i", out, k + 5)[0]))
            k += 7 + n
        self.assertEqual(replies[-1], (ACK, 0), "the OBC still says goodbye")
        return replies[:-1]

    def test_a_bad_crc_is_answered(self):
        self.assertEqual(self.talk(frame(0x03, b"\x01", good=False)), [(ACK, E_CRC)])

    def test_a_frame_over_the_limit_is_answered(self):
        self.assertEqual(self.talk(frame(0x03, b"", n=5000)[:7]), [(ACK, E_LONG)])

    def test_a_config_too_short_for_its_start_time_is_refused(self):
        self.assertEqual(self.talk(frame(0x01, b"\x00" * 4)), [(ACK, E_SHORT)])

    def test_a_tick_whose_counts_run_past_its_payload_is_refused(self):
        for p in (tick()[:-1], tick(n1=200), tick(ncan=2), tick() + b"\x00"):
            with self.subTest(n=len(p)):
                self.assertEqual(self.talk(frame(0x02, p)), [(ACK, E_SHORT)])

    def test_a_tick_beyond_what_the_obc_holds_is_refused(self):
        self.assertEqual(self.talk(frame(0x02, tick(ncan=33, cans=bytes(13 * 33)))), [(ACK, E_FULL)])
        self.assertEqual(self.talk(frame(0x02, tick(uart1=bytes(1025)))), [(ACK, E_FULL)])

    def test_an_unknown_type_is_answered(self):
        self.assertEqual(self.talk(frame(0x7E, b"")), [(ACK, E_TYPE)])


if __name__ == "__main__":
    unittest.main()
