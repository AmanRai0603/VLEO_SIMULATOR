#!/usr/bin/env python3
"""Signed work put back on the drive by one program, and found by another.

docs/briefs/F3.md, F3-c. A small drive of invented people is written
(`cargo run -p vleo-server --example a_small_drive`). Two instances of the
program are started, each with its own working copy and its own keys folder,
as two people on two computers that share the drive. Test Pia opens her key
in both. Then:

- the second copies the drive, and My work there finds her node unsigned;
- the first copies it, signs her node, and asks to put back: the report
  names the node, and nothing is written yet;
- asked with the files the report named, the first writes them on the drive;
- the second copies the drive again, and My work there no longer finds the
  node unsigned: her signature counts in what the drive holds now.

    python3 tools/put_back_check.py

Exit 0 when every step held; 1 with the steps that did not.
"""
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import threading
import time
import urllib.parse
import urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PASSPHRASE = 'a long enough passphrase'
URL = re.compile(r'http://127\.0\.0\.1:(\d+)')
NODE = 'sw_activity_band'
PATH = 'groups/l3_solar/nodes/sw_activity_band.vnode'


def start(scratch, keys, copy, port):
    """One instance of the program on its own copy. Returns (program, port, word)."""
    env = dict(os.environ)
    for k in ['VLEO_DRIVE', 'VLEO_DESIGN']:
        env.pop(k, None)
    env.update({
        'VLEO_KEYS': keys,
        'VLEO_COPY': copy,
        'VLEO_CASE': os.path.join(keys, 'case.csv'),
        'VLEO_RESULTS': os.path.join(keys, 'results'),
        'VLEO_LOG': os.path.join(keys, 'log'),
        'VLEO_PORT': str(port),
    })
    program = subprocess.Popen(['cargo', 'run', '-q', '-p', 'vleo-daemon'], cwd=ROOT, env=env,
                               stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    served = None
    deadline = time.time() + 600
    while served is None and time.time() < deadline:
        line = program.stdout.readline()
        if not line and program.poll() is not None:
            raise SystemExit('the program stopped before it served')
        m = URL.search(line)
        if m:
            served = m.group(1)
    if served is None:
        program.kill()
        raise SystemExit(f'the program on {port} did not say where it serves within ten minutes')
    # What it prints after is read and dropped, so a full pipe never stops it.
    threading.Thread(target=lambda: [None for _ in program.stdout], daemon=True).start()
    for _ in range(120):
        try:
            urllib.request.urlopen(f'http://127.0.0.1:{served}/v1/version', timeout=2)
            break
        except OSError:
            time.sleep(0.5)
    page = open(os.path.join(keys, 'open.html')).read()
    word = re.search(r'session=([0-9a-f]+)', page).group(1)
    return program, served, word


def ask(port, path, fields=None, word=''):
    url = f'http://127.0.0.1:{port}{path}'
    if fields is None:
        return json.load(urllib.request.urlopen(url, timeout=60))
    body = urllib.parse.urlencode({'session': word, **fields}).encode()
    req = urllib.request.Request(url, data=body, headers={'content-type': 'application/x-www-form-urlencoded'})
    return json.load(urllib.request.urlopen(req, timeout=60))


def unsigned(port):
    """The nodes My work finds unsigned, as it says them."""
    items = ask(port, '/v1/my-work').get('items', [])
    return [i for i in items if i.get('kind') == 'unsigned']


def check():
    fails = []

    def step(what, ok, detail=''):
        print(f"  {'ok  ' if ok else 'FAIL'} {what}{' — ' + str(detail)[:300] if detail else ''}")
        if not ok:
            fails.append(what)

    scratch = tempfile.mkdtemp(prefix='vleo-put-back-')
    programs = []
    try:
        made = subprocess.run(
            ['cargo', 'run', '-q', '-p', 'vleo-server', '--example', 'a_small_drive', '--', scratch],
            cwd=ROOT, capture_output=True, text=True, check=True)
        anchor = made.stdout.strip().splitlines()[-1]
        drive = os.path.join(scratch, 'drive')
        one_keys, two_keys = os.path.join(scratch, 'keys'), os.path.join(scratch, 'keys-two')
        shutil.copytree(one_keys, two_keys)
        one, p1, w1 = start(scratch, one_keys, os.path.join(scratch, 'copy-one'), 7863)
        programs.append(one)
        two, p2, w2 = start(scratch, two_keys, os.path.join(scratch, 'copy-two'), 7864)
        programs.append(two)
        for port, word in [(p1, w1), (p2, w2)]:
            r = ask(port, '/v1/copy', {'drive': drive}, word)
            step(f'the instance at {port} copies the drive', r.get('ok'), r)
            r = ask(port, '/v1/key/open', {'file': 'test-pia.vkey', 'passphrase': PASSPHRASE,
                                           'accept_anchor': anchor}, word)
            step(f'Test Pia opens her key on the instance at {port}', r.get('open'), r)
        before = unsigned(p2)
        step('the second finds her node unsigned', any(NODE in i.get('node', '') for i in before), before)

        opened = ask(p1, f'/v1/node/{NODE}/file')
        r = ask(p1, f'/v1/node/{NODE}/sign', {'opened': opened.get('opened', '')}, w1)
        step('the first signs her node', r.get('ok') and r.get('signed_by') == 'Test Pia', r)
        on_drive = open(os.path.join(drive, PATH), 'rb').read()
        r = ask(p1, '/v1/put-back', {}, w1)
        step('the report names the node, and nothing is written yet',
             r.get('back') == [PATH] and r.get('written') is False
             and open(os.path.join(drive, PATH), 'rb').read() == on_drive, r)
        named = '\n'.join(f"{f['digest']} {f['path']}" for f in r.get('files', []) if f.get('back'))
        r = ask(p1, '/v1/put-back', {'files': named}, w1)
        step('asked with the files it named, the first writes them on the drive', r.get('written'), r)
        mine = open(os.path.join(scratch, 'copy-one', PATH), 'rb').read()
        step('the drive holds the node as the first signed it',
             open(os.path.join(drive, PATH), 'rb').read() == mine)

        r = ask(p2, '/v1/copy', {'drive': drive}, w2)
        step('the second copies the drive again', r.get('ok'), r)
        after = unsigned(p2)
        step('and finds her node signed', not any(NODE in i.get('node', '') for i in after), after)
        step('its copy holds the node as the first signed it',
             open(os.path.join(scratch, 'copy-two', PATH), 'rb').read() == mine)
    finally:
        for p in programs:
            p.terminate()
            try:
                p.wait(timeout=10)
            except subprocess.TimeoutExpired:
                p.kill()
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
