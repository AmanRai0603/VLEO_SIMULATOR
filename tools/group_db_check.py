#!/usr/bin/env python3
"""The group and node applications work on database files — in a real browser, offline.

web/group.html and web/node.html keep a group's work in SQLite database files
(groups/schema.sql): the structure the lead edits, one node file per author,
and the release assembled from them. This walks the whole loop the way the
people do, each page opened from a file with no network:

  the lead      keeps the example folder as a database; reopens it and sees
                the same group; starts a new group, adds nodes and an arrow,
                is refused a bad id, issues every node file, changes a contract
                and sees the file behind;
  an author     opens their node file in the node application, writes, pastes
                results from a spreadsheet, sees the pseudocode read, signs and
                saves;
  the lead      assembles the author's file into the release, where the
                author's signature is still current; signs the rest and seals.

Python's own sqlite3 reads every file the pages write, and the vendored engine
is the one web/vendor/sqlite/SOURCE.toml records.

    python3 tools/group_db_check.py

Exit 0 when every step held; 1 with the steps that did not.
"""
import gzip
import hashlib
import os
import sqlite3
import sys
import tempfile
import tomllib
import zipfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
GROUP = 'file://' + os.path.join(ROOT, 'web', 'group.html')
NODE = 'file://' + os.path.join(ROOT, 'web', 'node.html')
EXAMPLE = os.path.join(ROOT, 'groups', 'example')
# Headless Chromium has the save dialogs but nobody to answer them: the pages
# fall back to downloads, as they do in every browser without them.
NO_PICKERS = 'delete window.showSaveFilePicker; delete window.showOpenFilePicker; delete window.showDirectoryPicker;'
OPEN = '#gtitle:not(:text("nothing open"))'


def chromium_path():
    from panel_check import chromium_path as p
    return p()


def check():
    from playwright.sync_api import sync_playwright

    fails, errors = [], []
    tmp = tempfile.mkdtemp()

    def step(what, ok, detail=''):
        print(f"  {'ok  ' if ok else 'FAIL'} {what}{' — ' + detail if detail else ''}")
        if not ok:
            fails.append(what)

    # The engine the pages carry is the one recorded.
    src = tomllib.load(open(os.path.join(ROOT, 'web/vendor/sqlite/SOURCE.toml'), 'rb'))
    wasm = gzip.decompress(open(os.path.join(ROOT, 'web/vendor/sqlite/sqlite3.wasm.gz'), 'rb').read())
    step('the vendored SQLite wasm is the one SOURCE.toml records', hashlib.sha256(wasm).hexdigest() == src['wasm_sha256'])

    def db(path):
        return sqlite3.connect('file:' + path + '?mode=ro', uri=True)

    with sync_playwright() as pw:
        exe = chromium_path()
        b = pw.chromium.launch(**({'executable_path': exe} if exe else {}))

        def page(url, width=1280):
            p = b.new_page(viewport={'width': width, 'height': 900}, accept_downloads=True)
            p.on('pageerror', lambda e: errors.append(url.rsplit('/', 1)[-1] + ': ' + str(e)))
            p.add_init_script(NO_PICKERS)
            p.goto(url)
            return p

        def save(p, click):
            with p.expect_download() as d:
                p.click(click)
            path = os.path.join(tmp, d.value.suggested_filename)
            d.value.save_as(path)
            return path

        def go(p, h, wait=600):
            p.evaluate(f"location.hash='{h}'")
            p.wait_for_timeout(wait)

        # ── the lead keeps the example folder as a database ──
        g = page(GROUP)
        g.set_input_files('#gpickdir', EXAMPLE)
        g.wait_for_selector(OPEN, timeout=20000)
        g.wait_for_timeout(600)
        before = g.inner_text('#gmain')
        go(g, '#/files')
        rel = save(g, '#gk-go')
        g.wait_for_timeout(800)
        c = db(rel)
        step('the folder becomes a release database Python reads',
             c.execute('pragma application_id').fetchone()[0] == 1447838031
             and c.execute("select value from meta where key='file_kind'").fetchone()[0] == 'release'
             and c.execute('select count(*) from node').fetchone()[0] == 6, os.path.basename(rel))
        g.close()
        g = page(GROUP)
        g.set_input_files('#gpickdb', rel)
        g.wait_for_selector(OPEN, timeout=20000)
        g.wait_for_timeout(800)
        step('reopened, the database shows the group exactly as the folder did', g.inner_text('#gmain') == before)
        go(g, '#/checks')
        step('and it has no errors', 'Errors (0)' in g.inner_text('#gmain'))

        # ── the lead issues a node file; an author fills it ──
        go(g, '#/files')
        nf = save(g, '[data-issue="orbit_speed"]')
        c = db(nf)
        meta = dict(c.execute('select key, value from meta').fetchall())
        step('a node file carries every contract and only its own content',
             meta.get('file_kind') == 'node' and meta.get('node_uid') == 'orbit_speed'
             and c.execute('select count(*) from node').fetchone()[0] == 6
             and c.execute("select count(*) from doc where scope not in ('orbit_speed')").fetchone()[0] == 0)
        n = page(NODE)
        n.set_input_files('#npick', rel)
        n.wait_for_timeout(1500)
        step('the node application refuses a file that is not a node file', 'not a node file' in n.inner_text('#gmain'))
        n.close()
        n = page(NODE)
        n.set_input_files('#npick', nf)
        n.wait_for_selector(OPEN, timeout=20000)
        n.wait_for_timeout(800)
        step('the node application opens on the node\'s contract', 'How fast does the spacecraft move' in n.inner_text('#gmain')
             and n.locator('.gwire .gbox').count() >= 2)
        steps = n.locator('#gnav .gnv').count()
        step('a computed node is walked through every step', steps >= 11, f'{steps} steps')
        go(n, '#/step/explain', 900)
        n.locator('textarea[data-h="Common misreading"]').fill('A *faster* orbit is a lower one. {{guess Is a higher orbit faster? || No: slower.}}')
        n.wait_for_timeout(900)
        pv = n.inner_text('[data-pv="Common misreading"]')
        step('what is typed is shown as it will read', 'faster orbit is a lower one' in pv and 'GUESS FIRST' in pv.upper())
        go(n, '#/step/results', 900)
        rows_before = n.locator('.te tbody tr').count()
        n.click('[data-te=paste]')
        n.fill('.te-in', 'r [m]\tanswer [m/s]\ttolerance\trefuses\torigin\n7000000\t7546.05\t1e-6\tno\thand')
        n.click('[data-te=append]')
        n.wait_for_timeout(900)
        step('rows pasted from a spreadsheet are added', n.locator('.te tbody tr').count() == rows_before + 1)
        go(n, '#/step/pseudocode', 700)
        step('the pseudocode is typeset and read', n.locator('#npc-eq math').count() >= 1 and 'It reads' in n.inner_text('#npc-lint'))
        n.locator('#npc').fill('let v = sqrt(MU_EARTH / r\nif r <= R_EARTH')
        n.wait_for_timeout(700)
        lint = n.inner_text('#npc-lint')
        step('broken pseudocode is named, line by line', 'brackets do not pair' in lint and 'not closed' in lint and 'never returns' in lint)
        n.locator('#npc').fill('# Vallado (2013), eq. 1-18.\nif r <= R_EARTH then\n  refuse "the orbit is inside the Earth"\nend\nreturn sqrt(MU_EARTH / r)')
        n.wait_for_timeout(700)
        for w in (400, 1280):
            n.set_viewport_size({'width': w, 'height': 900})
            wide = []
            for s in ['contract', 'explain', 'theory', 'pseudocode', 'inputs', 'results', 'evidence', 'pictures', 'code', 'sign']:
                go(n, f'#/step/{s}', 500)
                if n.evaluate('document.documentElement.scrollWidth - document.documentElement.clientWidth') > 0:
                    wide.append(s)
            step(f'the node application has no sideways scroll at {w} px', not wide, ', '.join(wide))
        go(n, '#/step/sign', 1200)
        n.fill('#nsig-me', 'Cy Example')
        n.click('#nsig-go')
        n.wait_for_timeout(800)
        step('only the node\'s author may sign it', 'not this node\'s author' in n.inner_text('#gmain'))
        n.fill('#nsig-me', 'Ben Example')
        n.click('#nsig-go')
        n.wait_for_timeout(1200)
        step('its author signs it', 'signed by Ben Example' in n.inner_text('#gmain'))
        filled = save(n, '#gsave')
        c = db(filled)
        step('saved, the node file is a new revision with its sign-off',
             c.execute("select revision from node where uid='orbit_speed'").fetchone()[0] == 1
             and c.execute("select name from review where scope='orbit_speed'").fetchone()[0] == 'Ben Example'
             and 'lower one' in c.execute("select body from doc where scope='orbit_speed' and kind='explanation'").fetchone()[0])
        n.close()

        # ── the lead assembles it, signs the rest and seals ──
        go(g, '#/files')
        g.set_input_files('#gf-nodes', [filled])
        g.wait_for_timeout(2000)
        step('the lead assembles the node file into the release', 'Assembled 1 node file' in g.inner_text('#gf-notes'))
        go(g, '#/node/orbit_speed/explain', 900)
        step('the author\'s words are in the release', 'lower one' in g.inner_text('#gmain'))
        go(g, '#/sign', 1200)
        step('the author\'s signature is still current in the release', 'signed ok by Ben Example' in g.inner_text('#gmain'))
        g.select_option('#gme', 'Ada Example')
        g.wait_for_timeout(800)
        for scope in ['group', 'mission_altitude', 'period_limit', 'orbit_radius', 'orbit_period', 'period_achieved']:
            g.locator(f'.gsig[data-scope="{scope}"][data-v="ok"]').click()
            g.wait_for_timeout(700)
        sealed = save(g, '#gseal')
        c = db(sealed)
        meta = dict(c.execute('select key, value from meta').fetchall())
        step('the sealed release says who sealed it and its fingerprint',
             meta.get('sealed') and meta.get('sealed_by') == 'Ada Example' and len(meta.get('fingerprint', '')) == 64,
             os.path.basename(sealed))
        step('every change is recorded', c.execute("select count(*) from change where what like 'sealed version%'").fetchone()[0] == 1)
        go(g, '#/structure', 700)
        step('a sealed release cannot be edited', 'sealed release' in g.inner_text('#gmain') and g.locator('[data-op="node"]').count() == 0)
        g.close()

        # ── a new group, from nothing ──
        g = page(GROUP)
        g.fill('#gn-id', 'demo')
        g.fill('#gn-name', 'Demo')
        g.fill('#gn-me', 'Lead Person')
        g.click('#gn-go')
        g.wait_for_selector('.gs-root', timeout=20000)

        def add(id, kind):
            g.fill('#gs-nid', id)
            g.select_option('#gs-nkind', kind)
            g.click('[data-act=add-node]')
            g.wait_for_timeout(400)

        def edit(sel, value):
            g.fill(sel, value)
            g.press(sel, 'Tab')
            g.wait_for_timeout(400)

        add('altitude', 'declared')
        add('radius', 'computed')
        edit('[data-key="unit"]', 'm')
        g.click('[data-act=add-input]')
        g.wait_for_timeout(400)
        edit('[data-key="input_1|source"]', 'altitude')
        edit('[data-key="input_1|name"]', 'h')
        step('nodes and an arrow make the map', g.locator('.gs-map .gbox').count() == 2 and g.locator('.gs-map .garrow').count() == 1)
        add('Bad Id', 'computed')
        step('a bad id is refused, by name', 'is not' in (g.inner_text('.gs-err') if g.locator('.gs-err').count() else ''))
        edit('[data-key="h|source"]', 'radius')
        step('a node cannot feed itself', 'cannot feed itself' in g.inner_text('.gs-err'))
        g.click('[data-act=flow]')
        g.wait_for_timeout(400)
        step('the flow is written from the inputs', 'radius <- altitude' in g.input_value('.gs-flow'))
        go(g, '#/files')
        z = save(g, '#gf-all')
        names = sorted(zipfile.ZipFile(z).namelist())
        step('every node file is issued', names == ['altitude.vnode', 'radius.vnode'], ', '.join(names))
        go(g, '#/structure/radius')
        edit('[data-key="unit"]', 'km')
        step('a contract changed after issue leaves its file behind', 'behind their contract: radius' in g.inner_text('.answer-first'))
        g.set_viewport_size({'width': 400, 'height': 900})
        wide = []
        for h in ['#/structure', '#/structure/radius', '#/files', '#/open']:
            go(g, h, 500)
            if g.evaluate('document.documentElement.scrollWidth - document.documentElement.clientWidth') > 0:
                wide.append(h)
        step('the lead\'s pages have no sideways scroll at 400 px', not wide, ', '.join(wide))
        structure = save(g, '#gsave')
        c = db(structure)
        step('the structure file Python reads names who changed what',
             c.execute("select count(*) from change where who='Lead Person'").fetchone()[0] >= 6
             and c.execute("select contract_version from node where uid='radius'").fetchone()[0] == 2)
        g.close()
        b.close()
    step('no page errors', not errors, '; '.join(errors[:3]))
    return fails


if __name__ == '__main__':
    failed = check()
    if failed:
        print(f'group_db_check: {len(failed)} step(s) did not hold')
        sys.exit(1)
    print('group_db_check: the group and node applications work on database files')
