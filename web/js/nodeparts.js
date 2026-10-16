/*
  A node's six parts, and running its node engineer's code again.

  docs/PLAN_F.md, stage F3: the Node workspace shows down its side the node's
  six parts — identity, ports, behaviour, evidence, explanation, history —
  each done, missing or failing, and offers "run my code again".

  NOTHING HERE JUDGES A PART. Each state is the engine's (`/v1/parts/<id>`): a
  part fails when one of the gate's own checks of it fails, in the gate's
  words, and is missing when the node holds nothing of it yet. And the code is
  run only when the button is pressed, with a POST: opening a node never runs
  anyone's code.
*/
'use strict';

import { $, esc } from './dom.js';

const MARK = { done: '✓', missing: '○', failing: '✗' };

/** Draw the parts of node `id` into `host`, and wire its button. */
export async function mountParts(host, id) {
  if (!host) return;
  let p;
  try {
    p = await (await fetch('/v1/parts/' + encodeURIComponent(id))).json();
  } catch (e) {
    p = { ok: false, message: 'the engine did not answer: ' + e };
  }
  if (!p.ok) {
    host.innerHTML = '<div class="blocked"><b>its parts</b><div>' + esc(p.message || '') + '</div></div>';
    return;
  }
  const done = p.parts.filter(x => x.state === 'done').length;
  host.innerHTML =
    '<p class="np-sum">' + done + ' of 6 parts done · it answers by <b>' + esc(p.behaviour) + '</b></p>' +
    '<ol class="np-parts">' + p.parts.map(x =>
      '<li class="np-part np-' + esc(x.state) + '"><span class="np-mark">' + (MARK[x.state] || '') +
      '</span> <b>' + esc(x.name) + '</b> <span class="np-state">' + esc(x.state) + '</span>' +
      (x.why.length ? '<ul>' + x.why.map(w => '<li>' + esc(w) + '</li>').join('') + '</ul>' : '') +
      '</li>').join('') + '</ol>' +
    '<p><button class="ctl np-rerun">run my code again</button> ' +
    '<span class="hint-inline">the node engineer’s own code, on each case recorded from it — a check only</span></p>' +
    '<div class="np-rerun-out"></div>';
  $('.np-rerun', host).onclick = async () => {
    const out = $('.np-rerun-out', host);
    out.innerHTML = '<p class="hint-inline">running…</p>';
    let r;
    try {
      r = await (await fetch('/v1/rerun/' + encodeURIComponent(id), { method: 'POST' })).json();
    } catch (e) {
      r = { ok: false, message: 'the engine did not answer: ' + e };
    }
    if (!r.ok) {
      out.innerHTML = '<div class="blocked"><b>not run</b><div>' + esc(r.message || '') + '</div></div>';
      return;
    }
    const head = { agrees: 'it still gives all ' + r.cases + ' recorded case(s)',
                   differs: 'it no longer gives every recorded case',
                   'not-run': 'not run here' }[r.verdict] || r.verdict;
    out.innerHTML = '<p class="np-rerun-' + esc(r.verdict) + '"><b>' + esc(head) + '</b>' +
      (r.language ? ' · ' + esc(r.language) : '') + '</p>' +
      (r.said.length ? '<ul>' + r.said.map(s => '<li>' + esc(s) + '</li>').join('') + '</ul>' : '');
  };
}
