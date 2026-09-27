# Precision Orbit Propagator (POP)

Self-contained, modular, heavily-commented **Cowell** orbit propagator in pure
MATLAB (Octave-compatible). Wraps your force-model, ephemeris, density and frame
modules behind one interface; every force can be toggled and every multi-model
force can be swapped; returns `r,v` at any UTC after the epoch. Includes a
**separate** TLE-based validation section, runnable examples, and a test suite.

```matlab
setup_paths
cfg = config.defaultConfig();
cfg.epoch=[2025 6 1 0 0 0]; cfg.r0=[6778137;0;0]; cfg.v0=[0;7668.6;0]; cfg.tspan=6000;
sol = op.propagate(cfg);
rv  = sol.stateAt(1234.5);      % state 1234.5 s after epoch
```

**Read `docs/00_START_HERE.md` first.** Topic docs: `FORCES`, `GRAVITY`,
`INTEGRATORS`, `ATMOSPHERE`, `FRAMES`, `VALIDATION`.

Quick map: `+op` engine · `+config` presets · `+forces` force terms ·
`+grav` gravity · `+integ` integrators · `+atmos` density · `+frames` ECI/ECEF ·
`ephemeris` DE440+time · `+validation` TLE cross-check · `examples/` · `test/`.

Run `run_all_tests` to verify the build. Everything ships **offline-capable**
(embedded zonal gravity, exponential atmosphere, GMST frame); switch to
high-degree `.gfc` gravity, GOCE density models, and IAU/EOP frames for precise
work — see the docs.

Requires MATLAB. No toolboxes needed for the default stack; the Aerospace
Toolbox and your GOCE_density_study models are optional upgrades (NRLMSISE,
high-degree toolbox gravity). Tested in MATLAB and GNU Octave 8.
