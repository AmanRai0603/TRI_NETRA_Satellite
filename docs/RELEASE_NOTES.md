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
