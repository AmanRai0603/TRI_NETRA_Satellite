# Gravity

## Engine (`+grav/`)
- `twoBody(r,mu)` — point mass.
- `j2accel(r,mu,Re,J)` — fast analytic zonal J2..J6 (exact closed-form J2,
  higher zonals routed through the harmonic engine).
- `sphericalHarmonic(r,mu,Re,Cbar,Sbar,nmax,mmax)` — Cunningham/Montenbruck-Gill
  Cartesian V,W recursion, **pole-safe**. Input coefficients are fully
  normalised; the routine denormalises internally.
- `potential(r,mu,Re,Cbar,Sbar,nmax)` — geopotential value (for gradient checks).
- `defaultField()` — embedded zonal J2..J6, works **offline**, `mu=3.986004415e14`,
  `Re=6378136.3`.
- `loadGFC(file,maxdeg)` — reads ICGEM `.gfc` files (EIGEN/GOCO/GGM/EGM2008),
  static coefficients only, Fortran `D` exponents handled.

## Choosing a field
```matlab
cfg.gravityField = struct('field','default','degree',6);     % offline zonal
cfg.gravityField = struct('field','gravity_data/EGM2008.gfc','degree',70);
cfg.forces.gravity = struct('on',true,'model','sphharm','degree',70,'order',70);
```
Put `.gfc` files in `gravity_data/`. Download from ICGEM
(https://icgem.gfz-potsdam.de). For very high degree with tesserals in your own
MATLAB, `model='toolbox'` uses the Aerospace Toolbox
`gravitysphericalharmonic` with a `{C,S}` `.mat`.

## Accuracy notes
- Native `sphericalHarmonic` is validated against analytic J2 to ~1e-16 at all
  latitudes including the pole, and against finite-difference of `potential` to
  ~1e-9 relative (`test_gravity`).
- The internal denormalisation loses precision beyond ~degree 45 for tesserals;
  the code warns once. For degree >45 field work, prefer the `toolbox` path.

## Choosing the gravity field: EGM2008 and EIGEN-6C4 (both first-class)
Switch the field with one line; both are in the `data.gravity` registry and
download+cache once from ICGEM (or drop the `.gfc` in `gravity_data/` offline):
```matlab
cfg.gravityField = struct('field','EGM2008',  'degree',120);   % NGA EGM2008
cfg.gravityField = struct('field','EIGEN-6C4','degree',120);   % GFZ/GRGS EIGEN-6C4
cfg.gravityField = struct('field','default',  'degree',6);     % embedded zonal, offline
cfg.gravityField = struct('field','C:\path\my_model.gfc','degree',80); % any ICGEM .gfc
```
`op.gravLoad` resolves the name → fetches via `data.gravity` → parses with
`grav.loadGFC`. If an ICGEM URL ever 404s (they embed a version hash), pass
`cfg.gravityField.url` with the current ICGEM link, or download the `.gfc` and
give its path. Same coefficients power the `sphharm` force to whatever degree you
set — so EGM2008-vs-EIGEN-6C4 is a direct, tweakable comparison.
