#!/usr/bin/env python3
"""Emit docs/figures/architecture_languages.svg (clean, hand-readable SVG)."""
# Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
from xml.sax.saxutils import escape as esc

import pathlib
OUT = str(pathlib.Path(__file__).resolve().parents[2] / "docs" / "figures" / "architecture_languages.svg")
W = 1400
NAVY = "#1f2a44"
MUTED = "#475569"
PAL = {
    "flight": ("#4338ca", "#eef2ff"),
    "hw": ("#b45309", "#fffbeb"),
    "engine": ("#0f766e", "#ecfdf5"),
    "py": ("#0369a1", "#f0f9ff"),
    "mat": ("#475569", "#f1f5f9"),
    "out": ("#1f2a44", "#f8fafc"),
}
X0, X1 = 270, 1350      # child area
GAP = 14

out = []
def a(s): out.append(s)

def text_lines(x, y, lines, size=13, color=NAVY, anchor="start", weight=None, lh=18):
    fw = f' font-weight="{weight}"' if weight else ""
    a(f'    <text x="{x}" y="{y}" font-size="{size}" fill="{color}" text-anchor="{anchor}"{fw}>')
    for i, ln in enumerate(lines):
        a(f'      <tspan x="{x}" dy="{0 if i == 0 else lh}">{esc(ln)}</tspan>')
    a('    </text>')

def band(key, y, h, title, sub):
    acc, fill = PAL[key]
    a(f'  <rect x="30" y="{y}" width="1340" height="{h}" rx="14" fill="{fill}" stroke="{acc}" stroke-width="1.5" stroke-opacity="0.5"/>')
    titles = title if isinstance(title, list) else [title]
    text_lines(50, y + 30, titles, size=16, color=acc, weight="bold", lh=20)
    text_lines(50, y + 52 + 20 * (len(titles) - 1), sub, size=12, color=MUTED, lh=16)

def box(key, x, y, w, h, title, lines, center=True, tsize=15):
    acc, _ = PAL[key]
    a('  <g>')
    a(f'    <rect x="{x}" y="{y}" width="{w}" height="{h}" rx="8" fill="#ffffff" stroke="{acc}" stroke-width="1.5"/>')
    cx = x + w / 2 if center else x + 14
    anc = "middle" if center else "start"
    a(f'    <text x="{cx}" y="{y+26}" font-size="{tsize}" font-weight="bold" fill="{NAVY}" text-anchor="{anc}">{esc(title)}</text>')
    if lines:
        text_lines(cx, y + 47, lines, anchor=anc)
    a('  </g>')

def row(key, y, h, items, bh=None, pad=14, tsize=15):
    n = len(items)
    gap = 10
    w = (X1 - X0 - gap * (n - 1)) / n
    bh = bh or h - 2 * pad
    for i, (t, ls) in enumerate(items):
        box(key, round(X0 + i * (w + gap), 1), y + pad, round(w, 1), bh, t, ls, tsize=tsize)

a('<?xml version="1.0" encoding="UTF-8"?>')
body_start = len(out)
a('  <defs>')
a('    <marker id="arr" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="8" markerHeight="8" orient="auto-start-reverse">')
a(f'      <path d="M0,0 L10,5 L0,10 z" fill="{PAL["flight"][0]}"/>')
a('    </marker>')
a('  </defs>')
BG_IDX = len(out)
a('')  # placeholder for background rect

a(f'  <text x="30" y="42" font-size="24" font-weight="bold" fill="{NAVY}">Who runs what: languages and layers</text>')
a(f'  <text x="30" y="66" font-size="14" fill="{MUTED}">TRI-NETRA ADCS, from the flight contract at the top to the ground tooling and outputs at the bottom</text>')

y = 88
# 1. Pseudocode contract
h = 76
band("flight", y, h, "Pseudocode contract", ["fsw/pseudocode"])
acc = PAL["flight"][0]
a(f'  <rect x="{X0}" y="{y+14}" width="{X1-X0}" height="{h-28}" rx="8" fill="#ffffff" stroke="{acc}" stroke-width="1.5"/>')
a(f'  <text x="{(X0+X1)/2}" y="{y+h/2+5}" text-anchor="middle" font-size="14" fill="{NAVY}">'
  f'<tspan font-weight="bold">fsw/pseudocode 00–09</tspan>'
  f'<tspan> — the reference algorithm specification both flight implementations follow</tspan></text>')
y += h + GAP

# 2. Flight software
h = 100
band("flight", y, h, "Flight software", ["two implementations,", "one contract"])
bw = 400
box("flight", X0, y + 14, bw, h - 28, "Embedded C99", ["fsw/"], tsize=16)
box("flight", X1 - bw, y + 14, bw, h - 28, "Rust no_std", ["fsw-rs/"], tsize=16)
mx1, mx2 = X0 + bw, X1 - bw
mc = (mx1 + mx2) / 2
a(f'  <path d="M{mx1+10},{y+30} L{mx2-10},{y+30}" fill="none" stroke="{acc}" stroke-width="1.5" marker-start="url(#arr)" marker-end="url(#arr)"/>')
text_lines(mc, y + 52, ["same adcs_fsw.h ABI", "same adcs-fswcfg/1 blob", "bit-identical"], size=13, anchor="middle")
y += h + GAP

# 3. HAL boundary
h = 100
band("hw", y, h, "HAL boundary", ["adcs_hal.h, byte level"])
row("hw", y, h, [
    ("I2C", ["magnetometer 0x1E"]),
    ("SPI", ["gyro"]),
    ("I2C", ["Sun 0x60 · Earth 0x30"]),
    ("UART", ["star tracker · GNSS"]),
    ("CAN", ["rotors"]),
    ("PWM", ["coils"]),
])
y += h + GAP

# 4. Where the flight software runs
h = 150
band("hw", y, h, ["Where the flight", "software runs"], ["in-loop targets"])
# second title line: reuse band subtitle area for the real subtitle
row("hw", y, 0, [
    ("In-process", ["C / Rust linked into the engine"]),
    ("Virtual OBC", ["separate obc-posix process"]),
    ("QEMU Cortex-M4F", ["firmware image (soft OILS)"]),
    ("Real OBC", ["tcp / serial (OILS, HILS)"]),
], bh=74)
acc_hw = PAL["hw"][0]
by = y + 104
n, g = 4, 10
w4 = (X1 - X0 - g * (n - 1)) / n
for i in range(n):
    cx = X0 + i * (w4 + g) + w4 / 2
    a(f'  <line x1="{cx}" y1="{y+88}" x2="{cx}" y2="{by}" stroke="{acc_hw}" stroke-width="1.5"/>')
a(f'  <rect x="{X0}" y="{by}" width="{X1-X0}" height="32" rx="6" fill="{acc_hw}"/>')
a(f'  <text x="{(X0+X1)/2}" y="{by+21}" text-anchor="middle" font-size="14" fill="#ffffff">'
  f'<tspan font-weight="bold">adcs-link/1</tspan><tspan> — A5 5A frames, CRC16 — one protocol to every target</tspan></text>')
y += h + GAP

# 5. Rust engine
h = 112
band("engine", y, h, "Rust engine", ["engine/"])
row("engine", y, h, [
    ("adcs-pop", ["POP v51 port"]),
    ("adcs-sim-core", ["no_std plant, sensors,", "actuators, device codecs"]),
    ("adcs-sim", ["config, loop,", "metrics, recorder"]),
    ("adcs-design", ["demand, sizing,", "products"]),
    ("adcs-fsw-abi", ["bus + link"]),
    ("adcs-cli", ["adcs run | size |", "params | parity"]),
])
y += h + GAP

# 6. Python
h = 96
band("py", y, h, "Python", ["tools/"])
row("py", y, h, [
    ("pipeline.py", ["design loop nodes"]),
    ("engine.py", ["runs, MC, campaigns, parity, soft OILS"]),
    ("report.py + vv_report.py", ["HTML / PDF reports"]),
])
y += h + GAP

# 7. MATLAB / Octave
h = 96
band("mat", y, h, "MATLAB / Octave", ["matlab_sils/"])
row("mat", y, h, [
    ("Design twin", ["reference for the Rust engine"]),
    ("POP v51", ["orbit and environment"]),
    ("+asils packages", ["orbit, env, devices, FSW, campaigns"]),
])
y += h + GAP

# 8. Outputs
h = 96
band("out", y, h, "Outputs", ["what a run leaves behind"])
row("out", y, h, [
    ("results/*.md", ["parity, solutions, virtual OBC"]),
    ("results/index.html", ["browsable results index + figures"]),
    ("dist/", ["flight + engine zip, V&V report PDF"]),
])
y += h

H = y + 40
a(f'  <text x="1370" y="{H-14}" text-anchor="end" font-size="11" fill="{MUTED}">TRI-NETRA ADCS · Agastya</text>')
a('</svg>')

out[BG_IDX] = f'  <rect x="0" y="0" width="{W}" height="{H}" fill="#ffffff"/>'
header = [
    f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" width="{W}" height="{H}" '
    f'font-family="Helvetica, Arial, sans-serif" role="img" aria-labelledby="t d">',
    '  <title id="t">Who runs what: languages and layers</title>',
    '  <desc id="d">TRI-NETRA ADCS layers: pseudocode contract; C99 and Rust no_std flight software sharing one ABI; '
    'the byte-level HAL; the four places flight software runs over adcs-link/1; the Rust engine crates; '
    'Python tools; the MATLAB/Octave twin; and the outputs.</desc>',
]
out[body_start:body_start] = header
open(OUT, "w", encoding="utf-8").write("\n".join(out) + "\n")
print("wrote", OUT, "H =", H)
