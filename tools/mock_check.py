#!/usr/bin/env python3
"""The page works on the mock engine — no Rust built, no engine running.

The frontend is developed against the contract, not against a build of the
engine (contract/README.md). This holds that promise: it starts
tools/mock_engine.py, opens the page in a real browser, and walks what a
frontend developer would — the tree, a node's page, a run, the Results page,
Explore —
failing on any page error, on a banner that does not say the numbers are
recorded examples, and on a run panel that does not show the recorded answer.

    python3 tools/mock_check.py              the check (a browser: Playwright)
    python3 tools/mock_check.py --selftest   the mock engine's own selftest

Exit 0 when every step held; 1 with the steps that did not.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import mock_engine  # noqa: E402


def chromium_path():
    """The browser panel_check already found. One rule, one place."""
    from panel_check import chromium_path as p
    return p()


def check():
    from playwright.sync_api import sync_playwright

    srv, port = mock_engine.serve(7790, background=True)
    url = f"http://127.0.0.1:{port}"
    fails, errors = [], []

    def step(what, ok, detail=""):
        print(f"  {'ok  ' if ok else 'FAIL'} {what}{' — ' + detail if detail else ''}")
        if not ok:
            fails.append(what)

    with sync_playwright() as pw:
        b = pw.chromium.launch(executable_path=chromium_path())
        p = b.new_page(viewport={"width": 1300, "height": 950})
        p.on("pageerror", lambda e: errors.append(str(e)))
        p.on("console", lambda m: errors.append(m.text) if m.type == "error" else None)
        p.goto(url + "/")
        p.wait_for_load_state("networkidle")
        bar = p.locator("#mock-bar")
        step("the page says its answers are recorded examples", bar.count() == 1,
             bar.inner_text()[:60] if bar.count() else "no #mock-bar")
        step("no contract mismatch is reported", p.locator("#contract-bar").count() == 0)
        p.click('button.tab[data-layer="3"]')
        p.wait_for_timeout(300)
        cells = p.locator("#matrix-cells .mcell").count()
        step("the tree is drawn from the recorded index", cells > 20, f"{cells} cells on layer 3")

        p.click('button.tab[data-layer="4"]')
        p.select_option(".run-target", "sw_ap_design")
        p.wait_for_timeout(300)
        p.click(".run-go")
        p.wait_for_selector(".run-out .answer", timeout=15000)
        ans = p.inner_text(".run-out .answer")
        step("the run panel shows the recorded answer", "Ap" in ans, ans)

        p.click("#results-tab")
        p.wait_for_selector(".res-list, .empty", timeout=15000)
        rows = p.locator(".res-row").count()
        step("the Results page lists the recorded results", rows >= 1, f"{rows} row(s)")

        # Every kind of figure the engine can describe, drawn from its samples.
        p.locator("#tabs .tab", has_text="Architecture").first.click()
        p.click('[data-arch="figures"]')
        p.wait_for_selector(".fig-sample canvas", timeout=15000)
        kinds = p.eval_on_selector_all(".fig-sample", "s => s.map(x => x.dataset.kind)")
        drawn = p.locator(".fig-sample canvas").count()
        step("every kind of figure is drawn from its sample", drawn == 6 and "scene3d" in kinds,
             f"{drawn} drawn: {', '.join(kinds)}")

        # Explore: which design, where it breaks, a closure traced, two files
        # compared, each drawn from the engine's recorded answers.
        p.click("#explore-tab")
        p.wait_for_selector("#explore-body .answer-first", timeout=15000)
        opening = p.inner_text("#explore-body .answer-first")
        step("Explore says which design it shows", "own files" in opening, opening[:80])
        shut = p.locator('#explore-body .ex-grp[aria-expanded="false"]')
        before = p.locator("#explore-body .ex-grp").count()
        if shut.count():
            shut.first.click()
            p.wait_for_timeout(200)
        after = p.locator("#explore-body .ex-grp").count()
        step("Explore draws the health map, and a branch opens on a click", 0 < before < after,
             f"{before} group(s), then {after}")
        p.locator("#explore-body .ex-trace").first.click()
        p.wait_for_selector("#explore-body .ex-trace-out h4", timeout=15000)
        traced = p.inner_text("#explore-body .ex-trace-out")
        step("a closure is traced", "over its open values" in traced, traced[:60])
        p.fill("#explore-body .ex-a", "design:groups/l3_solar/nodes/sw_ap_design.vnode")
        p.fill("#explore-body .ex-b", "design:groups/l3_solar/nodes/sw_ap_mean_band_spread.vnode")
        p.click("#explore-body .ex-compare-go")
        p.wait_for_selector("#explore-body .ex-compare-out b", timeout=15000)
        compared = p.inner_text("#explore-body .ex-compare-out")
        step("two files are compared", "block(s)" in compared, compared[:60])

        step("no page error", not errors, "; ".join(errors[:3]))

        # An engine from another release: the page must say so, not misread.
        import json as _json

        def other(route):
            data = _json.loads(route.fetch().text())
            data["contract"] = "2"
            route.fulfill(status=200, content_type="application/json", body=_json.dumps(data))

        q = b.new_page()
        q.route("**/v1/version", other)
        q.goto(url + "/")
        q.wait_for_load_state("networkidle")
        mism = q.locator("#contract-bar")
        step("an engine speaking another contract is named", mism.count() == 1 and "contract 2" in mism.inner_text(),
             mism.inner_text()[:80] if mism.count() else "no #contract-bar")
        b.close()
    srv.shutdown()
    if fails:
        print(f"mock_check: {len(fails)} step(s) did not hold")
        return 1
    print("mock_check: the page works on the mock engine")
    return 0


if __name__ == "__main__":
    if "--selftest" in sys.argv[1:]:
        sys.exit(mock_engine.selftest())
    sys.exit(check())
