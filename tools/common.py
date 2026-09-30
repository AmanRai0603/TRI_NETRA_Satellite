"""What every tool shares: where the repository is, how a command is run and shown,
and how a file is written so it is whole or absent.

A tool writes through `write_text`, `write_bytes`, `write_json` or `atomic_path`,
never `Path.write_text`: each puts the content in a temporary file beside the target
and renames it into place, so an interrupted run (Ctrl-C, a full disk, a crash) leaves
the previous file or the new one, never a half-written ledger or report that a later
step would read as complete.

Copyright (c) 2026 Agastya. All rights reserved.
"""
import contextlib
import json
import os
import pathlib
import subprocess
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[1]


def _tmp_beside(path):
    path = pathlib.Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    fd, tmp = tempfile.mkstemp(prefix="." + path.name + ".", suffix=".tmp", dir=path.parent)
    return fd, pathlib.Path(tmp)


def write_bytes(path, data):
    """Write `data` to `path` whole: temporary file, flushed to disk, renamed over it."""
    fd, tmp = _tmp_beside(path)
    try:
        with os.fdopen(fd, "wb") as f:
            f.write(data)
            f.flush()
            os.fsync(f.fileno())
        os.replace(tmp, path)
    except BaseException:
        with contextlib.suppress(OSError):
            tmp.unlink()
        raise
    return len(data)


def write_text(path, text, encoding="utf-8", errors=None, newline=None):
    """`Path.write_text`, whole or not at all. Newlines are written as given."""
    return write_bytes(path, text.encode(encoding, errors or "strict")) and len(text)


def write_json(path, obj, **kw):
    """JSON, whole or not at all (a trailing newline, like the ledgers already have)."""
    return write_text(path, json.dumps(obj, **kw) + "\n")


@contextlib.contextmanager
def atomic_path(path):
    """For a writer that wants a file name (zipfile, matplotlib): yields a temporary
    path beside `path` and renames it into place only when the block succeeds."""
    fd, tmp = _tmp_beside(path)
    os.close(fd)
    try:
        yield tmp
        os.replace(tmp, path)
    except BaseException:
        with contextlib.suppress(OSError):
            tmp.unlink()
        raise


def sh(cmd, cwd=ROOT, check=True, **kw):
    """Run a command, showing it first, so every step a tool takes is on the screen."""
    print("$", " ".join(map(str, cmd)), flush=True)
    return subprocess.run(cmd, cwd=cwd, check=check, **kw)
