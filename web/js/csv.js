/*
  CSV, read and written the one way the group folder uses it.

  RFC 4180: commas, a header row, a field in double quotes when it holds a
  comma, a quote or a line break, and a doubled quote for a quote inside one.
  A unit rides in the header in square brackets — `altitude [km]` — and is
  split off here, once, so no view ever parses a header by hand.

  A file that does not parse is said: every problem comes back with its line,
  and the rows that did parse come back with it. Nothing is dropped silently.
*/
'use strict';

/** `{ head, rows, problems }` from the text of one CSV file. */
export function parseCsv(text) {
  const src = String(text || '').replace(/^﻿/, '');
  const out = [];
  const problems = [];
  let row = [], field = '', quoted = false, line = 1, rowLine = 1, i = 0;
  const endField = () => { row.push(field); field = ''; };
  const endRow = () => {
    endField();
    if (!(row.length === 1 && row[0].trim() === '')) out.push({ cells: row, line: rowLine });
    row = []; rowLine = line;
  };
  while (i < src.length) {
    const ch = src[i];
    if (quoted) {
      if (ch === '"') {
        if (src[i + 1] === '"') { field += '"'; i += 2; continue; }
        quoted = false; i++; continue;
      }
      if (ch === '\n') line++;
      field += ch; i++; continue;
    }
    if (ch === '"') {
      if (field.trim() === '') { field = ''; quoted = true; i++; continue; }
      field += ch; i++; continue;
    }
    if (ch === ',') { endField(); i++; continue; }
    if (ch === '\r') { i++; continue; }
    if (ch === '\n') { line++; endRow(); i++; continue; }
    field += ch; i++;
  }
  if (quoted) problems.push({ line: rowLine, msg: 'a quoted field is never closed' });
  if (field !== '' || row.length) endRow();
  if (!out.length) return { head: [], rows: [], lines: [], problems: problems.concat([{ line: 1, msg: 'the file is empty' }]) };
  const head = out[0].cells.map(h => h.trim());
  const rows = [], lines = [];
  for (const r of out.slice(1)) {
    if (r.cells.length !== head.length) {
      problems.push({ line: r.line, msg: 'has ' + r.cells.length + ' fields where the header has ' + head.length });
    }
    rows.push(head.map((_, k) => (r.cells[k] ?? '').trim()));
    lines.push(r.line);
  }
  return { head, rows, lines, problems };
}

/** The name and unit of a header: `altitude [km]` → `{ name: 'altitude', unit: 'km' }`. */
export function splitUnit(h) {
  const m = /^(.*?)\s*\[([^\]]*)\]\s*$/.exec(String(h || ''));
  return m ? { name: m[1].trim(), unit: m[2].trim() } : { name: String(h || '').trim(), unit: '' };
}

/** Each row as an object keyed by header name (unit stripped), with `_line` for messages. */
export function records(t) {
  const names = t.head.map(h => splitUnit(h).name);
  return t.rows.map((r, i) => {
    const o = { _line: t.lines[i] };
    names.forEach((n, k) => { o[n] = r[k]; });
    return o;
  });
}

/** A column's position by name, ignoring its unit; -1 when absent. */
export function column(t, name) {
  return t.head.findIndex(h => splitUnit(h).name === name);
}

/** A number from a cell, or null when the cell is blank or is not one. */
export function num(cell) {
  const s = String(cell ?? '').trim();
  if (s === '') return null;
  const v = Number(s);
  return Number.isFinite(v) ? v : null;
}

/** CSV text from a header and rows, quoting only where it must. */
export function toCsv(head, rows) {
  const q = v => {
    const s = String(v ?? '');
    return /[",\n\r]/.test(s) ? '"' + s.replace(/"/g, '""') + '"' : s;
  };
  return [head].concat(rows).map(r => r.map(q).join(',')).join('\n') + '\n';
}
