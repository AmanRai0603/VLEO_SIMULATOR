#!/usr/bin/env python3
"""A table pasted in the Node workspace, in a real browser on the real program.

docs/briefs/F8.md, F8-b item 1. A small drive of invented people is written
(`cargo run -p vleo-server --example a_small_drive`), and the program is
started on it. Then, as Test Pia, the band's node engineer, would:

- Pia pastes, in *paste a table*, the band's own cases as a spreadsheet
  copies them, with tabs and without their origin column: a moment later
  the program says where their expected values come from is to be said,
  and nothing is written;
- she says they come from a paper: the program says how many cases it
  read, each number as typed and as read, and the cases it becomes, and
  runs the band's method on them, every one agreeing;
- a save with that table in its box is refused on screen: a table is saved
  only as the cases it became, seen in *its cases*;
- a CSV file of the same cases, picked as a file, is read into the box and
  read back the same way;
- *Use these as its cases* puts the cases into *its cases* and clears the
  table; said who made them, they are saved as revision 2, and the file's
  cases are the ones the table became;
- no page error.

The band's cases are the release's own, taken from its file on the drive.

    python3 tools/table_check.py

Exit 0 when every step held; 1 with the steps that did not.
"""
import csv
import io
import os
import shutil
import subprocess
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from put_back_check import ROOT, start, ask  # noqa: E402
from node_check import chromium_path, open_key, copy_drive, workspace  # noqa: E402
from edit_check import field, said_after  # noqa: E402

NODE = 'sw_activity_band'
PATH = 'groups/l3_solar/nodes/sw_activity_band.vnode'


def read_after(p, has):
    """The table's reading once it says `has`."""
    p.wait_for_function(
        "h => { const n = document.querySelector('.nw-table-read'); return n && n.innerText.includes(h); }",
        arg=has, timeout=60000)
    return p.inner_text('.nw-table-read')


def check():
    from playwright.sync_api import sync_playwright
    fails = []

    def step(what, ok, detail=''):
        print(f"  {'ok  ' if ok else 'FAIL'} {what}{' — ' + str(detail)[:300] if detail else ''}")
        if not ok:
            fails.append(what)

    scratch = tempfile.mkdtemp(prefix='vleo-table-')
    programs = []
    try:
        made = subprocess.run(
            ['cargo', 'run', '-q', '-p', 'vleo-server', '--example', 'a_small_drive', '--', scratch],
            cwd=ROOT, capture_output=True, text=True, check=True)
        anchor = made.stdout.strip().splitlines()[-1]
        drive = os.path.join(scratch, 'drive')
        keys = os.path.join(scratch, 'keys')
        one, port, word = start(scratch, keys, os.path.join(scratch, 'copy'), 7891)
        programs.append(one)
        mine = os.path.join(scratch, 'copy', PATH)
        errors = []
        with sync_playwright() as pw:
            b = pw.chromium.launch(executable_path=chromium_path())
            p = b.new_page(viewport={'width': 1400, 'height': 1000})
            p.on('pageerror', lambda e: errors.append(str(e)))
            p.goto(f'http://127.0.0.1:{port}/app#session={word}')
            p.wait_for_selector('#sh-key-card', timeout=60000)
            copy_drive(p, drive)
            open_key(p, 'Test Pia', anchor)
            p.goto(f'http://127.0.0.1:{port}/app#node/{NODE}')
            workspace(p)
            p.wait_for_selector('[data-edit="case_paste"]', timeout=60000)
            before = open(mine, 'rb').read()

            # The band's own cases, as a spreadsheet copies them: tabs, and
            # no origin column.
            own = ask(port, f'/v1/node/{NODE}/file')['content']['case_csv']
            rows = list(csv.reader(io.StringIO(own)))
            at = rows[0].index('origin')
            tabs = '\n'.join('\t'.join(c for k, c in enumerate(r) if k != at) for r in rows if r)
            cases = len(rows) - 1

            field(p, 'case_paste').click()
            field(p, 'case_paste').fill(tabs)
            read = read_after(p, "say where these cases' expected values come from")
            step('a table with no origin is refused until its origin is said', 'never filled in for you' in read, read)
            step('and nothing is written', open(mine, 'rb').read() == before)

            field(p, 'case_paste.origin').select_option('paper')
            read = read_after(p, f'{cases} cases read')
            step('said to come from a paper, it is read case by case', f'{cases} cases read' in read, read)
            # Folded while no number changes unit: read whether shown or not.
            cells = p.text_content('.nw-table-cells')
            step('each number as typed and as read', 'line 2, f107: 64 is 64' in cells, cells[:200])
            step('and the cases it becomes are shown',
                 'f107 [1],answer [1],tolerance,refuses,says,origin' in p.inner_text('.nw-table-csv'),
                 p.inner_text('.nw-table-csv')[:200])
            p.wait_for_selector('.nw-live .nw-cases', timeout=60000)
            judged = p.inner_text('.nw-live .nw-cases')
            step("the band's method is run on them, every one agreeing",
                 'agrees' in judged and p.locator('.nw-live .nw-cases tr.nw-differs').count() == 0, judged[:300])
            step('and nothing is written', open(mine, 'rb').read() == before)

            # A save with a table in its box: never saved unseen.
            p.click('#nw-save')
            notice = said_after(p, '')
            step('a save with a table pasted and not used is refused on screen, nothing written',
                 'A table is pasted and not used' in notice and open(mine, 'rb').read() == before, notice)

            # A CSV file of the same cases, picked as a file.
            dropped = os.path.join(scratch, 'band-cases.csv')
            with open(dropped, 'w') as out:
                out.write(own)
            field(p, 'case_paste.origin').select_option('')
            p.set_input_files('#nw-table-file', dropped)
            p.wait_for_function(
                "() => { const n = document.querySelector('[data-edit=\"case_paste\"]'); return n && n.value.includes(','); }",
                timeout=60000)
            read = read_after(p, f'{cases} cases read')
            step("a CSV file picked is read into the table, its own origin column kept", f'{cases} cases read' in read, read)

            # Used as its cases, and saved.
            shown = p.inner_text('.nw-table-csv')
            p.click('#nw-use-table')
            step('Use these as its cases puts them in its cases and clears the table',
                 field(p, 'case_csv').input_value().strip() == shown.strip()
                 and field(p, 'case_paste').input_value() == '', field(p, 'case_csv').input_value()[:200])
            field(p, 'declared.author').fill('Test Pia')
            field(p, 'declared.ai').select_option('none')
            p.click('#nw-save')
            notice = said_after(p, notice)
            step('saved as revision 2', 'is saved as revision 2' in notice, notice)
            saved = ask(port, f'/v1/node/{NODE}/file')['content']['case_csv']
            step("the file's cases are the ones the table became", saved.strip() == shown.strip(), saved[:200])
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
