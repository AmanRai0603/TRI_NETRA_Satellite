"""check_all.py chooses its checks, says NOT RUN for a check that cannot run here, refuses a
check it does not have, and exits 1 on any failure. The checks themselves are not run: the
command runner is replaced. Copyright (c) 2026 Agastya. All rights reserved."""
import contextlib
import io
import subprocess
import sys
import unittest
from unittest import mock

import _path  # puts tools/ on the import path
import check_all

_ = _path  # imported for its effect: tools/ on sys.path

ALL = [c[0] for c in check_all.CHECKS + check_all.OCTAVE + check_all.PAGES + check_all.MUTATION]


class Fake:
    """subprocess.run stand-in: records the commands, fails the ones named."""
    def __init__(self, fail=()):
        self.ran, self.fail = [], set(fail)

    def __call__(self, cmd, cwd=None, **kw):
        name = next(c[0] for c in check_all.CHECKS + check_all.OCTAVE + check_all.PAGES + check_all.MUTATION if c[2] == cmd and check_all.ROOT / c[3] == cwd)
        self.ran.append(name)
        return subprocess.CompletedProcess(cmd, 1 if name in self.fail else 0, stdout=f"{name} output\nlast line of {name}\n")


def run(argv, fail=(), have=lambda need: True):
    fake, out = Fake(fail), io.StringIO()
    with mock.patch.object(check_all.subprocess, "run", fake), mock.patch.object(check_all, "have", have), contextlib.redirect_stdout(out):
        rc = check_all.main(argv)
    return rc, fake.ran, out.getvalue()


class Choice(unittest.TestCase):
    def test_the_check_names_are_unique(self):
        self.assertEqual(len(ALL), len(set(ALL)))

    def test_the_default_runs_the_fast_checks_only(self):
        rc, ran, out = run([])
        self.assertEqual(rc, 0)
        self.assertEqual(ran, [c[0] for c in check_all.CHECKS])
        self.assertIn(f"check_all: {len(check_all.CHECKS)}/{len(check_all.CHECKS)} ok", out)

    def test_flags_add_the_octave_suites_the_pages_and_mutation_testing(self):
        rc, ran, _ = run(["--octave", "--pages"])
        self.assertEqual(ran, [n for n in ALL if n != "mutation"], "mutation testing only when asked")
        rc, ran, _ = run(["--octave", "--pages", "--mutation"])
        self.assertEqual(ran, ALL)

    def test_only_runs_just_those_and_reaches_an_optional_group_by_name(self):
        rc, ran, _ = run(["--only", "lint", "twin", "pages"])
        self.assertEqual(ran, ["lint", "twin", "pages"])

    def test_an_unknown_check_is_refused_naming_every_check(self):
        err = io.StringIO()
        with contextlib.redirect_stderr(err), self.assertRaises(SystemExit) as e:
            run(["--only", "lint", "no-such-check"])
        self.assertEqual(e.exception.code, 2)
        self.assertIn("no check no-such-check", err.getvalue())
        for name in ALL:
            self.assertIn(name, err.getvalue())


class Verdicts(unittest.TestCase):
    def test_a_failure_exits_1_and_is_named(self):
        rc, ran, out = run(["--only", "lint", "catalogue"], fail=["catalogue"])
        self.assertEqual(rc, 1)
        self.assertIn("check_all: 1/2 ok; FAILED: catalogue", out)
        self.assertIn("catalogue output", out, "a failing check's output is shown")

    def test_a_check_that_cannot_run_here_is_not_run_and_not_passed(self):
        rc, ran, out = run(["--only", "fsw-c", "lint"], have=lambda need: need != "gcc")
        self.assertEqual(ran, ["lint"])
        self.assertRegex(out, r"fsw-c\s+NOT RUN\s+0 s\s+needs gcc")
        self.assertEqual(rc, 0, "not run is not a failure")

    def test_strict_makes_a_check_that_cannot_run_a_failure(self):
        rc, ran, out = run(["--strict", "--only", "fsw-c", "lint"], have=lambda need: need != "gcc")
        self.assertEqual(ran, ["lint"])
        self.assertEqual(rc, 1)
        self.assertIn("FAILED: fsw-c", out)


class Have(unittest.TestCase):
    def test_a_program_is_looked_up_on_the_path(self):
        self.assertTrue(check_all.have("sh"))
        self.assertFalse(check_all.have("no-such-program-anywhere-xyz"))

    def test_a_python_module_is_looked_up_by_import_spec(self):
        self.assertTrue(check_all.have("py:json"))
        self.assertFalse(check_all.have("py:no_such_module_xyz"))

    def test_the_browser_and_time_series_needs_answer_a_boolean(self):
        self.assertIsInstance(check_all.have("browser"), bool)
        self.assertIsInstance(check_all.have("time series"), bool)

    def test_every_check_runs_this_python(self):
        for c in check_all.CHECKS:
            if c[2][0].endswith("python3") or "python" in c[2][0]:
                self.assertEqual(c[2][0], sys.executable)


if __name__ == "__main__":
    unittest.main()
