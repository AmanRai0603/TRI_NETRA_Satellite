"""The flight software's deepest stack fits the stack the OBC firmware reserves.

The Cortex-M4 firmware (fsw/targets/qemu-mps2, the C flight software behind adcs-link/1) is
compiled with GCC's -fstack-usage -fcallgraph-info=su: every function's frame and every call it
makes, the algorithms written from the design (fsw/alg) among them. This walks the call graph from the firmware's entry (Reset_Handler: the link loop, init,
one step) and adds the frames along the deepest path. Library routines (libm, the soft-double
helpers) have no frame in the graph; each is charged LIB_FRAME bytes and named. The total must
fit the stack region fsw/targets/qemu-mps2/link.ld reserves (_stack_size), which the linker in
turn keeps clear of .data and .bss. A call cycle (recursion) is refused: the stack of a
recursive function has no bound.

    python3 tools/fsw_stack.py           the deepest path, its bytes, and the verdict

Copyright (c) 2026 Agastya. All rights reserved.
"""
import pathlib
import re
import subprocess
import sys
import tempfile

from common import ROOT

FSW = ROOT / "fsw"
ROOTS = ["Reset_Handler"]
LIB_FRAME = 256          # bytes charged to a library routine the graph has no frame for
ARMFLAGS = ["-mcpu=cortex-m4", "-mthumb", "-mfloat-abi=hard", "-mfpu=fpv4-sp-d16", "-std=c99", "-O2",
            "-ffp-contract=off", "-fno-fast-math", "-ffunction-sections", "-fdata-sections",
            f"-I{FSW / 'include'}", f"-I{FSW / 'alg/include'}", f"-I{FSW / 'targets/link'}", "-fstack-usage", "-fcallgraph-info=su"]

NODE = re.compile(r'node: \{ title: "([^"]+)" label: "([^"]*)"')
EDGE = re.compile(r'edge: \{ sourcename: "([^"]+)" targetname: "([^"]+)"')


def reserved():
    """The stack the linker script reserves, in bytes."""
    m = re.search(r"_stack_size\s*=\s*(0x[0-9A-Fa-f]+|\d+)\s*;", (FSW / "targets/qemu-mps2/link.ld").read_text())
    if not m:
        sys.exit("fsw/targets/qemu-mps2/link.ld reserves no stack (_stack_size)")
    return int(m.group(1), 0)


def graph(out):
    src = sorted(str(p.relative_to(FSW)) for p in (FSW / "src").glob("*.c"))
    src += sorted(str(p.relative_to(FSW)) for p in (FSW / "alg/src").glob("*.c"))     # the algorithms written from the design
    src += ["targets/link/adcs_link.c", "targets/qemu-mps2/main.c"]
    r = subprocess.run(["arm-none-eabi-gcc", *ARMFLAGS, "-c", *[str(FSW / s) for s in src]], cwd=out,
                       capture_output=True, text=True)
    if r.returncode:
        sys.exit("arm-none-eabi-gcc failed:\n" + r.stderr)
    frames, calls, local = {}, {}, {}
    for ci in pathlib.Path(out).glob("*.ci"):
        text = ci.read_text()
        for title, label in NODE.findall(text):
            m = re.search(r"(\d+) bytes \((static|dynamic[^)]*)\)", label)
            if m:
                if not m.group(2).startswith("static"):
                    sys.exit(f"{title}: a {m.group(2)} stack frame has no static bound")
                frames[title] = int(m.group(1))
                # a static function is titled file:name; calls from its own file name it so
                if ":" in title:
                    local[title.split(":")[-1].split(".")[0]] = title
        for a, b in EDGE.findall(text):
            calls.setdefault(a, []).append(b)
    return frames, calls


def deepest(frames, calls):
    memo, onpath = {}, set()

    def depth(f):
        if f in memo:
            return memo[f]
        if f in onpath:
            sys.exit(f"recursion through {f}: its stack has no bound")
        onpath.add(f)
        own = frames.get(f, LIB_FRAME)
        best = (0, [])
        for g in dict.fromkeys(calls.get(f, [])):
            d = depth(g)
            if d[0] > best[0]:
                best = d
        onpath.discard(f)
        memo[f] = (own + best[0], [(f, own, f not in frames)] + best[1])
        return memo[f]

    return max((depth(r) for r in ROOTS), key=lambda d: d[0])


def main():
    budget = reserved()
    with tempfile.TemporaryDirectory() as out:
        frames, calls = graph(out)
    total, path = deepest(frames, calls)
    for f, b, lib in path:
        print(f"  {b:6d}  {f.split(':')[-1]}{'  (library, charged)' if lib else ''}")
    print(f"deepest stack {total} bytes of the {budget} reserved ({100*total/budget:.0f} %)")
    if total > budget:
        sys.exit(f"the flight software's deepest stack ({total} bytes) is over the {budget} bytes link.ld reserves")


if __name__ == "__main__":
    main()
