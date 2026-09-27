# Case keys
<!-- kind: reference; depth: expert -->

**In one line:** every key of the case format `adcs-case/1`, in the template's order, with its unit, what a blank does, and what it means; generated from the format itself, so it always matches the release you are using.

- **Blank**: *stated* means a blank leaves the value unstated, and whatever needs it is blocked, by name; *default* means a blank takes the reference value, and the report lists it as assumed.
- **Range**: the row takes `lo` and `hi`. **Level**: the row takes a level in per cent (requirements only; blank takes 99.73).
- **Sense** (requirements): *at most* means the achieved value must be at or under yours; *at least*, at or over.

## About the case

| Key | What | Notes |
|---|---|---|
| `meta.schema` | Case format | fixed; the importer refuses any other version it cannot migrate |
| `meta.case_id` | Case id | lower-case letters, digits and _; unique among the sender's cases |
| `meta.title` | Title | free text |
| `meta.class` | Satellite class | optional: a class id in catalogue/classes.toml; blank lets the solver infer it |
| `meta.families` | Families to search | optional: family ids joined by ';'; blank searches all four |

## What the ADCS must achieve

| Key | What | Unit | Sense | What it means |
|---|---|---|---|---|
| `req.ape` | Absolute pointing error (APE) | Degree | at most | largest angle allowed between where the payload axis points and where it should point, at the level given |
| `req.ake` | Absolute knowledge error (AKE) | Degree | at most | largest error allowed in what the ADCS believes its attitude is, at the level given |
| `req.rpe` | Relative pointing error (RPE) | Degree | at most | largest pointing wander allowed within a short window (jitter), at the level given |
| `req.pde` | Pointing drift error (PDE) | Degree | at most | largest slow pointing drift allowed over a long window (thermal, bias), at the level given |
| `req.rks` | Rate stability | DegreePerSecond | at most | largest body rate allowed while holding a target, at the level given |
| `req.rke` | Relative knowledge error (RKE) | Degree | at most | largest error allowed in the attitude change the ADCS believes happened over a short window |
| `req.slew` | Reference slew time | Second | at most | longest time allowed to turn through the reference slew angle (mission.sangle) |
| `req.settle` | Settling time after a slew | Second | at most | longest time allowed after a slew before pointing is back within its requirement |
| `req.spo` | Slews per orbit | Count | at least | fewest slews the ADCS must be able to make per orbit |
| `req.track` | Target-tracking rate | DegreePerSecond | at least | slowest rate at which the ADCS must be able to follow a moving target |
| `req.wmax` | Maximum body rate | DegreePerSecond | at least | lowest top body rate the ADCS must be able to reach |
| `req.detumble` | Detumble time | Minute | at most | longest time allowed from separation, at mission.w0, to a slow stable spin |
| `req.sunacq` | Sun-acquisition time in safe mode | Minute | at most | longest time allowed in safe mode to point the panels at the Sun |
| `req.faults` | Faults tolerated | Count | at least | fewest single failures the ADCS must survive and keep working |
| `req.recover` | Recovery time after a single fault | Minute | at most | longest time allowed to recover full service after one fault |
| `req.hsat` | Momentum saturation margin | One | at least | fraction of the stored momentum left unused, 0 to 1 |
| `req.dump` | Momentum dump interval | Hour | at least | shortest time allowed between momentum dumps |
| `req.mass` | ADCS mass | Kilogram | at most | heaviest the whole ADCS may be |
| `req.pavg` | ADCS orbit-average power | Watt | at most | most power the ADCS may draw on average over an orbit |
| `req.ppk` | ADCS peak power | Watt | at most | most power the ADCS may draw at any moment |
| `req.vol` | ADCS volume | Litre | at most | most volume the whole ADCS may take |
| `req.prop` | RCS propellant per year | Kilogram | at most | most thruster propellant the ADCS may use per year |

## Mission

| Key | What | Unit | Blank | Range | What it means |
|---|---|---|---|---|---|
| `mission.life` | Mission lifetime | Year | stated |  | how long the satellite must work in orbit |
| `mission.epoch` | Launch epoch | Year | stated |  | years after J2000.0: 27.0 is early 2027 |
| `mission.duty` | Fine-pointing duty cycle | One | stated |  | fraction of the orbit in fine pointing, 0 to 1 |
| `mission.spd` | Slews per day | Count | stated |  | slews per day |
| `mission.sangle` | Reference slew angle | Degree | stated |  | the slew angle the slew-time requirement is stated for |
| `mission.w0` | Initial tumble rate after separation | DegreePerSecond | stated | yes | body rate just after separation from the launcher; give lo and hi if the launcher states a range |

## Orbit

| Key | What | Unit | Blank | Range | What it means |
|---|---|---|---|---|---|
| `orbit.alt` | Altitude | Kilometre | stated | yes | mean altitude of a circular orbit |
| `orbit.inc` | Inclination | Degree | stated | yes | orbit inclination; about 97 to 98 degrees for sun-synchronous orbits at a few hundred km |
| `orbit.ecc` | Eccentricity | One | default |  | orbit eccentricity; blank means circular, and the report lists it as assumed |
| `orbit.ltan` | LTAN | Hour | stated | yes | local time of the ascending node, hours |

## Mass properties

| Key | What | Unit | Blank | Range | What it means |
|---|---|---|---|---|---|
| `mass.m` | Satellite mass | Kilogram | stated | yes | the whole satellite, wet, at launch |
| `mass.imax` | Largest principal inertia | KilogramSquareMetre | stated | yes | largest principal moment of inertia of the whole satellite |
| `mass.iint` | Intermediate principal inertia | KilogramSquareMetre | stated | yes | middle principal moment of inertia |
| `mass.imin` | Smallest principal inertia | KilogramSquareMetre | stated | yes | smallest principal moment of inertia |
| `mass.cm` | Centre-of-mass offset | Millimetre | stated | yes | centre-of-mass offset from the geometric centre |
| `mass.iunc` | Inertia uncertainty | One | stated |  | fraction, 0 to 1 |

## Surfaces

| Key | What | Unit | Blank | Range | What it means |
|---|---|---|---|---|---|
| `surface.afr` | Frontal area | SquareMetre | stated | yes | area facing the direction of flight, in the attitude it flies most |
| `surface.cpa` | Aerodynamic centre-of-pressure offset | Metre | stated | yes | distance between the centre of pressure for drag and the centre of mass |
| `surface.asun` | Sunlit area | SquareMetre | stated | yes | area facing the Sun, in the attitude it flies most |
| `surface.cps` | Solar centre-of-pressure offset | Metre | stated | yes | distance between the centre of pressure for sunlight and the centre of mass |
| `surface.refl` | Surface reflectivity | One | default | yes | fraction of sunlight the surfaces reflect, 0 to 1; blank takes the reference value, listed |
| `surface.cd` | Drag coefficient | One | default | yes | drag coefficient; blank takes the reference value, listed |

## Magnetic cleanliness

| Key | What | Unit | Blank | Range | What it means |
|---|---|---|---|---|---|
| `magnetic.dres` | Residual dipole | AmpereSquareMetre | stated | yes | the satellite's own magnetic dipole, from a magnetic test or a budget |
| `magnetic.dunc` | Residual dipole uncertainty | AmpereSquareMetre | stated |  | how uncertain that residual dipole is |

## Flexible modes

| Key | What | Unit | Blank | Range | What it means |
|---|---|---|---|---|---|
| `flex.fmode` | First flexible mode frequency | Hertz | stated | yes | lowest structural or panel mode frequency; leave blank if the satellite is rigid |
| `flex.mpart` | Modal participation | One | stated |  | fraction, 0 to 1 |

## Resources offered to the ADCS

| Key | What | Unit | Blank | Range | What it means |
|---|---|---|---|---|---|
| `resources.palloc` | Power allocated to the ADCS | Watt | stated |  | power the platform can give the ADCS |
| `resources.malloc` | Mass allocated to the ADCS | Kilogram | stated |  | mass the platform can give the ADCS |
| `resources.valloc` | Volume allocated to the ADCS | Litre | stated |  | volume the platform can give the ADCS |
| `resources.vbus` | Bus voltage | Volt | stated |  | voltage of the bus the ADCS is powered from |
| `resources.nif` | OBC data interfaces offered | Count | stated |  | OBC data interfaces offered to the ADCS |

## Pointing budget inputs

| Key | What | Unit | Blank | Range | What it means |
|---|---|---|---|---|---|
| `pointing.et` | Thermal distortion contribution | Degree | stated |  | pointing error from thermal distortion between the payload and the ADCS sensors, if known |
