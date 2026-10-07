"""The design database (design.tndb) from Python: every group and node as released, the edges
between them, the catalogue of outputs, and the engine's inputs (the cases row by row and the
scenarios), read with the standard library's sqlite3 and nothing to install.

    import trinetra_adcs.design as d
    db = d.open()                          # $TRINETRA_DESIGN, else the one this package carries
    db.groups()                            # [{"id", "version", "fingerprint", "merged_at"}]
    db.node("gd_0")["content"]             # what the node's release says, as data
    db.readers("gd_0")                     # the nodes that read it
    db.case("ais_3u")["req.ape"]           # 10.0 (a number; None when the case leaves it blank)
    db.case_rows("ais_3u")                 # every row: key, label, unit, value, ..., the declaring node
    db.scenario("nadir_hold_ais")          # the scenario as the engine flies it

    python -m trinetra_adcs.design [groups | nodes GROUP | node ID | cases | case ID | scenarios]

The file is opened read-only; nothing here writes it (the group app and tools/group.py merge do).
A file that is not a design database, or one older than the engine's inputs (format version 2),
is refused by name.

Copyright (c) 2026 Agastya. All rights reserved.
"""
import json
import os
import pathlib
import sqlite3
import sys

FORMAT_VERSION = 2
# The toolbox this package's engine offers (engine/crates/adcs-sim/src/source.rs, TOOLBOX); a design
# built for another, or one that needs a newer application, is refused by name.
TOOLBOX = "trinetra-toolbox/2"


def _semver(v):
    try:
        t = tuple(int(x) for x in str(v).strip().lstrip("v").split("."))
    except ValueError:
        return None
    return t if len(t) == 3 else None


def cannot_run(meta, version=None):
    """Why this package cannot run a design with this meta (None: it can)."""
    if version is None:
        try:
            from ._build import VERSION as version
        except ImportError:  # a checkout: the repository's VERSION
            f = pathlib.Path(__file__).resolve().parents[2] / "VERSION"
            version = f.read_text().strip() if f.is_file() else None
    rebuild = "rebuild it (python3 tools/seed_design.py, or tools/group.py merge)"
    t = meta.get("toolbox")
    if not t:
        return f"it names no toolbox: built before designs said what they need; {rebuild}"
    if t != TOOLBOX:
        return f"it was built for the toolbox {t}; this package offers {TOOLBOX}"
    need = _semver(meta.get("needs_application"))
    if need is None:
        return f"it names no application version it needs ({meta.get('needs_application')!r}); {rebuild}"
    me = _semver(version) if version else None
    if me is not None and need > me:
        return f"it needs TRI-NETRA {meta.get('needs_application')} or later; this is {version}"
    return None


class DesignError(Exception):
    """A file this reader cannot take."""


def default_path():
    """$TRINETRA_DESIGN, else the design.tndb this package carries."""
    if os.environ.get("TRINETRA_DESIGN"):
        return pathlib.Path(os.environ["TRINETRA_DESIGN"])
    here = pathlib.Path(__file__).resolve().parent
    p = here / "_kit" / "design.tndb"
    if p.is_file():
        return p
    raise DesignError("no design database: set TRINETRA_DESIGN (a checkout builds one with python3 tools/seed_design.py)")


def _number(s):
    s = (s or "").strip()
    if not s:
        return None
    try:
        return float(s)
    except ValueError:
        return s


class Design:
    def __init__(self, path):
        self.path = pathlib.Path(path)
        if self.path.is_dir():
            inside = next((f for f in (self.path / "design.tndb", self.path / "Design" / "design.tndb") if f.is_file()), None)
            raise DesignError(f"{self.path}: a folder, not a design database" +
                              (f"; the design database in it is {inside}" if inside else ", and it holds no design.tndb"))
        if not self.path.is_file():
            raise DesignError(f"{self.path}: no such file")
        self._c = sqlite3.connect(f"file:{self.path.as_posix()}?mode=ro", uri=True)
        try:
            self.meta = dict(self._c.execute('SELECT "key", "value" FROM meta'))
        except sqlite3.Error as e:
            raise DesignError(f"{self.path}: not a design file ({e})") from None
        if self.meta.get("format") != "design":
            raise DesignError(f"{self.path}: a {self.meta.get('format')} file, not a design database")
        if int(self.meta.get("format_version", 0)) < FORMAT_VERSION:
            raise DesignError(f"{self.path}: format version {self.meta.get('format_version')} holds no engine inputs; rebuild it")
        why = cannot_run(self.meta)
        if why:
            raise DesignError(f"{self.path}: this package cannot run it: {why}")

    def close(self):
        self._c.close()

    def __enter__(self):
        return self

    def __exit__(self, *_):
        self.close()

    def _rows(self, sql, args=()):
        cur = self._c.execute(sql, args)
        names = [d[0] for d in cur.description]
        return [dict(zip(names, r)) for r in cur]

    @property
    def fingerprint(self):
        """One hash over every engine input it holds (a run flown from it records the same)."""
        return self.meta.get("inputs_fingerprint")

    # the design
    def groups(self):
        return self._rows('SELECT * FROM design_group ORDER BY "id"')

    def nodes(self, group=None):
        q = 'SELECT "id", "group_id", "stage", "layer", "kind", "label", "release" FROM design_node'
        return self._rows(q + (' WHERE "group_id" = ? ORDER BY "id"' if group else ' ORDER BY "id"'), (group,) if group else ())

    def node(self, nid):
        r = self._rows('SELECT * FROM design_node WHERE "id" = ?', (nid,))
        if not r:
            raise KeyError(f"no node {nid} in {self.path.name}")
        r = r[0]
        r["content"] = json.loads(r["content"]) if r["content"] else None
        return r

    def edges(self):
        return self._rows('SELECT * FROM edge')

    def readers(self, nid):
        return sorted({e["to_node"] for e in self._rows('SELECT "to_node" FROM edge WHERE "from_node" = ?', (nid,))})

    def outputs(self):
        return self._rows('SELECT * FROM catalogue_output ORDER BY "node", "output"')

    # the engine's inputs
    def cases(self):
        return [r[0] for r in self._c.execute('SELECT DISTINCT "case_id" FROM design_case ORDER BY "case_id"')]

    def case_rows(self, case_id):
        rows = self._rows('SELECT * FROM design_case WHERE "case_id" = ? ORDER BY "ord"', (case_id,))
        if not rows:
            raise KeyError(f"no case {case_id} in {self.path.name}")
        return rows

    def case(self, case_id):
        """{key: value}: numbers as numbers, blanks as None, the meta rows as text."""
        return {r["key"]: (r["value"] if r["key"].startswith("meta.") else _number(r["value"])) for r in self.case_rows(case_id)}

    def input_file(self, path):
        r = self._c.execute('SELECT "body" FROM engine_input WHERE "path" = ?', (path,)).fetchone()
        if r is None:
            raise KeyError(f"{path} is not in {self.path.name}")
        return r[0]

    def scenarios(self):
        return [p[len("data/scenarios/"):-5] for (p,) in self._c.execute(
            'SELECT "path" FROM engine_input WHERE "path" LIKE \'data/scenarios/%.json\' ORDER BY "path"')]

    def scenario(self, sid):
        return json.loads(self.input_file(f"data/scenarios/{sid}.json"))


def open(path=None):  # noqa: A001 -- the module's verb, as sqlite3.connect is
    return Design(path or default_path())


def main(argv=None):
    a = sys.argv[1:] if argv is None else argv
    try:
        db = open()
    except DesignError as e:
        print(f"design: {e}", file=sys.stderr)
        return 2
    with db:
        what = a[0] if a else "groups"
        if what == "groups":
            for g in db.groups():
                print(f"{g['id']:<12} {g['version'] or '':<10} {len(db.nodes(g['id'])):>4} nodes")
        elif what == "nodes" and len(a) == 2:
            for n in db.nodes(a[1]):
                print(f"{n['id']:<10} {n['kind'] or '':<10} {n['label']}")
        elif what == "node" and len(a) == 2:
            print(json.dumps(db.node(a[1]), indent=1, ensure_ascii=False))
        elif what == "cases":
            print("\n".join(db.cases()))
        elif what == "case" and len(a) == 2:
            for r in db.case_rows(a[1]):
                print(f"{r['key']:<28} {r['value'] or '':>14} {r['unit'] or '':<12} {r['node'] or ''}")
        elif what == "scenarios":
            print("\n".join(db.scenarios()))
        else:
            print(__doc__.split("\n\n")[2], file=sys.stderr)
            return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
