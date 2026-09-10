/*
  The four helpers every other module uses, and nothing else.

  It is here so that no view module has to reach for the document root to find
  a shortcut, and so that escaping is one function rather than one per file —
  the second copy of an escape function is the one that gets forgotten.
*/
'use strict';

export const $  = (s, r = document) => r.querySelector(s);
export const $$ = (s, r = document) => Array.from(r.querySelectorAll(s));

/** Text that will be put inside HTML. Never optional, never inlined. */
export const esc = s => String(s == null ? '' : s)
  .replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');

/** A number for a person to read. Six significant figures, then exponent. */
export function fmt(v) {
  if (v == null || !isFinite(v)) return '—';
  const a = Math.abs(v);
  if (a === 0) return '0';
  if (a >= 1e6 || a < 1e-3) return v.toExponential(3);
  return String(Number(v.toPrecision(6)));
}

/** `n thing` / `n things`, because the alternative is `1 boxes`. */
export const plural = (n, one, many) => n + ' ' + (n === 1 ? one : (many || one + 's'));
