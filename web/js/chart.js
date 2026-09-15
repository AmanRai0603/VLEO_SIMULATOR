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

export const INK = {
  grid: '#ece8de',
  axis: '#8a8880',
  text: '#1a1a1a',
  muted: '#8a8880',
  series: ['#b5731a', '#2a6f97', '#7a9e3f', '#8a3ffc', '#c1440e', '#4a4a4a'],
  mark: '#8a3ffc',
};

const nice = v => {
  const a = Math.abs(v);
  if (a === 0) return '0';
  if (a >= 1000) return v.toFixed(0);
  if (a >= 10) return v.toFixed(1);
  if (a >= 1) return v.toFixed(2);
  return v.toPrecision(3);
};

/**
 * @param spec {
 *   x: {label, min, max, ticks?, fmt?},
 *   y: {label, min, max, ticks?, fmt?},
 *   series: [{ name, kind: 'line'|'dots'|'bars'|'step', x:[], y:[], colour?, width?, dash? }],
 *   marks: [{ axis:'y'|'x', at, label, colour? }],
 *   note: string
 * }
 */
export function drawChart(canvas, spec) {
  const ctx = canvas.getContext('2d');
  const W = canvas.width, H = canvas.height;
  const L = 74, R = 18, T = 18, B = 42;
  ctx.clearRect(0, 0, W, H);
  ctx.fillStyle = '#fff';
  ctx.fillRect(0, 0, W, H);

  const all = spec.series.filter(s => s.x.length);
  // THE EXTENT IS ALWAYS COMPUTED, AND A DECLARED BOUND THEN OVERRIDES ITS OWN
  // END. This used to be two either-or branches keyed on the MINIMUM: a spec
  // that supplied `min` and left `max` open skipped the branch entirely and left
  // the far end undefined, so every tick came out NaN and the chart drew nothing
  // but gridlines. The Segmentation histogram declares `min: 0` on both axes and
  // had been rendering blank.
  const xsAll = all.flatMap(s => s.x.filter(v => v !== null && isFinite(v)));
  const ysAll = all.flatMap(s => s.y.filter(v => v !== null && isFinite(v)));
  let x0 = xsAll.length ? Math.min(...xsAll) : 0;
  let x1 = xsAll.length ? Math.max(...xsAll) : 1;
  let y0 = ysAll.length ? Math.min(...ysAll) : 0;
  let y1 = ysAll.length ? Math.max(...ysAll) : 1;
  // A mark outside the data is still a fact about the data, so the frame
  // grows to hold it. Clipping a bound off the top draws a picture in which
  // the bound is always met.
  for (const m of spec.marks || []) {
    if (m.axis === 'x') { x0 = Math.min(x0, m.at); x1 = Math.max(x1, m.at); }
    else { y0 = Math.min(y0, m.at); y1 = Math.max(y1, m.at); }
  }
  const given = v => v !== undefined && v !== null && isFinite(v);
  if (!given(spec.y.min) || !given(spec.y.max)) {
    const pad = (y1 - y0) * 0.06 || 1;
    if (!given(spec.y.min)) y0 -= pad;
    if (!given(spec.y.max)) y1 += pad;
  }
  if (given(spec.x.min)) x0 = spec.x.min;
  if (given(spec.x.max)) x1 = spec.x.max;
  if (given(spec.y.min)) y0 = spec.y.min;
  if (given(spec.y.max)) y1 = spec.y.max;
  if (y0 === y1) { y0 -= 1; y1 += 1; }
  if (x0 === x1) { x0 -= 1; x1 += 1; }

  // A log y axis where the quantity is geometric. ap runs 0 to 400 across the
  // Kp scale in roughly equal ratios, so a linear axis spends nine tenths of
  // its height on the top two points and buries everything a design reads.
  // Zero and negatives have no place on it and are dropped to null rather than
  // clamped to the floor, which would draw them as the smallest real value.
  const lg = !!spec.y.log;
  const tl = v => (v === null || !isFinite(v) || v <= 0 ? null : Math.log10(v));
  if (lg) { y0 = Math.max(y0, 0.5); }
  const ly0 = lg ? Math.log10(y0) : y0, ly1 = lg ? Math.log10(y1) : y1;
  const px = v => L + (v - x0) / (x1 - x0) * (W - L - R);
  const py = v => {
    const t = lg ? tl(v) : v;
    if (t === null) return H - B;
    return H - B - (t - ly0) / (ly1 - ly0) * (H - B - T);
  };
  const fx = spec.x.fmt || nice, fy = spec.y.fmt || nice;

  ctx.strokeStyle = INK.grid; ctx.lineWidth = 1;
  ctx.fillStyle = INK.axis; ctx.font = '10px ui-monospace, monospace';
  const ny = spec.y.ticks || 5, nx = spec.x.ticks || 6;
  for (let i = 0; i <= ny; i++) {
    const y = T + i * (H - B - T) / ny;
    ctx.beginPath(); ctx.moveTo(L, y); ctx.lineTo(W - R, y); ctx.stroke();
    ctx.textAlign = 'right';
    const at = lg ? Math.pow(10, ly1 - i * (ly1 - ly0) / ny) : y1 - i * (y1 - y0) / ny;
    ctx.fillText(fy(at), L - 6, y + 3);
  }
  ctx.textAlign = 'center';
  for (let i = 0; i <= nx; i++) {
    const x = L + i * (W - L - R) / nx;
    ctx.beginPath(); ctx.moveTo(x, T); ctx.lineTo(x, H - B); ctx.stroke();
    ctx.fillText(fx(x0 + i * (x1 - x0) / nx), x, H - B + 15);
  }
  ctx.textAlign = 'left';

  spec.series.forEach((s, i) => {
    const col = s.colour || INK.series[i % INK.series.length];
    ctx.save();
    ctx.strokeStyle = col; ctx.fillStyle = col;
    ctx.lineWidth = s.width || 1.6;
    if (s.dash) ctx.setLineDash(s.dash);
    if (s.kind === 'dots') {
      const r = s.width || 1.6;
      for (let k = 0; k < s.x.length; k++) {
        if (s.y[k] === null) continue;
        ctx.globalAlpha = s.alpha === undefined ? 0.5 : s.alpha;
        ctx.beginPath(); ctx.arc(px(s.x[k]), py(s.y[k]), r, 0, 6.284); ctx.fill();
      }
    } else if (s.kind === 'bars') {
      const w = Math.max(1, (W - L - R) / s.x.length - 1);
      ctx.globalAlpha = s.alpha === undefined ? 0.75 : s.alpha;
      for (let k = 0; k < s.x.length; k++) {
        if (s.y[k] === null) continue;
        const h = py(y0 < 0 ? 0 : y0) - py(s.y[k]);
        ctx.fillRect(px(s.x[k]) - w / 2, py(s.y[k]), w, Math.max(1, h));
      }
    } else {
      // A null breaks the line rather than being skipped over.
      let open = false;
      ctx.beginPath();
      for (let k = 0; k < s.x.length; k++) {
        const v = s.y[k];
        if (v === null || !isFinite(v) || (lg && v <= 0)) { open = false; continue; }
        const X = px(s.x[k]), Y = py(v);
        if (!open) { ctx.moveTo(X, Y); open = true; }
        else if (s.kind === 'step') { ctx.lineTo(X, py(s.y[k - 1] === null ? v : s.y[k - 1])); ctx.lineTo(X, Y); }
        else ctx.lineTo(X, Y);
      }
      ctx.stroke();
    }
    ctx.restore();
  });

  // Marks, and their labels stacked so two nearby marks do not print on top of
  // each other. Two boundaries four Ap apart on a 275-wide axis are four pixels
  // apart, and their labels used to overlap into something unreadable — which is
  // the case where a reader most needs to know which boundary is which.
  ctx.font = '10px ui-monospace, monospace';
  const taken = { x: [], y: [] };
  /** The first row in which this label does not overlap one already drawn. */
  const slotFor = (at, w) => {
    for (let row = 0; row < 8; row++) {
      if (!taken.x.some(o => o.row === row && at < o.end && at + w > o.start)) {
        taken.x.push({ row, start: at, end: at + w });
        return row;
      }
    }
    return 0;
  };
  for (const m of spec.marks || []) {
    const col = m.colour || INK.mark;
    ctx.save();
    ctx.strokeStyle = col; ctx.fillStyle = col; ctx.lineWidth = 1.2;
    ctx.setLineDash([5, 4]);
    ctx.beginPath();
    if (m.axis === 'x') { ctx.moveTo(px(m.at), T); ctx.lineTo(px(m.at), H - B); }
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
      while (taken.y.some(o => Math.abs(o.at - y) < 11) && y < H - B - 2) y += 11;
      taken.y.push({ at: y });
      ctx.fillText(m.label, L + 4, y);
    }
  }

  ctx.fillStyle = INK.text; ctx.font = '11px ui-monospace, monospace';
  ctx.fillText(spec.y.label, L, 12);
  ctx.textAlign = 'right';
  ctx.fillText(spec.x.label, W - R, H - 6);
  ctx.textAlign = 'left';

  // Everything a hover needs to answer "what is the value here", kept on the
  // canvas so the readout cannot drift from the picture it is drawn over.
  canvas._chart = { spec, x0, x1, y0, y1, L, R, T, B, px, py, fx, fy, log: lg };

  const named = spec.series.filter(s => s.name);
  if (named.length > 1) {
    let lx = L + 6, ly = T + 12;
    ctx.font = '10px ui-monospace, monospace';
    named.forEach((s, i) => {
      const col = s.colour || INK.series[spec.series.indexOf(s) % INK.series.length];
      ctx.fillStyle = col;
      ctx.fillRect(lx, ly - 6, 14, 3);
      ctx.fillStyle = INK.text;
      ctx.fillText(s.name, lx + 19, ly);
      ly += 13;
    });
  }
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
export function attachHover(canvas, onLeave) {
  canvas.onmousemove = ev => {
    const c = canvas._chart;
    if (!c) return;
    const r = canvas.getBoundingClientRect();
    const mx = (ev.clientX - r.left) * (canvas.width / r.width);
    if (mx < c.L || mx > canvas.width - c.R) return;
    const xv = c.x0 + (mx - c.L) / (canvas.width - c.L - c.R) * (c.x1 - c.x0);
    drawChart(canvas, c.spec);
    const ctx = canvas.getContext('2d');
    ctx.save();
    ctx.strokeStyle = INK.axis; ctx.lineWidth = 1; ctx.setLineDash([2, 3]);
    ctx.beginPath(); ctx.moveTo(mx, c.T); ctx.lineTo(mx, canvas.height - c.B); ctx.stroke();
    ctx.restore();
    // The nearest actual point of each series, never an interpolation: a
    // readout that invents a value between two measurements is reporting the
    // chart's arithmetic rather than the record.
    const hits = [];
    c.spec.series.forEach((s, i) => {
      if (!s.x.length) return;
      let bi = -1, bd = Infinity;
      for (let k = 0; k < s.x.length; k++) {
        if (s.y[k] === null || !isFinite(s.y[k])) continue;
        const d = Math.abs(s.x[k] - xv);
        if (d < bd) { bd = d; bi = k; }
      }
      if (bi < 0) return;
      const col = s.colour || INK.series[i % INK.series.length];
      hits.push({ col, name: s.name, x: s.x[bi], y: s.y[bi] });
      ctx.fillStyle = col;
      ctx.beginPath(); ctx.arc(c.px(s.x[bi]), c.py(s.y[bi]), 3.2, 0, 6.284); ctx.fill();
    });
    if (!hits.length) return;
    const lines = [c.spec.x.label.split('  [')[0] + ' ' + c.fx(hits[0].x)]
      .concat(hits.map(h => (h.name ? h.name + ': ' : '') + c.fy(h.y)));
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
  };
  canvas.onmouseleave = () => {
    if (canvas._chart) drawChart(canvas, canvas._chart.spec);
    if (onLeave) onLeave();
  };
}
