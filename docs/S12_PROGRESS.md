# S12 progress: where the environment, frames, requirements and orbit stand (hand-over record)

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.

This is the working record of S12 (`docs/PLAN_2_0.md`, S12 Environment, frames, requirements, orbit (U1)). The steps are those of `docs/S12_INVENTORY.md` §6. Each session keeps this record current. `docs/S12_REPORT.md` is written from it at the phase's end. Everything here is the developer's and unsigned.

## Steps done (on `claude/brave-dijkstra-rshrb7`)

| step | items | commits | what it shows |
|---|---|---|---|
| S12 step 1 | the inventory | 4bfb8d6 | 44 items, 24 owner decisions each with a default, 18 steps (`docs/S12_INVENTORY.md`, `results/s12_inventory.json`). It was traced by three read-only agents and written by the main session. The done-when fails today in three ways. First, 4 of 17 environment models have a published-vector case (3 only in part). Second, 4 of 39 physics relations have a sourced vector. Third, six requirement blanks are filled by the code, and most requirement values are the developer's UNCONFIRMED reference. |
| S12.1 | the 24 owner decisions | dd601a3e, 4136359d | 16 new decision records and 2 amended (DR-gdn-002 tip-off refused until the launch provider states it; DR-design-003 growth and propellant), 36 in all, each on its default, proposed and unsigned. Each S12 question 1-24 is carried exactly once (`tests/test_decisions.py`). Records carry `owner_question_stage = "S12"`; DR-design-003 carries its S12 questions in `owner_question_s12`. Where a choice lands in a node not yet made, `sets` names it as a placeholder for its step. `verify_work --since 4bfb8d6` ok (all 7 checks, parity equal including oils and twin). |
| S12.3 | P4, TS1 (a), (b), (d), (e), (g) | see the merge commits after 4136359d | Analytic oracles, code only, all pass on today's code with no ignore. RK78 within 2.4e-6 m of closed-form Kepler over one period; orders RK4 4, Luther 5.9, DP45 5.3, RK78 8.2; GJ8 2 mm from RK78 on a J2 day; energy drift 1.6e-13 and 5.1e-12 over ten orbits; Hermite ratio 16 per halving. Precession rate to 1e-9; libration period 0.002 %; J2 node rate -0.055 %; sun-synchronous rate at its own inclination 0.012 %; umbra 0.046 s of Vallado's conical form; flat-plate SRP and drag torque to 1e-12. Each oracle was made to fail by one wrong constant. The `integrators` row joins `docs/references.toml`. |

## Open, found on the way

- **The environment's network allows package registries only.**
  - Available through packages: IGRF-14 (`ppigrf`, `pyIGRF14`), the IERS EOP and leap seconds (`astropy-iers-data`), CelesTrak SW-All (`spaceweather`) and the NRLMSIS references (`pymsis`).
  - EGM2008 (`icgem.gfz-potsdam.de`) is not reachable. S12.8 proposes the host when it is reached.
- **The engine and `fsw obc` were rebuilt at 914210de** (12 Nov).
- **TS1(c), the magnetic detumble decay rate, is not done.** Avanzini and Giulietti's decay-rate expression was not to hand, and the oracle is not written from memory. The inventory cites the paper as JGCD 2012 and the S12.3 brief as Acta Astronautica 77; one of the two is wrong. It needs the paper's expression or the owner's choice of the existing 2/3 · duty · k/I estimate. The 0.55-1.05 band at `adcs-sim/tests/engine.rs:43` stands.
- **Nystrom 4 converges at order 5 in position** for a force-only y'' = f(y). This is the method, not a defect; the test accepts 3.8-5.3 and says why.
- **The test's inclination, 97.6 deg at 550 km, is 0.05 deg off sun-synchronous** (the closed form gives 97.557 deg), so its node drifts 0.58 % fast. The cases derive their inclination from altitude and LTAN; S12.10 checks that the derivation uses the J2 closed form.
- **The moment-coefficient citation in TS1(g)** is to Moe and Moe at section level, with no equation number; it is to be checked against the paper.

## Steps left, in order

S12.2 and S12.4 to S12.18 of `docs/S12_INVENTORY.md` §6.
