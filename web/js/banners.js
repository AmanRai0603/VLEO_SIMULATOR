/**
 * THE BANNERS A PAGE CARRIES WHEN ITS ANSWERS ARE IN DOUBT.
 *
 * Two, each shown above every view before anything else is drawn: a page and
 * an engine from different releases, and answers from the mock engine.
 */

import { esc } from './dom.js';
import { S, CONTRACT } from './state.js';

/**
 * Say so when the engine speaks another contract than this page was built for
 * — a page from one release against an engine from another. The page carries
 * on, because most of an older contract still reads; but every answer it shows
 * is now in doubt, and a reader must know that before trusting one.
 */
export function showContract() {
  const said = S.version && S.version.contract;
  if (!S.version || said === CONTRACT || document.getElementById('contract-bar')) return;
  const bar = document.createElement('div');
  bar.id = 'contract-bar';
  bar.setAttribute('role', 'alert');
  bar.innerHTML = '<b>MISMATCH</b> — this page was built for contract ' + esc(CONTRACT) + ', and the engine ' +
    (said ? 'speaks contract ' + esc(said) : 'does not say which contract it speaks') + '. The page and the ' +
    'engine come from different releases: start the tool from one release, or reload after updating it.';
  document.body.insertBefore(bar, document.body.firstChild);
}

/**
 * Say so when the answers come from the mock engine (tools/mock_engine.py):
 * recorded examples, for working on the page without building the engine.
 * Every number on such a page is an example, and one run on another row shows
 * the recorded row's numbers — so the page must never look like the tool.
 */
export function showMock() {
  if (!S.version || S.version.endpoint !== 'mock-engine' || document.getElementById('mock-bar')) return;
  const bar = document.createElement('div');
  bar.id = 'mock-bar';
  bar.setAttribute('role', 'status');
  bar.innerHTML = '<b>MOCK ENGINE</b> — every answer on this page is a recorded example from ' +
    '<code>contract/examples</code>, not the design. For working on the page; run the tool for numbers.';
  document.body.insertBefore(bar, document.body.firstChild);
}
