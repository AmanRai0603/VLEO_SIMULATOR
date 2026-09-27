/*
  Saved results: what runs returned, kept, and shown again without running.

  A run is a question somebody asked, and the answer is worth keeping — to look
  at again, to send, to compare with the next one. A result is saved from the
  run panel (or `vleo run --save`) as one CSV: every input it ran on, every
  value it returned, every row it could not run, and the run's identity. The
  daemon keeps it outside the repository, beside the saved case.

  NOTHING ON THIS PAGE RUNS THE ENGINE. A saved result is a record of what the
  engine said then; running it again today is a different result, and showing
  today's number under the old one's name is the substitution the fifth rule
  forbids. So a result is shown exactly as it was saved — and a result from
  somebody else is uploaded and shown the same way.

  Two results side by side is how a change is read: the same question on two
  sets of inputs, or before and after a release. The comparison is arithmetic
  on the two records, nothing more.
*/
'use strict';

import { $, $$, esc, fmt, plural } from './dom.js';
import { caseChanged } from './state.js';

const POST = { method: 'POST', headers: { 'content-type': 'application/x-www-form-urlencoded' } };
const PAGE = { open: null, compare: '', filter: '' };

async function get(url) {
  try { return await (await fetch(url)).json(); } catch (e) { return { ok: false, message: String(e) }; }
}
async function post(url, params) {
  try {
    return await (await fetch(url, { ...POST, body: new URLSearchParams(params).toString() })).json();
  } catch (e) {
    return { ok: false, message: 'the engine did not answer: ' + e };
  }
}

const unit = u => (!u || u === '-') ? '' : ' ' + u;
const answerText = r => r.answer ? r.answer.value + unit(r.answer.unit) : 'not computed';

/** Open one result on the page, by its file name. */
export function openResult(file) { PAGE.open = file; PAGE.compare = ''; }

export async function renderResults(host) {
  if (!host) return;
  const list = await get('/v1/results');
  if (!list.ok) {
    host.innerHTML = '<div class="blocked"><b>no answer</b><div>' + esc(list.message || '') + '</div></div>';
    return;
  }
  let h = '<div class="node-head"><h2>Results — what runs returned</h2>' +
    '<p class="ident">A saved result is one run, kept: every input it ran on, every value it returned, ' +
    'and exactly which engine and tree produced it. Showing one runs nothing — it is shown as it was.</p></div>';
  h += '<div class="runbar res-actions"><label class="ctl res-up-l">upload a result…' +
    '<input type="file" class="res-up" accept=".csv,.html,text/csv,text/html" hidden></label>' +
    '<span class="muted">a result CSV, or the report page it rides in. Kept at <code>' + esc(list.path) +
    '</code> — outside the repository.</span><span class="why res-said"></span></div>';
  if (!list.results.length) {
    h += '<p class="empty">No result is saved yet. Run a row — on its page, or on <b>4 The run</b> — and ' +
      'press <b>save this result</b>.</p>';
  } else {
    h += '<div class="ri-wrap"><table class="fx res-list"><thead><tr><th>saved</th><th>row</th><th>answer</th>' +
      '<th>inputs</th><th>ran</th><th></th></tr></thead><tbody>' + list.results.map(r =>
        '<tr class="res-row' + (r.file === PAGE.open ? ' sel' : '') + '" data-file="' + esc(r.file) + '">' +
        '<td>' + esc(r.saved) + (r.name ? '<div class="muted">' + esc(r.name) + '</div>' : '') + '</td>' +
        '<td><code>' + esc(r.target) + '</code></td><td><b>' + esc(answerText(r)) + '</b></td>' +
        '<td>' + r.changed + ' changed</td><td>' + r.ran + ' · ' + r.blocked + ' blocked</td>' +
        '<td><button class="ctl res-open" data-file="' + esc(r.file) + '">open</button></td></tr>').join('') +
      '</tbody></table></div>';
  }
  if (list.unreadable.length) {
    h += '<div class="blocked"><b>' + plural(list.unreadable.length, 'file') + ' in the results folder ' +
      (list.unreadable.length === 1 ? 'does' : 'do') + ' not read as a result</b>' + list.unreadable.map(u =>
        '<div><code>' + esc(u.file) + '</code> — ' + esc(u.why) + '</div>').join('') + '</div>';
  }
  h += '<div class="res-view"></div>';
  host.innerHTML = h;

  $$('.res-open', host).forEach(b => b.onclick = () => { openResult(b.dataset.file); renderResults(host); });
  $('.res-up', host).onchange = async e => {
    const f = e.target.files && e.target.files[0];
    e.target.value = '';
    if (!f) return;
    const said = $('.res-said', host);
    said.textContent = 'reading ' + f.name + '…';
    const r = await post('/v1/results/upload', { csv: await f.text() });
    if (!r.ok) { said.textContent = 'not kept: ' + (r.message || 'refused'); return; }
    openResult(r.file);
    renderResults(host);
  };
  if (PAGE.open && list.results.some(r => r.file === PAGE.open)) {
    await view($('.res-view', host), host, list.results);
  }
}

async function view(el, host, all) {
  const r = await get('/v1/result?name=' + encodeURIComponent(PAGE.open));
  if (!r.ok) { el.innerHTML = '<div class="blocked">' + esc(r.message || '') + '</div>'; return; }
  const q = encodeURIComponent(PAGE.open);
  let h = '<section class="res-one"><h3><code>' + esc(r.target) + '</code>' +
    (r.name ? ' — ' + esc(r.name) : '') + '</h3>' +
    '<div class="answer">' + esc(answerText(r)) + '</div>' +
    '<p class="run-for">saved ' + esc(r.saved) + ' · ' + r.ran + ' ran, ' + r.blocked + ' blocked · mode ' +
      esc(r.mode) + ' · <b>' + plural(r.changed, 'input') + ' changed</b> from the defaults</p>' +
    '<p class="chainline muted">chain <b>' + esc(r.chain) + '</b> · kernel ' + esc(r.kernel) + ' · graph ' +
      esc(r.graph) + (r.data ? ' · data ' + esc(r.data) : '') + '</p>' +
    (r.template_current ? '' : '<p class="run-stale">Saved against another set of inputs than this tool ' +
      'has now. It is shown as it was; loading its inputs as the case carries them over.</p>') +
    '<div class="runbar res-do">' +
      '<a class="ctl" href="/v1/result.csv?name=' + q + '" download="' + esc(PAGE.open) + '">download CSV</a>' +
      '<a class="ctl" href="/v1/result.html?name=' + q + '" download="' + esc(PAGE.open.replace(/\.csv$/, '.html')) +
        '">download report</a>' +
      '<button class="ctl res-case" title="make the inputs this result ran on the saved case">use its inputs as the case</button>' +
      '<button class="ctl res-del">delete</button><span class="why res-do-said"></span></div>';
  const others = all.filter(x => x.file !== PAGE.open);
  if (others.length) {
    h += '<div class="sweepctl">compare with <select class="ctl res-cmp"><option value="">nothing</option>' +
      others.map(x => '<option value="' + esc(x.file) + '"' + (x.file === PAGE.compare ? ' selected' : '') + '>' +
        esc(x.saved + ' · ' + x.target + (x.name ? ' · ' + x.name : '')) + '</option>').join('') + '</select></div>';
  }
  h += '<div class="res-cmp-out"></div>';
  const changed = r.inputs.filter(i => i.note === 'changed');
  h += '<h4>Inputs changed from their defaults</h4>' + (changed.length
    ? '<div class="ri-wrap"><table class="fx"><tbody>' + changed.map(i => '<tr><td><code>' + esc(i.id) +
      '</code></td><td>' + esc(i.name) + '</td><td><b>' + esc(i.value) + esc(unit(i.unit)) + '</b></td></tr>').join('') +
      '</tbody></table></div>'
    : '<p class="muted">none — every input at its default</p>');
  h += '<h4>Every value it returned</h4><input class="ctl res-filter" type="search" placeholder="filter by row" value="' +
    esc(PAGE.filter) + '"><div class="ri-wrap"><table class="fx res-out"><thead><tr><th>row</th><th>name</th>' +
    '<th>value</th><th>credibility</th><th>held down by</th></tr></thead><tbody></tbody></table></div>';
  if (r.blocked_rows.length) {
    h += '<details class="run-blocked"><summary>' + plural(r.blocked_rows.length, 'row') + ' could not run — why</summary>' +
      '<div class="blocked">' + r.blocked_rows.map(b => '<div>' + esc(b.id) + ' — ' + esc(b.note) + '</div>').join('') +
      '</div></details>';
  }
  el.innerHTML = h + '</section>';

  const paintOut = () => {
    const f = PAGE.filter.trim().toLowerCase();
    $('.res-out tbody', el).innerHTML = r.outputs
      .filter(o => !f || o.id.toLowerCase().includes(f) || o.name.toLowerCase().includes(f))
      .map(o => '<tr' + (o.id === r.target ? ' class="res-target"' : '') + '><td><code>' + esc(o.id) +
        '</code></td><td>' + esc(o.name) + '</td><td>' + esc(o.value) + esc(unit(o.unit)) + '</td><td>' +
        esc(o.credibility) + ' of 4</td><td>' + esc(o.governing) + '</td></tr>').join('');
  };
  paintOut();
  $('.res-filter', el).oninput = e => { PAGE.filter = e.target.value; paintOut(); };
  $('.res-case', el).onclick = async () => {
    const said = $('.res-do-said', el);
    const res = await post('/v1/results/as-case', { name: PAGE.open });
    if (!res.ok) { said.textContent = 'the case was not changed: ' + (res.message || 'refused'); return; }
    await caseChanged();
    said.textContent = 'the case is now the inputs this result ran on — ' + plural(res.changed, 'input') + ' changed';
  };
  $('.res-del', el).onclick = async () => {
    if (!confirm('Delete this saved result? Download it first to keep a copy.')) return;
    const res = await post('/v1/results/delete', { name: PAGE.open });
    if (res.ok) { PAGE.open = null; renderResults(host); }
  };
  const cmp = $('.res-cmp', el);
  if (cmp) cmp.onchange = () => { PAGE.compare = cmp.value; compare($('.res-cmp-out', el), r); };
  if (PAGE.compare) compare($('.res-cmp-out', el), r);
}

/** Two records, side by side: every value either returned, and how far it moved. */
async function compare(el, a) {
  if (!PAGE.compare) { el.innerHTML = ''; return; }
  const b = await get('/v1/result?name=' + encodeURIComponent(PAGE.compare));
  if (!b.ok) { el.innerHTML = '<div class="blocked">' + esc(b.message || '') + '</div>'; return; }
  const byId = new Map(b.outputs.map(o => [o.id, o]));
  const ids = [...new Set(a.outputs.map(o => o.id).concat(b.outputs.map(o => o.id)))];
  const rows = ids.map(id => {
    const x = a.outputs.find(o => o.id === id), y = byId.get(id);
    const d = x && y && x.si !== null && y.si !== null && x.si !== 0 ? (y.si - x.si) / Math.abs(x.si) : null;
    return { id, x, y, d };
  });
  const moved = rows.filter(r => r.d === null ? !(r.x && r.y) : r.d !== 0);
  const inA = new Map(a.inputs.map(i => [i.id, i])), inB = new Map(b.inputs.map(i => [i.id, i]));
  const inputsMoved = [...inB.keys()].filter(k => inA.has(k) && inA.get(k).si !== inB.get(k).si);
  const biggest = Math.max(1e-12, ...moved.filter(r => r.d !== null).map(r => Math.abs(r.d)));
  el.innerHTML = '<div class="res-cmp-box"><p><b>This result</b> against <b>' + esc(b.saved + ' · ' + b.target) +
    '</b>: ' + plural(inputsMoved.length, 'input') + (inputsMoved.length === 1 ? ' differs, ' : ' differ, ') +
    plural(moved.length, 'value') + ' moved' +
    (rows.length - moved.length ? ', ' + (rows.length - moved.length) + ' did not' : '') + '.' +
    (a.chain === b.chain ? ' The chains are the same: these are the same run.' : '') + '</p>' +
    (inputsMoved.length ? '<p class="muted">inputs that differ: ' + inputsMoved.map(k => '<code>' + esc(k) + '</code> ' +
      esc(inA.get(k).value) + ' → ' + esc(inB.get(k).value) + esc(unit(inB.get(k).unit))).join(', ') + '</p>' : '') +
    (moved.length ? '<div class="ri-wrap"><table class="fx"><thead><tr><th>row</th><th>this</th><th>that</th>' +
      '<th>moved</th><th></th></tr></thead><tbody>' + moved.map(r =>
        '<tr><td><code>' + esc(r.id) + '</code></td><td>' + (r.x ? esc(r.x.value) + esc(unit(r.x.unit)) : '—') +
        '</td><td>' + (r.y ? esc(r.y.value) + esc(unit(r.y.unit)) : '—') + '</td><td>' +
        (r.d === null ? (r.x ? 'only here' : 'only there') : (r.d > 0 ? '+' : '') + fmt(r.d * 100) + '%') +
        '</td><td class="res-bar-c">' + (r.d === null ? '' : '<i class="res-bar ' + (r.d > 0 ? 'up' : 'down') +
        '" style="width:' + Math.max(2, Math.round(Math.abs(r.d) / biggest * 100)) + '%"></i>') + '</td></tr>').join('') +
      '</tbody></table></div>' : '') + '</div>';
}
