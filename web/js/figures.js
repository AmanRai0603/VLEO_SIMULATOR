/*
  The figure player: any figure the engine describes, drawn.

  THE ENGINE SAYS WHAT THERE IS TO SEE; THIS ONLY DRAWS IT. A figure arrives as
  the description in contract/schemas/result.json (`figures`) — its kind, its
  axes and units, every series and the row it shows, and every point that
  could not be computed and why. Six kinds, one entry point:

    line, scatter, bar   the chart every panel uses (chart.js, figureSpec)
    heatmap              a value over two inputs, in the validated ordinal ramp
    animation            a line figure in frames: play, pause, scrub
    scene3d              bodies and paths, turned by dragging

  Every kind offers the same three things beside the picture: PNG of exactly
  what is on screen, CSV of the numbers behind it, and the numbers as a table.
  A kind this file does not know is said, never drawn as something else.

  No library: the 3D view is an orthographic projection on a 2D canvas. It is
  enough to see where things are; richer 3D grows in the same format.
*/
'use strict';

import { esc, plural } from './dom.js';
import { INK, drawChart, attachHover, figureSpec, sizeCanvas, cssSize, saveFile, watchScheme } from './chart.js';

const W = 900, H = 360;

/** Draw figure `f` into `host`, with its export strip. `stem` names the files. */
export function drawFigureInto(host, f, stem) {
  const base = String(stem || f.id || 'figure').replace(/[^\w.-]+/g, '_');
  const draw = { line: chartKind, scatter: chartKind, bar: chartKind, heatmap, animation, scene3d }[f.kind];
  host.innerHTML = '<p class="muted fig-says">' + esc(f.says || f.title || '') + '</p>' +
    (draw ? '<canvas class="plot fig-canvas" width="' + W + '" height="' + H + '"></canvas>' +
      '<div class="runbar fig-ctl"></div>' +
      '<div class="runbar"><button class="ctl fig-png" type="button">PNG</button>' +
      '<button class="ctl fig-csv" type="button">CSV</button><span class="muted fig-hint"></span></div>' +
      '<details><summary>the numbers behind this picture</summary><div class="fig-table"></div></details>'
      : '<p class="run-stale">A <b>' + esc(f.kind) + '</b> figure, which this page does not know how to draw. ' +
        'Its numbers are in the result.</p>');
  if (!draw) return;
  const c = host.querySelector('.fig-canvas');
  const fig = { f, c, host, ctl: host.querySelector('.fig-ctl'), hint: host.querySelector('.fig-hint') };
  const d = draw(fig);
  const paint = () => { if (c.isConnected) d.paint(); };
  paint();
  watchScheme(paint);
  host.querySelector('.fig-table').innerHTML = gridTable(d.grid());
  host.querySelector('.fig-png').onclick = () =>
    c.toBlob(b => { if (b) saveFile(base + '.png', b); }, 'image/png');
  host.querySelector('.fig-csv').onclick = () => saveFile(base + '.csv', gridCsv(d.grid()), 'text/csv');
}

const unit = a => (a.label || a.id) + '  [' + a.unit + ']';

// ── line, scatter, bar ─────────────────────────────────────────────────────

function chartKind({ f, c, hint }) {
  let zoom = null;
  const spec = () => figureSpec(f, { zoom });
  hint.textContent = 'hover to read a point, drag to zoom, double-click to undo';
  return {
    paint() {
      drawChart(c, spec());
      attachHover(c, {
        onBrush: w => { if (w.x) { zoom = w.x; this.paint(); } },
        onReset: () => { if (zoom) { zoom = null; this.paint(); } },
      });
    },
    grid() {
      const head = [unit(f.x)].concat(f.series.map(s => s.name + '  [' + f.y.unit + ']'));
      const xs = [...new Set(f.series.flatMap(s => s.x))].sort((a, b) => a - b);
      return { head, rows: xs.map(x => [x].concat(f.series.map(s => {
        const i = s.x.indexOf(x);
        return i < 0 ? '' : s.y[i];
      }))) };
    },
  };
}

// ── heatmap ────────────────────────────────────────────────────────────────

/**
 * A value over two inputs. Coloured in FIVE CLASSES of the validated ordinal
 * ramp, not a continuous blend: the ramp was checked as five steps a reader can
 * tell apart, and a blend between them is a colour nobody checked. A cell that
 * could not be computed is left open and hatched, and the legend says so.
 */
function heatmap({ f, c, hint }) {
  const g = f.grid, nx = g.x.length, ny = g.y.length;
  const vals = g.z.filter(v => v !== null && Number.isFinite(v));
  const lo = Math.min(...vals), hi = Math.max(...vals);
  const cls = v => v === null ? -1 : hi === lo ? 2 : Math.min(4, Math.floor((v - lo) / (hi - lo) * 5));
  const M = { l: 70, r: 150, t: 16, b: 46 };
  const edges = a => a.map((v, i) => [
    i === 0 ? v - ((a[1] ?? v + 1) - v) / 2 : (a[i - 1] + v) / 2,
    i === a.length - 1 ? v + (v - (a[i - 1] ?? v - 1)) / 2 : (v + a[i + 1]) / 2]);
  const ex = edges(g.x), ey = edges(g.y);
  let at = null;
  hint.textContent = 'hover a cell to read it';
  const d = {
    paint() {
      const ctx = c.getContext('2d');
      const { w, h } = cssSize(c);
      sizeCanvas(c, w, h);
      const dpr = c.width / w;
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      ctx.fillStyle = INK.surface; ctx.fillRect(0, 0, w, h);
      const pw = w - M.l - M.r, ph = h - M.t - M.b;
      const x0 = ex[0][0], x1 = ex[nx - 1][1], y0 = ey[0][0], y1 = ey[ny - 1][1];
      const px = x => M.l + (x - x0) / (x1 - x0) * pw, py = y => M.t + ph - (y - y0) / (y1 - y0) * ph;
      // The ramp's last step is its emphasised end in both schemes: the high values.
      const ramp = INK.ramp5;
      for (let j = 0; j < ny; j++) for (let i = 0; i < nx; i++) {
        const v = g.z[j * nx + i], k = cls(v);
        const X = px(ex[i][0]), Y = py(ey[j][1]), CW = px(ex[i][1]) - X, CH = py(ey[j][0]) - Y;
        if (k < 0) {
          ctx.strokeStyle = INK.muted; ctx.lineWidth = 1;
          ctx.save(); ctx.beginPath(); ctx.rect(X, Y, CW, CH); ctx.clip();
          for (let s = -CH; s < CW; s += 6) { ctx.beginPath(); ctx.moveTo(X + s, Y + CH); ctx.lineTo(X + s + CH, Y); ctx.stroke(); }
          ctx.restore();
        } else {
          ctx.fillStyle = ramp[k]; ctx.fillRect(X, Y, CW + 0.5, CH + 0.5);
        }
        if (at && at[0] === i && at[1] === j) {
          ctx.strokeStyle = INK.text; ctx.lineWidth = 2; ctx.strokeRect(X + 1, Y + 1, CW - 2, CH - 2);
        }
      }
      ctx.fillStyle = INK.text2; ctx.font = '11px ui-monospace, monospace'; ctx.textAlign = 'center';
      const step = Math.ceil(nx / 10);
      g.x.forEach((x, i) => { if (i % step === 0) ctx.fillText(num(x), px(x), M.t + ph + 16); });
      ctx.fillText(unit(f.x), M.l + pw / 2, h - 8);
      ctx.textAlign = 'right';
      const sy = Math.ceil(ny / 8);
      g.y.forEach((y, j) => { if (j % sy === 0) ctx.fillText(num(y), M.l - 6, py(y) + 4); });
      ctx.save(); ctx.translate(14, M.t + ph / 2); ctx.rotate(-Math.PI / 2); ctx.textAlign = 'center';
      ctx.fillText(unit(f.y), 0, 0); ctx.restore();
      // The legend: five classes and what an open cell means.
      ctx.textAlign = 'left';
      const lx = M.l + pw + 18;
      ctx.fillStyle = INK.text; ctx.fillText(unit(f.z), lx, M.t + 10);
      for (let k = 4; k >= 0; k--) {
        const yy = M.t + 22 + (4 - k) * 22;
        ctx.fillStyle = ramp[k]; ctx.fillRect(lx, yy, 16, 16);
        ctx.fillStyle = INK.text2;
        ctx.fillText(num(lo + (hi - lo) * k / 5) + ' – ' + num(lo + (hi - lo) * (k + 1) / 5), lx + 22, yy + 12);
      }
      if (g.z.some(v => v === null)) {
        const yy = M.t + 22 + 5 * 22;
        ctx.strokeStyle = INK.muted; ctx.strokeRect(lx + 0.5, yy + 0.5, 15, 15);
        ctx.fillStyle = INK.text2; ctx.fillText('not computed', lx + 22, yy + 12);
      }
      c._hm = { px, py, pw, ph };
    },
    grid() {
      const rows = [];
      for (let j = 0; j < ny; j++) for (let i = 0; i < nx; i++) rows.push([g.x[i], g.y[j], g.z[j * nx + i] ?? '']);
      return { head: [unit(f.x), unit(f.y), unit(f.z)], rows };
    },
  };
  c.onmousemove = e => {
    const r = c.getBoundingClientRect(), mx = e.clientX - r.left, my = e.clientY - r.top;
    const i = ex.findIndex(([a, b]) => mx >= c._hm.px(a) && mx < c._hm.px(b));
    const j = ey.findIndex(([a, b]) => my <= c._hm.py(a) && my > c._hm.py(b));
    const next = i >= 0 && j >= 0 ? [i, j] : null;
    if (String(next) === String(at)) return;
    at = next; d.paint();
    hint.textContent = at ? f.x.id + ' = ' + num(g.x[i]) + ', ' + f.y.id + ' = ' + num(g.y[j]) + ': ' +
      (g.z[j * nx + i] === null ? 'not computed' : f.z.id + ' = ' + num(g.z[j * nx + i]) + ' ' + f.z.unit)
      : 'hover a cell to read it';
  };
  return d;
}

// ── animation ──────────────────────────────────────────────────────────────

/**
 * A line figure in frames. THE AXES ARE THE UNION OF EVERY FRAME, fixed: an
 * axis that rescales each frame makes a curve that did not move look as if it
 * had, and hides the one that did.
 */
function animation({ f, c, ctl, hint }) {
  const all = f.frames.flatMap(fr => fr.series);
  const xs = all.flatMap(s => s.x), ys = all.flatMap(s => s.y).filter(v => v !== null);
  const pad = (Math.max(...ys) - Math.min(...ys)) * 0.05 || 1;
  const lim = { x: [Math.min(...xs), Math.max(...xs)], y: [Math.min(...ys) - pad, Math.max(...ys) + pad] };
  let k = 0, timer = null;
  ctl.innerHTML = '<button class="ctl fig-play" type="button">play</button>' +
    '<input class="fig-scrub" type="range" min="0" max="' + (f.frames.length - 1) + '" value="0" step="1">' +
    '<span class="fig-t"></span>';
  const play = ctl.querySelector('.fig-play'), scrub = ctl.querySelector('.fig-scrub');
  const stop = () => { clearInterval(timer); timer = null; play.textContent = 'play'; };
  const d = {
    paint() {
      const fr = f.frames[k];
      const spec = figureSpec({ ...f, kind: 'line', series: fr.series, notes: [] });
      spec.x.min = lim.x[0]; spec.x.max = lim.x[1];
      spec.y.min = lim.y[0]; spec.y.max = lim.y[1];
      drawChart(c, spec);
      ctl.querySelector('.fig-t').textContent = (f.z.label || f.z.id) + ' = ' + num(fr.t) + ' ' + f.z.unit +
        ' · frame ' + (k + 1) + ' of ' + f.frames.length;
      scrub.value = k;
    },
    grid() {
      const rows = [];
      f.frames.forEach(fr => fr.series.forEach(s => s.x.forEach((x, i) => rows.push([fr.t, s.name, x, s.y[i] ?? '']))));
      return { head: [unit(f.z), 'series', unit(f.x), unit(f.y)], rows };
    },
  };
  play.onclick = () => {
    if (timer) { stop(); return; }
    play.textContent = 'pause';
    timer = setInterval(() => {
      if (!c.isConnected) { stop(); return; }
      k = (k + 1) % f.frames.length; d.paint();
    }, 700);
  };
  scrub.oninput = () => { stop(); k = +scrub.value; d.paint(); };
  hint.textContent = plural(f.frames.length, 'frame') + '; the PNG is the frame on screen';
  return d;
}

// ── 3D scene ───────────────────────────────────────────────────────────────

/**
 * Bodies and paths, seen orthographically and turned by dragging. The same
 * scale on all three axes, always: a 3D view that stretches one axis to fill
 * the frame shows an orbit that is not the orbit.
 */
function scene3d({ f, c, hint }) {
  const pts = f.scene.flatMap(b => b.points);
  const ctr = [0, 1, 2].map(a => (Math.min(...pts.map(p => p[a])) + Math.max(...pts.map(p => p[a]))) / 2);
  const rad = Math.max(1e-12, ...pts.map(p => Math.hypot(p[0] - ctr[0], p[1] - ctr[1], p[2] - ctr[2])));
  const home = { yaw: -0.6, pitch: 0.45 };
  let view = { ...home }, drag = null;
  const project = (p, w, h) => {
    const [x, y, z] = [p[0] - ctr[0], p[1] - ctr[1], p[2] - ctr[2]];
    const cy = Math.cos(view.yaw), sy = Math.sin(view.yaw), cp = Math.cos(view.pitch), sp = Math.sin(view.pitch);
    const X = x * cy - y * sy, Y = x * sy + y * cy;
    const s = Math.min(w, h) * 0.42 / rad;
    return [w / 2 + X * s, h / 2 - (z * cp - Y * sp) * s, Y * cp + z * sp];
  };
  hint.textContent = 'drag to turn, double-click to go back; axes in ' + f.x.unit;
  const d = {
    paint() {
      const ctx = c.getContext('2d');
      const { w, h } = cssSize(c);
      sizeCanvas(c, w, h);
      const dpr = c.width / w;
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      ctx.fillStyle = INK.surface; ctx.fillRect(0, 0, w, h);
      ctx.font = '11px ui-monospace, monospace';
      // The axes, as a small triad in the corner: which way x, y and z point now.
      const o = [48, h - 40];
      [[1, 0, 0, f.x.id], [0, 1, 0, f.y.id], [0, 0, 1, f.z.id]].forEach(([a, b, z, name]) => {
        const p0 = project(ctr, 100, 100), p1 = project([ctr[0] + a * rad, ctr[1] + b * rad, ctr[2] + z * rad], 100, 100);
        const dx = (p1[0] - p0[0]) * 0.6, dy = (p1[1] - p0[1]) * 0.6;
        ctx.strokeStyle = INK.axis; ctx.lineWidth = 1.2;
        ctx.beginPath(); ctx.moveTo(o[0], o[1]); ctx.lineTo(o[0] + dx, o[1] + dy); ctx.stroke();
        ctx.fillStyle = INK.text2; ctx.fillText(name, o[0] + dx * 1.2 - 4, o[1] + dy * 1.2 + 4);
      });
      const order = f.scene.map((b, i) => ({ b, i, z: Math.min(...b.points.map(p => project(p, w, h)[2])) }))
        .sort((a, b) => a.z - b.z);
      for (const { b, i } of order) {
        const col = INK.series[i % INK.series.length];
        const q = b.points.map(p => project(p, w, h));
        ctx.strokeStyle = col; ctx.fillStyle = col; ctx.lineWidth = 2;
        if (b.shape === 'path') {
          ctx.beginPath(); q.forEach(([x, y], n) => n ? ctx.lineTo(x, y) : ctx.moveTo(x, y)); ctx.stroke();
        } else {
          ctx.beginPath(); ctx.arc(q[0][0], q[0][1], 5, 0, 2 * Math.PI); ctx.fill();
        }
        ctx.fillStyle = INK.text; ctx.fillText(b.name, q[0][0] + 8, q[0][1] - 6);
      }
    },
    grid() {
      const rows = [];
      f.scene.forEach(b => b.points.forEach((p, n) => rows.push([b.name, b.shape, n, p[0], p[1], p[2]])));
      return { head: ['body', 'shape', 'point', unit(f.x), unit(f.y), unit(f.z)], rows };
    },
  };
  c.onmousedown = e => { drag = [e.clientX, e.clientY, view.yaw, view.pitch]; };
  c.onmousemove = e => {
    if (!drag) return;
    view.yaw = drag[2] + (e.clientX - drag[0]) * 0.01;
    view.pitch = Math.max(-1.5, Math.min(1.5, drag[3] + (e.clientY - drag[1]) * 0.01));
    d.paint();
  };
  c.onmouseup = c.onmouseleave = () => { drag = null; };
  c.ondblclick = () => { view = { ...home }; d.paint(); };
  return d;
}

// ── the numbers ────────────────────────────────────────────────────────────

function num(v) {
  if (typeof v !== 'number') return String(v);
  const a = Math.abs(v);
  if (a === 0) return '0';
  if (a >= 1e5 || a < 1e-3) return v.toExponential(2).replace('e+', 'e');
  return String(+v.toPrecision(4));
}

function gridTable(g) {
  return '<div class="ri-wrap"><table class="fx"><thead><tr>' + g.head.map(h => '<th>' + esc(h) + '</th>').join('') +
    '</tr></thead><tbody>' + g.rows.slice(0, 2000).map(r => '<tr>' + r.map(v => '<td>' + esc(v === '' ? '—' : num(v)) +
    '</td>').join('') + '</tr>').join('') + '</tbody></table></div>' +
    (g.rows.length > 2000 ? '<p class="muted">the first 2000 of ' + g.rows.length + ' rows; the CSV has all of them</p>' : '');
}

function gridCsv(g) {
  const cell = v => /[",\n]/.test(String(v)) ? '"' + String(v).replace(/"/g, '""') + '"' : String(v);
  return [g.head].concat(g.rows).map(r => r.map(cell).join(',')).join('\n') + '\n';
}
