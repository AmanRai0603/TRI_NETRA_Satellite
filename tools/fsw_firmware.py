"""The virtual OBC's firmware is built from the flight sources and the parameter-blob layout: stale is refused by name.

    python3 tools/fsw_firmware.py [NAME ...]    rebuild what is out of date (default: obc_qemu.elf insn_count.so), exit 1 if it cannot

The firmware under fsw/build is not tracked, so it can be older than fsw/ (its sources, its headers, the
parameter table fsw/include/adcs_params.h written from the design). A firmware older than the blob the engine
writes refuses the configuration ("does not decode (size or CRC)"), which names neither cause. `stale` says
which file is older (make -q), `ensure` rebuilds it, `require_fresh` refuses by name; the parity gate calls
`ensure` before it flies the firmware, the tests that fly it call `require_fresh`.

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import pathlib
import subprocess
import sys

FSW = pathlib.Path(__file__).resolve().parents[1] / "fsw"
SOFT_OILS = ("obc_qemu.elf", "insn_count.so")


class FirmwareStale(Exception):
    pass


def _make(args, fsw):
    return subprocess.run(["make", "-C", str(fsw), *args], capture_output=True, text=True)


def stale(names=SOFT_OILS, fsw=FSW):
    """The firmware files (names under fsw/build) that are missing or older than what they are built from."""
    out = []
    for n in names:
        r = _make(["-q", f"build/{n}"], fsw)
        if r.returncode == 1 or not (pathlib.Path(fsw) / "build" / n).is_file():
            out.append(n)
        elif r.returncode:
            raise FirmwareStale(f"fsw/build/{n}: make cannot tell whether it is current: {(r.stderr or r.stdout).strip()[-300:]}")
    return out


def require_fresh(names=SOFT_OILS, fsw=FSW):
    """Refuse by name when a firmware file is older than its sources or the parameter-blob layout."""
    old = stale(names, fsw)
    if old:
        raise FirmwareStale("the firmware " + ", ".join(f"fsw/build/{n}" for n in old) +
                            " is older than its sources or the parameter-blob layout (fsw/include/adcs_params.h): "
                            "it would refuse the blob as `does not decode (size or CRC)`. Rebuild it: make -C fsw obc")


def ensure(names=SOFT_OILS, fsw=FSW):
    """Rebuild what is out of date; refuse by name when it cannot be built."""
    old = stale(names, fsw)
    if old:
        r = _make([f"build/{n}" for n in old], fsw)
        if r.returncode:
            raise FirmwareStale("the firmware " + ", ".join(f"fsw/build/{n}" for n in old) + " is out of date and make -C fsw cannot rebuild it: "
                                + (r.stderr or r.stdout).strip()[-400:])
    return old


def main(argv=None):
    names = tuple(argv if argv is not None else sys.argv[1:]) or SOFT_OILS
    try:
        done = ensure(names)
    except FirmwareStale as e:
        print(f"fsw_firmware: {e}", file=sys.stderr)
        return 1
    print("fsw_firmware: " + (f"rebuilt {', '.join(done)}" if done else "current: " + ", ".join(names)))
    return 0


if __name__ == "__main__":
    sys.exit(main())
