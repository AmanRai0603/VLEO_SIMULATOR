/*
  The pictures a group folder draws from its own tables.

  A CHART is handed to the tool's own figure player (figures.js), so a group
  sees its data exactly as the application will show it: the same axes, the
  same colours, the same hover, zoom, PNG and CSV. This file only turns a
  CSV and its row of figures.csv into the figure description that player
  reads.

  A FLOW is a table of arrows (from, to, label) and is laid out here: boxes
  in columns by how far along the flow they sit, arrows between them. STEPS
  walk through a flow one row at a time — what to light up and what to say —
  which is how a group animates an explanation without writing any code.
  The GROUP'S WIRING is the same drawing, with the arrows taken from every
  node's inputs.csv.

  Everything is SVG built from escaped text, coloured by the tool's tokens,
  so it follows light and dark and prints.
*/
'use strict';

import { esc } from './dom.js';
import { parseCsv, splitUnit, column, num } from './csv.js';
import { layers, topoOrder } from './gmodel.js';

// ── charts, for the figure player ──────────────────────────────────────────

const axis = (head, name) => {
  const k = head.findIndex(h => splitUnit(h).name === name);
  const { unit } = splitUnit(head[k] || name);
  return { id: name, label: name, unit: unit || '1' };
};
const col = (t, name) => {
  const k = column(t, name);
  return t.rows.map(r => num(r[k]));
};

/** The figure description figures.js draws, from a CSV and its figures.csv row. */
export function figureFromCsv(fig, text) {
  const t = parseCsv(text);
  const base = { id: fig.id, title: fig.title, says: fig.title, notes: [], gaps: [] };
  const ys = String(fig.y || '').split(/\s+/).filter(Boolean);
  if (['line', 'scatter', 'bar'].includes(fig.kind)) {
    const x = col(t, fig.x);
    return {
      ...base, kind: fig.kind, x: axis(t.head, fig.x), y: axis(t.head, ys[0]),
      series: ys.map(y => {
        const v = col(t, y);
        const pts = x.map((xv, i) => [xv, v[i]]).filter(p => p[0] !== null && p[1] !== null);
        return { name: y, x: pts.map(p => p[0]), y: pts.map(p => p[1]) };
      }),
    };
  }
  if (fig.kind === 'heatmap') {
    const x = col(t, fig.x), y = col(t, ys[0]), z = col(t, fig.z);
    const xs = [...new Set(x.filter(v => v !== null))].sort((a, b) => a - b);
    const yy = [...new Set(y.filter(v => v !== null))].sort((a, b) => a - b);
    const grid = new Array(xs.length * yy.length).fill(null);
    x.forEach((xv, i) => {
      const a = xs.indexOf(xv), b = yy.indexOf(y[i]);
      if (a >= 0 && b >= 0) grid[b * xs.length + a] = z[i];
    });
    return { ...base, kind: 'heatmap', x: axis(t.head, fig.x), y: axis(t.head, ys[0]), z: axis(t.head, fig.z), grid: { x: xs, y: yy, z: grid } };
  }
  if (fig.kind === 'animation') {
    const x = col(t, fig.x), fr = col(t, fig.z);
    const ts = [...new Set(fr.filter(v => v !== null))].sort((a, b) => a - b);
    return {
      ...base, kind: 'animation', x: axis(t.head, fig.x), y: axis(t.head, ys[0]), z: axis(t.head, fig.z),
      frames: ts.map(tv => ({
        t: tv,
        series: ys.map(y => {
          const v = col(t, y);
          const idx = fr.map((f, i) => f === tv ? i : -1).filter(i => i >= 0);
          return { name: y, x: idx.map(i => x[i]), y: idx.map(i => v[i]) };
        }),
      })),
    };
  }
  if (fig.kind === 'scene3d') {
    const kb = column(t, 'body'), ks = column(t, 'shape');
    const X = col(t, 'x'), Y = col(t, 'y'), Z = col(t, 'z');
    const bodies = new Map();
    t.rows.forEach((r, i) => {
      const name = r[kb] || 'body';
      if (!bodies.has(name)) bodies.set(name, { name, shape: ks >= 0 && r[ks] ? r[ks] : 'point', points: [] });
      if (X[i] !== null && Y[i] !== null && Z[i] !== null) bodies.get(name).points.push([X[i], Y[i], Z[i]]);
    });
    return { ...base, kind: 'scene3d', x: axis(t.head, 'x'), y: axis(t.head, 'y'), z: axis(t.head, 'z'), scene: [...bodies.values()].filter(b => b.points.length) };
  }
  return null;
}

// ── flows ──────────────────────────────────────────────────────────────────

/**
 * Boxes and arrows from a table: `from, to, label`; a row with no `to` gives a
 * box its caption. Answers `{ boxes: [{id, caption}], arrows: [{from, to, label}] }`.
 */
export function flowFromCsv(text) {
  const t = parseCsv(text);
  const kf = column(t, 'from'), kt = column(t, 'to'), kl = column(t, 'label');
  const boxes = new Map(), arrows = [];
  const box = id => { if (!boxes.has(id)) boxes.set(id, { id, caption: '' }); return boxes.get(id); };
  t.rows.forEach(r => {
    const from = r[kf], to = kt >= 0 ? r[kt] : '', label = kl >= 0 ? r[kl] : '';
    if (!from) return;
    if (!to) { box(from).caption = label; return; }
    box(from); box(to);
    arrows.push({ from, to, label });
  });
  return { boxes: [...boxes.values()], arrows };
}

/**
 * SVG of boxes in columns, arrows between them. `opts.kind(id)` may give a box
 * a class; `opts.sub(id)` a second line; `opts.link(id)` makes it clickable.
 */
export function flowSvg(flow, opts = {}) {
  const ids = flow.boxes.map(b => b.id);
  const edges = flow.arrows.map(a => ({ from: a.from, to: a.to }));
  const order = topoOrder(ids, edges);
  const L = layers(order, edges);
  const cols = new Map();
  for (const id of order) { const l = L.get(id) || 0; if (!cols.has(l)) cols.set(l, []); cols.get(l).push(id); }
  const labelled = flow.arrows.some(a => a.label);
  const BW = opts.boxWidth || 172, BH = opts.sub ? 50 : 40, GX = opts.gapX || (labelled ? 120 : 64), GY = 18, PAD = 12;
  const ncol = Math.max(1, cols.size), nrow = Math.max(1, ...[...cols.values()].map(c => c.length));
  const W = PAD * 2 + ncol * BW + (ncol - 1) * GX, H = PAD * 2 + nrow * BH + (nrow - 1) * GY + 8;
  const pos = new Map();
  [...cols.keys()].sort((a, b) => a - b).forEach((l, ci) => {
    const list = cols.get(l);
    const top = PAD + (H - PAD * 2 - (list.length * BH + (list.length - 1) * GY)) / 2;
    list.forEach((id, ri) => pos.set(id, { x: PAD + ci * (BW + GX), y: top + ri * (BH + GY) }));
  });
  const cap = new Map(flow.boxes.map(b => [b.id, b.caption]));
  const trim = (s, n) => (s.length > n ? s.slice(0, n - 1) + '…' : s);
  let svg = '<svg class="gflow" viewBox="0 0 ' + W + ' ' + H + '" width="' + W + '" role="img" aria-label="' +
    esc(opts.label || 'a flow diagram') + '"><defs><marker id="gah" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="gah"/></marker></defs>';
  flow.arrows.forEach((a, k) => {
    const p = pos.get(a.from), q = pos.get(a.to);
    if (!p || !q) return;
    const x1 = p.x + BW, y1 = p.y + BH / 2, x2 = q.x, y2 = q.y + BH / 2;
    const back = x2 <= x1;
    const d = back
      ? 'M' + (p.x + BW / 2) + ',' + (p.y + BH) + ' C' + (p.x + BW / 2) + ',' + (H) + ' ' + (q.x + BW / 2) + ',' + H + ' ' + (q.x + BW / 2) + ',' + (q.y + BH)
      : 'M' + x1 + ',' + y1 + ' C' + (x1 + GX / 2) + ',' + y1 + ' ' + (x2 - GX / 2) + ',' + y2 + ' ' + x2 + ',' + y2;
    svg += '<path class="garrow" data-from="' + esc(a.from) + '" data-to="' + esc(a.to) + '" data-k="' + k + '" d="' + d + '" marker-end="url(#gah)"/>';
    if (a.label) {
      // Near the arrow's head, where arrows leaving one box have spread apart.
      const mx = back ? (p.x + q.x + BW) / 2 : x1 + (x2 - x1) * 0.62, my = back ? H - 4 : y1 + (y2 - y1) * 0.9 - 6;
      svg += '<text class="glabel" x="' + mx + '" y="' + my + '" text-anchor="middle">' + esc(trim(a.label, 20)) + '</text>';
    }
  });
  for (const id of order) {
    const p = pos.get(id);
    const cls = 'gbox' + (opts.kind ? ' ' + opts.kind(id) : '') + (opts.link ? ' glink' : '');
    const title = cap.get(id) || (opts.title ? opts.title(id) : '');
    svg += '<g class="' + cls + '" data-id="' + esc(id) + '" transform="translate(' + p.x + ',' + p.y + ')"' +
      (opts.link ? ' tabindex="0" role="link"' : '') + '><title>' + esc(title || id) + '</title>' +
      '<rect width="' + BW + '" height="' + BH + '" rx="5"/>' +
      '<text class="gt1" x="10" y="' + (opts.sub ? 19 : 24) + '">' + esc(trim(opts.name ? opts.name(id) : (cap.get(id) || id), 24)) + '</text>' +
      (opts.sub ? '<text class="gt2" x="10" y="37">' + esc(trim(opts.sub(id) || '', 28)) + '</text>' : '') + '</g>';
  }
  return svg + '</svg>';
}

/** A drawn flow's boxes: the short id large, the caption beneath it. */
export function captioned(flow, opts = {}) {
  const cap = new Map(flow.boxes.map(b => [b.id, b.caption]));
  return { ...opts, name: id => id.replace(/_/g, ' '), sub: id => cap.get(id) || '', title: id => cap.get(id) || id };
}

/** Light up the boxes and arrows a step names: `a b` boxes, `a>b` an arrow. */
export function highlight(svgHost, spec) {
  const want = String(spec || '').split(/\s+/).filter(Boolean);
  const boxes = new Set(want.filter(w => !w.includes('>')));
  const arrows = new Set(want.filter(w => w.includes('>')));
  svgHost.querySelectorAll('.gbox').forEach(g => g.classList.toggle('on', boxes.has(g.dataset.id)));
  svgHost.querySelectorAll('.garrow').forEach(p => {
    const on = arrows.has(p.dataset.from + '>' + p.dataset.to) || (boxes.has(p.dataset.from) && boxes.has(p.dataset.to));
    p.classList.toggle('on', on);
  });
  svgHost.classList.toggle('lit', want.length > 0);
}

/**
 * A step-by-step walk through a flow: play, pause, back, forward, a scrubber,
 * and the caption of the step on show. `steps` is the table: step, caption,
 * highlight. Motion only when the reader presses play.
 */
export function mountSteps(host, flow, stepsText, opts = {}) {
  const t = parseCsv(stepsText);
  const kc = column(t, 'caption'), kh = column(t, 'highlight'), ks = column(t, 'step');
  const steps = t.rows.map(r => ({ n: r[ks], caption: r[kc] || '', on: kh >= 0 ? r[kh] : '' }));
  host.innerHTML = '<div class="gsteps-pic">' + flowSvg(flow, captioned(flow, opts)) + '</div>' +
    '<div class="runbar gsteps-ctl"><button class="ctl gs-play" type="button">▶ play</button>' +
    '<button class="ctl gs-prev" type="button">← back</button><button class="ctl gs-next" type="button">next →</button>' +
    '<input class="gs-scrub" type="range" min="0" max="' + Math.max(0, steps.length - 1) + '" value="0" aria-label="step">' +
    '<span class="chip gs-n"></span></div><p class="gsteps-cap" aria-live="polite"></p>';
  const pic = host.querySelector('.gsteps-pic'), cap = host.querySelector('.gsteps-cap');
  const scrub = host.querySelector('.gs-scrub'), play = host.querySelector('.gs-play');
  let k = 0, timer = null;
  const show = i => {
    k = Math.max(0, Math.min(steps.length - 1, i));
    const s = steps[k];
    if (!s) return;
    highlight(pic, s.on);
    cap.textContent = s.caption;
    scrub.value = String(k);
    host.querySelector('.gs-n').textContent = (k + 1) + ' / ' + steps.length;
  };
  const stop = () => { clearInterval(timer); timer = null; play.textContent = '▶ play'; };
  play.onclick = () => {
    if (timer) return stop();
    if (k >= steps.length - 1) show(0);
    play.textContent = '❚❚ pause';
    timer = setInterval(() => { if (k >= steps.length - 1) return stop(); show(k + 1); }, opts.ms || 2200);
  };
  host.querySelector('.gs-prev').onclick = () => { stop(); show(k - 1); };
  host.querySelector('.gs-next').onclick = () => { stop(); show(k + 1); };
  scrub.oninput = () => { stop(); show(+scrub.value); };
  if (steps.length) show(0); else cap.textContent = 'This walk-through has no steps yet.';
}

/** The group's wiring as a flow: nodes, the inputs from other groups, and what is sent on. */
export function wiringFlow(model) {
  const boxes = [...model.order].map(id => ({ id, caption: model.nodes.get(id).row.question || '' }));
  const arrows = model.edges.map(e => ({ from: e.from, to: e.to, label: e.input }));
  const ext = new Set();
  for (const e of model.external) {
    if (!ext.has(e.from)) { ext.add(e.from); boxes.unshift({ id: e.from, caption: 'from another group' }); }
    arrows.push({ from: e.from, to: e.to, label: e.input });
  }
  for (const p of model.group.publishes) {
    const out = '→ ' + (p.to || 'up');
    if (!boxes.some(b => b.id === out)) boxes.push({ id: out, caption: p.says || 'sent on' });
    arrows.push({ from: p.node, to: out, label: '' });
  }
  return { boxes, arrows };
}

/** Upstream and downstream of a node, through the derived wiring. */
export function reach(model, id) {
  const up = new Set(), down = new Set();
  const walk = (set, start, fwd) => {
    const stack = [start];
    while (stack.length) {
      const c = stack.pop();
      for (const e of model.edges) {
        const [a, b] = fwd ? [e.from, e.to] : [e.to, e.from];
        if (a === c && !set.has(b)) { set.add(b); stack.push(b); }
      }
    }
  };
  walk(up, id, false); walk(down, id, true);
  return { up, down };
}
