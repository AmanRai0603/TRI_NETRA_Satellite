"""Call one of the design's methods the engine carries (docs/S7_INVENTORY.md S7.15): a tool that needs a relation asks
the engine for the design's (`adcs design call module::function`), and keeps none of its own.

    from design_call import call, Buf
    n, *rest = call("looprules::loop_converge", Buf(mode_ix), ..., 0.05)

The inputs are flattened as the translator's dispatcher reads them: a number as itself, true 1 and false 0, a choice its
option's number, a fixed array its elements in order, a buffer (Buf) its length and then its elements. The outputs come
back flattened the same way: the method's outputs, then each input it may change (a buffer: its length, then its
elements). Every number crosses exactly: written as Python's repr (the shortest decimal that reads back the same double;
nan, inf), read back from the engine's shortest round trip.

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import subprocess

from common import ROOT

ENGINE = ROOT / "engine" / "target" / "release" / "adcs"


class Buf(list):
    """A buffer input: its length the caller's, written as its length, then its elements."""


def flatten(x, out):
    if isinstance(x, Buf):
        out.append(len(x))
        for v in x:
            flatten(v, out)
    elif isinstance(x, (list, tuple)):
        for v in x:
            flatten(v, out)
    elif isinstance(x, bool):
        out.append(1 if x else 0)
    else:
        out.append(x)
    return out


def call(name, *args):
    """The method's outputs, flattened (a list of floats)."""
    text = " ".join(repr(float(v)) for v in flatten(list(args), []))
    r = subprocess.run([str(ENGINE), "design", "call", name], input=text, capture_output=True, text=True)
    if r.returncode:
        raise SystemExit(f"design_call: {name}: {r.stderr.strip()[-2000:] or 'the engine failed'} ({ENGINE})")
    return [float(w) for w in r.stdout.split()]


def take(flat, *shapes):
    """The flat outputs cut into pieces: an int n takes n numbers (1 a number), 'buf' a buffer (its length, then it)."""
    out, at = [], 0
    for s in shapes:
        if s == "buf":
            n = int(flat[at])
            out.append(flat[at + 1:at + 1 + n])
            at += 1 + n
        elif s == 1:
            out.append(flat[at])
            at += 1
        else:
            out.append(flat[at:at + s])
            at += s
    if at != len(flat):
        raise SystemExit(f"design_call: {len(flat)} numbers back, {at} expected")
    return out
