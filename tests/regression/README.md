# The regression copy

`design.tndb` is the design the repository builds and tests against (`docs/PLAN_2_0.md` S4,
`docs/OPERATING_2_0.md` W15). The repository holds no design data of its own: every file the code still
reads that is design is generated from this copy by `python3 tools/from_design.py`, and
`python3 tools/from_design.py --check` fails when one is not what the design gives.

**Which design it is.** Today's design (`tools/design_build.py`) built from the conversion of 1.0.0
(`tools/convert_2_0.py`, S3): every group's baseline release 0.1, converted and not yet signed by a person.
From S10 it is replaced by a copy of the current released design, taken when an application release is
prepared (W15): a new application must give that design's answers unchanged.

**How it was made** (6 Oct 2026, from the 1.0.0 sources now in `archive/design-1.0/`; made again 7 Oct 2026 with the
developer's revision S7.1b, `design/revisions_2_0.toml`, which the conversion applies on top; again on 7 Oct 2026
for the toolbox it names, `trinetra-toolbox/2`, S7.2; and again on 7 Oct 2026 with the revisions S7.2b, published data
read into env's nodes by `tools/readers.py`, and S7.3, env's methods for time, frames and the field, transcribed under
`design/revisions/S7.3/`, for the toolbox `trinetra-toolbox/3`; and again on 7 Oct 2026 with S7.3b-S7.3e, the
atmosphere, DE440, gravity and tides and relativity, their data read by `tools/readers.py` and their methods under
`design/revisions/S7.3b/` to `S7.3e/`: 9.6 MB, of which JB2008's SET indices are 2.5 MB; and again on 8 Oct 2026 with
S7.4, env's environment and disturbance torques and six values dyn states, and S7.5, dyn's plant, their methods under
`design/revisions/S7.4/` and `S7.5/`; and again on 8 Oct 2026 with S7.6, env's orbit (the fast orbit's forces and start, the
precision orbit's spacecraft force models, force set and sum), and S7.7, act's actuators and eight values act states, their
methods under `design/revisions/S7.6/` and `S7.7/`, for the toolbox `trinetra-toolbox/4` (erf); and again on 8 Oct 2026
with S7.8, sens's simple sensors, the sky they see and the rotors' telemetry and six values gdn states, and S7.9, the star
tracker's unit, its onboard table and its attitude, their methods under `design/revisions/S7.8/` and `S7.9/`; and again on 8
Oct 2026 with S7.10, the star tracker's image chain, its methods under `design/revisions/S7.10/`, for the toolbox
`trinetra-toolbox/5` (buffers whose length is the caller's); and again on 8 Oct 2026 with S7.11, the truth plant, the
case's orbit and epoch, the plant's start and the devices' descriptors, and seven values env states, and S7.12, the device
emulators' scaling, their methods under `design/revisions/S7.11/` and `S7.12/`; since S7.11 the engine's inputs include
`data/stated.json`, every stated value of the design, which the engine reads by node; and again on 8 Oct 2026 with S7.13,
the flight software's parameters, 77 of the 147 fsw_param_* methods under `design/revisions/S7.13/` and the rest stated,
with the tuning a law takes and the run's defaults when a scenario states none, so `data/stated.json` holds lists too,
`adcs-stated/2`):

    python3 tools/convert_2_0.py --out DRIVE
    python3 tools/design_build.py DRIVE --out tests/regression/design.tndb

The two commands give the same bytes every time. A design change before the ownership stage is a revision in
`design/revisions_2_0.toml` (the developer's, unsigned, with its reason), and this copy is made again with them.

**What it proves.** `results/PARITY_2_0.md`: the engine reading this design alone gives 1.0.0's inputs,
parameter blobs, runs, campaigns and evaluation.

Never edit it by hand. Change the design, release it, and take a new copy.
