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


def _rel(path):
    try:
        return pathlib.Path(path).resolve().relative_to(ROOT).as_posix()
    except ValueError:
        return str(path)


def _tmp_beside(path):
    path = pathlib.Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    fd, tmp = tempfile.mkstemp(prefix="." + path.name + ".", suffix=".tmp", dir=path.parent)
    # mkstemp makes the file private (0600); give it the mode the target has, or the mode a
    # new file gets, so writing whole never changes who can read the file
    try:
        mode = path.stat().st_mode & 0o7777
    except FileNotFoundError:
        umask = os.umask(0)
        os.umask(umask)
        mode = 0o666 & ~umask
    os.chmod(tmp, mode)
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
    trace(f"wrote {_rel(path)} ({len(data)} bytes)")
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
        trace(f"wrote {_rel(path)}")
    except BaseException:
        with contextlib.suppress(OSError):
            tmp.unlink()
        raise


TRACE = ROOT / ".trace" / "trinetra.log"


def trace(what):
    """One line in .trace/trinetra.log: which tool, what it ran or wrote, when. Every
    command started through `sh` and every file written through `write_*` lands here, so
    `tail .trace/trinetra.log` says what the last run of any tool actually did.
    TRINETRA_TRACE=0 turns it off."""
    if os.environ.get("TRINETRA_TRACE", "1") == "0":
        return
    import datetime
    import sys
    tool = pathlib.Path(sys.argv[0]).name if sys.argv and sys.argv[0] else "python"
    try:
        TRACE.parent.mkdir(parents=True, exist_ok=True)
        with open(TRACE, "a", encoding="utf-8") as f:
            f.write(f"{datetime.datetime.now(datetime.timezone.utc):%Y-%m-%dT%H:%M:%SZ} {os.getpid()} {tool}: {what}\n")
    except OSError:
        pass


def source_date():
    """The date a generated document carries: $SOURCE_DATE_EPOCH (the reproducible-builds
    convention) when set, else the time of the last commit, so the same commit always gives
    the same document; the wall clock only outside a git checkout."""
    import datetime
    s = os.environ.get("SOURCE_DATE_EPOCH", "").strip()
    if s and not s.isdigit():
        raise SystemExit(f"SOURCE_DATE_EPOCH={s} is not a whole number of seconds since 1970")
    if not s:
        s = subprocess.run(["git", "log", "-1", "--format=%ct"], cwd=ROOT, capture_output=True, text=True).stdout.strip()
    try:
        return datetime.datetime.fromtimestamp(int(s), datetime.timezone.utc)
    except ValueError:
        return datetime.datetime.now(datetime.timezone.utc)


def sh(cmd, cwd=ROOT, check=True, **kw):
    """Run a command, showing it first, so every step a tool takes is on the screen."""
    print("$", " ".join(map(str, cmd)), flush=True)
    trace("ran " + " ".join(map(str, cmd)))
    return subprocess.run(cmd, cwd=cwd, check=check, **kw)
