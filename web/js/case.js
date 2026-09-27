/*
  The case: every input of the one design, and the values it runs at.

  There is one case — the multipayload design — and what changes from one
  customer, or one condition, to the next is its inputs, never the tree. Adding
  a customer used to mean adding a folder under cases/, a commit and a review;
  a tree that grew a case per customer would grow without bound. So the values
  live in one saved case that the daemon keeps outside the repository and lays
  under every run it answers, and this page is where a person sets them.

  Two ways in, one store. A value typed here, or a CSV uploaded here, both go
  through the same check on the daemon — the one in vleo-modules — and are
  saved only when every row of them can be applied. A file with one bad row is
  refused whole, the bad rows named, because keeping the good half would be a
  run on values nobody asked for.

  CUSTOMER AND CONDITION ARE HALVES OF ONE LIST, not two cases. The condition
  half is the orbit, the environment and the solar weather the design flies in
  (`cases/multipayload.toml` names them); everything else is what the customer
  chooses. The split is for reading; the run takes both.

  Nothing on this page writes a file git can see. The saved case sits at the
  path the daemon names, and a person keeps copies by downloading them.
*/
'use strict';

import { $, $$, esc, fmt, plural, answerFirst } from './dom.js';
import { S, loadSaved, caseChanged } from './state.js';
import { clearOverride, isInput } from './inputs.js';

const POST = { method: 'POST', headers: { 'content-type': 'application/x-www-form-urlencoded' } };

async function post(path, params) {
  try {
    return await (await fetch(path, { ...POST, body: params.toString() })).json();
  } catch (e) {
    return { ok: false, message: 'the engine did not answer: ' + e };
  }
}

const unitText = u => (!u || u === '-') ? '' : u;
const shown = (i, si) => si / (i.factor || 1);

/** What the saved case sets, as id → SI. */
export function savedValues() {
  return new Map(((S.saved && S.saved.set) || []).map(s => [s.id, s.value]));
}

/** One input of the case by id, as `/v1/inputs` describes it. */
export const caseInput = id => ((S.saved && S.saved.inputs) || []).find(i => i.id === id) || null;

/**
 * Save exactly these values as the case, replacing what was saved.
 *
 * Every value is re-checked by the daemon, and nothing is written unless all
 * of them pass. Saving nothing is a reset: an empty case and no case run the
 * same, and only one of them leaves a file behind.
 */
export async function saveCase(values, then) {
  const want = [...values].filter(([id, si]) => {
    const i = caseInput(id);
    return i && si !== i.default;
  });
  const res = want.length
    ? await post('/v1/inputs', new URLSearchParams(want.map(([id, si]) => ['set', id + ':' + si])))
    : await post('/v1/inputs/reset', new URLSearchParams());
  if (res.ok) {
    if (then) then();
    await caseChanged();
  }
  return res;
}

/**
 * The reader's what-if edits, made permanent: laid over the saved case, saved,
 * and then cleared — they are now the case, and a what-if that equals the case
 * is the absence of one.
 */
export async function saveOverridesToCase() {
  const values = savedValues();
  const moved = [];
  for (const [id, si] of S.overrides) {
    if (caseInput(id)) { values.set(id, si); moved.push(id); }
  }
  if (!moved.length) return { ok: false, message: 'none of the edits is an input of the case' };
  // Cleared before the redraw the save sets off, so nothing is drawn once
  // with the edits both saved and still laid over the case.
  return saveCase(values, () => { for (const id of moved) clearOverride(id); });
}

/** How many what-if edits could be saved into the case. */
export const savableOverrides = () => [...S.overrides.keys()].filter(id => caseInput(id)).length;

// ---------------------------------------------------------------------------
// the page

const PAGE = { group: 'customer', filter: '', draft: new Map(), preview: null };

export async function renderCase(host) {
  if (!host) return;
  if (!S.saved) await loadSaved();
  const d = S.saved;
  if (!d) {
    host.innerHTML = '<div class="blocked"><b>no answer</b><div>The engine did not describe the ' +
      'case. Nothing here can be shown or saved until it does — reload once it is running.</div></div>';
    return;
  }
  const all = d.inputs;
  const count = g => all.filter(i => i.group === g).length;
  const changedIn = g => all.filter(i => i.group === g && i.value !== null && i.value !== i.default).length;
  let h = '<div class="node-head"><h2>' + esc(d.label) + ' — the inputs</h2></div>' +
    answerFirst('Your case: every number the design runs on. Change any of them here, or download the ' +
      'CSV, edit it and upload it back — it is kept on this machine and never changes the design.',
      [esc(d.note), 'Customer inputs are what the customer chooses; condition inputs are the world it flies in.',
       'A run always uses the saved case unless you set a value just for that run.'], 'how-to');

  h += '<div class="case-state">' + stateLine(d) + '</div>';
  h += '<div class="runbar case-actions">' +
    '<a class="ctl case-dl" href="/v1/inputs.csv" download="vleo-case.csv">download the case as CSV</a>' +
    '<a class="ctl case-tpl" href="/v1/inputs.csv?inputs=defaults" download="vleo-case-template.csv">' +
      'download a blank template</a>' +
    '<label class="ctl case-up-l">upload a CSV…<input type="file" class="case-up" accept=".csv,text/csv" hidden></label>' +
    '<button class="ctl case-reset"' + (d.stored ? '' : ' disabled') + ' title="' +
      (d.stored ? 'every input back to its default, and the saved file removed' : 'nothing is saved') +
      '">reset every input to its default</button>' +
    '</div>';
  h += '<div class="case-preview"></div>';

  h += '<div class="tabrow case-tabs">' + ['customer', 'condition'].map(g =>
    '<button class="tab case-tab' + (PAGE.group === g ? ' sel' : '') + '" data-group="' + g + '">' +
      (g === 'customer' ? 'Customer' : 'Condition') + ' · ' + count(g) +
      (changedIn(g) ? ' <span class="case-n">' + changedIn(g) + ' changed</span>' : '') +
    '</button>').join('') +
    '<input class="ctl case-filter" type="search" placeholder="filter by name or id" value="' +
      esc(PAGE.filter) + '" aria-label="filter the inputs">' +
    '</div>';
  h += '<p class="muted case-group-note">' + (PAGE.group === 'customer'
    ? 'What the customer chooses: the payloads, the vehicle, the service and what it must deliver.'
    : 'What the design flies in: the orbit, the environment and the solar weather it is sized for.') +
    '</p>';
  h += '<div class="ri-wrap"><table class="fx case-table"><thead><tr><th>input</th><th>value</th>' +
    '<th>default</th><th>declared range</th><th></th></tr></thead><tbody></tbody></table></div>';
  h += '<div class="runbar case-save-bar"><button class="ctl case-save" disabled>save to the case</button>' +
    '<button class="ctl case-discard" disabled>discard edits</button>' +
    '<span class="why case-said"></span></div>';
  h += '<p class="muted">Every run, every sweep and every figure in this tool is for these values. ' +
    'Saving writes <code>' + esc(d.path) + '</code> — outside the repository, so git never sees a ' +
    'case — and every value is checked against its row\'s declared range before anything is ' +
    'written. A blank value in a CSV means the default. To keep a case, download it; to use one, ' +
    'upload it. The same file runs from a terminal with <code>vleo run &lt;node&gt; --inputs ' +
    '&lt;file.csv&gt;</code>.</p>';
  host.innerHTML = h;
  paintTable(host);
  wire(host);
}

function stateLine(d) {
  const bits = [];
  bits.push(plural(d.inputs.length, 'input') + ': ' +
    d.inputs.filter(i => i.group === 'customer').length + ' customer, ' +
    d.inputs.filter(i => i.group === 'condition').length + ' condition');
  bits.push(d.stored
    ? '<b>' + plural(d.changed, 'input') + ' changed from default</b> in the saved case'
    : 'nothing saved — every input is at its default');
  let h = '<p>' + bits.join(' · ') + '</p>';
  if (d.error) h += '<div class="blocked"><b>not written back</b><div>' + esc(d.error) + '</div></div>';
  // AFTER AN UPDATE, WHAT THE UPDATE DID. The daemon carried the case over to
  // the inputs the tool has now, kept the file as it was, and recorded every
  // value it could not carry. The record stays until the case is saved again,
  // so it is said here rather than once in a log nobody reads.
  if (d.upgrade) {
    h += '<div class="case-upgrade">' + upgradeHtml(d.upgrade,
      'The tool has changed since this case was saved, and the case was carried over to the ' +
      'inputs it has now.') +
      (d.upgrade.backup ? '<p>The file as it was is kept at <code>' + esc(d.upgrade.backup) + '</code>. ' +
        '<a class="ctl case-bk" href="/v1/inputs.csv?backup=1" download="vleo-case-before-update.csv">' +
        'download it</a></p>' : '') +
      '<p><button class="ctl case-keep">keep the carried-over case</button> ' +
      '<span class="muted">saves it as it stands, which clears this note</span></p></div>';
  }
  return h;
}

/**
 * What a carry-over does, in words: the inputs that are new and run at their
 * defaults, and every value that could not be carried, with the value itself
 * so it can be typed back if it still means something.
 */
function upgradeHtml(u, lead) {
  const aside = u.set_aside || [], fresh = u.new || [];
  let h = '<p><b>' + esc(lead) + '</b> ' + (fresh.length
    ? plural(fresh.length, 'input') + ' ' + (fresh.length === 1 ? 'is' : 'are') + ' new and ' +
      (fresh.length === 1 ? 'runs' : 'run') + ' at ' + (fresh.length === 1 ? 'its' : 'their') + ' default'
    : 'No input is new') + '; ' + (aside.length
    ? plural(aside.length, 'value') + ' could not be carried and ' + (aside.length === 1 ? 'is' : 'are') +
      ' set aside:'
    : 'every value was carried.') + '</p>';
  if (aside.length) {
    h += '<div class="blocked">' + aside.map(a =>
      '<div><code>' + esc(a.id) + '</code>' + (a.value ? ' = ' + esc(a.value) + ' ' + esc(unitText(a.unit)) : '') +
      ' — ' + esc(a.why) + '</div>').join('') + '</div>';
  }
  if (fresh.length) {
    const SHOW = 12;
    h += '<p class="muted">new: ' + fresh.slice(0, SHOW).map(n => '<code>' + esc(n) + '</code>').join(', ') +
      (fresh.length > SHOW ? ' and ' + (fresh.length - SHOW) + ' more' : '') + '</p>';
  }
  return h;
}

function visible() {
  const f = PAGE.filter.trim().toLowerCase();
  return S.saved.inputs.filter(i => i.group === PAGE.group &&
    (!f || i.id.toLowerCase().includes(f) || i.label.toLowerCase().includes(f)));
}

function paintTable(host) {
  const tb = $('.case-table tbody', host);
  const rows = visible();
  tb.innerHTML = rows.length ? rows.map(i => {
    const saved = i.value !== null && i.value !== i.default;
    const draft = PAGE.draft.has(i.id);
    const si = draft ? PAGE.draft.get(i.id) : i.value !== null ? i.value : i.default;
    return '<tr class="ci' + (saved ? ' saved' : '') + (draft ? ' on' : '') + '" data-id="' + esc(i.id) + '">' +
      '<td class="ri-name"><b>' + esc(i.symbol || i.id) + '</b> <span class="muted">' + esc(i.label) +
        '</span><div class="muted ri-id"><code>' + esc(i.id) + '</code></div></td>' +
      '<td class="ri-val"><input class="ovr-v ci-v" type="number" step="any" value="' +
        esc(fmt(shown(i, si))) + '" aria-label="' + esc(i.label) + '"> <span class="ovr-u">' +
        esc(unitText(i.unit)) + '</span> <span class="ri-src ' + (draft ? 'you' : saved ? 'saved' : 'design') +
        '">' + (draft ? 'edited, not saved' : saved ? 'saved' : 'default') + '</span>' +
        '<div class="ri-why"></div></td>' +
      '<td class="muted">' + esc(fmt(shown(i, i.default))) + '</td>' +
      '<td class="muted ri-range">' + esc(fmt(shown(i, i.lo))) + ' … ' + esc(fmt(shown(i, i.hi))) +
        ' ' + esc(unitText(i.unit)) + '</td>' +
      '<td><button class="ctl ci-def"' + (saved || draft ? '' : ' disabled') +
        ' title="back to the default">default</button></td></tr>';
  }).join('') : '<tr><td colspan="5" class="muted">no input matches</td></tr>';
  $$('tr.ci', tb).forEach(tr => wireRow(host, tr));
  paintSaveBar(host);
}

function wireRow(host, tr) {
  const i = caseInput(tr.dataset.id);
  const field = $('.ci-v', tr), why = $('.ri-why', tr);
  field.addEventListener('change', () => {
    const raw = field.value.trim();
    const now = i.value !== null ? i.value : i.default;
    const si = raw === '' ? i.default : Number(raw) * (i.factor || 1);
    // Refuse, never clamp: the same rule the daemon applies, said here first
    // so a person sees it on the field and not only on Save.
    const bad = !isFinite(si) ? 'not a number'
      : si < i.lo ? 'below ' + fmt(shown(i, i.lo)) + ' ' + unitText(i.unit)
      : si > i.hi ? 'above ' + fmt(shown(i, i.hi)) + ' ' + unitText(i.unit) : '';
    tr.classList.toggle('bad', !!bad);
    why.textContent = bad ? 'refused: ' + bad : '';
    if (bad) { PAGE.draft.delete(i.id); paintSaveBar(host); return; }
    if (si === now) PAGE.draft.delete(i.id); else PAGE.draft.set(i.id, si);
    paintTable(host);
  });
  $('.ci-def', tr).onclick = () => {
    const saved = i.value !== null && i.value !== i.default;
    if (saved) PAGE.draft.set(i.id, i.default); else PAGE.draft.delete(i.id);
    paintTable(host);
  };
}

function paintSaveBar(host) {
  const n = PAGE.draft.size;
  $('.case-save', host).disabled = !n || !!$('tr.ci.bad', host);
  $('.case-discard', host).disabled = !n;
  // The label stays put — the manual names it — and the count sits beside it.
  $('.case-said', host).textContent = n ? plural(n, 'edit') + ' not saved yet' : '';
}

function wire(host) {
  $$('.case-tab', host).forEach(b => b.onclick = () => { PAGE.group = b.dataset.group; renderCase(host); });
  const f = $('.case-filter', host);
  f.oninput = () => { PAGE.filter = f.value; paintTable(host); };
  $('.case-discard', host).onclick = () => { PAGE.draft.clear(); paintTable(host); };
  $('.case-save', host).onclick = async () => {
    const values = savedValues();
    for (const [id, si] of PAGE.draft) values.set(id, si);
    const said = $('.case-said', host);
    said.textContent = 'saving…';
    const res = await saveCase(values);
    if (!res.ok) { said.textContent = 'not saved: ' + refusalText(res); return; }
    PAGE.draft.clear();
    renderCase(host);
  };
  const keep = $('.case-keep', host);
  if (keep) keep.onclick = async () => {
    keep.disabled = true;
    const res = await saveCase(savedValues());
    if (res.ok) renderCase(host); else keep.disabled = false;
  };
  $('.case-reset', host).onclick = async () => {
    if (!confirm('Put every input back to its default and remove the saved case? ' +
                 'Download it first to keep a copy.')) return;
    const res = await post('/v1/inputs/reset', new URLSearchParams());
    if (res.ok) { PAGE.draft.clear(); await caseChanged(); renderCase(host); }
  };
  $('.case-up', host).onchange = async e => {
    const file = e.target.files && e.target.files[0];
    e.target.value = '';
    if (file) await preview(host, file.name, await file.text());
  };
}

function refusalText(res) {
  return (res.message || 'refused') + (res.refused && res.refused.length
    ? ' ' + res.refused.map(r => (r.line ? 'line ' + r.line + ' ' : '') + r.id + ' ' + r.why).join('; ')
    : '');
}

/**
 * An uploaded file, read by the daemon and shown before anything is saved:
 * what it sets, what it changes from the default, and every row it refuses.
 */
export async function preview(host, name, text) {
  const el = $('.case-preview', host);
  el.innerHTML = '<p class="muted">reading ' + esc(name) + '…</p>';
  const r = await post('/v1/inputs/check', new URLSearchParams({ csv: text }));
  const cur = savedValues();
  const moves = (r.set || []).map(s => {
    const i = caseInput(s.id);
    const was = cur.has(s.id) ? cur.get(s.id) : i ? i.default : null;
    return { i, id: s.id, si: s.value, was };
  }).filter(m => m.i && m.si !== m.was);
  // What the file leaves out goes back to its default: a file is the whole
  // case, not a patch over the last one, so the file alone says what runs.
  const dropped = [...cur.keys()].filter(id => !(r.set || []).some(s => s.id === id));
  let h = '<div class="case-file"><h4>' + esc(name) + '</h4>';
  if (!r.ok || (r.refused && r.refused.length)) {
    h += '<div class="blocked"><b>This file cannot be used: ' + plural((r.refused || []).length, 'row') +
      ' refused.</b> Nothing from it is applied — correct the rows and upload it again.' +
      (r.refused || []).map(x => '<div>' + (x.line ? 'line ' + x.line + ' · ' : '') +
        (x.id ? '<code>' + esc(x.id) + '</code> ' : '') + esc(x.why) + '</div>').join('') +
      '</div><button class="ctl case-pv-close">close</button></div>';
    el.innerHTML = h;
    $('.case-pv-close', el).onclick = () => { el.innerHTML = ''; };
    return;
  }
  // A file from an older version of the tool is carried over, not refused:
  // what it cannot carry is named here, before anything is saved.
  if (r.outdated && r.upgrade) {
    h += '<div class="case-upgrade">' + upgradeHtml(r.upgrade,
      'This file was written by an older version of the tool, for other inputs than it has now. ' +
      'Its values are carried over.') + '</div>';
  }
  h += '<p>It sets <b>' + plural(r.set.length, 'input') + '</b>, ' + plural(r.changed, 'of them', 'of them') +
    ' away from the default; ' + r.defaulted + ' stay at the default. Saving it replaces the case: ' +
    (moves.length || dropped.length
      ? plural(moves.length + dropped.length, 'value') + ' would change.'
      : 'nothing would change.') + '</p>';
  if (moves.length || dropped.length) {
    h += '<div class="ri-wrap"><table class="fx"><thead><tr><th>input</th><th>now</th><th>from the file</th></tr>' +
      '</thead><tbody>' + moves.map(m =>
        '<tr><td><code>' + esc(m.id) + '</code> <span class="muted">' + esc(m.i.group) + '</span></td><td>' +
        esc(fmt(shown(m.i, m.was))) + ' ' + esc(unitText(m.i.unit)) + '</td><td><b>' + esc(fmt(shown(m.i, m.si))) + '</b> ' +
        esc(unitText(m.i.unit)) + '</td></tr>').join('') +
      dropped.map(id => {
        const i = caseInput(id);
        return i ? '<tr><td><code>' + esc(id) + '</code> <span class="muted">' + esc(i.group) +
          '</span></td><td>' + esc(fmt(shown(i, cur.get(id)))) + '</td><td>default ' +
          esc(fmt(shown(i, i.default))) + ' ' + esc(unitText(i.unit)) + '</td></tr>' : '';
      }).join('') + '</tbody></table></div>';
  }
  h += '<div class="runbar"><button class="ctl case-pv-save">save as the case</button>' +
    '<button class="ctl case-pv-close">discard</button><span class="why case-pv-said"></span></div></div>';
  el.innerHTML = h;
  $('.case-pv-close', el).onclick = () => { el.innerHTML = ''; };
  $('.case-pv-save', el).onclick = async () => {
    const said = $('.case-pv-said', el);
    said.textContent = 'saving…';
    const res = await post('/v1/inputs', new URLSearchParams({ csv: text }));
    if (!res.ok) { said.textContent = 'not saved: ' + refusalText(res); return; }
    PAGE.draft.clear();
    await caseChanged();
    renderCase(host);
  };
}

/** Whether a row is an input of the case — offered here, and saved with it. */
export const inCase = r => isInput(r) && !!caseInput(r.id);
