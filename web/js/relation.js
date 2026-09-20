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
import { withOverrides } from './inputs.js';
import { drawChart, attachHover, tableFor, tableTsv, viewSpec, viewIsOn,
  watchScheme, INK } from './chart.js';

// THE SAME VIEW STATE THE PANELS HAVE. These are the generated per-node
// figures — one for every row with a relation, which is most of the tree — and
// until now they had the crosshair and the arrow keys and nothing else. A
// relation swept over a decade of its domain has the same problem `design` had:
// the part worth looking at is a sliver of the axis.
const RVIEWS = new WeakMap();
const rview = host => {
  let v = RVIEWS.get(host);
  if (!v) { v = { zoom: null, hidden: new Set(), pinned: null }; RVIEWS.set(host, v); }
  return v;
};

/**
 * The strip under a relation figure: what has been done to it, and the way out.
 *
 * Deliberately smaller than the panels': there is no key to click here — the
 * three series are one dataset shown three ways — and no view worth pinning.
 * What it carries is the window, the way back, and the numbers.
 */
function relStrip(host, shown, view, again) {
  const el = $('.rel-view', host);
  if (!el) return;
  const z = view.zoom || {};
  el.innerHTML = (z.x
    ? '<span class="sw-vs">showing ' + esc(fmt(z.x[0])) + ' to ' + esc(fmt(z.x[1])) +
      '</span><button class="ctl rel-unzoom" type="button">the whole domain</button>'
    : '') +
    '<button class="ctl rel-copy" type="button">copy as TSV</button>' +
    '<span class="sw-copied rel-copied"></span>' +
    '<span class="sw-hint muted">drag across the plot to zoom, double-click or Escape ' +
    'to undo</span>';
  const un = $('.rel-unzoom', el);
  if (un) un.onclick = () => { view.zoom = null; again(); };
  const cp = $('.rel-copy', el), said = $('.rel-copied', el);
  cp.onclick = async () => {
    let okay = true;
    try {
      await navigator.clipboard.writeText(tableTsv(shown));
    } catch (e) { okay = false; }
    if (said) {
      said.textContent = okay ? 'copied' : 'could not reach the clipboard';
      setTimeout(() => { said.textContent = ''; }, 2000);
    }
  };
}

// THESE FIGURES REDRAW ON A THEME CHANGE TOO. The panels have their own
// registry for it; these are the generated per-node relation and domain
// pictures, drawn once and then left, so without this they would keep the light
// palette on a dark page. Same shape as the panels': a redraw is dropped as
// soon as its host leaves the document, because a listener holding a detached
// node is a leak with a picture on it.
const LIVE = new Set();
const keepLive = (host, fn) => {
  fn._host = host;
  LIVE.add(fn);
};
watchScheme(() => {
  for (const fn of [...LIVE]) {
    if (fn._host && fn._host.isConnected) fn();
    else LIVE.delete(fn);
  }
});

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
    '<div class="sw-view rel-view"></div>' +
    // The relation as numbers. Same argument as the panels: the readout is
    // reached by pointing or by stepping, and a number somebody wants to quote
    // should be selectable rather than screenshotted.
    '<details class="sw-table"><summary>the numbers behind this picture</summary>' +
    '<div class="sw-table-body"></div></details>' +
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
        { name: '', kind: 'line', x: xs, y: ys, colour: INK.grid, width: 1.2 },
        { name: '', kind: 'line', x: xs.slice(0, n), y: ys.slice(0, n), width: 2.2 },
        { name: '', kind: 'dots', x: [xs[n - 1]], y: [ys[n - 1]], width: 4, alpha: 1 },
      ],
      marks: guardMarks(host, res),
    };
  };

  const redraw = () => {
    if (!res || !res.ok) return;
    const cv = $('.rel-plot', host);
    const view = rview(host);
    const built = spec();
    const shown = viewSpec(built, view);
    drawChart(cv, shown);
    attachHover(cv, {
      onBrush: win => {
        if (!win.x) return;
        const inside = built.series[0].x.filter(v => v >= win.x[0] && v <= win.x[1]);
        if (new Set(inside).size < 2) return;
        view.zoom = { ...(view.zoom || {}), x: win.x };
        redraw();
      },
      onReset: () => { if (viewIsOn(view)) { view.zoom = null; redraw(); } },
    });
    relStrip(host, shown, view, redraw);
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
    res = await (await fetch('/v1/sweep?' + withOverrides(p).toString())).json();
    if (!res.ok) {
      $('.rel-note', host).textContent = res.message || 'the engine refused the sweep';
      return;
    }
    scrub.value = 100;
    keepLive(host, redraw);
    redraw();
    // The WHOLE relation, once, and not the walk. The three series the chart
    // draws are one dataset shown three ways — faint behind, solid up to the
    // scrub, a dot at the head — so tabling all three would print the same
    // column three times. And it is built here rather than in redraw() because
    // the numbers do not change while the walk runs, only how much of them is
    // painted.
    $('.sw-table-body', host).innerHTML = tableFor({
      x: { label: res.x_id + '  [' + res.x_unit + ']' },
      y: { label: res.y_id + '  [' + res.y_unit + ']' },
      series: [{ name: '', kind: 'line',
        x: res.x.map(v => v / res.x_factor), y: res.y.map(v => v / res.y_factor) }],
    });
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
  if (near(lo)) out.push({ axis: 'y', at: lo, label: 'refuses below ' + fmt(lo), colour: INK.bound });
  if (near(hi)) out.push({ axis: 'y', at: hi, label: 'refuses above ' + fmt(hi), colour: INK.bound });
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
  // NO TABLE UNDER THIS ONE, deliberately. The picture is a bar of 101 identical
  // zeros with one mark on it, so a table of it would be 101 rows saying nothing
  // — and the three numbers it actually shows, the value and both bounds, are in
  // the note below in words. That is the equivalent a reader needs; a table here
  // would be the form of one without the content.
  host.innerHTML =
    '<canvas class="plot rel-plot" width="900" height="200"></canvas>' +
    '<div class="rel-note muted"></div>';
  const xs = [], ys = [];
  for (let i = 0; i <= 100; i++) { xs.push(lo + (hi - lo) * i / 100); ys.push(0); }
  const paint = () => drawChart($('.rel-plot', host), {
    x: { label: r.id + '  [' + (r.unit === '-' ? 'dimensionless' : r.unit) + ']', min: lo, max: hi },
    y: { label: 'the declared domain', min: -1, max: 1, ticks: 2, fmt: () => '' },
    series: [
      { name: '', kind: 'line', x: xs, y: ys, colour: INK.grid, width: 8 },
      { name: '', kind: 'dots', x: [v], y: [0], width: 7, alpha: 1 },
    ],
    marks: [
      { axis: 'x', at: lo, label: 'refuses below ' + fmt(lo), colour: INK.bound },
      { axis: 'x', at: hi, label: 'refuses above ' + fmt(hi), colour: INK.bound },
      { axis: 'x', at: v, label: r.symbol + ' = ' + fmt(v) },
    ],
  });
  keepLive(host, paint);
  paint();
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
