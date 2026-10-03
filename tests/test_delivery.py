"""Test, deliver, accept, ship, every group in five waves (docs/RELEASE_PLAN.md P12, tools/delivery.py,
docs/DELIVERY.md), rehearsed on a design folder seeded and carried over as the team gets it: wave by
wave, stand-in leads seal their groups in the group app's code (tests/js/waves.test.mjs), the
developer side delivers (the release verified, the generated code checked to be the release's, the
group's tests run, its test app and note written), and the leads accept. A wave is refused before
the one it reads; a group left unaccepted ships visibly UNCONFIRMED, with why; an acceptance that
names another delivery does not count.

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import json
import pathlib
import shutil
import sqlite3
import subprocess
import tempfile
import unittest

import _path  # puts tools/ on the import path
import carry_over
import delivery
import seed_design

_ = _path  # imported for its effect: tools/ on sys.path
ROOT = pathlib.Path(__file__).resolve().parents[1]
NODE, CARGO = shutil.which("node"), shutil.which("cargo")


def wasm_target():
    r = subprocess.run(["rustup", "target", "list", "--installed"], capture_output=True, text=True) if shutil.which("rustup") else None
    return bool(r and "wasm32-unknown-unknown" in r.stdout)


@unittest.skipUnless(NODE and CARGO and wasm_target(), "Node.js, cargo and the wasm32-unknown-unknown target are needed")
class Waves(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.TemporaryDirectory()
        cls.d = pathlib.Path(cls.tmp.name) / "Design"
        seed_design.seed(cls.d, sync=False)
        carry_over.carry(cls.d)
        cls.waves = delivery.waves()

    @classmethod
    def tearDownClass(cls):
        cls.tmp.cleanup()

    def js(self, step, *groups):
        r = subprocess.run([NODE, str(ROOT / "tests" / "js" / "waves.test.mjs"), str(self.d), step, *groups], capture_output=True, text=True, timeout=900)
        self.assertEqual(r.returncode, 0, r.stdout[-4000:] + r.stderr[-2000:])

    def test_five_waves_sealed_delivered_accepted_and_shipped(self):
        self.assertEqual([k for k, _, _ in self.waves], ["A", "B", "C", "D", "E"])
        self.assertEqual(sum(len(g) for _, _, g in self.waves), 20)
        last = self.waves[-1][2][-1]          # left unaccepted: it ships UNCONFIRMED
        for i, (k, _, gs) in enumerate(self.waves):
            self.js("seal", *gs)
            if i == 1:
                # wave B is sealed; a C group cannot go ahead of it
                made, problems = delivery.deliver(self.d, [self.waves[2][2][0]], test=False)
                self.assertTrue(any("not released" in p or "no release" in p for p in problems), problems)
                made, problems = delivery.deliver(self.d, wave="C", test=False)
                self.assertTrue(problems, "wave C has no release yet")
            made, problems = delivery.deliver(self.d, wave=k, test=(k == "A"))
            self.assertEqual(problems, [], f"wave {k}")
            self.assertEqual(sorted(g for g, _, _ in made), sorted(gs))
            for g, v, p in made:
                rec = json.loads(p.read_text())
                self.assertEqual((rec["group"], rec["version"], rec["wave"]), (g, "1.0", k))
                self.assertTrue((self.d / "deliveries" / rec["test_app"]).is_file())
                self.assertTrue(p.with_suffix("").with_suffix(".md").is_file())
                if k == "A":
                    self.assertTrue(rec["tests"]["passed"], rec["tests"])
            self.js("accept", *[g for g in gs if g != last])
        self.js("refusals", self.waves[0][2][0])
        rows = {r["group"]: r for r in delivery.status(self.d)}
        self.assertEqual(sum(r["state"] == "accepted" for r in rows.values()), 19)
        self.assertEqual(rows[last]["state"], "delivered")
        self.assertIn("not accepted yet", rows[last]["why"])
        rec, md, bad = delivery.ship(self.d, self.d / "shipping.json", require_accepted=True)
        self.assertTrue(bad)
        un = [g for g in rec["groups"] if g["ships_as"] == "UNCONFIRMED"]
        self.assertEqual([g["group"] for g in un], [last])
        self.assertIn("UNCONFIRMED", md)
        self.assertTrue((self.d / "shipping.md").is_file())
        # an acceptance naming another delivery does not count: the delivery is written again
        g = self.waves[0][2][0]
        p = delivery.delivery_path(self.d, g, "1.0")
        rec = json.loads(p.read_text())
        rec["delivered_at"] = "2000-01-01T00:00:00Z"
        p.write_text(json.dumps(rec, indent=1, sort_keys=True) + "\n")
        self.assertEqual({r["group"]: r for r in delivery.status(self.d)}[g]["state"], "delivered")
        # the merged design holds every group's release, and the engine flies from it
        with sqlite3.connect(self.d / "design.tndb") as c:
            self.assertEqual(c.execute("SELECT count(*) FROM design_group").fetchone()[0], 20)
            self.assertGreater(c.execute("SELECT count(*) FROM engine_input").fetchone()[0], 100)


if __name__ == "__main__":
    unittest.main()
