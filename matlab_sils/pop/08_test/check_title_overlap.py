#!/usr/bin/env python3
"""Detect title collisions by finding text rows in the top band of each figure.

Round 40's check ran on OD_acc_forces, the one figure with a single axes and no
subplot title above it -- so it passed while every 3-panel figure was still
colliding. A check that cannot fail is not a check. This one runs on ALL of them.
"""
import sys, os
import numpy as np
from PIL import Image

d = sys.argv[1]
print(f"  {'figure':<22} {'blocks':>6} {'min gap px':>11}  verdict")
print("  " + "-"*58)
bad = 0
for f in sorted(os.listdir(d)):
    if not f.endswith('.png'): continue
    im = np.array(Image.open(os.path.join(d,f)).convert('L'))
    h, w = im.shape
    band = im[:int(0.10*h), :]
    rows = (band < 100).sum(axis=1)
    ink = [i for i,v in enumerate(rows) if v > 2]
    if not ink:
        print(f"  {f[:-4]:<22} {'0':>6} {'-':>11}  no text")
        continue
    runs=[]; st=ink[0]; pv=ink[0]
    for i in ink[1:]:
        if i-pv > 2: runs.append((st,pv)); st=i
        pv=i
    runs.append((st,pv))
    gaps=[runs[i+1][0]-runs[i][1] for i in range(len(runs)-1)]
    mg = min(gaps) if gaps else None
    if len(runs) == 1 and (runs[0][1]-runs[0][0]) > 0.05*h:
        v = "MERGED - text blocks are touching"; bad += 1
    elif mg is not None and mg < 4:
        v = "COLLIDING"; bad += 1
    else:
        v = "ok"
    print(f"  {f[:-4]:<22} {len(runs):>6} {str(mg) if mg is not None else '-':>11}  {v}")
print(f"\n  {bad} figure(s) with colliding titles")
