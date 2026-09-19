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
  // Secondary text, matching app.css's --ink-2. The finding line wears it: it
  // is content and not furniture, so it may not have the axis grey, and it is
  // not the frame's title either. 7.9:1 on white.
  text2: '#4b4b4b',
  muted: '#8a8880',
  // THE SURFACE IS THE CARD THE CANVAS SITS ON, AND THAT CARD IS WHITE.
  // app.css gives `canvas.plot` `background: var(--card)`, which is #ffffff,
  // inside a 1px rule, on #fbfaf7 paper — so the figure is deliberately a white
  // card and there never was a canvas/page mismatch to fix. What WAS wrong is
  // this token: the 2px separator ring that lets two dots overlap and stay two
  // dots, and the 2px gap between touching bars, are drawn in INK.surface, so
  // both were being drawn three levels off the colour actually underneath them.
  //
  // §34 read the mismatch the other way round and proposed filling the canvas
  // #fcfcfb, which would have put the one-step seam INSIDE the border instead
  // of removing it. Both palettes re-validated against #ffffff — the six
  // categorical hues and `predict`'s four-step ramp — and both still pass, the
  // ramp's light end at 2.11:1 against a floor of 2.
  surface: '#ffffff',
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
 *   x: {label, min, max, ticks?, fmt?,
 *       grid?: false — no VERTICAL rules. A gridline is an invitation to read a
 *       value off the axis, and where the axis is five named scenarios there is
 *       no value between them to read: the rules are then ink that says nothing
 *       and separates categories the eye had already separated },
 *
 *   // ONE PANE, the ordinary case:
 *   y: {label, min, max, ticks?, fmt?, log?},
 *   series: [{ name, kind: 'line'|'dots'|'bars'|'step'|'band', x:[], y:[], colour?, width?, dash?,
 *              y0?: [] — with kind 'band', the OTHER edge. The area between y and
 *              y0 is filled. Drawn in array order like everything else, so a band
 *              goes FIRST in the list and the lines sit on top of it. A null in
 *              either edge breaks the band, the same way it breaks a line,
 *              n?: [] — the sample behind each point; where given, the line fades
 *              as the count falls, so a thin far end does not read as a firm one,
 *              context?: true — THIS IS NOT THE ANSWER. The row a panel is named
 *              for gets full weight; everything drawn to make it legible gets a
 *              thinner stroke and half the contrast, so a five-curve frame has an
 *              entry point instead of five equals. It is a flag rather than an
 *              alpha per panel because "how much should context recede" is one
 *              decision for the whole face, not eleven,
 *              fill?: true — fill between this line and ZERO. For a quantity that
 *              accumulates from nothing, the area is the quantity and a lone
 *              stroke leaves it to be imagined. Refused where zero is off the
 *              frame: a fill running to the floor of an axis that starts at 9
 *              would be drawing an area nobody measured,
 *              aside?: true — FURNITURE, not data. A significance band is a
 *              reference the curve is read against, the same job a mark does, and
 *              it happens to vary with x rather than sit at one value. Drawn, and
 *              named in the legend so the dashes mean something; kept out of the
 *              end labels, the readout and the table, which are for the record }],
 *   marks: [{ axis:'y'|'x', at, label, colour? }
 *           | { axis:'y'|'x', from, to, label, colour? }  — a shaded REGION
 *             rather than a rule. A dashed line at a bound asks the reader to
 *             work out which side they are on; a wash tells them. "Above the
 *             design level", "below zero skill", "the storm regime" are regions,
 *             and each was drawn as its edge ],
 *   notes: [{ x, y, text, colour? }] — A LABEL POINTING AT A PLACE ON A CURVE.
 *             A mark labels a position on an AXIS and runs the width or the
 *             height of the frame; a note points at one point of the data and
 *             says what happens there. "The design is exceeded here", "half a
 *             solar cycle", "273 days the record does not have" are sentences
 *             about a place in the picture, and every one of them was a
 *             paragraph underneath it. Drawn last, on top of everything, with a
 *             leader to a label placed clear of the data,
 *
 *   // OR SEVERAL, STACKED, SHARING ONE X AXIS:
 *   panes: [{ y, series, marks }, …],
 *
 *   finding: string — ONE LINE, INSIDE THE FRAME, SAYING WHAT THE PICTURE SHOWS.
 *     Not the panel's answer, which is a number above the chart and belongs to
 *     the question the panel asks. A finding is a relation between things DRAWN
 *     — which curve is above which and over how much of the axis, where two
 *     lines cross, how many points fall outside a band — phrased so a reader
 *     can check it against the picture and nothing else. Computed from the same
 *     arrays the figure is drawn from, never typed: a sentence a person wrote
 *     about a chart is a second copy of the chart. It quotes the answer again,
 *     it is not a finding.
 *
 *     Drawn under the first pane's y label, wrapped to the plot width, at most
 *     three lines — a finding that needs four is a paragraph and belongs in the
 *     note.
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
    : [{ y: spec.y, series: spec.series, marks: spec.marks, notes: spec.notes }])
    .map(p => ({
      y: p.y || {}, series: p.series || [], marks: p.marks || [],
      notes: p.notes || [],
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
      if (!q.name || q.aside || q.hidden || q.kind === 'bars' || q.kind === 'dots'
          || q.kind === 'band') continue;
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
  // THE FINDING, WRAPPED TO THE PLOT WIDTH, under the FIRST pane's y label. One
  // per figure rather than one per frame: a stack is one argument read down, and
  // three findings in a column is the paragraph this line exists to replace.
  const FIND_H = 13;
  ctx.font = '10px ui-monospace, monospace';
  const findLines = spec.finding ? _wrapText(ctx, spec.finding, W - L - R, 3) : [];
  // Each pane's own header: the y label, the finding on the first pane, and
  // however many legend rows the pane needs.
  const bands = legs.map((g, i) =>
    18 + (i === 0 ? findLines.length * FIND_H : 0) + g.rows.length * LEG_H);

  ctx.clearRect(0, 0, W, H);
  // THE SURFACE TOKEN, NOT A LITERAL. Same colour as before — the card is
  // white — but now there is one place that says what the surface is, and the
  // rings and bar gaps drawn in it agree with the fill underneath them.
  ctx.fillStyle = INK.surface;
  ctx.fillRect(0, 0, W, H);

  // THE X EXTENT IS COMPUTED ACROSS EVERY PANE, which is what registers them: a
  // vertical read down a stack has to land on the same lead in all three, and a
  // pane scaled to its own x would look right and be a different picture.
  const allS = panes.flatMap(p => p.series).filter(s => s.x.length);
  const xsAll = allS.flatMap(s => s.x.filter(v => v !== null && isFinite(v)));
  let x0 = xsAll.length ? Math.min(...xsAll) : 0;
  let x1 = xsAll.length ? Math.max(...xsAll) : 1;
  // A RULE HAS `at`; A REGION HAS `from`/`to`, AND AN OPEN END IS NOT A NUMBER.
  // This read m.at only, so the first x region ever drawn — §28.2's gap in the
  // thermosphere flux view — put `undefined` through Math.min and made the
  // extent NaN. Every x position is then NaN, so the frame drew its axes, its
  // ticks and not one mark of data: a blank picture that passed "the mount is
  // not blank" because the axis labels are still on it, passed its reference at
  // 2.4 per cent because three thin lines and some text ARE about 2.4 per cent
  // of a white canvas, and failed 2b for the one honest reason — a chart that
  // draws nothing draws the same nothing whatever the engine answers.
  //
  // The y extent below had always walked all three fields. The two loops did the
  // same job and only one of them knew about regions.
  for (const pn of panes) {
    for (const m of pn.marks) {
      if (m.axis !== 'x') continue;
      for (const v of [m.at, m.from, m.to]) {
        if (v === undefined || v === null || !isFinite(v)) continue;
        x0 = Math.min(x0, v); x1 = Math.max(x1, v);
      }
    }
  }
  const given = v => v !== undefined && v !== null && isFinite(v);
  if (given(spec.x.min)) x0 = spec.x.min;
  if (given(spec.x.max)) x1 = spec.x.max;
  // AND IT IS NOT ALLOWED TO BE SILENT AGAIN. A non-finite extent cannot draw
  // anything, and the failure mode is an empty frame that looks like a panel
  // with no data rather than like a bug. Thrown, the face marks the mount
  // `data-failed` and every check in tools/panel_check.py sees it at once.
  if (!isFinite(x0) || !isFinite(x1)) {
    throw new Error('the x extent is not a number: ' + x0 + ' to ' + x1 +
      ' — a mark or a series is carrying something that is not a coordinate');
  }
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
    head: geom[pi].head, leg: legs[pi], LEG_H, given, xgrid: spec.x.grid,
    find: pi === 0 ? findLines : [], FIND_H,
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
 * The hue of every series in one pane, resolved once and in one place.
 *
 * A series with no `colour` of its own takes the next hue in the fixed order —
 * and "next" must not count the bands. A band is the area under or between the
 * lines it belongs to rather than a series of its own, so inserting one pushed
 * every unnamed series below it one hue along: the design-window fill moved the
 * hot single-day curve onto the blue its sustained neighbour had already
 * declared, and two curves in one frame came out the same colour. Nothing
 * failed; the picture just started lying about which line was which.
 *
 * A band with no colour of its own borrows the hue of the series that follows
 * it, which is what a band drawn under one line wants anyway.
 */
function _hues(series) {
  let ci = 0;
  return series.map(s => s.colour
    || INK.series[(s.kind === 'band' ? ci : ci++) % INK.series.length]);
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
  const find = o.find || [], FIND_H = o.FIND_H || 13;
  const T = top, BOT = bot;

  // THE EXTENT IS ALWAYS COMPUTED, AND A DECLARED BOUND THEN OVERRIDES ITS OWN
  // END. This used to be two either-or branches keyed on the MINIMUM: a spec
  // that supplied `min` and left `max` open skipped the branch entirely and left
  // the far end undefined, so every tick came out NaN and the chart drew nothing
  // but gridlines. The Segmentation histogram declares `min: 0` on both axes and
  // had been rendering blank.
  const all = pn.series.filter(s => s.x.length && !s.hidden);
  // A band's far edge is data too. Without this the frame is scaled to one side
  // of it and the fill runs off the top.
  const ysAll = all.flatMap(s => (s.y || []).concat(s.y0 || [])
    .filter(v => v !== null && isFinite(v)));
  let y0 = ysAll.length ? Math.min(...ysAll) : 0;
  let y1 = ysAll.length ? Math.max(...ysAll) : 1;
  // A mark outside the data is still a fact about the data, so the frame
  // grows to hold it. Clipping a bound off the top draws a picture in which
  // the bound is always met.
  for (const m of pn.marks) {
    if (m.axis === 'x') continue;
    // A region is bounded by `from`/`to`; a rule by `at`. Either way a mark
    // outside the data is still a fact about the data and the frame grows to
    // hold it — except an open end, which is a region meaning "everything above
    // this" and must not drag the axis to infinity.
    for (const v of [m.at, m.from, m.to]) {
      if (v === undefined || v === null || !isFinite(v)) continue;
      y0 = Math.min(y0, v); y1 = Math.max(y1, v);
    }
  }
  if (!given(pn.y.min) || !given(pn.y.max)) {
    const pad = (y1 - y0) * 0.06 || 1;
    if (!given(pn.y.min)) y0 -= pad;
    if (!given(pn.y.max)) y1 += pad;
  }
  if (given(pn.y.min)) y0 = pn.y.min;
  if (given(pn.y.max)) y1 = pn.y.max;
  if (!isFinite(y0) || !isFinite(y1)) {
    throw new Error('the y extent of "' + _plain(pn.y.label) + '" is not a number: ' +
      y0 + ' to ' + y1 + ' — a mark or a series is carrying something that is not a coordinate');
  }
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
  // `grid: false` MEANS NO VERTICAL RULE AT ALL, INCLUDING AT ZERO. The first
  // form of this kept the zero rule on the grounds that which side of zero a
  // point sits on is a fact — true of a signed quantity, and a signed quantity
  // is never the axis that asks for this. On `drivers` the axis is five named
  // scenarios running -0.3 to 4.3, so the kept rule landed on the first
  // category and was the only vertical line in the frame: a mark, drawn darker
  // than a gridline, at a place that means "index 0".
  for (const at of xt) {
    if (o.xgrid === false) continue;
    const isZero = Math.abs(at) < 1e-12;
    const x = Math.round(px(at)) + 0.5;
    if (x < L - 0.5 || x > W - R + 0.5) continue;
    ctx.strokeStyle = isZero ? INK.axis : INK.grid;
    ctx.beginPath(); ctx.moveTo(x, T); ctx.lineTo(x, BOT); ctx.stroke();
    ctx.strokeStyle = INK.grid;
  }
  ctx.textAlign = 'left';

  // REGIONS, BEHIND EVERYTHING. A wash under the grid rather than over the data:
  // it is context for the marks, not a mark itself, and a fill on top of a line
  // would dim the line to say something about the space around it.
  //
  // An open end — `to` left off, or infinite — means "everything beyond this",
  // and is clamped to the frame rather than growing it. That is what makes
  // "above the design level" drawable without an upper bound to invent.
  for (const m of pn.marks) {
    if (m.from === undefined && m.to === undefined) continue;
    const lo = (m.from === undefined || m.from === null || !isFinite(m.from))
      ? -Infinity : m.from;
    const hi = (m.to === undefined || m.to === null || !isFinite(m.to))
      ? Infinity : m.to;
    ctx.save();
    ctx.fillStyle = m.colour || INK.mark;
    ctx.globalAlpha = m.alpha === undefined ? 0.07 : m.alpha;
    if (m.axis === 'x') {
      const a = Math.max(L, px(Math.max(lo, x0))), b = Math.min(W - R, px(Math.min(hi, x1)));
      if (b > a) ctx.fillRect(a, T, b - a, BOT - T);
    } else {
      const a = Math.min(BOT, py(Math.min(hi, y1))), b = Math.max(T, py(Math.max(lo, y0)));
      if (b > a) ctx.fillRect(L, a, W - L - R, b - a);
    }
    ctx.restore();
  }

  // MARK SPECS. Lines 2px with round joins and caps, so a curve does not go
  // spiky at a corner and a one-point run still draws. Dots at radius 4 — an
  // 8px mark is the smallest a person can point at — each carrying a 2px ring
  // in the SURFACE colour, which is what lets two dots overlap and stay two
  // dots. The ring is the separator; a stroke around a mark would be ink that
  // is not data.
  const ends = [];
  const hues = _hues(pn.series);
  // HOW FAR CONTEXT RECEDES IS ONE DECISION, HERE. A series marked `context` is
  // drawn at half contrast and, where it did not ask for a width, thinner. The
  // alternative was an alpha per panel, which is eleven decisions that start the
  // same and end up different.
  const CONTEXT_A = 0.5, CONTEXT_W = 1.3;
  pn.series.forEach((s, i) => {
    const col = hues[i];
    // HIDDEN, NOT REMOVED. A muted series stays in the array so `_hues` walks
    // the same list and the survivors keep their colours — a filter that
    // repaints what is left of a chart is the one interaction that can make a
    // reader misread the series they were comparing. The frame DOES rescale to
    // what is left, because rescaling is the whole point of isolating two
    // curves that sit on top of each other.
    if (s.hidden) return;
    const dim = s.context ? CONTEXT_A : 1;
    ctx.save();
    ctx.strokeStyle = col; ctx.fillStyle = col;
    ctx.lineWidth = s.width || (s.context ? CONTEXT_W : 2);
    ctx.lineJoin = 'round'; ctx.lineCap = 'round';
    if (s.dash) ctx.setLineDash(s.dash);
    if (s.kind === 'band') {
      // THE AREA BETWEEN TWO EDGES, and a null in either breaks it.
      //
      // Filled in one pass per unbroken run rather than as one polygon: a band
      // that closed across a gap would fill a region neither edge was measured
      // over, which is the area equivalent of drawing a line across a hole.
      ctx.globalAlpha = s.alpha === undefined ? 0.14 : s.alpha;
      const ok = k => {
        const a = s.y[k], b = s.y0 ? s.y0[k] : null;
        return a !== null && isFinite(a) && b !== null && isFinite(b)
          && !(lg && (a <= 0 || b <= 0));
      };
      let run = [];
      const flush = () => {
        if (run.length > 1) {
          ctx.beginPath();
          ctx.moveTo(px(s.x[run[0]]), py(s.y[run[0]]));
          for (const k of run) ctx.lineTo(px(s.x[k]), py(s.y[k]));
          for (let j = run.length - 1; j >= 0; j--) {
            ctx.lineTo(px(s.x[run[j]]), py(s.y0[run[j]]));
          }
          ctx.closePath();
          ctx.fill();
        }
        run = [];
      };
      for (let k = 0; k < s.x.length; k++) {
        if (ok(k)) run.push(k);
        else flush();
      }
      flush();
      ctx.globalAlpha = 1;
    } else if (s.kind === 'dots') {
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

      // THE AREA UNDER A LINE, WHERE THE AREA IS THE QUANTITY. A quantity that
      // accumulates from nothing — the temperature a term ADDS, an error
      // measured from perfect — is a size, and a lone stroke leaves that size to
      // be imagined off the axis. Refused where zero is off the frame, because
      // the fill would then run to a floor that is not zero and draw an area
      // nobody measured. Drawn before the stroke, so the line stays crisp.
      if (s.fill && s.kind !== 'step') {
        if (y0 > 0 || y1 < 0) {
          throw new Error('a fill under "' + (s.name || _plain(pn.y.label)) +
            '" was asked for, but zero is not on its axis (' + y0 + ' to ' + y1 +
            ') — the area would be measured from the frame and not from zero');
        }
        const base = py(0);
        ctx.save();
        ctx.globalAlpha = (s.fillAlpha === undefined ? 0.12 : s.fillAlpha) * dim;
        let run = [];
        const shed = () => {
          if (run.length > 1) {
            ctx.beginPath();
            ctx.moveTo(px(s.x[run[0]]), base);
            for (const k of run) ctx.lineTo(px(s.x[k]), py(s.y[k]));
            ctx.lineTo(px(s.x[run[run.length - 1]]), base);
            ctx.closePath();
            ctx.fill();
          }
          run = [];
        };
        for (let k = 0; k < s.x.length; k++) {
          const v = s.y[k];
          if (v !== null && isFinite(v) && !(lg && v <= 0)) run.push(k);
          else shed();
        }
        shed();
        ctx.restore();
      }
      const segment = (a, b) => {
        ctx.beginPath();
        ctx.moveTo(px(s.x[a]), py(s.y[a]));
        if (s.kind === 'step') { ctx.lineTo(px(s.x[b]), py(s.y[a])); }
        ctx.lineTo(px(s.x[b]), py(s.y[b]));
        ctx.stroke();
      };
      const live = k => { const v = s.y[k]; return v !== null && isFinite(v) && !(lg && v <= 0); };
      ctx.globalAlpha = dim;
      if (N) {
        for (let k = 1; k < s.x.length; k++) {
          if (!live(k) || !live(k - 1)) continue;
          ctx.globalAlpha = Math.min(wt(k), wt(k - 1)) * dim;
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
        ctx.globalAlpha = 1;
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
      if (s.name && !s.aside && s.kind !== 'band'
          && pn.series.filter(q => q.name).length > 1) {
        let lastK = -1;
        for (let k = s.x.length - 1; k >= 0; k--) {
          const v = s.y[k];
          if (v !== null && isFinite(v) && !(lg && v <= 0)) { lastK = k; break; }
        }
        if (lastK >= 0) {
          const X = px(s.x[lastK]), Y = py(s.y[lastK]);
          if (X >= L && X <= W - R && Y >= T && Y <= BOT) {
            ends.push({ x: X, y: Y, col, text: fy(s.y[lastK]), dim });
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
    // The label recedes with its line. A context curve drawn at half contrast
    // with its number at full strength puts the emphasis back where the width
    // just took it from.
    ctx.globalAlpha = e.dim === undefined ? 1 : e.dim;
    ctx.fillStyle = e.col;
    ctx.fillText(e.text, W - R + 6, y);
    ctx.globalAlpha = 1;
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
    // A REGION IS ALREADY DRAWN, as the wash above; it needs no rule. Its label
    // is placed at whichever edge the frame actually holds, so "everything above
    // the design level" labels the bottom of its own shading rather than an
    // infinity the axis does not reach.
    const region = m.from !== undefined || m.to !== undefined;
    const at = region
      ? (m.from !== undefined && m.from !== null && isFinite(m.from) ? m.from : m.to)
      : m.at;
    if (at === undefined || at === null || !isFinite(at)) continue;
    if (!region) {
      ctx.save();
      ctx.strokeStyle = col; ctx.fillStyle = col; ctx.lineWidth = 1.2;
      ctx.setLineDash([5, 4]);
      ctx.beginPath();
      if (m.axis === 'x') { ctx.moveTo(px(at), T); ctx.lineTo(px(at), BOT); }
      else { ctx.moveTo(L, py(at)); ctx.lineTo(W - R, py(at)); }
      ctx.stroke();
      ctx.restore();
    }
    if (!m.label) continue;
    ctx.fillStyle = col;
    const w = ctx.measureText(m.label).width + 8;
    if (m.axis === 'x') {
      const x = Math.min(W - R - w, px(at) + 4);
      ctx.fillText(m.label, x, T + 11 + 12 * slotFor(x, w));
    } else {
      // A y label is nudged DOWN rather than sideways: sideways would put it
      // over the data it is annotating.
      let y = Math.max(T + 10, py(at) - 4);
      while (taken.y.some(t => Math.abs(t.at - y) < 11) && y < BOT - 2) y += 11;
      // The box as well as the row, because the notes below place themselves
      // against these: a note is drawn after every mark and had no way to know
      // where a mark's label had already landed.
      taken.y.push({ at: y, x: L + 4, w });
      ctx.fillText(m.label, L + 4, y);
    }
  }

  // NOTES, LAST AND ON TOP OF EVERYTHING. A leader from the point to a label
  // set clear of it, and a ringed dot at the point itself so the place being
  // talked about is unambiguous on a curve two pixels wide.
  //
  // Placement is up-and-right, flipping left where the label would leave the
  // frame and down where there is no room above; where two notes would collide
  // the second drops a line. No cleverness beyond that: a note whose text does
  // not fit beside its point is a note that needs shorter text.
  const notes = (pn.notes || []).filter(n =>
    n && isFinite(n.x) && isFinite(n.y) && n.text);
  if (notes.length) {
    ctx.font = '10px ui-monospace, monospace';
    // Seeded with the y-mark labels, which are already on the canvas. Without
    // this a note landing beside a bound's label printed a line above it and
    // read as part of it.
    const placed = taken.y.map(t => ({ x: t.x, y: t.at, w: t.w }));
    for (const n of notes) {
      const X = px(n.x), Y = py(n.y);
      // A note about a point the frame does not hold is not drawn at the edge
      // pretending to be about the edge.
      if (X < L - 2 || X > W - R + 2 || Y < T - 2 || Y > BOT + 2) continue;
      const col = n.colour || INK.text;
      const tw = ctx.measureText(n.text).width;
      const LEAD = 15;
      const dir = (X + LEAD + tw + 6 <= W - R) ? 1 : -1;
      let ly = Y - LEAD;
      const above = ly >= T + 12;
      if (!above) ly = Y + LEAD + 8;
      const lx = dir === 1 ? X + LEAD : X - LEAD - tw;
      // A COLLIDING LABEL MOVES AWAY FROM ITS POINT, NOT DOWNWARD. Stepping
      // always down pushed `design`'s "exceeded here" from above its bound to
      // below it, so the note about the exceeded side ended up labelling the
      // side that is not exceeded. Which side of a line a label sits on is read
      // as part of what it says.
      const step = above ? -13 : 13;
      let guard = 0;
      while (guard++ < 8 && placed.some(q =>
        Math.abs(q.y - ly) < 12 && lx < q.x + q.w && lx + tw > q.x)) {
        ly += step;
        if (ly < T + 12 || ly > BOT - 2) { ly -= step * guard; break; }
      }
      placed.push({ x: lx, y: ly, w: tw });
      ctx.save();
      ctx.strokeStyle = col; ctx.fillStyle = col; ctx.lineWidth = 1;
      ctx.globalAlpha = 0.8;
      ctx.beginPath();
      ctx.moveTo(X, Y);
      ctx.lineTo(dir === 1 ? lx - 4 : lx + tw + 4, ly - 3);
      ctx.stroke();
      ctx.globalAlpha = 1;
      // The same 2px surface ring every other point marker in this face wears,
      // so the dot reads as a marker on the line rather than as a datum of its
      // own.
      ctx.strokeStyle = INK.surface; ctx.lineWidth = 2;
      ctx.beginPath(); ctx.arc(X, Y, 2.5, 0, 6.284); ctx.stroke();
      ctx.beginPath(); ctx.arc(X, Y, 2.5, 0, 6.284); ctx.fill();
      ctx.fillText(n.text, lx, ly);
      ctx.restore();
    }
  }

  // THE PANE'S OWN HEADER: its y label, and its legend under it. Positioned from
  // the pane's header top rather than from the canvas, which is what lets three
  // frames each name their own quantity.
  ctx.fillStyle = INK.text; ctx.font = '11px ui-monospace, monospace';
  ctx.fillText(pn.y.label, L, head + 12);

  // The finding, between the frame's name and its key. It is the one line a
  // reader who reads nothing else should get, so it sits where the eye already
  // is rather than over the data.
  if (find.length) {
    ctx.font = '10px ui-monospace, monospace';
    ctx.fillStyle = INK.text2;
    ctx.textAlign = 'left';
    find.forEach((line, i) => ctx.fillText(line, L, head + 12 + (i + 1) * FIND_H));
  }
  const findH = find.length * FIND_H;

  // WHERE EVERY LEGEND ENTRY WAS DRAWN, so a click can find it. Collected
  // rather than recomputed: a hit box worked out a second time from the same
  // inputs is a second layout, and the first time the two disagree the symptom
  // is a legend that responds to clicks a few pixels from where it looks.
  const legendHits = [];
  if (leg.rows.length) {
    ctx.font = '10px ui-monospace, monospace';
    ctx.textAlign = 'left';
    leg.rows.forEach((row, r) => {
      let lx = L;
      const ly = head + 12 + findH + (r + 1) * LEG_H;
      for (const s of row) {
        const col = hues[pn.series.indexOf(s)];
        const tw = ctx.measureText(s.name).width;
        // The SWATCH recedes with its line; the NAME does not. A legend entry
        // greyed out reads as disabled, and text in this face wears text ink
        // rather than its series' — the colour chip beside it carries identity.
        //
        // A HIDDEN one IS disabled, and says so: its swatch hollows out to a
        // ring and its name goes grey with a rule through it, so "off" is a
        // state a reader can see rather than a series they think is missing.
        ctx.globalAlpha = s.context && !s.hidden ? 0.5 : 1;
        ctx.fillStyle = col;
        if (s.hidden) {
          ctx.strokeStyle = col; ctx.lineWidth = 1;
          ctx.strokeRect(lx + 0.5, ly - 5.5, 13, 3);
        } else {
          ctx.fillRect(lx, ly - 5, 14, 3);
        }
        ctx.globalAlpha = 1;
        ctx.fillStyle = s.hidden ? INK.axis : INK.text;
        ctx.fillText(s.name, lx + 19, ly);
        if (s.hidden) {
          ctx.strokeStyle = INK.axis; ctx.lineWidth = 1;
          ctx.beginPath();
          ctx.moveTo(lx + 19, ly - 3.5); ctx.lineTo(lx + 19 + tw, ly - 3.5);
          ctx.stroke();
        }
        legendHits.push({ x: lx - 3, y: ly - 11, w: 19 + tw + 6, h: 15, name: s.name });
        lx += 19 + tw + 18;
      }
    });
  }

  return { y: pn.y, series: pn.series, marks: pn.marks, hues, py, fy, y0, y1,
    top: T, bot: BOT, log: lg, legendHits };
}

/**
 * Greedy word wrap to a pixel width, refusing to run past `maxLines`.
 *
 * Refusing rather than truncating, because a silently shortened finding is a
 * claim with its qualifier cut off — "the outlook beats persistence" where the
 * text said "from lead 1 to 22" — and that is a worse sentence than none. A
 * finding that needs four lines is a paragraph and belongs in the note under
 * the chart.
 */
function _wrapText(ctx, text, width, maxLines) {
  const words = String(text).split(/\s+/).filter(Boolean);
  const lines = [];
  let line = '';
  for (const w of words) {
    const probe = line ? line + ' ' + w : w;
    if (line && ctx.measureText(probe).width > width) { lines.push(line); line = w; }
    else line = probe;
  }
  if (line) lines.push(line);
  if (lines.length > maxLines) {
    throw new Error('the finding needs ' + lines.length + ' lines and ' + maxLines +
      ' is the limit — it is a paragraph, and the note below the chart is where a ' +
      'paragraph goes: "' + text + '"');
  }
  return lines;
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
    if (!s.x.length || s.aside || s.hidden || s.kind === 'band') return;
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
    const col = pane.hues[i];
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
export function attachHover(canvas, handlers) {
  // Back-compatible: relation.js passes a bare onLeave, the solar face passes a
  // table of handlers. A figure that wires nothing keeps exactly the pointer it
  // had — hover, arrow keys, Escape — and gains no interaction it has not asked
  // for.
  const h = typeof handlers === 'function' ? { onLeave: handlers } : (handlers || {});
  const onLeave = h.onLeave;
  const xsOf = () => {
    const c = canvas._chart;
    if (!c) return [];
    const all = [];
    for (const pane of c.panes) {
      for (const s of pane.series) {
        if (s.aside || s.hidden || s.kind === 'band') continue;
        for (let k = 0; k < s.x.length; k++) {
          if (s.y[k] !== null && isFinite(s.y[k])) all.push(s.x[k]);
        }
      }
    }
    return [...new Set(all)].sort((a, b) => a - b);
  };

  // Canvas pixels from a pointer event, whatever the element is scaled to.
  const at = ev => {
    const r = canvas.getBoundingClientRect();
    return {
      mx: (ev.clientX - r.left) * (canvas.width / r.width),
      my: (ev.clientY - r.top) * (canvas.height / r.height),
    };
  };
  const dataX = (c, mx) => c.x0 + (mx - c.L) / (canvas.width - c.L - c.R) * (c.x1 - c.x0);
  const inPlot = (c, mx, my) =>
    mx >= c.L && mx <= canvas.width - c.R && my >= c.T && my <= c.bot;

  // THE BRUSH. A drag across the plot selects a range of x; the band is drawn
  // over the finished chart rather than by redrawing it, so dragging costs one
  // fill and not one full render per frame.
  let drag = null;
  const MIN_DRAG = 6;

  canvas.onmousedown = ev => {
    const c = canvas._chart;
    // A press starts tracking if ANY pointer handler is wired: a click is a
    // drag that did not move, so isolating a series and opening a row come
    // through the same path as the brush and must not depend on it.
    if (!c || ev.button !== 0) return;
    if (!h.onBrush && !h.onIsolate && !h.onOpenRow) return;
    const { mx, my } = at(ev);
    // TRACKED WHEREVER THE PRESS LANDS, and only the BRUSH cares that it
    // started inside the plot. Requiring the plot here meant a press on a
    // legend entry — which sits in the header band, above the frame — never
    // started a gesture at all, so no mouseup arrived, so the click that hides
    // a series did nothing. The symptom was a picture that changed anyway,
    // because moving the pointer there had drawn the hover crosshair.
    // The finished chart, kept as pixels. Redrawing it on every pointer move
    // would re-render ten thousand dots to move two vertical lines, and on the
    // density panel that is visible as lag.
    drag = { from: mx, to: mx, my0: my, moved: false, plot: inPlot(c, mx, my),
      snap: canvas.getContext('2d').getImageData(0, 0, canvas.width, canvas.height) };
    ev.preventDefault();
  };

  canvas.onmousemove = ev => {
    const c = canvas._chart;
    if (!c) return;
    const { mx, my } = at(ev);
    if (drag) {
      drag.to = Math.max(c.L, Math.min(canvas.width - c.R, mx));
      if (drag.plot && Math.abs(drag.to - drag.from) >= MIN_DRAG) drag.moved = true;
      const g = canvas.getContext('2d');
      g.putImageData(drag.snap, 0, 0);
      if (drag.moved) {
        const a = Math.min(drag.from, drag.to), b = Math.max(drag.from, drag.to);
        g.save();
        g.fillStyle = INK.mark; g.globalAlpha = 0.10;
        g.fillRect(a, c.T, b - a, c.bot - c.T);
        g.globalAlpha = 1; g.strokeStyle = INK.mark; g.lineWidth = 1;
        g.beginPath();
        g.moveTo(a + 0.5, c.T); g.lineTo(a + 0.5, c.bot);
        g.moveTo(b - 0.5, c.T); g.lineTo(b - 0.5, c.bot);
        g.stroke();
        g.restore();
      }
      return;
    }
    if (mx < c.L || mx > canvas.width - c.R) return;
    drawReadout(canvas, dataX(c, mx), false);
  };

  canvas.onmouseup = ev => {
    const c = canvas._chart;
    if (!drag) return;
    const d = drag; drag = null;
    if (!c) return;
    if (!d.moved) { _clicked(c, { mx: d.from, my: d.my0 }); return; }
    // The band comes off before the handler runs. If the face decides the
    // window is not worth honouring it does nothing, and a rectangle left on
    // the canvas would be the only sign anything had happened.
    drawChart(canvas, c.spec);
    const a = dataX(c, Math.min(d.from, d.to)), b = dataX(c, Math.max(d.from, d.to));
    if (h.onBrush) h.onBrush(a, b);
  };

  canvas.ondblclick = () => { if (h.onReset) h.onReset(); };

  // A CLICK IS A DRAG THAT DID NOT MOVE, so both arrive here and nothing has to
  // decide between two listeners. A legend entry toggles its series; a point on
  // a series that names a row opens that row.
  const _clicked = (c, pt) => {
    for (const pane of c.panes) {
      for (const hit of (pane.legendHits || [])) {
        if (pt.mx >= hit.x && pt.mx <= hit.x + hit.w &&
            pt.my >= hit.y && pt.my <= hit.y + hit.h) {
          if (h.onIsolate) h.onIsolate(hit.name);
          return;
        }
      }
    }
    if (!h.onOpenRow || !inPlot(c, pt.mx, pt.my)) return;
    // The nearest drawn point within a finger's reach, across every pane. A
    // series with no `row` is not a miss to report — most curves in this face
    // are computed from the record and have no row behind them.
    let best = null;
    for (const pane of c.panes) {
      for (const ser of pane.series) {
        if (!ser.row || ser.hidden || !ser.x.length) continue;
        for (let k = 0; k < ser.x.length; k++) {
          const y = ser.y[k];
          if (y === null || !isFinite(y)) continue;
          const dx = c.px(ser.x[k]) - pt.mx, dy = pane.py(y) - pt.my;
          const d2 = dx * dx + dy * dy;
          if (!best || d2 < best.d2) best = { d2, row: ser.row };
        }
      }
      for (const m of (pane.marks || [])) {
        if (!m.row || m.at === undefined || !isFinite(m.at)) continue;
        const d = m.axis === 'x' ? Math.abs(c.px(m.at) - pt.mx) : Math.abs(pane.py(m.at) - pt.my);
        if (!best || d * d < best.d2) best = { d2: d * d, row: m.row };
      }
    }
    if (best && best.d2 <= 14 * 14) h.onOpenRow(best.row);
  };

  canvas.onmouseleave = () => {
    drag = null;
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
    else if (ev.key === 'Escape') {
      // Escape clears the readout; a second Escape, with nothing to clear,
      // undoes whatever the pointer did to the view. The keyboard reaches the
      // same state the pointer can put the figure into, which is the whole
      // reason the arrow keys exist here.
      if (ki < 0 && h.onReset) { h.onReset(); return; }
      ki = -1; drawChart(canvas, canvas._chart.spec); return;
    }
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
 * THE VIEW A READER HAS ASKED FOR, AS A SPEC.
 *
 * Zoom, mute and pin are not chart modes — they are a transform from the spec a
 * panel built to the spec that gets drawn. Everything downstream then agrees by
 * construction: the picture, the readout, the arrow-key ladder and the table
 * under the panel are all reading the one spec, so "the numbers behind this
 * picture" means THIS picture and not the one before the reader touched it.
 *
 * `view` is { zoom: [lo, hi] | null, hidden: Set<name>, pinned: spec | null }.
 *
 * ZOOM FILTERS THE POINTS rather than clamping the axis. Clamping would leave
 * the table listing fifteen years of leads under a frame showing two, and would
 * leave the y axis scaled to data the frame no longer holds. Marks are kept
 * whatever the window: a bound outside the view is still the bound.
 *
 * PIN OVERLAYS, AND REFUSES WHEN IT CANNOT. Two views of one panel are only
 * comparable if they are drawn against the same quantities, so the pinned spec
 * is dropped unless its axis titles match the live one's — silently overlaying
 * sfu on Ap is the dual-axis mistake wearing different clothes. The pinned
 * series are drawn in one neutral ink at context weight rather than taking
 * fresh hues: they are a reference, not a second categorical set, and a palette
 * cannot be asked for ten more colours.
 */
export function viewSpec(spec, view) {
  const v = view || {};
  const hidden = v.hidden || new Set();
  const win = v.zoom && isFinite(v.zoom[0]) && isFinite(v.zoom[1]) && v.zoom[1] > v.zoom[0]
    ? v.zoom : null;
  const cut = ser => {
    if (!win) return ser;
    const keep = [];
    for (let k = 0; k < ser.x.length; k++) {
      if (ser.x[k] >= win[0] && ser.x[k] <= win[1]) keep.push(k);
    }
    const pick = arr => (Array.isArray(arr) ? keep.map(k => arr[k]) : arr);
    return { ...ser, x: keep.map(k => ser.x[k]), y: pick(ser.y), y0: pick(ser.y0), n: pick(ser.n) };
  };
  const dress = ser => {
    const out = cut(ser);
    return hidden.has(ser.name) ? { ...out, hidden: true } : out;
  };
  const pinnedOf = pane => {
    if (!v.pinned) return [];
    const src = v.pinned.panes && v.pinned.panes.length ? v.pinned.panes : [v.pinned];
    const twin = src.find(q => (q.y || {}).label === (pane.y || {}).label);
    if (!twin || (v.pinned.x || {}).label !== (spec.x || {}).label) return [];
    return (twin.series || [])
      .filter(q => q.name && !q.hidden && q.kind !== 'band')
      .map(q => ({ ...cut(q), name: q.name + ' (pinned)', colour: INK.muted,
        context: true, dash: [2, 3], aside: false }));
  };
  const onePane = pane => ({
    ...pane,
    series: (pane.series || []).map(dress).concat(pinnedOf(pane)),
  });
  const out = { ...spec };
  if (win) out.x = { ...spec.x, min: win[0], max: win[1] };
  if (spec.panes && spec.panes.length) out.panes = spec.panes.map(onePane);
  else Object.assign(out, onePane({ y: spec.y, series: spec.series, marks: spec.marks,
    notes: spec.notes }));
  return out;
}

/** Is any part of this view not the one the panel built? */
export function viewIsOn(view) {
  return !!(view && (view.zoom || view.pinned || (view.hidden && view.hidden.size)));
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
function _grid(spec) {
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
      if (s.x && s.x.length && !s.aside && !s.hidden && s.kind !== 'band') {
        cols.push({ s, name: s.name || pn.y.label, fy: pn.y.fmt || nice });
      }
    }
  }
  if (!cols.length) return null;
  const xs = [...new Set(cols.flatMap(c => c.s.x))].sort((a, b) => a - b);
  const fx = spec.x.fmt || nice;
  const at = (c, x) => {
    const k = c.s.x.indexOf(x);
    return k < 0 || c.s.y[k] === null || !isFinite(c.s.y[k]) ? '' : String(c.fy(c.s.y[k]));
  };
  return {
    head: [_plain(spec.x.label)].concat(cols.map(c => _plain(c.name))),
    rows: xs.map(x => [String(fx(x))].concat(cols.map(c => at(c, x)))),
    xLabel: spec.x.label,
    names: cols.map(c => c.name),
  };
}

export function tableFor(spec) {
  const g = _grid(spec);
  if (!g) return '<p class="muted">nothing plotted.</p>';
  const head = '<tr><th>' + esc(g.xLabel) + '</th>' +
    g.names.map(n => '<th>' + esc(n) + '</th>').join('') + '</tr>';
  const rows = g.rows.map(r =>
    '<tr>' + r.map(c => '<td>' + esc(c) + '</td>').join('') + '</tr>').join('');
  return '<table class="fx chart-table"><thead>' + head + '</thead><tbody>' +
    rows + '</tbody></table>';
}

/**
 * The same grid as text, for the clipboard.
 *
 * Built from `_grid` rather than scraped out of the rendered table or rebuilt
 * from the panel's arrays: a third description of the same numbers is a third
 * chance for them to disagree, and a reader who pastes a column into a document
 * has no way of telling which of the three they got. Tab-separated because that
 * is what a spreadsheet and a document table both accept without being asked.
 *
 * Column headers lose their unit brackets here, the same way the readout does,
 * because the unit belongs to the axis title and a pasted "F10.7  [sfu]" is a
 * header nobody wants.
 */
export function tableTsv(spec) {
  const g = _grid(spec);
  if (!g) return '';
  return [g.head].concat(g.rows).map(r => r.join('\t')).join('\n') + '\n';
}

function esc(v) {
  return String(v).replace(/[&<>"]/g, c =>
    ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
}
