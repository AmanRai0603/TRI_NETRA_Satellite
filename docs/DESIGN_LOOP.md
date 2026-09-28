# The design loop: sizing that converges through the SILS

**Owner: Agastya.** Copyright (c) 2026 Agastya. All rights reserved.

![From customer case to flight](figures/flow_design_to_hils.svg)

Sizing is not the answer; it is the first guess. The loop sizes every actuator option to the
case, flies every mission mode with every option in the SILS, and lets what failed decide the
next sizing, sensor set or algorithm, until there is nothing left to change. What survives is
the selected actuators, sensors and algorithms, and they are dispatched and verified.

```
python3 tools/pipeline.py [case ...] [--seeds 1,2] [--max-iter 6] [--mc-runs 12] [--no-oils]
```

## Nodes

Every node is a step with a file: `matlab_sils/store/pipeline/<case>/...`.

| # | node | runs in | what it does | output |
|---|---|---|---|---|
| 1 | case | — | `adcs-case/1`: orbit, mass properties, surfaces, requirements | `matlab_sils/cases/<case>.csv` |
| 2 | demand | Rust (`adcs-design`) | one orbit on the POP orbit at four held attitudes (each axis on nadir, Sun referencing), the SILS torque models: peak disturbance, cyclic and secular momentum, weakest field, detumble and slew momentum | `iter_k/sized/sizing.json` → `demand` |
| 3 | size | Rust (`adcs size`) | MTQ, fluid loop, N2O RCS (ours), RW, CMG, VSCMG (benchmarks) sized to the demand with the current **knobs**; one product per family; mass / power / volume budget | `iter_k/sized/{parts,products}/` |
| 4 | matrix | Rust engine, C flight software | every mission mode × option × seed on the sized products (`ADCS_SIZED_DIR`); options that failed on performance also fly every algorithm of their slot | cached runs `cache/<hash>/` |
| 5 | assess | Python | per option: feasible on every seed? each failing requirement classed as performance, knowledge, power or propellant | `iter_k/assess.json` |
| 6 | converge | Python | the knob changes the failures call for (below); converged when nothing is left to change | `loop.json` |
| 7 | select | Python | the simplest **solution** family whose best usable option passes every mode and whose budget meets `req.mass` / `req.vol`; benchmarks scored the same way. When none passes, the closest one is named with its gaps. | `selection.json` |
| 8 | dispatch | Rust | the selected family's mission (detumble → Sun acquisition → nadir), its adcs-fswcfg/1 blob, the converged sized products, a C and Rust check (bit-identical) | `dist/dispatch/<case>/<family>/` |
| 9 | mc | Rust engine | Monte Carlo of the dispatched mission with the case's dispersions | `mc/summary.json` |
| 10 | soft_oils | Rust engine + QEMU | the dispatched mission with the flight software as Cortex-M4F firmware (C and Rust), exact instruction timing, next to its SILS run (`docs/SOFT_OILS.md`) | `soft_oils.json` |
| 11 | report | Python | `results/DESIGN_<case>.md`; the full V&V report `tools/vv_report.py` | `results/`, `dist/` |

## The knobs and the rules

| failure | cause class | knob change |
|---|---|---|
| time, APE, Sun angle, rate stability | performance | first fly every algorithm of the option's slot (detumble: B-dot gyro / mag / bang-bang / generalised; coils pointing: PD / LQR / SMC / rate damping; rotor pointing: PID / LQR / SMC; Sun acquisition: the Sun-spin variants); then the option's actuator authority ×1.5 (bounds ×0.5 … ×4) |
| AKE | knowledge | fit the star tracker on a coarse-class product |
| mean / peak power on the fluid loop | power | the permanent-magnet pump yoke (no field power, heavier yoke) |
| mean power on the coils-only family, performance passing | power | coil authority ×0.75 |
| power on a rotor | power | blocked: the rotor's standby power is the floor, no authority change helps |
| performance and power on the same option | — | blocked: conflict |
| propellant | propellant | RCS ×1.5 |

A part that one failure pushes up and another pushes down is frozen and reported as a conflict.
The loop stops when no change is proposed; `results/DESIGN_<case>.md` lists every iteration, every
change and, for the last one, why each remaining failure cannot be fixed by a knob.

## Why the sizing is in Rust now

`adcs-design` is a port of `+asils/+sizing` with the laws unchanged. On the two cases its demand
survey and every part and family budget match the MATLAB sizing to 1e-14. It runs in well under a
second, where the MATLAB survey takes minutes, which is what makes a loop of many sizings possible.
The MATLAB sizing stays as the reference; `ADCS_SIZED_DIR` keeps the loop's products separate from
the MATLAB ones.
