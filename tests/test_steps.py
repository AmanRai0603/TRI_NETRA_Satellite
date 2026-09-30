"""Every step a tool announces is a step the registry lists, and every listed step is announced.

The tools say where they are with common.Steps: `S = Steps("engine.py", "campaign")`, then
`S(2)` as the command reaches its second step, which prints that step's text from
docs/commands.toml. This test reads the tools' source as Python syntax (ast), not as text: it
finds each Steps object and every call of it, and holds them to the registry, so the steps
`python3 tools/trinetra.py explain` promises are the steps the command says as it runs.

Copyright (c) 2026 Agastya. All rights reserved.
"""
import ast
import unittest

import _path  # puts tools/ on the import path
import trinetra
from _path import ROOT

_ = _path  # imported for its effect: tools/ on sys.path

# The commands whose steps are announced as they run. (The single-purpose tools print what
# they write; their registry steps are read with `explain`.)
ANNOUNCED = {("engine.py", n) for n in ("build", "run", "mc", "fsw-parity", "twin-parity", "vobc", "dispatch", "campaign",
                                        "campaign-ledger", "oils", "oils-ledger", "solutions")} | {("pipeline.py", "pipeline")}


def announcements():
    """{(tool, command): {step numbers called}} over every tool, and where each was made."""
    found, where = {}, {}
    for f in sorted((ROOT / "tools").glob("*.py")):
        tree = ast.parse(f.read_text(), str(f))
        for fn in ast.walk(tree):
            if not isinstance(fn, ast.FunctionDef):   # one scope at a time: S names another command elsewhere
                continue
            names = {}
            for node in ast.walk(fn):
                if (isinstance(node, ast.Assign) and isinstance(node.value, ast.Call) and getattr(node.value.func, "id", None) == "Steps"
                        and len(node.targets) == 1 and isinstance(node.targets[0], ast.Name)):
                    args = node.value.args
                    assert len(args) >= 2 and all(isinstance(a, ast.Constant) for a in args[:2]), f"{f.name}:{node.lineno}: Steps needs literal names"
                    key = (args[0].value, args[1].value)
                    names[node.targets[0].id] = key
                    found.setdefault(key, set())
                    where[key] = f"{f.name}:{node.lineno}"
            for node in ast.walk(fn):
                if isinstance(node, ast.Call) and isinstance(node.func, ast.Name) and node.func.id in names:
                    first = node.args[0] if node.args else None
                    assert isinstance(first, ast.Constant) and isinstance(first.value, int), \
                        f"{f.name}:{node.lineno}: a step is called by its literal number"
                    found[names[node.func.id]].add(first.value)
    return found, where


class Steps(unittest.TestCase):
    def test_every_announcement_is_a_registered_step_and_every_step_is_announced(self):
        reg = {(c["tool"], c["name"]): len(c["steps"]) for c in trinetra.commands()}
        found, where = announcements()
        for key, called in found.items():
            self.assertIn(key, reg, f"{where[key]}: {key} is not in docs/commands.toml")
            n = reg[key]
            self.assertTrue(called <= set(range(1, n + 1)), f"{where[key]}: {key} announces {sorted(called)}, the registry lists {n} steps")
            self.assertEqual(called, set(range(1, n + 1)), f"{where[key]}: {key} never announces step(s) {sorted(set(range(1, n + 1)) - called)}")

    def test_the_commands_that_run_for_long_announce_their_steps(self):
        found, _ = announcements()
        self.assertEqual(ANNOUNCED - set(found), set(), "these commands do not announce their steps")

    def test_a_step_that_is_not_there_is_refused(self):
        import common
        s = common.Steps("engine.py", "campaign", show=False)
        s(4)
        with self.assertRaises(ValueError):
            s(5)
        with self.assertRaises(SystemExit):
            common.Steps("engine.py", "no-such-command", show=False)


if __name__ == "__main__":
    unittest.main()
