# Atmosphere / density

Drag needs a density (and temperature/species) at the spacecraft. The provider
returns a struct consumed directly by `drag.force`.

```matlab
atm = atmos.provider(model, geo, sw);
% geo: .alt_km .lat_deg .lon_deg .lst_h .doy .utc
% sw : space weather .F107 .F107a .Kp .ap (real models only)
```

| model | dependency | output |
|-------|-----------|--------|
| `exponential` | none (offline) | `.rho .T .Mmol .nO` (Vallado exponential table) |
| `nrlmsise` | Aerospace Toolbox `atmosnrlmsise00` | full species `.n.*` + `.T .rho` |
| `jb2008` | GOCE-study `jb2008_density` + SOLFSMY/DTCFILE | `.rho .T .nO` |
| `dtm2020` | GOCE-study `dtm2020_oper_density` + coeff `.dat` | species + `.T .rho` |

The default `exponential` model is altitude-only (no diurnal/solar variation);
it is a shape/sanity model and can be off by a factor of a few in absolute VLEO
density. For real work point `cfg.forces.drag.atmos` at one of the GOCE-study
models and make sure that study's code + data files are on the MATLAB path.

There is **no silent fallback**: if a real model's code or data is missing,
`atmos.provider` lets the error propagate so you know exactly what to install or
fetch. `exponential` is a *deliberate* selection (altitude-only, no space
weather), never an automatic substitute. Space-weather indices are fetched
automatically for real models (see SPACEWEATHER.md) or supplied manually.

### Wiring the GOCE models
Add to path (example):
```matlab
addpath(genpath('.../GOCE_density_study/3_density_models'));
```
Then:
```matlab
cfg.forces.drag = struct('on',true,'model','dria','atmos','nrlmsise', ...
                         'gsi',struct('Tw',300),'corotate',true);
% real space weather is auto-fetched (OMNI2+GFZ) for the propagation window;
% to override with manual values instead:
cfg.spaceweather.manual = struct('F107',150,'F107a',150,'Kp',3,'ap',6);
```
