# The regression copy

`design.tndb` is the design the repository builds and tests against (`docs/PLAN_2_0.md` S4,
`docs/OPERATING_2_0.md` W15). The repository holds no design data of its own: every file the code still
reads that is design is generated from this copy by `python3 tools/from_design.py`, and
`python3 tools/from_design.py --check` fails when one is not what the design gives.

**Which design it is.** Today's design (`tools/design_build.py`) built from the conversion of 1.0.0
(`tools/convert_2_0.py`, S3): every group's baseline release 0.1, converted and not yet signed by a person.
From S10 it is replaced by a copy of the current released design, taken when an application release is
prepared (W15): a new application must give that design's answers unchanged.

**How it was made** (6 Oct 2026, from the 1.0.0 sources now in `archive/design-1.0/`):

    python3 tools/convert_2_0.py --out DRIVE
    python3 tools/design_build.py DRIVE --out tests/regression/design.tndb

**What it proves.** `results/PARITY_2_0.md`: the engine reading this design alone gives 1.0.0's inputs,
parameter blobs, runs, campaigns and evaluation.

Never edit it by hand. Change the design, release it, and take a new copy.
