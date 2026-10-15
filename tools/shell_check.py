#!/usr/bin/env python3
"""The application's shell, and My work, in a real browser on the real program.

docs/briefs/F1.md, PR 3. A small drive of invented people is written
(`cargo run -p vleo-server --example a_small_drive`), the program is started
on it with a keys folder and a copy folder of its own, and the page it opens
(`<keys>/open.html`) is opened the way a person's browser would open it. Then,
as people would:

- the launch's word is taken out of the address at once;
- the database is copied from the drive;
- a key nobody registers opens nothing, says why, and gives what its holder
  hands over; the anchor is shown on first use and accepted only when the
  person types the fingerprint they were given;
- a wrong passphrase is refused in plain words;
- the programme manager registers a subsystem engineer from what he handed
  over, who then finds his role, and his group with nothing sealed, in My
  work; the system engineer finds his;
- the node engineer finds her unsigned node, and the subsystem engineer the
  group with nothing sealed;
- each other workspace says which stage brings it.

    python3 tools/shell_check.py

Exit 0 when every step held; 1 with the steps that did not.
"""
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time
import urllib.request

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PASSPHRASE = 'a long enough passphrase'
URL = re.compile(r'http://127\.0\.0\.1:(\d+)')


def chromium_path():
    """The browser panel_check already found. One rule, one place."""
    from panel_check import chromium_path as p
    return p()


def start(scratch):
    """The drive written, and the program started on it. Returns (program, port, anchor)."""
    made = subprocess.run(
        ['cargo', 'run', '-q', '-p', 'vleo-server', '--example', 'a_small_drive', '--', scratch],
        cwd=ROOT, capture_output=True, text=True, check=True)
    anchor = made.stdout.strip().splitlines()[-1]
    env = dict(os.environ)
    for k in ['VLEO_DRIVE', 'VLEO_DESIGN']:
        env.pop(k, None)
    env.update({
        'VLEO_KEYS': os.path.join(scratch, 'keys'),
        'VLEO_COPY': os.path.join(scratch, 'copy'),
        'VLEO_CASE': os.path.join(scratch, 'case.csv'),
        'VLEO_RESULTS': os.path.join(scratch, 'results'),
        'VLEO_LOG': os.path.join(scratch, 'log'),
        'VLEO_PORT': '7841',
    })
    program = subprocess.Popen(['cargo', 'run', '-q', '-p', 'vleo-daemon'], cwd=ROOT, env=env,
                               stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    port = None
    deadline = time.time() + 600
    while port is None and time.time() < deadline:
        line = program.stdout.readline()
        if not line and program.poll() is not None:
            raise SystemExit('the program stopped before it served')
        m = URL.search(line)
        if m:
            port = m.group(1)
    for _ in range(120):
        try:
            urllib.request.urlopen(f'http://127.0.0.1:{port}/v1/version', timeout=2)
            break
        except OSError:
            time.sleep(0.5)
    return program, port, anchor


def check():
    from playwright.sync_api import sync_playwright

    fails, errors = [], []

    def step(what, ok, detail=''):
        print(f"  {'ok  ' if ok else 'FAIL'} {what}{' — ' + detail if detail else ''}")
        if not ok:
            fails.append(what)

    scratch = tempfile.mkdtemp(prefix='vleo-shell-')
    program, port, anchor = start(scratch)
    try:
        with sync_playwright() as pw:
            exe = chromium_path()
            b = pw.chromium.launch(**({'executable_path': exe} if exe else {}))
            p = b.new_page(viewport={'width': 1280, 'height': 900})
            p.on('pageerror', lambda e: errors.append(str(e)))
            p.goto('file://' + os.path.join(scratch, 'keys', 'open.html'))
            p.wait_for_url(re.compile(r'/app'), timeout=20000)
            p.wait_for_selector('.answer-first', timeout=20000)
            notice = lambda: p.inner_text('#sh-notice')
            main = lambda: p.inner_text('#sh-main')
            who = lambda: p.inner_text('#sh-who')

            step('the launch\'s word is taken out of the address', 'session' not in p.url, p.url)
            step('the shell names its five workspaces',
                 [t.strip() for t in p.locator('#sh-nav .tab').all_inner_texts()] ==
                 ['My work', 'Node', 'System', 'Programme', 'Explore'])
            step('it opens on My work, asking for the database first', 'Copy the database' in main())

            p.fill('#sh-drive', os.path.join(scratch, 'drive'))
            p.click('#sh-copy')
            p.wait_for_selector('.sh-said', timeout=20000)
            step('the database is copied from the drive', 'copied from the drive' in notice(), notice())

            def open_key(file, passphrase=PASSPHRASE):
                p.select_option('#sh-keyfile', file)
                p.fill('#sh-pass', passphrase)
                p.click('#sh-open')
                p.wait_for_timeout(1500)

            # A stranger's key, on the first open of this drive.
            open_key('test-stranger.vkey')
            step('the anchor is shown on first use, with its drive',
                 p.locator('#sh-anchor-ask').count() == 1 and anchor in p.inner_text('#sh-anchor-ask')
                 and 'drive' in p.inner_text('#sh-anchor-ask'))
            step('the anchor field is never filled in for the person', p.input_value('#sh-anchor') == '')
            p.fill('#sh-anchor', '0' * 64)
            p.click('#sh-accept')
            p.wait_for_timeout(1500)
            step('a fingerprint that does not agree accepts nothing',
                 'not the fingerprint START HERE names' in notice(), notice())
            p.fill('#sh-anchor', anchor)
            p.click('#sh-accept')
            p.wait_for_timeout(1500)
            step('a key nobody registers opens nothing, and says why',
                 'opens nothing here' in notice() and 'no file registers this key' in notice(), notice())
            step('and gives what its holder hands over', p.locator('#sh-handover').count() == 1)

            open_key('test-pat.vkey', 'not the passphrase at all')
            step('a wrong passphrase is refused in plain words', 'does not unlock' in notice(), notice())

            # A subsystem engineer the systems' file assigns propulsion to, not yet
            # registered: his key opens nothing, and he hands over what it gives.
            open_key('test-ivo.vkey')
            ivo_key, ivo_proof = p.inner_text('#sh-pub'), p.inner_text('#sh-proof')
            step('a subsystem engineer, not yet registered, has what to hand over',
                 len(ivo_key) == 64 and len(ivo_proof) == 128)

            open_key('test-pat.vkey')
            step('the programme manager opens, and the shell says who he is',
                 'Test Pat' in who() and 'programme manager' in who(), who())
            p.fill('#sh-rname', 'Test Ivo')
            p.select_option('#sh-rrole', 'subsystem engineer')
            p.fill('#sh-rkey', ivo_key)
            p.fill('#sh-rproof', ivo_proof)
            p.click('#sh-register')
            p.wait_for_timeout(1500)
            step('the programme manager registers him',
                 'Test Ivo is registered as subsystem engineer' in notice(), notice())
            step('and People lists him', 'Test Ivo' in p.inner_text('#sh-people'))
            p.click('#sh-close')
            p.wait_for_timeout(800)

            open_key('test-ivo.vkey')
            step('who opens the application on the same folder and finds his role in My work',
                 'Test Ivo' in who() and 'subsystem engineer' in who()
                 and p.locator('.sh-item[data-kind="nothing sealed"]').count() == 1 and 'l3_prop' in main(), main())

            p.click('#sh-close')
            p.wait_for_timeout(800)
            open_key('test-sam.vkey')
            step('the system engineer finds his role', 'Test Sam' in who() and 'system engineer' in who(), who())

            p.click('#sh-close')
            p.wait_for_timeout(800)
            open_key('test-nia.vkey')
            step('the node engineer finds her node unsigned in My work',
                 p.locator('.sh-item[data-kind="unsigned"]').count() == 1 and 'sw_ap_design' in main(), main())

            p.click('#sh-close')
            p.wait_for_timeout(800)
            open_key('test-ola.vkey')
            step('the subsystem engineer finds her group with nothing sealed',
                 p.locator('.sh-item[data-kind="nothing sealed"]').count() == 1 and 'l3_solar' in main(), main())

            for name, stage in [('Node', 'F3'), ('System', 'F4 to F6'), ('Programme', 'F7'), ('Explore', 'F2')]:
                p.click(f'#sh-nav .tab:text-is("{name}")')
                p.wait_for_timeout(200)
                step(f'the {name} workspace says which stage brings it', f'arrives in {stage}' in main(), main())
            step('Explore points at today\'s tool until it arrives', p.locator('#sh-main a[href="/"]').count() == 1)
            b.close()
    finally:
        program.terminate()
        try:
            program.wait(timeout=10)
        except subprocess.TimeoutExpired:
            program.kill()
        shutil.rmtree(scratch, ignore_errors=True)

    for e in errors:
        print(f'  FAIL the page threw: {e}')
    if errors:
        fails.append('the page threw')
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
