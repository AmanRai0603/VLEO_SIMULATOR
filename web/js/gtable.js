/*
  A table to fill in: the CSV files a node keeps (results, evidence, sources,
  equations, pictures), edited as a grid.

  Every cell is a text box. A row is added or taken away with a button, and a
  block copied from a spreadsheet (Excel, Google Sheets, LibreOffice) is pasted
  in whole — tab-separated, as every spreadsheet copies — so nobody retypes a
  table they already have. What changes is handed back as `(head, rows)`;
  the caller writes the CSV.
*/
'use strict';

import { esc } from './dom.js';
import { parseCsv } from './csv.js';

/**
 * Put an editable table into `el`. `opts.head` is the header (fixed when
 * `opts.fixedHead`), `opts.rows` the rows, `opts.onChange(head, rows)` is called
 * after every change, `opts.hint` a header cell's tooltip.
 */
export function mountTable(el, opts) {
  let head = (opts.head || []).slice();
  let rows = (opts.rows || []).map(r => head.map((_, k) => r[k] ?? ''));
  const fixed = !!opts.fixedHead;
  const changed = () => opts.onChange(head.slice(), rows.map(r => r.slice()));
  const draw = () => {
    el.innerHTML = '<div class="ri-wrap"><table class="fx gtable te"><thead><tr>' +
      head.map((h, k) => '<th>' + (fixed ? '<span title="' + esc(opts.hint ? opts.hint(h) || '' : '') + '">' + esc(h) + '</span>'
        : '<input class="gs-in te-h" data-k="' + k + '" value="' + esc(h) + '" aria-label="column ' + (k + 1) + ' name">') + '</th>').join('') +
      (fixed ? '' : '<th><button class="ctl small" type="button" data-te="addcol" title="add a column">+ column</button></th>') + '<th></th></tr></thead><tbody>' +
      rows.map((r, i) => '<tr>' + r.map((c, k) => '<td><input class="gs-in te-c" data-r="' + i + '" data-k="' + k + '" value="' + esc(c) +
        '" aria-label="' + esc(head[k] || 'column ' + (k + 1)) + ', row ' + (i + 1) + '"></td>').join('') + (fixed ? '' : '<td></td>') +
        '<td><button class="ctl small gs-x" type="button" data-te="rm" data-r="' + i + '" aria-label="remove row ' + (i + 1) + '" title="remove this row">×</button></td></tr>').join('') +
      '</tbody></table></div>' +
      '<p class="te-acts"><button class="ctl small" type="button" data-te="add">add a row</button> ' +
      '<button class="ctl small" type="button" data-te="paste">paste from a spreadsheet…</button> <span class="muted small">' + rows.length + ' row(s)</span></p>' +
      '<div class="te-paste" hidden><p class="muted small">Copy the cells in your spreadsheet — with or without the header row — and paste them here. ' +
      (fixed ? 'The columns must be in the order shown above.' : 'A header row replaces the columns.') + '</p>' +
      '<textarea class="gnote te-in" rows="6" aria-label="pasted cells"></textarea><p>' +
      '<button class="ctl small" type="button" data-te="replace">replace the rows</button> <button class="ctl small" type="button" data-te="append">add to the rows</button></p></div>';
  };
  el.addEventListener('change', e => {
    const t = e.target;
    if (t.classList.contains('te-c')) { rows[+t.dataset.r][+t.dataset.k] = t.value; changed(); }
    else if (t.classList.contains('te-h')) { head[+t.dataset.k] = t.value.trim(); changed(); }
  });
  el.addEventListener('click', e => {
    const b = e.target.closest('[data-te]');
    if (!b) return;
    const act = b.dataset.te;
    if (act === 'add') { rows.push(head.map(() => '')); draw(); changed(); el.querySelector('tbody tr:last-child input').focus(); }
    else if (act === 'rm') { rows.splice(+b.dataset.r, 1); draw(); changed(); }
    else if (act === 'addcol') { head.push('column_' + (head.length + 1)); rows.forEach(r => r.push('')); draw(); changed(); }
    else if (act === 'paste') { const p = el.querySelector('.te-paste'); p.hidden = !p.hidden; if (!p.hidden) el.querySelector('.te-in').focus(); }
    else if (act === 'replace' || act === 'append') {
      const got = pasted(el.querySelector('.te-in').value);
      if (!got.length) return;
      let body = got;
      const first = got[0].map(c => c.trim().toLowerCase());
      const isHead = first.length && first.every(c => c && isNaN(Number(c)));
      if (isHead) {
        body = got.slice(1);
        if (!fixed && act === 'replace') head = got[0].map(c => c.trim());
      }
      const width = head.length;
      const fit = body.map(r => Array.from({ length: width }, (_, k) => (r[k] ?? '').trim()));
      rows = act === 'replace' ? fit : rows.concat(fit);
      draw();
      changed();
    }
  });
  draw();
}

/** Cells copied from a spreadsheet (tab-separated) or typed as CSV. */
export function pasted(text) {
  const t = String(text || '').replace(/\r/g, '').replace(/\n+$/, '');
  if (!t.trim()) return [];
  if (t.includes('\t')) return t.split('\n').map(l => l.split('\t'));
  const p = parseCsv(t);
  return [p.head].concat(p.rows);
}
