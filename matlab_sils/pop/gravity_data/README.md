# gravity_data/

**Drop manually-downloaded gravity field files (`.gfc`) here.**

This folder is on the path (`setup_paths.m`), so a file placed here is reachable by
name:

```matlab
cfg.gravityField.field = 'gravity_data/EGM2008.gfc';   % explicit path
fld = grav.loadGFC('gravity_data/EGM2008.gfc', 70);    % or load it directly
```

## Empty is the normal state

This is **not** the cache. `data.gravity('EGM2008')` auto-downloads from ICGEM into
`data_cache/gravity/`, and that is where a successful automatic fetch lands. This
folder exists for the case where the automatic fetch **cannot** work:

- **No network** (an air-gapped machine, or ICGEM unreachable).
- **The ICGEM hash changed.** Their download URLs embed a content hash that rotates,
  so a hard-coded URL eventually 404s. `data.gravity` tries several and then errors
  with instructions pointing here — that error is the intended path to this folder,
  not a bug.

## What to put here

| file | degree | where |
|---|---|---|
| `EGM2008.gfc` | 2190 | https://icgem.gfz.de/tom_longtime → right-click the `gfc` link → Save link as |
| `EIGEN-6C4.gfc` | 2190 | same page |

For **orbit propagation** these two are equivalent (differences well below µm/s²).
Use EGM2008 unless you have a reason.

## Why the bundled default is not enough

`grav.defaultField()` ships a zonal J2–J6 field so the toolbox runs offline out of
the box. It **caps at degree 6**, and asking for more warns rather than silently
truncating:

```
requested gravity degree 20 exceeds the loaded field max degree 6
(field=zonal J2-J6 (EGM-consistent)) -- using 6.
```

If you see four identical rows in a `FORCES.gravity.degree` sweep, that warning is
why: every value above 6 collapsed to 6. The degree sweep needs a real field, and a
real field means either a network or a file in this folder.
