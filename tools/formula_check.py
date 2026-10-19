#!/usr/bin/env python3
"""A formula typed in the Node workspace, in a real browser on the real program.

docs/briefs/F8.md, F8-a items 4 to 6. A small drive of invented people is
written (`cargo run -p vleo-server --example a_small_drive`), and the program
is started on it. Then, as Test Pia, the band's node engineer, would:

- Pia types, in *type a formula*, `band = f107 / q`: a moment later the
  program says `q` is no input of the node, and *Ask for these inputs* drafts
  a contract change naming it, for Pia to change; nothing is written;
- a save with that formula in its box is refused on screen: a formula is
  saved only as the method it became, seen in *its method*;
- a reading of older typing is withdrawn as soon as the formula changes;
- Pia types `band = f107 ** 0`: the program says it read `**` as `^`, found
  `f107` by its port's name, shows the method it becomes, saying what was
  typed, and runs it on the band's cases, some agreeing and some not;
- *Use it as the method* puts that text into *its method* and clears the
  formula; said who made it and which belief broke, it is saved as revision
  2, and the file's method is the one the formula became;
- no page error.

The band's cases are the release's own; the formula is invented for the
check, to be wrong on most of them.

    python3 tools/formula_check.py

Exit 0 when every step held; 1 with the steps that did not.
"""
import os
import shutil
import subprocess
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from put_back_check import ROOT, start, ask  # noqa: E402
from node_check import chromium_path, open_key, copy_drive, workspace  # noqa: E402
from edit_check import METHOD_WHY, field, said_after  # noqa: E402

NODE = 'sw_activity_band'
PATH = 'groups/l3_solar/nodes/sw_activity_band.vnode'
MISSING = 'band = f107 / q'
TYPED = 'band = f107 ** 0'


def read_after(p, has):
    """The formula's reading once it says `has`."""
    p.wait_for_function(
        "h => { const n = document.querySelector('.nw-formula-read'); return n && n.innerText.includes(h); }",
        arg=has, timeout=60000)
    return p.inner_text('.nw-formula-read')


def check():
    from playwright.sync_api import sync_playwright
    fails = []

    def step(what, ok, detail=''):
        print(f"  {'ok  ' if ok else 'FAIL'} {what}{' — ' + str(detail)[:300] if detail else ''}")
        if not ok:
            fails.append(what)

    scratch = tempfile.mkdtemp(prefix='vleo-formula-')
    programs = []
    try:
        made = subprocess.run(
            ['cargo', 'run', '-q', '-p', 'vleo-server', '--example', 'a_small_drive', '--', scratch],
            cwd=ROOT, capture_output=True, text=True, check=True)
        anchor = made.stdout.strip().splitlines()[-1]
        drive = os.path.join(scratch, 'drive')
        keys = os.path.join(scratch, 'keys')
        one, port, word = start(scratch, keys, os.path.join(scratch, 'copy'), 7889)
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
            p.wait_for_selector('[data-edit="formula"]', timeout=60000)
            before = open(mine, 'rb').read()

            # An input the node has not got: said, and asked for.
            field(p, 'formula').click()
            field(p, 'formula').fill(MISSING)
            read = read_after(p, 'q: no input of this node')
            step('a name no port has is said as no input of the node', 'f107: f107' in read, read)
            step('and Ask for these inputs is offered', p.locator('#nw-ask-inputs').count() == 1, read)
            p.click('#nw-ask-inputs')
            p.wait_for_function(
                "() => { const n = document.getElementById('nw-ask-body'); return n && n.value.includes('q'); }",
                timeout=60000)
            body = p.input_value('#nw-ask-body')
            step('which drafts a contract change naming it, for Pia to change',
                 body.startswith(f'Inputs for {NODE}: q,') and p.input_value('#nw-ask-kind') == 'contract', body)
            step('and nothing is written', open(mine, 'rb').read() == before)
            step('the reading is still shown after the page is drawn again',
                 'q: no input of this node' in p.inner_text('.nw-formula-read'), p.inner_text('.nw-formula-read'))
            # A save with a formula in its box: never saved unseen.
            p.click('#nw-save')
            notice = said_after(p, '')
            step('a save with a formula typed and not used is refused on screen, nothing written',
                 'A formula is typed and not used' in notice and open(mine, 'rb').read() == before, notice)

            # A formula that reads: said back, and run on the cases.
            field(p, 'formula').click()
            field(p, 'formula').fill('band = f107 ** 1')
            read_after(p, 'the method it becomes')
            field(p, 'formula').fill(TYPED)
            step('a reading of older typing is withdrawn as soon as the formula changes',
                 p.locator('#nw-use-formula').count() == 0)
            p.wait_for_function(
                "t => { const n = document.querySelector('.nw-formula-method'); return n && n.innerText.includes(t); }",
                arg='# Typed as: ' + TYPED, timeout=60000)
            read = read_after(p, 'the method it becomes')
            step('its spelling is said written back', '«**» is written «^»' in read, read)
            step('its input is found by its port\'s name', 'f107: f107' in read and 'no input' not in read, read)
            step('the method it becomes says what was typed',
                 '# Typed as: ' + TYPED in p.inner_text('.nw-formula-method'), p.inner_text('.nw-formula-method'))
            p.wait_for_selector('.nw-live .nw-cases', timeout=60000)
            cases = p.inner_text('.nw-live .nw-cases')
            step('its cases are run on it, some agreeing and some not',
                 'agrees' in cases and p.locator('.nw-live .nw-cases tr.nw-differs').count() > 0, cases[:300])
            step('and nothing is written', open(mine, 'rb').read() == before)

            # Used as the method, and saved.
            method = p.inner_text('.nw-formula-method')
            p.click('#nw-use-formula')
            step('Use it as the method puts it in its method and clears the formula',
                 field(p, 'method_text').input_value().strip() == method.strip()
                 and field(p, 'formula').input_value() == '', field(p, 'method_text').input_value()[:200])
            field(p, 'declared.author').fill('Test Pia')
            field(p, 'declared.ai').select_option('none')
            for k, v in METHOD_WHY.items():
                field(p, 'why.' + k).fill(v)
            p.click('#nw-save')
            notice = said_after(p, notice)
            step('saved as revision 2', 'is saved as revision 2' in notice, notice)
            saved = ask(port, f'/v1/node/{NODE}/file')['content']['method_text']
            step("the file's method is the one the formula became", saved.strip() == method.strip(), saved[:200])
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
