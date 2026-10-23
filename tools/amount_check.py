#!/usr/bin/env python3
"""Amounts typed in the frame's forms, in a real browser on the real program.

docs/briefs/F8.md, F8-a item 7. A small drive of invented people is written
(`cargo run -p vleo-server --example a_small_drive`), and the program is
started on it. Then, as people would:

- Test Ola, who owns solar, adds an open block under solar's leaf
  `sw_activity_count` in the System workspace, and types `1,5 mN` as an
  output's amount: the program
  says on screen why it does not read, and a fill of it writes nothing;
- typed as `55 ± 10% mN`, it is said back as a force, and filled: solar's
  group file in the copy holds its unit, its value and its ends as typed,
  and the answer says what the port now holds;
- named again, the form says what a fill keeps and takes the port's unit,
  and `58` is read in mN and filled, keeping the range and its reason;
- Test Sam, the system engineer, allocates `40 kg` to propulsion's mount,
  bounding the density it is designed for: refused on screen, both
  dimensions named, and nothing written;
- `2.5e-11 to 1.5e-10 kg/m^3` is allocated: the systems' file holds its
  unit and its ends as typed, and no value;
- no page error.

    python3 tools/amount_check.py

Exit 0 when every step held; 1 with the steps that did not.
"""
import hashlib
import os
import shutil
import sqlite3
import subprocess
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from put_back_check import ROOT, start  # noqa: E402
from node_check import chromium_path, open_key, copy_drive  # noqa: E402
from start_group_check import said_after, pick, open_group  # noqa: E402

LEAF = 'sw_activity_count'
BLOCK = 'sw_thruster'
SOLAR = 'groups/l3_solar/l3_solar.vgroup'
SYSTEMS = 'groups/systems/systems.vgroup'
WHY = 'The thrust and the density it is designed for are framed with their bands.'


def digest(at):
    with open(at, 'rb') as f:
        return hashlib.sha256(f.read()).hexdigest()


def held(at, block, name):
    """A port's unit, value and ends, as the file in the copy holds them."""
    db = sqlite3.connect(f'file:{at}?mode=ro', uri=True)
    try:
        return db.execute('SELECT unit, value, lower, upper FROM port WHERE block_uid = ? AND name = ?',
                          (block, name)).fetchone()
    finally:
        db.close()


def said(p, form, typed, unit=''):
    """What the program says of `typed` in the form `form` (sy-p or sy-a)."""
    before = p.inner_text(f'#{form}-said')
    p.fill(f'#{form}-unit', unit)
    p.fill(f'#{form}-amount', typed)
    p.dispatch_event(f'#{form}-amount', 'change')
    p.wait_for_function(
        "([id, b]) => { const n = document.getElementById(id); return n && n.innerText && n.innerText !== b; }",
        arg=[f'{form}-said', before], timeout=60000)
    return p.inner_text(f'#{form}-said')


def check():
    from playwright.sync_api import sync_playwright
    fails = []

    def step(what, ok, detail=''):
        print(f"  {'ok  ' if ok else 'FAIL'} {what}{' — ' + str(detail)[:300] if detail else ''}")
        if not ok:
            fails.append(what)

    scratch = tempfile.mkdtemp(prefix='vleo-amount-')
    programs = []
    try:
        made = subprocess.run(
            ['cargo', 'run', '-q', '-p', 'vleo-server', '--example', 'a_small_drive', '--', scratch],
            cwd=ROOT, capture_output=True, text=True, check=True)
        anchor = made.stdout.strip().splitlines()[-1]
        drive = os.path.join(scratch, 'drive')
        keys = os.path.join(scratch, 'keys')
        prog, port, word = start(scratch, keys, os.path.join(scratch, 'copy'), 7887)
        programs.append(prog)
        solar = os.path.join(scratch, 'copy', SOLAR)
        systems = os.path.join(scratch, 'copy', SYSTEMS)
        errors = []
        with sync_playwright() as pw:
            b = pw.chromium.launch(executable_path=chromium_path())
            p = b.new_page(viewport={'width': 1400, 'height': 1000})
            p.on('pageerror', lambda e: errors.append(str(e)))
            p.goto(f'http://127.0.0.1:{port}/app#session={word}')
            p.wait_for_selector('#sh-key-card', timeout=60000)
            copy_drive(p, drive)
            open_key(p, 'Test Ola', anchor)

            # Ola adds a block under solar's leaf, and fills it with an output
            # typed as an amount.
            open_group(p, port, 'l3_solar')
            pick(p, LEAF)
            p.fill('#sy-why', WHY)
            p.fill('#sy-add-id', BLOCK)
            p.fill('#sy-add-q', 'What thrust holds the orbit against the drag?')
            p.click('#sy-add')
            notice = said_after(p, '')
            step('Ola adds an open block to fill', 'made the edit "add"' in notice, notice)
            pick(p, BLOCK)
            p.fill('#sy-why', WHY)
            p.select_option('#sy-p-dir', 'out')
            p.fill('#sy-p-name', 'thrust')
            text = said(p, 'sy-p', '1,5 mN')
            step('an amount that does not read is said so on screen, with the fix',
                 'comma' in text and p.locator('#sy-p-said.sy-said-refused').count() == 1, text)
            before = digest(solar)
            p.click('#sy-fill-p')
            notice = said_after(p, notice)
            step('and filled, it is refused and nothing is written',
                 'comma' in notice and digest(solar) == before, notice)
            pick(p, BLOCK)
            p.fill('#sy-why', WHY)
            p.select_option('#sy-p-dir', 'out')
            p.fill('#sy-p-name', 'thrust')
            p.fill('#sy-p-reason', "the thruster's datasheet band")
            text = said(p, 'sy-p', '55 ± 10% mN')
            step('a spread is said back as a force, from its digits',
                 text == '55 mN, from 49.5 to 60.5, a Force', text)
            p.click('#sy-fill-p')
            notice = said_after(p, notice)
            step('filled, the program says the amount it wrote',
                 'made the edit "fill"' in notice and 'The amount is 55 mN, from 49.5 to 60.5' in notice, notice)
            step('and what the port now holds, its range with its reason',
                 "Its port thrust now holds the value 55 mN, the range 49.5 to 60.5 mN, because the thruster's datasheet band"
                 in notice, notice)
            got = held(solar, BLOCK, 'thrust')
            step("solar's group file holds its unit, value and ends as typed",
                 got == ('mN', '55', '49.5', '60.5'), got)

            # Filled again: the form says what is kept, and a bare number is
            # read in the port's unit.
            pick(p, BLOCK)
            p.fill('#sy-why', WHY)
            p.select_option('#sy-p-dir', 'out')
            p.fill('#sy-p-name', 'thrust')
            p.dispatch_event('#sy-p-name', 'change')
            kept = p.inner_text('#sy-p-said')
            step('naming a port to fill again says what the fill keeps',
                 'it keeps its value 55 mN, its range 49.5 to 60.5 mN' in kept
                 and p.input_value('#sy-p-unit') == 'mN', kept)
            text = said(p, 'sy-p', '58', p.input_value('#sy-p-unit'))
            step("a bare number is read in the port's unit, and said so",
                 '58 mN' in text and "the field's unit" in text, text)
            p.click('#sy-fill-p')
            notice = said_after(p, notice)
            step('filled, it keeps the range and its reason',
                 "now holds the value 58 mN, the range 49.5 to 60.5 mN, because the thruster's datasheet band" in notice,
                 notice)

            # Sam allocates to propulsion's mount, typed as amounts.
            p.goto(f'http://127.0.0.1:{port}/app')
            p.wait_for_selector('#sh-close', timeout=60000)
            p.click('#sh-close')
            p.wait_for_selector('#sh-open', timeout=60000)
            open_key(p, 'Test Sam', anchor)
            open_group(p, port, 'systems')
            pick(p, 'sys_propulsion')
            p.fill('#sy-f-why', WHY)
            p.fill('#sy-a-name', 'design_density_allocated')
            text = said(p, 'sy-a', '40 kg')
            p.select_option('#sy-a-bounds', 'design_density')
            p.select_option('#sy-a-sense', '<=')
            before = digest(systems)
            p.click('#sy-allocate')
            notice = said_after(p, '')
            step('a mass allocated to bound a density is refused on screen, both dimensions named',
                 'an allocation is of the dimension of the output it bounds' in notice
                 and 'in [kg]' in notice and 'in [kg/m^3]' in notice, notice)
            step('and nothing is written', digest(systems) == before)
            pick(p, 'sys_propulsion')
            p.fill('#sy-f-why', WHY)
            p.fill('#sy-a-name', 'design_density_allocated')
            text = said(p, 'sy-a', '2.5e-11 to 1.5e-10 kg/m^3')
            step('a range is said back as a density', 'MassDensity' in text, text)
            p.fill('#sy-a-reason', 'the solar-maximum band at 250 km')
            p.select_option('#sy-a-bounds', 'design_density')
            p.select_option('#sy-a-sense', '<=')
            p.click('#sy-allocate')
            notice = said_after(p, notice)
            step('allocated, the program says the amount it wrote',
                 'made the edit "allocate"' in notice and 'The amount is' in notice, notice)
            got = held(systems, 'sys_propulsion', 'design_density_allocated')
            step("the systems' file holds its unit and ends as typed, and no value",
                 got == ('kg/m^3', '', '2.5e-11', '1.5e-10'), got)
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
