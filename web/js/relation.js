/*
  The relation, walked.

  A node's mathematics is stated three times on its page: as an expression, as
  pseudocode, and as the generated Rust. All three are symbols. This is the
  fourth statement and the only one somebody can watch: the input crosses its
  declared domain and the answer moves, with the guards drawn as the walls they
  are.

  It teaches nothing the other three do not say. What it does is let a reader
  who cannot yet read the expression see the shape of it — where the answer is
  flat, where it turns, where it runs into a bound — and then go back to the
  symbols knowing what they are supposed to describe.

  IT DRAWS THE ENGINE'S OWN ANSWER. The curve is a sweep from /v1/sweep, which
  runs the same compiled node the run panel runs. Nothing here evaluates the
  relation, so a picture that disagrees with the node is impossible rather than
  unlikely. Where the engine refuses a point, the line breaks and the refusal is
  counted in the caption — a walk that quietly stepped over the refusals would
  be teaching the wrong shape.
*/
'use strict';

import { $, $$, esc, fmt } from './dom.js';
import { S, reachFrom, isSeeded } from './state.js';
import { drawChart, attachHover, INK } from './chart.js';

export async function mountRelation(host) {
  const id = host.dataset.node;
  const r = S.byId.get(id);
  if (!r) return;

  // The decisions upstream, which is what a sweep can move. A computed node
  // with nothing declared above it has no axis to walk along, and saying so is
  // better than drawing a single point and calling it a relation.
  const ins = Array.from(reachFrom([r.i], S.producers))
    .map(i => S.rows[i])
    .filter(x => x.kind === 'declared' && x.hi > x.lo && !isSeeded(x))
    .sort((a, b) => a.id.localeCompare(b.id));

  if (!ins.length) {
    // A declared row has no relation to walk — it has a number somebody chose.
    // What there IS to see is where that number sits inside the domain the
    // sheet declared for it, which is the question a reviewer asks of a
    // declared value: how much room did the author leave on each side, and is
    // the number near a bound it was never meant to approach?
    if (r.kind === 'declared') {
      await declaredValue(host, r);
      return;
    }
    host.innerHTML = '<p class="empty">Nothing declared upstream of this node carries a range, ' +
      'so there is no decision to walk along. That is a fact about where this row sits in the ' +
      'tree, not about the relation.</p>';
    return;
  }

  host.innerHTML =
    '<div class="sweepctl">along <select class="rel-over">' +
    ins.map(x => '<option value="' + esc(x.id) + '">' + esc(x.id) + '</option>').join('') +
    '</select>' +
    ' <button class="ctl rel-play">▶ walk it</button>' +
    ' <input class="rel-scrub" type="range" min="0" max="100" value="100" step="1">' +
    ' <span class="muted rel-read"></span></div>' +
    '<canvas class="plot rel-plot" width="900" height="320"></canvas>' +
    '<div class="rel-note muted">asking the engine…</div>';

  const over = $('.rel-over', host);
  const play = $('.rel-play', host);
  const scrub = $('.rel-scrub', host);
  let res = null, timer = null;

  const spec = () => {
    const xs = res.x.map(v => v / res.x_factor);
    const ys = res.y.map(v => v / res.y_factor);
    const n = Math.max(1, Math.round((+scrub.value / 100) * xs.length));
    return {
      x: { label: res.x_id + '  [' + res.x_unit + ']' },
      y: { label: res.y_id + '  [' + res.y_unit + ']' },
      series: [
        // The whole relation, faint, so the walk is seen against where it is
        // going rather than only where it has been.
        { name: '', kind: 'line', x: xs, y: ys, colour: '#e3dccd', width: 1.2 },
        { name: '', kind: 'line', x: xs.slice(0, n), y: ys.slice(0, n), width: 2.2 },
        { name: '', kind: 'dots', x: [xs[n - 1]], y: [ys[n - 1]], width: 4, alpha: 1 },
      ],
      marks: guardMarks(host, res),
    };
  };

  const redraw = () => {
    if (!res || !res.ok) return;
    const cv = $('.rel-plot', host);
    drawChart(cv, spec());
    attachHover(cv);
    const xs = res.x.map(v => v / res.x_factor);
    const ys = res.y.map(v => v / res.y_factor);
    const n = Math.max(1, Math.round((+scrub.value / 100) * xs.length));
    $('.rel-read', host).textContent =
      res.x_id + ' = ' + fmt(xs[n - 1]) + ' ' + res.x_unit +
      '   →   ' + res.y_id + ' = ' + fmt(ys[n - 1]) + ' ' + res.y_unit;
  };

  const run = async () => {
    const d = S.byId.get(over.value);
    $('.rel-note', host).textContent = 'asking the engine…';
    const p = new URLSearchParams({
      node: id, over: d.id, from: d.lo, to: d.hi,
      points: '120', case: S.engineCase, mode: 'branch',
    });
    res = await (await fetch('/v1/sweep?' + p.toString())).json();
    if (!res.ok) {
      $('.rel-note', host).textContent = res.message || 'the engine refused the sweep';
      return;
    }
    scrub.value = 100;
    redraw();
    const lo = +host.dataset.lo, hi = +host.dataset.hi;
    const ys = res.y.map(v => v / res.y_factor);
    const span = Math.max(...ys) - Math.min(...ys);
    $('.rel-note', host).innerHTML =
      res.x.length + ' point' + (res.x.length === 1 ? '' : 's') + ' ran' +
      (res.refused.length
        ? ', <b>' + res.refused.length + ' refused</b> — ' + esc(res.refused[0].why) +
          '. The line breaks where the engine refused rather than stepping over it.'
        : ', none refused.') +
      ' Across the whole declared domain of <code>' + esc(d.id) + '</code> the answer moves by ' +
      fmt(span) + ' ' + esc(res.y_unit) +
      (span === 0
        ? ' — it does not move at all, which is a fact about the relation worth knowing.'
        : '.') +
      ' The dashed lines are this node’s own declared bounds, ' + fmt(lo) + ' and ' + fmt(hi) +
      ' in SI; where the curve is far from them the guards are not what shapes the answer.';
  };

  over.onchange = run;
  scrub.oninput = () => { stop(); redraw(); };

  function stop() {
    if (timer) { clearInterval(timer); timer = null; play.textContent = '▶ walk it'; }
  }
  play.onclick = () => {
    if (timer) { stop(); return; }
    if (!res || !res.ok) return;
    scrub.value = 0;
    play.textContent = '❚❚ stop';
    timer = setInterval(() => {
      const v = +scrub.value + 1.5;
      if (v >= 100) { scrub.value = 100; redraw(); stop(); return; }
      scrub.value = v;
      redraw();
    }, 28);
  };

  await run();
}

/**
 * The node's own declared bounds, in the units the sweep is drawn in.
 *
 * Drawn only when they are near the answer. A guard three orders of magnitude
 * away from every value is true and uninformative, and including it would
 * flatten the relation into a line along the bottom of the frame — which is a
 * picture of the axis, not of the relation.
 */
function guardMarks(host, res) {
  const lo = +host.dataset.lo, hi = +host.dataset.hi;
  const ys = res.y.map(v => v / res.y_factor);
  const y0 = Math.min(...ys), y1 = Math.max(...ys);
  const span = (y1 - y0) || Math.abs(y1) || 1;
  const near = v => v >= y0 - 2 * span && v <= y1 + 2 * span;
  const out = [];
  if (near(lo)) out.push({ axis: 'y', at: lo, label: 'refuses below ' + fmt(lo), colour: '#c2185b' });
  if (near(hi)) out.push({ axis: 'y', at: hi, label: 'refuses above ' + fmt(hi), colour: '#c2185b' });
  return out;
}

/**
 * A declared value against its own declared domain.
 *
 * The value is RUN rather than read. The daemon's index carries every row's
 * declared domain and not its answer — a declared number lives inside the
 * generated model — so the only way to draw the number the engine will actually
 * use is to ask the engine for it. That is also the safer way round: what is
 * drawn is what a consumer of this row receives, not what the sheet was
 * intended to say.
 *
 * The bar is the domain and the mark is the value, so how much headroom the
 * author left is visible rather than arithmetic.
 */
async function declaredValue(host, r) {
  host.innerHTML = '<p class="muted">asking the engine for the declared value\u2026</p>';
  let v = null;
  try {
    const body = new URLSearchParams({ node: r.id, mode: 'branch', case: S.engineCase });
    const rr = await (await fetch('/v1/run', {
      method: 'POST',
      headers: { 'content-type': 'application/x-www-form-urlencoded' },
      body: body.toString(),
    })).json();
    const own = rr.ok && (rr.values || []).find(x => x.id === r.id);
    if (own && isFinite(own.si)) v = own.si;
  } catch (e) { /* handled below */ }
  if (v === null) {
    host.innerHTML = '<p class="empty">This row declares a value and the engine did not return one, ' +
      'so there is nothing honest to draw here.</p>';
    return;
  }
  const lo = r.lo, hi = r.hi;
  const frac = (v - lo) / (hi - lo);
  host.innerHTML =
    '<canvas class="plot rel-plot" width="900" height="200"></canvas>' +
    '<div class="rel-note muted"></div>';
  const xs = [], ys = [];
  for (let i = 0; i <= 100; i++) { xs.push(lo + (hi - lo) * i / 100); ys.push(0); }
  drawChart($('.rel-plot', host), {
    x: { label: r.id + '  [' + (r.unit === '-' ? 'dimensionless' : r.unit) + ']', min: lo, max: hi },
    y: { label: 'the declared domain', min: -1, max: 1, ticks: 2, fmt: () => '' },
    series: [
      { name: '', kind: 'line', x: xs, y: ys, colour: '#e3dccd', width: 8 },
      { name: '', kind: 'dots', x: [v], y: [0], width: 7, alpha: 1 },
    ],
    marks: [
      { axis: 'x', at: lo, label: 'refuses below ' + fmt(lo), colour: '#c2185b' },
      { axis: 'x', at: hi, label: 'refuses above ' + fmt(hi), colour: '#c2185b' },
      { axis: 'x', at: v, label: r.symbol + ' = ' + fmt(v) },
    ],
  });
  const pc = (frac * 100).toFixed(1);
  host.querySelector('.rel-note').innerHTML =
    'A declared value has no relation to walk: it is a number a person chose, and this is where ' +
    'they put it. <b>' + esc(r.symbol) + ' = ' + fmt(v) + '</b> sits ' + pc + ' per cent of the way ' +
    'across its declared domain of ' + fmt(lo) + ' to ' + fmt(hi) + ', leaving ' +
    fmt(v - lo) + ' below and ' + fmt(hi - v) + ' above. ' +
    (frac < 0.05 || frac > 0.95
      ? 'It sits within five per cent of a bound. That is worth a reviewer\u2019s attention: either ' +
        'the bound is tighter than the author needed, or the value is closer to refusing than anyone ' +
        'intended.'
      : 'Neither bound is close, so the guards are catching a mistake rather than constraining the ' +
        'choice \u2014 which is what a guard is for.');
}
