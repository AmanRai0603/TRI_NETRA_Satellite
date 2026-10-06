#!/usr/bin/env python3
"""Opens the three documents a team member uses in headless Chromium and proves
each loop works end to end (SPEC.md §5.10.8, §8.3.4, §13.5.6):

node form     draws with no script error; two edits and a name and a reason read
              "2 changes" with nothing to fix; "Save filled copy" writes a file that
              re-opens with the edits (a change with no belief record is flagged D01
              until the De-risking section is filled); the depth switch hides the
              tutorial in Expert and Rebuild it's answer in Learn until asked;
              tools/intake.py passes the saved file and
              writes a brief; an assistant's name as requester is flagged on the
              page and refused by the checker; the developer's reply shows in the
              returned file's status; a new-node request fills its id from group
              and label; unsaved answers are offered back after a reload.
case editor   a reference case loads; one edit downloads a CSV that passes
              tools/check_case.py and differs from the reference in that row only;
              a CSV with a wrong unit is refused with a message.
library       draws, lists every node, and search narrows it.
results       every chart draws, the channels decode, and a second result opened
              for comparison is overlaid; importing both into an empty store files
              each beside its case, and the index lists them.
phone width   the node form and the case editor have no sideways scroll at 390 px.

    python3 tools/form_browser_check.py

Needs Python 3.11+ and Playwright with Chromium (the CI image has both).
"""

import asyncio
import json
import os
import subprocess
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(ROOT, "tools"))


async def run(tmp):
    from playwright.async_api import async_playwright
    import forms
    import plan_model as pm
    import intake
    plan = pm.Plan()
    node = forms.export_node(plan, "gf_7", tmp)
    new = forms.export_new(plan, tmp)
    case = forms.export_case(plan, os.path.join(ROOT, "plan", "cases", "ais_img_3u.csv"), tmp)
    lib, n_nodes = forms.export_library(plan, os.path.join(tmp, "library"))
    fails, errs = [], []
    async with async_playwright() as p:
        b = await p.chromium.launch()
        ctx = await b.new_context(accept_downloads=True)
        dialogs = []

        async def page_of(path, width=1280):
            pg = await ctx.new_page()
            await pg.set_viewport_size({"width": width, "height": 900})
            pg.on("pageerror", lambda e: errs.append("%s: %s" % (os.path.basename(path), e)))
            pg.on("console", lambda m: errs.append(m.text) if m.type == "error" else None)

            async def on_dialog(d):
                dialogs.append(d.message)
                await d.accept()
            pg.on("dialog", on_dialog)
            await pg.goto("file://" + path)
            await pg.wait_for_timeout(150)
            return pg

        async def issues(pg):
            return [x.strip() for x in await pg.locator("#review .issues li.error").all_inner_texts()]

        # ---------------------------------------------------------- node form
        pg = await page_of(node)
        if (await pg.inner_text("#status")).strip() != "unchanged":
            fails.append("node form: a fresh form does not read 'unchanged'")
        await pg.fill('textarea[aria-label="What does a reader need told that the question does not say?"]', "Laminar flow only; see the assumption.")
        await pg.fill('input[aria-label="The highest value it may return"]', "600")
        await pg.fill('input[aria-label="Your name"]', "Form Check")
        await pg.fill('textarea[aria-label="Why"]', "browser check")
        # the belief behind the change (SPEC.md §5.13): without it the page lists D01-D03
        if not any("D01" in x for x in await issues(pg)):
            fails.append("node form: a change with no belief record is not flagged (D01)")
        await pg.select_option('select[aria-label="Which area is the belief about?"]', "node")
        await pg.fill('textarea[aria-label="What did we believe?"]', "readers would find the laminar limit under the assumptions")
        await pg.select_option('select[aria-label="Has a test settled it?"]', "broke")
        await pg.fill('textarea[aria-label="What was tested?"]', "a reading of the node page by the browser check")
        await pg.fill('textarea[aria-label="What do we now know?"]', "the limit belongs on the note")
        await pg.fill('textarea[aria-label="What was wrong with the version it replaces?"]', "the note did not state the limit")
        await pg.fill('textarea[aria-label="What does this version give?"]', "the limit is read first")
        await pg.fill('textarea[aria-label="What changed in the plan?"]', "notes state their limits")
        if (await pg.inner_text("#status")).strip() != "2 changes":
            fails.append("node form: two edits read %r, not '2 changes'" % (await pg.inner_text("#status")).strip())
        if await issues(pg):
            fails.append("node form: a complete change still lists problems: %s" % await issues(pg))
        async with pg.expect_download() as dl:
            await pg.click("#save")
        saved = os.path.join(tmp, "saved.request.html")
        await (await dl.value).save_as(saved)
        # the depth switch (adcs-explain/1, E03): Expert hides the tutorial, Learn hides the answer to Rebuild it until asked
        await pg.click('[data-depth-switch] button[data-d="expert"]')
        if await pg.locator("#start").is_visible():
            fails.append("node form: Expert still shows the Start here tutorial")
        await pg.click('[data-depth-switch] button[data-d="learn"]')
        rb = pg.locator('[data-explain="rebuild"] [data-show="learn"]')
        if await rb.locator(".xk-reveal").is_visible():
            fails.append("node form: Learn shows the rebuilt answer before it is asked for")
        await rb.locator("input").fill("0.75")
        await rb.locator("button").click()
        if "within the tolerance" not in await rb.inner_text():
            fails.append("node form: Rebuild it does not judge a right answer")
        await pg.click('[data-depth-switch] button[data-d="read"]')
        # unsaved answers come back after a reload
        await pg.fill('input[aria-label="Team"]', "GNC")
        await pg.reload()
        await pg.wait_for_timeout(200)
        if not await pg.locator("#draft.on").count():
            fails.append("node form: unsaved answers were not offered back after a reload")
        pg2 = await page_of(saved)
        if (await pg2.inner_text("#status")).strip() != "2 changes":
            fails.append("node form: the saved copy does not re-open with its 2 changes")
        rc, F, info, _ = intake.run_check(saved, False, os.path.join(tmp, "intake"), quiet=True, plan=plan)
        if rc:
            fails.append("intake: the saved request is refused: %s" % ["%s %s" % (f.code, f.text) for f in F if f.level == "error"])
        if sorted(c[0] for c in info.get("changes", [])) != ["note", "output.upper"]:
            fails.append("intake: the changes read back are %s" % [c[0] for c in info.get("changes", [])])
        rid = intake.request_id(json.loads(json.dumps(intake.read_form(saved)[0])))
        if not os.path.exists(os.path.join(tmp, "intake", rid, "brief.md")):
            fails.append("intake: no brief was written for a passing request")
        # an assistant's name is flagged on the page, and refused by the checker
        await pg2.fill('input[aria-label="Your name"]', "Claude")
        if not any("person" in x for x in await issues(pg2)):
            fails.append("node form: an assistant's name as requester is not flagged")
        async with pg2.expect_download() as dl:
            await pg2.click("#save")
        bad = os.path.join(tmp, "bad.request.html")
        await (await dl.value).save_as(bad)
        rc, F, _, _ = intake.run_check(bad, False, None, quiet=True, plan=plan)
        if rc == 0 or "P01" not in {f.code for f in F}:
            fails.append("intake: an assistant's name as requester was not refused by P01")
        # the developer's reply shows in the returned file
        back = os.path.join(tmp, "returned.request.html")
        intake.reply(saved, "released", "in 1.1.0", "1.1.0", None, back)
        pg3 = await page_of(back)
        if "released" not in (await pg3.inner_text("#reqchip")) or "in 1.1.0" not in await pg3.inner_text("#status-card"):
            fails.append("node form: the developer's reply does not show in the returned file")

        # a new-node request
        pn = await page_of(new)
        await pn.select_option('select[aria-label="Group"]', plan.node_id("gf"))
        await pn.fill('input[aria-label="Label"]', "Ring Reynolds number")
        idv = await pn.input_value('input[aria-label="Proposed id"]')
        if idv != "sys_fluid_momentum_rings_ring_reynolds_number":
            fails.append("new node: the id filled from group and label is %r" % idv)

        # ---------------------------------------------------------- case editor
        pc = await page_of(case)
        await pc.fill('input[aria-label="orbit.alt value"]', "450")
        if (await pc.inner_text("#state")).strip() != "format OK":
            fails.append("case editor: a valid edit reads %r" % (await pc.inner_text("#state")).strip())
        async with pc.expect_download() as dl:
            await pc.click("#dl")
        csv_path = os.path.join(tmp, "edited.csv")
        await (await dl.value).save_as(csv_path)
        badcsv = os.path.join(tmp, "wrong_unit.csv")
        open(badcsv, "w").write(open(os.path.join(ROOT, "plan", "cases", "ais_3u.csv")).read().replace("orbit.alt,Altitude,Kilometre", "orbit.alt,Altitude,Metre"))
        dialogs.clear()
        await pc.set_input_files('input[aria-label="Open a case CSV"]', badcsv)
        await pc.wait_for_timeout(300)
        if not any("orbit.alt" in d and "Kilometre" in d for d in dialogs):
            fails.append("case editor: a CSV with a wrong unit was not refused with a message (%s)" % dialogs)

        # ---------------------------------------------------------- library
        pl = await page_of(lib)
        n = await pl.locator("ul.nodes li").count()
        if n != n_nodes:
            fails.append("library: lists %d nodes, exported %d" % (n, n_nodes))
        await pl.fill("#q", "spin-down")
        vis = await pl.locator("ul.nodes li:not(.hidden)").count()
        if not 1 <= vis < n:
            fails.append("library: search for 'spin-down' shows %d of %d" % (vis, n))

        # ---------------------------------------------------------- phone width
        for path in (node, case):
            ph = await page_of(path, 390)
            sw = await ph.evaluate("document.documentElement.scrollWidth")
            if sw > 392:
                fails.append("%s: %d px wide at a 390 px phone width" % (os.path.basename(path), sw))

        # ---------------------------------------------------------- results
        import results
        rdir = os.path.join(ROOT, "results", "examples")
        mc = os.path.join(rdir, "demo_detumble_ais_3u_mc20.result.html")
        nom = os.path.join(rdir, "demo_detumble_ais_3u_nominal.result.html")
        if not (os.path.exists(mc) and os.path.exists(nom)):
            results.demo(rdir)
        pr = await page_of(mc)
        await pr.wait_for_timeout(1500)
        n_canvas = len(await pr.query_selector_all("canvas"))
        if n_canvas < 6:
            fails.append("results: drew %d charts, expected at least 6" % n_canvas)
        await pr.set_input_files("#file", nom)
        await pr.wait_for_timeout(1500)
        if "compared" not in await pr.inner_text("main"):
            fails.append("results: opening a second result did not add the comparison")
        await b.close()
    # a store: each result filed beside the case it ran, nothing run
    store = os.path.join(tmp, "store")
    done = results.store_import([mc, nom], store)
    cases = sorted(os.path.relpath(c, store) for _, c in done)
    if len(set(cases)) != 1 or not cases[0].startswith(os.path.join("cases", "ais_3u")):
        fails.append("results: import did not file both results beside ais_3u's one case: %s" % cases)
    if any(os.path.dirname(os.path.relpath(d, store)) != os.path.join("results", "ais_3u", os.path.basename(cases[0])[:-4]) for d, _ in done):
        fails.append("results: a result was not filed in its case's folder")
    if len(open(os.path.join(store, "index.csv")).read().splitlines()) != 3:
        fails.append("results: the store's index does not list the two results")
    if errs:
        fails.append("script errors: %s" % errs)
    out = subprocess.run([sys.executable, os.path.join(ROOT, "tools", "check_case.py"), csv_path], capture_output=True, text=True)
    if out.returncode:
        fails.append("case editor: the downloaded CSV fails the case check:\n" + out.stdout)
    orig = open(os.path.join(ROOT, "plan", "cases", "ais_img_3u.csv"), encoding="utf-8").read().splitlines()
    text = open(csv_path, encoding="utf-8").read().splitlines()
    diff = [(a, c) for a, c in zip(orig, text) if a != c]
    if len(orig) != len(text) or len(diff) != 1 or not diff[0][1].startswith("orbit,orbit.alt,Altitude,Kilometre,450,"):
        fails.append("case editor: the CSV differs from the reference in %d rows, not only orbit.alt: %s" % (len(diff), diff[:3]))
    return fails


def main():
    if sys.argv[1:]:
        print(__doc__.split("\n\n")[1])
        return 2
    with tempfile.TemporaryDirectory() as tmp:
        fails = asyncio.run(run(tmp))
    for x in fails:
        print("FAIL", x)
    print("form browser check: %s" % ("ok" if not fails else "%d failure(s)" % len(fails)))
    return 1 if fails else 0


if __name__ == "__main__":
    sys.exit(main())
