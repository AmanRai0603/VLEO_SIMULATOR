/*
  The Explore workspace of the application (docs/briefs/F2.md): today's design
  or the design's own files, whichever is open, looked at four ways.

  * WHICH DESIGN, AND WHERE IT BREAKS. Which design is open and what built it:
    each release taken, who sealed it, how its seal checked through the chain,
    and the released design it was based on; the health map; a closure traced
    to its cause; two files compared; and an issue raised from any mark that
    needs a person (explore.js).
  * RUN. One row, alone, its branch or everything, on the case (run.js).
  * INPUTS. The case every run is on (case.js).
  * RESULTS KEPT. Every result kept, each naming the design it ran on, by its
    fingerprint, and the engine that made it (results.js).

  The four are the views today's tool page holds, mounted here as they are:
  one drawing of each, so the two pages cannot drift while both exist. They
  are loaded the first time Explore is opened, never with My work.

  An issue is signed with the open key: the shell hands this module the one
  function that carries the launch's word, and nothing else here sees it.
*/
'use strict';

import { $, $$, esc } from './dom.js';
import { S as E, load, loadSaved } from './state.js';
import { loadOverrides } from './inputs.js';
import { renderExplore } from './explore.js';
import { renderRun } from './run.js';
import { renderCase } from './case.js';
import { renderResults, openResult } from './results.js';

const VIEWS = [
  ['design', 'Which design · where it breaks', 'which design is open and what built it, the health map, a trace, a comparison'],
  ['run', 'Run', 'one row, its branch or everything, on the case'],
  ['inputs', 'Inputs', 'the case every run is on'],
  ['results', 'Results kept', 'every result kept, each naming its design and its engine'],
];

const W = { view: 'design', loaded: null };

/** The index and the case, read once, as today's tool page reads them. */
function ready() {
  if (!W.loaded) {
    W.loaded = (async () => {
      await load();
      await loadSaved();
      loadOverrides();
    })();
    // A failed read is tried again the next time, never kept as the answer.
    W.loaded.catch(() => { W.loaded = null; });
  }
  return W.loaded;
}

/** Draw the workspace into `host`. `raise(params)` writes an issue. */
export async function mountExplore(host, raise) {
  host.innerHTML = '<nav class="tabrow sub ex-ws-nav" aria-label="explore">' + VIEWS.map(([id, name, holds]) =>
    '<button class="ctl ex-ws' + (W.view === id ? ' sel' : '') + '" data-view="' + id + '" title="' + esc(holds) + '">' +
    esc(name) + '</button>').join('') + '</nav><div class="ex-ws-body"><p class="sh-loading">asking the engine…</p></div>';
  $$('.ex-ws', host).forEach(b => b.onclick = () => { W.view = b.dataset.view; mountExplore(host, raise); });
  const body = $('.ex-ws-body', host);
  // A row named anywhere opens in Run; a result kept opens in Results kept.
  if (!host.dataset.wired) {
    host.dataset.wired = '1';
    host.addEventListener('click', e => {
      const x = e.target.closest('[data-goto], [data-results]');
      if (!x || !host.contains(x)) return;
      e.preventDefault();
      if (x.dataset.results) { openResult(x.dataset.results); W.view = 'results'; }
      else if (E.byId.has(x.dataset.goto)) { E.runTarget = x.dataset.goto; W.view = 'run'; }
      else return;
      mountExplore(host, raise);
    });
  }
  try {
    await ready();
  } catch (e) {
    body.innerHTML = '<div class="blocked"><b>no answer</b><div>the engine did not answer: ' + esc(String(e)) + '</div></div>';
    return;
  }
  if (W.view === 'design') return renderExplore(body, { raise });
  if (W.view === 'inputs') return renderCase(body);
  if (W.view === 'results') return renderResults(body);
  const id = E.runTarget || (E.byId.has('prop_thrust_to_drag') ? 'prop_thrust_to_drag' : (E.rows[0] || {}).id);
  E.runTarget = id;
  renderRun(body, E.byId.get(id), true);
}
