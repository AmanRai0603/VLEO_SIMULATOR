#!/usr/bin/env python3
"""Release design, refused on screen by the program, in a real browser.

docs/briefs/F6.md, F6-b. A small drive of invented people is written
(`cargo run -p vleo-server --example a_small_drive`), and the program is
started on it with today's design built from that drive. Solar's 1.2 is
taken, and its sustained band breaks the first solar closure; power holds
nothing sealed; thermal's file is refused; structure is mounted and not
started. Then, as people would:

- Test Sam, the system engineer, copies the drive and opens the systems'
  branch in the System workspace: the release panel says the integration
  checks, a line for each of W8's five for each group, solar's seal held
  and the groups with nothing taken failing, each with why;
- the closure solar 1.2 breaks is a red mark beside them, with a box to mark
  it understood and a way to raise it in Explore;
- *Release design* is refused on screen by the program, naming the lines
  that fail, and nothing is written in the copy;
- no page error.

The release that succeeds is proved by the program's own test
(`the_design_is_released_at_the_valve`), from a drive whose every check
holds.

    python3 tools/release_design_check.py

Exit 0 when every step held; 1 with the steps that did not.
"""
import os
import re
import shutil
import subprocess
import sys
import tempfile
import threading
import time
import urllib.request

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from put_back_check import ROOT, URL  # noqa: E402
from node_check import chromium_path, open_key, copy_drive  # noqa: E402


def start(scratch, keys, copy, drive, port):
    """The program on its own copy, with today's design built from `drive`.
    Returns (program, port, word)."""
    env = dict(os.environ)
    env.pop('VLEO_DESIGN', None)
    env.update({
        'VLEO_DRIVE': drive,
        'VLEO_KEYS': keys,
        'VLEO_COPY': copy,
        'VLEO_CASE': os.path.join(scratch, 'case.csv'),
        'VLEO_RESULTS': os.path.join(scratch, 'results'),
        'VLEO_LOG': os.path.join(scratch, 'log'),
        'VLEO_PORT': str(port),
    })
    program = subprocess.Popen(['cargo', 'run', '-q', '-p', 'vleo-daemon'], cwd=ROOT, env=env,
                               stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    served = None
    deadline = time.time() + 600
    while served is None and time.time() < deadline:
        line = program.stdout.readline()
        if not line and program.poll() is not None:
            raise SystemExit('the program stopped before it served')
        m = URL.search(line)
        if m:
            served = m.group(1)
    if served is None:
        program.kill()
        raise SystemExit(f'the program on {port} did not say where it serves within ten minutes')
    threading.Thread(target=lambda: [None for _ in program.stdout], daemon=True).start()
    for _ in range(120):
        try:
            urllib.request.urlopen(f'http://127.0.0.1:{served}/v1/version', timeout=2)
            break
        except OSError:
            time.sleep(0.5)
    page = open(os.path.join(keys, 'open.html')).read()
    word = re.search(r'session=([0-9a-f]+)', page).group(1)
    return program, served, word


def check():
    from playwright.sync_api import sync_playwright
    fails = []

    def step(what, ok, detail=''):
        print(f"  {'ok  ' if ok else 'FAIL'} {what}{' — ' + str(detail)[:300] if detail else ''}")
        if not ok:
            fails.append(what)

    scratch = tempfile.mkdtemp(prefix='vleo-release-')
    programs = []
    try:
        made = subprocess.run(
            ['cargo', 'run', '-q', '-p', 'vleo-server', '--example', 'a_small_drive', '--', scratch],
            cwd=ROOT, capture_output=True, text=True, check=True)
        anchor = made.stdout.strip().splitlines()[-1]
        drive = os.path.join(scratch, 'drive')
        keys = os.path.join(scratch, 'keys')
        copy = os.path.join(scratch, 'copy')
        prog, port, word = start(scratch, keys, copy, drive, 7879)
        programs.append(prog)
        errors = []
        with sync_playwright() as pw:
            b = pw.chromium.launch(executable_path=chromium_path())
            p = b.new_page(viewport={'width': 1400, 'height': 1000})
            p.on('pageerror', lambda e: errors.append(str(e)))
            p.goto(f'http://127.0.0.1:{port}/app#session={word}')
            p.wait_for_selector('#sh-key-card', timeout=60000)
            copy_drive(p, drive)
            open_key(p, 'Test Sam', anchor)

            p.goto(f'http://127.0.0.1:{port}/app#system/systems')
            p.wait_for_selector('.sy-release .sy-r-lines', timeout=180000)
            panel = p.inner_text('.sy-release')
            step('the release panel says the integration checks, and that they fail',
                 'lines fail: the design is not released until each holds' in panel, panel[:400])
            step("solar 1.2's seal holds, read again from the drive",
                 p.locator('.sy-r-holds:has-text("l3_solar 1.2 is taken")').count() == 1, panel[:600])
            fails_text = ' '.join(p.locator('.sy-r-fails').all_inner_texts())
            step('each group with nothing taken fails, with why',
                 all(g in fails_text for g in ['l3_power', 'l3_thermal', 'l3_struct'])
                 and 'l3_struct is mounted, and the drive holds no release of it' in fails_text,
                 fails_text[:500])
            step("a line for each of W8's five checks, for each group",
                 p.locator('.sy-r-lines tbody tr').count() % 5 == 0
                 and p.locator('.sy-r-lines tbody tr:has-text("both ends of every declared range")').count()
                 == p.locator('.sy-r-lines tbody tr').count() // 5, p.locator('.sy-r-lines tbody tr').count())
            p.wait_for_selector('.sy-budgets-said, .sy-budgets-refused', timeout=60000)
            said = p.inner_text('.sy-budgets')
            step('the budgets across releases are read from the drive\'s released designs, and none is released yet',
                 p.locator('.sy-budgets-said').count() == 1
                 and 'no design is released on the drive yet' in said, said[:300])
            marks = p.inner_text('.sy-r-marks')
            step('the closure solar 1.2 breaks is a red mark, to be understood or raised',
                 p.locator('.sy-r-understood[data-mark^="closure:"]').count() >= 1
                 and p.locator('.sy-r-raise[href="#explore/design"]').count() >= 1, marks[:300])

            for box in p.locator('.sy-r-understood').all():
                box.check()
            p.fill('#sy-r-changed', "Solar's sustained band, as 1.2 says it.")
            p.click('#sy-r-release')
            p.wait_for_selector('.sy-r-notice', timeout=120000)
            notice = p.inner_text('.sy-r-notice')
            step('Release design is refused on screen, naming the lines that fail',
                 notice.startswith('the design is not released, refused whole:')
                 and 'l3_struct, the seal and the signature chain' in notice, notice)
            step('what was typed is kept', p.input_value('#sy-r-changed') == "Solar's sustained band, as 1.2 says it.")
            step('and nothing is written in the copy',
                 not os.path.exists(os.path.join(copy, 'design')) and not os.path.exists(os.path.join(copy, 'readable')))
            step('no page error', not errors, '; '.join(errors[:3]))
            b.close()
    finally:
        for prog in programs:
            prog.terminate()
            try:
                prog.wait(timeout=10)
            except subprocess.TimeoutExpired:
                prog.kill()
        shutil.rmtree(scratch, ignore_errors=True)
    return fails


def main():
    fails = check()
    if fails:
        print(f'\n{len(fails)} step(s) did not hold.')
        return 1
    print('\nevery step held')
    return 0


if __name__ == '__main__':
    sys.exit(main())
