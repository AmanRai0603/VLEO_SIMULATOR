#!/usr/bin/env python3
"""The input control, driven the way a person drives it.

    tools/input_check.py              needs a daemon and a browser
    tools/input_check.py --selftest   needs neither

`tools/panel_check.py` checks that a figure renders, moves and matches. This
checks the other interactive thing in the face: the field on a declared row,
and the three runs behind it — the row alone, every active branch it is in,
then everything.

IT CLICKS RATHER THAN CALLS. Every defect this has caught so far was invisible
to the engine and to a source reading:

  * a control that repainted itself from inside its own change event, which
    destroys the element the event is travelling through;
  * an update button that re-rendered its own container when the field blurred,
    so the click reached nothing;
  * a refusal message carrying a 28-digit value, which under `white-space:
    nowrap` stretched the table past the viewport and pushed the answer column
    off the right-hand edge — every branch appeared to have no answer while the
    data was correct and only the layout was wrong;
  * a branch list that kept reading `{row: {id}}` after the rule moved into the
    engine, which sends `{id}`. One undefined property, and stage two produced
    nothing at all.

The last two are why this asserts on geometry and fails fast on a page error
rather than waiting for a selector. A test that hangs for ten minutes when the
page has already thrown is a test that reports nothing.
"""

import argparse
import os
import sys

# A row that is declared, published, has room between its bounds, and is read
# by enough of the tree that stage two has something to say. Mission duration
# is the one the whole design hangs off.
ROW = "sys_mission_requirements_mission_duration"
DECLARED, TRIED = "5", "7"
HOST = os.environ.get("VLEO_DAEMON", "http://localhost:7777")


def chromium_path():
    """The browser panel_check already found. One rule, one place."""
    sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
    from panel_check import chromium_path as p
    return p()


def check():
    from playwright.sync_api import sync_playwright

    bad, errs = [], []
    with sync_playwright() as pw:
        b = pw.chromium.launch(executable_path=chromium_path())
        pg = b.new_page(viewport={"width": 1400, "height": 1000})
        pg.on("pageerror", lambda e: errs.append("threw: %s" % str(e)[:160]))
        pg.on("console",
              lambda m: errs.append("console: %s" % m.text[:160]) if m.type == "error" else None)
        pg.goto(HOST + "/", wait_until="networkidle")
        pg.evaluate("id => window.openNode(id)", ROW)

        try:
            pg.wait_for_selector(".ovr-v", timeout=120000)
        except Exception as e:
            b.close()
            print("  FAIL no input control on %s: %s" % (ROW, str(e)[:90]))
            for x in errs:
                print("       %s" % x)
            return 1

        shown = pg.input_value(".ovr-v")
        if shown != DECLARED:
            bad.append("the field opens at %r, not the declared %r" % (shown, DECLARED))
        if DECLARED not in pg.inner_text(".ovr-dec span"):
            bad.append("the declared value is not shown beside the field")

        # The real path: into the field, type, click update. Not a tab-out
        # first — the blur is what broke this twice.
        pg.click(".ovr-v")
        pg.fill(".ovr-v", TRIED)
        pg.click(".ovr-go")

        # Fail on a throw rather than waiting out the timeout.
        for _ in range(400):
            if errs:
                break
            if pg.query_selector(".ovr-stage:last-of-type .chainline"):
                break
            pg.wait_for_timeout(1000)
        if errs:
            b.close()
            print("  FAIL the page threw while running the three stages")
            for x in errs[:4]:
                print("       %s" % x)
            return 1

        stages = pg.query_selector_all(".ovr-stage")
        if len(stages) != 3:
            bad.append("expected three stages, got %d" % len(stages))
        if "T_mis_req" not in pg.inner_text(".ovr-stage .answer"):
            bad.append("stage one did not report the row's own answer")

        rows = pg.query_selector_all(".ovr-bt tbody tr")
        if len(rows) < 2:
            bad.append("stage two listed %d branches" % len(rows))

        # GEOMETRY, NOT JUST CONTENT. The answer column has to be on screen.
        over = pg.evaluate("""() => {
            const t = document.querySelector('.ovr-bt');
            if (!t) return null;
            let hidden = 0;
            for (const tr of t.querySelectorAll('tbody tr')) {
                const td = tr.lastElementChild;
                if (!td.innerText.trim()) { hidden++; continue; }
                if (td.getBoundingClientRect().right >
                    document.documentElement.clientWidth) hidden++;
            }
            return { w: t.getBoundingClientRect().width, sw: t.scrollWidth, hidden };
        }""")
        if not over:
            bad.append("stage two drew no table at all")
        else:
            if over["sw"] > over["w"] + 1:
                bad.append("the branch table overflows: %d px of content in %d px"
                           % (over["sw"], over["w"]))
            if over["hidden"]:
                bad.append("%d branch row(s) have an empty or off-screen answer"
                           % over["hidden"])

        if "rows moved" not in pg.inner_text(".ovr-stage:last-of-type"):
            bad.append("stage three did not report what moved")

        # A CLEARED BOX MUST NOT SIT OVER A LIVE WHAT-IF. Clearing the field
        # cannot clear the override — there is nothing to set it to — so the
        # engine kept being sent seven years while the box read empty.
        pg.fill(".ovr-v", "")
        pg.click("body")
        pg.wait_for_timeout(400)
        if pg.input_value(".ovr-v").strip() == "":
            bad.append("a cleared field is left sitting over a live override")

        # The bar may not claim more than is true: a figure drawn from the
        # record does not move under an override, and saying otherwise tells a
        # reader a chart reflects their change when it does not.
        barred = pg.inner_text("#ovrbar") if pg.query_selector("#ovrbar:not([hidden])") else ""
        if "Everything below is a what-if" in barred:
            bad.append("the what-if bar claims the record moves with an override")

        # Out of range is refused, with the reason off the sheet.
        pg.fill(".ovr-v", "99")
        pg.wait_for_timeout(400)
        why = pg.inner_text(".ovr-why")
        if "refused" not in why:
            bad.append("99 yr was not refused (the sheet bounds it at 15)")

        # A STORED OVERRIDE THAT CANNOT BE USED MUST NOT COME BACK. `commit`
        # refuses an out-of-range value, so one is only ever stored by being
        # valid at the time and the sheet narrowing afterwards. Kept, it made
        # the engine refuse EVERY run anywhere in the tool, with nothing on the
        # field saying why, and it survived a reload.
        pg.evaluate("p => localStorage.setItem('vleo.overrides.v1', p)",
                    '{"%s": 3155760000}' % ROW)     # 100 yr, bounded at 15
        pg.reload(wait_until="networkidle")
        pg.wait_for_timeout(1500)
        kept = pg.evaluate("async()=>{const s=await import('/js/state.js');return s.S.overrides.size}")
        if kept:
            bad.append("an out-of-range value survived a reload and will refuse every run")
        chips = len(pg.query_selector_all(".ovr-chip"))
        if chips != kept:
            bad.append("the bar counts %d override(s) but draws %d chip(s)" % (kept, chips))
        pg.evaluate("() => localStorage.clear()")
        pg.reload(wait_until="networkidle")
        pg.wait_for_selector(".ovr-v", timeout=120000) if pg.query_selector(".ovr-v") else None
        pg.evaluate("id => window.openNode(id)", ROW)
        pg.wait_for_selector(".ovr-v", timeout=120000)
        pg.fill(".ovr-v", TRIED)
        pg.wait_for_timeout(300)

        # And reset puts it back.
        pg.click(".ovr-reset")
        pg.wait_for_timeout(400)
        if pg.input_value(".ovr-v") != DECLARED:
            bad.append("reset left the field at %r" % pg.input_value(".ovr-v"))
        if pg.query_selector(".ovr.on"):
            bad.append("the row is still marked a what-if after reset")
        b.close()

    bad += errs
    for x in bad:
        print("  FAIL %s" % x)
    print("input control: %s" % ("as expected" if not bad else "%d FAILED" % len(bad)))
    return 1 if bad else 0


def selftest():
    """The parts that need no browser: that the checks are the right shape."""
    bad = 0
    src = open(os.path.abspath(__file__), encoding="utf-8").read()

    # Each of the four defects in the docstring has an assertion behind it.
    for probe, why in (
        (".ovr-stage:last-of-type .chainline", "it waits for stage three to land"),
        ("scrollWidth", "it measures the table's overflow"),
        ("clientWidth", "it measures whether a cell is on screen"),
        ("pageerror", "it listens for a page error"),
        ('if errs:', "it stops on a page error instead of waiting"),
        ('.ovr-reset', "it checks reset puts the declared value back"),
        ('vleo.overrides.v1', "it checks a poisoned store is dropped"),
    ):
        if probe not in src:
            bad += 1
            print("  FAIL the check no longer proves %s (%r is gone)" % (why, probe))

    # A retired row is a question whose answer nothing should read any more, so
    # it must not get a live-looking control. The face once tested `state !==
    # 'empty'`, which let a deprecated row have a field while `/v1/branches` and
    # tools/branch_audit.py both counted only published rows — three statements
    # of one idea, and the face was the one that differed.
    face = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
                        "web", "js", "inputs.js")
    try:
        js = open(face, encoding="utf-8").read()
        if "state === 'published'" not in js:
            bad += 1
            print("  FAIL inputs.js no longer restricts the field to published rows")
    except OSError as e:
        bad += 1
        print("  FAIL could not read inputs.js: %s" % e)

    # THE WORKFLOW STILL RUNS THIS, AND NO STEP HAS BEEN SWALLOWED.
    #
    # Adding these two steps orphaned the line below them: `python3
    # tools/mat_parity.py` belonged to the daemon step's `run: |` block, ended
    # up after the new steps, and YAML folded it into one of them as
    # "branch_audit.py --sub solar python3 tools/mat_parity.py". The file still
    # PARSED — it is valid YAML — so a yaml.safe_load check said nothing, and
    # the effect was that the MATLAB .mat comparison silently stopped running.
    # A check that is quietly not running is worse than one that fails.
    wf = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
                      ".github", "workflows", "gate.yml")
    try:
        import yaml
        jobs = yaml.safe_load(open(wf, encoding="utf-8"))["jobs"]
        runs = [str(st.get("run", "")) for j in jobs.values() for st in j.get("steps", [])]
        for want in ("tools/input_check.py", "tools/branch_audit.py",
                     "tools/mat_parity.py", "tools/matlab_parity.py",
                     "tools/panel_check.py"):
            if not any(want in r for r in runs):
                bad += 1
                print("  FAIL the workflow no longer runs %s" % want)
        # A one-line `run:` naming two interpreters is the signature of a block
        # line that got folded into the step above it.
        for r in runs:
            if "\n" not in r.strip() and r.count("python3 ") > 1:
                bad += 1
                print("  FAIL a workflow step folded two commands onto one line: %r"
                      % r.strip()[:90])
    except ImportError:
        pass          # no pyyaml here; CI has it
    except (OSError, KeyError, TypeError) as e:
        bad += 1
        print("  FAIL could not read the workflow: %s" % e)

    # The row it drives has to be one the face would offer a field for, or the
    # whole check is vacuous. That rule is inputs.js's; this is its shape.
    if not ROW or "." in ROW:
        bad += 1
        print("  FAIL the row under test is not a node id: %r" % ROW)
    if DECLARED == TRIED:
        bad += 1
        print("  FAIL the value tried equals the declared one, so nothing changes")

    print("selftest: %d cases, %s" % (12, "all as expected" if not bad else "%d FAILED" % bad))
    return 1 if bad else 0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--selftest", action="store_true")
    a = ap.parse_args()
    if a.selftest:
        return selftest()
    try:
        return check()
    except Exception as e:
        print("could not drive the face: %s" % str(e)[:200])
        print("a daemon must be running: cargo run --release -p vleo-daemon")
        return 1


if __name__ == "__main__":
    sys.exit(main())
