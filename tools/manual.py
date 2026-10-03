#!/usr/bin/env python3
"""The apps' manual (docs/RELEASE_PLAN.md P7), from its one source, design/manual/: a guide per role
(author, lead, developer, user), the journey of a node, the glossary, the guide to TRI-NETRA Files,
and the tours of the apps (tours.toml). Every app opens it in place (Help), prints any page of it,
and shows a screen's tour once to someone opening it for the first time.

    python3 tools/manual.py            write design/js/manual.js and design/manual/journey.svg
    python3 tools/manual.py --check    say whether they are current, and hold the manual and the
                                       apps to its rules (exit 1 on any problem)

The rules:
  - every page says itself in one line first (**In one line:**), as the explanation standard asks,
    and names its role and id;
  - every tour target is in its app (a data-testid, or a tab by its name);
  - every field, choice and set of checks an app makes carries help beside it, except the node
    app's step fields, whose question, reason and example come from design/js/node_model.js; and
    every one of those has a question and a reason.

Pages are written in a small part of Markdown: headings, paragraphs, lists, tables, **bold**, *italic*,
`code`, and an image line naming a picture the generator draws (only `journey`).
Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import argparse
import base64
import json
import pathlib
import re
import sys
import tomllib

ROOT = pathlib.Path(__file__).resolve().parents[1]
SRC = ROOT / "design" / "manual"
OUT_JS = ROOT / "design" / "js" / "manual.js"
OUT_SVG = SRC / "journey.svg"
APPS = {"files": "files_app.js", "node": "node_app.js", "group": "group_app.js"}

INLINE = re.compile(r"(\*\*[^*]+\*\*|`[^`]+`|\*[^*\s][^*]*\*|\[[^\]]+\]\([^)]+\))")


def runs(text):
    """Inline Markdown as runs: [["t" | "b" | "i" | "code", text]]."""
    out = []
    for part in INLINE.split(text):
        if not part:
            continue
        if part.startswith("**"):
            out.append(["b", part[2:-2]])
        elif part.startswith("`"):
            out.append(["code", part[1:-1]])
        elif part.startswith("["):
            out.append(["t", part[1:part.index("]")]])
        elif part.startswith("*") and part.endswith("*") and len(part) > 2:
            out.append(["i", part[1:-1]])
        else:
            out.append(["t", part])
    return out


def cells(line):
    return [c.strip() for c in line.strip().strip("|").split("|")]


def parse(text):
    """A page: (meta, title, blocks)."""
    lines = text.split("\n")
    meta, title, blocks = {}, None, []
    i = 0
    para = []

    def flush():
        if para:
            blocks.append(["p", runs(" ".join(para))])
            para.clear()
    while i < len(lines):
        ln = lines[i]
        s = ln.strip()
        m = re.match(r"<!--\s*(.*?)\s*-->", s)
        if m:
            for kv in m.group(1).split(";"):
                if ":" in kv:
                    k, v = kv.split(":", 1)
                    meta[k.strip()] = v.strip()
            i += 1
            continue
        if not s:
            flush()
            i += 1
            continue
        h = re.match(r"(#{1,3})\s+(.*)", s)
        if h:
            flush()
            if len(h.group(1)) == 1 and title is None:
                title = h.group(2)
            else:
                blocks.append([f"h{len(h.group(1))}", runs(h.group(2))])
            i += 1
            continue
        img = re.match(r"!\[([^\]]*)\]\(([a-z_]+)\)$", s)
        if img:
            flush()
            blocks.append(["img", img.group(2), img.group(1)])
            i += 1
            continue
        if s.startswith("|"):
            flush()
            rows = []
            while i < len(lines) and lines[i].strip().startswith("|"):
                rows.append(lines[i])
                i += 1
            head = [runs(c) for c in cells(rows[0])]
            body = [[runs(c) for c in cells(r)] for r in rows[2:]]
            blocks.append(["table", head, body])
            continue
        lm = re.match(r"(-|\d+\.)\s+(.*)", s)
        if lm:
            flush()
            kind = "ul" if lm.group(1) == "-" else "ol"
            items = []
            while i < len(lines):
                t = lines[i]
                m2 = re.match(r"\s*(-|\d+\.)\s+(.*)", t)
                if m2 and (("ul" if m2.group(1) == "-" else "ol") == kind) and not t.startswith("    "):
                    items.append(m2.group(2))
                elif t.startswith("  ") and t.strip() and items:
                    items[-1] += " " + t.strip()
                else:
                    break
                i += 1
            blocks.append([kind, [runs(x) for x in items]])
            continue
        para.append(s)
        i += 1
    flush()
    return meta, title, blocks


def pages():
    out = []
    for f in sorted(SRC.glob("*.md")):
        meta, title, blocks = parse(f.read_text(encoding="utf-8"))
        out.append({"id": meta.get("id", f.stem), "role": meta.get("role", ""), "app": meta.get("app", ""), "title": title, "file": f.name, "blocks": blocks})
    return out


def tours():
    return tomllib.loads((SRC / "tours.toml").read_text(encoding="utf-8"))["tour"]


# ------------------------------------------------------------------ the journey, drawn

STEPS = [("Write", "author", "node app"), ("Check", "second engineer", "node app"), ("Sign stage", "stage owner", "group app"),
         ("Seal", "group lead", "group app"), ("Build", "developer team", "repository"), ("Accept", "group lead", "test app"),
         ("Release", "owner", "main")]


def journey_svg():
    w, bw, bh, gap, top = 1000, 118, 92, 22, 46
    x0 = (w - (len(STEPS) * bw + (len(STEPS) - 1) * gap)) / 2
    p = [f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} 250" width="{w}" height="250" font-family="Inter, Arial, sans-serif" role="img" aria-label="The journey of a node">',
         '<rect width="100%" height="100%" fill="#ffffff"/>',
         '<defs><marker id="a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto"><path d="M0,0 L10,5 L0,10 z" fill="#475569"/></marker></defs>',
         f'<text x="{w / 2}" y="26" text-anchor="middle" font-size="17" font-weight="600" fill="#0f172a">From an author to a release</text>']
    for k, (step, who, where) in enumerate(STEPS):
        x = x0 + k * (bw + gap)
        fill = "#e0f2fe" if where == "node app" else "#ede9fe" if where == "group app" else "#f1f5f9"
        p.append(f'<rect x="{x:.0f}" y="{top}" width="{bw}" height="{bh}" rx="10" fill="{fill}" stroke="#334155" stroke-width="1.2"/>')
        p.append(f'<text x="{x + bw / 2:.0f}" y="{top + 30}" text-anchor="middle" font-size="15" font-weight="600" fill="#0f172a">{k + 1} {step}</text>')
        p.append(f'<text x="{x + bw / 2:.0f}" y="{top + 54}" text-anchor="middle" font-size="12" fill="#334155">{who}</text>')
        p.append(f'<text x="{x + bw / 2:.0f}" y="{top + 74}" text-anchor="middle" font-size="12" fill="#64748b">{where}</text>')
        if k:
            xa = x - gap + 2
            p.append(f'<line x1="{xa:.0f}" y1="{top + bh / 2}" x2="{x - 2:.0f}" y2="{top + bh / 2}" stroke="#475569" stroke-width="1.6" marker-end="url(#a)"/>')
    # the ways back
    def back(a, b, y, label):
        xa = x0 + a * (bw + gap) + bw / 2
        xb = x0 + b * (bw + gap) + bw / 2
        p.append(f'<path d="M{xa:.0f},{top + bh} V{y} H{xb:.0f} V{top + bh + 4}" fill="none" stroke="#94a3b8" stroke-width="1.4" stroke-dasharray="5 4" marker-end="url(#a)"/>')
        p.append(f'<text x="{(xa + xb) / 2:.0f}" y="{y - 6}" text-anchor="middle" font-size="12" fill="#475569">{label}</text>')
    back(1, 0, top + bh + 34, "an edit takes the signature off")
    back(3, 0, top + bh + 70, "re-issue opens a sealed node again")
    back(5, 3, top + bh + 34, "not accepted: back with reasons")
    p.append("</svg>")
    return "\n".join(p) + "\n"


def module():
    svg = journey_svg()
    data = {"pages": pages(), "tours": tours(), "images": {"journey": "data:image/svg+xml;base64," + base64.b64encode(svg.encode()).decode()}}
    body = json.dumps(data, ensure_ascii=False, indent=1, sort_keys=True).replace("<", "\\u003c")
    return ("// manual.js -- written by tools/manual.py from design/manual/ (do not edit): the apps' manual,\n"
            "// its tours and the journey diagram (docs/RELEASE_PLAN.md P7).\n"
            "// Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.\n"
            f"export const MANUAL = {body};\n"), svg


# ------------------------------------------------------------------ the rules

def call_args(src, start):
    """The text of a call's arguments, from the "(" at start to its matching ")"."""
    depth, i, q = 0, start, None
    while i < len(src):
        c = src[i]
        if q:
            if c == "\\":
                i += 2
                continue
            if c == q:
                q = None
        elif c in "\"'`":
            q = c
        elif c in "([{":
            depth += 1
        elif c in ")]}":
            depth -= 1
            if depth == 0:
                return src[start + 1:i]
        i += 1
    return src[start + 1:]


def problems():
    out = []
    ps = pages()
    for p in ps:
        if not p["title"]:
            out.append(f"design/manual/{p['file']}: no title (# Title)")
        if not p["role"] or not p["id"]:
            out.append(f"design/manual/{p['file']}: names no role or id (<!-- role: …; id: … -->)")
        first = next((b for b in p["blocks"] if b[0] == "p"), None)
        if not first or not (first[1] and first[1][0] == ["b", "In one line:"]):
            out.append(f"design/manual/{p['file']}: does not start with **In one line:** (the explanation standard)")
    ids = [p["id"] for p in ps]
    if len(ids) != len(set(ids)):
        out.append(f"design/manual: two pages share an id: {ids}")
    srcs = {k: (ROOT / "design" / "js" / v).read_text(encoding="utf-8") for k, v in APPS.items()}
    tn_ui = (ROOT / "design" / "js" / "tn_ui.js").read_text(encoding="utf-8")
    for name, t in tours().items():
        src = srcs.get(t.get("app"))
        if src is None:
            out.append(f"tours.toml: {name}: no app {t.get('app')!r}")
            continue
        for s in t["steps"]:
            tg = s["target"]
            m = re.fullmatch(r'\[data-tab\^?="([^"]+)"\]', tg)
            if m:
                if f'"{m.group(1)}"' not in src and f'`{m.group(1)}' not in src:
                    out.append(f"tours.toml: {name}: {t['app']} has no tab {m.group(1)}")
            elif f'testid: "{tg}"' not in src and f'testid: "{tg}"' not in tn_ui:
                out.append(f"tours.toml: {name}: {t['app']} has nothing with data-testid {tg}")
            if not s.get("title") or not s.get("text"):
                out.append(f"tours.toml: {name}: a step without its title or text")
    for app, src in srcs.items():
        for m in re.finditer(r"\b(field|choice|checks)\(", src):
            if src[max(0, m.start() - 9):m.start()].endswith("function ") or src[m.end():m.end() + 2] == "s)":
                continue
            args = call_args(src, m.end() - 1)
            if "help:" in args:
                continue
            if app == "node" and args.lstrip().startswith("fd.label"):
                continue
            line = src.count("\n", 0, m.start()) + 1
            out.append(f"design/js/{APPS[app]}:{line}: a {m.group(1)} without help beside it")
    model = (ROOT / "design" / "js" / "node_model.js").read_text(encoding="utf-8")
    for m in re.finditer(r'\bf\("([a-z_.]+)",\s*"([^"]*)",\s*"([^"]*)",\s*"([^"]*)"', model):
        if not m.group(3).strip() or not m.group(4).strip():
            out.append(f"design/js/node_model.js: field {m.group(1)} has no question or no reason")
    return out


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--check", action="store_true", help="say whether the written files are current, and check the rules")
    a = ap.parse_args(argv)
    js, svg = module()
    if a.check:
        bad = problems()
        if not OUT_JS.exists() or OUT_JS.read_text(encoding="utf-8") != js:
            bad.append("design/js/manual.js is not current: run python3 tools/manual.py")
        if not OUT_SVG.exists() or OUT_SVG.read_text(encoding="utf-8") != svg:
            bad.append("design/manual/journey.svg is not current: run python3 tools/manual.py")
        for b in bad:
            print(b)
        print(f"manual: {len(pages())} page(s), {len(tours())} tour(s), {len(bad)} problem(s)")
        return 1 if bad else 0
    OUT_JS.write_text(js, encoding="utf-8")
    OUT_SVG.write_text(svg, encoding="utf-8")
    print(f"wrote design/js/manual.js ({len(pages())} pages, {len(tours())} tours) and design/manual/journey.svg")
    bad = problems()
    for b in bad:
        print(b)
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
