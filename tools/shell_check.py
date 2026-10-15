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
- the node engineer finds her unsigned node and not her signed one; the
  subsystem engineer her group with nothing sealed and not the sealed one,
  and her refused group, which the system engineer sees too and the
  programme manager does not;
- a name made of what a page must escape is shown as it is;
- what cannot be read is said, never shown as nothing to do;
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
        print(f"  {'ok  ' if ok else 'FAIL'} {what}{' — ' + detail[:300] if detail else ''}")
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
            p.on('dialog', lambda d: d.accept())
            p.goto('file://' + os.path.join(scratch, 'keys', 'open.html'))
            p.wait_for_url(re.compile(r'/app'), timeout=20000)
            p.wait_for_selector('.answer-first', timeout=20000)
            notice = lambda: p.inner_text('#sh-notice')
            main = lambda: p.inner_text('#sh-main')
            who = lambda: p.inner_text('#sh-who')
            items = lambda kind: [x.inner_text() for x in p.locator(f'.sh-item[data-kind="{kind}"]').all()]

            def drawn():
                return int(p.evaluate('document.body.dataset.drawn || 0'))

            def act(do):
                """Do something, and wait until the page has drawn what came of it."""
                n = drawn()
                do()
                p.wait_for_function(f'Number(document.body.dataset.drawn || 0) > {n}', timeout=30000)

            step('the launch\'s word is taken out of the address', 'session' not in p.url, p.url)
            step('it is kept in this tab only, never in storage that outlives it',
                 p.evaluate('sessionStorage.getItem("vleo-session")') and p.evaluate('localStorage.length') == 0)
            step('the shell names its five workspaces',
                 [t.strip() for t in p.locator('#sh-nav .tab').all_inner_texts()] ==
                 ['My work', 'Node', 'System', 'Programme', 'Explore'])
            step('it opens on My work, asking for the database first', 'Copy the database' in main())

            p.fill('#sh-drive', os.path.join(scratch, 'drive'))
            act(lambda: p.click('#sh-copy'))
            step('the database is copied from the drive', 'copied from the drive' in notice(), notice())

            def open_key(file, passphrase=PASSPHRASE):
                p.select_option('#sh-keyfile', file)
                p.fill('#sh-pass', passphrase)
                act(lambda: p.click('#sh-open'))

            def close_key():
                act(lambda: p.click('#sh-close'))

            # A stranger's key, on the first open of this drive.
            open_key('test-stranger.vkey')
            ask = lambda: p.inner_text('#sh-anchor-ask')
            step('the anchor is shown on first use, with its drive',
                 p.locator('#sh-anchor-ask').count() == 1 and anchor[:8] in ask() and 'drive' in ask())
            step('but never whole, so what is compared is what the person was given', anchor not in ask())
            step('the anchor field is never filled in for the person', p.input_value('#sh-anchor') == '')
            act(lambda: p.click('#sh-accept'))
            step('an empty answer accepts nothing', 'not the fingerprint START HERE names' in notice(), notice())
            p.fill('#sh-anchor', '0' * 64)
            act(lambda: p.click('#sh-accept'))
            step('a fingerprint that does not agree accepts nothing',
                 'not the fingerprint START HERE names' in notice(), notice())
            p.fill('#sh-anchor', anchor)
            act(lambda: p.click('#sh-accept'))
            step('a key nobody registers opens nothing, and says why',
                 'opens nothing here' in notice() and 'no file registers this key' in notice(), notice())
            step('and gives what its holder hands over, with the name it is for',
                 p.locator('#sh-handover').count() == 1 and p.inner_text('#sh-person') == 'Test Stranger')

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
            step('a group below the systems\' file is not the programme manager\'s to hear of',
                 not items('refused'), main())
            step('a folder that is not a group is no one\'s', 'archive' not in main())
            p.fill('#sh-rname', 'Test Ivo')
            p.select_option('#sh-rrole', 'subsystem engineer')
            p.fill('#sh-rkey', ivo_key)
            p.fill('#sh-rproof', ivo_proof)
            act(lambda: p.click('#sh-register'))
            step('the programme manager registers him',
                 'Test Ivo is registered as subsystem engineer' in notice(), notice())
            step('and People lists him', 'Test Ivo' in p.inner_text('#sh-people'))
            close_key()

            open_key('test-ivo.vkey')
            step('who opens the application on the same folder and finds his role in My work',
                 'Test Ivo' in who() and 'subsystem engineer' in who()
                 and any('l3_prop' in x for x in items('nothing sealed')), main())

            close_key()
            open_key('test-sam.vkey')
            step('the system engineer finds his role', 'Test Sam' in who() and 'system engineer' in who(), who())
            step('and a group his file mounts, refused, with why',
                 any('l3_thermal' in x and 'Test Ola' in x for x in items('refused')), main())

            close_key()
            open_key('test-nia.vkey')
            unsigned = items('unsigned')
            step('the node engineer finds her unsigned node, and only that one',
                 len(unsigned) == 1 and 'sw_ap_design' in unsigned[0], main())

            close_key()
            open_key('test-ola.vkey')
            sealed = items('nothing sealed')
            step('the subsystem engineer finds her group with nothing sealed, and not the one sealed',
                 len(sealed) == 1 and 'l3_solar' in sealed[0], main())
            step('a file in releases that is not a release seals nothing', 'l3_solar' in ' '.join(sealed))
            step('and her own group refused', any('l3_thermal' in x for x in items('refused')), main())
            act(lambda: p.select_option('#sh-pfile', 'l3_solar'))
            people = p.inner_text('#sh-people')
            step('a name is shown as it is, never read as the page\'s own',
                 'Test <b>Nell</b> & "Co"' in people and p.locator('#sh-people b').count() == 0, people)

            # What needs her cannot be read: My work says so, never "nothing".
            def reread():
                p.reload()
                p.wait_for_selector('.answer-first', timeout=20000)
                p.wait_for_function('Number(document.body.dataset.drawn || 0) > 0', timeout=30000)
                return p.inner_text('.answer-first')

            start_here = os.path.join(scratch, 'copy', 'START HERE.md')
            kept = open(start_here).read()
            with open(start_here, 'w') as f:
                f.write('# START HERE\n\nno key named here\n')
            said = reread()
            step('what needs you, when it cannot be read, is said, never shown as nothing to do',
                 'What needs you cannot be read' in said and 'Nothing needs you' not in main(), said)
            with open(start_here, 'w') as f:
                f.write(kept)
            with open(os.path.join(scratch, 'copy', 'copied.toml'), 'w') as f:
                f.write('not a record')
            said = reread()
            step('and a copy that cannot be read is said, never shown as no copy',
                 'copy of the database cannot be read' in said, said)

            for name, stage in [('Node', 'F3'), ('System', 'F4 to F6'), ('Programme', 'F7'), ('Explore', 'F2')]:
                act(lambda: p.click(f'#sh-nav .tab:text-is("{name}")'))
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
