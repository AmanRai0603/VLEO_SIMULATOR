/*
  The chart: axes, series and a legend, on a canvas.

  One drawing routine for eight panels rather than eight. A panel decides what
  the series ARE — that is the argument every tab is making — and hands them
  here; nothing in this module knows what F10.7 is.

  It draws only what it was given. There is no smoothing, no gap filling and no
  extrapolation: a null in a series is a break in the line, because a line drawn
  straight across nine missing months is a claim nobody made.
*/
'use strict';

// THE SERIES PALETTE, MEASURED RATHER THAN CHOSEN.
//
// The previous six were '#b5731a', '#2a6f97', '#7a9e3f', '#8a3ffc', '#c1440e',
// '#4a4a4a', and run through a colourblind-safety validator they FAILED two of
// five checks: #4a4a4a sits outside the lightness band at 0.409 and has zero
// chroma, so it reads as grey rather than as a series, and #2a6f97 is under the
// chroma floor at 0.093. "These look different enough" is not a check, and this
// is what running one says.
//
// These six pass all five in light mode AND in dark, each against its own
// surface. Slot 1 is unchanged so the orange that means "the record" everywhere
// in this tool keeps meaning it. Hues are assigned in this fixed order and never
// cycled: a seventh series folds into a small multiple rather than borrowing a
// hue that is already taken.
//
// The worst adjacent CVD separation is 8.2 against a floor of 8. That is a pass
// and not a comfortable one, which is why direct labelling and a legend are not
// decoration here — they are what makes the palette legal.
export const INK = {
  grid: '#ece8de',
  axis: '#8a8880',
  text: '#1a1a1a',
  muted: '#8a8880',
  surface: '#fcfcfb',
  series: ['#b5731a', '#2f6fa8', '#2e7d55', '#8f43e0', '#c2185b', '#00918f'],
  mark: '#8f43e0',
};

/**
 * Tick VALUES on round numbers, rather than whatever lands on an even pixel.
 *
 * The axes here used to be laid out by dividing the plot's HEIGHT into n equal
 * parts and labelling whichever data value fell at each one, with `nice` only
 * formatting what it was handed. So a panel would be labelled 58.8, 90.3,
 * 121.9, 153.4, 184.9, 216.5 — six numbers nobody can use. Reading a value off
 * such an axis means interpolating by eye between two arbitrary ones, every
 * time, on every panel: this single defect disfigured all ten.
 *
 * The step comes from the 1 / 2 / 2.5 / 5 ladder, which is the set of
 * multipliers whose decimal multiples people read without thinking. `target` is
 * a hint rather than a count — the number of ticks that actually fit a round
 * step is whatever it is, and forcing exactly six is what produced the problem.
 */
function niceTicks(lo, hi, target) {
  if (!(isFinite(lo) && isFinite(hi)) || hi <= lo) return [lo];
  const raw = (hi - lo) / Math.max(1, target);
  const mag = Math.pow(10, Math.floor(Math.log10(raw)));
  const norm = raw / mag;
  // The NEAREST rung, in log space, not the next one up. Rounding up always
  // overshoots: a range of 160 asking for five ticks gives raw 32, and taking
  // the next rung above 3.2 is 5, so a step of 50 and three ticks for a range
  // that wanted six. The nearest rung is 2.5, a step of 25, and six ticks.
  // Log space because the rungs are multiplicative — 2.5 is as far from 2 as 5
  // is from 2.5, which is not true of their differences.
  let step = 10 * mag, best = Infinity;
  for (const r of [1, 2, 2.5, 5, 10]) {
    const d = Math.abs(Math.log(norm / r));
    if (d < best) { best = d; step = r * mag; }
  }
  const out = [];
  // The half-step of slack absorbs binary floating point at the ends: a tick
  // that lands on the limit to fifteen digits should be drawn, and 0.1 + 0.2
  // says it might not.
  for (let i = Math.ceil(lo / step - 1e-9); i * step <= hi + step * 1e-9; i++) {
    out.push(+(i * step).toPrecision(12));
  }
  return out.length ? out : [lo, hi];
}

/** Powers of ten for a log axis — the only nice step a log scale has. */
function niceLogTicks(lo, hi) {
  const out = [];
  for (let e = Math.ceil(Math.log10(lo) - 1e-9); Math.pow(10, e) <= hi * (1 + 1e-9); e++) {
    out.push(Math.pow(10, e));
  }
  return out.length ? out : [lo, hi];
}

// Now that the ticks ARE round, the formatter's job is to stop adding decimals
// they do not have: 150 rather than 150.0, 0.2 rather than 0.200. It still has
// to cope with a value handed to it by a caller's own `fmt`, so it keeps a
// sensible precision and then drops what the trailing zeros claim.
const nice = v => {
  const a = Math.abs(v);
  if (a === 0) return '0';
  if (a >= 1e6 || a < 1e-4) return v.toExponential(1).replace('e+', 'e');
  const dp = a >= 100 ? 1 : a >= 1 ? 2 : 4;
  return v.toFixed(dp).replace(/\.?0+$/, '');
};

/**
 * @param spec {
 *   x: {label, min, max, ticks?, fmt?},
 *
 *   // ONE PANE, the ordinary case:
 *   y: {label, min, max, ticks?, fmt?, log?},
 *   series: [{ name, kind: 'line'|'dots'|'bars'|'step', x:[], y:[], colour?, width?, dash?,
 *              n?: [] — the sample behind each point; where given, the line fades
 *              as the count falls, so a thin far end does not read as a firm one,
 *              aside?: true — FURNITURE, not data. A significance band is a
 *              reference the curve is read against, the same job a mark does, and
 *              it happens to vary with x rather than sit at one value. Drawn, and
 *              named in the legend so the dashes mean something; kept out of the
 *              end labels, the readout and the table, which are for the record }],
 *   marks: [{ axis:'y'|'x', at, label, colour? }],
 *
 *   // OR SEVERAL, STACKED, SHARING ONE X AXIS:
 *   panes: [{ y, series, marks }, …],
 *
 *   note: string
 * }
 *
 * STACKED PANES ARE THE ANSWER TO THE DUAL-AXIS TEMPTATION, and that is the whole
 * reason they exist. `forecast` scores the published outlook three ways — skill, a
 * dimensionless ratio; bias, in sfu; RMS error, in sfu but on a different scale —
 * and all three are read against the same lead. Putting two of them on one frame
 * needs two y scales, which lets the author choose where the curves cross and is
 * the most reliable way to make a chart say something the data did not. Offering
 * them as three settings of a control instead is honest and costs the comparison:
 * the reader has to hold one picture in their head while looking at the next.
 *
 * Three frames in a column, one x axis at the bottom, each with its own y scale
 * and its own label, is the standard answer to exactly this and it keeps the
 * comparison the control was destroying.
 *
 * A pane carries its own y, its own series, its own marks and its own legend. The
 * x extent is computed across every pane, so the frames are registered and a
 * vertical read across them lands on the same lead in all three.
 */
export function drawChart(canvas, spec) {
  const ctx = canvas.getContext('2d');
  const W = canvas.width, H = canvas.height;
  const L = 74, B = 42;

  // One shape for both cases: everything below works over `panes`, and a spec
  // that names y/series/marks directly is one pane. Written this way rather than
  // as two branches because two branches is how a chart layer comes to have a
  // feature that works on one path and silently does nothing on the other.
  const panes = (spec.panes && spec.panes.length
    ? spec.panes
    : [{ y: spec.y, series: spec.series, marks: spec.marks }]).map(p => ({
      y: p.y || {}, series: p.series || [], marks: p.marks || [],
    }));

  // A GUTTER FOR THE END LABELS, so a direct label sits BESIDE its line rather
  // than on top of it and off the edge. The texts are known before the scale is:
  // each is the last finite value of a named series, which is data and not
  // geometry. Measured first, then the right margin is whatever holds the widest
  // of them — the same move the legend band makes vertically.
  ctx.font = '10px ui-monospace, monospace';
  const endTexts = [];
  for (const pn of panes) {
    if (pn.series.filter(q => q.name).length <= 1) continue;
    const f = pn.y.fmt || nice;
    for (const q of pn.series) {
      if (!q.name || q.aside || q.kind === 'bars' || q.kind === 'dots') continue;
      for (let k = q.y.length - 1; k >= 0; k--) {
        const v = q.y[k];
        if (v !== null && isFinite(v)) { endTexts.push(f(v)); break; }
      }
    }
  }
  const endW = endTexts.reduce((m, t) => Math.max(m, ctx.measureText(t).width), 0);
  const R = 18 + (endW ? endW + 8 : 0);

  // THE LEGEND GETS ITS OWN BAND, ABOVE ITS PANE.
  //
  // It used to be drawn inside the frame at the top left, over whatever the
  // data was doing there — on Repeatability that is exactly where cycles 23 and
  // 25 run, so four entries sat on top of the thing they identify. A legend
  // that hides data is a worse legend than none.
  //
  // Laid out in rows across the width rather than one per line, because the
  // vertical space it takes is stolen from the picture. A single series gets no
  // legend at all: there is one colour, and the axis title already names it.
  //
  // PER PANE, not shared, and that is not a detail. On `forecast` the skill pane
  // carries two baselines and the other two carry one line each; a single legend
  // above the stack would be read as naming series in all three frames, and the
  // one line in the bias pane would appear to be whichever entry shares its hue.
  const LEG_H = 13;
  const legs = panes.map(pn => {
    const named = pn.series.filter(s => s.name);
    return { named, rows: named.length > 1 ? _legendRows(ctx, named, W - L - R) : [] };
  });
  // Each pane's own header: the y label, plus however many legend rows it needs.
  const bands = legs.map(g => 18 + g.rows.length * LEG_H);

  ctx.clearRect(0, 0, W, H);
  ctx.fillStyle = '#fff';
  ctx.fillRect(0, 0, W, H);

  // THE X EXTENT IS COMPUTED ACROSS EVERY PANE, which is what registers them: a
  // vertical read down a stack has to land on the same lead in all three, and a
  // pane scaled to its own x would look right and be a different picture.
  const allS = panes.flatMap(p => p.series).filter(s => s.x.length);
  const xsAll = allS.flatMap(s => s.x.filter(v => v !== null && isFinite(v)));
  let x0 = xsAll.length ? Math.min(...xsAll) : 0;
  let x1 = xsAll.length ? Math.max(...xsAll) : 1;
  for (const pn of panes) {
    for (const m of pn.marks) {
      if (m.axis === 'x') { x0 = Math.min(x0, m.at); x1 = Math.max(x1, m.at); }
    }
  }
  const given = v => v !== undefined && v !== null && isFinite(v);
  if (given(spec.x.min)) x0 = spec.x.min;
  if (given(spec.x.max)) x1 = spec.x.max;
  if (x0 === x1) { x0 -= 1; x1 += 1; }
  const px = v => L + (v - x0) / (x1 - x0) * (W - L - R);
  const fx = spec.x.fmt || nice;
  const xt = niceTicks(x0, x1, spec.x.ticks || 6);

  // The stack: every pane the same height, each under its own header. With one
  // pane this is the single frame from T to H - B that it has always been, to
  // the pixel — which is the check that this refactor changed no picture.
  const GAP = panes.length > 1 ? 6 : 0;
  const bodyH = (H - B - bands.reduce((a, b) => a + b, 0) - (panes.length - 1) * GAP)
    / panes.length;
  const geom = [];
  let cursor = 0;
  for (let i = 0; i < panes.length; i++) {
    const top = cursor + bands[i];
    const bot = top + bodyH;
    geom.push({ head: cursor, top, bot });
    cursor = bot + GAP;
  }

  const drawn = panes.map((pn, pi) => _pane(ctx, {
    pn, W, H, L, R, B, px, x0, x1, xt, top: geom[pi].top, bot: geom[pi].bot,
    head: geom[pi].head, leg: legs[pi], LEG_H, given,
  }));

  // THE X AXIS ONCE, AT THE BOTTOM, because it is the axis the stack shares. A
  // tick strip under every frame would say the three are three charts.
  ctx.fillStyle = INK.axis; ctx.font = '10px ui-monospace, monospace';
  ctx.textAlign = 'center';
  for (const at of xt) {
    const x = Math.round(px(at)) + 0.5;
    if (x < L - 0.5 || x > W - R + 0.5) continue;
    ctx.fillText(fx(at), x, H - B + 15);
  }
  ctx.textAlign = 'right';
  ctx.fillStyle = INK.text; ctx.font = '11px ui-monospace, monospace';
  ctx.fillText(spec.x.label, W - R, H - 6);
  ctx.textAlign = 'left';

  // Everything a hover needs to answer "what is the value here", kept on the
  // canvas so the readout cannot drift from the picture it is drawn over.
  canvas._chart = {
    spec, x0, x1, L, R, B, px, fx,
    T: geom[0].top, bot: geom[geom.length - 1].bot,
    panes: drawn,
  };

  // WHAT THE FIGURE IS, before anybody points at it or steps into it. The canvas
  // carries role="img", and an img with no label is announced as "image" and
  // nothing else — so every panel was, to a reader who cannot see it, an
  // unnamed picture until the first arrow key. Set here rather than in the
  // hover wiring because it is a property of what was drawn; stepping through
  // the points then replaces it with the value under the cursor and blurring
  // restores it.
  const named = panes.flatMap((pn, i) => legs[i].named);
  const what = panes.map(pn => _plain(pn.y.label)).join(', then ')
    + ' by ' + _plain(spec.x.label)
    + (panes.length > 1 ? ', ' + panes.length + ' frames on one axis' : '')
    + (named.length > 1 ? ', ' + named.length + ' series: '
        + named.map(s => s.name).join(', ') : '')
    + (spec.note ? '. ' + spec.note : '');
  canvas._said = what;
  canvas.setAttribute('aria-label', what);
}

/**
 * One frame: its own y scale, its own grid, its own series, marks and legend.
 *
 * Returns what a readout needs to answer inside this frame — the y mapping is
 * the pane's, and a crosshair that used the stack's would report every value in
 * the wrong frame.
 */
function _pane(ctx, o) {
  const { pn, W, H, L, R, B, px, x0, x1, xt, top, bot, head, leg, LEG_H, given } = o;
  const T = top, BOT = bot;

  // THE EXTENT IS ALWAYS COMPUTED, AND A DECLARED BOUND THEN OVERRIDES ITS OWN
  // END. This used to be two either-or branches keyed on the MINIMUM: a spec
  // that supplied `min` and left `max` open skipped the branch entirely and left
  // the far end undefined, so every tick came out NaN and the chart drew nothing
  // but gridlines. The Segmentation histogram declares `min: 0` on both axes and
  // had been rendering blank.
  const all = pn.series.filter(s => s.x.length);
  const ysAll = all.flatMap(s => s.y.filter(v => v !== null && isFinite(v)));
  let y0 = ysAll.length ? Math.min(...ysAll) : 0;
  let y1 = ysAll.length ? Math.max(...ysAll) : 1;
  // A mark outside the data is still a fact about the data, so the frame
  // grows to hold it. Clipping a bound off the top draws a picture in which
  // the bound is always met.
  for (const m of pn.marks) {
    if (m.axis !== 'x') { y0 = Math.min(y0, m.at); y1 = Math.max(y1, m.at); }
  }
  if (!given(pn.y.min) || !given(pn.y.max)) {
    const pad = (y1 - y0) * 0.06 || 1;
    if (!given(pn.y.min)) y0 -= pad;
    if (!given(pn.y.max)) y1 += pad;
  }
  if (given(pn.y.min)) y0 = pn.y.min;
  if (given(pn.y.max)) y1 = pn.y.max;
  if (y0 === y1) { y0 -= 1; y1 += 1; }

  // A log y axis where the quantity is geometric. ap runs 0 to 400 across the
  // Kp scale in roughly equal ratios, so a linear axis spends nine tenths of
  // its height on the top two points and buries everything a design reads.
  // Zero and negatives have no place on it and are dropped to null rather than
  // clamped to the floor, which would draw them as the smallest real value.
  const lg = !!pn.y.log;
  const tl = v => (v === null || !isFinite(v) || v <= 0 ? null : Math.log10(v));
  if (lg) { y0 = Math.max(y0, 0.5); }
  const ly0 = lg ? Math.log10(y0) : y0, ly1 = lg ? Math.log10(y1) : y1;
  const py = v => {
    const t = lg ? tl(v) : v;
    if (t === null) return BOT;
    return BOT - (t - ly0) / (ly1 - ly0) * (BOT - T);
  };
  const fy = pn.y.fmt || nice;

  ctx.strokeStyle = INK.grid; ctx.lineWidth = 1;
  ctx.fillStyle = INK.axis; ctx.font = '10px ui-monospace, monospace';
  // Round tick VALUES, placed where the data puts them — not even pixels
  // labelled with whatever fell there. See niceTicks.
  const yt = lg ? niceLogTicks(y0, y1) : niceTicks(y0, y1, pn.y.ticks || 5);
  for (const at of yt) {
    const y = Math.round(py(at)) + 0.5;
    if (y < T - 0.5 || y > BOT + 0.5) continue;
    // Zero is the reference for anything that can be negative, and an axis
    // that labels -0.197 and 0.0426 instead of 0 hides it. Drawn one step
    // stronger where it is in range — an autocorrelation is read against it.
    const isZero = Math.abs(at) < 1e-12;
    ctx.strokeStyle = isZero ? INK.axis : INK.grid;
    ctx.beginPath(); ctx.moveTo(L, y); ctx.lineTo(W - R, y); ctx.stroke();
    ctx.strokeStyle = INK.grid;
    ctx.textAlign = 'right';
    ctx.fillText(fy(at), L - 6, y + 3);
  }
  ctx.textAlign = 'center';
  for (const at of xt) {
    const x = Math.round(px(at)) + 0.5;
    if (x < L - 0.5 || x > W - R + 0.5) continue;
    const isZero = Math.abs(at) < 1e-12;
    ctx.strokeStyle = isZero ? INK.axis : INK.grid;
    ctx.beginPath(); ctx.moveTo(x, T); ctx.lineTo(x, BOT); ctx.stroke();
    ctx.strokeStyle = INK.grid;
  }
  ctx.textAlign = 'left';

  // MARK SPECS. Lines 2px with round joins and caps, so a curve does not go
  // spiky at a corner and a one-point run still draws. Dots at radius 4 — an
  // 8px mark is the smallest a person can point at — each carrying a 2px ring
  // in the SURFACE colour, which is what lets two dots overlap and stay two
  // dots. The ring is the separator; a stroke around a mark would be ink that
  // is not data.
  const ends = [];
  pn.series.forEach((s, i) => {
    const col = s.colour || INK.series[i % INK.series.length];
    ctx.save();
    ctx.strokeStyle = col; ctx.fillStyle = col;
    ctx.lineWidth = s.width || 2;
    ctx.lineJoin = 'round'; ctx.lineCap = 'round';
    if (s.dash) ctx.setLineDash(s.dash);
    if (s.kind === 'dots') {
      const r = s.width || 4;
      for (let k = 0; k < s.x.length; k++) {
        if (s.y[k] === null) continue;
        ctx.globalAlpha = s.alpha === undefined ? 0.5 : s.alpha;
        const X = px(s.x[k]), Y = py(s.y[k]);
        // A scatter of thousands is a field, not a set of markers: the ring
        // would cost more than it buys and the alpha is doing the work. Rung it
        // only where the dots are countable.
        if (s.x.length <= 120) {
          ctx.save();
          ctx.globalAlpha = 1;
          ctx.strokeStyle = INK.surface; ctx.lineWidth = 2;
          ctx.beginPath(); ctx.arc(X, Y, r, 0, 6.284); ctx.stroke();
          ctx.restore();
        }
        ctx.beginPath(); ctx.arc(X, Y, r, 0, 6.284); ctx.fill();
      }
    } else if (s.kind === 'bars') {
      // A 2px gap in the surface colour is what separates touching bars —
      // white doing the separating, rather than a border drawn round each one.
      const w = Math.max(1, (W - L - R) / s.x.length - 2);
      ctx.globalAlpha = s.alpha === undefined ? 0.75 : s.alpha;
      for (let k = 0; k < s.x.length; k++) {
        if (s.y[k] === null) continue;
        const h = py(y0 < 0 ? 0 : y0) - py(s.y[k]);
        ctx.fillRect(px(s.x[k]) - w / 2, py(s.y[k]), w, Math.max(1, h));
      }
    } else {
      // WHERE THE SAMPLE THINS, SO DOES THE LINE.
      //
      // A curve of constant weight from end to end says it is equally well
      // known at both. predict's n falls away with lead — the far end of a
      // fifteen-year growth curve rests on a fraction of the pairs the near end
      // has — and the picture said nothing about it. A figure that looks
      // equally confident everywhere is the one way it can lie without
      // containing a wrong number.
      //
      // `n` is per point and optional. Where it is given, each segment is drawn
      // at an opacity following its own count against the best count in the
      // series, floored at a fifth so a thin stretch fades rather than vanishes.
      const N = s.n && s.n.length === s.x.length ? s.n : null;
      const nMax = N ? Math.max(...N.filter(v => isFinite(v))) : 0;
      const wt = k => (!N || !nMax ? 1 : Math.max(0.2, Math.min(1, (N[k] || 0) / nMax)));
      const segment = (a, b) => {
        ctx.beginPath();
        ctx.moveTo(px(s.x[a]), py(s.y[a]));
        if (s.kind === 'step') { ctx.lineTo(px(s.x[b]), py(s.y[a])); }
        ctx.lineTo(px(s.x[b]), py(s.y[b]));
        ctx.stroke();
      };
      const live = k => { const v = s.y[k]; return v !== null && isFinite(v) && !(lg && v <= 0); };
      if (N) {
        for (let k = 1; k < s.x.length; k++) {
          if (!live(k) || !live(k - 1)) continue;
          ctx.globalAlpha = Math.min(wt(k), wt(k - 1));
          segment(k - 1, k);
        }
        ctx.globalAlpha = 1;
      } else {
        // A null breaks the line rather than being skipped over.
        let open = false;
        ctx.beginPath();
        for (let k = 0; k < s.x.length; k++) {
          if (!live(k)) { open = false; continue; }
          const X = px(s.x[k]), Y = py(s.y[k]);
          if (!open) { ctx.moveTo(X, Y); open = true; }
          else if (s.kind === 'step') { ctx.lineTo(X, py(s.y[k - 1] === null ? s.y[k] : s.y[k - 1])); ctx.lineTo(X, Y); }
          else ctx.lineTo(X, Y);
        }
        ctx.stroke();
      }
      // DIRECT LABEL AT THE END OF THE LINE.
      //
      // Not decoration: the palette's worst adjacent CVD separation is 8.2
      // against a floor of 8, and a separation in that band is legal only with a
      // second encoding. The legend is one; a label riding the line itself is
      // the one that works when a reader is looking at the data rather than at
      // the key.
      //
      // Selective by construction — the END of each line and nowhere else. A
      // number beside every point is chaos and goes unread, and the axis, the
      // legend and the hover carry the rest.
      if (s.name && !s.aside && pn.series.filter(q => q.name).length > 1) {
        let lastK = -1;
        for (let k = s.x.length - 1; k >= 0; k--) {
          const v = s.y[k];
          if (v !== null && isFinite(v) && !(lg && v <= 0)) { lastK = k; break; }
        }
        if (lastK >= 0) {
          const X = px(s.x[lastK]), Y = py(s.y[lastK]);
          if (X >= L && X <= W - R && Y >= T && Y <= BOT) {
            ends.push({ x: X, y: Y, col, text: fy(s.y[lastK]) });
          }
        }
      }
    }
    ctx.restore();
  });

  // The end labels last, over every line, and nudged apart where two series
  // finish at the same height — two numbers printed on top of each other are
  // worth less than one.
  ctx.font = '10px ui-monospace, monospace';
  ctx.textAlign = 'left';
  ends.sort((a, b) => a.y - b.y);
  let prevY = -1e9;
  for (const e of ends) {
    const y = Math.max(e.y + 3, prevY + 11);
    prevY = y;
    // In the gutter, always — left-aligned just outside the plot, so the
    // labels form a column a reader can scan instead of four numbers scattered
    // wherever their lines happened to finish.
    ctx.fillStyle = e.col;
    ctx.fillText(e.text, W - R + 6, y);
  }

  // Marks, and their labels stacked so two nearby marks do not print on top of
  // each other. Two boundaries four Ap apart on a 275-wide axis are four pixels
  // apart, and their labels used to overlap into something unreadable — which is
  // the case where a reader most needs to know which boundary is which.
  ctx.font = '10px ui-monospace, monospace';
  const taken = { x: [], y: [] };
  /** The first row in which this label does not overlap one already drawn. */
  const slotFor = (at, w) => {
    for (let row = 0; row < 8; row++) {
      if (!taken.x.some(t => t.row === row && at < t.end && at + w > t.start)) {
        taken.x.push({ row, start: at, end: at + w });
        return row;
      }
    }
    return 0;
  };
  for (const m of pn.marks) {
    const col = m.colour || INK.mark;
    ctx.save();
    ctx.strokeStyle = col; ctx.fillStyle = col; ctx.lineWidth = 1.2;
    ctx.setLineDash([5, 4]);
    ctx.beginPath();
    if (m.axis === 'x') { ctx.moveTo(px(m.at), T); ctx.lineTo(px(m.at), BOT); }
    else { ctx.moveTo(L, py(m.at)); ctx.lineTo(W - R, py(m.at)); }
    ctx.stroke();
    ctx.restore();
    ctx.fillStyle = col;
    const w = ctx.measureText(m.label).width + 8;
    if (m.axis === 'x') {
      const x = Math.min(W - R - w, px(m.at) + 4);
      ctx.fillText(m.label, x, T + 11 + 12 * slotFor(x, w));
    } else {
      // A y label is nudged DOWN rather than sideways: sideways would put it
      // over the data it is annotating.
      let y = Math.max(T + 10, py(m.at) - 4);
      while (taken.y.some(t => Math.abs(t.at - y) < 11) && y < BOT - 2) y += 11;
      taken.y.push({ at: y });
      ctx.fillText(m.label, L + 4, y);
    }
  }

  // THE PANE'S OWN HEADER: its y label, and its legend under it. Positioned from
  // the pane's header top rather than from the canvas, which is what lets three
  // frames each name their own quantity.
  ctx.fillStyle = INK.text; ctx.font = '11px ui-monospace, monospace';
  ctx.fillText(pn.y.label, L, head + 12);

  if (leg.rows.length) {
    ctx.font = '10px ui-monospace, monospace';
    ctx.textAlign = 'left';
    leg.rows.forEach((row, r) => {
      let lx = L;
      const ly = head + 12 + (r + 1) * LEG_H;
      for (const s of row) {
        const col = s.colour || INK.series[pn.series.indexOf(s) % INK.series.length];
        ctx.fillStyle = col;
        ctx.fillRect(lx, ly - 5, 14, 3);
        ctx.fillStyle = INK.text;
        ctx.fillText(s.name, lx + 19, ly);
        lx += 19 + ctx.measureText(s.name).width + 18;
      }
    });
  }

  return { y: pn.y, series: pn.series, py, fy, y0, y1, top: T, bot: BOT, log: lg };
}

/** Pack the legend into rows that fit the plot width. */
function _legendRows(ctx, named, width) {
  ctx.font = '10px ui-monospace, monospace';
  const rows = [[]];
  let w = 0;
  for (const s of named) {
    const itemW = 19 + ctx.measureText(s.name).width + 18;
    if (w + itemW > width && rows[rows.length - 1].length) { rows.push([]); w = 0; }
    rows[rows.length - 1].push(s);
    w += itemW;
  }
  return rows[0].length ? rows : [];
}

/**
 * A readout under the cursor.
 *
 * Charts are read by pointing at them. Without this a viewer reads a value by
 * eye against a gridline, which is the same act as not reading it.
 *
 * It redraws the whole chart and then overlays, rather than keeping a second
 * buffer: one drawing path means the crosshair cannot end up describing a
 * picture that has since changed.
 */
// An axis title without its unit, for a line of running text. The unit is on
// the axis where a reader can see it once, rather than on every value.
const _plain = t => String(t || '').split('  [')[0].trim();

/**
 * Is there a mark under xv, on the series' own terms?
 *
 * In ARRAY ORDER rather than in x order, because that is the order the segments
 * are drawn in — a series whose x does not ascend is drawn as it was given, and a
 * readout that reasoned about a sorted copy would describe a different picture.
 *
 * A joined series needs a segment whose two ends are both present and which
 * straddles xv. A series of separate marks — bars, dots — needs xv inside the
 * mark's own slot, which is half the distance to its nearest neighbour.
 */
function _drawnAt(s, bi, xv) {
  const has = k => k >= 0 && k < s.x.length && s.y[k] !== null && isFinite(s.y[k]);
  if (s.kind === 'bars' || s.kind === 'dots') {
    let sp = Infinity;
    for (const k of [bi - 1, bi + 1]) {
      if (k >= 0 && k < s.x.length) sp = Math.min(sp, Math.abs(s.x[k] - s.x[bi]));
    }
    return !isFinite(sp) || Math.abs(s.x[bi] - xv) <= sp / 2;
  }
  for (const k of [bi - 1, bi]) {
    if (!has(k) || !has(k + 1)) continue;
    const a = Math.min(s.x[k], s.x[k + 1]), b = Math.max(s.x[k], s.x[k + 1]);
    if (xv >= a && xv <= b) return true;
  }
  return false;
}

/**
 * The readout at one position on the x axis, drawn over a fresh chart.
 *
 * One path, used by the pointer AND by the keyboard. Written as two was how the
 * keyboard came to have no readout at all: a chart is read by pointing at it,
 * and a reader who cannot point had the axis and nothing else.
 */
function drawReadout(canvas, xv, focused) {
  const c = canvas._chart;
  if (!c) return null;
  drawChart(canvas, c.spec);
  const ctx = canvas.getContext('2d');
  const mx = c.px(xv);
  ctx.save();
  ctx.strokeStyle = INK.axis; ctx.lineWidth = 1; ctx.setLineDash([2, 3]);
  // THROUGH THE WHOLE STACK, gaps included. On a stacked spec the crosshair is
  // the thing that makes the three frames one picture: it says "this lead" once
  // and every frame answers for it. Stopping it at each frame's edge would leave
  // three unrelated crosshairs that happen to line up.
  ctx.beginPath(); ctx.moveTo(mx, c.T); ctx.lineTo(mx, c.bot); ctx.stroke();
  ctx.restore();
  // The nearest actual point of each series, never an interpolation: a readout
  // that invents a value between two measurements is reporting the chart's
  // arithmetic rather than the record.
  const hits = [];
  for (const pane of c.panes) pane.series.forEach((s, i) => {
    if (!s.x.length || s.aside) return;
    // A SERIES ANSWERS ONLY WHERE SOMETHING IS DRAWN.
    //
    // Taking the nearest point unconditionally reported whatever the series had,
    // however far away it was, and it was wrong in two ways that both showed up
    // on Repeatability. The running cycle stops at phase 0.532, and the readout
    // answered "cycle 25: 153.4" at phase 0.975 — a value from less than half the
    // phase the crosshair stood on. And at phase 0.775 cycle 24 is missing, so it
    // reported the neighbour across the gap: a line is never drawn across a null
    // here, on the grounds that it would be a claim nobody made, and a readout
    // across one is the same claim in text.
    //
    // Both are found by the same test — is there a drawn mark at xv — and that is
    // what the table under the panel shows, so the two now agree by construction
    // rather than by coincidence. The disagreement was measured, not guessed: the
    // table's cell for cycle 24 at 0.775 was empty while the tooltip held 71.61.
    let bi = -1, bd = Infinity;
    for (let k = 0; k < s.x.length; k++) {
      if (s.y[k] === null || !isFinite(s.y[k])) continue;
      const d = Math.abs(s.x[k] - xv);
      if (d < bd) { bd = d; bi = k; }
    }
    if (bi < 0) return;
    if (bd > 0 && !_drawnAt(s, bi, xv)) return;
    const col = s.colour || INK.series[i % INK.series.length];
    // A single series carries no name, and the readout was therefore a bare
    // number: "F10.7 64. 5". The y axis says what 5 is, which a sighted reader
    // has and a reader hearing the aria-label does not, so the axis title
    // stands in — one path, so the tooltip and the announcement say the same.
    hits.push({ col, name: s.name || _plain(pane.y.label), x: s.x[bi], y: s.y[bi],
      f: pane.fy });
    // A 2px ring in the surface colour, so the highlighted point stays legible
    // where it sits on its own line.
    ctx.save();
    ctx.strokeStyle = INK.surface; ctx.lineWidth = 2;
    ctx.beginPath(); ctx.arc(c.px(s.x[bi]), pane.py(s.y[bi]), 4, 0, 6.284); ctx.stroke();
    ctx.restore();
    ctx.fillStyle = col;
    ctx.beginPath(); ctx.arc(c.px(s.x[bi]), pane.py(s.y[bi]), 4, 0, 6.284); ctx.fill();
  });
  if (!hits.length) return null;
  // Each value formatted by ITS OWN pane. Three frames in different units is the
  // reason the stack exists, and one formatter for all of them would print sfu
  // to the precision a dimensionless ratio wants.
  const lines = [_plain(c.spec.x.label) + ' ' + c.fx(hits[0].x)]
    .concat(hits.map(h => (h.name ? h.name + ': ' : '') + h.f(h.y)));
  ctx.font = '10px ui-monospace, monospace';
  const w = Math.max(...lines.map(t => ctx.measureText(t).width)) + 12;
  const h = lines.length * 13 + 8;
  const bx = Math.min(mx + 10, canvas.width - c.R - w);
  const by = c.T + 6;
  ctx.fillStyle = 'rgba(255,255,255,0.94)';
  ctx.strokeStyle = INK.grid;
  ctx.fillRect(bx, by, w, h); ctx.strokeRect(bx, by, w, h);
  ctx.fillStyle = INK.text;
  lines.forEach((t, k) => {
    if (k > 0) { ctx.fillStyle = hits[k - 1].col; }
    ctx.fillText(t, bx + 6, by + 14 + k * 13);
  });
  if (focused) {
    ctx.save();
    ctx.strokeStyle = INK.axis; ctx.lineWidth = 2;
    ctx.strokeRect(1, 1, canvas.width - 2, canvas.height - 2);
    ctx.restore();
  }
  return lines.join('. ');
}

/**
 * Pointer and keyboard, reading the same chart the same way.
 *
 * The keyboard path is not a courtesy. A tooltip that is the only way to reach a
 * value gates the data behind a mouse, and every number in these panels was
 * reachable only by hovering — the axis cannot be read to better than a tick.
 * Arrow keys step through the drawn points, Home and End jump to the ends, and
 * the readout is identical because it is the same function.
 */
export function attachHover(canvas, onLeave) {
  const xsOf = () => {
    const c = canvas._chart;
    if (!c) return [];
    const all = [];
    for (const pane of c.panes) {
      for (const s of pane.series) {
        if (s.aside) continue;
        for (let k = 0; k < s.x.length; k++) {
          if (s.y[k] !== null && isFinite(s.y[k])) all.push(s.x[k]);
        }
      }
    }
    return [...new Set(all)].sort((a, b) => a - b);
  };

  canvas.onmousemove = ev => {
    const c = canvas._chart;
    if (!c) return;
    const r = canvas.getBoundingClientRect();
    const mx = (ev.clientX - r.left) * (canvas.width / r.width);
    if (mx < c.L || mx > canvas.width - c.R) return;
    const xv = c.x0 + (mx - c.L) / (canvas.width - c.L - c.R) * (c.x1 - c.x0);
    drawReadout(canvas, xv, false);
  };
  canvas.onmouseleave = () => {
    if (canvas._chart && document.activeElement !== canvas) drawChart(canvas, canvas._chart.spec);
    if (onLeave) onLeave();
  };

  // Reachable, and announced. The canvas carries the readout as its aria-label
  // so a screen reader is told what a sighted reader sees, and the table view
  // under the panel carries every value for anyone who wants them all.
  canvas.tabIndex = 0;
  canvas.setAttribute('role', 'img');
  let ki = -1;
  canvas.onkeydown = ev => {
    const xs = xsOf();
    if (!xs.length) return;
    const step = ev.shiftKey ? Math.max(1, Math.round(xs.length / 10)) : 1;
    if (ev.key === 'ArrowRight') ki = ki < 0 ? 0 : Math.min(xs.length - 1, ki + step);
    else if (ev.key === 'ArrowLeft') ki = ki < 0 ? xs.length - 1 : Math.max(0, ki - step);
    else if (ev.key === 'Home') ki = 0;
    else if (ev.key === 'End') ki = xs.length - 1;
    else if (ev.key === 'Escape') { ki = -1; drawChart(canvas, canvas._chart.spec); return; }
    else return;
    ev.preventDefault();
    const said = drawReadout(canvas, xs[ki], true);
    if (said) canvas.setAttribute('aria-label', said);
  };
  canvas.onblur = () => {
    ki = -1;
    if (canvas._chart) drawChart(canvas, canvas._chart.spec);
  };
  canvas.onfocus = () => {
    if (canvas._said) canvas.setAttribute('aria-label', canvas._said);
  };
}

/**
 * The same numbers as a table, built from the spec the chart was drawn from.
 *
 * Two reasons, and the second is the one that matters. A tooltip that is the
 * only way to reach a value gates the data behind pointing at it, and the WCAG
 * position is that a chart needs an equivalent anyone can read. And a value a
 * person wants to quote in a document should not have to be read off a picture
 * or screenshotted: here it can be selected and copied.
 *
 * Built from `spec` rather than from the panel's own arrays on purpose. A table
 * assembled separately is a second description of the data, and the first time
 * it disagrees with the chart the disagreement is invisible — which is the same
 * argument that put the record's parser next to the engine's.
 */
export function tableFor(spec) {
  // Every series in every pane, in one table, because the stack shares an x and
  // a reader comparing frames is comparing rows. Each column carries its own
  // pane's formatter and, where the series has no name of its own, its pane's y
  // label — which on a stacked spec is the only thing telling the three columns
  // apart.
  const panes = spec.panes && spec.panes.length
    ? spec.panes
    : [{ y: spec.y, series: spec.series }];
  const cols = [];
  for (const pn of panes) {
    for (const s of (pn.series || [])) {
      if (s.x && s.x.length && !s.aside) {
        cols.push({ s, name: s.name || pn.y.label, fy: pn.y.fmt || nice });
      }
    }
  }
  if (!cols.length) return '<p class="muted">nothing plotted.</p>';
  const xs = [...new Set(cols.flatMap(c => c.s.x))].sort((a, b) => a - b);
  const fx = spec.x.fmt || nice;
  const head = ['<tr><th>' + esc(spec.x.label) + '</th>'].concat(
    cols.map(c => '<th>' + esc(c.name) + '</th>')).join('') + '</tr>';
  const at = (c, x) => {
    const k = c.s.x.indexOf(x);
    return k < 0 || c.s.y[k] === null || !isFinite(c.s.y[k]) ? '' : c.fy(c.s.y[k]);
  };
  const rows = xs.map(x =>
    '<tr><td>' + esc(fx(x)) + '</td>' +
    cols.map(c => '<td>' + esc(at(c, x)) + '</td>').join('') + '</tr>').join('');
  return '<table class="fx chart-table"><thead>' + head + '</thead><tbody>' +
    rows + '</tbody></table>';
}

function esc(v) {
  return String(v).replace(/[&<>"]/g, c =>
    ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
}
