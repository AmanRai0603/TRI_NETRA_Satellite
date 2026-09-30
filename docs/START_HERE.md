# Start here: TRI-NETRA ADCS

> **Answer first.** Open **TRI-NETRA ADCS** (the app), pick a case and a scenario, press
> **Fly it**, and read the verdicts: every metric against your satellite's requirement,
> PASS or FAIL. Or install the one Python file and type `trinetra-adcs-app`. Nothing needs
> the internet, and your runs are kept in `.trinetra/store` in your home folder.
>
> **Kind:** tutorial · **For:** whoever uses the tool

TRI-NETRA ADCS designs and tests the attitude determination and control of a satellite in
low Earth orbit. It flies the satellite, its environment and its orbit with the flight
software in the loop, the same C and Rust flight software that runs on the on-board
computer, and judges the result against the requirements in the case.

It comes with two cases: a 3U CubeSat for AIS at 550 km (coarse pointing, 10°) and a 3U
with an imager at 550 km (fine pointing). A case is one CSV file; see *Your own satellite*
below.

---

## 1 · Start it

**The app.** Choose one of these three:

| on | download | do this |
|---|---|---|
| Windows | `trinetra-adcs-<version>-windows-x86_64.zip` | right-click the zip → **Properties** → **Unblock** → **OK**; **Extract All**; open the folder and double-click **TRI-NETRA ADCS** |
| macOS | `TRI-NETRA-ADCS-<version>-macos-arm64.zip` | double-click the zip, drag **TRI-NETRA ADCS** into **Applications**, open it |
| Linux | `trinetra-adcs-<version>-linux-x86_64.zip` | unzip, then `./trinetra-app` in the folder |

The first time, Windows and macOS ask whether to open a program they do not know yet;
[FIRST_RUN.md](FIRST_RUN.md) shows each message and what to press.

**Or the Python package**, the same on every computer (Python 3.8 or newer):

    python -m pip install trinetra_adcs-<version>-py3-none-any.whl
    trinetra-adcs-app           # the app
    trinetra-adcs help          # the engine's command line

The app opens in your browser at `http://127.0.0.1:7788`. Only your computer can reach it.
**Quit** at the top of the page ends it; it also ends by itself a few minutes after you close
the page. Opening it again while it runs just opens the page again.

## 2 · Fly a scenario

1. **Case**: the satellite and its requirements.
2. **Scenario**: what it is asked to do (detumble, Sun acquisition, nadir hold, a slew, a
   fault) and for how long. The list shows the scenarios written for that case.
3. **Flight software**: C or Rust. They are the same algorithms and give bit-identical
   answers; either is fine.
4. **Duration**: leave it blank for the scenario's own; a shorter one gives a quick look.
5. **Fly it.** A scenario of a few orbits takes a second or two.

The verdicts table shows each metric, its value, its requirement and **PASS** or **FAIL**. A
metric with no requirement in the case is shown without a verdict.

## 3 · Keep and send a run

Every run is listed under **Your runs** with when it was flown and how many requirements it
passed and failed. **show** prints what it flew: the case, the scenario and every setting, each with a
fingerprint. **export** saves it as one `.trinetra` file to send to someone. They open it with

    trinetra-adcs results import <file>.trinetra --out <folder>

(or `adcs results import` from a kit), or unzip it: it is a plain zip file.

## 4 · The command line

Everything the app does, and more (sizing a satellite's actuators, running the flight
software on a virtual on-board computer, Monte Carlo seeds) is on the command line:

    adcs help                      every command, one line each
    adcs run nadir_hold_ais        fly a scenario
    adcs size ais_3u               size every actuator option for a case
    adcs results list              every run you have kept

From the Python package the command is `trinetra-adcs` instead of `adcs`. `COMMANDS.md`,
beside this file, describes every command and what it reads and writes.

## Your own satellite

A case is one CSV file in the fixed `adcs-case/1` format, one row per value (`orbit.alt`,
`mass.m`, the inertias, the surfaces, the requirements `req.*`). Copy one of the two cases,
change the values, and give it with `--case <file>`. A value left blank means *not stated*;
a value the engine cannot fly (text where a number belongs, an orbit outside low Earth orbit)
is refused with its name, never guessed.

## When something goes wrong

- **A message names a value and a range.** The input is outside what the engine models.
  Change that value.
- **The app shows "could not start".** The page says why; see [FIRST_RUN.md](FIRST_RUN.md).
- **An internal error.** A crash report is written to `.trinetra/log` in your home folder;
  send it to whoever looks after the tool.
