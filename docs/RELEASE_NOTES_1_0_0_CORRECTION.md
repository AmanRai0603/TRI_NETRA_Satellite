# Correction to the 1.0.0 release notes

> **What this is.** This is the text to add to the published v1.0.0 release on GitHub (Releases → v1.0.0 → Edit), under
> its notes, as a section named "Correction (5 Oct 2026)". It comes from the technical audit
> (`docs/TECHNICAL_ROADMAP.md` §0). It changes no binary. The fixes come in 2.0.0 (`docs/PLAN_2_0.md`, S11).
>
> **Status:** draft for the owner's approval (S0)

---

## Correction (5 Oct 2026)

An end-to-end technical audit after release found that some of the verdicts 1.0.0 reports are wrong, or claim more
than its evidence shows.

**The 1.0.0 programs are not affected.** They fly the models and the flight software as written, and their
numbers can be reproduced. What is corrected here is **what those numbers were said to prove**.

**What to read differently**

| 1.0.0 says | Read instead | Why |
|---|---|---|
| The APE and AKE closures **pass** for both reference cases (`results/EVALUATION.md`) | **Not demonstrated** | The design loop's 12-run Monte Carlo makes no probability claim, and the evaluation took the worst of 12 runs as a pass against a 99.73 % requirement. Twelve passing runs show only R ≥ 0.78 at 95 % confidence. The 1,109-run `mc_fine_img` campaign meets the APE requirement in 93.8 % of runs, not 99.73 %, as `results/TRACEABILITY.md` already says. The `ais_3u` pass is on a design marked not feasible |
| Orbit-average power passes `req.pavg` (0.35 W ≤ 2.0 W imaging; 0.12 W ≤ 0.5 W AIS) | **Actuator power only** | The power metric adds the coils, wheels and thrusters, and leaves out the sensors: 1.6 W on the AIS case and 2.6 W on the imaging case (GNSS, star tracker, gyro, Sun sensors, magnetometer). With them, every `ais_3u` actuator family is above 0.5 W, and the imaging selection is above 2.0 W |
| CMG and VSCMG benchmarks: about 1.4 kg and 5.9 W, failing power | **Four times too high** | The catalogue entry is a cluster of four CMGs, but the sizing counted it as one of four units. Its mass, power, volume and momentum are four times what they should be, so the comparison with the other families is not valid |

**Also wrong, not changing a verdict**
- `results/ENGINE_CAMPAIGNS.md` says the engine is the more conservative of engine and MATLAB twin. The twin flew 20–40
  runs per campaign and the engine 1,109. On `ape_los_p9973`, the twin passes 29 % of runs and the engine 95 %: the
  engine is the *less* conservative.
- `results/ENGINE_PARITY.md` says the magnetic field is bit-identical between engine and twin. It compares only the
  field's magnitude. Its direction differs by up to 0.23°, because the twin was not moved to the engine's frame.
- A failed fluid momentum ring keeps its momentum in the model, so the single-fault survivability of the
  ring-based family is optimistic.
- The PID and magnetic-PD loops fly at about 0.7 times the bandwidth they are tuned to.
- About 4 % of Monte Carlo inertia draws, and every edge-campaign inertia corner, are physically impossible bodies.

**What happens next**
- All of the above are fixed in 2.0.0, each with a test that failed before the fix.
- Until then, treat the reference cases' KPI verdicts as **analysis in progress**, not as a demonstration of
  compliance.
