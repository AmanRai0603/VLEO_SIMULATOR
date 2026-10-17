#!/usr/bin/env python3
"""The Node workspace, in a real browser on the real program.

docs/briefs/F3.md, F3-g1. A small drive of invented people is written
(`cargo run -p vleo-server --example a_small_drive`), and two instances of the
program are started on it, each with its own keys and working copy, as two
people on two computers that share the drive. Then, as people would:

- Test Pia copies the drive and opens her key; My work lists her node, and
  opens it in the Node workspace;
- the workspace shows its six parts, its page as today's design holds it,
  said so, and its file in her copy: revision 1, hers to write, not signed,
  and the one next step, *Sign revision 1*;
- a request written in the file meanwhile, from another tab, makes her
  signing a second writer: refused on screen, nothing written, and the file
  is opened again only when she asks;
- opened again, she signs it, and the workspace says it is signed and that
  the next step is to put it back;
- she asks for a breakdown, signed, and it is listed in the file;
- My work's *put back* names her node, writes nothing until she confirms,
  then puts it on the drive;
- Test Ola, solar's subsystem engineer, copies the drive on the other
  instance: the node is signed as it is now, not his to write, and both
  requests are there;
- a node of today's design with no file in the copy has its page said to be
  today's design's;
- a node today's design does not hold, and an id that is not one, are each
  said, never drawn as an empty page.

    python3 tools/node_check.py

Exit 0 when every step held; 1 with the steps that did not.
"""
import os
import re
import shutil
import subprocess
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from put_back_check import ROOT, PASSPHRASE, start, ask  # noqa: E402

NODE = 'sw_activity_band'
PATH = 'groups/l3_solar/nodes/sw_activity_band.vnode'


def chromium_path():
    """The browser panel_check already found. One rule, one place."""
    from panel_check import chromium_path as p
    return p()


def open_key(p, who, anchor):
    """Open `who`'s key on the page, accepting the anchor on first use."""
    p.select_option('#sh-keyfile', label=f'{who} · test-{who.split()[-1].lower()}.vkey')
    p.fill('#sh-pass', PASSPHRASE)
    p.click('#sh-open')
    p.wait_for_selector('#sh-anchor-ask, #sh-close', timeout=60000)
    if p.locator('#sh-anchor-ask').count():
        p.fill('#sh-anchor', anchor)
        p.click('#sh-accept')
    p.wait_for_selector('#sh-close', timeout=60000)


def copy_drive(p, drive):
    p.fill('#sh-drive', drive)
    p.click('#sh-copy')
    p.wait_for_selector('#sh-copy-card:has-text("Your copy is at")', timeout=60000)


def workspace(p, timeout=120000):
    """Wait for the Node workspace to have drawn its node."""
    p.wait_for_selector('.nw-grid .nw-right h3', timeout=timeout)


def said_after(p, before):
    """The workspace's notice once it says something other than `before`."""
    p.wait_for_function(
        "b => { const n = document.querySelector('.nw-notice'); return n && n.innerText !== b; }",
        arg=before, timeout=60000)
    return p.inner_text('.nw-notice')


def check():
    from playwright.sync_api import sync_playwright
    fails = []

    def step(what, ok, detail=''):
        print(f"  {'ok  ' if ok else 'FAIL'} {what}{' — ' + str(detail)[:300] if detail else ''}")
        if not ok:
            fails.append(what)

    scratch = tempfile.mkdtemp(prefix='vleo-node-')
    programs = []
    try:
        made = subprocess.run(
            ['cargo', 'run', '-q', '-p', 'vleo-server', '--example', 'a_small_drive', '--', scratch],
            cwd=ROOT, capture_output=True, text=True, check=True)
        anchor = made.stdout.strip().splitlines()[-1]
        drive = os.path.join(scratch, 'drive')
        one_keys, two_keys = os.path.join(scratch, 'keys'), os.path.join(scratch, 'keys-two')
        shutil.copytree(one_keys, two_keys)
        one, p1, w1 = start(scratch, one_keys, os.path.join(scratch, 'copy-one'), 7867)
        programs.append(one)
        two, p2, w2 = start(scratch, two_keys, os.path.join(scratch, 'copy-two'), 7868)
        programs.append(two)
        errors = []
        with sync_playwright() as pw:
            b = pw.chromium.launch(executable_path=chromium_path())
            p = b.new_page(viewport={'width': 1400, 'height': 1000})
            p.on('pageerror', lambda e: errors.append(str(e)))
            p.goto(f'http://127.0.0.1:{p1}/app#session={w1}')
            p.wait_for_selector('#sh-key-card', timeout=60000)
            copy_drive(p, drive)
            open_key(p, 'Test Pia', anchor)

            # My work lists her node, and opens it in the Node workspace.
            mine = p.locator(f'#sh-nodes [data-open-node="{NODE}"]')
            step('My work lists her node', mine.count() == 1, p.inner_text('#sh-main')[:200])
            mine.first.click()
            workspace(p)
            step('it opens in the Node workspace', p.locator('#sh-nav .tab.sel').inner_text() == 'Node')
            p.wait_for_selector('.nw-parts .np-part', timeout=60000)
            step('its six parts are drawn', p.locator('.nw-parts .np-part').count() == 6)
            p.wait_for_selector('.nw-page .sheet-read', timeout=60000)
            centre = p.inner_text('.nw-centre .nw-view.sel').lower()
            step("its page is drawn, said to be today's design's", "today's design" in centre, centre)
            step('once, down its side: the page in the centre draws none of its own',
                 p.locator('.nw-centre .np-part, .nw-centre .np-host').count() == 0)
            right = p.inner_text('.nw-right')
            step('its file in her copy: revision 1, hers to write, not signed',
                 'revision\n1, written by Test Pia' in right and 'yours to write' in right
                 and 'has no signature' in right and 'Sign revision 1' in right, right[:300])
            nxt = p.inner_text('.nw-next')
            step('the one next step is named', nxt == 'Next: Sign revision 1', nxt)

            # A request written meanwhile, from another tab: her signing is a
            # second writer, and nothing is written.
            before = open(os.path.join(scratch, 'copy-one', PATH), 'rb').read()
            opened = ask(p1, f'/v1/node/{NODE}/file')['opened']
            r = ask(p1, f'/v1/node/{NODE}/ask', {'opened': opened, 'kind': 'contract',
                                                 'body': 'The band needs its upper edge as an input.'}, w1)
            step('a request is written in the file from another tab', r.get('ok'), r)
            meanwhile = open(os.path.join(scratch, 'copy-one', PATH), 'rb').read()
            p.click('#nw-sign')
            notice = said_after(p, '')
            step('her signing is refused as a second writer, on screen',
                 'changed in your copy since you opened it' in notice, notice)
            step('and nothing is written',
                 open(os.path.join(scratch, 'copy-one', PATH), 'rb').read() == meanwhile and meanwhile != before)
            step('the file is not opened again behind her back',
                 p.locator('#nw-reopen').count() == 1
                 and 'The band needs its upper edge' not in p.inner_text('.nw-right'),
                 f"{p.locator('#nw-reopen').count()} {p.inner_text('.nw-right')[-300:]}")

            p.click('#nw-reopen')
            workspace(p)
            step('opened again, the request is listed',
                 'The band needs its upper edge' in p.inner_text('.nw-right'))
            p.click('#nw-sign')
            notice = said_after(p, '')
            step('she signs it', 'is signed by Test Pia' in notice, notice)
            step('the workspace says it is signed, and to put it back',
                 'signed as it is now' in p.inner_text('.nw-right')
                 and 'Put your work back' in p.inner_text('.nw-next'), p.inner_text('.nw-next'))

            # A request asked from what she opened before is a second writer:
            # refused, and what she typed is kept.
            opened = ask(p1, f'/v1/node/{NODE}/file')['opened']
            r = ask(p1, f'/v1/node/{NODE}/ask', {'opened': opened, 'kind': 'contract',
                                                 'body': 'Meanwhile, from another tab.'}, w1)
            p.select_option('#nw-ask-kind', 'breakdown')
            p.fill('#nw-ask-body', 'The band is two relations: break it in two.')
            p.click('#nw-ask')
            stopped = said_after(p, notice)
            step('a request from what she opened before is stopped, and what she typed is kept',
                 'Nothing is asked' in stopped
                 and p.input_value('#nw-ask-body') == 'The band is two relations: break it in two.'
                 and p.input_value('#nw-ask-kind') == 'breakdown', stopped)
            p.click('#nw-reopen')
            workspace(p)
            p.click('#nw-ask')
            asked = said_after(p, '')
            step('she asks for a breakdown, signed', 'asks, in' in asked and 'a breakdown' in asked, asked)
            step('and it is listed in the file', 'break it in two' in p.inner_text('.nw-right'))

            # Put back, from My work: named first, written only when confirmed.
            on_drive = open(os.path.join(drive, PATH), 'rb').read()
            p.click('#sh-nav .tab:text-is("My work")')
            p.click('#sh-put-back')
            p.wait_for_selector('#sh-put-back-report', timeout=60000)
            report = p.inner_text('#sh-put-back-report')
            step('put back names her node first, and writes nothing',
                 PATH in report and open(os.path.join(drive, PATH), 'rb').read() == on_drive, report[:300])
            p.click('#sh-put-back-confirm')
            p.wait_for_selector('#sh-notice .sh-said:has-text("back on the drive")', timeout=60000)
            mine = open(os.path.join(scratch, 'copy-one', PATH), 'rb').read()
            step('confirmed, it is on the drive as she signed it',
                 open(os.path.join(drive, PATH), 'rb').read() == mine, p.inner_text('#sh-notice'))
            p.click(f'#sh-nodes [data-open-node="{NODE}"]')
            workspace(p)
            nxt = p.inner_text('.nw-next')
            step('then nothing is left to do, and the old notice is not said again',
                 nxt.startswith('Next: Nothing: revision 1 is signed') and not p.locator('.nw-notice').count(), nxt)

            # A node of today's design with no file in the copy: its page is
            # today's, and the centre says so with no choice of page beside it.
            p.goto(f'http://127.0.0.1:{p1}/app#node/sw_kp_from_ap')
            workspace(p)
            p.wait_for_selector('.nw-page .sheet-read', timeout=60000)
            heading = p.inner_text('.nw-centre h3').lower()
            step("a node with no file in the copy: its page said to be today's design's",
                 "today's design" in heading and p.locator('.nw-views').count() == 0, heading)

            # A node today's design does not hold, and an id that is not one.
            p.goto(f'http://127.0.0.1:{p1}/app#node/no_such_node')
            workspace(p)
            step('a node nobody holds is said, not drawn empty',
                 'holds no node' in p.inner_text('.nw-centre')
                 and 'no node file no_such_node.vnode' in p.inner_text('.nw-right'), p.inner_text('.nw-centre')[:200])
            p.goto(f'http://127.0.0.1:{p1}/app#node/not%20an%20id')
            p.wait_for_selector('.nw-right:has-text("is not a node\'s id")', timeout=60000)
            step('an id that is not one is said', "is not a node's id" in p.inner_text('.nw-right'),
                 p.inner_text('.nw-right')[:200] + ' | ' + p.inner_text('.answer-first'))

            # Test Ola copies the drive on the other instance.
            q = b.new_page(viewport={'width': 1400, 'height': 1000})
            q.on('pageerror', lambda e: errors.append(str(e)))
            q.goto(f'http://127.0.0.1:{p2}/app#session={w2}')
            q.wait_for_selector('#sh-key-card', timeout=60000)
            copy_drive(q, drive)
            open_key(q, 'Test Ola', anchor)
            q.goto(f'http://127.0.0.1:{p2}/app#node/{NODE}')
            workspace(q)
            right = q.inner_text('.nw-right')
            step("on Ola's copy the node is signed as it is now, and not his to write",
                 'signed as it is now' in right and 'Not yours to write' in right, right[:300])
            step('every request is there',
                 'upper edge' in right and 'Meanwhile, from another tab' in right and 'break it in two' in right)
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
