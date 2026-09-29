/*
  The run, the result and the behaviour sweep.

  Everything crossing the boundary is SI. A face converts for display and never
  for transport, so nothing in this module does arithmetic on a value beyond
  dividing by the unit factor the engine sent with it.

  Every control is scoped to the host it was rendered into. The node page and
  the run page both hold a run panel, and a query that reaches the document
  root finds whichever one comes first in the markup — which is how a result
  once landed in a hidden panel and the visible one stayed blank.

  WHAT THE RUN IS ON COMES FIRST. There is one case, and a run is always on
  its saved inputs — the ones set on the Inputs page or uploaded there as a
  CSV — with whatever the reader typed over them. The panel says so, and lists
  every input the answer depends on with the value it will run at and where
  that value came from: the default, the saved case, or an edit. An edit here
  is a what-if held in this browser until it is saved to the case; neither
  ever writes a file git can see.
*/
'use strict';

import { $, $$, esc, fmt } from './dom.js';
import { S, reachFrom, isSeeded, isUndefined, isDeprecated, theCase, caseKey } from './state.js';
import { withOverrides, isInput, fromSI, toSI, unitOf, outOfRange, setOverride,
         clearOverride, onOverrideChange } from './inputs.js';
import { drawChart, attachHover, tableFor, tableTsv, viewSpec, viewIsOn,
         watchScheme, INK } from './chart.js';
import { savedValues, caseInput, saveOverridesToCase, savableOverrides } from './case.js';

const POST = { method: 'POST', headers: { 'content-type': 'application/x-www-form-urlencoded' } };

/** One run, POSTed. A refusal comes back as a value, never as an exception. */
async function runOnce(params) {
  try {
    return await (await fetch('/v1/run', { ...POST, body: params.toString() })).json();
  } catch (e) {
    return { ok: false, fault: 'no answer', message: 'the engine did not answer: ' + e };
  }
}

// ---------------------------------------------------------------------------
// what the run is on

/**
 * The case and its inputs, in a sentence: how many the saved case moves from
 * the default, and how many the reader has edited over it. A number read off
 * this panel is for exactly these.
 */
function caseNote() {
  const c = theCase();
  const d = S.saved;
  const edits = S.overrides.size;
  let h = 'Runs on <b>' + esc(c ? c.label : 'no case') + '</b>, ' + (!d ? 'whose inputs could not be read'
    : !d.stored || !d.changed ? 'every input at its declared default'
    : '<b>' + d.changed + ' input' + (d.changed === 1 ? '' : 's') + ' changed</b> from the default in ' +
      'the saved case') +
    (edits ? ', with <b>' + edits + ' what-if edit' + (edits === 1 ? '' : 's') + '</b> of yours over it' : '') +
    '. <button class="ctl xref" data-inputs>the inputs…</button>';
  return '<p>' + h + '</p>';
}

// ---------------------------------------------------------------------------
// the panel

export function renderRun(host, r, standalone) {
  if (!host || !r) return;
  const seeded = isSeeded(r);
  // Written, generated, compiled, fixtures passing — and it does not answer,
  // because nothing on its sheet says where the relation came from. The engine
  // refuses it; this stops the face offering a Run that cannot succeed.
  const inactive = isUndefined(r);

  let h = '';
  if (standalone) {
    h += '<div class="node-head"><h2>' + esc(r.label) + '</h2>' +
      '<p class="ident"><code>' + esc(r.id) + '</code> · <span class="kind">' + esc(r.kind) +
      '</span> · owner <b>' + esc(r.owner) + '</b> · layer <b>' + r.layer + '</b></p></div>' +
      '<div class="sweepctl">target <select class="run-target">' +
      S.rows.filter(x => !isSeeded(x)).map(x =>
        '<option value="' + esc(x.id) + '"' + (x.id === r.id ? ' selected' : '') + '>' +
        esc(x.id) + '</option>').join('') + '</select>' +
      '<span class="muted">only rows with something specified in them can be a target</span></div>';
  }
  h += '<div class="run-case">' + caseNote() + '</div>';

  if (seeded) {
    h += '<p class="empty">This row is seeded. The folder, the sheet and the row exist; nothing is ' +
      'specified in them yet, so the generated stub returns <code>NotRun</code> rather than a number. ' +
      'Six hundred grey rows on day one is not a failure — it is the decomposition, written down ' +
      'before anyone has been told to fill it in.</p>';
  }
  // Said here rather than only on the disabled button, because the thing a
  // reader needs is not "this is off" — it is what would turn it on, and by
  // whom. Nothing in this panel can do it: an assistant may never supply
  // mathematics, which is the reason this refusal exists.
  if (inactive) {
    h += '<p class="empty refuse"><b>INACTIVE — this row does not answer.</b> Its relation is ' +
      'stated and never derived. A function is defined by its derivation, not by its expression ' +
      'and not by its citation: the expression is one line anybody can type, and a citation says ' +
      'a paper exists rather than that this relation came out of it.<br><br>' +
      'To define it, the sheet needs a <code>[theory]</code> block — why it is this relation, what ' +
      'the answer means, and the steps it comes in. Until then the engine refuses it under its own ' +
      'name and everything downstream blocks on it, named. The inputs it reads keep their ' +
      'defaults and stay editable: an input is defined by carrying a value, a function is not.</p>';
  }
  if (!seeded) {
    h += '<details class="run-inputs" open><summary class="run-inputs-h">the inputs this answer ' +
      'depends on</summary><div class="run-inputs-body"><p class="muted">reading what the ' +
      'case runs at…</p></div></details>';
  }
  h += '<div class="runbar run-go-bar">' +
    '<button class="ctl mode' + (S.mode === 'alone' ? ' sel' : '') + '" data-mode="alone">alone</button>' +
    '<button class="ctl mode' + (S.mode === 'branch' ? ' sel' : '') + '" data-mode="branch">the branch</button>' +
    '<button class="ctl mode' + (S.mode === 'all' ? ' sel' : '') + '" data-mode="all">everything</button>' +
    '<button class="ctl run-go"' + (seeded || inactive ? ' disabled' : '') + '>run</button>' +
    '<span class="why run-why"></span></div>';
  h += '<div class="run-out"></div>';
  host.innerHTML = h;

  paintModes(host, r);
  $$('.mode', host).forEach(b => b.onclick = () => { S.mode = b.dataset.mode; paintModes(host, r); });
  $('.run-go', host).onclick = () => go(host, r);
  const t = $('.run-target', host);
  if (t) t.onchange = e => { S.runTarget = e.target.value; renderRun(host, S.byId.get(e.target.value), true); };
  renderResult(host, r);
  if (!seeded) mountInputs(host, r);
}

/** The mode buttons, and the one that is off saying why. */
function paintModes(host, r) {
  const known = new Set((S.lastRun && S.lastRun.values || []).map(v => v.id));
  const missing = r.in.map(i => S.rows[i].id).filter(x => !known.has(x));
  const inactive = isUndefined(r);
  const closure = reachFrom([r.i], S.producers).size + 1;
  // A disabled control that does not say why is a defect.
  const aloneBtn = $('.mode[data-mode="alone"]', host);
  aloneBtn.disabled = missing.length > 0 || inactive;
  aloneBtn.title = inactive
    ? 'Disabled: this row has no derivation, so it does not answer in any mode.'
    : missing.length
    ? 'Disabled: ' + missing.slice(0, 4).join(', ') +
      (missing.length > 4 ? ' and ' + (missing.length - 4) + ' more' : '') + ' ' +
      (missing.length === 1 ? 'has' : 'have') + ' never run. Run the branch instead.'
    : 'Only this node. Upstream values are whatever the store already holds.';
  if (aloneBtn.disabled && S.mode === 'alone') S.mode = 'branch';
  $$('.mode', host).forEach(b => b.classList.toggle('sel', b.dataset.mode === S.mode));
  $('.run-why', host).textContent = S.mode === 'branch' ? closure + ' nodes in the closure'
    : S.mode === 'all' ? S.rows.length + ' rows, everything buildable'
    : '1 node';
}

async function go(host, r, again) {
  const btn = $('.run-go', host);
  btn.disabled = true;
  $('.run-why', host).textContent = again ? 'running again…' : 'running…';
  // The overrides travel with every run, and the daemon lays the saved case
  // under them. A face that showed a what-if number on one panel and the
  // declared design on another would be the three-correct-numbers-at-three-
  // different-times bug this tool already has a comment about.
  //
  // A QUESTION ALREADY ANSWERED IS SHOWN, NOT ASKED AGAIN: `reuse` asks the
  // daemon for the saved result that answers exactly this — same row, mode,
  // inputs, engine and data — before running, and the answer says when it is
  // one. `again` is the reader asking for a fresh run anyway.
  const p = withOverrides(new URLSearchParams({ node: r.id, mode: S.mode, reuse: '1' }));
  if (again) p.set('again', '1');
  const res = await runOnce(p);
  if (!host.isConnected) return;
  S.lastRun = res.ok ? res : null;
  btn.disabled = false;
  paintModes(host, r);
  renderResult(host, r, res);
}

// ---------------------------------------------------------------------------
// the inputs, and where each one's value came from

/**
 * Which declared decisions actually move this answer, and by how much.
 *
 * Asked once per panel and shared by the inputs table and the sweep, because
 * the engine evaluates every decision at both ends of its range to answer it.
 * See `/v1/levers`.
 */
function levers(host, r) {
  if (!host._levers) {
    host._levers = (async () => {
      try {
        const lp = withOverrides(new URLSearchParams({ node: r.id, mode: 'branch' }));
        const lv = await (await fetch('/v1/levers?' + lp.toString())).json();
        return lv && lv.ok ? (lv.levers || []).filter(l => S.byId.has(l.id)) : [];
      } catch (e) {
        return [];
      }
    })();
  }
  return host._levers;
}

const pct = v => v >= 0.1 ? Math.round(v * 100) + '%' :
  v >= 0.001 ? (v * 100).toFixed(2) + '%' : v > 0 ? '<0.01%' : 'nothing';

/** Where this input's value on the run about to happen came from. */
function sourceOf(id) {
  if (S.overrides.has(id)) return ['you', 'your edit'];
  const i = caseInput(id);
  const saved = savedValues();
  if (i && saved.has(id) && saved.get(id) !== i.default) return ['saved', 'saved case'];
  return ['design', 'default'];
}

/** Customer or condition: which half of the case an input sits in. */
const groupOf = id => { const i = caseInput(id); return i ? i.group : ''; };

/**
 * The table of inputs.
 *
 * THE VALUE SHOWN IS THE VALUE THE ENGINE WILL USE, read from a run on the
 * saved case rather than from the sheet. The index carries every row's domain
 * but not its answer, and a saved value is not on the sheet at all — so running
 * is the only way to show the number that is about to go in.
 *
 * Most-moving first, from the levers the engine measured, with the ones that
 * move nothing saying so rather than hidden: that a decision does not reach
 * this row is a fact about the design, and often the surprising one.
 */
async function mountInputs(host, r) {
  const body = $('.run-inputs-body', host);
  if (!body) return;
  const key = caseKey();
  const ups = Array.from(reachFrom([r.i], S.producers)).map(i => S.rows[i]).filter(isInput);
  if (isInput(r) && !ups.includes(r)) ups.unshift(r);
  const summary = $('.run-inputs-h', host);
  if (!ups.length) {
    summary.textContent = 'no input a person can set reaches this answer';
    body.innerHTML = '<p class="muted">Nothing declared with a range sits upstream of this row, ' +
      'so every set of inputs runs it the same way and there is nothing here to edit.</p>';
    return;
  }

  const base = await runOnce(new URLSearchParams({ node: r.id, mode: 'branch' }));
  const lv = await levers(host, r);
  // The saved case may have changed while these were in flight; a table for
  // the old inputs drawn under the new ones would be exactly the mismatch this
  // panel exists to prevent.
  if (!host.isConnected || key !== caseKey()) return;
  const at = new Map();
  if (base.ok) for (const v of base.values || []) at.set(v.id, v.si);
  host._at = at;
  const span = new Map(lv.map(l => [l.id, l.span]));
  ups.sort((a, b) => ((span.get(b.id) ?? -1) - (span.get(a.id) ?? -1)) || a.id.localeCompare(b.id));

  const SHOW = 8;
  body.innerHTML = (base.ok ? '' :
      '<p class="blocked"><b>' + esc(base.fault || 'refused') + '</b> — ' + esc(base.message || '') +
      '. The values below could not be read for the case.</p>') +
    '<div class="ri-wrap"><table class="fx run-in-table"><thead><tr><th>input</th><th>from</th><th>runs at</th>' +
    '<th>declared range</th><th></th></tr></thead><tbody>' +
    ups.map((row, i) => inputRow(row, at.get(row.id), span.get(row.id), i >= SHOW)).join('') +
    '</tbody></table></div>' +
    (ups.length > SHOW
      ? '<button class="ctl ri-more">show all ' + ups.length + ' inputs</button>' : '') +
    '<p class="muted ri-foot">An edit here is a what-if held in this browser: it applies to every ' +
    'run and sweep until you reset it or save it to the case, and it is listed in the bar at the ' +
    'top. Neither writes a file git can see. <button class="ctl ri-go">run with these inputs</button>' +
    '<button class="ctl ri-save"' + (savableOverrides() ? '' : ' hidden') + '>save my edits to the case' +
    '</button><span class="why ri-said"></span></p>';

  const more = $('.ri-more', body);
  if (more) more.onclick = () => { $$('tr.ri.more', body).forEach(t => t.classList.remove('more')); more.remove(); };
  $('.ri-go', body).onclick = () => go(host, r);
  // Saving changes the case, and everything showing a number redraws for it —
  // this panel included. Only a refusal is left to say here.
  $('.ri-save', body).onclick = async () => {
    const res = await saveOverridesToCase();
    if (!res.ok && body.isConnected) $('.ri-said', body).textContent = 'not saved: ' + (res.message || 'refused');
  };
  $$('tr.ri', body).forEach(tr => wireInput(host, tr, at));
  paintSummary(host, ups);
  TABLES.add(host);
}

// AN INPUT CAN BE MOVED FROM ELSEWHERE — the × on a chip in the bar, "put
// everything back", the editor at the top of a declared row's own page — and a
// table still showing the old number beside a run that used the new one is the
// mismatch this panel exists to remove. Every live table is repainted from the
// store; a field the reader is typing in is left alone.
const TABLES = new Set();
onOverrideChange(() => {
  for (const host of [...TABLES]) {
    if (!host.isConnected || !host._at) { TABLES.delete(host); continue; }
    for (const tr of $$('tr.ri', host)) {
      const id = tr.dataset.ovr, row = S.byId.get(id), base = host._at.get(id);
      const field = $('.ri-v', tr);
      const has = S.overrides.has(id);
      const si = has ? S.overrides.get(id) : base;
      if (document.activeElement !== field && si != null) field.value = fmt(fromSI(row, si));
      tr.classList.toggle('on', has);
      $('.ri-reset', tr).disabled = !has;
      $('.ri-was', tr).hidden = !(has && base != null);
      const [cls, txt] = sourceOf(id);
      const src = $('.ri-src', tr);
      src.className = 'ri-src ' + cls;
      src.textContent = txt;
    }
    const save = $('.ri-save', host);
    if (save) save.hidden = !savableOverrides();
    paintSummary(host);
  }
});

function inputRow(row, baseSI, sp, more) {
  const has = S.overrides.has(row.id);
  const si = has ? S.overrides.get(row.id) : baseSI;
  const [cls, txt] = sourceOf(row.id);
  const moves = sp === 0 ? 'does not move this answer'
    : sp > 0 ? 'moves this answer by ' + pct(sp)
    : sp === null ? 'cannot be swept here' : '';
  return '<tr class="ri' + (has ? ' on' : '') + (more ? ' more' : '') + '" data-ovr="' + esc(row.id) + '">' +
    '<td class="ri-name"><b>' + esc(row.symbol || row.id) + '</b> <span class="muted">' +
      esc(row.label) + '</span><div class="muted ri-id"><code>' + esc(row.id) + '</code>' +
      (moves ? ' · ' + esc(moves) : '') + '</div></td>' +
    '<td><span class="ri-src ' + cls + '">' + esc(txt) + '</span>' +
      (groupOf(row.id) ? '<div class="muted ri-grp">' + esc(groupOf(row.id)) + '</div>' : '') + '</td>' +
    '<td class="ri-val"><input class="ovr-v ri-v" type="number" step="any" value="' +
      (si == null ? '' : esc(fmt(fromSI(row, si)))) + '" aria-label="' + esc(row.label) + '">' +
      ' <span class="ovr-u">' + esc(unitOf(row.unit)) + '</span>' +
      '<div class="muted ri-was"' + (has && baseSI != null ? '' : ' hidden') + '>' +
        (baseSI == null ? '' : 'the case: ' + esc(fmt(fromSI(row, baseSI)))) + '</div>' +
      '<div class="ri-why"></div></td>' +
    '<td class="muted ri-range">' + esc(fmt(fromSI(row, row.lo))) + ' … ' +
      esc(fmt(fromSI(row, row.hi))) + ' ' + esc(unitOf(row.unit)) + '</td>' +
    '<td><button class="ctl ri-reset"' + (has ? '' : ' disabled') +
      ' title="back to the case’s value">reset</button></td></tr>';
}

/**
 * One input row's field. Nothing here repaints the row: the field is built once
 * and only has classes and text toggled, because rebuilding the element a
 * change event is travelling through swallowed the click that caused it.
 */
function wireInput(host, tr, at) {
  const id = tr.dataset.ovr;
  const row = S.byId.get(id);
  const field = $('.ri-v', tr), why = $('.ri-why', tr), reset = $('.ri-reset', tr);
  const was = $('.ri-was', tr), src = $('.ri-src', tr);
  const paint = () => {
    const has = S.overrides.has(id);
    tr.classList.toggle('on', has);
    reset.disabled = !has;
    was.hidden = !(has && at.get(id) != null);
    const [cls, txt] = sourceOf(id);
    src.className = 'ri-src ' + cls;
    src.textContent = txt;
    staleResult(host);
    const save = $('.ri-save', host);
    if (save) save.hidden = !savableOverrides();
    paintSummary(host);
  };
  field.addEventListener('change', () => {
    const raw = field.value.trim();
    const base = at.get(id);
    if (raw === '') { clearOverride(id); if (base != null) field.value = fmt(fromSI(row, base)); why.textContent = ''; tr.classList.remove('bad'); paint(); return; }
    const si = toSI(row, Number(raw));
    // Refuse, never clamp. A value silently corrected is a design that drifted
    // without anyone deciding to.
    const bad = outOfRange(row, si);
    tr.classList.toggle('bad', !!bad);
    why.textContent = bad ? 'refused: ' + bad : '';
    if (bad) return;
    // Typing the case's own number back is the absence of an edit.
    if (base != null && si === base) clearOverride(id); else setOverride(id, si);
    paint();
  });
  reset.onclick = () => {
    clearOverride(id);
    const base = at.get(id);
    field.value = base == null ? '' : fmt(fromSI(row, base));
    why.textContent = '';
    tr.classList.remove('bad');
    paint();
  };
}

function paintSummary(host, ups) {
  const el = $('.run-inputs-h', host);
  if (!el) return;
  const rows = $$('tr.ri', host);
  const n = ups ? ups.length : rows.length;
  let you = 0, saved = 0;
  for (const tr of rows) {
    const [cls] = sourceOf(tr.dataset.ovr);
    if (cls === 'you') you++; else if (cls === 'saved') saved++;
  }
  const bits = [];
  if (saved) bits.push(saved + ' from the saved case');
  if (you) bits.push(you + ' edited by you');
  el.textContent = 'the inputs this answer depends on — ' + n +
    (bits.length ? ': ' + bits.join(', ') : ', all at their defaults');
}

/** A result on screen for inputs that have since changed says so. */
function staleResult(host) {
  const out = $('.run-out', host);
  if (!out || !$('.answer', out) || $('.run-stale', out)) return;
  out.insertAdjacentHTML('afterbegin', '<p class="run-stale">The inputs have changed since this ' +
    'run. Run again to see the answer for them.</p>');
}

// ---------------------------------------------------------------------------
// the result

export function renderResult(host, r, res) {
  const el = $('.run-out', host);
  if (!el) return;
  res = res || S.lastRun;
  if (!res) {
    el.innerHTML = '<p class="muted">Not run. The tabs are what the node <i>is</i>; ' +
      'this is what it <i>returns</i>.</p>' + sweepControls(r);
    wireSweep(host, r);
    return;
  }
  if (!res.ok) {
    el.innerHTML = '<div class="blocked"><b>' + esc(res.fault || 'refused') + '</b> · ' +
      esc(res.node || '') + '<div>' + esc(res.message || '') + '</div></div>' +
      '<p class="muted">A refusal is a value, not a magic number the caller may forget to check. ' +
      'It names the field, the bound it broke and the reason that bound exists.</p>' + sweepControls(r);
    wireSweep(host, r);
    return;
  }
  const v = res.values.find(x => x.id === r.id);
  let h = '';
  // FROM A SAVED RESULT, SAID FIRST. A number read back from a record must
  // never be taken for one run just now — the reader is told which result,
  // when it was saved, and how to run it fresh.
  const fs = res.from_saved;
  if (fs) {
    h += '<div class="run-saved"><b>Already answered</b> — shown from the saved result ' +
      (fs.name ? '<b>' + esc(fs.name) + '</b>, ' : '') + 'saved ' + esc(fs.saved) + '. The same row on the ' +
      'same inputs, engine and data gives the same answer, so nothing was run. ' +
      '<button class="ctl xref" data-results="' + esc(fs.file) + '">open it in Results</button> ' +
      '<button class="ctl run-again">run again anyway</button></div>';
  }
  if (v) {
    h += '<div class="answer">' + esc(v.symbol || r.symbol) + ' = ' + esc(v.shown) +
      ' <span class="unit">' + esc(unitOf(v.unit)) + '</span></div>';
  } else {
    h += '<div class="answer none">not computed on this run</div>';
  }
  // WHAT THIS NUMBER IS FOR, beside it — as the run itself reports it, not as
  // the page believes it asked. A number without its inputs is a number a
  // reader will quote for the wrong ones.
  const c = theCase();
  const inp = res.inputs || {};
  const edits = S.overrides.size;
  h += '<p class="run-for">on <b>' + esc(c ? c.label : 'the case') + '</b> · ' +
    (inp.defaults ? 'every input at its default'
      : inp.changed ? '<b>' + inp.changed + ' input' + (inp.changed === 1 ? '' : 's') + ' changed</b> in the saved case'
      : 'every input at its default') +
    (edits ? ', with <b>' + edits + ' input' + (edits === 1 ? '' : 's') + ' edited by you</b>' : '') +
    (inp.set_aside ? ' · <b>' + inp.set_aside + ' saved value' + (inp.set_aside === 1 ? '' : 's') +
      ' set aside</b> when the case was carried over to this version — see the Inputs page' : '') +
    '</p>';
  if (v) {
    h += credBars(v) +
      '<p class="muted">Eight factors, and the lowest governs — averaging would let a strong factor ' +
      'hide a zero. This answer is held down by <b>' + esc(v.governing) + '</b>.</p>';
  }
  h += '<div class="chainline">' + res.manifest.ran + ' ran · ' + res.manifest.blocked +
    ' blocked · ' + res.manifest.iterations + ' cycle sweep' +
    (res.manifest.iterations === 1 ? '' : 's') + '</div>';
  // Blocked rows are always named. Past a handful they fold, with the names
  // still in the summary line: forty refusals in full push the answer off the
  // screen, and hiding them entirely would be the substitution rule 5 forbids.
  if (res.blocked.length) {
    const names = res.blocked.map(b => b.id);
    const list = '<div class="blocked">' + res.blocked.map(b =>
      '<div>' + esc(b.id) + ' — ' + esc(b.message) + '</div>').join('') + '</div>';
    h += res.blocked.length <= 4 ? list
      : '<details class="run-blocked"><summary>' + res.blocked.length + ' blocked: ' +
        esc(names.slice(0, 3).join(', ')) + ' and ' + (names.length - 3) + ' more — why</summary>' +
        list + '</details>';
  }
  // THE EVIDENCE IS A VERDICT, THEN THE DETAIL. A run on a solar row executes a
  // dozen fixtures, and printing each with its arithmetic buried the one number
  // the reader ran for under a page of "got 86.8497, expected 86.8497". One
  // line says whether they held; a failure opens the list, because then the
  // detail is the news.
  if (fs) {
    h += '<p class="muted">The checks against known-good values ran when this result was made; ' +
      'they are not repeated for an answer read back. <i>Run again anyway</i> runs them.</p>';
  }
  if (res.verdicts.length) {
    const failed = res.verdicts.filter(x => !x.passed).length;
    h += '<details class="run-evidence"' + (failed ? ' open' : '') + '><summary>' +
      (failed ? '✗ ' + failed + ' of ' + res.verdicts.length + ' checks against known-good values FAILED'
              : '✓ all ' + res.verdicts.length + ' checks against known-good values held on this run') +
      '</summary><div class="verdicts">' + res.verdicts.map(x =>
        '<div class="' + (x.passed ? 'pass' : 'fail') + '">' + (x.passed ? '✓' : '✗') + ' ' +
        esc(x.node) + ' · ' + esc(x.label) + ' — got ' + fmt(x.got) + ', expected ' + fmt(x.expected) +
        ', relative error ' + x.error.toExponential(2) + ' against ' + x.tolerance.toExponential(1) +
        ' <span class="muted">' + esc(x.provenance) + ' / ' + esc(x.source) + '</span></div>').join('') +
      '</div></details>';
  }
  h += '<div class="chainline muted" title="The chain hash is the run\'s identity and its cache key. ' +
    'It covers every node the run reached and each one\'s implementation content — not the inputs ' +
    'alone, because rewriting a node\'s arithmetic without touching its interface must invalidate ' +
    'every result downstream of it.">chain <b>' + esc(res.manifest.chain) + '</b> · kernel ' +
    esc(res.manifest.kernel) + ' · graph ' + esc(res.manifest.graph) + ' · case ' +
    esc(res.manifest.case) + ' · endpoint ' + esc(res.manifest.endpoint) +
    (res.manifest.data.length ? ' · data ' + esc(res.manifest.data.join(' ')) : ' · no data bundle') +
    '</div>';
  // KEEP IT. The run as it stands — these inputs, this engine — saved as a
  // result the Results page shows again without running, and that can be sent
  // as a CSV or a report.
  h += fs ? '' : '<div class="runbar res-save-bar"><input class="ctl res-name" placeholder="a name for it (optional)" ' +
    'aria-label="a name for the result"><button class="ctl res-save">save this result</button>' +
    '<span class="why res-saved"></span></div>';
  h += sweepControls(r);
  el.innerHTML = h;
  const again = $('.run-again', el);
  if (again) again.onclick = () => go(host, r, true);
  const save = $('.res-save', el);
  if (save) save.onclick = async () => {
    const said = $('.res-saved', el);
    save.disabled = true;
    said.textContent = 'saving…';
    const p = withOverrides(new URLSearchParams({ node: r.id, mode: S.mode, label: $('.res-name', el).value }));
    let out;
    try {
      out = await (await fetch('/v1/results/save', { ...POST, body: p.toString() })).json();
    } catch (e) {
      out = { ok: false, message: 'the engine did not answer: ' + e };
    }
    save.disabled = false;
    said.innerHTML = !out.ok ? 'not saved: ' + esc(out.message || 'refused')
      : (out.already ? 'already kept — this exact question was saved ' + esc(out.saved) +
          (out.name ? ' as <b>' + esc(out.name) + '</b>' : '') + ', and is kept once. '
        : 'saved — ') +
        '<button class="ctl xref" data-results="' + esc(out.file) + '">open it in Results</button>';
  };
  wireSweep(host, r);
}

function credBars(v) {
  const names = ['maths', 'assump', 'verif', 'valid', 'pedigree', 'uncert', 'underst', 'reprod'];
  const min = Math.min.apply(null, v.vec);
  return '<div class="cred">' + v.vec.map((s, i) =>
    '<div class="f' + (s === min ? ' gov' : '') + '" title="' + names[i] + ': ' + s + ' of 4">' +
    '<i style="height:' + (s / 4 * 100) + '%"></i><span>' + names[i] + '</span></div>').join('') + '</div>';
}

// ---------------------------------------------------------------------------
// the sweep — drawn by the shared chart, from the numbers the engine returned.
// The page computes nothing but where to put a label.

/** The declared decisions upstream of this row that have a range to move in. */
function sweepable(r) {
  const ins = Array.from(reachFrom([r.i], S.producers)).map(i => S.rows[i]).filter(isInput);
  if (isInput(r) && !ins.includes(r)) ins.unshift(r);
  return ins.sort((a, b) => a.id.localeCompare(b.id));
}

function sweepControls(r) {
  const ins = sweepable(r);
  if (!ins.length) {
    return '<p class="muted">Nothing declared upstream of this node with a range, so there is no ' +
      'decision to sweep.</p>';
  }
  const def = ins.find(x => x.id === 'orbit_altitude') || ins[0];
  const marks = markLevels(r);
  // Only offered when there is something to compare: with nothing saved and
  // nothing edited, the defaults are the case and a second line would lie
  // exactly on the first.
  const moved = (S.saved && S.saved.stored && S.saved.changed) || S.overrides.size;
  return '<h4>behaviour sweep</h4><div class="sweepctl">over <select class="sw-over">' +
    ins.map(x => '<option value="' + esc(x.id) + '"' + (x.id === def.id ? ' selected' : '') + '>' +
      esc(x.id) + '</option>').join('') + '</select>' +
    ' from <input class="sw-from" value="' + esc(fmt(fromSI(def, def.lo))) + '">' +
    ' to <input class="sw-to" value="' + esc(fmt(fromSI(def, def.hi))) + '">' +
    ' <span class="sw-unit">' + esc(unitOf(def.unit)) + '</span>' +
    ' <select class="ctl sw-n" aria-label="points"><option>20</option><option selected>80</option>' +
    '<option>200</option></select> points' +
    ' <button class="ctl sw-go">sweep</button>' +
    ' <span class="muted sw-range"></span></div>' +
    '<div class="sweepctl">' +
    (moved ? '<label class="sw-all-l"><input type="checkbox" class="sw-all">' +
      ' the declared defaults as well, one line each</label>' : '') +
    (marks.length ? ' against <select class="sw-mark"><option value="">nothing — the curve alone</option>' +
      marks.map(x => '<option value="' + esc(x.id) + '">' + esc(x.id) + '</option>').join('') +
      '</select><span class="muted"> a level to read the crossing against — run on the case, ' +
      'and matched by subsystem and overlapping range, not by quantity, which the index cannot ' +
      'tell. Check the row it names.</span>' : '') +
    '</div>' +
    '<canvas class="plot sw-plot" width="900" height="320" hidden></canvas>' +
    '<div class="sw-view"></div>' +
    '<details class="sw-table" hidden><summary>the numbers behind this picture</summary>' +
    '<div class="sw-table-body"></div></details>' +
    '<div class="sw-note muted"></div>';
}

// The design window: a swept curve is only half a picture. What a design reads
// off it is where the curve crosses a level somebody committed to — the mission
// length at which a bound stops being met — and that crossing is a number, not
// an impression.
//
// WHICH ROWS ARE OFFERED, AND WHY THE TEST IS WEAK. A level is only meaningful
// on this axis if it is the same quantity. The index cannot say so: every Ratio
// carries the unit "-", so an F10.7 of 250 and an Ap of 250 are indistinguishable
// to this code. What is used instead is the same subsystem and an overlapping
// declared domain, which is a proxy and is wrong in both directions. The picture
// names the row it drew, so a wrong pairing is visible rather than silent.
//
// A RETIRED ROW IS NOT A LEVEL. It was offered, and a design read against a
// number nothing else reads any more is a design read against nothing.
function markLevels(r) {
  return S.rows
    .filter(x => x.id !== r.id && !isSeeded(x) && !isDeprecated(x) && x.sub === r.sub &&
                 x.hi > x.lo && x.lo <= r.hi && x.hi >= r.lo)
    .sort((a, b) => a.id.localeCompare(b.id));
}

// THE SAME VIEW STATE THE PANELS HAVE: a brush to zoom, Escape to undo, the
// numbers to copy. And a redraw on a theme change, which a canvas does not do
// for itself — without it the sweep kept the light palette on a dark page.
const VIEWS = new WeakMap();
const viewOf = host => {
  let v = VIEWS.get(host);
  if (!v) { v = { zoom: null, hidden: new Set(), pinned: null }; VIEWS.set(host, v); }
  return v;
};
const LIVE = new Set();
watchScheme(() => {
  for (const fn of [...LIVE]) {
    if (fn._host && fn._host.isConnected) fn();
    else LIVE.delete(fn);
  }
});

function wireSweep(host, r) {
  const go2 = $('.sw-go', host);
  if (!go2) return;
  const from = $('.sw-from', host), to = $('.sw-to', host), over = $('.sw-over', host);
  const range = $('.sw-range', host), unit = $('.sw-unit', host);
  const check = () => {
    const d = S.byId.get(over.value);
    // THE HINT FOLLOWS THE SELECTION. It was rendered once while the list was
    // re-ordered and re-picked twice after, and read "150000 … 450000" —
    // orbit_altitude in metres — beside two boxes holding a ratio.
    range.textContent = 'declared range ' + fmt(fromSI(d, d.lo)) + ' … ' + fmt(fromSI(d, d.hi)) +
      (unitOf(d.unit) ? ' ' + unitOf(d.unit) : '');
    unit.textContent = unitOf(d.unit);
    for (const el of [from, to]) {
      const si = toSI(d, parseFloat(el.value));
      // Refuse, never clamp.
      const bad = outOfRange(d, si);
      el.classList.toggle('bad', !!bad);
      el.title = bad ? d.id + ': ' + bad + '. Out of range is refused, not corrected.' : '';
    }
    go2.disabled = from.classList.contains('bad') || to.classList.contains('bad');
  };
  const pick = () => {
    const d = S.byId.get(over.value);
    from.value = fmt(fromSI(d, d.lo));
    to.value = fmt(fromSI(d, d.hi));
    check();
  };
  over.onchange = pick;
  from.oninput = to.oninput = check;
  check();

  // WHICH OF THESE DECISIONS ACTUALLY MOVES THE ANSWER. A term can be in the
  // relation, be right, and still be inert — today's flux is worth 0.0001 per
  // cent at a mission lead — and a list led by it opened eleven solar rows on a
  // flat line. The engine measures each decision at both ends of its range; the
  // list is re-ordered most-moving first and the dead ones say so.
  levers(host, r).then(keep => {
    if (!keep.length || !over.isConnected) return;
    over.innerHTML = keep.map(l => {
      const tag = l.span === null ? ' — cannot be swept: ' + (l.why || 'refused') :
        ' — moves this answer by ' + pct(l.span);
      return '<option value="' + esc(l.id) + '">' + esc(l.id) + esc(tag) + '</option>';
    }).join('');
    over.value = keep[0].id;
    pick();
    const dead = keep.filter(l => l.span === 0).length;
    if (dead) {
      $('.sw-note', host).textContent = dead + ' of ' + keep.length +
        ' decisions upstream of this row do not move its answer at all. They are ' +
        'still listed, because that is a fact about the design and not an omission.';
    }
  });

  let last = null;
  const sweepNow = async again => {
    const d = S.byId.get(over.value);
    const all = $('.sw-all', host);
    // THE CASE, AND WITH IT THE DECLARED DESIGN. The case is the saved inputs
    // with the reader's edits over them; the defaults line is the same sweep
    // with neither, so what the case changed is the gap between two lines.
    const who = [{ id: 'case', name: 'this case' }];
    if (all && all.checked) who.push({ id: 'defaults', name: 'the declared defaults' });
    $('.sw-note', host).textContent = 'sweeping…';
    go2.disabled = true;
    const runs = [];
    for (const c of who) {
      const p = sweepParams(host, r, d);
      // A sweep already saved for exactly this question is read back, and
      // says so; `again` sweeps fresh.
      p.set('reuse', '1');
      if (again) p.set('again', '1');
      if (c.id === 'defaults') p.set('inputs', 'defaults'); else withOverrides(p);
      let res;
      try {
        res = await (await fetch('/v1/sweep?' + p.toString())).json();
      } catch (e) {
        res = { ok: false, message: 'the engine did not answer: ' + e };
      }
      runs.push({ c, res });
    }
    // Where the design sits now: the swept input's current value, and the
    // answer RUN there — not read off the curve, which would be the page
    // interpolating a number the engine never gave.
    const now = S.overrides.has(d.id) ? S.overrides.get(d.id)
      : host._at && host._at.has(d.id) ? host._at.get(d.id) : null;
    let here = null;
    if (now !== null) {
      const hr = await runOnce(withOverrides(new URLSearchParams({ node: r.id, mode: 'branch' })));
      const hv = hr.ok && (hr.values || []).find(x => x.id === r.id);
      if (hv) here = { x: now, y: hv.si };
    }
    last = { d, runs, here, mark: null };
    const mk = $('.sw-mark', host);
    if (mk && mk.value) last.mark = await levelOf(mk.value);
    go2.disabled = false;
    viewOf(host).zoom = null;
    plot(host, r, last);
    keepBar(host, r, last, sweepNow);
  };
  go2.onclick = () => sweepNow(false);
  // Changing the level redraws from the sweep in hand. Re-running the engine to
  // move a horizontal line would be asking a question whose answer cannot have
  // changed.
  const mk = $('.sw-mark', host);
  if (mk) mk.onchange = async () => {
    if (!last) return;
    last.mark = mk.value ? await levelOf(mk.value) : null;
    plot(host, r, last);
  };
}

/** The sweep as the controls ask it: the row, the input, the range in SI, the points. */
function sweepParams(host, r, d) {
  return new URLSearchParams({
    node: r.id, over: d.id, from: toSI(d, parseFloat($('.sw-from', host).value)),
    to: toSI(d, parseFloat($('.sw-to', host).value)), points: $('.sw-n', host).value, mode: 'branch',
  });
}

/**
 * Under a sweep: where it came from, and keeping it. A sweep read back from a
 * saved result says which one, and offers a fresh sweep; one run now can be
 * saved — the run at the case and the line across the range, kept together,
 * and drawn again on the Results page without running.
 */
function keepBar(host, r, last, sweepNow) {
  const note = $('.sw-note', host);
  if (!note) return;
  $$('.sw-keep', host).forEach(b => b.remove());
  const theCaseLine = last.runs.find(x => x.c.id === 'case');
  const fs = theCaseLine && theCaseLine.res.ok && theCaseLine.res.from_saved;
  const bar = document.createElement('div');
  bar.className = 'runbar sw-keep';
  if (fs) {
    bar.innerHTML = '<span class="run-saved"><b>Already answered</b> — this sweep is shown from the saved ' +
      'result ' + (fs.name ? '<b>' + esc(fs.name) + '</b>, ' : '') + 'saved ' + esc(fs.saved) + '; nothing was ' +
      'run. <button class="ctl xref" data-results="' + esc(fs.file) + '">open it in Results</button> ' +
      '<button class="ctl sw-again">sweep again anyway</button></span>';
    note.after(bar);
    $('.sw-again', bar).onclick = () => sweepNow(true);
    return;
  }
  if (!theCaseLine || !theCaseLine.res.ok) return;
  bar.innerHTML = '<input class="ctl sw-name" placeholder="a name for it (optional)" aria-label="a name for ' +
    'the sweep"><button class="ctl sw-save">save this sweep</button><span class="why sw-saved"></span>';
  note.after(bar);
  $('.sw-save', bar).onclick = async () => {
    const said = $('.sw-saved', bar);
    const btn = $('.sw-save', bar);
    btn.disabled = true;
    said.textContent = 'saving…';
    const p = withOverrides(sweepParams(host, r, last.d));
    p.set('label', $('.sw-name', bar).value);
    let out;
    try {
      out = await (await fetch('/v1/results/save', { ...POST, body: p.toString() })).json();
    } catch (e) {
      out = { ok: false, message: 'the engine did not answer: ' + e };
    }
    btn.disabled = false;
    said.innerHTML = !out.ok ? 'not saved: ' + esc(out.message || 'refused')
      : (out.already ? 'already kept — this exact sweep was saved ' + esc(out.saved) + '. '
        : 'saved — ') + '<button class="ctl xref" data-results="' + esc(out.file) + '">open it in Results</button>';
  };
}

/** A level's own answer, RUN on the case — a declared level lives only in the model. */
async function levelOf(id) {
  const rr = await runOnce(withOverrides(new URLSearchParams({ node: id, mode: 'branch' })));
  const own = rr.ok && (rr.values || []).find(v => v.id === id);
  return own && isFinite(own.si)
    ? { id, si: own.si }
    : { id, si: null, why: rr.message || 'the level could not be run' };
}

function plot(host, r, last) {
  const c = $('.sw-plot', host);
  const note = $('.sw-note', host);
  const failed = last.runs.filter(x => !x.res.ok);
  const good = last.runs.filter(x => x.res.ok);
  if (!good.length) {
    note.textContent = (failed[0] && failed[0].res.message) || 'the sweep was refused';
    c.hidden = true;
    return;
  }
  const r0 = good[0].res;
  const fx = r0.x_factor, fy = r0.y_factor;
  const LINES = ['case', 'defaults'];
  // A REFUSED POINT IS A GAP, NOT A JOIN. The line breaks where the engine
  // refused, so a stretch of the axis nobody computed is not drawn straight
  // across as though somebody had.
  const series = good.map(({ c: cu, res }) => {
    const pts = res.x.map((x, i) => [x, res.y[i]])
      .concat(res.refused.map(q => [q.x, null]))
      .sort((a, b) => a[0] - b[0]);
    return {
      name: good.length > 1 ? cu.name : '',
      kind: 'line',
      // Colour follows the line, never its place in this list: the case is
      // the same colour whether or not the defaults are drawn beside it.
      colour: INK.series[LINES.indexOf(cu.id) % INK.series.length],
      width: cu.id === 'case' ? 2.2 : 1.4,
      x: pts.map(p => p[0] / fx),
      y: pts.map(p => (p[1] === null ? null : p[1] / fy)),
    };
  });
  const marks = [];
  if (last.mark && last.mark.si !== null) {
    marks.push({ axis: 'y', at: last.mark.si / fy, label: last.mark.id + ' = ' + fmt(last.mark.si / fy),
                 colour: INK.mark });
  }
  if (last.here) {
    series.push({ name: '', kind: 'dots', x: [last.here.x / fx], y: [last.here.y / fy], width: 5, alpha: 1,
                  colour: INK.text });
  }
  const spec = {
    x: { label: r0.x_id + '  [' + (unitOf(r0.x_unit) || 'dimensionless') + ']' },
    y: { label: r0.y_id + '  [' + (unitOf(r0.y_unit) || 'dimensionless') + ']' },
    series,
    marks,
    notes: last.here ? [{ x: last.here.x / fx, y: last.here.y / fy, text: 'this design, now' }] : [],
  };
  const view = viewOf(host);
  const paint = () => {
    c.hidden = false;
    const shown = viewSpec(spec, view);
    drawChart(c, shown);
    attachHover(c, {
      onBrush: win => {
        if (!win.x) return;
        view.zoom = { ...(view.zoom || {}), x: win.x };
        paint();
      },
      onReset: () => { if (viewIsOn(view)) { view.zoom = null; paint(); } },
    });
    strip(host, spec, shown, view, paint);
  };
  paint._host = host;
  LIVE.add(paint);
  paint();

  // What the picture says, in words: the points, the refusals, whether the
  // case and the defaults coincide, and the crossing.
  const bits = [];
  const refused = good.reduce((n, x) => n + x.res.refused.length, 0);
  const ran = r0.x.length + ' point' + (r0.x.length === 1 ? '' : 's') + ' ran' +
    (good.length > 1 ? ' per line' : '');
  if (refused) {
    const first = good.find(x => x.res.refused.length).res.refused[0];
    bits.push(ran + ', <b>' + refused + ' refused</b> — ' + esc(first.why) + '. Refusals are gaps ' +
      'in the line, never joined across: a sweep in which some points quietly used a substituted ' +
      'value is a sweep whose conclusion is unknown');
  } else {
    bits.push(ran + ', none refused');
  }
  if (good.length > 1) {
    const same = good.slice(1).some(x => x.res.y.length === r0.y.length &&
      x.res.y.every((v, i) => v === r0.y[i]));
    if (same) {
      bits.push('the declared defaults lie exactly on this case — nothing the case changes reaches ' +
        'this row along this sweep');
    }
  }
  for (const f of failed) bits.push('<b>' + esc(f.c.name) + ' was refused</b> — ' + esc(f.res.message || ''));
  if (last.mark) {
    if (last.mark.si === null) {
      bits.push('<b>' + esc(last.mark.id) + ' was not drawn</b> — ' + esc(last.mark.why) +
        '. A level that could not be run is left off rather than guessed at');
    } else {
      const lv = last.mark.si / fy;
      const ys = r0.y.map(v => v / fy), xs = r0.x.map(v => v / fx);
      let cross = null;
      for (let i = 1; i < ys.length; i++) {
        if ((ys[i - 1] - lv) * (ys[i] - lv) <= 0 && ys[i - 1] !== ys[i]) {
          cross = xs[i - 1] + (lv - ys[i - 1]) / (ys[i] - ys[i - 1]) * (xs[i] - xs[i - 1]);
          break;
        }
      }
      bits.push(cross === null
        ? 'the curve does not cross <b>' + esc(last.mark.id) + '</b> anywhere in this range'
        : 'it crosses <b>' + esc(last.mark.id) + '</b> at about ' + esc(fmt(cross)) + ' ' +
          esc(unitOf(r0.x_unit)) + ', between two computed points');
    }
  }
  if (!last.here) bits.push('the current design point is not marked: its value could not be read');
  note.innerHTML = bits.join('. ') + '.';

  const tb = $('.sw-table', host);
  tb.hidden = false;
  $('.sw-table-body', tb).innerHTML = tableFor(spec);
}

/** Under the picture: the window, the way back, and the numbers to copy. */
function strip(host, spec, shown, view, again) {
  const el = $('.sw-view', host);
  if (!el) return;
  const z = view.zoom || {};
  el.innerHTML = (z.x
    ? '<span class="sw-vs">showing ' + esc(fmt(z.x[0])) + ' to ' + esc(fmt(z.x[1])) +
      '</span><button class="ctl sw-unzoom" type="button">the whole range</button>'
    : '') +
    '<button class="ctl sw-copy" type="button">copy as TSV</button>' +
    '<span class="sw-copied"></span>' +
    '<span class="sw-hint muted">hover to read a point, drag across the plot to zoom, ' +
    'double-click or Escape to undo</span>';
  const un = $('.sw-unzoom', el);
  if (un) un.onclick = () => { view.zoom = null; again(); };
  $('.sw-copy', el).onclick = async () => {
    const said = $('.sw-copied', el);
    let okay = false;
    try { await navigator.clipboard.writeText(tableTsv(spec)); okay = true; } catch (e) { okay = false; }
    said.textContent = okay ? 'copied' : 'could not reach the clipboard';
    setTimeout(() => { said.textContent = ''; }, 2000);
  };
}
