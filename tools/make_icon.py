#!/usr/bin/env python3
"""Draw the desktop app's icon: three eyes (tri-netra) on an orbit, as PNG, ICO and ICNS.

    python3 tools/make_icon.py        -> engine/crates/trinetra-app/icon/trinetra.{png,ico,icns}

Drawn, not traced: circles and an ellipse at 1024 px, scaled down for each size. Run it only
to change the icon; the three files are committed.
Copyright (c) 2026 Agastya. All rights reserved.
"""
import io
import struct

from PIL import Image, ImageDraw

from common import ROOT, write_bytes

OUT = ROOT / "engine" / "crates" / "trinetra-app" / "icon"
NAVY, SKY, GOLD, WHITE = (18, 34, 66, 255), (86, 156, 214, 255), (240, 180, 40, 255), (255, 255, 255, 255)


def draw(n=1024):
    im = Image.new("RGBA", (n, n), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    s = n / 1024
    d.rounded_rectangle([40 * s, 40 * s, 984 * s, 984 * s], radius=200 * s, fill=NAVY)
    # the orbit
    d.ellipse([130 * s, 360 * s, 894 * s, 664 * s], outline=SKY, width=int(34 * s))
    # three eyes: the two below, the third above them
    for cx, cy in ((352, 512), (672, 512), (512, 300)):
        r = 118
        d.ellipse([(cx - r) * s, (cy - r * 0.62) * s, (cx + r) * s, (cy + r * 0.62) * s], fill=WHITE)
        d.ellipse([(cx - 52) * s, (cy - 52) * s, (cx + 52) * s, (cy + 52) * s], fill=GOLD)
        d.ellipse([(cx - 22) * s, (cy - 22) * s, (cx + 22) * s, (cy + 22) * s], fill=NAVY)
    # the satellite on the orbit
    d.rectangle([760 * s, 610 * s, 830 * s, 680 * s], fill=GOLD)
    d.rectangle([690 * s, 632 * s, 750 * s, 658 * s], fill=SKY)
    d.rectangle([840 * s, 632 * s, 900 * s, 658 * s], fill=SKY)
    return im


def png(im, size):
    b = io.BytesIO()
    im.resize((size, size), Image.LANCZOS).save(b, "PNG", optimize=True)
    return b.getvalue()


def icns(im):
    # PNG-encoded entries: ic07 128, ic08 256, ic09 512, ic10 1024
    entries = b"".join(t + struct.pack(">I", len(p) + 8) + p for t, p in
                       ((b"ic07", png(im, 128)), (b"ic08", png(im, 256)), (b"ic09", png(im, 512)), (b"ic10", png(im, 1024))))
    return b"icns" + struct.pack(">I", len(entries) + 8) + entries


def ico(im):
    b = io.BytesIO()
    im.save(b, "ICO", sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)])
    return b.getvalue()


if __name__ == "__main__":
    im = draw()
    write_bytes(OUT / "trinetra.png", png(im, 512))
    write_bytes(OUT / "trinetra.ico", ico(im))
    write_bytes(OUT / "trinetra.icns", icns(im))
    print(f"wrote {OUT.relative_to(ROOT)}/trinetra.png, .ico, .icns")
