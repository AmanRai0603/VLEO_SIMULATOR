#!/usr/bin/env python3
"""The application's shell, My work and Explore, in a real browser on the real program.

docs/briefs/F1.md, PR 3, and docs/briefs/F2.md. A small drive of invented people is written
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
  over, who then finds his role in My work, and his group, which has no file
  yet; the system engineer finds his;
- the node engineer finds her unsigned node and not her signed one; the
  subsystem engineer her group with nothing sealed and not the sealed one,
  and her refused group, which the system engineer sees too and the
  programme manager does not;
- a name made of what a page must escape is shown as it is;
- what cannot be read is said, never shown as nothing to do;
- Explore says which design is open, by its fingerprint; a release the chain
  does not admit is refused on screen, and its group's last good one taken
  and marked, with who sealed it and how its seal checked;
- a failing closure is traced to the node the release taken changed, and
  raised as an issue: the file is in the copy's issues/, addressed to its
  group and its node, signed by whoever raised it;
- a run on today's design is kept, and the result names the design it ran
  on, by its fingerprint, and the engine;
- a second instance on the same drive builds the same today's design, by
  fingerprint;
- each other workspace says which stage brings it.

    python3 tools/shell_check.py

Exit 0 when every step held; 1 with the steps that did not.
"""
import os
import re
import json
import shutil
import sqlite3
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


def start(scratch, second=''):
    """The drive written, and the program started on it, today's design built
    from it. Returns (program, port, anchor). `second` starts another instance
    on the same drive, with folders of its own."""
    if not second:
        made = subprocess.run(
            ['cargo', 'run', '-q', '-p', 'vleo-server', '--example', 'a_small_drive', '--', scratch],
            cwd=ROOT, capture_output=True, text=True, check=True)
        anchor = made.stdout.strip().splitlines()[-1]
    else:
        anchor = ''
    env = dict(os.environ)
    env.pop('VLEO_DESIGN', None)
    own = os.path.join(scratch, second) if second else scratch
    os.makedirs(own, exist_ok=True)
    env.update({
        'VLEO_DRIVE': os.path.join(scratch, 'drive'),
        'VLEO_KEYS': os.path.join(own, 'keys'),
        'VLEO_COPY': os.path.join(own, 'copy'),
        'VLEO_CASE': os.path.join(own, 'case.csv'),
        'VLEO_RESULTS': os.path.join(own, 'results'),
        'VLEO_LOG': os.path.join(own, 'log'),
        'VLEO_PORT': '7842' if second else '7841',
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


def explore(p, step, scratch, port):
    """Explore, as Test Ola, whose key is open: F2's proofs on screen."""
    p.click('#sh-nav .tab:text-is("Explore")')
    p.wait_for_selector('.ex-releases', timeout=120000)
    which = p.inner_text('.ex-which')
    fp = p.get_attribute('.ex-fp', 'title') or ''
    step('Explore says which design is open, by its fingerprint',
         which.startswith("today's design, built from the drive") and len(fp) == 64, which + ' ' + fp)
    step('and the released design it is based on', 'none has been released yet' in p.inner_text('.ex-base'))
    row = p.inner_text('.ex-releases tr[data-group="l3_solar"]')
    step('a release the chain does not admit is refused on screen',
         '1.3 refused' in row and 'does not check through the chain' in row, row)
    step('its group\'s last good one is taken, and marked', '1.2' in row and 'its last good one' in row, row)
    step('with who sealed it, and how its seal checked',
         ' by ' in row and 'anchored by the programme manager' in row, row)
    step('and the anchor this computer accepted when a key was opened is said as accepted',
         'has not accepted' not in row, row)

    # The closure 1.2 breaks, traced and raised.
    p.click('.ex-trace[data-closure="l3_solar_ach_01"]')
    p.wait_for_selector('.ex-trace-out h4', timeout=60000)
    traced = p.inner_text('.ex-trace-out')
    step('a failing closure is traced to the node the release taken changed',
         'fails' in traced and 'sw_f107_design_long' in traced and 'changed by l3_solar 1.2' in traced, traced)
    p.click('.ex-trace-out h4 .ex-raise')
    p.fill('.ex-raising .ex-what', 'The first solar closure fails since solar 1.2.')
    p.click('.ex-raising .ex-raise-go')
    p.wait_for_selector('.ex-raised, .ex-raise-refused', timeout=30000)
    said = p.inner_text('.ex-raising')
    step('and raised as an issue', p.locator('.ex-raised').count() == 1 and 'issue 1 is raised on l3_solar' in said, said)
    issue = os.path.join(scratch, 'copy', 'issues', '1-l3_solar.vissue')
    held = {}
    if os.path.isfile(issue):
        db = sqlite3.connect(issue)
        held = dict(db.execute('SELECT key, value FROM meta').fetchall())
        held['row'] = db.execute('SELECT group_id, place, raised_by, evidence FROM issue').fetchone()
        held['signer'] = db.execute("SELECT signer FROM signature WHERE scope = 'file'").fetchone()
        db.close()
    row = held.get('row') or ('', '', '', '')
    step('the issue file is in the copy, addressed to its group and its node, with the evidence',
         row[0] == 'l3_solar' and row[1] == 'l3_solar_ach_01' and 'caused by sw_f107_design_long' in row[3]
         and fp in row[3], str(held)[:300])
    step('signed by whoever raised it',
         row[2] == 'Test Ola' and held.get('writer') == 'Test Ola' and held.get('signer') == ('Test Ola',), str(held)[:300])
    step('and not on the drive until the work goes back',
         not os.path.exists(os.path.join(scratch, 'drive', 'issues')))

    # A run on today's design, kept, and listed with its design and engine.
    p.click('.ex-ws[data-view="run"]')
    p.wait_for_selector('.run-target', timeout=60000)
    p.select_option('.run-target', 'l3_solar_ach_01')
    p.wait_for_selector('.run-go:not([disabled])', timeout=30000)
    p.click('.run-go')
    p.wait_for_selector('.res-save', timeout=120000)
    p.click('.res-save')
    p.wait_for_selector('.res-saved [data-results]', timeout=60000)
    p.click('.res-saved [data-results]')
    p.wait_for_selector('.res-list', timeout=60000)
    listed = p.inner_text('.res-list')
    step('a run on today\'s design is kept, naming the design and the engine',
         "today's design, built from the drive" in listed and 'this design' in listed and 'kernel' in listed, listed)
    p.wait_for_selector('.res-on', timeout=30000)
    step('and the result says the design by its whole fingerprint', fp in p.inner_text('.res-on'))

    # Another instance on the same drive builds the same design.
    second, port2, _ = start(scratch, second='second')
    try:
        there = json.load(urllib.request.urlopen(f'http://127.0.0.1:{port2}/v1/today', timeout=60))
        here = json.load(urllib.request.urlopen(f'http://127.0.0.1:{port}/v1/today', timeout=60))
        step('a second instance on the same drive builds the same today\'s design, by fingerprint',
             there['design']['fingerprint'] == here['design']['fingerprint'] == fp, there['design']['fingerprint'])
    finally:
        second.terminate()
        try:
            second.wait(timeout=10)
        except subprocess.TimeoutExpired:
            second.kill()


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
            # Mounted for him and allocated nothing yet: not his to start until
            # the system engineer gives it what it must reach or may use (F6's
            # question 43).
            step('who opens the application on the same folder and finds his role in My work',
                 'Test Ivo' in who() and 'subsystem engineer' in who()
                 and any('l3_prop' in x and 'allocates l3_prop nothing yet' in x for x in items('not started'))
                 and not any('l3_prop' in x for x in items('start')), main())

            close_key()
            open_key('test-sam.vkey')
            step('the system engineer finds his role', 'Test Sam' in who() and 'system engineer' in who(), who())
            step('and a group his file mounts, refused, with why',
                 any('l3_thermal' in x and 'Test Ola' in x for x in items('refused')), main())
            step('and the groups mounted and allocated nothing, his to allocate',
                 any('Allocate l3_prop' in x for x in items('allocate'))
                 and any('Allocate l3_struct' in x for x in items('allocate')), main())

            close_key()
            open_key('test-nia.vkey')
            unsigned = items('unsigned')
            step('the node engineer finds her unsigned node, and only that one',
                 len(unsigned) == 1 and 'sw_ap_design' in unsigned[0], main())

            close_key()
            open_key('test-ola.vkey')
            sealed = items('nothing sealed')
            step('the subsystem engineer finds her group with nothing sealed, and not the one sealed',
                 len(sealed) == 1 and 'l3_power' in sealed[0], main())
            step('a file in releases that is not a release seals nothing', 'l3_power' in ' '.join(sealed))
            step('and her own group refused', any('l3_thermal' in x for x in items('refused')), main())
            # Mounted for them and not started: their step, said as that, never
            # as a refusal (F6-a).
            step('and a mounted group with no file yet, said as theirs once it is allocated, not passed over',
                 any('l3_struct' in x and 'allocates l3_struct nothing yet' in x for x in items('not started'))
                 and not any('l3_struct' in x for x in items('refused')), main())
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
            record = os.path.join(scratch, 'copy', 'copied.toml')
            kept_record = open(record).read()
            with open(record, 'w') as f:
                f.write('not a record')
            said = reread()
            step('and a copy that cannot be read is said, never shown as no copy',
                 'copy of the database cannot be read' in said, said)
            with open(record, 'w') as f:
                f.write(kept_record)
            reread()

            explore(p, step, scratch, port)

            # The Node workspace is tools/node_check.py's; System is tools/branch_check.py's.
            for name, stage in [('Programme', 'F7')]:
                act(lambda: p.click(f'#sh-nav .tab:text-is("{name}")'))
                step(f'the {name} workspace says which stage brings it', f'arrives in {stage}' in main(), main())
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
