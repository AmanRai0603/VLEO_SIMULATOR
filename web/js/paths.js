/*
  The paths column.

  It numbers every row and draws one arc per direct neighbour of the selection.
  It is the join between a name in the tree and a mark in the grid: without it
  a reader counts rows, and counting rows is where a reader stops trusting the
  picture.
*/
'use strict';

import { $ } from './dom.js';
import { S, cell } from './state.js';

export function drawPaths(disp, rel) {
  const C = cell();
  const dots = $('#paths-dots');
  dots.innerHTML = disp.map((d, i) =>
    '<div class="pdot' + (d.id === S.selected ? ' sel' : '') + '">' + (i + 1) + '</div>').join('');

  const svg = $('#paths-svg');
  svg.style.top = dots.offsetTop + 'px';
  svg.setAttribute('height', disp.length * C);
  svg.setAttribute('viewBox', '0 0 62 ' + (disp.length * C));
  if (!rel.sel) { svg.innerHTML = ''; return; }

  const k = S.dispIndex.get(S.selected);
  const y = n => (n + 0.5) * C;
  const arcs = [];
  disp.forEach((d, i) => {
    if (i === k) return;
    const o = d.members.some(m => rel.dOut.has(m));
    const n = d.members.some(m => rel.dIn.has(m));
    if (!o && !n) return;
    const col = o && n ? 'var(--both)' : o ? 'var(--out)' : 'var(--in)';
    arcs.push('<path d="M 22 ' + y(i).toFixed(1) + ' C 42 ' + y(i).toFixed(1) +
      ', 42 ' + y(k).toFixed(1) + ', 60 ' + y(k).toFixed(1) +
      '" fill="none" stroke="' + col + '" stroke-width="1" opacity=".75"/>');
  });
  svg.innerHTML = arcs.join('');
}
