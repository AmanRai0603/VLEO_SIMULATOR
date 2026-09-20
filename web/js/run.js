/*
  The run, the result and the behaviour sweep.

  Everything crossing the boundary is SI. A face converts for display and never
  for transport, so nothing in this module does arithmetic on a value beyond
  dividing by the unit factor the engine sent with it.

  Every control is scoped to the host it was rendered into. The node page and
  the run page both hold a run panel, and a query that reaches the document
  root finds whichever one comes first in the markup — which is how a result
  once landed in a hidden panel and the visible one stayed blank.
*/
'use strict';

import { $, $$, esc, fmt } from './dom.js';
import { S, reachFrom, isSeeded } from './state.js';
import { withOverrides } from './inputs.js';

export function renderRun(host, r, standalone) {
  if (!host || !r) return;
  const known = new Set((S.lastRun && S.lastRun.values || []).map(v => v.id));
  const missing = r.in.map(i => S.rows[i].id).filter(x => !known.has(x));
  const closure = reachFrom([r.i], S.producers).size + 1;
  const seeded = isSeeded(r);

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
  h += '<div class="runbar">' +
    '<button class="ctl mode' + (S.mode === 'alone' ? ' sel' : '') + '" data-mode="alone">alone</button>' +
    '<button class="ctl mode' + (S.mode === 'branch' ? ' sel' : '') + '" data-mode="branch">the branch</button>' +
    '<button class="ctl mode' + (S.mode === 'all' ? ' sel' : '') + '" data-mode="all">everything</button>' +
    '<span class="lbl">case</span><select class="ctl engine-case">' +
    S.index.cases.map(c => '<option value="' + esc(c.id) + '"' +
      (c.id === S.engineCase ? ' selected' : '') + ' title="' + esc(c.note) + '">' +
      esc(c.label) + '</option>').join('') + '</select>' +
    '<button class="ctl run-go"' + (seeded ? ' disabled' : '') + '>run</button>' +
    '<span class="why run-why"></span></div>';

  if (seeded) {
    h += '<p class="empty">This row is seeded. The folder, the sheet and the row exist; nothing is ' +
      'specified in them yet, so the generated stub returns <code>NotRun</code> rather than a number. ' +
      'Six hundred grey rows on day one is not a failure — it is the decomposition, written down ' +
      'before anyone has been told to fill it in.</p>';
  }
  h += '<div class="run-out"></div>';
  host.innerHTML = h;

  // A disabled control that does not say why is a defect.
  const aloneBtn = $('.mode[data-mode="alone"]', host);
  aloneBtn.disabled = missing.length > 0;
  aloneBtn.title = missing.length
    ? 'Disabled: ' + missing.slice(0, 4).join(', ') +
      (missing.length > 4 ? ' and ' + (missing.length - 4) + ' more' : '') + ' ' +
      (missing.length === 1 ? 'has' : 'have') + ' never run. Run the branch instead.'
    : 'Only this node. Upstream values are whatever the store already holds.';
  if (aloneBtn.disabled && S.mode === 'alone') S.mode = 'branch';
  $('.run-why', host).textContent = S.mode === 'branch' ? closure + ' nodes in the closure'
    : S.mode === 'all' ? S.rows.length + ' rows, everything buildable'
    : '1 node';

  $$('.mode', host).forEach(b => b.onclick = () => { S.mode = b.dataset.mode; renderRun(host, r, standalone); });
  $('.engine-case', host).onchange = e => { S.engineCase = e.target.value; S.lastRun = null; renderResult(host, r); };
  $('.run-go', host).onclick = () => go(host, r, standalone);
  const t = $('.run-target', host);
  if (t) t.onchange = e => { S.runTarget = e.target.value; renderRun(host, S.byId.get(e.target.value), true); };
  renderResult(host, r);
}

async function go(host, r, standalone) {
  $('.run-go', host).disabled = true;
  $('.run-why', host).textContent = 'running…';
  // The overrides travel with every run. A face that showed a what-if number
  // on one panel and the declared design on another would be the three-correct-
  // numbers-at-three-different-times bug this tool already has a comment about.
  const body = withOverrides(
    new URLSearchParams({ node: r.id, mode: S.mode, case: S.engineCase }));
  let res;
  try {
    res = await (await fetch('/v1/run', {
      method: 'POST',
      headers: { 'content-type': 'application/x-www-form-urlencoded' },
      body: body.toString(),
    })).json();
  } catch (e) {
    res = { ok: false, fault: 'no answer', node: r.id, message: 'the engine did not answer: ' + e };
  }
  S.lastRun = res.ok ? res : null;
  renderRun(host, r, standalone);
  renderResult(host, r, res);
}

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
  if (v) {
    h += '<div class="answer">' + esc(v.symbol || r.symbol) + ' = ' + esc(v.shown) +
      ' <span class="unit">' + esc(v.unit) + '</span></div>' + credBars(v) +
      '<p class="muted">Eight factors, and the lowest governs — averaging would let a strong factor ' +
      'hide a zero, which is the whole failure the score exists to prevent. This node is held down by ' +
      '<b>' + esc(v.governing) + '</b>. Never stored: a badge read out of a field is a claim about last March.</p>';
  } else {
    h += '<div class="answer none">not computed on this run</div>';
  }
  h += '<div class="chainline">' + res.manifest.ran + ' ran · ' + res.manifest.blocked +
    ' blocked · ' + res.manifest.iterations + ' cycle sweep' +
    (res.manifest.iterations === 1 ? '' : 's') + '</div>';
  if (res.blocked.length) {
    h += '<div class="blocked">' + res.blocked.slice(0, 40).map(b =>
      '<div>' + esc(b.id) + ' — ' + esc(b.message) + '</div>').join('') +
      (res.blocked.length > 40 ? '<div>… and ' + (res.blocked.length - 40) + ' more</div>' : '') + '</div>';
  }
  if (res.verdicts.length) {
    h += '<h4>evidence, executed on this run</h4><div class="verdicts">' + res.verdicts.map(x =>
      '<div class="' + (x.passed ? 'pass' : 'fail') + '">' + (x.passed ? '✓' : '✗') + ' ' +
      esc(x.node) + ' · ' + esc(x.label) + ' — got ' + fmt(x.got) + ', expected ' + fmt(x.expected) +
      ', relative error ' + x.error.toExponential(2) + ' against ' + x.tolerance.toExponential(1) +
      ' <span class="muted">' + esc(x.provenance) + ' / ' + esc(x.source) + '</span></div>').join('') + '</div>';
  }
  h += '<div class="chainline">kernel ' + esc(res.manifest.kernel) + ' · graph ' + esc(res.manifest.graph) +
    ' · case ' + esc(res.manifest.case) + ' · chain ' + esc(res.manifest.chain) +
    ' · endpoint ' + esc(res.manifest.endpoint) +
    (res.manifest.data.length ? ' · data ' + esc(res.manifest.data.join(' ')) : ' · no data bundle') + '</div>';
  h += '<p class="muted">The chain hash is the run\'s identity and its cache key. It covers every node ' +
    'the run reached and each one\'s implementation content — not the case alone, because rewriting a ' +
    'node\'s arithmetic without touching its interface must invalidate every result downstream of it.</p>';
  h += sweepControls(r);
  el.innerHTML = h;
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
// the sweep — the one place a canvas is right, because it is a field and not
// prose. Drawn from the numbers the engine returned; the page computes nothing.

function sweepControls(r) {
  // Every declared value the node depends on, however far upstream: a design
  // sweep asks about a decision, and the decisions are the declared rows. A
  // list of direct inputs would offer computed intermediates, which is asking
  // what happens if a consequence changes.
  const ins = Array.from(reachFrom([r.i], S.producers))
    .map(i => S.rows[i])
    .filter(x => x.kind === 'declared' && x.hi > x.lo)
    .sort((a, b) => a.id.localeCompare(b.id));
  if (!ins.length) {
    return '<p class="muted">Nothing declared upstream of this node with a range, so there is no ' +
      'decision to sweep.</p>';
  }
  // The order here is the graph's, which is not the order a reader wants: it
  // puts whatever sorts first in front, and that is routinely a decision the
  // answer does not depend on. wireSweep asks the engine which of these
  // actually move this row and re-orders the list from the reply; until it
  // comes back the list is alphabetical, which is at least stable.
  const def = ins.find(x => x.id === 'orbit_altitude') || ins[0];
  return '<h4>behaviour sweep</h4><div class="sweepctl">over <select class="sw-over">' +
    ins.map(x => '<option value="' + esc(x.id) + '"' + (x.id === def.id ? ' selected' : '') + '>' +
      esc(x.id) + '</option>').join('') + '</select>' +
    ' from <input class="sw-from" value="' + def.lo + '"> to <input class="sw-to" value="' + def.hi + '">' +
    ' <span class="muted">SI, and the declared range is ' + fmt(def.lo) + ' … ' + fmt(def.hi) + '</span>' +
    ' <button class="ctl sw-go">sweep</button></div>' +
    markControl(r) +
    '<canvas class="plot sw-plot" width="900" height="300" hidden></canvas>' +
    '<div class="sw-note muted"></div>';
}

// The design window: a swept curve is only half a picture. What a design reads
// off it is where the curve crosses a level somebody committed to — the mission
// length at which a bound stops being met — and that crossing is a number, not
// an impression.
//
// The level is obtained by RUNNING the row that holds it, not by reading a
// table. The index carries every row's declared domain but not its answer, and
// a declared value lives only inside the generated model. Running is also the
// only way a COMPUTED level can be drawn at all, which matters here: the design
// bound is sw_ap_design, computed from a G level, and it is the line a design
// most wants to see.
//
// WHICH ROWS ARE OFFERED, AND WHY THE TEST IS WEAK. A level is only meaningful
// on this axis if it is the same quantity. The index cannot say so: every Ratio
// carries the unit "-", so an F10.7 of 250 and an Ap of 250 are indistinguishable
// to this code. What is used instead is the same subsystem and an overlapping
// declared domain, which is a proxy and is wrong in both directions — it will
// offer a row of a different quantity whose range happens to overlap, and it
// will hide a comparable one whose range does not. The picture names the row it
// drew, so a wrong pairing is visible rather than silent, and that is the whole
// defence.
function markLevels(r) {
  return S.rows
    .filter(x => x.id !== r.id && !isSeeded(x) && x.sub === r.sub &&
                 x.hi > x.lo && x.lo <= r.hi && x.hi >= r.lo)
    .sort((a, b) => a.id.localeCompare(b.id));
}

function markControl(r) {
  const m = markLevels(r);
  if (!m.length) return '';
  return '<div class="sweepctl">against <select class="sw-mark">' +
    '<option value="">nothing — the curve alone</option>' +
    m.map(x => '<option value="' + esc(x.id) + '">' + esc(x.id) + '</option>').join('') +
    '</select><span class="muted">a level to read the crossing against. It is run, not looked up, ' +
    'and it is matched by subsystem and overlapping range — not by quantity, which the index cannot ' +
    'tell. Check the row it names.</span></div>';
}

function wireSweep(host, r) {
  let lastRes = null;
  const go2 = $('.sw-go', host);
  if (!go2) return;
  const from = $('.sw-from', host), to = $('.sw-to', host), over = $('.sw-over', host);
  const check = () => {
    const d = S.byId.get(over.value);
    for (const el of [from, to]) {
      const v = parseFloat(el.value);
      // Refuse, never clamp. A value silently corrected is a design that
      // drifted without anyone deciding to.
      const bad = !isFinite(v) || v < d.lo || v > d.hi;
      el.classList.toggle('bad', bad);
      el.title = bad ? d.id + ' is declared valid over ' + d.lo + ' … ' + d.hi +
        '. Out of range is refused, not corrected.' : '';
    }
    go2.disabled = from.classList.contains('bad') || to.classList.contains('bad');
  };
  over.onchange = () => { const d = S.byId.get(over.value); from.value = d.lo; to.value = d.hi; check(); };
  from.oninput = to.oninput = check;
  check();

  // WHICH OF THESE DECISIONS ACTUALLY MOVES THE ANSWER.
  //
  // Offering every declared row upstream is correct and insufficient. A term
  // can be in the relation, be right, and still be inert: sw_central_expectation
  // weights today's flux by exp(-lead/27), which at any mission lead is worth
  // 0.0001 per cent, so env_f107 sorted to the front of this list and eleven
  // rows in the solar subsystem opened with a sweep whose curve is a flat line.
  // A reader sees that and concludes the tool is broken.
  //
  // The engine is the only thing that knows, so it is asked. Each decision is
  // evaluated at both ends of its own declared range and the span of the answer
  // comes back; the list is re-ordered most-moving first and the ones that move
  // nothing say so in their own label rather than being hidden. Hiding them
  // would be worse: that a decision does not reach this row is a fact about the
  // design, and it is often the surprising one.
  (async () => {
    let lv;
    try {
      const lp = withOverrides(new URLSearchParams(
        { node: r.id, case: S.engineCase, mode: 'branch' }));
      lv = await (await fetch('/v1/levers?' + lp.toString())).json();
    } catch (e) { return; }
    if (!lv || !lv.ok || !lv.levers || !lv.levers.length) return;
    const keep = lv.levers.filter(l => S.byId.has(l.id));
    if (!keep.length) return;
    const pct = v => v >= 0.1 ? Math.round(v * 100) + '%' :
      v >= 0.001 ? (v * 100).toFixed(2) + '%' : v > 0 ? '<0.01%' : 'nothing';
    over.innerHTML = keep.map(l => {
      const tag = l.span === null ? ' — cannot be swept: ' + (l.why || 'refused') :
        ' — moves this answer by ' + pct(l.span);
      return '<option value="' + esc(l.id) + '">' + esc(l.id) + esc(tag) + '</option>';
    }).join('');
    // Lead with the decision that moves the answer most. If none of them move
    // it, the first is as good as any and the label already says so.
    over.value = keep[0].id;
    const d0 = S.byId.get(over.value);
    from.value = d0.lo; to.value = d0.hi;
    check();
    const dead = keep.filter(l => l.span === 0).length;
    if (dead) {
      $('.sw-note', host).textContent = dead + ' of ' + keep.length +
        ' decisions upstream of this row do not move its answer at all. They are ' +
        'still listed, because that is a fact about the design and not an omission.';
    }
  })();
  go2.onclick = async () => {
    const p = new URLSearchParams({
      node: r.id, over: over.value, from: from.value, to: to.value,
      points: '80', case: S.engineCase, mode: 'branch',
    });
    $('.sw-note', host).textContent = 'sweeping…';
    lastRes = await (await fetch('/v1/sweep?' + withOverrides(p).toString())).json();
    plot(host, lastRes);
  };
  // Changing the level redraws from the sweep already in hand. Re-running the
  // engine to move a horizontal line would be asking the kernel a question
  // whose answer cannot have changed.
  const mk = $('.sw-mark', host);
  if (mk) mk.onchange = async () => {
    markValue = null;
    if (mk.value) {
      const body = new URLSearchParams({ node: mk.value, mode: 'branch', case: S.engineCase });
      try {
        const rr = await (await fetch('/v1/run', {
          method: 'POST',
          headers: { 'content-type': 'application/x-www-form-urlencoded' },
          body: body.toString(),
        })).json();
        // The row's own answer, in SI, the same as everything else crossing the
        // boundary. A refusal leaves the level undrawn rather than drawn wrong.
        // `si` and not `value`: everything crossing the boundary is SI, and the
        // run reports the SI number beside the string it chose to show. A face
        // that read the shown string would be drawing a rounded level.
        const own = rr.ok && (rr.values || []).find(v => v.id === mk.value);
        markValue = own && isFinite(own.si) ? own.si : null;
        markWhy = own ? null : (rr.message || 'the level could not be run');
      } catch (e) { markWhy = 'the engine did not answer: ' + e; }
    }
    if (lastRes) plot(host, lastRes);
  };
}

let markValue = null, markWhy = null;

function plot(host, res) {
  const c = $('.sw-plot', host);
  if (!c) return;
  if (!res.ok) { $('.sw-note', host).textContent = res.message || 'the sweep was refused'; return; }
  c.hidden = false;
  const ctx = c.getContext('2d');
  const W = c.width, H = c.height, L = 78, B = 34, T = 14, R = 16;
  ctx.clearRect(0, 0, W, H);
  const xs = res.x.map(v => v / res.x_factor);
  const ys = res.y.map(v => v / res.y_factor);
  if (!xs.length) { $('.sw-note', host).textContent = 'every point was refused.'; return; }
  const x0 = Math.min.apply(null, xs), x1 = Math.max.apply(null, xs);
  let y0 = Math.min.apply(null, ys), y1 = Math.max.apply(null, ys);

  // The level to draw against, if one is chosen. Taken from the index rather
  // than from a second engine call: a declared value is a number somebody
  // wrote, and it cannot have changed since the sweep ran.
  const markSel = $('.sw-mark', host);
  const markRow = markSel && markSel.value ? S.byId.get(markSel.value) : null;
  const markY = markRow && markValue !== null ? markValue / res.y_factor : null;
  // Widen to include it, so a level the curve never reaches is still visible as
  // a level the curve never reaches. Clipping it off the top would draw a
  // picture in which the design always passes.
  if (markY !== null) { y0 = Math.min(y0, markY); y1 = Math.max(y1, markY); }
  if (y0 === y1) { y0 -= 1; y1 += 1; }
  const px = v => L + (v - x0) / (x1 - x0 || 1) * (W - L - R);
  const py = v => H - B - (v - y0) / (y1 - y0 || 1) * (H - B - T);

  ctx.strokeStyle = '#ece8de'; ctx.lineWidth = 1;
  ctx.fillStyle = '#8a8880'; ctx.font = '10px ui-monospace, monospace';
  for (let i = 0; i <= 4; i++) {
    const y = T + i * (H - B - T) / 4;
    ctx.beginPath(); ctx.moveTo(L, y); ctx.lineTo(W - R, y); ctx.stroke();
    ctx.fillText(fmt(y1 - i * (y1 - y0) / 4), 6, y + 3);
  }
  for (let i = 0; i <= 5; i++) {
    const x = L + i * (W - L - R) / 5;
    ctx.beginPath(); ctx.moveTo(x, T); ctx.lineTo(x, H - B); ctx.stroke();
    ctx.fillText(fmt(x0 + i * (x1 - x0) / 5), x - 16, H - B + 14);
  }
  ctx.strokeStyle = '#b5731a'; ctx.lineWidth = 1.6;
  ctx.beginPath();
  xs.forEach((x, i) => i ? ctx.lineTo(px(x), py(ys[i])) : ctx.moveTo(px(x), py(ys[i])));
  ctx.stroke();
  // The level, and where the curve crosses it. The crossing is the number a
  // design actually reads off this picture — the mission length at which the
  // committed level stops being met — so it is printed rather than left to be
  // eyeballed against a gridline.
  let crossing = null;
  if (markY !== null) {
    const y = py(markY);
    ctx.save();
    ctx.strokeStyle = '#8f43e0'; ctx.lineWidth = 1.2; ctx.setLineDash([5, 4]);
    ctx.beginPath(); ctx.moveTo(L, y); ctx.lineTo(W - R, y); ctx.stroke();
    ctx.restore();
    ctx.fillStyle = '#8f43e0'; ctx.font = '10px ui-monospace, monospace';
    ctx.fillText(markRow.id + ' = ' + fmt(markY), L + 4, y - 4);
    for (let i = 1; i < ys.length; i++) {
      const a = ys[i - 1], b = ys[i];
      if ((a - markY) * (b - markY) <= 0 && a !== b) {
        const t = (markY - a) / (b - a);
        crossing = xs[i - 1] + t * (xs[i] - xs[i - 1]);
        const cx = px(crossing);
        ctx.save();
        ctx.strokeStyle = '#8f43e0'; ctx.lineWidth = 1; ctx.setLineDash([2, 3]);
        ctx.beginPath(); ctx.moveTo(cx, y); ctx.lineTo(cx, H - B); ctx.stroke();
        ctx.restore();
        break;
      }
    }
  }
  ctx.fillStyle = '#1a1a1a'; ctx.font = '11px ui-monospace, monospace';
  ctx.fillText(res.y_id + '  [' + res.y_unit + ']', L, 11);
  ctx.fillText(res.x_id + '  [' + res.x_unit + ']', W - R - 220, H - 6);

  $('.sw-note', host).innerHTML = res.x.length + ' point' + (res.x.length === 1 ? '' : 's') + ' ran' +
    (res.refused.length
      ? ', <b>' + res.refused.length + ' refused</b> — ' + esc(res.refused[0].why) +
        '. Refusals are recorded, never dropped: a sweep in which some rows quietly used a substituted ' +
        'value is a sweep whose conclusion is unknown.'
      : ', none refused.');
  if (markRow && markY === null) {
    $('.sw-note', host).innerHTML +=
      ' <b>' + esc(markRow.id) + ' was not drawn</b> — ' + esc(markWhy || 'it returned no value') +
      '. A level that could not be run is left off rather than guessed at.';
  }
  if (markY !== null) {
    const el = $('.sw-note', host);
    el.innerHTML += crossing === null
      ? ' The curve does not cross <b>' + esc(markRow.id) + '</b> anywhere in this range' +
        (ys[ys.length - 1] > markY ? ' — it is above it throughout.' :
         ys[0] < markY ? ' — it is below it throughout.' : '.')
      : ' It crosses <b>' + esc(markRow.id) + '</b> at ' + fmt(crossing) + ' ' + esc(res.x_unit) +
        ', and is above it beyond that.';
  }
}
