"""Which model does which work (.claude/model_policy.toml, docs/PLAN_2_0.md "Working economy"): every agent names the
tier its kinds of work are given, every check the policy names exists, the token ledger reads both ways a transcript
records an agent's tokens and flags work run above its tier, and the work checker picks a change's checks by its paths.

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import json
import pathlib
import re
import tempfile
import unittest

import _path  # puts tools/ on the import path
import check_all
import tokens
import verify_work
from common import ROOT

_ = _path  # imported for its effect: tools/ on sys.path

AGENTS = ROOT / ".claude" / "agents"


def front(path):
    head = path.read_text(encoding="utf-8").split("---")[1]
    return dict(re.findall(r"^(\w+):\s*(.+)$", head, re.M))


class Policy(unittest.TestCase):
    pol = tokens.policy()

    def test_tiers_and_kinds(self):
        tiers = self.pol["tiers"]
        self.assertEqual(list(tiers), tokens.ORDER)
        kinds = [w["kind"] for w in self.pol["work"]]
        self.assertEqual(len(kinds), len(set(kinds)))
        for w in self.pol["work"]:
            self.assertIn(w["tier"], tiers, w["kind"])
            if w["agent"]:
                self.assertTrue((AGENTS / f"{w['agent']}.md").exists(), w["agent"])
        self.assertIn(self.pol["escalation"]["main_session"], tiers)

    def test_every_agent_on_its_tier(self):
        """An agent's `model:` is the alias of the one tier all its kinds of work have."""
        alias = {k: t["alias"] for k, t in self.pol["tiers"].items()}
        for f in sorted(AGENTS.glob("*.md")):
            name = front(f)["name"]
            want = {w["tier"] for w in self.pol["work"] if w["agent"] == name}
            self.assertEqual(len(want), 1, f"{name}: its kinds of work are on tiers {want}")
            self.assertEqual(front(f).get("model"), alias[want.pop()], name)

    def test_every_check_exists(self):
        known = {c[0] for c in check_all.CHECKS + check_all.OCTAVE + check_all.PAGES + check_all.MUTATION} | set(self.pol["checks"])
        named = {c for w in self.pol["work"] for c in w["checks"]} | {c for p in self.pol["paths"] for c in p["checks"]}
        self.assertEqual(named - known, set())


class Ledger(unittest.TestCase):
    pol = tokens.policy()

    def test_kind_of(self):
        self.assertEqual(tokens.kind_of("code: S11 F2 GNC logic", self.pol), ("code", "standard"))
        self.assertEqual(tokens.kind_of("Babysit the twin re-fly", self.pol), ("watch", "light"))
        self.assertEqual(tokens.kind_of("S7 inventory of code relations", self.pol), ("decide", "deep"))
        self.assertEqual(tokens.stage_of("S7.15b Floquet"), "S7")
        self.assertEqual(tokens.stage_of("S11 corrections"), "S11")

    def test_reads_a_transcript(self):
        def usage(cr, cw, out):
            return {"input_tokens": 3, "cache_read_input_tokens": cr, "cache_creation_input_tokens": cw,
                    "cache_creation": {"ephemeral_1h_input_tokens": cw, "ephemeral_5m_input_tokens": 0}, "output_tokens": out}
        lines = [
            {"type": "assistant", "timestamp": "2026-11-06T01:00:00Z", "message": {"model": "x-opus-y", "usage": usage(1000, 300000, 50)}},
            {"type": "assistant", "timestamp": "2026-11-06T01:01:00Z", "message": {"model": "x-opus-y", "usage": usage(301000, 10, 50)}},
            {"type": "user", "toolUseResult": {"agentId": "a1", "description": "watch: the twin re-fly", "resolvedModel": "x-opus-y"},
             "timestamp": "2026-11-06T01:02:00Z", "message": {"role": "user", "content": []}},
            {"type": "user", "toolUseResult": {"agentId": "a2", "description": "code: S11 F2", "resolvedModel": "x-sonnet-y"},
             "timestamp": "2026-11-06T01:02:00Z", "message": {"role": "user", "content": []}},
            {"type": "user", "message": {"role": "user", "content": "<task-notification>\n<task-id>a1</task-id>\n<result>r</result>\n"
             "<usage><subagent_tokens>1000</subagent_tokens><tool_uses>9</tool_uses><duration_ms>60000</duration_ms></usage>\n</task-notification>"}},
            {"type": "attachment", "attachment": {"type": "queued_command", "prompt": "<task-notification>\n<task-id>a2</task-id>",
             "usage": {"totalTokens": 5000, "toolUses": 4, "durationMs": 120000}}},
        ]
        with tempfile.TemporaryDirectory() as d:
            (pathlib.Path(d) / "s.jsonl").write_text("\n".join(json.dumps(x) for x in lines) + "\n")
            main, agents = tokens.read(tokens.transcripts(d), self.pol)
        self.assertEqual([t for _, t, _ in main], ["deep", "deep"])
        by = {a["agent"]: a for a in agents}
        self.assertEqual((by["a1"]["tokens"], by["a1"]["kind"], by["a1"]["tier"], by["a1"]["minutes"]), (1000, "watch", "deep", 1.0))
        self.assertEqual((by["a2"]["tokens"], by["a2"]["tier"], by["a2"]["policy_tier"]), (5000, "standard", "standard"))
        rows, saved = tokens.advice(agents, self.pol)
        self.assertEqual([a["agent"] for a, _ in rows], ["a1"])  # the watcher ran deep, its kind is light
        self.assertEqual(saved, 1000 * (self.pol["tiers"]["deep"]["weight"] - self.pol["tiers"]["light"]["weight"]))


class Checker(unittest.TestCase):
    pol = tokens.policy()

    def test_paths_pick_checks(self):
        got = dict(verify_work.plan(["design/js/pcode_gen.js", "docs/PLAN_2_0.md", "matlab_sils/store/results/x/m.json"], self.pol))
        self.assertEqual(set(got), {"pseudocode", "translators", "engine-build", "flight-build"})
        got = dict(verify_work.plan(["design/revisions_2_0.toml"], self.pol))
        self.assertIn("parity", got)
        self.assertEqual(dict(verify_work.plan(["somewhere/new.txt"], self.pol)), {"python-tests": ["somewhere/new.txt"]})
        self.assertIn("parity", dict(verify_work.plan([], self.pol, kind="migrate")))


if __name__ == "__main__":
    unittest.main()
