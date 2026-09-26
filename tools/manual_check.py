#!/usr/bin/env python3
"""Every command in the manual, run exactly as a person would paste it.

    tools/manual_check.py              run them; needs cargo, and builds what it must
    tools/manual_check.py --browser    follow the browser half on the real page
    tools/manual_check.py --selftest   prove this checker catches what it is for

The manual (docs/manual.toml, the Manual tab in the browser) tells people what
to paste and what they will see. `cargo test -p xtask --test the_manual_is_true`
proves every NAME in it exists. This proves every COMMAND DOES what the manual
says, by doing it:

  exits    run it; it must exit 0 and print what `expect` says
  fails    run it; it must REFUSE, and say what `expect` says — the step is
           showing a person what a refusal looks like, and a refusal that has
           quietly started succeeding is the drift
  serves   start it; it must come up, answer, and be in the mode the manual
           says — read-only unless it was started with VLEO_ALLOW_WRITE=1,
           asked of the running copy itself
  probe    a request to the copy `serves` started
  writes   not run: it would change the repository this is checking. The
           manual says so beside it, and its name is checked by the Rust test
  ci       not run here: the pipeline runs it as its own step, and the Rust
           test finds it there

AND A COMMAND LABELLED SAFE MUST BE SAFE. The page marks every `exits` command
"safe to run — only reads". That is a claim about the repository, so it is
checked on the repository: the working tree is compared before and after each
one, and a command that changed it fails here even if it exited 0 and printed
the right thing.

Commands are run with `bash -c`, from the top of the repository, with colour
switched off where the programs honour it and stripped where they do not.

THE BROWSER HALF IS FOLLOWED, NOT INSPECTED. `--browser` starts the tool the
way the manual says to, opens it in a real browser, and does what the browser
layer tells a person to do — clicking each label BY THE TEXT THE MANUAL GIVES
FOR IT, read from the running copy's own /v1/manual. So a label renamed in the
page and not the manual fails here, and so does one renamed in the manual and
not the page. It also holds the manual to the claims a person would be hurt by
if they were false: that a what-if and a paste write nothing, that a read-only
copy refuses a save and says how to start one that will not, and that a run
always says how many rows ran and how many were blocked.
"""

import argparse
import os
import re
import signal
import subprocess
import sys
import tempfile
import time
import tomllib
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MANUAL = ROOT / "docs" / "manual.toml"
ANSI = re.compile(r"\x1b\[[0-9;]*m")
URL = re.compile(r"http://127\.0\.0\.1:(\d+)")

# The first build of the daemon in release mode is the slow part of a fresh
# checkout, and the pipeline has usually done it already by the time this runs.
TIMEOUT_RUN = 900
TIMEOUT_SERVE = 1200


def plain(s):
    return ANSI.sub("", s)


def steps(manual):
    """Every step that carries a command, with where it is."""
    out = []
    for layer in manual.get("layer", []):
        for sec in layer.get("section", []):
            for n, st in enumerate(sec.get("step", []), 1):
                if "run" in st:
                    out.append((f"{sec['id']} #{n}", st))
    return out


def tree_state(cwd):
    """The working tree as git sees it — what a command marked safe must not move."""
    r = subprocess.run(
        ["git", "status", "--porcelain", "--untracked-files=all"],
        cwd=cwd, capture_output=True, text=True,
    )
    return r.stdout


def run(cmd, cwd, timeout=TIMEOUT_RUN):
    env = dict(os.environ, NO_COLOR="1", CARGO_TERM_COLOR="never")
    try:
        r = subprocess.run(
            ["bash", "-c", cmd], cwd=cwd, capture_output=True, text=True,
            timeout=timeout, env=env,
        )
    except subprocess.TimeoutExpired:
        return None, f"did not finish in {timeout}s"
    return r.returncode, plain(r.stdout + r.stderr)


def judge(kind, code, out, expect, before, after):
    """What is wrong with one run of an exits or fails step, or None.

    Separate from running it so the self-test can hand it outcomes directly.
    """
    if code is None:
        return out
    if kind == "exits" and code != 0:
        tail = "\n".join(out.strip().splitlines()[-6:])
        return f"exited {code}, and the manual says it succeeds:\n{tail}"
    if kind == "fails" and code == 0:
        return "succeeded, and the manual shows it as a refusal"
    if expect and expect not in out:
        tail = "\n".join(out.strip().splitlines()[-6:])
        return f"did not print `{expect}`, which the manual says it does. It printed:\n{tail}"
    if before != after:
        moved = sorted(set(after.splitlines()) ^ set(before.splitlines()))
        return ("changed the working tree, and the manual labels it safe to run: "
                + "; ".join(moved[:6]))
    return None


def port_of(text):
    """The port a daemon said it is serving on, or None."""
    m = URL.search(text)
    return int(m.group(1)) if m else None


def aimed(cmd, port):
    """A probe written for the default port, pointed at the one we actually got.

    Said out loud when it happens rather than done quietly: if 7777 was taken,
    the manual's own address would have reached whatever was holding it.
    """
    return cmd.replace("127.0.0.1:7777", f"127.0.0.1:{port}")


def get(url, timeout=10):
    with urllib.request.urlopen(url, timeout=timeout) as r:
        return r.status, r.read().decode("utf-8", "replace")


class Serving:
    """A `serves` command, started as written and stopped afterwards."""

    def __init__(self, cmd):
        self.cmd = cmd
        self.log = tempfile.NamedTemporaryFile("w+", suffix=".log", delete=False)
        self.proc = None
        self.port = None

    def __enter__(self):
        env = dict(os.environ, NO_COLOR="1", CARGO_TERM_COLOR="never")
        self.proc = subprocess.Popen(
            ["bash", "-c", self.cmd], cwd=ROOT, env=env,
            stdout=self.log, stderr=subprocess.STDOUT, start_new_session=True,
        )
        deadline = time.time() + TIMEOUT_SERVE
        while time.time() < deadline:
            if self.proc.poll() is not None:
                break
            self.log.flush()
            text = Path(self.log.name).read_text(errors="replace")
            p = port_of(text)
            if p:
                try:
                    code, _ = get(f"http://127.0.0.1:{p}/v1/version", timeout=2)
                    if code == 200:
                        self.port = p
                        return self
                except OSError:
                    pass
            time.sleep(0.5)
        return self

    def output(self):
        return plain(Path(self.log.name).read_text(errors="replace"))

    def __exit__(self, *a):
        if self.proc and self.proc.poll() is None:
            os.killpg(self.proc.pid, signal.SIGTERM)
            try:
                self.proc.wait(timeout=15)
            except subprocess.TimeoutExpired:
                os.killpg(self.proc.pid, signal.SIGKILL)
        os.unlink(self.log.name)


def check_serving(s):
    """What is wrong with a started daemon, given the command that started it."""
    if s.port is None:
        tail = "\n".join(s.output().strip().splitlines()[-8:])
        return f"never answered. It printed:\n{tail}"
    code, body = get(f"http://127.0.0.1:{s.port}/v1/manual")
    if code != 200 or '"ok":true' not in body:
        return f"/v1/manual answered {code}"
    # THE MODE IS WHAT THE MANUAL SAYS IT IS. "It starts read-only" and "start
    # it with VLEO_ALLOW_WRITE=1 to edit" are claims about the program, so the
    # program is asked.
    wants = "VLEO_ALLOW_WRITE=1" in s.cmd
    has = '"writes_allowed":true' in body
    if wants != has:
        return (f"started {'accepting edits' if has else 'read-only'}, and the manual "
                f"says this command starts it {'accepting edits' if wants else 'read-only'}")
    return None


def usage_problems(manual):
    """Each command's usage, against the help the program actually prints."""
    helps = {}
    for tool, cmd in [
        ("xtask", "cargo run -q -p xtask -- help"),
        ("vleo", "cargo run -q -p vleo-cli --bin vleo -- help"),
    ]:
        code, out = run(cmd, ROOT)
        if code != 0:
            return [f"`{cmd}` exited {code}"]
        helps[tool] = out
    bad = []
    for c in manual.get("command", []):
        if c["usage"] not in helps[c["tool"]]:
            bad.append(f"{c['tool']} {c['name']}: the manual's usage `{c['usage']}` "
                       f"is not in what `{c['tool']} help` prints")
    return bad


def main_run(args):
    manual = tomllib.loads(MANUAL.read_text())
    all_steps = steps(manual)
    failures, ran, skipped = [], 0, {"writes": 0, "ci": 0}

    def report(where, kind, cmd, problem):
        nonlocal ran
        ran += 1
        mark = "ok  " if problem is None else "FAIL"
        print(f"  {mark} {kind:6} {where:22} {cmd}")
        if problem:
            failures.append((where, cmd, problem))
            for line in problem.splitlines():
                print(f"         {line}")

    print("the commands a person is told to paste")
    for where, st in all_steps:
        kind, cmd = st["check"], st["run"]
        if kind in ("writes", "ci"):
            skipped[kind] += 1
            continue
        if kind in ("exits", "fails"):
            before = tree_state(ROOT)
            code, out = run(cmd, ROOT)
            after = tree_state(ROOT)
            report(where, kind, cmd, judge(kind, code, out, st.get("expect"), before, after))

    print("\nthe daemon, started as written, and asked about itself")
    serves = [(w, st) for w, st in all_steps if st["check"] == "serves"]
    probes = [(w, st) for w, st in all_steps if st["check"] == "probe"]
    for i, (where, st) in enumerate(serves):
        with Serving(st["run"]) as s:
            report(where, "serves", st["run"], check_serving(s))
            # The probes are asked of the first copy — the read-only one a
            # person starts first — while it is still running.
            if i == 0 and s.port is not None:
                if s.port != 7777:
                    print(f"         (7777 was taken; the probes below are sent to {s.port})")
                for pw, pst in probes:
                    code, out = run(aimed(pst["run"], s.port), ROOT, timeout=60)
                    report(pw, "probe", pst["run"], judge("exits", code, out, pst.get("expect"), "", ""))

    print("\nevery usage line, against what each program's help prints")
    bad = usage_problems(manual)
    for b in bad:
        print(f"  FAIL {b}")
        failures.append(("usage", "", b))
    if not bad:
        print(f"  ok   {len(manual.get('command', []))} usage lines")

    print(f"\n{ran} command(s) run, {len(failures)} wrong. Not run: {skipped['writes']} that "
          f"change files, {skipped['ci']} the pipeline runs as its own steps.")
    return 1 if failures else 0


# The rows the walk uses: a computed row whose whole chain answers, and the
# declared number it reads. Both in the solar subsystem, the one written all
# the way through.
WALK_COMPUTED = "sw_ap_design"
WALK_DECLARED = "sw_storm_design_level"
WALK_SUBSYS = "l3_solar"


def browser_walk():
    """Follow the browser layer of the manual, on the real page."""
    try:
        from playwright.sync_api import sync_playwright
    except ImportError:
        print("the browser walk needs playwright: pip install playwright")
        return 1
    sys.path.insert(0, str(ROOT / "tools"))
    from panel_check import chromium_path

    failures = []

    def ok(label, fn):
        try:
            fn()
            print(f"  ok   {label}")
        except Exception as e:  # noqa: BLE001 — every failure is reported, none stops the walk
            msg = str(e).strip().splitlines()[0][:300] if str(e).strip() else type(e).__name__
            print(f"  FAIL {label}\n         {msg}")
            failures.append(label)

    read_only = next(st["run"] for _, st in steps(tomllib.loads(MANUAL.read_text()))
                     if st["check"] == "serves" and "VLEO_" not in st["run"])
    with Serving(read_only) as s:
        if s.port is None:
            print(f"  FAIL the tool did not start with `{read_only}`\n{s.output()[-800:]}")
            return 1
        base = f"http://127.0.0.1:{s.port}/"
        _, body = get(base + "v1/manual")
        import json
        live = json.loads(body)
        # Every label, as the manual gives it: (section, step number) -> text.
        ui = {}
        for layer in live["manual"]["layers"]:
            for sec in layer["sections"]:
                for n, st in enumerate(sec["steps"], 1):
                    if st.get("ui"):
                        ui[(sec["id"], n)] = st["ui"]

        def U(sec, n):
            if (sec, n) not in ui:
                raise AssertionError(f"the manual no longer has a label at {sec} step {n}")
            return ui[(sec, n)]

        with sync_playwright() as pw:
            browser = pw.chromium.launch(executable_path=chromium_path())
            page = browser.new_page(viewport={"width": 1280, "height": 1000})
            errors = []
            page.on("pageerror", lambda e: errors.append(str(e)))
            page.set_default_timeout(15000)

            def home():
                page.goto(base, wait_until="networkidle")

            def tab(text):
                page.locator("#tabs .tab", has_text=text).first.click()
                page.wait_for_timeout(300)

            def open_row(row):
                home()
                tab(U("browser-find", 2))
                page.select_option("#subsys", WALK_SUBSYS)
                page.wait_for_timeout(400)
                page.locator(f'.mcell.diag[data-open="{row}"]').first.click()
                page.wait_for_selector("#nodeview:not([hidden]) .sheet-tabs")

            def button(scope, text):
                return page.locator(f"{scope} button", has_text=text).first

            clean = tree_state(ROOT)

            def untouched(what):
                now = tree_state(ROOT)
                assert now == clean, f"{what} changed the working tree: {now or '(clean)'} vs {clean or '(clean)'}"

            print("the manual itself")

            def manual_opens():
                home()
                page.locator("#manual-tab").click()
                page.wait_for_selector(".man-live")
                live_text = page.locator(".man-live").inner_text()
                assert "read-only" in live_text, f"the status card does not say read-only: {live_text[:200]}"
                titles = page.locator(".man-lt").all_inner_texts()
                for l in live["manual"]["layers"]:
                    assert l["title"] in titles, f"no tab for the layer '{l['title']}'"
            ok("the Manual tab opens, and says this copy is read-only", manual_opens)

            def deep_link():
                page.goto(base + "#manual/term-run", wait_until="networkidle")
                page.wait_for_selector("#man-term-run")
                sel = page.locator(".man-lt.sel").inner_text()
                assert sel == "In the terminal", f"a link to a terminal section opened '{sel}'"
                top = page.locator("#man-term-run").bounding_box()["y"]
                assert 0 <= top < 400, f"the linked section is not in view (it is at y={top})"
            ok("a link to one section opens the manual at that section", deep_link)

            def who_filter():
                page.locator(".man-f", has_text="a user").click()
                assert page.locator('.man-sec[data-who="developer"]').count() == 0, "a developer section shows to a user"
                page.locator(".man-f", has_text="a developer").click()
                assert page.locator('.man-sec[data-who="user"]').count() == 0, "a user section shows to a developer"
                page.locator(".man-f", has_text="everyone").click()
            ok("'for a user' and 'for a developer' each show only their own", who_filter)

            def copy_button():
                b = page.locator(".man-copy").first
                b.click()
                page.wait_for_timeout(200)
                assert b.inner_text() in ("copied", "selected \u2014 press Ctrl-C"), f"the copy button said '{b.inner_text()}'"
            ok("a command's copy button copies it", copy_button)

            # A manual that has to be scrolled sideways on a phone is a manual
            # nobody reads on one. The tree above the manual is not held to this.
            def phone():
                page.set_viewport_size({"width": 390, "height": 844})
                try:
                    for where in ("#manual", "#manual/terminal"):
                        page.goto(base + where, wait_until="networkidle")
                        page.wait_for_selector(".man-live")
                        wide = page.evaluate("document.documentElement.scrollWidth")
                        assert wide <= 390, f"{where} is {wide}px wide on a 390px screen"
                        assert not page.locator("#stepper").is_visible(), "the tree's stepper shows over the manual"
                        box = page.locator(".man-cmd-box").first.bounding_box()
                        btn = page.locator(".man-copy").first.bounding_box()
                        assert btn["y"] + btn["height"] <= box["y"] + box["height"] + 1, \
                            "a copy button sits outside the command it copies"
                finally:
                    page.set_viewport_size({"width": 1280, "height": 1000})
            ok("on a phone the manual fits, with the copy button in its command", phone)

            print("\nfind your way around")

            def layers():
                home()
                tab(U("browser-find", 1))
                assert page.locator("#figure").is_visible(), "the layer did not open"
                tab(U("browser-find", 2))
                assert page.locator("#subsys").is_visible(), "no subsystem menu on layer 3"
                # Expanding is shown where there is something to expand: the
                # system layer opens with its groups closed. A flat subsystem
                # has nothing to open, and "nothing changed" there is correct.
                tab("2 The system")
                before = page.locator(".mcell.diag").count()
                button("#controls", U("browser-find", 3)).click()
                page.wait_for_timeout(300)
                after = page.locator(".mcell.diag").count()
                assert after > before, f"expand all opened nothing ({before} -> {after} cells)"
            ok("pick a layer, choose a subsystem, expand everything", layers)

            def howto_and_arch():
                button("#controls", U("browser-find", 6)).click()
                assert page.locator("#howto-panel").is_visible(), "the explanation did not open"
                page.locator("#howto-close").click()
                tab(U("browser-find", 7))
                assert page.locator("#archview").is_visible(), "the architecture view did not open"
            ok("the tool explains its own figure, and the architecture tab opens", howto_and_arch)

            print("\nread a row")

            def read_a_row():
                open_row(WALK_COMPUTED)
                text = page.locator("#node-body").inner_text()
                for n in (1, 2):
                    assert U("browser-read", n).lower() in text.lower(), f"no '{U('browser-read', n)}' on the page"
                assert button("#node-body .sheet-tabs", U("browser-read", 3)).is_visible()
                # Back, by the label the manual gives it.
                page.locator("#nodeview button", has_text=U("browser-read", 4)).first.click()
                page.wait_for_timeout(300)
                assert page.locator("#figure").is_visible(), "back did not return to the layer"
            ok("a row's page has the parts the manual names, and back returns", read_a_row)

            print("\nrun a row")

            def run_a_row():
                open_row(WALK_COMPUTED)
                panel = "#run-panel"
                button(panel, U("browser-run", 2)).click()
                button(panel, U("browser-run", 4)).click()
                page.wait_for_function(
                    "() => /ran/.test(document.querySelector('#run-panel').innerText) && "
                    "/blocked/.test(document.querySelector('#run-panel').innerText)", timeout=60000)
                untouched("a run")
            ok("a run says how many rows ran and how many were blocked", run_a_row)

            def run_tab():
                home()
                tab(U("browser-run", 5))
                assert page.locator("#runview").is_visible(), "the run tab did not open"
            ok("the run tab opens", run_tab)

            print("\nwhat if")

            def computed_has_no_box():
                open_row(WALK_COMPUTED)
                assert page.locator(f'[data-ovr="{WALK_COMPUTED}"] .ovr-v').count() == 0, \
                    "a computed row offers a what-if box, and the manual says it cannot"
            ok("a computed row offers no what-if box", computed_has_no_box)

            def what_if():
                open_row(WALK_DECLARED)
                box = page.locator(f'[data-ovr="{WALK_DECLARED}"] .ovr-v').first
                box.fill("2")
                box.press("Tab")
                button("#node-body", U("browser-whatif", 2)).click()
                page.wait_for_selector("#ovrbar:not([hidden])", timeout=60000)
                bar = page.locator("#ovrbar")
                assert U("browser-whatif", 4) in bar.inner_text(), "the bar has no way to put everything back"
                untouched("a what-if")
                bar.locator("button", has_text=U("browser-whatif", 4)).click()
                page.wait_for_timeout(300)
                assert page.locator("#ovrbar").is_hidden(), "putting everything back left the bar up"
            ok("a what-if shows the bar, writes nothing, and puts back", what_if)

            print("\nthe form, on a read-only copy")

            def read_only_save():
                open_row(WALK_COMPUTED)
                button("#node-body .sheet-tabs", U("browser-edit", 1)).click()
                page.wait_for_selector('.sf-field[data-field="note"] textarea')
                box = page.locator('.sf-field[data-field="note"]')
                box.locator("textarea").fill("a note the walk types and is refused")
                box.locator("button.sf-save").click()
                page.wait_for_function(
                    "() => { const e=document.querySelector('.sf-field[data-field=\"note\"] .sf-said');"
                    " return e && !e.hidden && !e.classList.contains('waiting') && e.textContent.trim(); }")
                said = box.locator(".sf-said").inner_text()
                assert "read-only" in said and "VLEO_ALLOW_WRITE" in said, \
                    f"a save on a read-only copy said: {said}"
                untouched("a refused save")
            ok("a save is refused on a read-only copy, and says how to start one that allows it", read_only_save)

            def paste_writes_nothing():
                page.locator(".sf-paste summary", has_text=U("browser-paste", 1)).click()
                page.locator(".sf-paste-in").fill('[question]\nnote = "from a sibling"\n')
                button(".sf-paste", U("browser-paste", 2)).click()
                page.wait_for_selector(".sf-diff")
                assert "note" in page.locator(".sf-diff").inner_text()
                assert button(".sf-paste", U("browser-paste", 3)).is_visible()
                untouched("a paste preview")
            ok("a paste shows what would change, and writes nothing", paste_writes_nothing)

            def the_rest_is_where_it_says():
                page.locator(".sf-add summary").first.click()
                assert button(".sf-array .sf-add", U("browser-blocks", 1)).is_visible(), \
                    f"no '{U('browser-blocks', 1)}' once a list's add box is open"
                assert button(".sf-view", U("browser-blocks", 2)).is_visible()
                assert page.locator(".sf-propose summary", has_text=U("browser-share", 1)).is_visible()
            ok("the lists, the view, and the branch step are where the manual says", the_rest_is_where_it_says)

            if errors:
                failures.append("the page threw")
                print("  FAIL the page threw:\n         " + "\n         ".join(errors[:5]))
            browser.close()

    print(f"\nthe browser half: {len(failures)} wrong")
    return 1 if failures else 0


def selftest():
    """Hand every judgement a case it must catch and one it must pass."""
    problems = []

    def expect(label, got_problem, wanted_problem):
        if bool(got_problem) != wanted_problem:
            problems.append(f"{label}: {'missed it' if wanted_problem else 'false alarm'} — {got_problem}")

    same = "?? a\n"
    expect("exits, exit 0, expect found", judge("exits", 0, "hello world", "hello", same, same), False)
    expect("exits, nonzero", judge("exits", 1, "boom", None, same, same), True)
    expect("exits, expect missing", judge("exits", 0, "hello", "goodbye", same, same), True)
    expect("fails, refused", judge("fails", 1, "is computed from its inputs", "is computed", same, same), False)
    expect("fails, but succeeded", judge("fails", 0, "fine", None, same, same), True)
    expect("fails, refused for the wrong reason", judge("fails", 1, "no such node", "is computed", same, same), True)
    expect("timed out", judge("exits", None, "did not finish", None, same, same), True)
    expect("changed the tree", judge("exits", 0, "done", None, "", " M node.toml\n"), True)
    # Colour INSIDE the phrase, which is where it breaks a match: codes around
    # a phrase leave it intact either way, and a test built like that passed
    # with the stripping removed.
    expect("colour inside the phrase is not text",
           judge("exits", 0, plain("2 ran, \x1b[33m0\x1b[0m blocked"), "2 ran, 0 blocked", same, same), False)

    if port_of("serving on \x1b[1mhttp://127.0.0.1:7781\x1b[0m") != 7781:
        problems.append("did not read the port a daemon printed")
    if port_of("nothing here") is not None:
        problems.append("read a port from text that has none")
    if aimed("curl -s http://127.0.0.1:7777/v1/version", 7781) != "curl -s http://127.0.0.1:7781/v1/version":
        problems.append("did not point a probe at the port actually serving")

    # The real thing, on a throwaway repository: a command that says it only
    # reads and in fact writes a file must be caught by the tree comparison,
    # end to end — not only in `judge`.
    with tempfile.TemporaryDirectory() as d:
        subprocess.run(["git", "init", "-q", d], check=True)
        before = tree_state(d)
        code, out = run("touch sneaky.txt", d, timeout=30)
        after = tree_state(d)
        expect("a command that writes a file, run for real",
               judge("exits", code, out, None, before, after), True)
        before = tree_state(d)
        code, out = run("echo reading only", d, timeout=30)
        after = tree_state(d)
        expect("a command that only reads, run for real",
               judge("exits", code, out, "reading only", before, after), False)

    # And the manual itself parses, with every step that runs saying how.
    manual = tomllib.loads(MANUAL.read_text())
    for where, st in steps(manual):
        if st.get("check") not in ("exits", "fails", "serves", "writes", "ci", "probe"):
            problems.append(f"{where}: check = {st.get('check')!r}")

    for p in problems:
        print(f"  FAIL {p}")
    print(f"selftest: {'ok' if not problems else str(len(problems)) + ' problem(s)'}")
    return 1 if problems else 0


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--selftest", action="store_true")
    ap.add_argument("--browser", action="store_true")
    args = ap.parse_args()
    if args.selftest:
        sys.exit(selftest())
    sys.exit(browser_walk() if args.browser else main_run(args))


if __name__ == "__main__":
    main()
