/*
  The status line and the foot of today's tool page: what the view on screen
  is, and which engine and data answer it.

  Everything it prints is counted on this draw and never remembered: a
  number on a status line that was true the last time the page was drawn is
  worse than none, because it is read as a caption.
*/
'use strict';

import { $, esc } from './dom.js';
import { S, caseTag } from './state.js';

export function drawStatus() {
  const say = (status, note) => { $('#status').textContent = status; $('#caserow-note').textContent = note || ''; };
  if (S.view === 'node') {
    const r = S.byId.get(S.selected);
    const g = r ? S.G.get(r.parent) : null;
    return say(['Layer ' + (r ? r.layer : '—'), g ? g.label : '', caseTag(), r ? r.id : '', r ? r.state : '']
      .filter(Boolean).join(' · '), r ? 'owner ' + r.owner : '');
  }
  if (S.view === 'arch') return say('Architecture · the pattern every one of ' + S.rows.length + ' folders repeats');
  if (S.view === 'forms') return say('Forms · the inputs, results, and node forms');
  if (S.view === 'explore') return say('Explore · which design is open, where it breaks, what differs');
  if (S.view === 'manual') return say('The manual');
  if (S.view === 'rows') return say('Rows · ' + S.rows.length + ' of them, each with its page', '');
  // A view that forgot to say what it is says that, never what another view was.
  say('a view with no status line of its own');
}

export function drawFoot() {
  const v = S.version || {};
  const k = {};
  for (const r of S.rows) k[r.kind] = (k[r.kind] || 0) + 1;
  const data = v.data || [];
  $('#foot').innerHTML =
    'kernel <b>' + esc(v.kernel || '—') + '</b> · graph <b>' + esc(v.graph || '—') +
    '</b> · engine <b>' + esc(v.endpoint || 'none') + '</b> · ' +
    Object.entries(k).map(([a, b]) => b + ' ' + a).join(' · ') +
    ' · ' + (data.length
      ? 'data <b>' + esc(data.map(x => x.split('#')[0]).join(' · ')) + '</b>'
      : 'data <b>none synced</b> — every node that declares a bundle will refuse, naming what to sync');
}
