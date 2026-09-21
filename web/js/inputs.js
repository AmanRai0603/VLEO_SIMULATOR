/*
  The inputs a reader may move, and what moving one does to everything else.

  THIS IS A WHAT-IF, NOT AN EDIT TO THE DESIGN. Nothing here writes a sheet,
  and nothing here touches the repository. An override lives in this browser
  and travels to the engine as `set=<id>:<si>` on the run, which is the
  mechanism `Case::supply` has always had; the declared value in node.toml is
  what it was before and `git status` is empty after a session of it. That is
  deliberate twice over: rule 1 says the sheet is the only source, and the
  declared number on a row like mission duration carries a `confirmed_by` line
  naming a person and a date. A face that quietly replaced five years with
  seven would have destroyed an attestation and left the row still claiming
  somebody had checked it.

  So the contract this module keeps is: the declared value is always shown
  beside the override, an override is always visibly an override, and one
  click puts it back.

  UNITS. The engine speaks SI and only SI. The index sends `lo`, `hi` and
  `factor`, where the first two are SI and the factor converts the unit a
  person reads. Mission duration is `yr` with a lower bound of 15778800 —
  fifteen million, because that is five hundred days in seconds. Every number
  on its way in is multiplied by the factor and every number on its way out is
  divided by it, in the two functions at the top, and nowhere else.
*/
'use strict';

import { esc, fmt } from './dom.js';
import { S } from './state.js';

/** The key the browser remembers overrides under. Never a file, never the repo. */
const KEY = 'vleo.overrides.v1';

// ---------------------------------------------------------------------------
// the two conversions, and nothing else converts

/**
 * A unit as it should be read.
 *
 * The kernel writes a dimensionless quantity's unit as `-`, which is right in a
 * table of units and wrong beside a number: `8.40971 -` reads as a typo, or as
 * a minus sign that lost its operand. Dimensionless is the absence of a unit,
 * so it is shown as nothing.
 */
export const unitOf = u => (!u || u === '-' || u === 'one') ? '' : u;

/** A number as a person types it, in the row's own unit -> SI. */
export const toSI = (r, shown) => shown * (r.factor || 1);
/** A number as the engine holds it -> the row's own unit, for reading. */
export const fromSI = (r, si) => si / (r.factor || 1);

// ---------------------------------------------------------------------------
// which rows are inputs

/**
 * A row a reader may move.
 *
 * `declared` is the kernel's own word for a row whose answer is a number
 * somebody chose rather than one the tool derived — which is exactly the set
 * that can be supplied. The range test is the same one the levers endpoint
 * applies: a row whose upper bound does not exceed its lower has no room to be
 * moved in, and offering a field for it is offering a control that cannot do
 * anything.
 */
export const isInput = r =>
  !!r && r.kind === 'declared' && r.state !== 'empty' && r.hi > r.lo;

// ---------------------------------------------------------------------------
// the store

/**
 * Read what this browser remembered.
 *
 * Wrapped because storage throws rather than returns empty in a private
 * window, and a tool that will not boot without it is a tool that will not
 * boot. An unreadable store is an empty one.
 */
export function loadOverrides() {
  let raw = null;
  try {
    raw = window.localStorage.getItem(KEY);
  } catch (e) {
    return;
  }
  if (!raw) return;
  try {
    const o = JSON.parse(raw);
    for (const k of Object.keys(o)) {
      if (typeof o[k] === 'number' && isFinite(o[k])) S.overrides.set(k, o[k]);
    }
  } catch (e) {
    // Malformed is the same as absent. It is not worth a message: the reader
    // did not put it there and cannot act on it.
  }
}

function persist() {
  try {
    const o = {};
    for (const [k, v] of S.overrides) o[k] = v;
    window.localStorage.setItem(KEY, JSON.stringify(o));
  } catch (e) {
    // The session still works; only the remembering is lost.
  }
}

/**
 * Who to tell when the set of overrides changes.
 *
 * The shell draws the what-if bar and the node page draws the control, and the
 * one that must never be stale is the bar: it is the only thing on screen
 * saying the numbers below it are not the design. Setting an override from the
 * node page has to reach it.
 *
 * It is a subscription rather than a call into the shell because the module
 * graph here is a tree on purpose — app imports the views, the views import
 * the state, and nothing imports app. A view reaching for `draw()` would close
 * that loop for the sake of one redraw.
 */
const listeners = [];
export function onOverrideChange(fn) { listeners.push(fn); }
const changed = () => { persist(); for (const f of listeners) f(); };

/** Set one override, in SI. Out of range is refused here AND by the engine. */
export function setOverride(id, si) {
  S.overrides.set(id, si);
  changed();
}

export function clearOverride(id) {
  S.overrides.delete(id);
  changed();
}

export function clearAllOverrides() {
  S.overrides.clear();
  changed();
}

/** How many are live. The number the shell shows. */
export const overrideCount = () => S.overrides.size;

/**
 * The overrides as the engine takes them.
 *
 * An id that is not a row any more is dropped rather than sent: a remembered
 * override outlives a reseed, and the engine's refusal for an unknown variable
 * would blank a whole run over a stale browser key.
 */
export function overrideParams() {
  const out = [];
  for (const [id, si] of S.overrides) {
    if (S.byId.has(id)) out.push(['set', id + ':' + si]);
  }
  return out;
}

/** Those same pairs appended to a URLSearchParams, for a GET or a POST body. */
export function withOverrides(params) {
  for (const [k, v] of overrideParams()) params.append(k, v);
  return params;
}

// ---------------------------------------------------------------------------
// the range test, which is the engine's own

/**
 * Whether a value in SI is inside the row's declared range.
 *
 * The bound is the same `VARS[i].limit` the kernel's `supply_checked` compares
 * against, sent on the index, so this cannot drift from what the engine will
 * accept. It is applied here only to stop a pointless round trip and to say
 * why before the reader presses update — if anything slips through, the
 * engine refuses it and names the bound, the value and the reason, and the
 * run panel prints that refusal as it prints any other.
 */
export function outOfRange(r, si) {
  if (!isFinite(si)) return 'not a number';
  if (si < r.lo) {
    return 'below ' + fmt(fromSI(r, r.lo)) + ' ' + unitOf(r.unit) +
      (r.why_lo ? ' — ' + r.why_lo : '');
  }
  if (si > r.hi) {
    return 'above ' + fmt(fromSI(r, r.hi)) + ' ' + unitOf(r.unit) +
      (r.why_hi ? ' — ' + r.why_hi : '');
  }
  return '';
}

// ---------------------------------------------------------------------------
// what a change moved

/**
 * Two runs compared, row by row.
 *
 * This is the question the reader actually asked: not "what is mission
 * duration now" but "what did changing it move". Both sides are a full run, so
 * a row appears here because the engine computed a different number for it,
 * never because something in the face decided it should have.
 *
 * A row that ran in one and not the other is reported too, under `gained` or
 * `lost`. Those are the interesting ones — an override that pushes a row over
 * a guard removes a number from the design, and a diff that only compared the
 * rows present in both would show that as nothing at all.
 */
export function diffRuns(before, after) {
  const b = new Map((before && before.values || []).map(v => [v.id, v]));
  const a = new Map((after && after.values || []).map(v => [v.id, v]));
  const moved = [];
  const gained = [];
  const lost = [];
  for (const [id, av] of a) {
    const bv = b.get(id);
    if (!bv) { gained.push(av); continue; }
    if (bv.si === av.si) continue;
    // Relative where there is something to be relative to. A row whose before
    // was zero reports an absolute move rather than an infinity.
    const rel = bv.si !== 0 ? (av.si - bv.si) / Math.abs(bv.si) : null;
    moved.push({ id, label: av.label, symbol: av.symbol, unit: av.unit,
                 from: bv.si, to: av.si, shown: av.shown, was: bv.shown, rel });
  }
  for (const [id, bv] of b) if (!a.has(id)) lost.push(bv);
  // Biggest relative move first: the reader wants the row this decision
  // actually drives, not the alphabetical first one it touched.
  moved.sort((x, y) => Math.abs(y.rel == null ? 0 : y.rel) - Math.abs(x.rel == null ? 0 : x.rel));
  return { moved, gained, lost };
}

// ---------------------------------------------------------------------------
// drawing

/**
 * The control for one input row: what it was, what it is, and its range.
 *
 * The declared value is not decoration. It is the number a person put their
 * name against, and the whole reason this control is safe to offer is that it
 * stays on screen beside whatever the reader has typed over it.
 */
export function inputControl(r, declaredSI) {
  if (!isInput(r)) return '';
  const has = S.overrides.has(r.id);
  const si = has ? S.overrides.get(r.id) : declaredSI;
  const shown = si == null ? '' : fmt(fromSI(r, si));
  const dec = declaredSI == null ? null : fromSI(r, declaredSI);

  return '<div class="ovr' + (has ? ' on' : '') + '" data-ovr="' + esc(r.id) + '">' +
    '<div class="ovr-h"><b>' + esc(r.symbol || r.id) + '</b> ' +
      '<span class="muted">' + esc(r.label) + '</span>' +
      (has ? '<span class="ovr-tag">what-if</span>' : '') + '</div>' +
    '<div class="ovr-row">' +
      '<label class="ovr-dec">declared' +
        '<span>' + (dec == null ? '<i class="muted">not run</i>' : esc(fmt(dec)) + ' ' + esc(unitOf(r.unit))) +
        '</span></label>' +
      '<label class="ovr-in">running at' +
        '<input class="ovr-v" type="number" step="any" value="' + esc(shown) + '"' +
        ' aria-label="' + esc(r.label) + ' in ' + esc(r.unit) + '">' +
        '<span class="ovr-u">' + esc(unitOf(r.unit)) + '</span></label>' +
      '<button class="ctl ovr-reset"' + (has ? '' : ' disabled') +
        ' title="' + (has ? 'put it back to the declared value' :
          'nothing to put back — this is the declared value') + '">reset</button>' +
    '</div>' +
    '<div class="ovr-range muted">' + esc(fmt(fromSI(r, r.lo))) + ' … ' +
      esc(fmt(fromSI(r, r.hi))) + ' ' + esc(unitOf(r.unit)) +
      ' <span class="ovr-why"></span></div>' +
    '</div>';
}

/** The moved rows, as a table a person reads top-down. */
export function diffHtml(d, limit) {
  const n = d.moved.length + d.gained.length + d.lost.length;
  if (!n) {
    return '<p class="muted">Nothing moved. Every row the engine computed came ' +
      'back with the number it had before — so on this case, nothing downstream ' +
      'reads this input, or it reads it somewhere the value does not reach.</p>';
  }
  const cut = limit || 60;
  let h = '<p class="ovr-count"><b>' + n + '</b> row' + (n === 1 ? '' : 's') + ' moved.</p>';
  h += '<table class="ovr-diff"><thead><tr><th>row</th><th>was</th><th>now</th>' +
    '<th>change</th></tr></thead><tbody>';
  for (const m of d.moved.slice(0, cut)) {
    const pct = m.rel == null ? '—' : (m.rel * 100 >= 0 ? '+' : '') + fmt(m.rel * 100) + '%';
    // The row id is a cross-reference like every other id in the face, so the
    // one click rule in app.js carries it. A second navigation path wired here
    // would be a second answer to "what does clicking an id do".
    h += '<tr><td><a class="xref" data-goto="' + esc(m.id) + '"><code>' +
      esc(m.id) + '</code></a><br>' +
      '<span class="muted">' + esc(m.label) + '</span></td>' +
      '<td>' + esc(m.was) + '</td><td><b>' + esc(m.shown) + '</b>' +
      (unitOf(m.unit) ? ' <span class="unit">' + esc(unitOf(m.unit)) + '</span>' : '') + '</td>' +
      '<td class="' + (m.rel == null ? '' : (m.rel > 0 ? 'up' : 'down')) + '">' + esc(pct) + '</td></tr>';
  }
  h += '</tbody></table>';
  if (d.moved.length > cut) {
    h += '<p class="muted">… and ' + (d.moved.length - cut) + ' more.</p>';
  }
  // A row that stopped computing is the finding, not a footnote.
  if (d.lost.length) {
    h += '<div class="blocked"><b>' + d.lost.length + ' row' +
      (d.lost.length === 1 ? '' : 's') + ' stopped returning a number</b> under this ' +
      'input. That is a guard refusing, not a row going quiet — open it and the ' +
      'refusal names the bound.<div>' +
      d.lost.slice(0, 20).map(v => esc(v.id)).join(', ') +
      (d.lost.length > 20 ? ', and ' + (d.lost.length - 20) + ' more' : '') + '</div></div>';
  }
  if (d.gained.length) {
    h += '<p class="muted"><b>' + d.gained.length + '</b> row' +
      (d.gained.length === 1 ? '' : 's') + ' began returning a number that did not before.</p>';
  }
  return h;
}

// ---------------------------------------------------------------------------
// the engine calls this module makes for itself

/**
 * One run, with the overrides or deliberately without them.
 *
 * `bare` is how the baseline is taken: the same case, the same mode, the same
 * everything, with nothing supplied. Comparing a run against a baseline taken
 * any other way would attribute to the input whatever else differed.
 */
async function runOnce(node, mode, bare) {
  const p = new URLSearchParams({ node, mode, case: S.engineCase });
  if (!bare) withOverrides(p);
  try {
    return await (await fetch('/v1/run', {
      method: 'POST',
      headers: { 'content-type': 'application/x-www-form-urlencoded' },
      body: p.toString(),
    })).json();
  } catch (e) {
    return { ok: false, fault: 'no answer', node, message: 'the engine did not answer: ' + e };
  }
}

// ---------------------------------------------------------------------------
// the branches an input is in

/**
 * The active branches this input is part of, as the ENGINE computes them.
 *
 * This used to be a graph walk here, over the index. It was correct — it agreed
 * with the engine on all 130 editable inputs — and it was still the wrong place
 * for it: the audit in tools/ needs the same answer, and a rule with two
 * implementations is a rule that drifts. `/v1/branches` is now the one of them.
 *
 * What comes back: `branches`, the maximal active ones, biggest first; and
 * `read_by`, how many rows read this one at all. The second is what lets an
 * empty list say which of two different things it means.
 */
export async function activeBranches(row) {
  if (!row) return { branches: [], read_by: 0 };
  try {
    const r = await (await fetch('/v1/branches?node=' +
      encodeURIComponent(row.id))).json();
    return r && r.ok ? { branches: r.branches || [], read_by: r.read_by || 0 }
                     : { branches: [], read_by: 0 };
  } catch (e) {
    return { branches: [], read_by: 0, failed: true };
  }
}

// ---------------------------------------------------------------------------
// the control

/**
 * Draw and wire the input control for one row.
 *
 * ONE ASYNC STEP, THEN ONE RENDER. The baseline run is taken before anything
 * is drawn, because it carries both things the control needs: the declared
 * value for this row, and the numbers every other row has before the reader
 * touches it. Fetching the declared value after drawing meant repainting the
 * control from inside its own change event, which destroyed the element the
 * event was travelling through — the browser said so, and a click on update
 * could be swallowed by the repaint the blur triggered. Nothing here repaints
 * itself any more: the field is built once and only ever has a class toggled.
 */
export async function mountInput(host, r) {
  if (!host || !isInput(r)) return;
  host.innerHTML = '<p class="muted">reading the design as declared…</p>';

  const base = await runOnce(r.id, 'all', true);
  if (!base.ok) {
    host.innerHTML = '<div class="blocked"><b>' + esc(base.fault || 'refused') + '</b> · ' +
      esc(base.node || '') + '<div>' + esc(base.message || '') + '</div></div>' +
      '<p class="muted">The design does not run as declared on this case, so there is ' +
      'nothing to change it against.</p>';
    return;
  }
  S.baseline = { case: S.engineCase, node: r.id, res: base };
  const bv = base.values.find(v => v.id === r.id);
  const declared = bv ? bv.si : null;

  host.innerHTML = inputControl(r, declared) +
    '<div class="ovr-act">' +
      '<button class="ctl ovr-go">update — run everything</button>' +
      '<span class="ovr-say muted"></span>' +
    '</div><div class="ovr-out"></div>';

  const box = host.querySelector('.ovr');
  const field = host.querySelector('.ovr-v');
  const why = host.querySelector('.ovr-why');
  const reset = host.querySelector('.ovr-reset');
  const go = host.querySelector('.ovr-go');
  const say = host.querySelector('.ovr-say');
  const out = host.querySelector('.ovr-out');

  /** What the field currently says, in SI, and whether the engine would take it. */
  const read = () => {
    const raw = field.value.trim();
    if (raw === '') return { empty: true };
    const si = toSI(r, Number(raw));
    return { si, bad: outOfRange(r, si) };
  };

  /** The only thing that changes about this control: is it a what-if, or not. */
  const mark = () => {
    const on = S.overrides.has(r.id);
    box.classList.toggle('on', on);
    reset.disabled = !on;
    let tag = box.querySelector('.ovr-tag');
    if (on && !tag) {
      tag = document.createElement('span');
      tag.className = 'ovr-tag';
      tag.textContent = 'what-if';
      box.querySelector('.ovr-h').appendChild(tag);
    } else if (!on && tag) {
      tag.remove();
    }
  };

  const commit = () => {
    const v = read();
    why.textContent = v.empty || !v.bad ? '' : '— refused: ' + v.bad;
    box.classList.toggle('bad', !!(v.bad && !v.empty));
    if (v.empty || v.bad) return;
    // Typing the declared number back is the absence of an override, not one
    // whose value happens to match.
    if (declared != null && v.si === declared) clearOverride(r.id);
    else setOverride(r.id, v.si);
    mark();
  };

  field.oninput = commit;
  field.onchange = commit;
  field.onkeydown = e => { if (e.key === 'Enter') go.click(); };
  reset.onclick = () => {
    clearOverride(r.id);
    if (declared != null) field.value = fmt(fromSI(r, declared));
    why.textContent = '';
    box.classList.remove('bad');
    mark();
  };

  go.onclick = async () => {
    commit();
    if (box.classList.contains('bad')) return;
    go.disabled = true;
    out.innerHTML = '';
    try {
      await runStages(r, base, out, t => { say.textContent = t; });
    } finally {
      go.disabled = false;
      say.textContent = '';
    }
  };

  mark();
}

/**
 * The three runs an edited input asks for, in order, each drawn as it lands.
 *
 * 1 · ALONE. This row and nothing else, so the number the reader typed is
 *     confirmed to be the number the engine took. It is the shortest possible
 *     answer to "did my edit arrive", and when it disagrees with the field the
 *     fault is between the face and the engine rather than anywhere in the
 *     design.
 *
 * 2 · EACH ACTIVE BRANCH. A row is in as many branches as there are rows that
 *     read it, so the branches are found rather than chosen — see
 *     activeBranches — and each is run on its own. One branch failing is a fact
 *     about that branch: the others still run and still report, because a
 *     single refusal stopping the whole list would hide every branch behind it.
 *
 * 3 · EVERYTHING. The final update. Whatever is active recomputes and whatever
 *     is not stays blocked and is counted, and the rows that moved are listed
 *     against the baseline.
 *
 * Drawn incrementally: stage 2 can be a dozen engine calls and a reader should
 * not watch a blank panel while they run.
 */
export async function runStages(r, base, out, say) {
  const sec = (n, title, body) =>
    '<section class="ovr-stage"><h4><span class="ovr-n">' + n + '</span>' + title +
    '</h4>' + body + '</section>';

  // ---- 1 · alone --------------------------------------------------------
  say('1 · running this row alone…');
  const alone = await runOnce(r.id, 'alone', false);
  let h = sec(1, 'alone — this row, and nothing else', aloneHtml(r, alone));
  out.innerHTML = h;

  // ---- 2 · the active branches -----------------------------------------
  const found = await activeBranches(r);
  const branches = found.branches;
  // AN EMPTY LIST MEANS ONE OF TWO DIFFERENT THINGS and a reader needs to know
  // which. Seven solar rows are read by nothing in the tree at all — they are
  // answers a person reads off a figure, and there is no branch to run because
  // there is nothing downstream, not because anything is unfinished. That is a
  // different sentence from "everything that reads this is still seeded".
  const head = branches.length
    ? '<p class="muted">' + branches.length + ' active branch' +
      (branches.length === 1 ? '' : 'es') + ', of the ' + found.read_by +
      ' row' + (found.read_by === 1 ? '' : 's') + ' that read this one. Each is a ' +
      'dependency closure whose every row is published, and between them they ' +
      'cover every row in every active branch this input is in.</p>'
    : found.failed
      ? '<p class="muted">The engine did not answer when asked which branches this ' +
        'row is in.</p>'
      : found.read_by === 0
        ? '<p class="muted">Nothing in the tree reads this row. Its answer is read ' +
          'by a person, off a figure, so there is no branch to run — that is what ' +
          'this row is for, not something missing from it.</p>'
        : '<p class="muted">' + found.read_by + ' row' +
          (found.read_by === 1 ? '' : 's') + ' read this one, and none of them sits ' +
          'in a branch that is fully published, so there is nothing that would run ' +
          'end to end. That is how far the tree is filled in, not a failure.</p>';
  h += sec(2, 'branch by branch — every active branch this row is in',
           head + '<div class="ovr-branches"></div>');
  out.innerHTML = h;

  const list = out.querySelector('.ovr-branches');
  const done = [];
  for (let k = 0; k < branches.length; k++) {
    const b = branches[k];
    say('2 · branch ' + (k + 1) + ' of ' + branches.length + ' — ' + b.id);
    const res = await runOnce(b.id, 'branch', false);
    done.push(branchRow(b, res, base));
    list.innerHTML = '<table class="ovr-diff ovr-bt"><thead><tr><th>branch</th>' +
      '<th class="num">rows</th><th>was</th><th>now</th></tr></thead><tbody>' +
      done.join('') + '</tbody></table>';
  }

  // ---- 3 · everything ---------------------------------------------------
  say('3 · running everything…');
  const after = await runOnce(r.id, 'all', false);
  h = out.innerHTML + sec(3, 'everything — the final update',
    resultHtml({ before: base, after, diff: after.ok ? diffRuns(base, after) : null }));
  out.innerHTML = h;
}

/** Stage one: the row's own answer, or the refusal that replaced it. */
function aloneHtml(r, res) {
  if (!res.ok) {
    return '<div class="blocked"><b>' + esc(res.fault || 'refused') + '</b> · ' +
      esc(res.node || '') + '<div>' + esc(res.message || '') + '</div></div>';
  }
  const v = res.values.find(x => x.id === r.id);
  if (!v) {
    return '<div class="blocked">The engine ran but returned no value for this row.' +
      (res.blocked && res.blocked.length
        ? '<div>' + esc(res.blocked[0].message) + '</div>' : '') + '</div>';
  }
  return '<div class="answer">' + esc(v.symbol || r.symbol) + ' = ' + esc(v.shown) +
    (unitOf(v.unit) ? ' <span class="unit">' + esc(unitOf(v.unit)) + '</span>' : '') +
    '</div>';
}

/** Stage two: one branch, its head\u2019s new output, or why it refused. */
function branchRow(b, res, base) {
  const name = '<td><a class="xref" data-goto="' + esc(b.id) + '"><code>' +
    esc(b.id) + '</code></a><br><span class="muted">' + esc(b.label) +
    '</span></td><td class="num">' + b.rows + '</td>';
  if (!res.ok) {
    return '<tr class="ovr-refused">' + name + '<td colspan="2"><b>' +
      esc(res.fault || 'refused') + '</b> — ' + esc(res.message || '') + '</td></tr>';
  }
  const v = res.values.find(x => x.id === b.id);
  const was = (base.values || []).find(x => x.id === b.id);
  if (!v) {
    const why = (res.blocked || []).find(x => x.id === b.id);
    // WHOSE FAULT IS THIS. A row that was already refusing before the edit is
    // not evidence about the edit, and colouring it like a new failure is a
    // false alarm the reader learns to ignore — which then hides the real one.
    // The baseline says which: no value there either means it was already
    // blocked, and the edit only has to answer for a row that HAD a number.
    const already = !was;
    return '<tr class="' + (already ? 'ovr-pre' : 'ovr-refused') + '">' + name +
      '<td>' + (already ? '<span class="muted">blocked</span>' : esc(was.shown)) + '</td>' +
      '<td><b>blocked</b>' + (why ? ' — ' + esc(why.message) : '') +
      (already
        ? '<div class="muted">already blocked before this edit — not caused by it</div>'
        : '<div><b>this edit blocked it</b></div>') + '</td></tr>';
  }
  const moved = was && was.si !== v.si;
  return '<tr>' + name + '<td>' + esc(was ? was.shown : '—') + '</td>' +
    '<td' + (moved ? ' class="up"' : '') + '><b>' + esc(v.shown) + '</b>' +
    (unitOf(v.unit) ? ' <span class="unit">' + esc(unitOf(v.unit)) + '</span>' : '') +
    (moved ? '' : ' <span class="muted">unchanged</span>') + '</td></tr>';
}

/** What update produced: the refusal, or what moved. */
export function resultHtml(res) {
  if (res.after && !res.after.ok) {
    return '<div class="blocked"><b>' + esc(res.after.fault || 'refused') + '</b> · ' +
      esc(res.after.node || '') + '<div>' + esc(res.after.message || '') + '</div></div>' +
      '<p class="muted">The engine refused this input. A refusal is a value: it names ' +
      'the field, the bound it broke and the reason that bound exists.</p>';
  }
  if (!res.diff) return '<p class="muted">No comparison could be made.</p>';
  const m = res.after.manifest;
  let h = '<div class="chainline">' + m.ran + ' ran · ' + m.blocked + ' blocked · chain ' +
    esc(m.chain) + '</div>';
  // Anything the engine refused on this run is named, not counted. A row that
  // stopped returning a number because of this input is the finding.
  if (res.after.blocked && res.after.blocked.length && res.diff.lost.length) {
    const ids = new Set(res.diff.lost.map(v => v.id));
    const now = res.after.blocked.filter(b => ids.has(b.id));
    if (now.length) {
      h += '<div class="blocked"><b>' + now.length + ' row' + (now.length === 1 ? '' : 's') +
        ' stopped returning a number under this input</b>' +
        now.slice(0, 10).map(b => '<div>' + esc(b.id) + ' — ' + esc(b.message) + '</div>').join('') +
        (now.length > 10 ? '<div>… and ' + (now.length - 10) + ' more</div>' : '') + '</div>';
    }
  }
  return h + diffHtml(res.diff);
}
