The first release of TRI-NETRA ADCS: the attitude determination and control design and test engine for satellites in low Earth orbit, as programs anyone can open.

**Start here:** download the app for your system, or the one Python package for any system, then read `START_HERE.md` (inside every kit). The first time, Windows and macOS ask once before opening an unsigned program; `FIRST_RUN.md` shows what to press. Check a download against `SHA256SUMS.txt`.

| download | for |
|---|---|
| `trinetra-adcs-1.0.0-windows-x86_64.zip` | Windows: the app **TRI-NETRA ADCS** and `adcs.exe` |
| `TRI-NETRA-ADCS-1.0.0-macos-arm64.zip` | macOS (Apple silicon): the app |
| `trinetra-adcs-1.0.0-macos-arm64.zip` | macOS: the command line and the data |
| `trinetra-adcs-1.0.0-linux-x86_64.zip` | Linux: `trinetra-app` and `adcs` |
| `trinetra_adcs-1.0.0-py3-none-any.whl` | any of them, with Python 3.8+: `pip install` it, then `trinetra-adcs-app` |
| `TRINETRA_ADCS_SILS_matlab_asils-1.0.0.zip` | the MATLAB / GNU Octave SILS twin |
| `TRINETRA_ADCS_flight_engine_1.2.0.zip` | the flight software (C and Rust) and the engine's source |
| `TRINETRA_ADCS_VV_report.pdf` | the verification and validation report |

**What it does.** Pick a case (two are included: a 3U AIS CubeSat and a 3U imager, both at 550 km) and a scenario (detumble, Sun acquisition, nadir hold, slews, faults, with magnetorquers, the fluid momentum loop, RCS, reaction wheels, CMGs); the engine flies the satellite, the environment and the precision orbit with the C or Rust flight software in the loop and judges every metric against the case's requirements. Every run records what it flew and can be sent as one `.trinetra` file.

**In this version.** Inputs the engine cannot fly are refused by name, never guessed; files are written whole; internal errors leave a crash report; every command is described before it runs (`adcs help`, `COMMANDS.md`); the whole repository is checked by one command and in CI on every change.

<!-- tn:generated:start (tools/release_notes.py) -->

## The design in this release

**0 of 20 groups accepted by their leads; 20 ship visibly UNCONFIRMED**, each with why (`docs/DELIVERY.md`). A node sealed UNCONFIRMED says so, with its reasons, in the node app and in `design.tndb`.

| Wave | Group | Version | Ships as | Accepted by, or why not |
|---|---|---|---|---|
| A | `case` | — | UNCONFIRMED | no release yet (its lead seals one in the group app) |
| A | `dyn` | — | UNCONFIRMED | no release yet (its lead seals one in the group app) |
| A | `env` | — | UNCONFIRMED | no release yet (its lead seals one in the group app) |
| B | `act` | — | UNCONFIRMED | no release yet (its lead seals one in the group app) |
| B | `sens` | — | UNCONFIRMED | no release yet (its lead seals one in the group app) |
| C | `ctl` | — | UNCONFIRMED | no release yet (its lead seals one in the group app) |
| C | `fdir` | — | UNCONFIRMED | no release yet (its lead seals one in the group app) |
| C | `gdn` | — | UNCONFIRMED | no release yet (its lead seals one in the group app) |
| C | `nav` | — | UNCONFIRMED | no release yet (its lead seals one in the group app) |
| D | `catalogue` | — | UNCONFIRMED | no release yet (its lead seals one in the group app) |
| D | `design` | — | UNCONFIRMED | no release yet (its lead seals one in the group app) |
| D | `fsw` | — | UNCONFIRMED | no release yet (its lead seals one in the group app) |
| D | `kpi` | — | UNCONFIRMED | no release yet (its lead seals one in the group app) |
| D | `pnt` | — | UNCONFIRMED | no release yet (its lead seals one in the group app) |
| E | `business` | — | UNCONFIRMED | no release yet (its lead seals one in the group app) |
| E | `hils` | — | UNCONFIRMED | no release yet (its lead seals one in the group app) |
| E | `lab` | — | UNCONFIRMED | no release yet (its lead seals one in the group app) |
| E | `oils` | — | UNCONFIRMED | no release yet (its lead seals one in the group app) |
| E | `risk` | — | UNCONFIRMED | no release yet (its lead seals one in the group app) |
| E | `vv` | — | UNCONFIRMED | no release yet (its lead seals one in the group app) |

## Known gaps

30 gaps in the register (`docs/ADCS_GAPS.md`), each with its evidence and what closes it; none gates this release.

- **D. Design (the design loop and the sizing):** D1 Unconfirmed case values; D2 Pointing error budget incomplete; D3 Sensor trade and star-tracker outages; D4 Imaging power margin; D5 Spare ring mass; D6 FDIR beyond the rings; D7 Lifetime and momentum dumping; D8 Thermal and data budgets; D9 Beyond 3U; D10 Sizing constants and ranking; D11 Spec package; D12 RCS detumble floor; D13 Coils-only modes on AIS; D14 Physics relations awaiting their bundles
- **S. SILS fidelity and statistics:** S1 Monte Carlo claims; S2 Device fidelity left; S3 Eclipse transitions; S4 Engine vs twin; S5 Twin-only chains; S6 Filter consistency
- **V. Validation and evidence:** V1 Propagator outside validation; V2 Algorithms against references; V3 Catalogue; V4 Algorithms confirmed; V5 IGRF-14
- **H. Hardware (OILS with a real OBC, HILS):** H1 Reference board; H2 Soft-OILS calibration; H3 Firmware leftovers; H4 HILS; H5 Safe-mode policy

<!-- tn:generated:end -->
