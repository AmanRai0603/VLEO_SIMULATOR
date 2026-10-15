#!/usr/bin/env python3
"""A node saved in place by one program, and a second writer on another stopped.

docs/briefs/F3.md, F3-a. A small drive of invented people is written
(`cargo run -p vleo-server --example a_small_drive`), and two instances of the
program are started on one working copy, each with its own keys folder, as
two windows or two computers on a synced folder would be. Test Nia opens her
key in both, and both open her node. Then:

- the second instance saves first, as revision 2;
- the first instance's save, from what it opened before that, is refused as a
  second writer: it answers both versions, and nothing is written;
- the file in the copy is the second instance's, byte for byte;
- the first instance opens the node again, and its save then goes in, as
  revision 3.

    python3 tools/save_check.py

Exit 0 when every step held; 1 with the steps that did not.
"""
import base64
import json
import os
import re
import shutil
import sqlite3
import subprocess
import sys
import tempfile
import time
import urllib.parse
import urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PASSPHRASE = 'a long enough passphrase'
URL = re.compile(r'http://127\.0\.0\.1:(\d+)')
NODE = 'sw_ap_design'


def start(scratch, keys, port):
    """One instance of the program on the shared copy. Returns (program, port, word)."""
    env = dict(os.environ)
    for k in ['VLEO_DRIVE', 'VLEO_DESIGN']:
        env.pop(k, None)
    env.update({
        'VLEO_KEYS': keys,
        'VLEO_COPY': os.path.join(scratch, 'copy'),
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


def edited(file_b64, line):
    """The node file, with a note of the writer's own, as base64."""
    fd, at = tempfile.mkstemp(suffix='.vnode')
    os.close(fd)
    try:
        with open(at, 'wb') as f:
            f.write(base64.b64decode(file_b64))
        db = sqlite3.connect(at)
        uid = db.execute("SELECT value FROM meta WHERE key = 'block_uid'").fetchone()[0]
        db.execute("DELETE FROM text WHERE scope = ? AND kind = 'note'", (uid,))
        db.execute("INSERT INTO text (scope, kind, body) VALUES (?, 'note', ?)", (uid, line))
        db.commit()
        db.close()
        return base64.b64encode(open(at, 'rb').read()).decode()
    finally:
        os.remove(at)


def check():
    fails = []

    def step(what, ok, detail=''):
        print(f"  {'ok  ' if ok else 'FAIL'} {what}{' — ' + str(detail)[:300] if detail else ''}")
        if not ok:
            fails.append(what)

    scratch = tempfile.mkdtemp(prefix='vleo-save-')
    made = subprocess.run(
        ['cargo', 'run', '-q', '-p', 'vleo-server', '--example', 'a_small_drive', '--', scratch],
        cwd=ROOT, capture_output=True, text=True, check=True)
    anchor = made.stdout.strip().splitlines()[-1]
    one_keys, two_keys = os.path.join(scratch, 'keys'), os.path.join(scratch, 'keys-two')
    shutil.copytree(one_keys, two_keys)
    programs = []
    try:
        one, p1, w1 = start(scratch, one_keys, 7861)
        programs.append(one)
        r = ask(p1, '/v1/copy', {'drive': os.path.join(scratch, 'drive')}, w1)
        step('the database is copied from the drive', r.get('ok'), r)
        two, p2, w2 = start(scratch, two_keys, 7862)
        programs.append(two)
        for port, word in [(p1, w1), (p2, w2)]:
            r = ask(port, '/v1/key/open', {'file': 'test-nia.vkey', 'passphrase': PASSPHRASE,
                                           'accept_anchor': anchor}, word)
            step(f'Test Nia opens her key on the instance at {port}', r.get('open'), r)
        a = ask(p1, f'/v1/node/{NODE}/file')
        b = ask(p2, f'/v1/node/{NODE}/file')
        step('both instances open her node, as one file', a.get('ok') and a.get('opened') == b.get('opened'), (a, b))

        r = ask(p2, f'/v1/node/{NODE}/save', {'opened': b['opened'], 'file': edited(b['file'], 'from the second')}, w2)
        step('the second instance saves first, as revision 2', r.get('ok') and r.get('revision') == 2, r)
        saved = open(os.path.join(scratch, 'copy', a['path']), 'rb').read()

        mine = edited(a['file'], 'from the first')
        r = ask(p1, f'/v1/node/{NODE}/save', {'opened': a['opened'], 'file': mine}, w1)
        step('the first instance is stopped as a second writer',
             r.get('conflict') and 'saved revision 2' in r.get('message', ''), r.get('message'))
        step('and is answered with both versions',
             r.get('yours') == mine and base64.b64decode(r.get('theirs', '')) == saved)
        step('and nothing is written over the file',
             open(os.path.join(scratch, 'copy', a['path']), 'rb').read() == saved)

        again = ask(p1, f'/v1/node/{NODE}/file')
        r = ask(p1, f'/v1/node/{NODE}/save', {'opened': again['opened'], 'file': edited(again['file'], 'kept')}, w1)
        step('opened again, the first instance saves as revision 3', r.get('ok') and r.get('revision') == 3, r)
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
