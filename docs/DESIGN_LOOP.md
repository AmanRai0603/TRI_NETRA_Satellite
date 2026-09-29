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

Every node is a step with a file: `matlab_sils/store/pipeline/<case>/...`. Each node's inputs, outputs, parameters
and rules are defined once in `matlab_sils/data/pipeline/nodes.json`; the pipeline reads its parameters from there,
and `docs/NODES.md` is generated from it (`python3 tools/nodes_doc.py`).

| # | node | runs in | what it does | output |
|---|---|---|---|---|
| 1 | case | — | `adcs-case/1`: orbit, mass properties, surfaces, requirements | `matlab_sils/cases/<case>.csv` |
| 2 | demand | Rust (`adcs-design`) | one orbit on the POP orbit at four held attitudes (each axis on nadir, Sun referencing), the SILS torque models: peak disturbance, cyclic and secular momentum, weakest field, detumble and slew momentum | `iter_k/sized/sizing.json` → `demand` |
| 3 | size / select_rotor | Rust (`adcs size`) | MTQ, fluid loop, N2O RCS (ours) designed to the demand with the current **knobs**; RW, CMG, VSCMG (benchmarks) **chosen from the datasheet catalogue** (`matlab_sils/data/catalogue`, `docs/CATALOGUE.md`): the lightest model that meets the per-unit need; one product per family; mass / power / volume budget | `iter_k/sized/{parts,products}/` |
| 4 | matrix | Rust engine, C flight software | every mission mode × option × seed on the sized products (`ADCS_SIZED_DIR`); options that failed on performance also fly every algorithm of their slot | cached runs `cache/<hash>/` |
| 5 | assess | Python | per option: feasible on every seed? each failing requirement classed as performance, knowledge, power or propellant | `iter_k/assess.json` |
| 6 | converge | Python | the knob changes the failures call for (below); converged when nothing is left to change | `loop.json` |
| 7 | select | Python | every family scored on the same modes and budget; the **lightest feasible solution** family is selected (then steady power, volume); the benchmarks are ranked by the same rule and the best one is reported beside it. When no solution passes, the closest one is named with its gaps. | `selection.json` |
| 8 | dispatch | Rust | the selected family's mission (detumble → Sun acquisition → nadir), its adcs-fswcfg/1 blob, the converged sized products, a C and Rust check (bit-identical) | `dist/dispatch/<case>/<family>/` |
| 9 | mc | Rust engine | Monte Carlo of the dispatched mission with the case's dispersions | `mc/summary.json` |
| 10 | soft_oils | Rust engine + QEMU | the dispatched mission with the flight software as Cortex-M4F firmware (C and Rust), exact instruction timing, next to its SILS run (`docs/SOFT_OILS.md`) | `soft_oils.json` |
| 11 | report | Python | `results/DESIGN_<case>.md`; the full V&V report `tools/vv_report.py` | `results/`, `dist/` |

## The knobs and the rules

| failure | cause class | what the loop changes (in this order) |
|---|---|---|
| time, APE, Sun angle | performance | 1. every algorithm of the option's slot, and for rotor/fluid pointing the bandwidth-tuned laws (`pid@bw2.5`, `pid@bw4`)<br>2. fluid loop: a quieter flow sensor, 2 → 0.5 → 0.12 → 0.05 mm/s (1σ), the loop's in-house sensor requirement<br>3. the option's actuator authority ×1.5 (bounds ×0.5 … ×4), undone if the violation does not fall by 5 %; for a bought rotor this raises the need, so the next catalogue model up is chosen |
| rate stability (fine class) | performance | gyro grade: noise ×0.3, then ×0.1, at mass and power ÷ grade (fibre-optic class); undone if rate stability does not improve |
| AKE | knowledge | the star tracker on a coarse-class product |
| mean / peak power on a fluid-loop option | power | the electromagnetic pump with more copper: λ ×3 (bound 3 kg/W); not when the power is the RCS valves' |
| mean power, coils-only family, performance passing | power | coil authority ×0.75 |
| power on a rotor | power | blocked: the catalogue model's standby power is the floor |
| mass budget of the closest solution family | mass | 1. a lighter gyro (grade ×3 back towards the catalogue unit)<br>2. one star-tracker head instead of two<br>3. a lighter pump (λ ÷3) while power allows<br>4. less fluid-loop momentum (×0.75)<br>each undone, and its lever closed, if it breaks a mode of that family or raises its requirement violation by more than 5 % |
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

`adcs-design` is a port of `+asils/+sizing`. Its demand survey and the coil, fluid-loop and RCS laws
matched the MATLAB sizing to 1e-14 on both cases. The benchmarks' wheels and CMGs now come from the datasheet catalogue,
not from the MATLAB rotor laws. Those laws made a notional unit sized exactly to the demand, with mass
and power from a scaling anchor. A bought unit comes in fixed sizes, with the mass, power and volume on its datasheet.
The MATLAB twin keeps the laws only for its own reference scenarios. It runs in well under a
second, where the MATLAB survey takes minutes, which is what makes a loop of many sizings possible.
The MATLAB sizing stays as the reference; `ADCS_SIZED_DIR` keeps the loop's products separate from
the MATLAB ones.

## When a requirement changes

The case file is part of every cached run's key, so after a requirement changes the loop flies
the matrix again rather than reusing verdicts judged against the old value. Runs stored outside
the loop (SILS scenarios, campaigns, parity, soft OILS) do not need re-flying: a requirement is a
threshold on a recorded metric. `python3 tools/rescore.py` re-judges every stored metric record,
and every campaign's statistics, against the case files as they are now. After that, regenerate
the ledgers (`tools/engine.py twin-parity | campaign-ledger | oils-ledger`, `tools/report.py`,
`tools/vv_report.py`).

Example: on ais_img_3u the owner relaxed rate stability from 0.001 to 0.005 deg/s and set the ADCS
budget to 1.6 kg and 1.0 L (1U of the 3U). The loop converged in 7 iterations on `mtq_fmr` at
1.596 kg and 0.534 L. That design uses the catalogue gyro, two star-tracker heads, a 0.125 mm/s flow
sensor and λ = 0.033 kg/W.
