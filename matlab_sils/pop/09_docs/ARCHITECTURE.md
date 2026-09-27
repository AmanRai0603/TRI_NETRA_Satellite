# Architecture — functions (engine) vs scripts (what you run)

The toolbox is split on one clear principle so you always know what controls what:

## Engine = FUNCTIONS (reusable, you rarely edit)
Called by the scripts; each does one repeatable job. Do not need editing for a
normal study.
- `01_core/`  propagate, accel, integrators, gravity, config
- `02_forces/` every force model (+ `+dgeom`/`+sgeom` geometry)
- `03_frames_time/` frames, time, DE440 ephemeris
- `04_atmosphere/` density models
- `05_data/` fetch + cache (`+data`), catalog (`+sat`), credentials
- `06_validation/+validation/` SP3/TLE readers, `gps_reference`, `track_reference`,
  `validate_against_gps` (the seed→propagate→RTN engine)
- `11_compare/` analysis helpers used by scripts: `get_reference` (the fetcher
  TEMPLATE — pulls SP3 or TU Delft truth, and the place to add your own
  satellite's source), `accelerations_timeseries` (every |a| vs time),
  `inputs_report` (space-weather/EOP drivers), `save_results`
- **Figures are drawn inside each script** (in its FIGURES section), not in a
  shared folder — each script makes exactly the plots it needs and saves them to
  `data.root()/results/<tag>/`.

## Interface = SCRIPTS (you edit the top, run, read results)
Every user-facing item is a sectioned script with the SAME top-to-bottom flow:
`1 CONFIG · 2 MODEL SELECTION · 3 DATA/SATELLITE · 4 PROPAGATE · 5 COMPARE · 6 FIGURES`.
Change the settings at the top and re-run; the same script with different inputs
gives a different study.

| script | purpose |
|--------|---------|
| `TEMPLATE_propagation.m` | base propagation (copy this to start anything) |
| `06_validation/realsat/example_orbit_validation.m` | validate ONE satellite vs its GPS orbit (+figures) |
| `06_validation/realsat/run_gps_validation.m` | validate SEVERAL satellites, one force config |
| `06_validation/verify_access.m` | what data can I fetch (per satellite)? |
| `11_compare/compare_orbit.m` | compare MANY force configs vs the GPS orbit |
| `11_compare/compare_density.m` | compare density models vs measured |
| `11_compare/compare_16u.m` | future 16U prediction (solar-activity bracket) |
| `08_test/run_all_tests.m` | regression suite |
| `07_examples/ex01..ex06` | short focused demos |

## The rule of thumb
- Want a different **result**? Edit a **script** top section (satellite, date,
  forces, integrator) and run.
- Want different **behaviour of a model**? That lives in a **function** under the
  engine folders — but you rarely need to touch it.

So: functions are the machine; scripts are the control panel. You configure and
read everything from the scripts.
