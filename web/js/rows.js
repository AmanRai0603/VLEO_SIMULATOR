/*
  Every row of the design, by its group, to open its page: the way into the
  node view now the layer figure is gone (docs/PLAN_F.md, F2). The node view
  itself moves to the application's Node workspace in F3; the design's runs,
  inputs, results and health are the application's Explore.

  Nothing here is counted or judged: it lists what the index holds, in the
  order the index holds it, and a filter narrows the list by id or label.
*/
'use strict';

import { $, esc, plural, answerFirst } from './dom.js';
import { S } from './state.js';

const PAGE = { filter: '' };

/** Draw the list into `host`. */
export function renderRows(host) {
  if (!host) return;
  host.innerHTML = answerFirst('Open any row of the design: its page holds its question, its interface, ' +
    'its method, its evidence and a run.',
    ['Type part of a row\'s id or label to narrow the list. ' + plural(S.rows.length, 'row') + ', by group.',
     'The design\'s runs, inputs, results and health map are in the application\'s <a href="/app">Explore</a>.'],
    'how-to') +
    '<p><label>find a row <input class="rows-find ctl" size="40" placeholder="sw_ap_design" value="' +
    esc(PAGE.filter) + '"></label> <span class="hint-inline rows-count"></span></p><div class="rows-list"></div>';
  const input = $('.rows-find', host);
  input.oninput = () => { PAGE.filter = input.value; list(host); };
  list(host);
}

function list(host) {
  const f = PAGE.filter.trim().toLowerCase();
  const hit = r => !f || r.id.toLowerCase().includes(f) || (r.label || '').toLowerCase().includes(f);
  const by = new Map();
  for (const r of S.rows) {
    if (!hit(r)) continue;
    if (!by.has(r.parent)) by.set(r.parent, []);
    by.get(r.parent).push(r);
  }
  let n = 0;
  const html = [...by].map(([gid, rows]) => {
    n += rows.length;
    const g = S.G.get(gid);
    return '<section class="rows-grp" data-group-id="' + esc(gid) + '"><h3>' + esc(g ? g.label : gid) +
      ' <span class="hint-inline">' + esc(gid) + '</span></h3><ul>' + rows.map(r =>
        '<li><a href="#" class="xref" data-goto="' + esc(r.id) + '"><code>' + esc(r.id) + '</code></a> ' +
        esc(r.label || '') + ' <span class="hint-inline">' + esc(r.kind) + ' · ' + esc(r.state) + '</span></li>')
        .join('') + '</ul></section>';
  }).join('');
  $('.rows-list', host).innerHTML = html || '<p class="empty">No row\'s id or label holds that.</p>';
  $('.rows-count', host).textContent = f ? plural(n, 'row') + ' match' : '';
}
