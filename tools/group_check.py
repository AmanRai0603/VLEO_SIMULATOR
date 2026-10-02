#!/usr/bin/env python3
"""The group application works on a group folder — in a real browser, offline.

web/group.html is the page a group opens its folder in (groups/SPEC.toml,
`cargo run -p xtask -- group-app`). This opens it from a file, the way a
member does, chooses groups/example, and walks what a member would: the group
page, every tab of a node, a picture played, the wiring pointed at, the
checks, every scope signed and the folder sealed. Then it opens the sealed
package and checks every file against its manifest, and that a folder with
a mistake in it is refused with that mistake named.

    python3 tools/group_check.py

Exit 0 when every step held; 1 with the steps that did not.
"""
import csv
import hashlib
import io
import os
import shutil
import sys
import tempfile
import zipfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PAGE = 'file://' + os.path.join(ROOT, 'web', 'group.html')
EXAMPLE = os.path.join(ROOT, 'groups', 'example')
SCOPES = ['group', 'mission_altitude', 'period_limit', 'orbit_radius', 'orbit_speed', 'orbit_period', 'period_achieved']


def chromium_path():
    """The browser panel_check already found. One rule, one place."""
    from panel_check import chromium_path as p
    return p()


def check():
    from playwright.sync_api import sync_playwright

    fails, errors = [], []

    def step(what, ok, detail=''):
        print(f"  {'ok  ' if ok else 'FAIL'} {what}{' — ' + detail if detail else ''}")
        if not ok:
            fails.append(what)

    with sync_playwright() as pw:
        exe = chromium_path()
        b = pw.chromium.launch(**({'executable_path': exe} if exe else {}))

        def opened(folder, **kw):
            p = b.new_page(viewport={'width': 1280, 'height': 900}, accept_downloads=True, **kw)
            p.on('pageerror', lambda e: errors.append(str(e)))
            p.goto(PAGE)
            p.set_input_files('#gpickdir', folder)
            # The title changes once the folder is read; the welcome page has an answer-first too.
            p.wait_for_selector('#gtitle:not(:text("nothing open"))', timeout=20000)
            p.wait_for_selector('.answer-first', timeout=20000)
            return p

        p = opened(EXAMPLE)
        main = lambda: p.inner_text('#gmain')
        step('the group page opens with its answer first', 'how fast a spacecraft' in main())
        step('the wiring is drawn from inputs.csv', p.locator('.gwire .gbox').count() >= 7)
        p.locator('.gwire .gbox[data-id="orbit_radius"]').first.hover()
        step('pointing at a node lights what feeds it and what it feeds',
             p.locator('.gwire .gbox.up').count() >= 1 and p.locator('.gwire .gbox.down').count() >= 2)
        p.click('.gs-next')
        step('the walk-through steps', '6,778 km' in p.inner_text('.gsteps-cap'))
        step('the 3D view and the charts are drawn', p.locator('canvas.fig-canvas').count() >= 2)

        p.evaluate("location.hash='#/node/orbit_period/explain'")
        p.wait_for_timeout(600)
        p.click('.fig-play')
        p.wait_for_timeout(900)
        step('an animation plays', 'frame' in p.inner_text('.fig-t'))
        p.click('[data-depth=learn]')
        p.click('.gguess-go')
        step('a guess is answered only when asked', p.locator('.gguess-a').first.is_visible())
        for tab, want in [('theory', 'Kepler'), ('algorithm', 'Nothing here runs it'), ('io', 'orbit_radius'),
                          ('results', '6 answer(s) and 1 refusal(s)')]:
            p.evaluate(f"location.hash='#/node/orbit_period/{tab}'")
            p.wait_for_timeout(500)
            step(f'the {tab} tab shows its file', want in main())
            if tab == 'algorithm':
                step('the pseudocode is shown as equations', p.locator('.galgo-row math').count() >= 1)
        p.evaluate("location.hash='#/node/orbit_radius/code'")
        p.wait_for_timeout(400)
        p.click('details.gcode summary')
        p.wait_for_timeout(300)
        step('the author\'s code is kept and shown', 'def speed' in main())

        p.evaluate("location.hash='#/checks'")
        p.wait_for_timeout(300)
        step('the example has no errors', 'Errors (0)' in main(), main().split('\n')[3] if 'Errors (0)' not in main() else '')

        p.evaluate("location.hash='#/sign'")
        p.wait_for_timeout(600)
        p.select_option('#gme', 'Ada Example')
        p.wait_for_timeout(600)
        for scope in SCOPES:
            with p.expect_download():
                p.locator(f'.gsig[data-scope="{scope}"][data-v="ok"]').click()
            p.wait_for_timeout(500)
        step('every scope signs', main().count('signed ok') == len(SCOPES))
        with p.expect_download() as d:
            p.click('#gseal')
        pkg = d.value.path()
        z = zipfile.ZipFile(pkg)
        man = list(csv.DictReader(io.StringIO(z.read('MANIFEST.csv').decode())))
        bad = [m['path'] for m in man if hashlib.sha256(z.read(m['path'])).hexdigest() != m['sha256']]
        step('the sealed package matches its manifest', man and not bad, f'{len(man)} files' if not bad else str(bad))
        seal = list(csv.DictReader(io.StringIO(z.read('SEAL.csv').decode())))[0]
        step('the seal names the group, version and who signed', seal['group'] == 'example_orbit' and 'Ada Example:group' in seal['signed'])

        # Each page is closed when done: several drawn pages left open have
        # crashed headless Chromium's renderer.
        p.close()

        # A folder with a mistake in it: the mistake is named, and the folder cannot be sealed.
        tmp = tempfile.mkdtemp()
        broken = os.path.join(tmp, 'example')
        shutil.copytree(EXAMPLE, broken)
        os.remove(os.path.join(broken, 'nodes', 'orbit_speed', 'pseudocode.txt'))
        # A unit mistake the method checker must name, by line and by unit.
        pc = os.path.join(broken, 'nodes', 'orbit_period', 'pseudocode.txt')
        with open(pc) as f:
            body = f.read()
        with open(pc, 'w') as f:
            f.write(body.replace('return 2 * PI * sqrt(r ^ 3 / MU_EARTH)', 'return 2 * PI * sqrt(r ^ 3 / MU_EARTH) + r'))
        with open(os.path.join(broken, 'nodes', 'orbit_speed', 'inputs.csv'), 'a') as f:
            f.write('mass,nowhere,kg,,,,\n')
        q = opened(broken)
        q.evaluate("location.hash='#/checks'")
        q.wait_for_timeout(400)
        t = q.inner_text('#gmain')
        step('a missing pseudocode is an error', 'nodes/orbit_speed/pseudocode.txt is missing' in t)
        step('an input from nowhere is an error', 'comes from "nowhere"' in t)
        step('an input with no default is an error', 'mass has no default value' in t)
        step('the method checker names a unit mistake by line', 'orbit_period/pseudocode.txt:5 a sum of unlike quantities' in t)
        q.evaluate("location.hash='#/sign'")
        q.wait_for_timeout(600)
        step('a folder with errors cannot be sealed', q.locator('#gseal').count() == 0)
        q.close()
        shutil.rmtree(tmp)

        for w, scheme in [(400, 'light'), (1280, 'dark')]:
            r = opened(EXAMPLE, color_scheme=scheme)
            r.set_viewport_size({'width': w, 'height': 900})
            wide = []
            for h in ['#/', '#/node/orbit_speed/explain', '#/node/orbit_speed/algorithm', '#/node/orbit_speed/results', '#/checks', '#/sign', '#/pattern', '#/helper']:
                r.evaluate(f"location.hash='{h}'")
                r.wait_for_timeout(400)
                if r.evaluate('document.documentElement.scrollWidth - document.documentElement.clientWidth') > 0:
                    wide.append(h)
            step(f'no sideways scroll at {w} px, {scheme}', not wide, ', '.join(wide))
            r.close()
        b.close()
    step('no page errors', not errors, '; '.join(errors[:3]))
    return fails


if __name__ == '__main__':
    failed = check()
    if failed:
        print(f'group_check: {len(failed)} step(s) did not hold')
        sys.exit(1)
    print('group_check: the group application works on a group folder')
