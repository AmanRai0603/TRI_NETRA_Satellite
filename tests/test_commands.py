"""The commands registry (docs/commands.toml) is true: every command a tool has is in it,
nothing in it is missing from the tool, and docs/COMMANDS.md is what it generates.
Copyright (c) 2026 Agastya. All rights reserved."""
import argparse
import re
import unittest

from _path import ROOT
import trinetra


def reg(tool):
    return {c["name"] for c in trinetra.commands() if c["tool"] == tool}


class Registry(unittest.TestCase):
    def test_engine_py_subcommands_are_the_registry(self):
        import engine
        seen = {}
        real = argparse.ArgumentParser.parse_args
        def capture(self_, *a, **k):
            for act in self_._actions:
                if isinstance(act, argparse._SubParsersAction):
                    seen.update(act.choices)
            raise SystemExit(0)
        argparse.ArgumentParser.parse_args = capture
        try:
            with self.assertRaises(SystemExit):
                engine.main()
        finally:
            argparse.ArgumentParser.parse_args = real
        self.assertEqual(set(seen), reg("engine.py"))

    def test_adcs_commands_are_the_registry(self):
        src = (ROOT / "engine/crates/adcs-cli/src/help.rs").read_text()
        block = src[src.index("pub const COMMANDS"):src.index("];", src.index("pub const COMMANDS"))]
        have = set(re.findall(r'^    \("([a-z]+)", "', block, re.M))
        self.assertEqual(len(have), 5, "the command table in help.rs moved: update this test")
        self.assertEqual(have, reg("adcs"))
        main = (ROOT / "engine/crates/adcs-cli/src/main.rs").read_text()
        for c in have:
            self.assertIn(f'"{c}"', main, f"adcs {c} has help but main.rs does not dispatch it")

    def test_every_python_tool_named_exists(self):
        for c in trinetra.commands():
            if c["tool"].endswith(".py"):
                self.assertTrue((ROOT / "tools" / c["tool"]).is_file(), c["tool"])

    def test_every_tool_with_a_main_is_registered(self):
        named = {c["tool"] for c in trinetra.commands()}
        for f in sorted((ROOT / "tools").glob("*.py")):
            if 'if __name__ == "__main__"' in f.read_text() and f.name not in ("common.py",):
                self.assertIn(f.name, named, f"{f.name} runs as a command but docs/commands.toml does not describe it")

    def test_every_command_says_what_it_does(self):
        for c in trinetra.commands():
            for k in ("tool", "name", "usage", "what", "steps", "reads", "writes", "runs"):
                self.assertIn(k, c, f"{c.get('tool')} {c.get('name')} has no {k}")
            self.assertTrue(c["steps"], f"{c['tool']} {c['name']} has no steps")

    def test_the_commands_document_is_current(self):
        self.assertEqual((ROOT / "docs/COMMANDS.md").read_text(), trinetra.document(),
                         "docs/COMMANDS.md is stale: python3 tools/trinetra.py docs")

    def test_why_names_the_command_that_writes_a_file(self):
        names = lambda f: {f"{c['tool']} {c['name']}" for c in trinetra.writers(f)}
        self.assertIn("trinetra.py explain", names("docs/COMMANDS.md"))
        self.assertIn("engine.py campaign", names("results/ENGINE_CAMPAIGNS.md"))
        self.assertIn("gen_fsw_params.py gen-fsw-params", names("fsw-rs/src/params.rs"))
        self.assertIn("export_catalogue.py export-catalogue", names("matlab_sils/data/scenarios/nadir_hold_ais.json"))
        self.assertEqual(names("engine/crates/adcs-sim/src/config.rs"), set())

    def test_every_generated_file_the_checks_know_names_its_writer(self):
        import export_catalogue, gen_fsw_params
        for p, _ in gen_fsw_params.outputs():
            self.assertTrue(trinetra.writers(p.relative_to(ROOT)), p)
        for _, p, _ in export_catalogue.outputs():
            self.assertTrue(trinetra.writers(p.relative_to(ROOT)), p)

    def test_an_ambiguous_name_is_refused_with_its_candidates(self):
        with self.assertRaises(SystemExit) as e:
            trinetra.find(["run"])
        self.assertIn("adcs run", str(e.exception))
        self.assertEqual(trinetra.find(["engine.py", "campaign"])["name"], "campaign")
        self.assertEqual(trinetra.find(["pipeline"])["tool"], "pipeline.py")


if __name__ == "__main__":
    unittest.main()
