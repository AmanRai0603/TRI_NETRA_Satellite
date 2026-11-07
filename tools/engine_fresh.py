"""The engine binary is built from its sources: stale is refused by name, as the firmware's is (tools/fsw_firmware.py).

    python3 tools/engine_fresh.py     say whether engine/target/release/adcs is current; exit 1 if it is older than its sources

`engine/target/release/adcs` is not tracked, so it can be older than the sources it is built from: the crates it depends
on (adcs-cli's path dependencies, transitively), the C and Rust flight software they carry (build.rs), the files the
crates include (`include_str!`, `include_bytes!`) and Cargo.lock. The design reaches it through the generated sources
under engine/crates/*/src/gen, so a design change is one of them. A stale binary does not fail by itself: a test or a
tool flies it and reports a result or an error that belongs to the old code (a refused value, a missing command, a
blob that does not decode). `stale` names the newest source it is older than; `require_fresh` refuses by name, giving the
rebuild command; the tools that call the binary call it before the first call.

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import functools
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
BIN = ROOT / "engine" / "target" / "release" / "adcs"
CRATE = ROOT / "engine" / "crates" / "adcs-cli"
REBUILD = "cd engine && cargo build --release -p adcs-cli -p trinetra-design"
SUFFIXES = {".rs", ".toml", ".c", ".h"}
# what build.rs fingerprints beyond the crates (the flight software's sources)
EXTRA = ("fsw/src", "fsw/include", "fsw/alg", "fsw/params", "fsw-rs/src")


class EngineStale(Exception):
    pass


def _crates(start=CRATE):
    """The crate directories the binary is built from: `start` and its path dependencies, transitively."""
    seen, todo = [], [pathlib.Path(start).resolve()]
    while todo:
        d = todo.pop()
        if d in seen:
            continue
        seen.append(d)
        manifest = d / "Cargo.toml"
        if not manifest.is_file():
            continue
        for m in re.finditer(r'path\s*=\s*"([^"]+)"', manifest.read_text()):
            todo.append((d / m.group(1)).resolve())
    return seen


def sources(start=CRATE, root=ROOT):
    """Every file the binary is built from."""
    files = set()
    for d in _crates(start):
        for p in d.rglob("*"):
            if p.is_file() and p.suffix in SUFFIXES and "target" not in p.relative_to(d).parts:
                files.add(p)
                if p.suffix == ".rs":
                    for m in re.finditer(r'include_(?:str|bytes)!\(\s*"([^"]+)"', p.read_text(errors="replace")):
                        q = (p.parent / m.group(1)).resolve()
                        if q.is_file():
                            files.add(q)
    for d in EXTRA:
        for p in (pathlib.Path(root) / d).rglob("*"):
            if p.is_file() and p.suffix in SUFFIXES:
                files.add(p)
    for name in ("Cargo.lock", "Cargo.toml"):
        if (pathlib.Path(root) / "engine" / name).is_file():
            files.add(pathlib.Path(root) / "engine" / name)
    return files


def stale(binary=BIN, start=CRATE, root=ROOT):
    """(the binary's age problem, the sources newer than it): ("missing", []), ("older", [paths, newest first]), or (None, [])."""
    binary = pathlib.Path(binary)
    if not binary.is_file():
        return "missing", []
    t = binary.stat().st_mtime
    newer = sorted((p for p in sources(start, root) if p.stat().st_mtime > t), key=lambda p: -p.stat().st_mtime)
    return ("older", newer) if newer else (None, [])


def require_fresh(binary=BIN, start=CRATE, root=ROOT):
    """Refuse by name when the engine binary is missing or older than its sources."""
    why, newer = stale(binary, start, root)
    if why == "missing":
        raise EngineStale(f"the engine {binary} is not built. Build it: {REBUILD}")
    if why:
        rel = lambda p: str(p.relative_to(root)) if p.is_relative_to(root) else str(p)
        raise EngineStale(f"the engine {rel(pathlib.Path(binary))} is older than its sources ({rel(newer[0])}"
                          + (f" and {len(newer) - 1} more" if len(newer) > 1 else "") + "): what it flies and answers is the old code's, "
                          f"and a test or a result built on it fails for a reason that is not the change. Rebuild it: {REBUILD}")


@functools.lru_cache(maxsize=None)
def check_once(binary=BIN):
    """require_fresh for the repository's binary, once a process (a tool that calls the binary many times)."""
    if pathlib.Path(binary) == BIN:
        require_fresh(binary)
    return True


def exit_if_stale(binary=BIN):
    """The tools' guard: stop with the message, not a traceback."""
    try:
        check_once(binary)
    except EngineStale as e:
        sys.exit(f"engine_fresh: {e}")


def main(argv=None):
    try:
        require_fresh()
    except EngineStale as e:
        print(f"engine_fresh: {e}", file=sys.stderr)
        return 1
    print("engine_fresh: current: " + str(BIN.relative_to(ROOT)))
    return 0


if __name__ == "__main__":
    sys.exit(main())
