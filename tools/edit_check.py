#!/usr/bin/env python3
"""Writing a node in the Node workspace, in a real browser on the real program.

docs/briefs/F3.md, F3-g2a. A small drive of invented people is written
(`cargo run -p vleo-server --example a_small_drive`), and the program is
started on it. Then, as Test Pia, the band's node engineer, would:

- Pia opens `sw_activity_band` in the Node workspace, and its file says it is
  Pia's to write: its question, the explanation's three texts, its method and
  its cases are in the form as the file holds them;
- Pia types in *Said simply*: a moment later the program says a save would
  write revision 2, with nothing written, and what was typed is still there,
  with the caret where it was left;
- Pia breaks the case at 130 sfu in the method: the case is shown
  disagreeing, and a save would be refused, since nothing says whether an
  assistant helped make the method;
- saved like that, it is refused on screen, and the typing is kept;
- Pia puts the method back, asks a new question and says which belief broke:
  saved as revision 2;
- a request written from another tab meanwhile makes the next save a second
  writer: refused on screen, nothing written, the typing kept, and the file
  opened again only when Pia asks; the other tab's *Common misreading*,
  written meanwhile, is taken, never written over; opened again, it is saved
  as revision 4, and Pia signs it.

    python3 tools/edit_check.py

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

NODE = 'sw_activity_band'
PATH = 'groups/l3_solar/nodes/sw_activity_band.vnode'
SAID = 'A reading of 228 means little alone: the band says it is high.'
QUESTION = 'Which level of solar activity does this F10.7 fall in?'
THEIRS = 'Band 4 is not a storm: it is a level of the radio flux, not of the field.'
WHY = {
    'believed': 'The question named the band, not what it is for.',
    'tested': 'Read beside the drag nodes that use it.',
    'learned': 'They read it as the level of activity.',
    'changed': 'The question asks for the level.',
    'rests_on': "NOAA's four F10.7 levels.",
    'breaks_if': 'A node reads it as a flux.',
}


def live_after(p, before):
    """The live answer once it says something other than `before`."""
    p.wait_for_function(
        "b => { const n = document.querySelector('.nw-live .nw-would'); return n && n.innerText !== b; }",
        arg=before, timeout=60000)
    return p.inner_text('.nw-live')


def said_after(p, before):
    p.wait_for_function(
        "b => { const n = document.querySelector('.nw-notice'); return n && n.innerText !== b; }",
        arg=before, timeout=60000)
    return p.inner_text('.nw-notice')


def field(p, name):
    return p.locator(f'[data-edit="{name}"]')


def check():
    from playwright.sync_api import sync_playwright
    fails = []

    def step(what, ok, detail=''):
        print(f"  {'ok  ' if ok else 'FAIL'} {what}{' — ' + str(detail)[:300] if detail else ''}")
        if not ok:
            fails.append(what)

    scratch = tempfile.mkdtemp(prefix='vleo-edit-')
    programs = []
    try:
        made = subprocess.run(
            ['cargo', 'run', '-q', '-p', 'vleo-server', '--example', 'a_small_drive', '--', scratch],
            cwd=ROOT, capture_output=True, text=True, check=True)
        anchor = made.stdout.strip().splitlines()[-1]
        drive = os.path.join(scratch, 'drive')
        keys = os.path.join(scratch, 'keys')
        one, p1, w1 = start(scratch, keys, os.path.join(scratch, 'copy'), 7873)
        programs.append(one)
        mine = os.path.join(scratch, 'copy', PATH)
        errors = []
        with sync_playwright() as pw:
            b = pw.chromium.launch(executable_path=chromium_path())
            p = b.new_page(viewport={'width': 1400, 'height': 1000})
            p.on('pageerror', lambda e: errors.append(str(e)))
            p.goto(f'http://127.0.0.1:{p1}/app#session={w1}')
            p.wait_for_selector('#sh-key-card', timeout=60000)
            copy_drive(p, drive)
            open_key(p, 'Test Pia', anchor)
            p.goto(f'http://127.0.0.1:{p1}/app#node/{NODE}')
            workspace(p)
            p.wait_for_selector('[data-edit="question"]', timeout=60000)
            step('the form holds the node as its file does',
                 field(p, 'question').input_value() == 'Which activity band does this F10.7 fall in?'
                 and 'if f107 >= 130 then' in field(p, 'method_text').input_value()
                 and len(field(p, 'explain_simply').input_value()) > 20
                 and 'origin' in field(p, 'case_csv').input_value(),
                 field(p, 'question').input_value())
            method = field(p, 'method_text').input_value()

            # Typing in Said simply: checked, nothing written, typing kept.
            opened = ask(p1, f'/v1/node/{NODE}/file')['opened']
            before = open(mine, 'rb').read()
            field(p, 'explain_simply').click()
            field(p, 'explain_simply').fill(SAID)
            live = live_after(p, '')
            step('a moment after typing, the program says a save would write revision 2',
                 'A save would write revision 2' in live, live)
            step('and nothing is written', open(mine, 'rb').read() == before
                 and ask(p1, f'/v1/node/{NODE}/file')['opened'] == opened)
            step('what was typed is still there, and the caret with it',
                 field(p, 'explain_simply').input_value() == SAID
                 and p.evaluate("document.activeElement && document.activeElement.dataset.edit") == 'explain_simply')

            # A method that breaks the case at 130 sfu.
            would = p.inner_text('.nw-live .nw-would')
            field(p, 'method_text').fill(method.replace('if f107 >= 130 then', 'if f107 >= 135 then'))
            live = live_after(p, would)
            step('a method that moves an edge shows the case at 130 sfu disagreeing',
                 p.locator('.nw-cases .nw-differs:has-text("130 sfu exactly")').count() == 1, live)
            step('and a save would be refused: whether an assistant helped is not said',
                 'A save would be refused' in live and 'silence is not none' in live, live)

            # Saved like that: refused, typing kept.
            p.click('#nw-save')
            notice = said_after(p, '')
            step('saved like that, it is refused on screen', 'silence is not none' in notice, notice)
            p.wait_for_selector('[data-edit="method_text"]', timeout=60000)
            step('and the typing is kept',
                 'if f107 >= 135 then' in field(p, 'method_text').input_value()
                 and field(p, 'explain_simply').input_value() == SAID)
            step('and nothing is written', open(mine, 'rb').read() == before)

            # The method put back, a new question, and which belief broke.
            field(p, 'method_text').fill(method)
            field(p, 'question').fill(QUESTION)
            live = live_after(p, p.inner_text('.nw-live .nw-would') if p.locator('.nw-live .nw-would').count() else '')
            step('a new question without its record would be refused, said why',
                 'does not say which belief broke' in live, live)
            for k, v in WHY.items():
                field(p, f'why.{k}').fill(v)
            live = live_after(p, p.inner_text('.nw-live .nw-would'))
            step('with its record, a save would write revision 2', 'A save would write revision 2' in live, live)
            p.click('#nw-save')
            notice = said_after(p, notice)
            step('saved as revision 2', 'saved as revision 2' in notice, notice)
            right = p.inner_text('.nw-right')
            step('the file says it: revision 2, written by Test Pia, the new question',
                 'revision\n2, written by Test Pia' in right and field(p, 'question').input_value() == QUESTION,
                 right[:200])

            # A second writer: a request asked from another tab meanwhile.
            opened = ask(p1, f'/v1/node/{NODE}/file')['opened']
            r = ask(p1, f'/v1/node/{NODE}/ask', {'opened': opened, 'kind': 'contract',
                                                 'body': 'The band needs its upper edge as an input.'}, w1)
            step('a request is asked from another tab', r.get('ok'), r)
            # And the common misreading written there too, as revision 3.
            opened = ask(p1, f'/v1/node/{NODE}/file')['opened']
            r = ask(p1, f'/v1/node/{NODE}/edit', {'opened': opened, 'explain_wrong': THEIRS}, w1)
            step('the common misreading is written from another tab, as revision 3',
                 r.get('written') and r.get('revision') == 3, r)
            meanwhile = open(mine, 'rb').read()
            field(p, 'explain_breaks').fill('Above 300 sfu the four bands say nothing more.')
            p.click('#nw-save')
            notice = said_after(p, notice)
            step("Pia's save is a second writer, refused on screen",
                 'changed in your copy since you opened it' in notice, notice)
            step('and nothing is written, and the typing is kept',
                 open(mine, 'rb').read() == meanwhile
                 and field(p, 'explain_breaks').input_value() == 'Above 300 sfu the four bands say nothing more.')
            step("the file is not opened again behind Pia's back", p.locator('#nw-reopen').count() == 1)
            p.click('#nw-reopen')
            workspace(p)
            p.wait_for_selector('[data-edit="explain_breaks"]', timeout=60000)
            step('opened again, the typing is still there, and what the other tab wrote is taken',
                 field(p, 'explain_breaks').input_value() == 'Above 300 sfu the four bands say nothing more.'
                 and field(p, 'explain_wrong').input_value() == THEIRS,
                 field(p, 'explain_wrong').input_value())
            p.click('#nw-save')
            notice = said_after(p, notice)
            step('saved as revision 4', 'saved as revision 4' in notice, notice)
            content = ask(p1, f'/v1/node/{NODE}/file')['content']['explain']
            step("and the other tab's common misreading is not written over",
                 content['wrong'] == THEIRS and content['breaks'] == 'Above 300 sfu the four bands say nothing more.',
                 content)
            p.click('#nw-sign')
            notice = said_after(p, notice)
            step('and Pia signs it', 'is signed by Test Pia' in notice, notice)
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
