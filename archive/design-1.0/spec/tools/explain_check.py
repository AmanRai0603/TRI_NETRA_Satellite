#!/usr/bin/env python3
"""Checks every page a person reads against the explanation standard
adcs-explain/1 (SPEC.md §5.12): the HTML templates, rendered in headless
Chromium from their examples, and every manual page.

    python3 tools/explain_check.py             check everything
    python3 tools/explain_check.py --md        the manual pages only (no browser needed)
    python3 tools/explain_check.py --selftest  each deliberate removal is caught by its rule

It checks that the marks the standard names are present and in order. It does
not check that a page is clear: that is a reader's test (risk R-17, §5.13).
"""

import asyncio
import os
import re
import shutil
import subprocess
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(ROOT, "tools"))

KINDS = ["tutorial", "how-to", "reference", "explanation"]
DEPTHS = ["learn", "read", "expert"]
RULES = {
    "E01": "answer first: a data-explain=answer block before any section (Markdown: the first paragraph starts **In one line:**)",
    "E02": "every section says its kind, tutorial, how-to, reference or explanation, with a visible badge (Markdown: the kind line)",
    "E03": "a Learn · Read · Expert switch; data-show names only those depths (Markdown: depth in the kind line)",
    "E04": "an overview before the details",
    "E05": "a breadcrumb saying where the reader is",
    "E08": "a page holding test vectors lets the reader rebuild one before it is shown",
    "E09": "every state is an icon and a word",
    "E10": "stations run: say it simply, the real thing, where it breaks, try it",
    "E12": "a two-cases box is not empty",
    "E13": "a common wrong idea says why it is wrong",
    "E14": "an analogy says where it stops being true",
    "E15": "a why-chain ends at a named floor",
    "E16": "a prediction is asked for in Learn only",
    "E17": "a document has its one-line summary and ends, in Learn, with a teach-back",
    "E18": "claim tags are sourced, derived, reference or example",
    "E19": "every listed source says what it is used for",
    "E20": "the page says where it breaks",
    "M01": "a manual page's kind has the sections its kind needs (tutorial: steps, Try it, Explain it back; how-to: steps, If it goes wrong; reference: a table; explanation: Say it simply, where it breaks, Common wrong idea)",
}

# The in-page check. Returns [[rule, text], ...]. `opts.teach` says whether the
# page is a document that must end with a teach-back.
PAGE_JS = r"""
(opts) => {
  const F = [];
  const add = (r, t) => F.push([r, t]);
  const all = (s, root) => Array.from((root || document).querySelectorAll(s));
  const KINDS = ["tutorial", "how-to", "reference", "explanation"], DEPTHS = ["learn", "read", "expert"];
  const main = document.querySelector("main") || document.body;
  const answer = document.querySelector('[data-explain="answer"]');
  if (!answer || !answer.textContent.trim()) add("E01", "no answer-first block");
  else {
    const first = all("section, [data-kind]", main).find(x => !x.contains(answer) && !answer.contains(x));
    if (first && (first.compareDocumentPosition(answer) & Node.DOCUMENT_POSITION_FOLLOWING)) add("E01", "a section comes before the answer");
  }
  const secs = all("section.card, section[data-kind]", main).filter(s => !s.closest("template"));
  if (!secs.length) add("E02", "no sections");
  secs.forEach(s => {
    const k = s.getAttribute("data-kind"), h = s.querySelector("h2, h3");
    const name = h ? h.textContent.trim().slice(0, 50) : (s.id || "?");
    if (KINDS.indexOf(k) < 0) add("E02", "section “" + name + "” has kind " + JSON.stringify(k));
    else if (!s.querySelector(".xk-kind")) add("E02", "section “" + name + "” shows no kind badge");
  });
  const sw = document.querySelector("[data-depth-switch]");
  if (!sw || sw.querySelectorAll("button").length !== 3) add("E03", "no Learn · Read · Expert switch");
  if (DEPTHS.indexOf(document.body.getAttribute("data-depth")) < 0) add("E03", "the body has no depth");
  all("[data-show]").forEach(x => {
    const v = x.getAttribute("data-show").split(/\s+/).filter(Boolean);
    if (!v.length || v.some(d => DEPTHS.indexOf(d) < 0)) add("E03", "data-show " + JSON.stringify(x.getAttribute("data-show")));
  });
  if (!document.querySelector('[data-explain="overview"]')) add("E04", "no overview");
  if (!document.querySelector('[data-explain="zoom"]')) add("E05", "no breadcrumb");
  if (document.querySelector("[data-holds-vectors]") && !document.querySelector('[data-explain="rebuild"]')) add("E08", "test vectors but no Rebuild it");
  all("[data-state]").forEach(x => {
    const icon = x.querySelector('[aria-hidden="true"]');
    const word = (x.textContent || "").replace(icon ? icon.textContent : "", "").trim();
    if (!icon || !icon.textContent.trim() || !word) add("E09", "a state without an icon and a word: " + JSON.stringify(x.textContent.trim().slice(0, 30)));
  });
  const ORDER = ["simply", "real", "breaks", "try"];
  const parents = new Set(all("[data-step]").map(x => x.parentElement));
  parents.forEach(p => {
    const seq = Array.from(p.children).filter(c => c.hasAttribute("data-step")).map(c => ORDER.indexOf(c.getAttribute("data-step")));
    if (seq.some(i => i < 0)) add("E10", "an unknown station step");
    for (let i = 1; i < seq.length; i++) if (seq[i] <= seq[i - 1]) { add("E10", "station steps out of order"); break; }
  });
  all('[data-explain="contrast"]').forEach(x => { if (!x.textContent.trim()) add("E12", "an empty two-cases box"); });
  all('[data-explain="wrong-idea"]').forEach(x => { if (!x.querySelector('[data-explain="because"]')) add("E13", "a wrong idea without its because"); });
  all('[data-explain="analogy"]').forEach(x => { if (!x.querySelector('[data-explain="analogy-breaks"]')) add("E14", "an analogy without where it breaks"); });
  all('[data-explain="why"]').forEach(x => { const li = x.querySelectorAll("li"); if (!li.length || !li[li.length - 1].textContent.trim()) add("E15", "a why-chain with no floor"); });
  all('[data-explain="predict"]').forEach(x => { if ((x.getAttribute("data-show") || x.closest("[data-show]")?.getAttribute("data-show") || "").trim() !== "learn") add("E16", "a prediction shown outside Learn"); });
  if (opts.teach) {
    const t = document.querySelector('[data-explain="teach-back"]');
    if (!t) add("E17", "no teach-back");
    else if ((t.getAttribute("data-show") || "").trim() !== "learn") add("E17", "the teach-back is not Learn-only");
    if (!document.querySelector('[data-explain="one-line"]')) add("E17", "no one-line summary");
  }
  all("[data-claim]").forEach(x => { if (["sourced", "derived", "reference", "example"].indexOf(x.getAttribute("data-claim")) < 0) add("E18", "claim " + JSON.stringify(x.getAttribute("data-claim"))); });
  all("[data-source-entry]").forEach(x => { if (!x.querySelector('[data-explain="used-for"]')) add("E19", "source " + JSON.stringify(x.textContent.trim().slice(0, 30)) + " says nothing about what it is used for"); });
  if (!document.querySelector('[data-explain="breaks"]')) add("E20", "nowhere says where the page breaks");
  return F;
}
"""


def pages(tmp):
    """(label, path, is a document that ends with a teach-back) for every page to check."""
    import forms  # noqa: F401  (the package's exporter)
    out = []
    for d in ("forms/examples", "results/examples"):
        for f in sorted(os.listdir(os.path.join(ROOT, d))):
            if f.endswith(".html"):
                teach = not f.startswith("case_")
                out.append((d + "/" + f, os.path.join(ROOT, d, f), teach))
    lib = os.path.join(tmp, "library")
    subprocess.run([sys.executable, os.path.join(ROOT, "tools", "forms.py"), "library", "--out", lib, "--only", "gf_7,p1k_0,rk4_0"],
                   check=True, stdout=subprocess.DEVNULL)
    out.append(("forms/library.html (as exported)", os.path.join(lib, "index.html"), False))
    nar = os.path.join(tmp, "narrative")
    subprocess.run([sys.executable, os.path.join(ROOT, "tools", "derisk.py"), "narrative", "--quarter", "Q3-26", "--out", nar],
                   check=True, stdout=subprocess.DEVNULL)
    out.append(("the de-risking narrative (as written)", os.path.join(nar, "narrative_Q3-26.html"), True))
    return out


async def check_html(tmp, mutate=None):
    from playwright.async_api import async_playwright
    found = {}
    async with async_playwright() as p:
        b = await p.chromium.launch()
        for label, path, teach in pages(tmp):
            pg = await b.new_page(viewport={"width": 1280, "height": 900})
            errs = []
            pg.on("pageerror", lambda e: errs.append(str(e)))
            await pg.goto("file://" + path)
            await pg.wait_for_timeout(250)
            if mutate:
                await pg.evaluate(mutate)
            F = await pg.evaluate(PAGE_JS, {"teach": teach})
            F += [["E00", "script error: " + e] for e in errs]
            found[label] = F
            await pg.close()
        await b.close()
    return found


KIND_LINE = re.compile(r"^<!--\s*kind:\s*(tutorial|how-to|reference|explanation)\s*;\s*depth:\s*(learn|read|expert)\s*-->\s*$")


def check_md_text(text):
    F = []
    lines = text.split("\n")
    if not lines or not lines[0].startswith("# "):
        return [["E01", "the page does not start with its title"]]
    body = [l for l in lines[1:6]]
    kl = next((l for l in body if l.startswith("<!--")), None)
    m = KIND_LINE.match(kl or "")
    if not m:
        F.append(["E02", "no kind line (<!-- kind: how-to; depth: read -->) under the title"])
        kind = None
    else:
        kind = m.group(1)
    paras = [p.strip() for p in "\n".join(lines[1:]).split("\n\n") if p.strip() and not p.strip().startswith("<!--")]
    if not paras or not paras[0].startswith("**In one line:**"):
        F.append(["E01", "the first paragraph does not start with **In one line:**"])
    heads = [l.lstrip("#").strip().lower() for l in lines if l.startswith("## ") or l.startswith("### ")]
    numbered = any(re.match(r"^\d+\. ", l) for l in lines) or any(h[:2].rstrip(".").isdigit() for h in heads)
    if kind == "tutorial":
        if not numbered:
            F.append(["M01", "a tutorial needs numbered steps"])
        if not any(h.startswith("try it") for h in heads):
            F.append(["M01", "a tutorial needs a “Try it” section"])
        if not any(h.startswith("explain it back") for h in heads):
            F.append(["M01", "a tutorial needs an “Explain it back” section"])
    elif kind == "how-to":
        if not numbered:
            F.append(["M01", "a how-to needs numbered steps"])
        if not any(h.startswith("if it goes wrong") for h in heads):
            F.append(["M01", "a how-to needs an “If it goes wrong” section"])
    elif kind == "reference":
        if not any(l.startswith("|") for l in lines):
            F.append(["M01", "a reference page needs a table"])
    elif kind == "explanation":
        if not any(h.startswith("say it simply") for h in heads):
            F.append(["M01", "an explanation needs “Say it simply”"])
        if not any("break" in h or "stops being true" in h for h in heads):
            F.append(["M01", "an explanation needs a section on where it breaks"])
        if not any(h.startswith("common wrong idea") for h in heads):
            F.append(["M01", "an explanation needs “Common wrong idea”"])
    if re.search(r"(?im)^##+ sources\b", text) and "Used for" not in text:
        F.append(["E19", "a Sources section without “Used for”"])
    return F


def md_files():
    out = []
    for d in ("manual/user", "manual/developer"):
        for f in sorted(os.listdir(os.path.join(ROOT, d))):
            if f.endswith(".md"):
                out.append((d + "/" + f, os.path.join(ROOT, d, f)))
    return out


def check_md():
    return {rel: check_md_text(open(p, encoding="utf-8").read()) for rel, p in md_files()}


def report(found):
    n = 0
    for label, F in found.items():
        for r, t in F:
            print("%s %s: %s" % (r, label, t))
            n += 1
    return n


def selftest():
    bad = 0
    good = "# T\n<!-- kind: explanation; depth: read -->\n\n**In one line:** x.\n\n## Say it simply\n\ny\n\n## Where it breaks\n\nz\n\n## Common wrong idea\n\nw\n"
    if check_md_text(good):
        print("a good page fails:", check_md_text(good))
        bad += 1
    md_cases = [
        ("E02", good.replace("<!-- kind: explanation; depth: read -->\n", "")),
        ("E01", good.replace("**In one line:** x.", "x.")),
        ("M01", good.replace("## Common wrong idea", "## Other")),
        ("M01", good.replace("explanation", "how-to")),
        ("M01", good.replace("explanation", "reference")),
        ("M01", good.replace("explanation", "tutorial")),
        ("E19", good + "\n## Sources\n\n- a book\n"),
    ]
    for code, text in md_cases:
        if code not in {r for r, _ in check_md_text(text)}:
            print("NOT CAUGHT %s in a Markdown page" % code)
            bad += 1
    html_cases = [
        ("E01", "document.querySelectorAll('[data-explain=\"answer\"]').forEach(x => x.remove())"),
        ("E02", "(document.querySelector('section.card .xk-kind') || {remove(){}}).remove()"),
        ("E03", "document.querySelectorAll('[data-depth-switch]').forEach(x => x.remove())"),
        ("E04", "document.querySelectorAll('[data-explain=\"overview\"]').forEach(x => x.removeAttribute('data-explain'))"),
        ("E05", "document.querySelectorAll('[data-explain=\"zoom\"]').forEach(x => x.remove())"),
        ("E20", "document.querySelectorAll('[data-explain=\"breaks\"]').forEach(x => x.removeAttribute('data-explain'))"),
        ("E14", "document.querySelectorAll('[data-explain=\"analogy-breaks\"]').forEach(x => x.remove())"),
        ("E13", "document.querySelectorAll('[data-explain=\"because\"]').forEach(x => x.remove())"),
        ("E18", "(function(){var s=document.createElement('span');s.setAttribute('data-claim','guess');document.body.appendChild(s);})()"),
        ("E08", "document.querySelectorAll('[data-explain=\"rebuild\"]').forEach(x => x.removeAttribute('data-explain'))"),
        ("E09", "document.querySelectorAll('[data-state] [aria-hidden=\"true\"]').forEach(x => { x.textContent = ''; })"),
        ("E10", "document.querySelectorAll('[data-step=\"simply\"]').forEach(x => x.parentElement.appendChild(x))"),
        ("E12", "document.querySelectorAll('[data-explain=\"contrast\"]').forEach(x => { x.textContent = ''; })"),
        ("E15", "document.querySelectorAll('[data-explain=\"why\"] li:last-child').forEach(x => { x.textContent = ''; })"),
        ("E16", "document.querySelectorAll('[data-explain=\"predict\"]').forEach(x => x.setAttribute('data-show', 'learn read'))"),
        ("E17", "document.querySelectorAll('[data-explain=\"teach-back\"]').forEach(x => x.removeAttribute('data-explain'))"),
        ("E19", "document.querySelectorAll('[data-explain=\"used-for\"]').forEach(x => x.removeAttribute('data-explain'))"),
    ]
    tmp = tempfile.mkdtemp()
    try:
        base = asyncio.run(check_html(tmp))
        if any(base.values()):
            print("the pages fail before any removal:")
            report(base)
            return 1
        for code, js in html_cases:
            got = asyncio.run(check_html(tmp, js))
            if not any(code in {r for r, _ in F} for F in got.values()):
                print("NOT CAUGHT %s on any page" % code)
                bad += 1
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    if bad:
        return 1
    print("selftest ok: %d removals in Markdown and %d in the rendered pages, each caught by its rule" % (len(md_cases), len(html_cases)))
    return 0


def main():
    a = sys.argv[1:]
    if a == ["--selftest"]:
        return selftest()
    if a not in ([], ["--md"]):
        print(__doc__)
        return 2
    found = check_md()
    n_md = len(found)
    if a != ["--md"]:
        tmp = tempfile.mkdtemp()
        try:
            found.update(asyncio.run(check_html(tmp)))
        finally:
            shutil.rmtree(tmp, ignore_errors=True)
    n = report(found)
    print("explain check: %s — %d manual pages%s" % ("ok" if not n else "%d finding(s)" % n, n_md,
                                                        "" if a == ["--md"] else ", %d rendered pages" % (len(found) - n_md)))
    return 1 if n else 0


if __name__ == "__main__":
    sys.exit(main())
