#!/usr/bin/env python3
"""Emit docs/figures/flow_design_to_hils.svg (clean, hand-readable SVG)."""
# Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
from xml.sax.saxutils import escape as esc

import pathlib
OUT = str(pathlib.Path(__file__).resolve().parents[2] / "docs" / "figures" / "flow_design_to_hils.svg")
W, H = 1400, 900
NAVY = "#1f2a44"
MUTED = "#475569"
BANDS = {
    "A": ("#0f766e", "#ecfdf5"),
    "B": ("#4338ca", "#eef2ff"),
    "C": ("#b45309", "#fffbeb"),
}
BW, BH = 295, 130
COLX = [50 + i * 335 for i in range(4)]

out = []
def a(s): out.append(s)

def node(num, col, y, band, title, lines, dashed=False):
    acc, _ = BANDS[band]
    x = COLX[col]
    dash = ' stroke-dasharray="6 4"' if dashed else ""
    a(f'  <g id="n{num}">')
    a(f'    <rect x="{x}" y="{y}" width="{BW}" height="{BH}" rx="10" fill="#ffffff" stroke="{acc}" stroke-width="1.5"{dash}/>')
    a(f'    <circle cx="{x+26}" cy="{y+26}" r="14" fill="{acc}"/>')
    a(f'    <text x="{x+26}" y="{y+31}" text-anchor="middle" font-size="13" font-weight="bold" fill="#ffffff">{num}</text>')
    a(f'    <text x="{x+50}" y="{y+32}" font-size="16" font-weight="bold" fill="{NAVY}">{esc(title)}</text>')
    if lines:
        a(f'    <text x="{x+16}" y="{y+62}" font-size="13" fill="{NAVY}">')
        for i, ln in enumerate(lines):
            dy = "0" if i == 0 else "18"
            a(f'      <tspan x="{x+16}" dy="{dy}">{esc(ln)}</tspan>')
        a('    </text>')
    a('  </g>')

def band(key, y, h, label):
    acc, fill = BANDS[key]
    a(f'  <rect x="30" y="{y}" width="1340" height="{h}" rx="14" fill="{fill}" stroke="{acc}" stroke-width="1.5" stroke-opacity="0.45"/>')
    a(f'  <text x="50" y="{y+24}" font-size="14" font-weight="bold" fill="{acc}" letter-spacing="0.5">{esc(label)}</text>')

def arrow(d, color=NAVY, dashed=False, marker="arr"):
    dash = ' stroke-dasharray="6 4"' if dashed else ""
    a(f'  <path d="{d}" fill="none" stroke="{color}" stroke-width="1.5"{dash} marker-end="url(#{marker})"/>')

def label(x, y, txt, color=MUTED, anchor="start", style=""):
    a(f'  <text x="{x}" y="{y}" font-size="12" fill="{color}" text-anchor="{anchor}"{style}>{esc(txt)}</text>')

a('<?xml version="1.0" encoding="UTF-8"?>')
a(f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" width="{W}" height="{H}" '
  f'font-family="Helvetica, Arial, sans-serif" role="img" aria-labelledby="t d">')
a('  <title id="t">TRI-NETRA ADCS: from customer case to flight</title>')
a('  <desc id="d">End-to-end verification and validation flow: design loop (case, demand survey, sizing, SILS mode matrix, '
  'assess, converge, select), verification (dispatch, Monte Carlo and edge campaigns, soft OILS, V&amp;V report) '
  'and hardware rungs (OILS, HILS).</desc>')
a('  <defs>')
for mid, col in (("arr", NAVY), ("arrA", BANDS["A"][0])):
    a(f'    <marker id="{mid}" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="8" markerHeight="8" orient="auto-start-reverse">')
    a(f'      <path d="M0,0 L10,5 L0,10 z" fill="{col}"/>')
    a('    </marker>')
a('  </defs>')
a(f'  <rect x="0" y="0" width="{W}" height="{H}" fill="#ffffff"/>')

# Title
a(f'  <text x="30" y="42" font-size="24" font-weight="bold" fill="{NAVY}">TRI-NETRA ADCS: from customer case to flight</text>')
a(f'  <text x="30" y="66" font-size="14" fill="{MUTED}">End-to-end V&amp;V flow: design loop, software verification, then hardware-in-the-loop rungs</text>')

# ---- Band A
YA, R1, R2 = 86, 122, 300
band("A", YA, 368, "A  ·  DESIGN  (Rust engine + Python orchestration)")
node(1, 0, R1, "A", "Case", ["adcs-case/1: orbit, mass,", "surfaces, requirements"])
node(2, 1, R1, "A", "Demand survey", ["POP orbit; disturbance torques", "at 4 attitudes; magnetic field;", "detumble & slew momentum"])
node(3, 2, R1, "A", "Sizing", ["Solutions: MTQ, FMR fluid loop, N2O RCS", "Benchmarks: RW, CMG, VSCMG", "Sensor suite by knowledge class"])
node(4, 3, R1, "A", "SILS mode matrix", ["every mission mode × actuator option", "× seed: detumble, sun acquisition,", "sun referencing, nadir pointing"])
node(5, 3, R2, "A", "Assess", ["failing requirement → cause:", "authority, momentum, power,", "knowledge"])
node(6, 2, R2, "A", "Converge", ["resize margins / upgrade sensor", "or algorithm; repeat 3–5 until", "the selection is stable"])
node(7, 1, R2, "A", "Select", ["lightest solution family passing", "every mode; per-mode method,", "sensors, algorithms"])

# MATLAB side note (band A, col 0, row 2)
nx, ny = COLX[0], R2
a('  <g id="matlab-note">')
a(f'    <rect x="{nx}" y="{ny}" width="{BW}" height="{BH}" rx="10" fill="#f1f5f9" stroke="#475569" stroke-width="1.5" stroke-dasharray="4 3"/>')
a(f'    <text x="{nx+16}" y="{ny+30}" font-size="15" font-weight="bold" fill="#475569">MATLAB / Octave twin</text>')
a(f'    <text x="{nx+16}" y="{ny+56}" font-size="13" fill="{NAVY}">')
for i, ln in enumerate(["Design reference; POP v51 orbit,", "same models. The Rust engine is", "bit-identical on orbit / environment."]):
    a(f'      <tspan x="{nx+16}" dy="{0 if i == 0 else 18}">{esc(ln)}</tspan>')
a('    </text>')
a(f'    <text x="{nx+16}" y="{ny+116}" font-size="12" font-style="italic" fill="{MUTED}">side reference, not a pipeline step</text>')
a('  </g>')

# Band A arrows
for c in range(3):
    x1 = COLX[c] + BW + 2
    x2 = COLX[c + 1] - 2
    y = R1 + BH / 2
    arrow(f"M{x1},{y} L{x2},{y}")
cx3 = COLX[3] + BW / 2
arrow(f"M{cx3},{R1+BH+2} L{cx3},{R2-2}")                      # 4 -> 5
y = R2 + BH / 2
arrow(f"M{COLX[3]-2},{y} L{COLX[2]+BW+2},{y}")               # 5 -> 6
arrow(f"M{COLX[2]-2},{y} L{COLX[1]+BW+2},{y}")               # 6 -> 7
# loop 6 -> 3
lx = COLX[2] + BW / 2
arrow(f"M{lx},{R2-2} L{lx},{R1+BH+2}", color=BANDS["A"][0], dashed=True, marker="arrA")
label(lx + 10, (R1 + BH + R2) / 2 + 4, "not converged: resize", color=BANDS["A"][0], style=' font-weight="bold"')

# ---- Band B
YB, RB = 470, 506
band("B", YB, 190, "B  ·  VERIFICATION")
node(8, 0, RB, "B", "Dispatch", ["adcs-fswcfg/1 blob, BUILD.md,", "C + Rust flight software"])
node(9, 1, RB, "B", "MC + edge campaigns", ["Monte Carlo and edge cases on", "the Rust engine; compared with", "the MATLAB twin"])
node(10, 2, RB, "B", "Soft OILS", ["flight software as Cortex-M4F", "firmware in QEMU over adcs-link/1;", "exact instruction counts → latency", "inside the control period"])
node(11, 3, RB, "B", "V&V report", ["HTML + PDF"])
for c in range(3):
    arrow(f"M{COLX[c]+BW+2},{RB+BH/2} L{COLX[c+1]-2},{RB+BH/2}")
# 7 -> 8 elbow through the gap between bands
c7 = COLX[1] + BW / 2
c8 = COLX[0] + BW / 2
arrow(f"M{c7},{R2+BH+2} L{c7},{YA+368+8} L{c8},{YA+368+8} L{c8},{RB-2}")
label(c7 + 10, R2 + BH + 18, "flight configuration", anchor="start")

# ---- Band C
YC, RC = 676, 712
band("C", YC, 190, "C  ·  HARDWARE  (next rungs)")
node(12, 3, RC, "C", "OILS", ["real OBC on adcs-link/1", "via tcp / serial, --realtime"], dashed=True)
node(13, 2, RC, "C", "HILS", ["Helmholtz cage, Sun simulator,", "air bearing; one device at a time"], dashed=True)
c11 = COLX[3] + BW / 2
arrow(f"M{c11},{RB+BH+2} L{c11},{RC-2}")                       # 11 -> 12
arrow(f"M{COLX[3]-2},{RC+BH/2} L{COLX[2]+BW+2},{RC+BH/2}")     # 12 -> 13
# flight terminal
fx, fw, fh = COLX[1] + 45, 205, 56
fy = RC + BH / 2 - fh / 2
arrow(f"M{COLX[2]-2},{RC+BH/2} L{fx+fw+2},{RC+BH/2}")
a(f'  <rect x="{fx}" y="{fy}" width="{fw}" height="{fh}" rx="28" fill="{NAVY}"/>')
a(f'  <text x="{fx+fw/2}" y="{fy+35}" text-anchor="middle" font-size="18" font-weight="bold" fill="#ffffff">Flight</text>')

# legend (band C, col 0)
lgx, lgy = COLX[0], RC + 6
a('  <g id="legend" font-size="12">')
a(f'    <text x="{lgx}" y="{lgy+12}" font-size="13" font-weight="bold" fill="{NAVY}">Legend</text>')
a(f'    <line x1="{lgx}" y1="{lgy+34}" x2="{lgx+40}" y2="{lgy+34}" stroke="{NAVY}" stroke-width="1.5" marker-end="url(#arr)"/>')
a(f'    <text x="{lgx+52}" y="{lgy+38}" fill="{NAVY}">flow to the next step</text>')
a(f'    <line x1="{lgx}" y1="{lgy+58}" x2="{lgx+40}" y2="{lgy+58}" stroke="{BANDS["A"][0]}" stroke-width="1.5" stroke-dasharray="6 4" marker-end="url(#arrA)"/>')
a(f'    <text x="{lgx+52}" y="{lgy+62}" fill="{NAVY}">design iteration (repeat 3–5)</text>')
a(f'    <rect x="{lgx}" y="{lgy+74}" width="40" height="18" rx="4" fill="#ffffff" stroke="{BANDS["C"][0]}" stroke-width="1.5" stroke-dasharray="6 4"/>')
a(f'    <text x="{lgx+52}" y="{lgy+87}" fill="{NAVY}">hardware rung, next step</text>')
a(f'    <rect x="{lgx}" y="{lgy+100}" width="40" height="18" rx="4" fill="#f1f5f9" stroke="#475569" stroke-width="1.5" stroke-dasharray="4 3"/>')
a(f'    <text x="{lgx+52}" y="{lgy+113}" fill="{NAVY}">reference model (side note)</text>')
a('  </g>')

a(f'  <text x="1370" y="{H-16}" text-anchor="end" font-size="11" fill="{MUTED}">TRI-NETRA ADCS · Agastya</text>')
a('</svg>')

open(OUT, "w", encoding="utf-8").write("\n".join(out) + "\n")
print("wrote", OUT)
