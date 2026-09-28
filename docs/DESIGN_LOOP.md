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

| failure | cause class | what the loop changes (in this order) |
|---|---|---|
| time, APE, Sun angle | performance | 1. every algorithm of the option's slot, and for rotor/fluid pointing the bandwidth-tuned laws (`pid@bw2.5`, `pid@bw4`)<br>2. fluid loop: a quieter flow sensor, 2 → 0.5 → 0.12 → 0.05 mm/s (1σ), the loop's in-house sensor requirement<br>3. the option's actuator authority ×1.5 (bounds ×0.5 … ×4), undone if the violation does not fall by 5 % |
| rate stability (fine class) | performance | gyro grade: noise ×0.3, then ×0.1, at mass and power ÷ grade (fibre-optic class); undone if rate stability does not improve |
| AKE | knowledge | the star tracker on a coarse-class product |
| mean / peak power on a fluid-loop option | power | the electromagnetic pump with more copper: λ ×3 (bound 3 kg/W); not when the power is the RCS valves' |
| mean power, coils-only family, performance passing | power | coil authority ×0.75 |
| power on a rotor | power | blocked: the rotor's standby power is the floor |
| mass budget of the closest solution family | mass | 1. one star-tracker head instead of two<br>2. a lighter pump (λ ÷3) while power allows<br>3. less fluid-loop momentum (×0.75)<br>each undone, and its lever closed, if it breaks a mode of that family or raises its requirement violation by more than 5 % |
| propellant | propellant | RCS ×1.5 |

A part that one failure pushes up and another pushes down is frozen and reported as a
conflict. The loop stops when no change is proposed. `results/DESIGN_<case>.md` lists every
iteration, every change and, for the last one, why each remaining failure cannot be fixed by a knob.

## The fluid loop's electromagnetic pump (`engine/crates/adcs-design/src/empump.rs`)

Our fluid loop is pumped by a DC conduction (Faraday) pump with an electromagnet. There is no
permanent magnet. The loop and its pump are designed together, per ring:

- **Loop.** h = N ρ A 2 S v. The loop cruises at 0.4 v_max; friction is Darcy (64/Re laminar),
  and a torque τ needs ρ L dv/dt of pressure.
- **Pump.** The bore is flattened to a b × a duct in the magnet gap, and Δp = B I / b. The
  electrode current is at most 10 A, which a practical low-voltage driver can supply.
  - Fluid resistance: R = ρ_e a / (b L_p).
  - Electrical power: P_e = (I² R + Δp Q) / (0.7 bypass × 0.85 driver).
  - Torque capacity: Δp_max × 2 S A / L. This covers both holding momentum against viscous
    spin-down and accelerating the fluid.
- **Electromagnet.** C-core; gap = b + 1 mm of walls; NI = 1.3 B g / μ0.
  - Coil power × copper mass is fixed by the geometry. For a mass/power rate λ [kg/W], the best
    coil is m_cu = √(λK), P_c = √(K/λ).
  - The iron carries the flux at 1.2 T.
- **Design.** A grid over v_max, bore, B and pump length keeps the design with the least
  mass + λ × steady power. The part records its Pareto front of mass against power over λ, and
  the design loop moves λ.

The engine flies the designed ring: coil power while the ring holds momentum, electrode power
from the hydraulic load and the designed efficiency, and the pressure-limited torque.

## Why the sizing is in Rust now

`adcs-design` is a port of `+asils/+sizing` with the laws unchanged. On the two cases its demand
survey and every part and family budget match the MATLAB sizing to 1e-14. It runs in well under a
second, where the MATLAB survey takes minutes, which is what makes a loop of many sizings possible.
The MATLAB sizing stays as the reference; `ADCS_SIZED_DIR` keeps the loop's products separate from
the MATLAB ones.
