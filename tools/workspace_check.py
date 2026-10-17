#!/usr/bin/env python3
"""What `/`'s node view draws, drawn in the Node workspace, in a real browser
on the real program.

docs/briefs/F3.md, F3-h2. The Node workspace in `/app` draws a node's page by
the same `drawNode` as `/`'s node view, and now carries what that view had
around it. With no key and no copy, as anyone reading the design would:

- a node's page says which inputs its numbers are for: the case, every input
  at its default;
- *Learn · Read · Expert* is in the header: Read is the default, Learn asks
  for a prediction, Expert drops the plain words;
- a what-if set on a declared row's own control raises the what-if bar,
  which names the row, writes nothing to the case, and puts everything back;
- a group named on a node's page opens that group's rows on today's tool
  page, in a tab of its own so the node's page keeps what is typed in it,
  and the run panel's *the inputs…* opens Explore at its inputs, where the
  depth switch is not shown;
- today's tool page holds no node view: a row's link there, and
  `#row/<id>`, open the row in the Node workspace (F3-h3);
- no page error.

    python3 tools/workspace_check.py

Exit 0 when every step held; 1 with the steps that did not.
"""
import os
import shutil
import subprocess
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from put_back_check import start  # noqa: E402
from node_check import chromium_path, workspace  # noqa: E402

RECORD = 'sw_central_expectation'
DECLARED = 'sw_storm_design_level'


def check():
    from playwright.sync_api import sync_playwright
    fails = []

    def step(what, ok, detail=''):
        print(f"  {'ok  ' if ok else 'FAIL'} {what}{' — ' + str(detail)[:300] if detail else ''}")
        if not ok:
            fails.append(what)

    scratch = tempfile.mkdtemp(prefix='vleo-workspace-')
    programs = []
    try:
        keys = os.path.join(scratch, 'keys')
        os.makedirs(keys)
        one, port, word = start(scratch, keys, os.path.join(scratch, 'copy'), 7875)
        programs.append(one)
        case = os.path.join(keys, 'case.csv')
        errors = []
        with sync_playwright() as pw:
            b = pw.chromium.launch(executable_path=chromium_path())
            p = b.new_page(viewport={'width': 1600, 'height': 1000})
            p.on('pageerror', lambda e: errors.append(str(e)))
            p.goto(f'http://127.0.0.1:{port}/app#session={word}')
            p.wait_for_selector('#sh-nav .tab', timeout=60000)
            p.goto(f'http://127.0.0.1:{port}/app#node/{RECORD}')
            workspace(p)
            p.wait_for_selector('.nw-page .sheet-read', timeout=60000)

            # Which inputs the numbers are for.
            said = p.inner_text('.nw-case')
            step("the node's page says which inputs its numbers are for",
                 'every input at its default' in said, said)

            # How deep the page goes.
            simply = p.locator('.nw-page .simply').first
            predict = p.locator('.nw-page .predict').first
            step('Read is the default depth', simply.is_visible() and not predict.is_visible())
            p.locator('#depth .dp', has_text='Learn').click()
            step('Learn asks for a prediction', predict.is_visible())
            p.locator('#depth .dp', has_text='Expert').click()
            step('Expert drops the plain words', not simply.is_visible())
            p.locator('#depth .dp', has_text='Read').click()
            step('and Read brings them back', simply.is_visible())

            # A what-if, on a declared row's own control.
            p.evaluate('id => window.openNode(id)', DECLARED)
            box = p.locator(f'.nw-page [data-ovr="{DECLARED}"] .ovr-v').first
            box.wait_for(timeout=60000)
            step('the what-if bar is hidden while nothing is overridden', p.locator('#ovrbar').is_hidden())
            box.fill('2')
            box.press('Tab')
            p.locator('.nw-page button', has_text='update — run everything').first.click()
            p.wait_for_selector('#ovrbar:not([hidden])', timeout=60000)
            bar = p.inner_text('#ovrbar')
            step('a what-if raises the bar, which says it is one', '1 input overridden' in bar, bar[:200])
            step('and nothing is written to the case', not os.path.exists(case))
            p.locator('#ovrbar button', has_text='put all back').click()
            p.wait_for_timeout(300)
            step('put all back lowers the bar', p.locator('#ovrbar').is_hidden())

            # A group, and the inputs, named on a node's page.
            p.evaluate('id => window.openNode(id)', RECORD)
            p.wait_for_selector('.nw-page .xref[data-group]', state='attached', timeout=60000)
            # The page's last group link, which names a group that holds rows,
            # so the rows list has a section for it. It opens in a tab of its
            # own: this one keeps what is typed in it.
            near = p.locator('.nw-page .xref[data-group]').last
            group = near.get_attribute('data-group')
            with p.context.expect_page() as opened:
                near.click()
            rows = opened.value
            rows.wait_for_selector('#rowsview:not([hidden]) .rows-grp', timeout=60000)
            rows.wait_for_timeout(300)
            # Opened at that group: its section is on screen, as the rows list
            # scrolls to it; the list itself holds every group either way.
            shown = rows.evaluate("""g => { const s = document.querySelector(`#rows-body .rows-grp[data-group-id="${g}"]`);
              if (!s) return null; const r = s.getBoundingClientRect(); return [r.top, innerHeight]; }""", group)
            step("a group named on the page opens its rows on today's tool page, at that group, in a tab of its own",
                 rows.url.endswith('/#group/' + group) and shown is not None and 0 <= shown[0] < shown[1]
                 and '/app' in p.url,
                 f'{rows.url} {shown} {p.url}')
            rows.close()
            p.wait_for_selector('.nw-run .xref[data-inputs]', timeout=60000)
            p.locator('.nw-run .xref[data-inputs]').first.click()
            p.wait_for_selector('#sh-explore .ex-ws.sel[data-view="inputs"]', timeout=60000)
            step("the run panel's inputs open Explore at its inputs",
                 p.locator('#sh-nav .tab.sel', has_text='Explore').count() == 1, p.inner_text('#sh-nav'))
            step('the depth switch is said only where a node is drawn', p.locator('#depth').is_hidden())
            # And today's tool page holds no node view (F3-h3): a row's link
            # there, and `#row/<id>`, open the row in the Node workspace.
            p.goto(f'http://127.0.0.1:{port}/')
            p.wait_for_selector('#rows-body .rows-find', timeout=60000)
            step("today's tool page holds no node view",
                 p.evaluate("() => !document.getElementById('nodeview') && typeof window.openNode === 'undefined'"))
            p.fill('#rows-body .rows-find', RECORD)
            p.click(f'#rows-body .xref[data-goto="{RECORD}"]')
            workspace(p)
            p.wait_for_selector('.nw-page .sheet-read', timeout=60000)
            step("a row's link there opens it in the Node workspace",
                 '/app' in p.url and RECORD in p.inner_text('.answer-first'), p.url)
            p.goto(f'http://127.0.0.1:{port}/#row/{DECLARED}')
            p.wait_for_url('**/app', timeout=60000)
            workspace(p)
            step('and so does #row/<id>', DECLARED in p.inner_text('.answer-first'), p.url)
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
