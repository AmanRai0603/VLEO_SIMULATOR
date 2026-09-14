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

  1 · it renders   the mount is not empty and not zero-sized
  2 · it moves     change each input it declares it reads, and its content must
                   change. A panel wired to nothing passes every other check,
                   including a screenshot comparison, because it draws the same
                   correct picture whatever the data says
  3 · it matches   against a stored reference, within tolerance

Check two is the one worth having. "Three correct power numbers on one screen,
correct at three different times" is a panel that renders, matches yesterday's
reference, and is reading state nobody refreshed.

This drives the real page in a real browser against the real daemon. A check
that runs against a mock proves the mock works.
"""

import argparse
import io
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


def selftest():
    """A panel wired to nothing must fail check two, and a blank one check one.

    Four cases: both failures on a DOM panel and both on a canvas panel. All
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
            break_it(work)
            ROOT, PANELS, REFERENCE = work, work / "panels", work / "panels" / "reference"
            try:
                found = check_all(ids={"sweep"} if "canvas" in label else {"tree"})
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
    found = check_all(set(a.panel) if a.panel else None, a.record)
    for pid, stage, why in found:
        print("  %-10s %-12s %s" % (pid, stage, why))
    n = len(specs())
    print("%d panel(s) declared, %d finding(s)" % (n, len(found)))
    return 1 if found else 0


if __name__ == "__main__":
    sys.exit(main())
