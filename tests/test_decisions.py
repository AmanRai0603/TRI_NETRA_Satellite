"""The decision records (S11.2): design/decisions/DR-*.toml are complete and unsigned, cover the decisions S11 names, and the
regression copy holds one node of kind decision per file, in the file's group, with its choice and an empty signature."""
import json
import pathlib
import re
import sqlite3
import tomllib

ROOT = pathlib.Path(__file__).resolve().parents[1]
FILES = sorted((ROOT / "design" / "decisions").glob("DR-*.toml"))
REQUIRED = ["schema", "id", "decisions", "title", "group", "parent", "owner_team", "status", "context", "current", "choice",
            "rationale", "consequences", "signature", "option"]
COVER = ["DD8", "DD2", "DD39", "DD15", "DD40", "DD50", "DD12", "DD59", "DD60", "DD9", "DD61", "DD14", "DD19", "DD46", "DD47",
         "DD22", "DD23", "DD33", "DD35", "DD57", "DD52", "DD1", "DD38"]


def load():
    return {f: tomllib.loads(f.read_text(encoding="utf-8")) for f in FILES}


def test_records_complete_and_unsigned():
    recs = load()
    assert len(recs) == 36
    for f, d in recs.items():
        assert not [k for k in REQUIRED if k not in d], f.name
        assert d["status"] == "proposed", f.name
        assert not d["signature"], f.name
        assert len(d["option"]) >= 2, f.name


def test_coverage():
    recs = load()
    dds = {x for d in recs.values() for x in d["decisions"]}
    assert not set(COVER) - dds, sorted(set(COVER) - dds)
    qs = {d["owner_question"] for d in recs.values() if "owner_question" in d and d.get("owner_question_stage", "S11") == "S11"}
    assert set(range(1, 10)) <= qs, sorted(qs)
    text = (ROOT / "docs" / "DESIGN_DECISIONS.md").read_text(encoding="utf-8")
    for x in sorted(dds):
        if re.fullmatch(r"DD\d+", x):
            assert re.search(rf"\b{x}\b", text), f"{x} is not in docs/DESIGN_DECISIONS.md"


def test_s12_questions_carried_once():
    """S12.1: every owner question 1-24 of docs/S12_INVENTORY.md section 5 is carried by exactly one record, on its default,
    proposed and unsigned. A record that also carries an S11 question holds its S12 ones in owner_question_s12."""
    carried = []
    for f, d in load().items():
        if d.get("owner_question_stage") == "S12":
            n = d["owner_question"]
            carried += n if isinstance(n, list) else [n]
        carried += d.get("owner_question_s12", [])
    assert sorted(carried) == list(range(1, 25)), sorted(carried)


def test_regression_copy_holds_them():
    c = sqlite3.connect(ROOT / "tests" / "regression" / "design.tndb")
    got = {r[0]: r[1] for r in c.execute("SELECT id, group_id FROM design_node WHERE kind = 'decision'")}
    recs = load()
    assert len(got) == len(recs)
    for f, d in recs.items():
        nid = d["id"].lower().replace("-", "_")
        assert got.get(nid) == d["group"], f.name
        rows = {r[1]: r[2] for r in json.loads(c.execute("SELECT content FROM design_node WHERE id = ?", (nid,)).fetchone()[0])["body"]["content"]
                if r[0] == "decision"}
        assert rows["choice"] == d["choice"], f.name
        assert rows["signature"] == "", f.name
