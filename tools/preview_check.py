#!/usr/bin/env python3
"""A preview of a release, written on screen by the program, in a real browser.

docs/briefs/F6.md, F6-c items 3, 4 and 10. A small drive of invented people
is written with a design released on it
(`cargo run -p vleo-server --example a_small_drive -- <folder> --released`):
design 2026.10.1 holds solar 1.1, and today's design takes solar 1.2, whose
long design F10.7 answers from another table below a centre of 95 sfu. Test
Uma owns the orbit's environment, which reads it. Then, as people would:

- Test Sam, the system engineer, copies the drive and opens the systems'
  branch: the release panel says solar 1.2 is in today's design and design
  2026.10.1 holds solar 1.1, and offers *Preview l3_solar 1.2*;
- pressed, the program builds both designs and runs them on the design as
  declared, and says on screen the preview it wrote in the copy and whom it
  names to answer; the preview is listed, in the copy and not put back,
  with each group it names unanswered and what moved, the engine's;
- the copy holds the preview, and the drive does not;
- Test Uma opens her key, and My work says *Answer l3_solar 1.2*;
- no page error.

Answering, and a preview put back, are proved by the program's own test
(`a_preview_names_who_answers_a_release`).

    python3 tools/preview_check.py

Exit 0 when every step held; 1 with the steps that did not.
"""
import os
import shutil
import subprocess
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from put_back_check import ROOT  # noqa: E402
from node_check import chromium_path, open_key, copy_drive  # noqa: E402
from release_design_check import start  # noqa: E402

PREVIEW = 'integration/l3_solar-1.2/preview-l3_solar-1.2-on-2026.10.1.vleo'


def check():
    from playwright.sync_api import sync_playwright
    fails = []

    def step(what, ok, detail=''):
        print(f"  {'ok  ' if ok else 'FAIL'} {what}{' — ' + str(detail)[:300] if detail else ''}")
        if not ok:
            fails.append(what)

    scratch = tempfile.mkdtemp(prefix='vleo-preview-')
    programs = []
    try:
        made = subprocess.run(
            ['cargo', 'run', '-q', '-p', 'vleo-server', '--example', 'a_small_drive', '--', scratch, '--released'],
            cwd=ROOT, capture_output=True, text=True, check=True)
        anchor = made.stdout.strip().splitlines()[-1]
        drive = os.path.join(scratch, 'drive')
        keys = os.path.join(scratch, 'keys')
        copy = os.path.join(scratch, 'copy')
        prog, port, word = start(scratch, keys, copy, drive, 7891)
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
            pending = p.inner_text('.sy-p-pendings')
            step('the panel says solar 1.2 is not the release design 2026.10.1 holds',
                 'l3_solar 1.2 is in today\'s design, and design 2026.10.1 holds l3_solar 1.1' in pending, pending)
            step('and offers Preview l3_solar 1.2',
                 p.locator('button[data-preview="l3_solar"]:has-text("Preview l3_solar 1.2")').count() == 1, pending)
            step('no preview is listed yet', p.locator('.sy-p-one').count() == 0)

            p.click('button[data-preview="l3_solar"]')
            p.wait_for_selector('.sy-p-notice', timeout=600000)
            notice = p.inner_text('.sy-p-notice')
            step('pressed, the program says the preview it wrote in the copy',
                 notice.startswith('Test Sam wrote the preview of l3_solar 1.2 on design 2026.10.1, in your copy: ' + PREVIEW),
                 notice)
            step('and whom it names to answer', 'names l3_x_envorbit to answer' in notice, notice)
            p.wait_for_selector('.sy-p-one', timeout=60000)
            one = p.inner_text('.sy-p-one')
            step('the preview is listed, in the copy and not put back',
                 'l3_solar 1.2 on design 2026.10.1, written by Test Sam, in your copy, not put back yet' in one, one[:400])
            step('each group it names unanswered',
                 p.locator('.sy-p-said.sy-p-unanswered:has-text("l3_x_envorbit")').count() == 1, one[:400])
            p.click('.sy-p-one summary')
            moved = p.inner_text('.sy-p-moved')
            step("and what moved, the orbit's environment's F10.7 among it",
                 'env_f107' in moved and 'l3_x_envorbit' in moved and 'l3_solar\t' not in moved, moved[:400])
            step('the pending line now names the preview, with no button',
                 PREVIEW in p.inner_text('.sy-p-pendings') and p.locator('button[data-preview]').count() == 0)
            step('the copy holds the preview, and the drive does not',
                 os.path.isfile(os.path.join(copy, PREVIEW)) and not os.path.exists(os.path.join(drive, PREVIEW)))

            # Uma's My work asks her to answer it.
            p.goto(f'http://127.0.0.1:{port}/app')
            p.wait_for_selector('#sh-close', timeout=60000)
            p.click('#sh-close')
            p.wait_for_selector('#sh-open', timeout=60000)
            open_key(p, 'Test Uma', anchor)
            p.wait_for_selector('#sh-items', timeout=60000)
            items = p.inner_text('#sh-items')
            step("Uma's My work says Answer l3_solar 1.2",
                 p.locator('#sh-items li[data-kind="answer"]:has-text("Answer l3_solar 1.2")').count() == 1, items[:400])
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
