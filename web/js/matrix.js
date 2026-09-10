/*
  The matrix.

  Rows and columns are the display list in tree order, and the hierarchy is
  drawn inside the grid as nested boxes on the diagonal. A flat 1333x1333 grid
  is 1.8 million cells of which 0.03% are filled and is useless; a rollup to
  eight branches is one cell in sixty-four and is useless for the opposite
  reason. The nested form keeps every subtree a contiguous band, so a mark
  inside a box is coupling that subtree owns and a mark outside one crosses a
  boundary that needs an interface.

  Only the cells that carry something are in the document. The empty ones are
  paper, and paper does not need a div.
*/
'use strict';

import { $, esc } from './dom.js';
import { S, cell, isSeeded } from './state.js';

export function drawMatrix(disp, rel) {
  const C = cell();
  const n = disp.length;
  const inner = $('#matrix-inner');
  const cells = $('#matrix-cells');
  const svg = $('#matrix-svg');
  inner.style.width = cells.style.width = (n * C) + 'px';
  inner.style.height = cells.style.height = (n * C) + 'px';

  // Which display row carries each node. A node inside a closed box is carried
  // by the box, which is what makes the rollup exact.
  const owner = new Map();
  disp.forEach((d, i) => { for (const m of d.members) owner.set(m, i); });

  const marks = new Set();
  for (let r = 0; r < n; r++) {
    for (const m of disp[r].members) {
      for (const c of S.consumers[m]) {
        const j = owner.get(c);
        if (j == null || j === r) continue;
        marks.add(r + ':' + j);
      }
    }
  }

  const k = S.dispIndex.get(S.selected);
  const out = [];

  // The bands: the selected row and the selected column, so the eye can travel
  // from a mark to the node it names without counting.
  if (k != null) {
    out.push('<div class="band rband" style="top:' + (k * C) + 'px;height:' + C + 'px;width:' + (n * C) + 'px"></div>');
    out.push('<div class="band cband" style="left:' + (k * C) + 'px;width:' + C + 'px;height:' + (n * C) + 'px"></div>');
  }

  let boxes = 0;
  for (let i = n - 1; i >= 0; i--) {
    let end = i;
    for (let j = i + 1; j < n; j++) { if (disp[j].depth > disp[i].depth) end = j; else break; }
    if (disp[i].kind === 'group' && disp[i].open && end > i) {
      boxes++;
      const x = i * C, w = (end - i + 1) * C;
      out.push('<div class="boxrect" style="left:' + x + 'px;top:' + x + 'px;width:' + w + 'px;height:' + w + 'px"></div>');
      out.push('<div class="boxlabel" style="left:' + (x + C + 3) + 'px;top:' + (x + 3) + 'px">' +
        esc(disp[i].label) + '</div>');
    }
  }

  for (let i = 0; i < n; i++) {
    const d = disp[i];
    const seeded = d.kind === 'node' && isSeeded(d.row);
    out.push('<div class="mcell diag' + (d.id === S.selected ? ' sel' : '') + (seeded ? ' seeded' : '') +
      '" data-open="' + esc(d.id) + '" data-i="' + i + '" title="' + esc(d.label) +
      ' — click to open" style="left:' + (i * C) + 'px;top:' + (i * C) + 'px">' +
      '<span class="num">' + (i + 1) + '</span></div>');
  }

  let crossing = 0, below = 0;
  for (const key of marks) {
    const [r, c] = key.split(':').map(Number);
    const inside = disp[r].parent === disp[c].parent;
    if (!inside) crossing++;
    if (c < r) below++;
    let cls = 'mk ' + (inside ? 'inside' : 'crosses');
    if (r === k) cls = 'mk rowsel';
    if (c === k) cls = 'mk colsel';
    if (c < r) cls += ' below';
    out.push('<div class="mcell" style="left:' + (c * C) + 'px;top:' + (r * C) + 'px" title="' +
      esc(disp[r].label) + ' feeds ' + esc(disp[c].label) + '"><i class="' + cls + '"></i></div>');
  }

  cells.innerHTML = out.join('');
  S.matrixStats = { marks: marks.size, crossing, below, boxes };

  // The routed arrows: out along the selected row, then down the far column
  // into that node's own square. Two straight legs, because a curve across a
  // grid cannot be followed back to the cell it came from.
  svg.setAttribute('width', n * C);
  svg.setAttribute('height', n * C);
  if (k == null) { svg.innerHTML = ''; return; }
  const mid = i => i * C + C / 2;
  const legs = [];
  for (const key of marks) {
    const [r, c] = key.split(':').map(Number);
    if (r === k) legs.push(leg(mid(k), mid(k), mid(c), mid(k), mid(c), mid(c), 'out'));
    else if (c === k) legs.push(leg(mid(r), mid(r), mid(k), mid(r), mid(k), mid(k), 'in'));
  }
  svg.innerHTML = defs() + legs.join('');
}

const defs = () =>
  '<defs><marker id="ah-out" viewBox="0 0 8 8" refX="6" refY="4" markerWidth="5" markerHeight="5" orient="auto">' +
  '<path d="M0 0 L8 4 L0 8 z" fill="var(--out)"/></marker>' +
  '<marker id="ah-in" viewBox="0 0 8 8" refX="6" refY="4" markerWidth="5" markerHeight="5" orient="auto">' +
  '<path d="M0 0 L8 4 L0 8 z" fill="var(--in)"/></marker></defs>';

function leg(x0, y0, x1, y1, x2, y2, dir) {
  const col = dir === 'out' ? 'var(--out)' : 'var(--in)';
  return '<polyline points="' + [x0, y0, x1, y1, x2, y2].map(v => v.toFixed(1)).join(' ') +
    '" fill="none" stroke="' + col + '" stroke-width="1.2" opacity=".85" marker-end="url(#ah-' + dir + ')"/>';
}
