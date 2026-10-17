#!/usr/bin/env python3
"""The System workspace's breakdown, in a real browser on the real program.

docs/briefs/F4.md, F4-b. A small drive of invented people is written
(`cargo run -p vleo-server --example a_small_drive`), and two instances of the
program are started on it, each with its own keys and working copy, as two
people on two computers that share the drive. Solar's branch holds
`sw_activity_band`, issued, and two children of it: one carrying the band's
own method, one whose elevated edge is at 135 sfu, invented to be wrong.
Then, as people would:

- Test Pia, the band's node engineer, signs it on the other instance and puts
  it back: a node's cases are evidence for its breakdown only once its node
  engineer has signed them;
- Test Ola, who owns solar, copies the drive and opens its branch in the
  System workspace: the tree, said to be their copy, and theirs to frame;
- they add a block under the band and fill its question, and its history
  says who, when and why; an edit with no why is refused;
- broken down into the child whose edge is wrong, it is refused on screen,
  naming the case at 130 sfu, and the group file is not written;
- broken down into the child that reproduces it, it is accepted, and every
  case is shown agreeing beside its estimate; folded back, the estimate
  answers again and both children stay;
- the block they added is removed: archived, shown struck through;
- an edit from what they opened before, after the file changed from another
  tab, is a second writer: refused, nothing written, opened again on a click;
- Test Pia, a node engineer of solar, on the other instance, reads the
  branch with no edit forms, said why; an edit of Pia's sent anyway is refused by
  the program, by name;
- a group no file mounts is said, never drawn as an empty tree.

    python3 tools/branch_check.py

Exit 0 when every step held; 1 with the steps that did not.
"""
import hashlib
import os
import shutil
import subprocess
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from put_back_check import ROOT, PASSPHRASE, start, ask  # noqa: E402
from node_check import chromium_path, open_key, copy_drive  # noqa: E402

GROUP = 'l3_solar'
BAND = 'sw_activity_band'
GOOD = 'sw_activity_count'
LATE = 'sw_activity_count_late'
ADDED = 'sw_activity_note'
PATH = f'groups/{GROUP}/{GROUP}.vgroup'
WHY = 'The band is counted from its edges; the count is a node of its own.'


def digest(at):
    with open(at, 'rb') as f:
        return hashlib.sha256(f.read()).hexdigest()


def branch(p):
    """Wait for the System workspace to have drawn a branch."""
    p.wait_for_selector('.sy-grid .sy-block h3', timeout=120000)


def said_after(p, before):
    """The workspace's notice once it says something other than `before`."""
    p.wait_for_function(
        "b => { const n = document.querySelector('.sy-notice'); return n && n.innerText !== b; }",
        arg=before, timeout=60000)
    return p.inner_text('.sy-notice')


def pick(p, uid):
    p.click(f'.sy-tree [data-block="{uid}"]')
    p.wait_for_selector(f'.sy-tree [data-block="{uid}"].sel', timeout=60000)


def check():
    from playwright.sync_api import sync_playwright
    fails = []

    def step(what, ok, detail=''):
        print(f"  {'ok  ' if ok else 'FAIL'} {what}{' — ' + str(detail)[:300] if detail else ''}")
        if not ok:
            fails.append(what)

    scratch = tempfile.mkdtemp(prefix='vleo-branch-')
    programs = []
    try:
        made = subprocess.run(
            ['cargo', 'run', '-q', '-p', 'vleo-server', '--example', 'a_small_drive', '--', scratch],
            cwd=ROOT, capture_output=True, text=True, check=True)
        anchor = made.stdout.strip().splitlines()[-1]
        drive = os.path.join(scratch, 'drive')
        one_keys, two_keys = os.path.join(scratch, 'keys'), os.path.join(scratch, 'keys-two')
        shutil.copytree(one_keys, two_keys)
        one, p1, w1 = start(scratch, one_keys, os.path.join(scratch, 'copy-one'), 7869)
        programs.append(one)
        two, p2, w2 = start(scratch, two_keys, os.path.join(scratch, 'copy-two'), 7870)
        programs.append(two)
        mine = os.path.join(scratch, 'copy-one', PATH)

        # Pia signs the band on the other instance, and puts it back.
        r = ask(p2, '/v1/copy', {'drive': drive}, w2)
        step('Pia copies the drive on the other instance', r.get('ok'), r)
        r = ask(p2, '/v1/key/open', {'file': 'test-pia.vkey', 'passphrase': PASSPHRASE,
                                     'accept_anchor': anchor}, w2)
        step('and opens their key', r.get('open'), r)
        opened = ask(p2, f'/v1/node/{BAND}/file').get('opened', '')
        r = ask(p2, f'/v1/node/{BAND}/sign', {'opened': opened}, w2)
        step('Pia signs the band', r.get('ok') and r.get('signed_by') == 'Test Pia', r)
        r = ask(p2, '/v1/put-back', {}, w2)
        named = '\n'.join(f"{f['digest']} {f['path']}" for f in r.get('files', []) if f.get('back'))
        r = ask(p2, '/v1/put-back', {'files': named}, w2)
        step('and puts it back on the drive', r.get('written'), r)
        errors = []
        with sync_playwright() as pw:
            b = pw.chromium.launch(executable_path=chromium_path())
            p = b.new_page(viewport={'width': 1400, 'height': 1000})
            p.on('pageerror', lambda e: errors.append(str(e)))
            p.goto(f'http://127.0.0.1:{p1}/app#session={w1}')
            p.wait_for_selector('#sh-key-card', timeout=60000)
            copy_drive(p, drive)
            open_key(p, 'Test Ola', anchor)

            # Ola's branch, in the System workspace.
            p.click('#sh-nav .tab:text-is("System")')
            p.wait_for_selector(f'[data-group-open="{GROUP}"]', timeout=60000)
            step('the System workspace offers the branches Ola owns, solar among them',
                 p.locator(f'[data-group-open="{GROUP}"]').count() == 1, p.inner_text('#sh-main')[:200])
            p.click(f'[data-group-open="{GROUP}"]')
            branch(p)
            top = p.inner_text('.answer-first')
            step('it says it shows their copy, and that they frame it',
                 'your copy' in top and 'You frame it.' in top, top[:300])
            step('the band and its two children are in the tree, under it',
                 all(p.locator(f'.sy-tree [data-block="{BAND}"] ~ ul [data-block="{c}"]').count() == 1
                     for c in [GOOD, LATE]), p.inner_text('.sy-tree')[:300])

            # A block added under the band, and its question filled.
            pick(p, BAND)
            p.fill('#sy-add-id', ADDED)
            p.click('#sy-add')
            notice = said_after(p, '')
            step('an edit with no why is refused, on screen', 'why' in notice.lower() and ADDED not in p.inner_text('.sy-tree'),
                 notice)
            p.fill('#sy-why', WHY)
            p.fill('#sy-add-id', ADDED)
            p.click('#sy-add')
            notice = said_after(p, notice)
            step('with a why, the block is added, open', 'made the edit "add"' in notice
                 and p.locator(f'.sy-tree [data-block="{ADDED}"]').count() == 1, notice)
            pick(p, ADDED)
            step('what is still open in it is named', 'its question' in p.inner_text('.sy-open-parts'),
                 p.inner_text('.sy-open-parts'))
            p.fill('#sy-why', WHY)
            p.fill('#sy-q', 'What does the band leave out?')
            p.click('#sy-fill-q')
            notice = said_after(p, notice)
            history = p.inner_text('.sy-history')
            step('its question is filled, and its history says who and why',
                 'made the edit "fill"' in notice and 'Test Ola' in history and WHY in history, history)

            # Broken down into the child whose edge is wrong: refused.
            pick(p, BAND)
            before = digest(mine)
            p.fill('#sy-why', WHY)
            p.select_option('.sy-bd-row[data-bd-out="band"] .sy-bd-child', LATE)
            p.click('#sy-break-down')
            notice = said_after(p, notice)
            step('broken down into the child whose edge is at 135 sfu, it is refused, naming the case at 130 sfu',
                 '130 sfu exactly' in notice and LATE in notice, notice)
            step('and the group file is not written', digest(mine) == before)

            # Into the child that reproduces it: accepted, its cases shown.
            p.fill('#sy-why', WHY)
            p.select_option('.sy-bd-row[data-bd-out="band"] .sy-bd-child', GOOD)
            p.click('#sy-break-down')
            notice = said_after(p, notice)
            step('broken down into the child that reproduces it, it is accepted',
                 'made the edit "break down"' in notice and 'reproduce all 10' in notice, notice)
            rows = p.locator('.sy-cases tbody tr')
            step('every case is shown, agreeing beside its estimate',
                 rows.count() == 10 and p.locator('.sy-cases .sy-differs').count() == 0
                 and '130 sfu exactly' in p.inner_text('.sy-cases'), p.inner_text('.sy-block')[:300])
            step('the tree marks it read as its children',
                 'children reproduce it' in p.inner_text('.sy-tree'), p.inner_text('.sy-tree')[:300])

            # Folded back.
            p.fill('#sy-why', 'The count waits for its own cases.')
            p.click('#sy-fold-back')
            notice = said_after(p, notice)
            step('folded back, its estimate answers again, and both children stay',
                 'made the edit "fold back"' in notice and p.locator('.sy-cases').count() == 0
                 and all(p.locator(f'.sy-tree [data-block="{c}"]').count() == 1 for c in [GOOD, LATE]), notice)

            # The block Ola added, removed: archived.
            pick(p, ADDED)
            p.fill('#sy-why', 'Nothing is left out that the band does not say.')
            p.click('#sy-remove')
            notice = said_after(p, notice)
            step('the block Ola added is removed: archived, shown struck through',
                 'made the edit "remove"' in notice
                 and p.locator(f'.sy-tree [data-block="{ADDED}"].sy-archived').count() == 1, notice)

            # A second writer: the file changed from another tab meanwhile.
            pick(p, BAND)
            opened = ask(p1, f'/v1/branch?group={GROUP}')['opened']
            r = ask(p1, '/v1/branch/add', {'group': GROUP, 'opened': opened, 'why': WHY,
                                            'parent': BAND, 'id': 'sw_activity_meanwhile'}, w1)
            step('a block is added from another tab', r.get('ok'), r)
            meanwhile = digest(mine)
            p.fill('#sy-why', WHY)
            p.fill('#sy-add-id', 'sw_activity_late_tab')
            p.click('#sy-add')
            notice = said_after(p, notice)
            step('an edit from what Ola opened before is a second writer, refused on screen',
                 'changed in your copy since you opened it' in notice, notice)
            step('and nothing is written', digest(mine) == meanwhile)
            step('the branch is not opened again behind their back',
                 p.locator('#sy-reopen').count() == 1
                 and 'sw_activity_meanwhile' not in p.inner_text('.sy-tree'), p.inner_text('.sy-tree')[:300])
            p.click('#sy-reopen')
            branch(p)
            step('opened again, the block added meanwhile is there',
                 'sw_activity_meanwhile' in p.inner_text('.sy-tree'))

            # Test Pia, on the other instance: not Pia's to frame.
            q = b.new_page(viewport={'width': 1400, 'height': 1000})
            q.on('pageerror', lambda e: errors.append(str(e)))
            # Pia's copy and key are open on it already, from signing.
            q.goto(f'http://127.0.0.1:{p2}/app#session={w2}')
            q.wait_for_selector('#sh-close', timeout=60000)
            q.goto(f'http://127.0.0.1:{p2}/app#system/{GROUP}')
            branch(q)
            pick(q, BAND)
            block = q.inner_text('.sy-block')
            step('to Pia it is read, with no edit forms, and said why',
                 q.locator('#sy-why').count() == 0 and q.locator('.sy-not-yours').count() == 1
                 and 'Framed by Test Ola' in q.inner_text('.answer-first')
                 and 'Not yours to frame' in q.inner_text('.sy-not-yours'), block[-300:])
            pias = os.path.join(scratch, 'copy-two', PATH)
            before = digest(pias)
            opened = ask(p2, f'/v1/branch?group={GROUP}')['opened']
            r = ask(p2, '/v1/branch/add', {'group': GROUP, 'opened': opened, 'why': WHY,
                                            'parent': BAND, 'id': 'sw_activity_pia'}, w2)
            step("an edit of Pia's sent anyway is refused by the program, by name",
                 not r.get('ok') and 'Test Pia' in r.get('message', '') and digest(pias) == before, r)

            # A group no file mounts.
            q.goto(f'http://127.0.0.1:{p2}/app#system/no_such_group')
            q.wait_for_selector('.answer-first:has-text("no_such_group")', timeout=60000)
            step('a group no file mounts is said, not drawn as an empty tree',
                 'no file mounts the group "no_such_group"' in q.inner_text('#sh-main')
                 and q.locator('.sy-tree').count() == 0, q.inner_text('#sh-main')[:300])
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
