#!/usr/bin/env python3
"""The three checks a declared panel has to pass.

    tools/panel_check.py                 # every panel in panels/
    tools/panel_check.py --panel tree
    tools/panel_check.py --record        # store reference images
    tools/panel_check.py --selftest

A wrong number crashes a test. A wrong chart looks beautiful — and the measured
record of this codebase says the visual layer is where the defects actually
live. The engine has generated verification; the visual layer had a prohibition
and a reviewer, which is backwards relative to the evidence.

So a panel gets a spec like a node does, and three checks that need no person:

  1 · it renders   the mount is not empty, not zero-sized, and not in a failed
                   state
  2 · it moves     change each input it declares it reads, and its content must
                   change — WITHOUT failing. A panel wired to nothing passes
                   every other check, including a screenshot comparison, because
                   it draws the same correct picture whatever the data says
  2b · it reads    for every row the panel declares in `engine`, serve that row
                   a different answer and the picture must change. A panel that
                   asks the engine and then ignores the reply fails here
  3 · it matches   against a stored reference, within tolerance

Check two is the one worth having. "Three correct power numbers on one screen,
correct at three different times" is a panel that renders, matches yesterday's
reference, and is reading state nobody refreshed.

AND CHECK TWO HAD A HOLE, WHICH 2b AND THE FAILED-STATE TEST CLOSE. It asked
whether the canvas changed. A failed render blanks the canvas, so the signature
changes and "it moves" was satisfied by the panel BREAKING — a deliberately
broken panel passed. The face now marks its mount `data-failed` when a render
throws, and every check here refuses a mount carrying it.

2b exists because declaring a row is not reading it. The panel can fetch the
value and draw a literal anyway, which is exactly the state four of `design`'s
numbers were in. Rewriting the engine's reply in the browser and demanding the
picture move is the only way to tell the two apart, and it needs no backdoor in
the product: the interception is the checker's, not the page's.

This drives the real page in a real browser against the real daemon. A check
that runs against a mock proves the mock works.
"""

import argparse
import io
import json
import os
import socket
import subprocess
import sys
import time
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PANELS = ROOT / "panels"
REFERENCE = PANELS / "reference"


def specs():
    return sorted(p for p in PANELS.glob("*.toml"))


def spec(path):
    d = tomllib.loads(path.read_text())
    for need in ("id", "label", "module", "mount", "draws", "correct"):
        if not d.get(need):
            raise SystemExit("%s has no %s — see panels/README.md" % (path.name, need))
    if not (ROOT / d["module"]).is_file():
        raise SystemExit("%s names %s, which does not exist" % (path.name, d["module"]))
    return d


class Server:
    """The real daemon, on a port nobody else is using.

    Not a static file server. The page asks the engine for the index and the
    version before it draws anything, so a static server gets a blank page and
    every panel fails check one for a reason that is not about the panel. And a
    check run against a mock proves the mock works.

    A fresh port each time, because the failure people actually hit is a stale
    daemon from an earlier session serving an older tree: it is internally
    consistent, so nothing looks wrong.
    """

    def __init__(self, root):
        self.root = root
        self.proc = None
        with socket.socket() as s:
            s.bind(("127.0.0.1", 0))
            self.port = s.getsockname()[1]

    def __enter__(self):
        env = dict(os.environ, VLEO_PORT=str(self.port))
        self.proc = subprocess.Popen(
            ["cargo", "run", "-q", "--release", "-p", "vleo-daemon"],
            cwd=self.root, env=env,
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
        )
        url = "http://127.0.0.1:%d/" % self.port
        for _ in range(600):
            if self.proc.poll() is not None:
                raise SystemExit("the daemon exited before it served anything")
            try:
                with socket.create_connection(("127.0.0.1", self.port), timeout=0.5):
                    return url
            except OSError:
                time.sleep(0.25)
        raise SystemExit("the daemon did not come up on %d" % self.port)

    def __exit__(self, *a):
        if self.proc:
            self.proc.terminate()
            try:
                self.proc.wait(timeout=10)
            except subprocess.TimeoutExpired:
                self.proc.kill()


def chromium_path():
    """The browser that is actually installed.

    The pinned Playwright package and the pre-installed browser can be
    different builds, and Playwright then hunts for a path that does not exist.
    Finding the real binary is two lines and removes a whole class of "works on
    my machine".
    """
    if os.environ.get("VLEO_CHROMIUM"):
        return os.environ["VLEO_CHROMIUM"]
    base = Path(os.environ.get("PLAYWRIGHT_BROWSERS_PATH", "/opt/pw-browsers"))
    for d in sorted(base.glob("chromium-*"), reverse=True):
        exe = d / "chrome-linux" / "chrome"
        if exe.is_file():
            return str(exe)
    return None


# A panel's content, whatever kind of element it draws into.
#
# The three panels this checker was written against all build DOM, so both
# "is it empty" and "did it change" read innerHTML. A canvas's innerHTML is
# empty by specification and never changes, so the one real chart in this
# repository — the behaviour sweep — could not be checked at all: it reported
# "still empty after the page settled" and nothing else. A canvas is compared
# on its rendered pixels instead, which is the only thing it has.
_SIG = """
sel => { const e = document.querySelector(sel); if (!e) return null;
         return e.tagName === 'CANVAS' ? e.toDataURL() : e.innerHTML; }
"""

# Did the panel's own render throw?
#
# The face marks its mount `data-failed` when build() raises, and clears it on a
# good draw. Without this, check 2 cannot tell a redraw from a collapse: both
# change the canvas signature, and a deliberately broken panel passed.
_FAILED = """
sel => { const e = document.querySelector(sel); return e ? (e.dataset.failed || '') : ''; }
"""


def _settle(page, mount, tries=20, gap=250):
    """Wait until the mount stops changing, then return its signature.

    A fixed pause before capturing the "before" picture is how 2b first failed
    to bite: the F10.7 view fetches the engine, was still settling at 600ms, and
    the completion of that first render then counted as "it changed when the
    answer moved". Two successive identical reads is the condition actually
    wanted, and it is quick when the panel is quick.
    """
    last = None
    for _ in range(tries):
        now = page.evaluate(_SIG, mount)
        if now is not None and now == last:
            return now
        last = now
        page.wait_for_timeout(gap)
    return last


def _intercept_run(page, row_id):
    """Serve one row a different answer, and report whether it was ever asked.

    The engine's reply is passed through untouched except for the one row, whose
    SI value is moved far enough that no rounding could hide it. Everything else
    on the path is left alone, so a panel reading several rows only sees the one
    it is being tested on move.

    `hit` separates the two ways a panel can fail 2b: it never asked, or it asked
    and ignored the answer. Those are different defects and the message says
    which.
    """
    state = {"hit": False}

    def bend(x):
        # Big enough that no axis rounding absorbs it, and away from zero so a
        # bound near zero is not crossed by accident.
        return x * 1.75 + 13.0

    def handler(route):
        req = route.request
        if ("node=" + row_id) not in req.url:
            route.continue_()
            return
        state["hit"] = True
        try:
            resp = route.fetch()
            body = resp.json()
        except Exception:
            route.continue_()
            return
        # A run carries the value; a sweep carries the whole relation. A panel
        # can read a row either way — `design` sweeps sw_storm_return_level for
        # its curve and runs l3_solar_req_03 for its line — so both are bent,
        # or 2b would be blind to exactly the rows a figure draws as a shape.
        for v in body.get("values", []):
            if v.get("id") == row_id and isinstance(v.get("si"), (int, float)):
                v["si"] = bend(v["si"])
                v["shown"] = "%g" % v["si"]
        if body.get("y_id") == row_id and isinstance(body.get("y"), list):
            body["y"] = [bend(v) if isinstance(v, (int, float)) else v for v in body["y"]]
        route.fulfill(status=200, content_type="application/json", body=json.dumps(body))

    page.route("**/v1/run**", handler)
    page.route("**/v1/sweep**", handler)
    return state


def _failed(page, mount):
    try:
        return page.evaluate(_FAILED, mount)
    except Exception:
        return ""


# Non-empty, for either kind. A blank canvas is not "empty" in any DOM sense,
# so it is compared against a fresh canvas of the same size: equal means
# nothing has been drawn. Exact, and it needs no threshold.
_NONBLANK = """
sel => { const e = document.querySelector(sel); if (!e) return false;
         if (e.tagName !== 'CANVAS') return e.innerHTML.trim().length > 0;
         const b = document.createElement('canvas');
         b.width = e.width; b.height = e.height;
         return e.toDataURL() !== b.toDataURL(); }
"""


def check_all(ids=None, record=False):
    from playwright.sync_api import sync_playwright

    found = []
    with Server(ROOT) as url, sync_playwright() as pw:
        browser = pw.chromium.launch(executable_path=chromium_path())
        for path in specs():
            d = spec(path)
            if ids and d["id"] not in ids:
                continue
            # Tall enough that a long pane is captured in one shot. Element
            # screenshots of a transparent pane taller than the viewport are
            # stitched, and the stitching composites whatever the page had
            # scrolled behind it — a reference that then changes when something
            # unrelated moves.
            page = browser.new_page(
                viewport={"width": d.get("width", 1440), "height": d.get("height", 1600)}
            )
            errors = []
            page.on("pageerror", lambda e: errors.append(str(e)))
            page.goto(url, wait_until="networkidle")
            mount = d["mount"]
            # Some panels do not draw until somebody asks. The sweep is the
            # example: its canvas is created hidden and stays blank until a
            # sweep has been run, so checking it at the state the page opens in
            # would report a defect that is the panel working as designed.
            # `ready` is evaluated before the checks, never during them.
            if d.get("ready"):
                try:
                    page.evaluate(d["ready"])
                except Exception as e:
                    found.append((d["id"], "1 renders", "could not reach the ready state: %s" % e))
                    page.close()
                    continue
            try:
                page.wait_for_selector(mount, timeout=15000, state="attached")
                page.wait_for_function(_NONBLANK, arg=mount, timeout=15000)
            except Exception:
                found.append((d["id"], "1 renders", "%s is still empty after the page settled" % mount))
                page.close()
                continue

            # 1 · it renders
            box = page.locator(mount).bounding_box()
            if not box or box["width"] < 1 or box["height"] < 1:
                found.append((d["id"], "1 renders", "%s occupies no space" % mount))
            if errors:
                found.append((d["id"], "1 renders", "the page threw: %s" % errors[0]))
            fail = _failed(page, mount)
            if fail:
                found.append((d["id"], "1 renders", "the panel reports a failed render: %s" % fail))

            # 2 · it moves
            settle = d.get("settle_ms", 3000)
            for inp in d.get("input", []):
                before = page.evaluate(_SIG, mount)
                try:
                    page.evaluate(inp["drive"])
                except Exception as e:
                    found.append((d["id"], "2 moves", "could not drive %r: %s" % (inp["name"], e)))
                    continue
                # Wait for it to CHANGE rather than sleeping and hoping. A fixed
                # pause has to be long enough for the slowest panel, and a panel
                # that redraws behind a fetch — the sweep does — is slower than
                # any pause anyone would write. Waiting on the condition is
                # faster when it is quick and correct when it is not; a panel
                # wired to nothing still fails, it just fails after the timeout.
                try:
                    page.wait_for_function(
                        "([sel, was]) => { const e = document.querySelector(sel); if (!e) return false;"
                        " const now = e.tagName === 'CANVAS' ? e.toDataURL() : e.innerHTML;"
                        " return now !== was; }",
                        arg=[mount, before],
                        timeout=settle,
                    )
                except Exception:
                    found.append((
                        d["id"], "2 moves",
                        "%s did not change when %r moved — %s"
                        % (mount, inp["name"], inp.get("why", "the panel claims to read it")),
                    ))
                # A render that threw blanks the canvas, and a blank canvas has a
                # different signature from a drawn one — so the wait above is
                # satisfied by the panel breaking. Changed is not the same as
                # moved, and this is where the two are told apart.
                #
                # SETTLE FIRST. The wait returns the instant the canvas differs,
                # and it differs as soon as the redraw CLEARS it — before the
                # render has finished and had the chance to record a failure.
                # Reading the flag there caught nothing and the check silently
                # went back to being the one it replaced.
                _settle(page, mount)
                fail = _failed(page, mount)
                if fail:
                    found.append((
                        d["id"], "2 moves",
                        "%s FAILED when %r moved rather than redrawing: %s"
                        % (mount, inp["name"], fail),
                    ))
                page.goto(url, wait_until="networkidle")
                if d.get("ready"):
                    try:
                        page.evaluate(d["ready"])
                    except Exception:
                        pass
                page.wait_for_selector(mount, state="attached")
                try:
                    page.wait_for_function(_NONBLANK, arg=mount, timeout=settle)
                except Exception:
                    pass

            # 2b · it reads what it says it reads
            #
            # A panel declares `engine = [...]`: the rows it takes values from
            # rather than the rows it argues about. Declaring is not reading —
            # a panel can fetch a value and go on drawing a literal, which is
            # exactly where four of `design`'s numbers were. So the engine's
            # reply is rewritten in the browser, the panel redrawn, and the
            # picture must move.
            #
            # The interception is the checker's. Nothing in the product knows
            # this is happening, which is the point: a test hook in the page
            # would be a thing to keep working rather than a thing that checks.
            def _open(settle_ms, state=None):
                page.goto(url, wait_until="networkidle")
                if d.get("ready"):
                    try:
                        page.evaluate(d["ready"])
                    except Exception:
                        return None
                try:
                    page.wait_for_selector(mount, state="attached")
                    page.wait_for_function(_NONBLANK, arg=mount, timeout=settle_ms)
                except Exception:
                    return None
                if state:
                    try:
                        page.evaluate(state)
                    except Exception:
                        return None
                return _settle(page, mount)

            # A row may only be read on one branch of the panel, so each entry
            # may carry the state that reaches it. A bare string uses the
            # panel's own `engine_state`; a table names its own. Without this,
            # testing an Ap requirement while the panel is on its F10.7 branch
            # reports a failure that is the fixture's, not the panel's.
            for _e in d.get("engine", []):
                if isinstance(_e, dict):
                    rid, _state = _e["row"], _e.get("state", d.get("engine_state"))
                else:
                    rid, _state = _e, d.get("engine_state")
                # TWO FULL RENDERS DOWN THE SAME PATH, differing only in what the
                # engine answered. An earlier form of this called a global redraw
                # hook on the open panel, which after check 2's reloads could
                # point at a DETACHED host — the redraw then drew into nothing,
                # the signature changed, and 2b passed a panel that ignores the
                # value. Rebuilding the page each time costs a second and cannot
                # go stale, and it leaves no test hook in the product.
                clean = _open(settle, _state)
                if clean is None:
                    found.append((d["id"], "2b reads",
                                  "the panel never drew, so %s could not be tested" % rid))
                    continue
                asked = _intercept_run(page, rid)
                moved = _open(settle, _state)
                fail = _failed(page, mount)
                page.unroute("**/v1/run**")
                page.unroute("**/v1/sweep**")
                if fail:
                    found.append((d["id"], "2b reads",
                                  "the panel FAILED when %s was perturbed: %s" % (rid, fail)))
                    continue
                if moved is None:
                    found.append((d["id"], "2b reads",
                                  "the panel stopped drawing when %s was perturbed" % rid))
                elif not asked["hit"]:
                    found.append((d["id"], "2b reads",
                                  "the panel never asked the engine for %s, though it "
                                  "declares it" % rid))
                elif moved == clean:
                    found.append((d["id"], "2b reads",
                                  "%s asked the engine for %s and drew the same picture when "
                                  "the answer changed — it is not using the value"
                                  % (mount, rid)))

            # 2b leaves the page wherever `engine_state` put it, and check 3
            # photographs whatever is on screen. Reload, so the reference is
            # taken from the panel's own opening state and not from the branch
            # 2b happened to need.
            if d.get("engine"):
                page.goto(url, wait_until="networkidle")
                if d.get("ready"):
                    try:
                        page.evaluate(d["ready"])
                    except Exception:
                        pass
                try:
                    page.wait_for_selector(mount, state="attached")
                    page.wait_for_function(_NONBLANK, arg=mount, timeout=settle)
                except Exception:
                    pass

            # 3 · it matches
            #
            # Taken in the state the spec names, not wherever the page happens
            # to open. A panel with nothing to draw at the default view gives a
            # reference that would still match if the panel broke — which is
            # the "snapshot of a bug" this check exists to avoid.
            if d.get("reference_state"):
                try:
                    page.evaluate(d["reference_state"])
                    page.wait_for_timeout(400)
                except Exception as e:
                    found.append((d["id"], "3 matches", "could not reach the reference state: %s" % e))
            REFERENCE.mkdir(exist_ok=True)
            ref = REFERENCE / ("%s.png" % d["id"])
            shot = page.locator(mount).screenshot()
            if record:
                ref.write_bytes(shot)
                print("  recorded %s (%d bytes)" % (ref.relative_to(ROOT), len(shot)))
            elif not ref.is_file():
                found.append((
                    d["id"], "3 matches",
                    "no reference image. `--record` stores one, and a person has to look at "
                    "the picture first — a reference nobody looked at is a snapshot of a bug",
                ))
            else:
                diff = _differ(ref.read_bytes(), shot)
                if diff > d.get("tolerance", 0.02):
                    found.append((
                        d["id"], "3 matches",
                        "%.1f%% of pixels differ from the reference; %.1f%% is the tolerance"
                        % (diff * 100, d.get("tolerance", 0.02) * 100),
                    ))
            page.close()
        browser.close()
    return found


def _differ(a, b):
    """Fraction of pixels that differ, by more than a hair.

    Pixels, not bytes. Comparing compressed PNG bytes gives a yes-or-no answer
    dressed up as a percentage — one changed pixel diverges the whole stream
    after it — and then the tolerance field means nothing. A tolerance that
    means nothing is worse than no tolerance, because somebody will tune it.

    The per-channel slack absorbs antialiasing, which moves a pixel by one or
    two levels on a redraw and is not a change anybody wants reported.
    """
    from PIL import Image

    ia = Image.open(io.BytesIO(a)).convert("RGB")
    ib = Image.open(io.BytesIO(b)).convert("RGB")
    if ia.size != ib.size:
        return 1.0
    pa, pb = ia.load(), ib.load()
    w, h = ia.size
    n = 0
    for y in range(h):
        for x in range(w):
            ca, cb = pa[x, y], pb[x, y]
            if max(abs(ca[0] - cb[0]), abs(ca[1] - cb[1]), abs(ca[2] - cb[2])) > 8:
                n += 1
    return n / float(w * h)


def _tree_sig(root):
    """A cheap signature of the working tree's web sources.

    The selftest breaks a panel by string replacement, and a replacement whose
    target has been reworded silently does nothing — the case then passes for
    the wrong reason and stops protecting anything. Comparing before and after
    turns that into a loud failure.
    """
    h = 0
    for f in sorted((root / "web" / "js").glob("*.js")):
        h = (h * 1000003 + hash(f.read_text())) & 0xFFFFFFFFFFFF
    return h


def selftest():
    """A panel wired to nothing must fail check two, and a blank one check one.

    Six cases: both failures on a DOM panel, both on a canvas panel, and the two
    only 2b and its failed-state probe can see. All
    four are built by breaking a real panel, because a checker demonstrated on a
    fixture is a checker demonstrated on a fixture — and the canvas pair exists
    because the DOM pair passed happily while a canvas panel could not be
    checked at all.
    """
    import shutil, tempfile

    global ROOT, PANELS, REFERENCE
    real_root, real_panels, real_ref = ROOT, PANELS, REFERENCE
    bad = 0
    cases = [
        ("a panel that draws nothing",
         lambda d: (d / "web" / "js" / "tree.js").write_text(
             "export function drawTree() { document.querySelector('#tree').innerHTML = ''; }\n"),
         "1 renders"),
        ("a panel wired to nothing",
         lambda d: (d / "web" / "js" / "tree.js").write_text(
             (d / "web" / "js" / "tree.js").read_text().replace(
                 "export function drawTree(disp, rel) {",
                 "let __once = null;\nexport function drawTree(disp, rel) {\n"
                 "  if (__once) { document.querySelector('#tree').innerHTML = __once; return; }\n"
                 "  const __r = __drawTree(disp, rel);\n"
                 "  __once = document.querySelector('#tree').innerHTML;\n  return __r;\n}\n"
                 "function __drawTree(disp, rel) {")),
         "2 moves"),
        # The same two failures on a CANVAS panel, because the first two cases
        # only exercise the innerHTML path. A canvas reports empty innerHTML
        # forever, so before these cases existed the checker passed a canvas
        # panel that drew nothing and a canvas panel wired to nothing — which is
        # how the behaviour sweep went unchecked while three panels were checked.
        ("a canvas panel that draws nothing",
         lambda d: (d / "web" / "js" / "run.js").write_text(
             (d / "web" / "js" / "run.js").read_text().replace(
                 "function plot(host, res) {",
                 "function plot(host, res) {\n  if (res) return;")),
         "1 renders"),
        ("a canvas panel wired to nothing",
         lambda d: (d / "web" / "js" / "run.js").write_text(
             (d / "web" / "js" / "run.js").read_text()
             .replace("const xs = res.x.map(v => v / res.x_factor);",
                      "const xs = [0, 1, 2, 3];")
             .replace("const ys = res.y.map(v => v / res.y_factor);",
                      "const ys = [0, 1, 0, 1];")),
         "2 moves"),
        # THE DEFECT 2b EXISTS FOR, and the one the other four cannot see: a
        # panel that asks the engine for a row it declares and then draws a
        # literal anyway. It renders, it moves when its controls move, and it
        # matches yesterday's reference — because it is drawing a perfectly
        # steady picture of a number nobody computed. Four of `design`'s numbers
        # were in exactly this state, one of them two revisions stale.
        ("a panel that asks the engine and ignores the answer",
         lambda d: (d / "web" / "js" / "solar.js").write_text(
             (d / "web" / "js" / "solar.js").read_text().replace(
                 "  const CENTRAL = c.si;",
                 "  const CENTRAL = 114.8437;")),
         "2b reads"),
        # And the hole 2b's failed-state probe closes: before it, a panel whose
        # render THREW passed check 2, because a failed render blanks the canvas
        # and a blank canvas has a different signature from a drawn one.
        ("a panel whose render throws",
         lambda d: (d / "web" / "js" / "solar.js").write_text(
             (d / "web" / "js" / "solar.js").read_text().replace(
                 "engine: ['sw_central_expectation',",
                 "engine: ['sw_no_such_row',")),
         "2 moves"),
    ]
    for label, break_it, want in cases:
        with tempfile.TemporaryDirectory() as tmp:
            work = Path(tmp) / "tree"
            shutil.copytree(real_root, work, ignore=shutil.ignore_patterns(".git", "target"))
            # The canvas cases need a canvas panel to break, and there is no
            # declared one: a panel in panels/ needs a reference image, and a
            # reference image needs a person to look at the picture. So the
            # spec is kept outside panels/ and installed into the throwaway
            # tree here. The selftest then exercises the canvas path without
            # the repository carrying a panel nobody has vouched for, which
            # would turn the panels check red for everybody.
            if "canvas" in label:
                shutil.copy(real_root / "tools" / "selftest_panels" / "sweep.toml",
                            work / "panels" / "sweep.toml")
            before_tree = _tree_sig(work)
            break_it(work)
            if _tree_sig(work) == before_tree:
                bad += 1
                print("  FAIL %s changed nothing — the patch has drifted from the "
                      "source it edits, so the case proves nothing" % label)
                continue
            ROOT, PANELS, REFERENCE = work, work / "panels", work / "panels" / "reference"
            try:
                which = ("sweep" if "canvas" in label
                         else "design" if "engine" in label or "throws" in label
                         else "tree")
                found = check_all(ids={which})
            finally:
                ROOT, PANELS, REFERENCE = real_root, real_panels, real_ref
            if not any(stage == want for _, stage, _ in found):
                bad += 1
                print("  FAIL %s was not caught by check %s; got %s" % (label, want, found))
    print("selftest: %d cases, %s" % (len(cases), "all as expected" if not bad else "%d FAILED" % bad))
    return 1 if bad else 0


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--panel", action="append")
    ap.add_argument("--record", action="store_true")
    ap.add_argument("--selftest", action="store_true")
    a = ap.parse_args()
    if a.selftest:
        return selftest()
    # A --panel nobody declared matched nothing and reported a clean run. Same
    # shape of defect as the one above: a check that examined zero things said
    # so in the language of success.
    if a.panel:
        known = {spec(x)["id"] for x in specs()}
        unknown = sorted(set(a.panel) - known)
        if unknown:
            print("no panel declared as: %s" % ", ".join(unknown))
            print("declared: %s" % ", ".join(sorted(known)))
            return 1
    found = check_all(set(a.panel) if a.panel else None, a.record)
    for pid, stage, why in found:
        print("  %-10s %-12s %s" % (pid, stage, why))
    n = len(specs())
    print("%d panel(s) declared, %d finding(s)" % (n, len(found)))
    return 1 if found else 0


if __name__ == "__main__":
    # A CRASH MUST LOOK LIKE A FAILURE ON THE LAST LINE, not like silence.
    #
    # This script's verdict is its last line, and that is how it gets read —
    # `panel_check.py | tail -1`, in a pipeline and by hand. When it died inside
    # Playwright instead of reaching that line, the traceback went to stderr and
    # the last line of stdout was nothing at all. A blank line was then read as
    # a clean run, and a broken matrix and three wrong pictures shipped behind
    # it. The exit code was 1 throughout and was the thing not looked at.
    #
    # So an unhandled failure now prints a verdict of its own, in the same shape
    # as the others, and a reader who only ever sees the last line still sees a
    # failure.
    try:
        sys.exit(main())
    except SystemExit:
        raise
    except BaseException as exc:  # noqa: BLE001 - a verdict for anything at all
        import traceback

        traceback.print_exc()
        print(
            "panel_check CRASHED before reaching a verdict: %s: %s"
            % (type(exc).__name__, exc)
        )
        sys.exit(1)
