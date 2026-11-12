# S12 progress: where the environment, frames, requirements and orbit stand (hand-over record)

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.

This is the working record of S12 (`docs/PLAN_2_0.md`, S12 Environment, frames, requirements, orbit (U1)). The steps are those of `docs/S12_INVENTORY.md` §6. Each session keeps this record current. `docs/S12_REPORT.md` is written from it at the phase's end. Everything here is the developer's and unsigned.

## Steps done (on `claude/brave-dijkstra-rshrb7`)

| step | items | commits | what it shows |
|---|---|---|---|
| S12 step 1 | the inventory | this commit | 44 items, 24 owner decisions each with a default, 18 steps (`docs/S12_INVENTORY.md`, `results/s12_inventory.json`). It was traced by three read-only agents and written by the main session. The done-when fails today in three ways. First, 4 of 17 environment models have a published-vector case (3 only in part). Second, 4 of 39 physics relations have a sourced vector. Third, six requirement blanks are filled by the code, and most requirement values are the developer's UNCONFIRMED reference. |

## Open, found on the way

- **The environment's network allows package registries only.**
  - Available through packages: IGRF-14 (`ppigrf`, `pyIGRF14`), the IERS EOP and leap seconds (`astropy-iers-data`), CelesTrak SW-All (`spaceweather`) and the NRLMSIS references (`pymsis`).
  - EGM2008 (`icgem.gfz-potsdam.de`) is not reachable. S12.8 proposes the host when it is reached.
- **The engine and `fsw obc` were rebuilt at 914210de** (12 Nov).

## Steps left, in order

S12.1 to S12.18 of `docs/S12_INVENTORY.md` §6.
