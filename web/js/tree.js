/*
  The tree column.

  It is drawn inside the figure's grid rather than beside it, so that it cannot
  drift out of alignment with the matrix as branches open. The gutter carries
  the hierarchy; the fill carries the relation to the selection; the left bar
  carries the group's tone.
*/
'use strict';

import { $, esc } from './dom.js';
import { S, subtreeNodes, isSeeded } from './state.js';
import { pillClass, ancestorAt } from './display.js';

export function drawTree(disp, rel) {
  const selIdx = S.dispIndex.get(S.selected);

  // The lit branch: the path from the root to what you selected.
  const lit = new Set();
  {
    let d = disp.find(x => x.id === S.selected);
    while (d) { lit.add(d.id); d = disp.find(x => x.id === d.parent); }
  }

  $('#tree').innerHTML = disp.map((d, i) => {
    let g = '';
    for (let k = 0; k < d.depth; k++) {
      const isLast = k === d.depth - 1;
      const on = isLast ? lit.has(d.id) : (lit.has(ancestorAt(disp, i, k)) && i <= selIdx);
      const cls = isLast ? (d.last ? 'elbow' : 'tee') : (d.bars[k] ? 'line' : '');
      g += '<i class="' + cls + (cls && on ? ' lit' : '') + '"></i>';
    }
    const tri = d.kind === 'group' && d.hasKids
      ? '<span class="tri">' + (d.open ? '▾' : '▸') + '</span>' : '';
    const seeded = d.kind === 'node' && isSeeded(d.row);
    const txt = '<span class="txt' + (seeded ? ' seeded' : '') + '">' + esc(d.label) + '</span>';
    const n = d.kind === 'group' && !d.open ? '<span class="tri">' + d.members.length + '</span>' : '';
    const title = d.kind === 'node'
      ? d.row.id + (d.row.question ? ' — ' + d.row.question : ' — seeded, not yet specified')
      : d.label + ' — ' + subtreeNodes(d.id).length + ' rows, owner ' + d.owner;
    return '<div class="trow" data-i="' + i + '">' +
      '<span class="gutter">' + g + '</span>' +
      '<span class="pill t-' + esc(d.tone) + pillClass(d, rel) +
        (d.crosses ? ' crossing' : '') + '" title="' + esc(title) + '">' +
        tri + txt + n +
      '</span></div>';
  }).join('');
}
