#!/usr/bin/env python3
"""The frame at the system engineer's valve, in a real browser on the real program.

docs/briefs/F6.md, F6-a. A small drive of invented people is written
(`cargo run -p vleo-server --example a_small_drive`), and the program is
started on it. Then, as people would:

- Test Sam, the system engineer, copies the drive and opens the systems'
  branch in the System workspace: the frame says the valve above it is the
  programme's file, and lists every group mounted below, structure not
  started;
- on structure's mount, an allocation with no sense chosen is refused on
  screen and nothing is written; with `<=` it is made, and the frame says it
  and which way it binds;
- on solar's leaf `sw_activity_count`, Sam mounts a group for Test Ola (W13),
  and that group's branch says it is mounted for Ola, not started, with Ola
  the valve above it, and not Sam's to start;
- Test Ola opens their key: My work says *Start l3_struct*, and its link
  opens the System workspace there, with *Start l3_struct*; started, the
  branch is drawn with its one heading, and what it is allocated;
- solar's leaf now carries the group mounted on it, and Ola is offered no
  edit of it that the program would refuse unsaid;
- the budgets are a link to Explore's health map, and no page error.

    python3 tools/start_group_check.py

Exit 0 when every step held; 1 with the steps that did not.
"""
import hashlib
import os
import shutil
import subprocess
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from put_back_check import ROOT, start  # noqa: E402
from node_check import chromium_path, open_key, copy_drive  # noqa: E402

LEAF = 'sw_activity_count'
NEW = 'l3_count'
SYSTEMS = 'groups/systems/systems.vgroup'
WHY = 'Structure is given its mass; the count is a group of its own.'


def digest(at):
    with open(at, 'rb') as f:
        return hashlib.sha256(f.read()).hexdigest()


def drawn(p, group):
    """Wait for the System workspace to have drawn `group`, started or not."""
    p.wait_for_selector(f'.answer-first:has-text("{group}")', timeout=120000)
    p.wait_for_selector('.sy-frame', timeout=120000)


def said_after(p, before):
    """The workspace's notice once it says something other than `before`."""
    p.wait_for_function(
        "b => { const n = document.querySelector('.sy-notice'); return n && n.innerText !== b; }",
        arg=before, timeout=60000)
    return p.inner_text('.sy-notice')


def pick(p, uid):
    p.click(f'.sy-tree [data-block="{uid}"]')
    p.wait_for_selector(f'.sy-tree [data-block="{uid}"].sel', timeout=60000)


def open_group(p, port, group):
    p.goto(f'http://127.0.0.1:{port}/app#system/{group}')
    drawn(p, group)


def check():
    from playwright.sync_api import sync_playwright
    fails = []

    def step(what, ok, detail=''):
        print(f"  {'ok  ' if ok else 'FAIL'} {what}{' — ' + str(detail)[:300] if detail else ''}")
        if not ok:
            fails.append(what)

    scratch = tempfile.mkdtemp(prefix='vleo-start-')
    programs = []
    try:
        made = subprocess.run(
            ['cargo', 'run', '-q', '-p', 'vleo-server', '--example', 'a_small_drive', '--', scratch],
            cwd=ROOT, capture_output=True, text=True, check=True)
        anchor = made.stdout.strip().splitlines()[-1]
        drive = os.path.join(scratch, 'drive')
        keys = os.path.join(scratch, 'keys')
        prog, port, word = start(scratch, keys, os.path.join(scratch, 'copy'), 7877)
        programs.append(prog)
        mine = os.path.join(scratch, 'copy', SYSTEMS)
        errors = []
        with sync_playwright() as pw:
            b = pw.chromium.launch(executable_path=chromium_path())
            p = b.new_page(viewport={'width': 1400, 'height': 1000})
            p.on('pageerror', lambda e: errors.append(str(e)))
            p.goto(f'http://127.0.0.1:{port}/app#session={word}')
            p.wait_for_selector('#sh-key-card', timeout=60000)
            copy_drive(p, drive)
            open_key(p, 'Test Sam', anchor)

            # The systems' branch, and the frame around it.
            open_group(p, port, 'systems')
            frame = p.inner_text('.sy-frame')
            step("the frame says the valve above the systems' branch is the programme's file",
                 "the programme's file" in p.inner_text('.sy-valve-above'), frame[:300])
            below = p.inner_text('.sy-below')
            step('and every group mounted below, structure not started',
                 all(g in below for g in ['l3_solar', 'l3_power', 'l3_struct'])
                 and 'l3_struct on sys_structure, owned by Test Ola, not started' in below, below[:400])

            # An allocation on structure's mount: with no sense, refused.
            pick(p, 'sys_structure')
            before = digest(mine)
            p.fill('#sy-f-why', WHY)
            p.fill('#sy-a-name', 'structure_mass_allocated')
            p.fill('#sy-a-value', '40')
            p.fill('#sy-a-unit', 'kg')
            p.select_option('#sy-a-bounds', 'structure_mass')
            p.click('#sy-allocate')
            notice = said_after(p, '')
            step('an allocation with no sense chosen is refused on screen',
                 'say which way structure_mass_allocated binds structure_mass' in notice, notice)
            step('and nothing is written', digest(mine) == before)
            pick(p, 'sys_structure')
            p.fill('#sy-f-why', WHY)
            p.fill('#sy-a-name', 'structure_mass_allocated')
            p.fill('#sy-a-value', '40')
            p.fill('#sy-a-unit', 'kg')
            p.select_option('#sy-a-bounds', 'structure_mass')
            p.select_option('#sy-a-sense', '<=')
            p.click('#sy-allocate')
            notice = said_after(p, notice)
            step('with <= it is made', 'made the edit "allocate"' in notice, notice)
            open_group(p, port, 'l3_struct')
            allocated = p.inner_text('.sy-allocations')
            step("structure's frame says what it is allocated, and which way it binds",
                 'structure_mass_allocated = 40 [kg]: structure_mass <= it' in allocated, allocated)

            # W13: a group mounted on solar's leaf, for Ola.
            open_group(p, port, 'l3_solar')
            pick(p, LEAF)
            p.fill('#sy-f-why', WHY)
            p.fill('#sy-m-group', NEW)
            p.fill('#sy-m-owner', 'Test Ola')
            p.click('#sy-mount')
            notice = said_after(p, '')
            step("Sam mounts a group on solar's leaf, for Ola", 'made the edit "mount"' in notice, notice)
            open_group(p, port, NEW)
            top = p.inner_text('.answer-first')
            step('its branch says it is mounted for Ola and not started, and whose step it is',
                 f'{NEW} is mounted for Test Ola, and its file is not started yet' in top
                 and f'Nothing yet: {NEW} is mounted for Test Ola, who starts its file' in top, top)
            step('with Ola, who owns solar, the valve above it',
                 'Test Ola, who owns l3_solar' in p.inner_text('.sy-valve-above'), p.inner_text('.sy-valve-above'))
            step('and it is not Sam\'s to start', p.locator('#sy-start').count() == 0
                 and 'Not yours to start' in p.inner_text('.sy-not-yours'), p.inner_text('#sh-main')[:300])

            # Ola: My work says Start, and its link opens the System workspace.
            p.goto(f'http://127.0.0.1:{port}/app')
            p.wait_for_selector('#sh-close', timeout=60000)
            p.click('#sh-close')
            p.wait_for_selector('#sh-open', timeout=60000)
            open_key(p, 'Test Ola', anchor)
            item = p.locator('.sh-item[data-kind="start"]', has_text='Start l3_struct')
            step('My work says Start l3_struct to its owner', item.count() == 1, p.inner_text('#sh-main')[:400])
            item.locator('a', has_text='open in System').click()
            drawn(p, 'l3_struct')
            step('its link opens the System workspace there, with Start',
                 p.locator('#sy-start').count() == 1, p.inner_text('#sh-main')[:300])
            p.click('#sy-start')
            p.wait_for_selector('.sy-grid .sy-tree [data-block="l3_struct"]', timeout=120000)
            notice = p.inner_text('.sy-notice')
            step('started, its branch is drawn with its one heading',
                 'Test Ola started l3_struct' in notice
                 and p.locator('.sy-tree .sy-node').count() == 1, notice)
            step('and what it is allocated, said beside it',
                 'structure_mass_allocated' in p.inner_text('.sy-allocations'))

            # Solar's leaf carries the group, and Ola's edits of it are not offered.
            open_group(p, port, 'l3_solar')
            pick(p, LEAF)
            blk = p.inner_text('.sy-block')
            step(f'solar\'s leaf says {NEW} hangs there', f'{NEW} hangs here' in blk, blk[:300])
            step('and Ola is offered no edit of it the program would refuse',
                 p.locator('#sy-q').count() == 0 and p.locator('#sy-remove').count() == 0
                 and p.locator('.sy-carried').count() == 1, blk[-300:])
            step('today\'s margins are a link to Explore\'s health map',
                 p.locator('.sy-margins-today[href="#explore/design"]').count() == 1)
            p.wait_for_selector('.sy-budgets-said, .sy-budgets-refused', timeout=15000)
            said = p.inner_text('.sy-budgets')
            step('the budgets across releases are not read from the design\'s own files, and the page says why',
                 p.locator('.sy-budgets-refused').count() == 1
                 and 'budgets are read from the designs released on the drive' in said, said[:300])
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
