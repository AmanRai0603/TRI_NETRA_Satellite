# Space weather

Real density models (NRLMSISE, JB2008, DTM2020) need solar/geomagnetic indices.
The exponential model does NOT (it is altitude-only). There is **no hardcoded
default** — you either fetch real indices or supply manual ones.

## Automatic (recommended)
When drag uses a real model, `op.buildWorld` fetches the indices for the
propagation window ONCE via `data.spaceweather(startDate,endDate)`:
- **F10.7 and its 81-day centred mean** from NASA/SPDF **OMNI2** (`get_f107`),
- **Kp / ap / Ap** from **GFZ Potsdam** (`get_gfz_hpo`),
normalised into one daily table and looked up per epoch by `atmos.spaceweather`.
For **JB2008** specifically, use `data.jb2008_indices` (SOLFSMY + DTCFILE from SET).

Everything caches under `data.root()/spaceweather`, so it downloads once.

## Manual (offline, or epoch not covered)
If the indices are unavailable for your epoch (offline, or a future/predicted date
not in the file), the code ERRORS and tells you to provide values:
```matlab
cfg.spaceweather.manual = struct('F107',120,'F107a',120,'Kp',2.0,'ap',7);
```
`TEMPLATE_propagation.m` shows both scenarios: it tries to fetch, and on failure
prints a message and switches to the manual struct you fill in.

## What each model consumes
| model | needs | provided by |
|-------|-------|-------------|
| exponential | nothing | — |
| NRLMSISE-00 | F10.7, F10.7a (81-d), ap (3-hourly + daily) | OMNI2 + GFZ |
| DTM2020 (oper) | F10.7, F10.7a, Kp | OMNI2 + GFZ |
| JB2008 | F10,S10,M10,Y10 (+81-d) + DSTDTC | `data.jb2008_indices` (SET) |

## Pseudocode (per step)
```
if drag model == exponential: atm = exponential(alt); return
sw = manual ? manual : lookup(SWtable, utc)      % SWtable loaded once for window
if sw not found and not manual: error "no SW for <date>; pass manual"
atm = density_model(geo, sw)
```
