"""Group releases (docs/RELEASE_PLAN.md P6): the group app's release side (design/js/release.js) on
the whole seeded design under Node (all 20 groups sealed 1.0, a node of each re-issued and changed and each sealed 1.1,
a damaged node file made again from a release, a node form imported), then through the page in
Chromium; and every release it wrote held by tools/release.py, a separate checker, which must also
find each release broken on purpose.

Needs Node.js; the browser test also Playwright with its Chromium (skipped, and says so, without).
Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import json
import os
import pathlib
import shutil
import sqlite3
import subprocess
import sys
import tempfile
import unittest

import _path  # puts tools/ on the import path
import group
import pages
import release
import seed_design
import tndb

_ = _path  # imported for its effect: tools/ on sys.path
ROOT = pathlib.Path(__file__).resolve().parents[1]
NODE = shutil.which("node")
PLAYWRIGHT = next((pathlib.Path(p) for p in (os.environ.get("PLAYWRIGHT_MODULE"), "/opt/node22/lib/node_modules/playwright/index.mjs")
                   if p and pathlib.Path(p).is_file()), None)
FORM_OPEN = '<script type="application/json" id="adcs-node-form">'


def filled_form(out_dir):
    """m2_4's seed node form (spec/tools/forms.py), filled in as a team member would: a requester,
    an explanation, a test vector from a book, a belief record."""
    out_dir = pathlib.Path(out_dir)
    subprocess.run([sys.executable, "tools/forms.py", "seed", "m2_4", "--out", str(out_dir / "forms")], cwd=ROOT / "spec", check=True, capture_output=True)
    src = next((out_dir / "forms").glob("*.html"))
    t = src.read_text(encoding="utf-8")
    i = t.index(FORM_OPEN) + len(FORM_OPEN)
    j = t.index("</script>", i)
    f = json.loads(t[i:j])
    r = f["request"]
    r["requested_by"], r["team"] = "Asha", "environment"
    p = r["proposed"]
    p["explain"]["simply"] = "Add the Earth's radius to the height."
    p["relation"]["why"] = "From the form."
    p["fixtures"] = [{"label": "book", "inputs": {"h": 500000}, "expect": 6878137, "tolerance": 1e-12,
                      "provenance": "published-source", "source": "vallado2013", "where": "p. 98, eq. 2-58"}]
    r["derisk"].update({"area": "model", "believed": "The radius is the altitude plus R_E.", "status": "held"})
    body = json.dumps(f, ensure_ascii=False, separators=(",", ":")).replace("<", "\\u003c")
    path = out_dir / "m2_4.request.html"
    path.write_text(t[:i] + body + t[j:], encoding="utf-8")
    return path


@unittest.skipUnless(NODE, "Node.js is needed")
class Releases(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.TemporaryDirectory()
        cls.d = pathlib.Path(cls.tmp.name)
        seed_design.seed(cls.d / "design", sync=False)
        form = filled_form(cls.d)
        cls.run_ = subprocess.run([NODE, str(ROOT / "tests" / "js" / "release.test.mjs"), str(cls.d / "design"), str(form)],
                                  capture_output=True, text=True, timeout=1200)

    @classmethod
    def tearDownClass(cls):
        cls.tmp.cleanup()

    def test_the_release_actions_on_the_whole_design(self):
        sys.stdout.write(self.run_.stdout[-3000:])
        self.assertEqual(self.run_.returncode, 0, self.run_.stdout[-4000:] + self.run_.stderr[-2000:])
        d = self.d / "design"
        self.assertEqual([e for f in sorted(d.rglob("*.tndb")) for e in tndb.check(f)], [])
        self.assertEqual(group.check(d), [])

    def test_every_release_checks_from_python(self):
        fs = release.files([self.d / "design"])
        self.assertEqual(len(fs), 40, [f.name for f in fs])     # every group at 1.0 and 1.1
        self.assertEqual([p for f in fs for p in release.check(f)], [])
        rows = {(g, v): (n, c) for g, v, _at, _by, n, c in release.listing(self.d / "design")}
        self.assertEqual(rows[("act", "1.0")], (138, 1))         # gm_0, checked and its stage signed
        self.assertEqual(rows[("act", "1.1")], (138, 1))

    def test_every_group_verifies_and_merges_into_the_design(self):
        d = self.d / "design"
        self.assertEqual({g: p for g, p in group.verify(d).items() if p}, {})
        summary, problems = group.merge(d, require_all=True)
        self.assertEqual(problems, [])
        self.assertEqual((summary["groups"], summary["nodes"], summary["not_released"]), (20, 734, []))
        self.assertEqual(tndb.check(d / "design.tndb"), [])
        self.assertTrue((d / "design.tndb.prev").exists(), "the seeded design database is kept beside it")
        with sqlite3.connect(d / "design.tndb") as c:
            self.assertEqual(c.execute("SELECT count(DISTINCT version) FROM design_group").fetchone()[0], 1)
            self.assertEqual(c.execute("SELECT version FROM design_group LIMIT 1").fetchone()[0], "1.1")
            self.assertEqual(c.execute("SELECT release FROM design_node WHERE id = 'gm_0'").fetchone()[0], "act 1.1")
            n_cat = c.execute("SELECT count(*) FROM catalogue_output").fetchone()[0]
            n_out = sum(len(json.loads(x)["body"]["output"]) for (x,) in c.execute("SELECT content FROM design_node"))
            self.assertEqual(n_cat, n_out, "the catalogue lists every output of every node")
            self.assertGreater(n_cat, 0)
        owner, readers, _ = group.impact(d, "m2_4")
        self.assertEqual(owner, "env")
        self.assertTrue(any(g != "env" for _, g, _ in readers), "m2_4 is read across groups")

    def test_a_release_the_group_has_moved_on_from_does_not_verify(self):
        d = self.d / "stale"
        shutil.copytree(self.d / "design", d)
        with sqlite3.connect(d / "structure" / "catalogue.group.tndb") as c:
            c.execute("UPDATE group_node SET state = 'archived' WHERE id = (SELECT id FROM group_node LIMIT 1)")
        ps = group.verify(d)["catalogue"]
        self.assertTrue(any("is no longer a node of catalogue" in p for p in ps), ps)
        self.assertEqual(group.merge(d)[0], None)

    def broken(self, name, edit):
        """A copy of a release file, changed by edit(conn), and what tools/release.py finds."""
        src = self.d / "design" / "releases" / name
        dst = self.d / "broken" / name
        dst.parent.mkdir(exist_ok=True)
        shutil.copy(src, dst)
        with sqlite3.connect(dst) as c:
            edit(c)
        return release.check(dst)

    def test_a_changed_node_is_found(self):
        def edit(c):
            (content,) = c.execute("SELECT content FROM release_node WHERE id = 'gm_1'").fetchone()
            c.execute("UPDATE release_node SET content = ? WHERE id = 'gm_1'", (content.replace("shell", "SHELL"),))
        self.assertTrue(any("gm_1: its fingerprint" in p for p in self.broken("act-1.0.tnrel", edit)))

    def test_a_dropped_node_is_found(self):
        p = self.broken("act-1.0.tnrel", lambda c: c.execute("DELETE FROM release_node WHERE id = 'gm_1'"))
        self.assertTrue(any("release fingerprint" in x for x in p) and any("missing ['gm_1']" in x for x in p), p)

    def test_a_computing_node_confirmed_without_an_outside_answer_is_found(self):
        # m2_4 as env 1.1 sealed it, rewritten as confirmed with a checker and fingerprints that
        # match: only the rule itself is left to catch it
        def edit(c):
            (content,) = c.execute("SELECT content FROM release_node WHERE id = 'm2_4'").fetchone()
            x = json.loads(content)
            body = json.loads(x["body"])
            body["fixture"] = [f[:5] + [0] for f in body["fixture"]]
            body["signature"] = [["checked by", "Ravi", "2026-10-03T10:00:00Z", "{}"]]
            x["body"] = json.dumps(body)
            x["body_fingerprint"] = release._sha(x["body"])
            x["sealed_as"], x["why"] = "confirmed", []
            new = json.dumps(x)
            c.execute("UPDATE release_node SET content = ?, fingerprint = ? WHERE id = 'm2_4'", (new, release._sha(new)))
            lines = sorted(f"{i} {fp}" for i, fp in c.execute("SELECT id, fingerprint FROM release_node"))
            c.execute("UPDATE release SET fingerprint = ?", (release._sha("\n".join(lines)),))
            c.execute("UPDATE signature SET statement = json_set(statement, '$.fingerprint', ?) WHERE role = 'sealed'", (release._sha("\n".join(lines)),))
        p = self.broken("env-1.1.tnrel", edit)
        self.assertEqual(p, [f"{self.d / 'broken' / 'env-1.1.tnrel'}: m2_4: a computing node sealed as confirmed with no test vector from outside the code"])

    def test_a_node_its_own_author_checked_is_found(self):
        def edit(c):
            (content,) = c.execute("SELECT content FROM release_node WHERE id = 'gm_0'").fetchone()
            x = json.loads(content)
            body = json.loads(x["body"])
            body["signature"] = [s if s[0] != "checked by" else [s[0], body["node"]["author"], s[2], s[3]] for s in body["signature"]]
            x["body"] = json.dumps(body)
            x["body_fingerprint"] = release._sha(x["body"])
            new = json.dumps(x)
            c.execute("UPDATE release_node SET content = ?, fingerprint = ? WHERE id = 'gm_0'", (new, release._sha(new)))
        self.assertTrue(any("nobody other than its author (Asha) checked it" in x for x in self.broken("act-1.0.tnrel", edit)))


@unittest.skipUnless(NODE and PLAYWRIGHT, "Node.js and Playwright (with its Chromium) are needed for the browser test")
class Browser(unittest.TestCase):
    def test_the_release_side_of_the_group_app(self):
        with tempfile.TemporaryDirectory() as d:
            d = pathlib.Path(d)
            built = pages.build(d / "pages")
            seed_design.seed(d / "design", sync=False)
            form = filled_form(d)
            r = subprocess.run([NODE, str(ROOT / "tests" / "browser" / "release.test.mjs"), str(built["group"][0]), str(built["node"][0]),
                                str(d / "design"), str(form), str(d / "out")],
                               capture_output=True, text=True, timeout=1500, env={**os.environ, "PLAYWRIGHT_MODULE": str(PLAYWRIGHT)})
            sys.stdout.write(r.stdout[-3000:])
            self.assertEqual(r.returncode, 0, r.stdout[-4000:] + r.stderr[-2000:])
            out = d / "out"
            self.assertEqual([e for f in sorted(out.rglob("*.tndb")) for e in tndb.check(f)], [])
            self.assertEqual(group.check(out), [])
            fs = release.files([out])
            self.assertEqual(sorted(f.name for f in fs), ["act-1.0.tnrel", "act-1.1.tnrel", "catalogue-1.0.tnrel", "env-1.0.tnrel"])
            self.assertEqual([p for f in fs for p in release.check(f)], [])


if __name__ == "__main__":
    unittest.main()
