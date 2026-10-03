#!/usr/bin/env python3
"""The offline pages (docs/RELEASE_PLAN.md P3): each one HTML file that runs from disk with nothing
fetched from anywhere, built from a template in design/pages/.

    python3 tools/pages.py build [--out DIR]   write every page (default build/pages/)
    python3 tools/pages.py check               build into a scratch folder and hold every page to
                                               the rules below; changes nothing

A template's directives:
    <!-- tn:generated -->          the "generated, do not edit" note, with what went in
    <!-- tn:css PATH -->           the stylesheet, its fonts (url("tn-font:NAME")) inlined from design/vendor/fonts
    <!-- tn:sqlite -->             SQLite in WebAssembly (design/vendor/sqljs): the script, and the wasm as base64
    <!-- tn:module PATH -->        an ES module and everything it imports ("./x.js"), as one inline module

The rules (check):
    - every vendored file is the one design/vendor/VENDOR.toml pins (SHA-256);
    - one component set: a page's own code makes no control, style or markup of its own (no
      button, input, textarea, select, dialog or link element, no style attribute or .style, no
      innerHTML): those come from design/js/tn_ui.js and design/css/tn.css;
    - no outside hosts: nothing in a built page loads from http(s) (src, href, url(), @import,
      fetch, import());
    - the modules of a page share one scope, so no two declare the same top-level name.

Built pages are not committed (they are built in CI and shipped in the kits); the browser tests
(tests/browser/files.test.mjs) run against a fresh build.
Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import argparse
import base64
import hashlib
import pathlib
import re
import sys
import tempfile
import tomllib

ROOT = pathlib.Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
from common import write_text  # noqa: E402

PAGES = {"files": "design/pages/files.template.html"}
VENDOR = ROOT / "design" / "vendor"
OUT = ROOT / "build" / "pages"
UI_FILES = {"design/js/tn_ui.js", "design/css/tn.css"}
MAX_PAGE_BYTES = 4 * 1024 * 1024

DIRECTIVE = re.compile(r"<!--\s*tn:(\w+)\s*([^>]*?)\s*-->")
IMPORT = re.compile(r'^\s*import\s*\{([^}]*)\}\s*from\s*"(\./[\w.]+)";?\s*$', re.M)
ANY_IMPORT = re.compile(r"^\s*import\b", re.M)
EXPORT = re.compile(r"^export\s+(?=(async\s+)?function|const|let|class)", re.M)
TOP_DECL = re.compile(r"^(?:export\s+)?(?:async\s+)?(?:function\*?|const|let|class)\s+([A-Za-z_$][\w$]*)", re.M)
FORBIDDEN = [
    (re.compile(r'\bh\(\s*"(button|input|textarea|select|dialog|a)"'), "makes a {0} element of its own"),
    (re.compile(r"createElement\("), "creates elements with createElement"),
    (re.compile(r"\.style\b|\bstyle\s*="), "sets a style of its own"),
    (re.compile(r"innerHTML|outerHTML|insertAdjacentHTML"), "writes markup as text"),
    (re.compile(r"<(button|input|textarea|select|dialog|style|script)\b", re.I), "has a <{0}> of its own"),
]
OUTSIDE = re.compile(r"""(?:\b(?:src|href)\s*=\s*["']?\s*https?:|url\(\s*["']?\s*https?:|@import\s+(?:url\()?\s*["']?https?:|\bfetch\(\s*["'`]https?:|\bimport\(\s*["'`]https?:)""", re.I)


class PageError(Exception):
    pass


def vendored(vendor=VENDOR):
    """The pinned vendored files, checked: {relative path: bytes}."""
    vendor = pathlib.Path(vendor)
    pins = tomllib.loads((vendor / "VENDOR.toml").read_text())
    out, errs = {}, []
    for lib, v in pins.items():
        for rel, want in v["files"].items():
            p = vendor / rel
            if not p.is_file():
                errs.append(f"design/vendor/{rel} ({lib}) is missing")
                continue
            b = p.read_bytes()
            got = hashlib.sha256(b).hexdigest()
            if got != want:
                errs.append(f"design/vendor/{rel} ({lib} {v['version']}) has SHA-256 {got}, VENDOR.toml pins {want}")
            out[rel] = b
    if errs:
        raise PageError("\n".join(errs))
    return out, pins


def bundle(entry):
    """An ES module and what it imports, one after the other, imports dropped and exports
    unmarked: one inline module. Returns (code, [files in order])."""
    order, seen = [], set()

    def visit(rel):
        if rel in seen:
            return
        seen.add(rel)
        text = (ROOT / rel).read_text()
        for _names, dep in IMPORT.findall(text):
            visit(str((pathlib.PurePosixPath(rel).parent / dep[2:])))
        stripped = IMPORT.sub("", text)
        if ANY_IMPORT.search(stripped):
            raise PageError(f"{rel}: an import the page builder cannot inline (only `import {{ a, b }} from \"./x.js\";`)")
        order.append((rel, EXPORT.sub("", stripped)))

    visit(entry)
    names = {}
    for rel, code in order:
        for n in TOP_DECL.findall(code):
            if n in names:
                raise PageError(f"{n} is declared at the top of both {names[n]} and {rel}; the page's modules share one scope")
            names[n] = rel
    code = "\n".join(f"// ---- {rel}\n{c}" for rel, c in order)
    return code, [rel for rel, _ in order]


def component_problems(rel):
    if rel in UI_FILES:
        return []
    text = (ROOT / rel).read_text()
    out = []
    for i, line in enumerate(text.splitlines(), 1):
        if line.lstrip().startswith("//"):
            continue
        for rx, msg in FORBIDDEN:
            m = rx.search(line)
            if m:
                out.append(f"{rel}:{i}: {msg.format(*(m.groups() or ['']))}; use design/js/tn_ui.js")
    return out


def build_page(template, files):
    t = (ROOT / template).read_text()
    used = [template]
    problems = []
    for m in DIRECTIVE.finditer(t):
        if m.group(1) == "module":
            problems += component_problems(m.group(2))
    leftover = DIRECTIVE.sub("", t)
    problems += [f"{template}: {p.split(': ', 1)[1]}" for p in component_problems_text(template, leftover)]
    if problems:
        raise PageError("\n".join(problems))

    def css(path):
        used.append(path)
        text = (ROOT / path).read_text()

        def font(m):
            rel = f"fonts/{m.group(1)}"
            if rel not in files:
                raise PageError(f"{path}: font {m.group(1)} is not vendored (design/vendor/VENDOR.toml)")
            return f'url("data:font/woff2;base64,{base64.b64encode(files[rel]).decode()}")'
        return "<style>\n" + re.sub(r'url\("tn-font:([\w.-]+)"\)', font, text) + "</style>"

    def module(path):
        code, deps = bundle(path)
        used.extend(deps)
        for d in deps:
            if d not in UI_FILES:
                problems.extend(component_problems(d))
        return f'<script type="module">\n{code}\n</script>'

    def sqlite():
        js, wasm = files["sqljs/sql-wasm.js"].decode(), files["sqljs/sql-wasm.wasm"]
        return (f"<script>\n{js}\n</script>\n"
                f'<script type="application/octet-stream" id="tn-sqlite-wasm">\n{base64.b64encode(wasm).decode()}\n</script>')

    def directive(m):
        kind, arg = m.group(1), m.group(2)
        if kind == "css":
            return css(arg)
        if kind == "module":
            return module(arg)
        if kind == "sqlite":
            return sqlite()
        if kind == "generated":
            return "<!--GENERATED-->"
        raise PageError(f"{template}: no directive tn:{kind}")

    page = DIRECTIVE.sub(directive, t)
    if problems:
        raise PageError("\n".join(problems))
    note = (f"<!-- Generated by tools/pages.py from {', '.join(used)}; do not edit. SQLite: sql.js "
            f"{files_version('sqljs')}; fonts: Inter, JetBrains Mono (SIL OFL 1.1). Nothing is loaded from outside this file.\n"
            "     Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved. -->")
    page = page.replace("<!--GENERATED-->", note)
    for m in OUTSIDE.finditer(page):
        line = page.count("\n", 0, m.start()) + 1
        raise PageError(f"{template}: the built page loads from an outside host (line {line}: {page[m.start():m.start() + 60]!r})")
    if len(page.encode()) > MAX_PAGE_BYTES:
        raise PageError(f"{template}: the built page is {len(page.encode())} bytes, over {MAX_PAGE_BYTES}")
    return page


def component_problems_text(name, text):
    out = []
    for i, line in enumerate(text.splitlines(), 1):
        for rx, msg in FORBIDDEN:
            m = rx.search(line)
            if m:
                out.append(f"{name}:{i}: {msg.format(*(m.groups() or ['']))}")
    return out


_PINS = {}


def files_version(lib):
    return _PINS.get(lib, {}).get("version", "?")


def build(out=OUT):
    files, pins = vendored()
    _PINS.update(pins)
    out = pathlib.Path(out)
    written = {}
    for name, template in PAGES.items():
        page = build_page(template, files)
        p = out / f"{name}.html"
        write_text(p, page)
        written[name] = (p, len(page.encode()))
    return written


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    b = sub.add_parser("build", help="write every page")
    b.add_argument("--out", default=str(OUT))
    sub.add_parser("check", help="build into a scratch folder and hold every page to the rules")
    a = ap.parse_args(argv)
    try:
        if a.cmd == "build":
            for name, (p, n) in build(a.out).items():
                print(f"pages: {name} -> {p} ({n} bytes)")
        else:
            with tempfile.TemporaryDirectory() as d:
                w = build(d)
            print(f"pages: {len(w)} page(s) built and checked: vendored files pinned, one component set, no outside hosts")
    except PageError as e:
        for line in str(e).splitlines():
            print("pages: " + line, file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
